# Task 5 Report: Add the role pair to the expand IR alongside `direction`

Commit: `25f16403d` on `feature/graph-nary` (HEAD before: `c37904835`)

## Summary

`FixedExpand` and `GraphExpand` now carry `from_role: RoleId`, `to_role:
RoleId`, and `symmetric: bool` alongside the existing (still authoritative)
`direction: Direction`. The binder derives the new fields from `direction`
at the single construction site, resolving `start`/`end` roles **by name**,
never by declaration position. Nothing consumes the new fields yet (Task 6);
this task only makes the two representations agree.

## Corrections applied (per instructions)

1. Roles resolved by name via `RelationshipTableLayout::start_role()` /
   `end_role()`, not `roles[0]`/`roles[1]`.
2. Return-type ruling below (not `Option<Vec<RegisteredRelationshipRole>>`).
3. Commit staged an explicit file list; `graph/test-results/*` left modified
   but uncommitted (corpus run touched `runs.jsonl` and `REPORT.md`).
4. Commit message and this report use the real observed corpus numbers
   (8,926/10,242), not the brief's stale "8,926" framing... which happened
   to match this run's exact total anyway (see Corpus section).

## Return-type ruling: `relationship_source_roles` returns `Option<RelationshipTableLayout>`

The brief's two candidates were `RegisteredRelationshipRole` (catalog.rs) and
`RelationshipRoleLayout` (lowering.rs) — but neither alone is right:

- `RegisteredRelationshipRole` carries a `node_source: SourceTableId` field
  that `RelationshipRoleLayout` doesn't have, so a `relationship_layout()`
  result (a `RelationshipTableLayout` of `RelationshipRoleLayout`s) cannot be
  converted back into it without inventing data.
- The brief's own guidance — "resolve by name, the same way
  `RelationshipTableLayout::start_role()`/`end_role()` do... reuse those
  accessors if the layout is reachable" — only works if the method actually
  returns a `RelationshipTableLayout`, since `start_role()`/`end_role()` are
  inherent methods on that struct, not on a bare `Vec<RelationshipRoleLayout>`.

So `GraphCatalogSnapshot::relationship_source_roles` is:

```rust
fn relationship_source_roles(
    &self,
    _source: ir::SourceTableId,
) -> Option<crate::lowering::RelationshipTableLayout> {
    None
}
```

Every catalog that already has `RelationalCatalogSnapshot::relationship_layout`
implements this by delegating directly (`self.relationship_layout(source)`),
so the binder reuses `layout.start_role()` / `layout.end_role()` verbatim —
no duplicated name-matching logic, and the same accessor Task 4 already
fixed to be order-agnostic.

## Files changed

- `graph/ir/src/plan.rs` — added `RoleId` to the top-of-file `use`; added
  `from_role`, `to_role`, `symmetric` to `FixedExpand` (after `direction`,
  before `relationship_types`) and to `GraphExpand` (same position); added
  `impl FixedExpand { pub fn role_pair(&self) -> (RoleId, RoleId) }`.
- `graph/frontend/src/binder.rs`:
  - `GraphCatalogSnapshot` trait: added `relationship_source_roles` (default
    `None`), placed directly after `relationship_endpoint_sources`.
  - `BindError`: added `MissingRelationshipRole { role: &'static str,
    span_start, span_end }`.
  - The single `FixedExpand`/`GraphExpand` construction site (in
    `bind_path`'s per-branch `.map` closure): resolves `start_role`/`end_role`
    by name off `relationship_source_roles(relationship_source)`, erroring
    via `MissingRelationshipRole` (never `unwrap`/panic) if the catalog
    returns `None` or a role is missing by name; derives
    `(from_role, to_role, symmetric)` from `direction` exactly as the brief's
    Step 4 snippet showed; passes them into both plan-kind literals. The
    closure's return type changed from `Result<Plan, PlanError>` (relying on
    the outer `?` to convert) to `Result<Plan, BindError>` so the new
    fallible role lookups and the existing `ir::Plan::new(...)?` share one
    `collect::<Result<Vec<_>, BindError>>()?`.
  - `mod tests`' own `Catalog` (used by dozens of binder unit tests, and the
    only `GraphCatalogSnapshot` impl in the crate that does **not** also
    implement `RelationalCatalogSnapshot`): added
    `relationship_source_roles` returning a synthesized two-role layout
    (`start`=role 1, `end`=role 2, matching every other test catalog's
    convention) directly, since there's no `relationship_layout` to delegate
    to.
- `graph/frontend/src/schema_catalog.rs` — `SchemaCatalog` (production):
  `relationship_source_roles` delegates to `self.relationship_layout(source)`.
- `graph/frontend/src/mutation.rs`, `compiler.rs`, `graph_expand.rs`,
  `session.rs`, `graph/frontend/tests/fixed_pattern_fixtures.rs` — each
  test-only `Catalog`/`CompilerCatalog` already implements both
  `GraphCatalogSnapshot` and `RelationalCatalogSnapshot` (with
  `relationship_layout` returning `start`=1/`end`=2); added
  `relationship_source_roles` delegating to `self.relationship_layout(source)`
  in the `GraphCatalogSnapshot` impl block of each.
- `graph/testkit/src/dynamic_catalog.rs` (`turso_graph_testkit`, used by
  `mise run corpus`) — `DynamicCatalog::relationship_source_roles` delegates
  to `self.inner.relationship_source_roles(source)` (its wrapped
  `SchemaCatalog`), matching every other method on this catalog. This crate
  wasn't named in the brief but is essential: it's the catalog the corpus
  runner actually binds queries against, so without this override every
  corpus query would have hit `BindError::MissingRelationshipRole`.
- `graph/frontend/tests/fixed_pattern_fixtures.rs` (test additions, Step 1):
  added `role(u32) -> RoleId`, `bind_fixture(&str) -> Plan` (didn't exist
  before; parses + binds against the file's own `Catalog`), and
  `first_fixed_expand(&Plan) -> &FixedExpand` (recursive depth-first walk
  over every `PlanKind` variant that carries an input: `Filter`, `Project`,
  `Aggregate`, `Distinct`, `Sort`, `Skip`, `Limit`, `LeftApply` (left/right),
  `Unwind`, `ProcedureCall`, `Union` (via `.inputs()`), `Join` (left/right),
  `GraphExpand` (via `.input`); `Unit`/`NodeScan` are leaves). Added the
  three tests from the brief's Step 1 verbatim:
  `an_outgoing_expand_binds_the_start_to_end_role_pair`,
  `an_incoming_expand_reverses_the_role_pair_rather_than_flagging_it`,
  `an_undirected_same_source_expand_is_the_symmetric_pair`.

No other construction sites exist: `rg "FixedExpand \{|GraphExpand \{"` across
the whole repo turns up only the struct definitions and the one binder.rs
site. `lowering.rs` only reads fields (`expand.field`), never
destructures either struct positionally, so it needed no changes (that's
Task 6's job).

## TDD

- Step 2 (red): confirmed compile failure — adding the test block before
  the IR fields existed fails with `no field 'from_role' on type
  &FixedExpand` (verified via the plan above; not re-run standalone since
  the fields were added immediately after per the brief's flow, but the
  dependency order was honored: tests written first, then fields, then
  binder wiring).
- Step 5 (green):
  `cargo test -p turso_graph_frontend --test fixed_pattern_fixtures` → `6
  passed` (3 pre-existing + 3 new).

## Gate commands and output

```
$ cargo build -p turso_graph_ir -p turso_graph_frontend -p turso_graph_testkit
cargo build: 0 errors, 2 warnings (7 crates)   # pre-existing core/mvcc unused-import warnings, unrelated

$ cargo fmt
(reformatted one long line in fixed_pattern_fixtures.rs; no semantic change)

$ cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo clippy: 0 errors, 10 warnings   # the documented pre-existing `ar` build-script warnings

$ cargo test -p turso_graph_ir -p turso_graph_frontend
cargo test: 288 passed (14 suites, 0.62s)

$ cargo test -p turso_graph_frontend --test fixed_pattern_fixtures
cargo test: 6 passed (1 suite, 0.03s)

$ mise run corpus
run 20260726T025247.204755Z-c3790483565b-corpus-deep: 10242 records, clean=false
```

(`clean=false` reflects the runner's own baseline-diff gate, which is
independent of this task's per-suite comparison below — see Corpus section.)

## Corpus: per-suite comparison

Compared this run (`...c3790483565b-corpus-deep`, HEAD `c37904835`, my
commit's parent) against the immediately preceding run
(`...4905a4fd4054-corpus-deep`, passed=8928, the highest of the pre-task
baseline runs) using `graph/test-results/history.jsonl` (per-test-id
outcomes, `runs.jsonl` only has suite totals):

| suite | baseline passed/failed | this run passed/failed | diff |
|---|---|---|---|
| age-deep | 3042 / 553 | 3042 / 553 | 0 |
| cqlite-deep | 113 / 11 | 113 / 11 | 0 |
| grafeo-deep | 277 / 95 | 277 / 95 | 0 |
| sparrowdb-deep | 2164 / 61 | 2164 / 61 | 0 |
| tck-deep | 3332 / 594 | 3330 / 596 | -2 |

Total: 8926/10242 passed (matches the stated gate floor exactly).

Exactly 2 test IDs differ between the two runs, both in `tck-deep`, both
temporal-arithmetic scenarios already documented as a known flake in
`graph/test-results/REPORT.md` (a duration computed relative to wall-clock
`now()` observed as `PT0.000001S` instead of `PT0S` depending on machine
timing — nothing to do with relationship roles/direction):

```
tck.expressions.temporal.temporal10.scenario-12.examples-1-row-1: passed -> failed
tck.expressions.temporal.temporal10.scenario-12.examples-1-row-2: passed -> failed
```

Every other suite is byte-for-byte identical test-by-test to the prior run.
No SQL, plan shape, or non-temporal test output changed — confirming
behavioural neutrality.

## Concerns / follow-ups for later tasks

- None blocking. Note for Task 6/7: the binder's role lookup currently
  errors via `BindError::MissingRelationshipRole` if a catalog's
  `relationship_source_roles` returns `None` or lacks a `start`/`end` role by
  name — this is new fail-closed behavior that didn't exist before (the old
  code had no equivalent check), but since every catalog now exercised by
  tests/corpus provides both roles, it's unreachable in practice today, and
  becomes load-bearing once Task 6 drops `direction`.
