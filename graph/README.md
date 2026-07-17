# Turso graph frontend

These crates implement the Turso-owned boundary for graph languages and graph
execution. Canonical data, transactions, storage, and bytecode remain owned by
Turso core.

```text
turso_graph_ir
    ^       ^
    |       |
cypher   runtime
    \       /
     frontend -> turso_core
```

- `turso_graph_ir` owns stable graph identities, bound plans, catalog traits,
  and semantic errors.
- `turso_graph_cypher` owns source text, parsing, source AST, spans, and
  diagnostics. It may lower into `turso_graph_ir`; donor AST types do not leave
  this crate.
- `turso_graph_runtime` owns Turso-adapted adjacency and traversal services. It
  consumes graph IR contracts and does not own canonical rows or transactions.
- `turso_graph_frontend` composes the parser, binder/IR, runtime, and core
  frontend preparation API. It never emits VDBE instructions directly.

The dependency direction is enforced by Cargo manifests. None of the four
crates contains donor implementation code in this scaffold. Before copied,
translated, or structurally adapted material enters a crate, follow
[`PROVENANCE.md`](PROVENANCE.md): pin its source and license, record the
adaptation, add file-level attribution, and install the required license and
NOTICE text in the same commit.
