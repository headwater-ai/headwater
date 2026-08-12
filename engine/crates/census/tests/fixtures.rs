// SPDX-License-Identifier: Apache-2.0
//! The walker's fixture tree, and the census of this repository.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots)
//! asks for the first by name: the census walker "ships with a fixture tree of
//! the pathological cases". The tree under `fixtures/walk/` is that, and the
//! recorded census beside it is what the walk must make of it.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-census --test fixtures
//!
//! Read the diff before committing it. A blessed fixture is the change, and for
//! this crate a blessed fixture is a change to the denominator.

use headwater_census::census::{self, Detail, Outcome};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::{Corpus, Exclusion};
use headwater_yaml::{Entry, Mapping, Span, Spanned, Value};
use std::path::{Path, PathBuf};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

fn blessing() -> bool {
    std::env::var_os("HEADWATER_BLESS").is_some()
}

fn compare(recorded: &Path, actual: &str) {
    if blessing() {
        std::fs::write(recorded, actual).expect("cannot write the expectation");
        return;
    }
    let expected = std::fs::read_to_string(recorded).unwrap_or_else(|e| {
        panic!(
            "{}: {e}. Run with HEADWATER_BLESS=1 to record it.",
            recorded.display()
        )
    });
    assert_eq!(
        expected,
        actual,
        "\nthe census no longer matches {}",
        recorded.display()
    );
}

fn load_map(path: &Path) -> Mapping {
    let source =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    headwater_yaml::load(&source)
        .unwrap_or_else(|errors| panic!("{}: {:?}", path.display(), errors))
        .value
        .as_map()
        .unwrap_or_else(|| panic!("{} is not a mapping", path.display()))
        .clone()
}

/// The pathological tree, every row recorded.
///
/// Every case here is one way a walk can lose a file without failing. The
/// recorded file is the whole assertion, because the thing under test is what
/// the walk *reports*, and a hand-written assertion per case would let a new
/// case arrive with no report at all.
#[test]
fn the_pathological_tree_walks_to_the_recorded_census() {
    let corpus = Corpus::new(fixtures_dir(), "walk").excluding(vec![Exclusion::new(
        "walk/excluded/**",
        "a declared exclusion, so that the fixture tree has one",
    )]);
    let taxonomy = Taxonomy::read(&load_map(&fixtures_dir().join("walk.taxonomy.yml")))
        .expect("the fixture taxonomy reads");

    let taken = census::take(&corpus, &taxonomy);

    // Every file, and no directory. A directory is not a file, and a walk that
    // counted one would inflate the denominator instead of shrinking it, which
    // is the same defect with the opposite sign.
    assert!(
        !taken.rows.iter().any(|row| row.path == "walk/nested"
            || row.path == "walk/directory.md"
            || row.path == "walk/spec"),
        "a directory reached the census"
    );

    compare(
        &fixtures_dir().join("walk.census"),
        &taken.render(Detail::EveryRow),
    );
}

/// This repository, typed by the taxonomy that types it.
///
/// M1 exists to put the real corpus in front of the code. The recorded file
/// holds the totals and every row that is not a typed document: a corpus adds a
/// typed document most weeks, and a recorded file that changes on every commit
/// is a file nobody reads. The totals still account for every file, so the
/// failure this census exists to catch — a shrinking denominator — still shows
/// up in the diff.
#[test]
fn this_repository_takes_the_recorded_census() {
    let root = repository_root();
    let taken = census::take(&corpus_of(&root), &resolved_taxonomy(&root));

    assert!(
        taken.rows.len() > 40,
        "only {} files under docs/, which is fewer than this repository has",
        taken.rows.len()
    );
    compare(
        &fixtures_dir().join("corpus.census"),
        &taken.render(Detail::Exceptions),
    );
}

/// The census and the parser's exception list are two accounts of one corpus.
///
/// [#44](https://github.com/headwater-ai/headwater/issues/44) says they must not
/// disagree. They are not the same list and they should not be: the parser
/// records what it could not read, and the census records what became of every
/// file, including the files that no shelf claims and the files a declared
/// exclusion covers. The agreement that has to hold is one implication in each
/// direction.
#[test]
fn the_census_and_the_parsers_exception_list_agree() {
    let root = repository_root();
    let taken = census::take(&corpus_of(&root), &resolved_taxonomy(&root));

    // The parser's fixture, read from where it lives. A copy here would be a
    // second record of one fact, and the two would drift.
    let exceptions = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../doc/fixtures/corpus.exceptions"),
    )
    .expect("the parser's exception list");
    let refused: Vec<&str> = exceptions
        .lines()
        .filter(|line| !line.starts_with(' ') && !line.is_empty())
        .collect();
    assert!(
        !refused.is_empty(),
        "the parser refuses nothing, so this proves nothing"
    );

    for path in &refused {
        let row = taken
            .rows
            .iter()
            .find(|row| row.path == *path)
            .unwrap_or_else(|| {
                panic!("{path} is refused by the parser and absent from the census")
            });
        // A file the parser cannot read may never come back typed. Which of the
        // other outcomes it carries is the census's judgment, not the parser's.
        assert!(
            !matches!(row.outcome, Outcome::Typed { .. }),
            "{path} is typed in the census and refused by the parser"
        );
    }

    // And the other direction: an unreadable row is a file the parser refused,
    // so a new one cannot appear here without appearing there too.
    for row in &taken.rows {
        if matches!(row.outcome, Outcome::Unreadable(_)) {
            assert!(
                refused.contains(&row.path.as_str()),
                "{} is unreadable in the census and absent from the parser's list",
                row.path
            );
        }
    }
}

// --- the M1 stand-in for overlay resolution ----------------------------------
//
// The consumer declaration names a package, a bundle selection and an overlay,
// and resolving those three into one taxonomy is the overlay resolver, which is
// [#50](https://github.com/headwater-ai/headwater/issues/50). Until it exists,
// this test needs shelves from somewhere, and the somewhere is here: the base
// package's `shelves` and `kinds` blocks, then every `shelves.*` and `kinds.*`
// address that the bundle and the overlay add.
//
// It is add-only and it does no confluence check, which is exactly the part
// that makes it a stand-in rather than a resolver. `tools/abox-check.py` does
// the same thing in Python and for the same reason; both go away together.

const BASE: &str = "docs/evaluations/default-taxonomy-first-run.md";
const BUNDLE: &str = "docs/taxonomies/design-spec/bundle.yml";
const OVERLAY: &str = ".headwater/overlay.yml";
const CONSUMER: &str = ".headwater/taxonomy.yml";

fn resolved_taxonomy(root: &Path) -> Taxonomy {
    let base = base_package(root);
    let mut shelves = Vec::new();
    let mut kinds = Vec::new();
    collect_block(&base, "shelves", &mut shelves);
    collect_block(&base, "kinds", &mut kinds);

    for source in [BUNDLE, OVERLAY] {
        let overlay = load_map(&root.join(source));
        let Some(adds) = overlay.get("add").and_then(|value| value.value.as_map()) else {
            continue;
        };
        for entry in adds {
            collect_add(entry, "shelves.", &mut shelves);
            collect_add(entry, "kinds.", &mut kinds);
        }
    }

    let synthetic = Mapping::new(vec![
        named("shelves", Value::Map(Mapping::new(shelves))),
        named("kinds", Value::Map(Mapping::new(kinds))),
    ]);
    Taxonomy::read(&synthetic).expect("the resolved taxonomy reads")
}

/// The base package, which is committed as a fenced block inside an evaluation
/// and nowhere else. The parser can find it, which is the tidy part of an
/// otherwise untidy arrangement: the block is code in a Markdown body, and
/// `headwater_doc` already reports the body's code blocks.
fn base_package(root: &Path) -> Mapping {
    let source = std::fs::read_to_string(root.join(BASE)).expect("the first-run walkthrough");
    let document = headwater_doc::parse(&source).expect("the walkthrough parses");
    for block in &document.body.blocks {
        let text = block.text();
        if !text.contains("package: headwater/standard") {
            continue;
        }
        return headwater_yaml::load(&text)
            .expect("the base package loads")
            .value
            .as_map()
            .expect("the base package is a mapping")
            .clone();
    }
    panic!("no base package in {BASE}");
}

fn collect_block(root: &Mapping, key: &str, into: &mut Vec<Entry>) {
    let Some(block) = root.get(key).and_then(|value| value.value.as_map()) else {
        return;
    };
    into.extend(block.iter().cloned());
}

/// One `add` address, if it names a member of `block` directly.
///
/// A deeper address (`kinds.design_spec.identifier`) reaches inside a member
/// that a previous operation declared, and this stand-in does not merge. It
/// skips them, and it can: no address of that shape declares a shelf or an
/// abstract kind, which are the only two things kind resolution reads.
fn collect_add(entry: &Entry, block: &str, into: &mut Vec<Entry>) {
    let Some(name) = entry.key.value.strip_prefix(block) else {
        return;
    };
    if name.contains('.') {
        return;
    }
    into.push(Entry {
        key: Spanned::new(name.to_string(), entry.key.span),
        value: entry.value.clone(),
    });
}

/// The corpus root and the exclusions, from the consumer declaration.
fn corpus_of(root: &Path) -> Corpus {
    let consumer = load_map(&root.join(CONSUMER));
    let block = consumer
        .get("corpus")
        .and_then(|value| value.value.as_map())
        .expect("the consumer declaration names a corpus");
    let corpus_root = block
        .get("root")
        .and_then(|value| value.value.as_scalar())
        .expect("the corpus names a root")
        .text
        .clone();

    let exclusions = block
        .get("exclude")
        .and_then(|value| value.value.as_seq())
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    let item = item.value.as_map().expect("an exclusion is a mapping");
                    let path = item
                        .get("path")
                        .and_then(|value| value.value.as_scalar())
                        .expect("an exclusion names a path");
                    // The reason is not optional. An exclusion with no reason is
                    // a silent pass with a configuration file in front of it.
                    let reason = item
                        .get("reason")
                        .and_then(|value| value.value.as_scalar())
                        .expect("an exclusion states a reason");
                    Exclusion::new(&path.text, &reason.text)
                })
                .collect()
        })
        .unwrap_or_default();

    Corpus::new(root, &corpus_root).excluding(exclusions)
}

fn named(key: &str, value: Value) -> Entry {
    Entry {
        key: Spanned::new(key.to_string(), Span::default()),
        value: Spanned::new(value, Span::default()),
    }
}
