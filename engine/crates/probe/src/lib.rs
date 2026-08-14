// SPDX-License-Identifier: Apache-2.0
//! The probe harness: the halves of it that are deterministic.
//!
//! [Spec 5](../../../../docs/spec/05-ai-integration.md#measuring-whether-any-of-this-works)
//! declares the probe suite and
//! [spec 6](../../../../docs/spec/06-engine-architecture.md#cli) says where it
//! sits: "`probe` is the one verb that reaches the network, so it never runs
//! inside `check` and never gates."
//!
//! # What a probe result is, and what makes it citable
//!
//! A probe result is **a function of three inputs and of nothing else**: the
//! transcript of a run, the expectations the probes declare, and the version of
//! the grader that evaluated them. That sentence is the whole design, and this
//! crate exists to make each of the three a committed artifact with a name.
//!
//! | input | where it is | who writes it |
//! |---|---|---|
//! | the transcript | a `probe_transcript` document on `docs/probe-runs/` | the recorder, mechanically |
//! | the expectations | a `probe` document on `docs/probes/` | a person |
//! | the grader version | the version of the crate that evaluates them | [#85](https://github.com/headwater-ai/headwater/issues/85) |
//!
//! A number is citable when a reader can fetch every input it came from and get
//! the number again. That is why the transcript is corpus content and the
//! capture-cost store is not: nothing recomputes a fact about a run that ended,
//! and a probe result is recomputed from the row above on every run of
//! `generate --check`. A result whose transcript is not committed carries the
//! `asserted` [warrant](../../../../docs/spec/03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two),
//! and an asserted document discharges no evidence obligation, so a measurement
//! layer that skipped the transcript would produce content this corpus refuses
//! as evidence.
//!
//! # The sweep's shape does not carry a transcript, and that is this crate's
//! sharpest finding
//!
//! [`headwater_sweep`](../../sweep/index.html) reaches a model in three parts:
//! the engine writes a briefing, an agent reads it, and the engine reads back
//! the file the agent wrote. The obvious move here was to copy all three. It
//! does not work, and the reason is a rule spec 5 already states.
//!
//! > A self-report is the agent's account of its own process, and no probe
//! > accepts one.
//!
//! A file that an agent writes about which documents it opened **is** the
//! agent's account of its own process. So a transcript can never arrive by the
//! sweep's middle part. It has to be recorded by something that observes the
//! session from outside it, event by event, and writes what it observed rather
//! than what the model reports. That is the recorder, it is the one component
//! of this layer that reaches the network, and it is not in this repository.
//!
//! The consequence is a smaller harness than the issue assumed and an honest
//! one. This crate plans a run and reads a recorded transcript back. Nothing
//! here writes a transcript, and there is no `--write` for one, for the same
//! reason the sweep has no `--write` for an edge.
//!
//! # Four things keep this out of every gate, and none of them is a promise
//!
//! The list is the one [spec 12](../../../../docs/spec/12-check-layer.md#four-things-stop-a-sweep-from-gating-and-none-of-them-is-a-rule-that-somebody-keeps)
//! fixes for the sampler path, applied here:
//!
//! 1. **A crate boundary the compiler holds.** This crate names
//!    `headwater-check`, so `headwater-check` cannot name it.
//! 2. **An exit status no probe moves.** [`plan::Plan`] carries its refusal as
//!    a value a reader sees. The verb over it exits zero whatever it holds.
//! 3. **No caller in any gate.** No hook, no CI step and no other verb calls
//!    it. `.claude/skills/fixtures.sh` asserts that.
//! 4. **No socket.** No crate of this engine depends on the network, and
//!    nothing here opens one.
//!
//! # The harness fails closed, and a refusal is the cheaper error
//!
//! Spec 5: "The harness projects the cost of a run before it starts, and it
//! refuses to start a run that exceeds the budget." Three things stop a run
//! here, and every one of them stops the whole run rather than dropping the
//! probe that caused it:
//!
//! - the projected cost is over the tier's declared budget;
//! - a probe declares `patched` and names an oracle that is not one of the
//!   [`headwater_check::RULES`] this engine carries;
//! - a probe declares an expectation over documents and names none.
//!
//! The second and the third are the shape `headwater conformance` already uses:
//! a rule this engine holds no reading for ends the run rather than being
//! skipped. A selection that quietly dropped a probe would report a rate over a
//! denominator nobody declared.

pub mod budget;
pub mod grade;
pub mod intake;
pub mod plan;

pub use budget::{Budgets, Envelope};
pub use grade::Results;
pub use intake::Record;
pub use plan::Plan;

/// What a probe asks, from the closed set
/// [spec 5](../../../../docs/spec/05-ai-integration.md#probe-categories)
/// declares.
///
/// Four, and two earlier ones are deliberately absent. Fidelity is not a probe,
/// because `generate --check` already proves that a projection agrees with its
/// source. The counterfactual is not a category either: it is the [`Arm`] of
/// every probe.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Category {
    /// Does the agent find the governing document at all?
    Discovery,
    /// Once it has, does it have enough to act correctly?
    Sufficiency,
    /// Can it get from a code path to the governing document, and back?
    Navigability,
    /// Same question, different phrasings, same answer?
    Consistency,
}

impl Category {
    pub const ALL: [Category; 4] = [
        Category::Discovery,
        Category::Sufficiency,
        Category::Navigability,
        Category::Consistency,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Category::Discovery => "discovery",
            Category::Sufficiency => "sufficiency",
            Category::Navigability => "navigability",
            Category::Consistency => "consistency",
        }
    }

    pub fn read(name: &str) -> Option<Category> {
        Category::ALL.into_iter().find(|it| it.name() == name)
    }
}

/// The predicate a probe declares, from the closed set
/// [spec 5](../../../../docs/spec/05-ai-integration.md#a-probe-is-a-document-with-a-declared-expectation)
/// fixes.
///
/// Closed because a grader evaluates it. A question whose answer needs a rubric
/// is not a probe: it is a coherence question, and the
/// [sweep](../../sweep/index.html) owns those.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expectation {
    /// The transcript shows that the session read one of the named documents.
    Opened,
    /// It read none of them.
    NotOpened,
    /// A produced artifact cites one of the named identifiers.
    Cited,
    /// The final answer is one named value from a closed set the probe
    /// declares.
    Answered,
    /// A produced patch passes a named check, which is the oracle route.
    Patched,
}

impl Expectation {
    pub const ALL: [Expectation; 5] = [
        Expectation::Opened,
        Expectation::NotOpened,
        Expectation::Cited,
        Expectation::Answered,
        Expectation::Patched,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Expectation::Opened => "opened",
            Expectation::NotOpened => "not_opened",
            Expectation::Cited => "cited",
            Expectation::Answered => "answered",
            Expectation::Patched => "patched",
        }
    }

    pub fn read(name: &str) -> Option<Expectation> {
        Expectation::ALL.into_iter().find(|it| it.name() == name)
    }

    /// Whether the predicate is over a set of documents that the probe names
    /// with `examines`.
    ///
    /// The three that are cannot be evaluated over an empty set: `opened` over
    /// nothing is never satisfied and `not_opened` over nothing always is, so
    /// each would report a rate that measures the declaration rather than the
    /// corpus.
    pub fn names_documents(self) -> bool {
        matches!(
            self,
            Expectation::Opened | Expectation::NotOpened | Expectation::Cited
        )
    }
}

/// Which run watches for a change and which estimates a difference.
///
/// Spec 5: cadence follows the purpose of the run and never the category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    /// A fixed scenario set, one arm, against a recorded baseline. It
    /// establishes no effect, so no published claim rests on it.
    Regression,
    /// Both arms, powered, one batch, one model version, for one named claim.
    Campaign,
}

impl Tier {
    pub const ALL: [Tier; 2] = [Tier::Regression, Tier::Campaign];

    pub fn name(self) -> &'static str {
        match self {
            Tier::Regression => "regression",
            Tier::Campaign => "campaign",
        }
    }

    pub fn read(name: &str) -> Option<Tier> {
        Tier::ALL.into_iter().find(|it| it.name() == name)
    }
}

/// Whether a run had the corpus present or absent.
///
/// Spec 5 removed the counterfactual from the category list because it applies
/// to every category, and listing it beside them hid that the pair doubles the
/// cost of whatever it measures.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Arm {
    Present,
    Absent,
}

impl Arm {
    pub const ALL: [Arm; 2] = [Arm::Present, Arm::Absent];

    pub fn name(self) -> &'static str {
        match self {
            Arm::Present => "present",
            Arm::Absent => "absent",
        }
    }

    pub fn read(name: &str) -> Option<Arm> {
        Arm::ALL.into_iter().find(|it| it.name() == name)
    }
}

/// Money, in whole cents.
///
/// Integers rather than a decimal, because a budget refusal is a comparison and
/// a comparison over a binary float is a refusal that depends on a rounding
/// rule nobody declared.
pub type Cents = u64;

/// Cents as a reader reads them.
pub fn dollars(cents: Cents) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}
