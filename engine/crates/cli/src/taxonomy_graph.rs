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
//! layout and no position. It carries four rules.
//!
//! - One subgraph for each purpose, holding each concrete kind whose purpose
//!   it is. A kind that names no purpose inherits the purpose of the nearest
//!   kind it is declared `is_a`.
//! - One hexagon for each anchor, drawn whether or not a relation reaches it.
//! - One edge for each `(from, to)` pair of each relation, labeled with the
//!   relation's name and styled by its family.
//! - **No edge where either end is an abstract kind.** An abstract kind stands
//!   for every concrete kind declared under it, so an edge to it would read as
//!   an edge to a node nobody can write. Each such pair is listed in a caption
//!   instead: a `%%` comment block and one note node. The rule reads
//!   `abstract: true` and names no relation.
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

/// Draw the resolved taxonomy of `package` at `version`.
pub fn render(package: &str, version: &str, resolved: &Mapping) -> String {
    let purposes: BTreeSet<String> = keys(resolved, "purposes");
    let anchors: BTreeSet<String> = keys(resolved, "anchors");
    let kinds = map(resolved, "kinds");

    let mut abstracts = BTreeSet::new();
    // purpose -> kinds, with `None` for a kind no purpose reaches.
    let mut lanes: BTreeMap<Option<String>, BTreeSet<String>> = BTreeMap::new();
    for purpose in &purposes {
        lanes.entry(Some(purpose.clone())).or_default();
    }
    if let Some(kinds) = kinds {
        for entry in kinds.iter() {
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
    let concrete: BTreeSet<String> = lanes.values().flatten().cloned().collect();

    // relation -> family, and every (relation, from, to) pair.
    let mut families: BTreeSet<String> = BTreeSet::new();
    let mut drawn: Vec<(String, String, String, Option<String>)> = Vec::new();
    let mut captioned: Vec<(String, String, String)> = Vec::new();
    if let Some(relations) = map(resolved, "relations") {
        for entry in relations.iter() {
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
                    if abstracts.contains(&from) || abstracts.contains(&to) {
                        captioned.push((name.clone(), from.clone(), to.clone()));
                    } else {
                        drawn.push((name.clone(), from.clone(), to.clone(), family.clone()));
                    }
                }
            }
        }
    }
    drawn.sort();
    drawn.dedup();
    captioned.sort();
    captioned.dedup();

    // An endpoint that is neither a declared kind nor an anchor. A validated
    // lock carries none, and one drawn outside every lane is a fact a reader
    // can see rather than an edge that silently went missing.
    let mut strays: BTreeSet<String> = BTreeSet::new();
    for (_, from, to, _) in &drawn {
        for end in [from, to] {
            if !concrete.contains(end) && !anchors.contains(end) {
                strays.insert(end.clone());
            }
        }
    }

    let node = |name: &str| -> String {
        if anchors.contains(name) {
            format!("anchor_{}", ident(name))
        } else {
            format!("kind_{}", ident(name))
        }
    };

    let mut out = String::new();
    out.push_str("flowchart LR\n");
    out.push_str(&format!(
        "%% The resolved taxonomy of {package} {version}, drawn by `headwater taxonomy graph` from .headwater/taxonomy.lock.\n"
    ));
    out.push_str(
        "%% A lane is a purpose, a rectangle is a concrete kind, a hexagon is an anchor, and an edge is labeled with its relation.\n",
    );
    if !captioned.is_empty() {
        out.push_str(&format!(
            "%% Not drawn: each pair below has an abstract kind ({}) at one end. An abstract kind stands for every concrete kind declared under it, so each of those kinds can take the relation.\n",
            join(&abstracts)
        ));
        for (relation, from, to) in &captioned {
            out.push_str(&format!("%%   {relation}: {from} -> {to}\n"));
        }
    }

    for (purpose, members) in &lanes {
        let Some(purpose) = purpose else { continue };
        out.push_str(&format!(
            "  subgraph purpose_{}[\"{}\"]\n",
            ident(purpose),
            label(purpose)
        ));
        for kind in members {
            out.push_str(&format!("    kind_{}[\"{}\"]\n", ident(kind), label(kind)));
        }
        out.push_str("  end\n");
    }
    let unlaned = lanes.get(&None).into_iter().flatten();
    for kind in unlaned.chain(strays.iter()) {
        out.push_str(&format!("  kind_{}[\"{}\"]\n", ident(kind), label(kind)));
    }
    for anchor in &anchors {
        out.push_str(&format!(
            "  anchor_{}{{{{\"{}\"}}}}\n",
            ident(anchor),
            label(anchor)
        ));
    }
    for (relation, from, to, _) in &drawn {
        out.push_str(&format!(
            "  {} -->|{}| {}\n",
            node(from),
            label(relation),
            node(to)
        ));
    }
    if !captioned.is_empty() {
        let mut text = format!(
            "Not drawn, because {} is abstract and stands for every concrete kind declared under it:",
            join(&abstracts)
        );
        for (relation, from, to) in &captioned {
            text.push_str(&format!(
                "<br/>{}: {} -> {}",
                label(relation),
                label(from),
                label(to)
            ));
        }
        out.push_str(&format!("  note_abstract[\"{text}\"]\n"));
        out.push_str("  class note_abstract note\n");
        out.push_str("  classDef note fill:#fff8dc,stroke:#999,stroke-dasharray:3 3\n");
    }
    if !anchors.is_empty() {
        out.push_str("  classDef anchor fill:#eee,stroke:#555\n");
        let members: Vec<String> = anchors
            .iter()
            .map(|a| format!("anchor_{}", ident(a)))
            .collect();
        out.push_str(&format!("  class {} anchor\n", members.join(",")));
    }
    for (index, family) in families.iter().enumerate() {
        let edges: Vec<String> = drawn
            .iter()
            .enumerate()
            .filter(|(_, (_, _, _, f))| f.as_deref() == Some(family.as_str()))
            .map(|(at, _)| at.to_string())
            .collect();
        if edges.is_empty() {
            continue;
        }
        out.push_str(&format!("  %% family {family}\n"));
        out.push_str(&format!(
            "  linkStyle {} stroke:{}\n",
            edges.join(","),
            PALETTE[index % PALETTE.len()]
        ));
    }
    out
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
        let text = render("x", "1", &taxonomy);
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
            "purposes:\n  p: {}\nkinds:\n  a:\n    purpose: p\nrelations:\n  r:\n    family: f\n    from: [a]\n    to: [a]\n",
        );
        let text = render("x", "1", &taxonomy);
        assert!(!text.contains("note_abstract"), "{text}");
        assert!(text.contains("  kind_a -->|r| kind_a\n"), "{text}");
        assert!(text.contains("  linkStyle 0 stroke:#1f77b4\n"), "{text}");
    }
}
