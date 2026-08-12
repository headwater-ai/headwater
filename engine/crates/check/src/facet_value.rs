// SPDX-License-Identifier: Apache-2.0
//! A Shape-origin check, generated from the value set a facet declares.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-five-origins-of-a-check)
//! lists "enum membership" second among the Shape-origin examples. Nothing
//! below names a facet or a value. The instance set is every typed document,
//! and the rule reads the declared set of every facet the document wrote.
//!
//! A vocabulary and a plain enumeration are one thing here. The lock resolves
//! `values: $vocabularies.lifecycle_state` into the list it names, so a check
//! never learns which spelling an author used.
//!
//! # Where the finding reports, and why it offers no fix
//!
//! At the value, because that is the text to change. The remediation names the
//! whole admitted set rather than one member of it: which value is correct is a
//! statement about the document that only its author can make, so no fix is
//! mechanical ([spec 12](../../../../docs/spec/12-check-layer.md#fixability)).
//!
//! # The generation step is per kind, and that is what keeps coverage honest
//!
//! A kind that **forbids** an enumerated facet can never carry a value outside
//! its set, so no instance is generated over it. That is a real reading of the
//! taxonomy rather than an optimization, and it matters for a reason one level
//! up.
//!
//! An instance that could only ever pass still counts its document as checked.
//! A rule that instantiated over every typed document whatever its kind would
//! make [`crate::coverage`]'s finding unreachable: OB-COV-2 asks whether a
//! classified document was routed to any check, and a universal rule answers
//! yes for every corpus before anything is looked at. The same trap
//! [`crate::coverage`] describes for a corpus-scoped check, met from the other
//! side. Spec 13 carries what is left of it.
//!
//! # What this check declines to decide
//!
//! A facet whose value is a mapping or a list where the taxonomy declared a set
//! of scalars produces nothing here. That is a shape defect, the meta-schema
//! owns shape, and a second report of it from the check layer would send an
//! author to two places. The same posture the rest of this crate takes toward a
//! taxonomy it can read but did not validate.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView};
use crate::shape::Shape;

pub const RULE: &str = "facet.value.not_permitted";

/// The check, generated from the facet declarations.
pub struct Values {
    /// Each kind, and every enumerated facet a document of it may carry, with
    /// the set. In facet declaration order, so two runs report two violations
    /// in one order.
    by_kind: Vec<(String, Vec<(String, Vec<String>)>)>,
}

impl Values {
    /// The generation step, in full.
    pub fn over(shape: &Shape) -> Self {
        let enumerated: Vec<&crate::shape::Facet> = shape
            .facets
            .iter()
            .filter(|facet| !facet.values.is_empty())
            .collect();
        Values {
            by_kind: shape
                .kinds
                .iter()
                .map(|kind| {
                    let forbidden: Vec<&str> = shape
                        .ancestry(&kind.name)
                        .iter()
                        .flat_map(|step| step.forbid.iter().map(String::as_str))
                        .collect();
                    (
                        kind.name.clone(),
                        enumerated
                            .iter()
                            .filter(|facet| !forbidden.contains(&facet.name.as_str()))
                            .map(|facet| (facet.name.clone(), facet.values.clone()))
                            .collect::<Vec<_>>(),
                    )
                })
                .filter(|(_, facets)| !facets.is_empty())
                .collect(),
        }
    }

    fn admitted_by(&self, kind: &str) -> &[(String, Vec<String>)] {
        self.by_kind
            .iter()
            .find(|(known, _)| known == kind)
            .map(|(_, facets)| facets.as_slice())
            .unwrap_or_default()
    }
}

impl DocumentCheck for Values {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 1;

    fn instantiates(&self, kind: &str) -> bool {
        !self.admitted_by(kind).is_empty()
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let findings = self
            .admitted_by(view.kind())
            .iter()
            .filter_map(|(facet, values)| {
                let entry = view.facets().entry(facet)?;
                // Absent is the required-facet rule's business, and a value
                // this engine cannot read as a scalar is the meta-schema's.
                let declared = entry.value.value.as_scalar()?;
                if values.iter().any(|value| value == &declared.text) {
                    return None;
                }
                let (line, column) = at(Some(entry.value.span));
                Some(Finding {
                    rule: self::RULE,
                    severity: Severity::Error,
                    obligation: None,
                    path: view.path().to_string(),
                    line,
                    column,
                    message: format!(
                        "`{facet}` admits {}, and this document declares `{}`",
                        values.join(", "),
                        declared.text
                    ),
                    remediation: format!(
                        "change `{facet}` in {} to one of: {}",
                        view.path(),
                        values.join(", ")
                    ),
                    fixable: false,
                })
            })
            .collect();
        Outcome::failed(findings)
    }
}
