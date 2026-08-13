// SPDX-License-Identifier: Apache-2.0
//! The runner: eleven checks, coverage against the census, a published read
//! set, a suppression inventory, and text findings in one order.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#two-phases-and-why-the-order-matters)
//! splits a run in two. Phase A classifies and builds, and it is
//! [`headwater_census`] and [`headwater_graph`]. Phase B runs the checks over
//! the graph that Phase A produced, and it is this crate. The order is the
//! answer to the silent pass: the denominator is fixed before any check starts.
//!
//! # Where the rules come from
//!
//! Nine of the eleven are **generated**. None of them names a facet, a kind, a
//! relation or a number of days: each reads a declaration out of the resolved
//! taxonomy and instantiates itself over whatever that declaration produced.
//! That is what [spec 12](../../../../docs/spec/12-check-layer.md#the-five-origins-of-a-check)
//! means by "a new facet or relation in the taxonomy produces its checks with
//! no code", and it is why the rule list is short while the instance count is
//! not. [`coverage`] is the runner's own accounting, and [`fragment`] reads no
//! declaration because the language has no member that turns prose-link
//! resolution on or off.
//!
//! Four of the five origins are represented. Shape, Graph and Document are
//! here. Corpus and Plugin are not, and neither has a rule that needs it yet.
//!
//! The four Document-origin rules are generated from a declaration in the same
//! sense the others are, and two of them meet a limit the language puts there.
//! `voice_regime.forbid` names categories and states no set of them, and
//! `language_regime.controlled` names a language and states no set either. So
//! the engine holds a closed set of each, and an instance that meets a name
//! outside it skips with that name in the reason rather than passing.
//!
//! # What is deliberately absent, and where each piece goes
//!
//! Two parts of the designed check layer are not here, and neither is an
//! oversight. Suppression used to be the third. It is now [`suppression`]: the
//! runner filters findings, records what it filtered, and feeds the inventory
//! into the coverage report, which is where spec 12 puts it.
//!
//! **Change-scoped evaluation, under the name `--changed-only`.** Every
//! instance is created on every run, and the flag does not exist.
//! [#58](https://github.com/headwater-ai/headwater/issues/58) declined to build
//! it and measured why: an instance is keyed on the content hashes of what it
//! read, and [`cache`] serves the ones nothing touched, so a warm run over this
//! repository takes 30 ms against 300 ms with no cache. The cache derives what
//! moved from the bytes rather than from a list a caller supplies, and spec 6
//! forbids a flag that puts an input into a verdict which no reviewer sees.
//! What is left unscoped is Phase A, which no flag reaches.
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
//! [`run`] names the ten checks it runs, which is the whole of registration.
//! The scope a trait fixes is now read twice: once for the report, and once as
//! a component of the cache key that spec 12 derives from the same fact. The
//! injected clock rides the same declaration, so a rule that reads a date
//! cannot be left out of its own key ([`cache`]).
//!
//! `Neighbourhood` is here, at depth 1, because
//! [`participation`] needed a grain that `Edge` does not reach: an edge-scoped
//! instance exists per edge, and a participation expectation is about an edge
//! that nobody declared. `Shelf` and a corpus-scoped *trait* are still absent,
//! and the reason is the one this module already applies to a cache: no rule
//! needs one. [`coverage`] states why a corpus-grained rule is the runner's
//! accounting rather than a check.
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

pub mod adoption;
pub mod cache;
pub mod context;
pub mod coverage;
pub mod endpoint;
pub mod facet_required;
pub mod facet_value;
pub mod finding;
pub mod fragment;
pub mod instance;
pub mod language;
pub mod participation;
pub mod placement;
pub mod readset;
pub mod reciprocity;
pub mod register;
pub mod scope;
pub mod sections;
pub mod shape;
pub mod suppression;
pub mod voice;

pub use adoption::Ledger;
pub use cache::Cache;
pub use context::{Context, Date};
pub use coverage::Coverage;
pub use finding::{Finding, Severity};
pub use instance::{Input, Instance, Outcome};
pub use readset::ReadSet;
pub use register::{Bound, Register};
pub use scope::{
    DocumentCheck, DocumentView, EdgeCheck, EdgeView, Grain, NeighbourhoodCheck, NeighbourhoodView,
    Scope,
};
pub use shape::{Purpose, Shape};
pub use suppression::Inventory;

use headwater_census::census::Census;
use headwater_census::shelves::Taxonomy;
use headwater_graph::{Declarations, Graph};

/// The rules this runner carries, in the order a report lists them.
///
/// Ten are generated from the taxonomy and one is the coverage guarantee
/// itself. A rule that is generated has no entry of its own anywhere: the list
/// is the *templates*, and the instance count is what a taxonomy decides.
///
/// The order is the five origins of
/// [spec 12](../../../../docs/spec/12-check-layer.md#the-five-origins-of-a-check),
/// which is Shape, then Graph, then the runner's own accounting.
pub const RULES: [&str; 13] = [
    facet_required::RULE,
    facet_value::RULE,
    placement::RULE,
    reciprocity::RULE,
    endpoint::RULE,
    participation::RULE,
    voice::RULE,
    language::RULE,
    sections::RULE,
    fragment::RULE,
    coverage::RULE,
    register::DISPOSITION,
    register::MECHANISM,
];

/// The declarations one run reads, from a taxonomy that is already resolved.
///
/// Four readers, each with the list of fields its own phase needs, and this is
/// where they arrive together. See [`shape`] for why widening one reader was
/// the alternative and what it would have cost.
pub struct Declared<'a> {
    /// The digest of the lock these four came out of.
    ///
    /// It is here rather than on [`Context`] because it is a fact about the
    /// taxonomy and not an injected value, and it is here rather than nowhere
    /// because [`ReadSet`] has to carry it: a lock that moved voids every
    /// result at once, and a gate reading a read set with no lock in it would
    /// decide that a verdict survived a taxonomy change.
    pub lock: &'a str,
    /// Shelves and abstract kinds, which is what kind resolution read.
    pub taxonomy: &'a Taxonomy,
    /// Facets, the kind hierarchy, and the participation expectations.
    pub shape: &'a Shape,
    /// Relation types and anchor kinds.
    pub relations: &'a Declarations,
    /// Obligations and controls: the path from a rule to what it serves.
    pub register: &'a Register,
    /// The `adoption` block of the lock, where the lock declares one.
    ///
    /// It arrives as a mapping rather than as tasks because the lock does not
    /// know what a rule is. [`crate::adoption::read`] turns it into tasks and
    /// reports what it could not read.
    pub adoption: Option<&'a headwater_yaml::Mapping>,
    /// Where the four above came from, as a path a reader can open.
    ///
    /// It is here because two rules of [`register`] are about the taxonomy
    /// rather than about the corpus, and a finding carries a path. For a run of
    /// the verb that is `.headwater/taxonomy.lock`, which spec 6 fixes as the
    /// one thing downstream reads.
    pub source: &'a str,
}

/// One run of the check layer over one corpus.
#[derive(Clone, Debug)]
pub struct Run {
    /// Every instance of every check, in the order the checks are listed.
    pub instances: Vec<Instance>,
    pub coverage: Coverage,
    /// Every finding a reader sees, in the one order
    /// [spec 12](../../../../docs/spec/12-check-layer.md#determinism-concretely)
    /// fixes. A finding an author suppressed is not here, and it is in
    /// [`Run::suppressions`] instead.
    pub findings: Vec<Finding>,
    /// What this run's authors suppressed, and what became of each directive.
    /// See [`suppression`]: the filter is the runner's, and a check never sees
    /// it.
    pub adoption: Ledger,
    pub suppressions: Inventory,
    /// What each rule sees and what it serves, in [`RULES`] order. A rule that
    /// reaches no obligation is in this list too, because a rule that cannot
    /// say which invariant it protects is what spec 4 asks a reader to notice.
    pub served: Vec<Serves>,
    /// The union of what this run read, with the lock, the clock and the check
    /// versions beside it. See [`readset`].
    pub read_set: ReadSet,
    /// The register: every obligation with its disposition, every control with
    /// its health, and what escaped under each obligation. Spec 4 makes it a
    /// projection of the two declarations, generated and never authored.
    pub register: register::Projection,
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
    /// Which edition of the rule ran. Read off the same trait as the scope,
    /// and for the same reason: it is a component of every key this run wrote,
    /// so a read set that stated a different one would describe another run.
    pub version: u32,
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
    declared: &Declared<'_>,
    ctx: &Context,
    cache: &mut Cache,
) -> Run {
    // Registration, in full: ten checks, each named once. The scope trait each
    // one implements decides what it is handed, so this function cannot widen
    // a view by calling the wrong instantiation.
    let required = facet_required::Required::over(declared.shape);
    let values = facet_value::Values::over(declared.shape);
    let placement = placement::Placement::over(declared.taxonomy);
    let reciprocity = reciprocity::Reciprocity::over(declared.relations);
    let endpoints = endpoint::Endpoints::over(declared.relations, declared.shape);
    let participation = participation::Participation::over(declared.shape, declared.relations);
    let voice = voice::Voice::over(declared.shape);
    let language = language::Language::over(declared.shape);
    let sections = sections::Sections::over(declared.shape);
    let fragments = fragment::Fragments;

    let digests = scope::Digests::of(census);
    let mut instances = scope::over_documents(&required, census, ctx, cache);
    instances.extend(scope::over_documents(&values, census, ctx, cache));
    instances.extend(scope::over_documents(&placement, census, ctx, cache));
    instances.extend(scope::over_edges(&reciprocity, graph, &digests, ctx, cache));
    instances.extend(scope::over_edges(&endpoints, graph, &digests, ctx, cache));
    instances.extend(scope::over_neighbourhoods(
        &participation,
        census,
        graph,
        &digests,
        ctx,
        cache,
    ));
    instances.extend(scope::over_documents(&voice, census, ctx, cache));
    instances.extend(scope::over_documents(&language, census, ctx, cache));
    instances.extend(scope::over_documents(&sections, census, ctx, cache));
    instances.extend(scope::over_documents(&fragments, census, ctx, cache));

    let coverage = Coverage::of(census, &instances);

    let mut register = register::Projection::of(declared.register);

    let mut findings: Vec<Finding> = instances
        .iter()
        .flat_map(|instance| instance.findings().iter().cloned())
        .collect();
    findings.extend(coverage.findings());
    // The register's two findings are about the taxonomy rather than about the
    // corpus, and they enter here for the reason coverage's do: neither rule
    // creates an instance, so neither accounts anything against the census.
    findings.extend(register.findings(declared.source));

    // The obligation is stamped here rather than written into each rule,
    // because the binding is data. A rule states its id, a control names that
    // id and the obligations it discharges, and one place reads the two
    // together. See [`register`] for why that place is not the check.
    let served: Vec<Serves> = [
        (
            facet_required::RULE,
            scope::document_scope::<facet_required::Required>(),
            scope::document_version::<facet_required::Required>(),
        ),
        (
            facet_value::RULE,
            scope::document_scope::<facet_value::Values>(),
            scope::document_version::<facet_value::Values>(),
        ),
        (
            placement::RULE,
            scope::document_scope::<placement::Placement>(),
            scope::document_version::<placement::Placement>(),
        ),
        (
            reciprocity::RULE,
            scope::edge_scope::<reciprocity::Reciprocity>(),
            scope::edge_version::<reciprocity::Reciprocity>(),
        ),
        (
            endpoint::RULE,
            scope::edge_scope::<endpoint::Endpoints<'_>>(),
            scope::edge_version::<endpoint::Endpoints<'_>>(),
        ),
        (
            participation::RULE,
            scope::neighbourhood_scope::<participation::Participation<'_>>(),
            scope::neighbourhood_version::<participation::Participation<'_>>(),
        ),
        (
            voice::RULE,
            scope::document_scope::<voice::Voice>(),
            scope::document_version::<voice::Voice>(),
        ),
        (
            language::RULE,
            scope::document_scope::<language::Language>(),
            scope::document_version::<language::Language>(),
        ),
        (
            sections::RULE,
            scope::document_scope::<sections::Sections>(),
            scope::document_version::<sections::Sections>(),
        ),
        (
            fragment::RULE,
            scope::document_scope::<fragment::Fragments>(),
            scope::document_version::<fragment::Fragments>(),
        ),
        (coverage::RULE, coverage::SCOPE, coverage::VERSION),
        (register::DISPOSITION, register::SCOPE, register::VERSION),
        (register::MECHANISM, register::SCOPE, register::VERSION),
    ]
    .into_iter()
    .map(|(rule, scope, version)| Serves {
        rule,
        scope,
        version,
        obligation: declared.register.bound(rule),
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

    // The filter is here, after every instance has an outcome and after the
    // obligation is stamped. So a cache holds what a check decided, an
    // inventory holds what a reader did not see, and the two cannot drift
    // ([`suppression`]).
    // The adoption payload runs first, which is the precedence spec 4 fixes:
    // waiver, then migration-pending, then suppression. A finding a task holds
    // never reaches a directive, so the two inventories partition by the order
    // of these two calls rather than by a rule checked afterwards.
    let (declared_tasks, task_refusals) = match declared.adoption {
        Some(block) => adoption::read(block, &RULES),
        None => (Vec::new(), Vec::new()),
    };
    let (findings, adoption) = adoption::apply(
        finding::sorted(findings),
        declared_tasks,
        task_refusals,
        ctx.now(),
    );

    let (declared_suppressions, refused) = suppression::declared(census, &RULES);
    let (findings, suppressions) =
        suppression::apply(findings, declared_suppressions, refused, ctx.now());

    // After both filters, because what escaped is what a reader of the findings
    // list did not see there, and the two inventories are where that is
    // recorded. They are counted apart so that the partition is checkable.
    register.escaped_from(declared.register, &suppressions);
    register.pending_from(declared.register, &adoption);

    let read_set = ReadSet::of(
        declared.lock,
        ctx.now(),
        served
            .iter()
            .map(|served| (served.rule, served.version))
            .collect(),
        &instances,
    );

    Run {
        instances,
        coverage,
        findings,
        adoption,
        suppressions,
        served,
        read_set,
        register,
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
    /// The totals alone: coverage, the instances each rule created, and what
    /// each rule serves. No finding, and no count of findings.
    ///
    /// Right for a recorded run over a corpus of *prose*, and wrong for a
    /// fixture tree. A finding of a Document-origin rule is a function of a
    /// sentence, and a sentence changes on most commits: a file that records
    /// them is a file that is re-blessed rather than read. What a regression
    /// moves is above this line — the denominator, the instance count per rule,
    /// and the obligation each rule reaches — and none of that moves when an
    /// author rewrites a paragraph.
    Totals,
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
        // Spec 7 puts the count of open pairs "beside coverage", and spec 4
        // says what it adds there: a payload that never shrinks is visible from
        // the second run rather than at its expiry.
        out.push_str(&self.adoption.render());
        // Spec 12 puts the read set here, "beside its coverage numbers". The
        // size is beside them and the union is an artifact of its own, because
        // a hash of every document is a thing a gate reads and a thing a
        // recorded report would re-bless on every edit to a paragraph.
        let _ = writeln!(out, "{}", self.read_set.summary());
        for (rule, count) in self.per_rule() {
            if count > 0 {
                let _ = writeln!(out, "  {count:5} instances of {rule}");
            }
        }

        // Spec 4 puts the suppression inventory in the coverage report, and
        // this is it. It is above the rule list rather than below the findings
        // for the reason coverage is above everything: a reader who stops here
        // has read what this run did not report as well as what it did.
        out.push_str(&self.suppressions.render());

        // What each rule sees. Spec 12 asks that the count of the barriers be a
        // number a reader can read, rather than a property discovered under
        // load, and this is that number written out per rule.
        //
        // What each rule *serves* used to be here too, one line under the
        // scope. It is in the register below now, from the obligation's side,
        // with the disposition and the control beside it. Two printings of one
        // binding is what spec 4 rules against in the declarations, and a
        // report is no different.
        out.push_str("rules, and for each the scope that binds it\n");
        for served in &self.served {
            let _ = writeln!(out, "  {}\n    {}", served.rule, served.scope.render());
        }

        // Spec 4 makes the register a projection of the two declarations, and
        // the coverage report — "what fraction of obligations are verified, by
        // severity, with the gap list" — generated from it. This is that.
        out.push_str(&self.register.render());

        if detail != Detail::Totals {
            let _ = writeln!(out, "{} findings", self.findings.len());
            for (severity, count) in self.counts() {
                if count > 0 {
                    let _ = writeln!(out, "  {count:5} {severity}");
                }
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

        if !self.findings.is_empty() && detail != Detail::Totals {
            out.push('\n');
            for finding in &self.findings {
                out.push_str(&finding.render());
            }
        }
        out
    }
}
