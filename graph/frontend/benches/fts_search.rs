use std::sync::Arc;

use divan::{black_box, Bencher};
use turso_graph_frontend::{
    core::{Database, MemoryIO, OpenOptions, SqliteDialect},
    register_graph, GraphConnection, GraphFtsEntityKind, GraphFtsIndexSpec, GraphFtsTokenizer,
    GraphRegistration, NodeSourceRegistration, ParameterTypes, Parameters,
};
use turso_graph_ir::{Nullability, ValueType};

const CORPUS_ROWS: usize = 10_000;
const MATCHING_ROWS: usize = 100;
const QUERY: &str = "needle";
const FTS_QUERY: &str = "MATCH (n) WHERE fts_match(n.body, $query) RETURN n.id LIMIT 20";
const SCAN_QUERY: &str = "MATCH (n) WHERE n.body CONTAINS $query RETURN n.id LIMIT 20";

fn main() {
    divan::main();
}

fn database() -> Arc<Database> {
    let database = Database::open(
        Arc::new(MemoryIO::new()),
        ":memory:graph-fts-search-bench",
        OpenOptions::new(Arc::new(SqliteDialect))
            .db_opts(turso_graph_frontend::DatabaseOpts::default().with_index_method(true)),
    )
    .expect("open benchmark database");
    let connection = database.connect().expect("connect benchmark database");
    connection
        .execute("CREATE TABLE documents(id INTEGER PRIMARY KEY, body TEXT)")
        .expect("create document source");
    for start in (0..CORPUS_ROWS).step_by(250) {
        let values = (start..(start + 250).min(CORPUS_ROWS))
            .map(|index| {
                let body = if index % (CORPUS_ROWS / MATCHING_ROWS) == 0 {
                    format!("{QUERY} document {index}")
                } else {
                    format!("ordinary document {index}")
                };
                format!("({index}, '{body}')")
            })
            .collect::<Vec<_>>()
            .join(",");
        connection
            .execute(format!("INSERT INTO documents VALUES {values}"))
            .expect("seed benchmark corpus");
    }
    register_graph(
        &connection,
        &GraphRegistration {
            name: "documents".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Document".to_owned(),
                table: "documents".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: Vec::new(),
        },
    )
    .expect("register benchmark graph");
    let session = open_session(&database);
    session
        .create_fts_index(&GraphFtsIndexSpec {
            name: "body_search".to_owned(),
            entity: GraphFtsEntityKind::Node,
            source: "Document".to_owned(),
            properties: vec!["body".to_owned()],
            tokenizer: GraphFtsTokenizer::Default,
            weights: Vec::new(),
        })
        .expect("create benchmark FTS index");
    database
}

fn open_session(database: &Arc<Database>) -> GraphConnection {
    GraphConnection::open_with_parameters(
        database.connect().expect("connect graph benchmark"),
        "documents",
        ParameterTypes::from([("query".to_owned(), (ValueType::Text, Nullability::NonNull))]),
    )
    .expect("open graph benchmark")
}

fn parameters() -> Parameters {
    Parameters::from([(
        "query".to_owned(),
        turso_graph_frontend::Value::build_text(QUERY),
    )])
}

/// 10,000 rows, 1% selectivity, LIMIT 20, warm session and FTS reader.
#[turso_macros::divan_bench]
fn indexed_warm_10k_1pct_limit20(bencher: Bencher) {
    let database = database();
    let session = open_session(&database);
    let parameters = parameters();
    session
        .query(FTS_QUERY, &parameters)
        .expect("warm FTS benchmark");
    bencher.bench_local(|| {
        black_box(
            session
                .query(FTS_QUERY, &parameters)
                .expect("run warm FTS benchmark"),
        )
    });
}

/// 10,000 rows, 1% selectivity, LIMIT 20, new graph/core cursor per sample.
#[turso_macros::divan_bench]
fn indexed_session_cold_10k_1pct_limit20(bencher: Bencher) {
    let database = database();
    let parameters = parameters();
    bencher.bench_local(|| {
        let session = open_session(&database);
        black_box(
            session
                .query(FTS_QUERY, &parameters)
                .expect("run session-cold FTS benchmark"),
        )
    });
}

/// Non-indexed control over the same 10,000-row corpus and result limit.
#[turso_macros::divan_bench]
fn contains_scan_10k_1pct_limit20(bencher: Bencher) {
    let database = database();
    let session = open_session(&database);
    let parameters = parameters();
    bencher.bench_local(|| {
        black_box(
            session
                .query(SCAN_QUERY, &parameters)
                .expect("run scan control"),
        )
    });
}
