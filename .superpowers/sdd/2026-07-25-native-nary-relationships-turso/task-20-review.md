# Task 20 review: documentation and gate deletion

## Verdicts

**Spec compliance: PASS.** Every requirement in `task-20-brief.md` — including
all six plan defects the brief flags as superseding the plan — was verified
against the tree, not taken on the report's word. Decision gate B is deleted
(not reworded) and replaced by a one-line pointer to the native-nary design
doc; the Global Constraint forbidding native n-ary is gone; all nine named
binary-language sites (five original + four "plan omitted") are rewritten in
role terms; the foedus reference is resolved at all three original stale
sites (two literal repoints, one eliminated along with the deleted gate text
that carried it); `graph/CONFORMANCE.md` carries the new run_id, a per-suite
table, and the tck-flake caveat; and the new "Roles" section covers every
item the brief's checklist requires.

**Task quality: PASS, high confidence.** I independently recompiled and ran
both Rust code examples from `docs/graph.md` (Quickstart and the semantic-schema
registration example) as standalone tests against the real crate — both
compile and execute unmodified. I traced every Cypher snippet and every
factual claim in the new Roles section to `graph/frontend/tests/nary_relations.rs`
and to the binder/schema_catalog source, and all matched exactly, including
the subtle `MissingSource` vs. `MissingRelationshipRole` distinction the
report called out. I mutated the code path (or the test's own input data)
behind each of the 3 new tests and confirmed each goes red — they encode real
regressions, not tautologies. Tree is clean; no changes remain.

---

## Findings

No Critical or Important findings.

**Minor — report's justification cites identifiers that don't exist in the code.**
The report's rationale for leaving `.specs/graph-semantic-schema-overlay.agent-spec.md`
lines ~305/345/348/384/393/511/538 untouched says they describe "the overlay's
own start/end-scoped registration/validation behavior (`SEMANTIC_ENDPOINTS_TABLE`,
`EndpointConstraint`, `endpoint_validation_...` test names)". I grepped
`graph/frontend/src/` and `graph/frontend/tests/` for these exact identifiers:
`EndpointConstraint` (struct) and `SEMANTIC_ENDPOINTS_TABLE` do not exist
anywhere in the current code; `InvalidEndpointType` (cited for the sibling
plan file) does not exist either — the actual bind-time error for this is
`BindError::RoleTargetTypeViolation` (`binder.rs:1689`), and the actual
overlay error enum is `SemanticCatalogError` (`semantic.rs:403`), neither of
which matches the cited names. Only `endpoint_validation_covers_both_directions`
(a test *name*, in `semantic_schema.rs:1638`) is real.

This doesn't change the correctness of the underlying judgment call — the
lines in question (test-matrix items, a SHOULD-list error-naming
recommendation, a checklist entry, a failure condition) are pre-existing,
untouched-by-this-task prose that doesn't assert anything false about the
general (now n-ary-capable) frontend; they describe either the overlay's
still-binary `SemanticEndpoint` constraint scope (verified real:
`semantic_constraints.rs:128`, exactly two variants, `Start`/`End`) or use
"endpoint" as loose terminology in a forward-looking SHOULD recommendation,
not a literal type reference. So the decision to leave them alone is right,
but the report's cited evidence for that decision was not verified against
actual identifiers and turned out to name things that don't exist. Worth a
note for whoever reads this report as a record, not worth reopening the task.

---

## Verification performed

**Compiled and ran, standalone, against the real crate (not eyeballed):**
- Quickstart (`docs/graph.md:20-58`), copied verbatim into a scratch
  integration test — compiled and passed (registers, opens, creates a
  `Person`, reads it back).
- The semantic-schema registration example (`docs/graph.md:225-260`) —
  compiled and passed as the doc presents it (registration call only; I
  initially appended a strict-mode CREATE of my own invention using the
  physical column names, which correctly failed with `PropertyNotOwned` —
  that failure is *my* fabrication, not a doc defect, since the doc's code
  block ends at the registration call and makes no claim about what a
  subsequent CREATE would look like).
- Both scratch test files were deleted afterward; `git status --short` is
  clean.

**Traced to source, not "matches the pattern used elsewhere":**
- `RelationshipSourceRegistration::binary` signature (`catalog.rs:60-81`) and
  `SemanticRelationshipType::binary` signature (`semantic.rs:302-332`) match
  the doc's call sites argument-for-argument.
- The CREATE-vs-MATCH/expand error-identity claim: read
  `binder.rs:1653-1718` (CREATE arrow path — loop at 1653 skips missing roles
  via `continue`, so it never fires `RoleTargetTypeViolation` for a
  start/end-less type; the unconditional lookup at 1701-1718 then raises
  `BindError::MissingRelationshipRole { role: "start" }`) and
  `binder.rs:3460-3567` plus `schema_catalog.rs:454-466`
  (`relationship_endpoint_sources` requires `role_by_name("start")` and
  `role_by_name("end")` to both resolve or returns `None`; the `for` loop at
  `binder.rs:3461` `continue`s past any source that returns `None`, so a
  start/end-less type never enters `expansion_sources`; the empty-check at
  `binder.rs:3506-3512` fires `BindError::MissingSource { entity: "compatible
  relationship" }` *before* the per-branch closure at 3542+ that contains the
  `MissingRelationshipRole` construction ever runs). This exactly matches
  both the report's claim and the doc's phrasing — the MATCH/expand
  `MissingRelationshipRole` construction is dead code for this scenario, the
  same way the report describes.
- Every Cypher snippet and refusal-wording claim in the new Roles section
  (three-role CREATE/MATCH syntax, bind-by-name, arrow-sugar plan equality,
  arrow-form refusal for both CREATE and expand, role-read arrow sugar
  through a `Many` role, the role/relationship-type-name ambiguity rule,
  `SET` replacing rather than appending, a relation filling another
  relation's role) — read against the corresponding test bodies in
  `nary_relations.rs` line by line; all match exactly, including exact
  variable names, exact role names (`witness` singular, not `witnesses`),
  and exact error substrings asserted.
- The "binary is a layout, not a kind" invariant: `rg -n "is_binary"
  graph/` returns zero hits — there is no such predicate anywhere in the
  crate, matching the doc's explicit claim.
- Variable-length traversal (`GraphExpand`, the `*`/`*min..max` forms) shares
  the identical `expansion_sources` computation as fixed-hop `RoleExpand`
  inside the same `bind_path` function (`binder.rs:3237` onward; the
  `relationship.range` branch that builds `min_hops`/`max_hops` sits after
  the same `expansion_sources` filter) — confirms the doc's claim that both
  forms are subject to the same start/end requirement, not just fixed-hop.

**Mutation-tested the 3 new tests (each reverted immediately after):**
- `an_arrow_form_expand_requires_a_start_and_end_role_pair`: changed
  `entity: "compatible relationship"` to a sentinel string in
  `binder.rs:3508` — test failed with a message no longer containing
  "compatible relationship". Reverted.
- `an_arrow_form_create_requires_a_start_and_end_role_pair`: changed
  `role: "start"` to a sentinel string in the `MissingRelationshipRole`
  construction at `binder.rs:1705` — test failed (message no longer
  contains "start"). Reverted.
- `a_match_role_pattern_reads_a_three_role_relation`: changed the CREATE
  statement's `year` literal from 1387 to 1400 while leaving the MATCH
  assertion at 1387 — test failed with a real value mismatch
  (`left: [[1400]], right: [[1387]]`), proving the assertion checks the
  actual round-tripped value rather than being vacuously true. Reverted.
- Confirmed via `diff` against pre-mutation backups that both `binder.rs`
  and `nary_relations.rs` are byte-identical to their pre-mutation state
  after revert; `git status --short` is clean throughout and at the end.

**Other checks:**
- `cargo test -p turso_graph_frontend --test nary_relations`: 47 passed, 0
  failed — matches the report's count.
- Confirmed the second commit (`851b5b927`) touches only
  `graph/test-results/{REPORT.md,benchmarks.jsonl,runs.jsonl}` and is
  authored separately from the implementer's commit (`26f785bfd`), matching
  the task instructions that test-results bookkeeping is committed by the
  controller, not the implementer.
- Rescanned both `.specs/graph-semantic-schema-overlay.agent-spec.md` and
  `docs/superpowers/plans/2026-07-22-graph-semantic-schema-overlay.md` for
  `binary|n-ary|nary|start/end|endpoint` (case-insensitive) after the diff;
  every remaining hit is either (a) newly-written text correctly scoping a
  claim to the overlay's own `SemanticEndpoint`/start-end-only constraint
  system (verified real: `semantic_constraints.rs:128` has exactly two
  `SemanticEndpoint` variants), (b) a historical quote/checklist entry the
  report explicitly declined to touch with articulated reasoning, or (c) the
  false-positive substring "nary" inside "ordinary" (line 81), which the
  report also caught. No missed site asserts that the general frontend is
  binary-only.

## Tree state

`git status --short` is empty. No changes remain from this review.
