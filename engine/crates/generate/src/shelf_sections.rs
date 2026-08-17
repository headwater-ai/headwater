// SPDX-License-Identifier: Apache-2.0
//! The shelf sections: one heading for each document on a shelf.
//!
//! # What this kind is for, and how it differs from a shelf index
//!
//! [`crate::shelf_index`] writes one bullet for each document. A bullet carries
//! no anchor, so nothing outside the file can cite one row of it. This kind
//! writes one **heading** for each document, and a heading is an address: a
//! reader, an evaluation and an agent can all cite `file.md#the-heading` and
//! land on the row rather than on the file.
//!
//! That is the whole of the difference, and it is why the two are two kinds
//! rather than one kind with a member. A member that changed the emitted shape
//! would be read by one kind and ignored by seven, which is the position that
//! the meta-schema's `kind` value set exists to avoid. The declaration names
//! the shape it wants, and an emitter answers to that name.
//!
//! # The heading text comes from a facet, and the facet is reached by its role
//!
//! A heading is prose, and this module writes none of it. The text is the value
//! of the facet in the `name` role, which
//! [spec 2](../../../../docs/spec/02-taxonomy-model.md#the-meta-schema)
//! carries in the closed role registry. Three candidates were argued and two
//! are refused.
//!
//! - **The identifier.** `HW-DR-0009` is minted once and never edited, which
//!   makes it the stable choice and the useless one. An index of identifiers is
//!   the artifact that the decision-record entry declared `title` to escape.
//! - **A template on the declaration.** The largest of the three, and the one
//!   that puts authored prose in a taxonomy source. The taxonomy is not under
//!   the corpus root, so no census row covers it, no language regime binds it,
//!   and no rule reads its links. Prose that a corpus governs would move to the
//!   one file that the corpus cannot see.
//!   [Q4](../../../../docs/spec/09-decisions.md#q4--relation-storage) refused a
//!   second authoring location for an edge. This is the same refusal for prose,
//!   and prose is the thing the system exists to govern.
//! - **A facet, read by role.** What ships. A role is the taxonomy's own
//!   statement of what a value is for, and it is the only thing an emitter may
//!   read. Reading the facet by *name* would bind this module to `title`, which
//!   means something in the decision-record entry and nothing outside it.
//!   [`headwater_query::Surface::summary`] already takes this posture over the
//!   `scent` role, and this is that rule applied a second time.
//!
//! So a projection interpolates a path into a path, and a facet value into a
//! body. It interpolates nothing that an author wrote in a taxonomy.
//!
//! # Every decline is whole, and never per document
//!
//! A file with one section missing is worse than no file. A reader cannot see
//! the absence, and a citation into the missing section fails with no statement
//! anywhere about why. So a shelf whose documents cannot all be named produces
//! no file at all, and the plan reports the document that stopped it. That is
//! [spec 4](../../../../docs/spec/04-assurance-model.md)'s rule against a silent
//! pass, applied to a write.
//!
//! # Two documents may not carry one name
//!
//! Two headings with one text produce one anchor, and a citation cannot say
//! which of the two it meant. Markdown renderers disambiguate by appending an
//! ordinal, so the second anchor depends on the reading order, which a corpus
//! edit moves. Reported, and no file written.

use crate::{pointers, shelf_of, Declaration, Kind, Output, Plan, Unwritten};
use headwater_census::census::Census;
use headwater_query::{Pointer, Surface};

/// The placeholder a shelf sections output path may carry, as for an index.
const SHELF: &str = "{shelf}";

/// One document, with the heading that stands for it.
struct Section {
    name: String,
    pointer: Pointer,
}

pub(crate) fn emit(
    surface: &Surface<'_>,
    _census: &Census,
    declaration: &Declaration,
    plan: &mut Plan,
) {
    let taxonomy = surface.taxonomy();
    let wanted: Vec<String> = match declaration.shelves.is_empty() {
        true => taxonomy.shelves.iter().map(|s| s.name.clone()).collect(),
        false => declaration.shelves.clone(),
    };

    if wanted.len() > 1 && !declaration.output.contains(SHELF) {
        plan.unwritten.push(Unwritten {
            at: declaration.output.clone(),
            kind: Kind::ShelfSections,
            reason: format!(
                "covers {} shelves and its output holds no `{SHELF}`, so every shelf would be \
                 written over the last one",
                wanted.len()
            ),
        });
        return;
    }

    // An identity is one document's, and a declaration that covers several
    // shelves writes several files. Writing one identifier into each of them
    // mints a collision that the identifier index would report against files
    // nobody wrote by hand.
    if wanted.len() > 1 && declaration.identity.is_some() {
        plan.unwritten.push(Unwritten {
            at: declaration.output.clone(),
            kind: Kind::ShelfSections,
            reason: format!(
                "covers {} shelves and states one `identity`, so every file it wrote would carry \
                 the same identifier",
                wanted.len()
            ),
        });
        return;
    }

    for name in wanted {
        let Some(shelf) = taxonomy.shelves.iter().find(|shelf| shelf.name == name) else {
            plan.unwritten.push(Unwritten {
                at: format!("{} for {name}", declaration.output),
                kind: Kind::ShelfSections,
                reason: "names a shelf this taxonomy does not declare".to_string(),
            });
            continue;
        };
        let directory = crate::shelf_index::directory_of(shelf.pattern.source());
        let path = declaration.output.replace(SHELF, &directory);

        let on_shelf: Vec<_> = surface
            .documents()
            .into_iter()
            .filter(|document| shelf_of(document.path, surface).is_some_and(|s| s.name == name))
            .collect();

        if on_shelf.is_empty() {
            plan.unwritten.push(Unwritten {
                at: path,
                kind: Kind::ShelfSections,
                reason: format!(
                    "the shelf `{name}` holds no document, and an index of nothing asserts that \
                     a shelf is there"
                ),
            });
            continue;
        }

        let mut ordered = pointers(surface, &on_shelf);
        surface.by_precedence(&mut ordered);

        // The heading of each document, in the order the sections will be
        // written. A document the taxonomy cannot name stops the whole file.
        let mut sections = Vec::new();
        let mut refused = None;
        for pointer in ordered {
            let Some(document) = on_shelf
                .iter()
                .find(|document| document.path == pointer.path)
            else {
                continue;
            };
            match surface.name(document) {
                Some(text) => sections.push(Section {
                    name: text,
                    pointer,
                }),
                None => {
                    refused = Some(unnamed(surface, &pointer));
                    break;
                }
            }
        }
        if let Some(reason) = refused.or_else(|| collision(&sections)) {
            plan.unwritten.push(Unwritten {
                at: path,
                kind: Kind::ShelfSections,
                reason,
            });
            continue;
        }

        // The identity of the file, before its body, because a declaration that
        // cannot say what the file is produces no file at all. A block written
        // after the sections would put a document in the corpus that carries an
        // identifier nothing resolves.
        let front = match &declaration.identity {
            None => None,
            Some(identity) => {
                match crate::identity::front_matter(surface, identity, &path, Kind::ShelfSections) {
                    Ok(block) => Some(block),
                    Err(reason) => {
                        plan.unwritten.push(Unwritten {
                            at: path,
                            kind: Kind::ShelfSections,
                            reason,
                        });
                        continue;
                    }
                }
            }
        };

        plan.outputs.push(Output {
            bytes: render(&name, &path, &sections, front.as_deref()),
            path,
            kind: Kind::ShelfSections,
        });
    }
}

/// Why one document could not be named, stated so that a reader knows whether
/// to edit the taxonomy or the document.
///
/// The two cases are far apart. A taxonomy with no facet in the `name` role
/// cannot name anything, so no document on the shelf is at fault and the repair
/// is a declaration. A taxonomy that has the facet and a document that leaves it
/// empty is one document short, and the repair is in that document.
fn unnamed(surface: &Surface<'_>, pointer: &Pointer) -> String {
    match surface.name_facet() {
        Some(facet) => format!(
            "`{}` declares no `{facet}`, and a section needs a heading that the corpus states. \
             A file with one section missing hides the document it dropped",
            pointer.path
        ),
        None => "this taxonomy declares no facet in the `name` role, and a heading is the value \
                 of that facet. Declare one, or write a `shelf_index` instead"
            .to_string(),
    }
}

/// Two sections whose headings would produce one anchor.
fn collision(sections: &[Section]) -> Option<String> {
    for (at, section) in sections.iter().enumerate() {
        if let Some(first) = sections[..at]
            .iter()
            .find(|earlier| earlier.name == section.name)
        {
            return Some(format!(
                "`{}` and `{}` both declare the name `{}`. Two headings with one text share one \
                 anchor, so a citation cannot say which document it meant",
                first.pointer.path, section.pointer.path, section.name
            ));
        }
    }
    None
}

fn render(shelf: &str, output: &str, sections: &[Section], front: Option<&str>) -> String {
    let mut out = String::new();
    // One marker, in whichever of the two places the file's shape admits. A
    // document declares an identity, so its marker is a member of the block; a
    // list declares none, so its marker is the first line. Writing both would
    // say one thing twice and leave a comment above the fences, where no
    // front-matter parser reads it.
    match front {
        Some(block) => out.push_str(block),
        None => {
            let mark = headwater_mark::marker(Kind::ShelfSections.name(), output)
                .unwrap_or_else(|| format!("<!-- {} -->", headwater_mark::MARKER));
            out.push_str(&mark);
            out.push_str("\n\n");
        }
    }
    out.push_str("# ");
    out.push_str(shelf);
    out.push_str("\n\n");
    out.push_str(&format!(
        "{} {} on this shelf, in the reading order this corpus derives. Each heading below is the \
         name that the document declares, so a citation of a heading is a citation of a \
         document.\n",
        sections.len(),
        match sections.len() {
            1 => "document",
            _ => "documents",
        }
    ));
    let base = crate::shelf_index::parent_of(output);
    for section in sections {
        out.push_str(&format!("\n## {}\n\n", section.name));
        let target = crate::shelf_index::relative(&base, &section.pointer.path);
        let label = section
            .pointer
            .id
            .clone()
            .unwrap_or_else(|| crate::shelf_index::file_name(&section.pointer.path));
        // The body is the shelf index's row, and the heading above it is the
        // whole of what this kind adds. Written any other way the two emitters
        // would come to disagree about how one document reads, and a corpus
        // that declares both would say two things about one row. It also keeps
        // this module out of the business of guessing whether a summary ends a
        // sentence, which is punctuation that no facet states.
        out.push_str(&format!("[{label}]({target})"));
        if let Some(summary) = &section.pointer.summary {
            out.push_str(&format!(" — {summary}"));
        }
        // Spec 6 rules that an emitter which cannot carry the warrant does not
        // carry the content. Markdown carries it, so the section says it.
        if section.pointer.unwarranted {
            out.push_str(" (asserted, and no human has accepted it)");
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pointer(path: &str, id: &str) -> Pointer {
        Pointer {
            path: path.to_string(),
            id: Some(id.to_string()),
            kind: "decision".to_string(),
            name: None,
            purpose: None,
            summary: Some("a summary".to_string()),
            unwarranted: false,
        }
    }

    fn section(path: &str, id: &str, name: &str) -> Section {
        Section {
            name: name.to_string(),
            pointer: pointer(path, id),
        }
    }

    /// The property the whole kind exists for: a heading, and therefore an
    /// anchor, for each document.
    #[test]
    fn every_document_gets_a_heading() {
        let sections = vec![
            section("d/0001.md", "DR-1", "Q1 — Implementation language"),
            section("d/0002.md", "DR-2", "Q2 — Schema format"),
        ];
        let out = render("decisions", "d/README.md", &sections, None);
        assert!(out.contains("\n## Q1 — Implementation language\n"), "{out}");
        assert!(out.contains("\n## Q2 — Schema format\n"), "{out}");
        assert!(
            out.starts_with("<!-- headwater:generated shelf_sections."),
            "{out}"
        );
        assert_eq!(out.matches("\n## ").count(), 2, "{out}");
    }

    /// The body under a heading is exactly the row that the shelf index
    /// writes, so the two kinds never disagree about how one document reads.
    #[test]
    fn the_section_body_carries_the_cue_then_the_destination() {
        let sections = vec![section("d/0001.md", "DR-1", "Q1")];
        let out = render("decisions", "d/README.md", &sections, None);
        assert!(out.contains("[DR-1](0001.md) — a summary"), "{out}");
    }

    /// Two documents with one name would produce one anchor, and the second
    /// anchor a renderer mints depends on the order. Refused.
    #[test]
    fn two_documents_with_one_name_are_refused() {
        let sections = vec![
            section("d/0001.md", "DR-1", "Relation storage"),
            section("d/0002.md", "DR-2", "Relation storage"),
        ];
        let reason = collision(&sections).expect("the collision is reported");
        assert!(reason.contains("d/0001.md"), "{reason}");
        assert!(reason.contains("d/0002.md"), "{reason}");
        assert!(reason.contains("Relation storage"), "{reason}");
    }

    /// A document declares an identity, and the marker moves inside the block.
    /// A comment above the fences would put the block on line two, where no
    /// front-matter parser reads it, so the two shapes have to be exclusive.
    #[test]
    fn an_identity_block_carries_the_marker_and_the_body_follows_it() {
        let sections = vec![section("d/0001.md", "DR-1", "Q1 — Implementation language")];
        let block = "---\n\"headwater:generated\": \"shelf_sections. x\"\nid: HW-REG-x\n---\n\n";
        let out = render(
            "decisions",
            "docs/spec/09-open-questions.md",
            &sections,
            Some(block),
        );
        assert!(out.starts_with("---\n"), "{out}");
        assert!(
            headwater_mark::carries_marker("docs/spec/09-open-questions.md", &out),
            "{out}"
        );
        assert!(out.contains("\n---\n\n# decisions\n"), "{out}");
        assert!(out.contains("\n## Q1 — Implementation language\n"), "{out}");
        // One marker, and never two. A file that carried both would state one
        // fact twice, which is the drift a generated file exists to remove.
        assert_eq!(out.matches(headwater_mark::MARKER).count(), 1, "{out}");
    }

    /// The shape that shipped first, unchanged: no identity, so a first-line
    /// comment and no front matter at all.
    #[test]
    fn a_list_with_no_identity_keeps_its_first_line_marker() {
        let sections = vec![section("d/0001.md", "DR-1", "Q1")];
        let out = render("decisions", "d/README.md", &sections, None);
        assert!(
            out.starts_with("<!-- headwater:generated shelf_sections."),
            "{out}"
        );
        assert!(!out.contains("\n---\n"), "{out}");
    }

    #[test]
    fn distinct_names_collide_with_nothing() {
        let sections = vec![
            section("d/0001.md", "DR-1", "One"),
            section("d/0002.md", "DR-2", "Two"),
        ];
        assert!(collision(&sections).is_none());
    }
}
