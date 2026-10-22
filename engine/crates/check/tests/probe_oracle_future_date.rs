// SPDX-License-Identifier: Apache-2.0
//! The hidden oracle of the probe
//! a-rule-that-reads-the-date-takes-it-from-the-run-and-ships-with-the-fixture-that-fails-it
//! (#1835).
//!
//! This file names that probe in its first lines, so `tools/probe/seal.sh`
//! removes it from every workspace a session of the probe works in, and
//! `tools/probe/grading-tree.sh` puts it back in the grading copy. The
//! `future-date` oracle under `oracles:` in `.headwater/probe.yml` holds it,
//! and runs it with `-- --ignored` over the copy that carries every artifact
//! the session produced.
//!
//! **Why the case is ignored.** On this repository's own tree no rule reports
//! a `status_since` after the date of the run, so the case fails here by
//! design. It passes only over a tree that a session of the probe changed,
//! and CI never runs it. It builds and lints on every change all the same,
//! because it is a target of this crate.
//!
//! The case asserts four things over the tree it is built in:
//!
//! 1. `RULES` keeps every rule of the pin below and holds exactly one rule
//!    more, written in the three-part form `<a>.<b>.<c>`.
//! 2. Over a corpus of one document whose `status_since` is 2031-01-01, a run
//!    whose injected clock reads 2030-06-01 reports that rule on the
//!    document, and a run whose clock reads 2031-06-01 does not.
//! 3. A rule that read the host clock fails (2), because 2031-01-01 is after
//!    the host's date as well, so it reports under both clocks.
//! 4. A document under `fixtures/check/` carries a `status_since` after the
//!    date `tests/fixtures.rs` pins its run at: the fixture the rule fails.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Register, Run, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

/// The rules of the pin, in the order of `RULES` when the probe was written.
/// A session may change the length of the array, so the case reads `RULES`
/// as a slice and compares it with this list.
const PIN: [&str; 49] = [
    "facet.required.missing",
    "facet.value.not_permitted",
    "facet.value.blank",
    "identifier.pattern.not_met",
    "shelf.placement_is_primary",
    "relation.target.unresolved",
    "relation.target.is_source",
    "relation.target.suspect",
    "relation.reciprocity.missing",
    "relation.endpoint.not_permitted",
    "lifecycle.dependency.on_terminal",
    "lifecycle.dependency.on_initial",
    "lifecycle.state.not_set_by_edge",
    "relation.pair.invalid",
    "warrant.evidence.unsupported",
    "relation.participation.overdue",
    "lifecycle.state.set_twice",
    "relation.declaration.unusable",
    "identifier.unusable",
    "identifier.claimed_twice",
    "shelf.discriminator.unresolved",
    "identifier.claim.missing",
    "identifier.claim.stale",
    "voice.forbidden_construction",
    "language.controlled.not_met",
    "language.retired_term.used",
    "surface.local_path.instructed",
    "surface.command.undeclared",
    "language.source_form.not_met",
    "section.required.missing",
    "link.fragment.unresolved",
    "link.path.unresolved",
    "link.identifier.mismatch",
    "warrant.promoted",
    "lifecycle.transition.not_permitted",
    "lifecycle.state.not_admitted",
    "warrant.value.not_permitted",
    "warrant.acceptance.unpaired",
    "lifecycle.deletion.not_permitted",
    "coverage.document_unchecked",
    "obligation.disposition.not_one",
    "control.mechanism.unimplemented",
    "control.observation.invalid",
    "adoption.task.expired",
    "language.outside_root.refused",
    "taxonomy.pin.diverged",
    "harvest.pin.unread",
    "import.pin.unread",
    "relation.target.verification.suspect",
];

/// The `status_since` of the one document of the in-memory corpus.
const FUTURE: &str = "2031-01-01";
/// A clock before that date, under which the rule reports.
const BEFORE: &str = "2030-06-01";
/// A clock after it, under which the rule does not.
const AFTER: &str = "2031-06-01";

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Whether a rule identifier has three parts of lower-case letters and
/// underscores.
fn three_parts(rule: &str) -> bool {
    let parts: Vec<&str> = rule.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_lowercase() || c == '_'))
}

/// A corpus of one decision record of the check fixture taxonomy, whose
/// `status_since` is [`FUTURE`], in a directory of its own.
fn future_corpus() -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "headwater-probe-oracle-future-date-{}",
        std::process::id()
    ));
    let shelf = base.join("check").join("spec");
    std::fs::create_dir_all(&shelf).expect("the corpus directory");
    let document = format!(
        "---\nid: DR-FIX-9001\ndoc_type: decision_record\nstatus: current\nstatus_since: {FUTURE}\nsummary: A record whose state is dated after the run.\n---\n\n# Dated after the run\n\nIts state is dated after the date of the run.\n\n## Context\n\nThe date is later than the injected clock of one run and earlier than the other.\n"
    );
    std::fs::write(shelf.join("00-future.md"), document).expect("the document");
    base
}

fn run_at(base: &Path, now: &str) -> Run {
    let corpus = Corpus::new(base, "check");
    let source = std::fs::read_to_string(crate_dir().join("fixtures/check.taxonomy.yml"))
        .expect("the fixture taxonomy");
    let root = headwater_yaml::load(&source)
        .expect("the fixture taxonomy loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();
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
    headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: "sha256:probe-oracle-future-date",
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            observations: &headwater_check::Observations::empty(),
            pin: None,
            harvests: &[],
            imports: &[],
            adoption: None,
            source: "engine/crates/check/fixtures/check.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(now).expect("a date")),
        &mut Cache::disabled(),
    )
}

/// Whether a run reports a rule on the one document.
fn reports(run: &Run, rule: &str) -> bool {
    run.findings
        .iter()
        .any(|finding| finding.rule == rule && finding.path.ends_with("00-future.md"))
}

/// The date `tests/fixtures.rs` pins its recorded run at.
fn pinned() -> String {
    let source = std::fs::read_to_string(crate_dir().join("tests/fixtures.rs"))
        .expect("tests/fixtures.rs");
    source
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("const PINNED: &str = \"")
                .and_then(|rest| rest.strip_suffix("\";"))
                .map(str::to_string)
        })
        .expect("tests/fixtures.rs pins a date")
}

/// Every `status_since` value of a document under a directory, recursively.
fn dates_under(dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            dates_under(&path, out);
        } else if path.extension().is_some_and(|it| it == "md") {
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            out.extend(text.lines().filter_map(|line| {
                line.strip_prefix("status_since:")
                    .map(|it| it.trim().trim_matches(['"', '\'']).to_string())
            }));
        }
    }
}

#[test]
#[ignore = "the oracle of a code probe; it passes only over a tree a session changed"]
fn the_rule_reports_a_date_after_the_injected_clock_and_ships_a_fixture() {
    // 1. One rule more than the pin, and every rule of the pin kept.
    let rules: &[&str] = &headwater_check::RULES;
    for kept in PIN {
        assert!(rules.contains(&kept), "RULES no longer holds {kept}");
    }
    let added: Vec<&str> = rules
        .iter()
        .copied()
        .filter(|rule| !PIN.contains(rule))
        .collect();
    assert_eq!(added.len(), 1, "RULES adds {added:?} to the pin, and not one rule");
    let rule = added[0];
    assert!(three_parts(rule), "{rule} is not in the form <a>.<b>.<c>");

    // 2 and 3. The injected clock decides, and the host clock cannot.
    let base = future_corpus();
    let before = run_at(&base, BEFORE);
    let after = run_at(&base, AFTER);
    let _ = std::fs::remove_dir_all(&base);
    assert!(
        reports(&before, rule),
        "{rule} does not report a status_since of {FUTURE} under a clock of {BEFORE}"
    );
    assert!(
        !reports(&after, rule),
        "{rule} reports a status_since of {FUTURE} under a clock of {AFTER}, so it does not \
         read the run's clock"
    );

    // 4. The fixture the rule fails.
    let pinned = Date::parse(&pinned()).expect("the pinned date parses");
    let mut dates = Vec::new();
    dates_under(&crate_dir().join("fixtures/check"), &mut dates);
    assert!(
        dates
            .iter()
            .filter_map(|date| Date::parse(date))
            .any(|date| date > pinned),
        "no document under fixtures/check/ carries a status_since after the date \
         tests/fixtures.rs pins"
    );
}
