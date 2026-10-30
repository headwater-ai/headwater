// SPDX-License-Identifier: Apache-2.0
//! The hidden oracle of the probe
//! HW-PROBE-a-session-lets-the-graph-export-list-the-patterns-of-every-anchor
//! (#1874, slice 4b), row 2 of HW-DR-0112.
//!
//! This file names that probe in its first lines, so `tools/probe/seal.sh`
//! removes it from every workspace, and `tools/probe/grading-tree.sh` puts it
//! back in the grading copy. The `r2-export-patterns` oracle under `oracles:`
//! in `.headwater/probe.yml` holds it and runs it with `-- --ignored`.
//!
//! **Why the case is ignored.** On the pin an anchor of one pattern writes no
//! `patterns`, so the case fails on this repository's own tree by design, and
//! CI never runs it. It builds and lints on every change all the same.
//!
//! The case runs the built binary over a corpus with one record that governs
//! one path and a list of two, and reads `headwater export --format json`:
//!
//! 1. Each entry of `graph.anchors`, and each edge target bound to an anchor,
//!    carries `patterns`. Where it holds one pattern, that pattern is `id`.
//! 2. `version` and `export_version` are `1.4`.
//!    `docs/interfaces/headwater-export.md`, under "Raising the version",
//!    states that a member the export writes in more cases than before moves
//!    the minor, and the pin writes `1.3`.

use headwater_yaml::json::{count, field};
use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// A corpus of the repository's own lock and declaration, and the files a
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
        std::fs::write(path, text).expect("the file writes");
    }

    /// `headwater export --format json` over the corpus, as its standard output.
    fn export(&self) -> String {
        let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(["export", "--format", "json", "--root"])
            .arg(&self.at)
            .output()
            .expect("the binary runs");
        String::from_utf8(output.stdout).expect("the export is UTF-8")
    }
}

fn at(document: &str, steps: &[&str]) -> Option<String> {
    field(
        document,
        &steps.iter().map(|step| step.to_string()).collect::<Vec<_>>(),
    )
}

fn len(document: &str, steps: &[&str]) -> Option<usize> {
    count(
        document,
        &steps.iter().map(|step| step.to_string()).collect::<Vec<_>>(),
    )
}

/// Whether the anchor at `steps` lists its patterns, and the one pattern of a
/// single anchor is its `id`.
fn lists_its_patterns(export: &str, steps: &[&str]) -> Result<(), String> {
    let mut at_patterns = steps.to_vec();
    at_patterns.push("patterns");
    let patterns = len(export, &at_patterns)
        .ok_or_else(|| format!("{} carries no `patterns`", steps.join(".")))?;
    if patterns == 0 {
        return Err(format!("{} lists no pattern", steps.join(".")));
    }
    if patterns == 1 {
        let mut at_id = steps.to_vec();
        at_id.push("id");
        let mut at_first = at_patterns.clone();
        at_first.push("0");
        let id = at(export, &at_id);
        let first = at(export, &at_first);
        if id != first {
            return Err(format!(
                "{} holds one pattern {first:?}, and its `id` is {id:?}",
                steps.join(".")
            ));
        }
    }
    Ok(())
}

#[test]
#[ignore = "the hidden oracle of one probe: it fails on the pin by design"]
fn every_anchor_lists_its_patterns_and_the_export_raises_its_version() {
    let corpus = Corpus::new("r2-export-patterns");
    corpus.write("engine/crates/x/src/a.rs", "// a\n");
    corpus.write("engine/crates/x/src/b.rs", "// b\n");
    corpus.write(
        "docs/decisions/0001-a-record.md",
        "---\nid: HW-DR-0001\nstatus: current\nstatus_since: 2026-10-30\nsummary: \"A record \
         that the oracle reads.\"\nlast_verified: 2026-10-30\ntitle: \"1 — A \
         record\"\nrelations:\n  governs:\n    - engine/crates/x/src/a.rs\n    - \
         [engine/crates/x/src/a.rs, engine/crates/x/src/b.rs]\n---\n\n# 1 — A record\n\n## \
         Context\n\nThe team reads the page.\n\n## Decision\n\nThe team keeps the page.\n\n## \
         Consequences\n\nNothing moves.\n",
    );
    let export = corpus.export();
    let anchors = len(&export, &["graph", "anchors"])
        .unwrap_or_else(|| panic!("the export holds no `graph.anchors`:\n{export}"));
    assert!(anchors >= 2, "the corpus declares two anchors, and the export holds {anchors}");
    let mut single = 0;
    for index in 0..anchors {
        let index = index.to_string();
        if let Err(why) = lists_its_patterns(&export, &["graph", "anchors", &index]) {
            panic!("{why}");
        }
        if len(&export, &["graph", "anchors", &index, "patterns"]) == Some(1) {
            single += 1;
        }
    }
    assert!(single >= 1, "the corpus declares an anchor of one pattern, and none holds one");
    let edges = len(&export, &["graph", "edges"]).unwrap_or(0);
    for index in 0..edges {
        let index = index.to_string();
        if at(&export, &["graph", "edges", &index, "target", "bound"]).as_deref() == Some("anchor") {
            if let Err(why) = lists_its_patterns(&export, &["graph", "edges", &index, "target"]) {
                panic!("{why}");
            }
        }
    }
    for member in ["version", "export_version"] {
        assert_eq!(
            at(&export, &[member]).as_deref(),
            Some("1.4"),
            "the export writes `patterns` in more cases than before, so `{member}` moves its minor"
        );
    }
}
