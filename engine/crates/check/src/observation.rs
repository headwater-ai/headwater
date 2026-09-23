// SPDX-License-Identifier: Apache-2.0
//! The committed observation snapshot: which control naming a mechanism
//! outside this engine has actually been seen to run, and at what commit, and
//! which verification the corpus has seen a build settle.
//!
//! [Spec 4](../../../../docs/spec/04-assurance-model.md#every-obligation-has-exactly-one-disposition)
//! rules that such a control discharges nothing on its own: the register can
//! read that a taxonomy *declared* the binding, and it cannot read that the
//! pipeline *ran*. [`Observation::Control`] is that half, and #934 shipped it.
//! [`Observation::Verification`] is #937's extension of the same shape to a
//! verification's own observation, on the terms
//! [HW-DR-0073](../../../../docs/decisions/0073-a-verification-is-a-kind-and-its-identity-is-minted-rather-than-found-in-the-code-that-cites-it.md)
//! ruling 3 states: "the corpus records observation and freshness, and never
//! an outcome." One file, one reader, one read-set path: a control and a
//! verification are the same claim in different populations, and
//! [`Observations`] is where both are read so that a caller never has two
//! parsers to keep in step.
//!
//! # Where it lives, and why a run tolerates its absence
//!
//! `.headwater/observations.yml`, beside the claim store this mirrors
//! ([`crate::claim`]). A corpus that has wired no external mechanism to write
//! one has observed nothing, which is a true report, on
//! [`crate::claim::Claims::at`]'s own terms: the cost of reading an absent or
//! unreadable file as empty is a run that reports every external control
//! unobserved, and the alternative is a verb that refuses a corpus over a file
//! that names no obligation and no control.
//!
//! # What a control entry reads, and what spec 4 does not promise
//!
//! It reads whether a control is named, and nothing about the commit beside
//! it. [`Observations::observed`] compares the control key alone; the
//! `commit` field of a control entry is stored and never inspected, so a
//! snapshot naming a commit that never existed discharges exactly as well as
//! one naming the commit that ran. [HW-OBL-0199](../../../../docs/obligations/0199-an-observation-snapshot-records-a-commit-and-nothing-reads-it-back.md)
//! is the record of that gap for a control, including the empty-string case.
//! A verification entry is narrower than that, on purpose: a control is a
//! line in a taxonomy, and comparing its commit against anything would need a
//! history to walk that this crate never opens. A verification names a
//! document on a shelf, and the criterion it proves is a second document on a
//! shelf, so a verification entry can compare content instead of asking a
//! question only a version-control command could answer. See
//! [`Observation::Verification`] and [`plausible_commit`] for the shape that
//! comparison takes and the boundary it stays inside of.
//!
//! # Absent, unreadable and malformed are three different reports
//!
//! A corpus that has never written the file has observed nothing, which is a
//! true report, on [`crate::claim::Claims::at`]'s own terms: the cost of
//! reading an absent file as empty is a run that reports every external
//! control unobserved, and the alternative is a verb that refuses a corpus
//! over a file that names no obligation and no control. That argument does
//! not extend to a file that exists and fails to read or to parse: silence
//! there hides a real snapshot behind an unreadable one, and an obligation
//! that this snapshot verified yesterday reads unverified today with nothing
//! to say why. [`Observations::problems`] is what carries that difference
//! forward, so [`crate::register::Projection`] can turn it into a finding
//! instead of a quiet flip. [`Problem`] keeps the whole-file case apart from a
//! one-entry case, because the two need different remediation: a caller
//! cannot "remove the entry" of a file that never parsed into entries at all.
//!
//! A verification entry that fails one of its own two extra tests — no
//! `criterion_digest`, or a `commit` that does not read as a plausible commit
//! reference — is the same one-entry case: [`Problem::Entry`] names it rather
//! than letting it read as silently `declared`, which is what
//! [`Observations::verification`] would otherwise report and which is the
//! silent acceptance [HW-OBL-0199](../../../../docs/obligations/0199-an-observation-snapshot-records-a-commit-and-nothing-reads-it-back.md)
//! names as its own example for a control's empty-string commit. This engine
//! still cannot say whether a plausible-looking commit is an ancestor of the
//! tree in front of it — that fact needs a version-control command, and
//! [`plausible_commit`]'s own comment says why this module stays on its side
//! of that line.
//!
//! # What a gate needs, beside what a run needs
//!
//! [`PATH`] is this file named the way [`crate::claim::STORE`] names the
//! claim store: a constant a read set and a gate both use, so the two never
//! drift on the string. [`Observations::read_set_digest`] is the other half –
//! what to record for it. An absent file is not an input this run read, so it
//! stays out of the read set entirely, the same way an unresolved import is
//! not a document a corpus-scoped rule read. A present file always is,
//! whether or not it parsed: a `gate` re-hashing this path on a later tree has
//! to see that the bytes moved even when this run rejected them, or a snapshot
//! repaired between two runs would carry a stale verdict the same way a
//! snapshot deleted between them would.

use headwater_yaml::Value;
use std::path::Path;

/// One entry of a committed observation snapshot.
///
/// A control and a verification share a file and a reader, and never a shape:
/// the module comment says why the two ask different questions. `kind:
/// control` (or no `kind` at all, for every snapshot #934 already wrote) reads
/// as [`Observation::Control`]. `kind: verification` reads as
/// [`Observation::Verification`], and only there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Observation {
    /// #934's shape: an external control, and the commit it ran at. The
    /// commit is stored and never compared: see the module comment.
    Control { control: String, commit: String },
    /// #937's shape: a verification, the commit it was observed at, and the
    /// digest of the acceptance criterion it proves, taken at the moment the
    /// snapshot was written. [`Observations::verification`] compares that
    /// digest against the criterion's digest today, which is what
    /// [HW-DR-0073](../../../../docs/decisions/0073-a-verification-is-a-kind-and-its-identity-is-minted-rather-than-found-in-the-code-that-cites-it.md)
    /// ruling 3 calls DOORS rule P.6: a verification is suspect once the
    /// criterion it proves changed after the snapshot.
    Verification {
        verification: String,
        commit: String,
        criterion_digest: String,
    },
}

impl Observation {
    /// The identifier this entry names, whichever population it is drawn
    /// from. A lookup keyed on the string alone, the way
    /// [`Observations::observed`] and [`Observations::verification`] both need
    /// one.
    pub fn id(&self) -> &str {
        match self {
            Observation::Control { control, .. } => control,
            Observation::Verification { verification, .. } => verification,
        }
    }

    /// The commit either shape names. Provenance for a control (never
    /// compared, see the module comment) and provenance plus half of the
    /// freshness test for a verification.
    pub fn commit(&self) -> &str {
        match self {
            Observation::Control { commit, .. } => commit,
            Observation::Verification { commit, .. } => commit,
        }
    }

    /// A discriminator for grouping, so a duplicate check can tell a control
    /// entry and a verification entry apart even where the two id spaces ever
    /// collided on one string. Not exposed beyond this crate: a caller outside
    /// it should match on the enum instead of a label.
    fn population(&self) -> &'static str {
        match self {
            Observation::Control { .. } => "control",
            Observation::Verification { .. } => "verification",
        }
    }
}

/// What the snapshot records for one verification. See
/// [`Observations::recorded`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Recorded<'a> {
    /// The snapshot read, and no entry names the identifier.
    Absent,
    /// An entry names it, with the commit and the criterion digest it holds.
    Entry {
        commit: &'a str,
        criterion_digest: &'a str,
    },
    /// The file, or the entry for this identifier, did not read. The reason
    /// is the one [`Problem`] carries.
    Unread(&'a str),
}

/// What [`Observations::at`] could not use, kept apart by shape because the
/// two need different remediation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    /// The whole file: unreadable, unparsable, or not a mapping of an
    /// identifier to an entry. No entry in it reads, because there is no
    /// entry to read.
    File(String),
    /// One entry inside a file that otherwise read: a malformed shape, a
    /// missing field either kind requires, or a verification whose `commit`
    /// does not read as a plausible commit reference. The rest of the file's
    /// entries still read. A duplicate identifier is not raised here: the
    /// YAML loader already refuses a literal duplicate key before this reader
    /// sees one, and [`crate::register::Projection`] catches the shape that
    /// survives that — two [`Observation`]s of one [`Observations`] naming
    /// the same identifier in the same population — because that check has
    /// to hold for [`Observations::of`] too, which this module never
    /// validates.
    ///
    /// The field is `id` and not `control`, because the entry this names
    /// may be either kind: a verification entry that fails its own shape
    /// check (a missing `criterion_digest`, an implausible `commit`) is not
    /// a control, and a field that called it one would mislabel it in the
    /// one place a reader meets it, the rendered finding.
    Entry { id: String, reason: String },
}

/// The observation snapshot of one corpus, read once at the start of a run.
#[derive(Clone, Debug, Default)]
pub struct Observations {
    entries: Vec<Observation>,
    /// See [`Problem`]. Empty for [`Observations::empty`] and
    /// [`Observations::of`], and for a file that is absent, since an absent
    /// file is the documented empty case rather than a problem.
    problems: Vec<Problem>,
    /// See [`Observations::read_set_digest`].
    presence: Presence,
}

/// What [`Observations::at`] found at the path, on the terms a read set and a
/// gate need rather than the terms a disposition needs: those two ask "did
/// this run see bytes here", not "did this run get entries out of them".
#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum Presence {
    /// Nothing at the path. Not an input this run read, so a read set omits
    /// it: see the module comment.
    #[default]
    Absent,
    /// Something at the path, and this run could not get its bytes. Recorded
    /// with no digest, which is [`crate::readset`]'s own "no content hash"
    /// case, and a gate never trusts one of those to carry.
    Unreadable,
    /// The bytes this run read, hashed. Recorded whether or not they went on
    /// to parse: see the module comment for why a rejected file still needs a
    /// digest.
    Read(String),
}

/// The path a run reads this at, relative to the corpus root, on the same
/// terms [`crate::claim::STORE`] names the claim store: one constant a read
/// set and a gate both use, so the two cannot drift on the string.
pub const PATH: &str = ".headwater/observations.yml";

/// The file a run reads, beside the claim store and the taxonomy lock.
const FILE: &str = "observations.yml";

/// Whether `commit` reads as a plausible commit reference: non-empty, all
/// hexadecimal digits, and within the length a short or a full SHA takes.
///
/// This is a syntax check and never a version-control query. It catches the
/// empty string [HW-OBL-0199](../../../../docs/obligations/0199-an-observation-snapshot-records-a-commit-and-nothing-reads-it-back.md)
/// names as a control's own accepted gap, and a label that was never a commit
/// at all — `"nonexistent"`, `""`, a path, a sentence. It does not, and
/// cannot from here, tell a fabricated but hex-looking commit apart from one
/// that is a real ancestor of the tree in front of this run: that fact needs
/// `git merge-base --is-ancestor` or its equivalent, which is a command this
/// crate does not run ([spec 12](../../../../docs/spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version)'s
/// boundary, restated in [`crate::change`]'s own module comment). Answering
/// that question for real needs a fact carried in from outside this crate —
/// a manifest, the way a change's prior version already is — and that is a
/// widening of the boundary this module stays inside of today. This function
/// is the narrower, honest thing this module can check without it, and
/// [HW-OBL-0199](../../../../docs/obligations/0199-an-observation-snapshot-records-a-commit-and-nothing-reads-it-back.md)
/// says so rather than leaving the gap unnamed.
fn plausible_commit(commit: &str) -> bool {
    !commit.is_empty()
        && commit.len() <= 40
        && commit.chars().all(|c| c.is_ascii_hexdigit())
        && commit.len() >= 4
}

impl Observations {
    /// No snapshot. A corpus that has never written one reads this way, and so
    /// does a run whose caller did not offer one.
    pub fn empty() -> Self {
        Observations::default()
    }

    /// A snapshot built from entries a caller already has, for a test that
    /// wants one with no file on disk. The precedent is
    /// [`crate::claim::Claims::of`]. Reads as [`Presence::Absent`], because no
    /// path backs it: a read set has nothing to hash.
    pub fn of(entries: Vec<Observation>) -> Self {
        Observations {
            entries,
            problems: Vec::new(),
            presence: Presence::Absent,
        }
    }

    /// The snapshot committed at `.headwater/observations.yml`.
    ///
    /// Absent reads as no observation, on [`crate::claim::Claims::at`]'s
    /// terms: a run over a corpus that has never written the file reports
    /// every external control unobserved rather than refusing to run.
    /// Present but unreadable, unparsable, or holding a shape this reader
    /// cannot use is different: entries still read as absent for
    /// [`Observations::observed`] and [`Observations::verification`] (fail
    /// toward "not yet verified" rather than toward trusting a file this
    /// reader could not check), and [`Observations::problems`] carries one for
    /// each so a run says why rather than only showing a lower count.
    pub fn at(root: &Path) -> Self {
        let path = root.join(".headwater").join(FILE);
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Observations::empty();
            }
            Err(error) => {
                return Observations {
                    entries: Vec::new(),
                    problems: vec![Problem::File(format!(
                        "`.headwater/{FILE}` exists and could not be read: {error}. Every \
                         control or verification it would have named reads unobserved rather \
                         than verified."
                    ))],
                    presence: Presence::Unreadable,
                };
            }
        };
        // Hashed once, here, over exactly the bytes this run read — whether or
        // not they go on to parse. A read set records this either way: see
        // the module comment on why a rejected file still needs a digest.
        let presence = Presence::Read(headwater_hash::digest(text.as_bytes()));
        let parsed = match headwater_yaml::load(&text) {
            Ok(parsed) => parsed,
            Err(errors) => {
                let message = headwater_yaml::error::render(&errors);
                return Observations {
                    entries: Vec::new(),
                    problems: vec![Problem::File(format!(
                        "`.headwater/{FILE}` did not parse as YAML: {}. Every control or \
                         verification it would have named reads unobserved rather than \
                         verified.",
                        message.trim_end().replace('\n', "; ")
                    ))],
                    presence,
                };
            }
        };
        let Value::Map(map) = &parsed.value else {
            return Observations {
                entries: Vec::new(),
                problems: vec![Problem::File(format!(
                    "`.headwater/{FILE}`'s top level is not a mapping of an identifier to an \
                     entry. Every control or verification it would have named reads unobserved \
                     rather than verified."
                ))],
                presence,
            };
        };
        let mut entries: Vec<Observation> = Vec::new();
        let mut problems = Vec::new();
        for entry in map {
            let id = &entry.key.value;
            let Value::Map(fields) = &entry.value.value else {
                problems.push(Problem::Entry {
                    id: id.clone(),
                    reason: "it names no mapping under the identifier".to_string(),
                });
                continue;
            };
            let text_field = |name: &str| {
                fields
                    .get(name)
                    .and_then(|node| node.value.as_scalar())
                    .map(|scalar| scalar.text.clone())
            };
            let kind = text_field("kind").unwrap_or_else(|| "control".to_string());
            match kind.as_str() {
                "control" => {
                    let Some(commit) = text_field("commit") else {
                        problems.push(Problem::Entry {
                            id: id.clone(),
                            reason: "it names no `commit`".to_string(),
                        });
                        continue;
                    };
                    entries.push(Observation::Control {
                        control: id.clone(),
                        commit,
                    });
                }
                "verification" => {
                    let Some(commit) = text_field("commit") else {
                        problems.push(Problem::Entry {
                            id: id.clone(),
                            reason: "it names no `commit`".to_string(),
                        });
                        continue;
                    };
                    let Some(criterion_digest) = text_field("criterion_digest") else {
                        problems.push(Problem::Entry {
                            id: id.clone(),
                            reason: "it names no `criterion_digest`".to_string(),
                        });
                        continue;
                    };
                    // An empty digest names no bytes, so a comparison with
                    // it would report a change that did not happen.
                    if criterion_digest.trim().is_empty() {
                        problems.push(Problem::Entry {
                            id: id.clone(),
                            reason: "its `criterion_digest` is empty".to_string(),
                        });
                        continue;
                    }
                    if !plausible_commit(&commit) {
                        problems.push(Problem::Entry {
                            id: id.clone(),
                            reason: format!(
                                "its `commit` (`{commit}`) does not read as a plausible commit \
                                 reference, so it cannot be treated as observed"
                            ),
                        });
                        continue;
                    }
                    entries.push(Observation::Verification {
                        verification: id.clone(),
                        commit,
                        criterion_digest,
                    });
                }
                other => {
                    problems.push(Problem::Entry {
                        id: id.clone(),
                        reason: format!(
                            "it names `kind: {other}`, which is neither `control` nor \
                             `verification`"
                        ),
                    });
                }
            }
        }
        Observations {
            entries,
            problems,
            presence,
        }
    }

    /// Whether a committed snapshot names this control at all. It does not
    /// compare the commit the snapshot recorded against the tree in front of
    /// it: see the module comment for why that half is not here. Reads only
    /// [`Observation::Control`] entries: a verification sharing this string by
    /// coincidence is a different population, on [`Observations::verification`]'s
    /// own terms.
    pub fn observed(&self, control: &str) -> bool {
        self.entries
            .iter()
            .any(|entry| matches!(entry, Observation::Control { control: id, .. } if id == control))
    }

    /// What a committed snapshot says about one verification: the commit it
    /// was observed at, and the digest of the criterion it proved then, where
    /// a snapshot names it. `None` where no snapshot names it, which is
    /// [`Observations::of`]'s and [`Observations::at`]'s own `declared` case,
    /// on [HW-DR-0073](../../../../docs/decisions/0073-a-verification-is-a-kind-and-its-identity-is-minted-rather-than-found-in-the-code-that-cites-it.md)
    /// ruling 3's naming.
    pub fn verification(&self, id: &str) -> Option<(&str, &str)> {
        self.entries.iter().find_map(|entry| match entry {
            Observation::Verification {
                verification,
                commit,
                criterion_digest,
            } if verification == id => Some((commit.as_str(), criterion_digest.as_str())),
            _ => None,
        })
    }

    /// What the snapshot says about one verification, with the case
    /// [`Observations::verification`] folds into `None` kept apart: a
    /// snapshot that could not be read, or an entry for this identifier that
    /// did not read, is not the same fact as no entry at all. `declared`
    /// means that no entry names the verification, and a run that could not
    /// read the file does not know that.
    pub fn recorded(&self, id: &str) -> Recorded<'_> {
        if let Some(Problem::File(reason)) = self
            .problems
            .iter()
            .find(|problem| matches!(problem, Problem::File(_)))
        {
            return Recorded::Unread(reason);
        }
        if let Some((commit, criterion_digest)) = self.verification(id) {
            return Recorded::Entry {
                commit,
                criterion_digest,
            };
        }
        match self.problems.iter().find_map(|problem| match problem {
            Problem::Entry { id: named, reason } if named == id => Some(reason.as_str()),
            _ => None,
        }) {
            Some(reason) => Recorded::Unread(reason),
            None => Recorded::Absent,
        }
    }

    /// The whole-file problem, where the snapshot could not be read at all.
    pub fn unread(&self) -> Option<&str> {
        self.problems.iter().find_map(|problem| match problem {
            Problem::File(reason) => Some(reason.as_str()),
            Problem::Entry { .. } => None,
        })
    }

    pub fn entries(&self) -> &[Observation] {
        &self.entries
    }

    /// What [`Observations::at`] found unreadable, unparsable or malformed.
    pub fn problems(&self) -> &[Problem] {
        &self.problems
    }

    /// What a read set or a gate should record for this path.
    ///
    /// `None` when the file is genuinely absent: not an input this run read,
    /// so a caller adds nothing. `Some(None)` when it exists and this reader
    /// could not get its bytes: a caller adds [`PATH`] with no digest, which
    /// is [`crate::readset`]'s own "no content hash" case, and a gate that
    /// meets one never claims the verdict carries. `Some(Some(digest))` when
    /// this run read the bytes, whatever it made of them afterward: a caller
    /// adds [`PATH`] with that digest, so a `gate` on a later tree sees the
    /// file move even when this run rejected its content.
    pub fn read_set_digest(&self) -> Option<Option<&str>> {
        match &self.presence {
            Presence::Absent => None,
            Presence::Unreadable => Some(None),
            Presence::Read(digest) => Some(Some(digest.as_str())),
        }
    }

    /// One whole-file problem, with no entries, for a test in
    /// [`crate::register`] that wants the shape [`Observations::at`] returns
    /// for an unreadable file without writing one to disk.
    #[cfg(test)]
    pub fn malformed_for_test(problem: String) -> Self {
        Observations {
            entries: Vec::new(),
            problems: vec![Problem::File(problem)],
            presence: Presence::Unreadable,
        }
    }
}

/// A duplicate identifier within one population: two [`Observation`]s that
/// name the same id and the same [`Observation::population`]. Shared between
/// [`crate::register::Projection::of`] and this module's own tests so the two
/// read one rule rather than two.
pub(crate) fn duplicate_ids(entries: &[Observation]) -> Vec<String> {
    let mut duplicate = Vec::new();
    for (at, entry) in entries.iter().enumerate() {
        let first_at = entries
            .iter()
            .position(|earlier| {
                earlier.id() == entry.id() && earlier.population() == entry.population()
            })
            .expect("the entry itself is in its own list");
        if first_at != at && !duplicate.contains(&entry.id().to_string()) {
            duplicate.push(entry.id().to_string());
        }
    }
    duplicate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_control_the_snapshot_names_is_observed() {
        let observations = Observations::of(vec![Observation::Control {
            control: "CT-EXT-1".to_string(),
            commit: "788885a9".to_string(),
        }]);
        assert!(observations.observed("CT-EXT-1"));
        assert!(!observations.observed("CT-EXT-2"));
    }

    #[test]
    fn a_constructed_snapshot_reads_as_absent_for_the_read_set() {
        let observations = Observations::of(vec![]);
        assert_eq!(observations.read_set_digest(), None);
        assert_eq!(Observations::empty().read_set_digest(), None);
    }

    #[test]
    fn no_file_is_no_observation() {
        let dir =
            std::env::temp_dir().join(format!("hw-observation-test-{}-empty", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let observations = Observations::at(&dir);
        assert!(observations.entries().is_empty());
        // Absence is the documented empty case, not a problem: see the module
        // comment for why the two are not read the same way.
        assert!(observations.problems().is_empty());
        assert_eq!(observations.read_set_digest(), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_committed_file_reads_the_control_and_the_commit_it_ran_against() {
        let dir = std::env::temp_dir().join(format!(
            "hw-observation-test-{}-present",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(dir.join(".headwater"));
        std::fs::write(
            dir.join(".headwater").join(FILE),
            "CT-EXT-1:\n  commit: 788885a9\n",
        )
        .expect("the fixture writes");
        let observations = Observations::at(&dir);
        assert!(observations.observed("CT-EXT-1"));
        assert_eq!(observations.entries()[0].commit(), "788885a9");
        assert!(observations.problems().is_empty());
        assert!(
            matches!(observations.read_set_digest(), Some(Some(_))),
            "a present, parsable file carries a digest a read set can use: {:?}",
            observations.read_set_digest()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_file_that_does_not_parse_as_yaml_is_a_whole_file_problem_with_a_readable_message() {
        let dir = std::env::temp_dir().join(format!(
            "hw-observation-test-{}-badyaml",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(dir.join(".headwater"));
        std::fs::write(
            dir.join(".headwater").join(FILE),
            "CT-EXT-1: [unterminated\n",
        )
        .expect("the fixture writes");
        let observations = Observations::at(&dir);
        assert!(observations.entries().is_empty());
        assert_eq!(
            observations.problems().len(),
            1,
            "{:?}",
            observations.problems()
        );
        let Problem::File(message) = &observations.problems()[0] else {
            panic!(
                "a whole-file failure is `Problem::File`: {:?}",
                observations.problems()
            );
        };
        assert!(message.contains(FILE));
        // Readable by a person, not a `{:?}` dump of the loader's error type.
        assert!(
            !message.contains("LoadError") && !message.contains("Span {"),
            "{message}"
        );
        // A rejected file still carries a digest: a later repair has to be
        // seen by a gate even though this run never got entries out of it.
        assert!(
            matches!(observations.read_set_digest(), Some(Some(_))),
            "{:?}",
            observations.read_set_digest()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_unreadable_file_carries_no_digest_and_never_carries_under_a_gate() {
        let dir = std::env::temp_dir().join(format!(
            "hw-observation-test-{}-unreadable",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(dir.join(".headwater"));
        // A directory where a file is expected is the portable way to provoke
        // a read error that is not `NotFound`, without touching permission
        // bits a CI sandbox may not let this process change.
        std::fs::create_dir_all(dir.join(".headwater").join(FILE)).expect("the fixture writes");
        let observations = Observations::at(&dir);
        assert!(observations.entries().is_empty());
        assert_eq!(observations.read_set_digest(), Some(None));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_file_whose_top_level_is_not_a_mapping_is_a_whole_file_problem() {
        let dir =
            std::env::temp_dir().join(format!("hw-observation-test-{}-notmap", std::process::id()));
        let _ = std::fs::create_dir_all(dir.join(".headwater"));
        std::fs::write(
            dir.join(".headwater").join(FILE),
            "- CT-EXT-1\n- CT-EXT-2\n",
        )
        .expect("the fixture writes");
        let observations = Observations::at(&dir);
        assert!(observations.entries().is_empty());
        assert_eq!(
            observations.problems().len(),
            1,
            "{:?}",
            observations.problems()
        );
        assert!(matches!(observations.problems()[0], Problem::File(_)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_entry_with_no_commit_is_an_entry_problem_and_the_others_still_read() {
        let dir = std::env::temp_dir().join(format!(
            "hw-observation-test-{}-nocommit",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(dir.join(".headwater"));
        std::fs::write(
            dir.join(".headwater").join(FILE),
            "CT-EXT-1:\n  commit: 788885a9\nCT-EXT-2:\n  posture: advisory\n",
        )
        .expect("the fixture writes");
        let observations = Observations::at(&dir);
        assert!(observations.observed("CT-EXT-1"));
        assert!(!observations.observed("CT-EXT-2"));
        assert_eq!(
            observations.problems().len(),
            1,
            "{:?}",
            observations.problems()
        );
        assert_eq!(
            observations.problems()[0],
            Problem::Entry {
                id: "CT-EXT-2".to_string(),
                reason: "it names no `commit`".to_string(),
            }
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A literal duplicate key in the file is a whole-file `Problem::File`,
    /// via the YAML loader's own refusal
    /// (`headwater_yaml::loader::tests::duplicate_keys_are_an_error_and_name_the_first`
    /// pins that refusal on the loader's own terms). This module raises
    /// nothing extra for it: it is the same "did not parse as YAML" path as
    /// any other syntax error. What this module cannot see this way is two
    /// [`Observation`]s of one [`Observations`] sharing an identifier by
    /// construction rather than by file syntax, which is why
    /// `crate::register`'s duplicate check reads [`Observations::entries`]
    /// directly instead of trusting this reader to have ruled it out.
    #[test]
    fn a_literal_duplicate_key_is_a_whole_file_problem_via_the_loader() {
        let dir =
            std::env::temp_dir().join(format!("hw-observation-test-{}-dup", std::process::id()));
        let _ = std::fs::create_dir_all(dir.join(".headwater"));
        std::fs::write(
            dir.join(".headwater").join(FILE),
            "CT-EXT-1:\n  commit: aaa\nCT-EXT-1:\n  commit: bbb\n",
        )
        .expect("the fixture writes");
        let observations = Observations::at(&dir);
        assert!(observations.entries().is_empty());
        assert_eq!(
            observations.problems().len(),
            1,
            "{:?}",
            observations.problems()
        );
        assert!(matches!(observations.problems()[0], Problem::File(_)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The decisive fixture for #937: a verification observed at a commit,
    /// with the criterion's digest at snapshot time recorded beside it, reads
    /// `observed`. The same snapshot against a criterion digest that has since
    /// moved reads `suspect` on the same data, because
    /// [`Observations::verification`] only ever reports what the snapshot
    /// said — the comparison against "now" is the caller's, in
    /// `crate::verification`.
    #[test]
    fn a_verification_entry_reads_its_commit_and_criterion_digest_back() {
        let dir = std::env::temp_dir().join(format!(
            "hw-observation-test-{}-verification",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(dir.join(".headwater"));
        std::fs::write(
            dir.join(".headwater").join(FILE),
            "FIX-VER-0001:\n  kind: verification\n  commit: 788885a9\n  criterion_digest: \
             abc123\n",
        )
        .expect("the fixture writes");
        let observations = Observations::at(&dir);
        assert!(
            observations.problems().is_empty(),
            "{:?}",
            observations.problems()
        );
        assert_eq!(
            observations.verification("FIX-VER-0001"),
            Some(("788885a9", "abc123"))
        );
        assert_eq!(observations.verification("FIX-VER-9999"), None);
        // A control lookup does not cross into the verification population.
        assert!(!observations.observed("FIX-VER-0001"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_verification_entry_with_no_criterion_digest_is_an_entry_problem() {
        let dir = std::env::temp_dir().join(format!(
            "hw-observation-test-{}-verification-nodigest",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(dir.join(".headwater"));
        std::fs::write(
            dir.join(".headwater").join(FILE),
            "FIX-VER-0001:\n  kind: verification\n  commit: 788885a9\n",
        )
        .expect("the fixture writes");
        let observations = Observations::at(&dir);
        assert_eq!(observations.verification("FIX-VER-0001"), None);
        assert_eq!(
            observations.problems(),
            &[Problem::Entry {
                id: "FIX-VER-0001".to_string(),
                reason: "it names no `criterion_digest`".to_string(),
            }]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The second decisive fixture: a fabricated, non-hex commit on a
    /// verification entry is a problem, not a silent `declared`. This is the
    /// narrowing of [HW-OBL-0199](../../../../docs/obligations/0199-an-observation-snapshot-records-a-commit-and-nothing-reads-it-back.md)
    /// this change makes: the empty-string case that obligation names as a
    /// control's own accepted gap does not repeat for a verification.
    #[test]
    fn a_verification_entry_with_an_implausible_commit_is_an_entry_problem() {
        let dir = std::env::temp_dir().join(format!(
            "hw-observation-test-{}-verification-badcommit",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(dir.join(".headwater"));
        std::fs::write(
            dir.join(".headwater").join(FILE),
            "FIX-VER-0001:\n  kind: verification\n  commit: \"\"\n  criterion_digest: abc123\n",
        )
        .expect("the fixture writes");
        let observations = Observations::at(&dir);
        assert_eq!(observations.verification("FIX-VER-0001"), None);
        assert_eq!(
            observations.problems().len(),
            1,
            "{:?}",
            observations.problems()
        );
        let Problem::Entry { reason, .. } = &observations.problems()[0] else {
            panic!("{:?}", observations.problems());
        };
        assert!(reason.contains("plausible commit reference"), "{reason}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_unrecognized_kind_is_an_entry_problem() {
        let dir = std::env::temp_dir().join(format!(
            "hw-observation-test-{}-badkind",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(dir.join(".headwater"));
        std::fs::write(
            dir.join(".headwater").join(FILE),
            "FIX-VER-0001:\n  kind: unknown\n  commit: 788885a9\n",
        )
        .expect("the fixture writes");
        let observations = Observations::at(&dir);
        assert!(observations.entries().is_empty());
        assert_eq!(
            observations.problems().len(),
            1,
            "{:?}",
            observations.problems()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn plausible_commit_accepts_hex_and_refuses_everything_else() {
        assert!(plausible_commit("788885a9"));
        assert!(plausible_commit(
            "788885a9788885a9788885a9788885a9788885a9"[..40].as_ref()
        ));
        assert!(!plausible_commit(""));
        assert!(!plausible_commit("nonexistent-commit"));
        assert!(!plausible_commit("abc")); // shorter than a short SHA
        assert!(!plausible_commit(&"a".repeat(41))); // longer than a full SHA
    }

    #[test]
    fn duplicate_ids_reads_within_one_population_and_not_across_two() {
        let entries = vec![
            Observation::Control {
                control: "SAME-ID".to_string(),
                commit: "aaa1".to_string(),
            },
            Observation::Verification {
                verification: "SAME-ID".to_string(),
                commit: "bbb2".to_string(),
                criterion_digest: "digest".to_string(),
            },
            Observation::Control {
                control: "SAME-ID".to_string(),
                commit: "ccc3".to_string(),
            },
        ];
        // Two `Control` entries share `SAME-ID`, so it is a duplicate. The
        // `Verification` entry shares the string but not the population, and
        // does not itself introduce a second report of it.
        assert_eq!(duplicate_ids(&entries), vec!["SAME-ID".to_string()]);
    }
}
