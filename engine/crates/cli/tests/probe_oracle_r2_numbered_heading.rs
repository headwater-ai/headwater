// SPDX-License-Identifier: Apache-2.0
//! The hidden oracle of the probe
//! HW-PROBE-a-session-lets-the-required-sections-rule-read-a-numbered-heading
//! (#1874, slice 4b), row 2 of HW-DR-0112.
//!
//! This file names that probe in its first lines, so `tools/probe/seal.sh`
//! removes it from every workspace, and `tools/probe/grading-tree.sh` puts it
//! back in the grading copy. The `r2-numbered-heading` oracle under
//! `oracles:` in `.headwater/probe.yml` holds it and runs it with
//! `-- --ignored`.
//!
//! **Why the case is ignored.** On the pin a numbered heading satisfies no
//! required section, so the case fails on this repository's own tree by
//! design, and CI never runs it. It builds and lints on every change all the
//! same.
//!
//! The case runs the built binary over a corpus of two decision records and
//! reads `headwater check --json`:
//!
//! 1. A record whose headings are `1. Context`, `2 Decision` and
//!    `3. Consequences` carries no finding of `section.required.missing`.
//! 2. A record whose second heading is `2. Decisions` still carries one: a
//!    leading number is not part of a section name, and every other word is.
//! 3. The entry of `rules` for `section.required.missing` reports version 2.
//!    `docs/subsystems/checks-and-cache.md` states that an author raises the
//!    edition of a rule by hand when it decides differently, and the pin
//!    reports 1. A warm cache would otherwise serve the old verdict.

use headwater_yaml::json::{count, field};
use std::path::{Path, PathBuf};
use std::process::Command;

const RULE: &str = "section.required.missing";

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// A corpus of the repository's own lock and declaration, and the records a
/// case writes, removed when it is dropped.
struct Corpus {
    at: PathBuf,
}

impl Drop for Corpus {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

impl Corpus {
    fn new(label: &str) -> Corpus {
        let at = std::env::temp_dir().join(format!(
            "headwater-probe-oracle-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(at.join(".headwater")).expect("the corpus directory is made");
        for name in ["taxonomy.lock", "taxonomy.yml"] {
            std::fs::copy(
                repository().join(".headwater").join(name),
                at.join(".headwater").join(name),
            )
            .expect("the declaration copies");
        }
        Corpus { at }
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.at.join(relative);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("the shelf is made");
        std::fs::write(path, text).expect("the record writes");
    }

    /// `headwater check --json` over the corpus, as its standard output.
    fn check(&self) -> String {
        let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(["check", "--json", "--root"])
            .arg(&self.at)
            .output()
            .expect("the binary runs");
        String::from_utf8(output.stdout).expect("the report is UTF-8")
    }
}

fn at(document: &str, steps: &[&str]) -> Option<String> {
    field(
        document,
        &steps.iter().map(|step| step.to_string()).collect::<Vec<_>>(),
    )
}

fn len(document: &str, steps: &[&str]) -> usize {
    count(
        document,
        &steps.iter().map(|step| step.to_string()).collect::<Vec<_>>(),
    )
    .unwrap_or(0)
}

fn record(number: &str, headings: [&str; 3]) -> String {
    format!(
        "---\nid: HW-DR-{number}\nstatus: current\nstatus_since: 2026-10-30\nsummary: \"A record \
         that the oracle reads.\"\nlast_verified: 2026-10-30\ntitle: \"{number} — A \
         record\"\n---\n\n# {number} — A record\n\n## {}\n\nThe team reads the page.\n\n## \
         {}\n\nThe team keeps the page.\n\n## {}\n\nNothing moves.\n",
        headings[0], headings[1], headings[2]
    )
}

#[test]
#[ignore = "the hidden oracle of one probe: it fails on the pin by design"]
fn a_numbered_heading_names_its_section_and_the_rule_raises_its_edition() {
    let corpus = Corpus::new("r2-numbered-heading");
    corpus.write(
        "docs/decisions/0001-numbered.md",
        &record("0001", ["1. Context", "2 Decision", "3. Consequences"]),
    );
    corpus.write(
        "docs/decisions/0002-misnamed.md",
        &record("0002", ["Context", "2. Decisions", "Consequences"]),
    );
    let report = corpus.check();
    let on = |path: &str| {
        (0..len(&report, &["findings"]))
            .filter(|index| {
                let index = index.to_string();
                at(&report, &["findings", &index, "rule"]).as_deref() == Some(RULE)
                    && at(&report, &["findings", &index, "path"]).as_deref() == Some(path)
            })
            .count()
    };
    assert_eq!(
        on("docs/decisions/0001-numbered.md"),
        0,
        "a heading `2 Decision` or `1. Context` names its section, and the record still \
         carries {RULE}"
    );
    assert_eq!(
        on("docs/decisions/0002-misnamed.md"),
        1,
        "`2. Decisions` names no `Decision` section, and the rule must still report it once"
    );
    let entry = (0..len(&report, &["rules"]))
        .map(|index| index.to_string())
        .find(|index| at(&report, &["rules", index, "rule"]).as_deref() == Some(RULE))
        .unwrap_or_else(|| panic!("the report serves no `{RULE}`:\n{report}"));
    assert_eq!(
        at(&report, &["rules", &entry, "version"]).as_deref(),
        Some("2"),
        "the rule decides differently, so its edition moves from 1 to 2"
    );
}
