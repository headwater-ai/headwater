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
//! # What this does not read
//!
//! It reads presence, and never freshness. A verification's `suspect` state
//! compares the commit a snapshot recorded against the commit that last
//! changed the criterion it proves, and that comparison needs a document's
//! history to walk. A control is a line in a taxonomy, not a document on a
//! shelf, and this module records the commit a snapshot names for one without
//! judging whether the taxonomy has moved since. That comparison is left
//! open, rather than silently skipped: see
//! [spec 13](../../../../docs/spec/13-open-obligations.md).

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
        Observations { entries }
    }

    /// The snapshot committed at `.headwater/observations.yml`.
    ///
    /// Absent, unreadable, or malformed all read as no observation, on
    /// [`crate::claim::Claims::at`]'s terms: a run over a corpus that has
    /// never written the file reports every external control unobserved
    /// rather than refusing to run.
    pub fn at(root: &Path) -> Self {
        let Ok(text) = std::fs::read_to_string(root.join(".headwater").join(FILE)) else {
            return Observations::empty();
        };
        let Ok(parsed) = headwater_yaml::load(&text) else {
            return Observations::empty();
        };
        let Value::Map(map) = &parsed.value else {
            return Observations::empty();
        };
        let mut entries = Vec::new();
        for entry in map {
            let Value::Map(fields) = &entry.value.value else {
                continue;
            };
            let Some(commit) = fields
                .get("commit")
                .and_then(|node| node.value.as_scalar())
                .map(|scalar| scalar.text.clone())
            else {
                continue;
            };
            entries.push(Observation {
                control: entry.key.value.clone(),
                commit,
            });
        }
        Observations { entries }
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
        let _ = std::fs::remove_dir_all(&dir);
    }
}
