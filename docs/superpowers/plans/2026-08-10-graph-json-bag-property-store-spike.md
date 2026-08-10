# Spike: JSON-bag property store vs wide-col indexes (graph frontend)

> **Status:** open spike on branch `spike/graph-json-bag-property-store`
> (branched from `feature/graph-frontend`). Design + measurement harness only;
> no production property-store switch lands from this branch until the decision
> gate below is answered with numbers. Wide-col / smoltable research folded in
> under [Wide-column reference](#wide-column-reference-smoltable-research)
> (research only; no storage port).

**Goal:** Measure whether collapsing ontology-scale graphs onto **shared physical
stores + JSON/JSONB property bags** is enough for the graph frontend, or whether
property predicates and lowering need a **true indexed property cell store**
(wide-col / EAV-with-indexes stand-in → later real storage type).

**Non-goals for this spike:**

- Exposing wide-column semantics through Cypher (that would be a new frontend).
- Implementing a new core storage format in this branch.
- Replacing role/endpoint indexes, CSR expand, spills, or label junctions.
- Full production migration of CREATE GRAPH defaults.

**Recommended end state (product direction this spike validates):**

```text
Cypher (unchanged)
  → semantic catalog (types are conceptual)
  → shared physical stores (few tables; types live in junctions)
  → property access method: columns | jsonb bag | indexed cells
  → core B-trees / VDBE
```

---

## Why this spike exists

At ontology scale (**~10K node types, ~5K relationship types**):

| Layout | Schema cost |
|---|---|
| One table per type (CREATE GRAPH default) | ~15K tables + ~15K endpoint/pair indexes → catalog open/prepare blow-up |
| Shared `nodes` / `edges` + junctions | O(1) tables; types are data |
| Shared stores + fixed property columns | Collapses tables, still needs wide/NULL or ALTER for prop evolution |
| Shared stores + JSON bag | Collapses props; weak selective property indexes |
| Shared stores + indexed cells (EAV or true wide-col) | Sparse props + real `(prop, value)` indexes for lowering |

JSON bag alone **does not** fix type explosion. Shared stores do. Bags and
indexed cells only compete **after** sources collapse.

---

## Hypotheses

| ID | Hypothesis | How we falsify |
|---|---|---|
| H1 | Type-per-table schema object count and open cost become unacceptable by **1K types**, catastrophic by **10K**. | Measure tables/indexes + registration / open time vs type count. |
| H2 | Shared stores alone remove the schema wall; property AM choice is then second-order for open cost. | Compare type-per-table vs shared-column at same entity count. |
| H3 | JSON bag is competitive with columns for **point property project** and **whole-map materialize** on shared stores. | Microbench project 1 / N props; bag within ~2× of column. |
| H4 | JSON bag is **not** competitive for **selective property filters** that should use an index (`WHERE n.p = ?` / `MATCH (n {p: ?})` at scale). | Filter latency + EXPLAIN: bag scan/extract vs EAV index probe. |
| H5 | If H4 holds and property-filtered scans are a real graph workload share, we need an indexed cell store (EAV proxy now; wide-col later) for lowering — not just bags. | Decision gate below. |
| H6 | Single-property SET cost favors cells over large bags once bag size grows. | SET one key with bag sizes 1 / 16 / 64 / 256 keys. |

---

## Configurations under test

All configs use the **same logical ontology** (type count, entities per type,
edges, property keys). Only physical layout changes.

### C0 — Baseline: type-per-table columns (today)

```text
NodeType_i(id, p0, p1, …) × N_types
RelType_j(id, start, end, …) × R_types
+ per-role indexes + label/type junctions after register_graph
```

### C1 — Shared stores + typed columns

```text
nodes(id PK, type_id, p0…pK)     -- K = union of property columns (dense)
edges(id PK, type_id, start, end, …)
+ indexes (type_id), (start), (end), (start,end)
+ optional label junction if multi-label
```

### C2 — Shared stores + JSON/JSONB bag

```text
nodes(id PK, type_id, props JSON/JSONB)
edges(id PK, type_id, start, end, props JSON/JSONB)
+ same structural indexes as C1
-- property access: json_extract(props, '$.name')
-- property filter: cannot use structural indexes; optional expr index later
```

### C3 — Shared stores + EAV cells (indexed-cell proxy for wide-col)

```text
nodes(id PK, type_id)
node_props(node_id, prop_id, value, PRIMARY KEY(node_id, prop_id))
CREATE INDEX node_props_by_value ON node_props(prop_id, value)
edges(...) + edge_props similarly
```

C3 is **not** a product design commitment. It is the cheapest way to measure
"what if lowering could probe a true property index?" without building a new
storage type. If C3 wins hard on filters and SET, that is evidence for a
**graph-frontend indexed-cell AM** (and only later, if still insufficient, a
core wide-col evaluation) — not evidence that EAV SQL is the final form.
See [Wide-column reference (smoltable research)](#wide-column-reference-smoltable-research).

### C4 — (optional, phase 2) Bag + selective expression indexes

Only if C2 loses H4 badly but product wants bags: measure cost of adding a
handful of `CREATE INDEX … ON nodes(json_extract(props,'$.hot'))` for hot keys.
Does not scale to open prop namespaces.

---

## Workloads

Fixed seeds. Report median of ≥5 runs after 1 warm-up. Release profile for
timing rows destined for history (`cargo test --release` / dedicated bench);
debug allowed only for harness correctness.

| Workload | Shape | What it stresses |
|---|---|---|
| **W-schema** | N ∈ {100, 1K, 5K, 10K} types; 0 rows | DDL, `sqlite_schema`, register/open |
| **W-load** | N=1K types; 50 entities/type; 2 props; R=500 types × 100 edges | bulk insert |
| **W-project-1** | load; `SELECT` / Cypher RETURN one property | materialize_properties path |
| **W-project-all** | load; return full property map | bag vs pivot vs wide row |
| **W-filter-eq** | load; filter on medium-selectivity prop (1% match) | indexability |
| **W-filter-miss** | load; filter matching 0 rows | index short-circuit |
| **W-set-one** | load; update one property on 1K entities | bag rewrite vs cell update |
| **W-hop+prop** | 2-hop pattern + property filter on intermediate | combined graph+prop plan |
| **W-ontology-open** | N types registered; open GraphConnection | catalog / SchemaCatalog cost |

Phase 1 of the spike may implement workloads as **raw SQL against the physical
layouts** (harness in `graph/frontend/tests/property_store_spike.rs`). Phase 2
wires the winning layout(s) through a **minimal property access method** in
lowering/mutation for a thin Cypher path (optional; only if SQL results leave
integration risk).

---

## Metrics

| Metric | Unit | Capture |
|---|---|---|
| `table_count` | count | `sqlite_schema` type=table |
| `index_count` | count | `sqlite_schema` type=index |
| `schema_sql_bytes` | bytes | sum of schema SQL text |
| `db_bytes` | bytes | file size after load (on-disk runs) |
| `register_ms` / `open_ms` | ms | wall time |
| `load_ms` | ms | bulk insert wall |
| `query_ms` p50/p95 | ms | per workload |
| `rows_touched` | count | where available / EXPLAIN estimate |
| `plan_uses_index` | bool | EXPLAIN QUERY PLAN contains index on filter path |
| `prepare_ms` | ms | first prepare vs steady |

Emit a single machine-readable line per run (JSON) so results can append to
`graph/test-results/property_store_spike.jsonl` without polluting conformance
history.

---

## Decision gate

Answer **one** of three outcomes with evidence from the metrics table.

### Outcome A — JSON bag is enough (for now)

Choose if **all** of:

1. H1 true and H2 true (shared stores fix the real wall).
2. H3 true (project within ~2× of columns).
3. H4 **false** or property-filtered scans are &lt; ~10% of target workload / acceptable as scans at target scale.
4. H6 bag SET acceptable for expected bag sizes.

**Next product work:** shared-store mapping + bag property AM behind
`property_column` / mutation SET; no wide-col storage project.

### Outcome B — Need indexed property cells

Choose if:

1. H2 true (shared stores still required), **and**
2. H4 true with large gap (EAV/index ≥5–10× faster than bag on W-filter-eq at load scale), **and**
3. Property filters / MERGE keys on properties matter for Foedus/ontology workloads.

**Next product work:**

- Near term: graph-internal EAV / `PropertyPhysical::Cell` (or hybrid: bag for
  cold props + indexed cells for declared/hot props) usable by lowering over
  ordinary tables. Still B-trees/VDBE; no core storage fork.
- Medium term: only if the cell proxy is insufficient (join tax, multi-prop
  materialize, SET), evaluate a **true wide-col table type in core**. That
  evaluation is separate from this spike and is **not** a port of smoltable LSM
  locality groups (see research section).

### Outcome C — Hybrid

Choose if project/map love bags (H3) but a **small set** of hot properties need
indexes (C4 or partial EAV). Common for ontologies with few key attributes and
large sparse tails.

**Next product work:** bag default + optional indexed property declaration in
semantic schema (`index: true` on SemanticProperty) without full wide-col.

### Explicit non-decisions

- Do **not** pick wide-col storage only because EAV looks cleaner in SQL.
- Do **not** keep type-per-table because bags "feel schemaless."
- Do **not** expose cell/wide-col syntax in Cypher from this spike.
- Do **not** treat smoltable/Bigtable as a production default for Turso core:
  use it as a **reference model** for sparse keys and scan locality only.
- Do **not** implement locality-group LSM partitions, levelled cell compaction,
  or storage-level per-family version/TTL GC from this spike (requires core
  abstractions beyond B-tree/page/WAL/MVCC).

---

## Spike implementation plan

### Phase 0 — Instrument baseline explosion (this branch)

- [x] Branch `spike/graph-json-bag-property-store` from `feature/graph-frontend`
- [x] This design doc
- [x] Harness skeleton: `graph/frontend/tests/property_store_spike.rs`
- [x] Run W-schema for C0 at N=1000 (release); table/index counts + schema_ms recorded
- [x] Document hard limits observed — see Phase 1 progress note (1K tables ~1.6s
      schema DDL; no OOM at 1K; 5K/10K still open)

### Phase 1 — Physical AM microbench (SQL only, no Cypher rewrite)

- [x] Implement C1/C2/C3 builders in the harness (shared stores)
- [x] W-load + W-project-1 + W-project-all + W-filter-eq + W-set-one
- [x] EXPLAIN capture for filter plans (`filter_plan_uses_index`)
- [x] Progress metrics: `graph/test-results/property_store_spike_phase1.md` +
      `property_store_spike.jsonl` (release TYPES=1000 / ENTITIES=50)
- [x] Wide-col / smoltable research folded into this plan (reference only;
      product path is B-tree `PropertyPhysical`, not LSM port)

**Phase 1 result so far (not final gate):** H1/H2 hold (shared stores required).
H4 holds at 50k rows (bag filter ~18.5 ms, no index; C3 ~0.008 ms, uses index).
H3 partial (bag wins whole-map project). H6 not stressed (PROPS=8 only).

### Phase 1b — Close measurement gaps before product AM (SQL only)

Research bound: if filters need indexes, **next product step is graph-frontend
`PropertyPhysical::Cell` (or hybrid) on ordinary B-trees**, not a smoltable/LSM
core fork. Phase 1b only finishes the numbers that choose A vs B vs C.

- [x] W-schema at TYPES=5000 for C0 vs C1–C3 (schema-only; C0 ~28 s, 5000 tables)
- [x] Shared C1–C3 full run with PROPS=32 (TYPES=100, ENTITIES=50)
- [x] C4: bag + expression index on hot key; filter uses index, matches cell speed
- [x] W-hop+prop: shared `edges(start,end)` + property filter on neighbor
- [x] Decision matrix: **Outcome C (hybrid)** — see
      `graph/test-results/property_store_spike_phase1b.md`

**Exit Phase 1b:** **Outcome C chosen.** Shared stores required. Bag-only
rejected for selective property equality without indexes. Prefer bag (or columns)
for map materialize plus indexed hot keys (expression index and/or Cell). Product
path remains frontend AM over B-trees; core wide-col stays a separate evaluation.

### Phase 2 — Thin graph integration (after 1b; opt-in physical mode only)

Phase 1b selected hybrid (C). Still no Cypher cell language and no core storage
type. Catalogs opt in via `property_physical`; default remains Column.

- [x] `PropertyPhysical` enum + `RelationalCatalogSnapshot::property_physical`
      (`graph/frontend/src/property_physical.rs`): Column | JsonBag | Cell
- [x] Lower property materialize + property expression read through physical modes
- [x] SET / REMOVE single property through physical modes (Cell = DELETE+INSERT)
- [x] Cypher integration tests: bag, cell, and default column
      (`graph/frontend/tests/property_physical_modes.rs`)
- [x] CREATE / MERGE write path for bag and cell properties (`insert_entity`)
- [x] JSONB bag + expression-index correctness (harness C2j/C4j + Cypher tests)
- [ ] If hot keys only: wire semantic `index: true` (or equivalent) to Cell/C4, not
      full Cell for every property (product follow-up; not required to close Phase 2)

### Phase 3 — Recommendation PR back to `feature/graph-frontend`

**Merge gate (explicit):** do **not** merge this spike into `feature/graph-frontend`
until both pass on the branch tip:

```bash
mise run corpus              # full conformance, release
mise run cypherbench-full    # full cypherbench, release
```

**Product decision (revised):** **one** property store for all graphs — Cell
(`prop_dict` + `node_props` / edge props). No consumer opt-in, no long-term
`PropertyStoreMode` bifurcation. SourceColumns dual path is transitional only
while CREATE GRAPH, semantic schema, fixtures, corpus, and cypherbench move.

- [x] Cell registration + SchemaCatalog routing (transitional API still has
      `register_graph_open_cell` until single-path flip)
- [x] E2E: `graph/frontend/tests/open_property_store.rs`
- [x] ADR in `docs/graph-internals.md` (Cell is the only store)
- [x] W1: whole-map SET, DELETE purges cells, procedure keys from dict
- [x] W2: `register_graph` always Cell; CREATE GRAPH topology-only + dict seed;
      `source_id` in node_props/edge_props; no opt-in API; no data migration
- [x] W4 cleanup: remove PropertyStoreMode / register_graph_open_cell /
      payload-column migration / SourceColumns fallback; edge_props installed;
      live `db.propertyKeys` from prop_dict
- [ ] W3: `mise run corpus` + `cypherbench-full` re-baseline on tip before merge
- [ ] Follow-up: SemanticProperty.column → name+type only (column maps remain
      when a real SQL column exists for multi-owner / STRICT)
- [x] Corpus vs feature tip (2026-08-10): zero OpenCell regressions while dual
- [x] cypherbench-full (2026-08-10): exit 0, zero outcome deltas
- [ ] Known debt: ~26 bare `MERGE (a)` / `EmptyMergeKey` (feature tip too)
- [ ] Do not merge experimental harness numbers into conformance baselines

---

## Property access method sketch (for Phase 2 / product)

Today:

```text
property_column(source, property) -> Option<String>
// lowering: alias.column
// mutation: UPDATE t SET column = ?
```

Target internal enum (not user-visible):

```rust
enum PropertyPhysical {
    Column { column: String },
    JsonBag { bag_column: String, key: String },
    // Indexed-cell proxy; true wide-col would share this lowering shape
    Cell { props_table: String, prop_key: String },
}
```

Lowering:

| Op | Column | JsonBag | Cell |
|---|---|---|---|
| Read | `a.col` | `json_extract(a.bag, '$.k')` | `(SELECT value FROM props WHERE id=a.id AND prop=?)` or join |
| Filter eq | `a.col = ?` | `json_extract(...) = ?` | `EXISTS (… prop=? AND value=?)` / join + index |
| SET | `SET col=?` | `json_set(bag, '$.k', ?)` | UPSERT props row |
| REMOVE | `SET col=NULL` | `json_remove(bag, '$.k')` | DELETE props row |
| properties(n) | enumerate payload cols | bag as-is / json() | group_concat / json_group_object |

Structural columns (id, role endpoints, type_id) **never** go in the bag/cells.

---

## Ontology mapping policy (paired recommendation)

Independent of bag vs cells, ontology mode should default to:

```text
1 node source table  + labels/types junctions  (or type_id column + junction)
1 relationship source table + type junction + start/end indexes
Many semantic types / CREATE GRAPH types → rows in membership, not new tables
```

CREATE GRAPH sugar today creates one table per NODE/RELATION. The spike may
leave that sugar alone and only measure shared layouts via harness DDL. Product
follow-up: `CREATE GRAPH … STORAGE SHARED` or registration-time physical policy
for large ontologies.

---

## Wide-column reference (smoltable research)

Research-only pass (2026-08-10) against
[mwatts/smoltable](https://github.com/mwatts/smoltable) and Turso core/graph
docs. **No implementation** from that pass. Use this section to bound what
"wide-col" means when Outcome B says "later true wide-col."

### What smoltable/Bigtable are

| Idea | Detail |
|---|---|
| Logical model | Sparse multi-version map: `(row_key, family:qualifier, timestamp) → bytes` |
| Column keys | Families are schema objects; qualifiers are dynamic UTF-8 (empty CQ allowed) |
| Physical cell key | `row_key:cf:cq:!timestamp` (timestamp bit-negated → newest-first in LSM order) |
| GC | Per–column-family `version_limit` / `ttl_secs` (lazy, not row-MVCC) |
| Locality groups | Selected families in **separate LSM partitions** so column-filtered scans skip unrelated families |
| Engine | Multi-partition LSM (fjall): `_man_`, `_dat_`, optional `_lg_*`; levelled compaction |

Bigtable (OSDI 2006 ancestor) is the same multi-dimensional sorted map plus
locality groups as separate SSTables — an **LSM/SSTable** design, not a single
B-tree row store.

### What Turso's durable path is (and is not)

| Turso today | Wide-col cells |
|---|---|
| SQLite-compatible **B-tree** table leaves (rowid + full-row payload) + index B-trees | One LSM key-value **per cell version** |
| WAL durability at **page-frame** granularity | Per-cell keys + levelled compaction |
| MVCC versions **whole rows** (begin/end ts, `.db-log`, checkpoint into B-tree) | Multi-version **cells** with per-family version/TTL GC |
| No locality-group partitions | Families split across LSM trees |

**Encodable without a core fork:** sparse multi-property (and even multi-version)
cells as ordinary tables/indexes — e.g. C3 EAV/`WITHOUT ROWID`, or hybrid bag +
hot-key indexes — still lowered to B-trees/VDBE.

**Not available without core work beyond this spike:** native locality-group LSM
partitions, levelled cell compaction, storage-level per-family version/TTL GC.

### Mapping onto this spike / graph frontend

Without forking core, "wide-column-style property store" means a **graph-frontend
physical access method** over shared relational tables, not a new storage format
or Cypher-visible cell language:

```text
Cypher → semantic catalog (unchanged)
      → shared physical node/edge stores
      → PropertyPhysical: Column | JsonBag | Cell
      → core B-trees / VDBE
```

- **`Cell`** is the indexed-cell / wide-col **proxy** (props table + prop key;
  C3 measures this). Identity and role endpoint columns never go into bag or cells.
- Labels / relationship types stay junction (or type_id + junction) over
  registered source identities — that is catalog/overlay, not the property layout.
- Ontology-scale types collapse onto **one node source + one relationship source**
  (see ontology mapping policy above).

### Extension surfaces (what belongs where)

| Surface | Use for property store? |
|---|---|
| **`IndexMethod`** (core) | Yes as the index AM attachment on ordinary tables (create/destroy, insert/delete, query patterns, cost, MVCC modes) — same family as FTS via `CREATE INDEX … USING …`. Best fit if property cells need custom indexes without replacing the main B-tree engine. |
| **Virtual tables** | No as primary durable property-cell store. Right for external data, generators, catalog emulation, expand operators. |
| **Dialect / FrontendCompiler** | Cypher/SQL surface + reprepare seams only — not sparse cell storage. |
| **Catalog hooks** (`Dialect::register_catalog`, graph semantic catalog) | Conceptual type/property identity over physical tables; decouple ontology types from per-type tables. |

Existing graph IR and session/namespace semantics stay **above** the physical
property layout: `GraphConnection` → registered graph; binder uses
source/type/property identities; semantic ids map to physical sources only at
lowering; generation counters refresh catalog snapshots (not per-cell versions).

### Implications for decision gate and product ordering

1. **C3 winning on W-filter-eq / W-set-one** → Outcome B or C → implement
   graph-internal `PropertyPhysical::Cell` (or hybrid), not "port smoltable."
2. **True wide-col table type in core** is medium-term and only if the cell
   proxy is insufficient. Even then, start from SQLite-compatible encodings
   (composite keys / family tables), not an LSM locality-group fork, unless a
   separate core project deliberately adds that stack.
3. **Cypher must not** grow cell/family/timestamp syntax from this spike.
4. **Open measurement gaps** (research Partial): no head-to-head
   EAV/`WITHOUT ROWID` vs smoltable prefix-scan numbers; no shipped
   `IndexMethod` for property cells (FTS/vector analogy only); no graph layout
   for versioned-per-property-cell history; shared-store product policy
   (`STORAGE SHARED`, etc.) proposed, not implemented.

### External references

- smoltable cell key / table GC / locality groups:
  `https://github.com/mwatts/smoltable` (and upstream
  `https://github.com/marvin-j97/smoltable`)
- Bigtable OSDI 2006 paper (model ancestor)
- Turso: `docs/agent-guides/storage-format.md`, `docs/agent-guides/mvcc.md`,
  `core/index_method/mod.rs`, `docs/graph-internals.md`,
  `docs/graph-frontend-core-alignment.md`

---

## Risks and mitigations

| Risk | Mitigation |
|---|---|
| JSON type fidelity vs Cypher (int/float/bool) | Use JSONB when custom types on; assert round-trip in harness |
| EAV overstates wide-col wins (join tax) | Report multi-prop project cost; any core wide-col must beat EAV on project/SET, not only filters |
| Confusing C3 with production wide-col / smoltable port | C3 = measurement proxy only; product path is `PropertyPhysical` over B-trees (see research section) |
| Scope creep into LSM locality groups / cell TTL GC | Explicit non-decision; needs separate core project if ever pursued |
| In-memory IO hides page cost | Dual-run MemoryIO vs tempfile for load/filter |
| 10K-table C0 never finishes | Cap C0 and extrapolate from 100/500/1K |
| Conformance regression from partial bag path | Phase 2 opt-in only; default remains columns |

---

## Commands

```bash
# Harness (correctness + small metrics; debug OK)
cargo test -p turso_graph_frontend --test property_store_spike -- --nocapture

# Larger timing (release, intentional for this spike)
cargo test -p turso_graph_frontend --test property_store_spike --release -- --nocapture

# Optional scale knobs (harness env)
PROPERTY_STORE_SPIKE_TYPES=1000 \
PROPERTY_STORE_SPIKE_ENTITIES=50 \
cargo test -p turso_graph_frontend --test property_store_spike --release -- --nocapture
```

Do not use these numbers as conformance history. Append spike JSON to
`graph/test-results/property_store_spike.jsonl` only.

---

## File map

| File | Role |
|---|---|
| `docs/superpowers/plans/2026-08-10-graph-json-bag-property-store-spike.md` | This design + wide-col research fold-in |
| `graph/frontend/src/property_physical.rs` | `PropertyPhysical` + SQL render for Column/JsonBag/Cell |
| `graph/frontend/tests/property_store_spike.rs` | Measurement harness (C0–C4, workloads, JSON lines) |
| `graph/frontend/tests/property_physical_modes.rs` | Phase 2 Cypher bag/cell/column integration |
| `graph/test-results/property_store_spike.jsonl` | Machine results |
| `graph/test-results/property_store_spike_phase1b.md` | Phase 1b decision (Outcome C) |
| `docs/graph-internals.md` | Pointer under Future features |
| External: [mwatts/smoltable](https://github.com/mwatts/smoltable) | Wide-col reference model only (LSM cells); not a Turso dep |

---

## Success criteria for closing the spike

1. Decision gate A/B/C filled with numbers for at least N=1K shared-store
   workloads and C0 schema scaling through the highest N that completes.
2. Clear written recommendation: bag-only, indexed-cells, or hybrid.
3. If B or C: listed follow-ups with ordering (shared store mapping first,
   then property AM, then optional core wide-col).
4. No production default change without a separate implementation PR.
