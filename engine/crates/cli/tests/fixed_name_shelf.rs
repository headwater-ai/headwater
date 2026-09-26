// SPDX-License-Identifier: Apache-2.0
//! What `headwater new` does with a shelf whose path fixes the file name.
//!
//! # The defect this target exists for
//!
//! [#1096](https://github.com/headwater-ai/headwater/issues/1096) found two
//! shapes of shelf that no route of this verb could write. A fixed file name
//! under a glob, such as `docs/modules/*/README.md`, was refused because the
//! glob names no directory, and the refusal offered no way out. A shelf that is
//! one file, such as `docs/INDEX.md`, was read as a directory, so the verb
//! composed `docs/INDEX.md/<slug>.md` and then reported its own defect.
//!
//! [#1136](https://github.com/headwater-ai/headwater/issues/1136) found that
//! `--directory docs/modules/*` wrote a directory literally named `*`, because
//! the shelf's glob matched it, and that two refusals of the flag were held by
//! no case here. So one case refuses a glob character in a segment, one
//! refuses the flag on a shelf that decides its own directory, and one
//! refuses a `..` segment for that cause and no other.
//!
//! # Why these cases drive the binary
//!
//! The claim an adopter depends on is that the file lands and is read back as
//! its kind. A case that called the scaffolder would prove the placement and
//! not the census that reads it, so each case assembles a scratch repository,
//! runs the built binary, and asks `explain` what the written path is.
//!
//! # Why the harness declares its own kinds
//!
//! The kind in this repository that sits on a fixed-name shelf sits on two
//! shelves, and `new` refuses it for that reason first. So the scratch overlay
//! declares one kind on each shape, each on one shelf.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// The declarations the scratch overlay adds, one kind on each shape of shelf.
const DECLARED: &str = "
  identifier_schemes.module_page_id:
    pattern: \"{namespace}-MOD-{slug}\"
    namespace: HW
    allocation: minted-once

  kinds.module_page:
    is_a: governed_document
    purpose: rationale
    voice: declarative
    lifecycle: standard
    language: ste_house
    identifier: {scheme: module_page_id}

  shelves.module_pages:
    title: The module pages
    path: docs/modules/*/README.md
    homogeneous: true
    kind: module_page

  identifier_schemes.corpus_index_id:
    pattern: \"{namespace}-IDX-{slug}\"
    namespace: HW
    allocation: minted-once

  kinds.corpus_index:
    is_a: governed_document
    purpose: rationale
    voice: declarative
    lifecycle: standard
    language: ste_house
    identifier: {scheme: corpus_index_id}

  shelves.corpus_index:
    title: The corpus index
    path: docs/INDEX.md
    homogeneous: true
    kind: corpus_index
";

/// A scratch repository with this repository's taxonomy and the two kinds.
struct Root {
    at: PathBuf,
}

impl Root {
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-fixed-name-shelf-{}-{label}",
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
            "the overlay opens its additions with `add:`, where the scratch kinds go"
        );
        std::fs::write(
            at.join(".headwater/overlay.yml"),
            overlay.replacen("\nadd:\n", &format!("\nadd:\n{DECLARED}\n"), 1),
        )
        .expect("the overlay writes");

        let root = Root { at };
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

/// With no directory the verb asks for one, names the flag, and writes nothing.
#[test]
fn a_fixed_name_under_a_glob_with_no_directory_names_the_flag() {
    let root = Root::new("no-directory");
    let ran = root.run(&["new", "module_page", "--title", "Alpha"]);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(ran.err.contains("docs/modules/*/README.md"), "{ran:?}");
    assert!(ran.err.contains("--directory"), "{ran:?}");
    assert!(!root.has("docs/modules"), "nothing was written: {ran:?}");
}

/// With a directory on the shelf the file lands, and the census reads it back
/// as its kind. This is the case the issue exists for.
#[test]
fn a_fixed_name_under_a_glob_lands_where_the_caller_names_and_reads_back() {
    let root = Root::new("named");
    let ran = root.run(&[
        "new",
        "module_page",
        "--title",
        "Alpha",
        "--directory",
        "docs/modules/alpha",
    ]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert!(root.has("docs/modules/alpha/README.md"), "{ran:?}");

    let explained = root.run(&["explain", "docs/modules/alpha/README.md"]);
    assert_eq!(explained.code, Some(0), "{explained:?}");
    assert!(
        explained.out.contains("module_page"),
        "the written file is read back as its kind\n{explained:?}"
    );
}

/// A directory off the shelf is the caller's input, refused, and not written.
#[test]
fn a_directory_off_the_shelf_is_refused_and_nothing_is_written() {
    let root = Root::new("off-shelf");
    let ran = root.run(&[
        "new",
        "module_page",
        "--title",
        "Alpha",
        "--directory",
        "docs/other/alpha",
    ]);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(ran.err.contains("docs/modules/*/README.md"), "{ran:?}");
    assert!(
        !ran.err.contains("defect"),
        "not blamed on the scaffolder: {ran:?}"
    );
    assert!(!root.has("docs/other"), "nothing was written: {ran:?}");
}

/// A shelf that is one file is written once, and a second run refuses and
/// leaves the first file's bytes as they were.
#[test]
fn a_single_file_shelf_is_written_once_and_never_overwritten() {
    let root = Root::new("single");
    let first = root.run(&["new", "corpus_index", "--title", "The index"]);
    assert_eq!(first.code, Some(0), "{first:?}");
    let written = std::fs::read(root.at.join("docs/INDEX.md")).expect("the index was written");

    let second = root.run(&["new", "corpus_index", "--title", "Another index"]);
    assert_eq!(second.code, Some(1), "{second:?}");
    assert!(second.err.contains("docs/INDEX.md"), "{second:?}");
    assert_eq!(
        std::fs::read(root.at.join("docs/INDEX.md")).expect("the index reads"),
        written,
        "the second run left the file alone"
    );
}

/// A `--directory` segment that holds a glob character is refused, because
/// the verb reads the directory as a literal path. Before #1136 the directory
/// `docs/modules/*` made `docs/modules/*/README.md`, which the shelf's own
/// glob matches, and the verb wrote a directory literally named `*`.
#[test]
fn a_directory_with_a_glob_character_is_refused_and_nothing_is_written() {
    for (label, directory, segment) in [
        ("glob-star", "docs/modules/*", "`*`"),
        ("glob-question", "docs/modules/a?b", "`a?b`"),
        ("glob-bracket", "docs/modules/[ab]", "`[ab]`"),
    ] {
        let root = Root::new(label);
        let ran = root.run(&[
            "new",
            "module_page",
            "--title",
            "Alpha",
            "--directory",
            directory,
        ]);
        assert_eq!(ran.code, Some(1), "{directory}: {ran:?}");
        assert!(
            ran.err.contains(&format!("the segment {segment}")),
            "{directory}: the refusal names the segment\n{ran:?}"
        );
        assert!(ran.err.contains("docs/modules/*/README.md"), "{ran:?}");
        assert!(
            !ran.err.contains("defect"),
            "not blamed on the scaffolder: {ran:?}"
        );
        assert!(!root.has("docs/modules"), "nothing was written: {ran:?}");
    }
}

/// A shelf that is one file decides its own directory, so a `--directory` on
/// it is refused rather than silently ignored.
#[test]
fn a_directory_for_a_shelf_that_decides_it_is_refused_and_nothing_is_written() {
    let root = Root::new("not-asked");
    let ran = root.run(&[
        "new",
        "corpus_index",
        "--title",
        "The index",
        "--directory",
        "docs",
    ]);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(ran.err.contains("--directory"), "{ran:?}");
    assert!(ran.err.contains("docs/INDEX.md"), "{ran:?}");
    assert!(ran.err.contains("decides the directory"), "{ran:?}");
    assert!(!root.has("docs/INDEX.md"), "nothing was written: {ran:?}");
}

/// A `..` segment is refused for what it is. The one-segment form matters: a
/// `*` in the shelf's pattern may match `..`, so the pattern check alone would
/// not stop `docs/modules/..`, and only the segment check names the cause.
#[test]
fn a_directory_that_climbs_out_is_refused_and_nothing_is_written() {
    let root = Root::new("climbs-out");
    let ran = root.run(&[
        "new",
        "module_page",
        "--title",
        "Alpha",
        "--directory",
        "docs/modules/..",
    ]);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(
        ran.err
            .contains("a `.`, `..` or empty segment names no directory"),
        "the refusal names the `..` segment as its cause\n{ran:?}"
    );
    assert!(!root.has("docs/README.md"), "nothing was written: {ran:?}");
    assert!(!root.has("docs/modules"), "nothing was written: {ran:?}");
}
