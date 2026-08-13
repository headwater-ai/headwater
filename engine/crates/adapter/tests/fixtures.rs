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

use headwater_adapter::{reported, Escape, Format, Subject};
use headwater_census::census::{self, Census};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
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
    let bare = run_at(None).run;
    let applied = bare
        .suppressions
        .suppressions
        .iter()
        .find(|suppression| !suppression.hid.is_empty())
        .expect("the fixture tree suppresses something");
    // A pair the directive does *not* cover, so the two inventories both end up
    // with something in them. The runner applies the payload first, so naming
    // the suppressed pair would empty the suppression inventory instead.
    let held = bare
        .findings
        .iter()
        .find(|finding| finding.rule != applied.rule || finding.path != applied.path)
        .expect("the fixture tree reports something");
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
    run_at(Some(&payload))
}

fn run_at(adoption: Option<&Mapping>) -> Ran {
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
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
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
    let ran = fixture_run();
    let lock = lock_digest();
    headwater_adapter::render(&ran.run, &ran.census, &ran.graph, &subject(&lock), format)
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
#[test]
fn every_finding_reaches_every_format() {
    let ran = fixture_run();
    let lock = lock_digest();
    for format in Format::ALL {
        let artifact =
            headwater_adapter::render(&ran.run, &ran.census, &ran.graph, &subject(&lock), format);
        let audited = headwater_adapter::census(&ran.run, &artifact);
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

/// A stock reader validates the document and finds the findings the run found.
///
/// Two directions. A record the reader produces that the run does not have is
/// an invented finding, and a finding of the run that the reader does not
/// produce is a dropped one. Both are valid SARIF, so the schema catches
/// neither, and this is the comparison that does.
#[test]
fn the_stock_reader_finds_the_same_findings_the_run_did() {
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
