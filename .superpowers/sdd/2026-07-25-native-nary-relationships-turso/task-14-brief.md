### Task 14a: Many-valued roles — write and delete

> **CONTROLLER CORRECTIONS — these override the brief body below wherever
> they conflict. Every item was verified against the tree at `754dce74d`.
> Read this section first, then the body.**
>
> **A. Scope: this is Task 14a, not Task 14.** Three of the brief's four
> tests, and all of Step 5, require a *standalone role pattern in `MATCH`*
> (`MATCH [x:Transcription](witness: w)`). That is Task 13b, which is not
> implemented and is parked on a design decision. Nothing can set
> `RoleExpand.from_role`/`to_role` to a `Many` role today — the binder only
> ever fills them from `start`/`end` for arrow patterns — so Step 5 would be
> unreachable dead code.
>
> **Do Steps 1-4, 6, 7, 8. Skip Step 5 entirely.** Do not write
> `role_join_expression`. Do not write
> `a_hop_through_a_many_valued_role_returns_every_player`. Those move to
> Task 14b, after 13b lands.
>
> **B. `ternary_session()` has no `witness` role, and you must not add one.**
> It returns `(Arc<Database>, GraphConnection)` — there is no `session` value
> with `.run()`, `.sql()`, `.query()`, or `.expect_error()`. Those four
> helpers do not exist; the brief's test bodies are pseudocode. Copy the real
> style from `graph/frontend/tests/nary_relations.rs:65-131`:
> `let (database, session) = fixture::ternary_session();`, seed via
> `fixture::second_connection(&database)` + the local `seed_node` helper,
> write via `session.execute(query, &Parameters::new())`, assert by preparing
> raw SQL on a second connection and comparing `Vec<Vec<Value>>`.
>
> Adding a role to `ternary_session` is not an option: physical role
> registrations project to `SemanticRole` with `optional: false`
> (`semantic.rs:147`), and `bind_create_role_pattern` requires every
> non-optional declared role (`binder.rs:1856`). A new `witness` role would
> break all three existing CREATE tests.
>
> **C. Add a new fixture instead — and make it binary + Many.** Add
> `pub fn witnessed_session() -> (Arc<Database>, GraphConnection)` to
> `graph/frontend/tests/fixture.rs`: one `Person` node source over
> `people(id INTEGER PRIMARY KEY)`, one relationship source `KNOWS` over
> `relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER)` with
> three `RoleSourceRegistration`s — `start`/col `src`/`Person`/`One`,
> `end`/col `dst`/`Person`/`One`, `witness`/`Person`/`Many`. Model it on
> `ternary_session` (fixture.rs:124), not on `social_graph_connection`:
> do **not** eagerly call `SnapshotStore::refresh`.
>
> Binary-plus-`Many` is the deliberate choice, and it is what makes 14a
> testable at all: the relation can be created by a role pattern
> (`CREATE [x:KNOWS](start: a, end: b, witness: w1, witness: w2)`, which 13a
> supports) *and* bound for deletion by today's arrow syntax
> (`MATCH (a:Person)-[r:KNOWS]->(b:Person) DELETE r`), with no 13b dependency
> on either side. A ternary relation can be created but never bound for
> deletion until 13b.
>
> For the `witness` role's `column` field pass the empty string — the
> `RelationshipRoleLayout::column` doc already reads "Empty for `Many` roles",
> `structural_columns()` already filters `Many` out (`lowering.rs:57-66`),
> and `register_graph` already routes `Many` to `install_spill_table`
> (`catalog.rs:526-533`), which creates `relationships__witness(relation_id,
> node_id)` with forward and reverse indexes. If the empty column turns out
> to break an unrelated code path, report it rather than working around it.
>
> Check while you are there that binding an arrow pattern over a three-role
> source works — `RoleExpand` resolves `start`/`end` by name and should
> ignore `witness`, but if some binary-only helper asserts `roles.len() == 2`,
> that is a real finding: report it, do not silently special-case it.
>
> **D. Step 3 is wrong in four ways.** `insert_relationship`
> (`mutation.rs:1836`) is a free function, not a method: there is no `self`
> and no `execute_internal`. There is no `relation_id` local — the identity
> comes back from `insert_entity`, so the spill writes must happen *after*
> that call and use its returned `Value`. `MutationError::NonIntegerPlayer`
> does not exist. And do not interpolate the player value into SQL: use the
> file's existing `run_ignore(connection, sql, parameters, &internal)` +
> `identity_parameter` binding mechanism, the same way `delete_entity` does
> (`mutation.rs:2101-2122`). The identifier helper in scope in `mutation.rs`
> is `quoted_identifier`, not `quote_identifier`.
>
> Delete the `assert!(spilled.is_empty(), ...)` at `mutation.rs:1881`.
>
> **E. Step 4 names only half the delete surface.** The relationship delete
> path is the `else` branch of `delete_entity` (`mutation.rs:2123-2155`) —
> yes, add the per-`Many`-role spill delete there. But the **node** delete
> path (`mutation.rs:2034-2094`) is where relation rows actually die under
> `DETACH DELETE`, and it deletes them with a bare
> `DELETE FROM <relation> WHERE <start> = $id OR <end> = $id`, leaving every
> spill row behind. Under the brief's own stated rationale — "no dangling
> participant can surface as a live player on a later hop" — that is the
> larger hole. Fix both, and cover both with tests. The `witnessed_session`
> fixture reaches both paths.
>
> (Note that the node delete path resolves relations only through
> `relationship_endpoint_sources`, which is two-role-only, so an n-ary
> relation is invisible to `DETACH DELETE` entirely. That is pre-existing and
> out of scope for 14a — report it, do not fix it here.)
>
> **F. Step 6: `BindError::RoleCardinalityViolation` does not exist.** Task
> 13a removed it as unreachable and the reviewer ruled that defensible. The
> duplicate-role refusal that already ships is
> `BindError::DuplicateRoleArgument` (`binder.rs:1791-1798`), and its test
> `naming_one_role_twice_is_refused_rather_than_last_write_wins` already
> passes. Step 6's "more than two arguments name the same `One` role" is an
> off-by-one: a duplicate is two, not three.
>
> So Step 6 reduces to: **do not weaken the existing duplicate check.** It
> currently rejects a repeated role of *any* cardinality via the `seen` set.
> `Many` roles must now be allowed to repeat, `One` roles must still be
> refused. Keep `DuplicateRoleArgument` for the `One` case — do not
> reintroduce a variant. The brief's
> `a_single_valued_role_given_two_players_is_refused` is therefore a
> regression guard on existing behavior under the new code path; keep it,
> written against `witnessed_session` with a repeated `start`.
>
> **G. Remove the binder's `Many` guard.** `binder.rs:1799-1808` currently
> returns `at_unsupported` for any `Many` role, with a comment naming Task 14.
> Delete it — that is the guard 14a replaces with real support. Task 13a
> deliberately shipped it untested and the ledger records the debt; it is
> discharged by this task's tests, not by a new test of the guard.
>
> **H. `single_valued_roles()` (`catalog.rs:136`) and `structural_columns()`
> (`lowering.rs:57`) are two implementations of one predicate.** This task
> was flagged as the one to watch it in. If you touch either, say so in the
> report; do not unify them unprompted.
>
> **I. Step 8 gate corrections.** Use `git add` with explicit paths, not
> `git add -A`. The corpus gate is **per suite**: every non-`tck` suite
> exactly at its baseline, and `tck-deep` within 3329-3332 (it is flaky by
> ±2 on identical commits). There is no single total to hit — do not write
> "corpus at 8,926" in the commit message; state the per-suite result you
> actually observed. Run `mise run corpus` and `mise run cypherbench-sample`
> **before** committing, and commit the code change only — the
> `graph/test-results/` rows are committed separately by the controller.
> Retitle the commit subject to `graph/frontend: write and delete
> many-valued role players` and drop the hop paragraph from its body.

---

### Task 14: Many-valued roles (original brief text, superseded above)

**Files:**
- Modify: `graph/frontend/src/mutation.rs` (`insert_relationship`, delete path)
- Modify: `graph/frontend/src/lowering.rs` (spill-table join)
- Modify: `graph/frontend/src/binder.rs` (`RoleCardinalityViolation` for a `One` role given a list)
- Test: `graph/frontend/tests/nary_relations.rs`

**Interfaces:**
- Consumes: `RelationshipRoleLayout::spill_table` (Task 4), the `spilled` vector from Task 10.
- Produces: no new public API.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn a_many_valued_role_holds_several_players_in_one_relation() {
    // Two witnesses to one transcription is one assertion with two players,
    // not two assertions. Splitting it into two rows would double-count the
    // transcription in every aggregate.
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}), \
               (w1:Person {id: 4}), (w2:Person {id: 5}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f, witness: w1, witness: w2)",
    );
    assert_eq!(
        session.sql("SELECT count(*) FROM transcriptions")[0],
        vec!["1"],
        "one relation row"
    );
    assert_eq!(
        session.sql("SELECT count(*) FROM transcriptions__witness")[0],
        vec!["2"],
        "two spilled players"
    );
}

#[test]
fn a_hop_through_a_many_valued_role_returns_every_player() {
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}), \
               (w1:Person {id: 4}), (w2:Person {id: 5}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f, witness: w1, witness: w2)",
    );
    let mut ids = session.query("MATCH [x:Transcription](witness: w) RETURN w.id");
    ids.sort();
    assert_eq!(ids, vec![vec!["4"], vec!["5"]]);
}

#[test]
fn deleting_a_relation_removes_its_spilled_players() {
    // A spill row pointing at a deleted relation is a dangling participant
    // that a later hop would surface as a live player.
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}), (w:Person {id: 4}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: f, witness: w)",
    );
    session.run("MATCH [x:Transcription](scribe: s) DELETE x");
    assert_eq!(
        session.sql("SELECT count(*) FROM transcriptions__witness")[0],
        vec!["0"]
    );
}

#[test]
fn a_single_valued_role_given_two_players_is_refused() {
    let session = ternary_session();
    let error = session.expect_error(
        "MATCH (p:Person {id: 1}), (q:Person {id: 4}), (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription](scribe: p, scribe: q, text: t, folio: f)",
    );
    assert!(error.contains("scribe"), "{error}");
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --test nary_relations many_valued`
Expected: FAIL — the assertion added in Task 10 fires:
`many-valued roles are written in a later step`.

- [ ] **Step 3: Write the spill rows**

Replace that assertion in `insert_relationship` with:

```rust
    for (role, player) in spilled {
        let table = role
            .spill_table
            .as_ref()
            .expect("a Many role always has a spill table");
        self.execute_internal(&format!(
            "INSERT INTO {}(relation_id, node_id) VALUES ({}, {})",
            quote_identifier(table),
            relation_id,
            player.as_integer().ok_or(MutationError::NonIntegerPlayer)?
        ))?;
    }
```

- [ ] **Step 4: Delete the spill rows with the relation**

In the relationship delete path, after deleting the relation row, add one delete
per many-valued role:

```rust
    for role in layout
        .roles
        .iter()
        .filter(|role| role.cardinality == ir::RoleCardinality::Many)
    {
        let table = role.spill_table.as_ref().expect("Many role spill table");
        self.execute_internal(&format!(
            "DELETE FROM {} WHERE relation_id = {}",
            quote_identifier(table),
            relation_id
        ))?;
    }
```

- [ ] **Step 5: Join through the spill table when hopping a `Many` role**

In `lower_role_expand`, when either role of the pair is many-valued, the join
goes through the spill table instead of a column:

```rust
    /// A `Many` role has no column on the relation table, so the hop runs
    /// relation -> spill -> player. The spill table is indexed in both
    /// directions, so this is an index probe from whichever side is bound.
    fn role_join_expression(
        layout: &RelationshipTableLayout,
        role: &RelationshipRoleLayout,
        relationship_alias: &str,
        spill_alias: &str,
    ) -> String {
        match &role.spill_table {
            None => format!("{relationship_alias}.{}", quote_identifier(&role.column)),
            Some(table) => format!(
                "(SELECT {spill_alias}.node_id FROM {} {spill_alias} \
                 WHERE {spill_alias}.relation_id = {relationship_alias}.{})",
                quote_identifier(table),
                quote_identifier(&layout.identity_column)
            ),
        }
    }
```

and emit a `JOIN` rather than a scalar subquery when the role is on the produced
side, so a relation with two witnesses yields two rows:

```rust
        if role.spill_table.is_some() {
            joins.push(format!(
                "JOIN {} {spill_alias} ON {spill_alias}.relation_id = {relationship_alias}.{}",
                quote_identifier(role.spill_table.as_ref().expect("checked")),
                quote_identifier(&layout.identity_column)
            ));
        }
```

- [ ] **Step 6: Refuse a list for a single-valued role**

In `bind_create_role_pattern`, the duplicate check from Task 13 already refuses
a repeated `One` role. Extend it to report `RoleCardinalityViolation` with the
observed count when more than two arguments name the same `One` role, so the
message says how many players were offered.

- [ ] **Step 7: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend --test nary_relations`
Expected: PASS.

- [ ] **Step 8: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend
mise run corpus
mise run cypherbench-sample
git add -A
git commit -S -m "graph/frontend: implement many-valued roles

A Many role stores its players in <relation>__<role>, indexed in both
directions. Creating a relation writes one row plus one spill row per
player; deleting it removes the spill rows, so no dangling participant can
surface as a live player on a later hop.

A hop through a many-valued role joins the spill table rather than reading a
column, so a relation with two players in one role yields two rows.

Tests: nary_relations many-valued create, hop, delete, and the refusal of a
list for a single-valued role; corpus at 8,926; cypherbench at baseline."
```

---

