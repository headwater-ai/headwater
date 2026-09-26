// SPDX-License-Identifier: Apache-2.0
//! Intent-time routing: a task description to a ranked set of pointers.
//!
//! [Spec 5](../../../../docs/spec/05-ai-integration.md#intent-time-routing):
//! "Routing matches the **declared purpose** of each kind against the intent of
//! the task before it matches any text… Lexical ranking then orders results
//! *within* the matched purpose, and derived reading precedence breaks ties."
//! Those three steps are the three sections of this file, in that order.
//!
//! # Why the matched purposes take turns at the budget
//!
//! [HW-DR-0070](../../../../docs/decisions/0070-the-matched-purposes-take-turns-at-a-route-budget-and-each-pointer-states-what-reached-it.md)
//! amends the second step. A total order on the purpose score let the purpose
//! that scored highest take every slot while it had candidates. On this
//! repository's own corpus that put the answering specification part past
//! 100th, behind a hundred obligation records. So the order within a purpose
//! is the same as before, and the budget is filled in turns: each matched
//! purpose offers its best remaining candidate, highest purpose score first,
//! and inside one purpose the kinds that serve it take turns the same way. The
//! first pointer is still the best candidate of the best purpose.
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
use headwater_check::paint::{dim, paint, ColorMode, Role};
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
    /// Why each pointer was offered, one entry for each pointer and in the same
    /// order. [`Route::offers`] reads the two together.
    pub evidence: Vec<Evidence>,
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
    /// The paths of the task that the governed scope admits and that no
    /// document governs, in task order (#953). Empty where the task names none.
    pub ungoverned: Vec<Ungoverned>,
}

/// A path of the task that the governed scope admits and that nothing governs.
///
/// The write-time hook asks the route one question per edit, and before #953
/// this case answered with silence: a path the taxonomy expects a `governs`
/// edge to reach, and no edge reaches it. The route states the fact and the
/// front-matter lines that would declare the edge, and it writes nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ungoverned {
    /// The path, normalized as the resolver normalizes an anchor.
    pub path: String,
    /// The relations that govern and whose target admits the anchor kind of a
    /// scope pattern that admits the path, in declaration order. They are read
    /// off the taxonomy and never named here, so an adopter's own governance
    /// relation appears with no change to this crate.
    pub relations: Vec<String>,
}

/// Why a route offered one pointer.
///
/// It states what the route read, and never how likely the pointer is to be
/// right. The scores behind a rank are counts of term overlap and not
/// probabilities, so no member here is a score, and a member named for a
/// confidence would claim a calibration that does not exist
/// ([HW-DR-0070](../../../../docs/decisions/0070-the-matched-purposes-take-turns-at-a-route-budget-and-each-pointer-states-what-reached-it.md)).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Evidence {
    /// The task named a path this document governs. Nothing was ranked.
    Named {
        /// The anchors of the task that this document governs, in task order.
        anchors: Vec<String>,
    },
    /// A distinctive term of the task reached the document under a matched
    /// purpose.
    Ranked {
        /// The distinctive terms that reached the document, in task order.
        terms: Vec<String>,
        /// The place of the document in the order by purpose score and then
        /// term score, counted from 1, before the purposes took turns.
        rank: usize,
        /// How many documents passed the gate, which is the length of that
        /// order.
        of: usize,
    },
}

impl Evidence {
    /// The evidence as the line under its pointer in the report.
    pub fn render(&self) -> String {
        match self {
            Evidence::Named { anchors } => format!("governs {}", anchors.join(" ")),
            Evidence::Ranked { terms, rank, of } => {
                format!("matched {}, rank {rank} of {of}", terms.join(" "))
            }
        }
    }
}

/// One candidate under a matched purpose, before the budget cuts the list.
struct Candidate {
    pointer: Pointer,
    /// The matched purpose it was offered under, which the turns rotate over.
    under: String,
    purpose: u32,
    lexical: u32,
    /// The distinctive terms that reached it.
    terms: Vec<String>,
}

/// What one document scored against the weighted terms of a task.
struct Scored {
    score: u32,
    /// Whether a separating term reached any surface.
    separated: bool,
    /// The separating terms that reached any surface, in task order.
    terms: Vec<String>,
}

/// What the tree holds at one path, as the verb that runs a route sees it.
///
/// This crate reads no file. The verb answers from the tree it loaded, and a
/// caller that holds no tree, such as the read tools of the MCP server,
/// answers [`Entry::Absent`] for every path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    File,
    Directory,
    Absent,
}

/// The readings of one word of a task as a path, longest first, each already
/// normalized as the resolver normalizes an anchor.
///
/// A task names a path the way a person writes one: at the end of a sentence,
/// in parentheses, as a possessive, or with a line number after a colon. So
/// `tools/hw-cargo.`, `tools/hw-cargo's` and `tools/hw-cargo:12` all name
/// `tools/hw-cargo`. The first reading is the word as written, with only the
/// brackets, quotes and punctuation around it taken off. Each later reading
/// takes off one more thing: trailing dots, a possessive, everything from the
/// first colon, and trailing dots again. [`Surface::read_path`] decides which reading the
/// word names, and the word as written is always tried first.
///
/// A word that holds `://` is a URL and has no reading.
fn readings(word: &str) -> Vec<String> {
    let written = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '.');
    if written.contains("://") {
        return Vec::new();
    }
    fn possessive(w: &str) -> &str {
        w.strip_suffix("'s")
            .or_else(|| w.strip_suffix("\u{2019}s"))
            .unwrap_or(w)
    }
    fn line(w: &str) -> &str {
        w.split(':').next().unwrap_or(w)
    }
    fn dots(w: &str) -> &str {
        w.trim_end_matches('.')
    }
    // Dots twice: `tools/hw-cargo's.` ends a sentence with a possessive, and
    // `tools/hw-cargo:12.` ends one with a line number.
    let strips: [fn(&str) -> &str; 4] = [dots, possessive, line, dots];
    let mut shapes: Vec<&str> = vec![written];
    let mut last = written;
    for strip in strips {
        last = strip(last);
        shapes.push(last);
    }
    let mut found: Vec<String> = Vec::new();
    for shape in shapes {
        if shape.is_empty() {
            continue;
        }
        if let Ok(normalized) = headwater_graph::anchors::normalize(shape) {
            if !found.contains(&normalized) {
                found.push(normalized);
            }
        }
    }
    found
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
    fn score(&self, weighted: &[Weighted]) -> Scored {
        let mut score = 0;
        let mut separated = false;
        let mut terms = Vec::new();
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
            if surface > 0 && term.separating {
                separated = true;
                terms.push(term.term.clone());
            }
        }
        Scored {
            score,
            separated,
            terms,
        }
    }
}

impl Surface<'_> {
    /// Route a task description to pointers.
    pub fn route(&self, task: &str, budget: Budget) -> Route {
        self.route_in(task, budget, &|_| Entry::Absent)
    }

    /// Route a task description, reading which paths of it are on the tree
    /// through `tree` (#953). A word is read as the path it names as written
    /// wherever the tree holds that path, and never as a shorter one, so the
    /// verb that holds the tree passes it here. [`Surface::route`] is this with
    /// a tree that holds nothing.
    pub fn route_in(&self, task: &str, budget: Budget, tree: &dyn Fn(&str) -> Entry) -> Route {
        let terms = terms(task);
        let mut route = Route {
            task: task.to_string(),
            terms: terms.clone(),
            separating: Vec::new(),
            distinctive: Vec::new(),
            anchors: Vec::new(),
            matched: Vec::new(),
            pointers: Vec::new(),
            evidence: Vec::new(),
            withheld: 0,
            silence: None,
            ungoverned: Vec::new(),
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
        route.anchors = self.named_anchors(task, tree);
        route.ungoverned = self.ungoverned_in_scope(task, tree);
        let mut anchored: Vec<Pointer> = Vec::new();
        let mut evidence: Vec<(String, Evidence)> = Vec::new();
        for anchor in &route.anchors {
            for pointer in self.governing_docs_for_path(anchor) {
                match evidence.iter_mut().find(|(path, _)| path == &pointer.path) {
                    Some((_, Evidence::Named { anchors })) => anchors.push(anchor.clone()),
                    _ => evidence.push((
                        pointer.path.clone(),
                        Evidence::Named {
                            anchors: vec![anchor.clone()],
                        },
                    )),
                }
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
            return route.on_anchors_alone(anchored, &evidence, Silence::NoPurposes);
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
            return route.on_anchors_alone(anchored, &evidence, Silence::NoPurposeMatched);
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
            let scored = document_surfaces.score(&weighted);
            if !scored.separated {
                continue;
            }
            let pointer = self.pointer(document);
            if anchored.contains(&pointer) {
                continue;
            }
            candidates.push(Candidate {
                pointer,
                under: matched.purpose.clone(),
                purpose: matched.score,
                lexical: scored.score,
                terms: scored.terms,
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
        // The rank is a place in that order, taken before the turns reorder it,
        // because it is the answer to "where did this stand overall".
        let of = candidates.len();
        for (place, candidate) in candidates.iter().enumerate() {
            evidence.push((
                candidate.pointer.path.clone(),
                Evidence::Ranked {
                    terms: candidate.terms.clone(),
                    rank: place + 1,
                    of,
                },
            ));
        }
        let mut candidates = in_turns(candidates, &route.matched);
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
        route.evidence = evidence_for(&route.pointers, &evidence);
        route
    }

    /// The anchors of the graph that the task named outright.
    ///
    /// A word of the task is normalized by the same function the resolver used,
    /// and a hit is an exact match against a node the graph already holds. That
    /// is why no lexical score competes with it: nothing was inferred.
    /// [HW-DR-0074](../../../../docs/decisions/0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md):
    /// "the anchor half of `headwater route` match[es] by `Pattern::matches`
    /// over every pattern of every anchor and never by string equality." A
    /// task names a real path, never the pattern (or list of patterns) an
    /// anchor is written as, so what has to hold is whether some anchor's
    /// pattern set *reaches* the word, not whether the word spells an
    /// anchor's own identity.
    fn named_anchors(&self, task: &str, tree: &dyn Fn(&str) -> Entry) -> Vec<String> {
        let mut found: Vec<String> = Vec::new();
        for word in task.split_whitespace() {
            let Some(normalized) = self.read_path(word, tree) else {
                continue;
            };
            if found.contains(&normalized) {
                continue;
            }
            let reached = self
                .graph
                .edges
                .iter()
                .any(|edge| edge.target.reaches(&normalized));
            if reached {
                found.push(normalized);
            }
        }
        found
    }

    /// The path one word of a task names, or `None` where it names none.
    ///
    /// The word as written comes first, and a shorter reading is taken only
    /// where the longer one is not on the tree and no edge reaches it. So a
    /// file that exists as `tools/a:b.sh` or `tools/zz.` is read as written,
    /// and never as `tools/a` or `tools/zz`. Where the word as written names
    /// nothing, the first shorter reading that is on the tree or that an edge
    /// reaches is the path: `tools/hw-cargo's` names `tools/hw-cargo`.
    ///
    /// A word that no reading resolves names the path as written when nothing
    /// was taken off it, which is how a write of a new file is named. Where
    /// something would have to be taken off to name a path, and no shorter
    /// reading exists either, the word names nothing. A wrong path costs more
    /// than a missing one, because the route would propose an edge onto it.
    fn read_path(&self, word: &str, tree: &dyn Fn(&str) -> Entry) -> Option<String> {
        let readings = readings(word);
        let known = |path: &str| {
            tree(path) != Entry::Absent
                || self
                    .graph
                    .edges
                    .iter()
                    .any(|edge| edge.target.reaches(path))
        };
        if let Some(found) = readings.iter().find(|reading| known(reading)) {
            return Some(found.clone());
        }
        match readings.as_slice() {
            [only] => Some(only.clone()),
            _ => None,
        }
    }

    /// The paths of the task that the governed scope admits and that no
    /// governing edge reaches.
    ///
    /// A word counts only when it holds a `/`, so a prose task whose words
    /// happen to match a pattern such as `site/**` does not. The scope is the
    /// one `taxonomy audit` counts, read through [`Scope::declared`], and the
    /// test is [`Scope`]'s pattern test and no second matcher. What git ignores
    /// is not read here, because this crate runs no version control command:
    /// the verb drops those paths afterward with [`Route::retain_unignored`].
    ///
    /// [`Scope`]: headwater_graph::scope::Scope
    /// [`Scope::declared`]: headwater_graph::scope::Scope::declared
    fn ungoverned_in_scope(&self, task: &str, tree: &dyn Fn(&str) -> Entry) -> Vec<Ungoverned> {
        let scope = headwater_graph::scope::Scope::declared(self.relations());
        let mut found: Vec<Ungoverned> = Vec::new();
        if scope.is_empty() {
            return found;
        }
        for word in task.split_whitespace() {
            if !word.contains('/') {
                continue;
            }
            let Some(normalized) = self.read_path(word, tree) else {
                continue;
            };
            // A directory is not a path an edge governs one file of.
            if tree(&normalized) == Entry::Directory {
                continue;
            }
            if found.iter().any(|seen| seen.path == normalized) {
                continue;
            }
            let kinds: Vec<&str> = scope
                .members
                .iter()
                .filter(|member| {
                    member
                        .pattern
                        .as_ref()
                        .is_ok_and(|pattern| pattern.matches(&normalized))
                })
                .map(|member| member.anchor_kind.as_str())
                .collect();
            if kinds.is_empty() || !self.governing_docs_for_path(&normalized).is_empty() {
                continue;
            }
            let relations = self
                .relations()
                .relations
                .iter()
                .filter(|relation| relation.governs() == Governs::Source)
                .filter(|relation| relation.to.iter().any(|to| kinds.contains(&to.as_str())))
                .map(|relation| relation.name.clone())
                .collect();
            found.push(Ungoverned {
                path: normalized,
                relations,
            });
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
            // `anchor_display`, not `normalized`: this text feeds term
            // matching, and a list anchor's identity encoding is digits and
            // colons that would read as spurious terms — see
            // `Target::anchor_display`.
            if let Some(display) = edge.target.anchor_display() {
                text.push_str(&display);
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

/// The ranked candidates, in the order the budget takes them.
///
/// The matched purposes take turns, in the order of their scores. At its turn a
/// purpose offers its best remaining candidate. Inside a purpose the kinds that
/// serve it take turns the same way, in the order of their best candidate. A
/// purpose or a kind with nothing left is passed over. So one purpose takes
/// every slot only where no other matched purpose has a candidate, and one kind
/// takes every slot of a purpose only where no other kind of it has one.
///
/// The input arrives in the total order, and every choice here is a position in
/// it, so the result is total as well.
fn in_turns(candidates: Vec<Candidate>, matched: &[Matched]) -> Vec<Candidate> {
    use std::collections::VecDeque;
    // One list of kind queues for each purpose, in the order of `matched`.
    let mut purposes: Vec<Vec<(String, VecDeque<Candidate>)>> =
        matched.iter().map(|_| Vec::new()).collect();
    for candidate in candidates {
        let Some(at) = matched
            .iter()
            .position(|known| known.purpose == candidate.under)
        else {
            continue;
        };
        let kinds = &mut purposes[at];
        match kinds
            .iter_mut()
            .find(|(kind, _)| kind == &candidate.pointer.kind)
        {
            Some((_, queue)) => queue.push_back(candidate),
            None => kinds.push((candidate.pointer.kind.clone(), VecDeque::from([candidate]))),
        }
    }
    let mut turns: Vec<VecDeque<Candidate>> = purposes
        .into_iter()
        .map(|kinds| {
            let mut queues: Vec<VecDeque<Candidate>> =
                kinds.into_iter().map(|(_, queue)| queue).collect();
            let mut order = VecDeque::new();
            while taken_in_turn(&mut queues, |candidate| order.push_back(candidate)) {}
            order
        })
        .collect();
    let mut order = Vec::new();
    while taken_in_turn(&mut turns, |candidate| order.push(candidate)) {}
    order
}

/// One turn over a set of queues: the front of each queue that has one, in
/// queue order. It answers whether anything was taken.
fn taken_in_turn(
    queues: &mut [std::collections::VecDeque<Candidate>],
    mut take: impl FnMut(Candidate),
) -> bool {
    let mut taken = false;
    for queue in queues {
        if let Some(candidate) = queue.pop_front() {
            take(candidate);
            taken = true;
        }
    }
    taken
}

/// The evidence for each pointer, in the order of the pointers.
///
/// A route offers a document once, so its path is the key.
fn evidence_for(pointers: &[Pointer], known: &[(String, Evidence)]) -> Vec<Evidence> {
    pointers
        .iter()
        .filter_map(|pointer| {
            known
                .iter()
                .find(|(path, _)| path == &pointer.path)
                .map(|(_, evidence)| evidence.clone())
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
    /// Each pointer with the evidence for it.
    ///
    /// The evidence is `None` only for a route built by hand with fewer entries
    /// than pointers. Every route [`Surface::route`] returns carries one entry
    /// for each pointer.
    pub fn offers(&self) -> impl Iterator<Item = (&Pointer, Option<&Evidence>)> {
        self.pointers
            .iter()
            .enumerate()
            .map(|(at, pointer)| (pointer, self.evidence.get(at)))
    }

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
    fn on_anchors_alone(
        mut self,
        anchored: Vec<Pointer>,
        evidence: &[(String, Evidence)],
        silence: Silence,
    ) -> Route {
        self.evidence = evidence_for(&anchored, evidence);
        self.pointers = anchored;
        if self.pointers.is_empty() {
            self.silence = Some(silence);
        }
        self
    }

    /// Drop each ungoverned path that git ignores (#951's owner ruling:
    /// "nobody governs a cache"). The verb calls this, because this crate reads
    /// no version control, and it reads git only where the list is not empty.
    pub fn retain_unignored(&mut self, ignored: &headwater_graph::scope::Ignored) {
        self.ungoverned.retain(|entry| !ignored.covers(&entry.path));
    }

    /// The route as text: what it matched, and what it offers.
    ///
    /// A silent route prints why it is silent. Spec 5 makes silence a result,
    /// and a caller that could not tell "no purpose answers this" from "the
    /// corpus declares none" would debug the wrong file.
    ///
    /// `mode` is [`ColorMode::Plain`] for every machine reader — the JSON
    /// document and the MCP tool both carry this text — and the terminal state
    /// of standard output for the one human caller. `docs/interfaces/headwater-route.md`
    /// states that the default senses the stream and colors only there, and
    /// `tools/engine/color-fixtures.sh` attaches a real terminal to hold it.
    pub fn render(&self, mode: ColorMode) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(
            out,
            "{}",
            paint(Role::Heading, &format!("route {:?}", self.task), mode)
        );
        // The terms and the distinctive terms are the task read back, which
        // HW-DR-0045's table calls already-stated text and gives dim weight.
        let _ = writeln!(
            out,
            "{}",
            dim(&format!("  terms {}", self.terms.join(" ")), mode)
        );
        if !self.distinctive.is_empty() {
            let _ = writeln!(
                out,
                "{}",
                dim(
                    &format!("  distinctive {}", self.distinctive.join(" ")),
                    mode
                )
            );
        }
        for anchor in &self.anchors {
            let _ = writeln!(out, "  names the anchor {anchor}");
        }
        // No em dash on any of these lines, for the reason the withheld line
        // below gives: the write-time hook selects pointer lines by one.
        for entry in &self.ungoverned {
            let _ = writeln!(
                out,
                "  {} is in the governed scope, and nothing governs it",
                entry.path
            );
            if entry.relations.is_empty() {
                out.push_str("    no declared relation that governs takes it as a target\n");
                continue;
            }
            out.push_str("    a document declares the edge in its front matter:\n");
            out.push_str("    relations:\n");
            for relation in &entry.relations {
                let _ = writeln!(out, "      {relation}:\n        - {}", entry.path);
            }
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
        // Filled first and painted after, in that order and not the other one.
        // `headwater_check::fill::filled` measures a line in characters, and an
        // SGR sequence is characters that occupy no column: a path painted
        // before the fold would push the fold six characters early and leave
        // the reset code counted as a word. So the pointer is composed plain,
        // folded, and the path — which the fold never splits, because it holds
        // no space — is painted in place afterwards.
        //
        // The evidence line under a pointer is the task read back, so it is
        // dim like the terms line. It carries no em dash, for the reason the
        // withheld line below gives.
        for (pointer, evidence) in self.offers() {
            let folded = headwater_check::fill::filled(
                &format!("  {}\n", pointer.render()),
                headwater_check::fill::WIDTH,
            );
            out.push_str(&folded.replacen(
                &pointer.path,
                &paint(Role::Path, &pointer.path, mode),
                1,
            ));
            if let Some(evidence) = evidence {
                let folded = headwater_check::fill::filled(
                    &format!("    {}\n", evidence.render()),
                    headwater_check::fill::WIDTH,
                );
                for line in folded.lines() {
                    let _ = writeln!(out, "{}", dim(line, mode));
                }
            }
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
            let _ = writeln!(
                out,
                "{}",
                dim(
                    &format!("  the budget withheld {} more {more}", self.withheld),
                    mode
                )
            );
        }
        out
    }
}

/// What the route report paints, and the one property that holds over all of it.
///
/// A table of `(what, the escape the palette gives it)`, in the shape
/// `headwater_check::paint::tests` and `engine/crates/cli/tests/width.rs`
/// already set. These are necessary and not sufficient: a call site that hands
/// `ColorMode::Plain` to a renderer forever passes every one of them, which is
/// what `tools/engine/color-fixtures.sh` attaches a real terminal to catch.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pointer;

    /// The text with every SGR sequence removed.
    ///
    /// Written here rather than taken from the painter, because the property
    /// below is that color adds escape sequences and changes nothing else, and
    /// a stripper that shared code with the painter could not state it.
    fn stripped(text: &str) -> String {
        let mut out = String::new();
        let mut chars = text.chars();
        while let Some(c) = chars.next() {
            if c != '\u{1b}' {
                out.push(c);
                continue;
            }
            for c in chars.by_ref() {
                if c == 'm' {
                    break;
                }
            }
        }
        out
    }

    /// A route with one of everything the report can print: a task, terms,
    /// distinctive terms, an anchor, a matched purpose, a pointer and a
    /// withheld count. A route missing any of them would leave a painted line
    /// unreached, and an unreached line is where a mode goes unthreaded.
    fn route() -> Route {
        Route {
            task: "add rate limiting".to_string(),
            terms: vec!["add".to_string(), "rate".to_string()],
            separating: vec!["rate".to_string()],
            distinctive: vec!["rate".to_string()],
            anchors: vec!["HW-DR-0045".to_string()],
            matched: vec![Matched {
                purpose: "behavior".to_string(),
                score: 7,
            }],
            pointers: vec![Pointer {
                path: "docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md"
                    .to_string(),
                id: Some("HW-DR-0045".to_string()),
                kind: "decision".to_string(),
                name: None,
                purpose: Some("rationale".to_string()),
                summary: Some("what colors, and where the banner goes".to_string()),
                unwarranted: false,
            }],
            evidence: vec![Evidence::Ranked {
                terms: vec!["rate".to_string()],
                rank: 2,
                of: 5,
            }],
            withheld: 3,
            silence: None,
            ungoverned: Vec::new(),
        }
    }

    #[test]
    fn each_line_reaches_the_role_the_palette_gives_it() {
        struct Case {
            what: &'static str,
            opens_with: &'static str,
        }
        let cases = [
            Case {
                what: "the task line is a heading, bold in the default color",
                opens_with: "\u{1b}[1mroute \"add rate limiting\"",
            },
            Case {
                what: "the terms are already-stated text, dim",
                opens_with: "\u{1b}[2m  terms add rate",
            },
            Case {
                what: "the distinctive terms are the same",
                opens_with: "\u{1b}[2m  distinctive rate",
            },
            Case {
                what: "a pointer path is a path, cyan",
                opens_with: "\u{1b}[36mdocs/decisions/0045-",
            },
            Case {
                what: "the evidence under a pointer is the task read back, dim",
                opens_with: "\u{1b}[2m    matched rate, rank 2 of 5",
            },
            Case {
                what: "the withheld count is a count, dim",
                opens_with: "\u{1b}[2m  the budget withheld 3 more pointers",
            },
        ];
        let painted = route().render(ColorMode::Ansi);
        for case in cases {
            assert!(
                painted.contains(case.opens_with),
                "{}: no `{}` in\n{painted}",
                case.what,
                case.opens_with.escape_debug()
            );
        }
    }

    /// The property no table can state: color adds escape sequences and moves
    /// no other byte. A painter that dropped a word, or that painted before the
    /// fold and so moved a line break, fails here and nowhere else.
    #[test]
    fn color_adds_escape_sequences_and_changes_nothing_else() {
        let route = route();
        assert_eq!(
            stripped(&route.render(ColorMode::Ansi)),
            route.render(ColorMode::Plain)
        );
    }

    /// `Plain` writes not one escape byte, which is what every recorded fixture
    /// and both machine readers of this text depend on.
    #[test]
    fn plain_writes_no_escape_byte() {
        assert!(!route().render(ColorMode::Plain).contains('\u{1b}'));
    }

    /// A pointer long enough to fold, painted. The fold runs first and the
    /// paint second, so the line break falls where an unpainted run puts it.
    #[test]
    fn a_folded_pointer_breaks_where_an_unpainted_one_does() {
        let mut route = route();
        route.pointers[0].summary = Some(
            "a summary long enough that the fill has to break it, twice over, so that a paint \
             that ran before the fold would be visible as a line break in a different place"
                .to_string(),
        );
        assert_eq!(
            stripped(&route.render(ColorMode::Ansi)),
            route.render(ColorMode::Plain)
        );
    }

    fn candidate(path: &str, kind: &str, under: &str, purpose: u32) -> Candidate {
        let mut pointer = route().pointers[0].clone();
        pointer.path = path.to_string();
        pointer.kind = kind.to_string();
        Candidate {
            pointer,
            under: under.to_string(),
            purpose,
            lexical: 0,
            terms: Vec::new(),
        }
    }

    /// The turns HW-DR-0070 rules, over a list already in the total order.
    ///
    /// The purpose that scored highest holds the first four candidates, and
    /// before the ruling it took every slot of a budget of four. Now each
    /// purpose offers one in turn, and inside `obligation` the second kind is
    /// offered before the first kind offers again.
    #[test]
    fn the_purposes_take_turns_and_the_kinds_inside_a_purpose_do_too() {
        let matched = [
            Matched {
                purpose: "obligation".to_string(),
                score: 7,
            },
            Matched {
                purpose: "behavior".to_string(),
                score: 3,
            },
        ];
        let ordered = vec![
            candidate("o1", "obligation_record", "obligation", 7),
            candidate("o2", "obligation_record", "obligation", 7),
            candidate("o3", "obligation_record", "obligation", 7),
            candidate("r1", "obligation_register", "obligation", 7),
            candidate("s1", "design_spec", "behavior", 3),
            candidate("i1", "interface_contract", "behavior", 3),
        ];
        let paths: Vec<String> = in_turns(ordered, &matched)
            .into_iter()
            .map(|candidate| candidate.pointer.path)
            .collect();
        assert_eq!(paths, ["o1", "s1", "r1", "i1", "o2", "o3"]);
    }

    /// One matched purpose with one kind keeps the order it arrived in, which
    /// is the order every route with a single purpose had before the ruling.
    #[test]
    fn a_single_purpose_of_a_single_kind_keeps_its_order() {
        let matched = [Matched {
            purpose: "rationale".to_string(),
            score: 4,
        }];
        let ordered = vec![
            candidate("a", "decision", "rationale", 4),
            candidate("b", "decision", "rationale", 4),
            candidate("c", "decision", "rationale", 4),
        ];
        let paths: Vec<String> = in_turns(ordered, &matched)
            .into_iter()
            .map(|candidate| candidate.pointer.path)
            .collect();
        assert_eq!(paths, ["a", "b", "c"]);
    }

    /// A silent route returns early, above the pointer loop, and its own lines
    /// are painted too.
    #[test]
    fn a_silent_route_is_painted_as_well() {
        let mut route = route();
        route.pointers.clear();
        route.silence = Some(Silence::NoPurposeMatched);
        let painted = route.render(ColorMode::Ansi);
        assert!(
            painted.contains("\u{1b}[1mroute \"add rate limiting\""),
            "{painted}"
        );
        assert_eq!(
            stripped(&painted),
            route.render(ColorMode::Plain),
            "a silent route paints nothing but escapes either"
        );
    }
}
