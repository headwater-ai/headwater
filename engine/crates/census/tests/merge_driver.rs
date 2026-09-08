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
//! `.gitattributes` and the real driver, in two arms that differ in one thing.
//! The unguarded arm is the defect: the merge succeeds at exit 0 and writes a
//! fold that is true of neither branch, with no conflict marker anywhere in the
//! tree. The guarded arm is the cure. A guard with no failing arm beside it
//! cannot tell a mechanism that works from one that was never selected, and
//! this experiment has already been run once with two arms that conflicted for
//! an unrelated reason and read as "no difference" where the truth was "no
//! evidence".
//!
//! [HW-DR-0049](../../../../docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md)
//! is the ruling both arms measure.

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
/// keyed on the pid alone hands both arms the same directory and each one
/// deletes the other's tree underneath it.
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

/// The page, in the shape the figure refresh writes: one element per figure,
/// holding a number and nothing else.
fn page(seen: usize) -> String {
    format!(
        "<!doctype html>\n<html><body>\n\
         <p>Headwater has seen <span data-figure=\"census.seen\">{seen}</span> documents.</p>\n\
         </body></html>\n"
    )
}

/// The other shape, which is the one that survives a merge: one line per
/// entity and no total. Both branches insert into it, and they insert far
/// apart, so a conflict reported on this file would mean the arms clashed
/// textually and the experiment measured nothing.
fn index(entries: &[&str]) -> String {
    let mut body = String::from("# The documents\n\n");
    for entry in entries {
        body.push_str(&format!("- [{entry}](./{entry}.md)\n"));
    }
    body
}

const BASE: [&str; 3] = ["alpha", "middle", "omega"];

/// Build the scratch repository and merge one arm into the other.
///
/// `attributes` is the text of the `.gitattributes` the repository gets, or
/// `None` for a repository that declares nothing. Everything else is identical
/// between the two arms, including the driver configuration, so the only thing
/// that can account for a difference in the result is the declaration.
fn merge_two_branches(case: &str, attributes: Option<&str>) -> (PathBuf, Output) {
    let repo = scratch(case);
    let driver = repository_root().join(".githooks/merge-regenerate");
    assert!(driver.is_file(), "{} is not there", driver.display());

    git_ok(&repo, &["init", "-q", "-b", "base"]);
    git_ok(&repo, &["config", "user.name", "Fixture"]);
    git_ok(&repo, &["config", "user.email", "fixture@example.invalid"]);
    git_ok(&repo, &["config", "commit.gpgsign", "false"]);
    // Without this git falls back to an ordinary merge, the driver never runs,
    // and the guarded arm passes for the wrong reason. It is set in both arms.
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
    write(&repo, "site/index.html", &page(BASE.len()));
    git_ok(&repo, &["add", "-A"]);
    git_ok(&repo, &["commit", "-q", "-m", "base"]);

    // Two branches, each adding one document, each rewriting the same fold to
    // the same new value. `aardvark` sorts before every base document and
    // `zebra` after every one of them, so their insertions into the list are
    // three lines apart and merge with no conflict.
    for (branch, name, entries) in [
        ("aardvark", "aardvark", vec!["aardvark", "alpha", "middle", "omega"]),
        ("zebra", "zebra", vec!["alpha", "middle", "omega", "zebra"]),
    ] {
        git_ok(&repo, &["checkout", "-q", "base"]);
        git_ok(&repo, &["checkout", "-q", "-b", branch]);
        write(&repo, &format!("docs/{name}.md"), &format!("# {name}\n"));
        write(&repo, "docs/README.md", &index(&entries));
        write(&repo, "site/index.html", &page(BASE.len() + 1));
        git_ok(&repo, &["add", "-A"]);
        git_ok(&repo, &["commit", "-q", "-m", branch]);
    }

    git_ok(&repo, &["checkout", "-q", "zebra"]);
    let merged = git(&repo, &["merge", "--no-edit", "aardvark"]);
    (repo, merged)
}

/// The paths git left conflicted, from the index rather than from the message.
fn conflicted(repo: &Path) -> Vec<String> {
    let mut paths: Vec<String> = git_ok(repo, &["diff", "--name-only", "--diff-filter=U"])
        .lines()
        .map(str::to_owned)
        .collect();
    paths.sort();
    paths.dedup();
    paths
}

/// The number the page states, read out of the element the refresh writes.
fn figure(repo: &Path) -> String {
    let body = read(repo, "site/index.html");
    let (_, rest) = body
        .split_once("data-figure=\"census.seen\">")
        .expect("the page still carries the figure");
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

/// With no declaration the merge succeeds and states a number true of nothing.
///
/// This is the failure the driver exists for, and it is held here rather than
/// described, because the guarded case below proves nothing on its own. A
/// mechanism that was never selected and a mechanism that works look the same
/// from the passing side.
#[test]
fn an_undeclared_page_merges_to_a_figure_true_of_neither_branch() {
    let (repo, merged) = merge_two_branches("undeclared", None);

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
    assert_eq!(held, 5, "the merged tree should hold every document of both branches");
    assert_eq!(
        figure(&repo),
        "4",
        "the merged page should carry the value both branches wrote"
    );
    assert_ne!(
        figure(&repo),
        held.to_string(),
        "the whole finding is that the merged page disagrees with the merged tree"
    );
}

/// With the repository's own declaration the merge refuses and names the cure.
///
/// The attributes are copied out of the real `.gitattributes` rather than
/// written here, so this case is a function of the declaration the repository
/// ships. Delete the `site/` block and this fails.
#[test]
fn a_declared_page_refuses_the_merge_and_names_the_command() {
    let attributes = std::fs::read_to_string(repository_root().join(".gitattributes"))
        .expect("the attributes file");
    let (repo, merged) = merge_two_branches("declared", Some(&attributes));

    assert!(
        !merged.status.success(),
        "the merge succeeded, so `site/index.html` still reconciles silently: \
         the page reads {} where the tree holds {}",
        figure(&repo),
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
         writes no conflict marker for a custom driver and the file in the tree \
         says nothing: {said}"
    );
}
