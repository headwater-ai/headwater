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
//! # The admitted set is per kind, and a kind that narrows nothing admits all
//!
//! A kind names the values of an enumerated facet it means, under
//! `facets.values`, and the set this rule reads for that kind is the
//! intersection of every narrowing in its inheritance chain with the facet's
//! declared list ([HW-DR-0066](../../../../docs/decisions/0066-a-kind-narrows-the-value-set-of-an-enumerated-facet-and-nothing-else-can.md)).
//! A kind that names none, and stands under no kind that names one, admits the
//! whole declared list, which is what this rule did for every kind before the
//! member existed.
//!
//! The message names the set the kind admits and not the set the facet
//! declares. They are one list wherever nothing narrows, and where something
//! does, a remediation quoting the declaration would send an author to a value
//! their own kind refuses.
//!
//! # The generation step is per kind, and that is what keeps coverage honest
//!
//! A kind that **forbids** an enumerated facet can never carry a value outside
//! its set, so no instance is generated over it. That is a real reading of the
//! taxonomy rather than an optimization, and it matters for a reason one level
//! up.
//!
//! A narrowing is not that reading and must not become one. A narrowed kind
//! carries the facet and admits fewer of its values, so it instantiates exactly
//! as it did before, and a change that made instantiation depend on a declared
//! narrowing would stop counting every kind that declares none — which is every
//! kind of most taxonomies.
//!
//! An instance that could only ever pass still counts its document as checked.
//! A rule that instantiated over every typed document whatever its kind would
//! make [`crate::coverage`]'s finding unreachable: OB-COV-2 asks whether a
//! classified document was routed to any check, and a universal rule answers
//! yes for every corpus before anything is looked at. The same trap
//! [`crate::coverage`] describes for a corpus-scoped check, met from the other
//! side. Spec 13 carries what is left of it.
//!
//! # A list is read item by item
//!
//! A facet whose value is a list holds one value per scalar item, and each item
//! is read against the admitted set, the way `invalid_pair` reads a list
//! (#1542). The probe kind's `moments` and `axes` are lists of this kind
//! (HW-DR-0112, #1873), and before version 3 `moments: [read, bogus]` passed
//! every strict run. Each item outside the set is one finding at that item.
//!
//! # What this check declines to decide
//!
//! A facet whose value is a mapping, and a list item that is not a scalar,
//! produce nothing here. That is a shape defect, the meta-schema owns shape,
//! and a second report of it from the check layer would send an author to two
//! places. The same posture the rest of this crate takes toward a taxonomy it
//! can read but did not validate.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView, ExportTargets};
use crate::shape::Shape;

pub const RULE: &str = "facet.value.not_permitted";

/// One enumerated facet, and the values it admits.
struct Enumerated {
    facet: String,
    values: Vec<String>,
}

/// Every enumerated facet one kind may carry.
struct Admitted {
    kind: String,
    /// In facet declaration order, so two runs report two violations in one
    /// order.
    facets: Vec<Enumerated>,
}

/// The check, generated from the facet declarations.
pub struct Values {
    admitted: Vec<Admitted>,
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
            admitted: shape
                .kinds
                .iter()
                .map(|kind| {
                    let forbidden: Vec<&str> = shape
                        .ancestry(&kind.name)
                        .iter()
                        .flat_map(|step| step.forbid.iter().map(String::as_str))
                        .collect();
                    Admitted {
                        kind: kind.name.clone(),
                        facets: enumerated
                            .iter()
                            .filter(|facet| !forbidden.contains(&facet.name.as_str()))
                            .map(|facet| Enumerated {
                                facet: facet.name.clone(),
                                values: shape
                                    .admitted_values(&kind.name, &facet.name)
                                    .unwrap_or_default()
                                    .into_iter()
                                    .map(str::to_string)
                                    .collect(),
                            })
                            .collect(),
                    }
                })
                .filter(|admitted| !admitted.facets.is_empty())
                .collect(),
        }
    }

    fn admitted_by(&self, kind: &str) -> &[Enumerated] {
        self.admitted
            .iter()
            .find(|admitted| admitted.kind == kind)
            .map(|admitted| admitted.facets.as_slice())
            .unwrap_or_default()
    }
}

impl DocumentCheck for Values {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`]. 2 reads a kind's
    /// `facets.values`, so a warm cache written by 1 holds the verdict of a
    /// rule that admitted the whole declared set on every kind. 3 reads each
    /// scalar item of a list (#1873), so a warm cache written by 2 holds a pass
    /// for a list with an item outside the set.
    const VERSION: u32 = 3;
    /// A declared value set becomes an `enum` over the same members, under a
    /// guard that lets a mapping through and enumerates each scalar item of a
    /// list. The guard is what makes the translation equivalent rather than
    /// stricter, because this check declines a value it cannot read as a
    /// scalar and a bare `enum` would reject one. See the module comment above,
    /// and the differential at
    /// `engine/crates/generate/tests/differential.rs`.
    const EXPORTABLE_AS: ExportTargets = &["jsonschema"];

    fn instantiates(&self, kind: &str) -> bool {
        !self.admitted_by(kind).is_empty()
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let findings = self
            .admitted_by(view.kind())
            .iter()
            .flat_map(|Enumerated { facet, values }| {
                // Absent is the required-facet rule's business, and a value
                // this engine cannot read as a scalar is the meta-schema's. A
                // list is read item by item (#1873).
                let read: Vec<_> = match view.facets().entry(facet) {
                    None => Vec::new(),
                    Some(entry) => {
                        match (entry.value.value.as_scalar(), entry.value.value.as_seq()) {
                            (Some(scalar), _) => vec![(scalar, entry.value.span)],
                            (None, Some(items)) => items
                                .iter()
                                .filter_map(|item| {
                                    item.value.as_scalar().map(|scalar| (scalar, item.span))
                                })
                                .collect(),
                            (None, None) => Vec::new(),
                        }
                    }
                };
                read.into_iter()
                    .filter(|(declared, _)| !values.iter().any(|value| value == &declared.text))
                    .map(|(declared, span)| (facet, values, declared, span))
                    .collect::<Vec<_>>()
            })
            .map(|(facet, values, declared, span)| {
                let (line, column) = at(Some(span));
                Finding {
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
                    patch: None,
                }
            })
            .collect();
        Outcome::failed(findings)
    }
}
