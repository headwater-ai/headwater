// SPDX-License-Identifier: Apache-2.0
//! What comes back, and the five things this engine can confirm about it.
//!
//! # This is the reproducible half of an unreproducible mechanism
//!
//! [`Report::read`] is a pure function of a return file, the tree it is read
//! over, and this code. Run it twice on one file and it writes one set of
//! bytes. That is deliberate and it is the whole reason a sweep is worth
//! acting on: the *sample* a model returned cannot be reproduced, and every
//! statement this engine makes about that sample can be.
//!
//! # What is confirmed
//!
//! 1. **The taxonomy.** The file names the lock it was planned against. A file
//!    planned against another one is refused whole, because a finding about a
//!    document that a different taxonomy typed is a finding about a corpus that
//!    is not in front of this run.
//! 2. **Membership.** Every path a finding names is a classified document of
//!    this corpus. A path outside it is a document nobody here governs, and a
//!    path that is not there at all is the plainest sign that a model invented
//!    it.
//! 3. **The citation.** Every quotation is in the document the finding
//!    attributes it to. Spec 4's third constraint asks a finding to quote the
//!    passages it believes are in conflict, "so a human can adjudicate in
//!    seconds"; a quotation the file does not hold costs that human the whole
//!    minute instead. This is the test that catches the failure a prose
//!    judgment fails by, and it is the reason a rejected finding never reaches
//!    a reader.
//! 4. **Novelty.** A finding whose class implies a relation, over two documents
//!    the graph already joins by that relation, is refused. Spec 4's fourth
//!    constraint: "A finding that restates an edge already in the front matter
//!    is a defect in the sweep, not a finding about the corpus."
//! 5. **The endpoints of a proposal.** A proposed edge joins two documents the
//!    relation admits at the ends it names, and it never joins a document to
//!    itself. The report prints the front matter that would declare the edge,
//!    so a proposal the taxonomy refuses is an instruction whose reader is then
//!    told by `relation.endpoint.not_permitted` that their own commit is wrong.
//!    The judge is `headwater_check::Shape::descends_from`, which is the same
//!    function that rule reads.
//!
//! # What is not confirmed, and is not confirmable
//!
//! Whether the two passages actually contradict each other. That is the
//! judgment, it stays the model's, and the report says so on the line that
//! carries it. The engine confirms that there is something real to read and
//! never that the reading is right.
//!
//! # A quotation matches with whitespace collapsed, and nothing else relaxed
//!
//! A passage that spans a paragraph break is one quotation, and the line breaks
//! inside it are not content. So both sides collapse every run of whitespace to
//! one space before the search. Nothing else is relaxed: no case folding, no
//! punctuation, no stemming. A paraphrase is refused, which is the intended
//! pressure on the agent — copy the passage.

use crate::{Class, PROVENANCE};
use headwater_census::census::{Census, Outcome};
use headwater_check::paint::{dim, paint, ColorMode, Role};
use headwater_check::{Finding, Severity, Shape};
use headwater_graph::declarations::{Declarations, Direction, Reciprocal};
use headwater_graph::edges::Target;
use headwater_graph::Graph;
use headwater_yaml::value::{Mapping, Value};
use headwater_yaml::Spanned;
use std::collections::BTreeMap;
use std::path::Path;

/// The tree a return file is read over.
pub struct Tree<'a> {
    pub root: &'a Path,
    pub census: &'a Census,
    pub graph: &'a Graph,
    pub relations: &'a Declarations,
    /// The kind hierarchy, which is what says whether a document may sit at an
    /// end of a relation. A proposal is front matter a person is told to write,
    /// so it answers to the same endpoint declarations `relation.endpoint`
    /// reads over a written edge.
    pub shape: &'a Shape,
    /// The digest of the lock this tree carries.
    pub lock: &'a str,
}

/// Why a whole return file reported nothing.
///
/// Every one of these still exits zero. See the crate comment: an exit status
/// that reads a model's output is the one thing the sweep may not have.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The file is not YAML.
    Unparsed(Vec<String>),
    /// The file parsed and is not a mapping of the shape the plan prints.
    Malformed(String),
    /// The file names a taxonomy this tree does not carry.
    TaxonomyMoved { claimed: String, tree: String },
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refusal::Unparsed(errors) => {
                write!(f, "the file did not parse as YAML: {}", errors.join("; "))
            }
            Refusal::Malformed(what) => write!(f, "{what}"),
            Refusal::TaxonomyMoved { claimed, tree } => write!(
                f,
                "it was planned against taxonomy {claimed} and this tree carries {tree}"
            ),
        }
    }
}

/// Why one finding did not reach a reader.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reason {
    UnknownClass(String),
    Missing(&'static str),
    NotAMember(String),
    EvidenceOffFinding(String),
    QuoteNotFound {
        path: String,
        quote: String,
    },
    UnknownRelation(String),
    UnknownEndpoint(String),
    /// One document at both ends.
    SelfEdge {
        relation: String,
        id: String,
    },
    /// A document whose kind the relation does not admit at the end it was
    /// proposed at. This is the refusal that keeps a sweep from printing front
    /// matter that `relation.endpoint.not_permitted` would then report as an
    /// error, in the corpus of whoever ran the sweep.
    EndpointNotPermitted {
        relation: String,
        /// `from` or `to`, as the proposal wrote them rather than as the
        /// relation declares them.
        end: &'static str,
        id: String,
        kind: String,
        permitted: Vec<String>,
    },
    AlreadyDeclared {
        from: String,
        relation: String,
        to: String,
    },
}

impl std::fmt::Display for Reason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reason::UnknownClass(name) => write!(
                f,
                "`{name}` is not a class of this sweep. The classes are: {}",
                Class::ALL
                    .iter()
                    .map(|class| class.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Reason::Missing(what) => write!(f, "it carries no {what}"),
            Reason::NotAMember(path) => {
                write!(f, "`{path}` is not a classified document of this corpus")
            }
            Reason::EvidenceOffFinding(path) => write!(
                f,
                "it quotes `{path}`, which is not one of the documents it names"
            ),
            Reason::QuoteNotFound { path, quote } => {
                write!(f, "`{path}` does not hold the quotation {}", quoted(quote))
            }
            Reason::UnknownRelation(name) => {
                write!(f, "`{name}` is not a relation this taxonomy declares")
            }
            Reason::UnknownEndpoint(id) => {
                write!(
                    f,
                    "`{id}` is not an identifier of a document of this corpus"
                )
            }
            Reason::SelfEdge { relation, id } => write!(
                f,
                "it proposes `{relation}` from `{id}` to itself, and a document declares nothing \
                 about itself by naming itself"
            ),
            Reason::EndpointNotPermitted {
                relation,
                end,
                id,
                kind,
                permitted,
            } => write!(
                f,
                "`{relation}` admits `{}` at the `{end}` end, and `{id}` there has the kind \
                 `{kind}`",
                permitted.join(", ")
            ),
            Reason::AlreadyDeclared { from, relation, to } => write!(
                f,
                "`{from} {relation} {to}` is already declared, so this restates the graph"
            ),
        }
    }
}

/// One finding this engine refused, and where in the file it was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rejected {
    /// One-based, in the order the file lists them.
    pub at: usize,
    /// The line of the return file the finding starts on.
    pub line: usize,
    pub reason: Reason,
}

/// One quotation, and where this engine found it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Evidence {
    pub path: String,
    pub quote: String,
    /// One-based, in the document quoted.
    pub line: usize,
}

/// An edge the sweep proposes, verified as new.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposal {
    pub relation: String,
    pub from: String,
    pub to: String,
    /// The path of the document at the `from` end, which is the one that would
    /// carry the front matter.
    ///
    /// It is resolved from the graph rather than taken from the finding's
    /// `documents`. The two are not the same list: a finding may compare three
    /// documents and propose an edge between two of them, and the first path it
    /// names is then a document the proposed edge says nothing about.
    pub path: String,
    /// The half the document at the far end owes, when the relation requires
    /// both ends, and nothing when it does not.
    ///
    /// `None` for a symmetric relation and for one that declares no
    /// reciprocity, because there a second block would tell a person to declare
    /// an edge their taxonomy never asked for.
    pub owed: Option<Owed>,
}

/// The second half of a `reciprocal: required` pair, as front matter.
///
/// A sweep prints front matter for a person to paste, and
/// [`headwater_check::reciprocity`] reports a pair carrying one half as an
/// error. So a report that prints one block alone hands a reader an
/// instruction that their own commit gate refuses, and the reader has no way
/// back to the sweep from the error they get.
///
/// The name is direction-dependent, and it is computed here the way that rule
/// computes it: the inverse name when the printed half is the declared one, and
/// the declared name when the printed half is the inverse one. The two have to
/// agree, and a disagreement is silent — the report prints a block and the gate
/// rejects it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Owed {
    /// The path of the document at the far end, which is the one that owes it.
    ///
    /// It is the `to` end of the proposal as the finding wrote it, resolved
    /// through the graph, and never the second path the finding names.
    pub path: String,
    /// The relation name that half is written under.
    pub relation: String,
    /// The identifier it names, which is the proposal's `from`.
    pub id: String,
}

/// One finding that survived every test above.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verified {
    pub finding: Finding,
    pub class: Class,
    pub documents: Vec<String>,
    pub evidence: Vec<Evidence>,
    pub proposal: Option<Proposal>,
}

/// One sweep, read back.
#[derive(Clone, Debug)]
pub struct Report {
    /// The slice the file names, as written.
    pub slice: String,
    /// Findings the file held, before any test.
    pub read: usize,
    pub verified: Vec<Verified>,
    pub rejected: Vec<Rejected>,
    pub refusal: Option<Refusal>,
    /// Classified documents under the named slice, on this tree.
    pub extent: usize,
    /// Classified documents in the whole corpus, on this tree.
    pub corpus: usize,
}

impl Report {
    /// Read a return file over a tree.
    ///
    /// It never fails. A file this engine cannot use produces a report with a
    /// [`Refusal`] on it, which is a thing a reader sees rather than a status a
    /// caller branches on.
    pub fn read(source: &str, tree: &Tree<'_>) -> Report {
        let mut report = Report {
            slice: String::new(),
            read: 0,
            verified: Vec::new(),
            rejected: Vec::new(),
            refusal: None,
            extent: 0,
            corpus: 0,
        };

        let loaded = match headwater_yaml::load(source) {
            Ok(loaded) => loaded,
            Err(errors) => {
                report.refusal = Some(Refusal::Unparsed(
                    errors.iter().map(|error| error.to_string()).collect(),
                ));
                return report;
            }
        };
        let Value::Map(root) = &loaded.value else {
            report.refusal = Some(Refusal::Malformed(
                "the file is not a mapping, and the plan prints the shape it wants".into(),
            ));
            return report;
        };

        report.slice = text(root, "slice").unwrap_or_else(|| ".".into());
        let (extent, corpus) = counts(tree.census, &report.slice);
        report.extent = extent;
        report.corpus = corpus;

        match text(root, "taxonomy") {
            Some(claimed) if claimed == tree.lock => {}
            Some(claimed) => {
                report.refusal = Some(Refusal::TaxonomyMoved {
                    claimed,
                    tree: tree.lock.to_string(),
                });
                return report;
            }
            None => {
                report.refusal = Some(Refusal::Malformed(
                    "the file names no `taxonomy`, so nothing says which lock it was planned against"
                        .into(),
                ));
                return report;
            }
        }

        let Some(entry) = root.get("findings") else {
            report.refusal = Some(Refusal::Malformed(
                "the file carries no `findings` key".into(),
            ));
            return report;
        };
        let Some(items) = entry.value.as_seq() else {
            report.refusal = Some(Refusal::Malformed("`findings` is not a sequence".into()));
            return report;
        };

        report.read = items.len();
        let mut sources: BTreeMap<String, String> = BTreeMap::new();
        for (index, item) in items.iter().enumerate() {
            let at = index + 1;
            let line = item.span.start.line;
            match one(item, tree, &mut sources) {
                Ok(verified) => report.verified.push(verified),
                Err(reason) => report.rejected.push(Rejected { at, line, reason }),
            }
        }
        report
            .verified
            .sort_by(|a, b| a.finding.order().cmp(&b.finding.order()));
        report
    }
}

/// One finding of the file, tested.
fn one(
    item: &Spanned<Value>,
    tree: &Tree<'_>,
    sources: &mut BTreeMap<String, String>,
) -> Result<Verified, Reason> {
    let Some(map) = item.value.as_map() else {
        return Err(Reason::Missing("mapping of its own"));
    };

    let name = text(map, "class").ok_or(Reason::Missing("class"))?;
    let class = Class::read(&name).ok_or(Reason::UnknownClass(name))?;
    let message = text(map, "message").ok_or(Reason::Missing("message"))?;
    if message.trim().is_empty() {
        return Err(Reason::Missing("message"));
    }

    let documents = strings(map, "documents");
    if documents.is_empty() {
        return Err(Reason::Missing("documents"));
    }
    for path in &documents {
        if !classified(tree.census, path) {
            return Err(Reason::NotAMember(path.clone()));
        }
    }

    let quotations = quotations(map)?;
    if quotations.is_empty() {
        return Err(Reason::Missing("evidence"));
    }
    let mut evidence = Vec::new();
    for (path, quote) in quotations {
        if !documents.contains(&path) {
            return Err(Reason::EvidenceOffFinding(path));
        }
        let source = match sources.get(&path) {
            Some(source) => source,
            None => {
                let read = std::fs::read_to_string(tree.root.join(&path)).unwrap_or_default();
                sources.entry(path.clone()).or_insert(read)
            }
        };
        match locate(source, &quote) {
            Some(line) => evidence.push(Evidence { path, quote, line }),
            None => return Err(Reason::QuoteNotFound { path, quote }),
        }
    }

    let proposal = match map.get("proposal") {
        None => None,
        Some(entry) => {
            let Some(block) = entry.value.as_map() else {
                return Err(Reason::Missing("proposal this engine can read"));
            };
            let relation = text(block, "relation").ok_or(Reason::Missing("proposal relation"))?;
            let from = text(block, "from").ok_or(Reason::Missing("proposal source"))?;
            let to = text(block, "to").ok_or(Reason::Missing("proposal target"))?;
            let Some(named) = tree.relations.named(&relation) else {
                return Err(Reason::UnknownRelation(relation));
            };
            let Some(source) = tree.graph.index.node(&from) else {
                return Err(Reason::UnknownEndpoint(from));
            };
            let Some(target) = tree.graph.index.node(&to) else {
                return Err(Reason::UnknownEndpoint(to));
            };
            if from == to {
                return Err(Reason::SelfEdge { relation, id: from });
            }

            // Which end each identifier sits at, from the name the finding
            // wrote. A relation reached under its inverse name puts the
            // proposal's `from` at the declared `to` end, so the two endpoint
            // sets swap and the identifiers do not.
            // `crates/scaffold/src/lib.rs` reads the same `Direction` for the
            // same reason on the write path.
            let (near, far) = match named.direction {
                Direction::AsDeclared => (&named.relation.from, &named.relation.to),
                Direction::Inverse => (&named.relation.to, &named.relation.from),
            };
            // The judge is `Shape::descends_from`, so an endpoint that names an
            // abstract parent admits every kind under it. This is the same
            // question `headwater_check`'s `relation.endpoint` rule asks of a
            // written edge, and the answer has to agree: a proposal is front
            // matter, and the person told to write it runs that rule next.
            let admits = |permitted: &[String], kind: &str| {
                permitted
                    .iter()
                    .any(|allowed| tree.shape.descends_from(kind, allowed))
            };
            for (end, node, permitted) in [("from", source, near), ("to", target, far)] {
                let kind = node.kind.clone().unwrap_or_default();
                if !admits(permitted, &kind) {
                    return Err(Reason::EndpointNotPermitted {
                        relation: relation.clone(),
                        end,
                        id: node.id.clone(),
                        kind,
                        permitted: permitted.clone(),
                    });
                }
            }

            // Novelty, in both orders when the relation is its own inverse. A
            // symmetric relation declared one way round is the same edge read
            // the other way round, so `B conflicts_with A` in the front matter
            // makes a proposed `A conflicts_with B` a restatement.
            let mut orders: Vec<(&str, &str)> = vec![(&from, &to)];
            if named.relation.reciprocal == Reciprocal::Symmetric {
                orders.push((&to, &from));
            }
            for (one, other) in orders {
                if declares(tree.graph, one, &relation, other) {
                    return Err(Reason::AlreadyDeclared {
                        from: one.to_string(),
                        relation: relation.clone(),
                        to: other.to_string(),
                    });
                }
            }
            let path = source.path.clone();
            // The second half, when the relation requires one. See [`Owed`]:
            // the owed name is `relation.inverse` where the printed half is the
            // declared one and `relation.name` where the printed half is the
            // inverse one, which is what `headwater_check::reciprocity` reads.
            // A required relation that declares no inverse owes its own name,
            // the same fallback that rule takes.
            let owed = (named.relation.reciprocal == Reciprocal::Required).then(|| {
                let relation = match named.direction {
                    Direction::AsDeclared => named
                        .relation
                        .inverse
                        .clone()
                        .unwrap_or_else(|| named.relation.name.clone()),
                    Direction::Inverse => named.relation.name.clone(),
                };
                Owed {
                    path: target.path.clone(),
                    relation,
                    id: from.clone(),
                }
            });
            Some(Proposal {
                relation,
                from,
                to,
                path,
                owed,
            })
        }
    };

    // Spec 4's fourth constraint, applied to the class rather than to the
    // proposal. A finding may name two documents and propose nothing, and it
    // still restates the graph when the relation its class implies is already
    // declared between them.
    if let (Some(relation), [first, second]) = (class.implies(), documents.as_slice()) {
        for (from, to) in [(first, second), (second, first)] {
            if let Some((from, to)) = ids(tree.graph, from, to) {
                if declares(tree.graph, &from, relation, &to) {
                    return Err(Reason::AlreadyDeclared {
                        from,
                        relation: relation.to_string(),
                        to,
                    });
                }
            }
        }
    }

    let path = documents[0].clone();
    let line = evidence
        .iter()
        .find(|found| found.path == path)
        .map(|found| found.line)
        .unwrap_or(0);
    Ok(Verified {
        finding: Finding {
            rule: class.rule(),
            // Every sweep finding is `info`, and the value is not a judgment
            // about how much the defect matters. Spec 12 puts severity on the
            // check and posture on the control; a sampler has neither, so the
            // one honest value is the one that says "read this" and claims
            // nothing about blocking. Nothing reads it either way: no control
            // names a sweep class, so no posture attaches to one.
            severity: Severity::Info,
            // No control names a `sweep:` mechanism in this repository, so no
            // obligation is reachable from here. `None` is what the shape says
            // for that, rather than an identifier no register would recognize.
            obligation: None,
            path,
            line,
            column: match line {
                0 => 0,
                _ => 1,
            },
            message,
            remediation: class.remediation().to_string(),
            // Never. Every remedy of a coherence class is a rewrite or a
            // declaration, so the engine writes none of them.
            patch: None,
        },
        class,
        documents,
        evidence,
        proposal,
    })
}

/// Whether the graph already carries this edge, in the direction given.
fn declares(graph: &Graph, from: &str, relation: &str, to: &str) -> bool {
    graph.edges.iter().any(|edge| {
        let Target::Document { id, .. } = &edge.target else {
            return false;
        };
        edge.source.id == from && edge.declared == relation && id == to
    })
}

/// The identifiers of two documents named by path, when both carry one.
fn ids(graph: &Graph, from: &str, to: &str) -> Option<(String, String)> {
    let from = graph.index.by_path(from)?.id.clone()?;
    let to = graph.index.by_path(to)?.id.clone()?;
    Some((from, to))
}

/// Whether one path is a classified document of this corpus.
///
/// This is the refusal that catches a model inventing a path, and it is
/// stronger than a test that the file exists.
/// [Q24](../../../../docs/decisions/0024-q24-readability-and-what-a-sweep-can-be-asked-about.md)
/// asked whether a source file could be a slice member without becoming a
/// document, and refused it here rather than in the plan. Relaxing this to
/// `Path::exists` would let a model satisfy membership by naming any file in
/// the tree, and the cost would fall on the four classes that asked for
/// nothing. Two smaller costs ride with it: [`crate::plan::Plan`] reports its
/// extent as classified documents over classified documents, so a slice that
/// may hold unclassified files makes one ratio out of two populations, and
/// [`crate::plan::Member`] has no `Option` around its kind.
fn classified(census: &Census, path: &str) -> bool {
    census
        .rows
        .iter()
        .any(|row| row.path == path && matches!(row.outcome, Outcome::Typed { .. }))
}

/// Classified documents under the slice, and in the whole corpus.
fn counts(census: &Census, slice: &str) -> (usize, usize) {
    let under = slice.trim_end_matches('/');
    let mut extent = 0;
    let mut corpus = 0;
    for row in &census.rows {
        if !matches!(row.outcome, Outcome::Typed { .. }) {
            continue;
        }
        corpus += 1;
        if under.is_empty()
            || under == "."
            || row.path == under
            || row.path.starts_with(&format!("{under}/"))
        {
            extent += 1;
        }
    }
    (extent, corpus)
}

fn quotations(map: &Mapping) -> Result<Vec<(String, String)>, Reason> {
    let Some(entry) = map.get("evidence") else {
        return Ok(Vec::new());
    };
    let Some(items) = entry.value.as_seq() else {
        return Err(Reason::Missing("evidence this engine can read"));
    };
    let mut out = Vec::new();
    for item in items {
        let Some(block) = item.value.as_map() else {
            return Err(Reason::Missing("evidence entry with a path and a quote"));
        };
        let path = text(block, "path").ok_or(Reason::Missing("evidence path"))?;
        let quote = text(block, "quote").ok_or(Reason::Missing("evidence quote"))?;
        out.push((path, quote));
    }
    Ok(out)
}

fn text(map: &Mapping, key: &str) -> Option<String> {
    Some(map.get(key)?.value.as_scalar()?.text.clone())
}

fn strings(map: &Mapping, key: &str) -> Vec<String> {
    let Some(entry) = map.get(key) else {
        return Vec::new();
    };
    let Some(items) = entry.value.as_seq() else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| item.value.as_scalar().map(|scalar| scalar.text.clone()))
        .collect()
}

/// The line a quotation starts on, with every run of whitespace collapsed on
/// both sides. `None` when the document does not hold it.
pub fn locate(source: &str, quote: &str) -> Option<usize> {
    let needle = collapse(quote).0;
    let needle = needle.trim();
    if needle.is_empty() {
        return None;
    }
    let (haystack, lines) = collapse(source);
    let at = haystack.find(needle)?;
    Some(lines.get(at).copied().unwrap_or(0))
}

/// The text with every run of whitespace collapsed to one space, and the source
/// line each byte of the result came from.
fn collapse(source: &str) -> (String, Vec<usize>) {
    let mut text = String::with_capacity(source.len());
    let mut lines = Vec::with_capacity(source.len());
    let mut line = 1;
    let mut in_space = false;
    for character in source.chars() {
        if character.is_whitespace() {
            if !in_space && !text.is_empty() {
                text.push(' ');
                lines.push(line);
            }
            in_space = true;
            if character == '\n' {
                line += 1;
            }
            continue;
        }
        in_space = false;
        let start = text.len();
        text.push(character);
        for _ in start..text.len() {
            lines.push(line);
        }
    }
    // A trailing space belongs to nothing, and a needle never ends in one.
    while text.ends_with(' ') {
        text.pop();
        lines.pop();
    }
    (text, lines)
}

fn quoted(text: &str) -> String {
    let one_line: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    match one_line.chars().count() > 60 {
        true => format!("\"{}…\"", one_line.chars().take(60).collect::<String>()),
        false => format!("\"{one_line}\""),
    }
}

/// Every finding of a report, in the engine's own words.
///
/// This is the one place a sweep finding becomes bytes a person reads, and
/// every one of them is stamped [`PROVENANCE`]. See [`crate::json`] for the
/// same report in the finding shape spec 4 declares.
impl Report {
    /// `mode` is the color decision the caller already made — see
    /// `headwater_check::paint`'s module comment for why this function reads
    /// no stream itself.
    pub fn render(&self, mode: ColorMode) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(
            out,
            "A coherence sweep over {}, {} of the {} classified documents of this corpus.",
            self.slice, self.extent, self.corpus
        );
        let _ = writeln!(
            out,
            "Every finding below was written by an agent. Provenance: {PROVENANCE}. Nothing here \
             is a verdict, and no exit status reads it."
        );
        let _ = writeln!(out);

        if let Some(refusal) = &self.refusal {
            let _ = writeln!(out, "This file reported nothing: {refusal}");
            return out;
        }

        for verified in &self.verified {
            let _ = write!(out, "{}", verified.finding.render(mode));
            for evidence in &verified.evidence {
                let _ = writeln!(
                    out,
                    "  {} {} at {}:{}",
                    dim("quoted:", mode),
                    quoted(&evidence.quote),
                    paint(Role::Path, &evidence.path, mode),
                    evidence.line
                );
            }
            if verified.documents.len() > 1 {
                let _ = writeln!(
                    out,
                    "  {} {}",
                    dim("compared:", mode),
                    verified.documents.join(", ")
                );
            }
            if let Some(proposal) = &verified.proposal {
                let _ = writeln!(
                    out,
                    "  {} {} {} {}, which the graph does not declare today",
                    dim("proposes:", mode),
                    proposal.from,
                    proposal.relation,
                    proposal.to
                );
                let _ = writeln!(
                    out,
                    "  {} {}:",
                    dim("to declare it, in the front matter of", mode),
                    paint(Role::Path, &proposal.path, mode)
                );
                let _ = writeln!(out, "      relations:");
                let _ = writeln!(out, "        {}:", proposal.relation);
                let _ = writeln!(out, "          - {}", proposal.to);
                // The other half, for a relation that requires both ends. A
                // reader who pastes the block above and stops has a corpus that
                // `relation.reciprocity.missing` refuses, so the instruction is
                // incomplete rather than wrong. See [`Owed`].
                if let Some(owed) = &proposal.owed {
                    let _ = writeln!(
                        out,
                        "  {} `{}` {}, in the front matter of {}:",
                        dim("and, because", mode),
                        proposal.relation,
                        dim("requires both ends", mode),
                        paint(Role::Path, &owed.path, mode)
                    );
                    let _ = writeln!(out, "      relations:");
                    let _ = writeln!(out, "        {}:", owed.relation);
                    let _ = writeln!(out, "          - {}", owed.id);
                }
            }
            let _ = writeln!(out);
        }

        if !self.rejected.is_empty() {
            let _ = writeln!(
                out,
                "{} of {} findings did not reach you, and each one is a defect in the sweep rather \
                 than a fact about the corpus:",
                self.rejected.len(),
                self.read
            );
            for rejected in &self.rejected {
                let _ = writeln!(
                    out,
                    "  finding {} (line {}): {}",
                    rejected.at, rejected.line, rejected.reason
                );
            }
            let _ = writeln!(out);
        }

        let _ = writeln!(
            out,
            "{} carried, {} refused, of {} the file held. The engine confirmed the citation and \
             the novelty of every carried finding, and it confirmed nothing about whether the \
             claim is true.",
            self.verified.len(),
            self.rejected.len(),
            self.read
        );
        let proposals = self
            .verified
            .iter()
            .filter(|verified| verified.proposal.is_some())
            .count();
        let _ = writeln!(
            out,
            "{} an edge, which is the outcome spec 4 asks a sweep for. Nothing here writes one: a \
             person accepts a proposal, and the engine never accepts its own.",
            match proposals {
                1 => "One carried finding proposes".to_string(),
                other => format!("{other} carried findings propose"),
            }
        );
        out
    }
}

#[cfg(test)]
mod render_tests {
    use super::{Evidence, Report, Verified};
    use crate::Class;
    use headwater_check::paint::ColorMode;
    use headwater_check::{Finding, Severity};

    fn report() -> Report {
        Report {
            slice: ".".to_string(),
            read: 1,
            verified: vec![Verified {
                finding: Finding {
                    rule: "sweep.undeclared_conflict",
                    severity: Severity::Info,
                    obligation: None,
                    path: "docs/spec/00-a.md".to_string(),
                    line: 3,
                    column: 1,
                    message: "one document contradicts another".to_string(),
                    remediation: "read both passages".to_string(),
                    patch: None,
                },
                class: Class::UndeclaredConflict,
                documents: vec!["docs/spec/00-a.md".to_string()],
                evidence: vec![Evidence {
                    path: "docs/spec/00-a.md".to_string(),
                    quote: "a quoted passage".to_string(),
                    line: 3,
                }],
                proposal: None,
            }],
            rejected: Vec::new(),
            refusal: None,
            extent: 1,
            corpus: 1,
        }
    }

    /// `Plain` writes no escape sequence anywhere in the report — the finding
    /// it carries included, on the rule `Finding::render`'s own test states.
    /// `Ansi` colors the finding and the evidence path, and dims the
    /// `quoted:` label, and neither rewrites a word.
    #[test]
    fn a_finding_and_its_evidence_carry_color_only_under_ansi() {
        let report = report();

        let plain = report.render(ColorMode::Plain);
        assert!(!plain.contains('\x1b'), "{plain:?}");
        assert!(plain.contains("· info"), "{plain:?}");
        assert!(plain.contains("quoted:"), "{plain:?}");

        let ansi = report.render(ColorMode::Ansi);
        assert!(ansi.contains('\x1b'), "{ansi:?}");
        for word in ["docs/spec/00-a.md", "info", "quoted:", "a quoted passage"] {
            assert!(ansi.contains(word), "{ansi:?} is missing {word:?}");
        }
    }
}
