# Task 18b Review — MERGE over a standalone role pattern

Diff range reviewed: `ae795a64c..71104578d` (commit `71104578d`, "graph: let MERGE
take a standalone role pattern"). `b949582c0` (test-results bookkeeping) is
excluded per instructions.

All sabotages below were applied with `Edit`, tested with `cargo test`, and
reverted before finishing. `git status` is clean apart from this report file.

## Sabotage results

1. **Delete the `if created` guard (`mutation.rs:2020`, `if created {` →
   `if true {`)**: `cargo test -p turso_graph_frontend --test nary_relations`
   goes from 23 passed to **22 passed, 1 failed**. The failure is exactly
   `merging_a_relation_with_a_many_valued_role_does_not_duplicate_spill_rows`:
   `assertion left == right failed: the witness is written once, not once per
   MERGE run / left: [[Integer(2)]] / right: [[Integer(1)]]`. Matches the
   implementer's reported output verbatim. Guard restored; suite back to 23/0.

2. **Drop the `Many`-role `EXISTS` predicate from the merge key** (stubbed the
   `merge_predicates.push(...)` call to a discarded `let _ = format!(...)`):
   **all 23 existing tests still pass.** Nothing catches this. I then added a
   throwaway test (`merge_with_different_witness_should_not_collapse`, in
   `witnessed_session`, same `start`/`end` but a different `witness` node on
   the second `MERGE`) — with the predicate stubbed out, it **fails**:
   `relationships` count comes out `1` instead of the expected `2` — the
   second MERGE silently matched the first relation and dropped the new
   witness fact. Restoring the real `EXISTS` predicate makes that same
   throwaway test pass. The throwaway test and my sabotage edit were both
   reverted; final state is the original diff, 23/23 passing.
   **Finding: the shipped `EXISTS` predicate is correct, but no test in the
   diff exercises it — the existing tests only vary `folio`/`start`/`end`
   (`One` roles), never the `Many` role alone.** This is a real, silently
   losable-data gap in test coverage for exactly the step the brief called
   "the one genuine gap."

3. **Truncate the merge key to a single (first bound) `One` role**
   (`fixed.truncate(1)`, gated on `if merge` so plain CREATE is unaffected):
   **exactly one test fails** —
   `merge_matches_on_the_full_set_of_bound_roles`, with the message `a
   different folio is a different assertion, not an update of the first`
   (`left: [[Integer(1)]]`, expected `[[Integer(2)]]`). All other 22 tests
   still pass. This is precisely the brief-named test and confirms the merge
   key is not silently subset-able. Reverted; 23/23 restored.

4. **Grammar regression checks**, run (not just read):
   - Ordinary arrow form `MERGE (a)-[r:KNOWS]->(b)`: parses (temp unit test,
     removed after) and executes end-to-end
     (`an_arrow_form_merge_inside_foreach_still_binds` and the pre-existing
     `parses_create_and_merge_patterns`, both pass).
   - `MERGE` nested in `FOREACH`, both forms: `a_role_pattern_merge_inside_foreach_still_binds`
     and `an_arrow_form_merge_inside_foreach_still_binds` both pass end-to-end
     against a real session (`cargo test -p turso_graph_frontend --test
     nary_relations` — 23/23).
   - `ON CREATE SET ... ON MATCH SET ...` still attaches to a role-pattern
     MERGE: added and ran a temporary unit test parsing `MERGE
     [x:Transcription](scribe: p, text: t, folio: f) ON CREATE SET x.year = 1
     ON MATCH SET x.year = 2` — `on_create.len() == 1`, `on_match.len() ==
     1`, passed. Removed after.
   - MERGE does not silently accept a comma-separated pattern list: added and
     ran a temporary unit test — `parse("MERGE (a:Person), (b:Person)")`
     returns `Err`, as required (`merge_clause` uses `pattern_element`, not
     `pattern`, in `cypher.pest`). Removed after.
   No regression found in any of the four checks.

5. **Test-count reconciliation.** `cargo test -p turso_graph_cypher -p
   turso_graph_frontend` reproduces the report's exact figure: **335 passed,
   3 ignored (15 suites)**. Task 17's "390 passed, 3 ignored (22 suites)" ran
   a *different, wider* package set: `grep` of `task-17-report.md` shows its
   gate command was `cargo test -p turso_graph_ir -p turso_graph_frontend -p
   turso_graph_runtime -p turso_graph_cypher` (four packages). Running just
   the two extra packages here (`cargo test -p turso_graph_ir -p
   turso_graph_runtime`) gives 60 passed. The 390→335 drop is **package
   scope** (missing `turso_graph_ir`/`turso_graph_runtime`, neither touched by
   this diff), not lost or disabled tests — the diff adds tests and fixes one
   in place, it deletes none. **However, the report's own label is
   inaccurate**: it calls the 335-count run "Full workspace `cargo test` (all
   15 suites)," but it is not the full workspace — it is exactly the
   two-package command from the brief's Step 6 gate line, mislabeled. Minor
   report-accuracy finding; not a defect in the diff.

6. **Clippy concern.** Did not use `git stash`. Reasoned structurally first:
   diff touches zero files under `core/`, and `core/` cannot depend on the
   graph crates, so nothing in this diff can change `core`'s compiled output.
   `git blame` on both flagged lines predates this diff by months
   (`core/mvcc/persistent_storage/logical_log.rs:262` from 2026-07-20;
   `core/vdbe/mod.rs:43` from 2026-01-18), consistent with the report's
   attribution to merge commit `62c779465`. Then ran the gate commands
   directly:
   - `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
     (the **literal** command Step 6 specifies) — **exit 0, zero warnings**,
     reproduced twice. This contradicts the report's claim that "the literal,
     full-workspace form" fails.
   - `cargo clippy -p turso_graph_cypher -p turso_graph_frontend
     --all-features --all-targets -- --deny=warnings` (a narrower,
     two-package invocation) — **does** fail, with exactly the two
     unused-import errors the report quotes, at the same two lines.
   The two commands select different feature-unification scopes for `core`
   (a Cargo mechanic, not code content), which is why the same lines are
   clean under one invocation and not the other. **Finding: the report's
   Concerns section describes the wrong command as failing** — the literal
   full-workspace gate specified by Step 6 passes cleanly; only a narrower,
   unspecified per-package invocation reproduces the warnings. Either way the
   warnings are pre-existing and unrelated to this diff (confirmed
   structurally, not by `git stash`), so this does not block the task, but
   the report over-states which gate command actually fails.

## Verdict 1 — Spec compliance (task-18b-brief.md)

- **Step 1 (grammar accepts a role pattern after MERGE):** ✅ met.
  `merge_clause = { MERGE ~ pattern_element ~ merge_action* }` preserves the
  `role_pattern | path_pattern` ordered choice and does not widen MERGE to a
  comma-separated `pattern` (verified: `MERGE (a:Person), (b:Person)` is a
  parse error). `merge_action*` still attaches (verified end-to-end).
- **Step 2 (write the failing tests first):** ✅ met, as far as the diff and
  report show. The four new `nary_relations.rs` tests follow the fixture
  idiom the brief specifies (`fixture::ternary_session`/`witnessed_session`,
  `second_connection`, `.expect_err`, `run_collect_rows`), and the report's
  "before" transcript for Step 1 and Step 5 shows genuine red-then-green
  cycles.
- **Step 3 (route role pattern through the merge binder, one implementation):**
  ✅ met. `bind_merge_pattern` is the single new dispatch point, used from both
  the top-level `clause` match and the `foreach_body` match; it calls the
  existing `bind_create_role_pattern` rather than re-implementing role
  resolution, duplicate-argument refusal, the required-role check, or the
  target-type check. Confirmed only two call sites of
  `bind_create_role_pattern` exist in the whole file (CREATE's existing one,
  and this new one).
- **Step 4 (`Many` roles in the merge key — "the one genuine gap"):**
  ⚠️ **partially met** — the production code is correct (sabotage 2 proved
  the `EXISTS` predicate is load-bearing: removing it silently collapses two
  relations differing only in a `Many` role), branches only on
  `role.spill_table`/`role.cardinality` (never name/position/`is_binary`),
  and uses `quoted_identifier` throughout. But **the diff ships no test that
  would fail if this predicate were removed or the `Many` role were left out
  of the merge key** — I had to write one myself to observe a red result.
  This is exactly the risk the brief called out by name ("the one genuine
  gap") and asked reviewers to scrutinize hardest; the implementation closes
  the gap, but the test suite does not verify that it does.
- **Step 5 (cover the `if created` spill guard):** ✅ met, verified live —
  sabotage 1 reproduced the implementer's reported failure exactly.
- **Step 6 (gate and commit):** ✅ met for `cargo fmt`, `cargo test -p
  turso_graph_cypher -p turso_graph_frontend` (335/3/0, reproduced), and the
  corpus/cypherbench runs (controller already verified the corpus row per
  the task instructions; not re-run here). ⚠️ the report's clippy narrative
  is inaccurate (see sabotage 6) — the literal gate command passes clean, so
  the gate is actually satisfied more cleanly than reported, but the report
  itself misidentifies which command fails.

## Verdict 2 — Task quality

**Findings by severity: 0 Critical, 1 Important, 2 Minor.**

- **Important — Step 4 has no red/green test for its own stated purpose**
  (sabotage 2). The `EXISTS`-predicate code is correct and well-formed, but
  none of the four new tests differ only in a `Many`-role player while
  holding every `One` role fixed, so nothing in the shipped suite would
  catch a regression here — the exact scenario the brief flagged as the
  hardest part of the task. `merge_matches_on_the_full_set_of_bound_roles`
  covers `One`-role (`folio`) subsetting; nothing covers `Many`-role
  subsetting. Recommend adding a test resembling my throwaway
  `merge_with_different_witness_should_not_collapse` (fixed `start`/`end`,
  varying `witness`) before considering Step 4 done.
- **Minor — report mislabels its own `cargo test` scope** (sabotage 5): calls
  a 2-package run "Full workspace cargo test (all 15 suites)" when it is the
  brief's own Step 6 two-package command, not the full workspace. No test was
  lost; this is a documentation-accuracy issue in the report only.
- **Minor — report's clippy Concerns section names the wrong failing command**
  (sabotage 6): the literal, full-workspace gate command Step 6 specifies
  passes with zero warnings; only an unspecified, narrower two-package
  invocation reproduces the two pre-existing `core/` warnings. The
  underlying fact (pre-existing, unrelated to this diff) is correct, but the
  claim that "the literal full-workspace form... fails" is not reproducible
  and should be corrected.

No other constraint violations found: no `if roles.len() == 2` / `is_binary`
/ hard-coded `"start"`/`"end"` anywhere in the diff (grepped); role
resolution and the merge key both key off `RoleId`/`role.cardinality`/
`role.spill_table`, never argument order or position; every new SQL
identifier interpolation uses `quoted_identifier` (with the `d`); MERGE's
grammar takes exactly one `pattern_element`, not a `pattern` list; and every
call site that used to read `MergeClause.path` as a `PathPattern` field was
updated to match the new enum (checked via `grep` across `graph/cypher/src`
and `graph/frontend/src` — no stale references found).
