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
