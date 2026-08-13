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
/// the path Q14 fixes. The package declares indexes for two shelves this tree
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
    // and a facet with none gets no constraint. A `type: string` here would be
    // the emitter inventing a rule the native check does not enforce.
    let properties = decision
        .get("properties")
        .and_then(|node| node.value.as_map())
        .expect("properties");
    let status = properties
        .get("status")
        .and_then(|node| node.value.as_map())
        .expect("the status property");
    assert!(
        status.get("enum").is_some(),
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
