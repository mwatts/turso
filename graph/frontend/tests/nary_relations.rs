//! Coverage for role-shaped relationship writes: `insert_relationship`
//! derives its fixed column list from `create.roles` instead of a
//! hard-coded start/end pair (see `mutation.rs::insert_relationship`).
//!
//! Both tests below describe the standalone role-pattern surface syntax
//! that lands in Task 12; until then there is no Cypher spelling for a
//! bare, unmatched three-role pattern, so they are `#[ignore]`d. The writer
//! itself (including the "role identity is `RoleId`, never position" and
//! "role players need not be distinct" invariants these tests describe) is
//! exercised now, directly against IR with no public API surface, by
//! `mutation.rs`'s own unit tests
//! (`role_players_are_resolved_by_role_id_not_by_position` and
//! `a_repeated_player_fills_two_roles_of_one_relation`).

mod fixture;

use turso_core::Value;
use turso_graph_frontend::Parameters;

#[test]
#[ignore = "surface syntax lands in Task 12"]
fn a_three_role_relation_writes_one_row_with_three_endpoint_columns() {
    let (database, session) = fixture::ternary_session();
    session
        .execute(
            "CREATE (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}), \
             (Transcription {scribe: p, text: t, folio: f, year: 1387})",
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
#[ignore = "surface syntax lands in Task 12"]
fn the_same_player_may_fill_two_roles_of_one_relation() {
    // Nothing may assume role players are distinct: a scribe transcribing
    // their own dictation could plausibly fill two roles of one relation.
    let (database, session) = fixture::ternary_session();
    session
        .execute(
            "CREATE (p:Person {id: 1}), (f:Folio {id: 2}), \
             (Transcription {scribe: p, text: p, folio: f, year: 1400})",
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
