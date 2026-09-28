// SPDX-License-Identifier: Apache-2.0
//! What a run would be, what it would cost, and whether it happens.
//!
//! # The plan is the half of the run identity that exists before the run
//!
//! [Spec 5](../../../../docs/spec/05-ai-integration.md#a-run-produces-a-snapshot-and-a-document)
//! lists nine members of a run identity: the model with its served version, the
//! corpus tree hash, the taxonomy lock hash, the probe selection hash, the
//! rotation seed, the harness version, the arm, and the time. Five of them are
//! decided before anything is called, and this is where they are decided. The
//! model, the served version and the time belong to the run, and the recorder
//! writes them into the transcript.
//!
//! [`crate::intake`] holds a transcript to both halves: the five here have to be
//! the five the plan fixed, or the transcript records a run of something else.
//!
//! # The seed is the caller's, and a derived one would be worse
//!
//! Spec 5 asks for deterministic rotation, so the seed is part of the run
//! identity. It would be easy to derive one from the lock and the selection,
//! and that would be a defect: the rotation would then change whenever the
//! corpus changed, and a difference between two runs would carry a change in
//! the phrasing and a change in the corpus at once with no way to tell them
//! apart. So the seed is a number the caller states, it is recorded as stated,
//! and a run that repeats a seed repeats a selection.
//!
//! # A refusal stops the run and never the probe
//!
//! Every [`Refusal`] below ends the whole run. The alternative is a selection
//! that quietly dropped what it could not read, which reports a rate over a
//! denominator nobody declared — the failure `headwater conformance` already
//! refuses when a rule has no reading.

use crate::budget::{Budgets, Envelope};
use crate::{dollars, Arm, Category, Cents, Expectation, Tier};
use headwater_census::census::{Census, Outcome};
use headwater_check::paint::{paint, ColorMode, Role};
use headwater_graph::edges::Target;
use headwater_graph::{Config, Graph};

/// The kind a probe document is, and the relation that names what it examines.
pub const KIND: &str = "probe";
/// The relation whose targets an `opened`, `not_opened` or `cited` expectation
/// is a predicate over.
pub const EXAMINES: &str = "examines";

/// The section a probe declares its expectation in, which the `probe` kind
/// already requires.
///
/// # The second conditional facet did not become a facet
///
/// `answered` is satisfied by "one named value from a closed set that the probe
/// declares", and nothing held that set. It is the same shape as `oracle`: a
/// declaration that applies to one value of `expectation` and to no other, and
/// [HW-OBL-0123](../../../../docs/obligations/0123-a-facet-that-applies-to-one-value-of-another-facet-has-nowhere-to-say-so.md)
/// says the taxonomy language cannot express that. `oracle` paid the sentinel:
/// every probe declares one and four forms write `none`.
///
/// This one does not, because the set is a list and no facet of this language
/// holds a list. It goes in a fenced block under the section the kind already
/// requires, and the pairing is enforced in both directions below exactly as
/// the oracle's is. That is a route the record does not name: **a required
/// section carries a per-document closed set where a facet cannot**, and it
/// costs the four other forms nothing. Whether `oracle` should follow it is a
/// question for the record rather than for this crate.
pub const EXPECTATION_SECTION: &str = "Expectation";
/// The key that names the closed answer set inside it.
pub const ANSWERS: &str = "answers";
/// The key that names, inside the same block, the values of that set which
/// satisfy the probe.
///
/// The set is the domain a recorder extracts a value from, so a wrong word the
/// task offers is still recorded as that word and not as no answer. The
/// expected values are the answer key. Before #1229 the grader read the set as
/// both, and the one `answered` probe on the shelf declared every word its task
/// offered, so every in-domain answer passed.
pub const EXPECTED: &str = "expected";

/// What a probe writes in `oracle` when its expectation is not `patched`.
///
/// A sentinel, and the taxonomy has nowhere else to put it: a facet that no
/// kind requires is refused as unread, and the language cannot say "required
/// when `expectation` is `patched`". The value is enforced in both directions
/// below, so it is a declared absence rather than an empty field.
pub const NO_ORACLE: &str = "none";

/// One target of an `examines` edge, under both of the names a transcript uses.
///
/// # One name was not enough, and building the grader is what showed it
///
/// This type replaces the list of strings that #84 handed over, and the reason
/// is a defect that only the next component could see. Three expectation forms
/// are predicates over these targets, and **they do not read the same name**. A
/// session opens a *file*, so `opened` and `not_opened` compare a tool-call
/// argument against a path. A produced artifact cites an *identifier*, so
/// `cited` compares against the identifier. A list of identifiers alone makes
/// every `opened` verdict false, and a list of paths alone makes every `cited`
/// verdict false. Neither failure raises anything: the run is green, the rate
/// is a number, and it is wrong in one direction for one form.
///
/// So the target carries both, and `id` is `None` where the target has none. An
/// external anchor — a code path, which is the end that lets a probe measure
/// the reach of a skill file — is a path and nothing cites it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Examined {
    /// The identifier, where the target is a document of this corpus.
    pub id: Option<String>,
    /// The path a session would open: the document's path, or the string the
    /// anchor resolver normalized.
    pub path: String,
}

impl Examined {
    /// The name a reader of the plan and of a verdict sees.
    pub fn name(&self) -> String {
        match &self.id {
            Some(id) => format!("{id} at {}", self.path),
            None => self.path.clone(),
        }
    }
}

/// Why a document is in the read set of a run.
///
/// Both arms are declarations rather than observations. A probe document is in
/// the set because its prose is the task a session was given, and an examined
/// document is in it because a probe of the selection points a session at it.
/// What a session then opened is in the transcript and never in a plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Because {
    /// The document is a probe of the selection.
    Probe,
    /// A probe of the selection examines it.
    Examined,
}

impl Because {
    pub fn name(self) -> &'static str {
        match self {
            Because::Probe => "probe",
            Because::Examined => "examined",
        }
    }
}

/// One document the read set of this run covers.
///
/// # This is the narrowing that the corpus tree digest cannot make
///
/// The `tree` digest covers every classified document, so it moves on every
/// commit, and a result held to it is stale the moment anybody edits anything.
/// The read set covers the documents the selection points a session at and
/// nothing else. An edit elsewhere leaves it alone, and that is what makes an
/// answer over it worth reading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Read {
    pub path: String,
    /// The content digest the census carried for it, and `None` where the
    /// census carried none. An unhashed member is written into the digest the
    /// way the tree digest writes one, so the two constructions agree.
    pub digest: Option<String>,
    pub because: Because,
}

/// One probe, as the harness meets it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selected {
    pub path: String,
    pub id: String,
    pub category: Category,
    pub expectation: Expectation,
    /// What the `examines` edges of this probe name, sorted by path.
    pub examines: Vec<Examined>,
    /// The rule a `patched` expectation grades against, and `None` for every
    /// other form.
    pub oracle: Option<String>,
    /// The closed set an `answered` expectation is graded against, sorted, and
    /// empty for every other form. See [`EXPECTATION_SECTION`].
    pub answers: Vec<String>,
    /// The values of `answers` that satisfy an `answered` expectation, sorted,
    /// and empty for every other form. A proper subset of `answers`, which the
    /// plan enforces.
    pub expected: Vec<String>,
}

/// Why a run does not happen.
///
/// It is a value on the plan rather than an error, because the verb over a plan
/// exits zero whatever it holds. A refusal is a statement about a run that a
/// reader acts on, and never a status a gate branches on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// No document of the probe kind is classified in this corpus.
    NoProbes,
    /// The selection filters left nothing.
    SelectionEmpty { category: Option<Category> },
    /// The declaration names no envelope for the tier asked for.
    TierUndeclared(Tier),
    /// A campaign narrowed to one arm. It would estimate nothing while
    /// carrying the name a published claim cites.
    CampaignNarrowed,
    /// The caller narrowed to an arm the tier's envelope does not declare.
    ///
    /// The narrowing selects nothing. Restoring the tier's own arms beside it
    /// planned a run of the arm the caller did not ask for and printed it
    /// under `arms:`, which is a plan for a question nobody put.
    ArmNotDeclared {
        tier: Tier,
        arm: Arm,
        declared: Vec<Arm>,
    },
    /// A probe carries no identifier, so nothing can name its result.
    Unnameable { path: String },
    /// A probe's category or expectation is missing or outside its closed set.
    Undeclared {
        path: String,
        facet: &'static str,
        found: Option<String>,
    },
    /// A `patched` probe that declares the sentinel instead of a rule.
    OracleUnnamed { probe: String },
    /// A probe of another form that names a rule. Its expectation is not the
    /// oracle route, so the rule would be declared and never read.
    OracleNotUsed {
        probe: String,
        expectation: Expectation,
        rule: String,
    },
    /// A `patched` probe naming a rule this engine does not carry. The grader
    /// would have nothing to run, and a probe that grades against nothing
    /// returns a verdict about the harness.
    OracleUnknown { probe: String, rule: String },
    /// An `answered` probe that declares no closed set of answers. Every string
    /// would then be the expected one, and the rate would be the session count.
    AnswersUndeclared { probe: String },
    /// An `answered` probe that declares a closed set and no expected value in
    /// it. Nothing then says which answer is right.
    ExpectedUndeclared { probe: String },
    /// An `answered` probe whose expected values are not all in its closed
    /// set. A recorder extracts only a value of the set, so an expected value
    /// outside it can never be met.
    ExpectedOutsideAnswers { probe: String, outside: Vec<String> },
    /// An `answered` probe whose expected values are every value of its closed
    /// set. Every answer the recorder can extract then satisfies it, and the
    /// rate is the count of sessions that answered at all.
    ExpectedEveryAnswer { probe: String },
    /// A probe of another form that declares a closed set of answers. Its
    /// expectation reads no answer, so the set is declared and never read.
    AnswersNotUsed {
        probe: String,
        expectation: Expectation,
    },
    /// An `opened`, `not_opened` or `cited` probe that names no document.
    /// `opened` over an empty set is never satisfied and `not_opened` over one
    /// always is, so either would report the declaration rather than the
    /// corpus.
    ExpectationNamesNothing {
        probe: String,
        expectation: Expectation,
    },
    /// A probe whose predicate names a document that the tier's own absent
    /// arm removes.
    ///
    /// A session in that arm cannot open or cite a document it never had, so
    /// the rate over the pair would measure the ablation and not the
    /// documents. `answered` and `patched` probes are admitted whatever they
    /// examine, because their predicates read the session's output.
    AblatedExamined {
        probe: String,
        /// The examined path, relative to the corpus root.
        path: String,
        /// The ablation entry the path equals or sits under.
        entry: String,
    },
    /// A probe whose predicate names a document under the instrument, which
    /// every arm of every tier removes. No session of any arm could open it.
    InstrumentExamined {
        probe: String,
        path: String,
        entry: String,
    },
    /// The caller excluded a probe that is not in the selection the other
    /// filters left.
    ///
    /// An exclusion that removes nothing plans the same run as no exclusion,
    /// and a misspelled identifier would then read as a probe kept out of a
    /// run it was never in (#980).
    ExcludedUnknown { probe: String },
    /// The caller asked for more repetitions than the tier declares.
    ///
    /// The narrowing may only lower the count. Raising it is spending more,
    /// and the number that says how much moves when a person edits the
    /// declaration and not when a caller passes a flag.
    RepetitionsRaised { asked: u32, declared: u32 },
    /// The caller asked for no repetitions, which plans a run of nothing.
    RepetitionsZero,
    /// The projected cost is above the tier's ceiling. Spec 5: a run that does
    /// not happen is the cheaper error.
    OverBudget {
        sessions: u64,
        projected: Cents,
        budget: Cents,
    },
}

impl Refusal {
    /// Whether this refusal also stops a grade of a run that already happened.
    ///
    /// [`Plan::over`] returns from inside the loop that composes the selection,
    /// so most refusals leave a part of it behind: the probes read before the
    /// offending one, and none of the rest. A component that graded against
    /// that part would report a rate over a denominator no document declares,
    /// so every one of those refusals stops a grade.
    ///
    /// Four do not, and the four are the ones decided after every probe has
    /// been read. They are about the shape and the **cost** of a run rather
    /// than about what the probes say, and the selection beside them is whole.
    /// A grade of a recorded run spends nothing, so a ceiling the run would
    /// have exceeded, a tier that declares no envelope, and an arm the tier
    /// does not carry all change no verdict.
    ///
    /// This is an exhaustive match and not a `matches!`, so that a refusal
    /// added later has to answer the question rather than inherit an answer.
    pub fn stops_a_grade(&self) -> bool {
        match self {
            Refusal::TierUndeclared(_) | Refusal::CampaignNarrowed => false,
            Refusal::ArmNotDeclared { .. } => false,
            Refusal::OverBudget { .. } => false,
            // A grade composes its selection with no narrowing, so neither of
            // these is reachable from one. They are about what a caller asked
            // a run to be, like an arm the tier does not declare.
            Refusal::RepetitionsRaised { .. } | Refusal::RepetitionsZero => false,
            Refusal::ExcludedUnknown { .. } => true,
            // A policy about which probes a paired run may carry, decided
            // after the whole selection is read, like a narrowed campaign.
            Refusal::AblatedExamined { .. } => false,
            Refusal::InstrumentExamined { .. } => false,
            Refusal::NoProbes
            | Refusal::SelectionEmpty { .. }
            | Refusal::Unnameable { .. }
            | Refusal::Undeclared { .. }
            | Refusal::OracleUnnamed { .. }
            | Refusal::OracleNotUsed { .. }
            | Refusal::OracleUnknown { .. }
            | Refusal::AnswersUndeclared { .. }
            | Refusal::ExpectedUndeclared { .. }
            | Refusal::ExpectedOutsideAnswers { .. }
            | Refusal::ExpectedEveryAnswer { .. }
            | Refusal::AnswersNotUsed { .. }
            | Refusal::ExpectationNamesNothing { .. } => true,
        }
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refusal::NoProbes => write!(
                f,
                "this corpus classifies no document of the `{KIND}` kind, so there is nothing to \
                 run and no claim it could measure"
            ),
            Refusal::SelectionEmpty { category } => match category {
                Some(category) => write!(
                    f,
                    "no probe of this corpus declares the `{}` category",
                    category.name()
                ),
                None => write!(f, "the selection is empty"),
            },
            Refusal::TierUndeclared(tier) => write!(
                f,
                "`{}` declares no envelope for the `{}` tier, so no ceiling exists to project \
                 against",
                crate::budget::PATH,
                tier.name()
            ),
            Refusal::CampaignNarrowed => write!(
                f,
                "a campaign narrowed to one arm estimates no difference, and the pair is what a \
                 published efficacy claim rests on"
            ),
            Refusal::ArmNotDeclared {
                tier,
                arm,
                declared,
            } => {
                let names: Vec<String> = declared
                    .iter()
                    .map(|arm| format!("`{}`", arm.name()))
                    .collect();
                let carries = match names.is_empty() {
                    true => "no arm at all".to_string(),
                    false => names.join(" and "),
                };
                write!(
                    f,
                    "the `{}` tier declares {carries}, and `--arm {}` narrows it to nothing. A \
                     plan for the arms the tier does carry would answer a question nobody asked",
                    tier.name(),
                    arm.name()
                )
            }
            Refusal::Unnameable { path } => write!(
                f,
                "the probe at {path} carries no identifier, so no result could name the probe it \
                 graded"
            ),
            Refusal::Undeclared { path, facet, found } => match found {
                Some(found) => write!(
                    f,
                    "the probe at {path} declares `{facet}: {found}`, which is outside the closed \
                     set the taxonomy fixes"
                ),
                None => write!(f, "the probe at {path} declares no `{facet}`"),
            },
            Refusal::OracleUnnamed { probe } => write!(
                f,
                "{probe} expects `patched` and declares `oracle: {NO_ORACLE}`. The `patched` form \
                 grades a produced patch against a named check, so a probe without one declares a \
                 predicate nothing can evaluate"
            ),
            Refusal::OracleNotUsed {
                probe,
                expectation,
                rule,
            } => write!(
                f,
                "{probe} expects `{}` and declares `oracle: {rule}`. Only `patched` grades against \
                 a check, so that rule is declared and never read. Write `oracle: {NO_ORACLE}`",
                expectation.name()
            ),
            Refusal::OracleUnknown { probe, rule } => write!(
                f,
                "{probe} grades against `{rule}`, which is not one of the {} rules this engine \
                 carries",
                headwater_check::RULES.len()
            ),
            Refusal::AnswersUndeclared { probe } => write!(
                f,
                "{probe} expects `answered` and declares no `{ANSWERS}` block under `## \
                 {EXPECTATION_SECTION}`. The form is satisfied by one value of a closed set the \
                 probe declares, so a probe without one is satisfied by every string a session \
                 returns"
            ),
            Refusal::ExpectedUndeclared { probe } => write!(
                f,
                "{probe} expects `answered` and declares no `{EXPECTED}` values beside its \
                 `{ANSWERS}`. The set is the domain a recorder reads, and nothing then says which \
                 value of it is right"
            ),
            Refusal::ExpectedOutsideAnswers { probe, outside } => write!(
                f,
                "{probe} expects {} outside its `{ANSWERS}`. A recorder extracts only a value of \
                 that set, so no session can meet it",
                outside
                    .iter()
                    .map(|value| format!("`{value}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Refusal::ExpectedEveryAnswer { probe } => write!(
                f,
                "{probe} expects every value of its `{ANSWERS}`, so every answer a recorder can \
                 extract satisfies it and the rate counts the sessions that answered at all. Name \
                 the right value in `{EXPECTED}`"
            ),
            Refusal::AnswersNotUsed { probe, expectation } => write!(
                f,
                "{probe} expects `{}` and declares an `{ANSWERS}` or `{EXPECTED}` block. Only \
                 `answered` reads the final answer, so that set is declared and never read",
                expectation.name()
            ),
            Refusal::ExpectationNamesNothing { probe, expectation } => write!(
                f,
                "{probe} expects `{}` and names no document with `{EXAMINES}`. That predicate over \
                 an empty set reports the declaration rather than the corpus",
                expectation.name()
            ),
            Refusal::AblatedExamined { probe, path, entry } => write!(
                f,
                "{probe} expects a predicate over `{path}`, and this tier's absent arm removes \
                 `{entry}`. A session cannot open or cite a document it never had, so the pair \
                 would measure the ablation. Narrow the selection to `answered` and `patched` \
                 probes, or run it at a tier whose ablation keeps the document"
            ),
            Refusal::InstrumentExamined { probe, path, entry } => write!(
                f,
                "{probe} expects a predicate over `{path}`, and every arm removes `{entry}`, the \
                 instrument, because a probe document states the answer it expects. No session \
                 could open or cite it"
            ),
            Refusal::ExcludedUnknown { probe } => write!(
                f,
                "`--exclude {probe}` names no probe of the selection, so the exclusion removes \
                 nothing and the run would be the one planned without it"
            ),
            Refusal::RepetitionsRaised { asked, declared } => write!(
                f,
                "`--repetitions {asked}` is above the {declared} the tier declares. The narrowing \
                 may only lower the count, because a higher one spends more than a person agreed \
                 to"
            ),
            Refusal::RepetitionsZero => write!(f, "`--repetitions 0` plans a run of no session"),
            Refusal::OverBudget {
                sessions,
                projected,
                budget,
            } => write!(
                f,
                "{sessions} sessions project {} against a declared ceiling of {}, so the run does \
                 not start",
                dollars(*projected),
                dollars(*budget)
            ),
        }
    }
}

/// A run that this engine has decided everything about except the calling.
#[derive(Clone, Debug)]
pub struct Plan {
    pub tier: Tier,
    pub arms: Vec<Arm>,
    pub repetitions: u32,
    /// The repetitions the tier declares, which `repetitions` equals unless
    /// the caller narrowed it for a pilot.
    pub declared_repetitions: u32,
    /// The probes the caller named out of the selection, in the order given.
    pub excluded: Vec<String>,
    /// The digest of the lock this plan was taken against.
    pub lock: String,
    /// The digest over every classified document of this corpus, by path and
    /// content. It is the corpus tree hash of the run identity, and it moves
    /// when any document moves, which is what makes a recorded result stale.
    pub tree: String,
    /// The digest over the identifiers selected, in order.
    pub selection: String,
    /// The digest over the read set of this run, by path and content.
    ///
    /// It is the third narrow member of the identity and it is the one that
    /// moves on a prose edit. The `selection` digest holds still when a probe's
    /// text changes, which is what makes it a statement about the population
    /// rather than about the corpus. This one moves, and only for a document
    /// that the selection points a session at.
    pub read_set: String,
    /// The documents the read-set digest covers, in path order.
    pub reads: Vec<Read>,
    /// Targets an `examines` edge names that are not documents of this corpus,
    /// in the text the resolver normalized.
    ///
    /// [Spec 12](../../../../docs/spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict)
    /// rules that "an external anchor is in no read set", because a read set is
    /// a list of corpus paths and an anchor names a target that is not one.
    /// They are listed here so that the exclusion has a location rather than
    /// being a silence.
    pub anchors: Vec<String>,
    /// The rotation seed, as the caller stated it.
    pub seed: u64,
    /// The version of this engine, which is the harness version.
    pub harness: &'static str,
    pub selected: Vec<Selected>,
    /// Classified documents in the whole corpus, which is what the tree digest
    /// covers.
    pub corpus: usize,
    pub sessions: u64,
    pub projected: Cents,
    pub budget: Cents,
    pub session_cost: Cents,
    pub refusal: Option<Refusal>,
}

/// What the caller narrowed the run to.
#[derive(Clone, Debug, Default)]
pub struct Narrowing {
    pub category: Option<Category>,
    pub arm: Option<Arm>,
    pub seed: u64,
    /// Probes to leave out of the selection, by identifier.
    ///
    /// A category is the coarsest selection a campaign can name, and one probe
    /// of the discovery category examines a path every campaign absent arm
    /// removes. So the machine half of Q16 had no selection that planned until
    /// a probe could be named out of it (#980). An excluded probe leaves the
    /// selection before its digest is taken, so the digest states the run that
    /// happens.
    pub exclude: Vec<String>,
    /// Fewer repetitions than the tier declares, for a pilot of the same
    /// selection and arms.
    ///
    /// It is not a member of the run identity. A pilot and the run it tests
    /// share every member the plan fixes, so what keeps a pilot out of a
    /// comparison is the `tree` digest, which moves when the pilot's own
    /// transcripts are committed.
    pub repetitions: Option<u32>,
}

/// The selection digest: the identifiers selected, in identifier order, one per
/// line.
///
/// A plan takes it, and a grade takes it again over the probes a transcript
/// names, to find the narrowed selection that transcript was planned over. Two
/// computations of one digest are one function, or they drift.
pub fn selection_digest(ids: &[&str]) -> String {
    headwater_hash::digest(ids.join("\n").as_bytes())
}

/// The corpus tree digest: every typed row's path and content digest, in census
/// order. A plan and a shadow-log line both carry it, so there is one function.
pub fn tree_digest(census: &Census) -> String {
    let mut tree = String::new();
    for row in &census.rows {
        if !matches!(row.outcome, Outcome::Typed { .. }) {
            continue;
        }
        tree.push_str(&row.path);
        tree.push('\t');
        tree.push_str(row.digest.as_deref().unwrap_or("-"));
        tree.push('\n');
    }
    headwater_hash::digest(tree.as_bytes())
}

impl Plan {
    /// Take a plan over a corpus.
    ///
    /// It never fails. A run that does not happen produces a plan with a
    /// [`Refusal`] on it and the numbers that led to it, because the reason a
    /// run was refused is the thing the caller needs.
    pub fn over(
        census: &Census,
        graph: &Graph,
        config: &Config,
        budgets: &Budgets,
        lock: &str,
        tier: Tier,
        narrowing: &Narrowing,
    ) -> Plan {
        let mut plan = Plan {
            tier,
            arms: Vec::new(),
            repetitions: 0,
            declared_repetitions: 0,
            excluded: narrowing.exclude.clone(),
            lock: lock.to_string(),
            tree: String::new(),
            selection: String::new(),
            read_set: String::new(),
            reads: Vec::new(),
            anchors: Vec::new(),
            seed: narrowing.seed,
            harness: headwater_resolve_version(),
            selected: Vec::new(),
            corpus: 0,
            sessions: 0,
            projected: 0,
            budget: 0,
            session_cost: 0,
            refusal: None,
        };

        let mut probes = Vec::new();
        for row in &census.rows {
            let Outcome::Typed { kind, .. } = &row.outcome else {
                continue;
            };
            plan.corpus += 1;
            if kind == KIND {
                probes.push(row);
            }
        }
        plan.tree = tree_digest(census);

        if probes.is_empty() {
            plan.refusal = Some(Refusal::NoProbes);
            return plan;
        }

        for row in probes {
            let facets = row.document.as_ref().map(|document| &document.facets);
            let read = |key: &str| -> Option<String> {
                facets?
                    .get(key)?
                    .value
                    .as_scalar()
                    .map(|scalar| scalar.text.clone())
            };

            let Some(id) = read(&config.identifier_facet) else {
                plan.refusal = Some(Refusal::Unnameable {
                    path: row.path.clone(),
                });
                return plan;
            };
            let raw_category = read("probe_category");
            let Some(category) = raw_category.as_deref().and_then(Category::read) else {
                plan.refusal = Some(Refusal::Undeclared {
                    path: row.path.clone(),
                    facet: "probe_category",
                    found: raw_category,
                });
                return plan;
            };
            let raw_expectation = read("expectation");
            let Some(expectation) = raw_expectation.as_deref().and_then(Expectation::read) else {
                plan.refusal = Some(Refusal::Undeclared {
                    path: row.path.clone(),
                    facet: "expectation",
                    found: raw_expectation,
                });
                return plan;
            };

            let mut examines: Vec<Examined> = graph
                .edges
                .iter()
                .filter(|edge| edge.source.path == row.path && edge.declared == EXAMINES)
                // Both names, because the three predicates over these targets
                // read two different ones. See [`Examined`].
                .filter_map(|edge| match &edge.target {
                    Target::Document { id, path, .. } => Some(Examined {
                        id: Some(id.clone()),
                        path: path.clone(),
                    }),
                    Target::Anchor { normalized, .. } => Some(Examined {
                        id: None,
                        path: normalized.clone(),
                    }),
                    Target::Withheld { .. } => None,
                    _ => None,
                })
                .collect();
            examines.sort_by(|a, b| a.path.cmp(&b.path));
            examines.dedup();

            let declared = read("oracle").unwrap_or_else(|| NO_ORACLE.to_string());
            let oracle = match declared.as_str() {
                NO_ORACLE => None,
                rule => Some(rule.to_string()),
            };
            match (expectation, &oracle) {
                (Expectation::Patched, None) => {
                    plan.refusal = Some(Refusal::OracleUnnamed { probe: id });
                    return plan;
                }
                (Expectation::Patched, Some(rule))
                    if !headwater_check::RULES.contains(&rule.as_str()) =>
                {
                    plan.refusal = Some(Refusal::OracleUnknown {
                        probe: id,
                        rule: rule.clone(),
                    });
                    return plan;
                }
                (other, Some(rule)) if other != Expectation::Patched => {
                    plan.refusal = Some(Refusal::OracleNotUsed {
                        probe: id,
                        expectation: other,
                        rule: rule.clone(),
                    });
                    return plan;
                }
                _ => {}
            }
            if expectation.names_documents() && examines.is_empty() {
                plan.refusal = Some(Refusal::ExpectationNamesNothing {
                    probe: id,
                    expectation,
                });
                return plan;
            }

            let declared = |key: &str| {
                let mut values = row
                    .document
                    .as_deref()
                    .map(|document| declared_answers(document, key))
                    .unwrap_or_default();
                values.sort();
                values.dedup();
                values
            };
            let answers = declared(ANSWERS);
            let expected = declared(EXPECTED);
            match (expectation, answers.is_empty() && expected.is_empty()) {
                (Expectation::Answered, _) if answers.is_empty() => {
                    plan.refusal = Some(Refusal::AnswersUndeclared { probe: id });
                    return plan;
                }
                (Expectation::Answered, _) if expected.is_empty() => {
                    plan.refusal = Some(Refusal::ExpectedUndeclared { probe: id });
                    return plan;
                }
                (Expectation::Answered, _)
                    if expected.iter().any(|value| !answers.contains(value)) =>
                {
                    plan.refusal = Some(Refusal::ExpectedOutsideAnswers {
                        probe: id,
                        outside: expected
                            .iter()
                            .filter(|value| !answers.contains(value))
                            .cloned()
                            .collect(),
                    });
                    return plan;
                }
                (Expectation::Answered, _) if expected == answers => {
                    plan.refusal = Some(Refusal::ExpectedEveryAnswer { probe: id });
                    return plan;
                }
                (other, false) if other != Expectation::Answered => {
                    plan.refusal = Some(Refusal::AnswersNotUsed {
                        probe: id,
                        expectation: other,
                    });
                    return plan;
                }
                _ => {}
            }

            plan.selected.push(Selected {
                path: row.path.clone(),
                id,
                category,
                expectation,
                examines,
                oracle,
                answers,
                expected,
            });
        }

        if let Some(category) = narrowing.category {
            plan.selected
                .retain(|selected| selected.category == category);
            if plan.selected.is_empty() {
                plan.refusal = Some(Refusal::SelectionEmpty {
                    category: Some(category),
                });
                return plan;
            }
        }
        for excluded in &narrowing.exclude {
            let before = plan.selected.len();
            plan.selected.retain(|selected| &selected.id != excluded);
            if plan.selected.len() == before {
                plan.refusal = Some(Refusal::ExcludedUnknown {
                    probe: excluded.clone(),
                });
                return plan;
            }
        }
        if plan.selected.is_empty() {
            plan.refusal = Some(Refusal::SelectionEmpty {
                category: narrowing.category,
            });
            return plan;
        }
        plan.selected.sort_by(|a, b| a.id.cmp(&b.id));

        let names: Vec<&str> = plan
            .selected
            .iter()
            .map(|selected| selected.id.as_str())
            .collect();
        plan.selection = selection_digest(&names);

        let composed = read_set_over(&plan.selected, census);
        plan.read_set = composed.digest;
        plan.reads = composed.reads;
        plan.anchors = composed.anchors;

        let Some(envelope) = budgets.of(tier) else {
            plan.refusal = Some(Refusal::TierUndeclared(tier));
            return plan;
        };
        plan.repetitions = envelope.repetitions;
        plan.declared_repetitions = envelope.repetitions;
        match narrowing.repetitions {
            None => {}
            Some(0) => {
                plan.refusal = Some(Refusal::RepetitionsZero);
                return plan;
            }
            Some(asked) if asked > envelope.repetitions => {
                plan.refusal = Some(Refusal::RepetitionsRaised {
                    asked,
                    declared: envelope.repetitions,
                });
                return plan;
            }
            Some(asked) => plan.repetitions = asked,
        }
        plan.budget = envelope.budget;
        plan.session_cost = envelope.session_cost;
        plan.arms = arms(envelope, narrowing.arm);
        if plan.arms.len() < envelope.arms.len() && tier.pairs_arms() {
            plan.refusal = Some(Refusal::CampaignNarrowed);
            return plan;
        }
        if plan.arms.is_empty() {
            // A narrowing that selected nothing is refused rather than undone.
            // Restoring the tier's own arms here is what printed `arms:
            // [present]` to a caller who asked for `absent`, byte for byte the
            // plan they would have got by asking for nothing.
            if let Some(arm) = narrowing.arm {
                plan.refusal = Some(Refusal::ArmNotDeclared {
                    tier,
                    arm,
                    declared: envelope.arms.clone(),
                });
                return plan;
            }
            // The caller narrowed nothing, so the emptiness is the envelope's
            // own. `arms` hands its list back unchanged on this path, which
            // makes the restore a no-op today and the guard that keeps it one
            // if that ever stops being true.
            plan.arms = envelope.arms.clone();
        }

        // Before the ceiling: a run over budget could be afforded later, and a
        // run whose absent arm removes what its probes read cannot measure the
        // claim at any price, so this is the reason a reader is given.
        if let Some(Refusal::AblatedExamined { probe, path, entry }) =
            ablated_examined(&plan.selected, &budgets.instrument)
        {
            plan.refusal = Some(Refusal::InstrumentExamined { probe, path, entry });
            return plan;
        }
        if plan.arms.contains(&Arm::Absent) {
            if let Some(refusal) = ablated_examined(&plan.selected, &envelope.ablation) {
                plan.refusal = Some(refusal);
                return plan;
            }
        }

        plan.sessions =
            plan.selected.len() as u64 * plan.arms.len() as u64 * u64::from(plan.repetitions);
        plan.projected = plan.sessions * plan.session_cost;
        if plan.projected > plan.budget {
            plan.refusal = Some(Refusal::OverBudget {
                sessions: plan.sessions,
                projected: plan.projected,
                budget: plan.budget,
            });
        }
        plan
    }

    /// Whether the run this plan describes may start.
    pub fn runs(&self) -> bool {
        self.refusal.is_none()
    }

    /// The selection anything may grade a recorded run against, or the refusal
    /// that stops one.
    ///
    /// This is the only route to a selection for grading, and it exists because
    /// two components read the wrong value out of a plan before it did.
    /// [`Plan::over`] returns from inside the loop that composes the selection,
    /// so a refusal usually leaves a **part** of one behind: the probes read
    /// before the offending one, and none of the rest. A caller that tested
    /// `selected.is_empty()` therefore let through every refusal that stopped
    /// late, and graded over a denominator that no document declares and that
    /// moves with the order the paths sort in.
    ///
    /// [`Refusal::stops_a_grade`] is the rule and this is where every grading
    /// caller reads it. `selected` stays public because `headwater probe plan`
    /// reports the selection a plan composed, refusal and all, which is a
    /// different question and the right answer to it.
    pub fn gradable(&self) -> Result<&[Selected], &Refusal> {
        match self
            .refusal
            .as_ref()
            .filter(|refusal| refusal.stops_a_grade())
        {
            Some(refusal) => Err(refusal),
            None => Ok(&self.selected),
        }
    }

    /// The briefing, for the recorder rather than for a model.
    ///
    /// `mode` is a parameter and never a read, so this crate holds no opinion
    /// about whether anything is a terminal. The one human call site in
    /// `headwater_cli` states `stdout_color()` and every test states
    /// [`ColorMode::Plain`]. Nothing here folds, so every paint is applied
    /// where the token is written.
    ///
    /// One format, and it is prose, for the reason the sweep's plan is prose.
    /// The difference is the reader: a sweep briefs a model and this briefs the
    /// process that drives one. Nothing here is a prompt, because a prompt that
    /// the engine wrote would put this engine's phrasing inside the thing under
    /// test.
    pub fn render(&self, mode: ColorMode) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(
            out,
            "A {} probe run over {} of the {} classified documents of this corpus.",
            self.tier.name(),
            self.selected.len(),
            self.corpus
        );
        let _ = writeln!(out);

        let _ = writeln!(
            out,
            "{}",
            paint(Role::Heading, "## The run identity this plan fixes", mode)
        );
        let _ = writeln!(out);
        let _ = writeln!(out, "```yaml");
        let _ = writeln!(out, "lock: {}", self.lock);
        let _ = writeln!(out, "tree: {}", self.tree);
        let _ = writeln!(out, "selection: {}", self.selection);
        let _ = writeln!(out, "read_set: {}", self.read_set);
        let _ = writeln!(out, "seed: {}", self.seed);
        let _ = writeln!(out, "harness: {}", self.harness);
        let _ = writeln!(out, "tier: {}", self.tier.name());
        let _ = writeln!(
            out,
            "arms: [{}]",
            self.arms
                .iter()
                .map(|arm| arm.name())
                .collect::<Vec<_>>()
                .join(", ")
        );
        let _ = writeln!(out, "```");
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "The model, its served version, the wall-clock time and the realized cost belong to \
             the run. The recorder writes those four into the transcript, and `headwater probe \
             record` holds the rest of the identity to the six above."
        );
        let _ = writeln!(out);

        // The cost section is printed only where a cost was computed. A run
        // refused before the envelope was read has no projection, and a table
        // of zeros beside a refusal reads as a run that costs nothing rather
        // than as a run that was never priced.
        if self.budget > 0 {
            let _ = writeln!(out, "{}", paint(Role::Heading, "## The cost", mode));
            let _ = writeln!(out);
            let _ = writeln!(
                out,
                "{} probes x {} arms x {} repetitions = {} sessions, at a declared {} each: {} \
                 against a ceiling of {}.",
                self.selected.len(),
                self.arms.len(),
                self.repetitions,
                self.sessions,
                dollars(self.session_cost),
                dollars(self.projected),
                dollars(self.budget)
            );
            let _ = writeln!(
                out,
                "The unit cost is a number a person declared in `{}`. No run has happened, so no \
                 realized cost has been recorded against it and this is arithmetic rather than a \
                 forecast.",
                crate::budget::PATH
            );
            if self.repetitions < self.declared_repetitions {
                let _ = writeln!(
                    out,
                    "The caller narrowed the repetitions to {} of the {} the tier declares. That \
                     is a pilot of this run and not the run, and no comparison pools the two.",
                    self.repetitions, self.declared_repetitions
                );
            }
            let _ = writeln!(out);
        }

        if let Some(refusal) = &self.refusal {
            let _ = writeln!(
                out,
                "{}",
                paint(Role::Heading, "## This run does not start", mode)
            );
            let _ = writeln!(out);
            let _ = writeln!(out, "{refusal}");
            return out;
        }

        let _ = writeln!(out, "{}", paint(Role::Heading, "## The selection", mode));
        let _ = writeln!(out);
        if !self.excluded.is_empty() {
            let _ = writeln!(
                out,
                "The caller excluded {}: {}.",
                crate::plural(self.excluded.len(), "probe"),
                self.excluded.join(", ")
            );
            let _ = writeln!(out);
        }
        for selected in &self.selected {
            let _ = writeln!(
                out,
                "- {} ({})",
                selected.id,
                paint(Role::Path, &selected.path, mode)
            );
            let _ = writeln!(out, "    category: {}", selected.category.name());
            let _ = writeln!(out, "    expects: {}", selected.expectation.name());
            if !selected.examines.is_empty() {
                let _ = writeln!(
                    out,
                    "    examines: {}",
                    selected
                        .examines
                        .iter()
                        .map(Examined::name)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            if let Some(oracle) = &selected.oracle {
                let _ = writeln!(out, "    oracle: {oracle}");
            }
            if !selected.answers.is_empty() {
                let _ = writeln!(out, "    answers: {}", selected.answers.join(", "));
            }
            if !selected.expected.is_empty() {
                let _ = writeln!(out, "    expected: {}", selected.expected.join(", "));
            }
        }
        let _ = writeln!(out);

        let _ = writeln!(out, "{}", paint(Role::Heading, "## The read set", mode));
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "{} the `read_set` digest above covers, which is every probe of the selection and \
             every document one of them examines:",
            crate::plural(self.reads.len(), "document")
        );
        let _ = writeln!(out);
        for read in &self.reads {
            let _ = writeln!(
                out,
                "- {} ({}) {}",
                paint(Role::Path, &read.path, mode),
                read.because.name(),
                read.digest.as_deref().unwrap_or("-")
            );
        }
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "A recorded result is stale when a document of this list moves, and it is not stale \
             when any other document of this corpus moves. That is the whole of what the digest \
             buys over the `tree` digest beside it, which moves on every commit."
        );
        if !self.anchors.is_empty() {
            let _ = writeln!(out);
            let _ = writeln!(
                out,
                "{} an `examines` edge names is outside this corpus, so it is in no read set and \
                 no comparison over one decides anything about it: {}.",
                crate::plural(self.anchors.len(), "target"),
                self.anchors.join(", ")
            );
        }
        let _ = writeln!(out);

        let _ = writeln!(
            out,
            "{}",
            paint(Role::Heading, "## What the recorder writes back", mode)
        );
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "One `probe_transcript` document on `docs/probe-runs/`, then `headwater probe record \
             <path>`. The transcript is the ordered tool-call events of each session, and it holds \
             no model prose: the intake refuses a key outside the closed set, which is how the \
             omission is enforced rather than asked for."
        );
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "**A transcript that an agent wrote about its own session is a self-report, and spec 5 \
             refuses one.** The events have to be observed from outside the session by the process \
             that drove it. That process reaches the network and it is not in this repository, so \
             no verb here writes a transcript and there is no flag that makes one."
        );
        out
    }
}

/// The read set of a selection over a census, and the digest a plan fixes
/// for it.
///
/// [`Plan::over`] takes it over the selection it composes. The intake takes
/// it again over the part of the selection a transcript was planned over,
/// on the tree in front of it, and that is what lets a transcript whose lock
/// moved stay gradable where its read set did not move (#1292).
pub fn read_set_over(selected: &[Selected], census: &Census) -> ReadSet {
    // The digest of every document the walk read bytes of, by path, which
    // is what a read-set member is looked up in. It comes from the same
    // walk the tree digest comes from, because two passes over one corpus
    // can disagree and a read set that disagreed with the tree would report
    // a state that no plan fixed.
    //
    // # Two populations, and this loop used to treat them as one
    //
    // The lookup is over every row that carries a digest. The corpus count
    // and the tree digest of [`Plan::over`] are over the typed rows alone. Those are
    // different populations, and collecting the lookup inside the typed
    // filter of that loop made a generated document unreachable.
    //
    // A generated document is classified `generated` rather than `typed`,
    // and the census still carries the digest of the bytes it read. An
    // `examines` edge may name one, because such a document is a file a
    // session can open: `docs/spec/09-open-questions.md` is a generated
    // shelf index that the decision register superseded, and a probe over
    // it asks whether a session opens the document that lost.
    //
    // Under the typed-only lookup that member reached the listing below as
    // the literal `-` that stands for "no digest". So the read-set digest
    // did not move when the document moved, and a recorded result over it
    // could never be reported stale — the silent-success shape, where a
    // comparison that cannot fire reads as a comparison that passed. The
    // listing still writes `-` for a member the walk read no bytes of,
    // which is a real absence rather than a lookup that missed.
    let digests: Vec<(&str, Option<&str>)> = census
        .rows
        .iter()
        .filter_map(|row| {
            row.digest
                .as_deref()
                .map(|digest| (row.path.as_str(), Some(digest)))
        })
        .collect();
    // The read set: every probe of the selection, and every document any of
    // them examines. A path that is both is one member, and the probe is
    // the reason a reader is given, because a probe document is in the set
    // whether or not anything examines it.
    let mut reads: Vec<Read> = Vec::new();
    let mut anchors: Vec<String> = Vec::new();
    for selected in selected {
        let mut add = |path: &str, because: Because| {
            if let Some(known) = reads.iter_mut().find(|read| read.path == path) {
                // A probe that another probe examines is in the set for
                // both reasons, and the reader is told the stronger one: it
                // is in the set whether or not anything examines it.
                if because == Because::Probe {
                    known.because = Because::Probe;
                }
                return;
            }
            let digest = digests
                .iter()
                .find(|(known, _)| *known == path)
                .and_then(|(_, digest)| digest.map(|text| text.to_string()));
            reads.push(Read {
                path: path.to_string(),
                digest,
                because,
            });
        };
        add(&selected.path, Because::Probe);
        for examined in &selected.examines {
            match examined.id {
                // A document of this corpus, which the census hashes.
                Some(_) => add(&examined.path, Because::Examined),
                // An external anchor. Spec 12: it is in no read set.
                None => {
                    if !anchors.contains(&examined.path) {
                        anchors.push(examined.path.clone());
                    }
                }
            }
        }
    }
    reads.sort_by(|a, b| a.path.cmp(&b.path));
    anchors.sort();
    let mut listing = String::new();
    for read in &reads {
        listing.push_str(&read.path);
        listing.push('\t');
        listing.push_str(read.digest.as_deref().unwrap_or("-"));
        listing.push('\n');
    }
    ReadSet {
        digest: headwater_hash::digest(listing.as_bytes()),
        reads,
        anchors,
    }
}

/// What [`read_set_over`] composes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadSet {
    /// The digest over the listing of `reads`, by path and content.
    pub digest: String,
    /// The documents the digest covers, in path order.
    pub reads: Vec<Read>,
    /// Targets an `examines` edge names that are not documents of this
    /// corpus. See [`Plan::anchors`].
    pub anchors: Vec<String>,
}

/// A list a probe declares under its expectation, read from the parse the
/// census took: the closed answer set for [`ANSWERS`], and the values of it that
/// satisfy for [`EXPECTED`].
///
/// The first fenced block under `## [EXPECTATION_SECTION]`, loaded as YAML, and
/// the `key` of it. Anything that does not read that way declares
/// nothing, and the caller refuses an `answered` probe that declares nothing —
/// so a malformed block and an absent one reach one refusal rather than one
/// refusal and one silent pass.
///
/// It reads the document the census parsed and never the file again, for the
/// reason [`headwater_census::census::Row`] carries a parse at all: a second
/// read is a second corpus, and the two accounts can differ.
pub fn declared_answers(document: &headwater_doc::Document, key: &str) -> Vec<String> {
    use headwater_doc::body::BlockKind;

    let mut under = false;
    for block in &document.body.blocks {
        match block.kind {
            BlockKind::Heading(2) => under = block.text().trim() == EXPECTATION_SECTION,
            BlockKind::Heading(_) => under = false,
            BlockKind::Code if under => {
                let Ok(loaded) = headwater_yaml::load(&block.text()) else {
                    return Vec::new();
                };
                let Some(map) = loaded.value.as_map() else {
                    return Vec::new();
                };
                let Some(items) = map.get(key).and_then(|entry| entry.value.as_seq()) else {
                    return Vec::new();
                };
                return items
                    .iter()
                    .filter_map(|item| item.value.as_scalar())
                    .map(|scalar| scalar.text.clone())
                    .collect();
            }
            _ => {}
        }
    }
    Vec::new()
}

/// The arms of a run: the tier's, narrowed by the caller if the caller asked.
fn arms(envelope: &Envelope, narrowing: Option<Arm>) -> Vec<Arm> {
    match narrowing {
        None => envelope.arms.clone(),
        Some(arm) => envelope
            .arms
            .iter()
            .copied()
            .filter(|declared| *declared == arm)
            .collect(),
    }
}

/// The harness version, which is the version of this engine.
///
/// A separate number would be a second version to keep in step with the one a
/// `requires_engine` range already reads, and two versions that drift name two
/// harnesses.
fn headwater_resolve_version() -> &'static str {
    headwater_resolve::release::ENGINE
}

/// The first probe, in selection order, whose predicate names a document that
/// an ablation entry removes.
pub fn ablated_examined(selected: &[Selected], ablation: &[String]) -> Option<Refusal> {
    selected
        .iter()
        .filter(|probe| probe.expectation.names_documents())
        .find_map(|probe| {
            probe.examines.iter().find_map(|examined| {
                ablation
                    .iter()
                    .find(|entry| removes(entry, &examined.path))
                    .map(|entry| Refusal::AblatedExamined {
                        probe: probe.id.clone(),
                        path: examined.path.clone(),
                        entry: entry.clone(),
                    })
            })
        })
}

/// Whether removing `entry` removes `path`: the two are equal, or `entry` is a
/// whole-component prefix of `path`. `docs` removes `docs/a.md` and never
/// `docsx/a.md`.
fn removes(entry: &str, path: &str) -> bool {
    path == entry
        || path
            .strip_prefix(entry)
            .is_some_and(|rest| rest.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn probe(id: &str, expectation: Expectation, examined: &str) -> Selected {
        Selected {
            path: format!("probes/{id}.md"),
            id: id.to_string(),
            category: Category::Sufficiency,
            expectation,
            examines: vec![Examined {
                id: Some("DOC".into()),
                path: examined.to_string(),
            }],
            oracle: None,
            answers: Vec::new(),
            expected: Vec::new(),
        }
    }

    #[test]
    fn an_ablation_entry_removes_a_whole_component_and_never_a_string_prefix() {
        assert!(removes("docs", "docs"));
        assert!(removes("docs", "docs/decisions/0001.md"));
        assert!(removes("docs/decisions", "docs/decisions/0001.md"));
        assert!(!removes("docs", "docsx/a.md"));
        assert!(!removes("docs", "docs.md"));
        assert!(!removes("docs/decisions", "docs/a.md"));
    }

    #[test]
    fn an_answered_or_patched_probe_is_admitted_whatever_it_examines() {
        let ablation = vec!["docs".to_string()];
        let admitted = vec![
            probe("A", Expectation::Answered, "docs/decisions/0001.md"),
            probe("B", Expectation::Patched, "docs/spec/05.md"),
        ];
        assert_eq!(ablated_examined(&admitted, &ablation), None);

        let mixed = vec![
            admitted[0].clone(),
            probe("C", Expectation::Cited, "docsx/kept.md"),
            probe("D", Expectation::Opened, "docs/spec/05.md"),
        ];
        assert_eq!(
            ablated_examined(&mixed, &ablation),
            Some(Refusal::AblatedExamined {
                probe: "D".into(),
                path: "docs/spec/05.md".into(),
                entry: "docs".into(),
            })
        );
    }

    #[test]
    fn a_patched_probe_grades_against_a_rule_this_engine_carries() {
        assert!(
            headwater_check::RULES.contains(&"section.required.missing"),
            "the oracle refusal reads this list, so a name in it is the arm that passes"
        );
        assert!(!headwater_check::RULES.contains(&"section.required.absent"));
    }

    #[test]
    fn the_three_document_forms_are_the_ones_that_need_a_target() {
        for expectation in Expectation::ALL {
            let expected = matches!(
                expectation,
                Expectation::Opened | Expectation::NotOpened | Expectation::Cited
            );
            assert_eq!(expectation.names_documents(), expected);
        }
    }
}
