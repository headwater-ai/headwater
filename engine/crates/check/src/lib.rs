// SPDX-License-Identifier: Apache-2.0
//! The minimal runner: two generated checks, coverage against the census, and
//! text findings in one order.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#two-phases-and-why-the-order-matters)
//! splits a run in two. Phase A classifies and builds, and it is
//! [`headwater_census`] and [`headwater_graph`]. Phase B runs the checks over
//! the graph that Phase A produced, and it is this crate. The order is the
//! answer to the silent pass: the denominator is fixed before any check starts.
//!
//! # What is deliberately absent, and where each piece goes
//!
//! This runner is the thinnest thing that closes the loop over a real corpus,
//! which is what [M1](https://github.com/headwater-ai/headwater/milestone/1)
//! exists to do. Four parts of the designed check layer are not here, and none
//! of them is an oversight.
//!
//! **Scope as a type.** [Spec 12](../../../../docs/spec/12-check-layer.md#the-declaration-is-a-type-not-a-returned-value)
//! rules that a check receives a scoped view and cannot ask for a wider one,
//! and that the enforcement is the feature. Both checks here take the whole
//! census or the whole graph, so neither declares a scope and no view enforces
//! one. That is [#54](https://github.com/headwater-ai/headwater/issues/54).
//!
//! **The cache, and change-scoped evaluation.** Every instance runs on every
//! run. Nothing is keyed, nothing is reused, and `--changed-only` does not
//! exist. That is [#55](https://github.com/headwater-ai/headwater/issues/55),
//! and a cache before a sound cache key would be the correctness root
//! [spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots)
//! warns about.
//!
//! **Suppression.** A finding here cannot be suppressed, so nothing is filtered
//! and there is no inventory to report. That is
//! [#58](https://github.com/headwater-ai/headwater/issues/58).
//!
//! **The obligation register.** Two of the three rules below name no
//! obligation, because an obligation is data and no package this repository
//! resolves declares any. See [`finding`], which states the gap where a reader
//! of a finding meets it.
//!
//! # Phase A outcomes stay in Phase A
//!
//! Spec 12 calls an unparseable file, an unclassifiable path, a dangling edge
//! and an ambiguous shelf match *structural findings*. This runner does not
//! re-report them as findings. The census and the graph already account for
//! every one of them, each with the row or the exception line that names the
//! author who can act, and a second report of one fact sends that author to two
//! places. Giving those outcomes a severity needs the obligation register, so
//! the two arrive together.

pub mod coverage;
pub mod finding;
pub mod instance;
pub mod placement;
pub mod reciprocity;

pub use coverage::Coverage;
pub use finding::{Finding, Severity};
pub use instance::{Instance, Outcome};

use headwater_census::census::Census;
use headwater_census::shelves::Taxonomy;
use headwater_graph::{Declarations, Graph};

/// The rules this runner carries, in the order a report lists them.
///
/// Two are generated from the taxonomy and one is the coverage guarantee
/// itself. A rule that is generated has no entry of its own anywhere: the list
/// is the *templates*, and the instance count is what a taxonomy decides.
pub const RULES: [&str; 3] = [placement::RULE, reciprocity::RULE, coverage::RULE];

/// One run of the check layer over one corpus.
#[derive(Clone, Debug)]
pub struct Run {
    /// Every instance of every check, in the order the checks are listed.
    pub instances: Vec<Instance>,
    pub coverage: Coverage,
    /// Every finding, in the one order
    /// [spec 12](../../../../docs/spec/12-check-layer.md#determinism-concretely)
    /// fixes.
    pub findings: Vec<Finding>,
}

/// Run every check over one census and the graph built from it.
///
/// The census and the graph are arguments rather than something this function
/// builds, for the reason the graph build takes a census: two passes over one
/// corpus can disagree, and the pair that would disagree here is the
/// denominator and the thing measured against it.
pub fn run(
    census: &Census,
    graph: &Graph,
    taxonomy: &Taxonomy,
    declarations: &Declarations,
) -> Run {
    let mut instances = placement::run(census, taxonomy);
    instances.extend(reciprocity::run(graph, declarations));

    let coverage = Coverage::of(census, &instances);

    let mut findings: Vec<Finding> = instances
        .iter()
        .filter_map(|instance| instance.finding().cloned())
        .collect();
    findings.extend(coverage.findings());

    Run {
        instances,
        coverage,
        findings: finding::sorted(findings),
    }
}

/// How much of a run to write out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Detail {
    /// Every instance, whatever it found. Right for a fixture tree, where each
    /// one is the point.
    EveryInstance,
    /// The totals, and every finding.
    Findings,
}

impl Run {
    /// Whether a `--strict` invocation would fail on this run.
    pub fn has_errors(&self) -> bool {
        self.findings
            .iter()
            .any(|finding| finding.severity == Severity::Error)
    }

    /// Findings per severity, in severity order, so that two reports line up.
    pub fn counts(&self) -> Vec<(Severity, usize)> {
        [Severity::Error, Severity::Warn, Severity::Info]
            .into_iter()
            .map(|severity| {
                (
                    severity,
                    self.findings
                        .iter()
                        .filter(|finding| finding.severity == severity)
                        .count(),
                )
            })
            .collect()
    }

    /// Instances per rule, in the order [`RULES`] lists them.
    pub fn per_rule(&self) -> Vec<(&'static str, usize)> {
        RULES
            .iter()
            .map(|rule| {
                (
                    *rule,
                    self.instances
                        .iter()
                        .filter(|instance| instance.rule == *rule)
                        .count(),
                )
            })
            .collect()
    }

    /// The run as text.
    ///
    /// Coverage comes first and it accounts for every file, whatever `detail`
    /// then prints. Same order and same argument as the census and the graph: a
    /// reader who checks nothing else still sees the denominator, and "no
    /// findings across 36 of 36" and "no findings across 30 of 36" are not the
    /// same result.
    pub fn render(&self, detail: Detail) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        out.push_str(&self.coverage.render());
        for (rule, count) in self.per_rule() {
            if count > 0 {
                let _ = writeln!(out, "  {count:5} instances of {rule}");
            }
        }

        let _ = writeln!(out, "{} findings", self.findings.len());
        for (severity, count) in self.counts() {
            if count > 0 {
                let _ = writeln!(out, "  {count:5} {severity}");
            }
        }

        if detail == Detail::EveryInstance {
            out.push_str("\ninstances\n");
            for instance in &self.instances {
                let _ = writeln!(
                    out,
                    "  {} {}\n    {}",
                    instance.rule,
                    instance.reads.join(" + "),
                    match &instance.outcome {
                        Outcome::Passed => "passed".to_string(),
                        Outcome::Failed(_) => "failed".to_string(),
                        Outcome::Skipped(reason) => format!("skipped: {reason}"),
                    }
                );
            }
        }

        if !self.findings.is_empty() {
            out.push('\n');
            for finding in &self.findings {
                out.push_str(&finding.render());
            }
        }
        out
    }
}
