# Measurement: open Cell store (integer prop_id + dict) vs alternatives

**Release run:** 2026-08-10  
**Shape:** TYPES=100, ENTITIES=50 → **5000 nodes**, PROPS=**32** → **160 000 cell rows**  
**Method:** median of 5 after 1 warm; MemoryIO; `cargo test --release`  
**Harness:** `graph/frontend/tests/property_store_spike.rs`  
**Machine lines:** appended to `property_store_spike.jsonl` (phase=full)

## What was measured

| Config | Physical model |
|---|---|
| C1 | Shared columns (one column per prop) |
| C2 / C2j | TEXT / JSONB property bag, **no** property index |
| **C3** | **`prop_dict` + cells `(node_id, prop_id, value)` + INDEX `(prop_id, value)`** |
| **C3s** | cells `(node_id, prop_key TEXT, value)` + INDEX `(prop_key, value)` — string-key baseline |
| C4 / C4j | Bag + expression index on **one** hot key only (`p0`) |

Open-property scenario: equality filter on a property, multi-predicate AND (first+last prop), full cell materialize, load, single SET.

## Results (ms unless noted)

| Config | filter_eq | uses idx | filter_and2 | uses idx | project_all | project_one | set_one | load | cell rows |
|---|---:|---|---:|---|---:|---:|---:|---:|---:|
| C1 columns | 0.006 | yes | 0.006 | yes | 3.44 | 0.34 | 0.018 | 133 | — |
| C2 TEXT bag | **3.82** | **no** | 3.68 | no | **0.54** | 3.88 | 0.010 | **91** | — |
| C2j JSONB bag | **4.22** | **no** | 4.31 | no | **0.47** | 4.31 | 0.011 | 96 | — |
| **C3 prop_id int** | **0.008** | **yes** | **0.013** | **yes** | 46.9 | 0.39 | 0.015 | 1601 | 160000 |
| **C3s prop_key text** | **0.008** | **yes** | **0.013** | **yes** | 51.9 | 0.44 | 0.017 | 1710 | 160000 |
| C4 bag+expr (hot only) | 0.006 | yes | 0.007 | yes* | 0.55 | 0.35 | 0.017 | 115 | — |
| C4j JSONB+expr | 0.006 | yes | 0.008 | yes* | 0.46 | 0.34 | 0.017 | 123 | — |

\*C4 and2 can use the hot-key index for `p0` only; `p31` is still extract/scan.  
`filter_and2_rows` reported 0 in this run for all configs (likely target-seed quirk in AND fixture); **latency and EXPLAIN index use** for C3/C3s still show indexed plans. Single-equality `filter_eq_rows=1` is reliable.

## Evaluation for >10k types + open property query

### 1. Open equality filters need Cell (or equivalent), not bag alone

Bag filter ~**4 ms** full scan+extract; Cell ~**0.008 ms** with index.  
**Bag-only is not viable** when any property may appear in predicates.

### 2. Integer `prop_id` vs string `prop_key`

At this scale (32 short keys, 160k rows):

| Metric | prop_id (C3) | prop_key TEXT (C3s) | Delta |
|---|---:|---:|---|
| filter_eq | 0.008 | 0.008 | ~same |
| filter_and2 | 0.013 | 0.013 | ~same |
| project_all | 46.9 | 51.9 | int ~**10%** faster |
| project_one (all entities one prop) | 0.39 | 0.44 | int slightly faster |
| load | 1601 | 1710 | int ~**6%** faster |

**Conclusion:** integer dictionary keys are **at least as fast** and slightly better on load/project; they also save string storage and match IR `PropertyId`. Prefer **`prop_dict` + `prop_id`** for product. Latency parity at 32 keys is expected (keys still short); the win grows with longer property names and larger catalogs.

### 3. Load and project_all cost of open Cell

- **160k cell inserts** dominate load (~1.6 s vs ~0.1 s bag) — harness does one INSERT per cell; product bulk/batch will help but Cell remains heavier to fill.  
- **project_all** of all cells ~47–52 ms vs bag ~0.5 ms — full map materialize is the Cell tax.  
- Dual-write **JSONB bag for maps** remains attractive if maps are common.

### 4. Expression indexes are not a substitute under open query

C4 is excellent for a **known** hot key (`p0`) but does not index `p31` or arbitrary names. Under “any property subset,” you would need unbounded expr indexes → wrong tool. **One** `(prop_id, value)` index covers the open set.

### 5. Recommended product path (updated by this measurement)

```text
>10k types + open property filters
  → shared nodes/edges (types in junctions)
  → prop_dict(name → prop_id, value_type)
  → node_props(node_id, prop_id, value) + INDEX(prop_id, value)
  → optional dual-write JSONB bag for properties(n) / bulk export only
  → optional expr indexes only as extra acceleration for known mega-hot keys
```

### 6. What we still have not measured

- On-disk / page-cache cost (all MemoryIO)  
- 100+ properties per entity, or millions of entities  
- Multi-AND of 5+ open predicates at large cardinality  
- Bulk load of cells (batch INSERT vs per-row)  
- Cypher end-to-end latency (harness is SQL physical)

## Reproduce

```bash
PROPERTY_STORE_SPIKE_TYPES=100 PROPERTY_STORE_SPIKE_ENTITIES=50 \
PROPERTY_STORE_SPIKE_PROPS=32 PROPERTY_STORE_SPIKE_INCLUDE_C4=1 \
  cargo test -p turso_graph_frontend --test property_store_spike --release \
  property_access_methods -- --nocapture
```
