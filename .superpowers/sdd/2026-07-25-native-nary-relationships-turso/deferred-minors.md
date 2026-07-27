# Deferred Minor findings — plan: native n-ary relationships (Turso)

Each line is 'progress.md line number: finding', extracted from the ledger.
Read the surrounding ledger context for any you want to act on.

CONTROLLER NOTES on this list:
- The Task 12 `compiler.rs` / `query_needs_traversal_snapshot` item was RETRACTED
  later as a controller error. Do not chase it.
- Task 13a's 'no test for the Many-role guard. Task 14 must add one' was CLOSED
  by Task 16's fix round, which added coverage for that refusal.
- Two items name the same underlying duplication and should be triaged together:
  `single_valued_roles()` (catalog.rs:136) and `structural_columns()`
  (lowering.rs:57) are two implementations of one predicate.
- Known deferred items NOT in this extract because they were recorded without the
  'minor (deferred)' marker: `semantic_constraints.rs` hard-codes "start"/"end";
  the arrow-form endpoint check at binder.rs:1666 does too (unreachable for relation
  players, verified against the grammar); Task 3's IncompatibleGraphLayout gap; node
  DETACH DELETE resolves relations only via `relationship_endpoint_sources`, which is
  two-role-only; and the label `Order` can never parse.

49:  - minor (deferred): `role_by_id` (catalog.rs:132) is unused public API.
65:  - minor (deferred): test asserts `message.contains("no migration")` -- couples
128:  - minor (deferred): Step 2's red state was REASONED, not run. Every dispatch
130:  - minor (deferred): per-branch RelationshipTableLayout clone at bind time.
160:  - minor (deferred): stray literal spaces before OR in the symmetric arm,
410:Task 9: minor (deferred): the `graph` parameter added to
601:- Task 11: minor (deferred): the adapted role assertion in
606:- Task 11: minor (deferred): the Task 9 sibling test's doc comment still claims the other test
665:- Task 12: minor (deferred): `compiler.rs`'s `query_needs_traversal_snapshot` answers `false`
669:- Task 12: minor (deferred): `bind_staged_match` (binder.rs:1256) clones the whole `Pattern`
672:- Task 12: minor (deferred): `rename_match_clause`'s "Roles is unreachable here" holds only
752:- Task 13a: minor (deferred): the semantic-mode fixture is a hand-rolled `GraphCatalogSnapshot`
756:- Task 13a: minor (deferred): no test for the Many-role guard. Task 14 must add one.
873:- Task 14: minor (deferred): the `if created { .. }` guard that skips spill
937:Task 15: minor (deferred): the null-player refusal reuses
1146:Task 17: minor (deferred): the early-exit threshold 135 is an empirical
1696:Task 19: minor (deferred): the `None => false` arm of `match binding.map(|e|
