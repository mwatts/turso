//! A row write must not reload the graph catalog.
//!
//! Writing a row bumps the data generation (the AFTER-DML triggers on mapped
//! source tables), which traversal snapshots need. It must not bump the
//! catalog's own generation, because reloading the catalog recompiles a dozen
//! internal statements and throws away the Cypher compile cache — per
//! mutation. Catalog reloads belong to catalog changes only.

use std::sync::{Arc, Mutex};

use tracing::field::{Field, Visit};
use tracing_subscriber::layer::{Context, Layer, SubscriberExt};
use turso_graph_frontend::{
    core::{Database, MemoryIO, SqliteDialect},
    graph_generation, register_graph, register_semantic_constraints, register_semantic_schema,
    GraphConnection, GraphRegistration, NodeSourceRegistration, Parameters,
    RelationshipSourceRegistration, SemanticConstraintRegistration, SemanticNodeType,
    SemanticProperty, SemanticRequiredProperty, SemanticSchemaRegistration,
};

/// Internal tables whose appearance in a prepared statement means the session
/// went back to the catalog.
const CATALOG_TABLES: [&str; 4] = [
    "__turso_internal_graph_sources",
    "__turso_internal_graph_node_sources",
    "__turso_internal_graph_semantic_types",
    "__turso_internal_graph_semantic_ownership",
];

/// Records the SQL that core compiles, by watching the `Preparing: {sql}`
/// debug event `prepare_with_origin` emits.
#[derive(Clone, Default)]
struct PreparedSql(Arc<Mutex<Vec<String>>>);

impl PreparedSql {
    fn take(&self) -> Vec<String> {
        std::mem::take(&mut *self.0.lock().unwrap())
    }
}

struct FindPrepare(Option<String>);

impl Visit for FindPrepare {
    fn record_debug(&mut self, _field: &Field, value: &dyn std::fmt::Debug) {
        let text = format!("{value:?}");
        if let Some(sql) = text.strip_prefix("Preparing: ") {
            self.0 = Some(sql.to_owned());
        }
    }
}

impl<S: tracing::Subscriber> Layer<S> for PreparedSql {
    fn on_event(&self, event: &tracing::Event<'_>, _context: Context<'_, S>) {
        let mut found = FindPrepare(None);
        event.record(&mut found);
        if let Some(sql) = found.0 {
            self.0.lock().unwrap().push(sql);
        }
    }
}

fn graph_database() -> Arc<Database> {
    let database = Database::open_file(
        Arc::new(MemoryIO::new()),
        ":memory:catalog-refresh",
        Arc::new(SqliteDialect),
    )
    .expect("open database");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY, full_name TEXT, birth_year INTEGER); \
             CREATE TABLE relationships(id INTEGER PRIMARY KEY, start_id INTEGER, end_id INTEGER);",
        )
        .expect("create source tables");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "people_src".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "relationships_src",
                "relationships",
                "id",
                "start_id",
                "end_id",
                "people_src",
                "people_src",
            )],
        },
    )
    .expect("register graph");
    register_semantic_schema(
        &connection,
        "social",
        &SemanticSchemaRegistration {
            node_types: vec![SemanticNodeType {
                name: "Person".to_owned(),
                source: "people_src".to_owned(),
                properties: vec![
                    SemanticProperty {
                        name: "displayName".to_owned(),
                        column: "full_name".to_owned(),
                    },
                    SemanticProperty {
                        name: "born".to_owned(),
                        column: "birth_year".to_owned(),
                    },
                ],
            }],
            relationship_types: Vec::new(),
        },
    )
    .expect("register semantic schema");
    database
}

fn create(session: &GraphConnection, name: &str) {
    session
        .execute(
            &format!("CREATE (:Person {{displayName: '{name}', born: 1900}})"),
            &Parameters::new(),
        )
        .expect("create person");
}

#[test]
fn writing_a_row_does_not_send_the_session_back_to_the_catalog() {
    let recorded = PreparedSql::default();
    let subscriber = tracing_subscriber::registry().with(recorded.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

    let database = graph_database();
    let session = GraphConnection::open(database.connect().expect("connect"), "social")
        .expect("open graph session");

    // The first mutation is allowed to warm whatever it needs.
    create(&session, "warm");
    recorded.take();

    create(&session, "steady");
    let prepared = recorded.take();
    let catalog_reads = prepared
        .iter()
        .filter(|sql| CATALOG_TABLES.iter().any(|table| sql.contains(table)))
        .collect::<Vec<_>>();
    assert!(
        catalog_reads.is_empty(),
        "a row write reloaded the catalog; it must only re-read the generation counter. \
         Catalog statements compiled: {catalog_reads:#?}"
    );

    // The cost of a mutation must not grow with how many came before it.
    let mut counts = Vec::new();
    for index in 0..5 {
        create(&session, &format!("bulk{index}"));
        counts.push(recorded.take().len());
    }
    assert!(
        counts.windows(2).all(|pair| pair[0] == pair[1]),
        "every mutation must compile the same number of statements, got {counts:?}"
    );
}

#[test]
fn a_row_write_advances_the_data_generation_but_not_the_schema_generation() {
    let database = graph_database();
    let connection = database.connect().expect("connect");
    let session = GraphConnection::open(connection.clone(), "social").expect("open graph session");

    let schema_generation = || {
        let rows = connection
            .prepare("SELECT schema_generation FROM __turso_internal_graph_generations")
            .expect("prepare schema generation")
            .run_collect_rows()
            .expect("read schema generation");
        format!("{rows:?}")
    };

    let data_before = graph_generation(&connection, "social").expect("data generation");
    let schema_before = schema_generation();

    create(&session, "Ada");

    assert!(
        graph_generation(&connection, "social").expect("data generation") > data_before,
        "a row write must advance the data generation so snapshots rebuild"
    );
    assert_eq!(
        schema_generation(),
        schema_before,
        "a row write must leave the schema generation alone"
    );
}

#[test]
fn registering_semantic_constraints_advances_the_schema_generation() {
    let database = graph_database();
    let connection = database.connect().expect("connect");

    let schema_generation = || {
        let rows = connection
            .prepare("SELECT schema_generation FROM __turso_internal_graph_generations")
            .expect("prepare schema generation")
            .run_collect_rows()
            .expect("read schema generation");
        format!("{rows:?}")
    };
    let before = schema_generation();

    register_semantic_constraints(
        &connection,
        "social",
        &SemanticConstraintRegistration {
            required: vec![SemanticRequiredProperty {
                owner: "Person".to_owned(),
                property: "displayName".to_owned(),
            }],
            ..SemanticConstraintRegistration::default()
        },
    )
    .expect("register semantic constraints");

    assert_ne!(
        schema_generation(),
        before,
        "a catalog change must advance the schema generation so open sessions reload"
    );
}
