### Task 16: Role-edge read sugar

`(x:KNOWS)-[:start]->(s)` reads the `start` role of a relation. It is sugar
over Task 13b's `MATCH [x:KNOWS](start: s)`.

Every claim below was **measured against the tree at the commit you are
branching from**, not read off the plan. Where the original plan text
conflicts with the tree, **this brief governs** — the conflicts are named
explicitly so you do not "fix" the brief back toward the plan.

**Files:**
- Modify: `graph/frontend/src/binder.rs` (anchor resolution + expand path)
- Modify: `graph/frontend/tests/fixture.rs` (new fixture + bind helper)
- Test: `graph/frontend/tests/nary_relations.rs`

---

## Measured facts — read these before designing anything

I ran these against `fixture::witnessed_session()` (one node source `Person`
over `people`; one relationship source `KNOWS` over `relationships` with
roles `start`→`src` (One), `end`→`dst` (One), `witness` (Many, spill)):

| Query (via `session.query`) | Actual result |
|---|---|
| `MATCH (x:KNOWS)-[:start]->(s) RETURN s.id` | ``Err("unknown label `KNOWS` at byte 9..14")`` |
| `MATCH (x:KNOWS)-[:witness]->(w) RETURN w.id` | ``Err("unknown label `KNOWS` at byte 9..14")`` |
| `MATCH [x:KNOWS](start: s) RETURN s.id` | `Ok([[Integer(1)]])` — Task 13b works |
| `MATCH (p:Person)-[:start]->(s) RETURN s.id` | ``Err("unknown relationship type `start` at byte 19..24")`` |

Four consequences, all binding:

1. **The plan is wrong about where this task's work is.** Its Step 2 predicts
   the failure is "`scribe` resolves as an unknown relationship type", and its
   Step 3 puts the whole fix "in the expand binding path". The measured
   failure is `UnknownLabel` on the **anchor node**, raised by
   `resolve_labels` (called from `bind_start_node`, `binder.rs:3669`) — the
   expand path is never reached. The anchor `(x:KNOWS)` must first bind as a
   **relation**; only then does the role-vs-type question in the expand path
   arise. Both halves are yours.
2. **`bind_start_node` unconditionally creates a node binding**
   (`CatalogEntity::Node`, `ir::ValueType::Node`). There is no existing path
   by which a bare `(x:Label)` becomes a relation binding.
3. **The plan's fourth test already passes today** (row 4 above: the error
   contains "relationship type"). Keep it as a regression guard; it needs no
   new production code. Say so in your report rather than inventing work for
   it.
4. **`ternary_session` cannot host these tests.** Its doc comment explains
   why: with three node sources and no semantic schema, node properties do
   not resolve — ``MATCH [x:Transcription](scribe: s) RETURN s.id`` fails with
   ``unknown property `id` ``, feature or no feature. The plan's Task 16 tests
   all use `ternary_session` + `s.id` and would fail for that unrelated
   reason. **Use `witnessed_session`.** Its role names are `start` / `end` /
   `witness`, not `scribe`.

Two more measured facts you will need:

- `session.query(sql, &Parameters::new()) -> Result<Vec<Vec<Value>>, _>` is
  the read path; `session.execute` is the mutation path and reports
  "Cypher mutation binding failed" for reads. Use `query` for reads.
- The catalog methods the plan calls **do not exist under those names**:
  - plan's `self.catalog.relationship_type_id(name)` → real one is
    `relationship_type(&self, graph: ir::GraphId, name: &str) -> Option<ir::RelationshipTypeId>`
    (`binder.rs:101`).
  - plan's `self.catalog.relationship_role(relation_type, name)` → real one
    takes a leading graph:
    `relationship_role(&self, graph: ir::GraphId, ty: ir::RelationshipTypeId, name: &str) -> Option<SemanticRole>`
    (`binder.rs:178`), and it matches with `eq_ignore_ascii_case`.
  - plan's `self.relation_type_of_binding(...)` and
    `self.relationship_type_name(...)` **do not exist**. The existing way to
    ask what a binding is:
    `self.entities.get(&id).map(|e| e.kind) == Some(CatalogEntity::Relationship)`,
    and `self.entity_type_names(binding, span) -> Result<Vec<String>, BindError>`
    (`binder.rs:2549`) for its type names. Model your code on the existing
    precedent at `binder.rs:2304-2325` (the role-update binder), which does
    exactly this: resolve the variable → check kind → `entity_type_names` →
    `relationship_type(self.graph, type_name)`.

`AmbiguousRoleName` genuinely does not exist yet — you add it, as the plan
says.

---

- [ ] **Step 1: Decide and state the two resolution rules**

Before writing code, write the two rules into your report. They are the whole
task; everything else is mechanics.

**Rule A — when is a bare `(x:Name)` a relation anchor?** The plan gives you
the governing principle, in its own words for the other rule: *adding a role
must never change what an existing node query means.* Apply it here too. The
conservative reading, which you should adopt unless you can show it wrong:
`Name` resolves as a node label whenever `catalog.label(graph, name)` returns
`Some` — unchanged, first, always. Only when it is **not** a node label may it
resolve as a relationship type and produce a relation anchor. If a name is
both a node label and a relationship type, the node reading wins silently and
existing queries keep their meaning.

If you conclude a different rule is right, argue it in the report — but do not
adopt a rule under which adding a relationship type can change what an
existing node query returns.

**Rule B — role or relationship type after the `:`?** As the plan has it: the
name after `-[:` is a role **only** when the source binding is a relation.
From a node it stays a relationship type, unchanged. If the name is both a
role of that relation's type *and* a relationship type, refuse as ambiguous
rather than guess.

Ambiguity must use the **same case rule** as resolution
(`eq_ignore_ascii_case`), or `Witness` could resolve as a role while dodging
the ambiguity check.

- [ ] **Step 2: Build the fixtures you need**

In `graph/frontend/tests/fixture.rs`:

- **An ambiguous-name session.** I verified this registers cleanly: a graph
  with node source `Person`/`people`, relationship source
  `KNOWS`/`relationships` whose roles include one named `witness`, **and a
  second relationship source literally named `witness`** over its own table.
  `register_graph` accepts it and `MATCH [x:KNOWS](witness: w)` binds against
  it today. Follow `witnessed_session` (`fixture.rs:237`) as the template.
- **A bind helper returning `ir::Plan` against a real catalog.** The existing
  `bind_fixture` (`fixture.rs:402`) is **useless for this task**: its stub
  `Catalog::label` returns `Some(LabelId(1))` for *every* name and
  `relationship_type` returns `Some(...)` for every name
  (`fixture.rs:329-335`), so under it every name is simultaneously a label, a
  type, and therefore ambiguous. Add a helper that binds against a real
  `SchemaCatalog` instead — `SchemaCatalog` implements `GraphCatalogSnapshot`
  (`schema_catalog.rs:417`), so `bind(&parse(q)?, registered.id, &catalog,
  &ParameterTypes::new())?.plan` works. `ir::Plan` derives `PartialEq`
  (`graph/ir/src/plan.rs:7`).

- [ ] **Step 3: Write the failing tests**

Append to `graph/frontend/tests/nary_relations.rs`, matching the idiom the
Task 13b tests there already use (`fixture::witnessed_session()`, seed with
`session.execute`, read with `session.query`, assert `Vec<Vec<Value>>`).

Four behaviors:

```rust
#[test]
fn an_arrow_from_a_relation_reads_that_relations_role() {
    // Sugar for the role pattern. Without it, reading one participant of an
    // n-ary relation needs the full role form even when one role is wanted.
    // Assert actual rows, not just Ok — an empty result is not a read.
}

#[test]
fn the_role_arrow_and_the_role_pattern_bind_to_the_same_plan() {
    // Both forms are relation-anchored, so unlike the arrow-vs-role goldens
    // in desugaring_golden.rs there is no reason for their plans to differ.
    // See "On plan identity" below before weakening this.
}

#[test]
fn a_name_that_is_both_a_role_and_a_relationship_type_is_ambiguous() {
    // Guessing would make a query mean one thing today and another after an
    // unrelated schema addition.
}

#[test]
fn the_role_arrow_is_only_available_from_a_relation_binding() {
    // Regression guard: this ALREADY passes (measured). From a node the name
    // must still resolve as a relationship type, or adding a role would
    // change what existing node queries mean.
}
```

For the first test, seed **two** relations that differ in the role you read,
and assert you get both rows. A one-row fixture cannot distinguish "reads the
role" from "returns the anchor's first player".

**On plan identity.** The desugaring goldens were just rewritten (commit
`3dab1431d`) to assert row-equivalence rather than plan identity, under a
human ruling that emitted behaviour — not plan-node identity — is the
contract. **That ruling does not transfer here**, and you should not cite it
to weaken this test. It applied to arrow-vs-role for a *node-anchored* arrow,
where the two forms legitimately plan differently (node-anchored vs
relation-anchored) and where the role form has no label slot. Here both sides
are relation-anchored and both are label-less, so identical plans are
achievable and are the stronger assertion. Assert plan equality. If you find
it genuinely unachievable, **report it with both plans quoted** — that is a
controller decision, not yours. Do not silently relax it to rows or to SQL
text.

- [ ] **Step 4: Run to verify they fail**

```bash
cargo test -p turso_graph_frontend --test nary_relations
```

Expected failure for the first two: ``unknown label `KNOWS` `` — **not** the
"unknown relationship type" the plan predicts. Quote the real messages in your
report. If you see the plan's predicted message instead, stop and report it:
something differs from what I measured.

- [ ] **Step 5: Implement**

Rule A in the anchor path (`bind_start_node`); also check whether the mid-path
node binder around `binder.rs:3352` needs the same treatment — a relation can
only be a path *anchor*, so if you conclude it does not, say why. Rule B in
the expand path where `relationship_type` is resolved for a MATCH step
(`binder.rs:3363-3374`).

For a relation anchor, reuse what Task 13b already built. `MATCH
[x:KNOWS](start: s)` binds through `bind_match_role_pattern`
(`binder.rs:2770`); the sugar is one named role against that same machinery.
**Do not re-implement role resolution, relation-source lookup, or player
binding** — call into the existing code, or the two forms will drift and the
Step 3 plan-equality test will tell you so.

Add the error variant:

```rust
#[error("`{name}` is both a role of `{relationship_type}` and a relationship type; write the role form `[x:{relationship_type}]({name}: target)` or qualify the type")]
AmbiguousRoleName {
    name: String,
    relationship_type: String,
    span_start: usize,
    span_end: usize,
},
```

**Constraints that bind every line you write:**

- **No arity branch.** No `if roles.len() == 2`, no `is_binary`, no
  hard-coded `"start"` / `"end"` in general machinery. Positional role
  resolution is the recurring defect class of this entire plan — resolve by
  `RoleId` and by name, never by position.
- A `Many` role is identified by `RelationshipRoleLayout.spill_table.is_some()`,
  never by name or position. `witness` in `witnessed_session` is `Many`.
  Hopping *through* a `Many` role is Task 14b's job. If the arrow sugar over a
  `Many` role falls out for free, test it; if you conclude it belongs to 14b,
  **say so explicitly and justify it** rather than leaving it silently
  unhandled or silently wrong.
- Every interpolated SQL identifier goes through `quoted_identifier`
  (`lowering.rs:305`).

- [ ] **Step 6: Run to verify they pass, then verify by sabotage**

```bash
cargo test -p turso_graph_cypher -p turso_graph_frontend
```

For each sabotage: make the change, run, report what the failure said, revert.

- Make the anchor resolve as a relation **before** checking `catalog.label` —
  i.e. break Rule A's ordering. If no test goes red, nothing guards the
  backwards-compatibility property that is the entire justification for Rule
  A; add a test that does.
- Change the resolved role to a different role of the same relation (e.g.
  `start` → `end`). A Step 3 test must go red. If the tests pass with roles
  permuted, they are resolving by position.
- Remove the ambiguity check so the role silently wins. The third test must go
  red.
- Delete the relation-binding guard in Rule B so a **node** source also tries
  role resolution. The fourth test must go red.

- [ ] **Step 7: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_cypher -p turso_graph_frontend
mise run corpus
mise run cypherbench-sample
```

This changes production read binding, so **both** corpus and cypherbench are
required — do not skip them.

The corpus gate is **per suite, never a total**: `age-deep` 3042, `cqlite-deep`
113, `grafeo-deep` 277, `sparrowdb-deep` 2164 each **exactly** at baseline;
`tck-deep` within **3329-3332** (flaky ±2 on identical commits). Do **not**
write "corpus at 8,926" — the plan's own commit message says that and it is
not a real number; state the per-suite figures you actually observed. If any
non-`tck` suite moves off baseline, stop and report BLOCKED with the suite and
the delta: Rule A is designed specifically so no existing node query changes
meaning, so a corpus move is direct evidence Rule A is wrong.

`git add` with **explicit paths** (never `git add -A` — the plan's Step 5 says
`git add -A`; ignore it), `git commit -S`, and commit **code only** — nothing
under `graph/test-results/`, which the controller commits separately.
