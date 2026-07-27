### Task 17: Role-aware traversal, path policy, and the semantic profile

> **CONTROLLER CORRECTIONS — these override the brief body below wherever
> they conflict. Every item was verified against the tree at `aa053d8a2`.
> Read this section first, then the body.**
>
> **A. The brief omits this task's other half: deleting `ir::Direction`.**
> Earlier tasks left `Direction` standing as a deliberate temporary and the
> code says so in three places, all naming Task 17 as the owner:
> `graph/runtime/src/traversal.rs:51` ("Goes away with `direction` once Task
> 17"), and `graph/frontend/src/graph_expand.rs:492-494` and `:1021`
> (`role_pair_to_direction` is "the one place left where a role pair is
> translated into a binary `Direction` — its entire justification is" that
> the runtime still requires one).
>
> Once adjacency is keyed by role pair, nothing needs `Direction`. Delete it
> and everything propping it up:
> - `ir::Direction` itself
> - `graph/runtime/src/csr.rs` — the `Direction` uses in edge building and
>   `neighbors` (`csr.rs:12`, `:34`, `:174`, `:192`, `:218-221`)
> - `graph/runtime/src/traversal.rs` — the `direction` field (`:36`), its
>   doc comment at `:51`, and the test uses at `:64`, `:442`, `:549`,
>   `:614`, `:649`
> - `graph/runtime/src/shortest.rs` — the `direction` field (`:26`, `:282`)
> - `graph/frontend/src/graph_expand.rs` — `role_pair_to_direction`
>   (`:496`), its call site (`:238`), and its test
>   `role_pair_to_direction_resolves_the_four_documented_cases` (`:1032`).
>   That test dies with the function; do not port it.
>
> This is the task's most important deliverable for the plan's central
> invariant. `role_pair_to_direction` matches on the literal strings
> `"start"`/`"end"` — the last hard-coded binary assumption in general
> machinery. If you finish the adjacency work but leave that function
> standing, the task is not done.
>
> **B. Step 5's arity guard references a parameter that does not exist.**
> `resolve_path_algorithm` (`path_policy.rs:103`) takes exactly
> `(uniqueness, selector, weights)`, and the brief's own new signature adds
> only `arity` and `role_pair`. Nothing passes `relationship_type`, so the
> guard as written cannot compile and the error variant has no way to be
> populated.
>
> Ruling: **the variant is `RolePairRequired { arity: usize }`** and the
> message drops the type name — e.g. "variable-length traversal over a
> relation with {arity} roles must name a role pair: it exposes {} directed
> pairs". `resolve_path_algorithm` is a pure legality table; threading a
> schema name into it is scope creep, and a caller that has the name can add
> it to its own error context.
>
> Consequently **Step 5's note about dropping `Copy` is wrong — skip it.**
> `PathPolicyError` derives `Copy` at `path_policy.rs:91`; a `usize`-only
> variant keeps that derive, so there are no call sites to fix. The brief's
> Step 1 test still matches: `RolePairRequired { arity: 3, .. }` works
> unchanged.
>
> **C. Step 6's snapshot references are stale.** There is no
> `start_column`/`end_column` extraction at `snapshot.rs:631-632`. The real
> binary-only assumption is two `.expect(...)` calls at
> `graph/frontend/src/snapshot.rs:617-623`:
>
> ```rust
> // Traversal snapshots are binary today: every relationship source
> // consumed here is registered with `start`/`end` roles (n-ary
> // traversal is a later task).
> let start_role = source.role_by_name("start")
>     .expect("traversal snapshot source has a start role");
> let end_role = source.role_by_name("end")
>     .expect("traversal snapshot source has an end role");
> ```
>
> Those two `expect`s, the comment above them, and the two-column `SELECT`
> they feed are what Step 6 replaces with one pass per ordered pair of
> single-valued roles plus one pass per (`One`, `Many`) pair joining the
> spill table. This is also why `ternary_session` could not eagerly build a
> traversal snapshot — after this task it can, and it would be worth
> checking whether that fixture's comment about the binary-only snapshot
> builder is now stale.
>
> **D. Confirmed-correct references, so you can move fast on these:**
> `PATH_POLICY_VERSION: u32 = 1` (`path_policy.rs:22`);
> `PathPolicyError` at `:91`; `Graph { nodes, node_indexes, forward, reverse }`
> at `csr.rs:61`; `EdgeInput` at `csr.rs:17`; `neighbors` at `csr.rs:171`;
> `SEMANTIC_PROFILE_VERSION: u32 = 2` (`graph/ir/src/semantics.rs:15`) with
> `path_policy_version: 1` at `:93`; the pinned digest
> `"fnv1a64:ad3c7f2313ac0e5d"` in `graph/ir/tests/semantic_profile_pin.rs:9`,
> whose doc comment says "at version 2" and must be updated to 3 alongside
> the digest. `GRAPH_CATALOG_VERSION` is `u64 = 3` at
> `graph/frontend/src/catalog.rs:24` — bump it to 4 there, not in
> `snapshot.rs`, which only imports it.
>
> **E. Step 1's csr tests assume helpers that may not exist.**
> `ternary_graph()`, `binary_graph()`, `shortest_path_in()`, `node()`,
> `role()`, `relation_type()` are written as if already available. Check
> `csr.rs`'s existing test module first and build on whatever constructors
> are actually there rather than inventing a parallel set.
>
> **F. Sequencing.** This is the largest task in the plan and the pieces are
> coupled: the snapshot must emit role-pair edges before the CSR can key by
> them, and the profile pin must move together with the policy version. Keep
> it one task, but sequence your commits-in-progress so the tree compiles at
> each step. If you find it genuinely cannot be done without a broken
> intermediate state, say so in your report rather than forcing it.
>
> **G. Corpus risk is real here, unlike every prior task.** This is the
> first change to traversal and runtime behavior rather than to binding and
> writing. The corpus is the blast radius. If a non-`tck` suite moves off its
> baseline, that is a BLOCKED report with the failing suite named — do not
> re-run until it looks better, and do not adjust a baseline.
>
> **H. Step 9 gate corrections.** `-p turso_cypher` names a package that
> does not exist — it is `turso_graph_cypher`. Use `git add` with explicit
> paths, not `git add -A`. Run both gates **before** committing and commit
> the code only; the `graph/test-results/` rows are committed separately by
> the controller. The corpus gate is **per suite**: every non-`tck` suite
> exactly at its baseline, and `tck-deep` within 3329-3332 (flaky by ±2 on
> identical commits). There is no single total — do not write "corpus at
> 8,926"; state the per-suite result you actually observed.
>
> **I. The signature change has a wider fanout than "two call sites", and
> the extra sites are a gift — use them.** `resolve_path_algorithm` is
> called from `graph/runtime/src/shortest.rs` at `:42` and `:144`
> (production) and `:373`, `:382`, `:397` (tests), plus re-exported from
> `lib.rs:19`. The two production sites are `debug_assert_eq!`s with
> all-constant arguments that exist purely to state the BFS/Dijkstra
> dependency explicitly:
>
> ```rust
> debug_assert_eq!(
>     resolve_path_algorithm(PathUniqueness::Walk, PathSelector::Shortest,
>                            WeightClass::Unweighted),
>     Ok(PathAlgorithm::BreadthFirst)
> );
> ```
>
> Pass **arity 2 and no role pair** at both. That is not busywork: it pins
> at the call site that a two-role relation with no role pair still resolves
> to exactly today's algorithm, which is the plan's central invariant
> ("binary is a layout, not a kind") enforced where it would break first.
> Keep both assertions in that shape.
>
> Note `ShortestPathRequest.direction: Direction` at `shortest.rs:26` — that
> is one of the fields Correction A deletes.
>
> **J. The test the brief tells you to extend currently asserts nothing.**
> `every_combination_in_the_table_has_a_verdict`
> (`path_policy.rs:308-336`) loops the three dimensions and then asserts
> `verdict.is_ok() || verdict.is_err()`, a tautology — it catches a panic
> and nothing else, despite a comment claiming "No combination may fall
> through to a default."
>
> Do not extend the tautology into the arity dimension; a vacuous loop with
> one more nested `for` is worse than none. The new dimension must assert
> something real, and there are exactly two facts worth asserting:
> arity ≥ 3 with no role pair is `Err(RolePairRequired { .. })`, and arity 2
> with no role pair yields **the identical verdict the three-argument call
> yields today** for every combination. Write the second one as a direct
> comparison against the pre-existing expected verdict, so a future edit
> that special-cases binary fails loudly.
>
> Rewriting the existing three-dimension assertions is out of scope —
> report the tautology in your report and leave it.

---

### Task 17: Role-aware traversal, path policy, and the semantic profile (original brief text, superseded above)

**Files:**
- Modify: `graph/runtime/src/csr.rs:30-200`, `graph/runtime/src/traversal.rs`, `graph/runtime/src/path_policy.rs:22`, `:102-179`
- Modify: `graph/frontend/src/snapshot.rs:620-660`
- Modify: `graph/ir/src/semantics.rs:8`, `:40-90`
- Modify: `graph/ir/tests/semantic_profile_pin.rs:8-9`
- Test: `graph/runtime/src/csr.rs`, `graph/runtime/src/path_policy.rs`

**Interfaces:**
- Produces:
  - `EdgeInput { relationship, from_role: RoleId, to_role: RoleId, source, target, relationship_type, weight }`
  - `Graph { nodes, node_indexes, adjacency: HashMap<(RelationshipTypeId, RoleId, RoleId), Csr> }`
  - `Graph::neighbors(&self, node: NodeId, pair: (RelationshipTypeId, RoleId, RoleId)) -> NeighborCursor`
  - `PathPolicyError::RolePairRequired { relationship_type: String, arity: usize }`
  - `resolve_path_algorithm(uniqueness, selector, weights, arity: usize, role_pair: Option<(RoleId, RoleId)>)`
  - `PATH_POLICY_VERSION: u32 = 2`; `SEMANTIC_PROFILE_VERSION: u32 = 3`; `SEMANTIC_PROFILE.path_policy_version = 2`.

- [ ] **Step 1: Write the failing tests**

In `graph/runtime/src/path_policy.rs`'s test module:

```rust
    #[test]
    fn a_relation_with_more_than_two_roles_requires_an_explicit_role_pair() {
        // A k-role relation exposes k*(k-1) directed pairs. Picking one is a
        // guess about which traversal the author meant, and the wrong guess
        // returns a plausible, wrong path.
        assert!(matches!(
            resolve_path_algorithm(
                PathUniqueness::Trail,
                PathSelector::Shortest,
                WeightClass::Unweighted,
                3,
                None,
            ),
            Err(PathPolicyError::RolePairRequired { arity: 3, .. })
        ));
    }

    #[test]
    fn a_two_role_relation_needs_no_explicit_pair_because_there_is_only_one() {
        // Arity 2 has exactly one ordered pair per direction, so there is
        // nothing to guess and every existing query keeps working.
        assert_eq!(
            resolve_path_algorithm(
                PathUniqueness::Trail,
                PathSelector::Shortest,
                WeightClass::Unweighted,
                2,
                None,
            ),
            Ok(PathAlgorithm::BreadthFirst)
        );
    }

    #[test]
    fn an_explicit_pair_over_a_ternary_relation_resolves_normally() {
        assert_eq!(
            resolve_path_algorithm(
                PathUniqueness::Trail,
                PathSelector::Shortest,
                WeightClass::Unweighted,
                3,
                Some((role(1), role(2))),
            ),
            Ok(PathAlgorithm::BreadthFirst)
        );
    }
```

In `graph/runtime/src/csr.rs`'s test module:

```rust
    #[test]
    fn adjacency_is_keyed_by_the_role_pair_it_was_built_from() {
        // A single forward/reverse pair cannot hold a ternary relation's six
        // directed pairs; merging them would let a scribe->text hop return a
        // folio.
        let graph = ternary_graph();
        let scribe_to_text = graph.neighbors(node(1), (relation_type(1), role(1), role(2)));
        assert_eq!(scribe_to_text.collect::<Vec<_>>(), vec![node(2)]);
        let scribe_to_folio = graph.neighbors(node(1), (relation_type(1), role(1), role(3)));
        assert_eq!(scribe_to_folio.collect::<Vec<_>>(), vec![node(3)]);
    }

    #[test]
    fn a_two_role_graph_has_exactly_the_two_pairs_it_had_as_forward_and_reverse() {
        let graph = binary_graph();
        assert_eq!(graph.adjacency.len(), 2, "one per direction, as before");
    }

    #[test]
    fn a_path_element_records_the_role_it_entered_and_left_by() {
        // Without the roles, a path over a ternary relation cannot be read
        // back: the same relation appears in several pairs.
        let path = shortest_path_in(&ternary_graph(), node(1), node(3));
        assert_eq!(path.elements[0].from_role, role(1));
        assert_eq!(path.elements[0].to_role, role(3));
    }
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p turso_graph_runtime`
Expected: FAIL to compile — `resolve_path_algorithm` takes three arguments and
`Graph` has no `adjacency`.

- [ ] **Step 3: Key adjacency by the role pair**

In `graph/runtime/src/csr.rs`, replace `forward`/`reverse` with:

```rust
/// Adjacency keyed by the ordered role pair it traverses.
///
/// A relation with k roles exposes k*(k-1) directed pairs. For k = 2 that is
/// exactly the forward and reverse CSR this replaces, so a binary graph builds
/// the same two structures it always did.
pub struct Graph {
    pub nodes: Vec<NodeId>,
    pub node_indexes: HashMap<NodeId, usize>,
    pub adjacency: HashMap<(RelationshipTypeId, RoleId, RoleId), Csr>,
}

impl Graph {
    pub fn neighbors(
        &self,
        node: NodeId,
        pair: (RelationshipTypeId, RoleId, RoleId),
    ) -> NeighborCursor<'_> {
        match self.adjacency.get(&pair) {
            Some(csr) => NeighborCursor::over(csr, self.node_indexes.get(&node).copied()),
            None => NeighborCursor::empty(),
        }
    }
}
```

and build one CSR per observed pair from the edge inputs, which now carry
`from_role`/`to_role`.

- [ ] **Step 4: Record roles on path elements**

Add `from_role: RoleId` and `to_role: RoleId` to the path element type in
`graph/runtime/src/traversal.rs`, populated from the pair the hop traversed.

- [ ] **Step 5: Extend the path policy**

In `graph/runtime/src/path_policy.rs`:

```rust
/// Bump on any change to the table below, and mirror into
/// `turso_graph_ir::SEMANTIC_PROFILE.path_policy_version`.
pub const PATH_POLICY_VERSION: u32 = 2;
```

Add the variant and the arity guard at the top of `resolve_path_algorithm`,
before the selector match:

```rust
    // A relation with k roles exposes k*(k-1) directed pairs. Choosing one
    // silently would answer a question the author did not ask, so the pair is
    // required rather than defaulted. Arity 2 has one pair per direction and
    // needs no annotation.
    if arity > 2 && role_pair.is_none() {
        return Err(PathPolicyError::RolePairRequired {
            relationship_type: relationship_type.to_owned(),
            arity,
        });
    }
```

```rust
    #[error("variable-length traversal over `{relationship_type}` must name a role pair: it has {arity} roles and therefore {} directed pairs", arity * (arity - 1))]
    RolePairRequired {
        relationship_type: String,
        arity: usize,
    },
```

`PathPolicyError` currently derives `Copy`; `RolePairRequired` carries a
`String`, so drop `Copy` from the derive and fix the two call sites that rely on
it.

Update `every_combination_in_the_table_has_a_verdict` to iterate arity 2 and 3
with and without a pair, so the table stays total over the new dimension.

- [ ] **Step 6: Extract role-pair edges into the snapshot**

In `graph/frontend/src/snapshot.rs`, replace the `start_column`/`end_column`
edge extraction (lines 631-632) with one pass per ordered pair of single-valued
roles, plus one pass per (`One`, `Many`) pair joining the spill table. Bump
`GRAPH_CATALOG_VERSION` to 4.

- [ ] **Step 7: Bump the semantic profile and re-pin the digest**

In `graph/ir/src/semantics.rs`:

```rust
pub const SEMANTIC_PROFILE_VERSION: u32 = 3;
```

with `path_policy_version: 2` and a new recorded choice:

```rust
    relationship_arity: "native n-ary: a relation declares named roles; \
                         binary is the two-role layout, not a separate kind",
```

Run the pin test, read the observed digest from the failure, and paste it into
`graph/ir/tests/semantic_profile_pin.rs`:

```rust
/// Digest of `SEMANTIC_PROFILE.render()` at version 3.
const PINNED_DIGEST: &str = "<paste the digest the failing test prints>";
```

- [ ] **Step 8: Run to verify they pass**

Run: `cargo test -p turso_graph_ir -p turso_graph_runtime -p turso_graph_frontend`
Expected: PASS, including `the_semantic_profile_mirrors_this_policy_version`.

- [ ] **Step 9: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime -p turso_cypher
mise run corpus
mise run cypherbench-sample
git add -A
git commit -S -m "graph/runtime: traverse by role pair and require one past arity 2

Adjacency is keyed by the ordered role pair it traverses instead of a single
forward/reverse pair, which for a two-role relation is exactly the two
structures it built before. Path elements record the role entered and left,
because a relation appearing in several pairs cannot otherwise be read back
from a path.

Variable-length and shortest-path traversal over a relation with more than
two roles must name a role pair: k roles expose k*(k-1) directed pairs, and
choosing one silently would answer a question the author did not ask.

PATH_POLICY_VERSION and SEMANTIC_PROFILE_VERSION move together with the
re-pinned digest, so every recorded corpus row stays interpretable against
the profile it was produced under.

Tests: csr role-pair adjacency and role-annotated paths, path_policy arity
rules, semantic profile pin; corpus at 8,926; cypherbench at baseline."
```

---

