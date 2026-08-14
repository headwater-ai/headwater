// SPDX-License-Identifier: Apache-2.0
//! The audit fixtures, and the properties an audit has to hold whatever the
//! corpus says.
//!
//! Two levels, at two grains, for the reason the read fixtures state. The tree
//! under `fixtures/` is written for these readings, so `audit.report` records
//! the whole report: nothing outside this crate can move those bytes, and the
//! clock is injected so that no run of it depends on the day.
//!
//! The run over this repository asserts properties instead. Its documents are
//! prose somebody edits and its taxonomy moves when a bundle does, so a
//! recorded report over it would be re-blessed most weeks. What is asserted
//! there is what no edit may break: that the creator reading accounts for every
//! edge the graph holds, that a creator with no relation is still named, and
//! that two audits of one tree at one date agree byte for byte.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-audit --test readings
//!
//! Read the diff before committing it. A blessed fixture is the change.

use headwater_audit::{Audit, Subject, CREATORS};
use headwater_census::census::{self, Census};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::context::Date;
use headwater_check::Shape;
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_yaml::Mapping;
use std::path::{Path, PathBuf};

/// The date the recorded report is taken at.
///
/// Every dwell figure and every staleness verdict in that file is a function of
/// this value and of the dates the fixture documents declare. A report taken
/// from the system clock would be a file that changed every night.
const AT: &str = "2026-01-01";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

fn compare(recorded: &Path, actual: &str) {
    if std::env::var_os("HEADWATER_BLESS").is_some() {
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
        "\nthe audit no longer matches {}",
        recorded.display()
    );
}

/// Everything the audit borrows, owned, so that a test may take one.
struct Built {
    census: Census,
    graph: Graph,
    shape: Shape,
    taxonomy: Taxonomy,
    relations: Declarations,
}

impl Built {
    fn over(corpus: &Corpus, root: &Mapping) -> Self {
        let taxonomy = Taxonomy::read(root).expect("the taxonomy reads");
        let relations = Declarations::read(root).expect("the declarations read");
        let shape = Shape::read(root).expect("the shape reads");
        let census = census::take(corpus, &taxonomy);
        let graph = Graph::build(
            &census,
            &relations,
            &Resolvers::over(corpus),
            corpus,
            &Config::default(),
        );
        Built {
            census,
            graph,
            shape,
            taxonomy,
            relations,
        }
    }

    fn audit(&self, now: &str) -> Audit {
        headwater_audit::take(
            Subject {
                package: "audit/fixture".to_string(),
                version: "1.0.0".to_string(),
                lock: "sha256:the-fixture".to_string(),
                now: Date::parse(now).expect("a date"),
            },
            &self.census,
            &self.graph,
            &self.taxonomy,
            &self.shape,
            &self.relations,
        )
    }
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

fn fixture_tree() -> Built {
    let corpus = Corpus::new(fixtures_dir(), "audit");
    let root = load_map(&fixtures_dir().join("audit.taxonomy.yml"));
    Built::over(&corpus, &root)
}

fn this_repository() -> Built {
    let root = repository_root();
    let lock = headwater_lock::at(&root).expect("the committed lock");
    let consumer = headwater_resolve::package::consumer(&root).expect("the consumer declaration");
    let corpus = Corpus::declared(&root, &consumer.corpus_root, &consumer.exclusions);
    Built::over(&corpus, &lock.taxonomy)
}

fn repository_audit(built: &Built, now: &str) -> Audit {
    built.audit(now)
}

#[test]
fn the_fixture_tree_produces_the_recorded_audit() {
    let built = fixture_tree();
    compare(
        &fixtures_dir().join("audit.report"),
        &built.audit(AT).render(),
    );
}

/// The one finding class exists, and the declaration is what produces it.
///
/// [Spec 12](../../../../docs/spec/12-check-layer.md) refuses a check with no
/// failing fixture under it, and the same bar reaches a reading that carries a
/// verdict. This is that fixture: on the recorded date two relations have a
/// half on a document past the declared window, and on a date inside the window
/// of every document neither does. A run that ignored `stale_after_days`, or
/// that compared against the wrong end of it, fails one of the two halves.
#[test]
fn a_finding_arrives_from_the_declared_window_and_leaves_when_the_date_moves() {
    let built = fixture_tree();

    let audit = built.audit(AT);
    let found: Vec<&str> = audit
        .findings()
        .iter()
        .map(|reading| reading.name.as_str())
        .collect();
    assert_eq!(found, ["catalogues", "supersedes"], "{}", audit.render());

    // 2025-08-01 is the earliest freshness date the tree declares, so on the
    // day after it every document is inside a 90-day window.
    let inside = built.audit("2025-08-02");
    assert!(
        inside.findings().is_empty(),
        "{}",
        inside.findings()[0].name
    );

    // And the window is read rather than assumed: a date far past every
    // freshness date puts every dated half over the line. Every relation that
    // has a dated half reaches the list, and `annotates` does not, because the
    // one half it has states no date at all.
    let outside = built.audit("2030-01-01");
    let found: Vec<&str> = outside
        .findings()
        .iter()
        .map(|reading| reading.name.as_str())
        .collect();
    assert_eq!(
        found,
        ["catalogues", "supersedes", "governs"],
        "{}",
        outside.render()
    );
}

/// A half whose document states no freshness date is counted apart, and never
/// as fresh.
///
/// The note of the fixture tree declares no `last_verified` at all, and the
/// facet is not required, so this is a legal state of a corpus rather than a
/// defect. A reading that treated it as inside the window would report a
/// verdict about a date nobody wrote.
#[test]
fn an_undated_document_is_counted_apart_from_a_fresh_one() {
    let built = fixture_tree();
    let audit = built.audit("2030-01-01");
    let annotates = audit
        .creators
        .by_creator
        .iter()
        .flat_map(|(_, readings)| readings)
        .find(|reading| reading.name == "annotates")
        .expect("the relation is declared");
    assert_eq!(annotates.halves, 1);
    assert_eq!(annotates.undated, 1);
    assert_eq!(annotates.expired, 0);
}

/// Every creator of spec 2's closed set has a row, in use or not.
///
/// This is the property the whole creator reading exists for. Q19 asks whether
/// an imported edge decays faster than a scaffolded one, and the answer over a
/// corpus that declares no `import` relation is that one arm is empty. A report
/// built from the values in use would print the arm that exists and say nothing
/// at all about the one that does not, which is the shape of a comparison that
/// looks complete and is not.
#[test]
fn every_declared_creator_has_a_row_and_the_absent_ones_are_named() {
    let built = fixture_tree();
    let audit = built.audit(AT);
    let rows: Vec<&str> = audit
        .creators
        .by_creator
        .iter()
        .map(|(creator, _)| creator.as_str())
        .collect();
    assert_eq!(rows, CREATORS);
    assert_eq!(audit.creators.absent(), ["generator", "import"]);

    let report = audit.render();
    assert!(
        report.contains("import — no relation declares it"),
        "{report}"
    );
    assert!(
        report.contains("generator — no relation declares it"),
        "{report}"
    );
}

/// The creator reading accounts for every edge half the graph holds.
///
/// Over this repository rather than over the tree written for it. The census
/// and the graph move on most weeks here, and this is the property that must
/// survive every one of those moves: a relation that produced halves and
/// reached no row would be a silent hole in the denominator of the one reading
/// two decisions depend on.
#[test]
fn the_creator_reading_accounts_for_every_half_of_this_corpus() {
    let built = this_repository();
    let audit = repository_audit(&built, "2026-01-01");
    let counted: usize = audit
        .creators
        .by_creator
        .iter()
        .flat_map(|(_, readings)| readings)
        .chain(audit.creators.undeclared.iter())
        .map(|reading| reading.halves)
        .sum();
    assert_eq!(counted, audit.halves);
    assert_eq!(audit.halves, built.graph.edges.len());
}

/// Two audits of one corpus at one date write one set of bytes.
///
/// [Spec 12](../../../../docs/spec/12-check-layer.md#determinism-concretely)
/// binds a run to "same corpus, same lock, same injected clock, byte-identical
/// output". Nothing gates on this verb, and it is held to that bar anyway,
/// because a report a person cites has to be reproducible by the person who
/// reads it. Two independent walks and two graphs, on purpose.
#[test]
fn two_audits_of_one_corpus_at_one_date_are_byte_identical() {
    let first = this_repository();
    let second = this_repository();
    assert_eq!(
        repository_audit(&first, "2026-01-01").render(),
        repository_audit(&second, "2026-01-01").render()
    );
}

/// A capture rate over an empty population is absent rather than zero.
///
/// The two read the same on a terminal and they are different facts. Zero
/// percent says every eligible document declined to declare the relation, and
/// no eligible document says the taxonomy put the relation somewhere no
/// document of this corpus can reach.
#[test]
fn a_rate_over_an_empty_population_is_no_rate_at_all() {
    let built = fixture_tree();
    let audit = built.audit(AT);
    let readings: Vec<_> = audit
        .creators
        .by_creator
        .iter()
        .flat_map(|(_, readings)| readings)
        .collect();
    for reading in readings {
        match reading.eligible {
            0 => assert!(reading.capture().is_none(), "{}", reading.name),
            _ => assert!(reading.capture().is_some(), "{}", reading.name),
        }
    }
}
