//! The semantic profile is a contract, not documentation. Any change to a
//! recorded choice must move `SEMANTIC_PROFILE_VERSION`, because every row in
//! `graph/test-results/history.jsonl` is interpreted against that number. This
//! test fails loudly on an unversioned edit and prints the digest to paste.

use turso_graph_ir::{semantic_profile_digest, SEMANTIC_PROFILE, SEMANTIC_PROFILE_VERSION};

/// Digest of `SEMANTIC_PROFILE.render()` at version 3.
const PINNED_DIGEST: &str = "fnv1a64:d064f72078704012";

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
