# Quality & Rust Hygiene Review — `feature/graph-frontend`

| Field | Value |
| --- | --- |
| **Base** | `origin/main` (`merge-base` of this branch) |
| **Head** | `feature/graph-frontend` @ `b4d65fb5b` |
| **Diff size** | ~1,078 files, +281,463 / −126 lines (228 commits) |
| **Primary surface** | `graph/{ir,cypher,runtime,frontend,temporal,testkit}`, `core/frontend.rs`, related core prepare/vtab/schema hooks |
| **Review mode** | Read-only; no code changes |
| **Date** | 2026-07-21 |
| **Bar** | Correctness, memory, performance, Rust hygiene, clarity, document alignment |

## Summary

This branch is a substantial multi-frontend graph delivery: Turso-owned IR identities, a pest Cypher frontend, CSR traversal runtime (pgGraph-derived), session-local snapshot overlays, SQL lowering through core’s engine AST, mutations, temporal extension, and a large donor/corpus harness. Architecture choices are mostly sound: `FrontendCompiler` is connection-stateless for reprepare, traversal budgets and cancel quanta exist, and transaction-visible snapshots avoid publishing uncommitted derived CSR state.

The highest-severity gaps are **semantic correctness** (relationship `MERGE` type matching; multi-source catalog only honoring the first source; aggregate detection only for bare calls; unbounded variable-length paths silently capped at 64 hops; incomplete three-valued equality for dynamic list/map values) and **resource honesty** (memory limits undercount real heap; snapshot build materializes full tables; uniqueness/type filters are O(depth)/O(|types|); mutation executor is N+1 prepare/execute). Document drift is severe enough to mislead agents: `graph/CONFORMANCE.md` still advertises 1,413/10,392 while `graph/test-results/REPORT.md` reports 8,711/10,242.

Overall: **ship-quality foundation with production-blocking semantic and multi-source bugs, soft safety limits, and stale published metrics.** Address bugs before treating corpus progress as a compatibility claim.

## Resolution pass — 2026-07-21

A follow-up pass addressed both P0 groups (correctness issues 1–8 and honesty
issues 32–36) plus the nits (28–30): fixes carry regression tests verified to
fail without the change, and per-issue outcomes are recorded in the `Status`
lines below. Issues 9–27, 31, and 37–39 (P1 safety/perf, semantics polish,
hygiene debt) remain open and triaged. Verification: all graph crate tests
(197), `turso_core` lib tests (2,187), `cargo fmt --check`, and graph-crate +
core clippy at `--deny=warnings` pass; the testkit smoke suite is clean; a
full corpus run after the fixes passes 8,807 / 10,242 versus the 8,800
baseline (no regressions, +7).

---

## What looks strong

- **Core multi-frontend boundary** (`core/frontend.rs`): `PreparedSource::{Dialect,Frontend}`, `FrontendCompilation` prerequisites documented as prepare-only, reprepare-safe compiler trait (`Send + Sync + 'static`, no connection state).
- **IR identities** (`graph/ir/src/identity.rs`): non-zero newtypes via `NonZeroU*`; clear `InvalidId` errors; no silent zero IDs.
- **Runtime structure** (`graph/runtime`): dual CSR, resumable `TraversalCursor` with work quanta, cancellation trait, typed `RuntimeError` / `LimitKind`, `#![forbid(unsafe_code)]` on the frontend crate.
- **Session overlays** (`SessionSnapshotStore` + savepoint visible builds): tests cover explicit txn, savepoint rollback, MVCC, cancel-before-install.
- **Graph expand VDBE integration**: core calls `next_step` / `filter_step`; cursor yields on `TraversalStep::Pending`; interrupt maps to cancel.
- **Catalog registration**: uniqueness checks, custom-types-disabled fail-closed, rollback of failed registration, generation triggers for invalidation.
- **SchemaCatalog type mapping**: careful array vs affinity handling with fixture regressions (INT/VARCHAR/STRUCT).
- **Workspace layout**: graph crates under `graph/*`, workspace deps, IR/cypher/runtime free of `turso_core` (good layering).

---

## Issues

Severity key: **bug** = correctness or safety contract break; **suggestion** = important quality/perf/hygiene debt; **nit** = polish.

### Correctness

#### Issue 1 — Severity: bug
- **File:** `graph/frontend/src/mutation.rs:1275–1312`, `895–958`
- **Description:** Relationship `MERGE` matches only on endpoint columns (and property columns), not relationship type. Fixed-length expand lowering filters through the type junction, but `insert_relationship` always passes `merge_predicates: &[]`. On a match, `record_relationship_type` still runs for the requested type. Concretely, an existing `(a)-[:LIKES]->(b)` can satisfy `MERGE (a)-[:KNOWS]->(b)`, report not-created, and attach `KNOWS` to the same relationship identity in the type junction — corrupting type membership and openCypher MERGE semantics.
- **Suggestion:** Mirror `node_label_predicates` with type-junction EXISTS predicates for each requested type on MERGE. Do not record types on a pure match (or only when the match already carried those types). Add a regression with two types on one relationship table.
- **Status:** fixed — `relationship_type_predicates` mirrors label predicates on MERGE; types recorded only on create. Regression: `merge_relationship_does_not_match_a_different_type`.

#### Issue 2 — Severity: bug
- **File:** `graph/frontend/src/schema_catalog.rs:27–41`, `350–370`; `binder.rs` single-source APIs; `mutation.rs` detach-delete paths
- **Description:** Registration models multiple node/relationship sources (`RegisteredGraph::{node,relationship}_sources`). `SchemaCatalog` layout/property resolution only uses `.first()` via `node_source_entry` / `relationship_source_entry`. Binder CREATE/MATCH targets the single `node_source` / `relationship_source`. Default `relationship_sources` is a one-element vector, so `DETACH DELETE` only clears the first relationship table. Multi-source graphs silently misroute or drop data.
- **Suggestion:** Resolve layouts by `SourceTableId` (and label/type → source maps) over the full vectors; override `relationship_sources` to return all ids; make property lookup source-aware — or reject multi-source registration until supported.
- **Status:** fixed (fail-closed) — registration now rejects >1 node or relationship source (`CatalogError::MultipleSourcesUnsupported`) until multi-source resolution exists end to end. Regression: `register_graph_rejects_multiple_sources_per_kind`.

#### Issue 3 — Severity: bug
- **File:** `graph/frontend/src/binder.rs:2354–2362`, `2680–2703`
- **Description:** Aggregate detection only treats a projection item that is a **bare** function call (`count`, `sum`, …) as aggregating. Expressions such as `count(*) + 1`, `2 * sum(x)`, or `avg(x) AS a, a + 1` fail to set `has_aggregates`, so the binder does not introduce an Aggregate plan and may lower aggregates as ordinary scalar calls with wrong grouping semantics.
- **Suggestion:** Walk expression trees for aggregate roots (with openCypher nesting rules). Add TCK-style fixtures for arithmetic over aggregates and mixed aggregate/non-aggregate projections.
- **Status:** fixed — recursive aggregate detection (`contains_aggregate_call`); embedded aggregates hoist into hidden aggregations with a post-aggregate Project for the remainder. Regression: `compound_aggregate_projections_compute_after_grouping`.

#### Issue 4 — Severity: bug
- **File:** `graph/frontend/src/binder.rs:191`, `2001–2006`
- **Description:** Unbounded variable-length ranges (`*`, `*min..`) silently fill `max_hops` with `DEFAULT_UNBOUNDED_MAX_HOPS` (64). That changes query meaning without a diagnostic: paths longer than 64 hops are invisible rather than rejected or truly unbounded under a resource budget.
- **Suggestion:** Either error on unbounded ranges with a clear “set an upper bound” message, or document and expose the cap as a session/pragma limit that surfaces in EXPLAIN and errors when hit.
- **Status:** fixed — unbounded ranges are flagged `unbounded` through IR/vtab/runtime; hitting the 64-hop cap with a longer admissible path now errors (`LimitExceeded{Hops}`) instead of silently truncating. Explicit bounds keep truncation-as-semantics. Regressions: `error_at_max_hops_rejects_silent_truncation`, `unbounded_expansion_past_the_hop_cap_fails_loudly`.

#### Issue 5 — Severity: bug
- **File:** binder/lowering for equality; recent three-valued list/map equality work
- **Description:** Deep/three-valued equality for list and map values is implemented in places, but dynamic/`Any` values (parameters, UNWIND of JSON text, untyped properties) can still fall through to SQL `=` rather than Cypher null-aware deep equality. JSON-looking text may also be promoted to structural list/map comparison in some equality paths, risking `'[]' = []`-class surprises.
- **Suggestion:** Route all Cypher equality through one null-aware deep-equal path for List/Map/Any; reject or explicitly document JSON-text promotion; add parameter and UNWIND fixtures.
- **Status:** partially fixed — `Any`-typed operands now route through null-aware `cypher_equals` (consistent with `IN`); regression `lowers_any_typed_equality_through_cypher_equals`. The `'[]' = []` JSON-text promotion remains: lists lower to JSON text, so it is unfixable without a typed Cypher value boundary; documented on `comparison_value`.

#### Issue 6 — Severity: bug
- **File:** `graph/frontend/src/session.rs:190–204`
- **Description:** `GraphSession::mutate` always `clear()`s the local snapshot store before returning the mutation `Result`. If the mutation succeeds but `clear` fails (`StorePoisoned`), the caller sees an error after a durable write. If the mutation fails and then `clear` fails, the original mutation error is discarded.
- **Suggestion:** Propagate the mutation result first; treat clear failure as secondary (log/attach), never overwrite a successful write with clear failure.
- **Status:** fixed — mutation result is consumed first; clear failure after a successful write is logged, never flips Ok to Err; a failed mutation keeps its own error. Regression: `mutate_result_survives_snapshot_clear_failure`.

#### Issue 7 — Severity: bug
- **File:** `graph/frontend/src/snapshot.rs:428–454` vs `460–485`
- **Description:** `build_traversal_snapshot` always issues `BEGIN`/`COMMIT` (used by `SnapshotStore::refresh`). The visible path correctly uses a nested savepoint. Shared refresh under an open user transaction will fail or interfere with caller transaction state. The same class of issue appears in catalog registration (`BEGIN IMMEDIATE`) without outer-txn detection.
- **Suggestion:** If already in a transaction, use savepoints; only open top-level transactions in autocommit. Document that shared publish observes committed state only.
- **Status:** fixed — shared refresh now refuses non-autocommit (`SnapshotError::RefreshInsideTransaction`, publishes committed state only); registration uses a savepoint inside a write transaction and cleanly rejects a read-only deferred transaction (internal/nested statements cannot upgrade Read→Write; previously panicked in `op_set_cookie`). Added `Connection::in_write_transaction`. Regressions: `shared_refresh_refuses_an_open_transaction`, `register_graph_inside_a_transaction_scopes_to_the_outer_transaction`, `register_graph_rejects_a_read_only_open_transaction`.

#### Issue 8 — Severity: bug
- **File:** `graph/frontend/src/snapshot.rs:733–755`; `graph_expand.rs:440–449`
- **Description:** `SourceIdentity::Real` force-maps ±0.0 to bit pattern `0` but uses raw `to_bits()` for other floats (including distinct NaN payloads). As a HashMap key encoding this is inconsistent and a poor primary-key model for graph identities.
- **Suggestion:** Reject REAL identity columns at registration, or define one total encoding (including NaN policy) and test it.
- **Status:** fixed — one canonical total encoding `SourceIdentity::real` shared by snapshot build and graph_expand: ±0.0 collapses, every NaN payload maps to the canonical quiet NaN. Regression: `real_identity_encoding_is_total_and_canonical`.

#### Issue 9 — Severity: bug
- **File:** binder nullability for `labels()` / entity introspection; OPTIONAL MATCH
- **Description:** Entity introspection helpers such as `labels(n)` can be typed `NonNull` even when `n` is a nullable OPTIONAL MATCH binding. openCypher propagates null (`labels(null) → null`). Wrong nullability misleads later type-driven lowering and result rendering.
- **Suggestion:** Propagate argument nullability into function result types for entity introspection; add OPTIONAL MATCH fixtures.
- **Status:** open — triaged, not addressed in this pass.

#### Issue 10 — Severity: bug
- **File:** `graph/temporal/src/lib.rs` (named-zone `Time` construction)
- **Description:** Named IANA time zones for `Time` values can be resolved via `Timestamp::now()`-relative offsets, making results DST- and clock-dependent rather than a stable calendar/offset interpretation.
- **Suggestion:** Use a fixed reference instant or store zone id + local fields without baking a transient offset from “now”. Add DST-boundary tests.
- **Status:** open — triaged, not addressed in this pass.

#### Issue 11 — Severity: bug
- **File:** mutation RETURN `ORDER BY` binding vs read-path projection binding
- **Description:** Mutation-stage `RETURN … ORDER BY` sort keys bind in pre-RETURN scope without the alias substitution used for read projections and mutation `WITH`. Aliases introduced in RETURN are unusable in ORDER BY on the mutation path (or bind the wrong expression).
- **Suggestion:** Share the read-path alias-substitution / post-aggregate sort-scope rules with mutation RETURN.
- **Status:** open — triaged, not addressed in this pass.

#### Issue 12 — Severity: bug
- **File:** quantifier alias substitution in list comprehensions / predicates
- **Description:** When a quantifier loop variable collides with a projection alias, substitution can disable **all** alias rewrites in the body rather than only the colliding name — shadowing unrelated aliases.
- **Suggestion:** Shadow per-name; keep other aliases active. Add a fixture with mixed outer aliases + loop var name collision.
- **Status:** open — triaged, not addressed in this pass.

### Memory

#### Issue 13 — Severity: bug
- **File:** `graph/runtime/src/limits.rs`, `traversal.rs`, `shortest.rs`, `csr.rs`
- **Description:** Memory budgets charge approximately `size_of::<T>() + len * size_of::<Elem>()`. They omit `Vec` capacity, allocator headers, `HashMap`/`HashSet` table overhead, frontier deque structure, and CSR **build peak** (per-row `Vec`s before flatten). Active path state is released when popped from the frontier and not re-retained while expanded. `LimitKind::Memory` is therefore a soft heuristic, not a hard safety bound — unsafe to advertise as OOM protection.
- **Suggestion:** Document as estimate; charge capacity + map overhead; re-retain active path; include peak build temps in `BuildLimits` checks; consider process RSS via existing `memory_stats` as a second line of defense.
- **Status:** open — triaged, not addressed in this pass.

#### Issue 14 — Severity: bug
- **File:** `graph/frontend/src/snapshot.rs:488–730`
- **Description:** Snapshot build materializes every node/relationship source into `Vec<Vec<Value>>`, builds node id maps twice (tuple keys then `NodeCoordinate`), clones identities on endpoint lookup and on `node_id()`, and retains coordinates + CSR + reverse CSR. `estimated_peak_build_bytes` undercounts intermediate row vectors. Large graphs pay multi-GB peaks before limits fire.
- **Suggestion:** Stream rows into the builder; single `HashMap<NodeCoordinate, NodeId>`; borrowed lookup keys (`Equivalent`); include intermediates in peak accounting.
- **Status:** open — triaged, not addressed in this pass.

#### Issue 15 — Severity: suggestion
- **File:** `graph/runtime/src/traversal.rs:189–199`; `shortest.rs:76–85`
- **Description:** Every accepted hop clones three `Vec`s (`nodes`, `relationships`, `relationship_types`). Under `Uniqueness::Walk` and high fanout this is combinatorial allocator pressure before path/work limits trip — worsened by soft memory accounting.
- **Suggestion:** Persistent/path-copy-on-write structures, or parent indices + edge stacks with materialization only on emit.
- **Status:** open — triaged, not addressed in this pass.

#### Issue 16 — Severity: suggestion
- **File:** `graph/frontend/src/session.rs` + `snapshot.rs` publish path
- **Description:** After committed generation bumps, sessions rebuild **local** snapshots only; the shared `SnapshotStore` stays stale until an explicit `refresh`. Every connection re-pays full graph build after writes (O(connections × |V|+|E|)).
- **Suggestion:** `publish_if_current` on autocommit-visible rebuilds; keep local overlays only for uncommitted state.
- **Status:** open — triaged, not addressed in this pass.

### Performance

#### Issue 17 — Severity: suggestion
- **File:** `graph/runtime/src/traversal.rs:337–347`; `csr.rs:229`
- **Description:** Trail/Path uniqueness uses `Vec::contains` (O(depth) per candidate edge). Relationship-type filters use linear `slice::contains` per neighbor step.
- **Suggestion:** Bitset/hash set for trail/path uniqueness (with hop-bounded depth); hashed or sorted type filters, especially for multi-type expansions.
- **Status:** open — triaged, not addressed in this pass.

#### Issue 18 — Severity: suggestion
- **File:** `graph/runtime/src/shortest.rs:65–86`, `170–196`
- **Description:** Shortest-path algorithms call `Graph::neighbors`, which allocates a full `Vec<Neighbor>` per visit. The main traversal path already streams via `NeighborCursor`. High-degree nodes allocate unbudgeted adjacency repeatedly.
- **Suggestion:** Drive BFS/Dijkstra from `neighbor_cursor` like `TraversalCursor`.
- **Status:** open — triaged, not addressed in this pass.

#### Issue 19 — Severity: suggestion
- **File:** `graph/frontend/src/mutation.rs` (execute path throughout)
- **Description:** Mutations are row-at-a-time and statement-at-a-time: re-prepare SQL for projections, SET/INSERT/DELETE, MATCH expansion, UNWIND, and ORDER BY keys. DISTINCT keys use `format!("{:?}", …)` — not a stable value-equality contract and allocation-heavy. Large `MATCH … CREATE` graphs are catastrophic.
- **Suggestion:** Batch SQL (INSERT…SELECT / UPDATE…FROM); cache prepared statements per op shape; proper Value equality/hash for DISTINCT.
- **Status:** open — triaged, not addressed in this pass.

#### Issue 20 — Severity: suggestion
- **File:** `graph/frontend/src/schema_catalog.rs` relationship type name lookup; `catalog.rs` `load_registered_graph`
- **Description:** Dynamic type name resolution can prepare/query the registry during lowering per hop. `load_registered_graph` re-validates columns via `PRAGMA table_info` on every load used by status/refresh/publish paths.
- **Suggestion:** Cache registry names; split cheap generation probe vs full validated load keyed by schema cookie/generation.
- **Status:** open — triaged, not addressed in this pass.

#### Issue 21 — Severity: suggestion
- **File:** `graph/runtime/src/csr.rs`; `graph/frontend/src/snapshot.rs`
- **Description:** Default `std` hasher for node indexes and coordinate maps. Trusted integer/binary keys would benefit from a faster hasher (`foldhash` / `rustc-hash`) on large builds.
- **Suggestion:** Use a documented fast hasher for internal maps once API stability allows (or type-alias `GraphHashMap`).
- **Status:** open — triaged, not addressed in this pass.

### Rust hygiene & API design

#### Issue 22 — Severity: suggestion
- **File:** `graph/frontend/src/binder.rs` (~5,977 LOC)
- **Description:** Binder is a monolith mixing read binding, mutation binding, expression typing, aggregates, path expansion, and tests. High clone density (~225 `.clone()` sites), hard to review, and encourages accidental coupling. Contrasts with cleaner IR/runtime module splits.
- **Suggestion:** Split into `bind/{query,mutation,expression,path,aggregate}.rs` with shared `Binder` state; reduce clone by borrowing spans/catalog lookups.
- **Status:** open — triaged, not addressed in this pass.

#### Issue 23 — Severity: suggestion
- **File:** `graph/frontend/src/mutation.rs` (multiple `#[allow(clippy::too_many_arguments)]`)
- **Description:** Six production `allow`s without `reason`, contrary to project preference for `#[expect(..., reason = "...")]` and parameter objects. Signals missing context structs (`MutationExecCtx`).
- **Suggestion:** Introduce a small execution context struct; replace allow with expect+reason or eliminate the lint.
- **Status:** open — triaged, not addressed in this pass.

#### Issue 24 — Severity: suggestion
- **File:** `graph/frontend/src/compiler.rs:45–58`
- **Description:** Bind/lower failures map to `LimboError::ParseError`, collapsing semantic/catalog errors into parse failures. Complicates diagnostics and any core policy that treats parse vs schema errors differently.
- **Suggestion:** Preserve error kinds or use a dedicated frontend error variant.
- **Status:** open — triaged, not addressed in this pass.

#### Issue 25 — Severity: suggestion
- **File:** `graph/frontend/src/session.rs` / `compiler.rs` — single `graph-cypher` FrontendId
- **Description:** Only one graph frontend compiler per connection. Second `GraphSession::install` fails with already-registered; `Drop` unregisters the shared id. Multi-graph-per-connection is unsupported and surprising.
- **Suggestion:** Multiplexer compiler, or `FrontendId` per graph name; document single-graph session limit.
- **Status:** open — triaged, not addressed in this pass.

#### Issue 26 — Severity: suggestion
- **File:** `graph/runtime/src/csr.rs` — `NeighborCursor`
- **Description:** `NeighborCursor::step` is `pub(crate)`, so external callers are pushed toward allocating `Graph::neighbors`. Incomplete public streaming API.
- **Suggestion:** Expose a public step/iterator API consistent with traversal budgets.
- **Status:** open — triaged, not addressed in this pass.

#### Issue 27 — Severity: suggestion
- **File:** `graph/ir/src/plan.rs` validation
- **Description:** `Plan::new` validation is shallow (result-shape bindings). Hop ranges, binding visibility across operators, and UNION type agreement are binder-enforced only — IR can represent invalid plans if constructed outside the binder.
- **Suggestion:** Strengthen IR invariants for production plan construction; keep binder as primary producer.
- **Status:** open — triaged, not addressed in this pass.

#### Issue 28 — Severity: nit
- **File:** `graph/frontend/src/session.rs:227–249`
- **Description:** `strip_explain_prefix` allocates `to_ascii_lowercase()` on the remainder each keyword loop iteration.
- **Suggestion:** Prefix checks with `eq_ignore_ascii_case` / manual ASCII folding without full-string allocation.
- **Status:** fixed — prefix keywords checked with `eq_ignore_ascii_case` on slices; no per-iteration lowercase allocation.

#### Issue 29 — Severity: nit
- **File:** `graph/frontend/src/session.rs:263–264`
- **Description:** `NonZero::new(raw_index).expect("parameter indexes start at one")` on a production path. Index is trusted from `1..=count` but `expect` is still a panic surface.
- **Suggestion:** `ok_or_else` → internal error.
- **Status:** fixed — `expect` replaced with a skip on the (unreachable) zero index; no panic surface.

#### Issue 30 — Severity: nit
- **File:** pest grammar / power operator
- **Description:** `^` associativity is left-associative in the grammar; mathematical/Cypher convention is right-associative (`2^3^2 = 2^(3^2)`).
- **Suggestion:** Fix grammar associativity and add a parser fixture.
- **Status:** fixed — `power_expression` now folds right-associatively in the parser (`2^3^2 = 2^(3^2)`). Fixture: `power_operator_is_right_associative`.

#### Issue 31 — Severity: suggestion
- **File:** binder unknown function handling
- **Description:** Unknown function names bind as `Any` rather than erroring. Soft-fail aids corpus progress but defers failures to runtime and hides typos.
- **Suggestion:** Strict mode for production sessions; keep permissive mode for corpus triage if needed.
- **Status:** open — triaged, not addressed in this pass.

### Document alignment

#### Issue 32 — Severity: bug
- **File:** `graph/CONFORMANCE.md`, `graph/README.md`, `graph/CYPHER_CORPUS_GAPS.md`
- **Description:** Published conformance still claims a strict run of **10,392** identities with **1,413 passed / 8,979 failed**. Live truth in `graph/test-results/REPORT.md` (run `20260721T112742…corpus-deep`) is **10,242** records with **8,711 passed / 1,531 failed**. README treats CONFORMANCE as the compatibility source of truth — agents and humans will understate maturity badly or chase phantom failures.
- **Suggestion:** Regenerate CONFORMANCE/GAPS from the latest REPORT; or demote CONFORMANCE to historical appendix and point README at REPORT.md + history.
- **Status:** fixed — CONFORMANCE.md regenerated from live data (10,242 identities; 8,800 passed / 1,442 failed, run 20260721T115532) and demoted to a summary that names `test-results/REPORT.md` as the source of truth; README points at REPORT.md; GAPS carries a historical-snapshot banner.

#### Issue 33 — Severity: bug
- **File:** `graph/LONG_TAIL.md`, `graph/DESIGN_DECISIONS.md`
- **Description:** Metrics and architecture claims lag code: older pass rates; `DESIGN_DECISIONS` still references non-existent `graph/duration` (real crate is `graph/temporal`); some option analyses still recommend designs already replaced by junction tables.
- **Suggestion:** Rebase long-tail triage on latest REPORT histogram; fix crate names; mark superseded decisions clearly.
- **Status:** fixed — LONG_TAIL.md carries a dated-snapshot banner pointing at REPORT.md; DESIGN_DECISIONS fixes `graph/duration`→`graph/temporal`, chrono→jiff, marks the reduce() hard-block superseded, and labels the option analysis historical.

#### Issue 34 — Severity: bug
- **File:** `docs/multi-frontend.md`, `docs/plans/2026-07-17-graph-*.md`
- **Description:** Plan/architecture docs still speak in present tense about Ladybug, empty `graph/testdata/conformance/`, an 18-scenario deep suite, and names like `ReprepareRecipe`. Code has `PreparedSource` + `FrontendCompiler`, Ladybug removed, deep corpus ~10k, GraphExpand shipped.
- **Suggestion:** Add a “status as of &lt;date&gt;” banner on plans; update multi-frontend § blockers that GraphExpand/reprepare recipe already address; keep plans archival with explicit superseded markers.
- **Status:** fixed — status-as-of-2026-07-21 banners added to all `docs/plans/2026-07-1*graph*` docs and `docs/multi-frontend.md` (Ladybug removed, ~10k corpus, PreparedSource/FrontendCompiler, GraphExpand shipped).

#### Issue 35 — Severity: suggestion
- **File:** `graph/memory-observability.md` vs `core` pragma + testkit
- **Description:** Design doc still frames later phases as planned and describes a different row schema than shipped `PRAGMA memory_stats`. Phase 2 appears shipped (harness records page_cache/wal); tests for the pragma are thin/missing; harness can zero stats on failure silently.
- **Suggestion:** Align the doc with the shipped pragma shape; add a unit/integration test; fail closed or warn when stats are unavailable during benches.
- **Status:** fixed (doc alignment) — memory-observability.md marks phase 2 shipped and documents the real `(stat, value)` row schema plus the thin-test / zeroed-stats caveats. Dedicated pragma tests and fail-closed harness behavior remain follow-ups.

#### Issue 36 — Severity: suggestion
- **File:** temporal install ownership vs `GraphSession`
- **Description:** Duration/temporal functions live in `turso_graph_temporal`, but only the testkit reliably installs them. Embedders using `GraphSession` alone can hit “runtime scalar function missing” (a top REPORT failure family: 269).
- **Suggestion:** Install temporal (and other required graph scalars) from `GraphSession::install` or document a single `install_graph_runtime(connection)` entrypoint.
- **Status:** fixed — `GraphSession::install` installs the temporal extension (which also provides `cypher_equals`); bare sessions no longer hit missing scalar functions. Regression: `install_registers_runtime_scalar_functions` (proven to fail without the install).

### Testkit / infrastructure

#### Issue 37 — Severity: bug
- **File:** `graph/testkit/src/{runner,tck,age,grafeo,cypherbench}.rs`
- **Description:** CypherBench has per-query timeouts; corpus/TCK/AGE/Grafeo paths generally do not. A single pathological traversal can hang a multi-hour corpus run. CypherBench timeout handling can abandon worker threads that still hold large graphs (process RSS leak across cases).
- **Suggestion:** Global per-query deadline with interrupt; join/shutdown workers after timeout; bound peak concurrency.
- **Status:** open — triaged, not addressed in this pass.

#### Issue 38 — Severity: suggestion
- **File:** `graph/testkit` history append
- **Description:** History uniqueness checks re-read full JSONL (can be very large) on append — O(n) I/O per write.
- **Suggestion:** Append-only with external index, or bloom/set of recent run ids.
- **Status:** open — triaged, not addressed in this pass.

#### Issue 39 — Severity: suggestion
- **File:** mutation DISTINCT / ORDER BY comparison helpers
- **Description:** Mixed numeric ordering uses `i64 as f64` paths that lose precision above 2^53 and can disagree with SQL ORDER BY on the read path.
- **Suggestion:** Shared Cypher order implementation with integer-safe comparisons.
- **Status:** open — triaged, not addressed in this pass.

---

## Cross-cutting risk register

| Risk | Impact | Likelihood | Mitigations already present | Gap |
| --- | --- | --- | --- | --- |
| Soft memory limits | Process OOM / host thrash | Medium–high on large graphs | Budgets, cancel, defaults | Undercount + full snapshot materialization |
| Multi-source silent wrong table | Data corruption / lost deletes | Medium if multi-source used | Registration allows N sources | Runtime only uses first |
| MERGE type confusion | Wrong graph topology / types | Medium | Type junction for MATCH | MERGE omits type predicates |
| Stale published conformance | Bad product/agent decisions | High | Live REPORT.md | CONFORMANCE not regenerated |
| Missing temporal install | Corpus “function missing” | High in embedders | Testkit installs | Session path incomplete |
| Unbounded `*` → 64 hops | Silent incomplete results | High for openCypher users | Traversal max_hops budget | No user-visible diagnostic |

---

## Recommended priority order

1. **P0 — correctness:** Issues 1 (MERGE type), 2 (multi-source), 3 (aggregates), 4 (unbounded hops policy), 5 (deep equality completeness), 6 (`mutate` clear), 7 (nested BEGIN).
2. **P0 — honesty:** Issues 32–34 (docs/metrics); Issue 36 (temporal install).
3. **P1 — safety/perf:** Issues 13–16 (memory accounting & snapshot build), 17–19 (uniqueness, neighbors, mutation N+1), 37 (testkit timeouts).
4. **P1 — semantics polish:** Issues 9–12 (nullability, temporal TZ, ORDER BY alias, quantifier shadow).
5. **P2 — hygiene:** Issues 22–31 (binder split, error mapping, API completeness, nits).

---

## Suggested verification after fixes

```bash
# Focused crate tests
cargo test -p turso_graph_runtime
cargo test -p turso_graph_frontend
cargo test -p turso_graph_ir
cargo test -p turso_graph_cypher
cargo test -p turso_graph_temporal

# Core multi-frontend / pragma
cargo test -p turso_core --test integration frontend_reprepare multi_frontend_doc pragma

# Lint bar used by the repo
cargo fmt --check
cargo clippy -p turso_graph_frontend -p turso_graph_runtime -p turso_graph_ir \
  -p turso_graph_cypher -p turso_graph_temporal --all-targets -- --deny=warnings

# Corpus truth after doc regen
cargo run -p turso_graph_testkit -- …   # project’s corpus entrypoint
# then refresh graph/CONFORMANCE.md from graph/test-results/REPORT.md
```

Regression fixtures that should exist for the P0 bugs:

- `MERGE` same endpoints, two relationship types — must not match the wrong type.
- Multi-source register + MATCH/CREATE/DETACH on non-first source.
- `RETURN count(*) + 1` and `RETURN sum(x) * 2` grouping.
- `MATCH ()-[*]->()` either errors with bound guidance or documents and enforces the 64-hop cap loudly.
- `mutate` success when snapshot lock is poisoned (clear failure must not flip Ok→Err).
- Equality: list/map parameters and UNWIND values under three-valued logic.

---

## Scope notes / out of scope

- Full line-by-line review of ~281k inserted lines (donor fixtures, expected outputs, lockfiles, version bumps) was not performed; testdata/donors were treated as imported golden material unless harness code mishandles them.
- Non-graph mainline merges on the branch (JS promise/async txn, windows locking, mvcc logical log serializer, sqltest rename, etc.) were not the focus of this review; spot-check those if this branch is the sole merge vehicle to main.
- This document is findings-only; no code was modified.

---

## Issue index by severity

| Severity | Count | IDs |
| --- | ---: | --- |
| bug | 20 | 1–14, 32–34, 37 |
| suggestion | 16 | 15–27, 31, 35–36, 38–39 |
| nit | 3 | 28–30 |

**Total: 39 issues.**

---

## Appendix A — Architecture map (as implemented)

```
Cypher text
  → turso_graph_cypher (pest AST)
  → turso_graph_frontend::bind / bind_mutation  →  turso_graph_ir plans/mutations
  → lower_relational / mutation executor  →  turso_parser AST / SQL
  → core prepare / VDBE
Variable-length hops
  → __turso_graph_expand internal vtab
  → SnapshotStore / SessionSnapshotStore (CSR Graph)
  → turso_graph_runtime::TraversalCursor
```

## Appendix B — Key files reviewed

| Area | Paths |
| --- | --- |
| Core frontend | `core/frontend.rs`, prepare/reprepare hooks in `core/connection.rs`, `core/vtab.rs` |
| IR | `graph/ir/src/*` |
| Runtime | `graph/runtime/src/{csr,traversal,shortest,limits,error,lib}.rs` |
| Frontend | `graph/frontend/src/{lib,compiler,session,snapshot,graph_expand,mutation,lowering,catalog,schema_catalog,binder,functions}.rs` |
| Parser/temporal | `graph/cypher/src/*`, `graph/temporal/src/lib.rs` |
| Docs/results | `graph/{README,CONFORMANCE,DESIGN_DECISIONS,LONG_TAIL,memory-observability}.md`, `graph/test-results/REPORT.md`, `docs/multi-frontend.md` |
| Testkit | `graph/testkit/src/*` (structure & risk) |

## Appendix C — Sub-review artifacts

Parallel deep-dives used while authoring this document (scratch, not canonical):

- runtime/ir review notes
- frontend orchestration review notes
- binder/parser/temporal review notes
- docs/testkit alignment review notes

This file is the single consolidated deliverable.
