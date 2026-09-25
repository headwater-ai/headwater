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

use std::collections::{BTreeMap, BTreeSet};
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
    graph_with(at, &[])
}

fn graph_with(at: &Path, options: &[&str]) -> Ran {
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["taxonomy", "graph", "--root"])
        .arg(at)
        .args(options)
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
    /// kind -> the kind it is declared `is_a`, for every kind that names one.
    parents: BTreeMap<String, String>,
    /// relation -> family.
    families: BTreeMap<String, String>,
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
    for entry in map(resolved, "purposes") {
        out.purposes.insert(entry.key.value.clone());
    }
    for entry in map(resolved, "anchors") {
        out.anchors.insert(entry.key.value.clone());
    }
    let kinds = map(resolved, "kinds");
    for entry in kinds {
        let kind = entry.value.value.as_map().expect("a kind is a mapping");
        if let Some(parent) = scalar(kind, "is_a") {
            out.parents.insert(entry.key.value.clone(), parent);
        }
        if scalar(kind, "abstract").as_deref() == Some("true") {
            out.abstracts.insert(entry.key.value.clone());
        } else {
            let purpose = scalar(kind, "purpose").expect("a concrete kind names its purpose");
            out.lanes.insert((purpose, entry.key.value.clone()));
        }
    }
    for entry in map(resolved, "relations") {
        let relation = entry.value.value.as_map().expect("a relation is a mapping");
        if let Some(family) = scalar(relation, "family") {
            out.families.insert(entry.key.value.clone(), family);
        }
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

/// Whether `line` declares the node of `kind`, with or without the lines that
/// name a relation the kind has to itself.
fn declares(line: &str, kind: &str) -> bool {
    let line = line.trim();
    line == format!("kind_{kind}[\"{kind}\"]")
        || line.starts_with(&format!("kind_{kind}[\"{kind}<br/>"))
}

/// A relation from a node to itself is no edge, and it is a line on the node.
fn loops_hold(text: &str, expected: &Expected, pairs: &BTreeSet<(String, String, String)>) {
    for (relation, from, to) in pairs.iter().filter(|(_, from, to)| from == to) {
        let edge = format!(
            "{} -->|{relation}| {}",
            node(expected, from),
            node(expected, to)
        );
        assert!(
            !text.lines().any(|l| l.trim() == edge),
            "the self-pair `{edge}` is no edge:\n{text}"
        );
        let open = format!("{}[", node(expected, from));
        let declaration = text
            .lines()
            .find(|l| l.trim().starts_with(&open))
            .unwrap_or_else(|| panic!("the node `{from}` is declared:\n{text}"));
        assert!(
            declaration.contains(&format!("\u{21bb} {relation}")),
            "the node `{from}` carries a line for `{relation}`:\n{text}"
        );
    }
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
            assert_eq!(
                inside.iter().any(|l| declares(l, kind)),
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
    let between: Vec<_> = expected.drawn.iter().filter(|(_, f, t)| f != t).collect();
    for (relation, from, to) in &between {
        let line = format!(
            "{} -->|{relation}| {}",
            node(expected, from),
            node(expected, to)
        );
        assert!(drawn.contains(&line), "the edge `{line}` is drawn:\n{text}");
    }
    assert_eq!(
        drawn.len(),
        between.len(),
        "one edge for each drawable pair between two nodes and no other:\n{text}"
    );
    loops_hold(text, expected, &expected.drawn);
    assert!(
        !text.lines().any(|l| l.starts_with("%%   ")) && !text.contains("note_abstract"),
        "no pair is listed and no note is drawn:\n{text}"
    );
    assert_eq!(
        text.contains("--view abstract"),
        !expected.captioned.is_empty(),
        "the pointer to the abstract view is there when a pair is left out, and only then:\n{text}"
    );
}

/// Whether `kind` is declared, at any depth, under an abstract kind.
fn under_abstract(expected: &Expected, kind: &str) -> bool {
    let mut seen = BTreeSet::new();
    let mut at = kind.to_string();
    while seen.insert(at.clone()) {
        let Some(parent) = expected.parents.get(&at) else {
            return false;
        };
        if expected.abstracts.contains(parent) {
            return true;
        }
        at = parent.clone();
    }
    false
}

/// Hold the abstract view to what its lock says, every set in full.
fn holds_abstract(text: &str, expected: &Expected) {
    assert!(
        text.lines().any(|l| l == "flowchart LR"),
        "a flowchart:\n{text}"
    );
    for abstract_kind in &expected.abstracts {
        let open = format!("kind_{abstract_kind}[\"{abstract_kind}<br/>abstract");
        assert!(
            text.lines().any(|l| l.trim().starts_with(&open)),
            "the abstract kind `{abstract_kind}` is a node:\n{text}"
        );
    }
    let members: BTreeSet<&(String, String)> = expected
        .lanes
        .iter()
        .filter(|(_, kind)| under_abstract(expected, kind))
        .collect();
    for (lane_of, kind) in &expected.lanes {
        let member = members.contains(&(lane_of.clone(), kind.clone()));
        assert_eq!(
            text.lines().any(|l| declares(l, kind)),
            member,
            "`{kind}` is drawn when it sits under an abstract kind, and only then:\n{text}"
        );
    }
    for purpose in &expected.purposes {
        let wanted: Vec<&str> = members
            .iter()
            .filter(|(lane_of, _)| lane_of == purpose)
            .map(|(_, kind)| kind.as_str())
            .collect();
        let open = format!("subgraph purpose_{purpose}[");
        if wanted.is_empty() {
            assert!(!text.contains(&open), "no empty lane `{purpose}`:\n{text}");
        } else {
            let inside = lane(text, purpose);
            for kind in wanted {
                assert!(
                    inside.iter().any(|l| declares(l, kind)),
                    "`{kind}` sits in the lane of `{purpose}`:\n{text}"
                );
            }
        }
    }
    let mut arrows = 0;
    for (kind, parent) in &expected.parents {
        let named =
            expected.abstracts.contains(kind) || members.iter().any(|(_, member)| member == kind);
        if named {
            arrows += 1;
            let line = format!("{} -.-> {}", node(expected, kind), node(expected, parent));
            assert!(
                text.lines().any(|l| l.trim() == line),
                "the arrow `{line}` is drawn:\n{text}"
            );
        }
    }
    let drawn_arrows = text
        .lines()
        .filter(|l| l.trim_start().starts_with("kind_") && l.contains(" -.-> "))
        .count();
    assert_eq!(
        drawn_arrows, arrows,
        "one is_a arrow for each kind that names a parent:\n{text}"
    );
    let between: Vec<_> = expected
        .captioned
        .iter()
        .filter(|(_, f, t)| f != t)
        .collect();
    for (relation, from, to) in &between {
        let line = format!(
            "{} -->|{relation}| {}",
            node(expected, from),
            node(expected, to)
        );
        assert!(
            text.lines().any(|l| l.trim() == line),
            "the edge `{line}` is drawn:\n{text}"
        );
    }
    loops_hold(text, expected, &expected.captioned);
    for (relation, from, to) in &expected.drawn {
        let line = format!(
            "{} -->|{relation}| {}",
            node(expected, from),
            node(expected, to)
        );
        assert!(
            !text.lines().any(|l| l.trim() == line),
            "the edge `{line}` belongs to the concrete view:\n{text}"
        );
    }
    assert_eq!(
        real_edges(text).len(),
        between.len() + arrows,
        "one edge for each pair with an abstract end and two nodes, and each arrow:\n{text}"
    );
    let reached: BTreeSet<&String> = expected
        .captioned
        .iter()
        .flat_map(|(_, from, to)| [from, to])
        .filter(|end| expected.anchors.contains(*end))
        .collect();
    for anchor in &expected.anchors {
        let line = format!("anchor_{anchor}{{{{\"{anchor}\"}}}}");
        assert_eq!(
            text.lines().any(|l| l.trim() == line),
            reached.contains(anchor),
            "the anchor `{anchor}` is drawn where a pair reaches it, and only then:\n{text}"
        );
    }
}

/// Every edge line of the drawing itself, in order, with no edge of a key.
fn real_edges(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| (l.contains(" -->|") || l.contains(" -.->")) && !l.starts_with("key_"))
        .map(str::to_string)
        .collect()
}

/// The stroke each `linkStyle` line gives each edge index.
fn strokes(text: &str) -> BTreeMap<usize, String> {
    let mut out = BTreeMap::new();
    for line in text.lines() {
        let Some(rest) = line.trim().strip_prefix("linkStyle ") else {
            continue;
        };
        let (indices, style) = rest
            .split_once(" stroke:")
            .expect("a linkStyle names a stroke");
        for index in indices.split(',') {
            out.insert(index.parse().expect("an index"), style.to_string());
        }
    }
    out
}

/// Hold the key of one drawing to the edges it explains: a family is keyed
/// exactly when an edge carries it, in the stroke that edge carries.
fn key_agrees(text: &str, expected: &Expected) {
    assert!(text.contains("subgraph key[\"key\"]"), "a key:\n{text}");
    let strokes = strokes(text);
    let all: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| l.contains(" -->|") || l.contains(" -.->"))
        .collect();
    let label = |line: &str| -> String {
        let after = line.split_once('|').expect("an edge label").1;
        after.split_once('|').expect("a closed label").0.to_string()
    };
    // Each loop line of a node: the family of its relation and the color it has.
    let loop_lines: Vec<(String, String)> = text
        .split("<span style='color:")
        .skip(1)
        .map(|chunk| {
            let color = chunk.split('\'').next().expect("a color").to_string();
            let relation = chunk
                .split_once("\u{21bb} ")
                .expect("a loop line names its relation")
                .1
                .split("</span>")
                .next()
                .expect("a closed span")
                .to_string();
            let family = expected
                .families
                .get(&relation)
                .unwrap_or_else(|| panic!("`{relation}` has a family"))
                .clone();
            (family, color)
        })
        .collect();
    let mut keyed = BTreeSet::new();
    for (index, line) in all.iter().enumerate() {
        if !line.starts_with("key_a") {
            continue;
        }
        let name = label(line);
        keyed.insert(name.clone());
        let stroke = strokes.get(&index).expect("a key edge is styled");
        let mut seen = 0;
        for (at, other) in all.iter().enumerate() {
            if other.starts_with("key_a") {
                continue;
            }
            let carries = if name == "is_a" {
                other.contains(" -.->")
            } else {
                other.contains(" -->|") && expected.families.get(&label(other)) == Some(&name)
            };
            if carries {
                seen += 1;
                assert_eq!(
                    strokes.get(&at),
                    Some(stroke),
                    "the key draws `{name}` in the stroke of its edges:\n{text}"
                );
            }
        }
        for (family, color) in &loop_lines {
            if *family == name {
                seen += 1;
                assert_eq!(
                    color, stroke,
                    "the key draws `{name}` in the color of its loop lines:\n{text}"
                );
            }
        }
        assert!(
            seen > 0,
            "the key names `{name}` and no edge or loop line carries it:\n{text}"
        );
    }
    for (family, _) in &loop_lines {
        assert!(
            keyed.contains(family),
            "the key names `{family}`, which a loop line carries:\n{text}"
        );
    }
    for other in all.iter().filter(|l| !l.starts_with("key_a")) {
        let name = if other.contains(" -.->") {
            "is_a".to_string()
        } else {
            expected
                .families
                .get(&label(other))
                .expect("a relation has a family")
                .clone()
        };
        assert!(
            keyed.contains(&name),
            "the key names `{name}`, which an edge carries:\n{text}"
        );
    }
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
        &format!(
            "    zz_kind:\n      is_a: {abstract_kind}\n      purpose: zz_lane\n    zz_deep:\n      is_a: zz_kind\n      purpose: zz_lane\n    zz_loose:\n      purpose: zz_lane\n"
        ),
    );
    edited = insert_after(
        &edited,
        "  relations:",
        &format!(
            "    zz_link:\n      family: association\n      from:\n        - zz_kind\n      to:\n        - {abstract_kind}\n      created_by: author\n"
        ),
    );
    // Two relations from a node to itself, in a family that no edge carries: one
    // on the new kind and one on the abstract kind.
    edited = insert_after(
        &edited,
        "  relations:",
        &format!(
            "    zz_self:\n      family: zz_family\n      from:\n        - zz_kind\n      to:\n        - zz_kind\n      created_by: author\n    zz_abstract_self:\n      family: zz_family\n      from:\n        - {abstract_kind}\n      to:\n        - {abstract_kind}\n      created_by: author\n"
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
        lane(&text, "zz_lane")
            .iter()
            .any(|l| declares(l, "zz_kind")),
        "the new lane holds the new kind:\n{text}"
    );
    assert!(
        !edges(&text).iter().any(|l| l.contains("|zz_link|")),
        "the new pair is not in the concrete view:\n{text}"
    );
    let abstract_text = graph_with(&at, &["--view", "abstract"]).text();
    holds_abstract(&abstract_text, &after);
    for view in ["concrete", "abstract"] {
        let keyed = graph_with(&at, &["--view", view, "--legend"]).text();
        key_agrees(&keyed, &after);
        assert!(
            keyed.contains("\u{21bb} zz_self") || view == "abstract",
            "the concrete view writes the self-pair on its node:\n{keyed}"
        );
        assert!(
            keyed.contains("\u{21bb} zz_abstract_self") || view == "concrete",
            "the abstract view writes the self-pair on the abstract node:\n{keyed}"
        );
    }
    assert!(
        lane(&abstract_text, "zz_lane")
            .iter()
            .any(|l| declares(l, "zz_kind")),
        "the abstract view carries the new lane:\n{abstract_text}"
    );
    assert!(
        !abstract_text.contains("kind_zz_loose"),
        "a kind under no abstract kind is not in the abstract view:\n{abstract_text}"
    );
    for line in [
        "kind_zz_deep -.-> kind_zz_kind".to_string(),
        format!("kind_zz_kind -.-> kind_{abstract_kind}"),
        format!("kind_zz_kind -->|zz_link| kind_{abstract_kind}"),
    ] {
        assert!(
            abstract_text.lines().any(|l| l.trim() == line),
            "the abstract view draws `{line}`:\n{abstract_text}"
        );
    }
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

#[test]
fn the_abstract_view_holds_every_abstract_kind_and_pair_the_committed_lock_declares() {
    let lock = committed();
    let expected = expected(&lock);
    let at = root("abstract", Some(&lock));
    let ran = graph_with(&at, &["--view", "abstract"]);
    assert_eq!(ran.code, Some(0), "the abstract view draws: {ran:?}");
    holds_abstract(&ran.text(), &expected);
    let _ = std::fs::remove_dir_all(&at);
}

#[test]
fn the_default_view_is_the_concrete_one() {
    let at = root("default", Some(&committed()));
    let plain = graph(&at);
    let named = graph_with(&at, &["--view", "concrete"]);
    assert_eq!(plain.code, Some(0), "{plain:?}");
    assert_eq!(
        plain.out, named.out,
        "no option and `--view concrete` agree"
    );
    let _ = std::fs::remove_dir_all(&at);
}

#[test]
fn a_view_outside_the_two_is_refused() {
    let at = root("view", Some(&committed()));
    let ran = graph_with(&at, &["--view", "everything"]);
    assert_ne!(ran.code, Some(0), "{ran:?}");
    assert!(ran.out.is_empty(), "nothing is drawn: {ran:?}");
    let _ = std::fs::remove_dir_all(&at);
}

#[test]
fn the_key_is_optional_and_names_what_the_drawing_uses() {
    let lock = committed();
    let expected = expected(&lock);
    let at = root("key", Some(&lock));
    for view in ["concrete", "abstract"] {
        let bare = graph_with(&at, &["--view", view]).text();
        let keyed = graph_with(&at, &["--view", view, "--legend"]);
        assert_eq!(keyed.code, Some(0), "{keyed:?}");
        let text = keyed.text();
        assert!(
            !bare.contains("subgraph key["),
            "no key without the option:\n{bare}"
        );
        key_agrees(&text, &expected);
        let stripped: Vec<&str> = bare.lines().collect();
        assert_eq!(
            real_edges(&text),
            real_edges(&bare),
            "the key adds no edge of the drawing itself ({view})"
        );
        assert!(!stripped.is_empty());
    }
    let _ = std::fs::remove_dir_all(&at);
}
