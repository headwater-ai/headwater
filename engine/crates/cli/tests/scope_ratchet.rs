// SPDX-License-Identifier: Apache-2.0
//! `headwater check --strict --change` fails on a file the change adds inside
//! the governed scope that no governance edge reaches, where the control of
//! `relation.scope.ungoverned` is at `posture: blocking` (#1573).
//!
//! The stub root copies this repository's overlay, which declares
//! `tools/**` in `anchors.code_path.scope` and promotes CT-REL-12 to
//! `blocking`. One decision governs `tools/governed.sh`. Each arm writes one
//! manifest, so the three directions of the gate differ in one line: an
//! ungoverned add fails, a governed add passes, and the same ungoverned add
//! passes once the overlay leaves the control at the package's advisory
//! posture.

mod common;
use common::Root;

/// The override line this repository's overlay carries.
const PROMOTED: &str = "  controls.CT-REL-12.posture: blocking\n";

/// One decision that governs `tools/governed.sh`, and its claim file, so that
/// no error of another rule stands in for the gate this file reads.
fn govern(at: &std::path::Path) {
    let id = "HW-DR-0002";
    let file = "0002-the-decision-that-governs-one-tool.md";
    let text = format!(
        "---\nid: {id}\ntitle: The decision that governs one tool\nstatus: \
         current\nstatus_since: 2026-08-01\nlast_verified: 2026-08-01\nsummary: One \
         decision that governs one tool.\nprovenance:\n  warrant: asserted\n  agency: \
         human\n  evidence_basis: unevidenced\nrelations:\n  governs:\n    - to: \
         tools/governed.sh\n---\n\n# The decision that governs one tool\n\n## Context\n\nA \
         fixture.\n\n## Decision\n\nIt governs one file.\n\n## Consequences\n\nThe check \
         reads it.\n"
    );
    std::fs::write(at.join("docs/decisions").join(file), text).expect("the decision writes");
    // A claim for every decision of the stub, its own included.
    let claims = at.join(".headwater/ids/decision_id");
    std::fs::create_dir_all(&claims).expect("the claim directory is made");
    for entry in std::fs::read_dir(at.join("docs/decisions")).expect("the shelf reads") {
        let path = entry.expect("the entry reads").path();
        let text = std::fs::read_to_string(&path).expect("the decision reads");
        let Some(id) = text.lines().find_map(|line| line.strip_prefix("id: ")) else {
            continue;
        };
        let name = path.file_name().expect("a file name").to_string_lossy();
        std::fs::write(claims.join(id), format!("docs/decisions/{name}\n"))
            .expect("the claim writes");
    }
}

/// The stub root with both tools on disk and the decision that governs one.
/// `advisory` sets CT-REL-12 back to the package's posture.
fn root(label: &str, advisory: bool) -> Root {
    Root::shaped(label, move |at| {
        std::fs::write(at.join("tools/governed.sh"), "echo governed\n").expect("a tool writes");
        std::fs::write(at.join("tools/ungoverned.sh"), "echo ungoverned\n").expect("a tool writes");
        govern(at);
        let file = at.join(".headwater/overlay.yml");
        let source = std::fs::read_to_string(&file).expect("the overlay reads");
        assert!(
            source.contains(PROMOTED),
            "this repository's overlay promotes CT-REL-12, and this case copies it"
        );
        if advisory {
            // The package's posture, stated: an empty `override` block is a
            // scalar the resolver refuses.
            let advised = source.replacen(PROMOTED, "  controls.CT-REL-12.posture: advisory\n", 1);
            std::fs::write(&file, advised).expect("the overlay writes");
        }
    })
}

/// A strict run over a change that adds the one path.
fn strict_over(root: &Root, added: &str) -> common::Ran {
    let manifest = root.at.join("change.manifest");
    std::fs::write(&manifest, format!("headwater change 1\nadded\t{added}\n"))
        .expect("the manifest writes");
    root.run(&[
        "check",
        "--strict",
        "--change",
        manifest.to_str().expect("a UTF-8 path"),
    ])
}

fn findings(out: &str) -> usize {
    out.matches("relation.scope.ungoverned (").count()
}

#[test]
fn a_strict_run_fails_on_an_ungoverned_add_when_the_control_blocks() {
    let root = root("scope-ratchet-blocking", false);
    let ran = strict_over(&root, "tools/ungoverned.sh");
    assert_eq!(ran.code, Some(1), "{}{}", ran.out, ran.err);
    assert_eq!(findings(&ran.out), 1, "{}", ran.out);
    assert!(ran.out.contains("blocked by CT-REL-12"), "{}", ran.out);
    assert!(
        ran.out.contains("`tools/**`"),
        "the pattern is named: {}",
        ran.out
    );
}

#[test]
fn a_strict_run_passes_a_governed_add() {
    let root = root("scope-ratchet-governed", false);
    let ran = strict_over(&root, "tools/governed.sh");
    assert_eq!(ran.code, Some(0), "{}{}", ran.out, ran.err);
    assert_eq!(findings(&ran.out), 0, "{}", ran.out);
}

#[test]
fn an_advisory_control_reports_the_add_and_passes_the_strict_gate() {
    let root = root("scope-ratchet-advisory", true);
    let ran = strict_over(&root, "tools/ungoverned.sh");
    assert_eq!(ran.code, Some(0), "{}{}", ran.out, ran.err);
    assert_eq!(
        findings(&ran.out),
        1,
        "the finding stands, at a warning: {}",
        ran.out
    );
}

/// The stub moved into `sub/` of a git repository whose top level is the
/// scratch root, with every file committed but `sub/tools/ungoverned.sh`, so
/// that `headwater change` names it `added` from the git top level (#2098).
/// The second value is a directory beside the repository for the manifest,
/// which `headwater change` refuses inside the tree it compares.
fn subfolder(label: &str) -> (Root, Scratch) {
    let root = root(label, false);
    let sub = root.at.join("sub");
    std::fs::create_dir(&sub).expect("the subfolder is made");
    for entry in std::fs::read_dir(&root.at).expect("the root reads") {
        let from = entry.expect("the entry reads").path();
        if from != sub {
            let to = sub.join(from.file_name().expect("a name"));
            std::fs::rename(&from, to).expect("the entry moves");
        }
    }
    let tool = sub.join("tools/ungoverned.sh");
    std::fs::remove_file(&tool).expect("the tool is set aside");
    git(&root, &["init", "-q"]);
    git(&root, &["add", "-A"]);
    git(
        &root,
        &[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "commit",
            "-qm",
            "base",
        ],
    );
    std::fs::write(&tool, "echo ungoverned\n").expect("the tool writes");
    let name = root.at.file_name().expect("a name").to_string_lossy();
    let out = Scratch(root.at.with_file_name(format!("{name}-out")));
    let _ = std::fs::remove_dir_all(&out.0);
    std::fs::create_dir_all(&out.0).expect("the out directory is made");
    (root, out)
}

/// A directory that removes itself, for the manifest `headwater change` writes.
struct Scratch(std::path::PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn git(root: &Root, arguments: &[&str]) {
    let mut command = std::process::Command::new("git");
    common::fence(&mut command, &root.at);
    let status = command
        .arg("-C")
        .arg(&root.at)
        .args(arguments)
        .status()
        .expect("git runs");
    assert!(status.success(), "git {arguments:?}");
}

/// The binary with `--root <outer>/sub`, fenced at the outer repository's
/// parent so that git finds the outer repository and nothing above it.
fn in_sub(root: &Root, arguments: &[&str]) -> common::Ran {
    let output = root
        .command()
        .args(arguments)
        .arg("--root")
        .arg(root.at.join("sub"))
        .output()
        .expect("the binary runs");
    common::Ran {
        code: output.status.code(),
        out: String::from_utf8_lossy(&output.stdout).into_owned(),
        err: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// A strict run from the subfolder over the manifest at `manifest`.
fn strict_in_sub(root: &Root, manifest: &std::path::Path) -> common::Ran {
    let manifest = manifest.to_str().expect("a UTF-8 path");
    in_sub(root, &["check", "--strict", "--change", manifest])
}

#[test]
fn a_subfolder_root_raises_the_ungoverned_add_from_the_manifest_headwater_change_wrote() {
    let (root, out) = subfolder("scope-ratchet-subfolder");
    let dir = out.0.join("change");
    let made = in_sub(
        &root,
        &["change", "HEAD", dir.to_str().expect("a UTF-8 path")],
    );
    assert_eq!(made.code, Some(0), "{}{}", made.out, made.err);
    let manifest = dir.join("manifest");
    let text = std::fs::read_to_string(&manifest).expect("the manifest reads");
    assert!(
        text.lines()
            .any(|line| line == "added\tsub/tools/ungoverned.sh"),
        "headwater change writes from the git top level: {text}"
    );
    let ran = strict_in_sub(&root, &manifest);
    assert_eq!(ran.code, Some(1), "{}{}", ran.out, ran.err);
    assert_eq!(findings(&ran.out), 1, "{}", ran.out);
    assert!(ran.out.contains("tools/ungoverned.sh"), "{}", ran.out);
    assert!(ran.out.contains("`tools/**`"), "{}", ran.out);
}

/// A path outside the root is named as the manifest wrote it and reaches no
/// rule, even where it would match a scope pattern read from the root:
/// `tools/outside.sh` at the git top level is not `tools/**` of the corpus.
#[test]
fn a_path_outside_the_subfolder_root_is_named_and_reaches_no_rule() {
    let (root, out) = subfolder("scope-ratchet-outside");
    let manifest = out.0.join("manifest");
    std::fs::write(
        &manifest,
        "headwater change 1\nadded\tother/x.sh\nadded\ttools/outside.sh\n\
         added\tsub/tools/ungoverned.sh\n",
    )
    .expect("the manifest writes");
    let ran = strict_in_sub(&root, &manifest);
    // One finding, for `sub/tools/ungoverned.sh`: a second would be
    // `tools/outside.sh` read as the corpus's own `tools/outside.sh`.
    assert_eq!(ran.code, Some(1), "{}{}", ran.out, ran.err);
    assert_eq!(findings(&ran.out), 1, "{}", ran.out);
    assert!(ran.out.contains("other/x.sh"), "{}", ran.out);
    assert!(ran.out.contains("tools/outside.sh"), "{}", ran.out);
}

/// The passing direction of the gate under a subfolder root: the governed
/// tool, named from the git top level, raises nothing.
#[test]
fn a_subfolder_root_passes_a_governed_add() {
    let (root, out) = subfolder("scope-ratchet-subfolder-governed");
    let manifest = out.0.join("manifest");
    std::fs::write(
        &manifest,
        "headwater change 1\nadded\tsub/tools/governed.sh\n",
    )
    .expect("the manifest writes");
    let ran = strict_in_sub(&root, &manifest);
    assert_eq!(ran.code, Some(0), "{}{}", ran.out, ran.err);
    assert_eq!(findings(&ran.out), 0, "{}", ran.out);
    assert!(!ran.out.contains("sub/tools/governed.sh"), "{}", ran.out);
}
