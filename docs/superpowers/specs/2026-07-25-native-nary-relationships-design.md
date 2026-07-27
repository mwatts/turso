# Native N-ary Relationships (Turso graph frontend) — Design

Date: 2026-07-25
Branch: `feature/graph-nary`, cut from `feature/graph-frontend`
Status: approved design, ready for an implementation plan

## Why

Relationships in the graph frontend are binary everywhere: `CreateRelationship`
and `FixedExpand` carry `from`/`to`, relationship layouts carry start/end
columns, and source registration declares exactly two endpoint columns.
Anything with three participants — a transcription linking a scribe, a text,
and a folio — has to be reified by the caller into a node plus one binary edge
per role.

`.specs/graph-semantic-schema-overlay.agent-spec.md` deferred native n-ary
storage behind Decision Gate B, on the reasoning that reification composes from
binary primitives and that native storage only collapses player-to-player
traversal from two indexed hops to one. That deferral is now withdrawn: n-ary
relationships are a product requirement, top to bottom, and reification is not
an acceptable representation at any layer of the stack.

Consumers make the cost of the deferral concrete. `tessera-turso` already plans
reified DDL for any relation that is not exactly two required roles, but its
write and read paths refuse anything other than arity two
(`crates/tessera-turso/src/store/relationship_ports.rs` guards
`relation.roles.len() != 2` and indexes `roles[0]`/`roles[1]`). Foedus's
`GraphOp::CreateRelationship` carries only `source_id`/`target_id`, so declared
roles are dropped at the port; `foedus-jd2` tracks this, and four corpus
feature files are tagged `skip_current_graph_port` because of it. Limen
declares `RelationType { name, roles: Vec<Role> }` in its ontology but flattens
to `source_id`/`target_id` when it writes.

## Scope

This spec covers the Turso graph frontend only: IR, catalog, parser, binder,
lowering, runtime, and storage layout.

Three further specs follow, in dependency order, each with its own plan and
implementation cycle:

1. Tessera schema and `tessera-turso` store — N-role writes and reads.
2. Foedus `foedus-core` port, WAL, and projection — roles across the wire.
3. Limen — role-carrying writes, reads, filters, and DTOs.

Out of scope here: role interfaces or role polymorphism across relation types;
inference and rules (Decision Gate C stays deferred); constraints beyond role
target types and cardinality.

## Decisions

| Decision | Choice |
| --- | --- |
| Storage layout | One indexed endpoint column per single-valued role; `Many` roles spill to a per-role child table |
| Binary relationships | Not a separate kind. Deleted as a code path; two-role relations are a layout of the one role model |
| Surface syntax | Standalone `[var:Type {props}](role: player, …)` pattern, plus role-edge read sugar |
| Existing on-disk graphs | Fresh start. No migration, no dual-read; incompatible catalogs fail loudly at open |
| Role semantics in v1 | Required, optional, repeated players, cardinality > 1, relation-as-player, role updates after create |

## Model

### Roles are local to their relation type

A new non-zero `u32` identity, `RoleId`, joins `LabelId`,
`RelationshipTypeId`, and `PropertyId` in `graph/ir/src/identity.rs`. Roles are
named per relation type. There are no global role interfaces: Milestone 3's
fragment-interface polymorphism was dropped from the semantic overlay, and
nothing here reintroduces it.

A relationship type carries its roles in the catalog:

```rust
pub struct RoleDef {
    pub role: RoleId,
    pub name: String,
    /// What a player of this role may be. Empty means unconstrained.
    pub target_types: Vec<RoleTarget>,
    pub optional: bool,
    pub cardinality: RoleCardinality, // One | Many
}

/// A role player is either a node of some label or a relation of some type;
/// the two identity spaces stay distinct rather than being flattened.
pub enum RoleTarget {
    Node(LabelId),
    Relation(RelationshipTypeId),
}
```

Semantic mode persists roles in a new `graph_semantic_role` catalog table,
registered and validated alongside the existing semantic node and relationship
type rows. Schemaless mode synthesizes two roles, `start` and `end`, both
required and both `One`, from the endpoint columns named at source
registration. Endpoint validation generalizes from the current start/end check
to a per-role `target_types` check.

### The graph frontend overlays user tables

Registration points the frontend at tables the user already owns and names
their endpoint columns. "Delete the binary path" therefore means deleting the
special-cased binary code path, not the ability to register a two-endpoint
table. Every donor corpus source registers as a two-role relation named
`start`/`end` and keeps working unchanged. Fresh-start-no-migration applies to
graph-owned catalog tables and to relations the frontend itself creates in
semantic mode.

### IR

`Direction` stops being a field on plan nodes and becomes an ordered role pair.
It survives only as a parser-level desugaring rule and does not exist below the
AST boundary.

- `FixedExpand` becomes `RoleExpand`:

  ```rust
  pub struct RoleExpand {
      pub input: Box<Plan>,
      pub relation_source: SourceTableId,
      pub target_node_source: SourceTableId,
      pub from: BindingId,
      pub from_role: RoleId,
      pub to_role: RoleId,
      pub relation: Binding,
      pub to: Binding,
      pub relationship_types: Vec<RelationshipTypeId>,
      pub bound_target: Option<BindingId>,
  }
  ```

  Today's outgoing hop is `from_role: start, to_role: end`; the incoming hop
  swaps the pair. `bound_target` and its composite-index fold are unchanged in
  meaning.

- `GraphExpand` takes the same ordered role pair for variable-length hops and
  keeps `min_hops`, `max_hops`, `unbounded`, `uniqueness`, and its path
  outputs.

- `CreateRelationship` becomes `CreateRelation`:

  ```rust
  pub struct CreateRelation {
      pub binding: Binding,
      pub source: SourceTableId,
      pub relationship_types: Vec<RelationshipTypeId>,
      pub roles: Vec<RoleBinding>, // { role: RoleId, value: BindingId }
      pub properties: Vec<PropertyValue>,
  }
  ```

  `MergeRelationship` wraps `CreateRelation` exactly as it wraps
  `CreateRelationship` today.

### Storage

Per relation type: an identity column, payload columns, and one endpoint column
per `One` role, each indexed. A role declared `Many` spills to
`<relation>__<role>(relation_id, node_id)`, indexed in both directions.
Composite indexes stay where the binary layout had them, keyed by the role pair
the planner selects rather than by start/end.

A two-role, all-required, all-`One` relation lands on exactly the physical
shape it has today: two indexed endpoint columns on one table.

### Runtime

The CSR builder loses its start/end arrays and builds adjacency for the
`(from_role, to_role)` pair under traversal. Path values keep alternating node
and relation elements; each element records the role entered and the role
exited, so a path over an n-ary relation is unambiguous when read back.

## Surface syntax

### Standalone role pattern

Two pest rules generalize Cypher's `[var:Type {props}]` so it can stand alone
with a role list. `[` never begins a pattern element in Cypher today, so the
standalone form introduces no grammatical ambiguity, and property maps keep
their usual position.

```cypher
CREATE [t:app.Transcription {status: 'done'}](scribe: $s, text: $x, folio: $f)
MATCH  [t:app.Transcription](scribe: s, text: x, folio: f) RETURN t.status, s.name
MATCH  [t:app.Transcription](scribe: s) DELETE t
```

Roles omitted from a `CREATE` must be declared `optional`. Roles omitted from a
`MATCH` are unconstrained — omission does not assert that the role is empty.

### Binary as sugar

The parser desugars the arrow forms before the AST reaches the binder:

- `(a)-[r:KNOWS]->(b)` ≡ `[r:KNOWS](start: a, end: b)`
- `(a)<-[r:KNOWS]-(b)` ≡ `[r:KNOWS](start: b, end: a)`
- the undirected form binds the pair in both orders, as it does today.

### Role-edge read sugar

`(t:Transcription)-[:scribe]->(s)` reads a role when `t` binds a relation and
`scribe` resolves to a role of its type. This is resolved in the binder, not
the grammar, because it depends on what `t` is bound to.

Relationship type names and role names occupy different namespaces. When a name
resolves as both a relationship type and a role of the bound relation, the
binder raises `AmbiguousRoleName` and names both candidates rather than
choosing. Schemaless donor graphs expose only `start` and `end` roles, so no
donor query can reach this error.

Role-edge sugar is read-only. Writes use the standalone role pattern, which is
what makes multi-role creation atomic.

## Binding, lowering, planning

**Binder.** Resolves role names to `RoleId` against the catalog and rejects:
unknown roles; required roles missing from a create; players whose labels fall
outside the role's `target_types`; a `One` role bound to a list. A `Many` role
accepts one player or many. The same entity may fill two roles of one relation:
no uniqueness is assumed across roles, and nothing in binding or lowering
depends on players being distinct. Relation-as-player needs no special case — a
role's `target_types` may carry `RoleTarget::Relation`, and a relation variable
binds into the role slot like any other player. The binder's existing read/write statement
classifier learns the standalone bracket form, so read-only connections
continue to refuse writes.

**Lowering.** One program per create: an `INSERT` into the relation table
setting every `One` role column, plus batched inserts into each `Many` spill
table, all inside the existing mutation savepoint. A partially built n-ary
relation is therefore never observable — the integrity property that reified
modeling cannot provide, because reification needs one statement per role.

`RoleExpand` lowers to a join on the `from_role` column's index projecting
`to_role`. A hop through a `Many` role goes through that role's spill table.
`MERGE` matches on the full set of bound required roles.

**Planner.** Index selection keys off the role pair instead of start/end. The
covering composite-endpoint-index win and the `bound_target` cycle fold both
carry over as "composite over (from_role, to_role)."

## Errors

New variants, raised at bind time unless noted:

| Variant | Raised when |
| --- | --- |
| `UnknownRole { relation_type, role }` | A role name is not declared on the relation type |
| `MissingRequiredRole { relation_type, role }` | A create omits a non-optional role |
| `RoleTargetTypeViolation { role, expected, actual }` | A player's labels fall outside the role's target types |
| `RoleCardinalityViolation { role }` | A `One` role receives a list |
| `AmbiguousRoleName { name, relation_type }` | A name resolves as both a relationship type and a role |
| `IncompatibleGraphLayout { detail }` | A graph catalog predating roles is opened |

Values that arrive as parameters or as `Any` are validated at execution time
*before* any physical write, reusing the pattern the semantic overlay
established for dynamic mutation values. `IncompatibleGraphLayout` is raised at
open and names both the incompatibility and the fresh-start policy; there is no
legacy reader.

## Path policy

A relation with k roles exposes k·(k−1) directed role pairs, so "shortest path
over this relation type" stops being well defined once k > 2.

Rule: variable-length and shortest-path traversal over a relation type with
more than two roles requires an explicit role pair. Unconstrained use is a
legality-table error in `graph/runtime/src/path_policy.rs`, never a silently
chosen pair. Relation types with exactly two roles keep today's behavior
unchanged.

## Conformance and documentation

- `semantics_version` moves v2 → v3 and the semantic profile digest is
  re-pinned.
- Decision Gate B is deleted from
  `.specs/graph-semantic-schema-overlay.agent-spec.md`, together with the
  no-native-n-ary line in its Global Constraints (line 101 region) and the
  matching constraint in
  `docs/superpowers/plans/2026-07-22-graph-semantic-schema-overlay.md`.
- That spec's pointer to foedus's archived
  `2026-07-23-turso-ontology-store-design.md` is repointed at the live
  `2026-07-25-turso-ontology-evolution-design.md`.
- `docs/graph.md` and `graph/CONFORMANCE.md` gain the role model, and
  `CONFORMANCE.md`'s pass counts are refreshed from the latest recorded run.

## Testing

Test-driven throughout: each step writes a failing test, verifies it fails for
the intended reason, then makes it pass.

New `graph/frontend/tests/nary_relations.rs`:

- three required roles round-trip through create, read, update, and delete;
- an optional role absent, then present;
- a `Many` role with several players;
- the same entity bound to two roles of one relation;
- a relation bound as a player of another relation;
- role participants updated after create;
- create atomicity: an injected failure partway through a three-role create
  leaves no relation row and no role rows;
- one test per new error variant, asserting the message names the offending
  role.

Runtime tests cover role-pair adjacency construction and every new
`path_policy` legality row. Desugaring gets golden tests proving
`(a)-[r:KNOWS]->(b)` and `[r:KNOWS](start: a, end: b)` lower to byte-identical
IR.

## Merge gates

- `mise run corpus`: at least 8,926 passed, no new failure family.
- `mise run cypherbench-sample`: parity with the recorded baseline. A two-role
  hop is the same physical shape as today's binary hop, so a regression here
  means the rewrite is wrong, not that n-ary costs more.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`.
- `cargo fmt --check`.
- Graph crate tests and `turso_core` lib tests.

## Risks

Deleting the binary path rewrites the hottest and best-tested part of the graph
runtime: `FixedExpand`, the relationship layouts, the CSR builder, mutation
lowering, and index selection. The corpus and cypherbench gates are the control
for this, applied per task rather than once at the end. The mitigating fact is
that a two-role relation keeps its exact physical shape, so the corpus should
exercise the same storage and index paths it does today; any corpus or
benchmark movement is a defect signal, not an expected cost of the feature.
