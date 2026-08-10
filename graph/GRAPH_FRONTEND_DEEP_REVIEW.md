# Graph frontend: quality audit and remediation plan

| Field | Value |
| --- | --- |
| Branch | `feature/graph-frontend` |
| Reviewed head | `bb5bfe094` (audit content); rewrite committed after that head |
| Base | `origin/main` (merge-base `a7c09f13aaf5` at audit time) |
| Audit date | 2026-08-10 |
| Mode | Read-only audit of existing code; this file is the only product of that pass |
| Prior audit | `graph/BRANCH_QUALITY_REVIEW.md` (2026-07-21); P0 issues 1–8 on that document are fixed on this head |
| Pass rate used as evidence | `graph/test-results/REPORT.md`, run `20260808T143627.062238Z-bb5bfe094633-corpus-deep`: **9,069 passed**, 53 unsupported, **1,120 failed** of 10,242 identities |

**Scope of this document.** It states what is wrong or incomplete in the graph crates today, what must become true after fixes, the contracts of the system as built, numbered requirements that an implementer can test, failure modes, non-goals, PR order, and open questions. It is not a product marketing brief and it does not re-run the full corpus.

**Terms used below (defined once).**

| Term | Meaning |
| --- | --- |
| Cypher | Graph query language parsed by `turso_graph_cypher` (pest grammar in `graph/cypher/src/cypher.pest`). |
| IR | Intermediate representation in `turso_graph_ir` (`Plan`, `Mutation`, identities). |
| Graph frontend | Crates under `graph/` that compile Cypher and orchestrate graph storage over Turso tables; entry types live in `turso_graph_frontend`. |
| Core | `turso_core`: storage, SQL translate, VDBE (virtual database engine), prepare APIs. |
| VDBE | Core bytecode interpreter that runs prepared SQL programs. |
| CSR | Compressed sparse row adjacency in `turso_graph_runtime::Graph` for variable-length walks. |
| Expand vtab | Internal virtual table `__turso_graph_expand` (`graph/frontend/src/graph_expand.rs`) that steps CSR paths from SQL. |
| Spill table | Table `{relation}__{role}` that stores Many-cardinality role players (`catalog.rs` install of spill tables). |
| Type junction | Catalog table that maps relationship identities to type names (`relationship_types_table` on the compilation catalog). |
| Label junction | Catalog table that maps node identities to label names (`labels_table`). |
| Change token | `Connection::table_change_token(table)` in core: per-process monotonic token advanced when a table may have changed after commit. |
| Derived generation | Hash in `catalog::derive_generation` used to decide if a CSR snapshot is stale. |
| Semantic constraints | Post-write checks in `SemanticConstraintSnapshot::validate_state` (required, unique, key, value, cardinality). |
| Read path | `GraphCompiler` → `Connection::prepare_frontend("graph-cypher")` → one SQL program. |
| Write path | `execute_cypher_mutation` → many `prepare_internal` SQL helpers → `validate_state`. |
| PreparedSource | Core type that records how a prepared program was compiled so reprepare can rebuild it. |

---

## 1. Problem

### 1.1 What fails today

Four defect classes still produce **wrong durable graph state** or **wrong query answers** without a clear Cypher error:

1. **DETACH DELETE leaves type-junction rows.** In `mutation.rs`, the DETACH branch for a node deletes spill rows and relationship table rows for matched relationship identities, then deletes the node and its label-junction rows. It does not delete matching rows from the relationship type junction. The non-DETACH relationship-entity branch does delete type-junction rows (`mutation.rs` around the `relationship_types_table` cleanup). After DETACH, type membership can outlive the relationship row.

2. **CSR invalidation ignores spill and membership tables.** `derive_generation` hashes `schema_generation` and `table_change_token` for each registered node and relationship **base** table only (`catalog.rs`). Spill tables and type/label junctions are omitted. A write that only touches a spill table or only the type junction can leave `derived_generation` unchanged. Variable-length expand then reuses a CSR that no longer matches tables.

3. **Path hop order is not forced in SQL.** Variable-length path materialization in `lowering.rs` builds `json_group_array(gx.node_identity)` and `json_group_array(gx.relationship_identity)` while grouping by `path_id`, without ordering by `path_position` inside the aggregate. SQLite/Turso do not promise aggregate input order. `nodes(p)`, `relationships(p)`, and named variable-length relationship lists can list hops out of walk order.

4. **Write-path DISTINCT and grouping key rows with Debug text.** `execute_cypher_mutation` and stage projection in `mutation.rs` use `format!("{row:?}")` (and similar) as set keys for DISTINCT and group-by. `Debug` is not Cypher equality. Float formatting, blob encoding, or Debug format changes can drop rows, keep duplicates, or merge groups that Cypher would keep separate.

Related write-path defects that return wrong numbers or crash the process:

- AVG on mutation stages divides by every non-null value while the sum skips non-numeric values (`mutation.rs` aggregate arm).
- Numeric ORDER BY on mutation RETURN compares via `f64`, so integers outside the exact float mantissa sort wrong, and NaN compares as equal.
- Integer SUM uses wrapping `sum::<i64>()` in release builds.
- `decode_mutation_rows` uses `assert_eq!` on column count and panics instead of returning `MutationError`.

### 1.2 Structural cost (not a silent wrong answer, but it blocks safe growth)

Reads and writes do not share one execution model:

| Concern | Read path | Write path |
| --- | --- | --- |
| Entry | `prepare_frontend` | `execute_cypher_mutation` |
| Program shape | One SQL AST / VDBE program | Many `prepare_internal` statements |
| Compile cache | Last source on `GraphCompiler` | None for the mutation itself |
| ORDER BY / DISTINCT / LIMIT on RETURN | SQL | Reimplemented in Rust on `MutationSummary.rows` |
| PreparedSource | Yes (frontend) | No single prepared source for the whole mutation |

`docs/graph-frontend-core-alignment.md` §6.1 already names this gap. The practical effect is that every new Cypher write feature needs a second, weaker implementation, and bugs cluster on the write path (REPORT: 191 failures tagged `mutation projection unsupported`, 26 `mutation operation unsupported`).

### 1.3 Scale cost with measured shape

`PERFORMANCE_BACKLOG.md` records a bootstrap case (in-memory DB, one node type, constraints on): a single CREATE compiled 21 SQL statements before the catalog-reload fix, 5 after; 20 CREATEs went from 420 compiles to 100. Item 3 on that backlog remains open: after each mutation, `validate_state` still scans whole source tables for in-scope constraints. The value-predicate branch selects every non-null property value into Rust and loops. Bulk load of N rows of one constrained type is **O(constraints × N²)** statement work, not O(N).

### 1.4 Evidence sources for this audit

- Code on head `bb5bfe094` under `graph/frontend`, `graph/runtime`, `graph/ir`, `graph/cypher`, and core hooks listed in appendix A.
- Live corpus thermometer: `graph/test-results/REPORT.md` (not the stale pass counts still printed in `DESIGN_DECISIONS.md` and the “Current result” block of `CONFORMANCE.md`).
- Prior fix status: July review P0s 1–8 (MERGE type match, multi-source catalog, aggregate detection, unbounded hop silence, mutate/snapshot clear ordering, nested BEGIN, REAL identity encoding) verified still fixed in current sources.

---

## 2. Intended end state

After the P0 and P1 work in §7 lands, these invariants hold:

**I1. DETACH cleanup is complete.** When DETACH DELETE removes a relationship identity `R` from a relationship source table, no row for `R` remains in that graph’s type junction (and spill rows for `R` are already removed, as today). Direct relationship DELETE and DETACH leave the same residual state for type membership of deleted relationships.

**I2. Derived generation moves when CSR inputs move.** Any committed write to a base node table, base relationship table, spill table, type junction, or label junction used by a registered graph changes `derived_generation` for that graph in the same process (or `table_change_token` returns `None` and the session treats the snapshot as untrusted and rebuilds). Variable-length expand never serves CSR edges that omit a committed spill player or type change that is visible to the same connection after commit.

**I3. Path lists preserve hop order.** For a variable-length path `p` with hop positions 0..k from the expand cursor, `nodes(p)` and `relationships(p)` list identities in increasing `path_position`. A test with distinct node ids per hop fails if order permutes.

**I4. Write-path DISTINCT and grouping use Cypher value equality.** Two rows that Cypher treats as equal are one group; two rows that Cypher treats as distinct remain two groups. Equality does not depend on Rust `Debug` text.

**I5. Mutation decode and catalog misses are errors, not panics.** Shape mismatch between lowered mutation columns and returned SQL rows returns `MutationError` with expected and actual arity. Missing spill table metadata returns a typed error, not `expect` panics, on the production mutation path.

**I6. Constraint validation cost scales with written identities when the statement reports them.** For CREATE/MERGE/SET that return or know affected ids, required/value checks do not re-scan the entire type membership for every unconstrained row of that type on every statement (see R14). Bulk load of N rows does not run N full-table unique scans that each cost Θ(N).

**I7. Documentation matches code for invalidation.** `graph/docs/core-changes.md` and static pass-rate blocks that agents read do not claim live generation triggers or obsolete corpus totals when REPORT.md and the token design say otherwise.

Until I1–I4 hold, the product must not treat “~88% corpus pass” as proof that variable-length paths, DETACH, or write RETURN DISTINCT are correct under multi-source or multi-hop data.

---

## 3. Design of the system as built

### 3.1 Crate contracts

```text
turso_graph_cypher     parse Cypher text → AST + spans
        ↓
turso_graph_ir         Plan, Mutation, GraphId / LabelId / RoleId, SEMANTIC_PROFILE
        ↓
turso_graph_frontend   bind, lower, mutate, catalog, snapshot, expand vtab, session
        ↓
turso_graph_runtime    CSR Graph, traverse, shortest_path, path_policy, BuildLimits
        ↓
turso_core             tables, prepare_frontend / prepare_internal, VDBE, tokens, JSON, FTS
```

Cargo enforces that `ir`, `cypher`, and `runtime` do not depend on `turso_core`. Only `frontend` prepares SQL.

Approximate sizes that drive review cost (line counts on this branch):

| Unit | ~LOC | Responsibility |
| --- | ---: | --- |
| `frontend/src/binder.rs` | 9,000 | Bind Cypher AST → IR for reads and writes |
| `frontend/src/lowering.rs` | 4,100 | IR plan → SQL / engine AST |
| `frontend/src/mutation.rs` | 3,600 | Write orchestration and in-Rust RETURN ops |
| `frontend` semantic + constraints + schema catalog | 6,600 | Types, fragments, validate_state |
| `frontend/src/snapshot.rs` | 2,000 | Load tables → CSR coordinates |
| `runtime` | 2,700 | Adjacency and path algorithms |
| `cypher` | 2,800 | Grammar and AST |
| `ir` | 1,500 | Shared contracts |

`lower_expression_with_references` has cyclomatic complexity about 308; `lower_plan` about 102; snapshot build about 123. Those functions are the first places wrong SQL or wrong path JSON appears.

### 3.2 Read path contract

- **Input:** Cypher text; optional named parameters on the session; catalog snapshot (`GraphCompilationCatalog`).
- **Output:** `FrontendCompilation` with engine SQL/AST; caller steps a core `Statement` and gets rows; Cypher result types recovered from `GraphCompiler::compile_outcome` / `take_result_types_for`.
- **Who prepares:** only `Connection::prepare_frontend` with frontend id from `graph_frontend_id()` (`"graph-cypher"`).
- **Success:** VDBE runs the lowered SQL; reprepare recompiles the same Cypher source through the registered compiler without reparsing Cypher as SQLite.
- **Failure:** parse/bind/lower map into core prepare errors (today often `LimboError::ParseError`, which collapses kinds).

Fixed-hop patterns lower to ordinary joins on endpoint columns or spill tables. Variable-length patterns lower to a join against `__turso_graph_expand(...)`, which reads a process-local CSR snapshot.

### 3.3 Write path contract

- **Input:** Cypher text; parameters; shared statement cache for constraint SQL only.
- **Output:** `MutationSummary` (counters + optional RETURN rows) after commit or savepoint release.
- **Transaction wrapper** (`transaction.rs` / `in_write_transaction`):
  - autocommit → `BEGIN IMMEDIATE` … `COMMIT` / `ROLLBACK`
  - existing write transaction → savepoint `__turso_graph_mutation` …
  - bare deferred `BEGIN` → `MutationError::RequiresWriteTransaction` (nested helpers cannot upgrade read → write under `prepare_internal`)
- **Success:** all role fills, junction rows, and spill inserts for one `run()` commit or roll back together; then `validate_state` runs inside the same transaction window when semantic constraints exist.
- **Failure:** typed `MutationError` / constraint violation; local session snapshot store is cleared after return so uncommitted CSR does not leak across statements (clear failure must not overwrite a successful write result; July fix).

Closed CREATE of a single node with no MATCH/WITH/RETURN uses a faster branch (`try_single_program_mutation`) but still prepares separate label-junction inserts when labels exist (documented in mutation comments).

### 3.4 Role and storage contract

A relationship source declares ordered **roles** (name, targets, cardinality One or Many). Storage rule:

| Cardinality | Storage |
| --- | --- |
| One | Column on the relation table |
| Many | Spill table with indexes on `(relation_id, node_id)` and `(node_id, relation_id)` |

Roles resolve by `RoleId` or declared name, never by position in general machinery. Arrow Cypher `(a)-[:T]->(b)` requires roles literally named `start` and `end`. N-ary relations use standalone role patterns.

Identities are **table-local**: equal integer ids in two source tables are different graph entities.

### 3.5 Snapshot and invalidation contract

- CSR is derived, never durable. Rebuild from tables.
- **Session store:** can rebuild under a nested savepoint so a connection sees its own uncommitted writes for expand.
- **Shared store:** publishes only committed state; refresh refuses an open user transaction (`SnapshotError::RefreshInsideTransaction`).
- Staleness: compare snapshot’s stored source generation to `RegisteredGraph::derived_generation`.
- `derived_generation` today: hash of schema generation + change tokens of **base** node and relationship tables only (gap in §1.1).

### 3.6 Semantic integrity contract

Semantic types and constraints are additive registration. `validate_state` runs after mutation SQL inside the write transaction. Direct SQL against mapped tables bypasses Cypher membership and semantic validation; only physical SQLite constraints remain. That boundary is intentional and must stay stated in user docs.

### 3.7 What already meets a high bar

These behaviors are correct on this head and must stay:

- Multi-frontend registration: `GraphCompiler` is `Send + Sync` and connection-stateless for reprepare; frontend crates do not emit VDBE opcodes.
- Unbounded variable-length ranges (`*`, `*min..`) set an `unbounded` flag; hitting the hop resource cap errors with `LimitExceeded` instead of silent truncation (July fix + runtime tests).
- Relationship MERGE matches type junction predicates, not endpoints alone (July fix).
- Multi-source registration routes by source id rather than `.first()` only (July fix).
- Generation DML triggers removed; core `table_change_token` is the invalidation primitive (design in `graph/docs/table-change-detection-design.md` “What landed”).
- Session `StatementCache` reuses exact SQL text for catalog freshness probes and constraint queries; cache lives on the session because `Statement` holds `Arc<Connection>`.
- Filter predicates can push into the join SELECT so core index seek and LEFT→INNER rewrites see real table columns (`PERFORMANCE_BACKLOG.md` item 6; core drops redundant parentheses around comparison operands).
- `reduce()` lowers to `WITH RECURSIVE`; aggregates inside `reduce()` are rejected at bind with a Cypher-facing message.
- Path policy table refuses infinite ALL+Walk and negative-weight simple shortest; unimplemented algorithms return `PathAlgorithmNotImplemented`, not a fake result.
- Graph crates forbid `unsafe`; identifiers go through quote helpers; values prefer bound parameters on mutation helpers.

---

## 4. Requirements

Requirements are what remediations must make true. Each is testable. Severity: **must** = wrong results or data dirt if violated; **should** = scale, concurrency, or maintainability; **may** = polish.

### 4.1 Correctness (must)

**R1. DETACH clears type membership.**  
When DETACH DELETE captures relationship identities `ids` and deletes those rows from the relationship table, it also deletes from the graph’s relationship type junction where `relationship_id IN ids` (and `source_id` when source-qualified membership is on), same predicate style as the direct relationship DELETE branch in `mutation.rs`.  
**Test:** create typed relationship, DETACH DELETE an endpoint node, assert zero type-junction rows for the deleted relationship id; compare to direct `DELETE` of the relationship.

**R2. Derived generation includes every CSR input table.**  
`derive_generation` hashes change tokens for: every node source table, every relationship source table, every spill table name from registered Many roles, the labels junction if present, the relationship types junction if present, and the type registry table if the snapshot reads it.  
**Test:** insert only into a spill table (or only into the type junction) through SQL under a write transaction that commits; `derived_generation` changes; expand rebuilds and observes the new edge or type.

**R3. Ordered path aggregation.**  
Path and relationship-list aggregates over expand output are equivalent to aggregating rows ordered by `gx.path_position` ascending (for example, subquery `ORDER BY path_position` then `json_group_array`, or another ordered-aggregate form core accepts).  
**Test:** multi-hop pattern with distinct node ids; `nodes(p)` equals the hop sequence from the start node.

**R4. Typed DISTINCT and group keys on the write path.**  
Replace Debug-string keys in `execute_cypher_mutation` DISTINCT, stage DISTINCT, and WITH aggregate grouping with a total Cypher-compatible encoding of `Value` (or lower those ops to SQL).  
**Test:** DISTINCT over two blob values that Debug might alias; DISTINCT over floats that print the same but compare unequal if that is Cypher policy; group-by that must not merge unequal maps.

**R5. Mutation AVG, SUM, and ORDER BY match numeric Cypher/SQL rules.**  
AVG’s divisor counts only values that enter the sum (or rejects non-numeric). Integer ORDER BY compares as integers when both sides are integers. Integer SUM uses checked arithmetic or promotes and errors on overflow instead of wrapping in release.  
**Test:** AVG over mix of numbers and text; ORDER BY of integers larger than 2^53; SUM near `i64::MAX`.

**R6. Mutation row decode returns errors.**  
`decode_mutation_rows` returns `MutationError` when `row.len() != columns.len()`, including expected and actual lengths in the message. No `assert_eq!` on that path.  
**Test:** unit or injection that forces arity mismatch does not abort the process.

**R7. MERGE Many-role match is exact player multiset (or documented subset).**  
Default product rule: a relationship matches MERGE only if the multiset of players for each Many role equals the pattern’s players (count and identity), not merely EXISTS for each named player. If subset match is intentional, document it in `docs/graph.md` and the IR comments and pin a corpus expectation.  
**Test:** relation with three players in a Many role does not match MERGE that names two of them unless docs say subset.

**R8. Concurrent MERGE does not double-insert the same pattern under single-writer IMMEDIATE without a unique key, without documenting the race.**  
Prefer physical unique indexes on MERGE keys where semantic mode allows, plus `INSERT … ON CONFLICT` or retry. Until that lands, document that two writers can both create.  
**Test:** two connections, concurrent MERGE same endpoints and type, assert one row or assert documented conflict error.

**R9. REMOVE label either works or fails closed.**  
openCypher `REMOVE n:Label` either deletes the label-junction row (new IR + executor) or bind rejects with an explicit unsupported message. Silent no-op is forbidden.  
**Test:** bind/execute fixture for both outcomes.

**R10. OPTIONAL MATCH nullability for entity helpers.**  
`labels(n)`, `label(n)`, and peer helpers produce nullable result types when `n` is a nullable OPTIONAL binding; evaluation returns null when the entity is null.  
**Test:** OPTIONAL MATCH miss then RETURN labels(n).

**R11. Mutation RETURN ORDER BY shares alias rules with the read binder.**  
An alias introduced in RETURN is usable in ORDER BY on the write path with the same substitution rules as read projections.  
**Test:** `CREATE … RETURN n.name AS a ORDER BY a`.

**R12. List comprehension / quantifier alias shadowing is per name.**  
A loop variable that collides with an outer alias only shadows that name; other aliases still rewrite.  
**Test:** body that uses both the loop var and a different outer alias.

**R13. Memory and peak build accounting are honest or labeled soft.**  
Either stream snapshot build into CSR without retaining full `Vec<Vec<Value>>` for all sources at once, and charge Vec capacity and map overhead in limit counters, or document `BuildLimits::max_memory_bytes` / traversal memory as best-effort estimates and never claim hard OOM safety.  
**Test:** limit that should fail during build fails before multi-GB peak, or docs assert soft semantics and tests only check the documented estimate.

**R14. Docs that agents treat as truth track REPORT and tokens.**  
`DESIGN_DECISIONS.md` must not hardcode obsolete pass rates; point at REPORT.md. `CONFORMANCE.md` “Current result” either regenerates from the same history as REPORT or defers entirely. `graph/docs/core-changes.md` §2 describes deleted generation triggers as historical and names `table_change_token` as current invalidation.

### 4.2 Scale and structure (should)

**R15. Constraint validation scopes to affected rows when known.**  
When `execute_bound` knows written identities (RETURNING ids or equivalent), required and value checks filter `entity.identity IN (…)`. Unique/key checks restrict to values present on written rows when that is sound. Value SQL uses predicates in SQL with `LIMIT 1` where a single violation suffices.  
**Acceptance:** bulk N inserts of one constrained type are O(N) validation work, not O(N²), measured by SQL prepare/step counts or wall time on a fixed machine (method and numbers recorded in PERFORMANCE_BACKLOG).

**R16. Mutation helpers reuse prepared statements.**  
SQL templates whose text is stable across rows (identity-parameterized helpers) go through session `StatementCache` or a mutation-local cache. Cypher for a mutation is parsed once per `execute` call for both snapshot need and bind.  
**Acceptance:** steady-state CREATE with constraints does not re-prepare the same helper text per row.

**R17. StatementCache eviction is not full clear.**  
At capacity, drop one entry (LRU or random), not the entire map, or size capacity by constraint count so the freshness probe is not thrashing.  
**Test:** >64 distinct constraint SQL strings still keep the probe query warm.

**R18. Filter and property materialization keep indexable shapes.**  
Continue pushdown of predicates onto joined base tables. Prefer materializing wanted properties at scan/join time over correlated property subqueries for each projection site.  
**Test:** EXPLAIN QUERY PLAN or existing native_capabilities covering-index tests stay green; add a multi-hop property filter that seeks.

**R19. Trail/Path uniqueness and shortest neighbors avoid O(depth) contains and full neighbor Vecs on hot loops.**  
Use HashSet/bitset for uniqueness; drive Dijkstra/BFS from `NeighborCursor`.  
**Benchmark:** path fanout shapes in `turso_graph_runtime` benches.

**R20. Expand limits are session-controlled.**  
Traversal budget literals in expand SQL come from session `BuildLimits` / traversal limits, not only `TraversalLimits::default()` baked at lower time without a knobs story.  
**Test:** lower two sessions with different max_paths and observe different expand args or errors.

**R21. Shared CSR publish after commit.**  
When a connection rebuilds a visible snapshot after autocommit writes, it can publish to the shared store when current, so other connections do not each rebuild from scratch without coordination. Dirty transactions still use session overlay only.

**R22. Split binder and mutation modules by concern.**  
New code for match bind, mutation bind, expression bind, scan lower, expand lower, expression lower, mutation ops, and mutation project lives in separate modules that share only IR types. No requirement to finish the split in one PR; require that new features do not grow the monoliths without extraction.

**R23. Catalog and spill expects become typed errors** on mutation and constraint paths (`MutationError` / `SemanticCatalogError`).

**R24. Compile cache keys by source and schema generation** with multi-entry LRU; bind failures preserve kind where core allows.

**R25. Cardinality constraints apply to arbitrary `RoleId`**, not only physical roles named `start`/`end`.

**R26. FTS-backed MATCH can drive the outer node source from the FTS rowid set** so the index replaces the full node scan when the query is a pure FTS filter (see graph-internals future work). Prefer this over a second procedure API.

### 4.3 Core features graph does not use yet (optional backlog)

Use these only when a requirement above needs them; do not add them for fashion.

| Core feature | Graph today | When to use |
| --- | --- | --- |
| Statement reuse / prepare cache | Constraints yes; mutation helpers no | R16 |
| `table_change_token` | Base tables only | R2 |
| UPSERT / ON CONFLICT | Unused on MERGE | R8 |
| Physical UNIQUE indexes | Semantic unique is scan-only | R8, R15 |
| Recursive CTE | `reduce()` only | Keep CSR for variable-length paths unless a measured win appears |
| Window functions | Unused | Top-k / ranked path lowerings |
| Covering index shapes | Pure `count(*)` improved | More column-free aggregates |
| Ordered JSON aggregates | Unordered path lists | R3 |
| FTS index method | Registered | R26 |
| Multi-statement PreparedSource | Writes absent | Long-term write path convergence |
| Error-raising SQL scalar | Missing | ~19 corpus TypeError-on-entity-in-list cases |
| PRAGMA optimize after bulk load | Not driven | Planner quality after ontology load |
| CDC | Rejected for invalidation | Keep rejected: too heavy for this job |

---

## 5. Failure modes

| Mode | Observable effect | Handler today | Required change |
| --- | --- | --- | --- |
| DETACH without type cleanup | Type junction still lists deleted relationship | Silent | R1 |
| Spill write without token move | Expand returns old adjacency | Silent wrong paths | R2 |
| Unordered `json_group_array` | `nodes(p)` permutes | Silent | R3 |
| Debug DISTINCT key | Wrong multiset of RETURN rows | Silent | R4 |
| AVG non-numeric mix | Wrong average | Silent | R5 |
| Decode arity mismatch | Process panic | `assert_eq!` | R6 |
| Concurrent MERGE | Duplicate relationships | Silent possible | R8 |
| REMOVE label unsupported | Missing feature or vague bind error | Incomplete | R9 |
| Unbounded `*` hits hop cap | `LimitExceeded` / runtime error | Correct (keep) | none |
| ALL + Walk path combo | Policy error, not infinite search | Correct (keep) | none |
| Shared snapshot refresh in user txn | `RefreshInsideTransaction` | Correct (keep) | none |
| Semantic constraint fail mid-mutation | Rollback of savepoint / txn; no partial create | Correct (keep) | none |
| Direct SQL write to mapped table | Violations until a later Cypher mutation revalidates in scope | Documented boundary | Do not “fix” by installing silent triggers without a product decision |
| `table_change_token` is `None` (multiprocess WAL) | Must treat as changed / rebuild always | Safe direction | Keep conservative |
| Soft memory limit undercount | Process OOM before LimitExceeded | Soft | R13 |
| StatementCache full clear | Extra prepare storms | Perf only | R17 |
| Value constraint full column pull | High latency on large tables | Perf / timeout | R15 |

---

## 6. Non-goals and rejected alternatives

**Non-goals for the remediation track in §7:**

- Full openCypher parity in one pass. Corpus still has ~1,120 failures; grammar and missing scalars are real but secondary to I1–I4 for data trust.
- Emitting VDBE from the graph frontend. Lowering stays SQL/AST.
- Persisting CSR on disk. Snapshots stay rebuildable derived state.
- Moving Cypher parse into core dialect parse. Cypher stays on `FrontendCompiler`.
- Making semantic constraints enforce under raw SQL writers without an explicit product design (triggers, generated CHECK, or refused BYO writes).

**Rejected or deferred alternatives (with criteria):**

| Alternative | Why not now |
| --- | --- |
| Keep generation triggers for invalidation | Measured ~2× WAL frames on single-row inserts; core carve-outs removed; tokens already land |
| Equality of trigger generation and token | Token is per-table; only one direction is required (token moves when data moves) |
| Variable-length paths only as recursive SQL | CSR + yield vtab already cooperates with VDBE IO; recursive CTE kept for `reduce()` |
| Second FTS procedure API | Duplicates scalar FTS and splits optimization paths; prefer outer scan from rowid set (R26) |
| Implement Yen before language surface | Cypher has no SHORTEST k syntax yet; stubs return not-implemented |
| Full mutation as one PreparedSource in the first PR | Correct end state for alignment with Postgres frontend, but multi-PR; R1–R6 do not depend on it |

**July issues that stay fixed and are non-goals to reopen:** MERGE type matching; multi-source first-only catalog; bare-only aggregate detection; silent unbounded hop truncation; mutate result overwritten by snapshot clear; shared refresh opening nested BEGIN under user txn; non-total REAL identity encoding.

---

## 7. Tests and PR order

### 7.1 Proof commands

After each PR:

```sh
cargo test -p turso_graph_frontend --lib
cargo test -p turso_graph_frontend --test native_capabilities
cargo test -p turso_graph_runtime
cargo test -p turso_graph_frontend --test catalog_refresh
cargo test -p turso_graph_frontend --test constraint_validation_scope
cargo fmt --check
cargo clippy -p turso_graph_frontend -p turso_graph_runtime -p turso_graph_ir -p turso_graph_cypher --all-targets -- --deny=warnings
```

After any change that can affect path order, DETACH, invalidation, or write DISTINCT:

```sh
mise run corpus
```

Corpus recording is release-profile by design (`Agents.md` / mise tasks). Compare per-suite counts in REPORT.md, not only the headline total (TCK can flake ±2).

### 7.2 Required new tests (map to requirements)

| Requirement | Minimum new test |
| --- | --- |
| R1 | DETACH then empty type junction for deleted rel |
| R2 | Spill-only write moves derived generation / rebuilds expand |
| R3 | Multi-hop `nodes(p)` order |
| R4 | DISTINCT with values that Debug may collide |
| R5 | AVG / big integer ORDER BY / SUM overflow |
| R6 | Decode arity mismatch → error |
| R7 | Many-role MERGE exact multiset |
| R10–R12 | OPTIONAL labels null; RETURN alias ORDER BY; list-comp shadow |
| R15 | Bulk create compile/step counts or time bounds |

### 7.3 PR sequence

Each PR stays reviewable and leaves main green for graph crate tests.

1. **PR-A: Data dirt (R1, R2).** DETACH type-junction delete; expand `derive_generation` table set. No behavior change for happy-path CREATE. Tests for DETACH and spill invalidation.
2. **PR-B: Path order (R3).** Ordered path aggregation only. Multi-hop fixture.
3. **PR-C: Write equality and numbers (R4, R5, R6).** Typed keys; AVG/SUM/ORDER BY; soft decode errors. Mutation-focused unit tests.
4. **PR-D: Docs (R14).** REPORT pointers; core-changes inventory matches tokens.
5. **PR-E: Constraints scale (R15, R17).** Affected-id validation; LIMIT 1 value SQL; StatementCache eviction. Update PERFORMANCE_BACKLOG measurements.
6. **PR-F: Mutation prepare (R16).** Single parse; helper statement reuse. Steady-state CREATE metrics.
7. **PR-G: MERGE concurrency and Many match (R7, R8).** Exact multiset + unique/UPSERT or documented race + test.
8. **PR-H: Cypher holes (R9–R12).** REMOVE label; nullability; ORDER BY aliases; quantifier shadowing.
9. **PR-I: Honesty and structure (R13, R22–R24).** Stream snapshot or document soft limits; start module splits; typed expects; compile cache LRU.
10. **PR-J: Product follow-ons (R20, R21, R25, R26, optional core multi-stmt).** Session expand limits; shared publish; role-general cardinality; FTS outer scan; design spike for mutation PreparedSource (no obligation to finish in J).

Do not combine PR-A with PR-J. Do not land path-order changes without R3’s multi-hop test.

### 7.4 Priority if only one engineer is available

Order by wrong data first: **A → B → C → D → E → F → G → H → I → J**.

---

## 8. Open questions

1. **MERGE Many-role semantics:** exact multiset (R7 default) or subset EXISTS (current code)? Product must pick one; corpus AGE/TCK rows may disagree with Neo4j on edge cases.
2. **Empty MERGE key** (`1 = 1 LIMIT 1` on property-less merge): remain Cypher-legal non-determinism, or fail closed under semantic multi-row sources?
3. **Hard vs soft memory limits:** is OOM-before-error acceptable for experimental graph, or must snapshot build stream before any “production” claim?
4. **Mutation PreparedSource timeline:** is multi-statement core support a graph-only script, or a multi-frontend core project shared with Postgres?
5. **Partial indexes for semantic unique across types:** core partial-index expressiveness for junction membership is still a research item; until then unique stays query-time.
6. **install vs open catalog freshness:** should `GraphConnection::install` gain optional generation tracking, or stay caller-owned forever?
7. **Missing scalar family (135 REPORT rows):** inventory whether failures are unregistered functions, AGE adapter install gaps, or intentional unsupported; that inventory is not in this audit.

---

## Appendix A: Files that define the contracts above

| Path | Contract role |
| --- | --- |
| `graph/frontend/src/binder.rs` | Cypher → IR |
| `graph/frontend/src/lowering.rs` | IR → SQL; path aggregates; expand args |
| `graph/frontend/src/mutation.rs` | Write orchestration; DETACH; MERGE; DISTINCT |
| `graph/frontend/src/session.rs` | open/install; execute; catalog freshness |
| `graph/frontend/src/catalog.rs` | Registration; `derive_generation`; spill install |
| `graph/frontend/src/semantic_constraints.rs` | `validate_state` |
| `graph/frontend/src/snapshot.rs` | CSR build; session vs shared store |
| `graph/frontend/src/graph_expand.rs` | Expand vtab cursor |
| `graph/frontend/src/statement_cache.rs` | Exact-text prepare reuse |
| `graph/frontend/src/compiler.rs` | `FrontendCompiler`; last-source cache |
| `graph/runtime/src/{csr,traversal,shortest,path_policy,limits}.rs` | Adjacency and path budgets |
| `core/connection.rs` | `table_change_token`, `prepare_frontend`, `prepare_internal` |
| `core/frontend.rs` | Frontend compiler trait and PreparedSource |
| `docs/graph-internals.md` | Pipeline map and role invariants |
| `docs/graph-frontend-core-alignment.md` | Read/write alignment vs Postgres |
| `graph/PERFORMANCE_BACKLOG.md` | Measured prepare counts and open scale items |
| `graph/test-results/REPORT.md` | Live corpus counts |

## Appendix B: Severity count of open remediation items

| Class | Count of R-ids |
| --- | ---: |
| must (R1–R14) | 14 |
| should (R15–R26) | 12 |
| Core optional table | not numbered |

July open residuals re-homed into R10–R13, R15–R16, R19, R22, R24 rather than a second issue list.
