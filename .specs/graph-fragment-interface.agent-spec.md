---
task_id: graph-fragment-interface
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

# Graph fragment-interface registration and scans

**Status: complete and validated (2026-07-23).**

## Required skills

- `rust`
- `rust-best-practice`
- `code-quality`
- `testing`
- `pr-workflow`

The implementing agent must read the public exports, immediate callers, and
shared catalog/lowering utilities before editing.

## Outcome

Extend the opt-in semantic graph schema with composable node fragments. A
fragment is a graph-scoped, stable, uninstantiable interface whose declared
properties are contributed to each member node type and whose Cypher label
selects the union of its concrete member types.

## Public API freeze

The existing `GraphRegistration`, `SemanticSchemaRegistration`,
`SemanticNodeType`, and `SemanticRelationshipType` layouts remain unchanged.
Fragment support is additive:

```rust
pub fn register_semantic_schema_with_fragments(
    connection: &Arc<Connection>,
    graph_name: &str,
    schema: &SemanticSchemaRegistration,
    fragments: &SemanticFragmentRegistration,
) -> Result<(), SemanticCatalogError>;

pub struct SemanticFragmentRegistration {
    pub fragments: Vec<SemanticFragment>,
}

pub struct SemanticFragment {
    pub name: String,
    pub properties: Vec<String>,
    pub members: Vec<SemanticFragmentMember>,
}

pub struct SemanticFragmentMember {
    pub node_type: String,
    pub properties: Vec<SemanticProperty>,
}
```

`register_semantic_schema` remains the fragment-free convenience API. The
fragment-aware entry point registers one complete definition atomically. It may
also add the first fragment definition to an already registered, identical
fragment-free schema; subsequent changes or removals conflict. Identical replay
is idempotent.

Each fragment property name is a conceptual declaration. Every member must map
every declared property exactly once to a physical column on that member
type's source. A member may not add undeclared fragment properties.

## Catalog and snapshot

- Persist fragments in an additive graph-scoped table with stable positive
  `LabelId` values that do not collide with concrete node-type IDs.
- Persist type-to-fragment memberships separately from contributed property
  mappings, so propertyless fragments remain representable.
- Persist each fragment's declared semantic property identities separately, so
  snapshot loading can reject missing or undeclared member mappings.
- Persist each fragment property mapping with fragment, member type, semantic
  property, physical source, and physical column identities.
- Load membership sets and contributed properties once into the immutable
  `SemanticSnapshot`. Binder and lowerer hot paths perform no catalog queries.
- A fragment name may not collide case-insensitively with any concrete
  semantic type or another fragment.
- Direct and fragment-contributed ownership of the same property on a type is
  valid only when it resolves to the same physical column and compatible value
  semantics.

## Binding and execution

- A fragment label resolves to the intersection of its precomputed concrete
  member set with every other label on the pattern.
- A fragment scan is a `Union` of existing concrete per-type `NodeScan`
  branches. Each branch carries the concrete type label, never the fragment
  label, so rows do not need redundant fragment rows in the label junction.
- Multiple labels are valid only when their intersection contains one or more
  concrete types. A concrete label plus fragments must select that concrete
  type.
- `CREATE` and `MERGE` require one explicitly written concrete label.
  Fragment-only creation is rejected even when the fragment currently has one
  member. Every written fragment label must be carried by that concrete type.
- Property resolution for a fragment binding uses all selected concrete member
  types. Existing all-owner compatibility and ambiguity errors apply.
- Endpoint names may name either concrete node types or fragments. Fragment
  endpoint constraints are expanded to concrete member type IDs before rows
  are persisted.

## Verification

- [x] Public API serialization and legacy registration remain source-compatible.
- [x] Registration rejects name collisions, unknown members, incomplete/extra
  mappings, conflicting contributed ownership, incompatible physical value
  types, and endpoint/source mismatches.
- [x] Identical replay is idempotent; conflicting replay is atomic.
- [x] Reopen reconstructs fragment IDs, memberships, properties, and endpoints.
- [x] Fragment scans union members across physical sources without duplicate rows.
- [x] Concrete-plus-fragment label conjunction narrows correctly.
- [x] Fragment property reads work when every selected member owns the property and
  retain existing ambiguity behavior otherwise.
- [x] Fragment-only mutation fails; concrete-plus-carried-fragment mutation works;
  concrete-plus-unrelated-fragment mutation fails.
- [x] Semantic open and fragment-scan prepare time/allocation measurements are
  reported without regressing the fragment-free open path.

## Non-goals

No inheritance, subtype overrides, abstract concrete types, fragment removal,
membership removal, relationship fragments, recursive catalog lookup, new
storage engine, new graph IR operator, or database-wide integrity claim.
