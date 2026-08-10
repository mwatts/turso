# Evaluation: JSONB property bags + expression indexes

**Date:** 2026-08-10  
**Branch:** `spike/graph-json-bag-property-store`  
**Question:** Do declared `JSONB` bags (custom types on) work with Cypher
`PropertyPhysical::JsonBag`, and do expression indexes on
`json_extract(props, '$.hot')` work the same as on TEXT bags?

## Correctness (debug tests)

`cargo test -p turso_graph_frontend --test property_physical_modes --test property_store_spike`

| Check | Result |
|---|---|
| TEXT bag Cypher read + SET | pass |
| JSONB bag Cypher read + SET (`props JSONB`, `jsonb(...)` insert) | pass |
| JSONB bag + expr index on `$.name`; EXPLAIN uses index | pass |
| JSONB bag without index; EXPLAIN does **not** use property index | pass |
| Harness C4 TEXT + C4 JSONB both `filter_plan_uses_index: true` | pass |
| Harness C2j control `filter_plan_uses_index: false` | pass |

Requirements for JSONB path:

- Open DB with `DatabaseOpts::with_custom_types(true)`.
- Column type `props JSONB NOT NULL`.
- Insert via `jsonb('…')` (or equivalent).
- Reads/writes still use `json_extract` / `json_set` (same as TEXT bag).
- Hot-key index: `CREATE INDEX … ON people(json_extract(props, '$.name'))`.

## Performance (release, 5000 nodes, 32 props)

Method: median of 5 after 1 warm; in-memory; `PROPERTY_STORE_SPIKE_INCLUDE_C4=1`.

| Config | bag type | filter_eq_ms | uses index? | project_all_ms | project_one_ms | set_one_ms | load_ms |
|---|---|---:|---|---:|---:|---:|---:|
| C2 TEXT bag | TEXT | 4.23 | no | 0.55 | 4.03 | 0.011 | 91 |
| C2j JSONB bag | JSONB | 4.43 | no | 0.50 | 4.52 | 0.012 | 97 |
| C4 TEXT + expr index | TEXT | **0.007** | **yes** | 0.62 | 0.38 | 0.016 | 121 |
| C4j JSONB + expr index | JSONB | **0.007** | **yes** | 0.53 | 0.37 | 0.016 | 125 |
| C3 EAV cells | — | 0.008 | yes | 50.1 | 0.46 | 0.015 | 1669 |
| C1 columns | — | 0.006 | yes | 3.46 | 0.32 | 0.017 | 134 |

## Evaluation

1. **JSONB bags are viable** for the hybrid product path. Cypher SET/RETURN works with custom types and `jsonb()` load.
2. **JSONB alone does not fix selective filters.** Without an expression index, filter cost matches TEXT bag (~4.2–4.4 ms scan + extract at this scale). Declaring `JSONB` is not a free index.
3. **Expression indexes work on both TEXT and JSONB bags** via the same
   `json_extract(props, '$.pN')` definition. Planner reports USING INDEX; filter
   drops to ~0.007 ms (same order as column / EAV).
4. **Map project stays bag-cheap** for both TEXT and JSONB (~0.5–0.6 ms) and far
   better than EAV full-cell project (~50 ms).
5. **Prefer product default bag as JSONB** when custom types are on (typed
   storage + same index recipe as TEXT). TEXT bags remain fine without custom
   types.
6. **Hybrid Outcome C stands:** bag (prefer JSONB when available) for maps;
   expression index or Cell for hot filter/merge keys. No need for LSM wide-col
   based on these numbers.

## Gaps still open

- Expression index is still **SQL harness / manual DDL**, not auto-created from
  semantic `index: true`.
- Cypher tests assert EXPLAIN on the **lowered SQL shape** for the hot key, not
  every query plan the binder might emit (aliases/subqueries).
- No on-disk page-cost comparison TEXT vs JSONB yet (all MemoryIO).
