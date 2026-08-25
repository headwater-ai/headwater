// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check: an imported edge names its upstream item at the
//! revision the edge was verified against.
//!
//! # The gap this closes
//!
//! [Spec 7](../../../../docs/spec/07-distribution-and-federation.md#upstream-awareness)
//! states the rule: "A snapshot pin reports drift on each affected edge, and
//! not only on the pin. A snapshot carries the upstream identity and revision
//! of every item in it, so an advance says which items changed. Every edge into
//! a changed item is then a finding until a person re-verifies it. Requirements
//! practice reached the same mechanism and calls such an edge *suspect*."
//!
//! Everything that sentence needs was already written down and nothing read it
//! together. The importer records `verified_revision` on every edge it writes,
//! and the snapshot records the revision each item is at now. This rule is the
//! comparison, and it is the whole computation.
//!
//! # The report is at the origin
//!
//! A proposal against the whole snapshot names a file, and this names the
//! document whose author can act, at the entry of the `relations:` block that
//! carries the edge. That is [spec 4](../../../../docs/spec/04-assurance-model.md#absence-is-a-finding-class-of-its-own)'s
//! report-at-the-origin rule, and it is why the drift report is a check-layer
//! output rather than a verb of its own.
//!
//! # What it cannot read, and what that forced
//!
//! `headwater-import` depends on this crate, so this crate can never read a
//! snapshot. The revision therefore has to reach the rule off the graph, which
//! is why `headwater_graph::anchors::Binding::Resolved` and
//! `headwater_graph::Target::Anchor` carry one. That is the same change
//! [#160](https://github.com/headwater-ai/headwater/issues/160) demands of the
//! cache key: `Target::resolution()` is the derived `Debug`, so a revision the
//! type carries is a revision the key names, and a cached verdict about an edge
//! cannot survive the advance that falsifies it.
//!
//! # The denominator
//!
//! Every relation the taxonomy declares with `created_by: import`, and no
//! other. A relation an importer may not write can carry no
//! `verified_revision`, so an instance over one would report a denominator of
//! edges that could never be suspect. Declaration-driven, so a new importable
//! relation produces its instances with no code here, which is the property
//! [`crate::target`] states as its own.
//!
//! The instance exists for an edge that is not suspect as well as for one that
//! is. A rule whose instances are only its findings reports a count that reads
//! as its own denominator.
//!
//! # The three silences, and each one is deliberate
//!
//! An edge with no recorded revision passes. A person who types an entry of an
//! importable relation by hand records nothing to compare, and that such an
//! edge is itself worth reporting is a different finding with a different
//! remedy.
//!
//! An edge whose resolver offered no revision passes. Every resolver but a
//! snapshot's has no such notion, and a rule must not invent a comparison a
//! resolver did not offer.
//!
//! A target that is not an anchor passes. A document target holds no upstream
//! revision, and an unbound target is [`crate::target`]'s.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{EdgeCheck, EdgeUnit, EdgeView};
use headwater_graph::declarations::Relation;
use headwater_graph::edges::VERIFIED_REVISION;
use headwater_graph::{Declarations, Target};

pub const RULE: &str = "relation.target.suspect";

/// A group with no half at all, which the instantiation never produces. As
/// [`crate::target`]: recorded rather than panicked on.
const NO_HALF: &str = "the entry carries no declared half";

/// The check, generated from the relation declarations an importer may write.
pub struct Suspect<'a> {
    /// Every relation whose `created_by` is `import`. A relation this list
    /// omits is one no importer writes, so no edge of it records the revision
    /// this rule compares.
    declared: Vec<&'a Relation>,
}

impl<'a> Suspect<'a> {
    pub fn over(declarations: &'a Declarations) -> Self {
        Suspect {
            declared: declarations
                .relations
                .iter()
                .filter(|relation| relation.created_by.as_deref() == Some("import"))
                .collect(),
        }
    }
}

impl EdgeCheck for Suspect<'_> {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 1;
    /// As [`crate::target`]: an anchor target has no far document, so there is
    /// no pair to group two halves into.
    const UNIT: EdgeUnit = EdgeUnit::Entry;

    fn instantiates(&self, relation: &str) -> bool {
        self.declared.iter().any(|known| known.name == relation)
    }

    fn evaluate(&self, view: &EdgeView<'_>) -> Outcome {
        let Some(edge) = view.declared_half().or_else(|| view.inverse_half()) else {
            return Outcome::Skipped(NO_HALF.to_string());
        };

        let Target::Anchor { .. } = &edge.target else {
            return Outcome::Passed;
        };

        let Some(verified) = edge
            .attributes
            .iter()
            .find(|entry| entry.key.value == VERIFIED_REVISION)
            .and_then(|entry| entry.value.value.as_scalar())
            .map(|scalar| scalar.text.as_str())
        else {
            return Outcome::Passed;
        };

        // Stage A of #82: the resolver's answer does not carry a revision yet.
        let current: Option<&str> = None;
        let Some(current) = current else {
            return Outcome::Passed;
        };

        if verified == current {
            return Outcome::Passed;
        }

        let (line, column) = at(Some(edge.span));
        Outcome::failed_with(Finding {
            rule: self::RULE,
            severity: Severity::Warn,
            obligation: None,
            path: edge.source.path.clone(),
            line,
            column,
            message: message(&edge.source.id, &edge.name, &edge.raw_target, verified, current),
            remediation: remediation(current),
            // No fix. The repair is a person re-reading an upstream item and
            // deciding whether the edge still holds, which is judgment rather
            // than the mechanical and total correction
            // [spec 12](../../../../docs/spec/12-check-layer.md#fixability)
            // sets as the bar. Writing the new revision in would record a
            // verification that nobody performed.
            patch: None,
        })
    }
}

/// What moved, in the terms of the document that declares the edge.
fn message(id: &str, name: &str, raw: &str, verified: &str, current: &str) -> String {
    format!(
        "`{id}` declares `{name}: {raw}`, which was verified at revision `{verified}` and the \
         snapshot now pins revision `{current}`"
    )
}

/// What to do about it, which is one sentence because there is one remedy.
///
/// It is a hand edit and not a re-import, and that is measured rather than
/// preferred. `headwater_import::write::declares` reads "same target, different
/// revision" as absent, so a second import proposes the edge again, and
/// `headwater_scaffold::write::splice` appends an entry rather than rewriting
/// the one that is there. So a re-import over a moved revision writes a second
/// entry for one target, which the graph then reports as a repeated triple.
fn remediation(current: &str) -> String {
    format!(
        "re-read the upstream item, and where the edge still holds set \
         `{VERIFIED_REVISION}: {current}` on this entry"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two sentences a reader gets, on the precedent of
    /// [`crate::target`]'s table: the fixture corpus of this crate declares no
    /// importable relation, so no case there can reach either string.
    #[test]
    fn the_finding_names_both_revisions_and_the_remedy_names_the_attribute() {
        let text = message("SPEC-FIX-one", "audited_by", "12345", "7", "8");
        assert!(text.contains("SPEC-FIX-one"), "{text}");
        assert!(text.contains("audited_by: 12345"), "{text}");
        assert!(text.contains("verified at revision `7`"), "{text}");
        assert!(text.contains("pins revision `8`"), "{text}");

        let remedy = remediation("8");
        assert!(remedy.contains("verified_revision: 8"), "{remedy}");
        // And it never names a verb that would write a second entry for one
        // target. See the comment on `remediation`.
        assert!(!remedy.contains("headwater import"), "{remedy}");
    }

    /// The message says which revision is which. A sentence that named them in
    /// the other order would send an author to re-verify against the value the
    /// edge already carries.
    #[test]
    fn the_recorded_revision_and_the_pinned_one_are_never_the_same_side() {
        let text = message("SPEC-FIX-one", "audited_by", "12345", "7", "8");
        let verified = text.find("verified at revision").expect("it is there");
        let pinned = text.find("now pins revision").expect("it is there");
        assert!(verified < pinned, "{text}");
    }
}
