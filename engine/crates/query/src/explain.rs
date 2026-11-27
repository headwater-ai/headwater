// SPDX-License-Identifier: Apache-2.0
//! `explain`: why the engine typed a document as it did, and what follows.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#kind-resolution):
//! "`headwater explain <path>` prints this derivation — which shelf matched,
//! which rule fired, and the kind's declared purpose. It also prints which
//! facets and sections are consequently required, and which relations are
//! permitted. Classification is never a black box, for a human or an agent."
//! That sentence is the contents of [`Explanation`], member for member.
//!
//! # The derivation is recorded rather than recomputed
//!
//! The census already resolved the kind, and it kept the steps: which shelf
//! matched, which shelves it beat, whether the placement carried the kind or a
//! discriminator decided it. This prints those steps. A second resolution here
//! could answer differently from the one the run used, and an explanation that
//! does not explain the run is worse than none.
//!
//! # A document the engine did not type still explains
//!
//! An untyped document is the case a reader most needs explained, and the
//! census records its derivation too: the shelf that matched and the step that
//! stopped. [`Surface::explain`] answers for it in the same shape, with no kind
//! and so no requirements. A verb that answered only for documents that already
//! worked would be a verb nobody needs.

use crate::{Neighbour, Pointer, Surface};
use headwater_census::census::Outcome;
use headwater_check::paint::{dim, paint, visible, ColorMode, Role};
use std::path::Path;

/// One document, explained.
#[derive(Clone, Debug)]
pub struct Explanation {
    pub path: String,
    pub id: Option<String>,
    /// The kind, and `None` for a document the census did not type.
    pub kind: Option<String>,
    /// The derivation, one step per line, as the census recorded it.
    pub derivation: Vec<String>,
    /// The purpose the kind serves, with the intent the taxonomy declares for
    /// it. A reader who opens the document then knows what it is *for*.
    pub purpose: Option<(String, Option<String>)>,
    pub summary: Option<String>,
    /// The warrant, printed always rather than only when it is `asserted`. A
    /// pointer is a glance and this is the answer to a question somebody asked.
    pub warrant: Option<String>,
    /// The facets the kind requires, inherited along the `is_a` chain.
    pub facets: Vec<String>,
    /// The sections it requires, on the same terms.
    pub sections: Vec<String>,
    /// The relations this kind may declare, each with the kinds it may reach.
    pub permitted: Vec<Permitted>,
    /// What this document is already related to, with the cue at each edge.
    pub related: Vec<Neighbour>,
    /// Why the walk did not read the entry, for a row it could not walk: a
    /// symlink, an unreadable directory, a name that is not UTF-8, or a named
    /// pipe, a socket or a device. The census never opened such an entry, so
    /// it knows no kind and nothing required of it, and a caller refuses the
    /// row with this reason rather than rendering it as a document (#1366).
    pub unwalkable: Option<String>,
}

/// One relation a kind may declare, and what it may reach.
#[derive(Clone, Debug)]
pub struct Permitted {
    pub relation: String,
    pub to: Vec<String>,
    /// The name the other end writes, where the relation declares one.
    pub inverse: Option<String>,
}

impl Surface<'_> {
    /// Explain the document at a path, or the one an identifier names.
    pub fn explain(&self, target: &str) -> Option<Explanation> {
        // A typed document, through the reader every other read uses.
        if let Some(document) = self.find(target) {
            let permitted = self.permitted(document.kind);
            return Some(Explanation {
                path: document.path.to_string(),
                id: document.id.map(str::to_string),
                kind: Some(document.kind.to_string()),
                derivation: document
                    .derivation
                    .map(|derivation| lines(&derivation.explain()))
                    .unwrap_or_default(),
                purpose: self
                    .shape()
                    .purpose_of(document.kind)
                    .map(|purpose| (purpose.name.clone(), purpose.intent.clone())),
                summary: self.summary(&document),
                warrant: self.warrant(&document),
                facets: self.shape().required_facets(document.kind),
                sections: self.shape().required_sections(document.kind),
                permitted,
                related: self.related(&document),
                unwalkable: None,
            });
        }

        // An untyped row of the census, which carries its own derivation.
        let row = self.census.rows.iter().find(|row| row.path == target)?;
        let derivation = match &row.outcome {
            Outcome::Untyped(headwater_census::census::Untyped::Unresolved {
                derivation, ..
            }) => lines(&derivation.explain()),
            other => vec![other.detail()],
        };
        Some(Explanation {
            path: row.path.clone(),
            id: None,
            kind: None,
            derivation,
            purpose: None,
            summary: None,
            warrant: None,
            facets: Vec::new(),
            sections: Vec::new(),
            permitted: Vec::new(),
            related: Vec::new(),
            unwalkable: match &row.outcome {
                Outcome::Unwalkable(_) => Some(row.outcome.detail()),
                _ => None,
            },
        })
    }

    /// The relations a kind may declare.
    ///
    /// Through the `is_a` chain, because a relation that names an abstract
    /// parent at its source end is declarable by every kind under it. A list
    /// that compared the two names directly would tell a `design_spec` that it
    /// may declare nothing, while the endpoint check permits it everything the
    /// parent has.
    pub fn permitted(&self, kind: &str) -> Vec<Permitted> {
        self.relations()
            .relations
            .iter()
            .filter(|relation| {
                relation
                    .from
                    .iter()
                    .any(|end| self.shape().descends_from(kind, end))
            })
            .map(|relation| Permitted {
                relation: relation.name.clone(),
                to: relation.to.clone(),
                inverse: relation.inverse.clone(),
            })
            .collect()
    }
}

fn lines(text: &str) -> Vec<String> {
    text.lines().map(str::to_string).collect()
}

impl Explanation {
    /// The sentence that refuses a row the walk could not read, and `None`
    /// for every other row. `headwater explain` prints it on standard error
    /// and the `explain` tool of `headwater mcp` answers with it, so the two
    /// refuse in one sentence (#1366).
    pub fn refusal(&self) -> Option<String> {
        self.refusal_for("explain")
    }

    /// [`Explanation::refusal`], with the verb that refuses named in it.
    /// `show` and the `related` tool of `headwater mcp` refuse the rows
    /// `explain` refuses, and this one function writes each sentence, so the
    /// sentences differ only in the verb (#1366).
    pub fn refusal_for(&self, verb: &str) -> Option<String> {
        self.unwalkable.as_ref().map(|reason| {
            format!(
                "`{}` is {}, so `{verb}` prints nothing",
                visible(&self.path),
                visible(reason)
            )
        })
    }

    /// The explanation as text, in spec 2's own order.
    ///
    /// `mode` colors the path and dims the repeated structural labels —
    /// `purpose`, `summary`, `warrant`, and the rest — that this report prints
    /// the same way on every run. See `headwater_check::paint`'s module
    /// comment for why this function reads no stream itself, and
    /// `crate::explain::tests` for why coloring a label before it is folded
    /// costs at most an early wrap rather than a broken one.
    ///
    /// Every value from a document goes through `visible`, so no control
    /// character reaches a terminal or a CI log and no value opens a line
    /// with `::` (#2069). A clean value is borrowed back unchanged.
    pub fn render(&self, mode: ColorMode) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(out, "{}", paint(Role::Path, &visible(&self.path), mode));
        if let Some(id) = &self.id {
            let _ = writeln!(out, "  {}", visible(id));
        }
        match &self.kind {
            Some(kind) => {
                let _ = writeln!(out, "  {} {}", dim("kind", mode), visible(kind));
            }
            None => out.push_str("  no kind, so nothing is required of it\n"),
        }
        for step in &self.derivation {
            out.push_str(&headwater_check::fill::filled(
                &format!("    {}\n", visible(step)),
                headwater_check::fill::WIDTH,
            ));
        }
        if let Some((name, intent)) = &self.purpose {
            let label = dim("purpose", mode);
            match intent {
                Some(intent) => out.push_str(&headwater_check::fill::filled(
                    &format!("  {label} {}, to {}\n", visible(name), visible(intent)),
                    headwater_check::fill::WIDTH,
                )),
                None => out.push_str(&headwater_check::fill::filled(
                    &format!("  {label} {}\n", visible(name)),
                    headwater_check::fill::WIDTH,
                )),
            };
        }
        if let Some(summary) = &self.summary {
            out.push_str(&headwater_check::fill::filled(
                &format!("  {} {}\n", dim("summary", mode), visible(summary)),
                headwater_check::fill::WIDTH,
            ));
        }
        if let Some(warrant) = &self.warrant {
            out.push_str(&headwater_check::fill::filled(
                &format!("  {} {}\n", dim("warrant", mode), visible(warrant)),
                headwater_check::fill::WIDTH,
            ));
        }
        if !self.facets.is_empty() {
            out.push_str(&headwater_check::fill::filled(
                &format!(
                    "  {} {}\n",
                    dim("requires the facets", mode),
                    visible(&self.facets.join(", "))
                ),
                headwater_check::fill::WIDTH,
            ));
        }
        if !self.sections.is_empty() {
            out.push_str(&headwater_check::fill::filled(
                &format!(
                    "  {} {}\n",
                    dim("requires the sections", mode),
                    visible(&self.sections.join(", "))
                ),
                headwater_check::fill::WIDTH,
            ));
        }
        for permitted in &self.permitted {
            let label = dim("may declare", mode);
            let line = match &permitted.inverse {
                Some(inverse) => format!(
                    "  {label} {} to {}, and the other end writes {}\n",
                    visible(&permitted.relation),
                    visible(&permitted.to.join(", ")),
                    visible(inverse)
                ),
                None => format!(
                    "  {label} {} to {}\n",
                    visible(&permitted.relation),
                    visible(&permitted.to.join(", "))
                ),
            };
            out.push_str(&headwater_check::fill::filled(
                &line,
                headwater_check::fill::WIDTH,
            ));
        }
        for neighbour in &self.related {
            out.push_str(&headwater_check::fill::filled(
                &format!("  {}\n", neighbour.render()),
                headwater_check::fill::WIDTH,
            ));
        }
        out
    }
}

/// What `headwater explain` found for a target: a document, or a code file and
/// the documents that govern it (#1968). The explanation is boxed because it
/// is six times the size of the other variant.
#[derive(Clone, Debug)]
pub enum Answer {
    Document(Box<Explanation>),
    Governed(Governed),
}

/// A code file, and the documents whose governing edges reach it.
///
/// `docs/interfaces/headwater-explain.md` states when `explain` answers with
/// this: the file is on the tree, it is not a census row, and at least one
/// governing edge reaches it. A path with no file keeps its refusal, because
/// `.claude/hooks/write.sh` reads that refusal for a document not yet written.
#[derive(Clone, Debug)]
pub struct Governed {
    pub path: String,
    pub pointers: Vec<Pointer>,
}

impl Surface<'_> {
    /// The documents that govern `relative`, a path that
    /// [`headwater_census::walk::within`] read inside `root`. A path that
    /// reaches its file through a link inside the root also gets the
    /// documents that govern the file at the end of the link (#1433). The MCP
    /// `governing_docs_for_path` tool and `explain` both read this, so the two
    /// answer one list.
    pub fn governing_docs_reached(&self, root: &Path, relative: &str) -> Vec<Pointer> {
        let mut governing = self.governing_docs_for_path(relative);
        let resolved =
            headwater_census::walk::real_within(root, relative).filter(|path| path != relative);
        if let Some(path) = resolved {
            for pointer in self.governing_docs_for_path(&path) {
                if !governing.iter().any(|seen| seen.path == pointer.path) {
                    governing.push(pointer);
                }
            }
        }
        governing
    }

    /// The code-file answer of `explain` for `relative`, and `None` where
    /// there is no regular file there or no governing edge reaches it.
    pub fn explain_code(&self, root: &Path, relative: &str) -> Option<Governed> {
        if !root.join(relative).is_file() {
            return None;
        }
        let pointers = self.governing_docs_reached(root, relative);
        match pointers.is_empty() {
            true => None,
            false => Some(Governed {
                path: relative.to_string(),
                pointers,
            }),
        }
    }
}

impl Governed {
    /// The answer as text: the path, then one `governed by` line for each
    /// document, with the pointer `route` writes and the identifier below it.
    pub fn render(&self, mode: ColorMode) -> String {
        let mut out = format!("{}\n", paint(Role::Path, &visible(&self.path), mode));
        for pointer in &self.pointers {
            out.push_str(&format!(
                "  {} {}\n",
                dim("governed by", mode),
                pointer.render()
            ));
            if let Some(id) = &pointer.id {
                out.push_str(&format!("    {}\n", visible(id)));
            }
        }
        out
    }
}

/// The sentence a refusal prints for a target shaped like an identifier of
/// this corpus that no document declares.
///
/// [#845](https://github.com/headwater-ai/headwater/issues/845): the four
/// states of [`classification_text`] are a reading of a path, and printing
/// one of them for an identifier told a reader something true and irrelevant
/// — "outside every corpus root" reads as evidence the identifier is
/// somewhere else, when the honest answer is that this corpus does not carry
/// it. This sentence does not say whether the identifier is a typo or an
/// invention, or whether an untyped document carries it: that finer read is
/// `resolve_identifier`'s, over `headwater mcp`.
pub fn identifier_text(target: &str) -> String {
    format!(
        "`{}` is shaped like an identifier of this corpus, and no document declares it",
        visible(target)
    )
}

/// The sentence a refusal prints for a target with no document, one per state
/// of [`headwater_census::walk::Classification`].
///
/// A closed match rather than a `Display` on the type itself: the type lives
/// in `headwater_census`, which states facts about a corpus to every crate
/// that reads it, and the words a caller reads belong beside the other
/// sentences `explain` and `show` print, such as [`crate::outside_text`].
pub fn classification_text(
    target: &str,
    classification: &headwater_census::walk::Classification,
) -> String {
    use headwater_census::walk::Classification;
    let shown = visible(target);
    match classification {
        Classification::Corpus => {
            format!("`{shown}` is a path of this corpus, with no document written there yet")
        }
        Classification::Excluded(pattern) => {
            format!(
                "`{shown}` is excluded by `{}`, so it is not corpus content",
                visible(pattern)
            )
        }
        Classification::Outside => {
            format!("`{shown}` is outside every corpus root this repository declares")
        }
        // #1227: this state is reached by a path that leaves the repository,
        // absolute, by `..` or through a symlink (#1249), and by a segment
        // that is not UTF-8, so the sentence names both. It is the one
        // sentence every path route prints, so it is not written here.
        Classification::Unclassifiable => crate::outside_text(target),
    }
}

#[cfg(test)]
mod tests {
    use super::Explanation;
    use headwater_check::paint::ColorMode;

    /// Each refusal sentence writes the target, and the pattern that excluded
    /// it, as visible text (#2069). A target arrives from argv or a hook, and
    /// a hook can pass a path read from the tree.
    #[test]
    fn a_refusal_writes_a_hostile_target_as_visible_text() {
        use headwater_census::walk::Classification;
        let target = "::error ::x\u{1b}[31m";
        let sentences = [
            super::identifier_text(target),
            super::classification_text(target, &Classification::Corpus),
            super::classification_text(
                target,
                &Classification::Excluded("::warning\u{1b}[2J/**".to_string()),
            ),
            super::classification_text(target, &Classification::Outside),
            super::classification_text(target, &Classification::Unclassifiable),
        ];
        for sentence in sentences {
            assert!(!sentence.contains('\u{1b}'), "{}", sentence.escape_debug());
            assert!(
                sentence.contains("`\\x3a:error ::x\\x1b[31m`"),
                "{sentence}"
            );
        }
    }

    fn minimal() -> Explanation {
        Explanation {
            path: "docs/spec/00-example.md".to_string(),
            id: None,
            kind: Some("design_spec".to_string()),
            derivation: Vec::new(),
            purpose: Some(("evidence".to_string(), Some("say why".to_string()))),
            summary: None,
            warrant: None,
            facets: Vec::new(),
            sections: Vec::new(),
            permitted: Vec::new(),
            related: Vec::new(),
            unwalkable: None,
        }
    }

    /// `Plain` writes no escape sequence anywhere in the report, on the rule
    /// `headwater_check::paint`'s module comment states. `Ansi` colors the
    /// path and dims the `purpose` label, and neither rewrites a word.
    #[test]
    fn the_path_and_the_structural_labels_carry_color_only_under_ansi() {
        let explanation = minimal();

        let plain = explanation.render(ColorMode::Plain);
        assert!(!plain.contains('\x1b'), "{plain:?}");
        assert!(plain.contains("docs/spec/00-example.md"), "{plain:?}");
        assert!(plain.contains("purpose evidence, to say why"), "{plain:?}");

        let ansi = explanation.render(ColorMode::Ansi);
        assert!(ansi.contains('\x1b'), "{ansi:?}");
        for word in ["docs/spec/00-example.md", "purpose", "evidence", "say why"] {
            assert!(ansi.contains(word), "{ansi:?} is missing {word:?}");
        }
    }
}
