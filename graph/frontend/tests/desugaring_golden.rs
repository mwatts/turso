//! Arrow syntax is sugar over roles. If the two forms ever bind to different
//! IR, then a "binary" query and its role-form equivalent can disagree at
//! runtime, and the claim that binary is a layout of the role model is false.

mod fixture;

use fixture::{bind_fixture, first_role_expand};

#[test]
#[ignore = "standalone role pattern lands in Task 12"]
fn arrow_and_role_forms_of_the_same_pattern_bind_identically() {
    let arrow = bind_fixture("MATCH (a:Person)-[r:KNOWS]->(b:Person) RETURN b");
    let roles = bind_fixture("MATCH (a:Person), (b:Person), [r:KNOWS](start: a, end: b) RETURN b");
    assert_eq!(first_role_expand(&arrow), first_role_expand(&roles));
}

#[test]
#[ignore = "standalone role pattern lands in Task 12"]
fn the_reversed_arrow_is_the_reversed_role_pair() {
    let arrow = bind_fixture("MATCH (a:Person)<-[r:KNOWS]-(b:Person) RETURN b");
    let roles = bind_fixture("MATCH (a:Person), (b:Person), [r:KNOWS](end: a, start: b) RETURN b");
    assert_eq!(first_role_expand(&arrow), first_role_expand(&roles));
}
