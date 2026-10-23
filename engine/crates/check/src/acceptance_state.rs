// SPDX-License-Identifier: Apache-2.0
//! A document that stands at the initial state of its regime and says a person
//! accepted it (#1832).
//!
//! [HW-DR-0052](../../../../docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md)
//! rules that an author writes the state the document will hold once it
//! merges, and that the initial state says the document is not proposed as
//! finished. An `accepted` warrant says that a person read the document as
//! finished ([spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#the-warrant-and-what-each-value-requires)).
//! A document that says both contradicts itself. Before this rule, nothing read
//! the two facets together: [`crate::warrant`] reads the provenance block
//! alone, and [`crate::lifecycle_state`] reads the state alone.
//!
//! # What "initial" means here
//!
//! The initial state is the one the regime of the document's kind declares in
//! `initial`, through [`crate::shape::Shape::lifecycle_of`]. It is not the
//! value that carries `role: initial` in the vocabulary, and it is not the
//! literal `draft`. The two readings agree in this repository and differ for a
//! regime that opens somewhere else, and the regime is what the kind binds.
//!
//! # It disagrees with `warrant.promoted`, and that is the point
//!
//! [`crate::promotion`] is change-scoped, and it passes a document that the
//! change adds, because an added document made no movement. So a document
//! added at the initial state with an `accepted` warrant passes that rule. This
//! rule reads the tree as it stands, so it reports that document on a branch as
//! well as on `main`.
//!
//! # What it skips
//!
//! An absent warrant, and a warrant outside the closed set, skip with a
//! reason: [`crate::warrant`] and
//! [HW-OBL-0125](../../../../docs/obligations/0125-nine-documents-state-a-warrant-outside-the-closed-set-and-only-a-person-can-set-the-value.md)
//! hold them. A document that writes no state, or a value the state facet
//! does not admit, or a state its regime never names, skips too, because the
//! facet rules and `lifecycle.state.not_admitted` report those. A generated
//! file gets no instance, through [`crate::scope::over_documents`].
//!
//! # Advisory, with no patch
//!
//! The remedy has two directions. Either the document is finished and its
//! state should be the one it will hold, or nobody accepted it and the warrant
//! should be `asserted`. Only the author knows which is true, so the finding is
//! a warning and carries no patch.
//!
//! # What it does not check
//!
//! It reads that the warrant is `accepted`, and never who wrote the acceptor.
//! [HW-OBL-0108](../../../../docs/obligations/0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md)
//! records that an agent writes every acceptance stamp in this corpus, and a
//! rule that reads a name cannot tell who wrote it.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::lifecycle_state::{StateFacet, Stood};
use crate::scope::{DocumentCheck, DocumentView};
use crate::shape::Shape;
use headwater_doc::{ACCEPTED, ACCEPTED_BY, WARRANT, WARRANTS};

/// The rule.
pub const RULE: &str = "warrant.acceptance.at_initial";

/// The check. It carries the shape, because the initial state is the regime's
/// declaration.
pub struct AtInitial<'a> {
    shape: &'a Shape,
    facet: StateFacet,
}

impl<'a> AtInitial<'a> {
    pub fn over(shape: &'a Shape) -> Self {
        AtInitial {
            shape,
            facet: StateFacet::of(shape),
        }
    }
}

impl DocumentCheck for AtInitial<'_> {
    const RULE: &'static str = self::RULE;
    /// The first edition. See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 1;

    /// A kind that binds a lifecycle regime, in a taxonomy with a state facet.
    /// The same set [`crate::lifecycle_state::StateAdmitted`] instantiates
    /// over: a kind with no regime has no initial state to stand at.
    fn instantiates(&self, kind: &str) -> bool {
        self.facet.name.is_some() && self.shape.lifecycle_of(kind).is_some()
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let Some(regime) = self.shape.lifecycle_of(view.kind()) else {
            return Outcome::Passed;
        };
        let facet = self.facet.name.as_deref().unwrap_or_default();
        let state = match self.facet.stood(view.facets()) {
            Stood::At(state) => state,
            Stood::Undeclared => {
                return Outcome::Skipped(format!(
                    "this document declares no value for `{facet}`, so there is no state to read \
                     beside the warrant"
                ))
            }
            Stood::NotAState(value) => {
                return Outcome::Skipped(format!(
                    "this document stands at `{value}`, which `{facet}` does not admit, and \
                     `facet.value.not_permitted` reports that"
                ))
            }
        };
        if !regime.states().contains(&state) {
            return Outcome::Skipped(format!(
                "this document stands at `{state}`, which the lifecycle regime `{}` never names, \
                 and `lifecycle.state.not_admitted` reports that",
                regime.name
            ));
        }
        let warrant = match headwater_doc::warrant(view.facets()) {
            None => {
                return Outcome::Skipped(
                    "this document declares no warrant, so there is no acceptance to read beside \
                     the state"
                        .to_string(),
                )
            }
            Some(warrant) if !WARRANTS.contains(&warrant) => {
                return Outcome::Skipped(format!(
                    "the warrant `{warrant}` is outside the closed set, and \
                     `warrant.value.not_permitted` reports that"
                ))
            }
            Some(warrant) => warrant,
        };
        if state != regime.initial || warrant != ACCEPTED {
            return Outcome::Passed;
        }

        // The anchor is the state facet's key, because the state is the half
        // that HW-DR-0052 names first as the likely error.
        let (line, column) = at(view.facets().key_span(facet));
        Outcome::failed_with(Finding {
            rule: self::RULE,
            severity: Severity::Warn,
            obligation: None,
            path: view.path().to_string(),
            line,
            column,
            message: format!(
                "`{facet}` is `{state}`, the initial state of regime `{}`, and \
                 `provenance.{WARRANT}` is `{ACCEPTED}`: the state says the document is not \
                 proposed as finished, and the warrant says a person accepted it as finished",
                regime.name
            ),
            remediation: format!(
                "in {}, set `{facet}` to the state the document will hold once it merges, or \
                 withdraw the acceptance: set `provenance.{WARRANT}` to `asserted` and remove \
                 `provenance.{ACCEPTED_BY}`",
                view.path()
            ),
            // No patch. Which of the two halves is wrong is a statement about
            // the document that only its author can make.
            patch: None,
        })
    }
}
