// SPDX-License-Identifier: Apache-2.0
//! The deterministic query surface: reads of the graph a run already built.
//!
//! [Spec 6](../../../../docs/spec/06-engine-architecture.md#library) makes the
//! CLI "a thin shell over a library API — load, graph, check, query, generate",
//! and says that "editor integrations, the MCP server, and CI adapters all
//! consume the library directly". This crate is `query`. It adds no phase to a
//! run: the census carries every row and the document it parsed, the graph
//! carries the edges and the identifier index, and every answer here is a
//! projection of those two.
//!
//! # What ships, and why the list is spec 5's rather than spec 6's
//!
//! [Spec 5](../../../../docs/spec/05-ai-integration.md#agent-surfaces) names
//! the tools of the query class: `route`, `governing_docs_for_path`,
//! `resolve_identifier`, `related`, `explain`, `check`. Those are the reads
//! here, and `check` is the runner rather than this crate.
//!
//! Spec 6's CLI list also holds `headwater query <expression>`, and no document
//! of the specification states what an expression is. This crate therefore has
//! no `query` function, and
//! [13 — Open obligations](../../../../docs/spec/13-open-obligations.md)
//! carries the gap. The posture is the resolver's over a `$package` reference:
//! name the gap rather than invent a form and have every later adopter live
//! with it.
//!
//! # Determinism, concretely
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#determinism-concretely)
//! binds the check layer to "same corpus, same lock, same injected clock,
//! byte-identical output", and a read that an agent depends on is held to the
//! same bar. Three rules carry it here.
//!
//! - **Every score is an integer.** A ranking over floating-point weights is
//!   reproducible on one machine and not obviously across two, and the ordering
//!   is the whole product of a route.
//! - **Every order is total.** Score, then score, then path. No comparison ends
//!   in a tie that a sort may break either way.
//! - **No read consults a clock, an environment variable, or a network.** A
//!   read of a document reads what the census already parsed.
//!
//! # A read never returns content
//!
//! Spec 5: "A task description resolves to a ranked, budget-capped set of
//! **pointers**: paths and one-line summaries, never content." So [`Pointer`]
//! carries a path, a summary and the facts a reader needs to decide whether to
//! open the document, and no body ever crosses this boundary.

pub mod explain;
pub mod json;
pub mod mcp;
pub mod route;

pub use explain::Explanation;
pub use route::{Budget, Route, Silence};

use headwater_census::census::{Census, Outcome, Row};
use headwater_census::resolve::Resolution;
use headwater_census::shelves::Taxonomy;
use headwater_check::Shape;
use headwater_graph::declarations::{Declarations, Direction, Governs};
use headwater_graph::{Edge, Graph, Target};
use headwater_yaml::Mapping;

/// The facet role that carries the routing cue, from spec 2's closed registry.
const SCENT: &str = "scent";

/// The facet role that carries what a document is called, from the same
/// registry. A projection that writes a heading reads this and never a facet
/// name: `title` means something in one taxonomy and nothing in the next.
const NAME: &str = "name";

/// The provenance member that states what stands behind a document.
///
/// [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed):
/// "The provenance block is the exception, and its shape belongs to the
/// engine." So the two names below are the engine's own, like the identifier
/// and relations facets that [`headwater_graph::Config`] holds, and no taxonomy
/// declares them.
const PROVENANCE: &str = "provenance";
const WARRANT: &str = "warrant";

/// The warrant value that spec 5 makes a pointer state out loud.
const ASSERTED: &str = "asserted";

/// The reads, over one census and the graph built from it.
///
/// It borrows rather than owns for the reason the check layer's views do: a run
/// has already built both, and a surface that copied them would answer about a
/// corpus that no run evaluated.
pub struct Surface<'a> {
    census: &'a Census,
    graph: &'a Graph,
    shape: &'a Shape,
    taxonomy: &'a Taxonomy,
    relations: &'a Declarations,
    /// The facet in the `scent` role, which is where a summary lives. A
    /// taxonomy that declares none has no cue to serve, and every pointer then
    /// carries a path and no summary.
    scent: Option<String>,
    /// The facet in the `name` role, which is where a heading lives. A taxonomy
    /// that declares none names no document, and a projection that needs a
    /// heading declines rather than inventing one.
    name: Option<String>,
}

/// One classified document, as a read sees it.
#[derive(Clone, Copy)]
pub struct Document<'a> {
    pub path: &'a str,
    pub kind: &'a str,
    /// The identifier, and `None` for a typed document that declares none.
    /// Such a document is not a node of the graph, and it is still a document a
    /// route may offer.
    pub id: Option<&'a str>,
    pub facets: &'a Mapping,
    /// The derivation the census recorded, which is what `explain` prints.
    pub derivation: &'a Resolution,
}

/// A pointer: what a read offers instead of content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pointer {
    pub path: String,
    pub id: Option<String>,
    pub kind: String,
    /// The reader intent the kind serves, and `None` where the taxonomy
    /// declares none for it.
    pub purpose: Option<String>,
    /// The `scent` facet: the cue a reader decides on.
    pub summary: Option<String>,
    /// The warrant, carried only when it is one a reader must be told about.
    /// Spec 5: a pointer to an `asserted` document "states that warrant beside
    /// the summary", because nobody accepted the document and an agent that
    /// follows the pointer has to know before it reads.
    pub unwarranted: bool,
}

/// What a traversal offers: the far end of one edge, with the cue that stands
/// at it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Neighbour {
    /// The relation as the far end is reached: the declared name for an
    /// outbound edge, and the inverse where one is declared for an inbound one.
    pub relation: String,
    /// Whether this document declared the edge or the far end did.
    pub inbound: bool,
    /// The far end. An edge that reaches an anchor or nothing carries no
    /// pointer, and [`Neighbour::target`] is what it reached instead.
    pub pointer: Option<Pointer>,
    /// The far end as a string, whatever it resolved to.
    pub target: String,
    /// Spec 5: "`related` and `explain` serve the cue where one exists, and the
    /// target's summary otherwise."
    pub cue: Option<String>,
    /// Whether the cue came from the edge rather than from the target.
    pub cue_is_declared: bool,
    /// Which end governs the reading of the pair, derived and never declared.
    pub governs: Governs,
}

impl<'a> Surface<'a> {
    pub fn over(
        census: &'a Census,
        graph: &'a Graph,
        shape: &'a Shape,
        taxonomy: &'a Taxonomy,
        relations: &'a Declarations,
    ) -> Self {
        Surface {
            census,
            graph,
            shape,
            taxonomy,
            relations,
            scent: shape.facet_in_role(SCENT).map(|facet| facet.name.clone()),
            name: shape.facet_in_role(NAME).map(|facet| facet.name.clone()),
        }
    }

    pub fn shape(&self) -> &'a Shape {
        self.shape
    }

    pub fn taxonomy(&self) -> &'a Taxonomy {
        self.taxonomy
    }

    /// The graph this surface reads.
    ///
    /// Every other read here is a projection of the graph and hands back a
    /// pointer or a neighbour, because spec 5 rules that "a read never returns
    /// content". An export is the one caller that needs the graph itself: it
    /// carries the property graph rather than an answer computed from it, and a
    /// second traversal built out of [`Surface::related`] would be a second
    /// derivation of the edge set that could disagree with this one.
    pub fn graph(&self) -> &'a Graph {
        self.graph
    }

    pub fn relations(&self) -> &'a Declarations {
        self.relations
    }

    /// Every classified document, in the census's own path order.
    pub fn documents(&self) -> Vec<Document<'a>> {
        self.census
            .rows
            .iter()
            .filter_map(|row| self.of(row))
            .collect()
    }

    /// One census row as a document, and `None` for a row that carries no kind.
    fn of(&self, row: &'a Row) -> Option<Document<'a>> {
        let Outcome::Typed { kind, derivation } = &row.outcome else {
            return None;
        };
        let document = row.document.as_ref()?;
        Some(Document {
            path: &row.path,
            kind,
            id: self
                .graph
                .index
                .by_path(&row.path)
                .and_then(|entry| entry.id.as_deref()),
            facets: &document.facets,
            derivation,
        })
    }

    /// The document at a path, or the one an identifier names.
    ///
    /// A path is tried first, because a path is what a reader has in hand when
    /// they ask. Both are accepted at one argument because spec 6 writes the
    /// verb as `headwater explain <path|identifier>`.
    pub fn find(&self, target: &str) -> Option<Document<'a>> {
        if let Some(document) = self.census.rows.iter().find(|row| row.path == target) {
            return self.of(document);
        }
        let node = self.graph.index.node(target)?;
        self.census
            .rows
            .iter()
            .find(|row| row.path == node.path)
            .and_then(|row| self.of(row))
    }

    /// The identifier index, as a read: spec 5's `resolve_identifier`.
    ///
    /// A miss reports the near miss the index already computes, because an
    /// identifier that differs by its case or its separator is the common
    /// mistake and a bare "not found" sends the caller to grep.
    pub fn resolve_identifier(&self, id: &str) -> Resolved {
        match self.graph.index.node(id) {
            Some(node) => match self.find(&node.path) {
                Some(document) => Resolved::Document(self.pointer(&document)),
                // A node of the index whose row carries no kind cannot happen:
                // the index types its nodes from the same census. Reported
                // rather than unwrapped, because a panic in a read an agent
                // calls is worse than a true statement about a strange corpus.
                None => Resolved::Nothing,
            },
            None => match self.graph.index.near_miss(id) {
                Some(node) => Resolved::NearMiss(node.id.clone()),
                None => Resolved::Nothing,
            },
        }
    }

    /// The documents that govern a path: spec 5's `governing_docs_for_path`.
    ///
    /// The path is matched against what an edge reached rather than against a
    /// pattern, so a resolver that normalizes two spellings of one target makes
    /// both spellings answer here. The relation has to be one whose source
    /// governs, which is spec 2's derivation and not a name this function
    /// knows: an adopter who declares a governance relation of their own gets
    /// it for nothing.
    pub fn governing_docs_for_path(&self, path: &str) -> Vec<Pointer> {
        let mut pointers: Vec<Pointer> = Vec::new();
        for edge in &self.graph.edges {
            if self.governs_of(edge) != Governs::Source {
                continue;
            }
            let reached = match &edge.target {
                Target::Anchor { normalized, .. } => normalized.as_str(),
                Target::Document { path, .. } => path.as_str(),
                _ => continue,
            };
            if reached != path {
                continue;
            }
            let Some(document) = self.find(&edge.source.path) else {
                continue;
            };
            let pointer = self.pointer(&document);
            if !pointers.contains(&pointer) {
                pointers.push(pointer);
            }
        }
        pointers.sort_by(|a, b| a.path.cmp(&b.path));
        pointers
    }

    /// Every edge that reaches this document or leaves it: spec 5's `related`.
    ///
    /// Both directions, because a reader who holds a document is owed what
    /// points at it as much as what it points at. The order is the declaration
    /// order of the outbound edges and then of the inbound ones, which is the
    /// order the graph holds and so the order two runs agree on.
    pub fn related(&self, document: &Document<'a>) -> Vec<Neighbour> {
        let mut neighbours = Vec::new();
        for edge in &self.graph.edges {
            if edge.source.path == document.path {
                neighbours.push(self.neighbour(edge, false));
            }
        }
        for edge in &self.graph.edges {
            let Target::Document { path, .. } = &edge.target else {
                continue;
            };
            if path == document.path {
                neighbours.push(self.neighbour(edge, true));
            }
        }
        neighbours
    }

    fn neighbour(&self, edge: &Edge, inbound: bool) -> Neighbour {
        let declared = self.relations.named(&edge.declared);
        let relation = match inbound {
            false => edge.declared.clone(),
            true => declared
                .as_ref()
                .and_then(|named| named.relation.inverse.clone())
                .unwrap_or_else(|| format!("{} of", edge.declared)),
        };
        let far = match inbound {
            true => self.find(&edge.source.path),
            false => match &edge.target {
                Target::Document { path, .. } => self.find(path),
                _ => None,
            },
        };
        let pointer = far.as_ref().map(|document| self.pointer(document));
        // Spec 5: the cue where one exists, and the target's summary otherwise.
        // The cue is an attribute the referring document wrote, so it stands on
        // the outbound reading of the edge and never on the inbound one: a
        // reader arriving from the far end did not read that text.
        let declared_cue = match inbound {
            true => None,
            false => edge
                .attributes
                .iter()
                .find(|entry| entry.key.value == "cue")
                .and_then(|entry| entry.value.value.as_scalar())
                .map(|scalar| scalar.text.clone()),
        };
        Neighbour {
            relation,
            inbound,
            cue_is_declared: declared_cue.is_some(),
            cue: declared_cue
                .or_else(|| pointer.as_ref().and_then(|pointer| pointer.summary.clone())),
            target: match inbound {
                true => edge.source.path.clone(),
                false => edge.normalized_target(),
            },
            governs: match (self.governs_of(edge), inbound) {
                (Governs::Neither, _) => Governs::Neither,
                (end, false) => end,
                // An inbound edge is the same edge read from the other side, so
                // the end that governs is the other one.
                (Governs::Source, true) => Governs::Target,
                (Governs::Target, true) => Governs::Source,
            },
            pointer,
        }
    }

    /// Which end of an edge governs the reading, as this edge is written.
    ///
    /// The relation derives it for its declared orientation, and an author who
    /// wrote the inverse wrote the same edge from the other end. So the answer
    /// is flipped for an inverse edge. Without that flip a succession pair
    /// where both halves are written reports two contradictory orders, and the
    /// half that wins is whichever the sort reached last. The fixture tree
    /// caught exactly that: it offered the superseded decision first.
    ///
    /// The direction comes off the edge and never from the name. `Edge::declared`
    /// is the canonical relation, resolved when the edge was built, so asking
    /// the declarations for it again always answers `AsDeclared` and the flip
    /// never fires. That is the defect this comment is here to keep fixed.
    fn governs_of(&self, edge: &Edge) -> Governs {
        let Some(named) = self.relations.named(&edge.declared) else {
            return Governs::Neither;
        };
        match (named.relation.governs(), edge.direction) {
            (governs, Direction::AsDeclared) => governs,
            (Governs::Source, Direction::Inverse) => Governs::Target,
            (Governs::Target, Direction::Inverse) => Governs::Source,
            (Governs::Neither, _) => Governs::Neither,
        }
    }

    /// A pointer to one document.
    pub fn pointer(&self, document: &Document<'a>) -> Pointer {
        Pointer {
            path: document.path.to_string(),
            id: document.id.map(str::to_string),
            kind: document.kind.to_string(),
            purpose: self
                .shape
                .purpose_of(document.kind)
                .map(|purpose| purpose.name.clone()),
            summary: self.summary(document),
            unwarranted: self.warrant(document).as_deref() == Some(ASSERTED),
        }
    }

    /// The cue that stands on the document itself.
    pub fn summary(&self, document: &Document<'a>) -> Option<String> {
        let name = self.scent.as_deref()?;
        document
            .facets
            .get(name)
            .and_then(|node| node.value.as_scalar())
            .map(|scalar| scalar.text.clone())
    }

    /// What the document is called: the value of the facet in the `name` role.
    ///
    /// `None` covers two states that a caller has to tell apart, and
    /// [`Surface::name_facet`] is how. Either this taxonomy declares no facet
    /// in the role at all, or it declares one and this document leaves it
    /// empty. The repair is a declaration in the first case and an edit to the
    /// document in the second.
    pub fn name(&self, document: &Document<'a>) -> Option<String> {
        let facet = self.name.as_deref()?;
        document
            .facets
            .get(facet)
            .and_then(|node| node.value.as_scalar())
            .map(|scalar| scalar.text.clone())
            .filter(|text| !text.trim().is_empty())
    }

    /// The facet this taxonomy puts in the `name` role, when it declares one.
    pub fn name_facet(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// What stands behind the document, from the provenance block.
    pub fn warrant(&self, document: &Document<'a>) -> Option<String> {
        document
            .facets
            .get(PROVENANCE)
            .and_then(|node| node.value.as_map())
            .and_then(|map| map.get(WARRANT))
            .and_then(|node| node.value.as_scalar())
            .map(|scalar| scalar.text.clone())
    }
}

/// What an identifier resolved to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resolved {
    Document(Pointer),
    /// No node carries the identifier, and one carries something close to it.
    NearMiss(String),
    Nothing,
}

impl Neighbour {
    /// One traversal as one line: where it goes, under what relation, with the
    /// cue that stands there and the end that governs the reading.
    ///
    /// One rendering, used by `explain` and by the MCP `related` tool. A client
    /// and a terminal that read different text about one edge is the drift this
    /// crate is written to avoid.
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut line = format!(
            "{} {} {}",
            match self.inbound {
                true => "from",
                false => "to",
            },
            self.target,
            self.relation
        );
        if let Some(cue) = &self.cue {
            let _ = write!(
                line,
                " — {cue}{}",
                match self.cue_is_declared {
                    true => "",
                    false => " (the target's own summary)",
                }
            );
        }
        line.push_str(match self.governs {
            Governs::Source => " [this document governs the reading]",
            Governs::Target => " [that document governs the reading]",
            Governs::Neither => "",
        });
        line
    }
}

impl Pointer {
    /// The pointer as one line: the path, the summary, and the warrant where a
    /// reader has to be told.
    pub fn render(&self) -> String {
        let mut line = self.path.clone();
        if let Some(summary) = &self.summary {
            line.push_str(" — ");
            line.push_str(summary);
        }
        if self.unwarranted {
            line.push_str(" [asserted: nobody accepted this document]");
        }
        line
    }
}

/// The words of a string, lowercased, in the order they were written.
///
/// A term is two characters or more. One character discriminates nothing at the
/// scale a corpus works at, and it costs a comparison against every declared
/// purpose.
///
/// This is deliberately not a tokenizer with a language model behind it. Spec 5
/// makes routing "a search over the corpus's intentional structure, not over
/// its prose", so the words here are matched against declarations that an
/// author wrote, and the engine holds no vocabulary of its own.
pub fn terms(text: &str) -> Vec<String> {
    let mut terms: Vec<String> = Vec::new();
    for word in text.split(|c: char| !c.is_alphanumeric()) {
        if word.chars().count() < 2 {
            continue;
        }
        let word = word.to_lowercase();
        if !terms.contains(&word) {
            terms.push(word);
        }
    }
    terms
}
