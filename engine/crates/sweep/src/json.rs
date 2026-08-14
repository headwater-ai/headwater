// SPDX-License-Identifier: Apache-2.0
//! One sweep in the finding shape spec 4 declares, with the two members a
//! sweep adds.
//!
//! [Spec 4](../../../../docs/spec/04-assurance-model.md#findings): "Every
//! finding, from every mechanism, has one shape." This is that shape, written
//! by the mechanism that is not a check. `headwater_adapter::json` writes the
//! same members for a run of the checks, and the two are deliberately separate
//! functions over separate types: a `Run` cannot hold a sweep finding, because
//! `headwater-check` cannot name this crate.
//!
//! # The two members a sweep adds, and why they are not on the shared shape
//!
//! `provenance` is `agent` on every finding here and on none there, so it is a
//! constant of the mechanism rather than a field of a finding. `evidence` is
//! the part spec 4's third constraint requires of a sweep and of nothing else.
//! A consumer that merges the two documents reads both members, which is why
//! they are written per finding rather than once at the top.
//!
//! # What a consumer may not do with this
//!
//! Count the findings and fail a build on the number. The set is a sample: a
//! second run of the same sweep returns a different one, and an empty document
//! is what a model that was never asked produces. The `sample` member says so
//! in the artifact, so a consumer that reads only these bytes still meets the
//! warning.

use crate::intake::Report;
use crate::PROVENANCE;
use headwater_yaml::json::Json;

/// The version of this document's own shape.
pub const VERSION: &str = "1.0";

pub fn render(report: &Report) -> String {
    document(report).render_pretty()
}

fn document(report: &Report) -> Json {
    let mut members = vec![
        ("version", Json::string(VERSION)),
        ("tool", Json::string("headwater")),
        ("mechanism", Json::string("sweep")),
        ("provenance", Json::string(PROVENANCE)),
        (
            "sample",
            Json::string(
                "a sample of one agent's reading, not a run over a rule set. Absence is not \
                 evidence, and no exit status reads this document.",
            ),
        ),
        (
            "slice",
            Json::object([
                ("under", Json::string(&report.slice)),
                ("documents", number(report.extent)),
                ("corpus", number(report.corpus)),
            ]),
        ),
        (
            "counts",
            Json::object([
                ("read", number(report.read)),
                ("carried", number(report.verified.len())),
                ("refused", number(report.rejected.len())),
            ]),
        ),
    ];
    if let Some(refusal) = &report.refusal {
        members.push(("refusal", Json::string(refusal.to_string())));
    }
    members.push((
        "findings",
        Json::Array(report.verified.iter().map(finding).collect()),
    ));
    members.push((
        "refused",
        Json::Array(
            report
                .rejected
                .iter()
                .map(|rejected| {
                    Json::object([
                        ("at", number(rejected.at)),
                        ("line", number(rejected.line)),
                        ("reason", Json::string(rejected.reason.to_string())),
                    ])
                })
                .collect(),
        ),
    ));
    Json::Object(
        members
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}

fn finding(verified: &crate::intake::Verified) -> Json {
    let found = &verified.finding;
    let mut members = vec![
        ("rule", Json::string(found.rule)),
        ("class", Json::string(verified.class.name())),
        ("provenance", Json::string(PROVENANCE)),
        ("severity", Json::string(found.severity.to_string())),
        // Null rather than absent: a consumer that reads a run of the checks
        // beside this one meets the member either way, and `None` there means
        // the same thing it means here.
        ("obligation", Json::Raw("null".into())),
        ("path", Json::string(&found.path)),
        ("line", number(found.line)),
        ("column", number(found.column)),
        ("message", Json::string(&found.message)),
        ("remediation", Json::string(&found.remediation)),
        ("fixable", Json::Bool(found.fixable())),
        (
            "documents",
            Json::Array(verified.documents.iter().map(Json::string).collect()),
        ),
        (
            "evidence",
            Json::Array(
                verified
                    .evidence
                    .iter()
                    .map(|evidence| {
                        Json::object([
                            ("path", Json::string(&evidence.path)),
                            ("line", number(evidence.line)),
                            ("quote", Json::string(&evidence.quote)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ];
    if let Some(proposal) = &verified.proposal {
        members.push((
            "proposal",
            Json::object([
                ("relation", Json::string(&proposal.relation)),
                ("from", Json::string(&proposal.from)),
                ("to", Json::string(&proposal.to)),
                ("declared", Json::Bool(false)),
            ]),
        ));
    }
    Json::Object(
        members
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}

fn number(value: usize) -> Json {
    Json::Raw(value.to_string())
}
