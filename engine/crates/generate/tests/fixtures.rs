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
use headwater_generate::{
    check, descriptor, plan, write, Emitter, Identity, Plan, Projections, Report, Verdict,
};
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
    let first = plan(&surface, &built.census, &projections, &fixture_identity());
    assert!(
        first.orphaned.is_empty(),
        "the fixture tree holds no marked file that nothing writes: {:?}",
        first.orphaned
    );

    let tree = empty_tree("censused");
    let report = write(&tree, &first);
    assert!(!report.has_errors(), "{}", report.render());

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
    let stale = plan(&surface, &again.census, &projections, &fixture_identity());
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
            headwater_mark::carries_marker(&output.path, &output.bytes),
            "{} opens with no generated-file marker",
            output.path
        );
    }
}

/// This repository generates its two artifacts, and it says why for everything
/// else.
///
/// A property and not a recording, for the reason the query crate states about
/// its own repository run: the corpus is prose somebody edits. What is asserted
/// is what a prose edit must not change. Four files are written. The descriptor
/// sits at the path Q14 fixes, and the index of the specification shelf sits at
/// the path this repository's overlay declares. That second one is the list the
/// root README used to carry by hand. The third is the index of the decisions
/// shelf, which the package has declared since the first-run walkthrough and
/// which produced a reason rather than a file until #124 filled that shelf. The
/// package still declares an index for one shelf this tree holds no document on,
/// so that one produces a reason, and the register produces a second.
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
fn this_repository_generates_its_four_artifacts_and_accounts_for_the_rest() {
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
        vec![
            "docs/decisions/README.md",
            "docs/spec/README.md",
            "docs/spec/09-open-questions.md",
            descriptor::PATH
        ],
        "this repository writes the decisions index, the specification index, the \
         redirect map and the descriptor, in that order"
    );
    // One declared shelf that holds no document, and the register. Nothing is
    // passed over: a projection that produced no file states a reason.
    assert_eq!(
        plan.unwritten.len(),
        2,
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
///    `REG-HW-open-questions` would resolve to nothing.
/// 3. **The reciprocal half of each of those edges is in the block**, derived
///    and never declared. Two documents supersede this one, `supersedes` says
///    `reciprocal: required`, and a file that omitted the halves would report
///    two findings against documents nobody edited.
///
/// The anchors are computed here rather than listed, so a decision renamed in
/// its own document fails this test at the citation rather than in a reader's
/// browser.
#[test]
fn the_redirect_map_keeps_every_anchor_that_this_corpus_cites_into_it() {
    let root = repository_root();
    let map = std::fs::read_to_string(root.join("docs/spec/09-open-questions.md"))
        .expect("the redirect map is committed");

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
        "id: REG-HW-open-questions",
        "doc_type: decision_register",
        "  superseded_by:",
        "    - REG-HW-decisions",
        "    - REG-HW-open-obligations",
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
    let plan = plan(&surface, &built.census, &projections, &fixture_identity());
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
        for facet in document.facets.iter() {
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
