# Task 14b fix round 1 — scoped re-review

Reviewed diff `6a5036017..8e2965192` (test-only, +169 lines across
`graph/frontend/tests/fixture.rs` and `graph/frontend/tests/nary_relations.rs`)
against the two Important findings from the full review. All verification
below was done independently (edited `lowering.rs`, ran the suite, reverted)
rather than trusting the report.

## Finding 1 — two `Many` roles named at once → Cartesian product

**CLOSED.**

- Test: `naming_two_many_valued_roles_at_once_produces_their_cartesian_product`
  (new fixture `two_many_roles_session()`). Asserts a literal 4-row vector:
  ```rust
  assert_eq!(
      rows,
      vec![
          vec![Value::from_i64(1), Value::from_i64(3)],
          vec![Value::from_i64(1), Value::from_i64(4)],
          vec![Value::from_i64(2), Value::from_i64(3)],
          vec![Value::from_i64(2), Value::from_i64(4)],
      ],
      "2 guests x 2 witnesses must read back as their full 4-row Cartesian \
       product, one row per (guest, witness) pair"
  );
  ```
  Exact literal count, not "at least one" and not derived from the query's
  own output.
- Sabotage performed myself: in `lower_role_join`'s `Fresh` arm
  (`graph/frontend/src/lowering.rs:1633`), changed the Many-role spill join
  from `JOIN <spill_table> AS s ON s.relation_id = q.<relation_column>` to
  `JOIN (SELECT relation_id, min(node_id) AS node_id FROM <spill_table>
  GROUP BY relation_id) AS s ON ...` — collapses each Many role join to at
  most one row per relation, which necessarily collapses any product of two
  such joins. Ran `cargo test -p turso_graph_frontend --test nary_relations`:
  ```
  thread 'naming_two_many_valued_roles_at_once_produces_their_cartesian_product' panicked at graph/frontend/tests/nary_relations.rs:1581:5:
  assertion `left == right` failed: 2 guests x 2 witnesses must read back as their full 4-row Cartesian product, one row per (guest, witness) pair
    left: [[Numeric(Integer(1)), Numeric(Integer(3))]]
   right: [[Numeric(Integer(1)), Numeric(Integer(3))], [Numeric(Integer(1)), Numeric(Integer(4))], [Numeric(Integer(2)), Numeric(Integer(3))], [Numeric(Integer(2)), Numeric(Integer(4))]]
  ```
  (34 passed; 5 failed total — the finding-1 test failed with the exact
  row-count mismatch required; four other Many-hop tests also collapsed as a
  side effect, which is expected since this sabotage degrades the shared
  Fresh-arm join, not something specific to the two-role case.) Reverted;
  confirmed clean via `git status --short` and 39/39 passing again.

## Finding 2 — duplicated player in one `Many` role's spill table → duplicate rows

**CLOSED.**

- Test: `a_duplicated_player_in_one_many_role_produces_duplicate_rows` (uses
  existing `witnessed_session()`). Asserts a literal 2-row vector:
  ```rust
  assert_eq!(
      rows,
      vec![vec![Value::from_i64(3)], vec![Value::from_i64(3)]],
      "the spill table holds two rows for player 3 (no unique constraint \
       on (relation_id, node_id)), so the hop must surface both -- \
       de-duplicating here would silently discard what the write side \
       accepted by design"
  );
  ```
  Exact literal count (2), not a range or a derived value.
- Sabotage performed myself: added `DISTINCT` to the Fresh arm's outer
  projection (`SELECT q.*, ...` → `SELECT DISTINCT q.*, ...`,
  `graph/frontend/src/lowering.rs:1650`). Ran the same command:
  ```
  thread 'a_duplicated_player_in_one_many_role_produces_duplicate_rows' panicked at graph/frontend/tests/nary_relations.rs:1627:5:
  assertion `left == right` failed: the spill table holds two rows for player 3 (no unique constraint on (relation_id, node_id)), so the hop must surface both -- de-duplicating here would silently discard what the write side accepted by design
    left: [[Numeric(Integer(3))]]
   right: [[Numeric(Integer(3))], [Numeric(Integer(3))]]
  ```
  38 passed, exactly 1 failed — precisely and only the targeted test.
  Reverted; confirmed clean via `git status --short` and 39/39 passing again.

## New findings

None. Both fixes hold up under independent sabotage.

**Additional check performed (informational, not a defect):** permuted the
declaration order of `guest`/`witness` in the new `two_many_roles_session()`
fixture (witness declared before guest) with no code change, and reran
`naming_two_many_valued_roles_at_once_produces_their_cartesian_product` —
stayed green, as expected for `RoleId`/name-based role resolution (not
positional). Reverted; no defect found. (`fixture.rs` role-order permutation
for `witnessed_session()`'s pre-existing `start`/`end`/`witness` roles was
already covered by the original round's sabotage 4, which produced "no such
column" errors under a code-level positional-resolution sabotage — not
re-run here since it targets pre-existing tests outside this diff's scope.)

## Question C — did the new fixture code perturb existing tests?

**No.** `git diff 6a5036017..8e2965192 -- graph/frontend/tests/fixture.rs`
shows 75 insertions and 0 deletions — the new `two_many_roles_session()`
function is inserted wholesale after `witnessed_session()`'s closing brace;
nothing inside `witnessed_session()` itself (or any other existing fixture
function) changed. No shared seed data was touched, so no existing test's
expected row counts could have been silently adjusted to compensate.

## Tree state

`git status --short` is empty — all sabotage edits (to `lowering.rs` and the
transient `fixture.rs` permutation) were reverted before finishing. Working
tree matches `8e2965192` exactly.
