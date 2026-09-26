// SPDX-License-Identifier: Apache-2.0
//! A generated page that two relations tell two different states, and the
//! ways a rule about it is easy to get wrong.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship)
//! sets the floor: "every check ships with at least one fixture that it fails
//! and one that it passes." The tree under `fixtures/state-set-twice/` is
//! [#1086](https://github.com/headwater-ai/headwater/issues/1086), and every
//! case in it is a reading `derived::incoming` in the generate crate takes.
//!
//! **A rule that centered on typed documents** never reaches
//! `pages/gen-clash.md`, because the census classifies a generated file apart,
//! and it reports `notes/authored-clash.md`, whose state generate never writes.
//!
//! **A rule that compared relation names** reports `pages/gen-agree.md`, where
//! `supersedes` and `replaces` both write `superseded`.
//!
//! **A rule that read the writing file as the source** reports
//! `pages/gen-inverse.md`, which another file named through an inverse half.
//!
//! **A rule that read every edge into the page** reports
//! `pages/gen-self-written.md`, which names itself under `retires`. Generate
//! reads no edge the page wrote, so only `superseded` reaches it.
//!
//! **A rule that ignored the kind** reports `plain/gen-unstated.md`, whose kind
//! requires no state, so generate writes none.
//!
//! **A gate that counted setters** runs an instance under
//! `state-set-twice-agree.taxonomy.yml`, where two setters name one state.
//!
//! No test here runs generate, because this crate cannot depend on it. The
//! reading each case pins is the one `derived::incoming` states clause by
//! clause.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Outcome, Register, Run, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

/// As every other recorded run: a verdict is a function of the injected clock.
const PINNED: &str = "2026-08-12";

const RULE: &str = headwater_check::state_set_twice::RULE;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn pinned() -> Context {
    Context::at(Date::parse(PINNED).expect("the pinned date"))
}

fn run_with(taxonomy_file: &str) -> Run {
    run_in(taxonomy_file, &pinned())
}

fn run_in(taxonomy_file: &str, ctx: &Context) -> Run {
    run_cached(taxonomy_file, ctx, &mut Cache::disabled())
}

fn run_cached(taxonomy_file: &str, ctx: &Context, cache: &mut Cache) -> Run {
    let corpus = Corpus::new(fixtures_dir(), "state-set-twice");
    let source =
        std::fs::read_to_string(fixtures_dir().join(taxonomy_file)).expect("the fixture taxonomy");
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
            lock: "sha256:state-set-twice-fixture",
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            observations: &headwater_check::Observations::empty(),
            adoption: None,
            source: "engine/crates/check/fixtures/state-set-twice.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        ctx,
        cache,
    )
}

fn run() -> Run {
    run_with("state-set-twice.taxonomy.yml")
}

/// The findings this rule reported, as `(path, message)`.
fn reported(run: &Run) -> Vec<(&str, &str)> {
    run.findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .map(|finding| (finding.path.as_str(), finding.message.as_str()))
        .collect()
}

fn at<'a>(run: &'a Run, file: &str) -> Vec<&'a str> {
    let path = format!("state-set-twice/{file}");
    reported(run)
        .into_iter()
        .filter(|(at, _)| *at == path)
        .map(|(_, message)| message)
        .collect()
}

fn instances(run: &Run) -> usize {
    run.instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .count()
}

/// The decisive case: one finding at the generated page, naming both
/// relations, both states, the document that wrote each edge, and the state
/// generate writes.
#[test]
fn a_generated_page_told_two_states_by_two_relations_is_reported_once() {
    let run = run();
    let messages = at(&run, "pages/gen-clash.md");
    assert_eq!(messages.len(), 1, "one finding at the page: {messages:?}");
    let message = messages[0];
    for needle in [
        "`supersedes`",
        "`retires`",
        "`superseded`",
        "`retired`",
        "NOTE-FIX-clash-superseder",
        "NOTE-FIX-clash-retirer",
        "writes `retired` onto this page",
    ] {
        assert!(message.contains(needle), "{needle} in {message}");
    }
    // At the state facet the last run of generate wrote, which is line 4.
    let finding = run
        .findings
        .iter()
        .find(|finding| finding.rule == RULE)
        .expect("the finding");
    assert_eq!((finding.line, finding.column), (4, 1), "{finding:?}");
}

/// A marked file that no output of the projection plan claims is one
/// `headwater generate` lists as orphaned and never writes. Its state was
/// written by a person, so the rule has no write to warn about (#1137).
#[test]
fn a_marked_page_that_no_projection_writes_is_not_read() {
    let orphaned = ["state-set-twice/pages/gen-clash.md".to_string()].into();
    let run = run_in(
        "state-set-twice.taxonomy.yml",
        &pinned().with_orphaned(orphaned),
    );
    assert!(
        at(&run, "pages/gen-clash.md").is_empty(),
        "{:?}",
        reported(&run)
    );
    let instance = run
        .instances
        .iter()
        .find(|instance| instance.rule == RULE)
        .expect("the one corpus instance still runs");
    assert!(
        matches!(instance.outcome, Outcome::Passed),
        "{:?}",
        instance.outcome
    );
}

/// The guard for the case above: an orphan elsewhere does not silence the
/// rule over a page the plan does write.
#[test]
fn an_orphan_elsewhere_leaves_a_written_page_reported() {
    let orphaned = ["state-set-twice/pages/gen-agree.md".to_string()].into();
    let run = run_in(
        "state-set-twice.taxonomy.yml",
        &pinned().with_orphaned(orphaned),
    );
    assert_eq!(
        at(&run, "pages/gen-clash.md").len(),
        1,
        "{:?}",
        reported(&run)
    );
}

/// The orphan set is in the cache key. Without it, a cached "fired" verdict
/// survives the change that turns the page into an orphan, because the read
/// set does not move: the same files are read, and generate's answer about
/// them is what changed.
#[test]
fn a_cached_verdict_does_not_survive_the_page_turning_orphan() {
    let dir = std::env::temp_dir().join(format!(
        "headwater-state-set-twice-cache-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the cache directory is made");
    let taxonomy = "state-set-twice.taxonomy.yml";
    let orphan: std::collections::BTreeSet<String> =
        ["state-set-twice/pages/gen-clash.md".to_string()].into();
    let elsewhere: std::collections::BTreeSet<String> =
        ["state-set-twice/pages/gen-agree.md".to_string()].into();
    let fired = |ctx: &Context| {
        let mut cache = Cache::at(&dir, "sha256:state-set-twice-fixture", "rules");
        let run = run_cached(taxonomy, ctx, &mut cache);
        cache.write(&dir).expect("the cache writes");
        (at(&run, "pages/gen-clash.md").len(), cache.report().misses)
    };

    let (first, _) = fired(&pinned());
    let (unchanged, misses) = fired(&pinned());
    let (orphaned, _) = fired(&pinned().with_orphaned(orphan.clone()));
    let (again, _) = fired(&pinned().with_orphaned(orphan));
    let (other, _) = fired(&pinned().with_orphaned(elsewhere));
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(first, 1, "the page fires with no orphan set");
    assert_eq!(unchanged, 1, "a warm run serves the same verdict");
    assert_eq!(misses, 0, "and serves every instance from the cache");
    assert_eq!(
        orphaned, 0,
        "a cached fired verdict survived the orphan set"
    );
    assert_eq!(again, 0, "the orphaned verdict is served warm");
    assert_eq!(other, 1, "an orphan elsewhere keys apart and fires");
}

/// Two relations that write the same state agree, and there is nothing to
/// report.
#[test]
fn two_relations_that_write_one_state_are_silent() {
    let run = run();
    assert!(
        at(&run, "pages/gen-agree.md").is_empty(),
        "{:?}",
        reported(&run)
    );
}

/// A page named by another file's inverse half is the source of that relation,
/// and generate reads no state from it.
#[test]
fn an_inverse_half_written_elsewhere_sets_no_state_on_the_page() {
    let run = run();
    assert!(
        at(&run, "pages/gen-inverse.md").is_empty(),
        "{:?}",
        reported(&run)
    );
}

/// An edge the page wrote at itself is one generate never reads, because the
/// page would then be a function of its own last version.
#[test]
fn edges_the_page_wrote_itself_are_not_read() {
    let run = run();
    assert!(
        at(&run, "pages/gen-self-written.md").is_empty(),
        "{:?}",
        reported(&run)
    );
}

/// Generate writes a state only where the kind requires the state facet, so a
/// generated file of a kind that requires none has nothing to choose.
#[test]
fn a_generated_kind_that_requires_no_state_is_silent() {
    let run = run();
    assert!(
        at(&run, "plain/gen-unstated.md").is_empty(),
        "{:?}",
        reported(&run)
    );
}

/// An authored document's state is what its author wrote, and generate
/// writes nothing onto it.
#[test]
fn an_authored_document_is_not_read() {
    let run = run();
    assert!(
        at(&run, "notes/authored-clash.md").is_empty(),
        "{:?}",
        reported(&run)
    );
}

/// Exactly the one clash, and one corpus instance that decided.
#[test]
fn the_tree_reports_exactly_the_one_clash() {
    let run = run();
    let paths: Vec<&str> = reported(&run).into_iter().map(|(path, _)| path).collect();
    assert_eq!(paths, ["state-set-twice/pages/gen-clash.md"]);
    assert_eq!(instances(&run), 1);
    assert!(
        run.instances
            .iter()
            .filter(|instance| instance.rule == RULE)
            .all(|instance| !matches!(instance.outcome, Outcome::Skipped(_))),
        "a skipped instance"
    );
}

/// Where one relation is the only one that writes a state, no document can be
/// told two, and the rule runs no instance.
#[test]
fn a_taxonomy_with_one_setting_relation_runs_no_instance() {
    let single = run_with("state-set-twice-single.taxonomy.yml");
    assert_eq!(instances(&single), 0);
}

/// Two setting relations that name one state cannot clash either. A gate that
/// counted setters instead of comparing their states runs an instance here.
#[test]
fn two_setting_relations_that_agree_run_no_instance() {
    let agree = run_with("state-set-twice-agree.taxonomy.yml");
    assert_eq!(instances(&agree), 0);
}
