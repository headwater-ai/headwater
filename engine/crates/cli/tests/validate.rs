// SPDX-License-Identifier: Apache-2.0
//! `taxonomy validate` and `taxonomy resolve`, over the record of what an
//! overlay makes.
//!
//! # The defect this target exists for
//!
//! The record is a statement and never a refusal, and the difference between
//! the two is one exit status. A wiring that printed the block and also failed
//! the verb would read as correct in every log and would gate a repository on
//! its overlay order, which is the thing
//! [`headwater_resolve::Founding`] must never do — `founded.rs` carries why, as
//! a case. So every case here asserts the status code as well as the text, and
//! the zero case asserts that the block prints at all: a block that vanished
//! when it emptied would leave a reader unable to tell a quiet corpus from a
//! reading nobody ran.
//!
//! The second defect is the stream. `taxonomy resolve --check` prints one line
//! of standard output on success, and a differential over that line is how a
//! consumer reads the verb. The record therefore goes to standard error, and
//! the third case is the byte comparison that holds it there.
//!
//! # The root each case runs over
//!
//! This repository's own package, overlay and consumer declaration, copied into
//! a temporary root that removes itself. Copied rather than committed a second
//! time, for the reason `diff.rs` and `wiring.rs` both give: a taxonomy under
//! `fixtures/` is a schema that no gate holds current, and it would go stale in
//! silence.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// A repository root that removes itself.
///
/// `label` names the case and not the target. Cargo runs the cases of one
/// target as threads of one process, so a directory keyed on the process
/// identifier alone is a directory one case removes while another is reading
/// it.
struct Root {
    at: PathBuf,
}

impl Root {
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-validate-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");

        let repository = repository();
        copy(
            &repository.join("packages/headwater-standard"),
            &at.join("packages/headwater-standard"),
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
        Root { at }
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

    /// Give the overlay one operation that makes the kind it addresses.
    ///
    /// Three lines, in this order. The first reaches into a `kinds.playbook`
    /// that nothing under this overlay declares, so the resolver makes it and
    /// records the operation. The second reaches into what the first made, and
    /// is therefore *not* a second founding, which is the distinction the
    /// record exists to draw. The shelf keeps the coverage rule satisfied, so
    /// the verdict stays valid and the case measures the record rather than an
    /// unrelated refusal.
    fn founds_a_kind(&self) {
        let overlay = self.at.join(".headwater/overlay.yml");
        let text = std::fs::read_to_string(&overlay).expect("the overlay reads");
        let anchor = "  shelves.tutorials:\n    path: docs/tutorials/**\n    homogeneous: true\n    kind: tutorial\n";
        assert!(
            text.contains(anchor),
            "the overlay still carries the anchor"
        );
        let added = format!(
            "{anchor}\n  kinds.playbook.is_a: governed_document\n  kinds.playbook.purpose: \
             behavior\n  shelves.playbooks: {{path: docs/playbooks/**, homogeneous: true, kind: \
             playbook}}\n"
        );
        std::fs::write(&overlay, text.replacen(anchor, &added, 1)).expect("the overlay writes");
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

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

/// The index of the one line that starts with `prefix`.
fn line_of(text: &str, prefix: &str) -> usize {
    let found: Vec<usize> = text
        .lines()
        .enumerate()
        .filter(|(_, line)| line.starts_with(prefix))
        .map(|(index, _)| index)
        .collect();
    assert_eq!(found.len(), 1, "one line starts with `{prefix}`:\n{text}");
    found[0]
}

/// The quiet direction, which is the one a block can lose in silence.
#[test]
fn validate_states_that_no_operation_makes_what_it_addresses() {
    let root = Root::new("no-founding");
    let ran = root.run(&["taxonomy", "validate"]);
    assert_eq!(
        ran.code,
        Some(0),
        "the copy of this repository is valid: {ran:?}"
    );
    assert!(
        ran.out
            .contains("operations that make what they address: 0"),
        "the block prints at zero: {ran:?}"
    );

    let sources = line_of(&ran.out, "sources, in application order");
    let block = line_of(&ran.out, "operations that make what they address:");
    let rules = line_of(&ran.out, "rules");
    assert!(
        sources < block && block < rules,
        "the block sits between the sources and the rules: {ran:?}"
    );

    // Nothing was made, so nothing explains why nothing refuses.
    assert!(
        !ran.out.contains("Nothing here refuses a founding"),
        "the note prints only beside a founding: {ran:?}"
    );
}

/// The ruling as a status code.
///
/// The overlay makes a kind that nothing under it declares, which is exactly
/// the state [#193](https://github.com/headwater-ai/headwater/issues/193) asked
/// whether to refuse. It is named, and the verb exits 0.
#[test]
fn an_overlay_that_makes_what_it_addresses_is_named_and_not_refused() {
    let root = Root::new("one-founding");
    root.founds_a_kind();
    let ran = root.run(&["taxonomy", "validate"]);

    assert_eq!(ran.code, Some(0), "a founding refuses nothing: {ran:?}");
    assert!(
        ran.out
            .contains("operations that make what they address: 1"),
        "the count is on the heading: {ran:?}"
    );
    assert!(
        ran.out.contains("add.kinds.playbook.is_a"),
        "the operation is named as an overlay writes it: {ran:?}"
    );
    assert!(
        ran.out.contains("makes `kinds.playbook`"),
        "the shallowest key it makes is named: {ran:?}"
    );
    assert!(
        ran.out.contains(".headwater/overlay.yml"),
        "the source that carries it is named: {ran:?}"
    );
    assert!(
        ran.out.contains("Nothing here refuses a founding"),
        "the reason prints beside the founding: {ran:?}"
    );

    // The second operation reaches into what the first made, so it is not a
    // second founding. A reading that counted it would report the shape of an
    // overlay rather than what the overlay makes.
    assert!(
        !ran.out.contains("add.kinds.playbook.purpose"),
        "only the operation that makes the key is reported: {ran:?}"
    );
}

/// The stream, and the lock.
///
/// `taxonomy resolve --check` prints one line of standard output on success,
/// and a consumer reads that line. So the record goes to standard error, where
/// a byte comparison of the verdict does not reach it, and the lock is written
/// all the same.
#[test]
fn resolve_reports_a_founding_on_standard_error_and_still_writes_the_lock() {
    let root = Root::new("resolve-founding");
    root.founds_a_kind();
    let ran = root.run(&["taxonomy", "resolve"]);

    assert_eq!(ran.code, Some(0), "the lock is written: {ran:?}");
    assert!(
        ran.err
            .contains("operations that make what they address: 1"),
        "the record is on standard error: {ran:?}"
    );
    assert!(
        !ran.out.contains("operations that make what they address"),
        "and not on standard output: {ran:?}"
    );
    assert!(
        root.at.join(".headwater/taxonomy.lock").is_file(),
        "the lock is on disk: {ran:?}"
    );

    // `--check` over the lock that was just written: the same record on
    // standard error, and standard output still exactly one line.
    let checked = root.run(&["taxonomy", "resolve", "--check"]);
    assert_eq!(checked.code, Some(0), "the lock is current: {checked:?}");
    assert!(
        checked
            .err
            .contains("operations that make what they address: 1"),
        "`--check` reports it too: {checked:?}"
    );
    assert_eq!(
        checked.out.lines().count(),
        1,
        "the verdict is still one line: {checked:?}"
    );
}
