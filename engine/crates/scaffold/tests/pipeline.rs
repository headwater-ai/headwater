// SPDX-License-Identifier: Apache-2.0
//! The claim [#70](https://github.com/headwater-ai/headwater/issues/70) is
//! about, tested rather than argued.
//!
//! "Its output goes through the same validation pipeline as authored input:
//! that the output is generated is never a reason to trust it."
//!
//! So this test scaffolds into a copy of the fixture corpus and then runs the
//! check layer over the result. Two assertions, and the second is the one that
//! matters.
//!
//! 1. No finding names a document this run wrote.
//! 2. No finding names a document this run *edited*. The reciprocal half goes
//!    into a file an author wrote, and a splice that broke the document at the
//!    far end would be the wrong-edge-at-scale failure that
//!    [spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots)
//!    names, arriving one document away from where anybody would look.
//!
//! **A green run proves nothing on its own**, so the test also records the
//! whole finding set of the run. A rule that stopped generating instances over
//! the scaffolded documents would make this test pass by reading nothing, and
//! the recorded instance count is what reports that.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-scaffold --test pipeline
//!
//! # Why it copies the tree
//!
//! The fixture corpus is committed, and a scaffolder writes. A test that wrote
//! into the committed tree would leave the next run a different corpus, so this
//! copies to a temporary directory, scaffolds there, and checks there. The
//! directory is named from the process identifier and removed at the end.

use headwater_census::census::{self, Census};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::scope::Digests;
use headwater_check::shape::Shape;
use headwater_check::{
    coverage, declaration, duplicate, endpoint, facet_required, facet_value, fragment, identity,
    language, participation, placement, reciprocity, retired, sections, source_form, target, voice,
    Cache, Context, Date, Declared, DocumentCheck, EdgeCheck, Register, Run,
};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::index::Index;
use headwater_graph::{Config, Graph};
use headwater_scaffold::{fix, propose, write, Request, Sources};
use headwater_yaml::Mapping;
use std::path::{Path, PathBuf};

const PINNED: &str = "2026-08-14";
const TAXONOMY: &str = "engine/crates/scaffold/fixtures/scaffold.taxonomy.yml";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
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

/// A copy of the fixture corpus, under a directory this test owns.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let at = std::env::temp_dir().join(format!("headwater-scaffold-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        copy_tree(&fixtures_dir().join("corpus"), &at.join("corpus"));
        Scratch(at)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the scratch directory");
    for entry in std::fs::read_dir(from)
        .expect("the fixture corpus")
        .flatten()
    {
        let target = to.join(entry.file_name());
        match entry.path().is_dir() {
            true => copy_tree(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("a fixture file");
            }
        }
    }
}

/// One run of the whole check layer over a tree, in the order spec 6 draws it.
fn check_over(root: &Path) -> Run {
    let resolved = load_map(&fixtures_dir().join("scaffold.taxonomy.yml"));
    let taxonomy = Taxonomy::read(&resolved).expect("the shelves read");
    let declarations = Declarations::read(&resolved).expect("the relations read");
    let register = Register::read(&resolved).expect("the register reads");
    let shape = Shape::read(&resolved).expect("the shape reads");
    let corpus = Corpus::new(root.to_path_buf(), "corpus");
    let taken = census::take(&corpus, &taxonomy);
    let config = Config::default();
    let graph = Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(&corpus),
        &corpus,
        &config,
    );
    let source = std::fs::read_to_string(fixtures_dir().join("scaffold.taxonomy.yml"))
        .expect("the fixture taxonomy");
    headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: &headwater_hash::hex(source.as_bytes()),
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            adoption: None,
            source: TAXONOMY,
        },
        &headwater_check::claim::Claims::at(root),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

/// Scaffold, then check, and record what the checks made of the result.
#[test]
fn what_the_scaffolder_wrote_passes_the_engines_own_checks() {
    let scratch = Scratch::new();
    let root = &scratch.0;

    // Bootstrap the claim store, the way `headwater check --fix` does over a
    // corpus that minted before it had one. The fixture tree ships documents
    // and no claims, and every one of those identifiers is spent, so a run
    // that skipped this would report the fixture's own documents rather than
    // the scaffolder's output.
    let bootstrap = fix::compose(
        root,
        &check_over(root)
            .findings
            .iter()
            .filter_map(|finding| finding.patch.clone())
            .collect::<Vec<_>>(),
    );
    assert!(
        !bootstrap.created.is_empty(),
        "the fixture corpus spends identifiers that no claim covers, and the \
         fixer offered no claim, so this bootstrap did nothing"
    );
    fix::make(root, &bootstrap.created).expect("the claims are made");

    // Three runs, because the three things a scaffolder writes are a document,
    // an edge, and a half of an edge in somebody else's document.
    type Ask = (&'static str, &'static str, Vec<(String, String)>);
    let requests: Vec<Ask> = vec![
        ("design_spec", "A scaffolded fourth part", vec![]),
        (
            "decision_record",
            "A scaffolded successor",
            vec![(
                "supersedes".to_string(),
                "SPEC-FIX-the-second-part".to_string(),
            )],
        ),
        (
            "decision_record",
            "A second successor",
            vec![("supersedes".to_string(), "DR-FIX-0007".to_string())],
        ),
    ];

    let mut touched: Vec<String> = Vec::new();
    for (kind, title, relates) in &requests {
        // Reload after every run. A scaffolder reads the corpus it is writing
        // into, so a second run has to see what the first one wrote — which is
        // also what makes the identifier allocator honest across two runs.
        let resolved = load_map(&fixtures_dir().join("scaffold.taxonomy.yml"));
        let shelves = Taxonomy::read(&resolved).expect("the shelves read");
        let relations = Declarations::read(&resolved).expect("the relations read");
        let shape = Shape::read(&resolved).expect("the shape reads");
        let corpus = Corpus::new(root.clone(), "corpus");
        let taken: Census = census::take(&corpus, &shelves);
        let config = Config::default();
        let index = Index::build(&taken, &config);
        // Re-read with the corpus, and for the same reason: the allocator's
        // upper bound is the corpus and the store together, so a second run has
        // to see the claim the first one made.
        let claims = headwater_check::claim::Claims::at(root);
        let sources = Sources {
            resolved: &resolved,
            shape: &shape,
            shelves: &shelves,
            relations: &relations,
            census: &taken,
            index: &index,
            config: &config,
            claims: &claims,
        };
        let request = Request {
            kind,
            title,
            now: Date::parse(PINNED).expect("the pinned date"),
            relates,
            given: &[],
        };
        let plan = propose(&sources, &request)
            .unwrap_or_else(|refusal| panic!("{kind} `{title}`: {refusal}"));
        let composed =
            write::compose(root, &plan).unwrap_or_else(|refusal| panic!("{kind}: {refusal}"));
        for file in &composed {
            touched.push(file.path.clone());
        }
        // The claim first and the document second, as the verb does it.
        headwater_scaffold::claim::write(root, &plan).expect("the claim writes");
        write::apply(root, &composed).expect("the files write");
    }

    assert_eq!(
        touched.len(),
        5,
        "three documents and two spliced far halves: {touched:?}"
    );

    let run = check_over(root);
    let report = run.render(
        headwater_check::Detail::EveryInstance,
        headwater_check::paint::ColorMode::Plain,
    );

    // The assertion the issue asks for.
    let against_scaffolded: Vec<String> = run
        .findings
        .iter()
        .filter(|finding| touched.contains(&finding.path))
        .map(|finding| format!("{}:{} {}", finding.path, finding.line, finding.message))
        .collect();
    assert!(
        against_scaffolded.is_empty(),
        "the scaffolder's own output does not pass the engine's own checks:\n{}",
        against_scaffolded.join("\n")
    );

    // And the guard against a green run that read nothing. The recorded report
    // carries the instance count and the coverage account, so a rule that
    // stopped generating over these documents changes this file.
    let mut recorded = String::new();
    for path in &touched {
        recorded.push_str(&format!("touched {path}\n"));
    }
    recorded.push('\n');
    recorded.push_str(&report);
    compare(&fixtures_dir().join("scaffold.pipeline"), &recorded);
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
        "\nthe run over the scaffolded tree no longer matches {}",
        recorded.display()
    );
}

/// Silence the unused-import warnings that the check crate's wide surface
/// brings in. Every name above is part of one pipeline, and a narrower import
/// list would hide which phase this test runs.
#[allow(dead_code)]
fn unused() {
    let _ = (
        coverage::RULE,
        declaration::RULE,
        duplicate::RULE,
        endpoint::RULE,
        facet_required::RULE,
        facet_value::RULE,
        fragment::RULE,
        identity::RULE,
        language::RULE,
        participation::RULE,
        placement::RULE,
        reciprocity::RULE,
        retired::RULE,
        sections::RULE,
        source_form::RULE,
        target::RULE,
        voice::RULE,
    );
    fn _traits<D: DocumentCheck, E: EdgeCheck>() {}
    let _: Option<Digests> = None;
}
