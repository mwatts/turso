---
task_id: graph-multi-source-registration-binding
complexity: high
risk: high
ambiguity: low
agent_pattern: single
subagent_type: general-purpose
model: opus
isolation: none
tools_required: [file_read, apply_patch, cargo, git]
estimated_tokens: 30000
timeout_minutes: 240
---

# TASK

Implement end-to-end multi-source registration and binding for the Turso graph
frontend without first-source fallbacks, identity collisions, or regressions to
legacy and strict-semantic behavior.

# REQUIRED SKILLS

| Skill | Path | Relevance |
|-------|------|-----------|
| `rust` | `/Users/markwatts/.agents/skills/rust/SKILL.md` | Keep catalog, IR, lowering, and error APIs idiomatic and additive. |
| `rust-best-practice` | `.claude/skills/rust-best-practice/SKILL.md` | Apply repository-mandated Rust implementation and verification rules. |
| `code-quality` | `.claude/skills/code-quality/SKILL.md` | Preserve production database invariants and avoid silent fallback behavior. |
| `testing` | `.claude/skills/testing/SKILL.md` | Add focused regression, conformance, and compatibility coverage. |
| `pr-workflow` | `.claude/skills/pr-workflow/SKILL.md` | Keep commits atomic and run the required publication gates. |

**Directive**: The implementing agent MUST read every skill above in full
before changing code. It MUST read each target file's exports, immediate
callers, and shared catalog utilities before editing.

# CONTEXT

## Codebase

- **Language**: Rust
- **Build system**: Cargo workspace
- **Primary crates**: `turso_graph_ir`, `turso_graph_frontend`,
  `turso_graph_testkit`

## Product boundary

This is the first post-Milestone-2 item from
`.specs/graph-semantic-schema-overlay.agent-spec.md`. It completes the existing
semantic type-to-source seam. It does not implement fragment polymorphism,
constraints, attribute instances, n-ary relationships, TypeQL, or inference.

Semantic types still map one-to-one to physical sources. Multiple semantic
types may share one source, and one registered graph may now contain multiple
node and relationship sources.

## Verified current architecture

- `GraphRegistration` already accepts vectors of node and relationship sources,
  but `validate_registration_names` rejects lengths greater than one through
  `CatalogError::MultipleSourcesUnsupported`.
- `SemanticTypeInfo::source`, `node_source_for_label`, and
  `relationship_source_for_type` already store and resolve per-type mappings.
- `SchemaCatalog` still uses `node_source_entry()` and
  `relationship_source_entry()` in layout, property, and payload resolution.
- Fresh node scans and relationship expansions in `binder.rs` still select the
  first source. `EntityBinding` records type names but not possible sources.
- `NodeScan`, `FixedExpand`, and `GraphExpand` each carry one physical source.
  Existing `ir::Union` can compose shape-compatible per-source branches.
- Lowering records one source in `BindingLayout`. A multi-source union therefore
  needs explicit source provenance and per-branch property materialization;
  treating the first branch's source as universal is incorrect.
- Node-label and relationship-type junction rows are currently keyed only by a
  table-local identity. Equal identities in two sources must not share labels or
  relationship types after multi-source registration.
- Relationship source registrations already declare their physical start and
  end node sources. Traversal branch selection must honor those mappings and
  swap them for incoming direction.
- `DELETE` already enumerates relationship sources for detach checks, but
  binding and junction cleanup still assume a singular entity source.

## Relevant files

| File | Purpose | Access |
|------|---------|--------|
| `.specs/graph-semantic-schema-overlay.agent-spec.md` | Parent milestone contract | read |
| `graph/frontend/src/catalog.rs` | Registration, physical source catalog, junction DDL | read-write |
| `graph/frontend/src/semantic.rs` | Semantic type-to-source snapshot | read-write |
| `graph/frontend/src/schema_catalog.rs` | Immutable name and relational layout resolution | read-write |
| `graph/frontend/src/binder.rs` | Source-aware scans, expansions, and mutation binding | read-write |
| `graph/ir/src/plan.rs` | Per-source plans and union representation | read-write if required |
| `graph/ir/src/mutation.rs` | Source-aware mutation targets | read-write if required |
| `graph/frontend/src/lowering.rs` | Union/source provenance and property lowering | read-write |
| `graph/frontend/src/mutation.rs` | Source-routed execution and junction maintenance | read-write |
| `graph/frontend/src/snapshot.rs` | Traversal snapshot source identity | read-write if required |
| `graph/testkit/src/dynamic_catalog.rs` | Legacy schemaless compatibility adapter | read-write |
| `graph/frontend/tests/semantic_schema.rs` | Strict semantic integration coverage | read-write |
| `graph/frontend/benches/semantic_prepare.rs` | Open/prepare allocation and timing measurements | read-write |

# INPUTS

| Input | Location | Format | Required |
|-------|----------|--------|----------|
| Parent roadmap | `.specs/graph-semantic-schema-overlay.agent-spec.md:508` | Markdown | yes |
| Physical graph definition | `GraphRegistration` | Rust API | yes |
| Semantic mappings | `SemanticSchemaRegistration` | Rust API | yes |
| Existing relational union | `turso_graph_ir::Union` | Rust IR | yes |

# OUTPUTS

| Output | Location | Format | Acceptance criteria |
|--------|----------|--------|---------------------|
| Focused design contract | This file | Markdown | Every named roadmap item has executable success criteria. |
| Multi-source registration | `graph/frontend/src/catalog.rs` | Rust | Multiple sources validate, persist, reload, and retain endpoint mappings. |
| Multi-source binding/lowering | Graph IR/frontend crates | Rust | Typed operations route exactly; unlabeled reads union all eligible sources with provenance. |
| Source-safe mutations | Binder/mutation/lowering | Rust | Reads and writes never select an unrelated first source. |
| Regression coverage | Existing graph test suites | Rust | Covers routing, unions, ownership, endpoints, collisions, reopen, and legacy behavior. |
| Measurements | Existing semantic benchmark | Rust/Divan | Reports single-source versus multi-source open and representative prepare costs including allocations. |

# REQUIREMENTS

## Registration and immutable catalog

1. Remove `CatalogError::MultipleSourcesUnsupported` and its validation paths.
2. Preserve case-insensitive uniqueness across all source names and validate
   every relationship endpoint against the complete node-source set.
3. Reload all registered sources in stable catalog-ID order.
4. Expose plural node and relationship source queries on the immutable catalog
   snapshot. Singular accessors may remain only as compatibility defaults for
   catalogs that actually contain one source; production multi-source paths
   MUST NOT call them.
5. Resolve physical layouts, property columns, JSONB classification, and
   payload columns by the supplied `SourceTableId`, never by the first entry.

## Typed routing

1. A semantic node label selects its mapped node source.
2. A semantic relationship type selects its mapped relationship source.
3. A relationship traversal derives its stored start/end node sources from the
   selected relationship source. Incoming traversal swaps start/end; undirected
   traversal covers both valid orientations.
4. `CREATE`, `MERGE`, `SET`, `REMOVE`, map replacement, and `DELETE` route using
   the bound entity's source, not the graph's first source.
5. A type/source mismatch is a typed bind or lowering error. It MUST NOT return
   an empty result or silently use another source.

## Unlabeled scans and source provenance

1. An unlabeled node pattern in a multi-source graph composes one `NodeScan` per
   eligible node source with `UNION ALL`. Use `UNION ALL` because equal local
   identities in different source tables are distinct graph entities.
2. An untyped relationship pattern composes one expansion branch per eligible
   relationship source and physical orientation. Labeled/type-constrained
   endpoints prune incompatible branches before lowering.
3. A single eligible source keeps the existing non-union plan.
4. Source provenance survives the union as hidden relational state wherever a
   later operation needs it. It must be sufficient to dispatch property reads,
   traversals, type/label functions, mutation writes, and deletes.
5. User-visible result shape and existing entity scalar representation remain
   unchanged. Hidden provenance MUST NOT appear in `RETURN *` or result metadata.
6. Equal numeric identities from different physical sources remain distinct
   throughout filtering, traversal, label/type lookup, and mutation execution.

## Owner-aware properties and endpoints

1. Unlabeled bindings retain the full possible semantic type set. A property is
   readable or writable only when every possible type owns it compatibly, as in
   Milestone 2.
2. Each union branch materializes the resolved property from that branch's own
   source and physical column. JSONB conversion is also branch-specific.
3. Whole-entity `properties()` dispatches by source and returns only the
   semantic properties valid for the binding's possible types.
4. Endpoint validation remains semantic-type-aware and direction-aware across
   different node sources.
5. Physical relationship endpoint source mappings and semantic endpoint
   constraints must agree for each allowed semantic type. Registration rejects
   an impossible combination atomically.

## Junction and snapshot correctness

1. Label and relationship-type membership must be qualified by
   `SourceTableId`; table-local identity alone is insufficient.
2. Scans, `labels()`, `label()`, type lookup, merge predicates, recording, and
   cleanup all use the source-qualified membership key.
3. Existing single-source graphs retain their behavior. If junction DDL changes,
   loading an existing catalog must either migrate it atomically or support both
   layouts explicitly; merely bumping `GRAPH_CATALOG_VERSION` is insufficient.
4. Traversal snapshots continue to use source-qualified node and relationship
   identities and rebuild when catalog compatibility requires it.

# CONSTRAINTS

## MUST

- Keep physical table and column names behind `RelationalCatalogSnapshot`.
- Keep `GraphRegistration` and semantic registration source-compatible.
- Use stable `SourceTableId`, `LabelId`, `RelationshipTypeId`, and `PropertyId`
  identities; do not encode source-list positions into semantic IDs.
- Preserve legacy `DynamicCatalog` behavior and donor corpus fixtures.
- Validate complete registration state before publishing catalog changes.
- Add tests that fail against the pre-change first-source implementation.

## MUST NOT

- Select `.first()` as a fallback when a binding has multiple possible sources.
- Assume identities are globally unique across registered source tables.
- Deduplicate multi-source entities with plain `UNION`.
- Broaden into fragment interfaces, constraints, storage redesign, or a new
  public graph value encoding.
- Record or edit conformance history/baseline files to make verification pass.
- Use release builds.

## SHOULD

- Compose existing per-source operators with `ir::Union`; add a new IR operator
  only if an executable regression proves `Union` cannot preserve the required
  scope and provenance.
- Precompute source/type candidate sets in `SchemaCatalog` rather than querying
  catalog tables in hot binder paths.
- Keep single-source generated SQL and allocations effectively unchanged.

# IMPLEMENTATION PHASES

## Phase 1 — catalog and layout

- Accept/persist/reload multiple sources.
- Add plural immutable catalog access and source-specific relationship endpoint
  lookup.
- Replace first-entry physical layout/property resolution with ID lookup.
- Add catalog and reopen tests before binder changes.

## Phase 2 — typed binding

- Track possible source IDs alongside entity type names.
- Route labeled node scans, typed relationship expansions, creates, merges, and
  subsequent mutations to their exact sources.
- Validate physical endpoint-source compatibility with semantic endpoint types.

## Phase 3 — unlabeled union binding and lowering

- Build `UNION ALL` node-scan and relationship-expansion branches.
- Carry hidden source provenance through lowering.
- Materialize source-specific properties and preserve user-visible shape.
- Make source-qualified label/type functions and junction operations correct for
  colliding local identities.

## Phase 4 — compatibility and measurements

- Extend semantic integration coverage to at least two node sources and two
  relationship sources.
- Add explicit colliding-ID, incoming/outgoing endpoint, owner-aware property,
  mutation, reopen, and legacy tests.
- Extend the Divan benchmark with single-source and multi-source open/prepare
  cases and report time plus allocation deltas.
- Run smoke and non-recorded corpus checks.

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
rtk cargo bench -p turso_graph_frontend --bench semantic_prepare
rtk cargo clippy --workspace --all-features --all-targets -- --deny=warnings
rtk git diff --check
```

Do not record a new conformance baseline. The corpus command may report known
baseline failures; success means the recorded pass/unsupported/fail totals are
unchanged and no result/history files changed.

## Success criteria

- [ ] A graph with at least two node and two relationship sources registers and reopens with stable source IDs.
- [ ] `MultipleSourcesUnsupported` no longer exists.
- [ ] Typed node and relationship reads/writes hit only their mapped tables.
- [ ] Unlabeled node scans return all rows from every source with `UNION ALL`.
- [ ] Untyped relationship scans traverse every compatible relationship source.
- [ ] Equal local identities in different sources do not cross-contaminate labels, types, properties, traversals, updates, or deletes.
- [ ] Incoming, outgoing, and undirected traversal honor physical and semantic endpoint mappings.
- [ ] Owner-aware property rejection and runtime value validation remain atomic across sources.
- [ ] No production multi-source path depends on singular `node_source` or `relationship_source`.
- [ ] Legacy single-source tests, smoke tests, and non-recorded donor corpus behavior remain unchanged.
- [ ] Benchmarks report single-source and multi-source open/prepare time and allocations.
- [ ] Formatting, focused tests, Clippy, and patch hygiene pass.

## Failure conditions

- Registration succeeds but any read/write still defaults to the first source.
- Unlabeled scans omit a source or deduplicate colliding local identities.
- A relationship branch joins an endpoint table not declared by its source.
- Owner-aware property or endpoint validation weakens for multi-source graphs.
- Hidden source provenance leaks into public result shapes.
- Junction membership remains keyed only by local identity.
- Existing graph registration APIs or donor fixtures require migration by callers.
- Verification changes recorded conformance expectations.
