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

## Open, in rough order of leverage

### 2. No prepared-statement cache anywhere

`catalog::query_rows` (`src/catalog.rs`) is
`connection.prepare(sql)?.run_collect_rows()`, and core's
`prepare_with_origin` has no cache: every call is parse + plan + codegen. The
catalog SQL is a small fixed set of strings with `graph_id`/name interpolated
into them.

Two options, not mutually exclusive:

- parameterise the catalog queries (`WHERE graph_id = ?`) and hold the
  prepared statements on the session;
- add an LRU keyed on SQL text inside the frontend.

Neither needs a core change. After fix 1 the remaining per-mutation catalog
traffic is one statement, so this now matters most for the constraint queries
(item 3) and for genuine schema reloads.

### 3. Constraint validation re-checks the whole graph after every statement

Partly fixed. `validate_state` now takes a `ValidationScope` and skips
constraints whose source table the mutation does not write, so the cost no
longer grows with the number of **installed types**. What remains is the cost
growing with the number of **rows**.

`SemanticConstraintSnapshot::validate_state` (`src/semantic_constraints.rs`)
still runs, per mutation, one full scan of the source table for every
in-scope property constraint, key and cardinality constraint. Two problems
left:

- **It is quadratic in rows.** Bulk-loading N rows of one type costs
  O(constraints × N²). At 118 rows this is invisible; at 50k it is a wall.
- **The value-predicate path has no `LIMIT`** (`src/semantic_constraints.rs`,
  `ResolvedPropertyPredicate::Value`). It pulls every non-NULL value of that
  property into Rust and checks them in a loop — on every mutation.

Fixes, increasing effort:

- restrict validation to the rows the mutation wrote. `execute_bound` already
  knows the affected identities (the `RETURNING id` values); add
  `entity.<identity> IN (…)`. Uniqueness, keys and cardinality still need a
  scan but can be restricted to the groups the new rows land in — for
  uniqueness, only the values the written rows carry; for cardinality, only
  the nodes the statement wrote or repointed. This is the change that turns
  O(N²) into O(N). Note that the identities have to be threaded through every
  operation branch **and** the closed-`CREATE` fast path: a branch that
  reports no writes silently stops validating.
- push the value predicate into SQL and add `LIMIT 1`, so a violation stops
  the scan instead of materialising the column.
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

### 5. Replace the generation triggers with a core primitive

The AFTER-DML triggers cost more than the invalidation they provide: every row
written to a mapped table also updates one hot row in the generations table,
and core carries carve-outs in `translate/update.rs`, `translate/index.rs` and
`translate/trigger.rs` purely to let those triggers exist.

See `graph/docs/table-change-detection-design.md` for the proposed core
change.
