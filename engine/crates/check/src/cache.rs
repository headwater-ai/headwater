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
//! before. That is the whole of what a partial run would have had to decide,
//! and [#58](https://github.com/headwater-ai/headwater/issues/58) found that
//! there is nothing left for one to do: the work a `--changed-only` flag would
//! scope is the work this module already skips, and it derives what moved from
//! the bytes rather than from a list that a caller supplies.
//!
//! # What the key covers, and how the fourth component arrived
//!
//! Four components, and all four are here now. The in-scope inputs arrive as
//! [`Input`]s carrying the census digest of each file. The lock digest is
//! [`headwater_lock::digest`], through the caller. The check version is a
//! constant on the scope trait.
//!
//! **The injected values used to have no instance**, and
//! [13 — Open obligations](../../../../docs/spec/13-open-obligations.md)
//! carried the trap that left for whoever added the first one. The clock is
//! that first one. A windowed participation expectation reads `ctx.now`, and a
//! key without it serves yesterday's verdict today — invisibly, because the
//! `--no-cache` differential holds one value of the clock on both sides of the
//! comparison.
//!
//! The key carries the clock **exactly when the scope declares it**, and the
//! declaration is the one [`crate::scope`] already enforces on the view. So the
//! date joins the key of an instance that could read it and stays out of the
//! key of every instance that could not, which is what keeps a warm run warm
//! for the rules that no calendar can move. A scope that declares the clock and
//! reaches this function without one is not keyed at all, on the rule the rest
//! of this module follows: fail toward re-running.
//!
//! **The prior version is the second injected value**, and it arrives on the
//! same terms. Spec 12: "the content hash of that version joins the cache key
//! like any other input." A key without it would serve the verdict of a run
//! whose change carried a different version of the document, and the
//! `--no-cache` differential cannot see that either, because both sides of that
//! comparison hold one change. The three states of the input write three
//! different lines, so an added document, an unchanged one and a modified one
//! never share an entry.
//!
//! Two further components are in the key that spec 12's sentence does not
//! name, and both are identity rather than input. The **rule** and the
//! **target** tell two instances apart that read the same documents: one pair
//! of documents can carry two relations, so their read sets are equal and
//! their results are not.
//!
//! # The sixth component: what a resolver said, for an instance that asked one
//!
//! A read set is a list of corpus paths and their hashes. An **external
//! anchor** names something that is not a corpus path, so no entry of that list
//! moves when the thing an anchor points at moves. The identity does not move
//! either, and that is the part that is easy to miss: a path anchor normalizes
//! to the text its author wrote, an unbound target falls back to that same
//! text, so the file's deletion leaves every component of the key where it was.
//! A cached run then reports zero unresolved targets over a tree that an
//! uncached run reports two on, which is
//! [OBL-repo-0117](../../../../docs/obligations/0117-a-cached-verdict-about-an-anchor-survives-the-change-that-falsifies-it.md).
//!
//! So an instance whose subject is a resolved target keys on
//! [`headwater_graph::Target::resolution`] beside the identity. The two
//! candidate shapes were **refuse the key** and **let the resolver state a
//! digest**, and this is the second one at the grain of one anchor: what the
//! run read is the answer a resolver gave about *this* string, rather than the
//! state of a tree that string could have named. The resolvers run in phase A
//! on every run and phase A is never cached, so that answer is already in hand
//! when the key is computed and it costs nothing to name.
//!
//! **Nothing loses a key over this, and that distinction matters.** Refusing
//! the key is the branch below for an input with no digest, and it makes an
//! instance permanently unkeyed and permanently re-evaluated. This component
//! only ever *divides* a key, so a rule stays as keyed as it was: what changes
//! is that two states of one anchor stop sharing an entry.
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

use crate::change::Prior;
use crate::context::Date;
use crate::finding::{Finding, Severity};
use crate::instance::{Input, Outcome};
use crate::patch::Patch;
use crate::scope::Scope;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Where a repository keeps the cache, beside the lock that keys it.
pub const CACHE: &str = ".headwater/cache/checks";

/// The format of the cache file. A reader that meets a later one starts empty
/// rather than guessing, which costs one full run.
///
/// Edition 2 carries the patch. A finding without its patch is a finding that
/// `check --fix` would not act on, so a warm run and a cold one would write
/// different files. That is the cache changing a result, one layer out from the
/// verdict the differential in `tests/cache.rs` holds.
pub const FORMAT: &str = "headwater check cache 2";

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

    /// An instance the runner decided without asking a check.
    ///
    /// A skip is never stored, so there is nothing here to key and nothing to
    /// serve. It is accounted anyway, because `hits + misses + unkeyed` is the
    /// instance count of a run, and an instance in none of the three is one
    /// that a reader who checks the arithmetic cannot find. The two callers are
    /// the two skips [`crate::scope`] decides before a view exists: a typed row
    /// that carries no document, and a prior-reading rule in a run with no
    /// change.
    pub(crate) fn undecided(&mut self) {
        self.report.unkeyed += 1;
    }

    /// The outcome of one instance, from this cache or from the check.
    ///
    /// The closure is what runs when the cache cannot answer, and it is the
    /// only place a check is called. So the cached path and the fresh path
    /// produce one value of one type, and a difference between them is a
    /// difference this function makes rather than one two call sites drifted
    /// into.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn outcome<F>(
        &mut self,
        rule: &'static str,
        version: u32,
        scope: Scope,
        target: &str,
        reads: &[Input],
        clock: Option<Date>,
        prior: Option<Prior<'_>>,
        resolution: Option<&str>,
        evaluate: F,
    ) -> Outcome
    where
        F: FnOnce() -> Outcome,
    {
        let Some(key) = self.key(
            rule, version, scope, target, reads, clock, prior, resolution,
        ) else {
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
    #[allow(clippy::too_many_arguments)]
    fn key(
        &self,
        rule: &'static str,
        version: u32,
        scope: Scope,
        target: &str,
        reads: &[Input],
        clock: Option<Date>,
        prior: Option<Prior<'_>>,
        resolution: Option<&str>,
    ) -> Option<String> {
        let lock = self.lock.as_ref()?;
        let mut text = String::from("headwater check key 1\n");
        text.push_str(&format!("lock {lock}\n"));
        text.push_str(&format!("rule {rule}\n"));
        text.push_str(&format!("version {version}\n"));
        text.push_str(&format!(
            "scope {} body={} phase_a={} clock={} prior={}\n",
            scope.grain().name(),
            scope.needs_body(),
            scope.needs_phase_a(),
            scope.needs_clock(),
            scope.needs_prior()
        ));
        // The one injected value, and it is written exactly when the scope
        // admits it to the view. A scope that declares the clock and was handed
        // none cannot be keyed: the alternative is a key over an input that the
        // instance did read and that nothing in the key names.
        if scope.needs_clock() {
            text.push_str(&format!("clock {}\n", clock?.render()));
        }
        // The second injected value, on the first one's terms. Spec 12: "the
        // content hash of that version joins the cache key like any other
        // input". The three states of the input write three different lines,
        // and a scope that declares the input and was handed nothing is not
        // keyed at all, which is the same refusal the clock takes one line up.
        if scope.needs_prior() {
            text.push_str(&format!("prior {}\n", prior?.key()));
        }
        // Escaped for the reason a record is: a target or a path is corpus
        // content, and a newline inside one would otherwise let a document
        // write a line of this text and claim another instance's key.
        text.push_str(&format!("target {}\n", escape(target)));
        // What something outside the corpus told this run. See the module
        // comment: an anchor target is in no read set, because a read set is a
        // list of corpus paths and an anchor names something that is not one.
        // The line is absent for an instance that read no such thing, so a
        // scope that reaches no resolver keys exactly as it did.
        if let Some(resolution) = resolution {
            text.push_str(&format!("resolution {}\n", escape(resolution)));
        }
        for input in reads {
            // An input the walk never read. The result cannot be keyed on a
            // hash that does not exist, and inventing one is the correctness
            // bug spec 12 names. So this instance is evaluated on every run.
            let digest = input.digest.as_ref()?;
            text.push_str(&format!("input {} {digest}\n", escape(&input.path)));
        }
        Some(headwater_hash::hex(text.as_bytes()))
    }

    /// The key of an instance that read nothing outside the corpus, which is
    /// every scope but the edge.
    ///
    /// A shorthand for the tests below, so that the one component a resolver
    /// decides is written out only where it is the subject of the test.
    #[cfg(test)]
    fn plain_key(
        &self,
        rule: &'static str,
        version: u32,
        scope: Scope,
        target: &str,
        reads: &[Input],
        clock: Option<Date>,
    ) -> Option<String> {
        self.key(rule, version, scope, target, reads, clock, None, None)
    }
}

/// One outcome as a record, and nothing for an outcome a cache does not hold.
///
/// The rule is not written, because the key already fixes it and a record that
/// carried it could disagree with the key that found it. The obligation is not
/// written for a stronger reason: `crate::run` stamps it from the control that
/// names the rule, so a record that carried one would be a second place the
/// binding lives, which is the drift [`crate::register`] exists to prevent.
/// A verdict may hold several findings, so the record states how many and then
/// writes six fields for each, and the patch after them. The count is what lets
/// a reader tell a truncated record from a complete one without a second
/// separator character that every field would then have to escape.
///
/// The patch opens with a word that says its shape, and each shape has a fixed
/// number of fields after that word. So the reader knows how far the finding
/// runs without a second count, and a shape this reader does not know is a
/// record it drops rather than a record it half-reads.
fn encode(outcome: &Outcome) -> Option<String> {
    match outcome {
        Outcome::Passed => Some("passed".to_string()),
        Outcome::Skipped(_) => None,
        Outcome::Failed(findings) => {
            let mut record = format!("failed\t{}", findings.len());
            for finding in findings {
                record.push_str(&format!(
                    "\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    finding.severity,
                    finding.line,
                    finding.column,
                    escape(&finding.path),
                    escape(&finding.message),
                    escape(&finding.remediation),
                    encode_patch(finding.patch.as_ref()),
                ));
            }
            Some(record)
        }
    }
}

/// A patch as the fields of a record, opening with the word that says its
/// shape.
fn encode_patch(patch: Option<&Patch>) -> String {
    match patch {
        None => "none".to_string(),
        Some(Patch::Text {
            path,
            start,
            end,
            expect,
            replacement,
        }) => format!(
            "text\t{start}\t{end}\t{}\t{}\t{}",
            escape(path),
            escape(expect),
            escape(replacement)
        ),
        Some(Patch::Half { path, relation, id }) => format!(
            "half\t{}\t{}\t{}",
            escape(path),
            escape(relation),
            escape(id)
        ),
    }
}

/// The patch a record carries, and nothing for a shape this reader does not
/// know.
///
/// The outer `Option` is the read: `None` means the record is unreadable and
/// the entry is dropped. The inner one is the finding's own, and `none` is the
/// ordinary case for a rule that offers no patch.
fn decode_patch<'a>(fields: &mut impl Iterator<Item = &'a str>) -> Option<Option<Patch>> {
    match fields.next()? {
        "none" => Some(None),
        "text" => Some(Some(Patch::Text {
            start: fields.next()?.parse().ok()?,
            end: fields.next()?.parse().ok()?,
            path: unescape(fields.next()?),
            expect: unescape(fields.next()?),
            replacement: unescape(fields.next()?),
        })),
        "half" => Some(Some(Patch::Half {
            path: unescape(fields.next()?),
            relation: unescape(fields.next()?),
            id: unescape(fields.next()?),
        })),
        _ => None,
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
            let count: usize = fields.next()?.parse().ok()?;
            // A verdict with no findings is a pass, and this format never
            // writes one. A record that claims zero came from somewhere else.
            if count == 0 {
                return None;
            }
            let mut findings = Vec::with_capacity(count);
            for _ in 0..count {
                let severity = match fields.next()? {
                    "error" => Severity::Error,
                    "warn" => Severity::Warn,
                    "info" => Severity::Info,
                    _ => return None,
                };
                findings.push(Finding {
                    rule,
                    severity,
                    obligation: None,
                    line: fields.next()?.parse().ok()?,
                    column: fields.next()?.parse().ok()?,
                    path: unescape(fields.next()?),
                    message: unescape(fields.next()?),
                    remediation: unescape(fields.next()?),
                    patch: decode_patch(&mut fields)?,
                });
            }
            match fields.next() {
                None => Some(Outcome::Failed(findings)),
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
    use crate::context::Date;
    use crate::scope::Scope;
    use headwater_yaml::Mapping;

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
            patch: Some(Patch::Text {
                path: "docs/spec/12-check-layer.md".to_string(),
                start: 41,
                end: 50,
                expect: "behaviour".to_string(),
                replacement: "behavior".to_string(),
            }),
        }
    }

    /// Every field of a finding survives the file, and the two the runner owns
    /// come back the way the runner sets them.
    #[test]
    fn a_failed_outcome_round_trips_through_a_record() {
        let record = encode(&Outcome::Failed(vec![finding()])).expect("a verdict is stored");
        assert!(!record.contains('\n'), "a record is one line: {record}");
        let Some(Outcome::Failed(back)) = decode("test.rule", &record) else {
            panic!("the record did not read back");
        };
        assert_eq!(back, vec![finding()]);
    }

    /// A verdict with several findings comes back whole and in order.
    ///
    /// A record that held one of them would make a warm run report less than a
    /// cold one, which is the cache changing a verdict.
    #[test]
    fn every_finding_of_one_verdict_survives_the_record() {
        let mut second = finding();
        second.line = 40;
        second.message = "another\tone".to_string();
        let outcome = Outcome::Failed(vec![finding(), second.clone()]);
        let record = encode(&outcome).expect("a verdict is stored");
        let Some(Outcome::Failed(back)) = decode("test.rule", &record) else {
            panic!("the record did not read back");
        };
        assert_eq!(back, vec![finding(), second]);
    }

    #[test]
    fn a_passed_outcome_round_trips_and_a_skip_is_never_stored() {
        assert!(matches!(
            decode("r", &encode(&Outcome::Passed).expect("stored")),
            Some(Outcome::Passed)
        ));
        assert_eq!(encode(&Outcome::Skipped("a reason".to_string())), None);
    }

    /// A record from a later engine, a truncated one, and a corrupted one are
    /// each a miss rather than a wrong answer.
    #[test]
    fn a_record_this_engine_cannot_read_is_a_miss() {
        for record in [
            "failed\t1\tcritical\t1\t1\tp\tm\tr\tnone",
            "failed\t1\terror\tnot-a-line\t1\tp\tm\tr\tnone",
            "failed\t1\terror\t1\t1\tp\tm\tr",
            "failed\t1\terror\t1\t1\tp\tm\tr\tnone\tone-more",
            "failed\t2\terror\t1\t1\tp\tm\tr\tnone",
            "failed\t0",
            "failed\tmany\terror\t1\t1\tp\tm\tr\tnone",
            "failed\terror\t1\t1\tp\tm\tr\tnone",
            // A patch shape this engine does not know, and one whose fields
            // run out. Neither is half-read: the record is dropped.
            "failed\t1\terror\t1\t1\tp\tm\tr\tsomething-else\tx",
            "failed\t1\terror\t1\t1\tp\tm\tr\ttext\t3",
            "failed\t1\terror\t1\t1\tp\tm\tr\ttext\tnot-an-offset\t4\tp\ta\tb",
            "failed\t1\terror\t1\t1\tp\tm\tr\thalf\tp\trel",
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
        assert!(read("headwater check cache 3\nk\tpassed\n").is_empty());
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

    fn day(text: &str) -> Option<Date> {
        Some(Date::parse(text).expect("a date"))
    }

    /// Each component of the key changes it, which is the property that makes
    /// the cache incapable of changing a verdict.
    #[test]
    fn every_component_of_the_key_moves_it() {
        let scope = Scope::document(false, false, false, false);
        let base = cache()
            .plain_key("r", 1, scope, "a.md", &inputs(Some("sha256:one")), None)
            .expect("a key");

        let others = [
            cache().plain_key("other", 1, scope, "a.md", &inputs(Some("sha256:one")), None),
            cache().plain_key("r", 2, scope, "a.md", &inputs(Some("sha256:one")), None),
            cache().plain_key(
                "r",
                1,
                Scope::document(true, false, false, false),
                "a.md",
                &inputs(Some("sha256:one")),
                None,
            ),
            cache().plain_key(
                "r",
                1,
                Scope::document(false, true, false, false),
                "a.md",
                &inputs(Some("sha256:one")),
                None,
            ),
            cache().plain_key(
                "r",
                1,
                Scope::edge(false),
                "a.md",
                &inputs(Some("sha256:one")),
                None,
            ),
            cache().plain_key(
                "r",
                1,
                Scope::neighbourhood(false),
                "a.md",
                &inputs(Some("sha256:one")),
                None,
            ),
            cache().plain_key("r", 1, scope, "b.md", &inputs(Some("sha256:one")), None),
            cache().plain_key("r", 1, scope, "a.md", &inputs(Some("sha256:two")), None),
            cache().plain_key(
                "r",
                1,
                scope,
                "a.md",
                &[Input::new("b.md", Some("sha256:one"))],
                None,
            ),
            cache().plain_key(
                "r",
                1,
                Scope::document(false, false, true, false),
                "a.md",
                &inputs(Some("sha256:one")),
                day("2026-08-12"),
            ),
            Cache::at(Path::new("/nonexistent"), "sha256:other").plain_key(
                "r",
                1,
                scope,
                "a.md",
                &inputs(Some("sha256:one")),
                None,
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

    /// The injected clock is a component of the key of a check that reads it.
    ///
    /// This is the hole spec 13 recorded. A windowed participation expectation
    /// compares a declared date against `ctx.now`, so two days are two verdicts,
    /// and a key that held one of them would serve yesterday's answer today.
    /// The `--no-cache` differential cannot catch that: it holds one value of
    /// the clock on both sides.
    #[test]
    fn two_days_are_two_keys_for_a_check_that_reads_the_clock() {
        let scope = Scope::document(false, false, true, false);
        let monday = cache()
            .plain_key(
                "r",
                1,
                scope,
                "a.md",
                &inputs(Some("sha256:one")),
                day("2026-08-12"),
            )
            .expect("a key");
        let tuesday = cache()
            .plain_key(
                "r",
                1,
                scope,
                "a.md",
                &inputs(Some("sha256:one")),
                day("2026-08-13"),
            )
            .expect("a key");
        assert_ne!(monday, tuesday, "the clock is not in the key");
    }

    /// And a check that does not read the clock keys the same on every day.
    ///
    /// The other half of the same property, and the reason the clock is keyed
    /// off the scope rather than added to every key: a rule that no calendar
    /// can move stays served from a cache when the date turns over.
    #[test]
    fn a_check_that_does_not_read_the_clock_keys_the_same_on_every_day() {
        let scope = Scope::document(false, false, false, false);
        let monday = cache().plain_key(
            "r",
            1,
            scope,
            "a.md",
            &inputs(Some("sha256:one")),
            day("2026-08-12"),
        );
        let tuesday = cache().plain_key(
            "r",
            1,
            scope,
            "a.md",
            &inputs(Some("sha256:one")),
            day("2026-08-13"),
        );
        assert_eq!(monday, tuesday);
        assert!(monday.is_some());
    }

    /// Two prior versions of one document are two keys.
    ///
    /// The sharpest form of the question this whole module answers: can two
    /// states that must differ produce one key? A transition check reads the
    /// version that stood before the change, so a run whose change carries a
    /// different prior version reached a different verdict about the same
    /// current bytes. Every input the read set names is equal on both sides
    /// here, so nothing else in the key could tell them apart.
    #[test]
    fn two_prior_versions_are_two_keys_for_a_check_that_reads_one() {
        let scope = Scope::document(false, false, false, true);
        let key = |prior: Prior<'_>| {
            cache().key(
                "warrant.promoted",
                1,
                scope,
                "a.md",
                &inputs(Some("sha256:one")),
                None,
                Some(prior),
                None,
            )
        };
        let facets = Mapping::default();
        let states = [
            key(Prior::Unchanged),
            key(Prior::Added),
            key(Prior::Committed {
                digest: "sha256:before",
                facets: &facets,
            }),
            key(Prior::Committed {
                digest: "sha256:another",
                facets: &facets,
            }),
        ];
        for (index, state) in states.iter().enumerate() {
            assert!(state.is_some(), "state {index} lost its key");
            for other in &states[index + 1..] {
                assert_ne!(state, other, "two states of the prior version share a key");
            }
        }
    }

    /// A scope that declares the prior version and was handed none is not
    /// keyed, and one that declares nothing keys as it did before the input
    /// existed.
    #[test]
    fn a_prior_reading_scope_with_no_prior_is_not_keyed() {
        assert_eq!(
            cache().key(
                "warrant.promoted",
                1,
                Scope::document(false, false, false, true),
                "a.md",
                &inputs(Some("sha256:one")),
                None,
                None,
                None,
            ),
            None
        );
        // And the declaration divides the key, so a rule that reads no prior
        // version cannot be served an entry that one wrote.
        assert_ne!(
            cache().plain_key(
                "r",
                1,
                Scope::document(false, false, false, true),
                "a.md",
                &inputs(Some("sha256:one")),
                None
            ),
            cache().plain_key(
                "r",
                1,
                Scope::document(false, false, false, false),
                "a.md",
                &inputs(Some("sha256:one")),
                None
            )
        );
    }

    /// A scope that declares the clock and was handed none is not keyed.
    ///
    /// The same answer as an input with no digest, for the same reason: the
    /// alternative is a key that omits an input the instance read.
    #[test]
    fn a_clock_reading_scope_with_no_clock_is_not_keyed() {
        assert_eq!(
            cache().plain_key(
                "r",
                1,
                Scope::document(false, false, true, false),
                "a.md",
                &inputs(Some("sha256:one")),
                None
            ),
            None
        );
    }

    /// An input with no content hash produces no key, so its instance is
    /// evaluated on every run. The alternative is a key over a hash that does
    /// not exist, which is the correctness bug spec 12 names.
    #[test]
    fn an_input_with_no_digest_is_not_keyed() {
        assert_eq!(
            cache().plain_key(
                "r",
                1,
                Scope::document(false, false, false, false),
                "a.md",
                &inputs(None),
                None
            ),
            None
        );
    }

    /// Two bindings of one anchor are two keys, and the instance keeps a key.
    ///
    /// This is the component [OBL-repo-0117] is about. The identity of the
    /// instance is unchanged across the change that falsifies the verdict — the
    /// target string below is the same on both sides, because a path anchor
    /// normalizes to the text an author wrote and an unbound target falls back
    /// to it. Only the resolution moves.
    ///
    /// The second assertion is the half that the byte-identity differential
    /// cannot see and that no gate reports. Refusing a key would also separate
    /// the two verdicts, and it would leave every anchor edge evaluated on
    /// every run forever. This component divides a key and never withholds one.
    ///
    /// [OBL-repo-0117]: ../../../../docs/obligations/0117-a-cached-verdict-about-an-anchor-survives-the-change-that-falsifies-it.md
    #[test]
    fn two_bindings_of_one_anchor_are_two_keys_and_neither_is_unkeyed() {
        let scope = Scope::edge(false);
        let target = "SPEC-HW-ai-integration\u{1f}governs\u{1f}.claude/hooks/lib.sh";
        let key = |resolution: Option<&str>| {
            cache().key(
                "relation.target.unresolved",
                1,
                scope,
                target,
                &inputs(Some("sha256:one")),
                None,
                None,
                resolution,
            )
        };

        let bound = key(Some("Anchor { normalized: \".claude/hooks/lib.sh\" }"));
        let gone = key(Some("Unbound(AnchorUnresolved { .. })"));
        assert_ne!(bound, gone, "the two states of one anchor share a key");
        assert!(bound.is_some(), "a resolved anchor lost its key");
        assert!(gone.is_some(), "an unresolved anchor lost its key");

        // And an instance that read no resolver keys as it did before this
        // component existed, which is what stops it from touching a scope that
        // reaches no anchor.
        assert_eq!(
            key(None),
            cache().plain_key(
                "relation.target.unresolved",
                1,
                scope,
                target,
                &inputs(Some("sha256:one")),
                None
            )
        );
    }

    /// A disabled cache computes no key at all, so `--no-cache` is a path that
    /// cannot read or write an entry rather than one that ignores what it read.
    #[test]
    fn a_disabled_cache_keys_nothing() {
        assert_eq!(
            Cache::disabled().plain_key(
                "r",
                1,
                Scope::document(false, false, false, false),
                "a.md",
                &inputs(Some("d")),
                None
            ),
            None
        );
    }
}
