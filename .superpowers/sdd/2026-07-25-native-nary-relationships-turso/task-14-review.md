# Task 14a review — write and delete many-valued role players

Commit reviewed: `a35a14c05` (`754dce74d..a35a14c05`)

## Method

Read the brief (Controller Corrections as governing text), the report, and
the full diff. Traced every `role.column` / `role.spill_table` usage across
`mutation.rs`, `lowering.rs`, and `catalog.rs` to check the empty-string
`column` on `Many` roles never reaches `quoted_identifier`. Ran the targeted
`nary_relations` test file (12 tests, all passing at baseline) and then
sabotaged the implementation six ways, one at a time, restoring the file from
a saved copy after each. Confirmed `cargo fmt -p turso_graph_frontend --check`
is clean on the restored tree. Did not re-run corpus/cypherbench or the full
crate suite, per instruction.

## Spec compliance (against the brief's corrected 14a scope): ✅

1. **Scope discipline (Correction A).** ✅ Step 5 (`role_join_expression`,
   the JOIN-vs-scalar-subquery hop) and
   `a_hop_through_a_many_valued_role_returns_every_player` are absent, exactly
   as required. No `lowering.rs` changes appear in the diff at all.
2. **`witnessed_session` fixture (Corrections B/C).** ✅. Binary + `Many`
   exactly as specified: `start`/`end` (`One`, columns `src`/`dst`) plus
   `witness` (`Many`, empty column), one `Person` node source, one `KNOWS`
   relationship source over `relationships(id, src, dst)`. Test bodies use
   the real fixture idioms (`session.execute`, `Parameters::new()`,
   `second_connection(&database).prepare(...).run_collect_rows()`), not the
   brief's pseudocode helpers. Confirmed by test run that binding an arrow
   pattern (`MATCH (a:Person)-[r:KNOWS]->(b:Person)`) over this three-role
   source works for both delete tests — no `roles.len() == 2` assertion
   blocks it.
3. **Step 3 — spill-row INSERT (Correction D).** ✅. Free function, not a
   method; `relation_id` comes from `insert_entity`'s returned `(identity,
   created)`; no invented `MutationError::NonIntegerPlayer` — the player
   `Value` is bound as a parameter via `run_ignore` + an internal
   `HashMap<String, Value>`, exactly `delete_entity`'s pattern; the
   `spilled.is_empty()` assertion is deleted; `quoted_identifier` (not
   `quote_identifier`) is used for the spill table name.
4. **Step 4 — both delete paths (Correction E).** ✅. The relationship-delete
   `else` branch and the node `DETACH DELETE` branch (via a
   `relation_id IN (SELECT ... WHERE <predicate>)` subquery, ordered before
   the bare relation-row delete) both purge spill rows. Verified as two
   genuinely independent code paths by sabotage (see below) — removing one
   spill-delete does not affect the other's test.
5. **Step 6 — narrowed duplicate check (Correction F).** ✅. No
   `RoleCardinalityViolation` variant reintroduced. `DuplicateRoleArgument`
   still fires for a repeated `One` role; a repeated `Many` role now passes
   the `seen`-set check without erroring.
6. **Step G — guard removal.** ✅. The `at_unsupported("creating a
   many-valued role in a role pattern")` block is deleted from
   `bind_create_role_pattern`.
7. **Correction H — duplicate predicates left alone.** ✅. Neither
   `single_valued_roles()` (`catalog.rs:136`) nor `structural_columns()`
   (`lowering.rs:57`) appears in the diff; report explicitly notes both were
   read, not touched.
8. **Correction I — gate/commit hygiene.** ✅. Commit subject is exactly
   `graph/frontend: write and delete many-valued role players`; body has no
   hop paragraph; `git show --stat` lists exactly the 4 intended files
   (no `graph/test-results/*` in this commit); per-suite corpus table in the
   report matches the required format (no single total asserted as a gate).
9. **Global constraint — binary is a layout.** ✅. No `roles.len() == 2`,
   no `is_binary`, no hard-coded `"start"`/`"end"` added to general
   machinery. All new loops iterate `roles.iter().filter(cardinality ==
   Many)`; a binary all-`One` relation has no `Many` roles, so the loops are
   no-ops and its SQL is byte-for-byte unchanged (confirmed by reading —
   the loops simply don't execute their bodies when `spilled`/the filtered
   iterator is empty).

No deviation rises to a compliance failure. **Verdict: ✅ compliant** with
the corrected 14a scope.

## Sabotage results (all six required, all confirmed, all restored)

1. **Deleted the spill-row INSERT loop** in `insert_relationship`. Exactly
   `a_many_valued_role_holds_several_players_in_one_relation` failed (0 spill
   rows instead of 2). Caught.
2. **Deleted the spill-row DELETE in the relationship-delete branch.**
   Exactly `deleting_a_relation_removes_its_spilled_players` failed (1 spill
   row survives instead of 0); `detach_deleting_a_node_...` still passed.
   Caught, and confirms the two delete paths are independently tested.
3. **Deleted the spill-row DELETE in the node `DETACH DELETE` branch.**
   Exactly `detach_deleting_a_node_removes_its_relations_spilled_players`
   failed; `deleting_a_relation_removes_its_spilled_players` still passed —
   a genuinely different test than #2, confirming the two paths are not
   accidentally covered by one shared assertion. Caught.
4. **Reintroduced the binder's `Many` guard** (`at_unsupported` on any
   `Many`-role argument). Three of the four new tests failed immediately at
   bind time (`Mutation(Bind(Unsupported {...}))`). Caught.
5. **Weakened the duplicate check to accept a repeated `One` role** (flipped
   the cardinality condition). `a_single_valued_role_given_two_players_is_refused`
   failed, and so did the pre-existing Task 13a regression test
   `naming_one_role_twice_is_refused_rather_than_last_write_wins`. Caught —
   confirms the guard is load-bearing for both the new and the pre-existing
   case.
6. **Reverted `.flat_map()`/`.filter()` back to `.filter_map()`/`.find()`**
   (the implementer's self-reported deviation). Confirmed the claim:
   `a_many_valued_role_holds_several_players_in_one_relation` failed (1 spill
   row instead of 2 — the second `witness` fill was silently dropped, exactly
   as the report describes). Separately confirmed by reading (not sabotage,
   since a `One` role can have at most one fill by construction — a second
   argument for it errors out via `DuplicateRoleArgument` before reaching the
   `roles` construction) that the new form still emits exactly one entry per
   `One` role and preserves declaration order: the outer iteration is still
   over `declared` in its original order, `flat_map` only changes how many
   entries come out of each declared role's inner match, not the outer
   ordering. `role_arguments_bind_by_name_regardless_of_source_order`
   (pre-existing, still passing at baseline) covers this ordering guarantee.

Working tree confirmed restored: after all six sabotages, `git diff` against
`a35a14c05` on `binder.rs`/`mutation.rs` is empty, `cargo fmt -p
turso_graph_frontend -- --check` exits 0, and `cargo test -p
turso_graph_frontend --test nary_relations` shows all 12 tests passing. Only
`graph/test-results/{REPORT.md,benchmarks.jsonl,runs.jsonl}` remain modified
(pre-existing, not touched by this review, not this task's to commit).

## Judgement calls beyond the sabotage checklist

- **Spill write ordering / parameter binding.** Confirmed: the spill INSERT
  runs after `insert_entity` returns `identity`, and the player `Value` is
  bound through the same internal-`HashMap` + named-parameter mechanism as
  every other mutation write in this file — never interpolated into the SQL
  string. Table names (`role.spill_table`) go through `quoted_identifier`;
  player and relation-id values go through bind parameters.
- **Empty-string `column` on `Many` roles.** Traced every consumer of
  `role.column` in `mutation.rs`, `lowering.rs`, and `catalog.rs`.
  `structural_columns()` and `single_valued_roles()` both filter to
  `RoleCardinality::One` before touching `.column`. `install_role_index` and
  `install_role_pair_indexes` (catalog.rs) are only ever called for `One`
  roles (the `match role.cardinality { One => install_role_index, Many =>
  install_spill_table }` dispatch in `register_graph`, and
  `install_role_pair_indexes`'s own `.filter(cardinality == One)`).
  `validate_source_identifiers` also only collects `One`-role columns. The
  one place the empty string does flow is `sql_string(&role.column)` when
  writing the catalog's own `RELATIONSHIP_ROLES_TABLE` metadata row — that's
  a bound/escaped SQL *string literal* recording metadata, not an
  *identifier*, so it is not the `quoted_identifier("")` hazard the review
  asked about. No path in the reachable code reaches
  `quoted_identifier("")`.
- **Structural-column exclusion, both directions.** `Many` roles are
  excluded from `structural_columns()` (correct — no column exists to list)
  and correctly *included* in both new delete loops and the insert loop,
  which explicitly iterate the *full* `layout.roles`/`relationship.roles`
  filtered to `Many`, not `single_valued_roles()`.
- **Test hygiene.** Each of the 4 new tests has a doc comment stating why
  the behavior matters (double-counting risk, dangling-participant risk,
  distinguishing the two delete paths, regression-guarding the existing
  duplicate check under new code). All four are load-bearing per the
  sabotage run above — none merely re-assert something another test already
  covers.
- **YAGNI.** One design decision beyond the letter of the brief: the insert
  loop is gated on `insert_entity`'s `created` flag, skipping spill writes
  when a `MERGE` matched an existing relation rather than creating a new
  one. This is a sound inference from existing `MERGE`/`on_create`/
  `on_match` semantics (avoids duplicating spill rows on a re-run MERGE) and
  is not scope creep — but see the finding below, it is untested.

## Task quality verdict: **Approved**, with one Minor finding

- **Minor — `MERGE` + `Many`-role interaction is untested.** The
  `if created { ... }` guard around the spill-write loop in
  `insert_relationship` is new logic this task added beyond the brief's
  literal Step 3, and it is not exercised by any test: no test creates a
  `witnessed_session` relation via `MERGE` with a `witness` role to confirm
  spill rows are (a) written once on first match-creates, and (b) not
  duplicated on a subsequent match-only run. The logic reads correctly by
  inspection and is consistent with existing `on_create`/`on_match`
  semantics elsewhere in the file, but per the project's "every change needs
  a test that fails without it" rule, this specific branch has no failing-
  without-it test. Low severity: `MERGE` over a `Many` role is not part of
  14a's four required scenarios, and getting this wrong would silently
  duplicate spill rows on repeated `MERGE`s rather than corrupt or crash —
  a real but narrow gap, appropriately deferred rather than a blocker.

No Critical or Important findings.

## ⚠️ Cannot verify from diff

None. Every requirement checked was either directly readable in the diff,
traceable through the surrounding (unmodified) code in the working tree, or
confirmed live by sabotage/test run.
