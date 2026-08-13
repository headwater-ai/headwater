// SPDX-License-Identifier: Apache-2.0
//! A Document-origin check: the controlled language a kind declares.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-five-origins-of-a-check)
//! lists normative language among the Document-origin examples, and a language
//! regime is where a taxonomy says which prose rules its documents obey.
//!
//! # The declaration this rule was written for is already committed
//!
//! `.headwater/overlay.yml` declares `regimes.language.ste_house` with
//! `controlled: ASD-STE100, profile: house`, binds it to three kinds, and says
//! in a comment what was wrong with that: "`tools/ste-lint.py` is the thing
//! that enforces it today ... the rule is in CLAUDE.md today, which is a place
//! no check can read". This rule is the place a check can read.
//!
//! # A profile the engine does not know skips, and says which one
//!
//! `language_regime` marks `controlled` a gap in the meta-schema, for a stated
//! reason: spec 2 names `none`, `ste-house` and `ste-strict`, this repository
//! writes `ASD-STE100` with a separate `profile`, and a closed set in the
//! language would refuse a committed source. So the engine matches the pairs it
//! knows, and an instance that meets any other pair **skips with a reason that
//! names it**. That keeps the gap visible per document. A regime whose
//! `controlled` is `none` generates no instance at all, because there is
//! nothing to be held to.
//!
//! # What the house profile holds prose to
//!
//! Three rules, and each one is an exact match rather than a judgment. Two of
//! the three are the reason [`headwater_doc::sentences`] exists.
//!
//! - **Sentence length**, ASD-STE100 writing rule 6.3 for descriptive text.
//!   Advisory: the remediation is a rewrite, and
//!   [spec 4](../../../../docs/spec/04-assurance-model.md#where-promotion-cannot-finish)
//!   makes that permanently advisory whatever the precision turns out to be.
//! - **No contraction.** An error, because the expansion is mechanical and
//!   total, which is the bar [spec 12](../../../../docs/spec/12-check-layer.md#fixability)
//!   sets. No patch rides along yet: `--fix` is
//!   [#71](https://github.com/headwater-ai/headwater/issues/71).
//! - **No semicolon in running prose**, ASD-STE100 writing rule 8.1. Advisory,
//!   on the same bar: the remediation is to decide which clause carries the
//!   sentence, and no rewrite of that kind is mechanical. Running prose is a
//!   paragraph or a list item, and never a heading or a table cell, where a
//!   semicolon separates items rather than joins clauses.
//! - **The spelling variant the tag declares.** An error on the same terms. The
//!   tag is data and the word list is not, which is the same split as the voice
//!   patterns and is recorded in spec 13 as one gap rather than two.
//!
//! # This rule does not retire `tools/ste-lint.py`, and the PR that added it
//! says what still keeps the Python alive
//!
//! The memory note is that the lint hooks are interim scaffolding to be retired
//! into this layer rather than grown. This is the first cut of that retirement
//! and not the end of it.

use crate::finding::{Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView};
use crate::shape::Shape;
use headwater_doc::body::BlockKind;

pub const RULE: &str = "language.controlled.not_met";

/// The sentence length ASD-STE100 rule 6.3 allows in descriptive text.
const MAX_WORDS: usize = 25;

/// The controlled language and profile pairs this engine has rules for.
///
/// Two spellings of one thing: spec 2 writes the profile into `controlled`,
/// and this repository's overlay separates the two. Both name the house
/// profile, and neither is more correct than the other while the meta-schema
/// leaves the position a gap.
const HOUSE: [(&str, Option<&str>); 2] = [("ASD-STE100", Some("house")), ("ste-house", None)];

/// The words whose American spelling this corpus writes, by their British one.
///
/// The list is the one `CLAUDE.md` states, moved to where a check reads it.
/// A closed table of exact words rather than a suffix rule, because a suffix
/// rule reports `enterprise` and `precise`, and
/// [Q5](../../../../docs/spec/09-decisions.md#q5--voice-checking-depth) measured
/// that a lexicon of exact strings is the highest-precision shape a lexical
/// rule takes.
const SPELLINGS: [(&str, &str); 24] = [
    ("organisation", "organization"),
    ("organisations", "organizations"),
    ("organise", "organize"),
    ("organised", "organized"),
    ("organises", "organizes"),
    ("organising", "organizing"),
    ("artefact", "artifact"),
    ("artefacts", "artifacts"),
    ("behaviour", "behavior"),
    ("behaviours", "behaviors"),
    ("behavioural", "behavioral"),
    ("customise", "customize"),
    ("customised", "customized"),
    ("customises", "customizes"),
    ("customising", "customizing"),
    ("customisation", "customization"),
    ("judgement", "judgment"),
    ("judgements", "judgments"),
    ("licence", "license"),
    ("licences", "licenses"),
    ("licenced", "licensed"),
    ("catalogue", "catalog"),
    ("catalogues", "catalogs"),
    ("catalogued", "cataloged"),
];

/// The check, generated from the language regimes and the kinds that bind them.
pub struct Language {
    bound: Vec<Bound>,
}

struct Bound {
    kind: String,
    regime: String,
    /// The profile this engine matched, and nothing for a pair it does not
    /// know. See the module comment.
    profile: Option<Profile>,
    /// How the regime spelled the language it declares, for the skip reason.
    declared: String,
    /// The BCP 47 tag, which decides whether the spelling rule applies.
    tag: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Profile {
    House,
}

impl Language {
    /// The generation step, in full.
    pub fn over(shape: &Shape) -> Self {
        let mut bound = Vec::new();
        for kind in &shape.kinds {
            let Some(regime) = shape.language_of(&kind.name) else {
                continue;
            };
            let controlled = regime.controlled.as_deref().unwrap_or("none");
            // `none` is a regime that holds prose to nothing, and a document
            // under one owes this rule no instance.
            if controlled.eq_ignore_ascii_case("none") {
                continue;
            }
            let profile = HOUSE
                .iter()
                .any(|(language, profile)| {
                    language.eq_ignore_ascii_case(controlled)
                        && *profile == regime.profile.as_deref()
                })
                .then_some(Profile::House);
            bound.push(Bound {
                kind: kind.name.clone(),
                regime: regime.name.clone(),
                profile,
                declared: match &regime.profile {
                    Some(profile) => format!("{controlled}, profile {profile}"),
                    None => controlled.to_string(),
                },
                tag: regime.tag.clone(),
            });
        }
        Language { bound }
    }

    fn bound_to(&self, kind: &str) -> Option<&Bound> {
        self.bound.iter().find(|bound| bound.kind == kind)
    }
}

impl DocumentCheck for Language {
    const RULE: &'static str = self::RULE;
    /// The first edition of this rule. Raise it when a profile's rules change.
    const VERSION: u32 = 1;
    const NEEDS_BODY: bool = true;

    fn instantiates(&self, kind: &str) -> bool {
        self.bound_to(kind).is_some()
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let Some(bound) = self.bound_to(view.kind()) else {
            return Outcome::Passed;
        };
        if bound.profile.is_none() {
            return Outcome::Skipped(format!(
                "`{}` declares {}, and this engine has no rules for it",
                bound.regime, bound.declared
            ));
        }
        let Some(body) = view.body() else {
            return Outcome::Passed;
        };

        let american = bound.tag.eq_ignore_ascii_case("en-US");
        let mut findings = Vec::new();
        for sentence in body.sentences() {
            let at = |severity, message: String, remediation: String| Finding {
                rule: self::RULE,
                severity,
                obligation: None,
                path: view.path().to_string(),
                line: sentence.span.start.line,
                column: sentence.span.start.col,
                message,
                remediation,
                // The expansion of a contraction is mechanical, and this rule
                // still offers no patch: `check --fix` is #71, and a `fixable`
                // flag with nothing behind it would claim a capability the
                // engine does not have.
                fixable: false,
            };

            if sentence.words > MAX_WORDS && !is_a_citation_line(&sentence.text) {
                findings.push(at(
                    Severity::Warn,
                    format!(
                        "`{}` holds prose to {MAX_WORDS} words a sentence, and this one has {}",
                        bound.regime, sentence.words
                    ),
                    "split the sentence, or move a clause into its own sentence".to_string(),
                ));
            }
            if runs_on(&sentence.authored) && is_running_prose(sentence.kind) {
                findings.push(at(
                    Severity::Warn,
                    format!(
                        "`{}` admits no semicolon in running prose, and this sentence writes one",
                        bound.regime
                    ),
                    "split the sentence in two, and let each half state one thing".to_string(),
                ));
            }
            if let Some(word) = contraction(&sentence.authored) {
                findings.push(at(
                    Severity::Error,
                    format!(
                        "`{}` admits no contraction, and this sentence writes `{word}`",
                        bound.regime
                    ),
                    format!("write `{word}` out in full"),
                ));
            }
            if american {
                if let Some((british, american)) = spelling(&sentence.authored) {
                    findings.push(at(
                        Severity::Error,
                        format!(
                            "`{}` declares the tag `{}`, and this sentence writes `{british}`",
                            bound.regime, bound.tag
                        ),
                        format!("write `{american}`"),
                    ));
                }
            }
        }
        findings.sort_by_key(|finding| (finding.line, finding.column));
        Outcome::failed(findings)
    }
}

/// Whether a run of text is a list of citations rather than a sentence.
///
/// The rule and its reason are `tools/ste-lint.py`'s, moved here unchanged: the
/// sentence limit is about how much a reader holds at once while following an
/// argument, and a bibliography line asks nothing of the sort. It is read one
/// entry at a time, and splitting it would only make it longer.
///
/// Two separators rather than one, so that a sentence with a single mid-dot in
/// it is still a sentence.
fn is_a_citation_line(text: &str) -> bool {
    text.matches(" · ").count() >= 2
}

/// Whether a block holds running prose, as opposed to a label.
///
/// A heading and a table cell are lists of terms with punctuation between them,
/// and a semicolon there separates items rather than joins two clauses. The
/// rule and its reason are `tools/ste-lint.py`'s, moved here unchanged.
fn is_running_prose(kind: BlockKind) -> bool {
    matches!(kind, BlockKind::Paragraph | BlockKind::Item)
}

/// Whether a sentence writes a semicolon outside a parenthesis.
///
/// A semicolon inside a parenthesis separates the items of an aside, and a
/// citation is the case this corpus writes: `(Smith, 2004; Jones, 2007)` is one
/// parenthetical rather than a run-on sentence.
fn runs_on(text: &str) -> bool {
    let mut depth = 0usize;
    for c in text.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ';' if depth == 0 => return true,
            _ => {}
        }
    }
    false
}

/// The first contraction of a sentence, if any.
///
/// A contraction is an apostrophe between letters where the letters after it
/// are one of a closed set of endings, or an `n't`. A possessive `'s` is not
/// one, and separating the two is the whole of the rule: this corpus writes
/// `Headwater's` on almost every page.
fn contraction(text: &str) -> Option<String> {
    let words = text.split(|c: char| c.is_whitespace());
    for word in words {
        let cleaned: String = word
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '\'' || *c == '\u{2019}')
            .collect();
        let lower = cleaned.to_lowercase();
        let Some(at) = lower.find(['\'', '\u{2019}']) else {
            continue;
        };
        let (head, tail) = lower.split_at(at);
        let tail: String = tail.chars().skip(1).collect();
        if head.is_empty() || tail.is_empty() {
            continue;
        }
        // `n't` is a contraction whatever the verb in front of it is.
        if head.ends_with('n') && tail == "t" {
            return Some(cleaned);
        }
        if ["re", "ve", "ll", "d", "m"].contains(&tail.as_str()) {
            return Some(cleaned);
        }
        // `it's`, `that's` and their kin are contractions; every other `'s` is
        // a possessive.
        if tail == "s"
            && [
                "it", "that", "there", "here", "what", "who", "let", "he", "she", "which", "this",
            ]
            .contains(&head)
        {
            return Some(cleaned);
        }
    }
    None
}

/// The first British spelling of a sentence, with the American one beside it.
fn spelling(text: &str) -> Option<(String, &'static str)> {
    for word in text.split(|c: char| !c.is_alphanumeric()) {
        let lower = word.to_lowercase();
        if let Some((_, american)) = SPELLINGS
            .iter()
            .find(|(british, _)| *british == lower.as_str())
        {
            return Some((word.to_string(), american));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The distinction the rule exists to make. This corpus writes a possessive
    /// on nearly every page, and a rule that read one as a contraction would
    /// report the whole specification.
    #[test]
    fn a_possessive_is_not_a_contraction() {
        assert_eq!(
            contraction("Headwater's own corpus and the engine's lock"),
            None
        );
        assert_eq!(contraction("the adopter's overlay"), None);
        assert_eq!(contraction("it does not hold"), None);
    }

    #[test]
    fn a_contraction_is_found_whatever_its_ending() {
        assert_eq!(contraction("it doesn't hold").as_deref(), Some("doesn't"));
        assert_eq!(contraction("they're here").as_deref(), Some("they're"));
        assert_eq!(contraction("it's here").as_deref(), Some("it's"));
        assert_eq!(contraction("we'll read it").as_deref(), Some("we'll"));
    }

    #[test]
    fn a_semicolon_inside_a_parenthesis_is_not_a_run_on() {
        assert!(runs_on(
            "The resolver reads one profile; the rest are the adopter's"
        ));
        assert!(!runs_on(
            "The two sources agree (Smith, 2004; Jones, 2007) on the shape"
        ));
        assert!(!runs_on("No semicolon at all"));
    }

    #[test]
    fn a_semicolon_is_a_run_on_only_in_running_prose() {
        assert!(is_running_prose(BlockKind::Paragraph));
        assert!(is_running_prose(BlockKind::Item));
        assert!(!is_running_prose(BlockKind::Heading(2)));
        assert!(!is_running_prose(BlockKind::TableCell));
    }

    #[test]
    fn a_british_spelling_is_found_with_its_american_form() {
        assert_eq!(
            spelling("the behaviour of a check"),
            Some(("behaviour".to_string(), "behavior"))
        );
        assert_eq!(spelling("the behavior of a check"), None);
        // A suffix rule would report both of these, and the table does not.
        assert_eq!(spelling("an enterprise that is precise"), None);
    }
}
