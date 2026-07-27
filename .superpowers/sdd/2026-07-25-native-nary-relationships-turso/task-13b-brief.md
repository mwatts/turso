### Task 13b: relation-anchored MATCH over a standalone role pattern

`MATCH [x:Transcription](scribe: s, folio: g) RETURN s.id, g.id` must bind and
execute. CREATE-side binding landed in Task 13a; this is the read side.

Every reference below was verified against the tree at the commit you are
branching from. Where the original plan text conflicts with the tree, this
brief governs — the conflicts are called out explicitly so you do not
"fix" the brief back toward the plan.

**Files:**
- Modify: `graph/ir/src/plan.rs` (new plan node(s))
- Modify: `graph/frontend/src/binder.rs` (MATCH-side role pattern)
- Modify: `graph/frontend/src/lowering.rs` (lower the new node(s))
- Modify: `graph/frontend/src/compiler.rs:190-203` (traversal-snapshot decision)
- Test: `graph/frontend/tests/nary_relations.rs`,
  `graph/frontend/tests/desugaring_golden.rs`

---

## The governing ruling — read this before anything else

The human was asked what "the arrow form and the role form bind to the same
plan" must mean, and ruled:

> **(B) Same layout and SQL.** The role-pattern form must produce the same
> physical layout and emit the same SQL as today's arrow form, but the plan
> node itself may differ. **Plan-node identity is NOT required.**

Three consequences, all binding:

1. **The arrow path must not be re-planned.** `MATCH (a)-[r:KNOWS]->(b)` keeps
   binding to `RoleExpand` exactly as it does today. Do not touch its binder
   path, its lowering, or its SQL. The corpus is the blast radius and it is
   the reason (A) was rejected.
2. **The two existing goldens must be rewritten**, because they assert the
   thing the ruling says is not the contract. See Step 4.
3. **No arity branch.** No `if roles.len() == 2`, no `is_binary`, no
   hard-coded `"start"`/`"end"` in general machinery. The general
   relation-anchored form must *degenerate* to today's SQL when the relation
   happens to have two roles with node players — that is a property you get
   from writing it generally, not a special case you write by hand. If you
   find yourself adding a two-role branch to make a golden pass, stop and
   report BLOCKED instead.

---

- [ ] **Step 1: Do not reuse `RoleExpand` — the plan's own design for this
      task does not hold**

The original plan text says to add `join_role_player` emitting

> a `RoleExpand` whose `from_role` is the role being joined and whose
> `to_role` is the same role — the relation is already bound, so the expand
> runs relation → player.

That is wrong against the tree, in two independent ways. Verify both yourself
before designing anything:

- `RoleExpand.from_node_source` is consumed as a **node** source. At
  `graph/frontend/src/lowering.rs:1479-1492` the lowering emits a source
  filter `source_q.<source_col> = expand.from_node_source.get()` against the
  input binding. The anchor of a role pattern is the *relation*, whose source
  is a relationship source, so this filter would be comparing the wrong
  thing.
- With `from_role == to_role`, `from_column` and `to_column` (resolved just
  below, at `lowering.rs:1497+`) are the **same column**, so the join
  degenerates to a self-join on one column. That is not "relation → player".

So this task needs a genuine new plan node. `PlanKind`
(`graph/ir/src/plan.rs:39-56`) currently has no relation scan — `NodeScan`,
`RoleExpand`, `GraphExpand`, and the relational operators, nothing that
anchors on a relationship table.

The shape recorded during the 13a split, which you may adopt or improve on:

```rust
RelationScan { graph, source, binding, relationship_types }
RoleJoin { input, relationship, relationship_source, role, player,
           player_node_source, bound_player }
```

`RelationScan` anchors on the relationship table; one `RoleJoin` per **named**
role joins out to that role's player. This composes to any arity by
construction — n named roles is n joins — which is why it has no arity
branch. If you choose a different decomposition, say why in your report; the
requirement is generality and the SQL contract, not these two names.

For a `Many` role the join goes through the spill table. A `Many` role is
identified by `RelationshipRoleLayout.spill_table.is_some()`, never by name
or position. (Task 14b covers hopping *through* a `Many` role in depth; here
you need only that a named `Many` role in a MATCH role pattern resolves
correctly. If you conclude the `Many` case belongs entirely to 14b, say so
explicitly and justify it rather than leaving it silently unhandled.)

- [ ] **Step 2: Write the failing tests**

Append to `graph/frontend/tests/nary_relations.rs`. Use the idiom the existing
tests in that file actually use — `fixture::ternary_session()`, seeding
through `fixture::second_connection(&database)` +
`load_registered_graph(&seed, "scriptorium")` + the local `seed_node(…)`,
running with `session.execute(sql, &Parameters::new())`, and asserting rows
through a second connection with `.prepare(…).run_collect_rows()`. There is
no `session.run`, no `session.sql`, and no `session.query` returning
`Vec<Vec<&str>>` — the plan text invents all three.

Two behaviors are required:

```rust
#[test]
fn a_match_role_pattern_binds_the_named_players() {
    // The relation is the anchor and each named role is a join out to its
    // player. Without this, reading a ternary relation would be impossible
    // in the surface language even though the storage holds it.
}

#[test]
fn a_match_role_pattern_may_leave_roles_unnamed() {
    // Naming a subset is a projection over the relation's participants, not
    // an under-specified match: the unnamed roles are simply not bound, and
    // must not silently constrain or drop rows.
}
```

The second is the load-bearing one. Seed **two** transcriptions that share a
`scribe` but differ in `text` and `folio`, then match on `scribe` alone and
assert you get **both** rows. A fixture where the subset match and the full
match return the same rows proves nothing — that is the vacuity that made
Task 10's first attempt worthless.

- [ ] **Step 3: Run to verify they fail**

`cargo test -p turso_graph_frontend --test nary_relations`

Expected: FAIL at bind time — the MATCH-side role pattern has no binder path.

- [ ] **Step 4: Rewrite the two goldens to assert the ruling's contract**

`graph/frontend/tests/desugaring_golden.rs` is 23 lines and currently reads:

```rust
//! Arrow syntax is sugar over roles. If the two forms ever bind to different
//! IR, then a "binary" query and its role-form equivalent can disagree at
//! runtime, and the claim that binary is a layout of the role model is false.

use fixture::{bind_fixture, first_role_expand};

#[test]
#[ignore = "standalone role pattern MATCH-side binding lands in Task 13b"]
fn arrow_and_role_forms_of_the_same_pattern_bind_identically() {
    let arrow = bind_fixture("MATCH (a:Person)-[r:KNOWS]->(b:Person) RETURN b");
    let roles = bind_fixture("MATCH (a:Person), (b:Person), [r:KNOWS](start: a, end: b) RETURN b");
    assert_eq!(first_role_expand(&arrow), first_role_expand(&roles));
}

#[test]
#[ignore = "standalone role pattern MATCH-side binding lands in Task 13b"]
fn the_reversed_arrow_is_the_reversed_role_pair() {
    let arrow = bind_fixture("MATCH (a:Person)<-[r:KNOWS]-(b:Person) RETURN b");
    let roles = bind_fixture("MATCH (a:Person), (b:Person), [r:KNOWS](end: a, start: b) RETURN b");
    assert_eq!(first_role_expand(&arrow), first_role_expand(&roles));
}
```

Both assert **plan-node identity**, which the ruling explicitly says is not
the contract. Under the design in Step 1 the role form emits no `RoleExpand`
at all, so `first_role_expand` would panic on it.

Rewrite both to compare **emitted SQL**. The pieces already exist:

- `lower_relational(plan, catalog) -> Result<ast::Stmt, LowerError>` is `pub`
  (`graph/frontend/src/lowering.rs:313`) and re-exported from
  `graph/frontend/src/lib.rs:51`.
- `fixture.rs:360` already has `impl RelationalCatalogSnapshot for Catalog`,
  the same `Catalog` `bind_fixture` (`fixture.rs:402`) binds against.
- `ast::Stmt` derives `PartialEq` (`sqlite/parser/src/ast.rs:78`).
- `graph/frontend/tests/fixed_pattern_fixtures.rs:200` and
  `dialect_alignment.rs:552` are two existing precedents for
  `lower_relational(&bound.plan, &Catalog)` in a test.

So each golden becomes a comparison of the two lowered statements. **Rewrite
the module doc comment too** — as it stands it documents the identity claim
the ruling just rejected, and leaving it would make the file assert one thing
and document another. State the actual contract: arrow syntax is sugar over
roles at the level of emitted SQL; the two forms may plan differently, but if
they ever *execute* differently then binary is not a layout of the role model.

Remove both `#[ignore]` attributes.

If the two statements differ **only** in alias naming or column ordering, do
**not** paper over it by normalizing the strings or relaxing the assertion.
Stop and report it with both statements quoted — that is a controller
decision, not yours.

`first_role_expand` (`fixture.rs:417`) may become unused by this file. It is
marked `#[allow(dead_code)]` and other suites may still use it — check before
deleting, and leave it alone if anything else calls it.

- [ ] **Step 5: Fix the traversal-snapshot decision**

`graph/frontend/src/compiler.rs:190-203`, inside
`query_needs_traversal_snapshot`:

```rust
    let turso_graph_cypher::PatternElement::Path(path) = element else {
        return false;
    };
```

A `PatternElement::Roles` therefore answers "no snapshot needed" without ever
being examined. This was recorded as a harmless deferred minor in Task 12
precisely because a role pattern could not bind in MATCH — **this task is what
makes it reachable.**

Determine, with evidence, whether a bound relation-anchored role pattern needs
the traversal snapshot, and state the answer in your report either way:

- If it does, this `else` arm is a silent wrong-answer path and you must fix
  it so a `Roles` element is examined on the same terms as a `Path`.
- If it genuinely does not — e.g. because the whole form lowers to plain SQL
  with no traversal — then `false` is the right answer for the wrong reason.
  Say so explicitly and make the code say so too, so the next reader does not
  re-derive it.

"I assumed it was fine" is not an acceptable answer here.

- [ ] **Step 6: Classify a MATCH role pattern as a read**

Confirm `classify_statement` (`graph/frontend/src/binder.rs`) already returns
`StatementKind::Read` for a statement whose only pattern element is `Roles`
under `MATCH`. Task 13a extended this walk for the CREATE side; verify the
MATCH side rather than assuming, and if it is already correct, say so instead
of touching it.

- [ ] **Step 7: Run to verify they pass**

```bash
cargo test -p turso_graph_frontend --test nary_relations --test desugaring_golden
```

Then the full package: `cargo test -p turso_graph_cypher -p turso_graph_frontend`.

**Verify by sabotage.** For each of these, make the change, run, report what
the failure said, and revert:

- Break the subset projection so an unnamed role constrains the match — the
  second test in Step 2 must go red.
- Change one `RoleJoin`'s role to a different role — a Step 2 test must go
  red. If the tests pass with the roles permuted, they are resolving by
  position, which is the recurring defect class of this entire plan.

- [ ] **Step 8: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_cypher -p turso_graph_frontend
mise run corpus
mise run cypherbench-sample
```

This task changes production read paths and adds IR, so **both** the corpus
and cypherbench are required — do not skip them.

The corpus gate is **per suite**, never a total. `age-deep` 3042,
`cqlite-deep` 113, `grafeo-deep` 277, `sparrowdb-deep` 2164 each **exactly**
at baseline; `tck-deep` within **3329-3332** (flaky by ±2 on identical
commits). Do not write "corpus at 8,926" — state the per-suite numbers you
actually observed. If any non-`tck` suite moves off baseline, stop and report
BLOCKED with the suite and the delta; given ruling (B) exists specifically to
keep the arrow path's SQL unchanged, a corpus move here is evidence you
re-planned the arrow path.

`git add` with explicit paths (never `git add -A`), `git commit -S`, and
commit **code only** — nothing under `graph/test-results/`, which the
controller commits separately.
