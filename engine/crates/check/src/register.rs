// SPDX-License-Identifier: Apache-2.0
//! The two declarations the check layer reads: `obligations` and `controls`.
//!
//! The same posture as [`headwater_census::shelves`] and
//! [`headwater_graph::declarations`], for the same reason. Nothing here
//! validates a taxonomy. The meta-schema owns shape, `taxonomy validate` owns
//! referential integrity, and this module reads what a finding needs and
//! refuses only what it cannot use.
//!
//! # What a finding needs, and where the binding lives
//!
//! [Spec 4](../../../../docs/spec/04-assurance-model.md#findings): "Every
//! finding names the obligation that it serves." An obligation is data, and so
//! is the binding. A control names a mechanism and the obligations that
//! mechanism discharges, and the base package writes
//! `mechanism: check:relation.reciprocity.missing`. So the path from a rule to
//! its obligation runs through the control that names the rule, and this module
//! is that path.
//!
//! The engine reads one mechanism prefix. `check:` names a rule id, and a
//! mechanism with any other prefix names something that is not a rule — a
//! schedule, a hook, a pipeline. Such a control binds nothing here, and it is
//! not an error: spec 4 declares controls that no check layer runs.
//!
//! # Where this differs from spec 12, and it is worth stating
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-plugin-interface)
//! writes `obligation() -> ObligationId` as a method that a check implements.
//! That would be a second place where the binding lives, and spec 4 rules on
//! the general form of that mistake: "Two sources of truth for one binding is
//! exactly the drift that this system exists to kill." A check that named its
//! own obligation could name one that no register holds, and an adopter could
//! not rebind it without an edit to code they do not own. So the check declares
//! its id, the taxonomy binds the id to an obligation, and the runner stamps
//! the finding.
//! [13 — Open obligations](../../../../docs/spec/13-open-obligations.md)
//! carries the disagreement.

use headwater_census::shelves::DeclarationError;
use headwater_yaml::{Mapping, Span, Value};

/// The prefix of a mechanism that names a rule of this engine.
const CHECK: &str = "check:";

/// The obligations and controls of a resolved taxonomy.
#[derive(Clone, Debug, Default)]
pub struct Register {
    /// In declaration order, because a report is read by a person.
    pub obligations: Vec<Obligation>,
    pub controls: Vec<Control>,
}

/// An obligation, read down to what a report needs.
///
/// The severity here is the obligation's, and it is not the severity a finding
/// carries. Spec 12 rules that severity is the check's, and spec 4's own
/// example carries `error` on a finding against an obligation that carries
/// `medium`. The two scales measure different things: how much the invariant
/// matters, and how loudly one breach of it is reported.
#[derive(Clone, Debug)]
pub struct Obligation {
    pub id: String,
    pub statement: String,
    pub severity: Option<String>,
    pub span: Span,
}

/// A control, read down to the binding.
#[derive(Clone, Debug)]
pub struct Control {
    pub id: String,
    pub mechanism: String,
    pub discharges: Vec<String>,
    pub span: Span,
}

/// What a rule reaches through the controls that name it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Bound {
    /// One obligation, which is the one a finding names.
    To(String),
    /// No control names this rule, so a finding from it names no obligation.
    /// Spec 4 calls such a rule one that did not earn its place, and the run
    /// says so rather than inventing an identifier no register would recognize.
    Unnamed,
    /// The controls that name this rule reach more than one obligation. Spec 4
    /// gives a finding one field, so the engine binds none and names them all.
    Several(Vec<String>),
}

impl Register {
    /// Read `obligations` and `controls` from the root of a resolved taxonomy.
    ///
    /// Both are optional at this layer. A taxonomy that declares neither is a
    /// corpus whose findings name no obligation, which is a true report of a
    /// package that has not declared any.
    pub fn read(root: &Mapping) -> Result<Self, Vec<DeclarationError>> {
        let mut errors = Vec::new();
        let mut register = Register::default();

        if let Some(obligations) = root.get("obligations") {
            match &obligations.value {
                Value::Map(map) => {
                    for entry in map {
                        match read_obligation(&entry.key.value, &entry.value.value, entry.key.span)
                        {
                            Ok(obligation) => register.obligations.push(obligation),
                            Err(error) => errors.push(error),
                        }
                    }
                }
                other => errors.push(DeclarationError {
                    message: format!(
                        "`obligations` is {}, and it names obligations",
                        other.kind_name()
                    ),
                    span: obligations.span,
                }),
            }
        }

        if let Some(controls) = root.get("controls") {
            match &controls.value {
                Value::Map(map) => {
                    for entry in map {
                        match read_control(&entry.key.value, &entry.value.value, entry.key.span) {
                            Ok(control) => register.controls.push(control),
                            Err(error) => errors.push(error),
                        }
                    }
                }
                other => errors.push(DeclarationError {
                    message: format!(
                        "`controls` is {}, and it names controls",
                        other.kind_name()
                    ),
                    span: controls.span,
                }),
            }
        }

        if errors.is_empty() {
            Ok(register)
        } else {
            Err(errors)
        }
    }

    /// The obligation a rule serves, through the controls that name it.
    pub fn bound(&self, rule: &str) -> Bound {
        let mut named: Vec<String> = Vec::new();
        for control in &self.controls {
            let Some(mechanism) = control.mechanism.strip_prefix(CHECK) else {
                continue;
            };
            if mechanism != rule {
                continue;
            }
            for obligation in &control.discharges {
                if !named.contains(obligation) {
                    named.push(obligation.clone());
                }
            }
        }
        match named.len() {
            0 => Bound::Unnamed,
            1 => Bound::To(named.remove(0)),
            _ => Bound::Several(named),
        }
    }

    pub fn obligation(&self, id: &str) -> Option<&Obligation> {
        self.obligations.iter().find(|entry| entry.id == id)
    }
}

fn read_obligation(id: &str, value: &Value, span: Span) -> Result<Obligation, DeclarationError> {
    let map = value.as_map().ok_or_else(|| DeclarationError {
        message: format!(
            "obligation `{id}` is {}, and an obligation is a mapping",
            value.kind_name()
        ),
        span,
    })?;
    let statement = scalar(map, "statement").ok_or_else(|| DeclarationError {
        message: format!("obligation `{id}` states no invariant, so nothing can serve it"),
        span,
    })?;
    Ok(Obligation {
        id: id.to_string(),
        statement,
        severity: scalar(map, "severity"),
        span,
    })
}

fn read_control(id: &str, value: &Value, span: Span) -> Result<Control, DeclarationError> {
    let map = value.as_map().ok_or_else(|| DeclarationError {
        message: format!(
            "control `{id}` is {}, and a control is a mapping",
            value.kind_name()
        ),
        span,
    })?;
    let mechanism = scalar(map, "mechanism").ok_or_else(|| DeclarationError {
        message: format!("control `{id}` names no mechanism, so nothing discharges through it"),
        span,
    })?;
    // A control that discharges nothing is a control that binds nothing, and
    // the resolver reports it. Reading it as an empty list keeps one report of
    // that fact rather than two.
    let discharges = map
        .get("discharges")
        .and_then(|node| node.value.as_seq())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.value.as_scalar().map(|scalar| scalar.text.clone()))
                .collect()
        })
        .unwrap_or_default();
    Ok(Control {
        id: id.to_string(),
        mechanism,
        discharges,
        span,
    })
}

fn scalar(map: &Mapping, key: &str) -> Option<String> {
    map.get(key)
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn register(source: &str) -> Register {
        let root = headwater_yaml::load(source).expect("the source loads");
        Register::read(root.value.as_map().expect("a mapping")).expect("the register reads")
    }

    const ONE: &str = "\
obligations:
  OB-REL-1:
    statement: A relation that requires reciprocity is declared at both ends
    severity: medium
controls:
  CT-REL-1:
    mechanism: check:relation.reciprocity.missing
    discharges: [OB-REL-1]
";

    #[test]
    fn a_rule_reaches_its_obligation_through_the_control_that_names_it() {
        let register = register(ONE);
        assert_eq!(
            register.bound("relation.reciprocity.missing"),
            Bound::To("OB-REL-1".to_string())
        );
        assert_eq!(
            register.obligation("OB-REL-1").map(|o| o.severity.clone()),
            Some(Some("medium".to_string()))
        );
    }

    /// A rule that no control names is the gap spec 4 describes, and it reads
    /// as one rather than as a rule bound to nothing in particular.
    #[test]
    fn a_rule_that_no_control_names_is_unnamed() {
        assert_eq!(register(ONE).bound("shelf.placement_is_primary"), Bound::Unnamed);
    }

    /// A mechanism that names no rule of this engine binds nothing, and it is
    /// not an error: spec 4 declares scheduled and hook mechanisms too.
    #[test]
    fn a_mechanism_that_is_not_a_check_binds_nothing() {
        let register = register(
            "controls:\n  CT-1:\n    mechanism: scheduled:staleness-sweep\n    discharges: [OB-3]\n",
        );
        assert_eq!(register.bound("staleness-sweep"), Bound::Unnamed);
    }

    /// Spec 4 gives a finding one obligation field. Two are not one, so the
    /// engine names both and binds neither, which is what it does when two
    /// anchor kinds claim one string.
    #[test]
    fn two_obligations_on_one_rule_bind_neither() {
        let register = register(
            "controls:\n  CT-1:\n    mechanism: check:r\n    discharges: [OB-1]\n  \
             CT-2:\n    mechanism: check:r\n    discharges: [OB-2]\n",
        );
        assert_eq!(
            register.bound("r"),
            Bound::Several(vec!["OB-1".to_string(), "OB-2".to_string()])
        );
    }
}
