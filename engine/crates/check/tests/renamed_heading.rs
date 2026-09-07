// SPDX-License-Identifier: Apache-2.0
//! The day the cross-document arm of `link.fragment.unresolved` exists for:
//! somebody renames a heading, and every prose link that cited it by
//! `path.md#fragment` now names a heading that is not there.
//!
//! [Issue #603](https://github.com/headwater-ai/headwater/issues/603) is that
//! run measured on this repository's own corpus. Renaming `### Projection` to
//! `### Projection of a graph` in `docs/spec/glossary.md` turned four *bare*
//! `#projection` links red, all four inside `glossary.md` itself, and produced
//! nothing at all for `docs/spec/06-engine-architecture.md:137`, which writes
//! `[the glossary](glossary.md#projection)`. One broken heading: four findings
//! from inside the file, silence from every document outside it.
//!
//! # This is not `renamed_target.rs`, and the difference is the whole test
//!
//! `tests/renamed_target.rs` moves a *file* and watches `link.path.unresolved`
//! fire. Here the file stays where it is and its *heading* changes, so every
//! path in the tree still resolves and the path rule has nothing to say. The
//! trap this test is written around is the other arm of its own rule: a green
//! same-document arm is not evidence about the cross-document one, so the
//! assertions below name the citing document and the rule together, and the
//! heading that moves is in a document no other test touches.
//!
//! # Why the run before the rename is green
//!
//! `tests/renamed_target.rs` states it: the tree under `fixtures/check/` is a
//! tree of deliberate defects, so the first run collects every
//! `(document, rule)` cell that carries an error and declares all of them as
//! adoption debt, which never blocks. The rename is then the only thing
//! between a green verdict and a red one.
//!
//! `00-both-halves.md` is the citer for that reason and no other: it is the
//! passing half of every prose rule and carries no error cell of its own, so a
//! new finding landing there cannot be absorbed by the payload. The document
//! whose heading moves is `20-fragment-target.md`, which nothing else in the
//! tree points at.

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

const RULE: &str = headwater_check::fragment::RULE;

/// The document whose heading moves, the heading as written, and the heading
/// the test rewrites it to.
const TARGET: &str = "check/spec/20-fragment-target.md";
const HEADING: &str = "## A heading that another document cites";
const RENAMED: &str = "## A heading that another document no longer cites";

/// The document that writes the citation, and the fragment it writes.
const CITED_BY: &str = "check/spec/00-both-halves.md";
const FRAGMENT: &str = "20-fragment-target.md#a-heading-that-another-document-cites";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// A copy of the fixture tree that this test may rewrite a heading inside.
///
/// The label is in the name beside the process id, because cargo runs the cases
/// of one target as threads of one process and a directory keyed on the pid
/// alone is one directory shared by two tests.
fn scratch(label: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "headwater-check-renamed-heading-{}-{label}",
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

/// The whole issue, in one rewrite of one heading.
///
/// On an engine whose fragment rule reads same-document links alone, the third
/// assertion is what fails: the rename produces no finding, the payload still
/// covers every error there is, and `has_errors` stays false. That is the state
/// this repository shipped, and it is what the run in the module comment
/// measured.
#[test]
fn renaming_a_cited_heading_turns_a_green_strict_run_red() {
    let base = scratch("rename");

    let before = run_over(&base, None);
    let payload = debt_of(&before);

    let held = run_over(&base, Some(&payload));
    assert!(
        !held.has_errors(),
        "every error of this tree is declared as debt, and debt never blocks"
    );

    // Nothing moves but the text of one heading. The file keeps its name, its
    // identifier and its front matter, so every path in the tree still
    // resolves and `link.path.unresolved` has nothing to say about this.
    let at = base.join(TARGET);
    let source = std::fs::read_to_string(&at).expect("the target document reads");
    assert!(source.contains(HEADING), "the heading is there to rename");
    std::fs::write(&at, source.replace(HEADING, RENAMED)).expect("the heading moves");

    let after = run_over(&base, Some(&payload));
    assert!(
        after.has_errors(),
        "a rename that broke a cross-document citation left a strict run passing"
    );

    let raised: Vec<&headwater_check::Finding> = after
        .findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .filter(|finding| finding.path == CITED_BY)
        .collect();
    assert_eq!(raised.len(), 1, "{raised:#?}");
    assert_eq!(
        raised[0].rule, RULE,
        "the fragment rule and not the path rule: {:#?}",
        raised[0]
    );
    assert!(
        raised[0].message.contains(FRAGMENT),
        "the finding names the destination the author wrote: {:#?}",
        raised[0]
    );
    assert!(
        raised[0].message.contains(TARGET),
        "and the document whose heading it did not find: {:#?}",
        raised[0]
    );
    assert!(
        raised[0].line > 0 && raised[0].column > 0,
        "at the line and column of the link: {:#?}",
        raised[0]
    );

    let _ = std::fs::remove_dir_all(&base);
}

/// The same-document arm still answers, and it says something else.
///
/// Two arms of one rule with one message would send an author to look for a
/// heading in the wrong file. `00-both-halves.md` writes both kinds of
/// fragment, so breaking the bare one and reading its message is the check that
/// the two sentences stayed apart.
#[test]
fn the_two_arms_of_the_rule_do_not_share_a_sentence() {
    let base = scratch("arms");

    let at = base.join(CITED_BY);
    let source = std::fs::read_to_string(&at).expect("the citing document reads");
    std::fs::write(
        &at,
        source.replace("## Where the pair is written", "## Where the pair was written"),
    )
    .expect("the same-document heading moves");

    let run = run_over(&base, None);
    let raised: Vec<&headwater_check::Finding> = run
        .findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .filter(|finding| finding.path == CITED_BY)
        .collect();
    assert_eq!(raised.len(), 1, "{raised:#?}");
    assert!(
        raised[0].message.contains("#where-the-pair-is-written"),
        "{raised:#?}"
    );
    assert!(
        raised[0].message.contains("this document"),
        "the near arm says `this document` rather than naming a path: {raised:#?}"
    );

    let _ = std::fs::remove_dir_all(&base);
}

/// The rule reads a real population rather than an empty one.
///
/// A rule that reports zero because its population is empty and a rule that
/// reports zero because a corpus is clean are the same number, and
/// `renamed_target.rs` separates them for the path rule. This is that
/// separation for the arm this test is about: the tree carries cross-document
/// fragments that resolve, so the passing verdict above was taken over
/// something.
#[test]
fn the_cross_document_fragments_are_a_population_and_not_an_empty_set() {
    let base = scratch("population");

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

    let across = graph
        .links
        .iter()
        .filter(|link| !link.fragment.as_deref().unwrap_or_default().is_empty())
        .filter(|link| {
            matches!(
                link.binding,
                headwater_graph::links::Binding::Corpus { .. }
            )
        })
        .count();
    assert!(
        across > 0,
        "the cross-document arm would report zero over a tree with no such link to read"
    );

    let _ = std::fs::remove_dir_all(&base);
}
