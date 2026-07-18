# Graph test history

Generated from `graph/test-results/history.jsonl`. Results are grouped by stable test identity; performance comparisons are meaningful only for matching environment and workload dimensions.

## Latest `age-deep` run

- Run: `20260718T022017.653879Z-8d6f4f6d535d-corpus-deep`
- Commit: `8d6f4f6d535d00d5364f23df8553956f052df599`
- Package: `0.7.0`
- Environment: `macos/aarch64` (`dev`)
- Records: 3677
- Passed: 0
- Unsupported: 3677
- Failed or changed: 0

### Results by source area

| Area | Outcome | Count |
|---|---|---:|
| age_global_graph | `unsupported` | 57 |
| age_load | `unsupported` | 13 |
| age_reduce | `unsupported` | 75 |
| age_shortest_path | `unsupported` | 194 |
| agtype | `unsupported` | 18 |
| agtype_jsonb_cast | `unsupported` | 3 |
| analyze | `unsupported` | 2 |
| catalog | `unsupported` | 7 |
| cypher | `unsupported` | 20 |
| cypher_call | `unsupported` | 42 |
| cypher_create | `unsupported` | 93 |
| cypher_delete | `unsupported` | 115 |
| cypher_match | `unsupported` | 413 |
| cypher_merge | `unsupported` | 273 |
| cypher_remove | `unsupported` | 42 |
| cypher_set | `unsupported` | 117 |
| cypher_subquery | `unsupported` | 53 |
| cypher_union | `unsupported` | 19 |
| cypher_unwind | `unsupported` | 17 |
| cypher_vle | `unsupported` | 112 |
| cypher_with | `unsupported` | 41 |
| direct_field_access | `unsupported` | 41 |
| expr | `unsupported` | 1089 |
| fuzzystrmatch | `unsupported` | 11 |
| generated_columns | `unsupported` | 10 |
| graph_generation | `unsupported` | 2 |
| index | `unsupported` | 65 |
| issue_369 | `unsupported` | 4 |
| jsonb_operators | `unsupported` | 159 |
| list_comprehension | `unsupported` | 126 |
| map_projection | `unsupported` | 18 |
| name_validation | `unsupported` | 10 |
| pattern_expression | `unsupported` | 32 |
| pg_trgm | `unsupported` | 6 |
| pgvector | `unsupported` | 71 |
| predicate_functions | `unsupported` | 62 |
| reserved_keyword_alias | `unsupported` | 31 |
| scan | `unsupported` | 57 |
| security | `unsupported` | 133 |
| subgraph | `unsupported` | 24 |

### Results by execution boundary

| Boundary | Outcome | Count |
|---|---|---:|
| `adapter` | `unsupported` | 1555 |
| `deduplicated` | `unsupported` | 620 |
| `parser` | `unsupported` | 1502 |

### Failures (0)

- None.

## Latest `cqlite-deep` run

- Run: `20260718T022017.653879Z-8d6f4f6d535d-corpus-deep`
- Commit: `8d6f4f6d535d00d5364f23df8553956f052df599`
- Package: `0.7.0`
- Environment: `macos/aarch64` (`dev`)
- Records: 137
- Passed: 0
- Unsupported: 137
- Failed or changed: 0

| Test | Kind | Area | Outcome | Duration |
|---|---|---|---|---:|
| `cqlite.basic-queries.run-a-to-b.query-1` | `Conformance` | basic_queries | `Unsupported` | 0.036 ms |
| `cqlite.basic-queries.run-a-to-b.query-2` | `Conformance` | basic_queries | `Unsupported` | 0.095 ms |
| `cqlite.basic-queries.run-a-to-b.query-3` | `Conformance` | basic_queries | `Unsupported` | 0.099 ms |
| `cqlite.basic-queries.run-a-edge-b.query-1` | `Conformance` | basic_queries | `Unsupported` | 0.062 ms |
| `cqlite.basic-queries.run-a-edge-b.query-2` | `Conformance` | basic_queries | `Unsupported` | 0.103 ms |
| `cqlite.basic-queries.run-a-to-a.query-1` | `Conformance` | basic_queries | `Unsupported` | 0.091 ms |
| `cqlite.basic-queries.run-a-to-a.query-2` | `Conformance` | basic_queries | `Unsupported` | 0.081 ms |
| `cqlite.basic-queries.run-a-edge-a.query-1` | `Conformance` | basic_queries | `Unsupported` | 0.187 ms |
| `cqlite.basic-queries.run-a-edge-a.query-2` | `Conformance` | basic_queries | `Unsupported` | 0.084 ms |
| `cqlite.basic-queries.run-a-knows-b.query-1` | `Conformance` | basic_queries | `Unsupported` | 0.095 ms |
| `cqlite.basic-queries.run-a-knows-b.query-2` | `Conformance` | basic_queries | `Unsupported` | 0.107 ms |
| `cqlite.basic-queries.run-a-edge-b-with-where-property.query-1` | `Conformance` | basic_queries | `Unsupported` | 0.092 ms |
| `cqlite.basic-queries.run-a-edge-b-with-where-property.query-2` | `Conformance` | basic_queries | `Unsupported` | 0.127 ms |
| `cqlite.basic-queries.run-a-edge-b-with-property-map.query-1` | `Conformance` | basic_queries | `Unsupported` | 0.084 ms |
| `cqlite.basic-queries.run-a-edge-b-with-property-map.query-2` | `Conformance` | basic_queries | `Unsupported` | 0.121 ms |
| `cqlite.basic-queries.run-a-edge-b-with-where-id.query-1` | `Conformance` | basic_queries | `Unsupported` | 0.037 ms |
| `cqlite.basic-queries.run-a-edge-b-with-where-id.query-2` | `Conformance` | basic_queries | `Unsupported` | 0.086 ms |
| `cqlite.basic-queries.run-a-where-with-parameters.query-1` | `Conformance` | basic_queries | `Unsupported` | 0.093 ms |
| `cqlite.basic-queries.run-a-where-with-parameters.query-2` | `Conformance` | basic_queries | `Unsupported` | 0.082 ms |
| `cqlite.basic-queries.run-set.query-1` | `Conformance` | basic_queries | `Unsupported` | 0.020 ms |
| `cqlite.basic-queries.run-set.query-2` | `Conformance` | basic_queries | `Unsupported` | 0.041 ms |
| `cqlite.basic-queries.run-set.query-3` | `Conformance` | basic_queries | `Unsupported` | 0.076 ms |
| `cqlite.basic-queries.return-from-set.query-1` | `Conformance` | basic_queries | `Unsupported` | 0.000 ms |
| `cqlite.basic-queries.return-from-set.query-2` | `Conformance` | basic_queries | `Unsupported` | 0.089 ms |
| `cqlite.basic-queries.return-from-set.query-3` | `Conformance` | basic_queries | `Unsupported` | 0.071 ms |
| `cqlite.basic-queries.run-delete-node.query-1` | `Conformance` | basic_queries | `Unsupported` | 0.000 ms |
| `cqlite.basic-queries.run-delete-node.query-2` | `Conformance` | basic_queries | `Unsupported` | 0.052 ms |
| `cqlite.basic-queries.run-delete-node.query-3` | `Conformance` | basic_queries | `Unsupported` | 0.029 ms |
| `cqlite.basic-queries.run-delete-edge.query-1` | `Conformance` | basic_queries | `Unsupported` | 0.046 ms |
| `cqlite.basic-queries.run-delete-edge.query-2` | `Conformance` | basic_queries | `Unsupported` | 0.057 ms |
| `cqlite.basic-queries.run-delete-edge.query-3` | `Conformance` | basic_queries | `Unsupported` | 0.039 ms |
| `cqlite.basic-queries.run-bad-delete.query-1` | `Conformance` | basic_queries | `Unsupported` | 0.000 ms |
| `cqlite.basic-queries.run-bad-delete.query-2` | `Conformance` | basic_queries | `Unsupported` | 0.000 ms |
| `cqlite.basic-queries.run-return-label.query-1` | `Conformance` | basic_queries | `Unsupported` | 0.046 ms |
| `cqlite.basic-queries.run-return-label.query-2` | `Conformance` | basic_queries | `Unsupported` | 0.083 ms |
| `cqlite.basic-queries.match-return-count.query-1` | `Conformance` | basic_queries | `Unsupported` | 0.062 ms |
| `cqlite.basic-queries.match-return-count.query-2` | `Conformance` | basic_queries | `Unsupported` | 0.079 ms |
| `cqlite.basic-queries.match-return-count.query-3` | `Conformance` | basic_queries | `Unsupported` | 0.032 ms |
| `cqlite.basic-queries.match-multiple-edges.query-1` | `Conformance` | basic_queries | `Unsupported` | 0.108 ms |
| `cqlite.basic-queries.match-multiple-edges.query-2` | `Conformance` | basic_queries | `Unsupported` | 0.022 ms |
| `cqlite.create-queries.create-label-only.query-1` | `Conformance` | create_queries | `Unsupported` | 0.052 ms |
| `cqlite.create-queries.create-label-only.query-2` | `Conformance` | create_queries | `Unsupported` | 0.048 ms |
| `cqlite.create-queries.create-with-properties.query-1` | `Conformance` | create_queries | `Unsupported` | 0.090 ms |
| `cqlite.create-queries.create-with-properties.query-2` | `Conformance` | create_queries | `Unsupported` | 0.060 ms |
| `cqlite.create-queries.create-with-properties-from-parameters.query-1` | `Conformance` | create_queries | `Unsupported` | 0.089 ms |
| `cqlite.create-queries.create-with-properties-from-parameters.query-2` | `Conformance` | create_queries | `Unsupported` | 0.000 ms |
| `cqlite.create-queries.create-edges-with-label.query-1` | `Conformance` | create_queries | `Unsupported` | 0.062 ms |
| `cqlite.create-queries.create-edges-with-label.query-2` | `Conformance` | create_queries | `Unsupported` | 0.105 ms |
| `cqlite.delete-queries.delete-node.query-1` | `Conformance` | delete_queries | `Unsupported` | 0.021 ms |
| `cqlite.delete-queries.delete-node.query-2` | `Conformance` | delete_queries | `Unsupported` | 0.048 ms |
| `cqlite.delete-queries.delete-node.query-3` | `Conformance` | delete_queries | `Unsupported` | 0.002 ms |
| `cqlite.delete-queries.delete-node.query-4` | `Conformance` | delete_queries | `Unsupported` | 0.000 ms |
| `cqlite.delete-queries.double-delete-node.query-1` | `Conformance` | delete_queries | `Unsupported` | 0.000 ms |
| `cqlite.delete-queries.double-delete-node.query-2` | `Conformance` | delete_queries | `Unsupported` | 0.000 ms |
| `cqlite.delete-queries.double-delete-node.query-3` | `Conformance` | delete_queries | `Unsupported` | 0.034 ms |
| `cqlite.delete-queries.double-delete-node.query-4` | `Conformance` | delete_queries | `Unsupported` | 0.000 ms |
| `cqlite.delete-queries.delete-edge.query-1` | `Conformance` | delete_queries | `Unsupported` | 0.060 ms |
| `cqlite.delete-queries.delete-edge.query-2` | `Conformance` | delete_queries | `Unsupported` | 0.023 ms |
| `cqlite.delete-queries.delete-edge.query-3` | `Conformance` | delete_queries | `Unsupported` | 0.035 ms |
| `cqlite.delete-queries.delete-edge.query-4` | `Conformance` | delete_queries | `Unsupported` | 0.000 ms |
| `cqlite.delete-queries.connected-delete-fails.query-1` | `Conformance` | delete_queries | `Unsupported` | 0.063 ms |
| `cqlite.delete-queries.connected-delete-fails.query-2` | `Conformance` | delete_queries | `Unsupported` | 0.000 ms |
| `cqlite.delete-queries.connected-delete-fails.query-3` | `Conformance` | delete_queries | `Unsupported` | 0.056 ms |
| `cqlite.delete-queries.connected-delete-fails.query-4` | `Conformance` | delete_queries | `Unsupported` | 0.000 ms |
| `cqlite.match-queries.create-test-graph.query-1` | `Conformance` | match_queries | `Unsupported` | 0.257 ms |
| `cqlite.match-queries.match-all-nodes.query-1` | `Conformance` | match_queries | `Unsupported` | 0.080 ms |
| `cqlite.match-queries.match-multiple-nodes.query-1` | `Conformance` | match_queries | `Unsupported` | 0.082 ms |
| `cqlite.match-queries.match-multiple-nodes.query-2` | `Conformance` | match_queries | `Unsupported` | 0.059 ms |
| `cqlite.match-queries.match-single-directed-edge.query-1` | `Conformance` | match_queries | `Unsupported` | 0.000 ms |
| `cqlite.match-queries.match-single-undirected-edge.query-1` | `Conformance` | match_queries | `Unsupported` | 0.000 ms |
| `cqlite.match-queries.match-single-path.query-1` | `Conformance` | match_queries | `Unsupported` | 0.025 ms |
| `cqlite.match-queries.match-path-with-multiple-clauses.query-1` | `Conformance` | match_queries | `Unsupported` | 0.026 ms |
| `cqlite.match-queries.match-long-path.query-1` | `Conformance` | match_queries | `Unsupported` | 0.025 ms |
| `cqlite.match-queries.match-labeled-nodes.query-1` | `Conformance` | match_queries | `Unsupported` | 0.000 ms |
| `cqlite.match-queries.match-labeled-nodes.query-2` | `Conformance` | match_queries | `Unsupported` | 0.052 ms |
| `cqlite.match-queries.match-labeled-nodes.query-3` | `Conformance` | match_queries | `Unsupported` | 0.051 ms |
| `cqlite.match-queries.match-labeled-edges.query-1` | `Conformance` | match_queries | `Unsupported` | 0.060 ms |
| `cqlite.match-queries.match-labeled-edges.query-2` | `Conformance` | match_queries | `Unsupported` | 0.061 ms |
| `cqlite.match-queries.match-nodes-with-properties.query-1` | `Conformance` | match_queries | `Unsupported` | 0.067 ms |
| `cqlite.match-queries.match-nodes-with-properties.query-2` | `Conformance` | match_queries | `Unsupported` | 0.061 ms |
| `cqlite.match-queries.match-nodes-with-properties.query-3` | `Conformance` | match_queries | `Unsupported` | 0.089 ms |
| `cqlite.match-queries.match-edges-with-properties.query-1` | `Conformance` | match_queries | `Unsupported` | 0.073 ms |
| `cqlite.match-queries.match-nodes-with-label.query-1` | `Conformance` | match_queries | `Unsupported` | 0.000 ms |
| `cqlite.match-queries-where.create-test-graph.query-1` | `Conformance` | match_queries_where | `Unsupported` | 0.520 ms |
| `cqlite.match-queries-where.match-where-node-id-eq.query-1` | `Conformance` | match_queries_where | `Unsupported` | 0.070 ms |
| `cqlite.match-queries-where.match-where-node-id-eq.query-2` | `Conformance` | match_queries_where | `Unsupported` | 0.068 ms |
| `cqlite.match-queries-where.match-where-node-id-eq-non-id.query-1` | `Conformance` | match_queries_where | `Unsupported` | 0.069 ms |
| `cqlite.match-queries-where.match-where-node-label-eq.query-1` | `Conformance` | match_queries_where | `Unsupported` | 0.043 ms |
| `cqlite.match-queries-where.match-where-node-label-eq.query-2` | `Conformance` | match_queries_where | `Unsupported` | 0.069 ms |
| `cqlite.match-queries-where.match-where-node-prop-eq.query-1` | `Conformance` | match_queries_where | `Unsupported` | 0.068 ms |
| `cqlite.match-queries-where.match-where-node-prop.query-1` | `Conformance` | match_queries_where | `Unsupported` | 0.061 ms |
| `cqlite.match-queries-where.match-where-not-node-prop.query-1` | `Conformance` | match_queries_where | `Unsupported` | 0.034 ms |
| `cqlite.match-queries-where.match-where-node-prop-eq-true-false.query-1` | `Conformance` | match_queries_where | `Unsupported` | 0.068 ms |
| `cqlite.match-queries-where.match-where-node-prop-eq-true-false.query-2` | `Conformance` | match_queries_where | `Unsupported` | 0.068 ms |
| `cqlite.match-queries-where.match-where-node-prop-ne-null.query-1` | `Conformance` | match_queries_where | `Unsupported` | 0.083 ms |
| `cqlite.match-queries-where.match-where-node-prop-lt-or-gt.query-1` | `Conformance` | match_queries_where | `Unsupported` | 0.068 ms |
| `cqlite.match-queries-where.match-where-node-prop-lt-or-gt.query-2` | `Conformance` | match_queries_where | `Unsupported` | 0.068 ms |
| `cqlite.match-queries-where.match-where-node-prop-lt-or-gt.query-3` | `Conformance` | match_queries_where | `Unsupported` | 0.068 ms |
| `cqlite.match-queries-where.match-where-edge-id-eq.query-1` | `Conformance` | match_queries_where | `Unsupported` | 0.129 ms |
| `cqlite.match-queries-where.match-where-edge-id-eq.query-2` | `Conformance` | match_queries_where | `Unsupported` | 0.131 ms |
| `cqlite.match-queries-where.match-where-edge-prop-eq.query-1` | `Conformance` | match_queries_where | `Unsupported` | 0.121 ms |
| `cqlite.match-queries-where.match-where-edge-prop-gt.query-1` | `Conformance` | match_queries_where | `Unsupported` | 0.119 ms |
| `cqlite.match-queries-where.match-where-a-or-b.query-1` | `Conformance` | match_queries_where | `Unsupported` | 0.147 ms |
| `cqlite.match-queries-where.match-long-path-with-id-constraint.query-1` | `Conformance` | match_queries_where | `Unsupported` | 0.028 ms |
| `cqlite.match-queries-where.match-long-path-with-id-constraint.query-2` | `Conformance` | match_queries_where | `Unsupported` | 0.028 ms |
| `cqlite.match-queries-where.match-short-path-with-id-constraint.query-1` | `Conformance` | match_queries_where | `Unsupported` | 0.025 ms |
| `cqlite.return-queries.return-parameter.query-1` | `Conformance` | return_queries | `Unsupported` | 0.023 ms |
| `cqlite.return-queries.return-id-of.query-1` | `Conformance` | return_queries | `Unsupported` | 0.048 ms |
| `cqlite.return-queries.return-id-of.query-2` | `Conformance` | return_queries | `Unsupported` | 0.045 ms |
| `cqlite.return-queries.return-label-of.query-1` | `Conformance` | return_queries | `Unsupported` | 0.047 ms |
| `cqlite.return-queries.return-label-of.query-2` | `Conformance` | return_queries | `Unsupported` | 0.045 ms |
| `cqlite.return-queries.create-and-return.query-1` | `Conformance` | return_queries | `Unsupported` | 0.089 ms |
| `cqlite.return-queries.create-and-return.query-2` | `Conformance` | return_queries | `Unsupported` | 0.055 ms |
| `cqlite.return-queries.set-and-return.query-1` | `Conformance` | return_queries | `Unsupported` | 0.000 ms |
| `cqlite.return-queries.set-and-return.query-2` | `Conformance` | return_queries | `Unsupported` | 0.065 ms |
| `cqlite.return-queries.delete-and-return.query-1` | `Conformance` | return_queries | `Unsupported` | 0.038 ms |
| `cqlite.return-queries.delete-and-return.query-2` | `Conformance` | return_queries | `Unsupported` | 0.049 ms |
| `cqlite.return-queries.return-out-of-bounds.query-1` | `Conformance` | return_queries | `Unsupported` | 0.035 ms |
| `cqlite.set-queries.set-once.query-1` | `Conformance` | set_queries | `Unsupported` | 0.061 ms |
| `cqlite.set-queries.set-once.query-2` | `Conformance` | set_queries | `Unsupported` | 0.063 ms |
| `cqlite.set-queries.set-once.query-3` | `Conformance` | set_queries | `Unsupported` | 0.002 ms |
| `cqlite.set-queries.set-after-create.query-1` | `Conformance` | set_queries | `Unsupported` | 0.081 ms |
| `cqlite.set-queries.set-after-create.query-2` | `Conformance` | set_queries | `Unsupported` | 0.000 ms |
| `cqlite.set-queries.set-multiple-times.query-1` | `Conformance` | set_queries | `Unsupported` | 0.130 ms |
| `cqlite.set-queries.set-multiple-times.query-2` | `Conformance` | set_queries | `Unsupported` | 0.000 ms |
| `cqlite.set-queries.delete-property.query-1` | `Conformance` | set_queries | `Unsupported` | 0.000 ms |
| `cqlite.set-queries.delete-property.query-2` | `Conformance` | set_queries | `Unsupported` | 0.062 ms |
| `cqlite.set-queries.delete-property.query-3` | `Conformance` | set_queries | `Unsupported` | 0.000 ms |
| `cqlite.txn-semantics.concurrent-reader-and-writer.query-1` | `Conformance` | txn_semantics | `Unsupported` | 0.037 ms |
| `cqlite.txn-semantics.concurrent-reader-and-writer.query-2` | `Conformance` | txn_semantics | `Unsupported` | 0.041 ms |
| `cqlite.txn-semantics.concurrent-reader-and-writer.query-3` | `Conformance` | txn_semantics | `Unsupported` | 0.038 ms |
| `cqlite.txn-semantics.concurrent-reader-and-writer.query-4` | `Conformance` | txn_semantics | `Unsupported` | 0.042 ms |
| `cqlite.txn-semantics.concurrent-reader-and-writer.query-5` | `Conformance` | txn_semantics | `Unsupported` | 0.000 ms |
| `cqlite.where-conditions.where-a-and-b.query-1` | `Conformance` | where_conditions | `Unsupported` | 0.010 ms |
| `cqlite.where-conditions.where-a-or-b.query-1` | `Conformance` | where_conditions | `Unsupported` | 0.009 ms |
| `cqlite.where-conditions.where-a.query-1` | `Conformance` | where_conditions | `Unsupported` | 0.009 ms |
| `cqlite.where-conditions.where-not-a.query-1` | `Conformance` | where_conditions | `Unsupported` | 0.009 ms |

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

## Latest `grafeo-deep` run

- Run: `20260718T022017.653879Z-8d6f4f6d535d-corpus-deep`
- Commit: `8d6f4f6d535d00d5364f23df8553956f052df599`
- Package: `0.7.0`
- Environment: `macos/aarch64` (`dev`)
- Records: 399
- Passed: 2
- Unsupported: 397
- Failed or changed: 0

| Test | Kind | Area | Outcome | Duration |
|---|---|---|---|---:|
| `grafeo.spec.common.index.correctness.create.index.then.query` | `Conformance` | common | `Unsupported` | 0.031 ms |
| `grafeo.spec.common.index.correctness.index.query.no.match` | `Conformance` | common | `Unsupported` | 0.017 ms |
| `grafeo.spec.common.index.correctness.index.multiple.matches` | `Conformance` | common | `Unsupported` | 0.016 ms |
| `grafeo.spec.common.index.correctness.index.with.null.property` | `Conformance` | common | `Unsupported` | 0.016 ms |
| `grafeo.spec.common.index.correctness.index.after.property.update` | `Conformance` | common | `Unsupported` | 0.003 ms |
| `grafeo.spec.common.index.correctness.index.old.value.gone.after.update` | `Conformance` | common | `Unsupported` | 0.003 ms |
| `grafeo.spec.common.index.correctness.index.after.delete` | `Conformance` | common | `Unsupported` | 0.002 ms |
| `grafeo.spec.common.index.correctness.index.remaining.after.delete` | `Conformance` | common | `Unsupported` | 0.002 ms |
| `grafeo.spec.common.index.correctness.index.reinsert.after.delete` | `Conformance` | common | `Unsupported` | 0.002 ms |
| `grafeo.spec.common.index.correctness.numeric.index.exact.lookup` | `Conformance` | common | `Unsupported` | 0.015 ms |
| `grafeo.spec.common.index.correctness.numeric.index.range.query` | `Conformance` | common | `Unsupported` | 0.003 ms |
| `grafeo.spec.common.index.correctness.bulk.insert.then.index` | `Conformance` | common | `Unsupported` | 0.262 ms |
| `grafeo.spec.common.index.correctness.index.count.all` | `Conformance` | common | `Unsupported` | 0.015 ms |
| `grafeo.spec.common.index.correctness.drop.index.query.still.works` | `Conformance` | common | `Unsupported` | 0.002 ms |
| `grafeo.spec.common.null.semantics.negative.limit.returns.empty.cypher.cypher-variant` | `Conformance` | common | `Unsupported` | 0.063 ms |
| `grafeo.spec.common.numeric.edge.cases.min.int64.cypher.cypher-variant` | `Conformance` | common | `Unsupported` | 0.047 ms |
| `grafeo.spec.common.numeric.edge.cases.nan.literal.keyword.cypher.cypher-variant` | `Conformance` | common | `Unsupported` | 5.499 ms |
| `grafeo.spec.common.numeric.edge.cases.inf.literal.keyword.cypher.cypher-variant` | `Conformance` | common | `Unsupported` | 5.261 ms |
| `grafeo.spec.lpg.cypher.admin.create.index.on.label.property` | `Conformance` | lpg | `Unsupported` | 0.004 ms |
| `grafeo.spec.lpg.cypher.admin.create.index.and.query` | `Conformance` | lpg | `Unsupported` | 0.023 ms |
| `grafeo.spec.lpg.cypher.admin.drop.index` | `Conformance` | lpg | `Unsupported` | 0.003 ms |
| `grafeo.spec.lpg.cypher.admin.show.indexes.empty` | `Conformance` | lpg | `Unsupported` | 0.008 ms |
| `grafeo.spec.lpg.cypher.admin.show.indexes.after.create` | `Conformance` | lpg | `Unsupported` | 0.004 ms |
| `grafeo.spec.lpg.cypher.admin.explain.match` | `Conformance` | lpg | `Unsupported` | 0.011 ms |
| `grafeo.spec.lpg.cypher.admin.profile.match` | `Conformance` | lpg | `Unsupported` | 0.011 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.pattern.comprehension.basic` | `Conformance` | lpg | `Unsupported` | 0.101 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.pattern.comprehension.with.filter` | `Conformance` | lpg | `Unsupported` | 0.092 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.pattern.comprehension.size` | `Conformance` | lpg | `Unsupported` | 0.098 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.pattern.comprehension.with.property.extraction` | `Conformance` | lpg | `Unsupported` | 0.085 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.list.comprehension.basic` | `Conformance` | lpg | `Unsupported` | 0.079 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.list.comprehension.transform` | `Conformance` | lpg | `Unsupported` | 0.076 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.list.comprehension.filter.and.transform` | `Conformance` | lpg | `Unsupported` | 0.074 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.list.comprehension.nested` | `Conformance` | lpg | `Unsupported` | 0.082 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.exists.subquery.actors.with.action.movies` | `Conformance` | lpg | `Unsupported` | 0.040 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.not.exists.subquery` | `Conformance` | lpg | `Unsupported` | 0.039 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.count.subquery.movies.per.actor` | `Conformance` | lpg | `Unsupported` | 0.039 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.count.subquery.prolific.directors` | `Conformance` | lpg | `Unsupported` | 0.039 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.call.subquery.basic` | `Conformance` | lpg | `Unsupported` | 0.029 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.call.subquery.with.aggregation` | `Conformance` | lpg | `Unsupported` | 0.029 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.foreach.set.property` | `Conformance` | lpg | `Unsupported` | 0.055 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.foreach.create.relationships` | `Conformance` | lpg | `Unsupported` | 0.079 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.actor.collaboration.via.comprehension` | `Conformance` | lpg | `Unsupported` | 0.101 ms |
| `grafeo.spec.lpg.cypher.comprehensions.advanced.genre.diversity.per.actor` | `Conformance` | lpg | `Unsupported` | 0.101 ms |
| `grafeo.spec.lpg.cypher.constraints.create.unique.constraint` | `Conformance` | lpg | `Unsupported` | 0.020 ms |
| `grafeo.spec.lpg.cypher.constraints.unique.constraint.allows.distinct.values` | `Conformance` | lpg | `Unsupported` | 0.071 ms |
| `grafeo.spec.lpg.cypher.constraints.unique.constraint.violation` | `Conformance` | lpg | `Unsupported` | 0.059 ms |
| `grafeo.spec.lpg.cypher.constraints.unique.constraint.null.allowed` | `Conformance` | lpg | `Unsupported` | 0.057 ms |
| `grafeo.spec.lpg.cypher.constraints.create.not.null.constraint` | `Conformance` | lpg | `Unsupported` | 0.020 ms |
| `grafeo.spec.lpg.cypher.constraints.not.null.constraint.satisfied` | `Conformance` | lpg | `Unsupported` | 0.067 ms |
| `grafeo.spec.lpg.cypher.constraints.not.null.constraint.violation` | `Conformance` | lpg | `Unsupported` | 0.034 ms |
| `grafeo.spec.lpg.cypher.constraints.not.null.constraint.violation.on.set` | `Conformance` | lpg | `Unsupported` | 0.047 ms |
| `grafeo.spec.lpg.cypher.constraints.create.node.key.constraint` | `Conformance` | lpg | `Unsupported` | 0.020 ms |
| `grafeo.spec.lpg.cypher.constraints.node.key.allows.different.combinations` | `Conformance` | lpg | `Unsupported` | 0.081 ms |
| `grafeo.spec.lpg.cypher.constraints.node.key.violation.duplicate` | `Conformance` | lpg | `Unsupported` | 0.048 ms |
| `grafeo.spec.lpg.cypher.constraints.node.key.violation.missing.property` | `Conformance` | lpg | `Unsupported` | 0.034 ms |
| `grafeo.spec.lpg.cypher.constraints.drop.constraint` | `Conformance` | lpg | `Unsupported` | 0.009 ms |
| `grafeo.spec.lpg.cypher.constraints.drop.nonexistent.constraint` | `Conformance` | lpg | `Passed` | 0.009 ms |
| `grafeo.spec.lpg.cypher.constraints.drop.constraint.if.exists` | `Conformance` | lpg | `Unsupported` | 0.011 ms |
| `grafeo.spec.lpg.cypher.constraints.show.constraints.after.create` | `Conformance` | lpg | `Unsupported` | 0.007 ms |
| `grafeo.spec.lpg.cypher.constraints.show.constraints.empty` | `Conformance` | lpg | `Unsupported` | 0.001 ms |
| `grafeo.spec.lpg.cypher.expressions.addition` | `Conformance` | lpg | `Unsupported` | 0.055 ms |
| `grafeo.spec.lpg.cypher.expressions.subtraction` | `Conformance` | lpg | `Unsupported` | 0.053 ms |
| `grafeo.spec.lpg.cypher.expressions.multiplication` | `Conformance` | lpg | `Unsupported` | 0.052 ms |
| `grafeo.spec.lpg.cypher.expressions.division` | `Conformance` | lpg | `Unsupported` | 0.051 ms |
| `grafeo.spec.lpg.cypher.expressions.modulo` | `Conformance` | lpg | `Unsupported` | 0.044 ms |
| `grafeo.spec.lpg.cypher.expressions.power` | `Conformance` | lpg | `Unsupported` | 0.047 ms |
| `grafeo.spec.lpg.cypher.expressions.unary.minus` | `Conformance` | lpg | `Unsupported` | 0.030 ms |
| `grafeo.spec.lpg.cypher.expressions.string.concat` | `Conformance` | lpg | `Unsupported` | 0.065 ms |
| `grafeo.spec.lpg.cypher.expressions.equals` | `Conformance` | lpg | `Unsupported` | 0.066 ms |
| `grafeo.spec.lpg.cypher.expressions.not.equals` | `Conformance` | lpg | `Unsupported` | 0.065 ms |
| `grafeo.spec.lpg.cypher.expressions.less.than` | `Conformance` | lpg | `Unsupported` | 0.064 ms |
| `grafeo.spec.lpg.cypher.expressions.greater.equal` | `Conformance` | lpg | `Unsupported` | 0.064 ms |
| `grafeo.spec.lpg.cypher.expressions.starts.with` | `Conformance` | lpg | `Unsupported` | 0.039 ms |
| `grafeo.spec.lpg.cypher.expressions.ends.with` | `Conformance` | lpg | `Unsupported` | 0.038 ms |
| `grafeo.spec.lpg.cypher.expressions.contains` | `Conformance` | lpg | `Unsupported` | 0.037 ms |
| `grafeo.spec.lpg.cypher.expressions.in.list` | `Conformance` | lpg | `Unsupported` | 0.038 ms |
| `grafeo.spec.lpg.cypher.expressions.regex.match` | `Conformance` | lpg | `Unsupported` | 0.042 ms |
| `grafeo.spec.lpg.cypher.expressions.is.null` | `Conformance` | lpg | `Unsupported` | 0.038 ms |
| `grafeo.spec.lpg.cypher.expressions.is.not.null` | `Conformance` | lpg | `Unsupported` | 0.037 ms |
| `grafeo.spec.lpg.cypher.expressions.case.simple` | `Conformance` | lpg | `Unsupported` | 0.048 ms |
| `grafeo.spec.lpg.cypher.expressions.case.searched` | `Conformance` | lpg | `Unsupported` | 0.046 ms |
| `grafeo.spec.lpg.cypher.expressions.list.literal` | `Conformance` | lpg | `Unsupported` | 0.066 ms |
| `grafeo.spec.lpg.cypher.expressions.list.comprehension` | `Conformance` | lpg | `Unsupported` | 0.046 ms |
| `grafeo.spec.lpg.cypher.expressions.list.comprehension.filter.only` | `Conformance` | lpg | `Unsupported` | 0.043 ms |
| `grafeo.spec.lpg.cypher.expressions.list.slice` | `Conformance` | lpg | `Unsupported` | 0.068 ms |
| `grafeo.spec.lpg.cypher.expressions.index.access` | `Conformance` | lpg | `Unsupported` | 0.056 ms |
| `grafeo.spec.lpg.cypher.expressions.coalesce` | `Conformance` | lpg | `Unsupported` | 0.082 ms |
| `grafeo.spec.lpg.cypher.expressions.reduce` | `Conformance` | lpg | `Unsupported` | 0.068 ms |
| `grafeo.spec.lpg.cypher.expressions.all.predicate` | `Conformance` | lpg | `Unsupported` | 0.053 ms |
| `grafeo.spec.lpg.cypher.expressions.any.predicate` | `Conformance` | lpg | `Unsupported` | 0.052 ms |
| `grafeo.spec.lpg.cypher.expressions.none.predicate` | `Conformance` | lpg | `Unsupported` | 0.054 ms |
| `grafeo.spec.lpg.cypher.expressions.single.predicate` | `Conformance` | lpg | `Unsupported` | 0.053 ms |
| `grafeo.spec.lpg.cypher.expressions.any.with.labels.in.where` | `Conformance` | lpg | `Unsupported` | 0.047 ms |
| `grafeo.spec.lpg.cypher.expressions.comparison.in.return` | `Conformance` | lpg | `Unsupported` | 0.053 ms |
| `grafeo.spec.lpg.cypher.expressions.aggregate.comparison.in.return` | `Conformance` | lpg | `Unsupported` | 0.060 ms |
| `grafeo.spec.lpg.cypher.functions.id.of.node` | `Conformance` | lpg | `Unsupported` | 0.065 ms |
| `grafeo.spec.lpg.cypher.functions.labels.single` | `Conformance` | lpg | `Unsupported` | 0.067 ms |
| `grafeo.spec.lpg.cypher.functions.labels.multiple` | `Conformance` | lpg | `Unsupported` | 0.072 ms |
| `grafeo.spec.lpg.cypher.functions.type.of.relationship` | `Conformance` | lpg | `Unsupported` | 0.071 ms |
| `grafeo.spec.lpg.cypher.functions.keys.of.node` | `Conformance` | lpg | `Unsupported` | 0.076 ms |
| `grafeo.spec.lpg.cypher.functions.properties.of.node` | `Conformance` | lpg | `Unsupported` | 0.064 ms |
| `grafeo.spec.lpg.cypher.functions.exists.property.true` | `Conformance` | lpg | `Unsupported` | 0.071 ms |
| `grafeo.spec.lpg.cypher.functions.exists.property.false` | `Conformance` | lpg | `Unsupported` | 0.003 ms |
| `grafeo.spec.lpg.cypher.functions.head.of.list` | `Conformance` | lpg | `Unsupported` | 0.076 ms |
| `grafeo.spec.lpg.cypher.functions.last.of.list` | `Conformance` | lpg | `Unsupported` | 0.076 ms |
| `grafeo.spec.lpg.cypher.functions.tail.of.list` | `Conformance` | lpg | `Unsupported` | 0.078 ms |
| `grafeo.spec.lpg.cypher.functions.range.default.step` | `Conformance` | lpg | `Unsupported` | 0.059 ms |
| `grafeo.spec.lpg.cypher.functions.range.with.step` | `Conformance` | lpg | `Unsupported` | 0.065 ms |
| `grafeo.spec.lpg.cypher.functions.size.of.list` | `Conformance` | lpg | `Unsupported` | 0.077 ms |
| `grafeo.spec.lpg.cypher.functions.size.of.string` | `Conformance` | lpg | `Unsupported` | 0.059 ms |
| `grafeo.spec.lpg.cypher.functions.to.lower` | `Conformance` | lpg | `Unsupported` | 0.055 ms |
| `grafeo.spec.lpg.cypher.functions.to.upper` | `Conformance` | lpg | `Unsupported` | 0.055 ms |
| `grafeo.spec.lpg.cypher.functions.trim.whitespace` | `Conformance` | lpg | `Unsupported` | 0.058 ms |
| `grafeo.spec.lpg.cypher.functions.replace.substring` | `Conformance` | lpg | `Unsupported` | 0.074 ms |
| `grafeo.spec.lpg.cypher.functions.substring.from.start` | `Conformance` | lpg | `Unsupported` | 0.074 ms |
| `grafeo.spec.lpg.cypher.functions.substring.to.end` | `Conformance` | lpg | `Unsupported` | 0.068 ms |
| `grafeo.spec.lpg.cypher.functions.split.string` | `Conformance` | lpg | `Unsupported` | 0.065 ms |
| `grafeo.spec.lpg.cypher.functions.left.string` | `Conformance` | lpg | `Unsupported` | 0.063 ms |
| `grafeo.spec.lpg.cypher.functions.right.string` | `Conformance` | lpg | `Unsupported` | 0.067 ms |
| `grafeo.spec.lpg.cypher.functions.reverse.string` | `Conformance` | lpg | `Unsupported` | 0.058 ms |
| `grafeo.spec.lpg.cypher.functions.abs.positive` | `Conformance` | lpg | `Unsupported` | 0.054 ms |
| `grafeo.spec.lpg.cypher.functions.ceil.float` | `Conformance` | lpg | `Unsupported` | 0.056 ms |
| `grafeo.spec.lpg.cypher.functions.floor.float` | `Conformance` | lpg | `Unsupported` | 0.053 ms |
| `grafeo.spec.lpg.cypher.functions.round.float` | `Conformance` | lpg | `Unsupported` | 0.051 ms |
| `grafeo.spec.lpg.cypher.functions.sign.positive` | `Conformance` | lpg | `Unsupported` | 0.055 ms |
| `grafeo.spec.lpg.cypher.functions.sign.negative` | `Conformance` | lpg | `Unsupported` | 0.002 ms |
| `grafeo.spec.lpg.cypher.functions.sign.zero` | `Conformance` | lpg | `Unsupported` | 0.002 ms |
| `grafeo.spec.lpg.cypher.functions.sqrt.perfect.square` | `Conformance` | lpg | `Unsupported` | 0.048 ms |
| `grafeo.spec.lpg.cypher.functions.to.integer.from.string` | `Conformance` | lpg | `Unsupported` | 0.057 ms |
| `grafeo.spec.lpg.cypher.functions.to.integer.from.float` | `Conformance` | lpg | `Unsupported` | 0.052 ms |
| `grafeo.spec.lpg.cypher.functions.to.float.from.string` | `Conformance` | lpg | `Unsupported` | 0.058 ms |
| `grafeo.spec.lpg.cypher.functions.to.float.from.integer` | `Conformance` | lpg | `Unsupported` | 0.002 ms |
| `grafeo.spec.lpg.cypher.functions.to.string.from.integer` | `Conformance` | lpg | `Unsupported` | 0.058 ms |
| `grafeo.spec.lpg.cypher.functions.to.boolean.from.string.true` | `Conformance` | lpg | `Unsupported` | 0.058 ms |
| `grafeo.spec.lpg.cypher.functions.to.boolean.from.string.false` | `Conformance` | lpg | `Unsupported` | 0.002 ms |
| `grafeo.spec.lpg.cypher.functions.date.from.string` | `Conformance` | lpg | `Unsupported` | 0.054 ms |
| `grafeo.spec.lpg.cypher.functions.date.from.map` | `Conformance` | lpg | `Unsupported` | 0.051 ms |
| `grafeo.spec.lpg.cypher.functions.datetime.from.string` | `Conformance` | lpg | `Unsupported` | 0.052 ms |
| `grafeo.spec.lpg.cypher.functions.duration.from.string` | `Conformance` | lpg | `Unsupported` | 0.051 ms |
| `grafeo.spec.lpg.cypher.functions.path.length` | `Conformance` | lpg | `Unsupported` | 0.103 ms |
| `grafeo.spec.lpg.cypher.functions.path.length.single.hop` | `Conformance` | lpg | `Unsupported` | 0.085 ms |
| `grafeo.spec.lpg.cypher.functions.collect.names` | `Conformance` | lpg | `Unsupported` | 0.060 ms |
| `grafeo.spec.lpg.cypher.functions.collect.distinct` | `Conformance` | lpg | `Unsupported` | 0.069 ms |
| `grafeo.spec.lpg.cypher.functions.count.with.distinct` | `Conformance` | lpg | `Unsupported` | 0.059 ms |
| `grafeo.spec.lpg.cypher.functions.sum.values` | `Conformance` | lpg | `Unsupported` | 0.059 ms |
| `grafeo.spec.lpg.cypher.functions.avg.values` | `Conformance` | lpg | `Unsupported` | 0.058 ms |
| `grafeo.spec.lpg.cypher.functions.min.values` | `Conformance` | lpg | `Unsupported` | 0.058 ms |
| `grafeo.spec.lpg.cypher.functions.max.values` | `Conformance` | lpg | `Unsupported` | 0.058 ms |
| `grafeo.spec.lpg.cypher.functions.chained.string.functions` | `Conformance` | lpg | `Unsupported` | 0.068 ms |
| `grafeo.spec.lpg.cypher.functions.nested.list.functions` | `Conformance` | lpg | `Unsupported` | 0.088 ms |
| `grafeo.spec.lpg.cypher.functions.extended.log.of.e` | `Conformance` | lpg | `Unsupported` | 0.083 ms |
| `grafeo.spec.lpg.cypher.functions.extended.log10.of.100` | `Conformance` | lpg | `Unsupported` | 0.055 ms |
| `grafeo.spec.lpg.cypher.functions.extended.exp.of.zero` | `Conformance` | lpg | `Unsupported` | 0.052 ms |
| `grafeo.spec.lpg.cypher.functions.extended.e.constant` | `Conformance` | lpg | `Unsupported` | 0.059 ms |
| `grafeo.spec.lpg.cypher.functions.extended.pi.constant` | `Conformance` | lpg | `Unsupported` | 0.057 ms |
| `grafeo.spec.lpg.cypher.functions.extended.rand.in.range` | `Conformance` | lpg | `Unsupported` | 0.081 ms |
| `grafeo.spec.lpg.cypher.functions.extended.sin.of.zero` | `Conformance` | lpg | `Unsupported` | 0.051 ms |
| `grafeo.spec.lpg.cypher.functions.extended.cos.of.zero` | `Conformance` | lpg | `Unsupported` | 0.051 ms |
| `grafeo.spec.lpg.cypher.functions.extended.tan.of.zero` | `Conformance` | lpg | `Unsupported` | 0.051 ms |
| `grafeo.spec.lpg.cypher.functions.extended.asin.of.one` | `Conformance` | lpg | `Unsupported` | 0.059 ms |
| `grafeo.spec.lpg.cypher.functions.extended.acos.of.one` | `Conformance` | lpg | `Unsupported` | 0.051 ms |
| `grafeo.spec.lpg.cypher.functions.extended.atan.of.one` | `Conformance` | lpg | `Unsupported` | 0.059 ms |
| `grafeo.spec.lpg.cypher.functions.extended.atan2.unit` | `Conformance` | lpg | `Unsupported` | 0.067 ms |
| `grafeo.spec.lpg.cypher.functions.extended.degrees.from.pi` | `Conformance` | lpg | `Unsupported` | 0.057 ms |
| `grafeo.spec.lpg.cypher.functions.extended.radians.from.180` | `Conformance` | lpg | `Unsupported` | 0.056 ms |
| `grafeo.spec.lpg.cypher.functions.extended.ltrim.whitespace` | `Conformance` | lpg | `Unsupported` | 0.058 ms |
| `grafeo.spec.lpg.cypher.functions.extended.rtrim.whitespace` | `Conformance` | lpg | `Unsupported` | 0.058 ms |
| `grafeo.spec.lpg.cypher.functions.extended.char.length.string` | `Conformance` | lpg | `Unsupported` | 0.059 ms |
| `grafeo.spec.lpg.cypher.functions.extended.length.of.string` | `Conformance` | lpg | `Unsupported` | 0.060 ms |
| `grafeo.spec.lpg.cypher.functions.extended.reverse.list` | `Conformance` | lpg | `Unsupported` | 0.080 ms |
| `grafeo.spec.lpg.cypher.functions.extended.keys.of.map` | `Conformance` | lpg | `Unsupported` | 0.064 ms |
| `grafeo.spec.lpg.cypher.functions.extended.stdev.sample` | `Conformance` | lpg | `Unsupported` | 0.068 ms |
| `grafeo.spec.lpg.cypher.functions.extended.stdevp.population` | `Conformance` | lpg | `Unsupported` | 0.068 ms |
| `grafeo.spec.lpg.cypher.functions.extended.percentile.cont.median` | `Conformance` | lpg | `Unsupported` | 0.070 ms |
| `grafeo.spec.lpg.cypher.functions.extended.percentile.disc.median` | `Conformance` | lpg | `Unsupported` | 0.070 ms |
| `grafeo.spec.lpg.cypher.functions.extended.element.id.not.null` | `Conformance` | lpg | `Unsupported` | 0.056 ms |
| `grafeo.spec.lpg.cypher.functions.extended.count.star` | `Conformance` | lpg | `Unsupported` | 0.050 ms |
| `grafeo.spec.lpg.cypher.functions.extended.count.expr` | `Conformance` | lpg | `Unsupported` | 0.055 ms |
| `grafeo.spec.lpg.cypher.functions.extended.nodes.of.path` | `Conformance` | lpg | `Unsupported` | 0.095 ms |
| `grafeo.spec.lpg.cypher.functions.extended.relationships.of.path` | `Conformance` | lpg | `Unsupported` | 0.094 ms |
| `grafeo.spec.lpg.cypher.functions.extended.nodes.of.multi.hop.path` | `Conformance` | lpg | `Unsupported` | 0.117 ms |
| `grafeo.spec.lpg.cypher.functions.extended.relationships.of.multi.hop.path` | `Conformance` | lpg | `Unsupported` | 0.112 ms |
| `grafeo.spec.lpg.cypher.functions.extended.date.no.args` | `Conformance` | lpg | `Unsupported` | 0.051 ms |
| `grafeo.spec.lpg.cypher.functions.extended.now.returns.value` | `Conformance` | lpg | `Unsupported` | 0.049 ms |
| `grafeo.spec.lpg.cypher.functions.extended.year.accessor` | `Conformance` | lpg | `Unsupported` | 0.064 ms |
| `grafeo.spec.lpg.cypher.functions.extended.month.accessor` | `Conformance` | lpg | `Unsupported` | 0.064 ms |
| `grafeo.spec.lpg.cypher.functions.extended.day.accessor` | `Conformance` | lpg | `Unsupported` | 0.065 ms |
| `grafeo.spec.lpg.cypher.functions.extended.time.from.string` | `Conformance` | lpg | `Unsupported` | 0.051 ms |
| `grafeo.spec.lpg.cypher.functions.extended.duration.from.map` | `Conformance` | lpg | `Unsupported` | 0.057 ms |
| `grafeo.spec.lpg.cypher.patterns.anonymous.node` | `Conformance` | lpg | `Unsupported` | 0.047 ms |
| `grafeo.spec.lpg.cypher.patterns.variable.binding` | `Conformance` | lpg | `Unsupported` | 0.043 ms |
| `grafeo.spec.lpg.cypher.patterns.single.label` | `Conformance` | lpg | `Unsupported` | 0.046 ms |
| `grafeo.spec.lpg.cypher.patterns.multiple.labels` | `Conformance` | lpg | `Unsupported` | 0.050 ms |
| `grafeo.spec.lpg.cypher.patterns.property.filter` | `Conformance` | lpg | `Unsupported` | 0.060 ms |
| `grafeo.spec.lpg.cypher.patterns.outgoing.relationship` | `Conformance` | lpg | `Unsupported` | 0.073 ms |
| `grafeo.spec.lpg.cypher.patterns.incoming.relationship` | `Conformance` | lpg | `Unsupported` | 0.056 ms |
| `grafeo.spec.lpg.cypher.patterns.undirected.relationship` | `Conformance` | lpg | `Unsupported` | 0.082 ms |
| `grafeo.spec.lpg.cypher.patterns.multiple.relationship.types` | `Conformance` | lpg | `Unsupported` | 0.079 ms |
| `grafeo.spec.lpg.cypher.patterns.relationship.properties` | `Conformance` | lpg | `Unsupported` | 0.087 ms |
| `grafeo.spec.lpg.cypher.patterns.untyped.relationship` | `Conformance` | lpg | `Unsupported` | 0.081 ms |
| `grafeo.spec.lpg.cypher.patterns.anonymous.relationship` | `Conformance` | lpg | `Unsupported` | 0.066 ms |
| `grafeo.spec.lpg.cypher.patterns.variable.length.unbounded` | `Conformance` | lpg | `Unsupported` | 0.082 ms |
| `grafeo.spec.lpg.cypher.patterns.variable.length.exact` | `Conformance` | lpg | `Unsupported` | 0.073 ms |
| `grafeo.spec.lpg.cypher.patterns.variable.length.range` | `Conformance` | lpg | `Unsupported` | 0.074 ms |
| `grafeo.spec.lpg.cypher.patterns.variable.length.max.only` | `Conformance` | lpg | `Unsupported` | 0.073 ms |
| `grafeo.spec.lpg.cypher.patterns.variable.length.min.only` | `Conformance` | lpg | `Unsupported` | 0.073 ms |
| `grafeo.spec.lpg.cypher.patterns.path.alias` | `Conformance` | lpg | `Unsupported` | 0.000 ms |
| `grafeo.spec.lpg.cypher.patterns.shortest.path` | `Conformance` | lpg | `Unsupported` | 0.022 ms |
| `grafeo.spec.lpg.cypher.patterns.all.shortest.paths` | `Conformance` | lpg | `Unsupported` | 0.021 ms |
| `grafeo.spec.lpg.cypher.patterns.pattern.comprehension` | `Conformance` | lpg | `Unsupported` | 0.085 ms |
| `grafeo.spec.lpg.cypher.patterns.exists.subquery` | `Conformance` | lpg | `Unsupported` | 0.042 ms |
| `grafeo.spec.lpg.cypher.patterns.not.exists` | `Conformance` | lpg | `Unsupported` | 0.041 ms |
| `grafeo.spec.lpg.cypher.patterns.count.subquery` | `Conformance` | lpg | `Unsupported` | 0.042 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.single.node` | `Conformance` | lpg | `Unsupported` | 0.053 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.by.label` | `Conformance` | lpg | `Unsupported` | 0.000 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.by.property` | `Conformance` | lpg | `Unsupported` | 0.061 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.multi.label` | `Conformance` | lpg | `Unsupported` | 0.000 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.comma.patterns` | `Conformance` | lpg | `Unsupported` | 0.075 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.multiple.clauses` | `Conformance` | lpg | `Unsupported` | 0.077 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.edge.outgoing` | `Conformance` | lpg | `Unsupported` | 0.080 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.edge.incoming` | `Conformance` | lpg | `Unsupported` | 0.079 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.match.edge.undirected` | `Conformance` | lpg | `Unsupported` | 0.085 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.optional.match.with.result` | `Conformance` | lpg | `Unsupported` | 0.091 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.optional.match.null` | `Conformance` | lpg | `Unsupported` | 0.100 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.where.comparison` | `Conformance` | lpg | `Unsupported` | 0.070 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.where.and` | `Conformance` | lpg | `Unsupported` | 0.094 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.where.or` | `Conformance` | lpg | `Unsupported` | 0.094 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.where.not` | `Conformance` | lpg | `Unsupported` | 0.041 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.where.xor` | `Conformance` | lpg | `Unsupported` | 0.043 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.with.projection` | `Conformance` | lpg | `Unsupported` | 0.105 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.with.distinct` | `Conformance` | lpg | `Unsupported` | 0.072 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.with.where` | `Conformance` | lpg | `Unsupported` | 0.090 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.with.star` | `Conformance` | lpg | `Unsupported` | 0.053 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.unwind.list` | `Conformance` | lpg | `Passed` | 6.119 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.unwind.with.match` | `Conformance` | lpg | `Unsupported` | 0.109 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.union` | `Conformance` | lpg | `Unsupported` | 0.057 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.union.all` | `Conformance` | lpg | `Unsupported` | 0.053 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.call.db.labels` | `Conformance` | lpg | `Unsupported` | 0.009 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.call.db.relationship.types` | `Conformance` | lpg | `Unsupported` | 0.011 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.call.db.property.keys` | `Conformance` | lpg | `Unsupported` | 0.010 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.call.subquery.basic` | `Conformance` | lpg | `Unsupported` | 0.013 ms |
| `grafeo.spec.lpg.cypher.reading.clauses.call.subquery.with.outer.scope` | `Conformance` | lpg | `Unsupported` | 0.043 ms |
| `grafeo.spec.lpg.cypher.regression.not.exists.with.type.filter` | `Conformance` | lpg | `Unsupported` | 0.065 ms |
| `grafeo.spec.lpg.cypher.regression.sum.case.when` | `Conformance` | lpg | `Unsupported` | 0.110 ms |
| `grafeo.spec.lpg.cypher.regression.any.labels.in.list.matches` | `Conformance` | lpg | `Unsupported` | 0.047 ms |
| `grafeo.spec.lpg.cypher.regression.any.labels.in.list.no.match` | `Conformance` | lpg | `Unsupported` | 0.046 ms |
| `grafeo.spec.lpg.cypher.regression.any.with.single.match` | `Conformance` | lpg | `Unsupported` | 0.047 ms |
| `grafeo.spec.lpg.cypher.regression.reduce.with.case.max` | `Conformance` | lpg | `Unsupported` | 0.106 ms |
| `grafeo.spec.lpg.cypher.regression.reduce.with.case.min` | `Conformance` | lpg | `Unsupported` | 0.105 ms |
| `grafeo.spec.lpg.cypher.regression.reduce.with.case.conditional.sum` | `Conformance` | lpg | `Unsupported` | 0.103 ms |
| `grafeo.spec.lpg.cypher.regression.outgoing.target.property.filter` | `Conformance` | lpg | `Unsupported` | 0.083 ms |
| `grafeo.spec.lpg.cypher.regression.target.property.filter.count` | `Conformance` | lpg | `Unsupported` | 0.075 ms |
| `grafeo.spec.lpg.cypher.regression.target.property.filter.no.match` | `Conformance` | lpg | `Unsupported` | 0.076 ms |
| `grafeo.spec.lpg.cypher.regression.edge.property.filter` | `Conformance` | lpg | `Unsupported` | 0.079 ms |
| `grafeo.spec.lpg.cypher.regression.optional.match.count.preserves.all.rows` | `Conformance` | lpg | `Unsupported` | 0.098 ms |
| `grafeo.spec.lpg.cypher.regression.union.deduplicates` | `Conformance` | lpg | `Unsupported` | 0.028 ms |
| `grafeo.spec.lpg.cypher.regression.union.all.preserves` | `Conformance` | lpg | `Unsupported` | 0.027 ms |
| `grafeo.spec.lpg.cypher.regression.two.hop.equivalence` | `Conformance` | lpg | `Unsupported` | 0.072 ms |
| `grafeo.spec.lpg.cypher.regression.merge.creates.new.after.delete` | `Conformance` | lpg | `Unsupported` | 0.156 ms |
| `grafeo.spec.lpg.cypher.regression.replace.edge` | `Conformance` | lpg | `Unsupported` | 0.309 ms |
| `grafeo.spec.lpg.cypher.regression.backward.arrow.forward` | `Conformance` | lpg | `Unsupported` | 0.059 ms |
| `grafeo.spec.lpg.cypher.regression.backward.arrow.reverse` | `Conformance` | lpg | `Unsupported` | 0.059 ms |
| `grafeo.spec.lpg.cypher.regression.backward.arrow.wrong.direction` | `Conformance` | lpg | `Unsupported` | 0.058 ms |
| `grafeo.spec.lpg.cypher.regression.null.equals.null.is.unknown` | `Conformance` | lpg | `Unsupported` | 0.057 ms |
| `grafeo.spec.lpg.cypher.regression.null.is.null.is.true` | `Conformance` | lpg | `Unsupported` | 0.033 ms |
| `grafeo.spec.lpg.cypher.regression.bool.to.string` | `Conformance` | lpg | `Unsupported` | 0.118 ms |
| `grafeo.spec.lpg.cypher.regression.int.to.string` | `Conformance` | lpg | `Unsupported` | 0.115 ms |
| `grafeo.spec.lpg.cypher.regression.string.false.ne.bool.false` | `Conformance` | lpg | `Unsupported` | 0.066 ms |
| `grafeo.spec.lpg.cypher.regression.neq.excludes.null` | `Conformance` | lpg | `Unsupported` | 0.066 ms |
| `grafeo.spec.lpg.cypher.regression.skip.plus.limit` | `Conformance` | lpg | `Unsupported` | 0.084 ms |
| `grafeo.spec.lpg.cypher.regression.distinct.values` | `Conformance` | lpg | `Unsupported` | 0.062 ms |
| `grafeo.spec.lpg.cypher.regression.distinct.collapses.nulls` | `Conformance` | lpg | `Unsupported` | 0.043 ms |
| `grafeo.spec.lpg.cypher.regression.order.by.property.matching.return.alias.with.edge` | `Conformance` | lpg | `Unsupported` | 0.123 ms |
| `grafeo.spec.lpg.cypher.regression.order.by.desc.with.relationship.traversal` | `Conformance` | lpg | `Unsupported` | 0.120 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.expression` | `Conformance` | lpg | `Unsupported` | 0.060 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.alias` | `Conformance` | lpg | `Unsupported` | 0.054 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.distinct` | `Conformance` | lpg | `Unsupported` | 0.046 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.star` | `Conformance` | lpg | `Unsupported` | 0.027 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.count.star` | `Conformance` | lpg | `Unsupported` | 0.047 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.arithmetic` | `Conformance` | lpg | `Unsupported` | 0.000 ms |
| `grafeo.spec.lpg.cypher.return.ordering.return.boolean.expression` | `Conformance` | lpg | `Unsupported` | 0.000 ms |
| `grafeo.spec.lpg.cypher.return.ordering.order.by.asc` | `Conformance` | lpg | `Unsupported` | 0.061 ms |
| `grafeo.spec.lpg.cypher.return.ordering.order.by.desc` | `Conformance` | lpg | `Unsupported` | 0.061 ms |
| `grafeo.spec.lpg.cypher.return.ordering.order.by.multiple.keys` | `Conformance` | lpg | `Unsupported` | 0.077 ms |
| `grafeo.spec.lpg.cypher.return.ordering.limit` | `Conformance` | lpg | `Unsupported` | 0.053 ms |
| `grafeo.spec.lpg.cypher.return.ordering.skip` | `Conformance` | lpg | `Unsupported` | 0.069 ms |
| `grafeo.spec.lpg.cypher.return.ordering.skip.and.limit` | `Conformance` | lpg | `Unsupported` | 0.078 ms |
| `grafeo.spec.lpg.cypher.types.integer.decimal` | `Conformance` | lpg | `Unsupported` | 0.041 ms |
| `grafeo.spec.lpg.cypher.types.integer.negative` | `Conformance` | lpg | `Unsupported` | 0.040 ms |
| `grafeo.spec.lpg.cypher.types.integer.zero` | `Conformance` | lpg | `Unsupported` | 0.039 ms |
| `grafeo.spec.lpg.cypher.types.integer.hex` | `Conformance` | lpg | `Unsupported` | 0.037 ms |
| `grafeo.spec.lpg.cypher.types.integer.octal` | `Conformance` | lpg | `Unsupported` | 0.036 ms |
| `grafeo.spec.lpg.cypher.types.float.decimal` | `Conformance` | lpg | `Unsupported` | 0.039 ms |
| `grafeo.spec.lpg.cypher.types.float.scientific` | `Conformance` | lpg | `Unsupported` | 0.036 ms |
| `grafeo.spec.lpg.cypher.types.float.negative` | `Conformance` | lpg | `Unsupported` | 0.039 ms |
| `grafeo.spec.lpg.cypher.types.string.single.quoted` | `Conformance` | lpg | `Unsupported` | 0.040 ms |
| `grafeo.spec.lpg.cypher.types.string.double.quoted` | `Conformance` | lpg | `Unsupported` | 0.031 ms |
| `grafeo.spec.lpg.cypher.types.string.empty` | `Conformance` | lpg | `Unsupported` | 0.040 ms |
| `grafeo.spec.lpg.cypher.types.boolean.true` | `Conformance` | lpg | `Unsupported` | 0.039 ms |
| `grafeo.spec.lpg.cypher.types.boolean.false` | `Conformance` | lpg | `Unsupported` | 0.040 ms |
| `grafeo.spec.lpg.cypher.types.null.literal` | `Conformance` | lpg | `Unsupported` | 0.040 ms |
| `grafeo.spec.lpg.cypher.types.null.comparison.is.null` | `Conformance` | lpg | `Unsupported` | 0.038 ms |
| `grafeo.spec.lpg.cypher.types.null.comparison.is.not.null` | `Conformance` | lpg | `Unsupported` | 0.038 ms |
| `grafeo.spec.lpg.cypher.types.null.equality.returns.null` | `Conformance` | lpg | `Unsupported` | 0.044 ms |
| `grafeo.spec.lpg.cypher.types.missing.property.is.null` | `Conformance` | lpg | `Unsupported` | 0.047 ms |
| `grafeo.spec.lpg.cypher.types.list.of.integers` | `Conformance` | lpg | `Unsupported` | 0.000 ms |
| `grafeo.spec.lpg.cypher.types.list.empty` | `Conformance` | lpg | `Unsupported` | 0.046 ms |
| `grafeo.spec.lpg.cypher.types.list.nested` | `Conformance` | lpg | `Unsupported` | 0.090 ms |
| `grafeo.spec.lpg.cypher.types.list.size` | `Conformance` | lpg | `Unsupported` | 0.086 ms |
| `grafeo.spec.lpg.cypher.types.map.literal` | `Conformance` | lpg | `Unsupported` | 0.034 ms |
| `grafeo.spec.lpg.cypher.types.map.key.count` | `Conformance` | lpg | `Unsupported` | 0.059 ms |
| `grafeo.spec.lpg.cypher.types.node.return` | `Conformance` | lpg | `Unsupported` | 0.040 ms |
| `grafeo.spec.lpg.cypher.types.relationship.return` | `Conformance` | lpg | `Unsupported` | 0.049 ms |
| `grafeo.spec.lpg.cypher.types.path.return` | `Conformance` | lpg | `Unsupported` | 0.059 ms |
| `grafeo.spec.lpg.cypher.types.date.from.string` | `Conformance` | lpg | `Unsupported` | 0.051 ms |
| `grafeo.spec.lpg.cypher.types.time.from.string` | `Conformance` | lpg | `Unsupported` | 0.051 ms |
| `grafeo.spec.lpg.cypher.types.datetime.from.string` | `Conformance` | lpg | `Unsupported` | 0.052 ms |
| `grafeo.spec.lpg.cypher.types.duration.from.string` | `Conformance` | lpg | `Unsupported` | 0.000 ms |
| `grafeo.spec.lpg.cypher.types.date.stored.as.property` | `Conformance` | lpg | `Unsupported` | 0.059 ms |
| `grafeo.spec.lpg.cypher.types.integer.to.float.arithmetic` | `Conformance` | lpg | `Unsupported` | 0.047 ms |
| `grafeo.spec.lpg.cypher.types.to.integer.truncation` | `Conformance` | lpg | `Unsupported` | 0.049 ms |
| `grafeo.spec.lpg.cypher.types.to.float.from.integer` | `Conformance` | lpg | `Unsupported` | 0.050 ms |
| `grafeo.spec.lpg.cypher.types.to.string.from.boolean` | `Conformance` | lpg | `Unsupported` | 0.053 ms |
| `grafeo.spec.lpg.cypher.types.to.boolean.from.string.false` | `Conformance` | lpg | `Unsupported` | 0.000 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.create.node` | `Conformance` | lpg | `Unsupported` | 0.061 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.create.node.multi.label` | `Conformance` | lpg | `Unsupported` | 0.063 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.create.relationship` | `Conformance` | lpg | `Unsupported` | 0.135 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.create.relationship.with.properties` | `Conformance` | lpg | `Unsupported` | 0.132 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.create.path.pattern` | `Conformance` | lpg | `Unsupported` | 0.159 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.delete.node` | `Conformance` | lpg | `Unsupported` | 0.120 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.delete.multiple` | `Conformance` | lpg | `Unsupported` | 0.041 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.detach.delete` | `Conformance` | lpg | `Unsupported` | 0.114 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.detach.delete.with.return` | `Conformance` | lpg | `Unsupported` | 0.083 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.property` | `Conformance` | lpg | `Unsupported` | 0.062 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.multiple.properties` | `Conformance` | lpg | `Unsupported` | 0.148 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.replace.all` | `Conformance` | lpg | `Unsupported` | 0.098 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.merge.map` | `Conformance` | lpg | `Unsupported` | 0.044 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.label` | `Conformance` | lpg | `Unsupported` | 0.039 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.multiple.labels` | `Conformance` | lpg | `Unsupported` | 0.039 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.set.label.preserves.variable.binding` | `Conformance` | lpg | `Unsupported` | 0.044 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.count.star.after.set.label` | `Conformance` | lpg | `Unsupported` | 0.029 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.count.var.after.set.label` | `Conformance` | lpg | `Unsupported` | 0.028 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.remove.property` | `Conformance` | lpg | `Unsupported` | 0.052 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.remove.label` | `Conformance` | lpg | `Unsupported` | 0.075 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.remove.label.preserves.variable.binding` | `Conformance` | lpg | `Unsupported` | 0.046 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.match.create.edge.no.phantoms` | `Conformance` | lpg | `Unsupported` | 0.126 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.match.create.edge.correct.endpoints` | `Conformance` | lpg | `Unsupported` | 0.085 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.create` | `Conformance` | lpg | `Unsupported` | 0.088 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.match` | `Conformance` | lpg | `Unsupported` | 0.006 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.on.create.set` | `Conformance` | lpg | `Unsupported` | 0.040 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.on.match.set` | `Conformance` | lpg | `Unsupported` | 0.083 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.on.match.set.self.reference.increment` | `Conformance` | lpg | `Unsupported` | 0.042 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.on.create.set.self.reference.coalesce` | `Conformance` | lpg | `Unsupported` | 0.043 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.relationship` | `Conformance` | lpg | `Unsupported` | 0.161 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.merge.relationship.set` | `Conformance` | lpg | `Unsupported` | 0.212 ms |
| `grafeo.spec.lpg.cypher.writing.clauses.foreach.create` | `Conformance` | lpg | `Unsupported` | 0.065 ms |
| `grafeo.spec.regression.edge.cases.cyclic.vlp.same.source.and.target.variable.cypher-variant` | `Conformance` | regression | `Unsupported` | 0.104 ms |
| `grafeo.spec.regression.edge.cases.cyclic.vlp.no.cycle.returns.empty.cypher-variant` | `Conformance` | regression | `Unsupported` | 0.082 ms |
| `grafeo.spec.rosetta.aggregation.count.products.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.054 ms |
| `grafeo.spec.rosetta.aggregation.sum.order.totals.cypher.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.023 ms |
| `grafeo.spec.rosetta.aggregation.avg.product.price.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.064 ms |
| `grafeo.spec.rosetta.aggregation.min.max.price.cypher.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.085 ms |
| `grafeo.spec.rosetta.aggregation.count.by.status.cypher.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.020 ms |
| `grafeo.spec.rosetta.aggregation.orders.per.customer.cypher.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.040 ms |
| `grafeo.spec.rosetta.aggregation.total.spend.per.customer.cypher.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.043 ms |
| `grafeo.spec.rosetta.aggregation.customers.with.multiple.orders.cypher.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.042 ms |
| `grafeo.spec.rosetta.aggregation.avg.review.rating.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.148 ms |
| `grafeo.spec.rosetta.basic.queries.count.all.nodes.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.047 ms |
| `grafeo.spec.rosetta.basic.queries.match.by.label.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.002 ms |
| `grafeo.spec.rosetta.basic.queries.filter.by.age.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.003 ms |
| `grafeo.spec.rosetta.basic.queries.edge.traversal.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.073 ms |
| `grafeo.spec.rosetta.basic.queries.two.hop.path.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.096 ms |
| `grafeo.spec.rosetta.basic.queries.aggregation.group.by.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.128 ms |
| `grafeo.spec.rosetta.crud.operations.create.node.and.count.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.003 ms |
| `grafeo.spec.rosetta.crud.operations.create.node.read.properties.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.098 ms |
| `grafeo.spec.rosetta.crud.operations.create.edge.and.traverse.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.099 ms |
| `grafeo.spec.rosetta.crud.operations.match.count.multiple.nodes.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.003 ms |
| `grafeo.spec.rosetta.crud.operations.set.property.and.read.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.003 ms |
| `grafeo.spec.rosetta.crud.operations.delete.node.and.count.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.002 ms |
| `grafeo.spec.rosetta.crud.operations.aggregate.sum.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.058 ms |
| `grafeo.spec.rosetta.crud.operations.aggregate.count.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.053 ms |
| `grafeo.spec.rosetta.crud.operations.aggregate.avg.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.057 ms |
| `grafeo.spec.rosetta.data.fidelity.gql.insert.cypher.read.name.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.089 ms |
| `grafeo.spec.rosetta.data.fidelity.gql.insert.cypher.read.count.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.053 ms |
| `grafeo.spec.rosetta.data.fidelity.gql.insert.cypher.read.edge.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.000 ms |
| `grafeo.spec.rosetta.data.fidelity.int.property.preserved.in.cypher.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.043 ms |
| `grafeo.spec.rosetta.data.fidelity.bool.property.preserved.in.cypher.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.043 ms |
| `grafeo.spec.rosetta.data.fidelity.string.property.preserved.in.cypher.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.042 ms |
| `grafeo.spec.rosetta.data.fidelity.missing.property.null.in.cypher.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.042 ms |
| `grafeo.spec.rosetta.data.fidelity.multi.label.visible.in.cypher.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.046 ms |
| `grafeo.spec.rosetta.data.fidelity.edge.type.in.cypher.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.053 ms |
| `grafeo.spec.rosetta.data.fidelity.gql.insert.all.read.count.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.002 ms |
| `grafeo.spec.rosetta.pattern.matching.count.actors.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.053 ms |
| `grafeo.spec.rosetta.pattern.matching.find.actor.by.name.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.058 ms |
| `grafeo.spec.rosetta.pattern.matching.actors.in.heist.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.095 ms |
| `grafeo.spec.rosetta.pattern.matching.genres.of.vincent.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.112 ms |
| `grafeo.spec.rosetta.pattern.matching.movies.per.director.cypher.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.104 ms |
| `grafeo.spec.rosetta.pattern.matching.actor.roles.in.movie.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.111 ms |
| `grafeo.spec.rosetta.pattern.matching.high.rated.movies.cypher-variant` | `Conformance` | rosetta | `Unsupported` | 0.085 ms |

## Latest `ladybug-deep` run

- Run: `20260718T022017.653879Z-8d6f4f6d535d-corpus-deep`
- Commit: `8d6f4f6d535d00d5364f23df8553956f052df599`
- Package: `0.7.0`
- Environment: `macos/aarch64` (`dev`)
- Records: 15940
- Passed: 1104
- Unsupported: 14836
- Failed or changed: 0

### Results by source area

| Area | Outcome | Count |
|---|---|---:|
| acc | `unsupported` | 6 |
| agg | `passed` | 10 |
| agg | `unsupported` | 240 |
| arithmetic | `passed` | 121 |
| arithmetic | `unsupported` | 259 |
| binary_demo | `unsupported` | 101 |
| cast | `passed` | 272 |
| cast | `unsupported` | 202 |
| comment | `unsupported` | 9 |
| common | `passed` | 20 |
| common | `unsupported` | 62 |
| copy | `passed` | 51 |
| copy | `unsupported` | 576 |
| csv | `passed` | 22 |
| csv | `unsupported` | 66 |
| cyclic | `unsupported` | 22 |
| cypherlogic | `unsupported` | 576 |
| ddl | `passed` | 44 |
| ddl | `unsupported` | 398 |
| demo_db | `passed` | 6 |
| demo_db | `unsupported` | 184 |
| dml_node | `passed` | 6 |
| dml_node | `unsupported` | 706 |
| dml_rel | `passed` | 11 |
| dml_rel | `unsupported` | 1494 |
| exceptions | `passed` | 159 |
| exceptions | `unsupported` | 256 |
| explain | `unsupported` | 26 |
| extension | `passed` | 9 |
| extension | `unsupported` | 9 |
| filter | `unsupported` | 112 |
| function | `passed` | 86 |
| function | `unsupported` | 2133 |
| generic_hash_join | `unsupported` | 14 |
| glob | `unsupported` | 3 |
| graph | `passed` | 16 |
| graph | `unsupported` | 83 |
| hint | `passed` | 7 |
| hint | `unsupported` | 3 |
| ice_disk | `passed` | 5 |
| ice_disk | `unsupported` | 23 |
| issue | `passed` | 8 |
| issue | `unsupported` | 515 |
| ldbc | `unsupported` | 26 |
| load_from | `passed` | 12 |
| load_from | `unsupported` | 26 |
| lsqb | `unsupported` | 18 |
| match | `unsupported` | 61 |
| md5testing | `unsupported` | 16 |
| nested_types | `passed` | 3 |
| nested_types | `unsupported` | 18 |
| npy_1d | `unsupported` | 4 |
| optional_match | `unsupported` | 26 |
| order_by | `unsupported` | 65 |
| parquet | `passed` | 2 |
| parquet | `unsupported` | 17 |
| path | `unsupported` | 16 |
| projection | `passed` | 20 |
| projection | `unsupported` | 123 |
| read_list | `unsupported` | 26 |
| reader | `unsupported` | 5 |
| recursive_join | `passed` | 13 |
| recursive_join | `unsupported` | 117 |
| rel_group | `passed` | 16 |
| rel_group | `unsupported` | 79 |
| shortest_path | `unsupported` | 29 |
| storage_version.test | `unsupported` | 7 |
| subquery | `unsupported` | 27 |
| tck | `passed` | 117 |
| tck | `unsupported` | 2251 |
| tensor_list | `unsupported` | 2 |
| transaction | `passed` | 51 |
| transaction | `unsupported` | 3607 |
| transfer_demo | `unsupported` | 10 |
| uint128 | `passed` | 10 |
| uint128 | `unsupported` | 82 |
| unwind | `passed` | 6 |
| unwind | `unsupported` | 43 |
| user_defined_types | `passed` | 1 |
| user_defined_types | `unsupported` | 57 |

### Results by execution boundary

| Boundary | Outcome | Count |
|---|---|---:|
| `adapter` | `unsupported` | 2756 |
| `deduplicated` | `passed` | 34 |
| `deduplicated` | `unsupported` | 5317 |
| `parser` | `passed` | 1070 |
| `parser` | `unsupported` | 6763 |

### Failures (0)

- None.

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

## Latest `sparrowdb-deep` run

- Run: `20260718T022017.653879Z-8d6f4f6d535d-corpus-deep`
- Commit: `8d6f4f6d535d00d5364f23df8553956f052df599`
- Package: `0.7.0`
- Environment: `macos/aarch64` (`dev`)
- Records: 2253
- Passed: 0
- Unsupported: 2253
- Failed or changed: 0

### Results by source area

| Area | Outcome | Count |
|---|---|---:|
| acceptance | `unsupported` | 38 |
| call_subquery | `unsupported` | 18 |
| cypher_range_function_test | `unsupported` | 3 |
| debug_case_when | `unsupported` | 4 |
| debug_so_subclass | `unsupported` | 2 |
| delete_edge | `unsupported` | 7 |
| export_import | `unsupported` | 19 |
| fts_index | `unsupported` | 8 |
| gap_10_parameterized_queries | `unsupported` | 19 |
| hybrid_search | `unsupported` | 5 |
| match_after_create | `unsupported` | 7 |
| match_property_index | `unsupported` | 13 |
| mcp_cypher_templates | `unsupported` | 19 |
| merge_node | `unsupported` | 4 |
| path_semantics | `unsupported` | 5 |
| property_range_index | `unsupported` | 6 |
| readtx_query | `unsupported` | 23 |
| regression_355 | `unsupported` | 9 |
| regression_363 | `unsupported` | 13 |
| regression_364 | `unsupported` | 3 |
| regression_366 | `unsupported` | 4 |
| regression_367 | `unsupported` | 10 |
| regression_368 | `unsupported` | 9 |
| regression_369 | `unsupported` | 4 |
| regression_372 | `unsupported` | 11 |
| regression_373 | `unsupported` | 18 |
| regression_379 | `unsupported` | 31 |
| regression_380 | `unsupported` | 15 |
| regression_406 | `unsupported` | 14 |
| regression_real_world | `unsupported` | 23 |
| reverse_arrow_294 | `unsupported` | 14 |
| spa157_cypher_mutations | `unsupported` | 5 |
| spa163_164_read_path | `unsupported` | 3 |
| spa191_rel_type_persistence | `unsupported` | 17 |
| spa_100_order_by_spill | `unsupported` | 6 |
| spa_111_ldbc_snb | `unsupported` | 2 |
| spa_119_compat_fixture | `unsupported` | 4 |
| spa_130_with_clause | `unsupported` | 9 |
| spa_131_optional_match | `unsupported` | 20 |
| spa_132_union | `unsupported` | 20 |
| spa_134_multi_clause | `unsupported` | 19 |
| spa_136_shortest_path | `unsupported` | 13 |
| spa_137_exists_subquery | `unsupported` | 12 |
| spa_138_case_when | `unsupported` | 9 |
| spa_139_phase9_path_acceptance | `unsupported` | 40 |
| spa_140_143_functions | `unsupported` | 28 |
| spa_148_import_bridge | `unsupported` | 11 |
| spa_149_visualizer | `unsupported` | 14 |
| spa_151_kms_query_validation | `unsupported` | 90 |
| spa_155_unwind_param | `unsupported` | 3 |
| spa_156_161 | `unsupported` | 16 |
| spa_165_col_prefix_property | `unsupported` | 9 |
| spa_168_degree_cache_wiring | `unsupported` | 16 |
| spa_168_match_create | `unsupported` | 9 |
| spa_169_string_props | `unsupported` | 19 |
| spa_172_count_distinct | `unsupported` | 17 |
| spa_178_edge_properties | `unsupported` | 27 |
| spa_182_create_path_rhs | `unsupported` | 5 |
| spa_183_match_create_bindings | `unsupported` | 16 |
| spa_185_rel_table_id | `unsupported` | 24 |
| spa_186_csr_nodeid | `unsupported` | 13 |
| spa_187_column_slot_alignment | `unsupported` | 17 |
| spa_188_two_hop_where | `unsupported` | 32 |
| spa_189_checkpoint_optimize | `unsupported` | 10 |
| spa_192_match_no_label | `unsupported` | 17 |
| spa_193_undirected_pattern | `unsupported` | 12 |
| spa_194_count_node_var | `unsupported` | 12 |
| spa_195_type_function | `unsupported` | 14 |
| spa_196_id_function | `unsupported` | 14 |
| spa_197_count_label_fastpath | `unsupported` | 16 |
| spa_197_missing_prop_null | `unsupported` | 7 |
| spa_198_limit_pushdown | `unsupported` | 8 |
| spa_198_unlabeled_rel_endpoint | `unsupported` | 8 |
| spa_199_bfs_early_exit | `unsupported` | 6 |
| spa_200_batch_hop_perf | `unsupported` | 17 |
| spa_201_csr_backward | `unsupported` | 32 |
| spa_206_contains_predicate | `unsupported` | 16 |
| spa_206_mlm_benchmark | `unsupported` | 1 |
| spa_207_labels_function | `unsupported` | 13 |
| spa_207_null_sentinel | `unsupported` | 16 |
| spa_208_reserved_labels | `unsupported` | 10 |
| spa_208_string_heap | `unsupported` | 13 |
| spa_209_schema_introspection | `unsupported` | 19 |
| spa_211_unlabeled_match_create | `unsupported` | 14 |
| spa_212_string_truncation | `unsupported` | 18 |
| spa_213_return_node_var | `unsupported` | 9 |
| spa_214_skip_clause | `unsupported` | 6 |
| spa_215_merge_return | `unsupported` | 7 |
| spa_216_delete_node | `unsupported` | 20 |
| spa_217_info_counts | `unsupported` | 13 |
| spa_222_csr_lazy_load | `unsupported` | 1 |
| spa_224_regression_no_so_label | `unsupported` | 4 |
| spa_224_varpath_reserved_label | `unsupported` | 9 |
| spa_229_add_property | `unsupported` | 13 |
| spa_229_edge_prop_float | `unsupported` | 15 |
| spa_233_merge_relationship | `unsupported` | 9 |
| spa_235_234_create_index_constraint | `unsupported` | 21 |
| spa_236_labels_predicate | `unsupported` | 19 |
| spa_237_unwind_match | `unsupported` | 19 |
| spa_240_coalesce | `unsupported` | 11 |
| spa_241_multihop_props | `unsupported` | 15 |
| spa_242_count_rel_var | `unsupported` | 16 |
| spa_243_create_entity | `unsupported` | 10 |
| spa_244_mcp_errors | `unsupported` | 5 |
| spa_245_unknown_label_returns_empty | `unsupported` | 10 |
| spa_249_property_index | `unsupported` | 37 |
| spa_250_batch_write | `unsupported` | 2 |
| spa_251_text_search_index | `unsupported` | 30 |
| spa_252_three_hop_binding | `unsupported` | 15 |
| spa_254_query_timeout | `unsupported` | 2 |
| spa_259_inline_prop_filter | `unsupported` | 10 |
| spa_261_edge_props_perf | `unsupported` | 11 |
| spa_263_two_hop_agg | `unsupported` | 23 |
| spa_263_two_hop_null | `unsupported` | 25 |
| spa_264_boolean_props | `unsupported` | 14 |
| spa_265_backtick_escaping | `unsupported` | 27 |
| spa_266_265_bugs | `unsupported` | 6 |
| spa_267_float_codec | `unsupported` | 17 |
| spa_268_bfs_bugs | `unsupported` | 21 |
| spa_272_degree_cache | `unsupported` | 11 |
| spa_272_q7_count_fastpath | `unsupported` | 24 |
| spa_272_q7_cypher_wiring | `unsupported` | 24 |
| spa_273_planner_stats | `unsupported` | 22 |
| spa_289_multi_label | `unsupported` | 28 |
| spa_296_bulk_loader | `unsupported` | 1 |
| spa_299_chunked_pipeline | `unsupported` | 31 |
| spa_299_phase2_parity | `unsupported` | 35 |
| spa_299_phase3_parity | `unsupported` | 66 |
| spa_299_phase4_parity | `unsupported` | 66 |
| spa_306_constraint_persistence | `unsupported` | 15 |
| spa_354_varlength_terminal_label | `unsupported` | 24 |
| spa_98_wal_encryption | `unsupported` | 2 |
| spa_aggregation | `unsupported` | 26 |
| spa_collect_agg | `unsupported` | 16 |
| spa_datetime_fns | `unsupported` | 7 |
| spa_fulltext | `unsupported` | 7 |
| spa_in_operator | `unsupported` | 16 |
| spa_is_null | `unsupported` | 15 |
| spa_list_predicates | `unsupported` | 38 |
| spa_type_labels | `unsupported` | 18 |
| spa_variable_paths | `unsupported` | 37 |
| test_pole | `unsupported` | 8 |
| test_reactome | `unsupported` | 5 |
| uc1_social_graph | `unsupported` | 2 |
| uc7_unwind | `unsupported` | 7 |
| uc_tracing | `unsupported` | 1 |
| vector_index | `unsupported` | 13 |

### Results by execution boundary

| Boundary | Outcome | Count |
|---|---|---:|
| `adapter` | `unsupported` | 1173 |
| `deduplicated` | `unsupported` | 906 |
| `parser` | `unsupported` | 174 |

### Failures (0)

- None.

## Latest `tck-deep` run

- Run: `20260718T022017.653879Z-8d6f4f6d535d-corpus-deep`
- Commit: `8d6f4f6d535d00d5364f23df8553956f052df599`
- Package: `0.7.0`
- Environment: `macos/aarch64` (`dev`)
- Records: 3926
- Passed: 364
- Unsupported: 3542
- Failed or changed: 20

### Results by source area

| Area | Outcome | Count |
|---|---|---:|
| clauses | `failed` | 8 |
| clauses | `passed` | 88 |
| clauses | `unsupported` | 1155 |
| expressions | `failed` | 12 |
| expressions | `passed` | 276 |
| expressions | `unsupported` | 2357 |
| useCases | `unsupported` | 30 |

### Results by execution boundary

| Boundary | Outcome | Count |
|---|---|---:|
| `adapter` | `unsupported` | 940 |
| `deduplicated` | `passed` | 4 |
| `deduplicated` | `unsupported` | 8 |
| `parser` | `passed` | 260 |
| `parser` | `unsupported` | 2353 |
| `scalar-execution` | `failed` | 20 |
| `scalar-execution` | `passed` | 100 |
| `scalar-execution` | `unsupported` | 241 |

### Failures (20)

- `tck.clauses.return-orderby.returnorderby1.scenario-1`: expected [["false"], ["true"]], observed [["0"], ["1"]]
- `tck.clauses.return-orderby.returnorderby1.scenario-2`: expected [["true"], ["false"]], observed [["1"], ["0"]]
- `tck.clauses.return-orderby.returnorderby1.scenario-9`: expected [["[]"], ["[\"a\"]"], ["[\"a\",1]"], ["[1]"], ["[1,\"a\"]"], ["[1,null]"], ["[null,1]"], ["[null,2]"]], observed [["[\"a\",1]"], ["[\"a\"]"], ["[1,\"a\"]"], ["[1,null]"], ["[1]"], ["[]"], ["[null,1]"], ["[null,2]"]]
- `tck.clauses.return-orderby.returnorderby1.scenario-10`: expected [["[null,2]"], ["[null,1]"], ["[1,null]"], ["[1,\"a\"]"], ["[1]"], ["[\"a\",1]"], ["[\"a\"]"], ["[]"]], observed [["[null,2]"], ["[null,1]"], ["[]"], ["[1]"], ["[1,null]"], ["[1,\"a\"]"], ["[\"a\"]"], ["[\"a\",1]"]]
- `tck.clauses.with-orderby.withorderby1.scenario-1`: expected [["false"]], observed [["0"]]
- `tck.clauses.with-orderby.withorderby1.scenario-2`: expected [["true"]], observed [["1"]]
- `tck.clauses.with-orderby.withorderby1.scenario-9`: expected [["[]"], ["[\"a\"]"], ["[\"a\",1]"], ["[1]"]], observed [["[\"a\",1]"], ["[\"a\"]"], ["[1,\"a\"]"], ["[1,null]"]]
- `tck.clauses.with-orderby.withorderby1.scenario-10`: expected [["[null,2]"], ["[null,1]"], ["[1,null]"], ["[1,\"a\"]"]], observed [["[null,2]"], ["[null,1]"], ["[]"], ["[1]"]]
- `tck.expressions.aggregation.aggregation2.scenario-3`: expected [["2.0"]], observed [["2"]]
- `tck.expressions.aggregation.aggregation2.scenario-9`: expected [["[2,1]"]], observed [["[2]"]]
- `tck.expressions.aggregation.aggregation2.scenario-11`: expected [["1"]], observed [["b"]]
- `tck.expressions.aggregation.aggregation2.scenario-12`: expected [["[1,2]"]], observed [["0.2"]]
- `tck.expressions.boolean.boolean1.scenario-4`: expected [["false", "false", "true"], ["false", "true", "true"], ["true", "false", "true"], ["true", "true", "true"]], observed [["0", "0", "1"], ["0", "1", "1"], ["1", "0", "1"], ["1", "1", "1"]]
- `tck.expressions.boolean.boolean1.scenario-6`: expected [["false", "false", "false", "true"], ["false", "false", "true", "true"], ["false", "true", "false", "true"], ["false", "true", "true", "true"], ["true", "false", "false", "true"], ["true", "false", "true", "true"], ["true", "true", "false", "true"], ["true", "true", "true", "true"]], observed [["0", "0", "0", "1"], ["0", "0", "1", "1"], ["0", "1", "0", "1"], ["0", "1", "1", "1"], ["1", "0", "0", "1"], ["1", "0", "1", "1"], ["1", "1", "0", "1"], ["1", "1", "1", "1"]]
- `tck.expressions.boolean.boolean2.scenario-4`: expected [["false", "false", "true"], ["false", "true", "true"], ["true", "false", "true"], ["true", "true", "true"]], observed [["0", "0", "1"], ["0", "1", "1"], ["1", "0", "1"], ["1", "1", "1"]]
- `tck.expressions.boolean.boolean2.scenario-6`: expected [["false", "false", "false", "true"], ["false", "false", "true", "true"], ["false", "true", "false", "true"], ["false", "true", "true", "true"], ["true", "false", "false", "true"], ["true", "false", "true", "true"], ["true", "true", "false", "true"], ["true", "true", "true", "true"]], observed [["0", "0", "0", "1"], ["0", "0", "1", "1"], ["0", "1", "0", "1"], ["0", "1", "1", "1"], ["1", "0", "0", "1"], ["1", "0", "1", "1"], ["1", "1", "0", "1"], ["1", "1", "1", "1"]]
- `tck.expressions.boolean.boolean5.scenario-1`: expected [["false", "false", "false", "true"], ["false", "false", "true", "true"], ["false", "true", "false", "true"], ["false", "true", "true", "true"], ["true", "false", "false", "true"], ["true", "false", "true", "true"], ["true", "true", "false", "true"], ["true", "true", "true", "true"]], observed [["0", "0", "0", "1"], ["0", "0", "1", "1"], ["0", "1", "0", "1"], ["0", "1", "1", "1"], ["1", "0", "0", "1"], ["1", "0", "1", "1"], ["1", "1", "0", "1"], ["1", "1", "1", "1"]]
- `tck.expressions.boolean.boolean5.scenario-3`: expected [["false", "false", "false", "true"], ["false", "false", "true", "true"], ["false", "true", "false", "true"], ["false", "true", "true", "true"], ["true", "false", "false", "true"], ["true", "false", "true", "true"], ["true", "true", "false", "true"], ["true", "true", "true", "true"]], observed [["0", "0", "0", "1"], ["0", "0", "1", "1"], ["0", "1", "0", "1"], ["0", "1", "1", "1"], ["1", "0", "0", "1"], ["1", "0", "1", "1"], ["1", "1", "0", "1"], ["1", "1", "1", "1"]]
- `tck.expressions.boolean.boolean5.scenario-7`: expected [["false", "false", "true"], ["false", "true", "true"], ["true", "false", "true"], ["true", "true", "true"]], observed [["0", "0", "1"], ["0", "1", "1"], ["1", "0", "1"], ["1", "1", "1"]]
- `tck.expressions.boolean.boolean5.scenario-8`: expected [["false", "false", "true"], ["false", "true", "true"], ["true", "false", "true"], ["true", "true", "true"]], observed [["0", "0", "1"], ["0", "1", "1"], ["1", "0", "1"], ["1", "1", "1"]]

## Longitudinal inventory

- Runs: 10
- Result records: 26403
- Unique test identities: 26385
