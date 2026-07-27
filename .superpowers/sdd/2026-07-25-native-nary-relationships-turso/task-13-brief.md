### Task 13: Bind the standalone role pattern

**Files:**
- Modify: `graph/frontend/src/binder.rs:225-416` (`BindError`), `:432-459` (`classify_statement`), `:1472-1605` (create), `:2700-2825` (match)
- Test: `graph/frontend/tests/nary_relations.rs`, `graph/frontend/tests/desugaring_golden.rs`

**Interfaces:**
- Consumes: `ast::{PatternElement, RolePattern, RoleArgument}` (Task 12), `SemanticRole` (Task 8).
- Produces: `BindError::{UnknownRole, MissingRequiredRole, RoleCardinalityViolation, DuplicateRoleArgument}`. (`RoleTargetTypeViolation` landed in Task 8; `AmbiguousRoleName` lands in Task 16.)

- [ ] **Step 1: Write the failing test**

Append to `graph/frontend/tests/nary_relations.rs`:

```rust
#[test]
fn an_unknown_role_names_the_roles_that_do_exist() {
    let session = ternary_session();
    let error = session.expect_error(
        "MATCH (p:Person {id: 1}) CREATE [x:Transcription](scribbe: p)",
    );
    assert!(error.contains("scribbe"), "the error must quote what was written: {error}");
    assert!(error.contains("scribe"), "and name a real role: {error}");
}

#[test]
fn a_missing_required_role_is_refused_at_bind_time() {
    // A relation missing a required role is a half-stated assertion. Writing
    // it and letting a NULL column stand for "unknown" would make every later
    // read of that role wrong in a way nothing reports.
    let session = ternary_session();
    let error = session.expect_error(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}) \
         CREATE [x:Transcription](scribe: p, text: t)",
    );
    assert!(error.contains("folio"), "the error must name the missing role: {error}");
}

#[test]
fn an_optional_role_may_be_omitted() {
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f)",
    );
    // `witness` is optional and was omitted; the create succeeded.
    assert_eq!(session.sql("SELECT count(*) FROM transcriptions")[0], vec!["1"]);
}

#[test]
fn naming_one_role_twice_is_refused_rather_than_last_write_wins() {
    let session = ternary_session();
    let error = session.expect_error(
        "MATCH (p:Person {id: 1}), (q:Person {id: 4}) \
         CREATE [x:Transcription](scribe: p, scribe: q)",
    );
    assert!(error.contains("scribe"), "{error}");
}

#[test]
fn a_role_rejects_a_player_of_the_wrong_type() {
    let session = ternary_session();
    let error = session.expect_error(
        "MATCH (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: t, text: t, folio: f)",
    );
    assert!(error.contains("scribe"), "{error}");
    assert!(error.contains("Text"), "the error must name what was offered: {error}");
}

#[test]
fn a_role_pattern_in_match_binds_the_relation_and_every_named_player() {
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f)",
    );
    let rows = session.query(
        "MATCH [x:Transcription](scribe: s, folio: g) RETURN s.id, g.id",
    );
    assert_eq!(rows, vec![vec!["1", "3"]]);
}

#[test]
fn a_match_role_pattern_may_leave_roles_unnamed() {
    // Naming a subset is a projection, not an under-specification: the
    // unnamed roles are simply not bound.
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f)",
    );
    assert_eq!(
        session.query("MATCH [x:Transcription](scribe: s) RETURN s.id"),
        vec![vec!["1"]]
    );
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --test nary_relations`
Expected: FAIL — the binder does not handle `PatternElement::Roles`.

- [ ] **Step 3: Add the errors**

```rust
    #[error("relationship type `{relationship_type}` has no role `{role}`; its roles are {known}")]
    UnknownRole {
        relationship_type: String,
        role: String,
        known: String,
        span_start: usize,
        span_end: usize,
    },
    #[error("relationship type `{relationship_type}` requires role `{role}`")]
    MissingRequiredRole {
        relationship_type: String,
        role: String,
        span_start: usize,
        span_end: usize,
    },
    #[error("role `{role}` of `{relationship_type}` holds one player; it was given {count}")]
    RoleCardinalityViolation {
        relationship_type: String,
        role: String,
        count: usize,
        span_start: usize,
        span_end: usize,
    },
    #[error("role `{role}` of `{relationship_type}` is named more than once")]
    DuplicateRoleArgument {
        relationship_type: String,
        role: String,
        span_start: usize,
        span_end: usize,
    },
```

- [ ] **Step 4: Bind a role pattern in CREATE**

Add to the create path, alongside the existing path-pattern handling:

```rust
    fn bind_create_role_pattern(
        &mut self,
        pattern: &ast::RolePattern,
    ) -> Result<ir::CreateRelation, BindError> {
        let (type_id, type_name) = self.single_relationship_type(&pattern.types, pattern.span)?;
        let declared = self.catalog.relationship_roles(type_id);
        let mut bound: Vec<ir::RoleBinding> = Vec::with_capacity(pattern.roles.len());
        let mut seen: HashMap<ir::RoleId, usize> = HashMap::new();

        for argument in &pattern.roles {
            let role = declared
                .iter()
                .find(|role| role.name.eq_ignore_ascii_case(&argument.name))
                .ok_or_else(|| BindError::UnknownRole {
                    relationship_type: type_name.clone(),
                    role: argument.name.clone(),
                    known: declared
                        .iter()
                        .map(|role| role.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    span_start: argument.span.start,
                    span_end: argument.span.end,
                })?;
            let count = seen.entry(role.role).or_insert(0);
            *count += 1;
            // Two arguments for a One role is a contradiction, not an
            // overwrite. Last-write-wins would silently discard a player the
            // author explicitly named.
            if *count > 1 && role.cardinality == ir::RoleCardinality::One {
                return Err(BindError::DuplicateRoleArgument {
                    relationship_type: type_name.clone(),
                    role: role.name.clone(),
                    span_start: argument.span.start,
                    span_end: argument.span.end,
                });
            }
            let value = self.bind_role_player(argument, role, &type_name)?;
            bound.push(ir::RoleBinding { role: role.role, value });
        }

        for role in declared.iter().filter(|role| !role.optional) {
            if !seen.contains_key(&role.role) {
                return Err(BindError::MissingRequiredRole {
                    relationship_type: type_name.clone(),
                    role: role.name.clone(),
                    span_start: pattern.span.start,
                    span_end: pattern.span.end,
                });
            }
        }

        // Declaration order, so the writer's column list is stable regardless
        // of the order the author wrote the arguments in.
        bound.sort_by_key(|binding| {
            declared
                .iter()
                .position(|role| role.role == binding.role)
                .unwrap_or(usize::MAX)
        });

        Ok(ir::CreateRelation {
            binding: self.declare_relationship_binding(pattern)?,
            source: self.catalog.relationship_source_for_type(type_id).ok_or(
                BindError::UnknownRelationshipType {
                    name: type_name.clone(),
                    span_start: pattern.span.start,
                    span_end: pattern.span.end,
                },
            )?,
            relationship_types: vec![type_id],
            roles: bound,
            properties: self.bind_property_map(pattern.properties.as_ref())?,
        })
    }
```

`bind_role_player` resolves the argument expression to a `BindingId` and checks
it against `role.targets`, raising `BindError::RoleTargetTypeViolation` (Task 8)
when the player's type is absent from a non-empty target list. **No cross-role
uniqueness check**: the same player under two roles is legal.

- [ ] **Step 5: Bind a role pattern in MATCH**

A MATCH role pattern with n named roles lowers to the relation scan plus one
join per named role. Add:

```rust
    fn bind_match_role_pattern(
        &mut self,
        pattern: &ast::RolePattern,
        input: ir::Plan,
    ) -> Result<ir::Plan, BindError> {
        let (type_id, type_name) = self.single_relationship_type(&pattern.types, pattern.span)?;
        let declared = self.catalog.relationship_roles(type_id);
        let source = self.relationship_source(type_id, &type_name, pattern.span)?;
        let relation = self.declare_relationship_binding(pattern)?;
        // The relation is the anchor; each named role is a join from it out to
        // its player. Unnamed roles are not bound, which is a projection over
        // the relation's participants, not an under-specified match.
        let mut plan = self.scan_relationship(input, source, relation.clone(), type_id)?;
        for argument in &pattern.roles {
            let role = self.resolve_declared_role(&declared, argument, &type_name)?;
            plan = self.join_role_player(plan, source, relation.id(), role, argument)?;
        }
        Ok(plan)
    }
```

`join_role_player` emits a `RoleExpand` whose `from_role` is the role being
joined and whose `to_role` is the same role — the relation is already bound, so
the expand runs relation → player. For a `Many` role it joins through the spill
table (Task 14).

- [ ] **Step 6: Classify a role pattern as a write when it appears under CREATE**

In `classify_statement` (`binder.rs:432-459`), extend the pattern walk to visit
`PatternElement::Roles` as well as `PatternElement::Path`, so a statement whose
only pattern is a role pattern is still classified `StatementKind::Write` under
CREATE and `StatementKind::Read` under MATCH.

- [ ] **Step 7: Un-ignore the surface-syntax tests**

Remove `#[ignore = "surface syntax lands in Task 12"]` from
`graph/frontend/tests/nary_relations.rs` and
`#[ignore = "standalone role pattern lands in Task 12"]` from
`graph/frontend/tests/desugaring_golden.rs`.

- [ ] **Step 8: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend --test nary_relations --test desugaring_golden`
Expected: PASS, including the desugaring goldens proving the arrow and role
forms bind to identical IR.

- [ ] **Step 9: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend
mise run corpus
mise run cypherbench-sample
git add -A
git commit -S -m "graph/binder: bind the standalone role pattern

CREATE resolves each named role against the type's declaration, refuses an
unknown role while naming the real ones, refuses a missing required role
rather than writing a NULL that would later read as a real answer, refuses a
repeated name for a single-valued role instead of last-write-wins, and does
not require cross-role uniqueness: the same player under two roles is legal.

MATCH anchors on the relation and joins one player per named role, leaving
unnamed roles unbound.

The desugaring goldens now run: the arrow form and the role form of the same
pattern bind to identical IR.

Tests: nary_relations, desugaring_golden; corpus at 8,926; cypherbench at
baseline."
```

---

## Controller corrections — these override the task text above

I verified every one of these against the tree. Where a correction and the
task text disagree, the correction governs.

### 0. This task is split. You are implementing Task 13a only.

The task text bundles two independently-rejectable deliverables, and the second
one needs IR that no task in this plan specifies. Splitting them:

- **Task 13a (yours): the CREATE side.** Steps 3, 4 and 6, plus the CREATE-side
  tests. Everything below is scoped to that.
- **Task 13b (not yours): the MATCH side.** Step 5, Step 7's desugaring
  goldens, and the new plan IR they need.

Why: Step 5 calls `self.scan_relationship(...)` and `self.join_role_player(...)`
as if they exist. They do not, and neither does anything they could be built
from. `ir::PlanKind` has `NodeScan` but **no relation scan**, and
`ir::RoleExpand` (`graph/ir/src/plan.rs:80`) is strictly node → relationship →
node: it has `from: BindingId`, `from_node_source` and `target_node_source`, so
it cannot express "the relation is already bound, join out to the player of one
role". `MATCH [x:Transcription](scribe: s) RETURN s.id` therefore cannot be
planned with today's IR at all. That is a genuine design gap in the plan, not a
stale reference, and it belongs in its own task with its own review.

**Do not attempt the MATCH side. Do not add a relation-scan plan node.** If you
find yourself editing `graph/ir/src/plan.rs`, you have left your scope.

### 1. Scope of Task 13a

Deliver:
- The four `BindError` variants in Step 3.
- `bind_create_role_pattern` (Step 4).
- `classify_statement` visiting `PatternElement::Roles` (Step 6) — but only far
  enough that a role pattern under CREATE classifies as `Write`. A role pattern
  under MATCH must keep the Task 12 bind error until 13b.
- Un-ignore **only** the two tests in `graph/frontend/tests/nary_relations.rs`
  (`#[ignore = "surface syntax lands in Task 12"]`, lines 21 and 50). Both are
  CREATE tests and both should now pass.
- **Leave both `#[ignore = "standalone role pattern lands in Task 12"]`
  attributes in `graph/frontend/tests/desugaring_golden.rs` in place.** They are
  MATCH goldens and belong to 13b. Update their reason strings to say Task 13b.
- The CREATE-side tests from Step 1: unknown role, missing required role,
  duplicate role name, wrong-typed player, optional role omitted. Plus one the
  task text omits and the plan's global constraints require: **a repeated player
  across two different roles must be accepted**, e.g.
  `(scribe: p, text: t, folio: p)`.

Skip from Step 1: `a_role_pattern_in_match_binds_the_relation_and_every_named_player`
and `a_match_role_pattern_may_leave_roles_unnamed`. Those are 13b's.

### 2. The test harness in Step 1 does not exist.

There is no `Session` type and no `session.run` / `session.sql` /
`session.expect_error`. `ternary_session()` (`graph/frontend/tests/fixture.rs:124`)
returns `(Arc<Database>, GraphConnection)`. The convention across this crate's
tests is:

```rust
let (_db, session) = fixture::ternary_session();
let rows = session.query(query, &Parameters::new());
```

Rewrite every test in the fixture crate's actual style. Match what
`nary_relations.rs` and its neighbours already do; do not add helpers to any
public API. (A prior task shipped a test helper as production public API on
`GraphConnection` and it was rejected — keep helpers in the test crate.)

### 3. Two of Step 1's tests cannot pass against `ternary_session` as it stands.

`ternary_session` registers exactly three roles — `scribe`, `text`, `folio` —
all `RoleCardinality::One`. `RoleSourceRegistration` has **no `optional` field
and no target list**: optionality and target types exist only on
`SemanticRole`, and the physical projection
(`semantic.rs:147`, `impl From<RelationshipRoleLayout> for SemanticRole`)
hard-codes `targets: Vec::new()` and `optional: false`. So against this fixture:

- `an_optional_role_may_be_omitted` is unwritable — there is no `witness` role,
  and a physically-registered role can never be optional.
- `a_role_rejects_a_player_of_the_wrong_type` cannot fire — `targets` is empty,
  which means unconstrained, so `RoleTargetTypeViolation` is unreachable.

Both paths must still be tested; shipping an unreachable error variant is
exactly the defect this plan's reviews keep finding. **Add a semantic-mode
ternary fixture** alongside `ternary_session` — `SemanticRoleRegistration`
(`semantic.rs:344`) carries both `optional` and `targets`. Give it the same
three roles with real targets plus an optional fourth role, and write those two
tests against it. `graph/frontend/tests/semantic_schema.rs` is your precedent
for registering a semantic schema in a test.

If that fixture turns out to be materially larger than the rest of this task,
stop and report BLOCKED with what specifically is in the way — do not silently
drop the two tests, and do not `#[ignore]` them.

### 4. Catalog signatures in Step 4 are wrong.

Task 9 added a `graph: ir::GraphId` parameter. The real signatures
(`binder.rs:144` and `:116`) are:

```rust
fn relationship_roles(&self, graph: ir::GraphId, ty: ir::RelationshipTypeId) -> Vec<SemanticRole>;
fn relationship_source_for_type(&self, graph: ir::GraphId, ty: ir::RelationshipTypeId) -> Option<ir::SourceTableId>;
```

`SemanticRole` (`semantic.rs:130`) has exactly: `role: ir::RoleId`,
`name: String`, `targets: Vec<ir::RoleTarget>`, `optional: bool`,
`cardinality: ir::RoleCardinality`. The Step 4 code's field reads are right;
only the call signatures are wrong.

### 5. AST accessors in Step 4 are wrong.

After Task 12, `ast::RoleArgument.name` is `Spanned<String>` (use `.value`) and
`.player` is `Spanned<Expression>`. `ast::RolePattern.properties` is
`Vec<(Spanned<String>, Spanned<Expression>)>`, **not** an `Option`, so
`pattern.properties.as_ref()` is wrong — pass it the way the existing
node/relationship property binding does.

### 6. Helper methods invoked by Step 4 and 5 that you must check before using.

`single_relationship_type`, `declare_relationship_binding`, `bind_property_map`,
`bind_role_player`, `resolve_declared_role` are all named as if they exist. Some
do not. Grep before calling; where one is missing, either reuse the equivalent
the arrow-form create path already uses or write it, but do not invent a name
and assume it resolves.

### 7. Preserve these two behaviours explicitly, and test them.

- **No cross-role uniqueness.** The same player under two roles is legal. The
  Step 4 code is already correct here (`seen` is keyed by `RoleId`, and the
  duplicate check fires only on a repeated *role*, not a repeated *player*) —
  keep it that way and add the test named in section 1.
- **Declaration-order sort, source-order input.** Role arguments arrive in
  source order and must be resolved by name. The final `bound.sort_by_key`
  reorders to declaration order so the writer's column list is stable. Nothing
  anywhere may index `pattern.roles` positionally. This is the recurring defect
  class of this plan — it has been caught at Tasks 4, 5, 6, 7, 9 and 10 — so
  write at least one test where source order and declaration order genuinely
  differ, rather than one where they happen to coincide.

### 8. Gate and commit.

`git add -A` is banned: it sweeps `graph/test-results/*`, which the corpus run
rewrites and which I commit separately. Stage only the files you changed, by
explicit path.

Run both `mise run corpus` and `mise run cypherbench-sample`. The gate is
**"every non-tck suite exactly at baseline, and tck-deep between 3329 and
3332"** — not a single total. tck-deep is flaky by ±2 on identical commits
(`runs.jsonl` shows SHA `2329d3fd1aa7` producing 3331 then 3330 back to back),
so the headline total legitimately lands anywhere in 8925-8928. Report the
numbers you actually get, and if a non-tck suite moves, that is yours.

Adjust the commit message to describe the CREATE side only.
