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
    bind, bind_mutation, labels_table_name, load_registered_graph, BindError, CatalogEntity,
    Error as FrontendError, GraphCatalogSnapshot, GraphConnection, MutationError, ParameterTypes,
    Parameters, RegisteredGraph, ResolvedProperty, SemanticRole,
};
use turso_graph_ir::{
    GraphId, LabelId, Nullability, Plan, PlanKind, PropertyId, RelationshipTypeId, RoleCardinality,
    RoleId, RoleTarget, SourceTableId, ValueType,
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

/// The standalone role pattern reads a relation's own properties back by
/// role-qualified binding, proving the `MATCH` form actually matches the
/// row `CREATE` wrote (not merely that it parses): naming all three roles
/// resolves the relation variable `x`, whose `year` property is readable
/// without ambiguity because this graph registers exactly one relationship
/// source. (Reading the *node* players' own properties, e.g. `s.id`, is not
/// available on this fixture -- see `fixture::ternary_session`'s doc
/// comment: property resolution without a semantic schema requires exactly
/// one node source, and this graph has three.)
#[test]
fn a_match_role_pattern_reads_a_three_role_relation() {
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
        .expect("create the three-role relation");

    let rows = session
        .query(
            "MATCH [x:Transcription](scribe: s, text: doc, folio: f) RETURN x.year",
            &Parameters::new(),
        )
        .expect("read the three-role relation back by role name");
    assert_eq!(
        rows,
        vec![vec![Value::from_i64(1387)]],
        "naming all three roles must still match the one row CREATE wrote"
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

// --- Task 18b: `MERGE [x:T](role: player, ...)` -- MERGE over a standalone
// role pattern. The grammar previously accepted only the arrow form after
// MERGE (`merge_clause` took `path_pattern` directly, bypassing the
// `role_pattern | path_pattern` alternation CREATE goes through); binding
// routes a role pattern under MERGE to `ir::Mutation::MergeRelation` by
// reusing `bind_create_role_pattern`, the same single implementation CREATE
// uses, rather than a second copy of role resolution.

/// Matching on a subset would make a second MERGE with a different folio
/// silently update the first transcription instead of creating a second
/// one, collapsing two distinct assertions into one.
#[test]
fn merge_matches_on_the_full_set_of_bound_roles() {
    let (database, session) = fixture::ternary_session();
    let seed = fixture::second_connection(&database);
    let graph = load_registered_graph(&seed, "scriptorium").expect("load registered graph");
    seed_node(&seed, &graph, "Person", "people", 1);
    seed_node(&seed, &graph, "Text", "texts", 2);
    seed_node(&seed, &graph, "Folio", "folios", 3);

    let merge_query = "MATCH (p:Person), (t:Text), (f:Folio) \
                        MERGE [x:Transcription](scribe: p, text: t, folio: f)";
    session
        .execute(merge_query, &Parameters::new())
        .expect("first merge creates the relation");
    session
        .execute(merge_query, &Parameters::new())
        .expect("second, identical merge matches rather than creating");

    let connection = fixture::second_connection(&database);
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM transcriptions")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(1)]],
        "running the same MERGE twice leaves exactly one row"
    );

    // A MERGE differing only in folio must be a different assertion, not an
    // update of the first. `ternary_session` cannot filter an existing
    // Folio by property (three node sources, no semantic schema -- see
    // `fixture::ternary_session`'s doc comment), so create a second, fresh
    // Folio in the same statement and merge through it instead of trying to
    // MATCH a specific one.
    session
        .execute(
            "MATCH (p:Person), (t:Text) CREATE (f:Folio) \
             MERGE [x:Transcription](scribe: p, text: t, folio: f)",
            &Parameters::new(),
        )
        .expect("merge with a different folio creates a second relation");
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM transcriptions")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(2)]],
        "a different folio is a different assertion, not an update of the first"
    );
}

/// The merge key must include a `Many` role's membership, not just the
/// `One` roles' fixed columns: if it did not, a second MERGE naming a
/// different `witness` but the same `start`/`end` would silently match the
/// first relation instead of creating a new one, discarding the new witness
/// fact with nothing reporting it. Unlike
/// `merge_matches_on_the_full_set_of_bound_roles` (which varies the `One`
/// role `folio`), every `One` role here (`start`, `end`) is held fixed and
/// only the `Many` role (`witness`) varies.
#[test]
fn merge_with_different_witness_does_not_collapse_into_the_first_relation() {
    let (database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), (:Person {id: 3}), (:Person {id: 4})",
            &Parameters::new(),
        )
        .expect("seed people");

    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}), (w:Person {id: 3}) \
             MERGE [x:KNOWS](start: a, end: b, witness: w)",
            &Parameters::new(),
        )
        .expect("first merge creates the relation with witness 3");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}), (w:Person {id: 4}) \
             MERGE [x:KNOWS](start: a, end: b, witness: w)",
            &Parameters::new(),
        )
        .expect(
            "second merge, same start/end but a different witness, must create a second relation",
        );

    let connection = fixture::second_connection(&database);
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM relationships")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(2)]],
        "a different witness is a different assertion, not an update of the first"
    );
}

/// R7 exact multiset: a relationship whose Many role has {w1,w2} must not
/// match a MERGE that names only {w1}. Subset EXISTS would silently attach
/// ON MATCH to the two-witness edge and discard the smaller assertion.
#[test]
fn merge_with_subset_of_many_players_does_not_match_a_superset_relation() {
    let (database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), (:Person {id: 3}), (:Person {id: 4})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}), (w1:Person {id: 3}), (w2:Person {id: 4}) \
             CREATE [x:KNOWS](start: a, end: b, witness: w1, witness: w2)",
            &Parameters::new(),
        )
        .expect("create two-witness relation");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}), (w:Person {id: 3}) \
             MERGE [x:KNOWS](start: a, end: b, witness: w)",
            &Parameters::new(),
        )
        .expect("subset MERGE must create its own one-witness relation, not match the superset");

    let connection = fixture::second_connection(&database);
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM relationships")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(2)]],
        "subset MERGE must not collapse into a Many-superset relationship"
    );
}

/// Empty MERGE key (no properties, labels, or endpoints) is fail-closed.
#[test]
fn propertyless_merge_without_a_match_key_is_rejected() {
    let (_database, session) = fixture::social_graph_connection();
    let error = session
        .execute("MERGE (n)", &Parameters::new())
        .expect_err("property-less MERGE must not use 1=1 LIMIT 1");
    let message = error.to_string();
    assert!(
        message.contains("match key") || message.contains("MERGE"),
        "expected EmptyMergeKey wording, got: {message}"
    );
}

/// R8: two sessions MERGEing the same pattern leave one relationship row.
/// The merge-keys catalog table serializes the claim (works for shared tables
/// and Many multisets, not only UNIQUE(src,dst)).
#[test]
fn concurrent_identical_relationship_merges_leave_one_row() {
    let (database, session_a) = fixture::social_graph_connection();
    // social fixture already seeded Person id 1 and 2.
    let session_b =
        GraphConnection::open(fixture::second_connection(&database), "social")
            .expect("open second session on the same registered graph");

    let merge = "MATCH (a:Person {id: 1}), (b:Person {id: 2}) MERGE (a)-[:KNOWS]->(b)";
    session_a
        .execute(merge, &Parameters::new())
        .expect("first session merge");
    session_b
        .execute(merge, &Parameters::new())
        .expect("second session merge of the same pattern");

    let connection = fixture::second_connection(&database);
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM relationships")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(1)]],
        "identical MERGEs from two sessions must claim one merge key / one relationship"
    );
}

/// `mutation.rs`'s `insert_relationship` guards its spill writes with `if
/// created` (added in Task 14a, untested until now: it can only be
/// exercised through MERGE over a role pattern). A relation matched by
/// MERGE already has whatever spill rows its original CREATE wrote; without
/// the guard, a second MERGE would insert the witness a second time.
#[test]
fn merging_a_relation_with_a_many_valued_role_does_not_duplicate_spill_rows() {
    let (database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), (:Person {id: 3})",
            &Parameters::new(),
        )
        .expect("seed people");
    let merge_query = "MATCH (a:Person {id: 1}), (b:Person {id: 2}), (w:Person {id: 3}) \
                        MERGE [x:KNOWS](start: a, end: b, witness: w)";
    session
        .execute(merge_query, &Parameters::new())
        .expect("first merge creates the relation with one witness");
    session
        .execute(merge_query, &Parameters::new())
        .expect("second, identical merge matches rather than creating");

    let connection = fixture::second_connection(&database);
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM relationships")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(1)]],
        "one relation row"
    );
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM relationships__witness")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(1)]],
        "the witness is written once, not once per MERGE run"
    );
}

/// `merge_clause` is reachable both from `clause` directly and from
/// `foreach_body`; changing it to accept a role pattern must not disturb
/// the `FOREACH (... | MERGE ...)` path, which routes through a separate
/// binder branch (`bind_foreach`) from a bare top-level MERGE.
#[test]
fn a_role_pattern_merge_inside_foreach_still_binds() {
    let (database, session) = fixture::ternary_session();
    let seed = fixture::second_connection(&database);
    let graph = load_registered_graph(&seed, "scriptorium").expect("load registered graph");
    seed_node(&seed, &graph, "Person", "people", 1);
    seed_node(&seed, &graph, "Text", "texts", 2);
    seed_node(&seed, &graph, "Folio", "folios", 3);

    session
        .execute(
            "MATCH (p:Person), (t:Text), (f:Folio) \
             FOREACH (i IN [1] | MERGE [x:Transcription](scribe: p, text: t, folio: f))",
            &Parameters::new(),
        )
        .expect("role-pattern MERGE inside FOREACH must bind");

    let connection = fixture::second_connection(&database);
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM transcriptions")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(1)]],
        "one relation row created through FOREACH"
    );
}

/// The arrow form must keep working inside `FOREACH` too: `bind_foreach`'s
/// MERGE branch now calls the shared `bind_merge_pattern` helper instead of
/// `bind_create_path` directly, and its `PatternElement::Path` arm must stay
/// a pass-through to that same call.
#[test]
fn an_arrow_form_merge_inside_foreach_still_binds() {
    let (database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2})",
            &Parameters::new(),
        )
        .expect("seed people");

    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}) \
             FOREACH (i IN [1] | MERGE (a)-[:KNOWS]->(b))",
            &Parameters::new(),
        )
        .expect("arrow-form MERGE inside FOREACH must bind");

    let connection = fixture::second_connection(&database);
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM relationships")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(1)]],
        "one relation row created through FOREACH"
    );
}

/// A `Many` role stores its players in a spill table rather than a column,
/// so creating a relation with two `witness` players is one relation row
/// plus two spill rows -- not two relation rows, which would double-count
/// the relation in any aggregate.
#[test]
fn a_many_valued_role_holds_several_players_in_one_relation() {
    let (database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), \
                    (:Person {id: 3}), (:Person {id: 4})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}), \
                   (w1:Person {id: 3}), (w2:Person {id: 4}) \
             CREATE [x:KNOWS](start: a, end: b, witness: w1, witness: w2)",
            &Parameters::new(),
        )
        .expect("create relation with two witnesses");

    let connection = fixture::second_connection(&database);
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM relationships")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(1)]],
        "one relation row"
    );
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM relationships__witness")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(2)]],
        "two spilled players"
    );
}

/// Deleting a relation must remove its spilled players too: a spill row
/// pointing at a relation that no longer exists is a dangling participant
/// that a later hop through that role would surface as a live player.
#[test]
fn deleting_a_relation_removes_its_spilled_players() {
    let (database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), (:Person {id: 3})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}), (w:Person {id: 3}) \
             CREATE [x:KNOWS](start: a, end: b, witness: w)",
            &Parameters::new(),
        )
        .expect("create relation with one witness");
    session
        .execute(
            "MATCH (a:Person)-[r:KNOWS]->(b:Person) DELETE r",
            &Parameters::new(),
        )
        .expect("delete the relation through today's arrow syntax");

    let connection = fixture::second_connection(&database);
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM relationships")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(0)]],
        "relation row is gone"
    );
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM relationships__witness")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(0)]],
        "spill row is gone with it"
    );
}

/// `DETACH DELETE` on a node deletes the relations that reference it through
/// a bare `DELETE FROM <relation> WHERE ...`, which does not on its own touch
/// a spill table. The spilled players of those relations must still be
/// cleaned up, for the same dangling-participant reason as a direct relation
/// delete.
#[test]
fn detach_deleting_a_node_removes_its_relations_spilled_players() {
    let (database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), (:Person {id: 3})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}), (w:Person {id: 3}) \
             CREATE [x:KNOWS](start: a, end: b, witness: w)",
            &Parameters::new(),
        )
        .expect("create relation with one witness");
    session
        .execute(
            "MATCH (a:Person {id: 1}) DETACH DELETE a",
            &Parameters::new(),
        )
        .expect("detach delete the start node");

    let connection = fixture::second_connection(&database);
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM relationships")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(0)]],
        "the relation referencing the deleted node is gone"
    );
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM relationships__witness")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(0)]],
        "its spill row must not dangle behind"
    );
}

/// A ternary relation has no `start`/`end` role at all, so `delete_entity`
/// resolving references only through `relationship_endpoint_sources` (the
/// two-role pattern-hop shape) would find no match here and silently skip
/// this relation type entirely. A `scribe` that is still cited by a
/// `Transcription` must refuse a bare `DELETE` exactly like a start/end
/// player already does, not vanish while `transcriptions.scribe` keeps
/// pointing at a now-nonexistent identity.
#[test]
fn deleting_a_ternary_relations_scribe_is_refused() {
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
        .expect("create the three-role relation");

    let error = session
        .execute("MATCH (p:Person) DELETE p", &Parameters::new())
        .expect_err("a scribe still cited by a transcription must refuse plain DELETE");
    assert!(
        matches!(
            &error,
            FrontendError::Mutation(MutationError::NodeHasRelationships)
        ),
        "{error:?}"
    );

    let connection = fixture::second_connection(&database);
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM people")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(1)]],
        "the refused delete must not remove the scribe"
    );
    assert_eq!(
        connection
            .prepare("SELECT scribe FROM transcriptions")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(1)]],
        "the transcription's scribe reference must be untouched"
    );
}

/// The `DETACH` counterpart of the refusal above: a `scribe` reference is a
/// real participation even though `scribe` is not `start`/`end`, so
/// `DETACH DELETE` must remove the transcription that cites it -- not leave
/// `transcriptions.scribe` dangling at a deleted identity, which is what the
/// two-role-only resolution silently did before this fix.
#[test]
fn detach_deleting_a_ternary_relations_scribe_removes_the_transcription() {
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
        .expect("create the three-role relation");

    session
        .execute("MATCH (p:Person) DETACH DELETE p", &Parameters::new())
        .expect("detach delete the scribe");

    let connection = fixture::second_connection(&database);
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM people")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(0)]],
        "the scribe is gone"
    );
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM transcriptions")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(0)]],
        "detaching the scribe must remove the transcription referencing it, \
         not leave a dangling scribe column behind"
    );
}

/// A `Many` role's players live only in its spill table, never a column on
/// the relation row, so `delete_entity`'s reference check must consider
/// spill-table membership too -- not just `start`/`end` columns -- or a
/// witness-only player (never `start` or `end`) can be plain-`DELETE`d with
/// no error, leaving `relationships__witness.node_id` dangling at a deleted
/// identity.
#[test]
fn deleting_a_witness_only_person_is_refused() {
    let (database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), (:Person {id: 3})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}), (w:Person {id: 3}) \
             CREATE [x:KNOWS](start: a, end: b, witness: w)",
            &Parameters::new(),
        )
        .expect("create relation with one witness");

    let error = session
        .execute("MATCH (w:Person {id: 3}) DELETE w", &Parameters::new())
        .expect_err(
            "a witness-only player still recorded in the spill table must refuse plain DELETE",
        );
    assert!(
        matches!(
            &error,
            FrontendError::Mutation(MutationError::NodeHasRelationships)
        ),
        "{error:?}"
    );

    let connection = fixture::second_connection(&database);
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM people WHERE id = 3")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(1)]],
        "the refused delete must not remove the witness"
    );
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM relationships__witness WHERE node_id = 3")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(1)]],
        "the spill row must be untouched"
    );
}

/// The `DETACH` counterpart: a witness-only player is a real participant in
/// the relation even though it never appears in `start`/`end`, so
/// `DETACH DELETE` must remove the relation (and its spill row) -- exactly
/// as it already does for a `start`/`end` player -- rather than silently
/// leaving `relationships__witness.node_id` pointing at a deleted identity.
#[test]
fn detach_deleting_a_witness_only_person_removes_the_relation_and_spill_row() {
    let (database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), (:Person {id: 3})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}), (w:Person {id: 3}) \
             CREATE [x:KNOWS](start: a, end: b, witness: w)",
            &Parameters::new(),
        )
        .expect("create relation with one witness");

    session
        .execute(
            "MATCH (w:Person {id: 3}) DETACH DELETE w",
            &Parameters::new(),
        )
        .expect("detach delete the witness-only player");

    let connection = fixture::second_connection(&database);
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM people")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(2)]],
        "the witness is gone, start and end players are unaffected"
    );
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM relationships")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(0)]],
        "detaching the only-witness player must remove the relation that referenced it"
    );
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM relationships__witness")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(0)]],
        "its spill row must not dangle behind"
    );
}

/// `GATHERING` (`fixture::two_many_roles_session`) has no `One` role at
/// all -- both `guest` and `witness` are `Many`, so every player lives only
/// in a spill table, never a `start`/`end`-style column on the relation
/// row. `delete_entity` must still refuse a plain `DELETE` of a still-cited
/// guest here, exactly as it does for the single-`Many`-role case in
/// `witnessed_session` -- there being no `One` role at all must not be
/// mistaken by the general per-role walk for there being no roles.
#[test]
fn deleting_an_all_many_role_relations_guest_is_refused() {
    let (database, session) = fixture::two_many_roles_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (g:Person {id: 1}), (w:Person {id: 2}) \
             CREATE [x:GATHERING](guest: g, witness: w)",
            &Parameters::new(),
        )
        .expect("create the all-Many-role gathering");

    let error = session
        .execute("MATCH (g:Person {id: 1}) DELETE g", &Parameters::new())
        .expect_err("a guest still recorded in a spill table must refuse plain DELETE");
    assert!(
        matches!(
            &error,
            FrontendError::Mutation(MutationError::NodeHasRelationships)
        ),
        "{error:?}"
    );

    let connection = fixture::second_connection(&database);
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM people WHERE id = 1")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(1)]],
        "the refused delete must not remove the guest"
    );
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM gatherings__guest WHERE node_id = 1")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(1)]],
        "the guest's spill row must be untouched"
    );
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM gatherings__witness")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(1)]],
        "the witness's spill row must be untouched"
    );
}

/// The `DETACH` counterpart: removing a guest from an all-`Many`-role
/// relation must clean up *both* spill tables, not just the guest's own --
/// leaving `gatherings__witness` behind would dangle a reference to a
/// relation row that no longer exists.
#[test]
fn detach_deleting_an_all_many_role_relation_removes_it_and_both_spill_tables() {
    let (database, session) = fixture::two_many_roles_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (g:Person {id: 1}), (w:Person {id: 2}) \
             CREATE [x:GATHERING](guest: g, witness: w)",
            &Parameters::new(),
        )
        .expect("create the all-Many-role gathering");

    session
        .execute(
            "MATCH (g:Person {id: 1}) DETACH DELETE g",
            &Parameters::new(),
        )
        .expect("detach delete the guest");

    let connection = fixture::second_connection(&database);
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM people")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(1)]],
        "the guest is gone, the witness is unaffected"
    );
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM gatherings")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(0)]],
        "detaching the guest must remove the gathering that referenced it"
    );
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM gatherings__guest")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(0)]],
        "the guest's spill row must not dangle behind"
    );
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM gatherings__witness")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(0)]],
        "the witness's spill row must be cleaned too -- it belongs to the \
         same now-deleted relation, even though it was never named by the \
         DETACH DELETE's own MATCH"
    );
}

/// Same two behaviors as above, against `two_many_roles_session_reordered`
/// (roles declared `witness` before `guest`) and deleting a `witness`
/// rather than a `guest`, to prove neither of the two GATHERING tests above
/// is silently passing because `guest` happens to be declared first: role
/// resolution must be by name/`RoleId`, never registration position.
#[test]
fn deleting_an_all_many_role_relations_witness_is_refused_with_roles_declared_in_reverse_order() {
    let (database, session) = fixture::two_many_roles_session_reordered();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (g:Person {id: 1}), (w:Person {id: 2}) \
             CREATE [x:GATHERING](guest: g, witness: w)",
            &Parameters::new(),
        )
        .expect("create the all-Many-role gathering");

    let error = session
        .execute("MATCH (w:Person {id: 2}) DELETE w", &Parameters::new())
        .expect_err("a witness still recorded in a spill table must refuse plain DELETE");
    assert!(
        matches!(
            &error,
            FrontendError::Mutation(MutationError::NodeHasRelationships)
        ),
        "{error:?}"
    );

    let connection = fixture::second_connection(&database);
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM people WHERE id = 2")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(1)]],
        "the refused delete must not remove the witness"
    );
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM gatherings__witness WHERE node_id = 2")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(1)]],
        "the witness's spill row must be untouched"
    );
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM gatherings__guest")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(1)]],
        "the guest's spill row must be untouched"
    );
}

#[test]
fn detach_deleting_an_all_many_role_relation_removes_it_and_both_spill_tables_with_roles_declared_in_reverse_order(
) {
    let (database, session) = fixture::two_many_roles_session_reordered();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (g:Person {id: 1}), (w:Person {id: 2}) \
             CREATE [x:GATHERING](guest: g, witness: w)",
            &Parameters::new(),
        )
        .expect("create the all-Many-role gathering");

    session
        .execute(
            "MATCH (w:Person {id: 2}) DETACH DELETE w",
            &Parameters::new(),
        )
        .expect("detach delete the witness");

    let connection = fixture::second_connection(&database);
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM people")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(1)]],
        "the witness is gone, the guest is unaffected"
    );
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM gatherings")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(0)]],
        "detaching the witness must remove the gathering that referenced it"
    );
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM gatherings__witness")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(0)]],
        "the witness's spill row must not dangle behind"
    );
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM gatherings__guest")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(0)]],
        "the guest's spill row must be cleaned too, even with roles declared \
         in reverse registration order"
    );
}

/// Regression guard on the pre-existing `DuplicateRoleArgument` refusal: it
/// must keep rejecting a second player for a `One` role now that the same
/// check lets a `Many` role repeat.
#[test]
fn a_single_valued_role_given_two_players_is_refused() {
    let (_database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), (:Person {id: 3})",
            &Parameters::new(),
        )
        .expect("seed people");
    let error = session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}), (c:Person {id: 3}) \
             CREATE [x:KNOWS](start: a, start: c, end: b, witness: c)",
            &Parameters::new(),
        )
        .expect_err("start is a One role and cannot take two players");
    let message = error.to_string();
    assert!(message.contains("start"), "{message}");
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

// --- Task 15: `SET [x](role: player, ...)` -- repointing roles of an
// already-bound relation. Task 13b (the standalone role pattern in MATCH,
// `MATCH [x:T](role: player)`) is not implemented, so every test below binds
// its relation with today's arrow form (`MATCH (a)-[r:KNOWS]->(b)`) instead.

/// Repointing a `One` role updates that role's endpoint column and leaves
/// every other role alone.
#[test]
fn a_single_valued_role_can_be_repointed_after_create() {
    let (database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), (:Person {id: 3}), (:Person {id: 4})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}), (w:Person {id: 4}) \
             CREATE [x:KNOWS](start: a, end: b, witness: w)",
            &Parameters::new(),
        )
        .expect("create relation");
    session
        .execute(
            "MATCH (a:Person)-[r:KNOWS]->(b:Person), (c:Person {id: 3}) \
             SET [r](start: c)",
            &Parameters::new(),
        )
        .expect("repoint start to c");

    let connection = fixture::second_connection(&database);
    let rows = connection
        .prepare("SELECT src, dst FROM relationships")
        .unwrap()
        .run_collect_rows()
        .unwrap();
    assert_eq!(
        rows,
        vec![vec![Value::from_i64(3), Value::from_i64(2)]],
        "start moved to c (3); end (2) is untouched"
    );
}

/// `SET` on a `Many` role replaces its whole player set rather than
/// appending to it. Append has no undo syntax -- there is no way to spell
/// "remove this one witness" -- so an appending SET would make running the
/// same statement twice mean something different from running it once
/// (two witnesses after one run, four after two). Replace does not have
/// that problem: running it twice leaves the same one witness both times.
#[test]
fn setting_a_many_valued_role_replaces_rather_than_appends() {
    let (database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), (:Person {id: 3}), \
                    (:Person {id: 4}), (:Person {id: 5})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}), \
                   (w1:Person {id: 3}), (w2:Person {id: 4}) \
             CREATE [x:KNOWS](start: a, end: b, witness: w1, witness: w2)",
            &Parameters::new(),
        )
        .expect("create relation with two witnesses");
    session
        .execute(
            "MATCH (a:Person)-[r:KNOWS]->(b:Person), (w3:Person {id: 5}) \
             SET [r](witness: w3)",
            &Parameters::new(),
        )
        .expect("replace the witness set with a single new witness");

    let connection = fixture::second_connection(&database);
    let rows = connection
        .prepare("SELECT node_id FROM relationships__witness")
        .unwrap()
        .run_collect_rows()
        .unwrap();
    assert_eq!(
        rows,
        vec![vec![Value::from_i64(5)]],
        "only the new witness remains -- the two old witnesses were replaced, not joined"
    );
}

/// The target-type check on a role update reuses `bind_role_player`, the
/// helper extracted from `bind_create_role_pattern`'s inline check (Task
/// 13a): `witnessed_session`'s roles all target `Person`, so it cannot
/// exercise a target-type refusal. Bind straight against `RoledCatalog`
/// instead, combining CREATE and SET in one statement bound (not executed)
/// through `bind_mutation`: CREATE registers `x`'s entity binding before SET
/// resolves it, so no MATCH -- arrow-form or standalone -- is needed at all.
#[test]
fn a_role_update_rejects_a_player_of_the_wrong_type() {
    let error = bind_role_pattern_query(
        "MATCH (p:Person), (t:Text), (f:Folio) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f) \
         SET [x](scribe: t)",
    )
    .expect_err("t is a Text, not a Person, and cannot fill scribe");
    let message = error.to_string();
    assert!(message.contains("scribe"), "{message}");
    assert!(message.contains("Text"), "{message}");
}

/// A role update names a subset of roles by design (unlike creation, it has
/// no required-role check), but naming a role with a null player is refused
/// rather than treated as "clear this role": SET has no syntax for clearing
/// a role, so a null player can only ever be a mistake.
#[test]
fn a_role_update_rejects_a_null_player() {
    let (_database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), (:Person {id: 3})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}), (w:Person {id: 3}) \
             CREATE [x:KNOWS](start: a, end: b, witness: w)",
            &Parameters::new(),
        )
        .expect("create relation");
    let error = session
        .execute(
            "MATCH (a:Person)-[r:KNOWS]->(b:Person) SET [r](start: null)",
            &Parameters::new(),
        )
        .expect_err("start cannot be cleared with a null player");
    let message = error.to_string();
    assert!(message.contains("start"), "{message}");
}

/// The central semantic claim behind "`SET` replaces, it does not append":
/// running the same `SET` twice must mean what running it once means. If
/// the executor appended instead of replacing, the second run would leave
/// two copies of the new witness instead of one.
#[test]
fn setting_a_many_valued_role_twice_is_idempotent() {
    let (database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), (:Person {id: 3}), \
                    (:Person {id: 4}), (:Person {id: 5})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}), \
                   (w1:Person {id: 3}), (w2:Person {id: 4}) \
             CREATE [x:KNOWS](start: a, end: b, witness: w1, witness: w2)",
            &Parameters::new(),
        )
        .expect("create relation with two witnesses");
    let set_query = "MATCH (a:Person)-[r:KNOWS]->(b:Person), (w3:Person {id: 5}) \
                     SET [r](witness: w3)";
    session
        .execute(set_query, &Parameters::new())
        .expect("replace the witness set with a single new witness (first run)");
    session
        .execute(set_query, &Parameters::new())
        .expect("replace the witness set with a single new witness (second run)");

    let connection = fixture::second_connection(&database);
    let rows = connection
        .prepare("SELECT node_id FROM relationships__witness")
        .unwrap()
        .run_collect_rows()
        .unwrap();
    assert_eq!(
        rows,
        vec![vec![Value::from_i64(5)]],
        "running the same SET twice leaves exactly one witness, not two"
    );
}

/// A single `SET` can name one `Many` role with more than one player
/// argument, same as `CREATE` (one argument per player). The executor must
/// purge the role's spill rows once per role, not once per argument -- a
/// purge-per-argument would delete an earlier argument's just-inserted row,
/// leaving only the last player standing.
#[test]
fn setting_a_many_valued_role_with_two_players_in_one_set_lands_both() {
    let (database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), (:Person {id: 3}), \
                    (:Person {id: 4}), (:Person {id: 5})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}), (w:Person {id: 3}) \
             CREATE [x:KNOWS](start: a, end: b, witness: w)",
            &Parameters::new(),
        )
        .expect("create relation with one witness");
    session
        .execute(
            "MATCH (a:Person)-[r:KNOWS]->(b:Person), \
                   (w1:Person {id: 4}), (w2:Person {id: 5}) \
             SET [r](witness: w1, witness: w2)",
            &Parameters::new(),
        )
        .expect("replace the witness set with two new witnesses in one SET");

    let connection = fixture::second_connection(&database);
    let mut rows = connection
        .prepare("SELECT node_id FROM relationships__witness")
        .unwrap()
        .run_collect_rows()
        .unwrap();
    rows.sort();
    assert_eq!(
        rows,
        vec![vec![Value::from_i64(4)], vec![Value::from_i64(5)]],
        "both players named for the witness role in one SET land, not just the last one"
    );
}

/// The `SET` path shares the create path's duplicate-role rule: a repeated
/// `One` role argument in a single `SET` is refused (a `One` role can only
/// ever hold one player), unlike a repeated `Many` role argument, which is
/// one argument per player and is legal.
#[test]
fn a_role_update_rejects_a_repeated_one_role_argument() {
    let (_database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), (:Person {id: 3}), \
                    (:Person {id: 4})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}), (w:Person {id: 4}) \
             CREATE [x:KNOWS](start: a, end: b, witness: w)",
            &Parameters::new(),
        )
        .expect("create relation");
    let error = session
        .execute(
            "MATCH (a:Person)-[r:KNOWS]->(b:Person), (c:Person {id: 3}) \
             SET [r](start: b, start: c)",
            &Parameters::new(),
        )
        .expect_err("start is a One role and cannot be named twice in one SET");
    let message = error.to_string();
    assert!(message.contains("start"), "{message}");
}

// --- Task 13b: `MATCH [x:T](role: player, ...)` -- the read side of the
// standalone role pattern. `RelationScan` anchors on the relation itself and
// one `RoleJoin` per named role argument joins that role's player out to it,
// so the plan composes to any arity with no arity branch. Seeded here
// through today's arrow-form CREATE (unaffected by role-pattern CREATE's
// all-declared-roles-required check) to isolate what is under test to the
// role-pattern MATCH path itself.

/// The named role arguments must resolve to the actual players of the
/// relation, by `RoleId`, not by name or position: swapping which column
/// `start`/`end` join through would still compile but return the wrong
/// player, which this asserts against directly.
#[test]
fn a_match_role_pattern_binds_the_named_players() {
    let (_database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}) CREATE (a)-[:KNOWS]->(b)",
            &Parameters::new(),
        )
        .expect("create relation via arrow form");

    let rows = session
        .query(
            "MATCH [x:KNOWS](start: s, end: e) RETURN s.id, e.id",
            &Parameters::new(),
        )
        .expect("match a standalone role pattern");
    assert_eq!(rows, vec![vec![Value::from_i64(1), Value::from_i64(2)]]);
}

/// Unlike CREATE (`MissingRequiredRole`), a MATCH role pattern may name a
/// subset of a relation's roles. Two relations share `start` but differ in
/// `end`; matching on `start` alone must still return both relations
/// instead of requiring `end` to be named, or collapsing them into one row.
#[test]
fn a_match_role_pattern_may_leave_roles_unnamed() {
    let (_database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), (:Person {id: 3})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}) CREATE (a)-[:KNOWS]->(b)",
            &Parameters::new(),
        )
        .expect("create first relation");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 3}) CREATE (a)-[:KNOWS]->(b)",
            &Parameters::new(),
        )
        .expect("create second relation");

    let mut rows = session
        .query("MATCH [x:KNOWS](start: s) RETURN x.id", &Parameters::new())
        .expect("match a role pattern naming only a subset of its roles");
    rows.sort();
    assert_eq!(
        rows,
        vec![vec![Value::from_i64(1)], vec![Value::from_i64(2)]],
        "both relations must be returned, not collapsed into one"
    );
}

/// `bind_match_role_pattern` no longer rejects a `Many`-cardinality role
/// argument (`witnessed_session`'s `witness` role, Task 14b): naming it
/// alongside the two `One` roles joins through its spill table like any
/// other role, one row per player.
#[test]
fn a_match_role_pattern_reads_a_many_cardinality_role_argument() {
    let (_database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), (:Person {id: 3})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}), (w:Person {id: 3}) \
             MERGE [x:KNOWS](start: a, end: b, witness: w)",
            &Parameters::new(),
        )
        .expect("seed a relation with a witness");

    let rows = session
        .query(
            "MATCH [x:KNOWS](start: s, end: e, witness: w) RETURN s.id, e.id, w.id",
            &Parameters::new(),
        )
        .expect("naming a Many-cardinality role in a MATCH role pattern must bind");
    assert_eq!(
        rows,
        vec![vec![
            Value::from_i64(1),
            Value::from_i64(2),
            Value::from_i64(3)
        ]],
        "all three named roles -- two `One`, one `Many` -- must resolve together"
    );
}

// --- Task 16: `(x:T)-[:role]->(player)` -- arrow sugar over a relation
// anchor. It is sugar over Task 13b's standalone role pattern, not a
// separate implementation: Rule A decides when a bare `(x:Name)` is a
// relation anchor rather than a node (`Name` is not a node label, but is a
// relationship type -- checked in that order, unconditionally, so a new
// relationship type can never change what an existing node query means);
// Rule B decides that once the source binding is a relation, the bracketed
// name is a role of that relation, never a relationship type, and refuses
// as ambiguous rather than guess when a name is both. Both forms delegate to
// `bind_match_role_pattern`'s `RelationScan`/`RoleJoin` machinery so they
// cannot drift apart.

/// Two relations share `start` but differ in `end`; reading only the `end`
/// role through the arrow sugar must return both players. A one-row fixture,
/// or reading `start` (the role a buggy "anchor's first player" fallback
/// would return regardless of which role was named), cannot tell "reads the
/// named role" apart from "returns the anchor's first player".
#[test]
fn an_arrow_from_a_relation_reads_that_relations_role() {
    let (_database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), (:Person {id: 3})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}) CREATE (a)-[:KNOWS]->(b)",
            &Parameters::new(),
        )
        .expect("create first relation");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 3}) CREATE (a)-[:KNOWS]->(b)",
            &Parameters::new(),
        )
        .expect("create second relation");

    let mut rows = session
        .query(
            "MATCH (x:KNOWS)-[:end]->(e) RETURN e.id",
            &Parameters::new(),
        )
        .expect("an arrow from a relation anchor reads the named role");
    rows.sort();
    assert_eq!(
        rows,
        vec![vec![Value::from_i64(2)], vec![Value::from_i64(3)]],
        "both relations' `end` players must be returned -- an empty result \
         is not a read, and both rows rule out a positional fallback"
    );
}

/// `bind_role_read_step` no longer refuses a role arrow over a
/// `Many`-cardinality role (`witnessed_session`'s `witness` role, Task 14b):
/// it joins through the spill table exactly like `bind_match_role_pattern`'s
/// standalone role form, since both delegate to the same `RoleJoin`
/// machinery -- and so must produce the same rows.
#[test]
fn a_many_role_hops_from_the_arrow_sugar_too() {
    let (_database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), \
                    (:Person {id: 3}), (:Person {id: 4})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}), \
                   (w1:Person {id: 3}), (w2:Person {id: 4}) \
             CREATE [x:KNOWS](start: a, end: b, witness: w1, witness: w2)",
            &Parameters::new(),
        )
        .expect("create a relation with two witnesses");

    let mut arrow_rows = session
        .query(
            "MATCH (x:KNOWS)-[:witness]->(w) RETURN w.id",
            &Parameters::new(),
        )
        .expect("a role arrow over a Many-cardinality role must bind");
    arrow_rows.sort();
    assert_eq!(
        arrow_rows,
        vec![vec![Value::from_i64(3)], vec![Value::from_i64(4)]],
        "both witnesses must be returned via the arrow sugar"
    );

    let mut pattern_rows = session
        .query(
            "MATCH [x:KNOWS](witness: w) RETURN w.id",
            &Parameters::new(),
        )
        .expect("the standalone role form must also bind");
    pattern_rows.sort();
    assert_eq!(
        arrow_rows, pattern_rows,
        "the arrow sugar and the standalone role form must agree"
    );
}

/// Both spellings are relation-anchored and label-less, so unlike the
/// arrow-vs-role goldens in `desugaring_golden.rs` -- rewritten (commit
/// `3dab1431d`) to assert row-equivalence under a ruling that emitted
/// behaviour, not plan identity, is *their* contract, because a
/// node-anchored arrow legitimately plans differently from the role form --
/// there is no reason for these two plans to differ here. Assert plan
/// equality, the stronger claim.
#[test]
fn the_role_arrow_and_the_role_pattern_bind_to_the_same_plan() {
    let (database, _session) = fixture::witnessed_session();
    let connection = fixture::second_connection(&database);
    let arrow_plan =
        fixture::bind_witnessed(&connection, "MATCH (x:KNOWS)-[:start]->(s) RETURN s.id");
    let role_plan = fixture::bind_witnessed(&connection, "MATCH [x:KNOWS](start: s) RETURN s.id");
    assert_eq!(
        arrow_plan, role_plan,
        "both spellings are relation-anchored and label-less; nothing \
         legitimately distinguishes their plans"
    );
}

/// `witness` is both a role of `KNOWS` and, in `ambiguous_session`, a
/// relationship type in its own right. Guessing which one the arrow means
/// would make this query mean one thing today and something else after an
/// unrelated relationship type happened to be registered with the same
/// name; refuse instead of guessing.
///
/// `ambiguous_session`'s `witness` role is `One`-cardinality specifically so
/// that the ambiguity check is the only thing that can produce this
/// failure: a `Many`-cardinality `witness` (as in `witnessed_session`) would
/// also trip `bind_role_read_step`'s separate Many-cardinality guard, and
/// removing only the ambiguity check would still leave this query failing
/// (for the wrong reason) instead of going red.
#[test]
fn a_name_that_is_both_a_role_and_a_relationship_type_is_ambiguous() {
    let (_database, session) = fixture::ambiguous_session();

    let error = session
        .query(
            "MATCH (x:KNOWS)-[:witness]->(w) RETURN w.id",
            &Parameters::new(),
        )
        .expect_err("witness is ambiguous between a role of KNOWS and a relationship type");
    let message = error.to_string();
    assert!(message.contains("witness"), "{message}");
    assert!(message.contains("role"), "{message}");
    assert!(message.contains("relationship type"), "{message}");

    // The ambiguity check must use the same case rule as role resolution
    // (`eq_ignore_ascii_case`): checking a differently-cased spelling here
    // catches a check that compared the user's raw-cased text against the
    // registered relationship type instead of the role's own canonical
    // name, which would let `Witness` resolve as a role while dodging the
    // ambiguity check the lowercase spelling correctly hits.
    let error = session
        .query(
            "MATCH (x:KNOWS)-[:Witness]->(w) RETURN w.id",
            &Parameters::new(),
        )
        .expect_err("a differently-cased arrow must not dodge the ambiguity check");
    let message = error.to_string();
    assert!(message.contains("role"), "{message}");
    assert!(message.contains("relationship type"), "{message}");
}

/// Regression guard: this already passes today, measured against the tree
/// this task branched from. `start` from a *node* binding must still
/// resolve as a relationship type, not a role, or adding `KNOWS`'s roles
/// would change what this existing node-anchored arrow query means.
#[test]
fn the_role_arrow_is_only_available_from_a_relation_binding() {
    let (_database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2})",
            &Parameters::new(),
        )
        .expect("seed people");

    let error = session
        .query(
            "MATCH (p:Person)-[:start]->(s) RETURN s.id",
            &Parameters::new(),
        )
        .expect_err("start is not a relationship type, so this must not bind from a node");
    // Must name `start` specifically, not merely say "relationship type": if
    // the relation-binding guard were missing, a node source would also try
    // role resolution using its own labels as fake "relationship type"
    // names, producing an UnknownRelationshipType error that names the
    // anchor's label (`Person`) instead -- a message that also contains
    // "relationship type" and so would not go red under a weaker assertion.
    let message = error.to_string();
    assert!(message.contains("relationship type"), "{message}");
    assert!(message.contains("start"), "{message}");
    assert!(!message.contains("Person"), "{message}");
}

/// The arrow-form read path (fixed-hop `RoleExpand` and, when a `*` range
/// is present, `GraphExpand`) discovers candidate relationship sources
/// through `relationship_endpoint_sources`, which resolves a source's
/// physical endpoints from roles literally named `start`/`end`
/// (`schema_catalog.rs::relationship_endpoint_sources`). A type without
/// that pair -- like `ternary_session`'s `Transcription`
/// (`scribe`/`text`/`folio`, no `start`/`end`) -- is filtered out before
/// the arrow form ever considers it, so it is bound exactly as if no
/// relationship type by that name related these two node types at all:
/// `BindError::MissingSource { entity: "compatible relationship" }`, not a
/// role-specific error. Reaching that role pair requires the standalone
/// role pattern instead (`[x:Transcription](scribe: s, text: t, folio: f)`).
/// This is a schema-only check -- it fails at bind time even with no rows
/// in the database.
#[test]
fn an_arrow_form_expand_requires_a_start_and_end_role_pair() {
    let (_database, session) = fixture::ternary_session();

    let error = session
        .query(
            "MATCH (p:Person)-[:Transcription]->(t:Text) RETURN t.id",
            &Parameters::new(),
        )
        .expect_err("Transcription has no start/end role pair, so the arrow form must not bind");
    let message = error.to_string();
    assert!(message.contains("compatible relationship"), "{message}");
}

/// The arrow-form *write* path (`CREATE (a)-[:TYPE]->(b)`) resolves the
/// same way, looking up roles literally named `start`/`end`
/// (`binder.rs:1697-1717`) regardless of what roles the type actually
/// declares. A type without that pair -- like `Transcription` -- must be
/// created through the standalone role pattern instead.
#[test]
fn an_arrow_form_create_requires_a_start_and_end_role_pair() {
    let (database, session) = fixture::ternary_session();
    let seed = fixture::second_connection(&database);
    let graph = load_registered_graph(&seed, "scriptorium").expect("load registered graph");
    seed_node(&seed, &graph, "Person", "people", 1);
    seed_node(&seed, &graph, "Text", "texts", 2);

    let error = session
        .execute(
            "MATCH (p:Person), (t:Text) CREATE (p)-[:Transcription]->(t)",
            &Parameters::new(),
        )
        .expect_err("Transcription has no start/end role pair, so the arrow form must not bind");
    let message = error.to_string();
    assert!(message.contains("start"), "{message}");
    assert!(message.contains("role"), "{message}");
}

/// A minimal, database-free catalog where the name `"Ambiguous"` resolves as
/// *both* a node label and a relationship type, and every other name
/// resolves as neither. Purpose-built for Rule A's node-label-first
/// ordering guard: unlike `dialect_alignment.rs`'s `BinaryCatalog`, whose
/// `label`/`relationship_type` both return `Some` for *every* name (so it
/// catches Rule A breakage only by accident, and would stop doing so the
/// moment that stub changed for an unrelated reason), this collides exactly
/// the one name under test and behaves normally otherwise -- so a test
/// against it fails only when Rule A's ordering is actually broken, not as
/// a side effect of some other name also being universally ambiguous.
struct DualNameCatalog;

impl GraphCatalogSnapshot for DualNameCatalog {
    fn node_source(&self, _graph: GraphId) -> Option<SourceTableId> {
        SourceTableId::new(1).ok()
    }

    fn relationship_source(&self, _graph: GraphId) -> Option<SourceTableId> {
        SourceTableId::new(2).ok()
    }

    fn label(&self, _graph: GraphId, name: &str) -> Option<LabelId> {
        (name == "Ambiguous").then(|| LabelId::new(1).unwrap())
    }

    fn relationship_type(&self, _graph: GraphId, name: &str) -> Option<RelationshipTypeId> {
        (name == "Ambiguous").then(|| RelationshipTypeId::new(1).unwrap())
    }

    fn property(
        &self,
        _graph: GraphId,
        _entity: CatalogEntity,
        name: &str,
    ) -> Option<ResolvedProperty> {
        (name == "id").then(|| ResolvedProperty {
            id: PropertyId::new(1).unwrap(),
            value_type: ValueType::Integer,
            nullability: Nullability::Nullable,
        })
    }
}

/// Depth-first walk to the pattern's anchor leaf, to tell a node reading
/// (`NodeScan`) apart from a relation reading (`RelationScan`) -- the two
/// leaf kinds Rule A (in `bind_start_node`) chooses between for a bare
/// `(x:Name)` with no further path steps.
fn anchor_is_node_scan(plan: &Plan) -> bool {
    match plan.kind() {
        PlanKind::NodeScan(_) => true,
        PlanKind::RelationScan(_) => false,
        PlanKind::Filter(filter) => anchor_is_node_scan(&filter.input),
        PlanKind::Project(project) => anchor_is_node_scan(&project.input),
        other => panic!("unexpected plan shape for a bare node/relation anchor: {other:?}"),
    }
}

/// Rule A: a name that resolves as *both* a node label and a relationship
/// type must read as a node, never a relation anchor -- `catalog.label` is
/// checked first, unconditionally, so registering a new relationship type
/// can never change what an existing node query returns. Before this test,
/// this ordering was caught only by an unrelated golden in
/// `dialect_alignment.rs` (see `DualNameCatalog`'s doc comment).
#[test]
fn a_name_that_is_both_a_label_and_a_relationship_type_reads_as_a_node() {
    let parsed = parse("MATCH (x:Ambiguous) RETURN x.id").expect("query must parse");
    let bound = bind(
        &parsed,
        GraphId::new(1).expect("graph id"),
        &DualNameCatalog,
        &ParameterTypes::new(),
    )
    .expect("Ambiguous must bind as a node label");
    assert!(
        anchor_is_node_scan(&bound.plan),
        "Rule A must read Ambiguous as a node label, not a relation anchor: {:?}",
        bound.plan
    );
}

// --- Task 14b: hopping through a `Many`-valued role (read side). Task 14a
// landed the write/delete side (spill rows on CREATE/MERGE/SET and their
// cleanup); these tests cover reading them back through `lower_role_join`.

/// Seeds `witnessed_session` with three `KNOWS` relations that deliberately
/// differ in their `witness` sets: relation A (start id 1, end id 2) has two
/// witnesses (ids 3, 4); relation B (start id 5, end id 6) has one witness
/// (id 7) -- a different count from A, so a spill join missing its
/// `relation_id` correlation (joining every relation to every spill row in
/// the table rather than just its own) would multiply row counts instead of
/// matching them; relation C (start id 8, end id 9) has none, created
/// through the plain arrow so it never touches the spill table at all.
/// A single-relation fixture cannot distinguish "returns every player of the
/// named role" from "returns every row in the spill table" -- with only one
/// relation the two coincide.
fn seed_witness_variety(session: &GraphConnection) {
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), (:Person {id: 3}), \
                    (:Person {id: 4}), (:Person {id: 5}), (:Person {id: 6}), \
                    (:Person {id: 7}), (:Person {id: 8}), (:Person {id: 9})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}), \
                   (w1:Person {id: 3}), (w2:Person {id: 4}) \
             CREATE [x:KNOWS](start: a, end: b, witness: w1, witness: w2)",
            &Parameters::new(),
        )
        .expect("create relation A with two witnesses");
    session
        .execute(
            "MATCH (a:Person {id: 5}), (b:Person {id: 6}), (w:Person {id: 7}) \
             CREATE [x:KNOWS](start: a, end: b, witness: w)",
            &Parameters::new(),
        )
        .expect("create relation B with one witness");
    session
        .execute(
            "MATCH (a:Person {id: 8}), (b:Person {id: 9}) CREATE (a)-[:KNOWS]->(b)",
            &Parameters::new(),
        )
        .expect("create relation C with no witness");
}

/// The load-bearing case: a hop through a `Many`-valued role must return
/// every player of every relation that named it, not truncate to one row per
/// relation. A scalar subquery (the plan's own rejected Step 5 snippet)
/// would collapse relation A's two witnesses into one, silently losing a
/// player while still looking like a working query.
#[test]
fn a_hop_through_a_many_valued_role_returns_every_player() {
    let (_database, session) = fixture::witnessed_session();
    seed_witness_variety(&session);

    let mut rows = session
        .query(
            "MATCH [x:KNOWS](start: s, witness: w) RETURN s.id, w.id",
            &Parameters::new(),
        )
        .expect("a hop through a Many-cardinality role must bind");
    rows.sort();
    assert_eq!(
        rows,
        vec![
            vec![Value::from_i64(1), Value::from_i64(3)],
            vec![Value::from_i64(1), Value::from_i64(4)],
            vec![Value::from_i64(5), Value::from_i64(7)],
        ],
        "every (relation, witness) pair must be returned -- collapsing \
         relation A's two witnesses into one row is the exact data loss a \
         scalar subquery would cause, and relation C (no witness) must \
         contribute nothing"
    );
}

/// The subset projection Task 13b's standalone role pattern established must
/// survive adding a `Many` role to the relation: a query that never names
/// `witness` must still return one row per relation, not one row per
/// (relation, witness) pair. A spill join that leaks into queries which
/// never named the role would turn every unrelated query's row count into a
/// function of witness count.
#[test]
fn not_naming_a_many_valued_role_does_not_multiply_rows() {
    let (_database, session) = fixture::witnessed_session();
    seed_witness_variety(&session);

    let mut rows = session
        .query("MATCH [x:KNOWS](start: s) RETURN s.id", &Parameters::new())
        .expect("naming only a One role must still bind");
    rows.sort();
    assert_eq!(
        rows,
        vec![
            vec![Value::from_i64(1)],
            vec![Value::from_i64(5)],
            vec![Value::from_i64(8)],
        ],
        "one row per relation regardless of witness count"
    );
}

/// Relation C has no players in `witness` at all. Naming `witness` must
/// exclude it -- an inner join through the spill table, not an outer one --
/// while a query that hops through a different role (`start`) must still see
/// it, so the absence is specific to the `witness` hop, not a symptom of
/// relation C being broken some other way.
#[test]
fn a_relation_with_no_players_in_a_many_role_is_absent_from_that_hop_but_not_others() {
    let (_database, session) = fixture::witnessed_session();
    seed_witness_variety(&session);

    let mut with_witness = session
        .query(
            "MATCH [x:KNOWS](start: s, witness: w) RETURN s.id",
            &Parameters::new(),
        )
        .expect("hop through witness must bind");
    with_witness.sort();
    assert_eq!(
        with_witness,
        vec![
            vec![Value::from_i64(1)],
            vec![Value::from_i64(1)],
            vec![Value::from_i64(5)],
        ],
        "relation C (start id 8) must be absent from the witness hop"
    );

    let mut all_starts = session
        .query("MATCH [x:KNOWS](start: s) RETURN s.id", &Parameters::new())
        .expect("start must still bind for every relation");
    all_starts.sort();
    assert_eq!(
        all_starts,
        vec![
            vec![Value::from_i64(1)],
            vec![Value::from_i64(5)],
            vec![Value::from_i64(8)],
        ],
        "relation C is absent only from the witness hop, not from every hop \
         -- distinguishing an inner join from an outer one"
    );
}

/// `RolePlayer::Bound` over a `Many`-cardinality role: the plan's own Step 5
/// guidance covers only a fresh player and does not mention this case at
/// all. The `One` arm folds a bound player to an identity equality
/// (`q.<role_column> = q.<binding>`), but a `Many` role has no role column,
/// so this needs a membership test against the spill table instead (the
/// same shape `mutation.rs` builds for MERGE's merge key over a `Many`
/// role). Two relations with different single witnesses (A's is id 3, B's
/// is id 7) verify the bound player actually selects -- not merely that
/// binding a player doesn't error.
#[test]
fn a_bound_player_constrains_a_many_valued_role() {
    let (_database, session) = fixture::witnessed_session();
    seed_witness_variety(&session);

    let rows = session
        .query(
            "MATCH (w:Person {id: 3}), [x:KNOWS](start: s, witness: w) RETURN s.id",
            &Parameters::new(),
        )
        .expect("a bound player must constrain a Many-cardinality role");
    assert_eq!(
        rows,
        vec![vec![Value::from_i64(1)]],
        "only relation A has witness id 3 among its players"
    );

    let rows = session
        .query(
            "MATCH (w:Person {id: 7}), [x:KNOWS](start: s, witness: w) RETURN s.id",
            &Parameters::new(),
        )
        .expect("a bound player must constrain a Many-cardinality role");
    assert_eq!(
        rows,
        vec![vec![Value::from_i64(5)]],
        "only relation B has witness id 7 among its players"
    );
}

/// Naming two `Many`-valued roles in the same hop must produce their
/// Cartesian product, not a zip and not a truncation to one row. Each
/// `Many` role is joined in independently by `lower_role_join` (no arity
/// branch, no special-casing for "more than one Many role on this
/// relation"): the `guest` join contributes one row per guest and the
/// `witness` join contributes one row per witness for *each* of those rows,
/// exactly the way composing any two ordinary joins multiplies their row
/// counts. Cypher's pattern semantics are bag semantics, not set-zipping --
/// there is no positional correspondence between two independently-declared
/// role arguments to zip by, so 2 guests x 2 witnesses must read back as 4
/// rows. Collapsing this to a zip (2 rows) or to a single row would both be
/// silent data loss dressed up as a simplification.
#[test]
fn naming_two_many_valued_roles_at_once_produces_their_cartesian_product() {
    let (_database, session) = fixture::two_many_roles_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), \
                    (:Person {id: 3}), (:Person {id: 4})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (g1:Person {id: 1}), (g2:Person {id: 2}), \
                   (w1:Person {id: 3}), (w2:Person {id: 4}) \
             CREATE [x:GATHERING](guest: g1, guest: g2, witness: w1, witness: w2)",
            &Parameters::new(),
        )
        .expect("create a gathering with two guests and two witnesses");

    let mut rows = session
        .query(
            "MATCH [x:GATHERING](guest: g, witness: w) RETURN g.id, w.id",
            &Parameters::new(),
        )
        .expect("naming two Many-cardinality roles at once must bind");
    rows.sort();
    assert_eq!(
        rows,
        vec![
            vec![Value::from_i64(1), Value::from_i64(3)],
            vec![Value::from_i64(1), Value::from_i64(4)],
            vec![Value::from_i64(2), Value::from_i64(3)],
            vec![Value::from_i64(2), Value::from_i64(4)],
        ],
        "2 guests x 2 witnesses must read back as their full 4-row Cartesian \
         product, one row per (guest, witness) pair"
    );
}

/// A `Many` role's spill table carries no uniqueness constraint on
/// `(relation_id, node_id)` (see `install_spill_table`), and Task 14a
/// deliberately kept the duplicate-role-argument refusal restricted to
/// `One` roles (`bind_match_role_pattern`'s `repeated && cardinality ==
/// One` check): naming the same `Many` role twice with the same player is
/// accepted at bind time and writes two spill rows. Reading that role back
/// must therefore surface both rows rather than silently de-duplicating --
/// a `SELECT DISTINCT` or any other collapse would discard data the write
/// side intentionally allowed onto disk, and there would be no way to tell
/// "witnessed once" from "witnessed twice" ever again.
#[test]
fn a_duplicated_player_in_one_many_role_produces_duplicate_rows() {
    let (_database, session) = fixture::witnessed_session();
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), (:Person {id: 3})",
            &Parameters::new(),
        )
        .expect("seed people");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}), (w:Person {id: 3}) \
             CREATE [x:KNOWS](start: a, end: b, witness: w, witness: w)",
            &Parameters::new(),
        )
        .expect("naming the same Many role twice with the same player must bind");

    let rows = session
        .query(
            "MATCH [x:KNOWS](start: s, witness: w) RETURN w.id",
            &Parameters::new(),
        )
        .expect("hop through the duplicated witness role");
    assert_eq!(
        rows,
        vec![vec![Value::from_i64(3)], vec![Value::from_i64(3)]],
        "the spill table holds two rows for player 3 (no unique constraint \
         on (relation_id, node_id)), so the hop must surface both -- \
         de-duplicating here would silently discard what the write side \
         accepted by design"
    );
}

/// Part A (Task 19): a relation row and its `Many`-role spill rows share one
/// transaction, so a failure that runs after both are staged must leave
/// neither behind. `Citation.label` is registered as a required property
/// (`SemanticConstraintRegistration`), which Cypher `CREATE` does not check
/// at bind time -- it is enforced by `constraints.validate_state`, which
/// `run()` (`mutation.rs`) calls only *after* `execute_bound` has already
/// inserted the relation row and its `witnesses` spill rows. Omitting
/// `label` is therefore a create that binds and writes successfully and only
/// then fails, with everything already on disk inside the still-open
/// transaction.
#[test]
fn a_failure_partway_through_an_n_ary_create_leaves_nothing_behind() {
    // The integrity property reified modeling cannot provide: reification
    // needs one statement per role, so a failure between them leaves a
    // partially stated assertion that reads as complete. Here the relation
    // row and its spill rows share one transaction, so a failure after the
    // spill inserts have executed still leaves BOTH tables empty.
    let (database, session) = fixture::citation_session();
    session
        .execute(
            "CREATE (:Text {title: 'main'}), (:Text {title: 'witness-one'}), \
                    (:Text {title: 'witness-two'})",
            &Parameters::new(),
        )
        .expect("seed text");
    session
        .execute(
            "MATCH (t:Text {title: 'main'}) CREATE [x:Transcription](source: t)",
            &Parameters::new(),
        )
        .expect("create the transcription to be cited");

    let error = session
        .execute(
            "MATCH [x:Transcription](source: t), \
                   (w1:Text {title: 'witness-one'}), (w2:Text {title: 'witness-two'}) \
             CREATE [c:Citation](cited: x, witnesses: w1, witnesses: w2)",
            &Parameters::new(),
        )
        .expect_err(
            "omitting the required `label` property must fail after the insert, not before",
        );
    assert!(
        matches!(
            &error,
            FrontendError::Mutation(MutationError::SemanticConstraintViolation(detail))
                if detail.contains("Citation.label") && detail.contains("required")
        ),
        "{error:?}"
    );

    let connection = fixture::second_connection(&database);
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM citations")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(0)]],
        "the relation row must not survive the constraint failure"
    );
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM citations__witnesses")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(0)]],
        "both spill rows, already inserted before validate_state ran, must not survive either"
    );
}

/// Part B (Task 19): `Citation.cited` targets only `Transcription`, so the
/// transcription created above must itself be an accepted player -- a
/// relation identity is an identity like any other.
#[test]
fn a_relation_may_be_a_player_of_another_relation() {
    // A relation identity is an identity: the role's target list carries
    // RoleTarget::Relation, so the transcription itself fills `cited`.
    let (_database, session) = fixture::citation_session();
    session
        .execute("CREATE (:Text {title: 'main'})", &Parameters::new())
        .expect("seed text");
    session
        .execute(
            "MATCH (t:Text {title: 'main'}) CREATE [x:Transcription](source: t)",
            &Parameters::new(),
        )
        .expect("create the transcription that will itself be cited");
    session
        .execute(
            "MATCH [x:Transcription](source: t) \
             CREATE [c:Citation {label: 'primary'}](cited: x)",
            &Parameters::new(),
        )
        .expect("a relation must be an accepted player of a role targeting only relations");

    let rows = session
        .query(
            "MATCH [c:Citation](cited: x) RETURN c.label",
            &Parameters::new(),
        )
        .expect("read the citation back");
    assert_eq!(
        rows,
        vec![vec![Value::build_text("primary")]],
        "the citation was created with the transcription as its `cited` player"
    );
}

/// The hole this task closes: before the fix, an all-`Relation` target list
/// made `bind_role_player`'s allowed-labels list come out empty, which the
/// check read as "unconstrained" and skipped entirely, so a node was
/// silently accepted into `cited`.
#[test]
fn a_role_that_accepts_only_relations_refuses_a_node_player() {
    // The hole this task closes: today `allowed` comes out empty for a
    // relation-only role and the check is skipped entirely, so a node is
    // accepted into `cited`. It must be refused.
    let (_database, session) = fixture::citation_session();
    session
        .execute("CREATE (:Text {title: 'main'})", &Parameters::new())
        .expect("seed text");

    let error = session
        .execute(
            "MATCH (t:Text {title: 'main'}) CREATE [c:Citation {label: 'x'}](cited: t)",
            &Parameters::new(),
        )
        .expect_err("a role targeting only relations must refuse a node player");
    assert!(
        matches!(
            &error,
            FrontendError::Mutation(MutationError::Bind(BindError::RoleTargetTypeViolation {
                relationship_type,
                role,
                found,
                ..
            })) if relationship_type == "Citation" && role == "cited" && found == "Text"
        ),
        "{error:?}"
    );
}

/// `source` targets only `Text`; a `Transcription` is a relation, not a
/// node, so it must be refused the same way a wrongly-labeled node would be.
#[test]
fn a_role_that_does_not_accept_relations_refuses_a_relation_player() {
    // `source` targets Text only, so the transcription is not a legal player.
    // Assert the actual RoleTargetTypeViolation text -- the plan's
    // `error.contains("source")` would also pass on a syntax error that
    // happens to echo the role name back.
    let (_database, session) = fixture::citation_session();
    session
        .execute("CREATE (:Text {title: 'main'})", &Parameters::new())
        .expect("seed text");
    session
        .execute(
            "MATCH (t:Text {title: 'main'}) CREATE [x:Transcription](source: t)",
            &Parameters::new(),
        )
        .expect("create a transcription to use as an illegal player");

    let error = session
        .execute(
            "MATCH [x:Transcription](source: t) CREATE [y:Transcription](source: x)",
            &Parameters::new(),
        )
        .expect_err("`source` targets Text only, so a Transcription is not a legal player");
    assert!(
        matches!(
            &error,
            FrontendError::Mutation(MutationError::Bind(BindError::RoleTargetTypeViolation {
                relationship_type,
                role,
                found,
                ..
            })) if relationship_type == "Transcription" && role == "source" && found == "Transcription"
        ),
        "{error:?}"
    );
}

/// The second hole this task closes: `reference` targets both `Text` and
/// `Transcription`, so both a node player and a relation player must be
/// accepted -- before the fix, a relationship type name never resolved
/// through `catalog.label`, so the mixed role rejected every relation.
#[test]
fn a_role_with_both_node_and_relation_targets_accepts_either() {
    // The second hole: with a mixed target list the current code rejects
    // every relation player, because a relationship type name never resolves
    // as a label.
    let (_database, session) = fixture::citation_session();
    session
        .execute(
            "CREATE (:Text {title: 'main'}), (:Text {title: 'appendix'})",
            &Parameters::new(),
        )
        .expect("seed text");
    session
        .execute(
            "MATCH (t:Text {title: 'main'}) CREATE [x:Transcription](source: t)",
            &Parameters::new(),
        )
        .expect("create the transcription used as both `cited` and a `reference` player");

    session
        .execute(
            "MATCH [x:Transcription](source: t), (r:Text {title: 'appendix'}) \
             CREATE [c1:Citation {label: 'node-ref'}](cited: x, reference: r)",
            &Parameters::new(),
        )
        .expect("a Text node must be an accepted `reference` player");
    session
        .execute(
            "MATCH [x:Transcription](source: t) \
             CREATE [c2:Citation {label: 'relation-ref'}](cited: x, reference: x)",
            &Parameters::new(),
        )
        .expect("a Transcription relation must also be an accepted `reference` player");

    let mut rows = session
        .query(
            "MATCH [c:Citation](cited: x) RETURN c.label",
            &Parameters::new(),
        )
        .expect("read both citations back");
    rows.sort();
    assert_eq!(
        rows,
        vec![
            vec![Value::build_text("node-ref")],
            vec![Value::build_text("relation-ref")],
        ],
        "both the node reference and the relation reference must have created their citation"
    );
}
