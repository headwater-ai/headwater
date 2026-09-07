// SPDX-License-Identifier: Apache-2.0
//! The day this rule exists for: somebody renames a document, and every prose
//! link that cited it by path now names a file that is not there.
//!
//! [Issue #406](https://github.com/headwater-ai/headwater/issues/406) is that
//! run measured on this repository's own corpus. `mv docs/spec/glossary.md
//! docs/spec/glossary-2.md` left thirteen dead citations across nine
//! documents, and `headwater check --strict` exited 0. The graph build had
//! already found all thirteen and printed each one with its file, line and
//! column; no rule read the set, so none of them was a finding and none of
//! them moved the exit status.
//!
//! # Why this test does not watch the count the graph prints
//!
//! `crates/graph/fixtures/corpus.graph` records `0 prose links that did not
//! resolve`, and CI diffs it. That number moves on a rename, and it is exactly
//! the instrument an adopter does not have: it holds this repository and no
//! corpus outside it. A test that asserted the count moved would pass on the
//! engine that shipped the defect. So every assertion below is about a
//! **finding** and about `has_errors`, which is what `--strict` reads.
//!
//! # Why the run before the rename is green
//!
//! The tree under `fixtures/check/` is a tree of deliberate defects, so a bare
//! run over it has errors before anything moves. The first run here collects
//! every `(document, rule)` cell that carries one and declares all of them as
//! adoption debt, which
//! [spec 7](../../../../docs/spec/07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states) says never
//! blocks. That is the same instrument `fixtures.rs` uses, and it leaves a copy
//! of this tree passing a strict run. The rename is then the only thing between
//! a green verdict and a red one.
//!
//! Two documents were picked for that reason and no other. The citer is
//! `00-both-halves.md`, the passing half of every prose rule, which carries no
//! error cell of its own: a new finding landing in a cell the payload already
//! held would be absorbed by it, and this test would then pass on an engine
//! that does nothing. The document that moves is `02-cited-only.md`, which
//! carries no error cell either, because a moved file takes its own cells to a
//! path the payload does not name and the run would go red without this rule
//! having said a word. Both halves of that were measured before they were
//! written down: the first draft renamed `13-dangling.md` and passed on an
//! engine with the rule wired out.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Register, Run, Severity, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_yaml::Mapping;
use std::path::{Path, PathBuf};

/// As every other run recorded in this crate: a verdict is a function of the
/// injected clock.
const PINNED: &str = "2026-08-12";

const RULE: &str = headwater_check::link_path::RULE;

/// The document the rename moves, and the one that cites it by path.
const MOVED: &str = "check/spec/02-cited-only.md";
const MOVED_TO: &str = "check/spec/02-cited-only-moved.md";
const CITED_BY: &str = "check/spec/00-both-halves.md";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// A copy of the fixture tree that this test may rename a file inside.
///
/// The label is in the name beside the process id, because cargo runs the
/// cases of one target as threads of one process and a directory keyed on the
/// pid alone is one directory shared by two tests.
fn scratch(label: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "headwater-check-renamed-{}-{label}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).expect("a scratch directory");
    copy_into(&fixtures_dir().join("check"), &at.join("check"));
    std::fs::copy(
        fixtures_dir().join("check.taxonomy.yml"),
        at.join("check.taxonomy.yml"),
    )
    .expect("the fixture taxonomy copies");
    at
}

fn copy_into(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a directory");
    for entry in std::fs::read_dir(from).expect("a readable fixture tree") {
        let entry = entry.expect("a directory entry");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("a file type").is_dir() {
            true => copy_into(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("a file copies");
            }
        }
    }
}

/// One run over the scratch tree, with whatever adoption payload it is handed.
fn run_over(base: &Path, adoption: Option<&Mapping>) -> Run {
    let corpus = Corpus::new(base.to_path_buf(), "check");
    let source = std::fs::read_to_string(base.join("check.taxonomy.yml"))
        .expect("the fixture taxonomy reads");
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
    // The fixture tree has no lock, so the digest of its taxonomy source stands
    // in for one, as it does in `fixtures.rs`.
    let lock = headwater_hash::hex(source.as_bytes());
    headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: &lock,
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            adoption,
            source: "fixtures/check.taxonomy.yml",
        },
        &headwater_check::claim::Claims::at(&corpus.base),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

/// Every `(document, rule)` cell of this run that carries an error, as an
/// adoption payload that declares all of them as debt.
fn debt_of(run: &Run) -> Mapping {
    let mut cells: Vec<(String, String)> = run
        .findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .map(|finding| (finding.path.clone(), finding.rule.to_string()))
        .collect();
    cells.sort();
    cells.dedup();
    let mut pairs = String::new();
    for (path, rule) in &cells {
        use std::fmt::Write;
        let _ = writeln!(pairs, "      - {{path: {path}, rule: {rule}}}");
    }
    headwater_yaml::load(&format!(
        "\
tasks:
  - id: AD-1
    statement: every error this tree raises before the rename, declared as debt
    owner: the fixture tree
    until: 2027-01-01
    pairs:
{pairs}"
    ))
    .expect("the payload loads")
    .value
    .as_map()
    .expect("a mapping")
    .clone()
}

fn findings_of<'a>(run: &'a Run, rule: &str) -> Vec<&'a headwater_check::Finding> {
    run.findings
        .iter()
        .filter(|finding| finding.rule == rule)
        .collect()
}

/// The whole issue, in one movement of one file.
///
/// On an engine with no `link.path.unresolved` the third assertion is what
/// fails: the rename produces no finding at all, the payload still covers every
/// error there is, and `has_errors` stays false. That is the state this
/// repository shipped, and it is what the run in the module comment measured.
#[test]
fn renaming_a_cited_document_turns_a_green_strict_run_red() {
    let base = scratch("rename");

    let before = run_over(&base, None);
    let payload = debt_of(&before);

    let held = run_over(&base, Some(&payload));
    assert!(
        !held.has_errors(),
        "every error of this tree is declared as debt, and debt never blocks"
    );

    // Nothing moves but the name of one file. Its identifier, its front matter
    // and every relation that names it are untouched, which is why no other
    // rule has anything to say about the rename.
    std::fs::rename(base.join(MOVED), base.join(MOVED_TO)).expect("the document moves");

    let after = run_over(&base, Some(&payload));
    assert!(
        after.has_errors(),
        "a rename that broke a citation left a strict run passing"
    );

    let raised: Vec<&headwater_check::Finding> = after
        .findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .filter(|finding| finding.path == CITED_BY)
        .collect();
    assert_eq!(raised.len(), 1, "{raised:#?}");
    assert_eq!(raised[0].rule, RULE);
    assert!(
        raised[0].message.contains("02-cited-only.md"),
        "the finding names the destination the author wrote: {:#?}",
        raised[0]
    );
    assert!(
        raised[0].message.contains(MOVED),
        "and the repository path it resolved to: {:#?}",
        raised[0]
    );
    assert!(
        raised[0].line > 0 && raised[0].column > 0,
        "at the line and column of the link: {:#?}",
        raised[0]
    );

    let _ = std::fs::remove_dir_all(&base);
}

/// The rule reads a real population rather than an empty one.
///
/// A lexical rule that reports zero because its pattern never matches and a
/// rule that reports zero because a corpus is clean are the same number. This
/// separates them: the tree carries prose links that resolve, and the rule
/// passes over them rather than never meeting one.
#[test]
fn the_links_that_resolve_are_a_population_and_not_an_empty_set() {
    let base = scratch("population");
    let run = run_over(&base, None);

    let corpus = Corpus::new(base.clone(), "check");
    let source = std::fs::read_to_string(base.join("check.taxonomy.yml")).expect("the taxonomy");
    let root = headwater_yaml::load(&source)
        .expect("it loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();
    let taxonomy = Taxonomy::read(&root).expect("the taxonomy reads");
    let declarations = Declarations::read(&root).expect("the declarations read");
    let graph = Graph::build(
        &census::take(&corpus, &taxonomy),
        &declarations,
        &Resolvers::over(&corpus),
        &corpus,
        &Config::default(),
    );

    let resolved = graph
        .links
        .iter()
        .filter(|link| !link.binding.is_broken())
        .count();
    assert!(
        resolved > 0,
        "the rule would report zero over a tree with no link to read"
    );
    // And the rule reported on the broken ones rather than on all of them.
    let reported = findings_of(&run, RULE).len();
    assert!(reported > 0 && reported < graph.links.len(), "{reported}");

    let _ = std::fs::remove_dir_all(&base);
}
