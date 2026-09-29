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

use headwater_meta::pattern::Pattern;
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

    /// Where `path` falls in this corpus, decided by name alone.
    ///
    /// [#319](https://github.com/headwater-ai/headwater/issues/319): the walk
    /// above answers this question for every file it finds, one entry at a
    /// time, and only for a file that is there to find. A write-time hook asks
    /// the same question of a path that does not exist yet — that is the one
    /// case the walk never reaches — and until this method existed, nothing in
    /// the engine could answer, so a hook read `.headwater/corpus.json` and
    /// matched the exclusion patterns itself, in a second language. This is
    /// [`Exclusion::pattern`] deciding both callers now.
    ///
    /// `path` may be absolute (on disk, under [`Corpus::base`]) or relative to
    /// the repository root; either form normalizes to the same answer.
    pub fn classify(&self, path: &Path) -> Classification {
        let Some(relative) = self.locate(path) else {
            return Classification::Unclassifiable;
        };
        let under_root = relative == self.root || relative.starts_with(&format!("{}/", self.root));
        if !under_root {
            return Classification::Outside;
        }
        match self.exclusion_for(&relative) {
            Some(exclusion) => Classification::Excluded(exclusion.pattern.source().to_string()),
            None => Classification::Corpus,
        }
    }

    /// `path`, relative to [`Corpus::base`] and written with `/` — or `None`
    /// where it names somewhere this repository has no way to express as one:
    /// outside `base` on disk, or a relative path whose `..` climbs above it.
    ///
    /// This never touches the filesystem. It reasons about the path as
    /// written, which is what lets it answer for a path with no file behind
    /// it.
    fn locate(&self, path: &Path) -> Option<String> {
        relative(&self.base, path)
    }
}

/// `path`, relative to the repository root `base` and written with `/`, or
/// `None` where it leaves the repository: an absolute path outside `base`, a
/// relative path whose `..` climbs above it, or a segment that is not UTF-8.
///
/// This is the one reading of a typed path against a repository.
/// [`Corpus::classify`] reads it, and so does every verb that finds a document
/// by a path a shell or an editor spelled: `./x`, `a/../x` and `<base>/x` all
/// come back as `x` ([#1227](https://github.com/headwater-ai/headwater/issues/1227)).
/// A relative path is read against `base`, never against the working
/// directory of the process. An absolute path is compared with `base` as
/// written, so a caller that holds a relative `base` makes it absolute first.
///
/// It never touches the filesystem, so it answers for a path with no file
/// behind it, and a `..` is lexical: `link/..` is the directory `link` sits
/// in, whatever `link` points at.
pub fn relative(base: &Path, path: &Path) -> Option<String> {
    let path = match path.is_absolute() {
        true => path.strip_prefix(base).ok()?,
        false => path,
    };
    let mut segments: Vec<&str> = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(segment) => segments.push(segment.to_str()?),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                segments.pop()?;
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => return None,
        }
    }
    Some(segments.join("/"))
}

/// A path as a reader typed it, relative to the repository root `root` and
/// written with `/`, or `None` where it leaves the repository.
///
/// [`relative`] does the reading, and this adds the root an absolute target is
/// compared with. A root such as `.` is relative, so an absolute target is
/// stripped against the root made absolute first. Where that misses, both
/// sides are made canonical and compared again, so a target typed through a
/// symlinked directory (a home directory, a temporary directory) still finds a
/// root reached the other way. That second read needs a file on disk, and a
/// target with none keeps the first answer. A relative target is read against
/// `root` and never against the working directory of the process.
pub fn typed(root: &Path, target: &str) -> Option<String> {
    let path = Path::new(target);
    if !path.is_absolute() {
        return relative(root, path);
    }
    let absolute_root = std::path::absolute(root).unwrap_or_else(|_| root.to_path_buf());
    relative(&absolute_root, path).or_else(|| {
        let canonical_root = root.canonicalize().ok()?;
        relative(&canonical_root, &path.canonicalize().ok()?)
    })
}

/// [`typed`], and `None` as well where the path passes through a symlink that
/// leads out of the repository
/// ([#1249](https://github.com/headwater-ai/headwater/issues/1249)).
///
/// [`typed`] is lexical, so `link/x.md` stays in the repository whatever
/// `link` points at. This is the reading a refusal takes: a reader who asks
/// about `link/x.md` is asking about a file somewhere else on the host, and
/// "outside this repository" is the true answer. A lookup of a census row
/// reads [`typed`] and not this, so a document row that is itself a symlink
/// still answers as the row the walk recorded.
///
/// The corpus root is the one symlink the walk follows: [`walk`] reads the
/// directory `corpus_root` names, wherever it leads. So a path under
/// `corpus_root` stays inside when it lands under where the corpus root leads,
/// and a new document under a linked corpus root is a path of the corpus, as
/// it will be once the walk reads it. A symlink below the corpus root is not
/// followed, and one that leads out still makes the path outside.
///
/// It reads the filesystem: the longest leading part of the path that exists
/// is made canonical and compared with the canonical root. A path with no part
/// on disk below the root, and a root that cannot be made canonical, keep the
/// lexical answer.
pub fn within(root: &Path, corpus_root: &str, target: &str) -> Option<String> {
    let relative = typed(root, target)?;
    match escapes(root, corpus_root, &relative) {
        true => None,
        false => Some(relative),
    }
}

/// Whether the longest leading part of `relative` that exists under `root`
/// resolves, through a symlink, to a place that is neither under `root` nor,
/// for a path under `corpus_root`, under where the corpus root leads.
/// `relative` is what [`typed`] returned, so it holds no `..`.
fn escapes(root: &Path, corpus_root: &str, relative: &str) -> bool {
    let Ok(canonical_root) = root.canonicalize() else {
        return false;
    };
    let linked_corpus = match Path::new(relative).starts_with(corpus_root) {
        true => root.join(corpus_root).canonicalize().ok(),
        false => None,
    };
    let mut at = Path::new(relative);
    while !at.as_os_str().is_empty() {
        if let Ok(canonical) = root.join(at).canonicalize() {
            let inside = canonical.starts_with(&canonical_root)
                || linked_corpus
                    .as_ref()
                    .is_some_and(|corpus| canonical.starts_with(corpus));
            return !inside;
        }
        at = at.parent().unwrap_or(Path::new(""));
    }
    false
}

/// Where a path falls in a corpus, decided by name alone — the closed set
/// [#319](https://github.com/headwater-ai/headwater/issues/319) asks for: a
/// path a caller has not written yet still lands in exactly one of these.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Classification {
    /// Under the corpus root, and no exclusion claims it.
    Corpus,
    /// Under the corpus root, and this pattern claims it.
    Excluded(String),
    /// Not under this corpus root at all.
    Outside,
    /// Not a path this repository can classify: outside the repository root
    /// on disk, or a relative path that climbs above it.
    Unclassifiable,
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
    /// A regular file.
    File,
    /// A named pipe, a socket or a device: an entry that is not a regular
    /// file, a directory or a symlink. Nothing opens one. Opening a named pipe
    /// that has no writer blocks the process forever, so a census that read a
    /// pipe at a document path never ended (#1333).
    Special,
    /// A symlink, and where it pointed, as written.
    Symlink { target: String },
    /// A directory the walk could not read, so the files under it are missing
    /// from the count and this entry is the only record of that.
    UnreadableDirectory { error: String },
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
                kind: match metadata.is_file() {
                    true => EntryKind::File,
                    false => EntryKind::Special,
                },
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
        sorted.sort_unstable();
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

    /// A named pipe is an entry of its own kind and not a file, so no reader
    /// of the walk opens it by mistake (#1333). Opening one that has no
    /// writer blocks forever.
    #[cfg(unix)]
    #[test]
    fn a_named_pipe_is_a_special_entry_and_not_a_file() {
        let at = std::env::temp_dir().join(format!(
            "headwater-walk-pipe-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("a clock later than the epoch")
                .subsec_nanos()
        ));
        std::fs::create_dir_all(at.join("docs")).expect("the root is made");
        std::fs::write(at.join("docs/a.md"), "# A\n").expect("the file writes");
        let fifo = std::process::Command::new("mkfifo")
            .arg(at.join("docs/x.md"))
            .status()
            .expect("mkfifo runs");
        assert!(fifo.success(), "the named pipe is made");

        let entries = walk(&Corpus::new(at.clone(), "docs"));
        std::fs::remove_dir_all(&at).ok();
        let kind = |path: &str| {
            entries
                .iter()
                .find(|entry| entry.path == path)
                .map(|entry| entry.kind.clone())
        };
        assert!(matches!(kind("docs/x.md"), Some(EntryKind::Special)));
        assert!(matches!(kind("docs/a.md"), Some(EntryKind::File)));
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

    /// [#319](https://github.com/headwater-ai/headwater/issues/319): the four
    /// states a path falls into, none of which needs the path to exist. This
    /// is the test the issue's first Done-when bar asks for.
    #[test]
    fn classify_answers_for_a_path_with_no_file_behind_it() {
        let corpus = fixture_corpus().excluding(vec![Exclusion::new(
            "walk/excluded/**",
            "a declared exclusion, so that a test has one",
        )]);

        assert_eq!(
            corpus.classify(Path::new("walk/never-written.md")),
            Classification::Corpus
        );
        assert_eq!(
            corpus.classify(Path::new("walk/excluded/never-written.md")),
            Classification::Excluded("walk/excluded/**".to_string())
        );
        assert_eq!(
            corpus.classify(Path::new("engine/never-written.rs")),
            Classification::Outside
        );
        assert_eq!(
            corpus.classify(Path::new("/etc/passwd")),
            Classification::Unclassifiable
        );
        assert_eq!(
            corpus.classify(Path::new("../../etc/passwd")),
            Classification::Unclassifiable
        );

        // The absolute form of the same path on disk answers exactly as the
        // relative form does — a caller with either shape gets one verdict.
        let absolute = corpus.base.join("walk/never-written.md");
        assert_eq!(corpus.classify(&absolute), Classification::Corpus);
    }

    /// #1249: `within` is `typed`, less every path that passes through a
    /// symlink out of the root, other than the corpus root the walk follows. A
    /// link that stays inside, a path with nothing on disk, a path under a
    /// linked corpus root and the root itself keep the lexical answer. `typed` still
    /// reads the escaping path lexically, because a lookup of a census row
    /// reads that and not this.
    #[cfg(unix)]
    #[test]
    fn within_refuses_a_path_through_a_symlink_that_leads_out_of_the_root() {
        let base =
            std::env::temp_dir().join(format!("headwater-census-within-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let root = base.join("root");
        let elsewhere = base.join("elsewhere");
        std::fs::create_dir_all(root.join("docs")).expect("the root is made");
        std::fs::create_dir_all(&elsewhere).expect("the outside is made");
        std::fs::write(elsewhere.join("x.md"), b"outside\n").expect("it writes");
        std::os::unix::fs::symlink(&elsewhere, root.join("escape")).expect("a link out");
        std::os::unix::fs::symlink(root.join("docs"), root.join("inside")).expect("a link in");
        // A corpus root that is itself a link out, as a scratch root shares a
        // corpus with a real tree: the walk follows it, so a path under it is
        // inside, and a link below it that leads out again is not.
        std::fs::create_dir_all(elsewhere.join("corpus/shelf")).expect("the linked corpus");
        std::os::unix::fs::symlink(elsewhere.join("corpus"), root.join("linked"))
            .expect("a linked corpus root");
        std::os::unix::fs::symlink(&elsewhere, elsewhere.join("corpus/out"))
            .expect("a link out below the corpus root");

        let cases: [(&str, Option<&str>); 12] = [
            ("linked/shelf/new.md", Some("linked/shelf/new.md")),
            ("./linked/new.md", Some("linked/new.md")),
            ("linked/out/x.md", None),
            ("escape/corpus/shelf/new.md", None),
            ("escape/x.md", None),
            ("./escape/x.md", None),
            ("escape/never-written.md", None),
            ("escape", None),
            ("inside/x.md", Some("inside/x.md")),
            ("docs/never-written.md", Some("docs/never-written.md")),
            ("./", Some("")),
            ("../elsewhere/x.md", None),
        ];
        let answers: Vec<(&str, Option<String>)> = cases
            .iter()
            .map(|(target, _)| (*target, within(&root, "linked", target)))
            .collect();
        // The same root reached through a symlink, as `/tmp` is on macOS or a
        // linked home directory is: `--root` is never made canonical before
        // it arrives, so every comparison has to be, and each case answers
        // what it answers over the root itself.
        let via = base.join("via");
        std::os::unix::fs::symlink(&root, &via).expect("a root reached by a link");
        let answers_via: Vec<Option<String>> = cases
            .iter()
            .map(|(target, _)| within(&via, "linked", target))
            .collect();
        let lexical = typed(&root, "escape/x.md");
        let _ = std::fs::remove_dir_all(&base);
        for ((target, expected), (_, answer)) in cases.iter().zip(&answers) {
            assert_eq!(answer.as_deref(), *expected, "`within` on `{target}`");
        }
        for ((target, expected), answer) in cases.iter().zip(&answers_via) {
            assert_eq!(
                answer.as_deref(),
                *expected,
                "`within` on `{target}` under a root reached by a link"
            );
        }
        assert_eq!(
            lexical.as_deref(),
            Some("escape/x.md"),
            "`typed` stays lexical"
        );
    }
}
