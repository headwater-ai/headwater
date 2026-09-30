// SPDX-License-Identifier: Apache-2.0
//! The envelope each tier declares, and the file a person writes it in.
//!
//! # Why the declaration is outside the corpus root and outside the taxonomy
//!
//! A budget is a policy of the repository that runs the probes, and it is not a
//! fact about any document. Nothing classifies it, no language regime binds it,
//! and no rule reads it — which is the test [#74](https://github.com/headwater-ai/headwater/issues/74)
//! applied to the capture-cost store and reached the same answer for. It is not
//! in `.headwater/taxonomy.yml` either, because that file is the consumer
//! declaration of spec 7 and a taxonomy is not what decides how much a run may
//! cost. So it is `.headwater/probe.yml`, beside the lock and beside the cache.
//!
//! [HW-DR-0076](../../../../docs/decisions/0076-a-probe-budget-prices-a-run-identity-fixed-before-the-run-and-a-committed-transcript-and-a-sweep-has-neither.md)
//! rules that this file stays here and does not move. What no ruling may do is
//! remove the number, because a harness with no budget is a harness that cannot
//! fail closed.
//!
//! # The unit cost is a person's estimate, and nothing here pretends otherwise
//!
//! `session_cost` is what somebody believes a session of this tier costs. No
//! run has happened, so no realized cost has ever been recorded here and there
//! is nothing to fit an estimate to. That makes the projection an arithmetic
//! statement about a declared number rather than a forecast, and
//! [`crate::plan::Plan::render`] says so on the line that prints it. When a run
//! does land, its transcript carries the realized cost and the two numbers are
//! then comparable — which is the only route from a guess to an estimate.
//!
//! # Every field is required, except the turn cap
//!
//! A default would be this engine choosing how much money to spend. A tier that
//! omits a field is refused, and a run at that tier does not happen. The one
//! exception is `max_turns`: it bounds a session rather than the batch, and a
//! tier with none runs each session to the harness's own end.

use crate::{Arm, Cents, Tier};
use headwater_yaml::value::{Mapping, Value};

/// The path, relative to the repository root.
pub const PATH: &str = ".headwater/probe.yml";

/// What one tier may spend, and what a session of it is believed to cost.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Envelope {
    pub tier: Tier,
    /// The ceiling. A projected cost above it refuses the run.
    pub budget: Cents,
    /// What one session is believed to cost. See the module comment: this is a
    /// declared number and not a measured one.
    pub session_cost: Cents,
    /// How many sessions of each probe in each arm. Spec 5 takes this from
    /// statistical power for a campaign, and a regression tier watches one arm
    /// once.
    pub repetitions: u32,
    /// The arms this tier runs. A campaign runs the pair or it estimates
    /// nothing.
    pub arms: Vec<Arm>,
    /// The paths, relative to the repository root, that the absent arm
    /// removes. Spec 5: the absent arm names a declared ablation. Empty for a
    /// tier that runs no absent arm, and refused as empty for one that does.
    pub ablation: Vec<String>,
    /// The turn cap each session of this tier runs under, where the tier
    /// declares one. A session the cap stops is recorded as observed and never
    /// drawn again (#1384), so the cap is part of what a rate of the tier
    /// measures and both arms share it. It is the one optional field: a
    /// regression tier watches one session and declares none.
    pub max_turns: Option<u32>,
    /// The delta of each component arm the tier runs, against the present
    /// tree, in declared order (#1472). A removing arm's paths are taken out of
    /// the present tree, and an adding arm's paths are put in. Empty for a tier
    /// that runs no component arm.
    pub components: Vec<(Arm, Vec<String>)>,
}

impl Envelope {
    /// The paths an arm's tree lacks against the present tree: the ablation
    /// for the absent arm, the delta of a removing component arm, and nothing
    /// for the present arm or an adding one.
    pub fn removes(&self, arm: Arm) -> &[String] {
        match arm {
            Arm::Present => &[],
            Arm::Absent => &self.ablation,
            _ if arm.adds() => &[],
            _ => self.delta(arm),
        }
    }

    /// The paths an arm's tree holds and the present tree does not.
    pub fn adds(&self, arm: Arm) -> &[String] {
        if arm.adds() {
            self.delta(arm)
        } else {
            &[]
        }
    }

    fn delta(&self, arm: Arm) -> &[String] {
        self.components
            .iter()
            .find(|(declared, _)| *declared == arm)
            .map(|(_, paths)| paths.as_slice())
            .unwrap_or(&[])
    }
}

/// Every declared tier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Budgets {
    pub tiers: Vec<Envelope>,
    /// The paths, relative to the repository root, that every arm of every
    /// tier removes: the probe shelves, whose documents state the answer each
    /// probe expects. They are the instrument and not the treatment, so no
    /// arm holds them. Empty where the file declares none.
    pub instrument: Vec<String>,
}

impl Budgets {
    pub fn of(&self, tier: Tier) -> Option<&Envelope> {
        self.tiers.iter().find(|envelope| envelope.tier == tier)
    }

    /// Read the declaration.
    ///
    /// It never half-reads: a file with one bad tier is refused whole, because
    /// a partial budget is a ceiling nobody wrote.
    pub fn read(source: &str) -> Result<Budgets, Unreadable> {
        let loaded = headwater_yaml::load(source).map_err(|errors| {
            Unreadable::Unparsed(errors.iter().map(|error| error.to_string()).collect())
        })?;
        let Value::Map(root) = &loaded.value else {
            return Err(Unreadable::Malformed(format!(
                "{PATH} is not a mapping, and a `tiers` key is what it owes"
            )));
        };
        let Some(entry) = root.get("tiers") else {
            return Err(Unreadable::Malformed(format!(
                "{PATH} carries no `tiers` key, so no tier declares a budget"
            )));
        };
        let Some(block) = entry.value.as_map() else {
            return Err(Unreadable::Malformed(format!(
                "`tiers` in {PATH} is not a mapping of tier name to envelope"
            )));
        };

        let instrument = match root.get("instrument") {
            None => Vec::new(),
            Some(listed) => paths(&listed.value)
                .ok_or_else(|| {
                    Unreadable::Malformed(format!(
                        "`instrument` in {PATH} is not a sequence of paths"
                    ))
                })?
                .map_err(|entry| Unreadable::InstrumentUnsafe { entry })?,
        };

        let mut tiers = Vec::new();
        for entry in block {
            let name = entry.key.value.as_str();
            let Some(tier) = Tier::read(name) else {
                return Err(Unreadable::UnknownTier(name.to_string()));
            };
            let Some(fields) = entry.value.value.as_map() else {
                return Err(Unreadable::Malformed(format!(
                    "the `{name}` tier in {PATH} is not a mapping"
                )));
            };
            tiers.push(envelope(tier, fields)?);
        }
        if tiers.is_empty() {
            return Err(Unreadable::Malformed(format!(
                "{PATH} declares no tier, and a run at a tier with no envelope cannot fail closed"
            )));
        }
        Ok(Budgets { tiers, instrument })
    }
}

fn envelope(tier: Tier, fields: &Mapping) -> Result<Envelope, Unreadable> {
    let name = tier.name();
    let cents = |key: &'static str| -> Result<Cents, Unreadable> {
        let text = text(fields, key).ok_or(Unreadable::Missing {
            tier: name,
            field: key,
        })?;
        text.parse::<Cents>().map_err(|_| Unreadable::NotACount {
            tier: name,
            field: key,
            found: text,
        })
    };
    let budget = cents("budget_cents")?;
    let session_cost = cents("session_cost_cents")?;
    let repetitions = cents("repetitions")? as u32;
    if repetitions == 0 {
        return Err(Unreadable::NotACount {
            tier: name,
            field: "repetitions",
            found: "0".into(),
        });
    }

    let listed = fields.get("arms").ok_or(Unreadable::Missing {
        tier: name,
        field: "arms",
    })?;
    let Some(items) = listed.value.as_seq() else {
        return Err(Unreadable::Missing {
            tier: name,
            field: "arms",
        });
    };
    let mut arms = Vec::new();
    for item in items {
        let text = item
            .value
            .as_scalar()
            .map(|scalar| scalar.text.clone())
            .unwrap_or_default();
        let arm = Arm::read(&text).ok_or(Unreadable::UnknownArm {
            tier: name,
            found: text,
        })?;
        if !arms.contains(&arm) {
            arms.push(arm);
        }
    }
    if arms.is_empty() {
        return Err(Unreadable::Missing {
            tier: name,
            field: "arms",
        });
    }
    // Spec 5: "A campaign runs as one batch, or it is not one measurement", and
    // the pair is what an efficacy claim rests on. A campaign declaring one arm
    // is a campaign that estimates nothing, which is the regression tier under
    // a name that would let a published claim cite it.
    //
    // A paired tier may run more than two arms since #1472, and every arm past
    // the present one is compared with the present one. So the present arm is
    // required, and one arm alone is still refused.
    if tier.pairs_arms() && arms.len() < 2 {
        return Err(Unreadable::PairHasOneArm { tier: name });
    }
    if tier.pairs_arms() && !arms.contains(&Arm::Present) {
        return Err(Unreadable::PairWithoutPresent { tier: name });
    }

    // Spec 5: the absent arm names a declared ablation. An absent arm with
    // none is whatever a script happens to remove, and an ablation with no
    // absent arm is declared and never applied.
    let ablation = match fields.get("ablation") {
        None => Vec::new(),
        Some(listed) => paths(&listed.value)
            .ok_or_else(|| {
                Unreadable::Malformed(format!(
                    "`ablation` of the `{name}` tier in {PATH} is not a sequence of paths"
                ))
            })?
            .map_err(|entry| Unreadable::AblationUnsafe { tier: name, entry })?,
    };
    let absent = arms.contains(&Arm::Absent);
    if absent && ablation.is_empty() {
        return Err(Unreadable::AblationUndeclared { tier: name });
    }
    if !absent && !ablation.is_empty() {
        return Err(Unreadable::AblationWithoutAbsent { tier: name });
    }

    // #1472: each component arm names its delta against the present tree, as
    // the absent arm names its ablation, and each delta names an arm the tier
    // runs. The present arm has no delta, and the absent arm's is `ablation`.
    let mut components: Vec<(Arm, Vec<String>)> = Vec::new();
    if let Some(listed) = fields.get("components") {
        let Some(block) = listed.value.as_map() else {
            return Err(Unreadable::Malformed(format!(
                "`components` of the `{name}` tier in {PATH} is not a mapping of arm to paths"
            )));
        };
        for entry in block {
            let found = entry.key.value.as_str();
            let arm = Arm::read(found).ok_or_else(|| Unreadable::UnknownArm {
                tier: name,
                found: found.to_string(),
            })?;
            if !arm.is_component() {
                return Err(Unreadable::ComponentOnBaseArm { tier: name, arm });
            }
            if !arms.contains(&arm) {
                return Err(Unreadable::ComponentWithoutArm { tier: name, arm });
            }
            let delta = paths(&entry.value.value)
                .ok_or_else(|| {
                    Unreadable::Malformed(format!(
                        "the `{found}` delta of the `{name}` tier in {PATH} is not a sequence of \
                         paths"
                    ))
                })?
                .map_err(|entry| Unreadable::ComponentUnsafe {
                    tier: name,
                    arm,
                    entry,
                })?;
            if !delta.is_empty() {
                components.push((arm, delta));
            }
        }
    }
    if let Some(arm) = arms
        .iter()
        .copied()
        .filter(|arm| arm.is_component())
        .find(|arm| !components.iter().any(|(declared, _)| declared == arm))
    {
        return Err(Unreadable::ComponentUndeclared { tier: name, arm });
    }

    let max_turns = match text(fields, "max_turns") {
        None => None,
        Some(found) => match found.parse::<u32>() {
            Ok(turns) if turns > 0 => Some(turns),
            _ => {
                return Err(Unreadable::NotACount {
                    tier: name,
                    field: "max_turns",
                    found,
                });
            }
        },
    };

    Ok(Envelope {
        tier,
        budget,
        session_cost,
        repetitions,
        arms,
        ablation,
        max_turns,
        components,
    })
}

/// A sequence of paths inside the tree, deduplicated in declared order.
///
/// `None` where the value is not a sequence, and the first unsafe entry as the
/// error.
fn paths(value: &Value) -> Option<Result<Vec<String>, String>> {
    let items = value.as_seq()?;
    let mut paths: Vec<String> = Vec::new();
    for item in items {
        let entry = item
            .value
            .as_scalar()
            .map(|scalar| scalar.text.clone())
            .unwrap_or_default();
        if !ablation_entry_is_safe(&entry) {
            return Some(Err(entry));
        }
        if !paths.contains(&entry) {
            paths.push(entry);
        }
    }
    Some(Ok(paths))
}

/// Whether an ablation entry names a path inside the tree.
///
/// `tools/probe/ablate.sh` hands every entry to `rm -rf` in a copy of the
/// tree, so an empty entry, an absolute one, or one with a `..` component
/// would remove the copy itself or something outside it.
fn ablation_entry_is_safe(entry: &str) -> bool {
    !entry.is_empty()
        && !entry.starts_with('/')
        && entry
            .split('/')
            .all(|component| component != ".." && component != "." && !component.is_empty())
}

fn text(map: &Mapping, key: &str) -> Option<String> {
    Some(map.get(key)?.value.as_scalar()?.text.clone())
}

/// Why no tier declares an envelope this engine can use.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unreadable {
    Unparsed(Vec<String>),
    Malformed(String),
    UnknownTier(String),
    Missing {
        tier: &'static str,
        field: &'static str,
    },
    NotACount {
        tier: &'static str,
        field: &'static str,
        found: String,
    },
    UnknownArm {
        tier: &'static str,
        found: String,
    },
    PairHasOneArm {
        tier: &'static str,
    },
    PairWithoutPresent {
        tier: &'static str,
    },
    ComponentUndeclared {
        tier: &'static str,
        arm: Arm,
    },
    ComponentWithoutArm {
        tier: &'static str,
        arm: Arm,
    },
    ComponentOnBaseArm {
        tier: &'static str,
        arm: Arm,
    },
    ComponentUnsafe {
        tier: &'static str,
        arm: Arm,
        entry: String,
    },
    AblationUndeclared {
        tier: &'static str,
    },
    AblationWithoutAbsent {
        tier: &'static str,
    },
    AblationUnsafe {
        tier: &'static str,
        entry: String,
    },
    InstrumentUnsafe {
        entry: String,
    },
}

impl std::fmt::Display for Unreadable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unreadable::Unparsed(errors) => {
                write!(f, "{PATH} did not parse as YAML: {}", errors.join("; "))
            }
            Unreadable::Malformed(what) => write!(f, "{what}"),
            Unreadable::UnknownTier(name) => write!(
                f,
                "`{name}` is not a tier. The tiers are: {}",
                Tier::ALL
                    .iter()
                    .map(|tier| tier.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Unreadable::Missing { tier, field } => write!(
                f,
                "the `{tier}` tier declares no `{field}`, and every field is required because a \
                 default would be this engine choosing how much to spend"
            ),
            Unreadable::NotACount { tier, field, found } => write!(
                f,
                "`{field}` of the `{tier}` tier is `{found}`, and it has to be a whole number \
                 above zero"
            ),
            Unreadable::UnknownArm { tier, found } => write!(
                f,
                "the `{tier}` tier names the arm `{found}`. The arms are {}",
                Arm::listed()
            ),
            Unreadable::PairHasOneArm { tier } => write!(
                f,
                "the `{tier}` tier names one arm. It estimates a difference, so it runs the \
                 pair, and with one arm it is the regression tier under a name a published \
                 claim would cite"
            ),
            Unreadable::PairWithoutPresent { tier } => write!(
                f,
                "the `{tier}` tier runs no `present` arm. Every other arm is compared with the \
                 present tree, so without it two arms differ by two deltas and neither is \
                 measured"
            ),
            Unreadable::ComponentUndeclared { tier, arm } => write!(
                f,
                "the `{tier}` tier runs the `{}` arm and declares no delta for it under \
                 `components`. A component arm names what it removes or adds, or the claim it \
                 measures is whatever a script did",
                arm.name()
            ),
            Unreadable::ComponentWithoutArm { tier, arm } => write!(
                f,
                "the `{tier}` tier declares a delta for the `{}` arm under `components` and \
                 does not run that arm, so the delta is declared and never applied",
                arm.name()
            ),
            Unreadable::ComponentOnBaseArm { tier, arm } => write!(
                f,
                "the `{tier}` tier declares a delta for the `{}` arm under `components`. The \
                 present arm is the tree every delta is taken from, and the absent arm's delta \
                 is `ablation`",
                arm.name()
            ),
            Unreadable::ComponentUnsafe { tier, arm, entry } => write!(
                f,
                "the `{}` delta of the `{tier}` tier names `{entry}`. An entry is a path inside \
                 the tree, relative to its root, with no `..`, `.` or empty component, because \
                 the arm's tree is built by removing or writing it",
                arm.name()
            ),
            Unreadable::AblationUndeclared { tier } => write!(
                f,
                "the `{tier}` tier runs the `absent` arm and declares no `ablation`. Spec 5: the \
                 absent arm names a declared ablation, or the claim it measures is whatever a \
                 script removed"
            ),
            Unreadable::AblationWithoutAbsent { tier } => write!(
                f,
                "the `{tier}` tier declares an `ablation` and runs no `absent` arm, so the \
                 ablation is declared and never applied"
            ),
            Unreadable::AblationUnsafe { tier, entry } => write!(
                f,
                "the `{tier}` tier's ablation names `{entry}`. An entry is a path inside the \
                 tree, relative to its root, with no `..`, `.` or empty component, because \
                 the absent arm removes it with `rm -rf`"
            ),
            Unreadable::InstrumentUnsafe { entry } => write!(
                f,
                "the `instrument` of {PATH} names `{entry}`. An entry is a path inside the tree, \
                 relative to its root, with no `..`, `.` or empty component, because every arm \
                 removes it with `rm -rf`"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = "\
instrument: [docs/probes, docs/probe-runs]
tiers:
  regression:
    budget_cents: 2000
    session_cost_cents: 4
    repetitions: 1
    arms: [present]
  campaign:
    budget_cents: 40000
    session_cost_cents: 4
    repetitions: 58
    arms: [present, absent]
    ablation: [CLAUDE.md, .claude, .githooks, .headwater]
  documentation:
    budget_cents: 40000
    session_cost_cents: 4
    repetitions: 58
    arms: [present, absent]
    ablation: [CLAUDE.md, .claude, .githooks, .headwater, docs]
";

    #[test]
    fn every_tier_reads_with_its_ablation() {
        let budgets = Budgets::read(GOOD).expect("reads");
        assert_eq!(budgets.tiers.len(), 3);
        let campaign = budgets.of(Tier::Campaign).expect("campaign");
        assert_eq!(campaign.repetitions, 58);
        assert_eq!(campaign.arms, vec![Arm::Present, Arm::Absent]);
        assert_eq!(
            campaign.ablation,
            vec!["CLAUDE.md", ".claude", ".githooks", ".headwater"]
        );
        let documentation = budgets.of(Tier::Documentation).expect("documentation");
        assert_eq!(
            documentation.ablation,
            vec!["CLAUDE.md", ".claude", ".githooks", ".headwater", "docs"]
        );
        let regression = budgets.of(Tier::Regression).expect("regression");
        assert!(regression.ablation.is_empty(), "one arm, nothing removed");
        assert_eq!(budgets.instrument, vec!["docs/probes", "docs/probe-runs"]);
    }

    #[test]
    fn a_file_with_no_instrument_removes_nothing_from_any_arm() {
        let source = GOOD.replace("instrument: [docs/probes, docs/probe-runs]\n", "");
        assert!(Budgets::read(&source).expect("reads").instrument.is_empty());
    }

    #[test]
    fn an_instrument_entry_that_leaves_the_tree_is_refused() {
        // `tools/probe/ablate.sh` hands every entry to `rm -rf` in every arm.
        let source = GOOD.replace("[docs/probes, docs/probe-runs]", "[docs/probes, ../x]");
        assert_eq!(
            Budgets::read(&source),
            Err(Unreadable::InstrumentUnsafe {
                entry: "../x".to_string()
            })
        );
    }

    #[test]
    fn a_campaign_with_one_arm_is_refused() {
        let source = GOOD.replacen("arms: [present, absent]", "arms: [present]", 1);
        assert_eq!(
            Budgets::read(&source),
            Err(Unreadable::PairHasOneArm { tier: "campaign" }),
            "a campaign estimates a difference and one arm estimates none"
        );
    }

    #[test]
    fn a_documentation_tier_with_one_arm_is_refused() {
        let source = GOOD
            .replace(
                "    arms: [present, absent]\n    ablation: [CLAUDE.md, .claude, .githooks, .headwater, docs]\n",
                "    arms: [present]\n",
            );
        assert_eq!(
            Budgets::read(&source),
            Err(Unreadable::PairHasOneArm {
                tier: "documentation"
            }),
            "the documentation tier estimates a difference too"
        );
    }

    #[test]
    fn an_absent_arm_with_no_ablation_is_refused() {
        // Spec 5: the absent arm names a declared ablation. Without one, the
        // absent tree is whatever a script removes, and no claim names it.
        let source = GOOD.replace(
            "    ablation: [CLAUDE.md, .claude, .githooks, .headwater]\n",
            "",
        );
        assert_eq!(
            Budgets::read(&source),
            Err(Unreadable::AblationUndeclared { tier: "campaign" })
        );
    }

    #[test]
    fn an_ablation_without_the_absent_arm_is_refused() {
        let source = GOOD.replace(
            "    arms: [present]\n",
            "    arms: [present]\n    ablation: [docs]\n",
        );
        assert_eq!(
            Budgets::read(&source),
            Err(Unreadable::AblationWithoutAbsent { tier: "regression" }),
            "an ablation no arm applies is declared and never read"
        );
    }

    #[test]
    fn an_ablation_entry_that_leaves_the_tree_is_refused() {
        // `tools/probe/ablate.sh` hands every entry to `rm -rf`.
        for entry in ["docs/../..", "/etc", "..", "\"\""] {
            let source = GOOD.replace(
                "[CLAUDE.md, .claude, .githooks, .headwater, docs]",
                &format!("[CLAUDE.md, {entry}]"),
            );
            assert_eq!(
                Budgets::read(&source),
                Err(Unreadable::AblationUnsafe {
                    tier: "documentation",
                    entry: entry.trim_matches('"').to_string(),
                }),
                "{entry}"
            );
        }
    }

    #[test]
    fn a_sweep_tier_is_refused_a_sweep_has_no_run_identity_to_price() {
        // HW-DR-0076: a budget prices a run whose identity is fixed before it
        // starts, and that commits a transcript. A sweep fixes only one of
        // the six run-identity members (the taxonomy lock) and commits no
        // transcript, so it is not a tier this file may declare. This guards
        // against a later change adding one without reopening that ruling.
        let source = format!(
            "{GOOD}  sweep:\n    budget_cents: 2000\n    session_cost_cents: 4\n    \
             repetitions: 1\n    arms: [present]\n"
        );
        assert_eq!(
            Budgets::read(&source),
            Err(Unreadable::UnknownTier("sweep".to_string())),
            "a sweep fixes only the lock and commits no transcript, so no budget prices it"
        );
    }

    #[test]
    fn a_missing_field_refuses_the_file_rather_than_defaulting() {
        let source = GOOD.replace("    budget_cents: 2000\n", "");
        assert_eq!(
            Budgets::read(&source),
            Err(Unreadable::Missing {
                tier: "regression",
                field: "budget_cents",
            })
        );
    }

    #[test]
    fn a_declared_turn_cap_is_read_and_a_tier_without_one_has_none() {
        let source = GOOD.replace(
            "    repetitions: 58\n    arms: [present, absent]\n    ablation: [CLAUDE.md, .claude, .githooks, .headwater]\n",
            "    repetitions: 58\n    max_turns: 80\n    arms: [present, absent]\n    ablation: [CLAUDE.md, .claude, .githooks, .headwater]\n",
        );
        assert_ne!(source, GOOD, "the replacement found the campaign tier");
        let budgets = Budgets::read(&source).expect("reads");
        assert_eq!(
            budgets.of(Tier::Campaign).expect("campaign").max_turns,
            Some(80)
        );
        assert_eq!(
            budgets.of(Tier::Regression).expect("regression").max_turns,
            None
        );
    }

    #[test]
    fn a_turn_cap_of_zero_or_a_word_is_refused() {
        for found in ["0", "many"] {
            let source = GOOD.replace(
                "    session_cost_cents: 4\n    repetitions: 1\n",
                &format!("    session_cost_cents: 4\n    repetitions: 1\n    max_turns: {found}\n"),
            );
            assert_eq!(
                Budgets::read(&source),
                Err(Unreadable::NotACount {
                    tier: "regression",
                    field: "max_turns",
                    found: found.to_string(),
                })
            );
        }
    }

    // --- the component arms of #1472 -----------------------------------------
    //
    // Each component arm is the present tree with one part of the Headwater
    // layer taken away or put in. Its delta is data in the declaration, the
    // way the absent arm's `ablation` is, so what an arm removed is a recorded
    // fact and not whatever a script did.

    const LAYER: &str = "\
tiers:
  campaign:
    budget_cents: 40000
    session_cost_cents: 4
    repetitions: 58
    arms: [present, absent, no-hook, no-skills, no-claude-md, mcp]
    ablation: [CLAUDE.md, .claude, .githooks, .headwater]
    components:
      no-hook: [.claude/hooks/intent.sh]
      no-skills: [.claude/skills]
      no-claude-md: [CLAUDE.md]
      mcp: [.mcp.json]
";

    #[test]
    fn a_campaign_reads_each_component_arm_with_its_delta() {
        let budgets = Budgets::read(LAYER).expect("reads");
        let campaign = budgets.of(Tier::Campaign).expect("campaign");
        assert_eq!(
            campaign.arms,
            vec![
                Arm::Present,
                Arm::Absent,
                Arm::NoHook,
                Arm::NoSkills,
                Arm::NoClaudeMd,
                Arm::Mcp
            ]
        );
        assert_eq!(campaign.removes(Arm::NoHook), [".claude/hooks/intent.sh"]);
        assert_eq!(campaign.removes(Arm::NoSkills), [".claude/skills"]);
        assert_eq!(campaign.removes(Arm::NoClaudeMd), ["CLAUDE.md"]);
        assert_eq!(
            campaign.removes(Arm::Absent),
            ["CLAUDE.md", ".claude", ".githooks", ".headwater"],
            "the absent arm's delta is its ablation"
        );
        assert!(campaign.removes(Arm::Mcp).is_empty(), "mcp removes nothing");
        assert!(campaign.removes(Arm::Present).is_empty());
        assert_eq!(campaign.adds(Arm::Mcp), [".mcp.json"]);
        assert!(campaign.adds(Arm::NoSkills).is_empty());
    }

    #[test]
    fn an_arm_nobody_declared_is_refused_and_the_refusal_names_every_arm() {
        let source = LAYER.replace("no-claude-md, mcp]", "no-claude-md, mcp, no-docs]");
        let refused = Budgets::read(&source);
        assert_eq!(
            refused,
            Err(Unreadable::UnknownArm {
                tier: "campaign",
                found: "no-docs".into()
            })
        );
        let message = refused.expect_err("refused").to_string();
        for arm in Arm::ALL {
            assert!(message.contains(&format!("`{}`", arm.name())), "{message}");
        }
    }

    #[test]
    fn a_delta_for_an_arm_nobody_declared_is_refused() {
        let source = LAYER.replace(
            "      mcp: [.mcp.json]\n",
            "      no-docs: [docs]\n      mcp: [.mcp.json]\n",
        );
        assert_eq!(
            Budgets::read(&source),
            Err(Unreadable::UnknownArm {
                tier: "campaign",
                found: "no-docs".into()
            })
        );
    }

    #[test]
    fn a_component_arm_with_no_delta_is_refused() {
        // The component arm's version of `AblationUndeclared`: an arm that
        // names no delta is whatever a script happens to remove.
        for delta in ["      no-skills: [.claude/skills]\n", ""] {
            let source = LAYER.replace(
                "      no-skills: [.claude/skills]\n",
                &delta.replace("[.claude/skills]", "[]"),
            );
            assert_eq!(
                Budgets::read(&source),
                Err(Unreadable::ComponentUndeclared {
                    tier: "campaign",
                    arm: Arm::NoSkills
                }),
                "{delta:?}"
            );
        }
    }

    #[test]
    fn a_delta_for_an_arm_the_tier_does_not_run_is_refused() {
        let source = LAYER.replace(", mcp]", "]");
        assert_eq!(
            Budgets::read(&source),
            Err(Unreadable::ComponentWithoutArm {
                tier: "campaign",
                arm: Arm::Mcp
            }),
            "a delta no arm applies is declared and never read"
        );
    }

    #[test]
    fn a_delta_on_the_present_or_the_absent_arm_is_refused() {
        // The present arm is what every delta is taken against, and the absent
        // arm's delta is `ablation`. A second place for either is a second
        // declaration of one fact.
        for arm in [Arm::Present, Arm::Absent] {
            let source = LAYER.replace(
                "      mcp: [.mcp.json]\n",
                &format!("      mcp: [.mcp.json]\n      {}: [docs]\n", arm.name()),
            );
            assert_eq!(
                Budgets::read(&source),
                Err(Unreadable::ComponentOnBaseArm {
                    tier: "campaign",
                    arm
                })
            );
        }
    }

    #[test]
    fn a_component_delta_that_leaves_the_tree_is_refused() {
        // `tools/probe/ablate.sh` hands a removing delta to `rm -rf` and
        // writes an adding one, so both stay inside the tree.
        // The refusal names the arm's delta, and never the ablation, which
        // is not at fault.
        for (line, entry, arm) in [
            ("      no-hook: [.claude/hooks/intent.sh]\n", "../x", Arm::NoHook),
            ("      mcp: [.mcp.json]\n", "/etc/passwd", Arm::Mcp),
        ] {
            let replaced = line.replace(
                &line[line.find('[').expect("[") + 1..line.find(']').expect("]")],
                entry,
            );
            let source = LAYER.replace(line, &replaced);
            let refused = Budgets::read(&source);
            assert_eq!(
                refused,
                Err(Unreadable::ComponentUnsafe {
                    tier: "campaign",
                    arm,
                    entry: entry.to_string()
                }),
                "{entry}"
            );
            let message = refused.expect_err("refused").to_string();
            assert!(
                message.contains(&format!("`{}` delta", arm.name())),
                "{message}"
            );
            assert!(!message.contains("ablation"), "{message}");
        }
    }

    #[test]
    fn a_paired_tier_with_no_present_arm_is_refused() {
        // Every delta is taken against the present tree, so a paired tier
        // without it compares two arms that differ by two deltas.
        let source = LAYER.replace("[present, absent, no-hook,", "[absent, no-hook,");
        assert_eq!(
            Budgets::read(&source),
            Err(Unreadable::PairWithoutPresent { tier: "campaign" })
        );
    }

    #[test]
    fn a_campaign_with_one_arm_and_its_components_is_still_refused() {
        let source = LAYER
            .replace(
                "[present, absent, no-hook, no-skills, no-claude-md, mcp]",
                "[present]",
            )
            .replace("    ablation: [CLAUDE.md, .claude, .githooks, .headwater]\n", "")
            .replace(
                "    components:\n      no-hook: [.claude/hooks/intent.sh]\n      no-skills: [.claude/skills]\n      no-claude-md: [CLAUDE.md]\n      mcp: [.mcp.json]\n",
                "",
            );
        assert_eq!(
            Budgets::read(&source),
            Err(Unreadable::PairHasOneArm { tier: "campaign" })
        );
    }

    #[test]
    fn every_refusal_of_a_component_arm_reads_as_a_sentence() {
        for refusal in [
            Unreadable::ComponentUndeclared {
                tier: "campaign",
                arm: Arm::NoHook,
            },
            Unreadable::ComponentWithoutArm {
                tier: "campaign",
                arm: Arm::Mcp,
            },
            Unreadable::ComponentOnBaseArm {
                tier: "campaign",
                arm: Arm::Absent,
            },
            Unreadable::PairWithoutPresent { tier: "campaign" },
        ] {
            let message = refusal.to_string();
            assert!(message.contains("`campaign`"), "{message}");
        }
    }
}
