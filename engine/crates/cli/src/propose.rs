// SPDX-License-Identifier: Apache-2.0
//! The overlay fragment `headwater infer` proposes from a tree, and the files
//! the fragment leaves untyped.
//!
//! [Spec 7](../../../../docs/spec/07-distribution-and-federation.md) and
//! HW-DR-0012 name three artifacts of one read of a tree: an overlay, a report
//! of what no proposed shelf or kind explains, and the adoption payload. The
//! payload is built in the verb. This module builds the first two, as a pure
//! function of the census rows and the names the bound taxonomy declares, so
//! a test reads a proposal without starting a process.
//!
//! The rules are the ones `docs/interfaces/headwater-infer.md` states under
//! *The overlay fragment*: group the files that no shelf claims by their
//! parent directory, propose one shelf and one kind for each directory with
//! two or more documents that carry front matter, and give each file left over
//! one reason.
//!
//! The fragment proposes and decides nothing. A tree states which keys and
//! headings its documents share, and it does not state what a document is
//! for, so the purpose each block declares is a first draft for an author to
//! rewrite.

use std::collections::BTreeSet;

use headwater_census::census::{Outcome, Row, Untyped};
use headwater_census::resolve::Untyped as Unresolved;
use headwater_resolve::render::quoted;

/// Front-matter keys the engine reads as something other than a facet.
///
/// `provenance` is the block whose shape belongs to the engine
/// (`headwater_doc::PROVENANCE`), `relations` holds the edges, and `id` is
/// where a document states its identifier. A kind that required any of them
/// as a facet would declare a second meaning for a key that already has one.
const RESERVED: [&str; 3] = ["id", "provenance", "relations"];

/// What the bound taxonomy already declares, read once by the caller.
#[derive(Debug, Default)]
pub struct Declared {
    /// Facet names, so a key that is already a facet is required or permitted
    /// by name and never declared a second time.
    pub facets: BTreeSet<String>,
    /// Kind, shelf and purpose names together, because one directory gives one
    /// name to all three and the name has to be free in each.
    pub names: BTreeSet<String>,
}

impl Declared {
    /// The declared names out of the two readings the verb has loaded.
    pub fn of(
        shape: &headwater_check::shape::Shape,
        taxonomy: &headwater_census::shelves::Taxonomy,
    ) -> Declared {
        Declared {
            facets: shape
                .facets
                .iter()
                .map(|facet| facet.name.clone())
                .collect(),
            names: shape
                .kinds
                .iter()
                .map(|kind| kind.name.clone())
                .chain(taxonomy.kinds.iter().map(|kind| kind.name.clone()))
                .chain(taxonomy.shelves.iter().map(|shelf| shelf.name.clone()))
                .chain(shape.purposes.iter().map(|purpose| purpose.name.clone()))
                .collect(),
        }
    }
}

/// One proposal over one census.
#[derive(Debug, Default)]
pub struct Proposal {
    /// Every row the census classified as nothing, untyped or unreadable.
    pub unexplained: usize,
    /// The fragment, one line each, already indented to go under `add:`.
    /// Empty where no directory holds two documents with front matter.
    pub fragment: Vec<String>,
    /// How many documents the fragment's shelves claim.
    pub covered: usize,
    /// Each file the fragment leaves untyped, with its reason, in census order.
    pub left: Vec<(String, &'static str)>,
}

/// The reason a row stays untyped whatever the fragment proposes, or `None`
/// for a document outside every shelf that carries front matter, which is the
/// only kind of row a proposal can type.
fn reason(row: &Row) -> Option<&'static str> {
    match &row.outcome {
        Outcome::Unreadable(_) => Some("unreadable"),
        Outcome::Untyped(Untyped::NoFrontMatter { .. }) => Some("no front matter"),
        Outcome::Untyped(Untyped::Unresolved { reason, .. }) => match reason {
            Unresolved::NoShelf => match row.document.as_ref() {
                Some(document) if !document.facets.is_empty() => None,
                _ => Some("no front matter"),
            },
            Unresolved::AmbiguousShelf { .. } => Some("ambiguous shelf"),
            Unresolved::MissingDiscriminator { .. }
            | Unresolved::UnknownDiscriminator { .. }
            | Unresolved::DiscriminatorNotAScalar { .. } => Some("discriminator"),
            Unresolved::AbstractKind { .. } => Some("abstract kind"),
        },
        _ => Some("unexplained"),
    }
}

fn unexplained(row: &Row) -> bool {
    matches!(row.outcome.class(), "untyped" | "unreadable")
}

fn directory(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(directory, _)| directory)
}

/// A name in the form the slug rule states: lowercase, and `_` for each
/// character that is not a letter or a digit.
fn slug(text: &str) -> String {
    let slug: String = text
        .chars()
        .map(|c| match c.is_ascii_alphanumeric() {
            true => c.to_ascii_lowercase(),
            false => '_',
        })
        .collect();
    match slug.is_empty() {
        true => "root".to_owned(),
        false => slug,
    }
}

/// A key the fragment can name in a dotted overlay path and a facet list.
fn nameable(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        && !RESERVED.contains(&key)
}

/// The proposal over `rows`, against what `declared` already names.
pub fn propose(rows: &[Row], declared: &Declared) -> Proposal {
    let mut proposal = Proposal {
        unexplained: rows.iter().filter(|row| unexplained(row)).count(),
        ..Proposal::default()
    };

    // The candidates, grouped by directory in census order.
    let mut groups: Vec<(&str, Vec<&Row>)> = Vec::new();
    for row in rows
        .iter()
        .filter(|row| unexplained(row) && reason(row).is_none())
    {
        let at = directory(&row.path);
        match groups.iter_mut().find(|(held, _)| *held == at) {
            Some((_, members)) => members.push(row),
            None => groups.push((at, vec![row])),
        }
    }
    groups.retain(|(_, members)| members.len() >= 2);

    // One name for each directory: the last segment, then the whole path,
    // then a numeric suffix, free of every declared name and of each other.
    let last = |at: &str| slug(at.rsplit('/').next().unwrap_or(at));
    let mut taken = declared.names.clone();
    let mut facets = declared.facets.clone();
    for (at, members) in &groups {
        let shared = groups
            .iter()
            .filter(|(other, _)| last(other) == last(at))
            .count();
        let mut name = match shared {
            1 => last(at),
            _ => slug(at),
        };
        if taken.contains(&name) {
            name = slug(at);
        }
        let base = name.clone();
        let mut n = 2;
        while taken.contains(&name) {
            name = format!("{base}_{n}");
            n += 1;
        }
        taken.insert(name.clone());
        proposal.covered += members.len();
        proposal
            .fragment
            .extend(block(at, &name, members, &mut facets));
    }

    let proposed: BTreeSet<&str> = groups.iter().map(|(at, _)| *at).collect();
    proposal.left = rows
        .iter()
        .filter(|row| unexplained(row))
        .filter_map(|row| match reason(row) {
            Some(why) => Some((row.path.clone(), why)),
            None => (!proposed.contains(directory(&row.path)))
                .then(|| (row.path.clone(), "alone in its directory")),
        })
        .collect();
    proposal
}

/// The lines of one directory's block. `facets` holds every facet name that
/// is declared or proposed so far, so a key two directories share is declared
/// once.
fn block(at: &str, name: &str, members: &[&Row], facets: &mut BTreeSet<String>) -> Vec<String> {
    let documents: Vec<_> = members
        .iter()
        .filter_map(|row| row.document.as_deref())
        .collect();

    // Keys: required where every document writes one, optional where some do,
    // both in the order the documents write them.
    let mut seen: Vec<String> = Vec::new();
    for document in &documents {
        for entry in document.facets.iter() {
            let key = &entry.key.value;
            if !seen.contains(key) {
                seen.push(key.clone());
            }
        }
    }
    let every = |key: &str| {
        documents
            .iter()
            .all(|document| document.facets.get(key).is_some())
    };
    let require: Vec<&String> = seen
        .iter()
        .filter(|key| nameable(key) && every(key))
        .collect();
    let optional: Vec<&String> = seen
        .iter()
        .filter(|key| nameable(key) && !every(key))
        .collect();
    let skipped: Vec<&String> = seen.iter().filter(|key| !nameable(key)).collect();

    // Headings: those of the first document that every other document also
    // writes, under the one comparison the section rule makes.
    let mut sections: Vec<String> = Vec::new();
    if let Some((first, rest)) = documents.split_first() {
        for heading in first
            .body
            .headings()
            .filter(|heading| heading.quote_depth == 0)
        {
            let text = heading.text().trim().to_owned();
            let wanted = [text.clone()];
            let everywhere = rest.iter().all(|document| {
                headwater_check::sections::absent_from_body(&document.body, &wanted).is_empty()
            });
            let again = sections
                .iter()
                .any(|held| held.to_lowercase() == text.to_lowercase());
            if !text.is_empty() && everywhere && !again {
                sections.push(text);
            }
        }
    }

    let list = |items: &[&String]| {
        items
            .iter()
            .map(|item| item.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let words = at.rsplit('/').next().unwrap_or(at).replace(['-', '_'], " ");
    let mut lines = vec![format!(
        "  # {at}: {} documents. Every one writes the keys {}{}{}{}.",
        documents.len(),
        match require.is_empty() {
            true => "(none)".to_owned(),
            false => list(&require),
        },
        match sections.is_empty() {
            true => String::new(),
            false => format!(" and the headings {}", sections.join(", ")),
        },
        match optional.is_empty() {
            true => String::new(),
            false => format!(". Some write {}", list(&optional)),
        },
        match skipped.is_empty() {
            true => String::new(),
            false => format!(
                ". It leaves out {}, which the engine reads as no facet or a dotted path cannot name",
                list(&skipped)
            ),
        },
    )
    // One comment line whatever a heading or a key holds: a line break in
    // the evidence would end the comment and start a line that is not YAML.
    .replace(|c: char| c.is_control(), " ")];
    for key in require.iter().chain(optional.iter()) {
        if facets.insert((*key).clone()) {
            lines.push(format!("  facets.{key}: {{volatility: mutable}}"));
        }
    }
    let quoted_list =
        |items: Vec<&str>| items.into_iter().map(quoted).collect::<Vec<_>>().join(", ");
    lines.push(format!("  purposes.{name}:"));
    lines.push(format!(
        "    intent: {}",
        quoted(&format!(
            "state what a document under {at} is for (a first draft: rewrite it)"
        ))
    ));
    lines.push(format!("    answers: [{}]", quoted(&words)));
    lines.push(format!("  kinds.{name}:"));
    lines.push("    is_a: governed_document".to_owned());
    lines.push(format!("    purpose: {name}"));
    if !require.is_empty() || !optional.is_empty() {
        lines.push("    facets:".to_owned());
        if !require.is_empty() {
            lines.push(format!("      require: [{}]", list(&require)));
        }
        if !optional.is_empty() {
            lines.push(format!("      optional: [{}]", list(&optional)));
        }
    }
    if !sections.is_empty() {
        lines.push("    sections:".to_owned());
        lines.push(format!(
            "      require: [{}]",
            quoted_list(sections.iter().map(String::as_str).collect())
        ));
    }
    let title = {
        let mut chars = words.chars();
        chars.next().map_or_else(String::new, |c| {
            c.to_uppercase().collect::<String>() + chars.as_str()
        })
    };
    lines.push(format!("  shelves.{name}:"));
    lines.push(format!("    title: {}", quoted(&title)));
    lines.push(format!(
        "    path: {}",
        quoted(&match at.is_empty() {
            true => "*.md".to_owned(),
            false => format!("{at}/*.md"),
        })
    ));
    lines.push("    homogeneous: true".to_owned());
    lines.push(format!("    kind: {name}"));
    lines
}

/// The report lines under `what this taxonomy does not explain`, after the
/// heading the verb prints.
pub fn report(proposal: &Proposal) -> Vec<String> {
    let mut lines = Vec::new();
    if proposal.unexplained == 0 {
        lines.push(
            "  every file the census walked classified, so there is nothing to propose".to_owned(),
        );
        return lines;
    }
    let (files, them) = match proposal.unexplained {
        1 => ("file", "it"),
        _ => ("files", "them"),
    };
    lines.push(format!(
        "  {} {files} classified as nothing, and no payload can hold {them}",
        proposal.unexplained
    ));
    match proposal.fragment.is_empty() {
        true => lines.push(
            "  no directory outside every shelf holds two documents with front matter, so there is \
             nothing to propose"
                .to_owned(),
        ),
        false => {
            lines.push(format!(
                "  this fragment proposes a shelf and a kind for {} of them. Put it under the `add:` \
                 key of .headwater/overlay.yml, then run `headwater taxonomy resolve`",
                proposal.covered
            ));
            lines.extend(proposal.fragment.iter().cloned());
            lines.push(String::new());
        }
    }
    if !proposal.left.is_empty() {
        lines.push(format!(
            "  {} {} that no proposed shelf or kind explains",
            proposal.left.len(),
            match proposal.left.len() {
                1 => "file",
                _ => "files",
            }
        ));
        for (path, why) in &proposal.left {
            lines.push(format!("    {path}  {why}"));
        }
    }
    lines
}
