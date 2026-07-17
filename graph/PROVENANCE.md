# Graph frontend provenance

This manifest pins the external sources used to design and test the graph
frontend. It is the source of truth for donor revisions and adaptation
boundaries. Add an entry here before copying, translating, or structurally
adapting external material into this repository.

The Turso repository is MIT licensed. All four selected donors are Apache-2.0
licensed, which permits their use here provided the Apache license, notices,
attribution, and modification requirements are preserved. Adapted files remain
subject to the donor's Apache-2.0 terms; this manifest does not relicense them
as MIT-only code.

## Pinned sources

| Source | Repository | Revision | License | Intended use |
| --- | --- | --- | --- | --- |
| Uni | <https://github.com/rustic-ai/uni-db> | `0812a496c62769b67cf688930750ae384e3de68d` | Apache-2.0 | Structurally adapt the Cypher parser and adapt selected parser/TCK fixtures. Do not import Uni storage, execution, or catalog types. |
| Grafeo | <https://github.com/GrafeoDB/grafeo> | `4ebae02f06f8f0cbc57543f74b6ba06f259dbed3` | Apache-2.0; NOTICE copyright 2025-2026 S.T. Grond | Use its graph plans and Cypher tests as design and behavioral references for a Turso-owned graph IR. Adapt selected fixtures. |
| Apache AGE | <https://github.com/apache/age> | `6876abcab0a3281eb65a7e2a91238e0b5abfdea7` | Apache-2.0; Apache Software Foundation and Bitnine NOTICE | Use parser transforms and regression tests as semantic references for relational lowering. Implement lowering in Turso Rust rather than translating PostgreSQL internals. |
| pgGraph | <https://github.com/Evokoa/pgGraph> | `d689bcf2b3b52d7f878f61718be69ebcb953affc` | Apache-2.0; NOTICE copyright 2026 Evokoa Pte. Ltd. | Translate portable CSR, traversal, path, and safety logic to Turso-owned interfaces. Exclude PostgreSQL/pgrx catalog, SPI, SQL facade, and extension lifecycle code. |

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
