// SPDX-License-Identifier: Apache-2.0
//! The paths outside the corpus root that a language regime lists.
//!
//! [HW-DR-0084](../../../../docs/decisions/0084-a-language-rule-reaches-front-door-prose-outside-the-corpus-root-and-no-other-rule-does.md)
//! lets a language regime name paths outside the corpus root under
//! `outside_root`, so that the prose a newcomer reads first is held to the
//! controlled language the rest of the corpus is held to. Three rules read
//! such a path and no other rule does, so the path is **not a census row**.
//! A row is a member of the denominator every coverage guarantee is computed
//! over, and a file that no kind binds would count against OB-COV-1 as a
//! document nothing checked.
//!
//! So this module keeps its own list beside [`crate::census::Census::rows`],
//! and the census report prints it on its own line. The patterns are the
//! language of [`headwater_meta::pattern`], matched against paths relative to
//! the repository root, and they resolve on the terms HW-DR-0074 states for a
//! code-path anchor: a pattern with no wildcard is a filesystem `exists`, and a
//! pattern with one walks the subtree its literal prefix fixes. A pattern that
//! matches nothing is kept and reported by name rather than skipped.

use crate::walk::{self, Corpus, EntryKind};
use headwater_doc::Document;
use headwater_meta::pattern::Pattern;

/// What a set of regimes lists outside the corpus root, as this walk found it.
#[derive(Clone, Debug, Default)]
pub struct Outside {
    /// One entry per path, in path order.
    pub rows: Vec<OutsideRow>,
    /// Every pattern that matched no path, in the order the regimes list them.
    pub unmatched: Vec<Unmatched>,
    /// Every pattern HW-DR-0084 clause 2 refuses, in the same order. A refused
    /// pattern reads no path.
    pub refused: Vec<Unmatched>,
}

/// One path outside the corpus root, and the regime that lists it.
#[derive(Clone, Debug)]
pub struct OutsideRow {
    /// Relative to the repository root, with `/` separators.
    pub path: String,
    /// The name of the language regime that lists this path.
    pub regime: String,
    /// The digest of the bytes read, and `None` where none were read.
    pub digest: Option<String>,
    /// The document, and `None` where the file did not read. A file with no
    /// front matter is a document with an empty mapping, because front-door
    /// prose has no facets and owes none.
    pub document: Option<Box<Document>>,
    /// Why the file did not read, where it did not.
    pub unread: Option<String>,
}

/// A pattern that a regime lists, and why it reads nothing: it matched no path,
/// or HW-DR-0084 clause 2 refuses it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unmatched {
    pub pattern: String,
    pub regime: String,
    /// Why nothing matched, in words a reader can act on.
    pub reason: String,
}

/// The paths one regime lists: its name and its patterns as declared.
#[derive(Clone, Debug)]
pub struct Listed {
    pub regime: String,
    pub patterns: Vec<String>,
}

impl Outside {
    /// The number of paths each regime binds, by regime name, in name order.
    pub fn by_regime(&self) -> Vec<(String, usize)> {
        let mut counts: Vec<(String, usize)> = Vec::new();
        for row in &self.rows {
            match counts.iter_mut().find(|(name, _)| *name == row.regime) {
                Some((_, count)) => *count += 1,
                None => counts.push((row.regime.clone(), 1)),
            }
        }
        counts.sort();
        counts
    }

    /// Whether this path is one the list holds.
    pub fn holds(&self, path: &str) -> bool {
        self.rows.iter().any(|row| row.path == path)
    }

    /// The report section, and the empty string where no regime lists a path.
    ///
    /// A corpus that declares no `outside_root` prints nothing, so its report
    /// is the one it printed before this list existed.
    pub fn render(&self) -> String {
        if self.rows.is_empty() && self.unmatched.is_empty() && self.refused.is_empty() {
            return String::new();
        }
        let counts = self
            .by_regime()
            .into_iter()
            .map(|(regime, count)| format!(", {regime} {count}"))
            .collect::<String>();
        let noun = match self.rows.len() {
            1 => "path",
            _ => "paths",
        };
        let mut out = format!(
            "outside the corpus root: {} {noun}{counts}\n",
            self.rows.len()
        );
        for row in &self.rows {
            if let Some(why) = &row.unread {
                out.push_str(&format!("  unread `{}`: {why}\n", row.path));
            }
        }
        for unmatched in &self.unmatched {
            out.push_str(&format!(
                "  unmatched `{}` in `{}`: {}\n",
                unmatched.pattern, unmatched.regime, unmatched.reason
            ));
        }
        for refused in &self.refused {
            out.push_str(&format!(
                "  refused `{}` in `{}`: {}\n",
                refused.pattern, refused.regime, refused.reason
            ));
        }
        out
    }
}

/// Resolve every listed pattern against the repository and read what matched.
///
/// HW-DR-0084 clause 2 refuses three entries, and a refused pattern reads no
/// path at all: a pattern that leaves the repository, a pattern that matches a
/// path under the corpus root, and a path that a second regime lists. A path
/// under the root is placed, and placement is the one way it gets a kind, so a
/// second reading here would give one file two answers. A symlink is refused
/// too, in either direction. `headwater check` reports each refusal and each
/// unmatched pattern as an error of `language.outside_root.refused`, and
/// `taxonomy validate` fails on both.
pub fn take(corpus: &Corpus, listed: &[Listed]) -> Outside {
    let mut outside = Outside::default();
    for regime in listed {
        for source in &regime.patterns {
            let note = |reason: String| Unmatched {
                pattern: source.clone(),
                regime: regime.regime.clone(),
                reason,
            };
            let matched = match resolve(corpus, source) {
                Resolved::Paths(matched) => matched,
                Resolved::Nothing(reason) => {
                    outside.unmatched.push(note(reason));
                    continue;
                }
                Resolved::Refused(reason) => {
                    outside.refused.push(note(reason));
                    continue;
                }
            };
            for path in matched {
                match outside.rows.iter().find(|row| row.path == path) {
                    // One regime that reaches one path through two of its
                    // patterns reads it once.
                    Some(row) if row.regime == regime.regime => {}
                    Some(row) => {
                        let reason = format!(
                            "`{path}` is listed by `{}` as well, and a path answers to one regime",
                            row.regime
                        );
                        outside.refused.push(note(reason));
                    }
                    None => outside.rows.push(read(corpus, path, &regime.regime)),
                }
            }
        }
    }
    outside.rows.sort_by(|a, b| a.path.cmp(&b.path));
    outside
}

/// What one pattern came to.
enum Resolved {
    /// The paths it matches, every one of them outside the corpus root.
    Paths(Vec<String>),
    /// It matched no file, and why.
    Nothing(String),
    /// HW-DR-0084 clause 2 refuses it, and why.
    Refused(String),
}

/// The paths one pattern matches outside the corpus root, or why none.
///
/// The pattern is normalized first: an empty segment and a `.` segment are
/// dropped, so `./docs/x.md` and `docs//x.md` meet the inside-root test as
/// `docs/x.md` does.
///
/// **A symlink is never followed**, which is the rule [`crate::walk`] states
/// for the census. A listed path that is a link, or that passes through a
/// linked directory, is refused and reads nothing, and so is a wildcard match
/// that is a link. A link out of the repository would read, and under
/// `check --fix` write, a file this repository does not hold. A link into the
/// corpus root would give one document two readings.
fn resolve(corpus: &Corpus, source: &str) -> Resolved {
    if let Some(why) = leaves_the_repository(source) {
        return Resolved::Refused(why);
    }
    let normalized = normalize(source);
    let pattern = Pattern::new(&normalized);
    let root = normalize(&corpus.root);
    let inside =
        |path: &str| root.is_empty() || path == root || path.starts_with(&format!("{root}/"));
    let matched: Vec<String> = if pattern.is_literal() {
        let path = pattern.literal_prefix();
        if let Some(link) = linked(&corpus.base, &path) {
            return Resolved::Refused(format!(
                "`{link}` is a symlink, and a path this regime lists is never read through one"
            ));
        }
        match std::fs::symlink_metadata(corpus.base.join(&path)) {
            Ok(meta) if meta.is_file() => vec![path],
            _ => return Resolved::Nothing(format!("no file `{path}` in the repository")),
        }
    } else {
        let prefix = pattern.literal_prefix();
        if prefix.is_empty() {
            return Resolved::Refused(
                "the pattern opens with a wildcard, and no literal segment bounds the search; \
                 write a literal directory before the first `*` or `**`"
                    .to_string(),
            );
        }
        if let Some(link) = linked(&corpus.base, &prefix) {
            return Resolved::Refused(format!(
                "`{link}` is a symlink, and a path this regime lists is never read through one"
            ));
        }
        let scoped = Corpus::new(corpus.base.clone(), &prefix);
        let entries: Vec<walk::Entry> = walk::walk(&scoped)
            .into_iter()
            .filter(|entry| pattern.matches(&entry.path))
            .collect();
        if let Some(link) = entries
            .iter()
            .find(|entry| matches!(entry.kind, EntryKind::Symlink { .. }))
        {
            return Resolved::Refused(format!(
                "the pattern matches `{}`, which is a symlink, and a path this regime lists is \
                 never read through one",
                link.path
            ));
        }
        let mut found: Vec<String> = entries
            .into_iter()
            .filter(|entry| matches!(entry.kind, EntryKind::File))
            .map(|entry| entry.path)
            .collect();
        found.sort();
        found.dedup();
        if found.is_empty() {
            return Resolved::Nothing("no file in the repository matches the pattern".to_string());
        }
        found
    };
    if let Some(path) = matched.iter().find(|path| inside(path)) {
        return Resolved::Refused(format!(
            "the pattern matches `{path}`, which is under the corpus root `{root}`; a path there \
             gets its regime through the kind its placement gives it"
        ));
    }
    Resolved::Paths(matched)
}

/// A path with its empty and `.` segments dropped, joined with `/`.
pub fn normalize(path: &str) -> String {
    path.split(['/', '\\'])
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .collect::<Vec<_>>()
        .join("/")
}

/// The first leading part of a relative path that is a symlink on disk, and
/// nothing when no part is one. A part that does not exist ends the test,
/// because nothing past it can be read.
fn linked(base: &std::path::Path, path: &str) -> Option<String> {
    let mut at = String::new();
    for segment in path.split('/') {
        if !at.is_empty() {
            at.push('/');
        }
        at.push_str(segment);
        match std::fs::symlink_metadata(base.join(&at)) {
            Ok(meta) if meta.file_type().is_symlink() => return Some(at),
            Ok(_) => {}
            Err(_) => return None,
        }
    }
    None
}

/// Why a pattern leaves the repository, and nothing when it stays inside it.
///
/// HW-DR-0084 clause 2 refuses both shapes: an absolute path, and a `..`
/// segment. The test runs before any file is opened, so a refused pattern never
/// reads a file outside the tree.
pub fn leaves_the_repository(source: &str) -> Option<String> {
    if source.starts_with('/') || source.starts_with('\\') || source.contains(':') {
        return Some(
            "the pattern is absolute, and a path outside the corpus root still has to be inside \
             the repository"
                .to_string(),
        );
    }
    if source.split(['/', '\\']).any(|segment| segment == "..") {
        return Some("the pattern climbs out of the repository with `..`".to_string());
    }
    None
}

fn read(corpus: &Corpus, path: String, regime: &str) -> OutsideRow {
    let unread = |why: String, digest: Option<String>| OutsideRow {
        path: path.clone(),
        regime: regime.to_string(),
        digest,
        document: None,
        unread: Some(why),
    };
    let bytes = match std::fs::read(corpus.base.join(&path)) {
        Ok(bytes) => bytes,
        Err(error) => return unread(error.to_string(), None),
    };
    let digest = headwater_hash::digest(&bytes);
    let Ok(source) = String::from_utf8(bytes) else {
        return unread("the bytes are not UTF-8".to_string(), Some(digest));
    };
    let document = match headwater_doc::parse_prose(&source) {
        Ok(document) => document,
        Err(errors) => {
            let why = errors
                .iter()
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            return unread(
                format!("the front matter did not load: {why}"),
                Some(digest),
            );
        }
    };
    OutsideRow {
        path,
        regime: regime.to_string(),
        digest: Some(digest),
        document: Some(Box::new(document)),
        unread: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pattern_that_leaves_the_repository_is_named() {
        assert!(leaves_the_repository("/etc/motd").is_some());
        assert!(leaves_the_repository("../README.md").is_some());
        assert!(leaves_the_repository("docs/../../x.md").is_some());
        assert!(leaves_the_repository("C:/x.md").is_some());
        assert!(leaves_the_repository("README.md").is_none());
        assert!(leaves_the_repository(".github/*.md").is_none());
    }
}
