# Specification: Tessera Semantic Overlay for the Turso Graph Frontend

Status: design specification. No code changes are part of this document.

Companion documents:

- `.specs/graph-semantic-schema-overlay.agent-spec.md` (called "the semantic-schema spec" below).
- `.specs/graph-native-capabilities.agent-spec.md` (called "the native-capabilities spec" below).
- `docs/superpowers/plans/2026-07-22-graph-semantic-schema-overlay.md` (the implementation plan for the Turso side).

This document uses Simplified Technical English style. Sentences are short.
Each sentence gives one fact or one instruction. The words MUST, MUST NOT,
and SHOULD have their usual RFC 2119 meanings.

---

## 1. Purpose

This document answers one question. Can the Tessera DSL become the schema
authoring language for the Turso graph frontend, and supply PERA ontology
capabilities, without adding TypeQL and without adding Tessera as a
dependency of Turso?

The answer is yes. This document specifies how.

## 2. Definitions

| Term | Definition |
|---|---|
| Tessera | A Rust library (`tessera` crate, v0.6.0) that parses `.tessera` schema files. It lives in the private repository `github.com/mwatts/tessera`. |
| Tessera DSL | The schema language that Tessera parses. It defines entities, relations, fragments, enums, actions, and events. |
| PERA IR | The resolved, backend-neutral intermediate representation that `tessera::build_ir` produces. The bytes from `tessera::serialize_ir` are the PERA ontology payload. |
| Graph frontend | The Turso crates under `graph/`, mainly `turso_graph_frontend`. It compiles Cypher against user-owned Turso tables. |
| Semantic schema catalog | The opt-in catalog that the semantic-schema spec defines. It stores conceptual types, properties, ownership, and endpoint rules. |
| Semantic registration | The data structure that `register_semantic_schema` accepts. The semantic-schema spec Milestone 1 defines this API. |
| tessera-turso | A planned adapter crate. It will live in the tessera repository. It converts a Tessera schema into a semantic registration for Turso. |

## 3. Top-level requirements

These requirements come from the project owner and from the semantic-schema
spec. They are not negotiable.

1. Tessera MUST NOT become a dependency of any crate in the Turso workspace.
   This includes normal dependencies, dev dependencies, build dependencies,
   and git dependencies.
2. The tessera-turso adapter MUST live in the tessera repository. It MUST
   NOT live in the Turso repository.
3. Changes to `turso/graph` are permitted. Each change MUST be a real
   improvement to the graph frontend on its own. A change that only serves
   Tessera is not acceptable.
4. The graph frontend MUST NOT gain TypeQL parsing, TypeQL syntax, or a
   TypeQL compatibility layer.
5. Turso documentation MUST NOT claim TypeDB, TypeQL, PERA, or hypergraph
   compatibility. The adapter documentation may describe its own PERA
   support, because that claim is true for Tessera.

## 4. Architecture overview

The design has two sides. The Turso side knows nothing about Tessera. The
Tessera side knows about Turso only through a data format.

```
Tessera repository                          Turso repository
------------------                          ----------------
.tessera schema files
        |
        v
tessera::parse / parse_multi
        |
        v
tessera::build_ir  --> PERA IR
        |
        v
tessera-turso adapter
  - checks the supported subset
  - resolves physical table mappings
  - emits a semantic registration
        |
        |   JSON document (the interchange format)
        +----------------------------------->  register_semantic_schema
                                                       |
                                                       v
                                               semantic schema catalog
                                                       |
                                                       v
                                               binder and runtime validation
```

The interchange format is the connection between the two sides. It is a
JSON document. Its structure is the serde serialization of the
`SemanticSchemaRegistration` type. The semantic-schema implementation plan
(Task 1) makes that type serialize and deserialize with serde. Because the
connection is a data format and not a Rust API, neither repository needs a
code dependency on the other to exchange schemas.

The adapter may also link `turso_graph_frontend` directly as a git
dependency inside the tessera workspace. That choice belongs to the
tessera repository. It does not affect Turso.

## 5. Why the fit is good

Four properties of Tessera match the Turso design closely.

1. **Properties are owned fields.** Tessera stores a property as a field on
   an entity or a relation. Turso stores a property as a column on a source
   table. Neither system uses attribute instances with their own identity.
   The semantic-schema spec defers attribute instances behind Decision
   Gate A. Tessera never pushes against that gate.
2. **Fragments flatten cleanly.** Tessera fragments are reusable field
   bundles. `build_ir` flattens fragment fields into each owner and records
   where each field came from. The flattened result maps directly onto the
   ownership rows of the semantic schema catalog. Fragments are
   composition. They are not inheritance. They do not conflict with the
   inheritance work that the semantic-schema spec plans for Milestone 3.
3. **Tessera schemas contain no physical names.** A `.tessera` file never
   names a table or a column. The semantic-schema spec requires the same
   separation: conceptual identity on one side, physical layout behind
   `RelationalCatalogSnapshot` on the other side. The two models agree.
4. **Registries are deterministic.** Tessera stores definitions in ordered
   maps. The adapter therefore produces the same registration for the same
   input every time. The semantic-schema spec requires idempotent
   registration. Deterministic output makes that easy to satisfy.

## 6. What "PERA capabilities" means in practice

The PERA capabilities come from the Tessera crate. They do not require any
Turso change beyond the semantic-schema spec Milestones 1 and 2.

| Capability | How it works | What it gives the user |
|---|---|---|
| Portable ontology artifact | `serialize_ir` produces a msgpack payload. `deserialize_ir` reads it back. | One schema artifact can configure many Turso databases. The artifact can be stored, versioned, and shipped. |
| Evolution gating | `tessera::diff` compares two schema versions. `SchemaDiff::has_breaking_changes()` reports breaking changes. | The adapter can refuse a schema change that would break existing data. This enforces the additive-only rule that the semantic-schema spec sets for Milestone 4. |
| Version discipline | `@version` on a namespace, checked by `validate_version_compatibility`. | A schema whose version number goes backward is rejected before it touches the catalog. |
| Application-level validation | `registry.validate_entity_params` checks a value map against the schema. | Application code can validate input before it sends a mutation to Turso. The Turso binder and runtime still validate independently. |
| Generated Rust types | `CodegenBuilder` with the Serde backend generates structs, builders, and enums. | Application code gets typed structs that mirror the ontology. |
| Documentation and linting | `generate_docs` renders Markdown or HTML. `lint` reports schema quality issues. `@doc` and `@meta` annotations survive into the IR. | The ontology documents itself. |

## 7. The physical mapping problem and its solution

A Tessera schema is purely conceptual. The semantic registration needs two
physical facts: which source backs each type, and which column backs each
owned property. The adapter must supply these facts. No change to the
Tessera DSL grammar is needed.

The adapter resolves physical mappings in two steps.

1. **Convention.** By default, the adapter converts the entity or relation
   name to snake_case and uses that as the source name. By default, the
   field name is the column name.
2. **Override with `@meta`.** The `@meta` directive is repeatable and is
   preserved in the IR. The adapter reads these keys:
   - `@meta("turso.source", "people_src")` on an entity or relation selects
     the registered source by name.
   - `@meta("turso.column", "full_name")` on a field selects the physical
     column.

The adapter only resolves names. Turso validates them. The
`register_semantic_schema` implementation checks every source name and
every column against the registered graph and the real table schema. A
missing table or column fails the registration before any catalog row is
written. This split keeps the trust boundary in Turso, where it belongs.

## 8. Supported subset

The graph frontend supports a subset of what the Tessera DSL can express.
The adapter enforces the subset. Every unsupported construct produces a
clear error or a recorded warning. Silent degradation is forbidden.

### 8.1 Constructs that map directly

| Tessera construct | Turso target |
|---|---|
| `entity` with fields | A semantic node type with owned properties. |
| `fragment` attached with `with` | Extra owned properties on the owner type, after IR flattening. |
| `relation` with exactly two required roles | A semantic relationship type. The first role maps to the start endpoint. The second role maps to the end endpoint. The role names survive only as metadata. |
| `string`, `text`, `uri`, `mime`, `datetime` fields | Properties on Text-typed columns. |
| `int`, `timestamp`, `bool` fields | Properties on Integer-typed columns. |
| `float` fields | Properties on Real-typed columns. |
| `blob` fields | Properties on Blob-typed columns. |

Type checking works in one direction only. The adapter states the expected
Turso value type for each field. Turso derives the actual value type of
each mapped column through `Schema::classify_column`. Registration fails
when the two disagree. Tessera types never become a second runtime type
system inside Turso. The semantic-schema spec forbids a second classifier,
and this design honors that rule.

### 8.2 Constructs the adapter must reject

| Tessera construct | Reason | Error guidance to the user |
|---|---|---|
| Relations with three or more roles | The graph frontend IR and storage are binary. Decision Gate B of the semantic-schema spec defers n-ary support. | Suggest reification: model the relation as a node type plus one binary relation per role. |
| Optional roles (`role?`) | A binary edge always has both endpoints. | Suggest a separate optional relation. |
| `ref(Type)` fields | A reference is an edge in a graph model, not a scalar property. | Suggest an explicit `relation`. |
| `map` fields | The graph frontend has no persisted map property type. `ValueType::Map` exists only for expressions. | Suggest a JSON text column outside the semantic layer, or restructure the data. |
| `decimal` and `duration` fields | Turso has no exact native column type for these. A lossy mapping would betray the declared type. | Suggest `int`, `float`, or `text` with an explicit note. Revisit when Turso adds richer types. |
| `@card` bounds other than `0..1` or `1..1` | Cardinality constraints belong to Milestone 4 of the semantic-schema spec, which does not exist yet. | State that cardinality is not enforced yet. |

### 8.3 Constructs the adapter accepts with a recorded warning

| Tessera construct | Behavior | Warning content |
|---|---|---|
| `enum` fields | The property maps to a Text column. The variant list travels with the payload. | Turso does not enforce the variant set until Milestone 4 lands. |
| `@unique`, `@key`, `@range`, `@regex`, `@values` | The adapter may check that a matching physical constraint or index exists and report the result. | Turso does not enforce these as semantic constraints yet. Physical SQL constraints still apply. |
| `document` fields | The property maps to a Blob column. | The CRDT semantics of the document type are lost inside Turso. |
| `action` and `event` definitions | Ignored for registration. They remain useful for codegen and documentation. | The graph frontend has no execution engine for them. |

### 8.4 The inheritance gap

The Tessera DSL has no subtyping today. Fragments are mixins. A set of
fragments does not form a single ancestor chain. The semantic-schema spec
plans single inheritance for Milestone 3. When Milestone 3 exists, Tessera
needs a new construct, for example `entity Sub extends Super`. That is
future work in the tessera repository. The adapter MUST NOT fake
inheritance by mapping fragments to supertypes. That mapping would be
wrong, because a type can attach many fragments but can have only one
supertype.

## 9. Required Turso-side changes and their standalone value

Every Turso-side change below is part of the semantic-schema spec
Milestones 1 and 2. Each one improves the graph frontend on its own. None
of them mentions or requires Tessera. The implementation plan document
breaks these into twelve tasks with tests.

| Change | Standalone value to the graph frontend |
|---|---|
| Persisted conceptual IDs for labels, relationship types, and properties | Today the IDs come from source-list positions and column ordinals (`schema_catalog.rs:256-308`). Positional IDs break when someone reorders a registration or alters a table. Persisted IDs make prepared plans and snapshots stable. |
| `register_semantic_schema` with atomic, idempotent registration | Any user who wants conceptual names that differ from table names gets that ability. Two conceptual types can share one table. This is useful with or without any external DSL. |
| Serde-serializable registration structs | Any external tool, script, or CI pipeline can author a schema as JSON. This creates a general integration surface. Tessera is one client of it. It will not be the last. |
| Owner-aware property resolution in the binder | The binder gains precise errors: `PropertyNotOwned`, `AmbiguousProperty`, with source spans. Typo detection on property names becomes possible for schema-registered graphs. |
| Strict type selection for CREATE and MERGE | Prevents silently untyped writes on graphs that declare a schema. |
| Endpoint validation for relationships | Prevents structurally wrong edges, in both query directions. |
| Runtime validation of parameters and dynamic maps | Closes the gap where a bad parameter value could reach the SQL layer and be coerced silently. Failures roll back atomically. |
| Generation bump on semantic registration | Reuses the existing snapshot-staleness mechanism, so traversal snapshots can never carry stale identities. |

## 10. Groundwork contract for tessera-turso

The Turso side lays groundwork through exactly one contract: the
interchange format.

1. The `SemanticSchemaRegistration` type and its nested types derive
   `serde::Serialize` and `serde::Deserialize`.
2. The JSON field names are the Rust field names. The implementation plan
   pins this with a round-trip test.
3. The types are additive. New optional fields may appear in later
   versions. Existing fields do not change meaning or disappear.
4. Registration semantics are documented in `docs/graph.md`: validation
   order, idempotency, atomicity, and the error catalog.

With this contract, the tessera repository can build the adapter with no
coordination beyond reading the documentation. The adapter emits the JSON
document. A small loader (in the application, in a CLI, or in the adapter
itself) calls `register_semantic_schema` with the deserialized value.

## 11. Work sequence

| Phase | Work | Repository | Gate to start |
|---|---|---|---|
| A | Implement semantic-schema spec Milestones 1 and 2, following the implementation plan. Include the serde round-trip test. | turso | None. This phase is first. |
| B | Build the tessera-turso adapter: subset checks from section 8, physical mapping from section 7, emission of the interchange JSON, and integration tests against a real Turso database. | tessera | Phase A merged. |
| C | Add payload persistence: store the serialized PERA IR as an opaque, versioned blob next to the catalog rows. On re-registration, load the stored payload, run `tessera::diff` in the adapter, and refuse breaking changes. | tessera, plus one small additive blob table in turso | Phase B complete. |
| D | Extend the Tessera DSL with `extends` for inheritance, and map constraint directives, when the semantic-schema spec Milestones 3 and 4 exist in Turso. | both | Turso Milestones 3 and 4 merged. |

Sequencing note. The native-capabilities spec touches the same files
(`binder.rs`, `catalog.rs`, `schema_catalog.rs`, `session.rs`, `lib.rs`).
Run that stream and Phase A in sequence, not in parallel. Whichever stream
lands second must make `db.propertyKeys()` and the FTS property validation
aware of semantic property names.

## 12. Risks and mitigations

| Risk | Mitigation |
|---|---|
| The adapter degrades a feature silently, and users assume enforcement that does not exist. | Section 8 classifies every construct as mapped, rejected, or warned. The adapter returns the warning list from its main entry point. Documentation lists every unenforced semantic. |
| The stored PERA payload and the catalog rows disagree. | The catalog rows are the authority inside Turso. The payload is provenance and evolution input only. The adapter verifies agreement when it loads both. |
| The Tessera IR format changes and old payloads become unreadable. | The payload uses msgpack with named fields for forward compatibility. The adapter refuses payloads newer than itself, with a clear error. |
| A future contributor adds Tessera as a Turso dependency for convenience. | Requirement 3.1 in this document forbids it. The interchange format removes the temptation, because JSON crosses the boundary already. |
| Role names on binary relations mislead data modelers. | The adapter documents the rule: first role becomes start, second role becomes end. Role names survive as metadata for documentation only. |

## 13. Non-goals

- No TypeQL anywhere in the pipeline, in either repository.
- No TypeDB, PERA, or hypergraph compatibility claims in Turso
  documentation.
- No n-ary relations, no attribute instances, no named role interfaces,
  and no inference in the graph frontend.
- No Tessera code, and no Tessera dependency, in the Turso workspace.
- No multi-source broadening of graph registration as a side effect of
  this work.
