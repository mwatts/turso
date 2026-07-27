# Task 16 report: role-edge read sugar

Commit: `3205e8309570cee0d2a0be90668e2952585da1f2` on `feature/graph-nary`.

## Step 1: the two resolution rules

**Rule A — when is a bare `(x:Name)` a relation anchor?**

`Name` resolves as a node label whenever `catalog.label(graph, name)` returns
`Some` — unchanged, first, always. Only when `Name` is *not* a node label may
it resolve as a relationship type and produce a relation anchor. If `Name` is
both a node label and a relationship type, the node reading wins silently.

Adopted the conservative reading verbatim from the brief; I did not find a
reason to argue for a different rule. The justification is the same one the
brief states for Rule B: adding a relationship type to the schema must never
change what an existing node query returns. Checking `label` first,
unconditionally, is the only ordering under which that holds — anything that
tries `relationship_type` first (or that treats "found in either" as
ambiguous) makes an unrelated schema addition capable of turning an existing
`(x:SomeLabel)` into an error or a different plan.

Implementation: in `bind_start_node` (`binder.rs`), for a node pattern with
exactly one label, if `catalog.label` returns `None` for it *and*
`catalog.relationship_type` returns `Some`, bind a relation anchor instead of
a node. Multiple labels (`(x:A:B)`) never take this path — a relation has
exactly one type, so there is no "which failed label might be a relation
type" question to ask; it falls through to the existing `resolve_labels`
error path unchanged.

I also checked whether the mid-path node binder (`bind_path`'s per-step
loop, `binder.rs:3352` in the brief's line numbers) needs the same Rule-A
treatment. It does not: every expansion operator in this IR
(`GraphExpand`, `RoleExpand`) always produces a `Node` target as `to`. There
is no plan operator that expands *into* a relation. So a relation binding can
only ever occur as a path's first `from`, straight out of `bind_start_node`
(or, after this change, `bind_relation_anchor`) — never at a later step. Rule
A is therefore only needed once, in the anchor position.

**Rule B — role or relationship type after the `:`?**

The name after `-[:` is a role only when the source binding (`from`) is
already a relation. From a node it stays a relationship type, unchanged. If
the name is both a role of that relation's type and a relationship type
elsewhere in the graph, refuse as ambiguous rather than guess — an
`AmbiguousRoleName` error, added exactly as the brief specifies.

Implementation: at the top of `bind_path`'s per-step loop, if
`self.entities.get(&from).map(|e| e.kind) == Some(CatalogEntity::Relationship)`,
dispatch to the new `bind_role_read_step` instead of the existing
node-to-node expand logic; otherwise fall through unchanged. Inside
`bind_role_read_step`, the role is looked up with `eq_ignore_ascii_case`
(matching `relationship_role`'s own case rule), and the ambiguity check
re-queries `catalog.relationship_type` using the role's own *canonical*
name (`role.name`, the spelling `SemanticRole` carries) rather than the
user's raw-cased input — so a differently-cased arrow that matched the role
case-insensitively cannot dodge the ambiguity check just because its literal
spelling isn't a registered type. Verified by sabotage 3 below.

## Step 4: verbatim pre-fix failure messages

Captured by temporarily `git stash push -- graph/frontend/src/binder.rs`
(isolating only the production-code changes, keeping the new fixture/tests in
place), running the two tests, then `git stash pop` to restore. This is
verifying my own pre-fix state on my own branch, not the banned
"revert-and-check-main" pattern.

```
test an_arrow_from_a_relation_reads_that_relations_role ... FAILED
an arrow from a relation anchor reads the named role: Database(ParseError("unknown label `KNOWS` at byte 9..14"))
```

```
test the_role_arrow_and_the_role_pattern_bind_to_the_same_plan ... FAILED
fixture query must bind: UnknownLabel { name: "KNOWS", span_start: 9, span_end: 14 }
```

Both match the brief's measured prediction exactly (`` unknown label `KNOWS` ``,
not the plan's predicted "unknown relationship type") — confirms the failure
is at the anchor (`bind_start_node`/`resolve_labels`), not in the expand path,
as the brief's measured facts said.

## What was implemented and where

All in `graph/frontend/src/binder.rs`:

- `BindError::AmbiguousRoleName` — new error variant, exact wording from the
  brief.
- `bind_relation_scan_anchor` — new shared helper (factored out of
  `bind_match_role_pattern`'s existing anchor-construction logic, behavior
  unchanged for that caller): builds the `RelationScan` and registers the
  relation binding. Used by both `bind_match_role_pattern` (unmodified after
  the extraction, other than calling this helper) and the new
  `bind_relation_anchor`, so the two spellings' anchors cannot drift.
- `bind_relation_anchor` — new method: Rule A's counterpart to
  `bind_match_role_pattern`'s standalone anchor. Calls
  `bind_relation_scan_anchor` then `bind_properties`.
- `bind_start_node` — Rule A check added: for a single-label node pattern,
  try `relationship_type` only if `label` returned `None`; on a hit, delegate
  to `bind_relation_anchor`. Everything else in this function is unchanged.
- `bind_path` — Rule B check added at the top of the per-step loop: if
  `from` is already a `CatalogEntity::Relationship`, delegate to
  `bind_role_read_step` and `continue`; otherwise the existing node-to-node
  logic runs unchanged.
- `bind_role_read_step` — new method: validates the arrow's shape (plain
  `->`, no range, no edge variable, no bracket properties — a role read has
  no separate edge entity), resolves the relationship type via
  `entity_type_names`/`relationship_type` (mirroring the role-update binder's
  existing precedent the brief pointed at, `binder.rs:2304-2325`), resolves
  the role via `relationship_roles`/`eq_ignore_ascii_case`, runs the
  ambiguity check, refuses `Many`-cardinality roles
  (`RoleCardinality::Many`, structural, never by name), binds the player
  (fresh bare-variable node, or a reused node variable with the same
  label/property handling `bind_start_node` uses), and emits the same
  `ir::RoleJoin`/`ir::RolePlayer` plan nodes `bind_match_role_pattern` uses.
  No arity branching, no hardcoded role names, anywhere in this path — roles
  are resolved by matching `SemanticRole.name`/`RoleId`, never by position or
  count.

`graph/frontend/tests/fixture.rs`:

- `ambiguous_session` — new fixture: `KNOWS`/`relationships` with roles
  `start`/`end`/`witness` (`witness` `Many`, spill-table, same shape as
  `witnessed_session`), plus a second, unrelated relationship source
  literally named `witness` over its own table. Registers cleanly (role
  names and relationship-source names are validated in disjoint namespaces —
  confirmed by reading `validate_registration_names`, `catalog.rs:673-721`).
- `bind_witnessed` — new helper: binds a query against `witnessed_session`'s
  real, persisted `SchemaCatalog` registration (via `load_registered_graph`),
  returning `ir::Plan` for equality assertions. Not `bind_fixture`'s stub
  `Catalog`, which is unusable here (see below).

`graph/frontend/tests/nary_relations.rs` — four new tests, detailed next.

## Step 3/6: the four tests and their sabotage results

All four tests pass with the fix in place (`cargo test -p turso_graph_frontend
--test nary_relations`: 31 passed). Full crate suite:
`cargo test -p turso_graph_cypher -p turso_graph_frontend`: 345 passed, 1
ignored (unchanged from before this task).

**1. `an_arrow_from_a_relation_reads_that_relations_role`** — seeds two
`KNOWS` relations sharing `start` (both `1`) but differing in `end` (`2` and
`3`), then reads only `end` through the arrow and asserts both rows come
back (`[[2], [3]]`). A one-row fixture, or reading `start` (the role a buggy
"anchor's first player" fallback would return regardless of which role was
named), could not distinguish "reads the named role" from "returns the
anchor's first player" — this fixture can.

Sabotage 2 (role swap, described below) turns this test red directly: it
returned `[[1], [1]]` (both relations' `start`, i.e. positionally-resolved)
instead of the expected `[[2], [3]]`.

**2. `the_role_arrow_and_the_role_pattern_bind_to_the_same_plan`** — asserts
`ir::Plan` equality (not row- or SQL-equivalence) between
`MATCH (x:KNOWS)-[:start]->(s) RETURN s.id` and
`MATCH [x:KNOWS](start: s) RETURN s.id`, bound separately via
`fixture::bind_witnessed`. This assertion is achievable and passes: both
forms are relation-anchored and label-less (unlike the arrow-vs-role goldens
in `desugaring_golden.rs`, rewritten under a ruling specific to a
*node*-anchored arrow legitimately planning differently from the role form —
that ruling does not transfer here, per the brief, and I did not cite it to
weaken this test).

Sabotage 2 turns this test red directly too: the two plans differ only in
`role: RoleId(2)` (arrow, sabotaged to resolve `end`) vs `role: RoleId(1)`
(role-pattern form, untouched) — the full plans were otherwise identical,
confirming the sabotage's effect was isolated to role resolution.

**3. `a_name_that_is_both_a_role_and_a_relationship_type_is_ambiguous`** —
using `ambiguous_session`, both `MATCH (x:KNOWS)-[:witness]->(w) RETURN w.id`
and the differently-cased `-[:Witness]->` variant are asserted to fail with a
message containing "role" and "relationship type" (the `AmbiguousRoleName`
wording).

Sabotage 3 (below) turns this test red, though I want to flag a wrinkle: see
"Concerns."

**4. `the_role_arrow_is_only_available_from_a_relation_binding`** — regression
guard. `MATCH (p:Person)-[:start]->(s) RETURN s.id` already fails today,
unmodified, with `` unknown relationship type `start` ``; this needs no new
production code, and I did not add any for it. I did strengthen its
assertion beyond a bare `contains("relationship type")` (see sabotage 4
below for why) to also require the message name `start` specifically and
*not* name the anchor's own label (`Person`).

### Sabotage 1 — break Rule A's ordering

Changed `bind_start_node`'s check to try `relationship_type` unconditionally,
without first checking whether `label` returned `None`. Ran
`cargo test -p turso_graph_cypher -p turso_graph_frontend`:

```
test result: FAILED. 11 passed; 1 failed; 1 ignored
thread 'role_lowering_emits_byte_identical_sql_for_a_two_role_relation' panicked at graph/frontend/tests/dialect_alignment.rs:551:6:
query must bind: Unsupported { feature: "naming the edge of a role arrow", span_start: 16, span_end: 28 }
```

This is an *existing* test (`dialect_alignment.rs`), not one of my four —
but it is a real, if incidental, guard: its `BinaryCatalog` stub returns
`Some` from both `label` and `relationship_type` for every name (by
construction — same pattern the brief warned `bind_fixture`'s stub has), so
under the correct code `label` always wins and `(a:Person)` binds as a node,
as intended; under the sabotage, `relationship_type` is now checked instead
and *every* node pattern in that golden-SQL test resolves as a relation
anchor, breaking a query that names its edge (`-[r:KNOWS]->`, which a role
arrow can't do). Since a test did go red, per the brief's instruction ("if no
test goes red... add a test that does") I did not add a further, more direct
test for this — see Concerns for why I judged this sufficient but not fully
satisfying.

Reverted; confirmed `cargo test -p turso_graph_cypher -p turso_graph_frontend`
back to 345 passed, 1 ignored.

### Sabotage 2 — swap the resolved role

In `bind_role_read_step`, swapped the role-name lookup so `start`↔`end` are
exchanged before the `.find(...)` call. Ran the full suite:

```
thread 'an_arrow_from_a_relation_reads_that_relations_role' panicked:
assertion `left == right` failed: both relations' `end` players must be returned...
  left: [[Numeric(Integer(1))], [Numeric(Integer(1))]]
 right: [[Numeric(Integer(2))], [Numeric(Integer(3))]]

thread 'the_role_arrow_and_the_role_pattern_bind_to_the_same_plan' panicked:
assertion `left == right` failed: both spellings are relation-anchored...
  left:  ... RoleJoin(RoleJoin { ..., role: RoleId(2), ... })
 right: ... RoleJoin(RoleJoin { ..., role: RoleId(1), ... })
```

Both Step 3 tests go red, as required. Reverted; suite back to 345 passed, 1
ignored.

### Sabotage 3 — remove the ambiguity check

Deleted the `if self.catalog.relationship_type(self.graph, &role.name).is_some() { return Err(AmbiguousRoleName{..}) }`
block entirely. Ran the full suite:

```
thread 'a_name_that_is_both_a_role_and_a_relationship_type_is_ambiguous' panicked at graph/frontend/tests/nary_relations.rs:1164:5:
Parse error: a role arrow over a Many-cardinality role is not supported in the initial graph slice at byte 15..28
```

The third test does go red, as required. Reverted (re-inserted the block
verbatim); confirmed suite back to 345 passed, 1 ignored.

**Wrinkle, surfaced rather than hidden:** `ambiguous_session`'s `witness`
role is `Many`-cardinality (I built it as a copy of `witnessed_session`'s
shape). So with the ambiguity check removed, the query does not "silently
win" as a role read — it still errors, just for the *different*,
pre-existing Many-cardinality-guard reason (which runs immediately after in
the same function). The test still correctly goes red under this sabotage,
but it is not a clean, single-cause signal for the ambiguity check
specifically; a change that broke *both* the ambiguity check and the
Many-cardinality guard at once, or a future refactor that reordered them,
could pass this test while the ambiguity check is silently gone, so long as
the Many-cardinality guard still fires first.

I tried to fix this by making `ambiguous_session`'s `witness` role
`One`-cardinality with its own column instead, to isolate the two guards.
That attempt uncovered what looks like an unrelated, pre-existing issue: with
`witness` as a `One`-cardinality role sharing its name with a second
relationship source, the ambiguity check's own `catalog.relationship_type`
lookup on the canonical role name silently returned `None` where it should
have returned `Some` — the query bound and ran to `Ok(vec![])` instead of
erroring at all, even *without* my sabotage. I did not chase this further:
it did not reproduce with the `Many`-cardinality role the brief's own
fixture recipe describes (which is what I kept), it is not something Task 16
introduces or needs to fix, and going deeper risked a rabbit hole outside
this task's scope. I reverted that fixture change back to `Many`-cardinality
(matching the brief's "follow `witnessed_session` as the template"
instruction) and left the test as originally written, entangled with the
Many-cardinality guard as described above. Flagging this for the controller
in case it is worth its own investigation later — see Concerns.

### Sabotage 4 — delete Rule B's relation-binding guard

Changed `bind_path`'s dispatch condition from checking
`CatalogEntity::Relationship` to an unconditional `if true`, so a node
source also tries role resolution. Ran the full suite:

```
test result: FAILED. 138 passed; 21 failed
```

21 tests failed — 20 pre-existing tests across `binder.rs`, `compiler.rs`,
`graph_expand.rs`, `schema_catalog.rs`, and `session.rs` (every one of them
exercising an ordinary node-to-node arrow, now broken because it's forced
through role-read validation instead), plus:

```
thread 'the_role_arrow_is_only_available_from_a_relation_binding' panicked at graph/frontend/tests/nary_relations.rs:1213:5:
Parse error: unknown relationship type `Person` at byte 16..27
```

The fourth test goes red, as required — but this run is *why* I strengthened
its assertion beyond a bare `contains("relationship type")`. The sabotaged
error message above still contains that substring (it names `Person`, the
anchor's own label, misused as a fake relationship-type name by the removed
guard, instead of `start`) — a weaker assertion would not have gone red here.
The final assertion checks for `start` specifically and explicitly asserts
`Person` is *absent*, so it distinguishes the correct failure
(`` unknown relationship type `start` ``) from the sabotaged one
(`` unknown relationship type `Person` ``).

Reverted; confirmed `cargo test -p turso_graph_cypher -p turso_graph_frontend`
back to 345 passed, 1 ignored, and `git diff --stat` showing only the three
intended files (`binder.rs`, `fixture.rs`, `nary_relations.rs`) with no
sabotage residue.

## Step 7: gate results

- `cargo fmt` — ran clean (reformatted import line-wrapping in `binder.rs`
  and `fixture.rs`; no logic changes). Rebuilt and re-ran the suite
  afterward: still 345 passed, 1 ignored.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
  — exit 0, 0 errors. The 10 "warnings" the summary line reported are
  build-tool `ar` invocation warnings from an unrelated native extension
  crate (`limbo_sqlite_test_ext`) on this machine's Xcode toolchain, not
  lint warnings; nothing lint-level was flagged.
- `cargo test -p turso_graph_cypher -p turso_graph_frontend` — 345 passed, 1
  ignored (15 suites).
- `mise run corpus` (release build) — per-suite results, read from the
  appended `graph/test-results/runs.jsonl` line for this run:
  - `age-deep`: **3042** passed (baseline 3042 — exact)
  - `cqlite-deep`: **113** passed (baseline 113 — exact)
  - `grafeo-deep`: **277** passed (baseline 277 — exact)
  - `sparrowdb-deep`: **2164** passed (baseline 2164 — exact)
  - `tck-deep`: **3331** passed (baseline range 3329-3332 — within flaky
    tolerance)

  All non-`tck` suites are exactly at baseline; no existing node query's
  meaning shifted. (Not reporting a total, per the brief's explicit
  instruction that the total is not a meaningful number.)
- `mise run cypherbench-sample` (release build) — all 7 domains
  byte-identical to the recorded baseline in `graph/test-results/benchmarks.jsonl`,
  0 errored in every domain:
  - company: matched=13, mismatched=12, errored=0
  - fictional_character: matched=14, mismatched=11, errored=0
  - flight_accident: matched=24, mismatched=1, errored=0
  - geography: matched=11, mismatched=14, errored=0
  - movie: matched=6, mismatched=19, errored=0
  - nba: matched=25, mismatched=0, errored=0
  - politics: matched=15, mismatched=10, errored=0

  (The nonzero `mismatched` counts are a pre-existing baseline, unrelated to
  this task — they matched the prior recorded run exactly, which is the
  relevant comparison; `errored=0` everywhere is the signal that no query
  newly failed to bind or execute.)

## Concerns

1. **Sabotage 1's guard is incidental, not deliberate.** The only test that
   caught breaking Rule A's ordering is an existing golden-SQL test whose
   catalog stub happens to make `label` and `relationship_type` both always
   return `Some`. It is a real signal, but it is fragile — a future change
   to that stub (e.g. making it more realistic) could silently remove this
   coverage. Per the brief's literal instruction I did not add a new test
   since one did go red, but a purpose-built regression test for this rule
   (a fixture where one name is genuinely both a valid node label and a
   valid relationship type) would be a more durable guard if the controller
   wants one.
2. **Sabotage 3's third test is entangled with the Many-cardinality guard**,
   as detailed above: `ambiguous_session`'s `witness` role is `Many`, so
   removing only the ambiguity check still errors, for the unrelated
   Many-cardinality reason, rather than "silently winning." The test still
   goes red under the specified sabotage, satisfying the letter of Step 6,
   but a compound regression that removed both guards at once would not be
   caught as cleanly.
3. **Possible unrelated catalog issue, not fixed, not chased further:**
   attempting to isolate concern 2 by giving `ambiguous_session`'s `witness`
   role `One` cardinality with its own column (instead of `Many`/spill-table)
   caused the ambiguity check's `catalog.relationship_type(self.graph, &role.name)`
   lookup to return `None` where it should return `Some` — the query bound
   and executed successfully instead of erroring, even without any sabotage.
   I reverted to the `Many`-cardinality fixture (matching the brief's
   template) rather than investigate, since it's outside Task 16's scope and
   doesn't reproduce with the fixture shape the brief specifies. Flagging in
   case it indicates a real gap in how `SchemaCatalog` resolves relationship
   types that share a name with a `One`-cardinality role of an unrelated
   relationship type.
4. **Many-cardinality role hop-through is out of scope, as instructed**, and
   does not fall out for free: `bind_role_read_step` explicitly refuses it
   (`RoleCardinality::Many` check, structural, never by name) with the same
   reasoning `bind_match_role_pattern` already uses, deferring to Task 14b.

## Fix round 1

Reviewed at `.superpowers/sdd/2026-07-25-native-nary-relationships-turso/task-16-review.md`.
Three findings fixed, two minor items also fixed. All changes are test- and
fixture-only (`graph/frontend/tests/fixture.rs`,
`graph/frontend/tests/nary_relations.rs`) plus a comment-only dedup in
`graph/frontend/src/binder.rs` — no production code *path* changed, so
`mise run corpus` / `mise run cypherbench-sample` were not rerun this round
(see "Gate scope" below for the justification).

### Critical: Many-cardinality refusal in `bind_role_read_step` had zero coverage

Added `a_role_arrow_over_a_many_cardinality_role_is_refused` in
`nary_relations.rs`, against `fixture::witnessed_session()` (whose `witness`
role is `Many`-cardinality with no colliding relationship-type name, so
nothing else in `bind_role_read_step` can produce a failure for this query):

```rust
let error = session
    .query(
        "MATCH (x:KNOWS)-[:witness]->(w) RETURN w.id",
        &Parameters::new(),
    )
    .expect_err("a role arrow over a Many-cardinality role must not bind");
assert!(
    error
        .to_string()
        .contains("a role arrow over a Many-cardinality role"),
    "unexpected error: {error}"
);
```

Verified by disabling the check at `binder.rs:3829`
(`if role.cardinality == ir::RoleCardinality::Many` → `if false && role.cardinality == ...`)
and running the new test alone. **First attempt's assertion was too loose**
(`contains("Many-cardinality role")`, no `"role arrow over"` prefix) and
passed even with the guard disabled:

```
cargo test: 1 passed, 32 filtered out (1 suite, 0.04s)
```

Root cause: `lowering.rs:1583` (`lower_role_join`) has its own,
differently-worded defense-in-depth guard for the exact same condition
(`"MATCH role pattern join through a Many-cardinality role"`), explicitly
commented as a fallback "in case the binder guard is bypassed" — its message
also contains the substring `"Many-cardinality role"`, so the loose
assertion could not tell which guard had fired. Tightened the assertion to
the binder-specific phrase (`"a role arrow over a Many-cardinality role"`,
not present in the lowering guard's message) and reran with the guard still
disabled — **red**:

```
thread 'a_role_arrow_over_a_many_cardinality_role_is_refused' panicked at graph/frontend/tests/nary_relations.rs:1167:5:
unexpected error: Parse error: unsupported relational graph operator: MATCH role pattern join through a Many-cardinality role
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 32 filtered out; finished in 0.04s
```

Restored the guard and reran — **green**:

```
cargo test: 1 passed, 32 filtered out (1 suite, 0.04s)
```

This also means the original Step-4 gate result the reviewer quoted (345
passed with the guard disabled, "identical to baseline") was real but
incomplete evidence of an actual gap — the binder guard genuinely had no
direct test — but the *behavior* was never silently wrong: a bypass of the
binder guard alone still gets caught by `lowering.rs`'s independent check.
Both guards are now exercised: the lowering one indirectly (via this test
under sabotage, above), the binder one directly (this test at baseline).

### Important: Rule A's ordering guard had no purpose-built test

Added `DualNameCatalog`, a minimal hand-rolled `GraphCatalogSnapshot` where
exactly one name (`"Ambiguous"`) resolves as *both* a node label and a
relationship type, and every other name resolves as neither — unlike
`dialect_alignment.rs`'s `BinaryCatalog`, which returns `Some` from both
`label()` and `relationship_type()` for *every* name (the accidental-golden
problem the review flagged). Added
`a_name_that_is_both_a_label_and_a_relationship_type_reads_as_a_node`,
binding `MATCH (x:Ambiguous) RETURN x.id` directly (no database) and
asserting the bound plan's anchor is a `NodeScan`, not a `RelationScan`, via
a small depth-first walk (`anchor_is_node_scan`).

Verified by inverting Rule A's check order at `binder.rs:3942-3950`
(removed the `self.catalog.label(...).is_none()` guard, so
`relationship_type` was checked unconditionally, unguarded by label
absence) and running the new test alone — **red**, the plan anchored on a
`RelationScan`:

```
thread 'a_name_that_is_both_a_label_and_a_relationship_type_reads_as_a_node' panicked at graph/frontend/tests/nary_relations.rs:1345:5:
Rule A must read Ambiguous as a node label, not a relation anchor: Plan { kind: Project(Project { input: Plan { kind: RelationScan(RelationScan { ... }), ... }
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 32 filtered out; finished in 0.01s
```

Restored the ordering and reran — **green**:

```
cargo test: 1 passed, 32 filtered out (1 suite, 0.01s)
```

### Important: test 3 did not isolate the ambiguity check

`ambiguous_session`'s `witness` role was `Many`-cardinality, so the same
query that trips the ambiguity check also trips `bind_role_read_step`'s
separate Many-cardinality guard — removing only the ambiguity check left
the query still failing, for the wrong reason, rather than binding
successfully. Changed `ambiguous_session`'s `witness` role to
`One`-cardinality (a real `witness` column added to `relationships`) so the
ambiguity check is the only thing standing between this query and a
successful bind; updated the fixture's and the test's doc comments to state
this explicitly. No other test uses `ambiguous_session`, so this was a
direct edit rather than a new parallel fixture.

Per the coordinator's fourth item, my prior Concern 3 (this same
One-cardinality change appearing to make the ambiguity check's
`catalog.relationship_type` lookup silently return `None`) is treated as
withdrawn: the reviewer could not reproduce it across three variants plus
direct `eprintln!` instrumentation. It did not recur here either — the
lookup fired correctly on every run below.

Verified by disabling the ambiguity check at `binder.rs:3813`
(`if self.catalog.relationship_type(...).is_some()` →
`if false && self.catalog.relationship_type(...).is_some()`) and running
the restructured test alone — **red**, the query bound and ran (returning
an empty row set) instead of erroring:

```
thread 'a_name_that_is_both_a_role_and_a_relationship_type_is_ambiguous' panicked at graph/frontend/tests/nary_relations.rs:1217:10:
witness is ambiguous between a role of KNOWS and a relationship type: []
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 32 filtered out; finished in 0.04s
```

Restored the check and reran — **green**:

```
cargo test: 1 passed, 32 filtered out (1 suite, 0.04s)
```

### Minor items also fixed

1. Deduplicated the verbatim seven-line comment block preceding the
   ambiguity check in `bind_role_read_step` (`binder.rs`, previously
   repeated at what were lines 3806–3812 and 3813–3819) — comment-only, no
   behavior change.
2. Commit `3205e8309570cee0d2a0be90668e2952585da1f2`'s message overclaims
   precision: it implies sabotage turned each of the four Task 16 tests red
   individually, but Rule A's ordering was in fact caught only by
   `dialect_alignment.rs`'s unrelated `role_lowering_emits_byte_identical_sql_for_a_two_role_relation`
   golden, not by any Task 16 test — that gap is exactly what this round's
   Important finding 2 fixes. Noting the correction here rather than
   amending the existing commit, as instructed.

### Gate scope

All three fixes plus both minor items touch only test/fixture code
(`graph/frontend/tests/fixture.rs`, `graph/frontend/tests/nary_relations.rs`)
and a comment-only deletion in `graph/frontend/src/binder.rs` (confirmed via
`git diff`: 7 lines removed, 0 added, no code). No production code path
changed, so per the coordinator's instruction `mise run corpus` and
`mise run cypherbench-sample` were skipped this round.

Full suite after all fixes: `cargo test -p turso_graph_cypher -p turso_graph_frontend`
→ 347 passed, 1 ignored (15 suites) — 345 baseline + 2 net new tests (the
Many-cardinality and Rule A tests; test 3 was restructured in place, not
added). `cargo fmt -- --check` clean.

`cargo clippy -p turso_graph_frontend --all-features --all-targets -- --deny=warnings`
could not run to completion: `turso_core` (an upstream dependency, not
touched by this round's diff) fails to compile under `--deny=warnings` with
two pre-existing unused-import errors (`core/mvcc/persistent_storage/logical_log.rs:262`,
`core/vdbe/mod.rs:43`), reproduced identically with and without
`--all-features`. Confirmed via `git diff --stat` that neither file is part
of this round's change set. Not fixed, per "surgical changes" — this is an
environment-level blocker that predates and is unrelated to Task 16.
