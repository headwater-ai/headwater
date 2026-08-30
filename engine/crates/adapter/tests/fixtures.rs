// SPDX-License-Identifier: Apache-2.0
//! The adapters, held to a corpus that carries all three classes of finding.
//!
//! # The corpus is the check crate's, and copying it would have been worse
//!
//! `engine/crates/check/fixtures/check` already breaks enough rules to produce
//! findings at two severities, and it carries a directive that hides one. This
//! suite reads that tree rather than growing a second one beside it. A second
//! corpus would be a second thing to keep true, and the first symptom of its
//! drifting would be an adapter test passing over a tree the runner no longer
//! agrees with.
//!
//! The third class is injected here. An adoption payload over one
//! `(document, rule)` pair the tree already fails gives the run a
//! `migration-pending` finding, which is the same technique the runner's own
//! precedence test uses. [`the_run_carries_all_three_classes`] is what stops
//! this file from becoming a comparison of one empty set with another.
//!
//! # What pins the reading of SARIF
//!
//! Nothing in this repository. `tests/sarif-schema-2.1.0.json` is the OASIS
//! Standard schema, errata 01, and
//! [`the_vendored_schema_is_the_published_copy`] records its digest so that a
//! reader can hold the file against the published one without trusting a
//! comment. `tests/sarif_oracle.py` beside it validates with the `jsonschema`
//! package and imports nothing this project wrote.
//!
//! The oracle does a second thing that validation cannot.
//! [`the_stock_reader_finds_the_same_findings_the_run_did`] holds the finding
//! set it reads back out of the document against the finding set the run
//! started from, in both directions. A dropped finding and an invented one are
//! both valid SARIF, so no schema catches either.
//!
//! Absent Python, the two tests that need it report the skip and the rest of
//! the suite runs, because the pinned build container carries no Python. Set
//! `HEADWATER_SARIF_VALIDATOR` and the skip becomes a failure. Continuous
//! integration sets it.
//!
//! # Blessing
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-adapter --test fixtures
//!
//! Read the diff before committing it. A blessed record is the change.

use headwater_adapter::{
    every_run, reported, Carrier, Escape, Format, Loss, OnFinding, OnRun, Place, Subject,
};
use headwater_census::census::{self, Census};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::change::{Change, Unbound, FORMAT};
use headwater_check::{Cache, Context, Date, Declared, Register, Run, Severity, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_yaml::Mapping;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The clock the runner's own fixtures are pinned at. One date, so a
/// participation window decides the same way in both suites.
const PINNED: &str = "2026-08-12";

/// The published digest of the vendored schema.
///
/// Both of these URLs serve these bytes, and they are the same bytes:
///
/// - <https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json>
/// - <https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json>
const SCHEMA_DIGEST: &str = "c3b4bb2d6093897483348925aaa73af03b3e3f4bd4ca38cef26dcb4212a2682e";

/// The check crate's fixture tree, from here.
fn check_fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../check/fixtures")
        .canonicalize()
        .expect("the check crate's fixtures")
}

fn tests_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests")
}

fn load_map(path: &Path) -> Mapping {
    let source =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    headwater_yaml::load(&source)
        .unwrap_or_else(|errors| panic!("{}: {errors:?}", path.display()))
        .value
        .as_map()
        .unwrap_or_else(|| panic!("{} is not a mapping", path.display()))
        .clone()
}

fn lock_digest() -> String {
    let source = std::fs::read_to_string(check_fixtures().join("check.taxonomy.yml"))
        .expect("the fixture taxonomy");
    headwater_hash::hex(source.as_bytes())
}

/// One run of the fixture tree, with the two structures it was held against.
///
/// The census and the graph are here because the text format writes both, and
/// `headwater_adapter::render` therefore takes both. A test that rebuilt them
/// beside the run would be holding one artifact against a second walk.
struct Ran {
    run: Run,
    census: Census,
    graph: Graph,
}

/// One run over the fixture tree, with a payload that holds one pair.
///
/// The pair is read off a run of the same tree rather than written here, so
/// this follows the fixture tree when its content changes instead of pinning a
/// path and a rule that a later edit moves.
fn fixture_run() -> Ran {
    ran(Scoping::Corpus)
}

/// The same tree and the same payload, over one change.
///
/// Every recorded artifact of this crate was a full-corpus run until #230, so
/// no fixture here would have moved if the block that states the scoping were
/// added and then removed again. These two are that fixture.
fn scoped_run() -> Ran {
    ran(Scoping::Change)
}

/// The same tree, over a change that named no document at all.
///
/// Not the corpus. #178 opens by naming this pair: a count over a change that
/// was never described is zero, and so is a count over a change that genuinely
/// carried nothing, and a consumer that reads the two as one artifact reads a
/// run nobody took as a run that found nothing.
fn named_nothing_run() -> Ran {
    ran(Scoping::Nothing)
}

/// What one run of the fixture tree is about.
#[derive(Clone, Copy)]
enum Scoping {
    /// The whole corpus.
    Corpus,
    /// One change, as [`manifest_over`] describes it.
    Change,
    /// One change that named nothing.
    Nothing,
}

/// The name a prior version is written under in the manifests below.
///
/// The source of a prior version is a path the caller owns and this engine
/// opens through the closure it is handed, so a test supplies a tree no
/// filesystem holds. Reading the current document and adding a paragraph is a
/// document whose prose moved and whose front matter did not, which is the
/// change that carries a prior version and moves no state.
const PRIOR: &str = "the prior version of ";

/// The manifest the scoped fixtures are recorded from, and the four states one
/// line of a manifest can reach.
///
/// `Held` has four arms and three of them are reachable in no recorded artifact
/// of this crate: a document the change adds, one whose prior version this run
/// read, and one whose prior version it could not. The fourth is a path that
/// binds to no row of the corpus, and four lines reach it here, because a real
/// change carries files that are no document of anything, a mistyped path looks
/// exactly the same from inside the engine, and `./` in front of a real path is
/// a spelling this module refuses to normalize.
///
/// **The counts are pairwise distinct, and that is the point of the line
/// counts below rather than an accident of them.** Reaching all four states is
/// not what a recorded artifact pins. An earlier version of this manifest
/// reached all four and produced `added: 1, carried: 1, unreadable: 1`, so an
/// emitter that wrote any one of those three where it meant another moved no
/// recorded byte and failed no test. [`no_two_counts_of_the_scoped_fixture_are_equal`]
/// is what holds the property now.
///
/// The real paths are read off the census in walk order rather than written
/// here, for the reason the adoption pair above is: a fixture that pinned a
/// name would stop describing the tree the day somebody renamed a document.
fn manifest_over(taken: &Census) -> Change {
    let paths: Vec<&str> = taken.rows.iter().map(|row| row.path.as_str()).collect();
    let (added, carried, unreadable) = (&paths[0..2], paths[2], &paths[3..6]);
    let mut manifest = String::from(FORMAT);
    manifest.push('\n');
    for path in added {
        manifest.push_str(&format!("added\t{path}\n"));
    }
    manifest.push_str(&format!("prior\t{carried}\t{PRIOR}{carried}\n"));
    for path in unreadable {
        manifest.push_str(&format!(
            "prior\t{path}\ta prior version that no tree holds\n"
        ));
    }
    // Four paths that bind to nothing. The last two name a prior version that
    // does read, because `Unbound::read` opens it before `Unbound::bind` ever
    // holds the path against the corpus, and the state a line ends in is the
    // binding rather than the read.
    manifest.push_str(&format!(
        "added\tREADME.md\n\
         added\tengine/crates/check/src/change.rs\n\
         prior\tcheck/spec/00-both-halves.markdown\t{PRIOR}{carried}\n\
         prior\t./{carried}\t{PRIOR}{carried}\n"
    ));
    bound(&manifest, taken)
}

/// One manifest, read and then held against the corpus this run walked.
fn bound(manifest: &str, taken: &Census) -> Change {
    let unbound = Unbound::read(manifest, |source| {
        match source.to_str().and_then(|name| name.strip_prefix(PRIOR)) {
            Some(path) => {
                let mut bytes = std::fs::read(check_fixtures().join(path))?;
                bytes.extend_from_slice(b"\nOne paragraph that this change removed.\n");
                Ok(bytes)
            }
            // A stated message rather than the operating system's, because the
            // count of unreadable prior versions is recorded and the reason a
            // host gives for a missing file is not the same string everywhere.
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no prior version stands at this name",
            )),
        }
    })
    .expect("the manifest reads");
    unbound.bind(|path| taken.rows.iter().any(|row| row.path == path))
}

/// One run of the fixture tree, at the stated scoping, with the payload that
/// holds one pair.
///
/// Every scoping carries that payload, so all three classes of finding are in
/// every artifact this file records. The pair is read off a run of the same
/// tree rather than written here, so it follows the fixture tree when its
/// content changes instead of pinning a path and a rule that a later edit
/// moves.
fn ran(scoping: Scoping) -> Ran {
    let bare = run_at(None).run;
    let applied = bare
        .suppressions
        .suppressions
        .iter()
        .find(|suppression| !suppression.hid.is_empty())
        .expect("the fixture tree suppresses something");
    let held = bare
        .findings
        .iter()
        .find(|finding| finding.rule != applied.rule || finding.path != applied.path)
        .expect("the fixture tree reports something");
    // A pair the directive does *not* cover, so the two inventories both end up
    // with something in them. The runner applies the payload first, so naming
    // the suppressed pair would empty the suppression inventory instead.
    let payload = headwater_yaml::load(&format!(
        "\
tasks:
  - id: AD-1
    statement: one pair, so that a run carries a migration-pending finding
    owner: the fixture tree
    until: 2027-01-01
    pairs:
      - {{path: {}, rule: {}}}
",
        held.path, held.rule
    ))
    .expect("the payload loads");
    let payload = payload.value.as_map().expect("a mapping").clone();
    scoped_at(Some(&payload), scoping)
}

fn run_at(adoption: Option<&Mapping>) -> Ran {
    scoped_at(adoption, Scoping::Corpus)
}

fn scoped_at(adoption: Option<&Mapping>, scoping: Scoping) -> Ran {
    let corpus = Corpus::new(check_fixtures(), "check");
    let root = load_map(&check_fixtures().join("check.taxonomy.yml"));
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
    let lock = lock_digest();
    let at = Context::at(Date::parse(PINNED).expect("the pinned date"));
    // The one constructor that produces a change-scoped run, so the answer to
    // "was this run scoped" is the presence of the value and never a flag
    // beside it. The manifest is bound against this walk rather than another,
    // which is what `bind` asks its caller for.
    let ctx = match scoping {
        Scoping::Corpus => at,
        Scoping::Change => at.scoped_to(manifest_over(&taken)),
        Scoping::Nothing => at.scoped_to(bound(&format!("{FORMAT}\n"), &taken)),
    };
    let run = headwater_check::run(
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
            source: "engine/crates/check/fixtures/check.taxonomy.yml",
        },
        &ctx,
        &mut Cache::disabled(),
    );
    Ran {
        run,
        census: taken,
        graph,
    }
}

fn subject(lock: &str) -> Subject<'_> {
    Subject {
        package: "headwater/fixture",
        version: "1.0.0",
        lock,
        now: PINNED,
    }
}

fn rendered(format: Format) -> String {
    render(&fixture_run(), format)
}

fn render(ran: &Ran, format: Format) -> String {
    let lock = lock_digest();
    headwater_adapter::render(&ran.run, &ran.census, &ran.graph, &subject(&lock), format)
}

fn render_at(ran: &Ran, format: Format, width: usize) -> String {
    let lock = lock_digest();
    headwater_adapter::render_at(
        &ran.run,
        &ran.census,
        &ran.graph,
        &subject(&lock),
        format,
        width,
        headwater_check::paint::ColorMode::Plain,
    )
}

/// One text with every whitespace run reduced to one space.
///
/// The text report is laid out at a width by `headwater_check::fill`, so a
/// sentence this engine composed as one line reaches a reader over two or three
/// of them. A case asking whether the report *states* something is asking about
/// the words and not about where they were broken, and this is the form that
/// question is asked in. A case that asked about the line as well would be
/// re-recording the layout, which is what `fixture.text` is for.
fn flowed(text: &str) -> String {
    text.split_whitespace().collect::<Vec<&str>>().join(" ")
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
        "\nthe artifact no longer matches {}",
        recorded.display()
    );
}

/// The tree has to break something in each of the three ways.
///
/// An adapter that marked nothing would pass every assertion below over a run
/// with no escaped finding in it, which is the vacuous test the differential
/// suite of the export emitters has its own guard against.
#[test]
fn the_run_carries_all_three_classes() {
    let run = fixture_run().run;
    let all = reported(&run);
    let count = |escape: Option<Escape>| all.iter().filter(|entry| entry.escape == escape).count();
    assert!(count(None) > 0, "the tree reports live findings");
    assert_eq!(
        count(Some(Escape::MigrationPending)),
        1,
        "the injected payload holds exactly one"
    );
    assert!(
        count(Some(Escape::Suppression)) > 0,
        "the tree suppresses something"
    );
    // And two severities, so that the level mapping below has two cases.
    let severities: BTreeSet<&str> = all
        .iter()
        .map(|entry| headwater_adapter::severity(entry.finding.severity))
        .collect();
    assert!(
        severities.len() > 1,
        "the tree reports more than one severity: {severities:?}"
    );
}

/// Every finding of the run reaches every format, or the format said why not.
///
/// The audit of the loss-set claim, in the shape spec 6 fixes for the graph
/// emitters. It reads the rendered bytes rather than the emitter. All four
/// formats are held to it, `text` among them: that format declares an empty
/// loss set, and an empty loss set is the strongest claim any of them makes.
///
/// Both scopings, because a scoped run is a run whose artifact carries a block
/// the other does not, and an emitter that dropped a finding while writing it
/// would pass this over the full-corpus run alone.
#[test]
fn every_finding_reaches_every_format() {
    for ran in [fixture_run(), scoped_run()] {
        for format in Format::ALL {
            let artifact = render(&ran, format);
            let audited = headwater_adapter::census(&ran.run, format, &artifact);
            assert!(
                !audited.is_defective(),
                "the {} adapter dropped {:?}",
                format.name(),
                audited.unaccounted
            );
            assert_eq!(audited.carried, audited.findings);
            assert!(audited.findings > 0);
        }
    }
}

/// Every entry of every loss set reaches one of the three outcomes, and the
/// three sum to the entries declared.
///
/// The tripwire under every other assertion here. An entry that fell out of all
/// three would be an entry the audit walked past, which is the whole defect this
/// half of the census was written for.
#[test]
fn every_entry_of_every_loss_set_is_accounted_for() {
    for ran in [fixture_run(), scoped_run()] {
        for format in Format::ALL {
            let artifact = render(&ran, format);
            let audited = headwater_adapter::census(&ran.run, format, &artifact);
            assert!(
                audited.accounts(),
                "{}: {} entries, {} held, {} adrift, {} unaudited",
                format.name(),
                audited.entries,
                audited.held,
                audited.adrift.len(),
                audited.unaudited
            );
            assert_eq!(audited.entries, format.loss().len());
            assert!(audited.adrift.is_empty(), "{:?}", audited.adrift);
        }
    }
}

/// What each format's loss set is held to, entry by entry.
///
/// The counts rather than the verdict. `is_defective` is false over a format
/// that audited nothing and over one that audited everything, so a suite that
/// asserted the verdict alone could not tell the two apart. Five of SARIF's
/// seven entries name a member of the document and two name nowhere. All four of
/// Markdown's name nowhere in it, because that artifact is prose. `text` and
/// `json` declare no loss at all.
#[test]
fn the_audited_and_the_unaudited_are_counted_apart() {
    for ran in [fixture_run(), scoped_run()] {
        for (format, entries, held, unaudited) in [
            (Format::Text, 0, 0, 0),
            (Format::Json, 0, 0, 0),
            (Format::Sarif, 7, 5, 2),
            (Format::Markdown, 4, 0, 4),
        ] {
            let artifact = render(&ran, format);
            let audited = headwater_adapter::census(&ran.run, format, &artifact);
            assert_eq!(
                (audited.entries, audited.held, audited.unaudited),
                (entries, held, unaudited),
                "{}",
                format.name()
            );
        }
    }
}

/// One place, at the lifetime a declaration has.
///
/// A loss set is `const` data in the crate it belongs to, so a place lives for
/// the program. A place this suite composes at run time does not, and leaking
/// one is what gives a test the same lifetime a declaration has. A handful of
/// single-element arrays for the life of one test binary.
fn place(at: &'static [&'static str], members: &'static [&'static str]) -> &'static [Place] {
    Box::leak(Box::new([Place { at, members }]))
}

/// One place in a SARIF run object, named by a test rather than by a format.
fn run_place(at: &'static [&'static str], members: &'static [&'static str], when: OnRun) -> Loss {
    Loss {
        field: "a field this test invented",
        reason: "so that a declaration this suite wrote can be held to an artifact",
        carrier: Carrier::Run {
            places: place(at, members),
            when,
        },
    }
}

/// One place in every result of a SARIF run.
fn finding_place(
    at: &'static [&'static str],
    members: &'static [&'static str],
    when: OnFinding,
) -> Loss {
    Loss {
        field: "a field this test invented",
        reason: "so that a declaration this suite wrote can be held to an artifact",
        carrier: Carrier::PerFinding {
            places: place(at, members),
            when,
        },
    }
}

fn no_run(_: &Run) -> bool {
    false
}

fn no_finding(_: &headwater_adapter::Reported<'_>, _: &Run) -> bool {
    false
}

/// A loss entry that is false fails the audit, and a true one passes it.
///
/// The test this half of the census exists for, at both grains. Before this
/// branch the audit reached neither: it walked the finding set, so a run-level
/// entry and a record-level entry were both claims that only a recorded artifact
/// held. "It passed" and "it was never asked" were one result.
///
/// The declaration is what varies and the artifact is fixed, because a format's
/// loss set is a `const`: a probe that could only doctor the artifact would
/// measure the emitter, and the instrument is what is on trial here.
#[test]
fn a_carrier_that_is_wrong_is_adrift_and_one_that_is_right_is_held() {
    let ran = fixture_run();
    let artifact = render(&ran, Format::Sarif);

    for (name, entry, adrift) in [
        (
            "a run-level member that is there",
            run_place(&["properties", "headwater", "coverage"], &[], every_run),
            false,
        ),
        (
            "a run-level member that is not",
            run_place(&["properties", "headwater", "covrage"], &[], every_run),
            true,
        ),
        (
            "a record-level member that is there",
            finding_place(
                &["properties", "headwater", "escape"],
                &[],
                headwater_adapter::every_finding,
            ),
            false,
        ),
        (
            "a record-level member that is not",
            finding_place(
                &["properties", "headwater", "escapee"],
                &[],
                headwater_adapter::every_finding,
            ),
            true,
        ),
    ] {
        let loss = [entry];
        let audited = headwater_adapter::census_with(&ran.run, Format::Sarif, &artifact, &loss);
        assert!(audited.accounts(), "{name}");
        assert_eq!(audited.entries, 1, "{name}");
        assert_eq!(audited.unaudited, 0, "{name}");
        assert_eq!(!audited.adrift.is_empty(), adrift, "{name}: {audited:?}");
        assert_eq!(audited.held, usize::from(!adrift), "{name}");
        assert_eq!(audited.is_defective(), adrift, "{name}");
    }
}

/// A block that is there and empty of the values the entry claims is adrift.
///
/// The injectivity question, over the case that produced #233. `coverage` named
/// a property bag while three of the seven values it reports went nowhere, and a
/// carrier that stopped at the bag would report that entry as held. The members
/// are what the entry names now, and a member that is not under the block fails
/// while the block itself resolves.
#[test]
fn a_carrier_that_names_the_block_and_not_the_values_cannot_see_the_difference() {
    let ran = fixture_run();
    let artifact = render(&ran, Format::Sarif);
    let at: &[&str] = &["properties", "headwater", "coverage"];

    let block = [run_place(at, &[], every_run)];
    let audited = headwater_adapter::census_with(&ran.run, Format::Sarif, &artifact, &block);
    assert_eq!(audited.held, 1, "the block alone resolves");

    let values = [run_place(at, &["seen", "skips", "unaccounted"], every_run)];
    let audited = headwater_adapter::census_with(&ran.run, Format::Sarif, &artifact, &values);
    assert_eq!(audited.held, 1, "and so do the values it carries");

    let missing = [run_place(
        at,
        &["seen", "the_document_each_skip_fell_on"],
        every_run,
    )];
    let audited = headwater_adapter::census_with(&ran.run, Format::Sarif, &artifact, &missing);
    assert!(
        !audited.adrift.is_empty(),
        "a value the block does not hold is adrift while the block resolves"
    );
    assert!(audited.adrift[0].contains("the_document_each_skip_fell_on"));
}

/// A member written where the run carries no value for it is adrift too.
///
/// The other direction, and it is the one #234 rests on: an absent `change`
/// member is what says a run read the whole corpus. An audit that only asked
/// whether a declared member was present would report an emitter that wrote the
/// member on every run as correct, and a reader of that artifact could no longer
/// tell the two runs apart.
#[test]
fn a_member_present_where_the_run_has_no_value_is_adrift() {
    let at: &[&str] = &["properties", "headwater", "change"];
    let full = fixture_run();
    let scoped = scoped_run();

    let claimed = [run_place(at, &[], every_run)];
    let audited = headwater_adapter::census_with(
        &full.run,
        Format::Sarif,
        &render(&full, Format::Sarif),
        &claimed,
    );
    assert!(
        !audited.adrift.is_empty(),
        "a full-corpus artifact has no change member to hold"
    );

    let denied = [run_place(at, &[], no_run)];
    let audited = headwater_adapter::census_with(
        &scoped.run,
        Format::Sarif,
        &render(&scoped, Format::Sarif),
        &denied,
    );
    assert!(
        !audited.adrift.is_empty(),
        "a scoped artifact has one, and a carrier that denies it is wrong about it"
    );
    assert!(audited.adrift[0].contains("carries no value for it"));

    let denied_per_finding = [finding_place(
        &["properties", "headwater", "escape"],
        &[],
        no_finding,
    )];
    let audited = headwater_adapter::census_with(
        &full.run,
        Format::Sarif,
        &render(&full, Format::Sarif),
        &denied_per_finding,
    );
    assert!(
        !audited.adrift.is_empty(),
        "and the same holds one grain down, on a member every result carries"
    );
}

/// A member path into an artifact nothing can parse is a defect, not a pass.
///
/// The `Option` this branch could have reintroduced one level up. The audit
/// resolves a path against a parsed document, so every step of it returns an
/// `Option`, and a reader that treated "I could not read this" as "there was
/// nothing to find" would make the whole instrument green over an artifact it
/// never opened. Two states reach it here: a format whose artifact is prose, and
/// bytes that are not a document at all.
#[test]
fn a_carrier_the_reader_cannot_reach_is_adrift_rather_than_unaudited() {
    let ran = fixture_run();
    let loss = [run_place(
        &["properties", "headwater", "coverage"],
        &[],
        every_run,
    )];

    for (name, format, artifact) in [
        ("prose", Format::Markdown, render(&ran, Format::Markdown)),
        ("prose", Format::Text, render(&ran, Format::Text)),
        (
            "not a document",
            Format::Sarif,
            "\t\tnot a document".to_string(),
        ),
        ("an empty document", Format::Sarif, "{}\n".to_string()),
    ] {
        let audited = headwater_adapter::census_with(&ran.run, format, &artifact, &loss);
        assert!(audited.accounts(), "{name}");
        assert_eq!(audited.unaudited, 0, "{name}: it was asked");
        assert_eq!(audited.held, 0, "{name}: and it did not hold");
        assert!(audited.is_defective(), "{name}");
    }
}

/// A record count that does not match the finding count is a defect.
///
/// The precondition under every per-record carrier. Both machine-readable
/// emitters write one record per reported finding, in that order, and an audit
/// that matched by position without saying so would read the wrong record after
/// any drift. The artifact here is one run's and the finding set is another's.
#[test]
fn a_record_list_that_does_not_line_up_with_the_finding_set_is_adrift() {
    let full = fixture_run();
    let loss = [finding_place(
        &["properties", "headwater", "escape"],
        &[],
        headwater_adapter::every_finding,
    )];
    let empty = r#"{"version": "2.1.0", "runs": [{"results": []}]}"#;
    let audited = headwater_adapter::census_with(&full.run, Format::Sarif, empty, &loss);
    assert!(!audited.adrift.is_empty());
    assert!(audited.adrift[0].contains("records for"));
}

/// The sentence an artifact carries is derived from the carrier and from nothing
/// else.
///
/// One source for the declaration and for the string a consumer reads. The
/// recorded artifacts pin the seven strings; this pins that they are pairwise
/// distinct where they say anything, so a permutation of two entries could not
/// pass the recordings.
#[test]
fn every_carrier_renders_the_one_sentence_and_no_two_agree() {
    for format in Format::ALL {
        let mut named: Vec<String> = format
            .loss()
            .iter()
            .map(|entry| entry.carried_in())
            .filter(|text| !text.is_empty())
            .collect();
        let before = named.len();
        named.sort();
        named.dedup();
        assert_eq!(named.len(), before, "{}", format.name());
    }
    let sarif: Vec<String> = Format::Sarif
        .loss()
        .iter()
        .map(|entry| entry.carried_in())
        .collect();
    assert_eq!(
        sarif,
        [
            "properties.headwater.obligation_severity",
            "properties.headwater.escape",
            "",
            "run.properties.headwater.coverage",
            "properties.headwater.remediation and message.markdown",
            "run.properties.headwater.change",
            "",
        ]
    );
}

/// Whether one artifact states what the run was scoped to.
///
/// Read off the rendered bytes and never off the run, for the reason
/// `headwater_adapter::census` reads bytes: an emitter that audited itself is
/// the untrusted projector one layer out. `promotions` is a word that no other
/// member of either JSON document holds.
fn states_the_scoping(format: Format, artifact: &str) -> bool {
    match format {
        Format::Text => artifact.contains("scoped to a change:"),
        Format::Json | Format::Sarif => artifact.contains("\"promotions\""),
        Format::Markdown => artifact.contains("**Scoped to a change.**"),
    }
}

/// The manifest the recorded scoped artifacts are written from reaches all four
/// states a line of one can carry.
///
/// The guard that stops every assertion below from comparing one empty set with
/// another. Three of these four counts are zero in every other fixture of this
/// crate, and a recorded artifact whose interesting values are all zero moves
/// when the block is deleted and never when a count is wrong.
#[test]
fn the_scoped_fixture_reaches_every_state_a_manifest_line_can() {
    let run = scoped_run().run;
    let scoped = run.change.expect("the run was scoped");
    let named = &scoped.named;
    assert_eq!(named.documents, 10);
    assert_eq!(named.added, 2, "two documents the change adds");
    assert_eq!(named.carried, 1, "one prior version this run read");
    assert_eq!(named.unreadable, 3, "three prior versions it could not");
    assert_eq!(named.unmatched, 4, "four paths that bind to no row");
    // The list and the count are two readings of one set, and an artifact that
    // wrote the list while a consumer read the count would put the two at odds.
    assert_eq!(scoped.unmatched.len(), named.unmatched);
    // In path order. A real path with `./` in front of it, a file that is no
    // document of anything, a document path with one character wrong, and a
    // source file the change carried. Those are the reasons a manifest names a
    // path that binds to nothing, and they are indistinguishable from inside
    // the engine, which is why the report names the path rather than counting
    // it.
    assert_eq!(
        scoped.unmatched,
        [
            "./check/evaluations/delta.md",
            "README.md",
            "check/spec/00-both-halves.markdown",
            "engine/crates/check/src/change.rs",
        ]
    );
}

/// No two values of the block the recorded scoped fixtures carry are equal.
///
/// The four states a manifest line can reach are not the property a recorded
/// artifact pins. The counts are. An earlier version of this manifest reached
/// all four states and produced `added: 1, carried: 1, unreadable: 1`, and two
/// deliberate swaps of those members — `carried` written from `added`, and
/// `carried` written from `unreadable` — passed all 24 tests of this file and
/// moved no recorded byte.
///
/// Every scalar of the block is distinct here, and the length of the unmatched
/// list is distinct from all of them, so a member written in the wrong place
/// changes the three recorded artifacts.
#[test]
fn no_two_counts_of_the_scoped_fixture_are_equal() {
    let run = scoped_run().run;
    let scoped = run.change.expect("the run was scoped");
    let named = &scoped.named;
    let counts = [
        ("documents", named.documents),
        ("added", named.added),
        ("carried", named.carried),
        ("unreadable", named.unreadable),
        ("promotions", scoped.promotions),
        ("the unmatched paths", scoped.unmatched.len()),
    ];
    for (one, left) in counts {
        for (two, right) in counts {
            assert!(
                one == two || left != right,
                "`{one}` and `{two}` are both {left}, so an emitter that wrote one where it \
                 means the other moves no recorded byte"
            );
        }
    }
}

/// A full-corpus run says nothing about a change, in any of the four.
///
/// The member is absent rather than present and empty, which is the statement
/// `Scoped`'s own `Option` makes: a run that read the corpus has no change to
/// report, and a value there would make every consumer tell "absent" from
/// "present and zero" before it could read either.
#[test]
fn a_full_corpus_run_states_no_change_in_any_format() {
    let ran = fixture_run();
    assert!(ran.run.change.is_none());
    for format in Format::ALL {
        let artifact = render(&ran, format);
        assert!(
            !states_the_scoping(format, &artifact),
            "the {} artifact of a full-corpus run states a scoping",
            format.name()
        );
    }
}

/// A scoped run says so in all four, which is what #230 is.
#[test]
fn a_scoped_run_states_the_change_in_every_format() {
    let ran = scoped_run();
    for format in Format::ALL {
        let artifact = render(&ran, format);
        assert!(
            states_the_scoping(format, &artifact),
            "the {} artifact of a scoped run states no scoping",
            format.name()
        );
    }
}

/// What this branch added to each format is one contiguous block and nothing
/// else.
///
/// One run, rendered twice: once as it stands and once with its change dropped.
/// That isolates the emitter from the runner, which the comparison below does
/// not, and an emitter that moved a count while writing the block fails here.
///
/// The block is where the format puts it, so the assertion is that the two
/// artifacts agree on a prefix of lines and on a suffix of lines and that
/// everything between is in one of them alone.
#[test]
fn each_format_gained_one_block_and_moved_nothing() {
    let mut ran = scoped_run();
    let with: Vec<String> = Format::ALL.map(|format| render(&ran, format)).to_vec();
    ran.run.change = None;
    for (format, with) in Format::ALL.into_iter().zip(with) {
        let without = render(&ran, format);
        assert!(!states_the_scoping(format, &without));
        let block = inserted(&without, &with).join("\n");
        assert!(
            states_the_scoping(format, &block),
            "the {} artifact gained a block that states no scoping: {block}",
            format.name()
        );
    }
}

/// The lines one artifact holds that the other does not, when the difference is
/// one insertion.
///
/// A panic when it is anything else. Two emitters could agree on every byte of
/// the block and still disagree about a count somewhere above it, and a test
/// that compared only the block would report that as a pass.
fn inserted<'a>(without: &str, with: &'a str) -> Vec<&'a str> {
    let short: Vec<&str> = without.lines().collect();
    let long: Vec<&str> = with.lines().collect();
    assert!(long.len() > short.len(), "nothing was inserted");
    let head = short
        .iter()
        .zip(&long)
        .take_while(|(one, two)| one == two)
        .count();
    let tail = short
        .iter()
        .rev()
        .zip(long.iter().rev())
        .take_while(|(one, two)| one == two)
        .count();
    // `>=` and not `==`. A blank line either side of the block matches from the
    // top and from the bottom alike, so the two greedy runs overlap, and the
    // decomposition into one insertion exists exactly when they cover the
    // shorter artifact between them.
    assert!(
        head + tail >= short.len(),
        "the two artifacts differ in more than one place: they agree on {head} lines from the \
         top and {tail} from the bottom, of {}",
        short.len()
    );
    let tail = short.len() - head;
    long[head..long.len() - tail].to_vec()
}

/// A scoped run is not a full-corpus run with one block added, and the
/// difference is the runner's rather than this crate's.
///
/// Every document of a scoped run has a prior version, including the ones the
/// change did not name, so the two `needs_prior` rules evaluate over all of
/// them. A document whose every instance is one of those is unchecked in a
/// full-corpus run and checked here, and both the coverage count and the
/// `coverage.document_unchecked` finding move with it.
///
/// A reviewer who diffs the recorded scoped artifacts against the full-corpus
/// ones meets that difference beside the block, and this is what it is.
#[test]
fn a_scoped_run_checks_a_document_a_full_corpus_run_cannot() {
    let full = fixture_run().run;
    let scoped = scoped_run().run;
    assert!(
        full.coverage.checked() < scoped.coverage.checked(),
        "a scoped run checks at least one document a full-corpus run leaves unchecked"
    );
    let unchecked = |run: &Run| {
        run.findings
            .iter()
            .filter(|finding| finding.rule == "coverage.document_unchecked")
            .count()
    };
    assert!(unchecked(&full) > unchecked(&scoped));
}

/// Every format states the scoping a second way, and this test asserted the
/// opposite until #233.
///
/// A full-corpus run skips every instance of the two `needs_prior` rules with
/// the reason `change-scoped-only`, and a scoped run skips none of them. That is
/// the same difference #230 closed, read off the outcome of the instances rather
/// than off the manifest, and it is the second thing a consumer can now hold one
/// run against another with.
///
/// The committed form of this test named the drop and required it: it asserted
/// that the three translations carry no `change-scoped` anywhere. A test that
/// holds a defect in place passes for as long as the defect stands, which is
/// what it did.
#[test]
fn every_format_states_the_scoping_a_second_way() {
    for format in Format::ALL {
        let full = render(&fixture_run(), format);
        assert!(
            full.contains("change-scoped-only"),
            "the {} artifact of a full-corpus run names no change-scoped-only skip",
            format.name()
        );
        let scoped = render(&scoped_run(), format);
        assert!(
            !scoped.contains("change-scoped-only"),
            "the {} artifact of a scoped run names a change-scoped-only skip, and a scoped run \
             reaches a verdict on every one of them",
            format.name()
        );
    }
}

/// The coverage block of one artifact, in the two formats that carry data.
fn coverage_block(format: Format, artifact: &str) -> Value {
    let document = parse(artifact);
    let holder = match format {
        Format::Json => document,
        Format::Sarif => member(&runs(&document)[0], "properties")
            .and_then(|properties| member(&properties, "headwater"))
            .expect("the property bag"),
        _ => panic!("{} carries no parseable coverage", format.name()),
    };
    member(&holder, "coverage").expect("the coverage block")
}

/// One member of the coverage block, as a number.
fn count(block: &Value, key: &str) -> usize {
    text(block, key).parse().expect("a whole number")
}

/// Every format states how many of the run's instances reached no verdict, and
/// under which class.
///
/// [Spec 4](../../../../docs/spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)
/// asks OB-COV-3 for the skips with their reasons, and three of the four formats
/// carried the instance total and nothing else. A consumer read `266 instances`
/// off a run where 66 of them decided nothing.
///
/// The two data formats are parsed and the member read. The two prose formats
/// assert a whole sentence, because every artifact of this engine carries a
/// sha256 in hexadecimal and a finding carries a line number, so an assertion
/// that one of them contains a short run of digits is true whatever the emitter
/// wrote.
#[test]
fn every_format_states_how_many_instances_reached_no_verdict() {
    let ran = fixture_run();
    let skipped = ran.run.coverage.skipped();
    let classes = ran.run.coverage.skips().len();
    assert!(
        skipped > 0 && classes > 1,
        "the tree skips under more than one class"
    );

    for format in [Format::Json, Format::Sarif] {
        let block = coverage_block(format, &render(&ran, format));
        assert_eq!(count(&block, "instances"), ran.run.coverage.instances);
        assert_eq!(count(&block, "skipped"), skipped);
        let skips = member(&block, "skips")
            .expect("the skip classes")
            .as_seq()
            .expect("an array")
            .iter()
            .map(|entry| entry.value.clone())
            .collect::<Vec<Value>>();
        assert_eq!(
            skips.len(),
            classes,
            "one entry per class in {}",
            format.name()
        );
        let summed: usize = skips.iter().map(|entry| count(entry, "instances")).sum();
        assert_eq!(
            summed, skipped,
            "the classes partition the skipped instances"
        );
        for (entry, (reason, instances)) in skips.iter().zip(ran.run.coverage.skips()) {
            assert_eq!(&text(entry, "reason"), reason);
            assert_eq!(count(entry, "instances"), *instances);
        }
        // Present and empty rather than absent. This is a full-corpus run, so
        // the one rule that reads outside the census reads nothing at all: it
        // declares the prior version and skips where there is no change. The
        // scoped fixture is where the member carries paths, and a member that
        // appeared only when it was non-empty would make a reader tell "none"
        // from "not reported".
        assert_eq!(
            member(&block, "unaccounted")
                .expect("the unaccounted paths")
                .as_seq()
                .expect("an array")
                .len(),
            0
        );
    }

    let text_report = render(&ran, Format::Text);
    let markdown = render(&ran, Format::Markdown);
    assert!(text_report.contains("267 check instances"));
    assert!(markdown.contains("It created 267 check instances, and 67 of them reached no verdict."));
    // The report is laid out at a width, so a long reason arrives over more
    // than one line. The class is a run of words either way.
    let flat = flowed(&text_report);
    for (reason, instances) in ran.run.coverage.skips() {
        assert!(
            flat.contains(&flowed(&format!("{instances} skipped: {reason}"))),
            "the text report states no `{reason}` class"
        );
        assert!(
            markdown.contains(&format!("- {instances} — {reason}")),
            "the markdown report states no `{reason}` class"
        );
    }
}

/// A path that no census row accounts for is named in the artifact.
///
/// `Coverage::unaccounted` is written when an instance reads a file the census
/// never walked. That was unreachable until `lifecycle.deletion.not_permitted`,
/// which reads the version of every path a change named that no row holds, so
/// the recorded scoped artifact now carries two entries and the recorded
/// full-corpus one still carries none. This test predates both and keeps its
/// own state, because what it holds is the emitter rather than the rule: one
/// instance over a path that is on no row of a real census.
///
/// What it proves is what the block is for — a check that read outside the
/// denominator read outside the set every coverage guarantee is computed over,
/// and the reader who can act on that needs the path rather than a count of
/// them.
#[test]
fn a_path_the_census_never_walked_is_named_and_not_counted() {
    let ran = fixture_run();
    let stray = "check/spec/a-file-no-walk-of-this-tree-reaches.md";
    let outside = headwater_check::Coverage::of(
        &ran.census,
        &[headwater_check::instance::Instance::of(
            "duplicate.identifier",
            headwater_check::scope::Grain::Document,
            vec![headwater_check::instance::Input {
                path: stray.to_string(),
                digest: None,
            }],
            headwater_check::instance::Outcome::Passed,
        )],
    );
    assert_eq!(outside.unaccounted, [stray]);
    let block = parse(&headwater_adapter::json::coverage(&outside).render_pretty());
    let named: Vec<String> = member(&block, "unaccounted")
        .expect("the unaccounted paths")
        .as_seq()
        .expect("an array")
        .iter()
        .map(|path| scalar(&path.value))
        .collect();
    assert_eq!(
        named,
        [stray.to_string()],
        "the path, and not a count of them"
    );
    // And the empty case is a different artifact, so the member states which of
    // the two this run was.
    let clean = headwater_adapter::json::coverage(&ran.run.coverage).render_pretty();
    assert!(ran.run.coverage.unaccounted.is_empty());
    assert_ne!(
        clean,
        headwater_adapter::json::coverage(&outside).render_pretty()
    );
}

/// No two counts of the coverage block the recorded fixtures carry are equal.
///
/// The same property [`no_two_counts_of_the_scoped_fixture_are_equal`] holds for
/// the change block, and for the same reason: seven independent reads of one
/// structure, and an emitter that wrote any of them where it meant another moves
/// no recorded byte when two of them are equal.
///
/// Over the **full-corpus** fixture, because a scoped run of this tree checks
/// every document it classified, so `classified` and `checked` are equal there by
/// construction. Both artifacts are recorded, so a swap of those two still fails
/// this crate.
#[test]
fn no_two_counts_of_the_coverage_block_are_equal() {
    let coverage = fixture_run().run.coverage;
    let counts = [
        ("seen", coverage.seen()),
        ("classified", coverage.classified()),
        ("checked", coverage.checked()),
        ("generated", coverage.generated()),
        ("instances", coverage.instances),
        ("skipped", coverage.skipped()),
        ("the unaccounted paths", coverage.unaccounted.len()),
    ];
    for (one, left) in counts {
        for (two, right) in counts {
            assert!(
                one == two || left != right,
                "`{one}` and `{two}` are both {left}, so an emitter that wrote one where it \
                 means the other moves no recorded byte"
            );
        }
    }
}

/// A run that skipped nothing says so, and it is not a run that reports no
/// skips.
///
/// The empty arm, and it has a location. A coverage block stands on every run,
/// so absence cannot be its statement the way it is the change block's, and
/// `"skipped": 0` with an empty class list is what "every instance this run
/// created reached a verdict" looks like. What separates that from a producer
/// with no such member to write is the shape version, which is why #233 moved it.
#[test]
fn a_run_that_skipped_nothing_is_not_a_run_that_reports_no_skips() {
    let ran = fixture_run();
    let nothing = headwater_check::Coverage::of(&ran.census, &[]);
    assert_eq!(nothing.skipped(), 0);
    let block = parse(&headwater_adapter::json::coverage(&nothing).render_pretty());
    assert_eq!(count(&block, "skipped"), 0);
    assert_eq!(
        member(&block, "skips")
            .expect("the skip classes")
            .as_seq()
            .expect("an array")
            .len(),
        0
    );
    let skipping = parse(&headwater_adapter::json::coverage(&ran.run.coverage).render_pretty());
    assert_ne!(
        headwater_adapter::json::coverage(&nothing).render_pretty(),
        headwater_adapter::json::coverage(&ran.run.coverage).render_pretty(),
        "a run that skipped nothing and a run that skipped 66 write one block"
    );
    assert!(count(&skipping, "skipped") > 0);
    // And the shape version is what dates the member, so a reader of a document
    // that carries no `skipped` knows which of the two it is holding.
    assert_eq!(headwater_adapter::json::VERSION, "1.2");
}

/// A change that named nothing is not a full-corpus run, in any of the four.
///
/// The pair #178 opens by naming, one level out: "a count over a change that
/// was never described is zero, and so is a count over a change that genuinely
/// promoted nothing". The text report told them apart before this branch and
/// the other three did not.
#[test]
fn a_change_that_named_nothing_is_not_a_full_corpus_run() {
    let nothing = named_nothing_run();
    let full = fixture_run();
    let named = nothing.run.change.as_ref().expect("the run was scoped");
    assert_eq!(named.named.documents, 0);
    assert_eq!(named.promotions, 0);
    for format in Format::ALL {
        let over_nothing = render(&nothing, format);
        assert!(
            states_the_scoping(format, &over_nothing),
            "the {} artifact of a change that named nothing states no scoping",
            format.name()
        );
        assert_ne!(
            over_nothing,
            render(&full, format),
            "the {} artifacts of a change that named nothing and of the corpus are one artifact",
            format.name()
        );
    }
}

/// The promotion count reaches every format, and it is the count the run holds.
///
/// The recorded artifacts carry zero promotions, because no document of the
/// fixture tree carries a warrant at all and a fixture that grew one would
/// re-bless the runner's own recorded reports for a reason unrelated to this.
/// So the non-zero value is exercised here, against the emitters, which are
/// what this crate is a test of.
///
/// Read out of the parsed document and never as a substring. A digest is
/// hexadecimal and a finding carries a line number, so `contains("37")` is true
/// of every artifact here whatever the emitter wrote: a probe that replaced the
/// count with a literal zero passed a test written that way, in all four.
#[test]
fn the_promotion_count_reaches_every_format() {
    let mut ran = scoped_run();
    let scoped = ran.run.change.as_mut().expect("the run was scoped");
    scoped.promotions = 37;

    let json = parse(&render(&ran, Format::Json));
    let block = member(&json, "change").expect("the change");
    assert_eq!(text(&block, "promotions"), "37");

    let document = parse(&render(&ran, Format::Sarif));
    let bag = member(&runs(&document)[0], "properties")
        .and_then(|properties| member(&properties, "headwater"))
        .and_then(|headwater| member(&headwater, "change"))
        .expect("the change");
    assert_eq!(text(&bag, "promotions"), "37");

    // The two prose formats have no member to read, so the sentence is the
    // assertion, and each one is the whole sentence rather than the number.
    assert!(render(&ran, Format::Markdown)
        .contains("37 promoted from `asserted` to `accepted` in this change."));
    assert!(render(&ran, Format::Text).contains("37 promoted from `asserted` to `accepted`"));
}

/// The change rides in the run's property bag, and in no member of the
/// vocabulary.
///
/// `invocations` is "the runtime environment of the analysis tool run" and
/// every member of it is a fact about the process. These counts are what the
/// engine made of a manifest, so they are not that. The schema decides neither
/// question: it admits a property bag on the run and on the invocation alike,
/// and it refuses an invented member on either. So this test holds the choice,
/// and the loss set states it.
#[test]
fn the_change_rides_in_the_runs_property_bag() {
    let artifact = render(&scoped_run(), Format::Sarif);
    let document = parse(&artifact);
    let run = runs(&document)[0].clone();
    let bag = member(&run, "properties")
        .and_then(|properties| member(&properties, "headwater"))
        .and_then(|headwater| member(&headwater, "change"))
        .expect("the change is in the run's property bag");
    assert_eq!(text(&bag, "documents"), "10");
    assert!(
        member(&run, "change").is_none(),
        "an invented member of the run object"
    );
    let invocation = member(&run, "invocations")
        .expect("the invocations")
        .as_seq()
        .expect("an array")[0]
        .value
        .clone();
    assert!(member(&invocation, "change").is_none());
    assert!(
        member(&invocation, "properties").is_none(),
        "the invocation carries no property bag of ours"
    );
    // And the loss set says so inside the artifact, which is what a consumer
    // that holds the bytes and not this repository reads.
    assert!(artifact.contains("run.properties.headwater.change"));
}

/// A suppressed finding is in the artifact and it is marked, and a live one is
/// in the artifact and is not.
///
/// This is the distinction a naive adapter flattens. A surface that dropped the
/// escaped findings would report a suppression nobody can count, and one that
/// showed them beside the live findings would report declared debt as a
/// regression.
#[test]
fn a_suppressed_finding_is_marked_and_a_live_one_is_not() {
    let artifact = rendered(Format::Sarif);
    let document = parse(&artifact);
    let results = results(&document);
    assert!(!results.is_empty());

    let live = results
        .iter()
        .filter(|result| member(result, "suppressions").is_none())
        .count();
    let escaped = results
        .iter()
        .filter(|result| member(result, "suppressions").is_some())
        .count();
    assert!(live > 0, "some results are live");
    assert!(escaped > 0, "some results are suppressed");

    let run = fixture_run().run;
    let all = reported(&run);
    assert_eq!(
        live,
        all.iter().filter(|entry| entry.is_live()).count(),
        "one SARIF result with no suppression per live finding"
    );
    assert_eq!(
        escaped,
        all.iter().filter(|entry| !entry.is_live()).count(),
        "one suppressed SARIF result per escaped finding"
    );
}

/// The two escape classes reach the two values SARIF has for the member.
///
/// A directive is written in the document it acts on, so it is persisted in
/// source. A task of the adoption payload is written in the lock, so it is
/// persisted outside it. The third class shares the second value when a waiver
/// mechanism exists, which is the entry the loss set carries.
#[test]
fn the_two_escape_classes_reach_the_two_sarif_kinds() {
    let artifact = rendered(Format::Sarif);
    let document = parse(&artifact);
    let mut kinds: Vec<(String, String)> = Vec::new();
    for result in results(&document) {
        let Some(suppressions) = member(&result, "suppressions") else {
            continue;
        };
        let first = suppressions.as_seq().expect("an array")[0].value.clone();
        kinds.push((
            text(&first, "kind"),
            text(
                member(&first, "properties")
                    .and_then(|bag| member(&bag, "headwater"))
                    .as_ref()
                    .expect("the property bag"),
                "escape",
            ),
        ));
    }
    assert!(kinds.contains(&(
        "inSource".to_string(),
        Escape::Suppression.name().to_string()
    )));
    assert!(kinds.contains(&(
        "external".to_string(),
        Escape::MigrationPending.name().to_string()
    )));
}

/// `level` is the check's severity, and the obligation's is the other member.
///
/// The fixture register declares obligations at `high`, `medium` and `low`, and
/// the tree reports findings at `error` and `warn`. If the emitter read the
/// obligation's scale, an `error` finding under a `medium` obligation would
/// carry a level derived from `medium`, and there is no such level.
#[test]
fn the_level_is_the_checks_severity_and_never_the_obligations() {
    let ran = fixture_run();
    let run = &ran.run;
    let lock = lock_digest();
    let artifact = headwater_adapter::render(
        &ran.run,
        &ran.census,
        &ran.graph,
        &subject(&lock),
        Format::Sarif,
    );
    let document = parse(&artifact);

    let mut seen = 0;
    let mut differed = 0;
    for (entry, result) in reported(run).iter().zip(results(&document)) {
        let expected = match entry.finding.severity {
            Severity::Error => "error",
            Severity::Warn => "warning",
            Severity::Info => "note",
        };
        assert_eq!(text(&result, "level"), expected, "{:?}", entry.finding.rule);
        seen += 1;
        let bag = member(&result, "properties")
            .and_then(|properties| member(&properties, "headwater"))
            .expect("the property bag");
        if let Some(declared) = member(&bag, "obligation_severity") {
            let declared = scalar(&declared);
            assert!(
                ["high", "medium", "low"].contains(&declared.as_str()),
                "the obligation scale is the other one: {declared}"
            );
            // The two scales share no value, so a member that carried the
            // obligation's severity into `level` would be caught above. This
            // counts the cases where the two are present together at all.
            differed += 1;
        }
    }
    assert!(seen > 0);
    assert!(
        differed > 0,
        "some finding names an obligation, so the two scales are both in the artifact"
    );
}

/// No rule declares the level it fires at.
///
/// A generated check reads its severity out of the taxonomy, so one rule spans
/// as many severities as there are declarations that generate it. A
/// `defaultConfiguration.level` would be a claim about the next run.
#[test]
fn no_rule_declares_a_default_configuration() {
    let artifact = rendered(Format::Sarif);
    assert!(
        !artifact.contains("defaultConfiguration"),
        "the emitter declares no level per rule"
    );
    // And every result carries its own, so nothing falls back to SARIF's
    // default of `warning`.
    let document = parse(&artifact);
    for result in results(&document) {
        assert!(member(&result, "level").is_some());
    }
}

/// The rule list describes the checker rather than the taxonomy.
///
/// Every rule the run served is in `tool.driver.rules`, and nothing else is. A
/// list built from the declarations would name a rule the engine declines to
/// evaluate, and a decline looks exactly like a constraint from the
/// declaration's side.
#[test]
fn the_rule_list_is_the_registry_that_ran() {
    let ran = fixture_run();
    let run = &ran.run;
    let lock = lock_digest();
    let artifact = headwater_adapter::render(
        &ran.run,
        &ran.census,
        &ran.graph,
        &subject(&lock),
        Format::Sarif,
    );
    let document = parse(&artifact);
    let rules = member(&runs(&document)[0], "tool")
        .and_then(|tool| member(&tool, "driver"))
        .and_then(|driver| member(&driver, "rules"))
        .expect("the rule list");
    let ids: Vec<String> = rules
        .as_seq()
        .expect("an array")
        .iter()
        .map(|rule| text(&rule.value, "id"))
        .collect();
    let served: Vec<String> = run
        .served
        .iter()
        .map(|served| served.rule.to_string())
        .collect();
    assert_eq!(ids, served);
}

/// The vendored schema is the copy OASIS published.
#[test]
fn the_vendored_schema_is_the_published_copy() {
    let bytes = std::fs::read(tests_dir().join("sarif-schema-2.1.0.json")).expect("the schema");
    assert_eq!(
        headwater_hash::hex(&bytes),
        SCHEMA_DIGEST,
        "the vendored schema is not the published one"
    );
}

#[test]
fn the_fixture_tree_renders_the_recorded_sarif() {
    compare(&tests_dir().join("fixture.sarif"), &rendered(Format::Sarif));
}

#[test]
fn the_fixture_tree_renders_the_recorded_json() {
    compare(&tests_dir().join("fixture.json"), &rendered(Format::Json));
}

#[test]
fn the_fixture_tree_renders_the_recorded_markdown() {
    compare(&tests_dir().join("fixture.md"), &rendered(Format::Markdown));
}

/// The report a person reads, recorded, which it had never been before.
///
/// The three formats above were recorded from the first day and the text was
/// not, because it was composed rather than laid out and nothing about it was
/// stable enough to diff. [#340](https://github.com/headwater-ai/headwater/issues/340)
/// laid it out, so this is the reviewable record of what the fill produces and
/// the file a later change to the fill has to re-bless.
#[test]
fn the_fixture_tree_renders_the_recorded_text() {
    compare(&tests_dir().join("fixture.text"), &rendered(Format::Text));
}

/// The default width is the width, and a wider one is a different report.
#[test]
fn the_width_a_caller_states_is_the_width_the_report_is_laid_out_at() {
    let ran = fixture_run();
    let ordinary = render(&ran, Format::Text);
    assert_eq!(
        render_at(&ran, Format::Text, headwater_check::fill::WIDTH),
        ordinary,
        "`render` is `render_at` at the standard width"
    );
    let wide = render_at(&ran, Format::Text, headwater_check::fill::WIDEST);
    assert_ne!(wide, ordinary, "the widest band widened nothing");
    // The same words either way. Only the breaks moved.
    assert_eq!(flowed(&wide), flowed(&ordinary));
    for (width, report) in [
        (headwater_check::fill::WIDTH, &ordinary),
        (headwater_check::fill::WIDEST, &wide),
    ] {
        // Everything but the read-set block, which is the artifact a gate
        // parses and is exempt for the reason `text.rs` states.
        let (laid_out, _) = report
            .split_once("\nread set\n")
            .expect("the report carries a read-set block");
        for line in laid_out.lines() {
            if line.chars().count() > width {
                assert!(
                    headwater_check::fill::unfoldable(line, width),
                    "at {width} columns a line the fill could have narrowed is {}: {line}",
                    line.chars().count()
                );
            }
        }
    }
}

/// The three machine formats ignore the width, at both ends of the band.
#[test]
fn a_width_reaches_the_report_a_person_reads_and_no_other_format() {
    let ran = fixture_run();
    for format in [Format::Json, Format::Sarif, Format::Markdown] {
        assert_eq!(
            render_at(&ran, format, headwater_check::fill::WIDEST),
            render(&ran, format),
            "`--format {}` is not laid out at a width",
            format.name()
        );
    }
}

/// The read-set block of the report is the artifact `--read-set` writes.
///
/// It is written verbatim to a file by `headwater check --read-set` and read
/// back by `headwater_check::Recorded::parse` in `headwater gate`, so it is a
/// grammar rather than prose and the fill does not touch it. This is the case
/// that goes red the day somebody folds it, at both ends of the band.
#[test]
fn the_read_set_block_is_not_laid_out_at_any_width() {
    let ran = fixture_run();
    let written = ran.run.read_set.render();
    assert!(!written.is_empty(), "the fixture run has a read set");
    for width in [headwater_check::fill::WIDTH, headwater_check::fill::WIDEST] {
        let report = render_at(&ran, Format::Text, width);
        let (_, block) = report
            .split_once("\nread set\n")
            .expect("the report carries a read-set block");
        let indented: String = written
            .lines()
            .map(|line| match line.is_empty() {
                true => String::from("\n"),
                false => format!("  {line}\n"),
            })
            .collect();
        assert_eq!(block, indented, "at {width} columns the block moved");
    }
}

// The same three formats over the same tree, scoped to a change. A reviewer
// reads these against the three above and the difference is the whole of what
// #230 shipped.

#[test]
fn a_scoped_run_renders_the_recorded_sarif() {
    compare(
        &tests_dir().join("fixture.scoped.sarif"),
        &render(&scoped_run(), Format::Sarif),
    );
}

#[test]
fn a_scoped_run_renders_the_recorded_json() {
    compare(
        &tests_dir().join("fixture.scoped.json"),
        &render(&scoped_run(), Format::Json),
    );
}

#[test]
fn a_scoped_run_renders_the_recorded_markdown() {
    compare(
        &tests_dir().join("fixture.scoped.md"),
        &render(&scoped_run(), Format::Markdown),
    );
}

/// A stock reader validates the document and finds the findings the run found.
///
/// Two directions. A record the reader produces that the run does not have is
/// an invented finding, and a finding of the run that the reader does not
/// produce is a dropped one. Both are valid SARIF, so the schema catches
/// neither, and this is the comparison that does.
#[test]
fn the_stock_reader_finds_the_same_findings_the_run_did() {
    for ran in [fixture_run(), scoped_run()] {
        stock_reader(&ran);
    }
}

/// One artifact, through the validator and the differential.
///
/// The scoped artifact goes through it too, because a property bag is where the
/// scoping rides and every object of this schema carries
/// `additionalProperties: false`. A member of ours written one level wrong is
/// an error there and nothing here would notice.
fn stock_reader(ran: &Ran) {
    let run = &ran.run;
    let artifact = render(ran, Format::Sarif);
    let Some(lines) = oracle(&artifact) else {
        return;
    };

    let mut reported_by_schema: BTreeSet<String> = BTreeSet::new();
    for line in &lines {
        let mut fields = line.split('\t');
        match fields.next() {
            Some("schema") => assert_eq!(
                fields.next(),
                Some("valid"),
                "the stock validator refused the document:\n{}",
                lines.join("\n")
            ),
            Some("result") => {
                reported_by_schema.insert(fields.collect::<Vec<&str>>().join("\t"));
            }
            _ => {}
        }
    }

    let mut reported_by_engine: BTreeSet<String> = BTreeSet::new();
    for entry in reported(run) {
        let finding = entry.finding;
        reported_by_engine.insert(format!(
            "{}\t{}\t{}\t{}\t{}",
            finding.rule,
            finding.path,
            finding.line,
            headwater_adapter::sarif::level(finding.severity),
            match entry.escape {
                None => "live",
                Some(escape) => headwater_adapter::sarif::kind(escape),
            }
        ));
    }

    let invented: Vec<&String> = reported_by_schema.difference(&reported_by_engine).collect();
    let dropped: Vec<&String> = reported_by_engine.difference(&reported_by_schema).collect();
    assert!(invented.is_empty(), "the emitter invented {invented:?}");
    assert!(dropped.is_empty(), "the emitter dropped {dropped:?}");
    assert!(!reported_by_engine.is_empty());
}

/// Run the oracle, or report why it did not run.
///
/// `None` when Python or the `jsonschema` package is absent, which is the case
/// in the pinned build container. `HEADWATER_SARIF_VALIDATOR` turns that into a
/// failure, so this gate cannot go quiet by losing a dependency.
fn oracle(artifact: &str) -> Option<Vec<String>> {
    let required = std::env::var_os("HEADWATER_SARIF_VALIDATOR").is_some();
    let written = Path::new(env!("CARGO_TARGET_TMPDIR")).join("adapter.sarif");
    std::fs::create_dir_all(written.parent().expect("a parent")).expect("the temporary directory");
    std::fs::write(&written, artifact).expect("cannot write the artifact");
    let ran = Command::new("python3")
        .arg(tests_dir().join("sarif_oracle.py"))
        .arg(tests_dir().join("sarif-schema-2.1.0.json"))
        .arg(&written)
        .output();
    let reason = match ran {
        Ok(output) if output.status.success() => {
            return Some(
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(str::to_string)
                    .collect(),
            )
        }
        Ok(output) => String::from_utf8_lossy(&output.stderr).trim().to_string(),
        Err(error) => error.to_string(),
    };
    assert!(
        !required,
        "HEADWATER_SARIF_VALIDATOR is set and the stock validator did not run, so nothing \
         outside this repository pins the reading of SARIF: {reason}\nInstall it with: \
         pip install jsonschema"
    );
    eprintln!(
        "note: the stock SARIF validator did not run ({reason}), so this test proved nothing"
    );
    None
}

// A reader for the artifacts, which is the YAML loader: JSON is a subset of the
// YAML 1.2 core schema this engine already implements, and a second parser is
// the drift spec 4 names.

use headwater_yaml::Value;

fn parse(artifact: &str) -> Value {
    headwater_yaml::load(artifact)
        .unwrap_or_else(|errors| panic!("the artifact does not parse: {errors:?}"))
        .value
}

fn member(value: &Value, key: &str) -> Option<Value> {
    value
        .as_map()
        .and_then(|map| map.get(key))
        .map(|spanned| spanned.value.clone())
}

fn text(value: &Value, key: &str) -> String {
    match member(value, key) {
        Some(Value::Scalar(scalar)) => headwater_yaml::core_schema::as_str(&scalar).to_string(),
        _ => panic!("no `{key}` member"),
    }
}

fn scalar(value: &Value) -> String {
    match value {
        Value::Scalar(found) => headwater_yaml::core_schema::as_str(found).to_string(),
        _ => panic!("not a scalar"),
    }
}

fn runs(document: &Value) -> Vec<Value> {
    member(document, "runs")
        .expect("the runs")
        .as_seq()
        .expect("an array")
        .iter()
        .map(|spanned| spanned.value.clone())
        .collect()
}

fn results(document: &Value) -> Vec<Value> {
    member(&runs(document)[0], "results")
        .expect("the results")
        .as_seq()
        .expect("an array")
        .iter()
        .map(|spanned| spanned.value.clone())
        .collect()
}

/// A member of one artifact, by path.
fn at<'a>(
    value: &'a headwater_yaml::Spanned<headwater_yaml::Value>,
    path: &[&str],
) -> Option<&'a headwater_yaml::Spanned<headwater_yaml::Value>> {
    let mut found = value;
    for key in path {
        found = found.value.as_map()?.get(key)?;
    }
    Some(found)
}

/// The members a run-level carrier names are the members the artifact writes.
///
/// The census holds the artifact to the declaration. Nothing held the
/// declaration to the artifact, and without this a member could leave the list
/// in silence: the audit would then ask for less, every count would still add
/// up, and the entry would read as held. It is the same weakening that produced
/// #233 one grain out, where the entry named a bag and the bag was there.
///
/// Derived rather than listed. Every entry of every loss set that names members
/// is read out of the declaration, and the names are read out of the emitted
/// bytes. An emitter that adds a member to a declared block fails this too,
/// because a value that reaches an artifact and no declaration is a value the
/// loss set does not account for.
#[test]
fn the_members_a_carrier_names_are_the_members_the_artifact_writes() {
    let mut checked = 0;
    for ran in [fixture_run(), scoped_run()] {
        for format in Format::ALL {
            let artifact = render(&ran, format);
            let Ok(root) = headwater_yaml::load(&artifact) else {
                continue;
            };
            let Some(runs) = at(&root, &["runs"]).and_then(|found| found.value.as_seq()) else {
                continue;
            };
            let object = runs.first().expect("one run");
            for entry in format.loss() {
                let Carrier::Run { places, when } = entry.carrier else {
                    continue;
                };
                if !when(&ran.run) {
                    continue;
                }
                for place in places.iter().filter(|place| !place.members.is_empty()) {
                    let block = at(object, place.at).expect("the block the entry names");
                    let written: BTreeSet<&str> = block
                        .value
                        .as_map()
                        .expect("a mapping")
                        .iter()
                        .map(|member| member.key.value.as_str())
                        .collect();
                    let declared: BTreeSet<&str> = place.members.iter().copied().collect();
                    assert_eq!(declared, written, "{} in {}", entry.field, format.name());
                    checked += 1;
                }
            }
        }
    }
    // Two entries over the full-corpus run and the scoped one, minus the change
    // entry that a full-corpus run does not write.
    assert_eq!(checked, 3);
}
