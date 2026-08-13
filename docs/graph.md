# Using the Graph (Cypher) Frontend

The graph frontend (`turso_graph_frontend`) runs Cypher queries against a
standard Turso database file. Nodes and relationships are ordinary SQL
tables that you *register* as a graph; Cypher compiles down to the same
AST → VDBE pipeline the SQL frontends use, so storage, WAL, and
transactions are all the engine's own. There is no separate graph file
format — graph metadata (labels, relationship types, generation counters)
lives as `__tdb_int_g_*` tables and indexes inside the same
`.db` file as your SQL schema.

> **Status:** experimental, source-only — no published crate, no language
> binding. Consumers embed `turso_graph_frontend` synchronously from a
> workspace path. The graph frontend lives on the `feature/graph-frontend`
> branch as five crates (`turso_graph_cypher`, `turso_graph_ir`,
> `turso_graph_runtime`, `turso_graph_frontend`, `turso_graph_temporal`),
> deliberately decoupled from the Postgres frontend — see "Composing
> frontends" below. Feature branches such as `feature/graph-nary` carry one
> piece of work each and merge back into it.
>
> This document is the user guide: how to use it, the language it accepts,
> what it does beyond standard Cypher, and how to test changes to it. For
> how it works inside — the pipelines, the role model's invariants, and
> where to change what — see
> [`docs/graph-internals.md`](graph-internals.md).

## Quickstart

In the shell, `CREATE GRAPH` declares a graph and the tables behind it in one
statement, and `.graph <name>` switches input to Cypher:

```text
turso> CREATE GRAPH social
   ...>   NODE Person (name TEXT, age INTEGER)
   ...>   RELATION KNOWS (since INTEGER)
   ...>     ROLE start -> Person
   ...>     ROLE end -> Person;
Graph social created. Use ".graph social" to query it.
turso> .graph social
Reading Cypher against graph social. Use ".graph off" for SQL.
social> CREATE (:Person {id: 1, name: 'Ada', age: 36});
social> CREATE (:Person {id: 2, name: 'Grace', age: 45});
social> MATCH (a:Person {id: 1}), (b:Person {id: 2})
   ...>   CREATE (a)-[:KNOWS {id: 1, since: 1952}]->(b);
social> MATCH (a:Person)-[k:KNOWS]->(b:Person) RETURN a.name, b.name, k.since;
Ada|Grace|1952
social> .graph off
Reading SQL.
turso> SELECT name, age FROM Person;
Ada|36
Grace|45
```

The mode is explicit because `CREATE` belongs to both languages, so the shell
cannot tell a Cypher statement from a SQL one by looking at it. `CREATE GRAPH`
itself is the exception — it is not valid SQL, so it works from either mode.

That last `SELECT` is the point: the graph is a view over ordinary tables, not
a second store. `CREATE GRAPH` adds no storage model — it infers the physical
names the statement leaves unsaid and calls the same `register_graph` a Rust
caller would.

### What gets inferred, and how to override it

| Declared | Inferred | Override |
|---|---|---|
| `NODE Person` | table `Person` | `NODE Person AS TABLE people` |
| `RELATION KNOWS` | table `KNOWS` | `RELATION KNOWS AS TABLE knows` |
| identity column | `id INTEGER PRIMARY KEY` | `KEY <column>` |
| `ROLE start -> Person` | column `start INTEGER` | `VIA <column>` |
| `ROLE witnesses -> Text MANY` | spill table `<relation>__witnesses` | — (`VIA` is refused) |

Tables are created `IF NOT EXISTS`, which is what lets one syntax both create
and adopt. Pointed at a schema you already have, the declaration registers
those tables instead of new ones and backfills membership for the rows already
in them, so Cypher sees data written before the graph existed:

```text
turso> CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT);
turso> CREATE TABLE knows(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);
turso> INSERT INTO people VALUES (1, 'Ada'), (2, 'Grace');
turso> INSERT INTO knows VALUES (1, 1, 2);
turso> CREATE GRAPH social
   ...>   NODE Person AS TABLE people KEY id (name TEXT)
   ...>   RELATION KNOWS AS TABLE knows KEY id
   ...>     ROLE start -> Person VIA src
   ...>     ROLE end -> Person VIA dst;
```

Adoption is not blind: registration verifies every named column exists and
that the identity column is unique, so a shape mismatch surfaces as an error
naming the exact column rather than as a graph that half-works. The whole
statement runs in one transaction — a declaration that fails leaves neither
tables nor catalog rows behind.

`CREATE GRAPH` reaches the physical registration only. Semantic types,
constraints, relation-as-player roles, and multi-target roles are still
declared through the Rust API — see "Optional semantic schema" below.

### From Rust

The same graph, built through the API the DDL calls:

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
        relationship_sources: vec![RelationshipSourceRegistration::binary(
            "KNOWS", "knows", "id", "src", "dst", "Person", "Person",
        )],
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
multiple node and relationship sources. `binary(...)` above is a convenience
constructor for the common two-role case; a relationship source can instead
declare any number of named roles directly — see "Roles" below. Either way,
identities are table-local coordinates: equal numeric identities in two
source tables remain distinct graph entities.

## Roles

Every relationship source declares one or more named **roles**. Each role has
a list of target source names it may point at, whether it is optional, and a
**cardinality** — `One` (a single player) or `Many` (players spill into a side
table named `<table>__<role>`). `RelationshipSourceRegistration::binary(...)`
used in the Quickstart above is a convenience constructor, not a separate
code path: it registers a two-role relation with the roles named `start` and
`end`. **Binary is a layout of the role model, not a separate kind** — there
is no `is_binary` flag and no branching on arity anywhere in the general
machinery. Roles resolve by declared name or `RoleId`, never by position or
count.

A relation with roles other than `start`/`end` (or with more than two roles)
is registered and queried with the standalone role-pattern syntax. Given a
`Transcription` relationship source with `scribe`/`text`/`folio` roles over
plain `Person`/`Text`/`Folio` node sources:

```cypher
MATCH (p:Person), (t:Text), (f:Folio)
CREATE [x:Transcription {year: 1387}](scribe: p, text: t, folio: f)

MATCH [x:Transcription](scribe: s, text: doc, folio: f) RETURN x.year
```

(`graph/frontend/tests/nary_relations.rs::a_three_role_relation_writes_one_row_with_three_endpoint_columns`
and `a_match_role_pattern_reads_a_three_role_relation` run the create and
match forms end to end against exactly this schema — `fixture::ternary_session`.
That fixture has three node sources and no semantic schema, so it cannot
resolve a *node* player's own properties, e.g. `s.id` — reading a role
player's properties this way needs either a semantic schema or a graph with
one node source per property name, same as any other Cypher property read.)

`[x:Type {props}](role: player, role2: player2, …)` both creates a relation
instance and matches one: each parenthesized role list names a role and
binds (or requires) its player(s); role arguments bind by name, not by the
order they're written
(`role_arguments_bind_by_name_regardless_of_source_order`). The familiar
arrow forms — `(a)-[:KNOWS]->(b)` for CREATE and `(x:TYPE)-[:role]->(player)`
for a read off an already-bound relation — are sugar over the same
`RoleJoin`/`RelationScan` machinery and bind to the identical plan as the
standalone pattern
(`the_role_arrow_and_the_role_pattern_bind_to_the_same_plan`). Both arrow
forms only work when the relation has roles literally named `start` and
`end`: `RelationshipSourceRegistration::binary(...)`'s two roles, or any
relation source that happens to declare roles by those names. A relation
without that pair — like the ternary `Transcription` above — cannot be
created or traversed with an arrow at all: attempting one fails at bind
time before touching any row
(`an_arrow_form_create_requires_a_start_and_end_role_pair`,
`an_arrow_form_expand_requires_a_start_and_end_role_pair`) and must use the
standalone pattern instead. This applies to `RoleExpand` (fixed-hop) and
`GraphExpand` (variable-length, `*`/`*min..max`) traversal too — both
discover candidate relationship sources through the same `start`/`end`
lookup, so a relation needs that literal role pair before it can be
traversed with an arrow in either form.

Reading a role by name off an already-bound relation is also available as
arrow-form sugar — `(x:KNOWS)-[:end]->(e)` resolves the `end` role's player,
including through a `Many` role
(`an_arrow_from_a_relation_reads_that_relations_role`,
`a_many_role_hops_from_the_arrow_sugar_too`). When a name is both a role of
the anchored relation type and a separate relationship type name, binding
rejects the query as ambiguous rather than guessing which one was meant
(`a_name_that_is_both_a_role_and_a_relationship_type_is_ambiguous`).

`SET` on a role **replaces** rather than appends for a `Many`-cardinality
role: `SET [r](witness: w3)` sets `r`'s entire `witness` player set to
`{w3}`, discarding whoever was there before — it is not an add
(`setting_a_many_valued_role_replaces_rather_than_appends`).

A relation may itself fill another relation's role: whether it may depends
entirely on that role's declared target list (a role can target node
sources, relation sources, or both), with no separate "relation-as-player"
code path — it is decided the same way a role's node-source targets are, by
membership in `targets`. For example, a `Citation` relationship type whose
`cited` role targets `Transcription` (a relation, not a node) accepts a
transcription's own relation identity as its `cited` player
(`a_relation_may_be_a_player_of_another_relation`, `fixture::citation_session`).

`graph/frontend/tests/nary_relations.rs` is the executable reference for this
section's exact syntax and refusal wording.

## The Cypher language surface

The frontend follows **openCypher/TCK-normative** semantics where donors
disagree. The authoritative surface is the code, not this list: clauses come
from `cypher::Clause` (`graph/cypher/src/ast.rs`), and scalar functions from the
name match in `graph/frontend/src/binder.rs`. What follows is the shape of it.

### Clauses

`MATCH` · `CREATE` · `MERGE` · `SET` · `REMOVE` · `DELETE` (and `DETACH DELETE`)
· `UNWIND` · `WITH` · `RETURN` · `FOREACH` · `CALL` · `CALL { … }` scoped
subquery · `UNION` / `UNION ALL`.

Projections carry the usual `DISTINCT`, `ORDER BY`, `SKIP`, `LIMIT`, and
aggregation with grouping.

`SET` has five forms, the last of which is not standard Cypher:

| Form | Meaning |
|---|---|
| `SET n.prop = v` | Set one property |
| `SET n = {…}` | Replace **every** property |
| `SET n += {…}` | Merge properties, keeping the rest |
| `SET n:Label1:Label2` | Add labels |
| `SET [x](role: player, …)` | Repoint named roles of an already-bound relation |

#### MERGE match rules (Turso product)

- **Match key required.** A MERGE must discriminate rows with properties,
  labels/types, and/or role endpoints. A property-less `MERGE (n)` with no
  labels is rejected (`EmptyMergeKey`) instead of matching an arbitrary row.
- **Many-role match is exact multiset.** For a `Many` role, the spill players
  must equal the pattern’s players (same identities and count). A relationship
  with witnesses `{A,B}` does **not** match `MERGE … (witness: A)` alone.
- **Concurrent MERGE uniqueness.** Relationship MERGE patterns claim a stable
  key in `__tdb_int_g_mkey` so two sessions MERGEing the same
  pattern (including shared multi-type tables and Many multisets) leave one
  relationship row.

### Patterns

Node patterns, arrow relationship patterns with `Outgoing` / `Incoming` /
`Both` direction, variable-length ranges (`*`, `*min..max`), inline property
maps, and the standalone role pattern `[x:T {props}](role: player, …)`.

Direction is a *parser-level* concept only: the binder resolves it to a role
pair, and nothing downstream reasons about incoming versus outgoing.

### Expressions

Variables, property access, function calls, unary and binary operators, `CASE`
(simple and searched), list indexing and slicing, casts, list and map literals,
parameters (`$name`), list comprehensions, and the quantified predicates
`ALL` / `ANY` / `NONE` / `SINGLE`.

### Functions

Aggregates: `count`, `sum`, `avg`, `min`, `max`, `collect`.

Scalars mapped by the binder include `id`, `toUpper`/`toUpperCase`,
`toLower`/`toLowerCase`, `toString`, `toInteger`, `toFloat`, `toBoolean`,
`toStringList`, `toIntegerList`, `toFloatList`, `toBooleanList`, `size`,
`range`, `split`, `keys`, `head`, `last`, `tail`, `left`, `right`, `isEmpty`,
`rand`, and `reduce`. Vector distance functions (`cosine_distance`,
`l2_distance`, `inner_product`) are available as Cypher-level scalars.

A name the binder does not map falls through to the dialect's own function
surface (see "Beyond standard Cypher" below) and then to core. That fallthrough
is why an unsupported name usually surfaces as a core resolution error rather
than a Cypher-level one — with the deliberate exception of the FTS scalars,
which fail during binding with an explicit unsupported-capability error when
the `fts` feature is off.

### Catalog procedures

`db.labels()`, `db.relationshipTypes()`, `db.propertyKeys()` — read-only, typed,
resolved case-insensitively. Unknown names, wrong arity, and unknown or
duplicate `YIELD` columns fail during binding. See "Catalog procedures" under
the session API for exactly what `db.propertyKeys()` counts.

### Graph declaration

`CREATE GRAPH` is Turso's own DDL, parsed by `parse_ddl` rather than the query
grammar — a declaration is a whole statement, not a clause, so it cannot be
combined with `MATCH` or `RETURN`:

```text
CREATE GRAPH <name>
  ( NODE <Label> [AS TABLE <table>] [KEY <column>] [(<col> <TYPE>, …)]
  | RELATION <Type> [AS TABLE <table>] [KEY <column>] [(<col> <TYPE>, …)]
      ( ROLE <role> -> <Label> [VIA <column>] [MANY] )+
  )+
```

Its keywords are not reserved: a query may still bind a variable named `node`,
`role`, or `key`. See "Quickstart" above for what each optional clause
overrides.

### Known gaps

`graph/DESIGN_DECISIONS.md` carries the failure taxonomy and the reasoning
behind each open family. Two are settled as permanent:

- Runtime `TypeError`s for entity values flowing through `Any`-typed lists need
  an error-raising SQL function; a `SELECT` cannot raise.
- AGE jsonb operators (`?`, `@>`, `#>`), pgvector `OPERATOR(...)`, and a set of
  expected-error adapter artifacts are donor-semantic conflicts with
  TCK-normative behavior. These are tracked as *divergences*, not bugs, and are
  enforced through `graph/registries/divergence.toml`.

## Beyond standard Cypher

Features here are Turso-specific. Portable Cypher does not have them, and
queries using them will not run on other engines.

| Feature | Surface | Section |
|---|---|---|
| **Native n-ary relations** | `[x:T {props}](role: player, …)` on CREATE/MERGE/MATCH/SET | [Roles](#roles) |
| **Many-cardinality roles** | Same syntax; players spill to `<table>__<role>` | [Roles](#roles) |
| **Relation-as-player** | A role whose `targets` name a relation source | [Roles](#roles) |
| **Semantic schema** | `register_semantic_schema` | [Optional semantic schema](#optional-semantic-schema) |
| **Fragment interfaces** | `register_semantic_schema_with_fragments`, `MATCH (n:Nameable)` | [Fragment interfaces](#fragment-interfaces) |
| **Semantic constraints** | `register_semantic_constraints` | [Semantic constraints and additive evolution](#semantic-constraints-and-additive-evolution) |
| **Full-text search** | `fts_match`, `fts_score`, `fts_highlight` + typed admin API | [Full-text search](#full-text-search) |
| **Vector scalars** | `vector32`, `vector64`, `vector8`, `vector1bit`, `vector32_sparse`, `vector_extract`, `vector_concat`, `vector_slice`, `vector_distance_*` | — |
| **Struct / union scalars** | `struct_pack`, `union_value`, `union_tag` | — |
| **Traversal diagnostics** | `GraphConnection::diagnostics()` | [Traversal snapshots](#traversal-snapshots-variable-length-paths) |
| **Statement classification** | `GraphConnection::classify()` → `StatementKind` | — |

The `turso_graph_temporal` extension registers a further scalar surface that
Cypher lowering targets: `duration_make`/`_parse`/`_get`/`_add`/`_neg`/`_between`,
`temporal_make`/`_truncate`/`_parse`/`_get`/`_now`, `datetime_add_duration`,
`datetime_sub_duration`, the `jsonb_*` accessors (`jsonb_get`, `jsonb_get_text`,
`jsonb_get_path`, `jsonb_exists`, `jsonb_exists_any`, `jsonb_exists_all`,
`jsonb_contains`), the Cypher-semantics helpers `cypher_raise`, `cypher_equals`,
`cypher_add`, `cypher_sub`, `cypher_concat`, `cypher_div`, and `split`. The
canonical list is `turso_graph_temporal::FUNCTION_NAMES`, which
`GraphDialect::resolve_function` treats as the dialect-owned surface.

Vendor names are intentionally **not** aliased: `spa.fulltext.queryNodes` and
`full_text_search` are unsupported on purpose, not by oversight.

## Open modes and Core seams

The graph frontend uses Core the same way Postgres does: one host `Dialect`
per database file, and a per-connection `FrontendCompiler` for Cypher reads.
Two open modes exist. Both are supported.

### Dialect-pinned open

`open_database` / `open_database_with_io` open the file with `GraphDialect`
(`name() == "graph-cypher"`). Root prepares resolve temporal/`cypher_*` names
on the dialect. `GraphConnection::install` still registers the static temporal
extension so mutation helpers (`prepare_internal` / InternalHelper, SQLite
symbol table only) can resolve the same names. `turso_graphs` is registered on
schema build. A later reopen with a different dialect name is rejected.

### Attach mode

`GraphConnection::open` / `install` attach the graph layer to an existing
connection (often `SqliteDialect`). Guarantees come from `install`: compiler
registration, temporal extension, and the expand virtual table. The file's
dialect name may stay `"sqlite"`. Prefer dialect-pinned open for new graph
databases; use attach when the file already hosts another dialect.

### Expand virtual table (session activation)

Variable-length paths lower to the internal `__tdb_int_g_expand` virtual
table. That table holds a process-local [`SnapshotStore`] — derived adjacency
state, not durable catalog rows — so it **cannot** be installed from
`GraphDialect::register_catalog` at schema build (no connection snapshot
exists there). Both dialect-pinned and attach sessions activate expand via
`install_graph_catalog` inside `GraphConnection::install`.

`install_graph_catalog` is **idempotent**: calling it more than once on the
same connection succeeds; a later call replaces the earlier `SnapshotStore`
binding. Prefer one shared store per database (via `install`) when multiple
connections should amortize committed snapshot rebuilds.

### Reads

Cypher reads go through `prepare_frontend("graph-cypher")`. The compiler
lowers to engine AST; Core owns translate, reprepare, and step.

### Mutations

Mutations are multi-statement orchestration today (not a single
`PreparedSource`). That split is known debt, not an accident. Transaction
wrapping matches other graph admin helpers that use `prepare_internal`:

| Host state | Wrapper |
|------------|---------|
| Autocommit | `BEGIN IMMEDIATE` → work → `COMMIT` / `ROLLBACK` |
| Existing write transaction | `SAVEPOINT __tdb_int_g_mut` → work → `RELEASE` / `ROLLBACK TO` |
| Deferred read transaction (`BEGIN` without write) | `MutationError::RequiresWriteTransaction` — use `BEGIN IMMEDIATE` or a prior write |

Mutation helper SQL is prepared with `prepare_internal` (InternalHelper), so
it relies on the session-installed temporal extension for `cypher_*` /
`duration_*` names even under dialect-pinned open.

### Composition

Postgres and Graph stay separate crates. An app that needs both languages on
one connection registers both compilers itself. See
[Composing frontends](#composing-frontends). Alignment findings live in
[`docs/graph-frontend-core-alignment.md`](graph-frontend-core-alignment.md).

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
        relationship_types: vec![SemanticRelationshipType::binary(
            "KNOWS",
            "KNOWS",
            vec!["Person".to_owned()],
            vec!["Person".to_owned()],
            Vec::new(),
        )],
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
  type and validate every one of its declared roles' target types — `start`
  and `end` under the binary layout, or any other role name for an n-ary
  type;
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

A role's target list may name a fragment. Registration expands that fragment
to its concrete member-type set, while still checking that every member is
compatible with the relationship source's physical role column (or spill
table, for a `Many` role).

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
fragment-membership removal, and non-additive constraint evolution remain
deferred. So does role cardinality constraint validation past `start`/`end` —
the semantic overlay's own cardinality constraints (`SemanticEndpoint`) cover
only the two-role binary layout, even though the frontend's general role
model natively supports n-ary relationships (see "Roles" above). This overlay
does not claim TypeDB, TypeQL, or PERA compatibility.

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
empty nullable column is still reported; identity and relationship role
columns (`start`/`end` under the binary layout, or any other role's column)
are excluded, shared names are returned once, and reserved physical columns
such as `cyprop_id` are reported by their logical name (`id`).

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
catalog. Identity and relationship role columns and statically non-text
properties are rejected. Configuration is bounded to 128 bytes per logical index name and 16
properties. Tokenizers and weights are typed values rather than SQL fragments.
The physical index uses a stable reserved `__tdb_int_g_fts_*` name, while a
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

- Each `execute()` is atomic: in **autocommit** it opens
  `BEGIN IMMEDIATE` … `COMMIT` / `ROLLBACK`; inside an existing **write**
  transaction it uses `SAVEPOINT __tdb_int_g_mut` … `RELEASE` /
  `ROLLBACK TO`. Nested helpers cannot upgrade a deferred read transaction,
  so bare `BEGIN` without a prior write returns
  `MutationError::RequiresWriteTransaction` — use `BEGIN IMMEDIATE` (or
  write first), same as graph registration and FTS admin.
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

## Testing

Three layers, each answering a different question.

### Rust tests — does this behavior work?

```sh
cargo test -p turso_graph_frontend -p turso_graph_cypher -p turso_graph_ir -p turso_graph_runtime
```

The behavioral record lives in `graph/frontend/tests/`:

| File | Covers |
|---|---|
| `semantic_schema.rs` | Semantic types, fragments, constraints, strict mode |
| `nary_relations.rs` | Roles: n-ary create/match/merge/set/delete, `Many` spill, relation-as-player, refusal wording |
| `native_capabilities.rs` | Vector, struct/union, FTS, other native surfaces |
| `dialect_alignment.rs` | Dialect-pinned vs attach open, function resolution |
| `desugaring_golden.rs` | Arrow form and role pattern bind to the same plan |
| `type_system_fixtures.rs`, `fixed_pattern_fixtures.rs` | Static result types, fixed-hop lowering |
| `statement_kind.rs`, `api_surface.rs` | Classification and public API shape |
| `fixture.rs` | Shared fixtures — `social_graph_connection`, `ternary_session`, `witnessed_session`, `two_many_roles_session`, `citation_session`, `ambiguous_session`, and the `bind_*`/`lower_*` helpers |

Fixtures return `(Arc<Database>, GraphConnection)`. Reads go through
`session.query(sql, &Parameters::new())`; mutations through `session.execute`.
Using `execute` for a read yields a misleading "Cypher mutation binding failed"
rather than a real refusal.

### Conformance corpus — do we match other engines?

The corpus runs 10,242 imported source identities from the openCypher TCK,
Apache AGE, Grafeo, SparrowDB, and CQLite. LadybugDB/Kuzu is excluded because
its suite mixes vendor-specific language and result contracts into
standard-looking Cypher.

```sh
mise run corpus              # all five suites (release build, by design)
mise run cypherbench-sample  # execution benchmark over seven domains

cargo run -q -p turso_graph_testkit -- run smoke --no-record
cargo run -q -p turso_graph_testkit -- corpus-stats
cargo run -q -p turso_graph_testkit -- divergence
cargo run -q -p turso_graph_testkit -- verify-history
```

The `mise` tasks are the documented exception to the repo's "never build with
`--release`" rule: rows appended to `graph/test-results/history.jsonl` are only
comparable against that history when produced by an optimized build, and each
row records the profile it was built with.

**Read the corpus gate per suite, never as a total.** `tck-deep` flakes ±2
across identical commits, so the total moves with no code change at all:

| Suite | Baseline |
|---|---:|
| `age-deep` | 3,042 exact |
| `cqlite-deep` | 113 exact |
| `grafeo-deep` | 277 exact |
| `sparrowdb-deep` | 2,164 exact |
| `tck-deep` | 3,329–3,332 |

Two further traps worth knowing: `mise run corpus` **exits 1 even when every
suite is at baseline**, so read the numbers rather than the exit code; and
`divergence` is an enforced gate, not a report — it fails when an unsupported
outcome has no registry entry, when an entry names a test the run no longer
contains, or when a registered divergence starts passing.

Omit `--no-record` on an intentional baseline run to append to
`history.jsonl` and regenerate `graph/test-results/REPORT.md`.

### Benchmarks — did it get slower?

```sh
cargo test -p turso_graph_runtime --test benchmark_shapes
cargo bench -p turso_graph_runtime --bench graph_shapes
cargo bench -p turso_graph_frontend --bench semantic_prepare
cargo run -q -p turso_graph_testkit -- performance smoke --no-record
```

### Writing a test for a role-model change

Positional role resolution — resolving a role by argument order instead of by
name or `RoleId` — is the recurring defect class in this area, and it passes any
test whose fixture declares roles in the order the query writes them. A change
to role handling needs both of these to hold:

1. Permute the role **names** in a fixture without changing their order, and
   permute the **argument order** without changing names. Behavior must be
   unchanged.
2. Sabotage the resolution and watch a specific named test go red. A review
   that only reads the code does not catch this; one that breaks the code does.

## Reference

- [`docs/graph-internals.md`](graph-internals.md) — implementation map: the
  read and write pipelines, the role model's invariants, catalog and snapshot
  design, where to change what, and future work
- `graph/README.md` — crate layout and quickstart
- `graph/DESIGN_DECISIONS.md` — storage overlay, catalog, snapshot design, and
  the conformance failure taxonomy
- `graph/CONFORMANCE.md` + `graph/test-results/REPORT.md` — Cypher
  conformance corpus and current pass rates
- `graph/PROVENANCE.md` — pinned donor sources and licenses; binding before
  importing adapted material
- [`docs/graph-frontend-core-alignment.md`](graph-frontend-core-alignment.md) —
  where the graph frontend diverges from core's frontend model
- `docs/archive/plans/2026-07-21-graph-frontend-api-alignment.md` — how
  the consumer API reached its current baseline-aligned shape
