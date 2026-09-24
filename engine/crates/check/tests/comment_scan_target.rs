// SPDX-License-Identifier: Apache-2.0
//! The decisive fixture for `headwater_graph::anchors::CommentScan`: a
//! `governs` edge onto a `test_site` anchor passes `headwater check` only
//! where the target file's comment cites the identifier of the document that
//! asserts the edge, and that identifier is minted.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship)
//! sets the floor this file meets: a check ships with a fixture it fails and
//! one it passes. `relation.target.unresolved` (`headwater_check::target`)
//! already had both for a `source-tree` anchor. This is its first fixture
//! over an anchor whose resolver reads the bytes at the target rather than
//! only asking whether the target exists, which is the property
//! [HW-DR-0073](../../../../docs/decisions/0073-a-verification-is-a-kind-and-its-identity-is-minted-rather-than-found-in-the-code-that-cites-it.md)
//! ruling 4 asked for.
//!
//! `engine/crates/check/fixtures/comment-scan-target-src/` holds four files
//! that differ only in which identifiers their opening comment cites.
//! `engine/crates/check/fixtures/comment-scan-target/` holds the four
//! documents that `governs` them, one each. The trees are otherwise
//! identical, so a difference in what `run` reports for two arms can come
//! from nothing but the resolver reading those lines against the identifier
//! of the document that asserts the edge.
//!
//! | arm | asserter | the file cites | expected |
//! |---|---|---|---|
//! | `minted` | `HW-VER-0001` | `HW-VER-0001` | clean |
//! | `unminted` | `HW-VER-9999` | `HW-VER-9999`, never minted | reported, names the near miss |
//! | `other` | `HW-VER-0003` | `HW-VER-0002`, minted | reported, names both |
//! | `two` | `HW-VER-0004` | `HW-VER-0002`, then `HW-VER-0004` | clean |
//!
//! `other` is the arm #967 found: before it, any minted identifier in the
//! file bound the edge, whoever asserted it.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Register, Run, Shape};
use headwater_graph::anchors::{CommentScan, Resolvers};
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const PINNED: &str = "2026-08-12";
const RULE: &str = headwater_check::target::RULE;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// Every claim [`CommentScan`] treats as minted, for this test alone: every
/// identifier the four arms name except `HW-VER-9999`, the one the
/// `unminted` arm exists to refuse. `engine/crates/cli/src/main.rs` is what reads a real claim store off
/// disk; this file holds the resolver's own behavior once it has a set to
/// check against.
fn minted() -> BTreeSet<String> {
    ["HW-VER-0001", "HW-VER-0002", "HW-VER-0003", "HW-VER-0004"]
        .into_iter()
        .map(String::from)
        .collect()
}

fn run() -> Run {
    let corpus = Corpus::new(fixtures_dir(), "comment-scan-target");
    let source = std::fs::read_to_string(fixtures_dir().join("comment-scan-target.taxonomy.yml"))
        .expect("the fixture taxonomy");
    let root = headwater_yaml::load(&source)
        .expect("the fixture taxonomy loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();

    let taxonomy = Taxonomy::read(&root).expect("the taxonomy reads");
    let declarations = Declarations::read(&root).expect("the declarations read");
    let register = Register::read(&root).expect("the register reads");
    let shape = Shape::read(&root).expect("the shape reads");
    let taken = census::take(&corpus, &taxonomy);
    let config = Config::default();
    let resolvers = Resolvers::over(&corpus)
        .with(Box::new(CommentScan::new(
            fixtures_dir(),
            "HW-VER-",
            minted(),
        )))
        .expect("comment-scan is the only resolver this fixture adds");
    let graph = Graph::build(&taken, &declarations, &resolvers, &corpus, &config);
    headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: "sha256:comment-scan-target-fixture",
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            observations: &headwater_check::Observations::empty(),
            adoption: None,
            source: "engine/crates/check/fixtures/comment-scan-target.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

fn finding<'a>(run: &'a Run, arm: &str) -> Option<&'a headwater_check::Finding> {
    let prefix = format!("comment-scan-target/{arm}/");
    run.findings
        .iter()
        .find(|finding| finding.rule == RULE && finding.path.starts_with(&prefix))
}

fn reported(run: &Run, arm: &str) -> bool {
    finding(run, arm).is_some()
}

/// The decisive fixture. An identifier this taxonomy never minted, cited in
/// the comment `governs` names, is refused and named; the same edge onto a
/// file citing the identifier the fixture treats as minted resolves clean.
#[test]
fn an_unminted_citation_fails_and_a_minted_one_passes() {
    let run = run();

    assert!(
        reported(&run, "unminted"),
        "an unminted citation was not reported: {:#?}",
        run.findings
    );
    assert!(
        !reported(&run, "minted"),
        "a minted citation was reported: {:#?}",
        run.findings
    );

    let finding = finding(&run, "unminted").expect("the unminted arm reported the rule");
    assert!(
        finding.message.contains("HW-VER-9999"),
        "the refusal does not name the near miss: {}",
        finding.message
    );
}

/// The decisive arm of #967. The file cites `HW-VER-0002`, which is minted,
/// and the document that asserts the edge is `HW-VER-0003`. A citation of a
/// different document proves nothing about this one, so the edge is refused,
/// and the refusal names both identifiers.
#[test]
fn a_citation_of_a_different_minted_identifier_is_refused_and_names_both() {
    let run = run();

    let finding = finding(&run, "other").unwrap_or_else(|| {
        panic!(
            "a citation of another document's identifier bound the edge: {:#?}",
            run.findings
        )
    });
    assert!(
        finding.message.contains("HW-VER-0002") && finding.message.contains("HW-VER-0003"),
        "the refusal does not name both the cited identifier and the asserter: {}",
        finding.message
    );
}

/// Order does not matter: the asserter's identifier binds the edge wherever
/// it sits among the citations, and a different identifier before it does
/// not stop the scan.
#[test]
fn the_asserter_cited_after_another_identifier_still_binds() {
    let run = run();

    assert!(
        !reported(&run, "two"),
        "the asserter's own citation, second in the file, was not read: {:#?}",
        run.findings
    );
}
