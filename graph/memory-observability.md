# Memory Observability Design — Graph Frontend

Status: design (phase 1 shipped; phases 2–4 planned)
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

## Phase 2 — engine counters via pragmas (next)

turso_core already tracks the inputs; expose them read-only:

- `PageCache`: `capacity`, live entry count (`n()`), and page size are
  present in `core/storage/page_cache.rs`; multiply for resident bytes.
  Add hit/miss/eviction counters (three `AtomicU64`s on the cache).
- WAL: frame count × frame size from the WAL header state in
  `core/storage/wal.rs`.
- Surface as `PRAGMA memory_stats` returning rows of
  `(component, bytes, count)` — pragma plumbing exists in
  `core/pragma.rs` (`cache_size`, `page_count` are precedents).

The graph session then reads it through the normal connection, no new
API. Record it in the bench harness next to `peak_rss_mb` as
`page_cache_mb` / `wal_mb`.

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
