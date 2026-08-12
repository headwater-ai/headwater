// SPDX-License-Identifier: Apache-2.0
//! The walk over a corpus root.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots) puts
//! this on the correctness-root list and states the failure precisely: "every
//! coverage guarantee (OB-COV-1..3) assumes that the walk enumerates the corpus
//! root correctly. A glob or symlink bug quietly shrinks the denominator, which
//! is the exact failure that the census exists to prevent."
//!
//! A shrunk denominator produces a *green* run. Nothing downstream can detect
//! it, because a file that the walk never yielded has no row, no finding, and no
//! absence to notice. So the rules below are stated rather than implied, and the
//! fixture tree under `fixtures/walk/` holds one case for each.
//!
//! # The rules
//!
//! - **Every file under the root yields exactly one entry.** A directory yields
//!   none, because a directory is not a file; the walk descends into it.
//! - **A symlink is never followed, and it yields an entry of its own.** To
//!   follow one either duplicates a file that is already in the denominator, or
//!   leaves the root entirely, or loops. All three corrupt the count, and the
//!   third does not terminate. The entry records where the link pointed, so the
//!   census reports the link rather than hiding it.
//! - **A declared exclusion does not stop the walk.** An excluded file still
//!   yields an entry, carrying the rule that excluded it.
//!   [Spec 7](../../../../docs/spec/07-distribution-and-federation.md) states
//!   the reason under a different mechanism: "no threshold ever converts an
//!   accounting into a silence". A subtree that vanishes from the walk is that
//!   conversion, whatever the reason for it was.
//! - **Order is the path, in bytes.** Two runs over one tree report identically,
//!   which is what makes a recorded census a fixture rather than a diff.
//! - **A directory the walk cannot read yields an entry for the directory.** The
//!   files under it are lost, and an entry is the only place that can say so.

use crate::pattern::Pattern;
use std::path::{Path, PathBuf};

/// The corpus root, and what it declares outside itself.
#[derive(Clone, Debug)]
pub struct Corpus {
    /// The repository root on disk. Every path a census reports is relative to
    /// it, because a shelf pattern is written that way and a finding is read
    /// that way.
    pub base: PathBuf,
    /// The corpus root, relative to `base`, with `/` separators.
    pub root: String,
    pub exclusions: Vec<Exclusion>,
}

/// A path that the corpus declares is not corpus content, and why.
///
/// The reason is required and it is prose. An exclusion with no reason is a
/// silent pass with a configuration file in front of it.
#[derive(Clone, Debug)]
pub struct Exclusion {
    pub pattern: Pattern,
    pub reason: String,
}

impl Exclusion {
    pub fn new(pattern: &str, reason: &str) -> Self {
        Self {
            pattern: Pattern::new(pattern),
            reason: reason.to_string(),
        }
    }
}

impl Corpus {
    pub fn new(base: impl Into<PathBuf>, root: &str) -> Self {
        Self {
            base: base.into(),
            root: root.trim_end_matches('/').to_string(),
            exclusions: Vec::new(),
        }
    }

    pub fn excluding(mut self, exclusions: Vec<Exclusion>) -> Self {
        self.exclusions = exclusions;
        self
    }

    /// The corpus a consumer declaration describes: a root, and each exclusion
    /// with the reason it states.
    ///
    /// The pairs come from [`headwater_resolve::Consumer`], and they arrive as
    /// pairs rather than as a type so that the walker keeps no dependency on
    /// the resolver. What the census needs from a consumer declaration is two
    /// strings per exclusion, and the reason is not optional: an exclusion with
    /// none is a silent pass with a configuration file in front of it.
    pub fn declared(base: impl Into<PathBuf>, root: &str, exclusions: &[(String, String)]) -> Self {
        Self::new(base, root).excluding(
            exclusions
                .iter()
                .map(|(path, reason)| Exclusion::new(path, reason))
                .collect(),
        )
    }

    fn exclusion_for(&self, path: &str) -> Option<&Exclusion> {
        self.exclusions
            .iter()
            .find(|exclusion| exclusion.pattern.matches(path))
    }
}

/// One entry of the walk: a path, and what the filesystem said it was.
#[derive(Clone, Debug)]
pub struct Entry {
    /// Relative to the repository root, with `/` separators.
    pub path: String,
    pub on_disk: PathBuf,
    pub kind: EntryKind,
    /// The exclusion that claims this path, if one does.
    pub excluded_by: Option<Exclusion>,
}

/// What the walk found, as a closed set.
#[derive(Clone, Debug)]
pub enum EntryKind {
    File,
    /// A symlink, and where it pointed, as written.
    Symlink {
        target: String,
    },
    /// A directory the walk could not read, so the files under it are missing
    /// from the count and this entry is the only record of that.
    UnreadableDirectory {
        error: String,
    },
    /// A name that is not UTF-8. It cannot be matched against a pattern that is
    /// UTF-8, so it can be neither shelved nor excluded, and the path below is
    /// the lossy form for a person to read.
    UnreadableName,
}

/// Walk the corpus root, in path order.
///
/// Missing root: one entry, so that a misconfigured root is a visible row rather
/// than an empty census that reads as a clean corpus.
pub fn walk(corpus: &Corpus) -> Vec<Entry> {
    let mut found = Vec::new();
    let root = corpus.base.join(&corpus.root);
    if !root.is_dir() {
        found.push(Entry {
            path: corpus.root.clone(),
            on_disk: root,
            kind: EntryKind::UnreadableDirectory {
                error: "the corpus root is not a directory".to_string(),
            },
            excluded_by: None,
        });
        return found;
    }
    descend(corpus, &root, &corpus.root, &mut found);
    found.sort_by(|a, b| a.path.cmp(&b.path));
    found
}

fn descend(corpus: &Corpus, directory: &Path, prefix: &str, found: &mut Vec<Entry>) {
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => {
            found.push(Entry {
                path: prefix.to_string(),
                on_disk: directory.to_path_buf(),
                kind: EntryKind::UnreadableDirectory {
                    error: error.to_string(),
                },
                excluded_by: None,
            });
            return;
        }
    };

    for entry in entries {
        let Ok(entry) = entry else { continue };
        let on_disk = entry.path();
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            found.push(Entry {
                path: format!(
                    "{prefix}/{}",
                    on_disk.file_name().unwrap_or_default().to_string_lossy()
                ),
                on_disk,
                kind: EntryKind::UnreadableName,
                excluded_by: None,
            });
            continue;
        };
        let path = format!("{prefix}/{name}");
        let excluded_by = corpus.exclusion_for(&path).cloned();

        // `symlink_metadata` rather than `metadata`, and that single call is the
        // whole symlink rule: `metadata` follows the link, so a link to a
        // directory would read as a directory and the walk would descend
        // through it.
        let metadata = match std::fs::symlink_metadata(&on_disk) {
            Ok(metadata) => metadata,
            Err(error) => {
                found.push(Entry {
                    path,
                    on_disk,
                    kind: EntryKind::UnreadableDirectory {
                        error: error.to_string(),
                    },
                    excluded_by,
                });
                continue;
            }
        };

        if metadata.is_symlink() {
            let target = std::fs::read_link(&on_disk)
                .map(|target| target.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|error| error.to_string());
            found.push(Entry {
                path,
                on_disk,
                kind: EntryKind::Symlink { target },
                excluded_by,
            });
        } else if metadata.is_dir() {
            descend(corpus, &on_disk, &path, found);
        } else {
            found.push(Entry {
                path,
                on_disk,
                kind: EntryKind::File,
                excluded_by,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_corpus() -> Corpus {
        Corpus::new(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures"),
            "walk",
        )
    }

    #[test]
    fn the_walk_is_ordered_by_path_so_two_runs_report_alike() {
        let entries = walk(&fixture_corpus());
        let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
        let mut sorted = paths.clone();
        sorted.sort();
        assert_eq!(paths, sorted);
        assert!(
            entries.len() > 10,
            "the fixture tree is thin: {}",
            entries.len()
        );
    }

    #[test]
    fn a_symlink_is_an_entry_and_is_never_followed() {
        let entries = walk(&fixture_corpus());
        let links: Vec<&Entry> = entries
            .iter()
            .filter(|e| matches!(e.kind, EntryKind::Symlink { .. }))
            .collect();
        assert!(!links.is_empty(), "the fixture tree has no symlink in it");
        // The proof that nothing followed the link to a directory: no entry
        // sits under it. Following it would have duplicated the whole subtree
        // into the denominator.
        assert!(
            !entries
                .iter()
                .any(|e| e.path.contains("link-to-directory/")),
            "the walk descended through a symlink"
        );
    }

    #[test]
    fn a_missing_root_is_a_row_rather_than_an_empty_census() {
        let entries = walk(&Corpus::new(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures"),
            "no-such-directory",
        ));
        assert_eq!(entries.len(), 1);
        assert!(matches!(
            entries[0].kind,
            EntryKind::UnreadableDirectory { .. }
        ));
    }

    #[test]
    fn an_excluded_file_is_still_walked_and_carries_the_rule_that_excluded_it() {
        let corpus = fixture_corpus().excluding(vec![Exclusion::new(
            "walk/excluded/**",
            "a declared exclusion, so that a test has one",
        )]);
        let entries = walk(&corpus);
        let excluded: Vec<&Entry> = entries.iter().filter(|e| e.excluded_by.is_some()).collect();
        assert!(!excluded.is_empty(), "nothing under walk/excluded/");
        // The rule of spec 7 applied to the walk: the exclusion changes the
        // outcome of a row and never the presence of one.
        assert!(entries.iter().any(|e| e.path.starts_with("walk/excluded/")));
    }
}
