### Task 14b: hop through a `Many`-valued role

`MATCH [x:KNOWS](witness: w)` must return **one row per player** of the
`Many` role. Today all three read paths refuse it outright.

Task 14 was split during execution: 14a landed the write and delete side of
`Many` roles (spill rows on CREATE/MERGE/SET, removal on both delete paths).
**14b is the read side, and it is lowering-only** — no IR change is needed
(see Step 1).

Every claim below was **measured against the tree at the commit you are
branching from**, not read off the plan. Where the plan text conflicts with
the tree, **this brief governs**.

**Files:**
- Modify: `graph/frontend/src/lowering.rs` (`lower_role_join`)
- Modify: `graph/frontend/src/binder.rs` (remove two refusals)
- Test: `graph/frontend/tests/nary_relations.rs`

---

## Measured facts

Against `fixture::witnessed_session()` — one node source `Person`/`people`;
relationship source `KNOWS`/`relationships` with roles `start`→`src` (One),
`end`→`dst` (One), `witness` (**Many**, spill). Seeded with four people and
`CREATE [x:KNOWS](start: a, end: b, witness: w1, witness: w2)`, which succeeds
(14a):

| Query (via `session.query`) | Actual result |
|---|---|
| `MATCH [x:KNOWS](witness: w) RETURN w.id` | ``Err("a Many-cardinality role in a MATCH role pattern is not supported in the initial graph slice at byte 16..26")`` |
| `MATCH [x:KNOWS](start: s, witness: w) RETURN s.id, w.id` | same refusal, byte 26..36 |
| `MATCH (x:KNOWS)-[:witness]->(w) RETURN w.id` | ``Err("a role arrow over a Many-cardinality role is not supported in the initial graph slice at byte 15..28")`` |
| `MATCH [x:KNOWS](start: s) RETURN s.id` | `Ok([[Integer(1)]])` — the `One` path works |

**Three refusals exist and all three are yours to deal with:**

1. `binder.rs:2907` — Task 13b's role-pattern MATCH: `"a Many-cardinality role
   in a MATCH role pattern"`. **Remove.**
2. `binder.rs:3829` — Task 16's arrow sugar (`bind_role_read_step`): `"a role
   arrow over a Many-cardinality role"`. **Remove.** Note Task 16 shipped a
   test pinning this refusal's exact wording
   (`a_role_arrow_over_a_many_cardinality_role_is_refused`,
   `nary_relations.rs:1167`). Replace that test with one asserting the hop now
   works — do not delete it and leave nothing behind.
3. `lowering.rs:1576-1585` — a defense-in-depth guard inside `lower_role_join`,
   whose own comment names this task. **Replace with the real implementation.**

**Physical layout, verified in `catalog.rs:941` (`install_spill_table`):**

```sql
CREATE TABLE "<relation_table>__<role>"(relation_id INTEGER NOT NULL, node_id INTEGER NOT NULL)
```

with indexes on both `(relation_id, node_id)` and `(node_id, relation_id)`, so
a probe from either side is an index probe. The name is available at lowering
time as `RelationshipRoleLayout.spill_table: Option<String>` — **a `Many` role
is identified by `spill_table.is_some()`, never by name, position, or arity.**

---

- [ ] **Step 1: Confirm this is lowering-only, then say so in your report**

`ir::RoleJoin` (`graph/ir/src/plan.rs:145`) carries `input`, `relationship`,
`relationship_source`, `role`, `player`. The spill table name comes from the
catalog layout at lowering time, so a `Many` role needs no new IR field.
Verify that yourself before you start; if you find it false, report BLOCKED
with the specific thing `RoleJoin` cannot express rather than growing the IR
speculatively.

Also determine, with evidence, whether `RoleExpand` (the **node**-anchored
arrow `(a:Person)-[r:KNOWS]->(b)`, lowered by `lower_role_expand`) can ever
carry a `Many` role. The Task 14 split recorded that it cannot — the binder
only fills its role pair from the two endpoint roles — but that was recorded
before Tasks 16 and 17 landed. If it still cannot, say so and leave
`lower_role_expand` alone. If it now can, it needs the same treatment and a
test. **Do not guess**; the plan's Step 5 targets `lower_role_expand`
specifically, and if the split's finding still holds then the plan is aiming
at the wrong function.

- [ ] **Step 2: Write the failing tests**

Append to `graph/frontend/tests/nary_relations.rs`, using the idiom the Task
13b/16 tests there already use (`fixture::witnessed_session()`, seed with
`session.execute`, read with `session.query`, assert `Vec<Vec<Value>>`).

Five behaviors. The first two are the load-bearing pair:

```rust
#[test]
fn a_hop_through_a_many_valued_role_returns_every_player() {
    // A relation with two witnesses must yield TWO rows. Returning one is
    // the failure mode the plan's own Step 5 code would ship (see Step 3):
    // it silently truncates a player set to whichever row the subquery
    // happens to pick, which is data loss that looks like a working query.
}

#[test]
fn not_naming_a_many_valued_role_does_not_multiply_rows() {
    // The subset projection from Task 13b must survive. With two witnesses
    // seeded, `MATCH [x:KNOWS](start: s)` must still return ONE row, not two.
    // A spill join that leaks into queries that never named the role turns
    // every unrelated query's row count into a function of witness count.
}

#[test]
fn a_relation_with_no_players_in_a_many_role_is_absent_from_that_hop_but_not_others() {
    // Seed a second relation with NO witness. `(witness: w)` must not return
    // it; `(start: s)` must. Distinguishes an inner join from an outer one.
}

#[test]
fn a_many_role_hops_from_the_arrow_sugar_too() {
    // Replaces Task 16's refusal test at nary_relations.rs:1167.
    // `MATCH (x:KNOWS)-[:witness]->(w)` and the role-pattern form must agree.
}

#[test]
fn a_bound_player_constrains_a_many_valued_role() {
    // The `RolePlayer::Bound` case — see Step 3, it is a SEPARATE code path
    // the plan does not mention at all.
    // `MATCH (w:Person {id: 3}), [x:KNOWS](witness: w)` must match the
    // relation that has w among its witnesses and not the one that does not.
}
```

Seed at least **two relations** in the fixtures for tests 1-3, differing in
their witness sets. A single-relation fixture cannot distinguish "returns
every player" from "returns every spill row in the table".

- [ ] **Step 3: Run to verify they fail, then implement**

```bash
cargo test -p turso_graph_frontend --test nary_relations
```

Expect the refusal messages quoted above. Quote the real ones in your report.

**The plan's Step 5 code is wrong and you must not transcribe it.** It offers
two snippets that contradict each other:

```rust
// Snippet A -- a scalar subquery:
Some(table) => format!(
    "(SELECT {spill_alias}.node_id FROM {} {spill_alias} \
     WHERE {spill_alias}.relation_id = {relationship_alias}.{})", ...)
```

A scalar subquery yields **one** value. Under it, a relation with two
witnesses returns one row and the second player is silently lost — the exact
bug test 1 exists to catch. The plan then says to "emit a `JOIN` rather than a
scalar subquery when the role is on the produced side", i.e. it contradicts
its own first snippet. **Use the JOIN.** Additionally, `lower_role_expand` has
no `joins` vector and no `spill_alias` for snippet B to push onto — that
structure does not exist in this tree.

Implement in `lower_role_join` (`lowering.rs:1561`), where the `One` case
already lives. Two distinct player cases, both of which you must handle:

- **`RolePlayer::Fresh`** — join relation → spill → the player's node table,
  so n players produce n rows. The `One` arm's use of `materialize_properties`
  and `BindingLayout` is your template; the difference is one extra join, not
  a different shape.
- **`RolePlayer::Bound`** — the `One` arm folds to an identity equality
  `q.<role_column> = q.<binding>`. **A `Many` role has no role column**, so
  that expression cannot be written and this arm needs a membership test
  instead. `mutation.rs:1888` already builds exactly such an `EXISTS (SELECT 1
  FROM <spill> WHERE ...)` predicate for MERGE's merge key over a `Many` role
  — reuse that shape. The plan does not mention this case at all; shipping the
  `Fresh` case alone would leave `Bound` either broken or still refusing.

Every interpolated identifier goes through `quote_identifier` /
`quoted_identifier` as the surrounding code does.

**Constraints:** no arity branch, no `is_binary`, no hard-coded `"start"` /
`"end"` in general machinery. `Many` is identified by `spill_table.is_some()`.
Positional (rather than by-`RoleId`) role resolution is the recurring defect
class of this entire plan — assume a reviewer will permute role order and role
names to make your tests pass when they should fail.

- [ ] **Step 4: Run to verify they pass, then verify by sabotage**

```bash
cargo test -p turso_graph_cypher -p turso_graph_frontend
```

For each: make the change, run, report verbatim what went red, revert.

- Replace the JOIN with the plan's scalar subquery (snippet A). Test 1 must go
  red with a row-count mismatch. If it does not, test 1 does not test what it
  claims.
- Make the spill join unconditional, so it also runs for roles nobody named.
  Test 2 must go red.
- Turn the join into a LEFT JOIN. Test 3 must go red.
- Change the resolved role to a different role of the same relation. A test
  must go red — if they all still pass, roles are resolving by position.
- Break the `Bound` membership predicate so it always holds. Test 5 must go
  red.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_cypher -p turso_graph_frontend
mise run corpus
mise run cypherbench-sample
```

Run the clippy command **exactly as written**. Three implementers on this plan
have substituted a narrower `-p <package>` invocation, watched it fail on two
pre-existing `core/` unused-import warnings, and reported the gate as broken.
The literal workspace form exits 0; the narrow form unifies `core`'s features
differently. If you believe the gate fails, paste the literal command and its
exit code.

This changes production read lowering, so **both** corpus and cypherbench are
required.

The corpus gate is **per suite, never a total**: `age-deep` 3042, `cqlite-deep`
113, `grafeo-deep` 277, `sparrowdb-deep` 2164 each **exactly** at baseline;
`tck-deep` within **3329-3332** (flaky ±2 on identical commits). Do **not**
write "corpus at 8,926" — the plan's commit messages say that and it is not a
real number; state the per-suite figures you observed. If any non-`tck` suite
moves off baseline, stop and report BLOCKED with the suite and the delta: this
task should not touch any query that does not name a `Many` role, so a corpus
move is direct evidence the spill join is leaking into unrelated reads —
exactly what test 2 guards.

`git add` with **explicit paths** (never `git add -A`), `git commit -S`, and
commit **code only** — nothing under `graph/test-results/`, which the
controller commits separately.
