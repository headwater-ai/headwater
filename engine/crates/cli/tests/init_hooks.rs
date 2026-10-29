// SPDX-License-Identifier: Apache-2.0
//! The hooks that `headwater init --harness` writes, run in a repository that
//! is not this one.
//!
//! # The defect this target exists for
//!
//! [#1871]. The hooks lived only in the checkout of the repository that
//! maintains the engine, and they found the engine under its `engine/`
//! directory, ran its `.githooks/` gate at the end of a turn, and read its
//! `tools/`. A measured hook that depends on something only that repository
//! has measures that repository and not the shipped layer.
//!
//! # What it holds
//!
//! Every case runs the copy that `init --harness` wrote, in a scratch
//! repository that has no `tools/`, no `.githooks/` and no `engine/`, with the
//! binary under test on the `PATH` and nowhere else. The first case is the
//! decisive one: a write the touch hook records, an error on that file that the
//! review hook prints and blocks on, an error on a file the session did not
//! write that it leaves alone, and a second run of the step that writes
//! nothing.
//!
//! [#1871]: https://github.com/headwater-ai/headwater/issues/1871

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// A scratch directory, removed when dropped. `label` names the case, because
/// cargo runs the cases of one target as threads of one process.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Scratch {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-init-hooks-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the scratch directory is made");
        Scratch(at.canonicalize().expect("the scratch directory resolves"))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_headwater")
}

fn headwater(root: &Path, arguments: &[&str]) -> Output {
    Command::new(binary())
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

/// Run git in `at` with no configuration of the host's that could move a byte.
fn git(at: &Path, arguments: &[&str]) {
    let output = Command::new("git")
        .args(["-c", "core.hooksPath=/dev/null"])
        .args(arguments)
        .current_dir(at)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .expect("git runs");
    succeeded(&output, &format!("git {}", arguments.join(" ")));
}

const PACKAGE: &str = "package: acme/hooked
version: 1.0.0
requires_engine: \">=0.1 <2\"

contents:
  taxonomy: taxonomy.yml
";

/// A taxonomy with one kind, one shelf, a `governs` relation onto code paths
/// and one projection, so every hook has something of its own to say.
const TAXONOMY: &str = "taxonomy: acme/hooked
version: 1.0.0

purposes:
  rationale: {intent: explain why a choice was made and what it forecloses}

vocabularies:
  lifecycle_state:
    - {value: draft, role: initial}
    - {value: current, role: live}
    - {value: superseded, role: terminal-retained}

facets:
  status:
    role: state
    values: $vocabularies.lifecycle_state
    required: true
    volatility: mutable
    guidance:
      draft: it is being argued over, and nothing may rely on it
      current: it states what holds now
      superseded: a later decision replaced it
  summary:
    role: scent
    required: true
    volatility: mutable
  title:
    role: name
    required: true
    volatility: stable

regimes:
  lifecycle:
    standard:
      initial: draft
      transitions: {draft: [current], current: [superseded]}
      retain_terminal: true

kinds:
  governed_document:
    abstract: true
    facets: {require: [status, summary, title]}
  answer:
    is_a: governed_document
    purpose: rationale
    lifecycle: standard
    identifier: {scheme: answer_id}

identifier_schemes:
  answer_id: {pattern: \"{namespace}-A-{seq:04d}\", allocation: reconcile-first, namespace: AC}

relations:
  governs:
    family: governance
    from: [answer]
    to: [code_path]
    cardinality: many
    created_by: author

anchors:
  code_path: {resolver: source-tree}

shelves:
  answers:
    path: docs/answers/**
    homogeneous: true
    kind: answer

core:
  requires:
    - facet_role: state

projections:
  - kind: graph_export
    profile: control
    format: json
    output: exports/control.json
";

/// One answer, with its claim. `summary` is left out where `summary` is
/// `None`, which `headwater check --strict` reports as an error.
fn answer(root: &Path, file: &str, id: &str, summary: Option<&str>, relations: &str) {
    let summary = summary.map_or(String::new(), |text| format!("summary: \"{text}\"\n"));
    std::fs::write(
        root.join("docs/answers").join(file),
        format!("---\nid: {id}\ntitle: {id}\nstatus: current\n{summary}{relations}---\n\n# {id}\n"),
    )
    .expect("the answer writes");
    let claims = root.join(".headwater/ids/answer_id");
    std::fs::create_dir_all(&claims).expect("the claim directory is made");
    std::fs::write(claims.join(id), format!("docs/answers/{file}\n")).expect("the claim writes");
}

/// A repository bound to `acme/hooked`, with one answer that governs
/// `src/main.rs`, a generated export, a clean strict check, and the harness
/// set written. It has no `tools/`, no `.githooks/` and no `engine/`.
fn an_adopter(label: &str) -> Scratch {
    let scratch = Scratch::new(label);
    let root = &scratch.0;
    git(root, &["init", "--quiet"]);
    let package = root.join(".headwater/packages/acme-hooked");
    std::fs::create_dir_all(&package).expect("the package directory is made");
    std::fs::write(package.join("package.yml"), PACKAGE).expect("writes");
    std::fs::write(package.join("taxonomy.yml"), TAXONOMY).expect("writes");
    std::fs::create_dir_all(root.join("docs/answers")).expect("made");
    std::fs::create_dir_all(root.join("src")).expect("made");
    std::fs::write(root.join("src/main.rs"), "fn main() {}\n").expect("writes");
    std::fs::write(root.join("src/other.rs"), "fn other() {}\n").expect("writes");
    answer(
        root,
        "main.md",
        "AC-A-0001",
        Some("main is the entry point of the program"),
        "relations:\n  governs:\n    - src/main.rs\n",
    );
    succeeded(
        &headwater(root, &["init", "--package", "acme/hooked"]),
        "`headwater init`",
    );
    succeeded(
        &headwater(root, &["taxonomy", "resolve"]),
        "`headwater taxonomy resolve`",
    );
    succeeded(&headwater(root, &["generate"]), "`headwater generate`");
    succeeded(
        &headwater(root, &["check", "--strict"]),
        "`headwater check --strict` over the adopter",
    );
    succeeded(
        &headwater(root, &["init", "--harness"]),
        "`headwater init --harness`",
    );
    for local in ["tools", ".githooks", "engine"] {
        assert!(!root.join(local).exists(), "the adopter has no {local}");
    }
    scratch
}

/// A `PATH` that holds the binary under test and the system directories, and
/// no other `headwater`.
fn path_with_the_binary() -> String {
    let directory = Path::new(binary())
        .parent()
        .expect("the binary has a directory");
    format!("{}:/usr/bin:/bin", directory.display())
}

/// Run one hook that the step wrote, with `payload` on standard input, the way
/// Claude Code runs it.
fn hook(root: &Path, name: &str, payload: &str) -> Output {
    let mut child = Command::new("sh")
        .arg(root.join(".claude/hooks").join(format!("{name}.sh")))
        .current_dir(root)
        .env_clear()
        .env("PATH", path_with_the_binary())
        .env("HOME", root)
        .env("CLAUDE_PROJECT_DIR", root)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the hook runs");
    child
        .stdin
        .take()
        .expect("a stdin")
        .write_all(payload.as_bytes())
        .expect("the payload writes");
    child.wait_with_output().expect("the hook ends")
}

fn quoted(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

fn tool_payload(root: &Path, event: &str, tool: &str, session: &str, path: &str) -> String {
    format!(
        "{{\"session_id\":{},\"hook_event_name\":{},\"tool_name\":{},\"cwd\":{},\"tool_input\":{{\"file_path\":{}}}}}",
        quoted(session),
        quoted(event),
        quoted(tool),
        quoted(&root.display().to_string()),
        quoted(&root.join(path).display().to_string())
    )
}

fn stop_payload(root: &Path, session: &str, active: bool) -> String {
    format!(
        "{{\"session_id\":{},\"hook_event_name\":\"Stop\",\"stop_hook_active\":{active},\"cwd\":{}}}",
        quoted(session),
        quoted(&root.display().to_string())
    )
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Every file under `.claude/` and `.mcp.json`, with its bytes.
fn the_harness(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn walk(directory: &Path, into: &mut Vec<(PathBuf, Vec<u8>)>) {
        let Ok(entries) = std::fs::read_dir(directory) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, into);
            } else {
                let bytes = std::fs::read(&path).expect("reads");
                into.push((path, bytes));
            }
        }
    }
    let mut found = Vec::new();
    walk(&root.join(".claude"), &mut found);
    found.push((
        root.join(".mcp.json"),
        std::fs::read(root.join(".mcp.json")).expect("the step wrote .mcp.json"),
    ));
    found.sort();
    found
}

#[test]
fn the_shipped_review_hook_blocks_on_an_error_in_a_file_the_session_wrote_and_a_second_run_writes_nothing(
) {
    let scratch = an_adopter("decisive");
    let root = &scratch.0;
    let settings = std::fs::read(root.join(".claude/settings.json")).expect("settings.json");

    // The session writes one answer, and the touch hook records the path.
    let touched = hook(
        root,
        "touch",
        &tool_payload(root, "PreToolUse", "Write", "s1", "docs/answers/bad.md"),
    );
    succeeded(&touched, "the touch hook");
    answer(root, "bad.md", "AC-A-0002", None, "");
    // An error the session did not write.
    answer(root, "other.md", "AC-A-0003", None, "");

    let review = hook(root, "review", &stop_payload(root, "s1", false));
    let stderr = text(&review.stderr);
    assert_eq!(
        review.status.code(),
        Some(2),
        "the review hook blocks the turn:\n{stderr}\n{}",
        text(&review.stdout)
    );
    assert!(
        stderr.contains("docs/answers/bad.md") && stderr.contains("facet.required.missing"),
        "the review hook prints the error on the file the session wrote:\n{stderr}"
    );
    assert!(
        !stderr.contains("docs/answers/other.md"),
        "the review hook prints an error on a file the session did not write:\n{stderr}"
    );

    // A session that wrote nothing, and a stop the harness already blocked
    // once, both end the turn.
    let idle = hook(root, "review", &stop_payload(root, "s2", false));
    succeeded(&idle, "the review hook over a session that wrote nothing");
    assert!(idle.stderr.is_empty() && idle.stdout.is_empty());
    let again = hook(root, "review", &stop_payload(root, "s1", true));
    succeeded(&again, "the review hook on a stop it already blocked");

    // A session that wrote only the file with no error ends its turn, though
    // the tree still fails the check.
    succeeded(
        &hook(
            root,
            "touch",
            &tool_payload(root, "PreToolUse", "Edit", "s3", "src/other.rs"),
        ),
        "the touch hook",
    );
    succeeded(
        &hook(root, "review", &stop_payload(root, "s3", false)),
        "the review hook over a session whose files hold no error",
    );

    // A session that wrote a file whose error a directive suppresses ends its
    // turn: the report prints the error and marks it `not reported`.
    succeeded(
        &hook(
            root,
            "touch",
            &tool_payload(root, "PreToolUse", "Write", "s4", "docs/answers/held.md"),
        ),
        "the touch hook",
    );
    answer(root, "held.md", "AC-A-0004", None, "");
    let held = root.join("docs/answers/held.md");
    let mut bytes = std::fs::read_to_string(&held).expect("reads");
    bytes.push_str("\n<!-- headwater allow=facet.required.missing scope=file until=2099-12-31 reason=accepted_deviation note=the fixture holds a suppressed error -->\n");
    std::fs::write(&held, bytes).expect("writes");
    let report = headwater(root, &["check", "--strict", "--no-color"]);
    assert!(
        text(&report.stdout).contains("docs/answers/held.md")
            && text(&report.stdout).contains("not reported ("),
        "the check prints the suppressed error:\n{}",
        text(&report.stdout)
    );
    let suppressed = hook(root, "review", &stop_payload(root, "s4", false));
    succeeded(
        &suppressed,
        "the review hook over a session whose only error is suppressed",
    );

    // An error that the report places at a line and a column has the head
    // `  <path>:<line>:<column> ✗ error`, and the hook still matches it to the
    // path the session wrote (verify of #1871: a missing facet has no position,
    // so the cases above never reached the strip of `:<line>:<column>`).
    succeeded(
        &hook(
            root,
            "touch",
            &tool_payload(root, "PreToolUse", "Write", "s5", "docs/answers/state.md"),
        ),
        "the touch hook",
    );
    std::fs::write(
        root.join("docs/answers/state.md"),
        "---\nid: AC-A-0005\ntitle: AC-A-0005\nstatus: bogus\nsummary: \"a state no vocabulary admits\"\n---\n\n# AC-A-0005\n",
    )
    .expect("the answer writes");
    std::fs::write(
        root.join(".headwater/ids/answer_id/AC-A-0005"),
        "docs/answers/state.md\n",
    )
    .expect("the claim writes");
    let report = headwater(root, &["check", "--strict", "--no-color"]);
    assert!(
        text(&report.stdout).contains("  docs/answers/state.md:4:9 ✗ error"),
        "the check places the error at a line and a column:\n{}",
        text(&report.stdout)
    );
    let placed = hook(root, "review", &stop_payload(root, "s5", false));
    let stderr = text(&placed.stderr);
    assert_eq!(
        placed.status.code(),
        Some(2),
        "the review hook blocks on a placed error in a file the session wrote:\n{stderr}"
    );
    assert!(
        stderr.contains("docs/answers/state.md:4:9")
            && stderr.contains("facet.value.not_permitted"),
        "the review hook prints the placed error:\n{stderr}"
    );
    assert!(
        !stderr.contains("docs/answers/bad.md"),
        "the review hook prints an error on a file this session did not write:\n{stderr}"
    );

    // The second run writes nothing, and settings.json keeps its bytes.
    let before = the_harness(root);
    let second = headwater(root, &["init", "--harness"]);
    succeeded(&second, "the second run");
    assert!(
        !text(&second.stdout).contains("wrote ") && !text(&second.stdout).contains("added "),
        "the second run wrote a file:\n{}",
        text(&second.stdout)
    );
    assert_eq!(
        std::fs::read(root.join(".claude/settings.json")).expect("reads"),
        settings,
        "settings.json moved"
    );
    assert_eq!(the_harness(root), before, "a file of the set moved");
    succeeded(
        &headwater(root, &["init", "--harness", "--check"]),
        "`--check` over what the step wrote",
    );
}

#[test]
fn the_shipped_configuration_calls_the_shipped_scripts_and_the_binary_on_the_path() {
    let scratch = an_adopter("configuration");
    let root = &scratch.0;
    let settings = std::fs::read_to_string(root.join(".claude/settings.json")).expect("reads");
    for (event, hook) in [
        ("UserPromptSubmit", "intent"),
        ("PreToolUse", "write"),
        ("PreToolUse", "touch"),
        ("PreToolUse", "read"),
        ("PostToolUse", "write"),
        ("Stop", "review"),
    ] {
        assert!(settings.contains(&format!("\"{event}\"")), "{event}");
        assert!(
            settings.contains(&format!("/.claude/hooks/{hook}.sh")),
            "{hook}"
        );
    }
    let mcp = std::fs::read_to_string(root.join(".mcp.json")).expect("reads");
    assert!(mcp.contains("\"command\": \"headwater\""), "{mcp}");
    assert!(mcp.contains("\"mcp\""), "{mcp}");
    assert!(!mcp.contains("engine/"), "{mcp}");
    for script in ["intent", "read", "write", "review", "lib", "touch"] {
        let body = std::fs::read_to_string(root.join(format!(".claude/hooks/{script}.sh")))
            .expect("the script reads");
        assert!(
            body.lines()
                .last()
                .is_some_and(|line| line
                    .starts_with("# installed by headwater init --harness, digest sha256:")),
            "{script} ends with its record"
        );
    }
}

#[test]
fn the_shipped_read_hook_speaks_on_a_governed_path_and_is_silent_on_another() {
    let scratch = an_adopter("read");
    let root = &scratch.0;
    let governed = hook(
        root,
        "read",
        &tool_payload(root, "PreToolUse", "Read", "r1", "src/main.rs"),
    );
    succeeded(&governed, "the read hook on a governed path");
    let out = text(&governed.stdout);
    assert!(
        out.contains("docs/answers/main.md") && out.contains("additionalContext"),
        "{out}"
    );
    let ungoverned = hook(
        root,
        "read",
        &tool_payload(root, "PreToolUse", "Read", "r1", "src/other.rs"),
    );
    succeeded(&ungoverned, "the read hook on an ungoverned path");
    assert!(ungoverned.stdout.is_empty(), "{}", text(&ungoverned.stdout));
}

#[test]
fn the_shipped_write_hook_refuses_a_raw_document_and_names_what_an_edit_affects() {
    let scratch = an_adopter("write");
    let root = &scratch.0;
    let pre = |path: &str| {
        hook(
            root,
            "write",
            &tool_payload(root, "PreToolUse", "Write", "w1", path),
        )
    };

    // A raw new document is refused, and the refusal names the verb.
    let raw = pre("docs/answers/new.md");
    succeeded(&raw, "the write hook on a new document");
    let out = text(&raw.stdout);
    assert!(
        out.contains("\"permissionDecision\":\"deny\"") && out.contains("headwater new"),
        "{out}"
    );

    // A governed code path names the document that governs it, and blocks
    // nothing.
    let governed = pre("src/main.rs");
    let out = text(&governed.stdout);
    assert!(
        out.contains("docs/answers/main.md") && !out.contains("permissionDecision"),
        "{out}"
    );

    // A document names the code paths it governs.
    let dependants = pre("docs/answers/main.md");
    let out = text(&dependants.stdout);
    assert!(
        out.contains("It governs these code paths") && out.contains("src/main.rs"),
        "{out}"
    );

    // A derived file names the command that rebuilds it.
    let derived = pre("exports/control.json");
    let out = text(&derived.stdout);
    assert!(
        out.contains("is a derived artifact") && out.contains("headwater generate"),
        "{out}"
    );

    // A path outside every case is silent.
    let silent = pre("src/other.rs");
    assert!(silent.stdout.is_empty(), "{}", text(&silent.stdout));
}

#[test]
fn the_shipped_intent_hook_routes_a_task_and_is_silent_on_a_notification() {
    let scratch = an_adopter("intent");
    let root = &scratch.0;
    let prompt = |task: &str| {
        hook(
            root,
            "intent",
            &format!(
                "{{\"session_id\":\"i1\",\"hook_event_name\":\"UserPromptSubmit\",\"cwd\":{},\"prompt\":{}}}",
                quoted(&root.display().to_string()),
                quoted(task)
            ),
        )
    };
    let routed = prompt("why is main the entry point of the program");
    succeeded(&routed, "the intent hook");
    assert!(
        text(&routed.stdout).contains("docs/answers/main.md"),
        "{}",
        text(&routed.stdout)
    );
    let notification = prompt("<task-notification>main is the entry</task-notification>");
    succeeded(&notification, "the intent hook on a notification");
    assert!(notification.stdout.is_empty());
}

#[test]
fn a_hook_with_no_binary_on_the_path_does_nothing() {
    let scratch = an_adopter("no-binary");
    let root = &scratch.0;
    for (name, payload) in [
        (
            "write",
            tool_payload(root, "PreToolUse", "Write", "n1", "docs/answers/new.md"),
        ),
        ("review", stop_payload(root, "n1", false)),
    ] {
        let mut child = Command::new("sh")
            .arg(root.join(format!(".claude/hooks/{name}.sh")))
            .current_dir(root)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("CLAUDE_PROJECT_DIR", root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the hook runs");
        child
            .stdin
            .take()
            .expect("a stdin")
            .write_all(payload.as_bytes())
            .expect("writes");
        let output = child.wait_with_output().expect("ends");
        succeeded(&output, &format!("the {name} hook with no binary"));
        assert!(
            output.stdout.is_empty() && output.stderr.is_empty(),
            "{name}"
        );
    }
}
