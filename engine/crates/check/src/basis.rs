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
//! This repository's own lock resolves that family to eight relations —
//! `traces_to`, `applied_in`, `assesses`, `cites_evidence`, `discharges`,
//! `examines`, `records` and `verified_by`. A rule that held `traces_to` alone
//! would read one of the eight, which is the mistake that halves the
//! population, and `tests/evidence_basis.rs` writes one edge of each so that no
//! later change can make it quietly.
//!
//! # Which end carries which question
//!
//! The claim is the **source's**: `evidence_basis` says what the citing
//! document rests on. The warrant is the **target's**: it says whether anybody
//! has read the thing being rested on. A rule that took either question to the
//! wrong end reads a corpus that is almost always green, because most documents
//! are `accepted` and most claim `evidenced`.
//!
//! The direction comes from the relation and never from the file that wrote the
//! entry, which is what [`crate::scope::EdgeView::ends`] normalizes once for
//! every edge-grained rule.
//!
//! # Three values that are not `asserted`, and one that is not `evidenced`
//!
//! `proposed` is a warrant this corpus writes and spec 3's closed set does not
//! name;
//! [HW-OBL-0125](../../../../docs/obligations/0125-nine-documents-state-a-warrant-the-closed-set-does-not-hold-and-no-check-reads-one.md)
//! holds the nine documents that do. It is not `asserted`, so this rule passes
//! it, and that is a decision rather than an accident: the defect there is the
//! value itself, and this rule is not the one that reports it.
//!
//! A target that declares **no** warrant is a different answer again, and it
//! skips. A rule that collapsed the absence into "not `asserted`" would return
//! green over a corpus that had lost every warrant it declares.
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

/// The warrant on the target side that does not support it.
pub const UNSUPPORTING: &str = "asserted";

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
    /// See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 1;
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

        // A document the census parsed nothing for is not a document that
        // declared nothing, and the two are kept apart at both ends.
        let (Some(source_facets), Some(target_facets)) = (source.facets(), target.facets()) else {
            return Outcome::Skipped(
                "the census parsed no document at one end of this edge, so there is no provenance \
                 to read there"
                    .to_string(),
            );
        };

        // The source made no claim this rule can refuse. A pass rather than a
        // skip: the rule looked at both ends and found nothing to report.
        if headwater_doc::evidence_basis(source_facets) != Some(CLAIMED) {
            return Outcome::Passed;
        }

        // An absent warrant is not a warrant read as supporting. A rule that
        // collapsed the two returns green over a corpus that lost every one.
        let Some(warrant) = headwater_doc::warrant(target_facets) else {
            return Outcome::Skipped(format!(
                "the document at the target end, `{}`, declares no warrant, so there is nothing \
                 there to say whether it supports an evidenced claim",
                target.id
            ));
        };
        if warrant != UNSUPPORTING {
            return Outcome::Passed;
        }

        let (line, column) = at(Some(half.span));
        Outcome::failed_with(Finding {
            rule: self::RULE,
            severity: Severity::Warn,
            obligation: None,
            path: half.source.path.clone(),
            line,
            column,
            message: format!(
                "`{}` claims `{CLAIMED}` and declares `{}` to `{}`, whose warrant is \
                 `{UNSUPPORTING}`: a document nobody has read is neither external nor auditable, \
                 so a pointer at it does not support the claim",
                source.id, relation.name, target.id
            ),
            remediation: format!(
                "point `{}` in {} at an artifact somebody can audit, have a person read {} and \
                 set its warrant, or write `evidence_basis: reconstructed` in {} and say what it \
                 was reconstructed from",
                half.name, half.source.path, target.path, source.path
            ),
            // No patch. Which of the three repairs is right is a statement
            // about the evidence that only an author can make.
            patch: None,
        })
    }
}
