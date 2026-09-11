// SPDX-License-Identifier: Apache-2.0
//! What every output kind does with the `name` role, stated once, as a table.
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
use headwater_check::Shape;
use headwater_generate::{plan, Identity, Kind, Plan, Projections, Runs};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_query::Surface;
use headwater_verbs::Verb;
use headwater_yaml::Mapping;
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

/// What reads each role of the closed registry, as of #539.
///
/// This is the second clause of #539's Done-when: the same question the table
/// above answers for `name`, answered for the other five. It is a record and
/// not a rule, for the reason the module doc gives.
///
/// The issue proposed that `name` is the only role with more than one consumer.
/// That is false, and the measured counts are below. What the measurement found
/// instead is a second live instance of #539's class, in the other direction:
/// **`created` is declared by no facet of this repository and read by no call
/// site of this engine.** A registry entry with zero declarations and zero
/// readers is a role a taxonomy may write and nothing will ever act on.
///
/// The counts are `Shape::facet_in_role` call sites, engine-wide, plus every
/// reader of `Pointer::name` for the `name` row. They are hand-taken and they
/// go stale; what this case holds is the *set of rows*, so a seventh role is a
/// compile-time hole here and a sixth role deleted is a failure.
const CONSUMERS: [(&str, &str); 6] = [
    (
        "name",
        "the four labelling emitters of the table above, `headwater query`, and two audit \
         readings. It reaches a reader on every rendered page",
    ),
    (
        "scent",
        "`headwater query` and the front-matter check. Two consumers, so `name` is not the \
         only role with more than one",
    ),
    ("state", "the lifecycle-state check and one audit reading"),
    (
        "state_entered",
        "the shape check and one audit reading. `STATE_LEFT` in the audit reads a role the \
         registry does not hold, which is a named wait and not a defect",
    ),
    (
        "freshness",
        "one audit reading, and nothing else. The only single-consumer role that has a \
         consumer at all",
    ),
    (
        "created",
        "nothing. No facet of this repository declares it and no call site of this engine \
         reads it. This is the finding of #539 clause 2",
    ),
];

/// The record covers the closed registry exactly.
///
/// A seventh role added to the registry and not to [`CONSUMERS`] fails here,
/// which is what makes the record answer the question for every role rather
/// than for the five somebody happened to look at.
#[test]
fn the_role_consumer_record_covers_the_closed_registry() {
    let mut recorded: Vec<&str> = CONSUMERS.iter().map(|(role, _)| *role).collect();
    recorded.sort_unstable();
    let mut registry = roles_in_rules_rs();
    registry.sort_unstable();
    assert_eq!(
        recorded, registry,
        "the role-consumer record and the closed registry name different roles. A role added \
         to the registry needs a row here saying what reads it, and a row that says `nothing` \
         is an answer"
    );
    for (role, consumers) in CONSUMERS {
        assert!(
            !consumers.is_empty(),
            "the row for `{role}` states no consumers at all. Write `nothing` and say so"
        );
    }
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
