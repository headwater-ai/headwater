// SPDX-License-Identifier: Apache-2.0
//! The conformance report, in the shape a program reads.
//!
//! [#321](https://github.com/headwater-ai/headwater/issues/321) asks that
//! `conformance` "emit JSON that `python3 -m json.tool` parses". The reader
//! this document is written for is not the person closing a gap —
//! [`crate::render`] is written for them, with the package's own remediation
//! under each gap. It is the CI job or the adopter's dashboard that asks one
//! question about a tree and needs the answer as a value: which rung is
//! reached, which rules are not met, and which deviations are covered and until
//! when.
//!
//! # The two verdicts stay apart here, exactly as they do in the report
//!
//! `reached` reads no waiver: a waived rule is not a met rule, so a waiver never
//! moves a rung ([`crate::LevelState::reached`]). `gate` is the other question,
//! and a live waiver answers for the rule it covers
//! ([`crate::Reading::passes_gate`]). A document that reported one number would
//! let an adopter's dashboard show a rung that the package did not grant, which
//! is the confusion spec 7 keeps the two readings apart to prevent. Both are
//! members below and neither is derived from the other.
//!
//! # `gate` is written only where a caller asked for it
//!
//! `--level` is what asks. Without it there is no rung under question, and a
//! member reporting on one would be an answer to a question nobody put. That is
//! the same rule [`crate::Report::gate`] follows and the same rule the text
//! report follows, where the closing line is printed only under `--level`.
//!
//! # Absent members and exhaustive destructuring
//!
//! [`Json`] carries no `null`, so an `Option::None` writes no member at all.
//! Every function here destructures its source exhaustively, so a field added
//! to [`Report`], [`LevelState`], [`Reading`], [`Rule`] or [`Waiver`] and not to
//! the document below does not compile.

use crate::{Cover, DecidedBy, LevelState, Reading, Report, Rule, Verdict, Waiver};
use headwater_yaml::json::Json;

/// The version of the document this module writes.
///
/// The document's own shape and never the engine's, for the reason the finding
/// shape states: a consumer outside this repository holds no clone of the
/// engine, so the bytes have to name what they are.
pub const VERSION: &str = "1.0";

/// One report as JSON.
///
/// `gate` is `Some((level, passes))` exactly where the caller wrote `--level`,
/// and the caller computes it because the exit status rests on the same value.
/// One reading of [`Report::gate`] per run, and never two that could disagree.
pub fn report(report: &Report, gate: Option<(&str, bool)>) -> String {
    document(report, gate).render_pretty()
}

fn document(report: &Report, gate: Option<(&str, bool)>) -> Json {
    let Report {
        package,
        version,
        digest,
        now,
        readings,
        levels,
        reached,
    } = report;
    let mut identity: Vec<(&'static str, Json)> = vec![
        ("name", Json::string(package.clone())),
        ("version", Json::string(version.clone())),
    ];
    if let Some(digest) = digest {
        identity.push(("digest", Json::string(digest.clone())));
    }
    let mut members: Vec<(&'static str, Json)> = vec![
        ("version", Json::string(VERSION)),
        ("package", Json::object(identity)),
        ("clock", Json::string(now.render())),
    ];
    // Absent where the first rung is not reached, which is a state this
    // repository has been in. A consumer that met an empty string here could
    // not tell it from a rung somebody named that.
    if let Some(reached) = reached {
        members.push(("reached", Json::string(reached.clone())));
    }
    members.extend([
        ("levels", Json::Array(levels.iter().map(level).collect())),
        (
            "readings",
            Json::Array(readings.iter().map(reading).collect()),
        ),
    ]);
    if let Some((name, passes)) = gate {
        members.push((
            "gate",
            Json::object([
                ("level", Json::string(name)),
                ("passes", Json::Bool(passes)),
            ]),
        ));
    }
    Json::object(members)
}

fn level(level: &LevelState) -> Json {
    let LevelState {
        name,
        title,
        rules,
        reached,
        met,
        gaps,
        waived,
        undecided,
    } = level;
    Json::object([
        ("name", Json::string(name.clone())),
        ("title", Json::string(title.clone())),
        (
            "rules",
            Json::Array(
                rules
                    .iter()
                    .map(|rule| Json::string(rule.clone()))
                    .collect(),
            ),
        ),
        ("reached", Json::Bool(*reached)),
        ("met", number(*met)),
        ("gaps", number(*gaps)),
        ("waived", number(*waived)),
        ("undecided", number(*undecided)),
    ])
}

/// One rule, its verdict, and the waiver over it.
///
/// `passes_gate` is a member rather than something a consumer joins for itself,
/// because the join is the one place the two verdicts get confused: it is true
/// of a rule that is met and of a rule a live waiver covers, and false of an
/// expired one. Deriving it here is one reading of [`Reading::passes_gate`]
/// rather than a rule a consumer reimplements.
fn reading(reading: &Reading) -> Json {
    let Reading {
        rule: declared,
        verdict: found,
        cover: waiver,
    } = reading;
    Json::object([
        ("rule", rule(declared)),
        ("verdict", verdict(found)),
        ("cover", cover(waiver)),
        ("passes_gate", Json::Bool(reading.passes_gate())),
    ])
}

fn rule(rule: &Rule) -> Json {
    let Rule {
        name,
        title,
        decided_by,
        statement,
        remediation,
    } = rule;
    Json::object([
        ("name", Json::string(name.clone())),
        ("title", Json::string(title.clone())),
        (
            "decided_by",
            Json::string(match decided_by {
                DecidedBy::Tree => "tree",
                DecidedBy::Attestation => "attestation",
            }),
        ),
        ("statement", Json::string(statement.clone())),
        ("remediation", Json::string(remediation.clone())),
    ])
}

/// The verdict: a state, and what this engine found where the state has
/// something to say.
///
/// `says` is absent for `met`, because [`Verdict::Met`] carries nothing. It is
/// the finding for a gap and the reading that would decide it for an undecided
/// rule, which are two different sentences under one member because they are
/// separated by `state` beside it.
fn verdict(verdict: &Verdict) -> Json {
    let mut members: Vec<(&'static str, Json)> = vec![(
        "state",
        Json::string(match verdict {
            Verdict::Met => "met",
            Verdict::Gap(_) => "gap",
            Verdict::NotDecided(_) => "not_decided",
        }),
    )];
    if let Verdict::Gap(says) | Verdict::NotDecided(says) = verdict {
        members.push(("says", Json::string(says.clone())));
    }
    Json::object(members)
}

/// Whether a waiver stands over the rule at the date of the run.
///
/// `state` is written on every reading, including `none`, because a rule with
/// no waiver over it is a fact and not a missing value. An expired waiver is
/// reported with its own state rather than dropped: it covers nothing, and a
/// consumer that could not see it would read the gap as one nobody had ever
/// looked at.
fn cover(cover: &Cover) -> Json {
    let mut members: Vec<(&'static str, Json)> = vec![(
        "state",
        Json::string(match cover {
            Cover::None => "none",
            Cover::Live(_) => "live",
            Cover::Expired(_) => "expired",
        }),
    )];
    if let Cover::Live(declared) | Cover::Expired(declared) = cover {
        members.push(("waiver", waiver(declared)));
    }
    Json::object(members)
}

fn waiver(waiver: &Waiver) -> Json {
    let Waiver {
        rule,
        reason,
        owner,
        until,
        note,
    } = waiver;
    let mut members: Vec<(&'static str, Json)> = vec![
        ("rule", Json::string(rule.clone())),
        ("reason", Json::string(reason.name())),
        ("owner", Json::string(owner.clone())),
        ("until", Json::string(until.render())),
    ];
    if let Some(note) = note {
        members.push(("note", Json::string(note.clone())));
    }
    Json::object(members)
}

fn number(value: usize) -> Json {
    Json::Raw(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use headwater_check::Date;

    fn a_rule(name: &str) -> Rule {
        Rule {
            name: name.to_string(),
            title: "a rule".to_string(),
            decided_by: DecidedBy::Tree,
            statement: "what it asks".to_string(),
            remediation: "what to do".to_string(),
        }
    }

    fn a_waiver(until: Date) -> Waiver {
        Waiver {
            rule: "pin.current".to_string(),
            reason: crate::Reason::AcceptedDeviation,
            owner: "somebody".to_string(),
            until,
            note: None,
        }
    }

    fn a_report(readings: Vec<Reading>, reached: Option<&str>) -> Report {
        Report {
            package: "headwater/standard".to_string(),
            version: "1.0.0".to_string(),
            digest: None,
            now: Date::parse("2026-08-24").expect("a date"),
            readings,
            levels: vec![LevelState {
                name: "L1".to_string(),
                title: "the first rung".to_string(),
                rules: vec!["pin.current".to_string()],
                reached: reached.is_some(),
                met: 0,
                gaps: 1,
                waived: 1,
                undecided: 0,
            }],
            reached: reached.map(str::to_string),
        }
    }

    /// A rung a waiver covers is not a rung the package granted.
    ///
    /// The two readings sit in one document and this is the case that separates
    /// them: `reached` is absent, and the reading under it passes the gate.
    #[test]
    fn a_live_waiver_passes_the_gate_and_moves_no_rung() {
        let report = a_report(
            vec![Reading {
                rule: a_rule("pin.current"),
                verdict: Verdict::Gap("no release record".to_string()),
                cover: Cover::Live(a_waiver(Date::parse("2027-01-01").expect("a date"))),
            }],
            None,
        );
        let value = document(&report, Some(("L1", true)));
        assert!(
            !names(&value, "reached"),
            "no rung is reached, so no top-level member says one is: {}",
            value.render_pretty()
        );
        let document = value.render_pretty();
        assert!(
            document.contains("\"passes_gate\": true"),
            "and the live waiver answers for the rule: {document}"
        );
        assert!(
            document.contains("\"state\": \"live\""),
            "with the waiver reported: {document}"
        );
        assert!(
            document.contains("\"reached\": false"),
            "and the rung itself reports that it is not reached: {document}"
        );
    }

    /// Whether the top-level object carries a member.
    ///
    /// A substring test cannot answer this: `reached` is a member of the
    /// document *and* of every entry of `levels`, so a grep for the key finds
    /// the wrong one. This reads the value the writer built.
    fn names(value: &Json, key: &str) -> bool {
        let Json::Object(members) = value else {
            panic!("the conformance document is an object");
        };
        members.iter().any(|(name, _)| name == key)
    }

    /// An expired waiver is reported, and it covers nothing.
    #[test]
    fn an_expired_waiver_is_carried_rather_than_dropped() {
        let report = a_report(
            vec![Reading {
                rule: a_rule("pin.current"),
                verdict: Verdict::Gap("no release record".to_string()),
                cover: Cover::Expired(a_waiver(Date::parse("2020-01-01").expect("a date"))),
            }],
            None,
        );
        let value = document(&report, None);
        assert!(
            !names(&value, "gate"),
            "no `--level` was asked, so nothing is gated: {}",
            value.render_pretty()
        );
        let document = value.render_pretty();
        assert!(
            document.contains("\"state\": \"expired\""),
            "the expired waiver is named: {document}"
        );
        assert!(
            document.contains("\"passes_gate\": false"),
            "and it covers nothing: {document}"
        );
    }
}
