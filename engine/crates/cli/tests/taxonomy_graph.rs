// SPDX-License-Identifier: Apache-2.0
//! `taxonomy graph`, the resolved taxonomy drawn as a Mermaid flowchart.
//!
//! # The defect this target exists for
//!
//! A picture of the taxonomy that somebody drew once, from a list of kinds and
//! lanes written into a script, is right on the day it is drawn and wrong the
//! day after the lock moves. Such a picture passes every case that reads only
//! today's lock. So the decisive case below edits the lock: it adds a purpose,
//! a kind in it and a relation to the abstract kind, removes a relation, and
//! holds the output to follow each edit. A drawing built from a list in code
//! passes the first case here and fails that one.
//!
//! # Where the expected sets come from
//!
//! From the lock, read in this file with the same YAML loader the engine
//! reads it with. No count and no name of a kind, a purpose or a relation of
//! the committed lock is written down here, so a taxonomy release moves the
//! expectation and the output together. The names the second case writes are
//! its own and nothing in the lock carries them.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use headwater_yaml::{Mapping, Value};

/// This repository, whose committed lock every case starts from.
fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// A fresh root holding one file, `.headwater/taxonomy.lock`.
///
/// Keyed on a counter as well as the pid, because cargo runs the cases of a
/// target as threads of one process.
fn root(label: &str, lock: Option<&str>) -> PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let at = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "taxonomy-graph-{}-{}-{label}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(at.join(".headwater")).expect("the root is made");
    if let Some(text) = lock {
        std::fs::write(at.join(".headwater/taxonomy.lock"), text).expect("the lock writes");
    }
    at
}

fn committed() -> String {
    std::fs::read_to_string(repository().join(".headwater/taxonomy.lock"))
        .expect("the committed lock reads")
}

#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: Vec<u8>,
    err: String,
}

impl Ran {
    fn text(&self) -> String {
        String::from_utf8(self.out.clone()).expect("the graph is utf-8")
    }
}

fn graph(at: &Path) -> Ran {
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["taxonomy", "graph", "--root"])
        .arg(at)
        .output()
        .expect("the binary runs");
    Ran {
        code: output.status.code(),
        out: output.stdout,
        err: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// What the lock says the drawing must hold.
#[derive(Debug, Default)]
struct Expected {
    /// `(purpose, kind)` for every concrete kind.
    lanes: BTreeSet<(String, String)>,
    purposes: BTreeSet<String>,
    anchors: BTreeSet<String>,
    abstracts: BTreeSet<String>,
    /// `(relation, from, to)` with no abstract kind at either end.
    drawn: BTreeSet<(String, String, String)>,
    /// `(relation, from, to)` with an abstract kind at one end or both.
    captioned: BTreeSet<(String, String, String)>,
}

fn names(node: Option<&headwater_yaml::Spanned<Value>>) -> Vec<String> {
    match node.map(|n| &n.value) {
        Some(Value::Seq(items)) => items
            .iter()
            .filter_map(|item| item.value.as_scalar().map(|s| s.text.clone()))
            .collect(),
        Some(Value::Scalar(s)) => vec![s.text.clone()],
        _ => Vec::new(),
    }
}

fn map<'a>(of: &'a Mapping, key: &str) -> &'a Mapping {
    of.get(key)
        .and_then(|n| n.value.as_map())
        .unwrap_or_else(|| panic!("the lock carries `{key}`"))
}

fn scalar(of: &Mapping, key: &str) -> Option<String> {
    of.get(key)
        .and_then(|n| n.value.as_scalar())
        .map(|s| s.text.clone())
}

fn expected(lock: &str) -> Expected {
    let loaded = headwater_yaml::load(lock).expect("the lock loads");
    let top = loaded.value.as_map().expect("the lock is a mapping");
    let resolved = map(top, "resolved");
    let mut out = Expected::default();
    for entry in map(resolved, "purposes").iter() {
        out.purposes.insert(entry.key.value.clone());
    }
    for entry in map(resolved, "anchors").iter() {
        out.anchors.insert(entry.key.value.clone());
    }
    let kinds = map(resolved, "kinds");
    for entry in kinds.iter() {
        let kind = entry.value.value.as_map().expect("a kind is a mapping");
        if scalar(kind, "abstract").as_deref() == Some("true") {
            out.abstracts.insert(entry.key.value.clone());
        } else {
            let purpose = scalar(kind, "purpose").expect("a concrete kind names its purpose");
            out.lanes.insert((purpose, entry.key.value.clone()));
        }
    }
    for entry in map(resolved, "relations").iter() {
        let relation = entry.value.value.as_map().expect("a relation is a mapping");
        for from in names(relation.get("from")) {
            for to in names(relation.get("to")) {
                let pair = (entry.key.value.clone(), from.clone(), to.clone());
                if out.abstracts.contains(&from) || out.abstracts.contains(&to) {
                    out.captioned.insert(pair);
                } else {
                    out.drawn.insert(pair);
                }
            }
        }
    }
    out
}

/// The Mermaid identifier of an endpoint, as the output spells it.
fn node(expected: &Expected, name: &str) -> String {
    if expected.anchors.contains(name) {
        format!("anchor_{name}")
    } else {
        format!("kind_{name}")
    }
}

/// The kind lines inside the subgraph of one purpose.
fn lane(text: &str, purpose: &str) -> Vec<String> {
    let open = format!("subgraph purpose_{purpose}[\"{purpose}\"]");
    let mut lines = text.lines().skip_while(|line| line.trim() != open);
    assert!(
        lines.next().is_some(),
        "the lane `{purpose}` is drawn:\n{text}"
    );
    lines
        .take_while(|line| line.trim() != "end")
        .map(|line| line.trim().to_string())
        .collect()
}

/// Every edge line of the drawing.
fn edges(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| line.contains(" -->|"))
        .map(|line| line.trim().to_string())
        .collect()
}

fn caption(relation: &str, from: &str, to: &str) -> String {
    format!("%%   {relation}: {from} -> {to}")
}

/// Hold one drawing to what its lock says, every set in full.
fn holds(text: &str, expected: &Expected) {
    assert!(
        text.starts_with("%%") || text.starts_with("flowchart"),
        "{text}"
    );
    assert!(
        text.lines().any(|l| l == "flowchart LR"),
        "a flowchart:\n{text}"
    );
    for purpose in &expected.purposes {
        let inside = lane(text, purpose);
        for (lane_of, kind) in &expected.lanes {
            let line = format!("kind_{kind}[\"{kind}\"]");
            assert_eq!(
                inside.contains(&line),
                lane_of == purpose,
                "`{kind}` sits in the lane of `{lane_of}` and no other:\n{text}"
            );
        }
    }
    for anchor in &expected.anchors {
        let line = format!("anchor_{anchor}{{{{\"{anchor}\"}}}}");
        assert!(
            text.lines().any(|l| l.trim() == line),
            "the anchor `{anchor}` is drawn, reached or not:\n{text}"
        );
    }
    for abstract_kind in &expected.abstracts {
        assert!(
            !text.contains(&format!("kind_{abstract_kind}")),
            "the abstract kind `{abstract_kind}` is no node:\n{text}"
        );
    }
    let drawn = edges(text);
    for (relation, from, to) in &expected.drawn {
        let line = format!(
            "{} -->|{relation}| {}",
            node(expected, from),
            node(expected, to)
        );
        assert!(drawn.contains(&line), "the edge `{line}` is drawn:\n{text}");
    }
    assert_eq!(
        drawn.len(),
        expected.drawn.len(),
        "one edge for each drawable pair and no other:\n{text}"
    );
    for (relation, from, to) in &expected.captioned {
        let line = caption(relation, from, to);
        assert!(
            text.lines().any(|l| l == line),
            "the pair `{line}` is in the caption:\n{text}"
        );
    }
    let listed = text.lines().filter(|l| l.starts_with("%%   ")).count();
    assert_eq!(
        listed,
        expected.captioned.len(),
        "the caption lists each pair once:\n{text}"
    );
}

#[test]
fn the_drawing_holds_every_kind_anchor_and_pair_the_committed_lock_declares() {
    let lock = committed();
    let expected = expected(&lock);
    let at = root("committed", Some(&lock));
    let ran = graph(&at);
    assert_eq!(ran.code, Some(0), "the graph draws: {ran:?}");
    let text = ran.text();
    holds(&text, &expected);
    // The note reads these, and they are counted here rather than written down.
    eprintln!(
        "concrete kinds {}, purposes {}, anchors {}, drawn pairs {}, captioned pairs {}",
        expected.lanes.len(),
        expected.purposes.len(),
        expected.anchors.len(),
        expected.drawn.len(),
        expected.captioned.len()
    );
    let _ = std::fs::remove_dir_all(&at);
}

/// Put `insert` on the line after the first line equal to `after`.
fn insert_after(text: &str, after: &str, insert: &str) -> String {
    let at = text
        .find(&format!("\n{after}\n"))
        .unwrap_or_else(|| panic!("the lock carries the line `{after}`"))
        + after.len()
        + 2;
    format!("{}{insert}{}", &text[..at], &text[at..])
}

/// Remove the relation `name` from the `relations` block.
fn remove_relation(text: &str, name: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines
        .iter()
        .position(|l| *l == "  relations:")
        .expect("the lock carries relations");
    let first = start
        + 1
        + lines[start + 1..]
            .iter()
            .position(|l| *l == format!("    {name}:"))
            .unwrap_or_else(|| panic!("the lock declares `{name}`"));
    let after = first
        + 1
        + lines[first + 1..]
            .iter()
            .position(|l| !l.starts_with("      "))
            .expect("a line follows the relation");
    let mut kept: Vec<&str> = lines[..first].to_vec();
    kept.extend_from_slice(&lines[after..]);
    kept.join("\n") + "\n"
}

/// Write the digest the edited body now has, so the lock reads as untampered.
fn redigest(text: &str) -> String {
    let loaded = headwater_yaml::load(text).expect("the edited lock loads");
    let top = loaded.value.as_map().expect("a mapping");
    let digest = headwater_lock::digest(&headwater_resolve::render::render(map(top, "resolved")));
    let old = text
        .lines()
        .find(|l| l.starts_with("  digest: sha256:"))
        .expect("the lock header carries a digest");
    text.replacen(old, &format!("  digest: {digest}"), 1)
}

#[test]
fn the_drawing_follows_an_edit_to_the_lock() {
    let committed = committed();
    let before = expected(&committed);
    let abstract_kind = before
        .abstracts
        .iter()
        .next()
        .expect("the committed lock declares an abstract kind")
        .clone();
    // The relation removed: one drawable pair of the committed lock, the first
    // in order, so this case names nothing the lock declares.
    let (gone, gone_from, gone_to) = before
        .drawn
        .iter()
        .find(|(relation, _, _)| before.drawn.iter().filter(|p| &p.0 == relation).count() == 1)
        .expect("a relation with one drawable pair")
        .clone();

    let mut edited = insert_after(
        &committed,
        "  purposes:",
        "    zz_lane:\n      intent: \"a lane no release declares\"\n      answers:\n        - \"what does this case add\"\n",
    );
    edited = insert_after(
        &edited,
        "  kinds:",
        &format!("    zz_kind:\n      is_a: {abstract_kind}\n      purpose: zz_lane\n"),
    );
    edited = insert_after(
        &edited,
        "  relations:",
        &format!(
            "    zz_link:\n      family: association\n      from:\n        - zz_kind\n      to:\n        - {abstract_kind}\n      created_by: author\n"
        ),
    );
    edited = remove_relation(&edited, &gone);
    edited = redigest(&edited);

    let after = expected(&edited);
    assert!(after.lanes.contains(&("zz_lane".into(), "zz_kind".into())));
    assert!(after
        .captioned
        .contains(&("zz_link".into(), "zz_kind".into(), abstract_kind.clone())));

    let at = root("edited", Some(&edited));
    let ran = graph(&at);
    assert_eq!(
        ran.code,
        Some(0),
        "the edited lock reads and draws: {ran:?}"
    );
    let text = ran.text();
    holds(&text, &after);

    assert!(
        lane(&text, "zz_lane").contains(&"kind_zz_kind[\"zz_kind\"]".to_string()),
        "the new lane holds the new kind:\n{text}"
    );
    assert!(
        text.lines()
            .any(|l| l == caption("zz_link", "zz_kind", &abstract_kind)),
        "the new pair is captioned:\n{text}"
    );
    assert!(
        !edges(&text).iter().any(|l| l.contains("|zz_link|")),
        "the new pair is not drawn:\n{text}"
    );
    let line = format!(
        "{} -->|{gone}| {}",
        node(&before, &gone_from),
        node(&before, &gone_to)
    );
    assert!(
        !text.contains(&format!("|{gone}|")),
        "the removed relation `{line}` is gone:\n{text}"
    );
    let _ = std::fs::remove_dir_all(&at);
}

#[test]
fn two_runs_over_one_lock_write_the_same_bytes() {
    let at = root("twice", Some(&committed()));
    let first = graph(&at);
    let second = graph(&at);
    assert_eq!(first.code, Some(0), "{first:?}");
    assert_eq!(
        first.out, second.out,
        "standard output is a function of the lock"
    );
    let _ = std::fs::remove_dir_all(&at);
}

#[test]
fn a_root_with_no_lock_exits_non_zero_with_one_sentence() {
    let at = root("absent", None);
    let ran = graph(&at);
    assert_ne!(ran.code, Some(0), "{ran:?}");
    assert!(ran.out.is_empty(), "nothing is drawn: {ran:?}");
    assert_eq!(ran.err.trim_end().lines().count(), 1, "one line: {ran:?}");
    assert!(
        ran.err.contains("taxonomy.lock"),
        "it names the lock: {ran:?}"
    );
    let _ = std::fs::remove_dir_all(&at);
}

#[test]
fn an_unreadable_lock_exits_non_zero() {
    let at = root("unreadable", Some("lock: [\n"));
    let ran = graph(&at);
    assert_ne!(ran.code, Some(0), "{ran:?}");
    assert!(ran.out.is_empty(), "nothing is drawn: {ran:?}");
    let _ = std::fs::remove_dir_all(&at);
}
