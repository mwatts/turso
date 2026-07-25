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
/// formatting or comments. A new field still changes the digest, so it still
/// requires a bump even when its value restates existing behavior.
pub const SEMANTIC_PROFILE_VERSION: u32 = 2;

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
    /// identifies the whole semantic contract. Mirrored rather than imported
    /// because this crate is the dependency root and must not depend on the
    /// runtime. `graph/runtime/src/path_policy.rs` pins the two together, from
    /// the side that can see both crates.
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
    path_policy_version: 1,
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
