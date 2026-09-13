// SPDX-License-Identifier: Apache-2.0
//! What every output kind does with the `name` role, and what reads each of the
//! other five, stated once, as two tables.
//!
//! [#539](https://github.com/headwater-ai/headwater/issues/539) is the class
//! behind [#427](https://github.com/headwater-ai/headwater/issues/427) and
//! [#123](https://github.com/headwater-ai/headwater/issues/123): a taxonomy
//! declares a facet in the `name` role, and an emitter that labels a document
//! reads the identifier instead. Nothing compared the closed role registry
//! against what the engine reads, so each instance was found by a reader
//! meeting `HW-DR-nnnn` on a rendered page.
//!
//! # Why a fixture and not a rule
//!
//! "An emitter must read a role" is a fact about this engine and not about any
//! document, and spec 12 rules out a verdict the engine derived from nothing a
//! corpus declared. [HW-OBL-0119](../../../../docs/obligations/0119-an-audit-reading-carries-no-declared-bar-so-a-distribution-cannot-become-a-finding.md)
//! is the obligation this records against: an audit reading carries no declared
//! bar, so a distribution cannot become a finding. The mechanism is therefore a
//! case here, where the engine is the thing under test.
//!
//! # Why an exhaustive match and not `Kind::ALL`
//!
//! `Kind::ALL` is hand-kept and its own doc-comment says a thirteenth variant
//! added without a line there compiles. [`labelling`] is an exhaustive `match`
//! for the same reason `Kind::name` and `Kind::unbuilt` are: a thirteenth
//! emitter is `E0004` rather than a silent omission. Those two are `E0004`
//! first, because they are in the crate the new variant is declared in and
//! this file is a test of it. What this file adds is a third question the
//! author has to answer in the same sitting — whether the new kind labels a
//! document. `ALL` would leave that merely untested.
//!
//! # How the reader enumeration is taken, and what it can still miss
//!
//! [`CONSUMERS`] answers #539's second clause for the other five roles, and the
//! first form of it was wrong in three rows on the day it merged. It was wrong
//! for the same reason #539 exists: the audit behind it counted call sites of
//! `Shape::facet_in_role` that name a role as a string literal, so it missed
//! every reader that resolves a role **by value** and every reader that calls
//! the **other** lookup function. `created` was recorded as read by nothing
//! while `check::participation::Participation::over` was resolving it on every
//! run of this repository's own suite.
//!
//! So the *has-a-reader* bit of each row is measured rather than written.
//! [`role_readers`] reads the engine's own source text, the way
//! [`roles_in_rules_rs`] reads `ROLES` out of `rules.rs` rather than widening an
//! interface for a test, and it takes the enumeration in three steps:
//!
//! 1. **The lookups.** A role lookup is a function or a closure that takes a
//!    `role: &str` parameter. [`role_lookups`] finds every one of them and
//!    [`the_role_lookup_functions_are_the_declared_ones`] holds that set against
//!    [`LOOKUPS`], so a third lookup function is a failure here rather than a
//!    quiet omission. [`LOOKUPS`] is also where a candidate that is not a facet
//!    role lookup is disclaimed by name and with a reason.
//! 2. **The call sites.** Every call to one of those lookups, with its argument
//!    classified: a string literal, a `const` of this engine that the scan
//!    resolves, or anything else, which is a by-value read.
//! 3. **The bit.** A role that some call site names is `Named`. A role that no
//!    call site names, where a by-value reader exists, is `ByValueOnly` — a
//!    taxonomy that declares the role gets it read and no source text of this
//!    engine mentions it. A role with neither is `Unread`.
//!
//! **Four things this still misses, and none of them is checked.** A lookup
//! whose signature is written across more than one line, because the scan reads
//! one line at a time and no signature in this engine is written that way. A
//! call site inside a `#[cfg(test)]` module, which is cut on purpose so that a
//! unit test does not report as a reader of the shipped engine. A lookup that
//! forwards its own `role` parameter to another lookup, which is a forwarding
//! rather than a reader and is skipped by the same rule that finds the
//! declarations. And, the one that matters most, **the transitive consumers**:
//! the four labelling emitters of the table above read `Pointer::name`, which
//! `query::Surface::over` filled from the `name` role, so the enumeration finds
//! the one lookup in `query` and none of the four emitters. The bit says a role
//! is reached; the prose in each row says by what, and that half is hand-taken
//! and goes stale. It went stale inside twenty four hours the first time, which
//! is [HW-OBL-0142](../../../../docs/obligations/0142-the-obligation-register-states-its-own-size-by-hand-and-it-went-stale-three-times-in-two-days.md)
//! again and is why the bit is not prose.
//!
//! # What is not here
//!
//! `no_label_a_generating_emitter_writes_is_identifier_shaped` in
//! `fixtures.rs` holds a pure-negative property — no label of this repository's
//! plan is identifier-shaped — over two emitters it names by hand, over this
//! corpus, and over identifier *shape* rather than over role reading. An
//! emitter that fell through to `shelf_index::file_name` prints
//! `00-vision-and-scope.md` and passes it. This file is the positive property
//! over every kind, on a surface built for the purpose.

use headwater_census::census::{self, Census};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Register, Run, Shape};
use headwater_generate::{plan, Identity, Kind, Plan, Projections, Runs};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_query::Surface;
use headwater_verbs::Verb;
use headwater_yaml::Mapping;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// What one output kind does with the document names it prints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Labelling {
    /// It prints a reader-facing label for a document, so it must read the
    /// `name` role and never the identifier, the path or the file name.
    ReadsTheNameRole,
    /// It prints no reader-facing label for any document. The string states
    /// what it prints instead, and why that is the right reading.
    NoDocumentLabel(&'static str),
    /// This release does not build it. `headwater_generate::unbuilt` carries
    /// the reason, and [`the_table_agrees_with_the_engine_s_own_built_split`]
    /// holds this arm against it rather than repeating any of it.
    Unbuilt,
}

/// The table. One arm per kind, and the `match` is exhaustive on purpose.
fn labelling(kind: Kind) -> Labelling {
    match kind {
        // `crate::label(pointer)` at `shelf_index.rs`, one bullet per document.
        Kind::ShelfIndex => Labelling::ReadsTheNameRole,
        // `surface.name(document)` becomes the `##` heading of each section,
        // and the emitter declines the whole file for a document it cannot
        // name. The identifier under the heading is a second string in a place
        // that has room for two, and never a fall-through.
        Kind::ShelfSections => Labelling::ReadsTheNameRole,
        // `crate::label(pointer)` at `site_nav.rs`, one leaf per document.
        Kind::SiteNav => Labelling::ReadsTheNameRole,
        // The contract cell is `[{name}]({path})`, where `name` is the
        // `name`-role value the join to the dispatch table already ran on.
        Kind::VerbIndex => Labelling::ReadsTheNameRole,
        Kind::GraphExport => Labelling::NoDocumentLabel(
            "a node carries `path`, `kind`, `id` and the whole facet map verbatim, so the \
             `name`-role value travels under its own facet key with no loss. A role reading \
             here would be a second, lossier copy of a value the consumer already holds",
        ),
        // THIS ARM IS A JUDGMENT AND NOT A MEASUREMENT
        //
        // `probe_result` is not role-blind: `probe_result.rs` reads the
        // declaration's own `name` when it mints an identifier per run. What is
        // undecided is the heading `# The result of {transcript}`, which is
        // reader-facing and names a governed document — the transcript — by its
        // path. On the reading taken here that path is an address rather than a
        // label, because the sentence under it repeats the path as one of the
        // three committed inputs a reader fetches to reproduce the file, and a
        // name would not be fetchable. On the other reading it is a label that
        // falls through to the path, and this arm should be
        // `ReadsTheNameRole`, which would fail until the emitter changed.
        //
        // Nothing here decides between the two. The `runs` fixture declares no
        // facet in the `name` role, so no case in this crate can tell them
        // apart, and that is a property of the fixture rather than of the
        // emitter. #539's verifier read this as a disclosed judgment. Whoever
        // settles it changes one word and the suite answers.
        Kind::ProbeResult => Labelling::NoDocumentLabel(
            "it names the transcript and the probes by the path a reader re-fetches, which \
             this table reads as an address rather than as a label. See the comment above: \
             the question is open and this is the reading taken",
        ),
        Kind::CorpusDescriptor => Labelling::NoDocumentLabel(
            "an entry point is `{shelf, path, id}` for a machine, and the `id` member is \
             absent where the document has none. It offers a reader no string at all",
        ),
        Kind::RelationView
        | Kind::AgentRules
        | Kind::Template
        | Kind::Transcription
        | Kind::CoverageReport => Labelling::Unbuilt,
    }
}

/// The kinds one of the two surface cases below actually runs an emitter for.
///
/// Hand-kept, and [`the_table_agrees_with_the_engine_s_own_built_split`] holds
/// it complete: a kind this table calls `ReadsTheNameRole` and no case below
/// exercises fails there. That is what stops the positive property from
/// becoming a claim about a set nobody ran.
const EXERCISED: [Kind; 4] = [
    Kind::ShelfIndex,
    Kind::ShelfSections,
    Kind::SiteNav,
    Kind::VerbIndex,
];

/// A `name`-role value that no identifier, path or file name of either fixture
/// tree carries, so a label that prints it read the role and nothing else.
const SENTINEL: &str = "The sentinel name that issue 539 asks an emitter to print";

/// The document the sentinel is written onto, by the stem of its path. An
/// output that mentions this stem mentions that document.
const SENTINEL_STEM: &str = "0002-rebuild-the-graph";

/// The same sentinel for the verb index, which joins on the `name`-role value
/// and therefore needs one the dispatch table it is handed also dispatches.
const VERB_SENTINEL: &str = "headwater sentinel-539";

const VERB_SENTINEL_STEM: &str = "headwater-check";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

/// Everything a surface borrows, owned, so that a test may build one.
struct Built {
    census: Census,
    graph: Graph,
    shape: Shape,
    taxonomy: Taxonomy,
    relations: Declarations,
    config: Config,
}

impl Built {
    fn over(corpus: &Corpus, root: &Mapping) -> Self {
        let taxonomy = Taxonomy::read(root).expect("the taxonomy reads");
        let relations = Declarations::read(root).expect("the declarations read");
        let shape = Shape::read(root).expect("the shape reads");
        let census = census::take(corpus, &taxonomy);
        let graph = Graph::build(
            &census,
            &relations,
            &Resolvers::over(corpus),
            corpus,
            &Config::default(),
        );
        Built {
            census,
            graph,
            shape,
            taxonomy,
            relations,
            config: Config::default(),
        }
    }

    fn surface(&self) -> Surface<'_> {
        Surface::over(
            &self.census,
            &self.graph,
            &self.shape,
            &self.taxonomy,
            &self.relations,
            &self.config,
        )
    }
}

fn load_map(path: &Path) -> Mapping {
    let source =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    headwater_yaml::load(&source)
        .unwrap_or_else(|errors| panic!("{}: {:?}", path.display(), errors))
        .value
        .as_map()
        .unwrap_or_else(|| panic!("{} is not a mapping", path.display()))
        .clone()
}

fn projections_from(source: &str) -> Projections {
    let root = headwater_yaml::load(source)
        .expect("the projection source loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();
    Projections::read(&root).expect("the projections read")
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the destination directory");
    for entry in std::fs::read_dir(from).expect("the source directory") {
        let entry = entry.expect("a directory entry");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("a file type").is_dir() {
            true => copy_tree(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("a copied file");
            }
        }
    }
}

/// A copy of one fixture corpus with one document's `name`-role facet rewritten
/// to a sentinel.
///
/// A copy rather than an edit in place, for the reason `fixtures.rs` gives for
/// never writing beside the inputs: a rewritten fixture would move every
/// recorded artifact of this crate, and the recorded ones are what hold the
/// emitters to their bytes. The sentinel only has to exist while these two
/// cases run.
///
/// The temporary directory is named for the case rather than for the pid,
/// because cargo runs the cases of one target as threads of one process.
fn sentinel_tree(corpus_root: &str, document: &str, old_title: &str, sentinel: &str) -> PathBuf {
    let at = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("role-reading-{corpus_root}"));
    let _ = std::fs::remove_dir_all(&at);
    copy_tree(&fixtures_dir().join(corpus_root), &at.join(corpus_root));
    let path = at.join(corpus_root).join(document);
    let source = std::fs::read_to_string(&path).expect("the document to retitle");
    let before = format!("title: {old_title}\n");
    assert!(
        source.contains(&before),
        "{} no longer states `{before}`, so this case rewrote nothing and the sentinel would \
         never be planned",
        path.display()
    );
    std::fs::write(
        &path,
        source.replace(&before, &format!("title: {sentinel}\n")),
    )
    .expect("the retitled document");
    at
}

/// Every arm of the table agrees with the engine's own built/unbuilt split, and
/// every labelling kind is exercised below.
///
/// This is what makes a thirteenth kind expensive in the right way. Adding one
/// is `E0004` in [`labelling`] first. Building an emitter for a kind this table
/// calls `Unbuilt` fails here. Calling one `ReadsTheNameRole` without adding it
/// to [`EXERCISED`] fails here too, so no arm can claim a property that no case
/// ran.
///
/// # Watched failing
///
/// Recorded in the build note: with `Kind::VerbIndex` removed from `EXERCISED`
/// this case failed naming `verb_index`, and with `Kind::SiteNav` mapped to
/// `Unbuilt` it failed naming `site_nav`.
#[test]
fn the_table_agrees_with_the_engine_s_own_built_split() {
    for kind in Kind::ALL {
        let unbuilt = headwater_generate::unbuilt(kind).is_some();
        assert_eq!(
            labelling(kind) == Labelling::Unbuilt,
            unbuilt,
            "the role-reading table and `headwater_generate::unbuilt` disagree about `{}`. The \
             engine says unbuilt={unbuilt}. A kind that has just grown an emitter needs a row \
             here saying whether it labels a document",
            kind.name()
        );
        if labelling(kind) == Labelling::ReadsTheNameRole {
            assert!(
                EXERCISED.contains(&kind),
                "`{}` is declared to read the `name` role and no case in this file runs it over \
                 the sentinel surface. A row that nothing exercises is a claim, not a property",
                kind.name()
            );
        }
    }
    for kind in EXERCISED {
        assert_eq!(
            labelling(kind),
            Labelling::ReadsTheNameRole,
            "`{}` is exercised as a labelling emitter and the table no longer says it labels",
            kind.name()
        );
    }
}

/// Every emitter that labels a document prints the document's declared name.
///
/// The surface is a copy of the `generate` fixture corpus in which
/// `decisions/0002-rebuild-the-graph.md` declares [`SENTINEL`] as its
/// `name`-role facet. That value differs from the document's identifier
/// (`DR-FIX-0002`), from its path and from its file name, so no fall-through
/// can produce it: an emitter that prints it read the role.
///
/// The property is per output rather than per emitter. Any planned output of a
/// labelling kind that mentions the document at all has to carry the sentinel
/// where it mentions it, so a second index page, a second nav or a second
/// section file is held by the same sentence.
///
/// # The decisive fixture
///
/// Revert `shelf_index.rs` from `crate::label(pointer)` to `pointer.id` and
/// this case fails naming `shelf_index` and the label it printed instead. The
/// build note records the run that watched it fail.
#[test]
fn every_emitter_that_labels_a_document_prints_its_declared_name() {
    let at = sentinel_tree(
        "generate",
        "decisions/0002-rebuild-the-graph.md",
        "Rebuild the graph on every run",
        SENTINEL,
    );
    let corpus = Corpus::new(at.clone(), "generate");
    let built = Built::over(
        &corpus,
        &load_map(&fixtures_dir().join("generate.taxonomy.yml")),
    );
    let surface = built.surface();
    // One declaration per labelling kind this tree can carry. Written with
    // explicit newlines rather than a `"\` continuation, which strips the
    // leading whitespace a YAML sequence item needs.
    let projections = projections_from(concat!(
        "projections:\n",
        "  - kind: shelf_index\n",
        "    for: [decisions, guides, archive]\n",
        "    output: \"{shelf}/README.md\"\n",
        "  - kind: shelf_sections\n",
        "    for: [decisions, guides, archive]\n",
        "    output: \"{shelf}/SECTIONS.md\"\n",
        "  - kind: site_nav\n",
        "    output: nav.yml\n",
    ));
    let plan: Plan = plan(
        &surface,
        &built.census,
        &projections,
        &Identity::default(),
        &Runs::default(),
        headwater_verbs::VERBS,
    );
    assert_sentinel_reached(
        &plan,
        SENTINEL_STEM,
        SENTINEL,
        &[Kind::ShelfIndex, Kind::ShelfSections, Kind::SiteNav],
    );
    let _ = std::fs::remove_dir_all(&at);
}

/// The verb index labels a contract with the document's declared name.
///
/// It is the one labelling emitter whose `name`-role read is also a join: the
/// value is matched against `Verb::described_as`, so the sentinel here is a
/// command line and the dispatch table handed to the emitter dispatches it.
/// The contract cell renders `[{name}]({path})`, and that `name` is the role
/// value.
///
/// A synthetic table of one verb, for the reason `verb_index.rs` gives: a case
/// that read the real table would move whenever a verb was added, which is a
/// property of the committed index rather than of the emitter.
#[test]
fn the_verb_index_labels_a_contract_with_its_declared_name() {
    let at = sentinel_tree(
        "verbs",
        "interfaces/headwater-check.md",
        "headwater check",
        VERB_SENTINEL,
    );
    let corpus = Corpus::new(at.clone(), "verbs");
    let built = Built::over(
        &corpus,
        &load_map(&fixtures_dir().join("verbs.taxonomy.yml")),
    );
    let surface = built.surface();
    let projections = projections_from(concat!(
        "projections:\n",
        "  - kind: verb_index\n",
        "    for: [interfaces]\n",
        "    output: \"verbs/interfaces/README.md\"\n",
    ));
    let verbs = [Verb {
        name: "sentinel-539",
        group: "Reading",
        summary: "the one verb the sentinel contract describes",
        description: "Synthetic, so that this case does not move when a verb is added.",
        words: &[],
    }];
    let plan: Plan = plan(
        &surface,
        &built.census,
        &projections,
        &Identity::default(),
        &Runs::default(),
        &verbs,
    );
    assert_sentinel_reached(&plan, VERB_SENTINEL_STEM, VERB_SENTINEL, &[Kind::VerbIndex]);
    let _ = std::fs::remove_dir_all(&at);
}

/// Every planned output of a labelling kind that mentions the document carries
/// the sentinel, and every named kind produced such an output.
///
/// The second half is the denominator. A property over an empty set reports as
/// a pass, and a refusal that moved every output into `plan.unwritten` would
/// empty this one silently, so the refusals are printed when nothing was found.
fn assert_sentinel_reached(plan: &Plan, stem: &str, sentinel: &str, kinds: &[Kind]) {
    let refusals: Vec<String> = plan
        .unwritten
        .iter()
        .map(|one| format!("  {} at {}: {}", one.kind.name(), one.at, one.reason))
        .collect();
    for kind in kinds {
        let mut mentions = 0usize;
        for output in plan.outputs.iter().filter(|output| output.kind == *kind) {
            if !output.bytes.contains(stem) {
                continue;
            }
            mentions += 1;
            assert!(
                output.bytes.contains(sentinel),
                "`{}` wrote `{}`, which mentions `{stem}`, and no label in it is the declared \
                 name `{sentinel}`. The emitter fell through to the identifier, the path or the \
                 file name for a document whose taxonomy declares a facet in the `name` role. \
                 The output was:\n{}",
                kind.name(),
                output.path,
                output.bytes
            );
        }
        assert!(
            mentions > 0,
            "no `{}` output of this plan mentions `{stem}`, so this case asserted nothing about \
             that emitter. It planned {} outputs and refused {}:\n{}",
            kind.name(),
            plan.outputs.len(),
            plan.unwritten.len(),
            refusals.join("\n")
        );
    }
}

/// The closed role registry has three copies in this engine, and all of them
/// have the same six entries.
///
/// `engine/crates/resolve/src/rules.rs` holds `ROLES` for the role-uniqueness
/// rule. `engine/crates/meta/meta-schema.yml` holds it twice: once as the
/// `role` enum a facet declaration is validated against, and once as the
/// `facet_role` enum of a core requirement. A role in one and not the others is
/// a role that either no source may write, or no requirement may name, or no
/// rule may check.
///
/// The `role` enum of `vocabulary_value` is a different registry — the
/// lifecycle roles `initial`, `live`, `terminal-retained` — and this case
/// separates the two by the block each enum sits in rather than by its
/// contents, which would be circular.
///
/// # Watched failing
///
/// Recorded in the build note: the first form of this case matched on the key
/// alone and read `[initial, live, terminal-retained]`, which is how the third
/// copy was found.
#[test]
fn the_three_copies_of_the_closed_role_registry_agree() {
    let registry = roles_in_rules_rs();
    let copies = roles_in_the_meta_schema();
    assert_eq!(
        copies.len(),
        2,
        "the meta-schema no longer declares the facet-role registry twice. It declared \
         {copies:?}"
    );
    for (block, roles) in copies {
        assert_eq!(
            roles, registry,
            "the copy of the closed role registry under `{block}` in the meta-schema and the \
             `ROLES` array of `rules.rs` no longer hold the same entries"
        );
    }
}

/// Whether the engine's own source reaches a role at all.
///
/// This is the half of a [`CONSUMERS`] row that is measured. The prose beside
/// it names *what* reads the role and is hand-taken; this says *whether*
/// anything does, and [`role_readers`] derives it from the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Readers {
    /// Some call site names the role: a string literal, or a `const` of this
    /// engine that the scan resolves to one.
    Named,
    /// No call site names it, and a by-value reader exists. A taxonomy that
    /// declares the role gets it read, and no source text of this engine
    /// mentions the role at all. This is the row that was recorded as
    /// `nothing` and was wrong.
    ByValueOnly,
    /// Nothing reaches it, by name or by value. A role a taxonomy may declare
    /// and nothing will ever act on.
    Unread,
}

/// What reads each role of the closed registry, as of #539.
///
/// This is the second clause of #539's Done-when: the same question the table
/// above answers for `name`, answered for the other five. It is a record and
/// not a rule, for the reason the module doc gives.
///
/// The issue proposed that `name` is the only role with more than one consumer.
/// That is false. The first form of this record then answered the clause and
/// got three of the six rows wrong, for the reason the module doc gives under
/// *How the reader enumeration is taken*. The middle column is what is held
/// against the source now; the prose is what a reader needs on top of it.
///
/// **`created` is declared by no facet of this repository, named by no call
/// site of this engine, and read all the same.** That is the finding, and the
/// shape of it is the whole of #539 one layer down: a role can be reached by a
/// lookup whose argument came from the taxonomy, so grepping the engine for the
/// role's name answers the wrong question.
const CONSUMERS: [(&str, Readers, &str); 6] = [
    (
        "name",
        Readers::Named,
        "`query::Surface::over` puts it on every `Pointer`, and the four labelling emitters of \
         the table above read the pointer rather than the role. `audit::hole` and \
         `audit::rendered` name it directly. It reaches a reader on every rendered page",
    ),
    (
        "scent",
        Readers::Named,
        "`check::frontmatter::scent_facet`, `query::Surface::over`, and \
         `generate::derived::members`, which writes it onto every generated document",
    ),
    (
        "state",
        Readers::Named,
        "`check::lifecycle_state::of`, `audit::dwell`, `resolve::rules::states` through the \
         second lookup function, and both `generate::derived::members` and \
         `generate::derived::standing`",
    ),
    (
        "state_entered",
        Readers::Named,
        "`audit::dwell` and `generate::derived::members`. Every expectation of this repository \
         also reaches it *by value*, through `check::participation::Participation::over`, which \
         resolves the expectation's own `since:` rather than a literal. `STATE_LEFT` in \
         `audit::waiting` names a role the registry does not hold, which is a named wait and \
         not a defect",
    ),
    (
        "freshness",
        Readers::Named,
        "`audit::freshness_window`, `generate::derived::members`, and \
         `resolve::rules::context_safety` through the second lookup function, which the audit \
         behind the first form of this record never looked at",
    ),
    (
        "created",
        Readers::ByValueOnly,
        "no call site of this engine names it, and it is read anyway. \
         `check::participation::Participation::over` resolves an expectation's `since:` by \
         value, so a taxonomy that declares `since: created` has its window measured from the \
         facet in this role. `the_created_role_is_read_by_the_expectation_window` is that case. \
         No facet of this repository's own taxonomy declares the role, which is why nothing \
         here exercised the path and why the first form of this row read `nothing`",
    ),
];

/// The record covers the closed registry exactly, and every row's
/// has-a-reader bit is what the engine's source says.
///
/// A seventh role added to the registry and not to [`CONSUMERS`] fails here,
/// which is what makes the record answer the question for every role rather
/// than for the five somebody happened to look at. A row that claims `Unread`
/// while a reader exists, and a row that claims a reader after the last one is
/// deleted, both fail here too, which is what stops the record going stale in
/// the direction it went stale in the first time.
///
/// # Watched failing
///
/// Recorded in the build note: with the `created` row left as it merged —
/// `Readers::Unread`, "nothing. No facet of this repository declares it and no
/// call site of this engine reads it" — this case failed naming `created`, the
/// bit it claimed and the two by-value call sites that disprove it.
#[test]
fn the_role_consumer_record_covers_the_closed_registry() {
    let mut recorded: Vec<&str> = CONSUMERS.iter().map(|(role, _, _)| *role).collect();
    recorded.sort_unstable();
    let mut registry = roles_in_rules_rs();
    registry.sort_unstable();
    assert_eq!(
        recorded, registry,
        "the role-consumer record and the closed registry name different roles. A role added \
         to the registry needs a row here saying what reads it, and a row that says nothing \
         reads it is an answer"
    );

    let readers = role_readers();
    let by_value: Vec<String> = readers
        .iter()
        .filter(|reader| reader.names.is_none())
        .map(|reader| format!("{}:{}", reader.file, reader.line))
        .collect();
    for (role, declared, consumers) in CONSUMERS {
        assert!(
            !consumers.is_empty(),
            "the row for `{role}` states no consumers at all. Write that nothing reads it and \
             say so"
        );
        let named: Vec<String> = readers
            .iter()
            .filter(|reader| reader.names.as_deref() == Some(role))
            .map(|reader| format!("{}:{}", reader.file, reader.line))
            .collect();
        let measured = match (named.is_empty(), by_value.is_empty()) {
            (false, _) => Readers::Named,
            (true, false) => Readers::ByValueOnly,
            (true, true) => Readers::Unread,
        };
        assert_eq!(
            declared, measured,
            "the row for `{role}` says {declared:?} and the engine's own source says \
             {measured:?}. Call sites that name it: {named:?}. Call sites that resolve a role \
             by value, which reach any role a taxonomy declares: {by_value:?}. The row's prose \
             may name readers this scan cannot see, and the bit may not"
        );
    }
}

/// One call site of a role lookup, and the role it names.
#[derive(Debug, Clone)]
struct Reader {
    /// The source file, relative to the repository root.
    file: String,
    /// The one-based line, so that a failure message is a place to look.
    line: usize,
    /// The lookup that was called, as [`LOOKUPS`] names it.
    lookup: &'static str,
    /// The role the call site names, or `None` where the argument is a value
    /// the taxonomy supplied. A by-value reader reaches every declared role.
    names: Option<String>,
}

/// Every function or closure of this engine that takes a `role: &str`, as
/// `(file, name, whether it looks a facet role up)`.
///
/// Hand-kept, and [`the_role_lookup_functions_are_the_declared_ones`] holds it
/// against the source. The set is mechanical; which candidates are facet-role
/// lookups is the one judgment, so it is written here by name and with a
/// reason rather than inferred.
const LOOKUPS: [(&str, &str, bool); 4] = [
    // The lookup. Everything below `Shape` reaches a role through it.
    ("engine/crates/check/src/shape.rs", "facet_in_role", true),
    // A one-line closure over `facet_in_role`, called four times in the same
    // function. The audit that produced the first form of `CONSUMERS` counted
    // `facet_in_role` call sites and saw one where there are four roles read.
    ("engine/crates/generate/src/derived.rs", "role_of", true),
    // The second lookup function, over a resolver view rather than a `Shape`.
    // It is the one the first audit's stated method could not have found.
    (
        "engine/crates/resolve/src/rules.rs",
        "facet_with_role",
        true,
    ),
    // NOT a facet-role lookup. Its `role` is a *lifecycle* role — `initial`,
    // `live`, `terminal-retained` — which is the other registry, the one
    // `the_three_copies_of_the_closed_role_registry_agree` separates by the
    // block it sits in. Counting its call sites would report readers for roles
    // that are not in the closed registry at all.
    (
        "engine/crates/resolve/src/core.rs",
        "role_is_terminal",
        false,
    ),
];

/// The set of role lookups is the declared one.
///
/// This is what makes a third facet-role lookup expensive in the right way. The
/// first form of [`CONSUMERS`] was wrong in one row because a second lookup
/// function existed and the audit's method only ever looked at the first. A
/// fifth candidate added to this engine fails here until somebody writes down
/// whether it reads a facet role.
#[test]
fn the_role_lookup_functions_are_the_declared_ones() {
    let mut declared: Vec<(String, String)> = LOOKUPS
        .iter()
        .map(|(file, name, _)| ((*file).to_string(), (*name).to_string()))
        .collect();
    declared.sort();
    let found = role_lookups(&engine_sources());
    assert_eq!(
        found, declared,
        "the functions and closures of this engine that take a `role: &str` are no longer the \
         declared ones. Add a row to `LOOKUPS` saying whether the new one looks a *facet* role \
         up or a lifecycle one, because `CONSUMERS` is measured over the facet-role ones and a \
         lookup nobody declared is a reader nothing counts"
    );

    // And every facet-role lookup has at least one call site, so that a scan
    // that silently found nothing cannot read as a scan that found no readers.
    let readers = role_readers();
    for (_, name, facet_role) in LOOKUPS {
        if !facet_role {
            continue;
        }
        assert!(
            readers.iter().any(|reader| reader.lookup == name),
            "the scan found no call site of `{name}` at all. Every row of `CONSUMERS` is \
             measured over these call sites, so a scan that stopped finding them would report \
             every role as unread and pass nothing"
        );
    }
}

/// Every `.rs` file under a `src/` directory of the engine, as
/// `(path relative to the repository root, source with its test module cut)`.
///
/// The cut is at the first `#[cfg(test)]` line. Every file scanned here has at
/// most one and it is last, by the convention rustc's own layout follows.
/// Without the cut, `check/src/shape.rs` alone would report a `state_entered`
/// reader that only its own unit test performs.
fn engine_sources() -> Vec<(String, String)> {
    let root = repository_root();
    let mut out = Vec::new();
    let mut stack = vec![root.join("engine/crates")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("an engine directory") {
            let entry = entry.expect("a directory entry");
            let path = entry.path();
            if entry.file_type().expect("a file type").is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            if !path.components().any(|part| part.as_os_str() == "src") {
                continue;
            }
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            let source = match source.find("#[cfg(test)]") {
                Some(at) => source[..at].to_string(),
                None => source,
            };
            let file = path
                .strip_prefix(&root)
                .expect("a path under the repository root")
                .to_string_lossy()
                .into_owned();
            out.push((file, source));
        }
    }
    out.sort();
    out
}

/// Every function or closure of the engine that takes a `role: &str`, as
/// `(file, name)`, sorted.
///
/// A signature written across more than one line is missed, and no signature in
/// this engine is written that way.
fn role_lookups(sources: &[(String, String)]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (file, source) in sources {
        for line in source.lines() {
            if !line.contains("role: &str") {
                continue;
            }
            // A `fn` declares its own name; a closure is bound by a `let`.
            let rest = match line.find("fn ") {
                Some(at) => &line[at + 3..],
                None => match line.find("let ") {
                    Some(at) => &line[at + 4..],
                    None => continue,
                },
            };
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if name.is_empty() {
                continue;
            }
            out.push((file.clone(), name));
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Every call site of a facet-role lookup in the engine's source, with the role
/// it names resolved.
///
/// The module doc states what this can miss and why each miss is the reading
/// taken. Nothing else in this file reads the engine's source for a role.
fn role_readers() -> Vec<Reader> {
    let sources = engine_sources();
    let constants = string_constants(&sources);
    let mut out = Vec::new();
    for (file, source) in &sources {
        for (index, line) in source.lines().enumerate() {
            // A lookup forwarding its own parameter to another lookup is a
            // forwarding and not a reader, and a declaration is the only place
            // `role: &str` is written.
            if line.contains("role: &str") {
                continue;
            }
            for (_, lookup, facet_role) in LOOKUPS {
                if !facet_role {
                    continue;
                }
                for argument in arguments_to(line, lookup) {
                    out.push(Reader {
                        file: file.clone(),
                        line: index + 1,
                        lookup,
                        names: role_named(file, &argument, &constants),
                    });
                }
            }
        }
    }
    out
}

/// The single argument of every call to `lookup` on one line.
///
/// A call with more than one argument at the top level is not a call of a
/// lookup's signature, which takes one. `resolve/src/rules.rs` holds a free
/// `role_of(states, state)` over lifecycle states that this separates from the
/// one-argument closure in `generate/src/derived.rs`.
fn arguments_to(line: &str, lookup: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let needle = format!("{lookup}(");
    let mut from = 0;
    while let Some(at) = line[from..].find(&needle) {
        let start = from + at;
        from = start + needle.len();
        // `my_facet_in_role(` is a different function with a name this one is
        // a suffix of.
        if start > 0 {
            let before = bytes[start - 1] as char;
            if before.is_ascii_alphanumeric() || before == '_' {
                continue;
            }
        }
        let mut depth = 1usize;
        let mut comma = false;
        let mut argument = String::new();
        for c in line[from..].chars() {
            match c {
                '(' | '[' => depth += 1,
                ')' | ']' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                ',' if depth == 1 => comma = true,
                _ => {}
            }
            argument.push(c);
        }
        if depth == 0 && !comma {
            out.push(argument);
        }
    }
    out
}

/// The role one argument names, or `None` where it is a value this scan cannot
/// resolve, which is a by-value read.
fn role_named(file: &str, argument: &str, constants: &Constants) -> Option<String> {
    let argument = argument.trim().trim_start_matches('&').trim();
    if let Some(inner) = argument.strip_prefix('"').and_then(|r| r.strip_suffix('"')) {
        return Some(inner.to_string());
    }
    let constant = !argument.is_empty()
        && argument
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
    if !constant {
        return None;
    }
    let key = (file.to_string(), argument.to_string());
    if let Some(value) = constants.by_file.get(&key) {
        return Some(value.clone());
    }
    match constants.by_name.get(argument) {
        Some(Some(value)) => Some(value.clone()),
        // A constant this scan cannot resolve, or one whose declarations
        // disagree, would otherwise be recorded as a by-value read, which would
        // report every role as reachable and hide exactly the finding this file
        // exists to hold.
        _ => panic!(
            "{file} passes `{argument}` to a role lookup and this scan cannot resolve it to one \
             role. Declare it as `const {argument}: &str = \"<role>\";` in the engine, or teach \
             `role_named` how to read it. Guessing here would report every role as read"
        ),
    }
}

/// Every `const NAME: &str = "value";` of the engine, per file and by name.
struct Constants {
    /// Keyed by `(file, name)`, which is how a file-local constant wins.
    by_file: BTreeMap<(String, String), String>,
    /// Keyed by name, and `None` where two declarations disagree.
    by_name: BTreeMap<String, Option<String>>,
}

fn string_constants(sources: &[(String, String)]) -> Constants {
    let mut by_file = BTreeMap::new();
    let mut by_name: BTreeMap<String, Option<String>> = BTreeMap::new();
    for (file, source) in sources {
        for line in source.lines() {
            let Some((name, value)) = string_constant(line) else {
                continue;
            };
            by_file.insert((file.clone(), name.clone()), value.clone());
            by_name
                .entry(name)
                .and_modify(|held| {
                    if held.as_deref() != Some(value.as_str()) {
                        *held = None;
                    }
                })
                .or_insert_with(|| Some(value));
        }
    }
    Constants { by_file, by_name }
}

/// One `const NAME: &str = "value";`, whatever its visibility.
fn string_constant(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    let at = line.find("const ")?;
    if !(at == 0 || line.starts_with("pub")) {
        return None;
    }
    let rest = &line[at + "const ".len()..];
    let (name, rest) = rest.split_once(':')?;
    let rest = rest.trim_start().strip_prefix("&str")?;
    let rest = rest.trim_start().strip_prefix('=')?;
    let rest = rest.trim_start().strip_prefix('"')?;
    let value = rest.split('"').next()?;
    Some((name.trim().to_string(), value.to_string()))
}

/// The clock the `created-role` tree is checked against.
///
/// As every other recorded run: a verdict is a function of the injected clock.
/// `2026-01-05` is 219 days before this date, past the 30-day window the
/// fixture expectation declares; `2026-08-01` is 11 days before it and inside.
const CREATED_ROLE_CLOCK: &str = "2026-08-12";

/// The `relation.participation.overdue` run over the `created-role` tree.
fn created_role_run() -> Run {
    let corpus = Corpus::new(fixtures_dir(), "created-role");
    let root = load_map(&fixtures_dir().join("created-role.taxonomy.yml"));
    let taxonomy = Taxonomy::read(&root).expect("the taxonomy reads");
    let declarations = Declarations::read(&root).expect("the declarations read");
    let register = Register::read(&root).expect("the register reads");
    let shape = Shape::read(&root).expect("the shape reads");
    let taken = census::take(&corpus, &taxonomy);
    let config = Config::default();
    let graph = Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(&corpus),
        &corpus,
        &config,
    );
    headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: "sha256:created-role-fixture",
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            adoption: None,
            source: "engine/crates/generate/fixtures/created-role.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(CREATED_ROLE_CLOCK).expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

/// The `created` role has a reader, and it is the expectation window.
///
/// This is the case that makes the `created` row of [`CONSUMERS`] answerable by
/// running something rather than by a reader's memory of a grep. No facet of
/// this repository's own taxonomy declares the role, so the fixture taxonomy
/// declares one: `filed` in the `created` role, `entered` in the
/// `state_entered` role, and one expectation that declares `since: created`.
///
/// # Why the two documents swap their dates
///
/// `overdue.md` and `inside-the-window.md` carry the same two dates the other
/// way round, and neither declares an edge. So the only thing that can put one
/// of them in the report and keep the other out is *which facet started the
/// window*. The correct reading reports `overdue.md`. Narrowing
/// `check::participation::Participation::over` from `&expectation.since_role`
/// to a literal `"state_entered"` — which is the `pointer.id`-instead-of-
/// `pointer.name` move of #427, one layer down — reports `inside-the-window.md`
/// and nothing else. Two readings, two disjoint answers, and no third reading
/// produces either.
///
/// `reviewed.md` carries the same dates as `overdue.md` and one declared edge,
/// so the rule generates over it and passes it. That is the passing fixture
/// [spec 12](../../../../docs/spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship)
/// asks for beside the failing one, and it is also what stops a fix that
/// silenced the report by generating nothing.
///
/// # Watched failing
///
/// Recorded in the build note: with `participation.rs` narrowed to
/// `.facet_in_role("state_entered")` this case failed, reporting
/// `inside-the-window.md` where it expects `overdue.md`.
#[test]
fn the_created_role_is_read_by_the_expectation_window() {
    const RULE: &str = headwater_check::participation::RULE;
    let run = created_role_run();

    // The denominator. Three records declare the expectation and the review
    // declares none, so a silence below is a verdict rather than an absent
    // instance.
    let instances = run
        .instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .count();
    assert_eq!(
        instances, 3,
        "every document of the kind that declares the expectation gets an instance"
    );

    let mut reported: Vec<&str> = run
        .findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .filter_map(|finding| finding.path.strip_prefix("created-role/records/"))
        .collect();
    reported.sort_unstable();
    assert_eq!(
        reported,
        vec!["overdue.md"],
        "the window is measured from the facet in the role the expectation names. \
         `inside-the-window.md` in this list is a window measured from `state_entered`, and an \
         empty list is an origin facet that resolved to nothing at all"
    );

    let message = run
        .findings
        .iter()
        .find(|finding| finding.rule == RULE && finding.path == "created-role/records/overdue.md")
        .map(|finding| finding.message.clone())
        .expect("overdue.md is reported");
    assert!(
        message.contains("219 days since `filed`"),
        "the finding counts from the `created`-role facet's own value: {message}"
    );
}

/// The registry as `engine/crates/resolve/src/rules.rs` holds it.
///
/// Read out of the source text, because `ROLES` is private to that crate and
/// making it public to satisfy a case would widen an interface for a test.
fn roles_in_rules_rs() -> Vec<String> {
    let at = repository_root().join("engine/crates/resolve/src/rules.rs");
    let source = std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("{}: {e}", at.display()));
    let start = source
        .find("const ROLES: [&str; ")
        .unwrap_or_else(|| panic!("{}: no `const ROLES` array", at.display()));
    let open = start
        + source[start..]
            .find('[')
            .expect("the array's opening bracket");
    let open = open
        + 1
        + source[open + 1..]
            .find('[')
            .expect("the initializer's opening bracket");
    let close = open
        + source[open..]
            .find(']')
            .expect("the initializer's closing bracket");
    source[open + 1..close]
        .split(',')
        .map(|entry| entry.trim().trim_matches('"').to_string())
        .filter(|entry| !entry.is_empty())
        .collect()
}

/// Every copy of the facet-role registry `engine/crates/meta/meta-schema.yml`
/// declares, as `(the block it sits in, its entries)`.
///
/// The block is tracked so that the lifecycle `role` enum of `vocabulary_value`
/// is separated from the facet-role ones by position and not by contents.
fn roles_in_the_meta_schema() -> Vec<(String, Vec<String>)> {
    let at = repository_root().join("engine/crates/meta/meta-schema.yml");
    let source = std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("{}: {e}", at.display()));
    let mut block = String::new();
    let mut out = Vec::new();
    for line in source.lines() {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim_start();
        if indent <= 2 && trimmed.ends_with(':') && !trimmed.starts_with('#') {
            block = trimmed.trim_end_matches(':').to_string();
        }
        if block == "vocabulary_value" {
            continue;
        }
        let Some(rest) = trimmed
            .strip_prefix("role: {enum: [")
            .or_else(|| trimmed.strip_prefix("facet_role: {enum: ["))
            .or_else(|| {
                trimmed
                    .find("facet_role: {enum: [")
                    .map(|at| &trimmed[at + "facet_role: {enum: [".len()..])
            })
        else {
            continue;
        };
        let close = rest.find(']').expect("the enum's closing bracket");
        out.push((
            block.clone(),
            rest[..close]
                .split(',')
                .map(|entry| entry.trim().to_string())
                .filter(|entry| !entry.is_empty())
                .collect(),
        ));
    }
    out
}
