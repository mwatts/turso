# Property-store spike — Phase 1 progress

## Problem

The graph frontend stores each Cypher node type and relationship type as its own
SQL table when you use the default `CREATE GRAPH` path (one table per declared
type). For an ontology with thousands of types, that creates thousands of tables
and indexes in `sqlite_schema`, which slows DDL and open and makes unlabeled
scans fan out across many sources.

Separately, once types share fewer tables, properties still need a physical
home. The open question is whether a single JSON text column per row (a
**property bag**) is enough for reads and filters, or whether property equality
filters need a real B-tree on `(property name or id, value)`.

This spike measures physical layouts only. It does not change Cypher syntax and
does not add a new storage engine type.

## Intended end state (product direction under test)

1. Many conceptual types (labels / relationship types) map to a small number of
   physical tables. Type membership lives in junction tables or a `type_id`
   column, not in “one SQL table per type.”
2. Property values are stored through a pluggable physical mode behind the
   existing `property_column` / mutation SET path: fixed columns, a JSON bag, or
   indexed property cells.
3. Cypher callers still write `n.name` and `SET n.name = …`. They do not see
   bag or cell layout names.

Phase 1 only proves (1) and compares the property modes with raw SQL. It does
not ship (2) into the frontend compiler.

## What we measured

**Branch:** `spike/graph-json-bag-property-store`  
**Harness:** `graph/frontend/tests/property_store_spike.rs`  
**Design:** `docs/superpowers/plans/2026-08-10-graph-json-bag-property-store-spike.md`  
**Raw JSON lines:** `graph/test-results/property_store_spike.jsonl`

Four physical layouts (configs):

| Id | Layout | What it models |
|---|---|---|
| C0 | One table per type; properties are columns | Today’s CREATE GRAPH default at scale |
| C1 | One `nodes` table; properties are columns | Shared store, fixed schema props |
| C2 | One `nodes` table; properties in one `props` JSON text column | Shared store + property bag |
| C3 | One `nodes` table plus `node_props(node_id, prop_id, value)` with index `node_props_by_value(prop_id, value)` | Shared store + **indexed property cells** (stand-in for a future wide-column cell store; not the final product shape) |

**Release run:** `PROPERTY_STORE_SPIKE_TYPES=1000`, `ENTITIES=50` per type,
`PROPS=8` → 50 000 node rows. Method: median of five timed queries after one
warm query, in-memory Turso (`MemoryIO`), optimized build (`cargo test --release`).

Default debug run (TYPES=50) also passes three tests; numbers below are release.

## Results that matter

### Schema cost (create tables and indexes only)

| Config | SQL tables | SQL indexes | Time to create schema |
|---|---:|---:|---:|
| C0 | 1000 | 1000 | ~1628 ms |
| C1 | 1 | 2 | ~0.15 ms |
| C2 | 1 | 1 | ~0.08 ms |
| C3 | 2 | 2 | ~0.17 ms |

**Checkable claim:** at 1000 types, type-per-table creates 1000 tables and
~1.6 s of schema DDL; shared layouts stay at one or two tables and under 1 ms.
JSON bags do not fix type explosion; fewer entity tables do.

### Property work (50 000 entities already loaded)

| Config | Equality filter (`prop = one unique value`) | Plan uses B-tree index? | Read one property from all rows | Read full property payload | Update one property on one row | Load all rows |
|---|---:|---|---:|---:|---:|---:|
| C1 columns | 0.006 ms | yes (`nodes_p0`) | 3.56 ms | 11.2 ms | 0.016 ms | 552 ms |
| C2 JSON bag | 18.5 ms | no | 19.7 ms | 4.1 ms | 0.013 ms | 361 ms |
| C3 indexed cells | 0.008 ms | yes (`node_props_by_value`) | 4.33 ms | 127 ms | 0.014 ms | 4257 ms |

How each filter is expressed in the harness:

- C1: `SELECT id FROM nodes WHERE p0 = '…'`
- C2: `SELECT id FROM nodes WHERE json_extract(props, '$.p0') = '…'`
- C3: `SELECT node_id FROM node_props WHERE prop_id = 0 AND value = '…'`

**Checkable claims:**

1. Bag filters do not use an index on this plan; cell filters do
   (`filter_plan_uses_index` in the JSON lines).
2. At this shape, bag equality filter median is ~18.5 ms; cell equality filter
   median is ~0.008 ms (about 2300× slower for the bag full scan + extract).
3. Returning the whole property map is cheapest with a bag (~4.1 ms for 50 000
   rows) because one column is already the map. Returning all cells is
   expensive (~127 ms for 400 000 property rows).
4. Updating a single property on one row is similar across C1–C3 when each bag
   holds only eight keys. That does not yet measure large-bag rewrite cost.
5. Cell load time (~4.3 s) is dominated by one INSERT per property value. That
   cost is a harness artifact of naive EAV inserts, not a claim about a future
   packed wide-column row format.

C0 query times in the same run only touch `node_0` (one type’s 50 rows). Do not
compare them to C1–C3 full-table scans.

## Decision status (not a product commit)

| Question | Answer so far | Evidence |
|---|---|---|
| Must large ontologies stop using one table per type? | **Yes** | C0 vs C1–C3 schema table counts and schema_ms |
| Is a JSON bag alone enough if property equality filters matter? | **No at 50k rows without extra indexes** | C2 filter_eq_ms and `filter_plan_uses_index: false` |
| Is a bag useless? | **No** | C2 project_all beats C3 for whole-map read |
| Do we need a true core wide-column table type today? | **Not decided** | C3 only proves “indexed cells help filters”; it does not prove EAV SQL is the right long-term format |

Working lean (still provisional):

- Prefer **shared physical stores** for many types.
- Prefer a **hybrid property mode** if both map materialize and selective
  filters matter: bag (or columns) for bulk property maps, plus indexed cells
  (or later a packed cell store) for properties that appear in filters and
  merge keys.
- If almost all queries are map project / hop without property predicates, bag
  alone after shared stores may be enough.

Do not change CREATE GRAPH defaults or Cypher lowering until Phase 1 finishes
the larger-bag and hop+filter runs below.

## Failure modes of each layout (for implementers)

| Layout | Fails when | Observable effect |
|---|---|---|
| C0 type-per-table | Type count grows into thousands | `sqlite_schema` size and schema DDL time grow with type count; unlabeled multi-type reads need many UNION branches |
| C1 shared columns | Property set is sparse or evolves often | Wide rows full of NULLs, or frequent `ALTER TABLE ADD COLUMN` |
| C2 bag | Filter or sort on a property at scale without expression indexes | Full table scan + `json_extract` on every row; EXPLAIN shows no useful property index |
| C3 EAV cells | Many properties read together, or bulk load via one-row-per-cell | High row count, slow full-map materialize, slow multi-insert load |

## Non-goals of this spike (unchanged)

- Cypher API for wide-column or EAV.
- A new Turso core storage format in this branch.
- Final production choice of bag-only vs hybrid vs cells-only without the
  remaining measurements.

## Tests that gate this work

```bash
cargo test -p turso_graph_frontend --test property_store_spike -- --nocapture

PROPERTY_STORE_SPIKE_TYPES=1000 PROPERTY_STORE_SPIKE_ENTITIES=50 PROPERTY_STORE_SPIKE_PROPS=8 \
  cargo test -p turso_graph_frontend --test property_store_spike --release -- --nocapture
```

Assertions in the harness: shared bag layout creates ≤4 user tables; C0 creates
one table per type; C3 equality filter EXPLAIN uses an index.

## Next measurements (in order)

1. **Schema only at TYPES=5000** for C0 vs C1–C3 (stop C0 if DDL time or memory
   is unacceptable; still record the highest N that finishes).
2. **Shared C1–C3 full run** at TYPES=1000, ENTITIES=100, PROPS=32 to stress bag
   size on project and single-key SET.
3. **Edge table + property filter** SQL (hop then filter, or filter then hop) so
   property mode is judged with a graph-shaped plan, not only single-table
   filters.
4. Only if those results stay ambiguous: Phase 2 thin frontend change that maps
   one Cypher `n.prop` read and one `SET` through bag and cell modes behind
   `RelationalCatalogSnapshot`, still opt-in.

## Open questions

1. What share of Foedus/ontology Cypher queries filter on properties versus only
   labels, types, and hops? That share decides hybrid vs bag-only.
2. Which properties must be merge keys or unique constraints? Those almost
   certainly need indexed cells or columns, not bag-only equality.
3. If indexed cells win filters, is the product form EAV tables, hybrid hot
   columns, or a later packed wide-column table type in core?
