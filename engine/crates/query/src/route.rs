// SPDX-License-Identifier: Apache-2.0
//! Intent-time routing: a task description to a ranked set of pointers.
//!
//! [Spec 5](../../../../docs/spec/05-ai-integration.md#intent-time-routing):
//! "Routing matches the **declared purpose** of each kind against the intent of
//! the task before it matches any text… Lexical ranking then orders results
//! *within* the matched purpose, and derived reading precedence breaks ties."
//! Those three steps are the three sections of this file, in that order.
//!
//! # Why a purpose is matched on the terms that separate it
//!
//! Spec 5 grades a cue "against the alternatives that it competes with at the
//! moment a reader reads it", and that rule is the whole of the weighting here.
//! A task term that appears in the declaration of *every* purpose separates
//! none of them, so it scores nothing. What is left is the terms that pick one
//! purpose out of the set, which is what a reader is doing when they ask.
//!
//! One corpus is the exception, and it is the corpus that declares a single
//! purpose. There is no alternative to separate it from, so every term counts.
//! A rule that scored nothing there would make a one-purpose taxonomy route to
//! silence for every task, which is a report about this function rather than
//! about the corpus.
//!
//! # Why the gate is a term that reaches the document
//!
//! Spec 5: "Routing is **confidence-gated and fails open**: below the threshold
//! it says nothing. A wrong pointer costs more than a missing one, because an
//! agent will follow it." The threshold here is stated rather than tuned: a
//! pointer is offered only where a term of the task reaches the document as
//! well as its purpose. A task that matched `rationale` and nothing else would
//! otherwise offer every decision in the corpus, ordered by a tie-break, and an
//! agent would open the first.
//!
//! # What a route reads, and what it does not
//!
//! Spec 5 fixes the read set: "a deterministic projection over the graph —
//! summaries, facets, relations, and code-path anchors — not a semantic
//! search." So the lexical step reads the summary, the facet values, the path,
//! and the anchors this document's edges reach. It never opens the body. A
//! route that ranked on prose would be a search engine with a taxonomy beside
//! it, and the ranking would move on every paragraph anybody edited.

use crate::{terms, Document, Pointer, Surface};
use headwater_graph::declarations::Governs;
use headwater_graph::Target;

/// How much of a route a ranking may fill, which spec 5 calls a budget.
///
/// It bounds the ranked offer and nothing else. A document that declares
/// `governs` over a path the task named was named rather than ranked, so it is
/// carried whatever this says, and [`Route::withheld`] reports what the bound
/// removed.
///
/// The default is five. The number is a stated choice rather than a measured
/// one: a pointer list is read by an agent that then opens documents, and a
/// list longer than a handful is a context cost with a falling return. An
/// adopter who wants another number passes one.
#[derive(Clone, Copy, Debug)]
pub struct Budget {
    /// How many ranked pointers a route may offer. A route may return more
    /// pointers than this, and every one of them was named by an anchor.
    pub pointers: usize,
}

impl Default for Budget {
    fn default() -> Self {
        Budget { pointers: 5 }
    }
}

/// One purpose the task matched, and what it scored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Matched {
    pub purpose: String,
    pub score: u32,
}

/// Why a route offered nothing. Spec 5 keeps the reasons apart because they are
/// different facts about the corpus, and a caller that saw one silence for all
/// of them could not tell a weak cue from an undeclared taxonomy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Silence {
    /// The taxonomy declares no purposes, so there is no intentional structure
    /// to search. Nothing about the task is weak.
    NoPurposes,
    /// The task carries no term this function can match.
    NoTerms,
    /// No declared purpose answers this task.
    NoPurposeMatched,
    /// A purpose matched, and no document under it was reached by a term of the
    /// task. This is the confidence gate.
    NoDocumentReached,
}

impl Silence {
    /// The name a caller branches on.
    ///
    /// It is deliberately not [`Silence::name`]. That is a sentence written for
    /// a person and it is rewritten whenever the wording improves; this is a
    /// token, and a consumer of `headwater route --json` that switched on it
    /// keeps working across such a rewrite. Spec 5 keeps the four reasons apart
    /// because they are different facts about the corpus, and a machine reader
    /// that had to match on prose could not tell them apart at all.
    pub fn token(&self) -> &'static str {
        match self {
            Silence::NoPurposes => "no_purposes",
            Silence::NoTerms => "no_terms",
            Silence::NoPurposeMatched => "no_purpose_matched",
            Silence::NoDocumentReached => "no_document_reached",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Silence::NoPurposes => "the taxonomy declares no purposes, so a task matches nothing",
            Silence::NoTerms => "the task carries no term of two characters or more",
            Silence::NoPurposeMatched => "no declared purpose answers this task",
            Silence::NoDocumentReached => {
                "a purpose matched and no document under it was reached, so the route is silent"
            }
        }
    }
}

/// A route: what was asked, what it matched, and what it offers.
#[derive(Clone, Debug)]
pub struct Route {
    pub task: String,
    /// The terms of the task, in the order they were written.
    pub terms: Vec<String>,
    /// The terms that separate one declared purpose from the others. A term
    /// outside this set matched every purpose or none, and it scored nothing.
    pub separating: Vec<String>,
    /// The terms that separate one document from the others, which is the same
    /// rule at the second surface. A term that most of the corpus carries is
    /// not one of them.
    pub distinctive: Vec<String>,
    /// The anchors the task named outright, in the order it named them.
    pub anchors: Vec<String>,
    /// The purposes that matched, highest first.
    pub matched: Vec<Matched>,
    /// The pointers, in the order a reader should take them.
    pub pointers: Vec<Pointer>,
    /// How many ranked pointers the budget removed from this route.
    ///
    /// This is the answer to "were there more". A silent cut destroyed the
    /// difference between "there were three" and "there were fifteen and you
    /// were shown three", and a reader who cannot tell them apart stops
    /// looking. Spec 5 already requires a document removed by an export filter
    /// to be reported, because nothing about the removal is uncertain, and a
    /// removal for cost is the same kind of fact.
    ///
    /// It counts ranked candidates only. A pointer an anchor named is never
    /// removed, so it is never counted here.
    pub withheld: usize,
    /// Why the pointer list is empty, and `None` where it is not.
    pub silence: Option<Silence>,
}

/// One candidate under a matched purpose, before the budget cuts the list.
struct Candidate {
    pointer: Pointer,
    purpose: u32,
    lexical: u32,
}

/// The four surfaces spec 5 lets a route read, for one document.
///
/// Held apart rather than joined, because each one weighs differently: a term
/// in the cue an author wrote for this document is worth more than the same
/// term in its path.
struct Surfaces {
    summary: String,
    anchors: String,
    facets: String,
    path: String,
}

impl Surfaces {
    /// Whether any surface holds the term. This is the reading the corpus-wide
    /// frequency count uses, because a term is distinctive or not about the
    /// document as a whole.
    fn holds(&self, term: &str) -> bool {
        holds(&self.summary, term)
            || holds(&self.anchors, term)
            || holds(&self.facets, term)
            || holds(&self.path, term)
    }

    /// The score over weighted terms, and whether a separating term reached
    /// this document at all.
    ///
    /// Two answers rather than one, because the two decide different things.
    /// The score orders a list. The second decides membership, and spec 5 puts
    /// the cost there: "a wrong pointer costs more than a missing one". A
    /// document that the task reached only through a term most of the corpus
    /// carries was not distinguished from anything, and offering it is a guess
    /// dressed as an answer.
    fn score(&self, weighted: &[Weighted]) -> (u32, bool) {
        let mut score = 0;
        let mut separated = false;
        for term in weighted {
            let mut surface = 0;
            if holds(&self.summary, &term.term) {
                surface += 3;
            }
            if holds(&self.anchors, &term.term) {
                surface += 2;
            }
            if holds(&self.facets, &term.term) {
                surface += 1;
            }
            if holds(&self.path, &term.term) {
                surface += 1;
            }
            score += surface * term.weight;
            separated |= surface > 0 && term.separating;
        }
        (score, separated)
    }
}

impl Surface<'_> {
    /// Route a task description to pointers.
    pub fn route(&self, task: &str, budget: Budget) -> Route {
        let terms = terms(task);
        let mut route = Route {
            task: task.to_string(),
            terms: terms.clone(),
            separating: Vec::new(),
            distinctive: Vec::new(),
            anchors: Vec::new(),
            matched: Vec::new(),
            pointers: Vec::new(),
            withheld: 0,
            silence: None,
        };

        // Step 0. An anchor the task named outright. This is an identity and
        // not a guess: the task holds a path, a resolver already normalized
        // that path into a node of the graph, and the documents that govern it
        // are the answer. It runs before the purposes because no lexical score
        // competes with an identity, and it is the one step that answers a task
        // in which no purpose has any scent at all.
        // The set is the union over every anchor the task named, and it is a
        // set: `governing_docs_for_path` deduplicates inside one anchor and
        // cannot see across them, so a document that governs two of the named
        // paths would otherwise be offered twice.
        route.anchors = self.named_anchors(task);
        let mut anchored: Vec<Pointer> = Vec::new();
        for anchor in &route.anchors {
            for pointer in self.governing_docs_for_path(anchor) {
                if !anchored.contains(&pointer) {
                    anchored.push(pointer);
                }
            }
        }

        // The budget never reaches this set, on any of the three branches
        // below. An anchor is an identity and the budget caps the ranking, so
        // removing a document nobody ranked has no declared basis — which is
        // the rule spec 5 applies to its other budget, where a bound engine
        // drops satellites before nuclei because nuclearity is declared.
        // `governing_docs_for_path` is a published tool that takes no budget,
        // and this is the step that answers the same question, so the two
        // surfaces have to agree.

        if self.shape().purposes.is_empty() {
            return route.on_anchors_alone(anchored, Silence::NoPurposes);
        }
        if terms.is_empty() {
            route.silence = Some(Silence::NoTerms);
            return route;
        }

        // Step 1. The purposes, on the terms that separate them.
        route.separating = terms
            .iter()
            .filter(|term| self.separates(term))
            .cloned()
            .collect();
        for purpose in &self.shape().purposes {
            let mut score = 0;
            for term in &route.separating {
                // `answers` carries the questions the purpose answers, which is
                // the nearest thing a taxonomy holds to a task description, so
                // it outweighs the one-sentence intent beside it.
                if holds(&purpose.answers.join(" "), term) {
                    score += 3;
                } else if holds(purpose.intent.as_deref().unwrap_or_default(), term) {
                    score += 1;
                }
            }
            if score > 0 {
                route.matched.push(Matched {
                    purpose: purpose.name.clone(),
                    score,
                });
            }
        }
        if route.matched.is_empty() {
            return route.on_anchors_alone(anchored, Silence::NoPurposeMatched);
        }
        route
            .matched
            .sort_by(|a, b| b.score.cmp(&a.score).then(a.purpose.cmp(&b.purpose)));

        // Step 2. The documents under those purposes, ordered lexically within
        // each. The gate is here: a document that no distinctive term of the
        // task reached is not offered.
        let surfaces: Vec<(crate::Document<'_>, Surfaces)> = self
            .documents()
            .into_iter()
            .map(|document| {
                let surfaces = self.surfaces(&document);
                (document, surfaces)
            })
            .collect();
        let weighted = weights(&terms, &surfaces);
        route.distinctive = weighted
            .iter()
            .filter(|term| term.separating)
            .map(|term| term.term.clone())
            .collect();

        let mut candidates: Vec<Candidate> = Vec::new();
        for (document, document_surfaces) in &surfaces {
            let Some(purpose) = self.shape().purpose_of(document.kind) else {
                continue;
            };
            let Some(matched) = route
                .matched
                .iter()
                .find(|matched| matched.purpose == purpose.name)
            else {
                continue;
            };
            let (lexical, separated) = document_surfaces.score(&weighted);
            if !separated {
                continue;
            }
            let pointer = self.pointer(document);
            if anchored.contains(&pointer) {
                continue;
            }
            candidates.push(Candidate {
                pointer,
                purpose: matched.score,
                lexical,
            });
        }
        if candidates.is_empty() && anchored.is_empty() {
            route.silence = Some(Silence::NoDocumentReached);
            return route;
        }

        // The order is total: two candidates that tie on both scores are
        // separated by their paths, which the census holds one of each.
        candidates.sort_by(|a, b| {
            b.purpose
                .cmp(&a.purpose)
                .then(b.lexical.cmp(&a.lexical))
                .then(a.pointer.path.cmp(&b.pointer.path))
        });
        // The one place a budget cuts anything. It cuts `Candidate`, which is
        // the private type a ranked guess arrives in, and an anchored pointer
        // is never built into one — so no anchor can be removed here whatever
        // the budget says.
        //
        // The count is taken before the cut, because the cut is what destroys
        // the length that answers "were there more".
        let ranked = budget.pointers.saturating_sub(anchored.len());
        route.withheld = candidates.len().saturating_sub(ranked);
        candidates.truncate(ranked);

        // Step 3. Derived reading precedence, over the list the budget left.
        // Spec 5: "Where two linked documents both match, routing offers a
        // nucleus before its satellite." It runs after the cut rather than
        // before it, because precedence orders a list and does not decide
        // membership: a satellite that outscored its nucleus is still one of
        // the answers, and the nucleus is offered first among those that are.
        //
        // The two blocks are ordered apart. Precedence is an order over
        // documents that both match, and an anchor hit did not match — it was
        // named. To run one pass over the joined list would let a lexical guess
        // displace an identity, which is the trade spec 5 makes in the other
        // direction.
        let mut offered: Vec<Pointer> = candidates
            .into_iter()
            .map(|candidate| candidate.pointer)
            .collect();
        self.by_precedence(&mut anchored);
        self.by_precedence(&mut offered);
        route.pointers = anchored;
        route.pointers.extend(offered);
        route
    }

    /// The anchors of the graph that the task named outright.
    ///
    /// A word of the task is normalized by the same function the resolver used,
    /// and a hit is an exact match against a node the graph already holds. That
    /// is why no lexical score competes with it: nothing was inferred.
    fn named_anchors(&self, task: &str) -> Vec<String> {
        let mut found: Vec<String> = Vec::new();
        let nodes = self.graph.anchor_nodes();
        for word in task.split_whitespace() {
            let word = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '.');
            let Ok(normalized) = headwater_graph::anchors::normalize(word) else {
                continue;
            };
            if nodes.iter().any(|node| node.normalized == normalized)
                && !found.contains(&normalized)
            {
                found.push(normalized);
            }
        }
        found
    }

    /// The four surfaces of one document, read once.
    fn surfaces(&self, document: &crate::Document<'_>) -> Surfaces {
        Surfaces {
            summary: self.summary(document).unwrap_or_default(),
            anchors: self.anchor_text(document),
            facets: facet_text(document),
            path: document.path.to_string(),
        }
    }

    /// Whether a term separates one declared purpose from the others.
    ///
    /// A term that no purpose holds separates nothing either, and it is left in
    /// the set: it scores zero against every purpose by arithmetic, and to
    /// filter it here would make the reported set of separating terms a
    /// different thing from the set the scoring used.
    fn separates(&self, term: &str) -> bool {
        let purposes = &self.shape().purposes;
        if purposes.len() < 2 {
            return true;
        }
        !purposes.iter().all(|purpose| {
            holds(&purpose.answers.join(" "), term)
                || holds(purpose.intent.as_deref().unwrap_or_default(), term)
        })
    }

    /// Every anchor this document's edges reach, as one string.
    ///
    /// This is the surface that makes "add rate limiting to the ingest API"
    /// reach the standard that governs `src/ingest/`, which is the example spec
    /// 5 opens with. It is a read of the graph rather than of the document.
    fn anchor_text(&self, document: &Document<'_>) -> String {
        let mut text = String::new();
        for edge in &self.graph.edges {
            if edge.source.path != document.path {
                continue;
            }
            if let Target::Anchor { normalized, .. } = &edge.target {
                text.push_str(normalized);
                text.push(' ');
            }
        }
        text
    }

    /// Order a matched list by derived reading precedence.
    ///
    /// A governing document is moved in front of the one it governs, and the
    /// pass repeats until nothing moves. Adjacent pairs are not enough: a
    /// nucleus can sort four places below its satellite, and a pass that only
    /// swapped neighbours would leave it there. The fixture tree caught that
    /// too.
    ///
    /// The repeat count is bounded by the list length. Precedence over a corpus
    /// is not guaranteed acyclic, and a cycle in it must reorder a list rather
    /// than hang a query an agent is waiting on.
    ///
    /// Public because a shelf index is the third consumer spec 2 names for this
    /// derivation, after a routing result and the document a conflict is
    /// reported against. `headwater generate` calls it. A generator that sorted
    /// its entries by any other rule would disagree with what `route` and
    /// `explain` already tell a reader about the same two documents.
    pub fn by_precedence(&self, pointers: &mut Vec<Pointer>) {
        for _ in 0..pointers.len() {
            let mut moved = false;
            'pass: for i in 0..pointers.len() {
                for j in (i + 1)..pointers.len() {
                    if self.governs_pair(&pointers[j], &pointers[i]) {
                        let governing = pointers.remove(j);
                        pointers.insert(i, governing);
                        moved = true;
                        break 'pass;
                    }
                }
            }
            if !moved {
                return;
            }
        }
    }

    /// Whether an edge between the two makes the first govern the reading.
    fn governs_pair(&self, first: &Pointer, second: &Pointer) -> bool {
        self.graph.edges.iter().any(|edge| {
            let Target::Document { path, .. } = &edge.target else {
                return false;
            };
            let (source, target) = (&edge.source.path, path);
            match self.governs_of(edge) {
                Governs::Source => source == &first.path && target == &second.path,
                Governs::Target => target == &first.path && source == &second.path,
                Governs::Neither => false,
            }
        })
    }
}

/// The task terms that separate one document of the corpus from the others.
///
/// The same rule the purposes are matched on, at the surface below them. Spec
/// 5: "Grade a cue against the alternatives that it competes with at the moment
/// a reader reads it", and its own measure of a summary is distinctiveness
/// against the siblings — "a summary that shares no discriminating term with
/// its siblings cannot separate them". A term that most of the corpus carries
/// is therefore worth nothing here, whatever it means in English. This is what
/// keeps `the` out of a ranking, and it does it without the engine holding a
/// word list for one language.
///
/// The weight is how many documents the term rules *out*: a corpus of thirty
/// where one document carries the term weighs it twenty-nine, and a term every
/// document carries weighs nothing. That is a gradient rather than a threshold,
/// and the difference matters — a threshold discards the only signal a task
/// has as soon as the corpus grows past it, and nothing in the report says why
/// the answer changed. A corpus of one document has no alternatives to compare
/// against, so every term weighs one.
fn weights(terms: &[String], corpus: &[(Document<'_>, Surfaces)]) -> Vec<Weighted> {
    terms
        .iter()
        .map(|term| {
            if corpus.len() < 2 {
                return Weighted {
                    term: term.clone(),
                    weight: 1,
                    separating: true,
                };
            }
            let carried = corpus
                .iter()
                .filter(|(_, surfaces)| surfaces.holds(term))
                .count();
            Weighted {
                term: term.clone(),
                weight: (corpus.len() - carried) as u32,
                separating: carried * 2 <= corpus.len(),
            }
        })
        .collect()
}

/// One task term, against the corpus it is asked of.
struct Weighted {
    term: String,
    /// How many documents it rules out, which is what orders a list.
    weight: u32,
    /// Whether at most half the corpus carries it, which is what decides
    /// whether it may be the only reason to offer a document.
    separating: bool,
}

/// Whether a text holds a term, as one of its terms.
///
/// A substring match would make `api` reach `rapid`, and a route that offered a
/// document for that reason is the wrong pointer spec 5 says costs more than a
/// missing one.
fn holds(text: &str, term: &str) -> bool {
    terms(text).iter().any(|word| word == term)
}

/// The scalar facet values of a document, as one string.
///
/// Values and not keys. A key is the taxonomy's word and it is the same on
/// every document of the kind, so it separates nothing.
fn facet_text(document: &Document<'_>) -> String {
    let mut text = String::new();
    for entry in document.facets {
        if let Some(scalar) = entry.value.value.as_scalar() {
            text.push_str(&scalar.text);
            text.push(' ');
        }
    }
    text
}

impl Route {
    /// Finish a route that has no ranking to do, on the pointers its anchors
    /// named.
    ///
    /// Two branches arrive here: a corpus whose taxonomy declares no purposes,
    /// which is the corpus of an adopter on the first day, and a task that
    /// matched none of the purposes a taxonomy does declare. A bare file path
    /// is such a task, so this is the branch every impact-detection call takes.
    ///
    /// They are one function because they were two copies of the same four
    /// lines, and each copy cut the anchored set to the budget while the third
    /// branch of `route` carried it whole. One copy cannot disagree with
    /// itself. This one takes no budget, so no edit here can reintroduce the
    /// cut without changing the signature.
    fn on_anchors_alone(mut self, anchored: Vec<Pointer>, silence: Silence) -> Route {
        self.pointers = anchored;
        if self.pointers.is_empty() {
            self.silence = Some(silence);
        }
        self
    }

    /// The route as text: what it matched, and what it offers.
    ///
    /// A silent route prints why it is silent. Spec 5 makes silence a result,
    /// and a caller that could not tell "no purpose answers this" from "the
    /// corpus declares none" would debug the wrong file.
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(out, "route {:?}", self.task);
        let _ = writeln!(out, "  terms {}", self.terms.join(" "));
        if !self.distinctive.is_empty() {
            let _ = writeln!(out, "  distinctive {}", self.distinctive.join(" "));
        }
        for anchor in &self.anchors {
            let _ = writeln!(out, "  names the anchor {anchor}");
        }
        match self.matched.is_empty() {
            true => out.push_str("  no purpose matched\n"),
            false => {
                for matched in &self.matched {
                    let _ = writeln!(out, "  purpose {} {}", matched.purpose, matched.score);
                }
            }
        }
        if let Some(silence) = &self.silence {
            let _ = writeln!(out, "  {}", silence.name());
            return out;
        }
        for pointer in &self.pointers {
            out.push_str(&headwater_check::filled(&pointer.render(), 2));
        }
        // Printed only where the budget removed something, so a route that cut
        // nothing renders exactly as it did before. The line carries no em dash,
        // because `.claude/hooks/write.sh` selects pointer lines with a grep for
        // one and would show this count to an author as though it were a
        // document.
        if self.withheld > 0 {
            let more = match self.withheld {
                1 => "pointer",
                _ => "pointers",
            };
            let _ = writeln!(out, "  the budget withheld {} more {more}", self.withheld);
        }
        out
    }
}
