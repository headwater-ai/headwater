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

use headwater_audit::reading::{Reading, TaskReading};
use headwater_audit::{Audit, Series, Subject, Supply, Waiting, CREATORS, DERIVED, WARRANTS};
use headwater_census::census::{self, Census};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::adoption::State;
use headwater_check::context::Date;
use headwater_check::paint::{paint, ColorMode, Role};
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
        self.audit_over(now, Series::none)
    }

    /// The same audit, with the adoption series the case chooses.
    ///
    /// The series is a parameter of `take` rather than something it reads, so a
    /// case states one the way it states a corpus. `Series::none` is the state
    /// of a corpus whose store nobody has opened, which is what the recorded
    /// report holds.
    fn audit_over(&self, now: &str, series: impl Fn(&str, Date) -> Series) -> Audit {
        let lock = "sha256:the-fixture";
        let date = Date::parse(now).expect("a date");
        headwater_audit::take(
            Subject {
                package: "audit/fixture".to_string(),
                version: "1.0.0".to_string(),
                lock: lock.to_string(),
                now: date,
            },
            &self.census,
            &self.graph,
            &self.taxonomy,
            &self.shape,
            &self.relations,
            series(lock, date),
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
        &built.audit(AT).render(ColorMode::Plain),
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
    assert_eq!(
        found,
        ["catalogues", "supersedes"],
        "{}",
        audit.render(ColorMode::Plain)
    );

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
        outside.render(ColorMode::Plain)
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

    let report = audit.render(ColorMode::Plain);
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
        repository_audit(&first, "2026-01-01").render(ColorMode::Plain),
        repository_audit(&second, "2026-01-01").render(ColorMode::Plain)
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
///
/// It declares one half of `catalogues` with a `cue` attribute on it. Q20 puts
/// the cue on the relation instance, so this is the one authoring act that ends
/// the scent-quality wait, and no source of this engine moves when it lands.
const CUED: &str = "\
---
id: AUD-FIX-0006
title: The cued decision
status: draft
status_since: 2025-12-22
summary: a decision that declares a half carrying the cue attribute the scent reading waits on
provenance:
  warrant: asserted
  agency: agent
  drafted_by: a-model
  activity: draft
relations:
  catalogues:
    - to: AUD-FIX-0001
      cue: the freshness window, and not the succession pair
---

# The cued decision

It carries the one attribute that no document of the recorded tree carries.
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
/// Before it the scent reading says that no half carries a cue, and after it
/// the same run says how many do. Removing the document brings the wait back,
/// so the reading is a function of the corpus in both directions.
///
/// This is the case that the array of string literals could not have passed. It
/// stood on the promotion reading until `warrant.promoted` was built, and that
/// reading left this verb: a rate whose numerator is change-scoped is not a
/// wait of a verb that reads one working tree.
#[test]
fn a_wait_ends_when_the_corpus_supplies_what_it_waits_on() {
    const CUE: &str = "a cue authored on an edge instance";
    let at = copied("authored-cue");
    let landing = at.join("audit/decisions/cued.md");

    let before = tree_at(&at).audit(AT);
    assert!(
        matches!(
            supply_of(&before, "scent quality", CUE),
            Supply::Unauthored(_)
        ),
        "the fixture tree already carries a cue, so this proves nothing: {}",
        supply_of(&before, "scent quality", CUE).says()
    );
    // Deliberately loose. The decisive assertion is the one after the document
    // lands, and a strict comparison here would fail first on wording and
    // report a rewording where the defect is a wait that never ends.
    assert!(
        supply_of(&before, "scent quality", CUE)
            .says()
            .contains("`cue` attribute"),
        "the wait does not name what would end it"
    );

    std::fs::write(&landing, CUED).expect("the document lands");

    let after = tree_at(&at).audit(AT);
    let supplied = supply_of(&after, "scent quality", CUE);
    assert!(
        matches!(supplied, Supply::Supplied(_)),
        "a corpus with an authored cue still reports one it does not have: {}",
        supplied.says()
    );
    assert_eq!(supplied.says(), "1 of 7 halves carry a `cue` attribute");
    let report = after.render(ColorMode::Plain);
    assert!(!report.contains("no half of the"), "{report}");
    assert!(
        report.contains("      supplied — 1 of 7 halves"),
        "{report}"
    );

    // The other reading still waits, and on a declaration rather than on an
    // authoring pass. A change that made every wait disappear would pass the
    // assertions above and be worse than the array it replaced.
    assert!(
        reading_of(&after, "transition continuity").waits(),
        "a facet role arrived from nowhere"
    );

    std::fs::remove_file(&landing).expect("the document leaves");
    let restored = tree_at(&at).audit(AT);
    assert!(
        matches!(
            supply_of(&restored, "scent quality", CUE),
            Supply::Unauthored(_)
        ),
        "the wait did not come back when the cue left"
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
        cue.says().contains('6'),
        "the population is named: {}",
        cue.says()
    );

    let report = audit.render(ColorMode::Plain);
    assert!(report.contains("nothing declares it —"), "{report}");
    assert!(report.contains("nothing authored one —"), "{report}");
    assert!(report.contains("2 of 2 still wait"), "{report}");
}

/// The warrant the engine derives is the warrant this report calls derived.
///
/// `warrant.evidence.unsupported` decides a pointer onto a generated document
/// on the value `headwater_doc::warrant_of` answers, and this report tells a
/// reader that a zero on that row is the engine's doing rather than the
/// authoring's. Two statements about one value, in two crates, and this is
/// where they are held to being one value. A rule that derived a word this
/// report does not name as derived would send a reader to the wrong row with
/// no test anywhere reporting it.
///
/// It also holds the derived value inside spec 3's closed set. A value outside
/// it is reported as one a corpus invented, and the engine's own answer must
/// not be one of those.
#[test]
fn the_warrant_the_engine_derives_is_named_derived_and_is_inside_the_closed_set() {
    // The reader borrows out of the front matter it was handed, so the empty
    // block outlives the answer rather than being a temporary in the call.
    let declares_nothing = headwater_yaml::Mapping::default();
    let derived = headwater_doc::warrant_of(&declares_nothing, true)
        .expect("a generated document has a warrant");
    assert!(
        DERIVED.contains(&derived),
        "`{derived}` is derived by the engine and this report does not name it: {DERIVED:?}"
    );
    assert!(
        WARRANTS.contains(&derived),
        "`{derived}` is outside spec 3's closed set: {WARRANTS:?}"
    );

    // And the same reader still answers the declared half for a document this
    // engine did not write, which is what keeps the two absences apart.
    assert_eq!(headwater_doc::warrant_of(&declares_nothing, false), None);
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

    let report = audit.render(ColorMode::Plain);
    assert!(report.contains("`pending`"), "{report}");
    assert!(
        report.contains("No\n  check of this engine reads a warrant"),
        "{report}"
    );
}

/// One reading of a task, for a case that states a series rather than a corpus.
fn a_task(id: &str, until: &str, open: usize, closed: usize, held: usize) -> TaskReading {
    TaskReading {
        id: id.to_string(),
        until: Date::parse(until).expect("a date"),
        state: State::Open,
        open,
        closed,
        held,
    }
}

fn a_reading(lock: &str, date: &str, tasks: Vec<TaskReading>) -> Reading {
    Reading {
        lock: lock.to_string(),
        date: Date::parse(date).expect("a date"),
        refused: 0,
        tasks,
    }
}

/// A store nobody has opened states that there is no series, and it is not the
/// same state as a corpus that declares no payload.
///
/// Two absences, and a report that printed one line for both would make
/// "nobody has recorded anything" and "the payload is gone" one sentence.
#[test]
fn an_empty_store_and_an_empty_payload_are_two_different_sentences() {
    let built = fixture_tree();
    let text = built.audit(AT).render(ColorMode::Plain);
    assert!(
        text.contains("no reading recorded"),
        "the store is empty and the section says so:\n{text}"
    );
    assert!(
        text.contains("declares no adoption payload"),
        "and the corpus declares none, which is the other absence:\n{text}"
    );
    assert!(
        text.contains("the fraction that reaches zero before its expiry has no"),
        "so the fraction has no population to divide by:\n{text}"
    );
}

/// The decisive property at crate grain: an open payload and a discharged one
/// render as two different sections.
///
/// The pair count is the thing that decays, so a section that read the same
/// over both would be an instrument that measures nothing. This is the same
/// separation `engine/crates/cli/tests/adoption.rs` makes end to end, at the
/// grain where the two states are one field apart.
#[test]
fn an_open_payload_and_a_discharged_one_render_differently() {
    let built = fixture_tree();
    let open = built
        .audit_over(AT, |lock, date| Series {
            reading: a_reading(
                lock,
                &date.render(),
                vec![a_task("AD-1", "2027-06-30", 4, 0, 6)],
            ),
            recorded: vec![],
            unreadable: vec![],
        })
        .render(ColorMode::Plain);
    let zero = built
        .audit_over(AT, |lock, date| Series {
            reading: a_reading(
                lock,
                &date.render(),
                vec![a_task("AD-1", "2027-06-30", 0, 4, 0)],
            ),
            recorded: vec![],
            unreadable: vec![],
        })
        .render(ColorMode::Plain);
    assert_ne!(open, zero, "two payloads, two readings");
    assert!(
        open.contains("4 pairs open") && open.contains("open 4, closed 0"),
        "the open payload states its pairs:\n{open}"
    );
    assert!(
        zero.contains("0 pairs open") && zero.contains("open 0, closed 4"),
        "and the discharged one states that it shrank:\n{zero}"
    );
}

/// Two lock digests are two denominators, and the report refuses to trend
/// across them.
///
/// `headwater capture` refuses the same average for the same reason: a
/// denominator made of declarations moves when the taxonomy moves. Here the
/// denominator is a pair set, and a rule the taxonomy stopped running closes a
/// pair with no change in what anybody wrote.
#[test]
fn two_digests_are_named_and_never_trended_across() {
    let built = fixture_tree();
    let text = built
        .audit_over(AT, |lock, date| Series {
            reading: a_reading(
                lock,
                &date.render(),
                vec![a_task("AD-1", "2027-06-30", 1, 3, 1)],
            ),
            recorded: vec![
                a_reading(
                    "sha256:one",
                    "2026-01-01",
                    vec![a_task("AD-1", "2027-06-30", 4, 0, 6)],
                ),
                a_reading(
                    "sha256:two",
                    "2026-06-01",
                    vec![a_task("AD-1", "2027-06-30", 1, 3, 1)],
                ),
            ],
            unreadable: vec![],
        })
        .render(ColorMode::Plain);
    assert!(
        text.contains("2 taxonomies produced these readings"),
        "the section counts the denominators:\n{text}"
    );
    assert!(
        text.contains("are not a trend"),
        "and refuses to trend across them:\n{text}"
    );
    assert!(
        text.contains("sha256:one") && text.contains("sha256:two"),
        "naming each one:\n{text}"
    );
    assert!(
        text.contains("open 4 on the first of 2 readings that hold it, and 1 on the last"),
        "and the per-task line states both ends rather than an average:\n{text}"
    );
}

/// A line the store cannot read is named by its line number and never counted.
#[test]
fn an_unreadable_line_is_named_and_counted_nowhere() {
    let built = fixture_tree();
    let text = built
        .audit_over(AT, |lock, date| Series {
            reading: a_reading(lock, &date.render(), vec![]),
            recorded: vec![a_reading(
                "sha256:one",
                "2026-01-01",
                vec![a_task("AD-1", "2027-06-30", 0, 4, 0)],
            )],
            unreadable: vec![headwater_audit::reading::Unreadable {
                line: 2,
                why: "it names no `tasks`".to_string(),
            }],
        })
        .render(ColorMode::Plain);
    assert!(
        text.contains("line 2 of the store is not a reading and is counted nowhere"),
        "the line is named:\n{text}"
    );
    assert!(
        text.contains("1 reading, 2026-01-01 to 2026-01-01"),
        "and the readable one is the whole population:\n{text}"
    );
    assert!(
        text.contains("payloads at zero before their expiry: 1 of 1 task"),
        "which is what the fraction divides by:\n{text}"
    );
}

/// An empty task list has two causes, and the section never states the one it
/// did not read.
///
/// This is the render half of the CLI arm
/// `a_refused_task_is_counted_and_the_report_names_the_refusal_as_the_cause`,
/// at the grain where the two states are one field apart. A report that stated
/// "the lock declares no adoption payload" over a lock that declares one and
/// could not read it is the wrong diagnosis for the adopter who has to go and
/// fix the lock.
#[test]
fn an_empty_task_list_names_the_cause_it_read() {
    let built = fixture_tree();
    let refused = built
        .audit_over(AT, |lock, date| Series {
            reading: Reading {
                lock: lock.to_string(),
                date,
                refused: 1,
                tasks: vec![],
            },
            recorded: vec![],
            unreadable: vec![],
        })
        .render(ColorMode::Plain);
    let undeclared = built.audit(AT).render(ColorMode::Plain);

    assert!(
        refused.contains("the cause is a refusal rather than an"),
        "a refused payload names the refusal:\n{refused}"
    );
    assert!(
        !refused.contains("because the lock declares no adoption payload"),
        "and never the absence it did not read:\n{refused}"
    );
    assert!(
        refused.contains("it could not read 1 task"),
        "with the count beside it:\n{refused}"
    );
    assert!(
        undeclared.contains("because the lock declares no adoption payload"),
        "and the other cause still states itself:\n{undeclared}"
    );
    assert!(
        !undeclared.contains("it could not read"),
        "with no refusal line, because nothing was refused:\n{undeclared}"
    );
}

/// Every SGR sequence in a string, taken back off it.
///
/// The report is composed, folded and then painted, so the painted report and
/// the plain report have to be one string with the color added and nothing
/// else moved. This is what says so.
fn stripped(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('\u{1b}') {
        out.push_str(&rest[..start]);
        match rest[start..].find('m') {
            Some(end) => rest = &rest[start + end + 1..],
            None => {
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

/// The colored audit strips to the plain audit, byte for byte.
///
/// `render` folds two blocks through [`headwater_check::filled`], which
/// measures a line in characters. An SGR sequence is characters that occupy no
/// column, so a layout name painted before the fold would spend nine of the
/// line's eighty characters on bytes a terminal never shows and every break
/// after it would land early. Both reports are internally consistent and both
/// are eighty columns wide, so nothing but this comparison can see it.
///
/// `this_repository` rather than the fixture tree, because the fold this
/// protects is the layouts block and the shelves of this corpus are what
/// declare a layout with a name long enough to reach a break.
#[test]
fn the_colored_audit_strips_to_the_plain_audit() {
    let built = this_repository();
    let ansi = repository_audit(&built, AT).render(ColorMode::Ansi);
    let plain = repository_audit(&built, AT).render(ColorMode::Plain);
    assert!(
        ansi.contains('\u{1b}'),
        "the colored report has to carry color, or this comparison holds nothing"
    );
    assert_eq!(
        stripped(&ansi),
        plain,
        "a break moved when the color arrived, so something painted before it folded"
    );
}

/// The roles `Audit::render` declares, enumerated from the renderer.
///
/// Three `paint(Role::` families reach this report: `Heading` at eleven
/// positions, `Path` at two, and `Obligation` at one. This array is the set,
/// and the cases below iterate it rather than a list somebody typed beside it.
/// A fourth role wired into the renderer and not added here fails
/// `the_audit_paints_no_role_this_enumeration_omits`.
const AUDIT_ROLES: [Role; 3] = [Role::Heading, Role::Path, Role::Obligation];

/// Every heading literal the renderer writes, in the order it writes them.
///
/// `findings` is written from one of two arms and both write the same literal,
/// so the count is one either way.
const AUDIT_HEADINGS: [&str; 11] = [
    "taxonomy audit of",
    "findings",
    "relations, by the creator each one declares",
    "relation families",
    "facets, and what each one separates",
    "shelves that hold several kinds",
    "file names, against the layout each shelf declares",
    "dwell in the current state",
    "warrants, over the closed set spec 3 declares",
    "adoption payload decay",
    "what this verb does not measure, and what each one waits on",
];

/// This corpus, with one adoption reading handed to it, so that every role the
/// renderer declares has a non-empty population in one report.
///
/// `this_repository().audit(AT)` is handed an empty series: the adoption store
/// is outside the corpus root and no case of this file reads one. So
/// `Role::Obligation` is reached zero times there, a count case over it would
/// hold `0 == 0`, and a position case would have nothing to find. **That
/// absence is why nothing asserted this report's only magenta until now**, and
/// a helper that supplies the population is what closes it rather than a
/// weaker assertion.
fn an_audit_reaching_every_role(built: &Built) -> Audit {
    built.audit_over(AT, |lock, date| Series {
        reading: a_reading(
            lock,
            &date.render(),
            vec![a_task("AD-1", "2027-06-30", 0, 1, 0)],
        ),
        recorded: vec![],
        unreadable: vec![],
    })
}

/// The opening SGR sequence a role writes, with no text and no reset.
///
/// Derived from `paint` rather than written as a literal, so a palette change
/// moves one place and every case here follows it.
fn opening(role: Role) -> String {
    let painted = paint(role, "x", ColorMode::Ansi);
    painted
        .strip_suffix("x\u{1b}[0m")
        .expect("paint wraps its text and closes with a reset")
        .to_string()
}

/// How many times each declared role is painted, against a count this run
/// derives from the audit rather than from the report.
///
/// This is the count half of the bar. A position case says a role reached the
/// one place only this surface puts it. This says it reached every one of them,
/// so setting a single call site to `Plain` is a failure rather than a quieter
/// report. Every expected count is asserted non-zero first, because a role
/// whose population is empty would otherwise be held by `0 == 0`.
#[test]
fn every_declared_role_of_the_audit_is_painted_the_number_of_times_it_is_reached() {
    let built = this_repository();
    let audit = an_audit_reaching_every_role(&built);
    let ansi = audit.render(ColorMode::Ansi);
    for role in AUDIT_ROLES {
        let wanted = match role {
            // One per literal above.
            Role::Heading => AUDIT_HEADINGS.len(),
            // The shelf line and the folded prose under it, per reading.
            Role::Path => 2 * audit.layouts.len(),
            // One per adoption task the lock declares.
            Role::Obligation => audit.adoption.reading.tasks.len(),
            other => panic!("{other:?} is in AUDIT_ROLES with no expected count"),
        };
        assert!(
            wanted > 0,
            "{role:?} has an empty population on this corpus, so its count holds nothing"
        );
        let got = ansi.matches(&opening(role)).count();
        assert_eq!(
            got, wanted,
            "{role:?} is painted {got} times and this run reaches it {wanted} times"
        );
    }
}

/// No role outside the enumeration is painted, so the enumeration is the set.
///
/// Without this, adding a fourth `paint(Role::` to the renderer and no case for
/// it leaves the coverage claim above false and nothing says so.
#[test]
fn the_audit_paints_no_role_this_enumeration_omits() {
    let built = this_repository();
    let ansi = an_audit_reaching_every_role(&built).render(ColorMode::Ansi);
    for role in [
        Role::Error,
        Role::Warn,
        Role::Info,
        Role::Path,
        Role::Verb,
        Role::Obligation,
        Role::Heading,
    ] {
        let painted = ansi.contains(&opening(role));
        // `Role` derives no `PartialEq`, and every opening sequence is distinct,
        // so the enumeration is searched by what each member writes.
        let declared = AUDIT_ROLES
            .iter()
            .any(|member| opening(*member) == opening(role));
        assert_eq!(
            painted, declared,
            "{role:?} is painted={painted} and enumerated={declared}. Add it to \
             AUDIT_ROLES with a position case and a count, or stop painting it"
        );
    }
}

/// `Role::Heading`, in the position only this surface writes it.
///
/// A whole painted literal per heading. A bare `\x1b[1m` would be satisfied by
/// any one of the eleven, which is the shape that let a deleted paint pass.
#[test]
fn every_audit_heading_is_painted_whole() {
    let built = this_repository();
    let ansi = repository_audit(&built, AT).render(ColorMode::Ansi);
    for heading in AUDIT_HEADINGS {
        let wanted = paint(Role::Heading, heading, ColorMode::Ansi);
        assert!(
            ansi.contains(&wanted),
            "the heading {heading:?} is not painted in the colored report"
        );
    }
}

/// `Role::Obligation`, in the position only the adoption section writes it.
///
/// This report's only magenta, and until this case nothing asserted it at all:
/// setting `render.rs`'s `paint(Role::Obligation, …)` to `Plain` took the
/// magenta from one occurrence to none with every suite green. The identifier
/// is followed by two spaces and `open `, which no other line of this report
/// writes, so the assertion cannot be satisfied by a sibling caller.
#[test]
fn the_adoption_task_identifier_is_painted_where_the_task_line_puts_it() {
    let built = this_repository();
    let audit = an_audit_reaching_every_role(&built);
    let ansi = audit.render(ColorMode::Ansi);
    let tasks = &audit.adoption.reading.tasks;
    assert!(
        !tasks.is_empty(),
        "the lock declares no adoption task, so this case asserts nothing"
    );
    for task in tasks {
        let wanted = format!(
            "    {}  open {}, closed {}",
            paint(Role::Obligation, &task.id, ColorMode::Ansi),
            task.open,
            task.closed
        );
        assert!(
            ansi.contains(&wanted),
            "the task {} is not painted on its own line:\n{wanted:?}",
            task.id
        );
    }
}

/// The layout name is painted **inside the prose that was folded**, and not
/// only on the unfolded line above it.
///
/// Two lines of the layouts section carry the layout name. The shelf line is
/// composed painted, and the line under it is composed plain, folded through
/// [`headwater_check::filled`], and painted afterwards by `painted_in_place`.
/// Deleting the second of those two paints leaves the first one standing, so
/// every assertion that asks only whether the report contains a cyan sequence
/// stays green while half the path color of this report disappears. That
/// regression was run: the audit target passed 20 of 20 and
/// `tools/color-fixtures.sh` passed 65 of 65 with the in-fold paint removed.
///
/// `strips_to_the_plain_audit` cannot see it either, because less color still
/// strips to the plain bytes. This case is the one that can: it asks for the
/// painted layout name in the position only the folded prose puts it in, which
/// is after the word `located()` prints and inside the backticks `says()`
/// writes.
#[test]
fn the_layout_name_is_painted_inside_the_folded_prose() {
    let built = this_repository();
    let audit = repository_audit(&built, AT);
    let ansi = audit.render(ColorMode::Ansi);
    assert!(
        !audit.layouts.is_empty(),
        "no shelf of this corpus declares a layout, so this case asserts nothing"
    );
    for reading in &audit.layouts {
        let wanted = format!(
            "{} — `{}`",
            reading.adherence.located(),
            paint(Role::Path, &reading.layout, ColorMode::Ansi)
        );
        assert!(
            ansi.contains(&wanted),
            "the layout of `{}` is not painted inside the folded prose. \
             Something paints it on the shelf line alone, or the fold and the \
             paint ran in the wrong order:\n{wanted:?}",
            reading.shelf
        );
    }
}

/// The two lines that carry a layout name are painted the same number of times.
///
/// The case above holds one position. This holds the count, so a change that
/// keeps the folded paint and drops the shelf-line paint is also seen. The
/// corpus declares several shelves that share one layout string, so the count
/// is taken per line rather than per token.
#[test]
fn both_lines_of_a_layout_reading_paint_the_name() {
    let built = this_repository();
    let audit = repository_audit(&built, AT);
    let ansi = audit.render(ColorMode::Ansi);
    let cyan = "\u{1b}[36m";
    // The shelf line is indented two spaces and the folded prose six.
    let shelf_lines = ansi
        .lines()
        .filter(|line| line.starts_with("  ") && !line.starts_with("   ") && line.contains(cyan))
        .count();
    let folded_lines = ansi
        .lines()
        .filter(|line| line.starts_with("      ") && line.contains(cyan))
        .count();
    assert_eq!(
        shelf_lines,
        audit.layouts.len(),
        "one shelf line per layout reading carries the painted name"
    );
    assert_eq!(
        folded_lines,
        audit.layouts.len(),
        "and one folded line per layout reading carries it too"
    );
}

/// The plain audit carries no escape byte at all.
#[test]
fn the_plain_audit_writes_no_escape_byte() {
    let built = this_repository();
    let plain = repository_audit(&built, AT).render(ColorMode::Plain);
    assert!(
        !plain.contains('\u{1b}'),
        "the plain report has to be plain"
    );
}
