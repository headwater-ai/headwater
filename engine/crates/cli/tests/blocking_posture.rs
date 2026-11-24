// SPDX-License-Identifier: Apache-2.0
//! `headwater check --strict` fails on a live finding that a `posture:
//! blocking` control discharges, whatever the finding's severity (#2015).
//!
//! Spec 12 separates the two words: severity is the check's, and posture is
//! the control's. The owner ruled on 2026-11-22 that posture is the adopter's
//! to configure, with an overlay `override`, and that a promotion by
//! configuration alone moves the gate. Before #2015 the gate read severity
//! alone, so an adopter who promoted a control to blocking saw nothing change.
//!
//! Every arm starts from one tree: the stub root and one decision whose
//! `governs` edge records a revision the target no longer has, which raises
//! exactly one `relation.target.suspect` at `warn`. CT-REL-6 discharges that
//! rule, and the standard package declares it `posture: advisory`.

mod common;
use common::Root;

/// One decision that governs `target` at a recorded revision the target does
/// not have, so its edge is suspect.
fn decision(id: &str, file: &str, target: &str) -> (String, String) {
    let text = format!(
        "---\nid: {id}\ntitle: The decision whose edge went stale\nstatus: \
         current\nstatus_since: 2026-08-01\nlast_verified: 2026-08-01\nsummary: One \
         decision that governs a tool at a recorded revision.\nprovenance:\n  \
         warrant: asserted\n  agency: human\n  evidence_basis: unevidenced\nrelations:\n  \
         governs:\n    - to: {target}\n      verified_revision: sha256:0000\n---\n\n# The \
         decision whose edge went stale\n\n## Context\n\nA fixture.\n\n## Decision\n\nIt \
         governs one file.\n\n## Consequences\n\nThe check reads it.\n"
    );
    (format!("docs/decisions/{file}"), text)
}

/// Write the claim file of every decision under `docs/decisions/`, so that
/// `identifier.claim.missing`, an error, does not stand in for the gate this
/// file reads.
fn claim(at: &std::path::Path) {
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

/// The override that promotes CT-REL-6, as an adopter writes it.
const PROMOTE: &str = "  controls.CT-REL-6.posture: blocking\n";

/// The stub root with one stale `governs` edge, and `overlay` appended to the
/// copied overlay before the root resolves.
fn root(label: &str, overlay: &'static str) -> Root {
    Root::shaped(label, move |at| {
        std::fs::write(at.join("tools/a.sh"), "echo a\n").expect("the target writes");
        let (path, text) = decision(
            "HW-DR-0002",
            "0002-the-decision-whose-edge-went-stale.md",
            "tools/a.sh",
        );
        std::fs::write(at.join(path), text).expect("the decision writes");
        claim(at);
        let file = at.join(".headwater/overlay.yml");
        let source = std::fs::read_to_string(&file).expect("the overlay reads");
        std::fs::write(&file, with_override(&source, overlay)).expect("the overlay writes");
    })
}

/// The live `relation.target.suspect` records of a text report, each the
/// three lines `Finding::render` writes.
fn suspect_count(out: &str) -> usize {
    out.matches("warn\n    relation.target.suspect (").count()
}

/// Arm A, the control: the advisory base passes a strict run, and the report
/// still holds the warning and names no blocking control.
#[test]
fn an_advisory_control_leaves_a_warning_out_of_the_strict_gate() {
    let root = root("posture-advisory", "");
    let checked = root.run(&["check", "--strict"]);
    assert_eq!(checked.code, Some(0), "{checked:?}");
    assert_eq!(suspect_count(&checked.out), 1, "{}", checked.out);
    assert!(
        !checked.out.contains("blocks a strict run"),
        "an advisory control blocks nothing, so the block is left out:\n{}",
        checked.out
    );
}

/// Arm B, the decisive arm: the identical tree with CT-REL-6 promoted by an
/// overlay `override` fails a strict run on the same warning, and both formats
/// name the control that blocked it.
#[test]
fn a_blocking_control_fails_a_strict_run_on_a_warning() {
    let root = root("posture-blocking", PROMOTE);
    let checked = root.run(&["check", "--strict"]);
    assert_eq!(
        checked.code,
        Some(1),
        "a warning that a blocking control discharges fails --strict\n{}{}",
        checked.out,
        checked.err
    );
    assert_eq!(suspect_count(&checked.out), 1, "{}", checked.out);
    let block = checked
        .out
        .split("\nblocks a strict run\n")
        .nth(1)
        .unwrap_or_else(|| panic!("no `blocks a strict run` block in\n{}", checked.out));
    let block = block.split("\n\n").next().unwrap_or_default();
    assert!(
        block.contains("relation.target.suspect")
            && block.contains("docs/decisions/0002-the-decision-whose-edge-went-stale.md")
            && block.contains("blocked by CT-REL-6"),
        "the block names the rule, the path and the control:\n{block}"
    );

    // The report does not depend on the flag: a run without --strict exits 0
    // and says the same thing.
    let plain = root.run(&["check"]);
    assert_eq!(plain.code, Some(0), "{plain:?}");
    assert!(plain.out.contains("blocked by CT-REL-6"), "{}", plain.out);

    let json = root.run(&["check", "--format", "json"]);
    assert_eq!(json.code, Some(0), "{json:?}");
    assert_eq!(
        json.out.matches("\"blocked_by\": \"CT-REL-6\"").count(),
        1,
        "one live finding carries the control that blocks it:\n{}",
        json.out
    );
}

/// Arm F: severity does not decide it. A `governs` edge whose target is a
/// directory has no digest, so `relation.target.suspect` reports it at `info`.
/// Under the advisory base the strict run passes, and with CT-REL-6 promoted
/// the same `info` finding fails it.
#[test]
fn a_blocking_control_fails_a_strict_run_on_an_info_finding() {
    let directory = |label: &str, overlay: &'static str| {
        Root::shaped(label, move |at| {
            std::fs::create_dir_all(at.join("tools/dir")).expect("the directory is made");
            std::fs::write(at.join("tools/dir/a.sh"), "echo a\n").expect("the file writes");
            let (path, text) = decision(
                "HW-DR-0002",
                "0002-the-decision-whose-edge-went-stale.md",
                "tools/dir",
            );
            std::fs::write(at.join(path), text).expect("the decision writes");
            claim(at);
            let file = at.join(".headwater/overlay.yml");
            let source = std::fs::read_to_string(&file).expect("the overlay reads");
            std::fs::write(&file, with_override(&source, overlay)).expect("the overlay writes");
        })
    };
    let info = "info\n    relation.target.suspect (";

    let advisory = directory("posture-info-advisory", "");
    let passed = advisory.run(&["check", "--strict"]);
    assert_eq!(passed.code, Some(0), "{passed:?}");
    assert_eq!(passed.out.matches(info).count(), 1, "{}", passed.out);

    let blocking = directory("posture-info-blocking", PROMOTE);
    let failed = blocking.run(&["check", "--strict"]);
    assert_eq!(
        failed.code,
        Some(1),
        "an info finding that a blocking control discharges fails --strict\n{}{}",
        failed.out,
        failed.err
    );
    assert_eq!(failed.out.matches(info).count(), 1, "{}", failed.out);
    assert!(
        failed
            .out
            .contains("relation.target.suspect blocked by CT-REL-6"),
        "{}",
        failed.out
    );
}

/// Arm C, the ratchet: the adoption block holds a finding that a blocking
/// control discharges, and a new finding of the same rule still fails.
#[test]
fn the_adoption_block_holds_a_blocking_finding_and_a_new_one_fails() {
    let root = root("posture-ratchet", PROMOTE);
    let inferred = root.run(&["infer", "--owner", "t", "--write"]);
    assert_eq!(inferred.code, Some(0), "{inferred:?}");

    let held = root.run(&["check", "--strict"]);
    assert_eq!(
        held.code,
        Some(0),
        "a parked finding does not block, whatever the posture\n{}{}",
        held.out,
        held.err
    );
    assert!(!held.out.contains("blocks a strict run"), "{}", held.out);

    std::fs::write(root.at.join("tools/b.sh"), "echo b\n").expect("the target writes");
    let (path, text) = decision(
        "HW-DR-0003",
        "0003-the-second-decision-whose-edge-went-stale.md",
        "tools/b.sh",
    );
    std::fs::write(root.at.join(path), text).expect("the decision writes");
    claim(&root.at);

    let fresh = root.run(&["check", "--strict"]);
    assert_eq!(
        fresh.code,
        Some(1),
        "the new finding is not parked, so it blocks\n{}{}",
        fresh.out,
        fresh.err
    );
    let block = fresh
        .out
        .split("\nblocks a strict run\n")
        .nth(1)
        .unwrap_or_else(|| panic!("no `blocks a strict run` block in\n{}", fresh.out));
    let block = block.split("\n\n").next().unwrap_or_default();
    assert!(block.contains("0003-the-second-decision"), "{block}");
    assert!(
        !block.contains("0002-the-decision"),
        "the parked finding is not among what blocks:\n{block}"
    );

    // The JSON report carries both findings, and only the live one names the
    // control: a parked finding blocks nothing.
    let json = root.run(&["check", "--format", "json"]);
    assert_eq!(json.code, Some(0), "{json:?}");
    assert!(
        json.out
            .contains("0002-the-decision-whose-edge-went-stale.md"),
        "the parked finding is in the report: {}",
        json.out
    );
    assert_eq!(
        json.out.matches("\"blocked_by\": \"CT-REL-6\"").count(),
        1,
        "the parked finding carries no `blocked_by`:\n{}",
        json.out
    );
}

/// Arm E, the cache: a promotion after a warm run moves the exit status, and
/// the warm report and a `--no-cache` report are the same bytes. Posture is
/// read from the register after the checks run, so no cached finding holds it
/// and no cache key has to move for the gate to see a promotion.
#[test]
fn a_promotion_after_a_warm_run_moves_the_gate_and_the_cache_agrees() {
    let root = root("posture-cache", "");
    let cold = root.run(&["check", "--strict"]);
    assert_eq!(cold.code, Some(0), "{cold:?}");

    let file = root.at.join(".headwater/overlay.yml");
    let source = std::fs::read_to_string(&file).expect("the overlay reads");
    std::fs::write(&file, with_override(&source, PROMOTE)).expect("the overlay writes");
    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_eq!(resolved.code, Some(0), "{resolved:?}");

    let warm = root.run(&["check", "--strict"]);
    let again = root.run(&["check", "--strict"]);
    let uncached = root.run(&["check", "--strict", "--no-cache"]);
    for ran in [&warm, &again, &uncached] {
        assert_eq!(ran.code, Some(1), "{ran:?}");
        assert!(ran.out.contains("blocked by CT-REL-6"), "{}", ran.out);
    }
    assert_eq!(
        warm.out, uncached.out,
        "the cache moves no byte of the report"
    );
    assert_eq!(
        again.out, uncached.out,
        "the cache moves no byte of the report"
    );
    assert!(
        !again.err.starts_with("0 served from cache"),
        "the second run after the promotion is served from the cache: {}",
        again.err
    );
}

/// Arm D: a posture outside the two admitted values is refused and names the
/// control, so a typo cannot leave the gate open in silence. The resolver
/// refuses it against the base's value set before any lock is written, and
/// `register::read_control` refuses it again for a lock that reached the
/// check another way.
#[test]
fn a_posture_outside_the_two_values_is_refused() {
    let root = root("posture-typo", "");
    let file = root.at.join(".headwater/overlay.yml");
    let source = std::fs::read_to_string(&file).expect("the overlay reads");
    std::fs::write(
        &file,
        with_override(&source, "  controls.CT-REL-6.posture: blockng\n"),
    )
    .expect("the overlay writes");
    let checked = root.run(&["taxonomy", "resolve"]);
    assert_ne!(checked.code, Some(0), "{checked:?}");
    let said = format!("{}{}", checked.out, checked.err);
    assert!(
        said.contains("CT-REL-6") && said.contains("blockng") && said.contains("`blocking`"),
        "the refusal names the control, the value and the admitted values:\n{said}"
    );
}

/// The overlay with `entries`, each a line indented two spaces, in its
/// `override` block. The repository's overlay closes on that block since
/// #1573, so the entries join it, and an overlay with none gains one. Two
/// `override` keys would be a duplicate key the loader refuses.
fn with_override(overlay: &str, entries: &str) -> String {
    match overlay.lines().position(|line| line == "override:") {
        Some(at) => {
            assert!(
                overlay
                    .lines()
                    .skip(at + 1)
                    .all(|line| line.is_empty() || line.starts_with(' ') || line.starts_with('#')),
                "the overlay's `override` block is not its last block, so this case cannot \
                 append to it"
            );
            format!("{}\n{entries}", overlay.trim_end())
        }
        None => format!("{overlay}\noverride:\n{entries}"),
    }
}
