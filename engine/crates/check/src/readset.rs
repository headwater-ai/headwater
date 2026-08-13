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
//! them with the three run-level components beside it.
//!
//! # What the union is for, and why a merge is then an ordinary change
//!
//! [Q21](../../../../docs/spec/09-decisions.md#q21--terminological-succession-and-validity-under-merge)
//! rules that validity is not preserved under merge: "two changes that are each
//! valid against the merge base can produce an invalid corpus, and no run
//! against either branch tip reports it". That is write skew, and a read set is
//! what a serializable database detects it with. Git holds none, which is why
//! it merges two such changes without a conflict.
//!
//! A published read set closes that. Given the read set of a run and the tree a
//! merge produced, a gate compares each input against the tree it now has. An
//! input whose hash moved invalidates the instances that read it, and an empty
//! result means that the verdict still applies. Neither answer needs a run.
//!
//! The three run-level components are not decoration. A verdict is about one
//! state of the corpus **and one day**: a run that evaluated a windowed
//! expectation a month ago says nothing about today, whatever the tree did. And
//! a lock that moved voids every result at once, because the lock digest is a
//! component of every cache key.
//!
//! # Where it is published, and what that costs
//!
//! [`Coverage`](crate::Coverage) reports the size beside its own numbers, and
//! the union itself is an artifact the CLI writes: a section of the report, and
//! a file for a gate that has to compare it against a later tree.
//!
//! Q21's open half asks whether publishing this is free. The measurement is one
//! subtraction and it is in
//! [13 — Open obligations](../../../../docs/spec/13-open-obligations.md): the
//! artifact is one line per document, against a corpus whose documents average
//! four orders of magnitude more than that. It is free at this corpus's scale
//! and the number that would decide an adopter's is not this repository's.
//!
//! # An input with no content hash is in the set, and it is counted apart
//!
//! [`crate::cache`] refuses to key an instance whose input carries no digest,
//! because a key over a hash that does not exist is the correctness bug spec 12
//! names. A read set has the same problem and it cannot take the same way out:
//! an input this run read has to appear, or the union is not the union. So it
//! appears with no hash, and the count of them is reported. A gate that meets
//! one cannot decide that the verdict survives, and a summary that hid them
//! would let it decide exactly that.

use crate::context::Date;
use crate::instance::{Input, Instance};

/// The union of what one run read, with the three components that are about
/// the run rather than about a document.
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
    pub fn of(
        lock: &str,
        clock: Date,
        versions: Vec<(&'static str, u32)>,
        instances: &[Instance],
    ) -> Self {
        let mut inputs: Vec<Input> = Vec::new();
        for instance in instances {
            for input in &instance.reads {
                match inputs.binary_search_by(|known| known.path.as_str().cmp(&input.path)) {
                    Ok(_) => {}
                    Err(at) => inputs.insert(at, input.clone()),
                }
            }
        }
        ReadSet {
            lock: lock.to_string(),
            clock,
            versions,
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
    pub fn summary(&self) -> String {
        match self.unhashed() {
            0 => format!("{} documents in the read set", self.inputs.len()),
            unhashed => format!(
                "{} documents in the read set, {unhashed} of them with no content hash",
                self.inputs.len()
            ),
        }
    }

    /// The artifact: everything a gate needs to decide whether a verdict
    /// survives a change, and nothing that is not an input to one.
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(out, "lock {}", self.lock);
        let _ = writeln!(out, "clock {}", self.clock.render());
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

    fn instance(rule: &'static str, reads: Vec<Input>) -> Instance {
        Instance::of(rule, crate::Grain::Document, reads, Outcome::Passed)
    }

    /// Two instances over one document contribute one entry, and the entries
    /// come out in path order however the instances arrived.
    #[test]
    fn the_union_holds_each_document_once_in_path_order() {
        let set = ReadSet::of(
            "sha256:lock",
            date(),
            vec![("r", 1)],
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
            Vec::new(),
            &[instance("r", vec![Input::new("a.md", None)])],
        );
        assert_eq!(set.unhashed(), 1);
        assert!(set.render().contains("input a.md -"));
        assert!(set.summary().contains("1 of them with no content hash"));
    }

    /// The artifact carries the three run-level components spec 12 names
    /// beside the documents.
    #[test]
    fn the_artifact_carries_the_lock_the_clock_and_the_versions() {
        let text = ReadSet::of(
            "sha256:lock",
            date(),
            vec![("first.rule", 1), ("second.rule", 3)],
            &[instance("first.rule", vec![Input::new("a.md", Some("d"))])],
        )
        .render();
        assert_eq!(
            text,
            "lock sha256:lock\nclock 2026-08-12\nversion first.rule 1\nversion second.rule 3\ninput a.md d\n"
        );
    }
}
