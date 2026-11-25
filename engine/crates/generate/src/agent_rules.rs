// SPDX-License-Identifier: Apache-2.0
//! Agent rules: one Copilot path-scoped instructions file for each document
//! that governs code.
//!
//! [HW-DR-0115](../../../../docs/decisions/0115-an-agent-rules-file-is-a-budgeted-pointer-at-the-copilot-instructions-path.md)
//! states the form, and this module implements it and restates none of it. A
//! change to the form, the body or the budget figures is a change to that
//! record before it is a change here.
//!
//! # Which documents are sources
//!
//! A document on a shelf that the declaration's `for` names, or on any shelf
//! when `for` is empty, as every other kind reads `for`. Of those, only a
//! document with at least one `governs` edge to an anchor: a rule file whose
//! `applyTo` admits nothing is a file no edit ever loads.
//!
//! # What a file holds
//!
//! The front matter opens on line 1, because that is where a harness looks for
//! it. `applyTo` holds the `governs` patterns as the document declared them,
//! not the paths they matched, because the harness evaluates the glob itself
//! against the file being edited. It is quoted, because a plain scalar that
//! opens with `@` is a YAML error. The patterns join with `,` and no space,
//! which is the form Copilot reads. The marker is a front-matter member, as
//! [`headwater_mark::carries_marker`] reads one on a `.md` path.
//!
//! The body is a pointer: the document's label, its summary and a link to it.
//! Nothing of its body is copied, so the rule file never states a rule the
//! governing document does not, and an edit of the rule moves nothing here.
//! The budget is #2090's to enforce, and the pointer stays far below it.
//!
//! # Two sources with one stem
//!
//! `{name}` is the file stem of the source, so two sources with one stem own
//! one path. Neither is written, and the plan names both, because a file that
//! one of them silently overwrote would load the wrong document's pointer.
use crate::{label, shelf_index, shelf_of, Declaration, Kind, Output, Plan, Unwritten};
use headwater_graph::declarations::Direction;
use headwater_graph::Target;
use headwater_query::Surface;
use std::collections::BTreeMap;

/// The placeholder for the source's file stem in `output`.
const NAME: &str = "{name}";

/// The relation a source governs code by.
const GOVERNS: &str = "governs";

pub(crate) fn emit(surface: &Surface<'_>, declaration: &Declaration, plan: &mut Plan) {
    if !declaration.output.contains(NAME) {
        plan.unwritten.push(Unwritten {
            at: declaration.output.clone(),
            kind: Kind::AgentRules,
            reason: format!(
                "the output names no `{NAME}`, so every source would own this one path. \
                 HW-DR-0115 writes one file for each source, named by its file stem"
            ),
        });
        return;
    }

    let mut sources: Vec<_> = surface
        .documents()
        .into_iter()
        .filter(|document| {
            declaration.shelves.is_empty()
                || shelf_of(document.path, surface)
                    .is_some_and(|shelf| declaration.shelves.contains(&shelf.name))
        })
        .filter_map(|document| {
            let patterns = patterns(surface, document.path);
            (!patterns.is_empty()).then_some((document, patterns))
        })
        .collect();
    sources.sort_by(|a, b| a.0.path.cmp(b.0.path));

    let mut by_path: BTreeMap<String, Vec<_>> = BTreeMap::new();
    for (document, patterns) in sources {
        let name = shelf_index::file_name(document.path);
        let stem = name.strip_suffix(".md").unwrap_or(&name);
        let path = declaration.output.replace(NAME, stem);
        by_path.entry(path).or_default().push((document, patterns));
    }

    for (path, owners) in by_path {
        if let [(document, patterns)] = owners.as_slice() {
            let pointer = surface.pointer(document);
            plan.outputs.push(Output {
                bytes: render(&path, &pointer, patterns),
                path,
                kind: Kind::AgentRules,
                committed: true,
            });
            continue;
        }
        let named: Vec<String> = owners
            .iter()
            .map(|(document, _)| format!("`{}`", document.path))
            .collect();
        plan.unwritten.push(Unwritten {
            at: path,
            kind: Kind::AgentRules,
            reason: format!(
                "{} share one file stem and so one rule path. HW-DR-0115 refuses, and does \
                 not overwrite, so neither is written. Rename one of them",
                named.join(" and ")
            ),
        });
    }
}

/// The `governs` patterns the document at `path` declares, in declared order
/// and once each. An unbound or withheld target holds no pattern.
fn patterns(surface: &Surface<'_>, path: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for edge in &surface.graph().edges {
        if edge.declared != GOVERNS
            || !matches!(edge.direction, Direction::AsDeclared)
            || edge.source.path != path
        {
            continue;
        }
        if let Target::Anchor { patterns, .. } = &edge.target {
            for member in patterns {
                if !out.contains(&member.pattern) {
                    out.push(member.pattern.clone());
                }
            }
        }
    }
    out
}

/// The `applyTo` line: the patterns joined by `,` inside one double-quoted
/// YAML scalar, with `\` and `"` escaped, because a path may hold either.
fn apply_to(patterns: &[String]) -> String {
    format!(
        "applyTo: \"{}\"\n",
        patterns
            .join(",")
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
    )
}

fn render(path: &str, pointer: &headwater_query::Pointer, patterns: &[String]) -> String {
    let mut out = String::from("---\n");
    out.push_str(&apply_to(patterns));
    out.push_str(&headwater_mark::marker_member(Kind::AgentRules.name()));
    out.push_str("\n---\n\n");
    let link = shelf_index::relative(&shelf_index::parent_of(path), &pointer.path);
    out.push_str(&format!(
        "[{}]({link}) (`{}`) governs these files.",
        label(pointer),
        pointer.path
    ));
    if let Some(summary) = pointer.summary.as_deref() {
        out.push_str(&format!(" {summary}"));
    }
    if pointer.unwarranted {
        out.push_str(" No person has accepted that document.");
    }
    out.push_str("\n\nRead that document before you edit these files.\n");
    out
}

#[cfg(test)]
mod tests {
    use super::apply_to;

    /// Each pattern loads back from the line as the source declared it. The
    /// `source-tree` resolver binds a path with a `"` and the CLI case holds
    /// it there, and this case holds the `\` that no CLI case can reach.
    #[test]
    fn an_apply_to_line_loads_back_to_the_declared_patterns_with_a_quote_and_a_backslash() {
        let patterns = vec![
            "src/q\"uote/**".to_string(),
            "src/back\\slash/**".to_string(),
            "@generated/**".to_string(),
        ];
        let line = apply_to(&patterns);
        let loaded = headwater_yaml::load(&line)
            .unwrap_or_else(|errors| panic!("{line} does not load: {errors:?}"))
            .value;
        let text = loaded
            .as_map()
            .and_then(|members| members.get("applyTo"))
            .and_then(|node| node.value.as_scalar())
            .map(|scalar| scalar.text.clone())
            .unwrap_or_else(|| panic!("{line} holds no `applyTo` string"));
        assert_eq!(text, patterns.join(","), "{line}");
    }
}
