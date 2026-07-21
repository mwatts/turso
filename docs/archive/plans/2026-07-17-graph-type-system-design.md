# Graph type system design: exposing Turso's full type surface in Cypher

> **Status as of 2026-07-21:** this document predates the graph frontend
> delivery on `feature/graph-frontend` and is retained as an archival plan.
> Since it was written: Ladybug/Kuzu was removed from the corpus, the deep
> corpus grew to ~10k identities, `PreparedSource` + `FrontendCompiler`
> replaced the `ReprepareRecipe` naming, and `__turso_graph_expand`
> (GraphExpand) shipped. Where this text and the code disagree, the code and
> `graph/test-results/REPORT.md` are authoritative.
>
> **Update (2026-07-21, later):** the public session API was subsequently
> renamed (`GraphSession`->`GraphConnection`, `prepare_query`->`prepare`,
> `mutate`->`execute`, `query_result_types`->`Statement::result_types()`,
> `MutationParameters`->`Parameters`), and the Postgres
> `graph.cypher()`/`install_graph` adapter described here was deliberately
> **removed** -- the Postgres and graph frontends are separate crates; apps
> compose them on one core connection via
> `Connection::register_frontend_compiler`. See `graph/README.md` for the
> current API.

Date: 2026-07-17

Branch: `feature/graph-frontend`

Status: design spec, not yet an implementation plan. Produced by inspecting
the current graph crates and `core`'s type system as of `6a1e8ac2f`.

## Goal

The graph frontend's IR type system (`graph/ir/src/expression.rs::ValueType`)
currently covers only `Any`, `Boolean`, `Integer`, `Real`, `Text`, `Bytes`,
`Node`, `Relationship`, `Path`, and `List`. Turso's underlying engine supports
a much richer type surface: user-defined and built-in custom types (`CREATE
TYPE ... BASE ...` with `ENCODE`/`DECODE`, including built-ins `date`, `time`,
`timestamp`, `varchar(N)`, `numeric(P,S)`, `smallint`, `boolean`, `uuid`,
`bytea`, `inet`, `json`, `jsonb`), composite `STRUCT`/`UNION` types, native
`ARRAY[n]` columns, and a native `VECTOR` type with similarity functions. None
of this reaches Cypher today, and — separately — no production
`GraphCatalogSnapshot`/`RelationalCatalogSnapshot` implementation exists
anywhere in the repo; every implementation found (`graph/frontend/src/{binder,
mutation,session,graph_expand}.rs`, `graph/testkit/src/runner.rs`,
`postgres/frontend/session.rs`) is a `#[cfg(test)]` stub. This design closes
both gaps together: it is the catalog wiring that makes any richer `ValueType`
reachable at all.

Blobs, custom/user-defined types, array types, vector types, and FTS are the
named priorities.

## Current state (verified)

- `core/types.rs:336` — storage-level `Value` has exactly four variants:
  `Null`, `Numeric` (`Integer`/`Float`), `Text`, `Blob`. Column-level affinity
  is `core/vdbe/affinity.rs:77` (`Affinity`); schema-level storage class is
  `core/schema.rs:5533` (`Type`, six-way `Null/Text/Numeric/Integer/Real/Blob`).
- `core/schema.rs:260-500` — `CREATE TYPE`/`CREATE DOMAIN` resolve to
  `TypeDefKind::{Custom, Struct, Union}`. `Custom` wraps a base primitive with
  `ENCODE`/`DECODE` expressions, params, operators, and domain `CHECK`
  constraints. `Struct`/`Union` (`schema.rs:228-256`) are named composite
  types over field/variant lists, each field with a base `Affinity` and type
  name.
- Custom types are **STRICT-table only** and gated behind
  `DatabaseOpts::with_custom_types(true)` (default off, experimental) —
  confirmed via `core/benches/struct_union_benchmark.rs:31` and its
  `CREATE TABLE ... STRICT` usage throughout.
- Struct/union values are built and read via SQL functions and dot access, not
  literal syntax: `struct_pack(v1, v2, ...)` (positional, by declared field
  order), `union_value('tag', value)`, `union_tag(val)`, and `val.field`
  (`Expr::FieldAccess`, `sqlite/parser/src/ast.rs:466`, resolved to
  `FieldAccessResolution::{StructField, UnionVariant}` at bind time) —
  verified against `core/benches/struct_union_benchmark.rs:114,209,271,304`.
- `core/vector/vector_types.rs:9-15` — native `VectorType` enum: `Float32Dense`,
  `Float64Dense`, `Float32Sparse`, `Float1Bit`, `Float8`. Stored as a plain
  BLOB; even-length blobs are implicitly `Float32Dense`, odd-length blobs
  carry a trailing type-tag byte, decoded only at the value level by
  `Vector::vector_type(blob: &[u8]) -> Result<(VectorType, usize, usize)>`
  (`vector_types.rs:37`). **There is no schema-level VECTOR column type** —
  confirmed by exhaustive grep of `core/schema.rs` (zero "vector" hits) and of
  `core` for any `F32_BLOB`-style declared-type convention (zero hits). A
  column holding vectors is, to the schema, an ordinary `BLOB`/`bytea`
  column; Turso itself cannot statically say a given column is a vector
  column, let alone its kind or dimension count, without inspecting stored
  bytes.
  - `VectorType` derives `Debug, Clone, PartialEq, Copy` only — no `Eq`/`Hash`.
  - Functions: `vector32`, `vector32_sparse`, `vector64`, `vector8`,
    `vector1bit`, `vector_extract`, `vector_distance_{cos,l2,jaccard,dot}`,
    `vector_concat`, `vector_slice` (`core/function.rs:330-363`). No real ANN
    index yet — search is brute-force scan;
    `core/index_method/toy_vector_sparse_ivf.rs` is experimental only.
- `core/schema.rs:5407-5418` — `Column::is_array()`/`array_dimensions()`:
  native `INTEGER[3]`-style array columns, distinct from JSON arrays
  (`core/json/jsonb.rs`) and from vector blobs.
- FTS is native and Tantivy-backed, **not** SQLite FTS5-compatible
  (`docs/fts.md`). Created via `CREATE INDEX ... USING fts(...)`. Query
  surface is three SQL functions: `fts_match`, `fts_score`, `fts_highlight` —
  no separate virtual-table module to bind against.
- `graph/frontend/src/catalog.rs::register_graph` performs only structural
  validation (table/column existence, identity uniqueness) — no STRICT check,
  no type validation. Every existing graph fixture in the repo
  (`graph/testkit/src/runner.rs:203`, `graph/frontend/tests/
  fixed_pattern_fixtures.rs:176`) uses plain non-STRICT tables with ordinary
  affinity columns.
- `graph/frontend/src/mutation.rs:613-631` (`parameter_types`) and every
  `GraphCatalogSnapshot::property` stub map only
  `Integer`/`Real`/`Text`/`Bytes` — confirming today's type surface is
  exactly the four storage classes, nothing richer.
- `graph/frontend/src/binder.rs:1102-1126` and `graph/frontend/src/
  lowering.rs:649-668` — function calls already bind and lower generically:
  any bare name becomes `ir::Expression::Function`, typed `ValueType::Any`,
  and lowers verbatim to `name(args...)` SQL text. This already mechanically
  reaches `vector_*`/`fts_*`/`struct_pack`/`union_*` — the gap is a *typed*
  signature, not reachability.
- `core::Statement::get_column_type_info` (`core/statement.rs:1112-1186`)
  already implements almost the exact column-to-logical-type classification
  the new catalog needs: it pulls `Column::ty_str`/`array_dimensions()`,
  calls `Schema::resolve_type(declared_name, table.is_strict())`
  (`core/schema.rs:993`), and classifies the resolved `TypeDef` into the
  public `#[non_exhaustive]` `ColumnTypeInfo { declared_name,
  array_dimensions, base_type, kind: ColumnTypeKind }` where
  `ColumnTypeKind` (`statement.rs:142-164`) is `Builtin | Custom | Domain |
  Struct | Union`. This logic is inline in `Statement`, keyed off a live
  prepared statement and result-column index, so graph/frontend cannot call
  it as-is — it needs a `(Column, is_strict)` input, not a `Statement`. See
  section 2 for the extraction this design reuses instead of duplicating it.
- `graph/cypher/src/cypher.pest:103` — a `map_literal` grammar rule already
  exists, but is wired only into `node_pattern`/`relationship_body`
  (`cypher.pest:69,76`), not into the general expression grammar. There is no
  `Expression::Map` anywhere in `graph/cypher/src/ast.rs`.

## Design

### 1. `ir::ValueType` extensions (`graph/ir/src/expression.rs`)

```rust
pub enum ValueType {
    Any, Boolean, Integer, Real, Text, Bytes,      // unchanged
    Node, Relationship, Path,                       // unchanged
    List(Box<ValueType>),                            // unchanged — now also covers ARRAY[n]
    Custom { name: String, base: Box<ValueType> },   // NEW
    Struct(Vec<(String, ValueType)>),                 // NEW
    Union(Vec<(String, ValueType)>),                   // NEW
    Vector(VectorKind, Option<u32>),                   // NEW: dims known only when produced
                                                        // by a typed vector function call (§5) —
                                                        // never from a column declaration, since
                                                        // no schema-level VECTOR column type exists
}

pub enum VectorKind { Float32Dense, Float64Dense, Float32Sparse, Float1Bit, Float8 }
```

`Custom { name, base }` mirrors `TypeDefKind::Custom` directly and covers all
twelve documented built-ins (`date`, `time`, `timestamp`, `varchar(N)`,
`numeric(P,S)`, `smallint`, `uuid`, `bytea`, `inet`, `json`, `jsonb`, plus
`boolean` handled specially below) *and* arbitrary future user-defined
`CREATE TYPE` domains, with one variant — avoiding a speculative per-built-in
enum arm for types Cypher never needs to special-case. `boolean` is the one
exception: it resolves to the existing `ValueType::Boolean`, not `Custom`,
because Cypher already gives booleans first-class meaning (`TRUE`/`FALSE`
literals, predicate typing) that a generic wrapper would lose. Plain `BLOB`
and the `bytea` custom type both resolve under `ValueType::Bytes`, the latter
wrapped in `Custom` so the name is preserved for diagnostics.

`ir::Literal` is unchanged. No new literal syntax is needed for these types —
values arrive only through property reads, bound parameters, or the map
literal below (Cypher has no vector/uuid/date literal syntax and none is
proposed).

### 2. Catalog wiring (new — no production implementation exists today)

A new `GraphCatalogSnapshot`/`RelationalCatalogSnapshot` implementation
backed by `core::Schema` (working name `SchemaCatalog`), wired wherever
`GraphSession`/`GraphCompiler` is constructed in production code. This is the
missing link the existing handoff doc
(`docs/plans/2026-07-17-graph-implementation-handoff.md`) does not mention:
today nothing outside tests builds a `GraphCompilationCatalog`.

Column classification reuses `core` rather than re-deriving it: a small
additive extraction, `Schema::classify_column(&self, column: &Column,
is_strict: bool) -> ColumnTypeInfo`, factors the classification logic
already inline in `Statement::get_column_type_info` (`core/statement.rs:
1120-1175`) out to `Schema` itself. `Statement::get_column_type_info` is
changed to delegate to it, and `SchemaCatalog::property()` calls the same
function. One classification implementation, two callers — no parallel
reimplementation of struct/domain/custom detection in graph/frontend.

`property()` resolution, per column, is **dual-path**:

- **Non-STRICT source table** (today's and every existing graph's case):
  unchanged — storage class from `Affinity`/declared type maps to
  `Integer`/`Real`/`Text`/`Bytes` exactly as the current test stubs do. No
  existing graph registration is affected by this design.
- **STRICT source table**: look up the column's declared type name against
  `core::Schema`'s `TypeDef` registry:
  - `boolean` → `ValueType::Boolean`.
  - any other built-in or user `Custom` type → `ValueType::Custom{name,
    base}`, `base` resolved recursively from `TypeDefKind::Custom.base`'s
    `Affinity`.
  - `Struct`/`Union` → resolve each field/variant's declared type recursively
    (same rules, including nested struct/union), producing `ValueType::
    Struct`/`Union`.
  - No column ever resolves to `ValueType::Vector` — there is no schema-level
    VECTOR column type to resolve against (verified above). A column storing
    vectors resolves as an ordinary `Bytes`/`Custom` BLOB column like any
    other; `ValueType::Vector` is produced only by the typed function
    registry (section 5) for calls to `vector32`/`vector64`/etc., whose
    return type carries the kind, and dims when statically known from a
    literal-length argument.
  - array-dimensioned column (`Column::is_array()`) → wrap the resolved
    element type in `ValueType::List`, dropping the declared dimension count
    (a storage-level constraint Turso still enforces on write; the graph IR
    does not track it).

`register_graph` gains one additive, non-blocking check: if a STRICT source
uses custom/struct/union/vector columns while
`DatabaseOpts::with_custom_types(false)` (the default), registration fails
closed with a descriptive error at registration time rather than a deferred
query-time failure. Every other registration path is unchanged.

### 3. Map-literal grammar (`graph/cypher/src/cypher.pest`, `ast.rs`, `binder.rs`)

The `map_literal` pest rule already exists; wire it into the primary
expression production so `{k: expr, ...}` parses as a standalone expression,
not only inside pattern property lists.

- New `cypher::Expression::Map(Vec<(Spanned<String>, Spanned<Expression>)>)`
  and `ir::Expression::Map(Vec<(String, TypedExpression)>)`.
- The binder requires a resolvable `Struct`/`Union` target type for a map
  literal (property-assignment position: `CREATE`, `SET`, `MERGE ... SET`). A
  map literal with no resolvable target is a bind error — Turso has no
  anonymous record/map value type, so untyped `RETURN {a: 1, b: 2}` is
  explicitly **out of scope** for this design, not silently degraded to
  something else.
- Lowering: against a `Struct` target, reorder the literal's entries into the
  type's declared field order and emit `struct_pack(v1, v2, ...)`; against a
  `Union` target, require exactly one entry and emit `union_value('tag',
  value)`. Unknown field/variant names, arity mismatches, or a `Union`
  literal with more than one entry are binder errors, not runtime SQL errors.

### 4. Read path

Struct field / union variant access (`n.address.city`, `n.status.slack`)
lowers to the same `Expr::FieldAccess` dot-chain core already supports
(`sqlite/parser/src/ast.rs:466`). No new SQL surface is needed — only binder
support for chaining `Expression::Property` through nested `Struct`/`Union`
field types resolved in section 1/2.

### 5. Typed function registry (`graph/frontend/src/functions.rs`, new)

A small static table mapping known function names to `(argument ValueTypes,
return ValueType)`:

- `vector32`, `vector32_sparse`, `vector64`, `vector8`, `vector1bit` return
  `ValueType::Vector(kind, dims)` — the only source of a `Vector`-typed
  expression in this design, since no column can be statically known to hold
  one (section 2). `vector_extract`, `vector_concat`, `vector_slice`,
  `vector_distance_{cos,l2,jaccard,dot}` accept `Vector` arguments and return
  `Bytes`/`Real` as appropriate.
- `struct_pack`, `union_value`, `union_tag` (used internally by map-literal
  lowering, and directly callable)
- `fts_match`, `fts_score`, `fts_highlight`

All SQL names are reused verbatim — no Cypher-idiomatic aliases — consistent
with "Turso code remains authoritative wherever donor functionality
overlaps it" from the existing handoff doc. The binder checks this table
first; any name not in it keeps today's untyped `ValueType::Any` pass-through
exactly as now. This is fully additive: no existing query's binding behavior
changes.

## Error handling

Every new failure mode is a **bind-time** error through the existing
`BindError` type, not a runtime SQL error:

- Map literal with no resolvable `Struct`/`Union` target, or targeting a
  non-composite type.
- Map literal field/variant name absent from the target type, or a `Union`
  literal with other than exactly one entry.
- A registered function call whose bound argument types don't match its
  typed-registry signature (e.g. `vector_distance_cos(n.name, ...)` where
  `n.name : Text`).
- A STRICT column whose declared type name has no matching `TypeDef` (schema
  drift) fails closed with a descriptive catalog error — this is distinct
  from the deliberate `Any` pass-through paths (non-STRICT columns, unknown
  function names), which remain intentional and unchanged.
- A STRICT source using custom/struct/union/vector columns while custom types
  are disabled on the database fails at `register_graph` time (section 2).

## Testing plan

- `graph/frontend/tests/`: new STRICT-source fixture(s) covering each new
  `ValueType` (custom scalar, struct, union, vector, array-of-int) — read,
  write, and round-trip through `GraphSession`, following the existing
  `fixed_pattern_fixtures.rs` pattern.
- `graph/testdata/suites/` or `testing/sqltests/tests/`: conformance-style
  cases for the map-literal grammar, both valid construction and each
  binder-error case above as a negative case.
- `graph/frontend/src/{binder,lowering}.rs` unit tests (existing `#[cfg(test)]`
  pattern): typed function registry — known-function type checking,
  unknown-function pass-through unchanged.
- Non-STRICT regression: the existing mixed-source conformance corpus (12
  supported / 0 failed / 6 unsupported per the performance report) must stay
  green untouched — the concrete check that dual-path resolution didn't
  disturb current behavior.
- At least one test must exercise the new `GraphCatalogSnapshot` impl against
  a **real** `core::Schema` built from an actual `CREATE TABLE ... STRICT`
  statement, not a hand-written match-arm stub — today's stubs would hide any
  bug in real schema-to-type resolution.

## Out of scope

- Untyped/anonymous map or record literals with no resolvable target type.
- A dedicated Cypher-native FTS clause or vector-search syntax beyond direct
  `fts_*`/`vector_*` function calls (both explicitly decided as SQL-name
  pass-through, not new syntax).
- ANN indexing for vectors (core itself only has brute-force scan and an
  experimental toy IVF method today).
- Fixed-dimension tracking for `ARRAY[n]` in the graph IR (dimension is
  enforced by storage on write; the IR only tracks element type via `List`).
- Everything already listed as a known gap in
  `docs/plans/2026-07-17-graph-implementation-handoff.md` (`CALL`
  subqueries, `shortestPath()` syntax, weighted expressions, multi-pattern
  join enumeration) — unrelated to this design and unaffected by it.

## Next step

Invoke the `writing-plans` skill against this spec to produce a task-broken
implementation plan (matching the frontmatter/task-list format used by the
other docs in `docs/plans/`), sequencing catalog wiring before the IR
extensions it exposes, and the map-literal grammar before struct/union write
support.
