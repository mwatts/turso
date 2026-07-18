# Turso graph conformance report

Generated from the typed manifests under `graph/testdata/suites/`. Supported
scenarios execute end-to-end; unordered results compare as multisets.
Unsupported scenarios must fail at their declared diagnostic boundary. A
supported scenario that errors or returns different rows fails the suite, as
does an unsupported scenario that unexpectedly succeeds. This curated
mixed-source slice is evidence of the listed behavior, not a claim of full
openCypher TCK conformance.

## Supported (32)

- `tck.with.with1.scenario-1`
- `grafeo.match.directed-edge`
- `age.vle.zero-length`
- `pggraph.traversal.exact-two-hops`
- `ladybug.match.undirected-edge`
- `ladybug.optional.null-extension`
- `sparrow.path.two-hop-multiplicity`
- `sparrow.merge.existing-node`
- `cqlite.match.labeled-node-scan`
- `cqlite.create.properties`
- `samyama.aggregate.global-count`
- `age.vle.unbounded-traversal`
- `grafeo.match.incoming-edge`
- `age.vle.fixed-multi-hop`
- `grafeo.with.projected-expression`
- `age.unwind.literal-list`
- `grafeo.order-by.nonprojected-property`
- `grafeo.pagination.skip-limit`
- `grafeo.optional.where-null-extends-pattern`
- `tck.where.numeric-comparison`
- `cqlite.set.property`
- `age.remove.property`
- `age.delete.relationship`
- `sparrow.merge.absent-node`
- `grafeo.regression.wrong-relationship-direction`
- `age.regression.zero-length-preserves-identity`
- `sparrow.regression.missing-property-is-null`
- `sparrow.regression.variable-path-terminal-label`
- `cqlite.regression.parameterized-property`
- `ladybug.regression.detach-delete`
- `grafeo.regression.optional-count-preserves-rows`
- `turso.regression.constraint-index-drop-error`

## Failed (0)

- None.

## Unsupported (6)

- `tck.call.subquery-scope` — CALL introduces a nested scope and execution boundary.
- `grafeo.path.all-shortest-paths` — all-shortest result multiplicity and memory limits need a separate contract.
- `pggraph.path.weight-expression` — Cypher weight-expression semantics are not yet bound into graph IR.
- `ladybug.path.shortest-keyword` — SHORTEST syntax is not part of the current parser slice.
- `sparrow.path.shortest-function` — shortestPath source syntax is not yet lowered into the shared shortest-path IR.
- `samyama.planner.independent-patterns` — multiple path patterns require join enumeration beyond the current single-pattern binder.
