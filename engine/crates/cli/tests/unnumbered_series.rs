// SPDX-License-Identifier: Apache-2.0
//! What the specification series of headwater/standard 5.0.0 names a part,
//! and what it requires of one.
//!
//! # The defect this target exists for
//!
//! Through 4.x the design-spec bundle laid out the `spec_series` shelf as
//! `{sequence:02d}-{slug}.md` and required the `sequence` facet on every kind
//! of the shelf. No decision record chose the number, no rule read it, and two
//! documents could take one value
//! ([HW-OBL-0175](../../../../docs/obligations/0175-two-documents-take-one-sequence-on-a-sequence-declaring-shelf-and-no-rule-reads-it.md)).
//! The owner ruled on #2137 that the series drops the number, and
//! [HW-DR-0117](../../../../docs/decisions/0117-the-specification-series-names-a-part-by-its-slug-and-carries-no-sequence.md)
//! records the ruling. 5.0.0 deletes the facet and lays the shelf out as
//! `{slug}.md`.
//!
//! # Why these cases drive the binary
//!
//! The claim is about the file a verb writes and the findings a run prints, so
//! each case assembles a scratch corpus that takes the vendored package with
//! the design-spec selection and runs the built binary over it. The corpus
//! declares no overlay, so what the cases read is the package as an adopter
//! receives it.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// The version of headwater/standard this repository vendors, read from its
/// own consumer declaration, so the scratch corpus pins what is on disk.
fn vendored_version() -> String {
    let text = std::fs::read_to_string(repository().join(".headwater/taxonomy.yml"))
        .expect("the consumer declaration reads");
    text.lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("version: "))
        .expect("the declaration pins a version")
        .to_string()
}

struct Root {
    at: PathBuf,
}

impl Root {
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-unnumbered-series-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");
        copy(
            &repository().join(headwater_resolve::package::PACKAGES),
            &at.join(headwater_resolve::package::PACKAGES),
        );
        std::fs::write(
            at.join(".headwater/taxonomy.yml"),
            format!(
                "taxonomy:\n  package: headwater/standard\n  version: {}\n  bundles: [design-spec, evidence-and-obligation]\n",
                vendored_version()
            ),
        )
        .expect("the declaration writes");
        let root = Root { at };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(resolved.code, Some(0), "the scratch corpus resolves\n{resolved:?}");
        root
    }

    fn write(&self, path: &str, text: &str) {
        let to = self.at.join(path);
        std::fs::create_dir_all(to.parent().expect("it has a parent"))
            .expect("the shelf directory is there");
        std::fs::write(to, text).expect("the document writes");
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

    /// Every line of the text report that names the rule.
    fn findings(&self, rule: &str) -> Vec<String> {
        let ran = self.run(&["check", "--no-cache"]);
        ran.out
            .lines()
            .filter(|line| line.contains(rule))
            .map(str::to_string)
            .collect()
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

/// The two streams held apart.
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

/// A hand-written part of the series with the front matter given. Its
/// sections are written, so the only findings the cases count are the ones
/// about its facets.
fn part(slug: &str, title: &str, extra: &str) -> String {
    format!(
        "---\nid: HW-SPEC-{slug}\ndoc_type: design_spec\ntitle: {title}\n{extra}summary: One part of the series.\nstatus: current\nstatus_since: 2026-11-28\nlast_verified: 2026-11-28\n---\n\n# {title}\n\nThis part states one thing about the system.\n"
    )
}

/// The decisive case. The scaffolder names a new part after its slug, writes
/// no `sequence`, and opens the body with the title alone.
#[test]
fn a_new_part_is_named_by_its_slug_and_carries_no_sequence() {
    let root = Root::new("new");
    let made = root.run(&["new", "design_spec", "--title", "Harness support"]);
    assert_eq!(made.code, Some(0), "the part scaffolds\n{made:?}");
    let written = root.at.join("docs/spec/harness-support.md");
    let text = std::fs::read_to_string(&written).unwrap_or_else(|error| {
        panic!(
            "the part is at {}: {error}\n{made:?}",
            written.display()
        )
    });
    assert!(
        !text.lines().any(|line| line.starts_with("sequence:")),
        "the part declares no sequence:\n{text}"
    );
    assert!(
        text.lines().any(|line| line == "# Harness support"),
        "the first heading is the title alone:\n{text}"
    );
}

/// A part with no `sequence` is complete: nothing reports the facet as
/// missing, on the scaffolded part or on a hand-written one.
#[test]
fn a_part_with_no_sequence_raises_no_missing_facet() {
    let root = Root::new("complete");
    let made = root.run(&["new", "design_spec", "--title", "Harness support"]);
    assert_eq!(made.code, Some(0), "the part scaffolds\n{made:?}");
    root.write(
        "docs/spec/conceptual-model.md",
        &part("conceptual-model", "Conceptual model", ""),
    );
    let missing = root.findings("facet.required.missing");
    assert!(
        missing.is_empty(),
        "no part is missing a required facet:\n{}",
        missing.join("\n")
    );
}

/// What an adopter who upgrades from 4.x keeps on disk: a numbered file name
/// and a `sequence` line. Both are inert. No rule reads a layout after birth
/// and no rule reads a front-matter key the taxonomy does not declare, so the
/// upgrade raises nothing on the part.
#[test]
fn a_part_left_numbered_by_4_x_raises_nothing_about_its_sequence() {
    let root = Root::new("left-over");
    root.write(
        "docs/spec/12-check-layer.md",
        &part("check-layer", "The check layer", "sequence: 12\n"),
    );
    let ran = root.run(&["check", "--no-cache"]);
    let about: Vec<&str> = ran
        .out
        .lines()
        .filter(|line| line.contains("sequence") || line.contains("facet."))
        .collect();
    assert!(
        about.is_empty(),
        "nothing names the left-over number or a facet:\n{}",
        ran.out
    );
}
