// SPDX-License-Identifier: Apache-2.0
//! What a run would be, what it would cost, and whether it happens.
//!
//! # The plan is the half of the run identity that exists before the run
//!
//! [Spec 5](../../../../docs/spec/05-ai-integration.md#a-run-produces-a-snapshot-and-a-document)
//! lists nine members of a run identity: the model with its served version, the
//! corpus tree hash, the taxonomy lock hash, the probe selection hash, the
//! rotation seed, the harness version, the arm, and the time. Five of them are
//! decided before anything is called, and this is where they are decided. The
//! model, the served version and the time belong to the run, and the recorder
//! writes them into the transcript.
//!
//! [`crate::intake`] holds a transcript to both halves: the five here have to be
//! the five the plan fixed, or the transcript records a run of something else.
//!
//! # The seed is the caller's, and a derived one would be worse
//!
//! Spec 5 asks for deterministic rotation, so the seed is part of the run
//! identity. It would be easy to derive one from the lock and the selection,
//! and that would be a defect: the rotation would then change whenever the
//! corpus changed, and a difference between two runs would carry a change in
//! the phrasing and a change in the corpus at once with no way to tell them
//! apart. So the seed is a number the caller states, it is recorded as stated,
//! and a run that repeats a seed repeats a selection.
//!
//! # A refusal stops the run and never the probe
//!
//! Every [`Refusal`] below ends the whole run. The alternative is a selection
//! that quietly dropped what it could not read, which reports a rate over a
//! denominator nobody declared — the failure `headwater conformance` already
//! refuses when a rule has no reading.

use crate::budget::{Budgets, Envelope};
use crate::{dollars, Arm, Category, Cents, Expectation, Tier};
use headwater_census::census::{Census, Outcome};
use headwater_graph::edges::Target;
use headwater_graph::{Config, Graph};

/// The kind a probe document is, and the relation that names what it examines.
pub const KIND: &str = "probe";
/// The relation whose targets an `opened`, `not_opened` or `cited` expectation
/// is a predicate over.
pub const EXAMINES: &str = "examines";

/// One probe, as the harness meets it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selected {
    pub path: String,
    pub id: String,
    pub category: Category,
    pub expectation: Expectation,
    /// The identifiers the `examines` edges of this probe name, sorted.
    pub examines: Vec<String>,
    /// The rule a `patched` expectation grades against, and `None` for every
    /// other form.
    pub oracle: Option<String>,
}

/// Why a run does not happen.
///
/// It is a value on the plan rather than an error, because the verb over a plan
/// exits zero whatever it holds. A refusal is a statement about a run that a
/// reader acts on, and never a status a gate branches on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// No document of the probe kind is classified in this corpus.
    NoProbes,
    /// The selection filters left nothing.
    SelectionEmpty { category: Option<Category> },
    /// The declaration names no envelope for the tier asked for.
    TierUndeclared(Tier),
    /// A campaign narrowed to one arm. It would estimate nothing while
    /// carrying the name a published claim cites.
    CampaignNarrowed,
    /// A probe carries no identifier, so nothing can name its result.
    Unnameable { path: String },
    /// A probe's category or expectation is missing or outside its closed set.
    Undeclared {
        path: String,
        facet: &'static str,
        found: Option<String>,
    },
    /// A `patched` probe that names no oracle.
    OracleUnnamed { probe: String },
    /// A `patched` probe naming a rule this engine does not carry. The grader
    /// would have nothing to run, and a probe that grades against nothing
    /// returns a verdict about the harness.
    OracleUnknown { probe: String, rule: String },
    /// An `opened`, `not_opened` or `cited` probe that names no document.
    /// `opened` over an empty set is never satisfied and `not_opened` over one
    /// always is, so either would report the declaration rather than the
    /// corpus.
    ExpectationNamesNothing {
        probe: String,
        expectation: Expectation,
    },
    /// The projected cost is above the tier's ceiling. Spec 5: a run that does
    /// not happen is the cheaper error.
    OverBudget {
        sessions: u64,
        projected: Cents,
        budget: Cents,
    },
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refusal::NoProbes => write!(
                f,
                "this corpus classifies no document of the `{KIND}` kind, so there is nothing to \
                 run and no claim it could measure"
            ),
            Refusal::SelectionEmpty { category } => match category {
                Some(category) => write!(
                    f,
                    "no probe of this corpus declares the `{}` category",
                    category.name()
                ),
                None => write!(f, "the selection is empty"),
            },
            Refusal::TierUndeclared(tier) => write!(
                f,
                "`{}` declares no envelope for the `{}` tier, so no ceiling exists to project \
                 against",
                crate::budget::PATH,
                tier.name()
            ),
            Refusal::CampaignNarrowed => write!(
                f,
                "a campaign narrowed to one arm estimates no difference, and the pair is what a \
                 published efficacy claim rests on"
            ),
            Refusal::Unnameable { path } => write!(
                f,
                "the probe at {path} carries no identifier, so no result could name the probe it \
                 graded"
            ),
            Refusal::Undeclared { path, facet, found } => match found {
                Some(found) => write!(
                    f,
                    "the probe at {path} declares `{facet}: {found}`, which is outside the closed \
                     set the taxonomy fixes"
                ),
                None => write!(f, "the probe at {path} declares no `{facet}`"),
            },
            Refusal::OracleUnnamed { probe } => write!(
                f,
                "{probe} expects `patched` and names no oracle. The `patched` form grades a \
                 produced patch against a named check, so a probe without one declares a \
                 predicate nothing can evaluate"
            ),
            Refusal::OracleUnknown { probe, rule } => write!(
                f,
                "{probe} grades against `{rule}`, which is not one of the {} rules this engine \
                 carries",
                headwater_check::RULES.len()
            ),
            Refusal::ExpectationNamesNothing { probe, expectation } => write!(
                f,
                "{probe} expects `{}` and names no document with `{EXAMINES}`. That predicate over \
                 an empty set reports the declaration rather than the corpus",
                expectation.name()
            ),
            Refusal::OverBudget {
                sessions,
                projected,
                budget,
            } => write!(
                f,
                "{sessions} sessions project {} against a declared ceiling of {}, so the run does \
                 not start",
                dollars(*projected),
                dollars(*budget)
            ),
        }
    }
}

/// A run that this engine has decided everything about except the calling.
#[derive(Clone, Debug)]
pub struct Plan {
    pub tier: Tier,
    pub arms: Vec<Arm>,
    pub repetitions: u32,
    /// The digest of the lock this plan was taken against.
    pub lock: String,
    /// The digest over every classified document of this corpus, by path and
    /// content. It is the corpus tree hash of the run identity, and it moves
    /// when any document moves, which is what makes a recorded result stale.
    pub tree: String,
    /// The digest over the identifiers selected, in order.
    pub selection: String,
    /// The rotation seed, as the caller stated it.
    pub seed: u64,
    /// The version of this engine, which is the harness version.
    pub harness: &'static str,
    pub selected: Vec<Selected>,
    /// Classified documents in the whole corpus, which is what the tree digest
    /// covers.
    pub corpus: usize,
    pub sessions: u64,
    pub projected: Cents,
    pub budget: Cents,
    pub session_cost: Cents,
    pub refusal: Option<Refusal>,
}

/// What the caller narrowed the run to.
#[derive(Clone, Debug, Default)]
pub struct Narrowing {
    pub category: Option<Category>,
    pub arm: Option<Arm>,
    pub seed: u64,
}

impl Plan {
    /// Take a plan over a corpus.
    ///
    /// It never fails. A run that does not happen produces a plan with a
    /// [`Refusal`] on it and the numbers that led to it, because the reason a
    /// run was refused is the thing the caller needs.
    pub fn over(
        census: &Census,
        graph: &Graph,
        config: &Config,
        budgets: &Budgets,
        lock: &str,
        tier: Tier,
        narrowing: &Narrowing,
    ) -> Plan {
        let mut plan = Plan {
            tier,
            arms: Vec::new(),
            repetitions: 0,
            lock: lock.to_string(),
            tree: String::new(),
            selection: String::new(),
            seed: narrowing.seed,
            harness: headwater_resolve_version(),
            selected: Vec::new(),
            corpus: 0,
            sessions: 0,
            projected: 0,
            budget: 0,
            session_cost: 0,
            refusal: None,
        };

        let mut tree = String::new();
        let mut probes = Vec::new();
        for row in &census.rows {
            let Outcome::Typed { kind, .. } = &row.outcome else {
                continue;
            };
            plan.corpus += 1;
            tree.push_str(&row.path);
            tree.push('\t');
            tree.push_str(row.digest.as_deref().unwrap_or("-"));
            tree.push('\n');
            if kind == KIND {
                probes.push(row);
            }
        }
        plan.tree = headwater_hash::digest(tree.as_bytes());

        if probes.is_empty() {
            plan.refusal = Some(Refusal::NoProbes);
            return plan;
        }

        for row in probes {
            let facets = row.document.as_ref().map(|document| &document.facets);
            let read = |key: &str| -> Option<String> {
                facets?
                    .get(key)?
                    .value
                    .as_scalar()
                    .map(|scalar| scalar.text.clone())
            };

            let Some(id) = read(&config.identifier_facet) else {
                plan.refusal = Some(Refusal::Unnameable {
                    path: row.path.clone(),
                });
                return plan;
            };
            let raw_category = read("probe_category");
            let Some(category) = raw_category.as_deref().and_then(Category::read) else {
                plan.refusal = Some(Refusal::Undeclared {
                    path: row.path.clone(),
                    facet: "probe_category",
                    found: raw_category,
                });
                return plan;
            };
            let raw_expectation = read("expectation");
            let Some(expectation) = raw_expectation.as_deref().and_then(Expectation::read) else {
                plan.refusal = Some(Refusal::Undeclared {
                    path: row.path.clone(),
                    facet: "expectation",
                    found: raw_expectation,
                });
                return plan;
            };

            let mut examines: Vec<String> = graph
                .edges
                .iter()
                .filter(|edge| edge.source.path == row.path && edge.declared == EXAMINES)
                .filter_map(|edge| match &edge.target {
                    Target::Document { id, .. } => Some(id.clone()),
                    _ => None,
                })
                .collect();
            examines.sort();
            examines.dedup();

            let oracle = read("oracle");
            if expectation == Expectation::Patched {
                let Some(rule) = oracle.clone() else {
                    plan.refusal = Some(Refusal::OracleUnnamed { probe: id });
                    return plan;
                };
                if !headwater_check::RULES.contains(&rule.as_str()) {
                    plan.refusal = Some(Refusal::OracleUnknown { probe: id, rule });
                    return plan;
                }
            }
            if expectation.names_documents() && examines.is_empty() {
                plan.refusal = Some(Refusal::ExpectationNamesNothing {
                    probe: id,
                    expectation,
                });
                return plan;
            }

            plan.selected.push(Selected {
                path: row.path.clone(),
                id,
                category,
                expectation,
                examines,
                oracle,
            });
        }

        if let Some(category) = narrowing.category {
            plan.selected
                .retain(|selected| selected.category == category);
            if plan.selected.is_empty() {
                plan.refusal = Some(Refusal::SelectionEmpty {
                    category: Some(category),
                });
                return plan;
            }
        }
        plan.selected.sort_by(|a, b| a.id.cmp(&b.id));

        let names: Vec<&str> = plan
            .selected
            .iter()
            .map(|selected| selected.id.as_str())
            .collect();
        plan.selection = headwater_hash::digest(names.join("\n").as_bytes());

        let Some(envelope) = budgets.of(tier) else {
            plan.refusal = Some(Refusal::TierUndeclared(tier));
            return plan;
        };
        plan.repetitions = envelope.repetitions;
        plan.budget = envelope.budget;
        plan.session_cost = envelope.session_cost;
        plan.arms = arms(envelope, narrowing.arm);
        if plan.arms.len() < envelope.arms.len() && tier == Tier::Campaign {
            plan.refusal = Some(Refusal::CampaignNarrowed);
            return plan;
        }
        if plan.arms.is_empty() {
            plan.arms = envelope.arms.clone();
        }

        plan.sessions =
            plan.selected.len() as u64 * plan.arms.len() as u64 * u64::from(plan.repetitions);
        plan.projected = plan.sessions * plan.session_cost;
        if plan.projected > plan.budget {
            plan.refusal = Some(Refusal::OverBudget {
                sessions: plan.sessions,
                projected: plan.projected,
                budget: plan.budget,
            });
        }
        plan
    }

    /// Whether the run this plan describes may start.
    pub fn runs(&self) -> bool {
        self.refusal.is_none()
    }

    /// The briefing, for the recorder rather than for a model.
    ///
    /// One format, and it is prose, for the reason the sweep's plan is prose.
    /// The difference is the reader: a sweep briefs a model and this briefs the
    /// process that drives one. Nothing here is a prompt, because a prompt that
    /// the engine wrote would put this engine's phrasing inside the thing under
    /// test.
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(
            out,
            "A {} probe run over {} of the {} classified documents of this corpus.",
            self.tier.name(),
            self.selected.len(),
            self.corpus
        );
        let _ = writeln!(out);

        let _ = writeln!(out, "## The run identity this plan fixes");
        let _ = writeln!(out);
        let _ = writeln!(out, "```yaml");
        let _ = writeln!(out, "lock: {}", self.lock);
        let _ = writeln!(out, "tree: {}", self.tree);
        let _ = writeln!(out, "selection: {}", self.selection);
        let _ = writeln!(out, "seed: {}", self.seed);
        let _ = writeln!(out, "harness: {}", self.harness);
        let _ = writeln!(out, "tier: {}", self.tier.name());
        let _ = writeln!(
            out,
            "arms: [{}]",
            self.arms
                .iter()
                .map(|arm| arm.name())
                .collect::<Vec<_>>()
                .join(", ")
        );
        let _ = writeln!(out, "```");
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "The model, its served version, the wall-clock time and the realized cost belong to \
             the run. The recorder writes those four into the transcript, and `headwater probe \
             record` holds the rest of the identity to the five above."
        );
        let _ = writeln!(out);

        // The cost section is printed only where a cost was computed. A run
        // refused before the envelope was read has no projection, and a table
        // of zeros beside a refusal reads as a run that costs nothing rather
        // than as a run that was never priced.
        if self.budget > 0 {
            let _ = writeln!(out, "## The cost");
            let _ = writeln!(out);
            let _ = writeln!(
                out,
                "{} probes x {} arms x {} repetitions = {} sessions, at a declared {} each: {} \
                 against a ceiling of {}.",
                self.selected.len(),
                self.arms.len(),
                self.repetitions,
                self.sessions,
                dollars(self.session_cost),
                dollars(self.projected),
                dollars(self.budget)
            );
            let _ = writeln!(
                out,
                "The unit cost is a number a person declared in `{}`. No run has happened, so no \
                 realized cost has been recorded against it and this is arithmetic rather than a \
                 forecast.",
                crate::budget::PATH
            );
            let _ = writeln!(out);
        }

        if let Some(refusal) = &self.refusal {
            let _ = writeln!(out, "## This run does not start");
            let _ = writeln!(out);
            let _ = writeln!(out, "{refusal}");
            return out;
        }

        let _ = writeln!(out, "## The selection");
        let _ = writeln!(out);
        for selected in &self.selected {
            let _ = writeln!(out, "- {} ({})", selected.id, selected.path);
            let _ = writeln!(out, "    category: {}", selected.category.name());
            let _ = writeln!(out, "    expects: {}", selected.expectation.name());
            if !selected.examines.is_empty() {
                let _ = writeln!(out, "    examines: {}", selected.examines.join(", "));
            }
            if let Some(oracle) = &selected.oracle {
                let _ = writeln!(out, "    oracle: {oracle}");
            }
        }
        let _ = writeln!(out);

        let _ = writeln!(out, "## What the recorder writes back");
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "One `probe_transcript` document on `docs/probe-runs/`, then `headwater probe record \
             <path>`. The transcript is the ordered tool-call events of each session, and it holds \
             no model prose: the intake refuses a key outside the closed set, which is how the \
             omission is enforced rather than asked for."
        );
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "**A transcript that an agent wrote about its own session is a self-report, and spec 5 \
             refuses one.** The events have to be observed from outside the session by the process \
             that drove it. That process reaches the network and it is not in this repository, so \
             no verb here writes a transcript and there is no flag that makes one."
        );
        out
    }
}

/// The arms of a run: the tier's, narrowed by the caller if the caller asked.
fn arms(envelope: &Envelope, narrowing: Option<Arm>) -> Vec<Arm> {
    match narrowing {
        None => envelope.arms.clone(),
        Some(arm) => envelope
            .arms
            .iter()
            .copied()
            .filter(|declared| *declared == arm)
            .collect(),
    }
}

/// The harness version, which is the version of this engine.
///
/// A separate number would be a second version to keep in step with the one a
/// `requires_engine` range already reads, and two versions that drift name two
/// harnesses.
fn headwater_resolve_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_patched_probe_grades_against_a_rule_this_engine_carries() {
        assert!(
            headwater_check::RULES.contains(&"section.required.missing"),
            "the oracle refusal reads this list, so a name in it is the arm that passes"
        );
        assert!(!headwater_check::RULES.contains(&"section.required.absent"));
    }

    #[test]
    fn the_three_document_forms_are_the_ones_that_need_a_target() {
        for expectation in Expectation::ALL {
            let expected = matches!(
                expectation,
                Expectation::Opened | Expectation::NotOpened | Expectation::Cited
            );
            assert_eq!(expectation.names_documents(), expected);
        }
    }
}
