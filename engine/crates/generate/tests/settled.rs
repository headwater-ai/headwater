// SPDX-License-Identifier: Apache-2.0
//! One `headwater generate` leaves a tree that `generate --check` accepts
//! ([#842](https://github.com/headwater-ai/headwater/issues/842)).
//!
//! # The shape
//!
//! `fixtures/chained.taxonomy.yml` declares a `shelf_sections` whose output is a
//! document with a composed summary, and a `shelf_index` that prints that
//! summary. Until [#1058](https://github.com/headwater-ai/headwater/issues/1058)
//! the summary counted the shelf, so adding a decision moved it, and one plan
//! read the summary from before its own write. One write left the index one
//! count behind, and a second write repaired it.
//!
//! #1058 took the count out of every composed summary, because a count is a
//! fold that a text merge writes wrong. So no emitter of this engine forms that
//! chain now, and [`one_write_after_adding_a_document_leaves_nothing_behind`]
//! holds that over the same fixture.
//!
//! [`headwater_generate::write_settled`] stays, because a future emitter can
//! print what another one composes. The cases below hold it over a chain built
//! by hand: one output lists the decisions, and a second output copies the
//! first one as it stands on disk.
//!
//! # The instrument before the measurement
//!
//! [`one_plan_written_once_leaves_the_copy_behind`] runs a single
//! [`headwater_generate::write`] over the hand-built chain, and it must leave a
//! difference. Without it, a chain that stopped reaching two passes would make
//! the case below pass for the wrong reason.
//!
//! # Watched failing
//!
//! With `write_settled` reduced to one pass,
//! [`one_call_after_adding_a_document_leaves_a_tree_the_check_accepts`] failed
//! on `chained/registers/README.md`, committed with the count 2 where the
//! register said 3. That was the emitter chain before #1058.

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

/// A third decision, which moves what the shelf lists.
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

/// The emitter chain of #842 is gone. The index lists the register, and the
/// register's summary names its shelf and counts nothing, so one write over an
/// added decision leaves nothing behind. If this starts to fail, an emitter
/// prints a value that another one composes from the shelf again.
#[test]
fn one_write_after_adding_a_document_leaves_nothing_behind() {
    let root = scratch("one-write");
    let settled = settle(&root);
    assert_eq!(settled.unsettled, None, "the fixture tree did not settle");

    add_a_decision(&root);
    let once = write(&root, &planned(&root));
    assert!(!once.has_errors(), "one write failed: {:?}", once.wrote);
    let left = drifted(&root);
    assert!(
        left.is_empty(),
        "one write over the added decision left {left:?} behind, so an emitter reads what \
         another one writes in the same run. The index reads:\n{}",
        read(&root, INDEX)
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// The output of the hand-built chain that lists the decisions. Both outputs
/// sit outside the corpus root, so neither is a document the emitters read.
const LIST: &str = "hand-built/list.md";
/// The output of the hand-built chain that copies [`LIST`] from disk.
const COPY: &str = "hand-built/copy.md";

fn marked(path: &str, body: &str) -> String {
    format!(
        "{}\n\n{body}\n",
        headwater_mark::marker(Kind::ShelfIndex.name(), path).expect("Markdown carries a marker")
    )
}

/// A plan whose second output reads the first one off disk, which is the shape
/// of #842 without an emitter that forms it.
fn chain(root: &Path) -> Plan {
    let mut names: Vec<String> = std::fs::read_dir(root.join("chained/decisions"))
        .expect("the decisions read")
        .map(|entry| {
            entry
                .expect("an entry reads")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    let list = marked(LIST, &names.join("\n"));
    let copied = std::fs::read_to_string(root.join(LIST)).unwrap_or_default();
    let copy = marked(COPY, &copied.replace("<!--", "(").replace("-->", ")"));
    Plan {
        outputs: vec![
            Output {
                path: LIST.to_string(),
                kind: Kind::ShelfIndex,
                bytes: list,
            },
            Output {
                path: COPY.to_string(),
                kind: Kind::ShelfIndex,
                bytes: copy,
            },
        ],
        ..Plan::default()
    }
}

fn chain_drifted(root: &Path) -> Vec<(String, Verdict)> {
    check(root, &chain(root))
        .wrote
        .into_iter()
        .filter(|wrote| wrote.verdict != Verdict::Unchanged)
        .map(|wrote| (wrote.path, wrote.verdict))
        .collect()
}

fn settle_chain(root: &Path) -> Report {
    match write_settled(root, || Ok::<_, Infallible>(chain(root))) {
        Ok(report) => report,
        Err(never) => match never {},
    }
}

/// The instrument. One plan, written once, over a settled chain with a decision
/// added, leaves the copy behind the list it copies. If this stops failing the
/// check, the chain no longer needs two passes and the case beside it proves
/// nothing.
#[test]
fn one_plan_written_once_leaves_the_copy_behind() {
    let root = scratch("instrument");
    let settled = settle_chain(&root);
    assert_eq!(settled.unsettled, None, "the chain did not settle");
    assert!(
        chain_drifted(&root).is_empty(),
        "the chain is not settled before the case begins: {:?}",
        chain_drifted(&root)
    );

    add_a_decision(&root);
    let once = write(&root, &chain(&root));
    assert!(!once.has_errors(), "one write failed: {:?}", once.wrote);

    let behind = chain_drifted(&root);
    assert!(
        behind.iter().any(|(path, _)| path == COPY),
        "one write over the added decision left `{COPY}` current, so this chain no longer \
         needs a second pass and the case beside it proves nothing. The check reported: \
         {behind:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// The decisive case. One call after adding a document leaves a tree that a
/// check accepts, over the emitters and over the hand-built chain.
#[test]
fn one_call_after_adding_a_document_leaves_a_tree_the_check_accepts() {
    let root = scratch("settled");
    settle(&root);
    settle_chain(&root);
    add_a_decision(&root);

    for report in [settle(&root), settle_chain(&root)] {
        assert_eq!(
            report.unsettled, None,
            "the run did not settle: {:?}",
            report.wrote
        );
        assert!(report.remedy().is_none(), "{:?}", report.remedy());
    }

    let left = drifted(&root);
    assert!(
        left.is_empty(),
        "one `write_settled` left a tree that `generate --check` rejects: {left:?}. The index \
         reads:\n{}",
        read(&root, INDEX)
    );
    let left = chain_drifted(&root);
    assert!(
        left.is_empty(),
        "one `write_settled` left the chain behind: {left:?}. The copy reads:\n{}",
        read(&root, COPY)
    );
    assert!(
        read(&root, COPY).contains("0003-decision-3.md"),
        "the copy does not carry the added decision:\n{}",
        read(&root, COPY)
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Each path once in a report, with what the first pass that changed it did.
/// The copy changed on both passes, and its line says so once.
#[test]
fn a_path_two_passes_changed_is_reported_once() {
    let root = scratch("once");
    settle_chain(&root);
    add_a_decision(&root);
    let report = settle_chain(&root);
    let lines: Vec<&str> = report
        .wrote
        .iter()
        .filter(|wrote| wrote.path == COPY)
        .map(|wrote| wrote.path.as_str())
        .collect();
    assert_eq!(lines, [COPY], "{:?}", report.wrote);
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
