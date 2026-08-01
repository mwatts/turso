# Main merge leverage for the graph frontend

Status: landed on `feature/graph-frontend` after merging `origin/main`
through `e99973a43`. This note is the contract between “merged” and
“used”: each high-value main theme → automatic vs graph-side work →
status and proof. Newest merge first.

# Merge 3 — `origin/main` through `e99973a43`

## Scope of the merge

- 79 commits since `ae22a4afa`: recursive common table expressions and
  their follow-up fixes, `percent_rank`/`cume_dist`, a large FTS
  perf/memory pass, invalid-UTF-8 TEXT record rejection, VDBE register
  widening, encryption page-size retargeting, MVCC/sync fixes, and the
  Postgres session-information functions.
- No conflicts, and no graph-side compile fixes were needed.
- Pre-existing on `origin/main` and not introduced here: `cargo clippy
  -p turso_core --all-targets -- --deny=warnings` fails on two unused
  imports (`core/mvcc/persistent_storage/logical_log.rs`,
  `core/vdbe/mod.rs`). Verified by running the same clippy invocation in
  a clean `origin/main` worktree.
- Corpus: 8926 -> 8956 of 10242, accounted for row by row:
  - +130 / -104 from the testkit expectation-parsing fix below. The 130
    are invocations that recover an expectation they always should have
    had; the 104 are rows that were passing on a defaulted expectation
    and now fail honestly.
  - +2 from the `reduce()` rewrite: `age.age.reduce.query-39` and
    `query-40`, a nested `reduce()` in the list and in the initial value.
  - +2 that are not a fix. `tck.…temporal10.scenario-12` rows 1 and 4
    compare `duration.between(now(), now())` against `PT0S` and observe
    `PT0.000001S` when the two clock reads straddle a microsecond. Rows 2
    and 5 still fail the same way. Wall-clock flake, counted here only so
    the arithmetic closes.
  - The literal-sort-key fix moves no corpus row; no donor query orders by
    a bare literal. It is covered by `literal_sort_keys_neither_reorder_nor_fail`.
  - The aggregate-in-`reduce()` rejection nets zero: `query-69` was
    passing by accident before (wrong expectation, wrong answer) and
    passes honestly now.

## High-value themes

| Theme | Auto vs graph work | Status | Evidence |
|---|---|---|---|
| Recursive CTEs (`4360b24f5` + follow-ups) | **Graph work done** — `reduce()` rewritten | **done** | `reduce(acc = init, x IN list \| body)` lowered to an unrolled ladder of ten sibling CTEs and raised `reduce() list exceeds 10 elements` past that. List length is query data, so the cap was a semantic hole, not a resource limit. The fold is now one `WITH RECURSIVE` over (accumulator, list, index) of fixed SQL size. Test: `reduce_folds_lists_longer_than_the_former_unroll_cap`. `DESIGN_DECISIONS.md` hard-block entry updated. |
| Aggregates inside `reduce()` (fallout of the recursive rewrite) | **Graph work done** — rejected at bind time | **done** | Core rejects aggregates in a recursive query, as SQLite does, so the fold's body leaked `recursive aggregate queries not supported`. The other two positions were already broken and neither error said so: an aggregate in the list leaked `no such function: collect`, and one in the seed silently answered from a bogus grouping. An aggregate anywhere in a `reduce()` cannot mean what it reads as — the fold's rows are not the outer rows — so the binder now rejects all three with AGE's and Neo4j's message. Test: `aggregates_inside_reduce_are_rejected_in_cypher_terms`. Chasing why the corpus still scored this row as expecting rows exposed a testkit bug that predates the merge: psql's `LINE 1: … cypher('g', $$` error echo has an unclosed `$$`, so the invocation regex swallowed the invocation *after* every errored one and left it defaulting to a row expectation. Fixed alongside; it is what moves `age-deep` by +28 net. |
| Positional `ORDER BY` narrowed to 32-bit literals (`8961168f6`) | **Graph work done** — divergence is the frontend's | **done** | Main's rule is SQLite's and correct for SQL, which makes the Cypher divergence graph-side: lowering emitted a literal sort key straight into SQL `ORDER BY`, so `RETURN n.name AS a ORDER BY 1 DESC` reversed the result and `ORDER BY 2` failed with "term out of range". Cypher reads a literal as a constant every row shares. Lowering now drops literal (and negated-literal) sort keys and emits no `ORDER BY` when none survives. Test: `literal_sort_keys_neither_reorder_nor_fail`. |
| FROM-clause call arguments rejected on tables and CTEs (`bb0603e64`) | **Compatibility check only**; vtabs keep their arguments | **done** (auto) | Variable-length path lowering emits a 16-argument call on the internal vtab `__turso_graph_expand(...)` in `FROM` (`lowering.rs:1086`, `:1129`). Main's tightening exempts virtual tables, so the expand join is unaffected; the traversal and native-capability suites stay green. |
| FTS perf/memory pass (`39d7f9c76`) | **Automatic** | **done** (auto) | Graph FTS registers core's index method rather than implementing `IndexMethodCursor`, so the segment/cache work and the new `IndexMethodCostContext` cost model apply unchanged. `graph/frontend/src/fts.rs` only gates on `experimental_index_method_enabled`. Covered by the graph FTS tests in `native_capabilities`. |
| Invalid UTF-8 in TEXT record payloads rejected (`1089c645e`) | **Automatic**; graph-reachable only through a foreign writer | **done** (auto), **test deferred** | Same shape as merge 2's vector-blob win: a property column is an ordinary TEXT column whose bytes come from whatever wrote the table, and the decoders previously built `&str` with `from_utf8_unchecked` (SIGSEGV in release). A graph regression test needs a genuinely malformed payload, and turso cannot produce one — `UPDATE people SET name = CAST(X'FF' AS TEXT)` stores a valid U+FFFD, and a `MATCH … RETURN n.name` over it returns normally. Writing the file with sqlite3 first would prove it; deferred rather than faked with a payload that is actually valid. |
| VDBE register fields widened to u32 (`ab5d692a8`) | **Automatic** | **done** (auto), **reachability unproven** | Removes a translate-time `value exceeds u16::MAX` panic. The reported trigger needs seven CTEs at the 2000-column limit; no Cypher surface produces a projection near that width, so this is hardening the graph inherits rather than a fixed graph bug. |
| MVCC checkpoint-end ordering, deferred-log CRC, internal-commit marking | **Automatic** for graph MVCC sessions | **done** (auto) | Graph mutations run through core transactions under the branch's shared write-transaction guard. Verified by the graph suites plus `turso_core --lib`. |
| Record payload extraction, inline short value copies (`c20a54999`, `d35bc84e1`, `bc6c2a50e`) | **Automatic** | **done** (auto) | Every graph statement is prepared SQL through `turso_core`; nothing is reimplemented in graph. |

## Secondary main deltas (optional for graph)

| Theme | Auto vs graph | Status |
|---|---|---|
| Recursive-CTE correctness follow-ups: correlated unqualified refs against CTE-backed outer tables (`6c85ac734`), `NullRow` on a pseudo cursor (`eeabdaf63`), outer-join ON clauses left of the recursive table (`fefca00fb`), self-reference affinity (`fc430dc6b`), queue null override (`d6f4c0b79`) | Automatic wherever graph emits recursive CTEs | **done** (auto) — the `reduce()` lowering qualifies every reference and joins no recursive table, so only the affinity fix is load-bearing today; the rest guard future recursive lowerings |
| Compound selects emitted without cloning the plan (`04e1c6133`) | Automatic on the `UNION` shapes lowering emits | **done** (auto) |
| `percent_rank()` / `cume_dist()` (`11389ce2c`) | Automatic if graph emits window SQL | **deferred** — Cypher still has no window surface syntax; same standing re-check as the merge 2 window rewrite |
| Recursive-CTE dedup-index seek elision (`d68190499`) | Automatic on `UNION` recursive CTEs | **skipped** — the `reduce()` fold is `UNION ALL`, which has no dedup index |
| Attached-database transaction handling (`9913f299d`) | Automatic | **skipped** — graph "attach mode" means installing on an existing connection, not `ATTACH DATABASE`; graph emits no `ATTACH` |
| Sequence conflict rollback (`3bf79fb4b`) | Automatic for sequence-backed identity columns | **skipped** — graph emits no sequences; BYO tables may still use them |
| Encryption page-size retargeting, sync composite-PK replay, portable-change rootpage resolution, Postgres session functions, antithesis/CI, benches | Out of graph scope | **skipped** (non-goals) |

## How to re-verify

```sh
cargo test -p turso_graph_frontend --test native_capabilities reduce_folds
cargo test -p turso_graph_frontend --test native_capabilities aggregates_inside_reduce
cargo test -p turso_graph_frontend --test native_capabilities literal_sort_keys
cargo test -p turso_graph_frontend
cargo test -p turso_graph_runtime
cargo test -p turso_graph_testkit
cargo test -p turso_core --lib
mise run corpus
```

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
