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
//! # Where a scoped run says so, and the two members that were read first
//!
//! A run of `headwater check --change` is told what one change carries. It
//! checks the whole corpus either way, and the manifest is what lets the rules
//! that read a prior version reach a verdict rather than a skip. That fact has
//! no member in this vocabulary, so it rides in
//! `run.properties.headwater.change` and [`LOSS`] records it as a loss. Two
//! defined members were held against it first, and the schema is what refused
//! both.
//!
//! The `invocation` object is "the runtime environment of the analysis tool
//! run", and every member of it is a fact about the process: the command line,
//! the arguments, the working directory, the exit code, the streams. The
//! counts a scoped run reports are not process facts. They are what the engine
//! made of a manifest after it read it, and a document whose path reached no
//! row of the corpus is a reading rather than an argument. So `invocations` is
//! the wrong object, and its own property bag would be a bag one level further
//! from the reader than the run's.
//!
//! `run.automationDetails.id` is the member a consumer reads to decide which
//! runs to compare, and it is the one that would stop a forge from baselining
//! a scoped run against a full-corpus one. The schema defines it as "a
//! hierarchical string that uniquely identifies this object's containing run
//! object", and `correlationGuid` beside it as a GUID. This engine mints no run
//! identity: [`crate::text`] and every artifact here are a function of the
//! corpus, the lock and the injected clock alone, so a value that varied per
//! run would break the determinism spec 12 fixes, and a constant would identify
//! a class rather than a run. Writing either would be this emitter filling a
//! uniqueness member with something that is not unique. The residue is real: a
//! consumer that baselines still cannot tell the two runs apart from a member
//! of the vocabulary, and the property bag is where the answer is.
//!
//! One entry is worth reading twice. Spec 6 asks a run to report "the corpus
//! tree, the taxonomy lock hash, and its read set". SARIF has
//! `run.automationDetails.id`, which is exactly where a corpus tree would go,
//! and this emitter leaves it out: nothing in the engine computes a corpus tree,
//! and the read set is not one, because the read set holds what the checks read
//! and a tree holds what the census walked. Writing the read set's identity into
//! a member that means *the tree this run is about* would be the emitter
//! printing something false to fill a slot.

use crate::{
    a_scoped_run, every_finding, every_run, reported, Carrier, Escape, Format, Loss, Place,
    Reported, Subject, TOOL, TOOL_URI,
};
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
///
/// Five of the seven entries name a place in this document, six places between
/// them, and [`crate::census`] resolves each one against the emitted bytes. The
/// other two carry [`Carrier::Nowhere`]. The `coverage` entry names the eight
/// members its own reason lists, rather than the bag they sit in: an entry that
/// named the bag alone stood for as long as it existed while three of the seven
/// values coverage reports went nowhere, and a bag is what it named. Naming the
/// bag is now a fault the census reports rather than a habit a reader has to
/// catch, so the four places whose `members` are empty are four values that are
/// each one string.
pub const LOSS: &[Loss] = &[
    Loss {
        field: "the obligation's severity",
        reason: "`level` is the check's scale, and SARIF has no second member for how much an \
                 invariant matters. A consumer that ranks by `level` ranks by the check",
        carrier: Carrier::PerFinding {
            places: &[Place {
                at: &["properties", "headwater", "obligation_severity"],
                members: &[],
            }],
            // The member is on the results whose finding names an obligation
            // that the register grades, and on no others. The predicate reads
            // the same register the emitter reads and it reads it for itself,
            // so a change to the emitter's lookup alone moves the artifact
            // without moving the expectation, and this audit reports it.
            when: a_graded_obligation,
        },
    },
    Loss {
        field: "the escape class",
        reason: "`suppression.kind` has two values and spec 4 fixes three escape classes. A \
                 directive is `inSource`; a task of the adoption payload is `external`, and a \
                 waiver would be `external` too",
        carrier: Carrier::PerFinding {
            places: &[Place {
                at: &["properties", "headwater", "escape"],
                members: &[],
            }],
            when: every_finding,
        },
    },
    Loss {
        field: "the control's posture",
        reason: "no member says whether a finding stops a gate, and the engine emits what it \
                 evaluated and never orders what lands",
        carrier: Carrier::Nowhere,
    },
    Loss {
        field: "coverage",
        reason: "a `kind: \"pass\"` result would count check instances, and the census counts \
                 documents, including the ones no check classified. What is carried is every \
                 number the text report writes and one it does not: the counts, the skip classes \
                 with their reasons, the documents each class was routed to, and the paths no \
                 census row accounts for",
        carrier: Carrier::Run {
            places: &[Place {
                at: &["properties", "headwater", "coverage"],
                // The seven values the reason above names, and `generated`
                // beside them. Named one by one, because the bag is what the
                // entry used to name and the bag was there while three of them
                // were not.
                members: &[
                    "seen",
                    "classified",
                    "checked",
                    "generated",
                    "instances",
                    "skipped",
                    "skips",
                    "unaccounted",
                ],
            }],
            when: every_run,
        },
    },
    Loss {
        field: "the remediation of a mechanical fix",
        reason: "`fixes[]` carries replacement text and a finding carries prose with a \
                 `fixable` flag, so no result here carries a `fix`",
        carrier: Carrier::PerFinding {
            places: &[
                Place {
                    at: &["properties", "headwater", "remediation"],
                    members: &[],
                },
                Place {
                    at: &["message", "markdown"],
                    members: &[],
                },
            ],
            when: every_finding,
        },
    },
    Loss {
        field: "the change a run was scoped to",
        reason: "no member of this vocabulary says which change a run was told about. \
                 `invocations` holds the runtime environment of the tool process and these are \
                 readings rather than process facts, and `automationDetails.id` uniquely \
                 identifies one run, which is an identity this engine mints none of",
        carrier: Carrier::Run {
            places: &[Place {
                at: &["properties", "headwater", "change"],
                members: &[
                    "documents",
                    "added",
                    "carried",
                    "unreadable",
                    "unmatched",
                    "promotions",
                ],
            }],
            // A full-corpus run writes no member at all, and the absence is what
            // tells the two runs apart. So the census requires the member on a
            // scoped run and requires its absence on every other, which is the
            // reading #234 rests on.
            when: a_scoped_run,
        },
    },
    Loss {
        field: "the corpus tree",
        reason: "`run.automationDetails.id` is where one would go and nothing computes one. The \
                 read set is not a tree: it holds what the checks read",
        carrier: Carrier::Nowhere,
    },
];

/// The severity the register declares for the obligation a finding serves.
///
/// The emitter's one reader of it. The loss set's own predicate above asks the
/// same question of the same register and does not call this, because a
/// predicate that shared the emitter's lookup would follow it wherever it went:
/// a change here would move the artifact and the expectation together, and the
/// audit would stay silent on the one class of defect it exists to report.
pub fn obligation_severity(entry: &Reported<'_>, run: &Run) -> Option<String> {
    let obligation = entry.finding.obligation.as_ref()?;
    run.register
        .obligations
        .iter()
        .find(|disposed| disposed.id == *obligation)
        .and_then(|disposed| disposed.severity.clone())
}

/// Whether this result carries the member the first entry of [`LOSS`] names.
///
/// It reads the register off the run, and it does not call
/// [`obligation_severity`]. The two are the same question asked of the same
/// data, and that is the point: a predicate that called the emitter's own
/// lookup would move with any change to it, so the audit would agree with the
/// emitter whatever the emitter did. What audits an emitter has to be able to
/// disagree with it.
fn a_graded_obligation(entry: &Reported<'_>, run: &Run) -> bool {
    let Some(obligation) = entry.finding.obligation.as_ref() else {
        return false;
    };
    run.register
        .obligations
        .iter()
        .any(|disposed| disposed.id == *obligation && disposed.severity.is_some())
}

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

/// How this emitter reads the obligation severity of one reported finding.
///
/// A parameter rather than a call, for the reason [`crate::census_with`] takes
/// a loss set: the audit of the first [`LOSS`] entry is a claim about what this
/// emitter writes, and a test that cannot move the emitter can only ever
/// measure the emitter agreeing with itself. Production passes
/// [`obligation_severity`] and nothing else does.
pub type Grading = fn(&Reported<'_>, &Run) -> Option<String>;

/// One run as a SARIF 2.1.0 log.
pub fn render(run: &Run, subject: &Subject<'_>) -> String {
    render_with(run, subject, obligation_severity)
}

/// The same log, over a grading the caller names.
///
/// This exists for a suite, and the review question underneath it is the one
/// [`crate::census_with`] answers at the other end: a census that passed a
/// silent emitter would be an audit with nothing to report. Hand it a grading
/// that returns nothing over a run whose register grades an obligation, and
/// the census has to say so.
pub fn render_with(run: &Run, subject: &Subject<'_>, grading: Grading) -> String {
    document(run, subject, grading).render_pretty()
}

fn document(run: &Run, subject: &Subject<'_>, grading: Grading) -> Json {
    Json::object([
        ("$schema", Json::string(SCHEMA)),
        ("version", Json::string(VERSION)),
        ("runs", Json::Array(vec![one_run(run, subject, grading)])),
    ])
}

fn one_run(run: &Run, subject: &Subject<'_>, grading: Grading) -> Json {
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
                    .map(|entry| result(entry, &rules, run, grading))
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

fn result(
    entry: &Reported<'_>,
    rules: &[&'static str],
    run: &Run,
    grading: Grading,
) -> Json {
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
    members.push(("properties", result_properties(entry, run, grading)));
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
        markdown.push_str(&format!(
            "\n\n_Not reported: {}._",
            crate::held_by(entry, escape)
        ));
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
        ("justification", Json::string(crate::held_by(entry, escape))),
        (
            "properties",
            Json::object([(
                "headwater",
                Json::object([("escape", Json::string(escape.name()))]),
            )]),
        ),
    ])
}

fn result_properties(entry: &Reported<'_>, run: &Run, grading: Grading) -> Json {
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
        // [`obligation_severity`] is this emitter's one reader of it. The
        // predicate on that loss entry reads the register for itself, because
        // an audit that shared this lookup could not report a change to it.
        if let Some(declared) = grading(entry, run) {
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
    let mut headwater: Vec<(&'static str, Json)> = vec![
        // The version of the two shapes this bag shares with `--format json`.
        // Without it the statement each of them makes is available to a reader
        // of one artifact and not to a reader of the other: an absent `change`
        // means a full-corpus run only to a reader who knows the producer would
        // have written one, and `"skipped": 0` means every instance reached a
        // verdict only to a reader who knows this producer counts them.
        ("shape", Json::string(crate::json::VERSION)),
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
    // Above coverage here as in every other format, and written only for a run
    // that was scoped. See the module comment for the two members of the
    // vocabulary that were held against this bag first.
    if let Some(scoped) = &run.change {
        headwater.push(("change", crate::json::change(scoped)));
    }
    headwater.extend([
        (
            "coverage",
            crate::json::coverage(&run.coverage, &run.instances),
        ),
        (
            "escaped",
            Json::object([
                // In the precedence spec 4 fixes. The first is absent
                // because no waiver reaches a check finding, and a reader
                // who meets two counts should not have to work out whether
                // the third is zero or missing. The mechanism does ship —
                // `headwater conformance` reads a waiver over a conformance
                // rule — and spec 7 states that the reader for the other
                // population is what has not landed.
                ("waived", Json::string("no waiver reaches a check finding")),
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
    ]);
    Json::object([("headwater", Json::object(headwater))])
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
                    ("carried_in", Json::string(loss.carried_in())),
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
