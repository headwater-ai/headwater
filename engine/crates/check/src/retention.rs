// SPDX-License-Identifier: Apache-2.0
//! The third check that reads a lifecycle regime, and the first that is about
//! a document the corpus no longer holds.
//!
//! [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#lifecycle)
//! rules that "terminal states marked `retain_terminal: true` may never be
//! deleted. Lineage is the point."
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#shape)
//! declares the member on a lifecycle regime, the meta-schema types it
//! `boolean`, `taxonomy validate` refuses a value that is not one, and both
//! regimes of the base package have written `true` since 1.0.0. Nothing read
//! it. A member a corpus writes and no component reads is a promise the
//! taxonomy makes and cannot keep, and this is the component that keeps it.
//!
//! # A deletion is a fact about a change, so the grain is the corpus
//!
//! [`crate::transition`] reads one document against the version of itself that
//! stood before a change. That grain cannot reach this defect: a document the
//! change deleted has no row in the census, so no document-scoped instance is
//! generated over it, and the rule that would refuse the deletion is never
//! instantiated. So this check is corpus-grained and declares
//! `CorpusCheck::NEEDS_PRIOR`, which hands it every path the change named that
//! this corpus holds no row at, and the version of each one.
//!
//! One instance covers the whole change. A run that names no change skips it
//! with the reason spec 12 fixes, exactly as an instance of the two
//! document-scoped rules that declare the same input does.
//!
//! # A rename is not a deletion, and nothing here decides that
//!
//! The producer turns git's similarity detection on, so a renamed document
//! reaches the engine as one `prior` line naming the path the document arrived
//! at. The census holds a row there, so the entry binds, and it never reaches
//! [`crate::change::Change::departed`] at all. A document moved out of the
//! corpus root does reach it, and that is a departure by every reading this
//! engine has: no shelf places it, no rule runs over it, and no index names it.
//!
//! The trust boundary is the producer's, and [`crate::change`] states it: a
//! caller that turned similarity detection off would present a rename as a
//! deletion and an addition, and this rule would refuse the deletion. That is
//! the honest failure. The alternative is an engine that walks history, which
//! spec 12 rules out as an input.
//!
//! # What it reads out of the declaration, and the three answers
//!
//! The kind comes from the path and the prior front matter, through the same
//! resolver the census used, so a departed document is classified by the
//! declaration rather than by a guess. The kind binds a lifecycle regime, and
//! the regime answers three ways.
//!
//! - `retain_terminal: true` — a document standing at a terminal state of this
//!   regime is kept, and a change that deletes one is refused.
//! - `retain_terminal: false` — the regime has ruled that the deletion is
//!   permitted, and this reports nothing.
//! - the member is absent — the regime has said nothing, and this reports
//!   nothing either.
//!
//! The last two produce one behavior and they are not one fact.
//! [`crate::shape::LifecycleRegime::retain_terminal`] keeps them apart at the
//! parse, so the day a rule has something different to say about silence, the
//! difference is still there to read. No regime in this repository declares
//! `false`; the fixture taxonomy under `engine/crates/check/fixtures/` does,
//! and it is what holds that arm to a behavior.
//!
//! # Terminal is read off the machine and never off a list
//!
//! A state is terminal when the regime names it and it reaches nothing, which
//! is [`crate::shape::LifecycleRegime::terminal`] and the same reading
//! [`crate::transition`] takes. `regimes.lifecycle` also declares a `terminal`
//! member, and nothing reads it: a second list would be a second definition of
//! the same fact, free to disagree with the machine beside it. That gap is
//! recorded rather than filled here.
//!
//! # The severity is an error, and the finding carries no patch
//!
//! The remedy is mechanical and total: put the file back. That is the
//! [fixability](../../../../docs/spec/12-check-layer.md#fixability) bar, so the
//! rule is an error and the commit gate refuses the change. Nothing is written,
//! because this engine holds the prior front matter and not the prior body: a
//! patch that restored the file from what a check received would restore half
//! of it.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::lifecycle_state::{StateFacet, Stood};
use crate::scope::{CorpusCheck, CorpusView};
use crate::shape::Shape;
use headwater_census::shelves::Taxonomy;

pub const RULE: &str = "lifecycle.deletion.not_permitted";

/// The check. It carries the taxonomy because a departed path has no census
/// row to read a kind off, and the shape because the regime it holds the
/// document to is declared rather than known.
pub struct Retention<'a> {
    taxonomy: &'a Taxonomy,
    shape: &'a Shape,
    /// The facet in the `state` role and the values it admits, read by the one
    /// component that owns that reading, as [`crate::transition`] does.
    facet: StateFacet,
}

impl<'a> Retention<'a> {
    pub fn over(taxonomy: &'a Taxonomy, shape: &'a Shape) -> Self {
        Retention {
            taxonomy,
            shape,
            facet: StateFacet::of(shape),
        }
    }

    /// What one departed path is, or nothing where this rule decides nothing
    /// about it.
    ///
    /// Every `None` below is a path this rule has no jurisdiction over rather
    /// than a document it cleared: a file no shelf places, a kind that binds no
    /// lifecycle regime, a regime that permits the deletion or says nothing, a
    /// version that declared no state, and a state the machine calls anything
    /// but terminal. A run that names such a path still reports it, because
    /// the change block names every path that reached no row of this corpus.
    fn finding(&self, path: &str, facets: &headwater_yaml::Mapping) -> Option<Finding> {
        let kind = headwater_census::resolve::resolve(path, facets, self.taxonomy)
            .kind()?
            .to_string();
        let regime = self.shape.lifecycle_of(&kind)?;
        // The three answers, and only the first one is this rule's business.
        // `false` and absent produce one behavior here and stay two facts in
        // the shape, on the terms the module comment states.
        if regime.retain_terminal != Some(true) {
            return None;
        }
        let Stood::At(state) = self.facet.stood(facets) else {
            return None;
        };
        if !regime.terminal(state) {
            return None;
        }
        let name = facets
            .get("id")
            .and_then(|node| node.value.as_scalar())
            .map(|scalar| scalar.text.clone())
            .unwrap_or_else(|| path.to_string());
        let (line, column) = at(None);
        Some(Finding {
            rule: RULE,
            severity: Severity::Error,
            obligation: None,
            path: path.to_string(),
            line,
            column,
            message: format!(
                "{name} stood at `{state}`, which is a terminal state of the lifecycle regime \
                 `{}`, and that regime declares `retain_terminal: true`. This change leaves no \
                 document at `{path}`",
                regime.name
            ),
            remediation: format!(
                "Restore `{path}`. A document that reached the end of `{}` is kept as the record \
                 of what it said, and the reciprocal half of a succession relation is what names \
                 its successor. To retire the record rather than the document, declare \
                 `retain_terminal: false` on that regime and say why",
                regime.name
            ),
            patch: None,
        })
    }
}

impl CorpusCheck for Retention<'_> {
    const RULE: &'static str = self::RULE;
    /// The first edition of this rule.
    const VERSION: u32 = 1;
    /// The paths the change named that this corpus holds no row at, which is
    /// the only reading of a departed document available to a check.
    const NEEDS_PRIOR: bool = true;

    fn evaluate(&self, view: &CorpusView<'_>) -> Outcome {
        // A taxonomy that declares no facet in the `state` role has no state to
        // read, so no departure can be held to a terminal one. It is a skip
        // rather than a pass, because a corpus-grained rule has one instance
        // and a pass over it would report the whole change as clean.
        if self.facet.name.is_none() {
            return Outcome::Skipped(NO_STATE_FACET.to_string());
        }
        Outcome::failed(
            view.departed()
                .iter()
                .filter_map(|entry| self.finding(entry.path, entry.facets))
                .collect(),
        )
    }
}

/// The reason a corpus with no state facet carries. See [`Retention::evaluate`].
const NO_STATE_FACET: &str =
    "this taxonomy declares no facet in the `state` role, so no document stands anywhere and no \
     departure can be held to a terminal state";
