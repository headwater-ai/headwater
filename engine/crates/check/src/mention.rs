// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check at corpus grain: a document whose prose names another
//! document's identifier and declares no relation to it is offered the
//! relations the taxonomy declares for the pair (#1977).
//!
//! # The edge the author meant
//!
//! "This follows from HW-DR-0089" in the body of an obligation is a citation a
//! reader follows, and it is not an edge: spec 1 says no syntax promotes prose
//! into a relation. When the front matter declares no `traces_to`, the graph
//! never learns that the obligation rests on the decision, and nothing that
//! reads the graph sees it. The phase B forensics of #1881 found three of four
//! `add_document` failures were this. A rule cannot tell which relation the
//! author meant, or whether the mention was an aside. It can tell the author
//! that the taxonomy offers one, which is what Q4's advisory check on prose
//! links promised (see [`headwater_graph::links`]).
//!
//! # This rule computes nothing it does not have to
//!
//! [`headwater_graph::mentions::bind`] reads the author's own prose, finds each
//! token that is exactly an identifier of another document, and keeps the
//! first mention of each pair with the kinds of both ends. This rule reads
//! that list, the edges and the relation declarations, and nothing else.
//!
//! # When it is silent
//!
//! - **An edge joins the pair**, in either direction and on either half: the
//!   source wrote it, or the named document wrote it, or either wrote the
//!   inverse. An edge whose target did not bind still names the identifier as
//!   written, and it silences the pair too, because its defect is
//!   `relation.target.unresolved`'s news. A missing reciprocal half is
//!   `relation.reciprocity.missing`'s, and not this rule's.
//! - **No relation admits the pair.** A candidate is a relation whose `from`
//!   admits the source's kind and whose `to` admits the named document's kind,
//!   through the `is_a` chain as [`crate::endpoint`] reads it, in the forward
//!   direction only. With no candidate there is nothing to offer.
//!   [HW-OBL-0055](../../../../docs/obligations/0055-the-tradition-has-no-relation-for-the-link-that-this-corpus.md)
//!   records that the tradition has no relation for this corpus's most common
//!   citation, one specification part naming another. A taxonomy that declares
//!   none for a pair gets no note on it. A taxonomy whose `traces_to` or
//!   `supersedes` admits every governed document at both ends, as
//!   `headwater/standard` does, offers those two for every typed pair, and the
//!   note names them so that the author can see how little the offer says.
//! - **Either end has no kind.** A kind is what a relation's ends are made of.
//! - **The source is generated.** `headwater generate` writes its prose and its
//!   front matter, so nobody can declare the relation in it, and the remedy
//!   this rule names would be overwritten by the next run.
//!
//! A `relations:` block that names a relation the taxonomy does not declare
//! makes no edge, so a misspelled `trace_to` leaves the pair unjoined and
//! this note stands beside the error that names the misspelling.
//!
//! # Why the corpus grain
//!
//! [`crate::link_identifier`]'s argument: the verdict depends on the named
//! document's identifier and kind and on an edge the named document can
//! write, and none of those is a byte of the citing document. A
//! document-scoped key would keep a pass that a renumbering or an inverse
//! half removed elsewhere made false.
//!
//! # Severity, and why there is no patch
//!
//! Info. Whether a mention is a dependency is a judgment only the author can
//! make, so the control is permanently advisory, and on this repository's own
//! corpus the population is large enough that a warning would bury the rest
//! of the report. No patch, because which relation is meant, if any, is the
//! author's call.

use crate::finding::{Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{CorpusCheck, CorpusView};
use crate::shape::Shape;
use headwater_graph::declarations::Relation;
use headwater_graph::edges::{Edge, Target};
use headwater_graph::mentions::Mention;
use headwater_graph::Declarations;
use std::collections::HashSet;

pub const RULE: &str = "relation.mention.undeclared";

/// [`crate::link_path`]'s `NO_LINKS`, for the mentions.
const NO_MENTIONS: &str = "the view carries no bound prose mentions for this corpus";

/// The view carried the mentions and not the edges, which the scope never
/// does. Recorded rather than read as "no edge", because that reading would
/// report every mention of the corpus.
const NO_EDGES: &str = "the view carries no edges for this corpus";

/// The check, generated from the relation declarations and the kind hierarchy.
pub struct Mentioned<'a> {
    /// The relations that declare both endpoint sets, in declaration order,
    /// on [`crate::endpoint::Endpoints`]'s terms.
    declared: Vec<&'a Relation>,
    shape: &'a Shape,
}

impl<'a> Mentioned<'a> {
    pub fn over(declarations: &'a Declarations, shape: &'a Shape) -> Self {
        Mentioned {
            declared: declarations
                .relations
                .iter()
                .filter(|relation| !relation.from.is_empty() && !relation.to.is_empty())
                .collect(),
            shape,
        }
    }

    fn admits(&self, permitted: &[String], kind: &str) -> bool {
        permitted
            .iter()
            .any(|allowed| self.shape.descends_from(kind, allowed))
    }

    /// The relations the source could declare toward the named document, in
    /// declaration order.
    fn candidates(&self, mention: &Mention) -> Vec<&'a str> {
        let (Some(from), Some(to)) = (&mention.source_kind, &mention.named_kind) else {
            return Vec::new();
        };
        self.declared
            .iter()
            .filter(|relation| self.admits(&relation.from, from) && self.admits(&relation.to, to))
            .map(|relation| relation.name.as_str())
            .collect()
    }
}

impl CorpusCheck for Mentioned<'_> {
    const RULE: &'static str = self::RULE;
    /// The first edition of this rule.
    ///
    /// Raise the rule version when a rule's behavior changes.
    const VERSION: u32 = 1;
    const NEEDS_MENTIONS: bool = true;
    const NEEDS_GRAPH: bool = true;

    fn evaluate(&self, view: &CorpusView<'_>) -> Outcome {
        let Some(mentions) = view.mentions() else {
            return Outcome::Skipped(NO_MENTIONS.to_string());
        };
        let Some(edges) = view.edges() else {
            return Outcome::Skipped(NO_EDGES.to_string());
        };
        let joined = joined(edges);
        // A generated document's prose is the generator's, so its author has
        // no front matter to declare a relation in.
        let generated: HashSet<&str> = view
            .generated()
            .unwrap_or_default()
            .iter()
            .map(|document| document.path)
            .collect();
        Outcome::failed(
            mentions
                .iter()
                .filter(|mention| !generated.contains(mention.source_path.as_str()))
                .filter(|mention| !joined.contains(&pair(&mention.source_id, &mention.named)))
                .filter_map(|mention| finding(mention, &self.candidates(mention)))
                .collect(),
        )
    }
}

/// An unordered pair of identifiers, so that an edge in either direction is
/// one key.
fn pair<'a>(a: &'a str, b: &'a str) -> (&'a str, &'a str) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

/// Every pair of identifiers some edge joins: the document that wrote it and
/// each identifier its target names, as bound and as written.
fn joined(edges: &[Edge]) -> HashSet<(&str, &str)> {
    let mut out = HashSet::new();
    for edge in edges {
        let source = edge.source.id.as_str();
        if let Target::Document { id, .. } = &edge.target {
            out.insert(pair(source, id));
        }
        for written in &edge.raw_targets {
            out.insert(pair(source, written.trim()));
        }
    }
    out
}

/// One note for a mention the graph holds no edge for, when the taxonomy
/// offers at least one relation.
fn finding(mention: &Mention, candidates: &[&str]) -> Option<Finding> {
    let first = candidates.first()?;
    let named = &mention.named;
    let offered = candidates
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ");
    let (relation, these) = match candidates.len() {
        1 => ("relation", "it"),
        _ => ("relations", "one of them"),
    };
    Some(Finding {
        rule: self::RULE,
        severity: Severity::Info,
        obligation: None,
        path: mention.source_path.clone(),
        line: mention.span.start.line,
        column: mention.span.start.col,
        message: format!(
            "the prose names `{named}` and the front matter declares no relation to it; \
             the taxonomy offers the {relation} {offered} from `{from}` to `{to}`",
            from = mention.source_kind.as_deref().unwrap_or_default(),
            to = mention.named_kind.as_deref().unwrap_or_default(),
        ),
        remediation: format!(
            "if this document rests on `{named}`, declare {these} under `relations:`, \
             for example `{first}: [{named}]`; if the mention is an aside, leave it. \
             `headwater explain {named}` names the document, and `headwater explain` on \
             a relation says what it means"
        ),
        // See the module comment: which relation is meant is the author's call.
        patch: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use headwater_yaml::{Position, Span};

    fn mention(source_kind: Option<&str>, named_kind: Option<&str>) -> Mention {
        Mention {
            source_path: "obligations/a.md".to_string(),
            source_id: "OBL-1".to_string(),
            source_kind: source_kind.map(str::to_string),
            named: "DEC-1".to_string(),
            named_path: "decisions/b.md".to_string(),
            named_kind: named_kind.map(str::to_string),
            span: Span::new(Position::new(9, 19, 0), Position::new(9, 24, 5)),
            times: 1,
        }
    }

    #[test]
    fn a_finding_names_every_candidate_in_order_and_carries_no_patch() {
        let found = finding(
            &mention(Some("obligation"), Some("decision")),
            &["traces_to", "cites"],
        )
        .expect("a finding");
        assert_eq!(found.severity, Severity::Info);
        assert_eq!((found.line, found.column), (9, 19));
        assert!(
            found
                .message
                .contains("relations `traces_to`, `cites` from `obligation` to `decision`"),
            "{found:#?}"
        );
        assert!(
            found.remediation.contains("`traces_to: [DEC-1]`"),
            "{found:#?}"
        );
        assert!(!found.fixable(), "{found:#?}");
    }

    #[test]
    fn no_candidate_is_no_finding() {
        assert!(finding(&mention(Some("decision"), Some("obligation")), &[]).is_none());
    }

    #[test]
    fn a_pair_is_unordered() {
        assert_eq!(pair("OBL-1", "DEC-1"), pair("DEC-1", "OBL-1"));
    }
}
