// SPDX-License-Identifier: Apache-2.0
//! SARIF 2.1.0: the run as a forge ingests it.
//!
//! Static Analysis Results Interchange Format, OASIS Standard, version 2.1.0
//! with errata 01. [Spec 6](../../../../docs/spec/06-engine-architecture.md#cli)
//! names it as a `check --format` target and
//! [the glossary](../../../../docs/spec/glossary.md) lists it among the
//! standards this project adopts rather than invents.
//!
//! # Three severity scales meet here, and only one of them reaches `level`
//!
//! [Spec 4](../../../../docs/spec/04-assurance-model.md#obligations-are-data):
//! "Two scales share one word." An **obligation** carries `high`, `medium` or
//! `low`, and it says how much the invariant matters. A **check** carries
//! `error`, `warn` or `info`, and a finding reports the one that the check that
//! produced it carries ([spec 12](../../../../docs/spec/12-check-layer.md#severity-is-the-checks-posture-is-the-controls)).
//! A **control** carries a posture, `advisory` or `blocking`, and it says
//! whether a finding stops a gate. SARIF adds a fourth: `level`, over `error`,
//! `warning`, `note` and `none`.
//!
//! `level` is written from the **check's** severity, and from nothing else.
//!
//! ```text
//! error -> error      warn -> warning      info -> note
//! ```
//!
//! The obligation's scale cannot be the source. It is a property of the
//! invariant rather than of the finding, so every finding of every rule that
//! serves one obligation would carry one level, and two findings of one check
//! would carry two. That is the control's judgment arriving inside the check's
//! field, which is the confusion spec 12 exists to prevent.
//!
//! The posture cannot be the source either, and for a stronger reason. Spec 6
//! rules that the engine "emits what it evaluated, and never orders what
//! lands". An artifact whose `level` said *this blocks* would be an order, and
//! it would also be wrong on the next run, because a promotion from advisory to
//! blocking is a configuration change that no artifact of the previous posture
//! knows about.
//!
//! `none` is unreachable. Headwater has three check severities and SARIF has
//! four levels, so the mapping is one-to-one into a larger set: nothing is lost
//! going out, and a SARIF document that arrives with `level: "none"` names a
//! state this engine cannot produce.
//!
//! # What this emitter declines to state, and why that is not modesty
//!
//! `tool.driver.rules[].defaultConfiguration.level` is where a SARIF producer
//! declares the level a rule fires at. This emitter writes no
//! `defaultConfiguration` at all, because the engine has no such value to
//! write. A generated check reads its severity out of the taxonomy
//! ([spec 13](../../../../docs/spec/13-open-obligations.md)), so one rule
//! spans as many severities as there are declarations that generate it, and an
//! adopter moves a finding from `warn` to `error` by editing data. A single
//! declared level per rule would be a claim about the next run rather than a
//! report of this one, and it would be the *declaration* speaking where the
//! checker is the thing under translation.
//!
//! Every `result` therefore carries its own `level`, read off the finding.
//!
//! # A suppressed finding is in the output, and it is marked
//!
//! This is the half a naive adapter drops. `headwater check` reports a live
//! finding, holds a `migration-pending` one under a task of the adoption
//! payload, and hides a suppressed one behind a directive
//! ([spec 4](../../../../docs/spec/04-assurance-model.md#suppression), in the
//! precedence waiver, then migration-pending, then suppression). All three are
//! in this artifact. The two escaped classes carry a SARIF `suppressions` array,
//! which is the member a consumer reads to show a result as dismissed rather
//! than as open, and the live ones carry none.
//!
//! `suppression.kind` has exactly two values, `inSource` and `external`.
//! Headwater has three escape classes. A directive is written in the document
//! it acts on, so it is `inSource`. A task of the adoption payload is written
//! in the lock, so it is `external`. A waiver, when a waiver mechanism exists,
//! is written by a publisher and is `external` too — so the two widest classes
//! will share one SARIF value, and `properties.headwater.escape` is what keeps
//! them apart. That is [`LOSS`]'s second entry, and it is a fact about the
//! vocabulary rather than about this emitter.
//!
//! # What no member of this vocabulary can hold
//!
//! [`LOSS`] is the declaration and [`crate::census`] is the audit of it, in the
//! shape spec 6 fixes for the graph emitters. A value that rides in a property
//! bag is still a loss, because a property bag is not a member of the
//! vocabulary and a consumer that reads SARIF alone does not find it.
//!
//! One entry is worth reading twice. Spec 6 asks a run to report "the corpus
//! tree, the taxonomy lock hash, and its read set". SARIF has
//! `run.automationDetails.id`, which is exactly where a corpus tree would go,
//! and this emitter leaves it out: nothing in the engine computes a corpus tree,
//! and the read set is not one, because the read set holds what the checks read
//! and a tree holds what the census walked. Writing the read set's identity into
//! a member that means *the tree this run is about* would be the emitter
//! printing something false to fill a slot.

use crate::{reported, Escape, Format, Loss, Reported, Subject, TOOL, TOOL_URI};
use headwater_check::instance::Input;
use headwater_check::register::Bound;
use headwater_check::{Run, Severity};
use headwater_yaml::json::Json;

/// The version of the format, which is a member of the document.
pub const VERSION: &str = "2.1.0";

/// The schema every artifact this module writes validates against.
///
/// The OASIS Standard copy, errata 01. The repository holds the same bytes at
/// `engine/crates/adapter/tests/sarif-schema-2.1.0.json`, and the test beside it
/// records the digest of both, so a reader can check that the vendored copy is
/// the published one without trusting this comment.
pub const SCHEMA: &str =
    "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json";

/// What SARIF cannot carry, and where each value went instead.
pub const LOSS: &[Loss] = &[
    Loss {
        field: "the obligation's severity",
        reason: "`level` is the check's scale, and SARIF has no second member for how much an \
                 invariant matters. A consumer that ranks by `level` ranks by the check",
        carried_in: "properties.headwater.obligation_severity",
    },
    Loss {
        field: "the escape class",
        reason: "`suppression.kind` has two values and spec 4 fixes three escape classes. A \
                 directive is `inSource`; a task of the adoption payload is `external`, and a \
                 waiver would be `external` too",
        carried_in: "properties.headwater.escape",
    },
    Loss {
        field: "the control's posture",
        reason: "no member says whether a finding stops a gate, and the engine emits what it \
                 evaluated and never orders what lands",
        carried_in: "",
    },
    Loss {
        field: "coverage",
        reason: "a `kind: \"pass\"` result would count check instances, and the census counts \
                 documents, including the ones no check classified",
        carried_in: "run.properties.headwater.coverage",
    },
    Loss {
        field: "the remediation of a mechanical fix",
        reason: "`fixes[]` carries replacement text and a finding carries prose with a \
                 `fixable` flag, so no result here carries a `fix`",
        carried_in: "properties.headwater.remediation and message.markdown",
    },
    Loss {
        field: "the corpus tree",
        reason: "`run.automationDetails.id` is where one would go and nothing computes one. The \
                 read set is not a tree: it holds what the checks read",
        carried_in: "",
    },
];

/// The check's severity as a SARIF `level`.
///
/// See the module comment for why this is the only scale that reaches the
/// field.
pub fn level(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warn => "warning",
        Severity::Info => "note",
    }
}

/// Where a suppression of this class is persisted, in SARIF's two-value
/// vocabulary.
pub fn kind(escape: Escape) -> &'static str {
    match escape {
        // The directive is a comment in the document it acts on.
        Escape::Suppression => "inSource",
        // The task is a block of the committed lock, and never of the document.
        Escape::MigrationPending => "external",
    }
}

/// One run as a SARIF 2.1.0 log.
pub fn render(run: &Run, subject: &Subject<'_>) -> String {
    document(run, subject).render_pretty()
}

fn document(run: &Run, subject: &Subject<'_>) -> Json {
    Json::object([
        ("$schema", Json::string(SCHEMA)),
        ("version", Json::string(VERSION)),
        ("runs", Json::Array(vec![one_run(run, subject)])),
    ])
}

fn one_run(run: &Run, subject: &Subject<'_>) -> Json {
    let rules: Vec<&'static str> = run.served.iter().map(|served| served.rule).collect();
    Json::object([
        ("tool", tool(run)),
        // Every column below counts characters rather than UTF-16 code units:
        // the loader converts the parser's character index once, at its own
        // boundary, and this states which of the two a consumer is reading.
        ("columnKind", Json::string("unicodeCodePoints")),
        (
            "invocations",
            Json::Array(vec![Json::object([(
                // The run produced a report. It is not a verdict about the
                // corpus and it is not a verdict about a merge: a run that
                // finds errors is a successful execution, and `--strict` is
                // where a posture lives.
                "executionSuccessful",
                Json::Bool(true),
            )])]),
        ),
        ("artifacts", Json::Array(artifacts(&run.read_set.inputs))),
        (
            "results",
            Json::Array(
                reported(run)
                    .iter()
                    .map(|entry| result(entry, &rules, run))
                    .collect(),
            ),
        ),
        ("properties", run_properties(run, subject)),
    ])
}

fn tool(run: &Run) -> Json {
    Json::object([(
        "driver",
        Json::object([
            ("name", Json::string(TOOL)),
            ("informationUri", Json::string(TOOL_URI)),
            (
                "rules",
                Json::Array(run.served.iter().map(descriptor).collect()),
            ),
        ]),
    )])
}

/// One rule, described from the registry that ran rather than from the
/// taxonomy that declared it.
///
/// The scope, the edition and the export targets are read off the trait that
/// binds the check, so this list describes the checker. A list built from the
/// declarations would name a rule the engine declines to evaluate, and a
/// decline is invisible in a declaration: it looks exactly like a constraint.
fn descriptor(served: &headwater_check::Serves) -> Json {
    let obligation = match &served.obligation {
        Bound::To(one) => Json::Array(vec![Json::string(one.clone())]),
        Bound::Several(many) => {
            Json::Array(many.iter().map(|one| Json::string(one.clone())).collect())
        }
        Bound::Unnamed => Json::Array(Vec::new()),
    };
    Json::object([
        ("id", Json::string(served.rule)),
        (
            "shortDescription",
            Json::object([("text", Json::string(served.scope.render()))]),
        ),
        (
            "properties",
            Json::object([(
                "headwater",
                Json::object([
                    ("scope", Json::string(served.scope.grain().name())),
                    ("version", number(served.version as usize)),
                    ("obligations", obligation),
                    (
                        "exportable_as",
                        Json::Array(
                            served
                                .exportable_as
                                .iter()
                                .map(|target| Json::string(*target))
                                .collect(),
                        ),
                    ),
                ]),
            )]),
        ),
    ])
}

/// The read set as SARIF artifacts.
///
/// `analysisTarget` is the role, because every one of these is a file a check
/// read. An input the run could not hash carries no `hashes` member rather than
/// an empty one: spec 12 counts those apart, because a gate can decide nothing
/// about a file whose content it cannot identify.
fn artifacts(inputs: &[Input]) -> Vec<Json> {
    inputs
        .iter()
        .map(|input| {
            let mut members: Vec<(&'static str, Json)> = vec![
                (
                    "location",
                    Json::object([("uri", Json::string(input.path.clone()))]),
                ),
                ("roles", Json::Array(vec![Json::string("analysisTarget")])),
            ];
            if let Some(digest) = &input.digest {
                members.push((
                    "hashes",
                    Json::object([("sha-256", Json::string(bare(digest)))]),
                ));
            }
            Json::object(members)
        })
        .collect()
}

/// A digest without the `sha256:` label the engine prints.
///
/// SARIF keys the algorithm and holds the value alone, so a label inside the
/// value would be a second statement of the algorithm and a hash that no
/// consumer could compare.
fn bare(digest: &str) -> String {
    digest
        .split_once(':')
        .map(|(_, value)| value.to_string())
        .unwrap_or_else(|| digest.to_string())
}

fn result(entry: &Reported<'_>, rules: &[&'static str], run: &Run) -> Json {
    let finding = entry.finding;
    let mut members: Vec<(&'static str, Json)> = vec![("ruleId", Json::string(finding.rule))];
    if let Some(at) = rules.iter().position(|rule| *rule == finding.rule) {
        members.push(("ruleIndex", number(at)));
    }
    members.push(("level", Json::string(level(finding.severity))));
    // Every result is a `fail`. A rule that ran and found nothing produces no
    // result here: see the coverage entry of the loss set for why a `pass`
    // result would count the wrong thing.
    members.push(("kind", Json::string("fail")));
    members.push(("message", message(entry)));
    members.push(("locations", Json::Array(vec![location(finding)])));
    if let Some(escape) = entry.escape {
        members.push((
            "suppressions",
            Json::Array(vec![suppression(entry, escape)]),
        ));
    }
    members.push(("properties", result_properties(entry, run)));
    Json::object(members)
}

/// The finding's message, and its remediation beside it in the member SARIF
/// keeps for one.
///
/// `text` is the finding. `markdown` adds the remedy, because
/// [spec 4](../../../../docs/spec/04-assurance-model.md#findings) asks for "one
/// path from finding to fix" and a consumer that renders Markdown is the one
/// place in this vocabulary where the path can be shown rather than stored.
fn message(entry: &Reported<'_>) -> Json {
    let finding = entry.finding;
    let mut markdown = format!("{}\n\n**Fix:** {}", finding.message, finding.remediation);
    if let Some(escape) = entry.escape {
        markdown.push_str(&format!("\n\n_Not reported: {}._", held_by(entry, escape)));
    }
    Json::object([
        ("text", Json::string(finding.message.clone())),
        ("markdown", Json::string(markdown)),
    ])
}

fn location(finding: &headwater_check::Finding) -> Json {
    let artifact = Json::object([("uri", Json::string(finding.path.clone()))]);
    // A finding with no line is about the document rather than about a place in
    // it. SARIF wants a region to start at line 1 or later, so such a finding
    // carries the file and no region, which is what the text report does when
    // it prints the path alone.
    let physical = match finding.line {
        0 => Json::object([("artifactLocation", artifact)]),
        line => {
            let mut region: Vec<(&'static str, Json)> = vec![("startLine", number(line))];
            if finding.column > 0 {
                region.push(("startColumn", number(finding.column)));
            }
            Json::object([
                ("artifactLocation", artifact),
                ("region", Json::object(region)),
            ])
        }
    };
    Json::object([("physicalLocation", physical)])
}

/// Why this finding is not among the ones a reader sees.
fn held_by(entry: &Reported<'_>, escape: Escape) -> String {
    match escape {
        Escape::MigrationPending => match entry.task {
            Some(task) => format!(
                "held by the adoption task {}, owned by {}, until {}",
                task.id,
                task.owner,
                task.until.render()
            ),
            None => "held by the adoption payload".to_string(),
        },
        Escape::Suppression => match entry.directive {
            Some(directive) => {
                let note = match directive.note.is_empty() {
                    true => String::new(),
                    false => format!(" ({})", directive.note),
                };
                format!(
                    "suppressed at {}:{} as {}, until {}{note}",
                    directive.path,
                    directive.line,
                    directive.reason.name(),
                    directive.until.render()
                )
            }
            None => "suppressed by a directive".to_string(),
        },
    }
}

/// One SARIF suppression.
///
/// `status` is `accepted` in both classes, and it is a statement about the
/// escape rather than about the finding: a directive carries a reason from a
/// closed set and a task carries an owner, so both were reviewed by somebody
/// with a name. `underReview` would describe a state this engine has no member
/// for.
fn suppression(entry: &Reported<'_>, escape: Escape) -> Json {
    Json::object([
        ("kind", Json::string(kind(escape))),
        ("status", Json::string("accepted")),
        ("justification", Json::string(held_by(entry, escape))),
        (
            "properties",
            Json::object([(
                "headwater",
                Json::object([("escape", Json::string(escape.name()))]),
            )]),
        ),
    ])
}

fn result_properties(entry: &Reported<'_>, run: &Run) -> Json {
    let finding = entry.finding;
    let mut headwater: Vec<(&'static str, Json)> = vec![
        // The check's severity in the engine's own word, beside the SARIF level
        // above. `warn` and `warning` are the same judgment in two vocabularies,
        // and a consumer that reads one should be able to see the other.
        ("severity", Json::string(crate::severity(finding.severity))),
        ("fixable", Json::Bool(finding.fixable())),
        ("remediation", Json::string(finding.remediation.clone())),
        (
            "escape",
            match entry.escape {
                Some(escape) => Json::string(escape.name()),
                None => Json::string("none"),
            },
        ),
    ];
    if let Some(obligation) = &finding.obligation {
        headwater.push(("obligation", Json::string(obligation.clone())));
        // The other scale, in the member the loss set points at. It is read off
        // the register rather than off the finding, because it is a property of
        // the invariant and a finding carries the check's severity instead.
        let declared = run
            .register
            .obligations
            .iter()
            .find(|disposed| disposed.id == *obligation)
            .and_then(|disposed| disposed.severity.clone());
        if let Some(declared) = declared {
            headwater.push(("obligation_severity", Json::string(declared)));
        }
    }
    Json::object([("headwater", Json::object(headwater))])
}

fn run_properties(run: &Run, subject: &Subject<'_>) -> Json {
    let all = reported(run);
    let escaped = |escape: Escape| {
        all.iter()
            .filter(|entry| entry.escape == Some(escape))
            .count()
    };
    Json::object([(
        "headwater",
        Json::object([
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
            (
                "escaped",
                Json::object([
                    // In the precedence spec 4 fixes. The first is absent
                    // because this engine has no waiver mechanism, and a
                    // reader who meets two counts should not have to work out
                    // whether the third is zero or missing.
                    ("waived", Json::string("no waiver mechanism exists")),
                    (
                        "migration_pending",
                        number(escaped(Escape::MigrationPending)),
                    ),
                    ("suppressed", number(escaped(Escape::Suppression))),
                ]),
            ),
            (
                "read_set",
                Json::object([
                    ("inputs", number(run.read_set.inputs.len())),
                    ("unhashed", number(run.read_set.unhashed())),
                ]),
            ),
            ("loss_set", loss_set()),
        ]),
    )])
}

/// The loss set, written into the artifact.
///
/// The declaration travels with the file, as it does for the graph emitters:
/// a consumer that holds the bytes and not the repository can still read what
/// this vocabulary could not carry.
pub fn loss_set() -> Json {
    Json::Array(
        Format::Sarif
            .loss()
            .iter()
            .map(|loss| {
                Json::object([
                    ("field", Json::string(loss.field)),
                    ("reason", Json::string(loss.reason)),
                    ("carried_in", Json::string(loss.carried_in)),
                ])
            })
            .collect(),
    )
}

/// A whole number as JSON.
///
/// [`Json`] carries no number type, because the protocol it was written for
/// echoes an identifier in the type it arrived in. A count is written through
/// `Raw` for the same reason every scalar the export emitters write is a
/// string: one writer, and the shape of the value is the caller's statement.
fn number(value: usize) -> Json {
    Json::Raw(value.to_string())
}
