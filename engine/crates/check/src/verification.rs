// SPDX-License-Identifier: Apache-2.0
//! A Graph-origin check: a verification's freshness against the committed
//! observation snapshot and the acceptance criterion it proves.
//!
//! # The gap this closes
//!
//! [HW-DR-0073](../../../../docs/decisions/0073-a-verification-is-a-kind-and-its-identity-is-minted-rather-than-found-in-the-code-that-cites-it.md)
//! ruling 3: "A verification carries three states, which are `declared`,
//! `observed at commit X` and `suspect` ... A verification is `suspect` when
//! the criterion it proves changed after the snapshot commit, which is the
//! rule DOORS states as P.6." Nothing read that comparison before this rule:
//! [`crate::observation::Observations`] could name a verification and the
//! commit it ran at, and nothing turned "the criterion changed since" into a
//! finding. [#937](https://github.com/headwater-ai/headwater/issues/937) is
//! the report, and this module is the rule.
//!
//! # Content stands in for history, and the module comment on why
//!
//! "Changed after the snapshot commit" reads as a question for version
//! control, and [`crate::observation`]'s own module comment states why this
//! crate does not ask one: it runs no VCS command and opens no file that a
//! caller did not name. The comparison this rule actually needs is narrower
//! than the sentence suggests. A snapshot that recorded the acceptance
//! criterion's content digest at the moment it observed the verification, set
//! against that criterion's digest today, answers the same question a commit
//! walk would: the two disagree exactly when the criterion's bytes moved at
//! some point after the snapshot, which is what "changed after the snapshot
//! commit" means. [`crate::observation::Observation::Verification`] is that
//! recorded digest, and [`EdgeEnd::digest`](crate::scope::EdgeEnd::digest) is
//! today's, off the same census read every other edge-scoped rule already
//! trusts. Neither needs a git command, and the `commit` field stays
//! provenance: a person reading the snapshot still sees which run it was, and
//! [HW-OBL-0199](../../../../docs/obligations/0199-an-observation-snapshot-records-a-commit-and-nothing-reads-it-back.md)
//! is where the syntax check on that field, and the boundary it stays inside
//! of, are recorded.
//!
//! # What is not suspect, and where the other two states are reported
//!
//! `declared` — no committed snapshot names this verification yet — is a
//! pass, not a finding: a verification with nothing observed yet is not an
//! author's mistake, it is a project not yet at that stage. `observed at
//! <commit>` — a snapshot names it and the criterion has not moved since — is
//! also a pass, for the same reason [`crate::basis`] passes a warrant that
//! supports its claim: only `suspect` needs a person to act.
//!
//! A pass is not a silence, though. [`Block`] is the verification block of
//! the report, and it names every verification of the corpus with its state,
//! the two passing states included. The register already lists every
//! unobserved control by name for the same reason: a run that only counts
//! cannot show whether a verification was ever observed or was only
//! declared ([#937](https://github.com/headwater-ai/headwater/issues/937)).
//! The block is a report block and not a finding, so a green run still has
//! no findings.
//!
//! A snapshot that did not read is none of the three states. A run that
//! could not read the file, or the entry for one verification, does not know
//! whether an entry names it, so the state is `unknown` and the rule
//! decides nothing for that instance. Reading it as `declared` turned a
//! suspect verification green in the block, which is the veto on #1056.
//! `control.observation.invalid` reports the file or the entry. An entry
//! that names no verification of the corpus is named too, rather than
//! dropped. The rule and the block both call [`compare`], so the state a
//! finding reports and the state the block prints cannot disagree.
//!
//! # The denominator
//!
//! Every relation the taxonomy declares whose target kind includes
//! `verification`, and no other, the same declaration-driven shape
//! [`crate::basis`] takes for the `evidence` family: a taxonomy that renames
//! `proven_by` or adds a second relation reaching a verification is read by
//! this rule unchanged, because the rule holds no relation name of its own.
//!
//! # The three silences
//!
//! An entry with no declared half passes: an author who wrote only the
//! inverse half named nothing for this rule to anchor a finding on, and a
//! reciprocity rule is the one that reports the missing half.
//!
//! An edge whose far end is not a document passes: `EdgeUnit::Pair` never
//! groups one, so this is unreachable rather than tolerated, on
//! [`crate::basis`]'s own precedent.
//!
//! An edge whose criterion end carries no digest passes: the census read no
//! bytes there, which is a different report with a different owner.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::observation::{Observation, Observations, Recorded};
use crate::scope::{EdgeCheck, EdgeUnit, EdgeView};
use headwater_census::census::Census;
use headwater_graph::declarations::Relation;
use headwater_graph::Declarations;
use headwater_graph::{Direction, Graph, Target};
use std::collections::{BTreeMap, BTreeSet};

pub const RULE: &str = "relation.target.verification.suspect";

/// The kind a relation must reach for this rule to instantiate over it.
const TARGET_KIND: &str = "verification";

/// A group with no half at all, which the instantiation never produces. As
/// [`crate::basis`]: recorded rather than panicked on.
const NO_HALF: &str = "the entry carries no declared half";

/// The far end is not a document. `EdgeUnit::Pair` never groups one, so this
/// is unreachable rather than tolerated.
const NO_PAIR: &str = "this edge has no second document to read";

/// The check, generated from every relation that reaches a `verification`.
pub struct Verified<'a> {
    /// Every relation whose `to` names [`TARGET_KIND`]. Empty for a taxonomy
    /// that declares no such relation, and then this rule generates no
    /// instance at all.
    declared: Vec<&'a Relation>,
    /// The committed observation snapshot, read once for the whole run on the
    /// same terms [`crate::register::Projection::of`] reads it.
    observations: &'a Observations,
}

impl<'a> Verified<'a> {
    pub fn over(declarations: &'a Declarations, observations: &'a Observations) -> Self {
        Verified {
            declared: declarations
                .relations
                .iter()
                .filter(|relation| relation.to.iter().any(|kind| kind == TARGET_KIND))
                .collect(),
            observations,
        }
    }
}

impl EdgeCheck for Verified<'_> {
    const RULE: &'static str = self::RULE;
    /// See [`crate::placement::Placement::VERSION`].
    const VERSION: u32 = 2;
    /// Both ends have to be documents: the rule reads a content digest at
    /// each of them.
    const UNIT: EdgeUnit = EdgeUnit::Pair;
    /// This rule reads [`Observations`] directly in [`Verified::evaluate`],
    /// so its cache key has to carry the snapshot's digest or an edit to
    /// `.headwater/observations.yml` with no other document moving would
    /// never be seen again after the first cache write. See
    /// [`crate::scope::EdgeCheck::NEEDS_OBSERVATIONS`].
    const NEEDS_OBSERVATIONS: bool = true;

    /// A relation reaching [`TARGET_KIND`], and no other.
    fn instantiates(&self, relation: &str) -> bool {
        self.declared.iter().any(|known| known.name == relation)
    }

    fn evaluate(&self, view: &EdgeView<'_>) -> Outcome {
        let Some(half) = view.declared_half().or_else(|| view.inverse_half()) else {
            return Outcome::Skipped(NO_HALF.to_string());
        };
        let Some((criterion, verification)) = view.ends() else {
            return Outcome::Skipped(NO_PAIR.to_string());
        };

        let Some(current_digest) = criterion.digest() else {
            return Outcome::Skipped(
                "the census read no digest for the document at the criterion end of this edge, \
                 so there is nothing to compare a snapshot against"
                    .to_string(),
            );
        };

        // `declared` and `observed at <commit>` are passes, and the module
        // comment says why. [`compare`] is the one comparison, and
        // [`Block`] calls it too.
        let commit = match compare(self.observations, verification.id, current_digest) {
            Freshness::Declared | Freshness::Observed { .. } => return Outcome::Passed,
            // Not a pass: a snapshot this run could not read says nothing
            // about whether the criterion moved. `control.observation.invalid`
            // reports the file or the entry, and this instance decides nothing.
            Freshness::Unknown { reason } => {
                return Outcome::Skipped(format!(
                    "the observation snapshot did not read for this verification: {reason}"
                ))
            }
            Freshness::Suspect { commit } => commit,
        };

        // `suspect`: the criterion changed after the snapshot. See the module
        // comment for why a content digest answers the same question a
        // commit walk would.
        let (line, column) = at(Some(half.span));
        Outcome::failed_with(Finding {
            rule: self::RULE,
            severity: Severity::Warn,
            obligation: None,
            path: half.source.path.clone(),
            line,
            column,
            message: message(criterion.id, verification.id, commit),
            remediation: remediation(verification.id),
            // No fix. Whether the change to the criterion still leaves the
            // verification settling it is a person's judgment, the same
            // reason `basis.rs` and `suspect.rs` both carry no patch.
            patch: None,
        })
    }
}

/// One verification's state against one criterion that reaches it, on
/// [HW-DR-0073](../../../../docs/decisions/0073-a-verification-is-a-kind-and-its-identity-is-minted-rather-than-found-in-the-code-that-cites-it.md)
/// ruling 3's three names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Freshness<'a> {
    /// No committed snapshot names the verification.
    Declared,
    /// A snapshot names it, and its recorded digest of the criterion matches
    /// the criterion's bytes today.
    Observed { commit: &'a str },
    /// A snapshot names it, and the criterion changed after the snapshot.
    Suspect { commit: &'a str },
    /// The snapshot, or its entry for this verification, did not read. A run
    /// that could not read it does not know whether an entry names the
    /// verification, so this is not `declared`.
    Unknown { reason: &'a str },
}

/// The one comparison. [`Verified::evaluate`] calls it for each edge, and
/// [`Block::of`] calls it for each criterion that reaches a verification, so
/// the finding and the report line read one answer.
pub fn compare<'a>(
    observations: &'a Observations,
    verification: &str,
    current_digest: &str,
) -> Freshness<'a> {
    match observations.recorded(verification) {
        Recorded::Absent => Freshness::Declared,
        Recorded::Unread(reason) => Freshness::Unknown { reason },
        Recorded::Entry {
            commit,
            criterion_digest,
        } if criterion_digest == current_digest => Freshness::Observed { commit },
        Recorded::Entry { commit, .. } => Freshness::Suspect { commit },
    }
}

/// One line of the verification block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub verification: String,
    pub state: State,
}

/// The state one line prints. It owns its strings because a [`crate::Run`]
/// outlives the snapshot it read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    Declared,
    Observed {
        commit: String,
    },
    /// `criteria` names every criterion whose digest moved, sorted.
    Suspect {
        commit: String,
        criteria: Vec<String>,
    },
    /// The snapshot or its entry did not read, or a criterion has no bytes
    /// to compare. Not `declared`: see [`Freshness::Unknown`].
    Unknown {
        reason: String,
    },
}

/// The verification block of the report: every verification document of the
/// corpus, sorted by identifier, each with one state. See the module comment
/// for why the two passing states are named here and are not findings.
///
/// A projection of the census, the graph and the snapshot, which no cache
/// stores, the same terms as [`crate::register::Projection`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Block {
    pub lines: Vec<Line>,
    /// The digest of `.headwater/observations.yml` as this run read it, and
    /// nothing where the run read no bytes there. The header prints it,
    /// because it is what every `observed` and `suspect` state is
    /// transcribed from.
    pub snapshot: Option<String>,
    /// Every verification entry of the snapshot that names no verification
    /// of the corpus, sorted.
    pub orphans: Vec<String>,
    /// Whether the whole snapshot file did not read.
    pub unread: bool,
}

impl Block {
    /// Build the block over the relations that reach a `verification`, the
    /// same declaration-driven denominator [`Verified`] reads.
    pub fn of(
        declarations: &Declarations,
        census: &Census,
        graph: &Graph,
        observations: &Observations,
    ) -> Self {
        let reaching: BTreeSet<&str> = declarations
            .relations
            .iter()
            .filter(|relation| relation.to.iter().any(|kind| kind == TARGET_KIND))
            .map(|relation| relation.name.as_str())
            .collect();

        // Each verification with the criteria that reach it. Both halves of
        // one edge may be written, so a set holds each pair once.
        let mut reached: BTreeMap<&str, BTreeSet<(&str, &str)>> = graph
            .index
            .typed
            .iter()
            .filter(|node| node.kind.as_deref() == Some(TARGET_KIND))
            .map(|node| (node.id.as_str(), BTreeSet::new()))
            .collect();
        for edge in &graph.edges {
            if !reaching.contains(edge.declared.as_str()) {
                continue;
            }
            let Target::Document { id, path, .. } = &edge.target else {
                continue;
            };
            let (criterion, verification) = match edge.direction {
                Direction::AsDeclared => ((edge.source.id.as_str(), edge.source.path.as_str()), id),
                Direction::Inverse => ((id.as_str(), path.as_str()), &edge.source.id),
            };
            if let Some(criteria) = reached.get_mut(verification.as_str()) {
                criteria.insert(criterion);
            }
        }

        let unread = observations.unread().is_some();
        let lines: Vec<Line> = reached
            .into_iter()
            .map(|(verification, criteria)| {
                let state = match observations.recorded(verification) {
                    Recorded::Absent => State::Declared,
                    // The file did not read: the header says so once, and the
                    // finding of `control.observation.invalid` says why.
                    Recorded::Unread(_) if unread => State::Unknown {
                        reason: "the snapshot did not read".to_string(),
                    },
                    Recorded::Unread(reason) => State::Unknown {
                        reason: format!("its snapshot entry did not read: {reason}"),
                    },
                    Recorded::Entry { commit, .. } => {
                        let mut moved = Vec::new();
                        let mut unknown = None;
                        for (criterion, path) in criteria {
                            // The rule skips the same edge: no bytes, nothing
                            // to compare.
                            let Some(digest) = digest_at(census, path) else {
                                unknown.get_or_insert_with(|| {
                                    format!("the census read no bytes of {criterion}")
                                });
                                continue;
                            };
                            if let Freshness::Suspect { .. } =
                                compare(observations, verification, digest)
                            {
                                moved.push(criterion.to_string());
                            }
                        }
                        // An entry that no criterion reaches is observed:
                        // nothing it proves has moved.
                        match (moved.is_empty(), unknown) {
                            (false, _) => State::Suspect {
                                commit: commit.to_string(),
                                criteria: moved,
                            },
                            (true, Some(reason)) => State::Unknown { reason },
                            (true, None) => State::Observed {
                                commit: commit.to_string(),
                            },
                        }
                    }
                };
                Line {
                    verification: verification.to_string(),
                    state,
                }
            })
            .collect();

        // A verification entry that names no verification of the corpus. No
        // line above can carry it, and dropping it would hide a typo in the
        // snapshot.
        let orphans = observations
            .entries()
            .iter()
            .filter_map(|entry| match entry {
                Observation::Verification { verification, .. }
                    if lines
                        .binary_search_by(|line| line.verification.as_str().cmp(verification))
                        .is_err() =>
                {
                    Some(verification.clone())
                }
                _ => None,
            })
            .collect::<BTreeSet<String>>()
            .into_iter()
            .collect();

        Block {
            lines,
            orphans,
            snapshot: observations.read_set_digest().flatten().map(str::to_string),
            unread,
        }
    }

    /// How many lines carry each state: declared, observed, suspect, unknown.
    pub fn counts(&self) -> (usize, usize, usize, usize) {
        self.lines
            .iter()
            .fold((0, 0, 0, 0), |(d, o, s, u), line| match line.state {
                State::Declared => (d + 1, o, s, u),
                State::Observed { .. } => (d, o + 1, s, u),
                State::Suspect { .. } => (d, o, s + 1, u),
                State::Unknown { .. } => (d, o, s, u + 1),
            })
    }

    /// The block as text, and nothing for a corpus with no verification.
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        if self.lines.is_empty() && self.orphans.is_empty() {
            return out;
        }
        let (declared, observed, suspect, unknown) = self.counts();
        out.push_str("verifications\n");
        let _ = writeln!(
            out,
            "  {} verifications: {declared} declared, {observed} observed, {suspect} suspect, \
             {unknown} unknown",
            self.lines.len()
        );
        if self.unread {
            // No state below is transcribed from anything: the file did not
            // read, and `control.observation.invalid` says why.
            let _ = writeln!(
                out,
                "  {} did not read, so no verification has a known state",
                crate::observation::PATH
            );
        } else {
            // The source of every `observed` and `suspect` state, stated
            // once. The state is transcribed from the snapshot, and the
            // snapshot's digest pins it. HW-OBL-0070 records why this line is
            // not an instance of the `transcribed` warrant.
            let _ = writeln!(
                out,
                "  each observed or suspect state is transcribed from {}{}",
                crate::observation::PATH,
                match &self.snapshot {
                    Some(digest) => format!(" at {digest}"),
                    None => String::new(),
                }
            );
        }
        for line in &self.lines {
            let _ = match &line.state {
                State::Declared => writeln!(out, "  {} declared", line.verification),
                State::Observed { commit } => {
                    writeln!(out, "  {} observed at {commit}", line.verification)
                }
                State::Suspect { commit, criteria } => writeln!(
                    out,
                    "  {} suspect since {commit}, {} changed",
                    line.verification,
                    criteria.join(", ")
                ),
                State::Unknown { reason } => {
                    writeln!(out, "  {} unknown, {reason}", line.verification)
                }
            };
        }
        for orphan in &self.orphans {
            let _ = writeln!(
                out,
                "  {orphan} is named in the snapshot and is no verification of this corpus"
            );
        }
        out
    }
}

/// The census's digest of the bytes at one path, which is the same digest
/// [`crate::scope::EdgeEnd::digest`] hands the rule.
fn digest_at<'a>(census: &'a Census, path: &str) -> Option<&'a str> {
    census
        .rows
        .binary_search_by(|row| row.path.as_str().cmp(path))
        .ok()
        .and_then(|index| census.rows[index].digest.as_deref())
}

/// What moved, in the terms of the criterion an author is looking at.
fn message(criterion: &str, verification: &str, commit: &str) -> String {
    format!(
        "`{criterion}` names `proven_by: {verification}`, which a snapshot observed at commit \
         `{commit}`; `{criterion}` has changed since that snapshot was written, so \
         `{verification}` is suspect until a new snapshot observes it against the criterion as \
         it now reads"
    )
}

/// What to do about it, which is one sentence because there is one remedy.
fn remediation(verification: &str) -> String {
    format!(
        "re-run the verification against the criterion as it now reads and commit a fresh entry \
         for `{verification}` in `.headwater/observations.yml`, or confirm the change to the \
         criterion did not affect what `{verification}` proves and record a new snapshot anyway"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_message_names_the_criterion_the_verification_and_the_commit() {
        let text = message("FIX-AC-1", "FIX-VER-1", "788885a9");
        assert!(text.contains("FIX-AC-1"), "{text}");
        assert!(text.contains("proven_by: FIX-VER-1"), "{text}");
        assert!(text.contains("commit `788885a9`"), "{text}");
        assert!(text.contains("suspect"), "{text}");
    }

    #[test]
    fn the_remediation_names_the_verification_and_the_snapshot_file() {
        let text = remediation("FIX-VER-1");
        assert!(text.contains("FIX-VER-1"), "{text}");
        assert!(text.contains(".headwater/observations.yml"), "{text}");
        // And it never tells the reader to reach for a version-control
        // command: the module comment states why this rule stays on content.
        assert!(!text.contains("git "), "{text}");
    }
}
