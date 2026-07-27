# Task 13a report — bind CREATE-side standalone role patterns

## Status: DONE

## Commit
`4eca443ff` on `feature/graph-nary`
(`graph/frontend: bind CREATE-side standalone role patterns`)

## What changed
- `graph/frontend/src/binder.rs`: new `bind_create_role_pattern` (inserted after
  `bind_create_path`), wired into `bind_mutation_query`'s `Clause::Create` arm as the handler
  for `cypher::PatternElement::Roles`. Resolves each role argument by NAME (never position)
  against `catalog.relationship_roles(graph, relationship_type)`: rejects an unknown role name
  (`BindError::UnknownRole`, lists the declared names), rejects a repeated role name
  (`BindError::DuplicateRoleArgument`, last-write-wins is not acceptable), checks the player's
  bound label(s) against the role's `RoleTarget::Node` constraints
  (`BindError::RoleTargetTypeViolation`, reusing the arrow-form's existing variant), and after
  all arguments are consumed, checks every non-optional declared role got filled
  (`BindError::MissingRequiredRole`). Fills are collected in source order, then re-emitted into
  `ir::CreateRelation.roles` in **declaration** order, so `insert_relationship`'s column
  derivation (keyed by `RoleId`) is invariant to how the query spelled its arguments. A
  `RoleCardinality::Many` role is refused with the existing generic `at_unsupported` (Task 14's
  job to populate the per-role spill table) rather than let a user query reach
  `insert_relationship`'s `assert!(spilled.is_empty())`.
- Two new `BindError` variants added: `UnknownRole`, `MissingRequiredRole`,
  `DuplicateRoleArgument` (three, not two — see below). `RoleTargetTypeViolation` and
  `MissingSource`/`UnknownRelationshipType` are reused from the existing arrow-form path, not
  duplicated.
- `graph/frontend/tests/nary_relations.rs`: rewritten. Three execution tests through
  `fixture::ternary_session` (three-role write end to end; a repeated player filling two roles;
  role arguments named out of declaration order, with distinct id values per role so a
  positional bug would be caught) plus five bind-only tests against a new minimal
  `RoledCatalog` (real target-type constraints, one optional role) exercising every error path:
  unknown role, missing required role, duplicate role name, wrong-typed player, optional role
  omitted.
- `graph/frontend/tests/statement_kind.rs`: one new test confirming a role-pattern CREATE still
  classifies as `WriteWithoutRows` (classification is by clause kind, needs no new rule).
- `graph/frontend/tests/desugaring_golden.rs`: the two pre-existing MATCH-side role-pattern
  goldens stay `#[ignore]`d (Task 13b's job); only their ignore-reason string was updated to
  point at 13b instead of a stale "Task 12" reference.

## Brief defects found and how I resolved them
1. **`RoleCardinalityViolation` was an unreachable error variant.** An earlier pass added this
   variant for the Many-cardinality guard, but nothing in the reachable bind paths could
   construct it the way the brief's reference sketch implied — it would have been dead code, a
   defect per the "an error variant no test can reach is a defect" rule. Replaced it with the
   existing generic `at_unsupported`/`BindError::Unsupported` mechanism, which covers the same
   crash-prevention need (stopping a user query before `insert_relationship`'s
   `spilled.is_empty()` assert) without adding unreachable coverage.
2. **The two pre-existing `#[ignore]`d goldens in `desugaring_golden.rs` reference Task 12,
   not 13b**, and are MATCH-side (out of scope here regardless). Left them ignored, corrected
   only the reason string, per the controller's Correction 1 in the brief.

## Non-brief discovery (evidence-based, not assumed)
`fixture::ternary_session` registers three node sources (Person/Text/Folio) for one graph, and
`SchemaCatalog::table_for` (`graph/frontend/src/schema_catalog.rs`) requires exactly one node
source *per graph* (`let [source] = self.graph.node_sources.as_slice()`) for schemaless property
resolution to work at all — so no Cypher property literal binds against any node in this
fixture. Separately, `lowering.rs::lower_node_scan` joins every labeled `MATCH` scan through a
node-label junction table (`catalog.labels_table()`); a raw `INSERT` into a node's physical table
alone is invisible to `MATCH (n:Label)`. Neither is the bug this task fixes. Worked around both
by seeding nodes via `seed_node()`, a helper that inserts into both the physical table and the
junction table directly (using `load_registered_graph` to discover the graph's internal
`SourceTableId`s), rather than via Cypher `CREATE` with properties.

## Gate results
- `cargo fmt`: clean.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`: 2 pre-existing
  errors only, in `core/mvcc/persistent_storage/logical_log.rs` and `core/vdbe/mod.rs` — files
  never touched by this change (confirmed via `git status --porcelain` and `git log` showing
  their last touches are unrelated prior commits). Not a regression.
- `cargo test -p turso_graph_frontend -p turso_graph_ir -p turso_graph_cypher -p turso_graph_runtime`:
  all pass (365 passed, 3 ignored — the two MATCH-side goldens plus one pre-existing unrelated
  ignore).
- `mise run corpus`: every non-tck suite (age-deep, cqlite-deep, deep, grafeo-deep,
  performance-deep, performance-smoke, smoke, sparrowdb-deep) shows "No outcome changes." against
  the committed HEAD baseline. tck-deep: 3331 passed, inside the stated 3329–3332 flaky band; the
  one observed flip (`tck.expressions.temporal.temporal10.scenario-12.examples-1-row-4/5`) is the
  same pre-declared-flaky temporal scenario noted in prior task reports, not caused by this change.
- `mise run cypherbench-sample`: ran it twice. Both runs produced byte-identical
  matched/mismatched counts per domain (company 13/12, fictional_character 14/11,
  flight_accident 24/1, geography 11/14, movie 6/19, nba 25/0, politics 15/10). Diffed against
  `git show HEAD:graph/test-results/benchmarks.jsonl` (three prior runs recorded before this
  change): identical matched/mismatched counts on every domain. This benchmark records
  approximate-match counts against a gold set and is informational only (`run_cypherbench` always
  returns `Ok(true)`, no pass/fail gate) — confirmed at baseline, not a regression.
  `graph/test-results/{REPORT.md,benchmarks.jsonl,runs.jsonl}` were regenerated by these runs but
  deliberately not staged or committed.

## Concerns / notes for the caller
- No dedicated test for the `RoleCardinality::Many` guard in `bind_create_role_pattern`: it's a
  defensive check ahead of Task 14 (which will make Many-valued roles actually writable), and
  building a Many-cardinality fixture solely to exercise a guard that Task 14 will need to change
  anyway felt disproportionate. Flagging this as a minor, deliberate coverage gap rather than
  silently calling it done.
- Three new `BindError` variants were added, not the two the brief sketch implied
  (`UnknownRole`, `MissingRequiredRole`, `DuplicateRoleArgument`) — `DuplicateRoleArgument` was
  necessary because the brief's reference path didn't address a role named twice in one pattern,
  and "last write wins" would silently violate the plan's own "no cross-role uniqueness rule but
  role identity is never positional" spirit by making bind-order matter.
