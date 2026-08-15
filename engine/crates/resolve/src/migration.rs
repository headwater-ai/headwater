// SPDX-License-Identifier: Apache-2.0
//! The migration payload a major version ships, and what a publisher may not
//! ship in one.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#versioning-by-measured-compatibility):
//! "A major version ships a **migration payload**: machine-readable steps that
//! declare what moved, what was renamed, and what must be re-stated. The steps
//! are split into what the engine can apply mechanically and what needs human
//! or agent judgment." [Spec 7](../../../../docs/spec/07-distribution-and-federation.md#the-migration-payload)
//! states the form. This module reads it.
//!
//! # The split runs through a step, and no field declares which side it is on
//!
//! The obvious shape is a rename map with a list of judgment tasks beside it,
//! and it is wrong. [The schema-format walkthrough](../../../../docs/evaluations/schema-format-walkthrough.md)
//! found the case that breaks it: one old value maps to a *set* of new ones, and
//! the author chooses. That step is a rename in every respect except the one
//! that decides whether a program may apply it. So the split is per step, and it
//! is derived from the target list rather than declared beside it.
//!
//! [`Application`] is where that lives. One target is mechanical, two or more
//! are a closed choice, and none is a re-statement. A publisher cannot write a
//! step that claims to be mechanical and offers a choice, because there is no
//! field in which to make the claim.
//!
//! # A task on a mechanical step is refused rather than dropped
//!
//! [`Application::Mechanical`] carries no task, so a `task:` beside a single
//! target has nowhere to go. To drop it would be the defect this whole issue is
//! about — a key a publisher wrote and a consumer never saw — so it is refused
//! and the message names both halves.
//!
//! # The two ends of the wire check different halves, and neither can check both
//!
//! **The publisher can check its own taxonomy and not the one it came from.**
//! [`holds`] is that check: a step whose source is still declared in the
//! taxonomy being published renames nothing, and a step whose target is not
//! declared there renames a value into a taxonomy that has no such value. Both
//! are defects the publisher can act on before the digest is taken.
//!
//! **The consumer can check the taxonomy it holds and not the publisher's.** Its
//! reading of the old side is the base under *its own* overlays, and an overlay
//! may legitimately have removed the value a step names. So a step that names
//! nothing this repository holds is reported by `taxonomy diff` and never
//! refused: for that consumer the step is vacuous rather than wrong.

use crate::error::{ResolveError, ResolveErrorKind};
use crate::release;
use headwater_yaml::Mapping;
use std::path::{Path, PathBuf};

/// The format of a payload file. A reader that meets a later one says so rather
/// than guessing, on the terms the lock, the release record and the conformance
/// rule set all take.
pub const FORMAT: u32 = 1;

/// The manifest key under `contents` that points at the payload directory.
pub const CONTENTS_KEY: &str = "migrations";

/// What the old name of a step names.
///
/// Closed, and each arm fixes the compatibility dimension the step is a remedy
/// for. The mapping belongs to the engine and no payload may vary it: a
/// `remedies:` key a publisher wrote would be an assertion, and this issue is
/// about the difference between a claim and a thing a run can check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Subject {
    /// One value of one facet. Every document carrying the value is a subject.
    FacetValue { facet: String },
    /// One kind. Every document the census typed as it is a subject.
    Kind,
}

impl Subject {
    /// The names a payload may write, in the message that refuses another one.
    pub const NAMES: [&'static str; 2] = ["facet_value", "kind"];

    pub fn name(&self) -> &'static str {
        match self {
            Subject::FacetValue { .. } => "facet_value",
            Subject::Kind => "kind",
        }
    }

    /// The dimensions of [spec 2](../../../../docs/spec/02-taxonomy-model.md#versioning-by-measured-compatibility)
    /// that a step over this subject is a remedy for.
    ///
    /// A facet value names two because spec 2 rules that one contains the
    /// other: "a broken `instance_validity` breaks `consequence` too". A kind
    /// names one, because whether a kind rename also stops a document
    /// validating depends on the facet requirements of the two kinds, and the
    /// payload does not know them.
    pub fn remedies(&self) -> &'static [&'static str] {
        match self {
            Subject::FacetValue { .. } => &["instance_validity", "consequence"],
            Subject::Kind => &["classification"],
        }
    }

    /// How a reader of a report names the thing this step moved.
    pub fn at(&self, value: &str) -> String {
        match self {
            Subject::FacetValue { facet } => format!("{facet}/{value}"),
            Subject::Kind => value.to_string(),
        }
    }
}

/// How a step is applied, derived from the target list and never declared.
///
/// [`Application::over`] is the only constructor, so a state in which a step
/// offers two targets and claims to be mechanical cannot be built.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Application {
    /// Exactly one target. The engine applies it and nobody is asked anything.
    Mechanical { to: String },
    /// Two or more targets. The author settles it per subject, out of a set the
    /// publisher closed.
    Choice { among: Vec<String>, task: String },
    /// No target. Nothing replaces the old value and the subject is re-stated,
    /// which is spec 2's third category.
    Restatement { task: String },
}

impl Application {
    /// The one route in. `task` is what the payload declared, and it is
    /// [`Option`] rather than a string so that "no task" and "an empty task"
    /// are two different refusals.
    pub fn over(targets: Vec<String>, task: Option<String>) -> Result<Application, String> {
        match (targets.len(), task) {
            (1, Some(_)) => Err(
                "a step with one target applies with no judgment, and the `task` \
                                 beside it names a question that nothing would ever ask. Drop one \
                                 of the two"
                    .to_string(),
            ),
            (1, None) => Ok(Application::Mechanical {
                to: targets.into_iter().next().expect("one target"),
            }),
            (0, None) => Err(
                "a step with no target states that nothing replaces the old value, \
                              so the author has to re-state the subject. That needs a `task` \
                              saying what to re-state"
                    .to_string(),
            ),
            (0, Some(task)) => Ok(Application::Restatement { task }),
            (_, None) => Err(
                "a step with more than one target leaves a choice the author \
                              settles, and a choice with no `task` beside it is a question \
                              nobody was asked. Add `task`"
                    .to_string(),
            ),
            (_, Some(task)) => Ok(Application::Choice {
                among: targets,
                task,
            }),
        }
    }

    /// Whether the engine can apply this step with no judgment.
    pub fn mechanical(&self) -> bool {
        matches!(self, Application::Mechanical { .. })
    }

    /// Every value this step can put in place of the old one. Empty for a
    /// re-statement, which is the state that says nothing replaces it.
    pub fn targets(&self) -> &[String] {
        match self {
            Application::Mechanical { to } => std::slice::from_ref(to),
            Application::Choice { among, .. } => among,
            Application::Restatement { .. } => &[],
        }
    }

    /// The question the author settles, where there is one.
    pub fn task(&self) -> Option<&str> {
        match self {
            Application::Mechanical { .. } => None,
            Application::Choice { task, .. } | Application::Restatement { task } => Some(task),
        }
    }

    /// One line a reader acts on.
    pub fn sentence(&self) -> String {
        match self {
            Application::Mechanical { to } => format!("mechanical, and it becomes `{to}`"),
            Application::Choice { among, .. } => format!(
                "a closed choice the author settles, among {}",
                among
                    .iter()
                    .map(|value| format!("`{value}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Application::Restatement { .. } => {
                "a re-statement, and nothing replaces the old value".to_string()
            }
        }
    }
}

/// One step of a payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub subject: Subject,
    /// The value that moved, as the old taxonomy declared it.
    pub from: String,
    pub apply: Application,
    /// Why the publisher moved it. Required, on the terms an exclusion states a
    /// reason: a step with no reason is a rewrite of somebody's corpus with a
    /// configuration file in front of it.
    pub because: String,
}

impl Step {
    /// How a report names this step.
    pub fn at(&self) -> String {
        format!("{} {}", self.subject.name(), self.subject.at(&self.from))
    }
}

/// One payload file: the transition it is for, and the steps it carries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Payload {
    /// The path it was read from, as a reader of the artifact would write it.
    pub at: String,
    /// The versions this payload migrates from, as a range.
    pub from: String,
    /// The versions it migrates to, as a range.
    pub to: String,
    pub steps: Vec<Step>,
}

impl Payload {
    /// Whether this payload is the one for a move between two versions.
    ///
    /// Both ends are ranges read by [`release::satisfies`], which is the one
    /// piece of version arithmetic this engine has. A second reading of a
    /// version string here could disagree with the one `requires_engine` and
    /// `--to` are read by.
    pub fn covers(&self, from: &str, to: &str) -> Result<bool, String> {
        Ok(release::satisfies(&self.from, from)? && release::satisfies(&self.to, to)?)
    }
}

/// What a payload refuses to be read as.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PayloadError {
    /// The declared directory is not one, or is not there.
    NoDirectory {
        at: String,
        why: String,
    },
    /// The declared path climbs out of the package that names it.
    Leaves {
        at: String,
    },
    Unreadable {
        at: String,
        why: String,
    },
    Malformed {
        at: String,
        what: String,
    },
    /// A later format than this engine knows.
    Format {
        at: String,
        found: String,
    },
    /// A step that names a subject outside the closed set.
    Subject {
        at: String,
        found: String,
    },
    /// A step whose target list and task do not make a legal application.
    Application {
        at: String,
        step: String,
        why: String,
    },
    /// A version range this engine cannot read.
    Range {
        at: String,
        range: String,
        why: String,
    },
    /// The payload does not carry the version it was published in.
    NotThisVersion {
        at: String,
        to: String,
        version: String,
    },
    /// The payload claims to migrate away from the version it was published in.
    FromItself {
        at: String,
        from: String,
        version: String,
    },
    /// A step whose source is still declared in the taxonomy being published.
    SourceStands {
        at: String,
        step: String,
        subject: &'static str,
    },
    /// A step whose target is not declared in the taxonomy being published.
    TargetAbsent {
        at: String,
        step: String,
        target: String,
        subject: &'static str,
    },
}

impl PayloadError {
    /// The file the refusal is about.
    pub fn at(&self) -> &str {
        match self {
            PayloadError::NoDirectory { at, .. }
            | PayloadError::Leaves { at }
            | PayloadError::Unreadable { at, .. }
            | PayloadError::Malformed { at, .. }
            | PayloadError::Format { at, .. }
            | PayloadError::Subject { at, .. }
            | PayloadError::Application { at, .. }
            | PayloadError::Range { at, .. }
            | PayloadError::NotThisVersion { at, .. }
            | PayloadError::FromItself { at, .. }
            | PayloadError::SourceStands { at, .. }
            | PayloadError::TargetAbsent { at, .. } => at,
        }
    }
}

impl std::fmt::Display for PayloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PayloadError::NoDirectory { at, why } => write!(
                f,
                "`contents.{CONTENTS_KEY}` names `{at}`, and there is no directory to read there: \
                 {why}"
            ),
            PayloadError::Leaves { at } => write!(
                f,
                "`contents.{CONTENTS_KEY}` names `{at}`, which climbs out of the package that \
                 declares it. A payload outside the package is one the artifact would not carry, \
                 and a consumer would then hold a migration nobody shipped"
            ),
            PayloadError::Unreadable { at, why } => write!(f, "{at} does not read: {why}"),
            PayloadError::Malformed { at, what } => write!(f, "{at} is malformed: {what}"),
            PayloadError::Format { at, found } => write!(
                f,
                "{at} declares format {found} and this engine reads {FORMAT}"
            ),
            PayloadError::Subject { at, found } => write!(
                f,
                "{at} declares a step over `{found}`, and a step names one of {}",
                Subject::NAMES.join(", ")
            ),
            PayloadError::Application { at, step, why } => write!(f, "{at}, at {step}: {why}"),
            PayloadError::Range { at, range, why } => write!(
                f,
                "{at} states the version range `{range}`, which this engine cannot read: {why}"
            ),
            PayloadError::NotThisVersion { at, to, version } => write!(
                f,
                "{at} migrates to `{to}` and this publishes {version}. A payload the artifact \
                 carries and the artifact's own version does not answer to is a payload no \
                 consumer of this release would ever select"
            ),
            PayloadError::FromItself { at, from, version } => write!(
                f,
                "{at} migrates from `{from}`, which admits {version}, the version this publishes. \
                 A payload cannot migrate a corpus away from the release that carries it"
            ),
            PayloadError::SourceStands { at, step, subject } => write!(
                f,
                "{at}, at {step}: the taxonomy this publishes still declares that {subject}, so \
                 the step renames something that did not move"
            ),
            PayloadError::TargetAbsent {
                at,
                step,
                target,
                subject,
            } => write!(
                f,
                "{at}, at {step}: the taxonomy this publishes declares no {subject} `{target}`, \
                 so the step renames a value into a taxonomy that cannot hold it"
            ),
        }
    }
}

/// Every payload a package ships, in path order.
///
/// `Ok(vec![])` where the manifest declares no `contents.migrations`, which is
/// the ordinary state of a package that has published no major version. Every
/// other absence is a refusal: a declared directory that is not there is a claim
/// the publisher made and the artifact does not carry.
pub fn at(directory: &Path, manifest: &Mapping) -> Result<Vec<Payload>, Vec<PayloadError>> {
    let Some(declared) = manifest
        .get("contents")
        .and_then(|node| node.value.as_map())
        .and_then(|contents| contents.get(CONTENTS_KEY))
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.clone())
    else {
        return Ok(Vec::new());
    };

    if Path::new(&declared)
        .components()
        .any(|part| part == std::path::Component::ParentDir)
        || Path::new(&declared).is_absolute()
    {
        return Err(vec![PayloadError::Leaves { at: declared }]);
    }

    let root = directory.join(&declared);
    let mut files: Vec<PathBuf> = match std::fs::read_dir(&root) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("yml"))
            .collect(),
        Err(error) => {
            return Err(vec![PayloadError::NoDirectory {
                at: declared,
                why: error.to_string(),
            }])
        }
    };
    files.sort();

    let mut payloads = Vec::new();
    let mut refusals = Vec::new();
    for file in files {
        let name = format!(
            "{}/{}",
            declared.trim_end_matches('/'),
            file.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
        );
        match std::fs::read_to_string(&file) {
            Err(error) => refusals.push(PayloadError::Unreadable {
                at: name,
                why: error.to_string(),
            }),
            Ok(text) => match read(&text, &name) {
                Ok(payload) => payloads.push(payload),
                Err(mut errors) => refusals.append(&mut errors),
            },
        }
    }
    match refusals.is_empty() {
        true => Ok(payloads),
        false => Err(refusals),
    }
}

/// Read one payload file.
pub fn read(text: &str, at: &str) -> Result<Payload, Vec<PayloadError>> {
    let malformed = |what: &str| {
        vec![PayloadError::Malformed {
            at: at.to_string(),
            what: what.to_string(),
        }]
    };

    let loaded = headwater_yaml::load(text).map_err(|errors| {
        vec![PayloadError::Unreadable {
            at: at.to_string(),
            why: headwater_yaml::error::render(&errors),
        }]
    })?;
    let root = loaded
        .value
        .as_map()
        .ok_or_else(|| malformed("the root is not a mapping"))?;
    let header = root
        .get("migration")
        .and_then(|node| node.value.as_map())
        .ok_or_else(|| malformed("no `migration` block, so nothing says what this migrates"))?;

    let format = text_of(header, "format").unwrap_or_else(|| FORMAT.to_string());
    if format.trim() != FORMAT.to_string() {
        return Err(vec![PayloadError::Format {
            at: at.to_string(),
            found: format,
        }]);
    }

    let from = text_of(header, "from")
        .ok_or_else(|| malformed("`migration.from` names the versions this migrates from"))?;
    let to = text_of(header, "to")
        .ok_or_else(|| malformed("`migration.to` names the versions this migrates to"))?;
    for range in [&from, &to] {
        if let Err(why) = release::satisfies(range, "0.0.0") {
            return Err(vec![PayloadError::Range {
                at: at.to_string(),
                range: range.clone(),
                why,
            }]);
        }
    }

    let declared = root
        .get("steps")
        .and_then(|node| node.value.as_seq())
        .ok_or_else(|| {
            malformed("no `steps` sequence, and a payload with no step migrates nothing")
        })?;
    if declared.is_empty() {
        return Err(malformed(
            "`steps` is empty, and a payload that moves nothing is a file a consumer would apply \
             and learn nothing from",
        ));
    }

    let mut steps = Vec::new();
    let mut refusals = Vec::new();
    for node in declared {
        let Some(entry) = node.value.as_map() else {
            refusals.append(&mut malformed("a step is a mapping"));
            continue;
        };
        match step(entry, at) {
            Ok(built) => steps.push(built),
            Err(mut errors) => refusals.append(&mut errors),
        }
    }
    match refusals.is_empty() {
        true => Ok(Payload {
            at: at.to_string(),
            from,
            to,
            steps,
        }),
        false => Err(refusals),
    }
}

fn step(entry: &Mapping, at: &str) -> Result<Step, Vec<PayloadError>> {
    let malformed = |what: String| {
        vec![PayloadError::Malformed {
            at: at.to_string(),
            what,
        }]
    };

    let named = text_of(entry, "subject")
        .ok_or_else(|| malformed("a step names the `subject` it moves".to_string()))?;
    let facet = text_of(entry, "facet");
    let subject = match (named.as_str(), facet) {
        ("facet_value", Some(facet)) => Subject::FacetValue { facet },
        ("facet_value", None) => {
            return Err(malformed(
                "a `facet_value` step names the `facet` the value belongs to".to_string(),
            ))
        }
        ("kind", None) => Subject::Kind,
        ("kind", Some(facet)) => {
            return Err(malformed(format!(
                "a `kind` step names no facet, and this one declares `facet: {facet}`. A key this \
                 engine read and dropped is what a payload exists to stop"
            )))
        }
        (other, _) => {
            return Err(vec![PayloadError::Subject {
                at: at.to_string(),
                found: other.to_string(),
            }])
        }
    };

    let from = text_of(entry, "from")
        .ok_or_else(|| malformed("a step names the `from` value that moved".to_string()))?;

    // `to` is required and an empty sequence is legal. The two are different
    // statements: an absent key is a publisher who did not say, and `to: []` is
    // a publisher who said that nothing replaces the value. An `unwrap_or_default`
    // here would read the first as the second.
    let targets = entry
        .get("to")
        .ok_or_else(|| {
            malformed(
                "a step names `to`, and `to: []` is how a step says that nothing replaces the old \
                 value. An absent key is a publisher who did not say"
                    .to_string(),
            )
        })?
        .value
        .as_seq()
        .ok_or_else(|| malformed("`to` is a sequence, even where it holds one value".to_string()))?
        .iter()
        .map(|item| {
            item.value
                .as_scalar()
                .map(|scalar| scalar.text.clone())
                .ok_or_else(|| malformed("a `to` entry is a scalar".to_string()))
        })
        .collect::<Result<Vec<String>, Vec<PayloadError>>>()?;

    let apply = Application::over(targets, text_of(entry, "task")).map_err(|why| {
        vec![PayloadError::Application {
            at: at.to_string(),
            step: format!("{} {}", subject.name(), subject.at(&from)),
            why,
        }]
    })?;

    let because = text_of(entry, "because").ok_or_else(|| {
        malformed("a step states `because`, which is why the publisher moved it".to_string())
    })?;

    Ok(Step {
        subject,
        from,
        apply,
        because,
    })
}

/// What the publisher can check about its own payload, against the taxonomy it
/// is publishing and the version it is publishing it as.
///
/// The other half — whether the source ever existed — needs the taxonomy the
/// consumer holds, and the module comment says why that half is reported rather
/// than refused.
pub fn holds(payload: &Payload, taxonomy: &Mapping, version: &str) -> Vec<PayloadError> {
    let mut refusals = Vec::new();

    match release::satisfies(&payload.to, version) {
        Err(why) => refusals.push(PayloadError::Range {
            at: payload.at.clone(),
            range: payload.to.clone(),
            why,
        }),
        Ok(false) => refusals.push(PayloadError::NotThisVersion {
            at: payload.at.clone(),
            to: payload.to.clone(),
            version: version.to_string(),
        }),
        Ok(true) => {}
    }
    match release::satisfies(&payload.from, version) {
        Err(why) => refusals.push(PayloadError::Range {
            at: payload.at.clone(),
            range: payload.from.clone(),
            why,
        }),
        Ok(true) => refusals.push(PayloadError::FromItself {
            at: payload.at.clone(),
            from: payload.from.clone(),
            version: version.to_string(),
        }),
        Ok(false) => {}
    }

    for step in &payload.steps {
        if declares(taxonomy, &step.subject, &step.from) {
            refusals.push(PayloadError::SourceStands {
                at: payload.at.clone(),
                step: step.at(),
                subject: step.subject.name(),
            });
        }
        for target in step.apply.targets() {
            if !declares(taxonomy, &step.subject, target) {
                refusals.push(PayloadError::TargetAbsent {
                    at: payload.at.clone(),
                    step: step.at(),
                    target: target.clone(),
                    subject: step.subject.name(),
                });
            }
        }
    }
    refusals
}

/// Whether a resolved taxonomy declares one value of one subject.
///
/// Over a resolved taxonomy and never over a source, because
/// `values: $vocabularies.lifecycle_state` is a reference and a reader of the
/// source would answer that a facet declares no value at all.
pub fn declares(taxonomy: &Mapping, subject: &Subject, value: &str) -> bool {
    match subject {
        Subject::Kind => taxonomy
            .get("kinds")
            .and_then(|node| node.value.as_map())
            .map(|kinds| kinds.get(value).is_some())
            .unwrap_or(false),
        Subject::FacetValue { facet } => taxonomy
            .get("facets")
            .and_then(|node| node.value.as_map())
            .and_then(|facets| facets.get(facet))
            .and_then(|node| node.value.as_map())
            .and_then(|declaration| declaration.get("values"))
            .and_then(|node| node.value.as_seq())
            .map(|values| values.iter().any(|item| named(&item.value) == Some(value)))
            .unwrap_or(false),
    }
}

/// The value one entry of a `values` list carries, in either spelling: a bare
/// scalar, or a mapping that carries `value:` beside a role.
fn named(value: &headwater_yaml::Value) -> Option<&str> {
    match value {
        headwater_yaml::Value::Scalar(scalar) => Some(scalar.text.as_str()),
        headwater_yaml::Value::Map(map) => map
            .get("value")
            .and_then(|node| node.value.as_scalar())
            .map(|scalar| scalar.text.as_str()),
        headwater_yaml::Value::Seq(_) => None,
    }
}

/// Every refusal as the error type the rest of this crate reports.
pub fn as_errors(source: &str, refusals: &[PayloadError]) -> Vec<ResolveError> {
    refusals
        .iter()
        .map(|refusal| {
            ResolveError::new(
                ResolveErrorKind::SourceRefused(refusal.to_string()),
                source,
                "",
                headwater_yaml::Span::default(),
            )
        })
        .collect()
}

fn text_of(map: &Mapping, key: &str) -> Option<String> {
    map.get(key)
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(steps: &str) -> Result<Payload, Vec<PayloadError>> {
        read(
            &format!("migration:\n  from: \">=1 <2\"\n  to: \">=2 <3\"\nsteps:\n{steps}"),
            "migrations/1-to-2.yml",
        )
    }

    fn one(steps: &str) -> PayloadError {
        let errors = payload(steps).expect_err("it refuses");
        assert_eq!(errors.len(), 1, "{errors:?}");
        errors.into_iter().next().expect("one")
    }

    /// The distinction the whole format rests on at the parse boundary.
    ///
    /// `to: []` is a publisher who said that nothing replaces the value, and an
    /// absent `to` is a publisher who did not say. An `unwrap_or_default` would
    /// read the second as the first, and every step that forgot a target would
    /// silently become a re-statement task over the documents that carry it.
    #[test]
    fn an_absent_target_list_is_not_an_empty_one() {
        let absent =
            one("  - subject: facet_value\n    facet: status\n    from: draft\n    because: why\n");
        assert!(
            matches!(&absent, PayloadError::Malformed { what, .. } if what.contains("did not say")),
            "{absent}"
        );

        let empty = payload(
            "  - subject: facet_value\n    facet: status\n    from: draft\n    to: []\n    task: \
             re-state it\n    because: why\n",
        )
        .expect("an empty list is a legal re-statement");
        assert!(matches!(
            empty.steps[0].apply,
            Application::Restatement { .. }
        ));
    }

    /// The three applications, out of the target list alone.
    #[test]
    fn the_application_is_derived_from_the_target_list() {
        let mechanical = Application::over(vec!["one".into()], None).expect("one target");
        assert!(mechanical.mechanical());
        assert_eq!(mechanical.targets().len(), 1);
        assert_eq!(mechanical.task(), None);

        let choice = Application::over(vec!["one".into(), "two".into()], Some("pick".into()))
            .expect("two targets and a task");
        assert!(!choice.mechanical());
        assert_eq!(choice.task(), Some("pick"));

        let restated =
            Application::over(Vec::new(), Some("re-state".into())).expect("no target and a task");
        assert!(restated.targets().is_empty());
        assert!(!restated.mechanical());
    }

    /// A task beside one target has nowhere to go, and a reader that dropped it
    /// would be the defect this payload exists to stop.
    #[test]
    fn a_task_on_a_mechanical_step_is_refused_rather_than_dropped() {
        let refused = Application::over(vec!["one".into()], Some("a question".into()))
            .expect_err("it refuses");
        assert!(refused.contains("applies with no judgment"), "{refused}");
    }

    /// A choice and a re-statement both need somebody to act, and a step that
    /// names nobody is a question that no task list would carry.
    #[test]
    fn a_judgment_step_with_no_task_is_refused() {
        for targets in [Vec::new(), vec!["one".into(), "two".into()]] {
            let count = targets.len();
            assert!(
                Application::over(targets, None).is_err(),
                "{count} targets and no task"
            );
        }
    }

    /// A `facet` key on a `kind` step is a key this engine would read and drop.
    #[test]
    fn a_key_a_subject_does_not_take_is_refused() {
        let refused = one(
            "  - subject: kind\n    facet: status\n    from: decision\n    to: \
                           [ruling]\n    because: why\n",
        );
        assert!(
            matches!(&refused, PayloadError::Malformed { what, .. } if what.contains("read and dropped")),
            "{refused}"
        );
    }

    /// A subject outside the closed set names the set it is outside of.
    #[test]
    fn a_subject_outside_the_set_names_the_set() {
        let refused = one(
            "  - subject: shelf\n    from: decisions\n    to: [rulings]\n    \
                           because: why\n",
        );
        assert!(
            matches!(&refused, PayloadError::Subject { found, .. } if found == "shelf"),
            "{refused}"
        );
        assert!(
            refused.to_string().contains("facet_value, kind"),
            "{refused}"
        );
    }

    /// A later format is refused rather than read as this one.
    #[test]
    fn a_later_format_is_refused() {
        let refused = read(
            "migration:\n  format: 2\n  from: \">=1 <2\"\n  to: \">=2 <3\"\nsteps: []\n",
            "migrations/1-to-2.yml",
        )
        .expect_err("it refuses");
        assert!(
            matches!(refused[0], PayloadError::Format { .. }),
            "{refused:?}"
        );
    }

    /// The subject fixes the dimension, and no payload can vary it.
    #[test]
    fn the_remedy_is_fixed_by_the_subject() {
        assert_eq!(
            Subject::FacetValue {
                facet: "status".into()
            }
            .remedies(),
            ["instance_validity", "consequence"]
        );
        assert_eq!(Subject::Kind.remedies(), ["classification"]);
    }

    fn taxonomy(text: &str) -> Mapping {
        headwater_yaml::load(text)
            .expect("it loads")
            .value
            .as_map()
            .expect("a mapping")
            .clone()
    }

    /// A `values` list is written in two spellings, and a reader of one alone
    /// would answer that the base package declares no lifecycle value at all.
    #[test]
    fn a_value_is_found_in_either_spelling() {
        let status = Subject::FacetValue {
            facet: "status".into(),
        };
        let mapped = taxonomy(
            "facets:\n  status:\n    values:\n      - {value: draft, role: initial}\n      - \
             {value: current}\n",
        );
        assert!(declares(&mapped, &status, "draft"));
        assert!(!declares(&mapped, &status, "retired"));

        let bare = taxonomy("facets:\n  status:\n    values: [draft, current]\n");
        assert!(declares(&bare, &status, "current"));
        assert!(!declares(&bare, &status, "retired"));

        let kinds = taxonomy("kinds:\n  decision: {}\n");
        assert!(declares(&kinds, &Subject::Kind, "decision"));
        assert!(!declares(&kinds, &Subject::Kind, "ruling"));
    }

    /// Both ends of a payload are ranges, read by the one piece of version
    /// arithmetic this engine has.
    #[test]
    fn a_payload_covers_a_move_between_two_versions() {
        let built = payload(
            "  - subject: facet_value\n    facet: status\n    from: draft\n    to: [outline]\n    \
             because: why\n",
        )
        .expect("it reads");
        assert!(built.covers("1.4.0", "2.0.0").expect("both read"));
        assert!(!built.covers("2.0.0", "3.0.0").expect("both read"));
        assert!(!built.covers("0.9.0", "2.0.0").expect("both read"));
    }

    /// The publisher's half: a source that still stands, and a target that is
    /// not there.
    #[test]
    fn the_publisher_is_held_to_the_taxonomy_it_ships() {
        let built = payload(
            "  - subject: facet_value\n    facet: status\n    from: draft\n    to: [outline]\n    \
             because: why\n",
        )
        .expect("it reads");

        let moved = taxonomy("facets:\n  status:\n    values: [outline, current]\n");
        assert!(holds(&built, &moved, "2.0.0").is_empty());

        let unmoved = taxonomy("facets:\n  status:\n    values: [draft, outline]\n");
        assert!(matches!(
            holds(&built, &unmoved, "2.0.0")[..],
            [PayloadError::SourceStands { .. }]
        ));

        let dangling = taxonomy("facets:\n  status:\n    values: [current]\n");
        assert!(matches!(
            holds(&built, &dangling, "2.0.0")[..],
            [PayloadError::TargetAbsent { .. }]
        ));

        // A payload for another release, carried by this one.
        assert!(holds(&built, &moved, "3.0.0")
            .iter()
            .any(|error| matches!(error, PayloadError::NotThisVersion { .. })));
        assert!(holds(&built, &moved, "1.5.0")
            .iter()
            .any(|error| matches!(error, PayloadError::FromItself { .. })));
    }
}
