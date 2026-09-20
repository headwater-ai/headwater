// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check: a verification's freshness against the committed
//! observation snapshot and the acceptance criterion it proves.
//!
//! # The gap this closes
//!
//! [HW-DR-0073](../../../../docs/decisions/0073-a-verification-is-a-kind-and-its-identity-is-minted-rather-than-found-in-the-code-that-cites-it.md)
//! ruling 3: "A verification carries three states, which are `declared`,
//! `observed at commit X` and `suspect` ... A verification is `suspect` when
//! the criterion it proves changed after the snapshot commit, which is the
//! rule DOORS states as P.6." Nothing read that comparison before this rule:
//! [`crate::observation::Observations`] could name a verification and the
//! commit it ran at, and nothing turned "the criterion changed since" into a
//! finding. [#937](https://github.com/headwater-ai/headwater/issues/937) is
//! the report, and this module is the rule.
//!
//! # Content stands in for history, and the module comment on why
//!
//! "Changed after the snapshot commit" reads as a question for version
//! control, and [`crate::observation`]'s own module comment states why this
//! crate does not ask one: it runs no VCS command and opens no file that a
//! caller did not name. The comparison this rule actually needs is narrower
//! than the sentence suggests. A snapshot that recorded the acceptance
//! criterion's content digest at the moment it observed the verification, set
//! against that criterion's digest today, answers the same question a commit
//! walk would: the two disagree exactly when the criterion's bytes moved at
//! some point after the snapshot, which is what "changed after the snapshot
//! commit" means. [`crate::observation::Observation::Verification`] is that
//! recorded digest, and [`EdgeEnd::digest`](crate::scope::EdgeEnd::digest) is
//! today's, off the same census read every other edge-scoped rule already
//! trusts. Neither needs a git command, and the `commit` field stays
//! provenance: a person reading the snapshot still sees which run it was, and
//! [HW-OBL-0199](../../../../docs/obligations/0199-an-observation-snapshot-records-a-commit-and-nothing-reads-it-back.md)
//! is where the syntax check on that field, and the boundary it stays inside
//! of, are recorded.
//!
//! # What is not suspect
//!
//! `declared` — no committed snapshot names this verification yet — is a
//! pass, not a finding: a verification with nothing observed yet is not an
//! author's mistake, it is a project not yet at that stage. `observed at
//! <commit>` — a snapshot names it and the criterion has not moved since — is
//! also a pass, for the same reason [`crate::basis`] passes a warrant that
//! supports its claim: the interesting report is the one case that needs a
//! person, and the other two would make every green run read as a wall of
//! text naming nothing wrong.
//!
//! # The denominator
//!
//! Every relation the taxonomy declares whose target kind includes
//! `verification`, and no other, the same declaration-driven shape
//! [`crate::basis`] takes for the `evidence` family: a taxonomy that renames
//! `proven_by` or adds a second relation reaching a verification is read by
//! this rule unchanged, because the rule holds no relation name of its own.
//!
//! # The three silences
//!
//! An entry with no declared half passes: an author who wrote only the
//! inverse half named nothing for this rule to anchor a finding on, and a
//! reciprocity rule is the one that reports the missing half.
//!
//! An edge whose far end is not a document passes: `EdgeUnit::Pair` never
//! groups one, so this is unreachable rather than tolerated, on
//! [`crate::basis`]'s own precedent.
//!
//! An edge whose criterion end carries no digest passes: the census read no
//! bytes there, which is a different report with a different owner.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::observation::Observations;
use crate::scope::{EdgeCheck, EdgeUnit, EdgeView};
use headwater_graph::declarations::Relation;
use headwater_graph::Declarations;

pub const RULE: &str = "relation.target.verification.suspect";

/// The kind a relation must reach for this rule to instantiate over it.
const TARGET_KIND: &str = "verification";

/// A group with no half at all, which the instantiation never produces. As
/// [`crate::basis`]: recorded rather than panicked on.
const NO_HALF: &str = "the entry carries no declared half";

/// The far end is not a document. `EdgeUnit::Pair` never groups one, so this
/// is unreachable rather than tolerated.
const NO_PAIR: &str = "this edge has no second document to read";

/// The check, generated from every relation that reaches a `verification`.
pub struct Verified<'a> {
    /// Every relation whose `to` names [`TARGET_KIND`]. Empty for a taxonomy
    /// that declares no such relation, and then this rule generates no
    /// instance at all.
    declared: Vec<&'a Relation>,
    /// The committed observation snapshot, read once for the whole run on the
    /// same terms [`crate::register::Projection::of`] reads it.
    observations: &'a Observations,
}

impl<'a> Verified<'a> {
    pub fn over(declarations: &'a Declarations, observations: &'a Observations) -> Self {
        Verified {
            declared: declarations
                .relations
                .iter()
                .filter(|relation| relation.to.iter().any(|kind| kind == TARGET_KIND))
                .collect(),
            observations,
        }
    }
}

impl EdgeCheck for Verified<'_> {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 1;
    /// Both ends have to be documents: the rule reads a content digest at
    /// each of them.
    const UNIT: EdgeUnit = EdgeUnit::Pair;
    /// This rule reads [`Observations`] directly in [`Verified::evaluate`],
    /// so its cache key has to carry the snapshot's digest or an edit to
    /// `.headwater/observations.yml` with no other document moving would
    /// never be seen again after the first cache write. See
    /// [`crate::scope::EdgeCheck::NEEDS_OBSERVATIONS`].
    const NEEDS_OBSERVATIONS: bool = true;

    /// A relation reaching [`TARGET_KIND`], and no other.
    fn instantiates(&self, relation: &str) -> bool {
        self.declared.iter().any(|known| known.name == relation)
    }

    fn evaluate(&self, view: &EdgeView<'_>) -> Outcome {
        let Some(half) = view.declared_half().or_else(|| view.inverse_half()) else {
            return Outcome::Skipped(NO_HALF.to_string());
        };
        let Some((criterion, verification)) = view.ends() else {
            return Outcome::Skipped(NO_PAIR.to_string());
        };

        let Some(current_digest) = criterion.digest() else {
            return Outcome::Skipped(
                "the census read no digest for the document at the criterion end of this edge, \
                 so there is nothing to compare a snapshot against"
                    .to_string(),
            );
        };

        // `declared`: no committed snapshot names this verification. A pass,
        // and the module comment says why.
        let Some((commit, recorded_digest)) = self.observations.verification(verification.id)
        else {
            return Outcome::Passed;
        };

        // `observed at <commit>`: the snapshot's own digest of the criterion
        // still matches what the criterion reads today. A pass.
        if recorded_digest == current_digest {
            return Outcome::Passed;
        }

        // `suspect`: the criterion changed after the snapshot. See the module
        // comment for why a content digest answers the same question a
        // commit walk would.
        let (line, column) = at(Some(half.span));
        Outcome::failed_with(Finding {
            rule: self::RULE,
            severity: Severity::Warn,
            obligation: None,
            path: half.source.path.clone(),
            line,
            column,
            message: message(criterion.id, verification.id, commit),
            remediation: remediation(verification.id),
            // No fix. Whether the change to the criterion still leaves the
            // verification settling it is a person's judgment, the same
            // reason `basis.rs` and `suspect.rs` both carry no patch.
            patch: None,
        })
    }
}

/// What moved, in the terms of the criterion an author is looking at.
fn message(criterion: &str, verification: &str, commit: &str) -> String {
    format!(
        "`{criterion}` names `proven_by: {verification}`, which a snapshot observed at commit \
         `{commit}`; `{criterion}` has changed since that snapshot was written, so \
         `{verification}` is suspect until a new snapshot observes it against the criterion as \
         it now reads"
    )
}

/// What to do about it, which is one sentence because there is one remedy.
fn remediation(verification: &str) -> String {
    format!(
        "re-run the verification against the criterion as it now reads and commit a fresh entry \
         for `{verification}` in `.headwater/observations.yml`, or confirm the change to the \
         criterion did not affect what `{verification}` proves and record a new snapshot anyway"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_message_names_the_criterion_the_verification_and_the_commit() {
        let text = message("FIX-AC-1", "FIX-VER-1", "788885a9");
        assert!(text.contains("FIX-AC-1"), "{text}");
        assert!(text.contains("proven_by: FIX-VER-1"), "{text}");
        assert!(text.contains("commit `788885a9`"), "{text}");
        assert!(text.contains("suspect"), "{text}");
    }

    #[test]
    fn the_remediation_names_the_verification_and_the_snapshot_file() {
        let text = remediation("FIX-VER-1");
        assert!(text.contains("FIX-VER-1"), "{text}");
        assert!(text.contains(".headwater/observations.yml"), "{text}");
        // And it never tells the reader to reach for a version-control
        // command: the module comment states why this rule stays on content.
        assert!(!text.contains("git "), "{text}");
    }
}
