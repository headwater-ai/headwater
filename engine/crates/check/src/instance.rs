// SPDX-License-Identifier: Apache-2.0
//! An instance: one check, applied to one target, with one outcome.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#instances-and-why-coverage-needs-them)
//! says a check is a template and the engine instantiates it per target, and
//! that coverage then follows with no added mechanism: "each run records, per
//! document, which instances were created, which ran, which were served from
//! cache, and which were skipped with a reason."
//!
//! Three of those four are here. Nothing is served from a cache, because this
//! runner has none ([#55](https://github.com/headwater-ai/headwater/issues/55)
//! owns the cache), and a class that no instance can be in is left out rather
//! than printed as a standing zero.
//!
//! # An instance is accounted to every document it read
//!
//! Coverage is per document, and an instance of a Graph-origin check has an
//! edge for a target rather than a document. Spec 12 fixes what an edge-scoped
//! instance reads: "one relation instance **and both endpoints**". So an
//! instance names every document it read, and coverage counts it against each
//! of them.
//!
//! Attributing it to one end instead would be the silent pass one level up. A
//! document at the receiving end of a reciprocity pair *is* checked — the rule
//! reads it, and it fires when that document is the one missing its half — and
//! a coverage report that called it unchecked would send its author to look for
//! a shelf pattern that is already right.
//!
//! A finding carries its own path, which is a different question: what a check
//! read, and where the author who can act on it is looking.

use crate::finding::Finding;

/// One check, applied to one target.
#[derive(Clone, Debug)]
pub struct Instance {
    pub rule: &'static str,
    /// Every document this instance read, relative to the repository root, in
    /// a fixed order and without repeats.
    pub reads: Vec<String>,
    pub outcome: Outcome,
}

/// What became of one instance. Closed, and matched exhaustively.
#[derive(Clone, Debug)]
pub enum Outcome {
    /// It ran and found nothing.
    Passed,
    /// It ran and found something.
    Failed(Box<Finding>),
    /// It did not run, and the reason is visible rather than silent
    /// ([spec 4](../../../../docs/spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)).
    Skipped(&'static str),
}

impl Instance {
    /// One instance, from what the view carried and what the check made of it.
    ///
    /// [`crate::scope`] is the only caller, and that is the point: the read
    /// set comes from the view rather than from the check, so a check cannot
    /// record that it read less than it was handed.
    pub fn of(rule: &'static str, reads: impl Reads, outcome: Outcome) -> Self {
        Instance {
            rule,
            reads: reads.paths(),
            outcome,
        }
    }

    pub fn passed(rule: &'static str, reads: impl Reads) -> Self {
        Instance {
            rule,
            reads: reads.paths(),
            outcome: Outcome::Passed,
        }
    }

    pub fn skipped(rule: &'static str, reads: impl Reads, reason: &'static str) -> Self {
        Instance {
            rule,
            reads: reads.paths(),
            outcome: Outcome::Skipped(reason),
        }
    }

    pub fn failed(rule: &'static str, reads: impl Reads, finding: Finding) -> Self {
        Instance {
            rule,
            reads: reads.paths(),
            outcome: Outcome::Failed(Box::new(finding)),
        }
    }

    pub fn ran(&self) -> bool {
        !matches!(self.outcome, Outcome::Skipped(_))
    }

    pub fn finding(&self) -> Option<&Finding> {
        match &self.outcome {
            Outcome::Failed(finding) => Some(finding),
            _ => None,
        }
    }

    /// The first document this instance read, which is the one a report sorts
    /// and groups by.
    pub fn at(&self) -> &str {
        self.reads.first().map(String::as_str).unwrap_or_default()
    }
}

/// What a check hands over as the set of documents an instance read.
///
/// One path, or several. The trait exists so that a Document-scoped check
/// writes one string and an Edge-scoped one writes two, and neither writes a
/// `vec![]` at every call site.
pub trait Reads {
    fn paths(self) -> Vec<String>;
}

impl Reads for String {
    fn paths(self) -> Vec<String> {
        vec![self]
    }
}

impl Reads for &str {
    fn paths(self) -> Vec<String> {
        vec![self.to_string()]
    }
}

impl Reads for Vec<String> {
    /// Repeats are removed, because one instance counts once against one
    /// document however many times it names it. A self-edge is the case: both
    /// endpoints are one file.
    fn paths(mut self) -> Vec<String> {
        let mut seen: Vec<String> = Vec::with_capacity(self.len());
        self.retain(|path| match seen.contains(path) {
            true => false,
            false => {
                seen.push(path.clone());
                true
            }
        });
        self
    }
}
