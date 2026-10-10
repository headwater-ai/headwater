// SPDX-License-Identifier: Apache-2.0
//! Two branches merged over a scratch copy of this repository, and every
//! member of the derived population checked against what its shape promises.
//!
//! # What this target holds that no other one does
//!
//! `merge_driver.rs` beside this file and `engine/crates/census/tests/merge_driver.rs`
//! merge named paths: the lock, the descriptor, one shelf index, the capture-cost
//! store, the recorded lock fixture. `engine/crates/census/tests/fixtures.rs`
//! holds the whole population against `.gitattributes`, but statically: it asks
//! whether a path carries the attribute its shape takes, and never whether a
//! merge under that attribute does what the attribute promises. The 1058-a
//! measurement recorded in `.gitattributes` asked that, once and by hand, over
//! seven pairs ([#1112](https://github.com/headwater-ai/headwater/issues/1112)).
//!
//! This target asks it on every run, over every member that
//! [`headwater_census::derived::population`] computes for the scratch copy. It
//! names no member and states no count of them, because a count stated by hand
//! is the defect of #676. A member that a later change adds is provoked and
//! checked on the day it is added.
//!
//! # The pair
//!
//! Branch `a` and branch `b` each, from one base:
//!
//! - edit the title of one document on every shelf that a generated shelf index
//!   indexes: `a` the first row of the index and `b` the last, so that the two
//!   edits are two hunks wherever the shelf holds more than two rows. A shelf of
//!   one row is edited by `a` alone, because two edits to one document conflict
//!   in the document and in every fold of it, which measures nothing. A title is
//!   what moves both the shelf index and the site navigation;
//! - add one decision record. In the main case the two files are named for the
//!   two ends of the shelf, so the shelf index, the navigation and the list of
//!   open questions each take two hunks and merge. In the arms the two files
//!   have adjacent sequence numbers past the last record, so each of those
//!   listings takes one hunk on both sides and conflicts as text, which is the
//!   record shape's ordinary conflict. Arm B turns that conflict into the
//!   silent defect;
//! - add one retired term to `.headwater/overlay.yml`, at the two ends of the
//!   list, which moves the taxonomy lock to a different value on each side;
//! - append one reading to every append store;
//! - insert one line into every decomposed recorded fixture, at its two ends;
//! - append one different line to every recorded fold.
//!
//! Each branch then runs `headwater taxonomy resolve` and `headwater generate`
//! and commits, and `b` merges `a` with `.gitattributes` exactly as the base
//! commits it and no driver configured. That is the merge a forge runs
//! ([HW-DR-0049](../../../../docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md)).
//!
//! # What each member must show
//!
//! The expectation is the treatment the member's **shape** takes, which is the
//! "What a merge does" column of the shape table in
//! [`what-a-check-can-know.md`](../../../../docs/evaluations/what-a-check-can-know.md#the-shapes-a-record-takes),
//! with the treatment applied. The population is computed over the base, before
//! an arm sets its override, and the main case asserts that the base agrees with
//! its own attributes, so the shape and the committed attribute name one
//! treatment for every member.
//!
//! - A fold (`-merge`) conflicts, holds the current side's bytes and carries no
//!   conflict marker.
//! - An append store (`merge=union`) merges clean with the readings of both sides.
//! - A record per entity (no attribute) either conflicts, which is the correct
//!   report, or merges clean with the lines of both sides. Where it merges clean
//!   and a producer of the scratch tree writes it, it must be the bytes that
//!   producer writes over the merged tree. The check resolves every conflict by
//!   taking the current side, runs both producers, and names every clean member
//!   they change.
//!
//! # The arms
//!
//! Each arm is a further merge of a pair that has already merged once with the
//! committed attributes and found nothing, and it differs from that merge in
//! one line of the clone's own `info/attributes`, which wins over every
//! `.gitattributes`. Each asserts the exact set of paths and findings that line
//! should cause, for the reason `census/tests/merge_driver.rs` gives at its
//! head: a case that is red for an unrelated reason measures nothing. A merge
//! after the first costs one `headwater generate` where a new pair costs two
//! more, and one `generate` of the debug binary over this repository took 16
//! seconds on 2026-10-10.
//!
//! Over the main case's pair:
//!
//! - Arm A sets the taxonomy lock's merge attribute back to unspecified, as if
//!   its `-merge` line left `.gitattributes`. The lock takes conflict markers.
//! - Arm C gives the lock `merge=union`. The lock merges clean, to both sides'
//!   lines, and the resolver rewrites it.
//!
//! Over a pair whose two decisions are adjacent, after a control merge in which
//! the decision shelf index conflicts:
//!
//! - Arm B gives the decision shelf index `merge=union`. Union keeps "ours, then
//!   theirs" for the two adjacent rows, out of the fixed order, so the merge is
//!   clean and `headwater generate` rewrites it. This is the silent one, and it
//!   is what #1112 exists to catch: a treatment that stops holding, found by the
//!   run over the whole population and not by a case written for that file.
//! - Arm D takes `merge=union` off one append store, which then conflicts.
//! - Arm E gives one decomposed recorded fixture a driver that keeps the current
//!   side and exits 0. The merge is clean and drops the other branch's line.
//! - Arm F gives the same fixture a driver that keeps the other side, so the
//!   merge drops the current branch's line. E and F together hold that the
//!   check reads the lines of both branches.
//!
//! The main case also asserts that each branch on its own moves every member
//! the pair reaches, so the provocation cannot become one-sided. A shelf index
//! with one editable row is the exception: branch `a` alone edits it, because
//! two edits to one document conflict in the document and in every fold of it.
//! The test counts those indexes and prints the count.

mod common;
use common::{fence, outside_base, repository};
use headwater_census::derived::{self, Member, Population, Producer, Shape, Treatment};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The shelf whose index Arm B gives `merge=union`, and where both branches
/// add a decision with adjacent sequence numbers.
const DECISIONS: &str = "docs/decisions";

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_headwater"))
}

/// Remove `path` and everything under it, and try again until it is gone, for
/// the reason `merge_driver.rs` gives on its own copy of this function.
fn remove_all(path: &Path) {
    for _ in 0..50 {
        let _ = std::fs::remove_dir_all(path);
        if !path.exists() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

/// A scratch copy of this repository, removed when it is dropped: on success,
/// on a panic and on an early return. CI's "The engine suite leaves nothing in
/// its temporary directory" step fails on anything left.
struct Scratch {
    at: PathBuf,
}

impl Drop for Scratch {
    fn drop(&mut self) {
        remove_all(&self.at);
    }
}

impl Scratch {
    /// Every tracked file and every untracked file that is not ignored, as
    /// `.githooks/fixtures.sh` copies them, committed as `main` in a fresh
    /// repository with its own isolated configuration. `None` where no scratch
    /// base lies outside every git repository.
    fn of_this_repository(label: &str) -> Option<Scratch> {
        let base = outside_base()?;
        let copy = Scratch {
            at: base.join(format!(
                "headwater-cli-merge-provocation-{}-{label}",
                std::process::id()
            )),
        };
        remove_all(&copy.at);
        std::fs::create_dir_all(&copy.at).expect("the scratch directory is made");

        let source = repository();
        let listed = Command::new("git")
            .args(["ls-files", "-co", "--exclude-standard", "-z"])
            .current_dir(&source)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .expect("git runs");
        assert!(
            listed.status.success(),
            "`git ls-files` lists this repository:\n{}",
            String::from_utf8_lossy(&listed.stderr)
        );
        for relative in listed.stdout.split(|byte| *byte == 0) {
            if relative.is_empty() {
                continue;
            }
            let relative = String::from_utf8_lossy(relative).into_owned();
            copy_one(&source.join(&relative), &copy.at.join(&relative));
        }

        let init = copy.git_output(&["init", "-q", "-b", "main", "."]);
        assert!(init.status.success(), "`git init` succeeds");
        copy.git(&["config", "user.email", "provocation@example.com"]);
        copy.git(&["config", "user.name", "The merge provocation"]);
        copy.git(&["config", "maintenance.auto", "false"]);
        copy.git(&["config", "gc.auto", "0"]);
        copy.git(&["config", "gc.autoDetach", "false"]);
        Some(copy)
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.at.join(relative)
    }

    fn read(&self, relative: &str) -> String {
        std::fs::read_to_string(self.path(relative))
            .unwrap_or_else(|error| panic!("{relative} reads: {error}"))
    }

    /// The bytes of `relative`, or `None` where the merge left no file there.
    fn bytes(&self, relative: &str) -> Option<Vec<u8>> {
        std::fs::read(self.path(relative)).ok()
    }

    fn write(&self, relative: &str, body: &str) {
        self.write_at(&self.path(relative), body);
    }

    fn write_at(&self, path: &Path, body: &str) {
        std::fs::create_dir_all(path.parent().expect("a file has a directory"))
            .expect("the directory is made");
        std::fs::write(path, body).expect("the file writes");
    }

    /// Git, isolated from the configuration of the machine running the suite
    /// and fenced at the scratch root.
    fn git_output(&self, args: &[&str]) -> Output {
        let mut command = Command::new("git");
        command
            .args(args)
            .current_dir(&self.at)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env_remove("GIT_DIR")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_WORK_TREE");
        fence(&mut command, &self.at);
        command.output().expect("git runs")
    }

    fn git(&self, args: &[&str]) -> String {
        let output = self.git_output(args);
        assert!(
            output.status.success(),
            "`git {}` succeeds:\n{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).into_owned()
    }

    fn headwater_ok(&self, args: &[&str]) {
        let mut command = Command::new(binary());
        command
            .args(args)
            .arg("--root")
            .arg(&self.at)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env_remove("GIT_DIR")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_WORK_TREE");
        fence(&mut command, &self.at);
        let output = command.output().expect("the binary runs");
        assert_eq!(
            output.status.code(),
            Some(0),
            "`headwater {}` exits 0:\n{}{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn lines(&self, args: &[&str]) -> BTreeSet<String> {
        self.git(args).lines().map(str::to_string).collect()
    }
}

/// Copy one file, or recreate one symbolic link, making its directory first.
/// A path git lists that is not on disk, a deletion not yet committed, is
/// passed over, as `tar` in `.githooks/fixtures.sh` would refuse it.
fn copy_one(from: &Path, to: &Path) {
    let Ok(meta) = std::fs::symlink_metadata(from) else {
        return;
    };
    std::fs::create_dir_all(to.parent().expect("a file has a directory"))
        .expect("the directory is made");
    #[cfg(unix)]
    if meta.file_type().is_symlink() {
        let target = std::fs::read_link(from).expect("the link reads");
        std::os::unix::fs::symlink(target, to).expect("the link is made");
        return;
    }
    if meta.is_file() {
        std::fs::copy(from, to).unwrap_or_else(|error| {
            panic!("{} copies: {error}", from.display());
        });
    }
}

/// The kind the generated-file marker of `body` names, such as `shelf_index`,
/// or `None` where the file carries no marker. A page with front matter
/// carries its marker below it, so the opening lines are read, not the first.
fn marker(body: &str) -> Option<String> {
    let line = body
        .lines()
        .take(40)
        .find(|line| line.contains("headwater:generated"))?;
    let after = &line[line.find("headwater:generated")? + "headwater:generated".len()..];
    let kind: String = after
        .trim_start_matches(|c: char| !c.is_ascii_alphanumeric())
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    (!kind.is_empty()).then_some(kind)
}

/// The link target of every row of a shelf index, in the order it lists them.
fn rows(index: &str) -> Vec<String> {
    index
        .lines()
        .filter(|line| line.starts_with("- ["))
        .filter_map(|line| {
            let open = line.find("](")? + 2;
            let close = open + line[open..].find(')')?;
            Some(line[open..close].to_string())
        })
        .collect()
}

/// `body` with ` (edited on branch <side>)` added to the end of the title
/// in its front matter, inside the quotes where the title is quoted.
fn retitled(body: &str, side: &str) -> Option<String> {
    let suffix = format!(" (edited on branch {side})");
    let mut out = String::with_capacity(body.len() + suffix.len());
    let mut fences = 0;
    let mut done = false;
    for line in body.split_inclusive('\n') {
        if line.trim_end() == "---" {
            fences += 1;
        }
        let bare = line.trim_end_matches('\n');
        match bare.strip_prefix("title: ") {
            Some(value) if fences == 1 && !done && !value.is_empty() => {
                let edited = match value.chars().last() {
                    Some(quote @ ('"' | '\'')) if value.len() > 1 && value.starts_with(quote) => {
                        format!("{}{suffix}{quote}", &value[..value.len() - 1])
                    }
                    _ if value.starts_with(['>', '|', '"', '\'']) => return None,
                    _ => format!("{value}{suffix}"),
                };
                out.push_str("title: ");
                out.push_str(&edited);
                out.push('\n');
                done = true;
            }
            _ => out.push_str(line),
        }
    }
    done.then_some(out)
}

/// Where the two branches put the decision record each one adds.
#[derive(Clone, Copy)]
enum Decisions {
    /// At the two ends of the shelf, so that every listing of it, ordered by
    /// sequence or by path, takes two hunks and merges.
    Apart,
    /// With adjacent file names past the last record, so that the shelf index
    /// and the navigation each take one hunk, which is the record shape's
    /// ordinary conflict under a text merge.
    Adjacent,
}

/// Two committed branches over one scratch copy, and what the base computed.
struct Pair {
    copy: Scratch,
    /// Every member of the population, as the base computed it.
    population: Population,
    /// Members either branch moved.
    moved: BTreeSet<String>,
    /// Members each branch moved, `a` then `b`.
    moved_by: [BTreeSet<String>; 2],
    /// Shelf indexes no branch could edit a row of, because every row links a
    /// producer output.
    unprovokable: BTreeSet<String>,
    /// Shelf indexes with one editable row, which branch `a` alone edits.
    one_sided: BTreeSet<String>,
}

/// How a member's merge broke what its shape promises.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Finding {
    /// A fold that both branches moved merged without a conflict.
    FoldMergedClean,
    /// A fold carries conflict markers.
    FoldMarkers,
    /// A fold does not hold the current side's bytes.
    FoldNotCurrentSide,
    /// An append store conflicted.
    StoreConflicted,
    /// A path merged clean without a line one branch added.
    LineMissing,
    /// A path merged clean to bytes its producer rewrites over the merged tree.
    ProducerRewrites,
}

/// What one merge of the pair showed, member by member.
struct Merged {
    /// Paths the merge left conflicted.
    conflicted: BTreeSet<String>,
    /// Each member whose merge is not what its shape promises, why, and a
    /// detail for the reader.
    findings: Vec<(String, Finding, String)>,
}

impl Merged {
    /// Each path named, with each way it broke.
    fn found(&self) -> BTreeSet<(&str, Finding)> {
        self.findings
            .iter()
            .map(|(path, finding, _)| (path.as_str(), *finding))
            .collect()
    }

    /// Whether a line branch `side` added is named as missing.
    fn dropped_by(&self, side: &str) -> bool {
        let prefix = format!("branch {side}:");
        self.findings.iter().any(|(_, finding, detail)| {
            *finding == Finding::LineMissing && detail.starts_with(&prefix)
        })
    }

    fn report(&self) -> String {
        self.findings
            .iter()
            .map(|(path, finding, detail)| format!("  {path}: {finding:?} {detail}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// The producer whose rule claimed `member`, where one did.
fn producer_of(population: &Population, member: &Member) -> Option<Producer> {
    population
        .outputs
        .iter()
        .find(|output| output.path == member.path)
        .map(|output| output.producer)
}

/// A producer this test runs over the merged tree.
fn runnable(producer: Option<Producer>) -> bool {
    matches!(
        producer,
        Some(Producer::Generate | Producer::TaxonomyResolve)
    )
}

impl Pair {
    /// Commit a scratch copy of this repository as `main`, compute its
    /// population, and build branches `a` and `b` over it.
    fn built(label: &str, decisions: Decisions) -> Option<Pair> {
        let copy = Scratch::of_this_repository(label)?;
        copy.git(&["add", "-A"]);
        copy.git(&["commit", "-q", "-m", "the base"]);
        let population = derived::population(&copy.at);
        assert!(
            !population.members.is_empty(),
            "the scratch copy has a derived population"
        );

        // What each branch provokes, from the population and nothing else.
        let mut edits: [Vec<String>; 2] = [Vec::new(), Vec::new()];
        let mut unprovokable = BTreeSet::new();
        let mut one_sided = BTreeSet::new();
        for member in &population.members {
            let body = copy.read(&member.path);
            if marker(&body).as_deref() != Some("shelf_index") {
                continue;
            }
            let shelf = Path::new(&member.path)
                .parent()
                .expect("an index has a shelf")
                .to_string_lossy()
                .into_owned();
            let editable: Vec<String> = rows(&body)
                .into_iter()
                .map(|row| format!("{shelf}/{row}"))
                .filter(|path| !population.outputs.iter().any(|output| output.path == *path))
                .filter(|path| {
                    std::fs::read_to_string(copy.path(path))
                        .ok()
                        .is_some_and(|body| retitled(&body, "a").is_some())
                })
                .collect();
            match (editable.first(), editable.last()) {
                (Some(first), Some(last)) => {
                    edits[0].push(first.clone());
                    // Counted from the rows, not from the edit below, so a
                    // branch `b` that stops editing is not excused by it.
                    if editable.len() == 1 {
                        one_sided.insert(member.path.clone());
                    }
                    if last != first {
                        edits[1].push(last.clone());
                    }
                }
                _ => {
                    unprovokable.insert(member.path.clone());
                }
            }
        }
        let next = next_decision(&copy);

        for (side, name) in ["a", "b"].iter().enumerate() {
            copy.git(&["checkout", "-q", "main"]);
            copy.git(&["checkout", "-q", "-b", name]);
            for path in &edits[side] {
                let body = copy.read(path);
                let edited = retitled(&body, name).expect("an editable row has a title");
                copy.write(path, &edited);
            }
            add_decision(&copy, side, name, next, decisions);
            add_retired_term(&copy, side, name);
            for member in &population.members {
                provoke_member(&copy, &population, member, side, name);
            }
            copy.headwater_ok(&["taxonomy", "resolve"]);
            copy.headwater_ok(&["generate"]);
            copy.git(&["add", "-A"]);
            copy.git(&["commit", "-q", "-m", &format!("branch {name}")]);
        }

        let moved_by = ["a", "b"].map(|side| {
            let changed = copy.lines(&["diff", "--name-only", "main", side]);
            population
                .members
                .iter()
                .map(|member| member.path.clone())
                .filter(|path| changed.contains(path))
                .collect::<BTreeSet<String>>()
        });
        let moved = moved_by[0].union(&moved_by[1]).cloned().collect();
        Some(Pair {
            copy,
            population,
            moved,
            moved_by,
            unprovokable,
            one_sided,
        })
    }

    /// Merge `a` into `b` with `.gitattributes` as the base commits it, no
    /// driver configured, and `override_` as the clone's own
    /// `info/attributes`, which wins over every `.gitattributes`. Check every
    /// member the pair moved, then put the tree back on `b`.
    fn merged(&self, override_: &str) -> Merged {
        let copy = &self.copy;
        let population = &self.population;
        let info = copy.at.join(
            copy.git(&["rev-parse", "--git-path", "info/attributes"])
                .trim_end(),
        );
        copy.write_at(&info, override_);
        copy.git(&["checkout", "-q", "b"]);
        let _ = copy.git_output(&["merge", "--no-edit", "a"]);
        let conflicted = copy.lines(&["diff", "--name-only", "--diff-filter=U"]);
        let merged: Vec<(String, Option<Vec<u8>>)> = population
            .members
            .iter()
            .map(|member| (member.path.clone(), copy.bytes(&member.path)))
            .collect();
        let merged_bytes = |path: &str| {
            merged
                .iter()
                .find(|(listed, _)| listed == path)
                .and_then(|(_, bytes)| bytes.clone())
        };

        let mut findings = Vec::new();
        for member in &population.members {
            if !self.moved.contains(&member.path) {
                continue;
            }
            let body = merged_bytes(&member.path)
                .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
                .unwrap_or_default();
            let is_conflicted = conflicted.contains(&member.path);
            let mut find = |finding: Finding, detail: String| {
                findings.push((member.path.clone(), finding, detail));
            };
            match (member.shape.treatment(), is_conflicted) {
                (Treatment::Refuse, _) => {
                    if !is_conflicted {
                        find(Finding::FoldMergedClean, String::new());
                    }
                    if body.contains("<<<<<<<") {
                        find(Finding::FoldMarkers, String::new());
                    }
                    let current = copy.git(&["show", &format!("HEAD:{}", member.path)]);
                    if body != current {
                        find(Finding::FoldNotCurrentSide, String::new());
                    }
                }
                (Treatment::Union, true) => find(Finding::StoreConflicted, String::new()),
                // A record per entity that conflicted is the correct report.
                (_, true) => {}
                // Git writes no conflict marker into a path it merged clean,
                // so what a clean merge can get wrong is a lost line, which
                // a driver that keeps one side does, and the order of lines,
                // which the producer check below reads.
                (_, false) => {
                    for side in ["a", "b"] {
                        for line in added_lines(copy, side, &member.path) {
                            if !body.lines().any(|merged| merged == line) {
                                find(Finding::LineMissing, format!("branch {side}: {line}"));
                            }
                        }
                    }
                }
            }
        }

        // Resolve every conflict by taking the current side, then run both
        // producers over the merged tree and name every clean member they move.
        for path in &conflicted {
            let _ = copy.git_output(&["checkout", "--ours", "--", path]);
        }
        // A fold the merge took clean is put back to the current side too,
        // because `headwater taxonomy resolve` refuses a lock that a union
        // merge interleaved rather than rewrite it.
        for (path, finding, _) in &findings {
            if *finding == Finding::FoldMergedClean {
                copy.git(&["checkout", "HEAD", "--", path]);
            }
        }
        copy.headwater_ok(&["taxonomy", "resolve"]);
        copy.headwater_ok(&["generate"]);
        for member in &population.members {
            let producer = producer_of(population, member);
            if conflicted.contains(&member.path) || !runnable(producer) {
                continue;
            }
            if copy.bytes(&member.path) != merged_bytes(&member.path) {
                findings.push((
                    member.path.clone(),
                    Finding::ProducerRewrites,
                    producer
                        .expect("a runnable member has a producer")
                        .command()
                        .to_string(),
                ));
            }
        }
        findings.sort();
        findings.dedup();

        copy.git(&["reset", "-q", "--hard", "b"]);
        copy.git(&["clean", "-q", "-f", "-d"]);
        let _ = std::fs::remove_file(&info);
        Merged {
            conflicted,
            findings,
        }
    }
}

/// Every line branch `side` added to `path` relative to the base.
fn added_lines(copy: &Scratch, side: &str, path: &str) -> Vec<String> {
    let diff = copy.git(&["diff", "-U0", "main", side, "--", path]);
    diff.lines()
        .filter(|line| line.starts_with('+') && !line.starts_with("+++"))
        .map(|line| line[1..].to_string())
        .collect()
}

/// The numbered records of the decision shelf, by sequence.
fn decisions(copy: &Scratch) -> Vec<(u32, String)> {
    let mut found: Vec<(u32, String)> = std::fs::read_dir(copy.path(DECISIONS))
        .expect("the decision shelf reads")
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().to_string_lossy().into_owned();
            let seq = name.get(..4)?.parse::<u32>().ok()?;
            Some((seq, name))
        })
        .collect();
    found.sort();
    found
}

/// The sequence one past the highest decision record of the shelf.
fn next_decision(copy: &Scratch) -> u32 {
    decisions(copy)
        .last()
        .expect("the decision shelf holds a numbered record")
        .0
        + 1
}

/// A decision record with identifier `next + side`, in the scheme of the
/// shelf's first record, in a file `placement` names.
fn add_decision(copy: &Scratch, side: usize, name: &str, next: u32, placement: Decisions) {
    let (_, record) = decisions(copy)
        .into_iter()
        .find(|(seq, _)| *seq > 0)
        .expect("the decision shelf holds a numbered record");
    let first = copy.read(&format!("{DECISIONS}/{record}"));
    let id = first
        .lines()
        .find_map(|line| line.strip_prefix("id: "))
        .expect("the first decision has an identifier");
    let prefix = id.trim_end_matches(|c: char| c.is_ascii_digit());
    let seq = next + side as u32;
    let file = match (placement, side) {
        (Decisions::Apart, 0) => format!("0000-the-merge-provocation-of-branch-{name}.md"),
        (Decisions::Apart, _) => format!("9999-the-merge-provocation-of-branch-{name}.md"),
        (Decisions::Adjacent, _) => format!("{seq:04}-the-merge-provocation-of-branch-{name}.md"),
    };
    copy.write(
        &format!("{DECISIONS}/{file}"),
        &format!(
            "---\nid: {prefix}{seq:04}\ntitle: \"The merge provocation of branch {name}\"\nstatus: draft\nsummary: \"The decision branch {name} adds.\"\n---\n\n# The merge provocation of branch {name}\n"
        ),
    );
}

/// One retired term in the overlay, first in the list on side 0 and last on
/// side 1, so the two edits are far apart and the lock moves on both.
fn add_retired_term(copy: &Scratch, side: usize, name: &str) {
    const OVERLAY: &str = ".headwater/overlay.yml";
    let overlay = copy.read(OVERLAY);
    let lines: Vec<&str> = overlay.split_inclusive('\n').collect();
    let opens = lines
        .iter()
        .position(|line| line.trim() == "retired_terms:")
        .expect("the overlay lists retired terms");
    let indent = lines[opens].len() - lines[opens].trim_start().len();
    let closes = lines[opens + 1..]
        .iter()
        .position(|line| {
            let body = line.trim_start();
            !body.is_empty() && line.len() - body.len() <= indent
        })
        .map_or(lines.len(), |offset| opens + 1 + offset);
    let at = if side == 0 { opens + 1 } else { closes };
    let pad = " ".repeat(indent + 2);
    let entry = format!(
        "{pad}- term: merge provocation {name}\n{pad}  reason: the term branch {name} retires\n"
    );
    let mut out: String = lines[..at].concat();
    out.push_str(&entry);
    out.push_str(&lines[at..].concat());
    copy.write(OVERLAY, &out);
}

/// The planted provocation for a member no branch's documents move: a reading
/// for an append store, a line at one end of a decomposed recorded fixture,
/// and a different last line for a recorded fold. A member a producer of the
/// scratch tree writes is moved by the documents and the overlay instead.
fn provoke_member(
    copy: &Scratch,
    population: &Population,
    member: &Member,
    side: usize,
    name: &str,
) {
    if runnable(producer_of(population, member)) {
        return;
    }
    let body = copy.read(&member.path);
    let edited = match member.shape {
        Shape::IndependentLines => {
            let last = body.lines().last().unwrap_or("{}");
            let reading = match last.strip_prefix('{') {
                Some(rest) => format!("{{\"provocation\":\"{name}\",{rest}"),
                None => format!("provocation {name}"),
            };
            format!("{body}{reading}\n")
        }
        Shape::RecordPerEntity if side == 0 => format!("provocation {name}\n{body}"),
        Shape::RecordPerEntity | Shape::Fold => format!("{body}provocation {name}\n"),
    };
    copy.write(&member.path, &edited);
}

/// The main case: the committed attributes, and every member holds. Arm A
/// follows it over the same pair.
#[test]
fn every_member_of_the_derived_population_merges_as_its_shape_promises() {
    let Some(pair) = Pair::built("main", Decisions::Apart) else {
        return;
    };
    let population = &pair.population;
    assert!(
        population.agrees(),
        "the base agrees with its own attributes, so an arm's one line is its only difference"
    );
    let merged = pair.merged("");
    assert!(
        merged.findings.is_empty(),
        "every member merges as its shape promises, and these do not:\n{}",
        merged.report()
    );

    // Coverage: a rule that stops matching makes this red, not vacuous.
    for shape in derived::SHAPES {
        assert!(
            population
                .members
                .iter()
                .any(|member| member.shape == *shape && pair.moved.contains(&member.path)),
            "the pair provokes a member of the shape {shape:?}"
        );
    }
    for producer in &population.held {
        let outputs: Vec<&str> = population
            .outputs
            .iter()
            .filter(|output| output.producer == *producer)
            .map(|output| output.path.as_str())
            .collect();
        if outputs.is_empty() {
            eprintln!(
                "`{}` is held and writes no output in this tree, so nothing of it is provoked",
                producer.command()
            );
            continue;
        }
        assert!(
            outputs.iter().any(|path| pair.moved.contains(*path)),
            "the pair provokes an output of `{}`",
            producer.command()
        );
    }

    // Every member the pair reaches moves. A producer output of another kind,
    // such as a probe-result page, moves only with what its producer reads
    // from a transcript, and is counted below rather than passed in silence.
    let must_move = |member: &Member| -> bool {
        if pair.unprovokable.contains(&member.path) {
            return false;
        }
        match producer_of(population, member) {
            Some(Producer::Generate) => matches!(
                marker(&pair.copy.git(&["show", &format!("main:{}", member.path)])).as_deref(),
                Some("shelf_index" | "site_nav" | "corpus_descriptor")
            ),
            _ => true,
        }
    };
    let still: Vec<&str> = population
        .members
        .iter()
        .filter(|member| must_move(member) && !pair.moved.contains(&member.path))
        .map(|member| member.path.as_str())
        .collect();
    assert!(
        still.is_empty(),
        "the pair moves every member it reaches, and not {still:?}"
    );
    // Both branches move each of them, or the provocation is one-sided and
    // a merge of it is no merge at all. A shelf index with one editable row
    // is edited by `a` alone, and is counted below.
    for (side, name) in ["a", "b"].iter().enumerate() {
        let missed: Vec<&str> = population
            .members
            .iter()
            .filter(|member| must_move(member) && !pair.one_sided.contains(&member.path))
            .filter(|member| !pair.moved_by[side].contains(&member.path))
            .map(|member| member.path.as_str())
            .collect();
        assert!(
            missed.is_empty(),
            "branch {name} moves every member it reaches, and not {missed:?}"
        );
    }
    eprintln!(
        "{} of {} shelf indexes have one editable row, which branch a alone moves",
        pair.one_sided.len(),
        population
            .members
            .iter()
            .filter(|member| {
                marker(&pair.copy.git(&["show", &format!("main:{}", member.path)])).as_deref()
                    == Some("shelf_index")
            })
            .count()
    );

    let total = population.members.len();
    let unmoved = total - pair.moved.len();
    let clean = pair
        .moved
        .iter()
        .filter(|path| !merged.conflicted.contains(*path))
        .count();
    eprintln!(
        "{} of {total} members moved: {clean} merged clean and held, {} conflicted as their shape allows",
        pair.moved.len(),
        pair.moved.len() - clean
    );
    eprintln!(
        "{unmoved} of {total} members no branch moved, of which {} are shelf indexes whose every row is a producer output",
        pair.unprovokable.len()
    );

    // Arm A: the lock's merge attribute is unspecified, as if its `-merge`
    // line left `.gitattributes`. The lock takes conflict markers.
    let lock = derived::LOCK;
    let arm_a = pair.merged(&format!("{lock} !merge\n"));
    assert_eq!(
        arm_a.found(),
        BTreeSet::from([
            (lock, Finding::FoldMarkers),
            (lock, Finding::FoldNotCurrentSide)
        ]),
        "Arm A names the lock for its markers, and nothing else:\n{}",
        arm_a.report()
    );

    // Arm C: the lock takes `merge=union`. The merge is clean, holds both
    // sides' digest lines, and the resolver rewrites it.
    let arm_c = pair.merged(&format!("{lock} merge=union\n"));
    assert_eq!(
        arm_c.found(),
        BTreeSet::from([
            (lock, Finding::FoldMergedClean),
            (lock, Finding::FoldNotCurrentSide),
            (lock, Finding::ProducerRewrites)
        ]),
        "Arm C names the lock for a clean merge the resolver rewrites, and nothing else:\n{}",
        arm_c.report()
    );
}

/// The decisive fixture, a record per entity whose treatment stops holding,
/// and the two arms for the shapes no producer of the scratch tree writes.
///
/// The pair adds two decisions with adjacent file names, so that the decision
/// shelf index takes one hunk on each side. With the committed attributes that
/// is an ordinary conflict, and the control finds nothing. Each arm then sets
/// one line of the clone's own `info/attributes`, and its findings name the
/// one path that line governs and no other.
#[test]
fn a_record_or_a_store_whose_treatment_stops_holding_is_named() {
    let index = format!("{DECISIONS}/README.md");
    let Some(pair) = Pair::built("arm-b", Decisions::Adjacent) else {
        return;
    };
    assert!(
        pair.moved.contains(&index),
        "the pair moves the decision shelf index"
    );

    let control = pair.merged("");
    assert!(
        control.findings.is_empty(),
        "the control finds nothing:\n{}",
        control.report()
    );
    assert!(
        control.conflicted.contains(&index),
        "two adjacent rows conflict as text in the control, so a clean merge of them below is the arm's doing"
    );

    // Arm B: the decision shelf index takes `merge=union`. The merge is clean
    // and out of the fixed order, which only the producer's bytes reveal.
    let arm_b = pair.merged(&format!("{index} merge=union\n"));
    assert_eq!(
        arm_b.found(),
        BTreeSet::from([(index.as_str(), Finding::ProducerRewrites)]),
        "Arm B names the decision shelf index as bytes `headwater generate` rewrites, and nothing else:\n{}",
        arm_b.report()
    );

    // Arm D: an append store loses `merge=union`. Both branches appended at
    // its end, so a text merge conflicts.
    let store = pair
        .population
        .members
        .iter()
        .find(|member| member.shape == Shape::IndependentLines && pair.moved.contains(&member.path))
        .expect("the pair moves an append store")
        .path
        .clone();
    let arm_d = pair.merged(&format!("{store} !merge\n"));
    assert_eq!(
        arm_d.found(),
        BTreeSet::from([(store.as_str(), Finding::StoreConflicted)]),
        "Arm D names the store for its conflict, and nothing else:\n{}",
        arm_d.report()
    );

    // Arm E: a decomposed recorded fixture takes a driver that keeps the
    // current side and exits 0, which no producer here can rerun. The merge
    // is clean and drops the line the other branch inserted.
    let fixture = pair
        .population
        .members
        .iter()
        .find(|member| {
            member.shape == Shape::RecordPerEntity
                && !runnable(producer_of(&pair.population, member))
                && pair.moved.contains(&member.path)
        })
        .expect("the pair moves a decomposed recorded fixture")
        .path
        .clone();
    pair.copy
        .git(&["config", "merge.keep-current.driver", "true"]);
    let arm_e = pair.merged(&format!("{fixture} merge=keep-current\n"));
    pair.copy
        .git(&["config", "--unset", "merge.keep-current.driver"]);
    assert_eq!(
        arm_e.found(),
        BTreeSet::from([(fixture.as_str(), Finding::LineMissing)]),
        "Arm E names the fixture for the line it dropped, and nothing else:\n{}",
        arm_e.report()
    );
    assert!(
        arm_e.dropped_by("a") && !arm_e.dropped_by("b"),
        "Arm E drops branch a's line, the other side's, and keeps branch b's:\n{}",
        arm_e.report()
    );

    // Arm F: the same fixture takes a driver that keeps the other side, so
    // the merge drops the line of the current branch instead.
    pair.copy
        .git(&["config", "merge.keep-other.driver", "cp %B %A"]);
    let arm_f = pair.merged(&format!("{fixture} merge=keep-other\n"));
    pair.copy
        .git(&["config", "--unset", "merge.keep-other.driver"]);
    assert_eq!(
        arm_f.found(),
        BTreeSet::from([(fixture.as_str(), Finding::LineMissing)]),
        "Arm F names the fixture for the line it dropped, and nothing else:\n{}",
        arm_f.report()
    );
    assert!(
        arm_f.dropped_by("b") && !arm_f.dropped_by("a"),
        "Arm F drops branch b's line, the current side's, and keeps branch a's:\n{}",
        arm_f.report()
    );
}
