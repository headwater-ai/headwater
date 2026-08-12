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
use headwater_census::standin;
use headwater_census::walk::{Corpus, Exclusion};
use headwater_yaml::Mapping;
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
// It lives in `headwater_census::standin`, and its module comment says what it
// is and what it is not. Two readers now need the base package, the bundle and
// the overlay put together — this test and the graph build's — and two copies
// of a stand-in can disagree about the taxonomy while each one passes its own
// fixtures.

fn resolved_taxonomy(root: &Path) -> Taxonomy {
    Taxonomy::read(&standin::resolved(root)).expect("the resolved taxonomy reads")
}

fn corpus_of(root: &Path) -> Corpus {
    standin::corpus(root)
}
