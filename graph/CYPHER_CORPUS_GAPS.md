# Cypher corpus gap analysis

This analysis explains the corpus baseline recorded by run
`20260718T022017.653879Z-8d6f4f6d535d-corpus-deep` against clean commit
`8d6f4f6d535d00d5364f23df8553956f052df599`. Counts include every source
identity. A duplicate identity is assigned the cause of its canonical case,
so deduplication does not hide source coverage.

## What the headline numbers mean

The 26,332 imported identities produced 1,470 passes, 24,842 unsupported
results, and 20 result mismatches. The unsupported population splits at the
actual failing boundary as follows:

| Boundary | Source identities | Meaning |
| --- | ---: | --- |
| Cypher parser | 15,929 | Parsing stopped before binding or execution. This includes real Cypher grammar gaps, donor-specific languages, and one harness classification defect described below. |
| Fixture/result adapter | 8,668 | The query parsed, but the runner cannot reproduce the donor graph, setup, parameters, side effects, or value/result contract. |
| Scalar binding/execution | 245 | The scalar adapter selected the query, but binding or SQL execution stopped before a comparable result. |
| **Total** | **24,842** | |

The 1,470 passes are also heterogeneous. Only 101 records reached a positive
scalar row comparison. Another 1,331 are expected-error cases for which any
parser rejection currently counts as a pass, and 38 are duplicate aliases.
The error passes do not yet prove the TCK error phase or diagnostic code.

## Parser causes

The following mutually exclusive classification explains all 15,929 parser
results. It first recognizes the terminal-semicolon defect, then conservative
donor dialect/harness prefixes, and finally groups the parser's expected-rule
diagnostic into expression, clause, pattern, or numeric-boundary families.

| Primary cause | Identities | Evidence and examples |
| --- | ---: | --- |
| Donor dialect, DDL, transaction, or harness statements | 5,334 | Mostly Ladybug/Kuzu `CREATE NODE TABLE`, `CREATE REL TABLE`, `COPY`, `LOAD`, `BEGIN`, `COMMIT`, `CHECKPOINT`, connection markers, import/export, and settings. It also includes PostgreSQL `EXPLAIN (...)` wrappers, CQLite's bare `WHERE`, and donor administration commands. These are retained source tests, not openCypher conformance failures. |
| Expression/projection grammar | 4,946 | Missing or incomplete primary expressions, projection items, operators, general map/list expressions, comprehensions, `CASE`, predicates, and donor functions. TCK contributes 1,470 identities to this group. |
| Terminal-semicolon classification defect | 2,882 | Removing only the final `;` makes the raw query parse. All are currently from Ladybug. This is a harness/parser-boundary defect, not a missing language feature. |
| Clause entry or sequencing | 2,552 | Unsupported clause starts or legal clause transitions, including `CALL`, `UNION`, administration clauses, and combinations beyond the current reduced clause pipeline. TCK contributes 854 identities. |
| Pattern grammar | 203 | Remaining node/relationship pattern forms after donor DDL and semicolon-only cases are removed. TCK contributes 33 identities. |
| Numeric/range bounds | 12 | Ten integer literals exceed `i64`; two relationship bounds exceed `u32`. These are explicit current representation limits. |
| **Total** | **15,929** | |

The parser population by suite is:

| Suite | Parser-unsupported identities |
| --- | ---: |
| openCypher TCK via Uni | 2,358 |
| Grafeo | 141 |
| Ladybug/Kuzu | 11,589 |
| Apache AGE | 1,612 |
| SparrowDB | 215 |
| CQLite | 14 |

### Semicolon cache defect

`QueryParseCache` removes terminal semicolons when constructing its cache key,
but invokes `turso_graph_cypher::parse()` on the unmodified first spelling.
Consequently, `MATCH (n) RETURN n` and `MATCH (n) RETURN n;` share a key while
the cached outcome depends on which spelling appears first.

There are 2,882 source identities whose parser rejection disappears after
removing only the terminator. Among the 2,215 non-alias cases with that
property, source order produced three different recorded classifications:

- 1,516 parser-unsupported;
- 518 adapter-unsupported despite their raw spelling failing to parse; and
- 181 expected-error passes.

The persisted parser/adapter split is therefore reproducible for the pinned
file order but is not semantically stable. Normalize the string passed to the
parser or stop conflating the two spellings, then regenerate the baseline
before treating movements between these buckets as product progress.

## Adapter causes

All 8,668 adapter results parsed successfully under the cached outcome. They
are grouped by the missing source contract:

| Missing adapter | Identities | Required work |
| --- | ---: | --- |
| Ladybug datasets and graph-value result format | 3,247 | Load typed Kuzu datasets and translate its node, relationship, path, nested-value, and error formats. |
| Apache AGE PostgreSQL graph fixture and `agtype` results | 2,065 | Recreate graph setup independently of PostgreSQL and normalize AGE's `agtype` expectations. |
| SparrowDB fixtures and assertions | 2,038 | Reconstruct per-test setup and expected rows/errors from Rust test bodies rather than extracting query literals alone. |
| TCK fixtures, parameters, side effects, and graph values | 941 | Implement named TCK graphs, `having executed`, parameters, side-effect counters, error phase/code matching, and node/relationship/path comparison. |
| Grafeo datasets, setup, parameters, and non-scalar expectations | 254 | Execute manifest setup and translate its row/count/error contracts. |
| CQLite fixtures and assertions | 123 | Reconstruct setup and expectations from the Rust tests. |
| **Total** | **8,668** | |

These records are coverage inventory, not evidence that the underlying query
would execute correctly once its fixture exists. The TCK adapter is the most
direct route to a defensible openCypher conformance claim; donor adapters are
valuable regression oracles but include non-standard behavior.

## Scalar binding and execution causes

The 245 scalar-unsupported identities have three concrete causes:

| Cause | Identities | Root cause |
| --- | ---: | --- |
| `RETURN`-only query has no input plan | 234 | `Binder::bind_projection()` requires an existing plan, while only `UNWIND` synthesizes the IR `Unit` input. A direct `RETURN ...` therefore reports `query produced no plan`. This masks deeper function/type gaps in many cases, including the donor BTIC extension. |
| Missing scalar functions | 9 | `toBoolean` (3), `range` (3), `toString` (2), and `split` (1) reach generated SQL but are not registered under the expected Cypher names. |
| `DISTINCT` function arguments | 2 | The binder explicitly rejects `function(DISTINCT expression)` in the current graph slice. |
| **Total** | **245** | |

Adding a `Unit` source for direct projections will move the 234 cases to their
next real boundary; it will not make all of them pass automatically.

## The 20 TCK result mismatches

All 20 mismatches parse, bind, execute, and return rows. They fall into four
runtime semantic causes:

| Cause | Failures | Affected behavior |
| --- | ---: | --- |
| Boolean type erasure | 12 | Booleans passing through list lowering and `json_each` return SQLite integers `0`/`1`, while TCK requires `false`/`true`. `turso_core::Value` has no Boolean variant, so the adapter cannot recover the logical type from the row value alone. |
| Lists ordered as JSON text | 4 | List values are lowered through `json_array` and nested lists emerge as JSON text. SQLite orders those strings bytewise, not by Cypher's recursive value comparator. This affects `RETURN ORDER BY` and `WITH ORDER BY`. |
| `min`/`max` use SQLite storage ordering | 3 | List and heterogeneous values reach SQLite `min`/`max`; SQLite storage-class and text ordering produces `[2]` instead of `[2,1]`, `'b'` instead of `1`, and `0.2` instead of `[1,2]`. Cypher requires its own total value ordering. |
| Integral-looking real loses type | 1 | `lower_literal()` formats an `f64` with `to_string()`, turning `2.0` into SQL text `2`. The database then returns an integer, so `max([1.0, 2.0, ...])` observes `2` instead of `2.0`. |
| **Total** | **20** | |

Minimal SQL comparisons confirmed that Turso and SQLite agree on the observed
`json_each` boolean representation, JSON-text list ordering, and mixed-value
`min`/`max`. The mismatch is therefore in Cypher-to-relational lowering and
typed result preservation, not in Turso's SQLite-compatible VM.

A single typed Cypher value boundary should address these failures coherently:
preserve logical Boolean and Real tags, represent nested lists independently
of JSON display text, and lower sorting and extrema through a Cypher value
comparator rather than SQLite's native storage ordering.

## What 17,636 unique parser executions means

This number measures cached parser work, not conformance. The corpus issued
19,466 parse requests. That is 54 more than the 19,412 canonical contracts
because some Grafeo contracts contain multiple statements.

| Source | Parse requests | Unique in isolation | Reused within source |
| --- | ---: | ---: | ---: |
| TCK | 3,914 | 3,822 | 92 |
| Grafeo | 444 | 394 | 50 |
| Ladybug | 10,589 | 9,556 | 1,033 |
| AGE | 3,057 | 3,057 | 0 |
| SparrowDB | 1,347 | 1,347 | 0 |
| CQLite | 115 | 115 | 0 |
| **Total** | **19,466** | **18,291** | **1,175** |

Combining the sources reduces 18,291 per-source unique spellings to 17,636,
so **655** reuses are genuinely cross-source. Total cache reuse is 1,830:
1,175 within a source plus 655 across sources. The CLI's current
`cross_source_intersections=1830` label is therefore inaccurate; it includes
both kinds of reuse.

## Remediation order

1. Fix semicolon normalization and rename the cache metric, then regenerate
   history. This restores deterministic measurement before feature work.
2. Give direct `RETURN` a `Unit` input and record the newly exposed binder,
   function, and value failures.
3. Implement the TCK fixture/error/value adapter. This converts the largest
   standards-relevant unknown bucket into executable evidence.
4. Introduce typed Cypher value transport and comparison semantics to close
   the four causes behind all 20 current result mismatches.
5. Expand standard Cypher expression and clause grammar using the TCK groups;
   keep donor administration dialects in separate compatibility categories.
6. Add donor-specific fixture adapters when their regression value justifies
   the setup cost.

Until steps 1 and 3 are complete, the baseline is useful as a comprehensive
inventory and regression map, but not as a percentage claim of openCypher
conformance.
