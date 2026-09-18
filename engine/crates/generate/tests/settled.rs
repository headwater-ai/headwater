// SPDX-License-Identifier: Apache-2.0
//! One `headwater generate` leaves a tree that `generate --check` accepts
//! ([#842](https://github.com/headwater-ai/headwater/issues/842)).
//!
//! # The shape
//!
//! `fixtures/chained.taxonomy.yml` declares a `shelf_sections` whose output is a
//! document with a composed summary, and a `shelf_index` that prints that
//! summary. Adding a decision moves the count in the summary, and one plan reads
//! the summary from before its own write. So one write left the index one count
//! behind, and a second write repaired it. Two writes agreeing is therefore no
//! evidence: they agreed while the defect was live. What this file asserts is
//! that one call leaves a tree a check accepts.
//!
//! # The instrument before the measurement
//!
//! [`one_plan_written_once_leaves_the_index_behind`] runs a single
//! [`headwater_generate::write`] over the same tree, and it must leave a
//! difference. Without it, a fixture that stopped reaching the chain would make
//! the case below pass for the wrong reason.
//!
//! # Watched failing
//!
//! With `write_settled` reduced to one pass,
//! [`one_call_after_adding_a_document_leaves_a_tree_the_check_accepts`] failed
//! on `chained/registers/README.md`, committed with the count 2 where the
//! register says 3.

use headwater_census::census::{self, Census};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::Shape;
use headwater_generate::{
    check, plan, write, write_settled, Identity, Kind, Output, Plan, Projections, Report, Runs,
    Verdict, PASSES,
};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_query::Surface;
use headwater_yaml::Mapping;
use std::convert::Infallible;
use std::path::{Path, PathBuf};

/// The output that prints a summary another output composes.
const INDEX: &str = "chained/registers/README.md";
/// The output whose summary carries the count.
const REGISTER: &str = "chained/registers/open.md";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn taxonomy() -> Mapping {
    let path = fixtures_dir().join("chained.taxonomy.yml");
    let source =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    headwater_yaml::load(&source)
        .unwrap_or_else(|errors| panic!("{}: {errors:?}", path.display()))
        .value
        .as_map()
        .unwrap_or_else(|| panic!("{} is not a mapping", path.display()))
        .clone()
}

fn identity() -> Identity {
    Identity {
        corpus_root: "chained".to_string(),
        exclusions: Vec::new(),
        package: "headwater/fixture".to_string(),
        version: "1.0.0".to_string(),
        lock: "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string(),
    }
}

/// The fixture corpus copied under a directory of its own, because a write
/// changes the tree it reads. `label` names the test, since cargo runs the
/// cases of one target as threads of one process.
fn scratch(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "headwater-generate-settled-{}-{label}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let from = fixtures_dir().join("chained/decisions");
    let to = root.join("chained/decisions");
    std::fs::create_dir_all(&to).expect("the scratch corpus is created");
    for entry in std::fs::read_dir(&from).expect("the fixture corpus reads") {
        let entry = entry.expect("a fixture entry reads");
        std::fs::copy(entry.path(), to.join(entry.file_name())).expect("a fixture copies");
    }
    root
}

/// A plan over the tree as it is on disk now, read the way the command line
/// reads it: census, graph, surface, plan.
fn planned(root: &Path) -> Plan {
    let taxonomy_map = taxonomy();
    let corpus = Corpus::new(root.to_path_buf(), "chained");
    let taxonomy = Taxonomy::read(&taxonomy_map).expect("the taxonomy reads");
    let relations = Declarations::read(&taxonomy_map).expect("the declarations read");
    let shape = Shape::read(&taxonomy_map).expect("the shape reads");
    let census: Census = census::take(&corpus, &taxonomy);
    let config = Config::default();
    let graph = Graph::build(
        &census,
        &relations,
        &Resolvers::over(&corpus),
        &corpus,
        &config,
    );
    let surface = Surface::over(&census, &graph, &shape, &taxonomy, &relations, &config);
    let projections = Projections::read(&taxonomy_map).expect("the projections read");
    plan(
        &surface,
        &census,
        &projections,
        &identity(),
        &Runs::default(),
        headwater_verbs::VERBS,
    )
}

fn settle(root: &Path) -> Report {
    match write_settled(root, || Ok::<_, Infallible>(planned(root))) {
        Ok(report) => report,
        Err(never) => match never {},
    }
}

/// A third decision, which moves the count the register composes.
fn add_a_decision(root: &Path) {
    std::fs::write(
        root.join("chained/decisions/0003-decision-3.md"),
        "---\nid: DR-FIX-0003\ntitle: Decision 3\nstatus: current\nstatus_since: 2026-05-03\n\
         summary: decision 3, the document this case adds\n---\n\n# Decision 3\n\nThe added one.\n",
    )
    .expect("the added decision writes");
}

/// The paths a check over the tree reports as anything but unchanged.
fn drifted(root: &Path) -> Vec<(String, Verdict)> {
    check(root, &planned(root))
        .wrote
        .into_iter()
        .filter(|wrote| wrote.verdict != Verdict::Unchanged)
        .map(|wrote| (wrote.path, wrote.verdict))
        .collect()
}

fn read(root: &Path, path: &str) -> String {
    std::fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The instrument. One plan, written once, over a settled tree with a decision
/// added, leaves the index behind the register it lists. If this stops
/// failing the check, the fixture no longer reaches the chain of #842.
#[test]
fn one_plan_written_once_leaves_the_index_behind() {
    let root = scratch("instrument");
    let settled = settle(&root);
    assert_eq!(settled.unsettled, None, "the fixture tree did not settle");
    assert!(
        drifted(&root).is_empty(),
        "the fixture tree is not settled before the case begins: {:?}",
        drifted(&root)
    );

    add_a_decision(&root);
    let once = write(&root, &planned(&root));
    assert!(!once.has_errors(), "one write failed: {:?}", once.wrote);

    let behind = drifted(&root);
    assert!(
        behind.iter().any(|(path, _)| path == INDEX),
        "one write over the added decision left `{INDEX}` current, so this fixture no longer \
         reaches the chain it exists for and the case beside it proves nothing. The check \
         reported: {behind:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// The decisive case. One call after adding a document leaves a tree that a
/// check accepts, and the index prints the count the register now composes.
#[test]
fn one_call_after_adding_a_document_leaves_a_tree_the_check_accepts() {
    let root = scratch("settled");
    settle(&root);
    add_a_decision(&root);

    let report = settle(&root);
    assert_eq!(
        report.unsettled, None,
        "the run did not settle: {:?}",
        report.wrote
    );
    assert!(report.remedy().is_none(), "{:?}", report.remedy());

    let left = drifted(&root);
    assert!(
        left.is_empty(),
        "one `write_settled` left a tree that `generate --check` rejects: {left:?}. The index \
         reads:\n{}",
        read(&root, INDEX)
    );
    assert!(
        read(&root, REGISTER).contains("each of the 3 documents"),
        "the register does not count the added decision:\n{}",
        read(&root, REGISTER)
    );
    assert!(
        read(&root, INDEX).contains("each of the 3 documents"),
        "the index prints a count the register no longer composes:\n{}",
        read(&root, INDEX)
    );

    // Each path once, with what the first pass that changed it did. The index
    // changed on both passes, and its line says so once.
    let lines: Vec<&str> = report
        .wrote
        .iter()
        .filter(|wrote| wrote.path == INDEX)
        .map(|wrote| wrote.path.as_str())
        .collect();
    assert_eq!(lines, [INDEX], "{:?}", report.wrote);
    let _ = std::fs::remove_dir_all(&root);
}

/// A cycle, which no number of passes settles, is said on the run that did not
/// settle rather than left for `--check` to find.
///
/// No declaration of this engine forms one, so the plans are made by hand: each
/// pass writes bytes the previous pass did not.
#[test]
fn a_run_that_never_settles_says_so_and_fails() {
    let root = scratch("cycle");
    let mut turn = 0;
    let report = write_settled(&root, || {
        turn += 1;
        Ok::<_, Infallible>(Plan {
            outputs: vec![Output {
                path: "chained/flip.md".to_string(),
                kind: Kind::ShelfIndex,
                bytes: format!(
                    "{}\n\npass {turn}\n",
                    headwater_mark::marker(Kind::ShelfIndex.name(), "chained/flip.md")
                        .expect("Markdown carries a marker")
                ),
            }],
            ..Plan::default()
        })
    })
    .unwrap_or_else(|never| match never {});

    assert_eq!(turn, PASSES, "the run planned {turn} times");
    assert_eq!(report.unsettled, Some(PASSES));
    assert!(report.has_errors());
    let remedy = report.remedy().expect("an unsettled run names a remedy");
    assert!(
        remedy.contains("generate --check") && !remedy.contains("Run `headwater generate`"),
        "the remedy of an unsettled run tells the reader to run the verb that just failed to \
         settle: {remedy}"
    );
    let _ = std::fs::remove_dir_all(&root);
}
