// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check, generated from the endpoint sets of a relation.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-five-origins-of-a-check)
//! lists "endpoint kinds" among the Graph-origin examples. A relation declares
//! `from` and `to`, and this rule is the whole of what those two keys mean at
//! run time. `headwater_graph` carries both and reads neither, and says so:
//! "whether the two endpoint kinds are permitted is a Graph check".
//!
//! # This is the rule the `is_a` chain exists for
//!
//! `cites_evidence` permits `evaluation` at its target end, and `traces_to`
//! permits `governed_document`. A document's kind is `design_spec` or
//! `review_record`, never `governed_document`, because
//! [spec 2](../../../../docs/spec/02-taxonomy-model.md#abstract-kinds) says no
//! document is an abstract kind. So an endpoint is permitted when the kind at
//! that end **descends from** a kind the relation names, and a comparison of
//! the two strings would report every inherited endpoint in a corpus as a
//! violation.
//!
//! [#56](https://github.com/headwater-ai/headwater/issues/56) recorded that the
//! chain is the one input the graph does not carry. [`crate::shape`] is where it
//! arrived, and this is the check that needed it.
//!
//! # Either half may be the one that is written, and the ends do not swap
//!
//! An author may write a relation under its own name or under its inverse, and
//! [`headwater_graph::Edge`] carries which. The *source* of an edge is the
//! document that wrote it, which for an inverse half is the document at the
//! relation's **target** end. So the ends are read off the direction and never
//! off which document happened to write the line.
//!
//! # Where the finding reports
//!
//! At the entry that declared the edge, in the document that declared it. That
//! is the line an author is looking at. When both halves exist the declared one
//! is chosen, so a pair that two documents wrote reports once and not twice.
//!
//! # The scope
//!
//! [`EdgeCheck`] is the whole declaration: one relation instance and both
//! endpoints. The kinds at the two ends are what the view carries, and nothing
//! here reads a third document.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{EdgeCheck, EdgeView};
use crate::shape::Shape;
use headwater_graph::declarations::Relation;
use headwater_graph::{Declarations, Direction, Target};

pub const RULE: &str = "relation.endpoint.not_permitted";

/// A pair that carries no half at all, which grouping never produces. Recorded
/// rather than panicked on, because a panic inside a check is a run that
/// reports nothing about the rest of the corpus.
const NO_HALF: &str = "the pair carries no declared half";

/// A pair of a relation that the declarations no longer hold. Unreachable for
/// the same reason and recorded on the same terms: the generation step took the
/// relation from those declarations.
const NO_RELATION: &str = "no declaration holds the relation this pair declares";

/// One end of a declared relation, as a pair reaches it.
struct Endpoint<'a> {
    /// `from` or `to`, which is the word a message uses.
    key: &'static str,
    id: &'a str,
    path: &'a str,
    kind: &'a str,
}

/// The check, generated from the relation declarations and the kind hierarchy.
pub struct Endpoints<'a> {
    /// The relations that declare both endpoint sets, which is the generation
    /// step in full. A relation that names one end and not the other states
    /// nothing this rule can test, and `taxonomy validate` owns the omission.
    declared: Vec<&'a Relation>,
    shape: &'a Shape,
}

impl<'a> Endpoints<'a> {
    pub fn over(declarations: &'a Declarations, shape: &'a Shape) -> Self {
        Endpoints {
            declared: declarations
                .relations
                .iter()
                .filter(|relation| !relation.from.is_empty() && !relation.to.is_empty())
                .collect(),
            shape,
        }
    }

    /// Whether a kind is admitted by one of the endpoint sets.
    ///
    /// An entry that names an anchor kind rather than a document kind matches
    /// nothing here, and correctly so: a document at an end that admits only an
    /// external anchor is an endpoint the relation does not permit.
    fn admits(&self, permitted: &[String], kind: &str) -> bool {
        permitted
            .iter()
            .any(|allowed| self.shape.descends_from(kind, allowed))
    }
}

impl EdgeCheck for Endpoints<'_> {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 1;

    fn instantiates(&self, relation: &str) -> bool {
        self.declared.iter().any(|known| known.name == relation)
    }

    fn evaluate(&self, view: &EdgeView<'_>) -> Outcome {
        // The declared half when a document wrote it, because its span is the
        // line the author of the source document is looking at.
        let Some(half) = view.declared_half().or_else(|| view.inverse_half()) else {
            return Outcome::Skipped(NO_HALF);
        };
        let Some(relation) = self
            .declared
            .iter()
            .find(|known| known.name == half.declared)
        else {
            return Outcome::Skipped(NO_RELATION);
        };

        let Target::Document { id, path, kind } = &half.target else {
            // An anchor, a withheld target or an unbound one. None of the three
            // is a document pair, and `declared_triple` already declines to
            // group them, so this is unreachable rather than tolerated.
            return Outcome::Skipped(NO_RELATION);
        };
        let writer = Endpoint {
            key: "from",
            id: &half.source.id,
            path: &half.source.path,
            kind: &half.source.kind,
        };
        let other = Endpoint {
            key: "to",
            id,
            path,
            kind,
        };
        // The direction says which end each document is at. The document that
        // wrote an inverse half sits at the relation's target end, so the two
        // labels swap and the documents do not move.
        let (source, target) = match half.direction {
            Direction::AsDeclared => (writer, other),
            Direction::Inverse => (
                Endpoint { key: "from", ..other },
                Endpoint { key: "to", ..writer },
            ),
        };

        let (line, column) = at(Some(half.span));
        let findings = [
            (&source, &relation.from),
            (&target, &relation.to),
        ]
        .into_iter()
        .filter(|(end, permitted)| !self.admits(permitted, end.kind))
        .map(|(end, permitted)| Finding {
            rule: self::RULE,
            severity: Severity::Error,
            obligation: None,
            path: half.source.path.clone(),
            line,
            column,
            message: format!(
                "`{}` declares `{}: [{}]`, and `{}` at that end has the kind `{}`",
                relation.name,
                end.key,
                permitted.join(", "),
                end.id,
                end.kind
            ),
            remediation: format!(
                "declare this relation between kinds `{}` permits, or widen its `{}` in the \
                 taxonomy; the document at that end is {}",
                relation.name, end.key, end.path
            ),
            fixable: false,
        })
        .collect();
        Outcome::failed(findings)
    }
}
