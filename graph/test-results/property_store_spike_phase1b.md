# Property-store spike — Phase 1b progress and decision matrix

## Research bound (smoltable / wide-col section)

The plan’s [wide-column reference](../../docs/superpowers/plans/2026-08-10-graph-json-bag-property-store-spike.md#wide-column-reference-smoltable-research)
states:

1. **smoltable/Bigtable** are LSM sparse multi-version cell stores with locality
   groups. Turso’s durable path is B-tree rows + WAL + (optional) row MVCC.
2. **C3 is a measurement proxy** for “indexed property cells on B-trees,” not a
   commitment to port smoltable.
3. If filters need indexes, **product work is graph-frontend
   `PropertyPhysical::Cell` or hybrid** over ordinary tables (or bag + hot-key
   expression indexes). A core wide-col table type is only a later evaluation if
   that fails.
4. **`IndexMethod`** is the right core hook for custom property indexes if plain
   B-tree indexes are not enough. Virtual tables are not the primary durable
   property store.

## How to read harness output

Each `PROPERTY_STORE_SPIKE {…}` line is self-describing. Look for:

| Field | Meaning |
|---|---|
| `phase` | `run_banner` (legend once per test), `schema`, `full`, or `c4_plan_check` |
| `config` / `config_means` | Which layout and what it stands for |
| `measures` | What SQL work was timed |
| `hypothesis` | H1–H6 tags from the spike plan |
| `feeds_outcome` | How the numbers push toward A / B / C |
| `filter_plan_uses_index` | Property-filter indexability (H4); **not** hop-only indexes |
| `hop_prop_plan_uses_index` | Any index on hop+prop plan (often edge index) |

Module docs in `graph/frontend/tests/property_store_spike.rs` list configs,
workloads, and outcome definitions in one place.

## Phase 1b measurements (release)

### W-schema TYPES=5000 (schema only)

| Config | tables | indexes | schema_ms |
|---|---:|---:|---:|
| C0 type-per-table | 5000 | 5000 | **~27988** (~28 s) |
| C1 shared columns | 2 | 5 | ~0.30 |
| C2 JSON bag | 2 | 4 | ~0.20 |
| C3 EAV cells | 3 | 5 | ~0.29 |

H1/H2 hold harder at 5k types: type-per-table schema create is tens of seconds;
shared layouts stay sub-millisecond.

### Shared full run: TYPES=100, ENTITIES=50 (5000 nodes), PROPS=32, C4 on

| Config | filter_eq_ms | filter uses index | project_all_ms | set_one_ms | hop_prop_ms | hop uses index |
|---|---:|---|---:|---:|---:|---|
| C1 columns | 0.007 | yes | 3.22 | 0.017 | 0.011 | yes |
| C2 JSON bag | **3.79** | **no** | **0.53** | 0.011 | 0.013 | yes (edge) |
| C3 EAV cells | 0.008 | yes | 46.5 | 0.014 | 0.018 | yes |
| C4 bag + expr index on `p0` | **0.007** | **yes** | **0.51** | 0.016 | 0.013 | yes |

Method: median of five timed queries after one warm run; in-memory Turso;
`cargo test --release`. Machine lines:
`graph/test-results/property_store_spike.jsonl` (appended Phase 1b rows).

### Hop+prop note

`hop_prop` is `edges.start = 0` then filter neighbor property. With a single
outgoing edge, edge indexes dominate; bag property extract on one row is cheap.
This does **not** reverse the C2 vs C3 filter gap on full-table property scans
(`filter_eq`).

### H6 (large bag SET)

Single-row SET with PROPS=32 is still ~0.01 ms for bag and cells. Bag rewrite
cost is not the differentiator at this bag size. Filter indexability is.

## Decision matrix (Phase 1b exit)

| Outcome | Criteria | Status |
|---|---|---|
| **A** bag-only | H4 false or filters rare; no need for property indexes | **Rejected** for workloads with selective property equality: C2 has no index and ~3.8 ms full scan at 5k rows / 32 props |
| **B** cells for all properties | Need index on every property; open namespace | **Possible** but C3 project_all and load cost are high; do not ship naive EAV as the only mode |
| **C hybrid** | Map project wants bag; hot filters want indexes | **Chosen for product direction** |

### Chosen direction: Outcome C (hybrid), implemented as B-tree property modes

**Must become true before production AM ships:**

1. Ontology-scale graphs use **shared node/edge tables** (types in junctions or
   `type_id`), not one SQL table per type.
2. Default property storage may be a **JSON/JSONB bag** (or fixed columns for
   dense known schemas).
3. Properties that appear in equality filters, MERGE keys, or unique constraints
   get an **indexed path**: either  
   - `PropertyPhysical::Cell` (props table + `(prop_id, value)` index), or  
   - bag + **declared hot-key expression index** (C4 shape: one index per hot
     property name).
4. Cypher does not expose cell/family/timestamp syntax.
5. Core does **not** gain an LSM wide-col engine from this spike.

C4 shows hot-key bag indexes restore filter indexability (~0.007 ms, uses
index) while keeping bag-speed map project (~0.51 ms). That is the hybrid
evidence without requiring every property to be a cell.

## Ordered product follow-ups (Phase 2+)

1. Shared-store physical policy for large ontologies (registration /
   `CREATE GRAPH` option).
2. Internal `PropertyPhysical { Column, JsonBag, Cell }` behind
   `property_column` / mutation SET.
3. Semantic (or registration) flag for “index this property” → Cell or expression
   index on bag.
4. Optional core `IndexMethod` only if expression indexes / EAV indexes are
   insufficient.
5. Core wide-col table type only if (2)–(4) fail product needs (separate project;
   not smoltable port by default).

## Phase 2 status

Phase 2 thin Cypher integration is **now in scope** (Outcome C is chosen).
Still opt-in physical mode only; default CREATE GRAPH remains columns until a
separate implementation PR.

## Reproduce

```bash
# Schema wall at 5k types
PROPERTY_STORE_SPIKE_TYPES=5000 cargo test -p turso_graph_frontend \
  --test property_store_spike --release schema_scaling -- --nocapture

# Hybrid evidence: bag vs cells vs bag+expr index
PROPERTY_STORE_SPIKE_TYPES=100 PROPERTY_STORE_SPIKE_ENTITIES=50 \
PROPERTY_STORE_SPIKE_PROPS=32 PROPERTY_STORE_SPIKE_INCLUDE_C4=1 \
  cargo test -p turso_graph_frontend --test property_store_spike --release -- --nocapture
```
