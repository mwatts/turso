# Using the Graph (Cypher) Frontend

The graph frontend (`turso_graph_frontend`) runs Cypher queries against a
standard Turso database file. Nodes and relationships are ordinary SQL
tables that you *register* as a graph; Cypher compiles down to the same
AST → VDBE pipeline the SQL frontends use, so storage, WAL, and
transactions are all the engine's own. There is no separate graph file
format — graph metadata (labels, relationship types, generation counters)
lives as `__turso_graph_*` tables, indexes, and triggers inside the same
`.db` file as your SQL schema.

> **Status:** experimental, source-only. The graph frontend lives on the
> `feature/graph-frontend` branch (crates `turso_graph_cypher`,
> `turso_graph_ir`, `turso_graph_runtime`, `turso_graph_frontend`,
> `turso_graph_temporal`). It is deliberately decoupled from the Postgres
> frontend — see "Composing frontends" below.

## Quickstart

```rust
use std::sync::Arc;
use turso_graph_frontend::{
    open_database, register_graph, GraphConnection, GraphRegistration,
    NodeSourceRegistration, RelationshipSourceRegistration,
    DatabaseOpts, OpenFlags,
};

// 1. Open a database (graph-cypher dialect, GraphDialect) — same shape as turso_pg::open_database.
let (_io, db) = open_database("app.db", None, OpenFlags::default(), DatabaseOpts::default())?;
let conn = db.connect()?;

// 2. Create the source tables and register them as a graph (once per database).
conn.execute(
    "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, age INTEGER); \
     CREATE TABLE knows(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);",
)?;
register_graph(
    &conn,
    &GraphRegistration {
        name: "social".to_owned(),
        node_sources: vec![NodeSourceRegistration {
            name: "Person".to_owned(),
            table: "people".to_owned(),
            identity_column: "id".to_owned(),
        }],
        relationship_sources: vec![RelationshipSourceRegistration {
            name: "KNOWS".to_owned(),
            table: "knows".to_owned(),
            identity_column: "id".to_owned(),
            start_column: "src".to_owned(),
            end_column: "dst".to_owned(),
            start_node_source: "Person".to_owned(),
            end_node_source: "Person".to_owned(),
        }],
    },
)?;

// 3. Attach a session to the registered graph — one call.
let graph = GraphConnection::open(conn, "social")?;

// 4. Write and read with Cypher.
let summary = graph.execute("CREATE (:Person {id: 1, name: 'Ada', age: 36})", &Default::default())?;
let rows = graph.query("MATCH (n:Person) RETURN n.name, n.age", &Default::default())?;
```

Registration is persistent: on later runs, skip step 2 and call
`GraphConnection::open(conn, "social")` directly. `load_registered_graph`
and `graph_generation` are available for introspection.

Identity columns must be `PRIMARY KEY` or `UNIQUE`. A graph may register
multiple node and relationship sources. Relationship sources name the node
source stored at each endpoint, so identities are table-local coordinates:
equal numeric identities in two source tables remain distinct graph entities.

## Optional semantic schema

`register_graph` keeps the source-derived, schemaless behavior shown above.
Applications that need stable conceptual identities and typed property
ownership can add a semantic schema to an already registered graph:

```rust
use turso_graph_frontend::{
    register_semantic_schema, SemanticNodeType, SemanticProperty,
    SemanticRelationshipType, SemanticSchemaRegistration,
};

register_semantic_schema(
    &conn,
    "social",
    &SemanticSchemaRegistration {
        node_types: vec![
            SemanticNodeType {
                name: "Person".to_owned(),
                source: "Person".to_owned(),
                properties: vec![
                    SemanticProperty {
                        name: "displayName".to_owned(),
                        column: "name".to_owned(),
                    },
                    SemanticProperty {
                        name: "age".to_owned(),
                        column: "age".to_owned(),
                    },
                ],
            },
        ],
        relationship_types: vec![
            SemanticRelationshipType {
                name: "KNOWS".to_owned(),
                source: "KNOWS".to_owned(),
                start: vec!["Person".to_owned()],
                end: vec!["Person".to_owned()],
                properties: Vec::new(),
            },
        ],
    },
)?;
```

Registration validates the complete definition before writing, commits it
atomically, and accepts an identical replay. A conflicting replay is rejected.
Semantic type and property IDs are persisted independently from source IDs and
column positions. The immutable semantic snapshot loaded by
`GraphConnection::open` maps those conceptual IDs to physical sources only at
lowering time.

Once a graph has semantic rows, Cypher uses strict semantic mode:

- node creation and merge require exactly one known semantic node type;
- relationship creation and merge require exactly one semantic relationship
  type and validate its start/end node types;
- each semantic type routes reads and writes to its declared physical source;
- unlabeled node and untyped relationship patterns union all compatible source
  branches without deduplicating table-local identities;
- reads and writes resolve only properties owned by every possible target type;
- statically known property values are checked while binding, while deferred
  expressions and dynamic maps are checked before physical mutation;
- any invalid value, dynamic key, endpoint, or later staged row aborts the
  complete Cypher mutation savepoint.

Graphs without semantic rows are not promoted and retain the legacy behavior.
Semantic registration does not change canonical storage: property values remain
in the application-owned source columns.

### Fragment interfaces

Fragments add reusable node-property interfaces and polymorphic scans without
changing the existing semantic registration structs. Register the complete
schema and fragment definition atomically:

```rust
use turso_graph_frontend::{
    register_semantic_schema_with_fragments, SemanticFragment,
    SemanticFragmentMember, SemanticFragmentRegistration,
};

let fragments = SemanticFragmentRegistration {
    fragments: vec![SemanticFragment {
        name: "Nameable".to_owned(),
        properties: vec!["displayName".to_owned()],
        members: vec![
            SemanticFragmentMember {
                node_type: "Person".to_owned(),
                properties: vec![SemanticProperty {
                    name: "displayName".to_owned(),
                    column: "name".to_owned(),
                }],
            },
        ],
    }],
};

register_semantic_schema_with_fragments(
    &conn,
    "social",
    &schema,
    &fragments,
)?;
```

Every member maps every declared fragment property onto a column on that
concrete type's source. Fragment names and identities are graph-scoped,
persisted, and case-insensitive. A fragment name cannot collide with a concrete
semantic type.

`MATCH (n:Nameable)` unions concrete member scans, including members backed by
different physical sources. `MATCH (n:Person:Nameable)` is label-set
intersection and selects `Person` only when `Person` carries `Nameable`.
Fragments are interfaces, not abstract node instances: `CREATE` and `MERGE`
still require one explicit concrete node type. A concrete label may be
accompanied only by fragments that type carries.

Relationship endpoint constraints may name a fragment. Registration expands
that fragment to its concrete member-type set, while still checking that every
member is compatible with the relationship source's physical endpoint.

The fragment-aware call can also add the first fragment definition to an
already registered semantic schema when the supplied base schema is identical.
That upgrade is atomic and idempotent. Changing or removing a persisted
fragment or membership remains an explicit future schema-evolution operation.

### Direct-SQL integrity boundary

Semantic ownership, value-type, and endpoint checks are graph-frontend
guarantees. SQL issued directly against a registered source table bypasses
them. Physical `NOT NULL`, `UNIQUE`, `CHECK`, foreign-key, and application
triggers still apply, but Milestones 1–2 do not install equivalent semantic
constraints for direct SQL writers.

If all writers must preserve semantic integrity, route writes through Cypher or
enforce the same rules with physical schema constraints under application
control. Direct-SQL enforcement, required/cardinality/key constraints,
fragment-membership removal, broader schema evolution, and native n-ary
relationships are deliberately deferred. This overlay does not claim TypeDB,
TypeQL, or PERA compatibility.

## The session API

`GraphConnection` (root alias `Connection`) decorates a
`turso_core::Connection` — the same session object SQL uses — and mirrors
the `turso`/`turso_pg` API shapes:

| Method | Purpose |
|---|---|
| `GraphConnection::open(conn, name)` | One-call attach: loads the registration, builds a `SchemaCatalog`, private `SnapshotStore`, default `BuildLimits`, and installs the Cypher compiler on the connection |
| `open_with_parameters(conn, name, ParameterTypes)` | Same, plus declares the `$parameter` names/types this session's queries may bind (`ParameterTypes = HashMap<String, (ValueType, Nullability)>`) |
| `install(conn, &graph, catalog, params, snapshots, limits)` | Advanced path: share one `SnapshotStore` across connections to the same `Database`, or tune `BuildLimits` |
| `prepare(source, &params) -> Statement` | Compile a Cypher read. Returns a `Statement` wrapper: `Deref`s to `turso_core::Statement` (step/bind/row exactly like SQL) and exposes `result_types()` — the static Cypher column types in projection order. Booleans reach storage as integers, so faithful rendering needs these types. `EXPLAIN`-prefixed queries delegate to core's `EXPLAIN QUERY PLAN` and report empty `result_types` |
| `query(source, &params) -> Vec<Vec<Value>>` | Prepare + run to completion, collecting rows |
| `execute(source, &params) -> MutationSummary` | Run a Cypher mutation (`CREATE`/`MERGE`/`SET`/`REMOVE`/`DELETE`). `MutationSummary` carries `matched_rows`, `operations_executed`, and `rows` (populated by a trailing `RETURN`) |
| `prepare_cancellable` / `query_cancellable` | Same as above with a cooperative `Cancellation` hook, checked during snapshot builds and long traversals |

Values are plain `turso_core::Value` — the crate re-exports the common
core types at its root (`Value`, `Row`, `OpenFlags`, `Database`, …) and
the whole engine as `turso_graph_frontend::core`, so consumers usually
don't need a direct `turso_core` dependency.

### Parameters

Cypher `$name` parameters bind from a `Parameters` map
(`HashMap<String, Value>`). Binding is validated both ways: every
declared parameter must have a value (`Error::MissingParameter`) and
every supplied name must be declared (`Error::UndeclaredParameter`).
Sessions created with `open()` declare no parameters; use
`open_with_parameters` (or `install`) to declare them.

### Errors

Everything surfaces as `turso_graph_frontend::Error`
(`Result<T>` alias at the crate root): `Parse`, `Bind` (span-annotated),
`Snapshot`, `Mutation`, `Database(LimboError)`, and the two parameter
variants above.

## Transactions

Transaction control is the connection's, exactly as in the SQL frontends:
issue `BEGIN` / `COMMIT` / `ROLLBACK` as SQL on the same connection. One
transaction spans SQL statements and Cypher statements alike — an outer
SQL `BEGIN` … `ROLLBACK` undoes a Cypher `CREATE`.

- Each `execute()` wraps its work in an internal savepoint
  (`SAVEPOINT __turso_graph_mutation`), so a mutation is atomic on its
  own in autocommit and nests correctly inside your explicit transaction.
- Deleting a node that still has relationships without `DETACH` fails
  with `MutationError::NodeHasRelationships`.

### Traversal snapshots (variable-length paths)

Fixed-hop `MATCH` patterns lower to plain relational joins with normal
read-your-writes semantics. Variable-length patterns (`*`, `*min..max`)
run against an in-memory adjacency snapshot:

- Each session keeps a **connection-local** snapshot that is rebuilt
  (inside a nested savepoint) before a traversal read whenever the
  graph's generation counter has moved — so a session sees its own
  uncommitted writes without publishing them.
- The **shared** `SnapshotStore` caches the last committed snapshot per
  graph; refreshing it requires autocommit (`SnapshotError::RefreshInsideTransaction`).
  Share one store across connections via `install` to amortize rebuilds.
- Snapshots are derived, process-local state — never persisted, always
  rebuildable from the tables.

## Composing frontends

The graph frontend neither depends on nor is depended on by the Postgres
frontend. All frontends plug into core's per-connection compiler registry
(`Connection::register_frontend_compiler`), so an application that wants
Cypher and Postgres SQL on one connection installs both itself — a
`GraphConnection` and a `turso_pg::PgConnection` can decorate the same
`Arc<turso_core::Connection>`, and one `BEGIN`…`COMMIT` then covers
statements from every frontend on that connection. Separate databases
per frontend work too, with fully independent transactions; there is no
cross-connection or cross-file atomic commit.

## Reference

- `graph/README.md` — crate layout and quickstart
- `graph/DESIGN_DECISIONS.md` — storage overlay, catalog, snapshot design
- `graph/CONFORMANCE.md` + `graph/test-results/REPORT.md` — Cypher
  conformance corpus and current pass rates
- `docs/archive/plans/2026-07-21-graph-frontend-api-alignment.md` — how
  the consumer API reached its current baseline-aligned shape
