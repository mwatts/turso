# Task 6 Report: Lower expands through roles

Commit: `04dda3703529e07fa4c7c3ab4bfaa8a927450b56`
Branch: `feature/graph-nary`, starting HEAD `25f16403d`

## Summary

`lower_fixed_expand` in `graph/frontend/src/lowering.rs` no longer reads
`expand.direction`. It resolves `from_column`/`to_column` via
`relationship.role(expand.from_role)` / `relationship.role(expand.to_role)`
and matches on `(bound_reference, expand.symmetric)` instead of
`(bound_reference, expand.direction)`. Binary relations are not special-cased
anywhere — the same four-arm match handles a two-role relation because a
binary relation is just a role-shaped relation whose role pair happens to be
named `start`/`end` by the binder.

## Corrections applied vs. the brief

1. `ir::RoleExpand` does not exist yet (Task 7 renames `FixedExpand` →
   `RoleExpand`). The ternary fixture uses a hand-built `ir::FixedExpand`.
2. Task 4 did not create a reusable three-role fixture — `binary_relationship_catalog()`
   is private to `schema_catalog.rs`'s test module. A three-role
   `TernaryCatalog` (scribe/folio/txt, all `RoleCardinality::One`) was built
   directly in `dialect_alignment.rs`.
3. Step 5's `git add -A` was not used. Staged explicitly:
   `graph/frontend/src/lowering.rs graph/frontend/tests/dialect_alignment.rs`.
   `graph/test-results/*` was left modified, uncommitted.

## Golden SQL (recorded BEFORE the lowering change, via
`cargo test -p turso_graph_frontend --test dialect_alignment print_binary_sql_goldens -- --exact --ignored --nocapture`)

Query 1 — `MATCH (a:Person)-[r:KNOWS]->(b:Person) RETURN b.name`:
```
SELECT q.b4 AS "b.name" FROM (SELECT q.b3_p1 AS b4 FROM (SELECT q.* FROM (SELECT q.*, r."id" AS b2, 2 AS b2_source, n."id" AS b3, 1 AS b3_source, n."name" AS b3_p1 FROM (SELECT n."id" AS b1, 1 AS b1_source FROM "people" AS n) AS q JOIN "relationships" AS r ON r."src" = q.b1 JOIN "people" AS n ON n."id" = r."dst") AS q WHERE TRUE) AS q) AS q
```

Query 2 — `MATCH (a:Person)<-[r:KNOWS]-(b:Person) RETURN b.name`:
```
SELECT q.b4 AS "b.name" FROM (SELECT q.b3_p1 AS b4 FROM (SELECT q.* FROM (SELECT q.*, r."id" AS b2, 2 AS b2_source, n."id" AS b3, 1 AS b3_source, n."name" AS b3_p1 FROM (SELECT n."id" AS b1, 1 AS b1_source FROM "people" AS n) AS q JOIN "relationships" AS r ON r."dst" = q.b1 JOIN "people" AS n ON n."id" = r."src") AS q WHERE TRUE) AS q) AS q
```

Query 3 — `MATCH (a:Person)-[r:KNOWS]-(b:Person) RETURN b.name` (undirected/symmetric):
```
SELECT q.b4 AS "b.name" FROM (SELECT q.b3_p1 AS b4 FROM (SELECT q.* FROM (SELECT q.*, r."id" AS b2, 2 AS b2_source, n."id" AS b3, 1 AS b3_source, n."name" AS b3_p1 FROM (SELECT n."id" AS b1, 1 AS b1_source FROM "people" AS n) AS q JOIN "relationships" AS r ON (r."src" = q.b1 OR r."dst" = q.b1) JOIN "people" AS n ON n."id" = CASE WHEN r."src" = q.b1 THEN r."dst" ELSE r."src" END) AS q WHERE TRUE) AS q) AS q
```

Query 4 — `MATCH (a:Person)-[r:KNOWS]->(b:Person) WHERE b.age > 30 RETURN b`:
```
SELECT q.b4 AS "b" FROM (SELECT q.b3 AS b4 FROM (SELECT q.* FROM (SELECT q.* FROM (SELECT q.*, r."id" AS b2, 2 AS b2_source, n."id" AS b3, 1 AS b3_source, n."age" AS b3_p2 FROM (SELECT n."id" AS b1, 1 AS b1_source FROM "people" AS n) AS q JOIN "relationships" AS r ON r."src" = q.b1 JOIN "people" AS n ON n."id" = r."dst") AS q WHERE TRUE) AS q WHERE (q.b3_p2) > (30)) AS q) AS q
```

`role_lowering_emits_byte_identical_sql_for_a_two_role_relation` asserts
`lower_to_sql(query) == expected_binary_sql(query)` for all four queries,
run **after** the lowering change. It passed on the first run — no golden
had to be edited, no lowering arm had to be tweaked to match it. The
undirected/symmetric case (query 3) reconciled cleanly: today's
`Direction::Both` handling and the new `symmetric == true` handling produce
byte-identical SQL, so the BLOCKED protocol was never invoked.

## Red-state verification (real terminal output, captured before Step 3's code change)

Ternary fixture: `TernaryCatalog` implements only `RelationalCatalogSnapshot`
(no `GraphCatalogSnapshot` needed — this test never calls `bind`, it builds
`ir::Plan`/`ir::FixedExpand` by hand). `relationship_layout` for source
`SourceTableId(10)` ("transcriptions") declares three `RelationshipRoleLayout`s,
all `RoleCardinality::One`:
- `RoleId(1)` name `"scribe"` column `"scribe"`
- `RoleId(2)` name `"folio"` column `"folio"`
- `RoleId(3)` name `"txt"` column `"txt"`

`lower_ternary_to_sql` builds a `NodeScan` over `scribe` (`SourceTableId(1)`,
"scribes") bound to `s`, then a `FixedExpand` with
`from_role = RoleId(1)` (scribe), `to_role = RoleId(2)` (folio),
`symmetric = false`, `relationship_source = SourceTableId(10)`,
`target_node_source = SourceTableId(2)` ("folios"). `direction` is set to a
dummy `Direction::Outgoing` — unused by the code path under test both before
and after the change is irrelevant here since the *old* code path used
`direction` to select an arm but resolved columns via
`relationship.start_role()`/`end_role()` by name, not by the role ids passed
in. Real output before the code change:

```
$ cargo test -p turso_graph_frontend --test dialect_alignment a_ternary_hop_lowers_through_the_named_role_pair -- --exact --nocapture
...
thread 'a_ternary_hop_lowers_through_the_named_role_pair' panicked at graph/frontend/tests/dialect_alignment.rs:...:
ternary hop must lower: MissingSource(SourceTableId(10))
test a_ternary_hop_lowers_through_the_named_role_pair ... FAILED
```

This differs from the brief's assumed narrative ("silently lowers as
start -> end and returns the text instead of the folio"). The actual old
code called `relationship.start_role()` / `.end_role()`, which look up a
role **by the literal name `"start"`/`"end"`**; the ternary fixture's roles
are named `scribe`/`folio`/`txt`, so both lookups return `None` and
`.ok_or(LowerError::MissingSource(...))?` propagates a hard error rather
than a silent wrong-column join. This is a real, verified difference from
the brief's exact wording, not a discrepancy that was papered over — the old
mechanism is strictly more defensive (loud failure) than the brief describes,
but the underlying defect it demonstrates is identical: direction-based
lowering cannot express a scribe→folio hop at all, because it has no way to
name any role but "start" and "end". After Step 3's change, the same test:

```
$ cargo test -p turso_graph_frontend --test dialect_alignment -- --exact role_lowering_emits_byte_identical_sql_for_a_two_role_relation a_ternary_hop_lowers_through_the_named_role_pair
test result: ok. 2 passed; 0 failed; 11 filtered out
```

passes, with the emitted SQL containing `scribe` and `folio` columns and
never `txt`.

## Gates

- `cargo fmt` — clean.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings` — exit 0 (10 known pre-existing `ar` build-script warnings only, no new warnings).
- `cargo test -p turso_graph_frontend` — 273 tests, 0 failures.
- `mise run corpus` — see per-suite breakdown below.
- `mise run cypherbench-sample` — see per-domain breakdown below.

### Corpus per-suite comparison (`graph/test-results/runs.jsonl`)

Baseline (last pre-change run, `20260726T025247...`) vs. this change's run
(`20260726T032226.899091Z-25f16403db05-corpus-deep`):

| suite | baseline passed | this run passed | moved? |
|---|---|---|---|
| age-deep | 3042 | 3042 | no |
| cqlite-deep | 113 | 113 | no |
| grafeo-deep | 277 | 277 | no |
| sparrowdb-deep | 2164 | 2164 | no |
| tck-deep | 3330 | 3330 | no (within documented 3330-3332 flake band; identical to immediately preceding run) |

Total: `passed=8926 / 10242`, matching the stated gate floor exactly. Only
`tck-deep` shows any run-to-run variance across the whole history
(3331 → 3330 → 3330), consistent with the one known-flaky temporal scenario
called out in the task instructions. No suite outside `tck-deep` moved, so
this is not a BLOCKED condition.

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

Identical to the two preceding recorded runs for every domain. Parity: yes.

## Commit

Staged explicitly (not `git add -A`):
`graph/frontend/src/lowering.rs`, `graph/frontend/tests/dialect_alignment.rs`.
`graph/test-results/REPORT.md`, `benchmarks.jsonl`, `runs.jsonl` left
modified, uncommitted (for a dedicated test-results commit).

Signed commit: `04dda3703529e07fa4c7c3ab4bfaa8a927450b56`
("graph/lowering: join expands through role columns").
