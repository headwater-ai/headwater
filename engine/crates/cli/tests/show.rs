// SPDX-License-Identifier: Apache-2.0
//! [#740](https://github.com/headwater-ai/headwater/issues/740): `headwater
//! show` prints the bytes of one document, found the way `explain` finds it.
//!
//! # What this target holds
//!
//! Two defects, one case each.
//!
//! A dump that re-renders the document rather than copying it. A reader who
//! pipes `show` into a file or a diff must get the file on disk back, so the
//! fixture document carries the three things a text round trip loses: CRLF
//! line endings, a character outside ASCII, and no newline at the end. The
//! comparison is on raw bytes and never through a lossy decode.
//!
//! A second resolver. `show` finds a document through the same function
//! `explain` does, so a target that `explain` refuses is refused by `show`
//! in the same sentence. The refusal names no verb, so the two standard
//! error streams are equal outright. A copy of the resolver drifts, which is
//! what #319 and #845 each had to repair in `explain` alone.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
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

/// The path of the one document this target writes, relative to the root.
const DOCUMENT: &str = "docs/decisions/0001-a-document-show-prints.md";

/// The identifier that document declares.
const IDENTIFIER: &str = "HW-DR-0001";

/// The front matter is LF, because the reader is not what this target tests.
/// The body carries CRLF, a character outside ASCII, and no final newline.
fn document() -> Vec<u8> {
    let front = "---\nid: HW-DR-0001\ntitle: A document show prints\nstatus: current\nstatus_since: 2026-09-27\nlast_verified: 2026-09-27\nsummary: A fixture for show, whose body a text round trip would change.\n---\n";
    let body = "\r\n# A document show prints\r\n\r\n## Context\r\n\r\nThe caf\u{e9} line ends in CRLF.\r\n\r\n## Decision\r\n\r\nThe last line has no newline.";
    [front.as_bytes(), body.as_bytes()].concat()
}

/// A scratch corpus over this repository's own taxonomy, with one decision
/// written in. The maintained source rather than the vendored artifact, the
/// same choice `classify.rs` makes, so this target does not go stale under a
/// version bump.
struct Root {
    at: PathBuf,
}

impl Root {
    /// `label` names the case, because cargo runs the cases of one target as
    /// threads of one process.
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-show-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");

        let repository = repository();
        copy(
            &repository.join("taxonomy-source/headwater-standard"),
            &at.join(".headwater/packages/headwater-standard"),
        );
        repoint_bundles(&at.join(".headwater/packages/headwater-standard"));
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
        let path = at.join(DOCUMENT);
        std::fs::create_dir_all(path.parent().expect("it has a parent"))
            .expect("the shelf is made");
        std::fs::write(&path, document()).expect("the document writes");

        let root = Root { at };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolved.status.code(),
            Some(0),
            "the fixture resolves\n{}",
            String::from_utf8_lossy(&resolved.stderr)
        );
        root
    }

    fn run(&self, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(arguments)
            .arg("--root")
            .arg(&self.at)
            .output()
            .expect("the binary runs")
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

/// The decisive case. Both spellings of the target print the file on disk,
/// byte for byte, with nothing on standard error; and each target `explain`
/// refuses is refused by `show` with no byte on standard output and the same
/// standard error.
#[test]
fn show_writes_the_bytes_on_disk_for_an_identifier_and_for_its_path() {
    let root = Root::new("bytes");
    let on_disk = std::fs::read(root.at.join(DOCUMENT)).expect("the document reads");
    assert_eq!(on_disk, document(), "the fixture wrote what it meant to");

    // The identifier must resolve through the typed reader, or the path case
    // alone could pass on an untyped row and the identifier case would be
    // testing nothing.
    let explained = root.run(&["explain", IDENTIFIER]);
    assert_eq!(
        explained.status.code(),
        Some(0),
        "the fixture document is typed and carries its identifier: {}",
        String::from_utf8_lossy(&explained.stderr)
    );

    for target in [IDENTIFIER, DOCUMENT] {
        let shown = root.run(&["show", target]);
        assert_eq!(
            shown.status.code(),
            Some(0),
            "`show {target}`: {}",
            String::from_utf8_lossy(&shown.stderr)
        );
        assert!(
            shown.stderr.is_empty(),
            "`show {target}` writes nothing on standard error: {}",
            String::from_utf8_lossy(&shown.stderr)
        );
        assert!(
            shown.stdout == on_disk,
            "`show {target}` writes the bytes on disk and nothing else:\n  got  {:?}\n  want {:?}",
            shown.stdout,
            on_disk
        );
    }

    for target in ["HW-DR-9999", "HW-DR-004", "engine/nowhere/at-all.rs"] {
        let shown = root.run(&["show", target]);
        let explained = root.run(&["explain", target]);
        assert_eq!(
            shown.status.code(),
            Some(1),
            "`show {target}` refuses: {}",
            String::from_utf8_lossy(&shown.stderr)
        );
        assert!(
            shown.stdout.is_empty(),
            "`show {target}` writes nothing on standard output"
        );
        assert_eq!(
            explained.status.code(),
            Some(1),
            "`explain {target}` refuses too"
        );
        assert_eq!(
            String::from_utf8_lossy(&shown.stderr),
            String::from_utf8_lossy(&explained.stderr),
            "`show` and `explain` refuse `{target}` in the same sentence, because one resolver answers both"
        );
    }
}

/// A bare `show` names what it takes, in the shape a bare `explain` does.
#[test]
fn a_bare_show_names_what_it_takes() {
    let root = Root::new("bare");
    let shown = root.run(&["show"]);
    assert_eq!(shown.status.code(), Some(1));
    assert!(shown.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&shown.stderr).contains("`show` takes a path or an identifier"),
        "{}",
        String::from_utf8_lossy(&shown.stderr)
    );
}

/// Repoint `contents.bundles` in a scratch copy of the authored manifest, as
/// `classify.rs` does and for the reason it states.
fn repoint_bundles(package: &Path) {
    let up = "../".repeat(headwater_resolve::package::PACKAGES.split('/').count() + 1);
    let manifest = package.join(headwater_resolve::package::MANIFEST);
    let text = std::fs::read_to_string(&manifest).expect("the scratch manifest reads");
    let from = "  bundles: ../../docs/taxonomies";
    assert!(text.contains(from), "the authored manifest states `{from}`");
    let to = format!("  bundles: {up}docs/taxonomies");
    std::fs::write(&manifest, text.replace(from, &to)).expect("the scratch manifest writes");
}
