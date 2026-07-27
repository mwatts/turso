# Task 15 report: role updates after create

## Scope

Per the brief's CONTROLLER CORRECTIONS: `SET [x](role: player, ...)` repoints
one or more roles of a relation already bound in the current statement (by
today's arrow form, since Task 13b's standalone MATCH-side role pattern is
not implemented). No type list, no property map — the relation is already
bound, so re-stating its type would be a second source of truth. `SET` on a
`Many` role replaces its whole player set; it does not append.

## What changed and why

1. **`graph/cypher/src/cypher.pest`** — added `set_role_item = { "[" ~
   identifier ~ "]" ~ role_arguments }`, prepended to `set_item`'s
   alternation (`set_role_item | set_property_item | set_merge_item |
   set_replace_item | set_label_item`). Reuses `role_arguments` from Task 12
   unchanged. No grammar ambiguity: none of the other four alternatives can
   start with `[`.

2. **`graph/cypher/src/ast.rs`** — added `SetItem::Roles { relation:
   Spanned<String>, roles: Vec<RoleArgument>, span: Span }`, matching the
   file's existing struct-variant-with-`Spanned<T>` style (not a bare
   `String`, not a new `RoleUpdate` struct — reused `ast::RoleArgument`
   as-is).

3. **`graph/cypher/src/parser.rs`** — added the `Rule::set_role_item` arm in
   `walk_set_item`, building `SetItem::Roles` from the existing
   `walk_role_argument`/`identifier_text`/`pair_span` helpers.

4. **`graph/ir/src/mutation.rs`** — added `Mutation::SetRoles(SetRoles)` and
   the `SetRoles { relation: BindingId, source: SourceTableId, roles:
   Vec<RoleBinding> }` struct. No `replace_many` field: whether a role
   replaces (vs. was never touched) is already derivable from
   `RelationshipTableLayout`'s `role.cardinality`/`role.spill_table`, which
   the executor reads directly — a redundant field would be a second source
   of truth. Re-exported `SetRoles` from `graph/ir/src/lib.rs` (a compile
   error surfaced this omission on first build; fixed immediately).

5. **`graph/frontend/src/binder.rs`**:
   - Extracted **`bind_role_player`** (the helper named per Correction E)
     from `bind_create_role_pattern`'s inline target-type check. Signature:
     `fn bind_role_player(&self, relationship_type: &str, role:
     &crate::semantic::SemanticRole, argument: &cypher::RoleArgument) ->
     Result<ir::BindingId, BindError>`. Both `bind_create_role_pattern` and
     the new `bind_set_roles` call it, so a create and an update refuse the
     same player shapes for the same reason instead of maintaining two
     copies of the check.
   - Added `fn bind_set_roles` implementing the full resolution: resolve the
     relation variable via the pre-existing `resolve_set_variable` (kind
     must be `Relationship`), read its relationship type via the
     pre-existing `entity_type_names` (refuse anything but exactly one
     type), look up declared roles and the source table, then per role
     argument: look up the declared role by name (`BindError::UnknownRole`
     if absent), apply the duplicate-role rule (repeated `One` role →
     `BindError::DuplicateRoleArgument`; repeated `Many` role → allowed),
     resolve the player via `bind_role_player`. **No required-role check** —
     a role update names a subset by design, unlike creation which must
     cover every non-optional role (this is explicit; the create path's
     check does not run on this branch at all, it is simply never called).
     Final `roles` list is rebuilt in **declaration order** via the same
     `flat_map`-over-declared-roles pattern `bind_create_role_pattern` uses,
     so a `Many` role's players stay grouped by `RoleId` however the query
     spelled its arguments.
   - Added the `cypher::SetItem::Roles { relation, roles, span } =>
     self.bind_set_roles(relation, roles, *span)` arm to `bind_set_item`.

6. **`graph/frontend/src/mutation.rs`** — added the `ir::Mutation::SetRoles`
   arm to `execute_operation` (modeled on the `SetProperty` arm, per
   Correction F): look up the relationship layout and the relation's
   identity value, then **group `update.roles` by `RoleId`** (first-seen
   order, linear scan — role counts per statement are small) so a `Many`
   role named by two arguments in one `SET` gets exactly one spill purge
   followed by one insert per player, not one purge per argument (which
   would delete an earlier argument's just-inserted row). Per group:
   - `One` — the binder's duplicate check guarantees exactly one player;
     bind it as a `$turso_internal_set_role_N` parameter and collect a
     `column = $param` assignment.
   - `Many` — `DELETE FROM <spill_table> WHERE relation_id = $id` (replace,
     not append — no "unset" syntax exists to undo an append, so appending
     SET would make running a statement twice mean something different
     from running it once), then one parameterized `INSERT INTO
     <spill_table>(relation_id, node_id) VALUES ($id, $player)` per player
     in the group.
   All `One`-role assignments combine into a single `UPDATE <table> SET
   col1 = $p1, col2 = $p2 WHERE <identity> = $id` statement. Every player
   value is bound as a named parameter through the accumulating `internal:
   HashMap<String, Value>` and `run_ignore` — never string-interpolated.

7. **`graph/frontend/tests/nary_relations.rs`** — 4 new tests, all against
   `fixture::witnessed_session()` except the wrong-type test (see below):
   - `a_single_valued_role_can_be_repointed_after_create` — CREATE a KNOWS
     with `start`/`end`/`witness`, then `MATCH
     (a:Person)-[r:KNOWS]->(b:Person), (c:Person {id: 3}) SET [r](start:
     c)`; asserts the raw `relationships` row's `src` moved to `c` while
     `dst` (end) is untouched.
   - `setting_a_many_valued_role_replaces_rather_than_appends` — CREATE with
     two witnesses, `SET [r](witness: w3)`, asserts
     `relationships__witness` holds exactly the one new witness (the two
     old rows are gone, not joined). Comment in the test states the reason
     replace-not-append is correct (running the same SET twice must mean
     the same thing both times).
   - `a_role_update_rejects_a_player_of_the_wrong_type` — per Correction H,
     `witnessed_session`'s three roles all target `Person`, so it cannot
     exercise a target-type refusal. Uses the file's existing hand-rolled
     `RoledCatalog: GraphCatalogSnapshot` + `bind_role_pattern_query`
     instead (bind-only, no database), combining CREATE and SET in one
     statement: `CREATE [x:Transcription](scribe: p, text: t, folio: f) SET
     [x](scribe: t)`. **Finding, not a blocker**: `RoledCatalog` binds the
     SET target fine, because CREATE registers `x`'s entity binding before
     SET resolves it in the same bind pass — no MATCH (arrow or standalone)
     is needed at all for a same-statement bind-only test.
   - `a_role_update_rejects_a_null_player` — `SET [r](start: null)` on a
     bound KNOWS; asserts the error message names the role (`"start"`).

## Deviation from the brief

**Correction G's literal code does not compile.** The brief's suggested
refusal is `at_unsupported(argument.player.span, &format!("clearing role
\`{}\` — there is no null player", role.name))`, but `at_unsupported`'s
signature is `fn at_unsupported(span: cypher::Span, feature: &'static str) ->
BindError` (`binder.rs:7457`, unchanged by this task) — it cannot accept an
owned `String`/`&String` from `format!`. Since Correction G also explicitly
forbids adding a new `BindError` variant, I reused the existing
`RoleTargetTypeViolation` variant (its fields are owned `String`s, not
`&'static str`) specifically for a literal `null` player: `bind_role_player`
matches `cypher::Expression::Literal(cypher::Literal::Null)` as a special
case before falling through to the generic `at_unsupported("a role player
that is not a bound variable")` for every other non-variable expression.
`RoleTargetTypeViolation`'s message names the role
(`a_role_update_rejects_a_null_player` asserts `.contains("start")`), so the
test's intent — a role-naming refusal — is satisfied without a new variant
and without a non-compiling `format!` call. Non-null non-variable
expressions (e.g. a literal, a property access) still go through the
generic `at_unsupported` path, since those aren't permanently meaningless the
way a null player is and might plausibly gain support later.

## Every `ir::Mutation` match site touched

Confirmed by repo-wide search: **`execute_operation` in
`graph/frontend/src/mutation.rs` is the only exhaustive match over
`ir::Mutation` outside test code**, and it is the only one edited. (The
brief's Correction D also names a match at `binder.rs:7551` — that line is
inside a *different* function, `attach_merge_actions`, whose match already
carries a wildcard arm, so no change was needed there; `testkit/performance.rs`
and `lowering.rs` match over unrelated types.)

## Found but not fixed

Nothing new found and left unfixed in this task's scope. (Task 14a's already
-documented DETACH DELETE / more-than-two-roles limitation is unrelated to
SET and untouched here.)

## Invariant preserved

No `roles.len() == 2` check and no hard-coded `"start"`/`"end"` name was
added to any general machinery. Every new/edited code path resolves roles by
`RoleId` or by declared name (`role.name`, `role.role`,
`relationship_roles()` lookups). `bind_role_player` and the executor's
per-role grouping treat `One` and `Many` identically as far as "what is a
role" goes; only the cardinality-specific write shape (single UPDATE column
vs. spill purge+insert) branches on `role.cardinality`, read from the
catalog layout, not hard-coded.

## Test commands and results

- `cargo test -p turso_graph_frontend --test nary_relations` — **16 passed**
  (12 pre-existing + 4 new).
- Verified the 4 new tests are not tautological: `git stash push` on the 7
  non-test implementation files (grammar/AST/parser/IR/binder/executor),
  re-ran the same test command — **12 passed, 4 failed**, all 4 failing at
  parse (`ParseError { message: "expected set_item", ... }`), i.e. the
  syntax genuinely does not parse without the change. `git stash pop`
  restored the implementation; re-ran — 16/16 passed again.
- `cargo test -p turso_graph_cypher -p turso_graph_frontend` — **325 passed,
  3 ignored (15 suites)**, no regressions.
- `cargo fmt -p turso_graph_cypher -p turso_graph_ir -p turso_graph_frontend
  --check` — exit 0, no diff.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
  — one `clippy::redundant_clone` error on first run
  (`graph/frontend/src/mutation.rs:1645`, `identity.clone()` cloned into a
  `HashMap` and never used again afterward); fixed by moving `identity` into
  the `HashMap::from([...])` literal instead of cloning it a second time.
  Re-ran: **0 errors** workspace-wide (the only other lines in the output
  are a pre-existing `ar -D`/Xcode toolchain build-script warning from
  `limbo_sqlite_test_ext`, unrelated to Rust/clippy and to any file this task
  touched).

## Gate runs (release, per `mise.toml`)

### `mise run corpus`

Run id `20260726T123944.708453Z-c8c859820afc-corpus-deep`, 10242 records,
compared per suite against the immediately preceding baseline
(`20260726T115028.302572Z-754dce74d819-corpus-deep`, recorded by Task 14a's
own gate run):

| suite | baseline passed | this run passed | verdict |
|---|---|---|---|
| age-deep | 3042 | 3042 | exact match, no outcome changes |
| cqlite-deep | 113 | 113 | exact match, no outcome changes |
| grafeo-deep | 277 | 277 | exact match, no outcome changes |
| sparrowdb-deep | 2164 | 2164 | exact match, no outcome changes |
| tck-deep | 3330 (band 3329-3332) | 3331 | inside band; +1 |

The one tck-deep change is
`tck.expressions.temporal.temporal10.scenario-12.examples-1-row-1`:
Failed → Passed. This is a timing-sensitive temporal-function scenario
entirely unrelated to role updates or n-ary relationships (this task never
touches temporal functions), and lands inside the brief's documented ±2 flake
band on an otherwise-identical commit. Total: 8927/10242 passed (baseline
8926) — the +1 total is exactly the one tck-deep flake; every other suite is
bit-for-bit identical. Gate: **pass**, per suite.

The corpus run itself still exits non-zero (`clean=false`, `[corpus] ERROR
task failed`) for the same ~1262 pre-existing failing queries documented in
prior task reports (unsupported procedures, missing functions, parameter-
binding gaps, etc.) — none related to this task.

### `mise run cypherbench-sample`

Recorded at `2026-07-26T12:41:17Z`, compared against every prior recorded
row in `graph/test-results/benchmarks.jsonl` (a long, stable history spanning
`2026-07-25T20:59` through `2026-07-26T11:55`, all identical):

| domain | matched | mismatched | errored |
|---|---|---|---|
| company | 13 | 12 | 0 |
| fictional_character | 14 | 11 | 0 |
| flight_accident | 24 | 1 | 0 |
| geography | 11 | 14 | 0 |
| movie | 6 | 19 | 0 |
| nba | 25 | 0 | 0 |
| politics | 15 | 10 | 0 |

Identical to baseline in every domain, `errored=0` throughout. Gate:
**pass**.

## Files touched

- `graph/cypher/src/ast.rs`
- `graph/cypher/src/cypher.pest`
- `graph/cypher/src/parser.rs`
- `graph/frontend/src/binder.rs`
- `graph/frontend/src/mutation.rs`
- `graph/frontend/tests/nary_relations.rs`
- `graph/ir/src/lib.rs`
- `graph/ir/src/mutation.rs`

`graph/test-results/{REPORT.md,benchmarks.jsonl,runs.jsonl}` were modified by
running the two mise gates (as required, before committing) but are **not**
part of the commit — the controller records `graph/test-results/` changes
separately.

## Commit (before review round 1)

`f26ee9db77408607467e23b57fa181cfab23e1af` — "graph/frontend: repoint a
bound relation's roles with SET [x](role: player)", signed, 8 files changed
(475 insertions, 45 deletions).

## Review round 1: three missing regression tests

Review verdict: spec compliance approved, quality approved with findings.
All three findings were "implementation is correct, but unprotected by a
shipped test" — no code fix needed, only tests. Fixed by amending commit
`f26ee9db7` (same task, review package scoped to that range) rather than a
follow-up commit.

Added 3 tests to `graph/frontend/tests/nary_relations.rs` (12 pre-existing +
4 from the original submission + these 3 = 19 total):

1. **`setting_a_many_valued_role_twice_is_idempotent`** — runs the same
   `SET [r](witness: w3)` twice against a relation that started with two
   witnesses, asserts `relationships__witness` holds exactly one row
   (`[5]`) afterward. Guards the brief's central semantic claim: running the
   same statement twice must mean what running it once means.
   **Sabotage run**: commented out the spill-table `DELETE` (the purge) in
   `execute_operation`'s `Many`-role branch, turning replace into append.
   Result: 3 tests went red, including this one
   (`setting_a_many_valued_role_twice_is_idempotent` failed with 4 rows
   `[3, 4, 5, 5]` instead of `[5]`). Restored the `DELETE` and reran — 19/19
   green.

2. **`setting_a_many_valued_role_with_two_players_in_one_set_lands_both`** —
   `SET [r](witness: w1, witness: w2)` in one statement, asserts both `w1`
   and `w2` land in the spill table. Guards the brief's explicit "purge once
   per role, not once per argument" requirement.
   **Sabotage run**: moved the spill-table `DELETE` from once-before-the-
   player-loop to inside the per-player loop (so each insert is preceded by
   its own purge, deleting the previous argument's just-inserted row).
   Result: exactly this one test went red (`[5]` instead of `[4, 5]`); the
   other 18 tests, including
   `setting_a_many_valued_role_twice_is_idempotent` (which only ever names
   one player per SET, so a purge-per-argument is indistinguishable from a
   purge-per-role there), stayed green. Restored the original ordering and
   reran — 19/19 green.

3. **`a_role_update_rejects_a_repeated_one_role_argument`** — `SET [r](start:
   b, start: c)` on a bound KNOWS, expects `BindError::DuplicateRoleArgument`
   naming `start`. Guards the SET-path half of the create/update shared
   duplicate-role rule (the existing
   `a_single_valued_role_given_two_players_is_refused` test only covers
   CREATE).
   **Sabotage run**: gated `bind_set_roles`'s duplicate check with `if false
   && repeated && ...`, disabling it while leaving the surrounding code
   otherwise untouched. Result: exactly this one test went red (the
   duplicate `start` argument silently bound instead of erroring); all 18
   others stayed green. Restored the check and reran — 19/19 green.

The one Minor from review (null-player refusal reusing
`BindError::RoleTargetTypeViolation` instead of a more precise variant) is
deferred, per the reviewer's own agreement that the rendered message is
accurate and nothing branches on the variant — left unchanged.

Post-fix verification: `cargo test -p turso_graph_frontend --test
nary_relations` — 19 passed; `cargo test -p turso_graph_cypher -p
turso_graph_frontend` — all suites green, 0 failed; `cargo fmt` — no diff
beyond the test file; `cargo clippy --workspace --all-features --all-targets
-- --deny=warnings` — clean. Did not re-run `mise run corpus` or `mise run
cypherbench-sample` (test-only change, gates already passed on this code per
the coordinator's instruction).

## Commit (after review round 1)

`bdfa4ce02b0b5d81ff77141fcb9b5ac51ed9b9df` — same subject, amended onto the
original commit, 8 files changed (596 insertions, 45 deletions).
