// SPDX-License-Identifier: Apache-2.0
//! A document that still governs does not credit spec 6 for a rule that moved
//! out of it (#1572, slice 4c, HW-DR-0106).
//!
//! Seven sections of spec 6 now state only a pointer to the home of their
//! rule. A governing document that links one of them for the rule itself is
//! repointed to the home, and nothing it claims changes. A record of a moment
//! (an evaluation, a review, a probe run, a probe result, a superseded
//! decision) stays as written, so the walk skips it, and a table holds the
//! count of each link that a record keeps to one of the seven sections.
//!
//! A second walk reads credits in prose (#1572 clause 9). A sentence that
//! names spec 6, as a link to `06-engine-architecture.md` with or without an
//! anchor or as the plain words, and puts a present-tense credit verb after
//! the name, with at most the relative pronoun "which" and one adverb between
//! them, credits that part with what follows. When that sentence or
//! the next one holds a term of `MOVED_CREDITS`, the credit is to a rule that
//! moved, and the sentence is repointed and its claim is corrected in place
//! (HW-DR-0106). A past-tense verb (said, stated) is history and is not read.
//! A third walk holds the records of a moment to the credits they already
//! keep, so that a record is not edited to add or drop one.
//!
//! Neither walk can see a present-tense credit to a moved rule whose wording
//! no row of `MOVED_CREDITS` names. A reader checks those, and a row is added
//! for each one found.

use std::collections::{BTreeMap, BTreeSet};
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

/// The rules and counts that spec 6 stated once and states no more, each with
/// its home now. Each row is `(term, home, held)`: `term` is the wording of a
/// credit to the old rule, lower case, and `held` is the wording at `home`
/// that holds the rule now.
const MOVED_CREDITS: &[(&str, &str, &str)] = &[
    // The probe budget sits outside the root. Spec 5 states the three
    // consequences, and the measurement subsystem states that nothing
    // recomputes a budget.
    (
        "probe budget",
        "docs/spec/05-ai-integration.md",
        "no taxonomy rule reads it, no language regime binds it, and no shelf classifies it",
    ),
    (
        "probe budget",
        "docs/subsystems/measurement.md",
        "a budget is a policy, and nothing recomputes it",
    ),
    // No crate of the checking loop opens a socket, and headwater-fetch is the
    // one crate that does.
    (
        "socket",
        "docs/requirements/0001-the-engine-reaches-no-network-at-check-time.md",
        "`headwater-fetch` opens a socket",
    ),
    // What `--format` writes, and what a refusal writes, is in each verb
    // contract.
    (
        "puts on standard output",
        "docs/interfaces/headwater-check.md",
        "a refusal writes nothing to standard output",
    ),
    // The `counted` grain of an export profile.
    (
        "placeholder",
        "docs/spec/07-distribution-and-federation.md",
        "a placeholder sits where each withheld node or edge would have been",
    ),
    // What a tombstone gives a reader: the rule identifier, and under
    // `counted` the digests that HW-DR-0100 added.
    (
        "all that a reader gets",
        "docs/spec/07-distribution-and-federation.md",
        "the rule identifier is what a reader needs to ask for access, and it is all that they get",
    ),
    // The generated-file marker and the closed identity block.
    (
        "when it was generated",
        "docs/interfaces/headwater-generate.md",
        "no generated file states when it was generated",
    ),
    (
        "refuses that shape",
        "docs/interfaces/headwater-generate.md",
        "three scalars: the identifier, the kind and the name",
    ),
    // A count: spec 6 still lists the projection kinds, and the count moved.
    (
        "ten declarable",
        SPEC_SIX,
        "eleven of the thirteen are declarable",
    ),
];

/// The verbs that make a sentence a present-tense credit to spec 6 when one
/// of them is the first word after the name.
const CREDIT_VERBS: &[&str] = &[
    "says",
    "states",
    "puts",
    "lists",
    "rules",
    "refuses",
    "describes",
    "declares",
    "requires",
    "gives",
    "calls",
    "closes",
];

/// The words that may stand between the name and the verb.
const ADVERBS: &[&str] = &["then", "also", "still", "now", "already", "only"];

/// The sentences of one line. A sentence ends at a full stop, a question
/// mark or an exclamation mark that a space or the end of the line follows.
fn sentences(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let bytes = line.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if matches!(b, b'.' | b'?' | b'!') && bytes.get(i + 1).is_none_or(|n| *n == b' ') {
            out.push(line[start..=i].trim());
            start = i + 1;
        }
    }
    let tail = line[start..].trim();
    if !tail.is_empty() {
        out.push(tail);
    }
    out
}

/// The byte offset just after each place that `sentence` names spec 6.
fn spec_six_names(sentence: &str) -> Vec<usize> {
    let mut ends = Vec::new();
    // A link: the name ends at the `)` that closes its target.
    let mut from = 0;
    while let Some(at) = sentence[from..].find(SPEC_SIX_FILE) {
        let after = from + at + SPEC_SIX_FILE.len();
        if let Some(close) = sentence[after..].find(')') {
            ends.push(after + close + 1);
        }
        from = after;
    }
    // The plain words, with a space or a comma after them, so that a link's
    // text ("[Spec 6]") is read once, as the link.
    let lower = sentence.to_lowercase();
    let mut from = 0;
    while let Some(at) = lower[from..].find("spec 6") {
        let end = from + at + "spec 6".len();
        if matches!(lower.as_bytes().get(end), Some(b' ' | b',')) {
            ends.push(end);
        }
        from = end;
    }
    ends
}

/// True when `sentence` names spec 6 and the first word after the name, past
/// at most one relative pronoun (`which`) and then at most one adverb, is a
/// present-tense credit verb.
fn credits_spec_six(sentence: &str) -> bool {
    spec_six_names(sentence).into_iter().any(|end| {
        let mut words = sentence[end..]
            .split_whitespace()
            .map(|w| {
                w.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase()
            })
            .filter(|w| !w.is_empty());
        let Some(mut word) = words.next() else {
            return false;
        };
        if word == "which" {
            let Some(next) = words.next() else {
                return false;
            };
            word = next;
        }
        if ADVERBS.contains(&word.as_str()) {
            let Some(next) = words.next() else {
                return false;
            };
            word = next;
        }
        CREDIT_VERBS.contains(&word.as_str())
    })
}

/// Every `(line, term)` in `text` where a present-tense credit to spec 6,
/// read with the sentence after it, holds a term of `MOVED_CREDITS`, for a
/// governing `rel`. A line is reported once for each term.
fn moved_credits(rel: &str, text: &str) -> Vec<(usize, &'static str)> {
    if !governs(rel, text) {
        return Vec::new();
    }
    credit_lines(text)
}

/// Every `(line, term)` in `text` where a present-tense credit to spec 6,
/// read with the sentence after it, holds a term of `MOVED_CREDITS`, whatever
/// document holds it.
fn credit_lines(text: &str) -> Vec<(usize, &'static str)> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let parts = sentences(line);
        for (k, sentence) in parts.iter().enumerate() {
            if !credits_spec_six(sentence) {
                continue;
            }
            let mut window = sentence.to_lowercase();
            if let Some(next) = parts.get(k + 1) {
                window.push(' ');
                window.push_str(&next.to_lowercase());
            }
            for (term, _, _) in MOVED_CREDITS {
                if window.contains(term) && !out.contains(&(i + 1, *term)) {
                    out.push((i + 1, *term));
                }
            }
        }
    }
    out
}

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
/// spec 6, whatever document holds it.
fn pointer_links(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let needle = format!("{SPEC_SIX_FILE}#");
    for (i, line) in text.lines().enumerate() {
        let mut rest = line;
        while let Some(at) = rest.find(&needle) {
            let after = &rest[at + needle.len()..];
            let end = after.find([')', ' ', '"', '>']).unwrap_or(after.len());
            let anchor = &after[..end];
            if POINTER_ONLY.contains(&anchor) {
                out.push((i + 1, anchor.to_owned()));
            }
            rest = &after[end..];
        }
    }
    out
}

/// Every `(line, anchor)` in `text` that links a pointer-only section of
/// spec 6 and that the allow table does not hold, for a governing `rel`.
fn offending(rel: &str, text: &str) -> Vec<(usize, String)> {
    if !governs(rel, text) {
        return Vec::new();
    }
    pointer_links(text)
        .into_iter()
        .filter(|(_, anchor)| !ALLOWED.iter().any(|(p, a, _)| *p == rel && a == anchor))
        .collect()
}

/// Every Markdown file the walk reads, as a path relative to the root.
fn corpus_files() -> Vec<String> {
    let root = repo().canonicalize().expect("repository root");
    let mut files = Vec::new();
    markdown_under(&root.join("docs"), &mut files);
    markdown_under(&root.join(".claude"), &mut files);
    files.push(root.join("README.md"));
    files.push(root.join("engine/README.md"));
    let mut rels: Vec<String> = files
        .iter()
        .map(|p| {
            p.strip_prefix(&root)
                .expect("under the root")
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    rels.sort();
    rels
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
    let mut hits = Vec::new();
    let mut walked = 0;
    for rel in corpus_files() {
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
fn no_governing_sentence_credits_spec_6_in_the_present_tense_with_a_rule_it_no_longer_holds() {
    let mut hits = Vec::new();
    let mut walked = 0;
    for rel in corpus_files() {
        let text = read(&rel);
        walked += 1;
        for (line, term) in moved_credits(&rel, &text) {
            hits.push(format!("{rel}:{line} \"{term}\""));
        }
    }
    assert!(
        walked > 100,
        "the walk read {walked} files; it found no corpus"
    );
    assert!(
        hits.is_empty(),
        "{} governing line(s) credit spec 6 in the present tense with a rule it no longer holds; repoint each to the home MOVED_CREDITS names and correct the claim in place (HW-DR-0106):\n{}",
        hits.len(),
        hits.join("\n")
    );
}

#[test]
fn each_moved_term_is_gone_from_spec_6_and_held_at_its_home() {
    let spec_six = read(SPEC_SIX).to_lowercase();
    for (term, home, held) in MOVED_CREDITS {
        assert!(
            !spec_six.contains(term),
            "spec 6 holds \"{term}\" again; drop its MOVED_CREDITS row or move the text"
        );
        assert!(
            read(home).to_lowercase().contains(held),
            "{home} no longer holds \"{held}\"; find the new home of \"{term}\""
        );
    }
}

#[test]
fn a_present_tense_credit_to_a_moved_rule_is_flagged_and_history_is_not() {
    let decision = "docs/decisions/0500-a-decision.md";
    let budget =
        "[Spec 6](../spec/06-engine-architecture.md#cli) puts the probe budget outside it.\n";
    assert_eq!(moved_credits(decision, budget), vec![(1, "probe budget")]);

    // An adverb may stand between the name and the verb, and a bare link is a
    // name.
    let then = "[Spec 6](../spec/06-engine-architecture.md) then puts the probe budget there.\n";
    assert_eq!(moved_credits(decision, then).len(), 1);

    // The same line in a record of a moment is not flagged.
    for prefix in RECORDS_OF_A_MOMENT {
        let rel = format!("{prefix}x.md");
        assert!(moved_credits(&rel, budget).is_empty(), "{rel} was flagged");
    }

    // The past tense is history.
    let said = "Spec 6 said that under `counted` a placeholder sat there.\n";
    assert!(moved_credits(decision, said).is_empty());
    let past = "When this was written, spec 6 said what `--format` puts on standard output.\n";
    assert!(moved_credits(decision, past).is_empty());

    // The plain words are a name.
    let plain = "Spec 6 says that under counted a placeholder sits there.\n";
    assert_eq!(moved_credits(decision, plain), vec![(1, "placeholder")]);
    let lower = "It is not the placeholder that spec 6 describes.\n";
    assert_eq!(moved_credits(decision, lower), vec![(1, "placeholder")]);

    // A credit is read with the sentence after it, and not the one after that.
    let next = "[Spec 6](../spec/06-engine-architecture.md#cli) states two things. No crate opens a socket.\n";
    assert_eq!(moved_credits(decision, next), vec![(1, "socket")]);
    let far = "Spec 6 states one thing. It is a shell. No crate opens a socket.\n";
    assert!(moved_credits(decision, far).is_empty());

    // A credit to a rule that spec 6 still states is not flagged.
    let kept = "[Spec 6](../spec/06-engine-architecture.md#library) states that the CLI is a thin shell.\n";
    assert!(moved_credits(decision, kept).is_empty());

    // A present-tense credit to another part is not a credit to spec 6.
    let other = "[Spec 5](../spec/05-ai-integration.md) puts the probe budget outside it.\n";
    assert!(moved_credits(decision, other).is_empty());

    // A relative clause may stand between the name and the verb, after the
    // plain words and after a link (HW-DR-0100 wrote "spec 6, which says").
    let which = "It amends one sentence of spec 6, which says that the rule identifier is all that a reader gets.\n";
    assert_eq!(
        moved_credits(decision, which),
        vec![(1, "all that a reader gets")]
    );
    let still = "It is the placeholder of spec 6, which still states it.\n";
    assert_eq!(moved_credits(decision, still), vec![(1, "placeholder")]);
    let linked = "It amends [spec 6](../spec/06-engine-architecture.md#cli), which says what a placeholder is.\n";
    assert_eq!(moved_credits(decision, linked), vec![(1, "placeholder")]);

    // A relative clause that credits nothing is not a credit, and neither is a
    // past-tense one.
    let split = "The placeholder rule left spec 6, which was split in HW-DR-0106.\n";
    assert!(moved_credits(decision, split).is_empty());
    let which_said = "It amends spec 6, which said that a placeholder sits there.\n";
    assert!(moved_credits(decision, which_said).is_empty());

    // The same relative clause in a record of a moment is not flagged.
    for prefix in RECORDS_OF_A_MOMENT {
        let rel = format!("{prefix}x.md");
        assert!(moved_credits(&rel, which).is_empty(), "{rel} was flagged");
    }
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

/// The links that the records of a moment hold to a pointer-only section of
/// spec 6, with the count of each, measured on 2026-10-03. A record of a
/// moment stays as written (HW-DR-0106), so a repoint inside one lowers a
/// count and a new link raises one, and the test compares the whole table.
/// Each row is `(path, anchor, count)`.
const RECORD_LINKS: &[(&str, &str, usize)] = &[
    (
        "docs/evaluations/adjacent-work.md",
        "an-export-is-a-projection-and-it-declares-what-it-dropped",
        3,
    ),
    (
        "docs/evaluations/adjacent-work.md",
        "an-export-profile-carries-a-filter",
        6,
    ),
    ("docs/evaluations/adjacent-work.md", "ci-adapters", 1),
    (
        "docs/evaluations/adjacent-work.md",
        "what-a-filtered-export-claims-and-what-it-does-not",
        4,
    ),
    (
        "docs/evaluations/first-contact.md",
        "what-a-filtered-export-claims-and-what-it-does-not",
        2,
    ),
    (
        "docs/evaluations/graph-export-and-federation.md",
        "checks",
        1,
    ),
    (
        "docs/evaluations/the-measurement-layer.md",
        "an-export-profile-carries-a-filter",
        2,
    ),
    (
        "docs/evaluations/the-serving-boundary.md",
        "an-export-is-a-projection-and-it-declares-what-it-dropped",
        2,
    ),
    (
        "docs/evaluations/the-serving-boundary.md",
        "an-export-profile-carries-a-filter",
        2,
    ),
    ("docs/evaluations/the-serving-boundary.md", "ci-adapters", 2),
    ("docs/evaluations/the-serving-boundary.md", "mcp-server", 1),
    (
        "docs/evaluations/the-serving-boundary.md",
        "what-a-filtered-export-claims-and-what-it-does-not",
        2,
    ),
    (
        "docs/evaluations/warrant-and-adjudication.md",
        "an-export-is-a-projection-and-it-declares-what-it-dropped",
        1,
    ),
    (
        "docs/evaluations/warrant-and-adjudication.md",
        "an-export-profile-carries-a-filter",
        2,
    ),
    (
        "docs/evaluations/warrant-and-adjudication.md",
        "what-a-filtered-export-claims-and-what-it-does-not",
        1,
    ),
    (
        "docs/evaluations/what-a-check-can-know.md",
        "ci-adapters",
        2,
    ),
];

/// Each `(path, anchor)` whose count in `found` differs from the table,
/// in either direction: a link that left and a link that arrived.
fn drift(found: &BTreeMap<(String, String), usize>, table: &[(&str, &str, usize)]) -> Vec<String> {
    let recorded: BTreeMap<(String, String), usize> = table
        .iter()
        .map(|(p, a, n)| ((p.to_string(), a.to_string()), *n))
        .collect();
    let mut moved = Vec::new();
    for key in recorded.keys().chain(found.keys()).collect::<BTreeSet<_>>() {
        let want = recorded.get(key).copied().unwrap_or(0);
        let got = found.get(key).copied().unwrap_or(0);
        if got != want {
            moved.push(format!(
                "{} #{}: {got} link(s), recorded {want}",
                key.0, key.1
            ));
        }
    }
    moved
}

#[test]
fn drift_reports_a_link_that_left_and_a_link_that_arrived() {
    let table = &[("docs/evaluations/e.md", "ci-adapters", 2)];
    let key = |a: &str| ("docs/evaluations/e.md".to_owned(), a.to_owned());

    let same = BTreeMap::from([(key("ci-adapters"), 2)]);
    assert!(drift(&same, table).is_empty());

    let left = BTreeMap::from([(key("ci-adapters"), 1)]);
    assert_eq!(drift(&left, table).len(), 1);

    let arrived = BTreeMap::from([(key("ci-adapters"), 2), (key("checks"), 1)]);
    assert_eq!(
        drift(&arrived, table),
        vec!["docs/evaluations/e.md #checks: 1 link(s), recorded 0"]
    );
}

#[test]
fn a_record_of_a_moment_keeps_its_links_to_spec_6() {
    let mut found: BTreeMap<(String, String), usize> = BTreeMap::new();
    for rel in corpus_files() {
        if !RECORDS_OF_A_MOMENT.iter().any(|p| rel.starts_with(p)) {
            continue;
        }
        for (_, anchor) in pointer_links(&read(&rel)) {
            *found.entry((rel.clone(), anchor)).or_default() += 1;
        }
    }
    let moved = drift(&found, RECORD_LINKS);
    assert!(
        moved.is_empty(),
        "a record of a moment stays as written (HW-DR-0106), and these links changed:\n{}",
        moved.join("\n")
    );
}

/// The present-tense credits to a moved rule that the records of a moment
/// hold, with the count of each. On 2026-10-03 the records held none. A
/// record stays as written (HW-DR-0106), so a credit added to one or taken
/// out of one fails here. Each row is `(path, term, count)`.
const RECORD_CREDITS: &[(&str, &str, usize)] = &[];

#[test]
fn a_record_of_a_moment_keeps_its_credits_to_spec_6() {
    let mut found: BTreeMap<(String, String), usize> = BTreeMap::new();
    for rel in corpus_files() {
        if !RECORDS_OF_A_MOMENT.iter().any(|p| rel.starts_with(p)) {
            continue;
        }
        for (_, term) in credit_lines(&read(&rel)) {
            *found.entry((rel.clone(), term.to_owned())).or_default() += 1;
        }
    }
    let moved = drift(&found, RECORD_CREDITS);
    assert!(
        moved.is_empty(),
        "a record of a moment stays as written (HW-DR-0106), and these credits changed:\n{}",
        moved.join("\n")
    );
}

#[test]
fn each_of_the_seven_pointer_sections_is_flagged() {
    // The seven sections of spec 6 that state only a pointer, as #1572
    // slice 4c names them. Dropping one from POINTER_ONLY fails here.
    for anchor in [
        "an-export-is-a-projection-and-it-declares-what-it-dropped",
        "an-export-profile-carries-a-filter",
        "what-a-filtered-export-claims-and-what-it-does-not",
        "a-verb-index-reads-the-command-surface-of-the-engine",
        "mcp-server",
        "ci-adapters",
        "checks",
    ] {
        let line = format!("[s](../spec/06-engine-architecture.md#{anchor})\n");
        assert_eq!(
            offending("docs/decisions/0500-a-decision.md", &line),
            vec![(1, anchor.to_owned())],
            "#{anchor} was not flagged"
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
