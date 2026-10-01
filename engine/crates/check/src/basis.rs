// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check: a document that claims `evidence_basis: evidenced`
//! does not rest that claim on a document nobody has read.
//!
//! # The gap this closes
//!
//! [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two)
//! already rules it, and has since the table of three honest states was
//! written: "A pointer to a document with the `asserted`
//! [warrant](../../../../docs/spec/01-conceptual-model.md#warrant) does not
//! support `evidenced`, because such a document is neither external nor
//! auditable. A fabricated *why* with a file name is the failure that this
//! table exists to prevent."
//!
//! The clause stood in the specification and no rule read it, so the failure it
//! names is exactly the failure a corpus could commit with every check green.
//! [#569](https://github.com/headwater-ai/headwater/issues/569) is the report.
//!
//! # What it reads, and why neither value comes from a declaration
//!
//! Both values live in the provenance block, which
//! [spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed)
//! makes the engine's rather than a taxonomy's: "The provenance block is the
//! exception, and its shape belongs to the engine." So no taxonomy declares a
//! warrant and none declares an evidence basis, and this rule reads both
//! through [`headwater_doc`] rather than by opening the block by key, on the
//! terms [`crate::promotion`] states. A second reader of that block is a second
//! answer to what a warrant is.
//!
//! # The family is read, and never one relation name
//!
//! Spec 3 says *a pointer*, and the pointers that carry evidential weight are
//! the `evidence` family. That name is not this rule's invention: the
//! meta-schema owns the closed set of six families, `evidence` is one of them,
//! and `headwater_graph::declarations::Relation::family` reads it as written.
//! So the rule holds one family name for the reason [`crate::promotion`] holds
//! two warrant values: the vocabulary is the engine's, and a taxonomy that
//! renames its relations is read by this rule unchanged.
//!
//! This repository's own lock resolves that family to eleven relations —
//! `traces_to`, `applied_in`, `assesses`, `cites_evidence`, `cited_in`,
//! `discharges`, `draws_on`, `examines`, `proven_by`, `records` and
//! `verified_by`. A rule that held `traces_to` alone would read one of the
//! eleven, which is the mistake that shrinks the population, and
//! `tests/evidence_basis.rs` writes one edge of each of eight so that no later
//! change can make it quietly.
//!
//! # Which end carries which question
//!
//! One end makes the claim and the other is the evidence. The claim is the
//! claimant's `evidence_basis`, which says what that document rests on. The
//! warrant is the evidence's, which says whether anybody has read the thing
//! being rested on. A rule that took either question to the wrong end reads a
//! corpus that is almost always green, because most documents are `accepted`
//! and most claim `evidenced`.
//!
//! Which end is which is the relation's declaration, `evidence_at`, and never
//! this rule's guess. Spec 2 leaves the direction of a relation to its author,
//! so the family cannot supply it. For most relations the evidence is the
//! **target**: `traces_to` points from the claim to what it rests on, and that
//! is the reading when a relation declares `to` or nothing. `discharges`
//! points the other way. The evaluation at the source end substantiates the
//! obligation at the target end, so it declares `evidence_at: from`, and the
//! rule reads the claim at the target and the warrant at the source. Spec 1
//! says why: "An asserted document does not discharge an evidence obligation."
//! An edition that read every relation one way reported an evidenced evaluation
//! for discharging an asserted obligation, and passed an asserted evaluation
//! that discharged an evidenced one (#1511).
//!
//! The direction comes from the relation and never from the file that wrote the
//! entry, which is what [`crate::scope::EdgeView::ends`] normalizes once for
//! every edge-grained rule. The finding still anchors at the declared half
//! whichever end is the claimant, because that is the entry an author is
//! looking at.
//!
//! # A value outside the closed set supports nothing
//!
//! `proposed` is a warrant this corpus writes and spec 3's closed set does not
//! name;
//! [HW-OBL-0125](../../../../docs/obligations/0125-nine-documents-state-a-warrant-outside-the-closed-set-and-only-a-person-can-set-the-value.md)
//! holds the nine documents that do. An earlier edition passed it, because it
//! is not `asserted` and the defect was the value, which another rule would
//! report. No rule did, so a misspelling such as `acepted` read as support
//! (#1438). [`crate::warrant`] now reports the value, and this rule reads a
//! value outside the set as what it is: nothing in it says a person read the
//! target. So the pointer is reported, with a message that names the value as
//! written and does not call it `asserted`.
//!
//! A target that declares **no** warrant is a different answer again, and it
//! skips. A rule that collapsed the absence into "not `asserted`" would return
//! green over a corpus that had lost every warrant it declares.
//!
//! # A generated target declares no warrant and has one
//!
//! Spec 3 again: "The engine derives `regenerated` from the marker, and never
//! from a declaration. A generated file that declares no front matter has no
//! block in which to state a warrant. A generated document that declares an
//! identity has one, and a declared warrant there would be a fact that a hand
//! edit can falsify." So the absence at such a target is a property of how the
//! value is derived rather than a gap in the corpus, and skipping there
//! declines to judge a pair whose answer is already known.
//!
//! This rule therefore reads [`headwater_doc::warrant_of`] rather than
//! [`headwater_doc::warrant`], and it hands over the census's own answer to
//! whether this engine wrote the file, which [`crate::scope::EdgeEnd::generated`]
//! carries. It never opens the marker itself: the predicate over
//! `headwater:generated` is `headwater_mark`'s, the census is its one caller
//! over a corpus, and a second reading here is how a rule and a census would
//! come to disagree about one file.
//!
//! `regenerated` is not `asserted`, so the pair passes. That is a pass and not
//! an exemption: the same document standing at `asserted` would be reported,
//! and a generated document cannot state `asserted` because it states nothing.
//! [#818](https://github.com/headwater-ai/headwater/issues/818) is the report,
//! and `tests/evidence_basis.rs` holds the ungenerated target beside it so that
//! the two absences keep their separate answers.
//!
//! On the source side, a value that is not `evidenced` passes rather than
//! skips: `unevidenced`, `reconstructed`, and the unfilled
//! `"{{evidenced | reconstructed | unevidenced}}"` placeholder that three
//! bundles of the base package ship are all documents that made no claim this
//! rule can refuse. The placeholder is read as one string and never split,
//! which is why a document still carrying it is a pass and not a panic.
//!
//! # The scope, and why an edge rather than a document
//!
//! The unit is [`EdgeUnit::Pair`], for [`crate::dependency`]'s reason: both
//! endpoints have to be documents, because the rule reads a value at each of
//! them. Of the declared edge halves in this repository, a little over a
//! quarter end on an anchor, and an anchor carries no warrant at all. `Pair`
//! never groups such a half, so the protection is inherited rather than written
//! here, and `tests/evidence_basis.rs` is the only thing that asserts it.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#scope--the-declaration-everything-else-rests-on)
//! fixes an edge-scoped read set at one relation instance and both endpoints,
//! so the two values this rule reads are both in the read set that keys it. An
//! edit to either end moves the key.
//!
//! # The severity is advisory, and the finding carries no patch
//!
//! The [fixability](../../../../docs/spec/12-check-layer.md#fixability) bar is
//! whether the remediation is mechanical and total. It is neither here. The
//! repair is a pointer at an artifact somebody can audit, a promotion of the
//! target by a person who reads it, or a demotion of the source's own claim to
//! `reconstructed`, and only an author knows which. `CT-WARRANT-2` is advisory
//! for that reason and it says so.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{EdgeCheck, EdgeUnit, EdgeView};
use headwater_graph::declarations::Relation;
use headwater_graph::Declarations;

pub const RULE: &str = "warrant.evidence.unsupported";

/// The relation family whose pointers carry evidential weight.
///
/// It is one of the six the meta-schema declares, and it is written here rather
/// than read from a taxonomy for [`crate::promotion::FROM`]'s reason: the
/// families are the language's and no taxonomy names a new one.
pub const FAMILY: &str = "evidence";

/// The claim on the source side that spec 3's sentence is about.
pub const CLAIMED: &str = "evidenced";

/// The warrant on the target side that does not support it. A value outside
/// [`headwater_doc::WARRANTS`] supports nothing either, and is reported with
/// its own message.
pub const UNSUPPORTING: &str = headwater_doc::ASSERTED;

/// The group carried no half at all, which the instantiation never produces.
/// Recorded rather than panicked on, for [`crate::target`]'s reason.
const NO_HALF: &str = "the entry carries no declared half";

/// The far end is not a document. `EdgeUnit::Pair` never groups one, so this is
/// unreachable rather than tolerated.
const NO_PAIR: &str = "this edge has no second document to read";

/// The check. It carries the relations of the evidence family, which the
/// taxonomy declares and this rule does not name one by one.
pub struct Basis<'a> {
    /// Every relation whose declared family is [`FAMILY`]. Empty for a taxonomy
    /// that declares no evidence relation, and then this rule generates no
    /// instance at all.
    evidential: Vec<&'a Relation>,
}

impl<'a> Basis<'a> {
    pub fn over(declarations: &'a Declarations) -> Self {
        Basis {
            evidential: declarations
                .relations
                .iter()
                .filter(|relation| relation.family.as_deref() == Some(FAMILY))
                .collect(),
        }
    }
}

impl EdgeCheck for Basis<'_> {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`]. Version 2 reads the
    /// derived warrant of a generated target, so the verdict over a pair that
    /// version 1 skipped is now a pass and a warm cache would serve the skip
    /// forever at the version it was written under.
    ///
    /// Version 3 reads a target warrant outside the closed set as supporting
    /// nothing, so the verdict over such a pair moves from a pass to a
    /// finding, and a warm cache would serve version 2's pass.
    ///
    /// Version 4 reads a relation that declares `evidence_at: from` the other
    /// way round, so the verdict over a `discharges` pair moves in both
    /// directions, and a warm cache would serve version 3's reading (#1511).
    const VERSION: u32 = 4;
    /// The Q4 pair. See the module comment: both ends have to be documents,
    /// because the rule reads a provenance member at each of them.
    const UNIT: EdgeUnit = EdgeUnit::Pair;

    /// A relation of the evidence family, and no other. A taxonomy that
    /// declares none has no pointer that carries evidential weight, and an
    /// instance over one could only ever pass.
    fn instantiates(&self, relation: &str) -> bool {
        self.evidential.iter().any(|known| known.name == relation)
    }

    fn evaluate(&self, view: &EdgeView<'_>) -> Outcome {
        // The declared half when a document wrote it, because a finding anchors
        // at the entry an author is looking at.
        let Some(half) = view.declared_half().or_else(|| view.inverse_half()) else {
            return Outcome::Skipped(NO_HALF.to_string());
        };
        let Some(relation) = self
            .evidential
            .iter()
            .find(|known| known.name == half.declared)
        else {
            return Outcome::Skipped(NO_PAIR.to_string());
        };
        let Some((source, target)) = view.ends() else {
            return Outcome::Skipped(NO_PAIR.to_string());
        };
        // Which end makes the claim and which end is the evidence is the
        // relation's declaration, and never this rule's guess (#1511).
        let at_source = relation.evidence_at_source();
        let (claimant, evidence) = if at_source {
            (target, source)
        } else {
            (source, target)
        };

        // A document the census parsed nothing for is not a document that
        // declared nothing, and the two are kept apart at both ends.
        let (Some(claimant_facets), Some(evidence_facets)) = (claimant.facets(), evidence.facets())
        else {
            return Outcome::Skipped(
                "the census parsed no document at one end of this edge, so there is no provenance \
                 to read there"
                    .to_string(),
            );
        };

        // The claimant made no claim this rule can refuse. A pass rather than a
        // skip: the rule looked at both ends and found nothing to report.
        if headwater_doc::evidence_basis(claimant_facets) != Some(CLAIMED) {
            return Outcome::Passed;
        }

        // An absent warrant is not a warrant read as supporting. A rule that
        // collapsed the two returns green over a corpus that lost every one.
        // A generated evidence document is the one absence that is not one: the
        // engine derives its warrant from the marker, and the census already
        // read it.
        let Some(warrant) = headwater_doc::warrant_of(evidence_facets, evidence.generated()) else {
            return Outcome::Skipped(format!(
                "the document at the {} end, `{}`, is the evidence and declares no warrant, so \
                 there is nothing there to say whether it supports an evidenced claim",
                if at_source { "source" } else { "target" },
                evidence.id
            ));
        };
        if warrant != UNSUPPORTING && headwater_doc::is_warrant(warrant) {
            return Outcome::Passed;
        }
        let (line, column) = at(Some(half.span));
        let closed = headwater_doc::is_warrant(warrant);

        // The words of the two directions. The `to` reading is the one this
        // rule had before a relation could declare `evidence_at`, and its
        // message and remediation are unchanged, so no finding over a
        // `traces_to` edge moves.
        let (message, remediation) = match (at_source, closed) {
            (false, false) => (
                format!(
                    "`{}` claims `{CLAIMED}` and declares `{}` to `{}`, whose warrant is \
                     `{warrant}`, which is not a value of spec 3's closed set: it says nothing \
                     about who read the target, so a pointer at it does not support the claim",
                    claimant.id, relation.name, evidence.id
                ),
                format!(
                    "correct the warrant of {} to one of the four values, point `{}` in {} at an \
                     artifact somebody can audit, or write `evidence_basis: reconstructed` in {}",
                    evidence.path, half.name, half.source.path, claimant.path
                ),
            ),
            (false, true) => (
                format!(
                    "`{}` claims `{CLAIMED}` and declares `{}` to `{}`, whose warrant is \
                     `{UNSUPPORTING}`: a document nobody has read is neither external nor \
                     auditable, so a pointer at it does not support the claim",
                    claimant.id, relation.name, evidence.id
                ),
                format!(
                    "point `{}` in {} at an artifact somebody can audit, have a person read {} \
                     and set its warrant, or write `evidence_basis: reconstructed` in {} and say \
                     what it was reconstructed from",
                    half.name, half.source.path, evidence.path, claimant.path
                ),
            ),
            (true, false) => (
                format!(
                    "`{}` claims `{CLAIMED}`, and `{}`, which `{}` makes its evidence, has the \
                     warrant `{warrant}`, which is not a value of spec 3's closed set: it says \
                     nothing about who read the evidence, so the edge does not support the claim",
                    claimant.id, evidence.id, relation.name
                ),
                format!(
                    "correct the warrant of {} to one of the four values, let an artifact \
                     somebody can audit declare `{}` to `{}` in place of `{}` in {}, or write \
                     `evidence_basis: reconstructed` in {}",
                    evidence.path,
                    relation.name,
                    claimant.id,
                    half.name,
                    half.source.path,
                    claimant.path
                ),
            ),
            (true, true) => (
                format!(
                    "`{}` claims `{CLAIMED}`, and `{}`, which `{}` makes its evidence, has the \
                     warrant `{UNSUPPORTING}`: a document nobody has read is neither external \
                     nor auditable, so the edge does not support the claim",
                    claimant.id, evidence.id, relation.name
                ),
                format!(
                    "have a person read {} and set its warrant, let an artifact somebody can \
                     audit declare `{}` to `{}` in place of `{}` in {}, or write \
                     `evidence_basis: reconstructed` in {} and say what it was reconstructed \
                     from",
                    evidence.path,
                    relation.name,
                    claimant.id,
                    half.name,
                    half.source.path,
                    claimant.path
                ),
            ),
        };

        Outcome::failed_with(Finding {
            rule: self::RULE,
            severity: Severity::Warn,
            obligation: None,
            path: half.source.path.clone(),
            line,
            column,
            message,
            remediation,
            // No patch. Which of the repairs is right is a statement about the
            // evidence that only an author can make.
            patch: None,
        })
    }
}
