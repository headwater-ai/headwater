// SPDX-License-Identifier: Apache-2.0
//! The grader: a pure function of a recorded transcript, the expectations the
//! selected probes declare, and its own version.
//!
//! # What earns this component the right to grade
//!
//! Every other mechanism in this engine that reaches toward a model refuses to
//! return a verdict. The [sweep](../../sweep/index.html) verifies the citations
//! of what an agent wrote and reports findings that a person accepts. The
//! [intake](../intake/index.html) confirms five things about a transcript and
//! evaluates no expectation. This module grades, and three properties are the
//! whole of why it may.
//!
//! **Its inputs carry no prose.** A transcript is an event log whose every key
//! is a member of a closed set, and an expectation is one of five predicates
//! that the taxonomy fixes. Nothing here reads a sentence, so nothing here
//! needs a rubric, and [spec 5](../../../../docs/spec/05-ai-integration.md#a-probe-is-a-document-with-a-declared-expectation)
//! sends a question that needs one to the sweep instead.
//!
//! **Every satisfied verdict carries a witness, and the type is what carries
//! it.** [`Verdict::Satisfied`] holds a [`Witness`], so a satisfied verdict
//! with nothing behind it does not compile. The witness names the event, the
//! call, the artifact or the answer that satisfied the predicate, and a reader
//! re-derives the verdict by counting events in the committed transcript. That
//! is the difference between a grade and an opinion: a grade names the line it
//! came from. A [`Miss`] carries the extent of the search for the same reason,
//! because "nothing matched" is a claim about how much was looked at.
//!
//! **It refuses where a green answer would be free.** Six conditions below
//! return no verdict rather than a passing one, and each is a place where the
//! cheapest wrong grader returns `Satisfied`. A `patched` expectation whose
//! probe declares no oracle. A produced artifact that nothing checked. A
//! `not_opened` over a session that made no call at all. A form whose input the
//! recorder never wrote down.
//!
//! # Determinism is necessary and it is not the argument
//!
//! `Results::over` reads two values and a version. Run it twice and it writes
//! one set of bytes, which is what makes a probe result reproducible where the
//! behavior under it is not. It is not what makes the result *right*: a grader
//! that returned `Satisfied` for everything is perfectly deterministic. So the
//! recorded fixture set is the instrument, and the property above is the design
//! that gives the fixtures something to hold. Spec 12 asks a correctness root
//! for its own failing fixtures, and every verdict form and every refusal here
//! has one.
//!
//! # The version, and why it is the harness version
//!
//! A result is a function of three inputs and the third is this code. A series
//! that averaged over two versions of it would report a change in the
//! instrument as a change in the corpus, so [`VERSION`] is printed on every
//! result and a report over a set of results says how many versions the set
//! spans. It is the version of this crate, which is the harness version, for
//! the reason the harness version is the engine's: two numbers that drift name
//! two graders. A change to this file inside one version is caught by the
//! recorded fixtures rather than by the number, which is why they are recorded
//! whole.
//!
//! # What this does not do
//!
//! It compares no arms. A transcript records one arm, so an efficacy claim is a
//! report over two results and this is one of them. It reads no probe that the
//! selection did not carry, opens no file, and reaches no network.

use crate::intake::{Answer, Event, Produced, Record};
use crate::plan::{Examined, Selected};
use crate::{Arm, Category, Expectation, Tier};
use headwater_check::paint::{paint, ColorMode, Role};

/// The grader version, which a result names for the reason a reading names its
/// lock digest. This is the engine version, read from the one constant that
/// defines it, rather than this crate's own manifest version — the two are
/// the same number by the workspace's own design, and reading the constant
/// directly is what keeps them from drifting apart.
pub const VERSION: &str = headwater_resolve::release::ENGINE;

/// The confidence coefficient of the reported interval, at 95%.
const Z: f64 = 1.959_964;

/// What in the transcript satisfied the predicate.
///
/// [`Verdict::Satisfied`] holds one, so there is no way to record a satisfied
/// verdict without naming the thing that satisfied it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Witness {
    /// A tool call whose argument named one of the examined documents.
    Read {
        event: usize,
        call: usize,
        tool: String,
        argument: String,
    },
    /// Every recorded call was read and none named one, which is what
    /// `not_opened` is satisfied by. The count is the extent of the search.
    NoneOf { calls: usize, over: usize },
    /// A produced artifact carries one of the examined identifiers.
    Cites {
        event: usize,
        artifact: String,
        identifier: String,
    },
    /// The final answer, and the closed set it is a member of.
    Answered { event: usize, value: String },
    /// A produced artifact was checked and the oracle did not report over it.
    Clean {
        event: usize,
        artifact: String,
        oracle: String,
    },
}

impl std::fmt::Display for Witness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Witness::Read {
                event,
                call,
                tool,
                argument,
            } => write!(f, "event {event}, call {call}: `{tool} {argument}`"),
            Witness::NoneOf { calls, over } => write!(
                f,
                "{}, and none named any of the {}",
                crate::plural(*calls, "recorded call"),
                crate::plural(*over, "document")
            ),
            Witness::Cites {
                event,
                artifact,
                identifier,
            } => write!(f, "event {event}: `{artifact}` cites {identifier}"),
            Witness::Answered { event, value } => write!(f, "event {event}: answered `{value}`"),
            Witness::Clean {
                event,
                artifact,
                oracle,
            } => write!(
                f,
                "event {event}: `{oracle}` reported nothing over `{artifact}`"
            ),
        }
    }
}

/// Why the predicate was not satisfied, with the extent of what was read.
///
/// A miss says how far the search went, because "nothing matched" over an
/// unstated denominator is the same sentence whether one call was read or none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Miss {
    /// No recorded call named any of the examined documents.
    NeverRead { calls: usize, over: usize },
    /// A call did, which is what refutes `not_opened`.
    Read {
        event: usize,
        call: usize,
        argument: String,
    },
    /// No produced artifact carries any of the examined identifiers.
    NeverCited { artifacts: usize, over: usize },
    /// The session ended with no answer, and the recorder observed that.
    NoAnswerGiven,
    /// The answer is outside the closed set the probe declares.
    Outside { event: usize, value: String },
    /// The answer is a value of the closed set and not one the probe expects.
    Wrong {
        event: usize,
        value: String,
        expected: Vec<String>,
    },
    /// The oracle reported over every produced artifact.
    OracleReported { artifacts: usize, oracle: String },
    /// A `patched` session produced no artifact, so the oracle had nothing to
    /// read. The pilot of #980 printed this as the oracle reporting "over each
    /// of the 0 produced artifacts", which names a finding nobody made.
    NothingProduced,
}

impl std::fmt::Display for Miss {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Miss::NeverRead { calls, over } => write!(
                f,
                "{}, and none named any of the {}",
                crate::plural(*calls, "recorded call"),
                crate::plural(*over, "document")
            ),
            Miss::Read {
                event,
                call,
                argument,
            } => write!(f, "event {event}, call {call} read `{argument}`"),
            Miss::NeverCited { artifacts, over } => write!(
                f,
                "{}, and none cites any of the {}",
                crate::plural(*artifacts, "produced artifact"),
                crate::plural(*over, "identifier")
            ),
            Miss::NoAnswerGiven => write!(f, "the session ended with no answer"),
            Miss::Outside { event, value } => write!(
                f,
                "event {event} answered `{value}`, which the probe does not declare"
            ),
            Miss::Wrong {
                event,
                value,
                expected,
            } => write!(
                f,
                "event {event} answered `{value}`, and the probe expects {}",
                expected
                    .iter()
                    .map(|value| format!("`{value}`"))
                    .collect::<Vec<_>>()
                    .join(" or ")
            ),
            Miss::OracleReported { artifacts, oracle } => write!(
                f,
                "`{oracle}` reported over each of the {}",
                crate::plural(*artifacts, "produced artifact")
            ),
            Miss::NothingProduced => write!(
                f,
                "the session produced no artifact, so the oracle had nothing to read"
            ),
        }
    }
}

/// Why there is no verdict.
///
/// Every one of these is a place where a grader that wanted a number could have
/// returned one. A refused session leaves the rate's denominator rather than
/// joining its numerator, and the count of them is printed beside the rate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The transcript records no event for this probe.
    NotRun,
    /// A `patched` expectation whose probe declares no oracle. The taxonomy
    /// cannot require the field on that form alone
    /// ([HW-OBL-0123](../../../../docs/obligations/0123-a-facet-that-applies-to-one-value-of-another-facet-has-nowhere-to-say-so.md)),
    /// so the sentinel reaches here as a declared absence and an absence is not
    /// a pass.
    OracleUndeclared,
    /// An `answered` expectation whose probe declares no closed set, or no
    /// expected value that is a proper part of one. Every string, or every
    /// in-domain string, would then be the expected one.
    AnswersUndeclared,
    /// A `cited` expectation whose examined targets carry no identifier. An
    /// external anchor is a path, and nothing cites a path.
    NothingCitable { over: usize },
    /// The event carries no key for the thing this form reads.
    Unrecorded { what: &'static str },
    /// A `not_opened` over a session that recorded no call at all. A session
    /// that did nothing is not evidence that it avoided something, and counting
    /// it as satisfied fills the numerator with sessions that failed to start.
    NothingObserved,
    /// A produced artifact that nothing checked, under a `patched` expectation.
    /// `findings: []` is checked and clean, and an absent key is unchecked.
    NotChecked { artifact: String },
}

impl Refusal {
    /// Whether the session caused this refusal, rather than the recorder or
    /// the probe declaration.
    ///
    /// The owner ruled on #980 (2026-09-28) that the two classes are different
    /// facts. A refusal the recorder or the declaration caused is a defect: the
    /// remedy is to record the session again, and a paired run that still holds
    /// one fails. A refusal the session caused is data about the arm it ran in:
    /// a `not_opened` session that made no call did nothing, and an arm whose
    /// sessions do nothing more often is a finding. So it is counted per arm and
    /// never fails the run. Before the ruling any difference in the refused
    /// count between two arms failed the run, and across hundreds of sessions
    /// per arm one stray session would discard the batch.
    ///
    /// This is an exhaustive match, so that a refusal added later has to answer
    /// the question rather than inherit an answer.
    pub fn is_session(&self) -> bool {
        match self {
            Refusal::NothingObserved => true,
            Refusal::NotRun
            | Refusal::OracleUndeclared
            | Refusal::AnswersUndeclared
            | Refusal::NothingCitable { .. }
            | Refusal::Unrecorded { .. }
            | Refusal::NotChecked { .. } => false,
        }
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refusal::NotRun => write!(
                f,
                "this transcript records no event for it, so the run says nothing about it either \
                 way"
            ),
            Refusal::OracleUndeclared => write!(
                f,
                "it expects `patched` and its probe declares `oracle: {}`. That sentinel is a \
                 declared absence, and an empty oracle graded as a pass would return a satisfied \
                 verdict for a check nobody ran",
                crate::plan::NO_ORACLE
            ),
            Refusal::AnswersUndeclared => write!(
                f,
                "it expects `answered` and its probe declares no closed set with a proper subset \
                 of expected values in it, so every answer the session returned would be the \
                 expected one"
            ),
            Refusal::NothingCitable { over } => write!(
                f,
                "it expects `cited` and none of the {} it examines carries an identifier. A \
                 produced artifact cites an identifier, and nothing cites a path",
                crate::plural(*over, "target")
            ),
            Refusal::Unrecorded { what } => write!(
                f,
                "the event carries no `{what}` key, so nothing recorded what this form reads. An \
                 empty list is a recorded absence and a missing key is not one"
            ),
            Refusal::NothingObserved => write!(
                f,
                "it expects `not_opened` and the session recorded no tool call at all. A session \
                 that did nothing is not evidence that it avoided something"
            ),
            Refusal::NotChecked { artifact } => write!(
                f,
                "`{artifact}` carries no `findings` key, so nothing says whether the oracle was \
                 ever run over it. `findings: []` is checked and clean"
            ),
        }
    }
}

/// One verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    Satisfied(Witness),
    NotSatisfied(Miss),
    Refused(Refusal),
}

impl Verdict {
    pub fn name(&self) -> &'static str {
        match self {
            Verdict::Satisfied(_) => "satisfied",
            Verdict::NotSatisfied(_) => "not satisfied",
            Verdict::Refused(_) => "no verdict",
        }
    }

    /// The evidence, whichever kind it is.
    pub fn because(&self) -> String {
        match self {
            Verdict::Satisfied(witness) => witness.to_string(),
            Verdict::NotSatisfied(miss) => miss.to_string(),
            Verdict::Refused(refusal) => refusal.to_string(),
        }
    }
}

/// One graded session of one probe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Graded {
    pub session: String,
    /// The events this session's record came from, one-based, in order.
    pub events: Vec<usize>,
    pub verdict: Verdict,
}

/// One probe of the selection, and every session the transcript ran it in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub probe: String,
    pub category: Category,
    pub expectation: Expectation,
    pub sessions: Vec<Graded>,
}

/// A closed-interval estimate of a rate, as a proportion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    pub point: f64,
    pub low: f64,
    pub high: f64,
}

impl Interval {
    /// The difference `self - other`, with a Newcombe hybrid score interval.
    ///
    /// Spec 5 powers a campaign for a two-sample test: 0.50 against 0.75 at 80%
    /// power and a 5% two-sided level is about 58 sessions per arm. Reading the
    /// two arms' Wilson intervals for overlap is a much stricter test, with
    /// roughly half that power at the same count (#980). This interval is the
    /// test the count was taken for. It is built from the two Wilson intervals
    /// the grader already reports, so it stays inside minus one and one at the
    /// session counts a pilot runs, and the difference is significant at the
    /// 5% level where it excludes zero.
    pub fn minus(&self, other: &Interval) -> Interval {
        let point = self.point - other.point;
        let low =
            point - ((self.point - self.low).powi(2) + (other.high - other.point).powi(2)).sqrt();
        let high =
            point + ((self.high - self.point).powi(2) + (other.point - other.low).powi(2)).sqrt();
        Interval {
            point,
            low: low.max(-1.0),
            high: high.min(1.0),
        }
    }
}

/// The part of a selection a transcript was planned over, where that is not the
/// whole selection.
///
/// A campaign is planned by category, and a probe can be named out of it
/// (#980), so its transcript records a selection digest over fewer probes than
/// this corpus declares. Graded against the whole selection, every probe the
/// run never planned reads as a refused session, and a refusal is a defect of
/// the recorder. So the probes the transcript names are taken out of the
/// selection, and they are the selection the run was planned over only when
/// their digest is the one the transcript recorded. Anything else is `None`,
/// and the caller grades against the whole selection as before: a narrowed run
/// that lost a probe entirely is a run whose digest no longer matches, and its
/// missing probe is reported rather than quietly dropped.
pub fn narrowed(selection: &[Selected], record: &Record) -> Option<Vec<Selected>> {
    let recorded = &record.identity.as_ref()?.selection;
    match planned_over(selection, recorded, &record.probes)? {
        Planned::Whole => None,
        Planned::Part(part) => Some(part),
    }
}

/// What a recorded selection digest is, against the selection this tree
/// composes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Planned {
    /// The recorded digest is the digest of the whole selection.
    Whole,
    /// The recorded digest is the digest of these probes, which are the probes
    /// of the selection that the transcript's events name.
    Part(Vec<Selected>),
}

/// The selection a transcript was planned over, recovered from the digest it
/// recorded and the probes its events name, or `None` where it is neither the
/// whole selection nor that part of it.
///
/// This is the one statement of the rule. [`narrowed`] grades against the
/// part, the intake compares the part's read set where the lock moved
/// (#1292), and [`crate::read_set::Staleness::over`] holds a committed result
/// against the part's read set (#1387). A transcript records no `--exclude`
/// list, so the digest over the probes it names is the only record of which
/// probes it was planned over.
pub fn planned_over(selection: &[Selected], recorded: &str, probes: &[String]) -> Option<Planned> {
    let whole: Vec<&str> = selection
        .iter()
        .map(|selected| selected.id.as_str())
        .collect();
    if crate::plan::selection_digest(&whole) == recorded {
        return Some(Planned::Whole);
    }
    let part: Vec<Selected> = selection
        .iter()
        .filter(|selected| probes.contains(&selected.id))
        .cloned()
        .collect();
    let ids: Vec<&str> = part.iter().map(|selected| selected.id.as_str()).collect();
    match !part.is_empty() && crate::plan::selection_digest(&ids) == recorded {
        true => Some(Planned::Part(part)),
        false => None,
    }
}

/// Every verdict of one transcript.
#[derive(Clone, Debug)]
pub struct Results {
    pub grader: &'static str,
    /// The run this transcript recorded, where the intake read one.
    pub tier: Option<Tier>,
    pub arm: Option<Arm>,
    pub model: Option<String>,
    pub served_version: Option<String>,
    pub rows: Vec<Row>,
    /// Why nothing was graded: the intake refused the transcript whole.
    pub unusable: Option<String>,
}

impl Results {
    /// Grade a record against the selection that planned it.
    ///
    /// It never fails. A transcript the intake refused produces results with
    /// [`Results::unusable`] set and no rows, because a refused transcript
    /// reaches no grader and saying so is the report.
    pub fn over(record: &Record, selection: &[Selected]) -> Results {
        let mut results = Results {
            grader: VERSION,
            tier: record.identity.as_ref().map(|identity| identity.tier),
            arm: record.identity.as_ref().map(|identity| identity.arm),
            model: record
                .identity
                .as_ref()
                .map(|identity| identity.model.clone()),
            served_version: record
                .identity
                .as_ref()
                .map(|identity| identity.served_version.clone()),
            rows: Vec::new(),
            unusable: None,
        };
        if let Some(refusal) = &record.refusal {
            results.unusable = Some(refusal.to_string());
            return results;
        }

        for selected in selection {
            let mut sessions = Vec::new();
            for session in sessions_of(record, &selected.id) {
                let events: Vec<usize> = session.iter().map(|event| event.at).collect();
                sessions.push(Graded {
                    session: session[0].session.clone(),
                    events,
                    verdict: verdict(selected, &session),
                });
            }
            if sessions.is_empty() {
                sessions.push(Graded {
                    session: String::new(),
                    events: Vec::new(),
                    verdict: Verdict::Refused(Refusal::NotRun),
                });
            }
            results.rows.push(Row {
                probe: selected.id.clone(),
                category: selected.category,
                expectation: selected.expectation,
                sessions,
            });
        }
        results.rows.sort_by(|a, b| a.probe.cmp(&b.probe));
        results
    }

    /// Sessions that reached a verdict, which is the denominator of the rate.
    pub fn graded(&self) -> usize {
        self.verdicts()
            .filter(|verdict| !matches!(verdict, Verdict::Refused(_)))
            .count()
    }

    pub fn satisfied(&self) -> usize {
        self.verdicts()
            .filter(|verdict| matches!(verdict, Verdict::Satisfied(_)))
            .count()
    }

    /// Refused sessions that the recorder or the probe declaration caused. See
    /// [`Refusal::is_session`].
    pub fn defects(&self) -> usize {
        self.verdicts()
            .filter(|verdict| matches!(verdict, Verdict::Refused(refusal) if !refusal.is_session()))
            .count()
    }

    /// Refused sessions that the session itself caused. See
    /// [`Refusal::is_session`].
    pub fn session_refusals(&self) -> usize {
        self.verdicts()
            .filter(|verdict| matches!(verdict, Verdict::Refused(refusal) if refusal.is_session()))
            .count()
    }

    /// Sessions that reached no verdict, which is the count printed beside the
    /// rate rather than folded into it.
    pub fn refused(&self) -> usize {
        self.verdicts()
            .filter(|verdict| matches!(verdict, Verdict::Refused(_)))
            .count()
    }

    fn verdicts(&self) -> impl Iterator<Item = &Verdict> {
        self.rows
            .iter()
            .flat_map(|row| row.sessions.iter().map(|session| &session.verdict))
    }

    /// The satisfied rate with its interval, and `None` where nothing was
    /// graded.
    ///
    /// Spec 5: "a probe result reports an interval rather than a point. An
    /// interval that overlaps the previous one is variance. An interval that
    /// does not overlap is drift." It is a Wilson score interval, which stays
    /// inside zero and one at the session counts a regression tier runs, where
    /// the textbook normal interval does not.
    pub fn rate(&self) -> Option<Interval> {
        let n = self.graded();
        if n == 0 {
            return None;
        }
        let n = n as f64;
        let p = self.satisfied() as f64 / n;
        let denominator = 1.0 + Z * Z / n;
        let center = (p + Z * Z / (2.0 * n)) / denominator;
        let half = Z * (p * (1.0 - p) / n + Z * Z / (4.0 * n * n)).sqrt() / denominator;
        Some(Interval {
            point: p,
            low: (center - half).max(0.0),
            high: (center + half).min(1.0),
        })
    }

    /// The report, in the engine's own words.
    ///
    /// `mode` is the color decision the caller already made — this function
    /// reads no stream itself, on the rule `headwater_check::paint`'s module
    /// comment states.
    pub fn render(&self, mode: ColorMode) -> String {
        use std::fmt::Write;
        let mut out = String::new();

        if let Some(why) = &self.unusable {
            let _ = writeln!(
                out,
                "This transcript reached no grader: {why}\n\nA transcript that fails any of the \
                 five confirmations is refused whole, so there are no verdicts to report and no \
                 rate over none."
            );
            return out;
        }

        let _ = writeln!(out, "Graded by grader {}.", self.grader);
        if let (Some(tier), Some(arm), Some(model), Some(served)) = (
            self.tier,
            self.arm,
            self.model.as_ref(),
            self.served_version.as_ref(),
        ) {
            let _ = writeln!(
                out,
                "A {} run in the {} arm, on {model} at served version {served}.",
                tier.name(),
                arm.name()
            );
        }
        let _ = writeln!(out);

        let _ = writeln!(out, "{}", paint(Role::Heading, "## The verdicts", mode));
        let _ = writeln!(out);
        for row in &self.rows {
            let _ = writeln!(
                out,
                "- {} ({}, expects {})",
                paint(Role::Path, &row.probe, mode),
                row.category.name(),
                row.expectation.name()
            );
            for graded in &row.sessions {
                let session = match graded.session.is_empty() {
                    true => "no session".to_string(),
                    false => format!("session {}", graded.session),
                };
                let _ = writeln!(
                    out,
                    "    {session}: {} — {}",
                    graded.verdict.name(),
                    graded.verdict.because()
                );
            }
        }
        let _ = writeln!(out);

        let _ = writeln!(
            out,
            "{}",
            paint(
                Role::Heading,
                "## The rate, and the denominator it is over",
                mode
            )
        );
        let _ = writeln!(out);
        match self.rate() {
            None => {
                let _ = writeln!(
                    out,
                    "No session reached a verdict, so this run reports no rate. {} were refused, \
                     and a rate over none of them would be a number about nothing.",
                    crate::plural(self.refused(), "session")
                );
            }
            Some(interval) => {
                let _ = writeln!(
                    out,
                    "{} of {} graded sessions satisfied their expectation: {}, in a 95% interval \
                     of {} to {}.",
                    self.satisfied(),
                    self.graded(),
                    percent(interval.point),
                    percent(interval.low),
                    percent(interval.high)
                );
                let _ = writeln!(
                    out,
                    "The denominator is the graded sessions and never the selected probes. {} \
                     reached no verdict, and a session with no verdict is outside both halves of \
                     that fraction.",
                    crate::plural(self.refused(), "session")
                );
            }
        }
        if self.rate().is_some() {
            let _ = writeln!(out);
            let _ = writeln!(
                out,
                "An interval that overlaps the previous run's is variance and one that does not \
                 is drift. This is one arm, so it estimates no effect: an efficacy claim is a \
                 comparison of two results, and the arm each one recorded is on it."
            );
        }
        out
    }
}

/// A proportion, as a reader reads it.
pub fn percent(value: f64) -> String {
    format!("{:.1}%", value * 100.0)
}

/// A difference of two proportions, in percentage points and with its sign.
pub fn points(value: f64) -> String {
    format!("{:+.1} points", value * 100.0)
}

/// The events of one probe, grouped by session, in the order the sessions first
/// appear.
///
/// A session is the unit a verdict is about, and a transcript may record one
/// session over several events. Grouping here rather than grading each event
/// separately is what keeps a session that read a document in its second event
/// from being graded twice and missed once.
fn sessions_of<'a>(record: &'a Record, probe: &str) -> Vec<Vec<&'a Event>> {
    let mut out: Vec<Vec<&Event>> = Vec::new();
    let mut names: Vec<&str> = Vec::new();
    for event in &record.events {
        if event.probe != probe {
            continue;
        }
        match names.iter().position(|name| *name == event.session) {
            Some(index) => out[index].push(event),
            None => {
                names.push(&event.session);
                out.push(vec![event]);
            }
        }
    }
    out
}

/// The predicate, evaluated over one session.
fn verdict(selected: &Selected, session: &[&Event]) -> Verdict {
    match selected.expectation {
        Expectation::Opened => opened(selected, session),
        Expectation::NotOpened => not_opened(selected, session),
        Expectation::Cited => cited(selected, session),
        Expectation::Answered => answered(selected, session),
        Expectation::Patched => patched(selected, session),
    }
}

/// One tool call, located.
struct Found<'a> {
    /// The event, one-based in the order the block lists them.
    event: usize,
    /// The call, one-based within that event.
    call: usize,
    made: &'a crate::intake::Call,
}

/// The first recorded call that named an examined document, where there is one.
///
/// It returns the call rather than a verdict, because `opened` and `not_opened`
/// read the same fact and draw opposite conclusions from it. A helper that
/// returned one of their verdicts would leave the other with a case it cannot
/// reach, and an unreachable case in a correctness root is a case nobody tests.
fn read<'a>(examines: &[Examined], session: &[&'a Event]) -> Option<Found<'a>> {
    for event in session {
        let Some(calls) = &event.calls else { continue };
        for (index, call) in calls.iter().enumerate() {
            if examines.iter().any(|target| names(&call.argument, target)) {
                return Some(Found {
                    event: event.at,
                    call: index + 1,
                    made: call,
                });
            }
        }
    }
    None
}

/// Whether a tool-call argument names an examined target.
///
/// Equality, or the argument ending at a path boundary. The recorder writes the
/// argument as the session passed it, and a session driven from another working
/// directory passes an absolute path for the same file. A boundary is required
/// so that `…/05-ai-integration.md.bak` does not match the document it was
/// copied from.
fn names(argument: &str, target: &Examined) -> bool {
    names_path(argument, &target.path)
}

/// The same rule, over a path rather than over an examined target.
///
/// [`crate::read_set`] compares a recorded argument against the path of a
/// census row, which is the same comparison against a different source of the
/// path. It reads this function rather than a copy of it, because two copies of
/// a boundary rule are two places for the boundary to go missing.
pub(crate) fn names_path(argument: &str, path: &str) -> bool {
    let argument = argument.replace('\\', "/");
    argument == path
        || (argument.len() > path.len()
            && argument.ends_with(path)
            && argument[..argument.len() - path.len()].ends_with('/'))
}

fn bash_names(_argument: &str, _target: &Examined) -> bool {
    false
}

/// Every recorded call of the session, and `None` where no event recorded any.
fn recorded_calls(session: &[&Event]) -> Option<usize> {
    let mut total = None;
    for event in session {
        if let Some(calls) = &event.calls {
            *total.get_or_insert(0) += calls.len();
        }
    }
    total
}

/// Every recorded artifact of the session, and `None` where no event recorded
/// the key.
fn recorded_produced<'a>(session: &[&'a Event]) -> Option<Vec<(usize, &'a Produced)>> {
    let mut total = None;
    for event in session {
        if let Some(produced) = &event.produced {
            let into: &mut Vec<(usize, &Produced)> = total.get_or_insert_with(Vec::new);
            into.extend(produced.iter().map(|artifact| (event.at, artifact)));
        }
    }
    total
}

fn opened(selected: &Selected, session: &[&Event]) -> Verdict {
    let Some(calls) = recorded_calls(session) else {
        return Verdict::Refused(Refusal::Unrecorded { what: "calls" });
    };
    match read(&selected.examines, session) {
        Some(found) => Verdict::Satisfied(Witness::Read {
            event: found.event,
            call: found.call,
            tool: found.made.tool.clone(),
            argument: found.made.argument.clone(),
        }),
        None => Verdict::NotSatisfied(Miss::NeverRead {
            calls,
            over: selected.examines.len(),
        }),
    }
}

fn not_opened(selected: &Selected, session: &[&Event]) -> Verdict {
    let Some(calls) = recorded_calls(session) else {
        return Verdict::Refused(Refusal::Unrecorded { what: "calls" });
    };
    if calls == 0 {
        return Verdict::Refused(Refusal::NothingObserved);
    }
    match read(&selected.examines, session) {
        Some(found) => Verdict::NotSatisfied(Miss::Read {
            event: found.event,
            call: found.call,
            argument: found.made.argument.clone(),
        }),
        None => Verdict::Satisfied(Witness::NoneOf {
            calls,
            over: selected.examines.len(),
        }),
    }
}

fn cited(selected: &Selected, session: &[&Event]) -> Verdict {
    let wanted: Vec<&String> = selected
        .examines
        .iter()
        .filter_map(|target| target.id.as_ref())
        .collect();
    if wanted.is_empty() {
        return Verdict::Refused(Refusal::NothingCitable {
            over: selected.examines.len(),
        });
    }
    let Some(artifacts) = recorded_produced(session) else {
        return Verdict::Refused(Refusal::Unrecorded { what: "produced" });
    };
    for (at, artifact) in &artifacts {
        for identifier in &artifact.cites {
            if wanted.contains(&identifier) {
                return Verdict::Satisfied(Witness::Cites {
                    event: *at,
                    artifact: artifact.path.clone(),
                    identifier: identifier.clone(),
                });
            }
        }
    }
    Verdict::NotSatisfied(Miss::NeverCited {
        artifacts: artifacts.len(),
        over: wanted.len(),
    })
}

fn answered(selected: &Selected, session: &[&Event]) -> Verdict {
    // The plan refuses each of these, and a selection assembled by hand is one
    // the plan never saw.
    let proper = !selected.expected.is_empty()
        && selected
            .expected
            .iter()
            .all(|value| selected.answers.contains(value))
        && selected.expected.len() < selected.answers.len();
    if selected.answers.is_empty() || !proper {
        return Verdict::Refused(Refusal::AnswersUndeclared);
    }
    // The last recorded answer of the session, because the expectation is over
    // the *final* answer and an earlier event is an earlier turn of one run.
    let mut last = None;
    for event in session {
        match &event.answer {
            Answer::Unrecorded => {}
            other => last = Some((event.at, other.clone())),
        }
    }
    match last {
        None => Verdict::Refused(Refusal::Unrecorded { what: "answer" }),
        Some((_, Answer::Absent)) => Verdict::NotSatisfied(Miss::NoAnswerGiven),
        Some((at, Answer::Value(value))) if selected.expected.contains(&value) => {
            Verdict::Satisfied(Witness::Answered { event: at, value })
        }
        Some((at, Answer::Value(value))) if selected.answers.contains(&value) => {
            Verdict::NotSatisfied(Miss::Wrong {
                event: at,
                value,
                expected: selected.expected.clone(),
            })
        }
        Some((at, Answer::Value(value))) => {
            Verdict::NotSatisfied(Miss::Outside { event: at, value })
        }
        Some((_, Answer::Unrecorded)) => unreachable!("the loop skips it"),
    }
}

fn patched(selected: &Selected, session: &[&Event]) -> Verdict {
    let Some(oracle) = &selected.oracle else {
        return Verdict::Refused(Refusal::OracleUndeclared);
    };
    let Some(artifacts) = recorded_produced(session) else {
        return Verdict::Refused(Refusal::Unrecorded { what: "produced" });
    };
    if artifacts.is_empty() {
        return Verdict::NotSatisfied(Miss::NothingProduced);
    }
    for (at, artifact) in &artifacts {
        let Some(findings) = &artifact.findings else {
            return Verdict::Refused(Refusal::NotChecked {
                artifact: artifact.path.clone(),
            });
        };
        if !findings.contains(oracle) {
            return Verdict::Satisfied(Witness::Clean {
                event: *at,
                artifact: artifact.path.clone(),
                oracle: oracle.clone(),
            });
        }
    }
    Verdict::NotSatisfied(Miss::OracleReported {
        artifacts: artifacts.len(),
        oracle: oracle.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 29 of 58 against 43 of 58: the Newcombe interval of the difference is
    /// narrower than the gap the two Wilson intervals leave, which is why the
    /// overlap reading has less power than the count was taken for (#980).
    #[test]
    fn the_difference_interval_is_the_newcombe_hybrid_score_interval() {
        let wilson = |satisfied: f64, n: f64| {
            let p = satisfied / n;
            let denominator = 1.0 + Z * Z / n;
            let center = (p + Z * Z / (2.0 * n)) / denominator;
            let half = Z * (p * (1.0 - p) / n + Z * Z / (4.0 * n * n)).sqrt() / denominator;
            Interval {
                point: p,
                low: center - half,
                high: center + half,
            }
        };
        let (present, absent) = (wilson(43.0, 58.0), wilson(29.0, 58.0));
        let difference = present.minus(&absent);
        assert!((difference.point - 14.0 / 58.0).abs() < 1e-9);
        assert!(
            difference.low > 0.0,
            "a 24-point difference at 58 per arm excludes zero: {difference:?}"
        );
        assert!(
            present.low < absent.high,
            "the two Wilson intervals overlap, so the overlap reading misses it"
        );
        let reversed = absent.minus(&present);
        assert!((reversed.low + difference.high).abs() < 1e-9);
        assert!((reversed.high + difference.low).abs() < 1e-9);
    }

    /// The owner's ruling on #980: a session that did nothing is data, and
    /// every refusal the recorder or the declaration caused is a defect.
    #[test]
    fn only_a_session_that_did_nothing_is_a_session_refusal() {
        assert!(Refusal::NothingObserved.is_session());
        for defect in [
            Refusal::NotRun,
            Refusal::OracleUndeclared,
            Refusal::AnswersUndeclared,
            Refusal::NothingCitable { over: 1 },
            Refusal::Unrecorded { what: "calls" },
            Refusal::NotChecked {
                artifact: "docs/x.md".to_string(),
            },
        ] {
            assert!(!defect.is_session(), "{defect:?}");
        }
    }

    fn target(id: Option<&str>, path: &str) -> Examined {
        Examined {
            id: id.map(str::to_string),
            path: path.to_string(),
        }
    }

    #[test]
    fn a_tool_call_names_a_document_by_equality_or_at_a_path_boundary() {
        let want = target(Some("HW-SPEC-05"), "docs/spec/05-ai-integration.md");
        assert!(names("docs/spec/05-ai-integration.md", &want));
        assert!(names("/home/a/repo/docs/spec/05-ai-integration.md", &want));
        assert!(names("C:\\repo\\docs\\spec\\05-ai-integration.md", &want));
        assert!(
            !names("docs/spec/05-ai-integration.md.bak", &want),
            "a copy is not the document"
        );
        assert!(
            !names("otherdocs/spec/05-ai-integration.md", &want),
            "the boundary is what stops a suffix from matching a different tree"
        );
    }

    fn call(tool: &str, argument: &str) -> crate::intake::Call {
        crate::intake::Call {
            tool: tool.into(),
            argument: argument.into(),
            result: "sha256:0".into(),
        }
    }

    fn over_the_probe_document(expectation: Expectation) -> Selected {
        Selected {
            expectation,
            examines: vec![target(Some("PROBE-FIX-one"), "docs/probes/one.md")],
            ..opened_over_one_document()
        }
    }

    /// A Bash read is a read (#1384). The recorder writes a Bash call's
    /// argument as the JSON of its input, so the whole argument never equals a
    /// path. The present arm reads through Bash about twice as often as the
    /// absent arm, so a grader blind to Bash misses on one arm more than the
    /// other.
    #[test]
    fn a_bash_command_that_names_the_document_is_a_read_of_it() {
        let sed = event(
            1,
            "sed",
            Some(vec![call(
                "Bash",
                r#"{"command":"sed -n 1,80p docs/probes/one.md","description":"x"}"#,
            )]),
        );
        assert!(
            matches!(
                verdict(&over_the_probe_document(Expectation::Opened), &[&sed]),
                Verdict::Satisfied(Witness::Read { call: 1, .. })
            ),
            "a Bash `sed` of the document opened it"
        );
        assert!(
            matches!(
                verdict(&over_the_probe_document(Expectation::NotOpened), &[&sed]),
                Verdict::NotSatisfied(Miss::Read { call: 1, .. })
            ),
            "a Bash `sed` of the document is the read `not_opened` forbids"
        );

        let copy = event(
            2,
            "copy",
            Some(vec![call(
                "Bash",
                r#"{"command":"cat docs/probes/one.md.bak","description":"x"}"#,
            )]),
        );
        assert_eq!(
            verdict(&over_the_probe_document(Expectation::Opened), &[&copy]),
            Verdict::NotSatisfied(Miss::NeverRead { calls: 1, over: 1 }),
            "the boundary rule holds per word, so a copy is not the document"
        );
    }

    /// The words of a command are the words a shell would pass: quotes are
    /// taken off, and an operator ends a word whether or not a space follows
    /// it.
    #[test]
    fn a_command_is_split_into_the_words_a_shell_would_pass() {
        let want = target(None, "docs/probes/one.md");
        for command in [
            r#"{"command":"cat 'docs/probes/one.md'"}"#,
            r#"{"command":"head -n 5 \"docs/probes/one.md\""}"#,
            r#"{"command":"wc -l <docs/probes/one.md"}"#,
            r#"{"command":"grep -c x docs/a.md;cat docs/probes/one.md|head"}"#,
            r#"{"command":"cd /repo && sed -n 1,9p /repo/docs/probes/one.md 2>/dev/null"}"#,
            r#"{"command":"cat docs/probes/one\\.md"}"#,
        ] {
            assert!(bash_names(command, &want), "{command}");
        }
        for command in [
            r#"{"command":"cat docs/probes/one.md.bak"}"#,
            r#"{"command":"cat otherdocs/probes/one.md"}"#,
            r#"{"command":"echo 'docs/probes/one.md is here'"}"#,
            r#"{"description":"docs/probes/one.md"}"#,
            "docs/probes/one.md is not JSON",
        ] {
            assert!(!bash_names(command, &want), "{command}");
        }
    }

    /// Only a Bash call's argument is read as a command. Another tool whose
    /// input happens to carry a `command` key has not run it.
    #[test]
    fn only_a_bash_call_is_read_as_a_command() {
        let other = event(
            1,
            "other",
            Some(vec![call(
                "Monitor",
                r#"{"command":"cat docs/probes/one.md"}"#,
            )]),
        );
        assert_eq!(
            verdict(&over_the_probe_document(Expectation::Opened), &[&other]),
            Verdict::NotSatisfied(Miss::NeverRead { calls: 1, over: 1 }),
        );
    }

    fn event(at: usize, session: &str, calls: Option<Vec<crate::intake::Call>>) -> Event {
        Event {
            at,
            probe: "PROBE-FIX-one".into(),
            session: session.into(),
            calls,
            produced: None,
            answer: Answer::Unrecorded,
        }
    }

    fn opened_over_one_document() -> Selected {
        Selected {
            path: "probes/one.md".into(),
            id: "PROBE-FIX-one".into(),
            category: Category::Discovery,
            expectation: Expectation::Opened,
            examines: vec![target(Some("PROBE-FIX-two"), "probes/two.md")],
            oracle: None,
            answers: Vec::new(),
            expected: Vec::new(),
        }
    }

    /// `calls: []` reaches a verdict and a missing `calls` key reaches none.
    ///
    /// Named, and asserted on both sides, because an invariant that lives only
    /// inside a `match` arm is an invariant a test suite cannot be asked about.
    /// The two sessions below differ in exactly one key, so nothing else can be
    /// the cause of the difference.
    ///
    /// The second half is the part that matters more than the verdicts. A
    /// session that reached no verdict leaves the denominator of the rate: it
    /// joins neither the numerator nor the failures, and it is printed beside
    /// the rate instead. A grader that read a missing key as an empty list
    /// would report two of two here, and the direction of that error is always
    /// the flattering one.
    #[test]
    fn an_omitted_calls_key_leaves_the_denominator_and_an_empty_list_joins_it() {
        let selected = opened_over_one_document();
        let watched = event(1, "watched-and-saw-nothing", Some(Vec::new()));
        let unwatched = event(2, "nothing-watched", None);

        assert_eq!(
            verdict(&selected, &[&watched]),
            Verdict::NotSatisfied(Miss::NeverRead { calls: 0, over: 1 }),
            "`calls: []` says the recorder watched, which is a fact a predicate reads"
        );
        assert_eq!(
            verdict(&selected, &[&unwatched]),
            Verdict::Refused(Refusal::Unrecorded { what: "calls" }),
            "an event with no `calls` key says nothing watched, which answers nothing"
        );

        let record = Record {
            identity: None,
            read: 2,
            probes: vec!["PROBE-FIX-one".into()],
            sessions: 2,
            calls: 0,
            events: vec![watched, unwatched],
            rejected: Vec::new(),
            refusal: None,
            declared: 1,
            lock_moved: None,
        };
        let results = Results::over(&record, std::slice::from_ref(&selected));
        assert_eq!(
            results.graded(),
            1,
            "the refused session left the denominator"
        );
        assert_eq!(results.satisfied(), 0);
        assert_eq!(results.refused(), 1);
        assert_eq!(
            results.rate().expect("one session was graded").point,
            0.0,
            "a refused session that joined the numerator would round this up"
        );
    }

    /// The interval is the reason a result is readable over quarters, and its
    /// one hard property is that it stays inside zero and one where a normal
    /// interval does not.
    #[test]
    fn the_interval_stays_inside_the_unit_at_the_counts_a_regression_tier_runs() {
        for (satisfied, graded) in [(0, 1), (1, 1), (2, 2), (1, 3), (58, 116)] {
            let mut results = Results {
                grader: VERSION,
                tier: None,
                arm: None,
                model: None,
                served_version: None,
                rows: Vec::new(),
                unusable: None,
            };
            results.rows.push(Row {
                probe: "PROBE-FIX-one".into(),
                category: Category::Discovery,
                expectation: Expectation::Answered,
                sessions: (0..graded)
                    .map(|index| Graded {
                        session: index.to_string(),
                        events: vec![index + 1],
                        verdict: match index < satisfied {
                            true => Verdict::Satisfied(Witness::Answered {
                                event: index + 1,
                                value: "yes".into(),
                            }),
                            false => Verdict::NotSatisfied(Miss::NoAnswerGiven),
                        },
                    })
                    .collect(),
            });
            let interval = results.rate().expect("something was graded");
            assert!(
                (0.0..=1.0).contains(&interval.low) && (0.0..=1.0).contains(&interval.high),
                "{satisfied} of {graded} left the unit: {interval:?}"
            );
            assert!(interval.low <= interval.point && interval.point <= interval.high);
        }
    }
}
