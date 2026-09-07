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
use headwater_check::{Coverage, Instance, Outcome, Run, Scoped};
use headwater_yaml::json::Json;
use std::collections::HashSet;

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
/// `1.3` added the routing of each skip: every entry of `coverage.skips` now
/// carries `documents`, the census rows that class of skip was routed to with
/// the rules that skipped over each, and `unrouted`, the instances of that class
/// that were routed to no document at all. The bump separates the same pair the
/// two above separate. A `1.3` document that writes `"documents": []` and
/// `"unrouted": 4` says every instance of that class fell outside the census
/// rows, and the same class in a `1.2` document says only that this producer had
/// no member for where a skip fell. Before `1.3` two runs of one corpus that
/// skipped the same number of instances under one class over **different**
/// documents wrote one artifact, because the only member that moved was a
/// content digest of the read set.
///
/// Two of the shapes here have a second reader: [`change`] and [`coverage`] are
/// what the SARIF property bag carries, so this constant versions them for that
/// artifact too and [`crate::sarif`] writes it there.
pub const VERSION: &str = "1.3";

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
/// Which document each skipped instance fell on is a member here since `1.3`,
/// inside the `skips` entry for its class. It was the one value of [`Coverage`]
/// that reached no format at all, which put a limit on what this block can
/// distinguish: two runs of one corpus that skipped the same number of instances
/// under one class, over different documents, wrote the same coverage block and
/// differed only in a content digest of the read set. A digest moves on any edit
/// and says nothing about what went unmeasured, so the one difference a consumer
/// could see was the one that misled them.
pub fn coverage(coverage: &Coverage, instances: &[Instance]) -> Json {
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
        ("skips", skips(coverage, instances)),
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

/// Each class of skip, with the instances it covers and where they fell.
///
/// Two arms, and the second is the one a document-keyed member would leave out.
///
/// `documents` is the routing, read off [`Coverage::documents`] in census order.
/// One entry per census row that class of skip was routed to, with the rules
/// that skipped over that row and the number of routed instances. It is a count
/// of **routings** and not of instances: an edge-scoped instance is routed to
/// both of its endpoints, so it is one instance and two entries here, which is
/// why the entries do not sum to `instances` above and are not meant to.
///
/// `unrouted` is the other arm. A corpus-scoped instance is routed to no
/// document at all — `lifecycle.deletion.not_permitted` is the live one — and an
/// instance that read only paths the census never walked is routed to none
/// either. Neither reaches `documents`, so a reader of that member alone cannot
/// tell "this class fell on no document" from "this producer does not report
/// where a skip fell". This member is what separates them, and it is written on
/// every run including a zero for the same reason every other member here is.
///
/// The two arms partition the instances of the class: an instance is routed to
/// at least one census row or it is counted here, never both and never neither.
fn skips(coverage: &Coverage, instances: &[Instance]) -> Json {
    let walked: HashSet<&str> = coverage
        .documents
        .iter()
        .map(|document| document.path.as_str())
        .collect();
    // One pass over the instance record rather than one per class, and over the
    // record rather than over the documents: the documents hold the routing,
    // and this is the count of what the routing never reached.
    let mut unrouted: Vec<usize> = vec![0; coverage.skips().len()];
    for instance in instances {
        let Outcome::Skipped(reason) = &instance.outcome else {
            continue;
        };
        if instance.grain.routes() && instance.paths().iter().any(|path| walked.contains(path)) {
            continue;
        }
        if let Some(at) = coverage
            .skips()
            .iter()
            .position(|(known, _)| known == reason)
        {
            unrouted[at] += 1;
        }
    }
    Json::Array(
        coverage
            .skips()
            .iter()
            .zip(unrouted)
            .map(|((reason, instances), unrouted)| {
                Json::object([
                    ("reason", Json::string(reason.clone())),
                    ("instances", number(*instances)),
                    ("documents", routed(coverage, reason)),
                    ("unrouted", number(unrouted)),
                ])
            })
            .collect(),
    )
}

/// The census rows one class of skip was routed to, in census order.
///
/// A row that class never fell on writes no entry, so the member is the routing
/// and not a second copy of the census. The rules are in the order they first
/// skipped over the row and each is named once, because a rule that skipped four
/// instances over one document is one fact for a reader and four lines for
/// nobody.
fn routed(coverage: &Coverage, reason: &str) -> Json {
    Json::Array(
        coverage
            .documents
            .iter()
            .filter_map(|document| {
                let mut rules: Vec<&'static str> = Vec::new();
                let mut instances = 0;
                for (rule, fell) in &document.skipped {
                    if fell != reason {
                        continue;
                    }
                    instances += 1;
                    if !rules.contains(rule) {
                        rules.push(rule);
                    }
                }
                (instances > 0).then(|| {
                    Json::object([
                        ("path", Json::string(document.path.clone())),
                        (
                            "rules",
                            Json::Array(rules.into_iter().map(Json::string).collect()),
                        ),
                        ("instances", number(instances)),
                    ])
                })
            })
            .collect(),
    )
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
        ("coverage", coverage(&run.coverage, &run.instances)),
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
