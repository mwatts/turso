---
task_id: graph-semantic-schema-overlay
complexity: high
risk: high
ambiguity: medium
agent_pattern: pipeline
subagent_type: general-purpose
model: opus
isolation: worktree
tools_required: [file_read, apply_patch, cargo, git]
estimated_tokens: 30000
timeout_minutes: 240
---

# TASK

Implement an opt-in semantic-schema overlay for the Turso graph frontend that decouples conceptual graph types from physical source tables and validates typed property ownership on reads and writes, without adding TypeQL, changing canonical row storage, or claiming TypeDB/PERA compatibility.

# REQUIRED SKILLS

| Skill | Path | Relevance |
|-------|------|-----------|
| `rust` | `/Users/markwatts/.agents/skills/rust/SKILL.md` | Public Rust API, catalog types, errors, and tests must remain idiomatic and additive. |
| `rust-best-practice` | `.claude/skills/rust-best-practice/SKILL.md` | Repository-mandated Rust implementation and verification rules. |
| `code-quality` | `.claude/skills/code-quality/SKILL.md` | Turso-specific correctness, simplicity, and review conventions. |
| `testing` | `.claude/skills/testing/SKILL.md` | Select the narrowest graph frontend and conformance coverage. |
| `pr-workflow` | `.claude/skills/pr-workflow/SKILL.md` | Formatting, commits, and final validation. |

**Directive**: The implementing agent MUST read every skill above in full before changing code. It MUST also read the target file's exports, its immediate callers, and the shared catalog utilities before editing.

# CONTEXT

## Product boundary

This stream adopts useful **data semantics** associated with TypeDB, not its language or storage implementation:

- conceptual entity, relation, and attribute/property types;
- schema-defined ownership and binary endpoint participation;
- semantic validation of reads and writes;
- later, single inheritance, abstract types, polymorphic scans, and constraints.

The frontend remains Cypher-facing and relationally lowered. Turso core continues to own canonical tables, transactions, constraints, and bytecode. “TypeDB-inspired” MUST NOT be presented as TypeDB, TypeQL, PERA, or hypergraph compatibility.

The smallest worthwhile delivery is Milestones 1 and 2 below. Later milestones are specified to protect the design seams, but are not part of that first implementation.

## Verified current architecture

- `GraphRegistration` registers physical node and relationship source tables in `graph/frontend/src/catalog.rs`.
- A source registration's `name` currently doubles as its conceptual label or relationship type. `SchemaCatalog::label` and `relationship_type` derive IDs from source-list positions.
- `SchemaCatalog::property` currently resolves against the first source table for an entity kind and uses physical column ordinal + 1 as `PropertyId`.
- `GraphCatalogSnapshot` resolves a single default node and relationship source. Although `relationship_sources` exists, binder paths still commonly call the singular methods.
- `RelationalCatalogSnapshot` is the correct physical-layout seam: stable source/property identities become table and column names only during lowering.
- `NodeScan`, `FixedExpand`, `CreateNode`, and `CreateRelationship` carry physical `SourceTableId`s. `CreateRelationship` and expansion IR are explicitly binary (`from`/`to`, start/end columns).
- `GraphConnection::open` constructs one `SchemaCatalog`, shared by binder and lowerer. `GraphCompiler` owns read compilation; mutation binding and execution use the same catalog boundary.
- `DynamicCatalog` in `graph/testkit/src/dynamic_catalog.rs` intentionally provisions schemaless donor labels, relationship types, and columns. It is a compatibility adapter, not the semantic-schema implementation.
- `GRAPH_CATALOG_VERSION` participates in traversal-snapshot compatibility. It is not presently a general catalog migration framework.

## TypeDB semantics used as design input

Use these official sources only to understand semantics and vocabulary:

- [TypeDB features](https://typedb.com/features)
- [Entities, relations, and attributes](https://typedb.com/docs/core-concepts/typeql/entities-relations-attributes/)
- [Schema and data validation](https://typedb.com/docs/core-concepts/typeql/schema-data/)
- [Cardinality, key, unique, and value constraints](https://typedb.com/docs/core-concepts/typeql/constraining-data/)
- [Functions versus rules](https://typedb.com/docs/typeql-reference/functions/functions-vs-rules/)

Relevant ideas are first-class conceptual types, scoped role interfaces, single inheritance, identity-by-value attributes, schema-checked writes, and database-state-aware constraint validation. TypeDB 3's explicit, on-demand recursive functions support deferring inference until Turso has a reusable table-valued graph-function seam.

## Semantic model for Turso

### Conceptual types versus Cypher labels

A **semantic type** is a graph-scoped catalog identity. It is not a table name.

- Node semantic types are addressed through Cypher labels.
- Relationship semantic types are addressed through Cypher relationship types.
- One physical source may back multiple semantic types.
- In Milestones 1-2, each semantic type maps to exactly one physical source. The mapping is represented independently so Milestone 3 can support one polymorphic type scan over multiple sources.
- In strict semantic mode, every resolved node label denotes a semantic node type and a mutation MUST identify exactly one semantic node type. Ordinary untyped labels remain available only in legacy/schemaless mode for Milestones 1-2. Multiple semantic labels are rejected until inheritance exists; Milestone 3 may allow labels along one ancestor chain and treat the most-specific type as the instance type.
- Relationship mutations MUST identify exactly one concrete semantic relationship type.

### Properties and ownership

A **semantic property** is a graph-scoped stable identity and name. An ownership declaration maps `(owner semantic type, semantic property)` to a physical source column.

- The physical column remains the stored value; do not create attribute-instance rows in Milestones 1-4.
- `ValueType` and nullability MUST be derived through the existing core schema classification in `SchemaCatalog`, not duplicated in a second serialized type system.
- If one semantic property is owned by multiple types, all mapped columns MUST resolve to compatible graph value types. Registration fails otherwise.
- Property IDs MUST be stable catalog IDs, not column ordinals.
- Structural identity and relationship endpoint columns MUST NOT become payload properties accidentally.

### Binary endpoint participation

Milestone 2 validates the useful subset of role semantics that fits current storage:

- a semantic relationship type declares allowed semantic node types for its start and end endpoint;
- the constraints lower onto the existing `start`/`end` columns and `from`/`to` IR;
- no public claim is made that `start` and `end` are general named role interfaces;
- arbitrary role names, repeated role players, relation-to-relation participation, and n-ary relations remain deferred.

### Compatibility modes

Semantic schema is opt-in per registered graph.

- A graph with no semantic-schema rows retains today's source-derived, schemaless behavior.
- `GraphRegistration` MUST remain source-compatible; adding a required field would break existing struct literals. Add a separate registration API and separate public registration types.
- `DynamicCatalog` and donor corpus loading stay in legacy mode.
- No existing graph is silently promoted to strict semantic mode.

## Required milestones

### Milestone 1 — decouple conceptual types from physical sources

Deliver an additive semantic catalog and snapshot representation:

1. Persist stable graph-scoped identities for semantic node types, relationship types, and properties. Reuse the existing strong `LabelId`, `RelationshipTypeId`, and `PropertyId` identities if their invariants fit; do not create redundant ID families and never reuse `SourceTableId` as a conceptual identity.
2. Add additive internal catalog tables for:
   - semantic types: graph, stable ID, kind, case-insensitive name, and concrete/abstract state reserved for Milestone 3;
   - semantic type-to-source mapping;
   - semantic properties: graph, stable ID, and case-insensitive name;
   - property ownership: owner type, property, source, and physical column;
   - binary endpoint constraints: relationship type, endpoint (`start` or `end`), allowed node type.
3. Add a separate `register_semantic_schema` API. Registration is idempotent for an identical definition and atomic: validate the complete definition before publishing any rows.
4. Materialize one immutable catalog snapshot per graph preparation context. Name resolution MUST return conceptual identity plus the physical source candidates required by current IR.
5. Keep physical names exclusively behind `RelationalCatalogSnapshot`.
6. Make snapshot/catalog compatibility explicit. If conceptual identities affect traversal snapshots, bump `GRAPH_CATALOG_VERSION` and prove stale snapshots rebuild; do not overload it as a general migration ledger without documenting that decision.

First delivery restriction: each concrete semantic type maps to one source. The schema representation MUST NOT require source names and type names to match and MUST allow multiple types to share a source.

### Milestone 2 — typed ownership and semantic write validation

Deliver strict validation only for graphs with a registered semantic schema:

1. Track the possible semantic types of every node/relationship binding through the binder. Label/type predicates narrow that set.
2. On read property access or pattern property predicates, resolve the property only when every possible concrete type owns it with compatible semantics. Return `PropertyNotOwned` when none owns it and a targeted ambiguity error when only a subset owns it; never compile a query with partial semantics.
3. On `CREATE`/`MERGE`, require exactly one known semantic type. All Milestones 1-2 types are concrete; the registration API MUST NOT expose the reserved abstract state until Milestone 3.
4. On `SET`, map replacement, property removal, `ON CREATE`, `ON MATCH`, and nested mutation stages, validate ownership against the target binding's possible semantic types.
5. Check statically known expression types in the binder. Parameters, `Any`, and dynamically produced map values MUST also be checked against the resolved property type at execution before physical SQL mutation.
6. Validate binary relationship endpoints against start/end allowed node types. Direction reversal MUST swap endpoint checks correctly.
7. Dynamic map replacement MUST reject unknown or unowned keys before changing any row. Multi-operation mutation validation MUST be atomic within the existing Turso transaction.
8. Return typed errors with semantic type/property names and source spans for binder failures. Runtime failures MUST identify the property/type and abort the mutation.

Milestone 2 does **not** introduce required/cardinality/key/unique/value constraints. Existing physical `NOT NULL`, `UNIQUE`, and `CHECK` constraints continue to work, but are not yet semantic constraints.

### Milestone 3 — single inheritance, abstract types, polymorphic scans

Do not implement until Milestones 1-2 are merged and measured.

1. Activate optional `supertype_id` and `is_abstract` catalog fields.
2. Enforce same-kind, graph-local, acyclic, single inheritance at registration.
3. Precompute or deterministically resolve ancestor/descendant closures in the immutable snapshot; do not add recursive catalog queries to hot binder paths.
4. Inherit property ownership and endpoint permissions. Reject conflicting inherited physical mappings or incompatible value types.
5. Make a scan for a semantic supertype include all concrete descendant types. Prefer composing existing per-source scans with `Union` when possible; introduce a new IR operator only if an executable plan proves `Union` cannot preserve identity/scope semantics.
6. Reject instantiation of abstract types while allowing them in reads.
7. Preserve Cypher label behavior: multiple semantic labels are valid only when they form one ancestor chain, and the most-specific label determines the concrete instance type.

### Milestone 4 — constraints and safe additive evolution

Implement in this order: required/minimum cardinality, key, unique, range/value/regex, then broader cardinality.

1. Define constraints in the semantic catalog; do not infer them merely from physical DDL.
2. Split validation by what can be proven at bind time, per-row runtime, and transaction/database-state time.
3. Prefer native Turso constraints or indexes for durable enforcement when semantics map exactly. Binder-only checks are insufficient because SQL can modify backing tables directly.
4. A schema change MUST validate all existing visible data before becoming active. Failed validation leaves the old schema and data unchanged.
5. Additive schema writes are idempotent. Constraint tightening, reparenting, type remapping, property removal, and value-type changes require an explicit later evolution design; do not smuggle them into an “additive” API.
6. Define direct-SQL bypass policy before claiming database-wide semantic integrity. Until core can protect owned backing tables, document integrity as guaranteed only for graph-frontend writes plus physical SQL constraints.

### Decision gate A — first-class attribute instances

Do not implement as part of Milestones 1-4. Produce an ADR before proceeding that compares:

- current property-as-column storage;
- identity-by-value attribute instances shared by owners;
- independent versus owner-dependent attribute lifecycle;
- multi-valued ownership and its required junction storage;
- query/result representation and migration cost;
- index, uniqueness, and deletion behavior.

Approval requires a workload that benefits from querying attributes as graph objects. A catalog-only imitation without instance semantics is not sufficient.

### Decision gate B — native named/n-ary relations

Defer. Current IR and storage are binary: `CreateRelationship`, `FixedExpand`, relationship layouts, and source registrations all have start/end endpoints.

Before implementation, an ADR MUST define role identity, repeated roles, relation-to-relation participation, n-ary storage, mutation syntax, traversal semantics, and lowering/runtime impact. Reifying an n-ary relation into a node plus binary edges is an application modeling option, not native n-ary support.

### Decision gate C — inference and rules

Defer. First design reusable, typed, table-valued graph functions with recursion limits, cancellation, deterministic semantics, and planner integration. Only then evaluate explicit on-demand recursive functions. Do not add implicit materialized inference or advertise TypeDB rule compatibility.

# INPUTS

| Input | Location | Format | Required |
|-------|----------|--------|----------|
| Graph catalog and registration | `graph/frontend/src/catalog.rs` | Rust | yes |
| Production catalog snapshot | `graph/frontend/src/schema_catalog.rs` | Rust | yes |
| Binder contract and mutation binding | `graph/frontend/src/binder.rs` | Rust | yes |
| Physical lowering contract | `graph/frontend/src/lowering.rs` | Rust | yes |
| Mutation execution | `graph/frontend/src/mutation.rs` | Rust | yes |
| Session/compiler assembly | `graph/frontend/src/session.rs`, `graph/frontend/src/compiler.rs` | Rust | yes |
| Graph identities and plans | `graph/ir/src/identity.rs`, `graph/ir/src/plan.rs`, `graph/ir/src/mutation.rs` | Rust | yes |
| Snapshot compatibility | `graph/frontend/src/snapshot.rs` | Rust | yes |
| Public API examples | `graph/README.md`, `docs/graph.md` | Markdown | yes |
| Existing catalog/binder/type tests | `graph/frontend/tests/`, inline tests in the files above | Rust/fixtures | yes |
| Schemaless compatibility adapter | `graph/testkit/src/dynamic_catalog.rs` | Rust | yes |
| Current compatibility baseline | `graph/test-results/REPORT.md` | Markdown | yes |

# OUTPUTS

## Milestones 1-2 implementation

| Output | Location | Format | Acceptance criteria |
|--------|----------|--------|---------------------|
| Stable semantic identities | `graph/ir/src/identity.rs` and catalog persistence | Rust | Existing conceptual ID types are persisted rather than derived from physical positions, remain distinct from source IDs, and are covered by tests. |
| Additive semantic catalog | `graph/frontend/src/catalog.rs` or a focused sibling module | Rust | Tables and registration API are additive, atomic, idempotent for identical input, and reject invalid mappings before writes. |
| Semantic catalog snapshot | `graph/frontend/src/schema_catalog.rs` or focused sibling | Rust | Conceptual resolution is independent of table/source spelling; physical layouts remain behind `RelationalCatalogSnapshot`. |
| Binder semantic typing | `graph/frontend/src/binder.rs` | Rust | Read and mutation bindings retain possible semantic type sets and emit typed span-bearing ownership/ambiguity errors. |
| Runtime write validation | `graph/frontend/src/mutation.rs` | Rust | Dynamic values/maps and binary endpoints are validated before mutation; failures leave data unchanged. |
| Public additive API | `graph/frontend/src/lib.rs` | Rust | Existing `GraphRegistration` callers compile unchanged; semantic registration has documented types and errors. |
| Regression tests | inline tests plus `graph/frontend/tests/` | Rust | Cover catalog, binder, executor, atomicity, compatibility, and physical/conceptual name independence. |
| User documentation | `docs/graph.md` and concise link/update in `graph/README.md` | Markdown | Explains opt-in guarantees and explicit non-goals without TypeDB compatibility claims. |

## Required test matrix

At minimum add tests for:

1. Two conceptual node types with names unrelated to one shared physical table.
2. Conceptual relationship type with a name unrelated to its relationship table.
3. Stable property IDs independent of column order and different physical column names per owner.
4. Registration rejection for missing tables/columns, structural columns, kind mismatch, duplicate names, incompatible shared property types, and invalid endpoints.
5. Legacy graph behavior unchanged when no semantic schema exists.
6. Valid typed reads, property predicates, creates, merges, sets, removals, replacements, `ON CREATE`, and `ON MATCH`.
7. Bind-time rejection of unowned properties, ambiguous targets, missing/unknown mutation types, and wrong statically known values.
8. Runtime rejection for wrong parameter values and dynamic map keys/values, with zero partial writes.
9. Relationship endpoint validation in outgoing and incoming syntax.
10. Catalog reload across a new connection and stale traversal-snapshot invalidation/rebuild if the catalog version changes.
11. Schemaless donor adapter and existing graph frontend suites remain green.

# CONSTRAINTS

## MUST

- Implement Milestones 1-2 as an opt-in additive capability.
- Keep conceptual IDs, physical source IDs, and physical column names as separate concepts.
- Derive physical value typing through core `Schema::classify_column` and current `SchemaCatalog` helpers.
- Validate a complete semantic registration before writing catalog rows; use a transaction or equivalent atomic core API.
- Preserve source spans in semantic bind errors.
- Validate every mutation route, including dynamic maps and staged `WITH`/`FOREACH`/`MERGE` branches.
- Keep runtime validation and physical mutation in one transaction so validation failures cannot partially apply.
- Add tests that fail without each implementation slice and explain the invariant protected.
- Keep public APIs additive and document all new public types.
- Use `rtk` for repository commands and conventional commits.

## MUST NOT

- Add TypeQL parsing, TypeQL syntax, or a TypeQL compatibility layer.
- Claim TypeDB, PERA, hypergraph, role-interface, or inference compatibility.
- Change canonical graph storage from user-owned Turso tables in Milestones 1-2.
- Represent first-class attribute instances, multi-valued ownership, native n-ary relations, or relation-to-relation roles.
- Add an IR operator before proving existing `Union`/filter composition is inadequate.
- Put physical table/column names into graph IR.
- Create a second independent primitive/custom type classifier.
- Make binder-only guarantees that direct SQL can bypass without documenting the boundary.
- Modify imported donor corpora to make tests pass.
- Broaden conformance work unrelated to semantic schema.

## SHOULD

- Put semantic catalog code in a focused module if `catalog.rs` or `schema_catalog.rs` would become harder to review.
- Reuse existing conceptual ID newtypes where their invariants fit, and use typed enums for type kind and endpoint kind.
- Precompute immutable lookup maps once per catalog snapshot rather than querying internal tables during each property bind.
- Keep legacy fallback explicit in one catalog boundary rather than scattering mode checks through binder and lowerer.
- Prefer precise errors such as `PropertyNotOwned`, `AmbiguousSemanticType`, `AbstractTypeInstantiation`, `IncompatiblePropertyType`, and `InvalidEndpointType` over generic unsupported errors.
- Measure prepare/bind overhead and catalog snapshot construction separately from execution.

# IMPLEMENTATION PIPELINE

Each slice is scoped to at most about 35 minutes of focused agent work. Complete and verify each slice before continuing. Do not parallelize slices that edit the same catalog or binder contracts.

## Phase 0 — baseline and design checkpoint

### Slice 0.1: establish baseline

- Run targeted frontend, IR, and testkit tests.
- Record existing failures without changing code.
- Confirm `.playwright-mcp/` or other unrelated files remain untouched.

### Slice 0.2: write catalog mini-design

- Write the exact additive table DDL, public registration structs, ID ownership, compatibility mode, and transaction boundary in the implementation PR/commit notes.
- Trace all `GraphCatalogSnapshot` and `RelationalCatalogSnapshot` implementors.
- Checkpoint: reviewer approves table keys, case-folding behavior, and no-breaking-change API before implementation.

## Phase 1 — semantic identities and catalog

### Slice 1.1: stabilize semantic identity contracts

- Confirm `LabelId`, `RelationshipTypeId`, and `PropertyId` are the conceptual identity contracts; persist them in the semantic catalog and add invariant tests without introducing duplicate newtypes.
- Do not yet change binder behavior.

### Slice 1.2: create additive semantic tables

- Add constants, DDL, uniqueness/check constraints, and table-creation tests.
- Decide and test catalog/snapshot version handling.

### Slice 1.3: add registration input validation

- Define additive registration structs and typed catalog errors.
- Validate names, sources, columns, entity kinds, and endpoint references in memory.

### Slice 1.4: make registration atomic and idempotent

- Insert a validated schema transactionally.
- Identical replay succeeds without changes; conflicting replay returns a typed error and leaves rows unchanged.

### Slice 1.5: load immutable semantic snapshot

- Load type, mapping, property, ownership, and endpoint maps once.
- Derive mapped column value types with current core schema classification.
- Add reopen/reload tests.

## Phase 2 — binder integration

### Slice 2.1: extend catalog resolution contracts

- Return semantic type candidates and owner-specific property resolutions without leaking physical names.
- Update all mock/test/dynamic catalog implementations with explicit legacy behavior.

### Slice 2.2: track binding semantic type sets

- Extend binder entity metadata and narrow sets through node labels and relationship types.
- Add read-only binder tests before mutation changes.

### Slice 2.3: validate read property ownership

- Resolve properties against possible semantic owner types.
- Emit ownership or ambiguity errors with spans.

### Slice 2.4: validate CREATE/MERGE type selection

- Require one concrete semantic type in strict mode.
- Carry the selected semantic type through mutation binding metadata.

### Slice 2.5: validate typed mutation properties

- Cover create properties, set/remove, literal map replacement, and merge branches.
- Add static type compatibility checks without rejecting `Any` prematurely.

### Slice 2.6: validate binary endpoints

- Check start/end permitted semantic types for both directions.
- Add targeted relationship creation and merge tests.

## Phase 3 — runtime dynamic-value validation

### Slice 3.1: validate parameter/Any property values

- Introduce the smallest typed runtime validator shared by create/set/merge paths.
- Fail before issuing physical mutation SQL for the row.

### Slice 3.2: validate dynamic property maps

- Resolve every map key against ownership and validate every value.
- Prove unknown keys and late invalid values leave all rows unchanged.

### Slice 3.3: validate staged and multi-row atomicity

- Exercise `WITH`, `FOREACH`, `UNWIND`, and multiple matched rows.
- Prove one invalid row aborts the whole graph mutation transaction.

## Phase 4 — compatibility, documentation, and performance

### Slice 4.1: legacy compatibility

- Run existing graph frontend and testkit tests.
- Confirm donor `DynamicCatalog` remains schemaless and no corpus fixtures were edited.

### Slice 4.2: document the API and integrity boundary

- Add registration example, validation examples, legacy behavior, direct-SQL limitation, and non-goals.

### Slice 4.3: add prepare-time benchmarks

- Benchmark legacy versus semantic catalog snapshot/open and representative typed query preparation.
- No hard pass/fail threshold until a baseline exists; report allocations/time and prevent accidental per-property catalog SQL.

### Slice 4.4: final gates and commits

- Run all verification commands.
- Review public API and internal table naming.
- Split commits by coherent phase with conventional titles and descriptive bodies.

# VERIFY

## Automated checks

Run from the repository root:

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

If full-workspace Clippy fails outside the touched graph crates, record the exact pre-existing diagnostic and additionally run:

```bash
rtk cargo clippy -p turso_graph_ir -p turso_graph_frontend -p turso_graph_testkit --all-features --all-targets -- --deny=warnings
```

Do not record a new conformance baseline merely to implement this feature. After Milestones 1-2 are complete and targeted gates pass, run an intentional recorded conformance and benchmark baseline as a separate commit so measurement changes remain reviewable.

## Milestones 1-2 success criteria

- [ ] Semantic type/property IDs are persisted and independent of physical source/column positions.
- [ ] Two differently named conceptual types can share one physical source.
- [ ] Existing `GraphRegistration` callers and legacy graphs behave unchanged.
- [ ] Strict semantic reads reject unowned or ambiguous properties with source spans.
- [ ] Every mutation path rejects unowned properties, incompatible values, missing/unknown/ambiguous types, and invalid endpoints.
- [ ] Dynamic validation failures cause zero partial writes.
- [ ] No binder or IR value contains physical table/column names.
- [ ] Reopening the database reconstructs the same semantic identities and mappings.
- [ ] Traversal snapshots cannot silently use incompatible catalog identities.
- [ ] Tests, formatting, targeted Clippy, smoke corpus, and non-recorded corpus pass.
- [ ] Documentation states the direct-SQL integrity boundary and all deferred semantics.

## Later milestone entry criteria

- Milestone 3 starts only after Milestones 1-2 have a reviewed public API and prepare-time measurements.
- Milestone 4 starts only after inheritance semantics and direct-SQL enforcement policy are explicit.
- First-class attributes, n-ary relations, and inference each require their named ADR decision gate.

## Failure conditions

- Existing graphs require semantic catalog rows or change behavior without opt-in.
- A conceptual label/type identity is still computed from source-list position in semantic mode.
- A semantic property ID is still a physical column ordinal.
- Invalid dynamic values or map keys can partially mutate data.
- Multiple semantic types share a source but binder/lowering silently chooses the wrong physical mapping.
- Endpoint checks ignore query direction.
- Catalog changes can leave existing snapshots apparently current with incompatible IDs.
- The implementation introduces TypeQL, attribute-instance storage, native n-ary relations, or inference.
- Documentation implies TypeDB/PERA compatibility.

# RISKS AND MITIGATIONS

| Risk | Impact | Required mitigation |
|------|--------|---------------------|
| Public struct breakage | Existing users and tests stop compiling | Separate additive semantic registration API; do not add required `GraphRegistration` fields. |
| Catalog identity drift | Prepared plans/snapshots address wrong types | Persist IDs, reload deterministically, version snapshots, and test reopen/staleness. |
| Source/type ambiguity | Wrong table chosen for a conceptual type | Carry explicit type-to-source mapping; reject ambiguous mutation targets in Milestones 1-2. |
| Physical schema drift | Ownership points at missing/incompatible columns | Validate on registration/open and fail loudly; define schema-generation/reprepare behavior. |
| Partial mutation | Semantic integrity violated | Validate within the same transaction and test multi-row rollback. |
| Direct SQL bypass | Catalog promises exceed enforcement | Reuse native constraints where exact and document graph-frontend-only guarantees until ownership enforcement exists. |
| Binder complexity | Large regressions in current Cypher support | Add semantic mode behind the catalog boundary and preserve explicit legacy resolution. |
| Hot-path catalog queries | Prepare latency regression | Build immutable maps once and benchmark snapshot/open plus prepare. |
| Premature hypergraph design | Large storage/IR rewrite with no proven workload | Keep binary endpoint subset; require ADR for native roles/n-ary relations. |

# NOTES

The implementation should use TypeDB documentation as semantic inspiration and Turso's current frontend architecture as the source of truth. Where the two conflict, preserve Turso's storage and Cypher model, state the narrower guarantee, and add a decision record instead of approximating a feature under a borrowed name.
