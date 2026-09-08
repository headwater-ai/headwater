// SPDX-License-Identifier: Apache-2.0
//! The merge driver, run against two branches that both moved one fold.
//!
//! `.gitattributes` and `.githooks/merge-regenerate` are the two halves of one
//! mechanism, and until this file nothing ran either of them. The list in
//! `.gitattributes` was held by
//! `the_generated_documents_are_declared_unmergeable` in
//! [`fixtures.rs`](fixtures.rs), which asks whether a path is named and never
//! whether naming it does anything.
//!
//! What runs here is a real `git merge` in a scratch repository, over the real
//! `.gitattributes` and the real driver. Every case differs from the one beside
//! it in one thing, because a guard with no failing arm beside it cannot tell a
//! mechanism that works from one that was never selected, and this experiment
//! has already been run once with two arms that conflicted for an unrelated
//! reason and read as "no difference" where the truth was "no evidence".
//!
//! The third case is the one to read. A custom merge driver is called when git
//! has two versions of a path to reconcile, and two branches that wrote **one
//! byte string** give it one version, which it takes at the tree level before
//! any driver runs. So the driver covers the branches that moved a fold to
//! different values and not the branches that moved it to the same value, and
//! `.gitattributes` describes the second. That is measured here rather than
//! argued.
//!
//! [HW-DR-0049](../../../../docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md)
//! is the ruling every case measures.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

/// A scratch directory keyed on the case as well as the process.
///
/// `cargo` runs the cases of one target as threads of one process, so a helper
/// keyed on the pid alone hands every case the same directory and each one
/// deletes the others' trees underneath them.
fn scratch(case: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "headwater-merge-driver-{}-{case}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the scratch directory");
    dir
}

fn git(repo: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("git {}: {e}", args.join(" ")))
}

fn git_ok(repo: &Path, args: &[&str]) -> String {
    let out = git(repo, args);
    assert!(
        out.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn write(repo: &Path, path: &str, body: &str) {
    let full = repo.join(path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).expect("the parent directory");
    }
    std::fs::write(&full, body).unwrap_or_else(|e| panic!("{}: {e}", full.display()));
}

fn read(repo: &Path, path: &str) -> String {
    std::fs::read_to_string(repo.join(path))
        .unwrap_or_else(|e| panic!("{}: {e}", repo.join(path).display()))
}

/// What a page states, in the shape the figure refresh writes: one element per
/// figure, holding a number and nothing else, one figure to a line.
///
/// Three figures rather than one, because the two shapes of this failure are
/// told apart by whether a second figure moved. The real landing page carries
/// 34 keys in 57 occurrences and every one of them is this shape.
#[derive(Clone, Copy)]
struct Figures {
    seen: usize,
    decisions: usize,
    obligations: usize,
}

fn page(f: Figures) -> String {
    let Figures {
        seen,
        decisions,
        obligations,
    } = f;
    // The prose between the figures is what keeps the three lines apart, and it
    // is the difference between a case and a coincidence. Adjacent changed
    // lines fall in one hunk, so two branches that each moved a neighboring
    // figure conflict textually with nothing declared, and a case built that
    // way reads a refusal as the driver working when the driver never ran.
    // The real landing page has whole paragraphs between its figures.
    let filler = "<p>Every number on this page came from a run of the engine.</p>\n".repeat(4);
    format!(
        "<!doctype html>\n<html><body>\n\
         <p>Headwater has seen <span data-figure=\"census.seen\">{seen}</span> documents.</p>\n\
         {filler}\
         <p><span data-figure=\"decisions.total\">{decisions}</span> decisions.</p>\n\
         {filler}\
         <p><span data-figure=\"oblig.total\">{obligations}</span> obligations.</p>\n\
         </body></html>\n"
    )
}

/// The other shape, which is the one that survives a merge: one line per
/// entity and no total. Both branches insert into it, and they insert far
/// apart, so a conflict reported on this file would mean the arms clashed
/// textually and the case measured nothing.
fn index(entries: &[&str]) -> String {
    let mut body = String::from("# The documents\n\n");
    for entry in entries {
        body.push_str(&format!("- [{entry}](./{entry}.md)\n"));
    }
    body
}

const BASE: [&str; 3] = ["alpha", "middle", "omega"];

const BASE_FIGURES: Figures = Figures {
    seen: 3,
    decisions: 1,
    obligations: 1,
};

/// Build the scratch repository and merge one branch into the other.
///
/// `attributes` is the text of the `.gitattributes` the repository gets, or
/// `None` for a repository that declares nothing. The two `Figures` are what
/// each branch writes onto the page. Everything else is identical across every
/// case, including the driver configuration, so the only thing that can account
/// for a difference in the result is one of those two arguments.
fn merge_two_branches(
    case: &str,
    attributes: Option<&str>,
    first: Figures,
    second: Figures,
) -> (PathBuf, Output) {
    let repo = scratch(case);
    let driver = repository_root().join(".githooks/merge-regenerate");
    assert!(driver.is_file(), "{} is not there", driver.display());

    git_ok(&repo, &["init", "-q", "-b", "base"]);
    git_ok(&repo, &["config", "user.name", "Fixture"]);
    git_ok(&repo, &["config", "user.email", "fixture@example.invalid"]);
    git_ok(&repo, &["config", "commit.gpgsign", "false"]);
    // Without this git falls back to an ordinary merge, the driver never runs,
    // and a declared case passes for the wrong reason. It is set in every case.
    git_ok(
        &repo,
        &[
            "config",
            "merge.headwater-regenerate.name",
            "regenerate a derived artifact",
        ],
    );
    git_ok(
        &repo,
        &[
            "config",
            "merge.headwater-regenerate.driver",
            &format!("{} %O %A %B %P", driver.display()),
        ],
    );

    if let Some(text) = attributes {
        write(&repo, ".gitattributes", text);
    }
    for name in BASE {
        write(&repo, &format!("docs/{name}.md"), &format!("# {name}\n"));
    }
    write(&repo, "docs/README.md", &index(&BASE));
    write(&repo, "site/index.html", &page(BASE_FIGURES));
    git_ok(&repo, &["add", "-A"]);
    git_ok(&repo, &["commit", "-q", "-m", "base"]);

    // Two branches, each adding one document. `aardvark` sorts before every
    // base document and `zebra` after every one of them, so their insertions
    // into the list are three lines apart and merge with no conflict.
    let arms = [
        (
            "aardvark",
            first,
            vec!["aardvark", "alpha", "middle", "omega"],
        ),
        ("zebra", second, vec!["alpha", "middle", "omega", "zebra"]),
    ];
    for (branch, figures, entries) in arms {
        git_ok(&repo, &["checkout", "-q", "base"]);
        git_ok(&repo, &["checkout", "-q", "-b", branch]);
        write(
            &repo,
            &format!("docs/{branch}.md"),
            &format!("# {branch}\n"),
        );
        write(&repo, "docs/README.md", &index(&entries));
        write(&repo, "site/index.html", &page(figures));
        git_ok(&repo, &["add", "-A"]);
        git_ok(&repo, &["commit", "-q", "-m", branch]);
    }

    git_ok(&repo, &["checkout", "-q", "zebra"]);
    let merged = git(&repo, &["merge", "--no-edit", "aardvark"]);
    (repo, merged)
}

/// The paths git left conflicted, read from the index rather than from the
/// message it printed.
fn conflicted(repo: &Path) -> Vec<String> {
    let mut paths: Vec<String> = git_ok(repo, &["diff", "--name-only", "--diff-filter=U"])
        .lines()
        .map(str::to_owned)
        .collect();
    paths.sort();
    paths.dedup();
    paths
}

/// One figure as the page now states it.
fn figure(repo: &Path, key: &str) -> String {
    let body = read(repo, "site/index.html");
    let marker = format!("data-figure=\"{key}\">");
    let (_, rest) = body
        .split_once(&marker)
        .unwrap_or_else(|| panic!("the page no longer carries {key}"));
    let (value, _) = rest.split_once('<').expect("the closing tag");
    value.to_owned()
}

/// The documents the merged tree actually holds, counted rather than claimed.
fn documents(repo: &Path) -> usize {
    std::fs::read_dir(repo.join("docs"))
        .expect("the documents")
        .filter_map(Result::ok)
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            name.ends_with(".md") && name != "README.md"
        })
        .count()
}

/// The repository's own declaration, which is what these cases are a function
/// of. Delete the `site/` block from `.gitattributes` and two of them fail.
fn attributes() -> String {
    std::fs::read_to_string(repository_root().join(".gitattributes")).expect("the attributes file")
}

/// One branch adds a decision, the other an obligation, and neither figure
/// survives a merge with nothing declared.
///
/// This is the failure the declaration cures, and it is held here rather than
/// described, because the case below it proves nothing on its own: a mechanism
/// that was never selected and a mechanism that works look the same from the
/// passing side.
///
/// The two pages differ, because a second figure moved on each branch, and git
/// merges them a line at a time. The two lines that moved on one side each
/// carry, the line that moved to `4` on both sides is read as one change
/// written twice and carries too, and the merged page states 4 documents where
/// the merged tree holds 5. Nothing is conflicted and nothing in the tree says
/// so.
#[test]
fn an_undeclared_page_merges_to_a_figure_true_of_neither_branch() {
    let a_decision = Figures {
        seen: 4,
        decisions: 2,
        obligations: 1,
    };
    let an_obligation = Figures {
        seen: 4,
        decisions: 1,
        obligations: 2,
    };
    let (repo, merged) = merge_two_branches("undeclared", None, a_decision, an_obligation);

    assert!(
        merged.status.success(),
        "the unguarded merge was expected to succeed, which is the defect: {}",
        String::from_utf8_lossy(&merged.stderr)
    );
    assert_eq!(
        conflicted(&repo),
        Vec::<String>::new(),
        "git reported a conflict with nothing declared, so this arm measured a \
         textual clash rather than the fold"
    );

    let held = documents(&repo);
    assert_eq!(
        held, 5,
        "the merged tree should hold every document of both branches"
    );
    assert_eq!(
        figure(&repo, "decisions.total"),
        "2",
        "the figure only one branch moved should carry"
    );
    assert_eq!(
        figure(&repo, "oblig.total"),
        "2",
        "the figure only the other branch moved should carry"
    );
    assert_ne!(
        figure(&repo, "census.seen"),
        held.to_string(),
        "the whole finding is that the figure both branches moved is now true \
         of neither of them and of nothing"
    );
}

/// With the repository's own declaration that merge refuses and names the cure.
///
/// The attributes are copied out of the real `.gitattributes` rather than
/// written here, so this is a function of what the repository ships.
#[test]
fn a_declared_page_refuses_the_merge_and_names_the_command() {
    let a_decision = Figures {
        seen: 4,
        decisions: 2,
        obligations: 1,
    };
    let an_obligation = Figures {
        seen: 4,
        decisions: 1,
        obligations: 2,
    };
    let (repo, merged) =
        merge_two_branches("declared", Some(&attributes()), a_decision, an_obligation);

    assert!(
        !merged.status.success(),
        "the merge succeeded, so `site/index.html` still reconciles silently: \
         the page reads {} where the tree holds {}",
        figure(&repo, "census.seen"),
        documents(&repo)
    );
    assert_eq!(
        conflicted(&repo),
        vec!["site/index.html".to_owned()],
        "the driver must refuse the page and nothing else. `docs/README.md` \
         holds one line per document and merges, and a conflict there would \
         mean the two arms clashed textually and this proved nothing"
    );

    let said = String::from_utf8_lossy(&merged.stderr);
    assert!(
        said.contains("site/index.html is a derived artifact"),
        "the driver ran but said nothing a reader could act on: {said}"
    );
    assert!(
        said.contains("tools/refresh-figures.sh"),
        "the refusal must name the command that rebuilds the page, because git \
         writes no conflict marker for a custom driver and the file left in the \
         tree says nothing: {said}"
    );
}

/// The hole. A merge driver is not called when both branches wrote one byte
/// string, and that is the case `.gitattributes` describes.
///
/// Git reconciles a path by content only when it has two contents. Two branches
/// that each add a decision measure the same 34 figures and write one identical
/// page, so the merge has one blob for the path, takes it at the tree level,
/// and no driver of any kind is consulted. The declaration is not wrong and it
/// is not idle — the case above is real and common, because two branches adding
/// documents of different kinds move different figures — but it is narrower than
/// the paragraph in `.gitattributes` claims, and the same limit holds for every
/// path in that block rather than only for these three.
///
/// This case asserts the behavior as it is, so that a change which closes the
/// hole fails here and is read rather than merged. What closes it is a gate on
/// the merged state: CI already runs `sh tools/refresh-figures.sh --check`
/// against `refs/pull/N/merge`, which is the union, and a local merge runs
/// `pre-merge-commit`, a hook this repository does not have.
/// [#717](https://github.com/headwater-ai/headwater/issues/717) owns that, and
/// it owns it for all eighteen declared paths rather than only these three.
#[test]
fn a_driver_is_not_called_when_both_branches_wrote_one_byte_string() {
    let same = Figures {
        seen: 4,
        decisions: 2,
        obligations: 1,
    };
    let (repo, merged) = merge_two_branches("identical", Some(&attributes()), same, same);

    assert!(
        merged.status.success(),
        "the merge refused, so git now calls a driver for a path both branches \
         wrote alike. That closes a hole this case records as open, and the \
         paragraph in `.gitattributes` and the note on this test both have to \
         move: {}",
        String::from_utf8_lossy(&merged.stderr)
    );
    assert_eq!(
        conflicted(&repo),
        Vec::<String>::new(),
        "nothing is conflicted, which is the point"
    );
    assert_eq!(
        figure(&repo, "census.seen"),
        "4",
        "the value both branches wrote"
    );
    assert_eq!(
        documents(&repo),
        5,
        "and the tree the merge produced, which no page states"
    );
}
