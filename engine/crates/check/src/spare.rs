// SPDX-License-Identifier: Apache-2.0
//! What a change did to the files a suspect `source-tree` edge reaches, and
//! whether a blocking control may leave the finding out of its gate (#2100,
//! #2028).
//!
//! `relation.target.suspect` reports an edge whose digest moved, and the
//! digest is over raw bytes, so a change of line endings or trailing white
//! space moves it as a change of content does. Under a blocking control both
//! stop a strict run. This module reads the manifest's prior version of each
//! touched file beside the current bytes and gives the file one class. It
//! changes no digest and no stamp: the comparison is the same, and only the
//! gate may read the class.
//!
//! # A run-level projection, read after the rules
//!
//! The classification is computed once over the run's live findings, as
//! [`crate::Run::blocked_by`] is, and never inside a rule. So the suspect
//! rule's version and its cache key do not move: a warm cache serves the same
//! finding, and this module classifies it again from the change and the tree.
//!
//! # The guard against earlier drift
//!
//! A white-space-only change is spared only where the value the edge had
//! before this change, rebuilt from the prior digests of the touched files and
//! the current digests of the rest, equals the recorded stamp. A content edit
//! that landed earlier and was never re-read leaves the stamp behind the prior
//! value, and then the edge still blocks: this change did not move it, and an
//! earlier one did, unread.

use crate::change::{Change, Digests, Touched};
use crate::finding::{at, Finding};
use headwater_graph::{Edge, Graph, Target};
use std::path::Path;

/// The class of one touched file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// An `added` line names it.
    Added,
    /// A `prior` line names it and no file stands at it now.
    Removed,
    /// Its raw bytes moved, and the bytes with white space taken out did not.
    /// See [`crate::change::normalized`].
    WhiteSpace,
    /// Anything else: a change of content, or a prior version that did not
    /// open, or bytes this run could not read.
    Content,
}

impl Class {
    /// The value as a report writes it.
    pub fn name(self) -> &'static str {
        match self {
            Class::Added => "added",
            Class::Removed => "removed",
            Class::WhiteSpace => "white_space",
            Class::Content => "content",
        }
    }
}

/// One suspect finding of a change-scoped run, and what the change did to the
/// files its edge reaches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Classified {
    /// Where the finding stands, which is where its edge's entry stands.
    pub path: String,
    pub line: usize,
    pub column: usize,
    /// Every manifest path the edge reaches, in path order, with its class.
    pub touched: Vec<(String, Class)>,
    /// The entries the edge's patterns match, each once, which is the count
    /// `touched` is read against.
    pub matched: usize,
    /// Every touched file is white space, there is at least one, and the value
    /// rebuilt from the prior digests equals the recorded stamp. Only then may
    /// a control that spares `white_space` leave the finding out of its gate.
    pub white_space: bool,
}

impl Classified {
    /// Whether this is the classification of `finding`.
    pub fn of(&self, finding: &Finding) -> bool {
        finding.rule == crate::suspect::RULE
            && finding.path == self.path
            && finding.line == self.line
            && finding.column == self.column
    }
}

/// The classification of every live suspect finding on a `source-tree` edge,
/// in finding order. Empty where the run carried no change, or named no base
/// to read the current bytes from.
pub fn classify(
    findings: &[Finding],
    graph: &Graph,
    change: Option<&Change>,
    base: Option<&Path>,
) -> Vec<Classified> {
    let (Some(change), Some(base)) = (change, base) else {
        return Vec::new();
    };
    let touched: Vec<(&str, Touched<'_>)> = change.touched().collect();
    findings
        .iter()
        .filter(|finding| finding.rule == crate::suspect::RULE)
        .filter_map(|finding| {
            let edge = graph.edges.iter().find(|edge| {
                edge.source.path == finding.path
                    && at(Some(edge.span)) == (finding.line, finding.column)
                    && edge.suspect_revisions().is_some()
            })?;
            one(finding, edge, &touched, base)
        })
        .collect()
}

fn one(
    finding: &Finding,
    edge: &Edge,
    touched: &[(&str, Touched<'_>)],
    base: &Path,
) -> Option<Classified> {
    let Target::Anchor {
        resolver, patterns, ..
    } = &edge.target
    else {
        return None;
    };
    if resolver != "source-tree" {
        return None;
    }
    let (verified, _) = edge.suspect_revisions()?;
    let mut union: Vec<String> = patterns
        .iter()
        .flat_map(|member| member.matched.iter().cloned())
        .collect();
    union.sort();
    union.dedup();
    let reached: Vec<(&str, Touched<'_>)> = touched
        .iter()
        .filter(|(path, _)| crate::scope::reaches_edge(edge, path))
        .copied()
        .collect();
    let classes: Vec<(String, Class)> = reached
        .iter()
        .map(|(path, line)| ((*path).to_string(), class(base, path, *line)))
        .collect();
    let white_space = !classes.is_empty()
        && classes.iter().all(|(_, class)| *class == Class::WhiteSpace)
        && rebuilt(base, &union, &reached).as_deref() == Some(verified);
    Some(Classified {
        path: finding.path.clone(),
        line: finding.line,
        column: finding.column,
        touched: classes,
        matched: union.len(),
        white_space,
    })
}

/// The class of the file at `path`, given what its manifest line said.
fn class(base: &Path, path: &str, line: Touched<'_>) -> Class {
    let prior = match line {
        Touched::Added => return Class::Added,
        Touched::Prior(prior) => prior,
    };
    let current = match std::fs::read(base.join(path)) {
        Ok(bytes) => Digests::of(&bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return match prior {
                Some(_) => Class::Removed,
                None => Class::Content,
            }
        }
        Err(_) => return Class::Content,
    };
    match prior {
        Some(prior) if prior.raw != current.raw && prior.normalized == current.normalized => {
            Class::WhiteSpace
        }
        _ => Class::Content,
    }
}

/// The value the edge had before this change: the manifest digest over its
/// matched union, with each touched file's prior raw digest in place of the
/// current one. Read only where every touched file is white space, so no file
/// was added or removed and the prior matched set is the current one.
fn rebuilt(base: &Path, union: &[String], reached: &[(&str, Touched<'_>)]) -> Option<String> {
    let given = |path: &str| {
        reached.iter().find_map(|(known, line)| match line {
            Touched::Prior(Some(prior)) if *known == path => Some(prior.raw.clone()),
            _ => None,
        })
    };
    headwater_graph::anchors::tree_revision_given(base, union, &given)
}
