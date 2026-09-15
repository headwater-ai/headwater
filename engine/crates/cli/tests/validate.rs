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
            &repository.join(".headwater/packages/headwater-standard"),
            &at.join(".headwater/packages/headwater-standard"),
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
    ///
    /// The anchor is one line and the operations go *above* it. An anchor
    /// that quoted the whole `shelves.tutorials` declaration is an anchor
    /// that an unrelated change to that declaration breaks, and #538
    /// declaring a display name on every shelf broke exactly that, with a
    /// failure that said nothing about why.
    fn founds_a_kind(&self) {
        let overlay = self.at.join(".headwater/overlay.yml");
        let text = std::fs::read_to_string(&overlay).expect("the overlay reads");
        let anchor = "  shelves.tutorials:\n";
        assert!(
            text.contains(anchor),
            "the overlay still declares a tutorials shelf"
        );
        let added = format!(
            "  kinds.playbook.is_a: governed_document\n  kinds.playbook.purpose: \
             behavior\n  shelves.playbooks: {{path: docs/playbooks/**, homogeneous: true, kind: \
             playbook}}\n\n{anchor}"
        );
        std::fs::write(&overlay, text.replacen(anchor, &added, 1)).expect("the overlay writes");
    }

    /// Take the display name off one shelf, leaving every other one declared.
    ///
    /// A removal from this repository's own overlay rather than a taxonomy
    /// written for the case, so what is measured is the reading of the
    /// declarations that ship.
    fn drops_a_display_name(&self) {
        let overlay = self.at.join(".headwater/overlay.yml");
        let text = std::fs::read_to_string(&overlay).expect("the overlay reads");
        let anchor = "    title: Tutorials\n";
        assert!(
            text.contains(anchor),
            "the overlay still gives the tutorials shelf a display name"
        );
        std::fs::write(&overlay, text.replacen(anchor, "", 1)).expect("the overlay writes");
    }

    /// Narrow the consumer's selection until its closure is incomplete.
    ///
    /// It removes one bundle from whatever `bundles:` names, rather than writing
    /// a selection of its own, so the case follows the declaration this
    /// repository ships instead of a copy of it that nothing holds current. The
    /// assertion is what makes that safe: a shipped selection that stops
    /// carrying `evidence-and-obligation` fails here by name, rather than
    /// quietly measuring a refusal these cases are not about.
    ///
    /// Returns what is left, so a case can say which bundles the advice is
    /// allowed to know about.
    fn drops_the_bundle_the_others_read(&self) -> Vec<String> {
        let path = self.at.join(".headwater/taxonomy.yml");
        let text = std::fs::read_to_string(&path).expect("the consumer declaration reads");
        let line = text
            .lines()
            .find(|line| line.trim_start().starts_with("bundles:"))
            .expect("the consumer declares `bundles:`")
            .to_string();
        let inside = line
            .split_once('[')
            .and_then(|(_, rest)| rest.split_once(']'))
            .map(|(inside, _)| inside)
            .expect("`bundles:` is a flow sequence");
        let held: Vec<String> = inside
            .split(',')
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty())
            .collect();
        assert!(
            held.iter().any(|name| name == DROPPED),
            "the shipped selection carries `{DROPPED}`, and these cases are about removing it: \
             {line}"
        );
        let kept: Vec<String> = held.into_iter().filter(|name| name != DROPPED).collect();
        let indent = &line[..line.len() - line.trim_start().len()];
        let replaced = format!("{indent}bundles: [{}]", kept.join(", "));
        std::fs::write(&path, text.replacen(&line, &replaced, 1))
            .expect("the consumer declaration writes");
        kept
    }
}

/// The bundle the shipped selection carries that the other two read.
const DROPPED: &str = "evidence-and-obligation";

/// The line the advice opens with. One copy, because three cases read it and
/// two of them read it for its absence.
const ADVICE: &str = "this bundle selection is incomplete";

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

/// The display-name block, in the direction a block can lose in silence.
///
/// This repository declares a display name on all thirteen of its shelves, so
/// the block prints at zero here. That is the reading a corpus wants and it is
/// also the reading that a block deleted by mistake produces, which is why the
/// count is asserted on the heading rather than inferred from the absence of
/// names under it.
#[test]
fn validate_states_that_every_shelf_carries_a_display_name() {
    let root = Root::new("no-bare-shelf");
    let ran = root.run(&["taxonomy", "validate"]);
    assert_eq!(
        ran.code,
        Some(0),
        "the copy of this repository is valid: {ran:?}"
    );
    assert!(
        ran.out
            .contains("shelves that print their key for want of a display name: 0"),
        "the block prints at zero: {ran:?}"
    );
    assert!(
        !ran.out.contains("Nothing here refuses. `shelves."),
        "the note prints only beside a name: {ran:?}"
    );
}

/// Clause 1 of [#538](https://github.com/headwater-ai/headwater/issues/538) as
/// a status code: a shelf that declares no display name is a decision and not
/// a defect.
///
/// The shelf is named and the verb exits 0. A refusal here would make
/// `shelves.<s>.title` required in everything but the meta-schema, and a
/// corpus whose keys are already the names its readers want would have to
/// restate each of them to pass.
#[test]
fn a_shelf_with_no_display_name_is_named_and_not_refused() {
    let root = Root::new("one-bare-shelf");
    root.drops_a_display_name();
    let ran = root.run(&["taxonomy", "validate"]);

    assert_eq!(
        ran.code,
        Some(0),
        "a shelf with no display name refuses nothing: {ran:?}"
    );
    assert!(
        ran.out
            .contains("shelves that print their key for want of a display name: 1"),
        "the count is on the heading: {ran:?}"
    );
    assert!(
        ran.out.contains("shelves.tutorials"),
        "the shelf is named: {ran:?}"
    );
    assert!(
        ran.out.contains("falls through to the key"),
        "and the reading is stated where the name is: {ran:?}"
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

/// The whole user-visible deliverable of
/// [#579](https://github.com/headwater-ai/headwater/issues/579), at the verb.
///
/// The library holds the derivation and `engine/crates/resolve/tests/selection.rs`
/// is where that is measured. Nothing there reaches the print. Both call sites
/// are one line each in `main.rs`, and deleting either one leaves that suite
/// green and this issue unfixed, so the text, the stream and the status code
/// are asserted here.
#[test]
fn resolve_names_the_bundle_an_incomplete_selection_left_out() {
    let root = Root::new("resolve-incomplete");
    let kept = root.drops_the_bundle_the_others_read();
    let ran = root.run(&["taxonomy", "resolve"]);

    assert_eq!(
        ran.code,
        Some(1),
        "an incomplete selection is refused: {ran:?}"
    );
    assert!(
        ran.err.contains("referential integrity"),
        "the refusal that reports it is the one it always was: {ran:?}"
    );
    assert!(
        ran.err.contains(ADVICE),
        "the advice reaches a reader: {ran:?}"
    );
    assert!(
        ran.err.contains(&format!("`{DROPPED}` declares")),
        "and it names the bundle to add, with what that bundle supplies: {ran:?}"
    );
    assert!(
        ran.err.contains(".headwater/taxonomy.yml"),
        "and the file to edit: {ran:?}"
    );

    // Every bundle the package ships that neither helps nor is selected. Taken
    // from the package rather than listed, so a library that grows a bundle
    // grows this assertion with it.
    for name in shipped_bundles() {
        if name == DROPPED || kept.contains(&name) {
            continue;
        }
        assert!(
            !ran.err.contains(&format!("`{name}`")),
            "the message names no bundle that does not help (`{name}`): {ran:?}"
        );
    }

    assert!(
        !ran.out.contains(ADVICE),
        "the advice is on standard error and never on standard output: {ran:?}"
    );
    assert!(
        !root.at.join(".headwater/taxonomy.lock").is_file(),
        "a refused taxonomy writes no lock: {ran:?}"
    );
}

/// The same advice at the other verb, which is a second call site and not the
/// same code path.
#[test]
fn validate_names_the_bundle_an_incomplete_selection_left_out() {
    let root = Root::new("validate-incomplete");
    root.drops_the_bundle_the_others_read();
    let ran = root.run(&["taxonomy", "validate"]);

    assert_eq!(ran.code, Some(1), "the taxonomy is not valid: {ran:?}");
    assert!(
        ran.out.contains("headwater/standard is not valid"),
        "the verdict is still on standard output: {ran:?}"
    );
    assert!(
        ran.err.contains("referential integrity"),
        "under the refusal that reports it: {ran:?}"
    );
    assert!(
        ran.err.contains(ADVICE) && ran.err.contains(&format!("`{DROPPED}` declares")),
        "the advice names the bundle to add: {ran:?}"
    );
    assert!(
        !ran.out.contains(ADVICE),
        "on standard error and never on standard output: {ran:?}"
    );
}

/// The quiet direction, which is the one a message can lose in silence.
///
/// A message that always prints is a message that will eventually name the
/// wrong thing, so the selection this repository actually ships must reach
/// neither stream.
#[test]
fn a_selection_whose_closure_is_complete_is_told_nothing_about_bundles() {
    let root = Root::new("complete-selection");
    let ran = root.run(&["taxonomy", "validate"]);

    assert_eq!(ran.code, Some(0), "the shipped selection is valid: {ran:?}");
    assert!(
        !ran.err.contains(ADVICE) && !ran.out.contains(ADVICE),
        "nothing is advised about a selection that resolves: {ran:?}"
    );

    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_eq!(resolved.code, Some(0), "and it resolves: {resolved:?}");
    assert!(
        !resolved.err.contains(ADVICE) && !resolved.out.contains(ADVICE),
        "at the other verb as well: {resolved:?}"
    );
}

/// Every bundle the copied package ships, read off the package.
fn shipped_bundles() -> Vec<String> {
    let at = repository().join(".headwater/packages/headwater-standard/bundles");
    let mut names: Vec<String> = std::fs::read_dir(&at)
        .expect("the bundle root reads")
        .filter_map(|entry| entry.ok())
        .filter(|entry| at.join(entry.file_name()).join("bundle.yml").is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert!(names.len() > 1, "the package ships bundles: {names:?}");
    names
}
