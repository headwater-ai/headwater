// SPDX-License-Identifier: Apache-2.0
//! The typed graph.
//!
//! One design rule in this module carries item 2, and it is worth stating
//! before the code rather than after it:
//!
//! > A `Document` holds no reference to any other `Document`, and no reference
//! > to the `Graph`.
//!
//! Edges live on the `Graph` as index pairs. So a `&Document` is not a foothold
//! from which a check can walk anywhere, and `DocumentView` is safe *because of
//! the data model*, not because the view happens to omit an accessor. That
//! distinction is the whole of what the spike is testing, and it is the part a
//! reviewer should check hardest.

use crate::frontmatter::FrontMatter;
use crate::span::{Span, Spanned};
use std::collections::HashMap;

pub type DocIdx = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    Decision,
    Standard,
    Guide,
    Evidence,
    Unclassified,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Decision => "decision",
            Kind::Standard => "standard",
            Kind::Guide => "guide",
            Kind::Evidence => "evidence",
            Kind::Unclassified => "unclassified",
        }
    }

    pub fn parse(s: &str) -> Kind {
        match s {
            "decision" => Kind::Decision,
            "standard" => Kind::Standard,
            "guide" => Kind::Guide,
            "evidence" => Kind::Evidence,
            _ => Kind::Unclassified,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Relation {
    Supersedes,
    SupersededBy,
    Governs,
    GovernedBy,
    ConflictsWith,
}

impl Relation {
    pub fn facet(self) -> &'static str {
        match self {
            Relation::Supersedes => "supersedes",
            Relation::SupersededBy => "superseded_by",
            Relation::Governs => "governs",
            Relation::GovernedBy => "governed_by",
            Relation::ConflictsWith => "conflicts_with",
        }
    }

    /// The declared inverse, which the reciprocity check enforces.
    pub fn inverse(self) -> Option<Relation> {
        match self {
            Relation::Supersedes => Some(Relation::SupersededBy),
            Relation::SupersededBy => Some(Relation::Supersedes),
            Relation::Governs => Some(Relation::GovernedBy),
            Relation::GovernedBy => Some(Relation::Governs),
            Relation::ConflictsWith => Some(Relation::ConflictsWith),
        }
    }

    pub fn all() -> &'static [Relation] {
        &[
            Relation::Supersedes,
            Relation::SupersededBy,
            Relation::Governs,
            Relation::GovernedBy,
            Relation::ConflictsWith,
        ]
    }
}

/// The body, parsed once and reduced to what a check can ask about.
#[derive(Clone, Debug, Default)]
pub struct Body {
    pub headings: Vec<Spanned<String>>,
    pub links: Vec<Spanned<String>>,
    pub word_count: usize,
}

#[derive(Clone, Debug)]
pub struct Document {
    pub path: String,
    pub id: String,
    pub kind: Kind,
    pub front: FrontMatter,
    pub body: Body,
    /// Hash of the file's bytes. One of the inputs to every cache key that
    /// covers this document.
    pub content_hash: u128,
    // Deliberately absent: any handle on a sibling or on the Graph.
}

#[derive(Clone, Debug)]
pub enum Target {
    Resolved(DocIdx),
    Dangling(String),
}

#[derive(Clone, Debug)]
pub struct Edge {
    pub from: DocIdx,
    pub target: Target,
    pub relation: Relation,
    /// Where the declaration sits in the source, so a finding about the edge
    /// points at the line the author wrote.
    pub span: Span,
}

#[derive(Debug, Default)]
pub struct Graph {
    docs: Vec<Document>,
    edges: Vec<Edge>,
    by_id: HashMap<String, DocIdx>,
    /// Edge indices touching each document, in *both* directions. Spec 12 makes
    /// this the unit of change-scoped invalidation, so it is built once here
    /// rather than derived per run.
    incident: Vec<Vec<usize>>,
}

impl Graph {
    pub fn build(docs: Vec<Document>) -> Graph {
        let mut by_id = HashMap::with_capacity(docs.len());
        for (i, d) in docs.iter().enumerate() {
            by_id.entry(d.id.clone()).or_insert(i);
        }

        let mut edges = Vec::new();
        for (i, d) in docs.iter().enumerate() {
            for relation in Relation::all() {
                for item in d.front.scalar_seq(relation.facet()) {
                    let target = match by_id.get(&item.value) {
                        Some(&j) => Target::Resolved(j),
                        None => Target::Dangling(item.value.clone()),
                    };
                    edges.push(Edge {
                        from: i,
                        target,
                        relation: *relation,
                        span: item.span,
                    });
                }
            }
        }

        let mut incident = vec![Vec::new(); docs.len()];
        for (ei, e) in edges.iter().enumerate() {
            incident[e.from].push(ei);
            if let Target::Resolved(j) = e.target {
                if j != e.from {
                    incident[j].push(ei);
                }
            }
        }

        Graph {
            docs,
            edges,
            by_id,
            incident,
        }
    }

    pub fn documents(&self) -> &[Document] {
        &self.docs
    }

    /// Hand the parsed documents back, so an incremental run can replace the
    /// one that changed and rebuild without re-parsing the other 999. The
    /// change-scoped path is not meaningful without this.
    pub fn into_documents(self) -> Vec<Document> {
        self.docs
    }

    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    pub fn doc(&self, idx: DocIdx) -> &Document {
        &self.docs[idx]
    }

    pub fn index_of(&self, id: &str) -> Option<DocIdx> {
        self.by_id.get(id).copied()
    }

    /// Every edge incident to this document, in both directions. This is the
    /// clause spec 12 calls the subgraph problem.
    pub fn incident_edges(&self, idx: DocIdx) -> &[usize] {
        &self.incident[idx]
    }

    pub fn has_edge(&self, from: DocIdx, to: DocIdx, relation: Relation) -> bool {
        self.incident[from].iter().any(|&ei| {
            let e = &self.edges[ei];
            e.from == from && e.relation == relation && matches!(e.target, Target::Resolved(t) if t == to)
        })
    }
}
