### Task 15: Role updates after create

> **CONTROLLER CORRECTIONS — these override the brief body below wherever
> they conflict. Every item was verified against the tree at `c8c859820`.
> Read this section first, then the body.**
>
> **A. The syntax assumption stands, with one narrowing.** `SET [x](role:
> player)` is confirmed unambiguous: `cypher.pest:69` is
> `set_item = { set_property_item | set_merge_item | set_replace_item |
> set_label_item }`, `set_property_item` begins with `property_target` and
> the other three with `identifier`, so nothing else can start with `[`.
> `role_arguments` already exists at `cypher.pest:97` from Task 12 — reuse
> it. Put `set_role_item` first in the alternation as the brief says.
>
> Note the deliberate narrowing: `role_pattern` (`cypher.pest:96`) is
> `relationship_body ~ role_arguments`, which admits types and a property
> map. `set_role_item` takes a bare `identifier` — the relation is already
> bound, so re-stating its type would be a second source of truth. Keep it
> bare.
>
> **B. Every test in the body binds the relation with `MATCH
> [x:Transcription](text: t)`. That is Task 13b, which is not implemented.**
> Bind the relation with today's arrow form instead, against the
> `witnessed_session` fixture Task 14a added to
> `graph/frontend/tests/fixture.rs` (one `Person` node source, one `KNOWS`
> relationship source over `relationships` with roles `start`/`end` (`One`)
> and `witness` (`Many`), spill table `relationships__witness`):
>
> ```text
> MATCH (a:Person)-[r:KNOWS]->(b:Person), (q:Person {id: 4}) SET [r](start: q)
> ```
>
> That reaches the same binder and executor paths the ternary form would.
> Do not create a ternary fixture and do not wait on 13b — this task ships
> in full today.
>
> There is no `session` value with `.run()`, `.sql()`, `.query()`, or
> `.expect_error()`; those four helpers do not exist and the body's test
> code is pseudocode. `witnessed_session()` returns `(Arc<Database>,
> GraphConnection)`. Copy the real style from
> `graph/frontend/tests/nary_relations.rs:65-131`: seed through
> `fixture::second_connection(&database)`, write through
> `session.execute(query, &Parameters::new())`, assert by preparing raw SQL
> on a second connection and comparing `Vec<Vec<Value>>`.
>
> **C. `ast::SetItem` variants are struct variants carrying `Spanned<T>`,
> not tuple variants carrying bare `String`** (`ast.rs:91-112`). Do not add
> `Roles(RoleUpdate)` with `relation: String`. Match the file:
>
> ```rust
> /// `SET [x](scribe: q)` — repoint one or more roles of an already-bound
> /// relation. Setting a many-valued role replaces its whole player set.
> Roles {
>     relation: Spanned<String>,
>     roles: Vec<RoleArgument>,
>     span: Span,
> },
> ```
>
> `ast::RoleArgument` (`ast.rs:211`) already has `name: Spanned<String>`,
> `player: Spanned<Expression>`, `span`. Do not introduce a `RoleUpdate`
> struct.
>
> **D. Drop `replace_many` from `ir::SetRoles`.** It is a pure function of
> the roles' cardinality, which the executor already reads off
> `RelationshipTableLayout` — `insert_relationship` does exactly that
> today via `role.cardinality` / `role.spill_table`. A field that restates
> derivable data is a second source of truth that can disagree with the
> first. `SetRoles { relation, source, roles }` is the whole IR.
>
> `ir::Mutation` is the enum at `graph/ir/src/mutation.rs:15`. Adding a
> variant ripples to every exhaustive match over it — expect to fix sites
> in `graph/frontend/src/mutation.rs` and `graph/frontend/src/binder.rs`
> (there is a match at `binder.rs:7551`). Do not add a catch-all arm to
> silence the ripple; fix each site.
>
> **E. `bind_role_player` does not exist.** Task 13a inlined the
> target-type check in `bind_create_role_pattern` at `binder.rs:1817-1852`.
> Extract it into a shared helper both the create and update paths call —
> two copies of a type check that must agree is exactly the divergence this
> plan keeps hitting. Say in your report what you named it.
>
> **F. Step 6's executor code is the wrong shape entirely.**
> `graph/frontend/src/mutation.rs` is free functions, not a struct with
> methods. There is no `self`, no `self.layout`, no
> `self.resolve_relation_id`, no `self.resolve_binding_value`, no
> `self.execute_internal`, no `sql_value`, and no
> `MutationError::NonIntegerPlayer`. The identifier helper in scope is
> `quoted_identifier`, not `quote_identifier`.
>
> Your arm goes in `execute_operation` beside the others; model it on the
> `ir::Mutation::SetProperty` arm at `mutation.rs:1304-1351`. That arm shows
> the real vocabulary: `mutation_source`, `entity_table`,
> `values.get(&…).ok_or(MutationError::MissingBinding(…))`,
> `identity_parameter`, and `run_ignore(connection, &sql, parameters,
> &internal)`. **Bind the player values as parameters. Do not interpolate
> them into SQL.** Task 14a's spill write is your model for the spill side.
>
> Keep Step 6's real requirement: group the arguments by role before
> executing, so two arguments naming one `Many` role in a single `SET`
> both land — the spill delete runs once per role, not once per argument.
>
> **G. The null test needs no new error variant.** `bind_create_role_pattern`
> already refuses any player that is not a bound variable
> (`binder.rs:1809-1814`, `at_unsupported("a role player that is not a bound
> variable")`), and `cypher::Expression::Null` (`ast.rs:251`) hits that path.
> Do not reintroduce `MissingRequiredRole` for this — Task 13a deleted a
> variant for exactly this reason and the reviewer ruled that correct.
> Instead make the shared helper's refusal name the role, e.g.
> `at_unsupported(argument.player.span, &format!("clearing role `{}` — there
> is no null player", role.name))`, so the test's `.contains("start")`
> assertion is testing something real.
>
> Also carry over the create path's duplicate rule: a repeated `One` role in
> one `SET` is refused; a repeated `Many` role is not.
>
> **H. The wrong-type test cannot use `witnessed_session`** — all three of
> its roles target `Person`, so no target-type constraint bites. Task 13a
> tested target-type refusal against `RoledCatalog`, the hand-rolled
> `GraphCatalogSnapshot` local to `nary_relations.rs`, driving `bind_mutation`
> directly with no database. Do the same here. If `RoledCatalog` cannot bind
> a relation variable for a `SET` to target, report that as a finding rather
> than dropping the test.
>
> **I. Step 8 gate corrections.** `cargo test -p turso_cypher` names a
> package that does not exist — it is `turso_graph_cypher`. Use `git add`
> with explicit paths, not `git add -A`. Run `mise run cypherbench-sample`
> as well as `mise run corpus`; run both **before** committing, and commit
> the code only — the `graph/test-results/` rows are committed separately by
> the controller. The corpus gate is **per suite**: every non-`tck` suite
> exactly at its baseline, and `tck-deep` within 3329-3332 (flaky by ±2 on
> identical commits). There is no single total — do not write "corpus at
> 8,926"; state the per-suite result you actually observed.

---

### Task 15: Role updates after create (original brief text, superseded above)

**Assumption, not a spec decision.** The spec lists role updates in v1 but gives
no syntax. This task uses `SET [t](scribe: s2)` — the standalone role pattern as
a `SET` item. `[` cannot begin a `set_item` today, so it is unambiguous, and the
form matches the create exactly. If a reviewer prefers another spelling, this is
the task to change.

**Files:**
- Modify: `graph/cypher/src/cypher.pest` (`set_item`), `graph/cypher/src/ast.rs`, `graph/cypher/src/parser.rs`
- Modify: `graph/ir/src/mutation.rs` (`SetRoles`), `graph/frontend/src/binder.rs`, `graph/frontend/src/mutation.rs`
- Test: `graph/frontend/tests/nary_relations.rs`

**Interfaces:**
- Produces:
  - Grammar: `set_item = { set_role_item | set_property_item | set_merge_item | set_replace_item | set_label_item }`, `set_role_item = { "[" ~ identifier ~ "]" ~ role_arguments }`.
  - `ast::SetItem::Roles(RoleUpdate { relation: String, roles: Vec<RoleArgument>, span })`
  - `ir::Mutation::SetRoles(SetRoles { relation: BindingId, source: SourceTableId, roles: Vec<RoleBinding>, replace_many: bool })`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn a_single_valued_role_can_be_repointed_after_create() {
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f)",
    );
    session.run(
        "MATCH [x:Transcription](text: t), (q:Person {id: 4}) SET [x](scribe: q)",
    );
    assert_eq!(session.sql("SELECT scribe FROM transcriptions")[0], vec!["4"]);
    assert_eq!(
        session.sql("SELECT count(*) FROM transcriptions")[0],
        vec!["1"],
        "an update repoints the relation; it does not create a second one"
    );
}

#[test]
fn setting_a_many_valued_role_replaces_its_whole_player_set() {
    // Replace, not append. Append has no syntax to undo, and a SET that
    // silently accumulated would make the same statement run twice mean
    // something different from running it once.
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}), (w:Person {id: 4}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f, witness: w)",
    );
    session.run(
        "MATCH [x:Transcription](text: t), (w2:Person {id: 5}) SET [x](witness: w2)",
    );
    let rows = session.sql("SELECT node_id FROM transcriptions__witness");
    assert_eq!(rows, vec![vec!["5"]], "the previous witness is gone");
}

#[test]
fn a_role_update_rejects_a_player_of_the_wrong_type() {
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f)",
    );
    let error = session.expect_error(
        "MATCH [x:Transcription](text: t) SET [x](scribe: t)",
    );
    assert!(error.contains("scribe"), "{error}");
}

#[test]
fn a_role_update_cannot_unset_a_required_role() {
    // There is no null player. Clearing a required role would leave a
    // half-stated assertion behind, which is the same thing the create path
    // refuses.
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f)",
    );
    assert!(session
        .expect_error("MATCH [x:Transcription](text: t) SET [x](scribe: null)")
        .contains("scribe"));
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --test nary_relations role_update`
Expected: FAIL — `SET [x](...)` is a parse error.

- [ ] **Step 3: Extend the grammar and AST**

```pest
set_item = { set_role_item | set_property_item | set_merge_item | set_replace_item | set_label_item }
set_role_item = { "[" ~ identifier ~ "]" ~ role_arguments }
```

`set_role_item` goes first so a leading `[` commits to it.

```rust
/// `SET [x](scribe: q)` — repoint one or more roles of an already-bound
/// relation. Setting a many-valued role replaces its whole player set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleUpdate {
    pub relation: String,
    pub roles: Vec<RoleArgument>,
    pub span: Span,
}
```

added to `ast::SetItem` as `Roles(RoleUpdate)`, with a `walk_set_role_item`
reusing the `role_arguments` walker from Task 12.

- [ ] **Step 4: Add the IR**

```rust
/// Repoint roles of an existing relation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetRoles {
    pub relation: BindingId,
    pub source: SourceTableId,
    pub roles: Vec<RoleBinding>,
    /// True when any named role is many-valued: its spill rows are deleted
    /// before the new players are written, so SET replaces rather than
    /// appends and running the statement twice means what running it once
    /// means.
    pub replace_many: bool,
}
```

as `Mutation::SetRoles(SetRoles)`.

- [ ] **Step 5: Bind it**

Reuse `bind_role_player` from Task 13 for the target-type check. The
required-role check does **not** apply — an update names a subset by design —
but a null or missing player for a named role is refused:

```rust
        if matches!(argument.player, ast::Expression::Null) {
            return Err(BindError::MissingRequiredRole {
                relationship_type: type_name.clone(),
                role: role.name.clone(),
                span_start: argument.span.start,
                span_end: argument.span.end,
            });
        }
```

- [ ] **Step 6: Execute it**

In `graph/frontend/src/mutation.rs`, add:

```rust
    fn set_roles(&mut self, update: &ir::SetRoles) -> Result<(), MutationError> {
        let layout = self.layout(update.source)?;
        let relation_id = self.resolve_relation_id(update.relation)?;
        let mut assignments = Vec::new();
        for binding in &update.roles {
            let role = layout
                .role(binding.role)
                .ok_or(MutationError::UnknownRole { role: binding.role })?;
            let player = self.resolve_binding_value(binding.value)?;
            match &role.spill_table {
                None => assignments.push(format!(
                    "{} = {}",
                    quote_identifier(&role.column),
                    sql_value(&player)
                )),
                Some(table) => {
                    self.execute_internal(&format!(
                        "DELETE FROM {} WHERE relation_id = {relation_id}",
                        quote_identifier(table)
                    ))?;
                    self.execute_internal(&format!(
                        "INSERT INTO {}(relation_id, node_id) VALUES ({relation_id}, {})",
                        quote_identifier(table),
                        player.as_integer().ok_or(MutationError::NonIntegerPlayer)?
                    ))?;
                }
            }
        }
        if !assignments.is_empty() {
            self.execute_internal(&format!(
                "UPDATE {} SET {} WHERE {} = {relation_id}",
                quote_identifier(&layout.table),
                assignments.join(", "),
                quote_identifier(&layout.identity_column)
            ))?;
        }
        Ok(())
    }
```

Two arguments naming one many-valued role in a single `SET` must both land: the
delete runs once per role, not once per argument. Group the arguments by role
before executing.

- [ ] **Step 7: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend --test nary_relations`
Expected: PASS.

- [ ] **Step 8: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_cypher -p turso_graph_frontend
mise run corpus
git add -A
git commit -S -m "graph/frontend: allow repointing roles after create

SET [x](role: player) repoints roles of an already-bound relation, using the
same standalone role syntax as the create. Setting a many-valued role
replaces its whole player set rather than appending, so running the same
statement twice means what running it once means.

A role update names a subset by design, so the required-role check does not
apply, but a null player is refused: there is no way to leave a required
role unfilled.

Tests: nary_relations role updates over single-valued repointing,
many-valued replacement, target-type refusal, and null refusal; corpus at
8,926."
```

---

