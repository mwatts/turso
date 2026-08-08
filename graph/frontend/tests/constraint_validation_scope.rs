//! A mutation must not pay for constraints it cannot have broken.
//!
//! `validate_state` used to re-check every constraint in the graph after every
//! statement. That makes a bootstrap loop superlinear: installing N types costs
//! N mutations, and mutation number K rescans the source table of all K types
//! installed so far. A consuming product reported a 51-entity ontology install
//! that had not finished after 16 minutes against a 60 s install timeout.
//!
//! The fix narrows validation to the source tables a statement actually writes.
//! These tests pin both halves of that: the cost stops growing, and the
//! constraints that *are* in scope still fire.

use std::sync::{Arc, Mutex};

use tracing::field::{Field, Visit};
use tracing_subscriber::layer::{Context, Layer, SubscriberExt};
use turso_graph_frontend::{
    core::{Database, MemoryIO, SqliteDialect},
    register_graph, register_semantic_constraints, register_semantic_schema, GraphConnection,
    GraphRegistration, NodeSourceRegistration, Parameters, SemanticConstraintRegistration,
    SemanticNodeType, SemanticProperty, SemanticRequiredProperty, SemanticSchemaRegistration,
    SemanticUniqueProperty,
};

/// Records the SQL core compiles, by watching the `Preparing: {sql}` debug
/// event `prepare_with_origin` emits.
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

/// A graph of `types` unrelated node types, each on its own table, each with a
/// required property. This is the shape of an ontology install: many types that
/// share nothing, written one at a time.
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
fn statements_per_create(types: usize) -> Vec<String> {
    let recorded = PreparedSql::default();
    let subscriber = tracing_subscriber::registry().with(recorded.clone());
    let _guard = tracing::subscriber::set_default(subscriber);

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
fn a_create_never_scans_a_type_it_did_not_touch() {
    let statements = statements_per_create(16);
    // Guards against the assertion below passing because nothing was recorded.
    assert!(
        statements
            .iter()
            .any(|sql| sql.contains("\"t0\"") || sql.contains(" t0 ")),
        "expected the CREATE to compile SQL against its own source table t0, but recorded \
         {statements:#?}"
    );

    let foreign = foreign_table_reads(&statements);
    assert!(
        foreign.is_empty(),
        "a CREATE on Type0 compiled {} statements against other types' tables. Validation is \
         still walking constraints the statement cannot have broken, which is what makes an \
         ontology install superlinear. Offending statements: {foreign:#?}",
        foreign.len()
    );
}

#[test]
fn a_write_nested_in_foreach_is_still_in_scope() {
    // Writes are not all in `request.operations`. FOREACH nests them inside
    // stage items, and a scope walk that reads only the top level declares the
    // statement touched nothing and skips validation entirely.
    let database = graph_with_unrelated_types(4);
    let session = GraphConnection::open(database.connect().expect("connect"), "ontology")
        .expect("open graph session");

    session
        .execute("CREATE (:Type3 {name: 'taken'})", &Parameters::new())
        .expect("seed the duplicate");
    let error = session
        .execute(
            "FOREACH (n IN [1] | CREATE (:Type3 {name: 'taken'}))",
            &Parameters::new(),
        )
        .expect_err("a duplicate created inside FOREACH must still be rejected");
    let message = error.to_string();
    assert!(
        message.contains("duplicate"),
        "expected a duplicate-value violation from inside FOREACH, got: {message}"
    );
}

#[test]
fn a_create_costs_the_same_no_matter_how_many_other_types_exist() {
    let few = statements_per_create(2).len();
    let many = statements_per_create(16).len();
    // Without the scoping this was 5 + 2 * (types - 1): one required-property
    // scan and one uniqueness scan for every type in the ontology.
    assert!(
        few > 0,
        "recorded no statements at all; the test is not measuring anything"
    );
    assert_eq!(
        few, many,
        "a CREATE on one type compiled {many} statements with 16 types registered but only {few} \
         with 2; per-mutation cost must not grow with the size of the ontology"
    );
}

#[test]
fn a_required_property_still_fails_on_the_type_being_written() {
    // Narrowing must not turn validation off for the source in scope.
    let database = graph_with_unrelated_types(4);
    let session = GraphConnection::open(database.connect().expect("connect"), "ontology")
        .expect("open graph session");

    let error = session
        .execute("CREATE (:Type2 {name: null})", &Parameters::new())
        .expect_err("a NULL required property must be rejected");
    let message = error.to_string();
    assert!(
        message.contains("name"),
        "expected the required-property violation to name the property, got: {message}"
    );
}

#[test]
fn a_unique_property_still_fails_against_a_row_written_earlier() {
    // The duplicate is only visible by looking at rows an earlier statement
    // wrote, so this is the case a naive "only check the new row" narrowing
    // would miss.
    let database = graph_with_unrelated_types(4);
    let session = GraphConnection::open(database.connect().expect("connect"), "ontology")
        .expect("open graph session");

    session
        .execute("CREATE (:Type1 {name: 'taken'})", &Parameters::new())
        .expect("first create");
    let error = session
        .execute("CREATE (:Type1 {name: 'taken'})", &Parameters::new())
        .expect_err("a duplicate unique property must be rejected");
    let message = error.to_string();
    assert!(
        message.contains("duplicate"),
        "expected a duplicate-value violation, got: {message}"
    );
}
