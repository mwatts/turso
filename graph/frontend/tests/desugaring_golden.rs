//! Arrow syntax is sugar over roles: the arrow form and the standalone
//! role-pattern form describe the same underlying relation two different
//! ways.
//!
//! ## The contract is rows, not plan shape or SQL text
//!
//! This file originally asserted the two forms lower to identical SQL text.
//! Task 13b's own experiment (`task-13b-report.md`) settled that this is
//! the wrong contract: the arrow form's `RoleExpand` anchors on a node
//! (`bind_path` picks whichever end is more selective and reaches the
//! relationship through its own join condition), while the role form's
//! `RelationScan`/`RoleJoin` anchors on the relation itself and joins each
//! named role's player out from there. These are genuinely different join
//! topologies -- different starting table, different join order -- for
//! every query pair tried, including the fully-fresh-players case with no
//! pre-binding involved. Chasing SQL-text identity would mean special-casing
//! the binder around one query shape just to satisfy a golden, which the
//! task brief forbids.
//!
//! What actually matters, and what this module's tests assert instead: the
//! two forms must never disagree about which rows a query returns. `RoleJoin`
//! anchored on `RelationScan` and `RoleExpand` anchored on `NodeScan` are
//! two different plans for one relation; "binary is a layout, not a kind"
//! is only true if executing either plan produces the same rows. If the two
//! forms ever returned different rows, that claim would be false.
//!
//! These tests therefore run both forms end to end through a real
//! `GraphConnection` (`fixture::witnessed_session`, real storage, real rows)
//! and compare the returned rows, not lowered SQL text or bound `ir::Plan`s.
//!
//! ## Fairness caveat: this comparison is only fair in a single-node-source graph
//!
//! The standalone role-pattern grammar has no label slot --
//! `[r:KNOWS](start: a, end: b)` cannot spell `a:Person` the way
//! `(a:Person)` can. In `witnessed_session`'s graph there is exactly one
//! node source (`Person`/`people`), so every id in `relationships.src`/`dst`
//! can only ever resolve to a `Person` regardless of whether the query says
//! so, and the arrow form's `:Person` label constraint is a no-op filter
//! here. In a multi-node-source schema the arrow form's label constraint
//! and the role form's total absence of one would make these different
//! queries, not equivalent ones. Do not port these tests to a
//! multi-node-source fixture without re-deriving whether the comparison
//! still holds.

mod fixture;

use fixture::witnessed_session;
use turso_core::Value;
use turso_graph_frontend::Parameters;

/// Seeds four people and four `KNOWS` relations with a deliberately
/// asymmetric shape (some `b.id` values repeat, none of the four relations
/// round-trips to itself under reversal) so that neither the forward nor
/// the reversed test below can pass by returning an empty or trivially
/// single-row result.
fn seed_relations(session: &turso_graph_frontend::GraphConnection) {
    session
        .execute(
            "CREATE (:Person {id: 1}), (:Person {id: 2}), (:Person {id: 3}), (:Person {id: 4})",
            &Parameters::new(),
        )
        .expect("seed people");
    for (src, dst) in [(1, 2), (1, 3), (4, 2), (3, 4)] {
        session
            .execute(
                &format!(
                    "MATCH (a:Person {{id: {src}}}), (b:Person {{id: {dst}}}) \
                     CREATE (a)-[:KNOWS]->(b)"
                ),
                &Parameters::new(),
            )
            .expect("seed relation");
    }
}

/// The arrow form and the role form of the same forward pattern
/// (`start` = the arrow's tail, `end` = the arrow's head) must return the
/// same rows. Row order is not part of the contract, only membership --
/// both sides are sorted before comparing.
#[test]
fn arrow_and_role_forms_return_the_same_rows() {
    let (_database, session) = witnessed_session();
    seed_relations(&session);

    let mut arrow = session
        .query(
            "MATCH (a:Person)-[r:KNOWS]->(b:Person) RETURN b.id",
            &Parameters::new(),
        )
        .expect("arrow form must run");
    let mut roles = session
        .query(
            "MATCH [r:KNOWS](start: a, end: b) RETURN b.id",
            &Parameters::new(),
        )
        .expect("role form must run");
    arrow.sort();
    roles.sort();

    let expected = vec![
        vec![Value::from_i64(2)],
        vec![Value::from_i64(2)],
        vec![Value::from_i64(3)],
        vec![Value::from_i64(4)],
    ];
    assert_eq!(
        arrow, expected,
        "sanity: the arrow form must return the seeded rows, not an empty or trivial set"
    );
    assert_eq!(
        roles, arrow,
        "role form must return the same rows as the arrow form: binary is a layout of the \
         role model, not a distinct kind, so the two forms must never disagree"
    );
}

/// The reversed arrow (`<-[r:KNOWS]-`) swaps which end is `start` and which
/// is `end`; its role-form equivalent swaps the named arguments the same
/// way. Seeded with the same asymmetric relations as the forward test, this
/// returns a different set of rows than the forward test (`src` ids instead
/// of `dst` ids) -- confirming the reversal assertion is not vacuous.
#[test]
fn the_reversed_arrow_is_the_reversed_role_pair() {
    let (_database, session) = witnessed_session();
    seed_relations(&session);

    let mut arrow = session
        .query(
            "MATCH (a:Person)<-[r:KNOWS]-(b:Person) RETURN b.id",
            &Parameters::new(),
        )
        .expect("reversed arrow form must run");
    let mut roles = session
        .query(
            "MATCH [r:KNOWS](end: a, start: b) RETURN b.id",
            &Parameters::new(),
        )
        .expect("reversed role form must run");
    arrow.sort();
    roles.sort();

    let expected = vec![
        vec![Value::from_i64(1)],
        vec![Value::from_i64(1)],
        vec![Value::from_i64(3)],
        vec![Value::from_i64(4)],
    ];
    assert_eq!(
        arrow, expected,
        "sanity: the reversed arrow form must return the seeded rows, not an empty or trivial set"
    );
    assert_ne!(
        arrow,
        vec![
            vec![Value::from_i64(2)],
            vec![Value::from_i64(2)],
            vec![Value::from_i64(3)],
            vec![Value::from_i64(4)],
        ],
        "the reversed pair must return a different set than the forward test, or this assertion is vacuous"
    );
    assert_eq!(
        roles, arrow,
        "reversed role form must return the same rows as the reversed arrow form"
    );
}
