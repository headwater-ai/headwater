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
//! So the index has two shelves. [`Index::typed`] is the graph's node set: only
//! a typed document can be an edge endpoint, because both ends of a declared
//! relation are kinds. [`Index::untyped`] is every other document that still
//! declares an identifier, and it exists to answer the second question and
//! nothing else. Nothing in it is a node.
//!
//! # The identifier facet is a parameter
//!
//! A kind declares `identifier: {scheme: …}`, and nothing in the specification
//! states which front-matter key holds the minted value. `.headwater/README.md`
//! records the guess this repository made — a key named `id` — and
//! [13 — Open obligations](../../../../docs/spec/13-open-obligations.md)
//! carries the gap. The engine takes the name rather than fixing it, so that
//! settling the question changes a declaration and not this file.

use crate::Config;
use headwater_census::census::{Census, Outcome, Row};
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
    Duplicate { id: String, other: String },
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
            Defect::Duplicate { id, other } => {
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
            let kind = match &row.outcome {
                Outcome::Typed { kind, .. } => Some(kind.clone()),
                _ => None,
            };
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

    /// A node of the graph, by identifier.
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.typed.iter().find(|node| node.id == id)
    }

    /// A document that carries this identifier and no kind.
    ///
    /// This is the whole reason the second shelf exists: it turns "fix this
    /// link" into "fix that document" for the one case where the link is right.
    pub fn near_miss(&self, id: &str) -> Option<&Node> {
        self.untyped.iter().find(|node| node.id == id)
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
    fn report_duplicates(&mut self) {
        let mut seen: Vec<(&str, &str)> = Vec::new();
        let mut found: Vec<Reported> = Vec::new();
        for node in self.typed.iter().chain(self.untyped.iter()) {
            match seen.iter().find(|(id, _)| *id == node.id) {
                Some((id, first)) => found.push(Reported {
                    path: node.path.clone(),
                    span: node.id_span,
                    defect: Defect::Duplicate {
                        id: (*id).to_string(),
                        other: (*first).to_string(),
                    },
                }),
                None => seen.push((&node.id, &node.path)),
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
