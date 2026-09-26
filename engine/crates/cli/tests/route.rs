// SPDX-License-Identifier: Apache-2.0
//! `headwater route` names a path that the governed scope admits and that no
//! document governs (#953, `docs/interfaces/headwater-route.md`).
//!
//! The write-time hook asks this verb one question per edit. A path in scope
//! with no governing edge is the case the hook used to meet in silence, and a
//! path outside the scope, or one a document governs, must stay silent.

mod common;
use common::Root;

/// The stub root, with one decision that governs `tools/governed.sh`.
fn root(label: &str) -> Root {
    Root::shaped(label, |at| {
        std::fs::write(at.join("tools/governed.sh"), "").expect("the governed file writes");
        std::fs::write(
            at.join("docs/decisions/0002-the-decision-that-governs-a-tool.md"),
            "---\nid: HW-DR-0002\ntitle: The decision that governs a tool\nstatus: \
             current\nstatus_since: 2026-08-01\nlast_verified: 2026-08-01\nsummary: One \
             decision that governs one file under the tools directory.\nprovenance:\n  \
             warrant: asserted\n  agency: human\n  evidence_basis: unevidenced\nrelations:\n  \
             governs:\n    - tools/governed.sh\n---\n\n# The decision that governs a \
             tool\n\n## Context\n\nA fixture.\n\n## Decision\n\nIt governs one \
             file.\n\n## Consequences\n\nThe route names it.\n",
        )
        .expect("the decision writes");
    })
}

fn ungoverned(json: &str) -> &str {
    let start = json.find("\"ungoverned\"").unwrap_or_else(|| panic!("no member in {json}"));
    let rest = &json[start..];
    let end = rest.find(']').expect("the array closes");
    &rest[..=end]
}

#[test]
fn a_path_in_scope_that_nothing_governs_is_named_with_the_lines_that_declare_it() {
    let root = root("route-ungoverned");
    let ran = root.run(&["route", "edit", "tools/unrelated.sh", "--json"]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    let member = ungoverned(&ran.out);
    assert!(member.contains("\"path\": \"tools/unrelated.sh\""), "{member}");
    assert!(member.contains("\"governs\""), "{member}");

    let text = root.run(&["route", "edit", "tools/unrelated.sh"]);
    assert_eq!(text.code, Some(0), "{text:?}");
    assert!(
        text.out
            .contains("tools/unrelated.sh is in the governed scope, and nothing governs it"),
        "{}",
        text.out
    );
    assert!(text.out.contains("      governs:\n        - tools/unrelated.sh\n"), "{}", text.out);
    assert!(!text.out.contains(" — "), "the hook selects pointers by an em dash: {}", text.out);
}

#[test]
fn a_governed_path_and_a_path_outside_the_scope_name_nothing() {
    let root = root("route-governed");
    for path in ["tools/governed.sh", "engine/crates/stub/tests/unrelated.rs", "fix the tools"] {
        let ran = root.run(&["route", "edit", path, "--json"]);
        assert_eq!(ran.code, Some(0), "{ran:?}");
        let member = ungoverned(&ran.out);
        assert!(!member.contains("\"path\""), "{path}: {member}");
        let text = root.run(&["route", "edit", path]);
        assert!(!text.out.contains("governed scope"), "{path}: {}", text.out);
    }
}
