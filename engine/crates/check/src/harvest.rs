// SPDX-License-Identifier: Apache-2.0
//! `harvest.pin.unread`: a pinned corpus export that did not read, as an error
//! that names the pin.
//!
//! [Spec 7](../../../../docs/spec/07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it)
//! says "a pinned export that the tier cannot read is a **finding that names
//! the pin**". Before this rule, the reason reached a reader only through an
//! anchor: `relation.target.unresolved` names the pin when an edge into it did
//! not bind. A pin that no anchor names reached nobody, so a tier that declared
//! a harvest and wrote no anchor into it yet heard nothing about an export that
//! was missing, moved or unpinned
//! ([#1311](https://github.com/headwater-ai/headwater/issues/1311)).
//!
//! This crate cannot read a pin: `headwater-import` depends on it and not the
//! other way round. So the caller reads each pin with
//! `headwater_import::harvest::over`, and hands this rule what it found as a
//! [`Harvest`]. The rule is about the declaration and not about the prose of a
//! file, so it creates no instance, and it enters the findings list for the
//! reason [`crate::pin`] does. It is never cached: the caller reads every pin
//! on every run, and each export path joins the read set, so `headwater gate`
//! over a later tree sees an export move.

use crate::finding::{Finding, Severity};
use crate::instance::Input;
use crate::scope::Scope;
use headwater_resolve::package::CONSUMER;

pub const RULE: &str = "harvest.pin.unread";

/// The grain of [`RULE`]: a fact about a pin the consumer declares, read
/// against the tree, so it creates no instance.
pub const SCOPE: Scope = Scope::taxonomy();

/// Which edition of [`findings`] reached a verdict.
pub const VERSION: u32 = 1;

/// Empty: a front-matter schema has no instance to hold this against.
pub const EXPORTABLE_AS: crate::scope::ExportTargets = &[];

/// One pinned export as a run read it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Harvest {
    /// The name the declaration is keyed by, under `harvests`.
    pub name: String,
    /// Where the committed export sits, relative to the repository root.
    pub at: String,
    /// The digest of the bytes at [`Self::at`], where they read, so that a
    /// gate sees them move whether or not they are the pinned bytes.
    pub digest: Option<String>,
    /// Why the export binds nothing, in the words its resolver refuses every
    /// string with, and `None` where it read.
    pub unread: Option<String>,
}

impl Harvest {
    /// What this reading read, as a read-set input: the export under its path
    /// from the root.
    pub fn input(&self) -> Input {
        Input::new(self.at.clone(), self.digest.as_deref())
    }
}

/// One error for each pin whose export did not read, in declaration order.
pub fn findings(harvests: &[Harvest]) -> Vec<Finding> {
    harvests
        .iter()
        .filter_map(|harvest| {
            let why = harvest.unread.as_ref()?;
            Some(Finding {
                rule: RULE,
                severity: Severity::Error,
                obligation: None,
                path: CONSUMER.to_string(),
                line: 0,
                column: 0,
                message: format!(
                    "`harvests.{}` pins an export at `{}` that binds nothing: {why}",
                    harvest.name, harvest.at
                ),
                remediation: "commit the export the pin names at that path, or pin the digest of \
                              the export that is there, or remove the pin"
                    .to_string(),
                patch: None,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn harvest(name: &str, unread: Option<&str>) -> Harvest {
        Harvest {
            name: name.to_string(),
            at: format!("harvest/{name}.json"),
            digest: None,
            unread: unread.map(str::to_string),
        }
    }

    #[test]
    fn a_pin_that_read_is_silent_and_one_that_did_not_is_named() {
        let found = findings(&[
            harvest("repo-a", None),
            harvest("repo-b", Some("it did not read")),
        ]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].rule, RULE);
        assert_eq!(found[0].severity, Severity::Error);
        assert_eq!(found[0].path, CONSUMER);
        assert!(
            found[0].message.contains("`harvests.repo-b`")
                && found[0].message.contains("harvest/repo-b.json")
                && found[0].message.contains("it did not read"),
            "{}",
            found[0].message
        );
    }

    #[test]
    fn no_pin_is_no_finding() {
        assert!(findings(&[]).is_empty());
    }

    #[test]
    fn every_unread_pin_is_its_own_finding_in_declaration_order() {
        let found = findings(&[
            harvest("repo-a", Some("it did not read")),
            harvest("repo-b", None),
            harvest("repo-c", Some("it is not the pinned artifact")),
        ]);
        let heads: Vec<bool> = found
            .iter()
            .zip(["repo-a", "repo-c"])
            .map(|(finding, name)| finding.message.starts_with(&format!("`harvests.{name}`")))
            .collect();
        assert_eq!(found.len(), 2);
        assert_eq!(heads, vec![true, true]);
    }
}
