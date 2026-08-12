// SPDX-License-Identifier: Apache-2.0
//! An instance: one check, applied to one target, with one outcome.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#instances-and-why-coverage-needs-them)
//! says a check is a template and the engine instantiates it per target, and
//! that coverage then follows with no added mechanism: "each run records, per
//! document, which instances were created, which ran, which were served from
//! cache, and which were skipped with a reason."
//!
//! Three of those four are here, and the fourth is deliberately not. Whether
//! an instance was served from a cache is a fact about a disk rather than
//! about a corpus, and [`crate::cache`] states why no report carries it.
//!
//! # A read is a path and a content hash
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict)
//! fixes what the read set of a run is: "the content hash of every document
//! and edge that an instance read". Coverage needs the path, because it counts
//! per document. A cache key needs the hash, because a key that omits an input
//! is a correctness bug rather than a performance bug. Both are one field, so
//! no second pass can produce a read set that disagrees with the first.
//!
//! The hash is the census's, of the bytes it read. An [`Input`] whose digest
//! is absent is a document that the walk never read, and [`crate::cache`]
//! refuses to key on it rather than substitute a value.
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
    /// Every document this instance read, in a fixed order and without
    /// repeats. It comes from the view rather than from the check, so a check
    /// cannot record that it read less than it was handed.
    pub reads: Vec<Input>,
    pub outcome: Outcome,
}

/// One in-scope input: a document, and the hash of the bytes it holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Input {
    /// Relative to the repository root, with `/` separators.
    pub path: String,
    /// The census's digest of the bytes of that file, and `None` where the
    /// walk read none. See the module comment.
    pub digest: Option<String>,
}

/// What became of one instance. Closed, and matched exhaustively.
#[derive(Clone, Debug)]
pub enum Outcome {
    /// It ran and found nothing.
    Passed,
    /// It ran and found something. Spec 12 gives a check the signature
    /// `check(view, ctx) -> [Finding]`, and the list is not a formality: one
    /// document can omit four facets that its kind requires, and each omission
    /// is a separate line for a separate author to add. A verdict that named
    /// the first would make the other three cost one run each to discover.
    ///
    /// Never empty. A check with nothing to say returns [`Outcome::Passed`],
    /// and [`Outcome::failed`] is the constructor that holds the two apart.
    Failed(Vec<Finding>),
    /// It did not run, and the reason is visible rather than silent
    /// ([spec 4](../../../../docs/spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)).
    Skipped(&'static str),
}

impl Outcome {
    /// The verdict of a check that collected its findings as it went.
    ///
    /// An empty list is a pass, which is the one place the two are decided
    /// together. A check that built the distinction itself would eventually get
    /// it wrong in one rule and report a failure with nothing in it.
    pub fn failed(findings: Vec<Finding>) -> Self {
        match findings.is_empty() {
            true => Outcome::Passed,
            false => Outcome::Failed(findings),
        }
    }

    /// One finding, for a rule that can only ever have one.
    pub fn failed_with(finding: Finding) -> Self {
        Outcome::Failed(vec![finding])
    }
}

impl Instance {
    /// One instance, from what the view carried and what the check made of it.
    ///
    /// [`crate::scope`] is the only caller, and that is the point: the read
    /// set comes from the view rather than from the check, so a check cannot
    /// record that it read less than it was handed.
    pub fn of(rule: &'static str, reads: Vec<Input>, outcome: Outcome) -> Self {
        Instance {
            rule,
            reads,
            outcome,
        }
    }

    pub fn skipped(rule: &'static str, reads: Vec<Input>, reason: &'static str) -> Self {
        Instance {
            rule,
            reads,
            outcome: Outcome::Skipped(reason),
        }
    }

    /// Every document this instance read, and nothing about their contents.
    pub fn paths(&self) -> Vec<&str> {
        self.reads.iter().map(|input| input.path.as_str()).collect()
    }

    pub fn ran(&self) -> bool {
        !matches!(self.outcome, Outcome::Skipped(_))
    }

    /// Every finding this instance reached, and none for one that passed or
    /// was skipped.
    pub fn findings(&self) -> &[Finding] {
        match &self.outcome {
            Outcome::Failed(findings) => findings,
            _ => &[],
        }
    }

    /// The first document this instance read, which is the one a report sorts
    /// and groups by.
    pub fn at(&self) -> &str {
        self.reads
            .first()
            .map(|input| input.path.as_str())
            .unwrap_or_default()
    }
}

impl Input {
    /// One input, from a census row's path and its digest.
    pub fn new(path: impl Into<String>, digest: Option<&str>) -> Self {
        Input {
            path: path.into(),
            digest: digest.map(str::to_string),
        }
    }
}
