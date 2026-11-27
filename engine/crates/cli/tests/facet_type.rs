// SPDX-License-Identifier: Apache-2.0
//! What `check --strict` makes of a document facet value that is not of the
//! type its declaration states.
//!
//! # The defect this target exists for
//!
//! HW-OBL-0171 (#1492) measured that nothing read a document's facet value
//! against the `type` of its declaration. `status_since: {from: x}` and
//! `last_verified: 2026-02-30` both passed a strict run at exit 0, and every
//! rule that reads a date then read a date nobody wrote. The rule
//! `facet.value.not_of_type` closes that.
//!
//! # Why these cases drive the binary
//!
//! The claim is about an exit status and a report, and both are the caller's,
//! so each case scaffolds one decision record into a scratch repository, writes
//! one value onto it and runs the built binary, as `blank_facet.rs` does. The
//! integer, boolean and duration cases are unit tests of the rule, because this
//! repository's taxonomy declares no document facet of those types that the
//! next major version of its package keeps.
//!
//! # Why `2026-02-30` is the first case
//!
//! The meta-schema's date test read digit positions only, so it admitted
//! `2026-02-30`. An implementation that reused that test passes every other
//! refusal below and fails this one, so it is the case that separates a
//! calendar check from a digit pattern.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The obligation the control of `headwater/standard` binds the rule to
/// (HW-OBL-0171, #1492). A finding names it beside the rule, so a control that
/// stops naming the rule changes the text every refusal below reads.
const OBLIGATION: &str = "OB-FACET-4";

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// A scratch repository holding one scaffolded decision record.
struct Root {
    at: PathBuf,
    document: PathBuf,
}

impl Root {
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-facet-type-{}-{label}",
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
        for name in ["taxonomy.yml", "overlay.yml"] {
            let to = at.join(".headwater").join(name);
            std::fs::create_dir_all(to.parent().expect("it has a parent"))
                .expect("the declaration directory is there");
            std::fs::copy(repository.join(".headwater").join(name), to)
                .expect("the declaration copies");
        }

        // The overlay lists four paths outside the corpus root under
        // `regimes.language.ste_house.outside_root` (HW-DR-0084), and a path
        // that matches no file is an error of a strict run, so each gets a
        // stub that obeys the regime.
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
        // The overlay declares a governed scope, and a pattern that matches no
        // entry of the tree is an error of a strict run (#2099), so one entry
        // under each pattern stands in for the code the copy does not carry.
        for entry in [
            "engine/crates/stub/src/lib.rs",
            "engine/crates/cli/harness/stub",
            ".githooks/stub",
            ".claude/hooks/stub",
            ".claude/agents/stub",
            ".claude/skills/stub",
            "tools/stub",
            "site/stub",
        ] {
            let to = at.join(entry);
            std::fs::create_dir_all(to.parent().expect("it has a parent"))
                .expect("the scope directory is there");
            std::fs::write(to, "").expect("the scope entry writes");
        }

        let mut root = Root {
            at,
            document: PathBuf::new(),
        };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolved.code,
            Some(0),
            "the fixture resolves\n{}{}",
            resolved.out,
            resolved.err
        );

        // Scaffolded rather than written out here, so a taxonomy that adds a
        // required facet fails this fixture rather than typing the document as
        // nothing and raising the finding for another reason.
        let made = root.run(&["new", "decision", "--title", "A scratch ruling"]);
        assert_eq!(
            made.code,
            Some(0),
            "the corpus document scaffolds\n{}{}",
            made.out,
            made.err
        );
        root.document = std::fs::read_dir(root.at.join("docs/decisions"))
            .expect("the decisions shelf is there")
            .map(|entry| entry.expect("the entry reads").path())
            .find(|path| path.extension().is_some_and(|kind| kind == "md"))
            .expect("the scaffolder wrote a document");
        root
    }

    /// The path the check layer names, relative to the corpus root.
    fn named(&self) -> String {
        let name = self.document.file_name().expect("the document has a name");
        format!("docs/decisions/{}", name.to_string_lossy())
    }

    /// Replace the whole line that opens with `key:` with one written out here.
    fn facet(&self, key: &str, line: &str) {
        let text = std::fs::read_to_string(&self.document).expect("the document reads");
        let prefix = format!("{key}:");
        let found = text
            .lines()
            .find(|line| line.starts_with(&prefix))
            .unwrap_or_else(|| panic!("the scaffolder wrote `{key}`"))
            .to_string();
        std::fs::write(&self.document, text.replacen(&found, line, 1))
            .expect("the document writes");
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

/// The two streams held apart. `headwater check` reports on one and states a
/// run statistic on the other, and a case that merged them reads the wrong one.
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
        match entry.file_type().expect("the file type reads").is_dir() {
            true => copy(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("the fixture copies");
            }
        }
    }
}

/// Write one value and assert the strict verdict, the rule and the value.
fn refuses(label: &str, key: &str, line: &str, complaint: &str) {
    let root = Root::new(label);
    root.facet(key, line);
    let ran = root.run(&["check", "--strict", "--no-cache"]);
    assert_eq!(
        ran.code,
        Some(1),
        "`{line}` passed a strict run\n{}{}",
        ran.out,
        ran.err
    );
    let named = root.named();
    // The text report wraps a long message onto a second line, so the words
    // are compared with the wrap taken out.
    let words = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
    let reported = ran.out.split(&named).skip(1).any(|after| {
        let after = words(after);
        after.contains(&format!("facet.value.not_of_type ({OBLIGATION})"))
            && after.contains(&words(complaint))
    });
    assert!(
        reported,
        "no `facet.value.not_of_type ({OBLIGATION})` finding on {named} saying {complaint}\n{}",
        ran.out
    );
}

/// A day that no month has. The digit pattern admits it, and the calendar
/// does not.
#[test]
fn a_date_that_no_calendar_has_stops_a_strict_run() {
    refuses(
        "february-30",
        "status_since",
        "status_since: 2026-02-30",
        "`status_since` is `2026-02-30`, and its declaration says `date`",
    );
    refuses(
        "zeros",
        "status_since",
        "status_since: 0000-00-00",
        "`status_since` is `0000-00-00`, and its declaration says `date`",
    );
}

/// The two values of the hostile review that overflow a field of four digits.
#[test]
fn a_date_past_four_digits_of_year_stops_a_strict_run() {
    refuses(
        "five-digits",
        "status_since",
        "status_since: 99999-99-99",
        "`status_since` is `99999-99-99`, and its declaration says `date`",
    );
    refuses(
        "overflow",
        "status_since",
        "status_since: 4294967296-01-01",
        "`status_since` is `4294967296-01-01`, and its declaration says `date`",
    );
}

/// A word in a date field.
#[test]
fn a_word_in_a_date_field_stops_a_strict_run() {
    refuses(
        "word",
        "status_since",
        "status_since: yesterday",
        "`status_since` is `yesterday`, and its declaration says `date`",
    );
}

/// The declaration says `date`, and none of these is a scalar.
#[test]
fn a_date_field_that_holds_no_scalar_stops_a_strict_run() {
    refuses(
        "mapping",
        "status_since",
        "status_since: {at: 2026-01-01}",
        "`status_since` is declared as a mapping, and its declaration says `date`",
    );
    refuses(
        "sequence",
        "last_verified",
        "last_verified: [2026-01-01]",
        "`last_verified` is declared as a sequence, and its declaration says `date`",
    );
    refuses(
        "null",
        "last_verified",
        "last_verified:",
        "`last_verified` is declared with no value, and its declaration says `date`",
    );
}

/// The negative arm. A leap day in a leap year is a date, and a rule that
/// refused it would be the first one somebody turns off.
#[test]
fn a_leap_day_in_a_leap_year_is_not_reported() {
    let root = Root::new("leap-day");
    root.facet("status_since", "status_since: 2024-02-29");
    let ran = root.run(&["check", "--strict", "--no-cache"]);
    assert_eq!(
        ran.code,
        Some(0),
        "a leap day in a leap year does not pass a strict run\n{}{}",
        ran.out,
        ran.err
    );
    assert!(
        // A finding names the rule before a colon, or before its obligation.
        // The census line `instances of facet.value.not_of_type` does neither.
        !ran.out.contains("facet.value.not_of_type:")
            && !ran.out.contains("facet.value.not_of_type ("),
        "the rule reported a real date\n{}",
        ran.out
    );
}
