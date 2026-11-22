// SPDX-License-Identifier: Apache-2.0
//! What `headwater generate` says about a shelf that an overlay adds.
//!
//! # The defect this target exists for
//!
//! [#1988](https://github.com/headwater-ai/headwater/issues/1988) found that a
//! shelf an adopter's overlay adds got no index and no line. The base package
//! declares a `shelf_index` for its own two shelves alone, so `generate` never
//! visited the new shelf, and the report said nothing about it. The adopter
//! could not tell a shelf the engine skipped from a shelf it forgot.
//!
//! So the first case adds a shelf with one document and no projection, and
//! asks for one printed line that names the shelf and the declaration that
//! would add an index. The second case adds that declaration, and asks for the
//! index and for no such line.
//!
//! # Why these cases drive the binary
//!
//! Only `taxonomy resolve` composes an overlay into the lock, and the claim is
//! about an overlay-added shelf. So each case assembles a scratch repository
//! with this repository's taxonomy, resolves it, and runs the built binary.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// The declarations the scratch overlay adds: one kind on one new shelf.
const DECLARED: &str = "
  identifier_schemes.guide_id:
    pattern: \"{namespace}-GDE-{slug}\"
    namespace: HW
    allocation: minted-once

  kinds.guide:
    is_a: governed_document
    purpose: rationale
    voice: declarative
    lifecycle: standard
    language: ste_house
    identifier: {scheme: guide_id}

  shelves.guides:
    title: The guides
    path: docs/guides/**
    homogeneous: true
    kind: guide
";

/// The projection that the second case adds, under `add_to: projections:`.
const INDEXED: &str = "    - {kind: shelf_index, for: [guides], output: \"{shelf}/README.md\"}\n";

/// The projection that the third case adds: a `shelf_index` with no `for`,
/// which covers every shelf.
const EVERY: &str = "    - {kind: shelf_index, output: \"{shelf}/README.md\"}\n";

/// Where `add_to: projections:` opens in this repository's overlay.
const PROJECTIONS: &str = "\nadd_to:\n\n  projections:\n";

/// The heading `generate` prints its unwritten lines under.
const UNWRITTEN: &str = "what this verb does not write, and why";

/// A scratch repository with this repository's taxonomy and the new shelf.
struct Root {
    at: PathBuf,
}

impl Root {
    fn new(label: &str, projection: Option<&str>) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-overlay-shelf-index-{}-{label}",
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
        std::fs::copy(
            repository.join(".headwater/taxonomy.yml"),
            at.join(".headwater/taxonomy.yml"),
        )
        .expect("the declaration copies");
        let overlay = std::fs::read_to_string(repository.join(".headwater/overlay.yml"))
            .expect("the overlay reads");
        assert!(
            overlay.contains("\nadd:\n"),
            "the overlay opens its additions with `add:`, where the scratch kind goes"
        );
        assert_eq!(
            overlay.matches(PROJECTIONS).count(),
            1,
            "the overlay opens its projections once, under `add_to:`"
        );
        let mut overlay = overlay.replacen("\nadd:\n", &format!("\nadd:\n{DECLARED}\n"), 1);
        if let Some(projection) = projection {
            overlay = overlay.replacen(PROJECTIONS, &format!("{PROJECTIONS}{projection}"), 1);
        }
        std::fs::write(at.join(".headwater/overlay.yml"), overlay).expect("the overlay writes");

        let root = Root { at };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolved.code,
            Some(0),
            "the fixture resolves\n{}{}",
            resolved.out,
            resolved.err
        );
        let written = root.run(&["new", "guide", "--title", "Alpha"]);
        assert_eq!(written.code, Some(0), "the guide is written\n{written:?}");
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

    fn has(&self, path: &str) -> bool {
        self.at.join(path).exists()
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

/// The shelves the unwritten block names as covered by no `shelf_index`.
fn uncovered(out: &str) -> Vec<String> {
    let Some((_, block)) = out.split_once(UNWRITTEN) else {
        return Vec::new();
    };
    block
        .lines()
        .filter_map(|line| line.strip_prefix("  shelf_index shelf `"))
        .filter_map(|rest| rest.strip_suffix('`'))
        .map(str::to_string)
        .collect()
}

/// The lines of the unwritten block that are about the `guides` shelf, each
/// with the reason line that follows it.
fn about_guides(out: &str) -> Vec<String> {
    let Some((_, block)) = out.split_once(UNWRITTEN) else {
        return Vec::new();
    };
    let lines: Vec<&str> = block.lines().collect();
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.starts_with("  shelf_index ") && line.contains("`guides`"))
        .map(|(at, line)| format!("{line}\n{}", lines.get(at + 1).unwrap_or(&"")))
        .collect()
}

/// A shelf the overlay adds, with a document and no `shelf_index`, gets one
/// printed line that names it and names the declaration that would add one.
#[test]
fn an_overlay_shelf_with_no_shelf_index_gets_one_line_naming_the_declaration() {
    let root = Root::new("uncovered", None);
    assert!(root.has("docs/guides"), "the guide is on its shelf");
    let ran = root.run(&["generate"]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    let lines = about_guides(&ran.out);
    assert_eq!(lines.len(), 1, "one line names the shelf\n{}", ran.out);
    let line = &lines[0];
    assert!(line.contains("shelf `guides`"), "{line}");
    assert!(line.contains("no `shelf_index` declaration"), "{line}");
    assert!(line.contains("`for`"), "{line}");
    assert!(line.contains("add_to: projections:"), "{line}");
    assert!(
        line.contains("{kind: shelf_index, for: [guides], output: \"{shelf}/README.md\"}"),
        "{line}"
    );
    assert!(!root.has("docs/guides/README.md"), "no index is written");
    // Every other shelf this repository's taxonomy declares is named by a
    // `shelf_index` except two: `interfaces`, which has a `verb_index`, and
    // `library_doctrine`. A covered shelf gets no such line, empty or not.
    let mut named = uncovered(&ran.out);
    named.sort();
    assert_eq!(
        named,
        ["guides", "interfaces", "library_doctrine"],
        "{}",
        ran.out
    );
}

/// Adding the declaration that line names writes the index and removes the
/// line.
#[test]
fn the_declaration_the_line_names_writes_the_index_and_removes_the_line() {
    let root = Root::new("covered", Some(INDEXED));
    let ran = root.run(&["generate"]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert!(
        root.has("docs/guides/README.md"),
        "the index is written\n{}",
        ran.out
    );
    assert_eq!(about_guides(&ran.out), Vec::<String>::new(), "{}", ran.out);
}

/// A `shelf_index` with no `for` covers every shelf, so no shelf gets the line.
#[test]
fn a_shelf_index_with_no_for_covers_every_shelf_and_leaves_no_line() {
    let root = Root::new("every", Some(EVERY));
    // The run refuses, because the copied `docs/taxonomies/README.md` carries
    // no generated-file marker and this declaration now covers that shelf too.
    // The report is what this case reads, and a refused run prints it whole.
    let ran = root.run(&["generate"]);
    assert!(
        ran.out.contains("shelf_index docs/guides/README.md"),
        "{ran:?}"
    );
    assert_eq!(uncovered(&ran.out), Vec::<String>::new(), "{}", ran.out);
}
