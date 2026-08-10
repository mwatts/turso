# Graph frontend performance backlog

Raised by a consuming application (tessera) reporting a ~1.0 s projection
drain when bootstrapping a 118-entity ontology. Their own profile pointed at
`SemanticConstraintSnapshot::validate_state` and, underneath it,
`catalog::query_rows → prepare_with_origin → compile_cmd → translate`.

## How the numbers here were measured

A temporary `tracing` layer watched the `Preparing: {sql}` debug event that
`Connection::prepare_with_origin` emits, and counted the SQL statements core
compiled per Cypher statement. Setup: in-memory database, one node source, one
node type with two properties, one required-property constraint and one
range constraint.

```
CREATE (:Person {displayName: '…', born: …})

                        before   after fix 1
  no constraints          16         3
  with constraints        21         5
  20 CREATEs             420       100
```

Only two of those statements ever did the work (`INSERT INTO people`,
`INSERT INTO __turso_graph_node_labels_1`). Nothing amortised: the 21st
mutation cost exactly what the 1st did.

Fix 2 makes the rest amortise. Counting the same way on a `CREATE` with one
required and one unique property, a steady-state mutation compiles 2
statements where it used to compile 5; the 3 it drops are the ones whose text
repeats.

`graph/frontend/tests/catalog_refresh.rs` keeps the fixed part honest.

## Fixed

### 1. Row writes no longer reload the catalog — done

`install_generation_triggers` (`src/catalog.rs`) puts AFTER
INSERT/UPDATE/DELETE triggers on the **user data tables** that bump
`__turso_internal_graph_generations.generation`. That is the same counter
`GraphConnection::refresh_catalog_if_stale` used to decide whether the
*schema* had changed, so every row written invalidated the catalog. The next
statement then recompiled the whole graph catalog and semantic schema — 17
statements — and threw away the Cypher compile cache via
`clear_last_compile()`.

The generations table now carries a second counter, `schema_generation`,
advanced only by catalog changes (`bump_semantic_generation`). Sessions track
that one and probe it with a single primary-key lookup; the data generation
keeps its old meaning and still drives traversal-snapshot rebuilds.

Databases written before the column fall back to the data generation, i.e.
the old always-reload behaviour, until a registration call migrates them.

### 2. The SQL a mutation repeats is now kept prepared — done

`catalog::query_rows` (`src/catalog.rs`) was
`connection.prepare(sql)?.run_collect_rows()`, and core's
`prepare_with_origin` has no cache, so every call was parse + plan + codegen.

The session now owns a `StatementCache` (`src/statement_cache.rs`) keyed on
the exact SQL text, and the two paths that repeat themselves — the catalog
freshness probe and `validate_state`'s constraint checks — go through it.
Parameterising on `graph_id` turned out to be unnecessary: those queries are
built from the resolved constraint, never from the row being written, so the
text already repeats byte-for-byte.

A held statement cannot answer a stale question. `Statement::step`
re-prepares when the connection's prepare context has moved, and a schema
change surfaces from the VDBE as `SchemaUpdated`, which also re-prepares and
retries. The cache lives on the session rather than on `Connection` because a
`Statement` owns an `Arc<Connection>` — a connection holding its own
statements never drops.

What is still uncached is the mutation's own write. Its text carries the
values being written, so it never repeats and a text-keyed cache cannot help
it; reusing it needs the plan bound to parameters instead, which is item 4.

### 6. Filters now reach the tables they constrain — done

Not from the tessera report; found by reading what a `main` merge brought in.
Core gained passes that turn a LEFT JOIN into an INNER JOIN when a predicate
rejects the null-extended rows, and that seek an index for `IS NULL`. Both
read the query's WHERE terms, and both need the term to name a table the
query joins.

`lower_plan`'s `Filter` arm wrapped its input in a derived table and filtered
outside it, so every predicate named derived-table columns. Core has no pass
that flattens a FROM-clause subquery, so from its side those terms constrained
an opaque subquery and neither pass could fire. The arm now appends the
predicate to the WHERE of the select that joins the constrained tables, and
folds a run of stacked filters into that one WHERE so a chain does not strand
the outer predicates a level up. A predicate reading a binding that select
does not join falls back to the old wrapper.

The lowering emits `(a) = (5)` — parenthesizing each operand is how it keeps
precedence right without tracking it. Core matched a constraint only when the
operand *was* a column, so a parenthesized column hid the index and every such
comparison scanned. `as_binary_components`
(`core/translate/expr/utils.rs`) now drops those parentheses, which is what
SQLite's grammar does while parsing. Row values are untouched: `(x, y)` is one
value, not a wrapper. `sqlite/conformance/sqlite-sqltests/parenthesized-operand-index-seek.sqltest`
pins it.

Both halves are needed. With only the pushdown the predicate still reads
`(n."age") > (30)` and core scans; with only the parentheses fix the predicate
still sits outside the join and names a derived table.

## Fixed (continued)

### 3a. Required/value checks scope to written identities; value predicates use SQL + `LIMIT 1` — done (R15)

`WrittenIdentities` is filled on every mutation path that can name the rows it
touched (closed-`CREATE` fast path, `execute_operation` create/merge/set/
replace/remove/set-roles). When `ValidationScope` is still source-precise,
`validate_state` stages those ids into a connection-local temp table
`__turso_graph_mutation_written` and required/value probes join it:

```sql
… AND entity."id" IN (
  SELECT written.identity FROM "__turso_graph_mutation_written" AS written
  WHERE written.source_id = <source>
) LIMIT 1
```

Ids live in the temp table, not in the prepared text, so the statement cache
still reuses the probe SQL across creates (same shape as fix 2).

Range and allowed value predicates lower to a pure-SQL violation expression
with `LIMIT 1`. Regex still evaluates in Rust, but only over the
identity-filtered rows when known, and stops at the first violation.

**Measurement method** (same `Preparing:` tracing as the top of this file;
in-memory DB; one node type; required + range constraints; 25 warm CREATEs
then one steady CREATE):

| Probe | Before (this item) | After |
| --- | --- | --- |
| Required SQL | full membership, `LIMIT 1` | membership ∩ written ids, `LIMIT 1` |
| Value SQL | `SELECT id, col … IS NOT NULL` (no limit; all rows into Rust) | violation predicate in SQL + written-id filter + `LIMIT 1` |
| Steady CREATE recompiles of required/value text | 0 after cache warm (text was stable) | still 0 (temp-table filter keeps text stable) |

Pinned by `graph/frontend/tests/constraint_identity_cost.rs` (SQL shape) and
`constraint_validation_scope` (range still fails after bulk load and on SET).
DETACH / binding-sourced writes still force `ValidationScope::All` and a full
membership pass so integrity is never under-scoped.

### 3b. StatementCache drop-one eviction — done (R17)

At capacity 64 the cache used to `clear()` the entire map. It now drops the
least-recently-used entry only (`statement_cache.rs`). Unit tests pin that a
hot probe survives overflow.

## Open, in rough order of leverage

### 3. Constraint validation residual scale (unique / key / cardinality)

Required and value property checks no longer grow with row count when the
mutation reports identities (see 3a). Still open:

- **Unique, key, and cardinality** still scan the type (or join) membership.
  Unique already has `HAVING COUNT(*) > 1 LIMIT 1`, but cost is still Θ(N) per
  CREATE of a constrained type. Restricting unique to the values the written
  rows carry, and cardinality to the nodes the statement wrote or repointed,
  would finish the O(N²) → O(N) story for those classes.
- skip constraint classes a statement cannot affect — a `CREATE` touching no
  relationship type `X` cannot change `X`'s cardinality on untouched nodes.
- defer validation to commit inside an explicit transaction. On its own this
  collapses a 118-statement bootstrap to a single validation pass.

### 4. Mutations parse Cypher twice and never use the compile cache

`GraphConnection::execute` (`src/session.rs`) calls
`turso_graph_cypher::parse` to decide whether the statement needs a traversal
snapshot; `execute_cypher_mutation` (`src/mutation.rs`) then parses the same
source again. Reads go through `prepare_frontend` and get a compile cache;
writes have none. Caching bound mutation plans keyed on
(source, schema generation) would pay for itself in exactly the
bootstrap-loop shape that prompted this work.

### 5. Replace the generation triggers with a core primitive — done

`Connection::table_change_token` landed; AFTER-DML generation triggers were
deleted. Snapshots compare `RegisteredGraph::derived_generation` (schema
generation + change tokens for mapped tables). See
`graph/docs/table-change-detection-design.md` (“What landed”) and
`graph/docs/core-changes.md` §2.
