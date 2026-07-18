# Turso graph Cypher conformance

The corpus-scale baseline imports **26,332 source test identities** from six
pinned upstream suites. It does not curate a small representative slice.
Every discovered identity is retained in history; exact duplicate contracts
are aliases of one canonical execution, and normalized queries share one
parser result across sources.

## Corpus inventory

| Source | Imported identities | Canonical within source | Exact duplicates |
| --- | ---: | ---: | ---: |
| openCypher TCK M23 (through Uni) | 3,926 | 3,914 | 12 |
| Grafeo `.gtest` suite | 399 | 390 | 9 |
| LadybugDB/Kuzu `.test` suite | 15,940 | 10,589 | 5,351 |
| Apache AGE regression SQL | 3,677 | 3,057 | 620 |
| SparrowDB Rust tests | 2,253 | 1,347 | 906 |
| CQLite Rust tests | 137 | 115 | 22 |
| **Total** | **26,332** | **19,412** | **6,920** |

The full run made 19,466 canonical parse requests. Cross-source query
intersection reduced those to 17,636 unique parser executions, avoiding 1,830
additional duplicate parses beyond the 6,920 within-source duplicate
contracts.

## Current result

The 2026-07-17 corpus run classified all 26,332 identities:

- 1,470 passed;
- 24,842 are explicitly unsupported at a recorded parser, scalar-execution,
  or adapter boundary; and
- 20 openCypher TCK scalar scenarios failed their expected result contract.

The failures are concentrated in boolean value representation, heterogeneous
list ordering, floating-point preservation, and mixed-type aggregation. They
remain failures rather than being hidden as unsupported.

This is broad regression and parser-compatibility coverage, not a claim of
full openCypher conformance. The generic adapter currently executes scalar
`RETURN`/`UNWIND` TCK scenarios. Tests requiring named graphs, donor-specific
fixtures, setup/side-effect assertions, parameters, graph values, PostgreSQL
AGE wrappers, or donor engine APIs remain visible but unsupported until those
adapters exist.

## Running it

```sh
cargo run -q -p turso_graph_testkit -- corpus-stats
cargo run -q -p turso_graph_testkit -- corpus --no-record
```

The second command returns failure while any conformance contract fails. Omit
`--no-record` only for an intentional append-only baseline run.
