// SPDX-License-Identifier: Apache-2.0
//! A recorded transcript, read back, and the five things this engine confirms.
//!
//! # This is the reproducible half of an unreproducible mechanism
//!
//! [`Record::read`] is a pure function of the transcript, the tree it is read
//! over, and this code. Run it twice on one file and it writes one set of
//! bytes. The behavior a model produced cannot be reproduced. Every statement
//! this engine makes about the record of that behavior can be, and that is the
//! difference between a probe result and an opinion.
//!
//! # What is confirmed
//!
//! 1. **The taxonomy.** The transcript names the lock it was planned against. A
//!    transcript planned against another one is refused whole: a rate over
//!    documents that a different taxonomy typed is a rate about a corpus that
//!    is not in front of this run.
//! 2. **The identity is complete.** Every member of spec 5's run identity is
//!    present. An incomplete identity is a measurement nobody can locate, and a
//!    missing served version is the one that
//!    [spec 5](../../../../docs/spec/05-ai-integration.md#a-run-produces-a-snapshot-and-a-document)
//!    names outright: "A model name is not a pin."
//! 3. **Membership.** Every probe an event names is a classified probe of this
//!    corpus. A probe that is not there is the plainest sign that the
//!    transcript and the corpus are not the same measurement.
//! 4. **No prose.** Every key of the transcript is one of the closed sets
//!    below, and one key outside them refuses the file. Spec 5 says the
//!    transcript "holds no model prose. That omission is the enforcement", and
//!    an omission that nothing tests is a claim rather than an enforcement.
//! 5. **The realized cost is present.** Spec 4 asks the adaptive layer to
//!    report the cost of its own instrument, and a run that did not record what
//!    it spent leaves that number to a guess.
//!
//! # What is not confirmed here, and by whom
//!
//! Every verdict. This module counts events and refuses malformed ones, and it
//! evaluates no expectation against anything. The grader is
//! [#85](https://github.com/headwater-ai/headwater/issues/85): a pure function
//! of the transcript, the expectations and its own version. The seam is exactly
//! this type — a `Record` that carried an expectation's verdict would be a
//! grader written where nobody is looking for one.

use crate::{Arm, Cents, Tier};
use headwater_census::census::{Census, Outcome};
use headwater_graph::Config;
use headwater_yaml::value::{Mapping, Value};

/// The kind a transcript document is.
pub const KIND: &str = "probe_transcript";

/// Every key a run identity may carry. Spec 5's nine members, plus the two the
/// harness needs to hold a run to the plan that fixed it.
pub const IDENTITY_KEYS: [&str; 11] = [
    "model",
    "served_version",
    "tree",
    "lock",
    "selection",
    "seed",
    "harness",
    "tier",
    "arm",
    "at",
    "cost_cents",
];

/// Every key one event may carry.
pub const EVENT_KEYS: [&str; 5] = ["probe", "session", "calls", "produced", "answer"];

/// Every key one tool call may carry.
///
/// Spec 5: a transcript holds "the ordered tool-call events with their
/// arguments and result identities". A result *identity* and not a result: the
/// bytes a tool returned are the corpus, and a transcript that inlined them
/// would be a second copy of the tree it already names by digest.
pub const CALL_KEYS: [&str; 3] = ["tool", "argument", "result"];

/// Every key one produced artifact may carry.
///
/// This list is the one thing the grader found wrong in the intake it
/// inherited. The closed-key test ran over the event and over the tool call and
/// never over a produced artifact, so a transcript carrying `produced: [{note:
/// I reasoned as follows}]` was accepted whole. "The transcript holds no model
/// prose" was an enforcement in two of the three places a key can appear.
///
/// `cites` and `findings` are what the recorder observed about the artifact,
/// and each is the state that changes a verdict rather than a digest of it. See
/// [`Produced`].
pub const PRODUCED_KEYS: [&str; 4] = ["path", "result", "cites", "findings"];

/// The tree a transcript is read over.
pub struct Tree<'a> {
    pub census: &'a Census,
    pub config: &'a Config,
    /// The digest of the lock this tree carries.
    pub lock: &'a str,
}

/// The part of the run identity that belongs to the run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    pub model: String,
    pub served_version: String,
    pub tree: String,
    pub lock: String,
    pub selection: String,
    pub seed: String,
    pub harness: String,
    pub tier: Tier,
    pub arm: Arm,
    pub at: String,
    pub cost: Cents,
}

/// Why a whole transcript reported nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The document carries no fenced block under the heading named.
    NoBlock(&'static str),
    Unparsed {
        block: &'static str,
        why: String,
    },
    Malformed(String),
    /// A key outside the closed set. This is the enforcement of "the transcript
    /// holds no model prose".
    KeyNotPermitted {
        block: &'static str,
        key: String,
        permitted: &'static [&'static str],
    },
    Missing(&'static str),
    TaxonomyMoved {
        claimed: String,
        tree: String,
    },
    UnknownTier(String),
    UnknownArm(String),
    CostNotACount(String),
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refusal::NoBlock(heading) => write!(
                f,
                "it carries no fenced `yaml` block under `## {heading}`, and `headwater probe \
                 plan` prints the shape it wants"
            ),
            Refusal::Unparsed { block, why } => {
                write!(f, "the `{block}` block did not parse as YAML: {why}")
            }
            Refusal::Malformed(what) => write!(f, "{what}"),
            Refusal::KeyNotPermitted {
                block,
                key,
                permitted,
            } => write!(
                f,
                "the `{block}` block carries `{key}`, which is not one of its keys: {}. A \
                 transcript holds no model prose, and refusing a key nobody declared is what makes \
                 that an enforcement rather than a request",
                permitted.join(", ")
            ),
            Refusal::Missing(what) => write!(
                f,
                "the run identity carries no `{what}`, and an incomplete identity is a measurement \
                 nobody can locate again"
            ),
            Refusal::TaxonomyMoved { claimed, tree } => write!(
                f,
                "it was planned against taxonomy {claimed} and this tree carries {tree}"
            ),
            Refusal::UnknownTier(found) => write!(
                f,
                "`{found}` is not a tier. The tiers are `regression` and `campaign`"
            ),
            Refusal::UnknownArm(found) => write!(
                f,
                "`{found}` is not an arm. The arms are `present` and `absent`"
            ),
            Refusal::CostNotACount(found) => write!(
                f,
                "`cost_cents` is `{found}`, and the realized cost has to be a whole number of \
                 cents. A run that recorded no cost leaves the cost of the instrument to a guess"
            ),
        }
    }
}

/// Why one event did not count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reason {
    Missing(&'static str),
    NotAProbe(String),
}

impl std::fmt::Display for Reason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reason::Missing(what) => write!(f, "it carries no {what}"),
            Reason::NotAProbe(id) => write!(
                f,
                "`{id}` is not a classified probe of this corpus, so nothing declares what a \
                 session against it was expected to do"
            ),
        }
    }
}

/// One event this engine refused, and where in the block it was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rejected {
    /// One-based, in the order the block lists them.
    pub at: usize,
    pub reason: Reason,
}

/// One tool call, as the recorder observed it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Call {
    pub tool: String,
    pub argument: String,
    /// The identity of what the call returned, and never the bytes.
    pub result: String,
}

/// One artifact a session produced.
///
/// # Two of these four keys carry a derivation, and that is deliberate
///
/// A digest of an artifact is an identity, and an identity answers no
/// expectation: two artifacts that cite different identifiers have different
/// digests and the digest says which of them cited what. So `cited` and
/// `patched` are predicates that a transcript of identities alone cannot
/// evaluate, and the choice is a derived key or a dead predicate form.
///
/// The key is safe where the recorder computes it **the same way for every
/// probe and never consults the probe**. `cites` is every identifier of this
/// corpus that appears in the artifact, and `findings` is every rule that
/// reported over it. Neither reads the expectation, and the grader does all the
/// selecting and all the comparing. A recorder that wrote only the identifiers
/// a probe named, or only the rule a probe's oracle names, would be a grader
/// with no fixture set and no version.
///
/// `findings` is an `Option` and never a defaulted empty list. `findings: []`
/// says the artifact was checked and nothing reported. An absent key says
/// nothing checked it. A grader that read the second as the first would return
/// a passing `patched` verdict for every run that never ran the oracle, which
/// is the systematically-green failure this whole component exists to avoid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Produced {
    pub path: String,
    /// The identity of the artifact, and never the bytes.
    pub result: String,
    /// Every identifier of this corpus that appears in the artifact, sorted.
    pub cites: Vec<String>,
    /// Every rule that reported over the artifact, and `None` where nothing
    /// checked it.
    pub findings: Option<Vec<String>>,
}

/// One event the intake accepted, with everything a predicate over the run
/// record reads.
///
/// This type is the whole of what [`crate::grade`] is given, and it carries no
/// verdict and no expectation. The intake counts and refuses; the grader
/// decides. A field here that answered an expectation would be a grader written
/// where nobody is looking for one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    /// One-based, in the order the block lists them. A witness cites it, so a
    /// reader finds the event a verdict came from by counting.
    pub at: usize,
    pub probe: String,
    pub session: String,
    /// The ordered tool calls, and `None` where the event carries no `calls`
    /// key at all.
    pub calls: Option<Vec<Call>>,
    /// The artifacts the session produced, and `None` where the event carries
    /// no `produced` key at all.
    pub produced: Option<Vec<Produced>>,
    pub answer: Answer,
}

/// What an event says about the final answer, in three states rather than two.
///
/// `calls`, `produced` and this key each carry the same distinction, and it is
/// the distinction a grader is wrong without. An empty list and an absent key
/// are different facts about the run: one says the recorder watched and saw
/// nothing, and the other says nothing watched. A grader that read them as one
/// value reports a verdict about the recorder as a verdict about the corpus.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    /// No `answer` key. Nothing recorded whether the session answered.
    Unrecorded,
    /// `answer: null`. The recorder observed the session end with no answer.
    Absent,
    /// A value. A quoted `"null"` reaches here, because the closed set belongs
    /// to the probe and this engine does not decide which strings are values.
    Value(String),
}

/// One transcript, read back.
#[derive(Clone, Debug)]
pub struct Record {
    pub identity: Option<Identity>,
    /// Events the block held, before any test.
    pub read: usize,
    /// Distinct probes the accepted events name, sorted.
    pub probes: Vec<String>,
    /// Sessions the accepted events name, counted distinct per probe.
    pub sessions: usize,
    /// Tool calls in the accepted events.
    pub calls: usize,
    /// The accepted events, in the order the block listed them.
    pub events: Vec<Event>,
    pub rejected: Vec<Rejected>,
    pub refusal: Option<Refusal>,
    /// Probes this corpus declares, which is the denominator the run covered
    /// part of.
    pub declared: usize,
}

impl Record {
    /// Read a transcript document over a tree.
    ///
    /// It never fails. A transcript this engine cannot use produces a record
    /// with a [`Refusal`] on it, which is a thing a reader sees rather than a
    /// status a caller branches on.
    pub fn read(source: &str, tree: &Tree<'_>) -> Record {
        let mut record = Record {
            identity: None,
            read: 0,
            probes: Vec::new(),
            sessions: 0,
            calls: 0,
            events: Vec::new(),
            rejected: Vec::new(),
            refusal: None,
            declared: 0,
        };

        let mut known = Vec::new();
        for row in &tree.census.rows {
            let Outcome::Typed { kind, .. } = &row.outcome else {
                continue;
            };
            if kind != crate::plan::KIND {
                continue;
            }
            record.declared += 1;
            if let Some(id) = row
                .document
                .as_ref()
                .and_then(|document| document.facets.get(&tree.config.identifier_facet))
                .and_then(|entry| entry.value.as_scalar())
            {
                known.push(id.text.clone());
            }
        }

        match identity(source, tree) {
            Ok(identity) => record.identity = Some(identity),
            Err(refusal) => {
                record.refusal = Some(refusal);
                return record;
            }
        }

        let Some(block) = fenced(source, "Events") else {
            record.refusal = Some(Refusal::NoBlock("Events"));
            return record;
        };
        let loaded = match headwater_yaml::load(block) {
            Ok(loaded) => loaded,
            Err(errors) => {
                record.refusal = Some(Refusal::Unparsed {
                    block: "Events",
                    why: errors
                        .iter()
                        .map(|error| error.to_string())
                        .collect::<Vec<_>>()
                        .join("; "),
                });
                return record;
            }
        };
        let Some(items) = loaded.value.as_seq() else {
            record.refusal = Some(Refusal::Malformed(
                "the `Events` block is not a sequence of events".into(),
            ));
            return record;
        };

        record.read = items.len();
        let mut sessions: Vec<(String, String)> = Vec::new();
        for (index, item) in items.iter().enumerate() {
            let at = index + 1;
            let Some(map) = item.value.as_map() else {
                record.rejected.push(Rejected {
                    at,
                    reason: Reason::Missing("mapping of its own"),
                });
                continue;
            };
            if let Some(refusal) = closed(map, "Events", &EVENT_KEYS) {
                record.refusal = Some(refusal);
                return record;
            }
            let Some(probe) = text(map, "probe") else {
                record.rejected.push(Rejected {
                    at,
                    reason: Reason::Missing("probe"),
                });
                continue;
            };
            if !known.contains(&probe) {
                record.rejected.push(Rejected {
                    at,
                    reason: Reason::NotAProbe(probe),
                });
                continue;
            }
            let session = text(map, "session").unwrap_or_default();

            let mut calls = None;
            if let Some(items) = map.get("calls").and_then(|entry| entry.value.as_seq()) {
                let calls = calls.insert(Vec::new());
                for call in items {
                    let Some(fields) = call.value.as_map() else {
                        record.refusal = Some(Refusal::Malformed(
                            "a tool call is not a mapping of its own".into(),
                        ));
                        return record;
                    };
                    if let Some(refusal) = closed(fields, "Events", &CALL_KEYS) {
                        record.refusal = Some(refusal);
                        return record;
                    }
                    record.calls += 1;
                    calls.push(Call {
                        tool: text(fields, "tool").unwrap_or_default(),
                        argument: text(fields, "argument").unwrap_or_default(),
                        result: text(fields, "result").unwrap_or_default(),
                    });
                }
            }

            let mut produced = None;
            if let Some(items) = map.get("produced").and_then(|entry| entry.value.as_seq()) {
                let produced = produced.insert(Vec::new());
                for artifact in items {
                    let Some(fields) = artifact.value.as_map() else {
                        record.refusal = Some(Refusal::Malformed(
                            "a produced artifact is not a mapping of its own".into(),
                        ));
                        return record;
                    };
                    if let Some(refusal) = closed(fields, "Events", &PRODUCED_KEYS) {
                        record.refusal = Some(refusal);
                        return record;
                    }
                    let mut cites = strings(fields, "cites").unwrap_or_default();
                    cites.sort();
                    cites.dedup();
                    produced.push(Produced {
                        path: text(fields, "path").unwrap_or_default(),
                        result: text(fields, "result").unwrap_or_default(),
                        cites,
                        findings: strings(fields, "findings").map(|mut rules| {
                            rules.sort();
                            rules.dedup();
                            rules
                        }),
                    });
                }
            }

            record.events.push(Event {
                at,
                probe: probe.clone(),
                session: session.clone(),
                calls,
                produced,
                answer: answer(map),
            });

            let key = (probe.clone(), session);
            if !sessions.contains(&key) {
                sessions.push(key);
            }
            if !record.probes.contains(&probe) {
                record.probes.push(probe);
            }
        }
        record.probes.sort();
        record.sessions = sessions.len();
        record
    }

    /// The report, in the engine's own words.
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();

        if let Some(refusal) = &self.refusal {
            let _ = writeln!(out, "This transcript recorded nothing usable: {refusal}");
            return out;
        }

        let identity = match &self.identity {
            Some(identity) => identity,
            None => {
                let _ = writeln!(out, "This transcript carries no run identity.");
                return out;
            }
        };

        let _ = writeln!(
            out,
            "A {} run in the {} arm, on {} at {}.",
            identity.tier.name(),
            identity.arm.name(),
            identity.model,
            identity.at
        );
        let _ = writeln!(
            out,
            "served version {}, tree {}, selection {}, seed {}, harness {}.",
            identity.served_version,
            identity.tree,
            identity.selection,
            identity.seed,
            identity.harness
        );
        let _ = writeln!(
            out,
            "realized cost {}, which the adaptive layer reads as the cost of its own instrument.",
            crate::dollars(identity.cost)
        );
        let _ = writeln!(out);

        let _ = writeln!(
            out,
            "{} events over {} of the {} probes this corpus declares, in {} sessions and {} tool \
             calls.",
            self.read,
            self.probes.len(),
            self.declared,
            self.sessions,
            self.calls
        );

        if !self.rejected.is_empty() {
            let _ = writeln!(out);
            let _ = writeln!(
                out,
                "{} of {} events did not count:",
                self.rejected.len(),
                self.read
            );
            for rejected in &self.rejected {
                let _ = writeln!(out, "  event {}: {}", rejected.at, rejected.reason);
            }
        }

        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "The engine confirmed the taxonomy, the completeness of the run identity, the \
             membership of every probe named, that no key outside the closed set appears, and \
             that a realized cost was recorded. It graded nothing: a verdict is a function of \
             this transcript, the expectations these probes declare and a grader version, and no \
             grader ships in this engine yet."
        );
        out
    }
}

/// The run identity block, read and tested.
fn identity(source: &str, tree: &Tree<'_>) -> Result<Identity, Refusal> {
    let block = fenced(source, "Run identity").ok_or(Refusal::NoBlock("Run identity"))?;
    let loaded = headwater_yaml::load(block).map_err(|errors| Refusal::Unparsed {
        block: "Run identity",
        why: errors
            .iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("; "),
    })?;
    let Value::Map(map) = &loaded.value else {
        return Err(Refusal::Malformed(
            "the `Run identity` block is not a mapping".into(),
        ));
    };
    if let Some(refusal) = closed(map, "Run identity", &IDENTITY_KEYS) {
        return Err(refusal);
    }

    let need = |key: &'static str| text(map, key).ok_or(Refusal::Missing(key));
    let lock = need("lock")?;
    if lock != tree.lock {
        return Err(Refusal::TaxonomyMoved {
            claimed: lock,
            tree: tree.lock.to_string(),
        });
    }
    let tier_name = need("tier")?;
    let tier = Tier::read(&tier_name).ok_or(Refusal::UnknownTier(tier_name))?;
    let arm_name = need("arm")?;
    let arm = Arm::read(&arm_name).ok_or(Refusal::UnknownArm(arm_name))?;
    let raw_cost = need("cost_cents")?;
    let cost = raw_cost
        .parse::<Cents>()
        .map_err(|_| Refusal::CostNotACount(raw_cost))?;

    Ok(Identity {
        model: need("model")?,
        served_version: need("served_version")?,
        tree: need("tree")?,
        lock,
        selection: need("selection")?,
        seed: need("seed")?,
        harness: need("harness")?,
        tier,
        arm,
        at: need("at")?,
        cost,
    })
}

/// The closed-key test, which is how "no model prose" is enforced.
fn closed(
    map: &Mapping,
    block: &'static str,
    permitted: &'static [&'static str],
) -> Option<Refusal> {
    for entry in map.iter() {
        if !permitted.contains(&entry.key.value.as_str()) {
            return Some(Refusal::KeyNotPermitted {
                block,
                key: entry.key.value.clone(),
                permitted,
            });
        }
    }
    None
}

/// The first fenced `yaml` block under a level-two heading.
///
/// A transcript is a document of this corpus, so every check reads it and the
/// body around the blocks is the prose a person wrote about the run. The blocks
/// are what this engine reads, and the heading is what says which is which.
pub fn fenced<'a>(source: &'a str, heading: &str) -> Option<&'a str> {
    let wanted = format!("## {heading}");
    let mut lines = source.lines();
    let mut offset = 0usize;
    let mut under = false;
    while let Some(line) = lines.next() {
        offset += line.len() + 1;
        let trimmed = line.trim_end();
        if trimmed == wanted {
            under = true;
            continue;
        }
        if under && trimmed.starts_with("## ") {
            return None;
        }
        if under && (trimmed == "```yaml" || trimmed == "```yml") {
            let start = offset;
            let mut end = start;
            for line in lines {
                if line.trim_end() == "```" {
                    return Some(&source[start..end]);
                }
                end += line.len() + 1;
            }
            return None;
        }
    }
    None
}

fn text(map: &Mapping, key: &str) -> Option<String> {
    Some(map.get(key)?.value.as_scalar()?.text.clone())
}

/// A sequence of scalars, and `None` where the key is absent.
///
/// An absent key and an empty sequence are different facts, and
/// [`Produced::findings`] is the place where reading one as the other returns a
/// verdict nobody measured. So this returns `None` for the first and
/// `Some(vec![])` for the second, and every caller decides which it wants.
fn strings(map: &Mapping, key: &str) -> Option<Vec<String>> {
    let items = map.get(key)?.value.as_seq()?;
    Some(
        items
            .iter()
            .filter_map(|item| item.value.as_scalar())
            .map(|scalar| scalar.text.clone())
            .collect(),
    )
}

/// The final answer, in the three states [`Answer`] separates.
fn answer(map: &Mapping) -> Answer {
    let Some(scalar) = map.get("answer").and_then(|entry| entry.value.as_scalar()) else {
        return Answer::Unrecorded;
    };
    match headwater_yaml::core_schema::as_null(scalar) {
        true => Answer::Absent,
        false => Answer::Value(scalar.text.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOCUMENT: &str = "\
# A run

## Run identity

```yaml
model: a-model
```

## Events

```yaml
- probe: PROBE-HW-one
```
";

    #[test]
    fn a_block_is_found_under_its_own_heading_and_nowhere_else() {
        assert_eq!(fenced(DOCUMENT, "Run identity"), Some("model: a-model\n"));
        assert_eq!(fenced(DOCUMENT, "Events"), Some("- probe: PROBE-HW-one\n"));
        assert_eq!(fenced(DOCUMENT, "Cost"), None);
    }

    #[test]
    fn a_heading_with_no_block_before_the_next_heading_finds_nothing() {
        let source = "## Run identity\n\nnothing here\n\n## Events\n\n```yaml\n- a\n```\n";
        assert_eq!(fenced(source, "Run identity"), None);
    }
}
