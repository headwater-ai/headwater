// SPDX-License-Identifier: Apache-2.0
//! The CI adapters: one run of the check layer, in a vocabulary a platform
//! reads.
//!
//! [Spec 6](../../../../docs/spec/06-engine-architecture.md#ci-adapters): "The
//! engine emits findings. Adapters translate them to the native vocabulary of a
//! platform: annotations, check runs, job summaries, review comments. Adapters
//! are thin and swappable so that no forge is privileged in the core."
//!
//! # Four formats, and one dispatcher over them
//!
//! Spec 6 fixes the set: `text`, `json`, `sarif`, `markdown`. [`render`] writes
//! every one of them, and it is the only place any caller turns a run into
//! bytes. [`text`] needs the census and the graph as well as the run, so
//! [`render`] takes all three and the three translations ignore two of them.
//! One argument list that says more than a translation needs is the price of a
//! single dispatcher. The alternative is a `None` arm for `text` and a caller
//! that composes that format for itself, which costs one composition per
//! caller: `headwater check` is one such caller and the MCP `check` tool is
//! another, and a terminal and an agent that read different text are reading
//! about different corpora as far as either can tell.
//!
//! # Why this is a crate and not a module of `headwater-check`
//!
//! The dependency direction is the boundary spec 8 asks for.
//! [Departure 6](../../../../docs/spec/08-design-departures.md): "a
//! platform-neutral core with thin adapters. No forge, tracker, or cloud is
//! privileged. More than one adapter exists from the start, and that enforces
//! the adapter boundary." A module inside the runner could reach a check. A
//! crate above it cannot, and the compiler is what says so rather than a
//! review. Nothing in `headwater-check` names this crate.
//!
//! # Two adapters and one neutral serialization, which is what "more than one"
//! buys
//!
//! [`Format::Sarif`] and [`Format::Markdown`] translate into two platform
//! vocabularies that spec 6 names: a check run, which a forge ingests as SARIF,
//! and a job summary, which a forge renders as Markdown. They lose different
//! things, and that is the point of shipping both. One adapter cannot show
//! where the boundary is, because everything it needs is in the core by
//! definition.
//!
//! [`Format::Json`] is not an adapter. It is the finding shape
//! [spec 4](../../../../docs/spec/04-assurance-model.md#findings) already
//! declares, written out, for an adapter that this repository did not write.
//! Its loss set is empty and [`census`] is what audits that claim.
//!
//! # Every format declares a loss set
//!
//! [Spec 6](../../../../docs/spec/06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped)
//! requires an emitter to declare "the node classes, edge classes, and
//! attributes that its target cannot carry, each with a reason". A finding is
//! not a node, so [`Loss`] here is about a *field of a run* rather than about a
//! class of the graph, and it is deliberately not
//! `headwater_generate::export::Loss`. Two subjects, two types, and an adapter
//! that depended on the graph emitters to say what a job summary drops would
//! have the boundary above pointing the wrong way.
//!
//! What the two share is the doctrine. A declaration is a claim, and a census
//! is the audit of it. [`census`] is that audit: every finding of a run is in
//! the output, whatever its escape class, or the format declared why not.
//!
//! # The engine emits and never orders
//!
//! No output here carries a verdict about a merge. `--strict` decides an exit
//! status and the artifact says the same thing whether or not the flag was
//! given, because the posture is the control's
//! ([spec 12](../../../../docs/spec/12-check-layer.md#severity-is-the-checks-posture-is-the-controls))
//! and the landing order is the forge's.

pub mod json;
pub mod markdown;
pub mod sarif;
pub mod text;

use headwater_census::census::Census as Taken;
use headwater_check::adoption::Task;
use headwater_check::suppression::Suppression;
use headwater_check::{Finding, Run, Severity};
use headwater_graph::Graph;
use headwater_yaml::{Spanned, Value};

/// The name this tool reports under, in every format that names it.
pub const TOOL: &str = "headwater";

/// Where a reader finds out what the tool is.
pub const TOOL_URI: &str = "https://github.com/headwater-ai/headwater";

/// One output format of `headwater check`.
///
/// The set is closed by [spec 6](../../../../docs/spec/06-engine-architecture.md#cli)
/// and this enum is that list. It is deliberately not
/// `headwater_generate::Emitter`: `export --format` names a vocabulary for the
/// *graph* and `check --format` names one for the *findings*, the two lists
/// share the word `json` and mean different artifacts by it, and one enum over
/// both would let `check --format shacl` parse.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// The report a person reads in a terminal, in the engine's own words. See
    /// [`text`].
    Text,
    /// The finding shape spec 4 declares. Not an adapter.
    Json,
    /// Static Analysis Results Interchange Format 2.1.0 (OASIS), which is what
    /// a forge ingests as a check run.
    Sarif,
    /// A job summary or a review comment, which is the other vocabulary spec 6
    /// names.
    Markdown,
}

impl Format {
    /// Every format, in the order spec 6 lists them.
    pub const ALL: [Format; 4] = [Format::Text, Format::Json, Format::Sarif, Format::Markdown];

    pub fn name(self) -> &'static str {
        match self {
            Format::Text => "text",
            Format::Json => "json",
            Format::Sarif => "sarif",
            Format::Markdown => "markdown",
        }
    }

    pub fn parse(text: &str) -> Option<Format> {
        Format::ALL.into_iter().find(|format| format.name() == text)
    }

    /// What this format cannot carry, and why.
    ///
    /// Empty for [`Format::Json`], which is the claim that it drops nothing,
    /// and [`census`] is what holds that claim. Empty for [`Format::Text`] on
    /// the same claim and for a different reason: a loss is a member of a
    /// target vocabulary that a run has no value for, and the terminal is the
    /// engine's own vocabulary rather than a target. That format carries more
    /// than the other three rather than less, because the census and the graph
    /// are in it.
    pub fn loss(self) -> &'static [Loss] {
        match self {
            Format::Text | Format::Json => &[],
            Format::Sarif => sarif::LOSS,
            Format::Markdown => markdown::LOSS,
        }
    }
}

/// One entry of a format's loss set: something a run carries, the target
/// vocabulary has no member for, and the reason.
///
/// [`Loss::carrier`] names where the value went instead. A property bag is not
/// a member of the vocabulary, so a value that rides in one is still a loss for
/// every consumer that reads the vocabulary alone. Saying which is the
/// difference between a loss set and an apology.
///
/// The carrier is a structure rather than the sentence it renders as, because
/// [`census`] resolves it against the emitted bytes. A `&'static str` there is
/// a claim in prose, and a claim in prose is what this crate shipped: SARIF
/// declared that the coverage went to `run.properties.headwater.coverage` while
/// three of the seven values coverage reports went nowhere, and no run read the
/// declaration against the document.
#[derive(Clone, Copy, Debug)]
pub struct Loss {
    pub field: &'static str,
    pub reason: &'static str,
    pub carrier: Carrier,
}

impl Loss {
    /// Where the value went, as the one sentence every artifact writes.
    ///
    /// The rendering is here and in no emitter, so the declaration and the
    /// string a consumer reads cannot disagree.
    pub fn carried_in(&self) -> String {
        self.carrier.render()
    }
}

/// One place in an artifact that a lost value went, and what it holds there.
///
/// `at` is the path from the object the [`Carrier`] is relative to. `members`
/// are the members of the value at `at` that the entry claims are there, and it
/// is empty where the value at `at` is the whole of what was carried.
///
/// The two grains are not decoration. A carrier that named a block alone would
/// pass over a block emptied of everything but its own name, which is the shape
/// of the defect this type exists to catch: an entry that said "the coverage is
/// in this bag" over a bag that held four of the seven values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Place {
    pub at: &'static [&'static str],
    pub members: &'static [&'static str],
}

impl Place {
    /// The path, dotted, with no statement about which object it starts from.
    fn path(&self) -> String {
        self.at.join(".")
    }
}

/// Whether a run writes a [`Carrier::Run`] place, read off the run and never
/// off the artifact.
///
/// A predicate rather than a flag, because a member written on some runs and
/// not on others is the case a presence test cannot audit. It reads the run,
/// which is what the emitter reads, so a drift between the two fails the census
/// rather than passing it.
pub type OnRun = fn(&Run) -> bool;

/// Whether one reported finding's record carries a [`Carrier::PerFinding`]
/// place.
pub type OnFinding = fn(&Reported<'_>, &Run) -> bool;

/// Every run writes it.
pub fn every_run(_: &Run) -> bool {
    true
}

/// A run that was told about a change writes it, and a full-corpus run does
/// not.
pub fn a_scoped_run(run: &Run) -> bool {
    run.change.is_some()
}

/// Every record carries it.
pub fn every_finding(_: &Reported<'_>, _: &Run) -> bool {
    true
}

/// Where a value the target vocabulary cannot hold went instead.
///
/// Four cases, and the split is what lets [`census`] say "this entry was not
/// audited" rather than reporting it as one that passed. The two that name a
/// member of the artifact are resolved against the emitted bytes. The two that
/// do not are counted, and the count stands in the census beside the held ones.
/// No `PartialEq`. Two of these variants hold a function pointer, and a
/// comparison of those compares addresses the compiler is free to merge or to
/// duplicate, which is a comparison that answers a question nobody asked.
#[derive(Clone, Copy, Debug)]
pub enum Carrier {
    /// Nowhere. A consumer that holds these bytes cannot recover the value.
    Nowhere,
    /// Outside these bytes, by the means named. A second command or a second
    /// format, in prose because it is not a path into this artifact.
    Elsewhere(&'static str),
    /// Members of the artifact's run object, on the runs `when` names.
    Run {
        places: &'static [Place],
        when: OnRun,
    },
    /// Members of the record the artifact writes per reported finding, on the
    /// records `when` names.
    PerFinding {
        places: &'static [Place],
        when: OnFinding,
    },
}

impl Carrier {
    /// The sentence an artifact carries in its `carried_in` member.
    ///
    /// A [`Carrier::Run`] place is written with a `run.` prefix and a
    /// [`Carrier::PerFinding`] place without one, which is the notation the
    /// SARIF entries already used and a difference a reader could not otherwise
    /// see. Two places join with `and`, because an entry whose value reached
    /// two members has to name both.
    pub fn render(&self) -> String {
        let joined = |places: &[Place], prefix: &str| {
            places
                .iter()
                .map(|place| format!("{prefix}{}", place.path()))
                .collect::<Vec<String>>()
                .join(" and ")
        };
        match self {
            Carrier::Nowhere => String::new(),
            Carrier::Elsewhere(text) => (*text).to_string(),
            Carrier::Run { places, .. } => joined(places, "run."),
            Carrier::PerFinding { places, .. } => joined(places, ""),
        }
    }

    /// Whether [`census`] can hold this entry to the bytes.
    pub fn names_a_member(&self) -> bool {
        matches!(self, Carrier::Run { .. } | Carrier::PerFinding { .. })
    }
}

/// Which escape mechanism holds a finding back, in the precedence
/// [spec 4](../../../../docs/spec/04-assurance-model.md#suppression) fixes:
/// waiver, then migration-pending, then suppression.
///
/// Two variants rather than three, because this engine has no waiver mechanism
/// and a variant nothing constructs is a claim with no instance behind it. The
/// run reports the absence in the same sentence that reports the other two, and
/// [`sarif`] states what happens to the mapping when a waiver arrives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Escape {
    /// A task of the adoption payload holds it: declared debt with an owner and
    /// an expiry ([spec 7](../../../../docs/spec/07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states)).
    MigrationPending,
    /// A directive in the source hides it: one author's local judgment.
    Suppression,
}

impl Escape {
    pub fn name(self) -> &'static str {
        match self {
            Escape::MigrationPending => "migration-pending",
            Escape::Suppression => "suppression",
        }
    }
}

/// One finding of a run, with what became of it.
///
/// The union that every format writes. A reader of [`Run::findings`] alone sees
/// what was reported and cannot see what was held or hidden, and the whole
/// question a CI surface answers is which of the three a line is.
pub struct Reported<'a> {
    pub finding: &'a Finding,
    /// `None` for a live finding.
    pub escape: Option<Escape>,
    /// The task that holds it, for [`Escape::MigrationPending`].
    pub task: Option<&'a Task>,
    /// The directive that hides it, for [`Escape::Suppression`].
    pub directive: Option<&'a Suppression>,
}

impl Reported<'_> {
    /// Whether a reader of the report sees this finding among the findings.
    pub fn is_live(&self) -> bool {
        self.escape.is_none()
    }
}

/// Every finding of a run, live and escaped, in the one order spec 12 fixes.
///
/// The three sources are disjoint by construction: the runner applies the
/// adoption payload first and the directives second, so a held finding never
/// reaches a directive. Sorting the union rather than concatenating three
/// sequences is what makes the output stable when a finding moves from one
/// bucket to another.
pub fn reported(run: &Run) -> Vec<Reported<'_>> {
    let mut all: Vec<Reported<'_>> = Vec::new();
    for finding in &run.findings {
        all.push(Reported {
            finding,
            escape: None,
            task: None,
            directive: None,
        });
    }
    for finding in &run.adoption.pending {
        all.push(Reported {
            finding,
            escape: Some(Escape::MigrationPending),
            task: run.adoption.tasks.iter().find(|task| {
                task.pairs
                    .iter()
                    .any(|pair| pair.rule == finding.rule && pair.path == finding.path)
            }),
            directive: None,
        });
    }
    for (finding, directive) in run.suppressions.hidden_findings() {
        all.push(Reported {
            finding,
            escape: Some(Escape::Suppression),
            task: None,
            directive: Some(directive),
        });
    }
    all.sort_by(|left, right| left.finding.order().cmp(&right.finding.order()));
    all
}

/// What a format carried, and what its loss set accounted for.
///
/// The audit of the claim a loss set makes, in the shape
/// [spec 6](../../../../docs/spec/06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped)
/// fixes for the graph emitters: "Every node and every edge in the graph is
/// either present in the output, or accounted for by a declared loss reason."
/// Here the denominator is every finding of the run rather than every node of
/// the graph, and the rest of the sentence is the same one.
///
/// # A loss set makes two claims and this holds both
///
/// The first three fields are the finding claim: every finding of the run is in
/// the output. The last four are the **carrier** claim, which nothing audited
/// until this type held it. An entry that says a value went to a member of the
/// artifact is a statement about the emitted bytes, and it was a statement a
/// recorded artifact held and nothing else. That is how one entry stood while it
/// named the whole of the coverage and pointed at four of its seven values.
///
/// `entries == held + adrift.len() + unaudited` on every census, which
/// [`Census::accounts`] states. An entry that fell out of all three outcomes
/// would be an entry nothing looked at and nothing reported, which is the state
/// this type exists to make impossible.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Census {
    pub findings: usize,
    pub carried: usize,
    /// Findings the output does not carry and no loss reason covers. A
    /// non-empty list is a defect in the adapter, not a fact about the corpus.
    pub unaccounted: Vec<String>,
    /// Loss entries the format declares.
    pub entries: usize,
    /// Entries whose carrier names members of this artifact, where the artifact
    /// agrees with the run about every one of them: there where this run has a
    /// value, and absent where it has none.
    pub held: usize,
    /// Entries whose carrier and artifact disagree. A non-empty list is a defect
    /// in the adapter, in the sense [`Census::unaccounted`] is one: the emitter
    /// and its own declaration have come apart, and the declaration is what a
    /// consumer reads.
    pub adrift: Vec<String>,
    /// Entries that name no member of this artifact: the value went nowhere, or
    /// it went somewhere these bytes are not. Counted rather than passed over,
    /// because "nothing here audits this" and "this was audited and it held" are
    /// different results, and only one of them is evidence.
    pub unaudited: usize,
}

impl Census {
    pub fn is_defective(&self) -> bool {
        !self.unaccounted.is_empty() || !self.adrift.is_empty()
    }

    /// Every declared entry reached one of the three outcomes.
    pub fn accounts(&self) -> bool {
        self.entries == self.held + self.adrift.len() + self.unaudited
    }

    /// What a caller prints when the audit fails.
    ///
    /// One rendering, because the three callers of [`census`] wrote three of
    /// them and a fourth outcome would have reached one caller. Empty when the
    /// census found no defect.
    pub fn complaint(&self, format: Format) -> String {
        let mut out = String::new();
        if !self.unaccounted.is_empty() {
            out.push_str(&format!(
                "the {} adapter dropped {} of {} findings with no declared loss reason:\n",
                format.name(),
                self.unaccounted.len(),
                self.findings
            ));
            for missing in &self.unaccounted {
                out.push_str(&format!("  {missing}\n"));
            }
        }
        if !self.adrift.is_empty() {
            out.push_str(&format!(
                "the {} loss set declares {} of {} carriers that this artifact does not agree with:\n",
                format.name(),
                self.adrift.len(),
                self.entries
            ));
            for adrift in &self.adrift {
                out.push_str(&format!("  {adrift}\n"));
            }
        }
        out
    }
}

/// The objects a loss carrier's paths start from, read out of the bytes.
///
/// The artifact is parsed rather than searched. `headwater_yaml::load` is the
/// reader that `headwater_yaml::json` names as the other half of its writer,
/// because JSON is a subset of the YAML 1.2 core schema that loader implements.
/// A search would carry the defect the finding half of [`census`] still has: a
/// substring reaches a document's index of names as readily as its records.
struct Objects<'a> {
    /// What a [`Carrier::Run`] place is relative to.
    run: &'a Spanned<Value>,
    /// One record per reported finding, in [`reported`] order, which is the
    /// order both emitters write them in.
    records: &'a [Spanned<Value>],
}

/// Where those two objects are in each format's document.
///
/// `None` for a format whose artifact is prose. That is not a pass: a carrier
/// that names a member of such an artifact reaches [`Census::adrift`], because a
/// member path into a document nothing can parse is a claim nothing can hold.
fn objects<'a>(format: Format, root: &'a Spanned<Value>) -> Option<Objects<'a>> {
    let empty: &'a [Spanned<Value>] = &[];
    let seq = |value: Option<&'a Spanned<Value>>| {
        value
            .and_then(|found| found.value.as_seq())
            .unwrap_or(empty)
    };
    match format {
        Format::Sarif => {
            let runs = root.value.as_map()?.get("runs")?.value.as_seq()?;
            let run = runs.first()?;
            Some(Objects {
                run,
                records: seq(run.value.as_map()?.get("results")),
            })
        }
        Format::Json => Some(Objects {
            run: root,
            records: seq(root.value.as_map()?.get("findings")),
        }),
        Format::Text | Format::Markdown => None,
    }
}

/// One step of a member path, and then the next.
fn member<'a>(value: &'a Spanned<Value>, path: &[&str]) -> Option<&'a Spanned<Value>> {
    let mut at = value;
    for key in path {
        at = at.value.as_map()?.get(key)?;
    }
    Some(at)
}

/// Whether one object carries one place: the path resolves, and every member the
/// place names is under it.
fn holds(object: &Spanned<Value>, place: &Place) -> Result<(), String> {
    let Some(at) = member(object, place.at) else {
        return Err(format!("`{}` is not there", place.path()));
    };
    let missing: Vec<&str> = place
        .members
        .iter()
        .copied()
        .filter(|name| member(at, &[name]).is_none())
        .collect();
    match missing.is_empty() {
        true => Ok(()),
        false => Err(format!(
            "`{}` is there and holds none of: {}",
            place.path(),
            missing.join(", ")
        )),
    }
}

/// Every place of one carrier, against one object that either writes it or does
/// not.
///
/// Both directions, because an entry is a statement about the artifact of a run
/// with a value **and** about the artifact of a run without one. A member
/// written where this run has nothing is what makes a reader think the absence
/// of that member means something, and #234 rests on exactly that reading.
fn against(object: &Spanned<Value>, places: &[Place], written: bool) -> Vec<String> {
    places
        .iter()
        .filter_map(|place| match written {
            true => holds(object, place).err(),
            false => member(object, place.at).is_some().then(|| {
                format!(
                    "`{}` is there and this run carries no value for it",
                    place.path()
                )
            }),
        })
        .collect()
}

/// One per-finding carrier, against every record of the artifact.
fn per_finding(
    read: &Objects<'_>,
    places: &[Place],
    when: OnFinding,
    all: &[Reported<'_>],
    run: &Run,
) -> Vec<String> {
    // The record list has to line up with the finding list before a per-record
    // place means anything. Both emitters write one record per `reported` entry,
    // in that order, and a census that assumed it in silence would audit the
    // wrong record after any drift.
    if read.records.len() != all.len() {
        return vec![format!(
            "the artifact writes {} records for {} reported findings",
            read.records.len(),
            all.len()
        )];
    }
    all.iter()
        .zip(read.records)
        .flat_map(|(entry, record)| {
            against(record, places, when(entry, run))
                .into_iter()
                .map(|fault| format!("{fault}, on {}", entry.finding.rule))
                .collect::<Vec<String>>()
        })
        .collect()
}

/// Audit one rendered artifact against the run it came from.
///
/// It reads the bytes rather than the emitter, for the reason
/// `headwater_generate::export::audit` does: an emitter that audited itself
/// would be the untrusted projector one layer out.
///
/// # The finding half, and what it still cannot say
///
/// A finding is carried when the output names its rule and its path. Those are
/// two independent substring tests over the whole document, so a format that
/// prints an index of rule names and an index of paths passes them whatever its
/// records hold. `HW-OBL-0110` records the measurement, and this function still
/// carries the defect. That record proposes a test per line or per record and
/// says that none of the four formats needs a parser for it. [`Objects`] is a
/// second route, available to the two machine-readable formats and to neither
/// prose one, and nothing here settles which of the two that record takes.
///
/// # The carrier half
///
/// Every entry of the format's loss set reaches exactly one of three outcomes.
/// An entry whose carrier names members of the artifact is resolved against the
/// parsed document and is held or adrift. An entry whose carrier names nowhere,
/// or a place outside these bytes, is counted as unaudited. Nothing falls out,
/// which [`Census::accounts`] states and the suite asserts.
///
/// # What a held entry does not prove
///
/// A place whose `members` are empty is held when its path resolves to anything
/// at all, so a carrier can be made vaguer without going adrift: name the bag
/// above the member and both resolve. What catches that today is the recorded
/// artifact rather than this function, because the sentence an entry renders is
/// derived from the path and changes with it.
///
/// And the predicate on an entry reads the run through the same function the
/// emitter reads it through, which is what stops the two from drifting apart. A
/// change to that one function moves the artifact and the expectation together,
/// so this audit stays silent and the recorded artifact is again what reports
/// it.
pub fn census(run: &Run, format: Format, artifact: &str) -> Census {
    census_with(run, format, artifact, format.loss())
}

/// The same audit, over a loss set the caller names.
///
/// [`census`] passes the format's own set and every caller in this engine calls
/// that one. This exists for a suite, and the reason it has to exist is the
/// review question underneath it: a test that could only doctor the artifact
/// would measure the emitter and never the instrument. A format's loss set is a
/// `const`, so the only way to ask "does a **wrong** declaration fail this
/// audit" is to hand the audit a wrong declaration.
pub fn census_with(run: &Run, format: Format, artifact: &str, loss: &[Loss]) -> Census {
    let all = reported(run);
    let mut unaccounted = Vec::new();
    let mut carried = 0;
    for entry in &all {
        let rule = artifact.contains(entry.finding.rule);
        let path = artifact.contains(entry.finding.path.as_str());
        match rule && path {
            true => carried += 1,
            false => unaccounted.push(format!(
                "{} {} at {}",
                entry.finding.rule, entry.finding.path, entry.finding.line
            )),
        }
    }

    // The document, parsed once, and only where an entry needs it. A format
    // whose loss set names no member of its own artifact never reaches a parser,
    // which is what keeps the two prose formats out of one.
    let parsed = loss
        .iter()
        .any(|entry| entry.carrier.names_a_member())
        .then(|| headwater_yaml::load(artifact));
    let read = match &parsed {
        Some(Ok(root)) => objects(format, root),
        Some(Err(_)) | None => None,
    };
    let unread = match &parsed {
        Some(Err(_)) => "this artifact does not parse",
        Some(Ok(_)) => "no reader of this format finds the object the path starts from",
        None => "this artifact was not read",
    };

    let mut held = 0;
    let mut unaudited = 0;
    let mut adrift = Vec::new();
    for entry in loss {
        let faults = match (entry.carrier, &read) {
            (Carrier::Nowhere | Carrier::Elsewhere(_), _) => {
                unaudited += 1;
                continue;
            }
            (Carrier::Run { .. } | Carrier::PerFinding { .. }, None) => {
                vec![unread.to_string()]
            }
            (Carrier::Run { places, when }, Some(read)) => against(read.run, places, when(run)),
            (Carrier::PerFinding { places, when }, Some(read)) => {
                per_finding(read, places, when, &all, run)
            }
        };
        match faults.first() {
            None => held += 1,
            // One line per entry, and the first fault on it. A carrier that is
            // wrong is wrong once: the same missing member repeats on every
            // record, and a list of forty-five copies of it is a list nobody
            // reads.
            Some(fault) => adrift.push(format!(
                "{} claims `{}`, and {fault}",
                entry.field,
                entry.carried_in()
            )),
        }
    }

    Census {
        findings: all.len(),
        carried,
        unaccounted,
        entries: loss.len(),
        held,
        adrift,
        unaudited,
    }
}

/// What every format states about the run beside the findings.
///
/// [Spec 6](../../../../docs/spec/06-engine-architecture.md#ci-adapters): "Every
/// run reports the corpus tree, the taxonomy lock hash, and its read set beside
/// the findings." Two of those three are here. The corpus tree is not, because
/// nothing computes one — see [`sarif`] for where that absence shows up and
/// what this crate declined to print in its place.
pub struct Subject<'a> {
    /// The taxonomy package the lock names.
    pub package: &'a str,
    pub version: &'a str,
    /// The lock digest, as `sha256:...`.
    pub lock: &'a str,
    /// The injected clock, as `YYYY-MM-DD`. An input to the verdict, so an
    /// output that omits it cannot be reproduced from itself.
    pub now: &'a str,
}

/// The run in one format: the whole of how this engine turns a run into bytes.
///
/// The census and the graph are here because [`text`] writes both and the three
/// translations write neither. See the module comment for why they are
/// parameters of the dispatcher rather than of one arm of it.
pub fn render(
    run: &Run,
    census: &Taken,
    graph: &Graph,
    subject: &Subject<'_>,
    format: Format,
) -> String {
    render_at(
        run,
        census,
        graph,
        subject,
        format,
        headwater_check::fill::WIDTH,
        headwater_check::paint::ColorMode::Plain,
    )
}

/// The same dispatch, with a width for the one format that is laid out.
///
/// [`Format::Text`] is the report a person reads and the only one a width means
/// anything to. The other three ignore it, exactly as they already ignore
/// `census` and `graph`: a machine format is parsed rather than read, and a line
/// break inside one would be a defect rather than a courtesy. `headwater check
/// --wide` is the one caller in the tree that states a number here, which is why
/// this is a second entry point rather than a sixth parameter on every call
/// site.
pub fn render_at(
    run: &Run,
    census: &Taken,
    graph: &Graph,
    subject: &Subject<'_>,
    format: Format,
    width: usize,
    mode: headwater_check::paint::ColorMode,
) -> String {
    match format {
        Format::Text => text::render_at(run, census, graph, subject, width, mode),
        Format::Json => json::render(run, subject),
        Format::Sarif => sarif::render(run, subject),
        Format::Markdown => markdown::render(run, subject),
    }
}

/// The check's severity as the word every format writes.
pub fn severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warn => "warn",
        Severity::Info => "info",
    }
}
