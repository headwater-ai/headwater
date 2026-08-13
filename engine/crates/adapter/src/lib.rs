// SPDX-License-Identifier: Apache-2.0
//! The CI adapters: one run of the check layer, in a vocabulary a platform
//! reads.
//!
//! [Spec 6](../../../../docs/spec/06-engine-architecture.md#ci-adapters): "The
//! engine emits findings. Adapters translate them to the native vocabulary of a
//! platform: annotations, check runs, job summaries, review comments. Adapters
//! are thin and swappable so that no forge is privileged in the core."
//!
//! # Three formats, and one of them is not written here
//!
//! Spec 6 fixes the set: `text`, `json`, `sarif`, `markdown`. This crate writes
//! the last three. `text` stays with the caller because the text report is
//! composed from more than a run — the census and the graph are in it, and this
//! crate receives neither. [`Format::Text`] is in the enum so that one list
//! names every format a reader may ask for, and [`render`] returns `None` for
//! it rather than writing a second, thinner text report beside the first.
//!
//! # Why this is a crate and not a module of `headwater-check`
//!
//! The dependency direction is the boundary spec 8 asks for.
//! [Departure 6](../../../../docs/spec/08-design-departures.md): "a
//! platform-neutral core with thin adapters. No forge, tracker, or cloud is
//! privileged. More than one adapter exists from the start, and that enforces
//! the adapter boundary." A module inside the runner could reach a check. A
//! crate above it cannot, and the compiler is what says so rather than a
//! review. Nothing in `headwater-check` names this crate.
//!
//! # Two adapters and one neutral serialization, which is what "more than one"
//! buys
//!
//! [`Format::Sarif`] and [`Format::Markdown`] translate into two platform
//! vocabularies that spec 6 names: a check run, which a forge ingests as SARIF,
//! and a job summary, which a forge renders as Markdown. They lose different
//! things, and that is the point of shipping both. One adapter cannot show
//! where the boundary is, because everything it needs is in the core by
//! definition.
//!
//! [`Format::Json`] is not an adapter. It is the finding shape
//! [spec 4](../../../../docs/spec/04-assurance-model.md#findings) already
//! declares, written out, for an adapter that this repository did not write.
//! Its loss set is empty and [`census`] is what audits that claim.
//!
//! # Every format declares a loss set
//!
//! [Spec 6](../../../../docs/spec/06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped)
//! requires an emitter to declare "the node classes, edge classes, and
//! attributes that its target cannot carry, each with a reason". A finding is
//! not a node, so [`Loss`] here is about a *field of a run* rather than about a
//! class of the graph, and it is deliberately not
//! `headwater_generate::export::Loss`. Two subjects, two types, and an adapter
//! that depended on the graph emitters to say what a job summary drops would
//! have the boundary above pointing the wrong way.
//!
//! What the two share is the doctrine. A declaration is a claim, and a census
//! is the audit of it. [`census`] is that audit: every finding of a run is in
//! the output, whatever its escape class, or the format declared why not.
//!
//! # The engine emits and never orders
//!
//! No output here carries a verdict about a merge. `--strict` decides an exit
//! status and the artifact says the same thing whether or not the flag was
//! given, because the posture is the control's
//! ([spec 12](../../../../docs/spec/12-check-layer.md#severity-is-the-checks-posture-is-the-controls))
//! and the landing order is the forge's.

pub mod json;
pub mod markdown;
pub mod sarif;

use headwater_check::adoption::Task;
use headwater_check::suppression::Suppression;
use headwater_check::{Finding, Run, Severity};

/// The name this tool reports under, in every format that names it.
pub const TOOL: &str = "headwater";

/// Where a reader finds out what the tool is.
pub const TOOL_URI: &str = "https://github.com/headwater-ai/headwater";

/// One output format of `headwater check`.
///
/// The set is closed by [spec 6](../../../../docs/spec/06-engine-architecture.md#cli)
/// and this enum is that list. It is deliberately not
/// `headwater_generate::Emitter`: `export --format` names a vocabulary for the
/// *graph* and `check --format` names one for the *findings*, the two lists
/// share the word `json` and mean different artifacts by it, and one enum over
/// both would let `check --format shacl` parse.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// The report a person reads in a terminal. Composed by the caller: see the
    /// module comment.
    Text,
    /// The finding shape spec 4 declares. Not an adapter.
    Json,
    /// Static Analysis Results Interchange Format 2.1.0 (OASIS), which is what
    /// a forge ingests as a check run.
    Sarif,
    /// A job summary or a review comment, which is the other vocabulary spec 6
    /// names.
    Markdown,
}

impl Format {
    /// Every format, in the order spec 6 lists them.
    pub const ALL: [Format; 4] = [Format::Text, Format::Json, Format::Sarif, Format::Markdown];

    pub fn name(self) -> &'static str {
        match self {
            Format::Text => "text",
            Format::Json => "json",
            Format::Sarif => "sarif",
            Format::Markdown => "markdown",
        }
    }

    pub fn parse(text: &str) -> Option<Format> {
        Format::ALL.into_iter().find(|format| format.name() == text)
    }

    /// What this format cannot carry, and why.
    ///
    /// Empty for [`Format::Json`], which is the claim that it drops nothing,
    /// and [`census`] is what holds that claim. Empty for [`Format::Text`] for
    /// a different reason: this crate does not write it, so it declares
    /// nothing about it.
    pub fn loss(self) -> &'static [Loss] {
        match self {
            Format::Text | Format::Json => &[],
            Format::Sarif => sarif::LOSS,
            Format::Markdown => markdown::LOSS,
        }
    }
}

/// One entry of a format's loss set: something a run carries, the target
/// vocabulary has no member for, and the reason.
///
/// `carried_in` names where the value went instead, and it is empty where the
/// value went nowhere. A property bag is not a member of the vocabulary, so a
/// value that rides in one is still a loss for every consumer that reads the
/// vocabulary alone. Saying which is the difference between a loss set and an
/// apology.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Loss {
    pub field: &'static str,
    pub reason: &'static str,
    pub carried_in: &'static str,
}

/// Which escape mechanism holds a finding back, in the precedence
/// [spec 4](../../../../docs/spec/04-assurance-model.md#suppression) fixes:
/// waiver, then migration-pending, then suppression.
///
/// Two variants rather than three, because this engine has no waiver mechanism
/// and a variant nothing constructs is a claim with no instance behind it. The
/// run reports the absence in the same sentence that reports the other two, and
/// [`sarif`] states what happens to the mapping when a waiver arrives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Escape {
    /// A task of the adoption payload holds it: declared debt with an owner and
    /// an expiry ([spec 7](../../../../docs/spec/07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states)).
    MigrationPending,
    /// A directive in the source hides it: one author's local judgment.
    Suppression,
}

impl Escape {
    pub fn name(self) -> &'static str {
        match self {
            Escape::MigrationPending => "migration-pending",
            Escape::Suppression => "suppression",
        }
    }
}

/// One finding of a run, with what became of it.
///
/// The union that every format writes. A reader of [`Run::findings`] alone sees
/// what was reported and cannot see what was held or hidden, and the whole
/// question a CI surface answers is which of the three a line is.
pub struct Reported<'a> {
    pub finding: &'a Finding,
    /// `None` for a live finding.
    pub escape: Option<Escape>,
    /// The task that holds it, for [`Escape::MigrationPending`].
    pub task: Option<&'a Task>,
    /// The directive that hides it, for [`Escape::Suppression`].
    pub directive: Option<&'a Suppression>,
}

impl Reported<'_> {
    /// Whether a reader of the report sees this finding among the findings.
    pub fn is_live(&self) -> bool {
        self.escape.is_none()
    }
}

/// Every finding of a run, live and escaped, in the one order spec 12 fixes.
///
/// The three sources are disjoint by construction: the runner applies the
/// adoption payload first and the directives second, so a held finding never
/// reaches a directive. Sorting the union rather than concatenating three
/// sequences is what makes the output stable when a finding moves from one
/// bucket to another.
pub fn reported(run: &Run) -> Vec<Reported<'_>> {
    let mut all: Vec<Reported<'_>> = Vec::new();
    for finding in &run.findings {
        all.push(Reported {
            finding,
            escape: None,
            task: None,
            directive: None,
        });
    }
    for finding in &run.adoption.pending {
        all.push(Reported {
            finding,
            escape: Some(Escape::MigrationPending),
            task: run
                .adoption
                .tasks
                .iter()
                .find(|task| {
                    task.pairs
                        .iter()
                        .any(|pair| pair.rule == finding.rule && pair.path == finding.path)
                }),
            directive: None,
        });
    }
    for (finding, directive) in run.suppressions.hidden_findings() {
        all.push(Reported {
            finding,
            escape: Some(Escape::Suppression),
            task: None,
            directive: Some(directive),
        });
    }
    all.sort_by(|left, right| left.finding.order().cmp(&right.finding.order()));
    all
}

/// What a format carried, and what its loss set accounted for.
///
/// The audit of the claim a loss set makes, in the shape
/// [spec 6](../../../../docs/spec/06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped)
/// fixes for the graph emitters: "Every node and every edge in the graph is
/// either present in the output, or accounted for by a declared loss reason."
/// Here the denominator is every finding of the run rather than every node of
/// the graph, and the rest of the sentence is the same one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Census {
    pub findings: usize,
    pub carried: usize,
    /// Findings the output does not carry and no loss reason covers. A
    /// non-empty list is a defect in the adapter, not a fact about the corpus.
    pub unaccounted: Vec<String>,
}

impl Census {
    pub fn is_defective(&self) -> bool {
        !self.unaccounted.is_empty()
    }
}

/// Audit one rendered artifact against the run it came from.
///
/// It reads the bytes rather than the emitter, for the reason
/// `headwater_generate::export::audit` does: an emitter that audited itself
/// would be the untrusted projector one layer out. A finding is carried when
/// the output names its rule and its path in one place a reader can find, which
/// is the coarsest grain all three formats share.
pub fn census(run: &Run, artifact: &str) -> Census {
    let all = reported(run);
    let mut unaccounted = Vec::new();
    let mut carried = 0;
    for entry in &all {
        let rule = artifact.contains(entry.finding.rule);
        let path = artifact.contains(entry.finding.path.as_str());
        match rule && path {
            true => carried += 1,
            false => unaccounted.push(format!(
                "{} {} at {}",
                entry.finding.rule, entry.finding.path, entry.finding.line
            )),
        }
    }
    Census {
        findings: all.len(),
        carried,
        unaccounted,
    }
}

/// What every format states about the run beside the findings.
///
/// [Spec 6](../../../../docs/spec/06-engine-architecture.md#ci-adapters): "Every
/// run reports the corpus tree, the taxonomy lock hash, and its read set beside
/// the findings." Two of those three are here. The corpus tree is not, because
/// nothing computes one — see [`sarif`] for where that absence shows up and
/// what this crate declined to print in its place.
pub struct Subject<'a> {
    /// The taxonomy package the lock names.
    pub package: &'a str,
    pub version: &'a str,
    /// The lock digest, as `sha256:...`.
    pub lock: &'a str,
    /// The injected clock, as `YYYY-MM-DD`. An input to the verdict, so an
    /// output that omits it cannot be reproduced from itself.
    pub now: &'a str,
}

/// The run in one format.
///
/// `None` for [`Format::Text`], which the caller composes: see the module
/// comment.
pub fn render(run: &Run, subject: &Subject<'_>, format: Format) -> Option<String> {
    match format {
        Format::Text => None,
        Format::Json => Some(json::render(run, subject)),
        Format::Sarif => Some(sarif::render(run, subject)),
        Format::Markdown => Some(markdown::render(run, subject)),
    }
}

/// The check's severity as the word every format writes.
pub fn severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warn => "warn",
        Severity::Info => "info",
    }
}
