---
task_id: graph-frontend-foundation
complexity: high
risk: high
ambiguity: medium
agent_pattern: pipeline
estimated_tokens: 30000
---

# Graph frontend foundation implementation plan

## Goal

Establish the two feasibility seams: frontend-aware preparation/reprepare and a
Turso-owned graph language boundary from Uni-derived source AST through binder
and graph IR. Finish with a transactional graph catalog whose generation is
invalidated by graph-frontend and direct SQL writes.

## Required skills

| Skill | Path | Relevance |
|-------|------|-----------|
| TursoDB | `/Users/markwatts/.agents/skills/tursodb/SKILL.md` | Current frontend and engine boundaries |
| Rust | `/Users/markwatts/.agents/skills/rust/SKILL.md` | Public library types, errors, and crate boundaries |
| Code quality | `.claude/skills/code-quality/SKILL.md` | Database correctness and invariants |
| Testing | `.claude/skills/testing/SKILL.md` | Test placement and commands |

Read each skill completely before implementation.

## Inputs and boundaries

- Architecture: `docs/multi-frontend.md`, especially §§6.2–6.10.
- Prepare path: `core/connection.rs`, `core/statement.rs`.
- Prepared state: `core/vdbe/mod.rs`, `core/vdbe/builder.rs`.
- Dialect examples: `core/dialect/`, `postgres/frontend/session.rs`.
- Workspace registration: root `Cargo.toml`.

Do not implement graph traversal, mutations, Bolt, HTTP, or pgGraph extraction
in this plan.

## Task 1: add failing frontend-reprepare tests

Modify `core/dialect/mod.rs` and `tests/integration/` to prove:

1. A non-SQL frontend compiler is invoked for initial preparation.
2. Schema invalidation invokes the same compiler with the original source.
3. Parameters survive reprepare.
4. Missing compiler registration returns a typed error rather than falling
   through to SQLite parsing.
5. The retry inside `Connection::compile_cmd` uses the recipe as well as
   `Statement::reprepare`.

Run the narrow tests and confirm they fail for the expected missing behavior.

## Task 2: implement frontend-aware prepared sources

Add a small core module, preferably `core/frontend.rs`, containing:

- `FrontendId`: stable, debug-printable newtype.
- `PreparedSource`: native dialect text or frontend id + original source.
- `FrontendCompiler`: `Send + Sync` service that compiles source to a Turso AST
  command and consumed byte count.
- A typed error for unknown or mismatched frontend compilers.

Store only `PreparedSource` in `PreparedProgram`; store compiler services in a
connection registry. Do not store callbacks or donor objects in bytecode.

Refactor initial compile, cross-process schema retry, and reprepare to use one
dispatcher. Preserve the existing `prepare` and `prepare_translated_stmt`
behavior. Add an explicit frontend preparation API rather than changing the
meaning of existing callers silently.

Exit check:

```bash
rtk cargo test -p turso_core dialect
rtk cargo test -p core_tester --test integration_tests reprepare
```

## Task 3: migrate the Postgres session as the first real compiler

Add a Postgres compiler implementation in `postgres/frontend` using the same
parser and translator as initial preparation. Register it when constructing a
`PgConnection`, then route root PostgreSQL statements through the new frontend
API. Keep engine-generated prerequisites on canonical Turso/SQLite AST paths.

Add a Postgres integration test that prepares a PostgreSQL-specific statement,
changes schema state, and executes successfully after reprepare.

## Task 4: scaffold graph crates and provenance

Add sibling workspace crates:

```text
graph/ir        turso_graph_ir
graph/cypher    turso_graph_cypher
graph/runtime   turso_graph_runtime
graph/frontend  turso_graph_frontend
```

Register all four in workspace members and dependencies. Use the repository's
edition and lint settings, even where generic Rust guidance differs.

Create `graph/PROVENANCE.md` recording source repository, pinned revision,
source path, license, and adaptation type. No donor code enters the tree
without a corresponding entry.

## Task 5: define the graph IR and errors

In `graph/ir`, define private-field identifier newtypes and fallible
constructors for graph, source table, node, relationship, label, relationship
type, and property identity. Define typed public errors.

Add only the first read-only operators:

- node scan and fixed relationship expand;
- filter and project;
- aggregate, distinct, sort, skip, and limit;
- optional/left apply, unwind, and union.

Represent scope, nullability, direction, and result shape explicitly. Do not
include Turso `Value`, PostgreSQL OIDs, Arrow types, or donor record ids.

## Task 6: adapt the initial Uni parser and build the binder

In `graph/cypher`, adapt the smallest Uni grammar/source-AST slice needed by
the selected fixtures. Preserve source spans and diagnostics. Exclude Uni
catalog, values, Arrow integration, planner, executor, storage, plugins,
time-travel, and unrelated DDL.

In `graph/frontend`, implement `GraphCatalogSnapshot` and a binder producing
`turso_graph_ir` plans. Add scope, duplicate-variable, unresolved-name,
parameter, nullability, and property-resolution tests.

## Task 7: implement graph registration and generation metadata

Create canonical internal tables for graphs, node sources, relationship
sources, and graph generations. Use ordinary Turso transactions and stable ids.

Registration must validate:

- source tables and columns exist;
- node identity columns are usable and unique by contract;
- relationship start/end columns resolve to registered node sources;
- reserved graph names cannot collide.

Generate INSERT/UPDATE/DELETE triggers on every mapped source table to advance
the affected graph generation. Trigger changes must roll back with the source
write. Test direct SQL writes, graph-session writes, rollback, and source-table
removal.

## Verification and completion

```bash
rtk cargo fmt --all -- --check
rtk cargo clippy -p turso_core -p turso_pg -p turso_graph_ir \
  -p turso_graph_cypher -p turso_graph_runtime -p turso_graph_frontend \
  --all-targets --all-features -- --deny=warnings
rtk cargo test -p turso_core dialect
rtk cargo test -p turso_pg_tests
rtk cargo test -p turso_graph_ir -p turso_graph_cypher -p turso_graph_frontend
rtk cargo test -p core_tester --test integration_tests reprepare
```

The plan is complete when frontend-specific reprepare is proven, the graph IR
contains no donor/core types, parser and binder fixtures pass, and catalog
generation changes are transactional. Commit each task or coherent pair with a
conventional commit and descriptive body.
