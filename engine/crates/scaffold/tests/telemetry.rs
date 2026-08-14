// SPDX-License-Identifier: Apache-2.0
//! The capture-cost store, held to the claims
//! [#74](https://github.com/headwater-ai/headwater/issues/74) makes for it.
//!
//! The unit tests beside the module cover the line format and the file. This
//! file covers the two claims that a format test cannot reach.
//!
//! 1. **A reading is derived from the plan.** It is not a counter that a run
//!    increments beside one, so a field that changes origin changes the reading
//!    with it. The test scaffolds a real document and holds the reading against
//!    [`headwater_scaffold::Plan::assisted`], through the file.
//! 2. **The reach join cannot be made to report a document the store never
//!    saw.** A capture-cost number is worth reading only if a document that
//!    arrived by another route lowers it. Four corpora below, one per way the
//!    join can be wrong.
//!
//! A third claim is about a path rather than about code: the store is outside
//! the corpus root this repository declares, so no census row covers it and no
//! language regime binds it. That one reads the committed consumer declaration,
//! in the way the identifier grammar test reads the committed lock.

use headwater_census::census::{self, Census};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::shape::Shape;
use headwater_check::Date;
use headwater_graph::declarations::Declarations;
use headwater_graph::index::Index;
use headwater_graph::Config;
use headwater_scaffold::reading::{self, Classified, Reach, Reading, Surface, STORE};
use headwater_scaffold::{propose, write, Assisted, Request, Sources};
use headwater_yaml::Mapping;
use std::path::{Path, PathBuf};

const PINNED: &str = "2026-08-14";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
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

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let at = std::env::temp_dir().join(format!("headwater-telemetry-{name}"));
        let _ = std::fs::remove_dir_all(&at);
        copy_tree(&fixtures_dir().join("corpus"), &at.join("corpus"));
        Scratch(at)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the scratch directory");
    for entry in std::fs::read_dir(from)
        .expect("the fixture corpus")
        .flatten()
    {
        let target = to.join(entry.file_name());
        match entry.path().is_dir() {
            true => copy_tree(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("a fixture file");
            }
        }
    }
}

/// Scaffold one document into a tree, and return the plan that wrote it.
fn scaffold(root: &Path, kind: &str, title: &str) -> headwater_scaffold::Plan {
    let resolved = load_map(&fixtures_dir().join("scaffold.taxonomy.yml"));
    let shelves = Taxonomy::read(&resolved).expect("the shelves read");
    let relations = Declarations::read(&resolved).expect("the relations read");
    let shape = Shape::read(&resolved).expect("the shape reads");
    let corpus = Corpus::new(root.to_path_buf(), "corpus");
    let taken: Census = census::take(&corpus, &shelves);
    let config = Config::default();
    let index = Index::build(&taken, &config);
    let plan = propose(
        &Sources {
            resolved: &resolved,
            shape: &shape,
            shelves: &shelves,
            relations: &relations,
            census: &taken,
            index: &index,
            config: &config,
        },
        &Request {
            kind,
            title,
            now: Date::parse(PINNED).expect("the pinned date"),
            relates: &[],
            given: &[],
        },
    )
    .unwrap_or_else(|refusal| panic!("{kind} `{title}`: {refusal}"));
    let composed =
        write::compose(root, &plan).unwrap_or_else(|refusal| panic!("{kind}: {refusal}"));
    write::apply(root, &composed).expect("the files write");
    plan
}

/// What a run wrote, read back off the disk, is what the plan says it wrote.
///
/// A counter incremented beside the plan would pass a format test and fail this
/// one the first time a facet changed origin.
#[test]
fn the_reading_on_disk_is_the_fraction_the_plan_derives() {
    let scratch = Scratch::new("derived");
    let root = &scratch.0;

    let first = scaffold(root, "design_spec", "A scaffolded fourth part");
    let second = scaffold(root, "decision_record", "A scaffolded decision");
    for plan in [&first, &second] {
        let reading = Reading::of(
            plan,
            "sha256:fixture",
            Date::parse(PINNED).expect("a date"),
            Surface::Terminal,
        );
        reading::append(root, &reading).expect("the reading appends");
    }

    let (readings, unreadable) = reading::load(root).expect("the store loads");
    assert_eq!(unreadable, vec![], "every line reads back as a reading");
    assert_eq!(readings.len(), 2);
    assert_eq!(readings[0].assisted, first.assisted());
    assert_eq!(readings[1].assisted, second.assisted());
    assert_eq!(readings[0].document, first.path);
    assert_eq!(
        readings[0].id,
        first.minting.as_ref().map(|minting| minting.id.clone())
    );

    // And the aggregate is the two summed, rather than either one.
    let total = reading::total(&readings);
    assert_eq!(
        total.supplied(),
        first.assisted().supplied() + second.assisted().supplied()
    );
    assert_eq!(
        total.total(),
        first.assisted().total() + second.assisted().total()
    );
    assert!(
        total.total() > first.assisted().total(),
        "a second reading has to move the denominator"
    );
}

/// The store lands beside the file it names and outside what the census walks.
#[test]
fn the_store_lands_outside_the_tree_the_census_walks() {
    let scratch = Scratch::new("placement");
    let root = &scratch.0;
    let plan = scaffold(root, "design_spec", "A scaffolded fourth part");
    let reading = Reading::of(
        &plan,
        "sha256:fixture",
        Date::parse(PINNED).expect("a date"),
        Surface::Terminal,
    );
    reading::append(root, &reading).expect("the reading appends");
    assert!(root.join(STORE).is_file(), "the store is on the tree");

    let resolved = load_map(&fixtures_dir().join("scaffold.taxonomy.yml"));
    let shelves = Taxonomy::read(&resolved).expect("the shelves read");
    let taken = census::take(&Corpus::new(root.to_path_buf(), "corpus"), &shelves);
    assert!(
        !taken
            .rows
            .iter()
            .any(|row| row.path.contains("capture-cost")),
        "the census walked the store: {:?}",
        taken.rows.iter().map(|row| &row.path).collect::<Vec<_>>()
    );
}

/// The same claim about this repository rather than about a fixture, because
/// the store's placement is a fact about a committed declaration and a
/// constant, and a change to either one can break it.
#[test]
fn the_store_is_outside_the_corpus_root_this_repository_declares() {
    let consumer = std::fs::read_to_string(repository_root().join(".headwater/taxonomy.yml"))
        .expect("the committed consumer declaration");
    let root = headwater_yaml::load(&consumer)
        .expect("it reads")
        .value
        .as_map()
        .and_then(|map| map.get("corpus"))
        .and_then(|node| node.value.as_map())
        .and_then(|map| map.get("root"))
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.clone())
        .expect("the corpus root");
    assert!(
        !STORE.starts_with(&format!("{root}/")) && STORE != root,
        "`{STORE}` is under the corpus root `{root}`, so a language regime would bind it and \
         every rule written for prose would read it"
    );
}

fn a_reading(kind: &str, document: &str, id: Option<&str>) -> Reading {
    Reading {
        lock: "sha256:fixture".to_string(),
        date: Date::parse(PINNED).expect("a date"),
        surface: Some(Surface::Terminal),
        kind: kind.to_string(),
        document: document.to_string(),
        id: id.map(str::to_string),
        assisted: Assisted {
            fields: (4, 5),
            sections: (3, 3),
            identifier: match id.is_some() {
                true => (1, 1),
                false => (0, 0),
            },
            edge_halves: (0, 0),
        },
    }
}

fn corpus(documents: &[(&str, Option<&str>)]) -> Classified {
    Classified {
        paths: documents
            .iter()
            .map(|(path, _)| (*path).to_string())
            .collect(),
        identified: documents
            .iter()
            .filter_map(|(path, id)| id.map(|id| (id.to_string(), (*path).to_string())))
            .collect(),
    }
}

/// A document nobody scaffolded raises the denominator and never the numerator.
///
/// This is the whole of what makes a reach figure worth citing. A store that
/// absorbed such a document would report a reach it never had, and the number
/// would rise on exactly the event it exists to report.
#[test]
fn a_document_that_arrived_by_another_route_lowers_the_reach() {
    let readings = vec![a_reading(
        "decision_record",
        "corpus/decisions/0008-a-scaffolded-one.md",
        Some("DR-FIX-0008"),
    )];
    let scaffolded_only = corpus(&[(
        "corpus/decisions/0008-a-scaffolded-one.md",
        Some("DR-FIX-0008"),
    )]);
    let with_a_hand_written_one = corpus(&[
        (
            "corpus/decisions/0008-a-scaffolded-one.md",
            Some("DR-FIX-0008"),
        ),
        (
            "corpus/decisions/0009-typed-by-hand.md",
            Some("DR-FIX-0009"),
        ),
    ]);

    let before = reading::reach(&readings, &scaffolded_only);
    let after = reading::reach(&readings, &with_a_hand_written_one);
    assert_eq!(before.reached.len(), 1);
    assert_eq!(after.reached.len(), 1, "the numerator does not move");
    assert_eq!(scaffolded_only.paths.len(), 1);
    assert_eq!(
        with_a_hand_written_one.paths.len(),
        2,
        "the denominator does"
    );
}

/// A reading whose document is gone is named and never counted.
#[test]
fn a_reading_that_resolves_to_nothing_is_named_and_never_counted() {
    let readings = vec![
        a_reading(
            "decision_record",
            "corpus/decisions/0008-still-here.md",
            Some("DR-FIX-0008"),
        ),
        a_reading(
            "decision_record",
            "corpus/decisions/0009-deleted-since.md",
            Some("DR-FIX-0009"),
        ),
    ];
    let reach = reading::reach(
        &readings,
        &corpus(&[("corpus/decisions/0008-still-here.md", Some("DR-FIX-0008"))]),
    );
    assert_eq!(
        reach,
        Reach {
            reached: vec!["corpus/decisions/0008-still-here.md".to_string()],
            moved: vec![],
            lost: vec![1],
        }
    );
}

/// A rename after the reading joins on the identifier, and the report says the
/// document moved. A store keyed on a path alone would report this as a loss,
/// and the reach figure would fall for a rename.
#[test]
fn a_document_that_moved_after_its_reading_joins_on_its_identifier() {
    let readings = vec![a_reading(
        "decision_record",
        "corpus/decisions/0008-the-name-it-was-born-with.md",
        Some("DR-FIX-0008"),
    )];
    let reach = reading::reach(
        &readings,
        &corpus(&[(
            "corpus/decisions/0008-the-name-somebody-chose.md",
            Some("DR-FIX-0008"),
        )]),
    );
    assert_eq!(
        reach,
        Reach {
            reached: vec!["corpus/decisions/0008-the-name-somebody-chose.md".to_string()],
            moved: vec![(
                0,
                "corpus/decisions/0008-the-name-somebody-chose.md".to_string()
            )],
            lost: vec![],
        }
    );
}

/// A kind that mints no identifier joins on its path, and a rename of such a
/// document is a loss that the report names rather than hides.
#[test]
fn a_reading_with_no_identifier_joins_on_its_path_alone() {
    let readings = vec![a_reading(
        "evaluation",
        "corpus/evaluations/a-look.md",
        None,
    )];
    let held = reading::reach(
        &readings,
        &corpus(&[("corpus/evaluations/a-look.md", None)]),
    );
    assert_eq!(held.reached.len(), 1);
    assert_eq!(held.lost, Vec::<usize>::new());

    let renamed = reading::reach(
        &readings,
        &corpus(&[("corpus/evaluations/a-second-look.md", None)]),
    );
    assert_eq!(renamed.reached, Vec::<String>::new());
    assert_eq!(renamed.lost, vec![0]);
}

/// Two readings over one document are one document reached. `headwater new`
/// never overwrites, so this needs a rename and a second scaffold under the old
/// name — and a numerator that counted readings rather than documents would
/// report a reach above one for a corpus of one document.
#[test]
fn two_readings_over_one_document_are_one_document_reached() {
    let readings = vec![
        a_reading("evaluation", "corpus/evaluations/a-look.md", None),
        a_reading("evaluation", "corpus/evaluations/a-look.md", None),
    ];
    let reach = reading::reach(
        &readings,
        &corpus(&[("corpus/evaluations/a-look.md", None)]),
    );
    assert_eq!(reach.reached.len(), 1);
    assert_eq!(reach.lost, Vec::<usize>::new());
}
