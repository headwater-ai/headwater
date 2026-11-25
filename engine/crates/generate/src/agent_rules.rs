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
//!
//! # The budget
//!
//! [`measure`] renders every rule and holds it against the declaration's
//! `budget`. A rule past `per_file` is withheld. The rest stand in the derived
//! reading precedence of their sources, and the prune drops them from the
//! satellite end until the total fits `aggregate` (#2090). A withheld rule is
//! an [`Unwritten`] that names the figure, and its path joins
//! [`Plan::pruned`], so that a marked copy an earlier run wrote there is stale
//! under `--check` and removed by a write rather than left to load. The check
//! rule `projection.budget.exceeded` reads [`readings`], which calls the same
//! [`measure`], so the rule and the prune cannot disagree. A source document is
//! never edited to fit: the projection writes rule files and nothing else.
//!
//! # Two sources with one stem
//!
//! `{name}` is the file stem of the source, so two sources with one stem own
//! one path. Neither is written, and the plan names both, because a file that
//! one of them silently overwrote would load the wrong document's pointer.
use crate::{
    label, shelf_index, shelf_of, Budget, Declaration, Kind, Output, Plan, Projections, Unwritten,
};
use headwater_graph::declarations::Direction;
use headwater_graph::Target;
use headwater_query::Surface;
use std::collections::BTreeMap;

/// The placeholder for the source's file stem in `output`.
const NAME: &str = "{name}";

/// The relation a source governs code by.
const GOVERNS: &str = "governs";

/// What becomes of one rule under the declaration's budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fate {
    /// Written.
    Written,
    /// Not written, because the file alone is past `per_file`.
    OverFile,
    /// Not written, because the rules before it in the reading order already
    /// fill `aggregate`.
    Pruned,
}

/// One rule this declaration would write if no budget bound it.
#[derive(Clone, Debug)]
pub struct Candidate {
    /// The path of the rule file.
    pub path: String,
    /// The source document it points at.
    pub source: String,
    /// The rendered file, marker included.
    pub bytes: String,
    pub fate: Fate,
}

impl Candidate {
    /// The size the budget reads: the bytes of the file as written.
    pub fn size(&self) -> u64 {
        self.bytes.len() as u64
    }
}

/// Every rule one declaration would write, held against its budget.
///
/// The emitter and the check rule both read this, so the file `generate`
/// withholds and the file the finding names are one measurement.
#[derive(Clone, Debug)]
pub struct Measured {
    /// Every rule with a path of its own, in derived reading precedence of
    /// its source: a nucleus before its satellite, and a governing document
    /// before the one it governs.
    pub candidates: Vec<Candidate>,
    /// The paths two sources share, which neither writes. Not measured,
    /// because neither is a rule.
    pub collisions: Vec<Unwritten>,
    /// The bytes of every candidate that is not past `per_file`, together.
    pub total: u64,
    pub budget: Option<Budget>,
}

/// Measure every rule of one declaration and settle its fate.
///
/// A rule past `per_file` is withheld. The rest stand in derived reading
/// precedence, and the prune drops them from the satellite end until the
/// total is at or under `aggregate`, which keeps the longest head of the
/// order that fits. A smaller rule further down is not admitted into the gap,
/// because that would keep a satellite and drop its nucleus. HW-DR-0115 says
/// why the order is the one `route` and `explain` already give.
///
/// `Err` where the output names no `{name}`, which is no measurement at all.
pub fn measure(surface: &Surface<'_>, declaration: &Declaration) -> Result<Measured, Unwritten> {
    if !declaration.output.contains(NAME) {
        return Err(Unwritten {
            at: declaration.output.clone(),
            kind: Kind::AgentRules,
            reason: format!(
                "the output names no `{NAME}`, so every source would own this one path. \
                 HW-DR-0115 writes one file for each source, named by its file stem"
            ),
        });
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

    let mut collisions = Vec::new();
    let mut pointers = Vec::new();
    let mut rendered: BTreeMap<String, (String, String)> = BTreeMap::new();
    for (path, owners) in by_path {
        if let [(document, patterns)] = owners.as_slice() {
            let pointer = surface.pointer(document);
            rendered.insert(
                pointer.path.clone(),
                (path.clone(), render(&path, &pointer, patterns)),
            );
            pointers.push(pointer);
            continue;
        }
        let named: Vec<String> = owners
            .iter()
            .map(|(document, _)| format!("`{}`", document.path))
            .collect();
        collisions.push(Unwritten {
            at: path,
            kind: Kind::AgentRules,
            reason: format!(
                "{} share one file stem and so one rule path. HW-DR-0115 refuses, and does \
                 not overwrite, so neither is written. Rename one of them",
                named.join(" and ")
            ),
        });
    }

    surface.by_precedence(&mut pointers);
    let mut candidates: Vec<Candidate> = pointers
        .into_iter()
        .filter_map(|pointer| {
            let (path, bytes) = rendered.remove(&pointer.path)?;
            Some(Candidate {
                path,
                source: pointer.path,
                bytes,
                fate: Fate::Written,
            })
        })
        .collect();

    let mut total = 0;
    if let Some(budget) = declaration.budget {
        for candidate in &mut candidates {
            if candidate.size() > budget.per_file {
                candidate.fate = Fate::OverFile;
            }
        }
        let mut kept = 0;
        let mut full = false;
        for candidate in &mut candidates {
            if candidate.fate == Fate::OverFile {
                continue;
            }
            total += candidate.size();
            if full || kept + candidate.size() > budget.aggregate {
                full = true;
                candidate.fate = Fate::Pruned;
                continue;
            }
            kept += candidate.size();
        }
    } else {
        total = candidates.iter().map(Candidate::size).sum();
    }

    Ok(Measured {
        candidates,
        collisions,
        total,
        budget: declaration.budget,
    })
}

/// The budget reading of every `agent_rules` declaration that states a budget,
/// for the check rule `projection.budget.exceeded` (#2090).
///
/// A declaration with no budget gives no reading, because `taxonomy validate`
/// refuses it, and one whose output names no `{name}` gives none, because the
/// plan already refuses it.
pub fn readings(
    surface: &Surface<'_>,
    projections: &Projections,
) -> Vec<headwater_check::agent_budget::Reading> {
    projections
        .declared
        .iter()
        .enumerate()
        .filter(|(_, declaration)| declaration.kind == Kind::AgentRules)
        .filter_map(|(index, declaration)| {
            let budget = declaration.budget?;
            let measured = measure(surface, declaration).ok()?;
            let mut over: Vec<headwater_check::agent_budget::Over> = measured
                .candidates
                .iter()
                .filter(|candidate| candidate.fate == Fate::OverFile)
                .map(|candidate| headwater_check::agent_budget::Over {
                    path: candidate.path.clone(),
                    source: candidate.source.clone(),
                    bytes: candidate.size(),
                })
                .collect();
            over.sort_by(|a, b| a.path.cmp(&b.path));
            Some(headwater_check::agent_budget::Reading {
                index,
                output: declaration.output.clone(),
                per_file: budget.per_file,
                aggregate: budget.aggregate,
                over,
                total: measured.total,
                pruned: measured
                    .candidates
                    .iter()
                    .filter(|candidate| candidate.fate == Fate::Pruned)
                    .count(),
            })
        })
        .collect()
}

pub(crate) fn emit(surface: &Surface<'_>, declaration: &Declaration, plan: &mut Plan) {
    let measured = match measure(surface, declaration) {
        Ok(measured) => measured,
        Err(refused) => {
            plan.unwritten.push(refused);
            return;
        }
    };
    let mut written: Vec<Output> = Vec::new();
    let mut withheld: Vec<Unwritten> = Vec::new();
    for candidate in &measured.candidates {
        let budget = measured.budget.unwrap_or(Budget {
            per_file: u64::MAX,
            aggregate: u64::MAX,
        });
        let reason = match candidate.fate {
            Fate::Written => {
                written.push(Output {
                    bytes: candidate.bytes.clone(),
                    path: candidate.path.clone(),
                    kind: Kind::AgentRules,
                    committed: true,
                });
                continue;
            }
            Fate::OverFile => format!(
                "the rule for `{}` is {} bytes, and `budget.per_file` is {}, so it is not \
                 written. Trim the `summary` of `{}`, or raise `budget.per_file`",
                candidate.source,
                candidate.size(),
                budget.per_file,
                candidate.source
            ),
            Fate::Pruned => format!(
                "the rules of `{}` total {} bytes, and `budget.aggregate` is {}, so the rule \
                 for `{}` is pruned. HW-DR-0115 prunes from the satellite end of the reading \
                 order",
                declaration.output, measured.total, budget.aggregate, candidate.source
            ),
        };
        plan.pruned.push(candidate.path.clone());
        withheld.push(Unwritten {
            at: candidate.path.clone(),
            kind: Kind::AgentRules,
            reason,
        });
    }
    // Path order, as before the budget, so that the plan does not move with
    // the reading order of sources that all fit.
    written.sort_by(|a, b| a.path.cmp(&b.path));
    withheld.sort_by(|a, b| a.at.cmp(&b.at));
    plan.outputs.extend(written);
    plan.unwritten.extend(withheld);
    plan.unwritten.extend(measured.collisions);
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
