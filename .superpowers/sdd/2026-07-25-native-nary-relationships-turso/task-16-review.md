# Task 16 review: role-edge read sugar

Range reviewed: `3dab1431d..3205e8309` (single commit `3205e8309`, "graph/frontend:
add role-edge read sugar over a relation anchor"). Working tree at HEAD
(`1dd6d5f79`) matches `3205e8309`'s code exactly; the only extra commit is a
test-results record, out of scope here.

Every property below was checked by sabotaging the working tree, running
`cargo test -p turso_graph_cypher -p turso_graph_frontend`, recording the
actual failure, and reverting (confirmed clean via `git diff --stat` /
`git status --short` before moving on). Baseline throughout: **345 passed, 1
ignored (15 suites)**. Where I only reasoned rather than experimented, I say
so explicitly.

---

## Verdict 1 — SPEC COMPLIANCE (against `task-16-brief.md`)

**Step 1 (state the two rules).** ✅ Met. The report adopts the brief's
conservative Rule A verbatim (label wins, checked first, unconditionally) and
states Rule B (role only from a relation binding, ambiguity refused via
`eq_ignore_ascii_case` on the role's *canonical* name). Code matches
(`binder.rs:3949-3957` Rule A, `binder.rs:3243-3249` Rule B dispatch,
`binder.rs:3806-3831` ambiguity check using `role.name` not the raw user
spelling).

**Step 2 (fixtures).** ✅ Met. `ambiguous_session` (`fixture.rs:317-397`)
registers `KNOWS` with a `witness` role alongside a second, literal
`witness` relationship source; `bind_witnessed` (`fixture.rs:499-508`) binds
against a real `SchemaCatalog` via `load_registered_graph`, exactly as
specified, and is used for the plan-equality test.

**Step 3 (four tests).** ⚠️ Partially met. All four tests exist and pass, and
match the brief's required shapes (two-relation fixture distinguishing role
resolution from "anchor's first player", `ir::Plan` equality between the two
spellings, ambiguity with a case-insensitivity check, and the pre-existing
node-anchor regression guard). But test 3
(`a_name_that_is_both_a_role_and_a_relationship_type_is_ambiguous`) does not
cleanly isolate the ambiguity check — see Verdict 2, finding 2. This is a test
that exists and passes, but does not fully test what it claims to.

**Step 4 (verify pre-fix failures).** ✅ Met, and independently reproduced.
I reverted only `binder.rs` to the pre-Task-16 tree (`git show 3dab1431d`) and
re-ran the four tests: three fail with the exact
`` unknown label `KNOWS` at byte 9..14 `` / `UnknownLabel` message the brief
predicted (not the plan's predicted "unknown relationship type"), and the
fourth (`the_role_arrow_is_only_available_from_a_relation_binding`) already
passes with zero new production code — confirming the report's claim that it
needs no fix and is a pure regression guard.

**Step 5 (implement).** ✅ Met. Rule A lives in `bind_start_node`
(`binder.rs:3949`); Rule B's dispatch lives in `bind_path`
(`binder.rs:3243`); `bind_role_read_step` (`binder.rs:3737`) reuses
`relationship_type`/`relationship_roles`/`RolePlayer`/`RoleJoin` rather than
reimplementing role resolution, and `bind_relation_scan_anchor` is shared
between the arrow anchor and the standalone role-pattern anchor so the two
spellings cannot drift (independently confirmed: the plan-equality test
passes and the sabotage that swaps roles changes both spellings' plans
identically). The report's claim that the mid-path node binder never needs
Rule A is structurally sound (every `GraphExpand`/`RoleExpand` step always
produces a `Node` target — confirmed by reading, not sabotaged, since there
is no code path to break). No `roles.len() == 2`, `is_binary`, or hardcoded
`"start"`/`"end"` appears in production code (grepped the full diff). No new
SQL-identifier interpolation was added (this task's new code is entirely at
the IR/binder layer), so the `quoted_identifier` constraint has nothing to
violate here.

**Step 6 (sabotage).** ⚠️ Partially met — three of the four required
sabotages behave exactly as specified; the Rule A ordering sabotage does not.
Full detail in Verdict 2, findings 1, 2, and 4 (an additional gap I found
beyond the brief's four, see finding 3).

**Step 7 (gate).** ✅ Met on what I could verify. `cargo fmt --check -p
turso_graph_frontend` is clean. Full `cargo test -p turso_graph_cypher -p
turso_graph_frontend` is 345 passed/1 ignored, matching the report. Per the
task's instructions I did not re-run `mise run corpus`/`cypherbench-sample`
(already verified by the controller); I did spot-check
`graph/test-results/runs.jsonl` and confirmed the recorded per-suite figures
match the report's claimed numbers exactly (age-deep 3042, cqlite-deep 113,
grafeo-deep 277, sparrowdb-deep 2164, tck-deep 3331). The run's `run_id`
embeds the *parent* commit hash (`3dab1431d5cd`) rather than `3205e8309` —
I checked this is not a Task-16-specific problem: the row recorded for Task
13b's commit (`075676383`) is likewise labeled with *its* parent
(`eab179db4`), so this is a pre-existing, consistent artifact of when the
corpus tool snapshots `HEAD` relative to the commit step, not evidence the
wrong tree was measured. (Reasoned via `git log`/`grep` on `runs.jsonl`, not
sabotaged — there's nothing to sabotage here.) The commit itself touches only
`binder.rs`, `fixture.rs`, `nary_relations.rs` (no `git add -A` residue, no
`graph/test-results/` in this commit).

---

## Verdict 2 — TASK QUALITY

### Critical

**1. The new Many-cardinality refusal in `bind_role_read_step` ships with
zero test coverage — verified by sabotage.** I changed the guard at
`binder.rs:3836` from `if role.cardinality == ir::RoleCardinality::Many` to
`if false && role.cardinality == ir::RoleCardinality::Many` (i.e., disabled
it) and ran the full gate: **345 passed, 1 ignored — identical to baseline.
Not one test went red.** This is the code this task's brief explicitly
required be tested or explicitly justified as deferred (item (d) in the
review brief). The report's Concern 4 says the refusal "does not fall out for
free" and is deferred to Task 14b, and that much is true and is structurally
sound (the check is `role.cardinality`, not the role's name or position — I
confirmed this satisfies "never by name" independent of the sabotage, by
reading `schema_catalog.rs:803-806` where `spill_table` is derived 1:1 from
`cardinality`). But the report does not disclose that the refusal path itself
is completely unexercised: the only query in the whole test suite that
reaches a `Many` role through the arrow (`ambiguous_session`'s `witness`) hits
the *ambiguity* check first (see finding 2) and never reaches this line. A
later refactor that silently deleted this check, inverted it, or merged it
incorrectly with the ambiguity check would be invisible to the entire test
gate. This should be closed with a test that (a) uses a fixture where a
`Many`-cardinality role is *not* also a relationship type name, so the
ambiguity check does not intercept it, and (b) asserts the specific "a role
arrow over a Many-cardinality role" error.

### Important

**2. The Rule A ordering guard is caught only by an unrelated, incidental
existing test — confirmed by sabotage, and none of Task 16's four tests catch
it.** I changed `bind_start_node`'s check from "try `relationship_type` only
if `label` returned `None`" to "always try `relationship_type` first"
(reverting Rule A's ordering) and ran the full gate:

```
test result: FAILED. 11 passed; 1 failed; 1 ignored
role_lowering_emits_byte_identical_sql_for_a_two_role_relation ... FAILED
  query must bind: Unsupported { feature: "naming the edge of a role arrow", ... }
```

I then ran `cargo test -p turso_graph_frontend --test nary_relations` in
isolation under the same sabotage: **all 31 tests passed, including all four
new Task 16 tests.** So the *only* thing this repo's test suite catches when
Rule A's ordering is broken is a pre-existing golden-SQL test
(`dialect_alignment.rs`) whose stub catalog happens to make every name both a
label and a relationship type. Rule A is, per the brief, "the sole thing
preventing this task from changing what existing node queries mean, and the
plan's whole justification rests on it" — and it has no purpose-built
regression test. This matches the implementer's own Concern 1, which is
honest and specific, but the brief's framing ("If it is an accident, that is
at minimum an Important finding") applies directly: this should be closed
with a fixture where one name is genuinely both a valid node label and a
valid relationship type, asserting the node reading wins.

**3. Test 3 does not isolate the property it claims to test — confirmed by
sabotage, matching the report's own disclosure.** I removed only the
ambiguity check (`binder.rs:3820-3831`) and ran
`a_name_that_is_both_a_role_and_a_relationship_type_is_ambiguous` in
isolation: it still fails, but with

```
Parse error: a role arrow over a Many-cardinality role is not supported in the initial graph slice at byte 15..28
```

— i.e. `assert!(message.contains("relationship type"))` fails because the
*wrong* error fired, not because the ambiguity check's absence was detected
directly. `ambiguous_session`'s `witness` role is `Many`-cardinality, so with
the ambiguity check gone the query still errors for the unrelated
Many-cardinality reason immediately afterward in the same function. The test
goes red under the exact sabotage the brief specifies, so it satisfies Step
6's letter, but a compound change that broke both the ambiguity check and the
Many-cardinality guard together (or reordered them) would sail through this
test undetected — the same coverage hole this creates for finding 1. This is
accurately disclosed in the report's Concern 2; I confirmed it holds.

**4. The report's central "silently dead" claim (item (c)) does not
reproduce — I could not confirm it, and my instrumentation points the other
way.** The report says changing `ambiguous_session`'s `witness` role from
`Many` to `One`-cardinality with its own column caused the ambiguity check's
`catalog.relationship_type(self.graph, &role.name)` lookup to "silently
return `None` where it should return `Some`", producing `Ok(vec![])` instead
of an error. I made this exact change three separate ways — (i) a fresh
column (`wit`) on `relationships`, (ii) reusing the existing `dst` column,
(iii) with the fixture otherwise untouched — and in every case the query
still correctly returned `AmbiguousRoleName`. I then instrumented the actual
check in `binder.rs` with an `eprintln!` printing `role.name` and the lookup
result and re-ran: `role.name="witness" lookup=Some(RelationshipTypeId(2))` —
correct, every time. I could not find a precondition under which this check
returns `None` for a name that is genuinely registered as a second
relationship source. This is good news for reliability (I found no evidence
the shipped check is conditionally dead), but it means the report's Concern
3 — flagged as worth "its own investigation" — is an unsubstantiated claim
that does not hold up under direct testing. I was not able to identify what
the implementer's actual experiment differed by; possibilities include a
stale build, an unrelated typo in their WIP edit, or a fixture detail not
described in the report. Given the brief marked this the **highest
priority** item, I recommend the controller not carry this concern forward
as a known gap without independent reproduction, since none was found here.

### Minor

**5. Duplicate comment block, verbatim, in `bind_role_read_step`.**
`binder.rs:3806-3812` and `3813-3819` are the same seven lines of comment
repeated twice, back to back — a copy-paste artifact with no functional
effect. Should be de-duplicated.

**6. The commit message overclaims precision for the Rule A sabotage.** It
reads "verified by sabotage (break Rule A's ordering, swap the resolved
role, drop the ambiguity check, drop the relation-binding guard) to each turn
a specific test red," which reads as "each of Task 16's four tests," but for
Rule A's ordering the test that went red is an unrelated pre-existing golden
test (see finding 2). The report itself is accurate and discloses this; the
commit message is not.

**7. No test exercises Rule A's stated multi-label carve-out.** The report
argues (and the code implements) that `(x:A:B)` never takes the relation-
anchor path since a relation has exactly one type. This is a reasonable,
structurally-argued claim I did not find reason to doubt by reading, but
there is no test pinning a case where one of two labels could plausibly also
be a relationship type name, so a future change to that branch has no
regression guard either. Lower risk than findings 1-2 since the branch is a
narrow `if let [label] = ...` guard, not new dispatch logic.

---

## Summary of what was sabotaged vs. reasoned

| Property | Method | Result |
|---|---|---|
| Rule A ordering guard | Sabotage (reverted) | Confirmed incidental — only an unrelated golden test catches it, not any Task 16 test |
| Role swap (start↔end) | Sabotage (reverted) | Confirmed: both role-reading tests go red exactly as claimed |
| Ambiguity check removal | Sabotage (reverted) | Confirmed: test goes red, but for the Many-cardinality guard's message, not the ambiguity check's absence |
| Node relation-binding guard removal | Sabotage (reverted) | Confirmed: 21 failures including the regression-guard test, matching report exactly |
| Many-cardinality refusal (new, in `bind_role_read_step`) | Sabotage (reverted) | **Not required by the brief's four, but found independently: disabling it causes zero test failures** |
| "One-cardinality witness silently returns None" (report Concern 3 / item (c)) | Sabotage, 3 variants + production-code instrumentation (reverted) | Could not reproduce; lookup and error fired correctly every time |
| Pre-fix failure messages (Step 4) | Reverted `binder.rs` only to pre-Task-16 tree, reran tests | Confirmed exact messages match brief's predictions |
| Mid-path node binder not needing Rule A | Reasoned (structural: every expand operator produces a `Node` target) | No code path exists to sabotage |
| `quoted_identifier` constraint | Reasoned (grep: no new SQL-string interpolation in this diff) | N/A — nothing to violate |
| Corpus/cypherbench run provenance | Reasoned (`git log`/`grep` on `runs.jsonl`, compared to Task 13b's row) | Commit-hash-in-run_id quirk is pre-existing and consistent, not Task-16-specific |
