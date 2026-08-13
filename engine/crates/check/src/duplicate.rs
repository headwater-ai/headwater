// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check at corpus grain: no identifier is claimed twice.
//!
//! # Why this one is a barrier and the other identity rules are not
//!
//! [`crate::identity`] carries the two identifier defects that are facts about
//! one document's own front matter. A document that declares no identifier, and
//! one that writes a sequence where an identifier belongs, are each visible in
//! the file that wrote them, so that rule reports at `Document` grain and its
//! read set is that one file.
//!
//! [`headwater_graph::index::Defect::Duplicate`] is not that. It is a fact
//! about **two** documents, and neither file is defective on its own: each one
//! declares a well-formed identifier that its scheme admits. What is wrong is
//! the pair, and no document holds the pair.
//!
//! That is a statement about the cache before it is a statement about a
//! message. A document-scoped instance reads one file, so its cache key names
//! one file, and a verdict keyed that way survives every edit to the other
//! claimant — which is the edit that settles it. Repair the first document and
//! the second still carries a cached failure. Break the first document and the
//! second still carries a cached pass.
//! [Spec 12](../../../../docs/spec/12-check-layer.md#determinism-concretely)
//! calls a key that omits an input "a correctness bug, not a performance bug",
//! and this is one: the omitted input is the other claimant.
//!
//! # The three grains that could hold it, and the one that does
//!
//! `Neighbourhood` does not reach it. Two documents that claim one identifier
//! are not one relation apart, and in the general case nothing connects them at
//! all: the fixture pair below is a `design_spec` and a `note` on two shelves,
//! and no edge runs between them.
//!
//! `Shelf` is declared in
//! [spec 12](../../../../docs/spec/12-check-layer.md#what-this-leaves-open) and
//! implemented nowhere, and it cannot hold this rule whatever it becomes. An
//! identifier scheme is declared per kind, several kinds sit on one shelf, and
//! one kind sits on several. So two claimants of one identifier need not share
//! a shelf, and a shelf-scoped instance would pass over exactly the collisions
//! that cross one.
//!
//! `Corpus` is what is left, and it is also what is correct rather than merely
//! sufficient. The set that decides this verdict is the set of identifiers the
//! corpus declares, and nothing smaller than the corpus holds it.
//!
//! # What the grain costs, and the thing it does not cost
//!
//! Spec 12 calls corpus-scoped checks the barriers: this instance reads every
//! document, so any change to any document invalidates it, and no incremental
//! test rescues one. That is one instance re-evaluated per edit, and it is the
//! whole of the cost.
//!
//! The cost it was expected to carry is the one it does not. A corpus-scoped
//! instance reads every document, so a coverage report that counted reading
//! would call every document checked and would make
//! `coverage.document_unchecked` unreachable. [`crate::coverage`] states the
//! ruling that keeps both rules alive: coverage counts routing, and this
//! instance is routed to the corpus rather than to any document.
//!
//! # Two findings, one per claimant
//!
//! The index reports one defect, against whichever document comes second in its
//! own order. Path order is not a fact an author can act on, and a single
//! finding against the second file reads as though that file is the wrong one.
//! Neither is: the two documents are symmetric, and the repair is a choice
//! between them.
//!
//! So one collision is two findings, one in each file, carrying one sentence
//! that names both documents. Each author sees it at the line they wrote,
//! neither is told they are at fault, and a reader who opens only one of the
//! two files still learns the other's name.
//!
//! # Severity and fixability
//!
//! Error, on [`crate::identity`]'s ground. A target that names the identifier
//! resolves to two documents, so it names neither, and the loss is invisible in
//! every count a report prints.
//!
//! No fix. To choose which of the two documents keeps the identifier is a
//! judgment, and to mint the replacement is an allocation, which is the part of
//! [spec 2](../../../../docs/spec/02-taxonomy-model.md) that no rule in this
//! engine implements.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{CorpusCheck, CorpusView};
use headwater_graph::index::{Defect, Reported};

pub const RULE: &str = "identifier.claimed_twice";

/// As [`crate::identity`]: a run that produced no phase-A report for this
/// corpus has nothing to read, and it says so rather than passing.
const NO_REPORT: &str = "the view carries no phase-A report for this corpus";

/// The check. It reads no declaration of its own, on [`crate::fragment`]'s
/// terms: no member of the language turns identifier uniqueness on or off, and
/// [spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#identifiers)
/// states it of every corpus.
pub struct Duplicate {
    /// The front-matter key an identifier is read from. The same parameter
    /// [`crate::identity`] and [`crate::identifier`] take, from the same
    /// [`headwater_graph::Config`], so that a repair names the key this engine
    /// actually read rather than the one it assumed.
    facet: String,
}

impl Duplicate {
    pub fn over(facet: &str) -> Self {
        Duplicate {
            facet: facet.to_string(),
        }
    }
}

impl CorpusCheck for Duplicate {
    const RULE: &'static str = self::RULE;
    /// The first edition of this rule.
    const VERSION: u32 = 1;
    /// The build's own report over every document, which is the only thing this
    /// rule reads. A second reading of the corpus would be a second definition
    /// of the defect, and the build's is the one every edge was bound against.
    const NEEDS_PHASE_A: bool = true;

    fn evaluate(&self, view: &CorpusView<'_>) -> Outcome {
        let Some(identity) = view.identity() else {
            return Outcome::Skipped(NO_REPORT.to_string());
        };
        Outcome::failed(
            identity
                .iter()
                .flat_map(|reported| self.findings(reported))
                .collect(),
        )
    }
}

impl Duplicate {
    /// One collision as two findings, and nothing for a defect of one document.
    ///
    /// The two other members of the enum are [`crate::identity`]'s, and they
    /// reach a rule at the grain that reads the one file each is about.
    fn findings(&self, reported: &Reported) -> Vec<Finding> {
        let Defect::Duplicate {
            id,
            other,
            other_span,
        } = &reported.defect
        else {
            return Vec::new();
        };

        // One sentence, in both files. It names both documents, so neither
        // reader is left to work out which other file the collision is with,
        // and it accuses neither, because the pair is what is wrong.
        let message = format!(
            "`{id}` is claimed by {other} and by {}, and an identifier names one document. A \
             target that names it resolves to two, so neither document is reachable by it",
            reported.path
        );
        let remediation = format!(
            "change the `{}` of one of the two documents, and repoint every relation that names \
             `{id}` at the document it meant",
            self.facet
        );

        [(&reported.path, reported.span), (other, *other_span)]
            .into_iter()
            .map(|(path, span)| {
                let (line, column) = at(span);
                Finding {
                    rule: self::RULE,
                    severity: Severity::Error,
                    obligation: None,
                    path: path.clone(),
                    line,
                    column,
                    message: message.clone(),
                    remediation: remediation.clone(),
                    patch: None,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use headwater_yaml::{Position, Span};

    fn span(line: usize) -> Option<Span> {
        Some(Span {
            start: Position {
                line,
                col: 1,
                offset: 0,
            },
            end: Position {
                line,
                col: 3,
                offset: 2,
            },
        })
    }

    fn collision() -> Reported {
        Reported {
            path: "docs/spec/04-twin.md".to_string(),
            span: span(2),
            defect: Defect::Duplicate {
                id: "SPEC-FIX-first".to_string(),
                other: "docs/spec/00-first.md".to_string(),
                other_span: span(3),
            },
        }
    }

    /// One collision is two findings, one in each file, at the line each author
    /// wrote. A rule that reported the second claimant alone would report path
    /// order as the defect.
    #[test]
    fn one_collision_reports_in_both_files_and_names_both() {
        let found = Duplicate::over("id").findings(&collision());
        assert_eq!(found.len(), 2, "{found:#?}");

        let paths: Vec<&str> = found.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(paths, vec!["docs/spec/04-twin.md", "docs/spec/00-first.md"]);
        assert_eq!(found[0].line, 2, "{found:#?}");
        assert_eq!(found[1].line, 3, "{found:#?}");

        for finding in &found {
            assert_eq!(finding.severity, Severity::Error, "{finding:#?}");
            assert!(!finding.fixable(), "{finding:#?}");
            // One sentence, and it names both documents wherever it is read.
            assert!(finding.message.contains("docs/spec/00-first.md"));
            assert!(finding.message.contains("docs/spec/04-twin.md"));
            assert!(finding.message.contains("`SPEC-FIX-first`"));
        }
        assert_eq!(found[0].message, found[1].message);
    }

    /// The repair names the key this engine reads an identifier from, and the
    /// key is a parameter rather than a word written into this rule.
    #[test]
    fn the_repair_names_the_configured_identifier_key() {
        let found = Duplicate::over("identifier").findings(&collision());
        assert!(
            found[0].remediation.contains("change the `identifier` of"),
            "{:#?}",
            found[0]
        );
    }

    /// The two defects of one document are [`crate::identity`]'s, and this rule
    /// returns nothing for either. A rule that reported both would put one
    /// author on two lines for one edit.
    #[test]
    fn a_defect_of_one_document_is_not_this_rule() {
        for defect in [
            Defect::NoIdentifier {
                kind: "design_spec".to_string(),
            },
            Defect::NotAScalar,
        ] {
            let reported = Reported {
                path: "docs/spec/03-anonymous.md".to_string(),
                span: span(2),
                defect,
            };
            assert!(Duplicate::over("id").findings(&reported).is_empty());
        }
    }
}
