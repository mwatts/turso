# Task 6 Review: Lower expands through roles

Base: 25f16403d, Head: 04dda3703 (signed, `git verify-commit` → Good signature, mwatts@users.noreply.github.com)

## Spec Compliance

- ✅ Step 1 (failing test): both required tests present in `graph/frontend/tests/dialect_alignment.rs` — `role_lowering_emits_byte_identical_sql_for_a_two_role_relation` (golden fence) and `a_ternary_hop_lowers_through_the_named_role_pair` (red-green driver), plus the `#[ignore]`d `print_binary_sql_goldens` printer as instructed.
- ✅ Step 3 (role match): `lower_fixed_expand` (`graph/frontend/src/lowering.rs:1441-1505` region) now resolves `from_column`/`to_column` via `relationship.role(expand.from_role)` / `.role(expand.to_role)` and matches on `(bound_reference, expand.symmetric)`, four arms, exactly per the brief's shape. `LowerError::UnknownRole { relation, role }` added (brief called it `LoweringError`; the enum is actually named `LowerError` in this codebase — cosmetic naming mismatch in the brief text, not a defect).
- ✅ Step 4 (SQL unchanged): golden test passes; verified independently below — goldens are real, recorded from the pre-change lowering, not reverse-engineered.
- ✅ Step 5 (gates/commit): commit message follows the template closely (scope prefix, imperative summary, body explains intent + implementation note about the six→four arm collapse, `Tests:` trailer with the actual numbers). `git add -A` was correctly NOT used — `git show --stat 04dda3703` contains exactly `graph/frontend/src/lowering.rs` and `graph/frontend/tests/dialect_alignment.rs`. `graph/test-results/*` is left modified and uncommitted, per the report's stated intent (separate controller commit) — confirmed via `git status --short` showing those three files still dirty at HEAD=04dda3703.
- ✅ Interfaces: no new public API; `lower_fixed_expand` was not renamed (Task 7's job, correctly left alone).
- ⚠️ Cannot verify from diff alone: the correctness of collapsing six `(bound, direction)` arms into four `(bound, symmetric)` arms depends on Task 5's binder swapping `from_role`/`to_role` for `Direction::Incoming` (`graph/frontend/src/binder.rs`, committed in 25f16403d, out of this diff). I read that code directly (named risk: cross-task contract) and confirmed `Incoming => (end_role, start_role, false)` — this is exactly what makes the new match reproduce the old `Incoming` arms. Documenting this as a cross-task dependency rather than a gap in Task 6.

No missing, extra, or misunderstood requirements found.

## Verification performed (sabotage, not reading)

1. **Goldens are real.** Built a worktree at 25f16403d (pre-Task-6), overlaid only the new test file, and ran `print_binary_sql_goldens --ignored --nocapture` against the *old* `lower_fixed_expand`. Output was byte-for-byte identical to the four literal strings hardcoded in `expected_binary_sql` (`graph/frontend/tests/dialect_alignment.rs:580-596`). Confirmed: not re-recorded from the new lowering.
2. **Six→four arm mapping.** Reconstructed the old six arms from the diff context and Task 5's `binder.rs` role-swap rule. All six old arms map onto the four new ones exactly as claimed; the symmetric `(Some, true)` and `(None, true)` arms are byte-identical to the old `Both` arms (confirmed empirically by golden query 3 passing). The report's claim about `Direction::Incoming` not being a literal old-code arm-for-arm match, but reconciling through the role swap, is correct and was independently verified against `binder.rs`, not just asserted.
3. **Ternary test is a genuine red-green driver.** Sabotaged `lower_fixed_expand` back to `relationship.start_role()`/`.end_role()` (name-based, direction-style resolution) in a HEAD worktree. Result: `a_ternary_hop_lowers_through_the_named_role_pair` FAILS with `UnknownRole { relation: "transcriptions", role: RoleId(1) }` (real terminal output, captured). Confirms the test cannot pass under direction/name-based lowering.
4. **No binary special case.** Grepped `graph/` for `roles.len()`, `is_binary`, `arity() == 2`. Only hits are test assertions in unrelated files (`schema_catalog.rs:1446`, `catalog.rs:1461/1544/1618/1705`) confirming fixture role counts — none are production branches, none are in this diff's two touched files.
5. **Role resolution by ID, not position.** Reversed the two-role fixture's `roles` vector order (`RoleId(2)`/"end" first, `RoleId(1)`/"start" second) in a HEAD worktree. `role_lowering_emits_byte_identical_sql_for_a_two_role_relation` still passed unchanged — confirms `RelationshipTableLayout::role()` (`lowering.rs:33`, `self.roles.iter().find(|entry| entry.role == role)`) resolves by `RoleId` equality, never by index.
6. **`UnknownRole` reachable, no panic.** Both lookups use `.ok_or_else(|| LowerError::UnknownRole {...})?` (`lowering.rs` new hunk). No `.unwrap()`/`.expect()` introduced on the role-lookup path.
7. **Test hygiene.** Both new tests are behavior-sensitive per sabotage runs 3 and 5 above — neither would pass against a broken implementation. The golden test is explicitly a regression fence per the brief (allowed to pass unconditionally as a fence, not a red/green driver) and is documented as such in its own comment.

Corpus/cypherbench numbers: `graph/test-results/runs.jsonl` line 135 (`20260726T032226.899091Z-25f16403db05-corpus-deep`, recorded 03:22:26 UTC — between commit 25f16403d at 02:55 UTC and commit 04dda3703 at 03:27:31 UTC, consistent with gates running before commit) shows `passed=8926/10242` with per-suite breakdown `age-deep 3042, cqlite-deep 113, grafeo-deep 277, sparrowdb-deep 2164, tck-deep 3330`, identical to the immediately preceding baseline row (line 134, `c3790483565b`). No non-tck suite moved; tck-deep is within the documented 3330-3332 flake band. `graph/test-results/benchmarks.jsonl`'s last row (recorded 03:26:50 UTC) matches the report's per-domain table exactly and is identical to the two preceding runs. Report's numbers are accurate.

## Strengths

- The role-swap-in-binder dependency (Task 5) is correctly leaned on rather than re-derived or duplicated in lowering — no redundant direction-awareness crept back in.
- The report's TDD narrative is unusually rigorous: it caught and explicitly corrected a factual assumption in the brief (the old code's actual failure mode for the ternary case was a typed `MissingSource` error via name lookup, not a "silent wrong join" as the brief's comment speculated) and demonstrated the real terminal output rather than asserting the brief's narrative was correct. This is exactly the kind of verification the process demands, and it held up under independent re-verification.
- Comments added at the point of change (`lowering.rs`) explain the *why* (binary is a layout, not a kind; symmetric is not a `Both` arm) rather than narrating the diff.
- Commit hygiene: explicit `git add` of only the two intended files, matching the report's stated intent, correctly leaving the test-results commit to the controller.

## Issues

### Critical (Must Fix)
None.

### Important (Should Fix)
None.

### Minor (Nice to Have)
- `graph/frontend/src/lowering.rs`, the `(Some(bound), true)` arm's format string contains a long run of literal spaces before `OR` (carried over verbatim from the old `Both` arm). Harmless (matches golden byte-for-byte) but worth a follow-up cleanup pass since it reads as an accidental leftover rather than intentional formatting; not this task's job to fix given the byte-identical-SQL constraint.
- The brief's error-variant name (`LoweringError::UnknownRole`) doesn't match the actual enum name (`LowerError::UnknownRole`) — brief authoring nit, not a code defect; flagging so the plan text can be corrected for later readers.

## Assessment

**Task quality:** Approved

**Reasoning:** Every sabotage check performed produced exactly the expected result (goldens are genuinely pre-change, positional dependence is genuinely absent, the ternary test genuinely requires role-based lowering, no binary special case exists), and the cross-task role-swap dependency on Task 5 was independently confirmed in `binder.rs` rather than taken on faith. The report's corpus/cypherbench claims match `graph/test-results/*` exactly. This is a clean, well-verified contract move with no defects found.
