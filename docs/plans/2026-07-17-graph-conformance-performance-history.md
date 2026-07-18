# Graph conformance, regression, and performance history plan

Date: 2026-07-17

Branch: `feature/graph-frontend`

## Goal

Build a durable graph-validation program that measures Cypher compatibility,
guards fixed defects, provides a fast smoke gate and a deeper regression gate,
and records graph lifecycle performance over time. Every result must retain a
stable test identity, source provenance, repository revision, package version,
run time, suite, test type, status, and measured duration in append-only JSONL.

The program must distinguish four different claims:

1. a scenario is supported and returned the required result;
2. a scenario is intentionally unsupported and failed at the declared boundary;
3. a supported scenario regressed; or
4. a previously unsupported scenario began to work and needs reclassification.

An unsupported result is therefore visible evidence, not a skipped test.

## Sources and adaptation order

The pinned revisions and licenses in `graph/PROVENANCE.md` remain authoritative.
Fixture adaptations retain repository, revision, path, case, license, and
adaptation type.

1. **openCypher M23 through Uni** supplies the standards-oriented scenario
   vocabulary, ordered and unordered result rules, error phases, side effects,
   and named sample graphs.
2. **Grafeo** supplies compact setup/query/expectation cases that can be
   normalized with little harness-specific translation.
3. **Apache AGE** supplies deep path, mutation, subquery, transaction, and exact
   multiplicity regressions. PostgreSQL wrapper output and generated identities
   are normalized away.
4. **Ladybug/Kuzu** supplies recursive, shortest-path, transaction, exception,
   and larger sample-graph cases. Its typed schema and dataset commands are
   translated to Turso graph registration.
5. **SparrowDB** supplies issue-numbered bug regressions for nulls, labels,
   paths, mutations, and identity handling.
6. **CQLite** supplies small parser, matching, mutation, and transaction smoke
   cases.

## Repository layout

```text
graph/testkit/                         # runner and longitudinal reporter
graph/testdata/suites/                 # executable scenario manifests
  conformance.toml                     # standards-oriented supported/unsupported matrix
  portable.toml                        # additional portable donor cases
  regressions.toml                     # deep semantics and donor bug regressions
  performance.toml                     # lifecycle workloads and graph sizes
graph/test-results/history.jsonl       # append-only per-test and per-operation records
graph/test-results/REPORT.md            # generated current and longitudinal summary
```

## Stable identity and suite contract

Every executable case has a globally unique lowercase identifier composed of a
source namespace and durable case name, for example
`tck.with.with1.scenario-1` or `sparrow.spa-263.two-hop-null`. Renaming an ID
creates a new time series and therefore requires an explicit supersession
record rather than an in-place edit.

Each case declares:

- `kind`: `smoke`, `conformance`, `regression`, `bug-regression`, or
  `performance`;
- `area`: parser, binder, match, traversal, mutation, transaction, error,
  lifecycle, or another stable feature area;
- `expectation`: supported result, supported error, or known unsupported;
- ordered or multiset row comparison;
- graph fixture and setup actions;
- source and license provenance; and
- optional upstream issue or fixed Turso commit.

The smoke suite is a tagged subset of the same case definitions used by the
deep runner. It must not duplicate or weaken their assertions.

## JSONL history contract

`graph/test-results/history.jsonl` is append-only. One record is written for
each scenario or performance operation. Records include:

- schema version and globally unique run ID;
- UTC timestamp;
- Git commit, dirty flag, package version, build profile, OS, and architecture;
- suite, test ID, kind, area, graph fixture, source identity, and expectation;
- actual status, duration, and failure details;
- result cardinality and a canonical result digest for correctness cases; and
- operation, graph shape, scale, iterations, elapsed time, throughput, and
  validated entity counts for performance cases.

The reporter rejects duplicate `(run_id, test_id, operation)` records, malformed
identities, missing provenance, unknown statuses, and history whose schema is
newer than the reader. Generated reports show the latest run for each suite,
changes from the previous comparable run, newly supported or regressed cases,
performance deltas by operation and scale, and the total longitudinal
inventory.

## Performance workloads

The initial lifecycle measurements use a deterministic synthetic line graph.
Smoke covers 10 and 100 nodes; deep covers 100 and 1,000 nodes. The typed
performance manifest is the extension point for additional sizes and shapes;
larger ladders should only be added after setting an explicit runtime and
memory budget.

Each configured scale records:

1. row-at-a-time Cypher creation;
2. batched SQL bulk load;
3. snapshot load through bounded variable traversal;
4. warmed property-filtered query; and
5. detach delete and cleanup.

Every measurement validates resulting node and relationship counts. Timing an
operation whose postconditions are wrong is a failed correctness record, not a
performance sample.

Smoke performance runs use small scales and three measured query iterations.
Deep history runs use ten warmed query iterations and the complete configured
size ladder.
Machine-local results are reported as time series for that environment; they
are not presented as cross-machine capacity claims.

## Delivery phases and gates

### Phase 1: framework and migration

- Add `turso_graph_testkit` with `run`, `report`, and `verify-history` commands.
- Define typed manifests, identities, outcomes, JSONL records, and validation.
- Migrate the existing 18 mixed-source scenarios without losing coverage.
- Generate the existing compatibility report from runner results.

Gate: framework unit tests, migrated scenario parity, append/read/report round
trip, duplicate detection, zero discovery failure, formatting, and strict
Clippy.

### Phase 2: valuable corpus expansion

- Add a quick smoke tag to critical supported behaviors.
- Normalize high-value Uni, Grafeo, AGE, Ladybug, SparrowDB, and CQLite cases.
- Include positive results, precise errors, side effects, multiplicity,
  bug regressions, and unsupported feature identities.
- Reuse a named social graph fixture across the portable corpus.

Gate: every scenario executes or produces its declared unsupported result; no
silent skips; every donor and bug case is traceable.

### Phase 3: lifecycle performance history

- Add deterministic workload generation and operation-level correctness gates.
- Record smoke and deep lifecycle results in the shared JSONL contract.
- Generate longitudinal correctness and performance sections.

Gate: all configured operations and smoke scales recorded; deep sizes either
recorded or explicitly marked resource-exhausted with a typed reason.

### Phase 4: operational integration and completion audit

- Document local and CI commands.
- Keep the testkit in the workspace default members so normal CI discovers its
  integration gates, while append-only baseline recording remains an
  intentional maintainer operation.
- Run smoke, deep conformance/regression, and lifecycle performance suites.
- Persist the first baseline and audit every requirement in this plan.

The work is complete only when the runner, imported suites, history, generated
report, performance workloads, first recorded baseline, and verification gates
all exist and execute from a clean checkout.

## Execution status

Completed on 2026-07-17:

- 38 deep identities execute: 32 supported cases pass and 6 unsupported cases
  fail at their declared diagnostic boundary.
- The 13-case smoke subset passes without duplicating scenario definitions.
- Five lifecycle operations pass at two smoke and two deep scales with
  post-operation count and result validation outside the timed interval.
- The first four-suite baseline contains 71 schema-versioned JSONL records from
  clean commit `e1d73880b74901c879c5bcf4cc96b1006f2d16b5` and passes
  `verify-history`.
- Strict Clippy, all testkit tests, and all graph-frontend tests pass.
