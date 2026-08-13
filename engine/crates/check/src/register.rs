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
//! The engine reads two mechanism prefixes. `check:` names a rule id, and that
//! is the binding a finding travels along. `phase:` names a phase of the engine
//! itself, out of the closed set [`PHASES`] holds: a phase discharges an
//! obligation by doing the thing rather than by reporting a finding about it,
//! so it binds no finding and it still verifies its obligation. A mechanism
//! with any other prefix names something outside this engine — a schedule, a
//! hook, a pipeline. Such a control binds nothing here, and it is not an error:
//! spec 4 declares controls that no check layer runs.
//!
//! # The second prefix is what the two coverage obligations needed
//!
//! `OB-COV-1` and `OB-COV-3` of the base package carried no control, and the
//! file said so in a comment that read them as gaps. Neither is a gap. The
//! census classifies every file under the corpus root and writes an exception
//! line for each one it cannot, and every run prints what it saw, classified,
//! checked and skipped before it prints a finding. Both run on every
//! invocation. What was missing was a way to name a mechanism that discharges
//! an obligation without producing a finding, so the gap was in the mechanism
//! vocabulary rather than in the disposition vocabulary.
//!
//! # The projection, and the one rule it enforces about itself
//!
//! [Spec 4](../../../../docs/spec/04-assurance-model.md#every-obligation-has-exactly-one-disposition)
//! makes the register "generated, never authored": coverage by obligation, the
//! disposition of each, control health and the escaped findings are a
//! projection of the two declarations and of one run over them. [`Projection`]
//! is that view, and [`Projection::render`] is the artifact.
//!
//! Spec 4 admits no silence: "an obligation with no disposition is itself a
//! finding". `verified` follows from a control, and the other two follow from a
//! member of the obligation. So an obligation that no control discharges and
//! that declares neither is the fourth state spec 4 forbids, and so is one that
//! declares a disposition *and* has a control. [`DISPOSITION`] reports both,
//! and [`MECHANISM`] reports spec 4's other sentence about the register: "a
//! control that names a mechanism that the engine does not implement, or a
//! pipeline that does not exist, is a finding".
//!
//! # A control the engine cannot run discharges nothing
//!
//! Spec 4 reads `verified` as "one or more controls discharge it", and a
//! control whose mechanism this engine does not implement discharges nothing:
//! [`MECHANISM`]'s own message says so. An earlier edition still counted such
//! an obligation as verified, so a taxonomy could move its whole register to
//! `15 verified` with a mechanism name that reaches no code. That is the exact
//! claim the register exists to refuse, printed by the register.
//!
//! So [`Disposed::disposition`] reads what this run can run, and the obligation
//! falls to the disposition it states for itself. Where it states none, that is
//! [`Disposition::Undeclared`], and the engine invents neither of the other
//! two: spec 4's `gap` carries an owner and its `unverifiable` carries
//! reasoning, and neither is a value the engine is in a position to supply.
//! [`Disposed::contradicted`] keeps reading the declaration, and its comment
//! says why the two differ.
//!
//! Neither rule creates an instance, and [`crate::coverage`] is the precedent:
//! a rule whose subject is not a document accounts nothing against the census.
//! Their grain is [`Grain::Taxonomy`](crate::Grain::Taxonomy), which is a fifth
//! grain that spec 12's list does not hold.
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

use crate::finding::{Finding, Severity};
use crate::scope::Scope;
use crate::suppression::Inventory;
use headwater_census::shelves::DeclarationError;
use headwater_yaml::{Mapping, Span, Value};

/// The prefix of a mechanism that names a rule of this engine.
const CHECK: &str = "check:";

/// The prefix of a mechanism that names a phase of this engine.
const PHASE: &str = "phase:";

/// The phases a control may name, which is the whole of what `phase:` reaches.
///
/// A phase discharges an obligation by running rather than by reporting. It is
/// a closed set for the reason `check:` is: a control that names a mechanism
/// this engine does not implement is a finding, and a set the engine cannot
/// enumerate cannot produce one.
pub const PHASES: [&str; 2] = ["census.classification", "runner.coverage_report"];

/// Spec 4: an obligation carries exactly one disposition, and this reports the
/// two ways a taxonomy fails that.
pub const DISPOSITION: &str = "obligation.disposition.not_one";

/// Spec 4: "A control that names a mechanism that the engine does not
/// implement, or a pipeline that does not exist, is a finding."
pub const MECHANISM: &str = "control.mechanism.unimplemented";

/// The grain of both rules above. See the module comment: neither reads a
/// document, so neither creates an instance and neither accounts against the
/// census.
pub const SCOPE: Scope = Scope::taxonomy();

/// Which edition of the two rules reached a verdict. Stated here for the
/// reason [`crate::coverage::VERSION`] is: no trait carries it.
pub const VERSION: u32 = 1;

/// The emitter targets these two rules export to, stated here for the reason
/// [`SCOPE`] is. Empty, and not because nothing could say it: these rules are
/// about the taxonomy rather than about a document, and a front-matter schema
/// has no instance to hold them against.
pub const EXPORTABLE_AS: crate::scope::ExportTargets = &[];

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
    /// The disposition this obligation states for itself, and `None` where it
    /// states none. `verified` is not a value here and never will be: it
    /// follows from a control, and a second place to write it is the drift
    /// spec 4 exists to kill.
    pub disposition: Option<Stated>,
    pub span: Span,
}

/// A disposition an obligation states, which is one of the two that no control
/// can supply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stated {
    /// Spec 4: "No control yet. Wanted, and tracked with an owner and, ideally,
    /// a target."
    Gap {
        owner: String,
        target: Option<String>,
    },
    /// Spec 4: "No mechanism can exist. Accepted, with the reasoning recorded."
    Unverifiable { reasoning: String },
}

/// A control, read down to the binding and to what a report needs of it.
#[derive(Clone, Debug)]
pub struct Control {
    pub id: String,
    pub mechanism: String,
    pub discharges: Vec<String>,
    /// Whether a finding of this control blocks a gate, which is spec 12's
    /// sense of the word. Spec 4 wrote one member holding this and the class
    /// below, and control health cannot be read off a member holding two
    /// vocabularies.
    pub posture: Option<String>,
    /// When this control acts: one of spec 4's four control classes.
    pub acts: Option<String>,
    /// What spec 4 asks to be "recorded with the control", and `None` where a
    /// control records nothing.
    pub promotion: Option<Promotion>,
    pub span: Span,
}

/// The promotion record: one key per subsection of spec 4's promotion section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Promotion {
    /// The control is on the promotion path, and these are the criteria that
    /// would finish it.
    Criteria {
        window: String,
        threshold: String,
        sample: String,
    },
    /// "Where promotion does not apply": one error class is unrecoverable, so
    /// the control ships at its final posture.
    FinalPosture { reasoning: String },
    /// "Where promotion cannot finish": the remediation needs judgment,
    /// whatever the measured false-positive rate.
    PermanentlyAdvisory { reasoning: String },
    /// "Promotion measures a rule, and not a producer of facts": there is no
    /// false-positive rate to measure, and the instrument is a fixture set.
    ProducesFacts { reasoning: String },
}

impl Promotion {
    /// The phrase a report prints, in the order a summary counts them.
    pub fn name(&self) -> &'static str {
        match self {
            Promotion::Criteria { .. } => "with criteria",
            Promotion::FinalPosture { .. } => "at a final posture",
            Promotion::PermanentlyAdvisory { .. } => "permanently advisory",
            Promotion::ProducesFacts { .. } => "producing facts rather than findings",
        }
    }
}

/// What a control's mechanism names, as far as this engine can tell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mechanism {
    /// A rule this run carries.
    Rule(String),
    /// A phase of this engine, from [`PHASES`].
    Phase(String),
    /// A `check:` or `phase:` name this engine does not implement. Spec 4 calls
    /// this a finding, and [`MECHANISM`] is that finding.
    Unimplemented(String),
    /// A prefix this engine does not read. Not an error: spec 4 declares
    /// controls that no check layer runs, and the register says only that this
    /// run did not observe it.
    External(String),
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
                    message: format!("`controls` is {}, and it names controls", other.kind_name()),
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

    /// What a control's mechanism names, as far as this engine can tell.
    pub fn mechanism(&self, control: &Control) -> Mechanism {
        if let Some(rule) = control.mechanism.strip_prefix(CHECK) {
            return match crate::RULES.contains(&rule) {
                true => Mechanism::Rule(rule.to_string()),
                false => Mechanism::Unimplemented(control.mechanism.clone()),
            };
        }
        if let Some(phase) = control.mechanism.strip_prefix(PHASE) {
            return match PHASES.contains(&phase) {
                true => Mechanism::Phase(phase.to_string()),
                false => Mechanism::Unimplemented(control.mechanism.clone()),
            };
        }
        Mechanism::External(control.mechanism.clone())
    }
}

/// The disposition of one obligation, which is derived and never declared
/// whole. See [`Disposed`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disposition {
    /// One or more controls discharge it, and this engine can run at least one
    /// of them. A control that names a mechanism the engine does not implement
    /// discharges nothing, and it is a control only on paper: see
    /// [`Disposed::unimplemented`].
    Verified,
    /// Nothing discharges it, and the obligation states a gap.
    Gap,
    /// Nothing discharges it, and the obligation is accepted as unverifiable.
    Unverifiable,
    /// Nothing discharges it and it states nothing. The state spec 4 forbids.
    Undeclared,
}

impl Disposition {
    pub fn name(self) -> &'static str {
        match self {
            Disposition::Verified => "verified",
            Disposition::Gap => "gap",
            Disposition::Unverifiable => "unverifiable",
            Disposition::Undeclared => "with no disposition",
        }
    }
}

/// One obligation and what became of it.
///
/// The disposition is not a field, because storing it beside the two things it
/// is derived from is the second source of truth all over again. It is a
/// function of the controls that discharge this obligation and of what the
/// obligation states for itself, and both are here.
#[derive(Clone, Debug)]
pub struct Disposed {
    pub id: String,
    pub severity: Option<String>,
    /// The controls that name it under `discharges`, in declaration order.
    /// Naming it is not discharging it: see [`Disposed::unimplemented`].
    pub controls: Vec<String>,
    /// The subset of [`Disposed::controls`] whose mechanism is
    /// [`Mechanism::Unimplemented`], in the same order.
    ///
    /// Such a control discharges nothing in this run, and the disposition is
    /// derived from the difference. An engine that later implements the
    /// mechanism moves the obligation to `verified` with no edit to any
    /// declaration, which is the property that makes this a run-time fact
    /// rather than a declaration defect.
    pub unimplemented: Vec<String>,
    pub stated: Option<Stated>,
    /// Findings against this obligation that an author suppressed. A
    /// suppression does not undo a control, so it does not move the
    /// disposition. It is here because a reader of a verified obligation is
    /// owed the count of what escaped under it.
    pub escaped: usize,
    /// Findings against this obligation that the adoption payload holds.
    ///
    /// A separate counter from [`Disposed::escaped`] rather than a shared one.
    /// Spec 4 fixes the precedence over three inventories so that they
    /// partition the escaped findings and no finding is counted twice, and one
    /// counter serving two buckets is that double count with the evidence
    /// thrown away. It does not move the disposition either: declared debt is
    /// not a missing control.
    pub pending: usize,
}

impl Disposed {
    /// Whether a control this engine can run discharges it.
    ///
    /// The count rather than the list, because [`Disposed::unimplemented`] is
    /// a subset of [`Disposed::controls`] by construction.
    pub fn discharged(&self) -> bool {
        self.controls.len() > self.unimplemented.len()
    }

    pub fn disposition(&self) -> Disposition {
        match (self.discharged(), &self.stated) {
            (true, _) => Disposition::Verified,
            (false, Some(Stated::Gap { .. })) => Disposition::Gap,
            (false, Some(Stated::Unverifiable { .. })) => Disposition::Unverifiable,
            (false, None) => Disposition::Undeclared,
        }
    }

    /// Whether this obligation carries two dispositions rather than one: a
    /// control claims to discharge it and it also states a gap or an
    /// acceptance. Spec 4 gives it exactly one, so this is the second thing
    /// [`DISPOSITION`] reports.
    ///
    /// This reads the declaration and not the run, which is the opposite of
    /// [`Disposed::disposition`] above and is deliberate. The defect is that
    /// the taxonomy wrote the binding in two places, and that is true of the
    /// text whether or not this engine implements the mechanism. To read the
    /// run here would hide one declaration defect behind another, and a
    /// corrected mechanism name would then raise a contradiction that was
    /// there all along.
    pub fn contradicted(&self) -> bool {
        !self.controls.is_empty() && self.stated.is_some()
    }

    /// Whether every control that names it is one this engine cannot run, so
    /// nothing discharges it and the controls are the reason.
    fn claimed_only(&self) -> bool {
        !self.controls.is_empty() && !self.discharged()
    }
}

/// One control and what this run can say about it.
#[derive(Clone, Debug)]
pub struct Health {
    pub id: String,
    pub mechanism: Mechanism,
    pub posture: Option<String>,
    pub acts: Option<String>,
    pub discharges: Vec<String>,
    pub promotion: Option<Promotion>,
}

/// The register as spec 4 asks for it: coverage by obligation, the disposition
/// of each, control health, and the findings that escaped under each.
#[derive(Clone, Debug, Default)]
pub struct Projection {
    /// In declaration order, because a report is read by a person.
    pub obligations: Vec<Disposed>,
    pub controls: Vec<Health>,
    /// Rules that reach no obligation, or more than one. Spec 4: "a check that
    /// cannot say which invariant it protects did not earn its place."
    pub unbound: Vec<(&'static str, Bound)>,
}

impl Projection {
    /// The projection of one register over the rules this engine carries.
    ///
    /// It takes no run, because none of this is about a run: the same two
    /// declarations produce the same register whatever the corpus holds. What
    /// a run adds is the escaped count, and [`Projection::escaped_from`] adds
    /// it afterwards for a reason its comment gives.
    pub fn of(register: &Register) -> Self {
        let obligations = register
            .obligations
            .iter()
            .map(|obligation| {
                let naming: Vec<&Control> = register
                    .controls
                    .iter()
                    .filter(|control| control.discharges.contains(&obligation.id))
                    .collect();
                Disposed {
                    id: obligation.id.clone(),
                    severity: obligation.severity.clone(),
                    controls: naming.iter().map(|control| control.id.clone()).collect(),
                    unimplemented: naming
                        .iter()
                        .filter(|control| {
                            matches!(register.mechanism(control), Mechanism::Unimplemented(_))
                        })
                        .map(|control| control.id.clone())
                        .collect(),
                    stated: obligation.disposition.clone(),
                    escaped: 0,
                    pending: 0,
                }
            })
            .collect();
        let controls = register
            .controls
            .iter()
            .map(|control| Health {
                id: control.id.clone(),
                mechanism: register.mechanism(control),
                posture: control.posture.clone(),
                acts: control.acts.clone(),
                discharges: control.discharges.clone(),
                promotion: control.promotion.clone(),
            })
            .collect();
        let unbound = crate::RULES
            .iter()
            .filter_map(|rule| match register.bound(rule) {
                Bound::To(_) => None,
                other => Some((*rule, other)),
            })
            .collect();
        Projection {
            obligations,
            controls,
            unbound,
        }
    }

    /// Account this run's suppressed findings to the obligations they name.
    ///
    /// Separate from [`Projection::of`] because of where in a run each half is
    /// available. The dispositions are known before any check runs, and the
    /// findings of [`Projection::findings`] are among the findings the filter
    /// then reads. The inventory exists only after that filter, so a
    /// constructor that took it would run after the findings it produces.
    /// Attribute pending findings to the obligations their rules serve.
    ///
    /// Counted apart from [`Projection::escaped_from`] rather than into it. The
    /// precedence spec 4 fixes exists so that three inventories partition the
    /// escaped findings, and one counter for two buckets is the double count it
    /// forbids.
    pub fn pending_from(&mut self, register: &Register, ledger: &crate::adoption::Ledger) {
        for (rule, count) in ledger.by_rule() {
            let Bound::To(obligation) = register.bound(rule) else {
                continue;
            };
            if let Some(disposed) = self.obligations.iter_mut().find(|d| d.id == obligation) {
                disposed.pending += count;
            }
        }
    }

    pub fn escaped_from(&mut self, register: &Register, inventory: &Inventory) {
        for (rule, count) in inventory.by_rule() {
            let Bound::To(obligation) = register.bound(rule) else {
                continue;
            };
            if let Some(disposed) = self.obligations.iter_mut().find(|d| d.id == obligation) {
                disposed.escaped += count;
            }
        }
    }

    /// The two findings spec 4 asks the register to make about itself.
    ///
    /// The path is the taxonomy this run read rather than a document, because
    /// neither defect is in the corpus. See [`crate::coverage`] for the
    /// precedent: a rule whose subject is not a document creates no instance
    /// and accounts nothing against the census.
    pub fn findings(&self, source: &str) -> Vec<Finding> {
        let mut findings = Vec::new();
        for obligation in &self.obligations {
            let id = &obligation.id;
            // Why nothing discharges it, which is one of two sentences: no
            // control names it at all, or every control that does names a
            // mechanism this engine cannot run.
            let nothing = match obligation.claimed_only() {
                true => format!(
                    "{} {} a mechanism this engine does not implement, so nothing discharges it",
                    obligation.unimplemented.join(", "),
                    verb(obligation.unimplemented.len(), "names", "name")
                ),
                false => "no control discharges it".to_string(),
            };
            let message = match obligation.disposition() {
                Disposition::Undeclared => Some(format!(
                    "obligation {id} carries no disposition: {nothing}, and it states neither a \
                     gap nor an acceptance"
                )),
                _ if obligation.contradicted() => Some(match obligation.claimed_only() {
                    true => format!(
                        "obligation {id} carries two dispositions: {} claims to discharge it, and \
                         it also states one for itself",
                        obligation.controls.join(", ")
                    ),
                    false => format!(
                        "obligation {id} carries two dispositions: {} discharges it, and it also \
                         states one for itself",
                        obligation.controls.join(", ")
                    ),
                }),
                _ => None,
            };
            let Some(message) = message else { continue };
            findings.push(Finding {
                rule: DISPOSITION,
                // Advisory, like every other rule this engine ships. Spec 4
                // writes "the register is complete by construction, or the
                // build fails", and principle 4 rules that a new check ships
                // advisory whatever it will end at. The control is what
                // promotes it, and CT-REG-1 records that it is advisory today.
                severity: Severity::Warn,
                obligation: None,
                path: source.to_string(),
                line: 0,
                column: 0,
                message,
                // The first clause names the fix that is actually open. Where a
                // control already claims the obligation, "give it a control" is
                // advice the author has taken, and the mechanism is the edit.
                remediation: format!(
                    "{}, or state `disposition: {{gap: {{owner: …}}}}` or `disposition: \
                     {{unverifiable: {{reasoning: …}}}}` on it, and exactly one of the three",
                    match obligation.claimed_only() {
                        true => format!(
                            "name a mechanism this engine implements on {}",
                            obligation.unimplemented.join(", ")
                        ),
                        false => "give the obligation a control that discharges it".to_string(),
                    }
                ),
                patch: None,
            });
        }
        for control in &self.controls {
            let Mechanism::Unimplemented(mechanism) = &control.mechanism else {
                continue;
            };
            findings.push(Finding {
                rule: MECHANISM,
                severity: Severity::Warn,
                obligation: None,
                path: source.to_string(),
                line: 0,
                column: 0,
                message: format!(
                    "control {} names the mechanism {mechanism}, which this engine does not \
                     implement, so nothing discharges {}",
                    control.id,
                    control.discharges.join(", ")
                ),
                remediation: "name a rule this engine carries, or a phase of it, or a mechanism \
                              outside it under a prefix this engine does not read"
                    .to_string(),
                patch: None,
            });
        }
        findings
    }

    /// Obligations at one disposition.
    pub fn at(&self, disposition: Disposition) -> impl Iterator<Item = &Disposed> {
        self.obligations
            .iter()
            .filter(move |o| o.disposition() == disposition)
    }

    /// The severities an obligation of this register carries, in spec 4's own
    /// order, with the count at each and the number of those verified.
    ///
    /// An obligation that declares no severity is counted under `-`, so the
    /// rows sum to the total whatever a source omitted.
    pub fn by_severity(&self) -> Vec<(&str, usize, usize)> {
        let mut rows: Vec<(&str, usize, usize)> = ["high", "medium", "low", "-"]
            .into_iter()
            .map(|severity| (severity, 0, 0))
            .collect();
        for obligation in &self.obligations {
            let severity = obligation.severity.as_deref().unwrap_or("-");
            let row = match rows.iter_mut().find(|(known, _, _)| *known == severity) {
                Some(row) => row,
                // A severity outside spec 4's set. The meta-schema refuses one,
                // and this reader refuses nothing, so it is counted where a
                // reader will see it rather than dropped.
                None => {
                    rows.push((severity, 0, 0));
                    rows.last_mut().expect("just pushed")
                }
            };
            row.1 += 1;
            if obligation.disposition() == Disposition::Verified {
                row.2 += 1;
            }
        }
        rows.retain(|(_, count, _)| *count > 0);
        rows
    }

    /// The register as text: the artifact, and the section of the report.
    ///
    /// One rendering rather than two. A summary that a reader sees and an
    /// artifact that a gate reads would be two views of one projection, and
    /// two views of one fact is the drift this whole module is about.
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        if self.obligations.is_empty() && self.controls.is_empty() {
            return out;
        }
        out.push_str("register\n");
        let counts: Vec<String> = [
            Disposition::Verified,
            Disposition::Gap,
            Disposition::Unverifiable,
            Disposition::Undeclared,
        ]
        .into_iter()
        .map(|disposition| format!("{} {}", self.at(disposition).count(), disposition.name()))
        .collect();
        let _ = writeln!(
            out,
            "  {} obligations: {}",
            self.obligations.len(),
            counts.join(", ")
        );
        for (severity, count, verified) in self.by_severity() {
            let _ = writeln!(out, "  {count:5} {severity}, {verified} verified");
        }

        // The gap list spec 4 asks the coverage report to carry, and the two
        // states beside it. A verified obligation is not listed: the control
        // block below names what discharges each one, and a list of every
        // obligation is a list nobody finishes.
        for obligation in self.obligations.iter().filter(|o| {
            o.disposition() != Disposition::Verified
                || o.contradicted()
                || o.escaped > 0
                || o.pending > 0
        }) {
            let _ = write!(
                out,
                "  {} {}",
                obligation.id,
                obligation.disposition().name()
            );
            // The detail of the disposition the register went with. What an
            // obligation states is in force only where nothing discharges it,
            // so the two arms match the disposition rather than the statement.
            let _ = match (obligation.disposition(), &obligation.stated) {
                (Disposition::Gap, Some(Stated::Gap { owner, target })) => match target {
                    Some(target) => write!(out, ", owner {owner}, target {target}"),
                    None => write!(out, ", owner {owner}, and no target"),
                },
                (Disposition::Unverifiable, Some(Stated::Unverifiable { reasoning })) => {
                    write!(out, ", {reasoning}")
                }
                _ => Ok(()),
            };

            // Why an obligation that a control names is not verified. Without
            // this clause the line reads as an obligation nobody wrote a
            // control for, which is a different defect with a different fix.
            if obligation.claimed_only() {
                let _ = write!(
                    out,
                    ", and {} {} to discharge it under a mechanism this engine does not implement",
                    obligation.unimplemented.join(", "),
                    verb(obligation.unimplemented.len(), "claims", "claim")
                );
            }

            // A control claims it and it states a disposition too, so it
            // carries two. Where the control runs, the statement is reported as
            // the second one rather than spelled out, because the control is
            // what the register went with.
            if obligation.contradicted() {
                out.push_str(match (obligation.claimed_only(), &obligation.stated) {
                    (true, _) => ", which is two dispositions",
                    (false, Some(Stated::Gap { .. })) => {
                        ", and it states a gap as well, which is two dispositions"
                    }
                    (false, _) => {
                        ", and it states an acceptance as well, which is two dispositions"
                    }
                });
            }
            let _ = match obligation.escaped {
                0 => Ok(()),
                1 => write!(out, ", 1 finding escaped under it"),
                escaped => write!(out, ", {escaped} findings escaped under it"),
            };
            // Named apart from the line above, because the two are different
            // claims about the same obligation. One says an author hid a
            // finding. The other says the corpus declared the debt, with an
            // owner and a date.
            let _ = match obligation.pending {
                0 => Ok(()),
                1 => write!(out, ", 1 finding is migration-pending under it"),
                pending => write!(out, ", {pending} findings are migration-pending under it"),
            };
            out.push('\n');
        }

        // Control health. The posture and the class are counted apart because
        // they are two vocabularies, which is the whole reason they are two
        // members.
        let _ = writeln!(out, "  {} controls", self.controls.len());
        for (label, mut values) in [
            (
                "posture",
                self.tally(self.controls.iter().map(|c| c.posture.as_deref())),
            ),
            (
                "acts",
                self.tally(self.controls.iter().map(|c| c.acts.as_deref())),
            ),
        ] {
            values.sort();
            for (value, count) in values {
                let _ = writeln!(out, "  {count:5} {label} {value}");
            }
        }
        for (mechanism, count) in self.mechanisms() {
            let _ = writeln!(out, "  {count:5} {mechanism}");
        }

        // The promotion record. Spec 4 asks that the criteria be recorded with
        // the control, so a control with no record is the row that matters and
        // it is printed first.
        let bare = self
            .controls
            .iter()
            .filter(|control| control.promotion.is_none())
            .count();
        if bare > 0 {
            let _ = writeln!(
                out,
                "  {bare:5} with no promotion record, so nothing states what would promote them"
            );
        }
        for name in [
            "with criteria",
            "at a final posture",
            "permanently advisory",
            "producing facts rather than findings",
        ] {
            let count = self
                .controls
                .iter()
                .filter(|control| control.promotion.as_ref().is_some_and(|p| p.name() == name))
                .count();
            if count > 0 {
                let _ = writeln!(out, "  {count:5} {name}");
            }
        }

        // Spec 4 fixes a precedence over three inventories "so the three
        // inventories partition the escaped findings, and no finding is counted
        // three times". Two of the three exist now. The one that does not is
        // printed as absent rather than left out, because a partition with a
        // silent member is one a reader cannot check.
        //
        // The two numbers come from two counters that no code adds together,
        // which is what makes this line a partition a reader can audit rather
        // than a sentence about one.
        let escaped: usize = self.obligations.iter().map(|o| o.escaped).sum();
        let pending: usize = self.obligations.iter().map(|o| o.pending).sum();
        let _ = writeln!(
            out,
            "  escaped findings, in the precedence spec 4 fixes: no waiver mechanism exists, \
             {pending} migration-pending, {escaped} suppressed"
        );

        // A rule that reaches no obligation. The line is printed when there are
        // none for the reason the suppression inventory prints its one shelf: a
        // reader who has to ask whether the report ran cannot read a silence as
        // an answer.
        match self.unbound.is_empty() {
            true => out.push_str("  every rule this engine carries reaches one obligation\n"),
            false => {
                for (rule, bound) in &self.unbound {
                    let _ = match bound {
                        Bound::Unnamed => {
                            writeln!(out, "  {rule} reaches no obligation, so it names none")
                        }
                        Bound::Several(obligations) => writeln!(
                            out,
                            "  {rule} reaches {}, and a finding names one obligation, so it \
                             names none",
                            obligations.join(", ")
                        ),
                        Bound::To(_) => Ok(()),
                    };
                }
            }
        }
        out
    }

    /// Each mechanism state and the number of controls in it, in a fixed order
    /// so that two reports line up.
    fn mechanisms(&self) -> Vec<(&'static str, usize)> {
        let mut rows = [
            ("name a rule this engine carries", 0),
            ("name a phase this engine runs", 0),
            ("name a mechanism this engine does not implement", 0),
            ("name a mechanism outside this engine", 0),
        ];
        for control in &self.controls {
            rows[match control.mechanism {
                Mechanism::Rule(_) => 0,
                Mechanism::Phase(_) => 1,
                Mechanism::Unimplemented(_) => 2,
                Mechanism::External(_) => 3,
            }]
            .1 += 1;
        }
        rows.into_iter().filter(|(_, count)| *count > 0).collect()
    }

    /// One member of a control, counted by value. A control that declares none
    /// is counted under `-`, for the reason [`Projection::by_severity`] counts
    /// one there.
    fn tally<'a>(&self, values: impl Iterator<Item = Option<&'a str>>) -> Vec<(&'a str, usize)> {
        let mut counts: Vec<(&str, usize)> = Vec::new();
        for value in values {
            let value = value.unwrap_or("-");
            match counts.iter_mut().find(|(known, _)| *known == value) {
                Some((_, count)) => *count += 1,
                None => counts.push((value, 1)),
            }
        }
        counts
    }
}

/// The verb form for a list of that length. A report that says "CT-1, CT-2
/// claims" is a report somebody has to read twice.
fn verb(count: usize, one: &'static str, many: &'static str) -> &'static str {
    match count {
        1 => one,
        _ => many,
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
        disposition: read_disposition(id, map)?,
        span,
    })
}

/// The disposition an obligation states, or `None` where it states none.
///
/// The meta-schema owns the shape and this reads what a report needs, which is
/// the posture of every reader in this crate. One thing is refused here and not
/// there, because it is not a shape: `verified` is a value no obligation may
/// declare. The meta-schema expresses that by omitting the key, and a source
/// that writes it reaches this function as an unknown key rather than as a
/// disposition. Saying so by name is worth one branch, because the author who
/// wrote it is asking a reasonable question and the answer is a rule.
fn read_disposition(id: &str, map: &Mapping) -> Result<Option<Stated>, DeclarationError> {
    let Some(node) = map.get("disposition") else {
        return Ok(None);
    };
    let Some(stated) = node.value.as_map() else {
        return Err(DeclarationError {
            message: format!(
                "the disposition of obligation `{id}` is {}, and it names one of `gap` and \
                 `unverifiable`",
                node.value.kind_name()
            ),
            span: node.span,
        });
    };
    if let Some(gap) = stated.get("gap").and_then(|node| node.value.as_map()) {
        let owner = scalar(gap, "owner").ok_or_else(|| DeclarationError {
            message: format!(
                "the gap of obligation `{id}` names no owner, and spec 4 tracks a gap with one"
            ),
            span: node.span,
        })?;
        return Ok(Some(Stated::Gap {
            owner,
            target: scalar(gap, "target"),
        }));
    }
    if let Some(accepted) = stated
        .get("unverifiable")
        .and_then(|node| node.value.as_map())
    {
        let reasoning = scalar(accepted, "reasoning").ok_or_else(|| DeclarationError {
            message: format!(
                "obligation `{id}` is accepted as unverifiable and records no reasoning, which \
                 is the half of that disposition spec 4 asks for"
            ),
            span: node.span,
        })?;
        return Ok(Some(Stated::Unverifiable { reasoning }));
    }
    Err(DeclarationError {
        message: match stated.get("verified").is_some() {
            true => format!(
                "obligation `{id}` declares itself verified, and no obligation may. Verified \
                 follows from a control that discharges it, and a second place to write the \
                 binding is what the register exists to prevent"
            ),
            false => format!(
                "the disposition of obligation `{id}` names neither `gap` nor `unverifiable`"
            ),
        },
        span: node.span,
    })
}

/// The promotion record of a control, or `None` where it records nothing.
fn read_promotion(id: &str, map: &Mapping) -> Result<Option<Promotion>, DeclarationError> {
    let Some(node) = map.get("promotion") else {
        return Ok(None);
    };
    let Some(record) = node.value.as_map() else {
        return Err(DeclarationError {
            message: format!(
                "the promotion record of control `{id}` is {}, and it names one of `criteria`, \
                 `final_posture`, `permanently_advisory` and `produces_facts`",
                node.value.kind_name()
            ),
            span: node.span,
        });
    };
    if let Some(criteria) = record.get("criteria").and_then(|node| node.value.as_map()) {
        let missing = |member: &str| DeclarationError {
            message: format!(
                "the promotion criteria of control `{id}` state no {member}, and spec 4 names it"
            ),
            span: node.span,
        };
        return Ok(Some(Promotion::Criteria {
            window: scalar(criteria, "window").ok_or_else(|| missing("observation window"))?,
            threshold: scalar(criteria, "threshold").ok_or_else(|| missing("threshold"))?,
            sample: scalar(criteria, "sample").ok_or_else(|| missing("adjudicated sample"))?,
        }));
    }
    for (key, build) in [
        (
            "final_posture",
            (|reasoning| Promotion::FinalPosture { reasoning }) as fn(String) -> Promotion,
        ),
        ("permanently_advisory", |reasoning| {
            Promotion::PermanentlyAdvisory { reasoning }
        }),
        ("produces_facts", |reasoning| Promotion::ProducesFacts {
            reasoning,
        }),
    ] {
        let Some(exemption) = record.get(key).and_then(|node| node.value.as_map()) else {
            continue;
        };
        let reasoning = scalar(exemption, "reasoning").ok_or_else(|| DeclarationError {
            message: format!(
                "control `{id}` is off the promotion path under `{key}` and records no \
                 reasoning, which leaves the assertion of taste that spec 4 refuses"
            ),
            span: node.span,
        })?;
        return Ok(Some(build(reasoning)));
    }
    Err(DeclarationError {
        message: format!(
            "the promotion record of control `{id}` names none of `criteria`, `final_posture`, \
             `permanently_advisory` and `produces_facts`"
        ),
        span: node.span,
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
        posture: scalar(map, "posture"),
        acts: scalar(map, "acts"),
        promotion: read_promotion(id, map)?,
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

    /// What this reader refuses, as one string. Every refusal here is about a
    /// declaration it cannot use rather than about a shape, which the
    /// meta-schema owns.
    fn refusal(source: &str) -> String {
        let root = headwater_yaml::load(source).expect("the source loads");
        Register::read(root.value.as_map().expect("a mapping"))
            .expect_err("the register is refused")
            .iter()
            .map(|error| error.message.clone())
            .collect::<Vec<String>>()
            .join("\n")
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
        assert_eq!(
            register(ONE).bound("shelf.placement_is_primary"),
            Bound::Unnamed
        );
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

    /// The disposition an obligation states, and the one value it may never
    /// state. `verified` follows from a control, so a source that writes it is
    /// asking for a second place to keep the binding.
    #[test]
    fn an_obligation_states_a_gap_or_an_acceptance_and_never_verified() {
        let register = register(
            "obligations:\n  \
             OB-1:\n    statement: s\n    disposition: {gap: {owner: o, target: t}}\n  \
             OB-2:\n    statement: s\n    disposition: {unverifiable: {reasoning: r}}\n",
        );
        assert_eq!(
            register
                .obligation("OB-1")
                .and_then(|o| o.disposition.clone()),
            Some(Stated::Gap {
                owner: "o".to_string(),
                target: Some("t".to_string()),
            })
        );
        assert_eq!(
            register
                .obligation("OB-2")
                .and_then(|o| o.disposition.clone()),
            Some(Stated::Unverifiable {
                reasoning: "r".to_string(),
            })
        );

        let refused =
            refusal("obligations:\n  OB-1:\n    statement: s\n    disposition: {verified: {}}\n");
        assert!(refused.contains("declares itself verified"), "{refused}");
    }

    /// A gap with no owner is not the gap spec 4 describes, and an acceptance
    /// with no reasoning is not the acceptance it describes either.
    #[test]
    fn each_disposition_carries_the_half_spec_4_asks_for() {
        assert!(
            refusal("obligations:\n  OB-1:\n    statement: s\n    disposition: {gap: {}}\n")
                .contains("names no owner")
        );
        assert!(refusal(
            "obligations:\n  OB-1:\n    statement: s\n    disposition: {unverifiable: {}}\n"
        )
        .contains("records no reasoning"));
    }

    /// A phase discharges an obligation by running, and it binds no finding.
    /// Both halves matter: the register calls the obligation verified, and
    /// `bound` still reports that no rule reaches it.
    #[test]
    fn a_phase_discharges_an_obligation_and_binds_no_finding() {
        let register = register(
            "obligations:\n  OB-1:\n    statement: s\n\
             controls:\n  CT-1:\n    mechanism: phase:census.classification\n    \
             discharges: [OB-1]\n",
        );
        assert_eq!(
            register.mechanism(&register.controls[0]),
            Mechanism::Phase("census.classification".to_string())
        );
        assert_eq!(register.bound("census.classification"), Bound::Unnamed);
        let projection = Projection::of(&register);
        assert_eq!(
            projection.obligations[0].disposition(),
            Disposition::Verified
        );
        assert!(projection.findings("t.yml").is_empty());
    }

    /// The two ways a taxonomy gives an obligation something other than one
    /// disposition, and the one way a control names a mechanism that nothing
    /// implements. Spec 4 calls all three findings.
    #[test]
    fn the_register_reports_what_spec_4_calls_a_finding_about_itself() {
        let register = register(
            "obligations:\n  \
             OB-1:\n    statement: s\n  \
             OB-2:\n    statement: s\n    disposition: {gap: {owner: o}}\n  \
             OB-3:\n    statement: s\n\
             controls:\n  \
             CT-1:\n    mechanism: check:coverage.document_unchecked\n    discharges: [OB-2]\n  \
             CT-2:\n    mechanism: check:no.such.rule\n    discharges: [OB-3]\n",
        );
        let projection = Projection::of(&register);
        let findings = projection.findings("t.yml");
        let messages: Vec<&str> = findings.iter().map(|f| f.message.as_str()).collect();
        assert_eq!(findings.len(), 4, "{messages:?}");
        assert!(messages[0].contains("OB-1 carries no disposition"));
        assert!(messages[1].contains("OB-2 carries two dispositions"));
        // Two findings for OB-3, and they say different things. The control is
        // broken, and the obligation it names is consequently undisposed.
        assert!(messages[2].contains("OB-3 carries no disposition"));
        assert!(messages[3].contains("does not implement"));
        assert!(findings.iter().all(|f| f.path == "t.yml"));
    }

    /// A control this engine cannot run discharges nothing, so the obligation
    /// it names is not verified.
    ///
    /// An earlier edition read `verified` off the existence of a control. A
    /// mechanism name that reaches no code then moved an obligation to
    /// verified, while the run reported in the same breath that nothing
    /// discharges it. Both halves are asserted here: the disposition, and the
    /// second finding that the register owes a reader who sees it.
    #[test]
    fn a_control_this_engine_cannot_run_verifies_nothing() {
        let register = register(
            "obligations:\n  OB-1:\n    statement: s\n\
             controls:\n  CT-1:\n    mechanism: check:no.such.rule\n    discharges: [OB-1]\n",
        );
        let projection = Projection::of(&register);
        let disposed = &projection.obligations[0];
        assert_eq!(disposed.controls, vec!["CT-1".to_string()]);
        assert_eq!(disposed.unimplemented, vec!["CT-1".to_string()]);
        assert!(!disposed.discharged());
        assert_eq!(disposed.disposition(), Disposition::Undeclared);

        let messages: Vec<String> = projection
            .findings("t.yml")
            .into_iter()
            .map(|finding| finding.message)
            .collect();
        assert!(
            messages[0].contains(
                "OB-1 carries no disposition: CT-1 names a mechanism this engine does not \
                 implement, so nothing discharges it"
            ),
            "{messages:?}"
        );
        assert!(
            messages[1].contains("control CT-1 names the mechanism"),
            "{messages:?}"
        );
        assert!(
            projection.render().contains("1 obligations: 0 verified"),
            "{}",
            projection.render()
        );
    }

    /// One runnable control among unrunnable ones still discharges it, and the
    /// obligation is verified. The `MECHANISM` finding is what reports the
    /// other control, and a disposition that read the worst of the set would
    /// report a defect the obligation does not have.
    #[test]
    fn one_control_this_engine_runs_is_enough() {
        let register = register(
            "obligations:\n  OB-1:\n    statement: s\n\
             controls:\n  \
             CT-1:\n    mechanism: check:no.such.rule\n    discharges: [OB-1]\n  \
             CT-2:\n    mechanism: check:coverage.document_unchecked\n    discharges: [OB-1]\n",
        );
        let projection = Projection::of(&register);
        assert_eq!(
            projection.obligations[0].disposition(),
            Disposition::Verified
        );
        assert_eq!(projection.findings("t.yml").len(), 1, "the mechanism only");
    }

    /// An obligation that states a gap, and whose only control this engine
    /// cannot run, takes the gap. The gap is now the true reading, and the
    /// contradiction is still reported: the taxonomy wrote the binding twice,
    /// and that is a defect of the text rather than of this run.
    #[test]
    fn a_stated_gap_stands_where_the_control_that_claims_it_cannot_run() {
        let register = register(
            "obligations:\n  OB-1:\n    statement: s\n    disposition: {gap: {owner: o}}\n\
             controls:\n  CT-1:\n    mechanism: check:no.such.rule\n    discharges: [OB-1]\n",
        );
        let projection = Projection::of(&register);
        assert_eq!(projection.obligations[0].disposition(), Disposition::Gap);
        assert!(projection.obligations[0].contradicted());
        let rendered = projection.render();
        assert!(
            rendered.contains(
                "OB-1 gap, owner o, and no target, and CT-1 claims to discharge it under a \
                 mechanism this engine does not implement, which is two dispositions"
            ),
            "{rendered}"
        );
    }

    /// A mechanism under a prefix this engine does not read is not a defect.
    /// Spec 4 declares scheduled and hook mechanisms, and the register says
    /// only that this run did not observe one.
    #[test]
    fn a_mechanism_outside_this_engine_is_not_a_finding() {
        let register = register(
            "obligations:\n  OB-1:\n    statement: s\n\
             controls:\n  CT-1:\n    mechanism: scheduled:staleness-sweep\n    \
             discharges: [OB-1]\n",
        );
        assert!(Projection::of(&register).findings("t.yml").is_empty());
    }

    /// A promotion record holds exactly one of spec 4's four cases, and an
    /// exemption with no reasoning is the assertion of taste it refuses.
    #[test]
    fn a_promotion_record_holds_one_case_and_states_its_reason() {
        let register = register(
            "controls:\n  \
             CT-1:\n    mechanism: check:r\n    promotion: {criteria: \
             {window: w, threshold: t, sample: s}}\n  \
             CT-2:\n    mechanism: check:r\n    promotion: {permanently_advisory: \
             {reasoning: r}}\n",
        );
        assert_eq!(
            register.controls[0].promotion,
            Some(Promotion::Criteria {
                window: "w".to_string(),
                threshold: "t".to_string(),
                sample: "s".to_string(),
            })
        );
        assert_eq!(
            register.controls[1].promotion,
            Some(Promotion::PermanentlyAdvisory {
                reasoning: "r".to_string(),
            })
        );
        assert!(refusal(
            "controls:\n  CT-1:\n    mechanism: check:r\n    promotion: {final_posture: {}}\n"
        )
        .contains("records no reasoning"));
        assert!(refusal(
            "controls:\n  CT-1:\n    mechanism: check:r\n    promotion: {criteria: {window: w}}\n"
        )
        .contains("state no threshold"));
    }

    /// A suppression does not undo a control, so it does not move a
    /// disposition. What it moves is the count a reader of that obligation is
    /// owed.
    #[test]
    fn what_escaped_is_counted_against_the_obligation_and_moves_no_disposition() {
        let register = register(
            "obligations:\n  OB-1:\n    statement: s\n\
             controls:\n  CT-1:\n    mechanism: check:coverage.document_unchecked\n    \
             discharges: [OB-1]\n",
        );
        let mut projection = Projection::of(&register);
        projection.escaped_from(&register, &Inventory::default());
        assert_eq!(projection.obligations[0].escaped, 0);
        assert_eq!(
            projection.obligations[0].disposition(),
            Disposition::Verified
        );
    }
}
