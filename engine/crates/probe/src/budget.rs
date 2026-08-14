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
//! [#87](https://github.com/headwater-ai/headwater/issues/87) owns the two
//! tiers and their cadence, and it may move this file. What it may not do is
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
//! # Every field is required
//!
//! A default would be this engine choosing how much money to spend. A tier that
//! omits a field is refused, and a run at that tier does not happen.

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
}

/// Every declared tier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Budgets {
    pub tiers: Vec<Envelope>,
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

        let mut tiers = Vec::new();
        for entry in block.iter() {
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
        Ok(Budgets { tiers })
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
    if tier == Tier::Campaign && arms.len() != Arm::ALL.len() {
        return Err(Unreadable::CampaignHasOneArm);
    }

    Ok(Envelope {
        tier,
        budget,
        session_cost,
        repetitions,
        arms,
    })
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
    CampaignHasOneArm,
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
                "the `{tier}` tier names the arm `{found}`. The arms are `present` and `absent`"
            ),
            Unreadable::CampaignHasOneArm => write!(
                f,
                "the `campaign` tier names one arm. A campaign estimates a difference, so it runs \
                 the pair, and a campaign with one arm is the regression tier under a name a \
                 published claim would cite"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = "\
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
";

    #[test]
    fn both_tiers_read() {
        let budgets = Budgets::read(GOOD).expect("reads");
        assert_eq!(budgets.tiers.len(), 2);
        let campaign = budgets.of(Tier::Campaign).expect("campaign");
        assert_eq!(campaign.repetitions, 58);
        assert_eq!(campaign.arms, vec![Arm::Present, Arm::Absent]);
    }

    #[test]
    fn a_campaign_with_one_arm_is_refused() {
        let source = GOOD.replace("arms: [present, absent]", "arms: [present]");
        assert_eq!(
            Budgets::read(&source),
            Err(Unreadable::CampaignHasOneArm),
            "a campaign estimates a difference and one arm estimates none"
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
}
