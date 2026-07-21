# Turso graph Cypher conformance

Live results are published in
[`test-results/REPORT.md`](test-results/REPORT.md), regenerated from
`test-results/history.jsonl` on every recorded baseline run. That report is
the source of truth for current pass/fail state; this file summarizes the
corpus contract and is refreshed manually, so on any disagreement trust
REPORT.md.

The corpus-scale baseline imports **10,242 source test identities** from five
pinned suites (verify with `corpus-stats` below). Every retained identity must
exercise openCypher-compatible syntax or a deliberately tracked graph
extension. Vendor suites that mix database-specific language, harness
commands, and result contracts into the query stream are excluded rather than
counted as Cypher failures.

## Corpus inventory

| Source | Imported identities |
| --- | ---: |
| openCypher TCK M23 (through Uni) | 3,926 |
| Grafeo `.gtest` suite | 372 |
| Apache AGE regression SQL | 3,595 |
| SparrowDB Rust tests | 2,225 |
| CQLite Rust tests | 124 |
| **Total** | **10,242** |

Across sources, 8,707 identities carry a canonical contract and 1,535 are
exact duplicates of another identity. Every source identity still runs
independently; duplicates are inventory metadata only.

The LadybugDB/Kuzu suite and its four curated fixture adaptations were removed
because the source mixes vendor-specific DDL, transaction and administration
commands, functions, types, test-harness markers, datasets, and result formats
with standard-looking Cypher. Retaining only apparently portable statements
would still allow vendor semantics to leak into Turso's conformance contract.

## Current result

Latest recorded corpus run
(`20260721T115532.802656Z-b4d65fb5b95b-corpus-deep`), strict binary outcome
over all 10,242 identities:

- **8,800 passed**;
- **1,442 failed** with a non-empty reason; and
- 0 were skipped, unsupported, or satisfied through a canonical-result alias.

The dominant failure families from that run (full histogram in
[`test-results/REPORT.md`](test-results/REPORT.md)):

| Failure family | Identities |
| --- | ---: |
| `execution`: other | 552 |
| `execution`: runtime scalar function missing | 269 |
| `execution`: mutation projection unsupported | 261 |
| `parser`: other grammar | 112 |
| `parser`: expression/operator continuation grammar | 58 |
| remaining families | 190 |
| **Total** | **1,442** |

This is broad regression and parser-compatibility coverage, not a claim of
full openCypher conformance. The runner attempts parsed read and mutation
statements, TCK setup queries, scalar parameters, pinned named-graph fixtures,
Grafeo named datasets, and multi-statement cases. Missing grammar or semantics
therefore appear as failed tests with their parser, setup, execution, or
comparison reason rather than as an accepted coverage category.

See [`CYPHER_CORPUS_GAPS.md`](CYPHER_CORPUS_GAPS.md) and
[`LONG_TAIL.md`](LONG_TAIL.md) for dated gap-analysis snapshots and
[`CYPHER_PARSER_GAP_HISTOGRAMS.md`](CYPHER_PARSER_GAP_HISTOGRAMS.md)
for provenance and quality-impact histograms after the source removal.

## Running it

```sh
cargo run -q -p turso_graph_testkit -- corpus-stats
cargo run -q -p turso_graph_testkit -- corpus --no-record
```

The second command returns failure while any conformance contract fails. Omit
`--no-record` only for an intentional append-only baseline run.
