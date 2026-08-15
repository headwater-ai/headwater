// SPDX-License-Identifier: Apache-2.0
//! The first check that declares `needs_prior`: a warrant promoted in one
//! change.
//!
//! [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#promotion-is-one-human-one-document-one-diff)
//! states the act and the failure mode in four sentences. "An `asserted`
//! document becomes `accepted` when a person reads it, sets the warrant, and
//! names themselves." Then: "The failure mode is bulk. A script that stamps
//! forty documents writes bytes that are identical to forty real acceptances,
//! and no check can separate them. So the engine does not try to tell one stamp
//! from another. What it can do is count them in a change."
//!
//! # Why a standing population cannot answer this and a change can
//!
//! `taxonomy audit` reports how many documents stand at `asserted`. That figure
//! is a stock rather than a flow, and a bulk stamp lowers it by exactly what
//! forty separate acceptances would. So it answers whether anything is ever
//! promoted and it cannot answer whether one change was a review.
//!
//! A count per change is a different reading, and it needs a different input.
//! One value of a facet cannot say how it was reached, so this check reads the
//! version of the document that stood before the change
//! ([`crate::change::Prior`]). That input is available only in change-scoped
//! evaluation, so every instance of this rule in a full-corpus run is reported
//! as skipped with the reason `change-scoped-only`, which is what
//! [`crate::scope`] does before a view exists.
//!
//! # What separates a promotion from a document that is merely accepted
//!
//! Three states of one document are `accepted` now, and only one of them is a
//! promotion.
//!
//! | before the change | now | this rule |
//! |---|---|---|
//! | `asserted` | `accepted` | a promotion, counted |
//! | nothing: the change adds the document | `accepted` | drafted straight to `accepted`, and no transition |
//! | `accepted` | `accepted` | no movement |
//!
//! A rule that reported every document now standing at `accepted` looks
//! identical to this one in a report over a corpus where every acceptance was a
//! promotion. `tests/promotion.rs` is the corpus that separates them, and it is
//! the failing fixture
//! [spec 12](../../../../docs/spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship)
//! requires.
//!
//! # The severity is `info`, and no bar exists to raise it
//!
//! Spec 3: "Forty in one change is a finding about the review rather than about
//! the documents. The posture is advisory permanently... A threshold teaches
//! people to promote in smaller batches, and nothing declares one." So one
//! promotion is not a defect and this rule reports it at `info`. The count is
//! the reading, and the report states it.
//!
//! [HW-OBL-0119](../../../../docs/obligations/0119-an-audit-reading-carries-no-declared-bar-so-a-distribution-cannot-become-a-finding.md)
//! holds the missing bar. Nothing in a taxonomy says how many promotions in one
//! change is too many, so a number invented here would be a verdict this engine
//! derived from nothing a corpus declared.
//!
//! # It reads a warrant, and it is the first check that does
//!
//! Every other reader of a warrant is a report: the shelf index, `query
//! explain`, and the warrant reading of `taxonomy audit`. This rule reads one
//! through [`headwater_doc::warrant`] rather than by opening the provenance
//! block by key, because a second reader of that block is a second answer to
//! what a warrant is.

use crate::change::Prior;
use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView};
use headwater_doc::{PROVENANCE, WARRANT};
use headwater_yaml::Mapping;

pub const RULE: &str = "warrant.promoted";

/// The warrant a promotion moves from, and the one it moves to.
///
/// Both are values of the closed set that
/// [spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#the-warrant-and-what-each-value-requires)
/// declares. They are written here rather than read from a declaration because
/// the promotion act is spec 3's rather than a taxonomy's: no member of the
/// language names a warrant, and `taxonomy audit` writes the same closed set out
/// for the same reason.
pub const FROM: &str = "asserted";
pub const TO: &str = "accepted";

/// The check. It carries no state: the transition it reads is spec 3's, and
/// nothing in a taxonomy parameterizes it.
pub struct Promoted;

impl DocumentCheck for Promoted {
    const RULE: &'static str = RULE;
    const VERSION: u32 = 1;
    const NEEDS_PRIOR: bool = true;

    /// Every kind. A warrant is engine-owned rather than declared, so no
    /// taxonomy says which kinds carry one, and a kind left out here would be a
    /// kind whose promotions nothing counts.
    fn instantiates(&self, kind: &str) -> bool {
        let _ = kind;
        true
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        // Unreachable for this check: the runner skips an instance that has no
        // prior version to bind, so a view is never built without one. It is a
        // pass rather than a panic for the reason no check panics: one row must
        // not silence the rest of the corpus.
        let Some(prior) = view.prior() else {
            return Outcome::Passed;
        };
        let Prior::Committed { facets, .. } = prior else {
            // Added, or not carried by the change. Neither is a movement, and
            // the first is the case a count of accepted documents gets wrong.
            return Outcome::Passed;
        };
        if headwater_doc::warrant(facets) != Some(FROM) {
            return Outcome::Passed;
        }
        if headwater_doc::warrant(view.facets()) != Some(TO) {
            return Outcome::Passed;
        }

        let (line, column) = at(warrant_span(view.facets()));
        Outcome::Failed(vec![Finding {
            rule: RULE,
            severity: Severity::Info,
            obligation: None,
            path: view.path().to_string(),
            line,
            column,
            message: format!(
                "the warrant moved from `{FROM}` to `{TO}` in this change, which is a promotion"
            ),
            remediation:
                ("nothing, for one. Spec 3 makes promotion one human, one document, one diff, and \
                 the reading is the count of this rule over the change rather than any one line \
                 of it. Nothing declares how many promotions in one change is too many, which \
                 HW-OBL-0119 holds")
                    .to_string(),
            // No patch. There is nothing to correct: this rule reports an act
            // rather than a defect, and spec 12's fixability bar is about a
            // remediation this engine can write.
            patch: None,
        }])
    }
}

/// The span of the `warrant` key, so a report points at the line a person
/// wrote rather than at the top of the file.
///
/// The provenance block is the fallback, because a block whose shape this
/// engine cannot read is still the right place to send a reader.
/// `facet.value.not_permitted` is the rule that reports the shape.
fn warrant_span(facets: &Mapping) -> Option<headwater_yaml::Span> {
    facets
        .get(PROVENANCE)?
        .value
        .as_map()
        .and_then(|block| block.key_span(WARRANT))
        .or_else(|| facets.key_span(PROVENANCE))
}
