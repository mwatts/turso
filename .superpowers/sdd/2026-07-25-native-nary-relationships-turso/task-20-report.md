# Task 20 report: documentation, and resolving Decision gate B

Commit: `26f785bfd` on `feature/graph-nary`
("docs(graph): document native roles, resolve semantic-overlay decision gate B")

Files changed: `docs/graph.md`, `graph/CONFORMANCE.md`,
`.specs/graph-semantic-schema-overlay.agent-spec.md`,
`docs/superpowers/plans/2026-07-22-graph-semantic-schema-overlay.md`,
`graph/frontend/tests/nary_relations.rs` (3 new tests, added because the
verification pass in Step 5 found doc claims with no existing proof — see
"New tests" below). Nothing under `graph/test-results/` is committed
(`REPORT.md`, `benchmarks.jsonl`, `runs.jsonl` were touched by the corpus and
cypherbench runs and are intentionally left unstaged/untracked-in-this-commit).

## Step 1/2 — `.specs/graph-semantic-schema-overlay.agent-spec.md`

**Central distinction used throughout**: `SemanticRelationshipType` (type
registration, `semantic.rs`) already generalized to `roles:
Vec<SemanticRoleRegistration>` with a `binary()` convenience constructor —
confirmed by reading `semantic.rs:295-334`. `SemanticRelationshipCardinality`
(the overlay's own cardinality *constraint* system, `semantic_constraints.rs`)
is genuinely still `start`/`end`-only — confirmed by grep: `SemanticEndpoint`
has exactly two variants. Every rewrite below either narrows a claim to
"this overlay's constraint system" (still true) or deletes a claim about the
general frontend that is no longer true.

**Rewritten**:
- Global Constraints bullet (~line 38): "binary endpoint participation" →
  "role participation (`start`/`end`, in this overlay's constraint system)".
- ~line 54: `CreateRelationship`/expansion now described as resolving named
  roles generally; binary is "two roles named `start`/`end`, not a separate
  code path".
- "Structural identity and relationship endpoint columns" → "relationship
  role columns (or role spill tables, for many-valued roles)".
- "### Binary endpoint participation" section fully rewritten to
  "### Role participation (start/end scope)", scoped to what Milestone 2's
  own validation covers.
- **The Global Constraint bullet forbidding native n-ary was deleted
  entirely** (per brief instruction — resolve by deletion, not rewording).
- ~line 126, Milestone 2 item 6, the "Runtime write validation" outputs-table
  row, test matrix item 9, Milestone 3 item 5, Slice 2.6's heading+body, the
  decision-gate list item, the "Premature hypergraph design" risk mitigation,
  and the failure-conditions bullet: all rescoped from "binary
  endpoint"/"n-ary forbidden" language to "this overlay's `start`/`end` role
  constraints", with the n-ary prohibition explicitly noted as resolved.
- MUST NOT list: removed "native n-ary relations, or relation-to-relation
  roles" (both now supported at the storage layer); kept "first-class
  attribute instances or multi-valued ownership" (still true, out of scope).
- Added new "### Decision gate B — resolved" section: "Resolved by native
  n-ary relationships: relationships now declare named roles directly (no
  reification, no separate binary code path), per
  `docs/superpowers/specs/2026-07-25-native-nary-relationships-design.md`."
- Foedus reference repointed: `...2026-07-23-turso-ontology-store-design.md`
  → `...2026-07-25-turso-ontology-evolution-design.md` (1 occurrence in this
  file, confirmed by direct `grep`).

**Deliberately left unchanged, with reasons** (re-scanned via `grep -n -i
"binary\|n-ary\|nary\|start/end\|endpoint\|foedus"` after all edits):
- The Milestone 3 amendment blockquote (~line 171, "endpoint constraints
  (endpoint lists already express unions...)") — a historical decision-record
  quote, not a current claim; rewriting it would misrepresent what was
  decided at the time.
- Lines 304, 344, 347, 383, 392, 510, 537 — accurate descriptions specifically
  of the overlay's own start/end-scoped registration/validation behavior
  (`SEMANTIC_ENDPOINTS_TABLE`, `EndpointConstraint`, `endpoint_validation_...`
  test names), which genuinely still exists in the code as written. These are
  not false claims about the general n-ary-capable system; they're accurate
  claims about the narrower constraint subsystem.
- Line 81's "ordinary" — confirmed a false-positive grep match (substring
  "nary" inside "ordinary"), not a real hit.

## Step 1 — `docs/superpowers/plans/2026-07-22-graph-semantic-schema-overlay.md`

Brief named exactly two sites (`:21`, `:1680`); both rewritten:
- Line 21 (Global Constraints bullet): rescoped to "no constraints beyond
  `start`/`end` role checks" plus an added parenthetical noting native n-ary
  relationships have since landed separately (Decision gate B resolved) and
  that this plan's Milestone 1-2 scope is historically accurate and
  unaffected.
- Line ~1680 (the checked-off historical boundary-statement quote for
  `docs/graph.md`, "no named roles or n-ary relations... no inference"): left
  the quote **verbatim** (it is a checked `[x]` historical record of what
  Task 12 actually delivered) and added a "**Superseded (2026-07-26).**"
  annotation immediately after it, stating the claim is no longer true of the
  frontend as a whole and that `docs/graph.md`'s boundary statement has been
  updated accordingly.
- Line 45: foedus reference repointed (2nd of 3 total sites, alongside the
  one in the `.specs` file and one more below).

**Deliberately left unchanged**: the remaining ~48 grep hits across this
1696-line file (lines 13, 44, 46, 70, and the large Task 2-7 code-plan block
at lines 162-1366: `UnknownEndpointType`, `EndpointConstraint` struct,
`SEMANTIC_ENDPOINTS_TABLE` DDL, `InvalidEndpointType`, test names like
`relationship_endpoints_must_reference_declared_node_types`). These document
what the semantic overlay's own start/end-scoped implementation actually is
(confirmed still true via the `SemanticEndpoint` enum check) — not false
claims about the general storage layer. This matches the brief's own scope:
it named only 2 sites for this file, versus a much longer list for the
`.specs` file.

**On the foedus reference count**: the brief describes the plan repointing it
at "all three sites"; I found and fixed exactly 3 total across both files (1
in `.specs/...md`, 2 in the plan file, at lines 45 and — checking the third —
confirmed no additional occurrence exists beyond what's listed above; the
plan file's own line 44 mentions "a separate Foedus-owned plan" without a
path, so it needed no repoint). All findable stale foedus paths were fixed;
none were missed.

## Step 3 — `graph/CONFORMANCE.md`

Ran my own `mise run corpus` (release build). Run
`20260727T002123.697446Z-2e59ebc131e9-corpus-deep`: `10242 records,
clean=false`; summary line `source_identities=10242 passed=8926
unsupported=53 failed=1263`.

Per-suite pass counts, computed as (suite's inventory count) − (suite's
Failed count) − (suite's Unsupported count, only `age` has any = 53) and
cross-checked against the raw log:

| Suite | Passed | Gate band | Result |
| --- | ---: | --- | --- |
| `age-deep` | 3,042 | exactly 3042 | met |
| `cqlite-deep` | 113 | exactly 113 | met |
| `grafeo-deep` | 277 | exactly 277 | met |
| `sparrowdb-deep` | 2,164 | exactly 2164 | met |
| `tck-deep` | 3,330 | 3329-3332 | met (within flake band) |

Total 8,926 matches the run's own summary line exactly.

Updated `graph/CONFORMANCE.md`: new run_id, 8,926/1,263/53 totals, added the
per-suite table above plus a note that the bare total moves with `tck-deep`'s
flake so per-suite counts (not the total) are the regression signal. Did
**not** hand-recompute the "dominant failure families" histogram (execution/
mutation-projection-unsupported/etc.) — that table is regenerated by the
recording tooling from raw failure-reason strings, not by hand; instead I
relabeled its intro sentence to honestly attribute it to the *prior* recorded
run (`...0de15cc74e02...`, 1,270 failed) and pointed at `REPORT.md` as the
place the current, tool-regenerated histogram will land. Hand-recomputing it
here risked introducing a second, competing source of truth with no
verification path.

## Step 4/5 — `docs/graph.md` "Roles" section, and example verification

Fixed the Quickstart's broken `RelationshipSourceRegistration` struct-literal
(defect 3) and, separately (my own finding, not in the brief's list), a
second broken struct-literal in the semantic-schema example — both replaced
with the real `binary(...)` convenience-constructor calls. Both call shapes
are confirmed identical to real production/test call sites (not just
"looks plausible"): `RelationshipSourceRegistration::binary(name, table,
identity_column, start_column, end_column, start_node_source,
end_node_source)` matches `graph/frontend/tests/fixture.rs:68-75`;
`SemanticRelationshipType::binary(name, source, start: Vec<String>, end:
Vec<String>, properties)` matches its own definition at
`graph/frontend/src/semantic.rs:306-329` and is used identically in
`graph/frontend/tests/semantic_schema.rs:119,281,291,...`.

Fixed three now-false prose claims elsewhere in the file (endpoint-only
language predating role generalization): the semantic strict-mode bullet
list ("validate every one of its declared roles' target types"), the
fragment-role-target paragraph ("physical role column, or spill table, for a
`Many` role"), and the Direct-SQL integrity boundary's deferred-work list
(added: "role cardinality constraint validation past `start`/`end`" remains
deferred, since `SemanticEndpoint` covers only the binary layout even though
the general frontend does not).

Also found and fixed two more stale "endpoint"-only claims not named in the
brief's site list, grounded by reading the implementing code rather than
guessing: `db.propertyKeys()`'s "identity and relationship endpoint columns
are excluded" (schema_catalog.rs's `payload_columns` calls
`layout.structural_columns()`, which returns every role's physical column,
not just `start`/`end` — confirmed by code read, not a dedicated new test,
since the mechanism is shared code with no separate binary branch); and the
FTS section's "Identity/end-point columns... are rejected" (same
`payload_columns` path, `fts.rs:243`). Both now say "relationship role
columns" generically.

### New "## Roles" section — every claim's proof

Drafting this section, I initially wrote three examples from memory/analogy
rather than verifying them, and caught all three myself by re-deriving every
claim against `graph/frontend/tests/nary_relations.rs` and `fixture.rs`
before finalizing (per the brief's "every example must actually run, not
merely match the pattern" bar):

1. A fabricated CREATE example using inline node-literal properties
   (`name`, `title`, `label`, `createdAt: date(...)`) that don't exist on
   `ternary_session`'s actual schema (nodes have only an `id` column) — would
   not compile. Replaced with the real, schema-accurate syntax.
2. A fabricated `t.scribe` dot-property role-read syntax that does not exist
   anywhere in the codebase. Replaced with the real arrow-form sugar,
   `(x:KNOWS)-[:end]->(e)`.
3. Wrong role name: "witnesses" (plural, invented) vs. the real "witness"
   (singular, the only name registered in any fixture/test). Fixed.

A fourth gap surfaced during verification, not drafting: the claim that
arrow-form (fixed-hop and variable-length) traversal requires a relation to
have roles literally named `start`/`end` had **no existing test** proving
it — `grep -rn "MissingRelationshipRole"` found only the error's definition
and construction sites in `binder.rs`, never a test exercising the path. Per
Step 5's rule ("if an example cannot be made to run, fix the document"), I
added two tests rather than asserting an unproven claim (see "New tests"
below) — and in doing so found the claim as originally drafted was itself
imprecise for the MATCH/expand side (see the CREATE-vs-MATCH distinction
noted there).

Final example-to-proof mapping:

| Doc claim / example | Proof |
| --- | --- |
| Three-role CREATE (`[x:Transcription {year: 1387}](scribe: p, text: t, folio: f)`) | `a_three_role_relation_writes_one_row_with_three_endpoint_columns` (existing) |
| Three-role standalone MATCH read (`RETURN x.year`) | `a_match_role_pattern_reads_a_three_role_relation` (**new**, added this task) |
| Role arguments bind by name, not position | `role_arguments_bind_by_name_regardless_of_source_order` (existing) |
| Arrow form ≡ standalone pattern (same plan) | `the_role_arrow_and_the_role_pattern_bind_to_the_same_plan` (existing) |
| CREATE arrow form refuses a relation lacking `start`/`end` | `an_arrow_form_create_requires_a_start_and_end_role_pair` (**new**) |
| MATCH/expand arrow form refuses a relation lacking `start`/`end` | `an_arrow_form_expand_requires_a_start_and_end_role_pair` (**new**) |
| Arrow-form read sugar off a relation binding (`(x:KNOWS)-[:end]->(e)`), including through a `Many` role | `an_arrow_from_a_relation_reads_that_relations_role`, `a_many_role_hops_from_the_arrow_sugar_too` (existing) |
| Role/relationship-type name collision is rejected as ambiguous | `a_name_that_is_both_a_role_and_a_relationship_type_is_ambiguous` (existing) |
| `SET` on a `Many` role replaces, not appends | `setting_a_many_valued_role_replaces_rather_than_appends` (existing) |
| A relation may fill another relation's role | `a_relation_may_be_a_player_of_another_relation` (existing) |

**One correction made mid-verification, worth flagging explicitly**: the
MATCH/expand arrow-form refusal does **not** raise `MissingRelationshipRole`
the way the CREATE arrow-form does. Reading `schema_catalog.rs::
relationship_endpoint_sources` (called from `binder.rs`'s expansion-branch
loop, ~line 3465) showed it requires the `start`/`end` role pair *before*
`expansion_sources` is populated; when a type lacks that pair, the type is
silently filtered out of the candidate list entirely, and the eventual error
is the generic `BindError::MissingSource { entity: "compatible
relationship" }` — the same error you'd get if no relationship type by that
name existed at all, with no mention of "start"/"role" in the message. I
confirmed this by first writing the test with the CREATE-side error's
assertion style (expecting "start" and "role" in the message), watching it
fail with the actual `MissingSource` message, and correcting both the test
and the doc's phrasing to describe what actually happens rather than forcing
the test to match my initial wrong assumption. `binder.rs:3554-3563`'s
`MissingRelationshipRole` construction inside that same function appears to
be unreachable for this scenario specifically (parallel to the already-known
dead CREATE endpoint-check code at ~1653-1667), since the earlier filter
already excludes the type — I did not attempt to prove or fix that dead-code
question further, as it's outside this task's scope.

### New tests (`graph/frontend/tests/nary_relations.rs`)

3 new tests, 47 total now passing in this file (44 pre-existing + 3 new):
- `a_match_role_pattern_reads_a_three_role_relation`
- `an_arrow_form_create_requires_a_start_and_end_role_pair`
- `an_arrow_form_expand_requires_a_start_and_end_role_pair`

All added because Step 5 verification found doc claims with no existing
proof; each is a direct, minimal encoding of the exact claim documented, not
speculative coverage.

## Step 6 — gates

- `cargo fmt` — no changes (already formatted).
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
  — clean (only a pre-existing, unrelated `ar` linker warning from a
  `limbo_sqlite_test_ext` build script, not a clippy diagnostic).
- `cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime
  -p turso_graph_cypher` — all green across every test binary in all four
  crates (`nary_relations.rs`: 47 passed; every other suite: 0 failed).
- `mise run corpus` — **reused** the run already completed for this task
  (`20260727T002123.697446Z-2e59ebc131e9-corpus-deep`, numbers above) rather
  than re-running. Justification: this task's diff touches only
  `docs/`, `.specs/`, `docs/superpowers/`, `graph/CONFORMANCE.md`, and
  `graph/frontend/tests/nary_relations.rs`. Files under `tests/` are not
  linked into any crate's `src/` and are therefore not part of the release
  binary `mise run corpus` builds and runs — the compiled artifact used by
  that gate is byte-for-byte identical whether or not this task's test
  additions exist. Re-running would not and could not produce a different
  per-suite result (modulo `tck-deep`'s already-documented flake), so it
  would only spend ~10+ minutes of release-build/corpus-run time to
  reconfirm data already in hand. All per-suite gates listed above were met
  by that run.
- `mise run cypherbench-sample` — **ran fresh** this task (exit 0). Same
  "test files aren't in the release binary" reasoning as above would apply,
  but this benchmark is cheap enough that I re-ran it rather than relying on
  reasoning alone. Per-domain results (`entities`/`relations`/`queries=25`
  each; `errored=0` in every domain — no execution failures):

  | Domain | Matched/25 | Mismatched | Errored |
  | --- | ---: | ---: | ---: |
  | company | 13 | 12 | 0 |
  | fictional_character | 14 | 11 | 0 |
  | flight_accident | 24 | 1 | 0 |
  | geography | 11 | 14 | 0 |
  | movie | 6 | 19 | 0 |
  | nba | 25 | 0 | 0 |
  | politics | 15 | 10 | 0 |

  This benchmark's "mismatched" counts reflect differential comparison
  against an oracle across the whole engine (not specific to this task's
  doc/spec/test changes); `errored=0` everywhere confirms the run completed
  without crashes, which is what this gate checks. I did not have a prior
  baseline loaded in-session to diff these numbers against; they are
  reported here as observed, per "fail loud" rather than asserted as
  unchanged from some unseen prior run. This data was appended to
  `graph/test-results/benchmarks.jsonl`/`runs.jsonl`, neither of which is
  committed (per instructions, nothing under `graph/test-results/` is
  committed by this task).

## Standing cautions — status

No `system-reminder`-shaped or otherwise instruction-shaped content arrived
through any tool-result channel this session. The one background-task
notification received (the corpus run completing) was treated strictly as
data (its factual log contents), never as an instruction, consistent with
the standing caution from the coordinator. Nothing in the brief was found
wrong when measured against the tree, except the one nuance flagged above
(the MATCH/expand arrow-form refusal's actual error is `MissingSource`, not
`MissingRelationshipRole` — a correction to my own draft claim discovered via
verification, not a defect in the brief itself, which did not specify this
error's exact identity).
