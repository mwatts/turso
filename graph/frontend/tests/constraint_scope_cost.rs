//! A mutation must not pay for constraints it cannot have broken.
//!
//! `validate_state` used to re-check every constraint in the graph after every
//! statement. That makes a bootstrap loop superlinear: installing N types costs
//! N mutations, and mutation number K rescans the source table of all K types
//! installed so far. A consuming product reported a 51-entity ontology install
//! that had not finished after 16 minutes against a 60 s install timeout.
//!
//! Validation is now narrowed to the source tables a statement writes. That the
//! narrowed constraints still fire is pinned in `constraint_validation_scope`;
//! what is measured here is that the cost stops growing.
//!
//! One test in this binary, by design: see `prepared_sql`.

mod prepared_sql;

use std::sync::Arc;

use prepared_sql::PreparedSql;
use turso_graph_frontend::{
    core::{Database, MemoryIO, SqliteDialect},
    register_graph, register_semantic_constraints, register_semantic_schema, GraphConnection,
    GraphRegistration, NodeSourceRegistration, Parameters, SemanticConstraintRegistration,
    SemanticNodeType, SemanticProperty, SemanticRequiredProperty, SemanticSchemaRegistration,
    SemanticUniqueProperty,
};

/// A graph of `types` unrelated node types, each on its own table, each with a
/// required and a unique property. This is the shape of an ontology install:
/// many types that share nothing, written one at a time.
fn graph_with_unrelated_types(types: usize) -> Arc<Database> {
    let database = Database::open_file(
        Arc::new(MemoryIO::new()),
        &format!(":memory:validation-scope-{types}"),
        Arc::new(SqliteDialect),
    )
    .expect("open database");
    let connection = database.connect().expect("connect");
    for index in 0..types {
        connection
            .execute(format!(
                "CREATE TABLE t{index}(id INTEGER PRIMARY KEY, name TEXT)"
            ))
            .expect("create source table");
    }
    register_graph(
        &connection,
        &GraphRegistration {
            name: "ontology".to_owned(),
            node_sources: (0..types)
                .map(|index| NodeSourceRegistration {
                    name: format!("src{index}"),
                    table: format!("t{index}"),
                    identity_column: "id".to_owned(),
                })
                .collect(),
            relationship_sources: Vec::new(),
        },
    )
    .expect("register graph");
    register_semantic_schema(
        &connection,
        "ontology",
        &SemanticSchemaRegistration {
            node_types: (0..types)
                .map(|index| SemanticNodeType {
                    name: format!("Type{index}"),
                    source: format!("src{index}"),
                    properties: vec![SemanticProperty {
                        name: "name".to_owned(),
                        column: "name".to_owned(),
                    }],
                })
                .collect(),
            relationship_types: Vec::new(),
        },
    )
    .expect("register semantic schema");
    register_semantic_constraints(
        &connection,
        "ontology",
        &SemanticConstraintRegistration {
            required: (0..types)
                .map(|index| SemanticRequiredProperty {
                    owner: format!("Type{index}"),
                    property: "name".to_owned(),
                })
                .collect(),
            unique: (0..types)
                .map(|index| SemanticUniqueProperty {
                    owner: format!("Type{index}"),
                    property: "name".to_owned(),
                })
                .collect(),
            ..SemanticConstraintRegistration::default()
        },
    )
    .expect("register semantic constraints");
    database
}

/// SQL compiled by one steady-state `CREATE (:Type0 …)` against a graph of
/// `types` types. The first mutation warms the session, so it is excluded.
fn statements_per_create(recorded: &PreparedSql, types: usize) -> Vec<String> {
    let database = graph_with_unrelated_types(types);
    let session = GraphConnection::open(database.connect().expect("connect"), "ontology")
        .expect("open graph session");

    session
        .execute("CREATE (:Type0 {name: 'warm'})", &Parameters::new())
        .expect("warm create");
    recorded.take();

    session
        .execute("CREATE (:Type0 {name: 'steady'})", &Parameters::new())
        .expect("steady create");
    recorded.take()
}

/// Statements reading a type source table other than `t0`. `t1` and friends
/// belong to types the statement did not touch, so a validation pass that is
/// properly scoped never names them.
fn foreign_table_reads(statements: &[String]) -> Vec<&String> {
    statements
        .iter()
        .filter(|sql| {
            (1..64).any(|index| {
                sql.contains(&format!("\"t{index}\"")) || sql.contains(&format!(" t{index} "))
            })
        })
        .collect()
}

#[test]
fn a_create_costs_the_same_no_matter_how_many_other_types_exist() {
    let recorded = PreparedSql::install();
    let few = statements_per_create(&recorded, 2);
    let many = statements_per_create(&recorded, 16);

    // Guards the assertions below against passing because nothing was
    // recorded — an empty recording satisfies both of them.
    // Cell rail: steady CREATE often reuses a cached `INSERT INTO t0` and only
    // recompiles prop_dict / node_props statements. Accept either shape.
    assert!(
        many.iter().any(|sql| {
            sql.contains("\"t0\"")
                || sql.contains(" t0 ")
                || sql.contains("node_props")
                || sql.contains("prop_dict")
        }),
        "expected the CREATE to compile SQL against its own source (t0 or Cell tables), but \
         recorded {many:#?}"
    );

    let foreign = foreign_table_reads(&many);
    assert!(
        foreign.is_empty(),
        "a CREATE on Type0 compiled {} statements against other types' tables. Validation is \
         still walking constraints the statement cannot have broken, which is what makes an \
         ontology install superlinear. Offending statements: {foreign:#?}",
        foreign.len()
    );

    // Without the scoping this was 5 + 2 * (types - 1): one required-property
    // scan and one uniqueness scan for every type in the ontology.
    assert_eq!(
        few.len(),
        many.len(),
        "a CREATE on one type compiled {} statements with 16 types registered but only {} with 2; \
         per-mutation cost must not grow with the size of the ontology",
        many.len(),
        few.len()
    );
}
