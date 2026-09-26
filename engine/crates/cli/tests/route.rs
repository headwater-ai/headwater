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
    let start = json
        .find("\"ungoverned\"")
        .unwrap_or_else(|| panic!("no member in {json}"));
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
    assert!(
        member.contains("\"path\": \"tools/unrelated.sh\""),
        "{member}"
    );
    assert!(member.contains("\"governs\""), "{member}");

    let text = root.run(&["route", "edit", "tools/unrelated.sh"]);
    assert_eq!(text.code, Some(0), "{text:?}");
    assert!(
        text.out
            .contains("tools/unrelated.sh is in the governed scope, and nothing governs it"),
        "{}",
        text.out
    );
    assert!(
        text.out
            .contains("      governs:\n        - tools/unrelated.sh\n"),
        "{}",
        text.out
    );
    assert!(
        !text.out.contains(" — "),
        "the hook selects pointers by an em dash: {}",
        text.out
    );
}

#[test]
fn a_governed_path_and_a_path_outside_the_scope_name_nothing() {
    let root = root("route-governed");
    for path in [
        "tools/governed.sh",
        "engine/crates/stub/tests/unrelated.rs",
        "fix the tools",
    ] {
        let ran = root.run(&["route", "edit", path, "--json"]);
        assert_eq!(ran.code, Some(0), "{ran:?}");
        let member = ungoverned(&ran.out);
        assert!(!member.contains("\"path\""), "{path}: {member}");
        let text = root.run(&["route", "edit", path]);
        assert!(!text.out.contains("governed scope"), "{path}: {}", text.out);
    }
}

/// A word that ends in prose punctuation, or in a line number, names the path
/// without it. A governed file named that way is named as governed, and never
/// as a path that nothing governs with a malformed edge proposed for it.
#[test]
fn a_path_named_with_prose_punctuation_or_a_line_number_is_the_path_itself() {
    let root = root("route-punctuation");
    for task in [
        "Look at tools/governed.sh.",
        "tools/governed.sh:12",
        "(tools/governed.sh),",
        "tools/governed.sh's",
        "tools/governed.sh's.",
        "tools/governed.sh:12:3.",
    ] {
        let ran = root.run(&["route", task, "--json"]);
        assert_eq!(ran.code, Some(0), "{ran:?}");
        let member = ungoverned(&ran.out);
        assert!(!member.contains("\"path\""), "{task}: {member}");
        assert!(
            ran.out
                .contains("\"anchors\": [\n    \"tools/governed.sh\"\n  ]"),
            "{task}: {}",
            ran.out
        );
    }
    // An ungoverned file on the tree is named the same way.
    std::fs::write(root.at.join("tools/other.sh"), "").expect("the file writes");
    let ran = root.run(&["route", "edit", "tools/other.sh:3.", "--json"]);
    let member = ungoverned(&ran.out);
    assert!(member.contains("\"path\": \"tools/other.sh\""), "{member}");

    // Where something would have to come off and no shorter reading is on the
    // tree or reached by an edge, the word names nothing rather than a guess.
    let ran = root.run(&["route", "edit", "tools/unrelated.sh:3.", "--json"]);
    let member = ungoverned(&ran.out);
    assert!(!member.contains("\"path\""), "{member}");
}

/// A path that exists as written is read as written, never as a shorter path
/// that punctuation stripping would make of it. A directory and a URL name no
/// ungoverned file.
#[test]
fn a_path_on_the_tree_as_written_is_never_read_as_a_shorter_one() {
    let root = root("route-as-written");
    for (file, named) in [
        ("tools/a:b.sh", "tools/a:b.sh"),
        ("tools/zz.", "tools/zz."),
        ("tools/it's", "tools/it's"),
    ] {
        std::fs::write(root.at.join(file), "").expect("the file writes");
        let ran = root.run(&["route", "edit", file, "--json"]);
        assert_eq!(ran.code, Some(0), "{ran:?}");
        let member = ungoverned(&ran.out);
        assert!(
            member.contains(&format!("\"path\": \"{named}\"")),
            "{file}: {member}"
        );
        assert_eq!(member.matches("\"path\"").count(), 1, "{file}: {member}");
    }
    for task in [
        "tools/.",
        "tools/",
        "https://github.com/headwater-ai/headwater/blob/main/tools/other.sh",
    ] {
        let ran = root.run(&["route", "edit", task, "--json"]);
        assert_eq!(ran.code, Some(0), "{ran:?}");
        let member = ungoverned(&ran.out);
        assert!(!member.contains("\"path\""), "{task}: {member}");
    }
}

/// The #951 owner ruling, "nobody governs a cache": a path git ignores is not
/// named, though a scope pattern admits it. The same root before `git init`
/// names it, so the case cannot pass on a path the scope never admitted.
#[test]
fn a_path_git_ignores_is_not_named_as_ungoverned() {
    let root = root("route-ignored");
    let path = "tools/__pycache__/stub.cpython-312.pyc";
    let cache = root.at.join(path);
    std::fs::create_dir_all(cache.parent().expect("a parent")).expect("the cache is made");
    std::fs::write(&cache, "").expect("the cache writes");
    std::fs::write(root.at.join(".gitignore"), "__pycache__/\n").expect("the ignore file writes");

    let before = root.run(&["route", "edit", path, "--json"]);
    assert!(
        ungoverned(&before.out).contains(&format!("\"path\": \"{path}\"")),
        "{}",
        before.out
    );

    let init = std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&root.at)
        .status()
        .expect("git runs");
    assert!(init.success());

    let after = root.run(&["route", "edit", path, "--json"]);
    assert_eq!(after.code, Some(0), "{after:?}");
    assert!(
        !ungoverned(&after.out).contains("\"path\""),
        "{}",
        after.out
    );
    let text = root.run(&["route", "edit", path]);
    assert!(!text.out.contains("governed scope"), "{}", text.out);
}

/// The relations are read off the taxonomy. A bundle that declares a second
/// governance relation onto `code_path` puts it beside `governs`, with no
/// change to the engine, so a route that named `governs` alone fails here.
#[test]
fn every_declared_relation_that_governs_the_anchor_kind_is_proposed() {
    let root = Root::shaped("route-relations", |at| {
        common::write_bundle(
            at,
            "zz-rules",
            "add:\n  relations.zz_rules:\n    family: governance\n    from: \
             [governed_document]\n    to: [code_path]\n    cardinality: many\n    created_by: \
             hook\n",
        );
        let declaration = at.join(".headwater/taxonomy.yml");
        let text = std::fs::read_to_string(&declaration).expect("the declaration reads");
        let from = "  bundles: [";
        assert!(text.contains(from), "the declaration lists its bundles");
        std::fs::write(
            &declaration,
            text.replacen(from, "  bundles: [zz-rules, ", 1),
        )
        .expect("the declaration writes");
    });
    let ran = root.run(&["route", "edit", "tools/unrelated.sh", "--json"]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    let member = ungoverned(&ran.out);
    assert!(member.contains("\"governs\""), "{member}");
    assert!(member.contains("\"zz_rules\""), "{member}");
    let text = root.run(&["route", "edit", "tools/unrelated.sh"]);
    assert!(
        text.out
            .contains("      zz_rules:\n        - tools/unrelated.sh\n"),
        "{}",
        text.out
    );
    // Exactly these two. The taxonomy declares other relations whose target
    // admits `code_path`, `traces_to` among them, and none of them governs, so
    // a route that proposed one would propose an edge nobody reads as
    // governance.
    assert_eq!(
        text.out.matches("        - tools/unrelated.sh\n").count(),
        2,
        "{}",
        text.out
    );
}

/// The stub root, with one decision whose `governs` edge onto
/// `tools/stale.sh` records a revision the file no longer has, and one whose
/// edge onto `tools/governed.sh` records none.
fn stale_root(label: &str) -> Root {
    Root::shaped(label, |at| {
        std::fs::write(at.join("tools/stale.sh"), "echo moved\n").expect("the stale file writes");
        std::fs::write(at.join("tools/governed.sh"), "").expect("the governed file writes");
        let decision = |id: &str, file: &str, entry: &str| {
            std::fs::write(
                at.join(format!("docs/decisions/{file}")),
                format!(
                    "---\nid: {id}\ntitle: A decision that governs a tool\nstatus: \
                     current\nstatus_since: 2026-08-01\nlast_verified: 2026-08-01\nsummary: One \
                     decision that governs one file under the tools directory.\nprovenance:\n  \
                     warrant: asserted\n  agency: human\n  evidence_basis: \
                     unevidenced\nrelations:\n  governs:\n{entry}---\n\n# A decision that \
                     governs a tool\n\n## Context\n\nA fixture.\n\n## Decision\n\nIt governs \
                     one file.\n\n## Consequences\n\nThe route names it.\n"
                ),
            )
            .expect("the decision writes");
        };
        decision(
            "HW-DR-0002",
            "0002-the-decision-whose-edge-went-stale.md",
            "    - to: tools/stale.sh\n      verified_revision: sha256:0000\n",
        );
        decision(
            "HW-DR-0003",
            "0003-the-decision-that-recorded-nothing.md",
            "    - tools/governed.sh\n",
        );
    })
}

/// An edge whose recorded revision differs from the one its target has now is
/// named on the pointer, in both formats. An edge that recorded nothing is
/// not, and its pointer carries the member empty (#953).
#[test]
fn a_governing_edge_whose_recorded_revision_moved_is_named_on_its_pointer() {
    let root = stale_root("route-suspect");
    let ran = root.run(&["route", "edit", "tools/stale.sh", "--json"]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    let start = ran
        .out
        .find("\"suspect\"")
        .unwrap_or_else(|| panic!("no suspect member in {}", ran.out));
    let member = &ran.out[start..];
    let member = &member[..=member.find(']').expect("the array closes")];
    assert!(member.contains("\"target\": \"tools/stale.sh\""), "{member}");
    assert!(member.contains("\"verified\": \"sha256:0000\""), "{member}");
    assert!(member.contains("\"current\": \"sha256:"), "{member}");

    let text = root.run(&["route", "edit", "tools/stale.sh"]);
    assert_eq!(text.code, Some(0), "{text:?}");
    assert!(
        text.out.contains("suspect: tools/stale.sh") && text.out.contains("headwater check"),
        "{}",
        text.out
    );

    let quiet = root.run(&["route", "edit", "tools/governed.sh", "--json"]);
    assert_eq!(quiet.code, Some(0), "{quiet:?}");
    assert!(quiet.out.contains("\"suspect\": []"), "{}", quiet.out);
    let quiet = root.run(&["route", "edit", "tools/governed.sh"]);
    assert!(!quiet.out.contains("suspect"), "{}", quiet.out);
}

/// A word that a glob edge reaches as written, and whose shorter reading is on
/// the tree, names the file on the tree. A `**` edge admits
/// `tools/governed.sh's` as a string, and the route used to name that string
/// as the anchor (#953).
#[test]
fn a_reading_on_the_tree_wins_over_a_longer_one_only_a_glob_edge_reaches() {
    let root = Root::shaped("route-glob-reading", |at| {
        std::fs::write(at.join("tools/governed.sh"), "").expect("the governed file writes");
        std::fs::write(
            at.join("docs/decisions/0002-the-decision-that-governs-the-tools.md"),
            "---\nid: HW-DR-0002\ntitle: The decision that governs the tools\nstatus: \
             current\nstatus_since: 2026-08-01\nlast_verified: 2026-08-01\nsummary: One \
             decision that governs every file under the tools directory.\nprovenance:\n  \
             warrant: asserted\n  agency: human\n  evidence_basis: unevidenced\nrelations:\n  \
             governs:\n    - tools/**\n---\n\n# The decision that governs the tools\n\n## \
             Context\n\nA fixture.\n\n## Decision\n\nIt governs a directory.\n\n## \
             Consequences\n\nThe route names it.\n",
        )
        .expect("the decision writes");
    });
    let ran = root.run(&["route", "edit", "tools/governed.sh's", "header", "--json"]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    let start = ran.out.find("\"anchors\"").expect("the member is there");
    let member = &ran.out[start..];
    let member = &member[..=member.find(']').expect("the array closes")];
    assert!(member.contains("\"tools/governed.sh\""), "{member}");
    assert!(!member.contains("governed.sh's"), "{member}");
}
