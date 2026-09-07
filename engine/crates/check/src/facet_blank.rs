// SPDX-License-Identifier: Apache-2.0
//! A Shape-origin check, generated from the facets a taxonomy declares
//! `type: string`.
//!
//! [`crate::facet_required`] reports a key that is absent, and
//! [`crate::facet_value`] reports a scalar outside a closed set. Neither one
//! reaches the state between them: the key is declared, and it carries no
//! content. That is the state
//! [#541](https://github.com/headwater-ai/headwater/issues/541) measured, where
//! a `title` written as `title:` reached a rendered shelf index as `[~]` at
//! exit 0 through every gate this engine has.
//!
//! Nothing below names a facet or a kind. The population is every facet whose
//! declaration says `type: string` and that the kind does not forbid, so a
//! taxonomy that types one more facet as a string gets one more facet read and
//! this file does not change.
//!
//! # The three families, and why one predicate is not enough
//!
//! The shapes a blank value takes come from the parser rather than from what
//! blank ought to mean, and there are three of them.
//!
//! `title:` with nothing after the colon, `title: ~`, and `title: null` are all
//! **plain null**. [`headwater_yaml::core_schema::as_null`] is the resolver for
//! them, and it is the reason this rule reads that function rather than a
//! predicate of its own: saphyr hands `title:` back as a plain scalar whose
//! text is the literal `~`, so a rule written as `text.trim().is_empty()` reads
//! it as content and lets the whole family through. That family is the one the
//! issue does not know about, and it is the one that renders.
//!
//! `title: ""`, `title: "   "` and a block scalar with no content are **empty
//! text**. The author wrote a string and the string holds nothing. The two
//! families are held apart because the core schema holds them apart, which is
//! also why [`headwater_yaml::value::Scalar`] keeps its style.
//!
//! `title: []` and `title: {}` are **not a scalar at all**, where the
//! declaration says `string`. [`crate::facet_value`] declines that shape and
//! says so, because a value the meta-schema owns is not the check layer's to
//! type. It is reported here for the one facet family this rule already reads,
//! and the general question — whether `type:` binds a document facet value at
//! all — is a wider debt that
//! [spec 13](../../../../docs/spec/13-open-obligations.md) carries.
//!
//! # Where the finding reports, and why it offers no fix
//!
//! At the value, because that is the text to change. No fix is offered:
//! [spec 12](../../../../docs/spec/12-check-layer.md#fixability) makes one
//! mechanical only where there is one correct outcome derivable without
//! judgment, and the value of a blank `title` is a name somebody has to write.
//! Deleting the key is the other legal repair wherever the kind does not
//! require the facet, and the remediation names both, because an engine that
//! picked one would be making the author's choice.
//!
//! # Error, and not a warning
//!
//! Severity and fixability are separate axes here, and
//! [`crate::facet_required`] is the precedent: it offers no patch and carries
//! [`Severity::Error`] anyway. A missing name is an error and a blank one was
//! silent, which is the asymmetry the issue names, and the two states have to
//! land at the same severity or the asymmetry survives the fix.
//!
//! # One overlap, stated rather than removed
//!
//! A facet that declares both `type: string` and a closed value set is read by
//! this rule and by [`crate::facet_value`], and a blank value on one raises two
//! findings. Both are true, both are errors, and they name different repairs:
//! one says the key carries no content, the other names the admitted set. The
//! alternative is a rule whose population depends on a second declaration,
//! which is a narrowing no reader of the taxonomy could predict from it.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView, ExportTargets};
use crate::shape::Shape;

pub const RULE: &str = "facet.value.blank";

/// The type name this rule reads. A facet declaring any other type, or none, is
/// not in the population: the shapes below are what a *string* value may
/// degenerate into, and a blank date or a blank integer is a question about a
/// type this rule does not parse.
const STRING: &str = "string";

/// One string-typed facet a kind may carry, and whether that kind owes it.
struct Typed {
    facet: String,
    /// Whether deleting the key is also a repair. A required facet has one
    /// remedy and an optional one has two, and the remediation says which.
    required: bool,
}

/// Every string-typed facet one kind may carry.
struct Declared {
    kind: String,
    /// In facet declaration order, so two runs report two violations in one
    /// order.
    facets: Vec<Typed>,
}

/// The check, generated from the facet declarations.
pub struct Blank {
    declared: Vec<Declared>,
}

impl Blank {
    /// The generation step, in full.
    pub fn over(shape: &Shape) -> Self {
        let strings: Vec<&crate::shape::Facet> = shape
            .facets
            .iter()
            .filter(|facet| facet.value_type.as_deref() == Some(STRING))
            .collect();
        Blank {
            declared: shape
                .kinds
                .iter()
                .map(|kind| {
                    let forbidden: Vec<&str> = shape
                        .ancestry(&kind.name)
                        .iter()
                        .flat_map(|step| step.forbid.iter().map(String::as_str))
                        .collect();
                    let owed = shape.required_facets(&kind.name);
                    Declared {
                        kind: kind.name.clone(),
                        facets: strings
                            .iter()
                            .filter(|facet| !forbidden.contains(&facet.name.as_str()))
                            .map(|facet| Typed {
                                facet: facet.name.clone(),
                                required: owed.contains(&facet.name),
                            })
                            .collect(),
                    }
                })
                .filter(|declared| !declared.facets.is_empty())
                .collect(),
        }
    }

    fn declared_by(&self, kind: &str) -> &[Typed] {
        self.declared
            .iter()
            .find(|declared| declared.kind == kind)
            .map(|declared| declared.facets.as_slice())
            .unwrap_or_default()
    }
}

impl DocumentCheck for Blank {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 1;
    /// No emitter target. A JSON Schema `minLength` would carry the empty-text
    /// family and nothing of the other two: a schema reading YAML through a
    /// core-schema resolver has already turned `title:` into a null, so the
    /// family this rule exists for is the one the translation loses. A partial
    /// export is what [`crate::partition`] refuses to let a rule claim.
    const EXPORTABLE_AS: ExportTargets = &[];

    /// A kind that may carry no string-typed facet generates no instance, on
    /// the same accounting as [`crate::facet_value`]: an instance that could
    /// only ever pass counts a document as checked by a rule that had nothing
    /// to check.
    fn instantiates(&self, kind: &str) -> bool {
        !self.declared_by(kind).is_empty()
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let findings = self
            .declared_by(view.kind())
            .iter()
            .filter_map(|Typed { facet, required }| {
                let entry = view.facets().entry(facet)?;
                // An absent key is `facet.required.missing`'s finding, and
                // `entry` being `Some` is what says the author wrote one.
                let complaint = match entry.value.value.as_scalar() {
                    None => format!(
                        "`{facet}` is declared as {}, and its declaration says `string`",
                        entry.value.value.kind_name()
                    ),
                    Some(scalar) if headwater_yaml::core_schema::as_null(scalar) => {
                        format!("`{facet}` is declared with no value")
                    }
                    Some(scalar) if scalar.text.trim().is_empty() => {
                        format!("`{facet}` is declared as empty text")
                    }
                    Some(_) => return None,
                };
                let (line, column) = at(Some(entry.value.span));
                Some(Finding {
                    rule: self::RULE,
                    severity: Severity::Error,
                    obligation: None,
                    path: view.path().to_string(),
                    line,
                    column,
                    message: complaint,
                    remediation: match required {
                        true => format!(
                            "write the value of `{facet}` into {}, which this kind requires",
                            view.path()
                        ),
                        false => format!(
                            "write the value of `{facet}` into {}, or remove the key",
                            view.path()
                        ),
                    },
                    patch: None,
                })
            })
            .collect();
        Outcome::failed(findings)
    }
}
