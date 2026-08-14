// SPDX-License-Identifier: Apache-2.0
//! What comes back, and the four things this engine can confirm about it.
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
use headwater_check::{Finding, Severity};
use headwater_graph::declarations::Declarations;
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
            if tree.relations.named(&relation).is_none() {
                return Err(Reason::UnknownRelation(relation));
            }
            for id in [&from, &to] {
                if tree.graph.index.node(id).is_none() {
                    return Err(Reason::UnknownEndpoint(id.clone()));
                }
            }
            if declares(tree.graph, &from, &relation, &to) {
                return Err(Reason::AlreadyDeclared { from, relation, to });
            }
            Some(Proposal { relation, from, to })
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
    pub fn render(&self) -> String {
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
            let _ = write!(out, "{}", verified.finding.render());
            for evidence in &verified.evidence {
                let _ = writeln!(
                    out,
                    "  quoted: {} at {}:{}",
                    quoted(&evidence.quote),
                    evidence.path,
                    evidence.line
                );
            }
            if verified.documents.len() > 1 {
                let _ = writeln!(out, "  compared: {}", verified.documents.join(", "));
            }
            if let Some(proposal) = &verified.proposal {
                let _ = writeln!(
                    out,
                    "  proposes: {} {} {}, which the graph does not declare today",
                    proposal.from, proposal.relation, proposal.to
                );
                let _ = writeln!(
                    out,
                    "  to declare it, in the front matter of {}:",
                    verified.documents[0]
                );
                let _ = writeln!(out, "      relations:");
                let _ = writeln!(out, "        {}:", proposal.relation);
                let _ = writeln!(out, "          - {}", proposal.to);
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
            "{proposals} of them propose an edge, which is the outcome spec 4 asks a sweep for. \
             Nothing here writes one: a person accepts a proposal, and the engine never accepts \
             its own."
        );
        out
    }
}
