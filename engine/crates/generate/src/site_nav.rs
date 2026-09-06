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
//!
//! # A group opens with its shelf's generated index, which is no node
//!
//! Every other entry this module writes stands for a document of the graph. A
//! generated index carries no front matter, holds no identifier and is
//! therefore no node, so no reading of `surface.documents()` can reach one and
//! a served index page sits outside the navigation. That is the defect
//! [#528](https://github.com/headwater-ai/headwater/issues/528) reports, and
//! the reason this emitter takes the paths the rest of the plan already wrote
//! rather than a declaration of its own. [HW-DR-0036] records the ruling.
//!
//! The rule is keyed on the **path**, not on the kind of the projection that
//! wrote it: an output that sits directly in the shelf's own directory, whose
//! file stem is `README` or `index`, and which is no document of the graph.
//! Three properties follow from that shape, and each one is a defect avoided.
//! A `verb_index` is covered as well as a `shelf_index`, because this
//! repository's `docs/interfaces/README.md` is the first and a rule keyed on
//! [`Kind::ShelfIndex`] would leave it orphaned. A projection that writes a
//! document, such as the `shelf_sections` output that carries an `identity`
//! block and is already a node of the `spec_series` group, is excluded twice
//! over: by its stem and by the graph. And a shelf that holds no document has
//! no group here, so it takes no index entry either.
//!
//! The label is the constant `Index`. A shelf declaration is
//! `{path, homogeneous, kind}` and carries no reader-facing name, so nothing
//! in the taxonomy supplies one, and the index page's own heading is the
//! machine shelf name, which would restate the group key. [#427] and
//! HW-OBL-0044 own what the entries and the groups of this file are called.
//!
//! [HW-DR-0036]: ../../../../docs/decisions/0036-q36-which-of-mkdocs-docusaurus-or-astro-this-corpus-emits-navigation-for-and-why.md
//! [#427]: https://github.com/headwater-ai/headwater/issues/427

use crate::{pointers, shelf_of, Declaration, Identity, Kind, Output, Plan, Unwritten};
use headwater_census::census::Census;
use headwater_query::{Pointer, Surface};

/// What a group's index entry is called. See the module comment: nothing in a
/// shelf declaration supplies a name, so this emitter chooses a constant.
const INDEX_LABEL: &str = "Index";

/// The one group of the emitted `nav:` list: a shelf, the generated index of
/// that shelf if the plan wrote one, and the shelf's documents in order.
struct Group {
    shelf: String,
    index: Option<String>,
    ordered: Vec<Pointer>,
}

pub(crate) fn emit(
    surface: &Surface<'_>,
    _census: &Census,
    declaration: &Declaration,
    identity: &Identity,
    written: &[String],
    plan: &mut Plan,
) {
    let taxonomy = surface.taxonomy();
    // An empty `for` covers every shelf, the same reading `shelf_index` gives
    // it.
    let wanted: Vec<String> = match declaration.shelves.is_empty() {
        true => taxonomy.shelves.iter().map(|s| s.name.clone()).collect(),
        false => declaration.shelves.clone(),
    };

    // Read once rather than per shelf. `index_of` needs it to tell a generated
    // index from a projection that writes a document of the graph.
    let documents: Vec<&str> = surface
        .documents()
        .into_iter()
        .map(|document| document.path)
        .collect();

    let mut groups: Vec<Group> = Vec::new();
    for name in &wanted {
        let Some(shelf) = taxonomy.shelves.iter().find(|shelf| &shelf.name == name) else {
            plan.unwritten.push(Unwritten {
                at: declaration.output.clone(),
                kind: Kind::SiteNav,
                reason: format!("names a shelf `{name}` this taxonomy does not declare"),
            });
            return;
        };

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
        groups.push(Group {
            shelf: name.clone(),
            index: index_of(shelf.pattern.source(), written, &documents),
            ordered,
        });
    }

    plan.outputs.push(Output {
        bytes: render(&declaration.output, &groups, &identity.corpus_root),
        path: declaration.output.clone(),
        kind: Kind::SiteNav,
    });
}

/// The generated index of one shelf, among the paths the rest of the plan
/// wrote, or `None` where the shelf has none.
///
/// The three conditions are the module comment's rule, and each one is here
/// against a different way of getting this wrong. **Directly in the shelf's
/// own directory**, so a nested `README.md` under a deeper directory of the
/// same shelf is not read as the shelf's index. **A file stem of `README` or
/// `index`**, so a `shelf_sections` output such as `09-open-questions.md` is
/// not swept up and listed a second time under a group that already holds it
/// as a node. **No document of the graph**, so a projection that declares an
/// `identity` block and lands on `README.md` stays a node and is listed once,
/// by the loop above, rather than twice.
///
/// Where a plan somehow wrote two, the first in path order is taken, so that
/// the result is a function of the plan's outputs and not of their order.
fn index_of(pattern: &str, written: &[String], documents: &[&str]) -> Option<String> {
    let directory = crate::shelf_index::directory_of(pattern);
    let mut found: Vec<&String> = written
        .iter()
        .filter(|path| {
            path.rsplit_once('/')
                .is_some_and(|(parent, _)| parent == directory)
                && matches!(
                    crate::shelf_index::file_name(path)
                        .rsplit_once('.')
                        .map(|(stem, _)| stem),
                    Some("README") | Some("index")
                )
                && !documents.contains(&path.as_str())
        })
        .collect();
    found.sort();
    found.first().map(|path| (*path).clone())
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

fn render(output: &str, groups: &[Group], corpus_root: &str) -> String {
    let mut out = String::new();
    let mark = headwater_mark::marker(Kind::SiteNav.name(), output)
        .unwrap_or_else(|| format!("<!-- {} -->", headwater_mark::MARKER));
    out.push_str(&mark);
    out.push_str("\n\n");
    out.push_str("nav:\n");
    for Group {
        shelf,
        index,
        ordered,
    } in groups
    {
        out.push_str(&format!("  - {}:\n", quoted(shelf)));
        // The index first, so a reader who opens a group lands on the page
        // that summarizes it before the first document of it.
        if let Some(path) = index {
            out.push_str(&format!(
                "      - {}: {}\n",
                quoted(INDEX_LABEL),
                quoted(&crate::shelf_index::relative(corpus_root, path))
            ));
        }
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

/// One case per condition of [`index_of`], because the corpus this engine runs
/// over exercises two of the four and no fixture tree reaches the rest.
///
/// The repository's own `docs/spec/09-open-questions.md` is the live case for
/// the stem, and `docs/interfaces/README.md` for the kind the rule does not
/// read. Nothing committed anywhere reaches the nested-directory case or the
/// case of an index that is also a document, so each of those conditions
/// survives its own removal against every other test in this crate. These
/// cases are what fails instead, and each one was run against the removal of
/// the condition it names.
#[cfg(test)]
mod index_tests {
    use super::index_of;

    fn written(paths: &[&str]) -> Vec<String> {
        paths.iter().map(|path| (*path).to_string()).collect()
    }

    #[test]
    fn the_index_of_a_shelf_is_the_readme_in_its_own_directory() {
        let plan = written(&["docs/decisions/README.md", "docs/spec/README.md"]);
        assert_eq!(
            index_of("docs/decisions/**", &plan, &[]),
            Some("docs/decisions/README.md".to_string())
        );
    }

    #[test]
    fn an_index_stem_is_read_as_well_as_a_readme_stem() {
        let plan = written(&["docs/decisions/index.md"]);
        assert_eq!(
            index_of("docs/decisions/**", &plan, &[]),
            Some("docs/decisions/index.md".to_string())
        );
    }

    /// A shelf whose plan holds no index takes no entry, which is what leaves
    /// `a_declared_site_nav_is_held_to_regeneration` listing documents alone.
    #[test]
    fn a_shelf_with_no_index_among_the_outputs_has_none() {
        let plan = written(&["docs/spec/README.md"]);
        assert_eq!(index_of("docs/decisions/**", &plan, &[]), None);
    }

    /// The stem condition. `docs/spec/09-open-questions.md` is the live case:
    /// a `shelf_sections` output on the shelf, which the group already lists.
    #[test]
    fn an_output_on_the_shelf_that_is_not_an_index_is_not_read_as_one() {
        let plan = written(&["docs/spec/09-open-questions.md", "docs/spec/SECTIONS.md"]);
        assert_eq!(index_of("docs/spec/**", &plan, &[]), None);
    }

    /// The directory condition. A shelf's glob reaches every depth under it,
    /// and a `README.md` two directories down indexes that directory rather
    /// than the shelf.
    #[test]
    fn a_readme_below_the_shelf_directory_is_not_the_shelf_s_index() {
        let plan = written(&["docs/decisions/superseded/README.md"]);
        assert_eq!(index_of("docs/decisions/**", &plan, &[]), None);
    }

    /// The graph condition. A projection may declare an `identity` block and
    /// write a document, and the group lists that document as a node. To list
    /// it a second time as the group's index is a duplicate row in a
    /// visitor's sidebar.
    ///
    /// **Nothing downstream reports that, so this case is its only reader.**
    /// MkDocs 1.6.1 builds a `nav:` that names one page twice at exit 0 with
    /// no warning, both within one group and across two, under `--strict`.
    /// Measured on a scratch project whose known-bad arm — a nav entry to a
    /// file that is not there — does exit 1, so the green is a real negative
    /// rather than a harness that cannot fail.
    #[test]
    fn an_index_that_is_a_document_of_the_graph_is_left_to_the_node_listing() {
        let plan = written(&["docs/decisions/README.md"]);
        assert_eq!(
            index_of("docs/decisions/**", &plan, &["docs/decisions/README.md"]),
            None
        );
    }

    /// Two indexes in one directory is a plan nothing writes today. The answer
    /// is a function of the set rather than of the order, so that a second
    /// declaration cannot move an entry by being listed first.
    #[test]
    fn two_indexes_in_one_directory_resolve_by_path_and_not_by_plan_order() {
        let forward = written(&["docs/decisions/README.md", "docs/decisions/index.md"]);
        let backward = written(&["docs/decisions/index.md", "docs/decisions/README.md"]);
        assert_eq!(
            index_of("docs/decisions/**", &forward, &[]),
            index_of("docs/decisions/**", &backward, &[])
        );
        assert_eq!(
            index_of("docs/decisions/**", &forward, &[]),
            Some("docs/decisions/README.md".to_string())
        );
    }
}
