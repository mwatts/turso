# Graph frontend: ontology bootstrap is superlinear and recompiles SQL on every mutation — a 51-entity install exceeds 16 minutes

**Component:** `graph/frontend` (`turso_graph_frontend`), with one contributing item in `core`
**Branch:** `feature/graph-frontend`
**Severity:** blocks product installation
**Reported by:** limen (silo host daemon), via foedus / `tessera-turso`

---

## 1. Summary

Installing a 51-entity ontology through `TursoGraphStore::upsert_type` takes more than 16 minutes
on an empty database — it had still not finished when observation stopped. Each `upsert_type` costs 5.07 s on average, and the cost grows as the number
of already-installed types grows (3.1 s for the first ten, 6.8–7.1 s for types 51–80).

A CPU profile attributes the time to three things, all of which are per-mutation overhead rather
than the work the mutation asks for:

| Cost | Share of the working thread | Backlog item |
|---|---|---|
| `SemanticConstraintSnapshot::validate_state` → `catalog::query_rows` → `prepare_with_origin` → `compile_cmd` → `translate` | 2500 / 6153 = **40.6%** | 2 and 3 |
| `op_open_ephemeral` → `TempFile::new` → `tempfile::tempdir()` (create + `remove_dir_all`) | 961 / 6153 = **15.6%** | **not recorded — new** |
| `translate_insert` → `fire_trigger` → `translate_update` → `build_read_scope_tables` → `String::clone` | 434 / 6153 = **7.1%** | 5 |

Separately, the embedding WAL thread spends 841 of 847 working samples in `fsync`, one per frame.

This is the same profile `graph/PERFORMANCE_BACKLOG.md` records. Fix 1 (the
`schema_generation` split, `32f03b1c4`) is in the revision under test and did help; items 2, 3, 4
and 5 are open and are what remains.

## 2. Impact on the consuming product

`limen` installs itself as a macOS launchd user agent. The installer waits 60 s for the daemon to
bind its socket and answer `/v1/health`, then fails:

```
Error: the agent was loaded but never became healthy — see ~/Library/Logs/Limen/daemon.log.
Caused by: no health response on ~/.limen/daemon.sock within 60s
```

First-run installation on an empty store therefore fails 100% of the time. The daemon is not hung
— it is still installing schema — but no product can reasonably hold a first-run gate open for
15 minutes.

Steady state is also affected. A restart against an already-populated store, where the ontology is
byte-identical and nothing needs to change (`registered=0 migrated=0 reseeded=0 skipped=15`),
still took **8 m 45 s**, because the install path re-`upsert_type`s every type unconditionally and
each one pays the full validation cost.

## 3. Versions

| Component | Revision | Notes |
|---|---|---|
| turso | `5cef48f699d3f0f71a520775b1e4e118b0973db8` | includes `32f03b1c4` (backlog fix 1) |
| `feature/graph-frontend` tip | `099e06f39` | 3 commits ahead; see below |
| foedus | `72f03ee` | `origin/main` tip |
| limen | `main` | pins foedus `72f03ee` |

The three commits between the tested revision and the branch tip do not change this behavior:

```
099e06f39 docs: run conformance with CI=1 so it builds release      (docs/CI)
9038714e9 core: make committed DDL move every connection's change token
01cf068b6 core: add per-table change tokens without per-row cost
0d9f7cd1c graph/docs: fix three holes in the table-change-detection design   (docs)
```

`git diff --stat 5cef48f..099e06f39 -- graph/frontend/src` is **empty**. The core primitive for
backlog item 5 exists, but the frontend does not consume it: `install_generation_triggers` is
still called at `graph/frontend/src/catalog.rs:689`.

## 4. Reproduction

Any workload that creates N node types through the frontend in a loop reproduces it; the shape is
the bootstrap loop the backlog document already describes. What was measured here:

1. Empty store, `limen` daemon boot, which drives foedus `Engine::apply_graph_schema` and then one
   `foedus.LoadSchemaEntry` action per ontology type. Each action becomes one
   `TursoGraphStore::upsert_type` on the `tessera-turso` connection worker.
2. Ontology: 51 entities, 3 fragments, 3 relations, in two passes (18 built-in types, then the
   full set).
3. Machine: Apple Silicon, macOS 25.6.0, release build.

The `graph/frontend/tests/catalog_refresh.rs` harness plus the "count `Preparing: {sql}` events"
method recorded in `PERFORMANCE_BACKLOG.md` should reproduce the statement-count side of this
without the embedding stack.

## 5. Measurements

### 5.1 Wall clock, empty-store boot

```
01:05:04  boot start
01:05:17  engine started                                   (+13 s)
01:05:17  schemas loaded  entities=18 fragments=0 relations=1
01:05:28  graph schema installed                           (+11 s)
01:06:27  default sensitivity markings bootstrapped         (+59 s)
01:06:28  schemas loaded  entities=51 fragments=3 relations=3
01:07:24  graph schema installed                           (+56 s)
01:16:34  default sensitivity markings bootstrapped (2nd pass)
01:21:33  still installing — no socket bound
```

Elapsed at last observation: **16 m 39 s**, still short of "bundles ready", still making progress.
The install did not complete within the observation window. Gate is 60 s.

### 5.2 Per-`upsert_type` distribution (n = 81 completed)

```
total 411.0 s   mean 5.07 s   median 5.00 s   p90 10.00 s   max 15.00 s
```

Every one is serial: `action submitted` → `action completed` → next `action submitted`.

### 5.3 Cost grows with the number of installed types

Mean duration by position in the install sequence:

```
ops  1-10: 3.10 s
ops 11-20: 0.70 s
ops 21-30: 3.70 s
ops 31-40: 7.70 s
ops 41-50: 4.70 s
ops 51-60: 7.00 s
ops 61-70: 7.10 s
ops 71-80: 6.80 s
```

Early ops average 3.1 s, late ops 6.8–7.1 s — roughly a doubling across 80 types. This is
consistent with `validate_state` rescanning a growing catalog on every mutation. It is not a clean
isolation: the boot interleaves two install passes and other engine work, and the 11–20 dip is a
run of trivially small types. The controlled version of this measurement belongs in a frontend
benchmark, not in an embedder's log.

### 5.4 CPU profile

`sample <pid> 8` on the live process. 6968 samples per thread. Every tokio worker was parked;
process CPU was ~0%. All work is on two threads.

**`tessera-turso` connection worker** — 6153 samples, all under `TursoGraphStore::upsert_type`:

```
6153  upsert_type → sql::transaction
4051  └─ store::node::upsert_node → GraphConnection::execute
3895     └─ mutation::execute_cypher_mutation
2500        ├─ transaction::in_write_transaction
            │    └─ SemanticConstraintSnapshot::validate_state
            │         └─ catalog::query_rows
            │              └─ Connection::prepare_with_origin → compile_cmd → translate
1788        │                   ├─ translate_select → emit_select_plan → emit_program
            │                   │    → emit_program_for_select → emit_query → OpenLoop::emit
            │                   │    → SeekEmitter::emit → encode_seek_keys_for_custom_types
 712        │                   └─ (other translate branches)
1395        └─ mutation::execute_bound
1047           ├─ execute_operation → insert_node → insert_entity
 434           │    ├─ run_rows → prepare_with_origin → compile_cmd → translate_insert
               │    │    → trigger_exec::fire_trigger → execute_trigger_commands
               │    │    → translate_update → prepare_and_optimize_update_plan
               │    │    → UpdatePlan::build_read_scope_tables
               │    │    → JoinedTable::clone → String::clone → _platform_memmove  (433)
 421           │    ├─ run_rows → Statement::drop → reset_internal → ProgramState::reset
               │    │    → RawTable<TempFile>::clear → TempDir::drop → remove_dir_all
               │    │       openat (262), unlinkat (159)
 192           │    └─ run_rows → run_collect_rows → step → op_open_ephemeral
               │         → TempFile::new → tempfile::tempdir → mkdir  (192)
 348           └─ execute_operation → record_node_labels → run_ignore → run_ignore_rows
                    → step → op_open_ephemeral → TempFile::new → UnixIO::open_file
```

Note what is **not** there: no meaningful time in the two statements that do the actual work.
The profile is compilation and filesystem churn.

**`foedus-turso-wal` thread** — 847 working samples of 6968 (rest idle in `recv`):

```
846  WalStore::append → Connection::execute → Statement::__step → Program::step
       → op_auto_commit → commit_txn → step_end_write_txn → Pager::commit_tx
841    → commit_wal → commit_wal_inner → WalFile::sync → UnixFile::sync → fsync
```

One `fsync` per frame. This is an embedder-side batching question as much as a turso one, and is
listed here for completeness rather than as a turso defect.

## 6. Root causes

### 6.1 `validate_state` re-validates the whole graph on every mutation — backlog item 3

`graph/frontend/src/semantic_constraints.rs:347-353`:

```rust
for constraint in &self.property_constraints { … }
for key in &self.keys { … }
for cardinality in &self.cardinalities { … }
```

Per mutation, one full scan of the source table per constraint, per key, per cardinality —
including rows the statement never touched. This is the largest single cost (40.6%) and the source
of the growth in §5.3.

### 6.2 No prepared-statement cache — backlog item 2

`graph/frontend/src/catalog.rs:1315`:

```rust
pub(crate) fn query_rows(connection: &Arc<Connection>, sql: &str) -> Result<Vec<Vec<Value>>, CatalogError> {
    Ok(connection.prepare(sql)?.run_collect_rows()?)
}
```

Uncached — parse, plan and codegen on every call, for a small fixed set of SQL strings with
`graph_id`/name interpolated in. 1788 of the 2500 validation samples are in query *planning*
(`emit_program_for_select` and below), not execution. §6.1 and §6.2 compound: the quadratic scan
count multiplies an already expensive per-scan constant.

### 6.3 Generation triggers still installed — backlog item 5

`graph/frontend/src/catalog.rs:689` calls `install_generation_triggers`. Compiling the AFTER-DML
trigger body on every insert costs 434 samples (7.1%), of which 433 are `String::clone` inside
`UpdatePlan::build_read_scope_tables` cloning `JoinedTable`.

The core primitive that would let these go away landed in `01cf068b6` / `9038714e9`. The frontend
has not been wired to it.

At the time of writing there is uncommitted work in the checkout (`core/connection.rs`,
`graph/frontend/src/catalog.rs`, `graph/frontend/src/snapshot.rs`,
`tests/integration/table_change_token.rs`) that consumes `table_change_token` for
traversal-snapshot invalidation. That is a different consumer from the one measured here:
`install_generation_triggers` is still called, so the trigger-compilation cost in this profile
remains. Measurements in this report are against committed `5cef48f` and do not include that work.

### 6.4 NEW — `op_open_ephemeral` creates a real temporary *directory* per ephemeral table

Not in `PERFORMANCE_BACKLOG.md`. `core/io/mod.rs:299-313`:

```rust
impl TempFile {
    pub fn new(io: &Arc<dyn IO>) -> Result<Self> {
        let temp_dir = tempfile::tempdir()…;                       // mkdir
        let chunk_file_path = temp_dir.as_ref().join("tursodb_temp_file");
        let chunk_file = io.open_file(chunk_file_path_str, OpenFlags::Create, false)?;   // openat
        …
    }
}
```

and `TempDir::drop` runs `remove_dir_all` (recursive `openat` + `unlinkat`). Every
`op_open_ephemeral` therefore costs a `mkdir`, an `open`, and a recursive directory teardown.
Measured at **961 samples, 15.6%** of the working thread across three call sites
(`insert_entity` open, `insert_entity` drop, `record_node_labels` open).

An ephemeral table that holds a handful of rows should not touch the filesystem at all. Options,
in increasing order of change: (a) one process-wide temp directory created once and reused, so the
per-statement cost is one `open`/`unlink` instead of a directory lifecycle; (b) keep small
ephemeral tables in memory and spill to a file only past a threshold.

This is a core change, and it is independent of items 1–5 — it will still be there after the
frontend work lands.

### 6.5 Mutations parse Cypher twice — backlog item 4

Present in this path (`GraphConnection::execute` parses, then `execute_cypher_mutation` parses
again) but not separately attributable in this profile. Listed for completeness.

## 7. What I believe needs to be done

Ordered by measured leverage on this workload. Status as of branch
`perf/graph-bootstrap`:

| Item | Status |
|---|---|
| 1. Scope `validate_state` — by source table | **done**, `379bdf9e0` |
| 1. Scope `validate_state` — by written row | open, the larger half |
| 2. Cache or parameterise the catalog queries | **done**, `df549ed80` |
| 3. Stop `op_open_ephemeral` creating a temp directory | **done**, `74e3b4240` |
| 4. Wire the frontend to per-table change tokens | **done**, `1c333a51c`, merged in |
| 5. Cache bound mutation plans | open |

What landed and what it is measured to be worth:

- **Source scoping.** A mutation now validates only constraints reading a
  source table it writes. Steady-state `CREATE` on one type compiles 5
  statements at 2, 4 and 16 registered types, against `5 + 2 * (types - 1)`
  before — 35 at 16 types. This removes the term that grows with **ontology
  size**. It does not remove the term that grows with **row count**: a
  constraint in scope still scans its whole source table, so a workload where
  every entry lands in one table still sees §5.3's growth.
- **Change tokens.** Landed on `feature/graph-frontend` by other work and
  merged into this branch, so the numbers above were re-measured on the merged
  tree rather than carried over. The generation triggers are gone; snapshot
  invalidation now reads a per-table change token instead of a trigger-bumped
  counter.
- **Prepared statements.** The SQL a mutation runs around its writes does not
  depend on the row being written, so the session now holds it compiled, keyed
  on the exact text. A steady-state `CREATE` with a required and a unique
  property compiles 2 statements instead of 5: the catalog freshness probe, the
  required-property check and the uniqueness check are now steps of an
  already-compiled program. Bypassing the cache puts all 3 back, twice over two
  mutations. This is the constant that item 1 multiplies, so it is worth the
  same proportion whatever item 1 ends up costing.
- **Temp files.** `TempFile` no longer creates a directory per ephemeral table.
  Measured on the same syscall sequence in isolation: 292 µs → 215 µs per open
  and close, 1.36x. The remainder is the file create and unlink themselves;
  only keeping small ephemeral tables in memory until they spill, as SQLite
  does, removes those.

One behaviour was traded away with source scoping, deliberately: a plain-SQL
write to a mapped table that violates a constraint used to be caught by the next
Cypher mutation on *any* source, and is now caught only by one touching *that*
source. Validation was never a guarantee against out-of-band writes, and the
full pass cost 40.6% of the working thread.

The remaining half of item 1 is the one that matters for a single-table
ontology: restrict each validation query to the identities the statement wrote.
`execute_bound` has them, but they have to be threaded through roughly fifteen
operation branches plus the closed-`CREATE` fast path, and uniqueness, keys and
cardinality each need their own restriction that stays sound (for uniqueness,
"only values the written rows carry"; for cardinality, "only nodes the statement
wrote or repointed"). Getting one branch wrong silently stops validating, which
is how the FOREACH hole in the source-scoping change showed up.

**1. Scope `validate_state` to the rows the statement wrote, or defer it to commit.**
`execute_bound` already has the affected identities from `RETURNING id`. Adding
`entity.<identity> IN (…)` turns O(constraints × N²) into O(constraints × N). The stronger
version — defer validation to commit inside an explicit transaction — collapses an N-statement
bootstrap into a single validation pass, and is what this workload actually wants: foedus installs
the whole ontology as one logical unit. Backlog item 3 already proposes both; this is the item to
do first.

**2. Cache or parameterise the catalog queries.** Done in `df549ed80`.
The session holds prepared statements keyed on SQL text. Parameterising on
`graph_id` was not needed: the text already repeats byte-for-byte across
mutations, because every one of these queries is built from the resolved
constraint rather than from the row. What remains open here is the one query
whose text does vary — the bound mutation's own write — which is item 5.

**3. Stop `op_open_ephemeral` from creating a temp directory per statement.**
Core change, `core/io/mod.rs`. 15.6% of the working thread, and unlike items 1 and 2 it scales with
statement count for every workload, not just constraint-heavy ones. Reuse one temp directory for
the process, or keep small ephemeral tables in memory.

**4. Wire the frontend to the per-table change tokens and drop `install_generation_triggers`.**
The primitive exists as of `01cf068b6`. This removes the trigger-compilation cost (7.1%), removes
the second B-tree write per row, and lets the three core carve-outs in `translate/update.rs`,
`translate/index.rs` and `translate/trigger.rs` go.

**5. Cache bound mutation plans (backlog item 4).**
Keyed on (source, schema generation). Lowest measured leverage here, but it is the same
bootstrap-loop shape and becomes visible once 1–3 land.

### Suggested acceptance criterion

A frontend benchmark that installs 100 node types with constraints in a loop, asserting both:

- total wall time, and
- **that per-type cost does not grow with the number of installed types** — i.e. the mean of the
  last 10 is within a small factor of the mean of the first 10.

The second assertion is the one that would have caught this. Absolute time alone would not: at the
118-entity scale the original report mentioned, the quadratic term is survivable; the failure only
becomes total at scale plus a hard install timeout.

For the embedding product to install successfully, an empty-store ontology install of ~55 types
needs to finish in **well under 60 s** including engine start; today it is >16 minutes.

## 8. Notes on scope

Two costs in this profile are the embedder's problem, not turso's, and are being tracked
separately:

- foedus issues one `LoadSchemaEntry` action per type, serially, each its own transaction and its
  own WAL `fsync`. Batching the ontology install into one transaction would help regardless of what
  turso does — and is the change that makes "defer validation to commit" (item 1) pay off fully.
- limen installs the graph schema twice per boot (an 18-type built-in pass, then the full 51-type
  pass), paying the install cost twice.

Neither explains the per-mutation costs in §6.
