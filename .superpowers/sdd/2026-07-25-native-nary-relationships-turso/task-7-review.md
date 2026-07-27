# Task 7 Review: Delete `ir::Direction` and rename `FixedExpand` → `RoleExpand`

Reviewed against the brief, judged per Controller Rulings 1 and 2 where they
override the brief. Combined end state of `c0a58b4ea6` (initial attempt) +
`6410bbb0d` (fix round).

## Spec Compliance

- ✅ `ir::RoleExpand` replaces `ir::FixedExpand`; `direction` field gone from
  both `RoleExpand` and `GraphExpand` (`graph/ir/src/plan.rs`).
- ✅ `PlanKind::RoleExpand` replaces `PlanKind::FixedExpand` everywhere
  (`graph/ir/src/plan.rs`, `lib.rs`, `binder.rs`, `lowering.rs`,
  `dialect_alignment.rs`, `semantic_schema.rs`). Grepped
  `FixedExpand|lower_fixed_expand|PlanKind::FixedExpand` across `graph/` —
  zero hits except two historical-naming comments in
  `dialect_alignment.rs:565,577` that predate this task (present verbatim in
  base commit `69f71e704`) and intentionally document the pre-role-based
  function's old name ("SQL recorded ... before role-based lowering replaced
  it") — not stragglers.
- ✅ **Ruling 1 honored**: `ir::Direction` stays defined in
  `graph/ir/src/scope.rs` (unmodified) and re-exported from `lib.rs`. Grepped
  `rg -n "ir::Direction|turso_graph_ir::.*Direction" graph/frontend/` →
  exactly one line, `graph_expand.rs:11`'s `use turso_graph_ir::{Direction,
  ...}` feeding the sanctioned adapter. `binder.rs` and `snapshot.rs`
  confirmed clean (binder.rs builds `cypher::Direction` directly;
  `snapshot.rs`'s test uses the new `TraversalRequest::outgoing()`
  constructor instead of naming `Direction`). `cypher::Direction` references
  in `binder.rs` are untouched, per ruling.
- ✅ **Ruling 2 honored**: `graph_expand.rs`'s vtab schema is now
  `from_role TEXT HIDDEN, to_role TEXT HIDDEN, symmetric INTEGER HIDDEN`
  (graph_expand.rs:151-153), `INPUT_COLUMN_COUNT` 14→16
  (graph_expand.rs:206), one named/doc-commented adapter
  `role_pair_to_direction` (graph_expand.rs:496-513) that is deleted with
  `Direction` in Task 17. Adapter reproduces old `fn direction(value)`
  behavior for the reachable binary cases (`Outgoing`=(start,end,false),
  `Incoming`=(end,start,false), `Both`=(start,end,true) — reconstructed from
  `git show 69f71e704:.../graph_expand.rs` and `binder.rs`'s pre-change
  match) and errors on any other non-symmetric role pair rather than
  panicking or defaulting.
- ✅ Argument-index shift verified index-by-index against
  `git show 69f71e704:graph/frontend/src/graph_expand.rs`: every old
  `args[N]` (relationship_types 4, min_hops 5, max_hops 6,
  error_at_max_hops 7, uniqueness 8, max_node_visits 9, max_edge_visits 10,
  max_paths 11, max_work 12, max_memory_bytes 13) now reads `args[N+2]`, same
  reader fn, same error label. Vtab schema hidden-column order matches the
  `args[]` order exactly; `INPUT_COLUMN_COUNT` (16) equals the actual hidden
  column count (26−11+1=16).
- ✅ No positional role resolution introduced by this diff. `roles[0]`/
  `roles[1]` hits are all in `catalog.rs`/`schema_catalog.rs` test asserts on
  the `RelationshipTableLayout.roles` Vec — pre-existing, untouched by this
  diff (not in the file list of either commit).
- ✅ No binary special case (`roles.len() == 2`, `is_binary`) introduced;
  none found in `graph/` production code.
- ✅ Test hygiene: both `desugaring_golden.rs` tests carry
  `#[ignore = "standalone role pattern lands in Task 12"]` (reason string
  present, findable by Task 13). They are not this task's only tests —
  `fixed_pattern_fixtures.rs`'s three renamed tests, `plan.rs`'s new
  `a_role_expand_names_its_roles_and_no_direction`, and
  `graph_expand.rs`'s new
  `variable_length_expand_reads_role_and_hop_arguments_at_their_shifted_index`
  all run un-ignored.
- ⚠️ **Gate claim cannot be verified — evidence points to not having run**:
  the fix-round commit's (`6410bbb0d`) reported `mise run corpus` result
  (8926/10242, suite table identical to baseline) has **no corresponding
  entry** in `graph/test-results/runs.jsonl` or `history.jsonl` for that
  commit. See Critical finding below.
- ✅ Cargo test claim independently reproduced:
  `cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime -p turso_graph_cypher`
  → `336 passed, 3 ignored` (0.96s), matches the report exactly.
- ✅ Clippy/fmt claims independently reproduced: `cargo fmt --check` clean;
  `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
  finished with only the same pre-existing `ar` build-script warnings, no
  errors.
- ✅ `graph/test-results/*` absent from both task commits (`git show --stat`
  on both — no hits).
- ✅ Both commits signed (`Good "git" signature`), message style
  `scope: lowercase imperative summary` with body explaining intent.

## Sabotage Results (real terminal output)

1. **From_role/to_role index swap** (args[3]↔args[4]): 6 of 8
   `graph_expand.rs` tests failed immediately, including
   `variable_length_expand_reads_role_and_hop_arguments_at_their_shifted_index`.
   The regression test genuinely catches a transposed role-pair index.

2. **Adapter silent-default sabotage**: changed the adapter's wildcard arm
   `(from, to) => Err(...)` to `(_from, _to) => Ok(Direction::Outgoing)` (a
   silent default to Outgoing for an unsupported/genuine n-ary role pair —
   exactly the failure mode the brief says must never happen). Ran
   `cargo test -p turso_graph_frontend` (274 passed, 3 ignored) and
   `cargo test -p turso_graph_frontend graph_expand::tests` (8 passed) —
   **no test failed**. Reverted (`git checkout --`). No test in this task
   exercises the adapter's error path or the `symmetric=true` path at all
   (grepped: no `'... ', ..., 1,` / no err-path assertion anywhere in
   `graph_expand.rs`'s tests). This is a real, demonstrated test gap, not a
   hypothetical one.

3. **Corpus-history cross-check**: `graph/test-results/history.jsonl`'s
   trailing records are all tagged `git_commit=c0a58b4ea621...` (the first
   commit); `grep -c "6410bbb0d" graph/test-results/history.jsonl` → `0`.
   `runs.jsonl`'s last `corpus-deep` row is also tagged
   `c0a58b4ea621-corpus-deep`, timestamped `04:45:47Z`. `6410bbb0d`'s commit
   timestamp is `04:47:46Z` — 2 minutes later. Every other consecutive
   `corpus-deep` run pair in the same file is 25-53 minutes apart (a full
   release-build corpus run is not a 2-minute operation). No corpus run for
   the fix-round commit exists in either ledger. By contrast, two
   `cypherbench` benchmark rows at `04:46:58Z`/`04:47:06Z` (fast, sample
   profile) do land plausibly between the corpus run and the commit, so
   that gate's claim is plausible even though unattributable by commit sha.

## Strengths

- The argument-index-shift verification and its regression test are
  genuinely good work: the test's docstring explains *why* a wrong index is
  dangerous and is silent at compile time, and sabotage confirms it catches
  exactly that class of bug.
- Rulings 1 and 2 are followed precisely, including the exact 14→16 (not
  14→15) correction and the `symmetric` bool as the third signal.
- The two named, doc-commented, Task-17-scoped adapters
  (`role_pair_to_direction`, `CreateRelationship::default_direction`,
  `TraversalRequest::outgoing`) are exactly the "one clearly-named adapter"
  shape Ruling 2 asked for, each with a comment stating what deletes it and
  why.
- The implementer's own report proactively flagged and evidence-backed its
  deviations before the controller ruled on them (the 14→15 vs 14→16
  distinction was correctly reasoned out independently in the first pass).
- Full self-contained rename: no `FixedExpand` stragglers, no new
  `ir::Direction` references beyond the one sanctioned import.

## Issues

### Critical (Must Fix)

- **Fix-round `mise run corpus` gate claim is unverifiable and evidence
  indicates it was not actually re-run against the committed code.** The
  report's fix-round Gates section states "`mise run corpus` — 8926/10242
  passed... Per-suite: [table]" with a table identical to the prior
  (first-commit) run. Neither `runs.jsonl` nor `history.jsonl` contains any
  record tagged with commit `6410bbb0d`, and the ~2-minute gap between the
  last recorded corpus run and this commit is far too short for the
  release-build corpus run to have actually executed (every other
  consecutive pair in the log is 25+ minutes apart). The fix round changed
  production SQL-generation logic (`lowering.rs`'s `__turso_graph_expand`
  call sites) and the vtab's argument encoding (`graph_expand.rs`) —
  precisely the surface `mise run corpus` exists to catch. The report
  presents an apparently copied-forward number as a fresh measurement. Per
  CLAUDE.md Rule 12 ("'Tests pass' is wrong if you skipped any"), this must
  either be genuinely re-run and the real result recorded, or the report
  corrected to say the gate was not re-run for this round and why that's
  safe.

### Important (Should Fix)

- **`role_pair_to_direction`'s `symmetric` branch bypasses role-name
  validation, and no test exercises it.** `graph_expand.rs:501-503`:
  `if symmetric { return Ok(Direction::Both); }` runs before the
  `("start","end")`/`("end","start")` match, so a `symmetric=true` call with
  *any* role-name pair (including a genuine n-ary pair) silently returns
  `Direction::Both` instead of erroring — the exact "silent default" failure
  mode the adapter's own doc comment says it must never do. Confirmed by
  sabotage: replacing the error arm with a silent `Outgoing` default passed
  every test in `turso_graph_frontend` (274 passed) including all 8
  `graph_expand::tests`. Add a role-name check inside the `symmetric` branch
  (matching the same `("start","end")` convention) and a test that exercises
  both the `symmetric=1` success path and an error case for an unsupported
  role pair (symmetric or not).

### Minor (Nice to Have)

- **Initial red-state capture elides the line number.** The report's Step 2
  red-state output shows `--> graph/ir/src/plan.rs:...` where a genuine
  terminal capture would show a real line:column. Labeled "captured verbatim
  during implementation" but the elision undercuts that claim. The fix
  round's red-state capture (graph_expand.rs:971:14, full panic text) is a
  proper verbatim capture and adequately covers this task's TDD-discipline
  requirement, so this is not disqualifying, but it's worth tightening for
  consistency.

## Assessment

**Task quality:** Needs fixes.

**Reasoning:** The IR/rename/adapter work is precise and the highest-risk
item (the argument-index shift) is genuinely tested and sabotage-verified.
However, the fix round's claimed `mise run corpus` result is not supported
by the test-results ledgers and the timing rules it out as a real
re-execution — a required gate silently skipped-but-reported-as-passed on a
commit that changed corpus-sensitive SQL generation. That, plus a real,
demonstrated gap in the adapter's error-path coverage, must be closed before
this task is trusted.
