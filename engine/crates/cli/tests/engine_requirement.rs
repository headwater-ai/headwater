// SPDX-License-Identifier: Apache-2.0
//! The engine range a corpus asks for, read on every verb that reads it (#1600).
//!
//! # The defect this target exists for
//!
//! A corpus could not say which engine it needs. A gate run by an engine the
//! corpus did not ask for passed or failed by accident, and nothing said which
//! engine the corpus had been written for. The consumer declaration now carries
//! one top-level line, `engine: "<range>"`, in the syntax of a package's
//! `requires_engine` (spec 7, Consuming).
//!
//! # The cases
//!
//! The first case is the decisive one. An engine outside the range prints one
//! line on standard error that names both, `check` exits as before, and
//! `check --strict` fails. A range that admits this engine prints nothing. The
//! second case holds this repository's own declaration to the workspace
//! version, so a release bump that leaves the range behind fails here and not
//! in the commit gate of every clone. The third case holds `taxonomy resolve`
//! to the overlap of the corpus range and the package range.

mod common;

use common::Root;
use headwater_resolve::release::{self, satisfies};
use std::path::Path;

/// The phrase the one line carries, so a case can count the lines.
const ASKS: &str = "this corpus asks for engine";

/// Replace the top-level `engine:` line of the declaration, or remove it.
///
/// The line is written after the `taxonomy:` block, where `init` writes it.
fn set_engine(at: &Path, range: Option<&str>) {
    let path = at.join(".headwater/taxonomy.yml");
    let text = std::fs::read_to_string(&path).expect("the declaration reads");
    let mut kept: String = text
        .lines()
        .filter(|line| !line.starts_with("engine:"))
        .map(|line| format!("{line}\n"))
        .collect();
    if let Some(range) = range {
        kept = kept.replacen("\ncorpus:", &format!("\nengine: \"{range}\"\n\ncorpus:"), 1);
        assert!(kept.contains("\nengine: "), "the declaration has a corpus block");
    }
    std::fs::write(&path, kept).expect("the declaration writes");
}

/// The caret range of this engine, built by the rule `init` writes.
fn caret() -> String {
    let (major, minor, _) = release::parts(release::ENGINE).expect("this engine has a version");
    if major == 0 {
        format!(">={major}.{minor} <{major}.{}", minor + 1)
    } else {
        format!(">={major}.{minor} <{}", major + 1)
    }
}

/// The lines of standard error that report the requirement.
fn reports(err: &str) -> Vec<&str> {
    err.lines().filter(|line| line.contains(ASKS)).collect()
}

/// An engine outside the range the corpus asks for is reported on every run,
/// and `check --strict` fails on it. Standard output does not move.
#[test]
fn an_engine_outside_the_corpus_range_is_reported_and_fails_a_strict_check() {
    let root = Root::new("engine-requirement-a");

    set_engine(&root.at, None);
    let plain = root.run(&["check", "--strict"]);
    assert_eq!(reports(&plain.err), Vec::<&str>::new(), "{plain:?}");
    assert_eq!(plain.code, Some(0), "the fixture is clean: {plain:?}");

    set_engine(&root.at, Some(">=99"));
    for pass in ["cold", "warm"] {
        let strict = root.run(&["check", "--strict"]);
        assert_eq!(strict.code, Some(1), "{pass}: {strict:?}");
        let lines = reports(&strict.err);
        assert_eq!(lines.len(), 1, "{pass}: one line: {}", strict.err);
        assert!(lines[0].contains("\">=99\""), "{pass}: {}", lines[0]);
        assert!(lines[0].contains(release::ENGINE), "{pass}: {}", lines[0]);
        assert!(
            lines[0].contains(".headwater/taxonomy.yml"),
            "{pass}: {}",
            lines[0]
        );
        assert_eq!(strict.out, plain.out, "{pass}: standard output does not move");
    }

    let advisory = root.run(&["check"]);
    assert_eq!(advisory.code, Some(0), "{advisory:?}");
    let lines = reports(&advisory.err);
    assert_eq!(lines.len(), 1, "one line: {}", advisory.err);
    assert!(lines[0].contains("\">=99\"") && lines[0].contains(release::ENGINE));

    set_engine(&root.at, Some(&caret()));
    for arguments in [&["check", "--strict"][..], &["check"][..]] {
        let admitted = root.run(arguments);
        assert_eq!(admitted.code, Some(0), "{arguments:?}: {admitted:?}");
        assert_eq!(reports(&admitted.err), Vec::<&str>::new(), "{admitted:?}");
    }
}

/// A range this engine cannot read is refused, never read as no range.
#[test]
fn an_unreadable_corpus_range_is_refused() {
    let root = Root::new("engine-requirement-unreadable");
    set_engine(&root.at, Some("~0.5"));
    let ran = root.run(&["check", "--strict"]);
    assert_ne!(ran.code, Some(0), "{ran:?}");
    assert!(ran.err.contains("engine: \"~0.5\""), "{}", ran.err);
}

/// This repository asks for the engine it builds. A bump of the workspace
/// version past the range fails here, and the release guide says to move the
/// line in the same commit.
#[test]
fn this_repository_asks_for_the_engine_it_builds() {
    let declaration = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../.headwater/taxonomy.yml");
    let text = std::fs::read_to_string(&declaration).expect("the declaration reads");
    let ranges: Vec<&str> = text
        .lines()
        .filter_map(|line| line.strip_prefix("engine:"))
        .map(|value| value.trim().trim_matches('"'))
        .collect();
    assert_eq!(ranges.len(), 1, ".headwater/taxonomy.yml declares one `engine:` line");
    assert_eq!(
        satisfies(ranges[0], release::ENGINE),
        Ok(true),
        ".headwater/taxonomy.yml asks for engine \"{}\", and the workspace is {}: move the \
         `engine:` line with the version bump (docs/process/how-to/cut-a-release.md, step 1)",
        ranges[0],
        release::ENGINE
    );
}

/// A resolve refuses a corpus range that admits no engine the package admits,
/// and names both ranges. A range inside the package's resolves.
#[test]
fn a_resolve_refuses_a_corpus_range_the_package_range_does_not_meet() {
    let root = Root::new("engine-requirement-c");
    set_engine(&root.at, Some(">=2"));
    let refused = root.run(&["taxonomy", "resolve"]);
    assert_ne!(refused.code, Some(0), "{refused:?}");
    assert!(refused.err.contains("\">=2\""), "{}", refused.err);
    assert!(refused.err.contains("\">=0.1 <2\""), "{}", refused.err);

    set_engine(&root.at, Some(&caret()));
    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_eq!(resolved.code, Some(0), "{resolved:?}");
}
