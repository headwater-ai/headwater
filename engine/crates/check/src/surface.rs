// SPDX-License-Identifier: Apache-2.0
//! A Document-origin check: a page for an adopter that points at a file only
//! this repository holds.
//!
//! # The decision this implements
//!
//! [HW-DR-0077](../../../../docs/decisions/0077-the-consumer-surface-is-what-an-adopter-receives-runs-and-must-have-installed-and-it-is-a-closed-and-declared-list.md)
//! splits the tree into four populations and rules on the fourth: "No page for
//! an adopter instructs a file of this population. Such a page may cite one as
//! the way this repository does a thing, in a passage that says so."
//!
//! The record's Consequences say why a rule and not a review: "Nothing checks
//! this record." [#933](https://github.com/headwater-ai/headwater/issues/933)
//! refused a wide rule over a list of one entry, because such a list reports
//! zero by construction. So nothing here names a page or a root. The taxonomy
//! declares both under `surface`, and a corpus that declares no surface
//! generates no instance.
//!
//! # What counts as pointing at a file
//!
//! Three places, and none of them is prose. A path in a sentence is a mention,
//! and a mention is not an instruction.
//!
//! - an inline code span,
//! - a fenced or indented code block, which is where a command to run sits,
//! - a front-matter value that is one path and nothing else, which is how a
//!   `governs` edge names its target.
//!
//! A token counts when it opens with a declared local root, or with `./` and
//! then one. Raw HTML is left alone, because an HTML comment is a note to the
//! next editor and not a page an adopter reads.
//!
//! # A passage that says so
//!
//! The record lets a page cite this repository's way of doing a thing. The
//! marker for that passage is the suppression directive of spec 4, with
//! `reason=accepted_deviation`, because it already carries an expiry and a
//! reason from a closed set and the runner already counts it. A second marker
//! would be a second hatch with neither.
//!
//! # What it does not read
//!
//! A document the census does not type reaches no document check, so a page on
//! the list that is not typed goes unread. `README.md` at the root is one. The
//! `commands` member of `surface`, the second half of #976, has no reader yet.

use crate::finding::{Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView};
use crate::shape::Shape;
use headwater_doc::body::{BlockKind, Ownership};
use headwater_yaml::{Spanned, Value};

pub const RULE: &str = "surface.local_path.instructed";

/// The check, generated from the `surface` block of the taxonomy.
pub struct LocalPath {
    adopter_documents: Vec<String>,
    local_roots: Vec<String>,
}

impl LocalPath {
    /// The generation step. A surface that names no adopter document or no
    /// local root has nothing to hold a page against, and generates no
    /// instance.
    pub fn over(shape: &Shape) -> Self {
        LocalPath {
            adopter_documents: shape.surface.adopter_documents.clone(),
            local_roots: shape.surface.local_roots.clone(),
        }
    }

    fn declared(&self) -> bool {
        !self.adopter_documents.is_empty() && !self.local_roots.is_empty()
    }

    fn for_an_adopter(&self, path: &str) -> bool {
        self.adopter_documents
            .iter()
            .any(|glob| glob_matches(glob, path))
    }

    /// The local root a token opens with, if it opens with one.
    fn root_of(&self, token: &str) -> Option<&str> {
        let token = token.strip_prefix("./").unwrap_or(token);
        self.local_roots
            .iter()
            .find(|root| token.starts_with(root.as_str()))
            .map(String::as_str)
    }
}

impl DocumentCheck for LocalPath {
    const RULE: &'static str = self::RULE;
    const VERSION: u32 = 1;
    const NEEDS_BODY: bool = true;

    fn instantiates(&self, _kind: &str) -> bool {
        self.declared()
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        if !self.declared() || !self.for_an_adopter(view.path()) {
            return Outcome::Passed;
        }
        let mut findings = Vec::new();
        let mut report = |line: usize, column: usize, token: &str, root: &str, place: &str| {
            findings.push(Finding {
                rule: self::RULE,
                severity: Severity::Error,
                obligation: None,
                path: view.path().to_string(),
                line,
                column,
                message: format!(
                    "{place} names `{token}`, and `{root}` holds files only this repository has (HW-DR-0077, population 4)"
                ),
                remediation: format!(
                    "name the `headwater` verb or the declared prerequisite that does this, or mark the passage as how this repository does it with `<!-- headwater allow={} scope=block reason=accepted_deviation ... -->`",
                    self::RULE
                ),
                patch: None,
            });
        };

        for (key, node) in view
            .facets()
            .iter()
            .map(|entry| (&entry.key.value, &entry.value))
        {
            for item in paths_in(node) {
                let Some(text) = item.value.as_scalar().map(|scalar| scalar.text.as_str()) else {
                    continue;
                };
                if text.contains(char::is_whitespace) {
                    continue;
                }
                if let Some(root) = self.root_of(text) {
                    report(
                        item.span.start.line,
                        item.span.start.col,
                        text,
                        root,
                        &format!("the `{key}` front-matter value"),
                    );
                }
            }
        }

        for block in view
            .body()
            .map(|body| body.blocks.as_slice())
            .unwrap_or(&[])
        {
            if block.quote_depth > 0 || block.kind == BlockKind::Html {
                continue;
            }
            let place = match block.kind {
                BlockKind::Code => "a code block",
                _ => "a code span",
            };
            for run in &block.runs {
                if run.ownership != Ownership::Code || run.text.trim_start().starts_with('<') {
                    continue;
                }
                for (offset, line) in run.text.split('\n').enumerate() {
                    for token in line.split(|c: char| {
                        c.is_whitespace()
                            || matches!(c, '`' | '"' | '\'' | '(' | ')' | '=' | ',' | ';')
                    }) {
                        if let Some(root) = self.root_of(token) {
                            let line_number = run.span.start.line + offset;
                            let column = match offset {
                                0 => run.span.start.col,
                                _ => 1,
                            };
                            report(line_number, column, token, root, place);
                        }
                    }
                }
            }
        }
        findings.sort_by_key(|finding| (finding.line, finding.column));
        Outcome::failed(findings)
    }
}

/// Every scalar a front-matter value holds, through nested lists and mappings,
/// because a `governs` edge sits under `relations` and may group its targets
/// in an inner list.
fn paths_in(node: &Spanned<Value>) -> Vec<&Spanned<Value>> {
    match (&node.value.as_seq(), &node.value.as_map()) {
        (Some(items), _) => items.iter().flat_map(paths_in).collect(),
        (None, Some(map)) => map
            .iter()
            .flat_map(|entry| paths_in(&entry.value))
            .collect(),
        (None, None) => vec![node],
    }
}

/// `*` matches inside one path segment and `**` matches across any number.
pub fn glob_matches(glob: &str, path: &str) -> bool {
    let glob: Vec<&str> = glob.split('/').collect();
    let path: Vec<&str> = path.split('/').collect();
    segments(&glob, &path)
}

fn segments(glob: &[&str], path: &[&str]) -> bool {
    match glob.split_first() {
        None => path.is_empty(),
        Some((&"**", rest)) => (0..=path.len()).any(|skip| segments(rest, &path[skip..])),
        Some((first, rest)) => match path.split_first() {
            Some((segment, tail)) => segment_matches(first, segment) && segments(rest, tail),
            None => false,
        },
    }
}

fn segment_matches(glob: &str, text: &str) -> bool {
    match glob.split_once('*') {
        None => glob == text,
        Some((head, tail)) => {
            text.len() >= head.len()
                && text.starts_with(head)
                && (0..=text.len() - head.len())
                    .filter(|at| text.is_char_boundary(head.len() + at))
                    .any(|at| segment_matches(tail, &text[head.len() + at..]))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::glob_matches;

    #[test]
    fn a_glob_matches_inside_a_segment_and_across_segments() {
        assert!(glob_matches(
            "docs/interfaces/*.md",
            "docs/interfaces/headwater-check.md"
        ));
        assert!(!glob_matches(
            "docs/interfaces/*.md",
            "docs/interfaces/sub/x.md"
        ));
        assert!(glob_matches("docs/tutorials/**", "docs/tutorials/a/b.md"));
        assert!(glob_matches("README.md", "README.md"));
        assert!(!glob_matches(
            "docs/how-to/relocate-*.md",
            "docs/how-to/wire-x.md"
        ));
    }
}
