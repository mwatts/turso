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

## Documentation

Two documents cover this layer; everything else in `graph/` is a contract
(`PROVENANCE.md`, `CONFORMANCE.md`) or a record (`test-results/`).

- [`docs/graph.md`](../docs/graph.md) — **user guide.** Registration, sessions,
  parameters, transactions, roles and n-ary relations, semantic schemas,
  fragment-interface polymorphism, additive semantic constraints, the
  direct-SQL integrity boundary, traversal snapshots, the accepted Cypher
  surface, what Turso adds beyond it, and how to test a change.
- [`docs/graph-internals.md`](../docs/graph-internals.md) — **implementation
  map.** Crate topology, the read and write pipelines, the IR vocabulary, the
  role model's invariants, catalog and snapshot design, where to change what,
  and future work.

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
let summary = graph.execute("CREATE (:Person {name: 'Ada'})", &Default::default())?;
```

`open_database`/`open_database_with_io` return `turso_core::Result`;
`GraphConnection::open` (and `open_with_parameters`/`install`) take an
`Arc<turso_core::Connection>` and return `turso_graph_frontend::Result`.
`prepare`/`prepare_cancellable` return a `Statement` wrapper exposing
`.result_types()`; `query`/`query_cancellable` collect rows directly;
`execute` returns a `MutationSummary`. The crate also re-exports
`turso_core` as `core` plus the commonly needed core types (`Database`,
`DatabaseOpts`, `OpenFlags`, `Value`, ...) at the crate root.

## Opening a graph database

Preferred — dialect-pinned database (mirrors `turso_pg::open_database`):

    let (_io, db) = turso_graph_frontend::open_database(path, None, flags, opts)?;
    let conn = db.connect()?;
    turso_graph_frontend::register_graph(&conn, &registration)?;   // first time only
    let graph = turso_graph_frontend::GraphConnection::open(conn, "social")?;

The dialect gives you: `"graph-cypher"` database identity (mismatched
reopens rejected), the temporal/cypher function surface on every
connection, custom types always on, and the `turso_graphs` catalog
virtual table.

Attach mode — graph layer on an existing SQLite-dialect database:

    let session = GraphConnection::open(existing_conn, "social")?;

`GraphConnection::install` registers the per-connection compiler and the
temporal extension; nothing about the database file changes.

Open-mode and Core-seam contract (see
[`docs/graph-frontend-core-alignment.md`](../docs/graph-frontend-core-alignment.md)
for findings and the implementation plan):

- **Dialect-pinned open** — `open_database` / `open_database_with_io` use
  `GraphDialect` (`name() == "graph-cypher"`). Root temporal/`cypher_*`
  resolution is dialect-owned; `install` still registers the temporal
  extension for InternalHelper mutation SQL. `turso_graphs` is registered on
  schema build.
- **Attach mode** — `GraphConnection::open` / `install` on an existing
  connection (often `SqliteDialect`). Guarantees come from `install`
  (compiler registration, temporal extension, expand vtab). File dialect name
  may stay `"sqlite"`.
- **Reads** — go through `prepare_frontend("graph-cypher")`.
- **Mutations** — multi-statement orchestration today (autocommit:
  `BEGIN IMMEDIATE`; write txn: savepoint; bare `BEGIN`:
  `RequiresWriteTransaction`); not a single `PreparedSource`. Known debt.
- **Composition** — Postgres and Graph stay separate crates; apps register
  both compilers on one core connection if needed.

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
cargo bench -p turso_graph_frontend --bench semantic_prepare
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
