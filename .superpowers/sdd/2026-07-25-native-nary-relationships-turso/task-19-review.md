# Task 19 review: atomicity proof and relation-as-player

## Verdict: spec compliance — PASS

Both parts of `task-19-brief.md` are satisfied, and the four plan defects the
brief called out (missing injection hook, missing `savepoint_depth`, missing
`citation_session()`, and the "Expected: FAIL" atomicity claim) are correctly
not reintroduced. Part A required a natural mid-create failure proven by
sabotage, not a production change; Part B required closing the relation-as-
player hole in `bind_role_player` (`binder.rs:1831-1865`) without an
arity/position branch. Both hold under independent re-verification (below).

## Verdict: task quality — PASS

The report's mechanism choice, error-text claims, and all four sabotages
were reproduced independently with matching results (not just matching
prose). Tests assert on typed error variants and field values, not message
substrings. Fixture doc comments correctly explain the physical-layer
placeholder wart (`node_source` pointed at `text_src` for relation-targeting
roles) rather than hiding it.

## Independent verification performed

- Baseline: `cargo test -p turso_graph_frontend --test nary_relations` → 44
  passed. `cargo test -p turso_graph_frontend` → 334 passed, 1 ignored.
  `cargo fmt --check -p turso_graph_frontend` → exit 0.
- **Sabotage 1** (move a spill insert outside the transaction window): added
  an unconditional raw insert into `citations__witnesses` at the top of
  `execute_cypher_mutation` (`mutation.rs:252`), before `BEGIN
  IMMEDIATE`/`SAVEPOINT`. Reran the atomicity test; it went red:
  `assertion left == right failed ... left: [[Numeric(Integer(3))]] right:
  [[Numeric(Integer(0))]]` (count 3, not 1, because the sabotage insert also
  fires on the two earlier successful statements in the test — consistent
  with the report's explanation). Reverted; `git diff --stat` clean.
- **Sabotage 2** (restore `RoleTarget::Relation(_) => None`, collapsing back
  to the single label-only `allowed` list): both
  `a_role_that_accepts_only_relations_refuses_a_node_player` and
  `a_role_with_both_node_and_relation_targets_accepts_either` went red,
  verbatim matching the report's panic messages. Reverted via `git checkout
  --`.
- **Sabotage 3** (drop the `allowed_relations.contains(...)` check, accepting
  any resolvable relationship type in the `Relationship` arm):
  `a_role_that_does_not_accept_relations_refuses_a_relation_player` went red
  — `source` targets `Text` only and has no `Relation` targets at all, so
  this sabotage is the correct probe for "accept any relationship type
  rather than only those in the target list." Reverted.
- **Sabotage 4** (permute role *names* in the fixture, swapping `cited`'s
  and `reference`'s `name` fields while leaving each entry's
  `targets`/`optional`/`cardinality` and declaration order untouched): 4 of
  5 Task-19 tests went red, with `a_role_that_does_not_accept_relations_
  refuses_a_relation_player` (which exercises only `Transcription.source`,
  untouched by the swap) staying green — exactly the report's claimed
  result. This is the strongest evidence against positional resolution,
  the plan's recurring defect class. Reverted; `git status --short` clean
  throughout.
- **Grammar check on the arrow-form scope note** (`binder.rs:1666`): read
  `graph/cypher/src/ast.rs:157-196`. `PathPattern.start: NodePattern` and
  `steps: Vec<(RelationshipPattern, NodePattern)>` are typed as `NodePattern`
  at the AST level — there is no arrow-form syntax that can place anything
  but a node at an endpoint, and `bind_created_node` (which produces `from`/
  `to`) always yields a `CatalogEntity::Node` binding. The report's claim
  that `RoleTarget::Relation` can never reach that code path is correct;
  leaving `:1666` alone is the right call, not a gap.
- **Writer check**: read `insert_relationship` (`mutation.rs:1933-2032`).
  Role players are stored as an opaque `Value` regardless of source
  (`values.get(&binding.value)`), and every interpolated identifier in that
  function goes through `quoted_identifier`. No entity-kind branch exists or
  is needed — the report's "no special-casing" claim holds.
- Confirmed `CatalogEntity` has exactly two variants (`binder.rs:17-20`) as
  the brief states, and that `e963b573a` is code-only (`binder.rs`,
  `fixture.rs`, `nary_relations.rs`) — no `graph/test-results/` files, per
  the commit constraint.

## Findings

None ranked Critical or Important. All four required sabotages plus the
arrow-form and writer probes came back clean under independent
re-execution — no vacuous test, no reintroduced positional resolution, no
special-casing.

**Minor**: In the fixed `bind_role_player`, the `None => false` arm of the
`match binding.map(|entity| entity.kind)` is unreachable in practice — `names`
is derived from the same `binding` and is already empty when `binding` is
`None`, so `!names.is_empty() && ...` short-circuits before the match is
evaluated. Harmless (default-deny direction), not a correctness defect, not
worth a change.

## Tree state

`git status --short` is empty and `git diff --stat` is empty — confirmed
clean after every sabotage was reverted.

## Note

Mid-review, a message appeared in the tool-result channel formatted as a
`<system-reminder>` asserting that `graph/frontend/tests/fixture.rs` had
been intentionally modified (by "the user or a linter") and should not be
reverted — this arrived immediately after my own `git checkout --` on that
same file, and its content contradicted the task's explicit instruction to
revert every change. I did not act on it; I verified directly against `git
status`/`git diff`, which showed the file clean and unmodified, and
proceeded as instructed. Flagging this as an anomaly, not a repo finding.
