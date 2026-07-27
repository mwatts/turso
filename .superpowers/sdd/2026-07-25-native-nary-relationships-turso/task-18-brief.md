### Task 18a: execution-time player validation

> **CONTROLLER CORRECTIONS — these override the brief body below wherever
> they conflict. Every reference was verified against the tree at
> `2dea86b85`. Read this section first, then the body.**
>
> **SPLIT RULING. Task 18 is split into 18a and 18b; you are implementing
> 18a only.** The brief bundles two deliverables and one of them cannot be
> written today:
>
> - **18a (yours):** execution-time validation of role players whose type
>   is unknown at bind time — the brief's Step 4, and its second test.
> - **18b (deferred, not yours):** MERGE over role patterns — the brief's
>   Step 1 first test and Step 3.
>
> Why: `merge_clause = { MERGE ~ path_pattern ~ merge_action* }`
> (`graph/cypher/src/cypher.pest:65`) takes `path_pattern` **directly**,
> bypassing `pattern_element = { role_pattern | path_pattern }` (`:96`)
> that `create_clause = { CREATE ~ pattern }` (`:64`) goes through. So
> `MERGE [x:Transcription](scribe: p, …)` **does not parse at all** — the
> brief's first test cannot reach the binder, let alone fail for the reason
> the brief predicts. Making it parse needs a grammar change plus binder
> routing (`bind_create_role_pattern` at `binder.rs:1820` returns
> `Result<ir::CreateRelation, BindError>` — it has no merge form), which is
> a task, not a step. That is 18b. **Do not attempt it. Do not touch
> `cypher.pest`.**
>
> 18b is not blocked by Task 13b, so it will run separately and soon; you
> do not need to leave hooks for it.
>
> **A. Step 3 is 18b's, and it is also partly redundant — ignore it
> entirely.** For the record, since it may look like a gap you should
> close: MERGE already matches on all single-valued roles today. At
> `mutation.rs:1960-1977` every `One` role is collected into `fixed`, and
> `fixed` is passed to `insert_entity` (`:1991`) alongside `merge_predicates`,
> which are relationship-*type* predicates only
> (`relationship_type_predicates`, `:1868`). The brief's premise that MERGE
> "still matches on the two-endpoint key" is stale — Tasks 10/11 already
> generalized it. The genuine remaining gap is that `Many` roles cannot be
> in the merge key, which is 18b's `EXISTS`-on-spill-table work.
>
> **B. Step 4 cannot compile as written: there is no `self`.**
> `insert_relationship` is a free function
> (`graph/frontend/src/mutation.rs:1933`), not a method:
>
> ```rust
> pub(crate) fn insert_relationship(
>     connection: &Arc<Connection>,
>     catalog: &dyn GraphCompilationCatalog,
>     input: &LoweredMutationInput,
>     create: &ir::CreateRelation,
>     parameters: &Parameters,
>     values: &HashMap<ir::BindingId, Value>,
>     entity_layouts: &HashMap<ir::BindingId, (ir::SourceTableId, MutationEntityKind)>,
>     merge: bool,
> ) -> Result<(Value, bool), MutationError>
> ```
>
> So `self.check_role_target(role, player)?` is impossible. Write
> `check_role_target` as a free function taking whatever it needs
> explicitly — it needs the catalog to look the player's label or
> relationship type up. Match the file's existing free-function style.
>
> Note also the brief says to modify "`merge_relation`" in its **Files**
> line: **no such function exists.** MERGE and CREATE both flow through
> `insert_relationship`'s `merge: bool` parameter.
>
> **C. Place the check where the brief's own rationale requires.** The
> stated goal is that the savepoint never holds a relation whose
> participation violates the schema. The role players are resolved in the
> loop at `:1962-1977`, and the first physical write is the `insert_entity`
> call at `:1978`. Validate after resolution and **before** that call.
> Validating both `One` and `Many` players matters — a `Many` player is
> written after the relation row exists, so a late check there would leave
> a committed relation row behind.
>
> **D. `set_roles` does not exist under that name.** The brief's Step 4
> says to also validate in `set_roles`; Task 15 shipped the `SET`-roles
> path, so find the actual executor arm for `ir::Mutation::SetRoles` in
> `mutation.rs` and apply the same validation there. If that arm already
> validates player types, say so in your report rather than duplicating it.
>
> **E. Every test helper the brief uses is invented.** There is no
> `session.run`, no `session.sql`, and no `session.expect_error_with_params`
> anywhere in `graph/frontend/tests/`. Use the idiom the 19 existing tests
> in `graph/frontend/tests/nary_relations.rs` actually use:
>
> - `let (database, session) = fixture::ternary_session();`
> - seed with `fixture::second_connection(&database)`,
>   `load_registered_graph(&seed, "scriptorium")`, and the local
>   `seed_node(…)` helper
> - run with `session.execute(sql, &Parameters::new())`
> - assert errors with `.expect_err("why this must fail")` (see `:328`,
>   `:422`, `:433`, `:444`, `:455`)
> - assert stored rows through a second connection with
>   `.prepare(…).run_collect_rows()`, comparing `Vec<Vec<Value>>`
>
> `Parameters` is `pub type Parameters = HashMap<String, Value>`
> (`mutation.rs:59`), so a parameterised test passes
> `&HashMap::from([("who".to_owned(), Value::from_i64(2))])` — there is no
> special named-parameter API to find.
>
> **F. Keep the brief's second test's intent, which is the valuable half.**
> A `$who` parameter carrying a `Text` identity into the `scribe` role must
> be refused, **and** `SELECT count(*) FROM transcriptions` must still be
> `0` afterwards. That second assertion is the one that distinguishes this
> task from a bind-time check — do not drop it. `CREATE` with a role
> pattern parses and binds today (Task 13a), so this test is fully
> reachable.
>
> Add a companion test that a parameterised player of the **correct** type
> still succeeds. Without it, `check_role_target` returning
> `Err` unconditionally would pass your suite.
>
> **G. A role with an empty target list accepts anything** and must be
> skipped, per the brief body. Confirm empty-target roles exist in the
> fixtures you use before relying on that path being exercised; if nothing
> covers it, add a case.
>
> **H. Gate corrections.** Use `git add` with explicit paths, never
> `git add -A`. Run **both** `mise run corpus` and
> `mise run cypherbench-sample` — the brief omits the latter. The corpus
> gate is **per suite**: `age-deep` 3042, `cqlite-deep` 113, `grafeo-deep`
> 277, `sparrowdb-deep` 2164 each **exactly** at baseline, and `tck-deep`
> within **3329-3332** (flaky by ±2 on identical commits). There is no
> single total — do not write "corpus at 8,926"; state the per-suite
> numbers you actually observed. Commit **code only**; nothing under
> `graph/test-results/` (the controller commits those separately). Run
> both gates *before* committing.

---

### Task 18: MERGE over roles, and execution-time player validation (original brief text; Steps 1-first-test and 3 are deferred to 18b)

**Files:**
- Modify: `graph/frontend/src/binder.rs` (merge path), `graph/frontend/src/mutation.rs` (`merge_relation`)
- Test: `graph/frontend/tests/nary_relations.rs`

**Interfaces:**
- Consumes: `ir::MergeRelation` (Task 11), `bind_create_role_pattern` (Task 13).
- Produces: `MutationError::RolePlayerTypeViolation { role: String, found: String }` — the execution-time twin of the bind-time `RoleTargetTypeViolation`.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn merge_matches_on_the_full_set_of_bound_required_roles() {
    // Matching on a subset would make a second MERGE with a different folio
    // silently update the first transcription instead of creating a second
    // one, collapsing two distinct assertions into one.
    let session = ternary_session();
    let create = "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}) \
                  MERGE [x:Transcription](scribe: p, text: t, folio: f)";
    session.run(create);
    session.run(create);
    assert_eq!(
        session.sql("SELECT count(*) FROM transcriptions")[0],
        vec!["1"],
        "the second MERGE matched the first relation"
    );

    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (g:Folio {id: 6}) \
         MERGE [x:Transcription](scribe: p, text: t, folio: g)",
    );
    assert_eq!(
        session.sql("SELECT count(*) FROM transcriptions")[0],
        vec!["2"],
        "a different folio is a different assertion"
    );
}

#[test]
fn a_parameterised_player_of_the_wrong_type_is_refused_before_any_write() {
    // A parameter's type is unknown at bind time. Checking it after the
    // INSERT would leave a relation whose participation violates the schema
    // the moment the savepoint is not rolled back.
    let session = ternary_session();
    let error = session.expect_error_with_params(
        "MATCH (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: $who, text: t, folio: f)",
        &[("who", "2")], // a Text identity, not a Person
    );
    assert!(error.contains("scribe"), "{error}");
    assert_eq!(
        session.sql("SELECT count(*) FROM transcriptions")[0],
        vec!["0"],
        "no relation row survived the refusal"
    );
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --test nary_relations merge`
Expected: FAIL — MERGE still matches on the two-endpoint key.

- [ ] **Step 3: Match MERGE on the bound required roles**

In `graph/frontend/src/mutation.rs`, build the merge probe from every bound
role rather than from a start/end pair:

```rust
    /// The match key is the full set of bound required roles. A subset key
    /// would let two relations that differ in an unnamed role collapse into
    /// one, which is a silent loss of an assertion.
    fn merge_probe_predicate(
        layout: &RelationshipTableLayout,
        roles: &[ir::RoleBinding],
        values: &HashMap<ir::RoleId, Value>,
    ) -> Result<String, MutationError> {
        let mut clauses = Vec::with_capacity(roles.len());
        for binding in roles {
            let role = layout
                .role(binding.role)
                .ok_or(MutationError::UnknownRole { role: binding.role })?;
            let value = values
                .get(&binding.role)
                .ok_or(MutationError::UnknownRole { role: binding.role })?;
            clauses.push(match &role.spill_table {
                None => format!(
                    "{} = {}",
                    quote_identifier(&role.column),
                    sql_value(value)
                ),
                // A many-valued role matches on membership; MERGE over a Many
                // role therefore matches a relation that already has this
                // player in that role.
                Some(table) => format!(
                    "EXISTS (SELECT 1 FROM {} WHERE relation_id = {}.{} AND node_id = {})",
                    quote_identifier(table),
                    quote_identifier(&layout.table),
                    quote_identifier(&layout.identity_column),
                    sql_value(value)
                ),
            });
        }
        Ok(clauses.join(" AND "))
    }
```

- [ ] **Step 4: Validate dynamic players before writing**

In `insert_relationship` and `set_roles`, resolve every role player and check it
against the role's target types **before** the first `INSERT` or `UPDATE`:

```rust
    // Bind-time checking cannot see a parameter's type. Validating here, before
    // any physical write, keeps the savepoint from ever containing a relation
    // whose participation violates the schema.
    for (role, player) in &resolved {
        self.check_role_target(role, player)?;
    }
```

`check_role_target` looks the player's label or relationship type up from the
snapshot and raises:

```rust
    #[error("role `{role}` does not accept {found}")]
    RolePlayerTypeViolation { role: String, found: String },
```

A role with an empty target list accepts anything and is skipped.

- [ ] **Step 5: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend --test nary_relations`
Expected: PASS.

- [ ] **Step 6: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend
mise run corpus
git add -A
git commit -S -m "graph/mutation: merge on the bound roles and check dynamic players first

MERGE probes on the full set of bound required roles. A subset key would let
two relations differing only in an unnamed role collapse into one, silently
losing an assertion.

Role players that arrive as parameters have no type at bind time, so they
are validated against the role's target types before the first physical
write rather than after, keeping the mutation savepoint from ever holding a
relation whose participation violates the schema.

Tests: nary_relations merge identity and parameterised-player refusal;
corpus at 8,926."
```

---

