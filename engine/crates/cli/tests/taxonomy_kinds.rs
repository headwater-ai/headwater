// SPDX-License-Identifier: Apache-2.0
//! `taxonomy kinds`, the kinds a resolved taxonomy keeps, for an agent that is
//! about to draft a document (#1580).
//!
//! # The defect this target exists for
//!
//! A description of the kinds written once, for this repository's taxonomy,
//! reads true here and false in every repository whose taxonomy is a different
//! one. Such a description passes every case that reads only this
//! repository's lock. So the decisive case below writes a lock of another
//! package, with kinds this repository does not have, and holds the verb to
//! print those kinds and none of this repository's. The names of this
//! repository's kinds are read from its committed lock, never written down
//! here, so a taxonomy release moves the expectation with the output.

use std::path::{Path, PathBuf};
use std::process::Command;

/// This repository, whose committed lock supplies the lock header.
fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

fn committed() -> String {
    std::fs::read_to_string(repository().join(".headwater/taxonomy.lock"))
        .expect("the committed lock reads")
}

/// A fresh root holding one file, `.headwater/taxonomy.lock`.
///
/// Keyed on a counter as well as the pid, because cargo runs the cases of a
/// target as threads of one process.
fn root(label: &str, lock: &str) -> PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let at = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "taxonomy-kinds-{}-{}-{label}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(at.join(".headwater")).expect("the root is made");
    std::fs::write(at.join(".headwater/taxonomy.lock"), lock).expect("the lock writes");
    at
}

struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
}

fn kinds(at: &Path, options: &[&str]) -> Ran {
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["taxonomy", "kinds", "--root"])
        .arg(at)
        .args(options)
        .output()
        .expect("the binary runs");
    Ran {
        code: output.status.code(),
        out: String::from_utf8(output.stdout).expect("the report is utf-8"),
        err: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// The body of another package's taxonomy: two concrete kinds this
/// repository does not have, under one abstract parent that requires a facet
/// and a section the children name nowhere themselves.
const FOREIGN: &str = "\
resolved:
  purposes:
    procedure:
      intent: walk an operator through a repair step by step
      answers:
        - how do I bring the widget line back up
    evidence:
      intent: record what happened and what it showed
  facets:
    owner:
      required: false
    severity:
      required: false
    doc_type:
      required: false
  kinds:
    widget_page:
      abstract: true
      facets:
        require:
          - owner
      sections:
        require:
          - Summary
    runbook_widget:
      is_a: widget_page
      purpose: procedure
      sections:
        require:
          - Steps
    incident_widget:
      is_a: widget_page
      purpose: evidence
      facets:
        require:
          - severity
  shelves:
    widget_runbooks:
      path: ops/runbooks/**
      homogeneous: true
      kind: runbook_widget
    widget_incidents:
      path: ops/incidents/**
      homogeneous: true
      kind: incident_widget
    widget_mixed:
      discriminator: doc_type
      homogeneous: false
      kinds:
        - incident_widget
      path: ops/mixed/**
";

/// The committed lock's header, as another package, over [`FOREIGN`].
///
/// The format and the rule set come from the committed lock, so this lock
/// reads on every engine that reads that one. The digest is the one the body
/// has, because `headwater_lock::at` refuses a lock whose digest is wrong.
fn foreign_lock() -> String {
    let committed = committed();
    let mut header = String::new();
    let mut inside = false;
    for line in committed.lines() {
        if line == "lock:" {
            inside = true;
        }
        if !inside {
            continue;
        }
        if line == "  sources:" {
            break;
        }
        let line = match line {
            l if l.starts_with("  package: ") => "  package: acme/widgets",
            l if l.starts_with("  version: ") => "  version: 1.0.0",
            l => l,
        };
        header.push_str(line);
        header.push('\n');
    }
    header.push_str("  sources: []\n");
    redigest(&format!("{header}{FOREIGN}"))
}

fn map<'a>(map: &'a headwater_yaml::Mapping, key: &str) -> &'a headwater_yaml::Mapping {
    map.get(key)
        .and_then(|node| node.value.as_map())
        .unwrap_or_else(|| panic!("`{key}` is a mapping"))
}

/// Write the digest the body now has, so the lock reads as untampered.
fn redigest(text: &str) -> String {
    let loaded = headwater_yaml::load(text).expect("the lock loads");
    let top = loaded.value.as_map().expect("a mapping");
    let digest = headwater_lock::digest(&headwater_resolve::render::render(map(top, "resolved")));
    let old = text
        .lines()
        .find(|l| l.starts_with("  digest: sha256:"))
        .expect("the lock header carries a digest");
    text.replacen(old, &format!("  digest: {digest}"), 1)
}

/// Every kind name the committed lock declares.
fn our_kinds() -> Vec<String> {
    let committed = committed();
    let loaded = headwater_yaml::load(&committed).expect("the committed lock loads");
    let top = loaded.value.as_map().expect("a mapping");
    map(map(top, "resolved"), "kinds")
        .entries()
        .iter()
        .map(|entry| entry.key.value.clone())
        .collect()
}

/// The block of the report that lists one kind.
fn block<'a>(report: &'a str, kind: &str) -> &'a str {
    report
        .split("\n\n")
        .find(|block| block.starts_with(&format!("{kind}\n")))
        .unwrap_or_else(|| panic!("`{kind}` is listed: {report}"))
}

#[test]
fn a_foreign_lock_prints_its_own_kinds_and_none_of_this_repository() {
    let at = root("foreign", &foreign_lock());
    let ran = kinds(&at, &[]);
    assert_eq!(ran.code, Some(0), "{}{}", ran.out, ran.err);
    let report = ran.out;

    assert!(
        report.starts_with(
            "acme/widgets 1.0.0: 2 kind(s) a document can be, and 1 abstract kind(s) no \
             document is: widget_page\n"
        ),
        "{report}"
    );

    let runbook = block(&report, "runbook_widget");
    for line in [
        "  is a      widget_page\n",
        "  purpose   procedure: walk an operator through a repair step by step\n",
        "  answers   how do I bring the widget line back up\n",
        "  shelf     widget_runbooks (ops/runbooks/**)\n",
        // Inherited from the abstract parent.
        "  facets    owner\n",
        // The parent's section first, then the kind's own.
        "  sections  Summary, Steps\n",
        "  draft     headwater new runbook_widget --title \"<title>\"\n",
    ] {
        assert!(runbook.contains(line), "`{line}` in {runbook}");
    }
    // A heterogeneous shelf that does not list the kind does not carry it.
    assert!(!runbook.contains("widget_mixed"), "{runbook}");
    let incident = block(&report, "incident_widget");
    for line in [
        "  purpose   evidence: record what happened and what it showed\n",
        "  shelf     widget_incidents (ops/incidents/**)\n",
        // A heterogeneous shelf names the facet that selects the kind.
        "  shelf     widget_mixed (ops/mixed/**, where `doc_type` names it)\n",
        // The parent's facet first, then the kind's own.
        "  facets    owner, severity\n",
        "  sections  Summary\n",
        "  draft     headwater new incident_widget --title \"<title>\"\n",
    ] {
        assert!(incident.contains(line), "`{line}` in {incident}");
    }
    // Its purpose declares no answers, so no answers line is printed and the
    // when line does not point at one.
    assert!(!incident.contains("  answers "), "{incident}");

    // The abstract parent is never offered as a kind to draft.
    assert!(!report.contains("headwater new widget_page"), "{report}");
    assert!(!report.contains("\nwidget_page\n"), "{report}");

    // Not one kind of this repository's taxonomy is named anywhere.
    let ours = our_kinds();
    assert!(!ours.is_empty(), "the committed lock declares kinds");
    for kind in &ours {
        assert!(
            !report
                .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .any(|word| word == kind),
            "this repository's kind `{kind}` leaked into a foreign report: {report}"
        );
    }

    // The when line is one of the two fixed gap sentences, once per kind, and
    // nothing the engine wrote about this repository. The sentence that sends
    // the reader to the answers is printed only where answers are.
    // A block is cut at the blank line, so its last line has no newline.
    let when = format!("  when      {}", headwater_query::kinds::UNDECLARED_WHEN);
    assert!(runbook.ends_with(&when), "{runbook}");
    let unanswered = format!(
        "  when      {}\n",
        headwater_query::kinds::UNDECLARED_WHEN_UNANSWERED
    );
    assert!(incident.contains(&unanswered), "{incident}");
    assert_eq!(report.matches("  when      ").count(), 2, "{report}");
    let _ = std::fs::remove_dir_all(&at);
}

/// The draft line is in the grammar `headwater new` parses.
///
/// The next component is the verb the line names, so this case runs it with
/// the line's own words. The foreign root holds no consumer declaration, so
/// `new` refuses on that, after the parser has accepted the line. A line the
/// parser refused would never reach that refusal.
#[test]
fn the_draft_line_is_one_headwater_new_parses() {
    let at = root("draft", &foreign_lock());
    let report = kinds(&at, &[]).out;
    let line = block(&report, "runbook_widget")
        .lines()
        .find_map(|line| line.strip_prefix("  draft     headwater "))
        .expect("a draft line")
        .to_string();
    // `new runbook_widget --title "<title>"`, split as a shell would.
    let mut argv: Vec<String> = Vec::new();
    let mut rest = line.as_str();
    while !rest.is_empty() {
        rest = rest.trim_start();
        if let Some(quoted) = rest.strip_prefix('"') {
            let end = quoted.find('"').expect("a closing quote");
            argv.push(quoted[..end].to_string());
            rest = &quoted[end + 1..];
        } else {
            let end = rest.find(' ').unwrap_or(rest.len());
            argv.push(rest[..end].to_string());
            rest = &rest[end..];
        }
    }
    assert_eq!(argv, vec!["new", "runbook_widget", "--title", "<title>"]);
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(&argv)
        .arg("--root")
        .arg(&at)
        .output()
        .expect("the binary runs");
    let err = String::from_utf8_lossy(&output.stderr);
    for refused in ["unexpected argument", "Usage:", "required arguments were not provided"] {
        assert!(!err.contains(refused), "the parser refused the draft line: {err}");
    }
    let _ = std::fs::remove_dir_all(&at);
}

#[test]
fn json_is_the_same_content_as_one_document() {
    let at = root("json", &foreign_lock());
    let ran = kinds(&at, &["--json"]);
    assert_eq!(ran.code, Some(0), "{}{}", ran.out, ran.err);
    let loaded = headwater_yaml::load(&ran.out).expect("the json loads");
    let top = loaded.value.as_map().expect("an object");
    let scalar = |map: &headwater_yaml::Mapping, key: &str| {
        map.get(key)
            .and_then(|node| node.value.as_scalar())
            .map(|scalar| scalar.text.clone())
    };
    assert_eq!(scalar(top, "package").as_deref(), Some("acme/widgets"));
    let names: Vec<String> = top
        .get("kinds")
        .and_then(|node| node.value.as_seq())
        .expect("a kinds array")
        .iter()
        .map(|node| scalar(node.value.as_map().expect("a kind"), "name").expect("a name"))
        .collect();
    assert_eq!(names, vec!["runbook_widget", "incident_widget"]);

    // The heterogeneous shelf keeps the facet that selects the kind.
    let incident = top
        .get("kinds")
        .and_then(|node| node.value.as_seq())
        .and_then(|kinds| kinds.get(1))
        .and_then(|node| node.value.as_map())
        .expect("incident_widget");
    let discriminators: Vec<(Option<String>, Option<String>)> = incident
        .get("shelves")
        .and_then(|node| node.value.as_seq())
        .expect("a shelves array")
        .iter()
        .map(|node| {
            let shelf = node.value.as_map().expect("a shelf");
            (scalar(shelf, "name"), scalar(shelf, "discriminator"))
        })
        .collect();
    assert_eq!(
        discriminators,
        vec![
            // JSON `null`, which this loader reads as the plain scalar.
            (Some("widget_incidents".to_string()), Some("null".to_string())),
            (Some("widget_mixed".to_string()), Some("doc_type".to_string())),
        ]
    );
    let _ = std::fs::remove_dir_all(&at);
}

/// The verb and the MCP tool answer the same bytes over one tree.
///
/// The server is started over this repository, because it walks a corpus and
/// the foreign root has none. The verb reads the same lock.
#[test]
fn the_mcp_tool_answers_the_bytes_of_the_verb() {
    let repo = repository();
    let verb = kinds(&repo, &[]);
    assert_eq!(verb.code, Some(0), "{}", verb.err);

    let mut server = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["mcp", "--now", "2026-10-02", "--root"])
        .arg(&repo)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("the server starts");
    {
        use std::io::Write;
        let stdin = server.stdin.as_mut().expect("stdin");
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"kinds","arguments":{{"format":"text"}}}}}}"#
        )
        .expect("the request writes");
    }
    drop(server.stdin.take());
    let output = server.wait_with_output().expect("the server ends");
    let response = String::from_utf8(output.stdout).expect("utf-8");
    let loaded = headwater_yaml::load(response.trim()).expect("the response loads");
    let text = loaded
        .value
        .as_map()
        .and_then(|top| top.get("result"))
        .and_then(|node| node.value.as_map())
        .and_then(|result| result.get("content"))
        .and_then(|node| node.value.as_seq())
        .and_then(|blocks| blocks.first())
        .and_then(|node| node.value.as_map())
        .and_then(|block| block.get("text"))
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.clone())
        .unwrap_or_else(|| panic!("a text block: {response}"));
    assert_eq!(text, verb.out);
}
