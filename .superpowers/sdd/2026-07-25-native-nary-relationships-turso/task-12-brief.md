### Task 12: Parse the standalone role pattern

**Files:**
- Modify: `graph/cypher/src/cypher.pest:40-70`
- Modify: `graph/cypher/src/ast.rs:120-200`
- Modify: `graph/cypher/src/parser.rs:380-520`
- Test: `graph/cypher/src/parser.rs` (existing `mod tests`)

**Interfaces:**
- Produces:
  - Grammar: `pattern = { pattern_element ~ ("," ~ pattern_element)* }`, `pattern_element = { role_pattern | path_pattern }`, `role_pattern = { relationship_body ~ role_arguments }`, `role_arguments = { "(" ~ role_argument? ~ ("," ~ role_argument)* ~ ")" }`, `role_argument = { identifier ~ ":" ~ expression }`.
  - AST: `enum PatternElement { Path(PathPattern), Roles(RolePattern) }`; `struct RolePattern { relationship: RelationshipBody, roles: Vec<RoleArgument>, span: Span }`; `struct RoleArgument { name: String, player: Expression, span: Span }`.
  - `Pattern { elements: Vec<PatternElement>, span: Span }` replacing `Vec<PathPattern>`.

- [ ] **Step 1: Write the failing test**

In `graph/cypher/src/parser.rs`'s test module:

```rust
    #[test]
    fn a_standalone_role_pattern_parses_with_its_roles_in_source_order() {
        // `[` never begins a pattern element today, so the role form is
        // unambiguous against every existing pattern.
        let statement = parse("MATCH [x:Transcription {year: 1387}](scribe: p, text: t, folio: f) RETURN x");
        let PatternElement::Roles(roles) = &pattern_of(&statement).elements[0] else {
            panic!("expected a role pattern");
        };
        assert_eq!(roles.relationship.variable.as_deref(), Some("x"));
        assert_eq!(roles.relationship.types, vec!["Transcription".to_owned()]);
        assert_eq!(
            roles.roles.iter().map(|role| role.name.as_str()).collect::<Vec<_>>(),
            vec!["scribe", "text", "folio"]
        );
    }

    #[test]
    fn a_role_pattern_and_a_path_pattern_may_appear_in_one_comma_list() {
        let statement = parse("MATCH (a:Person), [x:Transcription](scribe: a) RETURN x");
        let elements = &pattern_of(&statement).elements;
        assert!(matches!(elements[0], PatternElement::Path(_)));
        assert!(matches!(elements[1], PatternElement::Roles(_)));
    }

    #[test]
    fn a_role_pattern_with_no_roles_is_a_parse_error_not_an_empty_relation() {
        // `[x:T]()` would otherwise read as a relation with no participants,
        // which the binder would then have to reject with a worse message.
        assert!(parse_result("MATCH [x:Transcription]() RETURN x").is_err());
    }

    #[test]
    fn an_arrow_pattern_still_parses_unchanged() {
        let statement = parse("MATCH (a)-[r:KNOWS]->(b) RETURN b");
        assert!(matches!(pattern_of(&statement).elements[0], PatternElement::Path(_)));
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_cypher --lib parser::`
Expected: FAIL to compile with `cannot find type PatternElement`.

- [ ] **Step 3: Extend the grammar**

In `graph/cypher/src/cypher.pest`, replace the `pattern` rule and add the role
rules. `role_pattern` must come first in the ordered choice so a leading `[`
commits to it:

```pest
pattern = { pattern_element ~ ("," ~ pattern_element)* }
pattern_element = { role_pattern | path_pattern }
role_pattern = { relationship_body ~ role_arguments }
role_arguments = { "(" ~ role_argument ~ ("," ~ role_argument)* ~ ")" }
role_argument = { identifier ~ ":" ~ expression }
```

The one-or-more form in `role_arguments` is what makes `[x:T]()` a parse error.

- [ ] **Step 4: Extend the AST**

In `graph/cypher/src/ast.rs`:

```rust
/// One comma-separated element of a MATCH or CREATE pattern.
///
/// The arrow form and the role form are different spellings of the same thing;
/// the binder resolves both to role pairs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PatternElement {
    Path(PathPattern),
    Roles(RolePattern),
}

/// `[x:Transcription {year: 1387}](scribe: p, text: t, folio: f)`
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RolePattern {
    pub variable: Option<String>,
    pub types: Vec<String>,
    pub properties: Option<MapLiteral>,
    /// Source order. The binder does not require declaration order.
    pub roles: Vec<RoleArgument>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleArgument {
    pub name: String,
    pub player: Expression,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pattern {
    pub elements: Vec<PatternElement>,
    pub span: Span,
}
```

Replace every `Vec<PathPattern>` field on `Match`, `Create`, `Merge`, and the
pattern predicate with `Pattern`.

- [ ] **Step 5: Extend the walkers**

In `graph/cypher/src/parser.rs`, add:

```rust
fn walk_pattern(pair: Pair<'_, Rule>) -> Result<ast::Pattern, ParseError> {
    let span = span_of(&pair);
    let mut elements = Vec::new();
    for element in pair.into_inner() {
        let inner = element
            .into_inner()
            .next()
            .ok_or(ParseError::MalformedPattern { span })?;
        elements.push(match inner.as_rule() {
            Rule::role_pattern => ast::PatternElement::Roles(walk_role_pattern(inner)?),
            Rule::path_pattern => ast::PatternElement::Path(walk_path_pattern(inner)?),
            other => return Err(ParseError::UnexpectedRule { rule: format!("{other:?}"), span }),
        });
    }
    Ok(ast::Pattern { elements, span })
}

fn walk_role_pattern(pair: Pair<'_, Rule>) -> Result<ast::RolePattern, ParseError> {
    let span = span_of(&pair);
    let mut inner = pair.into_inner();
    let body = inner.next().ok_or(ParseError::MalformedPattern { span })?;
    let (variable, types, range, properties) = walk_relationship_body(body)?;
    if range.is_some() {
        // A hop range names a repetition of one relationship. It has no
        // meaning on a role list, and accepting it silently would let
        // `[r:T*1..3](start: a)` look supported.
        return Err(ParseError::RangeOnRolePattern { span });
    }
    let mut roles = Vec::new();
    for argument in inner.next().into_iter().flat_map(Pair::into_inner) {
        let argument_span = span_of(&argument);
        let mut parts = argument.into_inner();
        let name = parts
            .next()
            .ok_or(ParseError::MalformedPattern { span: argument_span })?
            .as_str()
            .to_owned();
        let player = walk_expression(
            parts
                .next()
                .ok_or(ParseError::MalformedPattern { span: argument_span })?,
        )?;
        roles.push(ast::RoleArgument { name, player, span: argument_span });
    }
    Ok(ast::RolePattern { variable, types, properties, roles, span })
}
```

Factor the existing `relationship_body` handling out of `walk_relationship` into
`walk_relationship_body` returning the four-tuple, so both callers share it.

Add `ParseError::RangeOnRolePattern { span }` with the message
`"a hop range has no meaning on a role pattern"`.

- [ ] **Step 6: Run to verify it passes**

Run: `cargo test -p turso_cypher`
Expected: PASS.

- [ ] **Step 7: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_cypher -p turso_graph_frontend
mise run corpus
git add -A
git commit -S -m "graph/cypher: parse the standalone role pattern

`[x:T {props}](role: player, ...)` becomes a pattern element alongside the
path form. `[` never began a pattern element before, so the two forms are
unambiguous and every existing arrow query parses unchanged.

A role list must be non-empty, and a hop range on a role pattern is a parse
error rather than a silently accepted no-op.

Tests: parser unit tests over the role form, the mixed comma list, the empty
role list, and an unchanged arrow pattern; corpus at 8,926."
```

---

## Controller corrections — these override the task text above

I verified every one of these against the tree. Where a correction and the
task text disagree, the correction governs.

**1. Package name.** The crate is `turso_graph_cypher`, not `turso_cypher`.
Every `cargo test -p turso_cypher` above is wrong. Use:
`cargo test -p turso_graph_cypher`.

**2. `git add -A` is banned.** It sweeps `graph/test-results/*`, which the
corpus run rewrites and which I commit separately. Stage only the source and
test files you changed, by explicit path.

**3. `ParseError` is a struct, not an enum** (`parser.rs:31`). It has no
variants. Do NOT add `ParseError::MalformedPattern`,
`ParseError::UnexpectedRule`, or `ParseError::RangeOnRolePattern` — they do
not exist and must not be invented. Construct errors the way the file already
does:
- `ParseError::at(span, "message")`
- `unexpected(&pair, "expected-what", rule)` (`parser.rs:1435`)
Existing helpers you should reuse rather than re-derive: `only_child`
(`:1423`), `pair_span` (`:1430`), `walk_identifier` (`:1313`), `walk_map`
(`:1297`, returns `ParsedProperties`), `walk_expression`, `walk_path`,
`parse_range`.
The hop-range rejection stays, spelled
`ParseError::at(span, "a hop range has no meaning on a role pattern")`.

**4. The AST uses `Spanned<T>` everywhere; the task text's struct definitions
do not.** Match the file's convention, mirroring `RelationshipPattern`
(`ast.rs:176`) field for field:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct RolePattern {
    pub variable: Option<Spanned<String>>,
    pub types: Vec<Spanned<String>>,
    pub properties: Vec<(Spanned<String>, Spanned<Expression>)>,
    /// Source order. The binder does not require declaration order.
    pub roles: Vec<RoleArgument>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RoleArgument {
    pub name: Spanned<String>,
    pub player: Spanned<Expression>,
    pub span: Span,
}
```

Note: `PartialEq` only, not `Eq` — nothing in this AST derives `Eq`. There is
no `MapLiteral` type in this crate; properties are the pair vector above.
`RelationshipPattern` has no `has_property_map`, so `RolePattern` gets none
either.

**5. Step 1's test contradicts Step 4's struct.** The test reads
`roles.relationship.variable` and `roles.relationship.types`, but there is no
`relationship` field and no `RelationshipBody` type in this crate. The
flattened struct in correction 4 governs. Write the assertions against the
flattened fields, unwrapping `Spanned` via `.value`.

**6. `role_arguments` is one-or-more, not optional.** The Interfaces block
writes `role_argument?`; Step 3 and the empty-list-is-a-parse-error test both
require one-or-more. Step 3 governs:
`role_arguments = { "(" ~ role_argument ~ ("," ~ role_argument)* ~ ")" }`.

**7. The test helpers in Step 1 do not exist.** There is no `parse_result` and
no `pattern_of`. `parse` (`parser.rs:51`, re-exported from `lib.rs`) already
returns `Result<Query, ParseError>`, so it IS `parse_result`. Write whatever
small helper you need to reach the clause's pattern, in the existing test
module's style — do not add helpers to the public API.

**8. Exactly three `Vec<PathPattern>` fields exist, and MERGE is not one of
them.** They are `CreateClause.paths` (`ast.rs:74`), `MatchClause.paths`
(`ast.rs:139`), and `Expression::PatternSubquery.paths` (`ast.rs:322`). Those
three become `Pattern`.
`MergeClause.path` is a single `PathPattern`, and
`Expression::PatternPredicate.path` is a single `Box<PathPattern>`. Neither
goes through the `pattern` grammar rule — `merge_clause` names `path_pattern`
directly (`cypher.pest:65`) and `pattern_predicate` is its own rule
(`:145`). **Leave both entirely alone.** MERGE over a role pattern is out of
scope for this task. The task text's "and `Merge`, and the pattern predicate"
is wrong.

**9. `walk_relationship` (`parser.rs:472`) does not have a
`walk_relationship_body` to factor out yet, and it does not return a
four-tuple** — it folds the `relationship_body` children into a
`RelationshipPattern` inside a loop over the direction wrapper. Factor the
inner `for item in body.into_inner()` block into a shared helper that both
`walk_relationship` and `walk_role_pattern` call. Keep the returned shape
whatever is cleanest for those two callers; the task text's specific
four-tuple is a suggestion, not a requirement.

**10. Downstream ripple — this is the part the task text omits entirely.**
Changing those three fields to `Pattern` breaks 11 call sites outside the
parser: `graph/frontend/src/binder.rs` at 910, 1127, 1192, 1202, 1255, 2365,
2368, 2406, 4072, 6352, and `graph/frontend/src/compiler.rs:190`. Update them
all.

Task 12 is **purely syntactic** — binding the role form lands in Task 13. So
at every one of those sites, a `PatternElement::Roles` must produce a clear
binder error, never be silently skipped. A `.filter_map` that quietly drops
role elements would make `MATCH [x:T](a: n) RETURN x` return an empty result
instead of failing, and that is the exact failure this task must not ship.
Use the binder's existing error mechanism (match the neighbouring code) with a
message along the lines of `"role patterns are not supported yet"`.

**Add one non-ignored binder test** asserting that a query with a standalone
role pattern binds to an *error* rather than to an empty or partial plan. That
test is the only guard against the silent-drop failure above, and Task 13 will
flip it to a success assertion.

**11. Do not touch the `#[ignore]` attributes** on the existing role-syntax
tests in `graph/frontend/tests/nary_relations.rs` and
`desugaring_golden.rs`. They stay ignored until Task 13.
