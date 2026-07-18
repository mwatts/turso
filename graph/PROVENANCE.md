# Graph frontend provenance

This manifest pins the external sources used to design and test the graph
frontend. It is the source of truth for donor revisions and adaptation
boundaries. Add an entry here before copying, translating, or structurally
adapting external material into this repository.

The Turso repository is MIT licensed. Uni, Grafeo, Apache AGE, pgGraph, and
Samyama are Apache-2.0 licensed; Ladybug, SparrowDB, and CQLite are MIT
licensed. The applicable license, notice, attribution, and modification
requirements remain preserved. This manifest does not relicense adapted
material as MIT-only code.

## Pinned sources

| Source | Repository | Revision | License | Intended use |
| --- | --- | --- | --- | --- |
| Uni | <https://github.com/rustic-ai/uni-db> | `0812a496c62769b67cf688930750ae384e3de68d` | Apache-2.0 | Structurally adapt the Cypher parser and adapt selected parser/TCK fixtures. Do not import Uni storage, execution, or catalog types. |
| Grafeo | <https://github.com/GrafeoDB/grafeo> | `4ebae02f06f8f0cbc57543f74b6ba06f259dbed3` | Apache-2.0; NOTICE copyright 2025-2026 S.T. Grond | Use its graph plans and Cypher tests as design and behavioral references for a Turso-owned graph IR. Adapt selected fixtures. |
| Apache AGE | <https://github.com/apache/age> | `6876abcab0a3281eb65a7e2a91238e0b5abfdea7` | Apache-2.0; Apache Software Foundation and Bitnine NOTICE | Use parser transforms and regression tests as semantic references for relational lowering. Implement lowering in Turso Rust rather than translating PostgreSQL internals. |
| pgGraph | <https://github.com/Evokoa/pgGraph> | `d689bcf2b3b52d7f878f61718be69ebcb953affc` | Apache-2.0; NOTICE copyright 2026 Evokoa Pte. Ltd. | Translate portable CSR, traversal, path, and safety logic to Turso-owned interfaces. Exclude PostgreSQL/pgrx catalog, SPI, SQL facade, and extension lifecycle code. |
| Ladybug | <https://github.com/mwatts/ladybug> | `7eab431c6becf64f58f7c2ff4c0fb1f160acb492` | MIT; copyright 2022-2025 Kùzu Inc. | Adapt focused undirected, optional-match, recursive-range, shortest-path, mutation, and error test intent. Exclude the C++ engine, storage, and test harness. |
| SparrowDB | <https://github.com/ryaker/SparrowDB> | `82d85b7a861dfb2e127452ed89eebbcee74bfef0` | MIT; copyright 2026 Rich Yaker | Adapt path-multiplicity, mutation, null, and historical regression cases as a secondary behavior oracle. Exclude its binder and executor. |
| CQLite | <https://github.com/mwatts/cqlite> | `e2b677e8429a4cb0ead087ffbd9195f4f3999819` | MIT; copyright 2021 Tilman Roeder | Adapt compact parser, matching, property, mutation, and transaction smoke cases. Exclude its graph storage and execution engine. |
| Samyama | <https://github.com/samyama-ai/samyama-graph> | `4520154a65838d2e17a51b91882a99df816365c3` | Apache-2.0 | Use planner, join-enumeration, aggregation, and optimizer tests as behavioral references. Import no optimizer rule until a Turso benchmark identifies the deficiency. |

The corresponding upstream `LICENSE` and `NOTICE` files were inspected at the
pinned revisions. When adapted source first lands, copy the applicable license
and NOTICE text under `licenses/graph/` and add the dependency or vendored-code
entry to the root `NOTICE.md` in the same commit.

## Adaptation types

- `structural-adaptation`: an implementation is translated or reshaped while
  retaining recognizable donor structure. It requires file-level attribution,
  an upstream path and revision, and an explicit modification notice.
- `fixture-adaptation`: a donor test is represented as a Turso test or fixture.
  It requires the same source metadata even when syntax or expected output is
  changed for the Turso harness.
- `behavioral-reference`: the donor establishes semantics, invariants, or
  expected results, but the Turso implementation is written against Turso
  interfaces without copying donor structure.
- `design-reference`: the donor informs boundaries or vocabulary only. No
  donor implementation is copied.

Generated or LLM-assisted translation does not change the adaptation type or
remove attribution obligations. The author of an adapted file must classify it
by its relationship to the donor, not by who or what performed the translation.

## Source boundaries

### Uni

Candidate parser material is limited to:

- `crates/uni-cypher/src/ast.rs`
- `crates/uni-cypher/src/grammar/cypher.pest`
- `crates/uni-cypher/src/grammar/walker.rs`
- `crates/uni-cypher/src/lib.rs`
- selected tests under `crates/uni-cypher/tests/` and `crates/uni-tck/`

The parser must emit frontend-owned syntax nodes and diagnostics. Uni graph,
storage, planner, executor, and catalog types are outside the boundary.

The initial adaptation is installed in:

- `graph/cypher/src/cypher.pest`, structurally adapted from Uni's reduced
  grammar surface;
- `graph/cypher/src/parser.rs`, structurally adapted from Uni's walker shape
  while replacing its AST, errors, and downstream types;
- `graph/cypher/src/ast.rs`, a Turso-owned source AST using Uni only as a
  behavioral reference.

The applicable upstream license is copied to
`licenses/graph/uni-db-apache-license.md`. Uni has no upstream `NOTICE` file at
the pinned revision.

### Grafeo

The graph IR is Turso-owned. Grafeo material under
`crates/grafeo-engine/src/query/` is a design and behavioral reference; its
types must not cross into Turso core. Selected `.gtest` cases may be adapted as
fixtures with their original case names recorded.

The initial normalized fixture adaptations are recorded in
`graph/testdata/fixed-patterns/manifest.toml`, with source case, revision,
ordering contract, and parser-support status per case. The applicable license
and notice are copied to `licenses/graph/grafeo-apache-license.md` and
`licenses/graph/grafeo-notice.md`.

### Apache AGE

AGE code under `src/backend/parser/` and `src/backend/optimizer/` is a
behavioral reference for translating graph clauses into relational operations.
PostgreSQL `Node`, `Query`, `Plan`, executor, and extension types are excluded.
The regression SQL and expected-output pairs may be adapted as fixtures.

The AGE cases normalized into `graph/testdata/fixed-patterns/manifest.toml`
retain their regression file and source location. The applicable license and
notice are copied to `licenses/graph/apache-age-apache-license.md` and
`licenses/graph/apache-age-notice.md`.

### pgGraph

Portable candidates include `graph/src/edge_store.rs`, `bfs.rs`,
`path_finder.rs`, `filter_index.rs`, `resolution_index.rs`, `safety.rs`, and
the storage-independent parts of `graph/src/projection/`. They must be adapted
behind Turso graph runtime traits and Turso transaction snapshots.

The following are references only and must not become the integration seam:

- `graph/src/catalog*` and PostgreSQL OID/regclass identity
- `graph/src/pg_tests/` and `graph/src/pg_test.rs`
- `graph/src/sql_*` and `graph/src/sql_facade/`
- pgrx/SPI/GUC/background-worker and extension lifecycle code

The initial portable runtime adaptation is installed in:

- `graph/runtime/src/csr.rs`, structurally adapted from
  `graph/src/edge_store.rs` into safe, owned forward and reverse CSR arrays;
- `graph/runtime/src/traversal.rs`, structurally adapted from
  `graph/src/bfs.rs` into bounded BFS/DFS path enumeration with explicit
  walk, trail, and path uniqueness;
- `graph/runtime/src/shortest.rs`, structurally adapted from
  `graph/src/path_finder.rs` into bounded unweighted BFS and weighted Dijkstra
  over Turso graph identities;
- `graph/runtime/src/limits.rs`, structurally adapted from
  `graph/src/safety.rs` into typed caller-owned limits and cancellation.

The adaptation excludes mmap/sidecar persistence, PostgreSQL node stores,
tenant/ACL state, transaction overlays, and server error reporting. The
applicable license and notice are copied to
`licenses/graph/pggraph-apache-license.md` and
`licenses/graph/pggraph-notice.md`.

Normalized differential expectations from the pinned pgrx-free unit tests are
recorded in `graph/testdata/pggraph-runtime/manifest.toml` and executed by
`graph/runtime/tests/pggraph_equivalence.rs`.

### Mixed-source conformance donors

The executable mixed-source slice is recorded in the typed manifests under
`graph/testdata/suites/` and run by `graph/testkit`. It normalizes, without
copying donor test bodies, cases from the openCypher TCK copy pinned through
Uni, Grafeo, AGE, pgGraph, Ladybug, SparrowDB, CQLite, and Samyama. The stable
identity, exact source case, pinned revision, license, and adaptation type are
stored beside every case. `graph/CONFORMANCE.md` describes the current support
level; intentional baseline runs append per-result records to
`graph/test-results/history.jsonl` and generate the longitudinal report.
Required ordering is preserved; unordered results are sorted and compared as
multisets. The testkit fails on zero discovery, duplicate identities, supported
scenario failures, or changed unsupported boundaries.

Ladybug's MIT license is copied to
`licenses/graph/ladybug-mit-license.md`; SparrowDB's to
`licenses/graph/sparrowdb-mit-license.md`; CQLite's to
`licenses/graph/cqlite-mit-license.md`. Samyama's Apache license reference is at
`licenses/graph/samyama-apache-license.md`, with the full Apache-2.0 text
already reproduced in the graph license directory. No donor implementation
from these four projects crosses the Turso graph boundary.

## Per-file record

Every copied or adapted file must carry a short header or adjacent module-level
record with:

```text
Source: <repository URL>
Revision: <full commit SHA>
Path: <upstream path>
License: Apache-2.0
Adaptation: structural-adaptation | fixture-adaptation
Changes: <what was translated or changed>
```

Behavioral and design references are recorded in tests, module documentation,
or this manifest; they do not require a donor copyright header when no donor
expression is copied.
