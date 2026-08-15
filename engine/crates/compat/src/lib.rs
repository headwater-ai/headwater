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
use std::collections::BTreeMap;

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

/// `classification`: does every existing document still resolve to the same
/// kind?
///
/// The subject is every row of the census and not only the typed ones. A
/// document that stopped resolving to a kind is the break this dimension is
/// most about, and a comparison over typed rows alone would drop exactly that
/// case: the row is absent on one side and the two lists still agree on every
/// row they share.
pub fn classification(before: &Census, after: &Census) -> Outcome {
    Outcome::over(compare(
        &keyed(before, classified),
        &keyed(after, classified),
        "no such file",
    ))
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
pub fn instance_validity(before: &Run, after: &Run) -> Outcome {
    let document = |instance: &Instance| instance.grain == Grain::Document;
    Outcome::over(compare(
        &verdicts(&before.instances, document),
        &verdicts(&after.instances, document),
        "no instance",
    ))
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
    written.chain(unwritten).collect()
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
    let mut out = BTreeMap::new();
    for instance in instances.iter().filter(|instance| admit(instance)) {
        let read: Vec<&str> = instance
            .reads
            .iter()
            .map(|input| input.path.as_str())
            .collect();
        let at = match read.is_empty() {
            true => format!("{} over the corpus", instance.rule),
            false => format!("{} at {}", instance.rule, read.join(", ")),
        };
        out.insert(at, verdict(&instance.outcome));
    }
    out
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
    /// Whether the two resolved to the same canonical taxonomy text.
    ///
    /// This is the "what changed in the base" line of
    /// [spec 7](../../../../docs/spec/07-distribution-and-federation.md#upgrading),
    /// and it is reported and never gated on. Two publishes of one package
    /// resolve identically and two that differ in a description do not, and
    /// neither fact decides a dimension. See the module comment.
    pub base_moved: bool,
    pub measured: Measured,
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

    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("package  {}\n", self.package));
        out.push_str(&format!("taking   {} (the lock)\n", self.from));
        out.push_str(&format!("against  {} (the artifact)\n\n", self.to));
        out.push_str(match self.base_moved {
            true => "the base resolved to different text\n\n",
            false => "the base resolved to the same text, byte for byte\n\n",
        });
        for (name, outcome) in self.measured.dimensions() {
            out.push_str(&format!("  {name:<18} {}\n", outcome.word()));
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
        out.push_str(&format!("\n{}\n", self.bump().sentence()));
        out
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
