// SPDX-License-Identifier: Apache-2.0
//! `explain`: why the engine typed a document as it did, and what follows.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#kind-resolution):
//! "`headwater explain <path>` prints this derivation — which shelf matched,
//! which rule fired, and the kind's declared purpose. It also prints which
//! facets and sections are consequently required, and which relations are
//! permitted. Classification is never a black box, for a human or an agent."
//! That sentence is the contents of [`Explanation`], member for member.
//!
//! # The derivation is recorded rather than recomputed
//!
//! The census already resolved the kind, and it kept the steps: which shelf
//! matched, which shelves it beat, whether the placement carried the kind or a
//! discriminator decided it. This prints those steps. A second resolution here
//! could answer differently from the one the run used, and an explanation that
//! does not explain the run is worse than none.
//!
//! # A document the engine did not type still explains
//!
//! An untyped document is the case a reader most needs explained, and the
//! census records its derivation too: the shelf that matched and the step that
//! stopped. [`Surface::explain`] answers for it in the same shape, with no kind
//! and so no requirements. A verb that answered only for documents that already
//! worked would be a verb nobody needs.

use crate::{Neighbour, Surface};
use headwater_census::census::Outcome;

/// One document, explained.
#[derive(Clone, Debug)]
pub struct Explanation {
    pub path: String,
    pub id: Option<String>,
    /// The kind, and `None` for a document the census did not type.
    pub kind: Option<String>,
    /// The derivation, one step per line, as the census recorded it.
    pub derivation: Vec<String>,
    /// The purpose the kind serves, with the intent the taxonomy declares for
    /// it. A reader who opens the document then knows what it is *for*.
    pub purpose: Option<(String, Option<String>)>,
    pub summary: Option<String>,
    /// The warrant, printed always rather than only when it is `asserted`. A
    /// pointer is a glance and this is the answer to a question somebody asked.
    pub warrant: Option<String>,
    /// The facets the kind requires, inherited along the `is_a` chain.
    pub facets: Vec<String>,
    /// The sections it requires, on the same terms.
    pub sections: Vec<String>,
    /// The relations this kind may declare, each with the kinds it may reach.
    pub permitted: Vec<Permitted>,
    /// What this document is already related to, with the cue at each edge.
    pub related: Vec<Neighbour>,
}

/// One relation a kind may declare, and what it may reach.
#[derive(Clone, Debug)]
pub struct Permitted {
    pub relation: String,
    pub to: Vec<String>,
    /// The name the other end writes, where the relation declares one.
    pub inverse: Option<String>,
}

impl Surface<'_> {
    /// Explain the document at a path, or the one an identifier names.
    pub fn explain(&self, target: &str) -> Option<Explanation> {
        // A typed document, through the reader every other read uses.
        if let Some(document) = self.find(target) {
            let permitted = self.permitted(document.kind);
            return Some(Explanation {
                path: document.path.to_string(),
                id: document.id.map(str::to_string),
                kind: Some(document.kind.to_string()),
                derivation: document
                    .derivation
                    .map(|derivation| lines(&derivation.explain()))
                    .unwrap_or_default(),
                purpose: self
                    .shape()
                    .purpose_of(document.kind)
                    .map(|purpose| (purpose.name.clone(), purpose.intent.clone())),
                summary: self.summary(&document),
                warrant: self.warrant(&document),
                facets: self.shape().required_facets(document.kind),
                sections: self.shape().required_sections(document.kind),
                permitted,
                related: self.related(&document),
            });
        }

        // An untyped row of the census, which carries its own derivation.
        let row = self.census.rows.iter().find(|row| row.path == target)?;
        let derivation = match &row.outcome {
            Outcome::Untyped(headwater_census::census::Untyped::Unresolved {
                derivation, ..
            }) => lines(&derivation.explain()),
            other => vec![other.detail()],
        };
        Some(Explanation {
            path: row.path.clone(),
            id: None,
            kind: None,
            derivation,
            purpose: None,
            summary: None,
            warrant: None,
            facets: Vec::new(),
            sections: Vec::new(),
            permitted: Vec::new(),
            related: Vec::new(),
        })
    }

    /// The relations a kind may declare.
    ///
    /// Through the `is_a` chain, because a relation that names an abstract
    /// parent at its source end is declarable by every kind under it. A list
    /// that compared the two names directly would tell a `design_spec` that it
    /// may declare nothing, while the endpoint check permits it everything the
    /// parent has.
    pub fn permitted(&self, kind: &str) -> Vec<Permitted> {
        self.relations()
            .relations
            .iter()
            .filter(|relation| {
                relation
                    .from
                    .iter()
                    .any(|end| self.shape().descends_from(kind, end))
            })
            .map(|relation| Permitted {
                relation: relation.name.clone(),
                to: relation.to.clone(),
                inverse: relation.inverse.clone(),
            })
            .collect()
    }
}

fn lines(text: &str) -> Vec<String> {
    text.lines().map(str::to_string).collect()
}

impl Explanation {
    /// The explanation as text, in spec 2's own order.
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(out, "{}", self.path);
        if let Some(id) = &self.id {
            let _ = writeln!(out, "  {id}");
        }
        match &self.kind {
            Some(kind) => {
                let _ = writeln!(out, "  kind {kind}");
            }
            None => out.push_str("  no kind, so nothing is required of it\n"),
        }
        for step in &self.derivation {
            out.push_str(&headwater_check::filled(step, 4));
        }
        if let Some((name, intent)) = &self.purpose {
            let _ = match intent {
                Some(intent) => out.push_str(&headwater_check::filled(&format!("purpose {name}, to {intent}"), 2)),
                None => out.push_str(&headwater_check::filled(&format!("purpose {name}"), 2)),
            };
        }
        if let Some(summary) = &self.summary {
            out.push_str(&headwater_check::filled(&format!("summary {summary}"), 2));
        }
        if let Some(warrant) = &self.warrant {
            out.push_str(&headwater_check::filled(&format!("warrant {warrant}"), 2));
        }
        if !self.facets.is_empty() {
            out.push_str(&headwater_check::filled(&format!("requires the facets {}", self.facets.join(", ")), 2));
        }
        if !self.sections.is_empty() {
            out.push_str(&headwater_check::filled(&format!("requires the sections {}", self.sections.join(", ")), 2));
        }
        for permitted in &self.permitted {
            let line = match &permitted.inverse {
                Some(inverse) => format!("may declare {} to {}, and the other end writes {inverse}", permitted.relation, permitted.to.join(", ")),
                None => format!("may declare {} to {}", permitted.relation, permitted.to.join(", ")),
            };
            out.push_str(&headwater_check::filled(&line, 2));
        }
        for neighbour in &self.related {
            out.push_str(&headwater_check::filled(&neighbour.render(), 2));
        }
        out
    }
}
