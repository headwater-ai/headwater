// SPDX-License-Identifier: Apache-2.0
//! The content-addressed cache, and the key that makes it sound.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#determinism-concretely)
//! states the key in one sentence: "the content hashes of the in-scope inputs,
//! the taxonomy lock hash, the check version, and the injected values. A key
//! that omits an input is a correctness bug, not a performance bug." It also
//! names the cache a
//! [correctness root](../../../../docs/spec/12-check-layer.md#the-correctness-roots):
//! "a cache that can change a verdict is a store under another name."
//!
//! So the standing test is a differential rather than a benchmark.
//! `tests/cache.rs` runs one corpus with no cache, cold, and warm, and holds
//! the three renders to each other. `headwater check --no-cache` is the same
//! comparison from outside.
//!
//! # A cache hit is not a fact about the corpus, so no report carries one
//!
//! Spec 12 asks a run to record, per document, "which instances were created,
//! which ran, which were served from cache, and which were skipped with a
//! reason". It also fixes the verdict: "same corpus, same lock, same injected
//! clock, byte-identical output". A hit count in the report would break the
//! second ask to satisfy the first, because it is a function of what is on
//! this machine's disk and of nothing the first sentence names. Two people
//! with one tree would then read two reports.
//!
//! The two are kept apart rather than traded off. The cache accounting is a
//! [`Report`] on the run, and the CLI writes one line of it to standard error,
//! which is not the verdict stream. What goes to standard output is what the
//! corpus, the lock and the injected values decide, and a cache moves none of
//! them.
//!
//! For the same reason a cache does not make a run partial. Every instance is
//! created, every instance has an outcome, and coverage counts what it counted
//! before. A run that evaluates less of the corpus is `--changed-only`, which
//! is [#58](https://github.com/headwater-ai/headwater/issues/58), and that one
//! does have to decide what coverage means over a partial pass.
//!
//! # What the key covers, and the one component that has no instance
//!
//! Four components, and three of them are here. The in-scope inputs arrive as
//! [`Input`]s carrying the census digest of each file. The lock digest is
//! [`headwater_lock::digest`], through the caller. The check version is a
//! constant on the scope trait.
//!
//! **The injected values have no instance.** No check receives one: the clock
//! and the prior version are the two spec 12 names, and
//! [`crate::scope`] carries neither, because #54 removed the fields that
//! nothing enforced. There is no empty slot in the key standing in for them
//! either, for the same reason. What that leaves is a trap for whoever adds
//! the first injected value, and
//! [13 — Open obligations](../../../../docs/spec/13-open-obligations.md)
//! carries it rather than a comment here.
//!
//! Two further components are in the key that spec 12's sentence does not
//! name, and both are identity rather than input. The **rule** and the
//! **target** tell two instances apart that read the same documents: one pair
//! of documents can carry two relations, so their read sets are equal and
//! their results are not.
//!
//! # Why a skipped instance is never stored
//!
//! A cache holds verdicts. [`Outcome::Skipped`] is the statement that no
//! verdict was reached, and spec 4 wants the reason visible on every run. So a
//! skip is decided again each time, which costs one evaluation and can never
//! serve a stale reason from a disk.
//!
//! # Failing toward re-running
//!
//! Spec 12: "a false invalidation costs one run. A false survival ships an
//! invalid corpus with a green report." Every doubtful case here takes the
//! first cost. An input with no digest is not keyed. A record this engine
//! cannot read is a miss. A cache file that will not parse is an empty cache.
//! None of them is an error, and none of them can change a verdict.

use crate::finding::{Finding, Severity};
use crate::instance::{Input, Outcome};
use crate::scope::Scope;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Where a repository keeps the cache, beside the lock that keys it.
pub const CACHE: &str = ".headwater/cache/checks";

/// The format of the cache file. A reader that meets a later one starts empty
/// rather than guessing, which costs one full run.
pub const FORMAT: &str = "headwater check cache 1";

/// What a run did with its cache.
///
/// No render prints this. See the module comment: it is a fact about a disk,
/// and the verdict is a fact about a corpus.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// Instances whose outcome came from the cache.
    pub hits: usize,
    /// Instances that were keyed and evaluated.
    pub misses: usize,
    /// Instances that no key covers, so they are evaluated on every run. An
    /// input with no content hash is the case, and a skipped outcome is the
    /// other.
    pub unkeyed: usize,
}

impl Report {
    /// The one line the CLI writes to standard error.
    pub fn render(&self) -> String {
        format!(
            "{} served from cache, {} evaluated, {} not keyed\n",
            self.hits, self.misses, self.unkeyed
        )
    }
}

/// The cache of one run.
#[derive(Clone, Debug)]
pub struct Cache {
    /// The lock digest every key carries, and `None` when this cache is off.
    /// A cache with no lock cannot key anything, which is what makes
    /// `--no-cache` a path that computes no key rather than one that computes
    /// a key and ignores it.
    lock: Option<String>,
    /// What was on disk when the run started.
    found: BTreeMap<String, String>,
    /// What this run keyed, and the only thing [`Cache::write`] writes. So the
    /// file is a function of the corpus rather than a pile that grows: an
    /// entry for an instance that no longer exists is dropped by not being
    /// used.
    used: BTreeMap<String, String>,
    report: Report,
}

impl Cache {
    /// A cache that keys nothing and stores nothing, which is `--no-cache`.
    pub fn disabled() -> Self {
        Cache {
            lock: None,
            found: BTreeMap::new(),
            used: BTreeMap::new(),
            report: Report::default(),
        }
    }

    /// The cache of a repository, against the taxonomy lock that keys it.
    ///
    /// A file that is absent, unreadable, or written by another engine reads
    /// as an empty cache. None of the three is an error: the cost is one full
    /// run, and the alternative is a verb that refuses to check a corpus
    /// because of a file that holds no corpus content.
    pub fn at(root: &Path, lock: &str) -> Self {
        let found = std::fs::read_to_string(Self::path(root))
            .ok()
            .map(|text| read(&text))
            .unwrap_or_default();
        Cache {
            lock: Some(lock.to_string()),
            found,
            used: BTreeMap::new(),
            report: Report::default(),
        }
    }

    pub fn path(root: &Path) -> PathBuf {
        root.join(CACHE)
    }

    pub fn report(&self) -> Report {
        self.report
    }

    /// Write what this run used, and say nothing about a failure to.
    ///
    /// A cache that cannot be written is a run with no cache next time, which
    /// is slower and never wrong. Reporting it would put a message about a
    /// disk in the middle of a report about a corpus.
    pub fn write(&self, root: &Path) {
        let Some(_) = &self.lock else {
            return;
        };
        let path = Self::path(root);
        if let Some(parent) = path.parent() {
            if std::fs::create_dir_all(parent).is_err() {
                return;
            }
        }
        let mut text = String::from(FORMAT);
        text.push('\n');
        for (key, record) in &self.used {
            text.push_str(key);
            text.push('\t');
            text.push_str(record);
            text.push('\n');
        }
        let _ = std::fs::write(path, text);
    }

    /// The outcome of one instance, from this cache or from the check.
    ///
    /// The closure is what runs when the cache cannot answer, and it is the
    /// only place a check is called. So the cached path and the fresh path
    /// produce one value of one type, and a difference between them is a
    /// difference this function makes rather than one two call sites drifted
    /// into.
    pub(crate) fn outcome<F>(
        &mut self,
        rule: &'static str,
        version: u32,
        scope: Scope,
        target: &str,
        reads: &[Input],
        evaluate: F,
    ) -> Outcome
    where
        F: FnOnce() -> Outcome,
    {
        let Some(key) = self.key(rule, version, scope, target, reads) else {
            self.report.unkeyed += 1;
            return evaluate();
        };

        if let Some(record) = self.found.get(&key).cloned() {
            if let Some(outcome) = decode(rule, &record) {
                self.report.hits += 1;
                self.used.insert(key, record);
                return outcome;
            }
        }

        let outcome = evaluate();
        match encode(&outcome) {
            Some(record) => {
                self.report.misses += 1;
                self.used.insert(key, record);
            }
            // A skip. It is not stored, and it is not a miss either: nothing
            // about it will ever come from a cache.
            None => self.report.unkeyed += 1,
        }
        outcome
    }

    /// The key of one instance, and nothing when one cannot be computed.
    ///
    /// The text below is what is hashed, and it is written out in full rather
    /// than folded into one string, so that a reader can see every component
    /// spec 12 names and check that none is missing.
    fn key(
        &self,
        rule: &'static str,
        version: u32,
        scope: Scope,
        target: &str,
        reads: &[Input],
    ) -> Option<String> {
        let lock = self.lock.as_ref()?;
        let mut text = String::from("headwater check key 1\n");
        text.push_str(&format!("lock {lock}\n"));
        text.push_str(&format!("rule {rule}\n"));
        text.push_str(&format!("version {version}\n"));
        text.push_str(&format!(
            "scope {} body={}\n",
            scope.grain().name(),
            scope.needs_body()
        ));
        // Escaped for the reason a record is: a target or a path is corpus
        // content, and a newline inside one would otherwise let a document
        // write a line of this text and claim another instance's key.
        text.push_str(&format!("target {}\n", escape(target)));
        for input in reads {
            // An input the walk never read. The result cannot be keyed on a
            // hash that does not exist, and inventing one is the correctness
            // bug spec 12 names. So this instance is evaluated on every run.
            let digest = input.digest.as_ref()?;
            text.push_str(&format!("input {} {digest}\n", escape(&input.path)));
        }
        Some(headwater_hash::hex(text.as_bytes()))
    }
}

/// One outcome as a record, and nothing for an outcome a cache does not hold.
///
/// The rule is not written, because the key already fixes it and a record that
/// carried it could disagree with the key that found it. The obligation is not
/// written for a stronger reason: `crate::run` stamps it from the control that
/// names the rule, so a record that carried one would be a second place the
/// binding lives, which is the drift [`crate::register`] exists to prevent.
fn encode(outcome: &Outcome) -> Option<String> {
    match outcome {
        Outcome::Passed => Some("passed".to_string()),
        Outcome::Skipped(_) => None,
        Outcome::Failed(finding) => Some(format!(
            "failed\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            finding.severity,
            finding.line,
            finding.column,
            finding.fixable,
            escape(&finding.path),
            escape(&finding.message),
            escape(&finding.remediation),
        )),
    }
}

/// One record as an outcome, and nothing for a record this engine cannot read.
fn decode(rule: &'static str, record: &str) -> Option<Outcome> {
    let mut fields = record.split('\t');
    match fields.next()? {
        "passed" => match fields.next() {
            None => Some(Outcome::Passed),
            Some(_) => None,
        },
        "failed" => {
            let severity = match fields.next()? {
                "error" => Severity::Error,
                "warn" => Severity::Warn,
                "info" => Severity::Info,
                _ => return None,
            };
            let finding = Finding {
                rule,
                severity,
                obligation: None,
                line: fields.next()?.parse().ok()?,
                column: fields.next()?.parse().ok()?,
                fixable: match fields.next()? {
                    "true" => true,
                    "false" => false,
                    _ => return None,
                },
                path: unescape(fields.next()?),
                message: unescape(fields.next()?),
                remediation: unescape(fields.next()?),
            };
            match fields.next() {
                None => Some(Outcome::Failed(Box::new(finding))),
                Some(_) => None,
            }
        }
        _ => None,
    }
}

/// The file as entries. A line this reader cannot make sense of is dropped,
/// which costs one evaluation and can never change a verdict.
fn read(text: &str) -> BTreeMap<String, String> {
    let mut lines = text.lines();
    if lines.next() != Some(FORMAT) {
        return BTreeMap::new();
    }
    lines
        .filter_map(|line| {
            let (key, record) = line.split_once('\t')?;
            Some((key.to_string(), record.to_string()))
        })
        .collect()
}

/// A field, with the two characters the record format spends made writable.
fn escape(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
}

fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        match characters.next() {
            Some('t') => out.push('\t'),
            Some('n') => out.push('\n'),
            Some('\\') => out.push('\\'),
            // A sequence this writer never produces. Kept as written, because
            // a reader that guessed would return a message that differs from
            // the one the check would produce.
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scope::Scope;

    fn finding() -> Finding {
        Finding {
            rule: "test.rule",
            severity: Severity::Warn,
            obligation: None,
            path: "docs/spec/12-check-layer.md".to_string(),
            line: 12,
            column: 3,
            message: "a message with a\ttab and a\nnewline and a \\ in it".to_string(),
            remediation: "do the thing".to_string(),
            fixable: true,
        }
    }

    /// Every field of a finding survives the file, and the two the runner owns
    /// come back the way the runner sets them.
    #[test]
    fn a_failed_outcome_round_trips_through_a_record() {
        let record = encode(&Outcome::Failed(Box::new(finding()))).expect("a verdict is stored");
        assert!(!record.contains('\n'), "a record is one line: {record}");
        let Some(Outcome::Failed(back)) = decode("test.rule", &record) else {
            panic!("the record did not read back");
        };
        assert_eq!(*back, finding());
    }

    #[test]
    fn a_passed_outcome_round_trips_and_a_skip_is_never_stored() {
        assert!(matches!(
            decode("r", &encode(&Outcome::Passed).expect("stored")),
            Some(Outcome::Passed)
        ));
        assert_eq!(encode(&Outcome::Skipped("a reason")), None);
    }

    /// A record from a later engine, a truncated one, and a corrupted one are
    /// each a miss rather than a wrong answer.
    #[test]
    fn a_record_this_engine_cannot_read_is_a_miss() {
        for record in [
            "failed\tcritical\t1\t1\ttrue\tp\tm\tr",
            "failed\terror\tnot-a-line\t1\ttrue\tp\tm\tr",
            "failed\terror\t1\t1\tmaybe\tp\tm\tr",
            "failed\terror\t1\t1\ttrue\tp\tm",
            "failed\terror\t1\t1\ttrue\tp\tm\tr\tone-more",
            "passed\tand-something-else",
            "reused",
            "",
        ] {
            assert!(decode("r", record).is_none(), "{record} read as an outcome");
        }
    }

    /// A file from another engine is an empty cache and never a refusal.
    #[test]
    fn a_file_this_engine_did_not_write_reads_as_an_empty_cache() {
        assert!(read("headwater check cache 2\nk\tpassed\n").is_empty());
        assert!(read("").is_empty());
        assert_eq!(
            read(&format!("{FORMAT}\nk\tpassed\nno-tab-here\n")).len(),
            1
        );
    }

    fn inputs(digest: Option<&str>) -> Vec<Input> {
        vec![Input::new("a.md", digest)]
    }

    fn cache() -> Cache {
        Cache::at(Path::new("/nonexistent"), "sha256:lock")
    }

    /// Each component of the key changes it, which is the property that makes
    /// the cache incapable of changing a verdict.
    #[test]
    fn every_component_of_the_key_moves_it() {
        let scope = Scope::document(false);
        let base = cache()
            .key("r", 1, scope, "a.md", &inputs(Some("sha256:one")))
            .expect("a key");

        let others = [
            cache().key("other", 1, scope, "a.md", &inputs(Some("sha256:one"))),
            cache().key("r", 2, scope, "a.md", &inputs(Some("sha256:one"))),
            cache().key(
                "r",
                1,
                Scope::document(true),
                "a.md",
                &inputs(Some("sha256:one")),
            ),
            cache().key("r", 1, Scope::edge(), "a.md", &inputs(Some("sha256:one"))),
            cache().key("r", 1, scope, "b.md", &inputs(Some("sha256:one"))),
            cache().key("r", 1, scope, "a.md", &inputs(Some("sha256:two"))),
            cache().key(
                "r",
                1,
                scope,
                "a.md",
                &[Input::new("b.md", Some("sha256:one"))],
            ),
            Cache::at(Path::new("/nonexistent"), "sha256:other").key(
                "r",
                1,
                scope,
                "a.md",
                &inputs(Some("sha256:one")),
            ),
        ];
        for (index, other) in others.iter().enumerate() {
            assert_ne!(
                Some(&base),
                other.as_ref(),
                "component {index} is not keyed"
            );
        }
    }

    /// An input with no content hash produces no key, so its instance is
    /// evaluated on every run. The alternative is a key over a hash that does
    /// not exist, which is the correctness bug spec 12 names.
    #[test]
    fn an_input_with_no_digest_is_not_keyed() {
        assert_eq!(
            cache().key("r", 1, Scope::document(false), "a.md", &inputs(None)),
            None
        );
    }

    /// A disabled cache computes no key at all, so `--no-cache` is a path that
    /// cannot read or write an entry rather than one that ignores what it read.
    #[test]
    fn a_disabled_cache_keys_nothing() {
        assert_eq!(
            Cache::disabled().key("r", 1, Scope::document(false), "a.md", &inputs(Some("d"))),
            None
        );
    }
}
