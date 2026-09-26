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
    /// Keyed on the test as well as the process, because cargo runs the tests
    /// of one file as threads of one process.
    fn new(test: &str) -> Self {
        let at =
            std::env::temp_dir().join(format!("headwater-scaffold-{}-{test}", std::process::id()));
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
            observations: &headwater_check::Observations::empty(),
            adoption: None,
            source: TAXONOMY,
        },
        &headwater_check::claim::Claims::at(root),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

/// The two documents the successors supersede, which a scaffolder at the
/// initial state leaves alone (HW-DR-0086).
const FAR: [&str; 2] = [
    "corpus/decisions/an-earlier-decision.md",
    "corpus/spec/02-the-second-part.md",
];

/// Bootstrap the claim store, then run the scaffolder three times into the
/// tree, and return every path it composed, in the order it composed them.
fn scaffold_into(root: &Path) -> Vec<String> {
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
        let corpus = Corpus::new(root.to_path_buf(), "corpus");
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
            summary: None,
            now: Date::parse(PINNED).expect("the pinned date"),
            relates,
            given: &[],
            directory: None,
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
    touched
}

fn read(root: &Path, path: &str) -> String {
    std::fs::read_to_string(root.join(path)).unwrap_or_else(|why| panic!("{path}: {why}"))
}

/// Scaffold, then check, and record what the checks made of the result.
#[test]
fn what_the_scaffolder_wrote_passes_the_engines_own_checks() {
    let scratch = Scratch::new("passes");
    let root = &scratch.0;

    let before: Vec<String> = FAR.iter().map(|path| read(root, path)).collect();
    let touched = scaffold_into(root);

    // The decisive assertion of #1168. Each new document opens at `draft`, so
    // the far half of each `supersedes` is owed at promotion and not written
    // now: only the three new documents are composed, and the bytes of the
    // documents they supersede do not move.
    assert_eq!(
        touched.len(),
        3,
        "three documents and no spliced far half: {touched:?}"
    );
    for (path, before) in FAR.iter().zip(&before) {
        assert_eq!(
            &read(root, path),
            before,
            "{path} is a live document, and a draft wrote nothing into it"
        );
    }

    // The decisive assertion of
    // [#584](https://github.com/headwater-ai/headwater/issues/584), and it is
    // about the shelf rather than the scheme. `design_spec` mints under
    // `spec_id`, which allocates `minted-once`, onto `spec_series`, which
    // declares `{sequence:02d}-{slug}.md`. While the store covered the
    // `reconcile-first` schemes alone, `claim::write` returned `None` here and
    // no file was written at all, so two branches minting one `SPEC-FIX-…` at
    // two sequences wrote two paths and git merged both without a word.
    let claim = root.join(".headwater/ids/spec_id/SPEC-FIX-a-scaffolded-fourth-part");
    assert_eq!(
        std::fs::read_to_string(&claim).unwrap_or_else(|why| panic!(
            "{}: {why}. A mint onto a layout-declaring shelf claims its identifier.",
            claim.display()
        )),
        "corpus/spec/04-a-scaffolded-fourth-part.md\n",
        "the claim names the document that minted the identifier"
    );

    // And the other half of the predicate, unmoved. `decision_record` mints
    // under a `reconcile-first` scheme onto `decisions`, which declares no
    // layout, so a predicate that read the shelf alone would have taken this
    // claim away.
    let reconciled = root.join(".headwater/ids/decision_id/DR-FIX-0008");
    assert!(
        reconciled.exists(),
        "{}: a reconcile-first mint claims its identifier whatever its shelf declares",
        reconciled.display()
    );

    let run = check_over(root);
    let report = run.render(
        headwater_check::Detail::EveryInstance,
        headwater_check::paint::ColorMode::Plain,
    );

    // No live document names a draft. Before #1168 the scaffolder spliced the
    // required `superseded_by` into each live document a successor replaces,
    // and `lifecycle.dependency.on_initial` reads each half from the document
    // that wrote it (HW-DR-0085), so each live document warned that it rested
    // on a draft. This fixture's `supersedes` writes no state onto its target,
    // so the relation is not exempt. The far half is now owed at promotion
    // (HW-DR-0086), so nothing is spliced and nothing warns.
    let on_initial: Vec<&str> = run
        .findings
        .iter()
        .filter(|finding| finding.rule == headwater_check::initial_dependency::RULE)
        .filter(|finding| touched.contains(&finding.path) || FAR.contains(&finding.path.as_str()))
        .map(|finding| finding.path.as_str())
        .collect();
    assert_eq!(
        on_initial,
        Vec::<&str>::new(),
        "a draft writes nothing into a live document, so none of them warns"
    );

    // The assertion the issue asks for, with no rule exempt. A reciprocity
    // check that did not defer would anchor its finding on a new draft here.
    let against_scaffolded: Vec<String> = run
        .findings
        .iter()
        .filter(|finding| touched.contains(&finding.path) || FAR.contains(&finding.path.as_str()))
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

/// Promote what the scaffolder wrote, and the owed halves come due: the check
/// reports each one on the new document and `--fix` writes it into the far
/// one (HW-DR-0086).
#[test]
fn a_promoted_successor_is_owed_its_far_half_and_the_fix_writes_it() {
    let scratch = Scratch::new("promoted");
    let root = &scratch.0;
    let touched = scaffold_into(root);

    // The author's promotion, by hand, as HW-DR-0052 has it.
    let mut promoted = Vec::new();
    for path in &touched {
        let text = read(root, path);
        if text.contains("\nstatus: draft\n") {
            let text = text.replacen("\nstatus: draft\n", "\nstatus: current\n", 1);
            std::fs::write(root.join(path), text).expect("the promotion writes");
            promoted.push(path.clone());
        }
    }
    let successors = [
        "corpus/decisions/a-scaffolded-successor.md",
        "corpus/decisions/a-second-successor.md",
    ];
    for successor in successors {
        assert!(
            promoted.iter().any(|path| path == successor),
            "{successor} opened at `draft`: {promoted:?}"
        );
    }

    let run = check_over(root);
    let mut owed: Vec<(&str, String)> = run
        .findings
        .iter()
        .filter(|finding| finding.rule == reciprocity::RULE)
        .map(|finding| (finding.path.as_str(), finding.remediation.clone()))
        .collect();
    owed.sort();
    assert_eq!(owed.len(), 2, "one owed half per successor: {owed:?}");
    for ((path, remediation), (successor, far)) in owed
        .iter()
        .zip([(successors[0], FAR[1]), (successors[1], FAR[0])])
    {
        assert_eq!(*path, successor);
        assert!(
            remediation.contains(far) && remediation.contains("superseded_by"),
            "{successor} names {far} and `superseded_by`: {remediation}"
        );
    }

    let patches: Vec<_> = run
        .findings
        .iter()
        .filter(|finding| finding.rule == reciprocity::RULE)
        .filter_map(|finding| finding.patch.clone())
        .collect();
    assert_eq!(patches.len(), 2, "each owed half carries its patch");
    let composed = fix::compose(root, &patches);
    assert!(composed.refused.is_empty(), "{:?}", composed.refused);
    fix::apply(root, &composed.files).expect("the halves write");
    fix::make(root, &composed.created).expect("nothing to make");

    for (far, id) in [(FAR[0], "DR-FIX-0009"), (FAR[1], "DR-FIX-0008")] {
        let text = read(root, far);
        assert!(
            text.contains("superseded_by") && text.contains(id),
            "{far} carries `superseded_by: {id}`:\n{text}"
        );
    }

    // Both ends are live now, so neither rule has anything to say.
    let again = check_over(root);
    let left: Vec<String> = again
        .findings
        .iter()
        .filter(|finding| touched.contains(&finding.path) || FAR.contains(&finding.path.as_str()))
        .map(|finding| format!("{} {}: {}", finding.path, finding.rule, finding.message))
        .collect();
    assert!(left.is_empty(), "{}", left.join("\n"));
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
