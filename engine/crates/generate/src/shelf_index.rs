// SPDX-License-Identifier: Apache-2.0
//! The shelf index: every document on a shelf, in the order the corpus derives.
//!
//! # The order is not the file order
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#reading-precedence-is-derived) derives
//! reading precedence from the relation family and the nucleus, and names three
//! consumers: a routing result, the document a conflict is reported against, and
//! "reading order in generated indexes". This is the third one. It calls the
//! same [`headwater_query::Surface::by_precedence`] that `route` calls, so an
//! index and a route never disagree about which of two documents a reader opens
//! first.
//!
//! # `{shelf}` is the shelf's directory
//!
//! The one declaration this repository's package ships writes
//! `output: "{shelf}/README.md"`, and no document states what `{shelf}` holds.
//! It is not the shelf's name: a name is not a path, and `decisions/README.md`
//! would put the index outside the corpus root that holds the shelf. It is the
//! shelf's directory, which this module reads as the literal prefix of the
//! shelf's `path` glob. `docs/decisions/**` gives `docs/decisions`.
//!
//! That reading is the engine's, and the specification does not state it. It
//! sits beside the meta-schema's open question about which glob constructs a
//! shelf path admits, and [spec 13](../../../../docs/spec/13-open-obligations.md)
//! carries both.

use crate::profile::Admission;
use crate::{pointers, shelf_of, Declaration, Kind, Output, Plan, Unwritten};
use headwater_census::census::Census;
use headwater_query::{Document, Pointer, Surface};

/// The placeholder a shelf index's output path may carry.
const SHELF: &str = "{shelf}";

/// The documents of a shelf that this declaration's filter admits.
///
/// `projections[].filter` is declared in the meta-schema for every projection
/// kind, and it reached one caller: the graph export. A shelf projection took
/// every document on the shelf and never asked, so a declared filter validated,
/// resolved, generated and was discarded at exit 0. Both shelf emitters call
/// this, and an empty filter admits everything, which is what a declaration
/// that states none says.
pub(crate) fn admitted<'a>(
    declaration: &Declaration,
    on_shelf: Vec<Document<'a>>,
) -> Vec<Document<'a>> {
    let filter = &declaration.membership.filter;
    if filter.is_empty() {
        return on_shelf;
    }
    on_shelf
        .into_iter()
        .filter(|document| {
            filter.admits(&declaration.membership.name, document.facets) == Admission::Carried
        })
        .collect()
}

/// What a declaration whose filter admits nothing says, which is not what an
/// empty shelf says.
///
/// An index of an empty shelf asserts that a shelf is there. A filter that
/// names a value no document of the shelf states is a declaration that produced
/// nothing, and a reader acts on the difference.
pub(crate) fn filtered_out(declaration: &Declaration, shelf: &str) -> String {
    format!(
        "the shelf `{shelf}` holds documents and the `{}` filter admits none of them, so this \
         declaration writes nothing",
        declaration.membership.name
    )
}

pub(crate) fn emit(
    surface: &Surface<'_>,
    _census: &Census,
    declaration: &Declaration,
    plan: &mut Plan,
) {
    let taxonomy = surface.taxonomy();
    // An empty `for` covers every shelf, which is what a declaration that names
    // none says. Naming them is the common case and this repository's package
    // does name them.
    let wanted: Vec<String> = match declaration.shelves.is_empty() {
        true => taxonomy.shelves.iter().map(|s| s.name.clone()).collect(),
        false => declaration.shelves.clone(),
    };

    // One output path for several shelves is a collision, and the engine says so
    // rather than writing each shelf over the last. Referential integrity checks
    // that a named shelf exists; nothing checks that the output separates them.
    if wanted.len() > 1 && !declaration.output.contains(SHELF) {
        plan.unwritten.push(Unwritten {
            at: declaration.output.clone(),
            kind: Kind::ShelfIndex,
            reason: format!(
                "covers {} shelves and its output holds no `{SHELF}`, so every shelf would be \
                 written over the last one",
                wanted.len()
            ),
        });
        return;
    }

    // See [`crate::shelf_sections`]: one identity cannot name several files.
    if wanted.len() > 1 && declaration.identity.is_some() {
        plan.unwritten.push(Unwritten {
            at: declaration.output.clone(),
            kind: Kind::ShelfIndex,
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
                kind: Kind::ShelfIndex,
                reason: "names a shelf this taxonomy does not declare".to_string(),
            });
            continue;
        };
        let directory = directory_of(shelf.pattern.source());
        let path = declaration.output.replace(SHELF, &directory);

        let on_shelf: Vec<_> = surface
            .documents()
            .into_iter()
            .filter(|document| shelf_of(document.path, surface).is_some_and(|s| s.name == name))
            .collect();

        // An index of nothing is a file that asserts a shelf exists. This
        // repository's package declares indexes for two shelves that hold no
        // document, and writing them would create two directories for two
        // shelves the tree does not have. Reported, and not written.
        if on_shelf.is_empty() {
            plan.unwritten.push(Unwritten {
                at: path,
                kind: Kind::ShelfIndex,
                reason: format!(
                    "the shelf `{name}` holds no document, and an index of nothing asserts that \
                     a shelf is there"
                ),
            });
            continue;
        }

        let on_shelf = admitted(declaration, on_shelf);
        if on_shelf.is_empty() {
            plan.unwritten.push(Unwritten {
                at: path,
                kind: Kind::ShelfIndex,
                reason: filtered_out(declaration, &name),
            });
            continue;
        }

        let mut ordered = pointers(surface, &on_shelf);
        surface.by_precedence(&mut ordered);
        let front = match &declaration.identity {
            None => None,
            Some(identity) => {
                match crate::identity::front_matter(surface, identity, &path, Kind::ShelfIndex) {
                    Ok(block) => Some(block),
                    Err(reason) => {
                        plan.unwritten.push(Unwritten {
                            at: path,
                            kind: Kind::ShelfIndex,
                            reason,
                        });
                        continue;
                    }
                }
            }
        };
        plan.outputs.push(Output {
            bytes: render(&name, &path, &ordered, front.as_deref()),
            path,
            kind: Kind::ShelfIndex,
        });
    }
}

/// The directory a shelf's glob claims: everything before the first glob
/// construct, with no trailing separator.
pub(crate) fn directory_of(pattern: &str) -> String {
    let stop = pattern.find(['*', '?', '[', '{']).unwrap_or(pattern.len());
    pattern[..stop].trim_end_matches('/').to_string()
}

fn render(shelf: &str, output: &str, ordered: &[Pointer], front: Option<&str>) -> String {
    let mut out = String::new();
    // One marker, in whichever of the two places the file's shape admits: a
    // member of the front-matter block for an index that declares an identity,
    // and the first line for one that declares none. See
    // [`crate::shelf_sections`], which takes the same two shapes.
    //
    // `marker` answers `None` only for a format with no comment syntax, and a
    // shelf index is Markdown. The fallback is a plain line rather than an
    // unmarked file, because an unmarked generated file is the one thing this
    // module must never produce.
    match front {
        Some(block) => out.push_str(block),
        None => {
            let mark = headwater_mark::marker(Kind::ShelfIndex.name(), output)
                .unwrap_or_else(|| format!("<!-- {} -->", headwater_mark::MARKER));
            out.push_str(&mark);
            out.push_str("\n\n");
        }
    }
    out.push_str("# ");
    out.push_str(shelf);
    out.push_str("\n\n");
    out.push_str(&format!(
        "{} {} on this shelf, in the reading order this corpus derives.\n\n",
        ordered.len(),
        match ordered.len() {
            1 => "document",
            _ => "documents",
        }
    ));
    let base = parent_of(output);
    for pointer in ordered {
        let target = relative(&base, &pointer.path);
        let label = crate::label(pointer);
        out.push_str(&format!("- [{label}]({target})"));
        if let Some(summary) = &pointer.summary {
            out.push_str(&format!(" — {summary}"));
        }
        // Spec 6 rules that an emitter which cannot carry the warrant does not
        // carry the content. Markdown can carry it, so this one says it rather
        // than withholding the row.
        if pointer.unwarranted {
            out.push_str(" (asserted, and no human has accepted it)");
        }
        out.push('\n');
    }
    out
}

pub(crate) fn parent_of(path: &str) -> String {
    match path.rfind('/') {
        Some(at) => path[..at].to_string(),
        None => String::new(),
    }
}

pub(crate) fn file_name(path: &str) -> String {
    match path.rfind('/') {
        Some(at) => path[at + 1..].to_string(),
        None => path.to_string(),
    }
}

/// A link from the output to a document, relative to the output's own
/// directory.
///
/// Both arguments are repository-relative and both are known, so the `../` hops
/// are counted rather than guessed. That matters to
/// [`crate::shelf_sections`] and not to this module: a shelf index lands on the
/// shelf it indexes, so every document is under the base and the walk up never
/// runs. A projection whose output path a declaration chooses is under no such
/// rule, and this repository would declare one that sits two directories from
/// the shelf.
///
/// A leading `/` was the earlier answer and it is not one. Markdown has no
/// repository root, so a renderer reads `/docs/x.md` as a path on the host and
/// leaves the corpus entirely.
pub(crate) fn relative(base: &str, path: &str) -> String {
    if base.is_empty() {
        return path.to_string();
    }
    let from: Vec<&str> = base.split('/').collect();
    let to: Vec<&str> = path.split('/').collect();
    // The file name is never part of the common directory prefix, so the walk
    // stops one short of the end of `to`.
    let shared = from
        .iter()
        .zip(&to[..to.len() - 1])
        .take_while(|(a, b)| a == b)
        .count();
    let mut out = String::new();
    for _ in shared..from.len() {
        out.push_str("../");
    }
    out.push_str(&to[shared..].join("/"));
    out
}

#[cfg(test)]
mod tests {
    use super::relative;

    /// The case a shelf index never reaches and a declared output path does.
    /// A leading `/` was the earlier answer, and a renderer reads it as a path
    /// on the host rather than as a path in the repository.
    #[test]
    fn a_document_outside_the_output_directory_is_reached_by_walking_up() {
        assert_eq!(
            relative("docs/spec", "docs/decisions/0001-a.md"),
            "../decisions/0001-a.md"
        );
        assert_eq!(relative("a/b/c", "d/e.md"), "../../../d/e.md");
    }

    /// The case a shelf index does reach, unchanged.
    #[test]
    fn a_document_under_the_output_directory_keeps_its_bare_name() {
        assert_eq!(
            relative("docs/decisions", "docs/decisions/0001-a.md"),
            "0001-a.md"
        );
        assert_eq!(
            relative("docs", "docs/decisions/0001-a.md"),
            "decisions/0001-a.md"
        );
        assert_eq!(relative("", "docs/a.md"), "docs/a.md");
    }
}
