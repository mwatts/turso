# Graph Frontend Deep Quality & Structure Review

| Field | Value |
| --- | --- |
| **Branch** | `feature/graph-frontend` |
| **Head** | `bb5bfe094` |
| **Base** | `origin/main` (merge-base `a7c09f13aaf5`) |
| **Date** | 2026-08-10 |
| **Mode** | Read-only; no code changes |
| **Prior review** | `BRANCH_QUALITY_REVIEW.md` (2026-07-21) — P0 issues 1–8 fixed; residual P1/P2 re-checked |
| **Corpus truth** | `test-results/REPORT.md`: **9,069 / 10,242** passed, 53 unsupported, **1,120** failed |
| **Bar** | Rust quality, graph-DB correctness, feature structure, unused Turso core leverage, algorithm/SQL risk |

---

## Summary

The graph frontend is a **serious, production-shaped multi-crate system**: Cypher (pest) → typed IR → SQL AST / multi-statement mutations through `turso_core`, fixed hops as relational joins, variable hops via resumable `__turso_graph_expand` over an in-memory CSR snapshot. Layering is intentional and mostly enforced by Cargo. Recent work is high quality where it landed: change tokens instead of generation triggers, statement cache for constraint probes, filter pushdown + parenthesized-operand index seeks, recursive-CTE `reduce()`, covering `count(*)`, hop-cap honesty for unbounded `*`, and a carefully versioned path policy.

The dominant remaining problem is **not “missing parser features”** — it is a **split brain between the read pipeline and the mutation pipeline**, plus a few **invalidation and materialization bugs** that produce silent wrong graph state rather than clean errors.

| Strength | Residual risk |
| --- | --- |
| Core multi-frontend seam (`FrontendCompiler`, reprepare) | Mutations never become a single `PreparedSource` |
| Role model (n-ary, spill tables, name-based roles) | Cardinality constraints still binary `start`/`end` only |
| Change-token invalidation (triggers deleted) | Tokens omit spill + type-junction tables |
| Unbounded hop limits now error (not silent truncate) | Memory budgets remain soft estimates |
| Corpus 88.5% pass rate | 191 “mutation projection unsupported”, 135 missing scalars, 112 grammar gaps |

**Verdict:** ship-quality foundation for **fixed-pattern Cypher over application tables**, with honest resource caps on traversal and a correct role/storage story. Not yet a complete graph product: mutation correctness (DETACH type cleanup, Debug-keyed DISTINCT/grouping), snapshot invalidation holes, unordered path aggregation, and O(N²) constraint validation under bulk load are the main blockers before treating corpus progress as openCypher completeness.

---

## Architecture (as implemented)

```text
turso_graph_cypher  →  parse / AST / spans
        ↓
turso_graph_ir      →  identities, PlanKind, mutations, SEMANTIC_PROFILE
        ↓
turso_graph_frontend
   bind → lower (reads) → prepare_frontend → VDBE
   bind → execute_bound (writes) → many prepare_internal → validate_state
   catalog / semantic / FTS / expand vtab / session
        ↓
turso_graph_runtime → CSR, traversal, shortest, path_policy
        ↓
turso_core          → storage, translate, VDBE, change tokens, FTS, JSON, recursive CTE
```

| Crate | LOC (approx.) | Role |
| --- | ---: | --- |
| `frontend` binder | ~9,000 | Bind Cypher → IR (read + mutation) |
| `frontend` lowering | ~4,100 | IR → SQL (`cc≈308` on expression lower) |
| `frontend` mutation | ~3,600 | Multi-statement write executor |
| `frontend` semantic* | ~6,600 | Types, constraints, schema catalog |
| `frontend` snapshot | ~2,000 | CSR build + stores |
| `runtime` | ~2,700 | Adjacency + algorithms |
| `cypher` | ~2,800 | Pest grammar + AST |
| `ir` | ~1,500 | Stable contracts |

**Structural diagnosis:** IR and runtime are the right size. **Binder and mutation re-implement half of a query engine in Rust**, so every new Cypher clause costs twice (read path + write path) and the write path is weaker (ORDER BY, DISTINCT, aggregates, prepare reuse).

---

## What looks strong

1. **Multi-frontend boundary** — `GraphCompiler` is connection-stateless for reprepare; reads go through `prepare_frontend("graph-cypher")`; no direct VDBE emission.
2. **Role model discipline** — storage follows cardinality; roles resolve by name/id; binary is convenience, not a special IR kind; DETACH materializes relation ids before mutating spill tables (correct self-reference hazard).
3. **Invalidation evolution** — generation triggers deleted; `Connection::table_change_token` + `schema_generation` split; session statement cache for repeated probe/constraint SQL.
4. **Traversal honesty (partial)** — unbounded `*` hits hop cap with `LimitExceeded`, not silent drop; path policy refuses infinite ALL+Walk and NP-hard negative-weight simple paths.
5. **Main merge leverage** — recursive CTE `reduce()`, covering counts, filter-into-join for index seek, parenthesized comparison fix (documented in `MAIN_MERGE_LEVERAGE.md` / `PERFORMANCE_BACKLOG.md`).
6. **Safety defaults** — `#![forbid(unsafe_code)]` on graph crates; SQL identifiers quoted; values mostly parameterized on hot mutation paths.
7. **Testing surface** — unit/integration tests in crates + large donor corpus + divergence registry; performance and memory tooling exist.

---

## Issues

Severity: **bug** = correctness/safety contract break; **suggestion** = important quality/perf/structure debt; **nit** = polish.

Status for all items below: **open** unless noted. Line numbers are approximate anchors on `bb5bfe094`.

---

### Correctness

#### Issue 1 — Severity: bug
- **File:** `graph/frontend/src/mutation.rs:2344–2415` (compare direct relationship delete at `2462–2478`)
- **Description:** `DETACH DELETE` on a node purges Many-role spill rows and deletes relationship table rows, but **never deletes rows from `relationship_types_table`**. Direct relationship `DELETE` does clear the type junction. After DETACH, orphan type-membership rows remain; subsequent type-filtered scans / procedures / CSR type filters can see ghost types or identity collisions.
- **Suggestion:** After capturing `matched_ids`, also `DELETE FROM types_table WHERE relationship_id IN (…)` (with `source_id` when source-qualified), mirroring the non-detach path. Regression: DETACH then `db.relationshipTypes` / type-filtered MATCH count.
- **Evidence:** DETACH block ends at relation-row delete; types cleanup exists only on the relationship-entity branch.

#### Issue 2 — Severity: bug
- **File:** `graph/frontend/src/catalog.rs:493–514` (`derive_generation`)
- **Description:** Derived generation hashes `schema_generation` plus **node and relationship base tables only**. Many-role **spill tables** (`{relation}__{role}`) and **type/label junction tables** are omitted. Spill-only DML (or type-junction updates without touching the base relationship row) leaves `derived_generation` unchanged → **stale CSR** for variable-length expand.
- **Suggestion:** Include every spill table and membership/type/registry tables in the token set. Pin with: write spill via SQL, re-run expand, assert path change without base-table write.
- **Note:** Triggers used to bump on any mapped-table writer; the token design is correct but the **table set is incomplete** for n-ary storage.

#### Issue 3 — Severity: bug
- **File:** `graph/frontend/src/lowering.rs:1121–1145`
- **Description:** Variable-length path / relationship-list materialization uses `json_group_array(gx.node_identity)` / `relationship_identity` **without `ORDER BY path_position`** inside the aggregate. Grouping by `path_id` does not guarantee hop order → `nodes(p)`, `relationships(p)`, and named `*`-lists can reorder hops (wrong path values).
- **Suggestion:** Aggregate over a subselect ordered by `path_position`, or use ordered aggregation if/when core supports it. Multi-hop fixture with distinct node ids per hop.

#### Issue 4 — Severity: bug
- **File:** `graph/frontend/src/mutation.rs:414–417`, `900–913`, `1001–1014`, `1087–1097`
- **Description:** Mutation-path `RETURN DISTINCT`, stage DISTINCT, and aggregation grouping keys use `format!("{row:?}")` / `format!("{:?}@…")`. **Debug is not a value-equality contract** (float formatting, Blob/Text encoding, enum Debug changes). Can drop unequal rows, keep duplicates, or merge distinct groups.
- **Suggestion:** Canonical Cypher equality / typed encode for group keys (same rules as the read path where possible).

#### Issue 5 — Severity: bug
- **File:** `graph/frontend/src/mutation.rs:1139–1145`, `497–523`, `1121–1134`
- **Description:** Mutation-path aggregates and sort reimplement SQL poorly:
  - **AVG** divides by all non-null values while summing only numerics → text/blob shrink the average.
  - **ORDER BY** promotes all numerics to `f64` → integers outside 2^53 mis-order; NaN collapses to Equal.
  - **SUM** of integers uses wrapping `sum::<i64>()` in release.
- **Suggestion:** Align with SQL/`Numeric` semantics, or lower RETURN/WITH aggregates to SQL instead of Rust.

#### Issue 6 — Severity: bug
- **File:** `graph/frontend/src/mutation.rs:153–157`
- **Description:** `decode_mutation_rows` uses `assert_eq!` on column count. Lowering/metadata drift **panics the process** instead of returning `MutationError`.
- **Suggestion:** Soft error with diagnostic (expected vs actual arity).

#### Issue 7 — Severity: bug
- **File:** `graph/frontend/src/mutation.rs:2090–2112`, `2225–2278`
- **Description:** Relationship MERGE:
  - **Many-role match** is EXISTS-per-player (subset), not exact multiset of role players.
  - Match is **check-then-insert** without UPSERT / unique key → concurrent MERGEs can double-insert.
  - Property-less match uses `1=1 LIMIT 1` (Cypher-legal but non-deterministic under multi-row sources).
- **Suggestion:** Exact player-set match for Many roles; physical unique keys + `ON CONFLICT` / retry for atomic MERGE; optional fail-closed empty merge key in semantic mode.

#### Issue 8 — Severity: bug (incomplete Cypher)
- **File:** `graph/frontend/src/binder.rs` (~2480 REMOVE property only)
- **Description:** `REMOVE n.prop` works; openCypher **`REMOVE n:Label`** has no bind/IR/execute path. Labels are additive via SET only → silent “not supported at bind” or missing feature depending on parse path.
- **Suggestion:** `RemoveLabels` IR + junction deletes, or hard bind error with a clear message if deferred.

#### Issue 9 — Severity: bug (semantic polish, still open from July)
- **File:** binder nullability for `labels()` / entity helpers under OPTIONAL MATCH
- **Description:** Introspection results typed `NonNull` when the entity binding is nullable; openCypher propagates null (`labels(null) → null`).
- **Suggestion:** Propagate argument nullability; OPTIONAL MATCH fixtures.

#### Issue 10 — Severity: bug (still open from July)
- **File:** mutation RETURN `ORDER BY` vs read-path projection binding
- **Description:** Mutation RETURN sort keys do not share read-path alias substitution → RETURN aliases unusable or wrong on the write path.
- **Suggestion:** Share projection/sort scope rules between read and mutation binders.

#### Issue 11 — Severity: bug (still open from July)
- **File:** quantifier / list-comprehension alias substitution
- **Description:** Loop-variable name collision can disable **all** alias rewrites in the body, not only the colliding name.
- **Suggestion:** Shadow per name only.

#### Issue 12 — Severity: bug (resource honesty)
- **File:** `graph/runtime/src/limits.rs`, `graph/runtime/src/csr.rs:430–441`, `graph/frontend/src/snapshot.rs` build path
- **Description:** Memory budgets charge `size_of` × lengths and omit Vec capacity, HashMap overhead, and (critically) **full `Vec<Vec<Value>>` materialization during snapshot build**. Peak accounting can under-report multi-GB builds until after allocation.
- **Suggestion:** Stream rows into CSR; charge capacity + map overhead; document soft vs hard limits; optionally gate on `PRAGMA memory_stats`.

#### Issue 13 — Severity: bug (docs / agent truth)
- **File:** `graph/DESIGN_DECISIONS.md:28–29`, `graph/CONFORMANCE.md:38–48`, `graph/docs/core-changes.md` §2
- **Description:** DESIGN still cites ~6,161/10,392; CONFORMANCE “Current result” lags REPORT (8,926 vs live 9,069); `core-changes.md` still frames **generation triggers** as live core carve-outs though triggers are **deleted** and invalidation is `table_change_token`.
- **Suggestion:** Point static docs at REPORT.md; rewrite core-changes inventory to match landed design.

---

### Performance & algorithm

#### Issue 14 — Severity: suggestion
- **File:** `graph/frontend/src/semantic_constraints.rs:396–650`, `PERFORMANCE_BACKLOG.md` item 3
- **Description:** After each mutation, in-scope constraints re-scan source tables:
  - unique/key → `GROUP BY … HAVING COUNT(*) > 1`
  - value predicates → **pull every non-null property value into Rust** (no `LIMIT 1` early stop for the scan shape)
  - cardinality → full node scan + LEFT JOIN + GROUP BY per owner type  
  Bulk-loading N rows is **O(constraints × N²)** under the current design. Scope narrowing (by source table) helps type count, not row count.
- **Suggestion:** Validate affected identities only; push value predicates into SQL with `LIMIT 1`; defer validation to commit inside explicit transactions; physical UNIQUE indexes where multi-type membership allows.

#### Issue 15 — Severity: suggestion
- **File:** `graph/frontend/src/mutation.rs:2592–2616`, `session.rs:395–429`, `PERFORMANCE_BACKLOG.md` item 4
- **Description:** Mutations parse Cypher twice (`session` for snapshot need + `execute_cypher_mutation` for bind), never use the GraphCompiler compile cache, and **re-prepare every helper SQL per row/op** (`prepare_internal`). Statement cache only covers constraint/freshness SQL.
- **Suggestion:** Parameterized mutation plan cache keyed by `(source, schema_generation)`; reuse prepared helpers inside `execute_bound`.

#### Issue 16 — Severity: suggestion
- **File:** `graph/frontend/src/statement_cache.rs:32–37`, `66–70`
- **Description:** At capacity 64, the cache **clears the entire map**, thrashing the freshness probe and hot constraints when many distinct constraint SQLs exist.
- **Suggestion:** LRU / drop-one eviction; pin hot keys; size with constraint count.

#### Issue 17 — Severity: suggestion
- **File:** `graph/frontend/src/lowering.rs` default plan shape; `PERFORMANCE_BACKLOG` §6 (partially done)
- **Description:** Filter pushdown now works for scans/role expands, but many plan nodes still wrap derived tables; property access can fall back to **correlated subqueries** per occurrence; multi-pattern JOIN is often cartesian until a later filter. Core index / LEFT→INNER passes only fire when predicates name joined tables.
- **Suggestion:** Expose physical aliases further up; materialize wanted properties earlier; rewrite equi-joins across independent patterns when possible.

#### Issue 18 — Severity: suggestion
- **File:** `graph/runtime/src/traversal.rs` (uniqueness `Vec::contains`), `shortest.rs` (full `neighbors` Vec)
- **Description:** Trail/Path uniqueness is O(depth) per candidate edge; shortest-path APIs allocate full neighbor vectors instead of streaming `NeighborCursor`.
- **Suggestion:** HashSet/bitset uniqueness; cursor-driven Dijkstra/BFS.

#### Issue 19 — Severity: suggestion
- **File:** `graph/runtime/src/path_policy.rs`, `graph/frontend/src/graph_expand.rs`
- **Description:** Yen / ALL SHORTEST algorithms are **sound but unimplemented** (`PathAlgorithmNotImplemented`). Expand always uses BFS; under path/work/memory limits DFS enumeration would produce different truncated sets. Cypher has no SHORTEST/ALL SHORTEST/TRAIL syntax yet — library surface is ahead of language surface.
- **Suggestion:** Keep stubs until language lands; pass traversal order from policy; implement ALL SHORTEST before Yen.

#### Issue 20 — Severity: suggestion
- **File:** `graph/frontend/src/lowering.rs:1062`, `1169–1173`
- **Description:** Graph expand embeds `TraversalLimits::default()` as SQL literals; session limits / PRAGMA cannot retune without re-lower. Work quantum fixed in expand cursor.
- **Suggestion:** Session-level limit parameters on expand; document defaults in EXPLAIN.

#### Issue 21 — Severity: suggestion
- **File:** `graph/frontend/src/fts.rs`, internals docs “FTS-driven outer scan”
- **Description:** FTS registers core indexes and lowers a rowid-set subquery, but layered plans still **outer-scan the node relation**, so FTS does not replace the scan.
- **Suggestion:** Drive outer MATCH source from FTS rowid set (preferred over a second procedure surface).

#### Issue 22 — Severity: suggestion
- **File:** `graph/frontend/src/snapshot.rs` shared vs session stores
- **Description:** After commit, shared `SnapshotStore` stays stale until explicit refresh; each connection rebuilds local CSR → **O(connections × |V|+|E|)** after writes that touch variable-length queries.
- **Suggestion:** Publish on autocommit-visible rebuilds; local overlay only for dirty transactions.

---

### Rust hygiene & API structure

#### Issue 23 — Severity: suggestion
- **File:** `binder.rs` (~9k), `lowering.rs` (~4k), `mutation.rs` (~3.5k)
- **Description:** Three monoliths with extreme cyclomatic density (`lower_expression_with_references` `cc≈308`, `lower_plan` `cc≈102`, snapshot build `cc≈123`). Hard to review, hard to keep read/write paths consistent, high regression risk.
- **Suggestion:** Split by concern: `bind_match`, `bind_mutation`, `bind_expr`, `lower_scan`, `lower_expand`, `lower_expr`, `mutation_ops`, `mutation_project`. Keep IR as the only shared vocabulary.

#### Issue 24 — Severity: suggestion
- **File:** `graph/frontend/src/mutation.rs` (many `.expect` on spill/catalog), `semantic_constraints.rs`
- **Description:** Catalog invariants use `expect` in mutation hot paths. Corrupt or incomplete catalog panics instead of typed errors.
- **Suggestion:** Map to `MutationError` / `SemanticCatalogError`.

#### Issue 25 — Severity: suggestion
- **File:** `graph/frontend/src/compiler.rs:102–144`
- **Description:** Compile cache is **single last source** only; bind/lower failures map to `LimboError::ParseError`, collapsing semantic vs parse errors.
- **Suggestion:** LRU keyed by `(source, schema_generation)`; preserve error kinds through Core if possible.

#### Issue 26 — Severity: suggestion
- **File:** `session.rs` `CatalogFreshness::DataGeneration`, `install` → `CallerOwned`
- **Description:** Pre-`schema_generation` DBs reload full catalog after every mutation generation probe path is heavy; `install` never auto-reloads when another connection changes registration.
- **Suggestion:** Always prefer cheap schema probe after migration; document install ownership hard; optional generation tracking for install mode.

#### Issue 27 — Severity: suggestion
- **File:** `semantic_constraints.rs` endpoint cardinality
- **Description:** Cardinality constraints hard-code roles named `start`/`end` while the engine is role-general — last major place the overlay is narrower than the storage model (`docs/graph-internals.md` already calls this out).
- **Suggestion:** Cardinality on arbitrary `RoleId`.

#### Issue 28 — Severity: nit
- **File:** `mutation.rs:2638–2648`, savepoint name constants, parameter map sniffing
- **Description:** Mutation `bind_parameters` silently skips unknown names; fixed savepoint names risk nested collision if call graph grows; mutation param typing sniffs `{…}` text as Map.
- **Suggestion:** Assert all parameters bound; unique savepoint names; require declared parameter types for mutations.

---

## Graph database implementation assessment

| Concern | Assessment |
| --- | --- |
| **Physical model** | BYO tables + label/type junctions + spill tables — good SQLite-native story; no private graph file format |
| **Identity** | Table-local coordinates (correct); REAL identity encoding made total (July fix) |
| **Labels / types** | Junction tables + membership predicates; multi-label OK; REMOVE label missing |
| **Relationships** | Role-general n-ary; arrow sugar requires `start`/`end`; MERGE type-matching fixed in July |
| **Variable-length paths** | CSR + expand vtab is the right shape; invalidation + ordered path materialization still broken |
| **Constraints** | Post-write, frontend-only integrity; direct SQL bypass is documented — OK if product messaging is clear; concurrent MERGE/unique not physically enforced |
| **Transactions** | Autocommit IMMEDIATE / savepoint / reject deferred BEGIN — correct under `prepare_internal` |
| **MVCC** | Session tests exist; MERGE/constraints are not MVCC-aware (TOCTOU) |
| **Catalog evolution** | Additive-only semantic registration — deliberate; non-additive evolution still future work |

---

## Unused / underused Turso core features

| Core capability | Graph usage | Opportunity |
| --- | --- | --- |
| **`prepare` / statement reuse** | Reads: frontend reprepare; constraints: session cache; **mutations: re-prepare every helper** | Biggest runtime win |
| **`table_change_token`** | Used for derived generation | **Incomplete table set** (spill, types, labels) |
| **UPSERT / ON CONFLICT** | Unused on MERGE | Atomic MERGE under concurrency |
| **Physical UNIQUE / partial indexes** | Semantic unique is scan-only | Enforce uniqueness + speed validation |
| **Recursive CTE** | Used for `reduce()` | Not used for VLE (CSR preferred — OK) |
| **Window functions** (`lag`/`lead`/`ntile`/`percent_rank`) | Unused | Top-k-per-group / ranked paths without full materialize |
| **Covering indexes** | Pure `count(*)` paths fixed | More column-free aggregates |
| **JSON** (`json_each`, `json_group_array`, …) | Heavy use | Ordered aggregates for paths |
| **FTS index method** | Registered + lowered | Drive outer scan from rowid set |
| **Vector functions** | Exposed in binder/functions | Good; malformed blob fail-closed covered |
| **Custom types / domains** | Duration + property typing | Already shared via `classify_column` |
| **`FrontendCompilation` prerequisites** | Always empty | Optional multi-stmt prepare for labeled CREATE |
| **Multi-statement prepared programs** | Absent | Collapse mutation orchestration (largest structural gap vs PG frontend) |
| **PRAGMA optimize / ANALYZE** | Not driven after bulk load/register | Planner quality for junction/role indexes |
| **CDC / capture_data_changes** | Unused | Rejected for invalidation (too heavy) — correct |
| **MVCC journal mode** | Tested, not product-default for graph | Document interaction with MERGE races |
| **Error-raising SQL scalar** | Missing | Blocks ~19 TypeError-on-entity-in-list corpus cases (known hard block) |
| **Async `turso` wrapper** | None for graph (or PG) | Product gap, not engine gap |

---

## Algorithm & SQL risk register

| Risk | Level | Notes |
| --- | --- | --- |
| Stale CSR after spill / type-junction writes | **High** | Incomplete `derive_generation` tokens |
| DETACH type-junction orphans | **High** | Silent catalog dirt |
| Path list hop order | **High** | `json_group_array` without order |
| Mutation DISTINCT/group Debug keys | **High** | Silent wrong results |
| Constraint validation O(N²) | **High** at scale | Fine for tens of rows; wall at 50k+ |
| Soft memory + full snapshot materialize | **High** | Not OOM-safe |
| Concurrent MERGE duplicates | **Medium** | No unique key / UPSERT |
| Nested derived tables vs indexes | **Medium** | Partial pushdown only |
| BFS-only under enumeration limits | **Medium** | Different truncated sets vs DFS |
| Yen / ALL SHORTEST | **Medium** | Sound stubs only; no Cypher surface |
| Unbounded `*` hop cap | **Mitigated** | Errors instead of silent truncate |
| ALL + Walk infinite | **Mitigated** | Policy refuses |
| SQL injection | **Low** | Quote + escape discipline |
| Negative weights | **N/A today** | `u64` weights; policy ready if widened |

---

## Feature structure vs product goals

### Dual open modes (good)
- **Dialect-pinned** `open_database` → `"graph-cypher"` identity + temporal dialect surface.
- **Attach** on existing SQLite dialect → install compiler + extension + expand vtab.

Documented dual temporal resolution (Root dialect vs InternalHelper extension) is intentional and correct.

### Dual execution paths (structural debt)
| | Reads | Writes |
| --- | --- | --- |
| Entry | `prepare_frontend` | `execute_cypher_mutation` |
| Plan | single SQL AST | multi-statement orchestration |
| Cache | last compile outcome | none (helpers re-prepare) |
| RETURN ops | SQL ORDER/DISTINCT/LIMIT | Rust reimplementation (weaker) |
| EXPLAIN | via lowered SQL | not first-class as one program |

This is the **largest structure gap** vs Postgres frontend alignment (`docs/graph-frontend-core-alignment.md` §6.1). Closing it is a multi-PR project, not a tidy-up.

### Corpus failure shape (product prioritization)

From latest REPORT histogram:

| Family | Count | Implication |
| --- | ---: | --- |
| execution: other | 460 | Mixed semantic/runtime holes |
| mutation projection unsupported | 191 | Write RETURN surface incomplete |
| runtime scalar function missing | 135 | Function install/surface gaps remain |
| parser grammar | ~170 | Pest coverage still growing |
| mutation operation unsupported | 26 | Write feature holes (e.g. REMOVE label class) |

Prioritize **mutation projection + DETACH/invalidation correctness** before chasing long-tail grammar if the product claim is “Cypher over Turso tables.”

---

## Cross-check against July `BRANCH_QUALITY_REVIEW`

| July issue | Status on this head |
| --- | --- |
| MERGE type matching | **Fixed** |
| Multi-source first-only catalog | **Fixed** |
| Bare aggregate detection | **Fixed** |
| Silent unbounded hop cap | **Fixed** (errors now) |
| Mutate result overwritten by snapshot clear | **Fixed** |
| Shared refresh inside txn | **Fixed** |
| REAL identity non-total encoding | **Fixed** |
| Memory undercount / full snapshot materialize | **Still open** |
| Uniqueness O(depth), shortest neighbor Vec | **Still open** |
| labels() nullability, mutation ORDER BY, quantifier alias | **Still open** |
| Document drift | **Partially open** (REPORT better; DESIGN/core-changes lag) |
| Mutation N+1 prepare | **Still open** (constraints cached only) |

New critical findings since July: **DETACH type-junction cleanup**, **spill/type tokens missing from `derive_generation`**, **unordered path aggregation**, **Debug-keyed mutation DISTINCT/grouping**, **AVG/ORDER numeric bugs on mutation path**.

---

## Recommended priority order

1. **P0 correctness:** Issue 1 (DETACH types), Issue 2 (token table set), Issue 3 (path order), Issue 4 (Debug DISTINCT/group).
2. **P0 honesty:** Issue 6 (assert panic), Issue 12 (memory/snapshot peak).
3. **P1 scale:** Issue 14 (constraint row-scope), Issue 15 (mutation prepare reuse / double parse).
4. **P1 concurrency:** Issue 7 (MERGE atomicity + Many exact match).
5. **P1 structure:** Issue 23 (split binder/mutation modules), continue mutation→PreparedSource roadmap.
6. **P2 product:** REMOVE labels, FTS outer scan, ALL SHORTEST, window-powered top-k, docs refresh (Issue 13).
7. **P2 hygiene:** StatementCache LRU, soft expects, compile-cache LRU, cardinality on arbitrary roles.

---

## Suggested verification after fixes

```sh
# Focused
cargo test -p turso_graph_frontend --test native_capabilities
cargo test -p turso_graph_frontend --lib
cargo test -p turso_graph_runtime
cargo test -p turso_graph_frontend --test catalog_refresh
cargo test -p turso_graph_frontend --test constraint_validation_scope

# Regression classes for P0
# - DETACH then type-junction empty
# - spill write invalidates expand
# - multi-hop path nodes(p) order
# - DISTINCT with Blob/float twins

# Corpus (release, by design)
mise run corpus

# Hygiene
cargo fmt --check
cargo clippy -p turso_graph_frontend -p turso_graph_runtime -p turso_graph_ir -p turso_graph_cypher --all-targets -- --deny=warnings
```

---

## Scope notes

- Read-only review; **no code was changed**.
- Did not re-run full corpus in this session; pass counts quoted from `graph/test-results/REPORT.md` (run `20260808T143627…-bb5bfe094…-corpus-deep`).
- Did not re-audit entire testkit/donor trees in the same depth as frontend/runtime; testkit quality was covered in July (issues 37–39) and is not the primary risk now.
- Postgres frontend and non-graph core features mentioned only where graph fails to leverage them.

---

## Appendix A — Key files reviewed

| Path | Why |
| --- | --- |
| `graph/frontend/src/{binder,lowering,mutation,session,compiler}.rs` | Main pipelines |
| `graph/frontend/src/{catalog,schema_catalog,semantic,semantic_constraints}.rs` | Catalog + integrity |
| `graph/frontend/src/{snapshot,graph_expand,statement_cache,transaction,fts}.rs` | Derived state + helpers |
| `graph/runtime/src/{csr,traversal,shortest,path_policy,limits}.rs` | Algorithms |
| `graph/ir/src/*`, `graph/cypher/src/*` | Contracts / grammar |
| `graph/{DESIGN_DECISIONS,PERFORMANCE_BACKLOG,MAIN_MERGE_LEVERAGE,CONFORMANCE}.md` | Intent vs reality |
| `graph/docs/{core-changes,table-change-detection-design}.md` | Core seam history |
| `docs/graph-internals.md`, `docs/graph-frontend-core-alignment.md` | Architecture contracts |
| `graph/test-results/REPORT.md` | Live corpus thermometer |
| `core/connection.rs` (`table_change_token`), `core/frontend.rs` | Core primitives graph depends on |

## Appendix B — Issue index by severity

| Severity | Count (this review) |
| --- | ---: |
| bug | 13 |
| suggestion | 14 |
| nit | 1 |
| **Total numbered issues** | **28** |

(Plus algorithm/register items and unused-core table rows as non-numbered structural findings.)

---

*End of review. Single deliverable file; source tree otherwise untouched.*
