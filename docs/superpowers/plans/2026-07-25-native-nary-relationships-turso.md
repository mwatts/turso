# Native N-ary Relationships (Turso graph frontend) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the binary relationship code path in the Turso graph frontend with a native role model, so a relation may declare any number of named roles with target types, optionality, and cardinality, and a two-role relation lands on exactly the physical shape it has today.

**Architecture:** A new `RoleId` identity joins the IR identity set. Physical registration stops naming `start_column`/`end_column` and instead names an ordered list of roles, each with a column and a target node source; a `RelationshipSourceRegistration::binary` constructor keeps every existing two-endpoint call site a one-line edit. `FixedExpand` becomes `RoleExpand` carrying an ordered `(from_role, to_role)` pair instead of a `Direction`; `CreateRelationship` becomes `CreateRelation` carrying `Vec<RoleBinding>` instead of `from`/`to`. The parser gains a standalone `[var:Type {props}](role: player, …)` pattern and desugars the arrow forms into role pairs before the AST reaches the binder. Semantic mode persists per-role target types, optionality, and cardinality in a new catalog table; schemaless mode synthesizes two required single-valued roles named `start` and `end` from the registration.

**Tech Stack:** Rust 2021, `turso_graph_ir` / `turso_graph_frontend` / `turso_graph_runtime` / `turso_cypher` crates, `pest` grammar (`graph/cypher/src/cypher.pest`), `thiserror`, SQLite-compatible storage through `turso_core`.

## Global Constraints

- Scope is the Turso graph frontend only: IR, catalog, parser, binder, lowering, runtime, storage layout. Tessera, foedus, and limen are separate specs with separate plans; do not touch them.
- Out of scope: role interfaces or role polymorphism across relation types; inference and rules (Decision Gate C stays deferred); constraints beyond role target types and cardinality.
- Binary is not a separate kind. It is deleted as a code path and kept as a layout of the one role model. Do not add an `if roles.len() == 2` fast path anywhere.
- A two-role, all-required, all-`One` relation must land on exactly the physical shape it has today: two indexed endpoint columns on one table, plus the composite pair index.
- Fresh start. No migration, no dual-read. A graph catalog predating roles fails loudly at open with `CatalogError::IncompatibleGraphLayout`.
- Test-driven throughout: every step writes a failing test, verifies it fails for the intended reason, then makes it pass.
- Never build or run with `--release`, except the two `mise` graph tasks below, which are release by design.
- Merge gates, applied **per task**, not once at the end:
  - `mise run corpus`: at least **8,926** passed, no new failure family. (8,926 / 10,242 is the latest recorded run, `run_id` `20260725T205828.143422Z-e068dc04c359-corpus-deep`.)
  - `mise run cypherbench-sample`: parity with the recorded baseline.
  - `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
  - `cargo fmt --check`
  - `cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime -p turso_cypher` and `cargo test -p turso_core --lib`
- Commit messages use the repository convention: `[scope: ]<imperative summary>` with a body explaining intent. Conventional-Commit prefixes are not required here. Sign every commit (`git commit -S`).

## Three spec gaps resolved in this plan

The approved design leaves three things underspecified. Each is resolved here; a
reviewer who disagrees should say so before Task 4.

1. **`RoleExpand` must keep `from_node_source`.** The spec's struct literal drops
   it, but `lower_fixed_expand` uses it (`graph/frontend/src/lowering.rs:1420`) to
   filter the input row set when a binding can come from more than one node
   source. Dropping it silently widens multi-source expands. The field stays.
2. **Undirected patterns need a symmetric flag.** The spec says `Direction`
   becomes an ordered role pair and that the undirected form "binds the pair in
   both orders." An ordered pair cannot express that on its own. Today the binder
   already expands `(a)-[r]-(b)` into a `Union` of two directed branches when the
   endpoint sources differ, and only emits `Direction::Both` when both endpoints
   come from the same source (`graph/frontend/src/binder.rs:2717`). `RoleExpand`
   therefore carries `symmetric: bool`, which means "also match the reversed
   pair" and maps one-to-one onto today's `Direction::Both` lowering.
3. **Role updates after create need a surface syntax.** The spec lists role
   updates in both the decisions table and the test list but gives no syntax.
   This plan uses `SET [t](scribe: s2)` — the standalone role pattern in a `SET`
   item. `[` cannot begin a `set_item` today, so it introduces no ambiguity, and
   it mirrors the create form exactly. **Assumption, not a decision the spec
   made.** Task 15 implements it.

## File Structure

**Created:**

- `graph/ir/src/role.rs` — `RoleDef`, `RoleTarget`, `RoleCardinality`, `RoleBinding`. The role model, independent of catalog or plan concerns.
- `graph/frontend/tests/nary_relations.rs` — the spec's end-to-end role test list.
- `graph/frontend/tests/desugaring_golden.rs` — proves `(a)-[r:KNOWS]->(b)` and `[r:KNOWS](start: a, end: b)` bind to identical IR.

**Modified:**

- `graph/ir/src/identity.rs` — add `RoleId`.
- `graph/ir/src/lib.rs` — module and re-exports.
- `graph/ir/src/plan.rs` — `FixedExpand` → `RoleExpand`; `GraphExpand` role pair.
- `graph/ir/src/scope.rs` — delete `Direction`.
- `graph/ir/src/mutation.rs` — `CreateRelationship` → `CreateRelation`.
- `graph/ir/src/semantics.rs` — `SEMANTIC_PROFILE_VERSION` 2 → 3, `path_policy_version` 1 → 2.
- `graph/ir/tests/semantic_profile_pin.rs` — re-pin the digest.
- `graph/frontend/src/catalog.rs` — role-shaped relationship source registration, role catalog table, per-role and per-pair indexes, `IncompatibleGraphLayout`.
- `graph/frontend/src/semantic.rs` — `SemanticRole`, `graph_semantic_role` table, role target types replacing `EndpointConstraint`.
- `graph/frontend/src/schema_catalog.rs` — role-shaped `RelationshipTableLayout`, structural columns.
- `graph/frontend/src/lowering.rs` — `RelationshipTableLayout` roles, `RoleExpand` lowering, spill-table hops.
- `graph/frontend/src/binder.rs` — role resolution, standalone role pattern, role-edge read sugar, five new errors.
- `graph/frontend/src/mutation.rs` — n-role insert, spill inserts, role updates.
- `graph/frontend/src/graph_expand.rs` — role-pair columns replacing the direction column.
- `graph/frontend/src/snapshot.rs` — role-pair edge extraction.
- `graph/cypher/src/cypher.pest`, `graph/cypher/src/ast.rs`, `graph/cypher/src/parser.rs` — standalone role pattern.
- `graph/runtime/src/csr.rs`, `graph/runtime/src/traversal.rs`, `graph/runtime/src/path_policy.rs` — role-pair adjacency, role-annotated path elements, the k > 2 legality rule.
- `docs/graph.md`, `graph/CONFORMANCE.md`, `.specs/graph-semantic-schema-overlay.agent-spec.md`, `docs/superpowers/plans/2026-07-22-graph-semantic-schema-overlay.md` — Gate B deletion and role-model documentation.

## Task sequence rationale

Rust cannot land a type change in three separately-compiling pieces, so the
`Direction` removal and the `CreateRelationship` rewrite each use expand/contract:
add the role fields alongside the old ones (Task 5, Task 9), switch consumers
(Task 6, Task 10), delete the old fields (Task 7, Task 11). Every task compiles,
every task runs the full gate set.

---

### Task 1: `RoleId` identity and the role model

**Files:**
- Modify: `graph/ir/src/identity.rs:59-92`
- Create: `graph/ir/src/role.rs`
- Modify: `graph/ir/src/lib.rs:9-41`
- Test: `graph/ir/src/identity.rs` (existing `mod tests`), `graph/ir/src/role.rs` (new `mod tests`)

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces:
  - `turso_graph_ir::RoleId` — non-zero `u32` identity with `RoleId::new(u32) -> Result<Self, InvalidId>` and `RoleId::get(self) -> u32`.
  - `turso_graph_ir::RoleTarget` — `enum { Node(LabelId), Relation(RelationshipTypeId) }`.
  - `turso_graph_ir::RoleCardinality` — `enum { One, Many }`.
  - `turso_graph_ir::RoleDef { role: RoleId, name: String, target_types: Vec<RoleTarget>, optional: bool, cardinality: RoleCardinality }`.
  - `turso_graph_ir::RoleBinding { role: RoleId, value: BindingId }`.

- [ ] **Step 1: Write the failing identity test**

In `graph/ir/src/identity.rs`, inside `mod tests`, extend
`all_public_identities_reject_zero` with a `RoleId` line:

```rust
        assert!(RoleId::new(0).is_err());
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p turso_graph_ir all_public_identities_reject_zero`
Expected: FAIL to compile with `cannot find type RoleId in this scope`.

- [ ] **Step 3: Define `RoleId`**

In `graph/ir/src/identity.rs`, after the `PropertyId` line (`identity.rs:65`):

```rust
define_u32_id!(RoleId, "role");
```

- [ ] **Step 4: Run it to verify it passes**

Run: `cargo test -p turso_graph_ir all_public_identities_reject_zero`
Expected: PASS.

- [ ] **Step 5: Write the failing role-model test**

Create `graph/ir/src/role.rs` containing only its test module for now:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LabelId, RelationshipTypeId};

    #[test]
    fn a_role_target_keeps_node_and_relation_identity_spaces_distinct() {
        // A node label and a relationship type may share the numeric value 1.
        // Flattening them into one integer would make a role that accepts
        // Person silently accept the relationship type numbered 1.
        let node = RoleTarget::Node(LabelId::new(1).unwrap());
        let relation = RoleTarget::Relation(RelationshipTypeId::new(1).unwrap());
        assert_ne!(node, relation);
    }

    #[test]
    fn a_role_definition_carries_optionality_cardinality_and_targets() {
        let scribe = RoleDef {
            role: RoleId::new(1).unwrap(),
            name: "scribe".to_owned(),
            target_types: vec![RoleTarget::Node(LabelId::new(7).unwrap())],
            optional: false,
            cardinality: RoleCardinality::One,
        };
        assert!(!scribe.optional);
        assert_eq!(scribe.cardinality, RoleCardinality::One);
        assert_eq!(scribe.target_types.len(), 1);
    }

    #[test]
    fn empty_target_types_mean_unconstrained_not_uninhabited() {
        // An empty list is the schemaless default. Reading it as "no player is
        // allowed" would make every schemaless create fail.
        let any = RoleDef {
            role: RoleId::new(2).unwrap(),
            name: "start".to_owned(),
            target_types: Vec::new(),
            optional: false,
            cardinality: RoleCardinality::One,
        };
        assert!(any.accepts_any_target());
    }
}
```

- [ ] **Step 6: Wire the module and run to verify it fails**

Add to `graph/ir/src/lib.rs` beside the other `mod` lines:

```rust
mod role;
```

Run: `cargo test -p turso_graph_ir --lib role::`
Expected: FAIL to compile with `cannot find type RoleTarget in this scope`.

- [ ] **Step 7: Write the role model**

Prepend to `graph/ir/src/role.rs`, above the test module:

```rust
use crate::{BindingId, LabelId, RelationshipTypeId, RoleId};

/// What a player of a role may be.
///
/// A role player is either a node of some label or a relation of some type.
/// The two identity spaces stay distinct rather than being flattened: label 1
/// and relationship type 1 are different things, and a role that accepts one
/// must not accept the other.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RoleTarget {
    Node(LabelId),
    Relation(RelationshipTypeId),
}

/// How many players one role may hold in one relation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RoleCardinality {
    /// Exactly one player, stored in an indexed endpoint column on the
    /// relation table.
    One,
    /// Any number of players, stored in a per-role spill table.
    Many,
}

/// One named role of one relationship type. Roles are local to their relation
/// type: there are no global role interfaces.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleDef {
    pub role: RoleId,
    pub name: String,
    /// What a player of this role may be. Empty means unconstrained.
    pub target_types: Vec<RoleTarget>,
    pub optional: bool,
    pub cardinality: RoleCardinality,
}

impl RoleDef {
    /// True when this role constrains nothing. Schemaless roles are always
    /// unconstrained; a semantic role usually is not.
    pub fn accepts_any_target(&self) -> bool {
        self.target_types.is_empty()
    }
}

/// One role filled by one player in a mutation.
///
/// Repeated players are legal: the same `value` may appear under two different
/// `role`s of one relation, and nothing downstream assumes players are
/// distinct.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoleBinding {
    pub role: RoleId,
    pub value: BindingId,
}
```

- [ ] **Step 8: Re-export and run to verify it passes**

In `graph/ir/src/lib.rs`, add `RoleId` to the `identity` re-export list and add
a new re-export line:

```rust
pub use identity::{
    BindingId, GraphId, LabelId, NodeId, PropertyId, RelationshipId, RelationshipTypeId, RoleId,
    SourceTableId,
};
pub use role::{RoleBinding, RoleCardinality, RoleDef, RoleTarget};
```

Run: `cargo test -p turso_graph_ir`
Expected: PASS.

- [ ] **Step 9: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_ir
git add graph/ir/src/identity.rs graph/ir/src/role.rs graph/ir/src/lib.rs
git commit -S -m "graph/ir: add RoleId and the role definition model

Roles are the identity a native n-ary relation is built from. RoleTarget
keeps node labels and relationship types in distinct identity spaces so a
role that accepts a node label cannot silently accept the relationship type
with the same numeric value.

Tests: turso_graph_ir unit tests."
```

---

### Task 2: Role-shaped relationship source registration

**Files:**
- Modify: `graph/frontend/src/catalog.rs:32-41` (`RelationshipSourceRegistration`), `:58-68` (`RegisteredRelationshipSource`), `:219-259` (load), `:265-495` (register), `:512-529` (`create_catalog`), `:570-586` (validation), `:745-796` (indexes)
- Modify: `graph/frontend/src/lib.rs:39`
- Test: `graph/frontend/src/catalog.rs` (existing `mod tests`)

**Interfaces:**
- Consumes: `turso_graph_ir::{RoleId, RoleCardinality}` from Task 1.
- Produces:
  - `RoleSourceRegistration { name: String, column: String, node_source: String, cardinality: RoleCardinality }`
  - `RelationshipSourceRegistration { name: String, table: String, identity_column: String, roles: Vec<RoleSourceRegistration> }` — `start_column`, `end_column`, `start_node_source`, `end_node_source` are gone.
  - `RelationshipSourceRegistration::binary(name, table, identity_column, start_column, end_column, start_node_source, end_node_source) -> Self` — builds the two required single-valued roles named `start` and `end`.
  - `RegisteredRelationshipRole { role: ir::RoleId, name: String, column: String, node_source: ir::SourceTableId, cardinality: RoleCardinality }`
  - `RegisteredRelationshipSource { id, name, table, identity_column, roles: Vec<RegisteredRelationshipRole> }`
  - `RegisteredRelationshipSource::role_by_name(&self, name: &str) -> Option<&RegisteredRelationshipRole>` (case-insensitive)
  - `RELATIONSHIP_ROLES_TABLE: &str = "__turso_internal_graph_relationship_roles"`

- [ ] **Step 1: Write the failing test**

In `graph/frontend/src/catalog.rs`, inside `mod tests`, add:

```rust
    #[test]
    fn a_two_role_registration_lands_on_todays_physical_shape() {
        // Binary is a layout of the role model, not a separate kind. The
        // registration that used to name start_column/end_column must produce
        // the same two indexed columns plus the composite pair index, or every
        // donor corpus source silently changes its access path.
        let connection = connection();
        create_sources(&connection);
        let graph = register_graph(&connection, &registration("social")).expect("register graph");

        let source = &graph.relationship_sources[0];
        assert_eq!(source.roles.len(), 2);
        assert_eq!(source.roles[0].name, "start");
        assert_eq!(source.roles[0].column, "src");
        assert_eq!(source.roles[0].cardinality, RoleCardinality::One);
        assert_eq!(source.roles[1].name, "end");
        assert_eq!(source.roles[1].column, "dst");
        assert!(source.role_by_name("START").is_some(), "role lookup is case-insensitive");

        let indexes = query_rows(
            &connection,
            "SELECT name FROM sqlite_schema WHERE type = 'index' \
             AND name LIKE '__turso_internal_graph_ep_%'",
        )
        .expect("query endpoint indexes");
        // One per role plus the composite pair index: exactly today's three.
        assert_eq!(indexes.len(), 3);
    }

    #[test]
    fn a_three_role_registration_indexes_every_role_and_every_ordered_pair() {
        let connection = connection();
        connection
            .execute(
                "CREATE TABLE people(id INTEGER PRIMARY KEY); \
                 CREATE TABLE texts(id INTEGER PRIMARY KEY); \
                 CREATE TABLE folios(id INTEGER PRIMARY KEY); \
                 CREATE TABLE transcriptions(\
                     id INTEGER PRIMARY KEY, scribe INTEGER, txt INTEGER, folio INTEGER);",
            )
            .expect("create ternary sources");
        let graph = register_graph(
            &connection,
            &GraphRegistration {
                name: "scriptorium".to_owned(),
                node_sources: vec![
                    NodeSourceRegistration {
                        name: "Person".to_owned(),
                        table: "people".to_owned(),
                        identity_column: "id".to_owned(),
                    },
                    NodeSourceRegistration {
                        name: "Text".to_owned(),
                        table: "texts".to_owned(),
                        identity_column: "id".to_owned(),
                    },
                    NodeSourceRegistration {
                        name: "Folio".to_owned(),
                        table: "folios".to_owned(),
                        identity_column: "id".to_owned(),
                    },
                ],
                relationship_sources: vec![RelationshipSourceRegistration {
                    name: "Transcription".to_owned(),
                    table: "transcriptions".to_owned(),
                    identity_column: "id".to_owned(),
                    roles: vec![
                        RoleSourceRegistration {
                            name: "scribe".to_owned(),
                            column: "scribe".to_owned(),
                            node_source: "Person".to_owned(),
                            cardinality: RoleCardinality::One,
                        },
                        RoleSourceRegistration {
                            name: "text".to_owned(),
                            column: "txt".to_owned(),
                            node_source: "Text".to_owned(),
                            cardinality: RoleCardinality::One,
                        },
                        RoleSourceRegistration {
                            name: "folio".to_owned(),
                            column: "folio".to_owned(),
                            node_source: "Folio".to_owned(),
                            cardinality: RoleCardinality::One,
                        },
                    ],
                }],
            },
        )
        .expect("register ternary graph");

        assert_eq!(graph.relationship_sources[0].roles.len(), 3);
        let indexes = query_rows(
            &connection,
            "SELECT name FROM sqlite_schema WHERE type = 'index' \
             AND name LIKE '__turso_internal_graph_ep_%'",
        )
        .expect("query endpoint indexes");
        // Three single-role indexes plus one composite per unordered role pair
        // (scribe,text), (scribe,folio), (text,folio).
        assert_eq!(indexes.len(), 6);
    }

    #[test]
    fn a_role_must_name_a_registered_node_source() {
        let connection = connection();
        create_sources(&connection);
        let mut graph = registration("bad_endpoint");
        graph.relationship_sources[0].roles[1].node_source = "Missing".to_owned();
        assert!(matches!(
            register_graph(&connection, &graph),
            Err(CatalogError::UnknownEndpoint { relationship, node_source })
                if relationship == "KNOWS" && node_source == "Missing"
        ));
    }
```

Change the shared `registration` helper in the same test module to the
constructor form:

```rust
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "KNOWS", "friendships", "id", "src", "dst", "Person", "Person",
            )],
```

and delete the now-unused `relationship_endpoints_must_name_registered_node_sources`
test (replaced by `a_role_must_name_a_registered_node_source`) and the endpoint
assertions inside `registration_installs_stable_sources_indexes_and_generation_triggers`
that duplicate `a_two_role_registration_lands_on_todays_physical_shape`.

Add `use turso_graph_ir::RoleCardinality;` to the test module imports.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --lib catalog::`
Expected: FAIL to compile with `no function or associated item named binary found` and `struct RelationshipSourceRegistration has no field named roles`.

- [ ] **Step 3: Replace the registration and registered types**

In `graph/frontend/src/catalog.rs`, replace lines 32-41 and 58-68 with:

```rust
/// One named role of a relationship source and the physical column that
/// stores its player.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleSourceRegistration {
    pub name: String,
    /// Endpoint column on the relationship table. Ignored for `Many` roles,
    /// which store players in `<table>__<role>` instead; pass an empty string.
    pub column: String,
    /// Name of a registered node source.
    pub node_source: String,
    pub cardinality: RoleCardinality,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipSourceRegistration {
    pub name: String,
    pub table: String,
    pub identity_column: String,
    /// Declaration order is stable and becomes role ordinal order.
    pub roles: Vec<RoleSourceRegistration>,
}

impl RelationshipSourceRegistration {
    /// A two-endpoint table registered as a two-role relation named
    /// `start`/`end`. This is a layout of the role model, not a separate kind:
    /// every donor corpus source registers this way and keeps working.
    #[allow(clippy::too_many_arguments)]
    pub fn binary(
        name: impl Into<String>,
        table: impl Into<String>,
        identity_column: impl Into<String>,
        start_column: impl Into<String>,
        end_column: impl Into<String>,
        start_node_source: impl Into<String>,
        end_node_source: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            table: table.into(),
            identity_column: identity_column.into(),
            roles: vec![
                RoleSourceRegistration {
                    name: "start".to_owned(),
                    column: start_column.into(),
                    node_source: start_node_source.into(),
                    cardinality: RoleCardinality::One,
                },
                RoleSourceRegistration {
                    name: "end".to_owned(),
                    column: end_column.into(),
                    node_source: end_node_source.into(),
                    cardinality: RoleCardinality::One,
                },
            ],
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisteredRelationshipRole {
    pub role: ir::RoleId,
    pub name: String,
    pub column: String,
    pub node_source: SourceTableId,
    pub cardinality: RoleCardinality,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisteredRelationshipSource {
    pub id: SourceTableId,
    pub name: String,
    pub table: String,
    pub identity_column: String,
    pub roles: Vec<RegisteredRelationshipRole>,
}

impl RegisteredRelationshipSource {
    pub fn role_by_name(&self, name: &str) -> Option<&RegisteredRelationshipRole> {
        self.roles
            .iter()
            .find(|role| role.name.eq_ignore_ascii_case(name))
    }

    pub fn role_by_id(&self, role: ir::RoleId) -> Option<&RegisteredRelationshipRole> {
        self.roles.iter().find(|entry| entry.role == role)
    }

    /// Roles stored in an endpoint column on the relation table.
    pub fn single_valued_roles(&self) -> impl Iterator<Item = &RegisteredRelationshipRole> {
        self.roles
            .iter()
            .filter(|role| role.cardinality == RoleCardinality::One)
    }

    /// Spill table holding the players of a `Many` role.
    pub fn spill_table(&self, role: &RegisteredRelationshipRole) -> String {
        format!("{}__{}", self.table, role.name)
    }
}
```

Change the imports at the top of the file to
`use turso_graph_ir::{self as ir, GraphId, RoleCardinality, SourceTableId};`.

- [ ] **Step 4: Add the role catalog table**

In `create_catalog` (`catalog.rs:512`), drop the endpoint columns from
`RELATIONSHIP_SOURCES_TABLE` and add the roles table:

```rust
    execute_internal(connection, format!(
        "CREATE TABLE IF NOT EXISTS {RELATIONSHIP_SOURCES_TABLE}(source_id INTEGER PRIMARY KEY, table_name TEXT NOT NULL, identity_column TEXT NOT NULL)"
    ))?;
    execute_internal(connection, format!(
        "CREATE TABLE IF NOT EXISTS {RELATIONSHIP_ROLES_TABLE}(source_id INTEGER NOT NULL, ordinal INTEGER NOT NULL, name TEXT NOT NULL COLLATE NOCASE, column_name TEXT NOT NULL, node_source_id INTEGER NOT NULL, cardinality TEXT NOT NULL CHECK(cardinality IN ('one', 'many')), PRIMARY KEY(source_id, ordinal))"
    ))?;
```

and declare the constant beside the others (`catalog.rs:21`):

```rust
pub(crate) const RELATIONSHIP_ROLES_TABLE: &str = "__turso_internal_graph_relationship_roles";
```

- [ ] **Step 5: Write the role rows and indexes during registration**

Replace the relationship-source loop body (`catalog.rs:358-417`) with:

```rust
    for relationship in &registration.relationship_sources {
        execute_internal(
            connection,
            format!(
                "INSERT INTO {SOURCES_TABLE}(graph_id, name, kind) VALUES ({}, {}, 'relationship')",
                graph_id.get(),
                sql_string(&relationship.name)
            ),
        )?;
        let relationship_id = scalar_integer(
            connection,
            &format!(
                "SELECT id FROM {SOURCES_TABLE} WHERE graph_id = {} AND name = {}",
                graph_id.get(),
                sql_string(&relationship.name)
            ),
            "relationship source id",
        )?;
        execute_internal(connection, format!(
            "INSERT INTO {RELATIONSHIP_SOURCES_TABLE}(source_id, table_name, identity_column) VALUES ({}, {}, {})",
            relationship_id,
            sql_string(&relationship.table),
            sql_string(&relationship.identity_column)
        ))?;
        for (ordinal, role) in relationship.roles.iter().enumerate() {
            let node_source = node_ids.get(&role.node_source).ok_or_else(|| {
                CatalogError::UnknownEndpoint {
                    relationship: relationship.name.clone(),
                    node_source: role.node_source.clone(),
                }
            })?;
            execute_internal(connection, format!(
                "INSERT INTO {RELATIONSHIP_ROLES_TABLE}(source_id, ordinal, name, column_name, node_source_id, cardinality) \
                 VALUES ({}, {}, {}, {}, {}, {})",
                relationship_id,
                ordinal + 1,
                sql_string(&role.name),
                sql_string(&role.column),
                node_source.get(),
                sql_string(match role.cardinality {
                    RoleCardinality::One => "one",
                    RoleCardinality::Many => "many",
                })
            ))?;
            match role.cardinality {
                RoleCardinality::One => {
                    install_role_index(connection, graph_id, relationship, role)?;
                }
                RoleCardinality::Many => {
                    install_spill_table(connection, graph_id, relationship, role)?;
                }
            }
        }
        // Co-membership patterns bind two role players before matching the
        // second relationship; the composite index turns that probe from an
        // in-degree scan into an exact lookup. A two-role relation gets
        // exactly one such index, which is today's (start, end) index.
        install_role_pair_indexes(connection, graph_id, relationship)?;
    }
```

- [ ] **Step 6: Replace the index installers**

Replace `install_endpoint_index` and `install_endpoint_pair_index`
(`catalog.rs:745-796`) with:

```rust
fn install_role_index(
    connection: &Arc<Connection>,
    graph: GraphId,
    source: &RelationshipSourceRegistration,
    role: &RoleSourceRegistration,
) -> Result<(), CatalogError> {
    let name = format!(
        "{TURSO_GRAPH_CATALOG_PREFIX}ep_{}_{}_{:016x}",
        graph.get(),
        role.name.to_ascii_lowercase(),
        stable_hash(&format!("{}:{}", source.table, role.column))
    );
    execute_internal(
        connection,
        format!(
            "CREATE INDEX IF NOT EXISTS {} ON {}({})",
            quote_identifier(&name),
            quote_identifier(&source.table),
            quote_identifier(&role.column)
        ),
    )?;
    Ok(())
}

/// One composite index per unordered pair of single-valued roles. A two-role
/// relation therefore gets exactly the (start, end) index it has today.
fn install_role_pair_indexes(
    connection: &Arc<Connection>,
    graph: GraphId,
    source: &RelationshipSourceRegistration,
) -> Result<(), CatalogError> {
    let single = source
        .roles
        .iter()
        .filter(|role| role.cardinality == RoleCardinality::One)
        .collect::<Vec<_>>();
    for (index, left) in single.iter().enumerate() {
        for right in single.iter().skip(index + 1) {
            let name = format!(
                "{TURSO_GRAPH_CATALOG_PREFIX}ep_{}_pair_{:016x}",
                graph.get(),
                stable_hash(&format!("{}:{}:{}", source.table, left.column, right.column))
            );
            execute_internal(
                connection,
                format!(
                    "CREATE INDEX IF NOT EXISTS {} ON {}({}, {})",
                    quote_identifier(&name),
                    quote_identifier(&source.table),
                    quote_identifier(&left.column),
                    quote_identifier(&right.column)
                ),
            )?;
        }
    }
    Ok(())
}

/// A `Many` role stores its players in `<table>__<role>(relation_id, node_id)`,
/// indexed in both directions so a hop is an index probe from either side.
fn install_spill_table(
    connection: &Arc<Connection>,
    graph: GraphId,
    source: &RelationshipSourceRegistration,
    role: &RoleSourceRegistration,
) -> Result<(), CatalogError> {
    let table = format!("{}__{}", source.table, role.name);
    execute_internal(
        connection,
        format!(
            "CREATE TABLE IF NOT EXISTS {}(relation_id INTEGER NOT NULL, node_id INTEGER NOT NULL)",
            quote_identifier(&table)
        ),
    )?;
    for (suffix, columns) in [("fwd", "relation_id, node_id"), ("rev", "node_id, relation_id")] {
        let name = format!(
            "{TURSO_GRAPH_CATALOG_PREFIX}spill_{}_{suffix}_{:016x}",
            graph.get(),
            stable_hash(&table)
        );
        execute_internal(
            connection,
            format!(
                "CREATE INDEX IF NOT EXISTS {} ON {}({columns})",
                quote_identifier(&name),
                quote_identifier(&table)
            ),
        )?;
    }
    Ok(())
}
```

- [ ] **Step 7: Load roles back**

Replace the relationship-loading block (`catalog.rs:219-251`) with:

```rust
    let relationship_rows = query_rows(
        connection,
        &format!(
            "SELECT s.id, s.name, r.table_name, r.identity_column FROM {SOURCES_TABLE} s \
             JOIN {RELATIONSHIP_SOURCES_TABLE} r ON r.source_id = s.id \
             WHERE s.graph_id = {} ORDER BY s.id",
            graph_id.get()
        ),
    )?;
    let mut relationship_sources = Vec::with_capacity(relationship_rows.len());
    for row in relationship_rows {
        let id = source_id(integer(&row, 0, "relationship source id")?)?;
        let table = text(&row, 2, "relationship source table")?.to_owned();
        let identity_column = text(&row, 3, "relationship identity column")?.to_owned();
        let role_rows = query_rows(
            connection,
            &format!(
                "SELECT ordinal, name, column_name, node_source_id, cardinality \
                 FROM {RELATIONSHIP_ROLES_TABLE} WHERE source_id = {} ORDER BY ordinal",
                id.get()
            ),
        )?;
        let mut roles = Vec::with_capacity(role_rows.len());
        let mut required_columns = vec![identity_column.clone()];
        for role_row in role_rows {
            let ordinal = integer(&role_row, 0, "role ordinal")?;
            let role = u32::try_from(ordinal)
                .ok()
                .and_then(|value| ir::RoleId::new(value).ok())
                .ok_or(CatalogError::InvalidIdentity {
                    kind: "role",
                    value: ordinal,
                })?;
            let cardinality = match text(&role_row, 4, "role cardinality")? {
                "one" => RoleCardinality::One,
                "many" => RoleCardinality::Many,
                _ => return Err(CatalogError::InvalidCatalogValue("role cardinality")),
            };
            let column = text(&role_row, 2, "role column")?.to_owned();
            if cardinality == RoleCardinality::One {
                required_columns.push(column.clone());
            }
            roles.push(RegisteredRelationshipRole {
                role,
                name: text(&role_row, 1, "role name")?.to_owned(),
                column,
                node_source: source_id(integer(&role_row, 3, "role node source id")?)?,
                cardinality,
            });
        }
        let borrowed = required_columns.iter().map(String::as_str).collect::<Vec<_>>();
        require_columns(connection, &table, &borrowed)?;
        relationship_sources.push(RegisteredRelationshipSource {
            id,
            name: text(&row, 1, "relationship source name")?.to_owned(),
            table,
            identity_column,
            roles,
        });
    }
```

Note the role identity is the ordinal: role identities are per relation type,
so the ordinal is already a stable non-zero `u32` within its source.

- [ ] **Step 8: Update validation**

In `validate_registration_names` (`catalog.rs:570-586`), replace the
relationship arm with:

```rust
    for source in &registration.relationship_sources {
        validate_name("relationship source", &source.name)?;
        let mut columns = vec![source.identity_column.as_str()];
        let mut role_names = HashSet::new();
        for role in &source.roles {
            validate_name("role", &role.name)?;
            if !role_names.insert(role.name.to_ascii_lowercase()) {
                return Err(CatalogError::DuplicateName {
                    kind: "role",
                    name: role.name.clone(),
                });
            }
            if role.cardinality == RoleCardinality::One {
                columns.push(role.column.as_str());
            }
        }
        validate_source_identifiers(&source.table, &columns)?;
        if !source_names.insert(source.name.to_ascii_lowercase()) {
            return Err(CatalogError::DuplicateName {
                kind: "relationship source",
                name: source.name.clone(),
            });
        }
    }
```

and in `register_graph_in_transaction` (`catalog.rs:286-303`) replace the
`require_columns` call for relationships with the same
identity-plus-single-valued-role column list.

- [ ] **Step 9: Update every construction site**

Replace the struct literal with `RelationshipSourceRegistration::binary(...)` in:
`graph/testkit/src/runner.rs:211`, `tests/integration/multi_frontend_doc.rs:230`,
`graph/frontend/src/snapshot.rs:915`, `graph/frontend/src/schema_catalog.rs:1016`,
`graph/frontend/src/graph_expand.rs:633`, `graph/frontend/src/dialect.rs:414`,
`graph/frontend/src/session.rs:601` and `:1199`,
`graph/frontend/benches/semantic_prepare.rs:52`, `:128`, `:137`,
`graph/frontend/examples/snapshot_profile.rs:52`,
`graph/frontend/tests/native_capabilities.rs:893`, and the multi-source literals
in `graph/frontend/src/catalog.rs:1116` and `:1125`.

Update `graph/frontend/src/lib.rs:39` to export the new names:

```rust
    RegisteredRelationshipRole, RegisteredRelationshipSource, RelationshipSourceRegistration,
    RoleSourceRegistration, GRAPH_CATALOG_VERSION,
```

- [ ] **Step 10: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend --lib catalog::`
Expected: PASS, including both new role tests.

- [ ] **Step 11: Full gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend
mise run corpus
mise run cypherbench-sample
git add -A
git commit -S -m "graph/catalog: register relationship sources as roles

A relationship source now declares an ordered list of named roles, each with
an endpoint column, a target node source, and a cardinality. Binary is not a
separate kind: RelationshipSourceRegistration::binary builds the two required
single-valued roles named start/end, so a two-endpoint table lands on exactly
the two indexed columns plus composite pair index it had before.

Many-valued roles spill to <table>__<role>(relation_id, node_id), indexed in
both directions.

Tests: catalog unit tests assert the two-role physical shape is unchanged and
that a three-role source indexes every role and every ordered pair; corpus at
8,926 with no new failure family."
```

---

### Task 3: Fail loudly on a pre-role catalog

**Files:**
- Modify: `graph/frontend/src/catalog.rs:79-119` (`CatalogError`), `:178-197` (`load_registered_graph`)
- Test: `graph/frontend/src/catalog.rs` (existing `mod tests`)

**Interfaces:**
- Consumes: `RELATIONSHIP_ROLES_TABLE` from Task 2.
- Produces: `CatalogError::IncompatibleGraphLayout { detail: String }`.

- [ ] **Step 1: Write the failing test**

```rust
    #[test]
    fn a_catalog_predating_roles_fails_at_open_and_names_the_fresh_start_policy() {
        // Fresh start: there is no legacy reader and no migration. Opening a
        // pre-role catalog must say so rather than reporting a confusing
        // "invalid catalog value" from a missing column.
        let connection = connection();
        create_sources(&connection);
        register_graph(&connection, &registration("social")).expect("register graph");
        // Simulate the pre-role layout: the roles table did not exist.
        execute_internal(
            &connection,
            format!("DROP TABLE {RELATIONSHIP_ROLES_TABLE}"),
        )
        .expect("drop roles table");

        let error = load_registered_graph(&connection, "social").expect_err("pre-role catalog");
        let message = error.to_string();
        assert!(
            matches!(error, CatalogError::IncompatibleGraphLayout { .. }),
            "expected IncompatibleGraphLayout, got {message}"
        );
        assert!(
            message.contains("no migration"),
            "the error must name the fresh-start policy, got {message}"
        );
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --lib catalog::a_catalog_predating_roles`
Expected: FAIL to compile with `no variant named IncompatibleGraphLayout`.

- [ ] **Step 3: Add the variant**

In `CatalogError`:

```rust
    #[error("graph catalog predates native relationship roles ({detail}); this build reads only role-shaped catalogs and there is no migration, so the graph must be created fresh")]
    IncompatibleGraphLayout { detail: String },
```

- [ ] **Step 4: Detect the old layout at open**

In `load_registered_graph`, immediately after `ensure_catalog_exists`:

```rust
    if query_rows(
        connection,
        &format!(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name = {}",
            sql_string(RELATIONSHIP_ROLES_TABLE)
        ),
    )?
    .is_empty()
    {
        return Err(CatalogError::IncompatibleGraphLayout {
            detail: format!("{RELATIONSHIP_ROLES_TABLE} is absent"),
        });
    }
```

- [ ] **Step 5: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend --lib catalog::`
Expected: PASS.

- [ ] **Step 6: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend
git add graph/frontend/src/catalog.rs
git commit -S -m "graph/catalog: reject pre-role catalogs at open

There is no legacy reader and no migration path. A catalog without the
relationship-roles table must fail with a message that names the
fresh-start policy, not with an incidental invalid-value error from a
missing column.

Tests: catalog unit test drops the roles table and asserts the error text."
```

---

### Task 4: Role-shaped relationship table layout

**Files:**
- Modify: `graph/frontend/src/lowering.rs:14-40` (`RelationshipTableLayout`, `RelationalCatalogSnapshot`)
- Modify: `graph/frontend/src/schema_catalog.rs:762-770` (`relationship_layout`), `:828-842` (`payload_columns`)
- Test: `graph/frontend/src/schema_catalog.rs` (existing `mod tests`)

**Interfaces:**
- Consumes: `RegisteredRelationshipRole` from Task 2.
- Produces:
  - `RelationshipRoleLayout { role: ir::RoleId, name: String, column: String, cardinality: RoleCardinality, spill_table: Option<String> }`
  - `RelationshipTableLayout { table: String, identity_column: String, roles: Vec<RelationshipRoleLayout> }`
  - `RelationshipTableLayout::role(&self, role: ir::RoleId) -> Option<&RelationshipRoleLayout>`

- [ ] **Step 1: Write the failing test**

In `graph/frontend/src/schema_catalog.rs`, inside `mod tests`:

```rust
    #[test]
    fn a_relationship_layout_exposes_roles_and_excludes_them_from_payload() {
        // Payload columns are everything that is not structural. A role column
        // that leaked into the payload would be readable as a property and
        // writable by SET, which would corrupt the relation's participation.
        let (catalog, source) = binary_relationship_catalog();
        let layout = catalog
            .relationship_layout(source)
            .expect("relationship layout");
        assert_eq!(layout.roles.len(), 2);
        assert_eq!(layout.roles[0].name, "start");
        assert_eq!(layout.roles[0].column, "src");
        assert!(layout.roles[0].spill_table.is_none());
        assert_eq!(
            layout.role(layout.roles[1].role).map(|role| role.column.as_str()),
            Some("dst")
        );

        let payload = catalog.payload_columns(source).expect("payload columns");
        assert!(
            payload.iter().all(|(logical, _)| logical != "src" && logical != "dst"),
            "role columns must not appear as payload properties: {payload:?}"
        );
    }
```

Add a `binary_relationship_catalog()` helper next to the existing test helpers
in that module, returning the `SchemaCatalog` and the relationship
`SourceTableId` built from `RelationshipSourceRegistration::binary("KNOWS",
"friendships", "id", "src", "dst", "Person", "Person")`, reusing the existing
registration helper at `schema_catalog.rs:1016`.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --lib schema_catalog::`
Expected: FAIL to compile with `struct RelationshipTableLayout has no field named roles`.

- [ ] **Step 3: Replace the layout type**

In `graph/frontend/src/lowering.rs`, replace `RelationshipTableLayout`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipRoleLayout {
    pub role: ir::RoleId,
    pub name: String,
    /// Endpoint column on the relation table. Empty for `Many` roles.
    pub column: String,
    pub cardinality: ir::RoleCardinality,
    /// Set for `Many` roles: `<table>__<role>(relation_id, node_id)`.
    pub spill_table: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipTableLayout {
    pub table: String,
    pub identity_column: String,
    /// Declaration order. A two-role relation is `[start, end]`.
    pub roles: Vec<RelationshipRoleLayout>,
}

impl RelationshipTableLayout {
    pub fn role(&self, role: ir::RoleId) -> Option<&RelationshipRoleLayout> {
        self.roles.iter().find(|entry| entry.role == role)
    }

    /// Columns that carry participation rather than payload.
    pub fn structural_columns(&self) -> Vec<String> {
        let mut columns = vec![self.identity_column.clone()];
        columns.extend(
            self.roles
                .iter()
                .filter(|role| role.cardinality == ir::RoleCardinality::One)
                .map(|role| role.column.clone()),
        );
        columns
    }
}
```

- [ ] **Step 4: Build the layout from the registered roles**

In `graph/frontend/src/schema_catalog.rs`, replace `relationship_layout`:

```rust
    fn relationship_layout(&self, source: ir::SourceTableId) -> Option<RelationshipTableLayout> {
        let entry = self.relationship_source_entry(source)?;
        Some(RelationshipTableLayout {
            table: entry.table.clone(),
            identity_column: entry.identity_column.clone(),
            roles: entry
                .roles
                .iter()
                .map(|role| RelationshipRoleLayout {
                    role: role.role,
                    name: role.name.clone(),
                    column: role.column.clone(),
                    cardinality: role.cardinality,
                    spill_table: match role.cardinality {
                        ir::RoleCardinality::One => None,
                        ir::RoleCardinality::Many => Some(entry.spill_table(role)),
                    },
                })
                .collect(),
        })
    }
```

and in `payload_columns` replace the relationship arm's `structural` vector with
the layout's:

```rust
        } else if let Some(entry) = self.relationship_source_entry(source) {
            let mut structural = vec![entry.identity_column.clone()];
            structural.extend(
                entry
                    .single_valued_roles()
                    .map(|role| role.column.clone()),
            );
            (entry.table.clone(), structural)
        } else {
```

Import `RelationshipRoleLayout` alongside `RelationshipTableLayout` at
`schema_catalog.rs:13`.

- [ ] **Step 5: Update the remaining layout construction sites**

`RelationshipTableLayout` literals appear in test and fixture code at
`graph/frontend/src/graph_expand.rs:777`, `graph/frontend/src/session.rs`
(`use` at `:483`), `graph/frontend/tests/fixture.rs`,
`graph/frontend/tests/dialect_alignment.rs`, and
`graph/frontend/tests/fixed_pattern_fixtures.rs`. Replace each
`start_column`/`end_column` pair with a two-element `roles` vector using
`ir::RoleId::new(1)`/`ir::RoleId::new(2)` and `RoleCardinality::One`.

- [ ] **Step 6: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend`
Expected: PASS.

- [ ] **Step 7: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend
mise run corpus
git add -A
git commit -S -m "graph/frontend: make the relationship layout role-shaped

RelationshipTableLayout carries the relation's roles instead of a start and
end column, and structural_columns derives the payload exclusion set from
them, so a role column can never be read as a property or written by SET.

Tests: schema_catalog unit test asserts role exposure and payload exclusion;
corpus at 8,926."
```

---

### Task 5: Add the role pair to the expand IR alongside `direction`

This is the expand half of the expand/contract pair. The IR grows role fields;
`direction` stays and stays authoritative. Nothing changes behaviourally.

**Files:**
- Modify: `graph/ir/src/plan.rs:44-70` (`FixedExpand`), `:72-110` (`GraphExpand`)
- Modify: `graph/frontend/src/binder.rs:2700-2825`
- Test: `graph/ir/src/plan.rs` (existing `mod tests`), `graph/frontend/tests/fixed_pattern_fixtures.rs`

**Interfaces:**
- Consumes: `turso_graph_ir::RoleId` from Task 1.
- Produces: on both `FixedExpand` and `GraphExpand`:
  - `pub from_role: RoleId`
  - `pub to_role: RoleId`
  - `pub symmetric: bool`
  - `FixedExpand::role_pair(&self) -> (RoleId, RoleId)`

- [ ] **Step 1: Write the failing test**

In `graph/frontend/tests/fixed_pattern_fixtures.rs`:

```rust
#[test]
fn an_outgoing_expand_binds_the_start_to_end_role_pair() {
    // The role pair must agree with the direction it is replacing, or the
    // contract half of this migration silently reverses every traversal.
    let plan = bind_fixture("MATCH (a:Person)-[r:KNOWS]->(b:Person) RETURN b");
    let expand = first_fixed_expand(&plan);
    assert_eq!(expand.from_role.get(), 1, "role 1 is `start`");
    assert_eq!(expand.to_role.get(), 2, "role 2 is `end`");
    assert!(!expand.symmetric);
}

#[test]
fn an_incoming_expand_reverses_the_role_pair_rather_than_flagging_it() {
    let plan = bind_fixture("MATCH (a:Person)<-[r:KNOWS]-(b:Person) RETURN b");
    let expand = first_fixed_expand(&plan);
    assert_eq!(expand.role_pair(), (role(2), role(1)));
    assert!(!expand.symmetric);
}

#[test]
fn an_undirected_same_source_expand_is_the_symmetric_pair() {
    // Today's Direction::Both. The binder only emits it when both endpoints
    // come from one node source; otherwise it unions two directed branches,
    // and this test would find two expands rather than a symmetric one.
    let plan = bind_fixture("MATCH (a:Person)-[r:KNOWS]-(b:Person) RETURN b");
    let expand = first_fixed_expand(&plan);
    assert_eq!(expand.role_pair(), (role(1), role(2)));
    assert!(expand.symmetric, "an undirected pattern matches the pair in both orders");
}
```

Add the helpers `fn role(value: u32) -> RoleId { RoleId::new(value).unwrap() }` and
`fn first_fixed_expand(plan: &ir::Plan) -> &ir::FixedExpand` (a depth-first walk
of `PlanKind` returning the first `FixedExpand`) next to the existing fixture
helpers in that file.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --test fixed_pattern_fixtures role_pair`
Expected: FAIL to compile with `no field from_role on type &FixedExpand`.

- [ ] **Step 3: Add the fields**

In `graph/ir/src/plan.rs`, add to both `FixedExpand` and `GraphExpand`, directly
after `direction`:

```rust
    /// Role the traversal leaves the source binding through.
    pub from_role: RoleId,
    /// Role the traversal enters the target binding through.
    pub to_role: RoleId,
    /// Also match the reversed pair. This is what an undirected pattern means
    /// when both endpoints share a node source; a plain ordered pair cannot
    /// say it.
    pub symmetric: bool,
```

and on `FixedExpand`:

```rust
impl FixedExpand {
    pub fn role_pair(&self) -> (RoleId, RoleId) {
        (self.from_role, self.to_role)
    }
}
```

Import `RoleId` at the top of `plan.rs`.

- [ ] **Step 4: Populate them in the binder**

In `graph/frontend/src/binder.rs`, at each `ir::FixedExpand`/`ir::GraphExpand`
construction site, derive the pair from the direction that is already computed:

```rust
        let (from_role, to_role, symmetric) = match direction {
            ir::Direction::Outgoing => (start_role, end_role, false),
            ir::Direction::Incoming => (end_role, start_role, false),
            ir::Direction::Both => (start_role, end_role, true),
        };
```

where `start_role` and `end_role` come from the relationship source's role list:

```rust
        let roles = self
            .catalog
            .relationship_source_roles(relationship_source)
            .ok_or(BindError::UnknownRelationshipSource { .. })?;
        let start_role = roles[0].role;
        let end_role = roles[1].role;
```

Name it `relationship_source_roles`, keyed by `SourceTableId`. Task 8 adds a
separate `relationship_roles` keyed by `RelationshipTypeId` returning
`SemanticRole`; the two must not share a name.

Add `fn relationship_source_roles(&self, source: ir::SourceTableId) -> Option<Vec<RegisteredRelationshipRole>>`
to the `GraphCatalogSnapshot` trait (`binder.rs:55-138`) and implement it for
`SchemaCatalog` by delegating to `relationship_layout`, and for the test
snapshots in `graph/frontend/tests/fixture.rs` and
`graph/frontend/src/session.rs` by returning the two synthesized roles.

- [ ] **Step 5: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend --test fixed_pattern_fixtures`
Expected: PASS.

- [ ] **Step 6: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_ir -p turso_graph_frontend
mise run corpus
git add -A
git commit -S -m "graph/ir: carry the role pair on expands beside direction

Expand half of the direction migration. The binder derives (from_role,
to_role, symmetric) from the direction it already computes, so the two
representations agree everywhere before any consumer switches over.
Direction stays authoritative until lowering moves.

Tests: fixed-pattern fixtures assert outgoing, incoming, and undirected
patterns produce the pair that matches their direction; corpus at 8,926."
```

---

### Task 6: Lower expands through roles

The contract half's first move: lowering stops reading `direction` and reads the
role pair instead. The emitted SQL must not change for a two-role relation.

**Files:**
- Modify: `graph/frontend/src/lowering.rs:1398-1560` (`lower_fixed_expand`)
- Test: `graph/frontend/tests/dialect_alignment.rs`

**Interfaces:**
- Consumes: `RelationshipTableLayout::role` (Task 4), `FixedExpand::role_pair` (Task 5).
- Produces: no new public API. `lower_fixed_expand` is renamed `lower_role_expand` in Task 7, not here.

- [ ] **Step 1: Write the failing test**

In `graph/frontend/tests/dialect_alignment.rs`:

```rust
#[test]
fn role_lowering_emits_byte_identical_sql_for_a_two_role_relation() {
    // Binary is a layout of the role model. If role lowering produces even a
    // different alias or predicate order, every donor query's plan shifts and
    // the corpus number stops meaning what it meant.
    for query in [
        "MATCH (a:Person)-[r:KNOWS]->(b:Person) RETURN b.name",
        "MATCH (a:Person)<-[r:KNOWS]-(b:Person) RETURN b.name",
        "MATCH (a:Person)-[r:KNOWS]-(b:Person) RETURN b.name",
        "MATCH (a:Person)-[r:KNOWS]->(b:Person) WHERE b.age > 30 RETURN b",
    ] {
        assert_eq!(
            lower_to_sql(query),
            expected_binary_sql(query),
            "role lowering changed the SQL for {query}"
        );
    }
}
```

`expected_binary_sql` is a golden map recorded in the same file. Populate it by
running `lower_to_sql` for each query **before** Step 3 and pasting the output:

```bash
cargo test -p turso_graph_frontend --test dialect_alignment -- --nocapture print_binary_sql_goldens
```

Add that printer as a `#[test] #[ignore]` helper in the same file.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --test dialect_alignment role_lowering_emits`
Expected: PASS trivially before the change (goldens recorded from the current
lowering). This test is a regression fence, not a red-green cycle: its job is to
fail in Step 4 if role lowering drifts.

- [ ] **Step 3: Replace the direction match with a role match**

In `lower_fixed_expand`, replace the six-arm `(bound_reference, direction)` match
(`lowering.rs:1441-1505`) with:

```rust
    let from_column = layout
        .role(expand.from_role)
        .ok_or(LoweringError::UnknownRole {
            relation: layout.table.clone(),
            role: expand.from_role,
        })?
        .column
        .clone();
    let to_column = layout
        .role(expand.to_role)
        .ok_or(LoweringError::UnknownRole {
            relation: layout.table.clone(),
            role: expand.to_role,
        })?
        .column
        .clone();

    let join_predicate = match (bound_reference, expand.symmetric) {
        (Some(target), false) => format!(
            "{relationship_alias}.{from} = {source_alias}.{identity} \
             AND {relationship_alias}.{to} = {target}",
            from = quote_identifier(&from_column),
            to = quote_identifier(&to_column),
            identity = quote_identifier(&node_identity),
        ),
        (None, false) => format!(
            "{relationship_alias}.{from} = {source_alias}.{identity}",
            from = quote_identifier(&from_column),
            identity = quote_identifier(&node_identity),
        ),
        // Symmetric: the same relation row matches with the pair in either
        // order. This is the shape today's Direction::Both lowers to, and it
        // is only reachable when both roles target the same node source.
        (Some(target), true) => format!(
            "(({relationship_alias}.{from} = {source_alias}.{identity} \
               AND {relationship_alias}.{to} = {target}) \
              OR ({relationship_alias}.{to} = {source_alias}.{identity} \
               AND {relationship_alias}.{from} = {target}))",
            from = quote_identifier(&from_column),
            to = quote_identifier(&to_column),
            identity = quote_identifier(&node_identity),
        ),
        (None, true) => format!(
            "({relationship_alias}.{from} = {source_alias}.{identity} \
              OR {relationship_alias}.{to} = {source_alias}.{identity})",
            from = quote_identifier(&from_column),
            to = quote_identifier(&to_column),
            identity = quote_identifier(&node_identity),
        ),
    };
```

and replace the target-column selection that followed the old match with the
symmetric-aware expression:

```rust
    let target_expression = if expand.symmetric {
        format!(
            "CASE WHEN {relationship_alias}.{from} = {source_alias}.{identity} \
             THEN {relationship_alias}.{to} ELSE {relationship_alias}.{from} END",
            from = quote_identifier(&from_column),
            to = quote_identifier(&to_column),
            identity = quote_identifier(&node_identity),
        )
    } else {
        format!(
            "{relationship_alias}.{to}",
            to = quote_identifier(&to_column)
        )
    };
```

Add the error variant to `LoweringError`:

```rust
    #[error("relation {relation} has no role {role:?}")]
    UnknownRole {
        relation: String,
        role: ir::RoleId,
    },
```

Index selection needs no separate change: the frontend lowers to SQL, so naming
the role columns in the join is exactly what makes the storage planner key off
the role pair. The per-role and per-pair indexes installed in Task 2 are what it
selects from, and the `bound_target` cycle fold becomes "composite over
(from_role, to_role)" for free because that is the index the pair installer
created.

- [ ] **Step 4: Run to verify the SQL is unchanged**

Run: `cargo test -p turso_graph_frontend --test dialect_alignment`
Expected: PASS. If the golden differs, the difference is the bug — fix the
lowering to match the golden rather than re-recording it.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend
mise run corpus
mise run cypherbench-sample
git add -A
git commit -S -m "graph/lowering: join expands through role columns

Lowering reads the expand's role pair instead of its direction. The six
direction arms collapse to four cases over (bound target, symmetric),
which is the same shape with the endpoint columns named by role.

A golden test pins the emitted SQL for the two-role case so the migration
cannot shift a donor query's plan.

Tests: dialect_alignment goldens; corpus at 8,926; cypherbench at baseline."
```

---

### Task 7: Delete `Direction` and rename to `RoleExpand`

**Files:**
- Modify: `graph/ir/src/scope.rs:12-30` (delete `Direction`), `graph/ir/src/plan.rs`, `graph/ir/src/lib.rs`
- Modify: `graph/frontend/src/binder.rs`, `graph/frontend/src/lowering.rs`, `graph/frontend/src/graph_expand.rs:120-150`
- Modify: `graph/cypher/src/parser.rs:470-490` (desugar at the AST boundary)
- Test: `graph/ir/src/plan.rs`, `graph/frontend/tests/desugaring_golden.rs` (create)

**Interfaces:**
- Consumes: everything from Tasks 5 and 6.
- Produces:
  - `ir::RoleExpand` replacing `ir::FixedExpand`, with `direction` gone.
  - `ir::PlanKind::RoleExpand` replacing `PlanKind::FixedExpand`.
  - `ir::Direction` no longer exists. `turso_cypher::ast::Direction` survives — it is a parser-level spelling, and the binder is where it dies.

- [ ] **Step 1: Write the failing test**

Create `graph/frontend/tests/desugaring_golden.rs`:

```rust
//! Arrow syntax is sugar over roles. If the two forms ever bind to different
//! IR, then a "binary" query and its role-form equivalent can disagree at
//! runtime, and the claim that binary is a layout of the role model is false.

mod fixture;

use fixture::{bind_fixture, first_role_expand};

#[test]
fn arrow_and_role_forms_of_the_same_pattern_bind_identically() {
    let arrow = bind_fixture("MATCH (a:Person)-[r:KNOWS]->(b:Person) RETURN b");
    let roles = bind_fixture(
        "MATCH (a:Person), (b:Person), [r:KNOWS](start: a, end: b) RETURN b",
    );
    assert_eq!(first_role_expand(&arrow), first_role_expand(&roles));
}

#[test]
fn the_reversed_arrow_is_the_reversed_role_pair() {
    let arrow = bind_fixture("MATCH (a:Person)<-[r:KNOWS]-(b:Person) RETURN b");
    let roles = bind_fixture(
        "MATCH (a:Person), (b:Person), [r:KNOWS](end: a, start: b) RETURN b",
    );
    assert_eq!(first_role_expand(&arrow), first_role_expand(&roles));
}
```

This test depends on the standalone role pattern, which lands in Task 12. Mark
both `#[ignore = "standalone role pattern lands in Task 12"]` now and remove the
attribute in Task 13's final step.

Add the non-ignored rename assertion to `graph/ir/src/plan.rs`'s test module:

```rust
    #[test]
    fn a_role_expand_names_its_roles_and_no_direction() {
        // Direction is a parser spelling, not a plan concept. A plan that still
        // carried it would give two sources of truth for which way a traversal
        // runs.
        let expand = sample_role_expand();
        assert_eq!(expand.role_pair(), (RoleId::new(1).unwrap(), RoleId::new(2).unwrap()));
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_ir --lib plan::`
Expected: FAIL to compile with `cannot find function sample_role_expand`.

- [ ] **Step 3: Delete the direction field and rename the struct**

- In `graph/ir/src/plan.rs`: rename `FixedExpand` → `RoleExpand`, rename the
  `PlanKind::FixedExpand` variant → `PlanKind::RoleExpand`, delete `pub direction: Direction`
  from both `RoleExpand` and `GraphExpand`.
- In `graph/frontend/src/lowering.rs`: rename `lower_fixed_expand` →
  `lower_role_expand` (Tasks 14 and 15 refer to it by the new name).
- In `graph/frontend/tests/fixed_pattern_fixtures.rs`: rename the Task 5 helper
  `first_fixed_expand` → `first_role_expand` and move it into
  `graph/frontend/tests/fixture.rs`, because `desugaring_golden.rs` uses it too.
- In `graph/ir/src/scope.rs`: delete the `Direction` enum (lines 12-30).
- In `graph/ir/src/lib.rs`: drop `Direction` from the `scope` re-export, rename
  `FixedExpand` → `RoleExpand` in the `plan` re-export.
- Add `fn sample_role_expand() -> RoleExpand` to `plan.rs`'s test module,
  constructing every field with `SourceTableId::new(1)`-style literals.

- [ ] **Step 4: Desugar in the parser walker, not the binder**

In `graph/cypher/src/parser.rs`, leave `ast::Direction` as-is: it is the
grammar's spelling of an arrow. In `graph/frontend/src/binder.rs`, replace the
`ir::Direction` construction from Task 5 with a direct match on the AST:

```rust
        let (from_role, to_role, symmetric) = match pattern.direction {
            ast::Direction::Outgoing => (start_role, end_role, false),
            ast::Direction::Incoming => (end_role, start_role, false),
            // Undirected only stays one expand when both roles target the same
            // node source; otherwise the caller has already split it into a
            // union of two directed branches.
            ast::Direction::Both => (start_role, end_role, true),
        };
```

and delete every remaining `use turso_graph_ir::Direction` and
`ir::Direction::` reference across `binder.rs`, `lowering.rs`, `graph_expand.rs`,
`snapshot.rs`, and the test fixtures.

- [ ] **Step 5: Replace the vtab direction column**

In `graph/frontend/src/graph_expand.rs`, replace `fn direction(value: &Value)`
with role-name columns:

```rust
/// The vtab receives role names rather than a direction word, because a
/// relation with more than two roles has no direction to name.
fn role_name(value: &Value) -> Result<String, ExpandError> {
    match value {
        Value::Text(text) => Ok(text.as_str().to_ascii_lowercase()),
        other => Err(ExpandError::InvalidInput {
            column: "role",
            got: format!("{other:?}"),
        }),
    }
}
```

and change the two input columns previously carrying `direction` to
`from_role` and `to_role`, keeping `INPUT_COLUMN_COUNT` at 14 by replacing the
single direction column and adding one — update the constant to 15 and the
column-name table alongside it.

- [ ] **Step 6: Run to verify it passes**

Run: `cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_cypher`
Expected: PASS.

- [ ] **Step 7: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime -p turso_cypher
mise run corpus
mise run cypherbench-sample
git add -A
git commit -S -m "graph/ir: delete Direction and rename FixedExpand to RoleExpand

Direction was a second source of truth for which way a traversal runs. It
survives only as the parser's spelling of an arrow, desugared into a role
pair at the binder boundary; no plan, lowering, or runtime type mentions it.

The expand vtab takes from_role/to_role names instead of a direction word,
because a relation with more than two roles has no direction to name.

Tests: plan unit tests; corpus at 8,926; cypherbench at baseline."
```

---

### Task 8: Semantic roles

Semantic mode currently stores allowed endpoint node types as
`EndpointConstraint { start, end }`. That becomes a per-role target-type list
carrying optionality and cardinality.

**Files:**
- Modify: `graph/frontend/src/semantic.rs` — table list, `EndpointConstraint` → `SemanticRole`, `SemanticRelationshipType`
- Modify: `graph/frontend/src/binder.rs:55-138` (trait), `:225-416` (`BindError`)
- Test: `graph/frontend/src/semantic.rs` (existing `mod tests`)

**Interfaces:**
- Consumes: `ir::{RoleCardinality, RoleTarget, RoleId}` (Task 1), `RegisteredRelationshipRole` (Task 2).
- Produces:
  - `SemanticRole { role: ir::RoleId, name: String, targets: Vec<ir::RoleTarget>, optional: bool, cardinality: ir::RoleCardinality }`
  - `SemanticRelationshipType { name, source, roles: Vec<SemanticRole>, properties }` — `start`/`end` gone.
  - `SEMANTIC_ROLE_TABLE: &str = "__turso_internal_graph_semantic_roles"` replacing `__turso_internal_graph_semantic_endpoints`.
  - `GraphCatalogSnapshot::relationship_role(&self, ty: ir::RelationshipTypeId, name: &str) -> Option<SemanticRole>`
  - `GraphCatalogSnapshot::relationship_roles(&self, ty: ir::RelationshipTypeId) -> Vec<SemanticRole>`

- [ ] **Step 1: Write the failing test**

In `graph/frontend/src/semantic.rs`, inside `mod tests`:

```rust
    #[test]
    fn a_semantic_role_carries_targets_optionality_and_cardinality() {
        let connection = connection();
        install_semantic_schema(&connection, TERNARY_SCHEMA).expect("install schema");
        let catalog = load_semantic_catalog(&connection, "scriptorium").expect("load catalog");

        let transcription = catalog
            .relationship_type("Transcription")
            .expect("Transcription type");
        assert_eq!(transcription.roles.len(), 3);

        let scribe = transcription.role("scribe").expect("scribe role");
        assert!(!scribe.optional);
        assert_eq!(scribe.cardinality, ir::RoleCardinality::One);
        assert_eq!(scribe.targets.len(), 1, "scribe accepts Person only");

        let witnesses = transcription.role("witness").expect("witness role");
        assert!(witnesses.optional);
        assert_eq!(witnesses.cardinality, ir::RoleCardinality::Many);
    }

    #[test]
    fn a_role_may_target_a_relationship_type() {
        // Relation-as-player: a role whose player is itself a relation. A
        // target list that could only hold node labels would make this
        // unrepresentable.
        let connection = connection();
        install_semantic_schema(&connection, CITATION_SCHEMA).expect("install schema");
        let catalog = load_semantic_catalog(&connection, "scriptorium").expect("load catalog");

        let cites = catalog.relationship_type("Citation").expect("Citation type");
        let cited = cites.role("cited").expect("cited role");
        assert!(
            cited
                .targets
                .iter()
                .any(|target| matches!(target, ir::RoleTarget::Relation(_))),
            "cited must accept a relation player, got {:?}",
            cited.targets
        );
    }
```

Add `TERNARY_SCHEMA` and `CITATION_SCHEMA` beside the existing schema fixtures in
that module, declaring `Transcription` with roles `scribe`/`text`/`folio` (plus
an optional many-valued `witness`) and `Citation` with a `cited` role targeting
`Transcription`.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --lib semantic::`
Expected: FAIL to compile with `no field roles on type SemanticRelationshipType`.

- [ ] **Step 3: Replace the endpoint constraint with roles**

In `graph/frontend/src/semantic.rs`:

```rust
pub(crate) const SEMANTIC_ROLE_TABLE: &str = "__turso_internal_graph_semantic_roles";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticRole {
    pub role: ir::RoleId,
    pub name: String,
    /// What a player may be. Empty means unconstrained.
    pub targets: Vec<ir::RoleTarget>,
    pub optional: bool,
    pub cardinality: ir::RoleCardinality,
}

impl SemanticRelationshipType {
    pub fn role(&self, name: &str) -> Option<&SemanticRole> {
        self.roles
            .iter()
            .find(|role| role.name.eq_ignore_ascii_case(name))
    }

    pub fn required_roles(&self) -> impl Iterator<Item = &SemanticRole> {
        self.roles.iter().filter(|role| !role.optional)
    }
}
```

with `SemanticRelationshipType { name: String, source: ir::SourceTableId, roles: Vec<SemanticRole>, properties: Vec<SemanticProperty> }`.

- [ ] **Step 4: Replace the endpoint table**

Replace the `__turso_internal_graph_semantic_endpoints` DDL with:

```rust
    execute_internal(connection, format!(
        "CREATE TABLE IF NOT EXISTS {SEMANTIC_ROLE_TABLE}(\
            type_id INTEGER NOT NULL, ordinal INTEGER NOT NULL, \
            name TEXT NOT NULL COLLATE NOCASE, \
            optional INTEGER NOT NULL CHECK(optional IN (0, 1)), \
            cardinality TEXT NOT NULL CHECK(cardinality IN ('one', 'many')), \
            target_kind TEXT NOT NULL CHECK(target_kind IN ('node', 'relation')), \
            target_id INTEGER NOT NULL, \
            PRIMARY KEY(type_id, ordinal, target_kind, target_id))"
    ))?;
```

One row per (role, target). A role with an empty target list gets no rows and is
recovered from the physical registration's role list, which is why loading joins
the physical roles as the left side.

Write and read it with the same `target_kind` discriminator mapping to
`ir::RoleTarget::Node` / `ir::RoleTarget::Relation`. **Do not** collapse the two
kinds into one integer space — a label and a relationship type may share a value.

- [ ] **Step 5: Expose roles on the binder's catalog trait**

Add to `GraphCatalogSnapshot`, replacing `relationship_endpoints` and
`relationship_endpoint_sources`:

```rust
    /// Roles of a relationship type in declaration order. Empty when the type
    /// is unknown.
    fn relationship_roles(&self, ty: ir::RelationshipTypeId) -> Vec<SemanticRole>;

    fn relationship_role(&self, ty: ir::RelationshipTypeId, name: &str) -> Option<SemanticRole> {
        self.relationship_roles(ty)
            .into_iter()
            .find(|role| role.name.eq_ignore_ascii_case(name))
    }
```

Schemaless mode implements it by synthesizing two required single-valued roles
named `start` and `end` with empty target lists, from the physical registration.

Replace `BindError::InvalidEndpointType` with:

```rust
    #[error("role `{role}` of relationship type `{relationship_type}` does not accept {found}")]
    RoleTargetTypeViolation {
        relationship_type: String,
        role: String,
        found: String,
        span_start: usize,
        span_end: usize,
    },
```

and update the existing endpoint-type check to build it from the role's targets.

- [ ] **Step 6: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend`
Expected: PASS.

- [ ] **Step 7: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend
mise run corpus
git add -A
git commit -S -m "graph/semantic: store roles instead of endpoint constraints

A relationship type declares named roles carrying target types, optionality,
and cardinality. Targets keep node labels and relationship types in distinct
kinds so relation-as-player is representable and a label cannot be mistaken
for the relationship type sharing its number.

Schemaless mode synthesizes two required start/end roles with no target
constraint, so the two modes present one shape to the binder.

Tests: semantic unit tests over a ternary type with an optional many-valued
role and over a role targeting a relationship type; corpus at 8,926."
```

---

### Task 9: Add roles to the create-relationship IR alongside `from`/`to`

Expand half of the mutation migration.

**Files:**
- Modify: `graph/ir/src/mutation.rs:40-70` (`CreateRelationship`)
- Modify: `graph/frontend/src/binder.rs:1472-1605`
- Test: `graph/ir/src/mutation.rs` (existing `mod tests`)

**Interfaces:**
- Consumes: `ir::RoleBinding` (Task 1), `GraphCatalogSnapshot::relationship_roles` (Task 8).
- Produces: `pub roles: Vec<RoleBinding>` on `CreateRelationship`, populated from `from`/`to` and authoritative from Task 10 onward.

- [ ] **Step 1: Write the failing test**

In `graph/ir/src/mutation.rs`'s test module:

```rust
    #[test]
    fn a_created_relationship_lists_its_role_bindings_in_declaration_order() {
        let create = sample_create_relationship();
        assert_eq!(
            create.roles,
            vec![
                RoleBinding { role: RoleId::new(1).unwrap(), value: BindingId::new(1).unwrap() },
                RoleBinding { role: RoleId::new(2).unwrap(), value: BindingId::new(2).unwrap() },
            ]
        );
    }

    #[test]
    fn a_role_binding_list_permits_the_same_player_twice() {
        // Repeated players are legal: a Match with the same team in the home
        // and away roles is a real thing to record, and nothing downstream may
        // assume role players are distinct.
        let player = BindingId::new(1).unwrap();
        let roles = vec![
            RoleBinding { role: RoleId::new(1).unwrap(), value: player },
            RoleBinding { role: RoleId::new(2).unwrap(), value: player },
        ];
        assert_eq!(roles.iter().filter(|role| role.value == player).count(), 2);
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_ir --lib mutation::`
Expected: FAIL to compile with `no field roles on type CreateRelationship`.

- [ ] **Step 3: Add the field**

In `graph/ir/src/mutation.rs`, after `direction`:

```rust
    /// One entry per filled role, in the relation type's declaration order.
    /// A repeated player is legal; nothing here assumes distinct values.
    pub roles: Vec<RoleBinding>,
```

- [ ] **Step 4: Populate it in the binder**

At `binder.rs:1586`, where `ir::CreateRelationship` is constructed:

```rust
            roles: vec![
                ir::RoleBinding { role: start_role, value: from },
                ir::RoleBinding { role: end_role, value: to },
            ],
```

with `start_role`/`end_role` read from `self.catalog.relationship_roles(relationship_type)`.

- [ ] **Step 5: Run to verify it passes**

Run: `cargo test -p turso_graph_ir -p turso_graph_frontend`
Expected: PASS.

- [ ] **Step 6: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_ir -p turso_graph_frontend
mise run corpus
git add -A
git commit -S -m "graph/ir: carry role bindings on relationship creates

Expand half of the create migration. The binder fills roles from the from
and to bindings it already resolves, so both representations agree before
the writer switches over.

Tests: mutation unit tests, including that a repeated player across two
roles is representable; corpus at 8,926."
```

---

### Task 10: Write relations from their role bindings

**Files:**
- Modify: `graph/frontend/src/mutation.rs:1832-1884` (`insert_relationship`)
- Test: `graph/frontend/tests/nary_relations.rs` (create)

**Interfaces:**
- Consumes: `CreateRelationship::roles` (Task 9), `RelationshipTableLayout::role` (Task 4).
- Produces: no new public API; `insert_relationship` now derives its fixed columns from the role bindings.

- [ ] **Step 1: Write the failing test**

Create `graph/frontend/tests/nary_relations.rs`:

```rust
//! Native n-ary relations, end to end. Everything here would be expressible
//! only by reification under the old binary model, so each test names the
//! thing reification loses.

mod fixture;

use fixture::{ternary_session, Session};

#[test]
fn a_three_role_relation_writes_one_row_with_three_endpoint_columns() {
    // Reification would write a node plus three edges and lose the fact that
    // the three players are one assertion.
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription {year: 1387}](scribe: p, text: t, folio: f)",
    );
    let rows = session.sql("SELECT scribe, txt, folio, year FROM transcriptions");
    assert_eq!(rows.len(), 1, "one relation, one row");
    assert_eq!(rows[0], vec!["1", "2", "3", "1387"]);
}

#[test]
fn the_same_player_may_fill_two_roles_of_one_relation() {
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: p)",
    );
    // The old binary writer had no way to express this without two rows.
    let rows = session.sql("SELECT scribe, folio FROM transcriptions");
    assert_eq!(rows[0], vec!["1", "1"]);
}
```

Add `ternary_session()` to `graph/frontend/tests/fixture.rs`, registering the
`Person`/`Text`/`Folio` node sources and the three-role `Transcription`
relationship source from Task 2's test, plus `Session::run`/`Session::sql`
helpers if that file does not already have them.

This test uses the standalone role pattern from Task 12; mark it
`#[ignore = "surface syntax lands in Task 12"]` and remove the attribute at the
end of Task 13. Add one non-ignored test that exercises the writer through the
IR directly:

```rust
#[test]
fn the_writer_places_each_role_player_in_its_own_column() {
    let session = ternary_session();
    session.execute_create_relation(
        "Transcription",
        &[("scribe", 1), ("text", 2), ("folio", 3)],
        &[("year", "1387")],
    );
    assert_eq!(
        session.sql("SELECT scribe, txt, folio FROM transcriptions")[0],
        vec!["1", "2", "3"]
    );
}
```

`Session::execute_create_relation` builds an `ir::CreateRelationship` with the
named roles resolved through the layout and runs it, bypassing the parser.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --test nary_relations`
Expected: FAIL — the writer places only two players, so `folio` is NULL.

- [ ] **Step 3: Derive the fixed columns from the roles**

In `graph/frontend/src/mutation.rs`, replace the two-element `fixed` slice in
`insert_relationship`:

```rust
    let mut fixed = Vec::with_capacity(create.roles.len());
    let mut spilled = Vec::new();
    for binding in &create.roles {
        let role = layout
            .role(binding.role)
            .ok_or(MutationError::UnknownRole { role: binding.role })?;
        let player = self.resolve_binding_value(binding.value)?;
        match role.cardinality {
            ir::RoleCardinality::One => fixed.push((role.column.clone(), player)),
            // A many-valued role has no column on the relation table; its
            // players land in the spill table after the relation row exists
            // and has an identity to point at.
            ir::RoleCardinality::Many => spilled.push((role.clone(), player)),
        }
    }
    let relation_id = self.insert_entity(&layout.table, &layout.identity_column, &fixed, properties)?;
```

Spill inserts land in Task 14; leave `spilled` unused here with an explicit
assertion rather than a silent drop:

```rust
    assert!(
        spilled.is_empty(),
        "many-valued roles are written in a later step; a Many role must not reach here yet"
    );
```

Add to `MutationError`:

```rust
    #[error("relation has no role {role:?}")]
    UnknownRole { role: ir::RoleId },
    #[error("a role player must resolve to an integer identity")]
    NonIntegerPlayer,
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend --test nary_relations`
Expected: PASS for the non-ignored test.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend
mise run corpus
git add -A
git commit -S -m "graph/mutation: write relations from their role bindings

insert_relationship derives its fixed column list from the create's role
bindings instead of a start and end pair, so a relation with any number of
single-valued roles writes one row with one column per role.

Many-valued roles are collected and asserted absent for now; their spill
writes land with the spill tables.

Tests: nary_relations writer test over a three-role relation; corpus at 8,926."
```

---

### Task 11: Delete `from`/`to`/`direction` and rename to `CreateRelation`

**Files:**
- Modify: `graph/ir/src/mutation.rs`, `graph/ir/src/lib.rs`
- Modify: `graph/frontend/src/binder.rs:1472-1605`, `graph/frontend/src/mutation.rs`
- Test: `graph/ir/src/mutation.rs`

**Interfaces:**
- Produces: `ir::CreateRelation { binding, source, relationship_types, roles, properties }`; `ir::Mutation::CreateRelation`; `ir::MergeRelation { create: CreateRelation, on_create, on_match }`. `CreateRelationship`, `MergeRelationship`, and the `from`/`to`/`direction` fields no longer exist.

- [ ] **Step 1: Write the failing test**

```rust
    #[test]
    fn a_create_relation_names_only_roles() {
        // Two ways to say who participates is one way too many: a writer that
        // read `from` while the binder filled `roles` would silently ignore
        // every role past the second.
        let create = sample_create_relation();
        assert_eq!(create.roles.len(), 3);
    }
```

with `sample_create_relation()` building a three-role create.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_ir --lib mutation::`
Expected: FAIL to compile with `cannot find type CreateRelation`.

- [ ] **Step 3: Delete and rename**

- `graph/ir/src/mutation.rs`: rename `CreateRelationship` → `CreateRelation` and
  `MergeRelationship` → `MergeRelation`; delete `pub from`, `pub to`, `pub direction`;
  rename the `Mutation::CreateRelationship`/`MergeRelationship` variants.
- `graph/ir/src/lib.rs`: update the re-exports.
- `graph/frontend/src/binder.rs`: drop the `from`/`to`/`direction` initializers;
  keep the endpoint resolution that produces the role bindings.
- `graph/frontend/src/mutation.rs`: update the match arms and any remaining
  `create.from` / `create.to` reads.
- `graph/frontend/tests/fixture.rs`: update `Session::execute_create_relation`
  (added in Task 10) to build an `ir::CreateRelation`.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p turso_graph_ir -p turso_graph_frontend`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime -p turso_cypher
mise run corpus
mise run cypherbench-sample
git add -A
git commit -S -m "graph/ir: rename CreateRelationship to CreateRelation and drop endpoints

Roles are now the only statement of who participates in a created relation.
Keeping from/to alongside them would let a writer that read the old fields
silently ignore every role past the second.

Tests: mutation unit tests; corpus at 8,926; cypherbench at baseline."
```

---

### Task 12: Parse the standalone role pattern

**Files:**
- Modify: `graph/cypher/src/cypher.pest:40-70`
- Modify: `graph/cypher/src/ast.rs:120-200`
- Modify: `graph/cypher/src/parser.rs:380-520`
- Test: `graph/cypher/src/parser.rs` (existing `mod tests`)

**Interfaces:**
- Produces:
  - Grammar: `pattern = { pattern_element ~ ("," ~ pattern_element)* }`, `pattern_element = { role_pattern | path_pattern }`, `role_pattern = { relationship_body ~ role_arguments }`, `role_arguments = { "(" ~ role_argument? ~ ("," ~ role_argument)* ~ ")" }`, `role_argument = { identifier ~ ":" ~ expression }`.
  - AST: `enum PatternElement { Path(PathPattern), Roles(RolePattern) }`; `struct RolePattern { relationship: RelationshipBody, roles: Vec<RoleArgument>, span: Span }`; `struct RoleArgument { name: String, player: Expression, span: Span }`.
  - `Pattern { elements: Vec<PatternElement>, span: Span }` replacing `Vec<PathPattern>`.

- [ ] **Step 1: Write the failing test**

In `graph/cypher/src/parser.rs`'s test module:

```rust
    #[test]
    fn a_standalone_role_pattern_parses_with_its_roles_in_source_order() {
        // `[` never begins a pattern element today, so the role form is
        // unambiguous against every existing pattern.
        let statement = parse("MATCH [x:Transcription {year: 1387}](scribe: p, text: t, folio: f) RETURN x");
        let PatternElement::Roles(roles) = &pattern_of(&statement).elements[0] else {
            panic!("expected a role pattern");
        };
        assert_eq!(roles.relationship.variable.as_deref(), Some("x"));
        assert_eq!(roles.relationship.types, vec!["Transcription".to_owned()]);
        assert_eq!(
            roles.roles.iter().map(|role| role.name.as_str()).collect::<Vec<_>>(),
            vec!["scribe", "text", "folio"]
        );
    }

    #[test]
    fn a_role_pattern_and_a_path_pattern_may_appear_in_one_comma_list() {
        let statement = parse("MATCH (a:Person), [x:Transcription](scribe: a) RETURN x");
        let elements = &pattern_of(&statement).elements;
        assert!(matches!(elements[0], PatternElement::Path(_)));
        assert!(matches!(elements[1], PatternElement::Roles(_)));
    }

    #[test]
    fn a_role_pattern_with_no_roles_is_a_parse_error_not_an_empty_relation() {
        // `[x:T]()` would otherwise read as a relation with no participants,
        // which the binder would then have to reject with a worse message.
        assert!(parse_result("MATCH [x:Transcription]() RETURN x").is_err());
    }

    #[test]
    fn an_arrow_pattern_still_parses_unchanged() {
        let statement = parse("MATCH (a)-[r:KNOWS]->(b) RETURN b");
        assert!(matches!(pattern_of(&statement).elements[0], PatternElement::Path(_)));
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_cypher --lib parser::`
Expected: FAIL to compile with `cannot find type PatternElement`.

- [ ] **Step 3: Extend the grammar**

In `graph/cypher/src/cypher.pest`, replace the `pattern` rule and add the role
rules. `role_pattern` must come first in the ordered choice so a leading `[`
commits to it:

```pest
pattern = { pattern_element ~ ("," ~ pattern_element)* }
pattern_element = { role_pattern | path_pattern }
role_pattern = { relationship_body ~ role_arguments }
role_arguments = { "(" ~ role_argument ~ ("," ~ role_argument)* ~ ")" }
role_argument = { identifier ~ ":" ~ expression }
```

The one-or-more form in `role_arguments` is what makes `[x:T]()` a parse error.

- [ ] **Step 4: Extend the AST**

In `graph/cypher/src/ast.rs`:

```rust
/// One comma-separated element of a MATCH or CREATE pattern.
///
/// The arrow form and the role form are different spellings of the same thing;
/// the binder resolves both to role pairs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PatternElement {
    Path(PathPattern),
    Roles(RolePattern),
}

/// `[x:Transcription {year: 1387}](scribe: p, text: t, folio: f)`
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RolePattern {
    pub variable: Option<String>,
    pub types: Vec<String>,
    pub properties: Option<MapLiteral>,
    /// Source order. The binder does not require declaration order.
    pub roles: Vec<RoleArgument>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleArgument {
    pub name: String,
    pub player: Expression,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pattern {
    pub elements: Vec<PatternElement>,
    pub span: Span,
}
```

Replace every `Vec<PathPattern>` field on `Match`, `Create`, `Merge`, and the
pattern predicate with `Pattern`.

- [ ] **Step 5: Extend the walkers**

In `graph/cypher/src/parser.rs`, add:

```rust
fn walk_pattern(pair: Pair<'_, Rule>) -> Result<ast::Pattern, ParseError> {
    let span = span_of(&pair);
    let mut elements = Vec::new();
    for element in pair.into_inner() {
        let inner = element
            .into_inner()
            .next()
            .ok_or(ParseError::MalformedPattern { span })?;
        elements.push(match inner.as_rule() {
            Rule::role_pattern => ast::PatternElement::Roles(walk_role_pattern(inner)?),
            Rule::path_pattern => ast::PatternElement::Path(walk_path_pattern(inner)?),
            other => return Err(ParseError::UnexpectedRule { rule: format!("{other:?}"), span }),
        });
    }
    Ok(ast::Pattern { elements, span })
}

fn walk_role_pattern(pair: Pair<'_, Rule>) -> Result<ast::RolePattern, ParseError> {
    let span = span_of(&pair);
    let mut inner = pair.into_inner();
    let body = inner.next().ok_or(ParseError::MalformedPattern { span })?;
    let (variable, types, range, properties) = walk_relationship_body(body)?;
    if range.is_some() {
        // A hop range names a repetition of one relationship. It has no
        // meaning on a role list, and accepting it silently would let
        // `[r:T*1..3](start: a)` look supported.
        return Err(ParseError::RangeOnRolePattern { span });
    }
    let mut roles = Vec::new();
    for argument in inner.next().into_iter().flat_map(Pair::into_inner) {
        let argument_span = span_of(&argument);
        let mut parts = argument.into_inner();
        let name = parts
            .next()
            .ok_or(ParseError::MalformedPattern { span: argument_span })?
            .as_str()
            .to_owned();
        let player = walk_expression(
            parts
                .next()
                .ok_or(ParseError::MalformedPattern { span: argument_span })?,
        )?;
        roles.push(ast::RoleArgument { name, player, span: argument_span });
    }
    Ok(ast::RolePattern { variable, types, properties, roles, span })
}
```

Factor the existing `relationship_body` handling out of `walk_relationship` into
`walk_relationship_body` returning the four-tuple, so both callers share it.

Add `ParseError::RangeOnRolePattern { span }` with the message
`"a hop range has no meaning on a role pattern"`.

- [ ] **Step 6: Run to verify it passes**

Run: `cargo test -p turso_cypher`
Expected: PASS.

- [ ] **Step 7: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_cypher -p turso_graph_frontend
mise run corpus
git add -A
git commit -S -m "graph/cypher: parse the standalone role pattern

`[x:T {props}](role: player, ...)` becomes a pattern element alongside the
path form. `[` never began a pattern element before, so the two forms are
unambiguous and every existing arrow query parses unchanged.

A role list must be non-empty, and a hop range on a role pattern is a parse
error rather than a silently accepted no-op.

Tests: parser unit tests over the role form, the mixed comma list, the empty
role list, and an unchanged arrow pattern; corpus at 8,926."
```

---

### Task 13: Bind the standalone role pattern

**Files:**
- Modify: `graph/frontend/src/binder.rs:225-416` (`BindError`), `:432-459` (`classify_statement`), `:1472-1605` (create), `:2700-2825` (match)
- Test: `graph/frontend/tests/nary_relations.rs`, `graph/frontend/tests/desugaring_golden.rs`

**Interfaces:**
- Consumes: `ast::{PatternElement, RolePattern, RoleArgument}` (Task 12), `SemanticRole` (Task 8).
- Produces: `BindError::{UnknownRole, MissingRequiredRole, RoleCardinalityViolation, DuplicateRoleArgument}`. (`RoleTargetTypeViolation` landed in Task 8; `AmbiguousRoleName` lands in Task 16.)

- [ ] **Step 1: Write the failing test**

Append to `graph/frontend/tests/nary_relations.rs`:

```rust
#[test]
fn an_unknown_role_names_the_roles_that_do_exist() {
    let session = ternary_session();
    let error = session.expect_error(
        "MATCH (p:Person {id: 1}) CREATE [x:Transcription](scribbe: p)",
    );
    assert!(error.contains("scribbe"), "the error must quote what was written: {error}");
    assert!(error.contains("scribe"), "and name a real role: {error}");
}

#[test]
fn a_missing_required_role_is_refused_at_bind_time() {
    // A relation missing a required role is a half-stated assertion. Writing
    // it and letting a NULL column stand for "unknown" would make every later
    // read of that role wrong in a way nothing reports.
    let session = ternary_session();
    let error = session.expect_error(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}) \
         CREATE [x:Transcription](scribe: p, text: t)",
    );
    assert!(error.contains("folio"), "the error must name the missing role: {error}");
}

#[test]
fn an_optional_role_may_be_omitted() {
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f)",
    );
    // `witness` is optional and was omitted; the create succeeded.
    assert_eq!(session.sql("SELECT count(*) FROM transcriptions")[0], vec!["1"]);
}

#[test]
fn naming_one_role_twice_is_refused_rather_than_last_write_wins() {
    let session = ternary_session();
    let error = session.expect_error(
        "MATCH (p:Person {id: 1}), (q:Person {id: 4}) \
         CREATE [x:Transcription](scribe: p, scribe: q)",
    );
    assert!(error.contains("scribe"), "{error}");
}

#[test]
fn a_role_rejects_a_player_of_the_wrong_type() {
    let session = ternary_session();
    let error = session.expect_error(
        "MATCH (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: t, text: t, folio: f)",
    );
    assert!(error.contains("scribe"), "{error}");
    assert!(error.contains("Text"), "the error must name what was offered: {error}");
}

#[test]
fn a_role_pattern_in_match_binds_the_relation_and_every_named_player() {
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f)",
    );
    let rows = session.query(
        "MATCH [x:Transcription](scribe: s, folio: g) RETURN s.id, g.id",
    );
    assert_eq!(rows, vec![vec!["1", "3"]]);
}

#[test]
fn a_match_role_pattern_may_leave_roles_unnamed() {
    // Naming a subset is a projection, not an under-specification: the
    // unnamed roles are simply not bound.
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f)",
    );
    assert_eq!(
        session.query("MATCH [x:Transcription](scribe: s) RETURN s.id"),
        vec![vec!["1"]]
    );
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --test nary_relations`
Expected: FAIL — the binder does not handle `PatternElement::Roles`.

- [ ] **Step 3: Add the errors**

```rust
    #[error("relationship type `{relationship_type}` has no role `{role}`; its roles are {known}")]
    UnknownRole {
        relationship_type: String,
        role: String,
        known: String,
        span_start: usize,
        span_end: usize,
    },
    #[error("relationship type `{relationship_type}` requires role `{role}`")]
    MissingRequiredRole {
        relationship_type: String,
        role: String,
        span_start: usize,
        span_end: usize,
    },
    #[error("role `{role}` of `{relationship_type}` holds one player; it was given {count}")]
    RoleCardinalityViolation {
        relationship_type: String,
        role: String,
        count: usize,
        span_start: usize,
        span_end: usize,
    },
    #[error("role `{role}` of `{relationship_type}` is named more than once")]
    DuplicateRoleArgument {
        relationship_type: String,
        role: String,
        span_start: usize,
        span_end: usize,
    },
```

- [ ] **Step 4: Bind a role pattern in CREATE**

Add to the create path, alongside the existing path-pattern handling:

```rust
    fn bind_create_role_pattern(
        &mut self,
        pattern: &ast::RolePattern,
    ) -> Result<ir::CreateRelation, BindError> {
        let (type_id, type_name) = self.single_relationship_type(&pattern.types, pattern.span)?;
        let declared = self.catalog.relationship_roles(type_id);
        let mut bound: Vec<ir::RoleBinding> = Vec::with_capacity(pattern.roles.len());
        let mut seen: HashMap<ir::RoleId, usize> = HashMap::new();

        for argument in &pattern.roles {
            let role = declared
                .iter()
                .find(|role| role.name.eq_ignore_ascii_case(&argument.name))
                .ok_or_else(|| BindError::UnknownRole {
                    relationship_type: type_name.clone(),
                    role: argument.name.clone(),
                    known: declared
                        .iter()
                        .map(|role| role.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    span_start: argument.span.start,
                    span_end: argument.span.end,
                })?;
            let count = seen.entry(role.role).or_insert(0);
            *count += 1;
            // Two arguments for a One role is a contradiction, not an
            // overwrite. Last-write-wins would silently discard a player the
            // author explicitly named.
            if *count > 1 && role.cardinality == ir::RoleCardinality::One {
                return Err(BindError::DuplicateRoleArgument {
                    relationship_type: type_name.clone(),
                    role: role.name.clone(),
                    span_start: argument.span.start,
                    span_end: argument.span.end,
                });
            }
            let value = self.bind_role_player(argument, role, &type_name)?;
            bound.push(ir::RoleBinding { role: role.role, value });
        }

        for role in declared.iter().filter(|role| !role.optional) {
            if !seen.contains_key(&role.role) {
                return Err(BindError::MissingRequiredRole {
                    relationship_type: type_name.clone(),
                    role: role.name.clone(),
                    span_start: pattern.span.start,
                    span_end: pattern.span.end,
                });
            }
        }

        // Declaration order, so the writer's column list is stable regardless
        // of the order the author wrote the arguments in.
        bound.sort_by_key(|binding| {
            declared
                .iter()
                .position(|role| role.role == binding.role)
                .unwrap_or(usize::MAX)
        });

        Ok(ir::CreateRelation {
            binding: self.declare_relationship_binding(pattern)?,
            source: self.catalog.relationship_source_for_type(type_id).ok_or(
                BindError::UnknownRelationshipType {
                    name: type_name.clone(),
                    span_start: pattern.span.start,
                    span_end: pattern.span.end,
                },
            )?,
            relationship_types: vec![type_id],
            roles: bound,
            properties: self.bind_property_map(pattern.properties.as_ref())?,
        })
    }
```

`bind_role_player` resolves the argument expression to a `BindingId` and checks
it against `role.targets`, raising `BindError::RoleTargetTypeViolation` (Task 8)
when the player's type is absent from a non-empty target list. **No cross-role
uniqueness check**: the same player under two roles is legal.

- [ ] **Step 5: Bind a role pattern in MATCH**

A MATCH role pattern with n named roles lowers to the relation scan plus one
join per named role. Add:

```rust
    fn bind_match_role_pattern(
        &mut self,
        pattern: &ast::RolePattern,
        input: ir::Plan,
    ) -> Result<ir::Plan, BindError> {
        let (type_id, type_name) = self.single_relationship_type(&pattern.types, pattern.span)?;
        let declared = self.catalog.relationship_roles(type_id);
        let source = self.relationship_source(type_id, &type_name, pattern.span)?;
        let relation = self.declare_relationship_binding(pattern)?;
        // The relation is the anchor; each named role is a join from it out to
        // its player. Unnamed roles are not bound, which is a projection over
        // the relation's participants, not an under-specified match.
        let mut plan = self.scan_relationship(input, source, relation.clone(), type_id)?;
        for argument in &pattern.roles {
            let role = self.resolve_declared_role(&declared, argument, &type_name)?;
            plan = self.join_role_player(plan, source, relation.id(), role, argument)?;
        }
        Ok(plan)
    }
```

`join_role_player` emits a `RoleExpand` whose `from_role` is the role being
joined and whose `to_role` is the same role — the relation is already bound, so
the expand runs relation → player. For a `Many` role it joins through the spill
table (Task 14).

- [ ] **Step 6: Classify a role pattern as a write when it appears under CREATE**

In `classify_statement` (`binder.rs:432-459`), extend the pattern walk to visit
`PatternElement::Roles` as well as `PatternElement::Path`, so a statement whose
only pattern is a role pattern is still classified `StatementKind::Write` under
CREATE and `StatementKind::Read` under MATCH.

- [ ] **Step 7: Un-ignore the surface-syntax tests**

Remove `#[ignore = "surface syntax lands in Task 12"]` from
`graph/frontend/tests/nary_relations.rs` and
`#[ignore = "standalone role pattern lands in Task 12"]` from
`graph/frontend/tests/desugaring_golden.rs`.

- [ ] **Step 8: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend --test nary_relations --test desugaring_golden`
Expected: PASS, including the desugaring goldens proving the arrow and role
forms bind to identical IR.

- [ ] **Step 9: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend
mise run corpus
mise run cypherbench-sample
git add -A
git commit -S -m "graph/binder: bind the standalone role pattern

CREATE resolves each named role against the type's declaration, refuses an
unknown role while naming the real ones, refuses a missing required role
rather than writing a NULL that would later read as a real answer, refuses a
repeated name for a single-valued role instead of last-write-wins, and does
not require cross-role uniqueness: the same player under two roles is legal.

MATCH anchors on the relation and joins one player per named role, leaving
unnamed roles unbound.

The desugaring goldens now run: the arrow form and the role form of the same
pattern bind to identical IR.

Tests: nary_relations, desugaring_golden; corpus at 8,926; cypherbench at
baseline."
```

---

### Task 14: Many-valued roles

**Files:**
- Modify: `graph/frontend/src/mutation.rs` (`insert_relationship`, delete path)
- Modify: `graph/frontend/src/lowering.rs` (spill-table join)
- Modify: `graph/frontend/src/binder.rs` (`RoleCardinalityViolation` for a `One` role given a list)
- Test: `graph/frontend/tests/nary_relations.rs`

**Interfaces:**
- Consumes: `RelationshipRoleLayout::spill_table` (Task 4), the `spilled` vector from Task 10.
- Produces: no new public API.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn a_many_valued_role_holds_several_players_in_one_relation() {
    // Two witnesses to one transcription is one assertion with two players,
    // not two assertions. Splitting it into two rows would double-count the
    // transcription in every aggregate.
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}), \
               (w1:Person {id: 4}), (w2:Person {id: 5}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f, witness: w1, witness: w2)",
    );
    assert_eq!(
        session.sql("SELECT count(*) FROM transcriptions")[0],
        vec!["1"],
        "one relation row"
    );
    assert_eq!(
        session.sql("SELECT count(*) FROM transcriptions__witness")[0],
        vec!["2"],
        "two spilled players"
    );
}

#[test]
fn a_hop_through_a_many_valued_role_returns_every_player() {
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}), \
               (w1:Person {id: 4}), (w2:Person {id: 5}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f, witness: w1, witness: w2)",
    );
    let mut ids = session.query("MATCH [x:Transcription](witness: w) RETURN w.id");
    ids.sort();
    assert_eq!(ids, vec![vec!["4"], vec!["5"]]);
}

#[test]
fn deleting_a_relation_removes_its_spilled_players() {
    // A spill row pointing at a deleted relation is a dangling participant
    // that a later hop would surface as a live player.
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}), (w:Person {id: 4}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f, witness: w)",
    );
    session.run("MATCH [x:Transcription](scribe: s) DELETE x");
    assert_eq!(
        session.sql("SELECT count(*) FROM transcriptions__witness")[0],
        vec!["0"]
    );
}

#[test]
fn a_single_valued_role_given_two_players_is_refused() {
    let session = ternary_session();
    let error = session.expect_error(
        "MATCH (p:Person {id: 1}), (q:Person {id: 4}), (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: p, scribe: q, text: t, folio: f)",
    );
    assert!(error.contains("scribe"), "{error}");
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --test nary_relations many_valued`
Expected: FAIL — the assertion added in Task 10 fires:
`many-valued roles are written in a later step`.

- [ ] **Step 3: Write the spill rows**

Replace that assertion in `insert_relationship` with:

```rust
    for (role, player) in spilled {
        let table = role
            .spill_table
            .as_ref()
            .expect("a Many role always has a spill table");
        self.execute_internal(&format!(
            "INSERT INTO {}(relation_id, node_id) VALUES ({}, {})",
            quote_identifier(table),
            relation_id,
            player.as_integer().ok_or(MutationError::NonIntegerPlayer)?
        ))?;
    }
```

- [ ] **Step 4: Delete the spill rows with the relation**

In the relationship delete path, after deleting the relation row, add one delete
per many-valued role:

```rust
    for role in layout
        .roles
        .iter()
        .filter(|role| role.cardinality == ir::RoleCardinality::Many)
    {
        let table = role.spill_table.as_ref().expect("Many role spill table");
        self.execute_internal(&format!(
            "DELETE FROM {} WHERE relation_id = {}",
            quote_identifier(table),
            relation_id
        ))?;
    }
```

- [ ] **Step 5: Join through the spill table when hopping a `Many` role**

In `lower_role_expand`, when either role of the pair is many-valued, the join
goes through the spill table instead of a column:

```rust
    /// A `Many` role has no column on the relation table, so the hop runs
    /// relation -> spill -> player. The spill table is indexed in both
    /// directions, so this is an index probe from whichever side is bound.
    fn role_join_expression(
        layout: &RelationshipTableLayout,
        role: &RelationshipRoleLayout,
        relationship_alias: &str,
        spill_alias: &str,
    ) -> String {
        match &role.spill_table {
            None => format!("{relationship_alias}.{}", quote_identifier(&role.column)),
            Some(table) => format!(
                "(SELECT {spill_alias}.node_id FROM {} {spill_alias} \
                 WHERE {spill_alias}.relation_id = {relationship_alias}.{})",
                quote_identifier(table),
                quote_identifier(&layout.identity_column)
            ),
        }
    }
```

and emit a `JOIN` rather than a scalar subquery when the role is on the produced
side, so a relation with two witnesses yields two rows:

```rust
        if role.spill_table.is_some() {
            joins.push(format!(
                "JOIN {} {spill_alias} ON {spill_alias}.relation_id = {relationship_alias}.{}",
                quote_identifier(role.spill_table.as_ref().expect("checked")),
                quote_identifier(&layout.identity_column)
            ));
        }
```

- [ ] **Step 6: Refuse a list for a single-valued role**

In `bind_create_role_pattern`, the duplicate check from Task 13 already refuses
a repeated `One` role. Extend it to report `RoleCardinalityViolation` with the
observed count when more than two arguments name the same `One` role, so the
message says how many players were offered.

- [ ] **Step 7: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend --test nary_relations`
Expected: PASS.

- [ ] **Step 8: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend
mise run corpus
mise run cypherbench-sample
git add -A
git commit -S -m "graph/frontend: implement many-valued roles

A Many role stores its players in <relation>__<role>, indexed in both
directions. Creating a relation writes one row plus one spill row per
player; deleting it removes the spill rows, so no dangling participant can
surface as a live player on a later hop.

A hop through a many-valued role joins the spill table rather than reading a
column, so a relation with two players in one role yields two rows.

Tests: nary_relations many-valued create, hop, delete, and the refusal of a
list for a single-valued role; corpus at 8,926; cypherbench at baseline."
```

---

### Task 15: Role updates after create

**Assumption, not a spec decision.** The spec lists role updates in v1 but gives
no syntax. This task uses `SET [t](scribe: s2)` — the standalone role pattern as
a `SET` item. `[` cannot begin a `set_item` today, so it is unambiguous, and the
form matches the create exactly. If a reviewer prefers another spelling, this is
the task to change.

**Files:**
- Modify: `graph/cypher/src/cypher.pest` (`set_item`), `graph/cypher/src/ast.rs`, `graph/cypher/src/parser.rs`
- Modify: `graph/ir/src/mutation.rs` (`SetRoles`), `graph/frontend/src/binder.rs`, `graph/frontend/src/mutation.rs`
- Test: `graph/frontend/tests/nary_relations.rs`

**Interfaces:**
- Produces:
  - Grammar: `set_item = { set_role_item | set_property_item | set_merge_item | set_replace_item | set_label_item }`, `set_role_item = { "[" ~ identifier ~ "]" ~ role_arguments }`.
  - `ast::SetItem::Roles(RoleUpdate { relation: String, roles: Vec<RoleArgument>, span })`
  - `ir::Mutation::SetRoles(SetRoles { relation: BindingId, source: SourceTableId, roles: Vec<RoleBinding>, replace_many: bool })`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn a_single_valued_role_can_be_repointed_after_create() {
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f)",
    );
    session.run(
        "MATCH [x:Transcription](text: t), (q:Person {id: 4}) SET [x](scribe: q)",
    );
    assert_eq!(session.sql("SELECT scribe FROM transcriptions")[0], vec!["4"]);
    assert_eq!(
        session.sql("SELECT count(*) FROM transcriptions")[0],
        vec!["1"],
        "an update repoints the relation; it does not create a second one"
    );
}

#[test]
fn setting_a_many_valued_role_replaces_its_whole_player_set() {
    // Replace, not append. Append has no syntax to undo, and a SET that
    // silently accumulated would make the same statement run twice mean
    // something different from running it once.
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}), (w:Person {id: 4}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f, witness: w)",
    );
    session.run(
        "MATCH [x:Transcription](text: t), (w2:Person {id: 5}) SET [x](witness: w2)",
    );
    let rows = session.sql("SELECT node_id FROM transcriptions__witness");
    assert_eq!(rows, vec![vec!["5"]], "the previous witness is gone");
}

#[test]
fn a_role_update_rejects_a_player_of_the_wrong_type() {
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f)",
    );
    let error = session.expect_error(
        "MATCH [x:Transcription](text: t) SET [x](scribe: t)",
    );
    assert!(error.contains("scribe"), "{error}");
}

#[test]
fn a_role_update_cannot_unset_a_required_role() {
    // There is no null player. Clearing a required role would leave a
    // half-stated assertion behind, which is the same thing the create path
    // refuses.
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f)",
    );
    assert!(session
        .expect_error("MATCH [x:Transcription](text: t) SET [x](scribe: null)")
        .contains("scribe"));
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --test nary_relations role_update`
Expected: FAIL — `SET [x](...)` is a parse error.

- [ ] **Step 3: Extend the grammar and AST**

```pest
set_item = { set_role_item | set_property_item | set_merge_item | set_replace_item | set_label_item }
set_role_item = { "[" ~ identifier ~ "]" ~ role_arguments }
```

`set_role_item` goes first so a leading `[` commits to it.

```rust
/// `SET [x](scribe: q)` — repoint one or more roles of an already-bound
/// relation. Setting a many-valued role replaces its whole player set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleUpdate {
    pub relation: String,
    pub roles: Vec<RoleArgument>,
    pub span: Span,
}
```

added to `ast::SetItem` as `Roles(RoleUpdate)`, with a `walk_set_role_item`
reusing the `role_arguments` walker from Task 12.

- [ ] **Step 4: Add the IR**

```rust
/// Repoint roles of an existing relation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetRoles {
    pub relation: BindingId,
    pub source: SourceTableId,
    pub roles: Vec<RoleBinding>,
    /// True when any named role is many-valued: its spill rows are deleted
    /// before the new players are written, so SET replaces rather than
    /// appends and running the statement twice means what running it once
    /// means.
    pub replace_many: bool,
}
```

as `Mutation::SetRoles(SetRoles)`.

- [ ] **Step 5: Bind it**

Reuse `bind_role_player` from Task 13 for the target-type check. The
required-role check does **not** apply — an update names a subset by design —
but a null or missing player for a named role is refused:

```rust
        if matches!(argument.player, ast::Expression::Null) {
            return Err(BindError::MissingRequiredRole {
                relationship_type: type_name.clone(),
                role: role.name.clone(),
                span_start: argument.span.start,
                span_end: argument.span.end,
            });
        }
```

- [ ] **Step 6: Execute it**

In `graph/frontend/src/mutation.rs`, add:

```rust
    fn set_roles(&mut self, update: &ir::SetRoles) -> Result<(), MutationError> {
        let layout = self.layout(update.source)?;
        let relation_id = self.resolve_relation_id(update.relation)?;
        let mut assignments = Vec::new();
        for binding in &update.roles {
            let role = layout
                .role(binding.role)
                .ok_or(MutationError::UnknownRole { role: binding.role })?;
            let player = self.resolve_binding_value(binding.value)?;
            match &role.spill_table {
                None => assignments.push(format!(
                    "{} = {}",
                    quote_identifier(&role.column),
                    sql_value(&player)
                )),
                Some(table) => {
                    self.execute_internal(&format!(
                        "DELETE FROM {} WHERE relation_id = {relation_id}",
                        quote_identifier(table)
                    ))?;
                    self.execute_internal(&format!(
                        "INSERT INTO {}(relation_id, node_id) VALUES ({relation_id}, {})",
                        quote_identifier(table),
                        player.as_integer().ok_or(MutationError::NonIntegerPlayer)?
                    ))?;
                }
            }
        }
        if !assignments.is_empty() {
            self.execute_internal(&format!(
                "UPDATE {} SET {} WHERE {} = {relation_id}",
                quote_identifier(&layout.table),
                assignments.join(", "),
                quote_identifier(&layout.identity_column)
            ))?;
        }
        Ok(())
    }
```

Two arguments naming one many-valued role in a single `SET` must both land: the
delete runs once per role, not once per argument. Group the arguments by role
before executing.

- [ ] **Step 7: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend --test nary_relations`
Expected: PASS.

- [ ] **Step 8: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_cypher -p turso_graph_frontend
mise run corpus
git add -A
git commit -S -m "graph/frontend: allow repointing roles after create

SET [x](role: player) repoints roles of an already-bound relation, using the
same standalone role syntax as the create. Setting a many-valued role
replaces its whole player set rather than appending, so running the same
statement twice means what running it once means.

A role update names a subset by design, so the required-role check does not
apply, but a null player is refused: there is no way to leave a required
role unfilled.

Tests: nary_relations role updates over single-valued repointing,
many-valued replacement, target-type refusal, and null refusal; corpus at
8,926."
```

---

### Task 16: Role-edge read sugar

`(t:Transcription)-[:scribe]->(s)` reads the `scribe` role of a bound relation.
Resolved in the binder, not the grammar: whether `scribe` is a role or a
relationship type depends on what `t` is bound to.

**Files:**
- Modify: `graph/frontend/src/binder.rs:2700-2825`
- Test: `graph/frontend/tests/nary_relations.rs`

**Interfaces:**
- Produces: `BindError::AmbiguousRoleName { name, relationship_type, span_start, span_end }`.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn an_arrow_from_a_relation_reads_that_relations_role() {
    // Sugar for the role pattern. Without it, reading one participant of a
    // ternary relation would need the full role form even when only one role
    // is wanted.
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f)",
    );
    assert_eq!(
        session.query("MATCH (x:Transcription)-[:scribe]->(s) RETURN s.id"),
        vec![vec!["1"]]
    );
}

#[test]
fn the_role_arrow_and_the_role_pattern_bind_to_the_same_plan() {
    let sugar = bind_in_ternary("MATCH (x:Transcription)-[:scribe]->(s) RETURN s.id");
    let explicit = bind_in_ternary("MATCH [x:Transcription](scribe: s) RETURN s.id");
    assert_eq!(sugar, explicit);
}

#[test]
fn a_name_that_is_both_a_role_and_a_relationship_type_is_ambiguous() {
    // Guessing here would make a query mean one thing today and another after
    // an unrelated schema addition.
    let session = ambiguous_name_session();
    let error = session.expect_error("MATCH (x:Transcription)-[:scribe]->(s) RETURN s");
    assert!(error.contains("ambiguous"), "{error}");
    assert!(error.contains("scribe"), "{error}");
}

#[test]
fn the_role_arrow_is_only_available_from_a_relation_binding() {
    // From a node, `scribe` must still resolve as a relationship type, or
    // adding a role named like an existing type would change what existing
    // node queries mean.
    let session = ternary_session();
    let error = session.expect_error("MATCH (p:Person)-[:scribe]->(s) RETURN s");
    assert!(
        error.contains("relationship type"),
        "from a node the name must resolve as a type, got {error}"
    );
}
```

`ambiguous_name_session()` registers a graph where `scribe` is both a role of
`Transcription` and a relationship type.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --test nary_relations role_arrow`
Expected: FAIL — `scribe` resolves as an unknown relationship type.

- [ ] **Step 3: Resolve the name against the source binding**

In the expand binding path, before resolving the relationship type:

```rust
        // The name after `:` is a role only when the pattern's source binding
        // is a relation. From a node it is a relationship type, unchanged, so
        // adding a role cannot change what an existing node query means.
        if let Some(relation_type) = self.relation_type_of_binding(source_binding) {
            let role = self.catalog.relationship_role(relation_type, name);
            let is_type = self.catalog.relationship_type_id(name).is_some();
            match (role, is_type) {
                (Some(role), false) => {
                    return self.bind_role_edge(source_binding, relation_type, role, pattern);
                }
                (Some(role), true) => {
                    return Err(BindError::AmbiguousRoleName {
                        name: role.name.clone(),
                        relationship_type: self.relationship_type_name(relation_type),
                        span_start: pattern.span.start,
                        span_end: pattern.span.end,
                    });
                }
                (None, _) => {}
            }
        }
```

`bind_role_edge` builds exactly what `bind_match_role_pattern` builds for a
single named role, so the two forms produce identical plans.

Add:

```rust
    #[error("`{name}` is both a role of `{relationship_type}` and a relationship type; write the role form `[x:{relationship_type}]({name}: target)` or qualify the type")]
    AmbiguousRoleName {
        name: String,
        relationship_type: String,
        span_start: usize,
        span_end: usize,
    },
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend --test nary_relations`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend
mise run corpus
mise run cypherbench-sample
git add -A
git commit -S -m "graph/binder: read a relation's role through arrow syntax

(x:Transcription)-[:scribe]->(s) binds the same plan as the explicit role
pattern. The name resolves as a role only when the source binding is a
relation, so adding a role can never change what an existing node query
means, and a name that is both a role and a relationship type is refused as
ambiguous rather than guessed.

Tests: nary_relations role-arrow read, plan equality with the role form, the
ambiguity refusal, and node-source resolution unchanged; corpus at 8,926."
```

---

### Task 17: Role-aware traversal, path policy, and the semantic profile

**Files:**
- Modify: `graph/runtime/src/csr.rs:30-200`, `graph/runtime/src/traversal.rs`, `graph/runtime/src/path_policy.rs:22`, `:102-179`
- Modify: `graph/frontend/src/snapshot.rs:620-660`
- Modify: `graph/ir/src/semantics.rs:8`, `:40-90`
- Modify: `graph/ir/tests/semantic_profile_pin.rs:8-9`
- Test: `graph/runtime/src/csr.rs`, `graph/runtime/src/path_policy.rs`

**Interfaces:**
- Produces:
  - `EdgeInput { relationship, from_role: RoleId, to_role: RoleId, source, target, relationship_type, weight }`
  - `Graph { nodes, node_indexes, adjacency: HashMap<(RelationshipTypeId, RoleId, RoleId), Csr> }`
  - `Graph::neighbors(&self, node: NodeId, pair: (RelationshipTypeId, RoleId, RoleId)) -> NeighborCursor`
  - `PathPolicyError::RolePairRequired { relationship_type: String, arity: usize }`
  - `resolve_path_algorithm(uniqueness, selector, weights, arity: usize, role_pair: Option<(RoleId, RoleId)>)`
  - `PATH_POLICY_VERSION: u32 = 2`; `SEMANTIC_PROFILE_VERSION: u32 = 3`; `SEMANTIC_PROFILE.path_policy_version = 2`.

- [ ] **Step 1: Write the failing tests**

In `graph/runtime/src/path_policy.rs`'s test module:

```rust
    #[test]
    fn a_relation_with_more_than_two_roles_requires_an_explicit_role_pair() {
        // A k-role relation exposes k*(k-1) directed pairs. Picking one is a
        // guess about which traversal the author meant, and the wrong guess
        // returns a plausible, wrong path.
        assert!(matches!(
            resolve_path_algorithm(
                PathUniqueness::Trail,
                PathSelector::Shortest,
                WeightClass::Unweighted,
                3,
                None,
            ),
            Err(PathPolicyError::RolePairRequired { arity: 3, .. })
        ));
    }

    #[test]
    fn a_two_role_relation_needs_no_explicit_pair_because_there_is_only_one() {
        // Arity 2 has exactly one ordered pair per direction, so there is
        // nothing to guess and every existing query keeps working.
        assert_eq!(
            resolve_path_algorithm(
                PathUniqueness::Trail,
                PathSelector::Shortest,
                WeightClass::Unweighted,
                2,
                None,
            ),
            Ok(PathAlgorithm::BreadthFirst)
        );
    }

    #[test]
    fn an_explicit_pair_over_a_ternary_relation_resolves_normally() {
        assert_eq!(
            resolve_path_algorithm(
                PathUniqueness::Trail,
                PathSelector::Shortest,
                WeightClass::Unweighted,
                3,
                Some((role(1), role(2))),
            ),
            Ok(PathAlgorithm::BreadthFirst)
        );
    }
```

In `graph/runtime/src/csr.rs`'s test module:

```rust
    #[test]
    fn adjacency_is_keyed_by_the_role_pair_it_was_built_from() {
        // A single forward/reverse pair cannot hold a ternary relation's six
        // directed pairs; merging them would let a scribe->text hop return a
        // folio.
        let graph = ternary_graph();
        let scribe_to_text = graph.neighbors(node(1), (relation_type(1), role(1), role(2)));
        assert_eq!(scribe_to_text.collect::<Vec<_>>(), vec![node(2)]);
        let scribe_to_folio = graph.neighbors(node(1), (relation_type(1), role(1), role(3)));
        assert_eq!(scribe_to_folio.collect::<Vec<_>>(), vec![node(3)]);
    }

    #[test]
    fn a_two_role_graph_has_exactly_the_two_pairs_it_had_as_forward_and_reverse() {
        let graph = binary_graph();
        assert_eq!(graph.adjacency.len(), 2, "one per direction, as before");
    }

    #[test]
    fn a_path_element_records_the_role_it_entered_and_left_by() {
        // Without the roles, a path over a ternary relation cannot be read
        // back: the same relation appears in several pairs.
        let path = shortest_path_in(&ternary_graph(), node(1), node(3));
        assert_eq!(path.elements[0].from_role, role(1));
        assert_eq!(path.elements[0].to_role, role(3));
    }
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p turso_graph_runtime`
Expected: FAIL to compile — `resolve_path_algorithm` takes three arguments and
`Graph` has no `adjacency`.

- [ ] **Step 3: Key adjacency by the role pair**

In `graph/runtime/src/csr.rs`, replace `forward`/`reverse` with:

```rust
/// Adjacency keyed by the ordered role pair it traverses.
///
/// A relation with k roles exposes k*(k-1) directed pairs. For k = 2 that is
/// exactly the forward and reverse CSR this replaces, so a binary graph builds
/// the same two structures it always did.
pub struct Graph {
    pub nodes: Vec<NodeId>,
    pub node_indexes: HashMap<NodeId, usize>,
    pub adjacency: HashMap<(RelationshipTypeId, RoleId, RoleId), Csr>,
}

impl Graph {
    pub fn neighbors(
        &self,
        node: NodeId,
        pair: (RelationshipTypeId, RoleId, RoleId),
    ) -> NeighborCursor<'_> {
        match self.adjacency.get(&pair) {
            Some(csr) => NeighborCursor::over(csr, self.node_indexes.get(&node).copied()),
            None => NeighborCursor::empty(),
        }
    }
}
```

and build one CSR per observed pair from the edge inputs, which now carry
`from_role`/`to_role`.

- [ ] **Step 4: Record roles on path elements**

Add `from_role: RoleId` and `to_role: RoleId` to the path element type in
`graph/runtime/src/traversal.rs`, populated from the pair the hop traversed.

- [ ] **Step 5: Extend the path policy**

In `graph/runtime/src/path_policy.rs`:

```rust
/// Bump on any change to the table below, and mirror into
/// `turso_graph_ir::SEMANTIC_PROFILE.path_policy_version`.
pub const PATH_POLICY_VERSION: u32 = 2;
```

Add the variant and the arity guard at the top of `resolve_path_algorithm`,
before the selector match:

```rust
    // A relation with k roles exposes k*(k-1) directed pairs. Choosing one
    // silently would answer a question the author did not ask, so the pair is
    // required rather than defaulted. Arity 2 has one pair per direction and
    // needs no annotation.
    if arity > 2 && role_pair.is_none() {
        return Err(PathPolicyError::RolePairRequired {
            relationship_type: relationship_type.to_owned(),
            arity,
        });
    }
```

```rust
    #[error("variable-length traversal over `{relationship_type}` must name a role pair: it has {arity} roles and therefore {} directed pairs", arity * (arity - 1))]
    RolePairRequired {
        relationship_type: String,
        arity: usize,
    },
```

`PathPolicyError` currently derives `Copy`; `RolePairRequired` carries a
`String`, so drop `Copy` from the derive and fix the two call sites that rely on
it.

Update `every_combination_in_the_table_has_a_verdict` to iterate arity 2 and 3
with and without a pair, so the table stays total over the new dimension.

- [ ] **Step 6: Extract role-pair edges into the snapshot**

In `graph/frontend/src/snapshot.rs`, replace the `start_column`/`end_column`
edge extraction (lines 631-632) with one pass per ordered pair of single-valued
roles, plus one pass per (`One`, `Many`) pair joining the spill table. Bump
`GRAPH_CATALOG_VERSION` to 4.

- [ ] **Step 7: Bump the semantic profile and re-pin the digest**

In `graph/ir/src/semantics.rs`:

```rust
pub const SEMANTIC_PROFILE_VERSION: u32 = 3;
```

with `path_policy_version: 2` and a new recorded choice:

```rust
    relationship_arity: "native n-ary: a relation declares named roles; \
                         binary is the two-role layout, not a separate kind",
```

Run the pin test, read the observed digest from the failure, and paste it into
`graph/ir/tests/semantic_profile_pin.rs`:

```rust
/// Digest of `SEMANTIC_PROFILE.render()` at version 3.
const PINNED_DIGEST: &str = "<paste the digest the failing test prints>";
```

- [ ] **Step 8: Run to verify they pass**

Run: `cargo test -p turso_graph_ir -p turso_graph_runtime -p turso_graph_frontend`
Expected: PASS, including `the_semantic_profile_mirrors_this_policy_version`.

- [ ] **Step 9: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime -p turso_cypher
mise run corpus
mise run cypherbench-sample
git add -A
git commit -S -m "graph/runtime: traverse by role pair and require one past arity 2

Adjacency is keyed by the ordered role pair it traverses instead of a single
forward/reverse pair, which for a two-role relation is exactly the two
structures it built before. Path elements record the role entered and left,
because a relation appearing in several pairs cannot otherwise be read back
from a path.

Variable-length and shortest-path traversal over a relation with more than
two roles must name a role pair: k roles expose k*(k-1) directed pairs, and
choosing one silently would answer a question the author did not ask.

PATH_POLICY_VERSION and SEMANTIC_PROFILE_VERSION move together with the
re-pinned digest, so every recorded corpus row stays interpretable against
the profile it was produced under.

Tests: csr role-pair adjacency and role-annotated paths, path_policy arity
rules, semantic profile pin; corpus at 8,926; cypherbench at baseline."
```

---

### Task 18: MERGE over roles, and execution-time player validation

**Files:**
- Modify: `graph/frontend/src/binder.rs` (merge path), `graph/frontend/src/mutation.rs` (`merge_relation`)
- Test: `graph/frontend/tests/nary_relations.rs`

**Interfaces:**
- Consumes: `ir::MergeRelation` (Task 11), `bind_create_role_pattern` (Task 13).
- Produces: `MutationError::RolePlayerTypeViolation { role: String, found: String }` — the execution-time twin of the bind-time `RoleTargetTypeViolation`.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn merge_matches_on_the_full_set_of_bound_required_roles() {
    // Matching on a subset would make a second MERGE with a different folio
    // silently update the first transcription instead of creating a second
    // one, collapsing two distinct assertions into one.
    let session = ternary_session();
    let create = "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}) \
                  MERGE [x:Transcription](scribe: p, text: t, folio: f)";
    session.run(create);
    session.run(create);
    assert_eq!(
        session.sql("SELECT count(*) FROM transcriptions")[0],
        vec!["1"],
        "the second MERGE matched the first relation"
    );

    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (g:Folio {id: 6}) \
         MERGE [x:Transcription](scribe: p, text: t, folio: g)",
    );
    assert_eq!(
        session.sql("SELECT count(*) FROM transcriptions")[0],
        vec!["2"],
        "a different folio is a different assertion"
    );
}

#[test]
fn a_parameterised_player_of_the_wrong_type_is_refused_before_any_write() {
    // A parameter's type is unknown at bind time. Checking it after the
    // INSERT would leave a relation whose participation violates the schema
    // the moment the savepoint is not rolled back.
    let session = ternary_session();
    let error = session.expect_error_with_params(
        "MATCH (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: $who, text: t, folio: f)",
        &[("who", "2")], // a Text identity, not a Person
    );
    assert!(error.contains("scribe"), "{error}");
    assert_eq!(
        session.sql("SELECT count(*) FROM transcriptions")[0],
        vec!["0"],
        "no relation row survived the refusal"
    );
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --test nary_relations merge`
Expected: FAIL — MERGE still matches on the two-endpoint key.

- [ ] **Step 3: Match MERGE on the bound required roles**

In `graph/frontend/src/mutation.rs`, build the merge probe from every bound
role rather than from a start/end pair:

```rust
    /// The match key is the full set of bound required roles. A subset key
    /// would let two relations that differ in an unnamed role collapse into
    /// one, which is a silent loss of an assertion.
    fn merge_probe_predicate(
        layout: &RelationshipTableLayout,
        roles: &[ir::RoleBinding],
        values: &HashMap<ir::RoleId, Value>,
    ) -> Result<String, MutationError> {
        let mut clauses = Vec::with_capacity(roles.len());
        for binding in roles {
            let role = layout
                .role(binding.role)
                .ok_or(MutationError::UnknownRole { role: binding.role })?;
            let value = values
                .get(&binding.role)
                .ok_or(MutationError::UnknownRole { role: binding.role })?;
            clauses.push(match &role.spill_table {
                None => format!(
                    "{} = {}",
                    quote_identifier(&role.column),
                    sql_value(value)
                ),
                // A many-valued role matches on membership; MERGE over a Many
                // role therefore matches a relation that already has this
                // player in that role.
                Some(table) => format!(
                    "EXISTS (SELECT 1 FROM {} WHERE relation_id = {}.{} AND node_id = {})",
                    quote_identifier(table),
                    quote_identifier(&layout.table),
                    quote_identifier(&layout.identity_column),
                    sql_value(value)
                ),
            });
        }
        Ok(clauses.join(" AND "))
    }
```

- [ ] **Step 4: Validate dynamic players before writing**

In `insert_relationship` and `set_roles`, resolve every role player and check it
against the role's target types **before** the first `INSERT` or `UPDATE`:

```rust
    // Bind-time checking cannot see a parameter's type. Validating here, before
    // any physical write, keeps the savepoint from ever containing a relation
    // whose participation violates the schema.
    for (role, player) in &resolved {
        self.check_role_target(role, player)?;
    }
```

`check_role_target` looks the player's label or relationship type up from the
snapshot and raises:

```rust
    #[error("role `{role}` does not accept {found}")]
    RolePlayerTypeViolation { role: String, found: String },
```

A role with an empty target list accepts anything and is skipped.

- [ ] **Step 5: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend --test nary_relations`
Expected: PASS.

- [ ] **Step 6: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend
mise run corpus
git add -A
git commit -S -m "graph/mutation: merge on the bound roles and check dynamic players first

MERGE probes on the full set of bound required roles. A subset key would let
two relations differing only in an unnamed role collapse into one, silently
losing an assertion.

Role players that arrive as parameters have no type at bind time, so they
are validated against the role's target types before the first physical
write rather than after, keeping the mutation savepoint from ever holding a
relation whose participation violates the schema.

Tests: nary_relations merge identity and parameterised-player refusal;
corpus at 8,926."
```

---

### Task 19: Create atomicity and relation-as-player

**Files:**
- Modify: `graph/frontend/src/mutation.rs` (savepoint coverage of spill inserts)
- Test: `graph/frontend/tests/nary_relations.rs`

**Interfaces:**
- Consumes: everything through Task 18. No new public API.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn a_failure_partway_through_an_n_ary_create_leaves_nothing_behind() {
    // This is the integrity property reified modeling cannot provide:
    // reification needs one statement per role, so a failure between them
    // leaves a partially stated assertion that reads as complete.
    let session = ternary_session();
    session.fail_after_nth_internal_statement(2);
    let error = session.expect_error(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}), (w:Person {id: 4}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f, witness: w)",
    );
    assert!(!error.is_empty());
    assert_eq!(
        session.sql("SELECT count(*) FROM transcriptions")[0],
        vec!["0"],
        "no relation row"
    );
    assert_eq!(
        session.sql("SELECT count(*) FROM transcriptions__witness")[0],
        vec!["0"],
        "no spilled player either: the spill inserts share the relation's savepoint"
    );
}

#[test]
fn a_relation_may_be_a_player_of_another_relation() {
    // Relation-as-player needs no special case in the writer: a relation
    // identity is an identity like any other. What it needs is a role whose
    // target list carries RoleTarget::Relation, which the semantic layer
    // already stores.
    let session = citation_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f)",
    );
    session.run(
        "MATCH [x:Transcription](text: t), (c:Text {id: 7}) \
         CREATE [y:Citation](cited: x, source: c)",
    );
    assert_eq!(
        session.query("MATCH [y:Citation](cited: r) RETURN r.year"),
        vec![vec![""]],
        "the cited player is the transcription relation itself"
    );
}

#[test]
fn a_role_that_does_not_accept_relations_refuses_a_relation_player() {
    let session = citation_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f)",
    );
    let error = session.expect_error(
        "MATCH [x:Transcription](text: t) CREATE [y:Citation](cited: x, source: x)",
    );
    assert!(error.contains("source"), "{error}");
}
```

`citation_session()` extends `ternary_session()` with the `Citation` type from
Task 8's `CITATION_SCHEMA`: a `cited` role targeting `Transcription` and a
`source` role targeting `Text` only. `fail_after_nth_internal_statement` is a
test-only injection hook on the session's internal executor.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --test nary_relations atomicity`
Expected: FAIL — the spill insert commits independently of the relation row.

- [ ] **Step 3: Bring the spill inserts inside the relation's savepoint**

The mutation path already opens `SAVEPOINT __turso_graph_mutation`
(`graph/frontend/src/mutation.rs`). Confirm the spill inserts from Task 14 and
the role updates from Task 15 execute between that savepoint and its release,
and move them inside if they do not. Assert it rather than assuming:

```rust
        debug_assert!(
            self.savepoint_depth > 0,
            "a spill insert outside the mutation savepoint could survive a failed create"
        );
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend --test nary_relations`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend
mise run corpus
mise run cypherbench-sample
git add -A
git commit -S -m "graph/mutation: make n-ary creates atomic and allow relation players

A failure partway through a create leaves neither the relation row nor any
spilled player: the spill inserts share the relation's savepoint. This is the
integrity property reification cannot offer, since reification needs one
statement per role and a failure between them leaves a partially stated
assertion that reads as complete.

A relation may fill a role of another relation, and a role whose target list
names only node labels refuses one.

Tests: nary_relations atomicity under injected failure, relation-as-player
round trip, and the refusal; corpus at 8,926; cypherbench at baseline."
```

---

### Task 20: Documentation and gate deletion

**Files:**
- Modify: `docs/graph.md`
- Modify: `graph/CONFORMANCE.md`
- Modify: `.specs/graph-semantic-schema-overlay.agent-spec.md:38`, `:54`, `:94`, `:101`, `:126`, `:145`, `:170`, `:245`, `:276`
- Modify: `docs/superpowers/plans/2026-07-22-graph-semantic-schema-overlay.md:21`, `:1680`

**Interfaces:** none. Documentation only.

- [ ] **Step 1: Delete Decision Gate B**

In `.specs/graph-semantic-schema-overlay.agent-spec.md`, remove Decision Gate B
(line 245) entirely and replace it with a one-line note recording that it was
resolved by native n-ary relationships, pointing at
`docs/superpowers/specs/2026-07-25-native-nary-relationships-design.md`.

Delete the Global Constraint at line 101 that forbids native n-ary, and rewrite
the binary-endpoint language at lines 38, 54, 94, 126, and 145 in role terms.

Do the same for `docs/superpowers/plans/2026-07-22-graph-semantic-schema-overlay.md`
lines 21 and 1680.

- [ ] **Step 2: Repoint the archived foedus spec reference**

At `.specs/graph-semantic-schema-overlay.agent-spec.md:170` and `:276`, change
`foedus/docs/superpowers/specs/2026-07-23-turso-ontology-store-design.md` to
`foedus/docs/superpowers/specs/2026-07-25-turso-ontology-evolution-design.md`.

- [ ] **Step 3: Refresh the conformance number**

In `graph/CONFORMANCE.md`, replace **8,919 passed** with the number from the
final `mise run corpus` of Task 19, and note the `run_id` alongside it, matching
the format already used in the file.

- [ ] **Step 4: Document the role model**

In `docs/graph.md`, add a Roles section covering: a relation declares named
roles; each role has target types, optionality, and cardinality; binary is the
two-role layout named `start`/`end`; the standalone pattern
`[x:T {props}](role: player, …)`; the arrow forms as sugar; the role-edge read
sugar and its ambiguity rule; `SET [x](role: player)` replacing rather than
appending for many-valued roles; and the requirement to name a role pair for
variable-length traversal past arity 2.

- [ ] **Step 5: Verify every documented example runs**

Run each SQL/Cypher example from the new `docs/graph.md` section through
`cargo run -q --bin tursodb -- -q` against a scratch database and confirm the
output matches what the document claims. Fix the document, not the output.

- [ ] **Step 6: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime -p turso_cypher
mise run corpus
mise run cypherbench-sample
git add -A
git commit -S -m "docs/graph: document roles and close Decision Gate B

Gate B asked whether to narrow the binary-only constraint. Native n-ary
relationships answer it by deletion: there is no binary code path left to
narrow, so the gate and the constraint that framed it are removed rather
than reworded.

Also refreshes the stale conformance count and repoints the archived foedus
spec reference.

Tests: every example in the new Roles section was run against tursodb."
```

---

## Out of scope for this plan

These are the three follow-on specs named in the handoff. They are **not** tasks
here; each gets its own spec and plan, in this order:

1. **tessera / tessera-turso** — the arity-2 guard at
   `tessera-turso/src/relationship_ports.rs:19`.
2. **foedus port and WAL/projection** — `GraphOp::CreateRelationship`,
   `foedus-jd2`, and the four `skip_current_graph_port` feature files.
3. **limen** — `limen-foedus/src/lib.rs:530-540`; `RelationType.roles` is already
   declared there and currently unused.
