// SPDX-License-Identifier: Apache-2.0
//! What `diff.rs` and `migration.rs` both need, held once.
//!
//! Both targets build a scratch repository out of this one's `packages/`,
//! `docs/taxonomies/` and `.headwater/`, and both then read a step from one
//! package version to the next.

use std::path::Path;

/// Fix the copied package at one version, whatever this repository's own
/// package is at.
///
/// Every case in both targets reads a step from 1.0.0 to 2.0.0. Without this
/// the base version is inherited from `packages/`, so a major of the real
/// package rewrites a literal in every case at once. The version under test is
/// a property of the case and not of the repository the fixture is copied from.
///
/// Three files carry the number and all three move together: the manifest that
/// `resolve::package::sources` holds a consumer to, the taxonomy source beside
/// it, and the consumer declaration that pins what this root takes. The
/// manifest is read first, because it is the one the resolver compares.
pub(crate) fn pin(at: &Path, version: &str) {
    let manifest = at.join("packages/headwater-standard/package.yml");
    let text = std::fs::read_to_string(&manifest).expect("the manifest reads");
    let found = text
        .lines()
        .find_map(|line| line.strip_prefix("version: "))
        .expect("the manifest declares a version")
        .to_string();

    for (relative, indent) in [
        ("packages/headwater-standard/package.yml", ""),
        ("packages/headwater-standard/taxonomy.yml", ""),
        (".headwater/taxonomy.yml", "  "),
    ] {
        let path = at.join(relative);
        let text = std::fs::read_to_string(&path).expect("the source reads");
        let from = format!("\n{indent}version: {found}\n");
        assert!(
            text.contains(&from),
            "{relative} does not declare version {found}"
        );
        let to = format!("\n{indent}version: {version}\n");
        std::fs::write(&path, text.replacen(&from, &to, 1)).expect("the source writes");
    }
}
