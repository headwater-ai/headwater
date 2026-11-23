// SPDX-License-Identifier: Apache-2.0
//! The identifier index, and the path index beside it.
//!
//! # Why there are two indexes and not one
//!
//! [Q4](../../../../docs/spec/09-decisions.md#q4--relation-storage) rules that
//! an edge target is an identifier and never a path, so edge resolution reads
//! the identifier index. Every prose link in this corpus is written as a
//! relative path, so link binding reads the path index. Neither index can do
//! the other's work, and a single index keyed on both would admit a target
//! written as a path, which is the ruling Q4 made.
//!
//! # Why the identifier index holds untyped documents too
//!
//! [Spec 4](../../../../docs/spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)
//! wants a defect reported to the person who can repair it. An edge whose
//! target names a document that exists and carries no kind is a defect in *that
//! document*. An edge whose target names nothing at all is a defect in *this
//! link*. Told apart, each one reaches an author who can act. Reported as one
//! class, the first sends its author to repair a link that is already correct.
//!
//! So the index has two shelves. [`Index::typed`] is the graph's node set: a
//! document is on it when it resolved a kind, because both ends of a declared
//! relation are kinds. [`Index::untyped`] is every other document that still
//! declares an identifier, and it exists to answer the second question and
//! nothing else. Nothing in it is a node.
//!
//! # A generated document that declared an identity is on the node shelf
//!
//! Which rows resolved a kind is
//! [`headwater_census::census::Outcome::node`]'s answer and not this module's,
//! and a generated file that carries front matter is one of them.
//! [The `headwater generate` contract](../../../../docs/interfaces/headwater-generate.md#the-generated-file-marker)
//! excuses such a file from every *check*, on the ground that an author cannot repair its
//! content in the file. Its identity is a different fact: it comes from the
//! declaration that writes the file, and `generate --check` is what holds it.
//!
//! Reading the exemption as a statement about identity is what made a
//! projection unable to write a file that other documents cite. The property
//! that makes an artifact worth generating is that many documents depend on it,
//! and that was the property that made it ungeneratable.
//!
//! # The identifier facet is a parameter
//!
//! A kind declares `identifier: {scheme: …}`, and
//! [HW-DR-0068](../../../../docs/decisions/0068-the-front-matter-key-that-carries-a-minted-identifier-is-id.md)
//! names `id` as the front-matter key that holds the minted value, as a global
//! fixed rule with no per-kind override. The engine takes the name as a
//! parameter rather than a literal, so a later change to the declaration
//! changes one default and not this file.
//!
//! # A former identifier names the document that carried it
//!
//! [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#identifiers)
//! says an identifier resolves without its document and is never reused. A
//! document whose identifier changed lists the old value in the facet the
//! taxonomy declares in the `former_identifier` role (#2025), and
//! [`Config::former_identifier_facet`] names that facet. The index records each
//! former identifier on the node that lists it, and [`Index::node`] and
//! [`Index::near_miss`] answer one: a live identifier first, then a former one.
//! The node they return carries its live identifier, so every caller that asks
//! for "the document this identifier names", an edge binder included, gets the
//! live identifier back. HW-DR-0068 still fixes `id` as the key of the minted
//! value. A former identifier is a second key that resolves to the same
//! document, not a second minted value.
//!
//! Never reused means a former identifier is in the same namespace as the live
//! ones. One that equals a live identifier, or that two documents list, is the
//! [`Defect::Duplicate`] that two live identifiers are.

use crate::Config;
use headwater_census::census::{Census, Row};
use headwater_yaml::Span;

/// A document, as a node of the graph or as a near miss beside it.
#[derive(Clone, Debug)]
pub struct Node {
    pub id: String,
    /// Relative to the repository root, with `/` separators.
    pub path: String,
    /// The kind the census resolved, and `None` for a document it did not type.
    pub kind: Option<String>,
    /// The span of the identifier facet's key, which a finding about *this*
    /// document's identity anchors to.
    pub id_span: Option<Span>,
    /// The identifiers this document carried before [`Node::id`], each with
    /// the span of the item that lists it. Empty when the taxonomy declares no
    /// facet in the `former_identifier` role or the document lists none. See
    /// the module comment.
    pub former: Vec<(String, Option<Span>)>,
}

/// What the index could not make of a document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Defect {
    /// A typed document that declares no identifier. It can be neither end of
    /// an edge, so every edge it declares and every edge that names it is lost.
    NoIdentifier { kind: String },
    /// The identifier facet is a sequence or a mapping, and an identifier is a
    /// word.
    NotAScalar,
    /// Two documents claim one identifier. Both are in the index and neither is
    /// reachable: a target that names it resolves to two documents, and
    /// [spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#identifiers)
    /// says an identifier is never reused.
    ///
    /// The report carries the **other** claimant whole: its path, and the span
    /// of the identifier key it wrote. [`Reported`] carries the claimant that
    /// comes second in path order, and which document is second is a fact about
    /// the corpus rather than about either file. A rule that reported one of
    /// the two and not the other would report path order as the defect, and it
    /// cannot anchor a finding in a file whose line nothing told it.
    ///
    /// `former` is true when either claim is a former identifier (#2025), so
    /// that a repair can name the list a value has to leave as well as the key.
    Duplicate {
        id: String,
        other: String,
        other_span: Option<Span>,
        former: bool,
    },
}

impl std::fmt::Display for Defect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Defect::NoIdentifier { kind } => write!(
                f,
                "a typed `{kind}` that declares no identifier, so no edge can name it"
            ),
            Defect::NotAScalar => write!(
                f,
                "the identifier is not a scalar, and an identifier is a word"
            ),
            Defect::Duplicate { id, other, .. } => {
                write!(
                    f,
                    "`{id}` is also declared on {other}, and an identifier is never reused"
                )
            }
        }
    }
}

/// One defect, and the document it is about.
#[derive(Clone, Debug)]
pub struct Reported {
    pub path: String,
    pub span: Option<Span>,
    pub defect: Defect,
}

/// Every document of a census, reachable by identifier and by path.
#[derive(Clone, Debug, Default)]
pub struct Index {
    /// The node set: typed documents that declare an identifier, in path order.
    pub typed: Vec<Node>,
    /// Documents that carry an identifier and no kind. Not nodes.
    pub untyped: Vec<Node>,
    /// Every row of the census by path, so that a prose link to a file the
    /// corpus excluded says so rather than reading as a broken link.
    pub paths: Vec<PathEntry>,
    pub defects: Vec<Reported>,
}

/// One census row, seen from the path side.
#[derive(Clone, Debug)]
pub struct PathEntry {
    pub path: String,
    /// The census's own word for what became of the file.
    pub class: &'static str,
    /// The identifier, when the file has one.
    pub id: Option<String>,
    pub kind: Option<String>,
}

impl Index {
    /// Build both indexes from a census.
    pub fn build(census: &Census, config: &Config) -> Self {
        let mut index = Index::default();

        for row in &census.rows {
            let kind = row.outcome.node().map(|(kind, _)| kind.to_string());
            let id = identifier(row, config, &mut index.defects, kind.as_deref());

            index.paths.push(PathEntry {
                path: row.path.clone(),
                class: row.outcome.class(),
                id: id.as_ref().map(|(id, _)| id.clone()),
                kind: kind.clone(),
            });

            let Some((id, id_span)) = id else { continue };
            let node = Node {
                id,
                path: row.path.clone(),
                kind: kind.clone(),
                id_span,
                former: former_identifiers(row, config),
            };
            if kind.is_some() {
                index.typed.push(node);
            } else {
                index.untyped.push(node);
            }
        }

        index.report_duplicates();
        index
    }

    /// A node of the graph, by identifier: its live identifier first, and then
    /// an identifier it lists as former. The node carries its live identifier
    /// either way, so a caller that binds to it binds to that one.
    pub fn node(&self, id: &str) -> Option<&Node> {
        find(&self.typed, id)
    }

    /// A document that carries this identifier and no kind, live or former, in
    /// the same order as [`Index::node`].
    ///
    /// This is the whole reason the second shelf exists: it turns "fix this
    /// link" into "fix that document" for the one case where the link is right.
    pub fn near_miss(&self, id: &str) -> Option<&Node> {
        find(&self.untyped, id)
    }

    /// The node of the graph at this path: a typed document with an
    /// identifier. A path is never how a relation names one (#1410), and this
    /// is what lets a binder say which identifier was meant.
    pub fn typed_at(&self, path: &str) -> Option<&Node> {
        self.typed.iter().find(|node| node.path == path)
    }

    pub fn by_path(&self, path: &str) -> Option<&PathEntry> {
        self.paths.iter().find(|entry| entry.path == path)
    }

    /// Two documents that claim one identifier, reported against the second.
    ///
    /// The comparison is over both shelves at once. An identifier that a typed
    /// document and an untyped one both claim is the same collision, and
    /// reporting it only inside the node set would hide the half that a later
    /// typing pass turns into a real conflict.
    ///
    /// A former identifier is a claim like a live one (#2025), so the
    /// comparison is over every live and every former identifier. A document
    /// that lists one value twice, or its own live identifier as former, claims
    /// nothing twice against anybody, and that is not reported.
    fn report_duplicates(&mut self) {
        // (identifier, claimant path, span of the claim, whether it is former)
        let mut seen: Vec<(&str, &str, Option<Span>, bool)> = Vec::new();
        let mut found: Vec<Reported> = Vec::new();
        for node in self.typed.iter().chain(self.untyped.iter()) {
            let live = std::iter::once((&node.id, node.id_span, false));
            let former = node.former.iter().map(|(id, span)| (id, *span, true));
            for (id, span, is_former) in live.chain(former) {
                match seen.iter().find(|(seen, ..)| *seen == id.as_str()) {
                    Some((_, path, _, _)) if *path == node.path.as_str() => {}
                    Some((_, path, first_span, first_former)) => found.push(Reported {
                        path: node.path.clone(),
                        span,
                        defect: Defect::Duplicate {
                            id: id.clone(),
                            other: (*path).to_string(),
                            // The first claimant's own key, so that a rule can
                            // report the collision in that file too. Without it a
                            // finding could land only where path order put the
                            // report, and path order is not an author's doing.
                            other_span: *first_span,
                            former: is_former || *first_former,
                        },
                    }),
                    None => seen.push((id.as_str(), node.path.as_str(), span, is_former)),
                }
            }
        }
        self.defects.extend(found);
    }
}

/// The identifier a row declares, if it declares one this index can use.
fn identifier(
    row: &Row,
    config: &Config,
    defects: &mut Vec<Reported>,
    kind: Option<&str>,
) -> Option<(String, Option<Span>)> {
    let document = row.document.as_ref()?;
    let Some(declared) = document.facets.get(&config.identifier_facet) else {
        // Only a typed document is expected to carry one. A file that no shelf
        // claims has not been asked for an identifier by anybody.
        if let Some(kind) = kind {
            defects.push(Reported {
                path: row.path.clone(),
                span: Some(document.block),
                defect: Defect::NoIdentifier {
                    kind: kind.to_string(),
                },
            });
        }
        return None;
    };

    let span = document.facets.key_span(&config.identifier_facet);
    match declared.value.as_scalar() {
        Some(scalar) => Some((scalar.text.clone(), span)),
        None => {
            defects.push(Reported {
                path: row.path.clone(),
                span,
                defect: Defect::NotAScalar,
            });
            None
        }
    }
}

/// A node on one shelf by identifier: a live match first, then a former one.
///
/// Live first, so that a former identifier that collides with a live one
/// still finds the document that holds it live. That collision is a
/// [`Defect::Duplicate`] in any case.
fn find<'a>(shelf: &'a [Node], id: &str) -> Option<&'a Node> {
    shelf.iter().find(|node| node.id == id).or_else(|| {
        shelf
            .iter()
            .find(|node| node.former.iter().any(|(former, _)| former == id))
    })
}

/// The former identifiers a row lists, each with the span of its item.
///
/// The facet takes one word or a list of words. An item that is not a word is
/// the meta-schema's to report, not this index's, so it is skipped here.
fn former_identifiers(row: &Row, config: &Config) -> Vec<(String, Option<Span>)> {
    let Some(facet) = config.former_identifier_facet.as_deref() else {
        return Vec::new();
    };
    let Some(declared) = row
        .document
        .as_ref()
        .and_then(|document| document.facets.get(facet))
    else {
        return Vec::new();
    };
    if let Some(scalar) = declared.value.as_scalar() {
        return vec![(scalar.text.clone(), Some(declared.span))];
    }
    declared
        .value
        .as_seq()
        .unwrap_or_default()
        .iter()
        .filter_map(|item| {
            item.value
                .as_scalar()
                .map(|scalar| (scalar.text.clone(), Some(item.span)))
        })
        .collect()
}
