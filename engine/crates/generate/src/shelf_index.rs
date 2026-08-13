// SPDX-License-Identifier: Apache-2.0
//! The shelf index: every document on a shelf, in the order the corpus derives.
//!
//! # The order is not the file order
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#relations) derives
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

use crate::{marker, pointers, shelf_of, Declaration, Kind, Output, Plan, Unwritten};
use headwater_census::census::Census;
use headwater_query::{Pointer, Surface};

/// The placeholder a shelf index's output path may carry.
const SHELF: &str = "{shelf}";

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

        let mut ordered = pointers(surface, &on_shelf);
        surface.by_precedence(&mut ordered);
        plan.outputs.push(Output {
            bytes: render(&name, &path, &ordered),
            path,
            kind: Kind::ShelfIndex,
        });
    }
}

/// The directory a shelf's glob claims: everything before the first glob
/// construct, with no trailing separator.
fn directory_of(pattern: &str) -> String {
    let stop = pattern
        .find(|c| matches!(c, '*' | '?' | '[' | '{'))
        .unwrap_or(pattern.len());
    pattern[..stop].trim_end_matches('/').to_string()
}

fn render(shelf: &str, output: &str, ordered: &[Pointer]) -> String {
    let mut out = String::new();
    // `marker` answers `None` only for a format with no comment syntax, and a
    // shelf index is Markdown. The fallback is a plain line rather than an
    // unmarked file, because an unmarked generated file is the one thing this
    // module must never produce.
    let mark = marker(Kind::ShelfIndex, output)
        .unwrap_or_else(|| format!("<!-- {} -->", crate::MARKER));
    out.push_str(&mark);
    out.push_str("\n\n# ");
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
        let label = pointer.id.clone().unwrap_or_else(|| file_name(&pointer.path));
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

fn parent_of(path: &str) -> String {
    match path.rfind('/') {
        Some(at) => path[..at].to_string(),
        None => String::new(),
    }
}

fn file_name(path: &str) -> String {
    match path.rfind('/') {
        Some(at) => path[at + 1..].to_string(),
        None => path.to_string(),
    }
}

/// A link from the index to a document, relative to the index's own directory.
///
/// A document that is not under that directory keeps its repository-relative
/// path. A shelf glob can claim a path outside the directory its literal prefix
/// names, and a link built by counting `../` from a guess is worse than a link
/// that is plainly rooted.
fn relative(base: &str, path: &str) -> String {
    if base.is_empty() {
        return path.to_string();
    }
    match path.strip_prefix(&format!("{base}/")) {
        Some(rest) => rest.to_string(),
        None => format!("/{path}"),
    }
}
