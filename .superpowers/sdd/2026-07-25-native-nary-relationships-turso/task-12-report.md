# Task 12 report — standalone role pattern parser support

## Status: DONE

## Commit
`1e2c16b1606f3f4bc469def772fdb81b296884bf` on `feature/graph-nary`
(`cypher: parse standalone role pattern syntax`)

## What changed
- `graph/cypher/src/cypher.pest`: added `pattern_element = { role_pattern | path_pattern }`
  (role first, so a leading `[` commits unambiguously), `role_pattern = { relationship_body ~ role_arguments }`,
  `role_arguments` (one-or-more), `role_argument = { identifier ~ ":" ~ expression }`.
  `merge_clause` and `pattern_predicate` untouched (they don't go through `pattern`).
- `graph/cypher/src/ast.rs`: new `PatternElement { Path, Roles }`, `RolePattern`, `RoleArgument`,
  `Pattern { elements, span }`. `CreateClause.paths`, `MatchClause.paths`,
  `Expression::PatternSubquery.paths` changed from `Vec<PathPattern>` to `Pattern`.
  `MergeClause.path` and `Expression::PatternPredicate.path` left alone (out of scope).
- `graph/cypher/src/parser.rs`: factored `walk_relationship_body` out of `walk_relationship`'s
  inner loop and reused it in the new `walk_role_pattern`; added `walk_pattern`, `walk_role_argument`.
  Hop range on a role pattern rejected with `ParseError::at(span, "a hop range has no meaning on a role pattern")`.
  6 new parser tests: source-order role args, repeated role player, mixed role/path pattern list,
  `[x:T]()` parse error, hop-range-on-role parse error, unchanged arrow-pattern parsing.
- `graph/frontend/src/binder.rs`: new `only_paths` helper rejects a `Roles` element with
  `BindError::Unsupported { feature: "role patterns are not supported yet", .. }` instead of
  silently dropping it. Updated all 13 real call sites (11 from the brief plus 2 more discovered
  via compile errors — two synthetic `MatchClause` constructions for `HasLabels` and
  `PatternPredicate` existence checks). New binder test asserts
  `MATCH [x:KNOWS](start: a, end: b) RETURN x` binds to that error, not an empty/partial plan.
- `graph/frontend/src/compiler.rs`: `query_needs_traversal_snapshot` now filters to `Path`
  elements (a role pattern's grammar has no hop range at all, so it structurally can't need one).

`nary_relations.rs` / `desugaring_golden.rs` `#[ignore]` attributes left untouched per correction #11;
ran both with `--ignored` and confirmed they still fail as expected (role-pattern ones fail with
the new `Unsupported` error, confirming the message flows end-to-end).

## Gate results
- `cargo fmt`: clean (reformatted the 4 touched Rust files, expected).
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`: 0 errors. Fixed one
  `needless_lifetimes` on `only_paths`. Remaining warnings are pre-existing, in untouched
  `core/mvcc/persistent_storage/logical_log.rs` and `core/vdbe/mod.rs`.
- `cargo test -p turso_graph_cypher -p turso_graph_frontend -p turso_graph_ir -p turso_graph_runtime`: all pass.
- `mise run corpus`: **8927/10242 passed** (baseline 8926/10242).
  - age-deep, cqlite-deep, grafeo-deep, sparrowdb-deep: passed counts identical to the committed
    HEAD baseline (`git show HEAD:graph/test-results/REPORT.md`) — exactly at baseline as required.
  - tck-deep: 3330 → 3331. Ran corpus twice back-to-back on identical code; the two runs also
    differ by exactly one TCK test (`tck.expressions.temporal.temporal10.scenario-12.examples-1-row-2/4`
    flipping Failed/Passed between two runs of the same code), confirming this is pre-existing
    flakiness in a temporal scenario, not caused by this change.
  - The two `grafeo.spec.rosetta.aggregation.*` failures visible in the run log are pre-existing:
    confirmed `Failed` for both in `git show HEAD:graph/test-results/REPORT.md` before this change.
    Root cause (unrelated to this task): the grammar's `ORDER` keyword matches `Order` case-insensitively
    with a following non-ident char, so a node label literally named `Order` can never satisfy
    `identifier`'s `!keyword` lookahead — a pre-existing keyword/identifier collision bug.
  - Did not stage `graph/test-results/*`.

## Concerns / notes for the caller
- Two more binder call sites needed fixing than the brief's 11 (synthetic `MatchClause` builders for
  `HasLabels` and `PatternPredicate` existence checks) — genuine compile errors from the same
  `Pattern` field-type change, fixed the same way (`only_paths` / direct `Pattern` construction).
- The pre-existing `Order`-label keyword collision (see above) is a real bug but out of scope for
  Task 12; flagging it for separate cleanup rather than fixing here (surgical-changes rule).
