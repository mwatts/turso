# Task 10 review — write relations from their role bindings

Commit reviewed: `fa67a216c` (range `dffbc8bff..fa67a216c`)

## 1. Spec compliance

Step by step against `task-10-brief.md`:

- **Step 1 (failing test)** — ✅ with a disclosed, necessary adaptation. The
  brief's literal `Session`/`run`/`sql` harness and 4-arg `insert_entity`
  don't exist (pre-corrected in dispatch); the implementer used the real
  `fixture::ternary_session() -> (Arc<Database>, GraphConnection)` +
  `second_connection` + raw SQL convention instead, matching
  `native_capabilities.rs`. Reasonable.
- **Step 2 (verify it fails)** — ✅. Implementer's own sabotage proof
  (reverting to `layout.start_role()/end_role()`) panics against the
  3-role `Transcription` layout, confirming the new writer test can't pass
  under the old binary-only code. I independently corroborate this: the
  ternary layout has no `start`/`end` roles, so any hard-coded start/end
  path is structurally incapable of writing it.
- **Step 3 (derive fixed columns from roles)** — ✅ for the loop itself:
  resolves via `layout.role(binding.role)` (never `roles[i]`), routes
  `One`→`fixed`, `Many`→`spilled`, asserts `spilled.is_empty()`. Preserves
  column order for existing binary relations because the binder always
  emits `roles: vec![start_binding, end_binding]` in that fixed order
  regardless of `layout.roles` declaration order (confirmed at
  `binder.rs:1664-1681`) — the "byte-identical binary" claim checks out.
  `create.from`/`create.to` are now genuinely dead in this function
  (grep-confirmed), correctly left on the IR struct for Task 11 to remove.
  `MutationError::NonIntegerPlayer` was correctly omitted (dead code
  otherwise) — YAGNI applied correctly.
- **Step 4 (verify it passes)** — ✅, reproduced (`1 passed, 2 ignored`).
- **Step 5 (gate/commit)** — ✅ mostly: `cargo fmt -p turso_graph_frontend
  --check` clean; `cargo test -p turso_graph_frontend` reproduces
  `283 passed, 5 ignored`. ⚠️ Cannot independently verify the reported
  full-workspace `cargo clippy --workspace --all-features --all-targets`
  result — my sandbox's `core` crate fails to compile under clippy for
  reasons entirely unrelated to this diff (unused imports in
  `core/mvcc/persistent_storage/logical_log.rs` and `core/vdbe/mod.rs`,
  neither touched by this change), which blocks a workspace-wide run from
  my environment. Not attributable to this task.

- **❌ Violation of the brief's own interface contract.** The brief states
  under Interfaces: *"Produces: no new public API."* The diff adds
  `pub fn execute_create_relation` to `GraphConnection` in
  `graph/frontend/src/session.rs` — a type re-exported at the crate root
  (`pub use session::{..., GraphConnection, ...}`) — i.e. genuine new
  public API of `turso_graph_frontend`, not gated by `#[cfg(test)]` or any
  test-only feature flag (the crate has no such flag; `default = []`,
  only an unrelated `fts` feature exists). See finding 1 below.

- **❌ Global constraint check that fails on inspection.** "Role identity
  is the `RoleId`, never a vector index" is satisfied by the *code*
  (`layout.role(binding.role)`, no `roles[0]`/`roles[1]`), but the
  *enabled test suite does not actually verify it* — see finding 2 (test
  quality) below. Sabotaging positional resolution passes the shipped
  test unchanged.

## 2. Task quality — **not approved**

### Findings

**[Critical] New public production API contradicts the brief's explicit
"no new public API" and ships a documented semantic gap.**
`GraphConnection::execute_create_relation` (+122 lines, `graph/frontend/src/session.rs:415`)
is `pub`, reachable by any downstream consumer of the crate (Python/JS/Go
bindings included, since `GraphConnection` is re-exported at the crate
root). Its own doc comment concedes it "does not record relationship-type
junction membership" — i.e. a caller gets `Ok(identity)` back for a
relationship that is invisible to type-filtered reads
(`MATCH ()-[:Type]->()`). Per this repo's own stated hierarchy ("Crash >
corrupt"), an `Ok` result masking an incomplete write is exactly the worse
failure mode, and it is now part of the crate's stable public surface
rather than confined to test code. The Rust-idiomatic fix (a same-crate
`#[cfg(test)] mod tests` inside `mutation.rs` calling the now-`pub(crate)`
`insert_relationship` directly) would have satisfied the brief's "no new
public API" constraint without needing any new production surface at all;
the external-crate integration-test convention the implementer preserved
here required promoting this to `pub`, and that tradeoff should have been
surfaced, not made silently.

**[Important] The five failure paths inside `execute_create_relation` all
panic (`unwrap_or_else(|| panic!(...))`) on caller-controlled strings**
(unknown relation type, unknown source, unknown layout, unknown role,
unknown property), instead of returning `Result<_, Error>` like every
other public `GraphConnection` method (`execute`, `query`, ...). Since the
method is public, a mistyped role or relation name from any caller crashes
the process instead of surfacing a catchable error.

**[Important] Verified by sabotage: the enabled test suite does not
actually catch the two invariants this task exists to protect.**
- Replaced `layout.role(binding.role)` with positional indexing
  (`layout.roles.get(i)`) — the shipped, non-ignored
  `the_writer_places_each_role_player_in_its_own_column` test **still
  passed**, because it happens to pass roles in the same order
  (`scribe, text, folio`) as they're registered in `ternary_session()`.
  Re-running the identical sabotage with the test's role list reordered
  (`folio, scribe, text` — same values, same expected output) **did**
  fail, confirming the coverage gap is real, not a false alarm. This is
  exactly the "recurring defect class... caught at Tasks 4, 5, 6, 7, 9"
  the dispatch called out, and Task 10's own dedicated test cannot detect
  a regression into it.
- Made same-player-in-two-roles collapse to one column (dedup by value in
  the `fixed` push) — the `nary_relations` suite **still passed in full**;
  the sabotage was only caught incidentally by unrelated, pre-existing
  binary-relationship self-loop tests elsewhere in the workspace. The one
  test that would directly cover this
  (`the_same_player_may_fill_two_roles_of_one_relation`) is `#[ignore]`d.
  Both gaps are avoidable today: `execute_create_relation` bypasses the
  parser already, so a role-order-varied call and a repeated-player call
  could have been added as enabled tests without waiting on Task 12.
  Restored both sabotages; reran `cargo test -p turso_graph_frontend`,
  confirmed clean `283 passed, 5 ignored, 0 failed`.

**[Minor] Removing `assert!(spilled.is_empty(), ...)` has zero effect on
the current suite** (283/5 unchanged) — expected, since nothing before
Task 14 constructs a `Many`-cardinality role binding. Correct as a forward
guard per the brief; flagged only because it is, today, unverifiable by
any test (⚠️ cannot verify from diff/tests — inherent to the phased plan,
not a defect).

**[Minor] `#[ignore]` usage is legitimate, not concealment.** Ran both
ignored tests with `--ignored`: both fail on
`Mutation(Bind(UnknownProperty { name: "id", ... }))` / parser rejection
of the standalone role-pattern syntax — pre-existing gaps explicitly
attributed to Task 12, not to the writer logic under test here. Confirms
implementer's own concern (fixture doesn't seed `Person`/`Text`/`Folio`
rows) is real but inert until Task 12, as reported.

### Working tree

All sabotage edits to `graph/frontend/src/mutation.rs` and
`graph/frontend/tests/nary_relations.rs` were reverted; `git diff` on
those paths is empty. `graph/test-results/REPORT.md` and
`graph/test-results/runs.jsonl` remain modified/uncommitted exactly as
found (untouched by this review).

## Verdicts

1. **Spec compliance: ❌** — the diff adds new public production API
   (`GraphConnection::execute_create_relation`) directly contradicting the
   brief's stated "Produces: no new public API," and the shipped
   non-ignored test does not actually verify the RoleId-not-position
   invariant this task exists to enforce (verified by sabotage).
2. **Task quality: Not approved** — Critical: production-public,
   panic-prone, junction-membership-incomplete write path shipped as a
   test convenience. Important: two of the plan's core invariants
   (role-order independence, repeated-role-player support) are untested by
   anything that currently runs. Fix: make the test helper crate-internal
   (or feature-gated) and return `Result` instead of panicking; add
   enabled tests (via `execute_create_relation`, no parser needed) for
   out-of-order roles and a repeated player.
