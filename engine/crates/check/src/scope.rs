// SPDX-License-Identifier: Apache-2.0
//! Scope: what one check may read, carried by the type it receives.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#scope--the-declaration-everything-else-rests-on)
//! gives scope four uses, and the fourth is what makes the other three true:
//! "the view exposes *only* what the scope declared. A `Document`-scoped check
//! physically cannot read a sibling." A scope that nothing enforces "would
//! silently corrupt every cache key derived from it", which is why spec 12
//! puts scope enforcement on its list of
//! [correctness roots](../../../../docs/spec/12-check-layer.md#the-correctness-roots).
//!
//! # The declaration is a type, and this module is where that holds
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-declaration-is-a-type-not-a-returned-value)
//! rules out a `scope()` method: "a scope that a check *returns* is a second
//! fact beside the argument it receives. Nothing connects them, so a check can
//! declare `Document` and still be handed a view that reads the corpus."
//!
//! One trait per scope removes the second fact, and three properties of this
//! module are what make the removal real rather than a convention:
//!
//! 1. **A view has private fields, and only this module builds one.** So a
//!    check cannot widen the view it was handed, and cannot make a wider one.
//! 2. **[`Scope`] has private fields and no public constructor.** A check
//!    cannot mint one, so the reported scope is derived from the trait rather
//!    than supplied beside it. [`document_scope`] and [`edge_scope`] are the
//!    derivation, and each one reads only its own trait.
//! 3. **The read set is the view's and never the check's.** An instance
//!    records what the view carried, so a check cannot under-report what it
//!    read. Spec 12 needs that set twice over: as the cache key, and as the
//!    read set that makes a verdict honest under merge. Each read carries the
//!    census digest of the file, because a key over a path alone would survive
//!    every edit to the document it names.
//!
//! # What a check still declares, and what it cannot
//!
//! The grain comes from the trait. `needs_body` stays a declaration, because
//! it is a real input requirement rather than a claim about the grain, and it
//! is enforced the same way: [`DocumentView::body`] returns nothing to a check
//! that did not declare it.
//!
//! `VERSION` is the other declaration, and it is the one component of a cache
//! key that no input supplies. It says which edition of a rule reached a
//! verdict, so that a change to what a rule decides invalidates the entries
//! the earlier edition wrote. Nothing can derive it: two editions of a rule
//! read the same documents and the same lock, and they differ only in code.
//! So it is raised by hand, and
//! [13 — Open obligations](../../../../docs/spec/13-open-obligations.md)
//! carries the gap that no instrument catches an author who forgets.
//!
//! `needs_clock` and `needs_prior` are absent. No check declares either one
//! today, and no view has anything to put behind them: the clock is an
//! injected value and the prior version arrives only in change-scoped
//! evaluation, which is
//! [#58](https://github.com/headwater-ai/headwater/issues/58). A field that
//! nothing enforces is the comment this module exists to delete, so each one
//! lands with the first check that needs it — and it joins the cache key in
//! the same change, because a key that omits an input is a correctness bug.
//!
//! # Where the instance set comes from
//!
//! A check is a template, and the engine instantiates it per target. The
//! generation step reads the taxonomy alone: `instantiates` is asked about a
//! kind or a relation name, never about a document. So a check cannot choose
//! its own targets out of the corpus, which is the same failure as a returned
//! scope one level up.
//!
//! Order is the census's for a document check, and the graph's edge order for
//! an edge check. Both are path order, so a run reports its instances in one
//! order and `fixtures/check.report` records it.

use crate::cache::Cache;
use crate::instance::{Input, Instance, Outcome};
use headwater_census::census::{Census, Outcome as Classification};
use headwater_census::resolve::Step;
use headwater_doc::Body;
use headwater_graph::{Direction, Edge, Graph, Target};
use headwater_yaml::Mapping;

/// A typed row that carries no document. Unreachable, and recorded rather than
/// dropped: an instance that vanishes is a document that coverage calls
/// unchecked for a reason nobody can read.
const NO_DOCUMENT: &str = "the row carries no document to read";

/// The unit one instance is created over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grain {
    /// One document: its front matter, and its body when the check asks.
    Document,
    /// One relation instance and both endpoints.
    Edge,
    /// Everything. Spec 12 calls these the barriers.
    Corpus,
}

impl Grain {
    /// The word a report prints.
    pub fn name(self) -> &'static str {
        match self {
            Grain::Document => "document",
            Grain::Edge => "edge",
            Grain::Corpus => "corpus",
        }
    }
}

/// What one check instance may read.
///
/// The fields are private and no public constructor exists, so no check
/// outside this crate can build one. That is the point: a `Scope` is derived
/// from the trait a check implements, and it is never a second fact beside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scope {
    grain: Grain,
    needs_body: bool,
}

impl Scope {
    pub(crate) const fn document(needs_body: bool) -> Self {
        Scope {
            grain: Grain::Document,
            needs_body,
        }
    }

    pub(crate) const fn edge() -> Self {
        Scope {
            grain: Grain::Edge,
            needs_body: false,
        }
    }

    pub(crate) const fn corpus() -> Self {
        Scope {
            grain: Grain::Corpus,
            needs_body: false,
        }
    }

    pub fn grain(&self) -> Grain {
        self.grain
    }

    pub fn needs_body(&self) -> bool {
        self.needs_body
    }

    /// The scope as one line of a report, in spec 12's own words for the
    /// grain. A reader who counts the barriers reads them here.
    pub fn render(&self) -> String {
        let carries = match (self.grain, self.needs_body) {
            (Grain::Document, false) => "one document and its front matter",
            (Grain::Document, true) => "one document, its front matter and its body",
            (Grain::Edge, _) => "one relation instance and both endpoints",
            (Grain::Corpus, _) => "every row of the census, and it is a barrier",
        };
        format!("{} scope, {carries}", self.grain.name())
    }
}

/// A check over one document.
pub trait DocumentCheck {
    const RULE: &'static str;
    /// Which edition of this rule reached a verdict. See the module comment:
    /// it keys the cache, and raising it is what invalidates every entry an
    /// earlier edition wrote.
    const VERSION: u32;
    /// Whether the view carries the body. A check that does not declare it
    /// receives nothing from [`DocumentView::body`], so the declaration is the
    /// access rather than a note beside it.
    const NEEDS_BODY: bool = false;

    /// The generation step: whether this template has an instance over a
    /// document of this kind. It reads the taxonomy and never the corpus.
    fn instantiates(&self, kind: &str) -> bool {
        let _ = kind;
        true
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome;
}

/// A check over one relation instance and both of its endpoints.
pub trait EdgeCheck {
    const RULE: &'static str;
    /// As [`DocumentCheck::VERSION`].
    const VERSION: u32;

    /// The generation step, as [`DocumentCheck::instantiates`], over the name
    /// of the relation an edge declares.
    fn instantiates(&self, relation: &str) -> bool {
        let _ = relation;
        true
    }

    fn evaluate(&self, view: &EdgeView<'_>) -> Outcome;
}

/// The scope of a document-scoped check, derived from its trait.
pub fn document_scope<C: DocumentCheck>() -> Scope {
    Scope::document(C::NEEDS_BODY)
}

/// The scope of an edge-scoped check, derived from its trait.
pub fn edge_scope<C: EdgeCheck>() -> Scope {
    // `C` is the whole content of this function. The trait it implements is
    // what fixes the grain, and nothing the check states can vary it.
    let _ = C::RULE;
    Scope::edge()
}

/// One document, and nothing else.
///
/// There is no accessor for a second document, for the taxonomy, or for the
/// graph. That absence is the enforcement: a check that wants a sibling has to
/// implement a wider trait, and the runner then keys its cache accordingly.
pub struct DocumentView<'a> {
    path: &'a str,
    digest: Option<&'a str>,
    kind: &'a str,
    placed_on: Option<&'a str>,
    facets: &'a Mapping,
    body: Option<&'a Body>,
}

impl<'a> DocumentView<'a> {
    /// Relative to the repository root, with `/` separators.
    pub fn path(&self) -> &'a str {
        self.path
    }

    /// The kind the census resolved for this document.
    pub fn kind(&self) -> &'a str {
        self.kind
    }

    /// The shelf whose placement carried the kind, and nothing when a
    /// discriminator in front matter carried it instead. Read back from the
    /// derivation the census recorded, so it cannot disagree with the kind.
    pub fn placed_on(&self) -> Option<&'a str> {
        self.placed_on
    }

    /// The front matter, as the author declared it.
    pub fn facets(&self) -> &'a Mapping {
        self.facets
    }

    /// The body, and only for a check that declared `NEEDS_BODY`.
    pub fn body(&self) -> Option<&'a Body> {
        self.body
    }

    /// What this view lets a check read: one document, and the hash of it.
    pub fn reads(&self) -> Vec<Input> {
        vec![Input::new(self.path, self.digest)]
    }
}

/// One relation instance, both endpoints, and the halves that declared it.
///
/// A pair is one target however many documents wrote it, which is
/// [Q4](../../../../docs/spec/09-decisions.md#q4--relation-storage)'s identity
/// for an edge: the source identifier, the relation name, and the normalized
/// target. Position in a list is not part of it.
pub struct EdgeView<'a> {
    relation: &'a str,
    declared: Option<&'a Edge>,
    inverse: Option<&'a Edge>,
    reads: Vec<Input>,
}

impl<'a> EdgeView<'a> {
    /// Build the view over the halves of one pair, or nothing when no half
    /// carries a direction. Nothing is what an empty group would produce, and
    /// this returns rather than panics for the reason a check never panics:
    /// one bad group must not silence the rest of the corpus.
    ///
    /// The digests come from the census, because the digest of a document is
    /// what the walk that read it recorded. An edge carries no bytes of its
    /// own: it is declared inside the front matter of one of its endpoints, so
    /// hashing both endpoints covers the relation name, the target and every
    /// instance attribute on it.
    fn over(halves: &[&'a Edge], digests: &Digests) -> Option<Self> {
        let declared = halves
            .iter()
            .copied()
            .find(|edge| edge.direction == Direction::AsDeclared);
        let inverse = halves
            .iter()
            .copied()
            .find(|edge| edge.direction == Direction::Inverse);

        // The read set names the declaring end first, and the declared
        // direction wins when both ends wrote their half. So one pair reads
        // the same two documents in the same order whatever the verdict is.
        let anchor = declared.or(inverse)?;
        let mut reads = vec![digests.input(&anchor.source.path)];
        if let Target::Document { path, .. } = &anchor.target {
            // A self-edge is one file at both ends, and one instance counts
            // once against one document however many times it names it.
            if !reads.iter().any(|input| &input.path == path) {
                reads.push(digests.input(path));
            }
        }

        Some(EdgeView {
            relation: anchor.declared.as_str(),
            declared,
            inverse,
            reads,
        })
    }

    /// The relation this pair declares, after a name resolved to its type.
    pub fn relation(&self) -> &'a str {
        self.relation
    }

    /// The half written from the source end, when a document wrote it.
    pub fn declared_half(&self) -> Option<&'a Edge> {
        self.declared
    }

    /// The half written from the target end, when a document wrote it.
    pub fn inverse_half(&self) -> Option<&'a Edge> {
        self.inverse
    }

    /// Both endpoints: spec 12 fixes an edge-scoped read set at "one relation
    /// instance **and both endpoints**", and coverage counts the instance
    /// against each of them.
    pub fn reads(&self) -> &[Input] {
        &self.reads
    }
}

/// The digest of each document the census read, by path.
///
/// An edge-scoped view is built from the graph, and a digest is a fact the
/// census holds. This is the one lookup between them, so that no phase invents
/// a hash for a file the walk never read.
pub struct Digests {
    /// In the census's own order, which is path order, so a lookup is a binary
    /// search and never a scan of the corpus per edge.
    by_path: Vec<(String, Option<String>)>,
}

impl Digests {
    pub fn of(census: &Census) -> Self {
        Digests {
            by_path: census
                .rows
                .iter()
                .map(|row| (row.path.clone(), row.digest.clone()))
                .collect(),
        }
    }

    /// One input. A path the census never walked carries no digest, so nothing
    /// over it is keyed, which is the same answer as a file that was not read.
    pub fn input(&self, path: &str) -> Input {
        let digest = self
            .by_path
            .binary_search_by(|(known, _)| known.as_str().cmp(path))
            .ok()
            .and_then(|index| self.by_path[index].1.as_deref());
        Input::new(path, digest)
    }
}

/// Instantiate a document-scoped check over a census.
///
/// One instance per typed document the check generates over. An untyped row
/// has no kind and so no document instance, and the census already reports it
/// with its own outcome.
pub fn over_documents<C: DocumentCheck>(
    check: &C,
    census: &Census,
    cache: &mut Cache,
) -> Vec<Instance> {
    let mut instances = Vec::new();
    for row in &census.rows {
        let Classification::Typed { kind, derivation } = &row.outcome else {
            continue;
        };
        if !check.instantiates(kind) {
            continue;
        }
        let Some(document) = &row.document else {
            instances.push(Instance::skipped(
                C::RULE,
                vec![Input::new(&row.path, row.digest.as_deref())],
                NO_DOCUMENT,
            ));
            continue;
        };

        let view = DocumentView {
            path: &row.path,
            digest: row.digest.as_deref(),
            kind,
            placed_on: derivation.steps.iter().find_map(|step| match step {
                Step::PlacementCarriesTheKind { shelf, .. } => Some(shelf.as_str()),
                _ => None,
            }),
            facets: &document.facets,
            body: match C::NEEDS_BODY {
                true => Some(&document.body),
                false => None,
            },
        };
        // The target of a document-scoped instance is the document, so the
        // path is its identity as well as its one input.
        let reads = view.reads();
        let outcome = cache.outcome(
            C::RULE,
            C::VERSION,
            document_scope::<C>(),
            &row.path,
            &reads,
            || check.evaluate(&view),
        );
        instances.push(Instance::of(C::RULE, reads, outcome));
    }
    instances
}

/// Instantiate an edge-scoped check over a graph.
///
/// One instance per declared pair the check generates over, and a pair is the
/// unit whichever end wrote it. An anchor and an unbound target are not
/// document pairs, and `declared_triple` says so by returning nothing.
pub fn over_edges<C: EdgeCheck>(
    check: &C,
    graph: &Graph,
    digests: &Digests,
    cache: &mut Cache,
) -> Vec<Instance> {
    // The edges arrive in path order, so the groups come out in a stable order
    // with no sort here.
    let mut pairs: Vec<(String, Vec<&Edge>)> = Vec::new();
    for edge in &graph.edges {
        let Some((source, relation, target)) = edge.declared_triple() else {
            continue;
        };
        if !check.instantiates(&relation) {
            continue;
        }
        let key = format!("{source}\u{1f}{relation}\u{1f}{target}");
        match pairs.iter_mut().find(|(known, _)| known == &key) {
            Some((_, halves)) => halves.push(edge),
            None => pairs.push((key, vec![edge])),
        }
    }

    let mut instances = Vec::with_capacity(pairs.len());
    for (triple, halves) in &pairs {
        let Some(view) = EdgeView::over(halves, digests) else {
            continue;
        };
        // The triple is the identity Q4 gives an edge, and it is what tells
        // two instances apart that read the same two documents. One pair of
        // documents can carry two relations, and their read sets are equal.
        let reads = view.reads().to_vec();
        let outcome = cache.outcome(
            C::RULE,
            C::VERSION,
            edge_scope::<C>(),
            triple,
            &reads,
            || check.evaluate(&view),
        );
        instances.push(Instance::of(C::RULE, reads, outcome));
    }
    instances
}
