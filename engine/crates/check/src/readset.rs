// SPDX-License-Identifier: Apache-2.0
//! The read set: the union of the in-scope inputs behind one run.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict)
//! fixes what it is and what it is for. "The **read set** of a run is the union
//! of the in-scope inputs that produced its results. That is the content hash of
//! every document and edge that an instance read. It also holds the taxonomy
//! lock hash, the check versions, and the injected values. Every run already
//! computes it, one instance at a time. What is new is that the run reports the
//! union beside its coverage numbers."
//!
//! So this module adds no measurement. [`crate::Instance`] has carried a path
//! and a content hash per input since the cache landed, and this is the union of
//! them with the run-level components beside it.
//!
//! # What the union is for, and the one test it supports
//!
//! [Q21](../../../../docs/spec/09-decisions.md#q21--terminological-succession-and-validity-under-merge)
//! rules that validity is not preserved under merge: "two changes that are each
//! valid against the merge base can produce an invalid corpus, and no run
//! against either branch tip reports it". That is write skew, and a read set is
//! what a serializable database detects it with. Git holds none, which is why
//! it merges two such changes without a conflict.
//!
//! Spec 12 rules what a gate reads, and the ruling is one test:
//! [`crate::gate`] holds the published set against a later tree, and it reads
//! **nothing that the set does not list**. That is what makes the answer cheap,
//! and it is also the limit of what the answer is worth. An input whose hash
//! moved voids the verdict. A file the set does not list is invisible to the
//! comparison, so the artifact carries, in lines of its own, every reason a
//! comparison over a list cannot decide.
//!
//! There are three such reasons and this module writes all three down.
//!
//! **A barrier.** A corpus-grained instance decides its verdict from the
//! *extent* of the census rather than from the contents of any member of it.
//! `identifier.claimed_twice` fires on the presence of a second claimant, so
//! its verdict rests on the absence of a document rather than on the bytes of
//! a listed one. No list of `(path, digest)` pairs states "and no other
//! document exists". So each such rule gets a `barrier` line, and a gate that
//! reads one reports that the verdict does not carry.
//!
//! **The clock.** A verdict is about one state of the corpus **and one day**. A
//! run that evaluated a windowed expectation a month ago says nothing about
//! today, whatever the tree did. The rules that read the clock get a `windowed`
//! line, and a gate that reads one on a later day reports that the verdict does
//! not carry.
//!
//! **An input with no content hash.** [`crate::cache`] refuses to key an
//! instance whose input carries no digest, because a key over a hash that does
//! not exist is the correctness bug spec 12 names. A read set cannot take the
//! same way out: an input this run read has to appear, or the union is not the
//! union. So it appears with no hash, it is counted apart, and a gate that
//! meets one cannot decide that the verdict survives.
//!
//! The lock is the fourth component, and it is a comparison rather than a
//! refusal. A lock that moved voids every result at once, because the lock
//! digest is a component of every cache key.
//!
//! # Where it is published, and what that costs
//!
//! [`Coverage`](crate::Coverage) reports the size beside its own numbers, and
//! the union itself is an artifact the CLI writes: a section of the report, and
//! a file that `headwater gate` compares against a later tree.
//!
//! Q21's open half asks whether publishing this is free. The measurement is one
//! subtraction and it is in
//! [13 — Open obligations](../../../../docs/spec/13-open-obligations.md): the
//! artifact is one line per document, against a corpus whose documents average
//! four orders of magnitude more than that. It is free at this corpus's scale
//! and the number that would decide an adopter's is not this repository's.
//!
//! # The union is a union, so a gate decides about the run
//!
//! One thing this artifact does not carry is which instance read which input.
//! The union is taken across every instance, and the mapping is gone by the
//! time the file is written. A gate therefore answers one question about the
//! whole run rather than one question per instance, and [`crate::gate`] says so
//! in the words it prints. A per-instance answer wants a per-instance artifact,
//! and this is not one.

use crate::context::Date;
use crate::instance::{Input, Instance, Outcome};
use crate::scope::Grain;

/// One rule as a run served it: which edition ran, and whether it read the
/// clock.
///
/// Both come off the trait the check implements, by way of [`crate::Serves`].
/// Neither is stated beside the scope, for the reason [`crate::scope`] gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rule {
    pub name: &'static str,
    pub version: u32,
    pub needs_clock: bool,
    /// Whether this rule read the version its document stood at before the
    /// change. See [`ReadSet::scoped`].
    pub needs_prior: bool,
}

/// The union of what one run read, with the components that are about the run
/// rather than about a document.
#[derive(Clone, Debug)]
pub struct ReadSet {
    /// The digest of the taxonomy lock every result rests on.
    pub lock: String,
    /// The injected clock. It is here whether or not a rule declared it,
    /// because a reader of this artifact is asking what state a verdict is
    /// about, and the answer is a tree and a day.
    pub clock: Date,
    /// Each rule and the edition of it that ran, in [`crate::RULES`] order.
    pub versions: Vec<(&'static str, u32)>,
    /// The rules that put a corpus-grained instance in this run, in
    /// [`crate::RULES`] order.
    ///
    /// Derived from the instances rather than from the registry, because a
    /// rule that generated nothing read nothing, and a barrier line is a claim
    /// about what ran.
    pub barriers: Vec<&'static str>,
    /// The rules that read the injected clock and put an instance in this run,
    /// in [`crate::RULES`] order. Same derivation, and the same reason.
    pub windowed: Vec<&'static str>,
    /// The rules that read the version a document stood at before the change,
    /// and reached a verdict in this run.
    ///
    /// The third reason a comparison over listed inputs cannot decide, and it
    /// arrives for the clock's reason one input further out. A verdict of such
    /// a rule is about a tree, a day **and a change**. The change is not a
    /// corpus path, so no line of this artifact names it, and two runs over one
    /// tree under two changes reach two verdicts that every listed hash agrees
    /// with.
    ///
    /// The derivation is a verdict rather than an instance, which is what keeps
    /// this list empty in every full-corpus run: every instance of such a rule
    /// skips there, and an artifact of a run that decided nothing has nothing
    /// for a gate to refuse.
    pub scoped: Vec<&'static str>,
    /// Every input any instance read, once each, in path order.
    pub inputs: Vec<Input>,
}

impl ReadSet {
    /// The union over the instances of one run.
    ///
    /// The digests come from the instances rather than from a second pass over
    /// the census, for the reason the instances take them from the census: two
    /// passes over one corpus can disagree, and a read set that disagreed with
    /// the cache keys would report a state that no run evaluated.
    pub fn of(lock: &str, clock: Date, rules: &[Rule], instances: &[Instance]) -> Self {
        let mut inputs: Vec<Input> = Vec::new();
        for instance in instances {
            for input in &instance.reads {
                match inputs.binary_search_by(|known| known.path.as_str().cmp(&input.path)) {
                    Ok(_) => {}
                    Err(at) => inputs.insert(at, input.clone()),
                }
            }
        }
        let barrier = |rule: &Rule| {
            instances
                .iter()
                .any(|instance| instance.rule == rule.name && instance.grain == Grain::Corpus)
        };
        let ran = |rule: &Rule| instances.iter().any(|instance| instance.rule == rule.name);
        let decided = |rule: &Rule| {
            instances.iter().any(|instance| {
                instance.rule == rule.name && !matches!(instance.outcome, Outcome::Skipped(_))
            })
        };
        ReadSet {
            lock: lock.to_string(),
            clock,
            versions: rules.iter().map(|rule| (rule.name, rule.version)).collect(),
            barriers: rules
                .iter()
                .filter(|rule| barrier(rule))
                .map(|rule| rule.name)
                .collect(),
            windowed: rules
                .iter()
                .filter(|rule| rule.needs_clock && ran(rule))
                .map(|rule| rule.name)
                .collect(),
            scoped: rules
                .iter()
                .filter(|rule| rule.needs_prior && decided(rule))
                .map(|rule| rule.name)
                .collect(),
            inputs,
        }
    }

    /// Inputs this run read and could not hash. See the module comment: they
    /// are in the union and they are counted apart, because a gate cannot
    /// decide anything about one.
    pub fn unhashed(&self) -> usize {
        self.inputs
            .iter()
            .filter(|input| input.digest.is_none())
            .count()
    }

    /// The one line that goes beside the coverage numbers.
    ///
    /// It carries no digest. A recorded report holds this line, and a hash of a
    /// paragraph belongs in the artifact rather than in a file that a reader
    /// re-blesses whenever a sentence moves.
    ///
    /// A barrier is named here rather than left to the artifact alone, because
    /// a count of documents reads as a verdict that a gate could carry, and a
    /// run that holds a barrier has no such verdict to offer.
    pub fn summary(&self) -> String {
        let mut line = match self.unhashed() {
            0 => format!("{} documents in the read set", self.inputs.len()),
            unhashed => format!(
                "{} documents in the read set, {unhashed} of them with no content hash",
                self.inputs.len()
            ),
        };
        if !self.barriers.is_empty() {
            let barriers = self.barriers.len();
            let noun = match barriers {
                1 => "barrier",
                _ => "barriers",
            };
            line.push_str(&format!(
                ", and {barriers} {noun} that no gate carries across a merge"
            ));
        }
        line
    }

    /// The artifact: everything a gate needs to decide whether a verdict
    /// survives a change, and nothing that is not an input to one.
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(out, "lock {}", self.lock);
        let _ = writeln!(out, "clock {}", self.clock.render());
        // Above the versions, because these two lines decide the answer on
        // their own. A reader who stops at the first block has read why this
        // verdict does not carry, where it does not.
        for rule in &self.barriers {
            let _ = writeln!(out, "barrier {rule}");
        }
        for rule in &self.windowed {
            let _ = writeln!(out, "windowed {rule}");
        }
        for rule in &self.scoped {
            let _ = writeln!(out, "change-scoped {rule}");
        }
        for (rule, version) in &self.versions {
            let _ = writeln!(out, "version {rule} {version}");
        }
        for input in &self.inputs {
            let _ = match &input.digest {
                Some(digest) => writeln!(out, "input {} {digest}", input.path),
                // Written rather than dropped, and written so that a reader
                // cannot mistake it for a hash. An input with no digest is the
                // one entry that stops a gate deciding anything.
                None => writeln!(out, "input {} -", input.path),
            };
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::Outcome;

    fn date() -> Date {
        Date::parse("2026-08-12").expect("a date")
    }

    fn rule(name: &'static str, needs_clock: bool) -> Rule {
        Rule {
            name,
            version: 1,
            needs_clock,
            needs_prior: false,
        }
    }

    /// A rule that reads the version a document stood at before the change.
    fn prior_reading(name: &'static str) -> Rule {
        Rule {
            name,
            version: 1,
            needs_clock: false,
            needs_prior: true,
        }
    }

    fn skipped(rule: &'static str, reads: Vec<Input>) -> Instance {
        Instance::skipped(
            rule,
            Grain::Document,
            reads,
            crate::scope::CHANGE_SCOPED_ONLY,
        )
    }

    fn instance(rule: &'static str, reads: Vec<Input>) -> Instance {
        Instance::of(rule, Grain::Document, reads, Outcome::Passed)
    }

    fn over_the_corpus(rule: &'static str) -> Instance {
        Instance::of(
            rule,
            Grain::Corpus,
            vec![Input::new("a.md", Some("sha256:a"))],
            Outcome::Passed,
        )
    }

    /// Two instances over one document contribute one entry, and the entries
    /// come out in path order however the instances arrived.
    #[test]
    fn the_union_holds_each_document_once_in_path_order() {
        let set = ReadSet::of(
            "sha256:lock",
            date(),
            &[rule("r", false)],
            &[
                instance("r", vec![Input::new("b.md", Some("sha256:b"))]),
                instance(
                    "s",
                    vec![
                        Input::new("b.md", Some("sha256:b")),
                        Input::new("a.md", Some("sha256:a")),
                    ],
                ),
            ],
        );
        let paths: Vec<&str> = set.inputs.iter().map(|i| i.path.as_str()).collect();
        assert_eq!(paths, vec!["a.md", "b.md"]);
    }

    /// An input the walk never hashed is in the union and is counted apart.
    #[test]
    fn an_input_with_no_digest_is_published_and_counted() {
        let set = ReadSet::of(
            "sha256:lock",
            date(),
            &[],
            &[instance("r", vec![Input::new("a.md", None)])],
        );
        assert_eq!(set.unhashed(), 1);
        assert!(set.render().contains("input a.md -"));
        assert!(set.summary().contains("1 of them with no content hash"));
    }

    /// The artifact carries the lock, the clock and the versions that spec 12
    /// names, beside the documents.
    #[test]
    fn the_artifact_carries_the_lock_the_clock_and_the_versions() {
        let text = ReadSet::of(
            "sha256:lock",
            date(),
            &[rule("first.rule", false), rule("second.rule", false)],
            &[instance("first.rule", vec![Input::new("a.md", Some("d"))])],
        )
        .render();
        assert_eq!(
            text,
            "lock sha256:lock\nclock 2026-08-12\nversion first.rule 1\nversion second.rule 1\ninput a.md d\n"
        );
    }

    /// A rule with a corpus-grained instance is written down as a barrier, and
    /// the summary states what that costs a gate.
    #[test]
    fn a_corpus_grained_instance_writes_a_barrier_line() {
        let set = ReadSet::of(
            "sha256:lock",
            date(),
            &[rule("wide.rule", false)],
            &[over_the_corpus("wide.rule")],
        );
        assert_eq!(set.barriers, vec!["wide.rule"]);
        assert!(set.render().contains("\nbarrier wide.rule\n"));
        assert!(set.summary().contains("1 barrier that no gate carries"));
    }

    /// A rule that declares the clock and generated an instance is windowed. A
    /// rule that declares it and generated nothing read nothing, so it is not.
    #[test]
    fn windowed_names_the_rules_that_read_the_clock_and_ran() {
        let set = ReadSet::of(
            "sha256:lock",
            date(),
            &[rule("dated.rule", true), rule("absent.rule", true)],
            &[instance("dated.rule", vec![Input::new("a.md", Some("d"))])],
        );
        assert_eq!(set.windowed, vec!["dated.rule"]);
        assert!(!set.render().contains("windowed absent.rule"));
    }

    /// A rule that generated a document-grained instance is no barrier,
    /// however many documents the run holds.
    #[test]
    fn a_document_grained_rule_is_no_barrier() {
        let set = ReadSet::of(
            "sha256:lock",
            date(),
            &[rule("narrow.rule", false)],
            &[instance("narrow.rule", vec![Input::new("a.md", Some("d"))])],
        );
        assert!(set.barriers.is_empty());
        assert!(!set.render().contains("barrier"));
        assert_eq!(set.summary(), "1 documents in the read set");
    }

    /// A run that reached a verdict from a prior version says so, and a run
    /// that skipped every such instance says nothing.
    ///
    /// The second half is what keeps every artifact of this repository the
    /// bytes it was: a full-corpus run skips every instance of a prior-reading
    /// rule, so the line is absent and a gate reads what it read before. The
    /// first half is the one a change-scoped run publishes, and a gate refuses
    /// it for the reason it refuses a barrier: a change is not a corpus path,
    /// so no listed hash decides anything about one.
    #[test]
    fn a_verdict_that_rests_on_a_change_is_named_and_a_skipped_one_is_not() {
        let inputs = vec![Input::new("a.md", Some("sha256:one"))];
        let decided = ReadSet::of(
            "sha256:lock",
            date(),
            &[prior_reading("warrant.promoted")],
            &[instance("warrant.promoted", inputs.clone())],
        );
        assert_eq!(decided.scoped, vec!["warrant.promoted"]);
        assert!(decided
            .render()
            .contains("change-scoped warrant.promoted\n"));

        let skipped_only = ReadSet::of(
            "sha256:lock",
            date(),
            &[prior_reading("warrant.promoted")],
            &[skipped("warrant.promoted", inputs)],
        );
        assert!(
            skipped_only.scoped.is_empty(),
            "a run that decided nothing published a verdict a gate has to refuse"
        );
        assert!(!skipped_only.render().contains("change-scoped"));
    }
}
