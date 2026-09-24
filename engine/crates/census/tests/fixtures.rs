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
        .filter(|line| !line.starts_with('#') && line.split_whitespace().nth(1) == Some("-merge"))
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
        .filter(|line| !line.starts_with('#') && line.split_whitespace().nth(1) == Some("-merge"))
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

/// A gitignored producer output does not reach the population as undeclared.
///
/// This is the #817 fixture: `.headwater/site-deploy/` is build output,
/// `.gitignore` excludes it, and no `merge=headwater-regenerate` declaration
/// can ever cover a path nothing commits. Watched failing against the unfixed
/// `collect`, which walked the filesystem with no regard for `.gitignore` and
/// reported the marker file here as undeclared — the exact shape quoted in the
/// issue, reproduced with a scratch tree in place of a locally built site.
#[test]
fn a_gitignored_producer_output_is_not_undeclared() {
    let root = TempTree::new("gitignored");
    root.git_init();
    root.write(".gitattributes", "");
    root.write(".gitignore", "build-preview/\n");
    root.write(
        "build-preview/corpus.json",
        &format!(
            "{{\n  {},\n  \"documents\": []\n}}\n",
            headwater_mark::marker_member("site_deploy")
        ),
    );

    let population = headwater_census::derived::population(root.path());

    assert!(
        population.undeclared.is_empty(),
        "a gitignored producer output reached the population as undeclared, so \
         a tree that has been locally built once would redden a clean \
         checkout's suite too: {:#?}",
        population.undeclared
    );
}

/// The same rule holds for one gitignored file, not a whole ignored directory.
///
/// Done-when clause 4 of #817 asks that the fix be the ignore rule and not a
/// name: a second gitignored, marker-bearing file anywhere in the tree must
/// not reopen this. A directory-only exclusion would pass the case above and
/// fail this one, so the two together are the generalization the issue asks
/// for.
#[test]
fn a_second_gitignored_marker_bearing_file_does_not_reopen_this() {
    let root = TempTree::new("gitignored-file");
    root.git_init();
    root.write(".gitattributes", "");
    root.write(".gitignore", "scratch.json\n");
    root.write(
        "scratch.json",
        &format!(
            "{{\n  {},\n  \"documents\": []\n}}\n",
            headwater_mark::marker_member("site_deploy")
        ),
    );

    let population = headwater_census::derived::population(root.path());

    assert!(
        population.undeclared.is_empty(),
        "an individually gitignored file, not a whole ignored directory, \
         reached the population as undeclared: {:#?}",
        population.undeclared
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
///
/// # The vocabulary is generated, and the first draft of this case was the
/// # defect one level up
///
/// That draft held a hand-written list of number words. It carried `six`,
/// `seven`, `twenty-one` and thirteen others, and it was missing `four`,
/// `eight`, `nine`, `twenty-two` and `twenty-six`. So a case written to forbid
/// a hand-maintained count was keyed on a hand-maintained list, and a later
/// writer who chose an unlisted word would have passed it. That is the shape of
/// [#676](https://github.com/headwater-ai/headwater/issues/676) itself.
///
/// [`cardinals`] generates the vocabulary from the morphemes of English rather
/// than listing the words. Nine units, ten teens, eight tens and the compounds
/// of the last two give every cardinal below one hundred, plus the scale words
/// above it. A digit is caught by its own rule, so no spelling of a number gets
/// through by being written the other way.
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
    let vocabulary = cardinals();
    for (path, locator) in statements {
        let text =
            std::fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"));
        let sentence = text
            .lines()
            .find(|line| line.contains(locator))
            .unwrap_or_else(|| {
                panic!(
                    "{path} no longer carries a line saying {locator:?}, so this \
                     case no longer reads the sentence it was written for"
                )
            });
        let counted: Vec<String> = words_of(sentence)
            .into_iter()
            .filter(|word| vocabulary.contains(word) || word.chars().all(|c| c.is_ascii_digit()))
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

/// Every cardinal number word of English below one hundred, and the scale words.
///
/// Generated from the morphemes rather than listed, because a list of number
/// words is the defect this file is about. The three words that were actually
/// wrong in this tree are asserted present below, so a generator that stopped
/// generating reddens rather than passing everything.
fn cardinals() -> std::collections::BTreeSet<String> {
    const UNITS: [&str; 9] = [
        "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
    ];
    const TEENS: [&str; 10] = [
        "ten",
        "eleven",
        "twelve",
        "thirteen",
        "fourteen",
        "fifteen",
        "sixteen",
        "seventeen",
        "eighteen",
        "nineteen",
    ];
    const TENS: [&str; 8] = [
        "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
    ];
    const SCALES: [&str; 4] = ["zero", "hundred", "thousand", "million"];

    let mut words: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for word in UNITS.iter().chain(TEENS.iter()).chain(SCALES.iter()) {
        words.insert((*word).to_string());
    }
    for ten in TENS {
        words.insert(ten.to_string());
        for unit in UNITS {
            words.insert(format!("{ten}-{unit}"));
        }
    }
    assert_eq!(
        words.len(),
        9 + 10 + 4 + 8 + 8 * 9,
        "the generator no longer produces every cardinal below one hundred"
    );
    for wrong in ["six", "twenty-one", "twenty-four", "four", "eight", "nine"] {
        assert!(
            words.contains(wrong),
            "the generated vocabulary is missing {wrong}, which is a form a hand \
             count of this population has been or could be written in"
        );
    }
    words
}

/// The prose words of a line, with the spans that hold no prose removed.
///
/// A code span quotes a path or a command and a Markdown link target holds an
/// identifier, and neither is a statement about the population. `#676` inside a
/// link is a name rather than a count, and reading it as one would force every
/// sentence here to cite nothing.
fn words_of(line: &str) -> Vec<String> {
    let mut prose = String::new();
    let mut in_code = false;
    let mut in_link = false;
    for c in line.chars() {
        match c {
            '`' => in_code = !in_code,
            '[' => in_link = true,
            ')' if in_link => in_link = false,
            _ if in_code || in_link => {}
            _ => prose.push(c),
        }
    }
    prose
        .to_ascii_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '-')
        .filter(|word| !word.is_empty())
        .map(str::to_string)
        .collect()
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

    let rendered = population.render(headwater_paint::ColorMode::Plain);
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

/// Every row of the shapes table that names a file has a member in the report.
///
/// This is the guard on the whole change, and it is the same shape as the
/// `for producer in PRODUCERS` loop above. A row that silently stops being
/// reported leaves the report short in a way nothing prints, and three of the
/// four rows had no member at all in the population `headwater derived`
/// computed before this case existed: the two append-only stores are written
/// by no producer, and the two decomposed recorded fixtures are excluded by
/// the fold rule. So the reported set is wider than the population, and this
/// case is what holds it wide.
#[test]
fn every_shape_that_names_a_file_has_a_member_of_this_tree() {
    let root = repository_root();
    let population = headwater_census::derived::population(&root);

    for shape in headwater_census::derived::SHAPES {
        assert!(
            population
                .members
                .iter()
                .any(|member| member.shape == *shape),
            "no path of this tree has the shape {:?} ({}), so that row of \
             `The shapes a record takes` has stopped being reported and the \
             report is short in a way nothing prints",
            shape,
            shape.row()
        );
    }

    assert!(
        population.members.len() > population.outputs.len(),
        "the reported set ({}) is no wider than the computed population ({}), \
         so the paths that no producer writes have dropped out of it",
        population.members.len(),
        population.outputs.len()
    );
}

/// This tree agrees, which is the correct answer and is why it is not evidence.
#[test]
fn no_path_of_this_tree_carries_an_attribute_that_is_not_its_shapes_treatment() {
    let root = repository_root();
    let population = headwater_census::derived::population(&root);

    let mismatched: Vec<String> = population
        .members
        .iter()
        .filter_map(|member| {
            member
                .disagreement()
                .map(|cost| format!("{} ({:?}): {cost}", member.path, member.treatment))
        })
        .collect();
    assert!(
        mismatched.is_empty(),
        "a path carries a merge attribute that its shape does not take: {mismatched:#?}"
    );
    assert!(
        population.unreadable.is_empty(),
        "a line of `.gitattributes` carries a merge attribute this reader \
         cannot expand, so the rule passes over it in silence: {:#?}",
        population.unreadable
    );
}

/// All six disagreements, provoked in one planted tree.
///
/// The fourth of them is the one nothing in this repository could see before
/// this case. A fold declared `merge=union` has an attribute, so the
/// `undeclared` direction stays silent, and a producer writes it, so the
/// `unproduced` direction stays silent too. What `union` then does is
/// interleave two folds into a value that was true on neither branch.
///
/// The tree of this repository provokes none of the six, which is the correct
/// answer for it and is the whole reason this case plants a tree.
#[test]
fn every_disagreement_between_a_shape_and_a_treatment_is_reported() {
    use headwater_census::derived::{Shape, Treatment};

    let root = TempTree::new("shapes");
    root.write(
        ".gitattributes",
        "store/appended.jsonl merge=headwater-regenerate\n\
         engine/crates/a/fixtures/corpus.a merge=headwater-regenerate\n\
         engine/crates/b/fixtures/corpus.b merge=union\n\
         docs/interleaved/README.md merge=union\n\
         engine/crates/ok/fixtures/corpus.ok merge=headwater-regenerate\n\
         store/ok.jsonl merge=union\n",
    );
    // Row 1, declared for regeneration: no producer rewrites an append store.
    root.write("store/appended.jsonl", "{\"a\":1}\n{\"a\":2}\n");
    // Row 1, declared nothing: every parallel append conflicts.
    root.write("store/conflicting.jsonl", "{\"b\":1}\n");
    // Row 2, declared for regeneration: the driver refuses the merge it takes.
    root.write(
        "engine/crates/a/fixtures/corpus.a",
        "docs/x.md\n  a record\n",
    );
    // Row 2, declared union: two record streams interleave out of order.
    root.write(
        "engine/crates/b/fixtures/corpus.b",
        "docs/y.md\n  a record\n",
    );
    // A fold declared union: the worst of the six, and nothing reported it.
    root.write(
        "docs/interleaved/README.md",
        "<!-- headwater:generated shelf_index. -->\n\n# 48 decisions\n",
    );
    // A fold declared nothing: the direction the verb already reported.
    root.write(
        "site/silent/index.html",
        "<p><span data-figure=\"census.seen\">426</span></p>\n",
    );
    // The three that agree.
    root.write(
        "engine/crates/ok/fixtures/corpus.ok",
        "426 seen\n  a finding\n",
    );
    root.write("store/ok.jsonl", "{\"c\":1}\n");
    root.write(
        "engine/crates/c/fixtures/corpus.c",
        "docs/z.md\n  a record\n",
    );

    let population = headwater_census::derived::population(root.path());
    let found: Vec<(&str, Shape, Treatment)> = population
        .members
        .iter()
        .filter(|member| member.disagreement().is_some())
        .map(|member| (member.path.as_str(), member.shape, member.treatment))
        .collect();

    assert_eq!(
        found,
        vec![
            ("docs/interleaved/README.md", Shape::Fold, Treatment::Union),
            (
                "engine/crates/a/fixtures/corpus.a",
                Shape::RecordPerEntity,
                Treatment::Regenerate
            ),
            (
                "engine/crates/b/fixtures/corpus.b",
                Shape::RecordPerEntity,
                Treatment::Union
            ),
            ("site/silent/index.html", Shape::Fold, Treatment::Unset),
            (
                "store/appended.jsonl",
                Shape::IndependentLines,
                Treatment::Regenerate
            ),
            (
                "store/conflicting.jsonl",
                Shape::IndependentLines,
                Treatment::Unset
            ),
        ],
        "the six disagreements were not all reported, or a path that agrees \
         was reported as one:\n{}",
        population.render(headwater_paint::ColorMode::Plain)
    );
    assert!(!population.agrees(), "the report claims the tree agrees");

    // The three agreeing paths are members and carry no finding.
    for path in [
        "engine/crates/ok/fixtures/corpus.ok",
        "store/ok.jsonl",
        "engine/crates/c/fixtures/corpus.c",
    ] {
        let member = population
            .members
            .iter()
            .find(|member| member.path == path)
            .unwrap_or_else(|| {
                panic!(
                    "{path} is not in the report:\n{}",
                    population.render(headwater_paint::ColorMode::Plain)
                )
            });
        assert!(
            member.disagreement().is_none(),
            "{path} agrees with its shape and was reported as a disagreement"
        );
    }
}

/// The report names the row, the legitimate treatment, and the rebuild command.
///
/// One file of each shape, which is what the contract asks for.
#[test]
fn the_report_names_the_row_the_treatment_and_the_rebuild_for_one_file_of_each_shape() {
    let root = TempTree::new("render");
    root.write(
        ".gitattributes",
        "store/readings.jsonl merge=union\n\
         docs/shelf/README.md merge=headwater-regenerate\n",
    );
    root.write("store/readings.jsonl", "{\"a\":1}\n");
    root.write(
        "docs/shelf/README.md",
        "<!-- headwater:generated shelf_index. -->\n\n# A shelf\n",
    );
    root.write(
        "engine/crates/census/fixtures/corpus.census",
        "docs/LICENSE\n  not a document\n",
    );

    let rendered = headwater_census::derived::population(root.path())
        .render(headwater_paint::ColorMode::Plain);
    for expected in [
        // Row 1: the path, its row, and the treatment that row takes.
        "store/readings.jsonl",
        "A line that depends on nothing",
        "merge=union",
        // Row 2: the path, its row, and what rebuilds it.
        "engine/crates/census/fixtures/corpus.census",
        "One record for each entity, in a fixed order",
        "HEADWATER_BLESS=1 cargo test",
        // Rows 3 and 4, which structure does not separate.
        "docs/shelf/README.md",
        "A fold over those records",
        "A fold over everything",
        "merge=headwater-regenerate",
        "headwater generate",
    ] {
        assert!(
            rendered.contains(expected),
            "the report does not name {expected:?}:\n{rendered}"
        );
    }
}

/// The row names are the evaluation table's words and not a paraphrase.
///
/// The evaluation is accepted and this verb cites it. A row renamed here and
/// not there leaves two vocabularies for one set of shapes.
#[test]
fn the_report_uses_the_words_of_the_evaluation_table() {
    let root = repository_root();
    let evaluation =
        std::fs::read_to_string(root.join("docs/evaluations/what-a-check-can-know.md"))
            .expect("the evaluation reads");
    for shape in headwater_census::derived::SHAPES {
        for row in shape.rows() {
            assert!(
                evaluation.contains(row),
                "`The shapes a record takes` no longer holds a row named \
                 {row:?}, so the report and the evaluation now use two \
                 vocabularies"
            );
        }
    }
}

/// A merge attribute this reader cannot expand is named rather than passed over.
///
/// `.gitattributes` is read as a list of paths. A pattern with a glob character
/// reaches files this reader cannot enumerate, and a declaration nothing reads
/// looks exactly like a declaration that agrees.
#[test]
fn a_merge_attribute_behind_a_glob_is_reported_rather_than_skipped() {
    let root = TempTree::new("glob");
    root.write(
        ".gitattributes",
        "# a comment naming merge=union, which is not a declaration\n\
         docs/**/*.md merge=headwater-regenerate\n\
         vendor/** whitespace=-trailing-space\n",
    );
    root.write("docs/shelf/README.md", "# hand written\n");

    let population = headwater_census::derived::population(root.path());
    assert_eq!(
        population.unreadable,
        vec!["docs/**/*.md".to_string()],
        "a merge attribute behind a glob was passed over, or a pattern with no \
         merge attribute was reported"
    );
    assert!(!population.agrees(), "the report claims the tree agrees");
    assert!(
        population
            .render(headwater_paint::ColorMode::Plain)
            .contains("docs/**/*.md"),
        "the report does not name the pattern it could not read:\n{}",
        population.render(headwater_paint::ColorMode::Plain)
    );
}

/// A path that two shape rules both match takes the shape of the first rule.
///
/// The contract reads the rules in a fixed order, and a producer output is a
/// fold before it is anything else. Each file below is written by a producer
/// and is also one complete record on every line. So the record-stream rule
/// also matches it, and read first it would call the fold `union`-safe. Swap
/// the fold and record-stream arms of `shape_of`, and this case fails.
///
/// A generated JSON projection is not planted here. The marker rule reads the
/// `headwater:generated` key only at the start of a line, and a line of a
/// record stream starts with `{`, so no generated file can be both.
#[test]
fn a_producer_output_whose_every_line_is_a_record_is_a_fold() {
    use headwater_census::derived::{Producer, Shape};

    let root = TempTree::new("order");
    root.write(".gitattributes", "");
    root.write(
        "engine/crates/a/fixtures/corpus.a",
        "{\"digest\":\"sha256:0a1b\"}\n{\"seen\":426}\n",
    );
    root.write(
        "site/figure/index.html",
        "{<span data-figure=\"census.seen\">426</span>}\n",
    );

    let population = headwater_census::derived::population(root.path());
    let report = population.render(headwater_paint::ColorMode::Plain);
    for (path, producer) in [
        ("engine/crates/a/fixtures/corpus.a", Producer::RecordedFold),
        ("site/figure/index.html", Producer::FigureRefresh),
    ] {
        assert!(
            population
                .outputs
                .iter()
                .any(|output| output.path == path && output.producer == producer),
            "{path} is not claimed by {producer:?}, so it cannot test the rule \
             order:\n{report}"
        );
        let member = population
            .members
            .iter()
            .find(|member| member.path == path)
            .unwrap_or_else(|| panic!("{path} is not in the report:\n{report}"));
        assert_eq!(
            member.shape,
            Shape::Fold,
            "{path} is a producer output and every line of it is a record; the \
             record-stream rule was read before the fold rule:\n{report}"
        );
    }
}

/// Inside a git repository, the merge attribute of every path is git's answer.
///
/// The verb once read the root `.gitattributes` alone, as a list of literal
/// paths. Git reads more than that: a `.gitattributes` in any directory, where
/// the deeper file wins; `$GIT_DIR/info/attributes`, which wins over all of
/// them; macros such as `binary`; a pattern with no slash, which matches at any
/// depth below its file; a glob; and a root file that opens with a byte order
/// mark. This tree plants each one, and each one gives git an answer the root
/// reader did not.
///
/// There is no table of expected layouts here. For every file of the tree the
/// case asks `git check-attr merge` and holds the verb's answer against it, so
/// the case passes only when the verb's answer is git's answer.
#[test]
fn derived_agrees_with_git_check_attr_on_every_path() {
    let root = TempTree::new("check-attr");
    root.git_init();
    plant_attribute_layouts(&root, true);

    let population = headwater_census::derived::population(root.path());
    let report = population.render(headwater_paint::ColorMode::Plain);
    let attributes = headwater_census::derived::merge_attributes(root.path());

    let files = files_of(root.path());
    assert!(
        files.len() >= 10,
        "the planted tree is missing files: {files:#?}"
    );
    for path in &files {
        let git = git_treatment(root.path(), path);
        let verb = attributes
            .iter()
            .find(|(declared, _)| declared == path)
            .map(|(_, treatment)| *treatment)
            .unwrap_or(headwater_census::derived::Treatment::Unset);
        assert_eq!(
            verb, git,
            "the verb gives {path} {verb:?}, and `git check-attr merge` gives {git:?}:\n{report}"
        );
        if let Some(member) = population
            .members
            .iter()
            .find(|member| member.path == *path)
        {
            assert_eq!(
                member.treatment, git,
                "the report gives {path} {:?}, and `git check-attr merge` gives {git:?}:\n{report}",
                member.treatment
            );
        }
    }
    for fold in FOLDS {
        assert!(
            population.members.iter().any(|member| member.path == *fold),
            "{fold} is a producer output and is not in the report:\n{report}"
        );
    }
    assert!(
        population.unreadable.is_empty(),
        "git expands its own patterns, so no merge attribute is unreadable \
         inside a repository: {:#?}",
        population.unreadable
    );
}

/// Outside a git repository, the verb reads the root `.gitattributes` alone.
///
/// The same layouts as the case above, with no `git init`. No git answer
/// exists here, so the root file is the whole declaration. A nested file is
/// not read, and a glob in the root file is named as unreadable.
#[test]
fn outside_a_git_repository_the_root_gitattributes_alone_is_read() {
    use headwater_census::derived::Treatment;

    let root = TempTree::new("no-repository");
    plant_attribute_layouts(&root, false);

    let population = headwater_census::derived::population(root.path());
    let report = population.render(headwater_paint::ColorMode::Plain);
    let treatment_of = |path: &str| {
        population
            .members
            .iter()
            .find(|member| member.path == path)
            .unwrap_or_else(|| panic!("{path} is not in the report:\n{report}"))
            .treatment
    };
    for (path, expected) in [
        // The root declares it, and the nested `-merge` is not read.
        ("sub/out.md", Treatment::Regenerate),
        // Only a nested file declares it.
        ("other/gen.md", Treatment::Unset),
        // The root declares it, and the nested `binary` is not read.
        ("m/README.md", Treatment::Regenerate),
    ] {
        assert_eq!(
            treatment_of(path),
            expected,
            "outside a repository {path} did not take the root file's answer:\n{report}"
        );
    }
    assert_eq!(
        population.unreadable,
        vec!["glob/*.md".to_string()],
        "the root reader did not name the glob it cannot expand:\n{report}"
    );
}

/// A merge attribute behind a glob is expanded by git inside a repository.
///
/// The companion of `a_merge_attribute_behind_a_glob_is_reported_rather_than_skipped`,
/// which holds the same tree with no repository.
#[test]
fn a_merge_attribute_behind_a_glob_is_expanded_by_git_inside_a_repository() {
    let root = TempTree::new("glob-git");
    root.git_init();
    root.write(
        ".gitattributes",
        "docs/**/*.md merge=headwater-regenerate\n\
         vendor/** whitespace=-trailing-space\n",
    );
    root.write("docs/shelf/README.md", "# hand written\n");

    let population = headwater_census::derived::population(root.path());
    let report = population.render(headwater_paint::ColorMode::Plain);
    assert!(
        population.unreadable.is_empty(),
        "git expands the glob, so nothing is unreadable:\n{report}"
    );
    assert_eq!(
        population.unproduced,
        vec!["docs/shelf/README.md".to_string()],
        "the glob reaches a hand-written file, and the report did not say so:\n{report}"
    );
}

/// A fold that the committed file declares `-merge` agrees, in a clone with no override.
///
/// This is the tree CI checks out and the tree an unconfigured clone holds
/// ([#1058](https://github.com/headwater-ai/headwater/issues/1058)): the
/// committed `.gitattributes` unsets the merge of each fold, so a merge keeps
/// the current side and records a conflict, and only a clone that ran
/// `headwater init --git --git-config` overrides it with the driver in
/// `$GIT_DIR/info/attributes`. Both answers agree for a fold. The second arm
/// is what makes the first one a measurement: the same fold with no attribute
/// is still reported.
#[test]
fn a_fold_declared_unset_in_the_committed_file_agrees_with_or_without_the_override() {
    use headwater_census::derived::Treatment;
    let fold = "<!-- headwater:generated shelf_index. -->\n\n# Decisions\n";

    let root = TempTree::new("unset-fold");
    root.git_init();
    root.write(".gitattributes", "k/README.md -merge\n");
    root.write("k/README.md", fold);
    let population = headwater_census::derived::population(root.path());
    let report = population.render(headwater_paint::ColorMode::Plain);
    let member = population
        .members
        .iter()
        .find(|member| member.path == "k/README.md")
        .unwrap_or_else(|| panic!("k/README.md is not in the report:\n{report}"));
    assert_eq!(member.treatment, Treatment::Refuse, "{report}");
    assert!(population.agrees(), "a `-merge` fold with no override agrees:\n{report}");
    assert_eq!(
        headwater_census::derived::declared_paths(root.path()),
        vec!["k/README.md".to_string()],
        "a `-merge` line is a declaration, so `init --git` appends no second one"
    );

    root.write(".git/info/attributes", "k/README.md merge=headwater-regenerate\n");
    let population = headwater_census::derived::population(root.path());
    let report = population.render(headwater_paint::ColorMode::Plain);
    assert!(population.agrees(), "a fold under the override agrees:\n{report}");

    let bare = TempTree::new("unattributed-fold");
    bare.git_init();
    bare.write("k/README.md", fold);
    let population = headwater_census::derived::population(bare.path());
    let report = population.render(headwater_paint::ColorMode::Plain);
    assert!(!population.agrees(), "a fold with no attribute is reported:\n{report}");
}

/// A merge driver that the verb does not know is named in the report.
///
/// `merge=ours` is not a treatment that a shape takes. A fold under it is a
/// disagreement, and the driver name is printed rather than read as no
/// attribute.
#[test]
fn a_merge_driver_the_verb_does_not_know_is_named() {
    use headwater_census::derived::Treatment;

    let root = TempTree::new("unknown-driver");
    root.git_init();
    root.write(".gitattributes", "k/README.md merge=ours\n");
    root.write(
        "k/README.md",
        "<!-- headwater:generated shelf_index. -->\n\n# 48 decisions\n",
    );

    let population = headwater_census::derived::population(root.path());
    let report = population.render(headwater_paint::ColorMode::Plain);
    let member = population
        .members
        .iter()
        .find(|member| member.path == "k/README.md")
        .unwrap_or_else(|| panic!("k/README.md is not in the report:\n{report}"));
    assert_eq!(member.treatment, Treatment::Unknown, "{report}");
    assert!(member.disagreement().is_some(), "{report}");
    assert_eq!(
        population.drivers,
        vec![("k/README.md".to_string(), "ours".to_string())],
        "{report}"
    );
    assert!(report.contains("k/README.md merge=ours"), "{report}");
    assert!(!population.agrees(), "{report}");
}

/// Inside a repository, a declared path with no file still reaches git.
///
/// Git answers only for the paths it is asked about, and the walk asks only
/// about files on disk. So the verb also asks about every literal path of the
/// root `.gitattributes`, and about the lock. Without the first, a
/// declaration whose file is gone drops out of the `unproduced` direction.
/// Without the second, a nested file that declares the lock before
/// `headwater taxonomy resolve` has written it is not seen, and
/// `headwater init --git` appends a second declaration.
#[test]
fn a_declared_path_with_no_file_is_asked_of_git_inside_a_repository() {
    let root = TempTree::new("absent");
    root.git_init();
    root.write(
        ".gitattributes",
        "docs/gone.md merge=headwater-regenerate\n",
    );
    root.write(
        ".headwater/.gitattributes",
        "taxonomy.lock merge=headwater-regenerate\n",
    );

    let population = headwater_census::derived::population(root.path());
    let report = population.render(headwater_paint::ColorMode::Plain);
    assert!(
        population.unproduced.contains(&"docs/gone.md".to_string()),
        "a root declaration whose file is gone was not reported:\n{report}"
    );
    assert!(
        !population.agrees(),
        "the report claims the tree agrees:\n{report}"
    );
    assert!(
        headwater_census::derived::declared_paths(root.path())
            .contains(&headwater_census::derived::LOCK.to_string()),
        "a nested declaration of the lock, before the lock exists, was not read:\n{report}"
    );
}

/// A root declaration that begins with `/` is asked of git without the `/`.
///
/// The `/` anchors a pattern to the root, and git refuses the whole
/// `check-attr` run for a path that begins with one. Asked with it, git's
/// answer for every path of the tree is lost, and the nested declaration of
/// `sub/notes.md` below drops out of the report.
#[test]
fn a_root_declaration_with_a_leading_slash_is_asked_of_git_without_it() {
    let root = TempTree::new("leading-slash");
    root.git_init();
    root.write(".gitattributes", "/gone.md merge=headwater-regenerate\n");
    root.write(
        "sub/.gitattributes",
        "notes.md merge=headwater-regenerate\n",
    );
    root.write("sub/notes.md", "# hand written\n");

    let population = headwater_census::derived::population(root.path());
    let report = population.render(headwater_paint::ColorMode::Plain);
    assert_eq!(
        population.refused, None,
        "git refused the question:\n{report}"
    );
    assert_eq!(
        population.unproduced,
        vec!["gone.md".to_string(), "sub/notes.md".to_string()],
        "a declared path with no producer was not reported:\n{report}"
    );
}

/// A root declaration of a path outside the tree does not cost git's answer.
///
/// Git refuses the whole `check-attr` run for `../escape.md`, and no pattern
/// that names it can match a file of the tree. So the verb does not ask about
/// it, and the nested declaration of the lock is still read.
#[test]
fn a_root_declaration_outside_the_tree_does_not_discard_git_s_answer() {
    let root = TempTree::new("escape");
    root.git_init();
    root.write(
        ".gitattributes",
        "../escape.md merge=headwater-regenerate\n",
    );
    root.write(
        ".headwater/.gitattributes",
        "taxonomy.lock merge=headwater-regenerate\n",
    );

    let population = headwater_census::derived::population(root.path());
    let report = population.render(headwater_paint::ColorMode::Plain);
    assert_eq!(
        population.refused, None,
        "git refused the question:\n{report}"
    );
    assert!(
        headwater_census::derived::declared_paths(root.path())
            .contains(&headwater_census::derived::LOCK.to_string()),
        "the nested declaration of the lock was lost:\n{report}"
    );
}

/// The producer outputs `plant_attribute_layouts` writes.
const FOLDS: &[&str] = &[
    "top.md",
    "sub/out.md",
    "other/gen.md",
    "m/README.md",
    "c/d/README.md",
    "i/README.md",
    "glob/a.md",
    "plain/README.md",
];

/// Every layout by which git gives a path a merge attribute that the root file does not state.
///
/// Every file except the `.gitattributes` files and the store is a producer
/// output, so each one is a member of the report and carries a treatment.
fn plant_attribute_layouts(root: &TempTree, repository: bool) {
    let fold = "<!-- headwater:generated shelf_index. -->\n\n# 48 decisions\n";
    root.write(
        ".gitattributes",
        "\u{feff}top.md merge=headwater-regenerate\n\
         sub/out.md merge=headwater-regenerate\n\
         m/README.md merge=headwater-regenerate\n\
         c/d/README.md merge=headwater-regenerate\n\
         i/README.md merge=headwater-regenerate\n\
         glob/*.md merge=headwater-regenerate\n\
         plain/README.md merge=headwater-regenerate\n\
         store/readings.jsonl merge=union\n",
    );
    // A nested file unsets the root's driver.
    root.write("sub/.gitattributes", "out.md -merge\n");
    // A nested file declares a producer output that the root does not name.
    root.write(
        "other/.gitattributes",
        "gen.md merge=headwater-regenerate\n",
    );
    // The `binary` macro expands to `-merge`.
    root.write("m/.gitattributes", "README.md binary\n");
    // A pattern with no slash matches at any depth below its file.
    root.write("c/.gitattributes", "README.md merge=union\n");
    if repository {
        // `$GIT_DIR/info/attributes` wins over every `.gitattributes`.
        root.write(".git/info/attributes", "i/README.md merge=union\n");
    }
    for path in FOLDS {
        root.write(path, fold);
    }
    root.write("store/readings.jsonl", "{\"a\":1}\n");
}

/// Every file of a tree except git's own state, relative to its root.
fn files_of(root: &Path) -> Vec<String> {
    fn walk(dir: &Path, root: &Path, found: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).expect("a directory").flatten() {
            let path = entry.path();
            if entry.file_name() == ".git" {
                continue;
            }
            if path.is_dir() {
                walk(&path, root, found);
            } else {
                let relative = path.strip_prefix(root).expect("under the root");
                found.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut found = Vec::new();
    walk(root, root, &mut found);
    found.sort();
    found
}

/// The treatment `git check-attr merge` gives a path, asked of git directly.
fn git_treatment(root: &Path, path: &str) -> headwater_census::derived::Treatment {
    use headwater_census::derived::Treatment;
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["check-attr", "merge", "--", path])
        .output()
        .expect("git runs");
    assert!(output.status.success(), "git check-attr failed on {path}");
    let text = String::from_utf8_lossy(&output.stdout);
    let value = text
        .trim_end()
        .rsplit(": ")
        .next()
        .expect("git prints a value");
    match value {
        "union" => Treatment::Union,
        "headwater-regenerate" => Treatment::Regenerate,
        "unset" | "binary" => Treatment::Refuse,
        "unspecified" | "set" | "text" => Treatment::Unset,
        other => panic!("this tree plants no merge driver named {other}"),
    }
}

/// A tree under a directory this process owns, removed when the case ends.
///
/// Keyed on the process identifier and a label, because `cargo` runs the cases
/// of one target as threads of one process and a helper keyed on the pid alone
/// races with its siblings.
struct TempTree(PathBuf);

impl TempTree {
    fn new(label: &str) -> TempTree {
        let path =
            std::env::temp_dir().join(format!("headwater-derived-{}-{label}", std::process::id()));
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

    /// Turn this tree into a git repository, so `headwater_vcs::ignored` —
    /// which `population` calls — reads back a `.gitignore` this tree wrote.
    /// Git needs a repository to resolve an ignore rule at all: a
    /// `.gitignore` beside no `.git` binds nothing, the same fact
    /// `headwater-vcs`'s own `a_tree_with_no_git_repository_reports_nothing_ignored`
    /// covers from the other side.
    fn git_init(&self) {
        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(&self.0)
            .args(["init", "-q"])
            .status()
            .expect("git runs");
        assert!(status.success(), "git init failed in {}", self.0.display());
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
