### Task 18b: MERGE over role patterns

Split from Task 18. `MERGE [x:Transcription](scribe: p, text: t, folio: f)`
must work end to end. Every reference below was verified against the tree at
`ae795a64c`.

**Files:**
- Modify: `graph/cypher/src/cypher.pest` (merge clause)
- Modify: `graph/cypher/src/parser.rs` (merge clause construction)
- Modify: `graph/frontend/src/binder.rs` (merge routing for role patterns)
- Modify: `graph/frontend/src/mutation.rs` (merge key for `Many` roles)
- Test: `graph/frontend/tests/nary_relations.rs`

**What is already done, so you do not redo it:** every single-valued role is
*already* part of the merge key. `insert_relationship`
(`graph/frontend/src/mutation.rs:1933`) collects each `One` role into `fixed`
at `:1962-1977` and passes it to `insert_entity` at `:1991`. Tasks 10 and 11
generalized that away from a start/end pair. The `merge_predicates` computed
at `:1946` are relationship-**type** predicates only
(`relationship_type_predicates`, `:1868`). Do not rewrite any of this.

---

- [ ] **Step 1: Make the grammar accept a role pattern after MERGE**

Today:

```
merge_clause = { MERGE ~ path_pattern ~ merge_action* }   // cypher.pest:65
```

It takes `path_pattern` **directly**, bypassing the alternation that CREATE
goes through:

```
create_clause  = { CREATE ~ pattern }                      // :64
pattern_element = { role_pattern | path_pattern }          // :96
role_pattern   = { relationship_body ~ role_arguments }    // :100
```

That asymmetry is the entire reason `MERGE [x:T](role: p)` does not parse.
Change `merge_clause` so a role pattern is accepted in the same position,
keeping `merge_action*` intact. Note the comment at `:91`: `pattern_element`
is an ordered choice with `role_pattern` first because `[` never begins a
`path_pattern` — preserve that ordering property in whatever you write.

MERGE takes a **single** pattern, not a comma-separated `pattern` list. Do
not silently widen MERGE to accept multiple patterns; if you believe it
should, raise it rather than doing it.

- [ ] **Step 2: Write the failing tests**

Use the idiom the existing tests in `graph/frontend/tests/nary_relations.rs`
actually use — `let (database, session) = fixture::ternary_session();`,
seeding via `fixture::second_connection(&database)` +
`load_registered_graph(&seed, "scriptorium")` + the local `seed_node(…)`,
running with `session.execute(sql, &Parameters::new())`, asserting errors
with `.expect_err("why")`, and asserting stored rows through a second
connection with `.prepare(…).run_collect_rows()`.

```rust
#[test]
fn merge_matches_on_the_full_set_of_bound_roles() {
    // Matching on a subset would make a second MERGE with a different folio
    // silently update the first transcription instead of creating a second
    // one, collapsing two distinct assertions into one.
}
```

Assert, in one test or several:
- the same MERGE run twice leaves exactly **one** row in `transcriptions`
- a MERGE differing only in `folio` leaves **two** rows — a different folio
  is a different assertion
- a MERGE that matches an existing relation does **not** duplicate that
  relation's spill rows (see Step 4)

- [ ] **Step 3: Route a role pattern through the merge binder**

`bind_create_role_pattern` (`graph/frontend/src/binder.rs:1820`) returns
`Result<ir::CreateRelation, BindError>` and has no merge form. The merge
path already produces `ir::Mutation::MergeRelation` at `binder.rs:1712`.
Connect the two so a role pattern under MERGE yields `MergeRelation` rather
than `CreateRelation`.

Prefer reusing `bind_create_role_pattern`'s body over copying it — the role
resolution, duplicate-argument refusal, required-role check, and target-type
check must stay **one** implementation. Two copies would drift, and this
plan already carries two implementations of one predicate
(`single_valued_roles()` / `structural_columns()`) as a known wart. Do not
add a third.

- [ ] **Step 4: Put `Many` roles into the merge key**

This is the one genuine gap. A `Many` role has no column on the relation
table, so it cannot appear in `fixed`. Match it on membership instead:

```sql
EXISTS (SELECT 1 FROM <spill_table> WHERE relation_id = <table>.<identity> AND node_id = <player>)
```

`RelationshipRoleLayout.spill_table` (`graph/frontend/src/lowering.rs`) is
`Some(_)` exactly for `Many` roles and `None` for `One` roles — branch on
that, not on a name or a position. Use `quoted_identifier` (note the `d` —
`quote_identifier` does not exist in this file) for every identifier you
interpolate.

Derive the predicate from the role's cardinality alone. No `if
roles.len() == 2`, no `is_binary`, no hard-coded `"start"`/`"end"`.

- [ ] **Step 5: Cover the `if created` spill guard**

`mutation.rs:1994` guards the spill writes:

```rust
    // A relation matched by MERGE already exists with whatever spill rows its
    // original CREATE wrote; only a freshly created relation needs its
    // many-valued role players written.
    if created {
```

This guard has been in the tree since Task 14a and is **untested** — it was
recorded then as a deferred minor, and it can only be exercised through
MERGE over a role pattern, which is why it lands here. Add a test that
fails if the guard is removed: MERGE a relation with a `Many` role twice and
assert the spill table holds the players **once**, not twice.

Verify by sabotage: delete the `if created` condition, confirm your test
goes red, restore it, and report what the failure said. A test that passes
either way is worth nothing here.

- [ ] **Step 6: Gate and commit**

Run before committing:

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_cypher -p turso_graph_frontend
mise run corpus
mise run cypherbench-sample
```

Corpus gate is **per suite**: `age-deep` 3042, `cqlite-deep` 113,
`grafeo-deep` 277, `sparrowdb-deep` 2164 each **exactly** at baseline;
`tck-deep` within **3329-3332** (flaky by ±2 on identical commits). There is
no single total — state the per-suite numbers you observed. A grammar change
is a broad blast radius, so if any non-`tck` suite moves off baseline, stop
and report BLOCKED with the suite and delta.

`git add` with explicit paths (never `git add -A`), `git commit -S`, and
commit **code only** — nothing under `graph/test-results/`.
