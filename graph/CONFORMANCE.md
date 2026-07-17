# Turso graph conformance report

Generated from `graph/testdata/conformance/manifest.toml`. Supported scenarios execute end-to-end; unordered results compare as multisets. Unsupported scenarios must fail at the frontend boundary. A supported scenario that errors or returns different rows is reported separately as failed and fails CI. This curated mixed-source slice is not a claim of full openCypher TCK conformance.

## Supported (12)

- `tck-with-node-scope`
- `grafeo-directed-edge`
- `age-zero-length-path`
- `pggraph-exact-two-hops`
- `ladybug-undirected-edge`
- `ladybug-optional-null-extension`
- `sparrow-exact-two-hop-multiplicity`
- `sparrow-merge-existing-node`
- `cqlite-labeled-node-scan`
- `cqlite-create-properties`
- `samyama-global-count`
- `age-unbounded-traversal`

## Failed (0)

- None.

## Unsupported (6)

- `tck-call-subquery` — CALL introduces a nested scope and execution boundary.
- `grafeo-all-shortest-paths` — all-shortest result multiplicity and memory limits need a separate contract.
- `pggraph-weight-expression` — Cypher weight-expression semantics are not yet bound into graph IR.
- `ladybug-shortest-keyword` — SHORTEST syntax is not part of the current parser slice.
- `sparrow-shortest-function` — shortestPath source syntax is not yet lowered into the shared shortest-path IR.
- `samyama-independent-pattern-enumeration` — multiple path patterns require join enumeration beyond the current single-pattern binder.
