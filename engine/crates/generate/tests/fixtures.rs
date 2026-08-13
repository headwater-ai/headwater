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
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::Shape;
use headwater_generate::{check, descriptor, plan, write, Identity, Plan, Projections, Report, Verdict};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_query::Surface;
use headwater_yaml::Mapping;
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
        }
    }

    fn surface(&self) -> Surface<'_> {
        Surface::over(
            &self.census,
            &self.graph,
            &self.shape,
            &self.taxonomy,
            &self.relations,
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
        lock: "sha256:0000000000000000000000000000000000000000000000000000000000000000"
            .to_string(),
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
    out.push_str(&report.render());
    out.push_str(&format!("fails the run: {}\n", report.has_errors()));
}

#[test]
fn the_fixture_tree_generates_the_recorded_projections() {
    let (built, root) = fixture_tree();
    let surface = built.surface();
    let projections = Projections::read(&root).expect("the projections read");
    let plan = plan(&surface, &built.census, &projections, &fixture_identity());

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
    let one = plan(&surface, &built.census, &projections, &fixture_identity());
    let two = plan(&surface, &built.census, &projections, &fixture_identity());
    assert_eq!(one.outputs.len(), two.outputs.len());
    for (left, right) in one.outputs.iter().zip(&two.outputs) {
        assert_eq!(left.path, right.path);
        assert_eq!(left.bytes, right.bytes);
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
    let plan = plan(&surface, &built.census, &projections, &fixture_identity());
    assert!(!plan.outputs.is_empty(), "the fixture generates something");
    for output in &plan.outputs {
        assert!(
            headwater_generate::carries_marker(&output.path, &output.bytes),
            "{} opens with no generated-file marker",
            output.path
        );
    }
}

/// This repository generates its descriptor, and it says why for everything
/// else.
///
/// A property and not a recording, for the reason the query crate states about
/// its own repository run: the corpus is prose somebody edits. What is asserted
/// is what a prose edit must not change. One file is written, the descriptor at
/// the path Q20 fixes. The package declares indexes for two shelves this tree
/// holds no document on, so those two produce a reason rather than a file, and
/// the register produces a third.
///
/// The gate at the end is the dogfood. It reads the committed
/// `.headwater/corpus.json` and compares bytes, so a contributor who moves a
/// shelf and does not regenerate fails this test before CI runs.
#[test]
fn this_repository_generates_its_descriptor_and_accounts_for_the_rest() {
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
    let plan: Plan = plan(&surface, &built.census, &projections, &identity);

    let paths: Vec<&str> = plan.outputs.iter().map(|o| o.path.as_str()).collect();
    assert_eq!(
        paths,
        vec![descriptor::PATH],
        "this repository writes the descriptor and nothing else"
    );
    // Two declared shelves that hold no document, and the register. Nothing is
    // passed over: a projection that produced no file states a reason.
    assert_eq!(
        plan.unwritten.len(),
        3,
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
        "the committed descriptor is not what this corpus and this lock produce. \
         Run `headwater generate` and commit the result"
    );
    assert!(
        held.wrote
            .iter()
            .all(|wrote| wrote.verdict != Verdict::Occupied),
        "a declared output path is held by an authored document"
    );
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
    let plan = plan(&surface, &built.census, &projections, &fixture_identity());
    let written = plan
        .outputs
        .iter()
        .find(|output| output.path == descriptor::PATH)
        .expect("the descriptor is planned");

    // This engine's own output is recognized, so a second run overwrites it.
    assert!(
        headwater_generate::carries_marker(&written.path, &written.bytes),
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
