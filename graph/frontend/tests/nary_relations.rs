//! Coverage for CREATE-side binding of the standalone role pattern
//! (`[x:T {props}](role: player, ...)`, Task 13a): resolving role
//! arguments by name against a relation type's declared roles, in source
//! order, and writing them to `ir::CreateRelation.roles` in declaration
//! order so `insert_relationship` derives its fixed column list from
//! `create.roles` instead of a hard-coded start/end pair (see
//! `mutation.rs::insert_relationship`).
//!
//! The two positive tests below exercise a real three-role write end to
//! end through `ternary_session` (unconstrained, schemaless roles). The
//! bind-time error paths (`UnknownRole`, `MissingRequiredRole`,
//! `DuplicateRoleArgument`, a wrong-typed player, an omitted optional role)
//! are exercised against `RoledCatalog`, a minimal hand-rolled catalog with
//! real role target-type constraints and one optional role, via
//! `bind_mutation` directly -- no database required, since none of these
//! are about what gets written, only about what binds. The writer itself
//! (including the "role identity is `RoleId`, never position" and "role
//! players need not be distinct" invariants) is exercised separately,
//! directly against IR with no public API surface, by `mutation.rs`'s own
//! unit tests (`role_players_are_resolved_by_role_id_not_by_position` and
//! `a_repeated_player_fills_two_roles_of_one_relation`).

mod fixture;

use std::sync::Arc;

use turso_core::{Connection, Value};
use turso_graph_cypher::parse;
use turso_graph_frontend::{
    bind_mutation, labels_table_name, load_registered_graph, BindError, CatalogEntity,
    GraphCatalogSnapshot, ParameterTypes, Parameters, RegisteredGraph, ResolvedProperty,
    SemanticRole,
};
use turso_graph_ir::{
    GraphId, LabelId, RelationshipTypeId, RoleCardinality, RoleId, RoleTarget, SourceTableId,
};

/// Inserts a node row directly (bypassing Cypher, whose property binding
/// this schemaless three-node-source graph cannot resolve -- see
/// `fixture::ternary_session`'s doc comment) plus the node-label junction
/// row `MATCH` reads labeled scans through, so the row is visible to a
/// label-only `MATCH` exactly as if `CREATE (:Label {id: ..})` had made it.
fn seed_node(
    connection: &Arc<Connection>,
    graph: &RegisteredGraph,
    label: &str,
    table: &str,
    id: i64,
) {
    let source = graph
        .node_sources
        .iter()
        .find(|source| source.name == label)
        .unwrap_or_else(|| panic!("no {label} node source registered"));
    connection
        .execute(format!(
            "INSERT INTO {table}(id) VALUES ({id}); \
             INSERT INTO \"{}\"(source_id, node_id, label) VALUES ({}, {id}, '{label}');",
            labels_table_name(graph.id),
            source.id.get(),
        ))
        .expect("seed node");
}

#[test]
fn a_three_role_relation_writes_one_row_with_three_endpoint_columns() {
    let (database, session) = fixture::ternary_session();
    let seed = fixture::second_connection(&database);
    let graph = load_registered_graph(&seed, "scriptorium").expect("load registered graph");
    seed_node(&seed, &graph, "Person", "people", 1);
    seed_node(&seed, &graph, "Text", "texts", 2);
    seed_node(&seed, &graph, "Folio", "folios", 3);

    session
        .execute(
            "MATCH (p:Person), (t:Text), (f:Folio) \
             CREATE [x:Transcription {year: 1387}](scribe: p, text: t, folio: f)",
            &Parameters::new(),
        )
        .expect("create three-role relation");

    let connection = fixture::second_connection(&database);
    let rows = connection
        .prepare("SELECT scribe, txt, folio, year FROM transcriptions")
        .unwrap()
        .run_collect_rows()
        .unwrap();
    assert_eq!(
        rows,
        vec![vec![
            Value::from_i64(1),
            Value::from_i64(2),
            Value::from_i64(3),
            Value::from_i64(1387),
        ]]
    );
}

#[test]
fn the_same_player_may_fill_two_roles_of_one_relation() {
    // Nothing may assume role players are distinct: a scribe transcribing
    // their own dictation could plausibly fill two roles of one relation.
    let (database, session) = fixture::ternary_session();
    let seed = fixture::second_connection(&database);
    let graph = load_registered_graph(&seed, "scriptorium").expect("load registered graph");
    seed_node(&seed, &graph, "Person", "people", 1);
    seed_node(&seed, &graph, "Folio", "folios", 2);

    session
        .execute(
            "MATCH (p:Person), (f:Folio) \
             CREATE [x:Transcription {year: 1400}](scribe: p, text: p, folio: f)",
            &Parameters::new(),
        )
        .expect("create relation with a repeated role player");

    let connection = fixture::second_connection(&database);
    let rows = connection
        .prepare("SELECT scribe, txt, folio FROM transcriptions")
        .unwrap()
        .run_collect_rows()
        .unwrap();
    assert_eq!(
        rows,
        vec![vec![
            Value::from_i64(1),
            Value::from_i64(1),
            Value::from_i64(2)
        ]]
    );
}

#[test]
fn role_arguments_bind_by_name_regardless_of_source_order() {
    // Declared order is scribe, text, folio; this query names them
    // folio, scribe, text -- a different order still -- so a bug that
    // resolved roles positionally instead of by name would misassign
    // every column.
    let (database, session) = fixture::ternary_session();
    let seed = fixture::second_connection(&database);
    let graph = load_registered_graph(&seed, "scriptorium").expect("load registered graph");
    seed_node(&seed, &graph, "Person", "people", 10);
    seed_node(&seed, &graph, "Text", "texts", 20);
    seed_node(&seed, &graph, "Folio", "folios", 30);

    session
        .execute(
            "MATCH (p:Person), (t:Text), (f:Folio) \
             CREATE [x:Transcription](folio: f, scribe: p, text: t)",
            &Parameters::new(),
        )
        .expect("create relation with roles named out of declaration order");

    let connection = fixture::second_connection(&database);
    let rows = connection
        .prepare("SELECT scribe, txt, folio FROM transcriptions")
        .unwrap()
        .run_collect_rows()
        .unwrap();
    assert_eq!(
        rows,
        vec![vec![
            Value::from_i64(10),
            Value::from_i64(20),
            Value::from_i64(30),
        ]]
    );
}

/// A minimal, database-free `Transcription` catalog with real role
/// target-type constraints (`scribe`/`witness` -> `Person`, `text` ->
/// `Text`, `folio` -> `Folio`) and one optional role (`witness`), for
/// exercising bind-time role resolution without a real database: none of
/// the tests using it are about what gets written, only about what binds
/// (or fails to).
struct RoledCatalog;

impl GraphCatalogSnapshot for RoledCatalog {
    fn node_source(&self, _graph: GraphId) -> Option<SourceTableId> {
        SourceTableId::new(1).ok()
    }

    fn relationship_source(&self, _graph: GraphId) -> Option<SourceTableId> {
        SourceTableId::new(2).ok()
    }

    fn label(&self, _graph: GraphId, name: &str) -> Option<LabelId> {
        match name {
            "Person" => LabelId::new(1).ok(),
            "Text" => LabelId::new(2).ok(),
            "Folio" => LabelId::new(3).ok(),
            _ => None,
        }
    }

    fn relationship_type(&self, _graph: GraphId, name: &str) -> Option<RelationshipTypeId> {
        (name == "Transcription").then(|| RelationshipTypeId::new(1).unwrap())
    }

    fn property(
        &self,
        _graph: GraphId,
        _entity: CatalogEntity,
        _name: &str,
    ) -> Option<ResolvedProperty> {
        None
    }

    fn relationship_roles(&self, _graph: GraphId, _ty: RelationshipTypeId) -> Vec<SemanticRole> {
        vec![
            SemanticRole {
                role: RoleId::new(1).unwrap(),
                name: "scribe".to_owned(),
                targets: vec![RoleTarget::Node(LabelId::new(1).unwrap())],
                optional: false,
                cardinality: RoleCardinality::One,
            },
            SemanticRole {
                role: RoleId::new(2).unwrap(),
                name: "text".to_owned(),
                targets: vec![RoleTarget::Node(LabelId::new(2).unwrap())],
                optional: false,
                cardinality: RoleCardinality::One,
            },
            SemanticRole {
                role: RoleId::new(3).unwrap(),
                name: "folio".to_owned(),
                targets: vec![RoleTarget::Node(LabelId::new(3).unwrap())],
                optional: false,
                cardinality: RoleCardinality::One,
            },
            SemanticRole {
                role: RoleId::new(4).unwrap(),
                name: "witness".to_owned(),
                targets: vec![RoleTarget::Node(LabelId::new(1).unwrap())],
                optional: true,
                cardinality: RoleCardinality::One,
            },
        ]
    }
}

fn bind_role_pattern_query(query: &str) -> Result<turso_graph_frontend::BoundMutation, BindError> {
    let parsed = parse(query).expect("query must parse");
    bind_mutation(
        &parsed,
        GraphId::new(1).expect("graph id"),
        &RoledCatalog,
        &ParameterTypes::new(),
    )
}

#[test]
fn an_unknown_role_names_the_roles_that_do_exist() {
    let error = bind_role_pattern_query(
        "MATCH (p:Person), (t:Text), (f:Folio) \
         CREATE [x:Transcription](scribbe: p, text: t, folio: f)",
    )
    .expect_err("scribbe is not a declared role");
    let message = error.to_string();
    assert!(message.contains("scribbe"), "{message}");
    assert!(message.contains("scribe"), "{message}");
}

#[test]
fn a_missing_required_role_is_refused_at_bind_time() {
    let error = bind_role_pattern_query(
        "MATCH (p:Person), (t:Text) CREATE [x:Transcription](scribe: p, text: t)",
    )
    .expect_err("folio is required and was not named");
    let message = error.to_string();
    assert!(message.contains("folio"), "{message}");
}

#[test]
fn naming_one_role_twice_is_refused_rather_than_last_write_wins() {
    let error = bind_role_pattern_query(
        "MATCH (p:Person), (t:Text), (f:Folio) \
         CREATE [x:Transcription](scribe: p, scribe: p, text: t, folio: f)",
    )
    .expect_err("scribe is named twice");
    let message = error.to_string();
    assert!(message.contains("scribe"), "{message}");
}

#[test]
fn a_role_rejects_a_player_of_the_wrong_type() {
    let error = bind_role_pattern_query(
        "MATCH (t:Text), (f:Folio) \
         CREATE [x:Transcription](scribe: t, text: t, folio: f)",
    )
    .expect_err("t is a Text, not a Person, and cannot fill scribe");
    let message = error.to_string();
    assert!(message.contains("scribe"), "{message}");
    assert!(message.contains("Text"), "{message}");
}

#[test]
fn an_optional_role_may_be_omitted() {
    let bound = bind_role_pattern_query(
        "MATCH (p:Person), (t:Text), (f:Folio) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f)",
    )
    .expect("witness is optional and may be omitted");
    let turso_graph_ir::Mutation::CreateRelation(create) = &bound.request.operations[0] else {
        panic!(
            "expected a CreateRelation operation, got {:?}",
            bound.request.operations[0]
        );
    };
    assert_eq!(
        create.roles.len(),
        3,
        "only the three named roles should be filled"
    );
}
