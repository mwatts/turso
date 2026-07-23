---
task_id: graph-semantic-constraints
complexity: high
risk: high
ambiguity: low
agent_pattern: pipeline
subagent_type: general-purpose
model: inherit
isolation: worktree
tools_required: [file_read, apply_patch, ripgrep, cargo, git]
estimated_tokens: 32000
timeout_minutes: 360
---

# Graph semantic constraints and safe additive evolution

## Status

Complete and validated (2026-07-23).

## Scope

Complete Milestone 4 of
`.specs/graph-semantic-schema-overlay.agent-spec.md` with an additive constraint
registration API and immutable constraint snapshot.

Constraints apply to one concrete semantic owner type. Fragment-owned
properties are addressed through each concrete member type so physical
mappings and row membership remain unambiguous.

## Public model

`SemanticConstraintRegistration` contains:

- required properties;
- composite keys;
- unique properties;
- property value predicates:
  - inclusive or exclusive ranges;
  - finite allowed-value sets;
  - regular expressions;
- relationship endpoint cardinalities with a minimum and optional maximum.

A relationship endpoint cardinality counts relationships of one semantic
relationship type for every node carrying one of that endpoint's permitted
concrete semantic types. It is evaluated against the stored `start` or `end`
endpoint selected by the registration.

## Persistence and evolution

- Constraint rows use graph-scoped semantic type and property IDs, never
  physical table or column names.
- Registration is additive, atomic, and idempotent.
- Identical replay writes nothing and does not bump graph generation.
- Adding a constraint validates all visible data before commit.
- A failed validation rolls back catalog rows and generation changes.
- Changing or removing an active constraint is not part of additive
  registration and returns a typed evolution error.
- Reopening a graph reconstructs the same constraint snapshot.

## Enforcement

- The binder rejects literal values that violate range, allowed-value, or
  regex predicates.
- The binder rejects statically known removal or replacement of required
  properties.
- Runtime values and dynamic maps are checked before their physical mutation
  when the value is available.
- The complete constraint state is checked before the graph mutation
  savepoint is released. This permits a multi-operation query to repair a
  temporary intermediate state while guaranteeing zero partial writes.
- Required, key, unique, and cardinality checks are scoped by semantic
  label/type membership, including source-qualified junction rows.
- Existing physical `NOT NULL`, `UNIQUE`, and `CHECK` constraints continue to
  enforce direct SQL. Semantic constraints are guaranteed for graph-frontend
  writes. They are not claimed as database-wide direct-SQL enforcement when
  semantic membership cannot be represented by an equivalent native Turso
  constraint or index.

## Required functional coverage

- [x] Registration creates additive catalog tables and persists conceptual IDs.
- [x] Identical replay is idempotent.
- [x] Conflicting evolution is rejected without catalog or generation changes.
- [x] Existing invalid data rejects activation atomically.
- [x] Required properties cover CREATE, MERGE, SET, REMOVE, literal and dynamic
      replacement, ON CREATE, ON MATCH, WITH, FOREACH, and multi-row rollback.
- [x] Composite keys reject NULL members and duplicate tuples.
- [x] Unique properties reject duplicate non-NULL values within one concrete
      type without cross-contaminating another type sharing the source.
- [x] Range predicates cover numeric and text bounds with inclusive/exclusive
      endpoints.
- [x] Allowed-value predicates cover Boolean, integer, real, and text values.
- [x] Regex predicates validate text properties and reject invalid patterns at
      registration.
- [x] Relationship minimum and maximum cardinality cover stored start and end
      endpoints, direction reversal, creation, deletion, and node creation.
- [x] A single mutation may temporarily violate a database-state constraint if
      its final state is valid.
- [x] Reopen reconstructs and enforces every constraint kind.
- [x] Legacy and semantic graphs without constraints behave unchanged.
- [x] Documentation explains the API, activation validation, evolution rules,
      and direct-SQL boundary.
- [x] Formatting, focused tests, Clippy, smoke, and non-recorded corpus pass.

## Verification

```bash
rtk cargo fmt --all -- --check
rtk cargo test -p turso_graph_ir
rtk cargo test -p turso_graph_frontend
rtk cargo test -p turso_graph_testkit
rtk cargo run -q -p turso_graph_testkit -- run smoke --no-record
rtk cargo run -q -p turso_graph_testkit -- corpus --no-record
rtk cargo clippy --workspace --all-features --all-targets -- --deny=warnings
rtk git diff --check
```

Validation recorded on 2026-07-23:

- semantic integration: 62/62 passed;
- `turso_graph_frontend`: 201 passed;
- `turso_graph_ir`: 10 passed;
- `turso_graph_testkit`: 41 passed;
- smoke corpus: 11/11 clean;
- non-recorded deep corpus unchanged at 8,919 passed, 53 unsupported, and
  1,270 failed;
- workspace Clippy completed with zero errors; formatting and patch hygiene
  checks passed.
