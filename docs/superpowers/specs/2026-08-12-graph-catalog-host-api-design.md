# Graph catalog host API

Date: 2026-08-12
Branch: `feature/graph-frontend`
Status: draft for implementation
Issues: `turso-graph-catalog-host-api-gv7` (epic),
`turso-extend-graph-registration-7yx`,
`turso-replace-semantic-overlay-oje`,
`turso-graph-catalog-host-guide-pfz`
Consumers: foedus `tessera-turso` (only writer of catalog SQL today); limen (embeds foedus, must not grow a second copy)

## 1. Problem

`register_graph` and `register_semantic_schema_with_fragments` cover first
install. They do not cover the two host operations foedus needs after that:

1. **Add sources to an already-registered graph** while keeping existing
   source ids. `register_graph` returns `CatalogError::GraphAlreadyExists`.
2. **Replace a semantic overlay** when the stored overlay is not an exact
   replay of the requested one. `register_semantic_schema_with_fragments`
   returns `SemanticCatalogError::ConflictingSchema`.

Foedus therefore writes Turso catalog tables itself, in
`crates/tessera-turso/src/catalog.rs` (foedus `6c8131a`, Turso pin
`80810f94`, `GRAPH_CATALOG_VERSION` 7):

- `INSERT` into `__tdb_int_g_src`, `__tdb_int_g_nsrc`, `__tdb_int_g_rsrc`,
  `__tdb_int_g_roles`, and the per-graph relationship-type registry.
- `CREATE INDEX` names of the form `__tdb_int_g_ep_{graph}_{role}_{hash}`
  and `__tdb_int_g_ep_{graph}_pair_{hash}`.
- `DELETE FROM` eleven semantic tables (`__tdb_int_g_styp` … `__tdb_int_g_sccard`)
  on `ConflictingSchema`, then re-register.
- Tests `SELECT` leftover generation triggers by the reserved prefixes
  `__tdb_int_g_gen_%` / `__turso_internal_graph_gen_%`, and one resume
  test `DELETE`s `__tdb_int_g_sprop` through `prepare_internal`.

Those names are `pub(crate)` in this crate. A host that copies them is
broken on the next catalog rename (this happened at version 7). Limen does
not write the names today; it will if anyone pastes the foedus workaround
into `limen-foedus`.

Foedus already named the missing API as v2 in
`docs/superpowers/specs/2026-07-25-turso-ontology-evolution-design.md`
(§ Graph & Semantic Registration). This document is that API.

## 2. Intended end state

After this work and the consumer migrations:

- A host registers, extends, inspects, and replaces graph catalog state
  only through `turso_graph_frontend` functions exported from `lib.rs`.
- No host crate contains the strings `__tdb_int_g_` or
  `__turso_internal_graph_` in Rust or TOML (docs and lockfiles excluded).
- No host calls `Connection::prepare_internal` to mutate catalog tables.
- `GRAPH_CATALOG_VERSION` stays **7**. This is a code API. It does not
  change on-disk catalog layout.
- Existing source ids stay stable across additive extend. Existing entity
  rows stay readable after a semantic replace.
- First-install functions stay fail-closed: `register_graph` still returns
  `GraphAlreadyExists`; `register_semantic_schema_with_fragments` still
  returns `ConflictingSchema`.

Acceptance: foedus can delete `GRAPH_SOURCES_TABLE`,
`SEMANTIC_CATALOG_TABLES`, `execute_internal`, `add_node_source`,
`add_relationship_source`, `install_role_index`,
`install_role_pair_indexes`, and `clear_semantic_catalog`. Limen bumps
`driver` after that delete and adds a deny-grep so the strings cannot
return.

## 3. Design

Two new functions. They take the same registration structs the host already
builds for first install. They own every catalog write, endpoint index, and
`schema_generation` bump that foedus currently copies.

### 3.1 `extend_graph_registration`

```rust
pub fn extend_graph_registration(
    connection: &Arc<Connection>,
    registration: &GraphRegistration,
    polymorphic_roles: &[PolymorphicRoleRegistration],
) -> Result<RegisteredGraph, CatalogError>
```

Re-export from `lib.rs` next to `register_graph`.

**Precondition.** Graph `registration.name` already exists. Otherwise
`CatalogError::GraphNotFound`.

**Input.** The *desired complete* registration: every node source and
relationship source the host wants after the call, including sources that
already exist. Same shape as `register_graph`.

**Per existing source (matched by `name`):**

- Table, identity column, and role shape equal the stored row → keep the
  stored `SourceTableId`. Do not rewrite the row.
- Any of those fields differs → `CatalogError::SourceConflict` (new
  variant, below). Do not mutate any source.

**Per new source (name absent from the stored graph):**

- Allocate the next source id with the same generator `register_graph`
  uses.
- Insert source, node/relationship row, and roles.
- For each `RoleCardinality::One` role, create the same endpoint index
  `register_graph` would create (including polymorphic
  `(discriminator, endpoint)` and pair indexes).
- Insert the relationship name into the per-graph type registry with the
  next registry id.

**Removals.** A stored source whose name is not in `registration` is
`CatalogError::SourceRemoved`. The function does not drop sources, label
membership, or endpoint indexes.

**Idempotence.** A second call with the same `registration` returns
`Ok(RegisteredGraph)` equal to the first return (same source ids) and
writes no catalog rows.

**Generation.** If the call inserted at least one source, bump the graph
generation the same way a semantic insert does today, so a long-lived
`GraphConnection` reloads `SchemaCatalog`. If the call was a no-op, do not
bump.

**Transaction.** One savepoint, same pattern as
`register_graph_with_options` (`turso_graph_register` or a sibling name
`turso_graph_extend`). Failure rolls back every insert and index from this
call.

**Validation.** Reuse `validate_registration_names` and the physical
table/column/identity checks `register_graph` already runs, but only for
*new* sources. Existing sources are not re-validated against `sqlite_schema`
beyond the stored catalog row.

### 3.2 `replace_semantic_overlay`

```rust
pub fn replace_semantic_overlay(
    connection: &Arc<Connection>,
    graph_name: &str,
    schema: &SemanticSchemaRegistration,
    fragments: &SemanticFragmentRegistration,
    constraints: &SemanticConstraintRegistration,
) -> Result<SemanticReplaceOutcome, SemanticCatalogError>

pub enum SemanticReplaceOutcome {
    Unchanged,
    Replaced {
        previous_generation: u64,
        generation: u64,
    },
}
```

Re-export from `lib.rs` next to `register_semantic_schema_with_fragments`.

**Precondition.** Graph exists. Otherwise
`SemanticCatalogError::GraphNotFound`. Physical sources in `schema` /
`fragments` must still validate against the *current* registered graph
(`validate_against_graph`). Source ids do not change.

**Unchanged.** If the stored overlay (types, properties, fragments, roles,
ownership, and constraints) is an exact replay of the arguments, return
`Unchanged` and write nothing.

**Replaced.** In one savepoint:

1. Delete every semantic-schema and semantic-constraint row for this
   `graph_id` (the eleven tables foedus lists in `SEMANTIC_CATALOG_TABLES`,
   plus any constraint table `register_semantic_constraints` uses that is
   not already in that list).
2. Insert the requested schema, fragments, and constraints using the same
   writers as `register_semantic_schema_with_fragments` and
   `register_semantic_constraints`.
3. Validate every newly requested constraint against visible graph data
   before the savepoint releases, same as `register_semantic_constraints`.
4. Bump `schema_generation`. Return `Replaced` with the old and new
   values.

**What it must not do.** Drop or rewrite graph sources, label membership
tables, relationship-type registry rows, or user tables.

### 3.3 Errors

Add to `CatalogError` (keep existing variants):

```rust
#[error("graph `{graph}` source `{source}` is registered with a different shape: {reason}")]
SourceConflict {
    graph: String,
    source: String,
    reason: String,
},
#[error("graph `{graph}` refuses to drop registered source `{source}`")]
SourceRemoved { graph: String, source: String },
```

`reason` names the mismatched field (`table`, `identity_column`, `roles`).
Do not put SQL or internal table names in `reason`.

`SemanticCatalogError::ConflictingSchema` stays on the exact-replay
register functions only. `replace_semantic_overlay` does not return it.

### 3.4 What stays as-is

| Function | Role after this spec |
| --- | --- |
| `register_graph` / `register_graph_with_polymorphic_roles` / `register_graph_with_options` | First install only. Still `GraphAlreadyExists`. |
| `load_registered_graph` | Read-back. Hosts keep using it. |
| `GraphConnection::inspect_schema` | Operator inspection. |
| `register_semantic_schema_with_fragments` | First semantic install. Still `ConflictingSchema`. |
| `register_semantic_constraints` | Append-only constraints on an already-matching schema. |
| `create_fts_index` / `list_fts_indexes` / `drop_fts_index` | Unchanged. |
| `labels_table_name` and sibling junction helpers | Remain for lowering. Hosts must not use them to `INSERT` catalog rows. |
| `GRAPH_CATALOG_VERSION` | Stays 7. |
| `GraphRegisterOptions` | Stays empty. Do not add an `Extend` flag that changes `register_graph`. |

`register_graph` stays fail-closed so a mistaken second create cannot
silently become an extend.

### 3.5 Host contract

A crate that depends on `turso_graph_frontend` or `turso_core` and is not
this repository:

- Must not mention `__tdb_int_g_` or `__turso_internal_graph_` in Rust or
  TOML.
- Must not call `prepare_internal` on catalog objects.
- Must treat `RegisteredGraph` source ids as the only durable identifiers
  for membership and endpoints.

## 4. Requirements

1. `extend_graph_registration` on a missing graph returns `GraphNotFound`
   and writes nothing.
2. `extend_graph_registration` on a new graph name is not a create path;
   hosts still call `register_graph` for first install.
3. Adding node source `B` to a graph that already has node source `A`
   leaves `A.id` unchanged. A follow-up `load_registered_graph` shows both,
   with `B.id != A.id`.
4. Adding a relationship whose role names a newly added node source
   succeeds in the same call (the function must assign node ids before
   writing roles).
5. A second `extend_graph_registration` with an identical registration is
   a no-op: same returned ids, no generation bump.
6. Changing an existing source's table, identity column, or role shape
   returns `SourceConflict` and leaves every catalog row unchanged.
7. Omitting a stored source from `registration` returns `SourceRemoved`
   and leaves every catalog row unchanged.
8. New `RoleCardinality::One` roles receive the same endpoint indexes
   `register_graph` would create. The host does not pass index names.
9. A generation bump happens if and only if at least one source row was
   inserted.
10. `replace_semantic_overlay` on a missing graph returns
    `GraphNotFound` and writes nothing.
11. `replace_semantic_overlay` with an exact replay of the stored overlay
    returns `Unchanged` and does not bump generation.
12. `replace_semantic_overlay` with a schema that adds a type, after
    entities of the old types exist, returns `Replaced`, keeps those
    entity rows, and still enforces old and new constraints.
13. `replace_semantic_overlay` that fails constraint validation against
    existing rows rolls back: stored overlay and generation unchanged.
14. Neither function requires a `GRAPH_CATALOG_VERSION` bump. Opening a
    version-7 catalog with the new functions succeeds.
15. `cargo test -p turso_graph_frontend` covers requirements 1–13 with
    on-disk and in-memory connections. Tests that need a divergent overlay
    live in this crate; they do not export a host-facing “delete semantic
    row” helper.

## 5. Failure modes

| Case | Result | Durable effect |
| --- | --- | --- |
| Extend, graph missing | `GraphNotFound` | None |
| Extend, source shape drift | `SourceConflict` | None |
| Extend, host dropped a source | `SourceRemoved` | None |
| Extend, new source table missing | existing `SourceTableMissing` | None |
| Extend, mid-function IO error | savepoint rollback | None |
| Replace, graph missing | `GraphNotFound` | None |
| Replace, schema does not match physical sources | existing semantic validation error | None |
| Replace, new constraint fails on existing rows | existing constraint error | Overlay and generation unchanged |
| First install called twice | `GraphAlreadyExists` / `ConflictingSchema` | Unchanged. Host must switch to extend / replace. |

## 6. Non-goals and rejected alternatives

- **No on-disk catalog migration.** Version 7 names stay. The bug is the
  host writing them, not the names themselves.
- **No `GraphRegisterOptions { extend: true }`.** A flag on
  `register_graph` would make “create” mean “maybe mutate”. Separate
  functions keep the first-install contract.
- **No incremental semantic patch API** (`add_type`, `add_property`).
  Foedus already rebuilds the overlay as a whole. Incremental register is
  a later spec.
- **No source rename or source drop.** Those are breaking ontology changes
  and stay host-side migration work.
- **No public table-name constants.** Exporting `__tdb_int_g_src` as
  `pub const` would freeze the leak.
- **No host test hook that `DELETE`s catalog rows.** Divergence tests
  belong in this crate.
- **Not a limen or foedus feature.** Those repos only delete their copies
  of the SQL after this crate ships the functions.

## 7. Tests and PR order

### PR 1 — `extend_graph_registration`

Files: `graph/frontend/src/catalog.rs`, `graph/frontend/src/lib.rs`,
tests under `graph/frontend`.

- Create graph with node `Person`. Extend with node `Team` and relation
  `Member`. Assert `Person` source id unchanged; `Team` and `Member` appear;
  generation moved once.
- Repeat the same extend. Assert `Unchanged` behavior (same ids, generation
  unchanged).
- Extend that changes `Person.table` → `SourceConflict`; row counts on
  catalog tables unchanged (count via this crate’s internal helpers, not a
  public name).
- Extend that omits `Person` → `SourceRemoved`.
- Extend on an unknown graph name → `GraphNotFound`.
- New `One` role has an endpoint index; a second extend does not create a
  second index.

### PR 2 — `replace_semantic_overlay`

Files: `graph/frontend/src/semantic.rs`,
`graph/frontend/src/semantic_constraints.rs`, `graph/frontend/src/lib.rs`.

- Register schema `{Person}`; insert a person row; replace with
  `{Person, Team}`. Person row still reads; both types present; generation
  moved.
- Replace with the same overlay → `Unchanged`.
- Replace that adds a required constraint the existing row fails → error,
  overlay unchanged.
- `register_semantic_schema_with_fragments` after a non-identical stored
  overlay still returns `ConflictingSchema` (exact-replay contract
  preserved).

### PR 3 — host guide

Files: `docs/graph.md` (new subsection “Evolving a registered graph”),
this spec set to implemented.

Do not edit foedus or limen in the Turso PRs.

### Consumer order (other repos)

1. Foedus switches `ensure_graph_registration` to
   `extend_graph_registration` and `ensure_semantic_registration` conflict
   recovery to `replace_semantic_overlay`. Then it deletes the string
   tables and `execute_internal` catalog writes. Then it rewrites the
   resume test that `DELETE`s `__tdb_int_g_sprop`.
2. Limen bumps `driver` to that foedus revision, removes catalog-name
   comments from `Cargo.toml`, and adds a workspace deny-grep.

## 8. Open questions

None that block implementation. Incremental semantic register, source
rename, and source drop stay out of this spec.
