# Foedus graph schema baseline

Date: 2026-08-05

## Scope

This report describes `/Users/markwatts/tmp/limen-db/foedus.graph.turso`.
The review used `tursodb` in read-only mode. It enabled the custom type, view,
trigger, and custom index features that the database schema needs.

The DDL archive is `/Users/markwatts/tmp/limen-db/foedus.graph.schema.sql`, next
to the source database. The archive contains all SQL from `sqlite_schema`,
except SQLite-owned objects. It contains tables, indexes, and triggers. It does
not contain row data and is not a database backup.

| Item | Value |
|---|---:|
| Database SHA-256 | `4d257ae9615b8f828aa4e283491f6dd4b1fbeb92c66173ac2c354a6e1a2de8dc` |
| DDL archive SHA-256 | `aa04ed715b4602dd73ef5d8c5b30cf936eb50ce23beafdeadf10ac4d20238f18` |
| Database size | 4,153,344 bytes |
| Page size | 4,096 bytes |
| Page count | 1,014 |
| Free pages | 0 |
| Graph | `foedus_ontology` |
| Tessera catalog version | 3 |
| Tessera seal state | `current` |

The Turso checkout was at `dac286c42a17500b29f3f4ca81b2b9fe8486784f`.
The Foedus checkout was based on `13a8a8ae89a02e26b0940deabf2e255967441d05`.
The Foedus worktree had uncommitted changes during this review.

## Object inventory

The schema has 1,223 named objects. This total excludes names that start with
`sqlite_`.

| Object type | Count |
|---|---:|
| Tables | 179 |
| Indexes | 684 |
| Triggers | 360 |

The 179 tables have these roles.

| Table role | Count |
|---|---:|
| Node source tables | 51 |
| Reified relation node tables | 1 |
| Binary relationship tables | 2 |
| Reified role edge tables | 66 |
| Foedus schema control tables | 1 |
| Foedus search and projection tables | 3 |
| Turso graph catalog tables | 21 |
| Turso FTS directory tables | 34 |

The graph catalog registers 120 sources. It registers 52 node-like sources and
68 relationship sources. Turso creates three generation triggers for each
source. This rule accounts for all 360 triggers.

The application owns 124 tables and 408 indexes. The graph frontend owns 55
tables, 276 indexes, and all 360 triggers.

## Data occupancy

Only 8 of the 120 graph sources contain rows. The sources contain 147 rows in
total. All 68 relationship sources are empty.

| Source | Rows |
|---|---:|
| `foedus.ActionInvocation` | 21 |
| `foedus.DerivationRecord` | 21 |
| `foedus.Identity` | 1 |
| `foedus.Marking` | 5 |
| `foedus_system.Config` | 1 |
| `foedus_system.RegisteredEntity` | 21 |
| `foedus_system.SchemaEntry` | 62 |
| `system.SourceManifest` | 15 |

The projection cursor table contains 4 rows. The vector tables contain no rows.
The other 112 graph sources contain no rows.

## Table shapes

### Relationship tables

The 66 `core.Mentions` edge tables have the exact same shape:

```sql
(
    id        INTEGER PRIMARY KEY,
    object_id TEXT,
    start_id  INTEGER NOT NULL,
    end_id    INTEGER NOT NULL
) STRICT
```

Each table has three indexes. One index covers each endpoint. A unique index
covers `object_id`. These empty tables add 198 application indexes and 198
generation triggers.

Foedus creates this family because both `core.Mentions` roles target the open
`core.Referenceable` fragment. That fragment has 33 member types. The adapter
creates one edge table for each role and each member: `2 * 33 = 66`.

The two other relationship tables use the same four base columns. The
`foedus.DerivationEdge` table adds four properties. The `projects.LinksTo`
table adds no properties.

### Node tables

All 52 node-like tables use this base shape:

```sql
id        INTEGER PRIMARY KEY,
object_id TEXT
```

Each type then adds its typed property columns. The property count ranges from
0 to 15. The distribution is:

| Property columns | Node types |
|---:|---:|
| 0 | 1 |
| 1 | 3 |
| 2 | 7 |
| 3 | 4 |
| 4 | 8 |
| 5 | 8 |
| 6 | 7 |
| 7 | 3 |
| 8 | 3 |
| 9 | 1 |
| 10 | 2 |
| 11 | 2 |
| 12 | 1 |
| 14 | 1 |
| 15 | 1 |

The common properties come from flattened fragments. The most common fields
are `name` on 22 types, `created` on 17 types, and `updated` on 17 types.
`description` and `status` each occur on 8 types.

Only three node-type pairs have the same complete property set:

| Property set | Types |
|---|---|
| `address, name` | `core.Contact`, `email.Account` |
| `description, name, sync_etag, sync_modified, url` | `podcasts.Podcast`, `rss.Feed` |
| `name` | `core.Tag`, `email.Mailbox` |

This result does not support one generic node table. Most node types have a
different property set. A generic table would move type checks into JSON or
application code.

### Full-text search tables

The schema creates 34 graph FTS indexes. Each index adds one FTS directory
table and one backing-tree index. Most indexed node sources contain no rows.

At the time of capture, `PRAGMA quick_check` reported incorrect entry counts
for these two backing-tree indexes:

- `__turso_internal_fts_dir___turso_graph_fts_1_31e67aaf7ecd05a2_key`
- `__turso_internal_fts_dir___turso_graph_fts_1_89f77d4a798a5459_key`

They belong to the FTS indexes for `foedus.Identity.display_name` and
`system.SourceManifest.title`. Those source tables contain 1 and 15 indexed
rows. Their FTS directory tables contain no rows.

The cause was an integrity-check defect. The check counted custom-index backing
B-trees as ordinary SQL indexes, even though their entries do not map one to
one to directory-table rows. The graph frontend fix keeps the structural
B-tree check and skips only the invalid ordinary-index row-count comparison.
With that fix, the archived database returns `ok` from `PRAGMA quick_check` in
read-only mode. No index or database content was changed or rebuilt.

### Duplicate access indexes

The schema has 154 pairs with the same table, uniqueness flag, and indexed
column list. Twenty-two are intentional: one member is a normal B-tree and the
other is a Turso FTS index method. They serve different query shapes and are
not duplicates.

The other 132 pairs are ordinary B-tree indexes on `start_id` or `end_id` in
the 66 `core.Mentions` edge tables. The schema producer creates an endpoint
index, then graph registration creates another index with the same leading
column. A rebuilt copy of this schema does not need the second 132 indexes.

## Turso follow-up implementation

The reusable frontend work is implemented on `feature/graph-frontend`. The
example database and its producer were not changed.

- Graph catalog version 5 can register a single-valued role with an endpoint
  source discriminator and an allowed set of node sources.
- Existing `register_graph` callers keep the fixed-source registration API.
  Compact schemas use `register_graph_with_polymorphic_roles` as an additive
  registration surface.
- Catalog loading, snapshot construction, role joins, relationship writes,
  role updates, detach-delete checks, and endpoint indexes carry the source
  discriminator.
- Graph registration reuses an existing full B-tree index when its leading
  columns cover a role lookup. It does not treat a partial index or a Turso
  custom index method as an ordinary B-tree. This would avoid all 132 duplicate
  endpoint indexes in a fresh copy of the baseline schema.
- `GraphConnection::inspect_schema` returns readable semantic and physical
  names, role mappings, row counts, and FTS definitions.
- A regression uses one relationship table for two polymorphic roles and
  proves that equal node identities in different source tables remain
  distinct.
- FTS integrity regression coverage proves that custom-index backing B-trees
  pass both `quick_check` and `integrity_check` without weakening structural
  checks.

## Recommendations

### 1. Replace open-role expansion with polymorphic role storage

This change gives the largest safe reduction. Keep the `core.Mentions` relation
node table. Replace the 66 member-specific edge tables with two role tables:

```sql
CREATE TABLE mention_role_source (
    id               INTEGER PRIMARY KEY,
    object_id        TEXT UNIQUE,
    mention_id       INTEGER NOT NULL,
    target_source_id INTEGER NOT NULL,
    target_node_id   INTEGER NOT NULL
) STRICT;

CREATE TABLE mention_role_target (
    id               INTEGER PRIMARY KEY,
    object_id        TEXT UNIQUE,
    mention_id       INTEGER NOT NULL,
    target_source_id INTEGER NOT NULL,
    target_node_id   INTEGER NOT NULL
) STRICT;
```

Index `(target_source_id, target_node_id)` and `mention_id` on each table.
Validate `target_source_id` against the fragment membership catalog.

This layout reduces the application table count from 124 to 60. It reduces the
registered graph source count from 120 to 56. It also removes about 192 indexes
and 192 generation triggers.

At capture time, the graph frontend could not register a role whose endpoint
source changed per row. Graph catalog version 5 now supports that physical
shape. A schema producer still has to adopt compact storage before an instance
will use fewer tables. Do not hide the discriminator in `object_id`.

### 2. Keep one typed table for each concrete node type

Do not merge all node types into one JSON property table. The current tables
keep STRICT types, property indexes, unique constraints, and direct FTS column
maps. Most node types do not have the same property set.

Do not split `Named`, `Timestamped`, or `Referenceable` into separate property
tables. Such normalization adds tables and joins. The repeated columns are a
small and useful cost of typed reads.

If SQL access must be easier, add generated read-only views with readable names
and column aliases. Keep the encoded physical names internal. Keep all domain
writes on the Foedus action path.

### 3. Create graph FTS indexes only when a type needs search

Do not create all 34 FTS indexes during an empty schema install. Create an index
when a searchable type becomes active or receives its first searchable row.
Record the desired index in the schema catalog before the physical build.

This policy can remove most of the 34 FTS directory tables from a new empty
database. It also narrows integrity and migration work. Add a test that inserts
rows before and after index creation and compares the search result.

The `quick_check` defect is now covered by a small FTS regression. Do not
rebuild the indexes in this archived database; it is preserved as captured.

### 4. Add installation profiles only if startup schema size is a product issue

This database installs every Limen domain, but 112 graph sources are empty.
A profile can install only the core types and enabled modules. A later schema
change can add a module.

This option reduces empty tables without a graph frontend change. It also makes
database contents depend on enabled modules. Use it only if Foedus can preserve
one canonical ontology fingerprint across profiles or can state profile changes
in the fingerprint.

### 5. Add a readable schema inspection surface

The physical names use hex encoding. This protects names and makes lowering
deterministic, but it makes direct inspection difficult.

Add one read-only inspection command or view that joins semantic names to
physical tables, columns, roles, row counts, FTS indexes, and integrity status.
Use the semantic name as the main label. Show the physical name as detail.

## Additional Turso-native options

These options use Turso core primitives. They are separate from the graph
frontend, which is custom to this branch.

### A. Replace per-source generation triggers with a core write epoch

This is the next largest schema-clarity opportunity. The database has 360
generation triggers: one each for insert, update, and delete on 120 sources.
Add a core primitive that increments a transaction-aware epoch when a
registered source table changes. The graph snapshot can compare that epoch
instead of installing SQL triggers.

Turso CDC proves that the core can observe changes while respecting commit and
rollback boundaries, but the current CDC surface writes a change log and is
enabled per connection. It is not a direct replacement. Reuse its transaction
plumbing or add a lighter registered-table epoch. Do not replace the triggers
with action-path bookkeeping because raw SQL writes must also invalidate the
snapshot. See the [Turso PRAGMA and CDC reference](https://docs.turso.tech/sql-reference/pragmas).

Potential reduction for this baseline: 360 triggers to zero, with one small
catalog structure in core. Benchmark write cost and verify same-connection,
other-connection, rollback, and reopen behavior before adopting it.

### B. Use composite primary keys for junction and spill tables

The label junction, relationship-type junction, and `Many`-role spill tables
represent sets of pairs or triples. Give each one a composite primary key in
its forward lookup order and keep one reverse index. This rejects duplicate
membership and can remove one explicit forward index per table.

`WITHOUT ROWID` can make the composite primary key the table storage itself,
but it remains experimental in this checkout. Turso documents composite keys
as supported and `WITHOUT ROWID` behind `--experimental-without-rowid`. Use the
primary-key change first; evaluate `WITHOUT ROWID` only with a benchmark and
the feature enabled. See
[Turso CREATE TABLE](https://docs.turso.tech/sql-reference/statements/create-table).

This does not materially change the table count. It reduces index objects,
prevents duplicate role players, and makes the physical intent clearer.

### C. Run `ANALYZE` after schema activation and bulk loads

The graph schema gives the planner hundreds of possible indexes. Statistics
help it choose an index and join order. Run `ANALYZE` after the initial data
load, after a module becomes active, and after large distribution changes. Do
not analyze the empty install and assume those statistics remain useful.
[Turso documents](https://docs.turso.tech/sql-reference/statements/analyze)
that statistics are stored in `sqlite_stat1` and guide index selection, join
order, and scan choice.

This improves query planning. It does not reduce tables or indexes.

### D. Keep FTS indexes source-local, but create and optimize them on demand

The current design already combines multiple searchable columns of one source
into one FTS index where needed. Do not merge FTS across unrelated source
tables: a Turso index method attaches to one base table, and a cross-source
search table would add synchronization work.

Keep lazy creation as recommended above. After a bulk import, run
`OPTIMIZE INDEX` for active FTS indexes. Turso's FTS is a Tantivy-backed custom
index method, not SQLite FTS5; its index is maintained with base-table DML.
See the [Turso FTS reference](https://docs.turso.tech/sql-reference/functions/fts)
and [CREATE INDEX reference](https://docs.turso.tech/sql-reference/statements/create-index).

### E. Use views for clarity; use materialized views only for measured hot reads

`GraphConnection::inspect_schema` is the lowest-object-count inspection
surface. If SQL tools need stable readable names, create connection-local
temporary views. Turso views store no data and can provide a stable interface
over encoded physical names. See
[Turso CREATE VIEW](https://docs.turso.tech/sql-reference/statements/create-view).

Do not generate one persistent view for every node type by default; that trades
table clutter for view clutter. Do not use materialized views as a general
schema facade. Turso maintains them in the same transaction as base-table
changes, so they add stored state and write work. Use them only for a measured
hot aggregation or cross-type read model. See
[Turso materialized views](https://docs.turso.tech/sql-reference/statements/create-materialized-view).

### F. Keep typed node tables; do not replace them with one STRUCT, UNION, or JSON table

Turso custom, STRUCT, UNION, and array types can make individual values richer,
but they do not solve the access-path problem. A single generic node table
would weaken source-specific constraints and complicate ordinary B-tree and FTS
indexing. The baseline's node property sets are too different for this trade.
Use custom types for real value domains, not as a container for the whole node.
See [Turso data types](https://docs.turso.tech/sql-reference/data-types).

### G. Use partial or expression indexes only from query evidence

Partial indexes are useful for a stable sparse predicate such as active tasks
or non-null optional fields. They do not help the dense, `NOT NULL` relationship
endpoint columns in this schema. Expression indexes can help normalized lookup
forms such as `lower(email)`, but only when queries use the same expression.
Turso's [CREATE INDEX reference](https://docs.turso.tech/sql-reference/statements/create-index)
documents both and notes that composite index column order matters.

## Recommended order

1. Done: diagnose and fix the two FTS integrity findings.
2. Done: add the readable schema inspection surface.
3. Done: add polymorphic role registration to the graph frontend.
4. Producer work, outside this repository: use compact role storage when a
   schema producer adopts the new registration surface.
5. Producer policy, outside this repository: create FTS indexes lazily if its
   workload benefits.
6. Consider module profiles only after measurements show a startup or storage problem.

## Acceptance checks for a new baseline

- `PRAGMA quick_check` returns `ok`.
- The DDL archive has a source database checksum.
- `core.Mentions` uses 2 role tables, not 66 member-specific tables.
- Cypher traversals return the same typed endpoints before and after migration.
- Fragment membership rejects an endpoint with the wrong type.
- Reopen, rollback, and schema evolution keep role rows and FTS results correct.
- Direct SQL inspection uses readable views or a supported inspection command.
- Domain writes still use the Foedus action path.
