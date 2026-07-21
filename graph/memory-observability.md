# Memory Observability Design — Graph Frontend

Status: design (phases 1–2 shipped; phases 3–4 planned)
Owner: graph frontend
Last updated: 2026-07-21

## Goal

Attribute memory to the things an operator can act on: page cache, WAL
frames, per-connection statement state, and graph-layer materialization
(JSON path/list values, junction tables). Today the only signal is
process-level peak RSS (phase 1); everything below is engine-attributed
and answers "which component grew".

## Phase 1 — process peak RSS (shipped)

`turso_graph_testkit::cypherbench::peak_rss_mb()` records
`getrusage(RUSAGE_SELF).ru_maxrss` per benchmark domain into
`benchmarks.jsonl` (`peak_rss_mb`, monotone across domains; per-domain
growth is the delta). Zero engine changes.

Limits: whole-process, monotone (no release visibility), includes
allocator slack and harness overhead.

## Policy — memory in every perf evaluation

Every performance surface records memory next to time, from two sources:
process peak RSS (`peak_rss_mb`) and engine-attributed `PRAGMA
memory_stats` (`page_cache_mb`, `wal_mb`). Current coverage:

- CypherBench domains: all three fields in `benchmarks.jsonl`.
- Lifecycle performance suite: all three in each record's `dimensions`.
- New perf harnesses must do the same before merging.

First attribution results (movie, 459k entities / 1.9M relations):
in-memory fixtures peak at ~5.5 GB RSS with only 7 MB of page cache —
the driver is the in-memory database heap, not the pager. File-backed
fixtures (`TURSO_GRAPH_BENCH_DB_DIR`) bound residency at ~1.7 GB
(-69%) for ~10% query latency; accuracy identical. Known issue: the
pragma under-reports WAL frames on the bench connection (0 against a
322 MB WAL file) — frame-count read needs fixing.

## Phase 2 — engine counters via pragmas (shipped)

Shipped shape differs from the original `(component, bytes, count)`
sketch: `PRAGMA memory_stats` returns `(stat, value)` rows —
`page_cache_pages`, `page_cache_capacity`, `page_size`,
`page_cache_bytes`, `wal_frames`, and `wal_bytes` (frame = page image +
24-byte frame header). See `PragmaName::MemoryStats` in
`core/translate/pragma.rs`. Hit/miss/eviction counters remain future
work.

The graph session reads it through the normal connection, no new API.
The bench harness records it next to `peak_rss_mb` as `page_cache_mb` /
`wal_mb`. Known gaps: the pragma has thin direct test coverage, and the
harness records zeroed stats rather than failing when the pragma is
unavailable — treat zero `wal_mb` next to a large WAL file as suspect.

## Phase 3 — graph-layer attribution

Graph-specific allocations that RSS cannot separate:

- Junction tables (`__turso_graph_node_labels_*`,
  `__turso_graph_relationship_types_*`) and their three indexes: report
  via `dbstat`-style page counts per table, aggregated by prefix. This
  is a query, not instrumentation — add a testkit helper that sums
  page counts for `__turso_graph_%` objects.
- Materialized path/list JSON: count bytes produced by
  `json_object('nodes', ...)` projections per query. Cheapest proxy:
  track cumulative result-text bytes in the session
  (`GraphSession::query` already owns the row loop). Expose as a
  per-query stat in the detail JSONL (`result_bytes`).

## Phase 4 — allocator-level (on demand, not CI)

For investigations only: run the bench binary under Instruments
(macOS allocations template) or heaptrack (Linux). No code changes;
document the invocation in the testkit README. A `#[global_allocator]`
counting wrapper was considered and rejected for CI use — it taxes
every allocation and the pragma counters above answer the recurring
questions.

## Acceptance

- Phase 2: `PRAGMA memory_stats` returns page-cache and WAL rows;
  bench harness records both per domain; numbers reconcile with RSS
  deltas within allocator slack on the nba domain.
- Phase 3: junction/index page counts and per-query `result_bytes`
  visible in detail JSONL; movie co-membership queries show
  materialization cost separately from cache growth.
