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
//! exists to do. Three parts of the designed check layer are not here, and
//! none of them is an oversight.
//!
//! **Change-scoped evaluation.** Every instance is created on every run, and
//! `--changed-only` does not exist. What does exist is the cache the same
//! sentence of spec 12 needs first: an instance is keyed on the content hashes
//! of what it read, and [`cache`] serves the ones nothing touched. That does
//! not make a run partial — every instance still has an outcome and coverage
//! counts what it counted — and a run that evaluates less of the corpus is
//! [#58](https://github.com/headwater-ai/headwater/issues/58).
//!
//! **Suppression.** A finding here cannot be suppressed, so nothing is filtered
//! and there is no inventory to report. That is
//! [#58](https://github.com/headwater-ai/headwater/issues/58).
//!
//! **The generated obligation register.** Every rule below now names the
//! obligation it serves, because the base package declares `obligations` and
//! `controls` and [`register`] is the path from a rule to its obligation. What
//! is absent is the register as an artifact: coverage by obligation,
//! dispositions, and the control health that spec 4 asks a projection to carry.
//! That is [#59](https://github.com/headwater-ai/headwater/issues/59).
//!
//! # Scope is a type, and [`scope`] is where that holds
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-declaration-is-a-type-not-a-returned-value)
//! rules that a check receives a scoped view and cannot ask for a wider one,
//! and that the enforcement is the feature. Each rule below implements one
//! scope trait, and that trait is the only way to receive the matching view.
//! [`run`] names the two checks it runs, which is the whole of registration.
//! The scope a trait fixes is now read twice: once for the report, and once as
//! a component of the cache key that spec 12 derives from the same fact.
//!
//! `Neighbourhood`, `Shelf` and a corpus-scoped *trait* are absent, and the
//! reason is the one this module already applies to a cache: no rule needs
//! one. Spec 12 leaves the first two open, and [`coverage`] states why a
//! corpus-grained rule is the runner's accounting rather than a check.
//!
//! # Phase A outcomes stay in Phase A
//!
//! Spec 12 calls an unparseable file, an unclassifiable path, a dangling edge
//! and an ambiguous shelf match *structural findings*. This runner does not
//! re-report them as findings. The census and the graph already account for
//! every one of them, each with the row or the exception line that names the
//! author who can act, and a second report of one fact sends that author to two
//! places. Turning them into findings needs an obligation for each class, which
//! the base package does not declare, and #59 is where that lands.

pub mod cache;
pub mod coverage;
pub mod finding;
pub mod instance;
pub mod placement;
pub mod reciprocity;
pub mod register;
pub mod scope;

pub use cache::Cache;
pub use coverage::Coverage;
pub use finding::{Finding, Severity};
pub use instance::{Input, Instance, Outcome};
pub use register::{Bound, Register};
pub use scope::{DocumentCheck, DocumentView, EdgeCheck, EdgeView, Grain, Scope};

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
    /// What each rule sees and what it serves, in [`RULES`] order. A rule that
    /// reaches no obligation is in this list too, because a rule that cannot
    /// say which invariant it protects is what spec 4 asks a reader to notice.
    pub served: Vec<Serves>,
    /// What this run did with its cache. Deliberately outside [`Run::render`]:
    /// see [`cache`] for why a hit count is not part of a verdict.
    pub cache: cache::Report,
}

/// One rule, the scope that binds it, and the obligation it serves.
#[derive(Clone, Debug)]
pub struct Serves {
    pub rule: &'static str,
    /// Derived from the trait the check implements, and never stated beside
    /// it. See [`scope`] for why that distinction is the whole feature.
    pub scope: Scope,
    pub obligation: Bound,
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
    register: &Register,
    cache: &mut Cache,
) -> Run {
    // Registration, in full: two checks, each named once. The scope trait each
    // one implements decides what it is handed, so this function cannot widen
    // a view by calling the wrong instantiation.
    let placement = placement::Placement::over(taxonomy);
    let reciprocity = reciprocity::Reciprocity::over(declarations);
    let digests = scope::Digests::of(census);
    let mut instances = scope::over_documents(&placement, census, cache);
    instances.extend(scope::over_edges(&reciprocity, graph, &digests, cache));

    let coverage = Coverage::of(census, &instances);

    let mut findings: Vec<Finding> = instances
        .iter()
        .filter_map(|instance| instance.finding().cloned())
        .collect();
    findings.extend(coverage.findings());

    // The obligation is stamped here rather than written into each rule,
    // because the binding is data. A rule states its id, a control names that
    // id and the obligations it discharges, and one place reads the two
    // together. See [`register`] for why that place is not the check.
    let served: Vec<Serves> = [
        (
            placement::RULE,
            scope::document_scope::<placement::Placement>(),
        ),
        (
            reciprocity::RULE,
            scope::edge_scope::<reciprocity::Reciprocity>(),
        ),
        (coverage::RULE, coverage::SCOPE),
    ]
    .into_iter()
    .map(|(rule, scope)| Serves {
        rule,
        scope,
        obligation: register.bound(rule),
    })
    .collect();
    for finding in &mut findings {
        finding.obligation = match served.iter().find(|served| served.rule == finding.rule) {
            Some(Serves {
                obligation: Bound::To(obligation),
                ..
            }) => Some(obligation.clone()),
            _ => None,
        };
    }

    Run {
        instances,
        coverage,
        findings: finding::sorted(findings),
        served,
        cache: cache.report(),
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

        // What each rule sees and what it serves. Spec 4 asks every finding to
        // name its obligation, so a rule that names none is a fact about the
        // taxonomy and it belongs beside the counts rather than in a reader's
        // inference. The scope is here for the same reason and one more: spec
        // 12 asks that the count of the barriers be a number a reader can
        // read, rather than a property discovered under load.
        out.push_str("rules, and for each the scope that binds it and what it serves\n");
        for served in &self.served {
            let rule = served.rule;
            let _ = writeln!(out, "  {rule}\n    {}", served.scope.render());
            let _ = match &served.obligation {
                Bound::To(obligation) => writeln!(out, "    {obligation}"),
                Bound::Unnamed => writeln!(
                    out,
                    "    no control names this rule, so its findings name no obligation"
                ),
                Bound::Several(obligations) => writeln!(
                    out,
                    "    reaches {}, and a finding names one obligation, so it names none",
                    obligations.join(", ")
                ),
            };
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
                    instance.paths().join(" + "),
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
