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
use headwater_check::{Coverage, Run, Scoped};
use headwater_yaml::json::Json;

/// The version of this document's own shape.
///
/// A consumer outside the repository reads these bytes and holds no clone, so
/// the shape has to name itself. It is not the engine's version: two engines
/// that write one shape should not make a reader re-read it.
///
/// `1.1` added [`change`], and the bump is what makes its absence a statement.
/// The member is written for a change-scoped run and for no other, so a reader
/// of a `1.1` document that carries none knows the run read the whole corpus. A
/// reader of a `1.0` document knows only that this producer had no such member
/// to write, and those two are exactly the pair the member exists to separate.
///
/// `1.2` added the skip accounting to [`coverage`], and the bump does the same
/// work for a value that is present on every run rather than absent on most.
/// `"skipped": 0` at `1.2` says every instance of the run reached a verdict. The
/// same document at `1.1` says only that this producer had no member for the
/// ones that did not, and a consumer that read the four counts alone read
/// `3498 instances` off a run where 585 of them decided nothing.
///
/// Two of the shapes here have a second reader: [`change`] and [`coverage`] are
/// what the SARIF property bag carries, so this constant versions them for that
/// artifact too and [`crate::sarif`] writes it there.
pub const VERSION: &str = "1.2";

/// One run as JSON.
pub fn render(run: &Run, subject: &Subject<'_>) -> String {
    document(run, subject).render_pretty()
}

/// The change a run was scoped to, in the shape this document and the SARIF
/// property bag both write.
///
/// One shape with two readers rather than two shapes that agree until somebody
/// edits one of them. [`crate::sarif`] calls this: a consumer that holds both
/// artifacts of one run should not have to learn the same six values twice.
///
/// The `Option` is the caller's and never this function's. Nothing reaches here
/// except a run that was scoped, so a full-corpus run writes no member at all,
/// which is the statement [`Scoped`]'s own declaration makes.
pub fn change(scoped: &Scoped) -> Json {
    Json::object([
        ("documents", number(scoped.named.documents)),
        ("added", number(scoped.named.added)),
        ("carried", number(scoped.named.carried)),
        ("unreadable", number(scoped.named.unreadable)),
        // The paths, and not the count that `Named` holds beside them. A caller
        // who mistyped one character needs the path, and a count sends them to
        // read their own manifest against a census by hand. The length is the
        // count, so the two cannot disagree here.
        (
            "unmatched",
            Json::Array(
                scoped
                    .unmatched
                    .iter()
                    .map(|path| Json::string(path.clone()))
                    .collect(),
            ),
        ),
        ("promotions", number(scoped.promotions)),
    ])
}

/// What a run looked at, in the shape this document and the SARIF property bag
/// both write.
///
/// One shape with two readers, for the reason [`change`] is one. The four counts
/// were the whole of it until #233: a consumer read `3498 instances` and could
/// not see that 585 of them reached no verdict, which crosses spec 4's rule that
/// a skip is visible rather than silent.
///
/// **Every member is written on every run, including a zero and an empty list.**
/// The absence of a member is [`change`]'s statement and it cannot be this one's:
/// coverage stands on every run, so a reader that met no `skipped` member could
/// not tell a run that skipped nothing from a producer that does not report
/// skips. [`VERSION`] is what separates those two, and the value separates
/// nothing on its own.
///
/// The one value of [`Coverage`] that no member here carries is which document
/// each skipped instance fell on. No format of this engine carries it and no
/// flag of `headwater check` prints it, so it is not a loss of this target
/// against another: `Detail::EveryInstance` is the one renderer that holds it and
/// nothing wires it to a surface.
pub fn coverage(coverage: &Coverage) -> Json {
    Json::object([
        ("seen", number(coverage.seen())),
        ("classified", number(coverage.classified())),
        ("checked", number(coverage.checked())),
        ("generated", number(coverage.generated())),
        ("instances", number(coverage.instances)),
        // The count that is comparable with `instances` above: an instance that
        // reached no verdict, counted once whatever it was routed to. See
        // `Coverage::skips` for the two ways a count read off the documents is
        // not that number.
        ("skipped", number(coverage.skipped())),
        (
            "skips",
            Json::Array(
                coverage
                    .skips()
                    .iter()
                    .map(|(reason, instances)| {
                        Json::object([
                            ("reason", Json::string(reason.clone())),
                            ("instances", number(*instances)),
                        ])
                    })
                    .collect(),
            ),
        ),
        // The paths, and not the count of them, for the reason `change` names
        // its unmatched paths: a check that read outside the census read
        // outside the set every guarantee here is computed over, and the reader
        // who can act on that needs the path.
        (
            "unaccounted",
            Json::Array(
                coverage
                    .unaccounted
                    .iter()
                    .map(|path| Json::string(path.clone()))
                    .collect(),
            ),
        ),
    ])
}

fn document(run: &Run, subject: &Subject<'_>) -> Json {
    let mut members: Vec<(&'static str, Json)> = vec![
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
    ];
    // Above coverage, for the reason the text report puts it above coverage: it
    // is the input that decides which instances reached a verdict at all, and
    // the counts below count the ones that did not.
    if let Some(scoped) = &run.change {
        members.push(("change", change(scoped)));
    }
    members.extend([
        ("coverage", coverage(&run.coverage)),
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
    ]);
    Json::object(members)
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
