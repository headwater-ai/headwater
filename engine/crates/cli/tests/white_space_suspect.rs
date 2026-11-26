// SPDX-License-Identifier: Apache-2.0
//! A white-space-only change to a governed file does not fail a blocking gate
//! where the control spares `white_space`, and a change of content still does
//! (#2100). The touched files and their count against the matched set are in
//! the report (#2028).
//!
//! Every arm starts from one tree: the stub root, CT-REL-6 promoted to
//! `blocking` by an overlay `override`, and one decision whose `governs` edge
//! is stamped with a real digest by `check --fix --baseline`. Each arm then
//! edits the governed file and hands `check --strict --change` a manifest
//! written by hand, whose `prior` line names a file of the bytes before the
//! edit.
//!
//! Arm 3 is the decisive one. A content edit lands first and is never
//! re-stamped, and then a white-space edit lands whose prior bytes are the
//! bytes after the content edit. The change alone is white space, and the
//! edge drifted unread before it, so the gate must still close: the prior
//! digest rebuilt from the manifest does not equal the stamp.

mod common;
use common::Root;

/// The decision every arm stamps, and the path a finding stands at.
const DOCUMENT: &str = "docs/decisions/0002-the-decision-that-governs-a-tool.md";

/// The override that promotes CT-REL-6, as an adopter writes it.
const PROMOTE: &str = "  controls.CT-REL-6.posture: blocking\n";

/// The `add` that makes CT-REL-6 spare a white-space-only change.
const SPARE: &str = "  controls.CT-REL-6.spares: [white_space]\n";

/// The stub root with one decision that governs `target`, the files `tools`
/// under `tools/`, CT-REL-6 promoted, and `spare` added to the overlay's
/// `add` block. The edge is stamped before the root is handed back.
fn root(
    label: &str,
    target: &'static str,
    tools: &'static [(&str, &str)],
    spare: &'static str,
) -> Root {
    let root = Root::shaped(label, move |at| {
        std::fs::write(
            at.join(DOCUMENT),
            format!(
                "---\nid: HW-DR-0002\ntitle: The decision that governs a tool\nstatus: \
                 current\nstatus_since: 2026-08-01\nlast_verified: 2026-08-01\nsummary: One \
                 decision that governs a tool.\nprovenance:\n  warrant: asserted\n  agency: \
                 human\n  evidence_basis: unevidenced\nrelations:\n  governs:\n    - to: \
                 {target}\n---\n\n# The decision that governs a tool\n\n## Context\n\nA \
                 fixture.\n\n## Decision\n\nIt governs a tool.\n\n## Consequences\n\nThe \
                 check reads it.\n"
            ),
        )
        .expect("the decision writes");
        // The claim file, so that `identifier.claim.missing`, an error, does
        // not stand in for the gate this file reads.
        let claims = at.join(".headwater/ids/decision_id");
        std::fs::create_dir_all(&claims).expect("the claim directory is made");
        std::fs::write(claims.join("HW-DR-0002"), format!("{DOCUMENT}\n"))
            .expect("the claim writes");
        std::fs::create_dir_all(at.join("tools")).expect("the tools directory is made");
        for (name, bytes) in tools {
            std::fs::write(at.join("tools").join(name), bytes).expect("the tool writes");
        }
        let file = at.join(".headwater/overlay.yml");
        let source = std::fs::read_to_string(&file).expect("the overlay reads");
        let source = source.replacen("\nadd:\n", &format!("\nadd:\n{spare}"), 1);
        std::fs::write(&file, with_override(&source, PROMOTE)).expect("the overlay writes");
    });
    let stamped = root.run(&["check", "--fix", "--baseline"]);
    assert_eq!(stamped.code, Some(0), "{stamped:?}");
    let decision = std::fs::read_to_string(root.at.join(DOCUMENT)).expect("the decision reads");
    assert!(
        decision.contains("verified_revision: sha256:"),
        "{decision}"
    );
    root
}

/// Write `before` as the prior version of `tools/<name>` and `after` as the
/// current one, and return the manifest that names the change.
fn change(root: &Root, name: &str, before: &str, after: &str) -> String {
    let prior = root.at.join("prior");
    std::fs::create_dir_all(&prior).expect("the prior directory is made");
    let source = prior.join(name);
    std::fs::write(&source, before).expect("the prior version writes");
    std::fs::write(root.at.join("tools").join(name), after).expect("the tool writes");
    let manifest = root.at.join("manifest");
    std::fs::write(
        &manifest,
        format!(
            "headwater change 1\nprior\ttools/{name}\t{}\n",
            source.to_string_lossy()
        ),
    )
    .expect("the manifest writes");
    manifest.to_string_lossy().into_owned()
}

/// The `touched by the change` block of a text report, and empty where the
/// report has none.
fn touched(out: &str) -> String {
    out.split("\ntouched by the change\n")
        .nth(1)
        .and_then(|rest| rest.split("\n\n").next())
        .unwrap_or_default()
        .to_string()
}

/// The `touched` object of the first finding of a JSON report, as the
/// `json field` verb reads each member.
fn json_touched(root: &Root, manifest: &str, path: &[&str]) -> String {
    let report = root.run(&["check", "--change", manifest, "--format", "json"]);
    let suspect = (0..)
        .map(|index| index.to_string())
        .take_while(|index| {
            root.run_with(&["json", "field", "findings", index, "rule"], &report.out)
                .code
                == Some(0)
        })
        .find(|index| {
            root.run_with(&["json", "field", "findings", index, "rule"], &report.out)
                .out
                .trim()
                == "relation.target.suspect"
        })
        .unwrap_or_else(|| panic!("no suspect finding in\n{}", report.out));
    let mut args = vec!["json", "field", "findings", suspect.as_str()];
    args.extend_from_slice(path);
    let read = root.run_with(&args, &report.out);
    assert_eq!(read.code, Some(0), "{read:?}\n{}", report.out);
    read.out.trim().to_string()
}

/// The live `relation.target.suspect` records of a text report.
fn suspect_count(out: &str) -> usize {
    out.matches("warn\n    relation.target.suspect (").count()
}

/// Arm 1: trailing spaces and CRLF and nothing else, with the control
/// sparing `white_space`. The run passes, the finding is still reported, and
/// both formats classify the file `white_space` and name the control.
#[test]
fn a_white_space_change_passes_a_blocking_control_that_spares_it() {
    let root = root(
        "ws-spared",
        "tools/a.sh",
        &[("a.sh", "echo a\nexit 0\n")],
        SPARE,
    );
    let manifest = change(
        &root,
        "a.sh",
        "echo a\nexit 0\n",
        "echo a  \r\n\r\nexit 0\t\r\n",
    );
    let checked = root.run(&["check", "--strict", "--change", &manifest]);
    assert_eq!(checked.code, Some(0), "{}{}", checked.out, checked.err);
    assert_eq!(suspect_count(&checked.out), 1, "{}", checked.out);
    assert!(
        !checked.out.contains("blocked by CT-REL-6"),
        "{}",
        checked.out
    );
    let block = touched(&checked.out);
    assert!(
        block.contains(DOCUMENT)
            && block.contains("relation.target.suspect touched 1 of 1 matched")
            && block.contains("tools/a.sh white_space")
            && block.contains("spared by CT-REL-6"),
        "{}",
        checked.out
    );
    assert_eq!(
        json_touched(&root, &manifest, &["touched", "files", "0", "class"]),
        "white_space"
    );
    assert_eq!(
        json_touched(&root, &manifest, &["touched", "white_space"]),
        "true"
    );
    assert_eq!(json_touched(&root, &manifest, &["spared_by"]), "CT-REL-6");
}

/// Arm 2: one token replaced. The run fails, the class is `content`, and the
/// finding is blocked by CT-REL-6.
#[test]
fn a_content_change_still_fails_a_blocking_control_that_spares_white_space() {
    let root = root("ws-content", "tools/a.sh", &[("a.sh", "echo a\n")], SPARE);
    let manifest = change(&root, "a.sh", "echo a\n", "echo b\n");
    let checked = root.run(&["check", "--strict", "--change", &manifest]);
    assert_eq!(checked.code, Some(1), "{}{}", checked.out, checked.err);
    assert!(
        checked.out.contains("blocked by CT-REL-6"),
        "{}",
        checked.out
    );
    let block = touched(&checked.out);
    assert!(
        block.contains("tools/a.sh content") && !block.contains("spared by"),
        "{}",
        checked.out
    );
    assert_eq!(
        json_touched(&root, &manifest, &["touched", "files", "0", "class"]),
        "content"
    );
    assert_eq!(json_touched(&root, &manifest, &["blocked_by"]), "CT-REL-6");
}

/// Arm 3, the decisive arm: a content edit lands unread, and then a
/// white-space edit whose prior bytes are the edited ones. The class is
/// `white_space`, and the run still fails, because the prior digest the
/// manifest rebuilds is not the stamp.
#[test]
fn a_white_space_change_after_an_unread_content_change_still_fails() {
    let root = root("ws-drift", "tools/a.sh", &[("a.sh", "echo a\n")], SPARE);
    // The earlier change, which nobody re-read and nothing re-stamped.
    std::fs::write(root.at.join("tools/a.sh"), "echo b\n").expect("the tool writes");
    let manifest = change(&root, "a.sh", "echo b\n", "echo b  \r\n");
    let checked = root.run(&["check", "--strict", "--change", &manifest]);
    assert_eq!(
        checked.code,
        Some(1),
        "an edge that drifted before this change still blocks\n{}{}",
        checked.out,
        checked.err
    );
    assert!(
        checked.out.contains("blocked by CT-REL-6"),
        "{}",
        checked.out
    );
    let block = touched(&checked.out);
    assert!(
        block.contains("tools/a.sh white_space") && !block.contains("spared by"),
        "{}",
        checked.out
    );
    assert_eq!(
        json_touched(&root, &manifest, &["touched", "white_space"]),
        "false"
    );
}

/// Arm 4: arm 1 with no `spares` declared. The default is unchanged, so the
/// white-space change fails the blocking gate.
#[test]
fn a_white_space_change_fails_a_blocking_control_that_spares_nothing() {
    let root = root("ws-default", "tools/a.sh", &[("a.sh", "echo a\n")], "");
    let manifest = change(&root, "a.sh", "echo a\n", "echo a  \r\n");
    let checked = root.run(&["check", "--strict", "--change", &manifest]);
    assert_eq!(checked.code, Some(1), "{}{}", checked.out, checked.err);
    assert!(
        checked.out.contains("blocked by CT-REL-6"),
        "{}",
        checked.out
    );
    assert!(
        touched(&checked.out).contains("tools/a.sh white_space"),
        "{}",
        checked.out
    );
}

/// Arm 5: a glob edge over two files, one touched white-space-only. The run
/// passes, and the report states 1 touched file of 2 matched (#2028).
#[test]
fn a_glob_edge_reports_its_touched_files_against_the_matched_set() {
    let root = root(
        "ws-glob",
        "tools/*.sh",
        &[("a.sh", "echo a\n"), ("b.sh", "echo b\n")],
        SPARE,
    );
    let manifest = change(&root, "a.sh", "echo a\n", "echo a \n");
    let checked = root.run(&["check", "--strict", "--change", &manifest]);
    assert_eq!(checked.code, Some(0), "{}{}", checked.out, checked.err);
    let block = touched(&checked.out);
    assert!(
        block.contains("relation.target.suspect touched 1 of 2 matched")
            && block.contains("tools/a.sh white_space")
            && !block.contains("tools/b.sh")
            && block.contains("spared by CT-REL-6"),
        "{}",
        checked.out
    );
    assert_eq!(json_touched(&root, &manifest, &["touched", "matched"]), "2");
    assert_eq!(
        json_touched(&root, &manifest, &["touched", "files", "0", "path"]),
        "tools/a.sh"
    );
}

/// The overlay with `entries`, each a line indented two spaces, in its
/// `override` block, as `blocking_posture.rs` appends them.
fn with_override(overlay: &str, entries: &str) -> String {
    match overlay.lines().position(|line| line == "override:") {
        Some(_) => format!("{}\n{entries}", overlay.trim_end()),
        None => format!("{overlay}\noverride:\n{entries}"),
    }
}
