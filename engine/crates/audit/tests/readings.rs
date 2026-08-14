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

use headwater_audit::{Audit, Subject, Supply, Waiting, CREATORS, WARRANTS};
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
    /// The resolved taxonomy, for the one shelf member that no typed reader
    /// carries: `layout`.
    resolved: Mapping,
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
            resolved: root.clone(),
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
            &self.resolved,
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

/// An arm that cannot be measured has a location, and it is never a zero.
///
/// This is the defect that the figure in spec 3 carried before this reading
/// existed. That sentence read `56 of 142`, and the denominator held 16
/// documents of a kind that requires no facet in the `name` role. A rule of the
/// shape it was arguing about could not have measured one of them, so counting
/// them as documents that disagree reported a missing declaration as an
/// authoring failure. The three shelves below are the three answers a shelf can
/// produce, and a run that folded any of them into the others fails here.
#[test]
fn a_shelf_that_cannot_be_measured_is_located_rather_than_counted_at_zero() {
    let audit = fixture_tree().audit(AT);
    let reading = |shelf: &str| {
        audit
            .layouts
            .iter()
            .find(|reading| reading.shelf == shelf)
            .unwrap_or_else(|| panic!("no layout reading for `{shelf}`"))
    };

    // Measurable, and one of the three files disagrees. A reading that compared
    // the path rather than the file name, or that slugged something other than
    // the facet in the `name` role, moves this number.
    let decisions = reading("decisions");
    assert_eq!(
        (decisions.identified, decisions.measured, decisions.renders),
        (3, 3, 2)
    );
    assert!(matches!(decisions.adherence, Supply::Supplied(_)));

    // Not measurable, and the absence is in the schema. No kind of this shelf
    // requires the facet in the `name` role, so `{slug}` has no source and
    // neither document can be held to the layout.
    let library = reading("library");
    assert_eq!(
        (library.identified, library.measured, library.renders),
        (2, 0, 0)
    );
    assert!(
        matches!(library.adherence, Supply::Undeclared(_)),
        "an unmeasurable shelf reported as {}",
        library.adherence.located()
    );
    assert!(
        library.adherence.says().contains("`name` role"),
        "the reason names no declaration: {}",
        library.adherence.says()
    );

    // Declared and empty. The absence is the corpus's, and the row exists so
    // that a shelf nobody writes to is told apart from a shelf that drifted.
    let archive = reading("archive");
    assert_eq!(
        (archive.identified, archive.measured, archive.renders),
        (0, 0, 0)
    );
    assert!(
        matches!(archive.adherence, Supply::Unauthored(_)),
        "an empty shelf reported as {}",
        archive.adherence.located()
    );
}

/// Every shelf that declares a layout reports a population it could measure.
///
/// Over this repository rather than over the tree written for it, and it is the
/// arithmetic that spec 3 now cites: a document that was not measured is in no
/// numerator and in no denominator. A reading whose measured count exceeded its
/// identified count would be counting a document twice, and one whose renders
/// exceeded its measured count would be reporting a name it never rendered.
#[test]
fn no_layout_reading_counts_more_than_it_measured() {
    let audit = repository_audit(&this_repository(), AT);
    assert!(
        !audit.layouts.is_empty(),
        "this repository declares a shelf layout and the reading found none"
    );
    for reading in &audit.layouts {
        assert!(
            reading.renders <= reading.measured,
            "`{}` renders {} of {} measured",
            reading.shelf,
            reading.renders,
            reading.measured
        );
        assert!(
            reading.measured <= reading.identified,
            "`{}` measured {} of {} identified",
            reading.shelf,
            reading.measured,
            reading.identified
        );
        assert_eq!(
            reading.measured == 0 || reading.identified == 0,
            !reading.adherence.met(),
            "`{}` reports `{}` over {} measured",
            reading.shelf,
            reading.adherence.located(),
            reading.measured
        );
    }
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

// --- a wait that a corpus can end -------------------------------------------
//
// The three readings this verb names and does not take were three string
// literals, so the verb asserted facts about corpus content at compile time.
// One of them said that no document carried `warrant: asserted` and it went
// false while the corpus moved underneath it, and nothing reported that. The
// cases below are the property that replaces it.

/// A decision that states the warrant a promotion moves away from.
///
/// It is written into a scratch copy rather than committed, because the whole
/// claim under test is that the report changes when a corpus does. A committed
/// one would make both arms of the comparison the same tree.
const ASSERTED: &str = "\
---
id: AUD-FIX-0006
status: draft
status_since: 2025-12-22
summary: a decision that nobody has accepted, which is the population a promotion rate divides by
provenance:
  warrant: asserted
  agency: agent
  drafted_by: a-model
  activity: draft
---

# The unaccepted decision

Nothing here is accepted. It states the warrant that a promotion moves away
from, and it names no acceptor, which is what that warrant forbids.
";

/// A scratch copy of the fixture tree, which a test may write into.
fn copied(name: &str) -> PathBuf {
    let at = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&at);
    copy_into(&fixtures_dir().join("audit"), &at.join("audit"));
    at
}

fn copy_into(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a directory");
    for entry in std::fs::read_dir(from).expect("the fixture tree") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("a file type").is_dir() {
            true => copy_into(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("a file");
            }
        }
    }
}

fn tree_at(at: &Path) -> Built {
    let corpus = Corpus::new(at.to_path_buf(), "audit");
    let root = load_map(&fixtures_dir().join("audit.taxonomy.yml"));
    Built::over(&corpus, &root)
}

fn reading_of<'a>(audit: &'a Audit, reading: &str) -> &'a Waiting {
    audit
        .waiting
        .iter()
        .find(|waiting| waiting.reading == reading)
        .unwrap_or_else(|| panic!("`{reading}` is not a reading this verb names"))
}

fn supply_of<'a>(audit: &'a Audit, reading: &str, needs: &str) -> &'a Supply {
    &reading_of(audit, reading)
        .needs
        .iter()
        .find(|need| need.needs == needs)
        .unwrap_or_else(|| panic!("`{reading}` does not need `{needs}`"))
        .supply
}

/// A wait ends when the corpus supplies what it waits on, and no source moves.
///
/// One document lands in a scratch copy of the fixture tree and nothing else
/// changes: not this engine, not the taxonomy, and not the recorded report.
/// Before it the promotion reading says that no document states the warrant,
/// and after it the same run says how many do. Removing the document brings the
/// wait back, so the reading is a function of the corpus in both directions.
///
/// This is the case that the array of string literals could not have passed.
#[test]
fn a_wait_ends_when_the_corpus_supplies_what_it_waits_on() {
    const POPULATION: &str = "an `asserted` population to divide by";
    let at = copied("asserted-population");
    let landing = at.join("audit/decisions/unaccepted.md");

    let before = tree_at(&at).audit(AT);
    assert!(
        matches!(
            supply_of(&before, "promotion rate", POPULATION),
            Supply::Unauthored(_)
        ),
        "the fixture tree already has an asserted population, so this proves nothing: {}",
        supply_of(&before, "promotion rate", POPULATION).says()
    );
    // Deliberately loose. The decisive assertion is the one after the document
    // lands, and a strict comparison here would fail first on wording and
    // report a rewording where the defect is a wait that never ends.
    assert!(
        supply_of(&before, "promotion rate", POPULATION)
            .says()
            .contains("`warrant: asserted`"),
        "the wait does not name what would end it"
    );
    let report = before.render();
    assert!(
        report.contains("The `asserted` population is 0"),
        "{report}"
    );

    std::fs::write(&landing, ASSERTED).expect("the document lands");

    let after = tree_at(&at).audit(AT);
    let supplied = supply_of(&after, "promotion rate", POPULATION);
    assert!(
        matches!(supplied, Supply::Supplied(_)),
        "a corpus with an asserted population still reports one it does not have: {}",
        supplied.says()
    );
    assert_eq!(
        supplied.says(),
        "1 of 6 classified documents state `warrant: asserted`"
    );
    let report = after.render();
    assert!(!report.contains("nothing to promote from"), "{report}");
    assert!(
        report.contains("The `asserted` population is 1"),
        "{report}"
    );
    assert!(
        report.contains("      supplied — 1 of 6 classified"),
        "{report}"
    );

    // The reading still waits, and on the other half of what it needs. A change
    // that made every wait disappear would pass the assertions above and be
    // worse than the array it replaced.
    assert!(
        reading_of(&after, "promotion rate").waits(),
        "the numerator arrived from nowhere"
    );
    let numerator = supply_of(&after, "promotion rate", "a promotion to count");
    assert!(
        matches!(numerator, Supply::Unbuilt(_)),
        "{}",
        numerator.says()
    );
    assert!(
        numerator.says().contains("needs_prior"),
        "the wait does not name the input that ends it: {}",
        numerator.says()
    );

    std::fs::remove_file(&landing).expect("the document leaves");
    let restored = tree_at(&at).audit(AT);
    assert!(
        matches!(
            supply_of(&restored, "promotion rate", POPULATION),
            Supply::Unauthored(_)
        ),
        "the wait did not come back when the population left"
    );
}

/// A reading that still waits says so, and says where the absence lives.
///
/// The other direction of the same property. Two of the three readings wait on
/// this fixture corpus for two different reasons, and the difference is what a
/// reader acts on: a role that no registry declares closes with a declaration,
/// and a cue that nobody wrote closes with authoring. A report that flattened
/// the two into "waits" would be no better than the string it replaced.
#[test]
fn a_reading_that_still_waits_says_where_the_absence_lives() {
    let audit = fixture_tree().audit(AT);

    let state = supply_of(
        &audit,
        "transition continuity",
        "a record of the state a document left",
    );
    assert!(matches!(state, Supply::Undeclared(_)), "{}", state.says());
    assert!(state.says().contains("state_left"), "{}", state.says());

    let cue = supply_of(
        &audit,
        "scent quality",
        "a cue authored on an edge instance",
    );
    assert!(matches!(cue, Supply::Unauthored(_)), "{}", cue.says());
    assert!(
        cue.says().contains("6"),
        "the population is named: {}",
        cue.says()
    );

    let report = audit.render();
    assert!(report.contains("nothing declares it —"), "{report}");
    assert!(report.contains("nothing authored one —"), "{report}");
    assert!(report.contains("designed and unbuilt —"), "{report}");
    assert!(report.contains("3 of 3 still wait"), "{report}");
}

/// The closed warrant set is walked in full, whatever a corpus states.
///
/// The same property the creator reading holds, one layer out. A promotion rate
/// over the values in use would report `accepted` and `asserted` on a corpus
/// that has both and say nothing at all about a corpus that has neither, which
/// is the shape of a comparison that looks complete and is not.
#[test]
fn every_warrant_of_the_closed_set_has_a_row_and_a_value_outside_it_is_reported() {
    let audit = fixture_tree().audit(AT);
    let rows: Vec<&str> = audit
        .warrants
        .readings
        .iter()
        .filter(|reading| reading.closed_set)
        .map(|reading| reading.value.as_str())
        .collect();
    assert_eq!(rows, WARRANTS);

    let outside = audit.warrants.outside();
    assert_eq!(outside.len(), 1);
    assert_eq!(outside[0].value, "pending");
    assert_eq!(outside[0].stated, 1);

    // Every classified document is either in a row or in the unstated count,
    // so no document falls out of this reading in silence.
    assert_eq!(
        audit.warrants.stated() + audit.warrants.unstated,
        audit.documents
    );

    let report = audit.render();
    assert!(report.contains("`pending`"), "{report}");
    assert!(
        report.contains("No\n  check of this engine reads a warrant"),
        "{report}"
    );
}
