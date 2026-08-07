# Detecting table changes without triggers

Design for the core primitive that would let the graph frontend drop its
AFTER-DML generation triggers. Status: proposal, nothing implemented.

## What the triggers do today

`install_generation_triggers` (`graph/frontend/src/catalog.rs`) installs, for
every mapped node-source and relationship-source table, three triggers:

```sql
CREATE TRIGGER __turso_internal_graph_gen_<graph>_<event>_<hash>
AFTER INSERT|UPDATE|DELETE ON <source table>
BEGIN UPDATE __turso_internal_graph_generations
      SET generation = generation + 1 WHERE graph_id = <graph>; END
```

They answer one question: *has anything written to a table this graph maps
since I last looked?* Traversal snapshots rebuild when the answer is yes.

They get four properties right, and any replacement has to keep them:

1. **Any writer counts.** Plain SQL through any connection, not just Cypher.
2. **Transactional.** A rolled-back write leaves the counter where it was —
   `source_write_rollback_restores_the_generation` in
   `graph/frontend/src/catalog.rs` pins this.
3. **Cross-connection.** Two connections on one database see each other.
4. **Exact.** A statement that writes no row does not advance the counter.

## What they cost

**Write amplification.** Every row written to a mapped table also updates one
row in the generations table — a B-tree descent and a page dirty per row, on
the single hottest row in the database. Under MVCC that same row becomes a
write-write conflict point between otherwise independent writers.

**Core carve-outs.** The generations table is a protected internal name, so
core needs an explicit exception for the trigger body:

- `core/translate/update.rs` — `validate_update` takes
  `is_internal_graph_trigger` and skips the protected-table check when the
  target is `TURSO_GRAPH_GENERATIONS_TABLE_NAME` **and** the firing trigger's
  name starts with `TURSO_GRAPH_GENERATION_TRIGGER_PREFIX`.
- `core/translate/trigger.rs` — `translate_create_trigger` /
  `translate_drop_trigger` take `internal: bool` so users cannot forge or drop
  a trigger under the graph prefix.
- `core/schema.rs` — `TURSO_GRAPH_GENERATION_TRIGGER_PREFIX` exists only for
  those checks.

These are the only graph triggers in the frontend. Removing them removes the
whole `is_internal_graph_trigger` path from core.

## What core already has

Checked against the current branch:

| Primitive | Where | Fits? |
|---|---|---|
| `Connection::total_changes` | `core/connection.rs:2773` | No — per connection, not per table, invisible across connections |
| Schema cookie | `core/connection.rs`, header `schema_cookie` | No — DDL only; catalog rows are ordinary DML |
| Header `change_counter` | `core/storage/sqlite3_ondisk.rs:338` | No — written to the file but not maintained as a live signal |
| `PRAGMA data_version` | absent | Would be coarse anyway; see rejected alternatives |
| CDC (`capture_data_changes`) | `core/connection.rs:391` | Too heavy — materialises per-row change records |

Two things core *does* already have that make the proposal small:

- `Insn::OpenWrite { cursor_id, root_page, db }` (`core/vdbe/insn.rs:1322`) —
  the set of tables a statement may write is known at translate time, before
  a single row moves.
- `PreparedProgram` (`core/vdbe/mod.rs:1519`) already carries
  `cursor_ref: Vec<(Option<CursorKey>, CursorType)>` and a `readonly` flag, so
  the program knows which B-trees it opens for write.

## Proposal: per-root change counters on `Database`

Add an in-memory, process-local map from `(database index, root page)` to a
monotonic counter, advanced once per committing statement.

### Core changes

**1. `core/database.rs` — the counters.**

```rust
/// Advances once per committed statement that wrote to the B-tree rooted at
/// this page. In-memory and process-local: a fresh process starts from zero,
/// so callers must treat "no prior token" as "assume changed".
table_change_counters: Mutex<HashMap<(usize, i64), u64>>,
```

**2. `core/vdbe/builder.rs` — collect the write set at translate time.**

`ProgramBuilder` already sees every `OpenWrite` it emits. Record
`(db, root_page)` into a small `Vec` and hand it to `PreparedProgram` as
`written_roots: Box<[(usize, i64)]>`. No runtime cost, no per-row cost.

**3. Commit path — bump.**

Where the statement's changes are already published to the connection
(`op_reset_count` / the halt-and-commit path in `core/vdbe/execute.rs`), and
only when the statement actually changed rows (`n_change != 0`), bump each
entry in `written_roots`. Bumping at commit, not at write, gives property 2
for free: a rolled-back statement never reaches this point.

**4. `core/connection.rs` — read it.**

```rust
/// Token for the current state of `table`. Compare two tokens from the same
/// process to learn whether the table may have changed between them.
/// `None` means "cannot tell" — an unknown table, or another process may be
/// writing this database.
pub fn table_change_token(&self, table: &str) -> Option<u64>;
```

Returns `None` when `Database::shared_wal_coordination()` is `Some`
(`core/database.rs:2617`), because a counter in this process cannot observe
another process's commits. `maybe_reparse_schema` (`core/connection.rs:1278`)
already tests the same condition before trusting cached schema state; it
recovers by dropping the page cache, which has no equivalent here — a counter
another process never touched cannot be repaired, only distrusted.

### Graph frontend changes

- Delete `install_generation_triggers` and its call site.
- `RegisteredGraph.generation` stops being a stored column and becomes derived:
  the tokens for the graph's mapped source tables, plus the persisted
  `schema_generation` from the catalog split already landed. Snapshot
  classification (`graph/frontend/src/snapshot.rs`) compares the derived value
  exactly as it compares the stored one now.
- `graph_generation()` keeps its signature; its value stops being comparable
  across process restarts, which snapshots never relied on because
  `SnapshotStore` is in-memory and session-scoped.
- Drop the `is_internal_graph_trigger` parameter from
  `core/translate/update.rs` and the graph-trigger prefix from
  `core/schema.rs`.

### Where the properties land

| | Triggers today | Proposal |
|---|---|---|
| Any writer counts | yes | yes, same process |
| Transactional | yes | yes — bumped at commit |
| Cross-connection | yes | yes — counters live on `Database` |
| Cross-process | yes | **no** — returns `None`, caller reloads every time |
| Exact | yes | **near** — a statement that opens a write cursor on two tables and changes rows in one bumps both |
| Survives restart | yes (persisted) | no — first read after start has no prior token |
| Cost per row written | one B-tree update | none |

The two losses are both safe: they cause extra reloads, never a stale
catalog or a stale snapshot. Multiprocess WAL is experimental and the graph
frontend is embedded-only, so the cross-process path degrades to exactly
today's behaviour rather than breaking.

The inexactness is worth a second look before implementing. If it turns out to
matter, the same hook can bump only roots that the cursor actually wrote, by
setting a per-cursor dirty bit in the write instructions instead of trusting
the open. That costs one branch per row instead of one B-tree update per row —
still far cheaper than the trigger — and can be added later without changing
the API.

## Rejected alternatives

**`PRAGMA data_version` (SQLite-compatible).** One counter for the whole
database, bumped whenever another connection commits. Cheap to add and worth
having for SQLite compatibility on its own merits, but useless for this
problem: in a write-heavy workload it changes on every commit, so the graph
frontend would reload as often as it does today. It only helps read-mostly
sessions.

**Persist per-root counters in the database header.** Exact and durable, but a
file-format change on a format whose whole selling point is SQLite
compatibility. Not worth it for a cache-invalidation hint.

**Keep triggers, make them cheaper.** There is no cheap version — a trigger
body that records anything is a row write.

**Derive it from the WAL frame count.** Monotonic, cross-connection, already
maintained. But it is database-wide, so it has the same problem as
`data_version`, and it says nothing under MVCC.

**MVCC logical log / CDC.** Both already know exactly which tables changed,
but both are opt-in, heavier, and unavailable on the default path.

## Suggested order

1. Land the counters and `table_change_token` behind no flag, with core tests
   covering: bump on commit, no bump on rollback, no bump when a statement
   changes nothing, visibility from a second connection, and `None` under
   multiprocess WAL.
2. Switch the graph frontend's snapshot invalidation to the token, keeping the
   triggers installed but unused, so the two can be compared on the corpus.
3. Delete the triggers and the three core carve-outs.

Step 2 is what makes this safe to land: the old and new signals can be
asserted equal across the whole conformance corpus before anything is
removed.
