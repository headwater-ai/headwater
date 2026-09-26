// SPDX-License-Identifier: Apache-2.0
//! The verdict ledger: a rule whose verdicts moved while its `VERSION` did not.
//!
//! `VERSION` is the one component of a cache key that no input supplies
//! ([`headwater_check::scope`] says why). An author raises it by hand when a
//! rule starts to decide something else, and a warm cache keeps serving the
//! earlier edition's verdicts to anyone whose author forgot. That happened to
//! `link.fragment.unresolved`, to `identifier.claim.missing` and to
//! `relation.target.suspect` (#952, #1105). Every other test of this workspace
//! runs under one binary, so none of them could see it: a cold run and a warm
//! run of the same compiled rule agree.
//!
//! `fixtures/editions.ledger` records, for each rule the recorded corpora
//! reach, its `VERSION` and a digest of every verdict it reaches there. The
//! digest is over what a cache would store for each instance: the instance's
//! identity (its corpus, its grain and the paths it read, without their byte
//! hashes) and [`headwater_check::cached_form`] of its outcome. A skip is not
//! cached and is digested as the word `skipped`. So a message, a remediation or
//! a patch that changes moves the digest too, because a cache stores those.
//!
//! The test recomputes the ledger and compares it row by row:
//!
//! - a digest that moved at an unchanged `VERSION` fails, names the rule and
//!   the file that declares it, and **fails under `HEADWATER_BLESS` too**.
//!   `DEVELOPING.md` tells an author to re-record with a blanket
//!   `HEADWATER_BLESS=1 cargo test -p headwater-check`, and a bless that
//!   rewrote this row would ship exactly the #952 defect;
//! - a `VERSION` that moved, with or without a moved digest, and a rule with no
//!   row yet, fail outside bless and are re-recorded under it;
//! - a row whose rule reached no instance fails outside bless, because it pins
//!   nothing, and bless drops it.
//!
//! What the ledger cannot see: a change that no recorded corpus exercises, a
//! rule with no instance on any of them (listed in [`UNCOVERED`] with the
//! reason), and a change visible only against a warm cache that an older
//! binary wrote. The last is the digest over the compiled rule that
//! [HW-OBL-0074](../../../../docs/obligations/0074-a-check-version-is-raised-by-hand-and-nothing-catches-a-stale.md)
//! asks for. The ledger approximates it over fixtures and does not close it.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-check --test editions
//!
//! Read the diff before committing it. A re-recorded row is a rule that now
//! decides something else.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{
    adoption, basis, command, coverage, dependency, endpoint, initial_dependency, reciprocity,
    register, surface, suspect, target, verification, Cache, Context, Date, Declared, Observation,
    Observations, Outcome, Register, Run, Shape, RULES,
};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_yaml::Mapping;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The rules whose instance is an edge. The issue this ledger exists for is
/// one of them, and each must pin at least one verdict.
const EDGE_RULES: [&str; 8] = [
    target::RULE,
    suspect::RULE,
    reciprocity::RULE,
    endpoint::RULE,
    dependency::RULE,
    initial_dependency::RULE,
    basis::RULE,
    verification::RULE,
];

/// The rules no recorded corpus reaches, each with the reason. A rule here has
/// no ledger row, so a change to what it decides passes this test. The list is
/// held honest from both sides: a rule that gains an instance must leave it.
const UNCOVERED: &[(&str, &str)] = &[
    (
        surface::RULE,
        "its cases are unit tests in src/surface.rs, and no recorded corpus declares a \
         `surface` and an adopter shelf",
    ),
    (
        command::RULE,
        "its cases are unit tests in src/command.rs, and no recorded corpus declares a \
         `surface` and an adopter shelf",
    ),
    (
        coverage::RULE,
        "the runner reaches it outside any instance, so no cache entry holds its verdict",
    ),
    (
        register::DISPOSITION,
        "the runner reaches it from the taxonomy outside any instance, so no cache entry \
         holds its verdict",
    ),
    (
        register::MECHANISM,
        "the runner reaches it from the taxonomy outside any instance, so no cache entry \
         holds its verdict",
    ),
    (
        register::OBSERVATION,
        "the runner reaches it from the snapshot outside any instance, so no cache entry \
         holds its verdict",
    ),
    (
        adoption::RULE,
        "the runner reaches it from the lock outside any instance, so no cache entry holds \
         its verdict",
    ),
];

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn ledger_path() -> PathBuf {
    fixtures_dir().join("editions.ledger")
}

fn load_map(path: &Path) -> Mapping {
    let source = std::fs::read_to_string(path).expect("the fixture taxonomy");
    headwater_yaml::load(&source)
        .expect("the fixture taxonomy loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone()
}

/// What one recorded corpus is run with. Each keeps the clock and the lock
/// string of the test that owns the corpus, so a verdict here is the verdict
/// that test records.
struct Recorded {
    /// The name a ledger message prints, relative to `fixtures/`.
    label: &'static str,
    base: PathBuf,
    name: &'static str,
    taxonomy: &'static str,
    lock: String,
    clock: &'static str,
    observations: Observations,
    /// Whether the run reads `check-rule` anchors and the claim store under
    /// the base, as `tests/fixtures.rs` does for the check tree.
    full: bool,
}

fn recorded() -> Vec<Recorded> {
    let check_source = std::fs::read_to_string(fixtures_dir().join("check.taxonomy.yml"))
        .expect("the fixture taxonomy");
    vec![
        Recorded {
            label: "check",
            base: fixtures_dir(),
            name: "check",
            taxonomy: "check.taxonomy.yml",
            lock: headwater_hash::hex(check_source.as_bytes()),
            clock: "2026-08-12",
            observations: Observations::empty(),
            full: true,
        },
        Recorded {
            label: "terminal-dependency",
            base: fixtures_dir(),
            name: "terminal-dependency",
            taxonomy: "terminal-dependency.taxonomy.yml",
            lock: "sha256:terminal-dependency-fixture".to_string(),
            clock: "2026-08-12",
            observations: Observations::empty(),
            full: false,
        },
        Recorded {
            label: "state-set-twice",
            base: fixtures_dir(),
            name: "state-set-twice",
            taxonomy: "state-set-twice.taxonomy.yml",
            lock: "sha256:state-set-twice-fixture".to_string(),
            clock: "2026-08-12",
            observations: Observations::empty(),
            full: false,
        },
        Recorded {
            label: "acceptance-criterion-proven",
            base: fixtures_dir(),
            name: "acceptance-criterion-proven",
            taxonomy: "acceptance-criterion-proven.taxonomy.yml",
            lock: "sha256:verification-suspect-fixture".to_string(),
            clock: "2026-08-12",
            // A snapshot whose digest is not the criterion's, so the rule
            // reaches its suspect arm and a finding with a message.
            observations: Observations::of(vec![Observation::Verification {
                verification: "ACP-FIX-verification-one".to_string(),
                commit: "788885a9".to_string(),
                criterion_digest: "sha256:0".to_string(),
            }]),
            full: false,
        },
        Recorded {
            label: "editions/governs-suspect",
            base: fixtures_dir().join("editions"),
            name: "governs-suspect",
            taxonomy: "governs-suspect.taxonomy.yml",
            lock: "sha256:editions-governs-suspect".to_string(),
            clock: "2026-09-24",
            observations: Observations::empty(),
            full: false,
        },
    ]
}

fn run(recorded: &Recorded) -> Run {
    let corpus = Corpus::new(recorded.base.clone(), recorded.name);
    let root = load_map(&fixtures_dir().join(recorded.taxonomy));
    let taxonomy = Taxonomy::read(&root).expect("the taxonomy reads");
    let declarations = Declarations::read(&root).expect("the declarations read");
    let register = Register::read(&root).expect("the register reads");
    let shape = Shape::read(&root).expect("the shape reads");
    let taken = census::take(&corpus, &taxonomy);
    let config = Config::default();
    let resolvers = if recorded.full {
        Resolvers::over(&corpus)
            .with(Box::new(headwater_check::anchors::Rules::shipped()))
            .expect("the check-rule resolver is the only one of its name")
    } else {
        Resolvers::over(&corpus)
    };
    let claims = if recorded.full {
        headwater_check::claim::Claims::at(&corpus.base)
    } else {
        headwater_check::claim::Claims::empty()
    };
    let graph = Graph::build(&taken, &declarations, &resolvers, &corpus, &config);
    let source = format!("engine/crates/check/fixtures/{}", recorded.taxonomy);
    headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: &recorded.lock,
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            observations: &recorded.observations,
            adoption: None,
            source: &source,
        },
        &claims,
        &Context::at(Date::parse(recorded.clock).expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

/// One rule's row: what the ledger records, and the corpora it was read over.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Row {
    rule: String,
    version: u32,
    digest: String,
    instances: usize,
    /// Where the rule reached a verdict. Not written to the ledger, and only
    /// for a message to name.
    corpora: Vec<String>,
}

/// The rows the recorded corpora give under this binary, sorted by rule.
fn compute() -> Vec<Row> {
    let mut versions: BTreeMap<&'static str, u32> = BTreeMap::new();
    let mut lines: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();
    let mut corpora: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();
    for recorded in recorded() {
        let ran = run(&recorded);
        for (rule, version) in &ran.read_set.versions {
            let earlier = versions.insert(rule, *version);
            assert!(
                earlier.is_none_or(|earlier| earlier == *version),
                "{rule} reported two versions in one binary"
            );
        }
        for instance in &ran.instances {
            let reads: Vec<&str> = instance.reads.iter().map(|i| i.path.as_str()).collect();
            let verdict = headwater_check::cached_form(&instance.outcome).unwrap_or_else(|| {
                debug_assert!(matches!(instance.outcome, Outcome::Skipped(_)));
                "skipped".to_string()
            });
            lines.entry(instance.rule).or_default().push(format!(
                "{}\t{:?}\t{}\t{verdict}",
                recorded.label,
                instance.grain,
                reads.join(",")
            ));
            let seen = corpora.entry(instance.rule).or_default();
            if seen.last().map(String::as_str) != Some(recorded.label) {
                seen.push(recorded.label.to_string());
            }
        }
    }
    lines
        .into_iter()
        .map(|(rule, mut instances)| {
            instances.sort_unstable();
            let mut text = instances.join("\n");
            text.push('\n');
            Row {
                rule: rule.to_string(),
                version: *versions
                    .get(rule)
                    .unwrap_or_else(|| panic!("{rule} reached a verdict and reported no version")),
                digest: headwater_hash::digest(text.as_bytes()),
                instances: instances.len(),
                corpora: corpora.remove(rule).unwrap_or_default(),
            }
        })
        .collect()
}

fn computed() -> &'static [Row] {
    static ROWS: OnceLock<Vec<Row>> = OnceLock::new();
    ROWS.get_or_init(compute)
}

const HEADER: &str = "\
# The verdict ledger of tests/editions.rs: rule, VERSION, digest of every
# verdict over the recorded corpora, instance count. Sorted by rule.
# HEADWATER_BLESS=1 re-records a row whose VERSION moved and never a row whose
# digest moved at the same VERSION: raise VERSION for that.
";

fn render(rows: &[Row]) -> String {
    let mut text = HEADER.to_string();
    for row in rows {
        text.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            row.rule, row.version, row.digest, row.instances
        ));
    }
    text
}

/// A ledger as rows, and a line it cannot read as a failure.
fn parse(text: &str) -> (Vec<Row>, Vec<String>) {
    let mut rows = Vec::new();
    let mut failures = Vec::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        match fields.as_slice() {
            [rule, version, digest, instances] => match (version.parse(), instances.parse()) {
                (Ok(version), Ok(instances)) => rows.push(Row {
                    rule: rule.to_string(),
                    version,
                    digest: digest.to_string(),
                    instances,
                    corpora: Vec::new(),
                }),
                _ => failures.push(format!("the ledger line `{line}` does not read")),
            },
            _ => failures.push(format!("the ledger line `{line}` does not read")),
        }
    }
    (rows, failures)
}

/// The file that declares a rule, found by its name as a string literal, so
/// that no second table of rules sits beside [`RULES`].
fn declaring_file(rule: &str) -> String {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let literal = format!("\"{rule}\"");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&source)
        .expect("the source directory reads")
        .map(|entry| entry.expect("a directory entry").path())
        .filter(|path| path.extension().is_some_and(|e| e == "rs"))
        .collect();
    files.sort();
    files
        .into_iter()
        .find(|path| {
            std::fs::read_to_string(path).is_ok_and(|text| {
                text.lines()
                    .any(|line| line.contains("const") && line.contains(&literal))
            })
        })
        .map(|path| {
            format!(
                "engine/crates/check/src/{}",
                path.file_name().expect("a file name").to_string_lossy()
            )
        })
        .unwrap_or_else(|| "the module that declares it".to_string())
}

/// What a comparison decided: the ledger to write under bless, and every
/// failure. Under bless the ledger is written and the failures still fail.
#[derive(Debug)]
struct Judgement {
    ledger: String,
    failures: Vec<String>,
}

/// Compare a recorded ledger with the rows this binary gives.
///
/// A pure function of its arguments, because a compiled `VERSION` cannot
/// change inside one process: the tests below stand a hand-made ledger in for
/// the one an older binary recorded, as `tests/cache.rs` stands in a second
/// rule list for an upgrade.
fn judge(recorded: &str, computed: &[Row], bless: bool) -> Judgement {
    let (rows, mut failures) = parse(recorded);
    let mut old: BTreeMap<&str, &Row> = rows.iter().map(|row| (row.rule.as_str(), row)).collect();
    let mut kept = Vec::new();
    for row in computed {
        match old.remove(row.rule.as_str()) {
            None => {
                if !bless {
                    failures.push(format!(
                        "{} has no row in the ledger: record it with HEADWATER_BLESS=1",
                        row.rule
                    ));
                }
                kept.push(row.clone());
            }
            Some(was) if was.version == row.version && was.digest != row.digest => {
                failures.push(format!(
                    "{} changed its verdicts over {} at VERSION {}, and VERSION did not move: \
                     raise it in {}, then re-record with HEADWATER_BLESS=1. A warm cache keeps \
                     serving what VERSION {} decided, and HEADWATER_BLESS does not re-record \
                     this row",
                    row.rule,
                    row.corpora.join(", "),
                    row.version,
                    declaring_file(&row.rule),
                    row.version,
                ));
                kept.push(was.clone());
            }
            Some(was) if was.version != row.version || was.instances != row.instances => {
                if !bless {
                    failures.push(format!(
                        "{} is at VERSION {} and the ledger records VERSION {}: re-record with \
                         HEADWATER_BLESS=1",
                        row.rule, row.version, was.version
                    ));
                }
                kept.push(row.clone());
            }
            Some(was) => kept.push(was.clone()),
        }
    }
    for (rule, _) in old {
        if !bless {
            failures.push(format!(
                "{rule} has a row in the ledger and reached no verdict on any recorded corpus, so \
                 the row pins nothing: re-record with HEADWATER_BLESS=1, and list the rule in \
                 UNCOVERED with the reason"
            ));
        }
    }
    kept.sort_by(|a, b| a.rule.cmp(&b.rule));
    Judgement {
        ledger: render(&kept),
        failures,
    }
}

fn bless() -> bool {
    std::env::var_os("HEADWATER_BLESS").is_some()
}

/// The standing test: the ledger on disk against this binary.
#[test]
fn every_ledgered_rule_decides_what_its_version_recorded() {
    let path = ledger_path();
    let recorded = std::fs::read_to_string(&path).unwrap_or_default();
    let judgement = judge(&recorded, computed(), bless());
    if bless() && judgement.ledger != recorded {
        std::fs::write(&path, &judgement.ledger).expect("cannot write the ledger");
    }
    assert!(
        judgement.failures.is_empty(),
        "\n{}\n",
        judgement.failures.join("\n")
    );
}

/// Every rule is ledgered or listed as uncovered with a reason, and never both.
/// Every edge rule is ledgered.
#[test]
fn every_rule_is_ledgered_or_named_as_uncovered() {
    let ledgered: Vec<&str> = computed().iter().map(|row| row.rule.as_str()).collect();
    let uncovered: Vec<&str> = UNCOVERED.iter().map(|(rule, _)| *rule).collect();
    for rule in EDGE_RULES {
        assert!(
            ledgered.contains(&rule),
            "the edge rule {rule} reaches no verdict on any recorded corpus"
        );
    }
    for rule in RULES {
        match (ledgered.contains(&rule), uncovered.contains(&rule)) {
            (true, true) => panic!("{rule} reaches a verdict now: take it out of UNCOVERED"),
            (false, false) => panic!(
                "{rule} reaches no verdict on any recorded corpus: add a fixture that reaches \
                 it, or list it in UNCOVERED with the reason"
            ),
            _ => {}
        }
    }
    for (rule, reason) in UNCOVERED {
        assert!(RULES.contains(rule), "{rule} in UNCOVERED is not a rule");
        assert!(!reason.is_empty(), "{rule} in UNCOVERED gives no reason");
    }
}

/// The decisive case, #952 as data: the ledger an older binary wrote holds
/// `relation.target.suspect` at the version this binary runs, with a digest
/// this binary does not reach. That fails, names the rule, and fails under
/// bless too, where the row stays as it was.
#[test]
fn a_moved_digest_at_an_unchanged_version_fails_and_bless_keeps_the_row() {
    let rows = computed();
    let row = rows
        .iter()
        .find(|row| row.rule == suspect::RULE)
        .expect("relation.target.suspect is ledgered");
    let stale = "sha256:recorded-by-an-older-binary";
    let older: Vec<Row> = rows
        .iter()
        .map(|r| {
            let mut r = r.clone();
            if r.rule == suspect::RULE {
                r.digest = stale.to_string();
            }
            r
        })
        .collect();
    let ledger = render(&older);

    for bless in [false, true] {
        let judgement = judge(&ledger, rows, bless);
        assert_eq!(judgement.failures.len(), 1, "{:?}", judgement.failures);
        let message = &judgement.failures[0];
        assert!(message.contains(suspect::RULE), "{message}");
        assert!(
            message.contains(&format!("VERSION {}", row.version)),
            "{message}"
        );
        assert!(
            message.contains("engine/crates/check/src/suspect.rs"),
            "{message}"
        );
        assert!(message.contains("editions/governs-suspect"), "{message}");
        assert_eq!(
            judgement.ledger, ledger,
            "bless={bless} rewrote a row whose digest moved at an unchanged VERSION"
        );
    }
}

/// A raised `VERSION` with an unchanged digest asks for a re-record outside
/// bless, and bless re-records it. Raising a version conservatively is not a
/// defect.
#[test]
fn a_raised_version_asks_for_a_re_record_and_bless_takes_it() {
    let rows = computed();
    let older: Vec<Row> = rows
        .iter()
        .map(|r| {
            let mut r = r.clone();
            if r.rule == suspect::RULE {
                r.version -= 1;
            }
            r
        })
        .collect();
    let ledger = render(&older);

    let judgement = judge(&ledger, rows, false);
    assert_eq!(judgement.failures.len(), 1, "{:?}", judgement.failures);
    assert!(judgement.failures[0].contains(suspect::RULE));
    assert!(judgement.failures[0].contains("re-record"));

    let judgement = judge(&ledger, rows, true);
    assert!(judgement.failures.is_empty(), "{:?}", judgement.failures);
    assert_eq!(judgement.ledger, render(rows));
}

/// A row whose rule reached nothing pins nothing, and a rule with no row is
/// recorded only under bless.
#[test]
fn a_row_that_pins_nothing_and_a_rule_with_no_row_fail_outside_bless() {
    let rows = computed();
    let mut ledger = render(&rows[1..]);
    ledger.push_str("no.such.rule\t1\tsha256:0\t1\n");

    let judgement = judge(&ledger, rows, false);
    assert_eq!(judgement.failures.len(), 2, "{:?}", judgement.failures);
    assert!(judgement
        .failures
        .iter()
        .any(|f| f.contains("no.such.rule") && f.contains("pins nothing")));
    assert!(judgement
        .failures
        .iter()
        .any(|f| f.contains(&rows[0].rule) && f.contains("no row")));

    let judgement = judge(&ledger, rows, true);
    assert!(judgement.failures.is_empty(), "{:?}", judgement.failures);
    assert_eq!(judgement.ledger, render(rows));
}
