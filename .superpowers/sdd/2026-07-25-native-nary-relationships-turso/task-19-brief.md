### Task 19: Create atomicity and relation-as-player

Every claim below was **measured against the tree at the commit you are
branching from**, not read off the plan. The plan's version of this task is
wrong in four separate ways, listed under "Plan defects". **Where the plan text
conflicts with this brief, this brief governs.**

**Files:**
- Modify: `graph/frontend/src/binder.rs` (the role-player target check, `:1831-1865`)
- Test: `graph/frontend/tests/nary_relations.rs`
- Test: `graph/frontend/tests/fixture.rs` (a new session fixture)

---

## Plan defects — read before you start

1. **`fail_after_nth_internal_statement` does not exist.** The plan calls it "a
   test-only injection hook on the session's internal executor." `rg` over
   `graph/` finds no failure-injection hook of any kind. Do **not** build one:
   see "Atomicity" below for the failure you already have.

2. **`self.savepoint_depth` does not exist.** The plan's Step 3 `debug_assert!`
   references a field that is not in this tree. Do not add one.

3. **`citation_session()` does not exist.** `CITATION_SCHEMA` **does** exist,
   at `graph/frontend/src/semantic.rs:2945`, but it is a const inside that
   crate's private test module and is **not importable from
   `graph/frontend/tests/`**. Read it for the shape and build your fixture in
   `graph/frontend/tests/fixture.rs`. Its comment at `:2941` already states the
   intent: "Relation-as-player: `Citation.cited` targets `Transcription`, itself
   a relation." Note `citations(id INTEGER PRIMARY KEY, cited_id INTEGER)` at
   `:2954`.

4. **The atomicity property already holds, so the plan's "Expected: FAIL" is
   wrong.** Measured: `mutation.rs:280` opens a `run` closure that calls
   `try_single_program_mutation` or `execute_bound`; every spill insert
   (`mutation.rs:1962-2030`, `:2215`, `:2315`) is inside that call tree; and
   `run()` is invoked at `:337` inside `BEGIN IMMEDIATE`/`COMMIT` or at `:352`
   inside `SAVEPOINT __turso_graph_mutation`/`RELEASE`. There is nothing to
   move. See "Atomicity" for what to do instead.

---

## Part A — Atomicity

The deliverable here is **a test and a proof, not a production change.** Do not
manufacture a change to justify a red-then-green cycle.

You do not need an injection hook, because a natural mid-create failure already
exists: `run()` calls `constraints.validate_state(connection)` **after**
`execute_bound` returns (`mutation.rs:288`). A create that inserts a relation
row and its spill rows and then violates a semantic constraint fails with every
insert already executed and still inside the transaction. That is exactly the
"partway through" condition the test wants.

Find a constraint in `semantic_constraints.rs` that `validate_state` enforces
and that an otherwise-valid n-ary create can violate. If you cannot find one
that fits, a second option is a create whose **last** role argument fails its
target-type check after earlier roles' spill rows are staged — verify which
phase that check runs in before relying on it. Report which mechanism you used
and why.

Write:

```rust
#[test]
fn a_failure_partway_through_an_n_ary_create_leaves_nothing_behind() {
    // The integrity property reified modeling cannot provide: reification
    // needs one statement per role, so a failure between them leaves a
    // partially stated assertion that reads as complete. Here the relation
    // row and its spill rows share one transaction, so a failure after the
    // spill inserts have executed still leaves BOTH tables empty.
}
```

Assert **both** counts are zero: the relation table AND the spill table. A test
that only checks the relation table would pass even if spill rows leaked.

**Then prove the test can fail.** In `mutation.rs`, move a spill insert outside
the transaction window — commit it on a separate connection, or execute it
before the `BEGIN IMMEDIATE`/`SAVEPOINT` line — run the test, and quote the
verbatim failure showing a non-zero spill count. Revert. **If the test stays
green under that sabotage, it does not test atomicity and you must rewrite
it.** This step is the whole value of Part A; do not skip it or replace it with
an argument that the code looks correct.

## Part B — Relation-as-player

This is where the production change is. Measured at
`graph/frontend/src/binder.rs:1831-1865` (the role-player target check inside
the CREATE role-pattern path):

```rust
let allowed = role
    .targets
    .iter()
    .filter_map(|target| match target {
        ir::RoleTarget::Node(label) => Some(*label),
        ir::RoleTarget::Relation(_) => None,     // <-- discarded
    })
    .collect::<Vec<_>>();
if !allowed.is_empty() {
    // ... requires every name on the binding to resolve via
    //     self.catalog.label(self.graph, name)
}
```

`ir::RoleTarget::Relation` **exists** (`graph/ir/src/role.rs:12`) and the
semantic layer stores it (`semantic.rs:2474`). The binder discards it. Two
holes follow, and **both are yours**:

- **A role targeting only relations is unchecked.** `allowed` comes out empty,
  the `if` is skipped, and the role accepts *anything* — a node, a relation of
  the wrong type, whatever. `Citation.cited` is exactly this shape.
- **A role with mixed Node and Relation targets rejects every relation.**
  `allowed` is non-empty, so the check runs, and a relation binding's names are
  relationship **type** names — `self.catalog.label(graph, "Transcription")`
  returns `None`, so `all_allowed` is false and `RoleTargetTypeViolation`
  fires on a player the schema explicitly permits.

The fix is to stop discarding relation targets: check a binding against the
target kind that matches it. The bound entity already carries what you need —
`kind: CatalogEntity` at `binder.rs:610`, with `CatalogEntity::{Node,
Relationship}` at `:17-20`. Resolve a `Relationship`-kind binding's names
through `self.catalog.relationship_type(self.graph, name)` and compare against
the `RoleTarget::Relation` ids, the mirror of what the existing code does with
`label()` and `RoleTarget::Node`.

Note the same discard appears at `binder.rs:1666` in the **arrow**-form
start/end check. That path is a known deferred item (it hard-codes `"start"`
and `"end"`); leave it alone unless one of your tests proves it reachable for a
relation player, and say which you concluded.

**Do not special-case the writer.** A relation identity is an identity like any
other; if the writer needs a branch to store one, report that as a finding
rather than adding one.

### Tests

Append to `graph/frontend/tests/nary_relations.rs`. Copy the idiom the existing
Task 13b/14b/16 tests there use — fixtures return `(Arc<Database>,
GraphConnection)`, reads go through `session.query(sql, &Parameters::new())`
returning `Result<Vec<Vec<Value>>, _>`, mutations through `session.execute`.
**The plan's `session.run` / `session.sql` / `session.expect_error` /
one-argument `session.query` do not exist** — do not transcribe them. Using
`execute` for a read yields "Cypher mutation binding failed", which is a
misleading failure, not a real refusal.

Four behaviors:

```rust
#[test]
fn a_relation_may_be_a_player_of_another_relation() {
    // A relation identity is an identity: the role's target list carries
    // RoleTarget::Relation, so the transcription itself fills `cited`.
}

#[test]
fn a_role_that_accepts_only_relations_refuses_a_node_player() {
    // The hole this task closes: today `allowed` comes out empty for a
    // relation-only role and the check is skipped entirely, so a node is
    // accepted into `cited`. It must be refused.
}

#[test]
fn a_role_that_does_not_accept_relations_refuses_a_relation_player() {
    // `source` targets Text only, so the transcription is not a legal player.
    // Assert the actual RoleTargetTypeViolation text -- the plan's
    // `error.contains("source")` would also pass on a syntax error that
    // happens to echo the role name back.
}

#[test]
fn a_role_with_both_node_and_relation_targets_accepts_either() {
    // The second hole: with a mixed target list the current code rejects
    // every relation player, because a relationship type name never resolves
    // as a label.
}
```

Assert on the real error text, not on a substring that a different error would
also contain. Quote the messages you observe in your report.

### Sabotages

For each: make the change, run `cargo test -p turso_graph_frontend`, quote
verbatim what went red, revert.

- Move a spill insert outside the transaction window. Part A's test must go red
  with a non-zero spill count.
- Restore `RoleTarget::Relation(_) => None` in the check you fixed. The
  relation-only refusal test and the mixed-target test must both go red.
- Make the relation-target comparison accept any relationship type rather than
  the ones in the target list. A test must go red.
- Permute the role **names** in your new fixture's schema (swap which name
  carries which target list) without changing their order. A test must go red
  — if everything still passes, roles are resolving by position, which is the
  recurring defect class of this entire plan.

**Constraints:** no arity branch, no `is_binary`, no hard-coded `"start"` /
`"end"` in general machinery; roles resolve by `RoleId` or by name, never by
position; a `Many` role is identified by `spill_table.is_some()`. Never build
with `--release`.

### Gate and commit

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_cypher -p turso_graph_frontend
mise run corpus
mise run cypherbench-sample
```

Run the clippy command **exactly as written**. Four implementers on this plan
have substituted a narrower `-p <package>` form, hit two pre-existing `core/`
unused-import warnings, and reported the gate as broken. The literal workspace
form exits 0. If you believe it fails, paste the literal command and its exit
code.

`mise run corpus` is known to exit 1 with "task failed" even when every suite
is at baseline — read the per-suite numbers, do not trust the exit code.

The corpus gate is **per suite, never a total**: `age-deep` 3042, `cqlite-deep`
113, `grafeo-deep` 277, `sparrowdb-deep` 2164 each **exactly** at baseline;
`tck-deep` within **3329-3332** (flaky ±2 on identical commits). Do **not**
write "corpus at 8,926" — the plan's commit message says that and it is not a
real number; state the per-suite figures you observed. If any non-`tck` suite
moves off baseline, stop and report BLOCKED with the suite and the delta.

`git add` with **explicit paths** — the plan says `git add -A`; do not. Use
`git commit -S`, and commit **code only**, nothing under `graph/test-results/`,
which the controller commits separately.
