// SPDX-License-Identifier: Apache-2.0
//! The LLM-assisted coherence sweep: the half of it that is deterministic.
//!
//! [Spec 4](../../../../docs/spec/04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep)
//! declares the sweep and
//! [spec 12](../../../../docs/spec/12-check-layer.md#where-the-llm-coherence-sweep-fits)
//! says where it sits: "It runs as a separate sampler, with the same finding
//! shape and the same reporting pipeline. Its provenance is marked `agent`, it
//! never gates, and it is never cached as if it is reproducible."
//!
//! # No model is reached from here, and that is the design rather than a gap
//!
//! A sweep has three parts and this crate is two of them. [`plan`] states what
//! an agent is to read. The agent reads it, reads the documents, and writes a
//! return file. [`intake`] reads that file back and reports what it could
//! confirm about it.
//!
//! The middle part is the only one that needs a model, and it is not in this
//! binary. No code here opens a socket, and nothing in the engine does. So the
//! build never depends on a model being reachable: an unreachable model means
//! nobody wrote a return file, and a verb with no file to read says so and
//! stops. That is what makes the constraint structural rather than a policy.
//!
//! # What a sweep's output is
//!
//! A **proposal**, and never a verdict. A carried finding here says three
//! things and a reader has to keep them apart:
//!
//! - a model claimed something about the corpus. This is judgment, it is not
//!   reproducible, and a rerun may not claim it again.
//! - the engine confirmed the *citation*: every quoted passage is in the
//!   document the finding attributes it to, and every document it names is a
//!   classified document of this corpus. This half is reproducible, and
//!   [`intake`] is a pure function of the return file and the tree.
//!   ([`Report::render`] over one file writes one set of bytes.)
//! - the engine confirmed the *claim to novelty*: the graph does not already
//!   declare the edge the finding proposes. Spec 4's fourth constraint makes a
//!   restatement a defect in the sweep rather than a fact about the corpus.
//!
//! So the sweep is a reproducible verification of an unreproducible sample.
//! That is what makes it worth acting on: a fabricated quotation is refused
//! before a person reads it, which is the failure mode a prose judgment fails
//! by. And it is what makes it unfit to gate: the *set* of findings is a
//! sample. Absence means nothing, so no exit status may read it, and this crate
//! provides no way for one to.
//!
//! # Why the exit status never moves
//!
//! [`intake`] returns a report and never a verdict, and the verb over it exits
//! zero whatever the report holds — findings carried, findings refused, or the
//! whole file refused. The only non-zero exits are a caller's: no path given,
//! or a path this process cannot read.
//!
//! A refusal is a verdict about the *sampler*, not about the corpus, and it is
//! tempting to exit non-zero on one. That would put a model's output on an exit
//! status, which is the one thing the constraint above forbids, and a scheduled
//! job that failed because a model wrote bad YAML is a build that depends on a
//! model after all.
//!
//! # The best outcome is an edge, so the report writes the edge out
//!
//! Spec 4: "the best outcome of a sweep is not a finding but an edge". A
//! carried finding that proposes one prints the front matter that declares it.
//! Nothing here writes it. A proposal an agent applies to itself is the same
//! act as an agent accepting its own work, which is what
//! [OBL-repo-0108](../../../../docs/obligations/0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md)
//! records, so the verb has no `--write` and this crate opens no file for
//! writing.

pub mod intake;
pub mod json;
pub mod plan;

pub use intake::{Refusal, Rejected, Report, Tree, Verified};
pub use plan::Plan;

/// What a sweep finding claims, from the closed set spec 4 declares.
///
/// Closed for the reason every value set in this engine is closed: a class
/// nobody declared is a class no reader can act on and no report can total. A
/// return file that names one outside this set is refused with the set printed
/// beside it.
///
/// # What a sixth class has to be, and why readability is not one
///
/// Every class below names **two things and says they do not fit**. Two
/// documents. A newer claim against an older one. A term against the place its
/// definition is owed. A document against its declared audience. A heading
/// against the prose beneath it. A reader holds one against the other and
/// adjudicates in seconds, which is the standard spec 4 sets, and
/// [`Class::remediation`] reads the same way: every remedy declares an edge,
/// defines a term, narrows an audience or writes a section.
///
/// A finding that names one thing fails that test whatever the intake can
/// confirm about it. [Q24](../../../../docs/decisions/0024-q24-readability-and-what-a-sweep-can-be-asked-about.md)
/// refused readability on exactly this ground, and it refused the easier
/// argument with it: three of the five below already return `None` from
/// [`Class::implies`], so a vacuous novelty leg disqualifies nothing. With no
/// second term the quotation is the finding rather than evidence for it, and
/// [`crate::intake`] then confirms what `grep` confirms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// Two documents that contradict each other while both remain current, and
    /// neither declares it.
    UndeclaredConflict,
    /// A claim that a newer source quietly superseded.
    QuietSupersession,
    /// A concept referenced across the corpus and defined nowhere.
    UndefinedConcept,
    /// A document whose declared audience cannot use it.
    AudienceMismatch,
    /// A heading a kind requires, over prose that says nothing about it. See
    /// [OBL-repo-0113](../../../../docs/obligations/0113-every-check-passes-a-document-that-is-still-the-scaffolder-s-placeholder.md).
    UnwrittenSection,
}

impl Class {
    /// Every class, in the order spec 4 lists them.
    pub const ALL: [Class; 5] = [
        Class::UndeclaredConflict,
        Class::QuietSupersession,
        Class::UndefinedConcept,
        Class::AudienceMismatch,
        Class::UnwrittenSection,
    ];

    /// The name an agent writes in a return file.
    pub fn name(self) -> &'static str {
        match self {
            Class::UndeclaredConflict => "undeclared_conflict",
            Class::QuietSupersession => "quiet_supersession",
            Class::UndefinedConcept => "undefined_concept",
            Class::AudienceMismatch => "audience_mismatch",
            Class::UnwrittenSection => "unwritten_section",
        }
    }

    pub fn read(name: &str) -> Option<Class> {
        Class::ALL.into_iter().find(|class| class.name() == name)
    }

    /// The rule name a finding of this class reports under.
    ///
    /// The `sweep.` prefix is what tells a reader that no check produced it.
    /// The strings are `&'static` because [`headwater_check::Finding`] holds
    /// the rule that way, and a sampler that could mint a rule name at run time
    /// would be a rule set nobody can enumerate.
    pub fn rule(self) -> &'static str {
        match self {
            Class::UndeclaredConflict => "sweep.undeclared_conflict",
            Class::QuietSupersession => "sweep.quiet_supersession",
            Class::UndefinedConcept => "sweep.undefined_concept",
            Class::AudienceMismatch => "sweep.audience_mismatch",
            Class::UnwrittenSection => "sweep.unwritten_section",
        }
    }

    /// The relation a finding of this class is a candidate for, when the class
    /// implies one. Spec 4's fourth constraint reads this: a finding of a class
    /// whose relation the graph already declares between the two documents is
    /// a restatement, and a restatement is a defect in the sweep.
    pub fn implies(self) -> Option<&'static str> {
        match self {
            Class::UndeclaredConflict => Some("conflicts_with"),
            Class::QuietSupersession => Some("supersedes"),
            Class::UndefinedConcept | Class::AudienceMismatch | Class::UnwrittenSection => None,
        }
    }

    /// What a reader does about a finding of this class.
    ///
    /// A finding that states no remedy is a complaint, which is the rule
    /// [`headwater_check::Finding::remediation`] already carries. Every remedy
    /// here is a rewrite or a declaration, and never an edit this engine
    /// writes, which is why no sweep finding carries a patch.
    pub fn remediation(self) -> &'static str {
        match self {
            Class::UndeclaredConflict => {
                "read both passages. Declare `conflicts_with` between the two documents, or record why they do not conflict"
            }
            Class::QuietSupersession => {
                "read both passages. Declare `supersedes` from the newer document, or correct the older one"
            }
            Class::UndefinedConcept => {
                "define the term where the corpus defines its terms, or narrow the documents that use it"
            }
            Class::AudienceMismatch => {
                "narrow the declared audience, or write the document the declared audience can use"
            }
            Class::UnwrittenSection => {
                "write the section, or drop the heading if the kind does not require it"
            }
        }
    }
}

/// The word every sweep finding is stamped with, wherever one is rendered.
///
/// Spec 12: "Its provenance is marked `agent`". It is a constant rather than a
/// member of [`headwater_check::Finding`], because a finding's provenance is a
/// fact about the mechanism that produced it, and this mechanism has one. A
/// field on the shared shape would be a constant in one crate and dead weight
/// in the other, and the two mechanisms cannot mix in one report: a `Run` holds
/// check findings and this crate cannot build one.
pub const PROVENANCE: &str = "agent";
