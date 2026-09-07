// SPDX-License-Identifier: Apache-2.0
//! A migration payload, accounted against what this corpus measured.
//!
//! [Spec 7](../../../../docs/spec/07-distribution-and-federation.md#upgrading)
//! asks the upgrade report for "which migration steps apply, split into
//! mechanical and judgment-bearing". This module is that half of the report, and
//! it is the half that turns a step's claim into a measurement.
//!
//! # What makes a step's remedy true rather than asserted
//!
//! A step declares nothing about which dimension it remedies. The subject fixes
//! it ([`headwater_resolve::migration::Subject::remedies`]), and what this
//! module adds is the other end: the documents of *this* corpus that the step
//! names, read off the census, beside the documents that the measurement
//! reports as no longer valid. A payload whose steps name every document that
//! moved has been checked against a corpus its publisher never saw. A payload
//! that leaves documents unaccounted says so by name.
//!
//! # An empty subject list is two different states, and they are told apart
//!
//! A step can name no document of this corpus because the value it renames is
//! one nobody here uses, or because this repository's overlay already removed
//! it. [`Accounted::declared`] is the difference. The second is why a step whose
//! source this repository does not hold is reported and never refused: the
//! consumer's old taxonomy is the base under its own overlays, so a step that is
//! right for the publisher can be vacuous here.
//!
//! # Two taxonomies, and the source half is held against both
//!
//! [`Accounted::declared`] reads the taxonomy this repository *takes*, which is
//! the one every `from` of the payload was written against.
//! [`Accounted::stands`] reads the other one: the candidate, resolved under this
//! repository's own selection and overlays, which is the taxonomy an adopter
//! holds after the upgrade. A step whose `from` that taxonomy still declares is
//! a step that did not happen for this consumer.
//!
//! `taxonomy publish` may not refuse such a step and this end must
//! ([#388](https://github.com/headwater-ai/headwater/issues/388)). A bundle is
//! add-only, so a value the base declares is declared under every selection, and
//! a value only a bundle declares moved for the consumers who selected that
//! bundle and for no other. The publisher holds one taxonomy and cannot state
//! the condition; this end holds both and reads it.

use headwater_census::census::Census;
use headwater_resolve::migration::{declares, Payload, Step, Subject};
use headwater_resolve::{Adopted, Resolution};
use std::collections::BTreeSet;

/// The sentence a report states about a step whose source the taxonomy this
/// artifact gives this repository still declares.
///
/// Here rather than at each caller, because `taxonomy diff` and
/// `taxonomy migrate` both say it and two spellings of one condition would read
/// as two conditions.
pub const STANDS: &str = "the taxonomy this artifact gives this repository still declares the old \
                          value, so this step does not complete here";

/// Whether the taxonomy this artifact gives *this* consumer still declares the
/// value a step says moved.
///
/// Two readings and not one, and the second was measured rather than reasoned
/// to. [`headwater_resolve::migration::declares`] over the resolved candidate is
/// the first, and alone it is wrong in one direction that four committed cases
/// met at once: an `add` addressed *below* a key the new base no longer declares
/// grafts that key back into the tree, so a rename the publisher really
/// performed reads as a value that still stands. `kinds.decision.language` in an
/// adopter's overlay does exactly that to a base that renamed `decision`.
///
/// That graft is [`headwater_resolve::Founding`], the report measures it on the
/// `addressability` line, and an `overlay_address` step is the remedy for it. It
/// is a different condition from this one, so a value that no source declares
/// and an address merely created does not stand.
pub fn stands(candidate: &Resolution, subject: &Subject, from: &str) -> bool {
    if !declares(&candidate.taxonomy, subject, from) {
        return false;
    }
    // The key that holds the value. A facet value lives in a `values` list that
    // no address founds, so the key is the facet's own declaration: an overlay
    // that grafts `facets.status` back is the same graft over the same reading.
    let held = match subject {
        Subject::Kind => format!("kinds.{from}"),
        Subject::FacetValue { facet } => format!("facets.{facet}"),
        Subject::OverlayAddress => from.to_string(),
    };
    !candidate.founded.iter().any(|founding| {
        held == founding.founds || held.starts_with(&format!("{}.", founding.founds))
    })
}

/// One payload, against one corpus.
#[derive(Clone, Debug)]
pub struct Accounting {
    pub at: String,
    pub from: String,
    pub to: String,
    pub steps: Vec<Accounted>,
    /// Every document that a dimension reports as moved and that no step of
    /// this payload names.
    pub unaccounted: Vec<String>,
    /// How many documents moved in all, which is the denominator of the line
    /// above.
    pub moved: usize,
}

/// One step, against one corpus.
#[derive(Clone, Debug)]
pub struct Accounted {
    /// What the step moves. The noun a report counts subjects in is fixed by
    /// it, and a second table of nouns beside the enum would be a place for the
    /// two to disagree.
    pub subject: Subject,
    /// How the payload names the step.
    pub at: String,
    pub how: String,
    pub remedies: &'static [&'static str],
    /// The documents of this corpus the step names.
    pub subjects: BTreeSet<String>,
    /// Whether the taxonomy this repository takes declares the value at all.
    pub declared: bool,
    /// Whether the taxonomy this artifact gives *this* repository still
    /// declares the value. See the module comment.
    pub stands: bool,
    pub task: Option<String>,
    pub because: String,
}

/// Account one payload against one corpus.
///
/// `moved` is the union of the sets that [`crate::classification`] and
/// [`crate::instance_validity`] answered with. Those two are the dimensions
/// that a subject names, and both answer at the grain of one document. So this
/// function performs no comparison of its own and cannot disagree with the
/// dimensions it prints under.
pub fn account(
    payload: &Payload,
    census: &Census,
    overlay: &Adopted,
    declares: impl Fn(&Step) -> bool,
    stands: impl Fn(&Step) -> bool,
    moved: &BTreeSet<String>,
) -> Accounting {
    let steps: Vec<Accounted> = payload
        .steps
        .iter()
        .map(|step| Accounted {
            subject: step.subject.clone(),
            at: step.at(),
            how: step.apply.sentence(),
            remedies: step.subject.remedies(),
            subjects: crate::subjects(step, census, overlay),
            declared: declares(step),
            stands: stands(step),
            task: step.apply.task().map(str::to_string),
            because: step.because.clone(),
        })
        .collect();

    // Only the document subjects, because `moved` is a set of document paths
    // and an overlay entry is not one. A key that could never match would not
    // change the answer, and it would make the denominator below read as
    // though it counted two kinds of thing.
    let named: BTreeSet<&String> = steps
        .iter()
        .filter(|step| !step.subject.addressed())
        .flat_map(|step| step.subjects.iter())
        .collect();
    Accounting {
        at: payload.at.clone(),
        from: payload.from.clone(),
        to: payload.to.clone(),
        unaccounted: moved
            .iter()
            .filter(|path| !named.contains(path))
            .cloned()
            .collect(),
        moved: moved.len(),
        steps,
    }
}

impl Accounting {
    /// How many steps the engine applies with no judgment.
    pub fn mechanical(&self) -> usize {
        self.steps.iter().filter(|step| step.task.is_none()).count()
    }

    /// Whether every document that moved lies under a step.
    pub fn complete(&self) -> bool {
        self.unaccounted.is_empty()
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "\nmigration payload  {}, from {} to {}\n",
            self.at, self.from, self.to
        ));
        out.push_str(&format!(
            "  {} step{}, {} mechanical and {} judgment-bearing\n",
            self.steps.len(),
            match self.steps.len() {
                1 => "",
                _ => "s",
            },
            self.mechanical(),
            self.steps.len() - self.mechanical()
        ));
        for step in &self.steps {
            out.push_str(&format!("\n  {}\n", step.at));
            out.push_str(&format!("    {}\n", step.how));
            out.push_str(&format!("    remedies {}\n", step.remedies.join(", ")));
            out.push_str(&format!("    {}\n", step.reach()));
            if let Some(task) = &step.task {
                out.push_str(&format!("    task  {task}\n"));
            }
            out.push_str(&format!("    why   {}\n", step.because));
        }
        out.push('\n');
        match self.complete() {
            true => out.push_str(&format!(
                "  {} of the {} documents a dimension reports as moved lie under a step of \
                 this payload\n",
                self.moved - self.unaccounted.len(),
                self.moved
            )),
            false => {
                out.push_str(&format!(
                    "  {} of the {} documents a dimension reports as moved lie under no step \
                     of this payload\n",
                    self.unaccounted.len(),
                    self.moved
                ));
                for path in &self.unaccounted {
                    out.push_str(&format!("    {path}\n"));
                }
            }
        }
        out
    }
}

impl Accounted {
    /// What this step reaches in this corpus, as one sentence.
    ///
    /// Four sentences rather than a count, because zero has two meanings and a
    /// consumer acts differently on each. See the module comment.
    ///
    /// The vacuous arm stays first. A step whose source this repository never
    /// held migrates nothing here whatever the artifact declares, and that is a
    /// weaker statement than the one below it rather than a competing one.
    /// [`Accounted::stands`] comes next, ahead of both counts, because a step
    /// that did not happen for this consumer is what a reader acts on first: the
    /// count of documents under it is the size of the damage rather than the
    /// reason.
    pub fn reach(&self) -> String {
        match (self.declared, self.stands, self.subjects.len()) {
            (false, _, _) => "the taxonomy this repository takes declares no such value, so this \
                              step migrates nothing here"
                .to_string(),
            (true, true, 0) => STANDS.to_string(),
            (true, true, count) => format!("{STANDS}, and {}", self.subject.reached(count)),
            (true, false, 0) => self.subject.reached_nothing().to_string(),
            (true, false, count) => self.subject.reached(count),
        }
    }
}
