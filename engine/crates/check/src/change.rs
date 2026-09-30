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
//!
//! # A path that binds to nothing is the same defect, and syntax cannot see it
//!
//! The sentence above is about a line this reader drops. A line it keeps and
//! that names a path no census row holds costs exactly the same thing: the
//! document it meant to name reads as one that did not change, and the count
//! comes back zero over a run that reported success. One character in a path is
//! enough, and so is a leading `./`.
//!
//! So a manifest is read in two steps and only the second one produces a value
//! this engine will run against. [`Unbound::read`] decides the syntax and reads
//! the bytes. [`Unbound::bind`] holds every path it names against the corpus the
//! run walked, and a path that matches nothing there is carried as
//! [`Held::Unmatched`], counted, and named in the report. It is not a refusal,
//! because a real change carries files that are not corpus documents at all: a
//! caller who writes `git diff --name-only` into a manifest names source files,
//! and refusing those would make the honest wrapper the one thing this engine
//! declines to accept.
//!
//! **No path is normalized.** `./docs/a.md` matches no census row and is
//! reported as one that matched nothing. A normalizer here would be one form
//! silently accepted and an invitation to the next one, and the duplicate guard
//! below would then have two spellings of one path to reconcile.
//!
//! # A re-verification the author states
//!
//! A `verified\t<path>` line states that the author re-read the document at
//! `<path>` in this change. It is a statement about the change and not a
//! version of the document, so it is held apart from the lines above and it
//! is never a [`Prior`]: a rule that reads the prior version does not see it,
//! and [`crate::suspect`] is the one rule that reads it. The path may also
//! carry an `added` or a `prior` line, or none, because an author can re-read
//! a document and change nothing in it (#1376).
//!
//! It exists for the second change of one day. A change states a
//! re-verification by moving the document's freshness facet, and a facet that
//! already reads today cannot move. The date does not decide instead, for the
//! reason [`crate::suspect`] gives.
//!
//! # What a caller may state, and what this engine cannot check
//!
//! A manifest that names `added` for a document that already stood, or that
//! omits a document the change carried, is a manifest that lies. Neither is
//! detectable without the history that spec 12 rules out as an input. That is
//! the trust boundary rather than a defect: the caller names the change, this
//! engine reads what the names reach, and every reading here is worth what the
//! caller's statement is worth.

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
    /// The caller named a path that the corpus of this run does not hold, and
    /// the version that stood at it, where the caller named one and this
    /// engine read it.
    ///
    /// No document-scoped instance exists over such a path, because there is
    /// no row to instantiate over. The prior version is kept rather than
    /// dropped, because a path that carried one and holds no row is the shape
    /// a deletion takes, and those bytes are the only evidence left of what
    /// stood there. Until [`Change::departed`] they were read, hashed, and
    /// then thrown away here.
    ///
    /// It is not a deletion by itself. A manifest writes `prior` for an edit
    /// as well as for a deletion, so an edit to a Markdown file that carries
    /// front matter and stands outside the corpus root reaches this arm
    /// holding one. What separates the two is the content, and
    /// [`crate::retention`] is the rule that reads it. A source file reaches
    /// this arm holding nothing, because bytes that are no document do not
    /// parse as one and [`read_prior`] has already said so.
    Unmatched { prior: Option<(String, Document)> },
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

/// A manifest that has been read and not yet held against a corpus.
///
/// It is a type of its own so that the binding step cannot be forgotten. A
/// [`Change`] is the only value [`crate::Context::over`] accepts, and
/// [`Unbound::bind`] is the only way to make one, so no run can be scoped to a
/// set of paths that nothing checked against the corpus in front of it.
#[derive(Clone, Debug)]
pub struct Unbound {
    entries: Vec<(String, Held)>,
    /// The paths a `verified` line names, in path order. See the module
    /// comment.
    verified: Vec<String>,
}

/// The documents one change carries, each held against the corpus this run
/// walked.
#[derive(Clone, Debug, Default)]
pub struct Change {
    /// In path order, so a lookup is a binary search and a report is a function
    /// of the change rather than of the order a caller wrote it in.
    entries: Vec<(String, Held)>,
    /// The paths a `verified` line names and a census row holds, in path
    /// order. One that no row holds is in `unverifiable` instead.
    verified: Vec<String>,
    /// The paths a `verified` line names and no census row holds, in path
    /// order. They are reported with the unmatched entries.
    unverifiable: Vec<String>,
}

/// What a run injected, for the report that states its own inputs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Named {
    /// Documents the manifest named, whatever became of each one, each
    /// counted once. A path that only a `verified` line names is one of them,
    /// so every count below is "of those" (#1376).
    pub documents: usize,
    /// Of those, the ones the change adds.
    pub added: usize,
    /// Of those, the ones that bound to a document of this corpus and whose
    /// prior version this run read.
    ///
    /// It counts bindings rather than file opens. An entry whose prior version
    /// opened and whose path reached no census row is counted below, because a
    /// line that said otherwise would tell a caller the injection worked when it
    /// reached nothing.
    pub carried: usize,
    /// Of those, the ones whose prior version this run could not read. Each one
    /// is a skipped instance with a reason rather than a pass.
    pub unreadable: usize,
    /// The paths this manifest named that no row of this corpus holds, each
    /// counted once. Nothing is checked over one, so none of them is a
    /// skipped instance either, and this count is the only place one is
    /// reported.
    ///
    /// It is the length of [`Change::unmatched`], and it is computed from that
    /// list, so the count and the list a report writes cannot disagree. A path
    /// that only a `verified` line names is counted here and in `documents`
    /// (#1376).
    pub unmatched: usize,
    /// Of those, the ones a `verified` line names that a row of this corpus
    /// holds. One that no row holds is counted in `unmatched` instead. A
    /// document can be counted here and in one of the counts above, because
    /// a `verified` line names no version of it and may sit beside a line that
    /// does. So this count is a reading and not one of the classes above.
    pub verified: usize,
    /// Of those, the ones that only a `verified` line names and that a row of
    /// this corpus holds. It is the class `verified` is not: a path that also
    /// has an `added` or `prior` line is counted in that line's class, and one
    /// that binds to no row is counted in `unmatched`.
    ///
    /// With it the classes are disjoint and they sum:
    /// `added + carried + unreadable + unmatched + verified_alone == documents`
    /// (#1398).
    pub verified_alone: usize,
}

impl Unbound {
    /// Read a manifest, and the prior version each line names.
    ///
    /// `open` is how a named file becomes bytes, so a test supplies a tree that
    /// no filesystem holds and the CLI supplies the filesystem.
    pub fn read(
        manifest: &str,
        open: impl Fn(&Path) -> std::io::Result<Vec<u8>>,
    ) -> Result<Unbound, String> {
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
        let mut verified: Vec<String> = Vec::new();
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
                (Some("verified"), Some(path), None, _) => {
                    if verified.iter().any(|known| known == path) {
                        return Err(format!(
                            "the change manifest states `{path}` verified twice"
                        ));
                    }
                    verified.push(path.to_string());
                    continue;
                }
                _ => {
                    return Err(format!(
                        "line {number} of the change manifest is none of `added\\t<path>`, \
                         `prior\\t<path>\\t<file>` and `verified\\t<path>`: `{line}`"
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
        verified.sort();
        Ok(Unbound { entries, verified })
    }

    /// Read a manifest from a file, which is what `headwater check --change`
    /// does. A path inside the manifest is resolved as written, because the
    /// caller wrote it and the prior versions sit outside the corpus.
    pub fn at(manifest: &Path) -> Result<Unbound, String> {
        let text = std::fs::read_to_string(manifest)
            .map_err(|error| format!("{}: {error}", manifest.display()))?;
        Unbound::read(&text, |path| std::fs::read(path))
    }

    /// Hold every path this manifest names against the corpus the run walked.
    ///
    /// `holds` answers whether the walk read a file at that path. It is the
    /// census's own set rather than the set of documents an instance exists
    /// over, so a caller who names a file the corpus holds and no check reads
    /// gets no warning about a path they wrote correctly.
    ///
    /// The corpus is the argument rather than something this module reaches,
    /// for the reason the check layer takes a census: two walks of one tree can
    /// disagree, and the pair that would disagree here is the set a run checks
    /// and the set a manifest is measured against.
    pub fn bind(self, holds: impl Fn(&str) -> bool) -> Change {
        let (verified, unverifiable) = self.verified.into_iter().partition(|path| holds(path));
        Change {
            verified,
            unverifiable,
            entries: self
                .entries
                .into_iter()
                .map(|(path, held)| match (holds(&path), held) {
                    (true, held) => (path, held),
                    // The path binds to no row, and what the caller named it
                    // as decides what is left of it. A `prior` line carried
                    // bytes, and they are held. An `added` line carried none,
                    // and a prior version this engine could not read leaves
                    // nothing to hold either: the reason it names is about a
                    // file no check will be handed.
                    (false, Held::Committed { digest, document }) => (
                        path,
                        Held::Unmatched {
                            prior: Some((digest, document)),
                        },
                    ),
                    (false, _) => (path, Held::Unmatched { prior: None }),
                })
                .collect(),
        }
    }

    /// The paths this manifest names, in path order.
    ///
    /// A caller reads it to say what it named, before anything binds. Nothing
    /// in a run uses it: a run reads [`Change`] and never this.
    pub fn paths(&self) -> Vec<&str> {
        self.entries.iter().map(|(path, _)| path.as_str()).collect()
    }

    /// The paths a `verified` line names, in path order.
    pub fn verified(&self) -> Vec<&str> {
        self.verified.iter().map(String::as_str).collect()
    }
}

impl Change {
    /// What this change named, for the report that states its own inputs.
    pub fn named(&self) -> Named {
        // Every path any line names, once: a `verified` path that also has an
        // `added` or `prior` line is one document, and one with neither is a
        // document too, so that `unmatched` never counts past `documents`.
        let mut paths: Vec<&str> = self
            .entries
            .iter()
            .map(|(path, _)| path.as_str())
            .chain(self.verified.iter().map(String::as_str))
            .chain(self.unverifiable.iter().map(String::as_str))
            .collect();
        paths.sort_unstable();
        paths.dedup();
        let mut named = Named {
            documents: paths.len(),
            verified: self.verified.len(),
            ..Named::default()
        };
        for (_, held) in &self.entries {
            match held {
                Held::Added => named.added += 1,
                Held::Committed { .. } => named.carried += 1,
                Held::Unreadable { .. } => named.unreadable += 1,
                // Counted below, from the list, with the `verified` paths
                // that bind to no row.
                Held::Unmatched { .. } => {}
            }
        }
        named.unmatched = self.unmatched().len();
        // A bound `verified` path that no entry names is a document of its own
        // class, and the only one the classes above do not hold (#1398).
        named.verified_alone = self
            .verified
            .iter()
            .filter(|path| {
                self.entries
                    .binary_search_by(|(known, _)| known.as_str().cmp(path.as_str()))
                    .is_err()
            })
            .count();
        named
    }

    /// The paths this change named that no row of the corpus holds, in path
    /// order.
    ///
    /// Named rather than counted alone, because a caller who mistyped one
    /// character needs to see which path, and a count sends them to read their
    /// own manifest against a census by hand.
    ///
    /// A `verified` line that names such a path is listed too, once, because
    /// it is the same mistyped path and it states nothing about a document.
    pub fn unmatched(&self) -> Vec<&str> {
        let mut paths: Vec<&str> = self
            .entries
            .iter()
            .filter(|(_, held)| matches!(held, Held::Unmatched { .. }))
            .map(|(path, _)| path.as_str())
            .chain(self.unverifiable.iter().map(String::as_str))
            .collect();
        paths.sort_unstable();
        paths.dedup();
        paths
    }

    /// Whether the change states that its author re-read the document at
    /// `path`. See the module comment.
    pub(crate) fn verified(&self, path: &str) -> bool {
        self.verified
            .binary_search_by(|known| known.as_str().cmp(path))
            .is_ok()
    }

    /// Every path this change named that no row of this corpus holds, and
    /// whose prior version this run read, in path order.
    ///
    /// This is what a document leaving the corpus looks like from inside the
    /// engine, and it is the only reading of one available: the working tree
    /// holds no file there, so the census has no row and no document-scoped
    /// instance exists. It is deliberately not called a deletion. A rename
    /// inside the corpus never reaches it, because the manifest names the path
    /// the document arrived at and the census holds a row there. A file whose
    /// prior bytes are no document never reaches it either, because they did
    /// not parse. What does reach it beside a deletion is a document that
    /// moved out of the corpus root, and that is a departure by every reading
    /// this engine has.
    ///
    /// A governed document whose *prior* front matter did not parse is absent
    /// here, and this is the one reading that could hide a deletion. It costs
    /// nothing to accept: a document whose committed front matter does not
    /// parse fails the run that committed it.
    pub fn departed(&self) -> Vec<Departed<'_>> {
        self.entries
            .iter()
            .filter_map(|(path, held)| match held {
                Held::Unmatched {
                    prior: Some((digest, document)),
                } => Some(Departed {
                    path: path.as_str(),
                    digest: digest.as_str(),
                    facets: &document.facets,
                }),
                _ => None,
            })
            .collect()
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
            // Unreachable: the census holds no row at this path, so no instance
            // exists to ask. It is an arm rather than a fallthrough because the
            // day an instance does exist over an unmatched path, the answer has
            // to be the one below and never a silent `Unchanged`.
            Held::Unmatched { .. } => Err(UNMATCHED),
        }
    }
}

/// One path a change named that this corpus holds no row at, and the version
/// of it the caller supplied.
///
/// Borrowed from the [`Change`], the way [`Prior::Committed`] is, and for the
/// same reason: the bytes were read once and no phase of a run holds a second
/// copy of them.
#[derive(Clone, Copy, Debug)]
pub struct Departed<'a> {
    /// As the manifest wrote it. Nothing normalizes a path here, on the terms
    /// the module comment states.
    pub path: &'a str,
    /// Over the prior bytes, by the function the census hashes a document
    /// with, so a key built from it is comparable with every other input.
    pub digest: &'a str,
    /// The front matter of the version that stood there. A file that is no
    /// document carries an empty mapping rather than nothing, which is what
    /// makes the reading of it a judgment the caller states.
    pub facets: &'a Mapping,
}

/// The reason an instance over an unmatched path would carry. See
/// [`Change::prior_of`]: no instance exists over one today.
const UNMATCHED: &str = "the change manifest names this path and this corpus holds no row at it";

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

    /// A corpus that holds every path these manifests name. The one test about
    /// binding supplies its own.
    fn every(_: &str) -> bool {
        true
    }

    fn manifest() -> String {
        format!("{FORMAT}\nadded\tdocs/b.md\nprior\tdocs/a.md\tprior/a.md\n")
    }

    /// The three states a document of a change can be in, and the fourth that a
    /// check never reaches.
    #[test]
    fn a_change_holds_three_states_and_refuses_to_hand_over_the_fourth() {
        let change = Unbound::read(&manifest(), tree(&[("prior/a.md", ASSERTED)]))
            .expect("a change")
            .bind(every);
        assert!(matches!(
            change.prior_of("docs/a.md"),
            Ok(Prior::Committed { .. })
        ));
        assert_eq!(change.prior_of("docs/b.md"), Ok(Prior::Added));
        // Named by nothing, so it stands as it stood.
        assert_eq!(change.prior_of("docs/c.md"), Ok(Prior::Unchanged));

        let missing = Unbound::read(&manifest(), tree(&[]))
            .expect("a change")
            .bind(every);
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
            &format!("{FORMAT}\nverified\n"),
            &format!("{FORMAT}\nverified\tdocs/a.md\textra\n"),
            &format!("{FORMAT}\nverified\tdocs/a.md\nverified\tdocs/a.md\n"),
        ] {
            assert!(
                Unbound::read(manifest, tree(&[("prior/a.md", ASSERTED)])).is_err(),
                "read as a change: {manifest:?}"
            );
        }
    }

    /// A `verified` line states a re-verification and names no version of the
    /// document, so the path it names may carry a `prior` line, an `added`
    /// line or neither, and its prior version is what those lines say
    /// (#1376). A path no row holds is reported with the unmatched paths, once.
    #[test]
    fn a_verified_line_states_a_re_verification_and_no_version() {
        let manifest = format!(
            "{FORMAT}\nverified\tdocs/c.md\nprior\tdocs/a.md\tprior/a.md\nverified\tdocs/a.md\n\
             verified\tdocs/gone.md\nprior\tdocs/gone.md\tprior/a.md\nverified\tdocs/typo.md\n"
        );
        let unbound =
            Unbound::read(&manifest, tree(&[("prior/a.md", ASSERTED)])).expect("a change");
        assert_eq!(
            unbound.verified(),
            vec!["docs/a.md", "docs/c.md", "docs/gone.md", "docs/typo.md"]
        );
        let change = unbound.bind(|path| path == "docs/a.md" || path == "docs/c.md");

        assert!(change.verified("docs/a.md"));
        assert!(change.verified("docs/c.md"));
        assert!(!change.verified("docs/b.md"));
        assert!(!change.verified("docs/typo.md"), "no row holds it");
        assert!(matches!(
            change.prior_of("docs/a.md"),
            Ok(Prior::Committed { .. })
        ));
        assert_eq!(change.prior_of("docs/c.md"), Ok(Prior::Unchanged));
        assert_eq!(change.unmatched(), vec!["docs/gone.md", "docs/typo.md"]);
        assert_eq!(
            change.named(),
            Named {
                documents: 4,
                added: 0,
                carried: 1,
                unreadable: 0,
                unmatched: 2,
                verified: 2,
                verified_alone: 1,
            }
        );
        // Every count is "of those" documents, as each report words it. A
        // path named only in a `verified` line that binds to no row is one
        // of the documents, so the report never says more paths named no row
        // than documents were named.
        assert!(change.named().unmatched <= change.named().documents);
        // The count and the list are two readings of one set, and a report
        // writes both. `docs/typo.md` is named only in a `verified` line, and
        // `docs/gone.md` in both kinds, so it counts once.
        assert_eq!(change.named().unmatched, change.unmatched().len());
        // `docs/c.md` is named only in a `verified` line and binds, so it is
        // the one document of its own class, and the classes sum (#1398).
        let named = change.named();
        assert_eq!(
            named.added + named.carried + named.unreadable + named.unmatched + named.verified_alone,
            named.documents
        );
    }

    /// What the run reports about its own input.
    #[test]
    fn a_change_states_what_it_named() {
        let manifest = format!(
            "{FORMAT}\nadded\tdocs/b.md\nprior\tdocs/a.md\tprior/a.md\nprior\tdocs/c.md\tgone.md\n"
        );
        let change = Unbound::read(&manifest, tree(&[("prior/a.md", ASSERTED)]))
            .expect("a change")
            .bind(every);
        assert_eq!(
            change.named(),
            Named {
                documents: 3,
                added: 1,
                carried: 1,
                unreadable: 1,
                unmatched: 0,
                verified: 0,
                verified_alone: 0,
            }
        );
    }

    /// A path the corpus holds no row at is carried as unmatched, and it never
    /// answers as a document that did not change.
    ///
    /// The count is what the report reads, and `carried` is the field that was
    /// wrong before this arm existed: the prior version of `docs/a.md` opens
    /// either way, so a count of file opens says one document was read over a
    /// manifest that reached none.
    #[test]
    fn a_path_no_row_holds_binds_to_nothing_and_says_so() {
        let unbound =
            || Unbound::read(&manifest(), tree(&[("prior/a.md", ASSERTED)])).expect("a change");
        assert_eq!(unbound().paths(), vec!["docs/a.md", "docs/b.md"]);

        let bound = unbound().bind(|path| path == "docs/b.md");
        assert_eq!(
            bound.named(),
            Named {
                documents: 2,
                added: 1,
                carried: 0,
                unreadable: 0,
                unmatched: 1,
                verified: 0,
                verified_alone: 0,
            }
        );
        assert_eq!(bound.unmatched(), vec!["docs/a.md"]);

        // And it is never answered as a document that stood where it stands. An
        // instance cannot reach this today, because the corpus holds no row to
        // create one over, and the answer is the reason rather than a state.
        assert!(bound.prior_of("docs/a.md").is_err());
        assert_eq!(bound.prior_of("docs/b.md"), Ok(Prior::Added));
    }
}
