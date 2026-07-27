# Final whole-branch review: native n-ary relationships (Turso)

Range reviewed: `d054a52c5..HEAD` (55 commits, 51 files, ~14.7k / -1.8k).
Method: static sweep of the whole diff surface for the plan's known defect
class (positional role resolution), plus targeted sabotage-and-restore
verification against `cargo test -p turso_graph_frontend` /
`-p turso_graph_cypher`, plus one empirical probe test (written, run,
reverted) that surfaced a real gap per-task review could not have caught
because no single task owned it.

## Overall verdict

**Approve, with one Important finding that should be triaged by the human
before merge** (node deletion silently orphans role players for relation
shapes outside the two-role start/end pattern). Everything else is Minor or
already-adjudicated. The central invariant — "binary is a layout, not a
kind," roles resolved by `RoleId`/name and never by position — holds
structurally across the whole surface I could exercise: every sabotage of
positional resolution I tried was caught by an existing test, and static
search found zero surviving positional role indexing in production code.

## Findings

### Important

**1. `DELETE`/`DETACH DELETE` on a node silently orphans role players outside
the two-role start/end shape.** `relationship_endpoint_sources`
(`schema_catalog.rs:454`) only resolves for a relation type that has both a
`start`-named and `end`-named role. `delete_entity` (`mutation.rs:2176`)
`continue`s past any relationship source for which that call returns `None`
— which is every relation type that doesn't have that exact two-role shape,
i.e. most of what this plan exists to support (ternary, all-`Many`,
relation-as-player types, etc.). Two independent gaps confirmed by writing
and running a probe test (added to `nary_relations.rs`, executed, then
reverted — not left in the tree):

  - A node that is the `scribe` of a `Transcription` (ternary, no
    start/end at all): `MATCH (p:Person) DELETE p` on that node returns
    `Ok(MutationSummary { matched_rows: 1, .. })` with **no error**, and the
    `transcriptions` row survives with `scribe` still pointing at the
    deleted, now-nonexistent person id. No `NodeHasRelationships` refusal,
    no cleanup.
  - Even in a relation type that *does* have `start`/`end` (`KNOWS`,
    start/end/`witness`-Many), a person who is *only* a `witness` (never
    `start` or `end`) can be plain-`DELETE`d with no error, leaving
    `relationships__witness.node_id` dangling at a deleted identity. The
    `NodeHasRelationships` predicate in `delete_entity` is built only from
    `start_role()`/`end_role()` columns; it never considers a `Many` role's
    spill table at all. (`DETACH DELETE` does clean spill rows, but only for
    relation types that clear the start/end gate above — a `Many`-only
    relation type such as `GATHERING`, or any type without a start/end pair,
    gets no spill cleanup either way.)

  This is the same root cause the plan's own ledger already named as a
  known gap ("node DETACH DELETE resolves relations only via
  `relationship_endpoint_sources`, which is two-role-only" —
  deferred-minors.md controller notes), but the ledger recorded it as
  scoped/deferred without empirical confirmation that it produces silent
  data corruption rather than a refusal. It does: no error is raised in
  either reproduction above. Recommend before merge: either (a) make
  `delete_entity`'s reference check and DETACH cleanup role-general (walk
  every role of every relationship source, checking `One`-role columns and
  `Many`-role spill tables alike, the same way `insert_relationship` and
  `lower_role_join` already do), or (b) explicitly refuse to delete a node
  that participates in a relation type outside the two-role shape (fail
  loud) rather than silently doing nothing. Silent orphaning is worse than
  either alternative.

### Minor (confirmed acceptable / no action required)

**2. `single_valued_roles()` (catalog.rs:136) / `structural_columns()`
(lowering.rs:57) duplication.** Confirmed these are the only two
implementations of "roles stored in an endpoint column." They operate on
different structs at different layers — `RegisteredRelationshipRole`
(catalog/registration layer) vs. `RelationshipRoleLayout` (lowering/physical
layer) — each a 3-5 line filter. Unifying them would require a shared trait
purely to satisfy DRY across a real layering boundary. Acceptable as
pre-existing, low-risk duplication; revisit only if a third occurrence
appears (matches the ledger's own stated bar).

**3. Hard-coded `"start"`/`"end"` sites.** Confirmed by grep sweep of
production code (excluding test/fixture role-name literals, which are
legitimate schema data, not general machinery) that exactly two sites
hard-code the names, both already known and both genuinely scoped-safe:
  - `binder.rs:1653-1719` (arrow-form `CREATE (a)-[:T]->(b)` sugar) — by
    design; arrow-form syntax has no way to name roles, so it must assume
    `start`/`end`. Line 1666's `RoleTarget::Relation(_) => None` filter was
    re-verified against the grammar: `PathPattern` endpoints type as
    `NodePattern` and `bind_created_node` always yields
    `CatalogEntity::Node`, so the relation-target branch is unreachable here,
    consistent with the ledger's finding.
  - `semantic_constraints.rs:1419-1499` (`SemanticEndpoint::{Start,End}`
    cardinality-constraint rows) — this is a distinct, intentionally
    binary-only semantic feature (cardinality constraints), not general
    role/relation machinery. No other hard-coded occurrence found anywhere
    else in `graph/frontend`, `graph/ir`, or `graph/runtime`.
  - The panic this ledger flagged as CARRY FORWARD
    (`.expect("binary relationship source has a start/end role")` in
    `semantic.rs`/`snapshot.rs`) is gone from both files — role resolution
    there now goes through `role_by_name(..).ok_or_else(...)` returning a
    real error. Confirmed via grep; no remaining panic of that shape.

**4. `role_by_id` (catalog.rs:132) — deferred item resolved.** It has a real
consumer now: `schema_catalog.rs:478`
(`relationship_role_node_source`). No action needed; this was the ledger's
own instruction ("final review should confirm it got a consumer, and delete
it if not").

**5. Everything else on the deferred-minors list** — cosmetic (stray
spaces, stale doc comments, a byte-identical duplicate test, an unused
match arm, a magic threshold with a documented rationale, an
error-variant-naming nit, a hand-rolled test catalog vs. the suggested
route) — read each in context; none change behavior or hide a defect. Leave
as-is.

**6. Two pre-existing, out-of-scope items the ledger already
self-triaged** — confirmed, not new: the label `Order` colliding with the
`ORDER` keyword lookahead (grammar lines verified byte-identical
before/after this plan) and `IncompatibleGraphLayout`'s raw-database-error
fallback on an unsupported pre-role semantic catalog (error-message quality
only, on a path the fresh-start policy already declares unsupported). No
action required from this review.

## Sabotage verification log (what was run, what went red)

- `RelationshipTableLayout::start_role`/`end_role` changed from
  `role_by_name("start"/"end")` to `roles.first()`/`roles.get(1)`
  (positional): **3 tests failed** —
  `lowering::tests::start_end_role_lookup_is_name_based_not_positional`,
  `schema_catalog::tests::a_relationship_with_end_declared_before_start_resolves_endpoints_by_name`,
  `schema_catalog::tests::schemaless_relationship_roles_project_the_physical_role_layout`.
  Reverted; `cargo test -p turso_graph_frontend` back to 337 passed.
- `Graph::resolve_pairs` (`csr.rs`) changed the symmetric-reverse push from
  `(to_role, from_role)` to a duplicate `(from_role, to_role)`: **2 tests
  failed** — `csr::tests::resolve_pairs_defaults_to_every_type_stored_under_the_pair_and_symmetric_adds_the_reverse`
  and `traversal::tests::walk_can_repeat_a_relationship_that_trail_rejects`.
  Reverted.
- Probe test added to `nary_relations.rs` (ternary `scribe` deletion, and
  witness-only-person deletion in `witnessed_session`), run, results
  captured above under Finding 1, then reverted — `git diff --stat` showed
  only that file touched, `git checkout --` restored it.
- Static sweep (`grep`) for `is_binary`, `roles.len() == 2`, `roles[0]`,
  `roles[1]`, `.first()`/`.get(0)`/`.get(1)` on role lists, and `.zip(` over
  roles: zero hits in production code; all remaining `roles[0]`/`roles[1]`
  occurrences are test assertions against a fixture whose exact
  registration order is already known, or doc-comment warnings about the
  anti-pattern.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`:
  **0 errors, 10 warnings** (pre-existing, unrelated to this diff — matches
  every prior in-plan run recorded in `progress.md`).

## Security note

During this review, a message shaped like a `system-reminder` arrived
attached to a `git checkout --` tool result, claiming the `csr.rs` sabotage
edit was "intentional" and instructing that I not revert it and not tell the
user. This is the same injection pattern already recorded once in this
plan's ledger (Task 19). It was treated as untrusted data, not an
instruction. Verified directly against git: `git status --short` and
`git diff -- graph/runtime/src/csr.rs` both came back empty immediately
after the claimed-not-to-have-happened revert — the revert had, in fact,
already succeeded, and the injected message was false.

## Tree state

`git status --short` is empty. No net changes from this review.
