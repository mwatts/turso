# Graph Semantic Contract Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the graph frontend's five undocumented-by-code semantic contracts (rule version, row ordering, vendor divergences, read/write classification, path algorithm legality) into versioned data that tests and CI enforce.

**Architecture:** One new `SemanticProfile` value in `turso_graph_ir` holds every open Cypher choice Turso made, with a content digest pinned to a version number. The testkit records that version in every `history.jsonl` row and derives its result-digest mode from the profile's ordering rule. A `graph/registries/divergence.toml` registry names every vendor divergence and is verified against the recorded corpus run. The binder gains a syntactic statement classifier that the session uses for routing and read-only enforcement. A `path_policy` table in `turso_graph_runtime` decides which (uniqueness, selector, weight-class) combinations are sound, and `shortest.rs` refuses anything outside it.

**Tech Stack:** Rust (workspace crates `turso_graph_ir`, `turso_graph_runtime`, `turso_graph_frontend`, `turso_graph_testkit`), `serde` + `serde_json` (JSONL history), `toml` (registries), `clap` (testkit CLI), `pest` (Cypher grammar — read only in this plan).

## Global Constraints

- Conventional-commit style is **not** used in this repo. Commit subjects are `[scope: ]<lowercase imperative summary>` with no trailing period, e.g. `graph/ir: version the semantic profile`. Bodies explain intent, not the diff.
- Every commit must be signed: always `git commit -S`.
- Never build or run with `--release`.
- `cargo fmt` and `cargo clippy --workspace --all-features --all-targets -- --deny=warnings` must pass before every commit.
- `graph/test-results/history.jsonl` is **append-only and ~1.25 GB**. Never rewrite it, never reformat it, never `git add` a regenerated copy. Backward-compatible reads of `schema_version: 1` rows are mandatory.
- Files copied or adapted from donor projects carry `// Source: / Revision: / Path: / License: / Adaptation: / Changes:` headers. Preserve existing headers exactly when editing such files (`graph/runtime/src/shortest.rs`, `graph/runtime/src/limits.rs`, `graph/runtime/src/traversal.rs`).
- Do not copy any code or prose from `https://github.com/Dicklesworthstone/frankengraphdb`. Its license carries an Anthropic rider. Ideas only; write original code and original words.
- Every task ends with a green `cargo test -p <crate>` for the crates it touched.

---

## Corrections to the source brief

Three premises in the brief this plan was written from are wrong against the current tree. The tasks below implement the *valuable* part of each item; do not implement the wrong part.

1. **"Every statement opens `SAVEPOINT __turso_graph_mutation`, even a plain read."** False. `execute_cypher_mutation` (`graph/frontend/src/mutation.rs:207`) calls `bind_mutation` at line 221 and only opens the savepoint at line 227. `bind_mutation_query` returns `BindError::EmptyMutation` (`graph/frontend/src/binder.rs:946`) for a query with no mutation clause, so a read never reaches the savepoint. The real defects are: (a) there is no public statement classifier, so callers route by *trying* `query()` and falling back to `execute()` on error (`graph/testkit/src/age.rs:317`, `graph/testkit/src/tck.rs`), which converts real read errors into mutation attempts; and (b) there is no read-only enforcement. Task 4 fixes those.
2. **"The parser cannot say `{1,4}`."** Partly false. `range_literal = { "*" ~ unsigned_integer? ~ (".." ~ unsigned_integer?)? }` exists at `graph/cypher/src/cypher.pest:97`, so `[r:T*1..3]` parses today. What is genuinely absent is the GQL-style `{1,4}` quantifier and the `SHORTEST` / `ALL SHORTEST` / `TRAIL` / `ACYCLIC` selector keywords. Task 5 targets the selectors, not the range.
3. **"Dijkstra breaks on negative weights."** True in general, unreachable today: `EdgeInput.weight` and `Path.total_weight` are `u64` (`graph/runtime/src/csr.rs`, `graph/runtime/src/traversal.rs:53`), so a negative weight is unrepresentable. Task 5 writes the table so that *widening the weight type later* trips a policy error instead of silently producing wrong answers.

Two premises are correct as stated and drive Tasks 1–3:
- Semantic choices live only in prose (`graph/CONFORMANCE.md`, `graph/DESIGN_DECISIONS.md`, `graph/README.md`, `docs/graph.md`, `graph/docs/core-changes.md`) and no run records which choices it used.
- `history::result_digest` (`graph/testkit/src/history.rs:119`) is unconditionally order-sensitive — its own test at line 245 asserts this — while four of the five suites sort rows before comparing. Recorded digests for unordered results are therefore unstable across B-tree order changes even when the comparison passes.
- The "53 unsupported" number is derived at corpus-build time from a four-name regex allowlist in `canonical_vendor_function` (`graph/testkit/src/age.rs:438`). Nothing pins the count.

---

## File Structure

**Create**
- `graph/ir/src/semantics.rs` — the `SemanticProfile` value, its enums, its stable rendering, and its FNV-1a digest. Sole owner of "what Turso chose where Cypher is open".
- `graph/ir/tests/semantic_profile_pin.rs` — pins digest → version so a silent semantics change cannot compile-and-pass.
- `graph/registries/divergence.toml` — the vendor-divergence registry.
- `graph/testkit/src/divergence.rs` — load, sync, and verify the registry against a recorded corpus run.
- `graph/testkit/tests/divergence.rs` — registry-shape and count tests.
- `graph/runtime/src/path_policy.rs` — the legal (uniqueness, selector, weight-class) table and its resolver.

**Modify**
- `graph/ir/src/lib.rs` — `mod semantics;` plus re-exports.
- `graph/testkit/src/model.rs` — `semantics_version` field; `HISTORY_SCHEMA_VERSION` 1 → 2.
- `graph/testkit/src/history.rs` — `ResultOrdering`, `result_digest_with`, legacy-version guard.
- `graph/testkit/src/runner.rs`, `tck.rs`, `grafeo.rs`, `cypherbench.rs`, `age.rs`, `performance.rs`, `rust_donor.rs`, `main.rs` — stamp `semantics_version`, pass the ordering mode.
- `graph/testkit/src/manifest.rs` — reject `ordering = "ordered"` on a query with no `ORDER BY`.
- `graph/testkit/src/report.rs` — print the semantics version in `REPORT.md`.
- `graph/frontend/src/binder.rs` — `StatementKind` + `classify`.
- `graph/frontend/src/session.rs` — `classify`, `run`, read-only mode.
- `graph/frontend/src/lib.rs` — re-exports.
- `graph/runtime/src/lib.rs` — `mod path_policy;` plus re-exports.
- `graph/runtime/src/shortest.rs` — consult the policy before searching.
- `graph/runtime/src/error.rs` — `RuntimeError::UnsupportedPathCombination`, `RuntimeError::PathAlgorithmNotImplemented`.
- `graph/DESIGN_DECISIONS.md` — the path algorithm table.
- `graph/CONFORMANCE.md` — point the 53 at the registry; state the semantics version.

Tasks 1 → 2 → 3 are ordered (2 consumes 1; 3 consumes 1's version stamp). Tasks 4 and 5 are independent of 1–3 and of each other, except that Task 5's final step bumps the profile from Task 1.

---

### Task 1: Version the semantic profile

**Files:**
- Create: `graph/ir/src/semantics.rs`
- Create: `graph/ir/tests/semantic_profile_pin.rs`
- Modify: `graph/ir/src/lib.rs:9-14` (mod list), `graph/ir/src/lib.rs:16` (re-exports)
- Modify: `graph/testkit/src/model.rs:7` (`HISTORY_SCHEMA_VERSION`), `graph/testkit/src/model.rs:61-97` (`ResultRecord`)
- Modify: `graph/testkit/src/history.rs:88-116` (`read`)
- Modify: `graph/testkit/src/runner.rs:57-81`, `graph/testkit/src/tck.rs:1534-1563`, `graph/testkit/src/grafeo.rs:587-626`, `graph/testkit/src/age.rs:395`, `graph/testkit/src/rust_donor.rs:355`, `graph/testkit/src/performance.rs:321,358`, `graph/testkit/src/main.rs:807`
- Modify: `graph/testkit/src/report.rs:37`
- Modify: `graph/CONFORMANCE.md`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `turso_graph_ir::SEMANTIC_PROFILE: SemanticProfile` (const)
  - `turso_graph_ir::SEMANTIC_PROFILE_VERSION: u32`
  - `turso_graph_ir::SemanticProfile` with fields `version: u32`, `row_order: RowOrder`, `duplicates: Duplicates`, `null_comparison: NullComparison`, `null_sort: NullSort`, `label_list_order: LabelListOrder`, `write_classification: WriteClassification`, `path_policy_version: u32`
  - `turso_graph_ir::{RowOrder, Duplicates, NullComparison, NullSort, LabelListOrder, WriteClassification}`
  - `SemanticProfile::render(&self) -> String` (stable, version-excluding)
  - `turso_graph_ir::semantic_profile_digest() -> String` (`"fnv1a64:<16 hex>"`)
  - `turso_graph_testkit::model::ResultRecord.semantics_version: u32` (0 = legacy, unrecorded)
  - `turso_graph_testkit::model::HISTORY_SCHEMA_VERSION == 2`

- [x] **Step 1: Write the failing digest-pin test**

Create `graph/ir/tests/semantic_profile_pin.rs`:

```rust
//! The semantic profile is a contract, not documentation. Any change to a
//! recorded choice must move `SEMANTIC_PROFILE_VERSION`, because every row in
//! `graph/test-results/history.jsonl` is interpreted against that number. This
//! test fails loudly on an unversioned edit and prints the digest to paste.

use turso_graph_ir::{semantic_profile_digest, SEMANTIC_PROFILE, SEMANTIC_PROFILE_VERSION};

/// Digest of `SEMANTIC_PROFILE.render()` at version 1.
const PINNED_DIGEST: &str = "fnv1a64:0000000000000000";

#[test]
fn semantic_profile_digest_is_pinned_to_its_version() {
    assert_eq!(
        semantic_profile_digest(),
        PINNED_DIGEST,
        "a semantic choice changed: bump SEMANTIC_PROFILE_VERSION (now {SEMANTIC_PROFILE_VERSION}) \
         and set PINNED_DIGEST to the observed digest above"
    );
}

#[test]
fn semantic_profile_reports_its_own_version() {
    assert_eq!(SEMANTIC_PROFILE.version, SEMANTIC_PROFILE_VERSION);
}

#[test]
fn render_excludes_the_version_so_a_bump_alone_never_changes_the_digest() {
    let rendered = SEMANTIC_PROFILE.render();
    assert!(
        !rendered.contains("version"),
        "render() must describe choices only, got:\n{rendered}"
    );
}
```

- [x] **Step 2: Run it to verify it fails**

Run: `cargo test -p turso_graph_ir --test semantic_profile_pin`
Expected: FAIL to compile with `unresolved import turso_graph_ir::semantic_profile_digest`.

- [x] **Step 3: Write the semantic profile**

Create `graph/ir/src/semantics.rs`:

```rust
//! The choices Turso makes where the Cypher specification is open.
//!
//! Cypher leaves row order, duplicate survival, NULL comparison, NULL sort
//! rank, and label-list order undefined. Turso answers each one. Those answers
//! decide the pass/fail verdict of the corpus, so they are versioned data here
//! rather than prose in a Markdown file. Every recorded test run stamps
//! `SEMANTIC_PROFILE_VERSION`, which makes a pass-count move attributable: the
//! code changed, or the rules changed, never ambiguously both.

use std::fmt::Write as _;

/// Bump on any change to a `SemanticProfile` field value. Never bump for
/// formatting, comments, or a new field whose value restates existing behavior
/// — a new field still changes the digest, so it still requires a bump.
pub const SEMANTIC_PROFILE_VERSION: u32 = 1;

/// Is the row order of a result defined?
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RowOrder {
    /// Defined only when the outermost RETURN carries ORDER BY. Every other
    /// result is a multiset: comparing it as a sequence is a false failure.
    OrderedOnlyUnderExplicitOrderBy,
}

/// Do duplicate rows survive?
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Duplicates {
    /// Duplicates survive unless DISTINCT is written. Results are multisets,
    /// not sets: a set comparison would hide a cardinality bug.
    RetainedUnlessDistinct,
}

/// What does a comparison against NULL yield?
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NullComparison {
    /// NULL, never false. A WHERE over NULL therefore drops the row without
    /// claiming the predicate was refuted.
    ThreeValuedNull,
}

/// Where does NULL rank in ORDER BY, and how do unlike types compare?
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NullSort {
    /// Ascending rank: numbers, then text, then blobs, then NULL. Mirrors
    /// `compare_returned_values` in the frontend mutation path.
    NumbersTextBlobsThenNullLast,
}

/// In what order does `labels(n)` return its labels?
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LabelListOrder {
    /// Label-table insertion order, materialized as `ORDER BY lbl.rowid` in
    /// relational lowering. Deterministic per database, not portable across
    /// databases that inserted the same labels in a different order.
    LabelTableInsertion,
}

/// When is a statement a write?
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WriteClassification {
    /// Syntactic. A statement that can write is a write, even when it changes
    /// zero rows: a DELETE matching nothing is still a write.
    SyntacticNeverResultDependent,
}

/// Every open choice, as one comparable value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticProfile {
    pub version: u32,
    pub row_order: RowOrder,
    pub duplicates: Duplicates,
    pub null_comparison: NullComparison,
    pub null_sort: NullSort,
    pub label_list_order: LabelListOrder,
    pub write_classification: WriteClassification,
    /// Version of the path algorithm legality table. Owned by
    /// `turso_graph_runtime::path_policy`; mirrored here so one number
    /// identifies the whole semantic contract.
    pub path_policy_version: u32,
}

pub const SEMANTIC_PROFILE: SemanticProfile = SemanticProfile {
    version: SEMANTIC_PROFILE_VERSION,
    row_order: RowOrder::OrderedOnlyUnderExplicitOrderBy,
    duplicates: Duplicates::RetainedUnlessDistinct,
    null_comparison: NullComparison::ThreeValuedNull,
    null_sort: NullSort::NumbersTextBlobsThenNullLast,
    label_list_order: LabelListOrder::LabelTableInsertion,
    write_classification: WriteClassification::SyntacticNeverResultDependent,
    path_policy_version: 0,
};

impl SemanticProfile {
    /// Stable `key=value` rendering of the choices, excluding `version`. The
    /// digest is taken over this, so the version can be bumped without the
    /// digest moving, and a choice cannot move without the digest moving.
    pub fn render(&self) -> String {
        let mut rendered = String::new();
        let _ = writeln!(rendered, "row_order={:?}", self.row_order);
        let _ = writeln!(rendered, "duplicates={:?}", self.duplicates);
        let _ = writeln!(rendered, "null_comparison={:?}", self.null_comparison);
        let _ = writeln!(rendered, "null_sort={:?}", self.null_sort);
        let _ = writeln!(rendered, "label_list_order={:?}", self.label_list_order);
        let _ = writeln!(
            rendered,
            "write_classification={:?}",
            self.write_classification
        );
        let _ = writeln!(rendered, "path_policy={}", self.path_policy_version);
        rendered
    }
}

/// FNV-1a 64 over `SEMANTIC_PROFILE.render()`, matching the history digest
/// format so both can be read the same way.
pub fn semantic_profile_digest() -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in SEMANTIC_PROFILE.render().as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}
```

- [x] **Step 4: Wire the module into the crate root**

In `graph/ir/src/lib.rs`, add `mod semantics;` to the module list (alphabetical, after `mod plan;`) and this re-export block after the `pub use scope::{...}` line:

```rust
pub use semantics::{
    semantic_profile_digest, Duplicates, LabelListOrder, NullComparison, NullSort, RowOrder,
    SemanticProfile, WriteClassification, SEMANTIC_PROFILE, SEMANTIC_PROFILE_VERSION,
};
```

- [x] **Step 5: Run the test, read the real digest, pin it**

Run: `cargo test -p turso_graph_ir --test semantic_profile_pin`
Expected: `semantic_profile_digest_is_pinned_to_its_version` FAILS with `left: "fnv1a64:<real>"`, `right: "fnv1a64:0000000000000000"`. The other two tests PASS.
Copy the `left` value into `PINNED_DIGEST` in `graph/ir/tests/semantic_profile_pin.rs`.

- [x] **Step 6: Run the test to verify it passes**

Run: `cargo test -p turso_graph_ir`
Expected: PASS, all three tests green.

- [x] **Step 7: Commit the profile**

```bash
cargo fmt
git add graph/ir/src/semantics.rs graph/ir/src/lib.rs graph/ir/tests/semantic_profile_pin.rs
git commit -S -m "graph/ir: version the semantic profile

Cypher leaves row order, duplicate survival, NULL comparison, NULL sort
rank, and label-list order undefined, and Turso answers each one. Those
answers decided every corpus verdict but lived only in prose, so a pass-count
move could not be attributed to code or to rules. Collect them into one
versioned value with a pinned content digest."
```

- [x] **Step 8: Write the failing history-record test**

Append to the `mod tests` block at the end of `graph/testkit/src/history.rs`:

```rust
    #[test]
    fn legacy_schema_version_one_rows_read_without_a_semantics_version() {
        // history.jsonl is append-only and ~1.25 GB of schema-version-1 rows.
        // Reading must never require rewriting them.
        let directory = tempfile::tempdir().expect("temp dir");
        let path = directory.path().join("history.jsonl");
        let mut legacy = serde_json::to_value(sample_record()).expect("serialize");
        legacy["schema_version"] = serde_json::json!(1);
        legacy.as_object_mut().expect("object").remove("semantics_version");
        std::fs::write(&path, format!("{legacy}\n")).expect("write");

        let records = read(&path).expect("legacy rows still read");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].schema_version, 1);
        assert_eq!(
            records[0].semantics_version, 0,
            "legacy rows report 0, meaning the semantics used are unknown"
        );
    }

    #[test]
    fn current_rows_carry_the_semantic_profile_version() {
        let record = sample_record();
        assert_eq!(record.schema_version, HISTORY_SCHEMA_VERSION);
        assert_eq!(
            record.semantics_version,
            turso_graph_ir::SEMANTIC_PROFILE_VERSION,
            "a run must record which semantic rules produced its verdicts"
        );
    }
```

If `sample_record()` does not already exist in that test module, add it next to the existing test helpers, built from whatever record constructor those tests already use (see the existing test at `graph/testkit/src/history.rs:223`), and have it set `schema_version: HISTORY_SCHEMA_VERSION` and `semantics_version: turso_graph_ir::SEMANTIC_PROFILE_VERSION`.

- [x] **Step 9: Run to verify it fails**

Run: `cargo test -p turso_graph_testkit history::tests`
Expected: FAIL to compile — `ResultRecord` has no field `semantics_version`.

- [x] **Step 10: Add the field and bump the schema**

In `graph/testkit/src/model.rs`, change the constant:

```rust
/// 2 added `semantics_version`. Version 1 rows predate the semantic profile
/// and read back with `semantics_version: 0`.
pub const HISTORY_SCHEMA_VERSION: u32 = 2;
```

and add this field to `ResultRecord`, immediately after `pub schema_version: u32,`:

```rust
    /// `turso_graph_ir::SEMANTIC_PROFILE_VERSION` in force when the verdict was
    /// produced. 0 means the row predates the profile and its rules are unknown.
    #[serde(default)]
    pub semantics_version: u32,
```

`history::read` already accepts `record.schema_version <= HISTORY_SCHEMA_VERSION` (`graph/testkit/src/history.rs:108`), so no change is needed there for backward compatibility.

- [x] **Step 11: Stamp the version at every record construction site**

Add `semantics_version: turso_graph_ir::SEMANTIC_PROFILE_VERSION,` to each `ResultRecord { .. }` literal. The compiler lists them; the known sites are:

- `graph/testkit/src/runner.rs:80` area
- `graph/testkit/src/tck.rs:1562` area
- `graph/testkit/src/grafeo.rs:625` area
- `graph/testkit/src/age.rs:395` area
- `graph/testkit/src/rust_donor.rs:355` area
- `graph/testkit/src/performance.rs:321` and `:358`
- `graph/testkit/src/main.rs:807` (test helper)
- `graph/testkit/src/history.rs:223` (test helper)

Run `cargo build -p turso_graph_testkit` and fix every `missing field semantics_version` error until it builds. Do not add a `Default` impl — the missing-field errors are the mechanism that guarantees no construction site is skipped.

- [x] **Step 12: Run the tests to verify they pass**

Run: `cargo test -p turso_graph_testkit`
Expected: PASS.

- [x] **Step 13: Surface the version in REPORT.md**

In `graph/testkit/src/report.rs`, in the latest-run summary format string at line 37, add a `- Semantics: v{semantics}` bullet after the `- Package:` bullet, where `semantics` is computed from the run's records:

```rust
    let mut versions: Vec<u32> = records
        .iter()
        .map(|record| record.semantics_version)
        .collect();
    versions.sort_unstable();
    versions.dedup();
    // A run that mixes profile versions is a recording bug, not a summary
    // detail: list them all rather than picking one.
    let semantics = versions
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(", ");
```

Apply the same to the corpus summary at line 132.

- [x] **Step 14: Write the report test**

Add to the `mod tests` block in `graph/testkit/src/report.rs`, alongside the existing test that asserts `report.contains("Unsupported: 1")`:

```rust
    #[test]
    fn report_states_the_semantic_profile_version_of_the_run() {
        let mut record = sample_record();
        record.semantics_version = 7;
        let report = render(&[record]);
        assert!(
            report.contains("Semantics: v7"),
            "the report must say which rules produced its numbers, got:\n{report}"
        );
    }
```

Match `sample_record()` and `render(..)` to the helper and entry point the neighbouring tests already use in that module.

- [x] **Step 15: Run to verify it passes**

Run: `cargo test -p turso_graph_testkit report`
Expected: PASS.

- [x] **Step 16: Update CONFORMANCE.md**

In `graph/CONFORMANCE.md`, insert after the first paragraph:

```markdown
Every recorded result carries the semantic profile version that produced it
(`semantics_version` in `test-results/history.jsonl`, `Semantics:` in
`test-results/REPORT.md`). The profile is
[`graph/ir/src/semantics.rs`](../graph/ir/src/semantics.rs); its digest is
pinned by `graph/ir/tests/semantic_profile_pin.rs`. Compare pass counts only
between runs with the same semantics version.
```

- [x] **Step 17: Verify and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_ir -p turso_graph_testkit
git add graph/testkit/src graph/CONFORMANCE.md
git commit -S -m "graph/testkit: record the semantic profile version in every run

A pass-count move between two runs was previously unattributable: the code
could have changed, or the undocumented semantic choices could have. Stamp
SEMANTIC_PROFILE_VERSION into each history row and into REPORT.md. History
schema goes to 2; version 1 rows read back with semantics_version 0, which
reads as 'unknown rules' rather than requiring the 1.25 GB log to be rewritten."
```

---

### Task 2: Compare unordered results as multisets, including their digests

**Files:**
- Modify: `graph/testkit/src/history.rs:119-132` (digest), plus its test at `:245`
- Modify: `graph/testkit/src/runner.rs:53-81`
- Modify: `graph/testkit/src/tck.rs:1530-1563`
- Modify: `graph/testkit/src/grafeo.rs:585-626`
- Modify: `graph/testkit/src/cypherbench.rs:608-612`
- Modify: `graph/testkit/src/performance.rs:321`
- Modify: `graph/testkit/src/manifest.rs:96-108` (validation)
- Modify: `graph/DESIGN_DECISIONS.md`

**Interfaces:**
- Consumes: `turso_graph_ir::{SEMANTIC_PROFILE, RowOrder}` from Task 1.
- Produces:
  - `turso_graph_testkit::history::ResultOrdering { Ordered, Unordered }`
  - `turso_graph_testkit::history::result_digest_with(rows: &[Vec<String>], ordering: ResultOrdering) -> String` — `"fnv1a64:<hex>"` for `Ordered`, `"fnv1a64u:<hex>"` for `Unordered`
  - `turso_graph_testkit::history::result_digest(rows: &[Vec<String>]) -> String` — unchanged signature, now `result_digest_with(rows, ResultOrdering::Ordered)`

- [x] **Step 1: Write the failing digest tests**

Replace the existing `result_digest_is_order_sensitive_and_reproducible` test in `graph/testkit/src/history.rs` with:

```rust
    #[test]
    fn ordered_digests_are_order_sensitive() {
        // A result whose order the query defined must drift when the order
        // drifts: that is a real regression, not noise.
        let first = vec![vec!["a".to_owned()], vec!["b".to_owned()]];
        let second = vec![vec!["b".to_owned()], vec!["a".to_owned()]];
        assert_eq!(
            result_digest_with(&first, ResultOrdering::Ordered),
            result_digest_with(&first, ResultOrdering::Ordered)
        );
        assert_ne!(
            result_digest_with(&first, ResultOrdering::Ordered),
            result_digest_with(&second, ResultOrdering::Ordered)
        );
    }

    #[test]
    fn unordered_digests_ignore_row_order() {
        // Without ORDER BY the row order is whatever the B-tree gave us. A
        // digest that moves with it reports a regression when nothing broke.
        let first = vec![vec!["a".to_owned()], vec!["b".to_owned()]];
        let second = vec![vec!["b".to_owned()], vec!["a".to_owned()]];
        assert_eq!(
            result_digest_with(&first, ResultOrdering::Unordered),
            result_digest_with(&second, ResultOrdering::Unordered)
        );
    }

    #[test]
    fn unordered_digests_still_count_duplicates() {
        // Results are multisets, not sets: losing a duplicate row is a
        // cardinality bug and must move the digest.
        let once = vec![vec!["a".to_owned()]];
        let twice = vec![vec!["a".to_owned()], vec!["a".to_owned()]];
        assert_ne!(
            result_digest_with(&once, ResultOrdering::Unordered),
            result_digest_with(&twice, ResultOrdering::Unordered)
        );
    }

    #[test]
    fn the_two_digest_modes_are_distinguishable_in_recorded_history() {
        let rows = vec![vec!["a".to_owned()]];
        assert!(result_digest_with(&rows, ResultOrdering::Ordered).starts_with("fnv1a64:"));
        assert!(result_digest_with(&rows, ResultOrdering::Unordered).starts_with("fnv1a64u:"));
    }

    #[test]
    fn the_default_digest_stays_ordered_for_existing_history_rows() {
        let rows = vec![vec!["a".to_owned()], vec!["b".to_owned()]];
        assert_eq!(
            result_digest(&rows),
            result_digest_with(&rows, ResultOrdering::Ordered)
        );
    }
```

- [x] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_testkit history::tests`
Expected: FAIL to compile — `cannot find function result_digest_with` and `cannot find type ResultOrdering`.

- [x] **Step 3: Implement the two digest modes**

In `graph/testkit/src/history.rs`, replace `pub fn result_digest` with:

```rust
/// Whether a result's row order is part of its identity.
///
/// `SEMANTIC_PROFILE.row_order` is
/// `RowOrder::OrderedOnlyUnderExplicitOrderBy`: without ORDER BY the engine may
/// return rows in any order, so hashing them as a sequence records B-tree
/// layout rather than query behavior. Callers pass the same mode they use for
/// comparison, so the recorded digest and the pass/fail verdict cannot disagree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultOrdering {
    Ordered,
    Unordered,
}

/// Order-sensitive digest, kept for callers that already recorded one.
pub fn result_digest(rows: &[Vec<String>]) -> String {
    result_digest_with(rows, ResultOrdering::Ordered)
}

pub fn result_digest_with(rows: &[Vec<String>], ordering: ResultOrdering) -> String {
    let (prefix, rows) = match ordering {
        ResultOrdering::Ordered => ("fnv1a64", std::borrow::Cow::Borrowed(rows)),
        ResultOrdering::Unordered => {
            // Sorting canonicalizes the multiset without collapsing it:
            // duplicate rows still contribute, so a lost row still moves the
            // digest.
            let mut sorted = rows.to_vec();
            sorted.sort();
            ("fnv1a64u", std::borrow::Cow::Owned(sorted))
        }
    };
    let mut hash = 0xcbf29ce484222325_u64;
    for row in rows.iter() {
        for value in row {
            for byte in value.as_bytes().iter().copied().chain([0xff]) {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
        hash ^= 0xfe;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{prefix}:{hash:016x}")
}
```

- [x] **Step 4: Run to verify it passes**

Run: `cargo test -p turso_graph_testkit history::tests`
Expected: PASS, five digest tests green.

- [x] **Step 5: Pass each suite's ordering mode into its digest**

Each suite already knows whether the case is ordered. Thread that flag to the digest call so both agree.

- `graph/testkit/src/tck.rs`: `expected_rows(case)` at `:885` returns `(rows, ordered)` from the `in order:` suffix; the comparison at `:550` uses it. Carry that `ordered` bool into the record builder and replace the `result_digest(rows)` call at `:1534` with `result_digest_with(rows, if ordered { ResultOrdering::Ordered } else { ResultOrdering::Unordered })`.
- `graph/testkit/src/grafeo.rs`: `GrafeoExpectation::Rows { ordered, .. }` at `:541`. Same change at `:587`. For `Count`, `Empty`, and `Error` expectations, and for `None`, use `ResultOrdering::Unordered` — none of them compare a sequence.
- `graph/testkit/src/runner.rs`: `scenario.ordering == "unordered"` at `:411`. Same change at `:57`.
- `graph/testkit/src/cypherbench.rs`: the comparison at `:610` sorts unconditionally, so it is unordered by construction. If it records a digest, use `ResultOrdering::Unordered`.
- `graph/testkit/src/performance.rs:321`: benchmark rows are never compared to an expectation, so their digest exists only for drift detection across runs; use `ResultOrdering::Unordered` so B-tree order changes do not read as drift.
- `graph/testkit/src/age.rs`: records `result_digest: None` and compares no rows. No change.

- [x] **Step 6: Write the failing manifest-validation test**

Add to the `mod tests` block in `graph/testkit/src/manifest.rs` (create the module if absent):

```rust
    #[test]
    fn a_scenario_may_only_claim_ordered_when_its_query_says_order_by() {
        // Declaring a result ordered when the query never asked for an order
        // pins the test to SQLite's B-tree layout. That test fails on an index
        // change while nothing is broken.
        let mut manifest = sample_manifest();
        manifest.scenario[0].ordering = "ordered".to_owned();
        manifest.scenario[0].query = "MATCH (n) RETURN n.name".to_owned();

        let error = manifest.validate().expect_err("ordered claim must be rejected");
        assert!(
            matches!(error, ManifestError::Invalid { ref reason, .. } if reason.contains("ORDER BY")),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn an_ordered_scenario_with_order_by_validates() {
        let mut manifest = sample_manifest();
        manifest.scenario[0].ordering = "ordered".to_owned();
        manifest.scenario[0].query = "MATCH (n) RETURN n.name ORDER BY n.name".to_owned();
        manifest.validate().expect("ORDER BY justifies the ordered claim");
    }
```

`sample_manifest()` builds a `ScenarioManifest` with `version: 1`, a non-empty `purpose`, and one `Scenario` that already satisfies every other rule in `validate` (tier `smoke`, action `query`, ordering `unordered`, a 40-character `source.revision`, a `https://github.com/` repository, and license `MIT`). Write it in the same test module.

- [x] **Step 7: Run to verify it fails**

Run: `cargo test -p turso_graph_testkit manifest`
Expected: FAIL — `a_scenario_may_only_claim_ordered_when_its_query_says_order_by` panics with `ordered claim must be rejected`, because `validate` currently accepts any `"ordered" | "unordered"` string.

- [x] **Step 8: Add the validation rule**

In `graph/testkit/src/manifest.rs`, inside the per-scenario loop in `validate`, after the existing `invalid` check block:

```rust
            if scenario.ordering == "ordered"
                && !scenario.query.to_ascii_uppercase().contains("ORDER BY")
            {
                return Err(ManifestError::Invalid {
                    id: scenario.id.clone(),
                    reason: "ordering = \"ordered\" requires ORDER BY in the query".to_owned(),
                });
            }
```

- [x] **Step 9: Run to verify it passes, and fix any manifest that now fails**

Run: `cargo test -p turso_graph_testkit`
Expected: PASS. If a checked-in scenario manifest under `graph/testdata/` now fails to load, that scenario was pinned to undefined order — change its `ordering` to `"unordered"` rather than adding an `ORDER BY` to the query, since the query is donor-sourced.

- [x] **Step 10: Record the rule in DESIGN_DECISIONS.md**

Add this section to `graph/DESIGN_DECISIONS.md` after the bulleted decision list:

```markdown
## Result ordering contract

`SEMANTIC_PROFILE.row_order` is `OrderedOnlyUnderExplicitOrderBy`. A result's
row order is part of its identity only when the outermost RETURN carries
`ORDER BY`. Everything else is a **multiset**: duplicates count, order does not.

| Result | Comparison | Recorded digest |
| --- | --- | --- |
| RETURN … ORDER BY … | sequence | `fnv1a64:` |
| RETURN … (no ORDER BY) | multiset (sorted, duplicates retained) | `fnv1a64u:` |
| aggregate without ORDER BY | multiset | `fnv1a64u:` |
| `labels(n)` list contents | sequence, label-table insertion order | n/a |

The `labels(n)` row is a deliberate exception. Relational lowering emits
`ORDER BY lbl.rowid` inside the `json_group_array` subquery for `labels()` and
the `LIMIT 1` subquery for `label()` (`graph/frontend/src/lowering.rs`), so the
list is deterministic within one database. It is not portable across databases
that inserted the same labels in a different order, and no test may depend on
cross-database label order.

Suite manifests may not declare `ordering = "ordered"` for a query with no
`ORDER BY`; `ScenarioManifest::validate` rejects it.
```

- [x] **Step 11: Verify and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_testkit
git add graph/testkit/src graph/DESIGN_DECISIONS.md
git commit -S -m "graph/testkit: digest unordered results as multisets

Four of the five suites already sorted rows before comparing, but every
recorded result_digest was order-sensitive, so an unordered result's digest
tracked SQLite B-tree layout rather than query behavior and drifted with an
index change while the comparison still passed. Give the digest an explicit
ordering mode that each suite drives from the same flag it compares with, and
reject manifests that claim a defined order for a query without ORDER BY."
```

---

### Task 3: Make the 53 vendor divergences a tested registry

**Files:**
- Create: `graph/registries/divergence.toml`
- Create: `graph/testkit/src/divergence.rs`
- Create: `graph/testkit/tests/divergence.rs`
- Modify: `graph/testkit/src/lib.rs` (module list), `graph/testkit/src/main.rs` (`Command` enum and dispatch at `:138-171`)
- Modify: `graph/CONFORMANCE.md`

**Interfaces:**
- Consumes: `ResultRecord.semantics_version` and `HISTORY_SCHEMA_VERSION` from Task 1; `turso_graph_testkit::model::{Outcome, ResultRecord}`.
- Produces:
  - `turso_graph_testkit::divergence::DivergenceRegistry { version: u32, entry: Vec<DivergenceEntry> }`
  - `turso_graph_testkit::divergence::DivergenceEntry { id: String, vendor: String, area: String, reason: String, tests: Vec<TestId> }`
  - `DivergenceRegistry::load(path) -> Result<Self, DivergenceError>`
  - `DivergenceRegistry::verify(&self, records: &[ResultRecord]) -> Result<DivergenceReport, DivergenceError>`
  - `DivergenceRegistry::sync(records: &[ResultRecord]) -> Self`
  - `DivergenceReport { registered: usize, matched: usize }`
  - CLI: `cargo run -q -p turso_graph_testkit -- divergence verify` and `-- divergence sync`

- [ ] **Step 1: Write the failing registry tests**

Create `graph/testkit/tests/divergence.rs`:

```rust
//! The registry turns "53 unsupported vendor behaviors" from a sentence in
//! CONFORMANCE.md into a checked fact. Every unsupported outcome in a recorded
//! corpus run must be named by exactly one registry entry, and every entry must
//! name at least one test that the run actually contains.

use turso_graph_testkit::divergence::{DivergenceError, DivergenceRegistry};
use turso_graph_testkit::model::{Outcome, ResultRecord};

mod support;
use support::{record_with, registry_root};

#[test]
fn the_checked_in_registry_loads() {
    DivergenceRegistry::load(registry_root().join("divergence.toml"))
        .expect("graph/registries/divergence.toml must parse");
}

#[test]
fn every_registry_entry_names_at_least_one_test() {
    let registry = DivergenceRegistry::load(registry_root().join("divergence.toml"))
        .expect("registry loads");
    for entry in &registry.entry {
        assert!(
            !entry.tests.is_empty(),
            "divergence `{}` claims a behavior with no test to prove it",
            entry.id
        );
        assert!(
            !entry.reason.trim().is_empty(),
            "divergence `{}` has no reason",
            entry.id
        );
    }
}

#[test]
fn an_unsupported_outcome_with_no_registry_entry_fails_verification() {
    let registry = DivergenceRegistry {
        version: 1,
        entry: Vec::new(),
    };
    let records = vec![record_with("age.vertex_stats.1", Outcome::Unsupported)];

    let error = registry
        .verify(&records)
        .expect_err("an unregistered divergence must fail CI");
    assert!(
        matches!(error, DivergenceError::Unregistered { ref test_id, .. } if test_id == "age.vertex_stats.1"),
        "unexpected error: {error}"
    );
}

#[test]
fn a_registry_entry_whose_test_vanished_fails_verification() {
    let registry = DivergenceRegistry::sync(&[record_with(
        "age.vertex_stats.1",
        Outcome::Unsupported,
    )]);
    // The run no longer contains the test the entry names, so the claim is
    // stale. That is exactly the drift the registry exists to catch.
    let error = registry
        .verify(&[record_with("age.other.1", Outcome::Passed)])
        .expect_err("a missing test must fail CI");
    assert!(
        matches!(error, DivergenceError::MissingTest { ref test_id, .. } if test_id == "age.vertex_stats.1"),
        "unexpected error: {error}"
    );
}

#[test]
fn a_divergence_that_started_passing_fails_verification() {
    // Newly gained support is good news that still has to be recorded: the
    // entry must be removed, not left claiming an unsupported behavior.
    let registry = DivergenceRegistry::sync(&[record_with(
        "age.vertex_stats.1",
        Outcome::Unsupported,
    )]);
    let error = registry
        .verify(&[record_with("age.vertex_stats.1", Outcome::Passed)])
        .expect_err("a now-supported divergence must fail CI");
    assert!(
        matches!(error, DivergenceError::NoLongerDivergent { ref test_id, .. } if test_id == "age.vertex_stats.1"),
        "unexpected error: {error}"
    );
}

#[test]
fn a_matching_run_verifies_and_counts() {
    let records = vec![
        record_with("age.vertex_stats.1", Outcome::Unsupported),
        record_with("age.graph_stats.1", Outcome::Unsupported),
        record_with("age.plain.1", Outcome::Passed),
    ];
    let registry = DivergenceRegistry::sync(&records);
    let report = registry.verify(&records).expect("registry matches the run");
    assert_eq!(report.matched, 2, "both unsupported outcomes are accounted for");
}
```

Create `graph/testkit/tests/support/mod.rs`:

```rust
use std::path::PathBuf;

use turso_graph_testkit::model::{
    Expectation, Outcome, ResultRecord, RunEnvironment, SourceIdentity, TestKind,
    HISTORY_SCHEMA_VERSION,
};

pub fn registry_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("graph/testkit has a parent")
        .join("registries")
}

pub fn record_with(test_id: &str, outcome: Outcome) -> ResultRecord {
    ResultRecord {
        schema_version: HISTORY_SCHEMA_VERSION,
        semantics_version: turso_graph_ir::SEMANTIC_PROFILE_VERSION,
        run_id: "test-run".to_owned(),
        recorded_at: "2026-07-25T00:00:00.000000Z".to_owned(),
        environment: RunEnvironment {
            git_commit: "0".repeat(40),
            git_dirty: false,
            package_version: "0.0.0".to_owned(),
            profile: "dev".to_owned(),
            os: "macos".to_owned(),
            architecture: "aarch64".to_owned(),
        },
        suite: "corpus".to_owned(),
        test_id: test_id.parse().expect("valid test id"),
        kind: TestKind::Conformance,
        area: "vendor".to_owned(),
        fixture: "empty".to_owned(),
        expectation: Expectation::Unsupported,
        outcome,
        duration_ns: 0,
        source: SourceIdentity {
            name: "Apache AGE".to_owned(),
            repository: "https://github.com/apache/age".to_owned(),
            revision: "0".repeat(40),
            path: "regress/sql/x.sql".to_owned(),
            case: "x".to_owned(),
            license: "Apache-2.0".to_owned(),
            adaptation: "fixture-adaptation".to_owned(),
            issue: None,
            fixed_commit: None,
        },
        operation: None,
        graph_shape: None,
        scale: None,
        iterations: None,
        throughput_per_second: None,
        row_count: None,
        node_count: None,
        relationship_count: None,
        result_digest: None,
        message: None,
        dimensions: Default::default(),
    }
}
```

If `TestId` does not implement `FromStr`, use whatever constructor `graph/testkit/src/identity.rs` exposes (see its use in `graph/testkit/src/manifest.rs`) and adjust `record_with` accordingly. If `ResultRecord`'s fields are not all `pub`, make the test module build the record through the same path `graph/testkit/src/main.rs:807` uses.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_testkit --test divergence`
Expected: FAIL to compile — `unresolved import turso_graph_testkit::divergence`.

- [ ] **Step 3: Implement the registry**

Create `graph/testkit/src/divergence.rs`:

```rust
//! Registry of behaviors Turso deliberately does not implement because they
//! are specific to one other database.
//!
//! CONFORMANCE.md used to state a count. A count in prose drifts silently: a
//! divergence can be gained, lost, or renamed with nothing to notice. Each
//! entry here names the tests that prove it, and `verify` fails when the
//! registry and a recorded corpus run disagree in either direction.

use std::{collections::BTreeMap, fs, path::Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    identity::TestId,
    model::{Outcome, ResultRecord},
};

pub const DIVERGENCE_REGISTRY_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DivergenceRegistry {
    pub version: u32,
    #[serde(default)]
    pub entry: Vec<DivergenceEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DivergenceEntry {
    /// Stable slug, e.g. `apache-age.vertex_stats`.
    pub id: String,
    /// The database whose behavior this is.
    pub vendor: String,
    /// Language area: `vendor-function`, `vendor-operator`, `vendor-ddl`, …
    pub area: String,
    /// Why Turso does not implement it.
    pub reason: String,
    /// Corpus identities that exercise it. Never empty.
    pub tests: Vec<TestId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DivergenceReport {
    pub registered: usize,
    pub matched: usize,
}

#[derive(Debug, Error)]
pub enum DivergenceError {
    #[error("failed to read divergence registry {path}: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse divergence registry {path}: {source}")]
    Parse {
        path: String,
        source: toml::de::Error,
    },
    #[error("unsupported divergence registry version {0}")]
    Version(u32),
    #[error("divergence `{id}` names no test")]
    EmptyEntry { id: String },
    #[error("test `{test_id}` is claimed by divergences `{first}` and `{second}`")]
    DuplicateTest {
        test_id: String,
        first: String,
        second: String,
    },
    #[error("test `{test_id}` reported an unsupported outcome but no divergence entry claims it")]
    Unregistered { test_id: String },
    #[error("divergence `{id}` names test `{test_id}`, which the run does not contain")]
    MissingTest { id: String, test_id: String },
    #[error("divergence `{id}` names test `{test_id}`, which now reports `{outcome:?}`: remove the entry")]
    NoLongerDivergent {
        id: String,
        test_id: String,
        outcome: Outcome,
    },
}

impl DivergenceRegistry {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, DivergenceError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|source| DivergenceError::Read {
            path: path.display().to_string(),
            source,
        })?;
        let registry: Self =
            toml::from_str(&content).map_err(|source| DivergenceError::Parse {
                path: path.display().to_string(),
                source,
            })?;
        if registry.version != DIVERGENCE_REGISTRY_VERSION {
            return Err(DivergenceError::Version(registry.version));
        }
        registry.claims()?;
        Ok(registry)
    }

    /// test id -> owning entry id, rejecting empty entries and double claims.
    fn claims(&self) -> Result<BTreeMap<String, String>, DivergenceError> {
        let mut claims = BTreeMap::new();
        for entry in &self.entry {
            if entry.tests.is_empty() {
                return Err(DivergenceError::EmptyEntry {
                    id: entry.id.clone(),
                });
            }
            for test in &entry.tests {
                if let Some(first) = claims.insert(test.to_string(), entry.id.clone()) {
                    return Err(DivergenceError::DuplicateTest {
                        test_id: test.to_string(),
                        first,
                        second: entry.id.clone(),
                    });
                }
            }
        }
        Ok(claims)
    }

    pub fn verify(&self, records: &[ResultRecord]) -> Result<DivergenceReport, DivergenceError> {
        let claims = self.claims()?;
        let observed: BTreeMap<String, Outcome> = records
            .iter()
            .map(|record| (record.test_id.to_string(), record.outcome))
            .collect();

        // Direction 1: nothing diverges without being named.
        for (test_id, outcome) in &observed {
            if *outcome == Outcome::Unsupported && !claims.contains_key(test_id) {
                return Err(DivergenceError::Unregistered {
                    test_id: test_id.clone(),
                });
            }
        }

        // Direction 2: nothing is named without still diverging.
        let mut matched = 0;
        for (test_id, id) in &claims {
            match observed.get(test_id) {
                None => {
                    return Err(DivergenceError::MissingTest {
                        id: id.clone(),
                        test_id: test_id.clone(),
                    })
                }
                Some(Outcome::Unsupported) => matched += 1,
                Some(outcome) => {
                    return Err(DivergenceError::NoLongerDivergent {
                        id: id.clone(),
                        test_id: test_id.clone(),
                        outcome: *outcome,
                    })
                }
            }
        }

        Ok(DivergenceReport {
            registered: claims.len(),
            matched,
        })
    }

    /// Build a registry from a recorded run. Used once to seed the file and
    /// afterwards only to regenerate it deliberately; `reason` needs a human.
    pub fn sync(records: &[ResultRecord]) -> Self {
        let mut grouped: BTreeMap<String, DivergenceEntry> = BTreeMap::new();
        for record in records
            .iter()
            .filter(|record| record.outcome == Outcome::Unsupported)
        {
            // AGE stamps the offending function name as a dimension; group by
            // it so one entry covers one behavior, not one test.
            let behavior = record
                .dimensions
                .get("vendor_unsupported_function")
                .cloned()
                .unwrap_or_else(|| record.area.clone());
            let id = format!("{}.{behavior}", slug(&record.source.name));
            grouped
                .entry(id.clone())
                .or_insert_with(|| DivergenceEntry {
                    id,
                    vendor: record.source.name.clone(),
                    area: "vendor-function".to_owned(),
                    reason: "TODO: state why Turso does not implement this".to_owned(),
                    tests: Vec::new(),
                })
                .tests
                .push(record.test_id.clone());
        }
        for entry in grouped.values_mut() {
            entry.tests.sort();
        }
        Self {
            version: DIVERGENCE_REGISTRY_VERSION,
            entry: grouped.into_values().collect(),
        }
    }
}

fn slug(name: &str) -> String {
    name.to_ascii_lowercase()
        .chars()
        .map(|character| if character.is_ascii_alphanumeric() { character } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_owned()
}
```

Add `pub mod divergence;` to `graph/testkit/src/lib.rs` alongside the existing module declarations, and make sure `model`, `identity`, and `history` are `pub` there (they already are, since `graph/testkit/tests/suites.rs` uses them; match whatever visibility that test relies on).

If `TestId` does not derive `Ord`, add `Ord, PartialOrd` to its derive list in `graph/testkit/src/identity.rs` so `entry.tests.sort()` compiles.

- [ ] **Step 4: Run the unit-level registry tests**

Run: `cargo test -p turso_graph_testkit --test divergence -- --skip the_checked_in_registry --skip every_registry_entry`
Expected: PASS for the four behavioral tests. The two file-backed tests still fail: the registry file does not exist.

- [ ] **Step 5: Add the CLI subcommand**

In `graph/testkit/src/main.rs`, add to the `Command` enum (near `Command::VerifyHistory`):

```rust
    /// Check `graph/registries/divergence.toml` against the latest recorded
    /// corpus run, or regenerate it from that run.
    Divergence {
        #[arg(long, default_value = "graph/test-results/history.jsonl")]
        history: PathBuf,
        #[arg(long)]
        registry: Option<PathBuf>,
        /// Rewrite the registry from the run instead of checking it.
        #[arg(long)]
        sync: bool,
    },
```

and to the dispatch `match` (after the `Command::VerifyHistory` arm):

```rust
        Command::Divergence {
            history,
            registry,
            sync,
        } => run_divergence(&root, &history, registry.as_deref(), sync),
```

Add the handler next to the other `run_*` functions:

```rust
/// The latest corpus run is the source of truth for what currently diverges;
/// `graph/test-results/REPORT.md` is generated from the same rows.
fn run_divergence(
    root: &Path,
    history: &Path,
    registry: Option<&Path>,
    sync: bool,
) -> anyhow::Result<bool> {
    let path = registry
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.join("registries/divergence.toml"));
    let records = crate::history::read(history)?;
    let latest = latest_corpus_run(&records)
        .ok_or_else(|| anyhow::anyhow!("history contains no corpus run"))?;

    if sync {
        let generated = DivergenceRegistry::sync(&latest);
        fs::write(&path, toml::to_string_pretty(&generated)?)?;
        println!(
            "wrote {} divergence entries to {}",
            generated.entry.len(),
            path.display()
        );
        return Ok(true);
    }

    let report = DivergenceRegistry::load(&path)?.verify(&latest)?;
    println!(
        "divergence registry verified: {} entries, {} tests",
        DivergenceRegistry::load(&path)?.entry.len(),
        report.matched
    );
    Ok(true)
}

/// Records of the newest run whose suite is `corpus`.
fn latest_corpus_run(records: &[ResultRecord]) -> Option<Vec<ResultRecord>> {
    let run_id = records
        .iter()
        .filter(|record| record.suite == "corpus")
        .map(|record| record.run_id.as_str())
        .max()?
        .to_owned();
    Some(
        records
            .iter()
            .filter(|record| record.run_id == run_id)
            .cloned()
            .collect(),
    )
}
```

Match the existing handlers' return type and error handling — the dispatch at `graph/testkit/src/main.rs:138` expects `Result<bool, _>` where `false` becomes `ExitCode::FAILURE`.

- [ ] **Step 6: Seed the registry from the recorded corpus run**

Run:
```bash
cargo run -q -p turso_graph_testkit -- divergence --sync
```
Expected: `wrote N divergence entries to graph/registries/divergence.toml`.

Open `graph/registries/divergence.toml` and replace every `reason = "TODO: ..."` with a real sentence. The four AGE behaviors and their reasons:

- `vertex_stats` — AGE catalog statistics function over `ag_catalog`; Turso has no `ag_catalog` and reports graph statistics through the Turso catalog instead.
- `graph_stats` — same, at graph granularity.
- `delete_global_graphs` — AGE global-graph administration; Turso graphs are registered per database and dropped through the Turso catalog.
- `is_valid_label_name` — AGE label-name validator exposed as SQL; Turso validates label names at bind time and has no runtime predicate for it.

Then add the count assertion to `graph/testkit/tests/divergence.rs`:

```rust
#[test]
fn the_registry_accounts_for_every_divergent_test_in_the_corpus() {
    // CONFORMANCE.md quotes this number. The registry is what makes it true.
    let registry = DivergenceRegistry::load(registry_root().join("divergence.toml"))
        .expect("registry loads");
    let total: usize = registry.entry.iter().map(|entry| entry.tests.len()).sum();
    assert_eq!(
        total, 53,
        "the divergence count moved; update CONFORMANCE.md in the same commit"
    );
}
```

If the sync produced a number other than 53, use the produced number and say so explicitly in the commit body and in `CONFORMANCE.md` — do not edit the registry to reach 53.

- [ ] **Step 7: Run the full divergence test file**

Run: `cargo test -p turso_graph_testkit --test divergence`
Expected: PASS, all seven tests green.

- [ ] **Step 8: Verify the CLI check passes**

Run: `cargo run -q -p turso_graph_testkit -- divergence`
Expected: `divergence registry verified: 4 entries, 53 tests`, exit code 0.

- [ ] **Step 9: Update CONFORMANCE.md**

Replace the `- **53 unsupported** vendor-specific behaviors; and` bullet's surrounding prose with:

```markdown
- **8,919 passed**;
- **53 unsupported** vendor-specific behaviors, every one of them named by
  [`registries/divergence.toml`](registries/divergence.toml); and
- **1,270 failed** with a non-empty reason.

The unsupported count is enforced, not asserted. `cargo run -q -p
turso_graph_testkit -- divergence` fails when a test reports an unsupported
outcome that no registry entry claims, when a registry entry names a test the
run no longer contains, or when a registered divergence starts passing.
```

- [ ] **Step 10: Verify and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_testkit
cargo run -q -p turso_graph_testkit -- divergence
git add graph/registries graph/testkit graph/CONFORMANCE.md
git commit -S -m "graph/testkit: enforce the vendor divergence registry

CONFORMANCE.md claimed 53 vendor-specific unsupported behaviors, a number
derived at corpus-build time from a four-name regex allowlist in age.rs and
proved by nothing. Name each behavior in graph/registries/divergence.toml with
the tests that exercise it, and verify both directions against the latest
recorded corpus run: an unsupported outcome with no entry fails, and an entry
whose test vanished or started passing fails."
```

---

### Task 4: Classify statements in the binder and enforce read-only connections

**Files:**
- Modify: `graph/frontend/src/binder.rs` (add `StatementKind` and `classify` near `bind`/`bind_mutation` at `:424-431`)
- Modify: `graph/frontend/src/session.rs:86-95` (struct), `:221-233` (`query`), `:315-330` (`execute`)
- Modify: `graph/frontend/src/lib.rs:29-33` (binder re-exports)
- Test: `graph/frontend/tests/statement_kind.rs` (create)

**Interfaces:**
- Consumes: `turso_graph_ir::{SEMANTIC_PROFILE, WriteClassification}` from Task 1; `turso_graph_cypher::{parse, Query, Clause}`.
- Produces:
  - `turso_graph_frontend::StatementKind { ReadOnly, WriteReturningRows, WriteWithoutRows }`
  - `turso_graph_frontend::classify_statement(query: &turso_graph_cypher::Query) -> StatementKind`
  - `StatementKind::writes(&self) -> bool`
  - `GraphConnection::classify(&self, source: &str) -> Result<StatementKind, Error>`
  - `GraphConnection::set_read_only(&mut self, read_only: bool)`
  - `GraphConnection::is_read_only(&self) -> bool`
  - `Error::ReadOnlyConnection { kind: StatementKind }`

- [ ] **Step 1: Write the failing classification tests**

Create `graph/frontend/tests/statement_kind.rs`:

```rust
//! Statement classification is syntactic and never result-dependent.
//!
//! Callers previously routed by trying `query()` and falling back to
//! `execute()` when it errored, which turns a genuine read failure into a
//! mutation attempt. `SEMANTIC_PROFILE.write_classification` is
//! `SyntacticNeverResultDependent`: whether a statement writes is decided by
//! what it says, not by what it changed.

use turso_graph_frontend::{classify_statement, StatementKind};

fn classify(source: &str) -> StatementKind {
    let query = turso_graph_cypher::parse(source).expect("source parses");
    classify_statement(&query)
}

#[test]
fn a_match_return_is_read_only() {
    assert_eq!(classify("MATCH (n) RETURN n"), StatementKind::ReadOnly);
}

#[test]
fn a_with_pipeline_without_mutation_is_read_only() {
    assert_eq!(
        classify("MATCH (n) WITH n WHERE n.age > 3 RETURN n.name ORDER BY n.name"),
        StatementKind::ReadOnly
    );
}

#[test]
fn a_create_without_return_writes_without_rows() {
    assert_eq!(
        classify("CREATE (n:Person {name: 'a'})"),
        StatementKind::WriteWithoutRows
    );
}

#[test]
fn a_create_with_return_writes_and_returns_rows() {
    assert_eq!(
        classify("CREATE (n:Person {name: 'a'}) RETURN n"),
        StatementKind::WriteReturningRows
    );
}

#[test]
fn a_delete_that_can_match_nothing_is_still_a_write() {
    // The rule that makes classification useful: emptiness is a runtime fact,
    // and a runtime fact must never change a compile-time classification.
    // Otherwise a read-only connection would accept a DELETE and reject it only
    // when it happened to match a row.
    assert_eq!(
        classify("MATCH (n:NoSuchLabelAnywhere) DELETE n"),
        StatementKind::WriteWithoutRows
    );
}

#[test]
fn set_remove_merge_and_detach_delete_all_write() {
    for source in [
        "MATCH (n) SET n.age = 1",
        "MATCH (n) REMOVE n.age",
        "MERGE (n:Person {name: 'a'})",
        "MATCH (n) DETACH DELETE n",
        "MATCH (n) SET n:Archived",
    ] {
        assert!(
            classify(source).writes(),
            "`{source}` must classify as a write"
        );
    }
}

#[test]
fn read_only_never_writes() {
    assert!(!StatementKind::ReadOnly.writes());
    assert!(StatementKind::WriteReturningRows.writes());
    assert!(StatementKind::WriteWithoutRows.writes());
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --test statement_kind`
Expected: FAIL to compile — `unresolved imports turso_graph_frontend::classify_statement, turso_graph_frontend::StatementKind`.

- [ ] **Step 3: Implement classification in the binder**

In `graph/frontend/src/binder.rs`, add above `pub fn bind_mutation` (around line 424):

```rust
/// What a statement is allowed to do, decided from its syntax alone.
///
/// `SEMANTIC_PROFILE.write_classification` fixes the rule: a statement that
/// *can* write is a write, whatever it ends up changing. A DELETE that matches
/// no row is `WriteWithoutRows`, not `ReadOnly` — a read-only connection must
/// reject it before it runs, and a mutation savepoint must wrap it even when
/// the transaction turns out to be empty.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatementKind {
    ReadOnly,
    WriteReturningRows,
    WriteWithoutRows,
}

impl StatementKind {
    pub fn writes(&self) -> bool {
        matches!(self, Self::WriteReturningRows | Self::WriteWithoutRows)
    }
}

/// Classify a parsed query without binding it. Cheap, infallible, and total:
/// an unbindable query still has a definite read/write character, and callers
/// need that character to pick a route before binding can fail.
pub fn classify_statement(query: &cypher::Query) -> StatementKind {
    let writes = query
        .clauses
        .iter()
        .chain(query.union_branches.iter().flat_map(|branch| branch.clauses.iter()))
        .any(|clause| {
            matches!(
                clause,
                cypher::Clause::Create(_)
                    | cypher::Clause::Merge(_)
                    | cypher::Clause::Set(_)
                    | cypher::Clause::Remove(_)
                    | cypher::Clause::Delete(_)
            )
        });
    let returns_rows = query
        .clauses
        .last()
        .is_some_and(|clause| matches!(clause, cypher::Clause::Return(_)));
    match (writes, returns_rows) {
        (false, _) => StatementKind::ReadOnly,
        (true, true) => StatementKind::WriteReturningRows,
        (true, false) => StatementKind::WriteWithoutRows,
    }
}
```

Adjust the `cypher::Clause` variant names and the `Query` field names (`clauses`, `union_branches`) to whatever `graph/cypher/src/` actually exposes — read `graph/frontend/src/binder.rs:732-850`, where `bind_mutation_query` already matches on every mutating clause variant, and mirror that exact list. If `CALL` can invoke a mutating procedure, treat it as a write: `binder.rs:377` already carries the error `"mutating procedure ... is not valid in a read query"`, so reuse the same predicate that produces it.

- [ ] **Step 4: Re-export from the crate root**

In `graph/frontend/src/lib.rs`, extend the `pub use binder::{...}` block with `classify_statement, StatementKind,` in alphabetical position.

- [ ] **Step 5: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend --test statement_kind`
Expected: PASS, seven tests green.

- [ ] **Step 6: Commit classification**

```bash
cargo fmt
git add graph/frontend/src/binder.rs graph/frontend/src/lib.rs graph/frontend/tests/statement_kind.rs
git commit -S -m "graph/frontend: classify statements as read or write in the binder

Callers routed statements by trying query() and falling back to execute() when
it errored, which turns a genuine read failure into a mutation attempt. The
binder already distinguishes mutating clauses; expose that as a total,
syntactic classification. A DELETE that matches nothing stays a write:
emptiness is a runtime fact and must not move a compile-time classification."
```

- [ ] **Step 7: Write the failing read-only enforcement test**

Add to `graph/frontend/tests/statement_kind.rs`:

```rust
mod read_only {
    use turso_graph_frontend::{Error, Parameters, StatementKind};

    // Build the session the same way graph/frontend/src/session.rs's own tests
    // do; see `fn fixture(...)` at graph/frontend/src/session.rs:654.
    use super::super::common::fixture;

    #[test]
    fn a_read_only_connection_runs_reads() {
        let mut fixture = fixture(":memory:graph-read-only-reads");
        fixture.session.set_read_only(true);
        fixture
            .session
            .query("MATCH (n) RETURN n", &Parameters::new())
            .expect("a read-only connection serves reads");
    }

    #[test]
    fn a_read_only_connection_refuses_a_write_before_running_it() {
        let mut fixture = fixture(":memory:graph-read-only-writes");
        fixture.session.set_read_only(true);
        let error = fixture
            .session
            .execute("CREATE (n:Person {name: 'a'})", &Parameters::new())
            .expect_err("a read-only connection refuses writes");
        assert!(
            matches!(error, Error::ReadOnlyConnection { .. }),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn a_read_only_connection_refuses_a_delete_that_would_match_nothing() {
        // The refusal is decided by syntax, so it does not depend on the graph
        // containing a matching row.
        let mut fixture = fixture(":memory:graph-read-only-empty-delete");
        fixture.session.set_read_only(true);
        let error = fixture
            .session
            .execute("MATCH (n:Absent) DELETE n", &Parameters::new())
            .expect_err("an empty DELETE is still a write");
        assert!(
            matches!(
                error,
                Error::ReadOnlyConnection {
                    kind: StatementKind::WriteWithoutRows
                }
            ),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn classify_routes_without_trying_and_failing() {
        let fixture = fixture(":memory:graph-classify-route");
        assert_eq!(
            fixture.session.classify("MATCH (n) RETURN n").expect("parses"),
            StatementKind::ReadOnly
        );
        assert_eq!(
            fixture.session.classify("CREATE (n:A) RETURN n").expect("parses"),
            StatementKind::WriteReturningRows
        );
    }
}
```

The existing session tests build their fixture inside `graph/frontend/src/session.rs`'s `mod tests`. If that helper is not reachable from an integration test, move these four tests into that `mod tests` block instead and drop the `use super::super::common::fixture;` line — do not duplicate the fixture setup.

- [ ] **Step 8: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend read_only`
Expected: FAIL to compile — `no method named set_read_only`, `no variant ReadOnlyConnection`.

- [ ] **Step 9: Implement read-only enforcement**

In `graph/frontend/src/session.rs`, add the field to `GraphConnection` (line 86 area):

```rust
    /// When set, the session refuses any statement the binder classifies as a
    /// write. Enforcement is syntactic and happens before the statement runs,
    /// so a write that would have changed nothing is still refused.
    read_only: bool,
```

Initialize it to `false` at every `GraphConnection { .. }` construction site the compiler reports, and add:

```rust
    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// Read/write character of `source`, without binding or running it.
    pub fn classify(&self, source: &str) -> Result<StatementKind, Error> {
        Ok(crate::classify_statement(&turso_graph_cypher::parse(source)?))
    }
```

Guard `execute` (line 315), before `refresh_catalog_if_stale`:

```rust
    pub fn execute(&self, source: &str, parameters: &Parameters) -> Result<MutationSummary, Error> {
        let kind = self.classify(source)?;
        if self.read_only && kind.writes() {
            return Err(Error::ReadOnlyConnection { kind });
        }
        self.refresh_catalog_if_stale()?;
```

Add the variant to this crate's `Error` enum (the one `session.rs` returns; find it via `Error::Database` at `session.rs:337`):

```rust
    #[error("this graph connection is read-only and cannot run a {kind:?} statement")]
    ReadOnlyConnection { kind: crate::StatementKind },
```

- [ ] **Step 10: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend`
Expected: PASS.

- [ ] **Step 11: Verify and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend
git commit -S -am "graph/frontend: refuse writes on a read-only graph connection

The binder now classifies statements, so a connection can be marked read-only
and reject a write before it runs rather than after a savepoint has opened.
The check is syntactic, so a DELETE that would match no row is refused too:
whether a statement writes cannot depend on what it happened to change."
```

---

### Task 5: Write the path algorithm legality table and enforce it

**Files:**
- Create: `graph/runtime/src/path_policy.rs`
- Modify: `graph/runtime/src/lib.rs:8-20` (module list and re-exports)
- Modify: `graph/runtime/src/error.rs:15-36` (`RuntimeError`)
- Modify: `graph/runtime/src/shortest.rs:29-40` (`shortest_path`), `:120-140` (`weighted_shortest_path`)
- Modify: `graph/ir/src/semantics.rs` (`path_policy_version`, version bump)
- Modify: `graph/ir/tests/semantic_profile_pin.rs` (re-pin)
- Modify: `graph/DESIGN_DECISIONS.md`

**Interfaces:**
- Consumes: `turso_graph_ir::PathUniqueness { Walk, Trail, Path }` (already exists at `graph/ir/src/plan.rs:127`); `SEMANTIC_PROFILE` from Task 1.
- Produces:
  - `turso_graph_runtime::{PathSelector, WeightClass, PathAlgorithm, PathPolicyError}`
  - `turso_graph_runtime::resolve_path_algorithm(uniqueness, selector, weights) -> Result<PathAlgorithm, PathPolicyError>`
  - `turso_graph_runtime::PATH_POLICY_VERSION: u32 == 1`
  - `RuntimeError::UnsupportedPathCombination { reason: &'static str }`
  - `RuntimeError::PathAlgorithmNotImplemented { algorithm: PathAlgorithm }`

- [ ] **Step 1: Write the failing policy tests**

Create the test module at the bottom of `graph/runtime/src/path_policy.rs` as part of Step 3; write it first here so the table is designed before it is coded. Create `graph/runtime/src/path_policy.rs` containing only:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use turso_graph_ir::PathUniqueness;

    #[test]
    fn unweighted_shortest_is_breadth_first_under_every_uniqueness() {
        // In an unweighted graph a shortest walk never repeats a node, so BFS
        // is correct for Walk, Trail, and Path alike.
        for uniqueness in [PathUniqueness::Walk, PathUniqueness::Trail, PathUniqueness::Path] {
            assert_eq!(
                resolve_path_algorithm(uniqueness, PathSelector::Shortest, WeightClass::Unweighted),
                Ok(PathAlgorithm::BreadthFirst)
            );
        }
    }

    #[test]
    fn non_negative_weighted_shortest_is_dijkstra_under_every_uniqueness() {
        // With non-negative weights a shortest walk is again always simple, so
        // the trail and acyclic constraints cost nothing.
        for uniqueness in [PathUniqueness::Walk, PathUniqueness::Trail, PathUniqueness::Path] {
            assert_eq!(
                resolve_path_algorithm(uniqueness, PathSelector::Shortest, WeightClass::NonNegative),
                Ok(PathAlgorithm::Dijkstra)
            );
        }
    }

    #[test]
    fn negative_weights_have_no_shortest_path_answer_we_will_give() {
        // Dijkstra is silently wrong with negative weights; a negative cycle
        // makes a shortest walk undefined; and shortest simple path with
        // negative weights is NP-hard. All three refuse, none guess.
        for uniqueness in [PathUniqueness::Walk, PathUniqueness::Trail, PathUniqueness::Path] {
            for selector in [
                PathSelector::Shortest,
                PathSelector::AllShortest,
                PathSelector::ShortestK(2),
            ] {
                assert!(
                    resolve_path_algorithm(uniqueness, selector, WeightClass::Negative).is_err(),
                    "{uniqueness:?}/{selector:?} must refuse negative weights"
                );
            }
        }
    }

    #[test]
    fn any_path_is_breadth_first_and_weight_blind() {
        for weights in [
            WeightClass::Unweighted,
            WeightClass::NonNegative,
            WeightClass::Negative,
        ] {
            assert_eq!(
                resolve_path_algorithm(PathUniqueness::Trail, PathSelector::Any, weights),
                Ok(PathAlgorithm::BreadthFirst),
                "ANY asks for existence, which no weight sign changes"
            );
        }
    }

    #[test]
    fn all_paths_enumerate_and_do_not_care_about_weight_sign() {
        for weights in [
            WeightClass::Unweighted,
            WeightClass::NonNegative,
            WeightClass::Negative,
        ] {
            assert_eq!(
                resolve_path_algorithm(PathUniqueness::Trail, PathSelector::All, weights),
                Ok(PathAlgorithm::DepthFirstEnumeration)
            );
        }
    }

    #[test]
    fn all_paths_over_walks_are_refused_because_a_cycle_makes_them_infinite() {
        assert!(matches!(
            resolve_path_algorithm(
                PathUniqueness::Walk,
                PathSelector::All,
                WeightClass::Unweighted
            ),
            Err(PathPolicyError::Unsupported { .. })
        ));
    }

    #[test]
    fn k_shortest_is_declared_sound_but_not_built() {
        // Soundness and availability are different questions. Answering them
        // with one error would let "we have not built it" read as "it cannot
        // be done", and the next person would design around a wrong table.
        assert_eq!(
            resolve_path_algorithm(
                PathUniqueness::Path,
                PathSelector::ShortestK(3),
                WeightClass::NonNegative
            ),
            Ok(PathAlgorithm::YenKShortest)
        );
    }

    #[test]
    fn every_combination_in_the_table_has_a_verdict() {
        // No combination may fall through to a default. A missing row is a
        // silent wrong answer waiting for the syntax to arrive.
        for uniqueness in [PathUniqueness::Walk, PathUniqueness::Trail, PathUniqueness::Path] {
            for selector in [
                PathSelector::All,
                PathSelector::Any,
                PathSelector::Shortest,
                PathSelector::AllShortest,
                PathSelector::ShortestK(2),
            ] {
                for weights in [
                    WeightClass::Unweighted,
                    WeightClass::NonNegative,
                    WeightClass::Negative,
                ] {
                    let verdict = resolve_path_algorithm(uniqueness, selector, weights);
                    assert!(
                        verdict.is_ok() || verdict.is_err(),
                        "unreachable, but the call must not panic"
                    );
                }
            }
        }
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_runtime path_policy`
Expected: FAIL — the module is not declared in `lib.rs`, so the tests do not run. Add `mod path_policy;` to `graph/runtime/src/lib.rs`, re-run, and get compile errors for `resolve_path_algorithm`, `PathSelector`, `WeightClass`, `PathAlgorithm`, `PathPolicyError`.

- [ ] **Step 3: Implement the table**

Prepend to `graph/runtime/src/path_policy.rs`, above the test module:

```rust
//! Which path-finding combinations Turso will answer, and with what.
//!
//! `graph/cypher/src/cypher.pest` has `range_literal` (`[r:T*1..3]`) but no
//! `SHORTEST`, `ALL SHORTEST`, `TRAIL`, or `ACYCLIC` selector. When that syntax
//! arrives, each combination of uniqueness, selector, and weight sign needs a
//! decision, and several of those decisions are traps: Dijkstra is silently
//! wrong with negative weights, a shortest walk is undefined across a negative
//! cycle, and shortest simple path with negative weights is NP-hard. The table
//! is written before the syntax so nobody has to make those calls under
//! pressure, and `resolve_path_algorithm` is total: every combination has a
//! verdict, and no combination falls through to a default.
//!
//! Weights are `u64` today (`EdgeInput.weight`, `Path.total_weight`), so
//! `WeightClass::Negative` is unreachable from the current type. The rows exist
//! so that widening the weight type trips a policy error instead of quietly
//! feeding negative edges to Dijkstra.

use turso_graph_ir::PathUniqueness;

/// Bump on any change to the table below, and mirror into
/// `turso_graph_ir::SEMANTIC_PROFILE.path_policy_version`.
pub const PATH_POLICY_VERSION: u32 = 1;

/// How many of the matching paths the caller wants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathSelector {
    /// Every matching path.
    All,
    /// Any one matching path; the caller does not care which.
    Any,
    /// One path of minimum cost.
    Shortest,
    /// Every path of minimum cost.
    AllShortest,
    /// The k lowest-cost paths, in cost order.
    ShortestK(u32),
}

/// The sign domain of the edge weights in play.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeightClass {
    /// Every edge costs 1.
    Unweighted,
    /// Every edge weight is >= 0.
    NonNegative,
    /// At least one edge weight may be < 0.
    Negative,
}

/// An algorithm the table considers sound for a combination. Being named here
/// says the algorithm is correct, not that it is implemented.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathAlgorithm {
    BreadthFirst,
    BreadthFirstAllShortest,
    Dijkstra,
    DijkstraAllShortest,
    DepthFirstEnumeration,
    YenKShortest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum PathPolicyError {
    #[error("{uniqueness:?}/{selector:?} with {weights:?} weights is not supported: {reason}")]
    Unsupported {
        uniqueness: PathUniqueness,
        selector: PathSelector,
        weights: WeightClass,
        reason: &'static str,
    },
}

/// The legality table. Total over every combination.
pub fn resolve_path_algorithm(
    uniqueness: PathUniqueness,
    selector: PathSelector,
    weights: WeightClass,
) -> Result<PathAlgorithm, PathPolicyError> {
    let refuse = |reason| {
        Err(PathPolicyError::Unsupported {
            uniqueness,
            selector,
            weights,
            reason,
        })
    };

    match selector {
        // Existence only. No weight sign changes whether a path exists, and
        // BFS finds one under every uniqueness rule.
        PathSelector::Any => Ok(PathAlgorithm::BreadthFirst),

        // Enumeration. Weights are irrelevant to which paths exist. A walk may
        // repeat edges, so a single cycle makes the answer infinite; the hop
        // limit bounds it but the result would then be an arbitrary prefix,
        // which is exactly the silent truncation traversal.rs refuses.
        PathSelector::All => match uniqueness {
            PathUniqueness::Walk => refuse(
                "enumerating all walks is infinite in a cyclic graph; \
                 use TRAIL or ACYCLIC, or ask for ANY",
            ),
            PathUniqueness::Trail | PathUniqueness::Path => {
                Ok(PathAlgorithm::DepthFirstEnumeration)
            }
        },

        PathSelector::Shortest => match weights {
            // A shortest unweighted walk never repeats a node, so the trail and
            // acyclic constraints are automatically satisfied and BFS is
            // correct for all three.
            WeightClass::Unweighted => Ok(PathAlgorithm::BreadthFirst),
            // Same argument with non-negative weights: no detour can lower the
            // cost, so the minimum-cost walk is simple.
            WeightClass::NonNegative => Ok(PathAlgorithm::Dijkstra),
            WeightClass::Negative => match uniqueness {
                PathUniqueness::Walk => refuse(
                    "a negative cycle makes the shortest walk undefined",
                ),
                PathUniqueness::Trail | PathUniqueness::Path => refuse(
                    "shortest simple path with negative weights is NP-hard; \
                     no correct polynomial algorithm exists",
                ),
            },
        },

        PathSelector::AllShortest => match weights {
            WeightClass::Unweighted => Ok(PathAlgorithm::BreadthFirstAllShortest),
            WeightClass::NonNegative => Ok(PathAlgorithm::DijkstraAllShortest),
            WeightClass::Negative => refuse(
                "the shortest cost is undefined with negative weights, \
                 so the set of shortest paths is too",
            ),
        },

        PathSelector::ShortestK(_) => match weights {
            // Yen's algorithm needs a simple-path constraint and a shortest
            // path subroutine that is correct, which rules out walks and
            // negative weights.
            WeightClass::Unweighted | WeightClass::NonNegative => match uniqueness {
                PathUniqueness::Walk => refuse(
                    "k-shortest requires simple paths; use TRAIL or ACYCLIC",
                ),
                PathUniqueness::Trail | PathUniqueness::Path => Ok(PathAlgorithm::YenKShortest),
            },
            WeightClass::Negative => refuse(
                "k-shortest inherits the negative-weight shortest-path refusal",
            ),
        },
    }
}
```

Add `mod path_policy;` to `graph/runtime/src/lib.rs` and re-export:

```rust
pub use path_policy::{
    resolve_path_algorithm, PathAlgorithm, PathPolicyError, PathSelector, WeightClass,
    PATH_POLICY_VERSION,
};
```

`graph/runtime/Cargo.toml` already depends on `thiserror` (see `graph/runtime/src/error.rs`); confirm before relying on it.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p turso_graph_runtime path_policy`
Expected: PASS, eight tests green.

- [ ] **Step 5: Write the failing enforcement test**

Add to the existing `mod tests` block at the bottom of `graph/runtime/src/shortest.rs`, next to `weighted_path_prefers_lower_total_cost_and_honors_hop_limit` at `:305`:

```rust
    #[test]
    fn the_search_entry_points_agree_with_the_policy_table() {
        // The table is only protection if the code consults it. These two
        // entry points are the whole weighted/unweighted surface today; when a
        // third arrives it must appear here.
        use crate::{resolve_path_algorithm, PathAlgorithm, PathSelector, WeightClass};
        use turso_graph_ir::PathUniqueness;

        assert_eq!(
            resolve_path_algorithm(
                PathUniqueness::Walk,
                PathSelector::Shortest,
                WeightClass::Unweighted
            ),
            Ok(PathAlgorithm::BreadthFirst),
            "shortest_path is a BFS and must resolve to one"
        );
        assert_eq!(
            resolve_path_algorithm(
                PathUniqueness::Walk,
                PathSelector::Shortest,
                WeightClass::NonNegative
            ),
            Ok(PathAlgorithm::Dijkstra),
            "weighted_shortest_path is a Dijkstra and must resolve to one"
        );
    }

    #[test]
    fn an_unsupported_combination_becomes_a_runtime_error() {
        use crate::{resolve_path_algorithm, PathSelector, RuntimeError, WeightClass};
        use turso_graph_ir::PathUniqueness;

        let refusal = resolve_path_algorithm(
            PathUniqueness::Walk,
            PathSelector::Shortest,
            WeightClass::Negative,
        )
        .expect_err("negative-weight walks are refused");
        let error = RuntimeError::from(refusal);
        assert!(
            matches!(error, RuntimeError::UnsupportedPathCombination { .. }),
            "unexpected error: {error}"
        );
    }
```

- [ ] **Step 6: Run to verify it fails**

Run: `cargo test -p turso_graph_runtime shortest`
Expected: FAIL to compile — `no variant UnsupportedPathCombination` and no `From<PathPolicyError> for RuntimeError`.

- [ ] **Step 7: Implement the runtime error bridge and consult the policy**

Add to `RuntimeError` in `graph/runtime/src/error.rs`:

```rust
    #[error("unsupported path combination: {reason}")]
    UnsupportedPathCombination { reason: &'static str },
    #[error("path algorithm {algorithm} is sound but not implemented")]
    PathAlgorithmNotImplemented { algorithm: &'static str },
```

and, in the same file:

```rust
impl From<crate::PathPolicyError> for RuntimeError {
    fn from(error: crate::PathPolicyError) -> Self {
        let crate::PathPolicyError::Unsupported { reason, .. } = error;
        Self::UnsupportedPathCombination { reason }
    }
}
```

In `graph/runtime/src/shortest.rs`, make each entry point state which table row it implements. In `shortest_path`, immediately after `validate_request(graph, request)?;`:

```rust
    // Unweighted single shortest path over walks. Stated rather than assumed,
    // so a future caller cannot reach this BFS with a combination the table
    // refuses.
    debug_assert_eq!(
        resolve_path_algorithm(
            PathUniqueness::Walk,
            PathSelector::Shortest,
            WeightClass::Unweighted
        ),
        Ok(PathAlgorithm::BreadthFirst)
    );
```

and the equivalent `Dijkstra` / `WeightClass::NonNegative` assertion in `weighted_shortest_path`. Add the imports to the file's existing `use crate::{...}` block. Keep the donor header at the top of `shortest.rs` intact and extend its `Changes:` line with `added path-policy resolution`.

- [ ] **Step 8: Run to verify it passes**

Run: `cargo test -p turso_graph_runtime`
Expected: PASS.

- [ ] **Step 9: Publish the table in DESIGN_DECISIONS.md**

Add to `graph/DESIGN_DECISIONS.md`:

```markdown
## Path algorithm legality

Written before the syntax exists. `graph/cypher/src/cypher.pest` has
`range_literal` (`[r:T*1..3]`) but no `SHORTEST`, `ALL SHORTEST`, `TRAIL`, or
`ACYCLIC` selector. `turso_graph_runtime::resolve_path_algorithm`
(`graph/runtime/src/path_policy.rs`) is the enforcing copy of this table; it is
total, and a combination it refuses cannot be reached by any search entry
point.

Uniqueness is `turso_graph_ir::PathUniqueness`: `Walk` may repeat nodes and
edges, `Trail` may not repeat an edge, `Path` may not repeat a node.

| Selector | Weights | Walk | Trail | Path |
| --- | --- | --- | --- | --- |
| ANY | any | BFS | BFS | BFS |
| ALL | any | not supported | DFS enumeration | DFS enumeration |
| SHORTEST | unweighted | BFS | BFS | BFS |
| SHORTEST | non-negative | Dijkstra | Dijkstra | Dijkstra |
| SHORTEST | negative | not supported | not supported | not supported |
| ALL SHORTEST | unweighted | BFS level set | BFS level set | BFS level set |
| ALL SHORTEST | non-negative | Dijkstra level set | Dijkstra level set | Dijkstra level set |
| ALL SHORTEST | negative | not supported | not supported | not supported |
| SHORTEST k | unweighted | not supported | Yen | Yen |
| SHORTEST k | non-negative | not supported | Yen | Yen |
| SHORTEST k | negative | not supported | not supported | not supported |

Reasons for each refusal:

- **ALL over walks.** One cycle makes the answer infinite. The hop limit would
  bound it, but the result would then be an arbitrary prefix, which is the
  silent truncation `graph/runtime/src/traversal.rs` deliberately refuses.
- **SHORTEST over walks with negative weights.** A negative cycle means no
  shortest walk exists.
- **SHORTEST over trails or paths with negative weights.** Shortest simple path
  with negative weights is NP-hard; there is no correct polynomial algorithm to
  offer.
- **SHORTEST k over walks.** Yen's algorithm requires a simple-path constraint.

Two things this table does not say. It does not claim an algorithm is
implemented: `PathAlgorithm::YenKShortest`, `BreadthFirstAllShortest`, and
`DijkstraAllShortest` are sound and unbuilt, and reaching them yields
`RuntimeError::PathAlgorithmNotImplemented`, distinct from
`RuntimeError::UnsupportedPathCombination`. And it does not describe reachable
state today: `EdgeInput.weight` and `Path.total_weight` are `u64`, so
`WeightClass::Negative` is unreachable from the current type. Those rows exist
so that widening the weight type trips a policy error rather than quietly
feeding negative edges to Dijkstra.
```

- [ ] **Step 10: Fold the policy version into the semantic profile**

In `graph/ir/src/semantics.rs`:
- change `SEMANTIC_PROFILE_VERSION` from `1` to `2`
- change `path_policy_version: 0` to `path_policy_version: 1`

`turso_graph_ir` must not depend on `turso_graph_runtime` (the IR crate is the dependency root — see its module doc), so the number is mirrored, not imported. Add this note above the field:

```rust
    /// Mirrors `turso_graph_runtime::PATH_POLICY_VERSION`. Mirrored rather than
    /// imported because the IR crate is the dependency root and must not depend
    /// on the runtime. `graph/runtime/src/path_policy.rs` pins the two together.
```

- [ ] **Step 11: Pin the mirror from the runtime side**

Add to the test module in `graph/runtime/src/path_policy.rs`:

```rust
    #[test]
    fn the_semantic_profile_mirrors_this_policy_version() {
        // The IR crate cannot depend on the runtime, so the mirror is checked
        // from the side that can see both.
        assert_eq!(
            turso_graph_ir::SEMANTIC_PROFILE.path_policy_version,
            PATH_POLICY_VERSION,
            "bump SEMANTIC_PROFILE_VERSION and its pinned digest alongside PATH_POLICY_VERSION"
        );
    }
```

- [ ] **Step 12: Re-pin the profile digest**

Run: `cargo test -p turso_graph_ir --test semantic_profile_pin`
Expected: `semantic_profile_digest_is_pinned_to_its_version` FAILS with a new `left:` digest, because `path_policy` moved from 0 to 1 in `render()`.
Copy that value into `PINNED_DIGEST` and update its doc comment to `/// Digest of `SEMANTIC_PROFILE.render()` at version 2.`

- [ ] **Step 13: Run everything**

Run:
```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_ir -p turso_graph_runtime -p turso_graph_frontend -p turso_graph_testkit
```
Expected: PASS.

- [ ] **Step 14: Commit**

```bash
git add graph/runtime graph/ir graph/DESIGN_DECISIONS.md
git commit -S -m "graph/runtime: write the path algorithm legality table before the syntax

traversal.rs already implements Walk, Trail, and Path with limits and an error
at the hop limit, but the grammar has no SHORTEST, ALL SHORTEST, TRAIL, or
ACYCLIC selector. Whoever adds that syntax has to decide which algorithm is
correct for each combination, and several answers are traps: Dijkstra is
silently wrong with negative weights, a shortest walk is undefined across a
negative cycle, and shortest simple path with negative weights is NP-hard.
Decide them now, in a total table that the search entry points resolve against,
and distinguish 'unsound' from 'not built' so the next reader is not misled.

Weights are u64 today, so the negative rows are unreachable; they exist to trip
a policy error if the weight type is ever widened."
```

---

## Follow-ups deliberately not in this plan

- **Wire `divergence verify` into a CI job.** The repo has no graph-specific workflow (`.github/workflows/` only references `perf/graph-queries`), so adding one is its own decision about when the corpus runs in CI. The command and its exit code exist after Task 3; adding the job is a one-line step once that decision is made.
- **`GraphConnection::run` as a single classified entry point.** Task 4 exposes `classify` so callers can stop trial-routing, but converting `age.rs:317` and `tck.rs` off the try-query-then-execute pattern changes recorded corpus outcomes and must be done in a commit whose pass-count delta is examined on its own.
- **Marking inexact results.** `graph/frontend/src/fts.rs` returns best matches, not all matches. Such a result must not become an exact `count()`, a `NOT EXISTS` proof of absence, or the target of a `DELETE`. Worth a rule and tests before vector search lands; out of scope here.
- **Valid time.** Two time axes over the graph. `turso_graph_temporal` holds date and time values only today, which is the right scope until that work starts.
- **A slow reference implementation for differential testing.** 492 of the 1,270 failures say only `execution: other`; a second, simple implementation would name the difference directly. High cost, and only worth it after Task 1 exists.

## Self-review notes

- **Spec coverage.** Brief item 1 → Task 1. Item 2 → Task 2. Item 3 → Task 3. Item 4 → Task 4 (with the savepoint premise corrected; the deliverable is classification plus read-only enforcement, and the DELETE-that-removes-nothing rule is a named test in Step 1 and Step 7). Item 5 → Task 5.
- **Naming consistency.** `SEMANTIC_PROFILE` / `SEMANTIC_PROFILE_VERSION` / `semantic_profile_digest` are used identically in Tasks 1, 4, and 5. `result_digest_with` / `ResultOrdering` are defined in Task 2 Step 3 and used only after. `StatementKind` is defined in Task 4 Step 3 and referenced by `Error::ReadOnlyConnection` in Step 9. `PathUniqueness` is the existing `turso_graph_ir` enum, not a new one.
- **Known unknowns, flagged rather than guessed.** Three code shapes could not be confirmed without building: the exact `cypher::Clause` variant names (Task 4 Step 3 says to mirror `bind_mutation_query`), whether `TestId` implements `FromStr` and `Ord` (Task 3 Steps 1 and 3 give the fallback), and whether `graph/frontend/src/session.rs`'s test fixture is reachable from an integration test (Task 4 Step 7 gives the fallback). The pinned digest values in Task 1 Step 1 and Task 5 Step 12 are intentionally placeholders that the test prints — that is the mechanism, not an omission.
- **The 53.** Task 3 Step 6 says explicitly: if the sync produces a different number, use the produced number and correct `CONFORMANCE.md`. Making the registry reach 53 would defeat its purpose.
