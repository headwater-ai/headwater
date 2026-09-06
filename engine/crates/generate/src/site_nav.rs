// SPDX-License-Identifier: Apache-2.0
//! The site navigation: one file, the reading order this corpus derives,
//! written in the shape MkDocs's `nav:` key reads.
//!
//! [HW-DR-0036](../../../../docs/decisions/0036-q36-which-of-mkdocs-docusaurus-or-astro-this-corpus-emits-navigation-for-and-why.md)
//! is why this emitter writes that shape and not Docusaurus's `sidebars.js` or
//! an Astro layout component. It reads
//! [`headwater_query::Surface::by_precedence`] through the same call
//! [`crate::shelf_index`] already makes, so the order a shelf index prints and
//! the order this file prints never disagree.
//!
//! # One file over many shelves, unlike a shelf index
//!
//! A shelf index writes one file per shelf, and a declaration that names more
//! than one shelf with no `{shelf}` placeholder in its output is refused,
//! because every shelf would write over the last one. A site nav is the
//! opposite: one file is the point, because a reader's sidebar holds every
//! shelf at once. So this emitter always writes exactly one output, and an
//! empty shelf is left out of its `nav:` list rather than reported
//! `Unwritten` — a `site_nav` that covers several shelves is still true with
//! one of them absent, where a `shelf_index` of an empty shelf would be a
//! file whose only content asserts that the shelf is there.
//!
//! # Every scalar is quoted
//!
//! A document's title may hold a colon or a quote — Q16's own title has one —
//! and a plain YAML scalar breaks on either. So every string this module
//! writes, the shelf name, the title, and the path alike, is written as a
//! double-quoted scalar, never as a bare one.
//!
//! # Every path is relative to the corpus root
//!
//! MkDocs resolves a `nav:` path against `docs_dir` and never against the
//! repository root. HW-DR-0036's third reason already reads `docs_dir` as this
//! corpus's root, so a pointer's repository-relative path is not the value
//! MkDocs reads: under `docs_dir: docs`, `docs/decisions/0001-a.md` resolves to
//! `docs/docs/decisions/0001-a.md`, which is nothing. Each entry therefore
//! carries [`crate::shelf_index::relative`] of the corpus root and the
//! pointer's path. A corpus rooted at the repository root is unmoved by this,
//! because the two forms coincide there.

use crate::{pointers, shelf_of, Declaration, Identity, Kind, Output, Plan, Unwritten};
use headwater_census::census::Census;
use headwater_query::{Pointer, Surface};

pub(crate) fn emit(
    surface: &Surface<'_>,
    _census: &Census,
    declaration: &Declaration,
    identity: &Identity,
    plan: &mut Plan,
) {
    let taxonomy = surface.taxonomy();
    // An empty `for` covers every shelf, the same reading `shelf_index` gives
    // it.
    let wanted: Vec<String> = match declaration.shelves.is_empty() {
        true => taxonomy.shelves.iter().map(|s| s.name.clone()).collect(),
        false => declaration.shelves.clone(),
    };

    let mut groups: Vec<(String, Vec<Pointer>)> = Vec::new();
    for name in &wanted {
        if !taxonomy.shelves.iter().any(|shelf| &shelf.name == name) {
            plan.unwritten.push(Unwritten {
                at: declaration.output.clone(),
                kind: Kind::SiteNav,
                reason: format!("names a shelf `{name}` this taxonomy does not declare"),
            });
            return;
        }

        let on_shelf: Vec<_> = surface
            .documents()
            .into_iter()
            .filter(|document| shelf_of(document.path, surface).is_some_and(|s| &s.name == name))
            .collect();

        // Left out of the list rather than written as an empty group. See the
        // module comment: unlike a shelf index, one empty shelf here does not
        // make the whole projection `Unwritten`.
        if on_shelf.is_empty() {
            continue;
        }

        let mut ordered = pointers(surface, &on_shelf);
        surface.by_precedence(&mut ordered);
        groups.push((name.clone(), ordered));
    }

    plan.outputs.push(Output {
        bytes: render(&declaration.output, &groups, &identity.corpus_root),
        path: declaration.output.clone(),
        kind: Kind::SiteNav,
    });
}

/// A double-quoted YAML scalar: `"` and `\` escaped, never written bare.
///
/// The one correctness-critical detail this emitter carries. A title in this
/// corpus routinely holds a colon (`Q36 — Which of MkDocs...`) or a quote, and
/// either breaks an unquoted scalar's shape one level up.
fn quoted(scalar: &str) -> String {
    let mut out = String::with_capacity(scalar.len() + 2);
    out.push('"');
    for ch in scalar.chars() {
        match ch {
            '"' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn render(output: &str, groups: &[(String, Vec<Pointer>)], corpus_root: &str) -> String {
    let mut out = String::new();
    let mark = headwater_mark::marker(Kind::SiteNav.name(), output)
        .unwrap_or_else(|| format!("<!-- {} -->", headwater_mark::MARKER));
    out.push_str(&mark);
    out.push_str("\n\n");
    out.push_str("nav:\n");
    for (shelf, ordered) in groups {
        out.push_str(&format!("  - {}:\n", quoted(shelf)));
        for pointer in ordered {
            out.push_str(&format!(
                "      - {}: {}\n",
                quoted(&crate::label(pointer)),
                quoted(&crate::shelf_index::relative(corpus_root, &pointer.path))
            ));
        }
    }
    out
}
