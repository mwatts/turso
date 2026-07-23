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

### Semantic constraints and additive evolution

Constraints are registered separately after the semantic schema so existing
schema registration structs remain source-compatible:

```rust
use turso_graph_frontend::{
    register_semantic_constraints, SemanticConstraintRegistration,
    SemanticEndpoint, SemanticKeyConstraint, SemanticPropertyValueConstraint,
    SemanticRangeBound, SemanticRelationshipCardinality,
    SemanticRequiredProperty, SemanticScalar, SemanticUniqueProperty,
    SemanticValuePredicate,
};

register_semantic_constraints(
    &conn,
    "social",
    &SemanticConstraintRegistration {
        required: vec![SemanticRequiredProperty {
            owner: "Person".to_owned(),
            property: "displayName".to_owned(),
        }],
        keys: vec![SemanticKeyConstraint {
            owner: "Person".to_owned(),
            properties: vec!["displayName".to_owned(), "age".to_owned()],
        }],
        unique: vec![SemanticUniqueProperty {
            owner: "Person".to_owned(),
            property: "displayName".to_owned(),
        }],
        values: vec![
            SemanticPropertyValueConstraint {
                owner: "Person".to_owned(),
                property: "age".to_owned(),
                predicate: SemanticValuePredicate::Range {
                    minimum: Some(SemanticRangeBound {
                        value: SemanticScalar::Integer(0),
                        inclusive: true,
                    }),
                    maximum: None,
                },
            },
            SemanticPropertyValueConstraint {
                owner: "Person".to_owned(),
                property: "displayName".to_owned(),
                predicate: SemanticValuePredicate::Regex {
                    pattern: r"^[A-Z]".to_owned(),
                },
            },
        ],
        cardinalities: vec![SemanticRelationshipCardinality {
            relationship_type: "KNOWS".to_owned(),
            endpoint: SemanticEndpoint::Start,
            minimum: 0,
            maximum: Some(100),
        }],
    },
)?;
```

A required property is non-NULL. Every member of a composite key is required,
and key tuples are unique within one concrete semantic owner type. A unique
property permits multiple NULL values but rejects duplicate non-NULL values
within its owner. Value predicates support inclusive/exclusive numeric or text
ranges, finite Boolean/integer/real/text allowed-value sets, and Rust regular
expressions over text.

Endpoint cardinality counts relationships of one semantic relationship type
at the selected stored start or end endpoint for every permitted concrete node
type. The minimum and optional maximum apply per node. Incoming Cypher syntax
does not reverse the stored constraint: endpoint validation follows the
relationship source's stored start/end mapping.

Constraint registration is append-only, atomic, and idempotent. New
constraints validate all visible data inside the registration transaction
before catalog rows become active. Failure leaves the prior catalog,
generation, and application data unchanged. Replaying an identical constraint
set writes nothing. Changing or removing an active constraint, remapping a
type/property, and removing a fragment membership require a future explicit
evolution API and are rejected by additive registration. A `GraphConnection`
created through `open` or `open_with_parameters` compares its catalog
generation before preparing reads or executing mutations and reloads the
immutable semantic snapshot when registration publishes a newer generation.
Sessions created through `install` retain their caller-supplied catalog.

Literal value predicates can fail during binding. Deferred expressions and
dynamic maps are checked at runtime. Required, key, unique, and endpoint
cardinality state is validated before the complete Cypher mutation savepoint
is released, so any failure rolls back every row and operation. Validation at
the final savepoint boundary also lets one multi-operation query repair a
temporary intermediate database-state violation before commit.

### Direct-SQL integrity boundary

Semantic ownership, value-type, endpoint, and Milestone 4 constraint checks are
graph-frontend guarantees. SQL issued directly against a registered source
table bypasses semantic membership and validation. Physical `NOT NULL`,
`UNIQUE`, `CHECK`, foreign-key, and application triggers still apply.

The graph frontend does not install a native unique index when multiple
semantic types share a source: such an index would incorrectly enforce
uniqueness across types, while a partial index cannot express membership in
the graph's label/type junction table. The same boundary applies to composite
keys and relationship participation counts. Existing physical constraints
remain the durable direct-SQL enforcement mechanism wherever their semantics
match.

If all writers must preserve semantic integrity, route writes through Cypher or
enforce the same rules with physical schema constraints under application
control. Database-wide protection of owned backing tables,
fragment-membership removal, non-additive constraint evolution, and native
n-ary relationships remain deferred. This overlay does not claim TypeDB,
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
| `diagnostics() -> GraphDiagnostics` | Inspect the calling session's traversal snapshot status and aggregate resource metadata without refreshing or publishing it |

Values are plain `turso_core::Value` — the crate re-exports the common
core types at its root (`Value`, `Row`, `OpenFlags`, `Database`, …) and
the whole engine as `turso_graph_frontend::core`, so consumers usually
don't need a direct `turso_core` dependency.

### Catalog procedures

Read queries expose three typed, read-only catalog procedures:

```cypher
CALL db.labels() YIELD label RETURN label ORDER BY label
CALL db.relationshipTypes() YIELD relationshipType RETURN relationshipType
CALL db.propertyKeys() YIELD propertyKey RETURN propertyKey ORDER BY propertyKey
```

`db.propertyKeys()` enumerates declared logical payload columns across every
registered node and relationship source. It does not scan graph rows, so an
empty nullable column is still reported; identity and relationship endpoint
columns are excluded, shared names are returned once, and reserved physical
columns such as `cyprop_id` are reported by their logical name (`id`).

For a graph with a semantic schema, all three procedures use semantic catalog
names. This includes concrete and fragment labels, relationship types, and the
union of properties owned by semantic types. Legacy graphs retain their
data-backed label/type enumeration and source-schema property enumeration.
Procedure names resolve case-insensitively; unknown names, invalid arity, and
unknown or duplicate `YIELD` columns fail during binding.

### Parameters

Cypher `$name` parameters bind from a `Parameters` map
(`HashMap<String, Value>`). Binding is validated both ways: every
declared parameter must have a value (`Error::MissingParameter`) and
every supplied name must be declared (`Error::UndeclaredParameter`).
Sessions created with `open()` declare no parameters; use
`open_with_parameters` (or `install`) to declare them.

### Full-text search

Graph full-text search is opt-in and native-only. Enable the
`turso_graph_frontend/fts` Cargo feature, which forwards to
`turso_core/fts`, and open the database with index methods enabled:

```rust
let options = DatabaseOpts::default().with_index_method(true);
```

The feature does not compile for wasm targets. Without it, `fts_match`,
`fts_score`, and `fts_highlight` fail during Cypher binding with an explicit
unsupported-capability error; they never fall through to a core
`no such function` error.

Index administration is a typed Rust API:

```rust
use turso_graph_frontend::{
    GraphFtsEntityKind, GraphFtsIndexSpec, GraphFtsTokenizer,
};

graph.create_fts_index(&GraphFtsIndexSpec {
    name: "article_search".to_owned(),
    entity: GraphFtsEntityKind::Node,
    source: "Article".to_owned(),
    properties: vec!["title".to_owned(), "body".to_owned()],
    tokenizer: GraphFtsTokenizer::Default,
    weights: Vec::new(),
})?;

let indexes = graph.list_fts_indexes()?;
graph.drop_fts_index("article_search")?;
```

Logical names and properties are validated against the registered graph
catalog. Identity/end-point columns and statically non-text properties are
rejected. Configuration is bounded to 128 bytes per logical index name and 16
properties. Tokenizers and weights are typed values rather than SQL fragments.
The physical index uses a stable reserved `__turso_graph_fts_*` name, while a
versioned internal metadata row preserves the logical definition for listing,
duplicate detection, reopen, and drop. Physical DDL and metadata share one
transaction/savepoint; a conflicting same-name definition is an error rather
than an implicit replacement.

Queries use the portable scalar surface and keep user text bound:

```cypher
MATCH (n:Article)
WHERE fts_match(n.title, n.body, $query)
RETURN n, fts_score(n.title, n.body, $query) AS score
ORDER BY score DESC
LIMIT 20
```

Lowering emits a core FTS rowid-set subquery tied to the matched node identity,
so the custom index is visible in `EXPLAIN QUERY PLAN`. The current layered
graph plan still scans its outer node relation; the FTS lookup does not yet
replace that outer scan. Core owns insert, update, delete, transaction, and
reopen durability. After an index is dropped, the scalar predicate has no
indexed matches and returns an empty result. Tokenizer semantics are preserved;
for example, `Simple` searches are case-sensitive.

No read-only `db.index.fulltext.queryNodes` procedure is added in this
milestone. Although a procedure could bypass the current outer scan, it would
duplicate the scalar query surface and create a second optimization path;
teaching normal `MATCH` lowering to drive its outer source from the FTS rowid
set is the preferred follow-up. Ranking, bound query text, `ORDER BY`, and
`LIMIT` remain expressible with the scalar form. Vendor names such as
`spa.fulltext.queryNodes` and `full_text_search` are intentionally unsupported.
The Rust administration API uses the caller's database authority; any future
network binding must add its own authorization before exposing these methods
remotely.

The `fts_search` benchmark records a 10,000-row corpus, 1% selectivity,
`LIMIT 20`, warm and new-session indexed queries, and a `CONTAINS` table-scan
control. On the Phase 4 development run, mean times were 8.56 ms indexed-warm,
8.34 ms indexed-new-session, and 1.18 ms for the scan control. The result is
expected for this small, early-exit workload because the current plan pays FTS
setup while retaining the outer scan; it is the saved comparison point for the
outer-source optimization above.

The Phase 5 validation rerun measured 7.783 ms indexed-warm, 7.913 ms
indexed-new-session, and 1.145 ms for the scan control under the same workload.
This is consistent with the Phase 4 baseline and does not change the
optimization conclusion above.

### Errors

Everything surfaces as `turso_graph_frontend::Error`
(`Result<T>` alias at the crate root): `Parse`, `Bind` (span-annotated),
`Snapshot`, `Mutation`, feature-gated `Fts`, `Database(LimboError)`, and the two parameter
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

`GraphConnection::diagnostics()` reports the graph identity/name, persistence
mode, and `SnapshotStatus` (`Missing`, `Current`, or `Stale`). Current and stale
metadata includes catalog/source generations, node and relationship counts,
build duration, retained-byte estimate, and peak-build-byte estimate. A stale
status also includes the currently visible generation. The calling session's
transaction-local overlay takes precedence over shared committed state, so
diagnostics match the snapshot a traversal would use.

The call is strictly observational: it performs no refresh, catalog write, or
snapshot publication, and its types contain no source row values, relationship
coordinates, or query text. SQL diagnostics are intentionally not installed:
the state is process- and session-local, and there is no SQL consumer requiring
a virtual-table projection beyond this typed API. This avoids implying that
diagnostics are persistent catalog state.

## Composing frontends

The graph frontend neither depends on nor is depended on by the Postgres
frontend. All frontends plug into core's per-connection compiler registry
(`Connection::register_frontend_compiler`), so an application that wants
Cypher and Postgres SQL on one connection installs both wrappers itself:

```rust
use turso_graph_frontend::GraphConnection;
use turso_pg::PgConnection;

let conn = db.connect()?;
// Create source tables and call register_graph(&conn, ...) before opening.
let postgres = PgConnection::new(conn.clone());
let graph = GraphConnection::open(conn.clone(), "social")?;

conn.execute("BEGIN IMMEDIATE")?;
graph.execute("CREATE (:Person {name: 'Ada'})", &Default::default())?;
postgres.execute("UPDATE people SET name = 'Grace'")?;
let rows = graph.query(
    "MATCH (n:Person) RETURN n.name",
    &Default::default(),
)?;
conn.execute("COMMIT")?;
```

The database has one host `Dialect`; the wrappers add statement compilers to
that connection. In attach mode, open the database with `SqliteDialect`, create
and register the graph through core SQL, and then construct both wrappers.
Opening with `GraphDialect` or `PostgresDialect` instead selects that dialect's
schema parsing and function surface for the whole database; it does not create
per-statement host dialects.

Use graph/Cypher for entity creation and relationship mutation. A direct SQL
insert into a graph source table does not create the semantic type-membership
rows required by strict semantic mode. SQL or PostgreSQL can read graph-created
rows and update their ordinary columns; those updates are visible to Cypher on
the same connection. The integration test
`postgres_and_graph_frontends_share_one_connection_transaction` proves
cross-frontend visibility and rollback.

Separate databases or connections have fully independent transactions; there
is no cross-connection or cross-file atomic commit. The raw core connection
also bypasses any frontend-level namespace policy, so keep it private when
frontend isolation is a security requirement.

## Reference

- `graph/README.md` — crate layout and quickstart
- `graph/DESIGN_DECISIONS.md` — storage overlay, catalog, snapshot design
- `graph/CONFORMANCE.md` + `graph/test-results/REPORT.md` — Cypher
  conformance corpus and current pass rates
- `docs/archive/plans/2026-07-21-graph-frontend-api-alignment.md` — how
  the consumer API reached its current baseline-aligned shape
