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

use headwater_census::census::Census;
use headwater_resolve::migration::{Payload, Step};
use std::collections::BTreeSet;

/// One payload, against one corpus.
#[derive(Clone, Debug)]
pub struct Accounting {
    pub at: String,
    pub from: String,
    pub to: String,
    pub steps: Vec<Accounted>,
    /// Every document whose document-grained verdict moved and which no step of
    /// this payload names.
    pub unaccounted: Vec<String>,
    /// How many documents moved in all, which is the denominator of the line
    /// above.
    pub moved: usize,
}

/// One step, against one corpus.
#[derive(Clone, Debug)]
pub struct Accounted {
    /// How the payload names the step.
    pub at: String,
    pub how: String,
    pub remedies: &'static [&'static str],
    /// The documents of this corpus the step names.
    pub subjects: BTreeSet<String>,
    /// Whether the taxonomy this repository takes declares the value at all.
    pub declared: bool,
    pub task: Option<String>,
    pub because: String,
}

/// Account one payload against one corpus.
///
/// `moved` is the set that [`crate::instance_validity`] answered with, so this
/// function performs no comparison of its own and cannot disagree with the
/// dimension it reports beside.
pub fn account(
    payload: &Payload,
    census: &Census,
    declares: impl Fn(&Step) -> bool,
    moved: &BTreeSet<String>,
) -> Accounting {
    let steps: Vec<Accounted> = payload
        .steps
        .iter()
        .map(|step| Accounted {
            at: step.at(),
            how: step.apply.sentence(),
            remedies: step.subject.remedies(),
            subjects: crate::subjects(step, census),
            declared: declares(step),
            task: step.apply.task().map(str::to_string),
            because: step.because.clone(),
        })
        .collect();

    let named: BTreeSet<&String> = steps
        .iter()
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
        self.steps
            .iter()
            .filter(|step| step.task.is_none())
            .count()
    }

    /// Whether every document whose validity moved lies under a step.
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
                "  {} of the {} documents whose validity moved lie under a step of this payload\n",
                self.moved - self.unaccounted.len(),
                self.moved
            )),
            false => {
                out.push_str(&format!(
                    "  {} of the {} documents whose validity moved lie under no step of this \
                     payload\n",
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
    /// Three sentences rather than a count, because zero has two meanings and a
    /// consumer acts differently on each. See the module comment.
    pub fn reach(&self) -> String {
        match (self.declared, self.subjects.len()) {
            (false, _) => "the taxonomy this repository takes declares no such value, so this \
                           step migrates nothing here"
                .to_string(),
            (true, 0) => {
                "no document of this corpus carries the old value, so this step is a no-op here"
                    .to_string()
            }
            (true, 1) => "1 document of this corpus carries the old value".to_string(),
            (true, count) => format!("{count} documents of this corpus carry the old value"),
        }
    }
}
