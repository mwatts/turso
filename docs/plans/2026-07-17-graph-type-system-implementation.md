# Graph type system implementation plan

> **Status as of 2026-07-21:** this document predates the graph frontend
> delivery on `feature/graph-frontend` and is retained as an archival plan.
> Since it was written: Ladybug/Kuzu was removed from the corpus, the deep
> corpus grew to ~10k identities, `PreparedSource` + `FrontendCompiler`
> replaced the `ReprepareRecipe` naming, and `__turso_graph_expand`
> (GraphExpand) shipped. Where this text and the code disagree, the code and
> `graph/test-results/REPORT.md` are authoritative.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the Cypher graph frontend full, verified type coverage over
every value Turso's storage/schema layer can hold — BLOB, `CREATE TYPE`
custom scalars, STRUCT, UNION, native arrays, vector functions, and FTS
functions — by extending `graph/ir::ValueType` and wiring a real,
schema-backed `GraphCatalogSnapshot`/`RelationalCatalogSnapshot`
implementation in place of today's hardcoded testkit stub.

**Architecture:** `graph/ir` gains four new `ValueType` variants
(`Custom`, `Struct`, `Union`, `Vector`) plus a `fields: Vec<String>` on
`Expression::Property` for nested struct/union field reads. A new
`graph/frontend/src/schema_catalog.rs` implements both catalog traits by
reading `core::Connection::current_schema()` directly — no PRAGMA
string-parsing, no parallel type model. Column classification is not
reimplemented: the exact logic already inline in
`Statement::get_column_type_info` is extracted to `Schema::classify_column`
in `core/schema.rs`, and both `Statement::get_column_type_info` and the new
`SchemaCatalog::property()` call it. Map literals (`{field: value}`) gain a
general expression-grammar position so `CREATE`/`SET` can construct
STRUCT/UNION values via `struct_pack`/`union_value`, and nested property
reads (`n.address.city`) lower to the same SQL dot-chain syntax core's own
struct/union column resolver already accepts.

**Tech Stack:** Rust workspace crates `core`, `graph/ir`, `graph/cypher`,
`graph/frontend`, `graph/testkit`. Pest grammar (`graph/cypher/src/cypher.pest`).
No new external dependencies.

## Global Constraints

- `graph/ir`, `graph/cypher`, `graph/runtime` MUST NOT depend on
  `turso_core` (verified zero-dependency architecture rule from
  `docs/plans/2026-07-17-graph-type-system-design.md`). `graph/frontend` is
  the only graph crate allowed to depend on `turso_core`.
- Reuse an existing `core` type before adding a parallel one. Every new type
  introduced by this plan documents, in its task, why direct reuse was not
  possible (crate-dependency boundary) rather than assuming it.
- New failure modes surface as `BindError` at bind time, not as opaque SQL
  errors at execution time, except the two existing deliberate `Any`
  pass-through paths (non-STRICT columns; unknown function names) — unchanged
  by this plan.
- `cargo build` (never `--release`). `cargo fmt` and
  `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
  must pass before every commit that changes non-test code.
- Conventional commits (`type(scope): message`), signed (`git commit -S`).
- Never touch files outside this plan's task list — another agent is active
  elsewhere in this repository.

---

## File structure

- `core/schema.rs` — **modify**: add `Schema::classify_column`.
- `core/statement.rs` — **modify**: `get_column_type_info` delegates to it.
- `graph/ir/src/expression.rs` — **modify**: new `ValueType` variants, new
  `VectorKind` enum, `fields` on `Expression::Property`.
- `graph/frontend/src/schema_catalog.rs` — **create**: `SchemaCatalog`, the
  production `GraphCatalogSnapshot` + `RelationalCatalogSnapshot` impl.
- `graph/frontend/src/catalog.rs` — **modify**: additive STRICT/custom-types
  gating check in `register_graph`.
- `graph/testkit/src/runner.rs` — **modify**: swap hardcoded `Catalog` stub
  for `SchemaCatalog`.
- `graph/cypher/src/cypher.pest`, `graph/cypher/src/ast.rs`,
  `graph/cypher/src/parser.rs` — **modify**: general-position map literal.
- `graph/frontend/src/binder.rs` — **modify**: map-literal binding for
  struct/union mutation targets; nested property-chain binding.
- `graph/frontend/src/lowering.rs` — **modify**: `Map` lowering to
  `struct_pack`/`union_value`; nested `Property.fields` lowering.
- `graph/frontend/src/functions.rs` — **create**: typed function registry
  (`vector_*`, `struct_pack`/`union_value`/`union_tag`, `fts_*`).
- `graph/frontend/tests/type_system_fixtures.rs` — **create**: STRICT-source
  end-to-end fixtures.

---

### Task 1: Extract `Schema::classify_column` from `Statement::get_column_type_info`

**Files:**
- Modify: `core/schema.rs` (add method near `Schema::resolve_type`, ~line 1010)
- Modify: `core/statement.rs:1112-1189`
- Test: `core/schema.rs` (new `#[cfg(test)]` block near the new method)

**Interfaces:**
- Produces: `impl Schema { pub fn classify_column(&self, column: &Column, is_strict: bool) -> ColumnTypeInfo }` —
  the exact classification logic every later task's `SchemaCatalog` and
  `register_graph` gating check calls. `ColumnTypeInfo`/`ColumnTypeKind`
  stay defined in `core/statement.rs` (public API, re-exported at
  `turso_core::{ColumnTypeInfo, ColumnTypeKind}` per `core/lib.rs:166`);
  `Schema::classify_column` references them via `crate::statement::{ColumnTypeInfo, ColumnTypeKind}`.

- [ ] **Step 1: Write the failing test**

Add to `core/schema.rs`, inside a new `#[cfg(test)] mod classify_column_tests`
placed after the existing `Schema` impl block (follow the file's existing
`#[cfg(test)]` convention — check the bottom of the file for the module name
pattern already in use before naming this one):

```rust
#[cfg(test)]
mod classify_column_tests {
    use super::*;
    use crate::statement::ColumnTypeKind;
    use crate::{Connection, Database, DatabaseOpts, MemoryIO, OpenFlags, SqliteDialect};
    use std::sync::Arc;

    fn connect(strict_custom_types: bool) -> Arc<Connection> {
        let io = Arc::new(MemoryIO::new());
        Database::open_file_with_flags(
            io,
            ":memory:classify-column",
            OpenFlags::default(),
            DatabaseOpts::new().with_custom_types(strict_custom_types),
            None,
            Arc::new(SqliteDialect),
        )
        .expect("open database")
        .connect()
        .expect("connect")
    }

    #[test]
    fn classifies_builtin_column_as_builtin() {
        let connection = connect(false);
        connection
            .execute("CREATE TABLE t(id INTEGER, name TEXT)")
            .expect("create table");
        let schema = connection.current_schema();
        let table = schema.get_table("t").expect("table exists");
        let (_, column) = table.get_column("name").expect("column exists");
        let info = schema.classify_column(column, table.is_strict());
        assert_eq!(info.kind, ColumnTypeKind::Builtin);
        assert_eq!(info.base_type, None);
        assert_eq!(info.array_dimensions, 0);
    }

    #[test]
    fn classifies_struct_column_under_strict_custom_types() {
        let connection = connect(true);
        connection
            .execute(
                "CREATE TYPE point AS STRUCT(x INTEGER, y INTEGER); \
                 CREATE TABLE shapes(id INTEGER PRIMARY KEY, origin point) STRICT",
            )
            .expect("create struct type and table");
        let schema = connection.current_schema();
        let table = schema.get_table("shapes").expect("table exists");
        let (_, column) = table.get_column("origin").expect("column exists");
        let info = schema.classify_column(column, table.is_strict());
        assert_eq!(info.kind, ColumnTypeKind::Struct);
        assert_eq!(info.declared_name, "point");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p turso_core classify_column_tests -- --nocapture`
Expected: compile error — `no method named 'classify_column' found for
struct 'Schema'`.

- [ ] **Step 3: Implement `Schema::classify_column`**

Add to `core/schema.rs`, in the `impl Schema` block that already contains
`resolve_type` (~line 993), immediately after `resolve_type_unchecked`:

```rust
/// Classifies one table column's declared type into a [`crate::statement::ColumnTypeInfo`],
/// resolving `CREATE TYPE`/`CREATE DOMAIN` chains against this schema's type
/// registry. Shared by [`crate::statement::Statement::get_column_type_info`]
/// (SQL result columns) and the graph frontend's `SchemaCatalog` (Cypher
/// property resolution) — one classification implementation, two callers.
pub fn classify_column(
    &self,
    column: &Column,
    is_strict: bool,
) -> crate::statement::ColumnTypeInfo {
    use crate::statement::{ColumnTypeInfo, ColumnTypeKind};

    let declared_name = column.ty_str.clone();
    let array_dimensions = column.array_dimensions();
    let resolved = self.resolve_type(&declared_name, is_strict).ok().flatten();
    let (base_type, kind) = match resolved {
        Some(resolved) => {
            let leaf = resolved.leaf();
            let kind = if leaf.is_struct() {
                ColumnTypeKind::Struct
            } else if leaf.is_union() {
                ColumnTypeKind::Union
            } else if leaf.is_domain {
                ColumnTypeKind::Domain
            } else {
                ColumnTypeKind::Custom
            };
            (Some(resolved.primitive.to_uppercase()), kind)
        }
        None => (None, ColumnTypeKind::Builtin),
    };
    ColumnTypeInfo {
        declared_name,
        array_dimensions,
        base_type,
        kind,
    }
}
```

Then replace the table-column branch of `Statement::get_column_type_info`
(`core/statement.rs:1128-1179`) — everything from
`if let turso_parser::ast::Expr::Column { ... }` through the matching closing
`}` before the `// Not a table column:` comment — with:

```rust
        if let turso_parser::ast::Expr::Column {
            table,
            column: column_idx,
            ..
        } = &column.expr
        {
            let Some((_, table_ref)) = self
                .program
                .table_references
                .find_table_by_internal_id(*table)
            else {
                return Ok(None);
            };
            let Some(table_column) = table_ref.get_column_at(*column_idx) else {
                return Ok(None);
            };
            let schema = self.program.connection.schema.read();
            let info = schema.classify_column(table_column, table_ref.is_strict());
            drop(schema);
            return Ok(Some(info));
        }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p turso_core classify_column_tests -- --nocapture`
Expected: both tests PASS.

Run: `cargo test -p turso_core get_column_type_info`
Expected: all pre-existing tests for the delegating caller still PASS
(regression gate for the extraction — this must not change behavior).

- [ ] **Step 5: Commit**

```bash
cargo fmt
cargo clippy -p turso_core --all-features --all-targets -- --deny=warnings
git add core/schema.rs core/statement.rs
git commit -S -m "core: extract Schema::classify_column from get_column_type_info

Statement::get_column_type_info's table-column classification logic
is the exact column-to-logical-type mapping the graph frontend's
upcoming SchemaCatalog needs. Factor it onto Schema so both callers
share one implementation instead of the graph crate re-deriving
STRUCT/UNION/DOMAIN/CUSTOM classification from scratch."
```

---

### Task 2: Extend `ir::ValueType` with `Custom`, `Struct`, `Union`, `Vector`

**Files:**
- Modify: `graph/ir/src/expression.rs`
- Test: `graph/ir/src/expression.rs` (new `#[cfg(test)]` block at end of file)

**Interfaces:**
- Consumes: nothing (leaf crate, no dependency on Task 1).
- Produces:
  ```rust
  pub enum VectorKind { Float32Dense, Float64Dense, Float32Sparse, Float1Bit, Float8 }
  pub enum ValueType {
      // ...existing 10 variants unchanged...
      Custom { name: String, base: Box<ValueType> },
      Struct(Vec<(String, ValueType)>),
      Union(Vec<(String, ValueType)>),
      Vector(VectorKind, Option<u32>),
  }
  ```
  Every later task (3, 8, 9, 10, 11, 12) constructs/matches these exact
  shapes.

**Why not reuse `core::vector::vector_types::VectorType` directly:**
`graph/ir` has zero `turso_core` dependency (Global Constraints). `VectorKind`
is a deliberate, minimal local mirror of `core::VectorType`'s 5 variants
(`Float32Dense, Float64Dense, Float32Sparse, Float1Bit, Float8`, verified in
`core/vector/vector_types.rs:9-15`), not a reimplementation of any behavior —
it carries no logic, only the closed set of vector encodings Turso's
`vector32`/`vector64`/etc. functions produce. `core::VectorType` derives only
`Debug, Clone, PartialEq, Copy` (no `Eq`); `VectorKind` additionally derives
`Eq` since `ValueType` itself derives `Eq` and needs every variant to.

- [ ] **Step 1: Write the failing test**

Append to `graph/ir/src/expression.rs`:

```rust
#[cfg(test)]
mod value_type_tests {
    use super::*;

    #[test]
    fn struct_and_union_value_types_compare_by_field_shape() {
        let a = ValueType::Struct(vec![
            ("x".to_owned(), ValueType::Integer),
            ("y".to_owned(), ValueType::Integer),
        ]);
        let b = ValueType::Struct(vec![
            ("x".to_owned(), ValueType::Integer),
            ("y".to_owned(), ValueType::Integer),
        ]);
        assert_eq!(a, b);

        let point = ValueType::Custom {
            name: "point".to_owned(),
            base: Box::new(ValueType::Bytes),
        };
        assert_eq!(point.clone(), point);

        let tagged = ValueType::Union(vec![("ok".to_owned(), ValueType::Text)]);
        assert_ne!(a, tagged);
    }

    #[test]
    fn vector_value_type_carries_kind_and_optional_dims() {
        let dense = ValueType::Vector(VectorKind::Float32Dense, Some(3));
        let unknown_dims = ValueType::Vector(VectorKind::Float32Dense, None);
        assert_ne!(dense, unknown_dims);
        assert_eq!(VectorKind::Float32Dense, VectorKind::Float32Dense);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p turso_graph_ir value_type_tests -- --nocapture`
Expected: compile error — `no variant named 'Struct' found for enum
'ValueType'` (and similarly for `Custom`, `Union`, `Vector`, `VectorKind`).

- [ ] **Step 3: Implement the new variants**

In `graph/ir/src/expression.rs`, replace the current `ValueType` enum
(lines 3-16) with:

```rust
/// Frontend-neutral value categories used during binding and planning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValueType {
    Any,
    Boolean,
    Integer,
    Real,
    Text,
    Bytes,
    Node,
    Relationship,
    Path,
    List(Box<ValueType>),
    /// A `CREATE TYPE name BASE <primitive> ENCODE ... DECODE ...` scalar.
    /// `base` is the underlying storage primitive (e.g. `Bytes` for a BLOB
    /// custom type).
    Custom { name: String, base: Box<ValueType> },
    /// A `CREATE TYPE name AS STRUCT(...)` composite, fields in declared order.
    Struct(Vec<(String, ValueType)>),
    /// A `CREATE TYPE name AS UNION(...)` tagged union, variants in declared order.
    Union(Vec<(String, ValueType)>),
    /// The result of a typed vector function call (`vector32`, `vector64`, ...).
    /// Dims are known only when statically determinable from the call site —
    /// never from a column declaration, since no schema-level VECTOR column
    /// type exists (see `docs/plans/2026-07-17-graph-type-system-design.md`).
    Vector(VectorKind, Option<u32>),
}

/// Mirrors `core::vector::vector_types::VectorType`'s 5 encodings. Kept local
/// (not reused from `core`) because `graph/ir` has zero `turso_core`
/// dependency; this enum carries no behavior, only the closed set of vector
/// encodings the typed function registry (Task 11) can produce.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VectorKind {
    Float32Dense,
    Float64Dense,
    Float32Sparse,
    Float1Bit,
    Float8,
}
```

- [ ] **Step 4: Fix downstream construction sites that changed shape**

`Expression::Property` gains a `fields` member in Task 10, not this task —
no other file constructs `ValueType::Struct`/`Union`/`Custom`/`Vector` yet,
so no other files need changes for this step. Confirm with:

```bash
grep -rn "ValueType::" graph/frontend/src graph/testkit/src graph/ir/src | grep -v expression.rs
```

Expected: every match already compiles against the untouched existing 10
variants (`Any`, `Boolean`, `Integer`, `Real`, `Text`, `Bytes`, `Node`,
`Relationship`, `List`) — adding variants to a non-`#[non_exhaustive]` enum
only breaks exhaustive `match` arms with no wildcard; none exist outside
`graph/ir/src/expression.rs` itself (verified: every existing match/construct
site either builds a specific variant or falls through a `_ =>` arm).

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p turso_graph_ir`
Expected: all tests PASS, including the two new ones.

- [ ] **Step 6: Commit**

```bash
cargo fmt
cargo clippy -p turso_graph_ir --all-features --all-targets -- --deny=warnings
git add graph/ir/src/expression.rs
git commit -S -m "graph/ir: add Custom, Struct, Union, Vector to ValueType

Extends the frontend-neutral value model to cover CREATE TYPE
custom scalars, STRUCT/UNION composites, and typed vector function
results, so the binder and catalog can express Turso's full type
system instead of falling back to Any."
```

---

### Task 3: `SchemaCatalog` — production catalog backed by `core::Schema`

**Files:**
- Create: `graph/frontend/src/schema_catalog.rs`
- Modify: `graph/frontend/src/lib.rs` (add `mod schema_catalog; pub use schema_catalog::SchemaCatalog;` — check the existing `mod`/`pub use` list at the top of the file and match its ordering/style before inserting)
- Test: `graph/frontend/src/schema_catalog.rs` (`#[cfg(test)]` module)

**Interfaces:**
- Consumes: `binder::{CatalogEntity, GraphCatalogSnapshot, ResolvedProperty}`
  (`graph/frontend/src/binder.rs:8-35`), `lowering::{RelationalCatalogSnapshot,
  NodeTableLayout, RelationshipTableLayout}` (`graph/frontend/src/lowering.rs:8-30`),
  `catalog::{RegisteredGraph, RegisteredNodeSource, RegisteredRelationshipSource}`
  (`graph/frontend/src/catalog.rs:51-77`), `core::{Connection, Schema, Table,
  Column, Affinity}`, `core::statement::ColumnTypeKind`,
  `Schema::classify_column` (Task 1).
- Produces:
  ```rust
  pub struct SchemaCatalog {
      pub fn new(connection: Arc<turso_core::Connection>, graph: RegisteredGraph) -> Self
  }
  impl GraphCatalogSnapshot for SchemaCatalog { /* ... */ }
  impl RelationalCatalogSnapshot for SchemaCatalog { /* ... */ }
  ```
  Task 5 constructs this from a real `CREATE TABLE` schema instead of the
  testkit's hand-rolled stub. Task 6/9/11/13 fixtures exercise it directly.

**PropertyId/LabelId/RelationshipTypeId assignment scheme:** 1-based index —
`PropertyId` is `1 + Table::get_column(name).0` (the `usize` index
`get_column` returns); `LabelId`/`RelationshipTypeId` are `1 + <index into
graph.node_sources / graph.relationship_sources>`. This matches the
convention already implicit in `graph/testkit/src/runner.rs`'s stub
(`id→1, name→2, age→3`, i.e. 1-based column position) and in
`graph/frontend/src/mutation.rs:676-682`'s test fixture, so `SchemaCatalog`
reproduces today's identities exactly for the existing "people"/"id, name,
age" fixture — required for Task 5's regression proof.

- [ ] **Step 1: Write the failing test**

Create `graph/frontend/src/schema_catalog.rs`:

```rust
use std::sync::Arc;

use turso_core::{Affinity, Column, Connection, Table};
use turso_graph_ir as ir;

use crate::binder::{CatalogEntity, GraphCatalogSnapshot, ResolvedProperty};
use crate::catalog::{RegisteredGraph, RegisteredNodeSource, RegisteredRelationshipSource};
use crate::lowering::{NodeTableLayout, RelationalCatalogSnapshot, RelationshipTableLayout};

/// Production catalog snapshot backed directly by `core::Schema` — no PRAGMA
/// string-parsing, no parallel type model. Column classification reuses
/// `Schema::classify_column` (`core/schema.rs`), the same function
/// `Statement::get_column_type_info` uses for SQL result columns.
pub struct SchemaCatalog {
    connection: Arc<Connection>,
    graph: RegisteredGraph,
}

impl SchemaCatalog {
    pub fn new(connection: Arc<Connection>, graph: RegisteredGraph) -> Self {
        Self { connection, graph }
    }

    fn node_source_entry(&self) -> Option<&RegisteredNodeSource> {
        self.graph.node_sources.first()
    }

    fn relationship_source_entry(&self) -> Option<&RegisteredRelationshipSource> {
        self.graph.relationship_sources.first()
    }

    fn table_for(&self, entity: CatalogEntity) -> Option<Arc<Table>> {
        let table_name = match entity {
            CatalogEntity::Node => &self.node_source_entry()?.table,
            CatalogEntity::Relationship => &self.relationship_source_entry()?.table,
        };
        self.connection.current_schema().get_table(table_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{
        GraphRegistration, NodeSourceRegistration, RelationshipSourceRegistration,
    };
    use std::sync::Arc;
    use turso_core::{Database, DatabaseOpts, MemoryIO, OpenFlags, SqliteDialect};
    use turso_graph_ir::GraphId;

    fn connect(strict_custom_types: bool) -> Arc<Connection> {
        let io = Arc::new(MemoryIO::new());
        Database::open_file_with_flags(
            io,
            ":memory:schema-catalog",
            OpenFlags::default(),
            DatabaseOpts::new().with_custom_types(strict_custom_types),
            None,
            Arc::new(SqliteDialect),
        )
        .expect("open database")
        .connect()
        .expect("connect")
    }

    fn registered_social_graph(connection: &Arc<Connection>) -> RegisteredGraph {
        connection
            .execute(
                "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, age INTEGER); \
                 CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);",
            )
            .expect("create sources");
        crate::catalog::register_graph(
            connection,
            &GraphRegistration {
                name: "social".to_owned(),
                node_sources: vec![NodeSourceRegistration {
                    name: "Person".to_owned(),
                    table: "people".to_owned(),
                    identity_column: "id".to_owned(),
                }],
                relationship_sources: vec![RelationshipSourceRegistration {
                    name: "KNOWS".to_owned(),
                    table: "relationships".to_owned(),
                    identity_column: "id".to_owned(),
                    start_column: "src".to_owned(),
                    end_column: "dst".to_owned(),
                    start_node_source: "Person".to_owned(),
                    end_node_source: "Person".to_owned(),
                }],
            },
        )
        .expect("register graph")
    }

    #[test]
    fn resolves_id_name_age_matching_testkit_stub_identities() {
        let connection = connect(false);
        let graph = registered_social_graph(&connection);
        let graph_id = graph.id;
        let catalog = SchemaCatalog::new(connection, graph);

        let id = catalog
            .property(graph_id, CatalogEntity::Node, "id")
            .expect("id resolves");
        assert_eq!(id.id, ir::PropertyId::new(1).unwrap());
        assert_eq!(id.value_type, ir::ValueType::Integer);
        assert_eq!(id.nullability, ir::Nullability::NonNull);

        let name = catalog
            .property(graph_id, CatalogEntity::Node, "name")
            .expect("name resolves");
        assert_eq!(name.id, ir::PropertyId::new(2).unwrap());
        assert_eq!(name.value_type, ir::ValueType::Text);
        assert_eq!(name.nullability, ir::Nullability::Nullable);
    }
}
```

**Interfaces used from `catalog.rs` that must be `pub`:** `GraphRegistration`,
`NodeSourceRegistration`, `RelationshipSourceRegistration`, `register_graph`
are already `pub fn`/`pub struct` in `graph/frontend/src/catalog.rs` (used
identically by its own `#[cfg(test)] mod tests`, verified at
`graph/frontend/src/catalog.rs:723-741`) — no visibility changes needed.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p turso_graph_frontend schema_catalog -- --nocapture`
Expected: compile error — `SchemaCatalog` does not implement
`GraphCatalogSnapshot` (no `property` method).

- [ ] **Step 3: Implement `GraphCatalogSnapshot` and `RelationalCatalogSnapshot`**

Append to `graph/frontend/src/schema_catalog.rs`, before the `#[cfg(test)]`
module:

```rust
fn affinity_value_type(affinity: Affinity) -> ir::ValueType {
    match affinity {
        Affinity::Integer => ir::ValueType::Integer,
        Affinity::Real => ir::ValueType::Real,
        Affinity::Text => ir::ValueType::Text,
        Affinity::Numeric => ir::ValueType::Any,
        Affinity::Blob => ir::ValueType::Bytes,
    }
}

fn primitive_value_type(primitive: &str) -> ir::ValueType {
    match primitive.to_ascii_uppercase().as_str() {
        "INTEGER" => ir::ValueType::Integer,
        "REAL" => ir::ValueType::Real,
        "TEXT" => ir::ValueType::Text,
        "BLOB" => ir::ValueType::Bytes,
        _ => ir::ValueType::Any,
    }
}

fn wrap_array(mut element: ir::ValueType, dimensions: u32) -> ir::ValueType {
    for _ in 0..dimensions {
        element = ir::ValueType::List(Box::new(element));
    }
    element
}

impl SchemaCatalog {
    /// Resolves a named `CREATE TYPE`/`CREATE DOMAIN` to a `ValueType`,
    /// recursing into STRUCT fields and UNION variants. Falls back to `Any`
    /// only when the type registry has no entry for `type_name` under the
    /// caller's strictness mode (mirrors `Schema::classify_column`'s own
    /// `None => Builtin` fallback rather than inventing a stricter failure).
    fn resolve_named_type(&self, schema: &turso_core::Schema, type_name: &str, is_strict: bool) -> ir::ValueType {
        let Some(resolved) = schema.resolve_type(type_name, is_strict).ok().flatten() else {
            return ir::ValueType::Any;
        };
        let leaf = resolved.leaf();
        if leaf.is_struct() {
            let fields = leaf
                .struct_def()
                .expect("is_struct implies struct_def")
                .fields
                .iter()
                .map(|field| {
                    (
                        field.name.clone(),
                        self.resolve_named_type(schema, &field.type_name, is_strict),
                    )
                })
                .collect();
            ir::ValueType::Struct(fields)
        } else if leaf.is_union() {
            let variants = leaf
                .union_def()
                .expect("is_union implies union_def")
                .variants
                .iter()
                .map(|variant| {
                    (
                        variant.tag_name.clone(),
                        self.resolve_named_type(schema, &variant.type_name, is_strict),
                    )
                })
                .collect();
            ir::ValueType::Union(variants)
        } else if leaf.is_domain {
            primitive_value_type(&resolved.primitive)
        } else {
            ir::ValueType::Custom {
                name: type_name.to_owned(),
                base: Box::new(primitive_value_type(&resolved.primitive)),
            }
        }
    }

    fn column_value_type(&self, schema: &turso_core::Schema, column: &Column, is_strict: bool) -> ir::ValueType {
        use turso_core::ColumnTypeKind;

        let info = schema.classify_column(column, is_strict);
        let scalar = match info.kind {
            ColumnTypeKind::Builtin | ColumnTypeKind::Domain => affinity_value_type(column.affinity()),
            ColumnTypeKind::Custom => ir::ValueType::Custom {
                name: info.declared_name.clone(),
                base: Box::new(affinity_value_type(column.affinity())),
            },
            ColumnTypeKind::Struct | ColumnTypeKind::Union => {
                self.resolve_named_type(schema, &info.declared_name, is_strict)
            }
        };
        wrap_array(scalar, column.array_dimensions())
    }
}

impl GraphCatalogSnapshot for SchemaCatalog {
    fn node_source(&self, graph: ir::GraphId) -> Option<ir::SourceTableId> {
        (graph == self.graph.id)
            .then(|| self.node_source_entry())
            .flatten()
            .map(|source| source.id)
    }

    fn relationship_source(&self, graph: ir::GraphId) -> Option<ir::SourceTableId> {
        (graph == self.graph.id)
            .then(|| self.relationship_source_entry())
            .flatten()
            .map(|source| source.id)
    }

    fn label(&self, graph: ir::GraphId, name: &str) -> Option<ir::LabelId> {
        if graph != self.graph.id {
            return None;
        }
        let index = self
            .graph
            .node_sources
            .iter()
            .position(|source| source.name == name)?;
        ir::LabelId::new((index as u32) + 1).ok()
    }

    fn relationship_type(&self, graph: ir::GraphId, name: &str) -> Option<ir::RelationshipTypeId> {
        if graph != self.graph.id {
            return None;
        }
        let index = self
            .graph
            .relationship_sources
            .iter()
            .position(|source| source.name == name)?;
        ir::RelationshipTypeId::new((index as u32) + 1).ok()
    }

    fn property(
        &self,
        graph: ir::GraphId,
        entity: CatalogEntity,
        name: &str,
    ) -> Option<ResolvedProperty> {
        if graph != self.graph.id {
            return None;
        }
        let table = self.table_for(entity)?;
        let (index, column) = table.get_column(name)?;
        let schema = self.connection.current_schema();
        let value_type = self.column_value_type(&schema, column, table.is_strict());
        let nullability = if column.explicit_notnull() {
            ir::Nullability::NonNull
        } else {
            ir::Nullability::Nullable
        };
        Some(ResolvedProperty {
            id: ir::PropertyId::new((index as u32) + 1).ok()?,
            value_type,
            nullability,
        })
    }
}

impl RelationalCatalogSnapshot for SchemaCatalog {
    fn node_layout(&self, source: ir::SourceTableId) -> Option<NodeTableLayout> {
        let entry = self.node_source_entry().filter(|entry| entry.id == source)?;
        Some(NodeTableLayout {
            table: entry.table.clone(),
            identity_column: entry.identity_column.clone(),
        })
    }

    fn relationship_layout(&self, source: ir::SourceTableId) -> Option<RelationshipTableLayout> {
        let entry = self
            .relationship_source_entry()
            .filter(|entry| entry.id == source)?;
        Some(RelationshipTableLayout {
            table: entry.table.clone(),
            identity_column: entry.identity_column.clone(),
            start_column: entry.start_column.clone(),
            end_column: entry.end_column.clone(),
        })
    }

    fn property_column(&self, source: ir::SourceTableId, property: ir::PropertyId) -> Option<String> {
        let table_name = if self.node_source_entry().is_some_and(|entry| entry.id == source) {
            &self.node_source_entry()?.table
        } else if self
            .relationship_source_entry()
            .is_some_and(|entry| entry.id == source)
        {
            &self.relationship_source_entry()?.table
        } else {
            return None;
        };
        let table = self.connection.current_schema().get_table(table_name)?;
        let index = (property.get() as usize).checked_sub(1)?;
        table.get_column_at(index)?.name.clone()
    }
}
```

Add to `graph/frontend/src/lib.rs` (match the existing `mod`/`pub use`
ordering already present in the file):

```rust
mod schema_catalog;
pub use schema_catalog::SchemaCatalog;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p turso_graph_frontend schema_catalog -- --nocapture`
Expected: `resolves_id_name_age_matching_testkit_stub_identities` PASSES.

- [ ] **Step 5: Commit**

```bash
cargo fmt
cargo clippy -p turso_graph_frontend --all-features --all-targets -- --deny=warnings
git add graph/frontend/src/schema_catalog.rs graph/frontend/src/lib.rs
git commit -S -m "graph/frontend: add SchemaCatalog backed by core::Schema

No production GraphCatalogSnapshot/RelationalCatalogSnapshot existed
anywhere — GraphCompiler::new, GraphSession::install, and
PgConnection::install_graph all merely forwarded a caller-supplied
catalog. SchemaCatalog reads core::Connection::current_schema()
directly and reuses Schema::classify_column for STRUCT/UNION/CUSTOM/
DOMAIN classification instead of re-deriving it."
```

---

### Task 4: Gate STRICT custom-typed sources at registration time

**Files:**
- Modify: `graph/frontend/src/catalog.rs`
- Modify: `graph/frontend/Cargo.toml` (add `tempfile` dev-dependency — needed
  for Step 1's file-backed reopen test; `turso_core`'s `fs` feature, which
  gates `Database::open_file_with_flags`/`PlatformIO`, is already enabled
  since it's a default `turso_core` feature and this crate does not disable
  default features)
- Test: `graph/frontend/src/catalog.rs` (existing `#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: `Connection::current_schema`,
  `Connection::experimental_custom_types_enabled`.
- Produces: `CatalogError::CustomTypesDisabled { table: String, column: String }`,
  called from `register_graph_in_transaction` for every node/relationship
  source table. Fails closed: a STRICT source with a custom/struct/union/
  domain column registers only when the connection has
  `--experimental-custom-types` (`DatabaseOpts::with_custom_types(true)`)
  enabled — otherwise `SchemaCatalog` (Task 3) would silently type such a
  column as `Any`/`Bytes` with no signal that richer typing exists but is
  disabled.

**Resolution note (added after Task 4's first implementation attempt was
BLOCKED, confirmed via `eprintln!` instrumentation and a static read of
`core`):** the detection mechanism below was originally specified as
`Schema::classify_column` (Task 1), matching this plan's "reuse before
duplicate" global constraint. That doesn't work: `core`'s `type_registry` is
only populated (both `bootstrap_builtin_types` in `Schema::new`,
`core/schema.rs:927-929`, and persisted `CREATE TYPE`/`CREATE DOMAIN`
definitions loaded at connect time, `core/lib.rs:1608-1633`) when
`--experimental-custom-types` is enabled. So on exactly the disabled
connection this gate exists to check, `classify_column` can only ever
report `Builtin` — it structurally cannot observe a column that was made
custom-typed earlier on a different, enabled connection. The human
(plan owner) chose, when presented with this blocker: detect a foreign type
name directly, by comparing the column's raw declared type string against
Turso's fixed STRICT builtin keyword set, instead of calling
`classify_column`. This is a deliberate, narrow, explicitly-documented
exception to this plan's "reuse `classify_column`" constraint for this one
case — not a silent reinterpretation. The exact builtin keyword set to
mirror is verified from `core/translate/schema.rs:788-791` (the same
`turso_macros::match_ignore_ascii_case!` set CREATE TABLE's own STRICT
column-type validator uses): `INT`, `INTEGER`, `REAL`, `TEXT`, `BLOB`, `ANY`.
That same code path (`core/translate/schema.rs:818-829`) proves the
soundness of this signal: a STRICT column can only end up with a type name
outside that set if a type definition was registered for it at `CREATE
TABLE` time (otherwise CREATE TABLE itself bails with `unknown datatype`) —
so "STRICT column, non-builtin-keyword type name" is proof of a
CUSTOM/DOMAIN/STRUCT/UNION column, regardless of whether the registry is
currently loaded. Non-STRICT tables never enforce this at all (SQLite
duck-typing accepts any type name loosely), so the gate below only inspects
STRICT tables.

- [ ] **Step 1: Write the failing test**

Add to `graph/frontend/src/catalog.rs`'s existing `#[cfg(test)] mod tests`
(after the existing `connection()`/`create_sources()`/`registration()`
helpers, ~line 741):

```rust
    fn connection_with_custom_types() -> Arc<Connection> {
        let io = Arc::new(MemoryIO::new());
        Database::open_file_with_flags(
            io,
            ":memory:graph-catalog-custom-types",
            OpenFlags::default(),
            DatabaseOpts::new().with_custom_types(true),
            None,
            Arc::new(SqliteDialect),
        )
        .expect("open database")
        .connect()
        .expect("connect")
    }

    #[test]
    fn register_graph_allows_struct_column_with_custom_types_enabled() {
        let connection = connection_with_custom_types();
        connection
            .execute(
                "CREATE TYPE point AS STRUCT(x INTEGER, y INTEGER); \
                 CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, origin point) STRICT; \
                 CREATE TABLE friendships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER) STRICT;",
            )
            .expect("create struct-typed source");

        let result = register_graph(&connection, &registration("people_graph"));
        assert!(result.is_ok(), "expected success: {result:?}");
    }

    #[test]
    fn register_graph_rejects_struct_column_when_custom_types_disabled() {
        // experimental_custom_types_enabled is fixed at Database-open time
        // (DatabaseOpts), not per-connection and not toggled by CREATE TYPE.
        // Two in-memory open_file calls never share state, so the only way
        // to reach "a STRICT table with a struct column exists, but this
        // connection has custom types disabled" is a real file: create it
        // with custom types enabled, fully close it, then reopen the same
        // file without the flag.
        let temp_dir = tempfile::TempDir::new().expect("tempdir");
        let db_path = temp_dir.path().join("graph-catalog-custom-types.db");
        let db_path_str = db_path.to_str().expect("utf8 path");

        {
            let io: Arc<dyn turso_core::IO> =
                Arc::new(turso_core::PlatformIO::new().expect("platform io"));
            let db = Database::open_file_with_flags(
                io,
                db_path_str,
                OpenFlags::default(),
                DatabaseOpts::new().with_custom_types(true),
                None,
                Arc::new(SqliteDialect),
            )
            .expect("open database with custom types enabled");
            let connection = db.connect().expect("connect");
            connection
                .execute(
                    "CREATE TYPE point AS STRUCT(x INTEGER, y INTEGER); \
                     CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, origin point) STRICT; \
                     CREATE TABLE friendships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER) STRICT;",
                )
                .expect("create struct-typed source");
            // Drop the connection and database so the file is fully closed
            // and the registry's cached Arc<Database> for this path/inode
            // has no live strong references before the next open.
            drop(connection);
            drop(db);
        }

        let io: Arc<dyn turso_core::IO> =
            Arc::new(turso_core::PlatformIO::new().expect("platform io"));
        let connection = Database::open_file_with_flags(
            io,
            db_path_str,
            OpenFlags::default(),
            DatabaseOpts::new(), // custom types NOT enabled on reopen
            None,
            Arc::new(SqliteDialect),
        )
        .expect("reopen database with custom types disabled")
        .connect()
        .expect("connect");

        let result = register_graph(&connection, &registration("people_graph"));
        assert!(
            matches!(
                &result,
                Err(CatalogError::CustomTypesDisabled { table, column })
                    if table == "people" && column == "origin"
            ),
            "expected CustomTypesDisabled error: {result:?}"
        );
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p turso_graph_frontend register_graph_allows_struct_column_with_custom_types_enabled register_graph_rejects_struct_column_when_custom_types_disabled -- --nocapture`
Expected: `register_graph_allows_struct_column_with_custom_types_enabled` PASSES
already (it exercises the enabled path, no new code needed yet — this step
confirms the *positive* path is unaffected before adding the gate).
`register_graph_rejects_struct_column_when_custom_types_disabled` FAILS —
with no gate yet, `register_graph` returns `Ok`, not
`Err(CustomTypesDisabled)`, so the `assert!` panics. Proceed to Step 3 to add
the actual registration-time gate.

- [ ] **Step 3: Implement the gate**

First, add the dev-dependency Step 1's test needs. In
`graph/frontend/Cargo.toml`'s `[dev-dependencies]`, add:

```toml
tempfile = { workspace = true }
```

No new import is needed in `graph/frontend/src/catalog.rs`'s
`use turso_core::{...}` block (line 7-13) — the detection below reads
`column.ty_str` directly, it does not call `Schema::classify_column` (see
the Resolution note above for why).

Add a new `CatalogError` variant (after `InvalidCatalogValue`, before
`Database`, ~line 107):

```rust
    #[error("source table `{table}` has a struct/union/custom-typed column `{column}` but this connection does not have --experimental-custom-types enabled")]
    CustomTypesDisabled { table: String, column: String },
```

Add a new function after `require_unique_identity` (~line 524, before its
definition or after — place it directly after `require_unique_identity`'s
closing brace to keep the two column-validation helpers adjacent):

```rust
/// Fails closed when a STRICT source table has a CUSTOM/DOMAIN/STRUCT/UNION
/// column but this connection lacks --experimental-custom-types. Without
/// this, SchemaCatalog would silently type such a column as Any/Bytes with
/// no signal that richer typing exists but is disabled for this connection.
///
/// Deliberately does NOT call `Schema::classify_column`: on a connection
/// with custom types disabled, `core`'s type_registry is entirely empty
/// (see `core/schema.rs:927-929`, `core/lib.rs:1608-1633`), so
/// `classify_column` can only ever report `Builtin` here — it cannot
/// observe a column that was made custom-typed earlier on a different,
/// enabled connection. Instead this compares the column's raw declared
/// type name against the exact builtin keyword set CREATE TABLE's own
/// STRICT column-type validator uses (`core/translate/schema.rs:788-791`).
/// That same validator (`core/translate/schema.rs:818-829`) guarantees the
/// soundness of this signal: a STRICT column can only have a non-builtin
/// type name if a type definition was registered for it at CREATE TABLE
/// time, so "STRICT column, non-builtin type name" is proof of a
/// CUSTOM/DOMAIN/STRUCT/UNION column even when the registry isn't loaded
/// right now. Non-STRICT tables never enforce this, so they're skipped.
fn require_custom_types_enabled_for_source(
    connection: &Arc<Connection>,
    table_name: &str,
) -> Result<(), CatalogError> {
    if connection.experimental_custom_types_enabled() {
        return Ok(());
    }
    let schema = connection.current_schema();
    let Some(table) = schema.get_table(table_name) else {
        return Err(CatalogError::SourceTableMissing(table_name.to_owned()));
    };
    if !table.is_strict() {
        return Ok(());
    }
    for column in table.columns() {
        let Some(name) = column.name.as_ref() else {
            continue;
        };
        let is_builtin = matches!(
            column.ty_str.to_ascii_uppercase().as_str(),
            "INT" | "INTEGER" | "REAL" | "TEXT" | "BLOB" | "ANY"
        );
        if !is_builtin {
            return Err(CatalogError::CustomTypesDisabled {
                table: table_name.to_owned(),
                column: name.clone(),
            });
        }
    }
    Ok(())
}
```

Call it from `register_graph_in_transaction` (`graph/frontend/src/catalog.rs:242-245`)
by replacing:

```rust
    for node in &registration.node_sources {
        require_columns(connection, &node.table, &[&node.identity_column])?;
        require_unique_identity(connection, &node.table, &node.identity_column)?;
    }
```

with:

```rust
    for node in &registration.node_sources {
        require_columns(connection, &node.table, &[&node.identity_column])?;
        require_unique_identity(connection, &node.table, &node.identity_column)?;
        require_custom_types_enabled_for_source(connection, &node.table)?;
    }
```

And immediately below (the `relationship_sources` loop starting at line 246),
add the same call after its existing `require_columns` call — read the loop
body first since it validates 5 columns (identity, start, end) in one
`require_columns` call rather than one at a time; add
`require_custom_types_enabled_for_source(connection, &relationship.table)?;`
as the next statement after that `require_columns` call, before whatever
validation currently follows it.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p turso_graph_frontend catalog::tests -- --nocapture`
Expected: all tests in `graph/frontend/src/catalog.rs::tests` PASS, including
the two new ones and all 9 pre-existing ones (regression gate).

- [ ] **Step 5: Commit**

```bash
cargo fmt
cargo clippy -p turso_graph_frontend --all-features --all-targets -- --deny=warnings
git add graph/frontend/src/catalog.rs graph/frontend/Cargo.toml
git commit -S -m "graph/frontend: fail closed on custom-typed sources without the feature flag

register_graph now rejects a STRICT source table that has a CUSTOM/
DOMAIN/STRUCT/UNION column when the registering connection lacks
--experimental-custom-types, instead of letting SchemaCatalog
silently downgrade such columns to Any/Bytes with no signal that
richer typing exists but is disabled."
```

---

### Task 5: Swap testkit's hardcoded `Catalog` stub for `SchemaCatalog`

**Files:**
- Modify: `graph/testkit/src/runner.rs`
- Test: existing conformance suite (`graph/testdata/suites/`), run via testkit binary.

**Interfaces:**
- Consumes: `turso_graph_frontend::SchemaCatalog::new` (Task 3),
  `turso_graph_frontend::catalog::{GraphRegistration, NodeSourceRegistration,
  RelationshipSourceRegistration, register_graph}`.
- Produces: no new public interface — this task proves `SchemaCatalog`
  reproduces the testkit's exact current behavior against the same
  hand-written `CREATE TABLE people/relationships` fixture the hardcoded
  stub simulated, which is the design spec's required non-STRICT regression
  proof.

- [ ] **Step 1: Read the current stub and its call sites**

Run: `sed -n '1,90p' graph/testkit/src/runner.rs` and
`sed -n '240,270p' graph/testkit/src/runner.rs` to see the exact `Catalog`
struct, its trait impls, and `build_fixture`'s `CREATE TABLE` statements
(needed to confirm the physical column layout `SchemaCatalog` must resolve
against matches the hardcoded `id→1, name→2, age→3` identities exactly).

- [ ] **Step 2: Replace the hardcoded `Catalog` construction with `SchemaCatalog`**

Locate the `struct Catalog` definition and its `impl GraphCatalogSnapshot for
Catalog` / `impl RelationalCatalogSnapshot for Catalog` blocks
(`graph/testkit/src/runner.rs`, confirmed non-`#[cfg(test)]` in prior
investigation). Delete the entire `struct Catalog { ... }` definition and
both trait impl blocks. Find wherever `Catalog { node_source: ..., relationship_source: ... }`
is constructed (in `build_fixture` or `fixture()`) and replace it with a call
that:

1. Registers the graph via `turso_graph_frontend::catalog::register_graph`
   against the already-created `people`/`relationships` tables (using the
   same table/column names the hardcoded stub assumed: `people` with
   `id`/`name`/`age`, `relationships` with `id`/`src`/`dst`, labels
   `"Person"`/`"KNOWS"` — confirm exact names from Step 1's read before
   writing this).
2. Constructs `turso_graph_frontend::SchemaCatalog::new(connection.clone(), registered_graph)`
   and passes `Arc::new(schema_catalog)` wherever the old `Arc::new(Catalog { ... })`
   was passed into `GraphSession::install`.

Write the exact replacement code only after Step 1's read confirms the
precise current construction site and table/column names — do not guess
field names not yet observed in this session.

- [ ] **Step 3: Run the full conformance suite**

Run: `cargo run -p turso_graph_testkit -- run --suite graph/testdata/suites/conformance.toml`
Expected: identical pass/fail counts to the last recorded baseline in
`docs/plans/2026-07-17-graph-conformance-performance-history.md`'s
"Execution status" section — 38 deep identities, 32 supported / 6
unsupported at their declared diagnostic boundary. Any deviation is a
regression introduced by the catalog swap and must be root-caused before
proceeding (per CLAUDE.md "Own your regressions" — do not revert to
"check if it fails on main").

Run: `cargo run -p turso_graph_testkit -- run --suite graph/testdata/suites/portable.toml`
Run: `cargo run -p turso_graph_testkit -- run --suite graph/testdata/suites/regressions.toml`
Expected: same pass/fail counts as the pre-change baseline for each.

- [ ] **Step 4: Run the testkit's own unit/integration tests**

Run: `cargo test -p turso_graph_testkit`
Expected: all PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt
cargo clippy -p turso_graph_testkit --all-features --all-targets -- --deny=warnings
git add graph/testkit/src/runner.rs
git commit -S -m "graph/testkit: replace hardcoded Catalog stub with SchemaCatalog

The conformance suite was exercising a hand-rolled catalog with
match arms hardcoded to the 'social' fixture's exact column names —
not the production catalog path. Swapping in SchemaCatalog proves
the real schema-backed catalog reproduces every existing conformance
result before any new type-system behavior is layered on top.

Tests: full conformance/portable/regressions suites unchanged from
the recorded baseline (32/38 supported, 6/38 unsupported)."
```

---

### Task 6: STRICT-source read fixtures — custom scalars and native arrays

**Files:**
- Create: `graph/frontend/tests/type_system_fixtures.rs`
- Modify: `graph/frontend/tests/` — check for an existing shared fixture
  helper module (e.g. `graph/frontend/tests/fixed_pattern_fixtures.rs` was
  seen referencing `ValueType::` in the earlier grep) and reuse its
  connection/registration helpers if one exists, rather than duplicating
  fixture setup.

**Interfaces:**
- Consumes: `SchemaCatalog` (Task 3), `GraphCompiler`/`GraphSession` or
  direct `bind`+`lower_relational` calls (whichever pattern
  `fixed_pattern_fixtures.rs` already established — read it first).

- [ ] **Step 1: Read the existing fixture pattern**

Run: `sed -n '1,80p' graph/frontend/tests/fixed_pattern_fixtures.rs` to learn
the exact helper functions (connection setup, graph registration, compile-and-run
pattern) already in use, so this task's new file matches established style
rather than inventing a parallel one.

- [ ] **Step 2: Write the failing tests**

Create `graph/frontend/tests/type_system_fixtures.rs` following the pattern
observed in Step 1. Include at minimum:

```rust
// Fixture setup mirrors graph/frontend/tests/fixed_pattern_fixtures.rs —
// see that file for the established connection/registration helpers this
// file reuses.

#[test]
fn custom_scalar_column_resolves_to_custom_value_type() {
    // 1. Open a connection with DatabaseOpts::new().with_custom_types(true).
    // 2. CREATE TYPE cents AS INTEGER; CREATE TABLE prices(id INTEGER PRIMARY KEY, amount cents) STRICT;
    // 3. Register a graph with `prices` as the node source.
    // 4. Bind `MATCH (p) RETURN p.amount` and assert the bound RETURN
    //    expression's ir::TypedExpression.value_type equals
    //    ir::ValueType::Custom { name: "cents".to_owned(), base: Box::new(ir::ValueType::Integer) }.
}

#[test]
fn integer_array_column_resolves_to_nested_list_value_type() {
    // 1. CREATE TABLE tags(id INTEGER PRIMARY KEY, labels INTEGER[]) STRICT;
    // 2. Register a graph with `tags` as the node source.
    // 3. Bind `MATCH (t) RETURN t.labels` and assert value_type equals
    //    ir::ValueType::List(Box::new(ir::ValueType::Integer)).
}

#[test]
fn blob_column_resolves_to_bytes_value_type() {
    // 1. CREATE TABLE embeddings(id INTEGER PRIMARY KEY, vector BLOB);
    //    (non-STRICT — vectors are plain BLOB columns, verified: no
    //    schema-level VECTOR column type exists.)
    // 2. Register a graph with `embeddings` as the node source.
    // 3. Bind `MATCH (e) RETURN e.vector` and assert value_type equals
    //    ir::ValueType::Bytes.
}
```

Fill in each test body using the exact connection/registration/bind helper
signatures found in Step 1 — do not leave the numbered-comment placeholders
above in the committed file; they mark what Step 1's research determines the
concrete code to be.

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p turso_graph_frontend --test type_system_fixtures -- --nocapture`
Expected: FAIL — either compile errors (if helper signatures differ from
assumptions) or assertion failures, since no code change has been made yet
in this task (Tasks 1-5 already provide everything needed; this task should
only need the test file itself, so failures here indicate the assumed
helper API doesn't match `fixed_pattern_fixtures.rs`'s actual API — fix the
test file, not production code).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p turso_graph_frontend --test type_system_fixtures -- --nocapture`
Expected: all three PASS once the test file's helper calls match the real
API from Step 1.

- [ ] **Step 5: Commit**

```bash
cargo fmt
cargo clippy -p turso_graph_frontend --all-features --all-targets -- --deny=warnings
git add graph/frontend/tests/type_system_fixtures.rs
git commit -S -m "graph/frontend: add STRICT-source read fixtures for custom/array/blob types

Proves SchemaCatalog resolves CREATE TYPE custom scalars, native
array columns, and plain BLOB columns to the correct ir::ValueType
end to end through MATCH/RETURN binding, not just at the unit level."
```

---

### Task 7: General-position map literal grammar (`{field: value}` as an expression)

**Files:**
- Modify: `graph/cypher/src/cypher.pest`
- Modify: `graph/cypher/src/ast.rs`
- Modify: `graph/cypher/src/parser.rs`
- Test: `graph/cypher/src/parser.rs` (existing `#[cfg(test)]` module)

**Interfaces:**
- Consumes: existing `map_literal`/`map_entry` pest rules
  (`graph/cypher/src/cypher.pest:103-104`, already implemented, currently
  scoped only to node/relationship pattern properties), existing `walk_map`
  parser function (`graph/cypher/src/parser.rs:588-599+`, already fully
  implemented).
- Produces: `ast::Expression::Map(Vec<(Spanned<String>, Spanned<Expression>)>)`,
  usable anywhere `primary_expression` is valid. Task 8 binds it.

- [ ] **Step 1: Write the failing test**

Add to `graph/cypher/src/parser.rs`'s existing `#[cfg(test)] mod tests`
(find it via `rg -n "mod tests" graph/cypher/src/parser.rs` and add near
other expression-parsing tests):

```rust
    #[test]
    fn parses_map_literal_as_general_expression() {
        let query = parse("RETURN {x: 1, y: 2}").expect("parses");
        // Exact assertion shape depends on how this test file's existing
        // tests unwrap a parsed Query's RETURN expression — mirror the
        // nearest existing RETURN-expression test's assertion pattern
        // (e.g. matching on query.return_clause / projections[0].expression.value)
        // rather than inventing a new access pattern.
    }
```

Read the nearest existing `RETURN <expr>`-parsing test in this same
`#[cfg(test)]` module first (e.g. search for `"RETURN "` string literals in
existing tests) and copy its exact assertion/unwrap pattern before filling
in this test's body — do not invent new `Query`/`Spanned` field access that
doesn't already appear elsewhere in this file.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p turso_graph_cypher parses_map_literal_as_general_expression -- --nocapture`
Expected: FAIL — parse error, since `primary_expression` does not currently
accept `map_literal`.

- [ ] **Step 3: Wire `map_literal` into `primary_expression`**

In `graph/cypher/src/cypher.pest`, change (line 97):

```pest
primary_expression = { function_call | list_literal | literal | parameter | identifier | "(" ~ expression ~ ")" }
```

to:

```pest
primary_expression = { function_call | list_literal | map_literal | literal | parameter | identifier | "(" ~ expression ~ ")" }
```

`map_literal` must be tried before `identifier` (already true by position)
and does not conflict with `list_literal`/`literal` since it starts with `{`,
a token none of those alternatives can begin with.

In `graph/cypher/src/ast.rs`, add a `Map` variant to `Expression` (add it
after `List(Vec<Spanned<Expression>>)`, matching the file's existing variant
ordering convention):

```rust
    Map(Vec<(Spanned<String>, Spanned<Expression>)>),
```

In `graph/cypher/src/parser.rs`'s `walk_expression` function
(lines 456-487), add a new arm. Find the arm currently handling
`Rule::list_literal` (or the nearest sibling arm) and add immediately after
it:

```rust
            Rule::map_literal => Expression::Map(walk_map(pair)?),
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p turso_graph_cypher parses_map_literal_as_general_expression -- --nocapture`
Expected: PASS.

Run: `cargo test -p turso_graph_cypher`
Expected: all pre-existing tests still PASS (regression gate — confirms
`map_literal` in pattern-property position, which already worked via
`walk_map` in a different call path, is unaffected).

- [ ] **Step 5: Commit**

```bash
cargo fmt
cargo clippy -p turso_graph_cypher --all-features --all-targets -- --deny=warnings
git add graph/cypher/src/cypher.pest graph/cypher/src/ast.rs graph/cypher/src/parser.rs
git commit -S -m "graph/cypher: allow map literals as a general expression

{field: value} was already parseable inside node/relationship pattern
properties via the existing map_literal/map_entry grammar and
walk_map, but not as a standalone expression — needed so CREATE/SET
can construct STRUCT/UNION property values via a map literal RHS."
```

---

### Task 8: Bind map literals to STRUCT/UNION mutation-property targets

**Files:**
- Modify: `graph/ir/src/mutation.rs` (no signature change — `PropertyValue.value: TypedExpression` already accepts any `ir::Expression`)
- Modify: `graph/ir/src/expression.rs` (add `ir::Expression::Map`)
- Modify: `graph/frontend/src/binder.rs`
- Test: `graph/frontend/src/binder.rs` (existing `#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: `ast::Expression::Map` (Task 7), `ir::ValueType::{Struct, Union}`
  (Task 2), `BindError::Unsupported` (existing, `graph/frontend/src/binder.rs:98-102`).
- Produces:
  ```rust
  // graph/ir/src/expression.rs
  pub enum Expression { /* ...existing... */ Map(Vec<(String, TypedExpression)>) }
  ```
  ```rust
  // graph/frontend/src/binder.rs
  fn bind_map_property(&self, target: &ir::ValueType, nullability: ir::Nullability, entries: &[(cypher::Spanned<String>, cypher::Spanned<cypher::Expression>)], span: cypher::Span) -> Result<ir::TypedExpression, BindError>
  ```
  Task 9's lowering matches on `ir::Expression::Map` and the `TypedExpression.value_type` it was bound with.

- [ ] **Step 1: Write the failing test**

Add to `graph/ir/src/expression.rs`'s `value_type_tests` module (from Task 2):

```rust
    #[test]
    fn map_expression_holds_ordered_field_bindings() {
        let map = Expression::Map(vec![(
            "x".to_owned(),
            TypedExpression {
                expression: Expression::Literal(Literal::Integer(1)),
                value_type: ValueType::Integer,
                nullability: Nullability::NonNull,
            },
        )]);
        match map {
            Expression::Map(entries) => assert_eq!(entries.len(), 1),
            _ => panic!("expected Map"),
        }
    }
```

Add to `graph/frontend/src/binder.rs`'s existing `#[cfg(test)] mod tests`
(after line 1330's `struct Catalog;` fixture — read the surrounding tests
first via `sed -n '1330,1420p' graph/frontend/src/binder.rs` to match its
exact `bind`/assertion helper pattern before writing):

```rust
    #[test]
    fn binds_map_literal_to_struct_mutation_property() {
        // Using this module's existing Catalog test fixture (extend its
        // `property` match arm — see Step 1's read — with a Struct-typed
        // "location" property on Node so this test can bind
        // `CREATE (:Person {location: {x: 1, y: 2}})` and assert the bound
        // ir::PropertyValue.value.expression is
        // ir::Expression::Map(vec![("x", ...Integer(1)), ("y", ...Integer(2))]).
    }
```

Read `graph/frontend/src/binder.rs`'s test `Catalog` fixture
(`sed -n '1330,1420p' graph/frontend/src/binder.rs`) before writing this
test's body, and extend its `property()` match arm with a `Struct`-typed
property rather than inventing a second fixture — match its existing style
(a `match (entity, name.as_str())` or similar, per the earlier-read pattern
`(CatalogEntity::Node, "id") => ...`).

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p turso_graph_ir map_expression_holds_ordered_field_bindings -- --nocapture`
Expected: compile error — `no variant named 'Map' found for enum 'Expression'`.

Run: `cargo test -p turso_graph_frontend binds_map_literal_to_struct_mutation_property -- --nocapture`
Expected: FAIL (compile error or unimplemented, once the ir test above is
fixed first — do these in the same step since they share the same PR-sized
change).

- [ ] **Step 3: Add `ir::Expression::Map`**

In `graph/ir/src/expression.rs`, add to the `Expression` enum (after
`List(Vec<TypedExpression>)`, the last existing variant):

```rust
    /// A `{field: value}` literal bound against a resolved STRUCT/UNION
    /// property target. Field order matches the target's declared field/
    /// variant order (established at bind time), not source order.
    Map(Vec<(String, TypedExpression)>),
```

- [ ] **Step 4: Implement `bind_map_property` and wire it into `bind_mutation_properties`**

In `graph/frontend/src/binder.rs`, add a new function after `bind_expression`
(after its closing brace, ~line 1150, before `resolve_binding`):

```rust
    fn bind_map_property(
        &self,
        target: &ir::ValueType,
        nullability: ir::Nullability,
        entries: &[(cypher::Spanned<String>, cypher::Spanned<cypher::Expression>)],
        span: cypher::Span,
    ) -> Result<ir::TypedExpression, BindError> {
        match target {
            ir::ValueType::Struct(fields) => {
                if entries.len() != fields.len() {
                    return Err(at_unsupported(span, "struct literal field count mismatch"));
                }
                let mut bound = Vec::with_capacity(entries.len());
                for (name, value) in entries {
                    let field_type = fields
                        .iter()
                        .find(|(field_name, _)| field_name == &name.value)
                        .map(|(_, field_type)| field_type)
                        .ok_or_else(|| BindError::UnknownProperty {
                            name: name.value.clone(),
                            span_start: name.span.start,
                            span_end: name.span.end,
                        })?;
                    let bound_value = self.bind_expression(value)?;
                    if &bound_value.value_type != field_type {
                        return Err(at_unsupported(value.span, "struct field type mismatch"));
                    }
                    bound.push((name.value.clone(), bound_value));
                }
                Ok(ir::TypedExpression {
                    expression: ir::Expression::Map(bound),
                    value_type: target.clone(),
                    nullability,
                })
            }
            ir::ValueType::Union(variants) => {
                if entries.len() != 1 {
                    return Err(at_unsupported(span, "union literal must set exactly one variant"));
                }
                let (name, value) = &entries[0];
                let variant_type = variants
                    .iter()
                    .find(|(variant_name, _)| variant_name == &name.value)
                    .map(|(_, variant_type)| variant_type)
                    .ok_or_else(|| BindError::UnknownProperty {
                        name: name.value.clone(),
                        span_start: name.span.start,
                        span_end: name.span.end,
                    })?;
                let bound_value = self.bind_expression(value)?;
                if &bound_value.value_type != variant_type {
                    return Err(at_unsupported(value.span, "union variant type mismatch"));
                }
                Ok(ir::TypedExpression {
                    expression: ir::Expression::Map(vec![(name.value.clone(), bound_value)]),
                    value_type: target.clone(),
                    nullability,
                })
            }
            _ => Err(at_unsupported(span, "map literal outside a struct or union property")),
        }
    }
```

Replace `bind_mutation_properties` (`graph/frontend/src/binder.rs:430-444`):

```rust
    fn bind_mutation_properties(
        &mut self,
        entity: CatalogEntity,
        properties: &[(cypher::Spanned<String>, cypher::Spanned<cypher::Expression>)],
    ) -> Result<Vec<ir::PropertyValue>, BindError> {
        properties
            .iter()
            .map(|(name, value)| {
                let resolved = self.resolve_property(entity, name)?;
                let bound_value = match &value.value {
                    cypher::Expression::Map(entries) => self.bind_map_property(
                        &resolved.value_type,
                        resolved.nullability,
                        entries,
                        value.span,
                    )?,
                    _ => self.bind_expression(value)?,
                };
                Ok(ir::PropertyValue {
                    property: resolved.id,
                    value: bound_value,
                })
            })
            .collect()
    }
```

Add a fallback arm to `bind_expression`'s match (`graph/frontend/src/binder.rs:1029-1144`)
so a map literal used outside a property assignment fails loud instead of
being unreachable. Add immediately after the `cypher::Expression::List(values) => { ... }`
arm (before the match's closing `};`):

```rust
            cypher::Expression::Map(_) => {
                return Err(at_unsupported(expression.span, "map literal outside a property assignment"));
            }
```

Also update the two existing `ir::Expression::Property` construction sites
so they compile against Task 10's forthcoming `fields` addition — **skip
this in Task 8**; Task 10 owns that change. This task only touches
`bind_mutation_properties`, `bind_map_property`, and the new `Map` fallback
arm.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p turso_graph_ir map_expression_holds_ordered_field_bindings -- --nocapture`
Run: `cargo test -p turso_graph_frontend binds_map_literal_to_struct_mutation_property -- --nocapture`
Expected: both PASS.

Run: `cargo test -p turso_graph_frontend`
Expected: all pre-existing binder tests still PASS.

- [ ] **Step 6: Commit**

```bash
cargo fmt
cargo clippy -p turso_graph_ir -p turso_graph_frontend --all-features --all-targets -- --deny=warnings
git add graph/ir/src/expression.rs graph/frontend/src/binder.rs
git commit -S -m "graph/frontend: bind map literals to STRUCT/UNION mutation properties

CREATE/SET property values now accept {field: value} literals when
the resolved property's catalog type is Struct or Union, validating
field/variant names and types against the target shape at bind time.
Map literals used anywhere else fail loud as BindError::Unsupported
rather than being silently unreachable."
```

---

### Task 9: STRICT-source struct/union write fixtures

**Files:**
- Create: append to `graph/frontend/tests/type_system_fixtures.rs` (Task 6)

**Interfaces:**
- Consumes: everything from Tasks 1-8. This task lowers the bound
  `ir::Expression::Map` to SQL — the `lower_expression_with_references`
  `Map` arm — which does not yet exist. **This task implements that arm**,
  since it's the last piece needed before the write fixture can execute
  end to end.

**Files (continued):**
- Modify: `graph/frontend/src/lowering.rs`
- Test: `graph/frontend/tests/type_system_fixtures.rs`

- [ ] **Step 1: Write the failing fixture test**

Append to `graph/frontend/tests/type_system_fixtures.rs`:

```rust
#[test]
fn create_with_struct_map_literal_lowers_and_executes() {
    // 1. CREATE TYPE point AS STRUCT(x INTEGER, y INTEGER);
    //    CREATE TABLE shapes(id INTEGER PRIMARY KEY, origin point) STRICT;
    // 2. Register a graph with `shapes` as the node source.
    // 3. Compile+execute `CREATE (:Shape {origin: {x: 1, y: 2}})` through
    //    the same GraphSession/mutation-execution path
    //    fixed_pattern_fixtures.rs uses for other CREATE fixtures.
    // 4. Query the underlying table directly (`SELECT origin FROM shapes`)
    //    and assert the stored value matches what `struct_pack(1, 2)`
    //    would have produced — use core's own struct_pack via a direct SQL
    //    SELECT struct_pack(1, 2) comparison rather than hand-decoding the
    //    blob.
}

#[test]
fn create_with_union_map_literal_lowers_and_executes() {
    // 1. CREATE TYPE contact AS UNION(email TEXT, phone TEXT);
    //    CREATE TABLE people(id INTEGER PRIMARY KEY, reach contact) STRICT;
    // 2. Register a graph with `people` as the node source.
    // 3. Compile+execute `CREATE (:Person {reach: {email: 'a@example.com'}})`.
    // 4. Assert the stored value matches `SELECT union_value('email', 'a@example.com')`.
}
```

Fill in each numbered comment with real code using the connection/execution
helper pattern already established in `fixed_pattern_fixtures.rs` (read it
again if the exact CREATE-mutation-execution call wasn't already captured in
Task 6's Step 1).

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p turso_graph_frontend --test type_system_fixtures create_with -- --nocapture`
Expected: FAIL — `LowerError` (no `Map` arm in `lower_expression_with_references`,
falls through to a non-exhaustive-match compile error since `ir::Expression`
is not `#[non_exhaustive]`).

- [ ] **Step 3: Implement `Map` lowering**

In `graph/frontend/src/lowering.rs`, add a new match arm to
`lower_expression_with_references` (`graph/frontend/src/lowering.rs:581-685`),
immediately after the `ir::Expression::List(values) => { ... }` arm and
before the match's closing `}`:

```rust
        ir::Expression::Map(entries) => match &expression.value_type {
            ir::ValueType::Struct(fields) => {
                let mut ordered = Vec::with_capacity(fields.len());
                for (field_name, _) in fields {
                    let (_, value) = entries
                        .iter()
                        .find(|(name, _)| name == field_name)
                        .ok_or_else(|| LowerError::InvalidName(field_name.clone()))?;
                    ordered.push(lower_expression_with_references(
                        value,
                        bindings,
                        catalog,
                        input_alias,
                        references,
                    )?);
                }
                Ok(format!("struct_pack({})", ordered.join(", ")))
            }
            ir::ValueType::Union(_) => {
                // Invariant: bind_map_property (graph/frontend/src/binder.rs)
                // only ever constructs a Union-typed Map with exactly one entry.
                let (tag, value) = &entries[0];
                let value_sql = lower_expression_with_references(
                    value,
                    bindings,
                    catalog,
                    input_alias,
                    references,
                )?;
                Ok(format!("union_value('{}', {value_sql})", tag.replace('\'', "''")))
            }
            _ => Err(LowerError::UnsupportedOperator(
                "map literal outside a struct or union property",
            )),
        },
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p turso_graph_frontend --test type_system_fixtures create_with -- --nocapture`
Expected: both PASS.

Run: `cargo test -p turso_graph_frontend`
Expected: all pre-existing lowering tests still PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt
cargo clippy -p turso_graph_frontend --all-features --all-targets -- --deny=warnings
git add graph/frontend/src/lowering.rs graph/frontend/tests/type_system_fixtures.rs
git commit -S -m "graph/frontend: lower Map expressions to struct_pack/union_value

Struct-typed map literals lower to struct_pack() with entries
reordered to the target's declared field order; union-typed map
literals lower to union_value('tag', value), relying on the bind-time
invariant that a Union-typed Map always has exactly one entry.

Tests: end-to-end CREATE ... {field: value} through GraphSession
against STRICT struct/union columns."
```

---

### Task 10: Nested struct/union property reads (`n.address.city`)

**Files:**
- Modify: `graph/ir/src/expression.rs`
- Modify: `graph/frontend/src/binder.rs`
- Modify: `graph/frontend/src/lowering.rs`
- Test: `graph/frontend/src/binder.rs`, `graph/frontend/src/lowering.rs`
  (existing `#[cfg(test)]` modules)

**Verified SQL-surface constraint:** core's dot-chain field-access resolver
(`core/translate/expr/binding.rs:411-612`) only accepts `alias.column.field`
(`Expr::Qualified` → 1 field level) and `alias.column.field.subfield`
(`Expr::DoublyQualified` → 2 field levels); there is no 4-identifier AST
expression for a third level, and the resolver requires the chain's root to
be a literal column reference visible in `referenced_tables` — **not** an
arbitrary parenthesized subquery result. This means nested field access must
be lowered *inside* the existing correlated subquery's `SELECT` list as
`p.{col}.{field1}[.{field2}]`, not as `({subquery}).{field}` appended
afterward. Consequently this task extends the existing
`ir::Expression::Property` variant with a `fields: Vec<String>` member
(capped at 2 entries, enforced at bind time) instead of adding a separate
`FieldAccess` IR node — the simpler design is also the one grounded in what
core's SQL parser can actually execute.

**Interfaces:**
- Produces:
  ```rust
  // graph/ir/src/expression.rs
  Property { entity: BindingId, property: PropertyId, fields: Vec<String> }
  ```

- [ ] **Step 1: Write the failing tests**

Add to `graph/ir/src/expression.rs`'s `value_type_tests` (Task 2/8):

```rust
    #[test]
    fn property_expression_carries_nested_field_chain() {
        let property = Expression::Property {
            entity: BindingId::new(1).unwrap(),
            property: PropertyId::new(1).unwrap(),
            fields: vec!["address".to_owned(), "city".to_owned()],
        };
        match property {
            Expression::Property { fields, .. } => assert_eq!(fields.len(), 2),
            _ => panic!("expected Property"),
        }
    }
```

Add to `graph/frontend/src/binder.rs`'s `#[cfg(test)] mod tests` (extend the
same Struct-typed fixture property added in Task 8's test, per that test's
setup):

```rust
    #[test]
    fn binds_nested_struct_field_access() {
        // Using the Struct-typed "location" property from
        // binds_map_literal_to_struct_mutation_property's fixture (a
        // {x: INTEGER, y: INTEGER} struct), bind
        // `MATCH (n) RETURN n.location.x` and assert the RETURN expression
        // is ir::Expression::Property { property: <location's id>, fields: vec!["x".to_owned()], .. }
        // with value_type == ir::ValueType::Integer.
    }

    #[test]
    fn rejects_field_access_deeper_than_two_levels() {
        // Bind `MATCH (n) RETURN n.location.x.y.z` (assuming a fixture
        // where that's nonsensical depth) and assert BindError::Unsupported
        // is returned, not a panic or a silently truncated chain.
    }
```

Add to `graph/frontend/src/lowering.rs`'s existing `#[cfg(test)] mod tests`
(find it via `rg -n "mod tests" graph/frontend/src/lowering.rs`):

```rust
    #[test]
    fn lowers_nested_property_field_chain_inside_correlated_subquery() {
        // Construct an ir::Expression::Property with fields: vec!["city".to_owned()]
        // against a test RelationalCatalogSnapshot fixture (mirror this
        // file's existing Property-lowering test's fixture) and assert the
        // generated SQL matches
        // `(SELECT p."address"."city" FROM "..." AS p WHERE p."..." = ...)`
        // — i.e., the field is appended inside the subquery's SELECT list,
        // not outside the closing parenthesis.
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p turso_graph_ir property_expression_carries_nested_field_chain -- --nocapture`
Expected: compile error — `Property` has no field named `fields`.

- [ ] **Step 3: Add `fields` to `ir::Expression::Property`**

In `graph/ir/src/expression.rs`, change the `Property` variant:

```rust
    Property {
        entity: BindingId,
        property: PropertyId,
        fields: Vec<String>,
    },
```

- [ ] **Step 4: Update the two existing `ir::Expression::Property` construction sites**

In `graph/frontend/src/binder.rs:822-825` (`bind_properties`, pattern-clause
property filters — always a direct single-level read, never nested), add
`fields: Vec::new()`:

```rust
                expression: ir::Expression::Property {
                    entity: binding.id(),
                    property: property.id,
                    fields: Vec::new(),
                },
```

Replace the `cypher::Expression::Property { entity, name } => { ... }` arm
in `bind_expression` (`graph/frontend/src/binder.rs:1057-1082`) with a call
to a new chain-flattening helper. First add the helper as a free function
near `bind_literal`/`bind_binary_operator` (~line 1230, before
`bind_literal`):

```rust
/// Flattens a chain of Cypher `Property` nodes (`n.a.b.c`) into its root
/// expression and an ordered list of field names, root-to-leaf. For `n.a`,
/// returns `(n, [a])`; for `n.a.b`, returns `(n, [a, b])`.
fn flatten_property_chain<'a>(
    expression: &'a cypher::Spanned<cypher::Expression>,
) -> (&'a cypher::Spanned<cypher::Expression>, Vec<&'a cypher::Spanned<String>>) {
    match &expression.value {
        cypher::Expression::Property { entity, name } => {
            let (root, mut fields) = flatten_property_chain(entity);
            fields.push(name);
            (root, fields)
        }
        _ => (expression, Vec::new()),
    }
}
```

Then, inside `impl<'a> Binder<'a>` (wherever `bind_expression` is defined —
this is a method, not a free function, so add it as a sibling method to
`resolve_property`, ~line 1163):

```rust
    fn resolve_field(&self, base_type: &ir::ValueType, name: &cypher::Spanned<String>) -> Result<ir::ValueType, BindError> {
        let fields: &[(String, ir::ValueType)] = match base_type {
            ir::ValueType::Struct(fields) => fields,
            ir::ValueType::Union(variants) => variants,
            _ => {
                return Err(BindError::InvalidPropertyTarget {
                    span_start: name.span.start,
                    span_end: name.span.end,
                })
            }
        };
        fields
            .iter()
            .find(|(field_name, _)| field_name == &name.value)
            .map(|(_, field_type)| field_type.clone())
            .ok_or_else(|| BindError::UnknownProperty {
                name: name.value.clone(),
                span_start: name.span.start,
                span_end: name.span.end,
            })
    }
```

Now replace the `Property` arm in `bind_expression`'s match
(`graph/frontend/src/binder.rs:1057-1082`):

```rust
            cypher::Expression::Property { .. } => {
                let (root, field_chain) = flatten_property_chain(expression);
                let cypher::Expression::Variable(variable) = &root.value else {
                    return Err(BindError::InvalidPropertyTarget {
                        span_start: root.span.start,
                        span_end: root.span.end,
                    });
                };
                let binding = self.resolve_binding(variable, root.span)?;
                let kind = self
                    .entities
                    .get(&binding.id())
                    .ok_or(BindError::InvalidPropertyTarget {
                        span_start: root.span.start,
                        span_end: root.span.end,
                    })?
                    .kind;
                let (property_name, nested_fields) = field_chain
                    .split_first()
                    .expect("flatten_property_chain always yields at least the outer Property's name");
                let property = self.resolve_property(kind, property_name)?;
                if nested_fields.len() > 2 {
                    return Err(at_unsupported(
                        expression.span,
                        "struct/union field access deeper than two levels",
                    ));
                }
                let mut value_type = property.value_type.clone();
                for field in nested_fields {
                    value_type = self.resolve_field(&value_type, field)?;
                }
                (
                    ir::Expression::Property {
                        entity: binding.id(),
                        property: property.id,
                        fields: nested_fields.iter().map(|field| field.value.clone()).collect(),
                    },
                    value_type,
                    nullable(binding.nullability(), property.nullability),
                )
            }
```

- [ ] **Step 5: Implement chain-aware lowering**

In `graph/frontend/src/lowering.rs`, replace the `ir::Expression::Property`
arm (lines 586-617):

```rust
        ir::Expression::Property { entity, property, fields } => {
            if fields.len() > 2 {
                return Err(LowerError::UnsupportedOperator(
                    "struct/union field access deeper than two levels",
                ));
            }
            let binding = bindings
                .get(entity)
                .ok_or(LowerError::MissingBinding(*entity))?;
            let column = catalog.property_column(binding.source, *property).ok_or(
                LowerError::MissingProperty {
                    source_id: binding.source,
                    property: *property,
                },
            )?;
            let (table, identity) = match binding.kind {
                EntityKind::Node => {
                    let layout = catalog
                        .node_layout(binding.source)
                        .ok_or(LowerError::MissingSource(binding.source))?;
                    (layout.table, layout.identity_column)
                }
                EntityKind::Relationship => {
                    let layout = catalog
                        .relationship_layout(binding.source)
                        .ok_or(LowerError::MissingSource(binding.source))?;
                    (layout.table, layout.identity_column)
                }
            };
            let mut selector = quote_identifier(&column);
            for field in fields {
                validate_bare_name(field)?;
                selector.push('.');
                selector.push_str(&quote_identifier(field));
            }
            Ok(format!(
                "(SELECT p.{} FROM {} AS p WHERE p.{} = {})",
                selector,
                quote_identifier(&table),
                quote_identifier(&identity),
                binding_reference(*entity, input_alias, references)
            ))
        }
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -p turso_graph_ir property_expression_carries_nested_field_chain -- --nocapture`
Run: `cargo test -p turso_graph_frontend binds_nested_struct_field_access rejects_field_access_deeper_than_two_levels lowers_nested_property_field_chain_inside_correlated_subquery -- --nocapture`
Expected: all PASS.

Run: `cargo test -p turso_graph_ir -p turso_graph_frontend`
Expected: all pre-existing tests still PASS (this step's `fields: Vec::new()`
addition to `bind_properties` and the rewritten `Property` lowering arm must
not change behavior for existing single-level property reads).

- [ ] **Step 7: Commit**

```bash
cargo fmt
cargo clippy -p turso_graph_ir -p turso_graph_frontend --all-features --all-targets -- --deny=warnings
git add graph/ir/src/expression.rs graph/frontend/src/binder.rs graph/frontend/src/lowering.rs
git commit -S -m "graph/frontend: support nested struct/union field reads up to two levels

n.address.city binds by flattening the Cypher Property chain to its
root variable plus an ordered field-name list, and lowers by
appending the field chain inside the existing correlated subquery's
SELECT list (p.address.city) rather than wrapping the subquery result
in dot access — core's dot-chain resolver requires the chain root to
be a literal column reference, not a parenthesized subquery, and caps
at two field levels (Qualified/DoublyQualified AST shapes; verified
in core/translate/expr/binding.rs). Deeper chains fail loud as
BindError::Unsupported at bind time."
```

---

### Task 11: STRICT-source nested-read fixtures

**Files:**
- Append to `graph/frontend/tests/type_system_fixtures.rs`

**Interfaces:**
- Consumes: Task 10.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn nested_struct_field_read_lowers_and_executes() {
    // 1. CREATE TYPE address AS STRUCT(city TEXT, zip INTEGER);
    //    CREATE TYPE person AS STRUCT(name TEXT, home address);
    //    CREATE TABLE people(id INTEGER PRIMARY KEY, info person) STRICT;
    // 2. Insert one row with a struct_pack literal:
    //    INSERT INTO people VALUES (1, struct_pack('Ada', struct_pack('London', 90210)));
    // 3. Register a graph with `people` as the node source.
    // 4. Bind+lower+execute `MATCH (p) RETURN p.info.home.city` — this
    //    exceeds the two-level cap (info -> home -> city is 2 nested
    //    fields beyond the root property, exactly at the limit) — assert
    //    the query returns "London".
    // 5. Bind `MATCH (p) RETURN p.info.home.city.extra` (3 nested fields)
    //    and assert bind() returns Err(BindError::Unsupported { .. }).
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p turso_graph_frontend --test type_system_fixtures nested_struct_field_read -- --nocapture`
Expected: FAIL — the test body above still has placeholder comments; replace
each with real code using the connection/execution helper from
`fixed_pattern_fixtures.rs` (Task 6 Step 1) before running. Once filled in,
first run confirms it either passes already (Task 10 landed correctly) or
surfaces a genuine gap to fix.

- [ ] **Step 3: Fix any gap found**

If the two-level boundary case fails unexpectedly, re-examine Task 10's
`nested_fields.len() > 2` check — it must accept exactly 2 nested fields
(`home.city` beyond the root `info` property = 2 entries in `fields`) and
reject 3. Do not adjust the SQL-surface constraint without re-verifying
against `core/translate/expr/binding.rs` — the 2-level cap is a hard fact
about core's parser, not a tunable.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p turso_graph_frontend --test type_system_fixtures`
Expected: all fixtures in this file PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt
cargo clippy -p turso_graph_frontend --all-features --all-targets -- --deny=warnings
git add graph/frontend/tests/type_system_fixtures.rs
git commit -S -m "graph/frontend: add end-to-end nested struct field read fixture

Exercises the two-level field-access cap at its exact boundary
(2 nested fields succeeds, 3 fails at bind time) against a real
STRICT nested-struct schema."
```

---

### Task 12: Typed function registry — vectors, struct/union accessors, FTS

**Files:**
- Create: `graph/frontend/src/functions.rs`
- Modify: `graph/frontend/src/lib.rs` (`mod functions;`)
- Modify: `graph/frontend/src/binder.rs`
- Test: `graph/frontend/src/functions.rs`, `graph/frontend/src/binder.rs`

**Interfaces:**
- Consumes: `ir::ValueType`, `ir::VectorKind` (Task 2).
- Produces:
  ```rust
  pub struct FunctionSignature {
      pub arguments: Vec<ArgumentType>, // ArgumentType::{Any, Exact(ValueType), Vector}
      pub return_type: fn(&[ir::ValueType]) -> ir::ValueType,
  }
  pub fn lookup(name: &str) -> Option<&'static FunctionSignature>
  ```
  Task's binder change: `bind_expression`'s `Function` arm consults
  `functions::lookup` before falling back to today's untyped `Any` pass-through.

**Why a `fn(&[ValueType]) -> ValueType` return-type callback instead of a
fixed `ValueType`:** `vector32(a, b, c)` etc. return a `Vector(kind, dims)`
where `dims` is the literal argument count when statically known — this
needs to inspect the bound argument list, not just the function name.

- [ ] **Step 1: Write the failing test**

Create `graph/frontend/src/functions.rs`:

```rust
use turso_graph_ir as ir;

#[derive(Clone, Debug, PartialEq)]
pub enum ArgumentType {
    Any,
    Exact(ir::ValueType),
    Vector,
}

pub struct FunctionSignature {
    pub arguments: Vec<ArgumentType>,
    pub return_type: fn(&[ir::ValueType]) -> ir::ValueType,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector32_returns_dims_from_argument_count() {
        let signature = lookup("vector32").expect("vector32 registered");
        let arguments = vec![ir::ValueType::Real, ir::ValueType::Real, ir::ValueType::Real];
        assert_eq!(
            (signature.return_type)(&arguments),
            ir::ValueType::Vector(ir::VectorKind::Float32Dense, Some(3))
        );
    }

    #[test]
    fn vector_distance_cos_returns_real() {
        let signature = lookup("vector_distance_cos").expect("registered");
        assert_eq!(
            (signature.return_type)(&[]),
            ir::ValueType::Real
        );
    }

    #[test]
    fn struct_pack_returns_bytes() {
        let signature = lookup("struct_pack").expect("registered");
        assert_eq!((signature.return_type)(&[]), ir::ValueType::Bytes);
    }

    #[test]
    fn fts_match_returns_boolean() {
        let signature = lookup("fts_match").expect("registered");
        assert_eq!((signature.return_type)(&[]), ir::ValueType::Boolean);
    }

    #[test]
    fn unknown_function_returns_none() {
        assert!(lookup("not_a_real_function").is_none());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p turso_graph_frontend functions:: -- --nocapture`
Expected: compile error — `lookup` is not defined.

- [ ] **Step 3: Implement the registry**

Append to `graph/frontend/src/functions.rs`, before `#[cfg(test)]`:

```rust
fn vector_dims_return(kind: ir::VectorKind) -> fn(&[ir::ValueType]) -> ir::ValueType {
    match kind {
        ir::VectorKind::Float32Dense => |args| ir::ValueType::Vector(ir::VectorKind::Float32Dense, Some(args.len() as u32)),
        ir::VectorKind::Float64Dense => |args| ir::ValueType::Vector(ir::VectorKind::Float64Dense, Some(args.len() as u32)),
        ir::VectorKind::Float32Sparse => |args| ir::ValueType::Vector(ir::VectorKind::Float32Sparse, Some(args.len() as u32)),
        ir::VectorKind::Float1Bit => |args| ir::ValueType::Vector(ir::VectorKind::Float1Bit, Some(args.len() as u32)),
        ir::VectorKind::Float8 => |args| ir::ValueType::Vector(ir::VectorKind::Float8, Some(args.len() as u32)),
    }
}

fn return_real(_: &[ir::ValueType]) -> ir::ValueType {
    ir::ValueType::Real
}

fn return_bytes(_: &[ir::ValueType]) -> ir::ValueType {
    ir::ValueType::Bytes
}

fn return_boolean(_: &[ir::ValueType]) -> ir::ValueType {
    ir::ValueType::Boolean
}

fn return_text(_: &[ir::ValueType]) -> ir::ValueType {
    ir::ValueType::Text
}

fn return_integer(_: &[ir::ValueType]) -> ir::ValueType {
    ir::ValueType::Integer
}

/// Static typed function registry. Functions not listed here keep today's
/// untyped `Any` pass-through in `Binder::bind_expression` — this table is
/// additive, not a closed world.
pub fn lookup(name: &str) -> Option<FunctionSignature> {
    let (arguments, return_type): (Vec<ArgumentType>, fn(&[ir::ValueType]) -> ir::ValueType) = match name {
        "vector32" => (vec![ArgumentType::Any], vector_dims_return(ir::VectorKind::Float32Dense)),
        "vector32_sparse" => (vec![ArgumentType::Any], vector_dims_return(ir::VectorKind::Float32Sparse)),
        "vector64" => (vec![ArgumentType::Any], vector_dims_return(ir::VectorKind::Float64Dense)),
        "vector8" => (vec![ArgumentType::Any], vector_dims_return(ir::VectorKind::Float8)),
        "vector1bit" => (vec![ArgumentType::Any], vector_dims_return(ir::VectorKind::Float1Bit)),
        "vector_extract" => (vec![ArgumentType::Vector, ArgumentType::Exact(ir::ValueType::Integer)], return_real),
        "vector_concat" => (vec![ArgumentType::Vector, ArgumentType::Vector], return_bytes),
        "vector_slice" => (vec![ArgumentType::Vector, ArgumentType::Exact(ir::ValueType::Integer), ArgumentType::Exact(ir::ValueType::Integer)], return_bytes),
        "vector_distance_cos" | "vector_distance_l2" | "vector_distance_jaccard" | "vector_distance_dot" => {
            (vec![ArgumentType::Vector, ArgumentType::Vector], return_real)
        }
        "struct_pack" => (vec![ArgumentType::Any], return_bytes),
        "union_value" => (vec![ArgumentType::Exact(ir::ValueType::Text), ArgumentType::Any], return_bytes),
        "union_tag" => (vec![ArgumentType::Any], return_text),
        "fts_match" => (vec![ArgumentType::Exact(ir::ValueType::Text), ArgumentType::Exact(ir::ValueType::Text)], return_boolean),
        "fts_score" => (vec![], return_real),
        "fts_highlight" => (vec![ArgumentType::Any], return_text),
        _ => return None,
    };
    let _ = return_integer; // silence unused-fn lint if no INTEGER-returning entry is added later; remove if used above.
    Some(FunctionSignature { arguments, return_type })
}
```

Note: remove the `let _ = return_integer;` line and the unused `return_integer`
function entirely if no signature in the match above ends up needing it —
re-check the final match arms before committing; do not leave dead code.

Wire it into `bind_expression`'s `Function` arm
(`graph/frontend/src/binder.rs:1102-1127`):

```rust
            cypher::Expression::Function {
                name,
                arguments,
                distinct,
            } => {
                if *distinct {
                    return Err(at_unsupported(
                        expression.span,
                        "DISTINCT function arguments",
                    ));
                }
                let function = ir::FunctionName::new(name.value.clone())
                    .ok_or_else(|| at_unsupported(name.span, "empty function names"))?;
                let arguments = arguments
                    .iter()
                    .map(|argument| self.bind_expression(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                let (value_type, nullability) = match crate::functions::lookup(function.as_str()) {
                    Some(signature) => {
                        let argument_types: Vec<ir::ValueType> =
                            arguments.iter().map(|argument| argument.value_type.clone()).collect();
                        (
                            (signature.return_type)(&argument_types),
                            ir::Nullability::Nullable,
                        )
                    }
                    None => (ir::ValueType::Any, ir::Nullability::Nullable),
                };
                (
                    ir::Expression::Function {
                        function,
                        arguments,
                    },
                    value_type,
                    nullability,
                )
            }
```

Add `pub(crate) mod functions;` (or `mod functions;` with the `lookup`
function marked `pub(crate)`, matching this crate's existing visibility
convention — check whether `binder`/`lowering` are `pub mod` or `mod` in
`lib.rs` first and mirror it) to `graph/frontend/src/lib.rs`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p turso_graph_frontend functions:: -- --nocapture`
Expected: all 5 tests PASS.

Run: `cargo test -p turso_graph_frontend`
Expected: all pre-existing tests PASS — the `Function` arm change must be a
strict superset: functions not in the registry still bind to `Any`/`Nullable`
exactly as before.

- [ ] **Step 5: Commit**

```bash
cargo fmt
cargo clippy -p turso_graph_frontend --all-features --all-targets -- --deny=warnings
git add graph/frontend/src/functions.rs graph/frontend/src/binder.rs graph/frontend/src/lib.rs
git commit -S -m "graph/frontend: add typed function registry for vector/struct/union/fts calls

vector32/64/8/1bit/sparse, vector_distance_*, struct_pack,
union_value/union_tag, and fts_match/fts_score/fts_highlight now bind
to their real return types instead of falling through to Any. Any
function not in the registry keeps the existing untyped pass-through
unchanged."
```

---

### Task 13: Vector and FTS fixture tests

**Files:**
- Append to `graph/frontend/tests/type_system_fixtures.rs`

**Interfaces:**
- Consumes: Task 12.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn vector32_call_binds_to_vector_value_type() {
    // 1. CREATE TABLE embeddings(id INTEGER PRIMARY KEY, vector BLOB);
    // 2. Register a graph with `embeddings` as the node source.
    // 3. Bind `RETURN vector32(1.0, 2.0, 3.0)` (no MATCH needed — a
    //    function-call-only RETURN, following whatever pattern
    //    fixed_pattern_fixtures.rs uses for parameter-only/literal-only
    //    RETURN queries, if one exists — otherwise MATCH () first).
    // 4. Assert the RETURN expression's value_type equals
    //    ir::ValueType::Vector(ir::VectorKind::Float32Dense, Some(3)).
}

#[test]
fn vector_distance_cos_call_binds_to_real() {
    // Bind `RETURN vector_distance_cos(vector32(1.0), vector32(2.0))` and
    // assert value_type == ir::ValueType::Real.
}

#[test]
fn fts_match_call_binds_to_boolean() {
    // Bind `RETURN fts_match('needle', 'haystack text')` and assert
    // value_type == ir::ValueType::Boolean.
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p turso_graph_frontend --test type_system_fixtures vector32_call fts_match_call vector_distance -- --nocapture`
Expected: FAIL until the placeholder comments are replaced with real code
per Task 6 Step 1's established connection/bind helper pattern — same
process as prior fixture tasks.

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test -p turso_graph_frontend --test type_system_fixtures`
Expected: every test in the file PASSES — this is the full fixture suite
built across Tasks 6, 9, 11, 13.

- [ ] **Step 4: Run the complete workspace gate**

Run: `cargo build`
Run: `cargo test -p turso_core -p turso_graph_ir -p turso_graph_cypher -p turso_graph_frontend -p turso_graph_testkit`
Run: `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
Run: `cargo fmt --check`
Expected: everything PASSES/is clean. This is the plan's final correctness
gate — every task's regression assertions plus a full-workspace clippy pass.

- [ ] **Step 5: Commit**

```bash
git add graph/frontend/tests/type_system_fixtures.rs
git commit -S -m "graph/frontend: add vector/FTS typed function-call fixtures

Closes out the type-system implementation: vector32/vector_distance_*/
fts_match now bind to their real ValueType through a full
MATCH/RETURN cycle, alongside the existing custom-scalar, array,
struct, and union coverage from earlier tasks."
```

---

## Self-review

**1. Spec coverage** (against `docs/plans/2026-07-17-graph-type-system-design.md`):

- BLOB exposure → `ir::ValueType::Bytes` via `Affinity::Blob` (Task 3),
  fixture in Task 6. ✅
- Custom scalar (`CREATE TYPE ... BASE ...`) → `ValueType::Custom` (Task 2/3),
  fixture Task 6. ✅
- STRUCT → `ValueType::Struct`, read (Task 3/10), write (Task 8/9), fixtures
  Tasks 9/11. ✅
- UNION → `ValueType::Union`, read (Task 3/10), write (Task 8/9), fixtures
  Tasks 9/11. ✅
- Native arrays → `ValueType::List` wrapping via `wrap_array`/
  `array_dimensions` (Task 3), fixture Task 6. ✅
- Vector types → corrected design (no schema-level VECTOR column) honored:
  `ValueType::Vector` produced only by the typed function registry (Task 12),
  fixture Task 13. ✅
- FTS → `fts_match`/`fts_score`/`fts_highlight` typed registry entries
  (Task 12), fixture Task 13. ✅
- Catalog wiring gap (no production `GraphCatalogSnapshot` existed) →
  `SchemaCatalog` (Task 3), swapped into the only real integration point,
  testkit's runner (Task 5), with a full conformance-suite regression proof. ✅
- STRICT + custom-types-disabled gating → Task 4, additive and fails closed. ✅
- Map-literal grammar → Task 7, general expression position. ✅
- Nested property reads → Task 10, scoped and capped to what core's SQL
  parser can actually execute (verified, not assumed). ✅
- Reuse-before-duplicate discipline → every task's Interfaces section states
  which `core` type is reused directly (`Affinity`, `Schema`, `Table`,
  `Column`, `TypeDef`/`StructDef`/`UnionDef` via `Schema::classify_column`)
  versus which new IR-local type was unavoidable and why (`VectorKind`
  mirrors `core::VectorType` only because `graph/ir` cannot depend on
  `turso_core`; `ValueType::{Custom,Struct,Union,Vector}` and
  `Expression::Map` are new IR surface with no core equivalent to reuse,
  since core's types describe schema/storage, not a frontend-neutral
  expression-typing model). ✅

**2. Placeholder scan:** Tasks 5, 6, 9, 11, 13 contain numbered-comment
scaffolds inside test bodies rather than fully inlined code. These are not
"TBD" placeholders in the prohibited sense — each names the exact
CREATE TABLE/TYPE statements, the exact Cypher query, and the exact
assertion target, and explicitly instructs the implementer to copy the
established helper pattern from `graph/frontend/tests/fixed_pattern_fixtures.rs`
(read once, in Task 6 Step 1, and referenced thereafter) rather than
inventing one. This is deliberate: `fixed_pattern_fixtures.rs`'s exact
helper function signatures were not read during this planning session, and
guessing them would risk writing code with fabricated APIs — a worse
violation of "No Placeholders" than pointing at the one authoritative
source to copy from. Every task that does this names its Step 1 as reading
that file *before* the placeholders in later steps are filled in, so no
task can be executed by copying comments verbatim.

**3. Type consistency:** `PropertyId`/`LabelId`/`RelationshipTypeId` 1-based
assignment is stated once in Task 3 and used consistently in Task 3's
`property`/`label`/`relationship_type`/`property_column` implementations.
`ValueType::{Custom, Struct, Union, Vector}` (Task 2) are the exact shapes
consumed in Tasks 3, 8, 9, 10, 12. `Expression::Property`'s `fields: Vec<String>`
(Task 10) is added at every one of its two construction sites (Task 10 Step 4)
and its one destructuring site (Task 10 Step 5) — no third site was found
(verified via `rg -n "Expression::Property" graph --type rust` during
planning: exactly binder.rs:822, binder.rs:1075→now the flatten-based
replacement, lowering.rs:586; the `cypher::Expression::Property` sites at
parser.rs:550 and binder.rs:1310 are the unrelated Cypher-AST type and are
unaffected). `bind_map_property`'s signature (Task 8) matches its two call
sites in the rewritten `bind_mutation_properties` (Task 8) exactly.

---

Plan complete and saved to `docs/plans/2026-07-17-graph-type-system-implementation.md`. Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
