// SPDX-License-Identifier: Apache-2.0
//! Named files of this repository route to the document that states their rule.
//!
//! [#1283](https://github.com/headwater-ai/headwater/issues/1283) measured that
//! most engine source has no `governs` edge, so an edit to a file warns nobody
//! whose text states the rule that file implements. Each row below is one edge
//! a governing document declares in its front matter, and the sentence that
//! warrants it is quoted in the pull request that added it. The case asks the
//! built binary what `headwater route edit <path>` names, over this repository
//! and not over a stub root, because the edges under test are this corpus's own.
//!
//! A row asserts the one governor it names, and not only that some document
//! governs the path. A subsystem spec that governs a whole crate by glob
//! ([HW-DR-0098](../../../../docs/decisions/0098-an-engine-subsystem-is-described-by-a-technical-design-spec-on-a-shelf-of-its-own-and-its-behavior-stays-where-it-is-already-written.md))
//! would otherwise satisfy a row whose own edge had been removed. The shape of
//! the root and of the binary call follows `subsystem_map.rs` and
//! `common/mod.rs`.

use std::path::{Path, PathBuf};
use std::process::Command;

/// `(path, governor)`: the governor declares `governs: <path>`.
const EDGES: &[(&str, &str)] = &[
    // Criterion 2: the two registers govern the fixtures that hold them.
    (
        "tools/repo/decision-register-fixtures.sh",
        "HW-REG-decisions",
    ),
    (
        "tools/repo/obligation-register-fixtures.sh",
        "HW-REG-open-obligations",
    ),
    // The engine files whose governor a clause of #1283 names.
    ("engine/crates/query/src/lib.rs", "HW-IFACE-headwater-mcp"),
    ("engine/crates/query/src/lib.rs", "HW-IFACE-headwater-show"),
    ("engine/crates/census/src/walk.rs", "HW-IFACE-headwater-mcp"),
    ("engine/crates/query/src/mcp.rs", "HW-IFACE-headwater-mcp"),
    ("engine/crates/import/src/harvest.rs", "HW-DR-0100"),
    (
        "engine/crates/check/src/harvest.rs",
        "HW-IFACE-headwater-check",
    ),
    ("engine/crates/check/src/pin.rs", "HW-IFACE-headwater-check"),
    (
        "engine/crates/check/src/claim.rs",
        "HW-IFACE-headwater-check",
    ),
    (
        "engine/crates/check/src/suspect.rs",
        "HW-SPEC-distribution-and-federation",
    ),
    (
        "engine/crates/check/src/link_identifier.rs",
        "HW-SPEC-check-layer",
    ),
    (
        "engine/crates/check/src/reciprocity.rs",
        "HW-SPEC-check-layer",
    ),
    ("engine/crates/check/src/target.rs", "HW-SPEC-check-layer"),
    (
        "engine/crates/check/src/self_target.rs",
        "HW-SPEC-taxonomy-model",
    ),
    ("engine/crates/resolve/src/order.rs", "HW-DR-0095"),
    ("engine/crates/resolve/src/confluence.rs", "HW-DR-0095"),
    ("engine/crates/meta/src/identifier.rs", "HW-DR-0025"),
    (
        "engine/crates/import/src/anchors.rs",
        "HW-IFACE-headwater-import",
    ),
    (
        "engine/crates/mark/src/lib.rs",
        "HW-IFACE-headwater-derived",
    ),
    (
        "engine/crates/probe/src/plan.rs",
        "HW-IFACE-headwater-probe",
    ),
    (
        "engine/crates/probe/src/budget.rs",
        "HW-IFACE-headwater-probe",
    ),
    (
        "engine/crates/probe/src/intake.rs",
        "HW-SPEC-the-recorder-contract",
    ),
    (
        "engine/crates/probe/src/grade.rs",
        "HW-SPEC-the-recorder-contract",
    ),
    (
        "engine/crates/generate/src/probe_result.rs",
        "HW-SPEC-the-recorder-contract",
    ),
    (
        "engine/crates/generate/src/lib.rs",
        "HW-IFACE-headwater-export",
    ),
    (
        "engine/crates/cli/tests/export.rs",
        "HW-IFACE-headwater-export",
    ),
    (
        "engine/crates/generate/src/site_nav.rs",
        "HW-IFACE-headwater-site",
    ),
    ("engine/crates/verbs/src/lib.rs", "HW-IFACE-headwater-help"),
    (
        "engine/crates/cli/tests/show.rs",
        "HW-IFACE-headwater-explain",
    ),
];

/// The repository root, from this crate's manifest directory.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// The text of one top-level member of the route document, from its key up to
/// the key that follows it.
fn member<'a>(json: &'a str, key: &str, next: &str) -> &'a str {
    let start = json
        .find(&format!("\"{key}\""))
        .unwrap_or_else(|| panic!("no `{key}` in {json}"));
    let rest = &json[start..];
    let end = rest
        .find(&format!("\"{next}\""))
        .unwrap_or_else(|| panic!("no `{next}` after `{key}` in {json}"));
    &rest[..end]
}

/// What is wrong with the route for one row, or nothing.
fn fault(path: &str, governor: &str) -> Option<String> {
    let root = root();
    let ran = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["route", "edit", path, "--json", "--root"])
        .arg(&root)
        .output()
        .expect("the binary runs");
    let out = String::from_utf8_lossy(&ran.stdout);
    if ran.status.code() != Some(0) {
        return Some(format!(
            "{path}: route exited {:?}: {}",
            ran.status.code(),
            String::from_utf8_lossy(&ran.stderr)
        ));
    }
    let ungoverned = member(&out, "ungoverned", "text");
    if !ungoverned.contains("\"ungoverned\": []") {
        return Some(format!("{path}: nothing governs it: {ungoverned}"));
    }
    let pointers = member(&out, "pointers", "withheld");
    let id = format!("\"id\": \"{governor}\"");
    let by_anchor = pointers
        .split("\"path\":")
        .any(|pointer| pointer.contains(&id) && pointer.contains("\"by\": \"anchor\""));
    if by_anchor {
        None
    } else {
        Some(format!(
            "{path}: {governor} is not a pointer reached by its anchor: {pointers}"
        ))
    }
}

#[test]
fn each_named_file_routes_to_the_document_that_states_its_rule() {
    let faults: Vec<String> = EDGES
        .iter()
        .filter_map(|(path, governor)| fault(path, governor))
        .collect();
    assert!(
        faults.is_empty(),
        "{} of {} rows fail:\n{}",
        faults.len(),
        EDGES.len(),
        faults.join("\n")
    );
}

#[test]
fn every_row_names_a_file_on_disk() {
    let root = root();
    for (path, _) in EDGES {
        assert!(root.join(path).is_file(), "{path} is not a file");
    }
}
