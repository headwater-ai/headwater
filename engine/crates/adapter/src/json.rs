// SPDX-License-Identifier: Apache-2.0
//! The run in the finding shape spec 4 already declares.
//!
//! [Spec 4](../../../../docs/spec/04-assurance-model.md#findings) fixes it:
//! "Every finding, from every mechanism, has one shape", and the worked
//! document there carries `rule`, `severity`, `obligation`, `path`, `line`,
//! `message`, `remediation` and `fixable`. Those eight members are the members
//! below, in that order, plus `column`, which the engine keeps because two
//! findings on one line are otherwise ordered by their message.
//!
//! # This is not an adapter, and the distinction is the point of the crate
//!
//! No platform reads this. It is the neutral surface that
//! [spec 8](../../../../docs/spec/08-design-departures.md) leaves for an
//! adapter that this repository did not write: a forge, a tracker or a cloud
//! that nobody here privileged, whose author reads bytes rather than links a
//! Rust library. The two adapters beside it — [`crate::sarif`] and
//! [`crate::markdown`] — read the same run through the same crate, and neither
//! reads this.
//!
//! # An empty loss set is a claim, and it is audited
//!
//! [`crate::Format::Json`] declares no loss. Every finding of the run is here,
//! live, held and hidden alike, with the escape that holds each one beside it,
//! and every field of every finding is a member. [`crate::census`] is what
//! holds the claim: a finding the output does not name is a defect in this
//! module, not a fact about the corpus.

use crate::{reported, Reported, Subject};
use headwater_check::register::Bound;
use headwater_check::Run;
use headwater_yaml::json::Json;

/// The version of this document's own shape.
///
/// A consumer outside the repository reads these bytes and holds no clone, so
/// the shape has to name itself. It is not the engine's version: two engines
/// that write one shape should not make a reader re-read it.
pub const VERSION: &str = "1.0";

/// One run as JSON.
pub fn render(run: &Run, subject: &Subject<'_>) -> String {
    document(run, subject).render_pretty()
}

fn document(run: &Run, subject: &Subject<'_>) -> Json {
    Json::object([
        ("version", Json::string(VERSION)),
        ("tool", Json::string(crate::TOOL)),
        (
            "taxonomy",
            Json::object([
                ("package", Json::string(subject.package)),
                ("version", Json::string(subject.version)),
                ("lock", Json::string(subject.lock)),
            ]),
        ),
        ("clock", Json::string(subject.now)),
        (
            "coverage",
            Json::object([
                ("seen", number(run.coverage.seen())),
                ("classified", number(run.coverage.classified())),
                ("checked", number(run.coverage.checked())),
                ("instances", number(run.coverage.instances)),
            ]),
        ),
        ("rules", Json::Array(run.served.iter().map(rule).collect())),
        (
            "findings",
            Json::Array(reported(run).iter().map(finding).collect()),
        ),
        // The read set, in full, with a digest per input. Spec 6 asks a run to
        // report it beside the findings, and the reader that needs it is a gate
        // holding this verdict against a later tree.
        (
            "read_set",
            Json::Array(
                run.read_set
                    .inputs
                    .iter()
                    .map(|input| {
                        Json::object([
                            ("path", Json::string(input.path.clone())),
                            (
                                "digest",
                                match &input.digest {
                                    Some(digest) => Json::string(digest.clone()),
                                    // Absent rather than empty. A gate can
                                    // decide nothing about an input whose
                                    // content nothing identified, and an empty
                                    // string is a value it could compare.
                                    None => Json::Array(Vec::new()),
                                },
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

fn rule(served: &headwater_check::Serves) -> Json {
    Json::object([
        ("rule", Json::string(served.rule)),
        ("scope", Json::string(served.scope.grain().name())),
        ("version", number(served.version as usize)),
        (
            "obligations",
            match &served.obligation {
                Bound::To(one) => Json::Array(vec![Json::string(one.clone())]),
                Bound::Several(many) => {
                    Json::Array(many.iter().map(|one| Json::string(one.clone())).collect())
                }
                Bound::Unnamed => Json::Array(Vec::new()),
            },
        ),
    ])
}

fn finding(entry: &Reported<'_>) -> Json {
    let finding = entry.finding;
    let mut members: Vec<(&'static str, Json)> = vec![
        ("rule", Json::string(finding.rule)),
        ("severity", Json::string(crate::severity(finding.severity))),
    ];
    if let Some(obligation) = &finding.obligation {
        members.push(("obligation", Json::string(obligation.clone())));
    }
    members.push(("path", Json::string(finding.path.clone())));
    members.push(("line", number(finding.line)));
    members.push(("column", number(finding.column)));
    members.push(("message", Json::string(finding.message.clone())));
    members.push(("remediation", Json::string(finding.remediation.clone())));
    members.push(("fixable", Json::Bool(finding.fixable())));
    // What became of it. A consumer that reads this member alone can partition
    // the set three ways, which is the question a CI surface exists to answer.
    members.push((
        "escape",
        match entry.escape {
            Some(escape) => Json::string(escape.name()),
            None => Json::string("none"),
        },
    ));
    if let Some(task) = entry.task {
        members.push((
            "held_by",
            Json::object([
                ("task", Json::string(task.id.clone())),
                ("owner", Json::string(task.owner.clone())),
                ("until", Json::string(task.until.render())),
            ]),
        ));
    }
    if let Some(directive) = entry.directive {
        members.push((
            "hidden_by",
            Json::object([
                ("path", Json::string(directive.path.clone())),
                ("line", number(directive.line)),
                ("reason", Json::string(directive.reason.name())),
                ("until", Json::string(directive.until.render())),
                ("note", Json::string(directive.note.clone())),
            ]),
        ));
    }
    Json::object(members)
}

fn number(value: usize) -> Json {
    Json::Raw(value.to_string())
}
