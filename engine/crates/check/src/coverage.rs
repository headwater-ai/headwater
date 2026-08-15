// SPDX-License-Identifier: Apache-2.0
//! Coverage: what this run looked at, computed against the census.
//!
//! [Spec 4](../../../../docs/spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)
//! states three obligations, and this module is where the second and the third
//! land.
//!
//! | | |
//! |---|---|
//! | OB-COV-1 | Every file under the corpus root is classified, or reported as unclassifiable |
//! | OB-COV-2 | Every classified document is routed to at least one check |
//! | OB-COV-3 | Every run reports its coverage: documents seen, classified, checked, and skipped — with reasons |
//!
//! OB-COV-1 is the census and it landed in
//! [#44](https://github.com/headwater-ai/headwater/issues/44). This module reads
//! that census as the **denominator** and never rebuilds one, which is the
//! ordering [spec 12](../../../../docs/spec/12-check-layer.md#two-phases-and-why-the-order-matters)
//! requires: "the coverage report in Phase B is computed against that census,
//! not against the set of documents that classified successfully".
//!
//! # A classified document with no instance is a finding
//!
//! Spec 12 says so in one sentence: "**a document with zero instances is a
//! finding.** It means that a shelf pattern is wrong or that a file is
//! misplaced. Both facts are good to know." That finding is the whole content
//! of OB-COV-2, and it is the reason coverage is a check rather than a
//! statistic printed under one.
//!
//! An **unclassified** document with no instance is not a finding here. It is
//! already a row of the census with its own outcome, and a second report of one
//! fact sends its author to two places.

//! # Corpus grain, and why no view enforces it
//!
//! This rule reads every row of the census, so its grain is `Corpus` and a
//! report says so. What it is not is a corpus-scoped *check*: it is the
//! runner's accounting over the instance record, which is where
//! [spec 12](../../../../docs/spec/12-check-layer.md#instances-and-why-coverage-needs-them)
//! puts it — "coverage accounting then comes directly from this, with no added
//! mechanism". It states [`SCOPE`] for a reader and receives no view, so the
//! enforcement question does not arise for it.
//!
//! # Coverage counts routing, and a corpus-scoped instance routes to nothing
//!
//! An earlier edition of this comment read the paragraph above as a bar on
//! corpus-scoped *checks* in general: an instance of one reads every document,
//! coverage counts what an instance reads, so the first such rule would account
//! every document as checked and make OB-COV-2 unreachable. The reading was
//! right about the arithmetic and wrong about which of the two is the defect.
//!
//! OB-COV-2 is "every classified document is **routed** to at least one
//! check", and routing is the generation step: a template, a declaration, and
//! one instance per target. A document-scoped instance is routed to its
//! document; an edge-scoped one to both endpoints, which is why coverage counts
//! it against each. A corpus-scoped instance has one target and it is the
//! corpus. It reads every document and is routed to none of them, so it is
//! counted against none of them: see [`crate::Grain::routes`], which is where
//! that ruling is written and matched exhaustively, so a new grain has to
//! decide it rather than inherit it.
//!
//! The finding this preserves is the reachable one. `check/spec/03-no-instance.md`
//! in the fixture tree is a classified document that no rule reads, and
//! [`crate::duplicate`] reads it along with every other document that carries
//! front matter. Counting the read would have deleted the failing fixture of
//! this rule while the instance count went up, which reads in every report as
//! an improvement.
//!
//! # A skip is counted once, and the routing is the reason that had to be said
//!
//! [`Coverage::skips`] reads the instance record and [`Document::skipped`]
//! reads the routing, and the two are different populations. An edge-scoped
//! instance is routed to both of its endpoints, so a count taken off the
//! documents holds one skip twice. A corpus-scoped one is routed to no document
//! at all, so a count taken off the documents never sees it. Only the first of
//! those two is comparable with [`Coverage::instances`], which is the number
//! every artifact of this engine prints beside it.
//!
//! The report used to take that count off the documents. On the corpus of this
//! repository it said `4 skipped` of a class that holds **two** instances of an
//! edge-scoped rule, and the cache's own count of what it cannot key — which is
//! a skipped instance or an input with no digest — said 585 where the report's
//! classes summed to 587. Two surfaces counted one population and disagreed by
//! the routing, and neither said which of the two it meant.

use crate::finding::{Finding, Severity};
use crate::instance::{Instance, Outcome as InstanceOutcome};
use crate::scope::Scope;
use headwater_census::census::Census;

pub const RULE: &str = "coverage.document_unchecked";

/// The grain this rule has. See the module comment for why it is stated here
/// rather than derived from a trait: this rule receives no view.
pub const SCOPE: Scope = Scope::corpus(false, false);

/// Which edition of this rule reached a verdict, stated here for the reason
/// [`SCOPE`] is: no trait carries it. It is published in the read set beside
/// every other rule's, because a reader of that artifact is asking which
/// engine produced the result and this rule produces some of it.
pub const VERSION: u32 = 1;

/// The emitter targets this rule exports to, stated here for the reason
/// [`SCOPE`] is. It is empty and it stays empty: this rule reads the whole
/// census, and a validator that holds one document in its hand cannot ask
/// whether another document was checked.
pub const EXPORTABLE_AS: crate::scope::ExportTargets = &[];

/// One document, and what this run did about it.
#[derive(Clone, Debug)]
pub struct Document {
    pub path: String,
    /// The census's own word for what became of the file.
    pub class: &'static str,
    /// Instances routed to this document. One instance may be routed to two,
    /// so the sum of this field over the corpus is larger than the number of
    /// routed instances. An instance that read this document without being
    /// routed to it is not here: see the module comment.
    pub created: usize,
    pub ran: usize,
    /// One entry per skipped instance: the rule, and why it did not run.
    pub skipped: Vec<(&'static str, String)>,
}

/// What this run looked at.
#[derive(Clone, Debug)]
pub struct Coverage {
    /// One entry per census row, in the census's own order.
    pub documents: Vec<Document>,
    /// Instances created, counted once each however many documents each read,
    /// and of every grain. This is the run's own total rather than the routed
    /// subset: a reader asking how much work a run did is asking about all of
    /// it.
    pub instances: usize,
    /// Instances accounted to a path the census never walked. Never zero
    /// without meaning it: such an instance is a check reading outside the
    /// denominator, which is the silent pass one level up.
    pub unaccounted: Vec<String>,
    /// Each reason an instance reached no verdict, with the number of
    /// **instances** it covers, in the order the reasons first appear.
    ///
    /// Read off the instance record and never off [`Document::skipped`], which
    /// holds the same skips routed. An edge-scoped instance is routed to both
    /// its endpoints, so a sum over the documents counts it twice, and a
    /// corpus-scoped one is routed to no document at all, so a sum over the
    /// documents never sees it. Neither of those is a number a reader can hold
    /// against [`Coverage::instances`], and this one is: `instances` less
    /// [`Coverage::skipped`] is what reached a verdict.
    skips: Vec<(String, usize)>,
}

impl Coverage {
    /// Account every instance of a run against the census that fixed the
    /// denominator.
    pub fn of(census: &Census, instances: &[Instance]) -> Self {
        let mut documents: Vec<Document> = census
            .rows
            .iter()
            .map(|row| Document {
                path: row.path.clone(),
                class: row.outcome.class(),
                created: 0,
                ran: 0,
                skipped: Vec::new(),
            })
            .collect();
        let mut unaccounted = Vec::new();
        let mut skips: Vec<(String, usize)> = Vec::new();

        for instance in instances {
            // Once per instance, before the routing loop below and outside it,
            // because this is the count of instances that reached no verdict
            // and the loop below is the count of documents one fell on.
            if let InstanceOutcome::Skipped(ref reason) = instance.outcome {
                match skips.iter_mut().find(|(known, _)| known == reason) {
                    Some((_, count)) => *count += 1,
                    None => skips.push((reason.clone(), 1)),
                }
            }
            for path in instance.paths() {
                let Some(document) = documents.iter_mut().find(|d| d.path == path) else {
                    // Recorded whatever the grain, because this is a statement
                    // about the denominator rather than about coverage: an
                    // instance of any grain that read outside the census read
                    // outside the set every guarantee here is computed over.
                    // A check that read a file the census never walked. It
                    // cannot happen today, because every instance is created
                    // from a census row, and it is recorded rather than
                    // dropped because dropping it is what hides the defect.
                    unaccounted.push(path.to_string());
                    continue;
                };
                // Read, and routed to the corpus rather than to this document.
                // See the module comment: to count it would make the finding
                // below unreachable for every document such a rule reads.
                if !instance.grain.routes() {
                    continue;
                }
                document.created += 1;
                match instance.outcome {
                    InstanceOutcome::Skipped(ref reason) => {
                        document.skipped.push((instance.rule, reason.clone()));
                    }
                    _ => document.ran += 1,
                }
            }
        }

        Coverage {
            documents,
            instances: instances.len(),
            unaccounted,
            skips,
        }
    }

    /// Files under the corpus root. The denominator, and it comes from the
    /// census rather than from anything this phase computed.
    pub fn seen(&self) -> usize {
        self.documents.len()
    }

    /// Documents the census gave a kind.
    pub fn classified(&self) -> usize {
        self.documents.iter().filter(|d| d.class == "typed").count()
    }

    /// Files this engine wrote, which the census reports under its own class.
    ///
    /// Counted and named rather than left inside `seen() - classified()`. A
    /// generated file is never classified and never checked, so it would
    /// otherwise be a gap between two numbers that a reader has to guess the
    /// composition of, and
    /// [spec 4](../../../../docs/spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)
    /// asks OB-COV-3 for the reasons as well as the counts.
    ///
    /// It is not an escape from OB-COV-2. That obligation is over classified
    /// documents, a generated file is not one, and what holds it instead is
    /// `headwater generate --check`: the file has to be the bytes its emitter
    /// produces now, and a marked file that no declaration writes is an error
    /// there.
    pub fn generated(&self) -> usize {
        self.documents
            .iter()
            .filter(|d| d.class == "generated")
            .count()
    }

    /// Classified documents that at least one instance ran over.
    pub fn checked(&self) -> usize {
        self.documents
            .iter()
            .filter(|d| d.class == "typed" && d.ran > 0)
            .count()
    }

    /// Each skip reason, with the number of instances it covers, in the order
    /// the reasons first appear.
    ///
    /// The classes partition the skipped instances, so the counts sum to
    /// [`Coverage::skipped`] and never to anything else.
    pub fn skips(&self) -> &[(String, usize)] {
        &self.skips
    }

    /// Instances that reached no verdict.
    ///
    /// A run states this in every format it writes, because
    /// [spec 4](../../../../docs/spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)
    /// asks OB-COV-3 for the skips as well as the three counts, and an artifact
    /// that carried the instance total alone would report a run that decided
    /// nothing as a run that decided everything.
    pub fn skipped(&self) -> usize {
        self.skips.iter().map(|(_, count)| count).sum()
    }

    /// The coverage rule, as findings.
    ///
    /// The obligation is left unset here, and [`crate::run`] stamps it from the
    /// control that names this rule. This module used to write `OB-COV-2` into
    /// the field, which put the binding in two places: in a package that an
    /// adopter can revise, and in code they cannot.
    pub fn findings(&self) -> Vec<Finding> {
        self.documents
            .iter()
            .filter(|document| document.class == "typed" && document.ran == 0)
            .map(|document| Finding {
                rule: RULE,
                severity: Severity::Warn,
                obligation: None,
                path: document.path.clone(),
                line: 0,
                column: 0,
                message: match document.created {
                    0 => "this document is classified and no check instance was created for it"
                        .to_string(),
                    created => format!(
                        "this document is classified and all {created} of its check \
                         instances were skipped"
                    ),
                },
                remediation: "check that the shelf pattern claims the right files, and that \
                              the kind declares something a check reads"
                    .to_string(),
                patch: None,
            })
            .collect()
    }

    /// Coverage as text.
    ///
    /// The four numbers OB-COV-3 names, then the reasons. The reasons are
    /// printed once each with a count rather than once per instance: a report
    /// that repeats one sentence 14 times is a report nobody finishes.
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(
            out,
            "{} seen, {} classified, {} checked, {} check instances",
            self.seen(),
            self.classified(),
            self.checked(),
            self.instances
        );
        if self.generated() > 0 {
            let _ = writeln!(
                out,
                "  {:5} generated, held to regeneration by `headwater generate --check`",
                self.generated()
            );
        }
        for (reason, count) in self.skips() {
            let _ = writeln!(out, "  {count:5} skipped: {reason}");
        }
        for path in &self.unaccounted {
            let _ = writeln!(out, "  a check read {path}, which the census never walked");
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::Input;
    use crate::scope::Grain;
    use headwater_census::census::{Outcome as Walked, Row};

    fn row(path: &str) -> Row {
        Row {
            path: path.to_string(),
            outcome: Walked::NotADocument,
            document: None,
            digest: None,
        }
    }

    fn input(path: &str) -> Input {
        Input {
            path: path.to_string(),
            digest: None,
        }
    }

    fn over(rows: Vec<Row>, instances: Vec<Instance>) -> Coverage {
        Coverage::of(&Census { rows }, &instances)
    }

    /// One edge-scoped skip is one instance, and the routing holds it twice.
    ///
    /// The two accounts are both here, so the test fails whichever of them the
    /// count is taken from by mistake. No fixture tree of this repository
    /// reaches this state: an edge-scoped rule that skips has to meet a corpus
    /// that declares the relation and an endpoint that is missing the facet, and
    /// [`crate::dependency`] over `HW-REG-open-questions` is the only place it
    /// happens. So the state is built here rather than found.
    #[test]
    fn an_edge_scoped_skip_is_one_instance_and_two_routed_documents() {
        let coverage = over(
            vec![row("source.md"), row("target.md")],
            vec![Instance::skipped(
                "dependency.terminal",
                Grain::Edge,
                vec![input("source.md"), input("target.md")],
                "no state to read at the target end",
            )],
        );
        assert_eq!(coverage.instances, 1);
        assert_eq!(coverage.skipped(), 1, "one instance reached no verdict");
        assert_eq!(
            coverage.skips(),
            &[("no state to read at the target end".to_string(), 1)]
        );
        let routed: usize = coverage
            .documents
            .iter()
            .map(|document| document.skipped.len())
            .sum();
        assert_eq!(routed, 2, "and the routing accounts for it at both ends");
    }

    /// A corpus-scoped skip is routed to no document and is still an instance
    /// that reached no verdict.
    ///
    /// The other direction of the same distinction, and the one a count off the
    /// documents loses altogether rather than doubles.
    #[test]
    fn a_skip_routed_to_no_document_is_still_a_skip() {
        let coverage = over(
            vec![row("source.md")],
            vec![Instance::skipped(
                "coverage.document_unchecked",
                Grain::Corpus,
                vec![input("source.md")],
                "the census carries no row this rule can read",
            )],
        );
        assert_eq!(coverage.skipped(), 1);
        let routed: usize = coverage
            .documents
            .iter()
            .map(|document| document.skipped.len())
            .sum();
        assert_eq!(routed, 0, "and the routing sees none of it");
    }

    /// A run that skipped nothing reports zero rather than reporting nothing.
    #[test]
    fn a_run_that_skipped_nothing_has_a_number_for_it() {
        let coverage = over(
            vec![row("source.md")],
            vec![Instance::of(
                "facet.required.missing",
                Grain::Document,
                vec![input("source.md")],
                InstanceOutcome::Passed,
            )],
        );
        assert_eq!(coverage.skipped(), 0);
        assert!(coverage.skips().is_empty());
    }
}
