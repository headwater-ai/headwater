// SPDX-License-Identifier: Apache-2.0
//! The decisive fixture for the digest printer and the baseline stamp (#2016).
//!
//! # The defect this target exists for
//!
//! A `governs` edge records `verified_revision`, and `relation.target.suspect`
//! compares it with the digest the edge reaches now. Before #2016 the only
//! route to a first digest was a change that re-verified the document, so a
//! corpus that adopted the rule late re-read every document to stamp it, and
//! no verb printed the digest a stamp would write.
//!
//! # Why these cases drive the binary
//!
//! The claim is a round trip through the verbs and the file between them:
//! the digest `explain` prints is the one `check --fix --baseline` writes,
//! the stamp says it is a baseline, `check` names the baseline when the
//! target moves, and a later reading drops the marker. A case that called
//! the functions would prove they agree inside one process, and not that the
//! document carries the agreement.

mod common;

use common::Root;

const DOCUMENT: &str = "docs/interfaces/thing.md";
const SCALAR: &str = "tools/a.sh";
const LIST: &str = "tools/b.sh, tools/c.sh";

/// A scratch root that holds one interface contract with two unstamped
/// `governs` entries, a scalar and a bare list, and the three files they
/// govern.
fn governed(label: &str) -> Root {
    Root::shaped(label, |at| {
        std::fs::create_dir_all(at.join("docs/interfaces")).expect("the shelf is made");
        std::fs::write(
            at.join(DOCUMENT),
            "---\nid: HW-IFACE-thing\nstatus: current\nstatus_since: 2026-11-23\n\
             summary: \"A fixture contract that governs three tools.\"\n\
             last_verified: 2026-11-23\ntitle: \"thing\"\nrelations:\n  governs:\n    \
             - tools/a.sh\n    - [tools/b.sh, tools/c.sh]\n---\n\n# thing\n\n\
             A fixture contract for #2016.\n",
        )
        .expect("the contract writes");
        std::fs::create_dir_all(at.join("tools")).expect("the tools directory is made");
        for (name, bytes) in [("a", "echo a\n"), ("b", "echo b\n"), ("c", "echo c\n")] {
            std::fs::write(at.join(format!("tools/{name}.sh")), bytes).expect("the tool writes");
        }
    })
}

/// One field of the related edge of `DOCUMENT` whose target is `target`, as
/// `explain --json` writes it and `headwater json field` reads it. `None`
/// where the edge writes no such field.
fn field(root: &Root, target: &str, path: &[&str]) -> Option<String> {
    let explained = root.run(&["explain", DOCUMENT, "--json"]);
    assert_eq!(explained.code, Some(0), "{explained:?}");
    for index in 0.. {
        let at = index.to_string();
        let named = root.run_with(&["json", "field", "related", &at, "target"], &explained.out);
        assert_eq!(
            named.code,
            Some(0),
            "no related edge onto {target}: {explained:?}"
        );
        if named.out.trim_end() != target {
            continue;
        }
        let mut arguments = vec!["json", "field", "related", &at];
        arguments.extend_from_slice(path);
        let read = root.run_with(&arguments, &explained.out);
        return match read.code {
            Some(0) => Some(read.out.trim_end().to_string()),
            _ => None,
        };
    }
    unreachable!()
}

fn contract(root: &Root) -> String {
    std::fs::read_to_string(root.at.join(DOCUMENT)).expect("the contract reads")
}

/// The lines of the text report that `relation.target.suspect` wrote.
fn suspect(out: &str) -> Vec<&str> {
    out.lines()
        .map(str::trim)
        .filter(|line| line.starts_with("relation.target.suspect"))
        .collect()
}

/// Steps 1 to 4 of the adjudication's fixture: the printed digest is the
/// stamped one, the list takes the stamp in the mapping form, and a second
/// run writes nothing.
fn stamps_what_explain_printed(root: &Root, before: &[&str]) {
    let scalar = field(root, SCALAR, &["digest", "current"]).expect("a digest for the scalar");
    let list = field(root, LIST, &["digest", "current"]).expect("a digest for the list");
    assert!(scalar.starts_with("sha256:"), "{scalar}");
    assert_ne!(scalar, list);
    assert_eq!(field(root, SCALAR, &["digest", "recorded"]), None);

    for arguments in before {
        let ran = root.run(&[arguments]);
        assert_eq!(ran.code, Some(0), "{ran:?}");
    }
    let fixed = root.run(&["check", "--fix", "--baseline"]);
    assert_eq!(fixed.code, Some(0), "{fixed:?}");
    let text = contract(root);
    assert!(text.contains("- to: [tools/b.sh, tools/c.sh]"), "{text}");
    assert_eq!(text.matches("verified_as: baseline").count(), 2, "{text}");
    for (target, digest) in [(SCALAR, &scalar), (LIST, &list)] {
        assert_eq!(
            field(root, target, &["digest", "recorded"]).as_ref(),
            Some(digest),
            "{target}: {text}"
        );
        assert_eq!(
            field(root, target, &["digest", "baseline"]).as_deref(),
            Some("true"),
            "{target}"
        );
    }

    let again = root.run(&["check", "--fix", "--baseline"]);
    assert_eq!(again.code, Some(0), "{again:?}");
    assert_eq!(contract(root), text, "a second run changed the contract");
}

/// The round trip the issue exists for, steps 1 to 4 and 6 to 7.
#[test]
fn a_baseline_stamp_writes_the_printed_digest_and_says_it_is_a_baseline() {
    let root = governed("round-trip");
    stamps_what_explain_printed(&root, &[]);

    // Step 6: one byte of a list member moves.
    std::fs::write(root.at.join("tools/c.sh"), "echo C\n").expect("the tool writes");
    let checked = root.run(&["check"]);
    let found = suspect(&checked.out);
    assert_eq!(found.len(), 1, "{}", checked.out);
    // The text report wraps a finding, so the case reads it as one line.
    let flat = checked.out.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        flat.contains(&format!("declares `governs: {LIST}`")),
        "{flat}"
    );
    assert!(
        flat.contains(
            "The recorded digest is a baseline, written by `headwater check --fix --baseline`, \
             and it records no reading of what the entry reaches"
        ),
        "{flat}"
    );
    let current = field(&root, LIST, &["digest", "current"]).expect("a digest");
    let recorded = field(&root, LIST, &["digest", "recorded"]).expect("a recorded digest");
    assert_ne!(current, recorded);
    let explained = root.run(&["explain", DOCUMENT]);
    let flat = explained
        .out
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        flat.contains(&format!(
            "(content now {current}, recorded {recorded} as a baseline, which records no reading)"
        )),
        "{flat}"
    );

    // Step 7: a change states that the author re-read the list edge.
    let manifest = root.at.join("manifest");
    std::fs::write(
        &manifest,
        format!("headwater change 1\nverified\t{DOCUMENT}\ttools/c.sh\n"),
    )
    .expect("the manifest writes");
    let manifest = manifest.to_string_lossy().into_owned();
    let read = root.run(&["check", "--fix", "--change", &manifest]);
    assert_eq!(read.code, Some(0), "{read:?}");
    let text = contract(&root);
    assert_eq!(text.matches("verified_as: baseline").count(), 1, "{text}");
    assert_eq!(
        field(&root, LIST, &["digest", "recorded"]),
        Some(current),
        "{text}"
    );
    assert_eq!(
        field(&root, LIST, &["digest", "baseline"]).as_deref(),
        Some("false")
    );
    assert_eq!(
        field(&root, SCALAR, &["digest", "baseline"]).as_deref(),
        Some("true")
    );
}

/// Step 5: a warm cache from a run without `--baseline` does not serve the
/// unpatched verdict to a run with it.
#[test]
fn a_warm_cache_does_not_hide_the_baseline_stamp() {
    let root = governed("warm");
    stamps_what_explain_printed(&root, &["check"]);
}

/// Without `--fix`, `--baseline` reports each stamp it would write and writes
/// nothing.
#[test]
fn a_baseline_without_fix_reports_and_writes_nothing() {
    let root = governed("report");
    let before = contract(&root);
    let ran = root.run(&["check", "--baseline"]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert_eq!(suspect(&ran.out).len(), 2, "{}", ran.out);
    assert_eq!(contract(&root), before);
}
