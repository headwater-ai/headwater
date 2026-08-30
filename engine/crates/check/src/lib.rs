// SPDX-License-Identifier: Apache-2.0
//! The runner: seventeen checks, coverage against the census, a published read
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
//! Fifteen of the seventeen are **generated**. None of them names a facet, a
//! kind, a relation, an identifier scheme or a number of days: each reads a
//! declaration out of the resolved taxonomy and instantiates itself over
//! whatever that declaration produced.
//! That is what [spec 12](../../../../docs/spec/12-check-layer.md#the-five-origins-of-a-check)
//! means by "a new facet or relation in the taxonomy produces its checks with
//! no code", and it is why the rule list is short while the instance count is
//! not. [`coverage`] is the runner's own accounting. [`fragment`] and
//! [`duplicate`] read no declaration, because the language has no member that
//! turns prose-link resolution or identifier uniqueness on or off: spec 3
//! states the second of them of every corpus.
//!
//! Four of the five origins are represented. Shape, Graph and Document are
//! here, and Plugin is not. `Corpus` in that table is an *origin* — a rule
//! generated from a declaration that needs many documents — and no such
//! declaration exists yet. [`duplicate`] is corpus-*scoped* and Graph-origin,
//! which is the distinction the two lists have always drawn: the origin is what
//! a rule reads a declaration from, and the grain is what one instance covers.
//!
//! The eight Graph-origin rules span all four grains. [`target`],
//! [`reciprocity`], [`endpoint`] and [`dependency`] are edge-grained,
//! [`participation`] is neighbourhood-grained, [`duplicate`] is corpus-grained,
//! and [`declaration`] and [`identity`] are **document-grained**. The last two
//! are the ones worth stating: they route the phase-A defects that stop an edge
//! from existing, and an edge-scoped instance exists per edge, so no
//! edge-scoped rule reaches a block whose entries produced none. The origin is
//! what a rule reads and the grain is what one instance covers, and the two
//! were never one statement.
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
//! repository takes about 57 ms against about 476 ms with no cache
//! ([HW-OBL-0080](../../../../docs/obligations/0080-changed-only-is-the-content-addressed-cache-under-another-name.md)
//! holds the conditions). The cache derives what
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
//! [`run`] names the seventeen checks it runs, which is the whole of
//! registration.
//! The scope a trait fixes is now read twice: once for the report, and once as
//! a component of the cache key that spec 12 derives from the same fact. The
//! injected clock rides the same declaration, so a rule that reads a date
//! cannot be left out of its own key ([`cache`]).
//!
//! `Neighbourhood` is here, at depth 1, because
//! [`participation`] needed a grain that `Edge` does not reach: an edge-scoped
//! instance exists per edge, and a participation expectation is about an edge
//! that nobody declared. `Corpus` is here for the same kind of reason and one
//! step further out: [`duplicate`] is about two documents that no edge
//! connects, so no relational grain reaches the pair. `Shelf` is still absent,
//! and [`duplicate`] states why it could not have carried this rule either.
//! [`coverage`] is corpus-grained and is still the runner's accounting rather
//! than a check, and it now says which of the two facts about that grain was
//! the reason.
//!
//! # Three phase-A outcomes stay in phase A, and one of them could not
//!
//! Spec 12 calls an unparseable file, an unclassifiable path, a dangling edge
//! and an ambiguous shelf match *structural findings*. Three of the four are a
//! census row. The census accounts for each one with the row that names the
//! author who can act, [`coverage`] reads that census, and a document with no
//! instance is already a finding. So this runner does not re-report them: a
//! second report of one fact sends its author to two places.
//!
//! **A dangling edge is the member with no row, and that made it the member
//! with no rule.** A row is a file and an edge is not one, so nothing in the
//! coverage account reaches it. The graph printed it under its own heading and
//! it answered to no obligation, carried no severity, and left `--strict`
//! exiting 0 over a corpus whose edges pointed at nothing. [`target`] closes
//! that, and #59 supplied what it was waiting for: the base package declares
//! the obligation and the control, so the finding travels the same binding as
//! every other one.
//!
//! The rest of `headwater_graph::Problem` is routed too, and at the other
//! grain. A `relations:` block that is not a mapping, an unknown relation name,
//! an unusable entry, an entry with no `to` and a repeated triple are
//! [`declaration`]. A source document with no identifier is [`identity`], with
//! the two identifier-index defects that say the same thing from the other
//! side. Each of those stops an edge from *existing*, so the unit that survives
//! is the document that wrote the block, and the report the build already
//! produced reaches the check on the view rather than beside it.
//!
//! Every defect of phase A now reaches a rule, and the last one to arrive is
//! the one that needed a fifth grain. Two documents that claim one identifier
//! are `headwater_graph::index::Defect::Duplicate`, a document-scoped instance
//! reads one of the two, and [`duplicate`] reads the corpus. It is the first
//! corpus-scoped check this engine carries, and spec 12 calls those the
//! barriers.

pub mod adoption;
pub mod cache;
pub mod change;
pub mod context;
pub mod coverage;
pub mod declaration;
pub mod dependency;
pub mod duplicate;
pub mod endpoint;
pub mod facet_required;
pub mod facet_value;
pub mod fill;
pub mod finding;
pub mod fragment;
pub mod gate;
pub mod identifier;
pub mod identity;
pub mod instance;
pub mod language;
pub mod lifecycle_state;
pub mod paint;
pub mod participation;
pub mod patch;
pub mod placement;
pub mod promotion;
pub mod readset;
pub mod reciprocity;
pub mod register;
pub mod retention;
pub mod retired;
pub mod scope;
pub mod sections;
pub mod shape;
pub mod source_form;
pub mod suppression;
pub mod suspect;
pub mod target;
pub mod transition;
pub mod voice;

pub use adoption::Ledger;
pub use cache::Cache;
pub use context::{Context, Date};
pub use coverage::Coverage;
pub use fill::{filled, WIDTH};
pub use finding::{Finding, Severity};
pub use gate::{Recorded, Verdict};
pub use instance::{Input, Instance, Outcome};
pub use patch::Patch;
pub use readset::{ReadSet, Rule};
pub use register::{Bound, Register};
pub use scope::{
    CorpusCheck, CorpusView, DocumentCheck, DocumentView, EdgeCheck, EdgeUnit, EdgeView, Grain,
    NeighbourhoodCheck, NeighbourhoodView, Scope,
};
pub use shape::{Purpose, Shape};
pub use suppression::Inventory;

use headwater_census::census::Census;
use headwater_census::shelves::Taxonomy;
use headwater_graph::{Declarations, Graph};

/// The rules this runner carries, in the order a report lists them.
///
/// Fifteen are generated from the taxonomy, two read no declaration, one is
/// the coverage guarantee itself, and the last two are about the taxonomy
/// rather than about the corpus. A rule that is generated has no entry of its
/// own anywhere: the list is the *templates*, and the instance count is what a
/// taxonomy decides.
///
/// The order is the five origins of
/// [spec 12](../../../../docs/spec/12-check-layer.md#the-five-origins-of-a-check),
/// which is Shape, then Graph, then the runner's own accounting.
pub const RULES: [&str; 27] = [
    facet_required::RULE,
    facet_value::RULE,
    identifier::RULE,
    placement::RULE,
    target::RULE,
    suspect::RULE,
    reciprocity::RULE,
    endpoint::RULE,
    dependency::RULE,
    participation::RULE,
    declaration::RULE,
    identity::RULE,
    duplicate::RULE,
    voice::RULE,
    language::RULE,
    retired::RULE,
    source_form::RULE,
    sections::RULE,
    fragment::RULE,
    promotion::RULE,
    transition::RULE,
    lifecycle_state::RULE,
    retention::RULE,
    coverage::RULE,
    register::DISPOSITION,
    register::MECHANISM,
    adoption::RULE,
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
    /// The two front-matter keys the graph phase reads by name, and the one
    /// thing here that is not a declaration.
    ///
    /// It is here because a rule that reads an identifier has to read it from
    /// somewhere, and no declaration states where. A kind declares
    /// `identifier: {scheme: …}` and nothing names the key that holds the
    /// minted value, so [`headwater_graph::Config`] carries the guess and
    /// `.headwater/README.md` records it. The alternative was to write `id`
    /// into [`identifier`], which would put the same guess in two places and
    /// let them disagree. Taking the parameter means that settling the question
    /// changes a declaration, and that a corpus whose identifiers live under
    /// another key gets one answer from the index and the same answer from the
    /// rule.
    pub config: &'a headwater_graph::Config,
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
    /// The change this run was scoped to, and nothing for a full-corpus run.
    ///
    /// It states an input rather than a verdict, which is why it is here beside
    /// the read set: a reader who cannot see what a run was scoped to cannot
    /// reproduce it from what it printed. See [`Scoped`].
    pub change: Option<Scoped>,
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
    /// The emitter targets this rule exports to, read off the same trait as
    /// the scope and the edition. See [`scope::ExportTargets`], and
    /// [`partition`] for the rule that keeps the claim honest.
    pub exportable_as: scope::ExportTargets,
}

/// The check registry: every rule, with the scope, the edition and the export
/// targets that its declaration carries.
///
/// One list, read in three places. A rule that appears here and not in
/// [`RULES`] is a compile error, because the array is sized from it.
fn registry() -> [(&'static str, Scope, u32, scope::ExportTargets); RULES.len()] {
    [
        (
            facet_required::RULE,
            scope::document_scope::<facet_required::Required>(),
            scope::document_version::<facet_required::Required>(),
            scope::document_exports::<facet_required::Required>(),
        ),
        (
            facet_value::RULE,
            scope::document_scope::<facet_value::Values>(),
            scope::document_version::<facet_value::Values>(),
            scope::document_exports::<facet_value::Values>(),
        ),
        (
            identifier::RULE,
            scope::document_scope::<identifier::Identifier>(),
            scope::document_version::<identifier::Identifier>(),
            scope::document_exports::<identifier::Identifier>(),
        ),
        (
            placement::RULE,
            scope::document_scope::<placement::Placement>(),
            scope::document_version::<placement::Placement>(),
            scope::document_exports::<placement::Placement>(),
        ),
        (
            target::RULE,
            scope::edge_scope::<target::Targets<'_>>(),
            scope::edge_version::<target::Targets<'_>>(),
            scope::edge_exports::<target::Targets<'_>>(),
        ),
        (
            suspect::RULE,
            scope::edge_scope::<suspect::Suspect<'_>>(),
            scope::edge_version::<suspect::Suspect<'_>>(),
            scope::edge_exports::<suspect::Suspect<'_>>(),
        ),
        (
            reciprocity::RULE,
            scope::edge_scope::<reciprocity::Reciprocity>(),
            scope::edge_version::<reciprocity::Reciprocity>(),
            scope::edge_exports::<reciprocity::Reciprocity>(),
        ),
        (
            endpoint::RULE,
            scope::edge_scope::<endpoint::Endpoints<'_>>(),
            scope::edge_version::<endpoint::Endpoints<'_>>(),
            scope::edge_exports::<endpoint::Endpoints<'_>>(),
        ),
        (
            dependency::RULE,
            scope::edge_scope::<dependency::Dependency<'_>>(),
            scope::edge_version::<dependency::Dependency<'_>>(),
            scope::edge_exports::<dependency::Dependency<'_>>(),
        ),
        (
            participation::RULE,
            scope::neighbourhood_scope::<participation::Participation<'_>>(),
            scope::neighbourhood_version::<participation::Participation<'_>>(),
            scope::neighbourhood_exports::<participation::Participation<'_>>(),
        ),
        (
            declaration::RULE,
            scope::document_scope::<declaration::Unusable<'_>>(),
            scope::document_version::<declaration::Unusable<'_>>(),
            scope::document_exports::<declaration::Unusable<'_>>(),
        ),
        (
            identity::RULE,
            scope::document_scope::<identity::Identity<'_>>(),
            scope::document_version::<identity::Identity<'_>>(),
            scope::document_exports::<identity::Identity<'_>>(),
        ),
        (
            duplicate::RULE,
            scope::corpus_scope::<duplicate::Duplicate>(),
            scope::corpus_version::<duplicate::Duplicate>(),
            scope::corpus_exports::<duplicate::Duplicate>(),
        ),
        (
            voice::RULE,
            scope::document_scope::<voice::Voice>(),
            scope::document_version::<voice::Voice>(),
            scope::document_exports::<voice::Voice>(),
        ),
        (
            language::RULE,
            scope::document_scope::<language::Language>(),
            scope::document_version::<language::Language>(),
            scope::document_exports::<language::Language>(),
        ),
        (
            retired::RULE,
            scope::document_scope::<retired::Retired>(),
            scope::document_version::<retired::Retired>(),
            scope::document_exports::<retired::Retired>(),
        ),
        (
            source_form::RULE,
            scope::document_scope::<source_form::SourceForm>(),
            scope::document_version::<source_form::SourceForm>(),
            scope::document_exports::<source_form::SourceForm>(),
        ),
        (
            sections::RULE,
            scope::document_scope::<sections::Sections>(),
            scope::document_version::<sections::Sections>(),
            scope::document_exports::<sections::Sections>(),
        ),
        (
            fragment::RULE,
            scope::document_scope::<fragment::Fragments>(),
            scope::document_version::<fragment::Fragments>(),
            scope::document_exports::<fragment::Fragments>(),
        ),
        (
            promotion::RULE,
            scope::document_scope::<promotion::Promoted>(),
            scope::document_version::<promotion::Promoted>(),
            scope::document_exports::<promotion::Promoted>(),
        ),
        (
            transition::RULE,
            scope::document_scope::<transition::Transition<'_>>(),
            scope::document_version::<transition::Transition<'_>>(),
            scope::document_exports::<transition::Transition<'_>>(),
        ),
        (
            lifecycle_state::RULE,
            scope::document_scope::<lifecycle_state::StateAdmitted<'_>>(),
            scope::document_version::<lifecycle_state::StateAdmitted<'_>>(),
            scope::document_exports::<lifecycle_state::StateAdmitted<'_>>(),
        ),
        (
            retention::RULE,
            scope::corpus_scope::<retention::Retention<'_>>(),
            scope::corpus_version::<retention::Retention<'_>>(),
            scope::corpus_exports::<retention::Retention<'_>>(),
        ),
        (
            coverage::RULE,
            coverage::SCOPE,
            coverage::VERSION,
            coverage::EXPORTABLE_AS,
        ),
        (
            register::DISPOSITION,
            register::SCOPE,
            register::VERSION,
            register::EXPORTABLE_AS,
        ),
        (
            register::MECHANISM,
            register::SCOPE,
            register::VERSION,
            register::EXPORTABLE_AS,
        ),
        (
            adoption::RULE,
            adoption::SCOPE,
            adoption::VERSION,
            adoption::EXPORTABLE_AS,
        ),
    ]
}

/// The check registry, split in two for one emitter target.
///
/// [Spec 12](../../../../docs/spec/12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule)
/// asks for both halves and for neither to be authored. Both come from
/// [`declared`], so no rule can fall into both or into neither, and a rule
/// added to the registry lands in one of them without anybody remembering to
/// put it there.
#[derive(Clone, Debug)]
pub struct Partition {
    pub target: String,
    pub exported: Vec<&'static str>,
    pub unexported: Vec<&'static str>,
}

/// Split the registry for `target`, in [`RULES`] order.
pub fn partition(target: &str) -> Partition {
    let mut exported = Vec::new();
    let mut unexported = Vec::new();
    for (rule, _, _, targets) in registry() {
        match targets.contains(&target) {
            true => exported.push(rule),
            false => unexported.push(rule),
        }
    }
    Partition {
        target: target.to_string(),
        exported,
        unexported,
    }
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
    // Registration, in full: seventeen checks, each named once. The scope trait each
    // one implements decides what it is handed, so this function cannot widen
    // a view by calling the wrong instantiation.
    let required = facet_required::Required::over(declared.shape);
    let values = facet_value::Values::over(declared.shape);
    let identifiers =
        identifier::Identifier::over(declared.shape, &declared.config.identifier_facet);
    let placement = placement::Placement::over(declared.taxonomy);
    let targets = target::Targets::over(declared.relations);
    // The drift rule, over the relations an importer may write. See [`suspect`].
    let suspect = suspect::Suspect::over(declared.relations);
    let reciprocity = reciprocity::Reciprocity::over(declared.relations);
    let endpoints = endpoint::Endpoints::over(declared.relations, declared.shape);
    // A live document resting on a terminal one, over the relations whose
    // family a core requirement declares `lifecycle_sensitive`. Edge-scoped
    // because the unit is the pair: one endpoint decides nothing here. See
    // [`dependency`].
    let dependency = dependency::Dependency::over(declared.relations, declared.shape);
    let participation = participation::Participation::over(declared.shape, declared.relations);
    let declarations = declaration::Unusable::over(declared.relations, declared.shape);
    let identities = identity::Identity::over(
        declared.relations,
        declared.shape,
        &declared.config.identifier_facet,
    );
    let duplicates = duplicate::Duplicate::over(&declared.config.identifier_facet);
    let voice = voice::Voice::over(declared.shape);
    let language = language::Language::over(declared.shape);
    let retired = retired::Retired::over(declared.shape);
    let source_form = source_form::SourceForm::over(declared.shape);
    let sections = sections::Sections::over(declared.shape);
    let fragments = fragment::Fragments;
    // The one rule that declares `NEEDS_PRIOR`, and it carries no declaration:
    // the transition it reads is spec 3's act rather than a member of any
    // taxonomy. See [`promotion`].
    let promoted = promotion::Promoted;
    // The second, and this one carries a declaration: the machine it reads is
    // `regimes.lifecycle` of the resolved taxonomy. See [`transition`].
    let transitions = transition::Transition::over(declared.shape);
    // The state a document stands in, against the states the regime of its kind
    // names. Document-scoped rather than change-scoped, because the defect
    // survives in the document and a document authored at a wrong state made no
    // movement to read. See [`lifecycle_state`].
    let standing = lifecycle_state::StateAdmitted::over(declared.shape);
    // The third, and the only one about a document this corpus no longer
    // holds. Corpus-scoped because a deleted document has no census row to
    // instantiate over, and it declares the corpus-grained half of the prior
    // input. See [`retention`].
    let retention = retention::Retention::over(declared.taxonomy, declared.shape);

    let digests = scope::Digests::of(census);
    let mut instances = scope::over_documents(&required, census, graph, ctx, cache);
    instances.extend(scope::over_documents(&values, census, graph, ctx, cache));
    instances.extend(scope::over_documents(
        &identifiers,
        census,
        graph,
        ctx,
        cache,
    ));
    instances.extend(scope::over_documents(&placement, census, graph, ctx, cache));
    instances.extend(scope::over_edges(
        &targets, census, graph, &digests, ctx, cache,
    ));
    instances.extend(scope::over_edges(
        &suspect, census, graph, &digests, ctx, cache,
    ));
    instances.extend(scope::over_edges(
        &reciprocity,
        census,
        graph,
        &digests,
        ctx,
        cache,
    ));
    instances.extend(scope::over_edges(
        &endpoints, census, graph, &digests, ctx, cache,
    ));
    instances.extend(scope::over_edges(
        &dependency,
        census,
        graph,
        &digests,
        ctx,
        cache,
    ));
    instances.extend(scope::over_neighbourhoods(
        &participation,
        census,
        graph,
        &digests,
        ctx,
        cache,
    ));
    instances.extend(scope::over_documents(
        &declarations,
        census,
        graph,
        ctx,
        cache,
    ));
    instances.extend(scope::over_documents(
        &identities,
        census,
        graph,
        ctx,
        cache,
    ));
    instances.extend(scope::over_corpus(&duplicates, census, graph, ctx, cache));
    instances.extend(scope::over_documents(&voice, census, graph, ctx, cache));
    instances.extend(scope::over_documents(&language, census, graph, ctx, cache));
    instances.extend(scope::over_documents(&retired, census, graph, ctx, cache));
    instances.extend(scope::over_documents(
        &source_form,
        census,
        graph,
        ctx,
        cache,
    ));
    instances.extend(scope::over_documents(&sections, census, graph, ctx, cache));
    instances.extend(scope::over_documents(&fragments, census, graph, ctx, cache));
    instances.extend(scope::over_documents(&promoted, census, graph, ctx, cache));
    instances.extend(scope::over_documents(
        &transitions,
        census,
        graph,
        ctx,
        cache,
    ));
    instances.extend(scope::over_documents(&standing, census, graph, ctx, cache));
    instances.extend(scope::over_corpus(&retention, census, graph, ctx, cache));

    let coverage = Coverage::of(census, &instances);

    let mut register = register::Projection::of(declared.register);

    // Read here, ahead of the findings list it feeds, rather than beside
    // `adoption::apply` below. `adoption::expired` needs it to contribute
    // findings of its own into the same list the register's two do, and that
    // list is stamped with an obligation and sorted before `apply` ever sees
    // it, so a finding `expired` invents there would never be stamped.
    let declared_payload = match declared.adoption {
        Some(block) => adoption::read(block, &RULES),
        None => adoption::Declared::default(),
    };

    let mut findings: Vec<Finding> = instances
        .iter()
        .flat_map(|instance| instance.findings().iter().cloned())
        .collect();
    findings.extend(coverage.findings());
    // The register's two findings and adoption's one are about the taxonomy
    // rather than about the corpus, and they enter here for the reason
    // coverage's does: none of the three rules creates an instance, so none
    // accounts anything against the census.
    findings.extend(register.findings(declared.source));
    findings.extend(adoption::expired(
        &declared_payload,
        declared.source,
        ctx.now(),
    ));

    // The obligation is stamped here rather than written into each rule,
    // because the binding is data. A rule states its id, a control names that
    // id and the obligations it discharges, and one place reads the two
    // together. See [`register`] for why that place is not the check.
    let served: Vec<Serves> = registry()
        .into_iter()
        .map(|(rule, scope, version, exportable_as)| Serves {
            rule,
            scope,
            version,
            obligation: declared.register.bound(rule),
            exportable_as,
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
    // `declared_payload` was read above, ahead of the findings list, so that
    // `adoption::expired` could contribute to it. It moves in by value here,
    // unchanged in shape.
    let (findings, adoption) =
        adoption::apply(finding::sorted(findings), declared_payload, ctx.now());

    let (declared_suppressions, refused) = suppression::declared(census, &RULES);
    let (findings, suppressions) =
        suppression::apply(findings, declared_suppressions, refused, ctx.now());

    // After both filters, because what escaped is what a reader of the findings
    // list did not see there, and the two inventories are where that is
    // recorded. They are counted apart so that the partition is checkable.
    register.escaped_from(declared.register, &suppressions);
    register.pending_from(declared.register, &adoption);

    // Every rule as this run served it, which is where a read set takes the
    // edition and the clock declaration from. Both come off the trait, so
    // neither is a second fact beside the scope ([`scope`]).
    let rules: Vec<Rule> = served
        .iter()
        .map(|served| Rule {
            name: served.rule,
            version: served.version,
            needs_clock: served.scope.needs_clock(),
            needs_prior: served.scope.needs_prior(),
        })
        .collect();
    let read_set = ReadSet::of(declared.lock, ctx.now(), &rules, &instances);

    // What the change carried, and what the one rule that reads it made of it.
    // The promotion count is derived from the findings rather than counted
    // beside them, so the line and the findings under it cannot disagree.
    let change = ctx.change().map(|change| Scoped {
        named: change.named(),
        unmatched: change.unmatched().into_iter().map(str::to_string).collect(),
        promotions: findings
            .iter()
            .filter(|finding| finding.rule == promotion::RULE)
            .count(),
    });

    Run {
        instances,
        coverage,
        findings,
        adoption,
        suppressions,
        served,
        read_set,
        register,
        change,
        cache: cache.report(),
    }
}

/// What one run's change carried, for the report that states its own inputs.
///
/// A full-corpus run has none of this, and the absence is the statement: spec 12
/// makes the prior version available only in change-scoped evaluation, and
/// coverage already reports every instance that skipped for want of one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scoped {
    /// The documents the change named, and what became of each.
    pub named: change::Named,
    /// The paths the change named that this corpus holds no row at, in path
    /// order. Nothing is checked over one, so the report is the only place one
    /// is ever seen.
    pub unmatched: Vec<String>,
    /// Warrants that moved from `asserted` to `accepted` in this change.
    ///
    /// The count [spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#promotion-is-one-human-one-document-one-diff)
    /// asks for, and the one reading that a standing population cannot give: a
    /// bulk stamp lowers the `asserted` count exactly as the same number of
    /// real acceptances would, and it raises this one all at once.
    pub promotions: usize,
}

impl Scoped {
    /// The block a report opens with when a run was scoped to a change.
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(
            out,
            "scoped to a change: {} documents named, {} added, {} with a prior version this run \
             read",
            self.named.documents, self.named.added, self.named.carried
        );
        if self.named.unreadable > 0 {
            let _ = writeln!(
                out,
                "  {:5} prior versions did not read, and every instance over one is skipped with \
                 its reason rather than passed",
                self.named.unreadable
            );
        }
        // Above the count, because it is the line that says the count is over
        // fewer documents than the caller named. A path that reached no row is
        // checked by nothing, so no skipped instance carries it and this is the
        // only report of one.
        if !self.unmatched.is_empty() {
            let _ = writeln!(
                out,
                "  {:5} named no row of this corpus, so nothing was checked over them:",
                self.unmatched.len()
            );
            for path in &self.unmatched {
                let _ = writeln!(out, "        {path}");
            }
        }
        let _ = writeln!(
            out,
            "  {:5} promoted from `{}` to `{}`. Nothing declares how many promotions in one \
             change is too many",
            self.promotions,
            promotion::FROM,
            promotion::TO
        );
        out
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
    ///
    /// `mode` is the color decision the caller already made, threaded down to
    /// every [`Finding::render`] the same way. See `crate::paint`'s module
    /// comment for why this function reads no stream itself.
    pub fn render(&self, detail: Detail, mode: crate::paint::ColorMode) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        // The change first, because it is the input that decides which
        // instances reached a verdict at all, and coverage below counts the
        // ones that did not.
        if let Some(change) = &self.change {
            out.push_str(&change.render());
        }
        out.push_str(&self.coverage.render());
        // Spec 7 puts the count of open pairs "beside coverage", and spec 4
        // says what it adds there: a payload that never shrinks is visible from
        // the second run rather than at its expiry.
        out.push_str(&self.adoption.render(mode));
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
        let _ = writeln!(
            out,
            "{}",
            crate::paint::paint(
                crate::paint::Role::Heading,
                "rules, and for each the scope that binds it",
                mode
            )
        );
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
                    let _ = writeln!(
                        out,
                        "  {count:5} {}",
                        crate::paint::severity_word(severity, mode)
                    );
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
                out.push_str(&finding.render(mode));
            }
        }
        out
    }
}
