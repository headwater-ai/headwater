// SPDX-License-Identifier: Apache-2.0
//! A document that still governs does not credit spec 6 for a rule that moved
//! out of it (#1572, slice 4c, HW-DR-0106).
//!
//! Seven sections of spec 6 now state only a pointer to the home of their
//! rule. A governing document that links one of them for the rule itself is
//! repointed to the home, and nothing it claims changes. A record of a moment
//! (an evaluation, a review, a probe run, a probe result, a superseded
//! decision) stays as written, so the walk skips it.
//!
//! The walk cannot see a bare `06-engine-architecture.md` credit, or a credit
//! to a section that still states something. A reader checks those.

use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn read(rel: &str) -> String {
    let path = repo().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

const SPEC_SIX_FILE: &str = "06-engine-architecture.md";
const SPEC_SIX: &str = "docs/spec/06-engine-architecture.md";

/// The sections of spec 6 that state only a pointer to the home of a rule.
const POINTER_ONLY: &[&str] = &[
    "an-export-is-a-projection-and-it-declares-what-it-dropped",
    "an-export-profile-carries-a-filter",
    "what-a-filtered-export-claims-and-what-it-does-not",
    "a-verb-index-reads-the-command-surface-of-the-engine",
    "mcp-server",
    "ci-adapters",
    "checks",
];

/// The records of a moment, by path prefix. Each stays as written.
const RECORDS_OF_A_MOMENT: &[&str] = &[
    "docs/evaluations/",
    "docs/process/evaluations/",
    "docs/reviews/",
    "docs/probe-runs/",
    "docs/probe-results/",
];

/// Links whose sentence credits spec 6 for what the pointer section still
/// says. Each row is `(path, anchor, reason)`.
const ALLOWED: &[(&str, &str, &str)] = &[
    (
        "docs/spec/07-distribution-and-federation.md",
        "an-export-is-a-projection-and-it-declares-what-it-dropped",
        // "spec 6 places the export among the other projections": the pointer
        // section still does that, under ## Projections.
        "places the export among the other projections",
    ),
    (
        "docs/subsystems/queries-and-explain.md",
        "mcp-server",
        // "its MCP server section states the promise of the server" and
        // "mcp.rs is the server of spec 6": the section still names the server
        // as the surface of the library that an agent reads.
        "the server of spec 6",
    ),
];

/// True when the front matter of `text` says `status: superseded`.
fn is_superseded(text: &str) -> bool {
    let Some(rest) = text.strip_prefix("---\n") else {
        return false;
    };
    let Some(end) = rest.find("\n---") else {
        return false;
    };
    rest[..end]
        .lines()
        .any(|l| l.trim() == "status: superseded")
}

/// True when the walk reads `rel` as a governing document.
fn governs(rel: &str, text: &str) -> bool {
    rel != SPEC_SIX
        && !RECORDS_OF_A_MOMENT.iter().any(|p| rel.starts_with(p))
        && !is_superseded(text)
}

/// Every `(line, anchor)` in `text` that links a pointer-only section of
/// spec 6 and that the allow table does not hold, for a governing `rel`.
fn offending(rel: &str, text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    if !governs(rel, text) {
        return out;
    }
    let needle = format!("{SPEC_SIX_FILE}#");
    for (i, line) in text.lines().enumerate() {
        let mut rest = line;
        while let Some(at) = rest.find(&needle) {
            let after = &rest[at + needle.len()..];
            let end = after
                .find(|c: char| c == ')' || c == ' ' || c == '"' || c == '>')
                .unwrap_or(after.len());
            let anchor = &after[..end];
            if POINTER_ONLY.contains(&anchor)
                && !ALLOWED.iter().any(|(p, a, _)| *p == rel && *a == anchor)
            {
                out.push((i + 1, anchor.to_owned()));
            }
            rest = &after[end..];
        }
    }
    out
}

fn markdown_under(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // A git worktree under .claude/worktrees/ is another checkout.
            if path.file_name().is_some_and(|n| n == "worktrees") {
                continue;
            }
            markdown_under(&path, out);
        } else if path.extension().is_some_and(|e| e == "md") {
            out.push(path);
        }
    }
}

#[test]
fn no_governing_document_links_a_pointer_only_section_of_spec_6() {
    let root = repo().canonicalize().expect("repository root");
    let mut files = Vec::new();
    markdown_under(&root.join("docs"), &mut files);
    markdown_under(&root.join(".claude"), &mut files);
    files.push(root.join("README.md"));
    files.push(root.join("engine/README.md"));
    files.sort();

    let mut hits = Vec::new();
    let mut walked = 0;
    for path in &files {
        let rel = path
            .strip_prefix(&root)
            .expect("under the root")
            .to_string_lossy()
            .replace('\\', "/");
        let text = read(&rel);
        walked += 1;
        for (line, anchor) in offending(&rel, &text) {
            hits.push(format!("{rel}:{line} #{anchor}"));
        }
    }
    assert!(
        walked > 100,
        "the walk read {walked} files; it found no corpus"
    );
    assert!(
        hits.is_empty(),
        "{} governing link(s) credit a pointer-only section of spec 6; repoint each to the home the section names (HW-DR-0106):\n{}",
        hits.len(),
        hits.join("\n")
    );
}

#[test]
fn every_allowed_link_is_still_in_its_document() {
    for (rel, anchor, reason) in ALLOWED {
        let text = read(rel);
        assert!(
            text.contains(&format!("{SPEC_SIX_FILE}#{anchor}")),
            "{rel} no longer links #{anchor}; drop the allow row ({reason})"
        );
    }
}

#[test]
fn the_walk_skips_a_record_of_a_moment_and_flags_a_governing_document() {
    let line =
        "See [spec 6](../spec/06-engine-architecture.md#an-export-profile-carries-a-filter).\n";

    // A governing document with a pointer-only link is flagged.
    let hit = offending("docs/decisions/0500-a-decision.md", line);
    assert_eq!(
        hit,
        vec![(1, "an-export-profile-carries-a-filter".to_owned())]
    );

    // The same line in a record of a moment is not.
    for prefix in RECORDS_OF_A_MOMENT {
        let rel = format!("{prefix}a-record.md");
        assert!(offending(&rel, line).is_empty(), "{rel} was flagged");
    }

    // A superseded decision is not.
    let superseded = format!("---\nid: HW-DR-0500\nstatus: superseded\n---\n\n{line}");
    assert!(offending("docs/decisions/0500-a-decision.md", &superseded).is_empty());
    let current = format!("---\nid: HW-DR-0500\nstatus: current\n---\n\n{line}");
    assert_eq!(
        offending("docs/decisions/0500-a-decision.md", &current).len(),
        1
    );

    // A link to a section that still states its rule is not.
    let pipeline = "See [spec 6](../spec/06-engine-architecture.md#pipeline).\n";
    assert!(offending("docs/decisions/0500-a-decision.md", pipeline).is_empty());

    // Spec 6 itself is not.
    let own = "[x](06-engine-architecture.md#ci-adapters)\n";
    assert!(offending(SPEC_SIX, own).is_empty());

    // An allowed row is not, and the same anchor elsewhere is.
    let mcp = "[s](../spec/06-engine-architecture.md#mcp-server)\n";
    assert!(offending("docs/subsystems/queries-and-explain.md", mcp).is_empty());
    assert_eq!(offending("docs/interfaces/headwater-mcp.md", mcp).len(), 1);

    // Two links on one line are two hits.
    let two =
        "[a](06-engine-architecture.md#checks) and [b](06-engine-architecture.md#mcp-server)\n";
    assert_eq!(offending("docs/spec/04-assurance-model.md", two).len(), 2);
}
