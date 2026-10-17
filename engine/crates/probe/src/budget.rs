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
    /// The `folds` sequence: the files outside `docs/` that name a probe and
    /// state no answer, which `tools/probe/seal.sh` keeps and the guard of
    /// `probe-record.sh` passes. Empty where the file declares none.
    pub folds: Vec<String>,
    /// The `answer_keys` mapping, in declared order: for each probe, the
    /// documents an earlier session wrote in answer to its task, which the
    /// seal removes. Empty where the file declares none.
    pub answer_keys: Vec<(String, Vec<String>)>,
    /// The `layers` mapping of the additive campaign (#1774), in declared
    /// order. Empty where the file declares none.
    pub layers: Vec<Layer>,
    /// The `scenarios` mapping of the additive campaign (#1774), in declared
    /// order. Empty where the file declares none.
    pub scenarios: Vec<Scenario>,
}

/// One layer of the additive campaign (#1774): the paths it adds to plain
/// code.
///
/// `paths` are copied from the present tree. `arms` are written from
/// `tools/probe/arms/<name>/`, for a layer that holds a file the present tree
/// does not hold in that form, such as the one hook entry of the hook layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layer {
    pub name: String,
    pub paths: Vec<String>,
    pub arms: Vec<String>,
}

/// Which of the two blocks of the milestone a scenario belongs to (#1774).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Block {
    /// The scenarios that answer each layer once.
    AnswerOnce,
    /// The scenarios that measure how two layers interact.
    Interaction,
}

impl Block {
    pub const ALL: [Block; 2] = [Block::AnswerOnce, Block::Interaction];

    pub fn name(self) -> &'static str {
        match self {
            Block::AnswerOnce => "answer-once",
            Block::Interaction => "interaction",
        }
    }

    fn read(name: &str) -> Option<Block> {
        Block::ALL.into_iter().find(|block| block.name() == name)
    }
}

/// One scenario of the additive campaign (#1774): the layers it holds, and
/// the one base it is compared with. Scenario 0 is plain code, with no layer
/// and no base.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scenario {
    pub number: u32,
    pub block: Block,
    pub base: Option<u32>,
    pub layers: Vec<String>,
}

/// What a scenario's tree does with one path of one layer, against the
/// present tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hold {
    /// The scenario does not hold the layer, so the path is removed.
    Removes,
    /// The scenario holds the layer, and the path is kept from the present
    /// tree.
    Keeps,
    /// The scenario holds the layer, and the path is written from
    /// `tools/probe/arms/<layer>/`.
    Writes,
}

impl Hold {
    /// The sign `headwater probe plan --scenario <n> --delta` prints.
    pub fn sign(self) -> char {
        match self {
            Hold::Removes => '-',
            Hold::Keeps => '=',
            Hold::Writes => '+',
        }
    }
}

impl Budgets {
    /// The scenario the file declares under this number.
    pub fn scenario(&self, number: u32) -> Option<&Scenario> {
        self.scenarios
            .iter()
            .find(|scenario| scenario.number == number)
    }

    /// How the tree of a scenario differs from the present tree: one entry
    /// for each path of each layer, in declared order.
    ///
    /// A layer the scenario holds keeps each of its `paths` and writes each of
    /// its `arms`. A layer it does not hold has each path removed. One rule
    /// joins them: an `arms` file stands in for a path that another layer
    /// holds from the present tree, so where the scenario holds that other
    /// layer, the present file is kept and the stand-in is neither written
    /// nor removed. The hook layer's one-entry `settings.json` is that case,
    /// and the rest of the harness holds the whole file. `None` for a
    /// scenario the file does not declare.
    pub fn scenario_delta(&self, number: u32) -> Option<Vec<(Hold, &str, &str)>> {
        let scenario = self.scenario(number)?;
        let held = |layer: &Layer| scenario.layers.contains(&layer.name);
        let kept_by_held = |path: &str| {
            self.layers
                .iter()
                .any(|layer| held(layer) && layer.paths.iter().any(|kept| kept == path))
        };
        let mut delta = Vec::new();
        for layer in &self.layers {
            for path in &layer.paths {
                let hold = if held(layer) {
                    Hold::Keeps
                } else if kept_by_held(path) {
                    continue;
                } else {
                    Hold::Removes
                };
                delta.push((hold, layer.name.as_str(), path.as_str()));
            }
            for path in &layer.arms {
                let hold = if kept_by_held(path) {
                    continue;
                } else if held(layer) {
                    Hold::Writes
                } else {
                    Hold::Removes
                };
                delta.push((hold, layer.name.as_str(), path.as_str()));
            }
        }
        Some(delta)
    }

    pub fn of(&self, tier: Tier) -> Option<&Envelope> {
        self.tiers.iter().find(|envelope| envelope.tier == tier)
    }

    /// The answer keys one probe declares, empty for a probe that declares
    /// none.
    pub fn answer_keys(&self, probe: &str) -> &[String] {
        self.answer_keys
            .iter()
            .find(|(declared, _)| declared == probe)
            .map(|(_, paths)| paths.as_slice())
            .unwrap_or(&[])
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

        let folds = match root.get("folds") {
            None => Vec::new(),
            Some(listed) => paths(&listed.value)
                .ok_or_else(|| {
                    Unreadable::Malformed(format!("`folds` in {PATH} is not a sequence of paths"))
                })?
                .map_err(|entry| Unreadable::FoldUnsafe { entry })?,
        };

        let mut answer_keys: Vec<(String, Vec<String>)> = Vec::new();
        if let Some(declared) = root.get("answer_keys") {
            let Some(by_probe) = declared.value.as_map() else {
                return Err(Unreadable::Malformed(format!(
                    "`answer_keys` in {PATH} is not a mapping of probe to a sequence of paths"
                )));
            };
            for entry in by_probe {
                let probe = entry.key.value.as_str().to_string();
                let keys = paths(&entry.value.value)
                    .ok_or_else(|| {
                        Unreadable::Malformed(format!(
                            "the answer keys of `{probe}` in {PATH} are not a sequence of paths"
                        ))
                    })?
                    .map_err(|entry| Unreadable::AnswerKeyUnsafe {
                        probe: probe.clone(),
                        entry,
                    })?;
                answer_keys.push((probe, keys));
            }
        }

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
        let layers = match root.get("layers") {
            None => Vec::new(),
            Some(declared) => layers(&declared.value)?,
        };
        let scenarios = match root.get("scenarios") {
            None => Vec::new(),
            Some(declared) => scenarios(&declared.value, &layers)?,
        };
        Ok(Budgets {
            tiers,
            instrument,
            folds,
            answer_keys,
            layers,
            scenarios,
        })
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

/// The `layers` mapping (#1774): each layer's name, and the paths it adds to
/// plain code.
///
/// It refuses a name that cannot be a directory under `tools/probe/arms/`, a
/// layer with no path, an unsafe path, and a path that two layers claim. Two
/// paths are claimed twice when they are equal or one is inside the other.
/// The one pair it admits is an `arms` file equal to a path another layer
/// keeps from the present tree: the file stands in for part of that path, and
/// [`Budgets::scenario_delta`] keeps the present file where both are held.
fn layers(value: &Value) -> Result<Vec<Layer>, Unreadable> {
    let Some(block) = value.as_map() else {
        return Err(Unreadable::Malformed(format!(
            "`layers` in {PATH} is not a mapping of layer name to its paths"
        )));
    };
    let mut layers: Vec<Layer> = Vec::new();
    for entry in block {
        let name = entry.key.value.as_str().to_string();
        let named = !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if !named {
            return Err(Unreadable::LayerName { found: name });
        }
        let Some(fields) = entry.value.value.as_map() else {
            return Err(Unreadable::Malformed(format!(
                "the `{name}` layer in {PATH} is not a mapping of `paths` and `arms`"
            )));
        };
        let mut layer = Layer {
            name: name.clone(),
            paths: Vec::new(),
            arms: Vec::new(),
        };
        for field in fields {
            let key = field.key.value.as_str();
            let listed = paths(&field.value.value)
                .ok_or_else(|| {
                    Unreadable::Malformed(format!(
                        "`{key}` of the `{name}` layer in {PATH} is not a sequence of paths"
                    ))
                })?
                .map_err(|entry| Unreadable::LayerUnsafe {
                    layer: name.clone(),
                    entry,
                })?;
            match key {
                "paths" => layer.paths = listed,
                "arms" => layer.arms = listed,
                _ => {
                    return Err(Unreadable::Malformed(format!(
                        "the `{name}` layer in {PATH} declares `{key}`. A layer declares \
                         `paths`, copied from the present tree, and `arms`, written from \
                         `tools/probe/arms/{name}/`, and nothing else"
                    )));
                }
            }
        }
        if layer.paths.is_empty() && layer.arms.is_empty() {
            return Err(Unreadable::LayerEmpty { layer: name });
        }
        layers.push(layer);
    }

    let inside = |a: &str, b: &str| {
        a == b
            || a.strip_prefix(b).is_some_and(|rest| rest.starts_with('/'))
            || b.strip_prefix(a).is_some_and(|rest| rest.starts_with('/'))
    };
    for (index, first) in layers.iter().enumerate() {
        for second in &layers[index + 1..] {
            let entries = |layer: &'_ Layer| {
                layer
                    .paths
                    .iter()
                    .map(|path| (path.clone(), false))
                    .chain(layer.arms.iter().map(|path| (path.clone(), true)))
                    .collect::<Vec<_>>()
            };
            for (a, a_arms) in entries(first) {
                for (b, b_arms) in entries(second) {
                    // An `arms` file that stands in for a path the other layer
                    // keeps from the present tree.
                    let stands_in = a == b && a_arms != b_arms;
                    if inside(&a, &b) && !stands_in {
                        return Err(Unreadable::PathInTwoLayers {
                            first_layer: first.name.clone(),
                            first_path: a,
                            second_layer: second.name.clone(),
                            second_path: b,
                        });
                    }
                }
            }
        }
    }
    Ok(layers)
}

/// The `scenarios` mapping (#1774): each scenario's block, base and layers.
///
/// It refuses a scenario that is not a whole number, a block outside the
/// two, a layer `layers` does not declare, a file with no scenario 0, a
/// scenario 0 that holds a layer or names a base, a scenario other than 0
/// with no base or a base the file does not declare, and a scenario that
/// differs from its base by anything but exactly one added layer.
fn scenarios(value: &Value, layers: &[Layer]) -> Result<Vec<Scenario>, Unreadable> {
    let Some(block) = value.as_map() else {
        return Err(Unreadable::Malformed(format!(
            "`scenarios` in {PATH} is not a mapping of scenario number to its layers"
        )));
    };
    // The base as written, checked once every scenario is read.
    let mut declared: Vec<(Scenario, Option<String>)> = Vec::new();
    for entry in block {
        let found = entry.key.value.as_str().to_string();
        let number = match found.parse::<u32>() {
            Ok(number) if number.to_string() == found => number,
            _ => return Err(Unreadable::ScenarioName { found }),
        };
        let Some(fields) = entry.value.value.as_map() else {
            return Err(Unreadable::Malformed(format!(
                "scenario {number} in {PATH} is not a mapping of `block`, `base` and `layers`"
            )));
        };
        let written = text(fields, "block").unwrap_or_default();
        let Some(chosen) = Block::read(&written) else {
            return Err(Unreadable::ScenarioBlock {
                scenario: number,
                found: written,
            });
        };
        let Some(listed) = fields
            .get("layers")
            .and_then(|listed| listed.value.as_seq())
        else {
            return Err(Unreadable::Malformed(format!(
                "scenario {number} in {PATH} declares no `layers` sequence. Scenario 0 \
                 declares `layers: []`"
            )));
        };
        let mut held: Vec<String> = Vec::new();
        for item in listed {
            let layer = item
                .value
                .as_scalar()
                .map(|scalar| scalar.text.clone())
                .unwrap_or_default();
            if !layers.iter().any(|declared| declared.name == layer) {
                return Err(Unreadable::ScenarioUnknownLayer {
                    scenario: number,
                    layer,
                });
            }
            if !held.contains(&layer) {
                held.push(layer);
            }
        }
        if let Some(field) = fields
            .iter()
            .map(|field| field.key.value.as_str())
            .find(|key| !["block", "base", "layers"].contains(key))
        {
            return Err(Unreadable::Malformed(format!(
                "scenario {number} in {PATH} declares `{field}`. A scenario declares `block`, \
                 `base` and `layers`, and nothing else"
            )));
        }
        declared.push((
            Scenario {
                number,
                block: chosen,
                base: None,
                layers: held,
            },
            text(fields, "base"),
        ));
    }

    if !declared.is_empty() && !declared.iter().any(|(scenario, _)| scenario.number == 0) {
        return Err(Unreadable::ScenariosWithoutPlainCode);
    }
    let mut scenarios = Vec::new();
    for (mut scenario, base) in declared.iter().cloned() {
        if scenario.number == 0 {
            if let Some(layer) = scenario.layers.first() {
                return Err(Unreadable::PlainCodeHoldsLayer {
                    layer: layer.clone(),
                });
            }
            if let Some(base) = base {
                return Err(Unreadable::PlainCodeHasBase { base });
            }
            scenarios.push(scenario);
            continue;
        }
        let Some(base) = base else {
            return Err(Unreadable::ScenarioWithoutBase {
                scenario: scenario.number,
            });
        };
        let Some((under, _)) = declared
            .iter()
            .find(|(other, _)| other.number.to_string() == base)
        else {
            return Err(Unreadable::ScenarioUnknownBase {
                scenario: scenario.number,
                base,
            });
        };
        let added: Vec<String> = scenario
            .layers
            .iter()
            .filter(|layer| !under.layers.contains(layer))
            .cloned()
            .collect();
        let dropped: Vec<String> = under
            .layers
            .iter()
            .filter(|layer| !scenario.layers.contains(layer))
            .cloned()
            .collect();
        if added.len() != 1 || !dropped.is_empty() {
            return Err(Unreadable::ScenarioNotOneLayer {
                scenario: scenario.number,
                base: under.number,
                added,
                dropped,
            });
        }
        scenario.base = Some(under.number);
        scenarios.push(scenario);
    }
    Ok(scenarios)
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
/// would remove the copy itself or something outside it. An entry with a
/// control character is refused too: `headwater probe plan --delta` prints one
/// entry per line, so a line break inside one entry would print two, and the
/// script would remove a path nobody declared (#1472, verify round 4).
fn ablation_entry_is_safe(entry: &str) -> bool {
    !entry.is_empty()
        && !entry.starts_with('/')
        && !entry.chars().any(char::is_control)
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
    FoldUnsafe {
        entry: String,
    },
    AnswerKeyUnsafe {
        probe: String,
        entry: String,
    },
    LayerName {
        found: String,
    },
    LayerUnsafe {
        layer: String,
        entry: String,
    },
    LayerEmpty {
        layer: String,
    },
    PathInTwoLayers {
        first_layer: String,
        first_path: String,
        second_layer: String,
        second_path: String,
    },
    ScenarioName {
        found: String,
    },
    ScenarioBlock {
        scenario: u32,
        found: String,
    },
    ScenarioUnknownLayer {
        scenario: u32,
        layer: String,
    },
    ScenariosWithoutPlainCode,
    PlainCodeHoldsLayer {
        layer: String,
    },
    PlainCodeHasBase {
        base: String,
    },
    ScenarioWithoutBase {
        scenario: u32,
    },
    ScenarioUnknownBase {
        scenario: u32,
        base: String,
    },
    ScenarioNotOneLayer {
        scenario: u32,
        base: u32,
        added: Vec<String>,
        dropped: Vec<String>,
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
                "the `{}` delta of the `{tier}` tier names `{}`. An entry is a path inside \
                 the tree, relative to its root, with no `..`, `.` or empty component and no control \
                 character, because the arm's tree is built by removing or writing it",
                arm.name(),
                entry.escape_debug()
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
                "the `{tier}` tier's ablation names `{}`. An entry is a path inside the \
                 tree, relative to its root, with no `..`, `.` or empty component and no \
                 control character, because the absent arm removes it with `rm -rf`",
                entry.escape_debug()
            ),
            Unreadable::InstrumentUnsafe { entry } => write!(
                f,
                "the `instrument` of {PATH} names `{}`. An entry is a path inside the tree, \
                 relative to its root, with no `..`, `.` or empty component and no control \
                 character, because every arm removes it with `rm -rf`",
                entry.escape_debug()
            ),
            Unreadable::FoldUnsafe { entry } => write!(
                f,
                "the `folds` of {PATH} name `{}`. An entry is a path inside the tree, \
                 relative to its root, with no `..`, `.` or empty component and no control \
                 character, because the seal and the guard compare it with a file of the \
                 workspace",
                entry.escape_debug()
            ),
            Unreadable::AnswerKeyUnsafe { probe, entry } => write!(
                f,
                "the answer keys of `{}` in {PATH} name `{}`. An entry is a path inside the \
                 tree, relative to its root, with no `..`, `.` or empty component and no \
                 control character, because the seal removes it with `rm`",
                probe.escape_debug(),
                entry.escape_debug()
            ),
            Unreadable::LayerName { found } => write!(
                f,
                "the `layers` of {PATH} name a layer `{}`. A layer name is lower-case letters, \
                 digits and hyphens, because it names the directory under `tools/probe/arms/` \
                 its written files come from and a column of the scenario delta",
                found.escape_debug()
            ),
            Unreadable::LayerUnsafe { layer, entry } => write!(
                f,
                "the `{layer}` layer in {PATH} names `{}`. An entry is a path inside the tree, \
                 relative to its root, with no `..`, `.` or empty component and no control \
                 character, because a scenario that does not hold the layer removes it with \
                 `rm -rf`",
                entry.escape_debug()
            ),
            Unreadable::LayerEmpty { layer } => write!(
                f,
                "the `{layer}` layer in {PATH} names no path under `paths` or `arms`, so a \
                 scenario that adds it adds nothing and differs from its base by no layer"
            ),
            Unreadable::PathInTwoLayers {
                first_layer,
                first_path,
                second_layer,
                second_path,
            } => write!(
                f,
                "the `{first_layer}` layer holds `{}` and the `{second_layer}` layer holds `{}` \
                 in {PATH}. A path belongs to one layer, or a scenario that adds one of the two \
                 layers adds part of the other too",
                first_path.escape_debug(),
                second_path.escape_debug()
            ),
            Unreadable::ScenarioName { found } => write!(
                f,
                "the `scenarios` of {PATH} name a scenario `{}`. A scenario is named by its \
                 number in the milestone, a whole number",
                found.escape_debug()
            ),
            Unreadable::ScenarioBlock { scenario, found } => write!(
                f,
                "scenario {scenario} in {PATH} names the block `{}`. The blocks are {}",
                found.escape_debug(),
                Block::ALL
                    .iter()
                    .map(|block| format!("`{}`", block.name()))
                    .collect::<Vec<_>>()
                    .join(" and ")
            ),
            Unreadable::ScenarioUnknownLayer { scenario, layer } => write!(
                f,
                "scenario {scenario} in {PATH} holds the layer `{}`, and `layers` declares no \
                 such layer, so nothing says which paths it adds",
                layer.escape_debug()
            ),
            Unreadable::ScenariosWithoutPlainCode => write!(
                f,
                "the `scenarios` of {PATH} declare no scenario 0. Scenario 0 is plain code, \
                 the tree every scenario's layers are added to"
            ),
            Unreadable::PlainCodeHoldsLayer { layer } => write!(
                f,
                "scenario 0 in {PATH} holds the layer `{}`. Scenario 0 is plain code and holds \
                 no layer",
                layer.escape_debug()
            ),
            Unreadable::PlainCodeHasBase { base } => write!(
                f,
                "scenario 0 in {PATH} names the base `{}`. Scenario 0 is plain code, and it is \
                 the base of others and has none",
                base.escape_debug()
            ),
            Unreadable::ScenarioWithoutBase { scenario } => write!(
                f,
                "scenario {scenario} in {PATH} names no base. Every scenario but 0 is compared \
                 with one base, and that comparison is the one the campaign registers"
            ),
            Unreadable::ScenarioUnknownBase { scenario, base } => write!(
                f,
                "scenario {scenario} in {PATH} names the base `{}`, which is not a scenario the \
                 file declares",
                base.escape_debug()
            ),
            Unreadable::ScenarioNotOneLayer {
                scenario,
                base,
                added,
                dropped,
            } => {
                let listed = |layers: &[String]| {
                    if layers.is_empty() {
                        "no layer".to_string()
                    } else {
                        layers
                            .iter()
                            .map(|layer| format!("`{layer}`"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                };
                write!(
                    f,
                    "scenario {scenario} in {PATH} adds {} to its base {base} and drops {}. A \
                     scenario differs from its base by exactly one added layer, or its \
                     difference from the base measures more than one thing",
                    listed(added),
                    listed(dropped)
                )
            }
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
    fn the_folds_and_the_answer_keys_read_in_every_sequence_form() {
        // `tools/probe/seal.sh` reads both lists from here (#1472, clause
        // 8). Its hand-written readers took a two-space block of folds and a
        // flow sequence of keys alone, so any other form read as no list.
        for (form, written) in [
            (
                "block",
                "folds:\n  - .headwater/nav.yml\n  - tools/x.spec\nanswer_keys:\n  P-1:\n  - docs/a.md\n  - docs/b.md\n  P-2: [docs/c.md]\n",
            ),
            (
                "four-space block",
                "folds:\n    - .headwater/nav.yml\n    - tools/x.spec\nanswer_keys:\n    P-1:\n        - docs/a.md\n        - docs/b.md\n    P-2:\n        - docs/c.md\n",
            ),
            (
                "flow",
                "folds: [.headwater/nav.yml, \"tools/x.spec\"] # the folds\nanswer_keys: {P-1: [docs/a.md, docs/b.md], P-2: [docs/c.md]}\n",
            ),
        ] {
            let budgets = Budgets::read(&format!("{written}{GOOD}")).expect(form);
            assert_eq!(budgets.folds, vec![".headwater/nav.yml", "tools/x.spec"], "{form}");
            assert_eq!(
                budgets.answer_keys,
                vec![
                    (
                        "P-1".to_string(),
                        vec!["docs/a.md".to_string(), "docs/b.md".to_string()]
                    ),
                    ("P-2".to_string(), vec!["docs/c.md".to_string()]),
                ],
                "{form}"
            );
            assert_eq!(budgets.answer_keys("P-2"), ["docs/c.md".to_string()]);
            assert!(budgets.answer_keys("P-3").is_empty(), "{form}");
            // A prefix of a declared probe is another probe, and the seal
            // must not remove a key that belongs to the longer one.
            assert!(budgets.answer_keys("P").is_empty(), "{form}");
            assert!(budgets.answer_keys("P-").is_empty(), "{form}");
        }
    }

    #[test]
    fn a_file_with_no_folds_and_no_answer_keys_reads_as_two_empty_lists() {
        let budgets = Budgets::read(GOOD).expect("reads");
        assert!(budgets.folds.is_empty());
        assert!(budgets.answer_keys.is_empty());
    }

    #[test]
    fn a_fold_or_an_answer_key_that_leaves_the_tree_is_refused() {
        // The seal runs `rm` on each answer key in a workspace, and a fold is
        // a path the guard compares. Either refuses the whole file.
        assert_eq!(
            Budgets::read(&format!("folds: [tools/x.spec, ../x]\n{GOOD}")),
            Err(Unreadable::FoldUnsafe {
                entry: "../x".to_string()
            })
        );
        assert_eq!(
            Budgets::read(&format!(
                "answer_keys:\n  P-1: [docs/a.md, /etc/passwd]\n{GOOD}"
            )),
            Err(Unreadable::AnswerKeyUnsafe {
                probe: "P-1".to_string(),
                entry: "/etc/passwd".to_string()
            })
        );
        // The check reads every probe, not only the first: a second probe's
        // unsafe key is refused while the first probe holds only safe ones
        // (#1552).
        assert_eq!(
            Budgets::read(&format!(
                "answer_keys:\n  P-1: [docs/a.md]\n  P-2: [../x]\n{GOOD}"
            )),
            Err(Unreadable::AnswerKeyUnsafe {
                probe: "P-2".to_string(),
                entry: "../x".to_string()
            })
        );
    }

    #[test]
    fn an_unsafe_answer_key_of_every_shape_is_refused_in_a_middle_probe() {
        // A probe between two safe ones, so a check of the first probe alone
        // or of the last probe alone passes the file (#1552). Each entry is
        // one shape `ablation_entry_is_safe` refuses: a `..` component, an
        // absolute path, an empty entry, a `.` component, an empty
        // component, and a control character.
        for (written, entry) in [
            ("../x", "../x"),
            ("/etc/passwd", "/etc/passwd"),
            ("\"\"", ""),
            ("docs/./a.md", "docs/./a.md"),
            ("docs//a.md", "docs//a.md"),
            ("\"docs/a.md\\n- CLAUDE.md\"", "docs/a.md\n- CLAUDE.md"),
        ] {
            let source = format!(
                "answer_keys:\n  P-1: [docs/a.md]\n  P-2: [docs/b.md, {written}]\n  P-3: [docs/c.md]\n{GOOD}"
            );
            assert_eq!(
                Budgets::read(&source),
                Err(Unreadable::AnswerKeyUnsafe {
                    probe: "P-2".to_string(),
                    entry: entry.to_string()
                }),
                "{written}"
            );
        }
    }

    #[test]
    fn folds_and_answer_keys_of_the_wrong_shape_are_refused() {
        for written in [
            "folds: tools/x.spec\n",
            "folds: {a: b}\n",
            "answer_keys: [docs/a.md]\n",
            "answer_keys:\n  P-1: docs/a.md\n",
        ] {
            assert!(
                matches!(
                    Budgets::read(&format!("{written}{GOOD}")),
                    Err(Unreadable::Malformed(_))
                ),
                "{written}"
            );
        }
    }

    #[test]
    fn an_entry_with_a_control_character_is_refused_in_every_list() {
        // `headwater probe plan --delta` prints one entry per line, so an
        // entry that holds a line break would print as two, and
        // `tools/probe/ablate.sh` would remove the second one as well
        // (#1472, verify round 4). YAML's double-quoted `\n` is the line
        // break, and a tab is a control character too.
        let instrument = GOOD.replace(
            "[docs/probes, docs/probe-runs]",
            "[docs/probes, \"docs/probe-runs\\n- CLAUDE.md\"]",
        );
        assert_eq!(
            Budgets::read(&instrument),
            Err(Unreadable::InstrumentUnsafe {
                entry: "docs/probe-runs\n- CLAUDE.md".to_string()
            })
        );
        let ablation = GOOD.replace(
            "ablation: [CLAUDE.md, .claude, .githooks, .headwater]",
            "ablation: [CLAUDE.md, \".claude\\t\", .githooks, .headwater]",
        );
        assert_eq!(
            Budgets::read(&ablation),
            Err(Unreadable::AblationUnsafe {
                tier: "campaign",
                entry: ".claude\t".to_string()
            })
        );
        let component = LAYER.replace(
            "      no-hook: [.claude/hooks/intent.sh]\n",
            "      no-hook: [\".claude/hooks/intent.sh\\n- CLAUDE.md\"]\n",
        );
        let refused = Budgets::read(&component);
        assert_eq!(
            refused,
            Err(Unreadable::ComponentUnsafe {
                tier: "campaign",
                arm: Arm::NoHook,
                entry: ".claude/hooks/intent.sh\n- CLAUDE.md".to_string()
            })
        );
        let message = refused.expect_err("refused").to_string();
        assert!(
            !message.contains('\n'),
            "the refusal prints the entry escaped, on one line: {message}"
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
            (
                "      no-hook: [.claude/hooks/intent.sh]\n",
                "../x",
                Arm::NoHook,
            ),
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

    // The additive campaign of #1774: six layers and the rest of the harness,
    // and scenarios 0 to 13 as the milestone's issues #1780 to #1793 state
    // them.
    const ADDITIVE: &str = "\
layers:
  docs:
    paths: [docs, engine/crates/census/fixtures/corpus.census]
  engine:
    paths: [.headwater, engine/target/dev-release/headwater]
  claude-md:
    paths: [CLAUDE.md]
  skills:
    paths: [.claude/skills/headwater-orient, .agents/skills]
  hook:
    paths: [.claude/hooks/intent.sh, .claude/hooks/lib.sh]
    arms: [.claude/settings.json]
  mcp:
    arms: [.mcp.json]
  harness:
    paths: [.claude/settings.json, .githooks, AGENTS.md]
scenarios:
  0:
    block: answer-once
    layers: []
  1:
    block: answer-once
    base: 0
    layers: [docs]
  2:
    block: interaction
    base: 1
    layers: [docs, claude-md]
  3:
    block: answer-once
    base: 1
    layers: [docs, engine]
  4:
    block: answer-once
    base: 3
    layers: [docs, engine, skills]
  5:
    block: answer-once
    base: 3
    layers: [docs, engine, hook]
  6:
    block: answer-once
    base: 3
    layers: [docs, engine, mcp]
  7:
    block: interaction
    base: 5
    layers: [docs, engine, hook, mcp]
  8:
    block: interaction
    base: 4
    layers: [docs, engine, claude-md, skills]
  9:
    block: interaction
    base: 5
    layers: [docs, engine, claude-md, hook]
  10:
    block: interaction
    base: 5
    layers: [docs, engine, skills, hook]
  11:
    block: answer-once
    base: 10
    layers: [docs, engine, claude-md, skills, hook]
  12:
    block: answer-once
    base: 11
    layers: [docs, engine, claude-md, skills, hook, mcp]
  13:
    block: answer-once
    base: 11
    layers: [docs, engine, claude-md, skills, hook, harness]
";

    fn additive() -> String {
        format!("{GOOD}{ADDITIVE}")
    }

    #[test]
    fn a_file_with_no_layers_declares_no_scenario() {
        let budgets = Budgets::read(GOOD).expect("reads");
        assert!(budgets.layers.is_empty());
        assert!(budgets.scenarios.is_empty());
        assert_eq!(budgets.scenario_delta(0), None);
    }

    #[test]
    fn fourteen_scenarios_read_with_their_layers_blocks_and_bases() {
        let budgets = Budgets::read(&additive()).expect("reads");
        assert_eq!(
            budgets
                .layers
                .iter()
                .map(|layer| layer.name.as_str())
                .collect::<Vec<_>>(),
            [
                "docs",
                "engine",
                "claude-md",
                "skills",
                "hook",
                "mcp",
                "harness"
            ]
        );
        let hook = &budgets.layers[4];
        assert_eq!(
            hook.paths,
            [".claude/hooks/intent.sh", ".claude/hooks/lib.sh"]
        );
        assert_eq!(hook.arms, [".claude/settings.json"]);
        assert_eq!(budgets.scenarios.len(), 14);
        let bases: Vec<Option<u32>> = budgets.scenarios.iter().map(|s| s.base).collect();
        assert_eq!(
            bases,
            [
                None,
                Some(0),
                Some(1),
                Some(1),
                Some(3),
                Some(3),
                Some(3),
                Some(5),
                Some(4),
                Some(5),
                Some(5),
                Some(10),
                Some(11),
                Some(11)
            ]
        );
        let interaction: Vec<u32> = budgets
            .scenarios
            .iter()
            .filter(|s| s.block == Block::Interaction)
            .map(|s| s.number)
            .collect();
        assert_eq!(interaction, [2, 7, 8, 9, 10]);
        assert!(budgets.scenario(0).expect("plain code").layers.is_empty());
        assert_eq!(
            budgets.scenario(13).expect("13").layers,
            ["docs", "engine", "claude-md", "skills", "hook", "harness"]
        );
    }

    #[test]
    fn plain_code_removes_the_path_of_every_layer() {
        let budgets = Budgets::read(&additive()).expect("reads");
        let delta = budgets.scenario_delta(0).expect("declared");
        assert!(
            delta.iter().all(|(hold, _, _)| *hold == Hold::Removes),
            "{delta:?}"
        );
        for path in [
            "docs",
            ".headwater",
            "engine/target/dev-release/headwater",
            "CLAUDE.md",
            ".agents/skills",
            ".claude/hooks/intent.sh",
            ".claude/settings.json",
            ".mcp.json",
            "AGENTS.md",
            ".githooks",
        ] {
            assert!(
                delta.iter().any(|(_, _, p)| *p == path),
                "{path}: {delta:?}"
            );
        }
    }

    #[test]
    fn the_hook_layer_writes_its_one_entry_and_the_harness_keeps_the_whole_file() {
        let budgets = Budgets::read(&additive()).expect("reads");
        let five = budgets.scenario_delta(5).expect("declared");
        assert!(
            five.contains(&(Hold::Writes, "hook", ".claude/settings.json")),
            "{five:?}"
        );
        assert!(
            five.contains(&(Hold::Removes, "harness", ".claude/settings.json")),
            "{five:?}"
        );
        assert!(
            five.contains(&(Hold::Keeps, "hook", ".claude/hooks/intent.sh")),
            "{five:?}"
        );
        assert!(
            five.contains(&(Hold::Keeps, "engine", ".headwater")),
            "{five:?}"
        );
        assert!(
            five.contains(&(Hold::Removes, "skills", ".agents/skills")),
            "{five:?}"
        );
        let thirteen = budgets.scenario_delta(13).expect("declared");
        assert!(
            thirteen.contains(&(Hold::Keeps, "harness", ".claude/settings.json")),
            "{thirteen:?}"
        );
        assert!(
            !thirteen
                .iter()
                .any(|(hold, _, path)| *path == ".claude/settings.json" && *hold != Hold::Keeps),
            "the stand-in is neither written nor removed where the whole file is held: \
             {thirteen:?}"
        );
        let six = budgets.scenario_delta(6).expect("declared");
        assert!(six.contains(&(Hold::Writes, "mcp", ".mcp.json")), "{six:?}");
        assert_eq!(budgets.scenario_delta(14), None);
    }

    #[test]
    fn a_scenario_that_differs_from_its_base_by_more_or_less_than_one_layer_is_refused() {
        for (from, to, scenario, base, added, dropped) in [
            // Two layers added: 4 against 1.
            (
                "    base: 3\n    layers: [docs, engine, skills]\n",
                "    base: 1\n    layers: [docs, engine, skills]\n",
                4,
                1,
                vec!["engine", "skills"],
                vec![],
            ),
            // One added and one dropped: 2 against 3.
            (
                "    base: 1\n    layers: [docs, claude-md]\n",
                "    base: 3\n    layers: [docs, claude-md]\n",
                2,
                3,
                vec!["claude-md"],
                vec!["engine"],
            ),
            // None added: 6 against 3 with 3's own layers.
            (
                "    base: 3\n    layers: [docs, engine, mcp]\n",
                "    base: 3\n    layers: [docs, engine]\n",
                6,
                3,
                vec![],
                vec![],
            ),
        ] {
            let source = additive().replace(from, to);
            assert_ne!(source, additive(), "{from}");
            let refused = Budgets::read(&source);
            assert_eq!(
                refused,
                Err(Unreadable::ScenarioNotOneLayer {
                    scenario,
                    base,
                    added: added.iter().map(|l| l.to_string()).collect(),
                    dropped: dropped.iter().map(|l| l.to_string()).collect(),
                }),
                "{to}"
            );
            let message = refused.expect_err("refused").to_string();
            assert!(
                message.contains(&format!("scenario {scenario}")),
                "{message}"
            );
        }
    }

    #[test]
    fn a_scenario_that_names_an_undeclared_layer_is_refused() {
        let source = additive().replace(
            "    base: 3\n    layers: [docs, engine, mcp]\n",
            "    base: 3\n    layers: [docs, engine, lsp]\n",
        );
        assert_eq!(
            Budgets::read(&source),
            Err(Unreadable::ScenarioUnknownLayer {
                scenario: 6,
                layer: "lsp".into()
            })
        );
    }

    #[test]
    fn a_path_two_layers_both_claim_is_refused() {
        for (from, to, first, second) in [
            // The same path.
            (
                "    paths: [CLAUDE.md]\n",
                "    paths: [CLAUDE.md, AGENTS.md]\n",
                ("claude-md", "AGENTS.md"),
                ("harness", "AGENTS.md"),
            ),
            // One path inside another.
            (
                "    paths: [CLAUDE.md]\n",
                "    paths: [CLAUDE.md, .claude/hooks]\n",
                ("claude-md", ".claude/hooks"),
                ("hook", ".claude/hooks/intent.sh"),
            ),
            // Two stand-ins for one path.
            (
                "    arms: [.mcp.json]\n",
                "    arms: [.mcp.json, .claude/settings.json]\n",
                ("hook", ".claude/settings.json"),
                ("mcp", ".claude/settings.json"),
            ),
            // A stand-in inside a path another layer holds whole.
            (
                "    arms: [.mcp.json]\n",
                "    arms: [.mcp.json, docs/x.md]\n",
                ("docs", "docs"),
                ("mcp", "docs/x.md"),
            ),
        ] {
            let source = additive().replace(from, to);
            assert_eq!(
                Budgets::read(&source),
                Err(Unreadable::PathInTwoLayers {
                    first_layer: first.0.into(),
                    first_path: first.1.into(),
                    second_layer: second.0.into(),
                    second_path: second.1.into(),
                }),
                "{to}"
            );
        }
    }

    #[test]
    fn a_layer_path_that_leaves_the_tree_is_refused() {
        for entry in ["../x", "/etc", "a/./b", ""] {
            let source = additive().replace(
                "    paths: [CLAUDE.md]\n",
                &format!("    paths: [CLAUDE.md, \"{entry}\"]\n"),
            );
            assert_eq!(
                Budgets::read(&source),
                Err(Unreadable::LayerUnsafe {
                    layer: "claude-md".into(),
                    entry: entry.into()
                }),
                "{entry:?}"
            );
        }
        let source = additive().replace("    arms: [.mcp.json]\n", "    arms: [../.mcp.json]\n");
        assert_eq!(
            Budgets::read(&source),
            Err(Unreadable::LayerUnsafe {
                layer: "mcp".into(),
                entry: "../.mcp.json".into()
            })
        );
    }

    #[test]
    fn plain_code_that_holds_a_layer_or_names_a_base_is_refused() {
        let holds = additive().replace(
            "  0:\n    block: answer-once\n    layers: []\n",
            "  0:\n    block: answer-once\n    layers: [docs]\n",
        );
        assert_eq!(
            Budgets::read(&holds),
            Err(Unreadable::PlainCodeHoldsLayer {
                layer: "docs".into()
            })
        );
        let based = additive().replace(
            "  0:\n    block: answer-once\n    layers: []\n",
            "  0:\n    block: answer-once\n    base: 1\n    layers: []\n",
        );
        assert_eq!(
            Budgets::read(&based),
            Err(Unreadable::PlainCodeHasBase { base: "1".into() })
        );
        let missing = additive().replace("  0:\n    block: answer-once\n    layers: []\n", "");
        assert_eq!(
            Budgets::read(&missing),
            Err(Unreadable::ScenariosWithoutPlainCode)
        );
    }

    #[test]
    fn a_scenario_with_no_base_or_an_undeclared_base_or_block_is_refused() {
        let no_base = additive().replace(
            "  1:\n    block: answer-once\n    base: 0\n",
            "  1:\n    block: answer-once\n",
        );
        assert_eq!(
            Budgets::read(&no_base),
            Err(Unreadable::ScenarioWithoutBase { scenario: 1 })
        );
        let unknown = additive().replace(
            "    base: 11\n    layers: [docs, engine, claude-md, skills, hook, mcp]\n",
            "    base: 20\n    layers: [docs, engine, claude-md, skills, hook, mcp]\n",
        );
        assert_eq!(
            Budgets::read(&unknown),
            Err(Unreadable::ScenarioUnknownBase {
                scenario: 12,
                base: "20".into()
            })
        );
        let block = additive().replace(
            "  2:\n    block: interaction\n",
            "  2:\n    block: pairwise\n",
        );
        assert_eq!(
            Budgets::read(&block),
            Err(Unreadable::ScenarioBlock {
                scenario: 2,
                found: "pairwise".into()
            })
        );
        let named = additive().replace("  13:\n", "  thirteen:\n");
        assert_eq!(
            Budgets::read(&named),
            Err(Unreadable::ScenarioName {
                found: "thirteen".into()
            })
        );
    }

    #[test]
    fn a_layer_with_no_path_or_a_name_that_is_not_a_directory_is_refused() {
        let empty = additive().replace("    arms: [.mcp.json]\n", "    arms: []\n");
        assert_eq!(
            Budgets::read(&empty),
            Err(Unreadable::LayerEmpty {
                layer: "mcp".into()
            })
        );
        let named = additive().replace("  mcp:\n    arms:", "  MCP server:\n    arms:");
        assert_eq!(
            Budgets::read(&named),
            Err(Unreadable::LayerName {
                found: "MCP server".into()
            })
        );
    }
}
