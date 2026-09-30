// SPDX-License-Identifier: Apache-2.0
//! Every verb that builds a graph names the two files the graph load reads.
//!
//! Since #1233 the one graph load every verb shares reads `.headwater/imports/`
//! and the file each `harvests.<name>.at` names, and it stops the verb with
//! exit 1 when either declaration does not read. So each such verb reads both
//! files, and its page under `docs/interfaces/` owes both rows in its `## Files`
//! table. [#1312](https://github.com/headwater-ai/headwater/issues/1312) found
//! that only the page of `check` named them.
//!
//! # Why this runs the binary
//!
//! A grep of `main.rs` for the callers of the load would hold the grep. So the
//! case table runs each verb against a scratch root whose `harvests` entry
//! names no `resolver`. A verb whose standard error then carries the refusal of
//! the graph load reached that load. A new caller of the load is caught
//! without an edit here, because its argv already sits in the table and its
//! observed answer changes.
//!
//! Three guards keep the witness honest. Every invocation that the command
//! surface lists has an argv in the table, so a new verb cannot pass by being
//! absent. Each argv states whether it reaches the load, and a disagreement
//! fails and names the argv. And some argv states that it does not, so the
//! witness cannot pass by matching every run.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The sentence the graph load writes when a `harvests` entry does not read.
const REFUSAL: &str = "the pinned export declarations did not read";

/// The `harvests` entry the scratch root declares. It names no `resolver`, so
/// the declaration does not read, and the graph load stops before it binds.
const UNREADABLE: &str = "\nharvests:\n  repo-b:\n    at: harvest/repo-b.json\n    digest: sha256:0000\n    channel: a job\n";

/// An `imports` entry, so that `headwater import` has an import to select and
/// reaches the load rather than refusing an undeclared import first.
const IMPORT: &str = "\nimports:\n  upstream:\n    at: .headwater/imports/upstream\n    digest: sha256:0000\n    channel: stable\n";

/// The files a verb's argv names. `{root}` is the scratch root.
const SITE: &str = "{root}/site";
const REPORT: &str = "{root}/report.json";
const TRANSCRIPT: &str = "{root}/transcript.jsonl";
const ARTIFACT: &str = "{root}/.headwater/packages/headwater-standard";

/// One argv of the table, and whether it reaches the graph load. `why` states
/// the reason for an argv that does not.
struct Case {
    argv: &'static [&'static str],
    loads: bool,
    why: &'static str,
}

const fn loads(argv: &'static [&'static str]) -> Case {
    Case {
        argv,
        loads: true,
        why: "",
    }
}

const fn stops(argv: &'static [&'static str], why: &'static str) -> Case {
    Case {
        argv,
        loads: false,
        why,
    }
}

const CASES: &[Case] = &[
    loads(&["check"]),
    stops(&["change"], "it reads git and no graph"),
    stops(&["gate"], "it reads a read set and no graph"),
    loads(&["conformance"]),
    stops(&["derived"], "it reads the declared producers and no graph"),
    loads(&["site", SITE]),
    stops(&["merge-driver"], "git hands it four files and no graph"),
    loads(&["route", "x"]),
    loads(&["neighbors", "x"]),
    loads(&["explain", "x"]),
    loads(&["show", "x"]),
    stops(&["query", "x"], "it implements no expression and refuses first"),
    loads(&["capture"]),
    loads(&["mcp"]),
    loads(&["new", "decision", "--title", "x"]),
    loads(&["infer"]),
    loads(&["generate"]),
    loads(&["import"]),
    loads(&["export"]),
    loads(&["sweep", "plan"]),
    loads(&["sweep", "report", REPORT]),
    loads(&["probe", "plan"]),
    loads(&["probe", "record", TRANSCRIPT]),
    loads(&["probe", "grade", TRANSCRIPT]),
    loads(&["probe", "stale"]),
    stops(&["init"], "the root is already bound"),
    stops(&["taxonomy", "validate"], "it reads the package and no graph"),
    stops(&["taxonomy", "resolve"], "it writes the lock and builds no graph"),
    loads(&["taxonomy", "audit"]),
    stops(&["taxonomy", "publish"], "it names no `--out` and refuses first"),
    stops(&["taxonomy", "vendor"], "it names no artifact and refuses first"),
    loads(&["taxonomy", "diff", ARTIFACT]),
    stops(
        &["taxonomy", "migrate", ARTIFACT],
        "the lock already takes the artifact's version, so it refuses before the load; `audit` and `diff` hold the page",
    ),
    stops(&["taxonomy", "graph"], "it reads the lock and no graph"),
    stops(&["json", "field", "x"], "it reads standard input alone"),
    stops(&["json", "count"], "it reads standard input alone"),
    stops(&["json", "quote"], "it reads standard input alone"),
    stops(&["help"], "it prints the grammar alone"),
    stops(&["completions", "bash"], "it prints a script alone"),
];

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the directory is made");
    for entry in std::fs::read_dir(from).expect("the directory reads") {
        let entry = entry.expect("the entry reads");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("the type reads").is_dir() {
            copy(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("the file copies");
        }
    }
}

fn write(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("it has a parent"))
        .expect("the directory is made");
    std::fs::write(path, text).expect("the file writes");
}

fn headwater(root: &Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(arguments)
        .arg("--root")
        .arg(root)
        .stdin(Stdio::null())
        .output()
        .expect("the binary runs")
}

/// A scratch root with this repository's taxonomy, resolved, and the files
/// the table's argv name. The unreadable `harvests` entry goes in last, after
/// the lock resolves.
fn template(at: &Path) {
    let _ = std::fs::remove_dir_all(at);
    let repository = repository();
    copy(
        &repository.join(".headwater/packages"),
        &at.join(".headwater/packages"),
    );
    copy(
        &repository.join("docs/taxonomies"),
        &at.join("docs/taxonomies"),
    );
    for file in [
        ".headwater/overlay.yml",
        ".headwater/taxonomy.yml",
        ".headwater/probe.yml",
    ] {
        std::fs::copy(repository.join(file), at.join(file)).expect("the file copies");
    }
    // The files this repository's language regime names outside the corpus
    // root. A scratch corpus that lacks them is refused for that.
    for stub in [
        "README.md",
        ".github/CONTRIBUTING.md",
        ".github/ISSUE_TEMPLATE/issue.md",
        ".github/SECURITY.md",
    ] {
        write(at, stub, "# Scratch\n\nThis file is a stub.\n");
    }
    let resolved = headwater(at, &["taxonomy", "resolve"]);
    assert_eq!(
        resolved.status.code(),
        Some(0),
        "the scratch root resolves: {}",
        String::from_utf8_lossy(&resolved.stderr)
    );
    write(at, ".headwater/imports/upstream/a.txt", "x\n");
    write(at, "site/.keep", "");
    write(at, "report.json", "{}\n");
    write(at, "transcript.jsonl", "{}\n");
    let declared =
        std::fs::read_to_string(at.join(".headwater/taxonomy.yml")).expect("the declaration reads");
    write(
        at,
        ".headwater/taxonomy.yml",
        &format!("{declared}{IMPORT}{UNREADABLE}"),
    );
}

/// The rows of the command surface: each verb, the page it links, and each
/// invocation its third column lists.
fn surface(repository: &Path) -> Vec<(String, String, Vec<Vec<String>>)> {
    let index = std::fs::read_to_string(repository.join("docs/interfaces/README.md"))
        .expect("the command surface reads");
    let mut rows = Vec::new();
    for line in index.lines() {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        // `| verb | what | typed | contract |` splits into six cells.
        if cells.len() != 6 || !cells[1].starts_with('`') {
            continue;
        }
        let verb = cells[1].trim_matches('`').to_string();
        let page = cells[4]
            .split_once("](")
            .and_then(|(_, rest)| rest.split_once(')'))
            .map(|(page, _)| page.to_string())
            .unwrap_or_else(|| panic!("the row of `{verb}` links a page: {line}"));
        let typed = cells[3]
            .split('`')
            .filter(|span| span.starts_with("headwater "))
            .map(|span| span.split_whitespace().skip(1).map(String::from).collect())
            .collect();
        rows.push((verb, page, typed));
    }
    assert!(
        rows.len() > 20,
        "the command surface lists its verbs: {} rows read",
        rows.len()
    );
    rows
}

/// The first cell of each row of the `## Files` table of one page.
fn files(page: &str) -> Vec<String> {
    let mut open = false;
    let mut out = Vec::new();
    for line in page.lines() {
        if line.starts_with("## ") {
            open = line == "## Files";
            continue;
        }
        if open && line.starts_with('|') {
            if let Some(first) = line.split('|').nth(1) {
                out.push(first.trim().to_string());
            }
        }
    }
    out
}

#[test]
fn every_verb_that_builds_a_graph_names_the_import_and_harvest_files() {
    let repository = repository();
    let scratch = std::env::temp_dir().join(format!(
        "headwater-cli-interface-files-{}",
        std::process::id()
    ));
    let base = scratch.join("template");
    template(&base);

    // Guard: every invocation the command surface lists has an argv here.
    let surface = surface(&repository);
    let mut absent = Vec::new();
    for (_, _, typed) in &surface {
        for words in typed {
            if !CASES
                .iter()
                .any(|case| case.argv.len() >= words.len() && case.argv[..words.len()] == words[..])
            {
                absent.push(format!("headwater {}", words.join(" ")));
            }
        }
    }
    assert!(
        absent.is_empty(),
        "the command surface lists invocations the case table has no argv for, so the probe never runs them: {absent:?}"
    );

    // Each argv against its own copy of the root, so a verb that writes cannot
    // move the next one's answer.
    let mut disagree = Vec::new();
    let mut reached = std::collections::BTreeSet::new();
    for (n, case) in CASES.iter().enumerate() {
        let root = scratch.join(format!("case-{n}"));
        copy(&base, &root);
        let root_text = root.to_string_lossy().into_owned();
        let argv: Vec<String> = case
            .argv
            .iter()
            .map(|word| word.replace("{root}", &root_text))
            .collect();
        let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
        let ran = headwater(&root, &argv);
        let err = String::from_utf8_lossy(&ran.stderr);
        let observed = err.contains(REFUSAL);
        if observed != case.loads {
            disagree.push(format!(
                "`headwater {}`: the table says {}, and the run {} (exit {:?}): {}",
                case.argv.join(" "),
                if case.loads {
                    "it reaches the graph load".to_string()
                } else {
                    format!("it does not, because {}", case.why)
                },
                if observed {
                    "wrote the refusal of the graph load"
                } else {
                    "did not"
                },
                ran.status.code(),
                err.lines().next().unwrap_or("")
            ));
        }
        if observed {
            reached.insert(case.argv[0]);
        }
        let _ = std::fs::remove_dir_all(&root);
    }
    let _ = std::fs::remove_dir_all(&scratch);
    assert!(
        disagree.is_empty(),
        "an argv disagrees with the table. A verb that never reached the load is a table error; a verb that newly reaches it owes its page the two rows:\n{}",
        disagree.join("\n")
    );
    assert!(
        CASES.iter().any(|case| !case.loads) && reached.len() < surface.len(),
        "the witness matched every verb, so it tells nothing apart"
    );

    // Each verb that reached the load names both files in its Files table.
    let mut owed = Vec::new();
    for (verb, page, _) in &surface {
        if !reached.contains(verb.as_str()) {
            continue;
        }
        let text = std::fs::read_to_string(repository.join("docs/interfaces").join(page))
            .unwrap_or_else(|e| panic!("the page of `{verb}` reads: {e}"));
        let rows = files(&text);
        let mut missing = Vec::new();
        if !rows.iter().any(|row| row.contains("`.headwater/imports/`")) {
            missing.push("`.headwater/imports/`");
        }
        if !rows.iter().any(|row| row.contains("`harvests.<name>.at`")) {
            missing.push("`harvests.<name>.at`");
        }
        if !missing.is_empty() {
            owed.push(format!(
                "docs/interfaces/{page} (`{verb}`) names no row for {}",
                missing.join(" or ")
            ));
        }
    }
    assert!(
        owed.is_empty(),
        "{} of {} verbs that build a graph omit a file the graph load reads:\n{}",
        owed.len(),
        reached.len(),
        owed.join("\n")
    );
}
