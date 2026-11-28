// SPDX-License-Identifier: Apache-2.0
//! The change: the documents one change carries, and the version of each that
//! stood before it.
//!
//! [The check layer](../../../../docs/spec/check-layer.md#temporal-inputs-the-clock-and-the-prior-version)
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
//! belong to the caller, and [the check layer](../../../../docs/spec/check-layer.md) states what the caller must anchor "prior"
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
//! [the assurance model](../../../../docs/spec/assurance-model.md#no-silent-passes-every-document-is-accounted-for)
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
//! declines to accept. The prior version such a path names is kept with it:
//! its front matter where it parsed, which [`Change::departed`] hands on, and
//! the reason it did not read where it did not, which [`Change::unread`] hands
//! on (HW-OBL-0127).
//!
//! **No path is normalized.** `./docs/a.md` matches no census row and is
//! reported as one that matched nothing. A normalizer here would be one form
//! silently accepted and an invitation to the next one, and the duplicate guard
//! below would then have two spellings of one path to reconcile.
//!
//! One mapping is not a normalizer, because it removes no spelling: a
//! manifest's paths are written from the git top level, which is where
//! `headwater change` writes them, and [`Unbound::under`] strips the corpus
//! root's own prefix below that top level from each one before the bind. A
//! path outside that prefix reaches no rule and is reported as written
//! (#2098).
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
//! A `verified\t<path>\t<target>` line states less: that the author re-read
//! the one edge of the document at `<path>` whose target is `<target>`. A
//! re-verified document no longer stamps every edge it declares, because each
//! edge carries its own `verified_revision`, so a stamp also needs the edge's
//! target named, and this line is how an author names it without carrying
//! the target in the change (#1520). `<target>` is the target as the entry
//! writes it, or a path its pattern matches. It is not held against the corpus,
//! because a governed source file is no corpus document. `<path>` is held, as
//! a two-field line's path is.
//!
//! # What a caller may state, and what this engine cannot check
//!
//! A manifest that names `added` for a document that already stood, or that
//! omits a document the change carried, is a manifest that lies. Neither is
//! detectable without the history that [the check layer](../../../../docs/spec/check-layer.md) rules out as an input. That is
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
    /// the module comment: over a path a row holds, it never reaches a check.
    ///
    /// `no_front_matter` says that the bytes parsed as far as the first line
    /// and held no block of front matter at all. Such a file declared no
    /// state, so no rule could have refused its deletion, and
    /// [`crate::retention`] reads the flag to leave it out (HW-OBL-0127).
    Unreadable { why: String, no_front_matter: bool },
    /// The caller named a path that the corpus of this run does not hold, and
    /// what this engine made of the version that stood at it, where the caller
    /// named one.
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
    /// [`crate::retention`] is the rule that reads it.
    ///
    /// A prior version that did not read is kept as [`Before::Unread`] with
    /// its reason, and is not dropped (HW-OBL-0127). A source file reaches
    /// this arm that way, because bytes that are no document do not parse as
    /// one. So does a governed document whose prior front matter did not
    /// parse, and that is a deletion no rule can read the state of.
    /// [`Change::unread`] hands both to a caller, and the caller separates
    /// them.
    ///
    /// `added` keeps what the line said: `added` rather than `prior`. Without
    /// it an added source file and an edited one bind to one shape, because
    /// neither holds a prior version here, and [`crate::scope_ratchet`] would
    /// read every edit to a file that already stood ungoverned as new debt
    /// (#1573). See [`Change::added`].
    Unmatched { prior: Before, added: bool },
}

/// What a path that binds to no row carried before the change.
#[derive(Clone, Debug)]
enum Before {
    /// The line named no prior version that opened as bytes for this engine
    /// to keep: an `added` line, which carries none.
    Nothing,
    /// The prior version, read and parsed.
    Read { digest: String, document: Document },
    /// The line named a prior version and this engine could not read it. The
    /// fields are those of [`Held::Unreadable`], which it was before the bind.
    Unread { why: String, no_front_matter: bool },
}

/// What a change states about one edge beside the prior version of the
/// document that declared it, and only for a check that declared
/// `NEEDS_DECLARER_PRIOR`. None of the four is a version of a document, so
/// each is held apart from [`Prior`], and each is a line of the cache key.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Stated {
    /// A `verified` line names the declaring document (#1376).
    pub(crate) document: bool,
    /// An `added` or a `prior` line names the edge's target, or a path the
    /// target's pattern matches, a deleted file included (#1520).
    pub(crate) target: bool,
    /// A `verified\t<document>\t<target>` line names this edge (#1520).
    pub(crate) edge: bool,
    /// The run asked for a baseline stamp, with `--baseline` (#2016). It is
    /// no statement of the change, and a run with no change can make it.
    pub(crate) baseline: bool,
}

impl Stated {
    /// What a run with no change, or an instance that reads no declarer,
    /// states.
    pub(crate) const NONE: Stated = Stated {
        document: false,
        target: false,
        edge: false,
        baseline: false,
    };

    /// What `change` states about the edge that the document at `document`
    /// declares, whose target `names` spell: the target as written and every
    /// path it reaches. `patterns` are the target's normalized patterns, and a
    /// path one of them matches names the target too, though the tree no
    /// longer holds it: a change that deletes a file under a pattern carries
    /// the file in a `prior` line, and the author read the edge to delete it.
    pub(crate) fn of(
        change: &Change,
        document: &str,
        names: &[&str],
        patterns: &[headwater_meta::Pattern],
    ) -> Stated {
        let reaches = |path: &str| reaches(names, patterns, path);
        Stated {
            document: change.verified(document),
            target: change.entries.iter().any(|(path, _)| reaches(path)),
            edge: change
                .edges
                .iter()
                .any(|(known, target)| known == document && reaches(target)),
            baseline: false,
        }
    }
}

/// Whether `path` names a target whose literal spellings are `names` and whose
/// normalized patterns are `patterns`: it is one of the spellings, or a
/// pattern matches it. [`Stated::of`] reads it, and so does the report of a
/// `verified` pair that names no edge, so the two cannot disagree (#1631).
pub(crate) fn reaches(names: &[&str], patterns: &[headwater_meta::Pattern], path: &str) -> bool {
    names.contains(&path) || patterns.iter().any(|pattern| pattern.matches(path))
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
    /// The `(document, target)` pairs a three-field `verified` line names,
    /// in order. See the module comment.
    edges: Vec<(String, String)>,
    /// The paths [`Unbound::under`] found outside the corpus root, as the
    /// manifest wrote them, in path order and once each. No rule reads one.
    outside: Vec<String>,
    /// The digests of every prior version that opened, by path, in path
    /// order. See [`Digests`].
    prints: Vec<(String, Digests)>,
}

/// The two digests of one prior version that opened, kept whether or not its
/// bytes parsed as a document (#2100).
///
/// A source file's prior bytes are no document, so [`Held`] keeps nothing of
/// them. These two are what is left, and the white-space classification of
/// [`crate::Run::classes`] reads them beside the current bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Digests {
    /// Over the raw bytes, by the function the census and
    /// `headwater_graph::anchors::tree_revision` hash a file with.
    pub raw: String,
    /// Over [`normalized`] of the same bytes, by the same function.
    pub normalized: String,
}

impl Digests {
    /// Both digests of `bytes`.
    pub fn of(bytes: &[u8]) -> Digests {
        Digests {
            raw: headwater_hash::digest(bytes),
            normalized: headwater_hash::digest(&normalized(bytes)),
        }
    }
}

/// The bytes with every white-space-only difference taken out, which is what
/// two versions share when they differ in white space alone (#2100).
///
/// CRLF becomes LF. Trailing spaces and tabs are removed from each line, a
/// line that is then empty is dropped, and the lines left are joined with LF.
/// Leading white space is content, because indentation carries meaning in
/// many languages. It works on bytes and reads no language and no encoding,
/// so it decodes nothing. A Markdown hard break written as two trailing
/// spaces is therefore white space here, which is acceptable because a
/// control spares the class only when an adopter declares it.
pub fn normalized(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    for line in bytes.split(|byte| *byte == b'\n') {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        let end = line
            .iter()
            .rposition(|byte| !matches!(byte, b' ' | b'\t' | b'\r'))
            .map_or(0, |last| last + 1);
        if end == 0 {
            continue;
        }
        if !out.is_empty() {
            out.push(b'\n');
        }
        out.extend_from_slice(&line[..end]);
    }
    out
}

/// What a change line said about one touched path, for the classification of
/// [`crate::Run::classes`] (#2100).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Touched<'a> {
    /// An `added` line names it.
    Added,
    /// A `prior` line names it, with the digests of the prior version, or
    /// `None` where that version did not open.
    Prior(Option<&'a Digests>),
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
    /// The `(document, target)` pairs a three-field `verified` line names
    /// and whose document a census row holds, in order. A pair whose document
    /// no row holds puts the document in `unverifiable` instead. The target
    /// is not held against the census, because it is what a document
    /// governs, and a governed source file is no corpus document (#1520).
    edges: Vec<(String, String)>,
    /// Every `(document, target)` pair a three-field `verified` line names,
    /// held or not, in the order [`Unbound`] holds them. The report lists
    /// each one that names no edge, and a pair whose document no row holds
    /// would otherwise reach it only as its document path (#1631).
    pairs: Vec<(String, String)>,
    /// The paths outside the corpus root, as [`Unbound::under`] found
    /// them. They are reported with the unmatched entries (#2098).
    outside: Vec<String>,
    /// As [`Unbound`] holds them, after the bind. See [`Change::touched`].
    prints: Vec<(String, Digests)>,
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
        let mut prints: Vec<(String, Digests)> = Vec::new();
        let mut verified: Vec<String> = Vec::new();
        let mut edges: Vec<(String, String)> = Vec::new();
        for (number, line) in lines.enumerate() {
            let number = number + 2;
            if line.trim().is_empty() {
                continue;
            }
            let mut fields = line.split('\t');
            let (path, held) = match (fields.next(), fields.next(), fields.next(), fields.next()) {
                (Some("added"), Some(path), None, _) => (path, Held::Added),
                (Some("prior"), Some(path), Some(source), None) => {
                    let (held, digests) = read_prior(source, &open);
                    if let Some(digests) = digests {
                        prints.push((path.to_string(), digests));
                    }
                    (path, held)
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
                // One edge of the document re-read, and not the document
                // (#1520). See the module comment.
                (Some("verified"), Some(path), Some(target), None) => {
                    if edges
                        .iter()
                        .any(|(known, to)| known == path && to == target)
                    {
                        return Err(format!(
                            "the change manifest states the edge from `{path}` to `{target}` \
                             verified twice"
                        ));
                    }
                    edges.push((path.to_string(), target.to_string()));
                    continue;
                }
                _ => {
                    return Err(format!(
                        "line {number} of the change manifest is none of `added\\t<path>`, \
                         `prior\\t<path>\\t<file>`, `verified\\t<path>` and \
                         `verified\\t<path>\\t<target>`: `{line}`"
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
        prints.sort_by(|(a, _), (b, _)| a.cmp(b));
        verified.sort();
        edges.sort();
        Ok(Unbound {
            entries,
            verified,
            edges,
            outside: Vec::new(),
            prints,
        })
    }

    /// Read every path from the corpus root, given `prefix`, the path of the
    /// root below the git top level that `headwater change` writes from, as
    /// `git rev-parse --show-prefix` prints it (#2098).
    ///
    /// A path under `prefix` loses it. A path outside it is taken out of
    /// every line, so no rule reads it, and [`Change::unmatched`] names it as
    /// the manifest wrote it: a change in a larger repository carries files
    /// of other trees, and refusing the run for one would refuse every such
    /// change. An edge is kept only when both of its paths are under
    /// `prefix`. An empty `prefix` changes nothing, and the third field of a
    /// `prior` line was read in [`Unbound::read`] and is no path here.
    pub fn under(self, prefix: &str) -> Unbound {
        let mut outside = self.outside;
        let mut inside = |path: String| match path.strip_prefix(prefix) {
            Some(rest) => Some(rest.to_string()),
            None => {
                outside.push(path);
                None
            }
        };
        let entries = (self.entries.into_iter())
            .filter_map(|(path, held)| Some((inside(path)?, held)))
            .collect();
        // Each path here is a path of an entry above, which has already
        // named it outside where it is, so this list only follows it.
        let prints = (self.prints.into_iter())
            .filter_map(|(path, digests)| Some((path.strip_prefix(prefix)?.to_string(), digests)))
            .collect();
        let verified = self.verified.into_iter().filter_map(&mut inside).collect();
        let edges = (self.edges.into_iter())
            .filter_map(
                |(document, target)| match (inside(document), inside(target)) {
                    (Some(document), Some(target)) => Some((document, target)),
                    _ => None,
                },
            )
            .collect();
        outside.sort();
        outside.dedup();
        Unbound {
            entries,
            verified,
            edges,
            outside,
            prints,
        }
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
        let (verified, mut unverifiable): (Vec<String>, Vec<String>) =
            self.verified.into_iter().partition(|path| holds(path));
        let pairs = self.edges.clone();
        let (edges, unheld): (Vec<_>, Vec<_>) = self
            .edges
            .into_iter()
            .partition(|(document, _)| holds(document));
        // An edge whose document no row holds is the same mistyped path a
        // two-field line would be, and it is reported the same way.
        unverifiable.extend(unheld.into_iter().map(|(document, _)| document));
        unverifiable.sort();
        unverifiable.dedup();
        Change {
            verified,
            unverifiable,
            edges,
            pairs,
            outside: self.outside,
            prints: self.prints,
            entries: self
                .entries
                .into_iter()
                .map(|(path, held)| match (holds(&path), held) {
                    (true, held) => (path, held),
                    // The path binds to no row, and what the caller named it
                    // as decides what is left of it. A `prior` line carried
                    // bytes, and they are held. An `added` line carried none.
                    (false, Held::Committed { digest, document }) => (
                        path,
                        Held::Unmatched {
                            prior: Before::Read { digest, document },
                            added: false,
                        },
                    ),
                    // A prior version this engine could not read keeps its
                    // reason. Until HW-OBL-0127 it was dropped here, and a
                    // deletion of a document whose front matter did not parse
                    // reached no rule. See `Change::unread`.
                    (
                        false,
                        Held::Unreadable {
                            why,
                            no_front_matter,
                        },
                    ) => (
                        path,
                        Held::Unmatched {
                            prior: Before::Unread {
                                why,
                                no_front_matter,
                            },
                            added: false,
                        },
                    ),
                    // The line's verb survives the bind, so an addition and
                    // an edit stay two facts. See `Held::Unmatched`.
                    (false, Held::Added) => (
                        path,
                        Held::Unmatched {
                            prior: Before::Nothing,
                            added: true,
                        },
                    ),
                    // Only a bind writes this arm, and a value is bound once.
                    (false, held @ Held::Unmatched { .. }) => (path, held),
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

    /// The `(document, target)` pairs a `verified\t<path>\t<target>` line
    /// names, in order.
    pub fn verified_edges(&self) -> Vec<(&str, &str)> {
        self.edges
            .iter()
            .map(|(document, target)| (document.as_str(), target.as_str()))
            .collect()
    }
}

impl Change {
    /// What this change named, for the report that states its own inputs.
    pub fn named(&self) -> Named {
        // Every path any line names, once: a `verified` path that also has an
        // `added` or `prior` line is one document, and one with neither is a
        // document too, so that `unmatched` never counts past `documents`.
        // A document that a `verified` line names, with or without a target,
        // and that a row holds, once (#1520).
        let mut stated: Vec<&str> = self
            .verified
            .iter()
            .map(String::as_str)
            .chain(self.edges.iter().map(|(document, _)| document.as_str()))
            .collect();
        stated.sort_unstable();
        stated.dedup();
        let mut paths: Vec<&str> = self
            .entries
            .iter()
            .map(|(path, _)| path.as_str())
            .chain(stated.iter().copied())
            .chain(self.unverifiable.iter().map(String::as_str))
            .collect();
        paths.sort_unstable();
        paths.dedup();
        let mut named = Named {
            // A path outside the root is written from another base, so it is
            // never one document with a corpus path of the same spelling, and
            // `unmatched` lists it apart for the same reason (#2098).
            documents: paths.len() + self.outside.len(),
            verified: stated.len(),
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
        named.verified_alone = stated
            .iter()
            .filter(|path| {
                self.entries
                    .binary_search_by(|(known, _)| known.as_str().cmp(path))
                    .is_err()
            })
            .count();
        named
    }

    /// The paths this change named that no row of the corpus holds, in path
    /// order, and then the paths outside the corpus root, in path order.
    ///
    /// Named rather than counted alone, because a caller who mistyped one
    /// character needs to see which path, and a count sends them to read their
    /// own manifest against a census by hand.
    ///
    /// A `verified` line that names such a path is listed too, once, because
    /// it is the same mistyped path and it states nothing about a document.
    ///
    /// A path outside the corpus root, which [`Unbound::under`] took out of
    /// every line, is listed as the manifest wrote it (#2098). It is never
    /// merged with a corpus path of the same spelling, because the two are
    /// written from different bases.
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
        paths.extend(self.outside.iter().map(String::as_str));
        paths
    }

    /// Every `(document, target)` pair a `verified\t<path>\t<target>` line
    /// names, whether or not a row holds its document, in the order
    /// [`Unbound::verified_edges`] gives them (#1631).
    pub fn verified_pairs(&self) -> impl Iterator<Item = (&str, &str)> {
        self.pairs
            .iter()
            .map(|(document, target)| (document.as_str(), target.as_str()))
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
    /// here, because every entry here carries the front matter it stood with
    /// and [`crate::scope_ratchet`] reads it. Nothing guarantees that such a
    /// document never stood: a document whose front matter does not parse
    /// passes a strict run, because no rule reads an unreadable census row. So
    /// [`Change::unread`] carries that shape apart, with the reason
    /// (HW-OBL-0127).
    pub fn departed(&self) -> Vec<Departed<'_>> {
        self.entries
            .iter()
            .filter_map(|(path, held)| match held {
                Held::Unmatched {
                    prior: Before::Read { digest, document },
                    ..
                } => Some(Departed {
                    path: path.as_str(),
                    digest: digest.as_str(),
                    facets: &document.facets,
                }),
                _ => None,
            })
            .collect()
    }

    /// Every path this change named that no row of this corpus holds, whose
    /// prior version the caller named and this run could not read, in path
    /// order (HW-OBL-0127).
    ///
    /// It is the other half of [`Change::departed`], and it is kept apart from
    /// it because no entry here carries front matter. Most entries are source
    /// files, whose bytes are no document. A governed document whose prior
    /// front matter did not parse is here too, and its deletion is the one
    /// that no rule can read the state of. This list does not separate the
    /// two, because the separation needs the taxonomy: [`crate::retention`]
    /// keeps a Markdown path that a shelf claims and that held a block of front
    /// matter, and reports it as a skip.
    pub fn unread(&self) -> Vec<Unread<'_>> {
        self.entries
            .iter()
            .filter_map(|(path, held)| match held {
                Held::Unmatched {
                    prior:
                        Before::Unread {
                            why,
                            no_front_matter,
                        },
                    ..
                } => Some(Unread {
                    path: path.as_str(),
                    why: why.as_str(),
                    no_front_matter: *no_front_matter,
                    digest: self
                        .prints
                        .binary_search_by(|(known, _)| known.as_str().cmp(path))
                        .ok()
                        .map(|index| self.prints[index].1.raw.as_str()),
                }),
                _ => None,
            })
            .collect()
    }

    /// Every document this change carries that a row of this corpus holds and
    /// whose prior version the engine read, in path order (#1573).
    ///
    /// It is the corpus-grained reading of [`Prior::Committed`]: a
    /// document-scoped check is handed one of these for its own row, and a
    /// corpus-scoped one that declared `NEEDS_PRIOR` is handed all of them.
    /// [`crate::scope_ratchet`] reads the governance entries each one wrote,
    /// to find a file that the change left with no edge. An added document,
    /// an unreadable prior and a path that no row holds are not here:
    /// [`Change::added`] and [`Change::departed`] carry the last two shapes a
    /// rule reads.
    pub fn committed(&self) -> Vec<Committed<'_>> {
        self.entries
            .iter()
            .filter_map(|(path, held)| match held {
                Held::Committed { digest, document } => Some(Committed {
                    path: path.as_str(),
                    digest: digest.as_str(),
                    facets: &document.facets,
                }),
                _ => None,
            })
            .collect()
    }

    /// Every path an `added` line names, whether or not a row of this corpus
    /// holds it, in path order (#1573).
    ///
    /// A source file of a corpus whose census holds no row for it binds to
    /// nothing, and one whose census does holds a row that carries no
    /// document. The line said `added` in both, and that is the one fact
    /// [`crate::scope_ratchet`] reads. A `prior` line is never here, so an edit
    /// to a file that already stood is not an addition.
    pub fn added(&self) -> Vec<&str> {
        self.entries
            .iter()
            .filter(|(_, held)| matches!(held, Held::Added | Held::Unmatched { added: true, .. }))
            .map(|(path, _)| path.as_str())
            .collect()
    }

    /// Every path an `added` or a `prior` line names, in path order, with what
    /// the line said and the digests of the prior version that opened
    /// (#2100).
    ///
    /// It reads every such line, bound to a row or not, because a governed
    /// source file is no corpus document and binds to nothing. The readers
    /// above keep their shapes: this is the one that keeps a source file's
    /// prior digests, which [`Held`] drops with the bytes.
    pub fn touched(&self) -> impl Iterator<Item = (&str, Touched<'_>)> {
        self.entries.iter().map(|(path, held)| {
            let touched = match held {
                Held::Added | Held::Unmatched { added: true, .. } => Touched::Added,
                _ => Touched::Prior(
                    self.prints
                        .binary_search_by(|(known, _)| known.as_str().cmp(path))
                        .ok()
                        .map(|index| &self.prints[index].1),
                ),
            };
            (path.as_str(), touched)
        })
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
            Held::Unreadable { why, .. } => Err(why),
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

/// One path a change named that this corpus holds no row at, whose prior
/// version the caller named and this engine could not read. Borrowed from the
/// [`Change`], on the terms [`Departed`] states (HW-OBL-0127).
#[derive(Clone, Copy, Debug)]
pub struct Unread<'a> {
    /// As the manifest wrote it.
    pub path: &'a str,
    /// Why the prior version did not read: it did not open, it is not UTF-8,
    /// or it did not parse.
    pub why: &'a str,
    /// The bytes parsed as far as the first line and held no block of front
    /// matter at all. Such a file declared no state.
    pub no_front_matter: bool,
    /// Over the raw prior bytes, by the function the census hashes a document
    /// with, where they opened. A prior version that did not open has none.
    pub digest: Option<&'a str>,
}

impl Unread<'_> {
    /// Whether the version that stood here may have been a document that
    /// declared a state: the path passes the census's own extension test,
    /// which is exact and lower case, and the bytes held a block of front
    /// matter.
    ///
    /// A source file fails the first test, and a shelf's untyped `README.md`
    /// fails the second. Whether a shelf claims the path is the third test,
    /// and only a caller that holds the taxonomy can apply it.
    pub fn may_be_a_document(&self) -> bool {
        self.path.ends_with(".md") && !self.no_front_matter
    }
}

/// One document a change carries that a row of this corpus holds, and the
/// version of it that stood before the change. Borrowed from the [`Change`],
/// on the terms [`Departed`] states.
#[derive(Clone, Copy, Debug)]
pub struct Committed<'a> {
    /// As the manifest wrote it, which the bind matched to a row.
    pub path: &'a str,
    /// Over the prior bytes, by the function the census hashes a document
    /// with.
    pub digest: &'a str,
    /// The front matter of the version that stood before the change.
    pub facets: &'a Mapping,
}

/// The reason an instance over an unmatched path would carry. See
/// [`Change::prior_of`]: no instance exists over one today.
const UNMATCHED: &str = "the change manifest names this path and this corpus holds no row at it";

/// One prior version, read and parsed, or the reason it was neither, and the
/// digests of its bytes wherever they opened.
///
/// The digests are taken before the decode and the parse, so a source file,
/// whose bytes are no document, keeps them too (#2100).
fn read_prior(
    source: &str,
    open: impl Fn(&Path) -> std::io::Result<Vec<u8>>,
) -> (Held, Option<Digests>) {
    let bytes = match open(Path::new(source)) {
        Ok(bytes) => bytes,
        Err(error) => {
            return (
                Held::Unreadable {
                    why: format!("the prior version at `{source}` did not open: {error}"),
                    no_front_matter: false,
                },
                None,
            )
        }
    };
    // The digest is over the bytes, which is what the census hashes a document
    // with. So the prior version and the current one are hashed by one function
    // and their digests are comparable.
    let digests = Digests::of(&bytes);
    let digest = digests.raw.clone();
    let Ok(text) = String::from_utf8(bytes) else {
        return (
            Held::Unreadable {
                why: format!("the prior version at `{source}` is not UTF-8"),
                no_front_matter: false,
            },
            Some(digests),
        );
    };
    let held = match headwater_doc::parse(&text) {
        Ok(document) => Held::Committed { digest, document },
        // A file with no block of front matter at all is told apart from one
        // whose block did not load, because the first declared no state and
        // the second may have (HW-OBL-0127).
        Err(errors) => Held::Unreadable {
            why: format!(
                "the prior version at `{source}` did not parse: {} error(s)",
                errors.len()
            ),
            no_front_matter: errors
                .iter()
                .any(|error| error.reason == headwater_doc::Reason::NoFrontMatter),
        },
    };
    (held, Some(digests))
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

    /// A manifest written from the git top level, read from a corpus root at
    /// `sub/` below it (#2098). The prior bytes sit at `sub/prior/a.md`, the
    /// third field as written, and they are still held after the mapping.
    #[test]
    fn a_path_under_the_root_prefix_loses_it_and_one_outside_is_named() {
        let text = format!(
            "{FORMAT}\nadded\tsub/docs/b.md\nadded\tdocs/b.md\nprior\tsub/docs/a.md\t\
             sub/prior/a.md\nverified\tsub/docs/a.md\nverified\tother/c.md\nadded\tother/c.md\n\
             added\tsub/tools/z.sh\nadded\ttools/z.sh\n\
             verified\tsub/docs/a.md\tsub/tools/x.sh\nverified\tsub/docs/a.md\ttools/y.sh\n"
        );
        let open = tree(&[("sub/prior/a.md", ASSERTED)]);
        let unbound = Unbound::read(&text, open).expect("a change").under("sub/");
        assert_eq!(unbound.paths(), ["docs/a.md", "docs/b.md", "tools/z.sh"]);
        assert_eq!(unbound.verified(), ["docs/a.md"]);
        assert_eq!(unbound.verified_edges(), [("docs/a.md", "tools/x.sh")]);
        // The corpus holds every path but `tools/z.sh`, a source file.
        let change = unbound.bind(|path| path != "tools/z.sh");
        assert!(matches!(
            change.prior_of("docs/a.md"),
            Ok(Prior::Committed { .. })
        ));
        assert_eq!(change.prior_of("docs/b.md"), Ok(Prior::Added));
        // `docs/b.md` and `tools/z.sh` at the top level are not the corpus's
        // own: they reach no rule and are named as written, after the
        // corpus's unmatched `tools/z.sh` and apart from it. `other/c.md`,
        // named on two lines, is named once.
        assert_eq!(change.added(), ["docs/b.md", "tools/z.sh"]);
        assert_eq!(
            change.unmatched(),
            [
                "tools/z.sh",
                "docs/b.md",
                "other/c.md",
                "tools/y.sh",
                "tools/z.sh"
            ]
        );
        // Three documents of the corpus and four paths outside it, each a
        // document of its own, so the counts sum to `documents`.
        let named = change.named();
        assert_eq!(named.unmatched, 5);
        assert_eq!(named.documents, 7);
        assert_eq!(
            named.added + named.carried + named.unreadable + named.verified_alone + named.unmatched,
            named.documents
        );

        // An empty prefix is a root at the top level, and nothing moves.
        let same = Unbound::read(&manifest(), tree(&[("prior/a.md", ASSERTED)]))
            .expect("a change")
            .under("");
        assert_eq!(same.paths(), ["docs/a.md", "docs/b.md"]);
        assert!(same.bind(every).unmatched().is_empty());
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
            &format!("{FORMAT}\nverified\tdocs/a.md\ttools/x.sh\textra\n"),
            &format!("{FORMAT}\nverified\tdocs/a.md\nverified\tdocs/a.md\n"),
            &format!(
                "{FORMAT}\nverified\tdocs/a.md\ttools/x.sh\nverified\tdocs/a.md\ttools/x.sh\n"
            ),
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

    /// A `verified\t<document>\t<target>` line states that one edge was
    /// re-read, and not the document: the document is not re-verified by it.
    /// The target is held against nothing, because a governed source file is
    /// no corpus document, and a document no row holds is reported with the
    /// unmatched paths, as a two-field line's is. An `added` or a `prior` line
    /// names its path whether or not a row holds it (#1520).
    #[test]
    fn a_verified_edge_line_states_one_edge_and_not_the_document() {
        let manifest = format!(
            "{FORMAT}\nverified\tdocs/a.md\ttools/x.sh\nverified\tdocs/a.md\tsrc/**\n\
             verified\tdocs/typo.md\ttools/x.sh\nverified\tdocs/a.md\nprior\ttools/y.sh\tprior/y.sh\n\
             verified\tdocs/c.md\ttools/x.sh\n"
        );
        let unbound =
            Unbound::read(&manifest, tree(&[("prior/y.sh", "echo\n")])).expect("a change");
        assert_eq!(
            unbound.verified_edges(),
            vec![
                ("docs/a.md", "src/**"),
                ("docs/a.md", "tools/x.sh"),
                ("docs/c.md", "tools/x.sh"),
                ("docs/typo.md", "tools/x.sh"),
            ]
        );
        assert_eq!(unbound.verified(), vec!["docs/a.md"]);
        let change = unbound.bind(|path| path == "docs/a.md" || path == "docs/c.md");

        // What the change states about an edge whose target is written as
        // `target`, with no pattern.
        let stated = |document: &str, target: &str| Stated::of(&change, document, &[target], &[]);
        assert!(stated("docs/a.md", "tools/x.sh").edge);
        assert!(stated("docs/a.md", "src/**").edge);
        assert!(stated("docs/c.md", "tools/x.sh").edge);
        assert!(!stated("docs/c.md", "src/**").edge);
        assert!(
            !stated("docs/typo.md", "tools/x.sh").edge,
            "no row holds it"
        );
        // An edge line re-verifies the one edge and not the document.
        assert!(change.verified("docs/a.md"), "its own two-field line");
        assert!(!change.verified("docs/c.md"));
        assert!(!stated("docs/c.md", "tools/x.sh").document);
        // A governed file the change carries is named though no row holds it.
        assert!(stated("docs/c.md", "tools/y.sh").target);
        assert!(
            !stated("docs/c.md", "tools/x.sh").target,
            "a verified line carries nothing"
        );
        // A pattern names a path it matches, and a path the tree no longer
        // holds is still one it matches.
        let tools = [headwater_meta::Pattern::new("tools/*.sh")];
        let by_pattern = Stated::of(&change, "docs/c.md", &["tools/*.sh"], &tools);
        assert!(by_pattern.target, "tools/y.sh is under tools/*.sh");
        assert!(by_pattern.edge, "the edge line names tools/x.sh");
        let elsewhere = [headwater_meta::Pattern::new("src/*.rs")];
        let unnamed = Stated::of(&change, "docs/c.md", &["src/*.rs"], &elsewhere);
        assert!(!unnamed.target && !unnamed.edge, "{unnamed:?}");
        assert_eq!(change.unmatched(), vec!["docs/typo.md", "tools/y.sh"]);
        let named = change.named();
        assert_eq!(named.verified, 2, "{named:?}");
        assert_eq!(
            named.added + named.carried + named.unreadable + named.unmatched + named.verified_alone,
            named.documents,
            "{named:?}"
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

    /// Line endings, trailing spaces and tabs and blank lines normalize away,
    /// and indentation and every other byte are content. It reads bytes, so a
    /// file that is not UTF-8 normalizes too (#2100).
    #[test]
    fn white_space_normalizes_away_and_indentation_does_not() {
        let base = normalized(b"fn a() {\n    b();\n}\n");
        assert_eq!(base, b"fn a() {\n    b();\n}".to_vec());
        assert_eq!(normalized(b"fn a() {  \r\n    b();\t\r\n\r\n\n}\r\n"), base);
        assert_eq!(normalized(b"\n\nfn a() {\n    b();\n}"), base);
        assert_ne!(normalized(b"fn a() {\n  b();\n}\n"), base);
        assert_ne!(normalized(b"fn a() {\n    c();\n}\n"), base);
        assert_eq!(normalized(b"\xff\xfe \r\n"), b"\xff\xfe".to_vec());
    }

    /// Bytes that are not UTF-8 are no document, and they keep both digests
    /// all the same, because the digests are taken before the decode (#2100).
    #[test]
    fn a_prior_version_that_is_not_utf8_keeps_its_digests() {
        let bytes = b"\xff\xfe echo  \r\n".to_vec();
        let given = bytes.clone();
        let change = Unbound::read(
            &format!("{FORMAT}\nprior\ttools/a.bin\tprior/a.bin\n"),
            move |_| Ok(given.clone()),
        )
        .expect("a change")
        .bind(|_| false);
        let touched: Vec<(&str, Touched<'_>)> = change.touched().collect();
        assert_eq!(
            touched,
            vec![("tools/a.bin", Touched::Prior(Some(&Digests::of(&bytes))))]
        );
    }

    /// A prior version that opened keeps both digests whether or not it
    /// parsed, a source file bound to no row included, and an `added` line
    /// and a prior that did not open carry none (#2100).
    #[test]
    fn a_source_file_keeps_its_prior_digests_through_the_bind() {
        let manifest = format!(
            "{FORMAT}\nprior\ttools/a.sh\tprior/a.sh\nadded\ttools/b.sh\n\
             prior\ttools/c.sh\tprior/missing.sh\nprior\tdocs/a.md\tprior/a.md\n"
        );
        let change = Unbound::read(
            &manifest,
            tree(&[("prior/a.sh", "echo a  \r\n"), ("prior/a.md", ASSERTED)]),
        )
        .expect("a change")
        .bind(|path| path == "docs/a.md");
        let touched: Vec<(&str, Touched<'_>)> = change.touched().collect();
        let expected = Digests::of(b"echo a  \r\n");
        assert_eq!(expected.raw, headwater_hash::digest(b"echo a  \r\n"));
        assert_eq!(expected.normalized, headwater_hash::digest(b"echo a"));
        let document = Digests::of(ASSERTED.as_bytes());
        assert_eq!(
            touched,
            vec![
                ("docs/a.md", Touched::Prior(Some(&document))),
                ("tools/a.sh", Touched::Prior(Some(&expected))),
                ("tools/b.sh", Touched::Added),
                ("tools/c.sh", Touched::Prior(None)),
            ]
        );
    }
}
