// SPDX-License-Identifier: Apache-2.0
//! A marked file that no projection writes, told two states by two relations.
//!
//! # The defect this target exists for
//!
//! [#1137](https://github.com/headwater-ai/headwater/issues/1137):
//! `lifecycle.state.set_twice` said "`headwater generate` writes `X` onto this
//! page" about a file that carries the generated marker and that no projection
//! declares. The census classifies a file as generated from the marker alone,
//! and generate lists such a file as orphaned and writes nothing onto it. So
//! the finding promised a write that never happens.
//!
//! # Why this case drives the binary
//!
//! The check crate cannot build the projection plan, because the generate
//! crate depends on it. The CLI builds the plan and injects the orphaned paths
//! into the check run. A test inside the check crate proves the rule reads the
//! set. Only a run of the binary proves the set reaches the rule. So this case
//! assembles a scratch repository from this repository's taxonomy, adds a
//! second setter relation so the rule can fire, and hand-places one marked
//! decision that two other decisions tell two states.

use std::path::{Path, PathBuf};
use std::process::Command;

const RULE: &str = "lifecycle.state.set_twice";

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// The overlay addition: a second relation that writes a state, and a state
/// different from the one `supersedes` writes, so a clash is possible.
const RETIRES: &str = concat!(
    "  relations.retires:\n",
    "    family: succession\n",
    "    from: [governed_document]\n",
    "    to: [governed_document]\n",
    "    on_target: {set_state: deprecated}\n",
    "    created_by: agent\n",
);

struct Root {
    at: PathBuf,
}

impl Root {
    /// The label keeps two cases apart: cargo runs them as threads of one
    /// process, so the pid alone names one directory for both.
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-stray-generated-{}-{label}",
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
        std::fs::copy(
            repository.join(".headwater/taxonomy.yml"),
            at.join(".headwater/taxonomy.yml"),
        )
        .expect("the declaration copies");
        let overlay = std::fs::read_to_string(repository.join(".headwater/overlay.yml"))
            .expect("the overlay reads");
        assert!(
            overlay.contains("\nadd:\n"),
            "the overlay opens an `add` block"
        );
        std::fs::write(
            at.join(".headwater/overlay.yml"),
            overlay.replacen("\nadd:\n", &format!("\nadd:\n{RETIRES}\n"), 1),
        )
        .expect("the overlay writes");

        let root = Root { at };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(resolved.code, Some(0), "the fixture resolves\n{resolved:?}");
        root
    }

    /// Scaffold one decision and return its path under the root and its
    /// identifier.
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
        self.run_with(arguments, "")
    }

    /// Run the binary with `input` on its standard input.
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

#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
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

/// Put `lines` into the front matter of `path`, before the block closes.
fn into_front_matter(path: &Path, lines: &str) {
    let text = std::fs::read_to_string(path).expect("the document reads");
    let body = text
        .strip_prefix("---\n")
        .expect("the document opens a block");
    let (block, rest) = body.split_once("\n---\n").expect("the block closes");
    std::fs::write(path, format!("---\n{block}\n{lines}---\n{rest}")).expect("the document writes");
}

/// The pinned date every run here takes, so two verbs report one day.
const NOW: &str = "2026-09-27";

/// A scratch corpus with one hand-placed marked decision that `supersedes`
/// and `retires` tell two states, and the path the check layer names it by.
fn orphan_corpus(label: &str) -> (Root, String) {
    let root = Root::new(label);
    let (page, page_id) = root.decision("A page nobody generates");
    let (superseder, _) = root.decision("A ruling that supersedes the page");
    let (retirer, _) = root.decision("A ruling that retires the page");
    into_front_matter(
        &superseder,
        &format!("relations:\n  supersedes:\n    - {page_id}\n"),
    );
    into_front_matter(
        &retirer,
        &format!("relations:\n  retires:\n    - {page_id}\n"),
    );
    // Hand-placed: the marker names a projection nothing declares.
    let text = std::fs::read_to_string(&page).expect("the page reads");
    std::fs::write(
        &page,
        text.replacen(
            "---\n",
            "---\n\"headwater:generated\": \"stray_pages. Placed by hand.\"\n",
            1,
        ),
    )
    .expect("the page writes");
    let named = format!(
        "docs/decisions/{}",
        page.file_name()
            .expect("the page has a name")
            .to_string_lossy()
    );
    (root, named)
}

/// The findings of this rule in a text report. A finding prints its location
/// on one line and `<rule>: <message>` on the next. The page is the only
/// document two setters reach, so any finding of this rule is a finding at it.
fn fired(report: &str) -> Vec<&str> {
    report
        .lines()
        .filter(|line| line.trim_start().starts_with(&format!("{RULE}: ")))
        .collect()
}

/// The page is marked and no projection writes it, so generate lists it as
/// orphaned and the rule says nothing about a write to it.
#[test]
fn a_marked_page_no_projection_writes_draws_no_set_twice_finding() {
    let (root, named) = orphan_corpus("check");

    let generated = root.run(&["generate", "--check"]);
    let orphans = generated
        .out
        .split_once("marked, and written by no declaration\n")
        .map(|(_, section)| section.split("\n\n").next().unwrap_or(""))
        .unwrap_or("");
    assert!(
        orphans.lines().any(|line| line.trim() == named),
        "generate lists {named} as orphaned\n{generated:?}"
    );

    let checked = root.run(&["check", "--no-cache", "--now", NOW]);
    // The rule ran: the taxonomy has two setters that name two states, so the
    // gate admits its one corpus instance, and a pass is a verdict rather than
    // a rule that never looked.
    assert!(
        checked.out.contains(&format!("1 instances of {RULE}\n")),
        "the rule runs one instance\n{}{}",
        checked.out,
        checked.err
    );
    let fired = fired(&checked.out);
    assert!(fired.is_empty(), "no {RULE} finding at {named}: {fired:?}");
}

/// `check --fix` runs the check layer twice, before and after the write, and
/// the report a caller reads is the second. Neither run names the orphan.
#[test]
fn the_fixer_reports_no_set_twice_finding_at_the_orphan() {
    let (root, named) = orphan_corpus("fix");
    let fixed = root.run(&["check", "--fix", "--no-cache", "--now", NOW]);
    assert!(
        fixed.out.contains(&format!("1 instances of {RULE}\n")),
        "the rule runs one instance\n{}{}",
        fixed.out,
        fixed.err
    );
    let fired = fired(&fixed.out);
    assert!(fired.is_empty(), "no {RULE} finding at {named}: {fired:?}");
}

/// The MCP `check` tool promises the bytes of `headwater check --format
/// <format>` over the same corpus at the same date. On an orphan corpus the
/// two must agree, and neither names the orphan.
#[test]
fn the_mcp_check_tool_and_the_check_verb_agree_on_an_orphan() {
    let (root, named) = orphan_corpus("mcp");
    let checked = root.run(&["check", "--no-cache", "--format", "json", "--now", NOW]);
    assert!(
        checked.out.contains(&format!("\"rule\": \"{RULE}\"")),
        "the JSON report names the rule it ran\n{}{}",
        checked.out,
        checked.err
    );
    assert!(
        !checked.out.contains("onto this page"),
        "no {RULE} finding at {named} from the verb"
    );

    let served = root.run_with(
        &["mcp", "--now", NOW],
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":",
            "{\"name\":\"check\",\"arguments\":{\"format\":\"json\"}}}\n",
        ),
    );
    assert!(
        !served.out.contains("onto this page"),
        "no {RULE} finding at {named} from the MCP tool"
    );
    // The artifact rides in the response as one JSON string, escaped the way
    // the server escapes every string.
    let artifact = headwater_yaml::json::Json::string(checked.out.as_str()).render();
    assert!(
        served.out.contains(&artifact),
        "the MCP tool answers the bytes of the verb ({} bytes from the verb, {} from the server)\n{}",
        checked.out.len(),
        served.out.len(),
        served.err
    );
}
