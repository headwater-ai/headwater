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
/// holds every row and no total, which is a deliberate reversal of what it
/// held before. It used to hold the totals and only the rows that were not
/// typed documents, on the argument that a file which changes on every commit
/// is a file nobody reads. That argument was about a reader. This one is about
/// a merge, and it wins because the earlier form is not merely noisy but
/// wrong: a total is a fold over the rows, two branches that each add one
/// document both rewrite the same total to the same value, and the merge takes
/// it without a conflict for a tree that holds one more file than the number
/// says. [`Census::render_rows`] carries the argument in full.
///
/// A shrinking denominator, which is the failure this census exists to catch,
/// is still what the diff shows. It shows it better: a deleted row names the
/// file that vanished, where a total that fell by one named nothing.
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
        &taken.render_rows(Detail::EveryRow),
    );
}

/// The recorded rows are in path order, and the order is what makes them merge.
///
/// Two branches that each add a document insert one record each. They merge
/// cleanly when the two records sit apart, and they raise an ordinary conflict
/// when they sit within a few lines of each other. Both outcomes are safe. An
/// unordered walk gives up the first and keeps none of the safety, because two
/// records could then land in one place for no reason a reader could predict.
///
/// So the ordering is not a presentation choice that a later change may take
/// back. It is the property the recorded form rests on, and this is the test
/// that says so.
#[test]
fn the_census_rows_are_in_path_order() {
    let root = repository_root();
    let taken = census::take(&corpus_of(&root), &resolved_taxonomy(&root));

    let paths: Vec<&str> = taken.rows.iter().map(|row| row.path.as_str()).collect();
    let mut sorted = paths.clone();
    sorted.sort_unstable();
    assert_eq!(
        paths, sorted,
        "the census rows are not in path order, so the recorded form no longer merges"
    );
}

/// Every generated document is declared unmergeable, and a list is why this runs.
///
/// A generated document opens with a count of the shelf below it, which is a
/// fold, and [HW-DR-0049](../../../../docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md)
/// rules that a fold answers to a check on the merged state rather than to a
/// merge. `.gitattributes` names each one, and it names them one at a time
/// rather than by a pattern, because `docs/*/README.md` reaches two files that
/// nobody generates and somebody edits by hand.
///
/// A list goes stale, and this is the guard on it. A new shelf brings a new
/// index, `headwater generate` writes it, and nothing else would notice that
/// the new file merges the way the old ones must not. The census already knows
/// which files are generated, so the list is checked against the corpus rather
/// than against somebody's memory of it.
#[test]
fn the_generated_documents_are_declared_unmergeable() {
    let root = repository_root();
    let taken = census::take(&corpus_of(&root), &resolved_taxonomy(&root));

    let attributes =
        std::fs::read_to_string(root.join(".gitattributes")).expect("the attributes file");
    let declared: Vec<&str> = attributes
        .lines()
        .filter(|line| line.contains("merge=headwater-regenerate"))
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    assert!(
        declared.len() > 5,
        "only {} paths declare the driver, so this proves nothing",
        declared.len()
    );

    let generated: Vec<&str> = taken
        .rows
        .iter()
        .filter(|row| row.outcome.class() == "generated")
        .map(|row| row.path.as_str())
        .collect();
    assert!(
        !generated.is_empty(),
        "the census reports no generated document, so this proves nothing"
    );

    let missing: Vec<&str> = generated
        .iter()
        .filter(|path| !declared.contains(*path))
        .copied()
        .collect();
    assert!(
        missing.is_empty(),
        "these generated documents merge like ordinary files, and each one opens \
         with a count that two branches would both move: {missing:#?}"
    );
}

/// Every page the figure refresh writes is declared unmergeable, enumerated.
///
/// The test above holds the documents `headwater generate` writes. This holds
/// the other producer of a fold in this repository. `tools/site/refresh-figures.sh`
/// substitutes a measured number into every element carrying `data-figure` on
/// every page under `site/`, so a page that carries one holds a fold in its
/// markup exactly the way a generated index holds one in its opening line.
///
/// Decomposition does not save these pages, and that is worth stating because
/// they look decomposed. There is one element per figure and 57 of them on the
/// landing page, so two branches that move *different* figures do merge
/// correctly. The case that survives is two branches that move the *same*
/// figure to the *same* new value: both add a document, both write
/// `census.seen">426`, git reads one change written twice, takes 426 with no
/// conflict, and the union is 427.
///
/// The set is enumerated from the pages rather than taken from a list, because
/// every hand-held account of it has been short. The contended set was
/// published as five artifacts and measured seven; the driver set was answered
/// as seventeen the day after a tenth shelf index made it eighteen; the count
/// of pages here was written as four and measured three.
/// [#676](https://github.com/headwater-ai/headwater/issues/676) owns the
/// general question of what enumerates the derived artifacts, and this is the
/// local instance of it.
#[test]
fn every_page_carrying_a_figure_is_declared_unmergeable() {
    let root = repository_root();

    // The producer's own two literals. This test enumerates the same way
    // `tools/site/refresh-figures.sh` does, and a copy of a rule goes stale in
    // silence, so the copy is held against the original rather than trusted.
    let producer = std::fs::read_to_string(root.join("tools/site/refresh-figures.sh"))
        .expect("the figure refresh");
    for literal in ["site/**/*.html", "data-figure="] {
        assert!(
            producer.contains(literal),
            "tools/site/refresh-figures.sh no longer says {literal}, so this test \
             enumerates a set the producer has stopped writing"
        );
    }

    let mut pages = Vec::new();
    collect_html(&root.join("site"), &root, &mut pages);
    pages.sort();
    assert!(
        pages.len() > 5,
        "only {} pages under site/, so this proves nothing",
        pages.len()
    );

    let written: Vec<&String> = pages
        .iter()
        .filter(|page| {
            std::fs::read_to_string(root.join(page))
                .expect("a page")
                .contains("data-figure=")
        })
        .collect();
    assert!(
        !written.is_empty(),
        "no page under site/ carries a figure, so this proves nothing. Either \
         the refresh writes nothing or the marker was renamed"
    );

    let attributes =
        std::fs::read_to_string(root.join(".gitattributes")).expect("the attributes file");
    let declared: Vec<&str> = attributes
        .lines()
        .filter(|line| line.contains("merge=headwater-regenerate"))
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    assert!(
        declared.len() > 5,
        "only {} paths declare the driver, so this proves nothing",
        declared.len()
    );

    let missing: Vec<&&String> = written
        .iter()
        .filter(|page| !declared.contains(&page.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "these pages carry a figure the refresh rewrites and merge like \
         ordinary files, so two branches that move one figure to one value \
         merge to a number true of neither: {missing:#?}"
    );

    // The other direction. A page that stops carrying a figure and keeps the
    // attribute refuses a merge of a file somebody now edits by hand, which
    // `.gitattributes` names as the worse of the two failures.
    let stale: Vec<&&str> = declared
        .iter()
        .filter(|path| path.starts_with("site/"))
        .filter(|path| !written.iter().any(|page| page.as_str() == **path))
        .collect();
    assert!(
        stale.is_empty(),
        "these pages are declared unmergeable and carry no figure, so the \
         declaration now refuses a merge of hand-written text: {stale:#?}"
    );
}

/// Every `.html` file under a directory, as a path relative to the root.
fn collect_html(dir: &Path, root: &Path, found: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_html(&path, root, found);
        } else if path.extension().is_some_and(|it| it == "html") {
            found.push(
                path.strip_prefix(root)
                    .expect("a path under the root")
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
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

// --- this repository, resolved -----------------------------------------------
//
// `headwater-resolve` reads the consumer declaration, resolves the package plus
// the bundle plus the overlay, and hands back both halves: what to walk, and
// the taxonomy to walk it with. It replaced the stand-in that this crate used
// to carry, and `corpus.census` did not move when it did.

fn repository(root: &Path) -> headwater_resolve::Repository {
    headwater_resolve::repository(root)
        .unwrap_or_else(|errors| panic!("{}", headwater_resolve::render_errors(&errors)))
}

fn resolved_taxonomy(root: &Path) -> Taxonomy {
    Taxonomy::read(&repository(root).resolution.taxonomy).expect("the resolved taxonomy reads")
}

fn corpus_of(root: &Path) -> Corpus {
    let consumer = repository(root).consumer;
    Corpus::declared(root, &consumer.corpus_root, &consumer.exclusions)
}

// --- the whole derived-artifact population ------------------------------------
//
// The two cases above hold two halves of one population against `.gitattributes`,
// each half enumerated from one producer. They are the right shape and they
// cover 15 of the 24 paths that carry the attribute. The three cases below hold
// the whole of it, from `headwater_census::derived`, which enumerates every
// producer by that producer's own rule.
//
// [#676](https://github.com/headwater-ai/headwater/issues/676) is the reason
// they exist, and the reason is not that the two above were wrong. It is that
// three separate hand-written statements of this one population disagreed in
// one tree at one commit, by 18 and by 3.

/// The computed population and `.gitattributes` agree, in both directions.
///
/// This generalizes `the_generated_documents_are_declared_unmergeable` and
/// `every_page_carrying_a_figure_is_declared_unmergeable` over all four
/// producers rather than replacing either. Those two hold a producer's rule
/// against the attribute; this holds the union, so a producer output that
/// belongs to neither of their two rules can no longer be missed.
#[test]
fn every_producer_output_is_declared_and_every_declared_path_has_a_producer() {
    let root = repository_root();
    let population = headwater_census::derived::population(&root);

    assert!(
        population.outputs.len() > 5,
        "only {} producer outputs, so this proves nothing: a rule has stopped \
         matching and the report would be silently short",
        population.outputs.len()
    );
    for producer in headwater_census::derived::PRODUCERS {
        assert!(
            population
                .outputs
                .iter()
                .any(|output| output.producer == *producer),
            "`{}` claims no file of this tree, so its rule ({}) no longer \
             matches and this case proves nothing about it",
            producer.command(),
            producer.rule()
        );
    }

    assert!(
        population.undeclared.is_empty(),
        "a producer writes these and no `merge=headwater-regenerate` covers \
         them, so two branches that move one to the same value merge it \
         silently: {:#?}",
        population.undeclared
    );
    assert!(
        population.unproduced.is_empty(),
        "these declare `merge=headwater-regenerate` and no producer writes \
         them, so the declaration refuses a merge of hand-written text: {:#?}",
        population.unproduced
    );
}

/// The producer's own two literals, held against the copy this crate enumerates by.
///
/// `every_page_carrying_a_figure_is_declared_unmergeable` already holds them
/// for its own copy of the rule. `headwater_census::derived` is a second reader
/// of the same script, so it needs the same guard: a copy of a rule goes stale
/// in silence.
#[test]
fn the_figure_refresh_still_writes_the_set_the_population_enumerates() {
    let root = repository_root();
    let producer =
        std::fs::read_to_string(root.join("tools/site/refresh-figures.sh")).expect("the refresh");
    for literal in ["site/**/*.html", headwater_census::derived::FIGURE] {
        assert!(
            producer.contains(literal),
            "tools/site/refresh-figures.sh no longer says {literal}, so \
             `headwater_census::derived` enumerates a set the producer has \
             stopped writing"
        );
    }
}

/// No prose of this repository states the size of the population as a number.
///
/// This is the case the issue is actually about. Every hand statement of this
/// population has been wrong, and at the commit this case was written three of
/// them disagreed at once: [HW-DR-0049](../../../../docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md)'s
/// consequence clause said six, `.githooks/merged-fold-check`'s header said
/// twenty-one, and `.gitattributes` held twenty-four. A count restated by hand
/// is the defect, so the remedy is that no hand states one: each of the three
/// sentences names the verb that computes it instead.
#[test]
fn no_statement_of_the_population_carries_a_count() {
    let root = repository_root();
    // (file, the words that locate the sentence stating the population)
    let statements = [
        (
            "docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md",
            "keep their folds",
        ),
        (".githooks/merged-fold-check", "declares"),
        (".gitattributes", "Measured rather than assumed"),
    ];
    for (path, locator) in statements {
        let text = std::fs::read_to_string(root.join(path))
            .unwrap_or_else(|e| panic!("{path}: {e}"));
        let sentence = text
            .lines()
            .find(|line| line.contains(locator))
            .unwrap_or_else(|| {
                panic!(
                    "{path} no longer carries a line saying {locator:?}, so this \
                     case no longer reads the sentence it was written for"
                )
            });
        let counted: Vec<&str> = COUNT_WORDS
            .iter()
            .filter(|word| holds_word(sentence, word))
            .copied()
            .collect();
        assert!(
            counted.is_empty(),
            "{path} states the size of the derived-artifact population by hand, \
             as {counted:?}. Every hand statement of it has been wrong. Name \
             `headwater derived` instead:\n  {sentence}"
        );
        assert!(
            sentence.contains("headwater derived"),
            "{path} states the population and does not name the verb that \
             computes it, so a reader has nothing to check it against:\n  {sentence}"
        );
    }
}

/// The words a hand count of this population has been written in.
const COUNT_WORDS: &[&str] = &[
    "six", "seven", "ten", "seventeen", "eighteen", "twenty-one", "twenty-four", "3", "5", "6",
    "7", "10", "17", "18", "21", "23", "24", "25",
];

/// Whether a sentence holds a word, rather than holding it inside another word.
fn holds_word(sentence: &str, word: &str) -> bool {
    sentence
        .to_ascii_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '-')
        .any(|found| found == word)
}

/// A planted producer output is found by a producer's rule, not by a list.
///
/// The tree of this repository agrees in both directions, which is the correct
/// answer and is why it is not evidence. The original witness the issue named
/// is gone too: `site/index.html` and `site/proof/index.html` carried no
/// attribute when [#676](https://github.com/headwater-ai/headwater/issues/676)
/// was filed, and #720 declared them. So the disagreement is provoked here, in
/// both directions at once.
///
/// The load of this case is the unedited verb. Nothing in
/// `headwater_census::derived` names `site/planted/index.html`. The figure
/// refresh writes every page under `site/` that carries a `data-figure`
/// element, the planted page carries one, and that is the whole reason it is
/// reported. A rule that read a list would report nothing here.
#[test]
fn a_planted_producer_output_and_a_planted_orphan_are_both_reported() {
    let root = TempTree::new("planted");
    root.write(
        ".gitattributes",
        "# a comment naming merge=headwater-regenerate, which is not a declaration\n\
         docs/shelf/README.md merge=headwater-regenerate\n\
         docs/nobody/README.md merge=headwater-regenerate\n",
    );
    // Produced and declared: neither direction reports it.
    root.write(
        "docs/shelf/README.md",
        "<!-- headwater:generated shelf_index. -->\n\n# A shelf\n",
    );
    // Produced by the figure refresh and declared by nothing.
    root.write(
        "site/planted/index.html",
        "<p><span data-figure=\"census.seen\">426</span></p>\n",
    );
    // Declared and written by no producer.
    // (no file at docs/nobody/README.md, and a file there with no marker would
    // read the same way)

    let population = headwater_census::derived::population(root.path());

    assert_eq!(
        population
            .undeclared
            .iter()
            .map(|output| (output.path.as_str(), output.producer))
            .collect::<Vec<_>>(),
        vec![(
            "site/planted/index.html",
            headwater_census::derived::Producer::FigureRefresh
        )],
        "a page the figure refresh writes carries no attribute and the report \
         missed it, or it named the wrong producer"
    );
    assert_eq!(
        population.unproduced,
        vec!["docs/nobody/README.md".to_string()],
        "a declared path that no producer writes was not reported"
    );
    assert!(!population.agrees(), "the report claims the tree agrees");

    let rendered = population.render();
    for expected in [
        "site/planted/index.html",
        "docs/nobody/README.md",
        "sh tools/site/refresh-figures.sh",
    ] {
        assert!(
            rendered.contains(expected),
            "the report does not name {expected}:\n{rendered}"
        );
    }
}

/// A recorded fixture is a member only where its opening states a fold.
///
/// This is the rule that separates `corpus.checks` from `corpus.census`.
/// HW-DR-0049 decomposed the second so that it merges, and a decomposed
/// artifact that declared the driver would refuse a merge it is built to take.
#[test]
fn a_decomposed_recorded_fixture_is_not_a_member_and_a_folded_one_is() {
    let root = TempTree::new("folds");
    root.write(".gitattributes", "");
    root.write(
        "engine/crates/check/fixtures/corpus.checks",
        "491 seen, 321 classified\n  a finding\n",
    );
    root.write(
        "engine/crates/lock/fixtures/corpus.lock",
        "headwater/standard 4.3.0\nsha256:abcdef\n",
    );
    root.write(
        "engine/crates/census/fixtures/corpus.census",
        "docs/LICENSE\n  not a document\n",
    );

    let population = headwater_census::derived::population(root.path());
    let claimed: Vec<&str> = population
        .outputs
        .iter()
        .map(|output| output.path.as_str())
        .collect();
    assert_eq!(
        claimed,
        vec![
            "engine/crates/check/fixtures/corpus.checks",
            "engine/crates/lock/fixtures/corpus.lock"
        ],
        "the fold rule claimed the wrong recorded fixtures"
    );
}

/// A tree under a directory this process owns, removed when the case ends.
///
/// Keyed on the process identifier and a label, because `cargo` runs the cases
/// of one target as threads of one process and a helper keyed on the pid alone
/// races with its siblings.
struct TempTree(PathBuf);

impl TempTree {
    fn new(label: &str) -> TempTree {
        let path = std::env::temp_dir().join(format!(
            "headwater-derived-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("a temporary tree");
        TempTree(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, relative: &str, text: &str) {
        let at = self.0.join(relative);
        std::fs::create_dir_all(at.parent().expect("a parent")).expect("a directory");
        std::fs::write(&at, text).expect("a file");
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
