# Cypher corpus gap analysis

> **Historical snapshot.** The numbers below describe the corpus run made
> shortly after the LadybugDB/Kuzu removal (10,392 identities: 1,413 passed,
> 8,979 failed). The corpus and pass rate have moved substantially since —
> the latest recorded run covers 10,242 identities with 8,800 passing. For
> current results always use
> [`test-results/REPORT.md`](test-results/REPORT.md); the family-level
> analysis below is retained for its qualitative triage, not its counts.

This analysis describes the five-source corpus after removing the complete
LadybugDB/Kuzu donor suite and its curated fixture adaptations. At the time
of this snapshot the corpus run contained 10,392 independently executed
identities: 1,413 passed and 8,979 failed with a recorded reason. No identity
is skipped, classified as unsupported, or satisfied by copying another
identity's result.

## Failure boundaries

| Boundary | Source identities | Meaning |
| --- | ---: | --- |
| Parser | 3,698 | The identity ran and parsing failed. |
| Query or mutation execution | 4,602 | Parsing succeeded and binding, lowering, or execution failed. |
| Scenario setup execution | 637 | The test's own setup statement ran and failed. |
| Named donor dataset execution | 16 | The pinned dataset setup ran and failed. |
| Named TCK fixture execution | 1 | The pinned named-graph setup ran and failed. |
| Parameter binding | 3 | The test parameter cannot yet be represented by the frontend value boundary. |
| Result comparison | 14 | Execution completed but the graph/scalar result contract is not representable or did not match. |
| Side-effect comparison | 8 | The mutation ran, but complete TCK side-effect accounting is not implemented. |
| **Total** | **8,979** | |

These numbers are source identities, not distinct implementation tasks.
Normalized queries share cached parse results, but every identity initializes
and runs its own fixture and execution path. Exact duplicates remain inventory
metadata only; they no longer bypass execution.

## Remaining source-specific language

The prior donor-language bucket was dominated by a suite that mixed its own
database language and harness controls into query tests. That suite has been
removed rather than filtered heuristically. The remaining known
source-specific population is 105 identities:

| Source-specific family | Identities | Treatment |
| --- | ---: | --- |
| AGE `EXPLAIN (...)` directives | 60 | Strip when measuring the underlying Cypher query; track AGE plan explanation separately. |
| Grafeo graph index/constraint DDL | 22 | Exclude from core openCypher scoring unless Turso adopts an explicit schema contract. |
| SparrowDB checkpoint and schema administration | 19 | Keep outside the query-language denominator. |
| CQLite bare query fragments | 4 | Repair extraction or exclude the incomplete fragment. |
| **Total** | **105** | |

Standard-shaped `CALL` and `YIELD` queries remain genuine clause grammar gaps
even when their procedure names originate in a donor. They are not classified
as source-specific syntax.

## Expression and projection grammar

After removing the vendor-mixed suite, 3,031 identities remain in the
expression/projection backlog:

| Source | Identities | Quality impact |
| --- | ---: | --- |
| openCypher TCK | 1,470 | Normative and highest priority. |
| Apache AGE | 1,319 | Useful Cypher pressure, but PostgreSQL casts, `agtype`, and AGE functions require an explicit compatibility decision. |
| SparrowDB | 154 | Secondary regression oracle, not a specification. |
| Grafeo | 87 | Independent interoperability evidence where behavior agrees with the TCK. |
| CQLite | 1 | Low impact by itself. |
| **Total** | **3,031** | |

The dominant semantic families remain list predicates, maps and lists,
projection continuation, wildcard aggregate arguments, comprehensions,
`CASE`, existential/count subqueries, null and string predicates, and
`reduce`. These are not parser-only changes: most require corresponding scope,
typing, lowering, execution, and result-comparison work.

Use the 1,470 TCK identities as the normative queue. Donor cases should add
independent regression pressure only after their syntax and expected semantics
are shown to match that contract.

## Scalar execution gaps

Direct `RETURN` and `WITH` projections now use the existing single-row IR
`Unit` input when no preceding clause supplies rows. That change moved 271
source identities to pass; other standalone projections now expose their next
real parameter, function, expression, or result boundary instead of failing
with `query produced no plan`. Missing Cypher function aliases and `DISTINCT`
function arguments remain separate execution and grammar groups.

## The 20 TCK result mismatches

All 20 mismatches parse, bind, execute, and return rows:

| Cause | Failures | Affected behavior |
| --- | ---: | --- |
| Boolean type erasure | 12 | Boolean values emerge from relational/list lowering as integers `0` and `1`. |
| Lists ordered as JSON text | 4 | Nested list ordering uses serialized text instead of Cypher's recursive value comparator. |
| `min`/`max` use SQLite storage ordering | 3 | Heterogeneous and list extrema follow SQLite storage-class ordering. |
| Integral-looking real loses type | 1 | Lowering formats `2.0` as SQL text `2`, losing the real tag. |
| **Total** | **20** | |

The mismatch is in Cypher lowering and typed result preservation, not in the
SQLite-compatible VM. A typed Cypher value boundary should preserve Boolean
and Real tags, represent nested lists independently of display JSON, and use a
Cypher comparator for sorting and extrema.

## Semicolon cache correction

`QueryParseCache` now removes one terminal semicolon from the spelling passed
to the parser as well as from cache identity. The result no longer depends on
whether the terminated or unterminated spelling is encountered first. The
removed donor suite contained the historical semicolon-sensitive population,
so it no longer contributes any corpus identities.
