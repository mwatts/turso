use std::sync::Arc;

use divan::{black_box, Bencher};
use turso_graph_frontend::{
    core::{Database, MemoryIO, SqliteDialect},
    register_graph, register_semantic_schema, GraphConnection, GraphRegistration,
    NodeSourceRegistration, RelationshipSourceRegistration, SemanticNodeType, SemanticProperty,
    SemanticSchemaRegistration,
};

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::main();
}

fn database(semantic: bool) -> Arc<Database> {
    let database = Database::open_file(
        Arc::new(MemoryIO::new()),
        ":memory:semantic-prepare-bench",
        Arc::new(SqliteDialect),
    )
    .expect("open benchmark database");
    let connection = database.connect().expect("connect benchmark database");
    connection
        .execute(
            "CREATE TABLE people(\
                 id INTEGER PRIMARY KEY, \
                 full_name TEXT, \
                 birth_year INTEGER\
             ); \
             CREATE TABLE relationships(\
                 id INTEGER PRIMARY KEY, \
                 start_id INTEGER, \
                 end_id INTEGER\
             );",
        )
        .expect("create benchmark sources");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "people_src".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration {
                name: "relationships_src".to_owned(),
                table: "relationships".to_owned(),
                identity_column: "id".to_owned(),
                start_column: "start_id".to_owned(),
                end_column: "end_id".to_owned(),
                start_node_source: "people_src".to_owned(),
                end_node_source: "people_src".to_owned(),
            }],
        },
    )
    .expect("register benchmark graph");
    if semantic {
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
        .expect("register benchmark semantic schema");
    }
    database
}

fn multi_source_database() -> Arc<Database> {
    let database = Database::open_file(
        Arc::new(MemoryIO::new()),
        ":memory:multi-source-semantic-prepare-bench",
        Arc::new(SqliteDialect),
    )
    .expect("open benchmark database");
    let connection = database.connect().expect("connect benchmark database");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY, display_name TEXT); \
             CREATE TABLE companies(id INTEGER PRIMARY KEY, legal_name TEXT); \
             CREATE TABLE employment(\
                 id INTEGER PRIMARY KEY, person_id INTEGER, company_id INTEGER\
             ); \
             CREATE TABLE ownership(\
                 id INTEGER PRIMARY KEY, company_id INTEGER, person_id INTEGER\
             );",
        )
        .expect("create multi-source benchmark tables");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![
                NodeSourceRegistration {
                    name: "people_src".to_owned(),
                    table: "people".to_owned(),
                    identity_column: "id".to_owned(),
                },
                NodeSourceRegistration {
                    name: "companies_src".to_owned(),
                    table: "companies".to_owned(),
                    identity_column: "id".to_owned(),
                },
            ],
            relationship_sources: vec![
                RelationshipSourceRegistration {
                    name: "employment_src".to_owned(),
                    table: "employment".to_owned(),
                    identity_column: "id".to_owned(),
                    start_column: "person_id".to_owned(),
                    end_column: "company_id".to_owned(),
                    start_node_source: "people_src".to_owned(),
                    end_node_source: "companies_src".to_owned(),
                },
                RelationshipSourceRegistration {
                    name: "ownership_src".to_owned(),
                    table: "ownership".to_owned(),
                    identity_column: "id".to_owned(),
                    start_column: "company_id".to_owned(),
                    end_column: "person_id".to_owned(),
                    start_node_source: "companies_src".to_owned(),
                    end_node_source: "people_src".to_owned(),
                },
            ],
        },
    )
    .expect("register multi-source benchmark graph");
    register_semantic_schema(
        &connection,
        "social",
        &SemanticSchemaRegistration {
            node_types: vec![
                SemanticNodeType {
                    name: "Person".to_owned(),
                    source: "people_src".to_owned(),
                    properties: vec![SemanticProperty {
                        name: "displayName".to_owned(),
                        column: "display_name".to_owned(),
                    }],
                },
                SemanticNodeType {
                    name: "Company".to_owned(),
                    source: "companies_src".to_owned(),
                    properties: vec![SemanticProperty {
                        name: "displayName".to_owned(),
                        column: "legal_name".to_owned(),
                    }],
                },
            ],
            relationship_types: Vec::new(),
        },
    )
    .expect("register multi-source semantic schema");
    database
}

#[turso_macros::divan_bench]
fn open_legacy(bencher: Bencher) {
    let database = database(false);
    bencher.bench_local(|| {
        let connection = database.connect().expect("connect legacy graph");
        black_box(GraphConnection::open(connection, "social").expect("open legacy graph"));
    });
}

#[turso_macros::divan_bench]
fn open_semantic(bencher: Bencher) {
    let database = database(true);
    bencher.bench_local(|| {
        let connection = database.connect().expect("connect semantic graph");
        black_box(GraphConnection::open(connection, "social").expect("open semantic graph"));
    });
}

#[turso_macros::divan_bench]
fn open_semantic_multi_source(bencher: Bencher) {
    let database = multi_source_database();
    bencher.bench_local(|| {
        let connection = database.connect().expect("connect semantic graph");
        black_box(GraphConnection::open(connection, "social").expect("open semantic graph"));
    });
}

#[turso_macros::divan_bench]
fn prepare_legacy(bencher: Bencher) {
    let database = database(false);
    let session =
        GraphConnection::open(database.connect().expect("connect"), "social").expect("open graph");
    bencher.bench_local(|| {
        black_box(
            session
                .prepare(
                    black_box("MATCH (n:people_src) RETURN n.full_name"),
                    &Default::default(),
                )
                .expect("prepare legacy query"),
        );
    });
}

#[turso_macros::divan_bench]
fn prepare_semantic(bencher: Bencher) {
    let database = database(true);
    let session =
        GraphConnection::open(database.connect().expect("connect"), "social").expect("open graph");
    bencher.bench_local(|| {
        black_box(
            session
                .prepare(
                    black_box("MATCH (n:Person) RETURN n.displayName"),
                    &Default::default(),
                )
                .expect("prepare semantic query"),
        );
    });
}

#[turso_macros::divan_bench]
fn prepare_semantic_unlabeled_single_source(bencher: Bencher) {
    let database = database(true);
    let session =
        GraphConnection::open(database.connect().expect("connect"), "social").expect("open graph");
    bencher.bench_local(|| {
        black_box(
            session
                .prepare(
                    black_box("MATCH (n) RETURN n.displayName"),
                    &Default::default(),
                )
                .expect("prepare single-source semantic query"),
        );
    });
}

#[turso_macros::divan_bench]
fn prepare_semantic_unlabeled_multi_source(bencher: Bencher) {
    let database = multi_source_database();
    let session =
        GraphConnection::open(database.connect().expect("connect"), "social").expect("open graph");
    bencher.bench_local(|| {
        black_box(
            session
                .prepare(
                    black_box("MATCH (n) RETURN n.displayName"),
                    &Default::default(),
                )
                .expect("prepare multi-source semantic query"),
        );
    });
}
