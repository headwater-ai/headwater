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
            // A scenario's tree is plain code with layers added, and plain code
            // lacks `docs/`. The plan's examined-target rule reads this, and it
            // would refuse every document-keyed probe in scenario 0. HW-DR-0111
            // keeps those probes in the set, with scenario 0 at zero by
            // construction as the floor scenario 1 is compared with, so a
            // scenario arm removes nothing as the rule reads it (#1774).
            Arm::Scenario(_) => &[],
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
    /// The `tooling` sequence of the additive campaign (#1872): the paths of
    /// this repository's own tooling, which no layer holds and every scenario
    /// removes, scenario 6 included. `headwater init` ships none of them, so
    /// a scenario that kept one would measure this repository rather than a
    /// layer. Empty where the file declares none.
    pub tooling: Vec<String>,
    /// The `scenarios` mapping of the additive campaign (#1774), in declared
    /// order. Empty where the file declares none.
    pub scenarios: Vec<Scenario>,
    /// The `sets` mapping of the additive campaign (#1774), in declared order:
    /// each set's name and the probe identifiers it holds. `headwater probe
    /// plan --set <name>` selects one. Empty where the file declares none.
    pub sets: Vec<(String, Vec<String>)>,
    /// The `oracles` mapping (#1834), in declared order: each command the
    /// repository declares as the oracle of a code artifact. A `patched` probe
    /// grades against `test:<name>` of a `cargo-test` oracle or
    /// `command:<name>` of an `exit` oracle, and against no other name of
    /// either prefix. Empty where the file declares none.
    pub oracles: Vec<Oracle>,
}

/// How the recorder reads the output of a command oracle (#1834).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OracleParser {
    /// `cargo test` output: each failed case, a build that failed, and the
    /// whole oracle when the command exits non-zero or no case passed.
    CargoTest,
    /// The exit status alone: a non-zero exit fails the oracle.
    Exit,
}

impl OracleParser {
    /// The name `.headwater/probe.yml` declares the parser by.
    pub fn name(self) -> &'static str {
        match self {
            OracleParser::CargoTest => "cargo-test",
            OracleParser::Exit => "exit",
        }
    }

    /// The prefix of the findings the parser writes, and of the `oracle:`
    /// value a `patched` probe declares to grade against it.
    pub fn prefix(self) -> &'static str {
        match self {
            OracleParser::CargoTest => "test",
            OracleParser::Exit => "command",
        }
    }

    fn read(name: &str) -> Option<OracleParser> {
        match name {
            "cargo-test" => Some(OracleParser::CargoTest),
            "exit" => Some(OracleParser::Exit),
            _ => None,
        }
    }
}

/// One command the repository declares as the oracle of a code artifact
/// (#1834).
///
/// The recorder runs `command` once per session in the grading copy, which
/// holds every artifact the session produced, when a produced artifact matches
/// one of `paths`. `holds` names the files the seal removes from every
/// workspace and the grading copy keeps, such as a hidden test that names its
/// probe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Oracle {
    pub name: String,
    pub paths: Vec<String>,
    pub command: String,
    pub parser: OracleParser,
    pub holds: Vec<String>,
}

impl Oracle {
    /// The finding the whole oracle writes when it fails, which is also the
    /// `oracle:` value a `patched` probe declares to grade against it.
    pub fn finding(&self) -> String {
        format!("{}:{}", self.parser.prefix(), self.name)
    }
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
    /// The scenario declares `exception: full-stack` (#1872, HW-DR-0112): it
    /// holds every declared layer and adds more than one to its base. One
    /// scenario at most declares it.
    pub exception: bool,
}

/// The one value the `exception` field of a scenario takes.
pub const FULL_STACK: &str = "full-stack";

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
            // Two layers never hold one path from the present tree, because
            // `layers` refuses it, so a path here is kept or removed alone.
            for path in &layer.paths {
                let hold = if held(layer) {
                    Hold::Keeps
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
        // The seal deletes the line of each answer key it removes, so a
        // sealed declaration whose every key it removed holds `answer_keys:`
        // and no entry. That reads as no answer key, because the recorder
        // reads the sealed tree's declaration for its command oracles
        // (#1834). Any other scalar is still refused.
        let emptied = |value: &Value| {
            value
                .as_scalar()
                .is_some_and(headwater_yaml::core_schema::as_null)
        };
        if let Some(declared) = root.get("answer_keys").filter(|d| !emptied(&d.value)) {
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
        // A scenario arm names a scenario this file declares, or its tree is
        // a delta nobody wrote (#1774).
        for envelope in &tiers {
            for arm in &envelope.arms {
                if let Arm::Scenario(number) = arm {
                    if !scenarios.iter().any(|scenario| scenario.number == *number) {
                        return Err(Unreadable::ScenarioArmUndeclared {
                            tier: envelope.tier.name(),
                            arm: *arm,
                        });
                    }
                }
            }
        }
        let sets = match root.get("sets") {
            None => Vec::new(),
            Some(declared) => sets(&declared.value)?,
        };
        let oracles = match root.get("oracles") {
            None => Vec::new(),
            Some(declared) => oracles(&declared.value)?,
        };
        Ok(Budgets {
            tiers,
            instrument,
            folds,
            answer_keys,
            layers,
            tooling: Vec::new(),
            scenarios,
            sets,
            oracles,
        })
    }

    /// The probes a declared set holds, or `None` for a set the file does not
    /// declare.
    pub fn set(&self, name: &str) -> Option<&[String]> {
        self.sets
            .iter()
            .find(|(declared, _)| declared == name)
            .map(|(_, members)| members.as_slice())
    }
}

/// The `sets` mapping (#1774): each set's name and its probe identifiers.
///
/// A set is the probes a round of the additive campaign runs, and the
/// held-out set is held out only while no probe of it is in the tuning set.
/// So an identifier in two sets is refused, and so is an empty set, which
/// plans a round of nothing. Whether each identifier names a probe document
/// is the plan's question, because only the plan reads the corpus.
/// The `oracles` mapping (#1834).
///
/// It refuses a name outside `[a-z0-9-]+`, the reserved name `build`, an
/// unknown parser, an empty command or one with a control character, no
/// `paths`, and a path that leaves the tree. The YAML loader refuses a
/// duplicate name before anything reaches here.
fn oracles(value: &Value) -> Result<Vec<Oracle>, Unreadable> {
    let Some(block) = value.as_map() else {
        return Err(Unreadable::Malformed(format!(
            "`oracles` in {PATH} is not a mapping of oracle name to its declaration"
        )));
    };
    let mut oracles: Vec<Oracle> = Vec::new();
    for entry in block {
        let name = entry.key.value.as_str().to_string();
        if name.is_empty()
            || !name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            || name == "build"
        {
            return Err(Unreadable::OracleName { found: name });
        }
        let Some(fields) = entry.value.value.as_map() else {
            return Err(Unreadable::Malformed(format!(
                "the `{name}` oracle in {PATH} is not a mapping of `paths`, `command`, `parser` \
                 and `holds`"
            )));
        };
        let parser = text(fields, "parser").unwrap_or_default();
        let Some(parser) = OracleParser::read(&parser) else {
            return Err(Unreadable::OracleParser {
                oracle: name,
                found: parser,
            });
        };
        let command = text(fields, "command").unwrap_or_default();
        if command.trim().is_empty() || command.chars().any(char::is_control) {
            return Err(Unreadable::OracleCommand {
                oracle: name,
                found: command,
            });
        }
        let paths_of = |key: &str| -> Result<Vec<String>, Unreadable> {
            match fields.get(key) {
                None => Ok(Vec::new()),
                Some(listed) => paths(&listed.value)
                    .ok_or_else(|| {
                        Unreadable::Malformed(format!(
                            "the `{key}` of the `{name}` oracle in {PATH} is not a sequence of \
                             paths"
                        ))
                    })?
                    .map_err(|entry| Unreadable::OracleUnsafe {
                        oracle: name.clone(),
                        entry,
                    }),
            }
        };
        let patterns = paths_of("paths")?;
        if patterns.is_empty() {
            return Err(Unreadable::OraclePathless { oracle: name });
        }
        let holds = paths_of("holds")?;
        oracles.push(Oracle {
            name,
            paths: patterns,
            command,
            parser,
            holds,
        });
    }
    Ok(oracles)
}

fn sets(value: &Value) -> Result<Vec<(String, Vec<String>)>, Unreadable> {
    let Some(block) = value.as_map() else {
        return Err(Unreadable::Malformed(format!(
            "`sets` in {PATH} is not a mapping of set name to a sequence of probe identifiers"
        )));
    };
    let mut sets: Vec<(String, Vec<String>)> = Vec::new();
    for entry in block {
        let name = entry.key.value.as_str().to_string();
        let Some(items) = entry.value.value.as_seq() else {
            return Err(Unreadable::Malformed(format!(
                "the `{}` set in {PATH} is not a sequence of probe identifiers",
                name.escape_debug()
            )));
        };
        let mut members: Vec<String> = Vec::new();
        for item in items {
            let probe = item
                .value
                .as_scalar()
                .map(|scalar| scalar.text.clone())
                .unwrap_or_default();
            if probe.is_empty() || probe.chars().any(|c| c.is_control() || c.is_whitespace()) {
                return Err(Unreadable::Malformed(format!(
                    "the `{}` set in {PATH} holds `{}`, which is not a probe identifier",
                    name.escape_debug(),
                    probe.escape_debug()
                )));
            }
            if let Some((other, _)) = sets
                .iter()
                .find(|(_, held)| held.iter().any(|held| held == &probe))
            {
                return Err(Unreadable::SetMemberInTwo {
                    probe,
                    first: other.clone(),
                    second: name,
                });
            }
            if !members.contains(&probe) {
                members.push(probe);
            }
        }
        if members.is_empty() {
            return Err(Unreadable::SetEmpty { set: name });
        }
        sets.push((name, members));
    }
    Ok(sets)
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
    // #1774: the additive tier runs scenario arms and nothing else, because
    // each scenario is compared with its declared base and a present or
    // component arm has no base. No other tier runs a scenario arm, because
    // its tree is plain code with layers added, and that is not the present
    // tree any other arm's delta is taken from.
    if let Some(arm) = arms
        .iter()
        .copied()
        .find(|arm| arm.is_scenario() != (tier == Tier::Additive))
    {
        return Err(Unreadable::ArmOutsideItsTier { tier: name, arm });
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
                exception: false,
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
    /// A scenario declares an `exception` other than `full-stack`.
    ScenarioExceptionUnknown {
        scenario: u32,
        found: String,
    },
    /// A scenario declares the exception and does not hold every layer.
    ScenarioExceptionNotFullStack {
        scenario: u32,
        missing: Vec<String>,
    },
    /// A scenario declares the exception and drops a layer of its base.
    ScenarioExceptionDrops {
        scenario: u32,
        base: u32,
        dropped: Vec<String>,
    },
    /// A second scenario declares the exception.
    ScenarioExceptionTwice {
        first: u32,
        second: u32,
    },
    /// A `tooling` path that leaves the tree.
    ToolingUnsafe {
        entry: String,
    },
    /// A `tooling` path that a layer also claims, equal or one inside the
    /// other.
    ToolingInLayer {
        layer: String,
        layer_path: String,
        tooling_path: String,
    },
    ArmOutsideItsTier {
        tier: &'static str,
        arm: Arm,
    },
    ScenarioArmUndeclared {
        tier: &'static str,
        arm: Arm,
    },
    SetEmpty {
        set: String,
    },
    SetMemberInTwo {
        probe: String,
        first: String,
        second: String,
    },
    OracleName {
        found: String,
    },
    OracleParser {
        oracle: String,
        found: String,
    },
    OracleCommand {
        oracle: String,
        found: String,
    },
    OraclePathless {
        oracle: String,
    },
    OracleUnsafe {
        oracle: String,
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
                     difference from the base measures more than one thing. The one \
                     exception is a scenario that declares `exception: {FULL_STACK}` and \
                     holds every declared layer",
                    listed(added),
                    listed(dropped)
                )
            }
            Unreadable::ScenarioExceptionUnknown { scenario, found } => write!(
                f,
                "scenario {scenario} in {PATH} declares the exception `{}`. The one exception \
                 a scenario declares is `{FULL_STACK}`",
                found.escape_debug()
            ),
            Unreadable::ScenarioExceptionNotFullStack { scenario, missing } => write!(
                f,
                "scenario {scenario} in {PATH} declares `exception: {FULL_STACK}` and does not \
                 hold {}. The exception admits more than one added layer only for the scenario \
                 that holds every declared layer",
                missing
                    .iter()
                    .map(|layer| format!("`{layer}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Unreadable::ScenarioExceptionDrops {
                scenario,
                base,
                dropped,
            } => write!(
                f,
                "scenario {scenario} in {PATH} declares `exception: {FULL_STACK}` and drops {} \
                 of its base {base}. The exception adds layers to its base and drops none",
                dropped
                    .iter()
                    .map(|layer| format!("`{layer}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Unreadable::ScenarioExceptionTwice { first, second } => write!(
                f,
                "scenarios {first} and {second} in {PATH} both declare `exception: \
                 {FULL_STACK}`. One scenario at most is the exception to the one-layer rule"
            ),
            Unreadable::ToolingUnsafe { entry } => write!(
                f,
                "`tooling` in {PATH} names `{}`, which leaves the tree",
                entry.escape_debug()
            ),
            Unreadable::ToolingInLayer {
                layer,
                layer_path,
                tooling_path,
            } => write!(
                f,
                "`tooling` in {PATH} names `{}`, and the `{layer}` layer claims `{}`. Every \
                 scenario removes a tooling path, so a layer that held it would hold nothing",
                tooling_path.escape_debug(),
                layer_path.escape_debug()
            ),
            Unreadable::ArmOutsideItsTier { tier, arm } if arm.is_scenario() => write!(
                f,
                "the `{tier}` tier names the arm `{}`. A scenario arm is plain code with layers \
                 added, and only the `additive` tier runs one",
                arm.name()
            ),
            Unreadable::ArmOutsideItsTier { tier, arm } => write!(
                f,
                "the `{tier}` tier names the arm `{}`. The `additive` tier runs scenario arms \
                 alone, `scenario-<n>`, because each scenario is compared with its declared \
                 base and this arm has none",
                arm.name()
            ),
            Unreadable::ScenarioArmUndeclared { tier, arm } => write!(
                f,
                "the `{tier}` tier names the arm `{}`, and `scenarios` in {PATH} declares no \
                 such scenario, so nothing says which layers its tree holds",
                arm.name()
            ),
            Unreadable::SetEmpty { set } => write!(
                f,
                "the `{}` set in {PATH} holds no probe, so a round over it plans nothing",
                set.escape_debug()
            ),
            Unreadable::SetMemberInTwo {
                probe,
                first,
                second,
            } => write!(
                f,
                "`{}` is in the `{}` set and the `{}` set in {PATH}. A probe is in one set, or \
                 a round over one set reads a result the other set holds out",
                probe.escape_debug(),
                first.escape_debug(),
                second.escape_debug()
            ),
            Unreadable::OracleName { found } => write!(
                f,
                "the `oracles` of {PATH} name an oracle `{}`. An oracle name is lower-case \
                 letters, digits and hyphens, and not `build`, because `test:build` is the \
                 finding of a build that failed",
                found.escape_debug()
            ),
            Unreadable::OracleParser { oracle, found } => write!(
                f,
                "the `{}` oracle in {PATH} declares the parser `{}`. A parser is `cargo-test` \
                 or `exit`",
                oracle.escape_debug(),
                found.escape_debug()
            ),
            Unreadable::OracleCommand { oracle, found } => write!(
                f,
                "the `{}` oracle in {PATH} declares the command `{}`. A command is one line \
                 that is not empty, because the recorder runs it with `sh -c`",
                oracle.escape_debug(),
                found.escape_debug()
            ),
            Unreadable::OraclePathless { oracle } => write!(
                f,
                "the `{}` oracle in {PATH} declares no `paths`, so no produced artifact would \
                 ever run it",
                oracle.escape_debug()
            ),
            Unreadable::OracleUnsafe { oracle, entry } => write!(
                f,
                "the `{}` oracle in {PATH} names `{}`. An entry is a path inside the tree, \
                 relative to its root, with no `..`, `.` or empty component and no control \
                 character, because the recorder matches it against a produced path and \
                 restores it into the grading copy",
                oracle.escape_debug(),
                entry.escape_debug()
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

    /// A sealed declaration whose every answer key the seal removed holds
    /// `answer_keys:` and no entry, and the recorder reads it for its command
    /// oracles (#1834). It reads as no answer key. A scalar is still refused.
    #[test]
    fn an_answer_keys_block_the_seal_emptied_reads_as_no_answer_key() {
        for emptied in ["answer_keys:\n", "answer_keys:\n\n# a comment\n"] {
            let budgets = Budgets::read(&format!("{emptied}{GOOD}")).expect(emptied);
            assert!(budgets.answer_keys.is_empty(), "{emptied}");
        }
        assert!(matches!(
            Budgets::read(&format!("answer_keys: docs/a.md\n{GOOD}")),
            Err(Unreadable::Malformed(_))
        ));
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

    const ORACLES: &str = "oracles:\n  unit:\n    paths: [src/**, tests/*.rs]\n    command: \
                           cargo test --test hidden\n    parser: cargo-test\n    holds: \
                           [tests/hidden.rs]\n  site:\n    paths: [site/**]\n    command: sh \
                           check.sh\n    parser: exit\n";

    #[test]
    fn the_oracles_read_in_declared_order_and_a_file_with_none_reads_empty() {
        assert!(Budgets::read(GOOD).expect("reads").oracles.is_empty());
        let budgets = Budgets::read(&format!("{ORACLES}{GOOD}")).expect("reads");
        assert_eq!(
            budgets.oracles,
            vec![
                Oracle {
                    name: "unit".to_string(),
                    paths: vec!["src/**".to_string(), "tests/*.rs".to_string()],
                    command: "cargo test --test hidden".to_string(),
                    parser: OracleParser::CargoTest,
                    holds: vec!["tests/hidden.rs".to_string()],
                },
                Oracle {
                    name: "site".to_string(),
                    paths: vec!["site/**".to_string()],
                    command: "sh check.sh".to_string(),
                    parser: OracleParser::Exit,
                    holds: Vec::new(),
                },
            ]
        );
        assert_eq!(budgets.oracles[0].finding(), "test:unit");
        assert_eq!(budgets.oracles[1].finding(), "command:site");
    }

    #[test]
    fn an_oracle_of_any_bad_shape_refuses_the_file() {
        let one = |name: &str, body: &str| format!("oracles:\n  {name}:\n{body}{GOOD}");
        let fine = "    paths: [src/**]\n    command: sh x.sh\n    parser: exit\n";
        assert!(Budgets::read(&one("ok-1", fine)).is_ok());
        for name in ["build", "Upper", "a_b", "a.b", "\"\""] {
            assert!(
                matches!(
                    Budgets::read(&one(name, fine)),
                    Err(Unreadable::OracleName { .. })
                ),
                "{name}"
            );
        }
        for (body, expected) in [
            (
                "    paths: [src/**]\n    command: sh x.sh\n    parser: pytest\n",
                "parser",
            ),
            ("    paths: [src/**]\n    command: sh x.sh\n", "parser"),
            ("    paths: [src/**]\n    parser: exit\n", "command"),
            (
                "    paths: [src/**]\n    command: \"  \"\n    parser: exit\n",
                "command",
            ),
            (
                "    paths: [src/**]\n    command: \"a\\nb\"\n    parser: exit\n",
                "command",
            ),
            ("    command: sh x.sh\n    parser: exit\n", "pathless"),
            (
                "    paths: []\n    command: sh x.sh\n    parser: exit\n",
                "pathless",
            ),
            (
                "    paths: [../src]\n    command: sh x.sh\n    parser: exit\n",
                "unsafe",
            ),
            (
                "    paths: [src/**]\n    command: sh x.sh\n    parser: exit\n    holds: \
                 [/etc/passwd]\n",
                "unsafe",
            ),
            (
                "    paths: src/**\n    command: sh x.sh\n    parser: exit\n",
                "malformed",
            ),
            ("    - src/**\n", "malformed"),
        ] {
            let read = Budgets::read(&one("x", body));
            let kind = match &read {
                Err(Unreadable::OracleParser { .. }) => "parser",
                Err(Unreadable::OracleCommand { .. }) => "command",
                Err(Unreadable::OraclePathless { .. }) => "pathless",
                Err(Unreadable::OracleUnsafe { .. }) => "unsafe",
                Err(Unreadable::Malformed(_)) => "malformed",
                _ => "other",
            };
            assert_eq!(kind, expected, "{body}: {read:?}");
            assert!(!read.unwrap_err().to_string().is_empty());
        }
        assert!(matches!(
            Budgets::read(&format!("oracles: [a]\n{GOOD}")),
            Err(Unreadable::Malformed(_))
        ));
    }

    #[test]
    fn an_oracle_declared_twice_refuses_the_file() {
        let twice = format!(
            "oracles:\n  a:\n    paths: [x]\n    command: c\n    parser: exit\n  a:\n    \
             paths: [y]\n    command: d\n    parser: exit\n{GOOD}"
        );
        // The loader refuses a duplicate key, before the oracles are read.
        assert!(
            matches!(Budgets::read(&twice), Err(Unreadable::Unparsed(_))),
            "{:?}",
            Budgets::read(&twice)
        );
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

    // The additive campaign as HW-DR-0112 cuts it (#1872): five layers, the
    // repository's own tooling in no layer, and scenarios 0 to 6, where
    // scenario 6 adds three layers to scenario 2 under the declared exception.
    const ADDITIVE: &str = "\
layers:
  docs:
    paths: [docs, engine/crates/census/fixtures/corpus.census]
  engine:
    paths: [.headwater, engine/target/dev-release/headwater]
  skills:
    paths: [.claude/skills/headwater-orient, .agents/skills]
  hooks:
    paths: [.claude/hooks/intent.sh, .claude/hooks/write.sh, .claude/hooks/lib.sh]
    arms: [.claude/settings.json]
  mcp:
    arms: [.mcp.json]
tooling: [CLAUDE.md, AGENTS.md, .githooks, .claude/commands]
scenarios:
  0:
    block: answer-once
    layers: []
  1:
    block: answer-once
    base: 0
    layers: [docs]
  2:
    block: answer-once
    base: 1
    layers: [docs, engine]
  3:
    block: answer-once
    base: 2
    layers: [docs, engine, skills]
  4:
    block: answer-once
    base: 2
    layers: [docs, engine, hooks]
  5:
    block: answer-once
    base: 2
    layers: [docs, engine, mcp]
  6:
    block: answer-once
    base: 2
    exception: full-stack
    layers: [docs, engine, skills, hooks, mcp]
";

    fn additive() -> String {
        format!("{GOOD}{ADDITIVE}")
    }

    /// `additive()` with `from` replaced by `to`, which must change it.
    fn recut(from: &str, to: &str) -> String {
        let source = additive().replace(from, to);
        assert_ne!(source, additive(), "{from:?} is not in ADDITIVE");
        source
    }

    const TOOLING: &str = "tooling: [CLAUDE.md, AGENTS.md, .githooks, .claude/commands]\n";

    #[test]
    fn a_file_with_no_layers_declares_no_scenario() {
        let budgets = Budgets::read(GOOD).expect("reads");
        assert!(budgets.layers.is_empty());
        assert!(budgets.tooling.is_empty());
        assert!(budgets.scenarios.is_empty());
        assert_eq!(budgets.scenario_delta(0), None);
    }

    #[test]
    fn seven_scenarios_read_with_their_layers_bases_and_the_one_exception() {
        let budgets = Budgets::read(&additive()).expect("reads");
        assert_eq!(
            budgets
                .layers
                .iter()
                .map(|layer| layer.name.as_str())
                .collect::<Vec<_>>(),
            ["docs", "engine", "skills", "hooks", "mcp"]
        );
        let hooks = &budgets.layers[3];
        assert_eq!(
            hooks.paths,
            [
                ".claude/hooks/intent.sh",
                ".claude/hooks/write.sh",
                ".claude/hooks/lib.sh"
            ]
        );
        assert_eq!(hooks.arms, [".claude/settings.json"]);
        assert_eq!(
            budgets.tooling,
            ["CLAUDE.md", "AGENTS.md", ".githooks", ".claude/commands"]
        );
        assert_eq!(budgets.scenarios.len(), 7);
        let bases: Vec<Option<u32>> = budgets.scenarios.iter().map(|s| s.base).collect();
        assert_eq!(
            bases,
            [None, Some(0), Some(1), Some(2), Some(2), Some(2), Some(2)]
        );
        let excepted: Vec<u32> = budgets
            .scenarios
            .iter()
            .filter(|s| s.exception)
            .map(|s| s.number)
            .collect();
        assert_eq!(excepted, [6]);
        assert!(budgets.scenario(0).expect("plain code").layers.is_empty());
        assert_eq!(
            budgets.scenario(6).expect("6").layers,
            ["docs", "engine", "skills", "hooks", "mcp"]
        );
    }

    #[test]
    fn plain_code_removes_the_path_of_every_layer_and_the_tooling() {
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
            ".agents/skills",
            ".claude/hooks/intent.sh",
            ".claude/settings.json",
            ".mcp.json",
            "CLAUDE.md",
            ".githooks",
        ] {
            assert!(
                delta.iter().any(|(_, _, p)| *p == path),
                "{path}: {delta:?}"
            );
        }
    }

    #[test]
    fn every_scenario_removes_the_tooling_and_only_a_scenario_with_hooks_writes_settings() {
        let budgets = Budgets::read(&additive()).expect("reads");
        for number in 0..=6 {
            let delta = budgets.scenario_delta(number).expect("declared");
            for path in ["CLAUDE.md", "AGENTS.md", ".githooks", ".claude/commands"] {
                assert!(
                    delta.contains(&(Hold::Removes, "tooling", path)),
                    "scenario {number} keeps {path}: {delta:?}"
                );
                assert_eq!(
                    delta.iter().filter(|(_, _, p)| *p == path).count(),
                    1,
                    "scenario {number}: {delta:?}"
                );
            }
            let settings: Vec<_> = delta
                .iter()
                .filter(|(_, _, p)| *p == ".claude/settings.json")
                .collect();
            let expected = if [4, 6].contains(&number) {
                (Hold::Writes, "hooks", ".claude/settings.json")
            } else {
                (Hold::Removes, "hooks", ".claude/settings.json")
            };
            assert_eq!(settings, [&expected], "scenario {number}");
        }
    }

    #[test]
    fn the_hooks_layer_keeps_its_scripts_and_writes_its_settings_where_it_is_held() {
        let budgets = Budgets::read(&additive()).expect("reads");
        let four = budgets.scenario_delta(4).expect("declared");
        for entry in [
            (Hold::Keeps, "hooks", ".claude/hooks/intent.sh"),
            (Hold::Keeps, "hooks", ".claude/hooks/write.sh"),
            (Hold::Writes, "hooks", ".claude/settings.json"),
            (Hold::Keeps, "engine", ".headwater"),
            (Hold::Removes, "skills", ".agents/skills"),
            (Hold::Removes, "mcp", ".mcp.json"),
        ] {
            assert!(four.contains(&entry), "{entry:?}: {four:?}");
        }
        let three = budgets.scenario_delta(3).expect("declared");
        assert!(
            three.contains(&(Hold::Removes, "hooks", ".claude/hooks/write.sh")),
            "{three:?}"
        );
        let six = budgets.scenario_delta(6).expect("declared");
        assert!(six.contains(&(Hold::Writes, "mcp", ".mcp.json")), "{six:?}");
        assert!(
            six.contains(&(Hold::Keeps, "skills", ".agents/skills")),
            "{six:?}"
        );
        assert_eq!(budgets.scenario_delta(7), None);
    }

    #[test]
    fn a_scenario_that_differs_from_its_base_by_more_or_less_than_one_layer_is_refused() {
        for (from, to, scenario, base, added, dropped) in [
            // Two layers added: 3 against 1.
            (
                "    base: 2\n    layers: [docs, engine, skills]\n",
                "    base: 1\n    layers: [docs, engine, skills]\n",
                3,
                1,
                vec!["engine", "skills"],
                vec![],
            ),
            // One added and one dropped: 5 against 3.
            (
                "    base: 2\n    layers: [docs, engine, mcp]\n",
                "    base: 3\n    layers: [docs, engine, mcp]\n",
                5,
                3,
                vec!["mcp"],
                vec!["skills"],
            ),
            // None added: 5 against 2 with 2's own layers.
            (
                "    base: 2\n    layers: [docs, engine, mcp]\n",
                "    base: 2\n    layers: [docs, engine]\n",
                5,
                2,
                vec![],
                vec![],
            ),
            // The full stack without the declared exception: three added.
            (
                "    exception: full-stack\n",
                "",
                6,
                2,
                vec!["skills", "hooks", "mcp"],
                vec![],
            ),
            // The exception adds no layer to a base that is itself.
            (
                "    base: 2\n    exception: full-stack\n",
                "    base: 6\n    exception: full-stack\n",
                6,
                6,
                vec![],
                vec![],
            ),
        ] {
            let source = recut(from, to);
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
    fn the_full_stack_exception_is_refused_wherever_one_of_its_conditions_fails() {
        let names = |layers: &[&str]| layers.iter().map(|l| l.to_string()).collect::<Vec<_>>();
        for (from, to, refusal) in [
            // Not the full stack: 2 + skills + hooks.
            (
                "    layers: [docs, engine, skills, hooks, mcp]\n",
                "    layers: [docs, engine, skills, hooks]\n",
                Unreadable::ScenarioExceptionNotFullStack {
                    scenario: 6,
                    missing: names(&["mcp"]),
                },
            ),
            // A second scenario that declares it.
            (
                "    exception: full-stack\n    layers: [docs, engine, skills, hooks, mcp]\n",
                "    exception: full-stack\n    layers: [docs, engine, skills, hooks, mcp]\n  \
                 7:\n    block: answer-once\n    base: 2\n    exception: full-stack\n    \
                 layers: [docs, engine, skills, hooks, mcp]\n",
                Unreadable::ScenarioExceptionTwice {
                    first: 6,
                    second: 7,
                },
            ),
            // It drops a layer of its base: 3 holds skills, and the
            // scenario does not.
            (
                "    base: 2\n    exception: full-stack\n    \
                 layers: [docs, engine, skills, hooks, mcp]\n",
                "    base: 3\n    exception: full-stack\n    layers: [docs, engine, hooks, mcp]\n",
                Unreadable::ScenarioExceptionDrops {
                    scenario: 6,
                    base: 3,
                    dropped: names(&["skills"]),
                },
            ),
            // Plain code declares it.
            (
                "  0:\n    block: answer-once\n    layers: []\n",
                "  0:\n    block: answer-once\n    exception: full-stack\n    layers: []\n",
                Unreadable::ScenarioExceptionNotFullStack {
                    scenario: 0,
                    missing: names(&["docs", "engine", "skills", "hooks", "mcp"]),
                },
            ),
            // An exception that is not the one declared.
            (
                "    exception: full-stack\n",
                "    exception: two-layers\n",
                Unreadable::ScenarioExceptionUnknown {
                    scenario: 6,
                    found: "two-layers".into(),
                },
            ),
        ] {
            let source = recut(from, to);
            let refused = Budgets::read(&source);
            assert_eq!(refused, Err(refusal), "{to}");
            let message = refused.expect_err("refused").to_string();
            assert!(message.contains("full-stack"), "{message}");
        }
    }

    #[test]
    fn a_tooling_path_that_a_layer_claims_or_that_leaves_the_tree_is_refused() {
        for (to, layer, layer_path, tooling_path) in [
            // The same path.
            (
                "tooling: [CLAUDE.md, .agents/skills]\n",
                "skills",
                ".agents/skills",
                ".agents/skills",
            ),
            // A tooling path inside a layer path.
            ("tooling: [CLAUDE.md, docs/x.md]\n", "docs", "docs", "docs/x.md"),
            // A layer path inside a tooling path.
            (
                "tooling: [CLAUDE.md, .claude/hooks]\n",
                "hooks",
                ".claude/hooks/intent.sh",
                ".claude/hooks",
            ),
            // An `arms` file of a layer.
            (
                "tooling: [CLAUDE.md, .claude/settings.json]\n",
                "hooks",
                ".claude/settings.json",
                ".claude/settings.json",
            ),
        ] {
            let source = recut(TOOLING, to);
            assert_eq!(
                Budgets::read(&source),
                Err(Unreadable::ToolingInLayer {
                    layer: layer.into(),
                    layer_path: layer_path.into(),
                    tooling_path: tooling_path.into(),
                }),
                "{to}"
            );
        }
        let source = recut(TOOLING, "tooling: [CLAUDE.md, ../x]\n");
        assert_eq!(
            Budgets::read(&source),
            Err(Unreadable::ToolingUnsafe {
                entry: "../x".into()
            })
        );
        let source = recut(TOOLING, "tooling: CLAUDE.md\n");
        assert!(
            matches!(Budgets::read(&source), Err(Unreadable::Malformed(_))),
            "a scalar is not a sequence of paths"
        );
        // A layer named `tooling` would print a delta line no reader could
        // tell from a tooling path.
        let source = recut("  mcp:\n    arms:", "  tooling:\n    arms:");
        assert_eq!(
            Budgets::read(&source),
            Err(Unreadable::LayerName {
                found: "tooling".into()
            })
        );
    }

    #[test]
    fn a_scenario_that_names_an_undeclared_layer_is_refused() {
        let source = recut(
            "    base: 2\n    layers: [docs, engine, mcp]\n",
            "    base: 2\n    layers: [docs, engine, lsp]\n",
        );
        assert_eq!(
            Budgets::read(&source),
            Err(Unreadable::ScenarioUnknownLayer {
                scenario: 5,
                layer: "lsp".into()
            })
        );
    }

    #[test]
    fn a_path_two_layers_both_claim_is_refused() {
        let skills = "    paths: [.claude/skills/headwater-orient, .agents/skills]\n";
        for (from, to, first, second) in [
            // The same path.
            (
                skills,
                "    paths: [.claude/skills/headwater-orient, .agents/skills, .claude/hooks/lib.sh]\n",
                ("skills", ".claude/hooks/lib.sh"),
                ("hooks", ".claude/hooks/lib.sh"),
            ),
            // One path inside another.
            (
                skills,
                "    paths: [.claude/skills/headwater-orient, .agents/skills, .claude/hooks]\n",
                ("skills", ".claude/hooks"),
                ("hooks", ".claude/hooks/intent.sh"),
            ),
            // One path inside a path an earlier layer holds.
            (
                "    paths: [.headwater, engine/target/dev-release/headwater]\n",
                "    paths: [.headwater, engine/target/dev-release/headwater, docs/x.md]\n",
                ("docs", "docs"),
                ("engine", "docs/x.md"),
            ),
            // An `arms` file equal to a path another layer keeps: since
            // #1872 no `arms` file stands in for part of another layer.
            (
                skills,
                "    paths: [.claude/skills/headwater-orient, .agents/skills, .claude/settings.json]\n",
                ("skills", ".claude/settings.json"),
                ("hooks", ".claude/settings.json"),
            ),
            // Two `arms` files for one path.
            (
                "    arms: [.mcp.json]\n",
                "    arms: [.mcp.json, .claude/settings.json]\n",
                ("hooks", ".claude/settings.json"),
                ("mcp", ".claude/settings.json"),
            ),
            // An `arms` file inside a path another layer holds whole.
            (
                "    arms: [.mcp.json]\n",
                "    arms: [.mcp.json, docs/x.md]\n",
                ("docs", "docs"),
                ("mcp", "docs/x.md"),
            ),
        ] {
            let source = recut(from, to);
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
    fn a_path_that_shares_a_name_prefix_with_another_layers_path_is_not_claimed_twice() {
        // `.claude/hooks/lib.sh.old` is beside `.claude/hooks/lib.sh`, not
        // inside it, and `.mcp.json.bak` is beside `.mcp.json`.
        let source = recut(
            "    paths: [.claude/skills/headwater-orient, .agents/skills]\n",
            "    paths: [.claude/skills/headwater-orient, .claude/hooks/lib.sh.old, .mcp.json.bak]\n",
        );
        let budgets = Budgets::read(&source).expect("reads");
        assert_eq!(
            budgets.layers[2].paths,
            [
                ".claude/skills/headwater-orient",
                ".claude/hooks/lib.sh.old",
                ".mcp.json.bak"
            ]
        );
    }

    #[test]
    fn a_layer_path_that_leaves_the_tree_is_refused() {
        for entry in ["../x", "/etc", "a/./b", ""] {
            let source = recut(
                "    paths: [.claude/skills/headwater-orient, .agents/skills]\n",
                &format!("    paths: [.agents/skills, \"{entry}\"]\n"),
            );
            assert_eq!(
                Budgets::read(&source),
                Err(Unreadable::LayerUnsafe {
                    layer: "skills".into(),
                    entry: entry.into()
                }),
                "{entry:?}"
            );
        }
        let source = recut("    arms: [.mcp.json]\n", "    arms: [../.mcp.json]\n");
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
        let holds = recut(
            "  0:\n    block: answer-once\n    layers: []\n",
            "  0:\n    block: answer-once\n    layers: [docs]\n",
        );
        assert_eq!(
            Budgets::read(&holds),
            Err(Unreadable::PlainCodeHoldsLayer {
                layer: "docs".into()
            })
        );
        let based = recut(
            "  0:\n    block: answer-once\n    layers: []\n",
            "  0:\n    block: answer-once\n    base: 1\n    layers: []\n",
        );
        assert_eq!(
            Budgets::read(&based),
            Err(Unreadable::PlainCodeHasBase { base: "1".into() })
        );
        let missing = recut("  0:\n    block: answer-once\n    layers: []\n", "");
        assert_eq!(
            Budgets::read(&missing),
            Err(Unreadable::ScenariosWithoutPlainCode)
        );
    }

    #[test]
    fn a_scenario_with_no_base_or_an_undeclared_base_or_block_is_refused() {
        let no_base = recut(
            "  1:\n    block: answer-once\n    base: 0\n",
            "  1:\n    block: answer-once\n",
        );
        assert_eq!(
            Budgets::read(&no_base),
            Err(Unreadable::ScenarioWithoutBase { scenario: 1 })
        );
        let unknown = recut(
            "    base: 2\n    layers: [docs, engine, mcp]\n",
            "    base: 20\n    layers: [docs, engine, mcp]\n",
        );
        assert_eq!(
            Budgets::read(&unknown),
            Err(Unreadable::ScenarioUnknownBase {
                scenario: 5,
                base: "20".into()
            })
        );
        let block = recut(
            "  2:\n    block: answer-once\n",
            "  2:\n    block: pairwise\n",
        );
        assert_eq!(
            Budgets::read(&block),
            Err(Unreadable::ScenarioBlock {
                scenario: 2,
                found: "pairwise".into()
            })
        );
        for found in ["six", "06", "-1"] {
            let named = recut("  6:\n", &format!("  {found}:\n"));
            assert_eq!(
                Budgets::read(&named),
                Err(Unreadable::ScenarioName {
                    found: found.into()
                }),
                "{found}"
            );
        }
    }

    #[test]
    fn a_layer_with_no_path_or_a_name_that_is_not_a_directory_is_refused() {
        let empty = recut("    arms: [.mcp.json]\n", "    arms: []\n");
        assert_eq!(
            Budgets::read(&empty),
            Err(Unreadable::LayerEmpty {
                layer: "mcp".into()
            })
        );
        let named = recut("  mcp:\n    arms:", "  MCP server:\n    arms:");
        assert_eq!(
            Budgets::read(&named),
            Err(Unreadable::LayerName {
                found: "MCP server".into()
            })
        );
    }

    // --- the additive tier, its scenario arms and its sets (#1774) ----------

    /// The declaration with an additive tier that runs `arms`, the layers and
    /// scenarios of `ADDITIVE`, and `sets` after them.
    fn additive_tier(arms: &str, sets: &str) -> String {
        format!(
            "{GOOD}  additive:\n    budget_cents: 0\n    session_cost_cents: 50\n    \
             repetitions: 55\n    max_turns: 80\n    arms: {arms}\n{ADDITIVE}{sets}"
        )
    }

    const SETS: &str = "\
sets:
  tuning: [HW-PROBE-one, HW-PROBE-two]
  held-out:
    - HW-PROBE-three
";

    #[test]
    fn a_scenario_arm_reads_back_to_the_name_it_was_written_as() {
        assert_eq!(Arm::read("scenario-3"), Some(Arm::Scenario(3)));
        assert_eq!(Arm::Scenario(3).name(), "scenario-3");
        assert_eq!(Arm::read("scenario-13"), Some(Arm::Scenario(13)));
        for unnamed in [
            "scenario-03",
            "scenario-",
            "scenario--1",
            "scenario-x",
            "scenario-+3",
        ] {
            assert_eq!(Arm::read(unnamed), None, "`{unnamed}` names no arm");
        }
        assert!(!Arm::Scenario(0).is_component());
        assert!(!Arm::ALL.contains(&Arm::Scenario(0)));
        assert_eq!(Tier::read("additive"), Some(Tier::Additive));
        assert!(!Tier::Additive.pairs_arms());
    }

    #[test]
    fn an_additive_tier_reads_its_scenario_arms_and_a_ceiling_of_zero() {
        let budgets = Budgets::read(&additive_tier(
            "[scenario-0, scenario-1, scenario-6]",
            SETS,
        ))
        .expect("reads");
        let envelope = budgets.of(Tier::Additive).expect("the additive tier");
        assert_eq!(
            envelope.arms,
            vec![Arm::Scenario(0), Arm::Scenario(1), Arm::Scenario(6)]
        );
        assert_eq!(envelope.budget, 0, "a ceiling nobody approved is zero");
        assert_eq!(envelope.repetitions, 55);
        assert!(
            envelope.removes(Arm::Scenario(0)).is_empty(),
            "the examined-target rule reads no removal for a scenario arm"
        );
        assert_eq!(
            budgets.sets,
            vec![
                (
                    "tuning".to_string(),
                    vec!["HW-PROBE-one".to_string(), "HW-PROBE-two".to_string()]
                ),
                ("held-out".to_string(), vec!["HW-PROBE-three".to_string()]),
            ]
        );
        assert_eq!(
            budgets.set("held-out"),
            Some(&["HW-PROBE-three".to_string()][..])
        );
        assert_eq!(budgets.set("final"), None);
    }

    #[test]
    fn the_additive_tier_refuses_every_arm_that_is_not_a_scenario() {
        for (arm, found) in [
            ("present", Arm::Present),
            ("absent", Arm::Absent),
            ("no-hook", Arm::NoHook),
            ("mcp", Arm::Mcp),
        ] {
            let source = additive_tier(&format!("[scenario-0, {arm}]"), "");
            assert_eq!(
                Budgets::read(&source),
                Err(Unreadable::ArmOutsideItsTier {
                    tier: "additive",
                    arm: found
                }),
                "{arm}"
            );
        }
        let refusal = Budgets::read(&additive_tier("[present]", "")).expect_err("refused");
        assert!(
            refusal.to_string().contains("scenario arms alone"),
            "{refusal}"
        );
    }

    #[test]
    fn every_other_tier_refuses_a_scenario_arm() {
        let regression = additive().replace(
            "    repetitions: 1\n    arms: [present]\n",
            "    repetitions: 1\n    arms: [scenario-1]\n",
        );
        assert_eq!(
            Budgets::read(&regression),
            Err(Unreadable::ArmOutsideItsTier {
                tier: "regression",
                arm: Arm::Scenario(1)
            })
        );
        for tier in ["campaign", "documentation"] {
            let source = additive().replacen(
                "    arms: [present, absent]\n",
                "    arms: [present, absent, scenario-1]\n",
                if tier == "campaign" { 1 } else { 2 },
            );
            // `replacen` with 2 also writes the campaign's line, and the
            // campaign is read first, so the documentation case reads the
            // campaign's refusal unless the campaign line is put back.
            let source = if tier == "documentation" {
                source.replacen(
                    "    arms: [present, absent, scenario-1]\n",
                    "    arms: [present, absent]\n",
                    1,
                )
            } else {
                source
            };
            assert_eq!(
                Budgets::read(&source),
                Err(Unreadable::ArmOutsideItsTier {
                    tier,
                    arm: Arm::Scenario(1)
                }),
                "{tier}"
            );
            let refusal = Budgets::read(&source).expect_err("refused").to_string();
            assert!(
                refusal.contains("only the `additive` tier runs one"),
                "{refusal}"
            );
        }
    }

    #[test]
    fn a_scenario_arm_the_file_does_not_declare_is_refused() {
        assert_eq!(
            Budgets::read(&additive_tier("[scenario-0, scenario-7]", "")),
            Err(Unreadable::ScenarioArmUndeclared {
                tier: "additive",
                arm: Arm::Scenario(7)
            })
        );
        // With no `scenarios` at all, scenario 0 is undeclared too.
        let source = format!(
            "{GOOD}  additive:\n    budget_cents: 0\n    session_cost_cents: 50\n    \
             repetitions: 55\n    arms: [scenario-0]\n"
        );
        assert_eq!(
            Budgets::read(&source),
            Err(Unreadable::ScenarioArmUndeclared {
                tier: "additive",
                arm: Arm::Scenario(0)
            })
        );
    }

    #[test]
    fn a_set_that_is_empty_or_shares_a_probe_with_another_is_refused() {
        assert_eq!(
            Budgets::read(&additive_tier("[scenario-0]", "sets:\n  tuning: []\n")),
            Err(Unreadable::SetEmpty {
                set: "tuning".into()
            })
        );
        assert_eq!(
            Budgets::read(&additive_tier(
                "[scenario-0]",
                "sets:\n  tuning: [HW-PROBE-one, HW-PROBE-two]\n  \
                 held-out: [HW-PROBE-three, HW-PROBE-two]\n"
            )),
            Err(Unreadable::SetMemberInTwo {
                probe: "HW-PROBE-two".into(),
                first: "tuning".into(),
                second: "held-out".into()
            })
        );
        for malformed in [
            "sets: [tuning]\n",
            "sets:\n  tuning: HW-PROBE-one\n",
            "sets:\n  tuning: [\"HW-PROBE one\"]\n",
            "sets:\n  tuning: [\"\"]\n",
        ] {
            assert!(
                matches!(
                    Budgets::read(&additive_tier("[scenario-0]", malformed)),
                    Err(Unreadable::Malformed(_))
                ),
                "{malformed}"
            );
        }
        let refusal = Budgets::read(&additive_tier(
            "[scenario-0]",
            "sets:\n  tuning: [HW-PROBE-one]\n  held-out: [HW-PROBE-one]\n",
        ))
        .expect_err("refused")
        .to_string();
        assert!(refusal.contains("A probe is in one set"), "{refusal}");
    }
}
