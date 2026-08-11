// SPDX-License-Identifier: Apache-2.0
//! Scoped views.
//!
//! **Item 2 of the spike.** Spec 12: "The view exposes *only* what the scope
//! declared. A `Document`-scoped check physically cannot read a sibling."
//!
//! Three properties are what make that true here, and the compile-fail tests in
//! `tests/ui/` are the evidence for each:
//!
//! 1. A view borrows for `'g` and holds no path back to the graph. There is no
//!    accessor to omit, because `Document` itself reaches nothing (see `model`).
//! 2. Constructors are `pub(crate)`. Code outside this crate that somehow holds
//!    a `&Graph` still cannot manufacture a `DocumentView` over an arbitrary
//!    document and hand it to a check.
//! 3. The views are not `'static`, so `Any`-based downcasting cannot reach the
//!    concrete type behind them. There is no cast that recovers the graph.

use crate::frontmatter::{FrontMatter, Value};
use crate::model::{Document, Edge, Graph, Kind, Relation};
use crate::span::{Span, Spanned};

/// What a `Document`-scoped check sees: one document, and nothing else.
pub struct DocumentView<'g> {
    doc: &'g Document,
}

impl<'g> DocumentView<'g> {
    pub(crate) fn new(doc: &'g Document) -> Self {
        Self { doc }
    }

    pub fn path(&self) -> &'g str {
        &self.doc.path
    }

    pub fn id(&self) -> &'g str {
        &self.doc.id
    }

    pub fn kind(&self) -> Kind {
        self.doc.kind
    }

    pub fn front(&self) -> &'g FrontMatter {
        &self.doc.front
    }

    pub fn facet(&self, key: &str) -> Option<&'g Value> {
        self.doc.front.get(key).map(|v| &v.value)
    }

    pub fn facet_span(&self, key: &str) -> Option<Span> {
        self.doc.front.key_span(key)
    }

    /// Where to anchor a finding about something that is *not* there.
    pub fn front_span(&self) -> Span {
        self.doc.front.span
    }

    pub fn headings(&self) -> &'g [Spanned<String>] {
        &self.doc.body.headings
    }

    pub fn links(&self) -> &'g [Spanned<String>] {
        &self.doc.body.links
    }

    pub fn word_count(&self) -> usize {
        self.doc.body.word_count
    }

    pub fn content_hash(&self) -> u128 {
        self.doc.content_hash
    }
}

/// What an `Edge`-scoped check sees: one relation instance and both endpoints.
pub struct EdgeView<'g> {
    edge: &'g Edge,
    from: &'g Document,
    to: Option<&'g Document>,
    graph: &'g Graph,
}

impl<'g> EdgeView<'g> {
    pub(crate) fn new(
        edge: &'g Edge,
        from: &'g Document,
        to: Option<&'g Document>,
        graph: &'g Graph,
    ) -> Self {
        Self {
            edge,
            from,
            to,
            graph,
        }
    }

    pub fn relation(&self) -> Relation {
        self.edge.relation
    }

    pub fn span(&self) -> Span {
        self.edge.span
    }

    pub fn from(&self) -> DocumentView<'g> {
        DocumentView::new(self.from)
    }

    /// `None` when the edge is dangling.
    pub fn to(&self) -> Option<DocumentView<'g>> {
        self.to.map(DocumentView::new)
    }

    pub fn dangling_target(&self) -> Option<&'g str> {
        match &self.edge.target {
            crate::model::Target::Dangling(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// The one question an edge check may ask of the wider graph: does the
    /// reciprocal edge exist? This is scoped on purpose. It answers a yes/no
    /// about the two endpoints already in scope, and it exposes no third
    /// document.
    pub fn reciprocal_exists(&self) -> bool {
        let (Some(inverse), crate::model::Target::Resolved(to)) =
            (self.edge.relation.inverse(), &self.edge.target)
        else {
            return false;
        };
        self.graph.has_edge(*to, self.edge.from, inverse)
    }
}

/// What a `Corpus`-scoped check sees: everything. Declaring this scope is what
/// buys the access, and it is what costs the check its cache granularity.
pub struct CorpusView<'g> {
    graph: &'g Graph,
}

impl<'g> CorpusView<'g> {
    pub(crate) fn new(graph: &'g Graph) -> Self {
        Self { graph }
    }

    pub fn documents(&self) -> impl Iterator<Item = DocumentView<'g>> + '_ {
        self.graph.documents().iter().map(DocumentView::new)
    }

    pub fn len(&self) -> usize {
        self.graph.documents().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
