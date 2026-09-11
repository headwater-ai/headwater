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
//! The cases from `a_merge_of_one_byte_string_is_refused_by_the_merged_tree_gate`
//! down are the cure, which is a gate on the merged tree rather than a driver
//! over the two sides of it: `.githooks/commit-msg`, running
//! `.githooks/merged-fold-check` whenever a merge is in progress. They are the
//! same experiment with the hook path wired in, so the only thing that can
//! account for the difference is the hook.
//!
//! [HW-DR-0049](../../../../docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md)
//! is the ruling every case measures.

use std::os::unix::fs::PermissionsExt;
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

/// What the scratch repository is given beyond the merge driver.
///
/// `None` is every case written before the gate existed, and it is what makes
/// those cases still mean what they meant: nothing runs after the merge there.
/// The other three wire `core.hooksPath` to this repository's own `.githooks`,
/// so `commit-msg` and the body it runs are the real files rather than copies.
///
/// The producers are planted rather than real, and that is the point rather than
/// a shortcut. A scratch repository is not this corpus: `headwater generate
/// --check` there has no corpus to read, and every gate in this repository fails
/// open on a missing engine, so a case that asserted only "the merge failed"
/// could not tell a refusal from a crash or from a hook that never ran. Each
/// planted producer appends its arguments to `producer-calls` in the repository,
/// so a case asserts which producer ran as well as what came of it.
#[derive(Clone, Copy, PartialEq)]
enum Gate {
    /// No `core.hooksPath`. Nothing runs after the merge.
    None,
    /// The hooks, and a `headwater generate --check` that disagrees with the
    /// merged tree.
    ProducersRefuse,
    /// The hooks, and producers that all agree.
    ProducersAgree,
    /// The hooks, and no engine anywhere under `engine/target/`. This is the
    /// fail-open limit, pinned rather than left to be discovered.
    NoEngine,
}

fn executable(repo: &Path, path: &str, body: &str) {
    write(repo, path, body);
    let full = repo.join(path);
    let mut mode = std::fs::metadata(&full)
        .expect("the planted file")
        .permissions();
    mode.set_mode(0o755);
    std::fs::set_permissions(&full, mode).expect("the executable bit");
}

/// Plant the engine and the four scripts the gates run, each of them recording
/// that it ran.
///
/// `tools/site/refresh-crawler-files.sh`, `tools/site/refresh-site-tokens.sh` and
/// `tools/site/render-tutorial.py` are here for a reason worth stating.
/// `.githooks/pre-commit` runs all three, and it runs before `commit-msg` on a
/// `git commit` that finishes a merge. Absent, they exit 127 (the two shell
/// scripts) or make `pre-commit`'s own `python3 ... --check` fail with no such
/// file (the Python one), and `pre-commit` refuses the commit while reporting a
/// stale derived file — a refusal that is real, is about something else, and
/// would let the case below pass with the gate under test never reached.
fn plant_producers(repo: &Path, generate_exit: i32) {
    let calls = repo.join("producer-calls");
    let calls = calls.display().to_string();
    executable(
        repo,
        "engine/target/dev-release/headwater",
        &format!(
            "#!/bin/sh\n\
             printf '%s\\n' \"$*\" >> \"{calls}\"\n\
             case \"$1\" in\n\
             generate)\n\
             echo 'the planted producer: this tree is not what a run produces' >&2\n\
             exit {generate_exit} ;;\n\
             taxonomy)\n\
             echo 'the planted producer: the lock is current' >&2\n\
             exit 0 ;;\n\
             esac\n\
             exit 0\n"
        ),
    );
    for script in [
        "tools/site/refresh-figures.sh",
        "tools/site/refresh-crawler-files.sh",
        "tools/site/refresh-site-tokens.sh",
    ] {
        executable(
            repo,
            script,
            &format!("#!/bin/sh\nprintf '%s\\n' \"{script} $*\" >> \"{calls}\"\nexit 0\n"),
        );
    }
    // `render-tutorial.py` is called as `python3 tools/site/render-tutorial.py
    // --check` rather than executed directly, so its planted body has to be
    // valid Python and not the shell the three scripts above take.
    executable(
        repo,
        "tools/site/render-tutorial.py",
        &format!(
            "#!/usr/bin/env python3\n\
             import sys\n\
             with open({calls:?}, 'a') as f:\n\
             \x20\x20\x20\x20f.write('tools/site/render-tutorial.py ' + ' '.join(sys.argv[1:]) + '\\n')\n\
             sys.exit(0)\n"
        ),
    );
}

/// Every producer invocation the gates made, in order.
fn calls(repo: &Path) -> Vec<String> {
    std::fs::read_to_string(repo.join("producer-calls"))
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect()
}

fn merge_head(repo: &Path) -> bool {
    repo.join(".git/MERGE_HEAD").is_file()
}

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
    gate: Gate,
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

    // After both arms are committed, so that nothing planted here is part of
    // either branch and nothing about the merge is a function of it. The gate
    // reads the working tree, which is where these are.
    if gate != Gate::None {
        let hooks = repository_root().join(".githooks");
        assert!(
            hooks.join("commit-msg").is_file() && hooks.join("merged-fold-check").is_file(),
            "the hook under test and the body it runs are not in {}",
            hooks.display()
        );
        // An absolute path, because the hooks are this repository's and the
        // scratch tree is not this repository. Measured on git 2.53.0: a
        // `core.hooksPath` outside the working tree is honored, and without
        // this line the scratch repository runs `.git/hooks`, where there is
        // nothing, and every case below would pass with no hook run at all.
        git_ok(
            &repo,
            &["config", "core.hooksPath", &hooks.display().to_string()],
        );
        if gate != Gate::NoEngine {
            plant_producers(&repo, if gate == Gate::ProducersRefuse { 1 } else { 0 });
        }
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
    let (repo, merged) =
        merge_two_branches("undeclared", None, Gate::None, a_decision, an_obligation);

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
    let (repo, merged) = merge_two_branches(
        "declared",
        Some(&attributes()),
        Gate::None,
        a_decision,
        an_obligation,
    );

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
        said.contains("tools/site/refresh-figures.sh"),
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
/// This case holds the *driver's* reach and no longer the repository's. It is
/// run with no hook path, so nothing follows the merge, and what it measures is
/// that a driver alone still does not see this merge. That is a property of git
/// rather than a defect now: the cure is not a wider driver, and the cases below
/// are the cure. Delete the gate and this case still passes, which is why the
/// cases below exist and why this one keeps saying what it says.
///
/// [#717](https://github.com/headwater-ai/headwater/issues/717) closed the hole
/// with `.githooks/commit-msg`, which runs `.githooks/merged-fold-check` over
/// the merged tree.
#[test]
fn a_driver_is_not_called_when_both_branches_wrote_one_byte_string() {
    let same = Figures {
        seen: 4,
        decisions: 2,
        obligations: 1,
    };
    let (repo, merged) =
        merge_two_branches("identical", Some(&attributes()), Gate::None, same, same);

    assert!(
        merged.status.success(),
        "the merge refused with no hook path set, so something other than the \
         gate refused it and every case below measures that thing instead: {}",
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

/// The cure. The same merge as the case above, with `core.hooksPath` set to this
/// repository's `.githooks`, is refused — and the refusal came from a producer
/// that ran rather than from anything else that can make `git merge` exit 1.
///
/// Three assertions rather than one, because "the merge failed" is the weakest
/// evidence in this file. The producer's own words have to be in what git
/// printed, its call has to be in `producer-calls`, and `MERGE_HEAD` has to be
/// there, which is what says git stopped at the commit rather than at the
/// content.
#[test]
fn a_merge_of_one_byte_string_is_refused_by_the_merged_tree_gate() {
    let same = Figures {
        seen: 4,
        decisions: 2,
        obligations: 1,
    };
    let (repo, merged) = merge_two_branches(
        "gated-refuse",
        Some(&attributes()),
        Gate::ProducersRefuse,
        same,
        same,
    );

    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&merged.stdout),
        String::from_utf8_lossy(&merged.stderr)
    );
    assert!(
        !merged.status.success(),
        "the merge succeeded, so the gate did not stop a tree that no derived \
         artifact in it describes: the page reads {} where the tree holds {}. \
         What ran: {:?}. What it said: {said}",
        figure(&repo, "census.seen"),
        documents(&repo),
        calls(&repo)
    );
    assert!(
        calls(&repo)
            .iter()
            .any(|c| c.starts_with("generate --check")),
        "the merge failed and no producer ran, so this measured something other \
         than the gate — a missing engine, a missing script, or a hook that was \
         never reached. What ran: {:?}. What it said: {said}",
        calls(&repo)
    );
    assert!(
        said.contains("this tree is not what a run produces"),
        "the refusal did not carry the producer's own words, so a reader is told \
         a merge failed and not which artifact disagrees: {said}"
    );
    assert!(
        said.contains("headwater generate"),
        "the refusal must name the command that rebuilds the artifact, because \
         the merged tree carries no marker of any kind: {said}"
    );
    assert!(
        merge_head(&repo),
        "git left no MERGE_HEAD, so the merge is not resumable and the remedy \
         the refusal prints — regenerate here and commit — cannot be followed"
    );
    assert_eq!(
        documents(&repo),
        5,
        "the merged content is in the working tree, which is what makes the \
         remedy one regenerate rather than a re-merge"
    );
}

/// The other arm, and it is what keeps the gate from being one that refuses
/// everything. The same wiring, with producers that agree, and the merge lands.
///
/// It also counts rather than looks. One hook runs the producers, so one merge
/// costs one run of each. A second hook on the `pre-merge-commit` position was
/// written first and deleted on this measurement: it ran them a second time for
/// every merge that passed, because `MERGE_HEAD` does not exist yet when
/// `pre-merge-commit` runs, so the two positions had no shared name for the
/// merge and no cheap way to hand off. The failure this count catches is a gate
/// that works and costs double.
#[test]
fn the_merged_tree_gate_is_silent_when_every_producer_agrees() {
    let same = Figures {
        seen: 4,
        decisions: 2,
        obligations: 1,
    };
    let (repo, merged) = merge_two_branches(
        "gated-agree",
        Some(&attributes()),
        Gate::ProducersAgree,
        same,
        same,
    );

    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&merged.stdout),
        String::from_utf8_lossy(&merged.stderr)
    );
    assert!(
        merged.status.success(),
        "the gate refused a merge that every producer agreed with, so the first \
         honest merge on `main` stops: {said}"
    );
    assert!(!merge_head(&repo), "the merge did not complete: {said}");

    let ran = calls(&repo);
    assert_eq!(
        ran.iter()
            .filter(|c| c.starts_with("generate --check"))
            .count(),
        1,
        "the producers should run once for one merge. Zero means `pre-merge-commit` \
         never ran and this case proves nothing; two means the stamp \
         `pre-merge-commit` leaves for `commit-msg` is not read. What ran: {ran:?}"
    );
    assert!(
        ran.iter()
            .any(|c| c.starts_with("taxonomy resolve --check")),
        "the lock's producer did not run: {ran:?}"
    );
    assert!(
        ran.iter()
            .any(|c| c.starts_with("tools/site/refresh-figures.sh")),
        "the figures' producer did not run, which is the one that writes the two \
         pages this whole file is about: {ran:?}"
    );
}

/// Finishing a refused merge by hand is refused too, which is the half a
/// `pre-merge-commit` alone does not have.
///
/// On a refusal git prints "use 'git commit' to complete the merge", and a gate
/// on the `pre-merge-commit` position is not run again by that command — it runs
/// `pre-commit`, `prepare-commit-msg` and `commit-msg`, and `.githooks/pre-commit`
/// runs none of the producers a fold answers to. This is the case that says the
/// command git itself suggests does not walk past the gate.
#[test]
fn completing_a_refused_merge_by_hand_is_refused_by_the_same_gate() {
    let same = Figures {
        seen: 4,
        decisions: 2,
        obligations: 1,
    };
    let (repo, merged) = merge_two_branches(
        "gated-by-hand",
        Some(&attributes()),
        Gate::ProducersRefuse,
        same,
        same,
    );
    assert!(
        !merged.status.success() && merge_head(&repo),
        "the merge was not left in progress, so there is nothing here to finish \
         by hand and this case measures nothing"
    );
    let before = calls(&repo).len();

    let commit = git(&repo, &["commit", "--no-edit"]);
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&commit.stdout),
        String::from_utf8_lossy(&commit.stderr)
    );
    let after = calls(&repo);
    assert!(
        after.len() > before,
        "no producer ran on the `git commit` that completes the merge, so \
         whatever happened there was not this gate. Before {before}, after \
         {after:?}. What it said: {said}"
    );
    assert!(
        !commit.status.success(),
        "`git commit` completed the merge that the gate had just refused, which \
         is the whole cure bypassed by the one command git prints. What ran: \
         {after:?}. What it said: {said}"
    );
    assert!(
        said.contains("this tree is not what a run produces"),
        "the refusal did not carry the producer's own words: {said}"
    );
    assert!(
        merge_head(&repo),
        "the merge is no longer in progress, so the commit landed after all"
    );
}

/// The limit, pinned rather than left to be discovered. A clone with no built
/// engine cannot run a producer, so the gate prints one line and lets the merge
/// through, exactly as `.githooks/pre-commit` does for the same reason.
///
/// It is here because the alternative reading of a passing merge is "the gate
/// held", and on a fresh worktree that reading is wrong. The line the hook
/// prints is the whole difference, so this case asserts the line.
#[test]
fn a_clone_with_no_built_engine_is_not_gated_and_says_so() {
    let same = Figures {
        seen: 4,
        decisions: 2,
        obligations: 1,
    };
    let (repo, merged) = merge_two_branches(
        "gated-no-engine",
        Some(&attributes()),
        Gate::NoEngine,
        same,
        same,
    );

    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&merged.stdout),
        String::from_utf8_lossy(&merged.stderr)
    );
    assert!(
        merged.status.success(),
        "the gate refused with no engine to refuse on, which is a repository \
         nobody can merge in: {said}"
    );
    assert!(
        said.contains("no built engine"),
        "the merge went through unchecked and said nothing, so the silence reads \
         as a pass: {said}"
    );
    assert_eq!(
        figure(&repo, "census.seen"),
        "4",
        "and the figure is still the one true of neither branch, which is what \
         the printed line is warning about"
    );
}
