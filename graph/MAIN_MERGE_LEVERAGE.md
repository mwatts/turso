# Main merge leverage for the graph frontend

Status: landed on `feature/graph-frontend` after merging `origin/main`
(`d14a446da` and ancestors). This note is the contract between “merged”
and “used”: each high-value main theme → automatic vs graph-side work →
status and proof.

## Scope of the merge

- Integrated main through the covering-index, VDBE/types recycling,
  aggregate-collation, and comparison/NULLS sorter work.
- Conflict resolution: only `core/vtab.rs` conflicted. Combined main’s
  `crate::alloc::Vec<Value>` filter args with this branch’s
  `InternalVirtualTableStep` return so graph expand can still yield.
- Follow-up compile fix: graph expand yield sites used
  `IOCompletions::Single(...)` after main made `IOCompletions` a
  single-field struct; updated to `IOCompletions(...)`.

## High-value themes

| Theme | Auto vs graph work | Status | Evidence |
|---|---|---|---|
| Column-free covering index scans | **Graph work done** + automatic core planning | **done** | Pure Cypher `count(*)` over a labeled node scan now lowers to a direct junction `count(*)` (single label) so core can SEARCH/USE the semantic-type-first complete index. Unlabeled pure `count(*)` lowers to `SELECT count(*) FROM table` so complete secondary indexes (including registration endpoint indexes on relationship tables) are eligible for covering count. Tests: `pure_count_star_uses_junction_covering_index`, `relationship_table_count_uses_registration_covering_index`, lowering unit tests `pure_count_star_over_*`. |
| VDBE / types record recycling (value clone, register copies, record buffers, staged buffers, pseudo registers, allocator-backed B-tree cells) | **Automatic** via core on every prepared SQL path graph uses | **done** (auto) | No graph API change. Graph still prepares/executes through `turso_core`; recycling applies to sorter materialization, joins, mutations, and snapshot SQL. Not reimplemented in graph; verified by green graph + `turso_core --lib` suites after merge. Further wins from reducing mutation N+1 / path clones are **deferred** (out of this merge’s non-goals). |
| Aggregate argument collation at translate time | **Automatic** for property columns; **no graph emission change required** | **done** (core-only) | Property columns use default BINARY collation; `min`/`max` over text match SQLite BINARY after main’s translate-time collation resolve. Catalog name lookups already emit `COLLATE NOCASE` where graph identity is case-insensitive. Test: `text_order_by_and_min_max_follow_sqlite_binary_collation`. |
| Sorter / comparison / NULLS handling | **Automatic** on lowered `ORDER BY … NULLS FIRST/LAST`; binder already maps ASC→NULLS LAST, DESC→NULLS FIRST | **done** (no graph emission change) | Cypher has no NULLS surface syntax; IR/lowering already emit SQL NULLS clauses. Main’s sorter refactor is exercised by the same ordered-text regression. Test: `text_order_by_and_min_max_follow_sqlite_binary_collation`. |

## Secondary main deltas (optional for graph)

| Theme | Auto vs graph | Status |
|---|---|---|
| Subqueries in aggregate args/FILTER before group-by | Automatic if graph emits those SQL shapes | **deferred** — no gap found on current lowering; re-check if compound aggregate FILTER patterns appear in corpus |
| Compound SELECT explicit COLLATE | Automatic for UNION ORDER BY with COLLATE | **deferred** — catalog uses COLLATE NOCASE on simple SELECTs; graph UNION paths do not yet need explicit compound COLLATE emission |
| EXPLAIN compound+ORDER BY panic fix | Automatic on EXPLAIN QUERY PLAN | **done** (auto) — graph EXPLAIN path uses core EQP |
| Checked `FromValue` / int→f64 | Automatic for typed `Row::get` consumers | **deferred** for graph frontend (matches on `Value` today); relevant if a typed Rust consumer wrapper is added later |
| `IOCompletions` unit struct | Graph glue fixed at merge | **done** |
| Window `nth_value`, serverless, sync JS, TCL, dependabot | Out of graph scope | **skipped** (non-goals) |

## Graph-side change that made covering counts real

File: `graph/frontend/src/lowering.rs` — `try_lower_column_free_node_count`.

Without it, `MATCH (n:Person) RETURN count(*)` lowered to
`SELECT count(*) FROM (SELECT n.id … JOIN labels …) AS q`, which cannot
use main’s column-free covering path on the junction indexes. With it:

- Labeled, single label →
  `SELECT count(*) AS bN FROM labels AS lbl0 WHERE source_id = S AND label = '…'`
- Unlabeled →
  `SELECT count(*) AS bN FROM "people"` (or the registered node table)

## How to re-verify

```sh
cargo test -p turso_graph_frontend --lib pure_count_star_over
cargo test -p turso_graph_frontend --test native_capabilities pure_count_star_uses_junction
cargo test -p turso_graph_frontend --test native_capabilities relationship_table_count
cargo test -p turso_graph_frontend --test native_capabilities text_order_by_and_min_max
cargo test -p turso_graph_frontend
cargo test -p turso_graph_runtime
cargo test -p turso_graph_testkit
cargo test -p turso_core --lib
```
