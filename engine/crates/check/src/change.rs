// SPDX-License-Identifier: Apache-2.0
//! The change: the documents one change carries, and the version of each that
//! stood before it.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version)
//! rules that "a change reaches this engine as a named set of inputs, and never
//! as a second tree. The prior version above is the one exception, and the
//! change that supplies it also bounds its scope." This module is that named
//! set. It walks no history, it runs no version control tool, and it opens no
//! file that the caller did not name.
//!
//! # What the caller supplies, and why the bytes rather than a verdict
//!
//! A manifest names each document the change carries. A document the change
//! adds is named as added. A document the change modifies is named with a
//! second path, holding the bytes that stood before the change. The engine
//! reads those bytes, parses them, and hashes them, and the hash joins the
//! cache key.
//!
//! The alternative was a manifest that states the prior *value* of a facet,
//! which is smaller and wrong: the caller would then compute the subject of the
//! check, and this engine would report a transition that it never read. What a
//! caller may state is which files moved. What they became is read here.
//!
//! The caller is a hook or a CI job, which is where a repository's version
//! control lives. `git show HEAD:<path>` and `git merge-base` are commands that
//! belong to the caller, and spec 12 states what the caller must anchor "prior"
//! to: the version on the branch where the change lands.
//!
//! # Three states, and the fourth that never reaches a check
//!
//! A document the manifest does not name did not change, and its prior version
//! is the version in front of the run. A document named as added had none. A
//! document named with a prior version has the one the caller supplied. Those
//! three are [`Prior`], and a check receives one of them.
//!
//! The fourth is a prior version this engine could not read, and it is not a
//! fourth arm of that type. A check that could reach one would have to decide
//! what to do about a version it cannot see, and every wrong answer to that
//! question is green. So the runner skips the instance with the reason, which
//! is the visible skip
//! [spec 4](../../../../docs/spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)
//! asks for, and the check is never handed one.
//!
//! # A manifest this reader cannot parse is refused whole
//!
//! [`crate::cache`] fails toward re-running, because a doubtful cache entry
//! costs one evaluation. This reader fails toward refusing, because the cost of
//! a dropped line is the opposite: a document the manifest named and this reader
//! skipped reads as a document that did not change, which is a promotion that
//! nothing counts. A caller that cannot be parsed gets an error and no run.

use headwater_doc::Document;
use headwater_yaml::Mapping;
use std::path::Path;

/// The first line of a manifest. A file that does not open with it is a file
/// this engine did not ask for, and it is refused rather than guessed at.
pub const FORMAT: &str = "headwater change 1";

/// What one document of a change carried before it.
///
/// Held rather than borrowed, because the bytes come from outside the corpus
/// walk and no other phase of a run holds them.
#[derive(Clone, Debug)]
enum Held {
    /// The change adds this document. Nothing stood at this path before it.
    Added,
    /// The version that stood before the change, as the caller named it.
    Committed { digest: String, document: Document },
    /// The caller named a prior version that this engine could not read. See
    /// the module comment: it never reaches a check.
    Unreadable { why: String },
}

/// The version of one document that stood before the change, as a check reads
/// it.
///
/// Three arms, and each one is a different fact. A check that folded two of them
/// together would answer a question nobody asked: a document that is new in this
/// change never moved, and a document the change does not carry never moved
/// either, and neither of those is a document whose warrant stayed where it was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prior<'a> {
    /// The change does not carry this document, so the version in front of the
    /// run is the version that stood before it.
    Unchanged,
    /// The change adds this document, and no version stood before it.
    Added,
    /// The version that stood before the change.
    ///
    /// The front matter is the same document at an earlier state, so this
    /// widens no scope: a document-scoped check still reads one document and
    /// still cannot reach a sibling.
    Committed {
        digest: &'a str,
        facets: &'a Mapping,
    },
}

impl Prior<'_> {
    /// The component this input writes into a cache key.
    ///
    /// Every state that changes a verdict writes different characters. The two
    /// words are spelled out rather than left as an empty line, because an
    /// absent line would make an added document and an unchanged one one key,
    /// and those are the two states whose verdicts differ most.
    pub(crate) fn key(&self) -> String {
        match self {
            Prior::Unchanged => "unchanged".to_string(),
            Prior::Added => "added".to_string(),
            Prior::Committed { digest, .. } => format!("committed {digest}"),
        }
    }
}

/// The documents one change carries.
#[derive(Clone, Debug, Default)]
pub struct Change {
    /// In path order, so a lookup is a binary search and a report is a function
    /// of the change rather than of the order a caller wrote it in.
    entries: Vec<(String, Held)>,
}

/// What a run injected, for the report that states its own inputs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Named {
    /// Documents the manifest named, whatever became of each one.
    pub documents: usize,
    /// Of those, the ones the change adds.
    pub added: usize,
    /// Of those, the ones whose prior version this run read.
    pub carried: usize,
    /// Of those, the ones whose prior version this run could not read. Each one
    /// is a skipped instance with a reason rather than a pass.
    pub unreadable: usize,
}

impl Change {
    /// Read a manifest, and the prior version each line names.
    ///
    /// `open` is how a named file becomes bytes, so a test supplies a tree that
    /// no filesystem holds and the CLI supplies the filesystem.
    pub fn read(
        manifest: &str,
        open: impl Fn(&Path) -> std::io::Result<Vec<u8>>,
    ) -> Result<Change, String> {
        let mut lines = manifest.lines();
        match lines.next() {
            Some(FORMAT) => {}
            Some(other) => {
                return Err(format!(
                    "a change manifest opens with `{FORMAT}`, and this one opens with `{other}`"
                ))
            }
            None => {
                return Err(format!(
                    "a change manifest opens with `{FORMAT}`, and this one is empty"
                ))
            }
        }

        let mut entries: Vec<(String, Held)> = Vec::new();
        for (number, line) in lines.enumerate() {
            let number = number + 2;
            if line.trim().is_empty() {
                continue;
            }
            let mut fields = line.split('\t');
            let (path, held) = match (fields.next(), fields.next(), fields.next(), fields.next()) {
                (Some("added"), Some(path), None, _) => (path, Held::Added),
                (Some("prior"), Some(path), Some(source), None) => {
                    (path, read_prior(source, &open))
                }
                _ => {
                    return Err(format!(
                        "line {number} of the change manifest is neither `added\\t<path>` nor \
                         `prior\\t<path>\\t<file>`: `{line}`"
                    ))
                }
            };
            if entries.iter().any(|(known, _)| known == path) {
                return Err(format!(
                    "the change manifest names `{path}` twice, and a document has one prior version"
                ));
            }
            entries.push((path.to_string(), held));
        }

        entries.sort_by(|(a, _), (b, _)| a.cmp(b));
        Ok(Change { entries })
    }

    /// Read a manifest from a file, which is what `headwater check --change`
    /// does. A path inside the manifest is resolved as written, because the
    /// caller wrote it and the prior versions sit outside the corpus.
    pub fn at(manifest: &Path) -> Result<Change, String> {
        let text = std::fs::read_to_string(manifest)
            .map_err(|error| format!("{}: {error}", manifest.display()))?;
        Change::read(&text, |path| std::fs::read(path))
    }

    /// What this change named, for the report that states its own inputs.
    pub fn named(&self) -> Named {
        let mut named = Named {
            documents: self.entries.len(),
            ..Named::default()
        };
        for (_, held) in &self.entries {
            match held {
                Held::Added => named.added += 1,
                Held::Committed { .. } => named.carried += 1,
                Held::Unreadable { .. } => named.unreadable += 1,
            }
        }
        named
    }

    /// The prior version of one document, or the reason no check may be handed
    /// one.
    ///
    /// A path this change does not name is [`Prior::Unchanged`], which is the
    /// whole of what "the change carries these documents and no others" means.
    pub(crate) fn prior_of(&self, path: &str) -> Result<Prior<'_>, &str> {
        let Ok(index) = self
            .entries
            .binary_search_by(|(known, _)| known.as_str().cmp(path))
        else {
            return Ok(Prior::Unchanged);
        };
        match &self.entries[index].1 {
            Held::Added => Ok(Prior::Added),
            Held::Committed { digest, document } => Ok(Prior::Committed {
                digest,
                facets: &document.facets,
            }),
            Held::Unreadable { why } => Err(why),
        }
    }
}

/// One prior version, read and parsed, or the reason it was neither.
fn read_prior(source: &str, open: impl Fn(&Path) -> std::io::Result<Vec<u8>>) -> Held {
    let bytes = match open(Path::new(source)) {
        Ok(bytes) => bytes,
        Err(error) => {
            return Held::Unreadable {
                why: format!("the prior version at `{source}` did not open: {error}"),
            }
        }
    };
    // The digest is over the bytes, which is what the census hashes a document
    // with. So the prior version and the current one are hashed by one function
    // and their digests are comparable.
    let digest = headwater_hash::digest(&bytes);
    let Ok(text) = String::from_utf8(bytes) else {
        return Held::Unreadable {
            why: format!("the prior version at `{source}` is not UTF-8"),
        };
    };
    match headwater_doc::parse(&text) {
        Ok(document) => Held::Committed { digest, document },
        Err(errors) => Held::Unreadable {
            why: format!(
                "the prior version at `{source}` did not parse: {} error(s)",
                errors.len()
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree<'a>(
        files: &'a [(&'a str, &'a str)],
    ) -> impl Fn(&Path) -> std::io::Result<Vec<u8>> + 'a {
        move |path| {
            files
                .iter()
                .find(|(name, _)| Path::new(name) == path)
                .map(|(_, text)| text.as_bytes().to_vec())
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no such file"))
        }
    }

    const ASSERTED: &str = "---\nid: A\nprovenance:\n  warrant: asserted\n---\n\n# A\n";

    fn manifest() -> String {
        format!("{FORMAT}\nadded\tdocs/b.md\nprior\tdocs/a.md\tprior/a.md\n")
    }

    /// The three states a document of a change can be in, and the fourth that a
    /// check never reaches.
    #[test]
    fn a_change_holds_three_states_and_refuses_to_hand_over_the_fourth() {
        let change =
            Change::read(&manifest(), tree(&[("prior/a.md", ASSERTED)])).expect("a change");
        assert!(matches!(
            change.prior_of("docs/a.md"),
            Ok(Prior::Committed { .. })
        ));
        assert_eq!(change.prior_of("docs/b.md"), Ok(Prior::Added));
        // Named by nothing, so it stands as it stood.
        assert_eq!(change.prior_of("docs/c.md"), Ok(Prior::Unchanged));

        let missing = Change::read(&manifest(), tree(&[])).expect("a change");
        let why = missing.prior_of("docs/a.md").expect_err("a reason");
        assert!(why.contains("prior/a.md"), "{why}");
    }

    /// Every state writes different characters into the key.
    ///
    /// The two that matter most are the two with no digest: an added document
    /// and an unchanged one are the states whose verdicts differ most, and an
    /// absent line would give them one key.
    #[test]
    fn every_state_of_a_prior_version_keys_apart() {
        let keys = [
            Prior::Unchanged.key(),
            Prior::Added.key(),
            Prior::Committed {
                digest: "sha256:one",
                facets: &Mapping::default(),
            }
            .key(),
            Prior::Committed {
                digest: "sha256:two",
                facets: &Mapping::default(),
            }
            .key(),
        ];
        for (index, key) in keys.iter().enumerate() {
            for other in &keys[index + 1..] {
                assert_ne!(key, other, "two states of the prior version share a key");
            }
        }
    }

    /// A manifest this reader cannot parse is refused whole.
    ///
    /// A dropped line reads as a document that did not change, which is a
    /// promotion nothing counts. So every one of these is an error rather than
    /// a shorter change.
    #[test]
    fn a_manifest_this_reader_cannot_parse_is_refused() {
        for manifest in [
            "",
            "headwater change 2\nadded\tdocs/a.md\n",
            &format!("{FORMAT}\nremoved\tdocs/a.md\n"),
            &format!("{FORMAT}\nadded\n"),
            &format!("{FORMAT}\nadded\tdocs/a.md\textra\n"),
            &format!("{FORMAT}\nprior\tdocs/a.md\n"),
            &format!("{FORMAT}\nprior\tdocs/a.md\tprior/a.md\textra\n"),
            &format!("{FORMAT}\nadded\tdocs/a.md\nadded\tdocs/a.md\n"),
        ] {
            assert!(
                Change::read(manifest, tree(&[("prior/a.md", ASSERTED)])).is_err(),
                "read as a change: {manifest:?}"
            );
        }
    }

    /// What the run reports about its own input.
    #[test]
    fn a_change_states_what_it_named() {
        let manifest = format!(
            "{FORMAT}\nadded\tdocs/b.md\nprior\tdocs/a.md\tprior/a.md\nprior\tdocs/c.md\tgone.md\n"
        );
        let change = Change::read(&manifest, tree(&[("prior/a.md", ASSERTED)])).expect("a change");
        assert_eq!(
            change.named(),
            Named {
                documents: 3,
                added: 1,
                carried: 1,
                unreadable: 1,
            }
        );
    }
}
