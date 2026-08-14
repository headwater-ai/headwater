// SPDX-License-Identifier: Apache-2.0
//! The read set of a probe result: which documents a recorded run read, and
//! what the tree in front of you did to them.
//!
//! # The question, and why the corpus tree digest is the wrong instrument
//!
//! [Q8](../../../../docs/decisions/0008-probe-cost-and-cadence.md) asks a run
//! to report "which recorded results the change voided". A probe result is a
//! verdict about one state of the corpus, and a trend over quarters through
//! points of unknown staleness compares nothing.
//!
//! The blunt answer is the `tree` digest, which covers every classified
//! document. It moves on every commit, so every result is stale on every
//! commit and the report says nothing a reader acts on.
//! [Spec 15](../../../../docs/spec/15-the-recorder-contract.md) records that
//! reasoning and defers the sharp answer to here.
//!
//! The sharp answer is this module. A read set is the documents a session was
//! pointed at and the documents it was observed to open, and an edit anywhere
//! else leaves it alone.
//!
//! # It has two halves, and each half is decided by a different record
//!
//! **The declared half is decided by a digest.** [`crate::plan::Plan::read_set`]
//! is taken over every probe of the selection and every document one of them
//! examines, by path and content. The recorder copies it into the run identity
//! the way it copies `tree` and `selection`, so a committed transcript names
//! the digest the run was planned over. Recompose it now and the comparison is
//! exact: it moved, or it did not.
//!
//! A digest names no offender. That is what the second half is for.
//!
//! **The observed half is decided per path.** A recorded tool call carries the
//! path it opened and the identity of what it returned. So a call is a witness
//! for one document, and a witness that disagrees with this corpus names the
//! document that moved. It also reaches documents the plan cannot know about: a
//! session that opened a document no probe examines read that document, and
//! only the transcript records it.
//!
//! The two halves check each other. Where the digest holds and a witness
//! disagrees, the document did not move, so what the recorder wrote as a result
//! identity is not this engine's content digest of that document. That is a
//! statement about the recorder, and this module prints it as one rather than
//! reporting a result as voided by a document that never changed.
//!
//! # An absent `calls` key does not widen the set
//!
//! [Spec 15](../../../../docs/spec/15-the-recorder-contract.md) fixes three
//! states for `calls`, and the middle one is the trap. `calls: []` says the
//! recorder watched and saw no call. No `calls` key says that nothing watched.
//! A read set that read the second as the first would report that such a
//! session opened no document, so no change could ever void it, and the
//! systematically-green failure would land on the one session with the least
//! evidence behind it.
//!
//! So an unwatched session contributes no witness and is named on its own line.
//! It does not widen the declared half either, and the reason is worth stating:
//! the declared half is what the plan pointed the session at, and the plan
//! pointed it at the same documents whether or not anything watched. What is
//! lost is only the observed half of that one session, which is exactly what
//! the line says.
//!
//! # Nothing here decides an exit status
//!
//! [`Staleness::render`] returns a string. There is no `Result`, no status and
//! no error arm, so no caller of this module can branch a build on a result
//! going stale. That is the shape [spec 5](../../../../docs/spec/05-ai-integration.md#two-tiers-and-the-cadence-follows-the-purpose)
//! asks for: a probe result going stale is a fact about a measurement, and an
//! exit status that carried it would put a model's behavior on a build.

use crate::intake::Record;
use crate::plan::{Because, Plan};
use headwater_census::census::{Census, Outcome};

/// One document a recorded tool call named, with the identity the recorder
/// observed it return.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Witness {
    pub path: String,
    /// The `result` of the call, which is the identity of what it returned.
    pub recorded: String,
    /// The event this call was in, one-based, so a reader finds it by counting.
    pub event: usize,
    /// The call inside that event, one-based.
    pub call: usize,
}

/// Why a document is in the read set of a recorded result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provenance {
    /// A probe of the selection. Its prose is the task a session was given.
    Probe,
    /// A probe of the selection examines it.
    Examined,
    /// A recorded call opened it and no probe of the selection names it.
    Opened,
}

impl Provenance {
    pub fn name(self) -> &'static str {
        match self {
            Provenance::Probe => "probe",
            Provenance::Examined => "examined",
            Provenance::Opened => "opened",
        }
    }

    /// Whether the declared digest covers this member.
    pub fn declared(self) -> bool {
        match self {
            Provenance::Probe | Provenance::Examined => true,
            Provenance::Opened => false,
        }
    }
}

/// One member of the read set of a committed result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    pub path: String,
    pub because: Provenance,
    /// What this corpus holds for it now, and `None` where this corpus holds no
    /// classified document at that path.
    pub now: Option<String>,
    /// The first recorded call that named it.
    pub witness: Option<Witness>,
}

impl Member {
    /// Whether a witness disagrees with what this corpus holds.
    ///
    /// It is `false` where nothing witnessed the member and where this corpus
    /// holds nothing at the path, because neither is a disagreement. What each
    /// of those two is instead is reported on its own line.
    pub fn disagrees(&self) -> bool {
        match (&self.witness, &self.now) {
            (Some(witness), Some(now)) => &witness.recorded != now,
            _ => false,
        }
    }
}

/// What the tree in front of this run did to one recorded result.
///
/// The five arms are the closed set a reader is offered, and every match over
/// them in this crate is exhaustive with no wildcard arm. A sixth state has to
/// be named here before anything can report it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// There is no read set to compare. Either the intake refused the
    /// transcript, or no plan composed one over this corpus.
    Unusable,
    /// The declared digest holds and no witness disagrees.
    Stands,
    /// The declared digest moved: a probe of the selection, or a document one
    /// of them examines, is not what this run was planned over.
    SetMoved,
    /// The declared digest holds and a document a session opened outside the
    /// set is not what the recorder observed.
    OpenedFileMoved,
    /// Both of the two above.
    Both,
}

impl Verdict {
    /// Whether this result is still about the corpus in front of the reader.
    pub fn stands(self) -> bool {
        match self {
            Verdict::Stands => true,
            Verdict::Unusable | Verdict::SetMoved | Verdict::OpenedFileMoved | Verdict::Both => {
                false
            }
        }
    }
}

/// The read set of one recorded result, held against the tree in front of it.
#[derive(Clone, Debug)]
pub struct Staleness {
    /// The read-set digest the transcript recorded, and empty where the intake
    /// refused the transcript.
    pub recorded: String,
    /// The read-set digest this corpus composes now.
    pub composed: String,
    /// Every member, in path order.
    pub members: Vec<Member>,
    /// Sessions that recorded no `calls` key, as `(probe, session)`.
    pub unwatched: Vec<(String, String)>,
    /// Calls that named a path this corpus holds no classified document at.
    pub outside: Vec<Witness>,
    /// `examines` targets that are not documents of this corpus.
    pub anchors: Vec<String>,
    /// Why there is nothing to compare, where the intake refused the file.
    pub unusable: Option<String>,
}

impl Staleness {
    /// Hold one recorded transcript against the plan this corpus composes now.
    ///
    /// It never fails. A transcript the intake refused produces a [`Staleness`]
    /// with [`Staleness::unusable`] set, because a refused transcript names no
    /// read set and saying so is the report.
    pub fn over(record: &Record, plan: &Plan, census: &Census) -> Staleness {
        let mut staleness = Staleness {
            recorded: String::new(),
            composed: plan.read_set.clone(),
            members: Vec::new(),
            unwatched: Vec::new(),
            outside: Vec::new(),
            anchors: plan.anchors.clone(),
            unusable: None,
        };
        // An empty composed digest is a plan that stopped before it composed
        // one, and never a corpus over which the read set is empty. A
        // comparison against it would report every committed result as voided
        // by a refusal that has nothing to do with the tree.
        if plan.read_set.is_empty() {
            staleness.unusable = Some(match &plan.refusal {
                Some(refusal) => format!("`headwater probe plan` composed no read set: {refusal}"),
                None => {
                    "`headwater probe plan` composed no read set and gave no reason".to_string()
                }
            });
            return staleness;
        }
        let Some(identity) = &record.identity else {
            staleness.unusable = Some(match &record.refusal {
                Some(refusal) => refusal.to_string(),
                None => "it carries no run identity".to_string(),
            });
            return staleness;
        };
        staleness.recorded = identity.read_set.clone();

        for read in &plan.reads {
            staleness.members.push(Member {
                path: read.path.clone(),
                because: match read.because {
                    Because::Probe => Provenance::Probe,
                    Because::Examined => Provenance::Examined,
                },
                now: read.digest.clone(),
                witness: None,
            });
        }

        // The observed half. A call is read in event order, and the first call
        // that named a path is the witness for it: a session that read one
        // document twice recorded one identity for it, and the second reading
        // decides nothing the first did not.
        for event in &record.events {
            let Some(calls) = &event.calls else {
                // Nothing watched this session. It widens nothing, and the
                // reason is in this module's own documentation.
                let key = (event.probe.clone(), event.session.clone());
                if !staleness.unwatched.contains(&key) {
                    staleness.unwatched.push(key);
                }
                continue;
            };
            for (index, call) in calls.iter().enumerate() {
                let witness = |path: &str| Witness {
                    path: path.to_string(),
                    recorded: call.result.clone(),
                    event: event.at,
                    call: index + 1,
                };
                match member_of(&staleness.members, &call.argument) {
                    Some(at) => {
                        if staleness.members[at].witness.is_none() {
                            let path = staleness.members[at].path.clone();
                            staleness.members[at].witness = Some(witness(&path));
                        }
                    }
                    None => match classified(census, &call.argument) {
                        Some((path, digest)) => staleness.members.push(Member {
                            path: path.clone(),
                            because: Provenance::Opened,
                            now: digest,
                            witness: Some(witness(&path)),
                        }),
                        // A path this corpus holds no classified document at.
                        // A read set is a list of corpus paths, so nothing here
                        // decides anything about it.
                        None => {
                            let seen = witness(&call.argument);
                            if !staleness.outside.contains(&seen) {
                                staleness.outside.push(seen);
                            }
                        }
                    },
                }
            }
        }
        staleness.members.sort_by(|a, b| a.path.cmp(&b.path));
        staleness
    }

    /// Every member a witness says is not what the recorder observed.
    pub fn moved(&self) -> Vec<&Member> {
        self.members
            .iter()
            .filter(|member| member.disagrees())
            .collect()
    }

    /// The verdict, which is the product of the two halves.
    pub fn verdict(&self) -> Verdict {
        if self.unusable.is_some() {
            return Verdict::Unusable;
        }
        let set_moved = self.recorded != self.composed;
        let opened_moved = self
            .members
            .iter()
            .any(|member| member.because == Provenance::Opened && member.disagrees());
        match (set_moved, opened_moved) {
            (false, false) => Verdict::Stands,
            (true, false) => Verdict::SetMoved,
            (false, true) => Verdict::OpenedFileMoved,
            (true, true) => Verdict::Both,
        }
    }

    /// The report, which no caller branches on.
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let verdict = self.verdict();

        match verdict {
            Verdict::Unusable => {
                let reason = self.unusable.as_deref().unwrap_or("there is no read set");
                let _ = writeln!(
                    out,
                    "Nothing here decides whether this result is stale: {reason}"
                );
                return out;
            }
            Verdict::Stands => {
                let _ = writeln!(
                    out,
                    "Nothing this run read has moved. The read set covers {}, and the digest the \
                     transcript recorded is the digest this corpus composes.",
                    crate::plural(self.declared(), "document")
                );
            }
            Verdict::SetMoved => {
                let _ = writeln!(
                    out,
                    "**This result is stale.** The read set covers {}, and the digest the \
                     transcript recorded is not the digest this corpus composes, so a document \
                     this run was planned over has moved.",
                    crate::plural(self.declared(), "document")
                );
            }
            Verdict::OpenedFileMoved => {
                let _ = writeln!(
                    out,
                    "**This result is stale.** The read set holds still, and a document a session \
                     opened is not what the recorder observed."
                );
            }
            Verdict::Both => {
                let _ = writeln!(
                    out,
                    "**This result is stale.** The read set covers {}, the digest the transcript \
                     recorded is not the digest this corpus composes, and a document a session \
                     opened is not what the recorder observed either.",
                    crate::plural(self.declared(), "document")
                );
            }
        }
        let _ = writeln!(out, "recorded {}", self.recorded);
        let _ = writeln!(out, "now      {}", self.composed);
        let _ = writeln!(out);

        for member in &self.members {
            let _ = writeln!(
                out,
                "- {} ({}) {}",
                member.path,
                member.because.name(),
                member.now.as_deref().unwrap_or("no such document"),
            );
            match &member.witness {
                None => {
                    let _ = writeln!(
                        out,
                        "    no recorded call named it, so no witness of this transcript says \
                         whether it moved"
                    );
                }
                Some(witness) => match member.disagrees() {
                    true => {
                        let _ = writeln!(
                            out,
                            "    event {} call {} observed {}, so this document moved",
                            witness.event, witness.call, witness.recorded
                        );
                    }
                    false => {
                        let _ = writeln!(
                            out,
                            "    event {} call {} observed the same identity, so this document \
                             stands",
                            witness.event, witness.call
                        );
                    }
                },
            }
        }

        // Where the digest holds and a witness disagrees, the document did not
        // move. So the disagreement is about the recorder rather than about the
        // corpus, and reporting it as staleness would void a result over a
        // document that nobody touched.
        let confused: Vec<&Member> = match verdict {
            Verdict::Stands | Verdict::OpenedFileMoved => self
                .members
                .iter()
                .filter(|member| member.because.declared() && member.disagrees())
                .collect(),
            Verdict::Unusable | Verdict::SetMoved | Verdict::Both => Vec::new(),
        };
        if !confused.is_empty() {
            let _ = writeln!(out);
            let _ = writeln!(
                out,
                "{} of the read set carry a recorded identity that this corpus does not hold, and \
                 the read-set digest says none of them moved. So what the recorder wrote as a \
                 result identity is not the content digest this engine takes over a document, and \
                 no comparison over one of those calls decides anything.",
                crate::plural(confused.len(), "member")
            );
        }

        if !self.unwatched.is_empty() {
            let _ = writeln!(out);
            let _ = writeln!(
                out,
                "{} recorded no `calls` key, so nothing observed what it opened: {}. The read set \
                 still covers what the plan pointed it at. What no list covers is a document such \
                 a session opened that no probe of the selection names.",
                crate::plural(self.unwatched.len(), "session"),
                self.unwatched
                    .iter()
                    .map(|(probe, session)| format!("{probe} / {session}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        if !self.outside.is_empty() {
            let _ = writeln!(out);
            let _ = writeln!(
                out,
                "{} named a path this corpus classifies no document at, and a read set is a list \
                 of corpus paths: {}.",
                crate::plural(self.outside.len(), "recorded call"),
                self.outside
                    .iter()
                    .map(|witness| witness.path.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        if !self.anchors.is_empty() {
            let _ = writeln!(out);
            let _ = writeln!(
                out,
                "{} an `examines` edge names is outside this corpus, so it is in no read set and \
                 nothing here decides about it: {}.",
                crate::plural(self.anchors.len(), "target"),
                self.anchors.join(", ")
            );
        }
        out
    }

    /// How many members the declared digest covers.
    fn declared(&self) -> usize {
        self.members
            .iter()
            .filter(|member| member.because.declared())
            .count()
    }
}

/// The member a tool-call argument names, by the rule the grader compares an
/// argument with.
fn member_of(members: &[Member], argument: &str) -> Option<usize> {
    members
        .iter()
        .position(|member| crate::grade::names_path(argument, &member.path))
}

/// The classified document a tool-call argument names, with its digest.
fn classified(census: &Census, argument: &str) -> Option<(String, Option<String>)> {
    census
        .rows
        .iter()
        .filter(|row| matches!(row.outcome, Outcome::Typed { .. }))
        .find(|row| crate::grade::names_path(argument, &row.path))
        .map(|row| (row.path.clone(), row.digest.clone()))
}
