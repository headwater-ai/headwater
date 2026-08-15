// SPDX-License-Identifier: Apache-2.0
//! Graph build: edges, the identifier index, external anchors, and a report of
//! what did not resolve.
//!
//! [Spec 6](../../../../docs/spec/06-engine-architecture.md) puts this phase
//! after the census and before every check: it "resolves relations into edges,
//! indexes identifiers, binds external anchors (code paths, work items, URLs),
//! and reports what it could not resolve".
//!
//! # The input set is the census, and the census is the whole input
//!
//! Only a row that resolved a kind can be an edge endpoint, because both ends
//! of a declared relation are kinds. Which rows those are is
//! [`headwater_census::census::Outcome::node`]'s answer rather than a second
//! reading here, and it holds a typed document and a generated document that
//! declared an identity. A row carries
//! the document it read, so nothing here opens a file. Two passes over one
//! corpus can disagree — an edit between them, or one rule drifting from the
//! other — and the disagreement would be between the denominator and the
//! graph, which is the pair that coverage is computed from.
//!
//! # What this phase decides, and what it refuses to decide
//!
//! It decides identity: which document an identifier names, which node two
//! spellings of an anchor are, and which relation an authored name is. Each one
//! is a **correctness root** in the sense of
//! [spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots):
//! wrong, it produces no error anywhere, and every check over it passes on the
//! wrong graph.
//!
//! It decides nothing about legality. Whether the kinds at the two ends are
//! permitted, whether a reciprocal half is missing, whether an instance
//! attribute is declared, whether a prose link should have been a relation —
//! each is a check, checks carry severities, and the check layer is
//! [M3](https://github.com/headwater-ai/headwater/milestone/3). This crate
//! reports outcomes and no severities, for the same reason
//! [`headwater_census`] does: whether an unresolved target blocks a run is a
//! control's business.
//!
//! # Example
//!
//! ```no_run
//! use headwater_census::{census, shelves::Taxonomy, walk::Corpus};
//! use headwater_graph::{anchors::Resolvers, declarations::Declarations, Config, Graph};
//!
//! let corpus = Corpus::new(".", "docs");
//! let root = headwater_yaml::load("shelves: {}\n").unwrap();
//! let root = root.value.as_map().unwrap();
//! let taxonomy = Taxonomy::read(root).unwrap();
//! let declarations = Declarations::read(root).unwrap();
//!
//! let taken = census::take(&corpus, &taxonomy);
//! let graph = Graph::build(&taken, &declarations, &Resolvers::over(&corpus), &corpus, &Config::default());
//! print!("{}", graph.render(headwater_graph::Detail::Exceptions));
//! ```

pub mod anchors;
pub mod declarations;
pub mod edges;
pub mod index;
pub mod links;

pub use declarations::{Declarations, Direction, Reciprocal};
pub use edges::{Edge, Target, Unbound};
pub use index::{Index, Node};

use headwater_census::census::Census;
use headwater_census::walk::Corpus;

/// The two front-matter keys this phase reads by name.
///
/// Both are parameters because neither is settled. A kind declares
/// `identifier: {scheme: …}` and nothing states which key holds the value.
/// [Q4](../../../../docs/spec/09-decisions.md#q4--relation-storage) does name
/// `relations:`, so that one is a parameter for symmetry rather than for doubt.
/// `.headwater/README.md` records both guesses this repository made.
#[derive(Clone, Debug)]
pub struct Config {
    pub identifier_facet: String,
    pub relations_facet: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            identifier_facet: "id".to_string(),
            relations_facet: "relations".to_string(),
        }
    }
}

/// How much of the graph to write out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Detail {
    /// Every node, every edge and every link. Right for a fixture tree, where
    /// each one is the point.
    EveryRow,
    /// The totals, the anchors, and everything that did not resolve.
    Exceptions,
}

/// The corpus graph.
#[derive(Clone, Debug)]
pub struct Graph {
    pub index: Index,
    pub edges: Vec<Edge>,
    pub links: Vec<links::Link>,
    pub skipped: links::Skipped,
    /// What a `relations:` block declared that produced no edge.
    pub problems: Vec<edges::Reported>,
}

/// What phase A could not make of one document.
///
/// Both lists hold what this build already decided, in the order it recorded
/// them. The type exists so that a check over one document reads the build's
/// answer rather than deriving a second one from the same front matter. Two
/// readings of one `relations:` block are two definitions of a defect, and only
/// the build's reading carries the resolution it did: whether two spellings of
/// one anchor are one target is a fact the resolvers hold, and no reader of
/// front matter alone reaches it.
#[derive(Clone, Debug, Default)]
pub struct Trouble<'a> {
    /// What the identifier index could not make of the document.
    pub identity: Vec<&'a index::Reported>,
    /// What the edge build could not make of its `relations:` block.
    pub relations: Vec<&'a edges::Reported>,
}

impl Trouble<'_> {
    pub fn is_empty(&self) -> bool {
        self.identity.is_empty() && self.relations.is_empty()
    }
}

/// An external anchor, as a node: one identity, however many edges reach it.
#[derive(Clone, Debug)]
pub struct AnchorNode {
    pub anchor_kind: String,
    pub resolver: String,
    pub normalized: String,
    pub excluded_by: Option<String>,
    pub edges: usize,
}

impl Graph {
    pub fn build(
        census: &Census,
        declarations: &Declarations,
        resolvers: &anchors::Resolvers,
        corpus: &Corpus,
        config: &Config,
    ) -> Self {
        let index = Index::build(census, config);
        let (edges, problems) = edges::build(census, &index, declarations, resolvers, config);
        let (links, skipped) = links::bind(census, &index, &corpus.base);
        Graph {
            index,
            edges,
            links,
            skipped,
            problems,
        }
    }

    /// Everything phase A could not make of one document, by path.
    ///
    /// One lookup rather than two, because a defect of the identifier index and
    /// a defect of a `relations:` block are one document's news to one author.
    /// A caller that filtered the two lists itself would be a second reading of
    /// which report belongs to which file.
    pub fn about(&self, path: &str) -> Trouble<'_> {
        Trouble {
            identity: self
                .index
                .defects
                .iter()
                .filter(|defect| defect.path == path)
                .collect(),
            relations: self
                .problems
                .iter()
                .filter(|problem| problem.path == path)
                .collect(),
        }
    }

    /// The distinct anchors the edges reach, in normalized order.
    ///
    /// One entry per identity rather than one per edge. That is what makes the
    /// count readable as an answer to "what does this corpus govern", and it is
    /// the count a resolver that mis-normalizes gets wrong.
    pub fn anchor_nodes(&self) -> Vec<AnchorNode> {
        let mut nodes: Vec<AnchorNode> = Vec::new();
        for edge in &self.edges {
            let Target::Anchor {
                anchor_kind,
                resolver,
                normalized,
                excluded_by,
            } = &edge.target
            else {
                continue;
            };
            match nodes
                .iter_mut()
                .find(|node| &node.normalized == normalized && &node.anchor_kind == anchor_kind)
            {
                Some(node) => node.edges += 1,
                None => nodes.push(AnchorNode {
                    anchor_kind: anchor_kind.clone(),
                    resolver: resolver.clone(),
                    normalized: normalized.clone(),
                    excluded_by: excluded_by.clone(),
                    edges: 1,
                }),
            }
        }
        nodes.sort_by(|a, b| a.normalized.cmp(&b.normalized));
        nodes
    }

    /// Edge counts per outcome class, in the order the classes are declared.
    pub fn counts(&self) -> Vec<(&'static str, usize)> {
        let mut document = 0;
        let mut anchor = 0;
        let mut withheld = 0;
        let mut unbound = 0;
        for edge in &self.edges {
            match edge.target {
                Target::Document { .. } => document += 1,
                Target::Anchor { .. } => anchor += 1,
                Target::Withheld { .. } => withheld += 1,
                Target::Unbound(_) => unbound += 1,
            }
        }
        vec![
            ("to a document", document),
            ("to an anchor", anchor),
            ("withheld", withheld),
            ("unbound", unbound),
        ]
    }

    /// Prose-link counts per binding class.
    pub fn link_counts(&self) -> Vec<(&'static str, usize)> {
        let mut corpus = 0;
        let mut repository = 0;
        let mut external = 0;
        let mut same = 0;
        let mut broken = 0;
        for link in &self.links {
            match &link.binding {
                links::Binding::Corpus { .. } => corpus += 1,
                links::Binding::Repository { .. } => repository += 1,
                links::Binding::External => external += 1,
                links::Binding::SameDocument => same += 1,
                links::Binding::Missing { .. } | links::Binding::Unnormalizable { .. } => {
                    broken += 1;
                }
            }
        }
        vec![
            ("to a corpus file", corpus),
            ("to a repository file", repository),
            ("to this document", same),
            ("outside the repository", external),
            ("broken", broken),
        ]
    }

    /// The graph as text.
    ///
    /// The totals come first and they account for every declared edge half,
    /// whatever `detail` then prints. Same order and same argument as the
    /// census: a reader who checks nothing else still sees the denominator.
    ///
    /// The prose-link accounting is the one thing [`Detail::Exceptions`] leaves
    /// out, and the reason is what a recorded file is for. A node count and an
    /// edge count move when somebody adds a document or declares a relation,
    /// which is the event the record exists to show. A link count moves when
    /// somebody writes a sentence with a link in it, so a fixture that held one
    /// would change on nearly every commit, and a file that changes on every
    /// commit is a file nobody reads. What survives at that grain is the number
    /// that a regression moves: how many links did not resolve.
    pub fn render(&self, detail: Detail) -> String {
        use std::fmt::Write;
        let mut out = String::new();

        let _ = writeln!(
            out,
            "{} nodes, {} declared edge halves",
            self.index.typed.len(),
            self.edges.len()
        );
        for (class, count) in self.counts() {
            if count > 0 {
                let _ = writeln!(out, "  {count:5} {class}");
            }
        }

        if detail == Detail::EveryRow {
            let _ = writeln!(out, "{} prose links", self.links.len());
            for (class, count) in self.link_counts() {
                if count > 0 {
                    let _ = writeln!(out, "  {count:5} {class}");
                }
            }
            if self.skipped.quoted > 0 {
                let _ = writeln!(
                    out,
                    "  {:5} skipped, quoted from another author",
                    self.skipped.quoted
                );
            }
            if self.skipped.images > 0 {
                let _ = writeln!(out, "  {:5} skipped, an image", self.skipped.images);
            }
        } else {
            let broken = self
                .links
                .iter()
                .filter(|link| link.binding.is_broken())
                .count();
            let _ = writeln!(out, "{broken} prose links that did not resolve");
        }

        let anchors = self.anchor_nodes();
        if !anchors.is_empty() {
            out.push_str("\nanchors\n");
            for node in anchors {
                let _ = writeln!(
                    out,
                    "  {:5} {} `{}` via {}",
                    node.edges, node.anchor_kind, node.normalized, node.resolver
                );
                if let Some(pattern) = node.excluded_by {
                    let _ = writeln!(
                        out,
                        "        inside `{pattern}`, which this corpus declares is not corpus content"
                    );
                }
            }
        }

        if detail == Detail::EveryRow {
            out.push_str("\nnodes\n");
            for node in &self.index.typed {
                let _ = writeln!(
                    out,
                    "  {} {}\n    {}",
                    node.kind.clone().unwrap_or_default(),
                    node.id,
                    node.path
                );
            }
            out.push_str("\nedges\n");
            for edge in &self.edges {
                let _ = writeln!(
                    out,
                    "  {} --{}--> {}\n    {}",
                    edge.source.id,
                    edge.name,
                    edge.normalized_target(),
                    describe(&edge.target)
                );
            }
            out.push_str("\nlinks\n");
            for link in &self.links {
                let _ = writeln!(
                    out,
                    "  {} -> {}\n    {}",
                    link.source_path, link.destination, link.binding
                );
            }
        }

        let report = self.exceptions();
        if !report.is_empty() {
            out.push('\n');
            for line in report {
                out.push_str(&line);
            }
        }
        out
    }

    /// Every fact that says something did not resolve, in path order.
    fn exceptions(&self) -> Vec<String> {
        let mut lines: Vec<(String, usize, String)> = Vec::new();

        for defect in &self.index.defects {
            lines.push((
                defect.path.clone(),
                defect.span.map(|span| span.start.line).unwrap_or(0),
                format!(
                    "{}\n  identifier: {}\n",
                    location(&defect.path, defect.span),
                    defect.defect
                ),
            ));
        }
        for problem in &self.problems {
            lines.push((
                problem.path.clone(),
                problem.span.map(|span| span.start.line).unwrap_or(0),
                format!(
                    "{}\n  relations: {}\n",
                    location(&problem.path, problem.span),
                    problem.problem
                ),
            ));
        }
        for edge in &self.edges {
            let Target::Unbound(unbound) = &edge.target else {
                continue;
            };
            lines.push((
                edge.source.path.clone(),
                edge.span.start.line,
                format!(
                    "{}\n  edge: `{}` -> {} {}\n",
                    location(&edge.source.path, Some(edge.span)),
                    edge.name,
                    edge.raw_target,
                    unbound
                ),
            ));
        }
        for link in &self.links {
            if !link.binding.is_broken() {
                continue;
            }
            lines.push((
                link.source_path.clone(),
                link.span.start.line,
                format!(
                    "{}\n  link: `{}` {}\n",
                    location(&link.source_path, Some(link.span)),
                    link.destination,
                    link.binding
                ),
            ));
        }

        lines.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        lines.into_iter().map(|(_, _, line)| line).collect()
    }
}

fn location(path: &str, span: Option<headwater_yaml::Span>) -> String {
    match span {
        Some(span) => format!("{path}:{}:{}", span.start.line, span.start.col),
        None => path.to_string(),
    }
}

fn describe(target: &Target) -> String {
    match target {
        Target::Document { path, kind, .. } => format!("{kind} at {path}"),
        Target::Anchor {
            anchor_kind,
            resolver,
            ..
        } => format!("{anchor_kind} via {resolver}"),
        Target::Withheld {
            anchor_kind,
            profile,
        } => {
            format!("{anchor_kind} withheld by the profile `{profile}`")
        }
        Target::Unbound(unbound) => format!("unbound: {unbound}"),
    }
}
