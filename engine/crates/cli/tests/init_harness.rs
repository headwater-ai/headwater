// SPDX-License-Identifier: Apache-2.0
//! What `headwater init --harness` writes into a repository that is not this
//! one, and what it refuses.
//!
//! # The defect this target exists for
//!
//! [#1578]. The skills and the maintainer agent that Headwater offers an agent
//! lived only in the checkout of the repository that maintains the engine,
//! and they cited its decisions, its paths and its kinds. An adopter could
//! install none of it, and a copy would have sent the adopter's agent after
//! files the adopter does not have.
//!
//! # What it holds
//!
//! The paths the step writes are the paths the contract's table names, read
//! out of `docs/interfaces/headwater-init.md` rather than typed here. No
//! written byte names an identifier, a path or a kind of this corpus that the
//! adopter's lock does not also declare. No written path is an internal file
//! of this repository. A second run writes nothing, an edited copy is refused
//! and kept, and a copy an earlier release wrote is replaced.
//!
//! The second half runs `--check` over this repository, which installs the set
//! the way an adopter does.
//!
//! [#1578]: https://github.com/headwater-ai/headwater/issues/1578

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The repository this test tree sits in.
fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// A scratch directory, removed when dropped. `label` names the case, because
/// cargo runs the cases of one target as threads of one process.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Scratch {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-init-harness-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the scratch directory is made");
        Scratch(at)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn headwater(root: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(arguments)
        .arg("--root")
        .arg(root)
        .output()
        .expect("the binary runs")
}

fn succeeded(output: &Output, what: &str) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{what} exits 0:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// The paths the contract's table under "The harness step" names: the last
/// code span of each body row.
fn the_paths_the_contract_names() -> BTreeSet<String> {
    let contract = std::fs::read_to_string(repository().join("docs/interfaces/headwater-init.md"))
        .expect("the contract reads");
    let section = contract
        .split("### The harness step")
        .nth(1)
        .expect("the contract has a section for the harness step");
    let section = section.split("\n#").next().unwrap_or(section);
    let paths: BTreeSet<String> = section
        .lines()
        .filter(|line| line.starts_with('|') && !line.starts_with("|---") && line.contains('`'))
        .filter(|line| !line.starts_with("| File "))
        .filter_map(|line| {
            let cells: Vec<&str> = line.trim_matches('|').split('|').collect();
            let last = cells.last()?.trim();
            Some(last.trim_matches('`').to_string())
        })
        .collect();
    assert!(!paths.is_empty(), "the contract's harness table names no path");
    paths
}

/// Every regular file under a directory, relative to `base`.
fn files_under(base: &Path, directory: &Path, into: &mut BTreeSet<String>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files_under(base, &path, into);
        } else {
            into.insert(
                path.strip_prefix(base)
                    .expect("under the base")
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
}

/// The files a harness reads under a root: everything under `.claude/` and
/// `.agents/`.
fn harness_files(root: &Path) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    files_under(root, &root.join(".claude"), &mut found);
    files_under(root, &root.join(".agents"), &mut found);
    found
}

/// Every concrete kind a resolved lock declares, read line by line: a kind is
/// a key at four spaces under `  kinds:`, and abstract when `abstract: true`
/// sits at six spaces under it.
fn concrete_kinds(lock: &str) -> BTreeSet<String> {
    let mut kinds: Vec<(String, bool)> = Vec::new();
    let mut inside = false;
    for line in lock.lines() {
        if line == "  kinds:" {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        if !line.trim().is_empty() && indent <= 2 {
            break;
        }
        if indent == 4 {
            if let Some(name) = line.trim().strip_suffix(':') {
                kinds.push((name.to_string(), false));
            }
        } else if indent == 6 && line.trim() == "abstract: true" {
            if let Some(last) = kinds.last_mut() {
                last.1 = true;
            }
        }
    }
    kinds
        .into_iter()
        .filter(|(_, is_abstract)| !is_abstract)
        .map(|(name, _)| name)
        .collect()
}

/// Every code span of a text, without its backticks.
fn spans(text: &str) -> Vec<&str> {
    text.split('`').skip(1).step_by(2).collect()
}

/// The names of every file under this repository's `.claude/agents`,
/// `.claude/skills` and `.claude/commands` that the contract does not ship:
/// the internal files.
fn internal_names() -> BTreeSet<String> {
    let shipped: BTreeSet<String> = the_paths_the_contract_names()
        .iter()
        .filter_map(|path| shipped_name(path))
        .collect();
    let mut names = BTreeSet::new();
    for directory in [".claude/agents", ".claude/skills", ".claude/commands"] {
        let Ok(entries) = std::fs::read_dir(repository().join(directory)) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let name = name.strip_suffix(".md").unwrap_or(&name).to_string();
            if name != "fixtures.sh" && !shipped.contains(&name) {
                names.insert(name);
            }
        }
    }
    assert!(names.contains("hw-build"), "the internal names include the build stage");
    names
}

/// The name of the skill or the agent a harness path addresses.
fn shipped_name(path: &str) -> Option<String> {
    let segments: Vec<&str> = path.split('/').collect();
    match segments.as_slice() {
        [_, "skills", name, "SKILL.md"] => Some((*name).to_string()),
        [".claude", "agents", file] => file.strip_suffix(".md").map(str::to_string),
        _ => None,
    }
}

/// A repository bound to `acme/answered-export`, which declares none of the
/// kinds of this corpus, and resolved.
fn a_repository_bound_to_another_taxonomy(label: &str) -> Scratch {
    let scratch = Scratch::new(label);
    let root = &scratch.0;
    std::fs::create_dir_all(root.join("docs")).expect("the corpus directory is made");
    std::fs::write(root.join("docs/one.md"), "# a document\n").expect("the document writes");
    let package = root.join(".headwater/packages/acme-answered-export");
    std::fs::create_dir_all(&package).expect("the package directory is made");
    let source = repository()
        .join("engine/crates/cli/fixtures/answered-export/packages/acme-answered-export");
    for file in ["package.yml", "taxonomy.yml"] {
        std::fs::copy(source.join(file), package.join(file)).expect("the package copies");
    }
    succeeded(
        &headwater(root, &["init", "--package", "acme/answered-export"]),
        "`headwater init` over the acme package",
    );
    succeeded(
        &headwater(root, &["taxonomy", "resolve"]),
        "`headwater taxonomy resolve` over the acme package",
    );
    scratch
}

#[test]
fn the_harness_step_writes_the_shipped_set_into_a_repository_bound_to_another_taxonomy_and_no_file_names_this_corpus(
) {
    let scratch = a_repository_bound_to_another_taxonomy("bound");
    let root = &scratch.0;
    let output = headwater(root, &["init", "--harness"]);
    succeeded(&output, "`headwater init --harness` on a bound repository");

    // (a) The paths written are the paths the contract names.
    let written = harness_files(root);
    assert_eq!(written, the_paths_the_contract_names());

    // The step ran alone: the declaration is the one `init` wrote before.
    assert!(root.join(".headwater/taxonomy.lock").is_file());

    // (b) No byte names an identifier or a path of this corpus, or a kind that
    // only this corpus's lock declares.
    let ours = concrete_kinds(
        &std::fs::read_to_string(repository().join(".headwater/taxonomy.lock"))
            .expect("this repository's lock reads"),
    );
    let theirs = concrete_kinds(
        &std::fs::read_to_string(root.join(".headwater/taxonomy.lock"))
            .expect("the adopter's lock reads"),
    );
    assert!(!theirs.is_empty() && !ours.is_empty());
    let only_ours: BTreeSet<&String> = ours.difference(&theirs).collect();
    let mut named = Vec::new();
    for path in &written {
        let text = std::fs::read_to_string(root.join(path)).expect("a written file reads");
        for needle in ["HW-", "docs/decisions/", "docs/spec/", "docs/"] {
            if text.contains(needle) {
                named.push(format!("{path} holds `{needle}`"));
            }
        }
        for span in spans(&text) {
            if only_ours.contains(&span.to_string()) {
                named.push(format!("{path} names the kind `{span}`"));
            }
        }
    }
    assert!(named.is_empty(), "{}", named.join("\n"));

    // (c) No written path is an internal file of this repository.
    let internal = internal_names();
    for path in &written {
        let name = shipped_name(path).expect("every written path addresses a skill or an agent");
        assert!(!internal.contains(&name), "{path} is an internal file");
        assert!(!name.starts_with("hw-"), "{path} is named as internal");
    }
}

#[test]
fn a_second_run_writes_nothing_and_an_edited_copy_is_refused_and_kept() {
    let scratch = a_repository_bound_to_another_taxonomy("rerun");
    let root = &scratch.0;
    succeeded(&headwater(root, &["init", "--harness"]), "the first run");
    let before: Vec<(String, Vec<u8>, std::time::SystemTime)> = harness_files(root)
        .into_iter()
        .map(|path| {
            let at = root.join(&path);
            let bytes = std::fs::read(&at).expect("reads");
            let modified = std::fs::metadata(&at)
                .and_then(|meta| meta.modified())
                .expect("a modified time");
            (path, bytes, modified)
        })
        .collect();

    std::thread::sleep(std::time::Duration::from_millis(20));
    let second = headwater(root, &["init", "--harness"]);
    succeeded(&second, "the second run");
    assert!(
        !String::from_utf8_lossy(&second.stdout).contains("wrote "),
        "the second run wrote a file:\n{}",
        String::from_utf8_lossy(&second.stdout)
    );
    for (path, bytes, modified) in &before {
        let at = root.join(path);
        assert_eq!(&std::fs::read(&at).expect("reads"), bytes, "{path} moved");
        assert_eq!(
            &std::fs::metadata(&at)
                .and_then(|meta| meta.modified())
                .expect("a modified time"),
            modified,
            "{path} was written again"
        );
    }

    // An edited copy is refused, kept byte for byte, and no other file of the
    // set is written in the same run.
    let edited = ".claude/skills/headwater-orient/SKILL.md";
    let mut text = std::fs::read_to_string(root.join(edited)).expect("reads");
    text = text.replacen("# ", "# An adopter's own heading, ", 1);
    std::fs::write(root.join(edited), &text).expect("the edit writes");
    let removed = ".agents/skills/headwater-sweep/SKILL.md";
    std::fs::remove_file(root.join(removed)).expect("the file is removed");

    let refused = headwater(root, &["init", "--harness"]);
    assert_eq!(refused.status.code(), Some(1), "an edited copy is refused");
    assert!(
        String::from_utf8_lossy(&refused.stderr).contains(edited),
        "the refusal names the path:\n{}",
        String::from_utf8_lossy(&refused.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(root.join(edited)).expect("reads"),
        text,
        "the edited copy was overwritten"
    );
    assert!(
        !root.join(removed).exists(),
        "a refused run wrote another file of the set"
    );
}

#[test]
fn a_copy_an_earlier_release_wrote_is_replaced_and_a_copy_with_no_record_is_refused() {
    let scratch = a_repository_bound_to_another_taxonomy("upgrade");
    let root = &scratch.0;
    succeeded(&headwater(root, &["init", "--harness"]), "the first run");
    let path = ".claude/agents/headwater-maintainer.md";
    let current = std::fs::read_to_string(root.join(path)).expect("reads");

    // An earlier release: other text, with a record that agrees with it.
    let earlier_text = "---\nname: headwater-maintainer\n---\n\nAn earlier text.\n";
    let earlier = format!(
        "{earlier_text}<!-- installed by headwater init --harness, digest sha256:{} -->\n",
        headwater_hash::hex(earlier_text.as_bytes())
    );
    std::fs::write(root.join(path), &earlier).expect("writes");
    succeeded(&headwater(root, &["init", "--harness"]), "the upgrade run");
    assert_eq!(
        std::fs::read_to_string(root.join(path)).expect("reads"),
        current,
        "the earlier copy is replaced by this release"
    );

    // The same text with no record is somebody else's file.
    std::fs::write(root.join(path), earlier_text).expect("writes");
    let refused = headwater(root, &["init", "--harness"]);
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(
        std::fs::read_to_string(root.join(path)).expect("reads"),
        earlier_text
    );
}

#[test]
fn the_harness_check_passes_on_this_repository_and_fails_on_one_changed_byte() {
    let output = headwater(&repository(), &["init", "--harness", "--check"]);
    succeeded(
        &output,
        "`headwater init --harness --check` over this repository",
    );

    // A scratch copy of the installed set, with one byte of one file changed.
    let scratch = Scratch::new("check");
    let root = &scratch.0;
    for path in the_paths_the_contract_names() {
        let to = root.join(&path);
        std::fs::create_dir_all(to.parent().expect("a parent")).expect("made");
        std::fs::copy(repository().join(&path), &to).expect("the installed copy copies");
    }
    succeeded(
        &headwater(root, &["init", "--harness", "--check"]),
        "`--check` over an exact copy",
    );
    let changed = ".claude/skills/headwater-orient/SKILL.md";
    let mut bytes = std::fs::read(root.join(changed)).expect("reads");
    let last = bytes.len() - 2;
    bytes[last] = if bytes[last] == b'x' { b'y' } else { b'x' };
    std::fs::write(root.join(changed), &bytes).expect("writes");
    let before = std::fs::read(root.join(changed)).expect("reads");

    let refused = headwater(root, &["init", "--harness", "--check"]);
    assert_eq!(refused.status.code(), Some(1), "one changed byte fails `--check`");
    assert!(
        String::from_utf8_lossy(&refused.stderr).contains(changed),
        "the failure names the path:\n{}",
        String::from_utf8_lossy(&refused.stderr)
    );
    assert_eq!(
        std::fs::read(root.join(changed)).expect("reads"),
        before,
        "`--check` wrote a file"
    );
    assert!(
        !root.join(".headwater").exists(),
        "`--check` bound the repository"
    );
}
