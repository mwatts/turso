# Turso graph Cypher conformance

The corpus-scale baseline imports **10,392 source test identities** from five
pinned suites. Every retained identity must exercise openCypher-compatible
syntax or a deliberately tracked graph extension. Vendor suites that mix
database-specific language, harness commands, and result contracts into the
query stream are excluded rather than counted as Cypher failures.

## Corpus inventory

| Source | Imported identities | Canonical within source | Exact duplicates |
| --- | ---: | ---: | ---: |
| openCypher TCK M23 (through Uni) | 3,926 | 3,914 | 12 |
| Grafeo `.gtest` suite | 399 | 390 | 9 |
| Apache AGE regression SQL | 3,677 | 3,057 | 620 |
| SparrowDB Rust tests | 2,253 | 1,347 | 906 |
| CQLite Rust tests | 137 | 115 | 22 |
| **Total** | **10,392** | **8,823** | **1,569** |

The LadybugDB/Kuzu suite and its four curated fixture adaptations were removed
because the source mixes vendor-specific DDL, transaction and administration
commands, functions, types, test-harness markers, datasets, and result formats
with standard-looking Cypher. Retaining only apparently portable statements
would still allow vendor semantics to leak into Turso's conformance contract.

## Current result

A strict binary-outcome corpus run classified all 10,392 identities:

- 1,413 passed;
- 8,979 failed with a non-empty reason; and
- 0 were skipped, unsupported, or satisfied through a canonical-result alias.

Every source identity runs independently, including the 1,569 identities whose
contracts exactly duplicate another source identity. The run made 10,446 parse
requests against 8,706 unique normalized queries, with 1,740 cache
intersections. Failures by their recorded boundary were:

| Boundary | Identities |
| --- | ---: |
| Parser | 3,698 |
| Query or mutation execution | 4,602 |
| Scenario setup execution | 637 |
| Named donor dataset execution | 16 |
| Named TCK fixture execution | 1 |
| Parameter binding | 3 |
| Scalar or graph result comparison | 14 |
| Side-effect comparison | 8 |
| **Total** | **8,979** |

This is broad regression and parser-compatibility coverage, not a claim of
full openCypher conformance. The runner attempts parsed read and mutation
statements, TCK setup queries, scalar parameters, pinned named-graph fixtures,
Grafeo named datasets, and multi-statement cases. Missing grammar or semantics
therefore appear as failed tests with their parser, setup, execution, or
comparison reason rather than as an accepted coverage category.

See [`CYPHER_CORPUS_GAPS.md`](CYPHER_CORPUS_GAPS.md) for the remaining gap
analysis and [`CYPHER_PARSER_GAP_HISTOGRAMS.md`](CYPHER_PARSER_GAP_HISTOGRAMS.md)
for provenance and quality-impact histograms after the source removal.

## Running it

```sh
cargo run -q -p turso_graph_testkit -- corpus-stats
cargo run -q -p turso_graph_testkit -- corpus --no-record
```

The second command returns failure while any conformance contract fails. Omit
`--no-record` only for an intentional append-only baseline run.
