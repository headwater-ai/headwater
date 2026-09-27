// SPDX-License-Identifier: Apache-2.0
//! `headwater route` names a path that the governed scope admits and that no
//! document governs (#953, `docs/interfaces/headwater-route.md`).
//!
//! The write-time hook asks this verb one question per edit. A path in scope
//! with no governing edge is the case the hook used to meet in silence, and a
//! path outside the scope, or one a document governs, must stay silent.

mod common;
use common::Root;

/// The stub root, with one decision that governs `tools/governed.sh`.
fn root(label: &str) -> Root {
    Root::shaped(label, |at| {
        std::fs::write(at.join("tools/governed.sh"), "").expect("the governed file writes");
        std::fs::write(
            at.join("docs/decisions/0002-the-decision-that-governs-a-tool.md"),
            "---\nid: HW-DR-0002\ntitle: The decision that governs a tool\nstatus: \
             current\nstatus_since: 2026-08-01\nlast_verified: 2026-08-01\nsummary: One \
             decision that governs one file under the tools directory.\nprovenance:\n  \
             warrant: asserted\n  agency: human\n  evidence_basis: unevidenced\nrelations:\n  \
             governs:\n    - tools/governed.sh\n---\n\n# The decision that governs a \
             tool\n\n## Context\n\nA fixture.\n\n## Decision\n\nIt governs one \
             file.\n\n## Consequences\n\nThe route names it.\n",
        )
        .expect("the decision writes");
    })
}

fn ungoverned(json: &str) -> &str {
    let start = json
        .find("\"ungoverned\"")
        .unwrap_or_else(|| panic!("no member in {json}"));
    let rest = &json[start..];
    let end = rest.find(']').expect("the array closes");
    &rest[..=end]
}

#[test]
fn a_path_in_scope_that_nothing_governs_is_named_with_the_lines_that_declare_it() {
    let root = root("route-ungoverned");
    let ran = root.run(&["route", "edit", "tools/unrelated.sh", "--json"]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    let member = ungoverned(&ran.out);
    assert!(
        member.contains("\"path\": \"tools/unrelated.sh\""),
        "{member}"
    );
    assert!(member.contains("\"governs\""), "{member}");

    let text = root.run(&["route", "edit", "tools/unrelated.sh"]);
    assert_eq!(text.code, Some(0), "{text:?}");
    assert!(
        text.out
            .contains("tools/unrelated.sh is in the governed scope, and nothing governs it"),
        "{}",
        text.out
    );
    assert!(
        text.out
            .contains("      governs:\n        - tools/unrelated.sh\n"),
        "{}",
        text.out
    );
    assert!(
        !text.out.contains(" — "),
        "the hook selects pointers by an em dash: {}",
        text.out
    );
}

#[test]
fn a_governed_path_and_a_path_outside_the_scope_name_nothing() {
    let root = root("route-governed");
    for path in [
        "tools/governed.sh",
        "engine/crates/stub/tests/unrelated.rs",
        "fix the tools",
    ] {
        let ran = root.run(&["route", "edit", path, "--json"]);
        assert_eq!(ran.code, Some(0), "{ran:?}");
        let member = ungoverned(&ran.out);
        assert!(!member.contains("\"path\""), "{path}: {member}");
        let text = root.run(&["route", "edit", path]);
        assert!(!text.out.contains("governed scope"), "{path}: {}", text.out);
    }
}

/// A word that ends in prose punctuation, or in a line number, names the path
/// without it. A governed file named that way is named as governed, and never
/// as a path that nothing governs with a malformed edge proposed for it.
#[test]
fn a_path_named_with_prose_punctuation_or_a_line_number_is_the_path_itself() {
    let root = root("route-punctuation");
    for task in [
        "Look at tools/governed.sh.",
        "tools/governed.sh:12",
        "(tools/governed.sh),",
        "tools/governed.sh's",
        "tools/governed.sh's.",
        "tools/governed.sh:12:3.",
    ] {
        let ran = root.run(&["route", task, "--json"]);
        assert_eq!(ran.code, Some(0), "{ran:?}");
        let member = ungoverned(&ran.out);
        assert!(!member.contains("\"path\""), "{task}: {member}");
        assert!(
            ran.out
                .contains("\"anchors\": [\n    \"tools/governed.sh\"\n  ]"),
            "{task}: {}",
            ran.out
        );
    }
    // An ungoverned file on the tree is named the same way.
    std::fs::write(root.at.join("tools/other.sh"), "").expect("the file writes");
    let ran = root.run(&["route", "edit", "tools/other.sh:3.", "--json"]);
    let member = ungoverned(&ran.out);
    assert!(member.contains("\"path\": \"tools/other.sh\""), "{member}");

    // Where something would have to come off and no shorter reading is on the
    // tree or reached by an edge, the word names nothing rather than a guess.
    let ran = root.run(&["route", "edit", "tools/unrelated.sh:3.", "--json"]);
    let member = ungoverned(&ran.out);
    assert!(!member.contains("\"path\""), "{member}");
}

/// A path that exists as written is read as written, never as a shorter path
/// that punctuation stripping would make of it. A directory and a URL name no
/// ungoverned file.
#[test]
fn a_path_on_the_tree_as_written_is_never_read_as_a_shorter_one() {
    let root = root("route-as-written");
    for (file, named) in [
        ("tools/a:b.sh", "tools/a:b.sh"),
        ("tools/zz.", "tools/zz."),
        ("tools/it's", "tools/it's"),
    ] {
        std::fs::write(root.at.join(file), "").expect("the file writes");
        let ran = root.run(&["route", "edit", file, "--json"]);
        assert_eq!(ran.code, Some(0), "{ran:?}");
        let member = ungoverned(&ran.out);
        assert!(
            member.contains(&format!("\"path\": \"{named}\"")),
            "{file}: {member}"
        );
        assert_eq!(member.matches("\"path\"").count(), 1, "{file}: {member}");
    }
    for task in [
        "tools/.",
        "tools/",
        "https://github.com/headwater-ai/headwater/blob/main/tools/other.sh",
    ] {
        let ran = root.run(&["route", "edit", task, "--json"]);
        assert_eq!(ran.code, Some(0), "{ran:?}");
        let member = ungoverned(&ran.out);
        assert!(!member.contains("\"path\""), "{task}: {member}");
    }
}

/// The #951 owner ruling, "nobody governs a cache": a path git ignores is not
/// named, though a scope pattern admits it. The same root before `git init`
/// names it, so the case cannot pass on a path the scope never admitted.
#[test]
fn a_path_git_ignores_is_not_named_as_ungoverned() {
    let root = root("route-ignored");
    let path = "tools/__pycache__/stub.cpython-312.pyc";
    let cache = root.at.join(path);
    std::fs::create_dir_all(cache.parent().expect("a parent")).expect("the cache is made");
    std::fs::write(&cache, "").expect("the cache writes");
    std::fs::write(root.at.join(".gitignore"), "__pycache__/\n").expect("the ignore file writes");

    let before = root.run(&["route", "edit", path, "--json"]);
    assert!(
        ungoverned(&before.out).contains(&format!("\"path\": \"{path}\"")),
        "{}",
        before.out
    );

    let init = std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&root.at)
        .status()
        .expect("git runs");
    assert!(init.success());

    let after = root.run(&["route", "edit", path, "--json"]);
    assert_eq!(after.code, Some(0), "{after:?}");
    assert!(
        !ungoverned(&after.out).contains("\"path\""),
        "{}",
        after.out
    );
    let text = root.run(&["route", "edit", path]);
    assert!(!text.out.contains("governed scope"), "{}", text.out);
}

/// The relations are read off the taxonomy. A bundle that declares a second
/// governance relation onto `code_path` puts it beside `governs`, with no
/// change to the engine, so a route that named `governs` alone fails here.
#[test]
fn every_declared_relation_that_governs_the_anchor_kind_is_proposed() {
    let root = Root::shaped("route-relations", |at| {
        common::write_bundle(
            at,
            "zz-rules",
            "add:\n  relations.zz_rules:\n    family: governance\n    from: \
             [governed_document]\n    to: [code_path]\n    cardinality: many\n    created_by: \
             hook\n",
        );
        let declaration = at.join(".headwater/taxonomy.yml");
        let text = std::fs::read_to_string(&declaration).expect("the declaration reads");
        let from = "  bundles: [";
        assert!(text.contains(from), "the declaration lists its bundles");
        std::fs::write(
            &declaration,
            text.replacen(from, "  bundles: [zz-rules, ", 1),
        )
        .expect("the declaration writes");
    });
    let ran = root.run(&["route", "edit", "tools/unrelated.sh", "--json"]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    let member = ungoverned(&ran.out);
    assert!(member.contains("\"governs\""), "{member}");
    assert!(member.contains("\"zz_rules\""), "{member}");
    let text = root.run(&["route", "edit", "tools/unrelated.sh"]);
    assert!(
        text.out
            .contains("      zz_rules:\n        - tools/unrelated.sh\n"),
        "{}",
        text.out
    );
    // Exactly these two. The taxonomy declares other relations whose target
    // admits `code_path`, `traces_to` among them, and none of them governs, so
    // a route that proposed one would propose an edge nobody reads as
    // governance.
    assert_eq!(
        text.out.matches("        - tools/unrelated.sh\n").count(),
        2,
        "{}",
        text.out
    );
}

/// The stub root, with one decision whose `governs` edge onto
/// `tools/stale.sh` records a revision the file no longer has, and one whose
/// edge onto `tools/governed.sh` records none.
fn stale_root(label: &str) -> Root {
    Root::shaped(label, |at| {
        std::fs::write(at.join("tools/stale.sh"), "echo moved\n").expect("the stale file writes");
        std::fs::write(at.join("tools/governed.sh"), "").expect("the governed file writes");
        let decision = |id: &str, file: &str, entry: &str| {
            std::fs::write(
                at.join(format!("docs/decisions/{file}")),
                format!(
                    "---\nid: {id}\ntitle: A decision that governs a tool\nstatus: \
                     current\nstatus_since: 2026-08-01\nlast_verified: 2026-08-01\nsummary: One \
                     decision that governs one file under the tools directory.\nprovenance:\n  \
                     warrant: asserted\n  agency: human\n  evidence_basis: \
                     unevidenced\nrelations:\n  governs:\n{entry}---\n\n# A decision that \
                     governs a tool\n\n## Context\n\nA fixture.\n\n## Decision\n\nIt governs \
                     one file.\n\n## Consequences\n\nThe route names it.\n"
                ),
            )
            .expect("the decision writes");
        };
        decision(
            "HW-DR-0002",
            "0002-the-decision-whose-edge-went-stale.md",
            "    - to: tools/stale.sh\n      verified_revision: sha256:0000\n",
        );
        decision(
            "HW-DR-0003",
            "0003-the-decision-that-recorded-nothing.md",
            "    - tools/governed.sh\n",
        );
    })
}

/// An edge whose recorded revision differs from the one its target has now is
/// named on the pointer, in both formats. An edge that recorded nothing is
/// not, and its pointer carries the member empty (#953).
#[test]
fn a_governing_edge_whose_recorded_revision_moved_is_named_on_its_pointer() {
    let root = stale_root("route-suspect");
    let ran = root.run(&["route", "edit", "tools/stale.sh", "--json"]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    let start = ran
        .out
        .find("\"suspect\"")
        .unwrap_or_else(|| panic!("no suspect member in {}", ran.out));
    let member = &ran.out[start..];
    let member = &member[..=member.find(']').expect("the array closes")];
    assert!(
        member.contains("\"target\": \"tools/stale.sh\""),
        "{member}"
    );
    assert!(member.contains("\"verified\": \"sha256:0000\""), "{member}");
    assert!(member.contains("\"current\": \"sha256:"), "{member}");

    let text = root.run(&["route", "edit", "tools/stale.sh"]);
    assert_eq!(text.code, Some(0), "{text:?}");
    assert!(
        text.out.contains("suspect: tools/stale.sh") && text.out.contains("headwater check"),
        "{}",
        text.out
    );

    let quiet = root.run(&["route", "edit", "tools/governed.sh", "--json"]);
    assert_eq!(quiet.code, Some(0), "{quiet:?}");
    assert!(quiet.out.contains("\"suspect\": []"), "{}", quiet.out);
    let quiet = root.run(&["route", "edit", "tools/governed.sh"]);
    assert!(!quiet.out.contains("suspect"), "{}", quiet.out);
}

/// A word that a glob edge reaches as written, and whose shorter reading is on
/// the tree, names the file on the tree. A `**` edge admits
/// `tools/governed.sh's` as a string, and the route used to name that string
/// as the anchor (#953).
#[test]
fn a_reading_on_the_tree_wins_over_a_longer_one_only_a_glob_edge_reaches() {
    let root = Root::shaped("route-glob-reading", |at| {
        std::fs::write(at.join("tools/governed.sh"), "").expect("the governed file writes");
        std::fs::write(
            at.join("docs/decisions/0002-the-decision-that-governs-the-tools.md"),
            "---\nid: HW-DR-0002\ntitle: The decision that governs the tools\nstatus: \
             current\nstatus_since: 2026-08-01\nlast_verified: 2026-08-01\nsummary: One \
             decision that governs every file under the tools directory.\nprovenance:\n  \
             warrant: asserted\n  agency: human\n  evidence_basis: unevidenced\nrelations:\n  \
             governs:\n    - tools/**\n---\n\n# The decision that governs the tools\n\n## \
             Context\n\nA fixture.\n\n## Decision\n\nIt governs a directory.\n\n## \
             Consequences\n\nThe route names it.\n",
        )
        .expect("the decision writes");
    });
    let ran = root.run(&["route", "edit", "tools/governed.sh's", "header", "--json"]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    let start = ran.out.find("\"anchors\"").expect("the member is there");
    let member = &ran.out[start..];
    let member = &member[..=member.find(']').expect("the array closes")];
    assert!(member.contains("\"tools/governed.sh\""), "{member}");
    assert!(!member.contains("governed.sh's"), "{member}");
}

/// Route reads the bytes of a governed file only for an edge whose revision
/// its answer names, and that is a governing edge of an anchor the task names
/// (#1160). A load that hashed every file a `governs` edge reaches cost this
/// repository's route 0.16 s of CPU against 0.06 s before, by the median
/// user+sys of 10 interleaved runs on 2026-09-28.
///
/// Two governed files here have no interest for the task. One is a named pipe
/// with no writer, so a process that opens it to read waits forever, and one
/// is mode 000, so a process that is not root cannot read it. The task names a
/// third file that a different document governs. A route that opens the pipe
/// never answers, so the case runs the verb under a deadline and fails on it.
#[cfg(unix)]
#[test]
fn route_never_opens_a_governed_file_that_no_anchor_of_the_task_reaches() {
    use std::os::unix::fs::PermissionsExt;

    let root = Root::shaped("route-lazy-revision", |at| {
        std::fs::write(at.join("tools/governed.sh"), "").expect("the governed file writes");
        let fifo = std::process::Command::new("mkfifo")
            .arg(at.join("tools/pipe"))
            .status()
            .expect("mkfifo runs");
        assert!(fifo.success(), "the named pipe is made");
        let locked = at.join("tools/locked.sh");
        std::fs::write(&locked, "echo locked\n").expect("the locked file writes");
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000))
            .expect("the mode is set");
        for (number, title, target) in [
            ("0002", "a tool", "tools/governed.sh"),
            ("0003", "a pipe", "tools/pipe"),
            ("0004", "a locked file", "tools/locked.sh"),
        ] {
            let slug = title.replace(' ', "-");
            std::fs::write(
                at.join(format!(
                    "docs/decisions/{number}-the-decision-that-governs-{slug}.md"
                )),
                format!(
                    "---\nid: HW-DR-{number}\ntitle: The decision that governs {title}\n\
                     status: current\nstatus_since: 2026-08-01\nlast_verified: 2026-08-01\n\
                     summary: One decision that governs one file under the tools \
                     directory.\nprovenance:\n  warrant: asserted\n  agency: human\n  \
                     evidence_basis: unevidenced\nrelations:\n  governs:\n    - {target}\n\
                     ---\n\n# The decision that governs {title}\n\n## Context\n\nA \
                     fixture.\n\n## Decision\n\nIt governs one file.\n\n## \
                     Consequences\n\nThe route names it.\n"
                ),
            )
            .expect("the decision writes");
        }
    });

    let (out, err) = route_under_deadline(
        &root,
        "tools/governed.sh",
        "route opened the named pipe that no anchor of its task reaches, and waited on it",
    );
    assert!(
        out.contains("HW-DR-0002"),
        "the governing decision is named: {out}"
    );
    assert!(!out.contains("HW-DR-0003"), "{out}");
    assert!(!out.contains("HW-DR-0004"), "{out}");
    assert!(!err.contains("revision"), "no revision error: {err}");
}

/// A governing edge that records no `verified_revision` can never go suspect,
/// so route has no revision to compare and reads no byte of what it governs,
/// even where the task names that very file (#1160). The edge here governs a
/// named pipe with no writer, and the task names the pipe. A route that
/// digests the pipe before it asks whether the edge recorded a revision never
/// answers.
#[cfg(unix)]
#[test]
fn route_never_reads_the_bytes_of_a_file_whose_governing_edge_records_no_revision() {
    let root = Root::shaped("route-unrecorded-revision", |at| {
        let fifo = std::process::Command::new("mkfifo")
            .arg(at.join("tools/pipe"))
            .status()
            .expect("mkfifo runs");
        assert!(fifo.success(), "the named pipe is made");
        std::fs::write(
            at.join("docs/decisions/0002-the-decision-that-governs-a-pipe.md"),
            "---\nid: HW-DR-0002\ntitle: The decision that governs a pipe\nstatus: \
             current\nstatus_since: 2026-08-01\nlast_verified: 2026-08-01\nsummary: One \
             decision that governs one named pipe under the tools directory.\nprovenance:\n  \
             warrant: asserted\n  agency: human\n  evidence_basis: unevidenced\nrelations:\n  \
             governs:\n    - tools/pipe\n---\n\n# The decision that governs a pipe\n\n## \
             Context\n\nA fixture.\n\n## Decision\n\nIt governs one file.\n\n## \
             Consequences\n\nThe route names it.\n",
        )
        .expect("the decision writes");
    });
    let (out, _) = route_under_deadline(
        &root,
        "tools/pipe",
        "route digested a governed file whose edge records no revision, and waited on it",
    );
    assert!(
        out.contains("HW-DR-0002"),
        "the governing decision is named: {out}"
    );
}

/// Run `route edit <path> --json` and fail with `hung` if it has not answered
/// in 60 s, which is how a route that opened a named pipe with no writer ends.
#[cfg(unix)]
fn route_under_deadline(root: &Root, path: &str, hung: &str) -> (String, String) {
    use std::time::{Duration, Instant};

    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["route", "edit", path, "--json", "--root"])
        .arg(&root.at)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("the binary runs");
    let deadline = Instant::now() + Duration::from_secs(60);
    let status = loop {
        if let Some(status) = child.try_wait().expect("the child is there") {
            break status;
        }
        if Instant::now() > deadline {
            child.kill().expect("the child stops");
            child.wait().expect("the child is reaped");
            panic!("{hung}");
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    let output = child.wait_with_output().expect("the output is read");
    let out = String::from_utf8_lossy(&output.stdout).into_owned();
    let err = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(status.success(), "{out}\n{err}");
    (out, err)
}

/// The MCP `route` tool answers the bytes `headwater route` prints, for a path
/// git ignores and for one it does not (#1161). The tool once routed with no
/// ignore list, so it named a cache the verb drops as ungoverned, and told an
/// adopter's AI client that a build product was a gap in the corpus. Both
/// paths exist on the tree, because git lists as ignored only a path that is
/// there.
#[test]
fn the_mcp_route_tool_and_the_route_verb_agree_on_an_ignored_path_and_a_tracked_one() {
    let root = root("route-mcp-ignored");
    let ignored = "tools/__pycache__/stub.cpython-312.pyc";
    let tracked = "tools/ungoverned.sh";
    let cache = root.at.join(ignored);
    std::fs::create_dir_all(cache.parent().expect("a parent")).expect("the cache is made");
    std::fs::write(&cache, "").expect("the cache writes");
    std::fs::write(root.at.join(tracked), "").expect("the tool writes");
    std::fs::write(root.at.join(".gitignore"), "__pycache__/\n").expect("the ignore file writes");
    let init = std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&root.at)
        .status()
        .expect("git runs");
    assert!(init.success());

    for (path, named) in [(ignored, false), (tracked, true)] {
        let verb = root.run(&["route", "edit", path]);
        assert_eq!(verb.code, Some(0), "{verb:?}");
        assert_eq!(
            verb.out.contains("governed scope"),
            named,
            "the verb on {path}\n{}",
            verb.out
        );
        let served = root.run_with(
            &["mcp", "--now", "2026-09-28"],
            &format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{{}}}}\n\
                 {{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":\
                 {{\"name\":\"route\",\"arguments\":{{\"task\":\"edit {path}\"}}}}}}\n"
            ),
        );
        assert_eq!(served.code, Some(0), "{served:?}");
        assert_eq!(
            served.out.contains("governed scope"),
            named,
            "the MCP tool on {path}\n{}",
            served.out
        );
        let answer = headwater_yaml::json::Json::string(verb.out.as_str()).render();
        assert!(
            served.out.contains(&answer),
            "the MCP tool answers the bytes of the verb on {path}\nverb:\n{}\nserved:\n{}\n{}",
            verb.out,
            served.out,
            served.err
        );
    }
}

/// The MCP server reads git's ignore list when a route asks for it, and not
/// once at start-up (#1161). A session outlives an edit to `.gitignore`, and
/// the verb reads the list on every run, so a list read once would name a path
/// the verb had stopped naming. One session routes an untracked file, the case
/// ignores it, and the same session routes it again.
#[test]
fn the_mcp_route_tool_reads_an_ignore_rule_written_after_the_session_started() {
    use std::io::{BufRead, Write};
    let root = root("route-mcp-ignored-later");
    let path = "tools/untracked.sh";
    std::fs::write(root.at.join(path), "").expect("the tool writes");
    let init = std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&root.at)
        .status()
        .expect("git runs");
    assert!(init.success());

    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["mcp", "--now", "2026-09-28", "--root"])
        .arg(&root.at)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("the binary runs");
    let mut input = child.stdin.take().expect("the standard input is piped");
    let mut output = std::io::BufReader::new(child.stdout.take().expect("piped"));
    let mut ask = |id: u32, message: String| -> String {
        writeln!(input, "{message}").expect("the request writes");
        input.flush().expect("the request flushes");
        let mut line = String::new();
        output.read_line(&mut line).expect("the response reads");
        assert!(line.contains(&format!("\"id\":{id}")), "{line}");
        line
    };
    let route = |id: u32| {
        format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"method\":\"tools/call\",\"params\":\
             {{\"name\":\"route\",\"arguments\":{{\"task\":\"edit {path}\"}}}}}}"
        )
    };
    ask(
        1,
        "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}".to_owned(),
    );
    let before = ask(2, route(2));
    assert!(
        before.contains("governed scope"),
        "untracked, named: {before}"
    );

    std::fs::write(root.at.join(".gitignore"), "untracked.sh\n").expect("the ignore file writes");
    let after = ask(3, route(3));
    assert!(
        !after.contains("governed scope"),
        "ignored, not named: {after}"
    );

    drop(input);
    assert!(child.wait().expect("the server ends").success());
}
