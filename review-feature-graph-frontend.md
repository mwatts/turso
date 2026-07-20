# Code Review: feature/graph-frontend vs main

Multi-agent review, high effort. 7 candidate findings, all 7 independently verified (CONFIRMED), 0 refuted.
2 correctness bugs, 5 cleanup/efficiency items. Ranked most-severe first.

---

## Correctness

### 1. Postgres reprepare path drops implicit CREATE SEQUENCE prereqs
**File:** `postgres/frontend/session.rs:39` — CONFIRMED

`PostgresCompiler::compile` (the `FrontendCompiler` used for `Statement::reprepare` and
`compile_cmd`'s cross-process schema retry) calls `PostgreSQLTranslator::translate()` instead of
`translate_with_prereqs()`, silently discarding the implicit `CREATE SEQUENCE` prerequisite
statements generated for `SERIAL`/`BIGSERIAL` columns.

**Failure scenario:** `PgConnection::prepare("CREATE TABLE t (id SERIAL PRIMARY KEY)")` — first
prepare (free function `prepare_statement`, session.rs:278-306) uses `translate_with_prereqs`,
executes `CREATE SEQUENCE IF NOT EXISTS t_id_seq`, then hands the CREATE TABLE to
`conn.prepare_frontend`. If that statement is later reprepared (schema-version mismatch via
`Statement::reprepare`, or `Connection::compile_cmd`'s cross-process schema-lookup retry), the
retained `PreparedSource::Frontend` recompiles through `PostgresCompiler::compile`, which only
calls `translate()` and drops `TranslateResult::prereqs`. The `CREATE SEQUENCE` never re-runs; if
the backing sequence is missing at reprepare time, CREATE TABLE proceeds anyway and a later
`INSERT` relying on `DEFAULT nextval('t_id_seq')` fails at runtime instead of populating the
identity column per PostgreSQL semantics.

### 2. Cypher parse_string ignores standard backslash escapes
**File:** `graph/cypher/src/parser.rs:632` — CONFIRMED

`cypher.pest:45` accepts any `"\\" ~ ANY` sequence (so `\n`, `\t`, `\\`, `\"`, `\uXXXX` all parse),
but `parse_string` (parser.rs:632-638) only does `body.replace("''", "'").replace("\\'", "'")`.

**Failure scenario:** `'line1\nline2'` is accepted by the grammar but yields the literal
9-character string `line1\nline2` (backslash + n, not a newline); `'a\\b'` stays as two backslash
characters instead of collapsing to one. Wrong `Text` literal values for any query using standard
Cypher string escapes.

---

## Cleanup / Efficiency (by hot-path impact)

### 3. Postgres prepare_statement parses + translates twice per prepare
**File:** `postgres/frontend/session.rs:306` — CONFIRMED

`prepare_statement` calls `turso_pg_parser::parse` + `translate_with_prereqs` to extract/run DDL
prereqs, then `conn.prepare_frontend(...)` re-invokes `PostgresCompiler::compile`, which parses and
translates the same SQL a second time. Every Postgres-frontend PREPARE pays full parse+translate
cost twice on the query-compile hot path.

### 4. neighbor_cursor clones relationship_types Vec per node expanded
**File:** `graph/runtime/src/csr.rs:202` — CONFIRMED

`neighbor_cursor` does `relationship_types.to_vec()` and is invoked once per node expanded during
traversal (traversal.rs:150, traversal.rs:329, shortest.rs:65,170). A traversal visiting N nodes
with a non-trivial filter performs N redundant small-Vec heap allocations of the same list;
`Arc<[RelationshipTypeId]>` (or similar) would share one allocation.

### 5. traverse() duplicates TraversalCursor's BFS/DFS expansion logic
**File:** `graph/runtime/src/traversal.rs:291` — CONFIRMED

Eager `traverse()` (lines 291-362) hand-duplicates the frontier-push/budget/uniqueness/child-
construction logic already in `TraversalCursor::step()` (lines 116-211) — two independently
maintained copies, cross-checked only by a single equivalence test. A future fix applied to one
but not the other silently diverges eager vs resumable APIs. Prefer implementing `traverse()` by
driving the cursor.

### 6. Duplicate PRAGMA table_info per registered graph source
**File:** `graph/frontend/src/catalog.rs:245` — CONFIRMED

`register_graph_in_transaction` calls `require_columns` at line 245 (result discarded), then
`require_unique_identity` at line 246 internally calls `require_columns` again (line 534) for the
same table — running `PRAGMA table_info(...)` (full prepare+execute+collect round trip) twice per
node source and twice per relationship source on every `register_graph`. Pass the fetched column
list into `require_unique_identity` instead.

### 7. vector_return repeats the same closure five times
**File:** `graph/frontend/src/functions.rs:90` — CONFIRMED

The five `VectorKind` arms (Float32Dense, Float64Dense, Float32Sparse, Float1Bit, Float8) in
`vector_return` (lines 90-128) duplicate an identical closure body differing only by two constants.
Future dims-derivation fixes require editing five closures in lockstep; a copy-paste slip when
adding a sixth kind would silently report the wrong element type. Collapse to one helper
parameterized by the `(ir::VectorKind, CoreVectorType)` pair.

---

**Stats:** 4 finder agents, 7 candidates, 7 verifier agents, 7 verified / 0 refuted.
