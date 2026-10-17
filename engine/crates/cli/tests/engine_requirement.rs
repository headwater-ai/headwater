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
//! to the overlap of the corpus range and the package range. A fourth case
//! holds a byte order mark before the first line to the same verdict as the
//! file without it (#1755).

use headwater_resolve::release::{self, satisfies};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository this test tree sits in.
fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// A resolved corpus that a strict check passes, in the shape `adoption.rs`
/// builds: the vendored package, the library without its doctrine pages, this
/// repository's declaration and overlay, and a stub for each path outside the
/// corpus root that the overlay names.
///
/// `label` names the case, because cargo runs the cases of one target as
/// threads of one process.
struct Root {
    at: PathBuf,
}

impl Root {
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-engine-requirement-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");

        let repository = repository();
        copy(
            &repository.join(headwater_resolve::package::PACKAGES),
            &at.join(headwater_resolve::package::PACKAGES),
        );
        copy(
            &repository.join("docs/taxonomies"),
            &at.join("docs/taxonomies"),
        );
        let library = at.join("docs/taxonomies");
        let _ = std::fs::remove_file(library.join("README.md"));
        for entry in std::fs::read_dir(&library).expect("the copied library reads") {
            let _ =
                std::fs::remove_file(entry.expect("the entry reads").path().join("doctrine.md"));
        }
        for name in ["taxonomy.yml", "overlay.yml"] {
            let to = at.join(".headwater").join(name);
            std::fs::create_dir_all(to.parent().expect("it has a parent"))
                .expect("the declaration directory is there");
            std::fs::copy(repository.join(".headwater").join(name), to)
                .expect("the declaration copies");
        }
        for entry in [
            "README.md",
            ".github/CONTRIBUTING.md",
            ".github/SECURITY.md",
            ".github/ISSUE_TEMPLATE/issue.md",
        ] {
            let to = at.join(entry);
            std::fs::create_dir_all(to.parent().expect("it has a parent"))
                .expect("the directory is there");
            std::fs::write(to, "# A stub\n\nThis file stands in for the real one.\n")
                .expect("the stub writes");
        }

        let root = Root { at };
        set_engine(&root.at, None);
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(resolved.code, Some(0), "the fixture resolves: {resolved:?}");
        root
    }

    fn run(&self, arguments: &[&str]) -> Ran {
        let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(arguments)
            .arg("--root")
            .arg(&self.at)
            .output()
            .expect("the binary runs");
        Ran {
            code: output.status.code(),
            out: String::from_utf8_lossy(&output.stdout).into_owned(),
            err: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

/// The two streams held apart. The requirement is reported on standard error
/// alone, so a case that merged them could not tell a report from the output.
#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the directory is there");
    for entry in std::fs::read_dir(from).expect("the fixture directory reads") {
        let entry = entry.expect("the entry reads");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("the entry has a type").is_dir() {
            copy(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("the fixture copies");
        }
    }
}

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
        assert!(
            kept.contains("\nengine: "),
            "the declaration has a corpus block"
        );
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

/// Standard output without the read set's digest of the declaration.
///
/// The read set records a digest of `.headwater/taxonomy.yml` as an input, and
/// a case that edits the file moves that one line for a reason that is not the
/// requirement. Every other byte of standard output stays.
fn without_declaration_digest(out: &str) -> String {
    out.lines()
        .filter(|line| {
            !line
                .trim_start()
                .starts_with("input .headwater/taxonomy.yml ")
        })
        .map(|line| format!("{line}\n"))
        .collect()
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
        assert_eq!(
            without_declaration_digest(&strict.out),
            without_declaration_digest(&plain.out),
            "{pass}: standard output does not move"
        );
        // The second run is served from the cache, and the requirement is read
        // off the declaration and never out of a cached verdict, so it holds
        // there too.
        if pass == "warm" {
            assert!(strict.err.contains(" 0 evaluated"), "warm: {}", strict.err);
        }
    }

    // `--fix` loads the corpus twice in one process, once to compose the
    // patches and once for the report. The line prints once.
    let fixing = root.run(&["check", "--fix"]);
    assert_eq!(reports(&fixing.err).len(), 1, "one line: {}", fixing.err);

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

/// An unquoted range that opens with `>` is not YAML a reader can load: YAML
/// reads a `>` there as the start of a folded block. The refusal names the
/// key and says to quote the range, rather than quoting the YAML parser
/// alone (#1600).
#[test]
fn an_unquoted_range_that_opens_with_a_greater_than_sign_is_refused_with_the_quotes_it_needs() {
    for (case, written, value) in [
        ("floor", ">=0.6", ">=0.6"),
        ("caret", ">=0.5 <0.6", ">=0.5 <0.6"),
        // A comment after the range is not part of the line to write.
        ("comment", ">=0.5 <0.6   # the release line", ">=0.5 <0.6"),
    ] {
        // Not `unquoted`: the refusal prints this directory, and the case
        // looks for the word `quote` in the refusal.
        let root = Root::new(&format!("engine-requirement-bare-{case}"));
        set_engine(&root.at, None);
        let path = root.at.join(".headwater/taxonomy.yml");
        let text = std::fs::read_to_string(&path).expect("the declaration reads");
        let text = text.replacen("\ncorpus:", &format!("\nengine: {written}\n\ncorpus:"), 1);
        std::fs::write(&path, text).expect("the declaration writes");
        for verb in [&["check", "--strict"][..], &["taxonomy", "resolve"][..]] {
            let ran = root.run(verb);
            assert_ne!(ran.code, Some(0), "{case} {verb:?}: {ran:?}");
            assert!(
                ran.err.contains("`engine`") && ran.err.contains("in quotes"),
                "{case} {verb:?}: {}",
                ran.err
            );
            assert!(
                ran.err.contains(&format!("engine: \"{value}\"`")),
                "{case} {verb:?}: the refusal shows the line as it must be written: {}",
                ran.err
            );
            assert!(
                !ran.err.contains("while scanning a block scalar"),
                "{case} {verb:?}: the parser's own text is not the refusal: {}",
                ran.err
            );
        }
    }
}

/// The declaration with `line` as its first line, and with a byte order mark
/// before it when `mark` is set. Every other line stays, and any `engine:`
/// line elsewhere is removed.
fn write_first_line(at: &Path, line: &str, mark: bool) {
    set_engine(at, None);
    let path = at.join(".headwater/taxonomy.yml");
    let text = std::fs::read_to_string(&path).expect("the declaration reads");
    let bom = if mark { "\u{feff}" } else { "" };
    std::fs::write(&path, format!("{bom}{line}\n{text}")).expect("the declaration writes");
}

/// A byte order mark before the first line does not hide the key on that line
/// (#1755). An editor on Windows can save one, and the YAML parser keeps it as
/// part of the first key, so a range behind it was read as no range and a
/// strict check passed on any engine. The check with the mark fails exactly as
/// it does without it. A mark before `taxonomy:` does not hide that block, and
/// an unquoted `>` range behind a mark gets the refusal that says to quote it.
#[test]
fn a_byte_order_mark_before_the_engine_line_does_not_hide_the_range() {
    let root = Root::new("engine-requirement-bom");

    let mut runs = Vec::new();
    for mark in [false, true] {
        write_first_line(&root.at, "engine: \">=99\"", mark);
        let strict = root.run(&["check", "--strict"]);
        assert_eq!(strict.code, Some(1), "mark {mark}: {strict:?}");
        let lines = reports(&strict.err);
        assert_eq!(lines.len(), 1, "mark {mark}: one line: {}", strict.err);
        assert!(lines[0].contains("\">=99\""), "mark {mark}: {}", lines[0]);
        assert!(
            lines[0].contains(release::ENGINE),
            "mark {mark}: {}",
            lines[0]
        );
        runs.push((strict.code, lines[0].to_string()));
    }
    assert_eq!(
        runs[0], runs[1],
        "the check with the mark fails as it does without it"
    );

    // The mark before `taxonomy:`, the order `init` writes. The comment
    // header goes, so that the mark sits on the key.
    set_engine(&root.at, Some(&caret()));
    let path = root.at.join(".headwater/taxonomy.yml");
    let text = std::fs::read_to_string(&path).expect("the declaration reads");
    let at = text
        .find("\ntaxonomy:")
        .expect("the fixture has a taxonomy block");
    assert!(
        text[..at]
            .lines()
            .all(|line| line.is_empty() || line.starts_with('#')),
        "only comments come before the taxonomy block"
    );
    let text = &text[at + 1..];
    std::fs::write(&path, format!("\u{feff}{text}")).expect("the declaration writes");
    let admitted = root.run(&["check", "--strict"]);
    assert_eq!(admitted.code, Some(0), "{admitted:?}");
    assert!(reports(&admitted.err).is_empty(), "{}", admitted.err);

    // An unquoted `>` range behind the mark.
    write_first_line(&root.at, "engine: >=99", true);
    for verb in [&["check", "--strict"][..], &["taxonomy", "resolve"][..]] {
        let ran = root.run(verb);
        assert_ne!(ran.code, Some(0), "{verb:?}: {ran:?}");
        assert!(
            ran.err.contains("`engine`") && ran.err.contains("in quotes"),
            "{verb:?}: {}",
            ran.err
        );
        assert!(
            ran.err.contains("engine: \">=99\"`"),
            "{verb:?}: {}",
            ran.err
        );
    }
}

/// A file that fails to load for another reason keeps the loader's own
/// refusal, so the hint above is never a guess about an unrelated fault. A
/// quoted range holds a `>` and is not the fault, and a `>` that is a folded
/// block header alone opens a block that loads.
#[test]
fn a_declaration_that_fails_to_load_for_another_reason_is_not_told_to_quote_its_range() {
    for (case, line) in [
        ("below", "engine: \"<0.6\""),
        // The case label names the scratch directory, which the refusal
        // prints, so no label holds the word the assertion looks for.
        ("caret", "engine: \">=0.5 <0.6\""),
        ("header", "engine: >"),
        ("header-chomp", "engine: >-"),
    ] {
        let root = Root::new(&format!("engine-requirement-other-load-failure-{case}"));
        set_engine(&root.at, None);
        let path = root.at.join(".headwater/taxonomy.yml");
        let text = std::fs::read_to_string(&path).expect("the declaration reads");
        let text = text.replacen(
            "\ncorpus:",
            &format!("\n{line}\nbroken: [unclosed\n\ncorpus:"),
            1,
        );
        std::fs::write(&path, text).expect("the declaration writes");
        let ran = root.run(&["check", "--strict"]);
        assert_ne!(ran.code, Some(0), "{case}: {ran:?}");
        assert!(!ran.err.contains("quote"), "{case}: {}", ran.err);
    }
}

/// A list or a mapping under `engine` is refused, never read as no range.
#[test]
fn an_engine_value_that_is_not_one_range_is_refused() {
    for (case, value) in [("list", "[\">=0.6\"]"), ("mapping", "{range: \">=0.6\"}")] {
        let root = Root::new(&format!("engine-requirement-{case}"));
        set_engine(&root.at, None);
        let path = root.at.join(".headwater/taxonomy.yml");
        let text = std::fs::read_to_string(&path).expect("the declaration reads");
        let text = text.replacen("\ncorpus:", &format!("\nengine: {value}\n\ncorpus:"), 1);
        std::fs::write(&path, text).expect("the declaration writes");
        for verb in [&["check", "--strict"][..], &["taxonomy", "resolve"][..]] {
            let ran = root.run(verb);
            assert_ne!(ran.code, Some(0), "{case} {verb:?}: {ran:?}");
            assert!(
                ran.err.contains("`engine` is one range on one line"),
                "{case} {verb:?}: {}",
                ran.err
            );
        }
    }
}

/// This repository asks for the engine it builds. A bump of the workspace
/// version past the range fails here, and the release guide says to move the
/// line in the same commit.
#[test]
fn this_repository_asks_for_the_engine_it_builds() {
    let declaration =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../.headwater/taxonomy.yml");
    let text = std::fs::read_to_string(&declaration).expect("the declaration reads");
    let ranges: Vec<&str> = text
        .lines()
        .filter_map(|line| line.strip_prefix("engine:"))
        .map(|value| value.trim().trim_matches('"'))
        .collect();
    assert_eq!(
        ranges.len(),
        1,
        ".headwater/taxonomy.yml declares one `engine:` line"
    );
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

    // A range that overlaps the package range but excludes this engine
    // resolves, and the resolve reports the engine once. The floor is the
    // minor above this engine, so the range moves with each release; a
    // literal floor stopped excluding the engine at the 0.6.0 bump.
    let (major, minor, _) = release::parts(release::ENGINE).expect("this engine has a version");
    assert!(major < 2, "the package range of this fixture ends below 2");
    set_engine(&root.at, Some(&format!(">={major}.{} <2", minor + 1)));
    let reported = root.run(&["taxonomy", "resolve"]);
    assert_eq!(reported.code, Some(0), "{reported:?}");
    assert_eq!(reports(&reported.err).len(), 1, "{}", reported.err);

    set_engine(&root.at, Some(&caret()));
    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_eq!(resolved.code, Some(0), "{resolved:?}");
    assert!(reports(&resolved.err).is_empty(), "{}", resolved.err);
}
