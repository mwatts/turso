//! Shared `GraphConnection` fixture for `graph_frontend` integration tests.
//!
//! Mirrors the "social" graph (`Person` nodes over `people`, `KNOWS`
//! relationships over `relationships`) `session.rs`'s own unit tests
//! install, but backed by `SchemaCatalog` rather than a private catalog
//! stub so this can run as an external integration-test crate.

use std::sync::Arc;

use turso_core::{Connection, Database, DatabaseOpts, MemoryIO, OpenOptions, SqliteDialect};
use turso_graph_cypher::parse;
use turso_graph_frontend::{
    bind, load_registered_graph, lower_relational, register_graph, CatalogEntity,
    GraphCatalogSnapshot, GraphCompilationCatalog, GraphConnection, GraphRegistration,
    NodeSourceRegistration, NodeTableLayout, ParameterTypes, Parameters, RelationalCatalogSnapshot,
    RelationshipRoleLayout, RelationshipSourceRegistration, RelationshipTableLayout,
    ResolvedProperty, RoleSourceRegistration, SchemaCatalog, SnapshotStore,
};
use turso_graph_ir::{
    GraphId, LabelId, Nullability, Plan, PlanKind, PropertyId, RelationshipTypeId, RoleCardinality,
    RoleExpand, RoleId, SourceTableId, ValueType,
};
use turso_graph_runtime::{BuildLimits, NeverCancelled};

/// Installs a `GraphConnection` over a fresh in-memory "social" graph:
/// `Person` nodes (`people(id, name, age)`), `KNOWS` relationships
/// (`relationships(id, src, dst)`), seeded with two people. Returns the
/// `Arc<Database>` alongside the session so callers can open further
/// connections onto the same graph (see [`second_connection`]).
#[allow(dead_code)] // This file is also compiled as its own integration-test crate.
pub fn social_graph_connection() -> (Arc<Database>, GraphConnection) {
    social_graph_connection_with_options(DatabaseOpts::default())
}

#[cfg(feature = "fts")]
#[allow(dead_code)] // This file is also compiled as its own integration-test crate.
pub fn social_graph_connection_with_fts() -> (Arc<Database>, GraphConnection) {
    social_graph_connection_with_options(DatabaseOpts::default().with_index_method(true))
}

fn social_graph_connection_with_options(opts: DatabaseOpts) -> (Arc<Database>, GraphConnection) {
    let io = Arc::new(MemoryIO::new());
    let database = Database::open(
        io,
        ":memory:fixture-social",
        OpenOptions::new(Arc::new(SqliteDialect)).db_opts(opts),
    )
    .expect("open database");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, age INTEGER); \
             CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);",
        )
        .expect("create sources");
    let registered = register_graph(
        &connection,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "KNOWS",
                "relationships",
                "id",
                "src",
                "dst",
                "Person",
                "Person",
            )],
        },
    )
    .expect("register graph");
    let catalog: Arc<dyn GraphCompilationCatalog> =
        Arc::new(SchemaCatalog::new(connection.clone(), registered.clone()));
    let shared_snapshots = Arc::new(SnapshotStore::default());
    shared_snapshots
        .refresh(
            &connection,
            &registered.name,
            BuildLimits::default(),
            &NeverCancelled,
        )
        .expect("build initial traversal snapshot");
    let session = GraphConnection::install(
        connection,
        &registered,
        catalog,
        ParameterTypes::new(),
        shared_snapshots,
        BuildLimits::default(),
    )
    .expect("install graph session");
    // Seeded through Cypher CREATE (not a raw INSERT) so the label junction
    // table `SchemaCatalog` relies on is populated the same way production
    // writes populate it.
    session
        .execute(
            "CREATE (:Person {id: 1, name: 'Ada', age: 36}), \
             (:Person {id: 2, name: 'Grace', age: 85})",
            &Parameters::new(),
        )
        .expect("seed people");
    (database, session)
}

/// Installs a `GraphConnection` over a fresh in-memory three-role
/// "scriptorium" graph: `Person`/`Text`/`Folio` node sources and a
/// `Transcription` relationship source with three single-valued roles
/// (`scribe` -> `people`, `text` -> `texts`, `folio` -> `folios`, columns
/// `scribe`/`txt`/`folio` on `transcriptions`), mirroring
/// `catalog.rs`'s `a_three_role_registration_indexes_every_role_and_every_ordered_pair`
/// fixture plus a `year` property on the relation. No rows are pre-seeded:
/// `SchemaCatalog` without a semantic schema only resolves node properties
/// when the graph has exactly one node source, which this graph (three node
/// sources) does not have, so seeding through Cypher `CREATE (:Person {..})`
/// is not available here. Relationship-property resolution still works
/// without a semantic schema because this graph registers exactly one
/// relationship source.
#[allow(dead_code)] // This file is also compiled as its own integration-test crate.
pub fn ternary_session() -> (Arc<Database>, GraphConnection) {
    let io = Arc::new(MemoryIO::new());
    let database = Database::open(
        io,
        ":memory:fixture-scriptorium",
        OpenOptions::new(Arc::new(SqliteDialect)),
    )
    .expect("open database");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY); \
             CREATE TABLE texts(id INTEGER PRIMARY KEY); \
             CREATE TABLE folios(id INTEGER PRIMARY KEY); \
             CREATE TABLE transcriptions(\
                 id INTEGER PRIMARY KEY, scribe INTEGER, txt INTEGER, folio INTEGER, \
                 year INTEGER);",
        )
        .expect("create sources");
    let registered = register_graph(
        &connection,
        &GraphRegistration {
            name: "scriptorium".to_owned(),
            node_sources: vec![
                NodeSourceRegistration {
                    name: "Person".to_owned(),
                    table: "people".to_owned(),
                    identity_column: "id".to_owned(),
                },
                NodeSourceRegistration {
                    name: "Text".to_owned(),
                    table: "texts".to_owned(),
                    identity_column: "id".to_owned(),
                },
                NodeSourceRegistration {
                    name: "Folio".to_owned(),
                    table: "folios".to_owned(),
                    identity_column: "id".to_owned(),
                },
            ],
            relationship_sources: vec![RelationshipSourceRegistration {
                name: "Transcription".to_owned(),
                table: "transcriptions".to_owned(),
                identity_column: "id".to_owned(),
                roles: vec![
                    RoleSourceRegistration {
                        name: "scribe".to_owned(),
                        column: "scribe".to_owned(),
                        node_source: "Person".to_owned(),
                        cardinality: RoleCardinality::One,
                    },
                    RoleSourceRegistration {
                        name: "text".to_owned(),
                        column: "txt".to_owned(),
                        node_source: "Text".to_owned(),
                        cardinality: RoleCardinality::One,
                    },
                    RoleSourceRegistration {
                        name: "folio".to_owned(),
                        column: "folio".to_owned(),
                        node_source: "Folio".to_owned(),
                        cardinality: RoleCardinality::One,
                    },
                ],
            }],
        },
    )
    .expect("register graph");
    let catalog: Arc<dyn GraphCompilationCatalog> =
        Arc::new(SchemaCatalog::new(connection.clone(), registered.clone()));
    // Unlike `social_graph_connection`, this does not eagerly build a
    // traversal snapshot: none of this fixture's tests run a Cypher
    // traversal that would need one, so the store builds lazily on demand
    // for whichever caller actually needs it. (The snapshot builder itself
    // is no longer binary-only -- it derives edges from every ordered pair
    // of single-valued roles, plus each single-valued/`Many` pair -- this is
    // just this fixture not exercising that path.)
    let shared_snapshots = Arc::new(SnapshotStore::default());
    let session = GraphConnection::install(
        connection,
        &registered,
        catalog,
        ParameterTypes::new(),
        shared_snapshots,
        BuildLimits::default(),
    )
    .expect("install graph session");
    (database, session)
}

/// Installs a `GraphConnection` over a fresh in-memory "witnessed" graph: a
/// single `Person` node source (`people(id INTEGER PRIMARY KEY)`) and a
/// `KNOWS` relationship source over `relationships(id INTEGER PRIMARY KEY,
/// src INTEGER, dst INTEGER)` with three roles -- `start`/`end` (single-valued,
/// the pattern-hop shape) plus `witness` (many-valued, no column of its own).
///
/// Binary-plus-`Many` is deliberate: it is the only role shape that lets a
/// relation be both created through the standalone role pattern (Task 13a,
/// `CREATE [x:KNOWS](start: a, end: b, witness: w)`) and bound for deletion
/// through today's arrow syntax (`MATCH (a:Person)-[r:KNOWS]->(b:Person)
/// DELETE r`), with no dependency on the standalone role pattern in `MATCH`
/// (Task 13b, not yet implemented). A ternary relation can be created but not
/// bound for deletion until then.
///
/// Modeled on `ternary_session`, not `social_graph_connection`: no eager
/// `SnapshotStore::refresh`, since this task's tests never run a
/// variable-length traversal that would need one -- the snapshot builder
/// itself supports `One`/`Many` role pairs like `witness` fine. Unlike
/// `ternary_session`,
/// this graph has exactly one node source, so `SchemaCatalog` can resolve
/// node properties and tests may seed through Cypher `CREATE (:Person {id:
/// ..})` directly, the same way `social_graph_connection` does.
#[allow(dead_code)] // This file is also compiled as its own integration-test crate.
pub fn witnessed_session() -> (Arc<Database>, GraphConnection) {
    let io = Arc::new(MemoryIO::new());
    let database = Database::open(
        io,
        ":memory:fixture-witnessed",
        OpenOptions::new(Arc::new(SqliteDialect)),
    )
    .expect("open database");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY); \
             CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);",
        )
        .expect("create sources");
    let registered = register_graph(
        &connection,
        &GraphRegistration {
            name: "witnessed".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration {
                name: "KNOWS".to_owned(),
                table: "relationships".to_owned(),
                identity_column: "id".to_owned(),
                roles: vec![
                    RoleSourceRegistration {
                        name: "start".to_owned(),
                        column: "src".to_owned(),
                        node_source: "Person".to_owned(),
                        cardinality: RoleCardinality::One,
                    },
                    RoleSourceRegistration {
                        name: "end".to_owned(),
                        column: "dst".to_owned(),
                        node_source: "Person".to_owned(),
                        cardinality: RoleCardinality::One,
                    },
                    RoleSourceRegistration {
                        name: "witness".to_owned(),
                        // Empty for `Many` roles: their players live in the
                        // spill table, not a column on the relation table.
                        column: String::new(),
                        node_source: "Person".to_owned(),
                        cardinality: RoleCardinality::Many,
                    },
                ],
            }],
        },
    )
    .expect("register graph");
    let catalog: Arc<dyn GraphCompilationCatalog> =
        Arc::new(SchemaCatalog::new(connection.clone(), registered.clone()));
    let shared_snapshots = Arc::new(SnapshotStore::default());
    let session = GraphConnection::install(
        connection,
        &registered,
        catalog,
        ParameterTypes::new(),
        shared_snapshots,
        BuildLimits::default(),
    )
    .expect("install graph session");
    (database, session)
}

/// Installs a `GraphConnection` over a graph whose only relationship type
/// has two independently `Many`-cardinality roles and no `One`-cardinality
/// role at all: `Person` nodes (`people(id)`), a `GATHERING` relationship
/// source over `gatherings(id)` with roles `guest` and `witness`, both
/// `Many`, each spilling into its own table (no shared endpoint column,
/// since neither role has one -- `insert_entity`'s `columns.is_empty()`
/// branch, `INSERT INTO gatherings DEFAULT VALUES`, is what makes a relation
/// row with no `One` role writable at all).
///
/// Exists solely to test what naming two `Many` roles in the same hop does:
/// `witnessed_session` only ever has one `Many` role live at a time (plus
/// two `One` roles), so it cannot exercise the case where two independent
/// multi-valued sets are joined into one row set.
#[allow(dead_code)] // This file is also compiled as its own integration-test crate.
pub fn two_many_roles_session() -> (Arc<Database>, GraphConnection) {
    let io = Arc::new(MemoryIO::new());
    let database = Database::open(
        io,
        ":memory:fixture-gathering",
        OpenOptions::new(Arc::new(SqliteDialect)),
    )
    .expect("open database");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY); \
             CREATE TABLE gatherings(id INTEGER PRIMARY KEY);",
        )
        .expect("create sources");
    let registered = register_graph(
        &connection,
        &GraphRegistration {
            name: "gathering".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration {
                name: "GATHERING".to_owned(),
                table: "gatherings".to_owned(),
                identity_column: "id".to_owned(),
                roles: vec![
                    RoleSourceRegistration {
                        name: "guest".to_owned(),
                        column: String::new(),
                        node_source: "Person".to_owned(),
                        cardinality: RoleCardinality::Many,
                    },
                    RoleSourceRegistration {
                        name: "witness".to_owned(),
                        column: String::new(),
                        node_source: "Person".to_owned(),
                        cardinality: RoleCardinality::Many,
                    },
                ],
            }],
        },
    )
    .expect("register graph");
    let catalog: Arc<dyn GraphCompilationCatalog> =
        Arc::new(SchemaCatalog::new(connection.clone(), registered.clone()));
    let shared_snapshots = Arc::new(SnapshotStore::default());
    let session = GraphConnection::install(
        connection,
        &registered,
        catalog,
        ParameterTypes::new(),
        shared_snapshots,
        BuildLimits::default(),
    )
    .expect("install graph session");
    (database, session)
}

/// Installs a `GraphConnection` over a `KNOWS` shape close to
/// `witnessed_session` (`Person`/`people`; `KNOWS`/`relationships` with roles
/// `start`/`end`/`witness`) plus a second, unrelated relationship source
/// literally named `witness`, over its own table (`witnesses`). `witness` is
/// then simultaneously a role of `KNOWS` and a relationship type in its own
/// right -- the shape Rule B's `AmbiguousRoleName` check exists to refuse
/// rather than silently guess at. `register_graph` accepts this cleanly
/// (role names and relationship-source names are validated in disjoint
/// namespaces) and `MATCH [x:KNOWS](witness: w)` already binds against it
/// through the standalone role pattern, unaffected by the second source.
///
/// `witness` is deliberately `One`-cardinality here (a real `witness` column
/// on `relationships`), not `Many` as in `witnessed_session`: a `Many`-role
/// `witness` would make this query also trip `bind_role_read_step`'s
/// separate Many-cardinality guard, so a test asserting on the ambiguity
/// error would still pass if the ambiguity check were removed (the
/// Many-cardinality guard would fire instead). `One`-cardinality means the
/// ambiguity check is the only thing standing between this query and a
/// successful bind.
#[allow(dead_code)] // This file is also compiled as its own integration-test crate.
pub fn ambiguous_session() -> (Arc<Database>, GraphConnection) {
    let io = Arc::new(MemoryIO::new());
    let database = Database::open(
        io,
        ":memory:fixture-ambiguous",
        OpenOptions::new(Arc::new(SqliteDialect)),
    )
    .expect("open database");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY); \
             CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER, \
                 witness INTEGER); \
             CREATE TABLE witnesses(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);",
        )
        .expect("create sources");
    let registered = register_graph(
        &connection,
        &GraphRegistration {
            name: "ambiguous".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![
                RelationshipSourceRegistration {
                    name: "KNOWS".to_owned(),
                    table: "relationships".to_owned(),
                    identity_column: "id".to_owned(),
                    roles: vec![
                        RoleSourceRegistration {
                            name: "start".to_owned(),
                            column: "src".to_owned(),
                            node_source: "Person".to_owned(),
                            cardinality: RoleCardinality::One,
                        },
                        RoleSourceRegistration {
                            name: "end".to_owned(),
                            column: "dst".to_owned(),
                            node_source: "Person".to_owned(),
                            cardinality: RoleCardinality::One,
                        },
                        RoleSourceRegistration {
                            name: "witness".to_owned(),
                            column: "witness".to_owned(),
                            node_source: "Person".to_owned(),
                            cardinality: RoleCardinality::One,
                        },
                    ],
                },
                RelationshipSourceRegistration::binary(
                    "witness",
                    "witnesses",
                    "id",
                    "src",
                    "dst",
                    "Person",
                    "Person",
                ),
            ],
        },
    )
    .expect("register graph");
    let catalog: Arc<dyn GraphCompilationCatalog> =
        Arc::new(SchemaCatalog::new(connection.clone(), registered.clone()));
    let shared_snapshots = Arc::new(SnapshotStore::default());
    let session = GraphConnection::install(
        connection,
        &registered,
        catalog,
        ParameterTypes::new(),
        shared_snapshots,
        BuildLimits::default(),
    )
    .expect("install graph session");
    (database, session)
}

/// A second connection onto the same underlying database as `database`, for
/// exercising session setup (like [`GraphConnection::open`]) that must not
/// depend on the connection that performed the original registration.
#[allow(dead_code)] // Shared fixture; not every integration crate calls this.
pub fn second_connection(database: &Arc<Database>) -> Arc<Connection> {
    database.connect().expect("connect")
}

/// A lightweight in-process `Person`/`KNOWS` catalog (role 1 = `start`, role
/// 2 = `end`) for tests that bind a query directly, without a real
/// `GraphConnection`.
#[allow(dead_code)] // Shared fixture; not every integration crate calls this.
struct Catalog;

impl GraphCatalogSnapshot for Catalog {
    fn node_source(&self, _graph: GraphId) -> Option<SourceTableId> {
        SourceTableId::new(1).ok()
    }

    fn relationship_source(&self, _graph: GraphId) -> Option<SourceTableId> {
        SourceTableId::new(2).ok()
    }

    fn label(&self, _graph: GraphId, _name: &str) -> Option<LabelId> {
        LabelId::new(1).ok()
    }

    fn relationship_type(&self, _graph: GraphId, _name: &str) -> Option<RelationshipTypeId> {
        RelationshipTypeId::new(1).ok()
    }

    fn property(
        &self,
        _graph: GraphId,
        _entity: CatalogEntity,
        name: &str,
    ) -> Option<ResolvedProperty> {
        let id = if name == "name" { 1 } else { 2 };
        Some(ResolvedProperty {
            id: PropertyId::new(id).ok()?,
            value_type: if name == "name" {
                ValueType::Text
            } else {
                ValueType::Integer
            },
            nullability: Nullability::Nullable,
        })
    }

    fn relationship_source_roles(&self, source: SourceTableId) -> Option<RelationshipTableLayout> {
        self.relationship_layout(source)
    }
}

impl RelationalCatalogSnapshot for Catalog {
    fn node_layout(&self, source: SourceTableId) -> Option<NodeTableLayout> {
        (source.get() == 1).then(|| NodeTableLayout {
            table: "people".to_owned(),
            identity_column: "id".to_owned(),
        })
    }

    fn relationship_layout(&self, source: SourceTableId) -> Option<RelationshipTableLayout> {
        (source.get() == 2).then(|| RelationshipTableLayout {
            table: "relationships".to_owned(),
            identity_column: "id".to_owned(),
            roles: vec![
                RelationshipRoleLayout {
                    role: RoleId::new(1).unwrap(),
                    name: "start".to_owned(),
                    column: "src".to_owned(),
                    cardinality: RoleCardinality::One,
                    spill_table: None,
                },
                RelationshipRoleLayout {
                    role: RoleId::new(2).unwrap(),
                    name: "end".to_owned(),
                    column: "dst".to_owned(),
                    cardinality: RoleCardinality::One,
                    spill_table: None,
                },
            ],
        })
    }

    fn property_column(&self, _source: SourceTableId, property: PropertyId) -> Option<String> {
        match property.get() {
            1 => Some("name".to_owned()),
            2 => Some("age".to_owned()),
            _ => None,
        }
    }
}

/// Binds `query` against [`Catalog`]'s `Person`/`KNOWS` shape.
#[allow(dead_code)] // Shared fixture; not every integration crate calls this.
pub fn bind_fixture(query: &str) -> Plan {
    let parsed = parse(query).expect("fixture query must parse");
    bind(
        &parsed,
        GraphId::new(1).expect("graph id"),
        &Catalog,
        &ParameterTypes::new(),
    )
    .expect("fixture query must bind")
    .plan
}

/// Binds `query` against [`witnessed_session`]'s real registered schema
/// (`SchemaCatalog`, loaded fresh through `load_registered_graph` -- not
/// [`bind_fixture`]'s stub `Catalog`, whose `label` and `relationship_type`
/// both return `Some` for *every* name, making every name simultaneously a
/// label, a type, and therefore ambiguous). Needed for plan-equality
/// assertions between different spellings of one relation-anchored pattern:
/// `ir::Plan` carries no connection state, only catalog-resolved ids, so two
/// separate calls (even against separately-opened databases) that register
/// the same graph shape produce comparable plans. `connection` must come
/// from a database that already ran `witnessed_session`'s `register_graph`
/// (e.g. via [`second_connection`]).
#[allow(dead_code)] // Shared fixture; not every integration crate calls this.
pub fn bind_witnessed(connection: &Arc<Connection>, query: &str) -> Plan {
    let registered =
        load_registered_graph(connection, "witnessed").expect("load witnessed registration");
    let catalog = SchemaCatalog::new(connection.clone(), registered.clone());
    let parsed = parse(query).expect("fixture query must parse");
    bind(&parsed, registered.id, &catalog, &ParameterTypes::new())
        .expect("fixture query must bind")
        .plan
}

/// Lowers a plan bound by [`bind_fixture`] to SQL against the same
/// `Catalog`, as text: the contract two differently-shaped plans (e.g. the
/// arrow and standalone-role-pattern forms of one relation) must share is
/// the SQL they lower to, not their `ir::Plan` shape.
#[allow(dead_code)] // Shared fixture; not every integration crate calls this.
pub fn lower_fixture(plan: &Plan) -> String {
    lower_relational(plan, &Catalog)
        .expect("fixture plan must lower")
        .to_string()
}

/// Depth-first walk to the first `RoleExpand` in a plan, following every
/// operator that carries an input.
#[allow(dead_code)] // Shared fixture; not every integration crate calls this.
pub fn first_role_expand(plan: &Plan) -> &RoleExpand {
    fn walk(plan: &Plan) -> Option<&RoleExpand> {
        match plan.kind() {
            PlanKind::RoleExpand(expand) => Some(expand),
            PlanKind::GraphExpand(expand) => walk(&expand.input),
            PlanKind::Unit(_) | PlanKind::NodeScan(_) => None,
            PlanKind::Filter(filter) => walk(&filter.input),
            PlanKind::Project(project) => walk(&project.input),
            PlanKind::Aggregate(aggregate) => walk(&aggregate.input),
            PlanKind::Distinct(distinct) => walk(&distinct.input),
            PlanKind::Sort(sort) => walk(&sort.input),
            PlanKind::Skip(skip) => walk(&skip.input),
            PlanKind::Limit(limit) => walk(&limit.input),
            PlanKind::LeftApply(left_apply) => {
                walk(&left_apply.left).or_else(|| walk(&left_apply.right))
            }
            PlanKind::Unwind(unwind) => walk(&unwind.input),
            PlanKind::ProcedureCall(call) => walk(&call.input),
            PlanKind::Union(union) => union.inputs().iter().find_map(walk),
            PlanKind::Join(join) => walk(&join.left).or_else(|| walk(&join.right)),
            PlanKind::RelationScan(_) => None,
            PlanKind::RoleJoin(join) => walk(&join.input),
        }
    }
    walk(plan).expect("plan must contain a RoleExpand")
}
