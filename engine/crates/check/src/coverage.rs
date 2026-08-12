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
//! report says so. What it is not is a corpus-scoped *check*, and the reason
//! is worth stating rather than discovering. An instance is what coverage
//! counts, so an instance of this rule over the corpus would read every
//! document and account every one of them as checked. The rule would then
//! make its own finding unreachable.
//!
//! So coverage is the runner's accounting over the instance record, which is
//! where [spec 12](../../../../docs/spec/12-check-layer.md#instances-and-why-coverage-needs-them)
//! puts it: "coverage accounting then comes directly from this, with no added
//! mechanism". It states [`SCOPE`] for a reader and receives no view, so the
//! enforcement question does not arise for it.

use crate::finding::{Finding, Severity};
use crate::instance::{Instance, Outcome as InstanceOutcome};
use crate::scope::Scope;
use headwater_census::census::Census;

pub const RULE: &str = "coverage.document_unchecked";

/// The grain this rule has. See the module comment for why it is stated here
/// rather than derived from a trait: this rule receives no view.
pub const SCOPE: Scope = Scope::corpus();

/// One document, and what this run did about it.
#[derive(Clone, Debug)]
pub struct Document {
    pub path: String,
    /// The census's own word for what became of the file.
    pub class: &'static str,
    /// Instances that read this document. One instance may read two, so the
    /// sum of this field over the corpus is larger than the instance count.
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
    /// Instances created, counted once each however many documents each read.
    pub instances: usize,
    /// Instances accounted to a path the census never walked. Never zero
    /// without meaning it: such an instance is a check reading outside the
    /// denominator, which is the silent pass one level up.
    pub unaccounted: Vec<String>,
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

        for instance in instances {
            for path in instance.paths() {
                let Some(document) = documents.iter_mut().find(|d| d.path == path) else {
                    // A check that read a file the census never walked. It
                    // cannot happen today, because every instance is created
                    // from a census row, and it is recorded rather than
                    // dropped because dropping it is what hides the defect.
                    unaccounted.push(path.to_string());
                    continue;
                };
                document.created += 1;
                match instance.outcome {
                    InstanceOutcome::Skipped(ref reason) => {
                        document.skipped.push((instance.rule, reason.clone()))
                    }
                    _ => document.ran += 1,
                }
            }
        }

        Coverage {
            documents,
            instances: instances.len(),
            unaccounted,
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

    /// Classified documents that at least one instance ran over.
    pub fn checked(&self) -> usize {
        self.documents
            .iter()
            .filter(|d| d.class == "typed" && d.ran > 0)
            .count()
    }

    /// Each skip reason, with the number of instances it covers, in the order
    /// the reasons first appear.
    pub fn skips(&self) -> Vec<(&str, usize)> {
        let mut reasons: Vec<(&str, usize)> = Vec::new();
        for document in &self.documents {
            for (_, reason) in &document.skipped {
                match reasons
                    .iter_mut()
                    .find(|(known, _)| *known == reason.as_str())
                {
                    Some((_, count)) => *count += 1,
                    None => reasons.push((reason.as_str(), 1)),
                }
            }
        }
        reasons
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
                fixable: false,
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
        for (reason, count) in self.skips() {
            let _ = writeln!(out, "  {count:5} skipped: {reason}");
        }
        for path in &self.unaccounted {
            let _ = writeln!(out, "  a check read {path}, which the census never walked");
        }
        out
    }
}
