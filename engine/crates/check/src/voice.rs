// SPDX-License-Identifier: Apache-2.0
//! A Document-origin check: the constructions a voice regime forbids.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-five-origins-of-a-check)
//! puts voice first among the Document-origin examples and says what separates
//! that origin from the two generated ones: "Document checks are the ones that
//! no graph standard can reach, because the body is not in the graph."
//!
//! # What is data here, and what is not
//!
//! The regime is data. Which kinds answer to it is data, and so is the list of
//! categories it forbids: this file names no kind and no category, and a
//! taxonomy that forbids one more produces one more finding with no code.
//!
//! The **patterns** are not data, and the meta-schema says why. `voice_regime`
//! carries one member, `forbid: {seq: {scalar: string}}`, with a comment
//! marking it a gap: "spec 3 names the forbidden constructions in prose and no
//! value set states them." So the names are an open set in the language and a
//! closed set in this engine, and the two disagree by construction.
//!
//! A category this engine has no pattern set for is therefore not ignored. The
//! instance **skips with a reason that names the category**, which is the same
//! posture [`crate::participation`] takes toward a window it cannot read, and
//! it is what keeps the gap visible per document rather than per release.
//!
//! # The patterns read author-owned sentences and nothing else
//!
//! [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#what-a-lexical-rule-gets-wrong-and-where-posture-comes-from)
//! rules that "a quotation, a code span, a citation line, and a generated block
//! are outside every voice rule by construction. This is not an exemption that
//! a rule declares." Nothing below declares one:
//! [`headwater_doc::Sentence::authored`] is the text the parser says this
//! author wrote as prose, and it is the only string these patterns see.
//!
//! # Severity, and why no fix is offered
//!
//! Advisory, permanently, and the reason is fixability rather than precision.
//! [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#voice) states it
//! after Q5 measured the alternative: "a category may become blocking only when
//! its remediation is mechanical and total ... A category whose remediation is
//! a rewrite stays advisory permanently, whatever its false-positive rate turns
//! out to be." Every category here is a rewrite. Spec 3 names phased-rollout
//! language as the most likely candidate for a mechanical fix, and adds that
//! nobody has built one. That is still true.
//!
//! # This rule is half of Q5's instrument
//!
//! [Q5](../../../../docs/spec/09-decisions.md#q5--voice-checking-depth) closed on
//! a measurement of three ASD-STE100 structural rules, used as proxies, and
//! [spec 13](../../../../docs/spec/13-open-obligations.md) records what stayed
//! unmeasured: nobody has measured `future_intent`, `change_narration` or
//! `phased_rollout`, "because no implementation of them exists". This file is
//! that implementation. The other half is a human campaign, and no code
//! discharges it: an adjudicated sample of at least 50 findings per category,
//! under the two labels spec 4 declares.

use crate::finding::{Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView};
use crate::shape::Shape;
use headwater_doc::Sentence;

pub const RULE: &str = "voice.forbidden_construction";

/// One forbidden category, and how it reads in prose.
struct Category {
    /// The name a regime writes.
    name: &'static str,
    /// What a reader is asked to write instead. One sentence, because it is
    /// the whole of the remediation a rewrite can be told in advance.
    instead: &'static str,
    /// The curated pattern set. Each entry is a lower-case substring of an
    /// author-owned sentence, matched on word boundaries.
    patterns: &'static [&'static str],
}

/// The categories this engine knows, and the only ones it can report on.
///
/// The set is closed here and open in the language. See the module comment for
/// why the two differ and what an instance does when it meets the difference.
///
/// Each pattern set is curated rather than derived, which is
/// [Q5](../../../../docs/spec/09-decisions.md#q5--voice-checking-depth)'s own
/// ruling: "the curated pattern set, the per-category posture and the reasoned
/// escape hatch all stand". The sets are small on purpose. Q5 measured where a
/// lexical checker's errors come from, and the lexicon produced approximately
/// none of them, so a set that stays inside what the words plainly mean keeps
/// that property.
const CATEGORIES: [Category; 3] = [
    Category {
        name: "future_intent",
        instead:
            "state what the system does now, or move the sentence to a document whose kind narrates",
        patterns: &[
            "will be",
            "will become",
            "will then",
            "will eventually",
            "will later",
            "is planned",
            "are planned",
            "we plan to",
            "in the future",
            "at a later date",
            "to be added",
            "to be decided",
            "coming soon",
            "for a future release",
        ],
    },
    Category {
        name: "change_narration",
        instead: "state the position that holds, and leave the change to the decision that made it",
        // Two tiers, and the tier is a record of curation rather than a
        // severity. Spec 3 rules that "posture per category comes from
        // fixability and never from precision", so a less precise pattern
        // reports at the same weight as a precise one or it does not ship.
        //
        // Tier one was measured against this corpus and every instance of each
        // entry was change narration. Tier two plainly means a change in
        // general English and has one false-positive mode here: a sentence that
        // *defines* a change rather than narrating one, as in "a term that the
        // corpus no longer uses". That is what the `false_positive` directive
        // is for.
        //
        // Three candidates were measured and rejected, and the reason each one
        // failed is why this set is curated against a corpus and not from
        // intuition. `the retired` matched four sentences and none was a
        // narration: "retired term" and "retired phrase" are this system's own
        // vocabulary. `at one point` matched the positional sense, "worth
        // making at one point in a text". `was replaced` matched three
        // sentences that describe a measurement procedure, "every inline
        // quotation was replaced by one word".
        patterns: &[
            // Tier one.
            "we moved from",
            "we changed",
            "we renamed",
            "we replaced",
            "used to be",
            "used to have",
            "used to",
            "was renamed",
            "were renamed",
            "has been renamed",
            "as before",
            "unlike before",
            "in the old",
            "the previous version",
            "an earlier version",
            "there was no",
            "there were no",
            "did not exist",
            "has since",
            "formerly",
            // Tier two.
            "no longer",
            "now that",
            "previously",
        ],
    },
    Category {
        name: "phased_rollout",
        instead: "state what holds, and record the sequence in the plan that owns it",
        patterns: &[
            "phase one",
            "phase two",
            "first phase",
            "second phase",
            "in a later phase",
            "for now",
            "at first",
            "to begin with",
            "in the first release",
            "in a later release",
            "rolls out",
            "rolled out in",
            "rollout of",
        ],
    },
];

/// The check, generated from the voice regimes and the kinds that bind them.
pub struct Voice {
    /// Each kind, and what its regime forbids: the categories this engine can
    /// report, and the names it cannot. Computed once, because the generation
    /// step reads the taxonomy and the evaluation step reads one document.
    bound: Vec<Bound>,
}

struct Bound {
    kind: String,
    regime: String,
    /// Indices into [`CATEGORIES`], in the order the regime declares them.
    known: Vec<usize>,
    /// The names the regime forbids that this engine has no pattern set for.
    unknown: Vec<String>,
}

impl Voice {
    /// The generation step, in full.
    pub fn over(shape: &Shape) -> Self {
        let mut bound = Vec::new();
        for kind in &shape.kinds {
            let Some(regime) = shape.voice_of(&kind.name) else {
                continue;
            };
            if regime.forbid.is_empty() {
                continue;
            }
            let mut known = Vec::new();
            let mut unknown = Vec::new();
            for name in &regime.forbid {
                match CATEGORIES
                    .iter()
                    .position(|category| category.name == name.as_str())
                {
                    Some(index) => known.push(index),
                    None => unknown.push(name.clone()),
                }
            }
            bound.push(Bound {
                kind: kind.name.clone(),
                regime: regime.name.clone(),
                known,
                unknown,
            });
        }
        Voice { bound }
    }

    fn bound_to(&self, kind: &str) -> Option<&Bound> {
        self.bound.iter().find(|bound| bound.kind == kind)
    }
}

impl DocumentCheck for Voice {
    const RULE: &'static str = self::RULE;
    /// Raise this when a pattern set changes, because a changed pattern set is
    /// a changed verdict and the cache holds the old one ([`crate::cache`]).
    ///
    /// Edition two widens `change_narration`. Edition one reported nothing over
    /// this corpus on 258 instances, and not one of its forty patterns occurred
    /// in a declarative document, so the set was saturated rather than
    /// satisfied.
    const VERSION: u32 = 2;
    /// The body, because the regime is about prose. This declaration is the
    /// access: without it [`DocumentView::body`] returns nothing.
    const NEEDS_BODY: bool = true;

    /// A kind whose chain binds no voice regime, or one that forbids nothing,
    /// generates no instance. The narrative regime is the second case: spec 3
    /// makes evidence documents "time-bound by nature, and exempt", and an
    /// instance over one would count it as checked by a rule with nothing to
    /// check.
    fn instantiates(&self, kind: &str) -> bool {
        self.bound_to(kind).is_some()
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let Some(bound) = self.bound_to(view.kind()) else {
            return Outcome::Passed;
        };
        // A regime whose every category is outside this engine's set decides
        // nothing, and says so. A regime with one readable category reports on
        // that one: a skip would lose the finding, and a silent pass would
        // claim the unreadable categories held.
        if bound.known.is_empty() {
            return Outcome::Skipped(format!(
                "`{}` forbids {}, and this engine has no pattern set for {}",
                bound.regime,
                bound.unknown.join(", "),
                match bound.unknown.len() {
                    1 => "it",
                    _ => "any of them",
                }
            ));
        }
        let Some(body) = view.body() else {
            return Outcome::Passed;
        };

        let sentences = body.sentences();
        let mut findings = Vec::new();
        for index in &bound.known {
            let category = &CATEGORIES[*index];
            for sentence in &sentences {
                let Some(pattern) = matched(category, sentence) else {
                    continue;
                };
                findings.push(Finding {
                    rule: self::RULE,
                    severity: Severity::Warn,
                    obligation: None,
                    path: view.path().to_string(),
                    line: sentence.span.start.line,
                    column: sentence.span.start.col,
                    message: format!(
                        "`{}` forbids {}, and this sentence writes `{pattern}`",
                        bound.regime, category.name
                    ),
                    remediation: category.instead.to_string(),
                    // A rewrite is never mechanical, which is the whole of why
                    // this rule is advisory. See the module comment.
                    patch: None,
                });
            }
        }
        // In document order, and one finding per sentence for each category it
        // offends. Two categories in one sentence are two things to fix.
        findings.sort_by_key(|finding| (finding.line, finding.column));
        Outcome::failed(findings)
    }
}

/// The first pattern of a category that a sentence writes, if any.
///
/// One finding per sentence per category. A sentence that writes `will be`
/// twice offends once, because the author reads the sentence once.
fn matched(category: &Category, sentence: &Sentence) -> Option<&'static str> {
    let text = sentence.authored.to_lowercase();
    category
        .patterns
        .iter()
        .copied()
        .find(|pattern| contains_word(&text, pattern))
}

/// Whether `text` holds `pattern` at word boundaries.
///
/// The boundary matters for the same reason it matters in the abbreviation
/// list of the splitter: `at first` inside `at first-order` is a different
/// phrase, and a rule that reported it would be a false positive that no
/// author can act on.
pub(crate) fn contains_word(text: &str, pattern: &str) -> bool {
    word_at(text, pattern).is_some()
}

/// Where `text` first holds `pattern` at word boundaries, as a byte offset.
///
/// One definition of the boundary rather than two. A retired term with a
/// replacement needs the position as well as the fact, and a second search that
/// drew the boundary differently would patch a word this rule did not report.
pub(crate) fn word_at(text: &str, pattern: &str) -> Option<usize> {
    if pattern.is_empty() {
        return None;
    }
    let mut from = 0usize;
    while let Some(offset) = text.get(from..)?.find(pattern) {
        let at = from + offset;
        let end = at + pattern.len();
        let before = text[..at].chars().next_back().is_none_or(|c| !is_word(c));
        let after = text[end..].chars().next().is_none_or(|c| !is_word(c));
        if before && after {
            return Some(at);
        }
        from = end;
    }
    None
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '-'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pattern_matches_at_word_boundaries_only() {
        assert!(contains_word("the rule will be read", "will be"));
        assert!(!contains_word("a goodwill better than", "will be"));
        assert!(!contains_word("at first-order logic", "at first"));
        assert!(contains_word("at first, it reads", "at first"));
    }

    /// The set that edition one missed. Each of these is a sentence this
    /// corpus wrote under a declarative regime and edition one passed.
    #[test]
    fn edition_two_reports_the_shapes_edition_one_missed() {
        let narration = &CATEGORIES[1];
        assert_eq!(narration.name, "change_narration");
        for sentence in [
            "until the ruling there was no machine form of this verb",
            "the emitter writes a routine that did not exist before",
            "the table that the entry used to carry decided it on the wrong axis",
            "that script has since retired into the check layer",
            "an earlier version of the entry named a second field",
            "the field is no longer read",
            "the register was formerly a single file",
        ] {
            assert!(
                narration
                    .patterns
                    .iter()
                    .any(|pattern| contains_word(sentence, pattern)),
                "no pattern matched `{sentence}`"
            );
        }
    }

    /// The three candidates measured and rejected, held here so that a later
    /// widening does not readmit one. Each sentence is from this corpus.
    #[test]
    fn the_rejected_candidates_stay_out() {
        let narration = &CATEGORIES[1];
        for sentence in [
            "the british spelling and the retired term are warnings",
            "evidence that an author found a reference worth making at one point in a text",
            "every inline quotation of five characters or more was replaced by one word",
        ] {
            assert!(
                !narration
                    .patterns
                    .iter()
                    .any(|pattern| contains_word(sentence, pattern)),
                "a pattern matched `{sentence}`, which is not change narration"
            );
        }
    }
}
