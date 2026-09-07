// SPDX-License-Identifier: Apache-2.0
//! `headwater conformance`: whether a consumer wired the method, rather than
//! only copied it.
//!
//! [Spec 7](../../../../docs/spec/07-distribution-and-federation.md#conformance):
//! "Vendoring content is not adoption. A consumer can hold a perfect copy of the
//! taxonomy and wire none of it." This crate is the evaluated question that
//! follows.
//!
//! # A rule is two halves in two places
//!
//! The package declares the name of a rule, the text a person reads, and the
//! remediation. This crate holds the code that decides the rule against a tree.
//! Neither half is any use alone.
//!
//! **A name this engine holds no reading for ends the run.**
//! [`SetError::NoReading`] is that refusal, and it is the same posture
//! `requires_engine` already takes one level down: a publisher declares a range,
//! [`headwater_resolve::package::sources`] reads it, and an engine outside the
//! range is refused before one source loads. An engine that skipped a rule it
//! could not read would report a level over a rule set the publisher and the
//! consumer disagree about, and the consumer would hold a green report about a
//! requirement that nothing evaluated.
//!
//! # What a level is, and the three things that stop it becoming a score
//!
//! A level is a named subset of the rules, and nothing more. It states what an
//! adopter wired up. It is not a measurement of a corpus, and this crate takes
//! no measurement of one.
//!
//! **Nothing declares a level, and a key that tried to would end the run.**
//! `waivers` is the only key the `conformance` block of the consumer
//! declaration takes, and [`waivers`] refuses any other one by name. [`Report`]
//! derives the level from the rules that passed, so there is no field an adopter
//! can write a larger number into, and a `level` key read and dropped would
//! leave an adopter holding a claim that nothing evaluated.
//!
//! **A level with no rule is refused.** [`SetError::EmptyLevel`]. A rung that
//! names no rule is a rung every repository already stands on.
//!
//! **A waiver moves the exit status and never the level.** [`LevelState::reached`]
//! reads [`Verdict::Met`] and reads no waiver at all. [`Report::gate`] is the
//! separate question a `--level` run asks, and that one honors a live waiver. So
//! a waiver buys a green gate and never a higher rung.
//!
//! # Why this file is not a taxonomy source
//!
//! A conformance rule says nothing about what a document is. It reaches no
//! kind, facet, shelf or check, so it never reaches the lock and
//! `headwater taxonomy resolve` never reads it. What makes it reproducible is
//! the pin rather than the lock. The file is a member of the published
//! artifact, so its bytes sit inside the digest a consumer pins and
//! `headwater taxonomy vendor` refuses an artifact whose bytes moved.

use headwater_census::census::{Census, Outcome};
use headwater_check::context::Date;
use headwater_lock::Lock;
use headwater_resolve::package::Consumer;
use headwater_resolve::release::{self, Divergence, ReleaseError};
use headwater_yaml::Mapping;
use std::path::Path;

pub mod json;
pub mod render;

/// The format of the rule set file. A reader that meets any other token refuses
/// rather than guessing, on the terms the lock and the release record take. It
/// compares and never orders, so it says nothing about which of the two came
/// first.
pub const FORMAT: u32 = 1;

/// The manifest key under `contents` that points at the rule set.
pub const CONTENTS_KEY: &str = "conformance";

/// The rule names this engine holds a reading for.
///
/// The list is here and not derived from the package, and that is the point. A
/// package that declares a tree rule outside this set names a reading that this
/// engine cannot perform, and the run ends rather than reporting a level over a
/// rule that nothing evaluated.
pub const READINGS: [&str; 4] = [
    "pin.current",
    "lock.current",
    "corpus.classified",
    "projections.current",
];

/// How a rule is decided.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecidedBy {
    /// This engine reads the repository and answers.
    Tree,
    /// No reading of a tree decides it, so the rule waits on a recorded
    /// attestation with an owner and a date. Spec 7: such a rule is "not
    /// silently dropped".
    Attestation,
}

impl DecidedBy {
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "tree" => Some(DecidedBy::Tree),
            "attestation" => Some(DecidedBy::Attestation),
            _ => None,
        }
    }
}

/// One rule, as the package declares it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub name: String,
    pub title: String,
    pub decided_by: DecidedBy,
    pub statement: String,
    pub remediation: String,
}

/// One rung: a name and the rules it adds to the rung below it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Level {
    pub name: String,
    pub title: String,
    /// The rules this rung adds. A rung includes every rung declared before it.
    pub rules: Vec<String>,
}

/// The rule set of one package, and the ladder over it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleSet {
    pub rules: Vec<Rule>,
    pub levels: Vec<Level>,
}

impl RuleSet {
    pub fn rule(&self, name: &str) -> Option<&Rule> {
        self.rules.iter().find(|rule| rule.name == name)
    }

    /// Every rule a rung requires, which is its own plus every earlier rung's.
    pub fn cumulative(&self, index: usize) -> Vec<String> {
        let mut out = Vec::new();
        for level in self.levels.iter().take(index + 1) {
            for name in &level.rules {
                if !out.contains(name) {
                    out.push(name.clone());
                }
            }
        }
        out
    }
}

/// What a rule set refuses to be read as.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetError {
    /// The manifest declares no rule set, so this package states no conformance
    /// requirement at all.
    Undeclared(String),
    Unreadable(String),
    Malformed(String),
    /// The declared `format` token is not the one this engine reads. [`read`]
    /// compares the raw token to [`FORMAT`] for inequality and never orders the
    /// two, so this variant carries a mismatch and no direction: a rule set at
    /// an earlier format, one at a later format, and a token that is not a
    /// number at all all arrive here. No engine need have published any of
    /// them, because a hand-edited `format` field lands here too. The message
    /// therefore says which token was found and which one this engine wants,
    /// and names no publisher.
    Format {
        found: String,
    },
    /// A tree rule whose reading this engine does not hold.
    NoReading {
        rule: String,
        package: String,
    },
    /// A rung that names no rule.
    EmptyLevel(String),
    /// A rung that names a rule the set does not declare.
    UnknownInLevel {
        level: String,
        rule: String,
    },
    /// The installed package carries a release record, and the bytes on disk
    /// no longer match it. The rule set below is read out of this same
    /// directory, so a rule set read from a diverged one is a rule set this
    /// run cannot trust to say what it is evaluated against.
    Diverged(Vec<Divergence>),
}

impl std::fmt::Display for SetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetError::Undeclared(package) => write!(
                f,
                "{package} declares no `contents.{CONTENTS_KEY}`, so it states no conformance \
                 rule and there is nothing to evaluate this repository against"
            ),
            SetError::Unreadable(why) => {
                write!(f, "the conformance rule set cannot be read: {why}")
            }
            SetError::Malformed(what) => write!(f, "the conformance rule set is malformed: {what}"),
            SetError::Format { found } => write!(
                f,
                "the conformance rule set declares format `{found}` and this engine reads \
                 {FORMAT}. Take an engine whose `requires_engine` range the package allows"
            ),
            SetError::NoReading { rule, package } => write!(
                f,
                "{package} declares the rule `{rule}` and this engine holds no reading for it. \
                 A rule this engine cannot decide is refused rather than skipped, because a \
                 level reported over it would cover a requirement that nothing evaluated. \
                 Take an engine whose `requires_engine` range the package allows"
            ),
            SetError::EmptyLevel(name) => write!(
                f,
                "the level `{name}` names no rule. A rung that nothing earns is a rung every \
                 repository already stands on"
            ),
            SetError::UnknownInLevel { level, rule } => write!(
                f,
                "the level `{level}` names the rule `{rule}` and the rule set declares no such rule"
            ),
            SetError::Diverged(divergences) => write!(
                f,
                "the installed package carries a release record, and the bytes on disk no \
                 longer match it, so the rule set this run would read out of it is not one this \
                 run can trust: {}",
                divergences
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        }
    }
}

/// Read a rule set.
pub fn read(text: &str, package: &str) -> Result<RuleSet, SetError> {
    let root = headwater_yaml::load(text)
        .map_err(|errors| SetError::Unreadable(headwater_yaml::error::render(&errors)))?;
    let map = root
        .value
        .as_map()
        .ok_or_else(|| SetError::Malformed("the root is not a mapping".to_string()))?;
    let header = map
        .get("conformance")
        .and_then(|node| node.value.as_map())
        .ok_or_else(|| SetError::Malformed("no `conformance` block".to_string()))?;

    let format = text_of(header, "format").unwrap_or_default();
    if format != FORMAT.to_string() {
        return Err(SetError::Format {
            found: format.to_string(),
        });
    }

    let mut rules = Vec::new();
    for entry in seq_of(header, "rules") {
        let name = text_of(entry, "name")
            .ok_or_else(|| SetError::Malformed("a rule names itself".to_string()))?
            .to_string();
        let decided = text_of(entry, "decided_by").ok_or_else(|| {
            SetError::Malformed(format!(
                "`{name}` states no `decided_by`, so nothing says \
                 whether a tree decides it"
            ))
        })?;
        let decided_by = DecidedBy::parse(decided).ok_or_else(|| {
            SetError::Malformed(format!(
                "`{name}` states `decided_by: {decided}`, and this engine reads `tree` \
                 and `attestation`"
            ))
        })?;
        // The refusal that keeps a report honest. A declared reading this engine
        // does not hold ends the run here, before any level is computed.
        if decided_by == DecidedBy::Tree && !READINGS.contains(&name.as_str()) {
            return Err(SetError::NoReading {
                rule: name,
                package: package.to_string(),
            });
        }
        rules.push(Rule {
            title: text_of(entry, "title").unwrap_or(&name).to_string(),
            statement: collapse(text_of(entry, "statement").unwrap_or_default()),
            remediation: collapse(text_of(entry, "remediation").unwrap_or_default()),
            decided_by,
            name,
        });
    }

    let mut levels = Vec::new();
    for entry in seq_of(header, "levels") {
        let name = text_of(entry, "name")
            .ok_or_else(|| SetError::Malformed("a level names itself".to_string()))?
            .to_string();
        let named: Vec<String> = entry
            .get("rules")
            .and_then(|node| node.value.as_seq())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.value.as_scalar())
                    .map(|scalar| scalar.text.clone())
                    .collect()
            })
            .unwrap_or_default();
        if named.is_empty() {
            return Err(SetError::EmptyLevel(name));
        }
        for rule in &named {
            if !rules.iter().any(|declared| &declared.name == rule) {
                return Err(SetError::UnknownInLevel {
                    level: name,
                    rule: rule.clone(),
                });
            }
        }
        levels.push(Level {
            title: text_of(entry, "title").unwrap_or(&name).to_string(),
            name,
            rules: named,
        });
    }

    Ok(RuleSet { rules, levels })
}

/// The rule set of the package a repository takes.
pub fn at(root: &Path, consumer: &Consumer) -> Result<RuleSet, SetError> {
    let Some((directory, manifest)) = headwater_resolve::package::located(root, &consumer.package)
    else {
        return Err(SetError::Unreadable(format!(
            "no package under `{}/` declares `{}`",
            headwater_resolve::package::PACKAGES,
            consumer.package
        )));
    };

    // The rule set is read out of this same directory below, and nothing has
    // held it to anything yet at that point. A directory that carries a
    // release record and no longer matches it is not one this run may trust
    // to say what it declares: the level definitions inside it, and not only
    // the prose a single rule reads, are bytes this same file carries. An
    // adopter who has never vendored anything carries no release record here,
    // and `release::at` answering `Err` is silent on purpose — that gap is
    // `pin_current`'s to report, not this function's to refuse over.
    if let Ok(record) = release::at(&directory) {
        if let Ok(divergences) = release::diverged(&directory, &record) {
            if !divergences.is_empty() {
                return Err(SetError::Diverged(divergences));
            }
        }
    }

    let named = manifest
        .get("contents")
        .and_then(|node| node.value.as_map())
        .and_then(|contents| contents.get(CONTENTS_KEY))
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.clone())
        .ok_or_else(|| SetError::Undeclared(consumer.package.clone()))?;
    let path = directory.join(&named);
    let text = std::fs::read_to_string(&path)
        .map_err(|error| SetError::Unreadable(format!("{}: {error}", path.display())))?;
    read(&text, &consumer.package)
}

/// Why a consumer deviates. The same closed set a suppression states, for the
/// same reason: an undifferentiated reason field conflates "wrong" with
/// "tolerated", and the false-positive rate is then unmeasurable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    FalsePositive,
    AcceptedDeviation,
}

impl Reason {
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "false_positive" => Some(Reason::FalsePositive),
            "accepted_deviation" => Some(Reason::AcceptedDeviation),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Reason::FalsePositive => "false_positive",
            Reason::AcceptedDeviation => "accepted_deviation",
        }
    }
}

/// One declared deviation.
///
/// Four fields and every one is required. Spec 7: "A waiver names the rule, the
/// reason, the owner, and an expiry." The expiry is what makes the mechanism
/// different from a decision nobody revisits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Waiver {
    pub rule: String,
    pub reason: Reason,
    pub owner: String,
    pub until: Date,
    pub note: Option<String>,
}

impl Waiver {
    /// Whether this waiver still stands at a date.
    ///
    /// The comparison is inclusive of the day named, so `until: 2027-02-28`
    /// covers the whole of that day and lapses on the first of March.
    pub fn live(&self, now: Date) -> bool {
        now.days_since(self.until) <= 0
    }
}

/// The waivers a consumer declares, read from the consumer declaration.
///
/// **They live in the files of the consumer for a structural reason.**
/// `headwater taxonomy vendor` replaces a vendored package directory whole and
/// refuses to overwrite one that carries no release record, so a shipped rule is
/// never edited locally. A waiver written beside the rule it waives would not
/// survive the next upgrade.
pub fn waivers(root: &Path) -> Result<Vec<Waiver>, Vec<String>> {
    let path = root.join(headwater_resolve::package::CONSUMER);
    // A repository with no consumer declaration has no pin either, and the
    // caller met that failure before it reached here.
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Ok(Vec::new());
    };
    let loaded = headwater_yaml::load(&text)
        .map_err(|errors| vec![headwater_yaml::error::render(&errors)])?;
    let Some(map) = loaded.value.as_map() else {
        return Ok(Vec::new());
    };
    let Some(block) = map.get("conformance").and_then(|node| node.value.as_map()) else {
        return Ok(Vec::new());
    };

    let mut out = Vec::new();
    let mut refusals = Vec::new();

    // `waivers` is the only key this block takes, and any other one ends the
    // run. **A key that was read and dropped is the failure this whole design
    // refuses one level up**, where a rule the engine cannot read ends the run
    // rather than being skipped. The refusal an adopter is most likely to meet
    // is `level`, which reads as a declaration and is not one: the report
    // derives a level from the rules that pass, so a level an adopter typed
    // would be a claim that nothing evaluated and nothing contradicted.
    for entry in block {
        if entry.key.value == "waivers" {
            continue;
        }
        refusals.push(format!(
            "`conformance.{}` is not a key this engine reads, and `waivers` is the only one it \
             takes. No key declares a level: `headwater conformance` derives the level from the \
             rules that pass, and `--level <name>` asks about a rung without recording an answer",
            entry.key.value
        ));
    }

    for entry in seq_of(block, "waivers") {
        let rule = text_of(entry, "rule").unwrap_or_default().to_string();
        let named = match rule.is_empty() {
            true => "a waiver".to_string(),
            false => format!("the waiver on `{rule}`"),
        };
        if rule.is_empty() {
            refusals.push("a waiver names the rule it waives".to_string());
            continue;
        }
        let Some(reason) = text_of(entry, "reason") else {
            refusals.push(format!(
                "{named} states no reason, and the two are `false_positive` and \
                 `accepted_deviation`"
            ));
            continue;
        };
        let Some(reason) = Reason::parse(reason) else {
            refusals.push(format!(
                "{named} states `reason: {reason}`, and the two are `false_positive` and \
                 `accepted_deviation`"
            ));
            continue;
        };
        let Some(owner) = text_of(entry, "owner") else {
            refusals.push(format!(
                "{named} names no owner. A deviation nobody owns is one nobody closes"
            ));
            continue;
        };
        let Some(until) = text_of(entry, "until") else {
            refusals.push(format!(
                "{named} states no `until`. The expiry is required, so that no deviation is \
                 permanent"
            ));
            continue;
        };
        let Some(until) = Date::parse(until) else {
            refusals.push(format!(
                "{named} states `until: {until}`, which is not a date"
            ));
            continue;
        };
        out.push(Waiver {
            rule,
            reason,
            owner: owner.to_string(),
            until,
            note: text_of(entry, "note").map(collapse),
        });
    }

    match refusals.is_empty() {
        true => Ok(out),
        false => Err(refusals),
    }
}

/// What a reading of one rule concluded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    Met,
    /// The rule is not met, and the string states what this engine found.
    Gap(String),
    /// No reading of a tree decides it, and the string states what would.
    NotDecided(String),
}

impl Verdict {
    pub fn met(&self) -> bool {
        matches!(self, Verdict::Met)
    }
}

/// Whether a waiver stands over a rule at the date of the run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cover {
    None,
    Live(Waiver),
    /// A waiver whose date has passed. It is reported and it covers nothing, so
    /// a run fails toward the rule rather than toward the deviation.
    Expired(Waiver),
}

/// One rule, its verdict, and the waiver over it.
#[derive(Clone, Debug)]
pub struct Reading {
    pub rule: Rule,
    pub verdict: Verdict,
    pub cover: Cover,
}

impl Reading {
    /// Whether a `--level` run passes this rule. A live waiver is what a waiver
    /// buys, and it buys only this.
    pub fn passes_gate(&self) -> bool {
        self.verdict.met() || matches!(self.cover, Cover::Live(_))
    }
}

/// One rung of the ladder, measured.
#[derive(Clone, Debug)]
pub struct LevelState {
    pub name: String,
    pub title: String,
    /// Every rule this rung requires, its own and every earlier rung's.
    pub rules: Vec<String>,
    /// Whether every rule it requires is met. **This reads no waiver.** A
    /// waived rule is not a met rule, so a waiver never moves a rung.
    pub reached: bool,
    pub met: usize,
    pub gaps: usize,
    pub waived: usize,
    pub undecided: usize,
}

/// A run of the verb.
#[derive(Clone, Debug)]
pub struct Report {
    /// The package identity, printed beside every level. A level with no
    /// package identity beside it means nothing, because the rule set that
    /// defines the rung came from that package.
    pub package: String,
    pub version: String,
    pub digest: Option<String>,
    pub now: Date,
    pub readings: Vec<Reading>,
    pub levels: Vec<LevelState>,
    /// The highest rung every rule of which is met, and `None` where the first
    /// rung is not reached.
    pub reached: Option<String>,
}

impl Report {
    /// Whether a `--level` run of this report exits 0.
    ///
    /// This is the one question a waiver answers. `reached` above answers the
    /// other one, and no waiver reaches it.
    pub fn gate(&self, level: &str) -> Result<bool, String> {
        let Some(state) = self.levels.iter().find(|one| one.name == level) else {
            return Err(format!(
                "`{level}` is not a level of {}. The rungs it declares are {}",
                self.package,
                match self.levels.is_empty() {
                    true => "none".to_string(),
                    false => self
                        .levels
                        .iter()
                        .map(|one| format!("`{}`", one.name))
                        .collect::<Vec<_>>()
                        .join(", "),
                }
            ));
        };
        Ok(state.rules.iter().all(|name| {
            self.readings
                .iter()
                .find(|reading| &reading.rule.name == name)
                .is_some_and(Reading::passes_gate)
        }))
    }
}

/// What the readings are taken over.
pub struct Subject<'a> {
    pub root: &'a Path,
    pub consumer: &'a Consumer,
    pub lock: &'a Lock,
    pub census: &'a Census,
    /// The projection plan this corpus and this lock produce. It is built by
    /// the caller because `generate` builds one already, and a second builder
    /// here would be a second answer to one question.
    pub plan: &'a headwater_generate::Plan,
    pub now: Date,
}

/// The package identity a report prints beside every rung.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    pub package: String,
    pub version: String,
    pub digest: Option<String>,
}

impl Identity {
    pub fn of(consumer: &Consumer) -> Self {
        Identity {
            package: consumer.package.clone(),
            version: consumer.version.clone(),
            digest: consumer.digest.clone(),
        }
    }
}

/// Evaluate a repository against a rule set.
pub fn evaluate(
    set: &RuleSet,
    waivers: &[Waiver],
    subject: &Subject<'_>,
) -> Result<Report, Vec<String>> {
    let taken: Vec<(String, Verdict)> = set
        .rules
        .iter()
        .map(|rule| {
            let verdict = match rule.decided_by {
                DecidedBy::Tree => reading(&rule.name, subject),
                DecidedBy::Attestation => Verdict::NotDecided(rule.remediation.clone()),
            };
            (rule.name.clone(), verdict)
        })
        .collect();
    assemble(
        set,
        waivers,
        &Identity::of(subject.consumer),
        &taken,
        subject.now,
    )
}

/// Assemble a report from verdicts already taken.
///
/// This is where the ladder is computed, and it is a separate function because
/// **the claim that a waiver cannot buy a rung has to be testable without a
/// tree.** A test that had to build a corpus to ask the question would measure
/// the readings and the ladder at once, and a defect in either would read as a
/// defect in the other.
///
/// A waiver that names a rule the set does not declare ends the run, and the
/// error names each one. A deviation that no report can list is one that nobody
/// reviews away.
pub fn assemble(
    set: &RuleSet,
    waivers: &[Waiver],
    identity: &Identity,
    taken: &[(String, Verdict)],
    now: Date,
) -> Result<Report, Vec<String>> {
    let orphans: Vec<String> = waivers
        .iter()
        .filter(|waiver| set.rule(&waiver.rule).is_none())
        .map(|waiver| {
            format!(
                "the waiver on `{}` names a rule that {} does not declare, so no report can \
                 list it and nobody reviews it away",
                waiver.rule, identity.package
            )
        })
        .collect();
    if !orphans.is_empty() {
        return Err(orphans);
    }

    let readings: Vec<Reading> = set
        .rules
        .iter()
        .map(|rule| {
            let verdict = taken
                .iter()
                .find(|(name, _)| name == &rule.name)
                .map(|(_, verdict)| verdict.clone())
                .unwrap_or_else(|| {
                    Verdict::Gap(format!("no verdict was taken for `{}`", rule.name))
                });
            let cover = match waivers.iter().find(|waiver| waiver.rule == rule.name) {
                None => Cover::None,
                Some(waiver) if waiver.live(now) => Cover::Live(waiver.clone()),
                Some(waiver) => Cover::Expired(waiver.clone()),
            };
            Reading {
                rule: rule.clone(),
                verdict,
                cover,
            }
        })
        .collect();

    let mut levels = Vec::new();
    let mut reached = None;
    let mut standing = true;
    for index in 0..set.levels.len() {
        let declared = &set.levels[index];
        let rules = set.cumulative(index);
        let mut met = 0;
        let mut gaps = 0;
        let mut waived = 0;
        let mut undecided = 0;
        for name in &rules {
            let Some(found) = readings.iter().find(|one| &one.rule.name == name) else {
                continue;
            };
            match &found.verdict {
                Verdict::Met => met += 1,
                Verdict::NotDecided(_) => undecided += 1,
                Verdict::Gap(_) => {
                    gaps += 1;
                    if matches!(found.cover, Cover::Live(_)) {
                        waived += 1;
                    }
                }
            }
        }
        // A rung is reached when every rule it requires is met. No waiver is
        // read here, and that is what stops a level from being bought.
        let all_met = met == rules.len();
        // The ladder is an ordering, so a rung above an unreached rung is not
        // reached either, whatever its own rules say.
        standing = standing && all_met;
        if standing {
            reached = Some(declared.name.clone());
        }
        levels.push(LevelState {
            name: declared.name.clone(),
            title: declared.title.clone(),
            rules,
            reached: all_met,
            met,
            gaps,
            waived,
            undecided,
        });
    }

    Ok(Report {
        package: identity.package.clone(),
        version: identity.version.clone(),
        digest: identity.digest.clone(),
        now,
        readings,
        levels,
        reached,
    })
}

/// The reading this engine holds for one rule name.
///
/// **Each reading below takes exactly what it reads, and this function is the
/// only thing that binds a name to one.** A reading that took the whole
/// [`Subject`] would be a reading a test could not drive without building a
/// corpus, and three of the four read nothing that looks like one.
pub fn reading(name: &str, subject: &Subject<'_>) -> Verdict {
    match name {
        "pin.current" => pin_current(subject.root, subject.consumer),
        "lock.current" => lock_current(subject.root, subject.lock),
        "corpus.classified" => corpus_classified(subject.census),
        "projections.current" => projections_current(subject.root, subject.plan),
        // Unreachable for a set that [`read`] accepted, which refuses a tree
        // rule outside `READINGS` before anything is evaluated. It is here
        // rather than a panic because a refusal in a report is a worse outcome
        // than a refusal at read time and a better one than a crash.
        other => Verdict::Gap(format!("this engine holds no reading for `{other}`")),
    }
}

/// **The pin is two numbers.** A published artifact is a digest over every file
/// in it, so a rule that read the version alone would pass a repository whose
/// pinned digest names an artifact nobody publishes any more.
///
/// **The pin names the record, and this rule now holds the record to the bytes
/// on disk too.** `release::at` reads `release.yml`'s own declared digest,
/// which is a number the file states about itself, and it is never a re-hash
/// of a member file. So a `pinned == record.digest` comparison alone answers
/// "is this the release the consumer took" and says nothing about whether a
/// vendored file was hand-edited after `vendor` installed it.
/// [`release::diverged`] is the call that re-hashes every member the record
/// names and reports what moved, and until this it ran only inside `vendor`'s
/// own install-time path — never again afterward. This rule calls it again
/// here, on every run, which is what makes a hand edit to a vendored file
/// visible on an ongoing basis rather than only at the moment of installation.
/// That matters most for a file such as `conformance.yml`, which carries no
/// taxonomy source and so never reaches `taxonomy resolve --check` either: this
/// rule is the only ongoing reading that holds it to anything at all.
pub fn pin_current(root: &Path, consumer: &Consumer) -> Verdict {
    let installed = headwater_resolve::package::find_version(root, &consumer.package);
    let version = match installed {
        None => {
            return Verdict::Gap(format!(
                "no package under `{}/` declares `{}`",
                headwater_resolve::package::PACKAGES,
                consumer.package
            ))
        }
        Some(found) if found != consumer.version => {
            return Verdict::Gap(format!(
                "this pins {} {} and the package installed is {found}",
                consumer.package, consumer.version
            ))
        }
        Some(found) => found,
    };

    let Some((directory, _)) = headwater_resolve::package::located(root, &consumer.package) else {
        return Verdict::Gap(format!("`{}` is not on disk", consumer.package));
    };

    let record = match release::at(&directory) {
        Ok(record) => record,
        Err(ReleaseError::Absent(_)) => {
            return match &consumer.digest {
                None => Verdict::Gap(format!(
                    "the version is {version} and there is no digest on either side. The \
                     package directory carries no release record, so no published artifact \
                     stands behind it"
                )),
                Some(pinned) => Verdict::Gap(format!(
                    "this pins {pinned} and the installed package carries no release record to \
                     check it against"
                )),
            };
        }
        Err(other) => return Verdict::Gap(other.to_string()),
    };

    let pinned = match &consumer.digest {
        None => {
            return Verdict::Gap(format!(
                "the version is {version} and nothing pins a digest. The installed artifact is \
                 {}, and `taxonomy.digest` is where a consumer states which artifact it takes",
                record.digest
            ))
        }
        Some(pinned) if pinned != &record.digest => {
            return Verdict::Gap(format!(
                "the version is {version} and the digest is not: this pins {pinned} and the \
                 installed artifact is {}",
                record.digest
            ))
        }
        Some(pinned) => pinned,
    };

    match release::diverged(&directory, &record) {
        Ok(divergences) if divergences.is_empty() => Verdict::Met,
        Ok(divergences) => Verdict::Gap(format!(
            "the version is {version} and the pin {pinned} matches the release record, but the \
             installed artifact no longer matches that record: {}",
            divergences
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ")
        )),
        Err(other) => Verdict::Gap(other.to_string()),
    }
}

/// The lock is what the sources resolve to.
///
/// **The reading is the comparison `taxonomy resolve --check` decides with,
/// called rather than approximated.** The sources are resolved, a lock is
/// written from them carrying the authored block the committed file holds, and
/// [`headwater_lock::diverged`] says how the committed bytes differ from that
/// text. That is the single question the CLI already prints an answer to, so
/// this is the reading that keeps the two verbs from being two answers.
///
/// **The reading it replaced was the explanatory half of that verb and not the
/// deciding one.** [`Lock::moved`] re-hashes the source files on disk against
/// the digests the lock records, and `taxonomy resolve --check` calls it only
/// after it has failed, to name which source moved. Everything the lock states
/// that is not a source file was invisible to it: the header, the `sources`
/// list itself, and the `founded:` record, which sits outside the digest
/// [`headwater_lock::read`] verifies. A lock with its `founded:` block deleted
/// by hand was reported met by this rule and refused by `taxonomy resolve
/// --check` one command later, which is a red build an adopter was told they
/// did not have.
///
/// **The cost this accepts:** the rule now resolves the taxonomy from source,
/// where before it read only the committed lock. `taxonomy resolve --check`
/// already pays exactly that, so the cost is known and bounded, and a root
/// whose sources no longer resolve is now a gap here rather than a met.
pub fn lock_current(root: &Path, lock: &Lock) -> Verdict {
    let repository = match headwater_resolve::repository(root) {
        Ok(repository) => repository,
        Err(errors) => {
            return Verdict::Gap(format!(
                "the sources do not resolve, so nothing here can say what they resolve to: {}",
                joined(&errors)
            ))
        }
    };
    let sources = match headwater_resolve::package::sources(root, &repository.consumer) {
        Ok(sources) => sources,
        Err(errors) => {
            return Verdict::Gap(format!(
                "the sources this lock names cannot be read: {}",
                joined(&errors)
            ))
        }
    };
    // The authored block is carried through from the committed file, exactly as
    // the resolver carries it. A comparison that dropped it would report every
    // adopter holding an `adoption` block as a lock that moved, over a block
    // this file's own header invites them to write.
    let authored = headwater_lock::authored_at(root);
    let text = match headwater_lock::write(
        &repository.consumer.package,
        &repository.consumer.version,
        &sources,
        &repository.resolution,
        authored.payload(),
    ) {
        Ok(text) => text,
        Err(findings) => {
            return Verdict::Gap(format!(
                "the taxonomy these sources resolve to does not validate, so there is no lock \
                 for this one to be: {}",
                joined(&findings)
            ))
        }
    };

    let committed = std::fs::read_to_string(root.join(headwater_lock::LOCK)).unwrap_or_default();
    match headwater_lock::diverged(&committed, &text) {
        headwater_lock::Divergence::Same => Verdict::Met,
        // The generated half moved. Which source moved is the line an author
        // acts on, and an empty list is the case this rule used to read as met:
        // what differs is generated from the resolution rather than hashed from
        // a file.
        headwater_lock::Divergence::Generated => {
            let moved = lock.moved(root);
            match moved.is_empty() {
                true => Verdict::Gap(format!(
                    "{} is not what the sources resolve to, and no source file under it moved. \
                     What differs is produced by the resolution rather than hashed from a \
                     source. Run `headwater taxonomy resolve` and commit the result",
                    headwater_lock::LOCK
                )),
                false => Verdict::Gap(format!(
                    "{} is not what the sources resolve to: {} source{} moved since it was \
                     written: {}",
                    headwater_lock::LOCK,
                    moved.len(),
                    match moved.len() {
                        1 => "",
                        _ => "s",
                    },
                    moved.join(", ")
                )),
            }
        }
        // A gap, and the sentence says why it is not a met. The content agrees,
        // so the temptation is to call it met — but `taxonomy resolve --check`
        // exits 1 on this file, and a rule that said met here would leave the
        // adopter holding the same contradiction in a second place.
        headwater_lock::Divergence::Form { adoption } => Verdict::Gap(format!(
            "{} carries the taxonomy its sources resolve to, and is not written in the form \
             `headwater taxonomy resolve` writes it. Nothing about the sources changed, and \
             `headwater taxonomy resolve --check` refuses this file. {}",
            headwater_lock::LOCK,
            match adoption {
                true =>
                    "The `adoption` block is where the two differ. That block is authored, \
                         so what moved is its form and not the debt it declares: a resolve \
                         carries every task, owner, expiry and pair through",
                false =>
                    "The difference is not inside the `adoption` block. Run `headwater \
                          taxonomy resolve` and commit the result",
            }
        )),
        // Unreachable from `headwater conformance`, which reads the lock before
        // any rule runs and refuses a run over one that will not read. The arm
        // says so rather than assuming it, because a caller that built a
        // [`Subject`] some other way would reach it.
        headwater_lock::Divergence::Unreadable(why) => Verdict::Gap(format!(
            "{} did not read, so nothing here can say whether it is what the sources resolve \
             to: {why}",
            headwater_lock::LOCK
        )),
    }
}

/// Several errors as one sentence, on the terms [`release::diverged`]'s caller
/// above already takes.
fn joined<E: ToString>(errors: &[E]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

/// Every file under the corpus root is classified or accounted for.
///
/// The population is the two outcomes that leave a file with no kind and no
/// stated reason. An excluded file states a reason, a generated file names its
/// projection, and a file that is not Markdown is not a document. Each of those
/// is an account, and this rule reads the rows that carry none.
pub fn corpus_classified(census: &Census) -> Verdict {
    let loose: Vec<&str> = census
        .rows
        .iter()
        .filter(|row| matches!(row.outcome, Outcome::Untyped(_) | Outcome::Unreadable(_)))
        .map(|row| row.path.as_str())
        .collect();
    match loose.is_empty() {
        true => Verdict::Met,
        false => Verdict::Gap(format!(
            "{} file{} under the corpus root carr{} no kind and no stated reason: {}",
            loose.len(),
            match loose.len() {
                1 => "",
                _ => "s",
            },
            match loose.len() {
                1 => "ies",
                _ => "y",
            },
            loose.join(", ")
        )),
    }
}

/// Every declared projection is what this corpus and this lock produce.
///
/// The comparison is `generate --check`, called rather than repeated.
pub fn projections_current(root: &Path, plan: &headwater_generate::Plan) -> Verdict {
    let report = headwater_generate::check(root, plan);
    if !report.has_errors() {
        return Verdict::Met;
    }
    let mut drifted: Vec<&str> = report
        .wrote
        .iter()
        .filter(|wrote| wrote.verdict.is_error())
        .map(|wrote| wrote.path.as_str())
        .collect();
    drifted.extend(report.orphaned.iter().map(|one| one.path.as_str()));
    Verdict::Gap(format!(
        "{} projection{} not what this corpus and this lock produce: {}",
        drifted.len(),
        match drifted.len() {
            1 => " is",
            _ => "s are",
        },
        drifted.join(", ")
    ))
}

fn text_of<'a>(map: &'a Mapping, key: &str) -> Option<&'a str> {
    map.get(key)
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.as_str())
}

fn seq_of<'a>(map: &'a Mapping, key: &str) -> Vec<&'a Mapping> {
    map.get(key)
        .and_then(|node| node.value.as_seq())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.value.as_map())
                .collect()
        })
        .unwrap_or_default()
}

/// A folded scalar arrives with its newlines, and a report writes one line.
fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
