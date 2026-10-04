// SPDX-License-Identifier: Apache-2.0
//! The site navigation: one file, each shelf's documents in path order,
//! written in the shape MkDocs's `nav:` key reads.
//!
//! # Path order, where a shelf index prints the reading order
//!
//! A shelf index lists a shelf in the order [`headwater_query::Surface::by_precedence`]
//! derives, because it is the page a reader opens to learn what to read first.
//! This file is a sidebar, which a reader uses to find a document they already
//! know the name or the number of. Precedence puts decision 101 before decision
//! 86 and gives no reader a rule to find either, so this emitter sorts a
//! shelf's documents by path. The numbered shelves carry a zero-padded prefix,
//! so path order is numeric order, and an unnumbered shelf reads alphabetically
//! by file name. This amends HW-DR-0036, which had this file follow
//! `by_precedence`, and
//! [HW-DR-0108](../../../../docs/decisions/0108-the-site-sidebar-lists-a-shelf-in-path-order-and-the-shelf-index-keeps-the-reading-order.md)
//! records the amendment.
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
//! The label is the shelf the index indexes. MkDocs serves a nav label as the
//! page's `<title>` and as its search-index entry, above the front-matter meta
//! title, above the first `<h1>` and above the file name, so a constant here
//! is ten browser tabs, ten bookmarks and ten search results that a reader
//! cannot tell apart. That was measured over the served bytes on ten pages and
//! is [#567]. A shelf declaration is `{path, homogeneous, kind}` and carries no
//! reader-facing name, so the machine shelf name is what there is, and it
//! restates the group key in the sidebar. That restatement is the smaller
//! defect and it is deliberate. [#538] is where a shelf display name replaces
//! both strings through one function, and this is the one call site it
//! redirects. [#427] and HW-OBL-0044 own what the entries and the groups of
//! this file are called.
//!
//! [HW-DR-0036]: ../../../../docs/decisions/0036-q36-which-of-mkdocs-docusaurus-or-astro-this-corpus-emits-navigation-for-and-why.md
//! [#427]: https://github.com/headwater-ai/headwater/issues/427
//! [#538]: https://github.com/headwater-ai/headwater/issues/538
//! [#567]: https://github.com/headwater-ai/headwater/issues/567

use crate::{
    pointers, shelf_of, Declaration, Identity, Kind, NavSectionBody, Output, Plan, Unwritten,
};
use headwater_census::census::Census;
use headwater_query::{Pointer, Surface};

/// The one group of the emitted `nav:` list: a shelf, the generated index of
/// that shelf if the plan wrote one, and the shelf's documents in order.
struct Group {
    shelf: String,
    index: Option<String>,
    /// Generated pages in the shelf's own directory that are neither its index
    /// nor a document of the graph, each with its label. The consumer surface
    /// page is the one kind today (#1051). Without an entry here such a page is
    /// served and named by no sidebar, which is #528 again.
    pages: Vec<(String, String)>,
    ordered: Vec<Pointer>,
}

pub(crate) fn emit(
    surface: &Surface<'_>,
    _census: &Census,
    declaration: &Declaration,
    identity: &Identity,
    written: &[String],
    pages: &[(String, &str)],
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

    // A shelf a section names leaves the top level for that section (#1681).
    // The parse has already refused a shelf named twice, so this is a
    // partition and not a choice.
    let in_sections: Vec<&str> = declaration
        .sections
        .iter()
        .flat_map(|section| section.shelves())
        .collect();

    // One group per non-empty shelf, or `Err` with the first name this
    // taxonomy does not declare.
    let group_of = |name: &str| -> Result<Option<Group>, String> {
        let Some(shelf) = taxonomy.shelves.iter().find(|shelf| shelf.name == name) else {
            return Err(name.to_string());
        };

        let on_shelf: Vec<_> = surface
            .documents()
            .into_iter()
            .filter(|document| shelf_of(document.path, surface).is_some_and(|s| s.name == name))
            .collect();

        // Left out of the list rather than written as an empty group. See the
        // module comment: unlike a shelf index, one empty shelf here does not
        // make the whole projection `Unwritten`.
        if on_shelf.is_empty() {
            return Ok(None);
        }

        // Path order, and not `by_precedence`. A sidebar is a list a reader
        // looks a document up in, and a shelf of numbered documents has to
        // read 86 and then 101. The shelf index keeps the derived reading
        // order. See the module comment.
        let mut ordered = pointers(surface, &on_shelf);
        ordered.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(Some(Group {
            // The declared display name, through the one function three
            // emitters read. `Group::shelf` is a label and nothing addresses
            // a shelf by it: `index_of` takes the pattern and the filter above
            // takes the key.
            shelf: crate::shelf_label(shelf),
            index: index_of(shelf.pattern.source(), written, &documents),
            pages: pages_of(shelf.pattern.source(), pages, &documents),
            ordered,
        }))
    };
    let shelves_of = |names: &mut dyn Iterator<Item = &str>| -> Result<Vec<Entry>, String> {
        let mut out = Vec::new();
        for name in names {
            if let Some(group) = group_of(name)? {
                out.push(Entry::Shelf(group));
            }
        }
        Ok(out)
    };

    let built = (|| -> Result<Vec<Entry>, String> {
        // The shelves no section names come first, in the order they always
        // had, so a declaration with no `sections` writes the bytes it wrote
        // before the member existed.
        let mut entries = shelves_of(
            &mut wanted
                .iter()
                .map(String::as_str)
                .filter(|name| !in_sections.contains(name)),
        )?;
        for section in &declaration.sections {
            let children = match &section.body {
                NavSectionBody::Shelves(names) => {
                    shelves_of(&mut names.iter().map(String::as_str))?
                }
                NavSectionBody::Groups(groups) => {
                    let mut out = Vec::new();
                    for group in groups {
                        let children = shelves_of(&mut group.shelves.iter().map(String::as_str))?;
                        // An empty group is left out, as an empty shelf is.
                        if !children.is_empty() {
                            out.push(Entry::Labeled {
                                title: group.title.clone(),
                                children,
                            });
                        }
                    }
                    out
                }
            };
            if !children.is_empty() {
                entries.push(Entry::Labeled {
                    title: section.title.clone(),
                    children,
                });
            }
        }
        Ok(entries)
    })();
    let entries = match built {
        Ok(entries) => entries,
        Err(name) => {
            plan.unwritten.push(Unwritten {
                at: declaration.output.clone(),
                kind: Kind::SiteNav,
                reason: format!("names a shelf `{name}` this taxonomy does not declare"),
            });
            return;
        }
    };

    // The directory of every shelf a `noindex` section names, relative to the
    // corpus root as a `nav:` path is. Read from the declaration and not from
    // the documents, so an empty shelf keeps its place on the list and a page
    // that later lands on it is not indexed in the meantime.
    let noindex: Vec<String> = declaration
        .sections
        .iter()
        .filter(|section| section.noindex)
        .flat_map(|section| section.shelves())
        .filter_map(|name| taxonomy.shelves.iter().find(|shelf| shelf.name == name))
        .map(|shelf| prefix_of(shelf.pattern.source(), &identity.corpus_root))
        .collect();

    for entry in &entries {
        entry.paths(&mut plan.navigation);
    }
    plan.outputs.push(Output {
        bytes: render(
            &declaration.output,
            &entries,
            &noindex,
            &identity.corpus_root,
        ),
        path: declaration.output.clone(),
        kind: Kind::SiteNav,
        committed: true,
    });
}

/// One entry of the emitted `nav:` tree: a shelf with its pages, or a labeled
/// section or group that holds more entries (#1681).
enum Entry {
    Shelf(Group),
    Labeled { title: String, children: Vec<Entry> },
}

impl Entry {
    /// Every path under this entry, in the order the file lists them.
    fn paths(&self, out: &mut Vec<String>) {
        match self {
            Entry::Shelf(group) => {
                out.extend(group.index.iter().cloned());
                out.extend(group.pages.iter().map(|(path, _)| path.clone()));
                out.extend(group.ordered.iter().map(|pointer| pointer.path.clone()));
            }
            Entry::Labeled { children, .. } => {
                for child in children {
                    child.paths(out);
                }
            }
        }
    }
}

/// The path prefix that every page of a shelf starts with, relative to the
/// corpus root, as a site template compares it against a page's source path.
///
/// The prefix is the literal text of the pattern before its first glob
/// character, so every path the pattern admits starts with it. A shelf
/// `process/decisions/**` takes `process/decisions/`, which never matches
/// `process/decisions-old/`. A shelf `internal-*.md` takes `internal-`. A
/// shelf whose pattern holds no glob claims one file, and its prefix is that
/// file. Where the literal text admits more than the pattern does, as
/// `taxonomies/` does for `taxonomies/*/doctrine.md`, the prefix covers more
/// than the shelf. That is the safe direction for `noindex`: it can hide a
/// page that is not on the shelf, and it never leaves a page of the shelf
/// indexed.
fn prefix_of(pattern: &str, corpus_root: &str) -> String {
    let stop = pattern.find(['*', '?', '[', '{']).unwrap_or(pattern.len());
    let literal = &pattern[..stop];
    if corpus_root.is_empty() {
        return literal.to_string();
    }
    if literal == corpus_root {
        return String::new();
    }
    literal
        .strip_prefix(&format!("{corpus_root}/"))
        .unwrap_or(literal)
        .to_string()
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

/// The labelled generated pages directly in a shelf's own directory that no
/// document of the graph is, in path order.
fn pages_of(pattern: &str, pages: &[(String, &str)], documents: &[&str]) -> Vec<(String, String)> {
    let directory = crate::shelf_index::directory_of(pattern);
    let mut found: Vec<(String, String)> = pages
        .iter()
        .filter(|(path, _)| {
            path.rsplit_once('/')
                .is_some_and(|(parent, _)| parent == directory)
                && !documents.contains(&path.as_str())
        })
        .map(|(path, label)| (path.clone(), (*label).to_string()))
        .collect();
    found.sort();
    found
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

fn render(output: &str, entries: &[Entry], noindex: &[String], corpus_root: &str) -> String {
    let mut out = String::new();
    let mark = headwater_mark::marker(Kind::SiteNav.name(), output)
        .unwrap_or_else(|| format!("<!-- {} -->", headwater_mark::MARKER));
    out.push_str(&mark);
    out.push_str("\n\n");
    out.push_str("nav:\n");
    for entry in entries {
        render_entry(&mut out, entry, "  ", corpus_root);
    }
    // The pages a site template marks `noindex`, as path prefixes relative to
    // the corpus root. MkDocs merges an `extra` mapping of an `INHERIT`ed file
    // into the parent's, so a template reads this as `config.extra`. Written
    // only where a section asks for it, so a declaration with none writes the
    // file it wrote before (#1681).
    if !noindex.is_empty() {
        out.push_str("\nextra:\n  headwater_noindex:\n");
        for prefix in noindex {
            out.push_str(&format!("    - {}\n", quoted(prefix)));
        }
    }
    out
}

/// One entry at one depth. `pad` is the indentation of its own `- ` line, and
/// every child sits four spaces deeper, the shape the flat file always had.
fn render_entry(out: &mut String, entry: &Entry, pad: &str, corpus_root: &str) {
    let inner = format!("{pad}    ");
    match entry {
        Entry::Labeled { title, children } => {
            out.push_str(&format!("{pad}- {}:\n", quoted(title)));
            for child in children {
                render_entry(out, child, &inner, corpus_root);
            }
        }
        Entry::Shelf(Group {
            shelf,
            index,
            pages,
            ordered,
        }) => {
            out.push_str(&format!("{pad}- {}:\n", quoted(shelf)));
            // The index first, so a reader who opens a group lands on the page
            // that summarizes it before the first document of it. Its label is
            // the shelf, because MkDocs serves this string as that page's title.
            if let Some(path) = index {
                out.push_str(&format!(
                    "{inner}- {}: {}\n",
                    quoted(shelf),
                    quoted(&crate::shelf_index::relative(corpus_root, path))
                ));
            }
            for (path, label) in pages {
                out.push_str(&format!(
                    "{inner}- {}: {}\n",
                    quoted(label),
                    quoted(&crate::shelf_index::relative(corpus_root, path))
                ));
            }
            for pointer in ordered {
                out.push_str(&format!(
                    "{inner}- {}: {}\n",
                    quoted(&crate::label(pointer)),
                    quoted(&crate::shelf_index::relative(corpus_root, &pointer.path))
                ));
            }
        }
    }
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

/// The `noindex` prefix of a shelf, against each shape a shelf pattern takes.
/// A template marks a page whose source path starts with the prefix, so every
/// page the pattern admits has to start with it. Before #1681's first verify
/// the prefix was cut at the directory and given a `/`, so `docs/internal-*.md`
/// wrote `internal-/`, which no page starts with.
#[cfg(test)]
mod prefix_tests {
    use super::prefix_of;

    #[test]
    fn a_directory_glob_takes_the_directory_with_its_slash() {
        assert_eq!(
            prefix_of("docs/process/decisions/**", "docs"),
            "process/decisions/"
        );
        assert_eq!(prefix_of("decisions/**", ""), "decisions/");
    }

    #[test]
    fn a_glob_inside_a_file_name_takes_the_literal_text_before_it() {
        assert_eq!(prefix_of("docs/internal-*.md", "docs"), "internal-");
        assert!("internal-plan.md".starts_with(&prefix_of("docs/internal-*.md", "docs")));
        assert_eq!(prefix_of("docs/decisions*", "docs"), "decisions");
    }

    #[test]
    fn a_glob_below_a_directory_takes_the_directory_before_it() {
        assert_eq!(
            prefix_of("docs/taxonomies/*/doctrine.md", "docs"),
            "taxonomies/"
        );
    }

    #[test]
    fn a_pattern_over_the_whole_root_takes_the_empty_prefix() {
        assert_eq!(prefix_of("docs/**", "docs"), "");
    }

    #[test]
    fn a_pattern_with_no_glob_is_its_one_file() {
        assert_eq!(prefix_of("docs/plan.md", "docs"), "plan.md");
    }
}
