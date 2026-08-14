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
//!    than supplied beside it. [`document_scope`], [`edge_scope`],
//!    [`neighbourhood_scope`] and [`corpus_scope`] are the derivation, and each
//!    one reads only its own trait.
//! 3. **The read set is the view's and never the check's.** An instance
//!    records what the view carried, so a check cannot under-report what it
//!    read. Spec 12 needs that set twice over: as the cache key, and as the
//!    read set that makes a verdict honest under merge. Each read carries the
//!    census digest of the file, because a key over a path alone would survive
//!    every edit to the document it names.
//!
//! # What a check still declares, and what it cannot
//!
//! The grain comes from the trait. `NEEDS_BODY`, `NEEDS_PHASE_A` and
//! `NEEDS_CLOCK` stay declarations, because each is a real input requirement
//! rather than a claim about the grain, and all three are enforced the same
//! way: a view returns nothing to a check that did not declare the input.
//!
//! `NEEDS_PHASE_A` is the third of them and the newest.
//! [`headwater_graph::Trouble`] is what the graph build could not make of one
//! document, and a rule that routes a phase-A defect to a finding has to read
//! it. It arrives **on the view**, restricted to the one document the instance
//! is over, rather than on the check: a check that held the whole report and
//! looked its own path up in it could look a sibling's path up just as easily,
//! and that is the widening this module exists to make impossible.
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
//! # The clock is one declaration with two uses, and that is the point
//!
//! `NEEDS_CLOCK` decides two things through one value of [`Scope`]. It decides
//! whether a view carries [`crate::Context::now`], and it decides whether the
//! cache key carries the same date. The two cannot drift, because the
//! instantiation functions below bind the clock once and hand the same binding
//! to the view and to [`crate::cache`].
//!
//! That is the whole answer to the hole spec 13 recorded against the key: a
//! check that reads the clock and does not key on it serves yesterday's
//! verdict today, and the `--no-cache` differential cannot see it, because both
//! sides of that comparison hold one value of the clock.
//!
//! `needs_prior` is still absent, and it stays absent for the reason this
//! module exists. Spec 12 makes the prior version available only in
//! change-scoped evaluation, and no check declares it, so the field would be a
//! declaration that nothing enforces and nothing reads. The first transition
//! legality check is what brings it, and it copies the shape of the clock: one
//! binding, read by the view and by the key, so that no second statement of one
//! property can drift from the first.
//!
//! # Where the instance set comes from
//!
//! A check is a template, and the engine instantiates it per target. The
//! generation step reads the taxonomy alone: `instantiates` is asked about a
//! kind or a relation name, never about a document. So a check cannot choose
//! its own targets out of the corpus, which is the same failure as a returned
//! scope one level up.
//!
//! Order is the census's for a document check and for a neighbourhood check,
//! and the graph's edge order for an edge check. All three are path order, so a
//! run reports its instances in one order and `fixtures/check.report` records
//! it.

use crate::cache::Cache;
use crate::context::{Context, Date};
use crate::instance::{Input, Instance, Outcome};
use headwater_census::census::{Census, Outcome as Classification};
use headwater_census::resolve::Step;
use headwater_doc::Body;
use headwater_graph::{Direction, Edge, Graph, Target, Trouble};
use headwater_yaml::{Mapping, Span};

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
    /// One document and the documents one relation away from it.
    ///
    /// [Spec 12](../../../../docs/spec/12-check-layer.md#what-this-leaves-open)
    /// leaves the depth open and suspects the grain is unnecessary: "if no real
    /// check needs depth > 1, the correct move is to cut it and to keep `Edge`
    /// as the only relational scope". The depth is fixed at 1 here for that
    /// reason, and the sentence is answered rather than dodged in
    /// [`crate::participation`]: an `Edge` instance exists per edge, and a
    /// participation expectation is about an edge that nobody declared.
    Neighbourhood { depth: u8 },
    /// Everything. Spec 12 calls these the barriers.
    Corpus,
    /// The resolved taxonomy, and no document at all.
    ///
    /// [Spec 12](../../../../docs/spec/12-check-layer.md#the-four-scopes) draws
    /// every scope over the corpus, and it names five. A rule that reads the
    /// taxonomy rather than the corpus fits none of them, and
    /// [`crate::register`] holds two: an obligation that carries no disposition
    /// and a control whose mechanism this engine does not implement are both
    /// defects of the taxonomy, and neither has a document to point at.
    ///
    /// The grain is here rather than folded into [`Grain::Corpus`] because the
    /// two read different things. A corpus-grained rule reads every row of the
    /// census, and its verdict moves when a document moves. A taxonomy-grained
    /// one reads the lock, and its verdict moves when the lock moves. To call
    /// the second one corpus-grained would put a document in a read set that no
    /// document was ever read for.
    /// [13 — Open obligations](../../../../docs/spec/13-open-obligations.md)
    /// carries what that costs spec 12's list.
    Taxonomy,
}

impl Grain {
    /// The word a report prints.
    pub fn name(self) -> &'static str {
        match self {
            Grain::Document => "document",
            Grain::Edge => "edge",
            Grain::Neighbourhood { .. } => "neighbourhood",
            Grain::Corpus => "corpus",
            Grain::Taxonomy => "taxonomy",
        }
    }

    /// Whether an instance of this grain is **routed** to the documents it
    /// reads, which is the question [`crate::coverage`] asks of it.
    ///
    /// [Spec 4](../../../../docs/spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)
    /// states OB-COV-2 as "every classified document is routed to at least one
    /// check", and routing is the generation step: a template, a declaration,
    /// and one instance per target. The three grains above route, and their
    /// targets are documents or the edges between them. The two below do not.
    /// A corpus-grained instance exists once and its target is the corpus, so
    /// it is nobody's routing however many documents it reads.
    ///
    /// The distinction is here because the alternative deletes a rule. A
    /// corpus-scoped instance reads every document, so a coverage report that
    /// counted reading would call every document checked, and
    /// `coverage.document_unchecked` could never fire again. One rule would
    /// then declare the whole corpus covered, which is the silent pass the
    /// census exists to prevent.
    pub fn routes(self) -> bool {
        match self {
            Grain::Document | Grain::Edge | Grain::Neighbourhood { .. } => true,
            Grain::Corpus | Grain::Taxonomy => false,
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
    needs_phase_a: bool,
    needs_clock: bool,
}

impl Scope {
    pub(crate) const fn document(needs_body: bool, needs_phase_a: bool, needs_clock: bool) -> Self {
        Scope {
            grain: Grain::Document,
            needs_body,
            needs_phase_a,
            needs_clock,
        }
    }

    pub(crate) const fn edge(needs_clock: bool) -> Self {
        Scope {
            grain: Grain::Edge,
            needs_body: false,
            needs_phase_a: false,
            needs_clock,
        }
    }

    pub(crate) const fn neighbourhood(needs_clock: bool) -> Self {
        Scope {
            grain: Grain::Neighbourhood { depth: 1 },
            needs_body: false,
            needs_phase_a: false,
            needs_clock,
        }
    }

    pub(crate) const fn corpus(needs_phase_a: bool) -> Self {
        Scope {
            grain: Grain::Corpus,
            needs_body: false,
            needs_phase_a,
            needs_clock: false,
        }
    }

    pub(crate) const fn taxonomy() -> Self {
        Scope {
            grain: Grain::Taxonomy,
            needs_body: false,
            needs_phase_a: false,
            needs_clock: false,
        }
    }

    pub fn grain(&self) -> Grain {
        self.grain
    }

    pub fn needs_body(&self) -> bool {
        self.needs_body
    }

    /// Whether an instance of this scope receives what phase A could not make
    /// of its document. It joins the cache key on the same terms the body does
    /// not need to: a report about one document is a function of that
    /// document's bytes, which the read set already carries, and the flag is in
    /// the key because it changes what the instance read.
    pub fn needs_phase_a(&self) -> bool {
        self.needs_phase_a
    }

    /// Whether an instance of this scope receives the injected clock, and so
    /// whether its cache key carries one. One fact, both uses.
    pub fn needs_clock(&self) -> bool {
        self.needs_clock
    }

    /// The scope as one line of a report, in spec 12's own words for the
    /// grain. A reader who counts the barriers reads them here.
    pub fn render(&self) -> String {
        let carries = match (self.grain, self.needs_body) {
            (Grain::Document, false) => "one document and its front matter",
            (Grain::Document, true) => "one document, its front matter and its body",
            (Grain::Edge, _) => "one relation instance and both endpoints",
            (Grain::Neighbourhood { .. }, _) => {
                "one document and the documents one relation away from it"
            }
            (Grain::Corpus, _) => "every row of the census",
            (Grain::Taxonomy, _) => "the resolved taxonomy, and no document",
        };
        // Phase A's report, where the rule declared it. A reader who counts the
        // barriers has to see that this instance read one more thing than front
        // matter, and the two grains that declare it read two different sets:
        // one document's news at document grain, and the identity of every
        // document at corpus grain.
        let phase_a = match (self.needs_phase_a, self.grain) {
            (false, _) => "",
            (true, Grain::Corpus) => {
                ", and what phase A could not make of each document's identity"
            }
            (true, _) => ", and what phase A could not make of it",
        };
        // The clock is named because it is an input like any other, and because
        // spec 12 puts it in the cache key. A reader who asks why a warm run
        // re-evaluated one rule and not another reads the answer here.
        let clock = match self.needs_clock {
            true => ", and the injected clock",
            false => "",
        };
        // Spec 12 calls the corpus-scoped checks the barriers, and the word is
        // last so that it reads as a statement about the scope rather than
        // about the inputs listed before it.
        let barrier = match self.grain {
            Grain::Corpus => ", and it is a barrier",
            _ => "",
        };
        format!(
            "{} scope, {carries}{phase_a}{clock}{barrier}",
            self.grain.name()
        )
    }
}

/// The emitter targets a check exports to, and the empty set is the common
/// value.
///
/// [Spec 12](../../../../docs/spec/12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule)
/// states the bar a name in this set has to clear: the emitted constraint
/// catches exactly what the native check catches, in both directions. A
/// construct that misses a document the check reports is a partial
/// translation, and a construct that rejects a document the check accepts is
/// worse, because a loss set cannot record it. A differential test is what
/// permits a value here, and `engine/crates/generate/tests/differential.rs`
/// is that test.
///
/// The declaration sits on the check rather than in a table beside it, for the
/// reason the scope does. A second list of rules is a list that drifts.
pub type ExportTargets = &'static [&'static str];

/// A check over one document.
pub trait DocumentCheck {
    const RULE: &'static str;
    /// Which edition of this rule reached a verdict. See the module comment:
    /// it keys the cache, and raising it is what invalidates every entry an
    /// earlier edition wrote.
    const VERSION: u32;
    /// The emitter targets this check exports to. See [`ExportTargets`]. The
    /// default is the empty set, because most checks reach nothing a schema
    /// language can say, and a default of "unexported" cannot claim coverage
    /// by accident.
    const EXPORTABLE_AS: ExportTargets = &[];
    /// Whether the view carries the body. A check that does not declare it
    /// receives nothing from [`DocumentView::body`], so the declaration is the
    /// access rather than a note beside it.
    const NEEDS_BODY: bool = false;
    /// Whether the view carries what phase A could not make of this document,
    /// on the same terms: a check that does not declare it receives nothing
    /// from [`DocumentView::phase_a`].
    ///
    /// It is what a rule that routes a structural finding needs, and it is
    /// deliberately not a handle on the graph. The view carries the report for
    /// the one document the instance is over and no other, so a rule that
    /// reports a phase-A defect still cannot read a sibling.
    const NEEDS_PHASE_A: bool = false;
    /// Whether the view carries the injected clock, on the same terms and with
    /// one more consequence: it joins the cache key.
    const NEEDS_CLOCK: bool = false;

    /// The generation step: whether this template has an instance over a
    /// document of this kind. It reads the taxonomy and never the corpus.
    fn instantiates(&self, kind: &str) -> bool {
        let _ = kind;
        true
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome;
}

/// What one edge-scoped instance is created over.
///
/// Both members are one relation instance, which is the grain
/// [spec 12](../../../../docs/spec/12-check-layer.md#the-four-scopes) fixes for
/// this trait. They differ over what counts as one, and the difference is
/// whether the target resolved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeUnit {
    /// The pair that
    /// [Q4](../../../../docs/spec/09-decisions.md#q4--relation-storage)
    /// identifies: a source identifier, a relation, and a target document. Two
    /// reciprocal halves are one instance of it, and an edge whose target is
    /// not a document is no instance at all, because there is no second
    /// endpoint to read.
    Pair,
    /// One entry of one `relations:` block, whatever its target became.
    ///
    /// A rule about the target *string* has to reach an entry whose string
    /// bound to nothing, and a pair cannot carry one. A target that no document
    /// answers to has no far end, so nothing at that end can have written the
    /// other half. Two halves of one bound pair are two instances here, and
    /// correctly so: they are two strings, written by two authors, in two
    /// files, and each one is right or wrong on its own.
    Entry,
}

/// A check over one relation instance and both of its endpoints.
pub trait EdgeCheck {
    const RULE: &'static str;
    /// As [`DocumentCheck::VERSION`].
    const VERSION: u32;
    /// As [`DocumentCheck::EXPORTABLE_AS`].
    const EXPORTABLE_AS: ExportTargets = &[];
    /// As [`DocumentCheck::NEEDS_CLOCK`].
    const NEEDS_CLOCK: bool = false;
    /// What one instance of this check covers. See [`EdgeUnit`].
    ///
    /// It is a declaration on the trait rather than an argument that the runner
    /// passes, for the reason the grain is a trait rather than a returned
    /// value: a unit that the caller chose is a second fact beside the check,
    /// and nothing holds the two together. It stays off [`Scope`] because it
    /// selects which instances exist rather than what one instance may read,
    /// and what one instance may read is the whole of what a scope states and
    /// what a cache key covers.
    const UNIT: EdgeUnit = EdgeUnit::Pair;

    /// The generation step, as [`DocumentCheck::instantiates`], over the name
    /// of the relation an edge declares.
    fn instantiates(&self, relation: &str) -> bool {
        let _ = relation;
        true
    }

    fn evaluate(&self, view: &EdgeView<'_>) -> Outcome;
}

/// A check over the whole corpus: spec 12's barrier.
///
/// There is no `instantiates`, and the absence is the grain. Every other trait
/// here generates one instance per target out of a declaration, and this one
/// has a single target that no declaration selects. So the count of the
/// barriers is the count of the corpus-scoped rules, which is the number
/// [spec 12](../../../../docs/spec/12-check-layer.md#scope--the-declaration-everything-else-rests-on)
/// asks a reader to be able to read rather than discover under load.
///
/// A rule belongs here when its subject is a relation between two documents
/// that no edge connects. Two documents that claim one identifier are the
/// case: nothing links them, so [`Grain::Neighbourhood`] does not reach them,
/// and a document-scoped instance reads one of the two, so its key survives
/// every edit to the other one.
pub trait CorpusCheck {
    const RULE: &'static str;
    /// As [`DocumentCheck::VERSION`].
    const VERSION: u32;
    /// As [`DocumentCheck::EXPORTABLE_AS`].
    const EXPORTABLE_AS: ExportTargets = &[];
    /// Whether the view carries what phase A could not make of the identity of
    /// any document, on [`DocumentCheck::NEEDS_PHASE_A`]'s terms.
    ///
    /// The document-grained flag hands a rule the report about its own file and
    /// nothing else, which is what keeps that grain honest. There is no such
    /// restriction to make here: this scope reads the corpus, so the report it
    /// receives is the corpus-wide one.
    const NEEDS_PHASE_A: bool = false;

    fn evaluate(&self, view: &CorpusView<'_>) -> Outcome;
}

/// A check over one document and the documents one relation away from it.
pub trait NeighbourhoodCheck {
    const RULE: &'static str;
    /// As [`DocumentCheck::VERSION`].
    const VERSION: u32;
    /// As [`DocumentCheck::EXPORTABLE_AS`].
    const EXPORTABLE_AS: ExportTargets = &[];
    /// As [`DocumentCheck::NEEDS_CLOCK`].
    const NEEDS_CLOCK: bool = false;

    /// The generation step, as [`DocumentCheck::instantiates`], over the kind
    /// of the document at the centre.
    fn instantiates(&self, kind: &str) -> bool {
        let _ = kind;
        true
    }

    fn evaluate(&self, view: &NeighbourhoodView<'_>) -> Outcome;
}

/// The scope of a document-scoped check, derived from its trait.
pub fn document_scope<C: DocumentCheck>() -> Scope {
    Scope::document(C::NEEDS_BODY, C::NEEDS_PHASE_A, C::NEEDS_CLOCK)
}

/// The scope of an edge-scoped check, derived from its trait.
pub fn edge_scope<C: EdgeCheck>() -> Scope {
    Scope::edge(C::NEEDS_CLOCK)
}

/// The scope of a neighbourhood-scoped check, derived from its trait.
pub fn neighbourhood_scope<C: NeighbourhoodCheck>() -> Scope {
    Scope::neighbourhood(C::NEEDS_CLOCK)
}

/// The scope of a corpus-scoped check, derived from its trait.
pub fn corpus_scope<C: CorpusCheck>() -> Scope {
    Scope::corpus(C::NEEDS_PHASE_A)
}

/// The edition of a document-scoped check, derived from its trait.
///
/// A report reads this the way it reads the scope, and for the same reason:
/// [`crate::ReadSet`] publishes the version of every rule that ran, and a
/// version written beside a rule rather than read off it is a second fact that
/// nothing holds to the first.
pub fn document_version<C: DocumentCheck>() -> u32 {
    C::VERSION
}

/// The edition of an edge-scoped check, derived from its trait.
pub fn edge_version<C: EdgeCheck>() -> u32 {
    C::VERSION
}

/// The edition of a neighbourhood-scoped check, derived from its trait.
pub fn neighbourhood_version<C: NeighbourhoodCheck>() -> u32 {
    C::VERSION
}

/// The edition of a corpus-scoped check, derived from its trait.
pub fn corpus_version<C: CorpusCheck>() -> u32 {
    C::VERSION
}

/// The export targets of a document-scoped check, derived from its trait.
pub fn document_exports<C: DocumentCheck>() -> ExportTargets {
    C::EXPORTABLE_AS
}

/// The export targets of an edge-scoped check, derived from its trait.
pub fn edge_exports<C: EdgeCheck>() -> ExportTargets {
    C::EXPORTABLE_AS
}

/// The export targets of a neighbourhood-scoped check, derived from its trait.
pub fn neighbourhood_exports<C: NeighbourhoodCheck>() -> ExportTargets {
    C::EXPORTABLE_AS
}

/// The export targets of a corpus-scoped check, derived from its trait.
pub fn corpus_exports<C: CorpusCheck>() -> ExportTargets {
    C::EXPORTABLE_AS
}

/// The clock a check of this scope receives, and nothing for one that did not
/// declare it.
///
/// One function, called once per instantiation, and its result goes to the view
/// and to the cache key. That is what makes the two incapable of disagreeing.
fn clock_for(scope: Scope, ctx: &Context) -> Option<Date> {
    match scope.needs_clock() {
        true => Some(ctx.now()),
        false => None,
    }
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
    phase_a: Option<Trouble<'a>>,
    clock: Option<Date>,
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

    /// What phase A could not make of **this** document, and only for a check
    /// that declared `NEEDS_PHASE_A`.
    ///
    /// It is the graph build's own answer rather than a second reading of the
    /// same front matter. See [`headwater_graph::Trouble`] for why a check that
    /// re-derived it would be a second definition of one defect.
    pub fn phase_a(&self) -> Option<&Trouble<'a>> {
        self.phase_a.as_ref()
    }

    /// The injected date, and only for a check that declared `NEEDS_CLOCK`.
    pub fn now(&self) -> Option<Date> {
        self.clock
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
    clock: Option<Date>,
    reads: Vec<Input>,
    resolution: String,
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
    fn over(halves: &[&'a Edge], digests: &Digests, clock: Option<Date>) -> Option<Self> {
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
            clock,
            reads,
            resolution: anchor.target.resolution(),
        })
    }

    /// What the target of this edge bound to, for the cache key and for
    /// nothing else.
    ///
    /// It is taken from the same half the read set is taken from, so one
    /// instance names one binding however many documents wrote a half of it.
    /// At [`EdgeUnit::Pair`] both halves are documents by construction, and
    /// both are in the read set above. At [`EdgeUnit::Entry`] the half is the
    /// entry, and this is the only component of the key that names what its
    /// target string reached.
    fn resolution(&self) -> &str {
        &self.resolution
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

    /// The injected date, and only for a check that declared `NEEDS_CLOCK`.
    pub fn now(&self) -> Option<Date> {
        self.clock
    }

    /// Both endpoints: spec 12 fixes an edge-scoped read set at "one relation
    /// instance **and both endpoints**", and coverage counts the instance
    /// against each of them.
    pub fn reads(&self) -> &[Input] {
        &self.reads
    }
}

/// Which end of a declared relation a document sits at.
///
/// An author may write either half, so the name in front matter does not say
/// this. The direction the graph resolved does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    Source,
    Target,
}

/// One document one relation away, and the relation that reaches it.
#[derive(Clone, Copy, Debug)]
pub struct Neighbour<'a> {
    /// The relation type, after a name resolved to it. Never the name an author
    /// wrote, because the two halves of one pair write two different names.
    pub relation: &'a str,
    /// Which end of that relation the document at the centre sits at.
    pub end: End,
    pub path: &'a str,
    pub kind: &'a str,
    pub id: &'a str,
    /// The entry that declared this edge, wherever it was written.
    pub span: Span,
}

/// One document, and the documents one relation away from it.
///
/// The centre is a full document view. A neighbour is an identity and a kind
/// and nothing more: no front matter, no body. That is the depth-1 boundary,
/// and it is what stops this grain from becoming a corpus view with extra
/// steps.
pub struct NeighbourhoodView<'a> {
    path: &'a str,
    kind: &'a str,
    facets: &'a Mapping,
    neighbours: Vec<Neighbour<'a>>,
    clock: Option<Date>,
    reads: Vec<Input>,
}

impl<'a> NeighbourhoodView<'a> {
    pub fn path(&self) -> &'a str {
        self.path
    }

    pub fn kind(&self) -> &'a str {
        self.kind
    }

    /// The front matter of the document at the centre, as its author wrote it.
    pub fn facets(&self) -> &'a Mapping {
        self.facets
    }

    /// Every document one relation away, without repeats, in the graph's edge
    /// order.
    pub fn neighbours(&self) -> &[Neighbour<'a>] {
        &self.neighbours
    }

    /// The injected date, and only for a check that declared `NEEDS_CLOCK`.
    pub fn now(&self) -> Option<Date> {
        self.clock
    }

    /// The centre and every neighbour. A neighbour's kind is a fact its own
    /// front matter decided, so an edit to that file changes what this instance
    /// decided and has to invalidate it.
    pub fn reads(&self) -> &[Input] {
        &self.reads
    }
}

/// The corpus, as the one rule that reads it needs it.
///
/// The read set is every census row that **carries a document**, and that set
/// is the derivation rather than a choice. `Index::build` opens no file: it
/// reads the identifier facet of each row that carries a document, and a row
/// that carries none contributed nothing and could not have. A row the walk
/// never read has no digest either, and an input with no digest is one
/// [`crate::cache`] refuses to key at all, so a read set that held every row
/// would leave this instance permanently unkeyed and the barrier permanently
/// re-evaluated.
///
/// Every transition into and out of that set moves the key. A file that gains
/// front matter gains a row with a document. One that loses it leaves the set.
/// A new file arrives as a new input. An exclusion pattern is in the lock, and
/// the lock digest is a component of every key. So there is no edit to this
/// corpus that changes what this rule decides and leaves its key where it was.
pub struct CorpusView<'a> {
    identity: Option<&'a [headwater_graph::index::Reported]>,
    reads: Vec<Input>,
}

impl<'a> CorpusView<'a> {
    /// What the identifier index could not make of any document, and only for
    /// a check that declared `NEEDS_PHASE_A`.
    ///
    /// It is the build's own answer rather than a second reading of the same
    /// front matter, which is [`DocumentView::phase_a`]'s reason and one more
    /// besides. Whether two documents claim one identifier is a judgment over
    /// both shelves of the index in one order, and a rule that re-derived it
    /// from the corpus would be a second definition of the defect that decides
    /// which of the two the graph already bound every edge to.
    pub fn identity(&self) -> Option<&'a [headwater_graph::index::Reported]> {
        self.identity
    }

    /// Every document this view was built over. See the type comment for why
    /// the set is the rows that carry a document rather than every row.
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
///
/// The graph is here for one reason: a check that declared `NEEDS_PHASE_A`
/// receives what the build could not make of *its* document. A row the census
/// classified as generated is not a typed row, so no instance is created over
/// one and no phase-A report about one reaches a rule. That is spec 6's
/// exemption holding at this grain rather than a second decision here: the
/// content of a generated file is a function of its emitter, and
/// `generate --check` is what holds it.
pub fn over_documents<C: DocumentCheck>(
    check: &C,
    census: &Census,
    graph: &Graph,
    ctx: &Context,
    cache: &mut Cache,
) -> Vec<Instance> {
    let scope = document_scope::<C>();
    // Bound once. The same value reaches the view and the key, which is what
    // makes a clock-reading check impossible to leave out of its own key.
    let clock = clock_for(scope, ctx);
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
                Grain::Document,
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
            // Built per row and only where the trait asked for it, so a rule
            // that did not declare the input pays nothing and receives nothing.
            phase_a: match C::NEEDS_PHASE_A {
                true => Some(graph.about(&row.path)),
                false => None,
            },
            clock,
        };
        // The target of a document-scoped instance is the document, so the
        // path is its identity as well as its one input.
        let reads = view.reads();
        // No resolution: a document-scoped instance reads one document, and a
        // document is a corpus path that the read set above already names.
        let outcome = cache.outcome(
            C::RULE,
            C::VERSION,
            scope,
            &row.path,
            &reads,
            clock,
            None,
            || check.evaluate(&view),
        );
        instances.push(Instance::of(C::RULE, Grain::Document, reads, outcome));
    }
    instances
}

/// Instantiate an edge-scoped check over a graph.
///
/// The unit comes off the trait, and it decides what one instance covers.
/// [`EdgeUnit::Pair`] groups the two reciprocal halves of one Q4 triple and
/// reaches no edge whose target is not a document, because `declared_triple`
/// returns nothing for one. [`EdgeUnit::Entry`] takes every declared edge as
/// written, which is the only unit that reaches a target that bound to
/// nothing.
///
/// One function rather than two, because the grouping is the only difference
/// and everything after it — the view, the read set, the key, the cache — has
/// to be the same for both. Two functions is where the two would drift.
pub fn over_edges<C: EdgeCheck>(
    check: &C,
    graph: &Graph,
    digests: &Digests,
    ctx: &Context,
    cache: &mut Cache,
) -> Vec<Instance> {
    let scope = edge_scope::<C>();
    let clock = clock_for(scope, ctx);
    // The edges arrive in path order, so the groups come out in a stable order
    // with no sort here.
    let mut pairs: Vec<(String, Vec<&Edge>)> = Vec::new();
    for edge in &graph.edges {
        let Some(key) = (match C::UNIT {
            EdgeUnit::Pair => edge
                .declared_triple()
                .and_then(
                    |(source, relation, target)| match check.instantiates(&relation) {
                        true => Some(format!("{source}\u{1f}{relation}\u{1f}{target}")),
                        false => None,
                    },
                ),
            // The authored triple, which the graph already holds to be unique
            // inside one document: a repeated one is `Problem::RepeatedTriple`
            // and declares no second edge. The name is the one an author wrote
            // rather than the relation it resolves to, because two halves of
            // one pair are two entries at this unit and they carry two names.
            EdgeUnit::Entry => match check.instantiates(&edge.declared) {
                true => {
                    let (source, name, target) = edge.triple();
                    Some(format!("{source}\u{1f}{name}\u{1f}{target}"))
                }
                false => None,
            },
        }) else {
            continue;
        };
        match pairs.iter_mut().find(|(known, _)| known == &key) {
            Some((_, halves)) => halves.push(edge),
            None => pairs.push((key, vec![edge])),
        }
    }

    let mut instances = Vec::with_capacity(pairs.len());
    for (triple, halves) in &pairs {
        let Some(view) = EdgeView::over(halves, digests, clock) else {
            continue;
        };
        // The triple is the identity Q4 gives an edge, and it is what tells
        // two instances apart that read the same two documents. One pair of
        // documents can carry two relations, and their read sets are equal.
        let reads = view.reads().to_vec();
        // The one scope that reaches a resolver. See [`crate::cache`]: the
        // identity of an anchor edge is the same string on both sides of the
        // change that falsifies its verdict, so the binding is named in the key
        // beside it.
        let outcome = cache.outcome(
            C::RULE,
            C::VERSION,
            scope,
            triple,
            &reads,
            clock,
            Some(view.resolution()),
            || check.evaluate(&view),
        );
        instances.push(Instance::of(C::RULE, Grain::Edge, reads, outcome));
    }
    instances
}

/// Instantiate a neighbourhood-scoped check over a census and a graph.
///
/// One instance per typed document the check generates over, as at document
/// grain. What differs is the view and so the read set: the centre, plus every
/// document one relation away from it.
pub fn over_neighbourhoods<C: NeighbourhoodCheck>(
    check: &C,
    census: &Census,
    graph: &Graph,
    digests: &Digests,
    ctx: &Context,
    cache: &mut Cache,
) -> Vec<Instance> {
    let scope = neighbourhood_scope::<C>();
    let clock = clock_for(scope, ctx);
    let adjacency = Adjacency::of(graph);

    let mut instances = Vec::new();
    for row in &census.rows {
        let Classification::Typed { kind, .. } = &row.outcome else {
            continue;
        };
        if !check.instantiates(kind) {
            continue;
        }
        let Some(document) = &row.document else {
            instances.push(Instance::skipped(
                C::RULE,
                Grain::Neighbourhood { depth: 1 },
                vec![Input::new(&row.path, row.digest.as_deref())],
                NO_DOCUMENT,
            ));
            continue;
        };

        let neighbours = adjacency.of_path(&row.path);
        let mut reads = vec![Input::new(&row.path, row.digest.as_deref())];
        for neighbour in &neighbours {
            if !reads.iter().any(|input| input.path == neighbour.path) {
                reads.push(digests.input(neighbour.path));
            }
        }

        let view = NeighbourhoodView {
            path: &row.path,
            kind,
            facets: &document.facets,
            neighbours,
            clock,
            reads: reads.clone(),
        };
        // No resolution either. [`Adjacency`] holds only the neighbours whose
        // target is a document, so this grain reaches no anchor and every input
        // it read is a path of the read set above.
        let outcome = cache.outcome(
            C::RULE,
            C::VERSION,
            scope,
            &row.path,
            &reads,
            clock,
            None,
            || check.evaluate(&view),
        );
        instances.push(Instance::of(
            C::RULE,
            Grain::Neighbourhood { depth: 1 },
            reads,
            outcome,
        ));
    }
    instances
}

/// The target of the one instance a corpus-scoped check creates.
///
/// A target tells two instances apart that read the same documents. There is
/// one instance of a corpus-scoped rule, so this is a constant, and it is in
/// the key for the reason every other target is: it says what the instance was
/// about, and no read set says that.
const CORPUS: &str = "the corpus";

/// Instantiate a corpus-scoped check over a census and a graph.
///
/// One instance, whatever the corpus holds. That is the count spec 12 calls the
/// barrier count, and it is the tell that separates this grain from a
/// document-scoped rule with a wide read set: the instance count moves by one
/// and the read set moves by the size of the corpus.
pub fn over_corpus<C: CorpusCheck>(
    check: &C,
    census: &Census,
    graph: &Graph,
    cache: &mut Cache,
) -> Vec<Instance> {
    let scope = corpus_scope::<C>();
    let reads: Vec<Input> = census
        .rows
        .iter()
        .filter(|row| row.document.is_some())
        .map(|row| Input::new(&row.path, row.digest.as_deref()))
        .collect();
    let view = CorpusView {
        identity: match C::NEEDS_PHASE_A {
            true => Some(&graph.index.defects),
            false => None,
        },
        reads: reads.clone(),
    };
    // No clock. `CorpusCheck` declares none, so `clock_for` would have nothing
    // to bind and the key would carry nothing about a day. The first
    // corpus-scoped rule that reads a date brings the declaration with it, on
    // the terms the other three traits already state.
    let outcome = cache.outcome(C::RULE, C::VERSION, scope, CORPUS, &reads, None, None, || {
        check.evaluate(&view)
    });
    vec![Instance::of(C::RULE, Grain::Corpus, reads, outcome)]
}

/// Every document one relation away from each document, built once per run.
///
/// A pair that both ends declared produces two edges, and both of them reach
/// the same two documents by the same relation. So the entries are deduplicated
/// on the identity that a check reads — the relation, the end, and the far
/// document — and never on the edge, which would count one neighbour twice.
struct Adjacency<'a> {
    by_path: Vec<(&'a str, Vec<Neighbour<'a>>)>,
}

impl<'a> Adjacency<'a> {
    fn of(graph: &'a Graph) -> Self {
        let mut adjacency = Adjacency {
            by_path: Vec::new(),
        };
        for edge in &graph.edges {
            let Target::Document { id, path, kind } = &edge.target else {
                continue;
            };
            // The direction says which end of the *declared* relation each
            // document sits at, whichever name its author reached for.
            let (near, far) = match edge.direction {
                Direction::AsDeclared => (End::Source, End::Target),
                Direction::Inverse => (End::Target, End::Source),
            };
            adjacency.push(
                &edge.source.path,
                Neighbour {
                    relation: &edge.declared,
                    end: near,
                    path,
                    kind,
                    id,
                    span: edge.span,
                },
            );
            adjacency.push(
                path,
                Neighbour {
                    relation: &edge.declared,
                    end: far,
                    path: &edge.source.path,
                    kind: &edge.source.kind,
                    id: &edge.source.id,
                    span: edge.span,
                },
            );
        }
        adjacency
    }

    fn push(&mut self, path: &'a str, neighbour: Neighbour<'a>) {
        let entry = match self.by_path.iter_mut().find(|(known, _)| *known == path) {
            Some(entry) => entry,
            None => {
                self.by_path.push((path, Vec::new()));
                self.by_path.last_mut().expect("just pushed")
            }
        };
        if !entry.1.iter().any(|known| {
            known.relation == neighbour.relation
                && known.end == neighbour.end
                && known.path == neighbour.path
        }) {
            entry.1.push(neighbour);
        }
    }

    fn of_path(&self, path: &str) -> Vec<Neighbour<'a>> {
        self.by_path
            .iter()
            .find(|(known, _)| *known == path)
            .map(|(_, neighbours)| neighbours.clone())
            .unwrap_or_default()
    }
}
