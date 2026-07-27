# Task 8 Report: Semantic roles

Commit: `3c7dccf9b09dd68ee2b281c496eb8563741fa382`
Branch: `feature/graph-nary`, starting HEAD `536a37291f72`

## Summary

Semantic-mode relationship types used to declare `EndpointConstraint { start,
end }`, hard-coding binary arity into the semantic layer. That's replaced by
`SemanticRole { role: ir::RoleId, name: String, targets: Vec<ir::RoleTarget>,
optional: bool, cardinality: ir::RoleCardinality }`, one persisted row per
(role, target) in the renamed `__turso_internal_graph_semantic_roles` table
(`SEMANTIC_ROLE_TABLE`, replacing `SEMANTIC_ENDPOINTS_TABLE`), keyed by a
`target_kind` discriminator (`'node'`/`'relation'`) so the node-label and
relationship-type id spaces never collapse into one integer space. A role's
`RoleId` is `physical_role.role` reused directly, both when persisting rows
and when reconstructing `SemanticRole`s at load time, so the semantic and
physical role ids can never drift apart by construction — no re-derivation,
no separate id allocator.

`GraphCatalogSnapshot::relationship_endpoints` is replaced by
`relationship_roles(ty: ir::RelationshipTypeId) -> Vec<SemanticRole>` (no
`graph` parameter — this matches the brief's own Step 5 signature verbatim,
not a deviation from it) with a default `relationship_role(ty, name)`.
`relationship_endpoint_sources` (a distinct, physical/MATCH-expand-only
method) was left untouched, per explicit instruction. `BindError::
InvalidEndpointType` is replaced by `RoleTargetTypeViolation { relationship_
type, role, found, span_start, span_end }`; the CREATE-relationship binder
logic now looks up the `start`/`end` roles by name via `relationship_role`
instead of reading dedicated struct fields.

Two carry-forward defects fixed: (a) two `.expect()` panics on any
non-binary relation reaching semantic validation, replaced by a proper
`UnknownPhysicalRole` error; (b) `check_owned_columns`'s independently
re-derived structural-column set, unified with `RegisteredRelationshipSource
::single_valued_roles()` so the two can no longer silently disagree about
which columns are role-owned.

A 2-role, all-required, all-`One` relationship still lowers to the same
physical shape and SQL as before this change — nothing here special-cases
role count (confirmed: no `roles.len() == 2` / `is_binary` anywhere in the
diff).

## Corrections applied vs. the brief / overrides

1. `relationship_roles(ty)` takes no `graph` parameter — matching the
   brief's own Step 5 snippet exactly. **Correction (fix round 1):** the
   original report wrongly called this a "documented deviation from the
   brief." It is not a deviation; the brief's own signature already omits
   `graph`.
2. `relationship_endpoint_sources` was explicitly out of scope (a distinct
   physical/MATCH-expand method) and was not touched.
3. Schemaless mode's default `relationship_roles` returns `Vec::new()`
   rather than literally "synthesizing two required start/end roles with
   empty target lists" as the brief's Step 5 prose describes. This is
   behavior-identical: the CREATE-relationship binder loop
   (`binder.rs:1582-1600`) does `let Some(role) = ...relationship_role(...)
   else { continue };`, and separately treats an empty `targets` list as "no
   constraint, skip" (`if allowed.is_empty() { continue; }`). A synthesized
   role with empty targets and an absent role converge on the exact same
   "no type check performed" outcome through this code, so the simpler
   `Vec::new()` default was kept rather than adding a synthesis path with no
   observable difference.
4. `SEMANTIC_ROLE_TABLE`'s actual DDL includes `graph_id INTEGER NOT NULL`
   (absent from the brief's snippet) and is keyed by `(role_id, ...)` rather
   than the brief's `ordinal`, matching the existing table family's
   convention of storing the physical `RoleId` directly rather than a
   position — needed for the RoleId-identity design in point above.
5. `CatalogRows`/`SemanticSnapshot` load path is a grouped join
   (`load_roles`, new function) rather than the una-row-per-role reading
   implied by the brief's snippet, because a role with an empty target list
   produces zero persisted rows and must still surface as a `SemanticRole`
   (recovered from the physical role list, joined as the left side).

## TDD: an honest account, not a fabricated red-green log

Implementation and the brief's two mandated tests were **not** written in
strict red-green order in this session. The prior session segment had
already driven every non-test file (`binder.rs`, `schema_catalog.rs`,
`semantic_constraints.rs`, `lib.rs`, `semantic.rs`'s production code) to a
compiling, feature-complete state before this segment wrote
`a_semantic_role_carries_targets_optionality_and_cardinality`,
`a_role_may_target_a_relationship_type`, and
`semantic_role_id_matches_the_physical_role_id`. Practically, the crate had
to be brought to a compiling state first (28 pre-existing compile errors
from the earlier edit batch had to be fixed before any test could even
build) — see the genuine, actually-observed red states below, which are
real but earlier in the sequence than the brief's own Step 1/2.

**Actually observed red state #1** (genuine, this session, before any of
the fixes in "Errors and fixes" below): `cargo check -p turso_graph_frontend
--tests` failed with 28 errors across `schema_catalog.rs`, `semantic.rs`'s
own `mod tests`, and `graph/frontend/tests/semantic_schema.rs` — old
`EndpointConstraint`/`SemanticRelationshipType { start, end, .. }` struct
literals, `SEMANTIC_ENDPOINTS_TABLE` imports, `InvalidEndpointType`/
`EndpointSourceMismatch`/`UnknownEndpointType` match arms, and `.endpoints()`
calls, all no longer present on the new types. All 28 were fixed (see the
prior segment's summary for the itemized list); the crate reached 0 errors
before this segment began.

**Actually observed red state #2**, when the three new tests were first
compiled in this segment: `cargo check -p turso_graph_frontend --lib`
returned 0 errors on the very first attempt, and `cargo nextest run -p
turso_graph_frontend --lib -E 'test(semantic::)'` reported all 9 tests
(including the 3 new ones) passing immediately — no compile failure, no
assertion failure, on first run. **This is not a genuine red state for
these three specific tests** — the implementation they exercise was already
correct and complete before they were written, so there was nothing left to
turn red.

**To close that gap honestly rather than paper over it**, I sabotaged the
implementation after the fact and re-ran the affected test to get a real,
verbatim failing assertion, then restored the correct code and confirmed
`git diff` showed zero difference from the committed version:

Sabotage (temporary, in `graph/frontend/src/semantic.rs`'s `load_roles`, the
`"relation" =>` match arm): forced a persisted relation-target row to be
misread back as `ir::RoleTarget::Node` instead of `ir::RoleTarget::Relation`.

```
$ cargo test -p turso_graph_frontend --lib semantic::tests::a_role_may_target_a_relationship_type -- --nocapture
running 1 test

thread 'semantic::tests::a_role_may_target_a_relationship_type' panicked at graph/frontend/src/semantic.rs:3078:9:
cited must accept a relation player, got [Node(LabelId(1))]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test semantic::tests::a_role_may_target_a_relationship_type ... FAILED

failures:
    semantic::tests::a_role_may_target_a_relationship_type

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.04s
error: test failed, to rerun pass `-p turso_graph_frontend --lib`
```

Restored the original file (`cp` from a pre-sabotage backup) and confirmed
`git diff -- graph/frontend/src/semantic.rs` showed no output (byte-identical
to the committed version) before re-running green:

```
$ cargo test -p turso_graph_frontend --lib semantic::tests::a_role_may_target_a_relationship_type -- --nocapture
running 1 test
test semantic::tests::a_role_may_target_a_relationship_type ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.04s
```

This proves the test actually exercises relation-as-player behavior rather
than being vacuously true, even though the historical red-green ordering
for this specific test was not observed live.

### Full semantic module test run (verbatim, via cargo-nextest)

```
$ cargo nextest run -p turso_graph_frontend --lib -E 'test(semantic::)'
────────────
 Nextest run ID 64aecd63-0cc7-4861-9513-a97a1e89674c with nextest profile: default
    Starting 9 tests across 1 binary (141 tests skipped)
        PASS [   0.010s] (1/9) turso_graph_frontend semantic::tests::fragment_registration_round_trips_through_serde_json
        PASS [   0.010s] (2/9) turso_graph_frontend semantic::tests::duplicate_type_names_are_rejected_case_insensitively
        PASS [   0.010s] (3/9) turso_graph_frontend semantic::tests::empty_names_are_rejected
        PASS [   0.010s] (4/9) turso_graph_frontend semantic::tests::registration_round_trips_through_serde_json
        PASS [   0.010s] (5/9) turso_graph_frontend semantic::tests::duplicate_property_names_within_one_owner_are_rejected
        PASS [   0.010s] (6/9) turso_graph_frontend semantic::tests::relationship_endpoints_must_reference_declared_node_types
        PASS [   0.030s] (7/9) turso_graph_frontend semantic::tests::a_role_may_target_a_relationship_type
        PASS [   0.030s] (8/9) turso_graph_frontend semantic::tests::a_semantic_role_carries_targets_optionality_and_cardinality
        PASS [   0.030s] (9/9) turso_graph_frontend semantic::tests::semantic_role_id_matches_the_physical_role_id
────────────
     Summary [   0.031s] 9 tests run: 9 passed, 141 skipped
```

## Gates

- `cargo fmt` — 4 files had formatting drift (from earlier-segment edits plus
  the new fixture code); applied, then `cargo fmt --check -p
  turso_graph_frontend` clean.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
  — 0 errors, 10 warnings (all outside `graph/*`, pre-existing `ar`
  build-script noise).
- `cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime
  -p turso_graph_cypher` — 340 + 17 = 357 passed, 3 skipped, 0 failed
  (`turso_graph_cypher` is a separate binary from the other three and needed
  its own `nextest run -p turso_graph_cypher` invocation: 17/17 passed).
- `mise run corpus` and `mise run cypherbench-sample` — run **after**
  committing, confirmed `git rev-parse --short HEAD` = `3c7dccf9b` matches
  the run_id tag, per the process fix documented in Task 7's report (a prior
  task in this same plan had reported a corpus result run against a stale
  HEAD; this task avoided repeating that mistake).

### Corpus per-suite comparison

Run: `20260726T064743.471434Z-3c7dccf9b09d-corpus-deep` (tag matches commit
`3c7dccf9b09d` exactly).

| suite | baseline passed | this run passed | moved? |
|---|---|---|---|
| age-deep | 3042 | 3042 | no |
| cqlite-deep | 113 | 113 | no |
| grafeo-deep | 277 | 277 | no |
| sparrowdb-deep | 2164 | 2164 | no |
| tck-deep | 3330-3332 | 3330 | no (within the documented flake band) |

Total: `passed=8926 / 10242`, `clean=false` (the corpus's steady-state known
non-golden failure set — 500 age-deep, 596 tck-deep, 95 grafeo-deep, 61
sparrowdb-deep, 11 cqlite-deep — same categories every prior run, mostly
unimplemented Cypher surface: `db.schema`/`db.index.fulltext.*` procedures,
`vector_*`/`hybrid_search` functions, `shortestPath`, parameter binding gaps
in a few suites' fixtures. `[corpus] ERROR task failed` simply reflects mise
treating a non-`clean` exit as failure; this is the corpus's ordinary
steady state, not a regression). No suite outside `tck-deep` moved.

### Cypherbench per-domain comparison

Same commit (`3c7dccf9b`):

| domain | matched | mismatched | errored |
|---|---|---|---|
| company | 13 | 12 | 0 |
| fictional_character | 14 | 11 | 0 |
| flight_accident | 24 | 1 | 0 |
| geography | 11 | 14 | 0 |
| movie | 6 | 19 | 0 |
| nba | 25 | 0 | 0 |
| politics | 15 | 10 | 0 |

Identical to every prior recorded run for every domain. Parity: yes.

## `check_owned_columns` ruling

Unified at the `RegisteredRelationshipSource` layer, using its own
`single_valued_roles()` method rather than an independently-derived set
(`graph/frontend/src/semantic.rs:1047-1052`):

```rust
let mut structural = vec![source.identity_column.as_str()];
structural.extend(
    source
        .single_valued_roles()
        .map(|role| role.column.as_str()),
);
```

**Correction (fix round 1):** the original report overstated this as "the
single source of truth also used by `RelationshipTableLayout::
structural_columns()`." That is wrong. They are two separate
implementations of the same predicate (identity column + every
single-valued role's column), on two different types:
`RegisteredRelationshipSource::single_valued_roles()` (catalog layer,
`graph/frontend/src/catalog.rs:136-140`) and `RelationshipTableLayout::
structural_columns()` (lowering layer, `graph/frontend/src/lowering.rs:
57-66`, filtering its own `roles: Vec<RelationshipRoleLayout>` by
`cardinality == One` independently). They agree today only because
`RelationshipTableLayout` is populated by field-copying
`RegisteredRelationshipSource`'s roles in `schema_catalog.rs`'s
`relationship_layout()` (`graph/frontend/src/schema_catalog.rs:757-771`) —
a projection, not a shared definition. A later task could change either
filter predicate independently and silently reintroduce a split; nothing
in the type system ties them together.

Judgment, not implemented: a genuine single definition looks cheap-ish but
not free — the two call sites want different ownership (`&str` slice vs.
owned `Vec<String>` field) and live in different modules (`catalog.rs` vs.
`lowering.rs`), so unifying them means extracting a small shared
free-function over `(identity_column, roles-with-cardinality)` that both
call and materialize differently. I did not implement this, per
instruction; flagging it for the coordinator's call.

## `IncompatibleGraphLayout` / renamed-table observation (report only, per
override — not fixed)

`CatalogError::IncompatibleGraphLayout` (`graph/frontend/src/catalog.rs:189,
272`) is scoped entirely to the **physical** layer: `load_registered_graph`
checks for `RELATIONSHIP_ROLES_TABLE`'s existence and, if absent, returns a
clear "graph catalog predates native relationship roles ... there is no
migration, so the graph must be created fresh" error (added by a separate,
earlier task in this plan, commit `008b8caf8`, which explicitly states the
project's fresh-start policy: no legacy reader, no migration path). This
check is unrelated to and unaffected by Task 8's `SEMANTIC_ENDPOINTS_TABLE`
→ `SEMANTIC_ROLE_TABLE` rename — a pre-Task-8 catalog already has
`RELATIONSHIP_ROLES_TABLE` (added earlier in this plan), so it passes this
gate regardless.

The **semantic** layer has no equivalent gate for its own table rename.
`load_semantic_snapshot`'s only existence check is against
`SEMANTIC_TYPES_TABLE` (unchanged by this task), which a pre-Task-8
semantic-mode catalog still has. Loading would then proceed into the new
`load_roles` function, which queries `SEMANTIC_ROLE_TABLE` — a table name
that does not exist in such a catalog (it would instead have
`__turso_internal_graph_semantic_endpoints`). That query fails with SQLite's
raw "no such table: __turso_internal_graph_semantic_roles" error, surfaced
through `SemanticCatalogError::Database(#[from] turso_core::LimboError)` —
**a confusing, generic error, not a clear one**, and inconsistent with the
fresh-start policy the physical layer already established and states
explicitly in its own error text.

**Suggested check** (not implemented, per the override): mirror
`load_registered_graph`'s pattern in `load_semantic_snapshot` — add an
explicit `SEMANTIC_ROLE_TABLE` existence check alongside (or instead of
relying solely on) the existing `SEMANTIC_TYPES_TABLE` check, and return a
new, clearly-worded `SemanticCatalogError` variant analogous to
`CatalogError::IncompatibleGraphLayout` (e.g. naming the same fresh-start
policy) rather than letting the "no such table" error surface unexplained
through `SemanticCatalogError::Database`.

## Files changed

- `graph/frontend/src/semantic.rs` — `SEMANTIC_ROLE_TABLE` DDL and
  read/write paths; `SemanticRole`/`SemanticRoleRegistration`/
  `SemanticRoleCardinality`; `SemanticRelationshipType::{binary, role}`;
  `SemanticTypeInfo::{role, required_roles}`; new `load_roles` function;
  `UnknownRoleTargetType`/`UnknownPhysicalRole`/`RoleSourceMismatch` error
  variants (replacing `UnknownEndpointType`/`EndpointSourceMismatch`);
  `check_owned_columns` call site unified on `single_valued_roles()`; fixed
  pre-existing test `relationship_endpoints_must_reference_declared_node_
  types`; new fixture infrastructure (`Schema`/`NodeSourceSpec`/
  `RoleSourceSpec`/`RelationshipSourceSpec`/`PropertySpec`/`NodeTypeSpec`/
  `RoleSpec`/`RelationshipTypeSpec`, `TERNARY_SCHEMA`, `CITATION_SCHEMA`,
  `connection`, `install_semantic_schema`, `load_semantic_catalog`) and 3 new
  tests.
- `graph/frontend/src/binder.rs` — `GraphCatalogSnapshot::relationship_roles`/
  `relationship_role` replacing `relationship_endpoints`;
  `BindError::RoleTargetTypeViolation` replacing `InvalidEndpointType`;
  CREATE-relationship binder logic rewired to resolve `start`/`end` roles by
  name.
- `graph/frontend/src/schema_catalog.rs` — `relationship_roles` implementation
  (the only trait implementor needing a real method body); test-module
  import/teardown renamed to `SEMANTIC_ROLE_TABLE`.
- `graph/frontend/src/semantic_constraints.rs` — two call sites switched from
  the removed `SemanticSnapshot::endpoints()` accessor to
  `relationship.role(name)` lookups, filtering `RoleTarget::Node` only.
- `graph/frontend/src/lib.rs` — export list updated (`EndpointConstraint`
  removed; `SemanticRole`, `SemanticRoleCardinality`,
  `SemanticRoleRegistration` added).
- `graph/frontend/tests/semantic_schema.rs` — 5 struct-literal call sites
  converted to `SemanticRelationshipType::binary(...)`; field-mutation sites
  updated to `.roles[n].targets`; 2 error-variant match sites updated
  (`RoleSourceMismatch`, `RoleTargetTypeViolation`); 2 message-string
  assertions updated; 3 `.endpoints()` call sites replaced with a new shared
  `role_node_ids` helper.
- `graph/frontend/src/session.rs`, `graph/frontend/tests/fixture.rs`,
  `graph/testkit/src/dynamic_catalog.rs` — **not modified**; all three rely
  on the trait's default `relationship_roles`/`relationship_role` bodies.

## Commit

Staged explicitly (not `git add -A`): `graph/frontend/src/binder.rs`,
`graph/frontend/src/lib.rs`, `graph/frontend/src/schema_catalog.rs`,
`graph/frontend/src/semantic.rs`, `graph/frontend/src/semantic_constraints.rs`,
`graph/frontend/tests/semantic_schema.rs`. `graph/test-results/REPORT.md`,
`benchmarks.jsonl`, `runs.jsonl` confirmed left modified, uncommitted, via
`git status --short` immediately before staging.

Signed commit: `3c7dccf9b09dd68ee2b281c496eb8563741fa382`
("graph/frontend: give relationship types named, typed roles instead of
endpoints").

## Fix round 1

Two Important findings, zero Critical. Both were missing coverage on paths
this task owns, not defects — production code was correct going in.

### Finding 1: the unconstrained-role left join had no test

`load_roles` recovers a role with an empty target list (zero rows in
`SEMANTIC_ROLE_TABLE`) by left-joining physical roles against grouped
persisted rows (`graph/frontend/src/semantic.rs`, `load_roles`, the loop
`for physical_role in &source.roles { match grouped.remove(...) { Some
=> recovered, None => synthesized with empty targets } }`). Nothing
proved this was a left join rather than an inner join.

Added `an_unconstrained_role_survives_the_left_join` plus a new fixture
`UNCONSTRAINED_ROLE_SCHEMA` — same physical shape as `TERNARY_SCHEMA`, but
the semantic registration's `Transcription` type omits a `RoleSpec` for
`witness` entirely (not merely an empty `targets` list — no entry at
all). The test asserts `transcription.roles.len() == 3` (all three
physical roles present) and that `role("witness")` is `Some(..)` with
`targets.is_empty()`.

**Sabotage**: changed the join in `load_roles` from left to inner —
physical roles with no matching persisted group are skipped (`continue`)
instead of recovered:

```rust
for physical_role in &source.roles {
    let role_id = physical_role.role.get();
    let Some(group) = grouped.remove(&(type_id, role_id)) else {
        continue;
    };
    info.roles.push(SemanticRole {
        role: physical_role.role,
        name: group.name,
        targets: group.targets,
        optional: group.optional,
        cardinality: group.cardinality,
    });
}
```

Ran the new test in isolation. Verbatim failure:

```
thread 'semantic::tests::an_unconstrained_role_survives_the_left_join' panicked at graph/frontend/src/semantic.rs:3212:9:
assertion `left == right` failed: witness must survive despite having no semantic entry
  left: 2
 right: 3
```

Restored the original code and confirmed byte-for-byte identity via `diff`
against a pre-sabotage backup before re-running the test green.

### Finding 2: `check_owned_columns`'s generalization was untested beyond binary

The relationship-type call site derives its structural-column set from
`source.single_valued_roles()` (every single-valued role's column, not
just `start`/`end`). The only existing `StructuralColumn` coverage was the
pre-existing binary test in `graph/frontend/tests/semantic_schema.rs`
(`registration_rejects_structural_missing_and_wrong_kind_mappings`,
mapping a property to the `start` role's endpoint column `a`) — which
passed before this generalization too, so it proves nothing about a third
role.

Added `check_owned_columns_protects_a_third_roles_structural_column`,
built directly against `TERNARY_SCHEMA`'s physical shape (not the
`install_semantic_schema` helper, to keep the deliberately-invalid
property isolated): a correct registration for `Transcription`'s three
roles (`scribe`, `folio`, `witness`) plus one extra property mapped to
`folio_id` — the `folio` role's structural column, distinct from
`start`/`end`. Asserts `register_semantic_schema` returns
`Err(SemanticCatalogError::StructuralColumn { .. })`.

**Sabotage**: reverted the call site's `structural` derivation to a
hardcoded start/end-only lookup:

```rust
let mut structural = vec![source.identity_column.as_str()];
if let Some(role) = source.role_by_name("start") {
    structural.push(role.column.as_str());
}
if let Some(role) = source.role_by_name("end") {
    structural.push(role.column.as_str());
}
```

Ran the new test in isolation. Verbatim failure:

```
thread 'semantic::tests::check_owned_columns_protects_a_third_roles_structural_column' panicked at graph/frontend/src/semantic.rs:3342:9:
folio_id is the folio role's structural column and must not be mappable as a property
```

Restored the original code and confirmed byte-for-byte identity via `diff`
against the same pre-sabotage backup before re-running the test green.

### Minor items

- Corrected the two inaccuracies flagged above (the `graph`-parameter
  "deviation" claim, and the "one source of truth" overstatement) inline
  in their original sections rather than as a separate errata list, so the
  ledger reads correctly in place.
- Tidied the four `.roles[0]`/`.roles[1]` positional-index fixture
  mutations in `graph/frontend/tests/semantic_schema.rs` to name-based
  `iter_mut().find(...)` lookups (optional, done anyway — same defect
  class this plan keeps catching).

### Gates (re-run after the fix-round commit)

- `cargo fmt` — clean.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
  — clean.
- `cargo nextest run -p turso_graph_frontend -p turso_graph_ir -p
  turso_graph_runtime -p turso_graph_cypher` — 342 passed, 0 failed, 3
  skipped (280 passed for `turso_graph_frontend` alone, up from 278: the
  two new tests).

### Corpus / cypherbench (produced after committing, per discipline)

Commit `71fa13b7ef08435ba2dda9f935aed61fcf6074a6`, confirmed via
`git rev-parse HEAD` immediately before running `mise run corpus`.

`mise run corpus` run id: `20260726T072023.908155Z-71fa13b7ef08-corpus-deep`
(short SHA `71fa13b7ef08` matches the commit above). Per-suite, identical
to every prior recorded run:

| suite | passed | failed |
|---|---|---|
| age-deep | 3042 | 553 |
| cqlite-deep | 113 | 11 |
| grafeo-deep | 277 | 95 |
| sparrowdb-deep | 2164 | 61 |
| tck-deep | 3330 | 596 |

`mise run cypherbench-sample` (recorded 2026-07-26T07:21:08Z), identical
to every prior recorded run for every domain:

| domain | matched | mismatched | errored |
|---|---|---|---|
| company | 13 | 12 | 0 |
| fictional_character | 14 | 11 | 0 |
| flight_accident | 24 | 1 | 0 |
| geography | 11 | 14 | 0 |
| movie | 6 | 19 | 0 |
| nba | 25 | 0 | 0 |
| politics | 15 | 10 | 0 |

### Commit (fix round 1)

Staged explicitly (not `git add -A`): `graph/frontend/src/semantic.rs`,
`graph/frontend/tests/semantic_schema.rs`. `graph/test-results/REPORT.md`,
`benchmarks.jsonl`, `runs.jsonl` confirmed left modified, uncommitted, via
`git status --short` immediately before staging.

Signed commit: `71fa13b7ef08435ba2dda9f935aed61fcf6074a6`
("graph/frontend: cover the unconstrained-role left join and the ternary
structural-column case").
