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
(`20260722T204051.387397Z-0de15cc74e02-corpus-deep`), classified outcome
over all 10,242 identities:

- **8,919 passed**;
- **53 unsupported** vendor-specific behaviors; and
- **1,270 failed** with a non-empty reason.

The dominant failure families from that run (full histogram in
[`test-results/REPORT.md`](test-results/REPORT.md)):

| Failure family | Identities |
| --- | ---: |
| `execution`: other | 492 |
| `execution`: mutation projection unsupported | 248 |
| `execution`: runtime scalar function missing | 187 |
| `parser`: other grammar | 112 |
| `parser`: expression/operator continuation grammar | 43 |
| remaining failure families | 188 |
| **Failed total** | **1,270** |

This is broad regression and parser-compatibility coverage, not a claim of
full openCypher conformance. The runner attempts parsed read and mutation
statements, TCK setup queries, scalar parameters, pinned named-graph fixtures,
Grafeo named datasets, and multi-statement cases. Missing grammar or semantics
therefore appear as failed tests with their parser, setup, execution, or
comparison reason. The unsupported category is reserved for explicitly
classified vendor-only behavior and remains distinct from a failed portable
contract.

See [`CYPHER_CORPUS_GAPS.md`](archive/CYPHER_CORPUS_GAPS.md) and
[`LONG_TAIL.md`](archive/LONG_TAIL.md) for dated gap-analysis snapshots and
[`CYPHER_PARSER_GAP_HISTOGRAMS.md`](archive/CYPHER_PARSER_GAP_HISTOGRAMS.md)
for provenance and quality-impact histograms after the source removal.

## Semantic profile

Cypher leaves row order, duplicate survival, NULL comparison, NULL sort rank,
and label-list order undefined. Turso answers each one, and those answers
decide pass/fail verdicts, so they are versioned data in
[`ir/src/semantics.rs`](ir/src/semantics.rs) rather than prose here. The
current answers:

| Open choice | Turso's answer |
| --- | --- |
| Row order | Defined only under an explicit outermost `ORDER BY` |
| Duplicates | Retained unless `DISTINCT` is written |
| Comparison against `NULL` | Three-valued: yields `NULL`, never false |
| `NULL` sort rank (ascending) | numbers, text, blobs, then `NULL` last |
| `labels(n)` order | Label-table insertion order |
| Write classification | Syntactic: a `DELETE` matching nothing is a write |

Every recorded run stamps `semantics_version` (history schema version 2), and
`REPORT.md` prints it. A moved pass count is therefore attributable: the code
changed, or the rules changed, never ambiguously both. Rows written before the
profile existed report `0`, meaning their rules are unknown. Changing any
answer above requires bumping `SEMANTIC_PROFILE_VERSION`; the pin test in
`ir/tests/semantic_profile_pin.rs` fails on an unversioned edit.

## Running it

```sh
cargo run -q -p turso_graph_testkit -- corpus-stats
cargo run -q -p turso_graph_testkit -- corpus --no-record
```

The second command returns failure while any conformance contract fails. Omit
`--no-record` only for an intentional append-only baseline run.

Recorded baseline runs go through `mise run corpus`, which pins `--release`:
history timings are only comparable against a release build. Each row records
the profile it was actually built with, so a debug run cannot silently mix in.

### Pruning history

`history.jsonl` is gitignored and local-only, but both `append` and `report`
read the whole file, so an unbounded history slows every recorded run. The
report itself only reads the newest run of each suite and the one before it.

```sh
cargo run --release -p turso_graph_testkit -- prune-history --keep 5
```

This writes `history.jsonl.pruned` and leaves the source untouched: pruning
never destroys, because these rows cannot be regenerated. Archive before
swapping, and confirm the report is unchanged:

```sh
gzip -6 -c history.jsonl > history-archive-$(date +%Y%m%d).jsonl.gz
gzip -t history-archive-*.jsonl.gz
mv history.jsonl.pruned history.jsonl
```

Retention is floored at 2 runs per suite, and counted per suite so a suite that
runs often cannot evict one that runs rarely.
