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
//! # What this does not read, and what spec 4 does not promise
//!
//! It reads presence, and never freshness. A verification's `suspect` state
//! compares the commit a snapshot recorded against the commit that last
//! changed the criterion it proves, and that comparison needs a document's
//! history to walk. A control is a line in a taxonomy, not a document on a
//! shelf, and this module records the commit a snapshot names for one without
//! judging whether the taxonomy has moved since, or whether the string is a
//! commit this repository ever held at all. That comparison is left open,
//! rather than silently skipped:
//! [HW-OBL-0199](../../../../docs/obligations/0199-an-observation-snapshot-records-a-commit-and-nothing-reads-it-back.md)
//! is the record of the gap, and spec 4's own sentence says only that a
//! snapshot "names the control and the commit it ran against" — a claim about
//! what the file holds, not a claim that the engine verifies either half of
//! it beyond the control's own identifier matching.
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
//! instead of a quiet flip.

use headwater_yaml::Value;
use std::path::Path;

/// One control's recorded observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub control: String,
    pub commit: String,
}

/// The observation snapshot of one corpus, read once at the start of a run.
#[derive(Clone, Debug, Default)]
pub struct Observations {
    entries: Vec<Observation>,
    /// What [`Observations::at`] could not read as an entry, each as a
    /// sentence naming the file and the reason. Empty for [`Observations::empty`]
    /// and [`Observations::of`], and for a file that is absent, since an
    /// absent file is the documented empty case rather than a problem.
    problems: Vec<String>,
}

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
    /// [`crate::claim::Claims::of`].
    pub fn of(entries: Vec<Observation>) -> Self {
        Observations {
            entries,
            problems: Vec::new(),
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
    /// [`Observations::problems`] carries a sentence for each so a run says
    /// why rather than only showing a lower count.
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
                    problems: vec![format!(
                        "`.headwater/{FILE}` could not be read ({error}), so every control it \
                         would have named reads unobserved rather than verified"
                    )],
                };
            }
        };
        let parsed = match headwater_yaml::load(&text) {
            Ok(parsed) => parsed,
            Err(errors) => {
                return Observations {
                    entries: Vec::new(),
                    problems: vec![format!(
                        "`.headwater/{FILE}` did not parse as YAML ({errors:?}), so every \
                         control it would have named reads unobserved rather than verified"
                    )],
                };
            }
        };
        let Value::Map(map) = &parsed.value else {
            return Observations {
                entries: Vec::new(),
                problems: vec![format!(
                    "`.headwater/{FILE}`'s top level is not a mapping of control id to entry, \
                     so every control it would have named reads unobserved rather than verified"
                )],
            };
        };
        let mut entries = Vec::new();
        let mut problems = Vec::new();
        for entry in map {
            let control = &entry.key.value;
            let Value::Map(fields) = &entry.value.value else {
                problems.push(format!(
                    "`.headwater/{FILE}` names `{control}` with no mapping under it, so it \
                     reads unobserved rather than verified"
                ));
                continue;
            };
            let Some(commit) = fields
                .get("commit")
                .and_then(|node| node.value.as_scalar())
                .map(|scalar| scalar.text.clone())
            else {
                problems.push(format!(
                    "`.headwater/{FILE}` names `{control}` with no `commit`, so it reads \
                     unobserved rather than verified"
                ));
                continue;
            };
            entries.push(Observation {
                control: control.clone(),
                commit,
            });
        }
        Observations { entries, problems }
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

    /// What [`Observations::at`] found unreadable, unparsable or malformed,
    /// each already the whole sentence a finding's message can carry.
    pub fn problems(&self) -> &[String] {
        &self.problems
    }

    /// One problem, with no entries, for a test in [`crate::register`] that
    /// wants the shape [`Observations::at`] returns for an unreadable file
    /// without writing one to disk.
    #[cfg(test)]
    pub fn malformed_for_test(problem: String) -> Self {
        Observations {
            entries: Vec::new(),
            problems: vec![problem],
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
    fn no_file_is_no_observation() {
        let dir =
            std::env::temp_dir().join(format!("hw-observation-test-{}-empty", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let observations = Observations::at(&dir);
        assert!(observations.entries().is_empty());
        // Absence is the documented empty case, not a problem: see the module
        // comment for why the two are not read the same way.
        assert!(observations.problems().is_empty());
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
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_file_that_does_not_parse_as_yaml_is_a_problem_and_not_a_silent_empty_read() {
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
        assert!(observations.problems()[0].contains(FILE));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_file_whose_top_level_is_not_a_mapping_is_a_problem() {
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
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_entry_with_no_commit_is_a_problem_and_the_others_still_read() {
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
        assert!(observations.problems()[0].contains("CT-EXT-2"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
