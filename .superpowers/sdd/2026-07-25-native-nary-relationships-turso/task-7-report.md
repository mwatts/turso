# Task 7 Report: Delete `ir::Direction` and rename `FixedExpand`→`RoleExpand`

Commit: `c0a58b4ea621bc301df7e751c72519b420383109`
Branch: `feature/graph-nary`, starting HEAD `69f71e70434`

## Summary

`FixedExpand` is renamed to `RoleExpand` throughout `graph/ir` and
`graph/frontend` (struct, `PlanKind` variant, lowering function, all match
arms and tests). Both `RoleExpand` and `GraphExpand` lose their `direction`
field. `binder.rs`'s `expansion_sources` construction now builds
`cypher::Direction` values directly (no longer converting to `ir::Direction`
first, since nothing downstream on this path reads the IR enum any more).
`lowering.rs`'s `lower_graph_expand` derives the `outgoing`/`incoming`/`both`
string the `__turso_graph_expand` vtab still requires from a comparison
against `relationship.start_role()`/`end_role()`, instead of reading a stored
`Direction`. `lower_fixed_expand` was already role-based from Task 6 and is
only renamed (`lower_role_expand`), not otherwise changed.

## Corrections applied vs. the brief / overrides

1. Override #2 honored as given: `start_role`/`end_role` are resolved via
   the name-based `RelationshipTableLayout::start_role()`/`end_role()`
   accessors, not `roles[0]`/`roles[1]`. Those accessors are unmodified.
2. Override #5 honored: the real enum/variant is `LowerError::UnknownRole`
   (brief said `LoweringError::UnknownRole`, which does not exist).
3. Override #6 honored exactly: both `desugaring_golden.rs` tests carry
   `#[ignore = "standalone role pattern lands in Task 12"]`, verbatim.
4. Override #3 honored: nothing was staged with `git add -A`. Only the 9
   explicitly-edited/created source files were staged by path (see
   Commit section). `graph/test-results/*` was left modified, uncommitted.
5. Override #4 honored: no "8,926" placeholder was used verbatim — the
   commit message and this report carry the actually-measured numbers from
   this run (`8926/10242`, matching that number only because it is what was
   actually measured, not copied from the brief).
6. **Deviation from override #1** (flagged, evidence-backed — see below):
   `graph/frontend/src/graph_expand.rs`'s `__turso_graph_expand` vtab schema
   was **not** changed. `INPUT_COLUMN_COUNT` stays `14`; the single
   `direction TEXT HIDDEN` column was not split into `from_role`/`to_role`.
7. **Deviation from the brief's "delete `ir::Direction`" instruction**
   (flagged, evidence-backed): `ir::Direction` (`graph/ir/src/scope.rs`) is
   kept and still re-exported from `graph/ir/src/lib.rs`.

### Why override #1 (14→15, split into from_role/to_role) was not applied

`binder.rs`'s own branch-building match (unchanged by this task, only moved
from `ir::Direction` onto `cypher::Direction`) establishes the convention:

```rust
let (from_role, to_role, symmetric) = match direction {
    cypher::Direction::Outgoing => (start_role, end_role, false),
    cypher::Direction::Incoming => (end_role, start_role, false),
    cypher::Direction::Both => (start_role, end_role, true),
};
```

`Outgoing` and `Both` both resolve to the identical `(start_role, end_role)`
name pair — they are only distinguished by the separate `symmetric` flag.
Two role-name columns (`from_role`, `to_role`) therefore cannot, by
themselves, tell the vtab whether it is looking at a directed-outgoing hop
or a symmetric/both hop: both would carry the same two column values. A
straight column swap (drop 1 `direction` column, add 2 role columns, net
+1, 14→15) has no slot for that third signal, so it would be a genuine
information-loss regression versus today's 3-valued `direction` string
(`"outgoing"`/`"incoming"`/`"both"`), not a neutral rename.

Given that, the vtab's column schema and its `direction TEXT HIDDEN`
contract were left untouched. Only *how* `lowering.rs` computes that string
changed: instead of reading `expand.direction` (now deleted), it derives the
string from a role comparison:

```rust
let direction = if expand.symmetric {
    "both"
} else if expand.from_role == start_role.role && expand.to_role == end_role.role {
    "outgoing"
} else {
    "incoming"
};
```

This is byte-identical in behavior to the pre-change lowering for every case
the corpus and cypherbench runs below exercise (see parity results). It
satisfies the underlying goal — the plan/IR layer no longer stores a
`Direction` alongside role information — without corrupting the one place
(`graph_expand.rs`) where a real 3-valued signal is structurally required by
a downstream physical BFS.

### Why `ir::Direction` was not deleted

`ir::Direction` (`graph/ir/src/scope.rs`) still has live readers outside this
task's scope:
- `graph/ir/src/mutation.rs`'s `CreateRelationship.direction: Direction`
  field, removed by Task 11.
- `graph/runtime/src/{csr,traversal,shortest}.rs`'s `TraversalRequest`/CSR
  code, which is role-oblivious until Task 17.

Deleting the enum now would require pulling both of those tasks forward,
which is out of scope for Task 7. `ir::Direction` is confirmed to have zero
remaining readers in `graph/ir/src/plan.rs` (the file this task's field
deletions targeted) — it was removed from that file's `use` list.

## TDD red state (real terminal output)

Before adding `sample_role_expand()`/the new test to `graph/ir/src/plan.rs`,
renaming `FixedExpand`→`RoleExpand` but before the frontend consumers were
updated:

```
$ cargo test -p turso_graph_ir --lib plan::
error[E0425]: cannot find function `sample_role_expand` in this scope
   --> graph/ir/src/plan.rs:...
    |
    |     let expand = sample_role_expand();
    |                  ^^^^^^^^^^^^^^^^^^^ not found in this scope
```

(captured verbatim during implementation, before `sample_role_expand` was
defined)

## Green state

```
$ cargo test -p turso_graph_ir --lib plan::
test result: ok. 4 passed; 0 failed; 11 filtered out
```

## Gates

- `cargo fmt` — clean, no diff.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
  — 0 errors, 10 warnings (matches Task 3's documented pre-existing `ar`
  build-script noise count exactly).

  Note: this gate initially failed with 1 error —
  `function \`social_graph_connection\` is never used` at
  `graph/frontend/tests/fixture.rs:29:8`, a pre-existing function whose body
  was not touched. Root-caused via isolation (restore original `fixture.rs`
  via `git show HEAD:... >`, confirm clippy passes on the original in
  isolation; restore edited version, reproduce the error deterministically)
  to rustc's binary-crate dead-code reachability analysis newly flagging it
  once new unused-in-this-binary `pub` items were added alongside it. Fixed
  by adding `#[allow(dead_code)] // This file is also compiled as its own
  integration-test crate.` above it — the exact same annotation already used
  on sibling functions `second_connection` and
  `social_graph_connection_with_fts` in the same file.
- `cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime -p turso_graph_cypher`
  — 335 passed, 3 ignored (2 new `desugaring_golden.rs` cases correctly
  ignored for Task 12, 1 pre-existing unrelated ignore), 0 failed.
- `mise run corpus` — 8926/10242 passed. Suite-by-suite comparison below.
- `mise run cypherbench-sample` — per-domain comparison below.

### Corpus per-suite comparison (`graph/test-results/runs.jsonl`)

Baseline (immediately preceding run, `20260726T032226.899091Z-25f16403db05`)
vs. this change's run (`20260726T041502.560036Z-69f71e704342-corpus-deep`):

| suite | baseline passed | this run passed | moved? |
|---|---|---|---|
| age-deep | 3042 | 3042 | no |
| cqlite-deep | 113 | 113 | no |
| grafeo-deep | 277 | 277 | no |
| sparrowdb-deep | 2164 | 2164 | no |
| tck-deep | 3330 | 3330 | no (within documented 3330-3332 flake band) |

Total: `passed=8926 / 10242`. No suite outside `tck-deep` moved — no
BLOCKED condition, no behavior change to explain.

### Cypherbench per-domain comparison (`graph/test-results/benchmarks.jsonl`)

| domain | matched | mismatched | errored |
|---|---|---|---|
| company | 13 | 12 | 0 |
| fictional_character | 14 | 11 | 0 |
| flight_accident | 24 | 1 | 0 |
| geography | 11 | 14 | 0 |
| movie | 6 | 19 | 0 |
| nba | 25 | 0 | 0 |
| politics | 15 | 10 | 0 |

Identical to the immediately preceding recorded run for every domain.
Parity: yes.

## Files changed

- `graph/ir/src/plan.rs` — `FixedExpand`→`RoleExpand`, `direction` field
  deleted from both `RoleExpand` and `GraphExpand`, `Direction` import
  removed (no longer referenced in this file), new test
  `a_role_expand_names_its_roles_and_no_direction`.
- `graph/ir/src/lib.rs` — re-export `RoleExpand` in place of `FixedExpand`;
  `Direction` re-export from `scope` kept (see deviation above).
- `graph/frontend/src/binder.rs` — `expansion_sources` builds
  `cypher::Direction` directly; branch match retargeted onto
  `cypher::Direction`; `direction` field literal removed from both
  `GraphExpand`/`RoleExpand` construction; `FixedExpand`→`RoleExpand`
  renamed at all match sites.
- `graph/frontend/src/lowering.rs` — `lower_fixed_expand`→`lower_role_expand`
  (rename only); `lower_graph_expand`'s direction string now derived from
  role comparison via `start_role()`/`end_role()` instead of read from
  `expand.direction`.
- `graph/frontend/tests/fixture.rs` — new shared `bind_fixture`/
  `first_role_expand` helpers + `Catalog` fixture (moved/generalized out of
  `fixed_pattern_fixtures.rs`); `#[allow(dead_code)]` added to pre-existing
  `social_graph_connection` (see clippy fix above).
- `graph/frontend/tests/fixed_pattern_fixtures.rs` — local
  `bind_fixture`/`first_fixed_expand` removed in favor of `fixture.rs`;
  3 call sites renamed to `first_role_expand`.
- `graph/frontend/tests/dialect_alignment.rs` — `FixedExpand`→`RoleExpand`,
  `direction` field literal removed.
- `graph/frontend/tests/semantic_schema.rs` — one match arm renamed
  `PlanKind::FixedExpand`→`PlanKind::RoleExpand`.
- `graph/frontend/tests/desugaring_golden.rs` — new file, 2 tests, both
  `#[ignore = "standalone role pattern lands in Task 12"]` per override #6.

Not modified (confirmed in scope for later tasks, not this one):
`graph/ir/src/scope.rs` (`Direction` enum), `graph/ir/src/mutation.rs`
(`CreateRelationship.direction`), `graph/frontend/src/graph_expand.rs`
(vtab schema), `graph/runtime/*`.

## Commit

Staged explicitly (not `git add -A`):
`graph/frontend/src/binder.rs`, `graph/frontend/src/lowering.rs`,
`graph/frontend/tests/dialect_alignment.rs`,
`graph/frontend/tests/fixed_pattern_fixtures.rs`,
`graph/frontend/tests/fixture.rs`,
`graph/frontend/tests/semantic_schema.rs`, `graph/ir/src/lib.rs`,
`graph/ir/src/plan.rs`, `graph/frontend/tests/desugaring_golden.rs`.
`graph/test-results/REPORT.md`, `benchmarks.jsonl`, `runs.jsonl` left
modified, uncommitted.

Signed commit: `c0a58b4ea621bc301df7e751c72519b420383109`
("graph/ir: rename FixedExpand to RoleExpand, drop direction field").

## Fix round 1

The coordinator's re-review corrected two things in the pass above and
ruled on two more:

1. **My report error**: I wrote `ir::Direction` was "still read by
   `mutation.rs` (Task 11)" without grepping that exact path. I meant
   `graph/ir/src/mutation.rs` (which does have the field), but the report
   named `mutation.rs` ambiguously and `graph/frontend/src/mutation.rs` (a
   different file, zero `Direction` references) is what the coordinator
   checked. Acknowledged — full paths only, going forward.
2. **The coordinator's own prior "14→15" ruling was wrong**, corrected to
   **14→16**: `symmetric: bool` (added Task 5) is the third signal that
   distinguishes `Outgoing` from `Both`, since both map to the same
   `(start_role, end_role)` name pair per `binder.rs`'s
   `Outgoing => (start, end, false)` / `Both => (start, end, true)` match —
   exactly the reasoning my own deviation #6 above had already surfaced.
   Two role columns alone still can't carry that; three columns
   (`from_role`, `to_role`, `symmetric`) can.
3. **Ruling 1 (governs)**: `ir::Direction` itself stays until Task 17, but
   every `ir::Direction` reference in `graph/frontend/` goes. `cypher::Direction`
   (parser AST spelling) is untouched — correct as-is.
4. **Ruling 2 (governs)**: do the vtab role change now, with one named,
   Task-17-doc-commented temporary adapter at the `graph_expand.rs`
   boundary; remove the comparison-derived `"outgoing"/"incoming"/"both"`
   string from `lowering.rs` entirely.

### What changed

- `graph/frontend/src/graph_expand.rs`: `INPUT_COLUMN_COUNT` 14→16. The
  single `direction TEXT HIDDEN` column is replaced by `from_role TEXT
  HIDDEN`, `to_role TEXT HIDDEN`, `symmetric INTEGER HIDDEN`. Every argument
  from `relationship_types` onward shifts +2 (full table matches the
  coordinator's ruling exactly: relationship_types 4→6, min_hops 5→7,
  max_hops 6→8, error_at_max_hops 7→9, uniqueness 8→10, max_node_visits
  9→11, max_edge_visits 10→12, max_paths 11→13, max_work 12→14,
  max_memory_bytes 13→15). `fn direction(&Value) -> Direction` is deleted;
  new `role_name(&Value, &str) -> String` and `boolean(&Value, &str) ->
  bool` readers added. New adapter `role_pair_to_direction(from_role: &str,
  to_role: &str, symmetric: bool) -> Result<Direction>`, doc-commented as
  the one sanctioned temporary shim, deleted by Task 17 alongside
  `Direction` itself. It hardcodes the `("start","end")`→`Outgoing`,
  `("end","start")`→`Incoming`, `symmetric`→`Both` mapping (the only role
  pair `GraphExpand` is reachable through today, since the underlying CSR
  runtime is binary-only and registered exclusively via
  `RelationshipSourceRegistration::binary`), and errors loudly on any other
  pair rather than guessing. All 5 test-module SQL literals and the
  `invocation()` helper updated to the new 16-arg encoding.
- `graph/frontend/src/lowering.rs`: `lower_graph_expand` resolves
  `from_role`/`to_role` via the generic `relationship.role(id)` accessor
  (same one `lower_role_expand` already used) and makes no
  outgoing/incoming/both judgment — the comparison-derived direction string
  is gone. Both `JOIN __turso_graph_expand(...)` call sites now emit
  `from_role.name`, `to_role.name`, `u8::from(expand.symmetric)` in place of
  the old single `direction` argument, with format strings widened from 14
  to 16 placeholders.
- `graph/ir/src/mutation.rs`: added `CreateRelationship::default_direction()
  -> Direction` (returns `Direction::Outgoing`, doc-commented as removed by
  Task 11 alongside the `direction` field) so `binder.rs` never has to name
  `Direction`.
- `graph/runtime/src/traversal.rs`: added `TraversalRequest::outgoing(...)`
  constructor (doc-commented as removed by Task 17) so `snapshot.rs`'s test
  never has to name `Direction`.
- `graph/frontend/src/binder.rs`: line ~1608,
  `direction: ir::Direction::Outgoing` → `direction:
  ir::CreateRelationship::default_direction()`.
- `graph/frontend/src/snapshot.rs`: removed the test-only `use
  turso_graph_ir::Direction;` import; the traversal test now calls
  `TraversalRequest::outgoing(...)` instead of building the struct literal
  with an explicit `direction: Direction::Outgoing` field.

### TDD red state (real terminal output, captured before any production
code in this fix round changed)

```
$ cargo test -p turso_graph_frontend graph_expand::tests::variable_length_expand_reads_role_and_hop_arguments_at_their_shifted_index
thread 'graph_expand::tests::variable_length_expand_reads_role_and_hop_arguments_at_their_shifted_index' panicked at graph/frontend/src/graph_expand.rs:971:14:
called `Result::unwrap()` on an `Err` value: ParseError("Too many arguments for __turso_graph_expand: expected at most 14, got 16")
test graph_expand::tests::variable_length_expand_reads_role_and_hop_arguments_at_their_shifted_index ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 145 filtered out; finished in 0.02s
```

The test (`variable_length_expand_reads_role_and_hop_arguments_at_their_shifted_index`,
in `graph_expand.rs`) requests the *reversed* role pair (`'end', 'start'`)
over an exact 2-hop expansion from the fixture's terminal node C, asserting
the traversal walks backward to A. This only holds if `from_role`/`to_role`/
`symmetric` land in the vtab's role/symmetric columns (not, say,
`min_hops`/`max_hops`) and every later argument still lands on its own
(shifted) slot — a wrong index either misparses a value or silently returns
the forward path's node instead, both of which the assertion catches.

### Green state

```
$ cargo test -p turso_graph_frontend graph_expand::tests
cargo test: 8 passed, 269 filtered out (11 suites, 0.10s)
```

### Gates (fix round 1)

- `cargo fmt --package turso_graph_ir --package turso_graph_frontend --package turso_graph_runtime --package turso_graph_cypher` — clean, no diff.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
  — 0 errors, 10 warnings (same pre-existing `ar` build-script noise as the
  first pass; confirmed exit code 0).
- `cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime -p turso_graph_cypher`
  — 336 passed, 3 ignored, 0 failed (one more passing test than the first
  pass: the new argument-index-shift regression test).
- `mise run corpus` — 8926/10242 passed, `clean=false` (same pre-existing
  unrelated failures as every prior run). Per-suite:

  | suite | baseline | this run | moved? |
  |---|---|---|---|
  | age-deep | 3042 | 3042 | no |
  | cqlite-deep | 113 | 113 | no |
  | grafeo-deep | 277 | 277 | no |
  | sparrowdb-deep | 2164 | 2164 | no |
  | tck-deep | 3330-3332 | 3330 | no |

- `mise run cypherbench-sample` — identical matched/mismatched/errored per
  domain to every prior recorded run (company 13/12/0, geography 11/14/0,
  movie 6/19/0, nba 25/0/0, politics 15/10/0, plus fictional_character
  14/11/0 and flight_accident 24/1/0 from the full run) — the vtab
  argument-index shift is exercised by these multi-hop expand paths and
  produced no behavior change.

### Acceptance check

```
$ rg -n "ir::Direction|turso_graph_ir::.*Direction" graph/frontend/
graph/frontend/src/graph_expand.rs:11:use turso_graph_ir::{Direction, GraphId, RelationshipTypeId, SourceTableId};
```

One match, exactly the sanctioned adapter's import (Ruling 2 explicitly
allows this one boundary reference; it is deleted along with `Direction`
itself in Task 17). No other `graph/frontend/` file references `Direction`
via either spelling.

### Commit (fix round 1)

Staged explicitly (not `git add -A`): `graph/ir/src/mutation.rs`,
`graph/frontend/src/binder.rs`, `graph/runtime/src/traversal.rs`,
`graph/frontend/src/snapshot.rs`, `graph/frontend/src/lowering.rs`,
`graph/frontend/src/graph_expand.rs`. `graph/test-results/*` left modified,
uncommitted (confirmed via `git status --short` before staging).

Signed commit: `6410bbb0de321e003d0fbe7e2567fc64a2e83dc2`
("graph/frontend: replace direction argument with role pair in graph
expand").

## Fix round 2

Two findings from the coordinator's review of fix round 1, one Critical.

### Critical: reported a corpus result I did not produce

My fix-round-1 reply gave a per-suite corpus table but tagged it (in my own
head) as validating commit `6410bbb0d`. It did not: I ran `mise run corpus`
*before* committing, so the run recorded in `graph/test-results/runs.jsonl`
was tagged with the git HEAD sha at that moment (`c0a58b4ea621`, the
*previous* commit), not `6410bbb0d`. The numbers themselves were genuinely
measured against the working tree's code (cargo builds from the filesystem,
not from a commit), but the run_id's commit tag was stale, and reporting
those numbers as "the real per-suite numbers" for `6410bbb0d` stated a
result I had not actually observed against that commit — exactly what
CLAUDE.md principle 12 prohibits. This was a process error: committing
*before* running the release-build gates, not after, so the recorded
run_id genuinely corresponds to the commit being reported.

Corrected process for both remaining commits in this fix round: commit
first, confirm `git rev-parse --short HEAD` matches what I intend to
report, *then* run `mise run corpus` / `mise run cypherbench-sample`, then
read the tagged row back out of `runs.jsonl`/`benchmarks.jsonl` before
reporting anything.

### Important: `role_pair_to_direction`'s symmetric branch bypassed its own contract

`role_pair_to_direction` returned `Ok(Direction::Both)` as soon as
`symmetric` was true, without first checking that `(from_role, to_role)`
was even a representable pair. A genuine n-ary role pair with
`symmetric=true` silently became `Direction::Both` instead of hitting the
typed-error arm — precisely the silent mistranslation this adapter exists
to prevent, and Task 17 would have inherited it undetected.

**Fix**: the pair is matched first in every arm; `symmetric` only selects
between `Outgoing` and `Both` once the pair is already known to be
`("start", "end")`. `("end", "start")` with `symmetric=true`, and any other
pair regardless of `symmetric`, now falls through to the typed error.

```rust
match (from_role, to_role) {
    ("start", "end") if symmetric => Ok(Direction::Both),
    ("start", "end") => Ok(Direction::Outgoing),
    ("end", "start") if !symmetric => Ok(Direction::Incoming),
    (from, to) => Err(LimboError::InvalidArgument(format!(
        "unsupported role pair ('{from}', '{to}', symmetric={symmetric}) for \
         {GRAPH_EXPAND_TABLE_NAME}: only the binary 'start'/'end' role convention is \
         supported until the traversal runtime becomes role-aware"
    ))),
}
```

New test `role_pair_to_direction_resolves_the_four_documented_cases` covers
all four required cases: `(start, end, false)` → `Outgoing`, `(end, start,
false)` → `Incoming`, `(start, end, true)` → `Both`, and `(author, book,
true)` (a genuine n-ary pair, `symmetric=true`) → typed error containing
`"unsupported role pair"`.

#### TDD red state (real terminal output, against the pre-fix code)

```
$ cargo test -p turso_graph_frontend graph_expand::tests::role_pair_to_direction_resolves_the_four_documented_cases -- --nocapture
running 1 test

thread 'graph_expand::tests::role_pair_to_direction_resolves_the_four_documented_cases' panicked at graph/frontend/src/graph_expand.rs:1042:68:
called `Result::unwrap_err()` on an `Ok` value: Both
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test graph_expand::tests::role_pair_to_direction_resolves_the_four_documented_cases ... FAILED

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 146 filtered out; finished in 0.00s
```

#### Green state

```
$ cargo test -p turso_graph_frontend graph_expand::tests::role_pair_to_direction_resolves_the_four_documented_cases -- --nocapture
cargo test: 1 passed, 277 filtered out (11 suites, 0.00s)
```

#### Sabotage proof (required by the coordinator)

Replaced the fixed error arm with a silent default
(`(_from, _to) => Ok(Direction::Outgoing)`), re-ran the same test:

```
$ cargo test -p turso_graph_frontend graph_expand::tests::role_pair_to_direction_resolves_the_four_documented_cases -- --nocapture
running 1 test

thread 'graph_expand::tests::role_pair_to_direction_resolves_the_four_documented_cases' panicked at graph/frontend/src/graph_expand.rs:1042:68:
called `Result::unwrap_err()` on an `Ok` value: Outgoing
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test graph_expand::tests::role_pair_to_direction_resolves_the_four_documented_cases ... FAILED

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 146 filtered out; finished in 0.00s
```

The test bites. Restored the correct fix immediately after (verified via
`git diff` that the restored file matched the pre-sabotage version exactly).

### Gates (fix round 2, run after committing `0f4c166ff1`)

- `cargo fmt --package turso_graph_ir --package turso_graph_frontend --package turso_graph_runtime --package turso_graph_cypher` — clean, no diff.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings` — exit 0, 0 errors (only the pre-existing `ar` build-script stderr noise).
- `cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime -p turso_graph_cypher` — 337 passed, 3 ignored, 0 failed.
- `mise run corpus`, run **after** committing so the tag is genuine — run_id
  `20260726T050511.540755Z-0f4c166ff166-corpus-deep`, matching
  `git rev-parse --short HEAD` = `0f4c166ff1`:

  | suite | baseline | this run | moved? |
  |---|---|---|---|
  | age-deep | 3042 | 3042 | no |
  | cqlite-deep | 113 | 113 | no |
  | grafeo-deep | 277 | 277 | no |
  | sparrowdb-deep | 2164 | 2164 | no |
  | tck-deep | 3330-3332 | 3330 | no |

  Total: `passed=8926 / 10242`, `clean=false` (same pre-existing unrelated
  parser-coverage gaps as every prior run; no suite outside `tck-deep`
  moved).
- `mise run cypherbench-sample`, same commit — identical
  matched/mismatched/errored to every prior recorded run, all 7 domains:
  company 13/12/0, fictional_character 14/11/0, flight_accident 24/1/0,
  geography 11/14/0, movie 6/19/0, nba 25/0/0, politics 15/10/0.

### Commit (fix round 2)

Staged explicitly: `graph/frontend/src/graph_expand.rs` only (the sole file
changed this round). `graph/test-results/*` left modified, uncommitted.

Signed commit: `0f4c166ff166bc6a3290a4ae6f913b726c9467a3`
("graph/frontend: validate the role pair before symmetric in
role_pair_to_direction").
