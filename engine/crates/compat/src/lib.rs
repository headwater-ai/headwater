// SPDX-License-Identifier: Apache-2.0
//! Measured compatibility between two taxonomies, over one corpus.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#versioning-by-measured-compatibility)
//! fixes six dimensions and states that the required version bump is a
//! consequence of the result rather than a publisher's guess about it. This
//! crate is the comparison and nothing else: every input arrives already built,
//! because the phase that built it is the phase whose answer the dimension is
//! about. A dimension that re-derived a kind or a verdict would report on a
//! reading no run performed.
//!
//! # Why the inputs are pairs of finished phases
//!
//! `classification` is the census. `consequence` and `instance_validity` are
//! the check runner. `projection` is the generator's plan. `identifier` is the
//! graph index. Each dimension takes two of the same thing and says where they
//! differ. So the caller runs each phase twice, once under each taxonomy, over
//! one corpus — and the corpus is the same tree in both runs, which is what
//! makes a difference attributable to the schema.
//!
//! # Two publishes of an unchanged package differ in trivia
//!
//! That is the failure mode this crate exists to avoid. A comparison of bytes
//! reports every release as a change, and a report that fires on every version
//! bump is indistinguishable from one that does not work. So no dimension here
//! reads a file digest, a manifest, a description or a declaration order. Every
//! one of them reads what the corpus did under each taxonomy. A taxonomy that
//! resolves to different text and produces an identical corpus reading is
//! compatible on all six, which is the answer a consumer needs.
//!
//! # `Preserved` is never what an unmeasured dimension says
//!
//! [`Outcome`] has three arms and the third is the reason for the type. A
//! candidate taxonomy that does not resolve produces no census, no run, no plan
//! and no graph, and five dimensions then have nothing to compare. Reporting
//! them as preserved would state the strongest possible claim on the strength
//! of a failure. [`Measured::against_nothing`] is the only route to that state
//! and it takes the reason, so a caller cannot reach it without one.

use headwater_census::census::{Census, Outcome as Row};
use headwater_check::instance::{Instance, Outcome as Verdict};
use headwater_check::scope::Grain;
use headwater_check::Run;
use headwater_generate::Plan;
use headwater_graph::Graph;
use headwater_resolve::migration::Step;
use headwater_resolve::{Adopted, FoundingRecord};
use std::collections::{BTreeMap, BTreeSet};

pub mod migrate;
pub mod payload;

/// The six dimensions, in the order [spec 2](../../../../docs/spec/02-taxonomy-model.md#versioning-by-measured-compatibility)
/// tabulates them. The set belongs to the engine and no taxonomy may vary it.
pub const DIMENSIONS: [&str; 6] = [
    "classification",
    "instance_validity",
    "consequence",
    "projection",
    "identifier",
    "addressability",
];

/// One thing that a version change moved.
///
/// Three fields rather than a sentence, because a reader acts on the subject
/// and a version bump is decided by whether the list is empty. `was` and `now`
/// are the two readings, in the words of the phase that produced them.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Break {
    /// What moved: a document path, an identifier, a projection output, or the
    /// rule and target of one check instance.
    pub at: String,
    pub was: String,
    pub now: String,
}

/// What one dimension found. Closed, and matched exhaustively.
#[derive(Clone, Debug)]
pub enum Outcome {
    /// It ran and the two readings agree.
    Preserved,
    /// It ran and they do not. Never empty: [`Outcome::over`] is the only
    /// constructor and it answers `Preserved` for an empty list, which holds
    /// the two apart in the way [`headwater_check::instance::Outcome`] holds a
    /// pass apart from a failure with no findings in it.
    Broken(Vec<Break>),
    /// It did not run, and the reason is here rather than nowhere. See the
    /// module comment.
    NotMeasured(String),
}

impl Outcome {
    /// The verdict of a dimension that collected its breaks as it went.
    pub fn over(breaks: Vec<Break>) -> Outcome {
        match breaks.is_empty() {
            true => Outcome::Preserved,
            false => Outcome::Broken(breaks),
        }
    }

    /// Whether this dimension forces a major version.
    ///
    /// Spec 2: "any dimension broken forces a major version". An unmeasured
    /// dimension forces nothing and proves nothing, which is why
    /// [`Measured::complete`] is a separate question a reader must also ask.
    pub fn forces_major(&self) -> bool {
        matches!(self, Outcome::Broken(_))
    }

    pub fn breaks(&self) -> &[Break] {
        match self {
            Outcome::Broken(breaks) => breaks,
            _ => &[],
        }
    }

    fn word(&self) -> String {
        match self {
            Outcome::Preserved => "preserved".to_string(),
            Outcome::Broken(breaks) => format!("BROKEN, {} of them", breaks.len()),
            Outcome::NotMeasured(why) => format!("not measured: {why}"),
        }
    }
}

/// The six dimensions of one comparison.
///
/// Named fields rather than a list, so a dimension that nothing filled in is a
/// compile error rather than a short report.
#[derive(Clone, Debug)]
pub struct Measured {
    pub classification: Outcome,
    pub instance_validity: Outcome,
    pub consequence: Outcome,
    pub projection: Outcome,
    pub identifier: Outcome,
    pub addressability: Outcome,
}

impl Measured {
    /// The five corpus dimensions, unmeasured, with the overlay dimension the
    /// caller did measure.
    ///
    /// The one route to a report in which a dimension did not run. A candidate
    /// taxonomy that does not resolve is the case: the resolver's refusal is
    /// itself the `addressability` reading when it names an overlay address,
    /// and the other five have no second reading to compare against.
    pub fn against_nothing(addressability: Outcome, why: &str) -> Measured {
        let absent = || Outcome::NotMeasured(why.to_string());
        Measured {
            classification: absent(),
            instance_validity: absent(),
            consequence: absent(),
            projection: absent(),
            identifier: absent(),
            addressability,
        }
    }

    /// The six, in [`DIMENSIONS`] order.
    pub fn dimensions(&self) -> [(&'static str, &Outcome); 6] {
        [
            (DIMENSIONS[0], &self.classification),
            (DIMENSIONS[1], &self.instance_validity),
            (DIMENSIONS[2], &self.consequence),
            (DIMENSIONS[3], &self.projection),
            (DIMENSIONS[4], &self.identifier),
            (DIMENSIONS[5], &self.addressability),
        ]
    }

    /// Whether the measurement forces a major version.
    pub fn forces_major(&self) -> bool {
        self.dimensions()
            .iter()
            .any(|(_, outcome)| outcome.forces_major())
    }

    /// Whether every dimension ran.
    ///
    /// A reader needs this beside [`Measured::forces_major`] and never instead
    /// of it. "No dimension forces a major" out of a run in which five of them
    /// did not happen is a claim about nothing.
    pub fn complete(&self) -> bool {
        !self
            .dimensions()
            .iter()
            .any(|(_, outcome)| matches!(outcome, Outcome::NotMeasured(_)))
    }
}

/// `addressability`: does every overlay address still reach the declaration it
/// was written against?
///
/// The dimension whose subject is the schema rather than the corpus, and the
/// one whose failing case is quiet. A refusal that names an overlay address is
/// the loud half and the caller passes it in, because only the caller holds the
/// refusal. The quiet half is [`headwater_resolve::Founding`]: the candidate
/// resolves, and it resolves because an `add` created the declaration the new
/// base no longer carries.
///
/// # Why a founding is a break and not a warning beside one
///
/// The corpus then holds a declaration that no taxonomy declares. Every reader
/// downstream — the census, the check runner, the generator — reads it and
/// finds it there, so no other dimension can see it: `classification` reports
/// the documents that moved kind and says nothing about the kind that came
/// back. Spec 2 makes any broken dimension force a major version, and a base
/// that removed a declaration an adopter addresses is exactly that.
///
/// The reading is quiet on a corpus that is right, and
/// `headwater_resolve`'s `tests/founded.rs` is where that is a case rather than
/// a sentence: every overlay this repository selects reaches a declaration that
/// is there.
///
/// # Why the quiet half takes two sides, like the other five
///
/// It read one side until
/// [#386](https://github.com/headwater-ai/headwater/issues/386), and the cost of
/// that was a verdict that depended on nothing the release under test did. A
/// founding is a property of the application order rather than of a taxonomy:
/// two overlays that write leaves near each other commute, and the consumer's
/// declared order decides which of them creates the shared parent. So a
/// consumer whose `bundles:` list happens to run the reaching overlay first
/// carries a founding on every resolution, on both sides of every diff, and a
/// one-sided reading reported it as a break of every release. The same consumer
/// with the same two bundles listed the other way was told the same release
/// preserved everything. One legal reordering of the consumer's own list, and
/// the opposite version verdict.
///
/// The comparison the other five dimensions make is the one that answers the
/// question this dimension is for: **did this release remove a declaration the
/// overlay was addressing?** A founding the release the lock names already
/// carried did not. A founding this release introduces did, and it is reported.
///
/// The identity is [`headwater_resolve::FoundingRecord::key`], the operation and
/// the key it makes, and never the source path — the two sides read the same
/// overlay out of two places, so the path differs on a founding that moved
/// nothing.
pub fn addressability(
    before: &[FoundingRecord],
    after: &[FoundingRecord],
    refused: Vec<Break>,
) -> Outcome {
    let carried: BTreeSet<(&str, &str)> = before.iter().map(FoundingRecord::key).collect();
    let mut breaks = refused;
    breaks.extend(
        after
            .iter()
            .filter(|founding| !carried.contains(&founding.key()))
            .map(|founding| Break {
                at: format!("{} in {}", founding.at, founding.source),
                was: format!(
                    "an address into `{}`, which the taxonomy under it declared",
                    founding.founds
                ),
                now: founding.sentence(),
            }),
    );
    Outcome::over(breaks)
}

/// `classification`: does every existing document still resolve to the same
/// kind?
///
/// The subject is every row of the census and not only the typed ones. A
/// document that stopped resolving to a kind is the break this dimension is
/// most about, and a comparison over typed rows alone would drop exactly that
/// case: the row is absent on one side and the two lists still agree on every
/// row they share.
///
/// # It answers with the documents beside the verdict
///
/// For the reason [`instance_validity`] does, and more directly: the key of
/// this comparison is a path already, so the set is the break list read at its
/// own grain. A `kind` step of a migration payload is a remedy for this
/// dimension, and a remedy that nothing held against a measurement would be a
/// claim.
pub fn classification(before: &Census, after: &Census) -> (Outcome, BTreeSet<String>) {
    let breaks = compare(
        &keyed(before, classified),
        &keyed(after, classified),
        "no such file",
    );
    let moved = breaks.iter().map(|entry| entry.at.clone()).collect();
    (Outcome::over(breaks), moved)
}

/// `identifier`: does every identifier still resolve to the same document?
///
/// Over the index rather than over the front matter, because the index is what
/// every later phase reads. Both sides of the index are here: an identifier
/// that stopped being claimed by a typed document and one that started are the
/// same break seen from two directions.
pub fn identifier(before: &Graph, after: &Graph) -> Outcome {
    Outcome::over(compare(&claimed(before), &claimed(after), "unclaimed"))
}

/// `projection`: does every projection produce identical output?
///
/// Over the plan and never over the tree, so the dimension answers about the
/// taxonomy rather than about whatever happens to be committed. A projection
/// that produced no file is compared too, by the reason it stated: a
/// declaration that stopped emitting is a projection change that a comparison
/// of written bytes alone would report as nothing at all.
pub fn projection(before: &Plan, after: &Plan) -> Outcome {
    Outcome::over(compare(&emitted(before), &emitted(after), "not emitted"))
}

/// `consequence`: does every check that passed still pass, and every check that
/// failed still fail?
///
/// Over every instance at every grain, and over the rules that reach a verdict
/// without creating an instance.
///
/// **Two rules of the check layer report a finding and create no instance.**
/// Their subject is the resolved taxonomy rather than a document, so there is
/// nothing for an instance to be created over, and they account against no
/// census row. A comparison of instances alone is therefore silent about every
/// declaration defect that a new base introduces — a control naming a mechanism
/// the engine cannot run, an obligation carrying two dispositions — which is a
/// whole origin of finding missing from the one dimension that is meant to hold
/// all of them. This was measured rather than reasoned: the case in
/// `crates/cli/tests/diff.rs` that separates this dimension from
/// [`instance_validity`] reported `preserved` against a base that had grown
/// exactly such a control.
///
/// **Which rules those are is derived and never listed.** A rule that produced
/// no instance on either side is one. A rule that produced none for another
/// reason — an edge rule over a corpus with no edges — produced no finding
/// either, so its entry reads the same on both sides and moves nothing.
///
/// **The comparison of those findings drops the path and the line.** A finding
/// about the taxonomy carries the path the taxonomy was read from, and the two
/// sides of a diff are read from two places by construction. That path is an
/// injected value, in the way the identity of a run is, and a dimension that
/// compared it would report the location of the artifact as a change the
/// artifact made.
///
/// **The summary word for this dimension is [`dimension_word`] and not
/// [`Outcome::word`].** A break here carries any of the readings [`Movement`]
/// enumerates, and `BROKEN, {n} of them` said one number about five different
/// events: a real failure, a check that declined to decide, a check that
/// stopped instantiating, an instanceless rule that started reporting, and a
/// check that started passing.
/// [#396](https://github.com/headwater-ai/headwater/issues/396).
pub fn consequence(before: &Run, after: &Run) -> Outcome {
    let instanceless: Vec<&'static str> = headwater_check::RULES
        .iter()
        .copied()
        .filter(|rule| !creates_an_instance(rule, before) && !creates_an_instance(rule, after))
        .collect();

    let mut was = verdicts(&before.instances, |_| true);
    let mut now = verdicts(&after.instances, |_| true);
    was.extend(reported(before, &instanceless));
    now.extend(reported(after, &instanceless));
    Outcome::over(compare(&was, &now, "no instance"))
}

fn creates_an_instance(rule: &str, run: &Run) -> bool {
    run.instances.iter().any(|instance| instance.rule == rule)
}

fn reported(run: &Run, rules: &[&'static str]) -> BTreeMap<String, String> {
    rules
        .iter()
        .map(|rule| {
            let lines: Vec<&str> = run
                .findings
                .iter()
                .filter(|finding| finding.rule == *rule)
                .map(|finding| finding.message.as_str())
                .collect();
            let reading = match lines.is_empty() {
                true => "nothing reported".to_string(),
                false => format!("reported: {}", lines.join(" | ")),
            };
            (format!("{rule} over the taxonomy"), reading)
        })
        .collect()
}

/// `instance_validity`: does every existing document still validate?
///
/// The same comparison as [`consequence`] over the document-grained instances
/// alone. The partition is [`Grain`], which is read off the trait a check
/// implements and is never declared beside it, so nothing here is a second list
/// of rules that a new check could fall out of.
///
/// The grain is the right partition because it is the question. An instance
/// whose grain is `Document` covers one document and reads nothing else, so its
/// verdict is a statement about whether that document validates. An edge,
/// neighbourhood or corpus instance is a statement about the corpus between
/// documents, and it belongs to `consequence` alone.
///
/// This dimension is therefore a subset of `consequence` and never disagrees
/// with it: a broken `instance_validity` always breaks `consequence` too. What
/// the split tells a reader is whether the change reached documents or only the
/// graph between them, which is the difference between a payload of rewrites
/// and a payload of re-statements.
///
/// # It answers with the documents beside the verdict, out of one comparison
///
/// A migration payload names what moved, and the question a consumer asks of it
/// is whether the documents that stopped validating are the documents the
/// payload claims. That needs the set, and a second pass over the same
/// instances to build it could disagree with the pass that decided the
/// dimension. So the set falls out of the one comparison that is already here.
///
/// The map from a key to a document is safe for exactly this grain and for no
/// other. An instance whose grain is `Document` reads one document, so its key
/// names one path. An edge instance reads two and a corpus instance reads every
/// row, which is why neither is in this partition.
pub fn instance_validity(before: &Run, after: &Run) -> (Outcome, BTreeSet<String>) {
    let document = |instance: &Instance| instance.grain == Grain::Document;
    let mut over: BTreeMap<String, String> = BTreeMap::new();
    for instance in before
        .instances
        .iter()
        .chain(after.instances.iter())
        .filter(|instance| document(instance))
    {
        if let Some(input) = instance.reads.first() {
            over.insert(key(instance), input.path.clone());
        }
    }
    let breaks = compare(
        &verdicts(&before.instances, document),
        &verdicts(&after.instances, document),
        "no instance",
    );
    let moved = breaks
        .iter()
        .filter_map(|entry| over.get(&entry.at).cloned())
        .collect();
    (Outcome::over(breaks), moved)
}

/// The rules whose instances stopped agreeing, read off an `instance_validity`
/// outcome.
///
/// A caller that wants to know *what kind* of break it is looking at has one
/// honest source for it, and this is that source. [`key`] is what puts the rule
/// at the front of `Break::at`, and the reading of that shape stays in this
/// module: a caller that split the field itself would carry a second copy of a
/// format only [`key`] decides.
///
/// Answers an empty set for a preserved dimension and for one that did not run,
/// which is the same answer as "no rule broke here". A caller that must tell
/// those apart is asking about the [`Outcome`] and matches it.
pub fn broken_rules(outcome: &Outcome) -> BTreeSet<String> {
    let Outcome::Broken(breaks) = outcome else {
        return BTreeSet::new();
    };
    breaks
        .iter()
        .filter_map(|entry| entry.at.split(' ').next())
        .map(str::to_string)
        .collect()
}

/// What one break's `now` reading says happened, for the summary word.
///
/// **The set is derived from the three writers of a reading, and never from a
/// list of strings somebody read out of a report.** A reading under
/// [`instance_validity`] or [`consequence`] is written by exactly three places
/// and by nothing else: [`verdict`] writes `passed`, `skipped: {why}` and
/// `failed: {…}`, one per variant of [`headwater_check::instance::Outcome`],
/// so a fourth verdict is a compile error before it is a mislabelled count;
/// [`reported`] writes `nothing reported` and `reported: {…}`; and [`compare`]
/// writes its `absent` argument, which both dimensions pass as `no instance`.
/// Six readings, and no seventh without a new writer.
///
/// [`verdicts`] joins two instances under one key with ` + `, so a reading can
/// also be a compound of the first three. [`Movement::of`] reads a compound at
/// its severest part, because a word that called the whole break a skip would
/// be a claim about the half of it that failed.
///
/// The variants are declared least severe first, so the derived [`Ord`] is the
/// order [`Movement::of`] takes a maximum over.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Movement {
    /// An instanceless rule that stopped reporting. Only [`consequence`]
    /// compares those, and this is the improving direction of that comparison.
    NoLongerReported,
    /// A check that started passing. An improvement, and it is counted apart
    /// because a total that folded it into `failed` would state the opposite
    /// of what happened.
    NowPassing,
    /// A check that stopped instantiating: one side has no reading at all.
    /// The movement is an absence of measurement rather than a measurement of
    /// a failure, and calling it `failed` is the false statement that
    /// [#396](https://github.com/headwater-ai/headwater/issues/396) reports.
    NoLongerMeasured,
    /// A check that ran and declined to decide.
    /// [#221](https://github.com/headwater-ai/headwater/issues/221).
    Skipped,
    /// A real break: `failed: {…}`, or an instanceless rule that started
    /// reporting.
    Failed,
}

impl Movement {
    fn of(now: &str) -> Movement {
        match Movement::of_one(now) {
            // Every reading a single writer produces whole is recognised
            // whole, so a ` + ` inside a skip reason or a finding message
            // never reaches the split below.
            Movement::Failed => now
                .split(" + ")
                .map(Movement::of_one)
                .max()
                .unwrap_or(Movement::Failed),
            single => single,
        }
    }

    fn of_one(now: &str) -> Movement {
        match now {
            "no instance" => Movement::NoLongerMeasured,
            "nothing reported" => Movement::NoLongerReported,
            "passed" => Movement::NowPassing,
            _ if now.starts_with("skipped: ") => Movement::Skipped,
            _ => Movement::Failed,
        }
    }

    fn word(self) -> &'static str {
        match self {
            Movement::NoLongerReported => "no longer reported",
            Movement::NowPassing => "now passing",
            Movement::NoLongerMeasured => "no longer measured",
            Movement::Skipped => "skipped",
            Movement::Failed => "failed",
        }
    }
}

/// The summary word for a dimension whose breaks are verdict movements.
///
/// `instance_validity` and `consequence` both are: both compare the value
/// space [`Movement`] enumerates, so both carry the same conflation and one
/// function answers for both. A sibling of the old `instance_validity_word`
/// would have left the `no instance` half wrong in each of them.
///
/// Only the non-zero counts are printed, severest first, so the common
/// single-cause report stays one count wide and never says `0 skipped` about a
/// corpus in which nothing was skipped.
///
/// This does not change which breaks are read, only which word each one is
/// counted under. [`Outcome::forces_major`] still answers `true` for a
/// skip-only, absence-only or improvement-only movement, because whether any
/// of those should force a major version is a different question, left open by
/// #221 and untouched here.
fn dimension_word(breaks: &[Break]) -> String {
    let mut counts: BTreeMap<Movement, usize> = BTreeMap::new();
    for entry in breaks {
        *counts.entry(Movement::of(&entry.now)).or_default() += 1;
    }
    let parts: Vec<String> = counts
        .iter()
        .rev()
        .map(|(movement, count)| format!("{count} {}", movement.word()))
        .collect();
    match parts.is_empty() {
        true => "BROKEN".to_string(),
        false => format!("BROKEN, {}", parts.join(" / ")),
    }
}

/// The documents of this corpus that one step of a migration payload names.
///
/// Derived from [`migrate::sites`], which answers the same question at the
/// grain a writer needs. Two readings of "which documents does this step name"
/// is exactly where a report and a write start to disagree, and the report is
/// the half nobody would notice going wrong.
pub fn subjects(step: &Step, census: &Census, overlay: &Adopted) -> BTreeSet<String> {
    migrate::sites(step, census, overlay)
        .into_iter()
        .map(|site| site.key())
        .collect()
}

/// Every key of either map, with the two readings under it.
///
/// `absent` is what a side that holds no reading says, and it is a parameter
/// rather than an empty string so that a break reads as a sentence about the
/// subject of the dimension.
fn compare(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
    absent: &str,
) -> Vec<Break> {
    let mut breaks = Vec::new();
    for at in before.keys().chain(after.keys()).collect::<Vec<_>>() {
        let was = before.get(at).map(String::as_str).unwrap_or(absent);
        let now = after.get(at).map(String::as_str).unwrap_or(absent);
        if was != now {
            breaks.push(Break {
                at: at.clone(),
                was: was.to_string(),
                now: now.to_string(),
            });
        }
    }
    breaks.sort();
    breaks.dedup();
    breaks
}

fn keyed(census: &Census, reading: fn(&Row) -> String) -> BTreeMap<String, String> {
    census
        .rows
        .iter()
        .map(|row| (row.path.clone(), reading(&row.outcome)))
        .collect()
}

/// What the census made of one file, as one word a reader can act on.
///
/// The kind is the whole reading for a typed row, because the derivation that
/// produced it is a route and this dimension asks about the destination. Every
/// other arm names the class alone: a file the walk excluded and a file it could
/// not read are both "not a document of this corpus", and neither becomes one
/// because a taxonomy moved.
fn classified(outcome: &Row) -> String {
    match outcome {
        Row::Typed { kind, .. } => kind.clone(),
        Row::Generated { kind, .. } => match kind {
            Some(kind) => format!("generated, {kind}"),
            None => "generated".to_string(),
        },
        Row::Untyped(_) => "untyped".to_string(),
        Row::Unreadable(_) => "unreadable".to_string(),
        Row::Excluded { pattern, .. } => format!("excluded by `{pattern}`"),
        Row::NotADocument => "not a document".to_string(),
        Row::Unwalkable(_) => "unwalkable".to_string(),
    }
}

fn claimed(graph: &Graph) -> BTreeMap<String, String> {
    graph
        .index
        .typed
        .iter()
        .map(|node| (node.id.clone(), node.path.clone()))
        .collect()
}

/// Every output a plan produced, and every declaration that produced none.
///
/// A declaration that stopped emitting is a projection change, and a comparison
/// of written bytes alone would report it as nothing at all. So the reason an
/// unwritten declaration states is compared beside the bytes of a written one,
/// under the path each names.
///
/// Gathered rather than inserted, for the reason [`verdicts`] gathers: two
/// declarations can name one path, and a map that kept the last would drop the
/// other on both sides at once.
fn emitted(plan: &Plan) -> BTreeMap<String, String> {
    let written = plan
        .outputs
        .iter()
        .map(|output| (output.path.clone(), output.bytes.clone()));
    let unwritten = plan.unwritten.iter().map(|unwritten| {
        (
            unwritten.at.clone(),
            format!("nothing: {}", unwritten.reason),
        )
    });
    let mut gathered: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (at, reading) in written.chain(unwritten) {
        gathered.entry(at).or_default().push(reading);
    }
    gathered
        .into_iter()
        .map(|(at, mut readings)| {
            readings.sort();
            (at, readings.join("\n---\n"))
        })
        .collect()
}

/// Every instance the filter admits, keyed by the rule and what it read.
///
/// The key holds the read set rather than one target path, because two
/// instances of one rule are told apart by what they were handed and by nothing
/// else. An edge-scoped instance reads both endpoints, and a key that named the
/// first would collide two edges out of one document.
///
/// The value is the whole verdict, findings included. Both runs are this one
/// engine over this one tree, so a message differs only when the fact under it
/// does. That is what makes the comparison sensitive enough to catch a rule
/// that started failing for a different reason at the same document.
fn verdicts(instances: &[Instance], admit: impl Fn(&Instance) -> bool) -> BTreeMap<String, String> {
    let mut gathered: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for instance in instances.iter().filter(|instance| admit(instance)) {
        gathered
            .entry(key(instance))
            .or_default()
            .push(verdict(&instance.outcome));
    }
    // Sorted, so that two instances under one key are compared as a set. The
    // order the runner emits them in is fixed, and to depend on it here would
    // make a dimension answer about the order of a list rather than about what
    // the list holds.
    gathered
        .into_iter()
        .map(|(at, mut readings)| {
            readings.sort();
            (at, readings.join(" + "))
        })
        .collect()
}

/// What one instance is told apart by.
///
/// **A key can hold more than one instance, and that is why the map above
/// gathers rather than inserts.** Two edges between one pair of documents are
/// two instances of an edge-scoped rule over one read set, so a key that
/// carried the last one would drop the verdict of the first without saying so.
///
/// **A corpus-grained or taxonomy-grained instance is keyed by its rule alone.**
/// Such an instance reads every row, so its read set is the corpus, and a key
/// built from it would name every document of the repository. What that key
/// would add is a report of the census under a rule name, and the census is
/// what [`classification`] already compares.
fn key(instance: &Instance) -> String {
    if !instance.grain.routes() {
        return format!("{} over the corpus", instance.rule);
    }
    let read: Vec<&str> = instance
        .reads
        .iter()
        .map(|input| input.path.as_str())
        .collect();
    match read.is_empty() {
        true => format!("{} over nothing it could read", instance.rule),
        false => format!("{} at {}", instance.rule, read.join(", ")),
    }
}

fn verdict(outcome: &Verdict) -> String {
    match outcome {
        Verdict::Passed => "passed".to_string(),
        Verdict::Skipped(why) => format!("skipped: {why}"),
        Verdict::Failed(findings) => {
            let lines: Vec<String> = findings
                .iter()
                .map(|finding| format!("{}:{} {}", finding.path, finding.line, finding.message))
                .collect();
            format!("failed: {}", lines.join(" | "))
        }
    }
}

/// One comparison, as a reader receives it.
///
/// It carries the two identities beside the six dimensions because a dimension
/// set with no versions against it is a measurement of nothing. Spec 7 asks the
/// consumer's run to *verify* a publisher's claim, and a verification names
/// what it verified.
#[derive(Clone, Debug)]
pub struct Report {
    pub package: String,
    /// The version this repository takes, from the lock.
    pub from: String,
    /// The version the artifact declares.
    pub to: String,
    /// What became of the base.
    ///
    /// This is the "what changed in the base" line of
    /// [spec 7](../../../../docs/spec/07-distribution-and-federation.md#upgrading),
    /// and it is reported and never gated on. Two publishes of one package
    /// resolve identically and two that differ in a description do not, and
    /// neither fact decides a dimension. See the module comment.
    pub base: Base,
    pub measured: Measured,
    /// The sources the lock records whose bytes on disk no longer hash to what
    /// it recorded, as [`headwater_lock::Lock::moved`] names them.
    ///
    /// The previous side of every dimension is read out of the lock and never
    /// out of these files, so this decides nothing and is never a refusal. It
    /// is here because a reader who is told what the previous side was is owed
    /// where it came from. See [`Caveat::SourcesMoved`].
    pub moved_sources: Vec<String>,
}

/// Something true of this comparison that no dimension is, and that this run
/// could not settle.
///
/// # A caveat and never a refusal
///
/// Both arms below are states a correct run reaches. The publisher shape every
/// case of `crates/cli/tests/diff.rs` uses — edit the package source in place,
/// publish it, diff the artifact against the lock — moves the recorded sources
/// on every run, so refusing on [`Caveat::SourcesMoved`] would refuse the
/// ordinary case. A check that reddens on correct input is one its first reader
/// turns off.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Caveat {
    /// [`Base::Same`] beside a broken `addressability`.
    ///
    /// The five other dimensions read the taxonomy body, and the digest of the
    /// lock covers that body, so a base that resolved to the same text makes
    /// them agree by construction. The founding record is the one reading of
    /// the previous side that sits outside that digest: it is a property of
    /// which operation created a key during the merge, and two merges can reach
    /// identical text by different routes.
    ///
    /// So the pair says the previous side's `founded:` block and the previous
    /// side's taxonomy no longer describe one resolution, or the release moved
    /// a founding without moving a declaration. Nothing this run holds
    /// separates them, which is why both are printed.
    FoundingOutsideTheDigest,
    /// The lock records source files that have since changed on disk.
    SourcesMoved(Vec<String>),
}

impl Caveat {
    /// The line a reader scans for, and never the name of a dimension: the
    /// harness in `crates/cli/tests/diff.rs` selects a dimension by the prefix
    /// of a trimmed line.
    pub fn heading(&self) -> &'static str {
        match self {
            Caveat::FoundingOutsideTheDigest => {
                "the base and the founding record disagree, and this run cannot settle it"
            }
            Caveat::SourcesMoved(_) => {
                "the previous side was read from a lock whose sources have moved"
            }
        }
    }

    /// The body, one line per sentence a reader acts on separately.
    pub fn lines(&self) -> Vec<String> {
        match self {
            Caveat::FoundingOutsideTheDigest => vec![
                "The base resolved to the same text, byte for byte, and `addressability` still \
                 reports a break."
                    .to_string(),
                "Every other dimension reads the taxonomy body, which the digest of the lock \
                 covers, so identical text makes those five agree. The founding record is the one \
                 reading of the previous side that the digest does not cover."
                    .to_string(),
                "Two things reach this state and nothing here separates them. Either the \
                 `founded:` block of the lock no longer describes the taxonomy beside it, which \
                 `headwater taxonomy resolve` rewrites — run it and diff again — or this release \
                 moved which operation creates an address without moving a declaration, which is \
                 the break the line below reports."
                    .to_string(),
            ],
            Caveat::SourcesMoved(paths) => {
                let mut lines = vec![format!(
                    "{} source{} the lock records no longer hash to what it recorded, so the \
                     previous side is the taxonomy the lock carries and not what these files \
                     resolve to now.",
                    paths.len(),
                    match paths.len() {
                        1 => "",
                        _ => "s",
                    }
                )];
                // `moved` and not the bare path: a source file named for a
                // dimension would otherwise read as that dimension's line.
                lines.extend(paths.iter().map(|path| format!("moved  {path}")));
                lines.push(
                    "That is the reading spec 6 asks for, because the lock is the one thing \
                     downstream takes. It is stated here and gated on nowhere."
                        .to_string(),
                );
                lines
            }
        }
    }
}

impl Report {
    /// What spec 2 makes the version bump a consequence of.
    ///
    /// Three answers rather than two. "No dimension forces a major" is a claim
    /// that a run in which five dimensions did not happen has not earned, and
    /// the third arm is what stops this function from making it.
    pub fn bump(&self) -> Bump {
        match (self.measured.forces_major(), self.measured.complete()) {
            (true, _) => Bump::Major,
            (false, true) => Bump::NoMajor,
            (false, false) => Bump::Undecided,
        }
    }

    /// Everything true of this comparison that no dimension is.
    ///
    /// The first arm is derived here rather than by the caller, so that no
    /// caller can build a report that contradicts itself and stay silent about
    /// it. It costs one comparison of two values the report already holds.
    pub fn caveats(&self) -> Vec<Caveat> {
        let mut out = Vec::new();
        if self.base == Base::Same && self.measured.addressability.forces_major() {
            out.push(Caveat::FoundingOutsideTheDigest);
        }
        if !self.moved_sources.is_empty() {
            out.push(Caveat::SourcesMoved(self.moved_sources.clone()));
        }
        out
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("package  {}\n", self.package));
        out.push_str(&format!("taking   {} (the lock)\n", self.from));
        out.push_str(&format!("against  {} (the artifact)\n\n", self.to));
        out.push_str(self.base.sentence());
        out.push_str("\n\n");
        for (name, outcome) in self.measured.dimensions() {
            let word = match (name, outcome) {
                // The two dimensions that compare a verdict. The other four
                // compare a value space with no unmeasured reading in it, so
                // `Outcome::word` is honest for them.
                ("instance_validity" | "consequence", Outcome::Broken(breaks)) => {
                    dimension_word(breaks)
                }
                _ => outcome.word(),
            };
            out.push_str(&format!("  {name:<18} {word}\n"));
        }
        for (name, outcome) in self.measured.dimensions() {
            let breaks = outcome.breaks();
            if breaks.is_empty() {
                continue;
            }
            out.push_str(&format!("\n{name}\n"));
            for entry in breaks {
                out.push_str(&format!("  {}\n", entry.at));
                out.push_str(&format!("    was  {}\n", entry.was));
                out.push_str(&format!("    now  {}\n", entry.now));
            }
        }
        // Between the readings and the verdict, because a caveat is about what
        // the verdict below rests on and a reader meets it in that order.
        for caveat in self.caveats() {
            out.push_str(&format!("\n{}\n", caveat.heading()));
            for line in caveat.lines() {
                out.push_str(&format!("  {line}\n"));
            }
        }
        out.push_str(&format!("\n{}\n", self.bump().sentence()));
        out
    }
}

/// What became of the base, as three states rather than two.
///
/// A candidate that did not resolve has no canonical text, so "it resolved to
/// different text" is a sentence about a resolution that did not happen. The
/// third arm is the difference between a base that moved and a base nobody
/// could read, and a boolean loses it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Base {
    /// The two resolved to the same canonical taxonomy text, byte for byte.
    Same,
    Moved,
    Unresolved,
}

impl Base {
    pub fn sentence(self) -> &'static str {
        match self {
            Base::Same => "the base resolved to the same text, byte for byte",
            Base::Moved => "the base resolved to different text",
            Base::Unresolved => "the base did not resolve, so there is no second text to compare",
        }
    }
}

/// What the measurement requires of the version number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bump {
    /// A dimension is broken, and spec 2 makes that a major version.
    Major,
    /// Every dimension ran and none is broken. This decides that no *major* is
    /// required and it decides nothing else: minor and patch are not
    /// measurable here, because the six dimensions ask what a change did to a
    /// corpus and neither of those two is a question about one.
    NoMajor,
    /// Nothing is broken among the dimensions that ran, and not all of them
    /// ran.
    Undecided,
}

impl Bump {
    pub fn sentence(self) -> &'static str {
        match self {
            Bump::Major => "a dimension is broken, so this change requires a major version",
            Bump::NoMajor => {
                "no dimension is broken, so nothing here requires a major version. Whether it is \
                 a minor or a patch is not a question these six ask"
            }
            Bump::Undecided => {
                "no dimension that ran is broken, and not every dimension ran, so this run \
                 decides nothing about the version"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use headwater_check::instance::Input;

    fn edge(rule: &'static str, ends: [&str; 2], outcome: Verdict) -> Instance {
        Instance {
            rule,
            grain: Grain::Edge,
            reads: ends
                .iter()
                .map(|path| Input {
                    path: (*path).to_string(),
                    digest: None,
                })
                .collect(),
            outcome,
        }
    }

    /// Two edges between one pair of documents are two instances of one
    /// edge-scoped rule over one read set.
    ///
    /// A map that inserted under that key would keep the last verdict and drop
    /// the first, so a taxonomy change that flipped the dropped one would
    /// report nothing at all. Nothing about that failure is visible in a count:
    /// both sides lose the same instance, and the dimension says `preserved`.
    #[test]
    fn two_instances_over_one_read_set_are_both_compared() {
        let rule = "relation.endpoint.not_permitted";
        let was = [
            edge(rule, ["a.md", "b.md"], Verdict::Passed),
            edge(rule, ["a.md", "b.md"], Verdict::Skipped("no window".into())),
        ];
        // The second edge is the one that moves, and it is the one an insert
        // would have kept, so this fails in the direction that hides a drop.
        let now = [
            edge(rule, ["a.md", "b.md"], Verdict::Skipped("no window".into())),
            edge(rule, ["a.md", "b.md"], Verdict::Skipped("no window".into())),
        ];
        let breaks = compare(
            &verdicts(&was, |_| true),
            &verdicts(&now, |_| true),
            "no instance",
        );
        assert_eq!(breaks.len(), 1, "{breaks:?}");
        assert!(breaks[0].was.contains("passed"), "{breaks:?}");
        assert!(!breaks[0].now.contains("passed"), "{breaks:?}");
    }

    /// The order the runner emits two instances in is not a dimension.
    #[test]
    fn the_order_of_two_instances_under_one_key_is_not_a_difference() {
        let rule = "relation.endpoint.not_permitted";
        let skipped = || Verdict::Skipped("no window".into());
        let was = [
            edge(rule, ["a.md", "b.md"], Verdict::Passed),
            edge(rule, ["a.md", "b.md"], skipped()),
        ];
        let now = [
            edge(rule, ["a.md", "b.md"], skipped()),
            edge(rule, ["a.md", "b.md"], Verdict::Passed),
        ];
        assert!(compare(
            &verdicts(&was, |_| true),
            &verdicts(&now, |_| true),
            "no instance",
        )
        .is_empty());
    }

    /// A corpus-grained instance reads every row, and its key names none of
    /// them. See [`key`].
    #[test]
    fn a_corpus_grained_instance_is_keyed_by_its_rule_alone() {
        let instance = Instance {
            rule: "identifier.claimed_twice",
            grain: Grain::Corpus,
            reads: vec![
                Input {
                    path: "a.md".into(),
                    digest: None,
                },
                Input {
                    path: "b.md".into(),
                    digest: None,
                },
            ],
            outcome: Verdict::Passed,
        };
        assert_eq!(key(&instance), "identifier.claimed_twice over the corpus");
    }

    /// An empty break list is a preserved dimension and never a broken one with
    /// nothing in it.
    #[test]
    fn a_dimension_with_no_break_is_preserved() {
        assert!(matches!(Outcome::over(Vec::new()), Outcome::Preserved));
        assert!(matches!(
            Outcome::over(vec![Break {
                at: "a".into(),
                was: "b".into(),
                now: "c".into(),
            }]),
            Outcome::Broken(_)
        ));
    }

    /// A report whose candidate did not resolve decides nothing about a
    /// version, and it never reads as compatible.
    #[test]
    fn an_unmeasured_run_decides_nothing_about_the_version() {
        let report = Report {
            package: "acme/fixture".into(),
            from: "1.0.0".into(),
            to: "2.0.0".into(),
            base: Base::Unresolved,
            moved_sources: Vec::new(),
            measured: Measured::against_nothing(Outcome::Preserved, "nothing resolved"),
        };
        assert!(!report.measured.complete());
        assert_eq!(report.bump(), Bump::Undecided);
        assert!(report
            .render()
            .contains("decides nothing about the version"));
        assert!(report.caveats().is_empty(), "{:?}", report.caveats());
    }

    fn measured(addressability: Outcome) -> Measured {
        Measured {
            classification: Outcome::Preserved,
            instance_validity: Outcome::Preserved,
            consequence: Outcome::Preserved,
            projection: Outcome::Preserved,
            identifier: Outcome::Preserved,
            addressability,
        }
    }

    fn reported(base: Base, addressability: Outcome, moved_sources: Vec<String>) -> Report {
        Report {
            package: "acme/fixture".into(),
            from: "1.0.0".into(),
            to: "1.0.0".into(),
            base,
            moved_sources,
            measured: measured(addressability),
        }
    }

    fn a_break() -> Outcome {
        Outcome::over(vec![Break {
            at: "add.kinds.zz_thing.voice".into(),
            was: "the previous release carried no such founding".into(),
            now: "it makes `kinds.zz_thing`".into(),
        }])
    }

    /// The contradiction the report could not previously see.
    ///
    /// A base that resolved to the same text makes the five body dimensions
    /// agree by construction, so a broken `addressability` beside it says the
    /// founding record and the taxonomy of the previous side came from two
    /// resolutions. The verdict below still says major, and the caveat is what
    /// tells a reader what that verdict rests on.
    #[test]
    fn an_identical_base_beside_a_broken_addressability_is_reported_as_unsettled() {
        let report = reported(Base::Same, a_break(), Vec::new());
        assert_eq!(report.caveats(), vec![Caveat::FoundingOutsideTheDigest]);
        let rendered = report.render();
        assert!(rendered.contains("this run cannot settle it"), "{rendered}");
        assert!(
            rendered.contains("`headwater taxonomy resolve` rewrites"),
            "{rendered}"
        );
        // The caveat precedes the verdict it qualifies.
        let caveat = rendered
            .find("cannot settle it")
            .expect("the caveat is there");
        let verdict = rendered
            .find("requires a major version")
            .expect("the verdict is there");
        assert!(caveat < verdict, "{rendered}");
    }

    /// The three honest shapes stay quiet. A base that moved explains a
    /// founding that moved, and a preserved `addressability` contradicts
    /// nothing whatever the base did.
    #[test]
    fn a_report_that_does_not_contradict_itself_states_no_caveat() {
        for (base, addressability) in [
            (Base::Moved, a_break()),
            (Base::Same, Outcome::Preserved),
            (Base::Moved, Outcome::Preserved),
        ] {
            let report = reported(base, addressability, Vec::new());
            assert!(
                report.caveats().is_empty(),
                "{base:?}: {:?}",
                report.caveats()
            );
        }
    }

    /// An `addressability` that did not run is not a break, so it contradicts
    /// nothing. `NotMeasured` reads as `Preserved` to a boolean and this is
    /// what holds the two apart here.
    #[test]
    fn an_addressability_that_did_not_run_contradicts_nothing() {
        let report = reported(
            Base::Same,
            Outcome::NotMeasured("no reading".into()),
            Vec::new(),
        );
        assert!(report.caveats().is_empty(), "{:?}", report.caveats());
    }

    /// The provenance of the previous side, stated and never gated on. It is
    /// independent of the contradiction, so a report can carry both.
    #[test]
    fn a_lock_whose_sources_moved_says_so_beside_any_other_caveat() {
        let moved = vec!["packages/acme/taxonomy.yml".to_string()];
        let report = reported(Base::Moved, Outcome::Preserved, moved.clone());
        assert_eq!(report.caveats(), vec![Caveat::SourcesMoved(moved.clone())]);
        let rendered = report.render();
        assert!(rendered.contains("whose sources have moved"), "{rendered}");
        assert!(
            rendered.contains("packages/acme/taxonomy.yml"),
            "{rendered}"
        );
        assert!(rendered.contains("1 source the lock records"), "{rendered}");

        let both = reported(Base::Same, a_break(), moved.clone());
        assert_eq!(
            both.caveats(),
            vec![
                Caveat::FoundingOutsideTheDigest,
                Caveat::SourcesMoved(moved)
            ]
        );
    }

    /// No caveat line begins with the name of a dimension.
    ///
    /// `crates/cli/tests/diff.rs` selects a dimension by the prefix of a
    /// trimmed line and takes the first match, so a caveat line that opened
    /// with one would answer for the summary block.
    #[test]
    fn no_caveat_line_reads_as_a_dimension_line() {
        for caveat in [
            Caveat::FoundingOutsideTheDigest,
            Caveat::SourcesMoved(vec!["identifier.yml".into()]),
        ] {
            let mut lines = vec![caveat.heading().to_string()];
            lines.extend(caveat.lines());
            for line in lines {
                for name in DIMENSIONS {
                    assert!(
                        !line.trim_start().starts_with(name),
                        "`{line}` reads as the `{name}` line"
                    );
                }
            }
        }
    }

    /// Every reading a writer of a `now` value can produce, and the word each
    /// one is counted under.
    ///
    /// The list is the derivation stated on [`Movement`] read back: three from
    /// [`verdict`], one per variant of the check layer's verdict; two from
    /// [`reported`]; and the `absent` argument both dimensions pass to
    /// [`compare`]. The `taxonomy diff` suite exercises four of the six today,
    /// and this case is what stops the other two from being classified by
    /// nobody until a corpus happens to produce them.
    #[test]
    fn every_reading_a_break_can_carry_has_a_word() {
        for (now, word) in [
            ("passed", "now passing"),
            ("skipped: no pattern set for it", "skipped"),
            ("failed: docs/a.md:1 a message", "failed"),
            ("nothing reported", "no longer reported"),
            ("reported: a message", "failed"),
            ("no instance", "no longer measured"),
        ] {
            assert_eq!(Movement::of(now).word(), word, "the reading `{now}`");
        }
    }

    /// A key holding two instances reads as a ` + ` join, and the severest
    /// part decides.
    ///
    /// [`verdicts`] writes that join, so it is a reading a break can carry and
    /// not a hypothetical. A word that read the whole string would fall
    /// through to `failed` for every one of these, including the two in which
    /// nothing failed.
    #[test]
    fn a_joined_reading_is_read_at_its_severest_part() {
        for (now, word) in [
            ("failed: docs/a.md:1 a message + passed", "failed"),
            ("passed + skipped: a reason", "skipped"),
            ("passed + passed", "now passing"),
            ("failed: a + skipped: b", "failed"),
            // A ` + ` inside a skip reason is not a join, because the whole
            // string is already a reading a writer produces.
            ("skipped: a + b are both absent", "skipped"),
        ] {
            assert_eq!(Movement::of(now).word(), word, "the reading `{now}`");
        }
    }

    /// The word prints only the movements that occurred, severest first.
    #[test]
    fn the_summary_word_prints_no_zero_and_orders_by_severity() {
        let entry = |now: &str| Break {
            at: format!("rule at {now}"),
            was: "passed".to_string(),
            now: now.to_string(),
        };
        assert_eq!(
            dimension_word(&[entry("no instance"), entry("no instance")]),
            "BROKEN, 2 no longer measured",
            "a corpus in which nothing was skipped never prints a skip count"
        );
        assert_eq!(
            dimension_word(&[
                entry("passed"),
                entry("no instance"),
                entry("skipped: a reason"),
                entry("failed: docs/a.md:1 a message"),
                entry("nothing reported"),
            ]),
            "BROKEN, 1 failed / 1 skipped / 1 no longer measured / 1 now passing / 1 no longer \
             reported"
        );
    }

    /// Naming a movement changes no verdict.
    ///
    /// `Outcome::forces_major` reads the shape of the outcome and never the
    /// word, so a dimension broken only by an absence of measurement or by an
    /// improvement still forces a major version. Whether it should is the
    /// question #221 left open and this change does not answer.
    #[test]
    fn a_word_that_names_no_failure_still_forces_a_major() {
        let outcome = Outcome::over(vec![Break {
            at: "rule at docs/a.md".to_string(),
            was: "passed".to_string(),
            now: "no instance".to_string(),
        }]);
        assert!(outcome.forces_major());
    }
}
