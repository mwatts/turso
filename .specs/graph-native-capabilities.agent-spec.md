---
task_id: graph-native-capabilities
complexity: high
risk: medium
ambiguity: medium
agent_pattern: pipeline
subagent_type: general-purpose
model: inherit
isolation: worktree
tools_required: [file_read, apply_patch, ripgrep, cargo, git]
estimated_tokens: 32000
timeout_minutes: 360
---

# TASK

Implement four Turso-native graph capabilities in dependency order: a typed procedure registry with `db.propertyKeys()`, graph full-text indexing over Turso FTS, portable `startNode()`/`endNode()`, and session-correct graph snapshot diagnostics.

# REQUIRED SKILLS

| Skill | Path | Relevance |
|-------|------|-----------|
| code-quality | `.claude/skills/code-quality/SKILL.md` | Required project rules for small, correct Rust changes |
| testing | `.claude/skills/testing/SKILL.md` | Required project test placement and intent rules |
| rust-best-practice | `/Users/markwatts/code/github/mwatts/turso/.agents/skills/rust-best-practice/SKILL.md` | Required before authoring Rust |
| rust | `/Users/markwatts/.agents/skills/rust/SKILL.md` | Rust API, error, and performance guidance |

**Directive**: The implementing agent MUST read every skill above completely before editing code. It MUST also read the repository `AGENTS.md`. If a referenced skill path is unavailable, stop and report that blocker rather than silently substituting a workflow.

# CONTEXT

## Codebase

- **Language**: Rust 2024 workspace
- **Build system**: Cargo
- **Graph architecture**: `turso_graph_cypher` parses source, `turso_graph_frontend` binds and lowers through `turso_graph_ir`, and Turso core executes generated SQL. Canonical data remains in source tables.
- **Current source limit**: registration accepts exactly one node source and at most one relationship source because binding and mutation resolve only the first. New work MUST preserve that explicit limitation instead of implying multi-source support.
- **Command convention**: prefix development commands with `rtk`; use `apply_patch` for edits; never run an explicit `--release` build.

## Existing Seams and Invariants

1. `graph/frontend/src/binder.rs::bind_call` is a two-entry match for `db.labels` and `db.relationshipTypes`. It lowers procedures through private sentinel functions and `Unwind`; it rejects arguments and multiple `YIELD` columns.
2. `graph/cypher/src/ast.rs::CallClause` carries a qualified name, expressions, and plain yielded identifiers. The grammar does not yet represent yield aliases or `YIELD *`.
3. `graph/frontend/src/functions.rs` is a typed scalar-function registry. It is not a procedure registry and MUST NOT be stretched into one.
4. `graph/ir/src/plan.rs::PlanKind` has no procedure operator. A proper procedure surface requires a frontend-neutral read-only plan node rather than more sentinel functions.
5. `GraphCatalogSnapshot` resolves semantic graph identities. `RelationalCatalogSnapshot` maps those identities to source tables/columns and already exposes payload columns, label tables, relationship-type tables, and endpoint layouts.
6. `SchemaCatalog::payload_columns` enumerates logical property names while excluding identity and endpoint columns and translating `cyprop_` payload names. This is the authoritative seed for `db.propertyKeys()`.
7. Relationships are represented in lowered SQL by their source identity value. `RelationshipTableLayout` supplies the relationship identity, start, and end columns. This is sufficient to implement `startNode()` and `endNode()` without a traversal snapshot.
8. Turso core FTS exists behind `turso_core/fts`, uses `CREATE INDEX ... USING fts (...)`, and exposes `fts_match`, `fts_score`, and `fts_highlight`. Graph function typing already recognizes those names, but graph frontend has no matching feature/capability contract and no graph-level index administration.
9. FTS is unavailable on wasm in core. A graph FTS API MUST fail at compile-time feature selection or with an explicit capability error; it MUST NOT defer to a runtime `no such function` failure.
10. `SnapshotStore` already reports `Missing`, `Current`, and `Stale` plus counts, generation, build duration, and heap estimates. Shared and connection-local/session overlay snapshots have different visibility. Diagnostics MUST report the snapshot the calling `GraphConnection` would actually use.
11. `GraphDialect` exposes the persistent `turso_graphs` internal virtual table. `GraphConnection::install` also installs connection-local graph virtual tables for traversal. Persistent catalog facts and process-local snapshot state MUST remain distinct.

## Product Boundary

The portable/Turso-native capabilities in this spec have direct user value:

- discover graph schema with a stable procedure contract;
- create and use FTS indexes without duplicating a search engine;
- obtain relationship endpoints with portable Cypher functions;
- diagnose traversal snapshot state and memory cost.

The following Apache AGE or donor-specific internals are deliberately out of scope: `vertex_stats`, `graph_stats`, `delete_global_graphs`, `start_id`, `end_id`, `btic`, `is_valid_label_name`, and a compatibility scalar named `full_text_search`. Do not stub these names for corpus points. If the conformance harness encounters them, classify them as vendor-specific unsupported behavior. `startNode()`/`endNode()` are portable entity functions and are not aliases for AGE's raw-id helpers.

## Coordination with the semantic-schema overlay stream

The semantic-schema overlay (`.specs/graph-semantic-schema-overlay.agent-spec.md`, Milestones 1-2) and this spec edit the same files (`binder.rs`, `catalog.rs`, `schema_catalog.rs`, `session.rs`, `lib.rs`). Run the streams in sequence, never in parallel worktrees. The recommended combined ordering, with rationale, is `tessera/.specs/tessera-turso.design-spec.md` (tessera repository) section 11.2: overlay Milestones 1-2 first, then multi-source registration/binding (a deliberate, separately specified lift of the single-source limit — promoted with documented rationale in that section and in the semantic-schema spec's later-milestone criteria; the ban below on broadening it *incidentally* still stands), then this spec's phases in the order procedures → endpoints → diagnostics → FTS, each built semantic-aware from the start. This spec's "Current source limit" statement above describes the state its own work must preserve, not the permanent end state.

Semantic-aware means, concretely:

- `db.labels()`, `db.relationshipTypes()`, and `db.propertyKeys()` return semantic names (from the semantic snapshot) when the graph has a registered semantic schema, and physical logical names otherwise. Enumeration stays catalog-only either way.
- The FTS administration API validates logical property names through semantic ownership when a semantic schema exists.
- `startNode()`/`endNode()` may use registered endpoint constraints to narrow the returned node's static semantic type set.
- Snapshot diagnostics need no extra work: semantic registration bumps the graph generation this spec's diagnostics already report.

If this spec's stream lands first instead, the overlay stream owns retrofitting the three items above; record that handoff explicitly in whichever lands second.

Adjacent future capability, out of scope here: a graph vector-index administration API for embedding similarity search. Core already ships the vector scalar layer (`vector32`/`vector64`/`vector8`/`vector1bit`/`vector32_sparse`, `vector_distance_cos`/`_l2`/`_dot`/`_jaccard`), the graph frontend already types those functions for Cypher, and the `CREATE INDEX ... USING <method>` seam this spec uses for FTS is the same seam a future ANN method would use. When a production ANN index method exists in core, the graph FTS administration API defined here is the template to mirror (feature gate, structured typed input, transactional versioned metadata, reopen stability). See `tessera/.specs/tessera-turso.design-spec.md` (tessera repository) section 8.6.

## Relevant Files

| File | Purpose | Access |
|------|---------|--------|
| `graph/cypher/src/ast.rs` | `CALL` syntax contract | read, possibly read-write only if a required portable syntax gap is proven |
| `graph/cypher/src/cypher.pest` | `CALL` grammar | read, possibly read-write only with a focused parser test |
| `graph/frontend/src/binder.rs` | Procedure and entity-function binding | read-write |
| `graph/frontend/src/functions.rs` | Typed scalar registry and FTS signatures | read-write |
| `graph/frontend/src/lowering.rs` | Plan/expression lowering and relational layouts | read-write |
| `graph/frontend/src/schema_catalog.rs` | Production graph-to-table catalog | read-write |
| `graph/frontend/src/catalog.rs` | Persistent graph registration and internal tables | read-write for FTS metadata/API only |
| `graph/frontend/src/snapshot.rs` | Snapshot state, visibility, and metrics | read-write |
| `graph/frontend/src/session.rs` | `GraphConnection` public API | read-write |
| `graph/frontend/src/dialect.rs` | Persistent `turso_graphs` catalog model | read; read-write only if SQL diagnostics is approved |
| `graph/frontend/src/graph_expand.rs` | Connection-local virtual-table installation | read; read-write only if SQL diagnostics is approved |
| `graph/frontend/src/lib.rs` | Intentional public exports | read-write |
| `graph/ir/src/plan.rs` | Add a frontend-neutral procedure plan operator | read-write |
| `graph/ir/src/lib.rs` | Export new IR types | read-write |
| `core/index_method/fts.rs` | Existing FTS engine and index lifecycle | read; modify only when a graph test proves a core defect |
| `core/dialect/sqlite.rs` | Feature-gated FTS function dispatch | read |
| `docs/fts.md` | Existing FTS behavior and limitations | read |
| `docs/graph.md` | Graph consumer documentation | read-write |
| `graph/frontend/tests/fixture.rs` | Shared production-catalog test fixture | read-write |
| `graph/frontend/tests/type_system_fixtures.rs` | End-to-end typed functions/procedures | read-write or split when the file would mix concerns |
| `graph/testkit/src/performance.rs` | Recorded graph lifecycle workloads | read-write for the graph FTS benchmark case |
| `graph/test-results/REPORT.md` | Current conformance evidence | read; regenerate only in the final recorded-run slice |

# INPUTS

| Input | Location | Format | Required |
|-------|----------|--------|----------|
| Current conformance failures | `graph/test-results/REPORT.md` | Markdown | yes |
| Graph catalog implementation | `graph/frontend/src/catalog.rs`, `schema_catalog.rs` | Rust | yes |
| Binder/lowering contracts | `graph/frontend/src/binder.rs`, `lowering.rs` | Rust | yes |
| Procedure syntax | `graph/cypher/src/ast.rs`, `cypher.pest` | Rust/Pest | yes |
| Core FTS contract | `core/index_method/fts.rs`, `docs/fts.md` | Rust/Markdown | yes |
| Snapshot lifecycle | `graph/frontend/src/snapshot.rs`, `session.rs` | Rust | yes |

# OUTPUTS

| Output | Location | Format | Acceptance Criteria |
|--------|----------|--------|---------------------|
| Typed procedure registry | New focused module(s) under `graph/frontend/src/` plus IR additions | Rust | Case-insensitive lookup, typed arguments/yields, stable errors, and no binder-local name match |
| `db.propertyKeys()` | Procedure registry, catalog/lowering, integration tests | Rust | Returns distinct logical payload keys across registered node and relationship sources; excludes identity/endpoints; deterministic under explicit `ORDER BY` |
| Graph FTS administration | `graph/frontend/src/catalog.rs` or a focused sibling module and public exports | Rust | Validates logical properties, creates/drops core FTS indexes transactionally, records portable metadata, survives reopen, and exposes capability cleanly |
| Graph FTS query support | Existing scalar registry/lowering plus integration tests | Rust | Cypher `MATCH ... WHERE fts_match(...)` and `fts_score(...)` use the existing core FTS index and bind query values as parameters |
| Endpoint functions | Binder/lowering and tests | Rust | `startNode(r)`/`endNode(r)` return node-typed endpoint identities for relationship bindings, preserve nulls, and reject wrong types |
| Snapshot diagnostics API | `snapshot.rs`, `session.rs`, exports, tests | Rust | Reports calling-session status/metadata without exposing graph rows or forcing a rebuild |
| Optional SQL diagnostics table | `dialect.rs`/`graph_expand.rs` only after the decision gate | Rust | Read-only, connection-local for process state, and semantically identical to the diagnostics API |
| Consumer documentation | `docs/graph.md` and only directly relevant reference docs | Markdown | Names capability gates, lifecycle, transaction behavior, and unsupported vendor internals |
| Verification evidence | Commit bodies and final handoff | Text | Exact commands/results, conformance delta, and benchmark comparison are recorded |

# REQUIRED DESIGN

## 1. Typed Procedure Registry

Create a dedicated registry module. Its public-to-crate contract MUST describe, at minimum:

- canonical qualified procedure name;
- ordered argument descriptors with `ValueType` and required/optional status;
- ordered yielded columns with stable names, `ValueType`, and nullability;
- read-only versus mutating classification;
- a closed implementation identity such as an enum, not an arbitrary SQL string or function pointer supplied by callers.

Add a frontend-neutral `ProcedureCall` plan kind to `turso_graph_ir`. It MUST carry its input plan, resolved procedure identity, bound arguments, and selected yielded bindings. `turso_graph_ir` MUST NOT depend on frontend/catalog/core types. Every exhaustive `PlanKind` match in lowering and binder tests must be updated deliberately.

Binder rules:

- resolve names case-insensitively and retain a canonical name in diagnostics;
- bind and type-check arguments using the descriptor;
- reject unknown procedures as unsupported and invalid calls as semantic errors with source spans;
- allow a subset/reordering of declared yield names where the grammar can represent it;
- reject duplicate/unknown yields;
- use the descriptor's default yield set when `YIELD` is omitted, preserving the existing bare-`CALL` result behavior;
- do not add aliases, `YIELD *`, or `YIELD ... WHERE` unless a failing portable scenario requires them and parser tests are added first;
- reject mutating procedures in read compilation until a write-procedure transaction/authorization contract exists.

Lowering rules:

- lower only closed implementation enum variants;
- compose a procedure call with an existing input as a relational pipeline, not as a global side effect;
- quote identifiers through existing helpers and bind user values as SQL parameters;
- do not retain `__cypher_all_labels` or `__cypher_all_relationship_types` as the procedure architecture after migration.

Registry seed set:

- `db.labels() -> label: Text non-null`;
- `db.relationshipTypes() -> relationshipType: Text non-null`;
- `db.propertyKeys() -> propertyKey: Text non-null`.

`db.propertyKeys()` MUST derive keys from `RelationalCatalogSnapshot::payload_columns` (or a narrowly factored semantic enumeration method backed by the same logic). It MUST include logical keys from node and relationship payloads, deduplicate keys shared by both, and exclude structural identity/start/end columns. It MUST not scan data rows; a declared nullable column with no values is still a property key.

## 2. Portable `startNode()` and `endNode()`

Treat these as typed Cypher entity functions, not SQL scalar aliases:

- exactly one relationship argument;
- return type `ValueType::Node` with the argument's nullability;
- a statically non-relationship argument is a bind error;
- a null argument returns null;
- relationship bindings carried through `WITH` remain supported when their source layout is still known;
- relationship lists and paths are not accepted as scalar relationships.

Follow the existing `properties(entity)` pattern: intercept the still-visible relationship binding before generic argument lowering, resolve its `RelationshipTableLayout`, and lower a correlated lookup from relationship identity to the `start_column` or `end_column`. Never parse or expose donor-specific entity encodings. Add a clear error when a relationship binding has no physical layout instead of producing malformed SQL.

## 3. Graph Full-Text Indexing

### Mandatory API boundary

Add a graph frontend Cargo feature named `fts` that forwards to `turso_core/fts`. FTS-only public types/functions and tests must be feature-gated consistently. When the feature is absent, graph FTS calls MUST bind to an explicit unsupported-capability error; do not let them reach core as `no such function`. Document the non-wasm constraint.

Expose a Rust administration API rather than a mutating Cypher procedure in this milestone. There is no graph authorization model, so `CALL ...createIndex` would create an unguarded DDL channel. The API MUST accept structured values, not raw SQL:

- graph name or `GraphId` resolved through the registered catalog;
- stable logical index name;
- entity kind (`Node` for the first delivery; relationship support may follow only with equivalent tests);
- one or more logical text property names;
- supported core FTS options represented by typed fields (tokenizer and weights); reject unknown options.

The API MUST:

1. validate the graph, source, and logical property names against `SchemaCatalog`;
2. reject structural columns and statically non-text properties unless an explicit, tested coercion policy is approved;
3. derive an internal physical index name under the reserved Turso prefix using the existing stable-hash/quoting conventions;
4. execute `CREATE INDEX ... USING fts` through internal statements inside a savepoint or immediate transaction consistent with graph registration;
5. persist graph-to-physical FTS metadata in versioned internal catalog tables so reopen, duplicate detection, listing, and drop do not depend on reconstructing names;
6. roll back metadata and physical DDL together on any error;
7. provide idempotent listing and an explicit drop API; never silently replace an existing differently configured index;
8. rely on core's existing update/delete/index durability lifecycle rather than duplicating an FTS store in graph code.

For queries, keep the portable Turso surface small:

```cypher
MATCH (n:Article)
WHERE fts_match(n.title, n.body, $query)
RETURN n, fts_score(n.title, n.body, $query) AS score
ORDER BY score DESC
LIMIT 20
```

Prove with `EXPLAIN QUERY PLAN` or equivalent planner evidence that the core FTS index is selected. User query text MUST remain a bound value and MUST never be interpolated into generated SQL. Define maximum accepted index-name/property counts in the public API or reuse an existing bounded core representation; reject unbounded configuration input before constructing DDL.

### FTS procedure decision gate

After the scalar/index API is green, write a short decision note in the implementation commit or `docs/graph.md` answering whether a read-only procedure such as `db.index.fulltext.queryNodes(index, query)` adds value beyond `MATCH` plus `fts_match`. Add it only if it provides a measurable capability (for example top-k index-driven node production that cannot be expressed efficiently by current lowering). If added, it MUST use the typed registry, yield a node plus score, consult persisted index metadata, and stay read-only. Do not implement donor names such as `spa.fulltext.queryNodes` or `full_text_search`.

## 4. Snapshot Diagnostics

First expose a typed Rust API on `GraphConnection`, for example `diagnostics() -> Result<GraphDiagnostics, Error>`. Final naming must match existing API conventions. It MUST report:

- graph id/name;
- persistence mode;
- status: missing/current/stale;
- catalog version and source generation where available;
- current generation when stale;
- node/relationship counts;
- build elapsed duration;
- estimated retained and peak-build bytes.

The API MUST classify the connection-local/session overlay snapshot used by the calling graph session before falling back to shared committed state. Calling it MUST be read-only: no refresh, no catalog writes, and no snapshot publication. It MUST not expose source row values, relationship coordinates, FTS query text, or other user data.

### SQL diagnostics decision gate

Only after the Rust API and lifecycle tests pass, decide whether SQL observability is required. If yes, add a read-only `turso_graph_snapshots` internal virtual table installed with the connection-local graph catalog, not a persistent table. It must delegate to the same API-level status projection and clearly expose that rows are process/session state. Do not add process-local columns to persistent `turso_graphs`; doing so would make results depend on which connection and process reads a nominally persistent catalog.

# IMPLEMENTATION PIPELINE

Each slice below is scoped to at most 35 minutes of agent work. Finish its focused checks and summarize the checkpoint before starting the next slice. If a slice cannot fit, split it further before editing.

## Phase 0 - Baseline and Contracts

| Slice | Work | Verification |
|-------|------|--------------|
| 0.1 | Run focused frontend/IR tests and record current FTS feature behavior and representative procedure/endpoint failures. | Existing tests green; exact failures captured in notes, not committed artifacts. |
| 0.2 | Write unit tests for registry lookup/signatures and IR construction before implementation. | New tests fail for the intended missing types only. |
| 0.3 | Add endpoint and diagnostics integration test skeletons using the shared production fixture. | Tests compile up to missing APIs and encode null/wrong-type/session cases. |

## Phase 1 - Procedure Registry and `db.propertyKeys()`

| Slice | Work | Verification |
|-------|------|--------------|
| 1.1 | Add closed procedure descriptors/identities and case-insensitive registry lookup in one focused module. | Unit tests cover all three seed procedures, unknown names, arity, and yields. |
| 1.2 | Add/export `ProcedureCall` IR types and update exhaustive IR consumers. | `rtk cargo test -p turso_graph_ir`. |
| 1.3 | Replace binder-local procedure matching with descriptor-driven binding, leaving lowering failure explicit. | Binder tests cover unknown/duplicate/unknown-yield and wrong-arity calls. |
| 1.4 | Lower label and relationship-type procedure variants through `ProcedureCall`; remove their sentinel path. | Existing `db.labels` and `db.relationshipTypes` tests remain green. |
| 1.5 | Add property-key catalog enumeration and lower `db.propertyKeys()`. | Fixture test covers node-only, relationship-only, duplicate, structural, empty-value, and `cyprop_` cases. |
| 1.6 | Run frontend/testkit focused gates and commit the coherent milestone. | Format, IR/frontend tests, and diff check pass. |

## Phase 2 - Endpoint Functions

| Slice | Work | Verification |
|-------|------|--------------|
| 2.1 | Bind typed `startNode`/`endNode` calls and wrong-type/null semantics. | Focused binder tests prove type/nullability. |
| 2.2 | Lower relationship endpoint lookups through `RelationshipTableLayout`. | Lowering tests assert quoted physical identifiers and missing-layout error. |
| 2.3 | Add end-to-end fixed and carried-through-`WITH` relationship tests. | Values match source `src`/`dst`; null remains null. |
| 2.4 | Run focused conformance identities and commit. | AGE portable endpoint scenarios improve; vendor `start_id`/`end_id` remain unsupported. |

## Phase 3 - Graph FTS

| Slice | Work | Verification |
|-------|------|--------------|
| 3.1 | Add the graph `fts` Cargo feature/capability errors and feature-gate function signatures consistently. | Builds/tests both with and without `fts`; no runtime `no such function` path. |
| 3.2 | Add versioned graph FTS catalog metadata types/table creation and load/list tests. | Fresh install and reopen return identical metadata. |
| 3.3 | Add validated node-index create API and transaction/savepoint rollback tests. | Invalid properties/options leave neither metadata nor physical indexes. |
| 3.4 | Add explicit list/drop APIs and duplicate/conflicting-definition behavior. | Drop is transactional; same-name conflict is deterministic. |
| 3.5 | Add Cypher `fts_match`/`fts_score` integration tests with parameter binding and planner evidence. | Correct matches/ranking and FTS plan selection. |
| 3.6 | Add update/delete/reopen and missing-index tests. | Index remains consistent across row lifecycle and reopen. |
| 3.7 | Add one representative graph FTS performance workload and compare against a non-indexed control or saved baseline. | Benchmark records corpus size, selectivity, warm/cold state, and latency. |
| 3.8 | Resolve the read-only FTS procedure decision gate, document, run gates, and commit. | Decision and evidence are explicit; no vendor compatibility names. |

## Phase 4 - Diagnostics

| Slice | Work | Verification |
|-------|------|--------------|
| 4.1 | Factor connection-aware status lookup in snapshot stores without changing refresh behavior. | Unit tests distinguish session overlay from shared state. |
| 4.2 | Add/export typed `GraphConnection` diagnostics. | Missing/current/stale fields match snapshot metadata; call causes no generation change. |
| 4.3 | Cover refresh, source mutation, discard, reopen/process-loss simulation, and transaction-visible overlay lifecycle. | All lifecycle assertions pass. |
| 4.4 | Resolve SQL diagnostics decision gate; if approved, add the read-only connection-local virtual table and parity tests. | API/table rows agree; attach and graph-dialect modes are explicit. |
| 4.5 | Document the API, run gates, and commit. | Public docs match exported names and feature flags. |

## Phase 5 - System Validation

| Slice | Work | Verification |
|-------|------|--------------|
| 5.1 | Run all graph crate tests with default features. | All pass. |
| 5.2 | Run all graph frontend/testkit tests with FTS enabled and the core FTS integration subset. | All pass. |
| 5.3 | Run smoke/deep conformance without recording; classify remaining vendor internals as unsupported. | No portable regression and no vendor stub. |
| 5.4 | Run performance smoke and the new FTS workload; compare to Phase 0/current benchmark records. | No unexplained regression; results summarized. |
| 5.5 | Only with explicit baseline-recording intent, run the recorded corpus/benchmark commands and commit generated history/report separately. | Append-only history verifies and generated changes are isolated. |

# CONSTRAINTS

## MUST

- Preserve the parser -> binder/IR -> lowering -> core boundary; graph frontend MUST NOT emit VDBE instructions.
- Use typed errors for public library failures and source-spanned semantic errors during binding.
- Keep procedure identities and implementations closed and auditable.
- Treat FTS index DDL and its metadata as one transactional operation.
- Use existing identifier validation/quoting and stable internal-name conventions.
- Keep all FTS user text parameterized.
- Test default and FTS-enabled feature configurations.
- Add tests that fail without each implementation and explain the behavior's invariant.
- Run `rtk cargo fmt --all -- --check` after Rust changes and relevant Clippy/tests before each milestone commit.
- Use conventional commits with descriptive bodies and a `Tests:` line.
- Keep generated conformance/benchmark history out of feature commits; use a separate final record commit only when intentionally recording a baseline.

## MUST NOT

- Stub AGE/SparrowDB administrative or catalog internals for failure-count gains.
- Add `start_id`, `end_id`, or `full_text_search` aliases.
- Scan graph data rows to implement `db.propertyKeys()`.
- Add a generic procedure hook that accepts raw SQL, arbitrary callbacks, or caller-defined yield schemas.
- Add mutating Cypher procedures before authorization and write-transaction semantics exist.
- Build a second FTS index implementation or copy Tantivy state into graph-owned storage.
- Claim wasm FTS support.
- Expose process-local snapshot status as persistent catalog state.
- Trigger a snapshot rebuild from diagnostics.
- Broaden graph registration to multiple sources as an incidental part of this work.
- Modify core FTS code unless a minimal graph integration test proves a core defect.
- Touch unrelated dirty or untracked files.

## SHOULD

- Prefer one focused new module each for procedures and FTS catalog administration over growing `binder.rs` or `catalog.rs` without bound.
- Keep public surface minimal and use `pub(crate)` for registry/lowering details.
- Reuse `GraphConnection` fixtures backed by `SchemaCatalog` rather than test-only mock catalogs for integration behavior.
- Use planner evidence and measured workloads before optimizing FTS lowering.
- Preserve deterministic display/order in catalog procedures, while allowing callers to specify final result ordering.
- Split implementation commits by the four coherent capability milestones so each can be reverted independently.

# SECURITY, PERFORMANCE, AND LIFECYCLE CHECKS

## Security

- Attempt index/procedure names containing quotes, NUL, reserved prefixes, SQL comments, and path-like input; verify validation rejects them or quoting makes them inert.
- Verify FTS query strings containing quotes/operators remain bound data.
- Verify unknown procedure and yield names cannot reach generated SQL.
- Verify diagnostics disclose metrics and identifiers only, never row/property values.
- Document that Rust FTS administration uses the caller's database authority; a future network binding must add its own authorization before exposing it remotely.

## Performance

- `db.propertyKeys()` is schema/catalog work proportional to declared columns, not row count.
- `startNode()`/`endNode()` lookups use the relationship identity uniqueness guarantee and do not rebuild traversal snapshots.
- FTS queries prove index selection; record cold and warm results, row count, selectivity, and limit.
- Diagnostics acquire snapshot locks briefly, clone only metadata, and do not retain an extra snapshot graph after return.

## Lifecycle

- FTS create/drop rollback under both autocommit and an already-open write transaction; reject or correctly scope deferred read transactions according to existing catalog policy.
- FTS metadata and physical index state agree after reopen and failed DDL.
- Source inserts, updates, and deletes update the FTS index through core's existing lifecycle.
- Diagnostics distinguish missing, current, and stale after refresh, mutation, discard, and reopen/process loss.
- Session-local uncommitted visibility is never published as shared committed diagnostics.

# VERIFY

## Automated Checks

Run the narrowest command after each slice, then all commands below before final handoff. If the workspace has a pre-existing unrelated Clippy failure, capture its exact file/line and still run Clippy for the changed graph packages where possible; do not misreport the gate as green.

```bash
rtk cargo fmt --all -- --check
rtk cargo test -p turso_graph_ir
rtk cargo test -p turso_graph_cypher
rtk cargo test -p turso_graph_frontend
rtk cargo test -p turso_graph_frontend --features fts
rtk cargo test -p turso_graph_testkit
rtk cargo test -p turso_core --features fts index_method::
rtk cargo clippy -p turso_graph_ir -p turso_graph_cypher -p turso_graph_frontend -p turso_graph_testkit --all-targets --all-features -- --deny=warnings
rtk cargo run -q -p turso_graph_testkit -- run smoke --no-record
rtk cargo run -q -p turso_graph_testkit -- run deep --no-record
rtk cargo run -q -p turso_graph_testkit -- performance smoke --no-record
rtk git diff --check
```

Run an intentional recorded baseline only when the owner asks for updated records:

```bash
rtk cargo run -q -p turso_graph_testkit -- corpus
rtk cargo run -q -p turso_graph_testkit -- verify-history
```

Run the existing core FTS benchmark plus the new focused graph workload without adding an explicit `--release` flag:

```bash
rtk cargo bench -p turso_core --bench fts_benchmark --features fts
```

## Acceptance Scenarios

1. `CALL db.propertyKeys() YIELD propertyKey RETURN propertyKey ORDER BY propertyKey` lists every declared logical payload key once and no structural columns.
2. Existing `CALL db.labels()` and `CALL db.relationshipTypes()` behavior is preserved through the new registry and IR operator.
3. Unknown procedures and yield columns fail during binding with source spans and cannot alter generated SQL.
4. `MATCH ()-[r]->() RETURN startNode(r), endNode(r)` returns the source table's start/end identities as node-typed values.
5. Endpoint calls over nullable relationship values return null; calls over nodes/scalars fail during binding.
6. With graph FTS disabled, an FTS Cypher call reports unsupported capability before SQL execution.
7. With graph FTS enabled, a structured API creates an index on declared text properties, a parameterized Cypher query returns ranked matches, and planner evidence selects FTS.
8. Failed/duplicate FTS creation leaves metadata and physical schema consistent; drop and reopen preserve the same truth.
9. Diagnostics report the exact snapshot visible to the calling session and do not rebuild or publish it.
10. Vendor-only admin functions remain explicitly unsupported and are not counted as portable regressions.

## Success Criteria

- [ ] All four mandatory capability milestones are implemented with focused tests.
- [ ] The procedure registry is descriptor-driven and represented explicitly in graph IR.
- [ ] `db.propertyKeys()` performs catalog enumeration only.
- [ ] `startNode()`/`endNode()` use physical relationship layouts and preserve graph value typing.
- [ ] Graph FTS reuses core FTS, has a clear feature gate, and has transactional metadata/API lifecycle.
- [ ] Snapshot diagnostics are calling-session correct, read-only, and data-minimizing.
- [ ] Default and FTS-enabled graph tests pass.
- [ ] Smoke/deep conformance shows no portable regression.
- [ ] Benchmark results include enough workload metadata for a meaningful before/after comparison.
- [ ] Feature commits and any generated baseline commit are separate and conventional.

## Failure Conditions

- Any implementation adds vendor compatibility stubs instead of Turso-native semantics.
- A procedure name, FTS query, property name, or index name is interpolated without existing validation and quoting/binding.
- `db.propertyKeys()` depends on current row contents.
- FTS metadata can commit without the physical index, or vice versa.
- FTS-disabled builds reach runtime with `no such function` for a graph-recognized FTS call.
- Diagnostics return shared state when a calling-session overlay exists, expose graph data, or trigger a refresh.
- The implementation silently broadens multi-source behavior or changes canonical storage ownership.
- Required tests, Clippy, conformance, or benchmark checks are skipped without an explicit blocker report.

# CHECKPOINT AND COMMIT POLICY

After each phase, report what changed, exact checks run, and remaining work. Commit only coherent green milestones, suggested as:

1. `feat(graph): add typed procedure registry and property keys`
2. `feat(graph): add portable relationship endpoint functions`
3. `feat(graph): expose native full-text indexing`
4. `feat(graph): expose traversal snapshot diagnostics`
5. `test(graph): record native capability conformance baseline` (only for an intentional recorded run)

Do not merge or push from an isolated implementation worktree. Return commit hashes and verification evidence to the supervising agent for review and integration.
