// SPDX-License-Identifier: Apache-2.0
//! A change that adds a file inside the governed scope and no `governance`
//! edge reaches it (#1573).
//!
//! The ratchet's baseline is the change itself, so nothing is stored and no
//! figure is compared across branches (HW-DR-0049). A file the change adds is
//! new debt. A file the change edits and that already stood ungoverned is old
//! debt, and the rule leaves it alone.
//!
//! The tree under `fixtures/scope-ratchet/` is the corpus, and it holds one
//! document that governs `a.rs` and `new_governed.rs`. The files under
//! `fixtures/scope-ratchet-src/` are the scope, which no census row holds,
//! exactly as a source file of this repository is held by none. The version of
//! `old_ungoverned.rs` that stood before the change is under
//! `fixtures/scope-ratchet-before/`.
//!
//! The decisive line is the edited file. Binding collapses an `added` line and
//! a `prior` line for a path with no census row into one shape, and a rule
//! that read that shape would refuse every edit to a file that already stood
//! ungoverned. That is a gate on existing debt and not a ratchet.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::change::{Change, Unbound};
use headwater_check::{Cache, Context, Date, Declared, Outcome, Register, Run, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

const PINNED: &str = "2026-08-12";

const RULE: &str = headwater_check::scope_ratchet::RULE;

const LOCK: &str = "sha256:scope-ratchet-fixture";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// The one document of the corpus. A manifest is bound against this set, as
/// the CLI binds it against the census, which holds no row for a source file.
const HELD: [&str; 1] = ["scope-ratchet/governing.md"];

const NEW_UNGOVERNED: &str = "scope-ratchet-src/new_ungoverned.rs";
const NEW_GOVERNED: &str = "scope-ratchet-src/new_governed.rs";
const OLD_UNGOVERNED: &str = "scope-ratchet-src/old_ungoverned.rs";

/// The `prior` line for the edited file that already stood ungoverned.
fn edited() -> String {
    format!(
        "prior\t{OLD_UNGOVERNED}\t{}\n",
        fixtures_dir()
            .join("scope-ratchet-before")
            .join("old_ungoverned.rs")
            .display()
    )
}

/// The three lines of the decisive case.
fn three_lines() -> String {
    format!(
        "added\t{NEW_UNGOVERNED}\nadded\t{NEW_GOVERNED}\n{}",
        edited()
    )
}

fn unbound(lines: &str) -> Unbound {
    let manifest = format!("headwater change 1\n{lines}");
    Unbound::read(&manifest, |path| std::fs::read(path)).expect("the manifest reads")
}

fn change_of(lines: &str) -> Change {
    unbound(lines).bind(|path| HELD.contains(&path))
}

fn run_with(ctx: &Context, cache: &mut Cache) -> Run {
    run_in("scope-ratchet", ctx, cache)
}

/// A run over one fixture corpus: `fixtures/<name>/` with the taxonomy
/// `fixtures/<name>.taxonomy.yml`.
fn run_in(name: &str, ctx: &Context, cache: &mut Cache) -> Run {
    let corpus = Corpus::new(fixtures_dir(), name);
    let source = std::fs::read_to_string(fixtures_dir().join(format!("{name}.taxonomy.yml")))
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
            lock: LOCK,
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            observations: &headwater_check::Observations::empty(),
            pin: None,
            lock_stale: None,
            agent_budgets: &[],
            scope_coverage: None,
            harvests: &[],
            imports: &[],
            adoption: None,
            source: "engine/crates/check/fixtures/scope-ratchet.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        ctx,
        cache,
    )
}

fn context(change: Change) -> Context {
    Context::over(Date::parse(PINNED).expect("the pinned date"), change)
}

fn over(lines: &str) -> Run {
    run_with(&context(change_of(lines)), &mut Cache::disabled())
}

/// The findings of this rule, as `(path, message)`.
fn findings(run: &Run) -> Vec<(&str, &str)> {
    run.findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .map(|finding| (finding.path.as_str(), finding.message.as_str()))
        .collect()
}

fn paths(run: &Run) -> Vec<&str> {
    findings(run).into_iter().map(|(path, _)| path).collect()
}

/// The one instance of this rule, whatever the run.
fn instance(run: &Run) -> &headwater_check::Instance {
    let mut found = run.instances.iter().filter(|one| one.rule == RULE);
    let one = found.next().expect("the rule generates an instance");
    assert!(
        found.next().is_none(),
        "a corpus-grained rule has one instance"
    );
    one
}

/// The decisive case: an ungoverned add is reported, a governed add is not,
/// and an edit to a file that already stood ungoverned is not.
#[test]
fn an_added_ungoverned_file_in_scope_is_reported_and_an_edited_one_is_not() {
    let run = over(&three_lines());
    let found = findings(&run);
    assert_eq!(
        paths(&run),
        [NEW_UNGOVERNED],
        "the one added file no edge reaches, and not the governed add or the edit: {found:?}"
    );
    let (_, message) = found[0];
    assert!(
        message.contains("scope-ratchet-src/**"),
        "the finding names the pattern that admits the path: {message}"
    );
    let finding = run
        .findings
        .iter()
        .find(|finding| finding.rule == RULE)
        .expect("the finding");
    assert_eq!(
        finding.severity,
        headwater_check::Severity::Warn,
        "the remedy is a rewrite, so the rule warns and posture decides whether it blocks"
    );
}

/// A corpus whose census holds a row at every path binds the added file to
/// that row, so the change carries it as an addition the census holds. The
/// rule reads an addition either way.
#[test]
fn an_added_file_that_a_census_row_holds_is_read_the_same_way() {
    let change = unbound(&three_lines()).bind(|_| true);
    let run = run_with(&context(change), &mut Cache::disabled());
    assert_eq!(paths(&run), [NEW_UNGOVERNED]);
}

/// An added path outside every scope pattern is no part of the ratchet.
#[test]
fn an_added_file_outside_the_scope_is_not_reported() {
    let run = over("added\tscope-ratchet-elsewhere/new.rs\n");
    assert!(paths(&run).is_empty(), "{:?}", findings(&run));
    assert!(matches!(instance(&run).outcome, Outcome::Passed));
}

/// A run with no change has no additions to read, so the instance is skipped
/// with the reason spec 12 fixes rather than passed.
#[test]
fn a_run_without_a_change_skips_the_rule() {
    let run = run_with(
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    );
    match &instance(&run).outcome {
        Outcome::Skipped(reason) => assert!(
            reason.starts_with("change-scoped-only"),
            "the reason spec 12 fixes: {reason}"
        ),
        other => panic!("a run with no change skips the rule, and it was {other:?}"),
    }
}

/// A scratch root for the cache, one per case, keyed on the case's name as
/// well as the pid, because cargo runs the cases of a target as threads of one
/// process.
fn scratch(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "headwater-scope-ratchet-{label}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the scratch root is made");
    root
}

/// A cached pass does not survive a later change that adds an ungoverned
/// file. The added path is not a census row, so the read set holds it only
/// because the rule put it there, and without it the second run would key
/// exactly as the first and be served its pass.
#[test]
fn a_cached_pass_does_not_survive_an_ungoverned_add() {
    let root = scratch("cache");
    let mut cold = Cache::at(&root, LOCK, "rules");
    let first = run_with(&context(change_of(&edited())), &mut cold);
    assert!(paths(&first).is_empty(), "the edit alone is clean");
    cold.write(&root).expect("the cache writes");

    // The same change again is served from the cache, so the case below
    // reads a cache that serves this rule and is not vacuous.
    let mut warm = Cache::at(&root, LOCK, "rules");
    let again = run_with(&context(change_of(&edited())), &mut warm);
    assert!(paths(&again).is_empty());
    assert!(
        warm.report().hits > 0,
        "the second run of one change is served from the cache: {:?}",
        warm.report()
    );

    let mut warm = Cache::at(&root, LOCK, "rules");
    let added = format!("added\t{NEW_UNGOVERNED}\n{}", edited());
    let second = run_with(&context(change_of(&added)), &mut warm);
    assert_eq!(
        paths(&second),
        [NEW_UNGOVERNED],
        "the add moved the key, so the rule ran again and found it"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// An added path that a row carrying a document already lists joins the read
/// set once, and an added source file joins it with a digest. A path listed
/// twice would be one input with two digests, and a source file listed with
/// none would leave the instance unkeyed.
#[test]
fn an_added_path_joins_the_read_set_once_and_with_a_digest() {
    let lines = format!("added\t{}\nadded\t{NEW_UNGOVERNED}\n", HELD[0]);
    let change = unbound(&lines).bind(|_| true);
    let run = run_with(&context(change), &mut Cache::disabled());
    let reads = &instance(&run).reads;
    for path in [HELD[0], NEW_UNGOVERNED] {
        let listed: Vec<_> = reads.iter().filter(|input| input.path == path).collect();
        assert_eq!(listed.len(), 1, "{path} is listed once: {reads:?}");
        assert!(listed[0].digest.is_some(), "{path} carries a digest");
    }
}

// The removal half (#1573, clause 4's first disjunct).
//
// A change that drops or narrows a `governs` entry, or deletes the document
// that held one, can leave a file that stands in the scope reached by no
// governance edge. The rule reads the prior version of each document the
// change carries, takes every governance target that version wrote onto a
// scoped anchor kind, and reports each in-scope file such a target matched
// that no edge reaches now.
//
// The corpus is `fixtures/scope-ratchet-removal/`, and its taxonomy and
// tree are described in `fixtures/scope-ratchet-removal.taxonomy.yml`.

const REMOVAL: &str = "scope-ratchet-removal";

/// The two documents of the removal corpus. `departing.md` is not here: the
/// change deletes it, so no census row holds it.
const REMOVAL_HELD: [&str; 2] = [
    "scope-ratchet-removal/narrowing.md",
    "scope-ratchet-removal/other.md",
];

const NARROWING: &str = "scope-ratchet-removal/narrowing.md";
const DEPARTING: &str = "scope-ratchet-removal/departing.md";
const DROPPED: &str = "scope-ratchet-removal-src/dropped.rs";
const NARROWED2: &str = "scope-ratchet-removal-src/narrowed2.rs";
const DEPARTED: &str = "scope-ratchet-removal-src/departed.rs";
const BEFORE: &str = "scope-ratchet-removal-before";

/// A `prior` line for a document of the removal corpus, read from `dir`.
fn prior_line(path: &str, dir: &str) -> String {
    let file = Path::new(path).file_name().expect("a file name");
    format!(
        "prior\t{path}\t{}\n",
        fixtures_dir().join(dir).join(file).display()
    )
}

/// The decisive manifest: `narrowing.md` narrowed, `departing.md` deleted.
fn removal_lines() -> String {
    format!(
        "{}{}",
        prior_line(NARROWING, BEFORE),
        prior_line(DEPARTING, BEFORE)
    )
}

fn removal_run(lines: &str, cache: &mut Cache) -> Run {
    let change = unbound(lines).bind(|path| REMOVAL_HELD.contains(&path));
    run_in(REMOVAL, &context(change), cache)
}

fn sorted_paths(run: &Run) -> Vec<&str> {
    let mut got = paths(run);
    got.sort_unstable();
    got
}

/// The decisive case. Three files lose their last governance edge, and each
/// finding names the document whose prior entry reached the file. No finding
/// on `kept.rs` (still governed), `shared.rs` (another document governs it),
/// `gone.rs` (the change deletes it, so it is in no scope now), `stray.rs`
/// (only a `cites` reached it) or `old_debt.rs` (it stood ungoverned and no
/// prior entry reached it).
#[test]
fn a_change_that_removes_the_last_governance_edge_onto_a_scoped_file_is_reported() {
    let run = removal_run(&removal_lines(), &mut Cache::disabled());
    let found = findings(&run);
    assert_eq!(
        sorted_paths(&run),
        [DEPARTED, DROPPED, NARROWED2],
        "the three files the change leaves ungoverned: {found:?}"
    );
    for (path, message) in &found {
        let document = if *path == DEPARTED {
            DEPARTING
        } else {
            NARROWING
        };
        assert!(
            message.contains(document),
            "the finding on {path} names {document}: {message}"
        );
        assert!(
            message.contains("scope-ratchet-removal-src/**"),
            "the finding names the admitting pattern: {message}"
        );
    }
    for finding in run.findings.iter().filter(|finding| finding.rule == RULE) {
        assert_eq!(finding.severity, headwater_check::Severity::Warn);
        assert!(finding.patch.is_none(), "the remedy is a rewrite");
    }
}

/// The control. The same tree, and a manifest whose prior for `narrowing.md`
/// is its current version, gives no finding. So the rule reads the prior
/// version and not the tree.
#[test]
fn a_change_whose_prior_governs_what_the_tree_governs_is_not_reported() {
    let run = removal_run(&prior_line(NARROWING, REMOVAL), &mut Cache::disabled());
    assert!(paths(&run).is_empty(), "{:?}", findings(&run));
    assert!(matches!(instance(&run).outcome, Outcome::Passed));
}

/// Each half of the decisive manifest alone gives its own findings: the
/// committed prior of a document the census still holds, and the prior of a
/// document the change deleted.
#[test]
fn a_narrowed_document_and_a_deleted_one_are_each_read() {
    let narrowed = removal_run(&prior_line(NARROWING, BEFORE), &mut Cache::disabled());
    assert_eq!(
        sorted_paths(&narrowed),
        [DROPPED, NARROWED2],
        "{:?}",
        findings(&narrowed)
    );

    let deleted = removal_run(&prior_line(DEPARTING, BEFORE), &mut Cache::disabled());
    assert_eq!(paths(&deleted), [DEPARTED], "{:?}", findings(&deleted));
}

/// A file the change adds and that a prior entry reached is the added half's
/// finding, and it is reported once, with the added half's message.
#[test]
fn a_path_the_change_adds_is_reported_once_by_the_added_half() {
    let lines = format!("added\t{DROPPED}\n{}", removal_lines());
    let run = removal_run(&lines, &mut Cache::disabled());
    let found = findings(&run);
    let on_dropped: Vec<_> = found.iter().filter(|(path, _)| *path == DROPPED).collect();
    assert_eq!(on_dropped.len(), 1, "one finding on {DROPPED}: {found:?}");
    assert!(
        on_dropped[0].1.contains("This change adds"),
        "the added half reports it: {}",
        on_dropped[0].1
    );
}

/// A cached pass does not survive a change that names a prior that governed
/// more. The tree is the same in both runs, and so is every document's
/// current digest, so only the prior digest in the key tells them apart.
/// HW-OBL-0117 is the edge-grain record of the same defect.
#[test]
fn a_cached_pass_does_not_survive_a_prior_that_governed_more() {
    let root = scratch("removal-cache");
    let control = prior_line(NARROWING, REMOVAL);
    let mut cold = Cache::at(&root, LOCK, "rules");
    let first = removal_run(&control, &mut cold);
    assert!(paths(&first).is_empty(), "the control is clean");
    cold.write(&root).expect("the cache writes");

    let mut warm = Cache::at(&root, LOCK, "rules");
    let again = removal_run(&control, &mut warm);
    assert!(paths(&again).is_empty());
    assert!(
        warm.report().hits > 0,
        "the second run of one change is served from the cache: {:?}",
        warm.report()
    );

    let mut warm = Cache::at(&root, LOCK, "rules");
    let second = removal_run(&prior_line(NARROWING, BEFORE), &mut warm);
    assert_eq!(
        sorted_paths(&second),
        [DROPPED, NARROWED2],
        "the prior moved the key, so the rule ran again and found them"
    );
    let _ = std::fs::remove_dir_all(&root);
}
