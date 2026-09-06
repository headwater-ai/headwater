// SPDX-License-Identifier: Apache-2.0
//! The promotion count, and the corpus that separates it from a count of
//! documents standing at `accepted`.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship)
//! sets the floor: "every check ships with at least one fixture that it fails
//! and one that it passes." A count is harder to hold to that bar than a defect
//! is, because a wrong count and a right one produce reports of the same shape.
//! So the tree under `fixtures/promotion/` is written to be wrong under exactly
//! the mistake this rule exists to avoid: `promoted.md` and `drafted.md` stand
//! at `accepted` and read alike, and only the prior version tells them apart.
//!
//! Four properties, and each one names the defect it fails under.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::change::{Change, Unbound};
use headwater_check::{Cache, Context, Date, Declared, Outcome, Register, Run, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

/// As every other recorded run: a verdict is a function of the injected clock.
const PINNED: &str = "2026-08-12";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// The manifest a caller writes. `promoted.md` carries a prior version and
/// `drafted.md` is added, which is the whole of the difference between the two
/// documents.
fn manifest(prior: &str) -> String {
    format!(
        "headwater change 1\nadded\tpromotion/drafted.md\nprior\tpromotion/promoted.md\t{}\n",
        fixtures_dir().join("before").join(prior).display()
    )
}

/// The corpus paths the fixture tree holds, which is what a manifest is bound
/// against. Written out rather than walked, because a test that took the census
/// from the same walk the run takes would bind every path a manifest could name.
const HELD: [&str; 3] = [
    "promotion/drafted.md",
    "promotion/promoted.md",
    "promotion/untouched.md",
];

fn bind(unbound: Unbound) -> Change {
    unbound.bind(|path| HELD.contains(&path))
}

fn change(prior: &str) -> Change {
    bind(read(&manifest(prior)))
}

fn read(manifest: &str) -> Unbound {
    Unbound::read(manifest, |path| std::fs::read(path)).expect("the manifest reads")
}

fn run_with(ctx: &Context, cache: &mut Cache) -> Run {
    let corpus = Corpus::new(fixtures_dir(), "promotion");
    let source = std::fs::read_to_string(fixtures_dir().join("promotion.taxonomy.yml"))
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
    let graph = Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(&corpus),
        &corpus,
        &config,
    );
    headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: "sha256:promotion-fixture",
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            adoption: None,
            source: "engine/crates/check/fixtures/promotion.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        ctx,
        cache,
    )
}

fn at(prior: Option<&str>) -> Context {
    let now = Date::parse(PINNED).expect("the pinned date");
    match prior {
        None => Context::at(now),
        Some(prior) => Context::over(now, change(prior)),
    }
}

fn promotions(run: &Run) -> Vec<&str> {
    run.findings
        .iter()
        .filter(|finding| finding.rule == headwater_check::promotion::RULE)
        .map(|finding| finding.path.as_str())
        .collect()
}

/// One document was promoted and one was drafted straight to `accepted`, and
/// the run counts one.
///
/// The decisive fixture. Both documents stand at `accepted` and both are
/// carried by the change, so a rule that read the current warrant alone reports
/// two, and a rule that read "is in the change and is accepted" reports two as
/// well. Only the prior version separates them.
///
/// `untouched.md` is the third arm: it stands at `asserted` and the change does
/// not name it, so a rule that counted documents the change did not carry would
/// have to say something about it, and this one says nothing.
#[test]
fn one_promotion_is_counted_and_a_document_drafted_straight_to_accepted_is_not() {
    let run = run_with(&at(Some("promoted.md")), &mut Cache::disabled());
    assert_eq!(
        promotions(&run),
        vec!["promotion/promoted.md"],
        "the count is not the transition"
    );
    let scoped = run.change.expect("a change-scoped run states its change");
    assert_eq!(scoped.promotions, 1);
    assert_eq!(scoped.named.documents, 2);
    assert_eq!(scoped.named.added, 1);
    assert_eq!(scoped.named.carried, 1);
    // And the report says it in one line, so a reader does not count findings.
    assert!(
        scoped
            .render()
            .contains("    1 promoted from `asserted` to `accepted`"),
        "{}",
        scoped.render()
    );
}

/// A prior version whose warrant already stood at `accepted` is no promotion.
///
/// The other direction of the same reading, over the same current bytes. A rule
/// that fired on "the current warrant is `accepted` and the change carries this
/// document" passes the test above and fails this one.
#[test]
fn a_document_that_was_already_accepted_is_not_promoted() {
    let run = run_with(&at(Some("accepted-already.md")), &mut Cache::disabled());
    assert_eq!(promotions(&run), Vec::<&str>::new());
    assert_eq!(run.change.expect("a change").promotions, 0);
}

/// A run with no change reports every instance as skipped, and never as passed.
///
/// [Spec 12](../../../../docs/spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version):
/// "In a full-corpus run, instances of such a check are counted and reported as
/// skipped, with the reason `change-scoped-only` — visible, never silent." A
/// rule that passed instead would contribute nothing in the mode this
/// repository actually runs, and no report would say so.
#[test]
fn a_full_corpus_run_reports_every_instance_as_skipped() {
    let run = run_with(&at(None), &mut Cache::disabled());
    assert!(run.change.is_none(), "a run with no change states one");
    let instances: Vec<&Outcome> = run
        .instances
        .iter()
        .filter(|instance| instance.rule == headwater_check::promotion::RULE)
        .map(|instance| &instance.outcome)
        .collect();
    assert_eq!(
        instances.len(),
        3,
        "one instance per document, or none at all"
    );
    for outcome in &instances {
        match outcome {
            Outcome::Skipped(why) => assert!(
                why.starts_with(headwater_check::scope::CHANGE_SCOPED_ONLY),
                "skipped for another reason: {why}"
            ),
            other => panic!("a full-corpus run reached a verdict: {other:?}"),
        }
    }
    // And coverage prints them, which is where a reader meets the skip.
    assert!(
        run.coverage
            .render()
            .contains(headwater_check::scope::CHANGE_SCOPED_ONLY),
        "the skip is not reported"
    );
}

/// A prior version this engine could not read is skipped rather than passed.
///
/// The third state of the input, and the one where every wrong answer is green.
/// A reader that treated an unreadable prior version as an absent one would
/// report the document as unpromoted, which is a verdict about a version nobody
/// saw.
#[test]
fn a_prior_version_that_did_not_read_is_skipped_rather_than_passed() {
    let change = bind(read(
        "headwater change 1\nprior\tpromotion/promoted.md\t/nonexistent/before.md\n",
    ));
    let ctx = Context::over(Date::parse(PINNED).expect("a date"), change);
    let run = run_with(&ctx, &mut Cache::disabled());
    assert_eq!(promotions(&run), Vec::<&str>::new());

    let skipped: Vec<&String> = run
        .instances
        .iter()
        .filter(|instance| instance.rule == headwater_check::promotion::RULE)
        .filter_map(|instance| match &instance.outcome {
            Outcome::Skipped(why) => Some(why),
            _ => None,
        })
        .collect();
    assert_eq!(skipped.len(), 1, "the unreadable version reached a verdict");
    assert!(
        skipped[0].contains("/nonexistent/before.md"),
        "{}",
        skipped[0]
    );
    assert_eq!(run.change.expect("a change").named.unreadable, 1);
}

/// A cached verdict does not survive a change to the prior version.
///
/// The current bytes are the same on both runs, the lock is the same, the clock
/// is the same, and the read set is equal document for document. The only input
/// that moved is the version the change carried, so a key that omitted it would
/// serve the first verdict to the second run. That is the defect this test
/// exists for, and it is invisible to the `--no-cache` differential, because
/// both sides of that comparison hold one change.
#[test]
fn a_warm_run_does_not_serve_a_verdict_across_a_change_to_the_prior_version() {
    let directory = std::env::temp_dir().join(format!(
        "headwater-promotion-{}-{}",
        std::process::id(),
        line!()
    ));
    std::fs::create_dir_all(&directory).expect("a scratch directory");

    let mut cold = Cache::at(&directory, "sha256:promotion-fixture");
    let first = run_with(&at(Some("promoted.md")), &mut cold);
    cold.write(&directory);
    assert_eq!(first.change.expect("a change").promotions, 1);

    let mut warm = Cache::at(&directory, "sha256:promotion-fixture");
    let second = run_with(&at(Some("accepted-already.md")), &mut warm);
    warm.write(&directory);
    assert_eq!(
        second.change.expect("a change").promotions,
        0,
        "a warm run served a verdict about another prior version"
    );

    // And the run that read one prior version twice is warm, so the key divides
    // rather than refuses. A rule that lost its key would pass the assertion
    // above by re-evaluating everything, forever.
    let mut again = Cache::at(&directory, "sha256:promotion-fixture");
    let third = run_with(&at(Some("accepted-already.md")), &mut again);
    assert_eq!(third.change.expect("a change").promotions, 0);
    assert!(
        again.report().hits > 0,
        "no instance was served from cache: {:?}",
        again.report()
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// A path the corpus holds no row at is reported, and never absorbed.
///
/// The manifest below is the honest one with a single character appended to a
/// path. Every other input is equal: the same prior bytes, the same tree, the
/// same clock. Before this test the run counted 0 promotions, reported `1 with
/// a prior version this run read`, and exited 0 with no other line about it. A
/// caller was told the injection worked over a manifest that reached nothing.
///
/// The second arm is the same defect in the form a person actually writes. A
/// leading `./` is a path no census row holds, and this engine normalizes
/// nothing: one accepted spelling invites the next, and the duplicate guard
/// would then have two spellings of one path to reconcile.
#[test]
fn a_path_that_binds_to_no_row_is_reported_rather_than_absorbed() {
    let prior = fixtures_dir().join("before").join("promoted.md");
    let at = |path: &str| format!("headwater change 1\nprior\t{path}\t{}\n", prior.display());

    // The honest manifest, which is the control arm.
    let honest = run_with(
        &Context::over(Date::parse(PINNED).expect("a date"), change("promoted.md")),
        &mut Cache::disabled(),
    );
    assert_eq!(honest.change.expect("a change").promotions, 1);

    for path in [
        "promotion/promoted.mdx",
        "./promotion/promoted.md",
        "docs/promotion/promoted.md",
    ] {
        let ctx = Context::over(Date::parse(PINNED).expect("a date"), bind(read(&at(path))));
        let run = run_with(&ctx, &mut Cache::disabled());
        let scoped = run
            .change
            .clone()
            .expect("a change-scoped run states its change");

        assert_eq!(
            promotions(&run),
            Vec::<&str>::new(),
            "`{path}` bound to a document"
        );
        assert_eq!(scoped.named.unmatched, 1, "`{path}` was absorbed");
        assert_eq!(
            scoped.named.carried, 0,
            "`{path}` opened a prior version and bound to nothing, and the report called that a \
             document this run read"
        );
        assert_eq!(scoped.unmatched, vec![path.to_string()]);
        let report = scoped.render();
        assert!(
            report.contains("named no row of this corpus") && report.contains(path),
            "the report does not name the path that reached nothing:\n{report}"
        );
    }
}
