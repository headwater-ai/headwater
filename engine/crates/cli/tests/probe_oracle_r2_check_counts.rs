// SPDX-License-Identifier: Apache-2.0
//! The hidden oracle of the probe
//! HW-PROBE-a-session-adds-a-count-of-findings-by-severity-to-the-json-report-of-the-check
//! (#1874, slice 4b), row 2 of HW-DR-0112.
//!
//! This file names that probe in its first lines, so `tools/probe/seal.sh`
//! removes it from every workspace, and `tools/probe/grading-tree.sh` puts it
//! back in the grading copy. The `r2-check-counts` oracle under `oracles:` in
//! `.headwater/probe.yml` holds it and runs it with `-- --ignored`.
//!
//! **Why the case is ignored.** The pin writes no `counts` member, so the case
//! fails on this repository's own tree by design, and CI never runs it. It
//! builds and lints on every change all the same.
//!
//! The case runs the built binary over a corpus that reports at least one
//! `error` and one `warn`, and reads `headwater check --json`:
//!
//! 1. `counts.error`, `counts.warn` and `counts.info` each equal the number of
//!    entries of `findings` with that `severity`.
//! 2. `version` is `1.7`. `docs/interfaces/headwater-check.md`, under
//!    "Raising the version", states that each added member moved the minor,
//!    and the pin writes `1.6`.

use headwater_yaml::json::{count, field};
use std::path::{Path, PathBuf};
use std::process::Command;

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

#[test]
#[ignore = "the hidden oracle of one probe: it fails on the pin by design"]
fn the_report_counts_its_findings_by_severity_and_raises_its_version() {
    let corpus = Corpus::new("r2-check-counts");
    corpus.write(
        "docs/decisions/0001-a-record.md",
        "---\nid: HW-DR-0001\nstatus: current\nstatus_since: 2026-10-30\nsummary: \"A record \
         that the oracle reads.\"\nlast_verified: 2026-10-30\ntitle: \"1 — A record\"\n---\n\n# 1 \
         — A record\n\n## Context\n\nThe team keeps one list; it is short.\n\n## Decision\n\nThe \
         organisation keeps the page.\n\n## Consequences\n\nNothing moves.\n",
    );
    let report = corpus.check();
    let total = count(&report, &["findings".to_string()]).expect("the report holds `findings`");
    let mut tally = [("error", 0usize), ("warn", 0), ("info", 0)];
    for index in 0..total {
        let severity = at(&report, &["findings", &index.to_string(), "severity"])
            .expect("each finding carries a severity");
        let slot = tally
            .iter_mut()
            .find(|(name, _)| *name == severity)
            .unwrap_or_else(|| panic!("`{severity}` is not a severity the report writes"));
        slot.1 += 1;
    }
    assert!(
        tally[0].1 > 0 && tally[1].1 > 0,
        "the corpus was written to report an error and a warning, and it reports {tally:?}"
    );
    for (severity, expected) in tally {
        let counted = at(&report, &["counts", severity]);
        assert_eq!(
            counted.as_deref(),
            Some(expected.to_string().as_str()),
            "`counts.{severity}` is {counted:?}, and the report lists {expected} finding(s) of \
             that severity"
        );
    }
    assert_eq!(
        at(&report, &["version"]).as_deref(),
        Some("1.7"),
        "a member added to the JSON shape of the check moves its minor version"
    );
}
