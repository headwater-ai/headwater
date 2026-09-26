// SPDX-License-Identifier: Apache-2.0
//! The paths outside the corpus root that a language regime lists.
//!
//! [HW-DR-0084](../../../../docs/decisions/0084-a-language-regime-reaches-front-door-prose-outside-the-corpus-root-and-no-other-rule-does.md)
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
use headwater_doc::{Document, Mapping, Reason, Span};
use headwater_meta::pattern::Pattern;

/// What a set of regimes lists outside the corpus root, as this walk found it.
#[derive(Clone, Debug, Default)]
pub struct Outside {
    /// One entry per path, in path order.
    pub rows: Vec<OutsideRow>,
    /// Every pattern that matched no path, in the order the regimes list them.
    pub unmatched: Vec<Unmatched>,
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

/// A pattern that a regime lists and that matched no path.
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
        if self.rows.is_empty() && self.unmatched.is_empty() {
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
        out
    }
}

/// Resolve every listed pattern against the repository and read what matched.
///
/// A path under the corpus root is not read here: the census already holds it
/// as a row, and a second reading would give one file two answers. The
/// refusal of such a pattern is `taxonomy validate`'s, which has the consumer
/// declaration in hand; here the path is dropped and the pattern, if nothing
/// else matched, is reported as unmatched with that reason.
pub fn take(corpus: &Corpus, listed: &[Listed]) -> Outside {
    let mut outside = Outside::default();
    for regime in listed {
        for source in &regime.patterns {
            let matched = match resolve(corpus, source) {
                Ok(matched) => matched,
                Err(reason) => {
                    outside.unmatched.push(Unmatched {
                        pattern: source.clone(),
                        regime: regime.regime.clone(),
                        reason,
                    });
                    continue;
                }
            };
            for path in matched {
                // Two regimes that list one path are refused at resolution.
                // The first reading stands here, so a lock that slipped past
                // the refusal still gives each path one regime.
                if outside.holds(&path) {
                    continue;
                }
                outside.rows.push(read(corpus, path, &regime.regime));
            }
        }
    }
    outside.rows.sort_by(|a, b| a.path.cmp(&b.path));
    outside
}

/// The paths one pattern matches outside the corpus root, or why none.
fn resolve(corpus: &Corpus, source: &str) -> Result<Vec<String>, String> {
    if let Some(why) = leaves_the_repository(source) {
        return Err(why);
    }
    let pattern = Pattern::new(source);
    let inside = |path: &str| {
        path == corpus.root || path.starts_with(&format!("{}/", corpus.root.trim_end_matches('/')))
    };
    let matched: Vec<String> = if pattern.is_literal() {
        let path = pattern.literal_prefix();
        match corpus.base.join(&path).is_file() {
            true => vec![path],
            false => return Err(format!("no file `{path}` in the repository")),
        }
    } else {
        let prefix = pattern.literal_prefix();
        if prefix.is_empty() {
            return Err(
                "the pattern opens with a wildcard, and no literal segment bounds the search; \
                 write a literal directory before the first `*` or `**`"
                    .to_string(),
            );
        }
        let scoped = Corpus::new(corpus.base.clone(), &prefix);
        let mut found: Vec<String> = walk::walk(&scoped)
            .into_iter()
            .filter(|entry| matches!(entry.kind, EntryKind::File))
            .map(|entry| entry.path)
            .filter(|path| pattern.matches(path))
            .collect();
        found.sort();
        found.dedup();
        if found.is_empty() {
            return Err("no file in the repository matches the pattern".to_string());
        }
        found
    };
    let outside: Vec<String> = matched.into_iter().filter(|path| !inside(path)).collect();
    if outside.is_empty() {
        return Err(format!(
            "every path the pattern matches is under the corpus root `{}`, where a kind binds the regime",
            corpus.root
        ));
    }
    Ok(outside)
}

/// Why a pattern leaves the repository, and nothing when it stays inside it.
///
/// HW-DR-0084 clause 2 refuses both shapes. `taxonomy resolve` refuses them
/// first; this is the reading that keeps a lock written before the refusal from
/// opening a file outside the tree.
pub fn leaves_the_repository(source: &str) -> Option<String> {
    if source.starts_with('/') || source.starts_with('\\') || source.contains(':') {
        return Some("the pattern is absolute, and a path outside the root is still inside the repository".to_string());
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
    let document = match headwater_doc::parse(&source) {
        Ok(document) => document,
        Err(errors) if errors.iter().any(|e| e.reason == Reason::NoFrontMatter) => Document {
            facets: Mapping::default(),
            block: Span::default(),
            body: headwater_doc::body::scan(&source, &source, 0),
        },
        Err(errors) => {
            let why = errors
                .iter()
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            return unread(format!("the front matter did not load: {why}"), Some(digest));
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
