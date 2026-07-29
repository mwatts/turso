# Main merge leverage for the graph frontend

Status: landed on `feature/graph-frontend` after merging `origin/main`
through `ae22a4afa`. This note is the contract between “merged” and
“used”: each high-value main theme → automatic vs graph-side work →
status and proof. Newest merge first.

# Merge 2 — `origin/main` through `ae22a4afa`

## Scope of the merge

- 113 commits since `a9926eb46`: window-function rewrite, a batch of
  panic/overflow hardening fixes across json/vector/uuid/regexp/
  percentile, MVCC rollback and passive-checkpoint fixes, `PRAGMA
  table_info` composite-PK positions, fallible B-tree and JSONB buffer
  growth, deferred-FK repayment, and the serverless drivers.
- Conflict resolution: only `Cargo.toml` conflicted, on the workspace
  crate-version bump to `0.8.0-pre.2`. Took main’s block and bumped the
  six `turso_graph_*` entries to match; graph crates use
  `version.workspace = true`, so they must not lag the workspace version.
- No compile fixes were needed on the graph side this time.

## High-value themes

| Theme | Auto vs graph work | Status | Evidence |
|---|---|---|---|
| Vector blob parse-boundary validation (sparse `idx` range, float1bit/float8 size underflow) | **Graph work done** (regression coverage) + automatic core validation | **done** | Graph registers `vector32_sparse`, `vector8`, `vector1bit`, and `vector_extract`, and a vector property is an ordinary BLOB column whose bytes come from whatever writer touched the table. Before `ac49eb07c` / `49ce7cc0b` a malformed blob aborted the process from a plain `MATCH … RETURN vector_extract(n.prop)`; now the statement fails and the session survives. Test: `malformed_vector_property_fails_the_query_instead_of_aborting`. |
| Remaining panic/overflow hardening (json bounds, JSONB validation, `uuid7`, `unixepoch` modifier, `percentile_disc`, `regexp` arity, `gcd`/`lcm` on `i64::MIN`) | **Not reachable from Cypher** | **skipped** (not reachable) | `graph/frontend/src/functions.rs` registers only the vector, `struct_pack`/`union_*`, and FTS families — no json/regexp/percentile/uuid passthrough. Cypher temporal goes through `turso_graph_temporal` (jiff), not core’s date functions, so the `unixepoch` modifier range check is core-only. Re-check if the registry grows a json or date family. |
| `PRAGMA table_info` composite primary-key position | **Automatic**; no graph change required | **done** (auto) | `catalog.rs::require_columns` reads column 5 as `> 0` and `require_unique_identity` counts the PK columns. Main replaced `emit_bool(primary_key())` with the 1-based key position, which agrees with `> 0` for every column and still yields more than one PK column for a composite key, so identity validation is unchanged. |
| MVCC: write-set transfer on rollback, passive-checkpoint/DROP races, sequence change-count leak | **Automatic** for graph MVCC sessions | **done** (auto) | Graph sessions run their mutations through core transactions; the branch’s shared write-transaction guard sits above them. Verified by the MVCC session suite (`mvcc_transaction_reads_its_writes_without_cross_connection_leakage` and siblings) plus `turso_core --lib`. |
| Fallible buffer growth (B-tree cell buffers, JSONB header) | **Automatic** | **done** (auto) | Graph allocates no core buffers itself; every graph statement goes through the same prepared-SQL path. Covered by the green graph suites after the merge. |
| Spilled sort-merge dropped comparison result (`16efbd6ce`) | **Automatic** on large `ORDER BY` | **done** (auto), **test deferred** | Graph `ORDER BY` lowers to core sorters, so the fix applies unchanged. A graph-level regression would need a sorter-spill-sized dataset; the corpus run is the practical coverage, and a targeted test is deferred rather than faked with a small dataset that never spills. |

## Secondary main deltas (optional for graph)

| Theme | Auto vs graph | Status |
|---|---|---|
| Window rewrite (SQLite frame-cursor model, `lag`/`lead`/`ntile`, `AggInverse`) | Automatic if graph emits window SQL | **deferred** — Cypher has no window surface syntax and lowering emits none; re-check if a top-k-per-group or ranked-path lowering appears |
| Deferred FK repayment per matching child on parent INSERT | Automatic for registered source tables that declare FKs | **done** (auto) — graph DDL emits no FK constraints, but BYO node/relationship tables commonly do |
| Views: star columns derived from joined sources | Automatic if graph reads views | **skipped** — graph registers base tables and its own junction/spill tables, not views |
| `CREATE TABLE … AS SELECT` reports zero result columns | Automatic | **skipped** — graph emits no CTAS |
| Blob literal prefix case preserved by the parser | Automatic | **done** (auto) — lowering emits uppercase `X'…'`; Cypher itself has no blob literal, so blob values arrive as parameters |
| `ALTER TABLE ADD COLUMN` with explicit `NULL` | Automatic | **skipped** — graph and testkit add columns without a nullability constraint |
| Outer-scope “no such column” echoes query casing | Automatic in surfaced core errors | **done** (auto) |
| Serverless drivers (Rust/Python/Go/JS), bindings, CI, TCL | Out of graph scope | **skipped** (non-goals) |

## How to re-verify

```sh
cargo test -p turso_graph_frontend --test native_capabilities malformed_vector
cargo test -p turso_graph_frontend
cargo test -p turso_graph_runtime
cargo test -p turso_graph_testkit
cargo test -p turso_core --lib
mise run corpus
```

Note: `cargo build --workspace --all-targets` fails in `perf/memory` with
`#[global_allocator]` conflicting with the one in `bindings/rust`. That
crate is not a `default-member`, the conflict predates this merge, and it
is unrelated to graph; `cargo build --all-targets` is clean.

# Merge 1 — `origin/main` through `d14a446da`

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
