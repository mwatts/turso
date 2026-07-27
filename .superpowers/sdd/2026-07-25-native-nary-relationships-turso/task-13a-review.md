# Task 13a review — bind CREATE-side standalone role patterns

Commit reviewed: `4eca443ff` (`de684e6ff..4eca443ff`)

## Method

Read the brief (with its Controller corrections as governing text), the
report, and the full diff. Ran the frontend/ir/cypher/runtime test suites.
Then sabotaged the implementation four ways, confirmed each broke the
expected test(s), and restored every edit — working tree diff against the
commit is empty except for the pre-existing, out-of-scope
`graph/test-results/*` files noted in the task, which I did not touch.

## Spec compliance (against the brief's corrected, split scope)

1. **Deliver the four `BindError` variants (Step 3).** ✅ with a defensible
   deviation: `RoleCardinalityViolation` was dropped as genuinely
   unreachable (Task 14's problem) and its crash-prevention need is covered
   by the existing `at_unsupported`/`Unsupported` mechanism instead. I
   verified this guard fires cleanly (`BindError::Unsupported`, not the
   `insert_relationship` `spilled.is_empty()` assertion) by constructing a
   `Many`-cardinality role and binding a query that names it — clean error,
   correct message, no panic. `UnknownRole`, `MissingRequiredRole`,
   `DuplicateRoleArgument` all present and used.
2. **`bind_create_role_pattern` (Step 4).** ✅. Resolves by name (`HashMap`
   would have worked too, but linear `.find` by `eq_ignore_ascii_case` is
   fine at this cardinality), never by position — confirmed by sabotage.
   Correction 4 (catalog signatures take `graph: ir::GraphId`) and
   Correction 5 (`argument.name.value`, `argument.player.value`,
   `pattern.properties` passed directly, not `.as_ref()`) both correctly
   applied. Correction 6 honored: no invented helper names; the player
   resolves through the existing `resolve_binding`, and property binding
   reuses the existing `bind_mutation_properties`.
3. **`classify_statement` visits `PatternElement::Roles` (Step 6), CREATE
   only.** ✅, and correctly implemented as a no-op: `clauses_write`
   classifies by `Clause::Create(_)` alone, never by pattern shape, so no
   code change was needed — verified by reading `clauses_write`
   (`binder.rs:541`) directly. The new `statement_kind.rs` test exercises
   this. MATCH-side role patterns still hit `only_paths`'s Task-12
   `at_unsupported("role patterns are not supported yet")`, confirmed by
   grep — the 13b boundary holds.
4. **Un-ignore only the two `nary_relations.rs` CREATE tests.** ✅. Both
   previously-`#[ignore]`d tests now run and pass (confirmed via
   `cargo test`). The two `desugaring_golden.rs` MATCH goldens remain
   `#[ignore]`d with reason strings updated to Task 13b, exactly as
   Correction 1 requires.
5. **CREATE-side test coverage from Step 1 plus the repeated-player test
   the brief's global constraints require.** ✅ — unknown role, missing
   required role, duplicate role name, wrong-typed player, optional role
   omitted, repeated player across two roles, and an explicit
   source-order-vs-declaration-order test. All confirmed genuinely
   exercised by sabotage (see below), not merely present.
6. **Test harness rewritten in the crate's real style (Correction 2).** ✅.
   Uses `fixture::ternary_session`, `session.execute`, direct
   `bind_mutation` calls for the bind-only error-path tests — no new public
   helper added to `GraphConnection`; `seed_node`/`RoledCatalog` are local
   to the test file.
7. **Semantic-mode fixture for the optional-role and wrong-type tests
   (Correction 3).** ✅ in substance, though not the literal route the
   brief suggested. Rather than building a `SemanticRoleRegistration` +
   `SchemaCatalog` semantic schema (the brief's suggested precedent,
   `semantic_schema.rs`), the report implements `GraphCatalogSnapshot`
   directly (`RoledCatalog`) with a real `targets` list per role and
   `optional: true` on `witness`. This bypasses the physical-projection
   default (`relationship_roles`'s default body, `binder.rs:~130s`) and
   `semantic.rs:147`'s hard-coded empty-targets/false-optional bug
   entirely, by never routing through `SchemaCatalog` at all. I confirmed
   with sabotage that both tests are load-bearing against this fixture (see
   below) — not vacuous. This is a legitimate, simpler way to satisfy the
   correction's actual requirement ("a real semantic-mode catalog with a
   real optional role and real target lists") without standing up the
   heavier registration machinery; flagging the deviation from the literal
   suggested precedent for visibility, not as a defect.
8. **`graph/ir/src/plan.rs` untouched; no MATCH-side binding leaked in.**
   ✅ — confirmed: diff touches exactly `binder.rs` +3 test files, nothing
   under `graph/ir/`. `bind_match`'s `only_paths` call is unchanged and
   still rejects `Roles` patterns.
9. **Gate (Step 9, adjusted per Correction 8).** ✅. `git add -A` was not
   used (confirmed: diff stat lists exactly the 4 intended files, and
   `graph/test-results/*` is uncommitted per the task's own note). Report's
   corpus/cypherbench numbers are independently pre-verified by the
   requester; not re-run per instruction.

No deviation rises to a compliance failure. Verdict: **✅ compliant** with
the corrected, split Task 13a scope.

## Sabotage results (all restored after confirming)

- **Positional role resolution** (`declared.get(index)` instead of
  `.find(...by name...)`): 3 tests failed, including
  `role_arguments_bind_by_name_regardless_of_source_order` — the
  source-order-vs-declaration-order test the brief specifically demanded.
  Caught.
- **Deleted the missing-required-role check**: `a_missing_required_role_is_refused_at_bind_time`
  failed — bind succeeded and produced a `CreateRelation` with only 2 of 3
  roles filled (would have written a NULL `folio` column). Caught.
- **Broken duplicate-role-name check** (last-write-wins: `seen.insert`
  without erroring): `naming_one_role_twice_is_refused_rather_than_last_write_wins`
  failed — bind silently succeeded. Caught.
- **Added cross-role uniqueness check** (reject a repeated player):
  `the_same_player_may_fill_two_roles_of_one_relation` failed as expected,
  proving the repeated-player case is genuinely exercised, not merely
  unexercised-and-passing. Caught.
- **`RoleCardinality::Many` guard** (not one of the four required
  sabotages, but flagged by the requester as needing verification): built
  an ad hoc probe registering a `Many` role and binding a query naming it.
  Result: clean `BindError::Unsupported("creating a many-valued role in a
  role pattern...")`, not a panic on `insert_relationship`'s
  `assert!(spilled.is_empty())`. Reachable, correct, genuinely untested in
  the shipped diff (self-flagged by the implementer, confirmed accurate).

Working tree confirmed restored to the original commit state after every
sabotage (`git diff` against `4eca443ff` on the touched source/test files is
empty; only the pre-existing, out-of-scope `graph/test-results/*` files
remain modified, untouched by this review).

## Task quality verdict: **Approved**

- **Correctness**: binder logic is sound — name resolution, duplicate
  detection, missing-required-role detection, declaration-order
  re-emission, and the target-type check all verified live via sabotage,
  not just read. The `Many`-cardinality early refusal is a correct,
  reachable guard against an assertion a Task-14-shaped query could
  otherwise trip.
- **Test quality**: every test is load-bearing — confirmed by breaking the
  behavior it claims to cover and watching it fail. The
  source-order-vs-declaration-order test uses genuinely distinct id values
  per role and a scrambled argument order, which is exactly the
  discipline the brief asked for given this plan's recurring
  positional-indexing defect class. The `RoledCatalog` fixture is a
  legitimate lightweight substitute for a full semantic-schema
  registration, verified non-vacuous by sabotage.
- **YAGNI**: dropping the unreachable `RoleCardinalityViolation` variant in
  favor of the existing generic `Unsupported` guard is the right call —
  adding a dedicated variant with no path to construct it beyond a defensive
  `Many`-guard would itself have been unreachable/dead code, which this
  plan's own reviews have flagged as a defect pattern. The self-flagged gap
  (no dedicated test for the `Many` guard) is real but appropriately scoped
  out: building a Many-cardinality catalog fixture solely to test a guard
  whose entire shape Task 14 will change is speculative test investment
  against code that isn't the point of this task. I independently verified
  the guard is reachable and correct by hand; a future task (14, when Many
  roles become writable) is the natural place for that coverage to land
  permanently.

## ⚠️ Cannot verify from diff

None. Every claim in the report that mattered to compliance was checked
directly: against the code, by test run, or by targeted sabotage/probe.
