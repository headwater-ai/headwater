// SPDX-License-Identifier: Apache-2.0
//! A language regime reaches a path outside the corpus root that it lists, and
//! no other rule does (HW-DR-0084, clauses 2 to 6).
//!
//! The tree `fixtures/outside-root/` is its own repository root. Its corpus root
//! is `docs/`, and `README.md` sits beside that root with one contraction in it.
//!
//! **A runner that did not read the list** reports nothing at `README.md`, which
//! is the state before HW-DR-0084 had an implementation.
//!
//! **A runner that made the path a census row** moves the denominator, and hands
//! the file to every rule, so `facet.required.missing` reports the `summary`
//! facet that front-door prose never declares.
//!
//! **A runner that skipped an unmatched pattern in silence** prints no line that
//! names `MISSING.md`.

use headwater_census::census::{self, Census, Detail};
use headwater_census::outside;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Finding, Register, Run, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

const PINNED: &str = "2026-09-27";
const TREE: &str = "outside-root";
const README: &str = "README.md";

/// The three rules HW-DR-0084 clause 3 names, as literals.
const LANGUAGE_RULES: [&str; 3] = [
    "language.controlled.not_met",
    "language.retired_term.used",
    "language.source_form.not_met",
];

fn base() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(TREE)
}

/// The census and the run, with the regime's `outside_root` replaced by
/// `listed` when it is given and read from the fixture taxonomy when not.
fn checked(listed: Option<&[&str]>) -> (Census, Run) {
    checked_at(&base(), listed)
}

/// [`checked`] over a tree at `at`, which a case may have built in scratch.
fn checked_at(at: &Path, listed: Option<&[&str]>) -> (Census, Run) {
    let corpus = Corpus::new(at.to_path_buf(), "docs");
    let path = base().with_file_name(format!("{TREE}.taxonomy.yml"));
    let source = std::fs::read_to_string(&path).expect("the fixture taxonomy");
    let root = headwater_yaml::load(&source)
        .expect("the fixture taxonomy loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();

    let taxonomy = Taxonomy::read(&root).expect("the taxonomy reads");
    let declarations = Declarations::read(&root).expect("the declarations read");
    let register = Register::read(&root).expect("the register reads");
    let mut shape = Shape::read(&root).expect("the shape reads");
    if let Some(listed) = listed {
        shape.language[0].outside_root = listed.iter().map(|p| p.to_string()).collect();
    }
    let mut taken = census::take(&corpus, &taxonomy);
    taken.outside = outside::take(&corpus, &shape.outside_root());
    let config = Config::default();
    let graph = Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(&corpus),
        &corpus,
        &config,
    );
    let lock = format!("sha256:{TREE}-fixture");
    let source = format!("engine/crates/check/fixtures/{TREE}.taxonomy.yml");
    let run = headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: &lock,
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            observations: &headwater_check::Observations::empty(),
            adoption: None,
            source: &source,
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    );
    (taken, run)
}

fn findings_at<'a>(run: &'a Run, path: &str) -> Vec<&'a Finding> {
    run.findings
        .iter()
        .filter(|finding| finding.path == path)
        .collect()
}

fn rules_reading(run: &Run, path: &str) -> Vec<&'static str> {
    let mut rules: Vec<&'static str> = run
        .instances
        .iter()
        .filter(|instance| instance.reads.iter().any(|input| input.path == path))
        .map(|instance| instance.rule)
        .collect();
    rules.sort_unstable();
    rules
}

#[test]
fn a_listed_path_outside_the_root_is_read_by_the_three_language_rules_and_no_other() {
    let (taken, run) = checked(None);

    let findings = findings_at(&run, README);
    assert_eq!(
        findings.len(),
        1,
        "one finding at {README}, and nothing from a rule HW-DR-0084 leaves out: {findings:#?}"
    );
    assert_eq!(findings[0].rule, "language.controlled.not_met");
    assert!(
        findings[0].message.contains("doesn't"),
        "the finding names the contraction: {}",
        findings[0].message
    );
    assert_eq!(findings[0].line, 3, "the sentence is on line 3");

    assert_eq!(
        rules_reading(&run, README),
        LANGUAGE_RULES.to_vec(),
        "exactly the three language rules read {README}"
    );

    let (unlisted, _) = checked(Some(&[]));
    assert_eq!(
        taken.rows.len(),
        unlisted.rows.len(),
        "a path outside the root is not a census row"
    );
    assert!(
        run.coverage.unaccounted.is_empty(),
        "an outside path is not a read outside the denominator: {:?}",
        run.coverage.unaccounted
    );
    let report = taken.render(Detail::Exceptions);
    assert!(
        report.contains("outside the corpus root: 1 path, house 1"),
        "the census counts the outside path on its own line:\n{report}"
    );
}

/// An allow directive on an outside path suppresses the finding it names, as
/// it does on a document under the root.
#[test]
fn an_allow_directive_on_an_outside_path_holds() {
    let (_, run) = checked(Some(&["README.md", "CONTRIBUTING.md"]));
    assert_eq!(
        findings_at(&run, "CONTRIBUTING.md"),
        Vec::<&Finding>::new(),
        "the directive holds the contraction"
    );
    assert_eq!(findings_at(&run, README).len(), 1, "and only that one");
    assert!(
        rules_reading(&run, "CONTRIBUTING.md").contains(&"language.controlled.not_met"),
        "the rule still read the file"
    );
}

#[test]
fn an_unlisted_path_outside_the_root_is_read_by_nothing() {
    let (taken, run) = checked(Some(&[]));
    assert_eq!(findings_at(&run, README), Vec::<&Finding>::new());
    assert_eq!(rules_reading(&run, README), Vec::<&str>::new());
    assert!(
        !taken
            .render(Detail::Exceptions)
            .contains("outside the corpus root"),
        "a corpus that lists nothing prints the report it printed before"
    );
}

#[test]
fn a_pattern_that_matches_nothing_is_reported_by_name() {
    let (taken, run) = checked(Some(&["MISSING.md"]));
    assert_eq!(rules_reading(&run, README), Vec::<&str>::new());
    assert_eq!(taken.outside.unmatched.len(), 1);
    let report = taken.render(Detail::Exceptions);
    assert!(
        report.contains("unmatched `MISSING.md` in `house`"),
        "the report names the pattern that matched nothing:\n{report}"
    );
}

#[test]
fn a_pattern_that_leaves_the_repository_is_refused_and_reads_nothing() {
    let (taken, _) = checked(Some(&["../outside-root/README.md"]));
    assert_eq!(taken.outside.rows.len(), 0);
    assert_eq!(taken.outside.refused.len(), 1);
    assert!(
        taken
            .render(Detail::Exceptions)
            .contains("refused `../outside-root/README.md` in `house`"),
        "the report names the refused pattern"
    );
}

/// A path two regimes list is refused, and the first regime's reading stands,
/// so the path still answers to one regime.
#[test]
fn a_path_two_regimes_list_is_refused() {
    let corpus = Corpus::new(base(), "docs");
    let listed = [
        outside::Listed {
            regime: "house".to_string(),
            patterns: vec![README.to_string()],
        },
        outside::Listed {
            regime: "strict".to_string(),
            patterns: vec![README.to_string()],
        },
    ];
    let taken = outside::take(&corpus, &listed);
    assert_eq!(taken.rows.len(), 1);
    assert_eq!(taken.rows[0].regime, "house");
    assert_eq!(taken.refused.len(), 1);
    assert_eq!(taken.refused[0].regime, "strict");
    assert!(taken.refused[0].reason.contains("listed by `house`"));
}

#[test]
fn a_pattern_that_matches_inside_the_root_is_refused_and_reads_nothing_twice() {
    let (taken, run) = checked(Some(&["docs/notes/plain.md"]));
    assert_eq!(taken.outside.rows.len(), 0);
    assert_eq!(taken.outside.refused.len(), 1);
    assert!(taken.outside.refused[0]
        .reason
        .contains("under the corpus root"));
    let rules = rules_reading(&run, "docs/notes/plain.md");
    let language = rules
        .iter()
        .filter(|rule| **rule == "language.controlled.not_met")
        .count();
    assert_eq!(language, 1, "the kind's own instance, and no second one");
}

/// The refusals by the rule that makes `check --strict` fail on them.
fn refusals(run: &Run) -> Vec<&Finding> {
    run.findings
        .iter()
        .filter(|finding| finding.rule == "language.outside_root.refused")
        .collect()
}

#[test]
fn a_refused_or_unmatched_pattern_is_an_error_that_fails_a_strict_run() {
    let (_, run) = checked(Some(&["MISSING.md", "docs/notes/plain.md"]));
    let found = refusals(&run);
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(found
        .iter()
        .all(|finding| finding.severity == headwater_check::Severity::Error));
    assert!(found[0].message.contains("`MISSING.md`"));
    assert!(found[1].message.contains("`docs/notes/plain.md`"));
    let (_, clean) = checked(None);
    assert_eq!(refusals(&clean), Vec::<&Finding>::new());
}

/// `./docs/…` names a path under the root as `docs/…` does, so it is refused
/// and reads nothing.
#[test]
fn a_dot_segment_does_not_carry_a_path_under_the_root_past_the_refusal() {
    for listed in [
        "./docs/notes/plain.md",
        "docs/./notes/plain.md",
        "./docs/notes/*.md",
    ] {
        let (taken, run) = checked(Some(&[listed]));
        assert_eq!(taken.outside.rows.len(), 0, "{listed}");
        assert_eq!(taken.outside.refused.len(), 1, "{listed}");
        assert_eq!(refusals(&run).len(), 1, "{listed}");
    }
}

/// A scratch copy of the tree beside a directory it does not hold, with a
/// symlink out of the tree and a symlink into the corpus root. Keyed on the
/// case name as well as the pid, because cargo runs cases as threads of one
/// process.
struct Scratch {
    top: PathBuf,
}

impl Scratch {
    fn new(case: &str) -> Scratch {
        let top = std::env::temp_dir().join(format!(
            "headwater-outside-root-{}-{case}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&top);
        copy(&base(), &top.join("repo"));
        std::fs::create_dir_all(top.join("elsewhere")).expect("the outside directory");
        std::fs::write(
            top.join("elsewhere/secret.md"),
            "# Not this repository\n\nThis file isn't in the repository.\n",
        )
        .expect("the outside file");
        let repo = top.join("repo");
        std::os::unix::fs::symlink("../elsewhere/secret.md", repo.join("link-out.md"))
            .expect("the link out");
        std::os::unix::fs::symlink("docs/notes/plain.md", repo.join("link-in.md"))
            .expect("the link in");
        std::os::unix::fs::symlink("../elsewhere", repo.join("linked-dir"))
            .expect("the linked directory");
        std::fs::create_dir_all(repo.join("front")).expect("a directory of front-door files");
        std::os::unix::fs::symlink("../../elsewhere/secret.md", repo.join("front/out.md"))
            .expect("a link a wildcard matches");
        Scratch { top }
    }

    fn repo(&self) -> PathBuf {
        self.top.join("repo")
    }

    fn secret(&self) -> String {
        std::fs::read_to_string(self.top.join("elsewhere/secret.md")).expect("the outside file")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.top);
    }
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the scratch directory");
    for entry in std::fs::read_dir(from).expect("the fixture tree").flatten() {
        let target = to.join(entry.file_name());
        match entry.path().is_dir() {
            true => copy(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("a fixture file");
            }
        }
    }
}

/// A symlink is never followed, in either direction and whatever the pattern:
/// each is refused as an error, reads nothing, and leaves `check --fix`
/// nothing to write through the link.
#[test]
fn a_symlink_is_refused_in_both_directions_and_nothing_is_read_or_written_through_it() {
    let scratch = Scratch::new("symlink");
    let before = scratch.secret();
    for listed in [
        "link-out.md",
        "link-in.md",
        "linked-dir/secret.md",
        "front/*.md",
    ] {
        let (taken, run) = checked_at(&scratch.repo(), Some(&[listed]));
        assert_eq!(taken.outside.rows.len(), 0, "{listed} reads nothing");
        assert_eq!(taken.outside.refused.len(), 1, "{listed} is refused");
        assert!(
            taken.outside.refused[0].reason.contains("symlink"),
            "{listed}: {}",
            taken.outside.refused[0].reason
        );
        assert_eq!(refusals(&run).len(), 1, "{listed} is an error");
        let through: Vec<&Finding> = run
            .findings
            .iter()
            .filter(|finding| finding.rule != "language.outside_root.refused")
            .filter(|finding| !finding.path.starts_with("docs/"))
            .collect();
        assert_eq!(through, Vec::<&Finding>::new(), "{listed}");
        let patches: Vec<_> = run
            .findings
            .iter()
            .filter_map(|finding| finding.patch.clone())
            .collect();
        assert!(patches.is_empty(), "{listed}: nothing for --fix to write");
    }
    assert_eq!(scratch.secret(), before, "the outside file is untouched");
}
