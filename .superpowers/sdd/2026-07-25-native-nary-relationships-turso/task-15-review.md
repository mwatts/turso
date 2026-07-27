# Task 15 review: `SET [x](role: player)` — role updates after create

Reviewed at `f26ee9db7` (base `c8c859820`), diff:
`review-c8c859820..f26ee9db7.diff`. Verified by sabotage (temporary edits,
run, reverted with `git checkout`) plus a targeted reading of the grammar,
IR, binder, and executor.

## Verdict 1: Spec compliance

**Compliant.** Every controller correction (A–I) was followed:

- Grammar: `set_role_item` reuses `role_arguments`, is first in `set_item`'s
  alternation, takes a bare `identifier` (no type list/property map). No
  other `set_item` alternative can start with `[`; confirmed no parse
  regression (`turso_graph_cypher`'s `parses_entity_set_forms` and the full
  23-test suite still pass unmodified).
- AST: struct variant `SetItem::Roles { relation: Spanned<String>, roles:
  Vec<RoleArgument>, span }`, matching file convention. No `RoleUpdate`
  struct.
- IR: `SetRoles { relation: BindingId, source: SourceTableId, roles:
  Vec<RoleBinding> }` — no `replace_many` field. `Mutation::SetRoles` added
  and re-exported from `graph/ir/src/lib.rs`.
- Binder: `bind_role_player` extracted and shared by
  `bind_create_role_pattern` and the new `bind_set_roles`; no required-role
  check on update; duplicate-role rule (`One` refused, `Many` allowed)
  carried over.
- Executor: real `ir::Mutation::SetRoles` arm in `execute_operation`
  (`graph/frontend/src/mutation.rs:1635`), modeled on the `SetProperty` arm,
  using `quoted_identifier`, `identity_parameter`, `run_ignore` — the
  fictitious struct/method shape from the original brief body was correctly
  discarded in favor of the corrected free-function shape.
- Gate corrections (package name, `git add` with explicit paths, both mise
  gates before commit, per-suite reporting) were followed per the
  implementer's report; not independently re-run per your instruction.

No `roles.len() == 2` / `is_binary` check and no hard-coded `"start"`/`"end"`
were introduced in general machinery — confirmed by reading every new/edited
line; all role resolution goes through `RoleId` or declared-name lookup.

## Verdict 2: Task quality

**Approved, with findings.** Nothing Critical. Three Important test-coverage
gaps (behavior is correct — verified directly — but unprotected by a shipped
regression test), one Minor error-classification defect, one informational
note.

### Findings

1. **Important — no shipped test runs the same `SET` on a `Many` role
   twice.** The brief made double-execution-idempotency the task's central
   semantic claim and asked to check it directly. I patched
   `setting_a_many_valued_role_replaces_rather_than_appends` to issue the
   identical `SET [r](witness: w3)` a second time: the player set stayed
   `[5]` both times, confirming the behavior is correct. But no test in the
   shipped suite exercises this — a future regression here (e.g. someone
   "optimizing" the purge into a conditional) would go undetected. Sabotage
   (deleting the spill-purge, making `Many` append) *was* caught by the
   existing single-execution test, so the replace-vs-append logic itself is
   guarded — only the twice-run scenario specifically is not.

2. **Important — no shipped test names one `Many` role with two players in
   one `SET`.** The brief explicitly called this out: "group the arguments
   by role... so two arguments naming one `Many` role in a single `SET` both
   land." I sabotaged the executor to purge per-argument instead of per-role
   and no existing test failed. I then added a probe test
   (`SET [r](witness: w5, witness: w6)`): under the sabotage only the last
   player (`6`) landed; with the real code both `5` and `6` landed. The
   implementation is correct, but this exact requirement — the one the brief
   flagged as most likely to regress — ships with zero test coverage.

3. **Important — no shipped test exercises the duplicate-`One`-role refusal
   on the `SET` path.** I disabled the `repeated && role.cardinality ==
   One` check inside `bind_set_roles` (the SET-specific copy at
   `binder.rs:2317` in the working tree, distinct from the CREATE-path
   check the existing `naming_one_role_twice_is_refused_rather_than_last_write_wins`
   test already guards) and all 16 tests still passed. The rule is
   implemented correctly and mirrors the create path, but is unguarded by a
   test for the update path specifically.

4. **Minor — the null-player refusal reuses `RoleTargetTypeViolation`, which
   misdescribes the failure category.** Correction G's own suggested code
   (`at_unsupported(span, &format!(...))`) does not compile because
   `at_unsupported` takes `&'static str` — the implementer's diagnosis of
   that is correct and is the controller's error, not theirs. Their
   substitute reuses `RoleTargetTypeViolation { relationship_type, role,
   found: "a null player -- there is no way to clear a role via SET", ...
   }`. I ran the actual test and captured the rendered message: `Cypher
   mutation binding failed: role \`start\` of relationship type \`KNOWS\`
   does not accept a null player -- there is no way to clear a role via SET
   at byte 54..58`. The *text* is accurate and unambiguous — it is not a
   dishonest message to a user. But the *variant* is semantically wrong: a
   null player is not a player of the wrong target type, it is the absence
   of a player, and the code comment even says so in three different places.
   No other code branches on this variant today (confirmed by
   `grep -rn RoleTargetTypeViolation`; only `Display` and two other bind-time
   tests reference it), so the practical blast radius is limited to
   maintainer confusion and error-triage/telemetry that might one day key
   off variant identity. A dedicated variant (e.g. `NullRolePlayer { role,
   relationship_type, span }`) or widening `at_unsupported`'s parameter to
   `impl Into<String>`/`Cow<'static, str>` would have been more honest than
   either of the two options actually on the table — though note widening
   `at_unsupported` alone would have been worse than what shipped: its
   `Unsupported` variant's message template reads "{feature} is not
   supported *in the initial graph slice*," which implies temporary
   unavailability, and a null role player is not "not yet supported," it can
   never mean anything. Between the controller's non-compiling suggestion,
   the temporariness-implying `Unsupported` path, and the shipped
   `RoleTargetTypeViolation` reuse, the implementer picked the option that
   reads correctly to a user, at the cost of the variant name no longer
   perfectly describing the failure. I rate this Minor rather than
   Important because the displayed message is correct and no downstream
   code depends on the variant's semantics — but it is a real defect worth
   a follow-up cleanup (a two-line new variant), not a shrug.

5. **Informational, not a defect — `RoledCatalog` binds the `SET` target
   with no `MATCH` at all.** The implementer flagged this themselves
   (Correction H asked exactly this question): CREATE registers `x`'s entity
   binding before SET resolves it in the same bind pass, so
   `a_role_update_rejects_a_player_of_the_wrong_type` combines `CREATE ...
   SET [x](scribe: t)` in one statement with no MATCH. This is a
   consequence of `RoledCatalog`'s bind-only, no-database design (inherited
   from Task 13a), not something Task 15 could or should have changed. The
   test still exercises the real `bind_role_player` target-type path
   correctly.

### Verified negatives (no finding)

- **Parameters, not interpolation.** Every player value flows through
  `run_ignore(connection, &sql_with_$placeholders, parameters, &internal)`
  with values in the `internal: HashMap<String, Value>` — never
  string-formatted into SQL. Confirmed by reading the full executor arm.
- **`ir::Mutation` ripple.** `execute_operation` in
  `graph/frontend/src/mutation.rs` has exactly 11 arms for the 11 `Mutation`
  variants, no wildcard — `SetRoles` got a real arm. The only other
  reference to the enum outside test code is `attach_merge_actions` in
  `binder.rs`, whose `_ => {}` wildcard is pre-existing (matches only
  `MergeNode`/`MergeRelation` to attach `ON CREATE`/`ON MATCH` actions,
  unrelated to this task) — confirmed by reading it; not a regression.
- **Grammar regression.** `cargo test -p turso_graph_cypher` (23 tests, incl.
  `parses_entity_set_forms`) and `cargo test -p turso_graph_frontend` (157 +
  4 + 12 + 6 + 16 + 7 + 75 + 11 + 1 + 13 passed, 3 ignored, 0 failed) both
  pass unmodified at `f26ee9db7`.
- **`bind_role_player` extraction genuinely unifies both paths.** Reverting
  its target-type check to a no-op fails both
  `a_role_rejects_a_player_of_the_wrong_type` (create path) and
  `a_role_update_rejects_a_player_of_the_wrong_type` (update path) — a single
  shared bug surface, not two copies that happen to agree today — satisfies
  Correction E's intent.
- **`One`-branch `UPDATE` is load-bearing.** Disabling it fails
  `a_single_valued_role_can_be_repointed_after_create`.
- **YAGNI.** No speculative fields, no unused error variants, no new
  abstractions beyond what the brief asked for (`bind_role_player` is the
  one new helper, and it is used by both call sites, not speculative).

## ⚠️ Cannot verify from diff

None. Everything needed (grammar, AST, IR, binder, executor, tests) is in
this diff or in surrounding code I could read directly; no requirement
depended on code outside `review-c8c859820..f26ee9db7.diff` that I could not
inspect.
