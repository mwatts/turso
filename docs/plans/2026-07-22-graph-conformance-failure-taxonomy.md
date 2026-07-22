# Graph conformance failure taxonomy (2026-07-22)

## Scope and source of truth

This analysis covers the latest complete corpus run in
`graph/test-results/REPORT.md`:

- run `20260722T204051.387397Z-0de15cc74e02-corpus-deep`;
- 10,242 records, 8,919 passed, 53 expected-unsupported, and 1,270 failed;
- specifically the 174 failures at the `parser` boundary and the 1,014 at the
  `execution` boundary.

The counts below are deterministic from the report and its local source
artifact, `graph/test-results/history.jsonl`. The JSONL is not present in the
isolated documentation worktree, but the run-producing main worktree retains
all 10,242 latest-run records. Every failure has a message, source provenance,
and execution-boundary dimension. Streaming its final 10,242 lines makes an
exact per-record classification possible without loading or rescanning its
full 1.2 GB history.

`REPORT.md` itself retains individual failure messages only for suites with
more than 500 records. That presentation rule hides the messages for CQLite's
11 failures and Grafeo's 96 failures, but it is a report-generation limitation,
not a history-schema or data-retention limitation.

## Exact boundary reconciliation

| Boundary | AGE | TCK | SparrowDB | Grafeo | CQLite | Total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `execution` | 423 | 484 | 55 | 41 | 11 | **1,014** |
| `parser` | 83 | 66 | 6 | 19 | 0 | **174** |
| `setup-execution` | 0 | 4 | 0 | 32 | 0 | 36 |
| `fixture-execution` | 0 | 19 | 0 | 0 | 0 | 19 |
| `side-effect-comparison` | 0 | 23 | 0 | 0 | 0 | 23 |
| `dataset-execution` | 0 | 0 | 0 | 4 | 0 | 4 |
| **All failed records** | **506** | **596** | **61** | **96** | **11** | **1,270** |

The 53 `Unsupported` records are not failures in these totals. They are policy
classifications and should remain separate from feature defects.

## Parser failures: 174

### Generated families

| Parser family | Exact total |
| --- | ---: |
| Other grammar | **112** |
| Expression/operator continuation grammar | **43** |
| Graph-pattern grammar | **12** |
| Map-literal grammar | **5** |
| Projection/expression item grammar | **2** |
| **Total** | **174** |

The family labels come from string matching in
`graph/testkit/src/report.rs::failure_family`, not from parser rule identity.
They are useful prioritization buckets, but they are not a semantic root-cause
taxonomy. In particular, a Pest error listing `comparison_op` is classified as
expression continuation even when the source construct is a map projection or
pattern expression.

### Why parsing still fails

The grammar in `graph/cypher/src/cypher.pest` is intentionally described as a
reduced initial slice. The remaining failures are concentrated at five missing
or overly narrow syntax seams:

1. **Clause termination and continuation (43 classified, plus part of the 112
   “other” records).** The grammar reaches a valid expression, then cannot
   consume the next source token. Representative failures include
   `age.cypher.subquery.query-9` (`expected ORDER, SKIP, LIMIT, AS, ...`) and
   `tck.expressions.list.list6.scenario-7` (`expected AND, OR,
   relationship_pattern, ...`). This includes unsupported postfix constructs,
   pattern expressions, and map projections rather than one missing operator.

2. **Reserved words and identifier positions.** Of all 174 parser records, 31
   normalize to `expected identifier`. The grammar globally
   excludes every token in `keyword` from `identifier`, while only aliases use
   the more permissive `alias_identifier`. Examples include
   `age.cypher.create.query-74` and
   `sparrowdb.spa-265-backtick-escaping.bare-keyword-label-order.query-1`.
   The fix seam is contextual identifiers in `cypher.pest`, not the binder.

3. **Clause/query shape.** Thirty-two records normalize to `expected
   EOI, UNION, or clause`; another six expect `EOI, WHERE, UNION, clause, or
   relationship_pattern`. Examples are
   `tck.clauses.call.call5.scenario-4.examples-1-row-1` and
   `tck.clauses.match.match3.scenario-19`. These expose unsupported clause
   continuations or pattern forms after a syntactically complete prefix.

4. **Mutation targets and primary expressions.** Fourteen records expect
   `property_target`, seven expect `primary_expression`, and five reject a CALL
   target that is not a plain/namespaced name. The current `property_target`
   rule is only `identifier "." identifier`; the expression and CALL grammars
   similarly encode a deliberately small surface. Representative records are
   `tck.clauses.set.set2.scenario-4` and `age.expr.query-732`.

5. **Pattern/map syntax.** The exact generated totals are 12 graph-pattern and
   five map-literal failures. `age.cypher.match.query-397` expects
   `relationship_types, range_literal, or map_literal`; the TCK Pattern2
   scenarios stop at `relationship_pattern`. These should be split by source
   query before changing the grammar because the current error-family matcher
   conflates map projections, map literals, and pattern continuations.

There are also small validation edges (three out-of-`u32` relationship ranges,
two out-of-`i64` literals, and three unsupported string escapes). Those are
parser-boundary failures by design but are not missing grammar
and should not drive grammar expansion.

### Parser conclusion

Parsing is no longer failing because of one broad parser defect. The largest
tractable seam is contextual continuation syntax, but the 112 “other” records
must first be clustered against the original query text. The history artifact
has all messages, IDs, and source provenance, but query text is not a dedicated
field and is embedded in messages only by some adapters. AGE and SparrowDB
queries must be recovered through their source references. A grammar patch
based only on expected-token strings risks accepting the wrong construct.

## Execution failures: 1,014

### Generated families

| Execution family | Exact total |
| --- | ---: |
| Other | **492** |
| Mutation projection unsupported | **248** |
| Runtime scalar function missing | **187** |
| Expected-error mismatch | **38** |
| Mutation operation unsupported | **31** |
| Parameter binding/declaration | **18** |
| **Total** | **1,014** |

These exact categories inspect the entire compound error message. They do not
necessarily identify the primary failure. The AGE/TCK/Sparrow adapters first
try `GraphConnection::query`; after any error they try
`GraphConnection::execute` and join both errors into one message
(`graph/testkit/src/age.rs` and `graph/testkit/src/tck.rs`). As a result, a read
query with a missing function can also carry a mutation-projection error.

### Primary-cause view of all 1,014 records

For the table below, “primary” means the query-path error before
`; mutation execution failed:`. Direct result and expected-error mismatches are
kept as their own causes. These categories are mutually exclusive and sum to
all 1,014 execution records.

| Primary cause | Records | Representative record |
| --- | ---: | --- |
| Result/value/row semantic mismatch | 291 | `tck.clauses.delete.delete1.scenario-5` |
| Missing scalar or graph function | 185 | `age.age.shortest.path.query-67` |
| Expected-error mismatch | 105 | `age.age.shortest.path.query-25` |
| Expression type/operand limitation | 105 | `age.age.global.graph.query-51` |
| Function validation/runtime error | 72 | `age.agtype.query-9` |
| Binding/scope/parameter resolution | 74 | `age.age.reduce.query-39` |
| Procedure registry gap | 60 | `age.cypher.call.query-3` |
| Query-shape/traversal limitation | 50 | `age.cypher.match.query-171` |
| Mutation routed through read binder | 37 | `age.cypher.create.query-36` |
| Invalid generated relational SQL | 22 | `age.cypher.match.query-136` |
| Snapshot lifecycle/precondition | 9 | `tck.clauses.match-where.matchwhere4.scenario-2` |
| Other execution semantics | 4 | `age.expr.query-750` |
| **Execution total** | **1,014** | |

This view explains why “execution” remains large: 396 records are
successful compilation followed by wrong results or wrong error acceptance
(291 + 105), while the rest span binder capability, function/procedure
dispatch, lowering, and fixture state. Execution is a boundary, not a single
implementation layer.

The 185 primary missing-function records include repeated families with clear
product decisions: `start_id` (11), `label` (10), `full_text_search` (8),
`btic` (8), `percentileCont` (8), `end_id` (7), `percentileDisc` (7),
`startNode` (5), `endNode` (5), plus shortest-path, vector, temporal, and AGE
vendor-internal names. The generated 187 count is two higher because it scans
the whole compound message and counts function errors found only on the
mutation fallback path.

The 492 generated “other” records are therefore a classifier catch-all, not an
evidence gap. Every record has a history message. Their dominant primary
subfamilies are result mismatches, expected-error mismatches, procedure
registry gaps, expression typing, scope/binding, and query-shape restrictions.

### Mutation fallback noise

There are 618 execution records with both query and mutation
diagnostics. Their secondary mutation failures are:

| Secondary mutation diagnostic | Records |
| --- | ---: |
| Projection unsupported | 434 |
| Procedure unsupported | 60 |
| Duplicate/unknown binding | 44 |
| Other mutation limitation | 28 |
| UNION unsupported | 14 |
| Property access/type | 10 |
| Already-bound CREATE semantics | 8 |
| Mutation SQL execution | 3 |
| SET whole-entity semantics | 3 |
| Variable-length relationship properties | 3 |
| Multiple OPTIONAL MATCH | 2 |
| Named path after mutation | 2 |
| Relationship creation type | 2 |
| CALL subquery after another clause | 5 |
| **Dual-diagnostic records** | **618** |

The 434 fallback projection messages must not be added to the exact 248
generated “mutation projection unsupported” family. They overlap heavily with
missing functions, procedure gaps, and binder errors. Supporting mutation
projections is still a high-yield feature, but 434 is an upper bound on records
that encounter that secondary seam, not an expected pass gain. The runner
should route statements from their parsed clause kind instead of treating every
query error as evidence that the statement may be a mutation.

### Root seams

- **Grammar:** `graph/cypher/src/cypher.pest` and
  `graph/cypher/src/parser.rs`.
- **Read binding, expression typing, procedures, and scope:**
  `graph/frontend/src/binder.rs`.
- **Mutation stages/projections:** `graph/frontend/src/mutation.rs` and the
  mutation path in `binder.rs`.
- **Function signatures and core/dialect dispatch:**
  `graph/frontend/src/functions.rs` and `graph/frontend/src/dialect.rs`.
- **Relational SQL generation:** `graph/frontend/src/lowering.rs`.
- **Snapshot preconditions:** `graph/frontend/src/snapshot.rs` and
  `graph/frontend/src/session.rs`.
- **Outcome routing and measurement:** `graph/testkit/src/age.rs`,
  `graph/testkit/src/tck.rs`, `graph/testkit/src/grafeo.rs`,
  `graph/testkit/src/rust_donor.rs`, and `graph/testkit/src/report.rs`.

## Ranked next actions

| Rank | Action | Expected corpus leverage | Cost/risk |
| ---: | --- | --- | --- |
| 1 | Preserve per-failure boundary, primary error, fallback error, and query/source reference in the generated report for every suite; classify only the primary error. | No direct passes, but removes the report's 107-record presentation blind spot and false overlap that can mis-rank every later change. | Low; testkit/report-only. |
| 2 | Implement mutation-stage `WITH`/`RETURN` projection support and route parsed mutation statements directly to `execute`. | Exact current family: 248; secondary projection encountered by up to 434 records, with overlap. | Medium; binder/mutation semantics and end-to-end side effects require care. |
| 3 | Split missing functions into portable, Turso-native, and vendor-internal registries; close portable scalar gaps and `startNode`/`endNode` first. | Exact family: 187; 185 primary records. Small functions can close clusters cheaply. | Low to medium per function; shortest-path and vendor internals are separate policy/features. |
| 4 | Cluster the 291 result mismatches by normalized expected/observed shape and source feature before changing execution. | Largest primary family; likely several high-yield semantic fixes. | Low analysis cost, variable implementation cost. |
| 5 | Add a real procedure registry, then implement portable catalog/full-text procedures. | 60 primary procedure failures, plus product-value beyond corpus points. | Medium; registry first prevents hard-coded CALL growth. |
| 6 | Fix contextual identifiers and one parser continuation family at a time, with source-query fixtures. | Parser exact totals: 43 continuation and 112 other; detailed repeated forms show leverage but not one feature. | Medium; Pest changes can broaden ambiguity. |
| 7 | Isolate generated-SQL failures with golden SQL and SQLite parse tests. | 22 records. | Low to medium; lowering-only if the bound IR is correct. |
| 8 | Defer shortest-path, broad variable-length traversal, and vendor-specific AGE temporal/catalog functions until the above clusters are measured. | Individually small visible clusters despite high implementation cost. | High. |

## Required runner instrumentation

The next recorded conformance run should retain structured fields rather than
requiring semicolon parsing:

- `primary_boundary` (`parser`, `binder`, `lowering`, `database-execution`, or
  comparison);
- `primary_error_code` and `primary_message`;
- `fallback_attempted`, `fallback_boundary`, and `fallback_message`;
- parsed statement kind/read-vs-mutation routing decision;
- source query text (or a stable source-file/line reference when redistribution
  policy forbids embedding it);
- expected and observed result-shape summaries for comparison failures.

`append_large_suite_summary` should not gate failure details on suite size.
Either always append failed records or emit a compact committed
machine-readable latest-run artifact alongside `REPORT.md`. The existing
history has enough information to recover every record, but not enough
structure to distinguish primary and fallback causes without parsing human
messages.

## Verification recipe

The numbers above were reproduced by streaming the final 10,242 JSON objects
from the local `graph/test-results/history.jsonl`, asserting that every object
has the named run ID, selecting `outcome == "failed"`, and grouping by
`dimensions.execution`. Primary-cause classification uses the text before
`; mutation execution failed:`; fallback classification uses the text after
that marker. The report's generated histogram remains the reconciliation check
for its legacy message-family matcher.

Before using this taxonomy against another branch, regenerate a complete corpus
run. The counts and overlaps are tied to the named run, not to current source
HEAD.
