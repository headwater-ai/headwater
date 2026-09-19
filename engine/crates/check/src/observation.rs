// SPDX-License-Identifier: Apache-2.0
//! The committed observation snapshot: which control naming a mechanism
//! outside this engine has actually been seen to run, and at what commit.
//!
//! [Spec 4](../../../../docs/spec/04-assurance-model.md#every-obligation-has-exactly-one-disposition)
//! rules that such a control discharges nothing on its own: the register can
//! read that a taxonomy *declared* the binding, and it cannot read that the
//! pipeline *ran*. This file is the second half. [HW-DR-0073](../../../../docs/decisions/0073-a-verification-is-a-kind-and-its-identity-is-minted-rather-than-found-in-the-code-that-cites-it.md)
//! ruling 3 is the design this module follows: a snapshot names an identifier
//! and the commit it ran against, with no outcome column, and the engine
//! reads it offline against a pin. #937 extends the same shape to carry a
//! verification's observation; this module carries a control's, which is the
//! half #934 owns.
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
//! # What this reads, and what spec 4 does not promise
//!
//! It reads whether a control is named, and nothing about the commit beside
//! it. `Observations::observed` compares the control key alone; the `commit`
//! field is stored and never inspected, so a snapshot naming a commit that
//! never existed discharges exactly as well as one naming the commit that
//! ran. [HW-OBL-0199](../../../../docs/obligations/0199-an-observation-snapshot-records-a-commit-and-nothing-reads-it-back.md)
//! is the record of that gap, including the empty-string case. This is
//! narrower than a verification's `suspect` state, which compares the commit
//! a snapshot recorded against the commit that last changed the criterion it
//! proves: that comparison needs a document's history to walk, and a control
//! is a line in a taxonomy, not a document on a shelf. Spec 4's own sentence
//! says only that a snapshot "names the control and the commit it ran
//! against" — a claim about what the file holds, not a claim that the engine
//! verifies either half of it.
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

/// One control's recorded observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub control: String,
    pub commit: String,
}

/// What [`Observations::at`] could not use, kept apart by shape because the
/// two need different remediation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    /// The whole file: unreadable, unparsable, or not a mapping of control id
    /// to entry. No entry in it reads, because there is no entry to read.
    File(String),
    /// One entry inside a file that otherwise read: a malformed shape, or no
    /// `commit`. The rest of the file's entries still read. A duplicate
    /// control is not raised here: the YAML loader already refuses a literal
    /// duplicate key before this reader sees one, and
    /// [`crate::register::Projection`] catches the shape that survives that —
    /// two [`Observation`]s of one [`Observations`] naming the same
    /// control — because that check has to hold for [`Observations::of`] too,
    /// which this module never validates.
    Entry { control: String, reason: String },
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
    /// [`Observations::observed`] (fail toward "not yet verified" rather than
    /// toward trusting a file this reader could not check), and
    /// [`Observations::problems`] carries one for each so a run says why
    /// rather than only showing a lower count.
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
                         control it would have named reads unobserved rather than verified."
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
                        "`.headwater/{FILE}` did not parse as YAML: {}. Every control it would \
                         have named reads unobserved rather than verified.",
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
                    "`.headwater/{FILE}`'s top level is not a mapping of control id to entry. \
                     Every control it would have named reads unobserved rather than verified."
                ))],
                presence,
            };
        };
        let mut entries: Vec<Observation> = Vec::new();
        let mut problems = Vec::new();
        for entry in map {
            let control = &entry.key.value;
            let Value::Map(fields) = &entry.value.value else {
                problems.push(Problem::Entry {
                    control: control.clone(),
                    reason: "it names no mapping under the control id".to_string(),
                });
                continue;
            };
            let Some(commit) = fields
                .get("commit")
                .and_then(|node| node.value.as_scalar())
                .map(|scalar| scalar.text.clone())
            else {
                problems.push(Problem::Entry {
                    control: control.clone(),
                    reason: "it names no `commit`".to_string(),
                });
                continue;
            };
            entries.push(Observation {
                control: control.clone(),
                commit,
            });
        }
        Observations {
            entries,
            problems,
            presence,
        }
    }

    /// Whether a committed snapshot names this control at all. It does not
    /// compare the commit the snapshot recorded against the tree in front of
    /// it: see the module comment for why that half is not here.
    pub fn observed(&self, control: &str) -> bool {
        self.entries.iter().any(|entry| entry.control == control)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_control_the_snapshot_names_is_observed() {
        let observations = Observations::of(vec![Observation {
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
        assert_eq!(observations.entries()[0].commit, "788885a9");
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
                control: "CT-EXT-2".to_string(),
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
    /// [`Observation`]s of one [`Observations`] sharing a control by
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
}
