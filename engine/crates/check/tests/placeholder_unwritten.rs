// SPDX-License-Identifier: Apache-2.0
//! A document that still holds the text the scaffolder wrote for a person to
//! replace, and the readings of that question that are easy to get wrong.
//!
//! [The check layer](../../../../docs/spec/check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship)
//! sets the floor: "every check ships with at least one fixture that it fails
//! and one that it passes." The tree under `fixtures/placeholder/` is
//! [HW-OBL-0113](../../../../docs/obligations/0113-every-check-passes-a-document-that-is-still-the-scaffolder-s-placeholder.md).
//!
//! **A rule that read any paragraph that starts with `TODO`** reports
//! `authored-todo.md`, whose `TODO: pick the pool size before launch.` is an
//! author's note and not the prompt.
//!
//! **A rule that searched the source for the string** reports `fenced.md`,
//! which quotes the prompt in a fenced block, and `span.md`, which quotes it in
//! an inline code span.
//!
//! **A rule that read a paragraph as authored when any one run is** reports
//! `mixed-span.md`, whose ``TODO write `this` section.`` reads as the prompt
//! once its runs are joined, and whose one word is in a code span. The
//! scaffolder writes no code span, so the author wrote that paragraph.
//!
//! **A rule that read "contains the prompt"** reports `started.md`, whose one
//! paragraph goes on after the prompt.
//!
//! **A rule that read every authored run** reports `quoted.md`, whose prompt
//! is in a block quotation and so belongs to another author.
//!
//! **A rule that read every kind of block** reports `item-and-heading.md`,
//! whose prompt is a tight list item and a heading. The scaffolder writes it
//! as a paragraph under a heading, and nowhere else.
//!
//! **A rule that read `TODO <any facet>`** reports `other-key.md`, whose
//! `owner` reads `TODO summary`. The scaffolder writes `TODO owner` into
//! `owner`, and nothing else.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Register, Run, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

/// As every other recorded run: a verdict is a function of the injected clock.
const PINNED: &str = "2026-08-12";

const RULE: &str = headwater_check::placeholder::RULE;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn run() -> (census::Census, Run) {
    let corpus = Corpus::new(fixtures_dir(), "placeholder");
    let source = std::fs::read_to_string(fixtures_dir().join("placeholder.taxonomy.yml"))
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
    let run = headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: "sha256:placeholder-fixture",
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
            source: "engine/crates/check/fixtures/placeholder.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    );
    (taken, run)
}

/// The lines this rule names in one file of the tree, in report order.
fn lines(run: &Run, file: &str) -> Vec<usize> {
    let path = format!("placeholder/{file}");
    run.findings
        .iter()
        .filter(|finding| finding.rule == RULE && finding.path == path)
        .map(|finding| finding.line)
        .collect()
}

/// The decisive case: a document as the scaffolder wrote it is named once for
/// each prompt it still holds, at the line of the prompt, as a warning with
/// no fix.
#[test]
fn a_scaffolded_document_is_named_once_for_each_prompt_it_still_holds() {
    let (_, run) = run();
    // `summary` and `owner` in the front matter, then the two sections.
    assert_eq!(lines(&run, "decisions/scaffolded.md"), [2, 3, 10, 14]);
    for finding in run.findings.iter().filter(|finding| finding.rule == RULE) {
        assert_eq!(finding.severity, headwater_check::Severity::Warn);
        assert!(
            finding.patch.is_none(),
            "only a person can write the section"
        );
    }
}

/// The pass, and every neighbor reading that would report a document this rule
/// passes.
#[test]
fn a_written_document_and_each_neighbor_reading_pass() {
    let (taken, run) = run();
    for file in [
        "decisions/written.md",
        "decisions/authored-todo.md",
        "decisions/fenced.md",
        "decisions/span.md",
        "decisions/mixed-span.md",
        "decisions/started.md",
        "decisions/quoted.md",
        "decisions/other-key.md",
        "decisions/item-and-heading.md",
    ] {
        let path = format!("placeholder/{file}");
        // Classified, so the silence is a verdict and not an omission.
        assert!(
            taken.rows.iter().any(|row| row.path == path
                && matches!(row.outcome, census::Outcome::Typed { .. })),
            "{path} is not classified, so this case proves nothing"
        );
        assert!(
            run.instances
                .iter()
                .any(|instance| instance.rule == RULE && instance.paths().contains(&path.as_str())),
            "{path} has no instance of the rule, so this case proves nothing"
        );
        assert_eq!(lines(&run, file), Vec::<usize>::new(), "{path}");
    }
}

/// A kind that requires neither a section nor a facet is one the scaffolder
/// writes no prompt into, so the rule has no instance over it.
#[test]
fn a_kind_that_requires_nothing_has_no_instance() {
    let (_, run) = run();
    let path = "placeholder/notes/requires-nothing.md";
    assert!(!run
        .instances
        .iter()
        .any(|instance| instance.rule == RULE && instance.paths().contains(&path)));
    assert_eq!(
        lines(&run, "notes/requires-nothing.md"),
        Vec::<usize>::new()
    );
}
