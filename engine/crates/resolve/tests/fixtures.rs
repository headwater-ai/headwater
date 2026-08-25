// SPDX-License-Identifier: Apache-2.0
//! The resolver's fixture corpus.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots)
//! asks this component for two things by name: "The resolver carries its own
//! round-trip and confluence fixtures." Both are here, and neither is an
//! assertion in Rust that a later rewrite could quietly weaken.
//!
//! Each case under `fixtures/cases/` is a directory: `base.yml` is the package,
//! every `overlay-*.yml` beside it is an overlay in name order, and
//! `expected.record` is the resolved taxonomy or the refusal an author reads.
//!
//! - **`expected.record`** — what the case resolves to, or why it does not.
//! - **`confluence.record`** — every permutation of every case's overlay set,
//!   resolved, and whether the results agree. This is the property that spec 2
//!   requires and that the static check claims to prove, tested by running it.
//! - **`roundtrip.record`** — the resolved taxonomy written out, loaded again
//!   as a package with no overlays, and resolved again. A resolver whose result
//!   is not a taxonomy source has produced something no lock can carry.
//! - **`corpus.resolve`** and **`corpus.taxonomy`** — this repository's own
//!   three sources, which is the resolution that `headwater check` runs on.
//!
//! To re-record after a deliberate change:
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-resolve --test fixtures
//!
//! Read the diff before committing it. A blessed fixture is the change.

use headwater_resolve::{render_errors, resolve, Role, Source};
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

fn compare(path: &Path, actual: &str) {
    if blessing() {
        std::fs::write(path, actual).expect("cannot write the expectation");
        return;
    }
    let expected = std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!(
            "{}: {e}. Run with HEADWATER_BLESS=1 to record it.",
            path.display()
        )
    });
    assert_eq!(expected, actual, "\n{} is out of date", path.display());
}

fn cases() -> Vec<PathBuf> {
    under("cases")
}

fn under(directory: &str) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(fixtures_dir().join(directory))
        .expect("the case directory")
        .map(|entry| entry.expect("a directory entry").path())
        .filter(|path| path.is_dir())
        .collect();
    found.sort();
    assert!(!found.is_empty(), "no cases");
    found
}

fn name(case: &Path) -> String {
    case.file_name().expect("a name").to_string_lossy().into()
}

/// The base package first, then every overlay in file-name order.
fn sources(case: &Path) -> Vec<Source> {
    let base = case.join("base.yml");
    let mut out =
        vec![Source::read(&base, "base.yml", Role::Taxonomy).expect("the base package loads")];
    let mut overlays: Vec<PathBuf> = std::fs::read_dir(case)
        .expect("a case directory")
        .map(|entry| entry.expect("a directory entry").path())
        .filter(|path| {
            path.file_name()
                .is_some_and(|found| found.to_string_lossy().starts_with("overlay-"))
        })
        .collect();
    overlays.sort();
    for overlay in overlays {
        let shown = overlay.file_name().unwrap().to_string_lossy().to_string();
        out.push(Source::read(&overlay, &shown, Role::Overlay).expect("an overlay loads"));
    }
    out
}

/// The resolved taxonomy, or the refusal, as one text.
fn outcome(sources: &[Source]) -> String {
    match resolve(sources) {
        Ok(resolution) => resolution.render(),
        Err(errors) => format!("refused\n\n{}", render_errors(&errors)),
    }
}

#[test]
fn every_case_resolves_to_the_recorded_result() {
    for case in cases() {
        compare(&case.join("expected.record"), &outcome(&sources(&case)));
    }
}

/// The property spec 2 requires: "the application of a set of overlays in any
/// legal order yields the same resolved taxonomy".
///
/// Every permutation of every case's overlay set is resolved, and the results
/// are held to each other. A case that the static check refuses has to be
/// refused in every order too, because a conflict that only one order finds is
/// a conflict that a release can ship.
#[test]
fn no_permutation_of_an_overlay_set_changes_the_result() {
    let mut out = String::new();
    for case in cases() {
        let sources = sources(&case);
        let (base, overlays) = sources.split_first().expect("a base");
        let orders = permutations(overlays.len());

        let mut results = Vec::new();
        for order in &orders {
            let mut set = vec![base.clone()];
            set.extend(order.iter().map(|index| overlays[*index].clone()));
            results.push(outcome(&set));
        }

        let refused = results[0].starts_with("refused");
        let agree = if refused {
            results.iter().all(|found| found.starts_with("refused"))
        } else {
            results.iter().all(|found| *found == results[0])
        };
        assert!(
            agree,
            "{} resolves differently under two orders",
            name(&case)
        );

        out.push_str(&format!(
            "{}: {} permutation{}, all {}\n",
            name(&case),
            orders.len(),
            if orders.len() == 1 { "" } else { "s" },
            if refused {
                "refused"
            } else {
                "resolve to one taxonomy"
            }
        ));
    }
    compare(&fixtures_dir().join("confluence.record"), &out);
}

/// What each case's overlays make, rather than reach into.
///
/// [`headwater_resolve::Founding`] is recorded by the merge and printed by two
/// verbs, and a reading that fires where it should not is worse than one that
/// never fires. The record holds every case at once, so a change to
/// [`headwater_resolve::apply`] that started founding on an address whose
/// parent is there moves a line here rather than passing in silence.
///
/// The counts are the ones the declared overlay order produces, which is the
/// order `sources` reads and the order a consumer declaration selects. A pair
/// that commutes can record a founding in one order and not the other, and
/// `founded.rs` is where that is a case.
#[test]
fn every_case_records_what_its_overlays_make() {
    let mut out = String::new();
    for case in cases() {
        let Ok(resolution) = resolve(&sources(&case)) else {
            out.push_str(&format!("{}: refused, so nothing is made\n", name(&case)));
            continue;
        };
        let founded = &resolution.founded;
        out.push_str(&match founded.len() {
            1 => format!("{}: 1 operation makes what it addresses\n", name(&case)),
            count => format!(
                "{}: {count} operations make what they address\n",
                name(&case)
            ),
        });
        for founding in founded {
            out.push_str(&format!(
                "  {}: {}\n",
                resolution.sources[founding.source],
                founding.sentence()
            ));
        }
    }
    compare(&fixtures_dir().join("founded.record"), &out);
}

/// The round trip: a resolved taxonomy is a taxonomy source.
///
/// The result is written out, loaded again as a package with no overlays, and
/// resolved again. If the two texts differ, the resolver has produced something
/// that only it can read, and the lock that [#51](https://github.com/headwater-ai/headwater/issues/51)
/// writes would not be the artifact every later verdict reads.
#[test]
fn a_resolved_taxonomy_resolves_to_itself() {
    let mut out = String::new();
    for case in cases() {
        let Ok(first) = resolve(&sources(&case)) else {
            out.push_str(&format!(
                "{}: refused, so nothing to write out\n",
                name(&case)
            ));
            continue;
        };
        let written = first.render();
        let again = Source::from_text("the resolved taxonomy", Role::Taxonomy, &written)
            .expect("the written taxonomy loads");
        let second = resolve(&[again]).expect("the written taxonomy resolves");
        assert_eq!(
            written,
            second.render(),
            "{} does not survive a round trip",
            name(&case)
        );
        out.push_str(&format!(
            "{}: {} lines, identical after a round trip\n",
            name(&case),
            written.lines().count()
        ));
    }
    compare(&fixtures_dir().join("roundtrip.record"), &out);
}

/// The rules of `taxonomy validate` that read a resolved taxonomy.
///
/// [Spec 12](../../../../docs/spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship):
/// "every check ships with at least one fixture that it fails and one that it
/// passes." `validate/valid/` is the passing side for every rule at once, and
/// each other directory is a mutation of it that trips a named group.
///
/// A case here resolves and then validates, which is the order the lock writer
/// uses. Every case resolves, because a taxonomy that fails to resolve never
/// reaches these rules and the merge cases already cover that.
#[test]
fn every_validation_case_reports_the_recorded_findings() {
    for case in under("validate") {
        let resolution = resolve(&sources(&case))
            .unwrap_or_else(|errors| panic!("{}: {}", name(&case), render_errors(&errors)));
        let findings = resolution.validate();
        let recorded = if findings.is_empty() {
            String::from("valid\n")
        } else {
            render_errors(&findings)
        };
        compare(&case.join("expected.record"), &recorded);
    }
}

/// Every rule that runs on the result is failed by at least one case.
///
/// The record above is what a reader reads, and this is what stops a rule from
/// quietly dropping out of the set. A rule that no fixture fails is a rule that
/// could return an empty vector for any reason and nothing would notice.
#[test]
fn every_rule_that_runs_here_has_a_case_that_fails_it() {
    use headwater_resolve::Ran;

    let mut fired: Vec<&str> = Vec::new();
    for case in under("validate") {
        let Ok(resolution) = resolve(&sources(&case)) else {
            continue;
        };
        for finding in resolution.validate() {
            if let headwater_resolve::ResolveErrorKind::Invalid { rule, .. } = finding.kind {
                if !fired.contains(&rule) {
                    fired.push(rule);
                }
            }
        }
    }

    let expected: Vec<&str> = headwater_resolve::RULES
        .iter()
        .filter(|(_, ran)| matches!(ran, Ran::Resolved(_) | Ran::Partly { .. }))
        .map(|(rule, _)| *rule)
        .collect();
    let missing: Vec<&&str> = expected
        .iter()
        .filter(|rule| !fired.contains(rule))
        .collect();
    assert!(
        missing.is_empty(),
        "no fixture fails {missing:?}, so nothing holds {} to its own report",
        if missing.len() == 1 { "it" } else { "them" }
    );
}

/// This repository's own resolution, which is the one `headwater check` runs.
///
/// Two records at two grains, for the reason the census and the graph keep two.
/// `corpus.taxonomy` is the whole resolved taxonomy, and it moves only when a
/// taxonomy source moves, which is the event that deserves a reader.
/// `corpus.resolve` is the accounting: the sources, every operation applied,
/// and what the resolver ran and did not run.
#[test]
fn this_repository_resolves_to_the_recorded_taxonomy() {
    let root = repository_root();
    let repository = headwater_resolve::repository(&root)
        .unwrap_or_else(|errors| panic!("{}", render_errors(&errors)));

    let mut out = String::new();
    out.push_str(&format!(
        "{} {}\n\n",
        repository.consumer.package, repository.consumer.version
    ));

    out.push_str("sources, in application order\n");
    for source in &repository.resolution.sources {
        out.push_str(&format!("  {source}\n"));
    }

    out.push_str("\noperations\n");
    for operation in &repository.resolution.operations {
        out.push_str(&format!(
            "  {} ({})\n",
            operation.at(),
            repository.resolution.sources[operation.source]
        ));
    }

    out.push_str("\ndeclarations, by member count\n");
    for entry in &repository.resolution.taxonomy {
        let plural = |count: usize, word: &str| {
            format!("{count} {word}{}", if count == 1 { "" } else { "s" })
        };
        let members = match &entry.value.value {
            headwater_yaml::Value::Map(map) => plural(map.len(), "member"),
            headwater_yaml::Value::Seq(items) => plural(items.len(), "item"),
            headwater_yaml::Value::Scalar(scalar) => scalar.text.clone(),
        };
        out.push_str(&format!("  {}: {members}\n", entry.key.value));
    }

    let findings = repository.resolution.validate();
    out.push_str(&format!(
        "\n`taxonomy validate`: {}\n",
        if findings.is_empty() {
            "valid".to_string()
        } else {
            format!("\n{}", render_errors(&findings))
        }
    ));

    out.push_str("\nrule by rule\n");
    out.push_str(&headwater_resolve::rules::render());

    compare(&fixtures_dir().join("corpus.resolve"), &out);
    compare(
        &fixtures_dir().join("corpus.taxonomy"),
        &repository.resolution.render(),
    );
}

/// The one thing no stand-in ever did.
///
/// The base writes `facets.status.values: $vocabularies.lifecycle_state`, and
/// neither stand-in resolved a reference, so neither ever produced a facet with
/// a value set. Nothing downstream noticed, because no check reads a facet's
/// values yet. This is the assertion that says it happens now.
#[test]
fn the_state_facet_of_this_repository_carries_the_vocabulary_it_reads() {
    let repository = headwater_resolve::repository(&repository_root()).expect("resolves");
    let values = headwater_resolve::merge::lookup(
        &repository.resolution.taxonomy,
        &["facets".into(), "status".into(), "values".into()],
    )
    .expect("the state facet declares its values");
    assert_eq!(
        values.value.as_seq().expect("a value set").len(),
        5,
        "the five lifecycle states of the base package"
    );
}

/// Every permutation of `count` items, as index lists. Heap's algorithm, which
/// is short and which nothing here needs to be faster than.
fn permutations(count: usize) -> Vec<Vec<usize>> {
    let mut current: Vec<usize> = (0..count).collect();
    let mut out = vec![current.clone()];
    let mut counters = vec![0; count];
    let mut index = 0;
    while index < count {
        if counters[index] < index {
            let swap = if index % 2 == 0 { 0 } else { counters[index] };
            current.swap(swap, index);
            out.push(current.clone());
            counters[index] += 1;
            index = 0;
        } else {
            counters[index] = 0;
            index += 1;
        }
    }
    out
}
