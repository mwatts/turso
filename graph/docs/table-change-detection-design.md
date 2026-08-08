# Detecting table changes without triggers

Design for the core primitive that lets the graph frontend drop its AFTER-DML
generation triggers.

Status: done. `Connection::table_change_token` landed first, then snapshot
invalidation moved onto it while the triggers stayed installed so the two
answers could be compared, and finally the triggers were deleted. Everything
below is left as written at proposal time, so it reads in the future tense and
in a few places names things that ended up slightly different -- what actually
landed, including where it diverged, is at the bottom under "What landed".

## What the triggers did

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

Measured against a table carrying the same three AFTER-DML triggers the
frontend installs, counting WAL frames so the numbers do not depend on build
profile:

| Workload | Without triggers | With triggers | |
| --- | --- | --- | --- |
| 100 single-row inserts | 100 frames | 200 frames | **2.00x**, +1 page per row |
| 400 single-row inserts | 403 frames | 803 frames | **1.99x**, +1 page per row |
| 20,000 rows in one transaction | 69 frames | 70 frames | +1 page total |
| Bytecode per inserted row | 20 instructions | 46 instructions | **2.3x** |

The cost lands on transaction count, not row count: the counter page is written
once per transaction however many rows it carries. A bulk load barely notices.
A row-at-a-time writer — the shape most application traffic takes — doubles its
durable writes, and every one of those writes lands on the same page.

**Core carve-outs.** The generations table is a protected internal name, so
core needed an explicit exception for the trigger body:

- `core/translate/update.rs` — `validate_update` took
  `is_internal_graph_trigger` and skipped the protected-table check when the
  target was `TURSO_GRAPH_GENERATIONS_TABLE_NAME` **and** the firing trigger's
  name started with `TURSO_GRAPH_GENERATION_TRIGGER_PREFIX`. **Removed.**
- `core/schema.rs` — `TURSO_GRAPH_GENERATION_TRIGGER_PREFIX` existed only for
  that check. **Removed from core**; the graph frontend keeps a private copy so
  it can recognize and drop what an older build installed.
- `core/translate/trigger.rs` — `translate_create_trigger` /
  `translate_drop_trigger` take `internal: bool` so users cannot forge or drop
  a trigger under the graph prefix. **Kept**: it guards the whole
  `TURSO_GRAPH_CATALOG_PREFIX` name space, and the drop pass needs it.

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

**3. Statement end — collect. Transaction commit — publish.**

These have to be two separate steps, and picking the wrong boundary for the
second one silently breaks property 2.

The obvious hook, where a statement's changes are published to the connection,
is the wrong one. `Insn::ResetCount` is only ever emitted by trigger
subprograms (`core/translate/trigger_exec.rs:619`), and inside an interactive
transaction `halt` publishes the statement's change count
(`core/vdbe/execute.rs:3614`) without committing anything. Bumping there means
`BEGIN; INSERT; ROLLBACK` advances the counter — exactly what
`source_write_rollback_restores_the_generation`
(`graph/frontend/src/catalog.rs:2079`) pins today.

The real boundary is `Program::commit_txn` (`core/vdbe/mod.rs:2099`), which
takes an explicit `rollback: bool` and which both `op_halt` and
`op_auto_commit` funnel through. So:

- At statement end, when the statement actually changed rows
  (`n_change != 0`), union `written_roots` into a pending set on the
  connection. Trigger and FK-action subprograms union into the same set, which
  is what makes a user trigger on table A that writes table B count for B.
- In `commit_txn`, when `!rollback` and the commit completes, bump the
  `Database` counter for every root in the pending set and clear it. On
  rollback, clear without bumping. Both the WAL and MVCC commit paths route
  through here, so neither needs its own hook.

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

**5. The token has to cover DDL too.**

A root page is not a stable name for a table. `DROP TABLE` does not open a
write cursor on the table it drops — it emits `Insn::Destroy`
(`core/translate/schema.rs:2057`) — so it never bumps the counter, and the
freed root page can be handed straight back to the next `CREATE TABLE`. Drop a
mapped table and recreate it and the counter reads back exactly where it was,
while the rows are gone. That is a stale snapshot, not an extra reload: the
only unsafe failure in this whole design. `VACUUM` and `ALTER TABLE` move root
pages the same way.

So `table_change_token` mixes three things, not one: the per-root counter, the
schema cookie (which every DDL statement moves), and a process-global
"unknown write" epoch for the rare `OpenWrite` whose root page is a register
rather than a literal — `core/translate/alter.rs` and `core/translate/index.rs`
emit those, and rather than resolve them the epoch just invalidates every
table at once. All three are conservative in the same direction: they can only
cause extra reloads.

### Graph frontend changes

- Delete `install_generation_triggers` and its call site, and replace it with a
  pass that drops any triggers a previous version already installed. Deleting
  the install alone leaves existing databases paying the full trigger cost
  forever, for a column nothing reads any more. This is why
  `translate_drop_trigger`'s `internal` parameter has to survive the carve-out
  cleanup even though `translate_create_trigger`'s does not.
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
  `core/schema.rs`. `users_cannot_mutate_catalog_or_forge_internal_generation_triggers`
  still passes afterwards: the generations table stays a protected name, so a
  forged trigger body cannot write it whatever the trigger is called.

### Where the properties land

| | Triggers today | Proposal |
|---|---|---|
| Any writer counts | yes | yes, same process |
| Transactional | yes | yes — bumped at commit |
| Cross-connection | yes | yes — counters live on `Database` |
| Cross-process | yes | **no** — returns `None`, caller reloads every time |
| Survives DDL | yes | yes — via the schema cookie, not the counter |
| Exact | yes | **near** — a statement that opens a write cursor on two tables and changes rows in one bumps both; any DDL invalidates every table |
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
   covering: bump on commit; **no bump on `BEGIN; INSERT; ROLLBACK`**; no bump
   when a statement changes nothing; visibility from a second connection; the
   token moving across a `DROP` plus `CREATE` that writes no rows; a user
   trigger on one table counting for the table it writes; and `None` under
   multiprocess WAL.
2. Switch the graph frontend's snapshot invalidation to the token, keeping the
   triggers installed but unused, so the two can be compared on the corpus.
   Measure the write amplification here too — the cost argument above deserves
   a number before the triggers go.
3. Delete the triggers, drop the ones already installed, and remove the core
   carve-outs.

Step 2 is what makes this safe to land, but the invariant to assert is
one-directional, not equality: the token must move whenever the trigger
generation moved. The reverse does not hold and should not — the token is
per-table where the counter is per-graph, so a write to one graph's table
leaves the other tables' tokens alone.

## What landed

1. `Connection::table_change_token` plus the per-root counters, with the core
   tests listed above.
2. Snapshot invalidation switched to the token, triggers still installed, the
   one-directional invariant asserted on a release corpus run of 10,242
   records. It caught two real gaps in the token -- a transaction not seeing
   its own uncommitted writes, and savepoint rollback leaving the token
   unmoved -- which is exactly what that step existed to do.
3. Triggers deleted. `load_registered_graph` drops any an older build left
   behind, because their bodies update a protected table that core no longer
   makes an exception for, so leaving one installed would fail the next write
   to a mapped table.

Two things came out different from the proposal above.

The stored `generation` column survives. The plan said it would stop being
stored and become derived; instead the derived value moved into a new
`RegisteredGraph::derived_generation`, which is what snapshots compare, and the
column stayed. Nothing bumps it per row any more, but `bump_semantic_generation`
still moves it on catalog changes, and sessions on a database predating
`schema_generation` watch it to decide when to reload their catalog.

`graph_generation()` kept its signature, but it is no longer the staleness
probe. It reads the stored column, which now moves only on catalog changes, so
the callers that care about source writes -- snapshots, and the tests that
cover them -- ask `load_registered_graph` for `derived_generation` instead.
There are two signals now, and one accessor cannot stand for both.
