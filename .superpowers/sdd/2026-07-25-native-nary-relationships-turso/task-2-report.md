# Task 2 Report: register relationship sources as roles

Commit: `82175eb5eb4d66a404c85a9a5112b5babb206d7d` (signed, on `feature/graph-nary`,
parent `0678787100afbc4d26e0d45942fe85503820fe25`).

## Summary

`RegisteredRelationshipSource` / `RelationshipSourceRegistration` now carry an
ordered `Vec` of named roles (`RoleSourceRegistration` / `RegisteredRelationshipRole`)
instead of hardcoded `start_column`/`end_column`/`start_node_source`/`end_node_source`
fields. `RelationshipSourceRegistration::binary(...)` is a convenience constructor
that builds two `RoleCardinality::One` roles named `start`/`end` — there is no
`if roles.len() == 2` fast path anywhere; a two-role all-required-`One` source
goes through the exact same code paths as any other role count and lands on the
same two indexed endpoint columns plus one composite pair index as before.

`Many`-cardinality roles have no endpoint column on the relationship table;
their players spill to `<table>__<role>(relation_id, node_id)`, indexed both
directions (`install_spill_table`).

## Changes per file

- **`graph/frontend/src/catalog.rs`** (main change, +556/-… net within the
  556 changed lines reported above):
  - Added `RELATIONSHIP_ROLES_TABLE = "__turso_internal_graph_relationship_roles"`.
  - Replaced `RelationshipSourceRegistration`'s endpoint fields with
    `roles: Vec<RoleSourceRegistration>`; added `RoleSourceRegistration` struct
    and `RelationshipSourceRegistration::binary(...)` constructor.
  - Replaced `RegisteredRelationshipSource`'s endpoint fields with
    `roles: Vec<RegisteredRelationshipRole>`; added `RegisteredRelationshipRole`
    struct and `role_by_name`, `role_by_id`, `single_valued_roles`, `spill_table`
    methods.
  - `create_catalog`: `RELATIONSHIP_SOURCES_TABLE` DDL now only has
    `source_id, table_name, identity_column`; added `RELATIONSHIP_ROLES_TABLE`
    DDL (`source_id, ordinal, name COLLATE NOCASE, column_name, node_source_id,
    cardinality CHECK IN ('one','many')`, `PRIMARY KEY(source_id, ordinal)`).
  - `load_registered_graph`: loads roles per source via a per-source query
    ordered by `ordinal`, reconstructing `RoleId` from the stored ordinal.
  - `register_graph_in_transaction`: validates/requires columns built from
    identity + single-valued role columns; per role, resolves the node source,
    inserts into `RELATIONSHIP_ROLES_TABLE`, and dispatches to
    `install_role_index` (cardinality `One`) or `install_spill_table`
    (cardinality `Many`); calls `install_role_pair_indexes` once per source
    after all roles are installed.
  - `validate_registration_names`: validates each role name, rejects duplicate
    role names (case-insensitive) within a source.
  - Index installers replaced: `install_role_index`, `install_role_pair_indexes`
    (iterates unordered pairs of single-valued roles only — one pair for a
    binary source, matching today's shape), `install_spill_table`.
  - Test module: added `RoleCardinality` import, converted `registration()`
    helper and both sources in the multi-source reopen test to
    `::binary(...)`, added three new tests (`a_two_role_registration_lands_on_todays_physical_shape`,
    `a_three_role_registration_indexes_every_role_and_every_ordered_pair`,
    `a_role_must_name_a_registered_node_source`), removed the now-duplicated
    `relationship_endpoints_must_name_registered_node_sources` test and trimmed
    the redundant 3-index assertion out of
    `registration_installs_stable_sources_indexes_and_generation_triggers`.

- **`graph/frontend/src/lib.rs`**: exported `RegisteredRelationshipRole` and
  `RoleSourceRegistration` alongside the existing catalog re-exports.

- **`graph/frontend/src/schema_catalog.rs`** (found via compile errors, not on
  the brief's Step 9 list): `relationship_endpoint_sources`, `relationship_layout`,
  and `payload_columns` now resolve `start`/`end` via `role_by_name` instead of
  direct fields; `payload_columns` generalized to iterate
  `single_valued_roles()` for the structural-columns list. One test
  construction-site literal (was on the brief's list) converted to `::binary`.

- **`graph/frontend/src/semantic.rs`** (compile-error fix, not on the brief's
  list): relationship-type semantic registration now looks up `start`/`end`
  roles via `role_by_name` before building the endpoint check list.

- **`graph/frontend/src/semantic_constraints.rs`** (compile-error fix, not on
  the brief's list): `ResolvedCardinalityConstraint` endpoint-column resolution
  now goes through `role_by_name("start"/"end")`.

- **`graph/frontend/src/snapshot.rs`** (compile-error fix, not on the brief's
  list, plus one listed construction-site literal): resolves `start`/`end`
  roles once per source with an explanatory comment that traversal snapshots
  are binary-only today (n-ary traversal is a later task); all SQL-building
  and endpoint-identity logic now reads `start_role.column` /
  `end_role.column` / `start_role.node_source` / `end_role.node_source`.

- **`graph/frontend/src/dialect.rs`** (runtime-only bug, NOT caught by the
  compiler — found via `rg` for stray field references, not on the brief's
  Step 9 list): `TursoGraphsCursor::load()`'s raw SQL directly selected
  `rs.start_column, rs.end_column`, which no longer exist on
  `RELATIONSHIP_SOURCES_TABLE`. Rewrote as correlated subqueries against
  `RELATIONSHIP_ROLES_TABLE` filtering `name = 'start'`/`'end'`. The public
  `turso_graphs` vtable schema (column names `start_column`/`end_column`) was
  left unchanged — only the query populating it was fixed. One listed
  construction-site literal converted to `::binary`.

- **Construction-site literals converted to `RelationshipSourceRegistration::binary(...)`**
  (brief's Step 9 list, verified all present and updated):
  `graph/frontend/src/graph_expand.rs`, `graph/frontend/src/session.rs` (×2),
  `graph/frontend/benches/semantic_prepare.rs` (×3),
  `graph/frontend/examples/snapshot_profile.rs`,
  `graph/testkit/src/runner.rs`, `tests/integration/multi_frontend_doc.rs`,
  `graph/frontend/tests/native_capabilities.rs`.

- **Construction-site literals found beyond the brief's Step 9 list** (via
  `rg -n "start_column|end_column|start_node_source|end_node_source"` across
  the crate): `graph/frontend/tests/dialect_alignment.rs` (5 occurrences, same
  KNOWS/relationships literal, one `replace_all` edit),
  `graph/frontend/tests/fixture.rs` (1), `graph/frontend/tests/semantic_schema.rs`
  (5 occurrences across distinct fixtures, including two multi-source
  two-relationship lists).

- Confirmed via a final `rg -n "\.start_column|\.end_column|\.start_node_source|\.end_node_source"`
  across `graph/` and `tests/` that no old-shape field references remain
  outside `binary()`'s own parameter names and doc comments.

## Tests

```
cargo test -p turso_graph_frontend --lib catalog::
```
24 passed (includes the 3 new role tests and all retained catalog/schema_catalog tests); 0 failed.

```
cargo test -p turso_graph_frontend
```
All test binaries green: 140, 4, 10, 3, 0(doc), 7, 75, 10, 1, 13 passed across
unit + integration crates = 263 passed, **0 failed**, 0 ignored.

## Merge gates

1. **`cargo fmt`** — ran (full workspace, not `--check`); reformatted several
   touched files (notably `catalog.rs`); structural content verified intact
   post-format via re-read and re-test.
2. **`cargo clippy --workspace --all-features --all-targets -- --deny=warnings`**
   — clean, 0 warnings, 0 errors. (The `#[global_allocator]` conflict I'd
   flagged earlier between `memory-benchmark`/`memory-benchmark-codspeed`/`turso`
   only manifests during linking of a full `build`/`test` binary; clippy's
   `check`-only pass does not link, so it did not surface here. Not
   independently bisected against pre-existing state since it never blocked
   any required gate — noting this as an unresolved-but-moot assumption rather
   than silently dropping it.)
3. **`cargo test -p turso_graph_frontend`** — see Tests above, 0 failed.
4. **`mise run corpus`** — `20260726T005932.453206Z-0678787100af-corpus-deep`:
   **10242 total, 8926 passed, 1316 failed**. Identical to the immediate
   parent-commit baseline run `e068dc04c359` (8926 passed / 1316 failed, same
   per-suite breakdown: age-deep 3042/553, cqlite-deep 113/11, grafeo-deep
   277/95, sparrowdb-deep 2164/61, tck-deep 3330/596). No new failure family;
   meets the ">= 8,926 passed" bar exactly. (Note: the brief's stated baseline
   of "8,927/1,262 at commit 0678787" does not match any observed run_id in
   `runs.jsonl` for that commit hash — the actual baseline run recorded at
   that commit is 8926/1316, which is what this comparison uses.)
5. **`mise run cypherbench-sample`** — per-domain `matched`/`mismatched`/`errored`
   identical to the prior recorded sample run for every domain (company
   13/12/0, fictional_character 14/11/0, flight_accident 24/1/0, geography
   11/14/0, movie 6/19/0, nba 25/0/0, politics 15/10/0); only timing/RSS
   numbers varied, as expected run-to-run. Baseline parity confirmed.

## Anything surprising

- **Runtime-only SQL bug in `dialect.rs`**: the `turso_graphs` vtable's loader
  built a raw SQL string against dropped columns. This was invisible to
  `cargo build` (it's a `format!()` string) and would only have surfaced as a
  runtime "no such column" error the first time that vtable was queried after
  this change — exactly the class of missed call site the brief warned about,
  but harder to catch than a compile error. Found via a manual `rg` sweep for
  the old field names across the whole crate rather than relying on compiler
  errors.
- **Consumer files beyond the brief's Step 9 list**: `schema_catalog.rs`,
  `semantic.rs`, `semantic_constraints.rs`, and `snapshot.rs` all read the old
  endpoint fields and needed `role_by_name` fixes; three additional test files
  (`dialect_alignment.rs`, `fixture.rs`, `semantic_schema.rs`) had construction
  literals not on the brief's list. All found by independent search per the
  task's explicit instruction to verify the call-site list rather than trust
  it blindly.
- **Shared working directory / concurrent corpus writer**: `graph/test-results/{REPORT.md,runs.jsonl,benchmarks.jsonl}`
  had an unrelated run appended (`run_id` tagged with commit `2b3e9362f6a4`,
  not this branch's HEAD or any commit I made) before I ran my own corpus
  pass — evidence of another process running `mise run corpus` against the
  same checked-out tree concurrently. Per repo convention (confirmed via
  `git show --stat -1 8343d8fd4`, a dedicated "graph/test-results: record..."
  commit separate from feature work), I deliberately excluded
  `graph/test-results/*` from this commit via an explicit file-list `git add`
  rather than `git add -A`, so my feature commit carries only the 17 source
  files I intentionally changed. `graph/test-results/*` remains modified but
  uncommitted in the working tree (containing both the foreign run and my own
  legitimate corpus/cypherbench runs) — the caller/user should decide whether
  to commit that separately, discard it, or let the next dedicated
  test-results commit pick it up.
- **Baseline number discrepancy**: the brief stated the baseline as "8,927
  passed / 1,262 failed at commit 0678787," but no run in `runs.jsonl` matches
  that failed count at any commit; the actual recorded run at the parent
  commit and the corresponding run at my commit both show 8926/1316. This
  report uses the observed, verifiable numbers rather than the brief's stated
  figures, which appear stale.

## Fix round 1

Reviewer finding (Important): the `RoleCardinality::Many` / spill-table path
(`install_spill_table`, the `"many"` branch in registration/load) had zero
test coverage — the brief's own Step 1 test list only covered all-`One`
relations. Reviewer confirmed with a throwaway (reverted) test that the path
is correct, so this was an untested branch, not a live bug.

Added one permanent test in `graph/frontend/src/catalog.rs`,
`a_many_role_spills_to_a_side_table_indexed_both_directions_and_is_excluded_from_pair_indexes`,
registering a relation with two `One` roles (`author`, `work`) plus one
`Many` role (`endorsers`). It asserts:

- the `Many` role gets no endpoint column on the relation table (`PRAGMA
  table_info(citations)` shows only `id, author_id, work_id`)
- its spill table is named `citations__endorsers` with columns exactly
  `(relation_id, node_id)`
- the spill table has exactly 2 indexes (both directions)
- the endpoint-index count is exactly 3 (one per `One` role plus one
  composite pair index for `(author, work)`) — proving the `Many` role is
  excluded from both individual role indexing and pair indexing
- the registration round-trips through `load_registered_graph`: role names,
  ordinals (`RoleId::new(1..3)`), and cardinalities (`One, One, Many`) come
  back unchanged

TDD verification: temporarily changed the endpoint-index assertion from `3`
to a deliberately wrong `4`, ran the single test, confirmed it failed with
`left: 3, right: 4` (i.e. the test genuinely observes the real index count
and would catch a regression), then reverted to `3` and confirmed green.

No non-test code was touched — the existing implementation was already
correct, matching the reviewer's own throwaway-test finding.

Gates: `cargo fmt` (only `catalog.rs` changed, +133/-0), `cargo clippy
--workspace --all-features --all-targets -- --deny=warnings` clean, `cargo
test -p turso_graph_frontend` — all binaries green, 264 passed / 0 failed
(was 263; +1 for the new test), including 25/25 in `catalog::`. `mise run
corpus` / `mise run cypherbench-sample` skipped per instruction (test-only
change, already verified green on this code in the base commit).

Commit: `040434d57d27129ffb1c735fff525de021a82836` (signed), on top of
`6b8981a34b610d6f207700d126f571ee78696d9a` (the coordinator's committed
test-results, which resolved the earlier baseline/foreign-process confusion:
the uncommitted `graph/test-results/*` files were the previous task's own
runs, not a concurrent process).

Re-review follow-up: tightened the "indexed in both directions" assertion
from a bare `spill_indexes.len() == 2` count to asserting the actual
`PRAGMA index_info` column order per index — one index must cover
`(relation_id, node_id)`, the other `(node_id, relation_id)` — since a count
alone can't tell forward-twice from forward-and-reverse. Verified via
sabotage: temporarily made `install_spill_table` build both indexes with
forward column order, confirmed the test failed with `expected a reverse
(node_id, relation_id) index, got [["relation_id", "node_id"], ["relation_id",
"node_id"]]`, then reverted the sabotage and confirmed green. Amended into
`040434d57` (now `b9029ec0960d86a167aa7e9661fd145ec7784f37`, signed, message
unchanged) since nothing was built on it yet.
