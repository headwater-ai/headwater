// SPDX-License-Identifier: Apache-2.0
//! A draft that declares `supersedes` at a current decision, over this
//! repository's own taxonomy (#1973).
//!
//! The draft retires nothing yet (HW-DR-0086), so `check --fix` writes nothing.
//! Before #1973 the fix printed only that no finding carried a patch, and
//! `check` said nothing at all, so an author read the edge as done. This file
//! holds what each surface now says: `check` warns against the draft, the
//! account of `check --fix` names the deferral, and the MCP check tool carries
//! the same finding text as the verb.

use std::path::{Path, PathBuf};
use std::process::Command;

const RULE: &str = "lifecycle.state.set_after_initial";

/// The obligation `CT-LIFE-8` of headwater/standard binds the rule to.
/// A finding names it beside the rule, so a control that stopped naming the
/// rule changes the line both cases read.
const OBLIGATION: &str = "OB-LIFE-8";

/// The pinned date every run here takes, so the fix cannot read a clock.
const NOW: &str = "2026-09-27";

/// The output with each run of whitespace folded to one space, so a finding
/// reads as one sentence wherever the report wrapped it.
fn flat(out: &str) -> String {
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

struct Root {
    at: PathBuf,
}

#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
}

impl Root {
    /// The label keeps two cases apart: cargo runs them as threads of one
    /// process, so the pid alone names one directory for both.
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-superseded-at-draft-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");
        let repository = repository();
        copy(
            &repository.join(headwater_resolve::package::PACKAGES),
            &at.join(headwater_resolve::package::PACKAGES),
        );
        copy(
            &repository.join("docs/taxonomies"),
            &at.join("docs/taxonomies"),
        );
        for file in [".headwater/taxonomy.yml", ".headwater/overlay.yml"] {
            std::fs::copy(repository.join(file), at.join(file)).expect("the declaration copies");
        }
        let root = Root { at };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(resolved.code, Some(0), "the fixture resolves\n{resolved:?}");
        root
    }

    /// Scaffold one decision and return its path and its identifier.
    fn decision(&self, title: &str) -> (PathBuf, String) {
        let before = self.decisions();
        let made = self.run(&["new", "decision", "--title", title]);
        assert_eq!(made.code, Some(0), "the decision scaffolds\n{made:?}");
        let path = self
            .decisions()
            .into_iter()
            .find(|path| !before.contains(path))
            .expect("the scaffolder wrote a document");
        let text = std::fs::read_to_string(&path).expect("the document reads");
        let id = text
            .lines()
            .find_map(|line| line.strip_prefix("id: "))
            .expect("the scaffolder wrote an identifier")
            .trim()
            .to_string();
        (path, id)
    }

    fn decisions(&self) -> Vec<PathBuf> {
        match std::fs::read_dir(self.at.join("docs/decisions")) {
            Ok(entries) => entries
                .map(|entry| entry.expect("the entry reads").path())
                .filter(|path| path.extension().is_some_and(|kind| kind == "md"))
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    fn run(&self, arguments: &[&str]) -> Ran {
        let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(arguments)
            .arg("--root")
            .arg(&self.at)
            .output()
            .expect("the binary runs");
        Ran {
            code: output.status.code(),
            out: String::from_utf8_lossy(&output.stdout).into_owned(),
            err: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    fn run_with(&self, arguments: &[&str], input: &str) -> Ran {
        use std::io::Write;
        let mut child = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(arguments)
            .arg("--root")
            .arg(&self.at)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("the binary runs");
        child
            .stdin
            .take()
            .expect("the standard input is piped")
            .write_all(input.as_bytes())
            .expect("the input writes");
        let output = child.wait_with_output().expect("the binary finishes");
        Ran {
            code: output.status.code(),
            out: String::from_utf8_lossy(&output.stdout).into_owned(),
            err: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the directory is there");
    for entry in std::fs::read_dir(from).expect("the fixture directory reads") {
        let entry = entry.expect("the entry reads");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("the file type reads").is_dir() {
            true => copy(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("the fixture copies");
            }
        }
    }
}

/// Write `key: value` into the front matter of `path`, replacing the line
/// that opens with `key:` or adding one before the block closes.
fn set_facet(path: &Path, key: &str, value: &str) {
    let text = std::fs::read_to_string(path).expect("the document reads");
    let body = text
        .strip_prefix("---\n")
        .expect("the document opens a block");
    let (block, rest) = body.split_once("\n---\n").expect("the block closes");
    let prefix = format!("{key}:");
    let mut lines: Vec<String> = block.lines().map(str::to_string).collect();
    match lines.iter().position(|line| line.starts_with(&prefix)) {
        Some(at) => lines[at] = format!("{key}: {value}"),
        None => lines.push(format!("{key}: {value}")),
    }
    std::fs::write(path, format!("---\n{}\n---\n{rest}", lines.join("\n")))
        .expect("the document writes");
}

/// Put `lines` into the front matter of `path`, before the block closes.
fn into_front_matter(path: &Path, lines: &str) {
    let text = std::fs::read_to_string(path).expect("the document reads");
    let body = text
        .strip_prefix("---\n")
        .expect("the document opens a block");
    let (block, rest) = body.split_once("\n---\n").expect("the block closes");
    std::fs::write(path, format!("---\n{block}\n{lines}---\n{rest}")).expect("the document writes");
}

fn facet<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key}: ");
    text.lines()
        .find_map(|line| line.strip_prefix(prefix.as_str()))
        .map(str::trim)
}

/// A draft successor over a current target, the case #1973 reports.
fn draft_over_current(label: &str) -> (Root, PathBuf, PathBuf, String, String) {
    let root = Root::new(label);
    let (target, target_id) = root.decision("A ruling that a later one replaces");
    let (successor, successor_id) = root.decision("The ruling that replaces it");
    set_facet(&target, "status", "current");
    set_facet(&target, "status_since", "2026-07-01");
    set_facet(&successor, "status", "draft");
    set_facet(&successor, "status_since", "2026-08-05");
    into_front_matter(
        &successor,
        &format!("relations:\n  supersedes:\n    - {target_id}\n"),
    );
    (root, target, successor, target_id, successor_id)
}

/// `check` warns against the draft, names the target and the state to set,
/// and `check --fix` writes nothing and says why, in place of the line that
/// no finding carries a patch.
#[test]
fn check_warns_on_the_draft_and_fix_names_the_deferral() {
    let (root, target, successor, target_id, successor_id) = draft_over_current("fix");

    let checked = root.run(&["check", "--no-cache", "--now", NOW]);
    let flat_out = flat(&checked.out);
    let named = format!(
        "{RULE} ({OBLIGATION}): `{successor_id}` declares `supersedes` to `{target_id}`, and \
         the supersession takes effect only once `{successor_id}` leaves `draft`: \
         `{target_id}` stays at `current` until then; if the replacement is settled, set \
         `status` to `current`"
    );
    assert!(
        flat_out.contains(&named),
        "the draft is warned\n{}{}",
        checked.out,
        checked.err
    );
    let file = successor
        .file_name()
        .and_then(|name| name.to_str())
        .expect("a file name");
    assert!(flat_out.contains(file), "{}", checked.out);

    let fixed = root.run(&["check", "--fix", "--no-cache", "--now", NOW]);
    let account = flat(&fixed.err);
    assert!(
        account.contains(&format!(
            "headwater: deferred docs/decisions/{file}: the target's state and the far half \
             are written once it leaves its initial state. `{successor_id}` declares"
        )),
        "the account names the deferral\n{}",
        fixed.err
    );
    assert!(
        account.contains("set `status` to `current`"),
        "the account names the state to set\n{}",
        fixed.err
    );
    assert!(
        !account.contains("no finding of this run carries a patch"),
        "the deferral replaces the silent line\n{}",
        fixed.err
    );
    let text = std::fs::read_to_string(&target).expect("the target reads");
    assert_eq!(facet(&text, "status"), Some("current"), "{text}");
    let text = std::fs::read_to_string(&successor).expect("the successor reads");
    assert_eq!(facet(&text, "status"), Some("draft"), "{text}");
}

/// Once the draft leaves `draft`, the warning is gone and the neighbor rule
/// takes the pair: `lifecycle.state.not_set_by_edge` reports the target.
#[test]
fn the_warning_goes_once_the_draft_is_current() {
    let (root, _target, successor, target_id, _successor_id) = draft_over_current("settled");
    set_facet(&successor, "status", "current");
    let checked = root.run(&["check", "--no-cache", "--now", NOW]);
    let flat_out = flat(&checked.out);
    assert!(
        !flat_out.contains(&format!("{RULE} ({OBLIGATION}):")),
        "{}",
        checked.out
    );
    assert!(
        flat_out.contains(&format!(
            "lifecycle.state.not_set_by_edge (OB-LIFE-6): `{target_id}` stands at `current`"
        )),
        "{}",
        checked.out
    );
}

/// The MCP check tool runs the same runner, so it carries the same finding
/// text as the verb.
#[test]
fn the_mcp_check_tool_carries_the_same_finding() {
    let (root, _target, _successor, target_id, successor_id) = draft_over_current("mcp");
    let served = root.run_with(
        &["mcp", "--now", NOW],
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":",
            "{\"name\":\"check\",\"arguments\":{\"format\":\"sarif\"}}}\n",
        ),
    );
    let message = format!(
        "the supersession takes effect only once `{successor_id}` leaves `draft`: \
         `{target_id}` stays at `current` until then"
    );
    assert!(
        served.out.contains(&message),
        "the MCP tool carries the finding\n{}{}",
        served.out,
        served.err
    );
    let sarif = root.run(&["check", "--no-cache", "--format", "sarif", "--now", NOW]);
    let artifact = headwater_yaml::json::Json::string(sarif.out.as_str()).render();
    assert!(
        served.out.contains(&artifact),
        "the MCP tool answers the bytes of the verb\n{}",
        served.err
    );
}
