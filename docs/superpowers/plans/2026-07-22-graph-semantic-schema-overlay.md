# Graph Semantic Schema Overlay (Milestones 1-2) Implementation Plan

> **Status: complete and validated as of 2026-07-23.** The checklist records
> the delivered Milestones 1–2 implementation. Multi-source registration and
> fragment-interface polymorphism subsequently completed the prerequisites for
> Milestone 4; the parent agent spec is the authoritative next-work tracker.

> **Historical execution note:** this plan originally required
> `superpowers:subagent-driven-development` or
> `superpowers:executing-plans`. Its checked steps are retained as delivery
> history, not as instructions for new work.

**Goal:** Add an opt-in semantic-schema catalog to the Turso graph frontend that decouples conceptual node/relationship/property types from physical source tables and validates typed ownership on reads and writes (spec Milestones 1-2 of `.specs/graph-semantic-schema-overlay.agent-spec.md`). The public registration API is the integration surface for downstream consumers such as the tessera-turso adapter (planned separately, in the tessera repository), which will depend on `turso_graph_frontend` directly and call `register_semantic_schema`. The registration structs also derive serde as a secondary convenience so tooling can cache, inspect, or transport a registration as JSON.

**Architecture:** A new focused module `graph/frontend/src/semantic.rs` owns the additive catalog tables, the `register_semantic_schema` API, and an immutable `SemanticSnapshot` loaded once per `GraphConnection::open`. `SchemaCatalog` holds `Option<Arc<SemanticSnapshot>>`; when present, the `GraphCatalogSnapshot` resolution methods switch to persisted conceptual IDs and owner-aware property resolution. The binder already tracks per-binding label/type names in `EntityBinding.names` (`binder.rs:218-222`) — semantic type tracking extends that. Physical names stay exclusively behind `RelationalCatalogSnapshot`. Legacy graphs (no semantic rows) behave byte-for-byte as today.

**Tech Stack:** Rust, turso_core internal SQL catalog tables, thiserror, serde (new dep for `turso_graph_frontend`).

## Global Constraints

- Spec: `.specs/graph-semantic-schema-overlay.agent-spec.md` — Milestones 1-2 ONLY. No fragment-interface polymorphism (that is the amended Milestone 3; there is no inheritance or abstract-type machinery in any milestone), no constraints beyond endpoint checks, no TypeQL, no attribute instances, no native n-ary storage (named/n-ary relation semantics arrive by adapter-side reification over these same Milestone 1-2 primitives — no plan impact).
- `GraphRegistration` and every existing public type MUST compile unchanged for existing callers (spec MUST, line 107). All API additions are new types/functions only.
- Conceptual IDs (`LabelId`, `RelationshipTypeId`, `PropertyId`) are persisted catalog values in semantic mode — never source-list positions or column ordinals (spec failure conditions, lines 442-443). Never reuse `SourceTableId` as a conceptual identity.
- Value types derive ONLY through `SchemaCatalog::column_value_type` → core `Schema::classify_column` (`graph/frontend/src/schema_catalog.rs:166-238`). No second type classifier (spec MUST NOT, line 263).
- No physical table/column names in graph IR or `GraphCatalogSnapshot` results (spec MUST NOT, line 262).
- All registration and mutation validation must be atomic: validation failure ⇒ zero catalog/data writes (spec MUST, lines 247, 250).
- Semantic bind errors carry source spans (`span_start`/`span_end` like every `BindError` variant, `binder.rs:123-190`).
- `DynamicCatalog` (`graph/testkit/src/dynamic_catalog.rs`) stays legacy/schemaless. Do not modify donor corpora.
- Commands: use `rtk` prefix (e.g. `rtk cargo test -p turso_graph_frontend`). Commits: signed (`git commit -S`), conventional format `type(scope): message`.
- Verification gates (run from repo root, spec lines 400-409):
  ```bash
  rtk cargo fmt --all -- --check
  rtk cargo test -p turso_graph_ir
  rtk cargo test -p turso_graph_frontend
  rtk cargo test -p turso_graph_testkit
  rtk cargo run -q -p turso_graph_testkit -- run smoke --no-record
  rtk cargo run -q -p turso_graph_testkit -- corpus --no-record
  rtk cargo clippy -p turso_graph_ir -p turso_graph_frontend -p turso_graph_testkit --all-features --all-targets -- --deny=warnings
  ```
- Do NOT record a new conformance baseline as part of this plan (spec line 417).
- Never build/run with `--release`.

**Scope note:** The tessera-turso adapter (lowering Tessera PERA IR onto this API) is a SEPARATE plan in the tessera repository — see `tessera/.specs/tessera-turso.design-spec.md` (tessera repository). The adapter will depend on `turso_graph_frontend` as a git dependency and call `register_semantic_schema` directly; this plan's obligation to it is only a stable, documented, additive public registration API. The serde derives on the registration structs are a secondary tooling convenience, not the integration mechanism.

**Related work — `.specs/graph-native-capabilities.agent-spec.md` (procedure registry, `db.propertyKeys()`, FTS, `startNode()`/`endNode()`, snapshot diagnostics):**
- **File overlap:** that stream also edits `binder.rs`, `catalog.rs`, `schema_catalog.rs`, `session.rs`, `lib.rs`. Do not run the two streams in parallel worktrees against the same files; sequence them. This plan touches `bind_call` (`binder.rs:365-395`) not at all, so the procedure-registry work composes, but rebases will be nontrivial in `binder.rs` and `schema_catalog.rs`.
- **Single-source limit is honored by BOTH streams:** that spec forbids broadening multi-source registration "as an incidental part"; this plan also keeps it — Milestones 1-2 need only one node source and one relationship source because multiple semantic types may share a source. Multi-source (different types → different tables) is future work gated on its own binder design.
- **`db.propertyKeys()` coordination:** that spec derives keys from `RelationalCatalogSnapshot::payload_columns` (physical logical names). When a graph has a semantic schema, property keys SHOULD be the semantic property names instead. Whichever stream lands second must add: semantic mode ⇒ `db.propertyKeys()` enumerates `SemanticSnapshot` property names (still catalog-only, no row scans). Same for the FTS admin API's "logical property names validated against `SchemaCatalog`" — in semantic mode those are semantic property names resolved through ownership.
- **Snapshot diagnostics:** semantic registration bumps the graph generation (Task 2), which that spec's diagnostics will correctly report as `Stale` — no coordination needed beyond the shared generation mechanism.

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `graph/frontend/src/semantic.rs` | Create | Registration structs, validation, catalog DDL, `register_semantic_schema`, `SemanticSnapshot` + loader, `SemanticCatalogError` |
| `graph/frontend/src/lib.rs` | Modify (`:9-50`) | `mod semantic;` + re-exports |
| `graph/frontend/Cargo.toml` | Modify | add `serde` dep |
| `graph/frontend/src/binder.rs` | Modify | trait additions (`:21-35`), new `BindError` variants (`:123-190`), owner-aware `resolve_property` (`:4245`), strict CREATE/MERGE type + endpoint checks (`:1100-1300`) |
| `graph/frontend/src/schema_catalog.rs` | Modify | `SchemaCatalog` holds snapshot; semantic-mode resolution in `GraphCatalogSnapshot` impl (`:241-309`) |
| `graph/frontend/src/session.rs` | Modify (`:134-158`) | load semantic snapshot at open |
| `graph/frontend/src/compiler.rs` | Modify | expose semantic runtime info to the mutation executor |
| `graph/frontend/src/mutation.rs` | Modify | runtime dynamic-value and dynamic-map validation |
| `graph/frontend/tests/semantic_schema.rs` | Create | integration tests: registration, reopen, binder, mutation atomicity |
| `docs/graph.md`, `graph/README.md` | Modify | user documentation |

Task order below follows the spec's implementation pipeline (Phases 1-4). Tasks 1-3 are pure additions (no behavior change); tasks 4-8 wire the binder; tasks 9-10 wire runtime validation; tasks 11-12 close out compatibility and docs.

---

### Task 1: Registration input types, error enum, and in-memory validation

**Files:**
- Create: `graph/frontend/src/semantic.rs`
- Modify: `graph/frontend/src/lib.rs:16` (add `mod semantic;` after `mod schema_catalog;`), `graph/frontend/src/lib.rs:39-40` (re-exports)
- Modify: `graph/frontend/Cargo.toml` (add serde)

**Interfaces:**
- Produces: `SemanticSchemaRegistration { node_types, relationship_types }`, `SemanticNodeType { name, source, properties }`, `SemanticRelationshipType { name, source, start, end, properties }`, `SemanticProperty { name, column }`, `SemanticCatalogError`, `fn validate_semantic_registration(&SemanticSchemaRegistration, &RegisteredGraph) -> Result<(), SemanticCatalogError>`. All registration structs derive `Serialize, Deserialize`.

- [x] **Step 1: Record baseline** (spec Slice 0.1)

Run and save output to the task log — these must be green before and after every task:
```bash
rtk cargo test -p turso_graph_frontend
rtk cargo test -p turso_graph_testkit
```
Expected: PASS (record any pre-existing failures verbatim; do not fix them).

- [x] **Step 2: Add serde dependency**

In `graph/frontend/Cargo.toml` under `[dependencies]` (match existing workspace-dep style used by sibling crates — check root `Cargo.toml` `[workspace.dependencies]` for a serde entry first; if present use `serde = { workspace = true, features = ["derive"] }`, otherwise `serde = { version = "1", features = ["derive"] }`).

- [x] **Step 3: Write failing unit tests for input validation**

Create `graph/frontend/src/semantic.rs` with the types and a `#[cfg(test)]` module. Tests first (they fail to compile until types exist — that is the failing state for pure-type tasks):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn person_registration() -> SemanticSchemaRegistration {
        SemanticSchemaRegistration {
            node_types: vec![SemanticNodeType {
                name: "Customer".to_owned(),
                source: "Person".to_owned(), // registered node-source NAME, not a table
                properties: vec![SemanticProperty {
                    name: "fullName".to_owned(),
                    column: "name".to_owned(),
                }],
            }],
            relationship_types: vec![],
        }
    }

    #[test]
    fn duplicate_type_names_are_rejected_case_insensitively() {
        let mut registration = person_registration();
        let mut duplicate = registration.node_types[0].clone();
        duplicate.name = "CUSTOMER".to_owned();
        registration.node_types.push(duplicate);
        assert!(matches!(
            validate_registration_shape(&registration),
            Err(SemanticCatalogError::DuplicateTypeName { name }) if name == "CUSTOMER"
        ));
    }

    #[test]
    fn empty_names_are_rejected() {
        let mut registration = person_registration();
        registration.node_types[0].properties[0].name = " ".to_owned();
        assert!(matches!(
            validate_registration_shape(&registration),
            Err(SemanticCatalogError::EmptyName { kind: "property" })
        ));
    }

    #[test]
    fn duplicate_property_names_within_one_owner_are_rejected() {
        let mut registration = person_registration();
        let duplicate = registration.node_types[0].properties[0].clone();
        registration.node_types[0].properties.push(duplicate);
        assert!(matches!(
            validate_registration_shape(&registration),
            Err(SemanticCatalogError::DuplicatePropertyName { .. })
        ));
    }

    #[test]
    fn relationship_endpoints_must_reference_declared_node_types() {
        let mut registration = person_registration();
        registration.relationship_types.push(SemanticRelationshipType {
            name: "OWNS".to_owned(),
            source: "KNOWS".to_owned(),
            start: vec!["Customer".to_owned()],
            end: vec!["Ghost".to_owned()],
            properties: vec![],
        });
        assert!(matches!(
            validate_registration_shape(&registration),
            Err(SemanticCatalogError::UnknownEndpointType { node_type, .. }) if node_type == "Ghost"
        ));
    }

    #[test]
    fn registration_round_trips_through_serde_json() {
        let registration = person_registration();
        let json = serde_json::to_string(&registration).expect("serialize");
        let back: SemanticSchemaRegistration = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(registration, back);
    }
}
```

Add `serde_json` to `[dev-dependencies]` in `graph/frontend/Cargo.toml` for the round-trip test.

- [x] **Step 4: Implement the types and shape validation**

```rust
//! Opt-in semantic schema catalog: conceptual node/relationship/property
//! types decoupled from physical source tables. Additive to the physical
//! catalog in `catalog.rs`; graphs without semantic rows keep legacy
//! source-derived resolution.

use std::collections::HashSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use turso_core::Connection;

use crate::catalog::RegisteredGraph;

/// Complete opt-in semantic schema for one graph. Serde-serializable so
/// external toolchains can author it as data (JSON) without linking this
/// crate's callers to any schema language.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticSchemaRegistration {
    pub node_types: Vec<SemanticNodeType>,
    pub relationship_types: Vec<SemanticRelationshipType>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticNodeType {
    /// Conceptual name addressed by Cypher labels. Case-insensitive unique
    /// per graph. Independent of any table or source name.
    pub name: String,
    /// Name of a registered node source (`NodeSourceRegistration.name`).
    pub source: String,
    pub properties: Vec<SemanticProperty>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticRelationshipType {
    pub name: String,
    /// Name of a registered relationship source.
    pub source: String,
    /// Allowed semantic node types for the start endpoint. Empty = any.
    pub start: Vec<String>,
    /// Allowed semantic node types for the end endpoint. Empty = any.
    pub end: Vec<String>,
    pub properties: Vec<SemanticProperty>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticProperty {
    /// Conceptual property name. Case-insensitive unique per owner.
    pub name: String,
    /// Physical payload column on the owner's source table.
    pub column: String,
}

#[derive(Debug, Error)]
pub enum SemanticCatalogError {
    #[error("{kind} name must not be empty")]
    EmptyName { kind: &'static str },
    #[error("semantic type name `{name}` is duplicated")]
    DuplicateTypeName { name: String },
    #[error("semantic property `{property}` is duplicated on type `{owner}`")]
    DuplicatePropertyName { owner: String, property: String },
    #[error("semantic type `{semantic_type}` references unknown {kind} source `{source}`")]
    UnknownSource {
        semantic_type: String,
        kind: &'static str,
        source: String,
    },
    #[error("relationship type `{relationship_type}` {endpoint} endpoint references unknown semantic node type `{node_type}`")]
    UnknownEndpointType {
        relationship_type: String,
        endpoint: &'static str,
        node_type: String,
    },
    #[error("semantic property `{property}` on type `{owner}` maps to structural column `{column}`")]
    StructuralColumn {
        owner: String,
        property: String,
        column: String,
    },
    #[error("semantic property `{property}` on type `{owner}` maps to missing column `{column}` on table `{table}`")]
    ColumnMissing {
        owner: String,
        property: String,
        column: String,
        table: String,
    },
    #[error("semantic property `{property}` has incompatible value types across owners: {first_owner} maps it to {first_type:?}, {second_owner} to {second_type:?}")]
    IncompatiblePropertyType {
        property: String,
        first_owner: String,
        first_type: turso_graph_ir::ValueType,
        second_owner: String,
        second_type: turso_graph_ir::ValueType,
    },
    #[error("graph `{0}` already has a different semantic schema registered")]
    ConflictingSchema(String),
    #[error("graph `{0}` is not registered")]
    GraphNotFound(String),
    #[error("semantic catalog row has an invalid value in `{0}`")]
    InvalidCatalogValue(&'static str),
    #[error("semantic catalog operation failed: {0}")]
    Catalog(#[from] crate::catalog::CatalogError),
    #[error("semantic catalog database operation failed: {0}")]
    Database(#[from] turso_core::LimboError),
}

fn fold(name: &str) -> String {
    name.to_lowercase()
}

fn require_name(kind: &'static str, name: &str) -> Result<(), SemanticCatalogError> {
    if name.trim().is_empty() || name.contains('\0') {
        Err(SemanticCatalogError::EmptyName { kind })
    } else {
        Ok(())
    }
}

/// Pure in-memory shape validation: names non-empty, case-insensitive
/// uniqueness, endpoint references resolve to declared node types. Physical
/// checks (sources, columns, value types) happen in
/// `validate_against_graph` because they need a connection.
pub(crate) fn validate_registration_shape(
    registration: &SemanticSchemaRegistration,
) -> Result<(), SemanticCatalogError> {
    let mut type_names = HashSet::new();
    let mut node_type_names = HashSet::new();
    for node_type in &registration.node_types {
        require_name("semantic type", &node_type.name)?;
        require_name("source", &node_type.source)?;
        if !type_names.insert(fold(&node_type.name)) {
            return Err(SemanticCatalogError::DuplicateTypeName {
                name: node_type.name.clone(),
            });
        }
        node_type_names.insert(fold(&node_type.name));
        validate_properties(&node_type.name, &node_type.properties)?;
    }
    for relationship in &registration.relationship_types {
        require_name("semantic type", &relationship.name)?;
        require_name("source", &relationship.source)?;
        if !type_names.insert(fold(&relationship.name)) {
            return Err(SemanticCatalogError::DuplicateTypeName {
                name: relationship.name.clone(),
            });
        }
        validate_properties(&relationship.name, &relationship.properties)?;
        for (endpoint, allowed) in [("start", &relationship.start), ("end", &relationship.end)] {
            for node_type in allowed {
                if !node_type_names.contains(&fold(node_type)) {
                    return Err(SemanticCatalogError::UnknownEndpointType {
                        relationship_type: relationship.name.clone(),
                        endpoint,
                        node_type: node_type.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

fn validate_properties(
    owner: &str,
    properties: &[SemanticProperty],
) -> Result<(), SemanticCatalogError> {
    let mut names = HashSet::new();
    for property in properties {
        require_name("property", &property.name)?;
        require_name("column", &property.column)?;
        if !names.insert(fold(&property.name)) {
            return Err(SemanticCatalogError::DuplicatePropertyName {
                owner: owner.to_owned(),
                property: property.name.clone(),
            });
        }
    }
    Ok(())
}
```

In `lib.rs` add after line 16 (`mod schema_catalog;` block):
```rust
mod semantic;
```
and the re-export (after the `pub use schema_catalog::SchemaCatalog;` line):
```rust
pub use semantic::{
    SemanticCatalogError, SemanticNodeType, SemanticProperty, SemanticRelationshipType,
    SemanticSchemaRegistration,
};
```

- [x] **Step 5: Run tests**

Run: `rtk cargo test -p turso_graph_frontend semantic`
Expected: PASS (all 5 new tests).

- [x] **Step 6: Commit**

```bash
git add graph/frontend/src/semantic.rs graph/frontend/src/lib.rs graph/frontend/Cargo.toml Cargo.lock
git commit -S -m "feat(graph): add semantic schema registration types and shape validation"
```

---

### Task 2: Physical validation, catalog DDL, atomic idempotent registration

**Files:**
- Modify: `graph/frontend/src/semantic.rs`
- Test: `graph/frontend/tests/semantic_schema.rs` (create)

**Interfaces:**
- Consumes: `validate_registration_shape`, `crate::catalog::{load_registered_graph, RegisteredGraph}`, catalog SQL helpers pattern from `catalog.rs:826-900` (reimplement the tiny `query_rows`/`sql_string` helpers locally or make the `catalog.rs` ones `pub(crate)` — prefer making `catalog.rs` helpers `pub(crate)`: `query_rows`, `execute_internal`, `scalar_integer`, `integer`, `text`, `sql_string` at `catalog.rs:826-899`).
- Produces: `pub fn register_semantic_schema(connection: &Arc<Connection>, graph_name: &str, registration: &SemanticSchemaRegistration) -> Result<(), SemanticCatalogError>`; internal tables `__turso_internal_graph_semantic_types`, `..._semantic_type_sources`, `..._semantic_properties`, `..._semantic_ownership`, `..._semantic_endpoints`.

**Catalog DDL** (mirrors `create_catalog` style, `catalog.rs:508-525`; conceptual IDs are per-graph, per-kind, dense from 1 — allocated at registration and persisted, NEVER derived afterward):

```sql
CREATE TABLE IF NOT EXISTS __turso_internal_graph_semantic_types(
    graph_id INTEGER NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN ('node', 'relationship')),
    type_id INTEGER NOT NULL CHECK(type_id > 0),        -- LabelId / RelationshipTypeId value
    name TEXT NOT NULL COLLATE NOCASE,
    source_id INTEGER NOT NULL,                          -- SourceTableId (type-to-source mapping, M1: exactly one)
    PRIMARY KEY(graph_id, kind, type_id),
    UNIQUE(graph_id, name)
);
CREATE TABLE IF NOT EXISTS __turso_internal_graph_semantic_properties(
    graph_id INTEGER NOT NULL,
    property_id INTEGER NOT NULL CHECK(property_id > 0), -- PropertyId value
    name TEXT NOT NULL COLLATE NOCASE,
    PRIMARY KEY(graph_id, property_id),
    UNIQUE(graph_id, name)
);
CREATE TABLE IF NOT EXISTS __turso_internal_graph_semantic_ownership(
    graph_id INTEGER NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN ('node', 'relationship')),
    type_id INTEGER NOT NULL,
    property_id INTEGER NOT NULL,
    source_id INTEGER NOT NULL,
    column_name TEXT NOT NULL,
    PRIMARY KEY(graph_id, kind, type_id, property_id)
);
CREATE TABLE IF NOT EXISTS __turso_internal_graph_semantic_endpoints(
    graph_id INTEGER NOT NULL,
    relationship_type_id INTEGER NOT NULL,
    endpoint TEXT NOT NULL CHECK(endpoint IN ('start', 'end')),
    node_type_id INTEGER NOT NULL,
    PRIMARY KEY(graph_id, relationship_type_id, endpoint, node_type_id)
);
```

Notes locked in here (spec Slice 0.2 checkpoint items):
- No `supertype_id`/`is_abstract` columns — ever. The amended Milestone 3 (fragment-interface polymorphism) uses additive fragment and fragment-membership tables added when that milestone starts; no inheritance state exists in this schema at any point.
- Case folding: `COLLATE NOCASE` + Rust-side `to_lowercase()` for map keys.
- Relationship types keep their own dense ID space (RelationshipTypeId), disjoint from labels.
- Registering a semantic schema bumps the graph generation (`UPDATE __turso_graph_generations SET generation = generation + 1 WHERE graph_id = ?`) so existing traversal snapshots (which validate against generation) rebuild rather than silently carrying legacy identities. `GRAPH_CATALOG_VERSION` (`catalog.rs:23`) stays 1: table additions are `IF NOT EXISTS`-additive and old code never reads them. State this decision in the commit body (spec M1 item 6).

- [x] **Step 1: Write failing integration tests**

Create `graph/frontend/tests/semantic_schema.rs`:

```rust
use std::sync::Arc;

use turso_graph_frontend::{
    register_graph, register_semantic_schema, GraphRegistration, NodeSourceRegistration,
    RelationshipSourceRegistration, SemanticCatalogError, SemanticNodeType, SemanticProperty,
    SemanticRelationshipType, SemanticSchemaRegistration,
};
use turso_graph_frontend::core::{Database, MemoryIO, SqliteDialect};

fn connection() -> Arc<turso_graph_frontend::core::Connection> {
    let io = Arc::new(MemoryIO::new());
    Database::open_file(io, ":memory:semantic-schema", Arc::new(SqliteDialect))
        .expect("open database")
        .connect()
        .expect("connect")
}

/// One physical people/friendships pair backing the graph. Table and
/// column names deliberately share NO spelling with the semantic names
/// used in tests: physical/conceptual independence is the invariant.
fn registered_graph(connection: &Arc<turso_graph_frontend::core::Connection>) {
    connection
        .execute(
            "CREATE TABLE tbl_people(pk INTEGER PRIMARY KEY, full_name TEXT, birth_year INTEGER); \
             CREATE TABLE tbl_edges(pk INTEGER PRIMARY KEY, a INTEGER, b INTEGER, since INTEGER);",
        )
        .expect("create sources");
    register_graph(
        connection,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "people_src".to_owned(),
                table: "tbl_people".to_owned(),
                identity_column: "pk".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration {
                name: "edges_src".to_owned(),
                table: "tbl_edges".to_owned(),
                identity_column: "pk".to_owned(),
                start_column: "a".to_owned(),
                end_column: "b".to_owned(),
                start_node_source: "people_src".to_owned(),
                end_node_source: "people_src".to_owned(),
            }],
        },
    )
    .expect("register graph");
}

/// Two conceptual node types over ONE shared physical source (spec test
/// matrix item 1) plus a relationship type whose name is unrelated to its
/// table (item 2).
fn semantic_registration() -> SemanticSchemaRegistration {
    SemanticSchemaRegistration {
        node_types: vec![
            SemanticNodeType {
                name: "Customer".to_owned(),
                source: "people_src".to_owned(),
                properties: vec![
                    SemanticProperty { name: "displayName".to_owned(), column: "full_name".to_owned() },
                    SemanticProperty { name: "born".to_owned(), column: "birth_year".to_owned() },
                ],
            },
            SemanticNodeType {
                name: "Supplier".to_owned(),
                source: "people_src".to_owned(),
                properties: vec![
                    SemanticProperty { name: "displayName".to_owned(), column: "full_name".to_owned() },
                ],
            },
        ],
        relationship_types: vec![SemanticRelationshipType {
            name: "TRADES_WITH".to_owned(),
            source: "edges_src".to_owned(),
            start: vec!["Customer".to_owned()],
            end: vec!["Supplier".to_owned()],
            properties: vec![SemanticProperty { name: "since".to_owned(), column: "since".to_owned() }],
        }],
    }
}

#[test]
fn registration_is_idempotent_for_identical_input() {
    let connection = connection();
    registered_graph(&connection);
    register_semantic_schema(&connection, "social", &semantic_registration()).expect("first");
    register_semantic_schema(&connection, "social", &semantic_registration())
        .expect("identical replay succeeds");
}

#[test]
fn conflicting_replay_is_rejected_and_leaves_rows_unchanged() {
    let connection = connection();
    registered_graph(&connection);
    register_semantic_schema(&connection, "social", &semantic_registration()).expect("first");
    let mut conflicting = semantic_registration();
    conflicting.node_types[0].properties[0].column = "birth_year".to_owned();
    assert!(matches!(
        register_semantic_schema(&connection, "social", &conflicting),
        Err(SemanticCatalogError::ConflictingSchema(name)) if name == "social"
    ));
    // Identical original still replays fine: rows unchanged.
    register_semantic_schema(&connection, "social", &semantic_registration()).expect("unchanged");
}

#[test]
fn registration_rejects_structural_columns_missing_columns_and_unknown_sources() {
    let connection = connection();
    registered_graph(&connection);

    let mut structural = semantic_registration();
    structural.node_types[0].properties[0].column = "pk".to_owned();
    assert!(matches!(
        register_semantic_schema(&connection, "social", &structural),
        Err(SemanticCatalogError::StructuralColumn { .. })
    ));

    let mut endpoint_column = semantic_registration();
    endpoint_column.relationship_types[0].properties[0].column = "a".to_owned();
    assert!(matches!(
        register_semantic_schema(&connection, "social", &endpoint_column),
        Err(SemanticCatalogError::StructuralColumn { .. })
    ));

    let mut missing = semantic_registration();
    missing.node_types[0].properties[0].column = "ghost".to_owned();
    assert!(matches!(
        register_semantic_schema(&connection, "social", &missing),
        Err(SemanticCatalogError::ColumnMissing { .. })
    ));

    let mut bad_source = semantic_registration();
    bad_source.node_types[0].source = "nope".to_owned();
    assert!(matches!(
        register_semantic_schema(&connection, "social", &bad_source),
        Err(SemanticCatalogError::UnknownSource { .. })
    ));

    // Node type may not reference a relationship source and vice versa.
    let mut kind_mismatch = semantic_registration();
    kind_mismatch.node_types[0].source = "edges_src".to_owned();
    assert!(matches!(
        register_semantic_schema(&connection, "social", &kind_mismatch),
        Err(SemanticCatalogError::UnknownSource { kind: "node", .. })
    ));
}

#[test]
fn shared_property_with_incompatible_column_types_is_rejected() {
    let connection = connection();
    registered_graph(&connection);
    let mut registration = semantic_registration();
    // Supplier maps displayName (Text on Customer) to birth_year (Integer).
    registration.node_types[1].properties[0].column = "birth_year".to_owned();
    assert!(matches!(
        register_semantic_schema(&connection, "social", &registration),
        Err(SemanticCatalogError::IncompatiblePropertyType { .. })
    ));
}

#[test]
fn failed_registration_writes_no_catalog_rows() {
    let connection = connection();
    registered_graph(&connection);
    let mut invalid = semantic_registration();
    invalid.node_types[1].properties[0].column = "ghost".to_owned();
    assert!(register_semantic_schema(&connection, "social", &invalid).is_err());
    // A valid registration must still succeed from a clean slate — proving
    // the failed attempt left nothing behind (atomicity).
    register_semantic_schema(&connection, "social", &semantic_registration()).expect("clean");
}
```

Run: `rtk cargo test -p turso_graph_frontend --test semantic_schema`
Expected: FAIL — `register_semantic_schema` not found.

- [x] **Step 2: Make `catalog.rs` SQL helpers `pub(crate)`**

In `catalog.rs`, change `fn query_rows` (`:826`), `fn execute_internal` (`:830`), `fn scalar_integer` (`:838`), `fn integer` (`:850`), `fn text` (`:863`), `fn sql_string` (`:894`) from private to `pub(crate) fn`. No other edits.

- [x] **Step 3: Implement registration**

In `semantic.rs` add (uses only already-shown types plus catalog helpers):

```rust
use crate::catalog::{
    execute_internal, load_registered_graph, query_rows, scalar_integer, sql_string,
    GENERATIONS_TABLE,
};
use crate::schema_catalog::SchemaCatalog;
use turso_graph_ir as ir;

pub(crate) const SEMANTIC_TYPES_TABLE: &str = "__turso_internal_graph_semantic_types";
pub(crate) const SEMANTIC_PROPERTIES_TABLE: &str = "__turso_internal_graph_semantic_properties";
pub(crate) const SEMANTIC_OWNERSHIP_TABLE: &str = "__turso_internal_graph_semantic_ownership";
pub(crate) const SEMANTIC_ENDPOINTS_TABLE: &str = "__turso_internal_graph_semantic_endpoints";

/// Registers (or idempotently re-registers) the semantic schema for a
/// graph. Validation order: shape → physical (sources, columns, value-type
/// compatibility) → existing-schema comparison → transactional insert.
/// Any error leaves the catalog untouched.
pub fn register_semantic_schema(
    connection: &Arc<Connection>,
    graph_name: &str,
    registration: &SemanticSchemaRegistration,
) -> Result<(), SemanticCatalogError> {
    validate_registration_shape(registration)?;
    let graph = load_registered_graph(connection, graph_name)
        .map_err(|_| SemanticCatalogError::GraphNotFound(graph_name.to_owned()))?;
    validate_against_graph(connection, &graph, registration)?;

    // Same transaction discipline as register_graph (catalog.rs:125-178):
    // BEGIN IMMEDIATE in autocommit, savepoint inside a write transaction.
    // Reuse the identical structure; on any error, roll back.
    // ... (copy the BEGIN IMMEDIATE / SAVEPOINT wrapper from
    // register_graph verbatim, calling register_semantic_in_transaction).
    run_in_registration_transaction(connection, |connection| {
        register_semantic_in_transaction(connection, &graph, registration)
    })
}
```

`validate_against_graph` — physical checks (all before any write):

```rust
fn validate_against_graph(
    connection: &Arc<Connection>,
    graph: &RegisteredGraph,
    registration: &SemanticSchemaRegistration,
) -> Result<(), SemanticCatalogError> {
    // Owner-independent value type per shared property name, for
    // compatibility checking (spec: "all mapped columns MUST resolve to
    // compatible graph value types").
    let mut property_types: std::collections::HashMap<String, (String, ir::ValueType)> =
        std::collections::HashMap::new();

    let node_source = |name: &str| {
        graph
            .node_sources
            .iter()
            .find(|source| source.name.eq_ignore_ascii_case(name))
    };
    let relationship_source = |name: &str| {
        graph
            .relationship_sources
            .iter()
            .find(|source| source.name.eq_ignore_ascii_case(name))
    };

    for node_type in &registration.node_types {
        let source = node_source(&node_type.source).ok_or_else(|| {
            SemanticCatalogError::UnknownSource {
                semantic_type: node_type.name.clone(),
                kind: "node",
                source: node_type.source.clone(),
            }
        })?;
        let structural = [source.identity_column.as_str()];
        check_owned_columns(
            connection, &node_type.name, &node_type.properties,
            &source.table, &structural, &mut property_types,
        )?;
    }
    for relationship in &registration.relationship_types {
        let source = relationship_source(&relationship.source).ok_or_else(|| {
            SemanticCatalogError::UnknownSource {
                semantic_type: relationship.name.clone(),
                kind: "relationship",
                source: relationship.source.clone(),
            }
        })?;
        let structural = [
            source.identity_column.as_str(),
            source.start_column.as_str(),
            source.end_column.as_str(),
        ];
        check_owned_columns(
            connection, &relationship.name, &relationship.properties,
            &source.table, &structural, &mut property_types,
        )?;
    }
    Ok(())
}
```

`check_owned_columns` resolves each mapped column's `ir::ValueType` through the SAME classification the catalog snapshot uses. Do NOT reimplement classification: extract the existing logic by making `SchemaCatalog::column_value_type` reachable — refactor it into a free `pub(crate) fn column_value_type(schema: &Schema, column: &Column, is_strict: bool) -> ir::ValueType` in `schema_catalog.rs` (move the body of the current method at `schema_catalog.rs:166-238`; the method becomes a one-line delegate so existing call sites are untouched). Then:

```rust
fn check_owned_columns(
    connection: &Arc<Connection>,
    owner: &str,
    properties: &[SemanticProperty],
    table_name: &str,
    structural: &[&str],
    property_types: &mut std::collections::HashMap<String, (String, ir::ValueType)>,
) -> Result<(), SemanticCatalogError> {
    let schema = connection.current_schema();
    let table = schema
        .get_table(table_name)
        .ok_or_else(|| SemanticCatalogError::Catalog(
            crate::catalog::CatalogError::SourceTableMissing(table_name.to_owned()),
        ))?;
    for property in properties {
        if structural.iter().any(|column| column.eq_ignore_ascii_case(&property.column)) {
            return Err(SemanticCatalogError::StructuralColumn {
                owner: owner.to_owned(),
                property: property.name.clone(),
                column: property.column.clone(),
            });
        }
        let Some((_, column)) = table.get_column_by_name(&property.column) else {
            return Err(SemanticCatalogError::ColumnMissing {
                owner: owner.to_owned(),
                property: property.name.clone(),
                column: property.column.clone(),
                table: table_name.to_owned(),
            });
        };
        let value_type =
            crate::schema_catalog::column_value_type(&schema, column, table.is_strict());
        match property_types.entry(fold(&property.name)) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert((owner.to_owned(), value_type));
            }
            std::collections::hash_map::Entry::Occupied(entry) => {
                let (first_owner, first_type) = entry.get();
                // Compatible = identical, or either side is Any.
                let compatible = *first_type == value_type
                    || *first_type == ir::ValueType::Any
                    || value_type == ir::ValueType::Any;
                if !compatible {
                    return Err(SemanticCatalogError::IncompatiblePropertyType {
                        property: property.name.clone(),
                        first_owner: first_owner.clone(),
                        first_type: first_type.clone(),
                        second_owner: owner.to_owned(),
                        second_type: value_type,
                    });
                }
            }
        }
    }
    Ok(())
}
```

`register_semantic_in_transaction`: create the four tables (`IF NOT EXISTS`), then:
1. If the graph already has semantic rows: load them (Task 3's loader reads raw rows; here compare against a canonicalized form of `registration` — same types, same sources, same property/column pairs, same endpoints, all case-insensitively). Identical ⇒ `return Ok(())` (idempotent). Different ⇒ `Err(ConflictingSchema)`.
2. Otherwise allocate dense IDs: node types get `type_id` 1..N in input order (these values ARE the `LabelId`s), relationship types 1..M (`RelationshipTypeId`s), distinct property names 1..K in first-seen order (`PropertyId`s). Insert types, properties, ownership (with the source's `SourceTableId` and column name), endpoints (resolving endpoint node-type names to the just-allocated node `type_id`s).
3. Bump generation: `execute_internal(connection, format!("UPDATE {GENERATIONS_TABLE} SET generation = generation + 1 WHERE graph_id = {}", graph.id.get()))`.

Export in `lib.rs`: add `register_semantic_schema` to the `pub use semantic::{...}` list.

- [x] **Step 4: Run tests**

Run: `rtk cargo test -p turso_graph_frontend --test semantic_schema`
Expected: PASS (all 5). Also `rtk cargo test -p turso_graph_frontend` — no regressions.

- [x] **Step 5: Commit**

```bash
git add graph/frontend/src/semantic.rs graph/frontend/src/catalog.rs graph/frontend/src/schema_catalog.rs graph/frontend/src/lib.rs graph/frontend/tests/semantic_schema.rs
git commit -S -m "feat(graph): atomic idempotent semantic schema registration

Conceptual type/property IDs are allocated densely at registration and
persisted; they are never derived from source positions or column
ordinals. Registration bumps the graph generation so traversal snapshots
rebuild instead of carrying legacy identities. GRAPH_CATALOG_VERSION
stays 1: the semantic tables are IF NOT EXISTS-additive and invisible to
legacy readers. No supertype/abstract columns exist: the amended
Milestone 3 uses additive fragment-membership tables instead of
inheritance state."
```

---

### Task 3: Immutable `SemanticSnapshot` loader

**Files:**
- Modify: `graph/frontend/src/semantic.rs`
- Test: extend `graph/frontend/tests/semantic_schema.rs`

**Interfaces:**
- Produces:
```rust
pub struct SemanticSnapshot { /* private maps */ }
impl SemanticSnapshot {
    pub fn node_type(&self, name: &str) -> Option<&SemanticTypeInfo>;
    pub fn relationship_type(&self, name: &str) -> Option<&SemanticTypeInfo>;
    pub fn node_type_by_id(&self, id: ir::LabelId) -> Option<&SemanticTypeInfo>;
    pub fn relationship_type_by_id(&self, id: ir::RelationshipTypeId) -> Option<&SemanticTypeInfo>;
    pub fn endpoints(&self, relationship: ir::RelationshipTypeId) -> Option<&EndpointConstraint>;
}
pub struct SemanticTypeInfo {
    pub name: String,
    pub type_id: u32,                 // LabelId / RelationshipTypeId value
    pub source: ir::SourceTableId,
    /// folded property name → resolved property + physical column
    properties: HashMap<String, OwnedProperty>,
}
impl SemanticTypeInfo {
    pub fn property(&self, name: &str) -> Option<&OwnedProperty>;
}
pub struct OwnedProperty {
    pub id: ir::PropertyId,
    pub value_type: ir::ValueType,
    pub nullability: ir::Nullability,
    pub column: String,               // NEVER exposed through GraphCatalogSnapshot
}
pub struct EndpointConstraint {
    /// Allowed node type_ids; empty = unconstrained.
    pub start: Vec<u32>,
    pub end: Vec<u32>,
}
pub fn load_semantic_snapshot(connection: &Arc<Connection>, graph: &RegisteredGraph)
    -> Result<Option<SemanticSnapshot>, SemanticCatalogError>;
```
- `load_semantic_snapshot` returns `Ok(None)` when the semantic tables don't exist or hold no rows for this graph (legacy mode). Value types/nullability are derived at load time via `column_value_type` + the nullability rule from `schema_catalog.rs:293-302` (extract that too as `pub(crate) fn column_nullability(column: &Column) -> ir::Nullability` so both call sites share it). Loading fails loudly (`ColumnMissing`) if a mapped column no longer exists (spec risk: physical schema drift).

- [x] **Step 1: Write failing tests** (append to `tests/semantic_schema.rs`)

```rust
use turso_graph_frontend::load_semantic_snapshot;

#[test]
fn snapshot_reloads_identical_identities_across_connections() {
    let connection = connection();
    registered_graph(&connection);
    register_semantic_schema(&connection, "social", &semantic_registration()).expect("register");
    let graph = turso_graph_frontend::load_registered_graph(&connection, "social").expect("load");

    let first = load_semantic_snapshot(&connection, &graph)
        .expect("load snapshot")
        .expect("semantic mode");
    let customer = first.node_type("customer").expect("case-insensitive lookup");
    let supplier = first.node_type("Supplier").expect("supplier");
    assert_ne!(customer.type_id, supplier.type_id);
    // Shared property name resolves to the SAME stable PropertyId on both
    // owners even though nothing about column order guarantees it (spec
    // test matrix item 3).
    assert_eq!(
        customer.property("displayname").expect("owned").id,
        supplier.property("displayName").expect("owned").id,
    );
    // displayName maps to full_name (column 2) but its PropertyId must be
    // allocation-ordered (1), independent of column ordinal.
    assert_eq!(customer.property("displayName").unwrap().id.get(), 1);

    let second = load_semantic_snapshot(&connection, &graph)
        .expect("reload")
        .expect("semantic mode");
    assert_eq!(
        second.node_type("Customer").unwrap().type_id,
        customer.type_id
    );
}

#[test]
fn legacy_graph_without_semantic_rows_loads_none() {
    let connection = connection();
    registered_graph(&connection);
    let graph = turso_graph_frontend::load_registered_graph(&connection, "social").expect("load");
    assert!(load_semantic_snapshot(&connection, &graph)
        .expect("no error")
        .is_none());
}
```

Run: `rtk cargo test -p turso_graph_frontend --test semantic_schema`
Expected: FAIL — `load_semantic_snapshot` not found.

- [x] **Step 2: Implement the loader**

Read all four tables for `graph.id` (guard: if `SEMANTIC_TYPES_TABLE` absent from `sqlite_schema`, return `Ok(None)`; zero rows for this graph also `Ok(None)`). Build the maps keyed by `fold(name)`. For each ownership row resolve the source's table via `graph.node_sources`/`relationship_sources` by `SourceTableId`, then the column via `table.get_column_by_name`, then value type/nullability via the extracted `column_value_type`/`column_nullability`. `PropertyId::new(property_id)`, `LabelId::new(type_id)` failures map to `InvalidCatalogValue`.

- [x] **Step 3: Run tests** — `rtk cargo test -p turso_graph_frontend --test semantic_schema` → PASS.

- [x] **Step 4: Commit**

```bash
git add graph/frontend/src/semantic.rs graph/frontend/src/schema_catalog.rs graph/frontend/src/lib.rs graph/frontend/tests/semantic_schema.rs
git commit -S -m "feat(graph): load immutable semantic snapshot with persisted identities"
```

---

### Task 4: Catalog resolution contract — semantic mode in `GraphCatalogSnapshot` and `SchemaCatalog`

**Files:**
- Modify: `graph/frontend/src/binder.rs:8-35` (trait + supporting types), `graph/frontend/src/schema_catalog.rs` (struct + impls), `graph/frontend/src/session.rs:134-158` (open path)
- Test: inline tests in `schema_catalog.rs` + extend `tests/semantic_schema.rs`

**Interfaces:**
- Produces (additive trait methods with legacy defaults — every existing implementor, including `graph/testkit/src/dynamic_catalog.rs` and binder-test mocks, compiles unchanged):

```rust
/// Owner-aware property resolution result for semantic mode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PropertyResolution {
    /// Every possible concrete type owns the property compatibly.
    Resolved(ResolvedProperty),
    /// No possible type owns it.
    NotOwned { types: Vec<String> },
    /// Only a subset of the possible types owns it.
    Ambiguous { owners: Vec<String>, non_owners: Vec<String> },
}

pub trait GraphCatalogSnapshot {
    // ...existing six methods unchanged (binder.rs:22-34)...

    /// True when this graph has a registered semantic schema. Legacy
    /// catalogs keep the default.
    fn semantic_mode(&self, _graph: ir::GraphId) -> bool { false }

    /// Source backing one semantic node type. Legacy default: the single
    /// node source.
    fn node_source_for_label(&self, graph: ir::GraphId, _label: ir::LabelId)
        -> Option<ir::SourceTableId> { self.node_source(graph) }

    /// Source backing one semantic relationship type. Legacy default: the
    /// single relationship source.
    fn relationship_source_for_type(&self, graph: ir::GraphId, _rt: ir::RelationshipTypeId)
        -> Option<ir::SourceTableId> { self.relationship_source(graph) }

    /// Owner-aware property resolution. `type_names` is the binding's
    /// possible semantic types (declared label/relationship-type names);
    /// legacy default ignores them and delegates to `property`.
    fn resolve_owned_property(
        &self,
        graph: ir::GraphId,
        entity: CatalogEntity,
        type_names: &[String],
        name: &str,
    ) -> Option<PropertyResolution> {
        let _ = type_names;
        self.property(graph, entity, name).map(PropertyResolution::Resolved)
    }

    /// Endpoint constraint for a semantic relationship type as
    /// (start allowed LabelIds, end allowed LabelIds); None/empty = any.
    fn relationship_endpoints(&self, _graph: ir::GraphId, _rt: ir::RelationshipTypeId)
        -> Option<(Vec<ir::LabelId>, Vec<ir::LabelId>)> { None }
}
```

- `SchemaCatalog` gains the snapshot:
```rust
pub struct SchemaCatalog {
    connection: Arc<Connection>,
    graph: RegisteredGraph,
    semantic: Option<Arc<crate::semantic::SemanticSnapshot>>,
}
impl SchemaCatalog {
    pub fn new(connection: Arc<Connection>, graph: RegisteredGraph) -> Self; // semantic: None (legacy, existing callers unchanged)
    pub fn with_semantic(connection: Arc<Connection>, graph: RegisteredGraph,
                         semantic: Option<Arc<SemanticSnapshot>>) -> Self;
}
```

**Semantic-mode resolution semantics in `SchemaCatalog`'s `GraphCatalogSnapshot` impl** (`schema_catalog.rs:241-309`):
- `label(graph, name)`: semantic → `semantic.node_type(name).map(|t| LabelId::new(t.type_id))` (persisted ID, NOT source position). Legacy branch unchanged.
- `relationship_type(graph, name)`: same via `relationship_type`.
- `resolve_owned_property(...)`: semantic → for each name in `type_names` look up the type; all own the property (same `PropertyId`, compatible type — registration guaranteed compatibility) ⇒ `Resolved` (value type: exact if identical across owners, else `Any`); none ⇒ `NotOwned`; subset ⇒ `Ambiguous`. Empty `type_names` (unlabeled binding) in semantic mode ⇒ resolve against ALL node types: owned by every type ⇒ `Resolved`, by some ⇒ `Ambiguous`, by none ⇒ `NotOwned`.
- `property(graph, entity, name)` (legacy signature) in semantic mode: delegate to `resolve_owned_property` with empty `type_names` and map `Resolved` ⇒ `Some`, else `None` — so no path silently bypasses ownership.
- `node_source_for_label`/`relationship_source_for_type`: semantic → the type's mapped `SourceTableId`.
- `relationship_endpoints`: from `SemanticSnapshot::endpoints`, mapping type_ids to `LabelId`s.
- `RelationalCatalogSnapshot::property_column(source, property)` (`schema_catalog.rs:372-393`): semantic → look up by persisted `PropertyId` → owned column name for a type mapped to `source`. This is the ONLY place semantic property IDs meet physical column names.
- `RelationalCatalogSnapshot::label_name(label)` (`schema_catalog.rs:316-321`) and `relationship_type_name(rt)` (`schema_catalog.rs:327-348`): semantic → resolve the persisted ID through `SemanticSnapshot::node_type_by_id`/`relationship_type_by_id` and return the semantic type name. CRITICAL: the legacy implementations index `node_sources`/`relationship_sources` by `id - 1`; with persisted semantic IDs that returns the wrong name or `None`. Both label recording (`mutation.rs:1148-1178 record_node_labels`) and labeled-scan filtering (`lowering.rs:935-945`) resolve through these methods, so without the semantic branch, instance-to-type membership silently breaks for semantic graphs: created nodes would get no (or wrong) junction rows and labeled MATCH would filter on the wrong name.

**Session wiring** (`session.rs:148`): replace
```rust
let catalog = Arc::new(crate::SchemaCatalog::new(connection.clone(), graph.clone()));
```
with
```rust
let semantic = crate::semantic::load_semantic_snapshot(&connection, &graph)
    .map_err(Error::from)? // add a `Semantic(#[from] SemanticCatalogError)` variant to session::Error
    .map(Arc::new);
let catalog = Arc::new(crate::SchemaCatalog::with_semantic(
    connection.clone(), graph.clone(), semantic,
));
```

- [x] **Step 1: Write failing inline tests** in `schema_catalog.rs` tests module: register the Task 2 semantic fixture, build `SchemaCatalog::with_semantic`, assert: `label` returns the persisted ID for "Customer" and `None` for the source name "people_src" (spec: physical spelling must NOT resolve); `resolve_owned_property` returns `Resolved` for `["Customer"]`+"displayName", `NotOwned` for `["Supplier"]`+"born", `Ambiguous` for `[]`+"born"; `property_column` maps (`people_src` id, PropertyId 1) → "full_name"; `label_name` round-trips — `label_name(label("Customer"))` returns "Customer" and `relationship_type_name(relationship_type("TRADES_WITH"))` returns "TRADES_WITH" (the inverse resolution feeds junction recording and label filters; the legacy `id - 1` indexing would fail this for persisted IDs).
- [x] **Step 2: Run** `rtk cargo test -p turso_graph_frontend schema_catalog` → FAIL.
- [x] **Step 3: Implement** trait additions + `SchemaCatalog` changes + session wiring as specified above. Export `PropertyResolution` from `lib.rs` (`pub use binder::{..., PropertyResolution}`).
- [x] **Step 4: Run** full crate tests: `rtk cargo test -p turso_graph_frontend` and `rtk cargo test -p turso_graph_testkit` (proves `DynamicCatalog` and mocks compile unchanged via defaults) → PASS.
- [x] **Step 5: Commit** — `git commit -S -m "feat(graph): semantic-mode catalog resolution behind GraphCatalogSnapshot"`

---

### Task 5: Binder — semantic type sets and strict CREATE/MERGE type selection

**Files:**
- Modify: `graph/frontend/src/binder.rs`
- Test: extend `tests/semantic_schema.rs` (drive through `GraphConnection` for realism)

**Interfaces:**
- Consumes: `semantic_mode`, `node_source_for_label`, `relationship_source_for_type` (Task 4).
- Produces: new `BindError` variants used by later tasks:

```rust
#[error("CREATE/MERGE requires exactly one semantic type in strict mode at byte {span_start}..{span_end}")]
MissingSemanticType { entity: &'static str, span_start: usize, span_end: usize },
#[error("multiple semantic labels {names:?} are not supported before fragment-interface polymorphism (Milestone 3) at byte {span_start}..{span_end}")]
MultipleSemanticTypes { names: Vec<String>, span_start: usize, span_end: usize },
```

**Changes:**
1. `bind_created_node` (`binder.rs:1240-1291`): after the already-bound check, in semantic mode (`self.catalog.semantic_mode(self.graph)`):
   - `node.labels.is_empty()` ⇒ `MissingSemanticType { entity: "node", .. }` with `node.span`.
   - `node.labels.len() > 1` ⇒ `MultipleSemanticTypes` with the label names and `node.span`.
   - Resolve the label to `LabelId` (existing `resolve_labels`, which already errors `UnknownLabel` with spans), then `let source = self.catalog.node_source_for_label(self.graph, label_id)` instead of `self.node_source(node.span)?` (`:1262`).
2. Relationship creation (`binder.rs:1129`): `relationship.types.len() != 1` is already enforced (`:1121-1126`). Resolve the type first (existing code `:1144-1156`), then in semantic mode use `relationship_source_for_type(self.graph, type_id)` instead of `self.relationship_source(relationship.span)?`.
3. Legacy mode: zero behavior change — the old `node_source`/`relationship_source` calls remain in the `else` branch.

The possible-type set for match bindings needs no new state: `EntityBinding.names` (`binder.rs:218-222`) already stores declared label/type names at binding time, for both MATCH and CREATE paths (`new_entity_binding` calls at `:1130`, `:1263`, and the read-path equivalents around `:1856-2001`). Later tasks read it via `self.entities`.

- [x] **Step 1: Write failing tests** (append to `tests/semantic_schema.rs`; helper `open()` builds a `GraphConnection` via `turso_graph_frontend::Connection::open` after registering graph + semantic schema):

```rust
fn semantic_session() -> turso_graph_frontend::Connection {
    let connection = connection();
    registered_graph(&connection);
    register_semantic_schema(&connection, "social", &semantic_registration()).expect("register");
    turso_graph_frontend::Connection::open(connection, "social").expect("open")
}

#[test]
fn create_without_a_label_is_rejected_in_semantic_mode() {
    let session = semantic_session();
    let error = session
        .execute("CREATE (n {displayName: 'Ada'})", &Default::default())
        .expect_err("must reject untyped create");
    assert!(error.to_string().contains("exactly one semantic type"), "{error}");
}

#[test]
fn create_with_multiple_labels_is_rejected_before_fragment_polymorphism() {
    let session = semantic_session();
    let error = session
        .execute("CREATE (n:Customer:Supplier)", &Default::default())
        .expect_err("must reject multiple semantic labels");
    assert!(error.to_string().contains("multiple semantic labels"), "{error}");
}

#[test]
fn create_with_one_semantic_type_succeeds_and_source_name_is_not_a_label() {
    let session = semantic_session();
    session
        .execute("CREATE (n:Customer {displayName: 'Ada'})", &Default::default())
        .expect("typed create");
    // The physical source name must NOT act as a label in semantic mode.
    let error = session
        .execute("CREATE (n:people_src {displayName: 'Bob'})", &Default::default())
        .expect_err("source name is not a semantic type");
    assert!(error.to_string().contains("unknown label"), "{error}");
}

#[test]
fn legacy_graph_still_creates_untyped_nodes() {
    let connection = connection();
    registered_graph(&connection);
    let session = turso_graph_frontend::Connection::open(connection, "social").expect("open");
    session
        .execute("CREATE (n {full_name: 'Ada'})", &Default::default())
        .expect("legacy untyped create unchanged");
}
```

Run: `rtk cargo test -p turso_graph_frontend --test semantic_schema` → the three semantic tests FAIL (untyped create currently succeeds), legacy test PASSES.

- [x] **Step 2: Implement** changes 1-3 + the two `BindError` variants.
- [x] **Step 3: Run** the test file + full `rtk cargo test -p turso_graph_frontend` → PASS.
- [x] **Step 4: Commit** — `git commit -S -m "feat(graph): strict semantic type selection for CREATE/MERGE"`

---

### Task 6: Binder — owner-aware property resolution (reads and writes)

**Files:**
- Modify: `graph/frontend/src/binder.rs`
- Test: extend `tests/semantic_schema.rs`

**Interfaces:**
- Consumes: `resolve_owned_property`, `PropertyResolution` (Task 4); `EntityBinding.names`.
- Produces: `BindError` variants:

```rust
#[error("property `{name}` is not owned by semantic type(s) {types:?} at byte {span_start}..{span_end}")]
PropertyNotOwned { name: String, types: Vec<String>, span_start: usize, span_end: usize },
#[error("property `{name}` is owned by {owners:?} but not by {non_owners:?} at byte {span_start}..{span_end}")]
AmbiguousProperty { name: String, owners: Vec<String>, non_owners: Vec<String>, span_start: usize, span_end: usize },
```

**Changes:**
1. Change `resolve_property` (`binder.rs:4245-4257`) to take the owner context:
```rust
fn resolve_property(
    &self,
    entity: CatalogEntity,
    type_names: &[String],
    name: &cypher::Spanned<String>,
) -> Result<ResolvedProperty, BindError> {
    match self
        .catalog
        .resolve_owned_property(self.graph, entity, type_names, &name.value)
    {
        Some(PropertyResolution::Resolved(resolved)) => Ok(resolved),
        Some(PropertyResolution::NotOwned { types }) => Err(BindError::PropertyNotOwned {
            name: name.value.clone(),
            types,
            span_start: name.span.start,
            span_end: name.span.end,
        }),
        Some(PropertyResolution::Ambiguous { owners, non_owners }) => {
            Err(BindError::AmbiguousProperty {
                name: name.value.clone(),
                owners,
                non_owners,
                span_start: name.span.start,
                span_end: name.span.end,
            })
        }
        None => Err(BindError::UnknownProperty {
            name: name.value.clone(),
            span_start: name.span.start,
            span_end: name.span.end,
        }),
    }
}
```
2. Update every call site to pass the binding's `names`. Enumerate them first:
```bash
rg -n "resolve_property\(" graph/frontend/src/binder.rs
```
Known sites and their owner context:
   - `:1315` (`bind_set_item` SET n.prop): `resolve_mutation_target` (`:1489-1507`) already returns the binding — extend it to also return the names: `self.entities.get(&binding.id()).map(|entity| entity.names.clone())`.
   - `:1421` (`bind_remove`): same via `resolve_mutation_target`.
   - `:1464` (`bind_mutation_properties`, `:1456-1487`): change the signature to `fn bind_mutation_properties(&mut self, entity: CatalogEntity, type_names: &[String], properties: &[...])` and thread the creating pattern's declared names from the three call sites (`:1158` relationship types, `:1279` node labels, `:1369` replace-map — the last gets names via `resolve_set_variable` + `self.entities`).
   - Any read-path property-access site the `rg` sweep finds (property access in expressions resolves the variable binding first — take `names` from `self.entities.get(&binding_id)`; a property access on a non-entity value keeps its current error).
3. Read-path narrowing: `MATCH (n:Customer)` already stores `["Customer"]` in `EntityBinding.names`; unlabeled `MATCH (n)` stores `[]`, which Task 4 defined as "all types" resolution. No extra narrowing logic is needed for Milestones 1-2.

- [x] **Step 1: Write failing tests:**

```rust
#[test]
fn reads_reject_unowned_and_ambiguous_properties_with_spans() {
    let session = semantic_session();
    session.execute("CREATE (n:Customer {displayName: 'Ada', born: 1815})", &Default::default()).expect("seed");

    // Supplier does not own `born` (spec: PropertyNotOwned).
    let error = session
        .query("MATCH (n:Supplier) RETURN n.born", &Default::default())
        .expect_err("unowned read");
    assert!(error.to_string().contains("not owned"), "{error}");

    // Unlabeled binding: only Customer owns `born` (ambiguous subset).
    let error = session
        .query("MATCH (n) RETURN n.born", &Default::default())
        .expect_err("ambiguous read");
    assert!(error.to_string().contains("owned by"), "{error}");

    // Owned everywhere: fine unlabeled.
    session
        .query("MATCH (n) RETURN n.displayName", &Default::default())
        .expect("displayName owned by all types");
}

#[test]
fn writes_reject_unowned_properties_on_every_route() {
    let session = semantic_session();
    session.execute("CREATE (n:Customer {displayName: 'Ada'})", &Default::default()).expect("seed");

    for query in [
        "CREATE (n:Supplier {born: 1815})",                       // create properties
        "MATCH (n:Supplier) SET n.born = 1815",                   // SET
        "MATCH (n:Supplier) REMOVE n.born",                       // REMOVE
        "MATCH (n:Supplier) SET n = {born: 1815}",                // literal map replace
        "MERGE (n:Supplier {displayName: 'S'}) ON CREATE SET n.born = 1", // ON CREATE
        "MERGE (n:Supplier {displayName: 'S'}) ON MATCH SET n.born = 1",  // ON MATCH
    ] {
        let error = session.execute(query, &Default::default()).expect_err(query);
        assert!(error.to_string().contains("not owned"), "{query}: {error}");
    }
}

#[test]
fn typed_reads_and_writes_still_work_in_semantic_mode() {
    let session = semantic_session();
    session.execute(
        "CREATE (c:Customer {displayName: 'Ada', born: 1815})-[:TRADES_WITH {since: 1840}]->(s:Supplier {displayName: 'Iron Co'})",
        &Default::default(),
    ).expect("typed create path");
    let rows = session.query(
        "MATCH (c:Customer)-[t:TRADES_WITH]->(s:Supplier) RETURN c.displayName, t.since, s.displayName",
        &Default::default(),
    ).expect("typed read");
    assert_eq!(rows.len(), 1);
}
```

Run → FAIL (unowned property reads/writes currently resolve by column name).

- [x] **Step 2: Implement** changes 1-3.
- [x] **Step 3: Run** the test file, then the full gate trio (`turso_graph_frontend`, `turso_graph_testkit`, smoke) — legacy behavior must be untouched because every legacy catalog resolves through the default `resolve_owned_property` ⇒ `Resolved`.
- [x] **Step 4: Commit** — `git commit -S -m "feat(graph): owner-aware semantic property resolution in binder"`

---

### Task 7: Binder — endpoint validation for relationship CREATE/MERGE (both directions)

**Files:**
- Modify: `graph/frontend/src/binder.rs:1100-1185`
- Test: extend `tests/semantic_schema.rs`

**Interfaces:**
- Consumes: `relationship_endpoints` (Task 4), `EntityBinding.names`, direction handling at `binder.rs:1160-1165`.
- Produces: `BindError` variant:

```rust
#[error("semantic type(s) {node_types:?} are not allowed as the {endpoint} endpoint of `{relationship_type}` at byte {span_start}..{span_end}")]
InvalidEndpointType {
    relationship_type: String,
    endpoint: &'static str,
    node_types: Vec<String>,
    span_start: usize,
    span_end: usize,
},
```

**Change:** in the relationship-creation loop (`binder.rs:1103-1185`), after `relationship_types` resolve (`:1156`) and after `(relationship_from, relationship_to)` are chosen (`:1160-1165` — this pair already swaps for `Direction::Incoming`, so checking `relationship_from` against `start` and `relationship_to` against `end` covers both syntaxes), add in semantic mode:

```rust
if let Some((start_allowed, end_allowed)) = self
    .catalog
    .relationship_endpoints(self.graph, relationship_types[0])
{
    for (endpoint, binding_id, allowed) in [
        ("start", relationship_from, &start_allowed),
        ("end", relationship_to, &end_allowed),
    ] {
        if allowed.is_empty() {
            continue;
        }
        let names = self
            .entities
            .get(&binding_id)
            .map(|entity| entity.names.clone())
            .unwrap_or_default();
        // Task 5 guarantees created nodes carry exactly one semantic label
        // in strict mode; matched endpoint bindings carry their declared
        // labels. Every declared type must be allowed.
        let allowed_names: Vec<ir::LabelId> = allowed.clone();
        let all_allowed = !names.is_empty()
            && names.iter().all(|name| {
                self.catalog
                    .label(self.graph, name)
                    .is_some_and(|label| allowed_names.contains(&label))
            });
        if !all_allowed {
            return Err(BindError::InvalidEndpointType {
                relationship_type: relationship.types[0].value.clone(),
                endpoint,
                node_types: names,
                span_start: relationship.span.start,
                span_end: relationship.span.end,
            });
        }
    }
}
```

(An unlabeled matched endpoint — `names` empty — fails closed with `InvalidEndpointType` when the endpoint is constrained: the binder cannot prove the type statically in Milestones 1-2.)

- [x] **Step 1: Failing tests:**

```rust
#[test]
fn endpoint_validation_covers_both_directions() {
    let session = semantic_session();
    // Outgoing, correct: Customer -> Supplier.
    session.execute(
        "CREATE (:Customer {displayName: 'A'})-[:TRADES_WITH]->(:Supplier {displayName: 'B'})",
        &Default::default(),
    ).expect("valid outgoing");
    // Incoming syntax, same semantics: Supplier <- Customer is still start=Customer.
    session.execute(
        "CREATE (:Supplier {displayName: 'C'})<-[:TRADES_WITH]-(:Customer {displayName: 'D'})",
        &Default::default(),
    ).expect("valid incoming syntax");
    // Wrong: Supplier -> Customer.
    let error = session.execute(
        "CREATE (:Supplier {displayName: 'E'})-[:TRADES_WITH]->(:Customer {displayName: 'F'})",
        &Default::default(),
    ).expect_err("start endpoint must be Customer");
    assert!(error.to_string().contains("not allowed as the start endpoint"), "{error}");
    // Wrong via incoming syntax too (direction reversal must swap checks).
    let error = session.execute(
        "CREATE (:Customer {displayName: 'G'})<-[:TRADES_WITH]-(:Supplier {displayName: 'H'})",
        &Default::default(),
    ).expect_err("incoming reversal");
    assert!(error.to_string().contains("endpoint"), "{error}");
}
```

Run → FAIL.
- [x] **Step 2: Implement.** Run → PASS + full crate tests.
- [x] **Step 3: Commit** — `git commit -S -m "feat(graph): validate semantic relationship endpoints in both directions"`

---

### Task 8: Binder — static value-type checks for typed mutation properties

**Files:**
- Modify: `graph/frontend/src/binder.rs` (`bind_mutation_properties` `:1456-1487`, `bind_set_item` `:1305-1340`)
- Test: extend `tests/semantic_schema.rs`

**Interfaces:**
- Consumes: `ResolvedProperty.value_type` already returned by `resolve_property`; bound expression `ir::TypedExpression.value_type`.
- Produces: `BindError` variant:

```rust
#[error("value of type {actual:?} is not assignable to property `{property}` of type {expected:?} at byte {span_start}..{span_end}")]
IncompatiblePropertyValue {
    property: String,
    expected: ir::ValueType,
    actual: ir::ValueType,
    span_start: usize,
    span_end: usize,
},
```

**Change:** add one helper and call it from both property-writing paths after binding the value expression (semantic mode only):

```rust
/// Statically known type mismatches fail at bind time; `Any` on either
/// side (parameters, dynamic maps, untyped columns) defers to the runtime
/// validator (Task 9) instead of rejecting prematurely (spec M2 item 5).
fn check_static_property_value(
    property_name: &str,
    expected: &ir::ValueType,
    value: &ir::TypedExpression,
    span: cypher::Span,
) -> Result<(), BindError> {
    let actual = &value.value_type;
    let compatible = matches!(expected, ir::ValueType::Any)
        || matches!(actual, ir::ValueType::Any)
        || expected == actual
        // Integer literals assign to Real columns (SQLite numeric affinity).
        || (matches!(expected, ir::ValueType::Real) && matches!(actual, ir::ValueType::Integer));
    if compatible {
        Ok(())
    } else {
        Err(BindError::IncompatiblePropertyValue {
            property: property_name.to_owned(),
            expected: expected.clone(),
            actual: actual.clone(),
            span_start: span.start,
            span_end: span.end,
        })
    }
}
```

Call sites: `bind_mutation_properties` (after the `bound_value` match, `:1465-1480`) and `bind_set_item`'s `SetItem::Property` arm (after `bound`, `:1316-1333`). Both already have the resolved property and value span in scope.

- [x] **Step 1: Failing tests:**

```rust
#[test]
fn statically_wrong_value_types_fail_at_bind_time() {
    let session = semantic_session();
    // born maps to birth_year INTEGER.
    let error = session
        .execute("CREATE (n:Customer {displayName: 'Ada', born: 'yesterday'})", &Default::default())
        .expect_err("Text into Integer property");
    assert!(error.to_string().contains("not assignable"), "{error}");
    let error = session
        .execute("MATCH (n:Customer) SET n.born = 'old'", &Default::default())
        .expect_err("SET Text into Integer");
    assert!(error.to_string().contains("not assignable"), "{error}");
    // Parameters are Any at bind time: must NOT fail here (runtime's job).
    session
        .prepare("CREATE (n:Customer {displayName: 'Ada', born: $b})")
        .expect("parameter defers to runtime");
}
```

(Adjust the parameter assertion to the actual prepare/execute API for mutations with parameters — `session.execute` with a `Parameters` map containing `b` → integer works end-to-end; the negative case is covered in Task 9.)

Run → FAIL.
- [x] **Step 2: Implement.** Run → PASS + full crate suite.
- [x] **Step 3: Commit** — `git commit -S -m "feat(graph): bind-time static type checks for semantic property writes"`

---

### Task 9: Runtime — parameter/Any value validation before physical mutation

**Files:**
- Modify: `graph/frontend/src/compiler.rs` (accessor on `GraphCompilationCatalog`), `graph/frontend/src/schema_catalog.rs` (impl), `graph/frontend/src/mutation.rs` (validation call sites)
- Test: extend `tests/semantic_schema.rs`

**Interfaces:**
- Produces additive trait method on `GraphCompilationCatalog` (`compiler.rs` — the trait combining `GraphCatalogSnapshot + RelationalCatalogSnapshot` that `execute_cypher_mutation` receives, `mutation.rs:57-62`):

```rust
/// Expected value type for a property on a source in semantic mode.
/// Legacy catalogs keep the default (no runtime semantic checks).
fn semantic_property_type(
    &self,
    _source: ir::SourceTableId,
    _property: ir::PropertyId,
) -> Option<(String, ir::ValueType)> { None }  // (property name, expected type)
```
`SchemaCatalog` implements it from `SemanticSnapshot` (property id → name + value type for any type mapped to that source).
- Produces `MutationError` variant (`mutation.rs:33-55`):

```rust
#[error("runtime value for property `{property}` is not assignable to {expected:?}")]
IncompatibleRuntimeValue { property: String, expected: ir::ValueType },
```
- Produces the shared validator in `mutation.rs`:

```rust
/// Runtime analog of the binder's static check, for values that were
/// `Any` at bind time (parameters, dynamic map values). Runs BEFORE the
/// physical INSERT/UPDATE SQL for the row is issued; the mutation's
/// enclosing transaction (mutation.rs:57-130 savepoint discipline) makes
/// a failure abort the whole graph mutation with zero partial writes.
fn check_runtime_value(
    catalog: &dyn GraphCompilationCatalog,
    source: ir::SourceTableId,
    property: ir::PropertyId,
    value: &Value,
) -> Result<(), MutationError> {
    let Some((name, expected)) = catalog.semantic_property_type(source, property) else {
        return Ok(()); // legacy mode
    };
    let ok = match (&expected, value) {
        (_, Value::Null) => true, // nullability is the physical column's job
        (ir::ValueType::Any, _) => true,
        (ir::ValueType::Integer, Value::Numeric(turso_core::Numeric::Integer(_))) => true,
        (ir::ValueType::Real, Value::Numeric(_)) => true, // Integer widens to Real
        (ir::ValueType::Text, Value::Text(_)) => true,
        (ir::ValueType::Bytes, Value::Blob(_)) => true,
        // Struct/Union/List/Custom columns accept their existing lowering
        // representation (JSON text / blob); delegate to the physical layer
        // exactly as today rather than double-validating shapes here.
        (ir::ValueType::Struct(_) | ir::ValueType::Union(_) | ir::ValueType::List(_)
            | ir::ValueType::Custom { .. } | ir::ValueType::Map | ir::ValueType::Vector(..), _) => true,
        _ => false,
    };
    if ok { Ok(()) } else {
        Err(MutationError::IncompatibleRuntimeValue { property: name, expected })
    }
}
```

**Call sites:** every point in `mutation.rs` where a bound `ir::PropertyValue` (or SET value) has been evaluated to a concrete `Value` and is about to be written. Enumerate them with `rg -n "PropertyValue|SetProperty" graph/frontend/src/mutation.rs`; the create-node path, create-relationship path (`:1320-1340` area), `SetProperty` execution (`:975-1010` area), and merge ON CREATE/ON MATCH actions all funnel evaluated values into row writes — insert `check_runtime_value(catalog.as_ref(), source, property, &value)?` immediately after evaluation in each.

- [x] **Step 1: Failing tests:**

```rust
#[test]
fn wrong_parameter_values_fail_at_runtime_with_zero_partial_writes() {
    let session = semantic_session();
    let mut parameters = turso_graph_frontend::mutation::Parameters::new();
    parameters.insert("b".to_owned(), turso_graph_frontend::Value::Text("old".into()));
    // Two creates in one statement; the second carries the bad parameter.
    let error = session
        .execute(
            "CREATE (a:Customer {displayName: 'First'}) CREATE (b:Customer {displayName: 'Second', born: $b})",
            &parameters,
        )
        .expect_err("Text parameter into Integer property");
    assert!(error.to_string().contains("not assignable"), "{error}");
    // Atomicity: the first create must have rolled back too.
    let rows = session
        .query("MATCH (n:Customer) RETURN n.displayName", &Default::default())
        .expect("read back");
    assert!(rows.is_empty(), "partial write leaked: {rows:?}");
}
```

(Use the real `Parameters` re-export path — `lib.rs:39` exports `Parameters` from `mutation`.)

Run → FAIL (bad value currently reaches SQL and either stores coerced or errors differently).
- [x] **Step 2: Implement** trait method + impl + validator + call-site insertion.
- [x] **Step 3: Run** test file + full crate suite → PASS.
- [x] **Step 4: Commit** — `git commit -S -m "feat(graph): runtime semantic validation for parameter values"`

---

### Task 10: Runtime — dynamic map replacement and staged-mutation atomicity

**Files:**
- Modify: `graph/frontend/src/mutation.rs` (`ReplacePropertiesDynamic` execution, `:1066-1080` area)
- Test: extend `tests/semantic_schema.rs`

**Interfaces:**
- Consumes: `check_runtime_value` (Task 9); additive `GraphCompilationCatalog` method:

```rust
/// Resolve a dynamic map key to an owned property on this source in
/// semantic mode. None in legacy mode (caller keeps today's
/// payload_columns behavior); Some(None) = key is not an owned property.
fn semantic_property_for_key(
    &self,
    _source: ir::SourceTableId,
    _key: &str,
) -> Option<Option<(ir::PropertyId, String, ir::ValueType)>> { None }
```
- Produces `MutationError` variant:

```rust
#[error("dynamic map key `{key}` is not an owned semantic property")]
UnknownDynamicKey { key: String },
```

**Change:** in the `ir::Mutation::ReplacePropertiesDynamic` arm (`mutation.rs:1066`), before mutating any row: iterate the evaluated map's keys; in semantic mode (`semantic_property_for_key` returns `Some(..)`), resolve every key first — any `Some(None)` ⇒ `UnknownDynamicKey`; then `check_runtime_value` every value against its resolved type; only then proceed to the column updates (which now target the resolved columns rather than raw `payload_columns` names). All checks complete before the first UPDATE statement for the entity.

- [x] **Step 1: Failing tests:**

```rust
#[test]
fn dynamic_map_replacement_rejects_unknown_keys_and_bad_values_atomically() {
    let session = semantic_session();
    session.execute("CREATE (n:Customer {displayName: 'Ada', born: 1815})", &Default::default()).expect("seed");

    let mut parameters = turso_graph_frontend::mutation::Parameters::new();
    // properties(m)-style dynamic map via parameter.
    parameters.insert("m".to_owned(), /* map value: {"displayName": "Eve", "ghost": 1} —
        construct with the Value map representation used by existing
        mutation tests for map parameters; check mutation.rs tests for the
        exact constructor */ todo_value());
    let error = session
        .execute("MATCH (n:Customer) SET n = $m", &parameters)
        .expect_err("unknown dynamic key");
    assert!(error.to_string().contains("ghost"), "{error}");

    // Original row untouched.
    let rows = session
        .query("MATCH (n:Customer) RETURN n.displayName", &Default::default())
        .expect("read back");
    assert_eq!(rows[0][0], turso_graph_frontend::Value::Text("Ada".into()));
}

#[test]
fn one_invalid_row_in_a_staged_mutation_aborts_everything() {
    let session = semantic_session();
    let mut parameters = turso_graph_frontend::mutation::Parameters::new();
    parameters.insert("bad".to_owned(), turso_graph_frontend::Value::Text("x".into()));
    // UNWIND creates two rows; the second write is invalid at runtime.
    let error = session
        .execute(
            "UNWIND [1, 2] AS i \
             CREATE (n:Customer {displayName: 'row', born: CASE i WHEN 1 THEN 1815 ELSE $bad END})",
            &parameters,
        )
        .expect_err("second row invalid");
    assert!(error.to_string().contains("not assignable"), "{error}");
    let rows = session
        .query("MATCH (n:Customer) RETURN n", &Default::default())
        .expect("read back");
    assert!(rows.is_empty(), "staged mutation leaked rows: {rows:?}");
}
```

Before writing the first test, find the existing map-parameter representation: `rg -n "Value::" graph/frontend/src/mutation.rs | rg -i "map|json" | head` and mirror it (the `todo_value()` placeholder above MUST be replaced with that concrete constructor during this step — it is a test-authoring lookup, not an implementation unknown).

Run → FAIL.
- [x] **Step 2: Implement.** Run → PASS + full crate suite.
- [x] **Step 3: Commit** — `git commit -S -m "feat(graph): atomic semantic validation for dynamic map mutations"`

---

### Task 11: Compatibility sweep — legacy graphs, testkit, snapshot staleness

**Files:**
- Test: extend `tests/semantic_schema.rs`; no production code expected (fixes only if the sweep finds regressions)

- [x] **Step 1: Snapshot-staleness test** (spec test matrix item 10):

```rust
#[test]
fn semantic_registration_bumps_generation_so_snapshots_rebuild() {
    let connection = connection();
    registered_graph(&connection);
    let before = turso_graph_frontend::graph_generation(&connection, "social").expect("gen");
    register_semantic_schema(&connection, "social", &semantic_registration()).expect("register");
    let after = turso_graph_frontend::graph_generation(&connection, "social").expect("gen");
    assert!(after > before, "semantic registration must invalidate traversal snapshots");
}
```

- [x] **Step 2: Run the full verification battery:**

```bash
rtk cargo fmt --all -- --check
rtk cargo test -p turso_graph_ir
rtk cargo test -p turso_graph_frontend
rtk cargo test -p turso_graph_testkit
rtk cargo run -q -p turso_graph_testkit -- run smoke --no-record
rtk cargo run -q -p turso_graph_testkit -- corpus --no-record
rtk cargo clippy -p turso_graph_ir -p turso_graph_frontend -p turso_graph_testkit --all-features --all-targets -- --deny=warnings
rtk git diff --check
```
Expected: all PASS; corpus results match the Task 1 baseline (donor `DynamicCatalog` untouched — verify `git diff --stat graph/testkit/src/dynamic_catalog.rs` is empty).

- [x] **Step 3: Commit** — `git commit -S -m "test(graph): semantic overlay compatibility and snapshot-staleness coverage"`

---

### Task 12: Documentation and prepare-time benchmark

**Files:**
- Modify: `docs/graph.md`, `graph/README.md`
- Create: benchmark in the pattern the graph crates already use (`turso_graph_runtime` has divan dev-deps; check `graph/frontend/Cargo.toml` bench setup first — add `[[bench]]` with `#[turso_macros::divan_bench]` per the CLAUDE.md benchmark-naming rule)

- [x] **Step 1: Documentation.** Add to `docs/graph.md`: a registration example (the Task 2 fixture verbatim), the strict-mode validation behaviors with example error messages, legacy-mode guarantee, and this exact boundary statement (spec Slice 4.2 + MUST NOT list):

> Semantic schema is an opt-in overlay validated by the graph frontend. It is inspired by TypeDB's conceptual data model but is not TypeDB, TypeQL, or PERA compatible: no inheritance (polymorphism, when it arrives in Milestone 3, is composition over fragment interfaces, not subtyping), no attribute instances, no named roles or n-ary relations, and no inference. Integrity is enforced for graph-frontend reads and writes plus any physical SQL constraints on the backing tables; direct SQL against backing tables is not semantically validated.

Link from `graph/README.md` in one sentence.

- [x] **Step 2: Benchmark.** Measure `GraphConnection::open` (catalog + snapshot construction) and one `prepare` of `MATCH (c:Customer)-[t:TRADES_WITH]->(s:Supplier) RETURN c.displayName` for (a) legacy graph, (b) semantic graph. No pass/fail threshold (spec Slice 4.3) — the benchmark exists to catch accidental per-property catalog SQL later. Assert in a companion test that `load_semantic_snapshot` issues a bounded number of queries (≤ 5: one per semantic table + existence check).

- [x] **Step 3: Final gates** (same battery as Task 11 Step 2) → all PASS.

- [x] **Step 4: Commit** — `git commit -S -m "docs(graph): semantic schema overlay usage and integrity boundary"`

---

## Self-Review Notes

- **Spec coverage:** M1 items 1-6 → Tasks 1-4 (item 6's version decision: generation bump, documented in Task 2 commit). M2 items 1-8 → Tasks 5-10 (item 1's type tracking is pre-existing `EntityBinding.names`; item 2 → Task 6; items 3-5 → Tasks 5, 6, 8, 9; item 6 → Task 7; item 7 → Task 10; item 8 → error variants across Tasks 5-10). Spec test matrix items 1-3 → Tasks 2-3 fixtures; 4 → Tasks 1-2; 5 → Tasks 3, 5, 11; 6-7 → Tasks 6-8; 8 → Tasks 9-10; 9 → Task 7; 10 → Tasks 3, 11; 11 → Tasks 4, 11.
- **Known lookups deferred to execution** (verifications, not design gaps): exact `session::Error` variant plumbing (Task 4), the full `rg` sweep of `resolve_property` read-path call sites (Task 6), mutation.rs evaluated-value insertion points (Task 9), and the map-parameter `Value` constructor (Task 10 — flagged inline as MUST-replace).
- **Out of scope, do not add:** fragment-interface polymorphism (amended Milestone 3: fragment catalog, membership tables, fragment-label scans), inheritance or abstract-type machinery in any form, constraint catalog, multi-source-per-type scans, `DynamicCatalog` semantic support, conformance re-baselining, tessera dependency.
