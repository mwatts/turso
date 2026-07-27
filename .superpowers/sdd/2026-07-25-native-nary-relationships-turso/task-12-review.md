# Task 12 review — parse the standalone role pattern

Commit reviewed: `1e2c16b1606f3f4bc469def772fdb81b296884bf`
Diff: `review-fb1efd408..1e2c16b16.diff`

## Method

Read the brief (including all 11 controller corrections), the report, and the
full diff. Then verified by sabotage rather than by reading: made four
targeted breaking edits in the working tree, ran the relevant tests, confirmed
each broke something, then restored the exact original text and reran
`cargo test -p turso_graph_cypher -p turso_graph_frontend -p turso_graph_ir -p
turso_graph_runtime` (356 passed, 5 ignored) to confirm a clean baseline.
Working tree is restored exactly as found (`git diff --stat` shows only the
pre-existing, intentionally uncommitted `graph/test-results/*` files).

## Sabotage results

1. **Silent-drop at `bind_match`** (`binder.rs:2366`, the site the brief's
   dedicated test targets): replaced `only_paths(&clause.paths)?` with a
   `.filter_map` that silently drops `Roles` elements. Result:
   `a_standalone_role_pattern_binds_to_an_error_not_an_empty_plan` failed
   immediately (assertion mismatch — no longer returns
   `BindError::Unsupported`). **The silent-drop guard is genuinely tested and
   bites.** Restored; confirmed clean diff after restore.

2. **Positional role-argument handling**: sorted `RolePattern.roles` by name
   after parsing (simulating a positional/declaration-order assumption).
   Result: `a_standalone_role_pattern_parses_with_its_roles_in_source_order`
   failed (`["folio", "scribe", "text"]` vs expected
   `["scribe", "text", "folio"]`). Source-order preservation is genuinely
   tested. Restored.

3. **Empty role list**: relaxed grammar `role_arguments` from one-or-more
   (`role_argument ~ (...)*`) to zero-or-more (`role_argument? ~ (...)*`).
   Result: `a_role_pattern_with_no_roles_is_a_parse_error_not_an_empty_relation`
   failed — `[x:Transcription]()` parsed successfully. Confirms this is a real
   grammar-enforced constraint, not a vacuously-passing test. Restored.

4. **Hop range on role pattern**: removed the `if let Some(range) = range { return Err(...) }`
   check in `walk_role_pattern`. Result:
   `a_hop_range_on_a_role_pattern_is_a_parse_error` failed —
   `[r:T*1..3](start: a)` parsed successfully. Confirms the rejection is a
   real code-level check, not something the grammar already rules out by
   construction. Restored.

All four sabotages were caught by an existing, non-ignored test, and all four
were restored to the exact original text (verified via `git diff`).

## Downstream call-site audit (Correction 10 — the highest-risk requirement)

Grepped `graph/frontend/src/{binder,compiler}.rs` for every consumer of
`cypher::Pattern`/`.paths`/`PatternElement`, plus `graph/frontend/src/lowering.rs`
and the `ir`/`runtime` crates for any other consumer of the parser's
`Pattern` type. Found exactly 13 sites, all in `binder.rs` and `compiler.rs`;
no other file in the tree touches `cypher::Pattern`:

| Site (current tree) | Behavior on `Roles` |
|---|---|
| `binder.rs:910` (top-level CREATE) | `only_paths(&value.paths)?` → errors |
| `binder.rs:1127` (FOREACH body CREATE) | `only_paths(&value.paths)?` → errors |
| `binder.rs:1192`/`1202` (`bind_staged_match`) | `only_paths(&clause.paths)?` → errors, computed once and reused |
| `binder.rs:1256` (`bind_staged_match` inner clause) | clones the whole `Pattern` into a new `MatchClause` and calls `self.bind_match`, which itself calls `only_paths` → errors (not a bypass) |
| `binder.rs:2366` (`bind_match`, the primary MATCH binder) | `only_paths(&clause.paths)?` → errors — this is the site the dedicated binder test exercises |
| `binder.rs:4074` (`bind_pattern_subquery`) | `only_paths(&clause.paths)?` → errors |
| `binder.rs:4434`/`4477` (synthetic `MatchClause` builders for `HasLabels`/`PatternPredicate`, not in the brief's original 11) | construct `Pattern` directly with a `PatternElement::Path` — no `Roles` possible, correctly scoped |
| `binder.rs:6361`/`6400` (`rename_match_clause`) | matches `Path` vs `Roles` explicitly and passes a `Roles` through unchanged rather than dropping or panicking; single caller (`bind_staged_match`) already calls `only_paths` before this runs, so `Roles` is provably unreachable here, not just asserted to be |
| `compiler.rs:191` (`query_needs_traversal_snapshot`) | returns `false` for a `Roles` element (a role pattern's grammar structurally cannot carry a hop range) — this is a boolean heuristic consumed immediately before `bind(...)?` in `compiler.rs:121`, which itself errors via `bind_match`'s `only_paths` before the heuristic's answer has any consequence. Not a silent-acceptance path. |

Every site either errors directly through `only_paths`, delegates to a site
that does, or is provably unreachable for `Roles` by construction. No
`.filter_map`/`if let Path(..)` skip pattern reaches user-visible behavior.
The `only_paths` helper itself is well-commented, explaining exactly the
wrong-answer scenario it prevents.

Confirmed the required non-ignored binder test exists
(`a_standalone_role_pattern_binds_to_an_error_not_an_empty_plan`,
`binder.rs`) and — per sabotage #1 above — that it actually bites.

## Scope-boundary corrections (8, 9, 11) — spot-checked, all correct

- `MergeClause.path` and `Expression::PatternPredicate.path` are untouched
  `PathPattern`/`Box<PathPattern>`; grammar confirms `merge_clause` and
  `pattern_predicate` still reference `path_pattern` directly, never routed
  through the new `pattern`/`pattern_element` rules. Correctly out of scope.
- `walk_relationship_body` was factored out of `walk_relationship`'s inner
  loop exactly as directed and reused by `walk_role_pattern`; no four-tuple
  type was invented beyond what correction 9 permitted as a suggestion.
- `#[ignore]` attributes in `graph/frontend/tests/nary_relations.rs` and
  `desugaring_golden.rs` are untouched — confirmed zero diff against those
  files across the whole commit range.

## `ParseError` / AST shape corrections (3, 4, 5, 6, 7)

- `ParseError` remains the existing struct; no invented enum variants
  (`MalformedPattern`, `UnexpectedRule`, `RangeOnRolePattern` do not exist in
  the tree). `ParseError::at`, `unexpected`, `only_child`, `pair_span`,
  `walk_identifier`, `walk_map`, `walk_expression`, `parse_range` are reused
  as directed.
- `RolePattern`/`RoleArgument` match the `Spanned<T>` convention mirroring
  `RelationshipPattern`, derive `PartialEq` only (no `Eq`), and properties are
  the pair-vector form, not a nonexistent `MapLiteral`. No `has_property_map`
  field was added to `RolePattern`, matching `RelationshipPattern`.
- `role_arguments` is one-or-more per correction 6 (verified by sabotage #3).
- Test helpers (`pattern_of`) are hand-written in the test module rather than
  exposed publicly, matching correction 7.

## `Order` keyword collision — pre-existing, not caused by this diff

Diffed `cypher.pest` lines 24/39/40 (the `ORDER` token and the `identifier`
rule's `!keyword` lookahead) between `fb1efd408` (pre-Task-12) and HEAD: byte
-identical. Task 12's grammar changes are confined to the `pattern`/
`pattern_element`/`role_*` rules added after `limit_clause`; nothing touches
the keyword/identifier rules. The implementer's claim that this is a
pre-existing bug merely discovered (not caused) by this task is correct, and
correctly left unfixed per the surgical-changes rule.

## Findings

- **binder.rs:1256 (`bind_staged_match`) clones the whole `Pattern` rather
  than calling `only_paths` again before constructing the inner
  `MatchClause`** — Minor. It is safe (the inner `self.bind_match` call
  re-validates via its own `only_paths`), but it means the function computes
  `only_paths` once at the top for the `named paths`/correlation-variable
  loops and then implicitly relies on `bind_match`'s later call for the
  actual guarantee. Not a bug; a slightly non-obvious two-hop guarantee worth
  a one-line comment for the next reader, not a blocker.
- **`rename_match_clause`'s "unreachable in practice" comment is correct but
  load-bearing on there being exactly one caller** — Minor. If a second
  caller is ever added without first calling `only_paths`, the pass-through
  arm would silently carry a `Roles` element forward instead of erroring.
  Currently true and correctly commented; flagging only so a future
  reviewer rechecks this invariant if `rename_match_clause` gains a second
  caller.
- No Critical or Important findings. All four sabotage attempts were caught
  by existing tests; all scope boundaries, corrected-AST shapes, and
  parser-error conventions were followed; the two extra call sites beyond
  the brief's 11 were genuine and handled the same way as the listed ones;
  the `Order` keyword bug is confirmed pre-existing and out of scope.

## Verdicts

1. **Spec compliance: ✅.** Every controller correction (1–11) was followed:
   crate name, banned `git add -A` (staged only source/test files — confirmed
   via the commit's file list), no invented `ParseError` variants, `Spanned`
   AST shapes, one-or-more `role_arguments`, correct test helpers, exact
   three-field scope (`CreateClause.paths`, `MatchClause.paths`,
   `PatternSubquery.paths`) with `Merge`/`PatternPredicate` correctly left
   alone, `walk_relationship_body` factored as directed, all 11 brief-listed
   call sites plus 2 more found by the implementer are updated to error
   rather than drop, the required non-ignored binder test exists and bites,
   and `#[ignore]` attributes are untouched.
2. **Task quality: approved.** Correctness verified by sabotage at every
   angle the review brief called out as highest-risk (silent-drop, positional
   role handling, empty-role-list, hop-range) — all four caught by real,
   non-ignored tests. Test quality is strong: it directly encodes the
   wrong-answer scenario the task exists to prevent, not just parser shape.
   No YAGNI violations — no speculative binding logic snuck in ahead of
   Task 13, `only_paths` is a minimal, well-scoped guard, and the diff stays
   inside the three in-scope fields.
