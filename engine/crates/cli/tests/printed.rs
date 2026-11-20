// SPDX-License-Identifier: Apache-2.0
//! What every command line of this binary prints, held byte for byte (#1958).
//!
//! The other targets of this crate assert on substrings of what a verb prints.
//! A command body that moves out of `main.rs` and prints one line in another
//! order, on the other stream, or with another exit status passes all of them.
//! This target holds the whole of it: for each line of
//! `fixtures/printed/manifest` it runs the binary over a fresh copy of a
//! committed tree, and compares standard output, standard error and the exit
//! status with three files under `fixtures/printed/blessed/`, as bytes.
//!
//! **To record a change a pull request means to make**, run
//!
//! ```text
//! HEADWATER_BLESS=1 cargo test -p headwater-cli --test printed
//! ```
//!
//! and read the diff of `fixtures/printed/blessed/` before you commit it. A
//! failure prints that command.
//!
//! Four things keep a blessed file from moving when nothing a command prints
//! moved:
//!
//! - **The trees carry their own taxonomy.** Each tree under
//!   `fixtures/printed/trees/` commits its package, its declaration and its
//!   lock, as `fixtures/answered-export/` does, so a change to this
//!   repository's taxonomy moves no blessed file. A committed lock can go
//!   stale, so the manifest must hold a passing `taxonomy resolve --check` line
//!   over each tree that carries one, and this target refuses a manifest that
//!   does not.
//! - **Every run is cold.** A line runs over a copy of its tree in a scratch
//!   directory, with that copy as the working directory and `--root .`, so no
//!   run reads a cache an earlier run wrote and a printed root is relative.
//! - **The environment is built and not inherited.** It is cleared, and then
//!   holds `PATH`, a scratch `HOME`, a git ceiling and one fixed git identity
//!   and date. `NO_COLOR`, `CLICOLOR_FORCE`, `COLUMNS` and
//!   `HEADWATER_NO_BANNER` all change bytes, and a developer's shell sets some
//!   of them. No line passes `--wide`, so the width is 80.
//! - **No captured stream names where it ran.** A stream that holds the
//!   scratch directory or this crate's own path fails before any comparison,
//!   because such a file passes here and fails on every other checkout.
//!
//! One `#[test]` reads and writes the blessed files, and no other case opens
//! one. Cargo runs the cases of a target as threads of one process, and a
//! second case that read a blessed file while a bless truncated it is what
//! HW-OBL-0176 and HW-OBL-0215 record. The cases at the end of this file hold
//! the manifest reader and the coverage rule over lines they build in memory.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The command a failure names, and the only way a blessed file is written.
const BLESS: &str = "HEADWATER_BLESS=1 cargo test -p headwater-cli --test printed";

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/printed")
}

/// What a line is expected to exit with. `Pass` is 0 and `Refuse` is 1, and
/// this binary has no third status.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Outcome {
    Pass,
    Refuse,
}

impl Outcome {
    fn code(self) -> i32 {
        match self {
            Outcome::Pass => 0,
            Outcome::Refuse => 1,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Outcome::Pass => "pass",
            Outcome::Refuse => "refuse",
        }
    }
}

/// One step that shapes the scratch root before the line runs.
#[derive(Debug)]
enum Step {
    /// A command line of this binary, held to exit 0. Nothing it prints is
    /// compared.
    Headwater(Vec<String>),
    /// A git command, held to success, under the fixed identity and date.
    Git(Vec<String>),
    /// Write a file under the root, parents included.
    Write(String, String),
    /// Remove a file or a directory under the root.
    Remove(String),
    /// Remove each line of a file under the root that opens with a prefix.
    DropLines(String, String),
}

/// One section of the manifest.
#[derive(Debug)]
struct Line {
    label: String,
    row: String,
    outcome: Outcome,
    /// Why the row has no line of the other outcome, where it has none.
    only: Option<String>,
    trees: Vec<String>,
    setup: Vec<Step>,
    stdin: Option<Vec<u8>>,
    run: Vec<String>,
    /// Where the section opens in the manifest, for a message.
    at: usize,
}

/// Split one manifest value into words.
///
/// A word is a run of characters with no blank in it, or a double-quoted
/// string in which `\n`, `\t`, `\"` and `\\` are the four escapes. Nothing
/// else is special, so a value reads the same in the file as on a command
/// line a person would type.
fn words(value: &str, at: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = value.chars().peekable();
    while let Some(&next) = chars.peek() {
        if next.is_whitespace() {
            chars.next();
            continue;
        }
        let mut word = String::new();
        if next == '"' {
            chars.next();
            loop {
                match chars.next() {
                    None => panic!("manifest line {at}: a quoted word does not close"),
                    Some('"') => break,
                    Some('\\') => match chars.next() {
                        Some('n') => word.push('\n'),
                        Some('t') => word.push('\t'),
                        Some('"') => word.push('"'),
                        Some('\\') => word.push('\\'),
                        other => panic!("manifest line {at}: no escape is `\\{other:?}`"),
                    },
                    Some(other) => word.push(other),
                }
            }
        } else {
            while let Some(&inner) = chars.peek() {
                if inner.is_whitespace() {
                    break;
                }
                word.push(inner);
                chars.next();
            }
        }
        out.push(word);
    }
    out
}

/// The bytes of a `stdin` value. The word is read as [`words`] reads one, and
/// then `%HH` is one byte, so a line can feed bytes that are not text. `%25`
/// is a percent sign.
fn bytes(word: &str, at: usize) -> Vec<u8> {
    let raw = word.as_bytes();
    let mut out = Vec::with_capacity(raw.len());
    let mut index = 0;
    while index < raw.len() {
        if raw[index] != b'%' {
            out.push(raw[index]);
            index += 1;
            continue;
        }
        let byte = raw
            .get(index + 1..index + 3)
            .and_then(|pair| std::str::from_utf8(pair).ok())
            .and_then(|pair| u8::from_str_radix(pair, 16).ok())
            .unwrap_or_else(|| panic!("manifest line {at}: `%` in `stdin` opens two hex digits"));
        out.push(byte);
        index += 3;
    }
    out
}

fn step(value: &str, at: usize) -> Step {
    let mut parts = words(value, at);
    assert!(!parts.is_empty(), "manifest line {at}: an empty setup step");
    let kind = parts.remove(0);
    match (kind.as_str(), parts.len()) {
        ("headwater", 1..) => Step::Headwater(parts),
        ("git", 1..) => Step::Git(parts),
        ("write", 2) => {
            let content = parts.pop().expect("two words");
            Step::Write(parts.pop().expect("two words"), content)
        }
        ("remove", 1) => Step::Remove(parts.pop().expect("one word")),
        ("drop-lines", 2) => {
            let prefix = parts.pop().expect("two words");
            Step::DropLines(parts.pop().expect("two words"), prefix)
        }
        _ => panic!(
            "manifest line {at}: a setup step is `headwater <words>`, `git <words>`, \
             `write <path> <content>`, `remove <path>` or `drop-lines <path> <prefix>`"
        ),
    }
}

/// A section being read: its label, the line it opens on, and each key with
/// its values and the line of each.
type Section<'a> = (String, usize, BTreeMap<&'a str, Vec<(String, usize)>>);

/// Read the manifest. Every malformed line is a panic that names its number,
/// because a manifest this target half-read is a fixture that half-ran.
fn manifest() -> Vec<Line> {
    let path = fixture().join("manifest");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let mut lines: Vec<Line> = Vec::new();
    let mut open: Option<Section<'_>> = None;
    let close = |open: Option<Section<'_>>, lines: &mut Vec<Line>| {
        let Some((label, at, mut keys)) = open else {
            return;
        };
        let mut one = |key: &'static str| -> Option<String> {
            let mut values = keys.remove(key)?;
            assert_eq!(
                values.len(),
                1,
                "manifest line {at}: [{label}] states `{key}` twice"
            );
            Some(values.remove(0).0)
        };
        let need = |value: Option<String>, key: &str| -> String {
            value.unwrap_or_else(|| panic!("manifest line {at}: [{label}] states no `{key}`"))
        };
        let row = need(one("row"), "row");
        let outcome = match need(one("outcome"), "outcome").as_str() {
            "pass" => Outcome::Pass,
            "refuse" => Outcome::Refuse,
            other => panic!(
                "manifest line {at}: [{label}] `outcome` is `pass` or `refuse`, not `{other}`"
            ),
        };
        let only = one("only");
        let trees = one("tree")
            .map(|value| value.split_whitespace().map(str::to_string).collect())
            .unwrap_or_default();
        let stdin = one("stdin").map(|value| {
            let mut parts = words(&value, at);
            assert_eq!(
                parts.len(),
                1,
                "manifest line {at}: [{label}] `stdin` is one quoted word"
            );
            bytes(&parts.remove(0), at)
        });
        let run = words(&need(one("run"), "run"), at);
        let setup = keys
            .remove("setup")
            .unwrap_or_default()
            .into_iter()
            .map(|(value, at)| step(&value, at))
            .collect();
        lines.push(Line {
            label,
            row,
            outcome,
            only,
            trees,
            setup,
            stdin,
            run,
            at,
        });
    };
    for (index, raw) in text.lines().enumerate() {
        let at = index + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(label) = line
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        {
            close(open.take(), &mut lines);
            assert!(
                !label.is_empty()
                    && label
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "manifest line {at}: a label is lower-case letters, digits and `-`, because it names three files"
            );
            open = Some((label.to_string(), at, BTreeMap::new()));
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .unwrap_or_else(|| panic!("manifest line {at}: neither `[label]` nor `key = value`"));
        let key = match key.trim() {
            "row" => "row",
            "outcome" => "outcome",
            "only" => "only",
            "tree" => "tree",
            "setup" => "setup",
            "stdin" => "stdin",
            "run" => "run",
            other => panic!("manifest line {at}: no key is `{other}`"),
        };
        let Some((_, _, keys)) = open.as_mut() else {
            panic!("manifest line {at}: a key before any `[label]`");
        };
        keys.entry(key)
            .or_default()
            .push((value.trim().to_string(), at));
    }
    close(open.take(), &mut lines);
    lines
}

/// The survey rows of HW-OBL-0132, derived and never listed.
///
/// Every form of `headwater_verbs::VERBS`, then for each verb that takes a
/// second word the bare verb and the verb with a word it does not carry, then
/// a first word that is no verb. On the tree this was written against that is
/// 41 forms, four of each second kind, and one: 50.
fn rows() -> Vec<String> {
    let mut rows = Vec::new();
    for verb in headwater_verbs::VERBS {
        rows.extend(verb.forms());
    }
    for verb in headwater_verbs::VERBS {
        if !verb.words.is_empty() {
            rows.push(format!("headwater {}", verb.name));
            rows.push(format!("headwater {} <other>", verb.name));
        }
    }
    rows.push("headwater <unknown>".to_string());
    rows
}

/// Whether the words a line runs are a command line of the row it names.
///
/// Without this a manifest could name a row and run another verb, and the
/// coverage below would count a row nothing ran.
fn runs_its_row(row: &str, run: &[String]) -> bool {
    let first = run.first().map(String::as_str);
    let second = run.get(1).map(String::as_str);
    let verb = |name: &str| headwater_verbs::VERBS.iter().find(|verb| verb.name == name);
    let parts: Vec<&str> = row.split_whitespace().skip(1).collect();
    match parts.as_slice() {
        ["<unknown>"] => first.is_some_and(|word| !word.starts_with('-') && verb(word).is_none()),
        [name, "<other>"] => {
            first == Some(*name)
                && second.is_some_and(|word| {
                    !word.starts_with('-')
                        && verb(name).is_some_and(|verb| verb.words.iter().all(|w| w.name != word))
                })
        }
        [name] => match verb(name).is_some_and(|verb| !verb.words.is_empty()) {
            true => first == Some(*name) && run.len() == 1,
            false => first == Some(*name),
        },
        [name, word] => first == Some(*name) && second == Some(*word),
        _ => false,
    }
}

/// Hold the manifest against the rows, and return every complaint.
fn coverage(lines: &[Line]) -> Vec<String> {
    let rows = rows();
    let mut complaints = Vec::new();
    let mut labels = BTreeSet::new();
    let mut seen: BTreeMap<&str, (BTreeSet<Outcome>, bool)> = BTreeMap::new();
    for line in lines {
        if !labels.insert(line.label.as_str()) {
            complaints.push(format!("[{}] is the label of two lines", line.label));
        }
        if !rows.contains(&line.row) {
            complaints.push(format!(
                "[{}] names the row `{}`, which is no row of the survey",
                line.label, line.row
            ));
            continue;
        }
        if !runs_its_row(&line.row, &line.run) {
            complaints.push(format!(
                "[{}] names the row `{}` and runs `headwater {}`, which is another command line",
                line.label,
                line.row,
                line.run.join(" ")
            ));
            continue;
        }
        let entry = seen.entry(line.row.as_str()).or_default();
        entry.0.insert(line.outcome);
        entry.1 |= line.only.is_some();
    }
    for row in &rows {
        match seen.get(row.as_str()) {
            None => complaints.push(format!("`{row}` is covered by no line")),
            Some((outcomes, stated)) => match (outcomes.len(), stated) {
                (2, false) => {}
                (2, true) => complaints.push(format!(
                    "`{row}` has a line of each outcome, and one of them states `only`"
                )),
                (_, true) => {}
                (_, false) => {
                    let has = outcomes.iter().next().expect("one outcome").name();
                    complaints.push(format!(
                        "`{row}` has only a line that is `{has}`, and none states with `only` why the row has no other outcome"
                    ));
                }
            },
        }
    }
    complaints
}

/// The trees a manifest line can name, each a directory under `trees/`.
fn trees() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(fixture().join("trees"))
        .map(|entries| {
            entries
                .map(|entry| entry.expect("the entry reads"))
                .filter(|entry| entry.path().is_dir())
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

/// Each tree that commits a lock needs one passing `taxonomy resolve --check`
/// line over that tree alone, with no setup, so a stale committed lock is a
/// red run of this target.
fn unheld_locks(lines: &[Line]) -> Vec<String> {
    trees()
        .into_iter()
        .filter(|tree| {
            fixture()
                .join("trees")
                .join(tree)
                .join(".headwater/taxonomy.lock")
                .is_file()
        })
        .filter(|tree| {
            !lines.iter().any(|line| {
                line.row == "headwater taxonomy resolve"
                    && line.outcome == Outcome::Pass
                    && line.trees == [tree.clone()]
                    && line.setup.is_empty()
                    && line.run.iter().any(|word| word == "--check")
            })
        })
        .map(|tree| {
            format!(
                "the tree `{tree}` commits a lock, and no passing line runs `taxonomy resolve --check` over it alone with no setup"
            )
        })
        .collect()
}

/// A scratch directory that is removed when it is dropped (#1158).
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A command under the environment every line and every setup step runs in.
///
/// Cleared first, so nothing a developer's shell or a CI job exports reaches
/// the binary. `PATH` is the one value taken from outside, because the lines
/// that reach git have to find it.
fn command(program: &str, scratch: &Path, root: &Path) -> Command {
    let mut command = Command::new(program);
    command.current_dir(root).env_clear();
    if let Some(path) = std::env::var_os("PATH") {
        command.env("PATH", path);
    }
    command
        .env("HOME", scratch.join("home"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", scratch.join("home/gitconfig"))
        .env("GIT_AUTHOR_NAME", "A Printer")
        .env("GIT_AUTHOR_EMAIL", "printer@example.invalid")
        .env("GIT_AUTHOR_DATE", "2026-01-15T00:00:00+00:00")
        .env("GIT_COMMITTER_NAME", "A Printer")
        .env("GIT_COMMITTER_EMAIL", "printer@example.invalid")
        .env("GIT_COMMITTER_DATE", "2026-01-15T00:00:00+00:00")
        .env("TZ", "UTC")
        .env("LC_ALL", "C");
    common::fence(&mut command, root);
    command
}

/// What one line printed and how it exited.
struct Ran {
    out: Vec<u8>,
    err: Vec<u8>,
    status: String,
}

/// Copy the trees of a line into a fresh scratch root, in the order named.
fn rooted(line: &Line) -> (Scratch, PathBuf) {
    let scratch = Scratch(common::scratch_base().join(format!(
        "headwater-cli-printed-{}-{}",
        std::process::id(),
        line.label
    )));
    let _ = std::fs::remove_dir_all(&scratch.0);
    let root = scratch.0.join("root");
    std::fs::create_dir_all(&root).expect("the scratch root is made");
    std::fs::create_dir_all(scratch.0.join("home")).expect("the scratch home is made");
    std::fs::write(scratch.0.join("home/gitconfig"), "").expect("the empty git config writes");
    for tree in &line.trees {
        let from = fixture().join("trees").join(tree);
        assert!(
            from.is_dir(),
            "[{}] names the tree `{tree}`, and {} is no directory",
            line.label,
            from.display()
        );
        common::copy(&from, &root);
    }
    (scratch, root)
}

fn prepare(line: &Line, scratch: &Path, root: &Path) {
    for step in &line.setup {
        let (program, arguments) = match step {
            Step::Write(path, content) => {
                let at = root.join(path);
                std::fs::create_dir_all(at.parent().expect("a file has a parent"))
                    .expect("the parent is made");
                std::fs::write(&at, content).expect("the setup file writes");
                continue;
            }
            Step::Remove(path) => {
                let at = root.join(path);
                match at.is_dir() {
                    true => std::fs::remove_dir_all(&at),
                    false => std::fs::remove_file(&at),
                }
                .unwrap_or_else(|error| {
                    panic!("[{}] setup cannot remove {path}: {error}", line.label)
                });
                continue;
            }
            Step::DropLines(path, prefix) => {
                let at = root.join(path);
                let text = std::fs::read_to_string(&at).unwrap_or_else(|error| {
                    panic!("[{}] setup cannot read {path}: {error}", line.label)
                });
                let kept: String = text
                    .split_inclusive('\n')
                    .filter(|line| !line.starts_with(prefix.as_str()))
                    .collect();
                assert_ne!(
                    kept, text,
                    "[{}] setup drops no line of {path}: none opens with `{prefix}`",
                    line.label
                );
                std::fs::write(&at, kept).expect("the setup file writes");
                continue;
            }
            Step::Headwater(arguments) => (env!("CARGO_BIN_EXE_headwater"), arguments),
            Step::Git(arguments) => ("git", arguments),
        };
        let output = command(program, scratch, root)
            .args(arguments)
            .stdin(Stdio::null())
            .output()
            .unwrap_or_else(|error| panic!("[{}] setup cannot run {program}: {error}", line.label));
        assert!(
            output.status.success(),
            "[{}] (manifest line {}) the setup step `{program} {}` did not exit 0:\n{}{}",
            line.label,
            line.at,
            arguments.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn ran(line: &Line, scratch: &Path, root: &Path) -> Ran {
    let mut child = command(env!("CARGO_BIN_EXE_headwater"), scratch, root)
        .args(&line.run)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary runs");
    // The stream is always a closed pipe and never this process's own
    // standard input, so no line waits on a terminal.
    let mut stdin = child.stdin.take().expect("the pipe is there");
    if let Some(bytes) = &line.stdin {
        // A verb that refuses before it reads closes the pipe first, and that
        // is its answer rather than a failure of this write.
        let _ = stdin.write_all(bytes);
    }
    drop(stdin);
    let output = child.wait_with_output().expect("the binary exits");
    Ran {
        out: output.stdout,
        err: output.stderr,
        status: match output.status.code() {
            Some(code) => format!("{code}\n"),
            None => "killed by a signal\n".to_string(),
        },
    }
}

fn holds(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

/// The byte strings no captured stream may hold: where this run's scratch
/// trees are, in both spellings, and where this checkout is.
fn local_paths(scratch: &Path) -> Vec<(String, Vec<u8>)> {
    let mut paths = vec![(
        "the scratch directory".to_string(),
        scratch.to_string_lossy().into_owned().into_bytes(),
    )];
    if let Ok(real) = scratch.canonicalize() {
        paths.push((
            "the scratch directory".to_string(),
            real.to_string_lossy().into_owned().into_bytes(),
        ));
    }
    paths.push((
        "this crate's directory".to_string(),
        env!("CARGO_MANIFEST_DIR").as_bytes().to_vec(),
    ));
    paths
}

/// Where the two byte strings first differ, as a line number and the two
/// lines, for a message a person reads before opening either file.
fn first_difference(blessed: &[u8], printed: &[u8]) -> String {
    let blessed = String::from_utf8_lossy(blessed);
    let printed = String::from_utf8_lossy(printed);
    let mut theirs = blessed.split('\n');
    let mut ours = printed.split('\n');
    let mut number = 1;
    loop {
        match (theirs.next(), ours.next()) {
            (Some(a), Some(b)) if a == b => number += 1,
            (a, b) => {
                let show = |line: Option<&str>| match line {
                    Some(text) => format!("`{text}`"),
                    None => "nothing, the stream ends".to_string(),
                };
                return format!("line {number}: blessed {}, printed {}", show(a), show(b));
            }
        }
    }
}

/// Under a bless, bring a committed lock back to what its sources resolve to,
/// and say so. Without this a bless after a resolver change would record a
/// failing `taxonomy resolve --check` line as the expectation.
fn reresolve(tree: &str) {
    let committed = fixture().join("trees").join(tree);
    let lock = committed.join(".headwater/taxonomy.lock");
    if !lock.is_file() {
        return;
    }
    let line = Line {
        label: format!("bless-resolve-{tree}"),
        row: String::new(),
        outcome: Outcome::Pass,
        only: None,
        trees: vec![tree.to_string()],
        setup: Vec::new(),
        stdin: None,
        run: Vec::new(),
        at: 0,
    };
    let (scratch, root) = rooted(&line);
    let resolve = |arguments: &[&str]| {
        command(env!("CARGO_BIN_EXE_headwater"), &scratch.0, &root)
            .args(arguments)
            .stdin(Stdio::null())
            .output()
            .expect("the binary runs")
    };
    if resolve(&["taxonomy", "resolve", "--check", "--root", "."])
        .status
        .success()
    {
        return;
    }
    let resolved = resolve(&["taxonomy", "resolve", "--root", "."]);
    assert!(
        resolved.status.success(),
        "the tree `{tree}` no longer resolves, so no bless can hold it:\n{}",
        String::from_utf8_lossy(&resolved.stderr)
    );
    std::fs::copy(root.join(".headwater/taxonomy.lock"), &lock).expect("the lock copies back");
    eprintln!(
        "blessed: the resolver moved the lock of the tree `{tree}`, and {} is re-resolved",
        lock.display()
    );
}

#[test]
fn every_command_line_prints_what_its_blessed_files_hold() {
    let lines = manifest();

    // The coverage is first, and it reads no blessed file, so a manifest that
    // misses a row fails for that reason and not for a file that is absent.
    let mut complaints = coverage(&lines);
    complaints.extend(unheld_locks(&lines));
    assert!(
        complaints.is_empty(),
        "fixtures/printed/manifest does not cover the {} command lines of this binary:\n  {}",
        rows().len(),
        complaints.join("\n  ")
    );

    let bless = std::env::var_os("HEADWATER_BLESS").is_some();
    let blessed = fixture().join("blessed");
    if bless {
        std::fs::create_dir_all(&blessed).expect("the blessed directory is made");
        for tree in trees() {
            reresolve(&tree);
        }
    }

    let mut moved = Vec::new();
    let mut named = BTreeSet::new();
    for line in &lines {
        let (scratch, root) = rooted(line);
        prepare(line, &scratch.0, &root);
        let ran = ran(line, &scratch.0, &root);
        moved.extend(judge(
            line,
            &ran,
            &local_paths(&scratch.0),
            &blessed,
            bless,
            &mut named,
        ));
    }
    moved.extend(strays(&blessed, &named, bless));

    assert!(
        moved.is_empty(),
        "what this binary prints is not what fixtures/printed/blessed holds, in {} place(s):\n  {}\n\n\
         If the change is one this pull request means to make, record it and read the diff:\n\n    \
         {BLESS}\n",
        moved.len(),
        moved.join("\n  ")
    );
}

/// Hold what one line printed against its three blessed files, or write them
/// under a bless, and return every complaint.
///
/// Two complaints are ones a bless cannot answer, and a bless writes no file
/// of a line that raises either. The first is a status that is not the outcome
/// the manifest states: recording it would make a refusal the passing line of
/// a row. The second is a stream that holds one of `local`, the paths of this
/// run: its blessed file would pass here and fail on every other checkout.
///
/// `named` gains the three file names of the line, so [`strays`] can tell a
/// blessed file no line names.
fn judge(
    line: &Line,
    ran: &Ran,
    local: &[(String, Vec<u8>)],
    blessed: &Path,
    bless: bool,
    named: &mut BTreeSet<String>,
) -> Vec<String> {
    let command = format!("headwater {}", line.run.join(" "));
    let streams: [(&str, &str, &[u8]); 3] = [
        ("out", "standard output", &ran.out),
        ("err", "standard error", &ran.err),
        ("status", "the exit status", ran.status.as_bytes()),
    ];
    let mut complaints = Vec::new();
    let mut unblessable = false;

    if ran.status != format!("{}\n", line.outcome.code()) {
        unblessable = true;
        complaints.push(format!(
            "[{}] (manifest line {}) is stated as `{}`, which is exit {}, and `{command}` exited {}. \
             A bless does not record this: the row loses its `{}` line, so the manifest has to change too",
            line.label,
            line.at,
            line.outcome.name(),
            line.outcome.code(),
            ran.status.trim_end(),
            line.outcome.name()
        ));
    }
    for (name, stream, bytes) in streams {
        named.insert(format!("{}.{name}", line.label));
        for (what, path) in local {
            if holds(bytes, path) {
                unblessable = true;
                complaints.push(format!(
                    "[{}] {stream} of `{command}` holds {what}. A bless does not record this: \
                     the file would pass on this checkout and fail on every other",
                    line.label
                ));
                break;
            }
        }
    }
    for (name, stream, bytes) in streams {
        let path = blessed.join(format!("{}.{name}", line.label));
        let recorded = std::fs::read(&path);
        if bless {
            // Written only where the bytes moved, so a second bless touches
            // nothing.
            if !unblessable && recorded.ok().as_deref() != Some(bytes) {
                std::fs::write(&path, bytes).expect("the blessed file writes");
                eprintln!("blessed: wrote {}", path.display());
            }
            continue;
        }
        match recorded {
            Err(error) => complaints.push(format!(
                "[{}] {stream} has no blessed file: fixtures/printed/blessed/{}.{name}: {error}",
                line.label, line.label
            )),
            Ok(recorded) if recorded != bytes => complaints.push(format!(
                "[{}] {stream} of `{command}` moved from fixtures/printed/blessed/{}.{name}\n      {}",
                line.label,
                line.label,
                first_difference(&recorded, bytes)
            )),
            Ok(_) => {}
        }
    }
    complaints
}

/// The blessed directory holds what the manifest names and nothing else, so a
/// line that left the manifest does not leave three files behind. A bless
/// removes each such file, and any other run complains of it.
fn strays(blessed: &Path, named: &BTreeSet<String>, bless: bool) -> Vec<String> {
    let mut complaints = Vec::new();
    let Ok(entries) = std::fs::read_dir(blessed) else {
        return complaints;
    };
    let mut names: Vec<String> = entries
        .map(|entry| {
            entry
                .expect("the entry reads")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|name| !named.contains(name))
        .collect();
    names.sort();
    for name in names {
        match bless {
            true => {
                std::fs::remove_file(blessed.join(&name)).expect("the stray file is removed");
                eprintln!("blessed: removed {}", blessed.join(&name).display());
            }
            false => complaints.push(format!(
                "fixtures/printed/blessed/{name} is the blessed file of no line of the manifest"
            )),
        }
    }
    complaints
}

/// A line as the manifest would state it, for the cases below. They read no
/// blessed file, so they do not race the one case that writes them.
fn stated(label: &str, row: &str, outcome: Outcome, only: Option<&str>, run: &str) -> Line {
    Line {
        label: label.to_string(),
        row: row.to_string(),
        outcome,
        only: only.map(str::to_string),
        trees: Vec::new(),
        setup: Vec::new(),
        stdin: None,
        run: words(run, 0),
        at: 0,
    }
}

/// The lines that cover every row but `headwater check`, so a case can add
/// its own lines for that row and read the one complaint they cause.
fn all_but_check() -> Vec<Line> {
    let mut lines = Vec::new();
    for row in rows() {
        if row == "headwater check" {
            continue;
        }
        let run = match row.split_once(' ').expect("a row has a word").1 {
            "<unknown>" => "nosuch".to_string(),
            other => other.replace("<other>", "nosuch"),
        };
        for outcome in [Outcome::Pass, Outcome::Refuse] {
            let label = format!("{}-{}", run.replace(' ', "-"), outcome.name());
            lines.push(stated(&label, &row, outcome, None, &run));
        }
    }
    lines
}

/// What [`coverage`] says about a manifest that covers every other row and
/// holds `extra`.
fn complaints(extra: Vec<Line>) -> Vec<String> {
    let mut lines = all_but_check();
    lines.extend(extra);
    coverage(&lines)
}

#[test]
fn the_rows_are_the_fifty_command_lines_of_the_survey() {
    let rows = rows();
    let forms: usize = headwater_verbs::VERBS
        .iter()
        .map(|verb| verb.forms().len())
        .sum();
    let worded = headwater_verbs::VERBS
        .iter()
        .filter(|verb| !verb.words.is_empty())
        .count();
    assert_eq!(rows.len(), forms + 2 * worded + 1);
    // The count HW-OBL-0132 states. A verb that joins `VERBS` moves it, and
    // the survey table gains a row in the same change.
    assert_eq!(rows.len(), 50, "{rows:?}");
    let distinct: BTreeSet<&String> = rows.iter().collect();
    assert_eq!(distinct.len(), rows.len(), "no row is listed twice");
}

#[test]
fn a_row_needs_both_outcomes_or_a_stated_reason() {
    let check = |label: &str, outcome, only| {
        stated(label, "headwater check", outcome, only, "check --root .")
    };
    let none: [&str; 0] = [];

    assert_eq!(
        complaints(Vec::new()),
        ["`headwater check` is covered by no line"]
    );
    assert_eq!(
        complaints(vec![
            check("a", Outcome::Pass, None),
            check("b", Outcome::Refuse, None)
        ]),
        none
    );
    let one = complaints(vec![check("a", Outcome::Pass, None)]);
    assert_eq!(one.len(), 1, "{one:?}");
    assert!(one[0].contains("has only a line that is `pass`"), "{one:?}");
    let two_passes = complaints(vec![
        check("a", Outcome::Pass, None),
        check("b", Outcome::Pass, None),
    ]);
    assert_eq!(
        two_passes.len(),
        1,
        "two lines of one outcome are one outcome: {two_passes:?}"
    );
    assert_eq!(
        complaints(vec![check("a", Outcome::Refuse, Some("it cannot pass"))]),
        none
    );
    let both_and_only = complaints(vec![
        check("a", Outcome::Pass, None),
        check("b", Outcome::Refuse, Some("it cannot pass")),
    ]);
    assert_eq!(both_and_only.len(), 1, "{both_and_only:?}");
    assert!(
        both_and_only[0].contains("states `only`"),
        "{both_and_only:?}"
    );
}

#[test]
fn a_line_is_counted_only_for_the_row_it_runs() {
    // A line that names `check` and runs another verb covers nothing.
    let other = complaints(vec![
        stated(
            "a",
            "headwater check",
            Outcome::Pass,
            None,
            "explain --root . x",
        ),
        stated(
            "b",
            "headwater check",
            Outcome::Refuse,
            None,
            "check --root .",
        ),
    ]);
    assert_eq!(other.len(), 2, "{other:?}");
    assert!(
        other[0].contains("which is another command line"),
        "{other:?}"
    );
    assert!(
        other[1].contains("has only a line that is `refuse`"),
        "{other:?}"
    );
    // A row the survey does not hold, and a label stated twice.
    let unknown = complaints(vec![stated(
        "a",
        "headwater nosuch",
        Outcome::Pass,
        None,
        "nosuch",
    )]);
    assert!(
        unknown[0].contains("is no row of the survey"),
        "{unknown:?}"
    );
    let twice = complaints(vec![
        stated("a", "headwater check", Outcome::Pass, None, "check"),
        stated("a", "headwater check", Outcome::Refuse, None, "check"),
    ]);
    assert_eq!(twice, ["[a] is the label of two lines"]);

    let run = |text: &str| words(text, 0);
    assert!(runs_its_row(
        "headwater sweep plan",
        &run("sweep plan --root .")
    ));
    assert!(!runs_its_row(
        "headwater sweep plan",
        &run("sweep report x")
    ));
    assert!(runs_its_row("headwater sweep", &run("sweep")));
    assert!(!runs_its_row("headwater sweep", &run("sweep plan")));
    assert!(runs_its_row(
        "headwater sweep <other>",
        &run("sweep nosuch")
    ));
    assert!(!runs_its_row("headwater sweep <other>", &run("sweep plan")));
    assert!(!runs_its_row(
        "headwater sweep <other>",
        &run("sweep --help")
    ));
    assert!(runs_its_row("headwater <unknown>", &run("nosuch")));
    assert!(!runs_its_row("headwater <unknown>", &run("check")));
    assert!(!runs_its_row("headwater <unknown>", &run("--version")));
}

#[test]
fn a_manifest_value_reads_as_words_and_bytes() {
    let none: [&str; 0] = [];
    assert_eq!(
        words(r#"new --title "A new answer" x"#, 0),
        ["new", "--title", "A new answer", "x"]
    );
    assert_eq!(words(r#""a\n\t\"\\b""#, 0), ["a\n\t\"\\b"]);
    assert_eq!(words("  ", 0), none);
    assert_eq!(bytes("a%FF%25b", 0), [b'a', 0xFF, b'%', b'b']);
    assert!(matches!(
        step("drop-lines f \"barrier \"", 0),
        Step::DropLines(path, prefix) if path == "f" && prefix == "barrier "
    ));
    assert!(matches!(
        step("write f \"x\\n\"", 0),
        Step::Write(path, content) if path == "f" && content == "x\n"
    ));
}

#[test]
fn a_stream_that_names_where_it_ran_is_found_and_a_difference_is_located() {
    assert!(holds(b"at /tmp/scratch-1/root now", b"/tmp/scratch-1"));
    assert!(!holds(b"at ./root now", b"/tmp/scratch-1"));
    assert!(!holds(b"anything", b""));
    let scratch = Path::new("/nonexistent/headwater-cli-printed-0-x");
    let paths = local_paths(scratch);
    assert!(paths
        .iter()
        .any(|(_, path)| path == b"/nonexistent/headwater-cli-printed-0-x"));
    assert!(paths
        .iter()
        .any(|(_, path)| path == env!("CARGO_MANIFEST_DIR").as_bytes()));
    assert_eq!(
        first_difference(b"a\nb\nc\n", b"a\nc\nb\n"),
        "line 2: blessed `b`, printed `c`"
    );
    assert_eq!(
        first_difference(b"a\n", b"a\nb\n"),
        "line 2: blessed ``, printed `b`"
    );
}

/// A directory of blessed files for one case, removed when it is dropped.
fn planted(label: &str, files: &[(&str, &str)]) -> Scratch {
    let at = Scratch(common::scratch_base().join(format!(
        "headwater-cli-printed-{}-planted-{label}",
        std::process::id()
    )));
    let _ = std::fs::remove_dir_all(&at.0);
    std::fs::create_dir_all(&at.0).expect("the directory is made");
    for (name, content) in files {
        std::fs::write(at.0.join(name), content).expect("the planted file writes");
    }
    at
}

fn printed(out: &str, err: &str, status: &str) -> Ran {
    Ran {
        out: out.as_bytes().to_vec(),
        err: err.as_bytes().to_vec(),
        status: status.to_string(),
    }
}

#[test]
fn each_of_the_three_is_compared_apart_and_a_failure_names_which_moved() {
    let line = stated("x", "headwater check", Outcome::Pass, None, "check");
    let files = [
        ("x.out", "report\n"),
        ("x.err", "statistic\n"),
        ("x.status", "0\n"),
    ];
    let judged = |label: &str, ran: &Ran| {
        let blessed = planted(label, &files);
        let mut named = BTreeSet::new();
        let complaints = judge(&line, ran, &[], &blessed.0, false, &mut named);
        assert_eq!(
            named.iter().map(String::as_str).collect::<Vec<_>>(),
            ["x.err", "x.out", "x.status"]
        );
        complaints
    };
    let none: [&str; 0] = [];

    assert_eq!(
        judged("same", &printed("report\n", "statistic\n", "0\n")),
        none
    );

    // The same bytes on the other stream are two differences, and neither is
    // hidden by the other stream holding what moved.
    let swapped = judged("swapped", &printed("statistic\n", "report\n", "0\n"));
    assert_eq!(swapped.len(), 2, "{swapped:?}");
    assert!(
        swapped[0].starts_with(
            "[x] standard output of `headwater check` moved from fixtures/printed/blessed/x.out"
        ),
        "{swapped:?}"
    );
    assert!(
        swapped[1].starts_with(
            "[x] standard error of `headwater check` moved from fixtures/printed/blessed/x.err"
        ),
        "{swapped:?}"
    );

    let out_only = judged("out", &printed("report \n", "statistic\n", "0\n"));
    assert_eq!(
        out_only.len(),
        1,
        "one trailing blank is a difference: {out_only:?}"
    );
    assert!(out_only[0].contains("x.out"), "{out_only:?}");

    let err_only = judged("err", &printed("report\n", "", "0\n"));
    assert_eq!(err_only.len(), 1, "{err_only:?}");
    assert!(
        err_only[0].contains("standard error") && err_only[0].contains("x.err"),
        "{err_only:?}"
    );

    // A status that moved is the blessed file and the stated outcome both.
    let status = judged("status", &printed("report\n", "statistic\n", "1\n"));
    assert_eq!(status.len(), 2, "{status:?}");
    assert!(
        status[0].contains("is stated as `pass`, which is exit 0")
            && status[0].contains("exited 1"),
        "{status:?}"
    );
    assert!(
        status[1].starts_with(
            "[x] the exit status of `headwater check` moved from fixtures/printed/blessed/x.status"
        ),
        "{status:?}"
    );

    // A file that is not there is a complaint that names it, and never a pass.
    let absent = planted("absent", &files[..2]);
    let missing = judge(
        &line,
        &printed("report\n", "statistic\n", "0\n"),
        &[],
        &absent.0,
        false,
        &mut BTreeSet::new(),
    );
    assert_eq!(missing.len(), 1, "{missing:?}");
    assert!(
        missing[0].starts_with(
            "[x] the exit status has no blessed file: fixtures/printed/blessed/x.status"
        ),
        "{missing:?}"
    );
}

#[test]
fn a_bless_writes_what_moved_and_refuses_what_it_cannot_answer() {
    let line = stated("x", "headwater check", Outcome::Pass, None, "check");
    let read = |at: &Scratch, name: &str| std::fs::read_to_string(at.0.join(name)).ok();
    let local = [(
        "the scratch directory".to_string(),
        b"/scratch/here".to_vec(),
    )];
    let none: [&str; 0] = [];

    // A bless records all three.
    let fresh = planted("bless-fresh", &[]);
    let wrote = judge(
        &line,
        &printed("report\n", "", "0\n"),
        &local,
        &fresh.0,
        true,
        &mut BTreeSet::new(),
    );
    assert_eq!(wrote, none);
    assert_eq!(read(&fresh, "x.out").as_deref(), Some("report\n"));
    assert_eq!(read(&fresh, "x.err").as_deref(), Some(""));
    assert_eq!(read(&fresh, "x.status").as_deref(), Some("0\n"));

    // A refusal is not recorded as the passing line of a row.
    let refused = planted("bless-refused", &[]);
    let outcome = judge(
        &line,
        &printed("", "refused\n", "1\n"),
        &local,
        &refused.0,
        true,
        &mut BTreeSet::new(),
    );
    assert_eq!(outcome.len(), 1, "{outcome:?}");
    assert!(
        outcome[0].contains("A bless does not record this"),
        "{outcome:?}"
    );
    assert_eq!(read(&refused, "x.status"), None, "nothing is written");
    assert_eq!(read(&refused, "x.err"), None, "nothing is written");

    // A stream that names where this run was is not recorded, on a bless or
    // on any other run.
    for bless in [true, false] {
        let leaked = planted(
            &format!("leak-{bless}"),
            &[("x.out", "report\n"), ("x.err", ""), ("x.status", "0\n")],
        );
        let leak = judge(
            &line,
            &printed("report\n", "read /scratch/here/root\n", "0\n"),
            &local,
            &leaked.0,
            bless,
            &mut BTreeSet::new(),
        );
        assert!(
            leak[0]
                .starts_with("[x] standard error of `headwater check` holds the scratch directory"),
            "{leak:?}"
        );
        assert_eq!(
            read(&leaked, "x.err").as_deref(),
            Some(""),
            "the file is left"
        );
    }
}

#[test]
fn a_blessed_file_no_line_names_is_a_complaint_and_a_bless_removes_it() {
    let named: BTreeSet<String> = ["x.out", "x.err", "x.status"]
        .into_iter()
        .map(str::to_string)
        .collect();
    let files = [
        ("x.out", ""),
        ("x.err", ""),
        ("x.status", "0\n"),
        ("gone.out", ""),
    ];
    let at = planted("strays", &files);
    assert_eq!(
        strays(&at.0, &named, false),
        ["fixtures/printed/blessed/gone.out is the blessed file of no line of the manifest"]
    );
    assert!(
        at.0.join("gone.out").is_file(),
        "a plain run removes nothing"
    );
    let none: [&str; 0] = [];
    assert_eq!(strays(&at.0, &named, true), none);
    assert!(!at.0.join("gone.out").exists(), "a bless removes it");
    assert!(at.0.join("x.out").is_file(), "and nothing a line names");
}

#[test]
fn a_line_runs_under_the_stated_environment_and_nothing_else() {
    let scratch = planted("environment", &[]);
    let output = command("env", &scratch.0, &scratch.0)
        .output()
        .expect("`env` runs");
    assert!(output.status.success());
    let printed = String::from_utf8_lossy(&output.stdout).into_owned();
    let mut names: Vec<&str> = printed
        .lines()
        .filter_map(|line| line.split_once('=').map(|(name, _)| name))
        .collect();
    names.sort_unstable();
    // Cargo exports `CARGO_MANIFEST_DIR` and more to this process, so a
    // command that inherited its environment would print them here.
    assert!(std::env::var_os("CARGO_MANIFEST_DIR").is_some());
    assert_eq!(
        names,
        [
            "GIT_AUTHOR_DATE",
            "GIT_AUTHOR_EMAIL",
            "GIT_AUTHOR_NAME",
            "GIT_CEILING_DIRECTORIES",
            "GIT_COMMITTER_DATE",
            "GIT_COMMITTER_EMAIL",
            "GIT_COMMITTER_NAME",
            "GIT_CONFIG_GLOBAL",
            "GIT_CONFIG_NOSYSTEM",
            "HOME",
            "LC_ALL",
            "PATH",
            "TZ",
        ],
        "{printed}"
    );
    assert!(printed.contains("TZ=UTC\n"), "{printed}");
    assert!(printed.contains("LC_ALL=C\n"), "{printed}");
    assert!(
        printed.contains("GIT_COMMITTER_DATE=2026-01-15T00:00:00+00:00\n"),
        "{printed}"
    );
}

#[test]
fn a_committed_lock_needs_a_passing_check_line_over_its_tree_alone() {
    let locked: Vec<String> = trees()
        .into_iter()
        .filter(|tree| {
            fixture()
                .join("trees")
                .join(tree)
                .join(".headwater/taxonomy.lock")
                .is_file()
        })
        .collect();
    assert!(
        locked.contains(&"base".to_string()),
        "the tree most lines run over commits a lock: {locked:?}"
    );
    let line = |tree: &str, run: &str| {
        let mut line = stated(tree, "headwater taxonomy resolve", Outcome::Pass, None, run);
        line.trees = vec![tree.to_string()];
        line
    };
    let held = |change: &dyn Fn(&mut Line)| {
        let mut lines: Vec<Line> = locked
            .iter()
            .map(|tree| line(tree, "taxonomy resolve --root . --check"))
            .collect();
        change(
            lines
                .iter_mut()
                .find(|line| line.label == "base")
                .expect("base"),
        );
        unheld_locks(&lines)
    };
    let none: [&str; 0] = [];
    let base = ["the tree `base` commits a lock, and no passing line runs `taxonomy resolve --check` over it alone with no setup"];

    assert_eq!(unheld_locks(&[]).len(), locked.len());
    assert_eq!(held(&|_| {}), none);
    // A line that writes the lock holds nothing about the committed one.
    assert_eq!(
        held(&|line| line.run.retain(|word| word != "--check")),
        base
    );
    assert_eq!(held(&|line| line.outcome = Outcome::Refuse), base);
    assert_eq!(held(&|line| line.row = "headwater check".to_string()), base);
    // A second tree or a setup step can replace or repair the lock first.
    assert_eq!(held(&|line| line.trees.push("next".to_string())), base);
    assert_eq!(
        held(&|line| line
            .setup
            .push(Step::Remove(".headwater/taxonomy.lock".to_string()))),
        base
    );
}

#[test]
fn a_stream_is_compared_as_bytes_and_not_as_the_text_it_decodes_to() {
    // Neither byte is UTF-8, so a lossy decoding reads both as U+FFFD and
    // calls them equal.
    let line = stated("x", "headwater check", Outcome::Pass, None, "check");
    let blessed = planted("bytes", &[("x.err", ""), ("x.status", "0\n")]);
    std::fs::write(blessed.0.join("x.out"), [b'a', 0xFF, b'\n']).expect("the planted file writes");
    let ran = |out: &[u8]| Ran {
        out: out.to_vec(),
        err: Vec::new(),
        status: "0\n".to_string(),
    };
    let judged = |ran: &Ran| judge(&line, ran, &[], &blessed.0, false, &mut BTreeSet::new());
    let none: [&str; 0] = [];

    assert_eq!(judged(&ran(&[b'a', 0xFF, b'\n'])), none);
    let moved = judged(&ran(&[b'a', 0xFE, b'\n']));
    assert_eq!(moved.len(), 1, "{moved:?}");
    assert!(
        moved[0].starts_with(
            "[x] standard output of `headwater check` moved from fixtures/printed/blessed/x.out"
        ),
        "{moved:?}"
    );
}

#[test]
fn a_setup_step_that_does_not_exit_0_stops_the_line_and_names_itself() {
    let prepared = |label: &str, step: Step| {
        let scratch = planted(label, &[]);
        let root = scratch.0.join("root");
        std::fs::create_dir_all(&root).expect("the root is made");
        std::fs::create_dir_all(scratch.0.join("home")).expect("the home is made");
        let mut line = stated(label, "headwater check", Outcome::Pass, None, "check");
        line.setup = vec![step];
        line.at = 7;
        std::panic::catch_unwind(|| prepare(&line, &scratch.0, &root)).map_err(|panic| {
            panic
                .downcast_ref::<String>()
                .cloned()
                .unwrap_or_else(|| "a panic that is no string".to_string())
        })
    };
    let words = |text: &str| words(text, 0);

    let refused = prepared("setup-refused", Step::Headwater(words("nosuch")))
        .expect_err("a `headwater` step that exits 1 stops the line");
    assert!(
        refused.starts_with("[setup-refused] (manifest line 7) the setup step `")
            && refused.contains("headwater nosuch` did not exit 0:")
            && refused.contains("`nosuch` is not a verb"),
        "{refused}"
    );
    let git = prepared("setup-git", Step::Git(words("nosuch")))
        .expect_err("a `git` step that fails stops the line");
    assert!(
        git.starts_with(
            "[setup-git] (manifest line 7) the setup step `git nosuch` did not exit 0:"
        ),
        "{git}"
    );
    assert_eq!(
        prepared("setup-held", Step::Git(words("init --quiet"))),
        Ok(()),
        "a step that exits 0 lets the line run"
    );
}
