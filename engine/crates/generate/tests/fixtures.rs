// SPDX-License-Identifier: Apache-2.0
//! What the verb writes, what it refuses to write, and what it says about both.
//!
//! One recorded file holds the whole of it: the bytes of every generated
//! artifact, then the report of five runs over three different trees. The runs
//! are the branches that matter and each one is here for a property.
//!
//! 1. A write into an empty tree. Every output is new.
//! 2. A second write over the first. The bytes do not move, which is what makes
//!    `--check` a gate rather than a coin toss.
//! 3. `--check` over that tree. Everything is unchanged.
//! 4. `--check` over an empty tree. Everything is missing, and the run fails.
//! 5. A write into a tree where an authored file already holds one of the output
//!    paths. That one is refused and the authored bytes are still there
//!    afterwards, which is the property spec 6 gives the marker.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-generate --test fixtures
//!
//! Read the diff before committing it. A blessed fixture is the change.

use headwater_census::census::{self, Census};
use headwater_census::resolve::{shelf_for, ShelfMatch};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::paint::ColorMode;
use headwater_check::Shape;
use headwater_generate::{
    check, descriptor, plan, write, Emitter, Identity, Kind, Plan, Projections, Report, Runs,
    Verdict,
};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_query::Surface;
use headwater_yaml::Mapping;
use saphyr::{LoadableYamlNode, YamlOwned};
use std::path::{Path, PathBuf};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

fn compare(recorded: &Path, actual: &str) {
    if std::env::var_os("HEADWATER_BLESS").is_some() {
        std::fs::write(recorded, actual).expect("cannot write the expectation");
        return;
    }
    let expected = std::fs::read_to_string(recorded).unwrap_or_else(|e| {
        panic!(
            "{}: {e}. Run with HEADWATER_BLESS=1 to record it.",
            recorded.display()
        )
    });
    assert_eq!(
        expected,
        actual,
        "\nthe run no longer matches {}",
        recorded.display()
    );
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

fn fixture_tree() -> (Built, Mapping) {
    let corpus = Corpus::new(fixtures_dir(), "generate");
    let root = load_map(&fixtures_dir().join("generate.taxonomy.yml"));
    (Built::over(&corpus, &root), root)
}

/// The identity the fixture corpus states about itself.
///
/// Fixed strings, and one exclusion, because the descriptor prints all of them
/// and a recorded fixture that read them from this repository's own lock would
/// move every time somebody re-resolved the taxonomy.
fn fixture_identity() -> Identity {
    Identity {
        corpus_root: "generate".to_string(),
        exclusions: vec![(
            "generate/drafts/**".to_string(),
            "drafts, and not governed content".to_string(),
        )],
        package: "headwater/fixture".to_string(),
        version: "1.0.0".to_string(),
        lock: "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string(),
    }
}

/// A fresh empty directory under Cargo's own temporary tree.
///
/// Never the fixture tree. A test that wrote its outputs beside its inputs would
/// leave a generated file that the next run's census walks, which is the exact
/// interaction this crate has to be careful about.
fn empty_tree(name: &str) -> PathBuf {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("a temporary tree");
    path
}

fn section(out: &mut String, title: &str, report: &Report) {
    out.push_str(&format!("\n--- {title} ---\n"));
    out.push_str(&report.render(ColorMode::Plain));
    out.push_str(&format!("fails the run: {}\n", report.has_errors()));
}

#[test]
fn the_fixture_tree_generates_the_recorded_projections() {
    let (built, root) = fixture_tree();
    let surface = built.surface();
    let projections = Projections::read(&root).expect("the projections read");
    let plan = plan(
        &surface,
        &built.census,
        &projections,
        &fixture_identity(),
        &Runs::default(),
        headwater_verbs::VERBS,
    );

    let mut out = String::new();
    out.push_str("the plan\n");
    for output in &plan.outputs {
        out.push_str(&format!(
            "\n=== {} ({}) ===\n",
            output.path,
            output.kind.name()
        ));
        out.push_str(&output.bytes);
    }

    // 1 and 2: a write, then the same write again.
    let tree = empty_tree("first");
    let first = write(&tree, &plan);
    section(&mut out, "write into an empty tree", &first);
    let written: Vec<String> = plan
        .outputs
        .iter()
        .map(|output| std::fs::read_to_string(tree.join(&output.path)).expect("it was written"))
        .collect();
    let second = write(&tree, &plan);
    section(&mut out, "write again over the first", &second);
    for (output, before) in plan.outputs.iter().zip(&written) {
        let after = std::fs::read_to_string(tree.join(&output.path)).expect("it is still there");
        assert_eq!(
            before, &after,
            "{} moved between two writes of one plan",
            output.path
        );
    }

    // A hand edit to a generated file, marker and all. The gate reports the
    // difference, and a write repairs it. This is the loop the marker exists
    // for: the file is this engine's, so it is overwritten rather than refused.
    let first_path = tree.join(&plan.outputs[0].path);
    let edited = format!("{}\n- a line somebody added by hand\n", written[0]);
    std::fs::write(&first_path, &edited).expect("the hand edit");
    let drifted = check(&tree, &plan);
    section(
        &mut out,
        "check over a hand-edited generated file",
        &drifted,
    );
    assert!(drifted.has_errors(), "a hand edit has to fail the gate");
    let repaired = write(&tree, &plan);
    section(
        &mut out,
        "write over a hand-edited generated file",
        &repaired,
    );
    assert_eq!(
        std::fs::read_to_string(&first_path).expect("it is there"),
        written[0],
        "a write did not repair a hand edit to its own output"
    );

    // 3: the gate over the tree the write produced.
    let held = check(&tree, &plan);
    section(&mut out, "check over what was written", &held);
    assert!(
        !held.has_errors(),
        "a fresh write does not satisfy its own check"
    );

    // 4: the gate over a tree that committed nothing.
    let bare = empty_tree("bare");
    let missing = check(&bare, &plan);
    section(&mut out, "check over an empty tree", &missing);
    assert!(
        missing.has_errors(),
        "a missing projection has to fail the gate"
    );

    // 5: an authored file already holds an output path.
    let occupied = empty_tree("occupied");
    let target = plan
        .outputs
        .first()
        .expect("the fixture generates one")
        .path
        .clone();
    let authored = "# A heading somebody wrote\n\nNo marker, and not this engine's to destroy.\n";
    let at = occupied.join(&target);
    std::fs::create_dir_all(at.parent().expect("a parent")).expect("the directory");
    std::fs::write(&at, authored).expect("the authored file");
    let refused = write(&occupied, &plan);
    section(&mut out, "write where an authored file is", &refused);
    assert_eq!(
        std::fs::read_to_string(&at).expect("it is still there"),
        authored,
        "the engine destroyed an authored document"
    );
    assert!(refused.has_errors(), "an occupied path has to fail the run");

    compare(&fixtures_dir().join("generate.record"), &out);
}

/// The plan is a function of the corpus and the lock, and of nothing else.
///
/// Two plans built over one tree hold the same bytes. This is the property that
/// makes `--check` meaningful, and it is asserted rather than recorded because a
/// recorded copy would prove only that one run equals itself.
#[test]
fn two_plans_over_one_tree_agree() {
    let (built, root) = fixture_tree();
    let surface = built.surface();
    let projections = Projections::read(&root).expect("the projections read");
    let one = plan(
        &surface,
        &built.census,
        &projections,
        &fixture_identity(),
        &Runs::default(),
        headwater_verbs::VERBS,
    );
    let two = plan(
        &surface,
        &built.census,
        &projections,
        &fixture_identity(),
        &Runs::default(),
        headwater_verbs::VERBS,
    );
    assert_eq!(one.outputs.len(), two.outputs.len());
    for (left, right) in one.outputs.iter().zip(&two.outputs) {
        assert_eq!(left.path, right.path);
        assert_eq!(left.bytes, right.bytes);
    }
}

/// A file this verb wrote is walked by the next census, and it comes back
/// `generated` rather than untyped.
///
/// The interaction this crate exists to be careful about, run for real rather
/// than reasoned about. A shelf index lands on the shelf it indexes, so the
/// write puts a file inside the corpus root and the next run walks it. The
/// property is the whole of issue #121: the artifact does not become a finding
/// against the corpus that produced it.
///
/// The second half is the same tree read a second way. This tree holds the
/// generated files and none of the documents they were generated from, so no
/// declaration produces them any more — which is exactly the state a corpus
/// reaches when a declaration is removed or its output path is repointed. Both
/// files are then orphaned, and the run fails. That failure is what stops the
/// marker from being a line an author can add to any document to exempt it from
/// every check.
#[test]
fn a_generated_file_is_censused_as_generated_and_orphaned_when_nothing_writes_it() {
    let (built, root) = fixture_tree();
    let surface = built.surface();
    let projections = Projections::read(&root).expect("the projections read");
    let first = plan(
        &surface,
        &built.census,
        &projections,
        &fixture_identity(),
        &Runs::default(),
        headwater_verbs::VERBS,
    );
    assert!(
        first.orphaned.is_empty(),
        "the fixture tree holds no marked file that nothing writes: {:?}",
        first.orphaned
    );

    let tree = empty_tree("censused");
    let report = write(&tree, &first);
    assert!(!report.has_errors(), "{}", report.render(ColorMode::Plain));

    // Which of the outputs landed inside the corpus root. The others sit
    // outside it by construction, and the census never sees those.
    let inside: Vec<String> = first
        .outputs
        .iter()
        .filter(|output| output.path.starts_with("generate/"))
        .map(|output| output.path.clone())
        .collect();
    assert!(
        !inside.is_empty(),
        "no output lands in the corpus root, so this test proves nothing"
    );

    let walked = Corpus::new(&tree, "generate");
    let again = Built::over(&walked, &root);
    for path in &inside {
        let row = again
            .census
            .rows
            .iter()
            .find(|row| &row.path == path)
            .unwrap_or_else(|| panic!("{path} is not in the census of the tree it was written to"));
        assert_eq!(
            row.outcome.class(),
            "generated",
            "{path} came back as `{}`: {}",
            row.outcome.class(),
            row.outcome.detail()
        );
    }

    // Nothing on these shelves now, so no declaration writes an index, so every
    // index already there is stale.
    let surface = again.surface();
    let stale = plan(
        &surface,
        &again.census,
        &projections,
        &fixture_identity(),
        &Runs::default(),
        headwater_verbs::VERBS,
    );
    let mut orphaned: Vec<&str> = stale
        .orphaned
        .iter()
        .map(|orphaned| orphaned.path.as_str())
        .collect();
    orphaned.sort_unstable();
    let mut expected: Vec<&str> = inside.iter().map(String::as_str).collect();
    expected.sort_unstable();
    assert_eq!(orphaned, expected);
    // The report names what each file claims to be, read back out of whichever
    // of the two places its shape put the marker: the first line of a list, and
    // a member of the front-matter block of a document.
    let mut claimed: Vec<(&str, Option<&str>)> = stale
        .orphaned
        .iter()
        .map(|orphaned| (orphaned.path.as_str(), orphaned.kind.as_deref()))
        .collect();
    claimed.sort_unstable();
    assert_eq!(
        claimed,
        vec![
            ("generate/archive/RETIRED.md", Some("shelf_sections")),
            ("generate/decisions/README.md", Some("shelf_index")),
            ("generate/decisions/SECTIONS.md", Some("shelf_sections")),
            ("generate/guides/README.md", Some("shelf_index")),
        ],
        "the report names what the file claims to be"
    );
    assert!(
        check(&tree, &stale).has_errors(),
        "a marked file that no declaration writes has to fail the gate"
    );
}

/// A generated document is a document of its shelf, and an index of that shelf
/// names it.
///
/// The engine held two definitions of "is a document" and only one of them
/// moved when #132 made a generated file a node. `Outcome::node` admitted a
/// generated row that resolved a kind, so the identifier index and the edge
/// builder both held it. `Surface::documents` read `Outcome::Typed` alone, so
/// every read built on it dropped the same file: a shelf index left the row
/// out, the export called its node set "every node the census holds" while
/// emitting one fewer than the graph carries, and `explain` answered "no kind"
/// about a document that resolves one.
///
/// Nothing reported the drop. `docs/spec/README.md` went from 16 rows to 15 the
/// day a projection took `docs/spec/09-open-questions.md`, and the only
/// evidence was the byte diff of a generated file. This test is the evidence
/// there was none of.
///
/// The shape it runs over is the repository's own, in the fixture tree: the
/// `archive` shelf holds no authored document and one generated one, which is
/// `generate/archive/RETIRED.md` with the identity `GD-FIX-archive`. So the
/// index of that shelf is written or it is not, with nothing else to confuse
/// the reading.
#[test]
fn a_generated_document_is_a_document_of_its_shelf() {
    // The fixture tree, copied, so that a write lands beside the documents the
    // plan was derived from rather than in a tree that holds only outputs.
    let tree = empty_tree("shelved");
    copy_tree(&fixtures_dir().join("generate"), &tree.join("generate"));

    let root = load_map(&fixtures_dir().join("generate.taxonomy.yml"));
    let before = Built::over(&Corpus::new(&tree, "generate"), &root);
    let projections = Projections::read(&root).expect("the projections read");
    let first = plan(
        &before.surface(),
        &before.census,
        &projections,
        &fixture_identity(),
        &Runs::default(),
        headwater_verbs::VERBS,
    );
    let report = write(&tree, &first);
    assert!(!report.has_errors(), "{}", report.render(ColorMode::Plain));

    // The tree now holds the generated document, so the census walks it.
    let after = Built::over(&Corpus::new(&tree, "generate"), &root);
    let surface = after.surface();
    const RETIRED: &str = "generate/archive/RETIRED.md";
    assert_eq!(
        after
            .census
            .rows
            .iter()
            .find(|row| row.path == RETIRED)
            .map(|row| row.outcome.class()),
        Some("generated"),
        "the fixture did not write the document this test is about"
    );

    // One: the read agrees with the graph about what a document is.
    let shelved = surface
        .documents()
        .into_iter()
        .find(|document| document.path == RETIRED)
        .expect("a generated document that declares an identity is a document");
    assert_eq!(shelved.kind, "guide");
    assert_eq!(shelved.id, Some("GD-FIX-archive"));

    // Two: `explain` answers about it, rather than reporting that nothing is
    // required of a file it can fully type.
    let explained = surface
        .explain(RETIRED)
        .expect("a generated document explains");
    assert_eq!(explained.kind.as_deref(), Some("guide"));
    assert_eq!(explained.id.as_deref(), Some("GD-FIX-archive"));

    // Three: the index of the shelf it sits on names it. On the reading this
    // test replaces, the `archive` shelf held no document, so the declaration
    // produced the reason below instead of a file.
    let second = plan(
        &surface,
        &after.census,
        &projections,
        &fixture_identity(),
        &Runs::default(),
        headwater_verbs::VERBS,
    );
    let index = second
        .outputs
        .iter()
        .find(|output| output.path == "generate/archive/README.md")
        .expect("an index of the shelf the generated document sits on");
    assert!(
        index.bytes.contains("[GD-FIX-archive](RETIRED.md)"),
        "{}",
        index.bytes
    );
    assert!(
        !second
            .unwritten
            .iter()
            .any(|unwritten| unwritten.at == "generate/archive/README.md"),
        "the shelf still reads as empty: {:?}",
        second.unwritten
    );
}

/// Copy a directory tree, so that a test may write into a corpus rather than
/// beside one.
fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the destination");
    for entry in std::fs::read_dir(from).expect("the source tree") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("a file type").is_dir() {
            true => copy_tree(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("the copy");
            }
        }
    }
}

/// Every generated file opens with the marker that lets the next run overwrite
/// it.
///
/// The property, over the plan rather than over one recorded artifact. A kind
/// added later gets this test for nothing, and an emitter that forgot the marker
/// would produce a file that the run after it refuses to touch.
#[test]
fn every_output_carries_its_own_marker() {
    let (built, root) = fixture_tree();
    let surface = built.surface();
    let projections = Projections::read(&root).expect("the projections read");
    let plan = plan(
        &surface,
        &built.census,
        &projections,
        &fixture_identity(),
        &Runs::default(),
        headwater_verbs::VERBS,
    );
    assert!(!plan.outputs.is_empty(), "the fixture generates something");
    for output in &plan.outputs {
        assert!(
            headwater_mark::carries_marker(&output.path, &output.bytes),
            "{} opens with no generated-file marker",
            output.path
        );
    }
}

/// This repository generates its eighteen artifacts, and it says why for
/// everything else.
///
/// A property and not a recording, for the reason the query crate states about
/// its own repository run: the corpus is prose somebody edits. What is asserted
/// is what a prose edit must not change.
///
/// **Twelve of the eighteen are shelf indexes, one per shelf that holds a
/// document.** The first is the decisions shelf, which the package has declared
/// since the first-run walkthrough and which produced a reason rather than a
/// file until #124 filled that shelf. The other eleven are the overlay's own
/// entry, in the order its `for` list names them, and the specification index
/// leads it because that is the list the root README used to carry by hand.
/// #528 is why the seven after it are there: each of those shelf roots answered
/// 404 on the served site while every record under it was served, because a
/// shelf with no index declaration writes no page for MkDocs to render.
///
/// The other six are one each. The redirect map that the open-questions
/// tombstone carries. The verb index #257 asked for, which reads the binary
/// rather than the `interfaces` shelf and is why that shelf is absent from the
/// index list above. The graph export #414 names: the whole graph, for the
/// reader Q16 draws with no principal to filter for. The site navigation
/// HW-DR-0036 and #418 name: MkDocs's `nav:` over the reading order
/// `by_precedence` derives. And the descriptor, at the path Q14 fixes.
///
/// Two declarations produce a reason rather than a file. The package declares
/// an index for one shelf this tree holds no document on, and the register is a
/// function of the clock.
///
/// **The order is the plan's order, and it is asserted.** A declared projection
/// is planned before the engine-defined descriptor, so a taxonomy that declares
/// a second index changes this list. That is the point: an emitter added or a
/// declaration removed fails here rather than in a reader's diff.
///
/// The gate at the end is the dogfood. It reads both committed files and
/// compares bytes, so a contributor who edits a `summary` and does not
/// regenerate fails this test before CI runs.
#[test]
fn this_repository_generates_its_eighteen_artifacts_and_accounts_for_the_rest() {
    let root = repository_root();
    let resolved = headwater_resolve::repository(&root)
        .unwrap_or_else(|errors| panic!("{}", headwater_resolve::render_errors(&errors)));
    let lock = headwater_lock::at(&root).expect("the lock reads");
    let corpus = Corpus::declared(
        &root,
        &resolved.consumer.corpus_root,
        &resolved.consumer.exclusions,
    );
    let built = Built::over(&corpus, &resolved.resolution.taxonomy);
    let surface = built.surface();
    let projections =
        Projections::read(&resolved.resolution.taxonomy).expect("the projections read");
    let identity = Identity {
        corpus_root: resolved.consumer.corpus_root.clone(),
        exclusions: resolved.consumer.exclusions.clone(),
        package: lock.package.clone(),
        version: lock.version.clone(),
        lock: lock.digest.clone(),
    };
    let mut runs = Runs::default();
    for row in &built.census.rows {
        let headwater_census::census::Outcome::Typed { kind, .. } = &row.outcome else {
            continue;
        };
        if kind != headwater_probe::intake::KIND {
            continue;
        }
        if let Ok(source) = std::fs::read_to_string(root.join(&row.path)) {
            runs.transcripts.push(headwater_generate::Transcript {
                path: row.path.clone(),
                source,
            });
        }
    }
    let declaration = std::fs::read_to_string(root.join(headwater_probe::budget::PATH))
        .expect("the probe budget declaration reads");
    let budgets = headwater_probe::Budgets::read(&declaration).expect("the probe budgets read");
    runs.graded_against(&headwater_probe::Plan::over(
        &built.census,
        &built.graph,
        &built.config,
        &budgets,
        &lock.digest,
        headwater_probe::Tier::Regression,
        &headwater_probe::plan::Narrowing::default(),
    ));
    let plan: Plan = plan(
        &surface,
        &built.census,
        &projections,
        &identity,
        &runs,
        headwater_verbs::VERBS,
    );

    let paths: Vec<&str> = plan.outputs.iter().map(|o| o.path.as_str()).collect();
    assert_eq!(
        paths,
        vec![
            "docs/decisions/README.md",
            "docs/spec/README.md",
            "docs/obligations/README.md",
            "docs/evaluations/README.md",
            "docs/requirements/README.md",
            "docs/acceptance-criteria/README.md",
            "docs/probes/README.md",
            "docs/probe-results/README.md",
            "docs/probe-runs/README.md",
            "docs/reviews/README.md",
            "docs/tutorials/README.md",
            "docs/process/decisions/README.md",
            "docs/spec/09-open-questions.md",
            "docs/probe-results/regression-probe-transcript-for-2026-09-09.md",
            "docs/interfaces/README.md",
            ".headwater/export.json",
            ".headwater/nav.yml",
            descriptor::PATH
        ],
        "this repository writes an index for each of its twelve shelves that hold a \
         document, then the redirect map, the verb index, the graph export, the \
         site navigation and the descriptor, in that order"
    );
    // One declared shelf that holds no document (`specifications`; the
    // `process_decisions` shelf held none for one commit, which put this
    // literal at 8 and left `docs/process/decisions/README.md` out of the list
    // above), one declared projection whose source this corpus does not hold,
    // and the register. Nothing is passed over: a projection that produced no
    // file states a reason. The other four are the declarable kinds this
    // engine does not emit and this corpus does not declare — `relation_view`,
    // `agent_rules`, `template` and `transcription` — which a run states
    // whether or not a declaration named them, because the reason is a
    // property of this engine rather than of the corpus.
    assert_eq!(
        plan.unwritten.len(),
        6,
        "a projection produced neither a file nor a reason"
    );
    for unwritten in &plan.unwritten {
        assert!(
            !unwritten.reason.is_empty(),
            "{} produced no file and no reason",
            unwritten.at
        );
    }

    let held = check(&root, &plan);
    assert!(
        !held.has_errors(),
        "a committed artifact is not what this corpus and this lock produce. \
         Run `headwater generate` and commit the result"
    );
    assert!(
        held.wrote
            .iter()
            .all(|wrote| wrote.verdict != Verdict::Occupied),
        "a declared output path is held by an authored document"
    );
}

/// The redirect map, which is the artifact the `identity` block exists for.
///
/// `docs/spec/09-open-questions.md` is a tombstone. All twenty-one questions
/// closed, each one is a document on the decisions shelf, and 136 citations in
/// this corpus name an anchor in this file. So three properties have to hold at
/// once, and each one fails differently.
///
/// 1. **Every anchor a citation names is a heading of the file.** A heading is
///    the only thing that makes an anchor, so a projection that wrote a bullet
///    would drop all 136 in silence.
/// 2. **The file declares the identity three other documents name.** A generated
///    file with no front matter is no node, and the three edges into
///    `HW-REG-open-questions` would resolve to nothing.
/// 3. **The reciprocal half of each of those edges is in the block**, derived
///    and never declared. Two documents supersede this one, `supersedes` says
///    `reciprocal: required`, and a file that omitted the halves would report
///    two findings against documents nobody edited.
///
/// The anchors are computed here rather than listed, so a decision renamed in
/// its own document fails this test at the citation rather than in a reader's
/// browser.
///
/// **The bytes come from the plan and not from the file**, so that a regression
/// in the emitter fails here rather than only in the byte comparison of the test
/// above. The committed file is then held against the same bytes, which is what
/// makes this one test cover both the derivation and the commit.
#[test]
fn the_redirect_map_keeps_every_anchor_that_this_corpus_cites_into_it() {
    let root = repository_root();
    let resolved = headwater_resolve::repository(&root)
        .unwrap_or_else(|errors| panic!("{}", headwater_resolve::render_errors(&errors)));
    let corpus = Corpus::declared(
        &root,
        &resolved.consumer.corpus_root,
        &resolved.consumer.exclusions,
    );
    let built = Built::over(&corpus, &resolved.resolution.taxonomy);
    let projections =
        Projections::read(&resolved.resolution.taxonomy).expect("the projections read");
    let plan: Plan = plan(
        &built.surface(),
        &built.census,
        &projections,
        &Identity::default(),
        &Runs::default(),
        headwater_verbs::VERBS,
    );
    let map = plan
        .outputs
        .iter()
        .find(|output| output.path == "docs/spec/09-open-questions.md")
        .map(|output| output.bytes.clone())
        .expect("the redirect map is planned");
    assert_eq!(
        std::fs::read_to_string(root.join("docs/spec/09-open-questions.md"))
            .expect("the redirect map is committed"),
        map,
        "the committed redirect map is not what this corpus and this lock produce"
    );

    // 2 and 3: the block, read as the census reads it.
    assert!(
        headwater_mark::carries_marker("docs/spec/09-open-questions.md", &map),
        "the redirect map carries no marker this engine can find"
    );
    assert!(
        map.starts_with("---\n"),
        "a document that declares an identity opens with the fence"
    );
    for member in [
        "id: HW-REG-open-questions",
        "doc_type: decision_register",
        "  superseded_by:",
        "    - HW-REG-decisions",
        "    - HW-REG-open-obligations",
    ] {
        assert!(
            map.contains(member),
            "the redirect map's block states no `{member}`"
        );
    }

    // 1: every anchor cited into the file, against every heading it writes.
    let headings: Vec<String> = map
        .lines()
        .filter_map(|line| line.strip_prefix("## "))
        .map(slug)
        .collect();
    let mut cited = Vec::new();
    for path in ["docs", ".headwater", "README.md"] {
        collect_citations(&root.join(path), "09-open-questions.md", &mut cited);
    }
    assert!(
        cited.len() > 100,
        "only {} citations into the redirect map, so this test is reading the wrong tree",
        cited.len()
    );
    let mut missing: Vec<&str> = cited
        .iter()
        .map(String::as_str)
        .filter(|anchor| !headings.contains(&anchor.to_string()))
        .collect();
    missing.sort_unstable();
    missing.dedup();
    assert_eq!(
        missing,
        Vec::<&str>::new(),
        "an anchor this corpus cites reaches no heading of the generated file"
    );
}

/// GitHub's heading slug: lower case, drop everything that is not a letter, a
/// digit, a hyphen or an underscore, and map each space to one hyphen. Runs of
/// spaces are not collapsed, which is why `Q1 — Implementation language` gives
/// `q1--implementation-language`.
fn slug(heading: &str) -> String {
    heading
        .trim()
        .to_lowercase()
        .chars()
        .filter_map(|c| match c {
            ' ' => Some('-'),
            c if c.is_alphanumeric() || c == '-' || c == '_' => Some(c),
            _ => None,
        })
        .collect()
}

/// Every `file#fragment` citation of one file name, anywhere under a directory.
fn collect_citations(at: &Path, file: &str, out: &mut Vec<String>) {
    if at.is_file() {
        let Ok(text) = std::fs::read_to_string(at) else {
            return;
        };
        for (_, rest) in text.match_indices(&format!("{file}#")).map(|(at, _)| {
            let rest = &text[at + file.len() + 1..];
            (at, rest)
        }) {
            let anchor: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            if !anchor.is_empty() {
                out.push(anchor);
            }
        }
        return;
    }
    let Ok(entries) = std::fs::read_dir(at) else {
        return;
    };
    for entry in entries.flatten() {
        collect_citations(&entry.path(), file, out);
    }
}

/// An authored file at the descriptor's path is refused, and JSON is the reason
/// this needs its own test.
///
/// The marker rule reads the first line of a commented format, and the first
/// line of a JSON object is the brace. A rule that only read lines would answer
/// "no marker" for this engine's own descriptor and refuse to overwrite it, or
/// answer "marker" for anything at all and destroy an authored file. Both
/// failures are here.
#[test]
fn the_descriptor_path_obeys_the_marker_rule() {
    let (built, root) = fixture_tree();
    let surface = built.surface();
    let projections = Projections::read(&root).expect("the projections read");
    let plan = plan(
        &surface,
        &built.census,
        &projections,
        &fixture_identity(),
        &Runs::default(),
        headwater_verbs::VERBS,
    );
    let written = plan
        .outputs
        .iter()
        .find(|output| output.path == descriptor::PATH)
        .expect("the descriptor is planned");

    // This engine's own output is recognized, so a second run overwrites it.
    assert!(
        headwater_mark::carries_marker(&written.path, &written.bytes),
        "the descriptor does not carry a marker this engine can find"
    );

    // An adopter's hand-written descriptor is not, so it survives.
    let tree = empty_tree("descriptor");
    let authored = "{\n  \"corpora\": []\n}\n";
    let at = tree.join(descriptor::PATH);
    std::fs::create_dir_all(at.parent().expect("a parent")).expect("the directory");
    std::fs::write(&at, authored).expect("the authored descriptor");
    let refused = write(&tree, &plan);
    assert_eq!(
        std::fs::read_to_string(&at).expect("it is still there"),
        authored,
        "the engine destroyed a hand-written descriptor"
    );
    assert!(
        refused
            .wrote
            .iter()
            .any(|w| w.path == descriptor::PATH && w.verdict == Verdict::Occupied),
        "a hand-written descriptor was not reported as occupied"
    );
}

/// The native export carries the graph with no loss, and the round trip is what
/// proves it.
///
/// [Spec 6](../../../../docs/spec/06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped):
/// "The native export keeps its round-trip test, because an empty loss set is
/// exactly what a round trip proves." So this reads the emitted bytes back with
/// the loader that reads every other file in this engine, and holds what it
/// finds against the graph the emitter read.
///
/// It compares the members rather than the byte count. A test that asserted a
/// length would pass over an emitter that wrote every document twice.
#[test]
fn the_native_export_round_trips_the_graph() {
    let (built, root) = fixture_tree();
    let surface = built.surface();
    let projections = Projections::read(&root).expect("the projections read");
    let profile = projections.profile("default").expect("the default profile");
    let emission = headwater_generate::export::emit(&surface, profile, Emitter::Json, None)
        .expect("the native export emits");

    assert!(
        emission.census.nodes.in_graph > 0 && emission.census.edges.in_graph > 0,
        "a round trip over an empty graph proves nothing"
    );
    assert_eq!(
        emission.census.nodes.carried, emission.census.nodes.in_graph,
        "the native export dropped a node, and it declares no loss"
    );
    assert_eq!(
        emission.census.edges.carried, emission.census.edges.in_graph,
        "the native export dropped an edge, and it declares no loss"
    );

    let read = headwater_yaml::load(&emission.bytes).expect("the export reads back");
    let map = read.value.as_map().expect("an object");
    let graph = map
        .get("graph")
        .and_then(|node| node.value.as_map())
        .expect("a graph");

    let documents = graph
        .get("documents")
        .and_then(|node| node.value.as_seq())
        .expect("documents");
    let mut seen: Vec<String> = Vec::new();
    for document in documents {
        let entry = document.value.as_map().expect("a document object");
        let path = entry
            .get("path")
            .and_then(|node| node.value.as_scalar())
            .expect("a path")
            .text
            .clone();
        assert!(!seen.contains(&path), "{path} was exported twice");
        seen.push(path);
    }
    let mut expected: Vec<String> = surface
        .documents()
        .iter()
        .map(|document| document.path.to_string())
        .collect();
    seen.sort();
    expected.sort();
    assert_eq!(
        seen, expected,
        "the export and the corpus hold different documents"
    );

    // Every facet the document wrote is on the far side, as the text it wrote.
    // This is the part a lossy emitter gets wrong and a count would not catch.
    for document in &surface.documents() {
        let entry = documents
            .iter()
            .find_map(|item| {
                let map = item.value.as_map()?;
                let path = map.get("path")?.value.as_scalar()?;
                (path.text == document.path).then_some(map)
            })
            .expect("the document is there");
        let facets = entry
            .get("facets")
            .and_then(|node| node.value.as_map())
            .expect("facets");
        for facet in document.facets {
            assert!(
                facets.get(&facet.key.value).is_some(),
                "{}: the facet `{}` did not survive the export",
                document.path,
                facet.key.value
            );
        }
    }

    let edges = graph
        .get("edges")
        .and_then(|node| node.value.as_seq())
        .expect("edges");
    assert_eq!(
        edges.len(),
        surface.graph().edges.len(),
        "the export and the graph hold different edge counts"
    );

    // An empty loss set, stated in the artifact rather than only in the census.
    let losses = map
        .get("loss_set")
        .and_then(|node| node.value.as_seq())
        .expect("a loss set");
    assert!(losses.is_empty(), "the native export declared a loss");
}

/// A filter withholds a document whole, and every edge that names it.
///
/// Spec 6's claim about a filtered export: it "contains no document that its
/// declared filter withholds, and no artifact inside the profile derives from
/// one". An edge into a withheld document states that the document exists and
/// what it is called, so it goes too.
#[test]
fn a_filtered_profile_withholds_a_document_and_its_edges() {
    let (built, root) = fixture_tree();
    let surface = built.surface();
    let projections = Projections::read(&root).expect("the projections read");
    let profile = projections.profile("partner").expect("the partner profile");
    let emission = headwater_generate::export::emit(&surface, profile, Emitter::Json, None)
        .expect("the filtered export emits");

    assert!(
        emission.census.nodes.accounted > 0,
        "the filter withheld nothing, so this proves nothing"
    );
    assert!(
        !emission.census.is_defective(),
        "a withholding is a loss reason, and this census did not account for one"
    );
    assert!(
        !emission.bytes.contains("GD-FIX-recovery"),
        "the withheld document reached the artifact"
    );
    assert!(
        !emission.bytes.contains("recovery.md"),
        "the path of the withheld document reached the artifact"
    );

    // The tombstone carries the rule identifier and never the document. Spec 6:
    // "The rule identifier is what a reader needs to ask for access, and it is
    // all that they get."
    assert!(
        emission.bytes.contains("partner.exclude.status"),
        "the tombstone does not name the rule that withheld anything"
    );
    // And the view says out loud that it is not total.
    assert!(
        emission.bytes.contains("\"filtered\": true"),
        "a filtered view presented itself as total"
    );
}

/// Under `sealed`, the view states that it is filtered and nothing else.
#[test]
fn a_sealed_grain_states_the_filtering_and_no_count() {
    let (built, root) = fixture_tree();
    let surface = built.surface();
    let projections = Projections::read(&root).expect("the projections read");
    let mut profile = projections
        .profile("partner")
        .expect("the partner profile")
        .clone();
    profile.tombstone = headwater_generate::Grain::Sealed;
    let emission = headwater_generate::export::emit(&surface, &profile, Emitter::Json, None)
        .expect("the sealed export emits");

    assert!(
        emission.bytes.contains("\"filtered\": true"),
        "a sealed view has to say that it is filtered: a reader who concludes \
         absence from silence is the harm the rule prevents"
    );
    assert!(
        !emission.bytes.contains("tombstones"),
        "a sealed view reported a tombstone"
    );
    assert!(
        !emission.bytes.contains("GD-FIX-recovery"),
        "the withheld document reached the artifact"
    );
}

/// The census fails a projector that dropped something with no reason.
///
/// This is the failing fixture for the rule spec 6 states: "An omission that no
/// reason covers is a projector defect, and it fails the run." It runs the
/// census against a projector that carried nothing and declared no loss, which
/// is the emitter this whole apparatus exists to catch, and then against one
/// that carried everything.
#[test]
fn the_census_fails_a_projector_that_dropped_a_node_with_no_reason() {
    let (built, root) = fixture_tree();
    let surface = built.surface();
    let projections = Projections::read(&root).expect("the projections read");
    let profile = projections.profile("default").expect("the default profile");

    let dropped = headwater_generate::export::audit(&surface, &[], &[], &[], &[]);
    assert!(
        dropped.is_defective(),
        "a projector that carried nothing and declared no loss passed the census"
    );
    assert_eq!(dropped.nodes.carried, 0);
    assert_eq!(
        dropped.unaccounted.len(),
        dropped.nodes.in_graph + dropped.edges.in_graph,
        "the census failed to name every omission"
    );

    // The same census over a projector that carried the lot. The keys are the
    // emitter's own, so this also holds the key scheme: a census that could not
    // match a carried node against a graph node would report every node missing.
    let emission = headwater_generate::export::emit(&surface, profile, Emitter::Json, None)
        .expect("the native export emits");
    assert!(
        !emission.census.is_defective(),
        "the native export failed its own census"
    );
}

/// A JSON Schema carries constraints and no instance, and it says so.
///
/// The two check families that clear spec 12's equivalence bar are here, and the
/// third construction that would not clear it is deliberately absent: the kind
/// is selected by a discriminator under `if`/`then`, never by a root `oneOf`
/// that reports an error where the native check has none.
#[test]
fn the_schema_export_carries_constraints_and_accounts_for_every_instance() {
    let (built, root) = fixture_tree();
    let surface = built.surface();
    let projections = Projections::read(&root).expect("the projections read");
    let profile = projections.profile("default").expect("the default profile");
    let emission = headwater_generate::export::emit(&surface, profile, Emitter::JsonSchema, None)
        .expect("the schema emits");

    assert_eq!(
        emission.census.nodes.carried, 0,
        "a JSON Schema carried an instance"
    );
    assert_eq!(
        emission.census.nodes.accounted, emission.census.nodes.in_graph,
        "a node of the corpus fell outside both the output and the loss set"
    );
    assert_eq!(
        emission.census.edges.accounted, emission.census.edges.in_graph,
        "an edge fell outside both the output and the loss set"
    );
    assert!(!emission.census.is_defective());

    assert!(
        !emission.bytes.contains("\"oneOf\""),
        "a root `oneOf` reports an error where the native check has none, because an \
         abstract kind and the kind that descends from it accept the same front matter"
    );
    let read = headwater_yaml::load(&emission.bytes).expect("the schema reads back");
    let map = read.value.as_map().expect("an object");
    let defs = map
        .get("$defs")
        .and_then(|node| node.value.as_map())
        .expect("definitions");
    let decision = defs
        .get("decision")
        .and_then(|node| node.value.as_map())
        .expect("a definition for `decision`");
    let required: Vec<String> = decision
        .get("required")
        .and_then(|node| node.value.as_seq())
        .expect("required")
        .iter()
        .filter_map(|item| item.value.as_scalar())
        .map(|scalar| scalar.text.clone())
        .collect();
    for facet in surface.shape().required_facets("decision") {
        assert!(
            required.contains(&facet),
            "the schema does not require `{facet}`, and the native check does"
        );
    }
    // A facet with a declared value set becomes an enum over exactly that set,
    // under a guard that admits a mapping or a list, and a facet with none gets
    // no constraint. A `type: string` here would be the emitter inventing a rule
    // the native check does not enforce, and a bare `enum` would be the emitter
    // enforcing one harder than the check does: `facet.value.not_permitted`
    // declines a value it cannot read as a scalar, and the guard is what makes
    // the two agree. The differential owns that argument
    // (`tests/differential.rs`); this line only holds the shape.
    let properties = decision
        .get("properties")
        .and_then(|node| node.value.as_map())
        .expect("properties");
    let status = properties
        .get("status")
        .and_then(|node| node.value.as_map())
        .expect("the status property");
    let guarded = status
        .get("anyOf")
        .and_then(|node| node.value.as_seq())
        .expect("the guard on a value set");
    assert_eq!(guarded.len(), 2, "the guard is not a two-branch choice");
    assert!(
        guarded[1]
            .value
            .as_map()
            .and_then(|branch| branch.get("enum"))
            .is_some(),
        "an enum facet lost its values"
    );
    let since = properties
        .get("status_since")
        .and_then(|node| node.value.as_map())
        .expect("the status_since property");
    assert_eq!(
        since.len(),
        0,
        "the schema invented a constraint for a facet whose type no declaration states"
    );
}

/// An emitter this release does not build reports the consumer it waits on.
///
/// Q13 stages six emitters and puts every one after the second behind a named
/// external consumer. None exists, so five of the seven targets parse and emit
/// nothing, and the message says which and why. An empty artifact would be a
/// worse answer than a refusal.
///
/// The first two assertions below are substrings, so they hold that the refusal
/// happens and names its emitter, and hold nothing about the wording.
/// `docs/spec/02-taxonomy-model.md` quotes that wording verbatim, and
/// `tests/spec_two_emitters.rs` is what compares the two byte for byte. Reword
/// the message and this case stays green; that one reddens.
///
/// The third assertion is the one that fires on a change to this engine rather
/// than to the document. The refusal names the emitters that ship, and a third
/// emitter marked built and left out of that sentence makes the sentence false
/// in the engine and in spec 2 at once — which is the state a comparison
/// between the two cannot see, because both sides are wrong in the same words.
#[test]
fn an_unbuilt_emitter_refuses_and_says_what_it_waits_on() {
    let (built, root) = fixture_tree();
    let surface = built.surface();
    let projections = Projections::read(&root).expect("the projections read");
    let profile = projections.profile("default").expect("the default profile");
    for emitter in Emitter::ALL {
        let outcome = headwater_generate::export::emit(&surface, profile, emitter, None);
        match emitter.is_built() {
            true => assert!(
                outcome.is_ok(),
                "{} is built and did not emit",
                emitter.name()
            ),
            false => {
                let reason = outcome.expect_err("an unbuilt emitter refuses").reason();
                assert!(
                    reason.contains(emitter.name()) && reason.contains("consumer"),
                    "{} refused without naming what it waits on",
                    emitter.name()
                );
                for shipped in Emitter::ALL.into_iter().filter(|one| one.is_built()) {
                    assert!(
                        reason.contains(&format!("`{}`", shipped.name())),
                        "the refusal for {} says which emitters ship and does not name {}, which \
                         this engine builds. The sentence is quoted in \
                         docs/spec/02-taxonomy-model.md, so a list that goes stale takes that \
                         document with it:\n{reason}",
                        emitter.name(),
                        shipped.name()
                    );
                }
            }
        }
    }
}

/// An exporter that cannot evaluate its filter emits nothing.
///
/// Spec 6, on principle 7 read for an exporter: "It never emits an unfiltered
/// artifact, and it never emits a partly filtered one." A clause over a facet
/// that the taxonomy does not declare withholds nothing at all, so the failure
/// it produces is the one an export may not have.
#[test]
fn a_filter_over_a_facet_that_does_not_exist_emits_nothing() {
    let (built, root) = fixture_tree();
    let surface = built.surface();
    let projections = Projections::read(&root).expect("the projections read");
    let mut profile = projections
        .profile("partner")
        .expect("the partner profile")
        .clone();
    profile.filter.exclude = vec![headwater_generate::Clause {
        facet: "confidentiality".to_string(),
        values: vec!["internal".to_string()],
    }];
    let outcome = headwater_generate::export::emit(&surface, &profile, Emitter::Json, None);
    let reason = outcome
        .expect_err("an unevaluable filter emits nothing")
        .reason();
    assert!(
        reason.contains("confidentiality"),
        "the refusal does not name the clause that could not be evaluated"
    );
}

/// One audience has one filter, and two entries that disagree are refused.
///
/// The confluence rule of [`headwater_generate::profile`]. Without it one of the
/// two filters wins, the winner is whichever the reader reached last, and the
/// artifact that lost carries more than the profile permits.
#[test]
fn two_entries_of_one_profile_may_not_declare_two_filters() {
    let source = "\
projections:
  - kind: graph_export
    profile: partner
    output: exports/one.json
    filter: {exclude: {status: [draft]}}
  - kind: graph_export
    profile: partner
    output: exports/two.json
    filter: {exclude: {status: [superseded]}}
";
    let root = headwater_yaml::load(source)
        .expect("it loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();
    let errors = Projections::read(&root).expect_err("two filters for one audience are refused");
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("one filter")),
        "the refusal does not say why: {errors:?}"
    );
}

/// A `graph_export` declared the way `.headwater/overlay.yml` declares one —
/// one profile, no filter, no `format` — held to regeneration through
/// `plan()`, `write()` and `check()`, the same functions `generate` and
/// `generate --check` call.
///
/// The two tests above call `export::emit()` directly against a hand-built
/// `Profile`, so neither one reaches `plan()`'s `Kind::GraphExport` arm. This
/// one does, over the crate's own `fixture_tree()`: it fails before the file
/// exists, fails after a hand edit that strips the marker, fails after a
/// deletion, and passes again once `write()` regenerates it — the four
/// branches the issue's own "generate fixture suite covers the new emitter"
/// bar names.
#[test]
fn a_declared_graph_export_is_held_to_regeneration() {
    let (built, _root) = fixture_tree();
    let surface = built.surface();
    let source = "\
projections:
  - kind: graph_export
    profile: site
    output: exports/site.json
";
    let root = headwater_yaml::load(source)
        .expect("it loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();
    let projections = Projections::read(&root).expect("the projections read");
    let plan = plan(
        &surface,
        &built.census,
        &projections,
        &fixture_identity(),
        &Runs::default(),
        headwater_verbs::VERBS,
    );
    let output = plan
        .outputs
        .iter()
        .find(|output| output.path == "exports/site.json")
        .expect("the declaration produced an output");
    assert_eq!(output.kind, Kind::GraphExport);

    // (a) before the file exists, the plan's own check would write it.
    let tree = empty_tree("graph-export-missing");
    let missing = check(&tree, &plan);
    assert!(
        missing.has_errors(),
        "an ungenerated graph_export has to fail the gate before it is ever written"
    );

    let first = write(&tree, &plan);
    assert!(
        !first.has_errors(),
        "the first write failed: {}",
        first.render(ColorMode::Plain)
    );
    let bytes_after_write =
        std::fs::read_to_string(tree.join("exports/site.json")).expect("it was written");
    let clean = check(&tree, &plan);
    assert!(
        !clean.has_errors(),
        "a fresh write does not satisfy its own check: {}",
        clean.render(ColorMode::Plain)
    );

    // (b) a hand edit strips the marker, and the gate has to catch it.
    std::fs::write(tree.join("exports/site.json"), "{\"tampered\": true}\n")
        .expect("the hand edit");
    let tampered = check(&tree, &plan);
    assert!(
        tampered.has_errors(),
        "a hand-edited graph_export with no marker has to fail the gate"
    );

    // (c) a deletion is the same failure the missing-file case above is.
    std::fs::remove_file(tree.join("exports/site.json")).expect("the deletion");
    let deleted = check(&tree, &plan);
    assert!(
        deleted.has_errors(),
        "a deleted graph_export has to fail the gate"
    );

    // (d) regeneration restores byte-identical output and a clean check.
    let second = write(&tree, &plan);
    assert!(
        !second.has_errors(),
        "regeneration failed: {}",
        second.render(ColorMode::Plain)
    );
    let bytes_after_regen =
        std::fs::read_to_string(tree.join("exports/site.json")).expect("it is there again");
    assert_eq!(
        bytes_after_write, bytes_after_regen,
        "regeneration did not restore byte-identical output"
    );
    let restored = check(&tree, &plan);
    assert!(
        !restored.has_errors(),
        "check does not pass again after regeneration: {}",
        restored.render(ColorMode::Plain)
    );
}

/// A `site_nav` declared the way `.headwater/overlay.yml` declares one — no
/// `for`, no `filter` — held to regeneration through `plan()`, `write()` and
/// `check()`. The direct analog of `a_declared_graph_export_is_held_to_
/// regeneration` above, over the same four branches.
///
/// It also parses the emitted bytes with `saphyr`, a YAML parser this engine
/// did not write, rather than trusting the bytes by eye, and checks the
/// `nav` sequence's order against [`Surface::by_precedence`] directly — the
/// same derivation `site_nav.rs` itself calls, read back through a second
/// implementation.
#[test]
fn a_declared_site_nav_is_held_to_regeneration() {
    let (built, _root) = fixture_tree();
    let surface = built.surface();
    let source = "\
projections:
  - kind: site_nav
    output: nav.yml
";
    let root = headwater_yaml::load(source)
        .expect("it loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();
    let projections = Projections::read(&root).expect("the projections read");
    let plan = plan(
        &surface,
        &built.census,
        &projections,
        &fixture_identity(),
        &Runs::default(),
        headwater_verbs::VERBS,
    );
    let output = plan
        .outputs
        .iter()
        .find(|output| output.path == "nav.yml")
        .expect("the declaration produced an output");
    assert_eq!(output.kind, Kind::SiteNav);

    // (a) before the file exists, the plan's own check would write it.
    let tree = empty_tree("site-nav-missing");
    let missing = check(&tree, &plan);
    assert!(
        missing.has_errors(),
        "an ungenerated site_nav has to fail the gate before it is ever written"
    );

    let first = write(&tree, &plan);
    assert!(
        !first.has_errors(),
        "the first write failed: {}",
        first.render(ColorMode::Plain)
    );
    let bytes_after_write = std::fs::read_to_string(tree.join("nav.yml")).expect("it was written");
    let clean = check(&tree, &plan);
    assert!(
        !clean.has_errors(),
        "a fresh write does not satisfy its own check: {}",
        clean.render(ColorMode::Plain)
    );

    // (b) a hand edit strips the marker, and the gate has to catch it.
    std::fs::write(tree.join("nav.yml"), "nav: []\n").expect("the hand edit");
    let tampered = check(&tree, &plan);
    assert!(
        tampered.has_errors(),
        "a hand-edited site_nav with no marker has to fail the gate"
    );

    // (c) a deletion is the same failure the missing-file case above is.
    std::fs::remove_file(tree.join("nav.yml")).expect("the deletion");
    let deleted = check(&tree, &plan);
    assert!(
        deleted.has_errors(),
        "a deleted site_nav has to fail the gate"
    );

    // (d) regeneration restores byte-identical output and a clean check.
    let second = write(&tree, &plan);
    assert!(
        !second.has_errors(),
        "regeneration failed: {}",
        second.render(ColorMode::Plain)
    );
    let bytes_after_regen =
        std::fs::read_to_string(tree.join("nav.yml")).expect("it is there again");
    assert_eq!(
        bytes_after_write, bytes_after_regen,
        "regeneration did not restore byte-identical output"
    );
    let restored = check(&tree, &plan);
    assert!(
        !restored.has_errors(),
        "check does not pass again after regeneration: {}",
        restored.render(ColorMode::Plain)
    );

    // Stronger than eyeballing: an independent parser, and the order checked
    // against `by_precedence` rather than assumed.
    let docs =
        YamlOwned::load_from_str(&bytes_after_write).expect("the emitted file is valid YAML");
    let doc = docs.first().expect("one YAML document");
    let nav = doc
        .as_mapping_get("nav")
        .expect("a top-level `nav` key")
        .as_sequence()
        .expect("`nav` is a sequence");

    // `decisions` and `guides` both hold a document; `archive` holds none and
    // is left out, so `nav` covers exactly two of the fixture tree's three
    // shelves.
    assert_eq!(nav.len(), 2, "an empty shelf should not appear in `nav`");

    for group in nav {
        let mapping = group
            .as_mapping()
            .expect("each nav entry is a one-key mapping");
        let (shelf_key, entries) = mapping.iter().next().expect("exactly one key");
        let shelf_name = shelf_key.as_str().expect("the shelf name is a string");

        let on_shelf: Vec<_> = surface
            .documents()
            .into_iter()
            .filter(|document| {
                matches!(
                    shelf_for(document.path, surface.taxonomy()),
                    ShelfMatch::Matched { shelf, .. } if shelf.name == shelf_name
                )
            })
            .collect();
        let mut expected: Vec<_> = on_shelf
            .iter()
            .map(|document| surface.pointer(document))
            .collect();
        surface.by_precedence(&mut expected);

        let entries = entries
            .as_sequence()
            .expect("the shelf's entries are a sequence");
        assert_eq!(
            entries.len(),
            expected.len(),
            "{shelf_name} has the wrong number of entries"
        );
        for (entry, pointer) in entries.iter().zip(&expected) {
            let entry_map = entry.as_mapping().expect("each entry is a one-key mapping");
            let (_, path_value) = entry_map.iter().next().expect("exactly one key");
            let path = path_value.as_str().expect("the path is a string");
            // MkDocs resolves a `nav:` path against `docs_dir`, and `docs_dir`
            // is the corpus root, so the entry names the document from the
            // root rather than from the repository. `fixture_identity` roots
            // this corpus at `generate`, and the prefix is stripped here by
            // hand rather than through the emitter's own helper, so that the
            // test states the expected form instead of restating the
            // transform.
            let expected_path = pointer
                .path
                .strip_prefix("generate/")
                .expect("every fixture document sits under the fixture corpus root");
            assert_eq!(
                path, expected_path,
                "{shelf_name}'s order does not match `by_precedence`"
            );
        }
    }
}

/// The same nav, over a corpus that also declares a shelf index — the case
/// [#528](https://github.com/headwater-ai/headwater/issues/528) is about.
///
/// A generated index carries no front matter, holds no identifier and is no
/// node of the graph, so the emitter above could never list one and ten served
/// pages sat outside the navigation of this repository's own site. This case
/// is the sibling of `a_declared_site_nav_is_held_to_regeneration` and differs
/// from it in exactly one input: the source declares a `shelf_index` beside the
/// `site_nav`. That one is deliberately left declaring a `site_nav` alone, so
/// the pair states both halves of the rule — a group whose shelf has an index
/// opens with it, and a group whose shelf has none still lists documents only.
///
/// `archive` is declared for the index and holds no document, so it produces
/// neither an index nor a group: a shelf with no group gets no index entry.
///
/// **The nav is declared above the index, and that is not cosmetic.** The
/// emitter reads the plan's other outputs, so `plan()` runs every `site_nav`
/// in a second pass after every other declaration rather than in its own turn.
/// `.headwater/overlay.yml` declares `site_nav` last, so a case written in
/// that order passes whether the second pass is there or not — reverting it
/// left all 70 cases of this crate green. Only a nav declared *above* the
/// index it lists separates the two. The case asserts the property as well as
/// the arrangement: two sources differing only in declaration order emit the
/// same bytes.
#[test]
fn a_site_nav_opens_each_group_with_that_shelf_s_generated_index() {
    let (built, _root) = fixture_tree();
    let surface = built.surface();
    // Written with explicit newlines rather than a `"\` continuation, because
    // that form strips the leading whitespace of every line it continues onto
    // and a YAML sequence item needs its indentation.
    let index = concat!(
        "  - kind: shelf_index\n",
        "    for: [decisions, guides, archive]\n",
        "    output: \"{shelf}/README.md\"\n",
    );
    let nav = concat!("  - kind: site_nav\n", "    output: nav.yml\n");
    let build = |source: String| {
        let root = headwater_yaml::load(&source)
            .expect("it loads")
            .value
            .as_map()
            .expect("a mapping")
            .clone();
        let projections = Projections::read(&root).expect("the projections read");
        plan(
            &surface,
            &built.census,
            &projections,
            &fixture_identity(),
            &Runs::default(),
            headwater_verbs::VERBS,
        )
    };

    // THE NAV IS DECLARED FIRST, WHICH IS WHAT HOLDS THE SECOND PASS
    //
    // `plan()` runs every `site_nav` after every other declaration, rather
    // than in its own turn, so that this file is a function of the plan's
    // outputs and not of the order the overlay lists its projections in.
    // `.headwater/overlay.yml` happens to declare `site_nav` last, so a
    // source that does the same holds nothing: an emitter that read the
    // outputs in its declaration turn would find the same two indexes there
    // and pass. This source declares the nav above the index it lists, which
    // is the one arrangement where the two implementations disagree.
    let plan = build(format!("projections:\n{nav}{index}"));

    // And the property itself, rather than the one arrangement that shows it.
    // Two sources that differ only in declaration order emit the same bytes.
    let reversed = build(format!("projections:\n{index}{nav}"));
    let bytes_of = |plan: &Plan| {
        plan.outputs
            .iter()
            .find(|output| output.path == "nav.yml")
            .expect("the declaration produced an output")
            .bytes
            .clone()
    };
    assert_eq!(
        bytes_of(&plan),
        bytes_of(&reversed),
        "the emitted nav depends on the order the projections are declared in"
    );

    let output = plan
        .outputs
        .iter()
        .find(|output| output.path == "nav.yml")
        .expect("the declaration produced an output");
    assert_eq!(output.kind, Kind::SiteNav);

    // The two indexes the first declaration writes. `archive` writes none,
    // because a shelf index of an empty shelf is declined rather than written.
    let indexes: Vec<&str> = plan
        .outputs
        .iter()
        .filter(|output| output.kind == Kind::ShelfIndex)
        .map(|output| output.path.as_str())
        .collect();
    assert_eq!(
        indexes,
        vec!["generate/decisions/README.md", "generate/guides/README.md"],
        "the fixture corpus has to produce exactly the two indexes this case reads"
    );

    let docs = YamlOwned::load_from_str(&output.bytes).expect("the emitted file is valid YAML");
    let doc = docs.first().expect("one YAML document");
    let nav = doc
        .as_mapping_get("nav")
        .expect("a top-level `nav` key")
        .as_sequence()
        .expect("`nav` is a sequence");
    assert_eq!(nav.len(), 2, "an empty shelf should not appear in `nav`");

    for group in nav {
        let mapping = group
            .as_mapping()
            .expect("each nav entry is a one-key mapping");
        let (shelf_key, entries) = mapping.iter().next().expect("exactly one key");
        let shelf_name = shelf_key.as_str().expect("the shelf name is a string");
        let entries = entries
            .as_sequence()
            .expect("the shelf's entries are a sequence");

        // The first entry is the shelf's own generated index, under the shelf
        // it indexes. The label is what MkDocs serves as that page's `<title>`
        // and as its search-index entry, so a constant here makes every shelf
        // index indistinguishable in a browser tab, a bookmark and a search
        // result. #567 is where that was measured over ten pages.
        let (first_label, first_path) = {
            let entry = entries
                .first()
                .expect("the group holds at least the index")
                .as_mapping()
                .expect("each entry is a one-key mapping");
            let (key, value) = entry.iter().next().expect("exactly one key");
            (
                key.as_str().expect("the label is a string").to_string(),
                value.as_str().expect("the path is a string").to_string(),
            )
        };
        assert_eq!(
            first_path,
            format!("{shelf_name}/README.md"),
            "{shelf_name} does not open with its own generated index"
        );
        assert_eq!(
            first_label, shelf_name,
            "{shelf_name}'s index entry is not labeled with the shelf it indexes"
        );

        // Everything after it is still the documents of the shelf, in the
        // order `by_precedence` derives, unmoved by the index in front.
        let on_shelf: Vec<_> = surface
            .documents()
            .into_iter()
            .filter(|document| {
                matches!(
                    shelf_for(document.path, surface.taxonomy()),
                    ShelfMatch::Matched { shelf, .. } if shelf.name == shelf_name
                )
            })
            .collect();
        let mut expected: Vec<_> = on_shelf
            .iter()
            .map(|document| surface.pointer(document))
            .collect();
        surface.by_precedence(&mut expected);
        assert_eq!(
            entries.len(),
            expected.len() + 1,
            "{shelf_name} has the wrong number of entries"
        );
        for (entry, pointer) in entries.iter().skip(1).zip(&expected) {
            let entry_map = entry.as_mapping().expect("each entry is a one-key mapping");
            let (_, path_value) = entry_map.iter().next().expect("exactly one key");
            let path = path_value.as_str().expect("the path is a string");
            let expected_path = pointer
                .path
                .strip_prefix("generate/")
                .expect("every fixture document sits under the fixture corpus root");
            assert_eq!(
                path, expected_path,
                "{shelf_name}'s order does not match `by_precedence`"
            );
        }
    }
}

/// A committed descriptor whose emitter set is not this engine's stops the
/// `--check` verdict from naming a remedy, and the report names both numbers.
///
/// # What this is evidence of, and what it would be evidence of without care
///
/// [The contract](../../../../docs/interfaces/headwater-generate.md) says that
/// `--check` reads the producer identity before it compares any projection's
/// bytes, and that on a difference it withholds the instruction to regenerate.
/// Ask what this run prints if the guard is absent: it prints a stale list and
/// the regenerate line, in **both** states. So an assertion on a non-zero exit,
/// or on a projection reported as stale, passes whether the guard ran or not
/// ([#211](https://github.com/headwater-ai/headwater/issues/211) names that
/// defect). The two decisive assertions here are the *absence* of the remedy
/// this command cannot honestly give, and the presence of both numbers.
///
/// The tree is written by this engine first, so every artifact in it is exactly
/// what this engine produces. The only thing that moves afterwards is the
/// recorded emitter set. That makes the corpus and the emitters the only two
/// candidate causes of the difference, which is the situation the guard is for.
#[test]
fn a_committed_descriptor_from_another_emitter_set_withholds_the_remedy() {
    let (built, root) = fixture_tree();
    let surface = built.surface();
    let projections = Projections::read(&root).expect("the projections read");
    let plan = plan(
        &surface,
        &built.census,
        &projections,
        &fixture_identity(),
        &Runs::default(),
        headwater_verbs::VERBS,
    );

    let tree = empty_tree("producer-identity");
    let written = write(&tree, &plan);
    assert!(
        !written.has_errors(),
        "the tree did not write clean: {}",
        written.render(ColorMode::Plain)
    );

    // Clause 4 of the issue: what this engine writes, this engine trusts. A
    // round trip in one engine reports no producer difference and exits zero.
    let round_trip = check(&tree, &plan);
    assert_eq!(
        round_trip.producer,
        None,
        "{}",
        round_trip.render(ColorMode::Plain)
    );
    assert!(
        !round_trip.has_errors(),
        "a round trip in one engine failed: {}",
        round_trip.render(ColorMode::Plain)
    );
    assert_eq!(round_trip.remedy(), None);

    // Now the only difference in the tree is which emitters wrote it.
    let current = headwater_generate::emitters::EMITTER_SET;
    let stale = current + 1;
    let at = tree.join(descriptor::PATH);
    let committed = std::fs::read_to_string(&at).expect("the written descriptor");
    let moved = committed.replacen(
        &format!("\"emitter_set\": {current}"),
        &format!("\"emitter_set\": {stale}"),
        1,
    );
    assert_ne!(moved, committed, "the descriptor records no emitter set");
    std::fs::write(&at, &moved).expect("the moved descriptor");

    let report = check(&tree, &plan);
    assert_eq!(
        report.producer,
        Some(headwater_generate::Producer {
            recorded: stale,
            current,
        }),
        "{}",
        report.render(ColorMode::Plain)
    );

    let remedy = report.remedy().expect("a failing run states why");
    assert!(
        remedy.contains(&stale.to_string()) && remedy.contains(&current.to_string()),
        "the sentence does not name both numbers: {remedy}"
    );
    assert!(
        !remedy.contains("headwater generate"),
        "the run told a reader to regenerate over a producer difference: {remedy}"
    );
    assert!(
        report.render(ColorMode::Plain).contains(&stale.to_string()),
        "the report does not state the recorded emitter set: {}",
        report.render(ColorMode::Plain)
    );
    assert!(report.has_errors(), "a producer difference passed the run");
}

/// A committed descriptor that records no emitter set is one an earlier engine
/// wrote, and absence is not disagreement.
///
/// The distinction matters at exactly one moment: the first run of an engine
/// that carries the member over a repository whose descriptor predates it. If
/// absence read as a difference, every adopter would meet the guard on upgrade
/// with no second number to compare and no way to clear it. The contract says
/// the run behaves as it always did, and this holds it to that.
#[test]
fn a_descriptor_that_records_no_emitter_set_is_not_a_producer_difference() {
    let (built, root) = fixture_tree();
    let surface = built.surface();
    let projections = Projections::read(&root).expect("the projections read");
    let plan = plan(
        &surface,
        &built.census,
        &projections,
        &fixture_identity(),
        &Runs::default(),
        headwater_verbs::VERBS,
    );

    let tree = empty_tree("producer-identity-absent");
    write(&tree, &plan);
    let at = tree.join(descriptor::PATH);
    let committed = std::fs::read_to_string(&at).expect("the written descriptor");
    let older: String = committed
        .lines()
        .filter(|line| !line.contains("\"emitter_set\""))
        .map(|line| format!("{line}\n"))
        .collect();
    assert_ne!(older, committed, "the descriptor records no emitter set");
    std::fs::write(&at, &older).expect("the older descriptor");

    let report = check(&tree, &plan);
    assert_eq!(report.producer, None, "{}", report.render(ColorMode::Plain));
    let remedy = report.remedy().expect("the descriptor itself is now stale");
    assert!(
        remedy.contains("headwater generate"),
        "an ordinary stale projection lost its remedy: {remedy}"
    );
}

/// No label a generating emitter writes is identifier-shaped.
///
/// This is the property [#627](https://github.com/headwater-ai/headwater/issues/627)
/// exists to hold. A projection that takes over the path of a document declares
/// an `identity`, and until that block could carry the `name`-role facet the
/// document's own kind requires, every emitter that labels a document fell
/// through to its identifier. A visitor to `/spec/` was then offered a row
/// reading `HW-REG-open-questions`, and the same string reached the sidebar and
/// the browser tab, because `crate::label` reads `pointer.name` first and there
/// was no name to read.
///
/// # Two surfaces, one property, derived from the plan
///
/// The nav leaves of `.headwater/nav.yml` and the rows of every generated
/// `shelf_index` page. Both are read out of the plan rather than out of the
/// served HTML, for two measured reasons. `tools/check-site-fragments.py` does
/// not capture anchor text, so a served-HTML property would be new state on the
/// parser that the fragment pass shares; and 881 of 100,337 served anchors
/// carry identifier link text, every one a legitimate prose citation, so the
/// same property over served anchors fires on 881 true anchors on a clean tree.
/// A generated artifact holds emitter output alone, so the property is exact
/// here and needs no parser.
///
/// # Both denominators are printed
///
/// A property over an empty set reports as a pass, and this one is written over
/// two sets that a refactor could empty. The assertion names the population it
/// read, and a lower bound on each one fails the day an extractor stops
/// matching.
///
/// # Watched failing
///
/// At `origin/main` before this change it failed at 3 identifier-shaped nav
/// leaves of 333 (`HW-RESULT-…`, `HW-RUN-…`, `HW-REG-open-questions`) and 3
/// identifier-shaped shelf-index rows of 202 across 12 generated index pages.
#[test]
fn no_label_a_generating_emitter_writes_is_identifier_shaped() {
    let root = repository_root();
    let resolved = headwater_resolve::repository(&root)
        .unwrap_or_else(|errors| panic!("{}", headwater_resolve::render_errors(&errors)));
    let corpus = Corpus::declared(
        &root,
        &resolved.consumer.corpus_root,
        &resolved.consumer.exclusions,
    );
    let built = Built::over(&corpus, &resolved.resolution.taxonomy);
    let projections =
        Projections::read(&resolved.resolution.taxonomy).expect("the projections read");
    let plan: Plan = plan(
        &built.surface(),
        &built.census,
        &projections,
        &Identity::default(),
        &Runs::default(),
        headwater_verbs::VERBS,
    );

    // The navigation. A leaf is `- "<label>": "<path>.md"`; a group key carries
    // no path and labels a shelf rather than a document.
    let nav = plan
        .outputs
        .iter()
        .find(|output| output.path == ".headwater/nav.yml")
        .map(|output| output.bytes.clone())
        .expect("the navigation is planned");
    let mut nav_leaves = Vec::new();
    for line in nav.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("- \"") else {
            continue;
        };
        let Some((label, path)) = rest.split_once("\": \"") else {
            continue;
        };
        if path.ends_with(".md\"") {
            nav_leaves.push(label.to_string());
        }
    }
    let nav_identifiers: Vec<&String> = nav_leaves
        .iter()
        .filter(|label| is_identifier_shaped(label))
        .collect();

    // Every generated shelf index. A row is `- [<label>](<path>)`.
    let mut index_pages = 0usize;
    let mut index_rows: Vec<String> = Vec::new();
    for output in &plan.outputs {
        if output.kind != Kind::ShelfIndex {
            continue;
        }
        index_pages += 1;
        for line in output.bytes.lines() {
            let Some(rest) = line.trim().strip_prefix("- [") else {
                continue;
            };
            let Some((label, _)) = rest.split_once("](") else {
                continue;
            };
            index_rows.push(label.to_string());
        }
    }
    let index_identifiers: Vec<&String> = index_rows
        .iter()
        .filter(|label| is_identifier_shaped(label))
        .collect();

    assert!(
        nav_leaves.len() > 300 && index_pages > 5 && index_rows.len() > 150,
        "this case read {} nav leaves and {} rows across {index_pages} generated index pages, \
         which is too few to be this corpus. The extractor is reading the wrong shape.",
        nav_leaves.len(),
        index_rows.len()
    );
    assert!(
        nav_identifiers.is_empty(),
        "{} of {} nav leaves are labelled with an identifier: {:?}. A generated label falls \
         through to the identifier when the document declares no `name`-role facet.",
        nav_identifiers.len(),
        nav_leaves.len(),
        nav_identifiers
    );
    assert!(
        index_identifiers.is_empty(),
        "{} of {} shelf-index rows across {index_pages} generated index pages are labelled with \
         an identifier: {:?}.",
        index_identifiers.len(),
        index_rows.len(),
        index_identifiers
    );
}

/// Whether a label is an identifier of this corpus rather than a name.
///
/// Every identifier scheme of this repository mints `HW-<SCHEME>-<rest>`, and
/// no name a person writes opens that way. The test above quotes the labels it
/// rejects, so a false positive names itself.
fn is_identifier_shaped(label: &str) -> bool {
    let Some(rest) = label.strip_prefix("HW-") else {
        return false;
    };
    let Some((scheme, _)) = rest.split_once('-') else {
        return false;
    };
    !scheme.is_empty() && scheme.chars().all(|c| c.is_ascii_uppercase())
}
