// SPDX-License-Identifier: Apache-2.0
//! A Shape-origin check, generated from the facets a taxonomy declares with a
//! scalar type other than `string`.
//!
//! A facet declaration can state `type: date`, `integer`, `boolean` or
//! `duration`, and until HW-OBL-0171 nothing read a document's value against
//! it. The meta-schema reads a taxonomy and never a document, and
//! [`crate::facet_value`] declines a value that is not a scalar. So
//! `status_since: {from: x}` and `last_verified: 2026-02-30` passed a strict
//! run, and every rule that reads a date then read a date nobody wrote.
//!
//! Nothing below names a facet or a kind. The population is every facet whose
//! declaration states one of those four types and that the kind does not
//! forbid, so a taxonomy that types one more facet gets one more facet read and
//! this file does not change.
//!
//! # One rule, and one predicate per type
//!
//! The owner ruled on #1492 for one rule with a case per declared type, rather
//! than one rule identifier per type name: four identifiers would carry four
//! controls and four rows in each recorded fixture for one predicate table.
//! The predicate is [`headwater_meta::ScalarType::admits`], the one the
//! meta-schema applies to a taxonomy value, so a date that `taxonomy validate`
//! refuses in a taxonomy is a date this rule refuses in a document. That
//! predicate read digit positions only before this rule existed, and admitted
//! `2026-02-30`. It now reads the calendar.
//!
//! `type: string` is not read here. [`crate::facet_blank`] is the string rule,
//! so the two populations do not overlap and one defect gets one finding. A
//! type name that [`headwater_meta::ScalarType::parse`] does not know decides
//! nothing. The meta-schema refuses such a name before it reaches a lock, so
//! the case is a guard and not a population.
//!
//! # What this rule does not read
//!
//! A date after the run is a question about time and not about type, so
//! `last_verified: 2999-01-01` passes here. The probe oracle `future-date`
//! holds that question (`.headwater/probe.yml`, #1835).
//!
//! # Where the finding reports, and why it offers no fix
//!
//! At the value, because that is the text to change. No fix is offered:
//! [spec 12](../../../../docs/spec/check-layer.md#fixability) makes one
//! mechanical only where there is one correct outcome derivable without
//! judgment, and the date a document entered its state is a fact the author
//! knows and the engine does not.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView, ExportTargets};
use crate::shape::Shape;
use headwater_meta::ScalarType;
use headwater_yaml::Value;

pub const RULE: &str = "facet.value.not_of_type";

/// One typed facet a kind may carry, with the type its declaration states.
struct Typed {
    facet: String,
    declared: ScalarType,
}

/// Every typed facet one kind may carry.
struct Declared {
    kind: String,
    /// In facet declaration order, so two runs report two violations in one
    /// order.
    facets: Vec<Typed>,
}

/// The check, generated from the facet declarations.
pub struct NotOfType {
    declared: Vec<Declared>,
}

/// The type a facet declaration states, where this rule reads it.
///
/// `string` is [`crate::facet_blank`]'s, and `null` is a type the meta-schema
/// gives to `extends:` and no facet. A name `parse` does not know gives `None`.
fn read_by_this_rule(value_type: Option<&str>) -> Option<ScalarType> {
    match ScalarType::parse(value_type?)? {
        ScalarType::String | ScalarType::Null => None,
        declared => Some(declared),
    }
}

/// What is wrong with `value` on `facet`, or `None` where it is of the type.
fn complaint(facet: &str, value: &Value, declared: ScalarType) -> Option<String> {
    let name = declared.name();
    let Some(scalar) = value.as_scalar() else {
        return Some(format!(
            "`{facet}` is declared as {}, and its declaration says `{name}`",
            value.kind_name()
        ));
    };
    if headwater_yaml::core_schema::as_null(scalar) {
        return Some(format!(
            "`{facet}` is declared with no value, and its declaration says `{name}`"
        ));
    }
    match declared.admits(scalar) {
        true => None,
        false => Some(format!(
            "`{facet}` is `{}`, and its declaration says `{name}`",
            scalar.text
        )),
    }
}

/// How a value of each type is written, for the remediation.
fn form(declared: ScalarType) -> &'static str {
    match declared {
        ScalarType::Date => "a calendar date written `YYYY-MM-DD`",
        ScalarType::Integer => "an integer",
        ScalarType::Boolean => "`true` or `false`",
        ScalarType::Duration => "a count and one unit of `d`, `w`, `m` and `y`, as `90d`",
        ScalarType::String | ScalarType::Null => "a value of its declared type",
    }
}

impl NotOfType {
    /// The generation step, in full.
    pub fn over(shape: &Shape) -> Self {
        let typed: Vec<(&crate::shape::Facet, ScalarType)> = shape
            .facets
            .iter()
            .filter_map(|facet| {
                read_by_this_rule(facet.value_type.as_deref()).map(|declared| (facet, declared))
            })
            .collect();
        NotOfType {
            declared: shape
                .kinds
                .iter()
                .map(|kind| {
                    let forbidden: Vec<&str> = shape
                        .ancestry(&kind.name)
                        .iter()
                        .flat_map(|step| step.forbid.iter().map(String::as_str))
                        .collect();
                    Declared {
                        kind: kind.name.clone(),
                        facets: typed
                            .iter()
                            .filter(|(facet, _)| !forbidden.contains(&facet.name.as_str()))
                            .map(|(facet, declared)| Typed {
                                facet: facet.name.clone(),
                                declared: *declared,
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

impl DocumentCheck for NotOfType {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
    ///
    /// Raise the rule version when a rule's behavior changes.
    const VERSION: u32 = 1;
    /// No emitter target. A JSON Schema `format: date` reads text, and not the
    /// YAML core-schema scalars that `integer` and `boolean` are read as here,
    /// so a translation would carry part of this rule and drop the rest. A
    /// partial export is what [`crate::partition`] refuses to let a rule claim.
    const EXPORTABLE_AS: ExportTargets = &[];

    /// A kind that may carry no typed facet generates no instance, on the same
    /// accounting as [`crate::facet_blank`]: an instance that could only ever
    /// pass counts a document as checked by a rule that had nothing to check.
    fn instantiates(&self, kind: &str) -> bool {
        !self.declared_by(kind).is_empty()
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let findings = self
            .declared_by(view.kind())
            .iter()
            .filter_map(|Typed { facet, declared }| {
                // An absent key is `facet.required.missing`'s finding, and
                // `entry` being `Some` is what says the author wrote one.
                let entry = view.facets().entry(facet)?;
                let message = complaint(facet, &entry.value.value, *declared)?;
                let (line, column) = at(Some(entry.value.span));
                Some(Finding {
                    rule: self::RULE,
                    severity: Severity::Error,
                    obligation: None,
                    path: view.path().to_string(),
                    line,
                    column,
                    message,
                    remediation: format!(
                        "write `{facet}` in {} as {}",
                        view.path(),
                        form(*declared)
                    ),
                    patch: None,
                })
            })
            .collect();
        Outcome::failed(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The value of `f` in a one-line YAML document.
    fn value(line: &str) -> Value {
        headwater_yaml::load(line)
            .expect("the line loads")
            .value
            .as_map()
            .expect("a mapping")
            .get("f")
            .expect("the key is there")
            .value
            .clone()
    }

    fn refused(line: &str, declared: ScalarType) -> Option<String> {
        complaint("f", &value(line), declared)
    }

    /// The population: `string` is the blank rule's, `null` is no facet type,
    /// and an unknown name decides nothing.
    #[test]
    fn the_rule_reads_four_type_names_and_no_other() {
        for name in ["date", "integer", "boolean", "duration"] {
            assert!(read_by_this_rule(Some(name)).is_some(), "{name} is read");
        }
        for name in [Some("string"), Some("null"), Some("timestamp"), None] {
            assert!(read_by_this_rule(name).is_none(), "{name:?} is read");
        }
    }

    /// The generation step: a kind that forbids its only typed facet, and a
    /// taxonomy that types nothing but strings, give no instance. An instance
    /// that could only pass would count a document as checked by a rule that
    /// had nothing to check.
    #[test]
    fn a_kind_with_no_typed_facet_generates_no_instance() {
        let source = "\
facets:
  title: {type: string}
  since: {type: date}
  rank: {type: integer}
kinds:
  record: {}
  note: {facets: {forbid: [since, rank]}}
  memo: {facets: {forbid: [since]}}
";
        let root = headwater_yaml::load(source)
            .expect("the taxonomy loads")
            .value
            .as_map()
            .expect("a mapping")
            .clone();
        let shape = Shape::read(&root).expect("the shape reads");
        let check = NotOfType::over(&shape);
        assert!(check.instantiates("record"));
        assert!(check.instantiates("memo"));
        assert!(!check.instantiates("note"), "a forbidden facet is read");
        let facets = |kind: &str| -> Vec<(String, &'static str)> {
            check
                .declared_by(kind)
                .iter()
                .map(|typed| (typed.facet.clone(), typed.declared.name()))
                .collect()
        };
        assert_eq!(
            facets("record"),
            vec![("since".to_owned(), "date"), ("rank".to_owned(), "integer")]
        );
        assert_eq!(facets("memo"), vec![("rank".to_owned(), "integer")]);

        let strings =
            headwater_yaml::load("facets:\n  title: {type: string}\nkinds:\n  record: {}\n")
                .expect("the taxonomy loads")
                .value
                .as_map()
                .expect("a mapping")
                .clone();
        let shape = Shape::read(&strings).expect("the shape reads");
        assert!(!NotOfType::over(&shape).instantiates("record"));
    }

    #[test]
    fn an_integer_facet_admits_a_core_schema_integer_and_nothing_else() {
        for line in ["f: 3", "f: -12", "f: 0x1F", "f: 0o17"] {
            assert_eq!(refused(line, ScalarType::Integer), None, "{line}");
        }
        for line in ["f: three", "f: 3.5", "f: \"3\"", "f: true"] {
            assert_eq!(
                refused(line, ScalarType::Integer),
                Some(format!(
                    "`f` is `{}`, and its declaration says `integer`",
                    line.trim_start_matches("f: ").trim_matches('"')
                )),
                "{line}"
            );
        }
    }

    #[test]
    fn a_boolean_facet_admits_a_core_schema_boolean_and_nothing_else() {
        for line in ["f: true", "f: false", "f: True", "f: FALSE"] {
            assert_eq!(refused(line, ScalarType::Boolean), None, "{line}");
        }
        for line in ["f: yes", "f: no", "f: 1", "f: \"true\""] {
            assert!(refused(line, ScalarType::Boolean).is_some(), "{line}");
        }
    }

    #[test]
    fn a_duration_facet_admits_a_count_and_one_unit() {
        for line in ["f: 90d", "f: 2w", "f: 6m", "f: 1y"] {
            assert_eq!(refused(line, ScalarType::Duration), None, "{line}");
        }
        for line in ["f: 90", "f: d", "f: 90h", "f: 9.5d", "f: -3d"] {
            assert!(refused(line, ScalarType::Duration).is_some(), "{line}");
        }
    }

    #[test]
    fn a_date_facet_admits_a_calendar_date_and_nothing_else() {
        // `2000-02-29` holds the 400-year arm of the leap rule.
        for line in [
            "f: 2024-02-29",
            "f: 2000-02-29",
            "f: 2026-12-31",
            "f: \"2026-01-01\"",
        ] {
            assert_eq!(refused(line, ScalarType::Date), None, "{line}");
        }
        for line in [
            "f: 2026-02-30",
            "f: 0000-00-00",
            "f: 2025-02-29",
            // A century year is not a leap year unless 400 divides it.
            "f: 1900-02-29",
            "f: 2100-02-29",
            // Day zero and month zero, each beside a valid other field, so
            // each bound is held by its own case.
            "f: 2026-02-00",
            "f: 2026-00-10",
            "f: 2026-13-01",
            "f: 2026-01-32",
            "f: 2026-8-12",
            "f: 99999-99-99",
        ] {
            assert!(refused(line, ScalarType::Date).is_some(), "{line}");
        }
    }

    /// A value that is not a scalar, and a null, are named as what they are on
    /// every type, rather than as the text the parser hands back.
    #[test]
    fn a_shape_that_is_no_value_is_named_for_every_type() {
        for declared in [
            ScalarType::Date,
            ScalarType::Integer,
            ScalarType::Boolean,
            ScalarType::Duration,
        ] {
            let name = declared.name();
            assert_eq!(
                refused("f: {a: 1}", declared),
                Some(format!(
                    "`f` is declared as a mapping, and its declaration says `{name}`"
                ))
            );
            assert_eq!(
                refused("f: [1]", declared),
                Some(format!(
                    "`f` is declared as a sequence, and its declaration says `{name}`"
                ))
            );
            for line in ["f:", "f: ~", "f: null"] {
                assert_eq!(
                    refused(line, declared),
                    Some(format!(
                        "`f` is declared with no value, and its declaration says `{name}`"
                    )),
                    "{line}"
                );
            }
        }
    }
}
