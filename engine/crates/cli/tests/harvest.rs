// SPDX-License-Identifier: Apache-2.0
//! A solution corpus resolves an anchor against the export it pinned, and
//! against no other one.
//!
//! [Spec 7](../../../../docs/spec/07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it)
//! says a tier above a corpus reaches it through one anchor kind, that exactly
//! one resolver owns that kind, and that the resolver reads a pinned corpus
//! export. It also says what happens when the export cannot be read: "a
//! pinned export that the tier cannot read is a **finding that names the
//! pin**. It is never a narrower answer, delivered quietly."
//!
//! [#1233](https://github.com/headwater-ai/headwater/issues/1233) is that
//! sentence as a test.
//!
//! # Why both exports hold `SVC-1`
//!
//! Two source repositories share the kind `service` and define it
//! differently, so each one's export holds a document `SVC-1` with its own
//! facets. That is the trap. If a failed lookup in B's export ever fell back to
//! A's, the edge into B would bind to A's `SVC-1` and nothing would report it.
//! So the case that matters deletes B's export and asserts that the edge into
//! B is reported and names B's pin, while A still holds an `SVC-1`.
//!
//! # Why this drives the binary
//!
//! The defect is that no resolver reaches an export, and a resolver set that a
//! test assembles by hand measures the test. So each case runs the built
//! binary, which builds its resolver set in the one place every verb does.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// The declarations the scratch overlay adds: a kind on one shelf, one anchor
/// kind per source repository, each naming its own resolver, and one relation
/// into each anchor kind.
const DECLARED: &str = "
  identifier_schemes.solution_note_id:
    pattern: \"{namespace}-SOL-{slug}\"
    namespace: HW
    allocation: minted-once

  kinds.solution_note:
    is_a: governed_document
    purpose: rationale
    voice: declarative
    lifecycle: standard
    language: ste_house
    identifier: {scheme: solution_note_id}

  shelves.solution_notes:
    title: The solution notes
    path: docs/solution/*.md
    homogeneous: true
    kind: solution_note

  anchors.service_in_a: {resolver: export-repo-a}
  anchors.service_in_b: {resolver: export-repo-b}

  relations.uses_service_in_a:
    family: association
    from: [solution_note]
    to:   [service_in_a]
    cardinality: many
    created_by: author

  relations.uses_service_in_b:
    family: association
    from: [solution_note]
    to:   [service_in_b]
    cardinality: many
    created_by: author
";

/// The one document of the solution corpus. It names `SVC-1` in each source
/// repository.
const NOTE: &str = "\
---
id: HW-SOL-checkout
title: The checkout path
summary: The checkout path calls one service in each source repository.
status: current
status_since: 2026-09-01
last_verified: 2026-09-01
relations:
  uses_service_in_a:
    - SVC-1
  uses_service_in_b:
    - SVC-1
---

# The checkout path

The checkout path calls one service in each repository.
";

/// A native export, in the shape `headwater export` writes, that holds one
/// document `SVC-1` of kind `service` with the facet value given.
fn export(tier: &str) -> String {
    format!(
        "{{\"version\":\"0.4.0\",\"export_version\":\"0.4.0\",\
         \"profile\":{{\"name\":\"full\",\"target\":\"native\",\"filtered\":false}},\
         \"graph\":{{\"documents\":[{{\"path\":\"docs/services/svc-1.md\",\"kind\":\"service\",\
         \"id\":\"SVC-1\",\"facets\":{{\"tier\":\"{tier}\"}}}}],\"anchors\":[],\"edges\":[]}}}}\n"
    )
}

const EXPORT_A: &str = "harvest/repo-a.json";
const EXPORT_B: &str = "harvest/repo-b.json";

/// A scratch solution corpus with this repository's taxonomy, the scratch
/// declarations, two committed exports and a pin for each.
struct Root {
    at: PathBuf,
}

impl Root {
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-harvest-{}-{label}",
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
        let overlay = std::fs::read_to_string(repository.join(".headwater/overlay.yml"))
            .expect("the overlay reads");
        assert!(
            overlay.contains("\nadd:\n"),
            "the overlay opens its additions with `add:`, where the scratch kinds go"
        );
        std::fs::write(
            at.join(".headwater/overlay.yml"),
            overlay.replacen("\nadd:\n", &format!("\nadd:\n{DECLARED}\n"), 1),
        )
        .expect("the overlay writes");

        let root = Root { at };
        root.write("docs/solution/checkout.md", NOTE);
        root.write(EXPORT_A, &export("gold"));
        root.write(EXPORT_B, &export("bronze"));
        // The files this repository's language regime names outside the corpus
        // root. A scratch corpus that lacks them is refused for that, which is
        // a finding about the scratch and not about the pins.
        for stub in [
            "README.md",
            ".github/CONTRIBUTING.md",
            ".github/ISSUE_TEMPLATE/issue.md",
            ".github/SECURITY.md",
        ] {
            root.write(stub, "# Scratch\n\nThis file is a stub.\n");
        }

        let consumer = std::fs::read_to_string(repository.join(".headwater/taxonomy.yml"))
            .expect("the declaration reads");
        let pins = format!(
            "\nharvests:\n  repo-a:\n    at: {EXPORT_A}\n    digest: {}\n    channel: the tier's nightly harvest job\n    resolver: export-repo-a\n  repo-b:\n    at: {EXPORT_B}\n    digest: {}\n    channel: the tier's nightly harvest job\n    resolver: export-repo-b\n",
            root.digest(EXPORT_A),
            root.digest(EXPORT_B),
        );
        root.write(".headwater/taxonomy.yml", &format!("{consumer}{pins}"));

        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolved.code,
            Some(0),
            "the fixture resolves\n{}{}",
            resolved.out,
            resolved.err
        );
        root
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.at.join(relative);
        std::fs::create_dir_all(path.parent().expect("it has a parent"))
            .expect("the directory is made");
        std::fs::write(path, text).expect("the file writes");
    }

    fn digest(&self, relative: &str) -> String {
        headwater_hash::digest(&std::fs::read(self.at.join(relative)).expect("the export reads"))
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

    /// Every error a check reports on the solution note, one block each: the
    /// header line that names the file and the lines indented under it.
    fn errors(ran: &Ran) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let mut open = false;
        for line in ran.out.lines() {
            if line.starts_with("  ") && !line.starts_with("   ") {
                open = line.starts_with("  docs/solution/checkout.md") && line.contains("error");
                if open {
                    out.push(line.to_string());
                }
            } else if open && line.starts_with("    ") {
                let last = out.last_mut().expect("a block is open");
                last.push(' ');
                last.push_str(line.trim());
            } else {
                open = false;
            }
        }
        out
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

/// The note's errors that name an unresolved relation target.
fn unresolved(ran: &Ran) -> Vec<String> {
    Root::errors(ran)
        .into_iter()
        .filter(|block| block.contains("relation.target.unresolved"))
        .collect()
}

/// Both pins read, so both edges bind, each into its own repository.
#[test]
fn an_anchor_into_each_pinned_export_binds_there() {
    let root = Root::new("both");
    let ran = root.run(&["check", "--strict"]);
    assert_eq!(Root::errors(&ran), Vec::<String>::new(), "{ran:?}");
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert!(!ran.out.contains("unbound"), "both edges bind: {ran:?}");
}

/// The case the issue exists for. B's export is gone, with a warm cache. The
/// edge into B is reported and the finding names B's pin, the edge into A still
/// binds, and nothing binds B's edge to the `SVC-1` that A's export holds.
#[test]
fn a_missing_export_is_a_finding_that_names_its_pin_and_never_a_lookup_elsewhere() {
    let root = Root::new("missing");
    let warm = root.run(&["check", "--strict"]);
    assert_eq!(warm.code, Some(0), "the cache is populated: {warm:?}");

    std::fs::remove_file(root.at.join(EXPORT_B)).expect("B's export is removed");
    let ran = root.run(&["check", "--strict"]);
    assert_ne!(ran.code, Some(0), "the strict gate fails: {ran:?}");
    assert!(
        !ran.err.starts_with("0 served from cache"),
        "the second run reads a warm cache, so the verdict that moved is one a key moved: {}",
        ran.err
    );
    let unresolved = unresolved(&ran);
    assert_eq!(unresolved.len(), 1, "only the edge into B: {unresolved:?}");
    let finding = &unresolved[0];
    assert!(finding.contains("uses_service_in_b"), "{finding}");
    assert!(
        finding.contains("`repo-b`") && finding.contains(EXPORT_B),
        "the finding names B's pin and where it is: {finding}"
    );
    assert!(
        !finding.contains("repo-a") && !finding.contains("which this run does not have"),
        "the finding names B's pin, not A's and not a missing resolver: {finding}"
    );
    assert_eq!(
        Root::errors(&ran).len(),
        1,
        "nothing else is wrong: {ran:?}"
    );
}

/// B's export is back with one byte changed, so it is not the pinned artifact.
/// The result is the result of a missing file, and the message says why.
#[test]
fn an_export_that_is_not_the_pinned_artifact_binds_nothing_and_names_the_pin() {
    let root = Root::new("moved");
    let warm = root.run(&["check", "--strict"]);
    assert_eq!(warm.code, Some(0), "the cache is populated: {warm:?}");

    root.write(EXPORT_B, &export("silver"));
    let ran = root.run(&["check", "--strict"]);
    assert_ne!(ran.code, Some(0), "the strict gate fails: {ran:?}");
    let unresolved = unresolved(&ran);
    assert_eq!(unresolved.len(), 1, "only the edge into B: {unresolved:?}");
    let finding = &unresolved[0];
    assert!(finding.contains("uses_service_in_b"), "{finding}");
    assert!(
        finding.contains("`repo-b`") && finding.contains("not the pinned artifact"),
        "the finding names the pin and says the bytes moved: {finding}"
    );
    assert!(!finding.contains("repo-a"), "{finding}");
}

/// An identifier that B's export does not hold is reported against B, even
/// though A's export holds it. The far end owns identity, so nothing looks for
/// it anywhere else.
#[test]
fn an_identifier_only_the_other_export_holds_is_unresolved() {
    let root = Root::new("elsewhere");
    let pinned = std::fs::read_to_string(root.at.join(".headwater/taxonomy.yml"))
        .expect("the declaration reads");
    let before = root.digest(EXPORT_A);
    root.write(EXPORT_A, &export("gold").replace("\"SVC-1\"", "\"SVC-2\""));
    root.write(
        ".headwater/taxonomy.yml",
        &pinned.replace(&before, &root.digest(EXPORT_A)),
    );
    root.write(
        "docs/solution/checkout.md",
        &NOTE.replace(
            "  uses_service_in_b:\n    - SVC-1",
            "  uses_service_in_b:\n    - SVC-2",
        ),
    );
    let ran = root.run(&["check", "--strict"]);
    let unresolved = unresolved(&ran);
    assert_eq!(unresolved.len(), 2, "{unresolved:?}");
    assert!(
        unresolved
            .iter()
            .any(|finding| finding.contains("uses_service_in_b")
                && finding.contains("`repo-b`")
                && finding.contains("SVC-2")),
        "{unresolved:?}"
    );
}

/// The hand-written exports above copy the shape `headwater export` writes.
/// This case holds the copy to the original: it pins what the binary itself
/// exports, and an anchor into a document of that export binds.
#[test]
fn an_export_this_binary_wrote_is_one_the_resolver_reads() {
    let root = Root::new("real");
    let exported = root.run(&["export", "--format", "json"]);
    assert_eq!(exported.code, Some(0), "{exported:?}");
    assert!(exported.out.contains("\"HW-SOL-checkout\""), "{exported:?}");
    let pinned = std::fs::read_to_string(root.at.join(".headwater/taxonomy.yml"))
        .expect("the declaration reads");
    let before = root.digest(EXPORT_A);
    root.write(EXPORT_A, &exported.out);
    root.write(
        ".headwater/taxonomy.yml",
        &pinned.replace(&before, &root.digest(EXPORT_A)),
    );
    root.write(
        "docs/solution/checkout.md",
        &NOTE.replace(
            "  uses_service_in_a:\n    - SVC-1",
            "  uses_service_in_a:\n    - HW-SOL-checkout",
        ),
    );
    let ran = root.run(&["check", "--strict"]);
    assert_eq!(unresolved(&ran), Vec::<String>::new(), "{ran:?}");
    assert_eq!(ran.code, Some(0), "{ran:?}");
}
