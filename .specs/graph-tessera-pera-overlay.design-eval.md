# Design Evaluation — Tessera DSL as Semantic Schema Overlay for the Turso Graph Frontend

Status: evaluation (no implementation). Companion to
`.specs/graph-semantic-schema-overlay.agent-spec.md` (referenced below as "the spec").

## Question evaluated

Can the Tessera DSL (tessera crate v0.6.0) serve as the authoring language for the
graph frontend's planned semantic-schema overlay, delivering PERA-style ontology
capabilities (Tessera's resolved IR payload) — without adding TypeQL anywhere?

## Verdict

Yes, with one architectural rule: **Tessera never enters the turso workspace.**
The spec's `register_semantic_schema` API (Milestone 1, unimplemented today) is the
seam. Tessera lowers its resolved IR onto that API from an adapter crate that lives
in the tessera repo. TypeQL is skipped entirely — `transpile_typeql` is simply never
called; the graph frontend stays Cypher-facing.

```
.tessera source
  → tessera::parse / parse_multi          (tessera repo)
  → tessera::build_ir                      → IrSchema  ("PERA payload" when serialized)
  → tessera-turso adapter: subset check + physical mapping
  → SemanticSchemaRegistration (serde-friendly structs / JSON)
  → turso: register_semantic_schema        (spec Milestone 1 API)
  → turso: binder + runtime validation     (spec Milestone 2)
```

## Why the fit is strong

1. **No attribute-instance impedance.** Tessera models properties as owned fields on
   entities/relations — exactly the spec's property-as-column model. Tessera has no
   identity-by-value attribute instances, so the spec's Decision Gate A (deferred)
   is not even pressured by the DSL.
2. **Fragments flatten cleanly.** `build_ir` flattens fragment fields with provenance.
   That maps directly to ownership declarations `(owner type, property) → column`.
   Fragments are composition, not subtyping — no conflict with Milestone 3.
3. **Conceptual/physical split already matches.** Tessera schemas contain zero table
   or column names, which is precisely the spec's requirement that conceptual
   identity be independent of physical spelling (`RelationalCatalogSnapshot` is the
   only physical seam — verified in `graph/frontend/src/lowering.rs:22-62`).
4. **Deterministic registries.** `SchemaRegistry` uses BTreeMaps; lowering is
   deterministic, so the spec's idempotent-replay registration requirement is easy
   to satisfy.

## What "PERA capabilities" concretely buys

All from the tessera crate, none requiring turso changes beyond Milestones 1-2:

| Capability | Mechanism | Use |
|---|---|---|
| Portable ontology artifact | `serialize_ir` / `deserialize_ir` (msgpack) | Store the payload as an opaque versioned blob next to the semantic catalog rows; re-derive/verify registration on open; ship one artifact to many turso instances |
| Evolution gating | `tessera::diff` → `SchemaDiff::has_breaking_changes()` | Enforce the spec's Milestone 4 "additive only" rule at the adapter: breaking diff ⇒ refuse to re-register |
| Version discipline | `@version` + `validate_version_compatibility` | Reject version decreases before touching the catalog |
| App-layer validation | `registry.validate_entity_params` | Pre-write checks in application code, complementing (not replacing) binder/runtime validation |
| Typed app code | `CodegenBuilder` (Serde backend) | Rust structs mirroring the ontology for code that talks to the graph frontend |
| Docs / lint | `generate_docs`, `lint` | Ontology documentation and quality checks; `@doc`/`@meta` preserved in IR |

Claims boundary (spec MUST NOT): turso documentation never says TypeDB/PERA/TypeQL
compatible. The adapter's documentation says: "Tessera ontologies lower onto the
turso graph semantic catalog **subset**; unsupported features are rejected at
lowering time." The PERA claim lives with tessera, where it is true.

## The physical-mapping gap (only real design gap)

Tessera is purely conceptual; the spec's registration needs type→source and
(owner, property)→column mappings. Resolution, no DSL changes required:

- **Convention default:** entity/relation name snake_cased → table name; field
  name → column name.
- **Override via `@meta`** (repeatable, IR-preserved): item-level
  `@meta("turso.table", "people")`, field-level `@meta("turso.column", "full_name")`,
  relation-level `@meta("turso.start_column", "src_id")` / `@meta("turso.end_column", "dst_id")`.
- Adapter resolves conventions+overrides into explicit mappings; turso's
  registration validates them against `PRAGMA table_info` as the spec already
  requires. Missing table/column fails registration, not query time.

## Subset matrix — what lowers, what is rejected

| Tessera feature | Turso graph frontend | Adapter action |
|---|---|---|
| `entity` + fields | Semantic node type + ownership (M1-2) | Lower |
| `fragment` / `with` | Flattened ownership | Lower (flattened; provenance → `@meta` if wanted) |
| Binary `relation` (exactly 2 required roles) | Endpoint constraints on start/end (M2) | Lower; first role → start, second → end; role *names* preserved only as metadata (honest loss — spec forbids claiming named role interfaces) |
| N-ary relations, optional roles (`role?`) | Deferred (Decision Gate B) | **Reject** with error suggesting reification (node + binary edges) |
| `@card` other than 0..1 / 1..1 | Deferred (Gate B / M4) | Reject or warn-and-drop (pick at implementation; recommend reject) |
| `enum` types | No semantic value constraints until M4 | Lower field as Text; enum variants carried in payload but **unenforced** — must be documented |
| `@unique` `@key` `@range` `@regex` `@values` | M4 constraint work | v1: advisory — optionally verify a matching physical constraint/index exists; never claim enforcement |
| Inheritance | M3 (supertype/abstract) | **DSL gap: Tessera has no subtyping.** Fragments are mixins, not an ancestor chain — do not fake it. Future tessera work (`extends`) gated on turso M3 landing |
| `ref(T)` field | Properties are scalar columns; references are edges | **Reject**; require an explicit `relation` |
| `list(T)` scalar | `ValueType::List` via array-typed columns (`schema_catalog.rs` array path) | Lower iff backing column is array-typed with matching scalar element |
| `map` | `ValueType::Map` is expression-only, never a persisted column type | **Reject** v1 (JSONB columns exist but only as a lowering read hint, not a typed property) |
| `blob`, `document` | Bytes | Lower to Bytes (document loses CRDT semantics — document it) |
| `string` `text` `uri` `mime` `datetime` | Text affinity | Lower; check below |
| `int` `timestamp` `bool` | Integer affinity | Lower |
| `float` | Real affinity | Lower |
| `decimal` `duration` | No exact native type | Reject v1 or map to Text/Integer with explicit warning (recommend reject; revisit) |
| `action` / `event` | No execution engine in graph frontend | Ignore for registration (still available to codegen/docs); do NOT map to anything |

**Type-checking direction (spec-critical):** the spec mandates value types derive
from core `Schema::classify_column` and forbids a second classifier. Therefore the
adapter does *compatibility assertion only*: tessera type → expected `ir::ValueType`
(table above), and registration fails when the classified physical column type
disagrees. Tessera types never become a runtime type system inside turso.

## Verified integration facts (from current code)

- `register_semantic_schema` does not exist yet; the spec is the plan. Current
  registration is `GraphRegistration` → `register_graph`
  (`graph/frontend/src/catalog.rs:26-48, :125`), transactional.
- IDs are position/ordinal-derived today (`schema_catalog.rs:256-308`) — exactly
  what spec Milestone 1 replaces with persisted IDs. The adapter depends on M1
  being done first; there is nothing stable to target today.
- **Load-bearing prerequisite:** `validate_registration_names`
  (`catalog.rs:543-596`) hard-rejects more than one node source and one
  relationship source per graph (`MultipleSourcesUnsupported`), and binder paths
  resolve the singular source. Any real ontology has many entity types over many
  tables, so M1/M2's per-semantic-type source resolution is not optional polish —
  it is the biggest lift and gates the whole overlay.
- Endpoints are binary everywhere (`RelationshipSourceRegistration`,
  `FixedExpand`, `CreateRelationship`, `RelationshipTableLayout`) — confirms the
  n-ary rejection above.
- The turso workspace has **zero external git dependencies**; tessera is a private
  git crate. Pulling tessera into turso would set a new precedent and couple a
  production DB fork to a private toolchain. Wrong direction.

## Recommended architecture

**Dependency direction: adapter depends on turso, never the reverse.**

1. **Turso side — implement spec Milestones 1-2 exactly as written**, tessera-blind,
   with one cheap addition: make the semantic registration input structs
   `serde::{Serialize, Deserialize}` from day one. That makes the registration
   describable as pure data (JSON/msgpack) and lets *any* external toolchain author
   schemas — tessera today, anything else later. Zero coupling cost.
2. **Tessera repo — new `tessera-turso` adapter crate**:
   `fn lower(ir: &IrSchema, mapping: &PhysicalMapping) -> Result<SemanticSchemaRegistration, LowerError>`
   plus the subset validator and `@meta`/convention mapping resolver. It either
   depends on `turso_graph_frontend` via git for the typed structs, or emits the
   serde JSON with no dependency at all (preferred for maximum decoupling).
3. **Payload persistence (phase 3):** store `serialize_ir` bytes as an opaque
   versioned blob in the semantic catalog. On re-registration, `deserialize_ir`
   the stored payload, `diff` against the incoming registry, and refuse breaking
   changes — this operationalizes the spec's Milestone 4 evolution rule with
   machinery tessera already ships.
4. **Future (gated):** tessera `extends` syntax for turso M3 inheritance; flow
   `@unique`/`@range`/`@regex`/`@values`/`@card` into turso M4 constraints when
   that catalog exists.

## Sequencing

| Phase | Work | Where | Gate |
|---|---|---|---|
| A | Spec M1-2 (semantic catalog, persisted IDs, multi-source binder, validation) with serde-able registration structs | turso | spec's own success criteria |
| B | `tessera-turso` adapter: subset validator, physical mapping, lowering, round-trip tests against a real turso DB | tessera repo | A merged |
| C | PERA payload persistence + diff-gated re-registration | adapter + small additive turso blob table | B |
| D | Inheritance (`extends`), constraint flow-through | both | turso M3/M4 |

## Risks specific to the overlay

| Risk | Mitigation |
|---|---|
| Adapter silently degrades features (enums, roles, constraints) and users assume enforcement | Every degradation is either a hard rejection or an explicit warning list returned from `lower()`; docs enumerate unenforced semantics |
| Tessera IR format drift vs stored payloads | Payload is versioned (tessera already targets forward-compat msgpack); adapter refuses payloads newer than itself |
| Two sources of truth (payload blob vs catalog rows) | Catalog rows are authoritative for turso; payload is provenance/evolution input only; adapter verifies they agree on load |
| Private-dep coupling | JSON interchange keeps turso buildable with no tessera anywhere |
| Role-name loss on relations misleads modelers | Adapter docs state start/end mapping rule; role names retained in `@meta` for docs only |

## Explicit non-goals (inherited + new)

- No TypeQL parsing, syntax, or transpilation anywhere in the pipeline.
- No TypeDB/PERA compatibility claims in turso; subset-lowering claims only, in the adapter.
- No n-ary relations, attribute instances, inference, or named role interfaces.
- No tessera dependency in the turso workspace.
