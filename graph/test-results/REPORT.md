# Graph test history

Generated from `graph/test-results/history.jsonl`. Results are grouped by stable test identity; performance comparisons are meaningful only for matching environment and workload dimensions.

## Latest `deep` run

- Run: `20260718T013941.952713Z-e1d73880b749-deep`
- Commit: `e1d73880b74901c879c5bcf4cc96b1006f2d16b5` (dirty)
- Package: `0.7.0`
- Environment: `macos/aarch64` (`dev`)
- Records: 38
- Passed: 32
- Unsupported: 6
- Failed or changed: 0

| Test | Kind | Area | Outcome | Duration |
|---|---|---|---|---:|
| `tck.with.with1.scenario-1` | `Conformance` | scope | `Passed` | 1.099 ms |
| `grafeo.match.directed-edge` | `Conformance` | match | `Passed` | 0.899 ms |
| `age.vle.zero-length` | `Conformance` | traversal | `Passed` | 2.688 ms |
| `pggraph.traversal.exact-two-hops` | `Conformance` | traversal | `Passed` | 2.352 ms |
| `ladybug.match.undirected-edge` | `Conformance` | match | `Passed` | 0.807 ms |
| `ladybug.optional.null-extension` | `Conformance` | optional-match | `Passed` | 0.686 ms |
| `sparrow.path.two-hop-multiplicity` | `Conformance` | traversal | `Passed` | 2.116 ms |
| `sparrow.merge.existing-node` | `Conformance` | mutation | `Passed` | 0.641 ms |
| `cqlite.match.labeled-node-scan` | `Conformance` | match | `Passed` | 0.511 ms |
| `cqlite.create.properties` | `Conformance` | mutation | `Passed` | 1.188 ms |
| `samyama.aggregate.global-count` | `Conformance` | aggregation | `Passed` | 0.321 ms |
| `age.vle.unbounded-traversal` | `Conformance` | traversal | `Passed` | 2.113 ms |
| `tck.call.subquery-scope` | `Conformance` | subquery | `Unsupported` | 0.020 ms |
| `grafeo.path.all-shortest-paths` | `Conformance` | shortest-path | `Unsupported` | 0.135 ms |
| `pggraph.path.weight-expression` | `Conformance` | shortest-path | `Unsupported` | 0.147 ms |
| `ladybug.path.shortest-keyword` | `Conformance` | shortest-path | `Unsupported` | 0.061 ms |
| `sparrow.path.shortest-function` | `Conformance` | shortest-path | `Unsupported` | 0.139 ms |
| `samyama.planner.independent-patterns` | `Conformance` | planning | `Unsupported` | 0.183 ms |
| `grafeo.match.incoming-edge` | `Conformance` | match | `Passed` | 0.710 ms |
| `age.vle.fixed-multi-hop` | `Conformance` | traversal | `Passed` | 1.976 ms |
| `grafeo.with.projected-expression` | `Conformance` | scope | `Passed` | 0.419 ms |
| `age.unwind.literal-list` | `Conformance` | unwind | `Passed` | 0.442 ms |
| `grafeo.order-by.nonprojected-property` | `Conformance` | ordering | `Passed` | 0.360 ms |
| `grafeo.pagination.skip-limit` | `Conformance` | pagination | `Passed` | 0.427 ms |
| `grafeo.optional.where-null-extends-pattern` | `Conformance` | optional-match | `Passed` | 0.804 ms |
| `tck.where.numeric-comparison` | `Conformance` | filter | `Passed` | 0.583 ms |
| `cqlite.set.property` | `Conformance` | mutation | `Passed` | 1.185 ms |
| `age.remove.property` | `Conformance` | mutation | `Passed` | 0.933 ms |
| `age.delete.relationship` | `Conformance` | mutation | `Passed` | 1.538 ms |
| `sparrow.merge.absent-node` | `Conformance` | mutation | `Passed` | 0.894 ms |
| `grafeo.regression.wrong-relationship-direction` | `BugRegression` | match | `Passed` | 0.666 ms |
| `age.regression.zero-length-preserves-identity` | `Regression` | traversal | `Passed` | 2.141 ms |
| `sparrow.regression.missing-property-is-null` | `BugRegression` | null | `Passed` | 0.430 ms |
| `sparrow.regression.variable-path-terminal-label` | `BugRegression` | traversal | `Passed` | 2.035 ms |
| `cqlite.regression.parameterized-property` | `Regression` | parameters | `Passed` | 0.376 ms |
| `ladybug.regression.detach-delete` | `Regression` | mutation | `Passed` | 1.014 ms |
| `grafeo.regression.optional-count-preserves-rows` | `BugRegression` | aggregation | `Passed` | 0.614 ms |
| `turso.regression.constraint-index-drop-error` | `BugRegression` | mutation | `Passed` | 0.431 ms |

## Latest `performance-deep` run

- Run: `20260718T013944.410388Z-e1d73880b749-performance-deep`
- Commit: `e1d73880b74901c879c5bcf4cc96b1006f2d16b5` (dirty)
- Package: `0.7.0`
- Environment: `macos/aarch64` (`dev`)
- Records: 10
- Passed: 10
- Unsupported: 0
- Failed or changed: 0

| Test | Operation | Scale | Outcome | Duration | Throughput/s |
|---|---|---:|---|---:|---:|
| `perf.line.create.s000100` | create | 100 | `Passed` | 40.369 ms | 2477.13 |
| `perf.line.bulk-load.s000100` | bulk-load | 100 | `Passed` | 12.613 ms | 15776.95 |
| `perf.line.load.s000100` | load | 100 | `Passed` | 3.720 ms | 53493.43 |
| `perf.line.query.s000100` | query | 100 | `Passed` | 7.510 ms | 1331.52 |
| `perf.line.delete.s000100` | delete | 100 | `Passed` | 38.866 ms | 2572.93 |
| `perf.line.create.s001000` | create | 1000 | `Passed` | 370.248 ms | 2700.89 |
| `perf.line.bulk-load.s001000` | bulk-load | 1000 | `Passed` | 282.280 ms | 7081.63 |
| `perf.line.load.s001000` | load | 1000 | `Passed` | 13.975 ms | 143038.16 |
| `perf.line.query.s001000` | query | 1000 | `Passed` | 61.859 ms | 161.66 |
| `perf.line.delete.s001000` | delete | 1000 | `Passed` | 503.315 ms | 1986.83 |

## Latest `performance-smoke` run

- Run: `20260718T013943.199409Z-e1d73880b749-performance-smoke`
- Commit: `e1d73880b74901c879c5bcf4cc96b1006f2d16b5` (dirty)
- Package: `0.7.0`
- Environment: `macos/aarch64` (`dev`)
- Records: 10
- Passed: 10
- Unsupported: 0
- Failed or changed: 0

| Test | Operation | Scale | Outcome | Duration | Throughput/s |
|---|---|---:|---|---:|---:|
| `perf.line.create.s000010` | create | 10 | `Passed` | 5.460 ms | 1831.53 |
| `perf.line.bulk-load.s000010` | bulk-load | 10 | `Passed` | 1.454 ms | 13064.78 |
| `perf.line.load.s000010` | load | 10 | `Passed` | 2.374 ms | 8003.79 |
| `perf.line.query.s000010` | query | 10 | `Passed` | 1.088 ms | 2756.61 |
| `perf.line.delete.s000010` | delete | 10 | `Passed` | 3.783 ms | 2643.64 |
| `perf.line.create.s000100` | create | 100 | `Passed` | 33.450 ms | 2989.58 |
| `perf.line.bulk-load.s000100` | bulk-load | 100 | `Passed` | 11.948 ms | 16655.45 |
| `perf.line.load.s000100` | load | 100 | `Passed` | 3.062 ms | 64996.40 |
| `perf.line.query.s000100` | query | 100 | `Passed` | 2.131 ms | 1407.82 |
| `perf.line.delete.s000100` | delete | 100 | `Passed` | 37.953 ms | 2634.83 |

## Latest `smoke` run

- Run: `20260718T013940.911425Z-e1d73880b749-smoke`
- Commit: `e1d73880b74901c879c5bcf4cc96b1006f2d16b5`
- Package: `0.7.0`
- Environment: `macos/aarch64` (`dev`)
- Records: 13
- Passed: 13
- Unsupported: 0
- Failed or changed: 0

| Test | Kind | Area | Outcome | Duration |
|---|---|---|---|---:|
| `tck.with.with1.scenario-1` | `Conformance` | scope | `Passed` | 1.219 ms |
| `grafeo.match.directed-edge` | `Conformance` | match | `Passed` | 1.133 ms |
| `age.vle.zero-length` | `Conformance` | traversal | `Passed` | 2.745 ms |
| `pggraph.traversal.exact-two-hops` | `Conformance` | traversal | `Passed` | 2.592 ms |
| `ladybug.match.undirected-edge` | `Conformance` | match | `Passed` | 0.977 ms |
| `ladybug.optional.null-extension` | `Conformance` | optional-match | `Passed` | 0.798 ms |
| `sparrow.path.two-hop-multiplicity` | `Conformance` | traversal | `Passed` | 2.105 ms |
| `sparrow.merge.existing-node` | `Conformance` | mutation | `Passed` | 0.610 ms |
| `cqlite.match.labeled-node-scan` | `Conformance` | match | `Passed` | 0.402 ms |
| `cqlite.create.properties` | `Conformance` | mutation | `Passed` | 1.104 ms |
| `samyama.aggregate.global-count` | `Conformance` | aggregation | `Passed` | 0.269 ms |
| `grafeo.regression.wrong-relationship-direction` | `BugRegression` | match | `Passed` | 0.663 ms |
| `cqlite.regression.parameterized-property` | `Regression` | parameters | `Passed` | 0.394 ms |

## Longitudinal inventory

- Runs: 4
- Result records: 71
- Unique test identities: 53
