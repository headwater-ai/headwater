// SPDX-License-Identifier: Apache-2.0
//! Spec 6's `### CLI` keeps the grammar and the three rules that no verb
//! contract states, and each verb's own rules live in its contract under
//! `docs/interfaces/` (#1572, slice 4a).
//!
//! The section keeps its heading, which inbound links name, and its fenced
//! grammar block, which `verbs.rs` and `completions.rs` read. Outside the
//! fence it keeps three paragraphs and one pointer paragraph. Every other
//! paragraph it held was a rule of one verb, and that rule is now stated once,
//! in the contract of that verb.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn read(rel: &str) -> String {
    let path = repo().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn spec_six() -> String {
    read("docs/spec/06-engine-architecture.md")
}

/// Text with Markdown link targets, emphasis and code ticks removed, in lower
/// case, with runs of whitespace folded to one space.
fn plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find("](") {
        out.push_str(&rest[..i]);
        rest = &rest[i + 2..];
        rest = rest.find(')').map_or("", |j| &rest[j + 1..]);
    }
    out.push_str(rest);
    let out = out.replace(['*', '`', '['], "").to_lowercase();
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Plain text cut into words, with punctuation gone.
fn words(text: &str) -> Vec<String> {
    plain(text)
        .split(|c: char| !(c.is_alphanumeric() || c == '\'' || c == '-' || c == '_'))
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

/// The fewest words in a row that spec 6's CLI prose may not share with a
/// verb contract.
const RUN_WORDS: usize = 8;

/// The prose paragraphs of spec 6's `### CLI`, from its heading to the next
/// heading, with the fenced grammar block left out. Each paragraph is one
/// logical line, because no Markdown source here is hard-wrapped.
fn cli_paragraphs(six: &str) -> Vec<String> {
    let mut lines = six.lines().skip_while(|l| *l != "### CLI");
    assert!(lines.next().is_some(), "spec 6 has no `### CLI`");
    let mut out = Vec::new();
    let mut fenced = false;
    for line in lines.take_while(|l| !l.starts_with('#')) {
        if line.starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced || line.trim().is_empty() {
            continue;
        }
        out.push(line.to_owned());
    }
    assert!(!fenced, "spec 6's `### CLI` has an unterminated fence");
    out
}

/// The lead sentences of the three paragraphs spec 6 keeps, because no verb
/// contract states their rules.
const KEPT_LEADS: [&str; 3] = [
    "This grammar is a statement of fact about the engine",
    "The CLI is advisory by default",
    "No flag decides which findings count",
];

/// The lead sentence of each of the 17 paragraphs that left spec 6, in the
/// order spec 6 held them.
const MOVED_LEADS: [&str; 17] = [
    "--fix writes the patch that rides with a finding",
    "new is the one verb that writes a document, and six flags carry its rules",
    "new also writes one line that is not a document",
    "sweep is two verbs and neither one reaches a model",
    "taxonomy vendor takes a path or a location",
    "json is the one verb that reads no corpus",
    "merge-driver is the one verb that git calls and a person does not",
    "conformance reads a rule set the package ships, and two of its flags carry a rule",
    "probe is four verbs, and none of them reaches a model",
    "The recorder is a separate process for a reason a flag could not carry",
    "A result is a function of the transcript, the declared expectations and the version of the grader",
    "The budget declaration is .headwater/probe.yml, beside the lock",
    "export is the projection contract under another verb",
    "--format names one target, and it writes to standard output",
    "A refusal writes nothing to standard output, and it is never a document in the named target",
    "--read-set writes what the report already states",
    "gate is that reader, and spec 12 states the one test it runs",
];

/// The longest pointer paragraph, in words.
const POINTER_WORDS: usize = 80;

/// Outside its fence, spec 6's `### CLI` holds the three kept paragraphs and
/// one short pointer to the contracts, and none of the paragraphs that left.
///
/// # Watched failing
///
/// On `main` at `d5c25f0a` this reddened: the section held 20 paragraphs
/// outside the fence, every one of the 17 moved leads among them.
#[test]
fn spec_6_cli_keeps_only_the_grammar_and_the_three_rules_no_contract_states() {
    let paragraphs = cli_paragraphs(&spec_six());
    let plains: Vec<String> = paragraphs.iter().map(|p| plain(p)).collect();
    let still: Vec<&str> = MOVED_LEADS
        .iter()
        .copied()
        .filter(|lead| plains.iter().any(|p| p.contains(&plain(lead))))
        .collect();
    assert!(
        still.is_empty(),
        "spec 6's `### CLI` still states these moved paragraphs: {still:#?}"
    );
    for lead in KEPT_LEADS {
        let lead = plain(lead);
        assert_eq!(
            plains.iter().filter(|p| p.starts_with(&lead)).count(),
            1,
            "spec 6's `### CLI` must keep one paragraph opening {lead:?}"
        );
    }
    let others: Vec<&String> = plains
        .iter()
        .filter(|p| !KEPT_LEADS.iter().any(|k| p.starts_with(&plain(k))))
        .collect();
    assert!(
        others.len() <= 1,
        "spec 6's `### CLI` holds {} paragraphs beside the three it keeps, and the bar is one \
         pointer: {others:#?}",
        others.len()
    );
    for pointer in &paragraphs {
        if KEPT_LEADS.iter().any(|k| plain(pointer).starts_with(&plain(k))) {
            continue;
        }
        let count = pointer.split_whitespace().count();
        assert!(
            count <= POINTER_WORDS,
            "the pointer paragraph of spec 6's `### CLI` is {count} words, and the bar is \
             {POINTER_WORDS}"
        );
        assert!(
            pointer.contains("../interfaces/README.md"),
            "the pointer paragraph of spec 6's `### CLI` does not link the interface index"
        );
    }
}

/// For each rule that moved and that its contract did not already state, the
/// contract and a sentence of that rule, as `plain` writes it.
const MOVED_RULES: [(&str, &str); 12] = [
    (
        "docs/interfaces/headwater-check.md",
        "the run opens every file of the batch before it writes the first byte",
    ),
    (
        "docs/interfaces/headwater-check.md",
        "where the engine guessed the shape of a document wrong, the verb refuses it",
    ),
    (
        "docs/interfaces/headwater-check.md",
        "the flag decides no finding and moves no verdict",
    ),
    (
        "docs/interfaces/headwater-new.md",
        "the file name and the facet in the name role both come from the title",
    ),
    (
        "docs/interfaces/headwater-new.md",
        "without --summary, that facet carries a prompt for a person to answer",
    ),
    (
        "docs/interfaces/headwater-taxonomy.md",
        "the non-negotiable rests on one call site, the vendor arm",
    ),
    (
        "docs/interfaces/headwater-taxonomy.md",
        "a binary built without the fetch feature opens no socket",
    ),
    (
        "docs/interfaces/headwater-probe.md",
        "no engine code performs the part between plan and record",
    ),
    (
        "docs/interfaces/headwater-probe.md",
        "the harness is a crate that headwater-check cannot name",
    ),
    (
        "docs/interfaces/headwater-export.md",
        "a consumer outside the repository asks for one format at a time",
    ),
    (
        "docs/interfaces/headwater-export.md",
        "a target that no taxonomy declared is still reachable",
    ),
    (
        "docs/interfaces/headwater-export.md",
        "so a filtered audience is never left out by accident",
    ),
];

/// Each rule that left spec 6 is stated in the contract of its verb, and
/// spec 6 no longer states it.
///
/// # Watched failing
///
/// On `main` at `d5c25f0a` this reddened: no contract held any of these
/// sentences.
#[test]
fn every_moved_cli_rule_is_stated_in_its_verb_contract() {
    let six = plain(&spec_six());
    let mut missing = Vec::new();
    let mut doubled = Vec::new();
    for (contract, sentence) in MOVED_RULES {
        if !plain(&read(contract)).contains(sentence) {
            missing.push(format!("{contract}: {sentence:?}"));
        }
        if six.contains(sentence) {
            doubled.push(sentence);
        }
    }
    assert!(
        missing.is_empty(),
        "these moved CLI rules are not stated in their contracts: {missing:#?}"
    );
    assert!(
        doubled.is_empty(),
        "spec 6 still states these moved CLI rules: {doubled:#?}"
    );
}

/// The eleven contracts that a paragraph of spec 6's `### CLI` stated a rule
/// of.
const CONTRACTS: [&str; 11] = [
    "docs/interfaces/headwater-check.md",
    "docs/interfaces/headwater-new.md",
    "docs/interfaces/headwater-capture.md",
    "docs/interfaces/headwater-sweep.md",
    "docs/interfaces/headwater-taxonomy.md",
    "docs/interfaces/headwater-json.md",
    "docs/interfaces/headwater-merge-driver.md",
    "docs/interfaces/headwater-conformance.md",
    "docs/interfaces/headwater-probe.md",
    "docs/interfaces/headwater-export.md",
    "docs/interfaces/headwater-gate.md",
];

/// Spec 6's CLI prose shares no run of eight words with a verb contract, so
/// that no rule is stated in both places. The grammar fence is left out,
/// because the synopsis of a contract is meant to match it.
///
/// # Watched failing
///
/// On `main` at `d5c25f0a` this reddened naming runs that the `--fix`,
/// `json` and `merge-driver` paragraphs share with their contracts.
#[test]
fn spec_6_shares_no_run_of_eight_words_with_a_verb_contract() {
    let prose = cli_paragraphs(&spec_six());
    let mut runs = BTreeSet::new();
    for contract in CONTRACTS {
        let text = format!(" {} ", words(&read(contract)).join(" "));
        for paragraph in &prose {
            for run in words(paragraph).windows(RUN_WORDS) {
                let run = run.join(" ");
                if text.contains(&format!(" {run} ")) {
                    runs.insert(format!("{contract}: {run}"));
                }
            }
        }
    }
    assert!(
        runs.is_empty(),
        "spec 6's `### CLI` shares these runs of {RUN_WORDS} words with a verb contract: \
         {runs:#?}"
    );
}

fn engine_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read an engine directory") {
        let path = entry.expect("read a directory entry").path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() {
            if name != "target" && !name.starts_with('.') {
                engine_sources(&path, out);
            }
        } else if name.ends_with(".rs") {
            out.push(path);
        }
    }
}

/// Clauses of the moved CLI rules that a comment can credit to spec 6. Each
/// one is a clause of a rule, never a word such as "fix", "export" or
/// "network", so a comment that names the verb and credits spec 6 with its
/// grammar does not match.
const MOVED_CLAIMS: [&str; 6] = [
    "asks for one format at a time",
    "with no profile named, the engine writes every declared profile",
    "reaches a model over the network",
    "the batch lands as a set",
    "rests on one call site",
    "the recorder is a separate process",
];

/// Where `text`, in lower case, first names spec 6: the words `spec 6`, or a
/// link into its file, whatever the link text says.
fn spec_six_mention(text: &str) -> Option<usize> {
    ["spec 6", "06-engine-architecture.md"]
        .iter()
        .filter_map(|m| text.find(m))
        .min()
}

/// A comment that credits spec 6 with a moved CLI rule points a reader at a
/// part that no longer holds it.
///
/// It reads every comment line that names spec 6, joined with the five lines
/// after it, and fails when the text from that mention on states one of
/// [`MOVED_CLAIMS`] within 240 bytes. This file is skipped, because it names
/// the claims it looks for.
///
/// # Watched failing
///
/// On `main` at `d5c25f0a` this reddened naming `cli/src/main.rs`,
/// `generate/src/lib.rs` and `probe/src/lib.rs`.
#[test]
fn no_comment_quotes_moved_cli_text_as_spec_6() {
    let engine = repo().join("engine");
    let mut files = Vec::new();
    engine_sources(&engine.join("crates"), &mut files);
    let mut credited = Vec::new();
    let mut read = 0;
    for file in files {
        if file.file_name().and_then(|n| n.to_str()) == Some("spec_six_cli_rules.rs") {
            continue;
        }
        let text = std::fs::read_to_string(&file).expect("read an engine source");
        let rel = file
            .strip_prefix(&engine)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if spec_six_mention(&line.to_lowercase()).is_none() {
                continue;
            }
            if !line.trim_start().starts_with("//") {
                continue;
            }
            let joined = lines[i..lines.len().min(i + 6)]
                .iter()
                .map(|l| l.trim().trim_start_matches(['/', '!']).trim())
                .collect::<Vec<_>>()
                .join(" ");
            let Some(at) = spec_six_mention(&joined.to_lowercase()) else {
                continue;
            };
            read += 1;
            let window: String = plain(&joined[at..]).chars().take(240).collect();
            if let Some(claim) = MOVED_CLAIMS.iter().find(|c| window.contains(*c)) {
                credited.push(format!("{rel}:{}: {claim:?}", i + 1));
            }
        }
    }
    assert!(read > 0, "no engine comment names spec 6 at all");
    assert!(
        credited.is_empty(),
        "these comments credit spec 6 with a CLI rule that a verb contract now states: \
         {credited:#?}"
    );
}

/// The module comment of the probe harness does not say that `probe` reaches
/// a model over the network, whatever it credits. The contract of the verb
/// says that no subcommand opens a network connection, and the comment once
/// said the opposite under a link to spec 6.
///
/// # Watched failing
///
/// With the sentence "`probe` reaches a model over the network." put back
/// into `probe/src/lib.rs`, with no link to spec 6 beside it, this reddened,
/// and `no_comment_quotes_moved_cli_text_as_spec_6` stayed green.
#[test]
fn the_probe_module_comment_does_not_say_probe_reaches_the_network() {
    let text = read("engine/crates/probe/src/lib.rs");
    let comments: String = text
        .lines()
        .map(str::trim_start)
        .filter(|l| l.starts_with("//"))
        .map(|l| l.trim_start_matches(['/', '!']).trim())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        !plain(&comments).contains("probe reaches a model over the network"),
        "a comment in probe/src/lib.rs says that probe reaches a model over the network, and \
         docs/interfaces/headwater-probe.md says that no subcommand opens a network connection"
    );
    assert!(
        text.contains("docs/interfaces/headwater-probe.md"),
        "the module comment of probe/src/lib.rs does not link the contract of the verb"
    );
}
