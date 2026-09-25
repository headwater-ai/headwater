// SPDX-License-Identifier: Apache-2.0
//! `headwater taxonomy graph`: the resolved taxonomy as a Mermaid flowchart.
//!
//! The drawing is a function of the `resolved` block of the lock and of
//! nothing else. Every lane, node and edge comes out of that block, so a
//! release that adds a kind or moves a relation moves the drawing with it, and
//! no list of kinds, purposes or relations is written down in this file.
//! [HW-DR-0082](../../../../docs/decisions/0082-the-resolved-taxonomy-is-drawn-by-a-verb-that-prints-mermaid-and-writes-no-file.md)
//! is why this is a verb that prints and not a projection that writes.
//!
//! **Mermaid, because the layout belongs to the renderer.** GitHub, MkDocs and
//! most Markdown viewers lay out a Mermaid flowchart, so this file carries no
//! layout and no position.
//!
//! **Two views, because one drawing cannot hold both questions.** The
//! concrete view answers which kind can relate to which. The abstract view
//! answers what an abstract kind gives the kinds under it. Both read
//! `abstract: true` and `is_a` from the lock, and neither names a kind.
//!
//! The concrete view carries four rules.
//!
//! - One subgraph for each purpose, holding each concrete kind whose purpose
//!   it is. A kind that names no purpose inherits the purpose of the nearest
//!   kind it is declared `is_a`.
//! - One hexagon for each anchor, drawn whether or not a relation reaches it.
//! - One edge for each `(from, to)` pair of each relation, labeled with the
//!   relation's name and styled by its family.
//! - **No edge where either end is an abstract kind.** An abstract kind stands
//!   for every concrete kind declared under it, so an edge to it would read as
//!   an edge to a node nobody can write. The abstract view draws those pairs.
//!
//! The abstract view draws each abstract kind as a dashed node, each kind
//! declared under one in the lane of its purpose, an `is_a` arrow from each of
//! those kinds to the kind it is declared under, and the pairs of relations
//! that have an abstract kind at one end. An anchor appears only where one of
//! those pairs reaches it.
//!
//! **A relation from a kind to itself is no edge.** Mermaid draws such an edge
//! as a long detour that two of them turn into a knot. Each view writes it on
//! the node as one line that starts with `↻`, in the color that its
//! family gives an edge, so the family still reads.
//!
//! A key is optional. It draws each shape and each edge style the drawing
//! uses, and it names a family from the same list that colors the edges.
//!
//! The output is sorted throughout and carries no clock and no digest, so two
//! runs over one lock write the same bytes.

use std::collections::{BTreeMap, BTreeSet};

use headwater_yaml::{Mapping, Spanned, Value};

/// A stroke color for each family, taken in the sorted order of the families
/// the lock declares. The list is a palette and names no family, so a family
/// a release adds takes the next color and a ninth one repeats the first.
const PALETTE: [&str; 8] = [
    "#1f77b4", "#d62728", "#2ca02c", "#9467bd", "#ff7f0e", "#8c564b", "#e377c2", "#17becf",
];

/// The stroke of an `is_a` arrow, which is no relation and has no family.
const IS_A: &str = "#888";

/// One `(relation, from, to, family)` pair a relation declares.
type Pair = (String, String, String, Option<String>);

/// Which drawing of the lock to print. A variant carries no doc comment, because
/// `clap` prints each one on its own line in a zsh completion script.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum View {
    // The concrete kinds, the anchors and the relations between them.
    #[default]
    Concrete,
    // The abstract kinds, the kinds under them and the relations that name one.
    Abstract,
}

/// What the lock says, read once and shared by both views.
struct Model<'a> {
    kinds: Option<&'a Mapping>,
    anchors: BTreeSet<String>,
    abstracts: BTreeSet<String>,
    /// purpose -> concrete kinds, with `None` for a kind no purpose reaches.
    lanes: BTreeMap<Option<String>, BTreeSet<String>>,
    families: BTreeSet<String>,
    /// Pairs with no abstract kind at either end.
    drawn: Vec<Pair>,
    /// Pairs with an abstract kind at one end or both.
    captioned: Vec<Pair>,
}

impl<'a> Model<'a> {
    fn of(resolved: &'a Mapping) -> Self {
        let purposes = keys(resolved, "purposes");
        let anchors = keys(resolved, "anchors");
        let kinds = map(resolved, "kinds");

        let mut abstracts = BTreeSet::new();
        let mut lanes: BTreeMap<Option<String>, BTreeSet<String>> = BTreeMap::new();
        for purpose in &purposes {
            lanes.entry(Some(purpose.clone())).or_default();
        }
        if let Some(kinds) = kinds {
            for entry in kinds {
                let name = entry.key.value.clone();
                let declared = entry.value.value.as_map();
                if declared.and_then(|k| scalar(k, "abstract")).as_deref() == Some("true") {
                    abstracts.insert(name);
                    continue;
                }
                lanes
                    .entry(purpose_of(kinds, &name))
                    .or_default()
                    .insert(name);
            }
        }

        let mut families: BTreeSet<String> = BTreeSet::new();
        let mut drawn: Vec<Pair> = Vec::new();
        let mut captioned: Vec<Pair> = Vec::new();
        if let Some(relations) = map(resolved, "relations") {
            for entry in relations {
                let Some(relation) = entry.value.value.as_map() else {
                    continue;
                };
                let name = entry.key.value.clone();
                let family = scalar(relation, "family");
                if let Some(family) = &family {
                    families.insert(family.clone());
                }
                for from in names(relation.get("from")) {
                    for to in names(relation.get("to")) {
                        let pair = (name.clone(), from.clone(), to.clone(), family.clone());
                        if abstracts.contains(&from) || abstracts.contains(&to) {
                            captioned.push(pair);
                        } else {
                            drawn.push(pair);
                        }
                    }
                }
            }
        }
        drawn.sort();
        drawn.dedup();
        captioned.sort();
        captioned.dedup();

        Model {
            kinds,
            anchors,
            abstracts,
            lanes,
            families,
            drawn,
            captioned,
        }
    }

    fn concrete(&self) -> BTreeSet<String> {
        self.lanes.values().flatten().cloned().collect()
    }

    fn node(&self, name: &str) -> String {
        if self.anchors.contains(name) {
            format!("anchor_{}", ident(name))
        } else {
            format!("kind_{}", ident(name))
        }
    }

    fn color(&self, family: &str) -> &'static str {
        let at = self.families.iter().position(|f| f == family).unwrap_or(0);
        PALETTE[at % PALETTE.len()]
    }
}

/// Draw the resolved taxonomy of `package` at `version`.
pub fn render(
    package: &str,
    version: &str,
    resolved: &Mapping,
    view: View,
    legend: bool,
) -> String {
    let model = Model::of(resolved);
    match view {
        View::Concrete => concrete_view(package, version, &model, legend),
        View::Abstract => abstract_view(package, version, &model, legend),
    }
}

/// The relations a node has to itself, each with the family that colors it.
type Loops = BTreeMap<String, Vec<(String, Option<String>)>>;

/// Split the pairs of a view into its edges and its loops. A pair whose two
/// ends are one node is drawn by Mermaid as a detour that reads badly, so the
/// view writes it on the node instead of drawing it.
fn split_loops(pairs: &[Pair]) -> (Vec<Pair>, Loops) {
    let mut edges = Vec::new();
    let mut loops: Loops = BTreeMap::new();
    for pair in pairs {
        let (relation, from, to, family) = pair;
        if from == to {
            loops
                .entry(from.clone())
                .or_default()
                .push((relation.clone(), family.clone()));
        } else {
            edges.push(pair.clone());
        }
    }
    (edges, loops)
}

/// The text of a node: its label, then one line for each relation the node has
/// to itself, in the color of the relation's family. A viewer that drops the
/// style keeps the text.
fn with_loops(model: &Model, text: String, name: &str, loops: &Loops) -> String {
    let mut out = text;
    for (relation, family) in loops.get(name).into_iter().flatten() {
        let line = format!("\u{21bb} {}", label(relation));
        match family {
            Some(family) => out.push_str(&format!(
                "<br/><span style='color:{}'>{line}</span>",
                model.color(family)
            )),
            None => out.push_str(&format!("<br/>{line}")),
        }
    }
    out
}

fn concrete_view(package: &str, version: &str, model: &Model, legend: bool) -> String {
    let concrete = model.concrete();
    let (edges, loops) = split_loops(&model.drawn);

    // An endpoint that is neither a declared kind nor an anchor. A validated
    // lock carries none, and one drawn outside every lane is a fact a reader
    // can see rather than an edge that silently went missing.
    let mut strays: BTreeSet<String> = BTreeSet::new();
    for (_, from, to, _) in &model.drawn {
        for end in [from, to] {
            if !concrete.contains(end) && !model.anchors.contains(end) {
                strays.insert(end.clone());
            }
        }
    }

    let kind_line = |kind: &String, indent: &str| -> String {
        format!(
            "{indent}kind_{}[\"{}\"]\n",
            ident(kind),
            with_loops(model, label(kind), kind, &loops)
        )
    };

    let mut out = String::new();
    out.push_str("flowchart LR\n");
    out.push_str(&format!(
        "%% The resolved taxonomy of {package} {version}, drawn by `headwater taxonomy graph` from .headwater/taxonomy.lock.\n"
    ));
    out.push_str(
        "%% A lane is a purpose, a rectangle is a concrete kind, a hexagon is an anchor, and an edge is labeled with its relation.\n",
    );
    if !loops.is_empty() {
        out.push_str(
            "%% A line that starts with \u{21bb} inside a node is a relation from that node to itself, in the color of its family.\n",
        );
    }
    if !model.captioned.is_empty() {
        out.push_str(&format!(
            "%% Not drawn here: a relation with an abstract kind ({}) at one end. `headwater taxonomy graph --view abstract` draws each one.\n",
            join(&model.abstracts)
        ));
    }

    for (purpose, members) in &model.lanes {
        let Some(purpose) = purpose else { continue };
        out.push_str(&format!(
            "  subgraph purpose_{}[\"{}\"]\n",
            ident(purpose),
            label(purpose)
        ));
        for kind in members {
            out.push_str(&kind_line(kind, "    "));
        }
        out.push_str("  end\n");
    }
    let unlaned = model.lanes.get(&None).into_iter().flatten();
    for kind in unlaned.chain(strays.iter()) {
        out.push_str(&kind_line(kind, "  "));
    }
    for anchor in &model.anchors {
        out.push_str(&format!(
            "  anchor_{}{{{{\"{}\"}}}}\n",
            ident(anchor),
            with_loops(model, label(anchor), anchor, &loops)
        ));
    }
    for pair in &edges {
        out.push_str(&edge(model, pair));
    }

    let mut styles = Vec::new();
    let mut anchor_ids: Vec<String> = model.anchors.iter().map(|a| model.node(a)).collect();
    if legend {
        let shown = Shown {
            abstract_kind: false,
            anchor: !anchor_ids.is_empty(),
            is_a: false,
            families: present(&edges, &loops),
        };
        styles = key(&mut out, model, &shown, edges.len());
        if shown.anchor {
            anchor_ids.push("key_anchor".to_string());
        }
    }
    if !anchor_ids.is_empty() {
        out.push_str("  classDef anchor fill:#eee,stroke:#555,color:#222\n");
        out.push_str(&format!("  class {} anchor\n", anchor_ids.join(",")));
    }
    family_styles(&mut out, model, &edges, 0);
    for style in styles {
        out.push_str(&style);
    }
    out
}

fn abstract_view(package: &str, version: &str, model: &Model, legend: bool) -> String {
    let mut out = String::new();
    out.push_str("flowchart LR\n");
    if model.abstracts.is_empty() {
        out.push_str(&format!(
            "%% The resolved taxonomy of {package} {version} declares no abstract kind, so this view has nothing to draw.\n"
        ));
        return out;
    }
    let (edges, loops) = split_loops(&model.captioned);

    // Every kind declared under an abstract kind, at any depth.
    let members: BTreeSet<String> = model
        .concrete()
        .into_iter()
        .filter(|name| under_abstract(model, name))
        .collect();
    let mut lanes: BTreeMap<Option<String>, BTreeSet<String>> = BTreeMap::new();
    for name in &members {
        let purpose = model.kinds.and_then(|kinds| purpose_of(kinds, name));
        lanes.entry(purpose).or_default().insert(name.clone());
    }

    // `is_a` arrows from every abstract and every member kind, sorted.
    let mut is_a: Vec<(String, String)> = Vec::new();
    if let Some(kinds) = model.kinds {
        for name in members.iter().chain(model.abstracts.iter()) {
            let parent = kinds
                .get(name)
                .and_then(|node| node.value.as_map())
                .and_then(|kind| scalar(kind, "is_a"));
            if let Some(parent) = parent {
                is_a.push((name.clone(), parent));
            }
        }
    }
    is_a.sort();

    // An endpoint of a pair that is no abstract kind, no member and no anchor.
    let mut strays: BTreeSet<String> = BTreeSet::new();
    let mut reached: BTreeSet<String> = BTreeSet::new();
    for (_, from, to, _) in &model.captioned {
        for end in [from, to] {
            if model.anchors.contains(end) {
                reached.insert(end.clone());
            } else if !members.contains(end) && !model.abstracts.contains(end) {
                strays.insert(end.clone());
            }
        }
    }

    let kind_line = |kind: &String, indent: &str| -> String {
        format!(
            "{indent}kind_{}[\"{}\"]\n",
            ident(kind),
            with_loops(model, label(kind), kind, &loops)
        )
    };

    out.push_str(&format!(
        "%% The abstract kinds of {package} {version} and the kinds declared under them, drawn by `headwater taxonomy graph --view abstract` from .headwater/taxonomy.lock.\n"
    ));
    out.push_str(
        "%% A dashed rectangle is an abstract kind, a dotted arrow is is_a, a hexagon is an anchor, and a solid edge is labeled with its relation.\n",
    );
    out.push_str(
        "%% A relation that names an abstract kind is open to every kind declared under it.\n",
    );
    if !loops.is_empty() {
        out.push_str(
            "%% A line that starts with \u{21bb} inside a node is a relation from that node to itself, in the color of its family.\n",
        );
    }

    for (purpose, kinds) in &lanes {
        match purpose {
            Some(purpose) => {
                out.push_str(&format!(
                    "  subgraph purpose_{}[\"{}\"]\n",
                    ident(purpose),
                    label(purpose)
                ));
                for kind in kinds {
                    out.push_str(&kind_line(kind, "    "));
                }
                out.push_str("  end\n");
            }
            None => {
                for kind in kinds {
                    out.push_str(&kind_line(kind, "  "));
                }
            }
        }
    }
    for kind in &strays {
        out.push_str(&kind_line(kind, "  "));
    }
    for name in &model.abstracts {
        let mut text = format!("{}<br/>abstract", label(name));
        let required = model
            .kinds
            .and_then(|kinds| kinds.get(name))
            .and_then(|node| node.value.as_map())
            .and_then(|kind| map(kind, "facets"))
            .map(|facets| names(facets.get("require")))
            .unwrap_or_default();
        if !required.is_empty() {
            text.push_str(&format!("<br/>requires: {}", label(&required.join(", "))));
        }
        let text = with_loops(model, text, name, &loops);
        out.push_str(&format!("  kind_{}[\"{text}\"]\n", ident(name)));
    }
    for anchor in &reached {
        out.push_str(&format!(
            "  anchor_{}{{{{\"{}\"}}}}\n",
            ident(anchor),
            with_loops(model, label(anchor), anchor, &loops)
        ));
    }
    for (kind, parent) in &is_a {
        out.push_str(&format!(
            "  {} -.-> {}\n",
            model.node(kind),
            model.node(parent)
        ));
    }
    for pair in &edges {
        out.push_str(&edge(model, pair));
    }

    let mut styles = Vec::new();
    let mut anchor_ids: Vec<String> = reached.iter().map(|a| model.node(a)).collect();
    let mut abstract_ids: Vec<String> = model.abstracts.iter().map(|a| model.node(a)).collect();
    let offset = is_a.len() + edges.len();
    if legend {
        let shown = Shown {
            abstract_kind: true,
            anchor: !anchor_ids.is_empty(),
            is_a: !is_a.is_empty(),
            families: present(&edges, &loops),
        };
        styles = key(&mut out, model, &shown, offset);
        abstract_ids.push("key_abstract".to_string());
        if shown.anchor {
            anchor_ids.push("key_anchor".to_string());
        }
    }
    out.push_str("  classDef abstract fill:#eee,stroke:#555,color:#222,stroke-dasharray:5 3\n");
    out.push_str(&format!("  class {} abstract\n", abstract_ids.join(",")));
    if !anchor_ids.is_empty() {
        out.push_str("  classDef anchor fill:#eee,stroke:#555,color:#222\n");
        out.push_str(&format!("  class {} anchor\n", anchor_ids.join(",")));
    }
    if !is_a.is_empty() {
        let at: Vec<String> = (0..is_a.len()).map(|i| i.to_string()).collect();
        out.push_str(&format!("  linkStyle {} stroke:{IS_A}\n", at.join(",")));
    }
    family_styles(&mut out, model, &edges, is_a.len());
    for style in styles {
        out.push_str(&style);
    }
    out
}

/// Whether a kind is declared, at any depth, under an abstract kind.
fn under_abstract(model: &Model, name: &str) -> bool {
    let Some(kinds) = model.kinds else {
        return false;
    };
    let mut seen = BTreeSet::new();
    let mut at = name.to_string();
    while seen.insert(at.clone()) {
        let parent = kinds
            .get(&at)
            .and_then(|node| node.value.as_map())
            .and_then(|kind| scalar(kind, "is_a"));
        let Some(parent) = parent else { return false };
        if model.abstracts.contains(&parent) {
            return true;
        }
        at = parent;
    }
    false
}

fn edge(model: &Model, (relation, from, to, _): &Pair) -> String {
    format!(
        "  {} -->|{}| {}\n",
        model.node(from),
        label(relation),
        model.node(to)
    )
}

/// The families that an edge or a loop line of a view carries, in the order
/// the lock sorts them.
fn present(edges: &[Pair], loops: &Loops) -> Vec<String> {
    let of_edges = edges.iter().filter_map(|(_, _, _, family)| family.clone());
    let of_loops = loops
        .values()
        .flatten()
        .filter_map(|(_, family)| family.clone());
    of_edges
        .chain(of_loops)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// One `linkStyle` for each family that has an edge, after `offset` edges of
/// another kind that came first.
fn family_styles(out: &mut String, model: &Model, edges: &[Pair], offset: usize) {
    for family in &model.families {
        let at: Vec<String> = edges
            .iter()
            .enumerate()
            .filter(|(_, (_, _, _, f))| f.as_deref() == Some(family.as_str()))
            .map(|(at, _)| (offset + at).to_string())
            .collect();
        if at.is_empty() {
            continue;
        }
        out.push_str(&format!("  %% family {family}\n"));
        out.push_str(&format!(
            "  linkStyle {} stroke:{}\n",
            at.join(","),
            model.color(family)
        ));
    }
}

/// What a key has to explain, because the drawing uses it.
struct Shown {
    abstract_kind: bool,
    anchor: bool,
    is_a: bool,
    families: Vec<String>,
}

/// The key, drawn as a lane of its own. Its edges come after every edge of the
/// drawing, so they take the indices from `start` on, and the `linkStyle`
/// lines this returns belong after every other one.
fn key(out: &mut String, model: &Model, shown: &Shown, start: usize) -> Vec<String> {
    out.push_str("  subgraph key[\"key\"]\n    direction TB\n");
    out.push_str("    key_kind[\"concrete kind\"]\n");
    if shown.abstract_kind {
        out.push_str("    key_abstract[\"abstract kind, stands for every kind under it\"]\n");
    }
    if shown.anchor {
        out.push_str("    key_anchor{{\"anchor, a thing a document points at\"}}\n");
    }
    let mut entries: Vec<(String, &str, bool)> = Vec::new();
    if shown.is_a {
        entries.push(("is_a".to_string(), IS_A, true));
    }
    for family in &shown.families {
        entries.push((family.clone(), model.color(family), false));
    }
    let mut styles = Vec::new();
    for (index, (name, color, dotted)) in entries.iter().enumerate() {
        let arrow = if *dotted { "-.->" } else { "-->" };
        out.push_str(&format!(
            "    key_a{index}[\" \"] {arrow}|{}| key_b{index}[\" \"]\n",
            label(name)
        ));
        styles.push(format!("  linkStyle {} stroke:{color}\n", start + index));
    }
    out.push_str("  end\n");
    styles
}

/// The purpose a kind declares, or the one the nearest kind above it declares.
fn purpose_of(kinds: &Mapping, name: &str) -> Option<String> {
    let mut seen = BTreeSet::new();
    let mut at = name.to_string();
    while seen.insert(at.clone()) {
        let kind = kinds.get(&at)?.value.as_map()?;
        if let Some(purpose) = scalar(kind, "purpose") {
            return Some(purpose);
        }
        at = scalar(kind, "is_a")?;
    }
    None
}

fn map<'a>(of: &'a Mapping, key: &str) -> Option<&'a Mapping> {
    of.get(key).and_then(|node| node.value.as_map())
}

fn keys(of: &Mapping, key: &str) -> BTreeSet<String> {
    map(of, key)
        .map(|m| m.iter().map(|e| e.key.value.clone()).collect())
        .unwrap_or_default()
}

fn scalar(of: &Mapping, key: &str) -> Option<String> {
    of.get(key)
        .and_then(|node| node.value.as_scalar())
        .map(|s| s.text.clone())
}

fn names(node: Option<&Spanned<Value>>) -> Vec<String> {
    match node.map(|n| &n.value) {
        Some(Value::Seq(items)) => items
            .iter()
            .filter_map(|item| item.value.as_scalar().map(|s| s.text.clone()))
            .collect(),
        Some(Value::Scalar(s)) => vec![s.text.clone()],
        _ => Vec::new(),
    }
}

fn join(names: &BTreeSet<String>) -> String {
    names.iter().cloned().collect::<Vec<_>>().join(", ")
}

/// A name as a Mermaid identifier: every character outside `[A-Za-z0-9_]`
/// becomes `_`. Every node carries a prefix, so no name meets a keyword.
fn ident(name: &str) -> String {
    name.chars()
        .map(|c| match c.is_ascii_alphanumeric() || c == '_' {
            true => c,
            false => '_',
        })
        .collect()
}

/// A name inside a quoted Mermaid label.
fn label(name: &str) -> String {
    name.replace('"', "#quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolved(text: &str) -> Mapping {
        let loaded = headwater_yaml::load(text).expect("the fixture loads");
        loaded.value.as_map().expect("a mapping").clone()
    }

    #[test]
    fn a_kind_with_no_purpose_of_its_own_takes_the_one_above_it() {
        let taxonomy = resolved(
            "purposes:\n  p: {}\nkinds:\n  base:\n    purpose: p\n  child:\n    is_a: base\n",
        );
        let text = render("x", "1", &taxonomy, View::Concrete, false);
        let lane: Vec<&str> = text
            .lines()
            .skip_while(|l| l.trim() != "subgraph purpose_p[\"p\"]")
            .take_while(|l| l.trim() != "end")
            .collect();
        assert!(lane.contains(&"    kind_child[\"child\"]"), "{text}");
    }

    #[test]
    fn a_lock_with_no_abstract_pair_carries_no_caption() {
        let taxonomy = resolved(
            "purposes:\n  p: {}\nkinds:\n  a:\n    purpose: p\n  b:\n    purpose: p\nrelations:\n  r:\n    family: f\n    from: [a]\n    to: [b]\n",
        );
        let text = render("x", "1", &taxonomy, View::Concrete, false);
        assert!(!text.contains("note_abstract"), "{text}");
        assert!(text.contains("  kind_a -->|r| kind_b\n"), "{text}");
        assert!(text.contains("  linkStyle 0 stroke:#1f77b4\n"), "{text}");
    }
    #[test]
    fn a_lock_with_no_abstract_kind_has_nothing_for_the_abstract_view_to_draw() {
        let taxonomy = resolved(
            "purposes:\n  p: {}\nkinds:\n  a:\n    purpose: p\nrelations:\n  r:\n    family: f\n    from: [a]\n    to: [a]\n",
        );
        let text = render("x", "1", &taxonomy, View::Abstract, true);
        assert!(text.contains("declares no abstract kind"), "{text}");
        assert!(
            !text.contains("subgraph") && !text.contains("-->"),
            "{text}"
        );
    }

    #[test]
    fn a_family_takes_one_color_in_both_views() {
        let taxonomy = resolved(
            "purposes:\n  p: {}\nkinds:\n  base:\n    abstract: true\n  a:\n    is_a: base\n    purpose: p\n  b:\n    is_a: base\n    purpose: p\nrelations:\n  s:\n    family: f\n    from: [a]\n    to: [b]\n  t:\n    family: g\n    from: [a]\n    to: [base]\n",
        );
        let concrete = render("x", "1", &taxonomy, View::Concrete, false);
        let abstract_view = render("x", "1", &taxonomy, View::Abstract, false);
        assert!(
            concrete.contains("linkStyle 0 stroke:#1f77b4"),
            "{concrete}"
        );
        assert!(
            abstract_view.contains("linkStyle 2 stroke:#d62728"),
            "family g is second in both views, after two is_a arrows here:\n{abstract_view}"
        );
    }
    #[test]
    fn a_relation_from_a_kind_to_itself_is_a_colored_line_on_the_kind() {
        let taxonomy = resolved(
            "purposes:\n  p: {}\nkinds:\n  a:\n    purpose: p\n  b:\n    purpose: p\nrelations:\n  f_first:\n    family: f\n    from: [a]\n    to: [a]\n  g_edge:\n    family: g\n    from: [a]\n    to: [b]\n",
        );
        let text = render("x", "1", &taxonomy, View::Concrete, true);
        assert!(
            !text.contains("-->|f_first|"),
            "no edge for the loop:\n{text}"
        );
        assert!(
            text.contains(
                "    kind_a[\"a<br/><span style='color:#1f77b4'>\u{21bb} f_first</span>\"]\n"
            ),
            "{text}"
        );
        assert!(
            text.contains("  linkStyle 0 stroke:#d62728\n"),
            "the one edge takes index 0 and the color of its own family:\n{text}"
        );
        assert!(
            text.contains("-->|f| key_b0") && text.contains("-->|g| key_b1"),
            "a family carried by a loop line only is keyed:\n{text}"
        );
    }

    #[test]
    fn a_pair_with_no_family_is_a_line_with_no_color() {
        let taxonomy = resolved(
            "purposes:\n  p: {}\nkinds:\n  a:\n    purpose: p\nrelations:\n  r:\n    from: [a]\n    to: [a]\n",
        );
        let text = render("x", "1", &taxonomy, View::Concrete, false);
        assert!(
            text.contains("    kind_a[\"a<br/>\u{21bb} r\"]\n"),
            "{text}"
        );
    }
}
