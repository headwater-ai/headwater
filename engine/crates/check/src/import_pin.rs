// SPDX-License-Identifier: Apache-2.0
//! `import.pin.unread`: a committed imports snapshot that binds nothing, as an
//! error that names the pin.
//!
//! [Q19](../../../../docs/spec/09-decisions.md#q19--inbound-integration-an-external-system-of-record) says a
//! committed snapshot "is a pin". Before this rule, a snapshot that did not
//! read reached a reader only through an anchor: `relation.target.unresolved`
//! carries the reason when an edge into it did not bind. A pin that no anchor
//! names reached nobody, so a repository that declared an import and wrote no
//! anchor into it yet heard nothing about a snapshot that was missing, moved or
//! unpinned ([#1345](https://github.com/headwater-ai/headwater/issues/1345)).
//! This is the twin of [`crate::harvest`], which does the same for a pinned
//! corpus export.
//!
//! This crate cannot read a snapshot: `headwater-import` depends on it and not
//! the other way round. So the caller reads each snapshot with
//! `headwater_import::anchors::over`, and hands this rule what it found as an
//! [`Import`]. The rule is about the declaration and not about the prose of a
//! file, so it creates no instance, and it enters the findings list for the
//! reason [`crate::pin`] does. It is never cached: the caller reads every
//! snapshot on every run, and each file of the snapshot joins the read set, so
//! `headwater gate` over a later tree sees a snapshot move.

use crate::finding::{Finding, Severity};
use crate::instance::Input;
use crate::scope::Scope;
use headwater_resolve::package::CONSUMER;

pub const RULE: &str = "import.pin.unread";

/// The grain of [`RULE`]: a fact about a pin the consumer declares, read
/// against the tree, so it creates no instance.
pub const SCOPE: Scope = Scope::taxonomy();

/// Which edition of [`findings`] reached a verdict.
pub const VERSION: u32 = 1;

/// Empty: a front-matter schema has no instance to hold this against.
pub const EXPORTABLE_AS: crate::scope::ExportTargets = &[];

/// One declared import snapshot as a run read it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Import {
    /// The name the declaration is keyed by, under `imports`.
    pub name: String,
    /// Where the committed snapshot directory sits, relative to the repository
    /// root.
    pub at: String,
    /// Each file of the snapshot as the run found it, under its path from the
    /// root: the release record and the payload always, with no digest where
    /// either is absent, and every other member the directory holds with the
    /// digest of its bytes. A gate then sees the snapshot move whether or not
    /// it is the pinned artifact.
    pub inputs: Vec<Input>,
    /// Why the snapshot binds nothing, in the words its resolver refuses every
    /// string with, and `None` where it read.
    pub unread: Option<String>,
}

impl Import {
    /// What this reading read, as read-set inputs, sorted by path.
    pub fn inputs(&self) -> Vec<Input> {
        let mut out = self.inputs.clone();
        out.sort_by(|a, b| a.path.cmp(&b.path));
        out.dedup_by(|a, b| a.path == b.path);
        out
    }
}

/// One error for each pin whose snapshot binds nothing, in declaration order.
pub fn findings(imports: &[Import]) -> Vec<Finding> {
    imports
        .iter()
        .filter_map(|import| {
            let why = import.unread.as_ref()?;
            Some(Finding {
                rule: RULE,
                severity: Severity::Error,
                obligation: None,
                path: CONSUMER.to_string(),
                line: 0,
                column: 0,
                message: format!(
                    "`imports.{}` pins a snapshot at `{}` that binds nothing: {why}",
                    import.name, import.at
                ),
                remediation: "commit the snapshot the pin names at that path, or pin the digest \
                              of the snapshot that is there, or remove the pin"
                    .to_string(),
                patch: None,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn import(name: &str, unread: Option<&str>) -> Import {
        Import {
            name: name.to_string(),
            at: format!(".headwater/imports/{name}"),
            inputs: Vec::new(),
            unread: unread.map(str::to_string),
        }
    }

    #[test]
    fn a_snapshot_that_read_is_silent_and_one_that_did_not_is_named() {
        let found = findings(&[
            import("upstream", None),
            import("tracker", Some("it did not read")),
        ]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].rule, RULE);
        assert_eq!(found[0].severity, Severity::Error);
        assert_eq!(found[0].path, CONSUMER);
        assert!(
            found[0].message.starts_with(
                "`imports.tracker` pins a snapshot at `.headwater/imports/tracker` that binds \
                 nothing: it did not read"
            ),
            "{}",
            found[0].message
        );
    }

    #[test]
    fn no_import_is_no_finding() {
        assert!(findings(&[]).is_empty());
    }

    #[test]
    fn every_unread_snapshot_is_its_own_finding_in_declaration_order() {
        let found = findings(&[
            import("a", Some("it did not read")),
            import("b", None),
            import("c", Some("it is not the pinned artifact")),
        ]);
        let heads: Vec<bool> = found
            .iter()
            .zip(["a", "c"])
            .map(|(finding, name)| finding.message.starts_with(&format!("`imports.{name}`")))
            .collect();
        assert_eq!(found.len(), 2);
        assert_eq!(heads, vec![true, true]);
    }

    #[test]
    fn the_inputs_are_sorted_and_name_each_path_once() {
        let mut reading = import("upstream", None);
        reading.inputs = vec![
            Input::new(".headwater/imports/upstream/snapshot.yml", Some("sha256:b")),
            Input::new(".headwater/imports/upstream/release.yml", None),
            Input::new(".headwater/imports/upstream/snapshot.yml", Some("sha256:b")),
        ];
        let paths: Vec<String> = reading.inputs().into_iter().map(|i| i.path).collect();
        assert_eq!(
            paths,
            vec![
                ".headwater/imports/upstream/release.yml".to_string(),
                ".headwater/imports/upstream/snapshot.yml".to_string(),
            ]
        );
    }
}
