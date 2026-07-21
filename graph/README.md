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

## Quickstart

```rust
let (io, db) = turso_graph_frontend::open_database(
    "app.db",
    None,
    OpenFlags::default(),
    DatabaseOpts::default(),
)?;
let conn = db.connect()?;
let graph = turso_graph_frontend::GraphConnection::open(conn, "social")?;
let stmt = graph.prepare("MATCH (n:Person) RETURN n.name", &Default::default())?;
let types = stmt.result_types();
let summary = graph.execute("CREATE (:Person {name: $name})", &params)?;
```

`open_database`/`open_database_with_io` return `turso_core::Result`;
`GraphConnection::open` (and `open_with_parameters`/`install`) take an
`Arc<turso_core::Connection>` and return `turso_graph_frontend::Result`.
`prepare`/`prepare_cancellable` return a `Statement` wrapper exposing
`.result_types()`; `query`/`query_cancellable` collect rows directly;
`execute` returns a `MutationSummary`. The crate also re-exports
`turso_core` as `core` plus the commonly needed core types (`Database`,
`DatabaseOpts`, `OpenFlags`, `Value`, ...) at the crate root.

- Frontend separation: this crate never depends on, and is never depended on
  by, the Postgres frontend. An app that wants Cypher and Postgres SQL on one
  connection installs both compilers itself via core's
  `Connection::register_frontend_compiler`.
- Roadmap gap: no `bindings/rust`-level ergonomic/async wrapper exists for the
  graph frontend (nor for `turso_pg`); only core SQL has the `turso` crate's
  async `Rows`/`Transaction` surface. Consumers embed this crate synchronously.

The dependency direction is enforced by Cargo manifests. Structurally adapted
code remains limited to the attributed parser and portable runtime files;
donor catalog, planner, storage, executor, and server types do not cross the
boundary. Before copied, translated, or structurally adapted material enters a
crate, follow
[`PROVENANCE.md`](PROVENANCE.md): pin its source and license, record the
adaptation, add file-level attribution, and install the required license and
NOTICE text in the same commit.

The live compatibility result is published in
[`test-results/REPORT.md`](test-results/REPORT.md) (regenerated on every
recorded baseline run); [`CONFORMANCE.md`](CONFORMANCE.md) summarizes the
corpus contract. The `turso_graph_testkit` crate owns the
typed mixed-source manifests, smoke/deep execution, append-only JSONL history,
longitudinal reporting, and lifecycle performance workloads. Run the gates and
representative CSR benchmarks with:

```sh
cargo run -q -p turso_graph_testkit -- run smoke --no-record
cargo run -q -p turso_graph_testkit -- run deep --no-record
cargo run -q -p turso_graph_testkit -- corpus-stats
cargo run -q -p turso_graph_testkit -- corpus --no-record
cargo run -q -p turso_graph_testkit -- performance smoke --no-record
cargo test -p turso_graph_testkit
cargo test -p turso_graph_runtime --test benchmark_shapes
cargo bench -p turso_graph_runtime --bench graph_shapes
```

Omit `--no-record` on an intentional baseline run to append one result per
stable test identity to `graph/test-results/history.jsonl` and regenerate
`graph/test-results/REPORT.md`. Use `verify-history` to validate the persisted
schema and uniqueness contract without running a workload.

The corpus commands cover all imported source identities from the
openCypher TCK, Grafeo, Apache AGE, SparrowDB, and CQLite. LadybugDB/Kuzu is
excluded because its suite mixes vendor-specific database language and result
contracts into standard-looking Cypher queries.
Canonical execution and cross-source parser caches remove duplicate work while
preserving every source identity in the result stream.
