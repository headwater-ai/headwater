// SPDX-License-Identifier: Apache-2.0
//! A check at corpus grain: a prose link's fragment names a heading of the
//! document it points at, whether that is the citing document or another one.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-five-origins-of-a-check)
//! lists prose-link resolution among the Document-origin examples, and
//! [`headwater_graph::links`] left the far half deliberately undone: "whether
//! `#q4--relation-storage` names a heading of the target is a Document check
//! ... a graph build that started checking headings would be the check layer
//! with no scope declaration."
//!
//! # Both halves are here now, and neither of them is a Document check
//!
//! A fragment with no path names a heading of the document that wrote it, and
//! that half did sit at `Document` grain for two editions.
//!
//! A fragment on a path names a heading of **another** document, and no
//! document-grained view carries one.
//! [HW-OBL-0077](../../../../docs/obligations/0077-a-fragment-on-a-path-needs-a-grain-that-no-scope-supplies.md)
//! recorded that gap, and it read as a missing fifth grain. It is not. Handing
//! a document-scoped rule the target documents as a declared input is unsound
//! for a reason the code states rather than argues: `Instance::paths()` is the
//! read set, and [`crate::coverage`] routes an instance to every path in
//! `paths()`. They are one list by construction. So every target document the
//! read set must name — and soundness demands it, or a renamed heading in the
//! target leaves a cached pass standing — is thereby counted as *checked by
//! this rule*, which is the miscount [`crate::coverage`] exists to name.
//!
//! `Corpus` is the grain, and [`crate::link_path`] already argues it word for
//! word for the same reference class: the defect is a relation between two
//! files, a rename touches no byte of the citing document, so a
//! document-scoped key would not move and a cached pass would outlive the
//! rename that made it false. One instance, whose findings each name the
//! citing document and the line the author wrote.
//!
//! # What this widening costs [`crate::coverage`], measured on this corpus
//!
//! A corpus-scoped instance routes to no document, so the 312 document-routed
//! instances this rule made become one. A classified document whose only
//! routed check was this one would become a `coverage.document_unchecked`
//! finding. That cannot happen here: `facet.required.missing` reports one
//! instance against every classified document and skips none of them, so every
//! classified document is already routed by an evaluated document-scoped rule
//! that is not this one.
//!
//! **That is a fact about this corpus and not a property of the engine.** A
//! thin shelf in an adopting corpus, whose kind requires no facet, could lose
//! its only routed check to this move.
//! [HW-OBL-0071](../../../../docs/obligations/0071-a-corpus-scoped-check-makes-the-coverage-rule-unreachable.md)
//! is the record of that shape, and this change makes it one instance larger.
//!
//! # Why the anchors are computed rather than read
//!
//! A Markdown heading has no anchor in the source. The anchor is what a
//! renderer derives from the heading text, so a check that resolves a fragment
//! has to derive it the same way, and two renderers do not agree. The rule
//! below is the one GitHub applies, because that is where this corpus is read,
//! and it is stated in one function so that an adopter reading a wrong verdict
//! finds one place to look.
//!
//! # The one instance says what it read
//!
//! A run whose scope admitted no links, and a run that reached no document
//! body, each **skip with that reason** rather than passing. A pass would
//! report a corpus this rule never read as clean, which is
//! [`crate::link_path`]'s `NO_LINKS` and [`crate::duplicate`]'s `NO_REPORT` at
//! the same grain.
//!
//! The population itself is stated by the graph report, which counts the
//! fragment-bearing links by arm beside the bindings it already prints. It is
//! there rather than here because a passing outcome carries no sentence, and a
//! reader who wants to know how much of the corpus this rule reads should not
//! have to break it to find out.
//!
//! # What is not this rule
//!
//! `Binding::Missing` and `Binding::Unnormalizable` are a path that resolved to
//! nothing, and they belong to [`crate::link_path`]. `Binding::Repository` and
//! `Binding::External` name a body this engine never read, so this rule has
//! nothing to compare a fragment against and says nothing.
//!
//! `Binding::Corpus` on a file the census walked but parsed no document out of
//! — an untyped or an excluded file — is the one case inside this rule's
//! bindings that it passes over. No anchor list exists for such a path, so a
//! finding there would be a guess. See [`Anchors::of`].

use crate::finding::{Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{CorpusCheck, CorpusView};
use headwater_graph::links::{Binding, Link};
use std::collections::{HashMap, HashSet};

pub const RULE: &str = "link.fragment.unresolved";

/// A run whose scope did not admit the links has nothing to read, and it says
/// so rather than passing. [`crate::link_path::NO_LINKS`] verbatim, at the same
/// grain and for the same reason.
const NO_LINKS: &str = "the view carries no bound prose links for this corpus";

/// A run that reached no document body cannot derive an anchor, so it has no
/// second half to compare a fragment against.
const NO_ANCHORS: &str = "the view carries no document headings for this corpus";

/// The check. It reads no declaration, and the module comment says why: the
/// taxonomy language has no member that turns prose-link resolution on or off,
/// because [spec 1](../../../../docs/spec/01-conceptual-model.md#prose-links-are-not-relations)
/// makes it a property of a corpus rather than of a kind.
pub struct Fragments;

impl CorpusCheck for Fragments {
    const RULE: &'static str = self::RULE;
    /// The third edition, and this one widened what the rule reads rather than
    /// changing an answer. A warm cache holding an edition-2 verdict holds a
    /// verdict about same-document fragments alone, and it would serve that
    /// forever over a corpus whose cross-document fragments were never
    /// examined. The second edition renumbered a repeated heading; the first
    /// counted every earlier anchor that began with its slug and a hyphen,
    /// which is not what a renderer does.
    const VERSION: u32 = 3;
    const NEEDS_LINKS: bool = true;
    const NEEDS_ANCHORS: bool = true;

    fn evaluate(&self, view: &CorpusView<'_>) -> Outcome {
        let Some(links) = view.links() else {
            return Outcome::Skipped(NO_LINKS.to_string());
        };
        let Some(anchors) = view.anchors() else {
            return Outcome::Skipped(NO_ANCHORS.to_string());
        };
        // A quoted link belongs to another author, and an image is a reference
        // to an asset. Both were dropped by the graph build, which is the one
        // reader of this corpus's prose links, so this rule inherits that
        // answer rather than writing a second one.
        Outcome::failed(
            links
                .iter()
                .filter_map(|link| finding(link, anchors))
                .collect(),
        )
    }
}

/// One finding per fragment that names no heading of the document it points at,
/// and nothing for every other link.
fn finding(link: &Link, anchors: &Anchors) -> Option<Finding> {
    let fragment = link.fragment.as_deref().filter(|it| !it.is_empty())?;
    // The two arms this rule reads. Every other binding is somebody else's, and
    // the module comment enumerates which.
    let target = match &link.binding {
        Binding::SameDocument => link.source_path.as_str(),
        Binding::Corpus { path, .. } => path.as_str(),
        Binding::Missing { .. }
        | Binding::Unnormalizable { .. }
        | Binding::Repository { .. }
        | Binding::External => return None,
    };
    // A corpus path the census walked and parsed no document out of. There is
    // no heading list, so there is no verdict, and a finding would be a guess.
    if anchors.resolves(target, fragment)? {
        return None;
    }
    let (message, remediation) = match &link.binding {
        Binding::SameDocument => (
            format!("`{}` names no heading of this document", link.destination),
            format!("point it at a heading of {target}, or write the heading it names"),
        ),
        _ => (
            format!("`{}` names no heading of `{target}`", link.destination),
            format!(
                "point it at a heading that `{target}` has, or write the heading it names there"
            ),
        ),
    };
    Some(Finding {
        rule: self::RULE,
        severity: Severity::Error,
        obligation: None,
        // The citing document and the line the author wrote, on
        // [`crate::link_path`]'s terms: the instance is one, and an author
        // still reads the defect where they made it.
        path: link.source_path.clone(),
        line: link.span.start.line,
        column: link.span.start.col,
        message,
        remediation,
        // The heading an author meant is a guess among the headings the target
        // has, and spec 12 admits a fix only where one outcome is derivable
        // without judgment.
        patch: None,
    })
}

/// Every anchor of every document of this corpus, by path.
///
/// # Why this is built beside the read set rather than injected into it
///
/// A corpus-scoped instance's read set is already every census row that carries
/// a document, so the bytes these anchors derive from are named in the key
/// before this index exists. A second key component would hash the same bytes
/// twice. The declaration is still made — [`CorpusCheck::NEEDS_ANCHORS`] —
/// because a rule that reads an input it never declared is the thing the scope
/// declarations exist to stop, and a reader of the trait should be able to see
/// every input this rule takes without reading its body.
pub struct Anchors {
    /// In the census's own order, which is path order, so a lookup is a binary
    /// search rather than a scan of the corpus per link.
    by_path: Vec<(String, Vec<String>)>,
}

impl Anchors {
    /// Derived from the rows that carry a document, which is exactly the read
    /// set of a corpus-scoped instance.
    ///
    /// A row that carries none contributed nothing and could not have: there is
    /// no parsed body, so there is no heading, so a link into that path has no
    /// answer here rather than a false one.
    pub fn of(census: &headwater_census::census::Census) -> Anchors {
        Anchors {
            by_path: census
                .rows
                .iter()
                .filter_map(|row| {
                    let document = row.document.as_ref()?;
                    Some((row.path.clone(), anchors(&document.body)))
                })
                .collect(),
        }
    }

    /// Whether `fragment` names a heading of `path`.
    ///
    /// `None` where this corpus holds no heading list for `path` at all, which
    /// is a path outside the corpus or a file the census parsed no document
    /// out of. That is the absence of a verdict rather than a passing one, and
    /// the caller reports nothing on it.
    /// An index built from a list, for the tests of the rule that reads one.
    /// `cfg(test)` so that no shipped path can build an index whose anchors did
    /// not come from a census.
    #[cfg(test)]
    pub(crate) fn of_pairs(pairs: &[(&str, &[&str])]) -> Anchors {
        let mut by_path: Vec<(String, Vec<String>)> = pairs
            .iter()
            .map(|(path, anchors)| {
                (
                    (*path).to_string(),
                    anchors.iter().map(|it| (*it).to_string()).collect(),
                )
            })
            .collect();
        // The shipped constructor inherits the census's path order, and
        // `resolves` binary searches. A test index that was not sorted would
        // answer `None` for a path it holds.
        by_path.sort();
        Anchors { by_path }
    }

    fn resolves(&self, path: &str, fragment: &str) -> Option<bool> {
        let at = self
            .by_path
            .binary_search_by(|(known, _)| known.as_str().cmp(path))
            .ok()?;
        let wanted = fragment.to_lowercase();
        Some(self.by_path[at].1.contains(&wanted))
    }
}

/// Every anchor a renderer gives this document, in heading order.
///
/// A repeated heading takes a numeric suffix, first occurrence bare. Two
/// `## Consequences` headings therefore give `consequences` and
/// `consequences-1`, and a link to the second resolves.
///
/// # Two rules, and either one alone gets a document of this corpus wrong
///
/// The suffix counts how many anchors this **exact** slug has already been the
/// base of. A longer heading whose slug merely begins with a shorter one is
/// not a repeat of it, so `## Edit sites, as spec 2 stands` standing above two
/// `## Edit sites` leaves them `edit-sites` and `edit-sites-1`.
/// `docs/evaluations/schema-format-walkthrough.md` is that document.
///
/// Then a candidate an earlier heading already took is stepped past, and the
/// counter resumes from where the search stopped. So `## One`, `## One` and
/// `## One-1` give `one`, `one-1` and `one-1-1`, and a fourth `## One` gives
/// `one-2` rather than colliding with the third heading.
///
/// That second half is what makes this function injective: two headings that
/// differ never receive one anchor, because an anchor is issued once. Counting
/// a prefix as a repeat was not injective, and the witness is short —
/// `## One-1` above two `## One` gave the first two headings the anchor
/// `one-1` each.
///
/// Both halves are settled against GitHub's own renderer rather than against a
/// second implementation of the rule, by posting headings to its `/markdown`
/// endpoint and reading the `user-content-` identifiers it returns. The tests
/// below carry the cases that endpoint answered.
fn anchors(body: &headwater_doc::Body) -> Vec<String> {
    let mut anchors: Vec<String> = Vec::new();
    let mut issued: HashSet<String> = HashSet::new();
    let mut repeats: HashMap<String, usize> = HashMap::new();
    for heading in body.headings() {
        let slug = slug(&heading.text());
        let mut n = repeats.get(&slug).copied().unwrap_or(0);
        let anchor = loop {
            let candidate = if n == 0 {
                slug.clone()
            } else {
                format!("{slug}-{n}")
            };
            n += 1;
            if !issued.contains(&candidate) {
                break candidate;
            }
        };
        repeats.insert(slug, n);
        issued.insert(anchor.clone());
        anchors.push(anchor);
    }
    anchors
}

/// The anchor a renderer derives from a heading.
///
/// Lower case; every character that is not a letter, a digit, a space or a
/// hyphen removed; then each space becomes a hyphen. An em dash therefore
/// leaves the two spaces around it, and `Q5 — Voice checking depth` resolves as
/// `q5--voice-checking-depth`, which is what this corpus writes.
fn slug(text: &str) -> String {
    text.trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}

/// Every relative link this engine's own comments write into `docs/`.
///
/// # This is a test and not a rule, and the module above says why
///
/// A fragment on a path names a heading of another document, and no scope this
/// engine has carries one. So the thing checked here cannot be a check-layer
/// rule at all: `corpus.root` is `docs`, a `.rs` file has no front matter and
/// therefore no kind, and the census never reaches one. What this material is
/// closer to is `cargo fmt` — a property of the engine's own source, held by
/// the engine's own suite.
///
/// It lives inside this module rather than beside it because the slugging rule
/// is here. `CLAUDE.md` states that no second copy of a rule lives in a script,
/// and a comment-link checker with its own slugger would be exactly that: two
/// definitions of what a heading anchor is, agreeing today and drifting on the
/// first edit. This test calls [`anchors`] and [`slug`], so a corpus link and a
/// comment link resolve by one rule or by neither.
///
/// # What a broken link costs
///
/// Nothing else reads a `.rs` comment. No check, no gate and no continuous
/// integration step, which is why twenty-nine of these rotted in silence:
/// twenty at the wrong `../` depth, six naming a heading that was retitled, and
/// three naming a document that was.
#[cfg(test)]
mod comment_links {
    use super::{anchors, slug};
    use std::path::{Path, PathBuf};

    /// The engine's own tree, four levels above this crate.
    fn engine_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("the engine root")
    }

    fn repository_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .expect("the repository root")
    }

    /// Where a comment sits in a Rust source file.
    ///
    /// A `//` inside a string literal is not a comment, and a `docs/` path
    /// inside a test fixture is not a link a reader follows. So this walks the
    /// file rather than matching a pattern over it, and it tracks the four
    /// states that can hide a `//`: a line comment, a block comment (which
    /// nests, as Rust allows), a string that may span lines, and a raw string
    /// whose terminator is decided by its own hash count.
    enum State {
        Code,
        Block(usize),
        Str,
        Raw(usize),
    }

    /// The comment text of a Rust file, one output line per input line.
    ///
    /// Every non-comment byte becomes nothing, so a reported line number is the
    /// line of the `.rs` file and needs no second mapping. The comment marker
    /// and the indentation in front of it go too: a doc comment inside an
    /// `impl` block is indented, and a Markdown reader takes four spaces for a
    /// code block and stops parsing the links inside it.
    fn comment_markdown(src: &str) -> String {
        let mut out = String::new();
        let mut state = State::Code;
        for line in src.lines() {
            let mut comment = String::new();
            let bytes: Vec<char> = line.chars().collect();
            let mut i = 0;
            while i < bytes.len() {
                match state {
                    State::Block(depth) => {
                        if bytes[i..].starts_with(&['*', '/']) {
                            state = if depth == 1 {
                                State::Code
                            } else {
                                State::Block(depth - 1)
                            };
                            i += 2;
                        } else if bytes[i..].starts_with(&['/', '*']) {
                            state = State::Block(depth + 1);
                            comment.push_str("/*");
                            i += 2;
                        } else {
                            comment.push(bytes[i]);
                            i += 1;
                        }
                    }
                    State::Str => {
                        if bytes[i] == '\\' {
                            i += 2;
                        } else if bytes[i] == '"' {
                            state = State::Code;
                            i += 1;
                        } else {
                            i += 1;
                        }
                    }
                    State::Raw(hashes) => {
                        if bytes[i] == '"'
                            && bytes[i + 1..]
                                .iter()
                                .take(hashes)
                                .filter(|c| **c == '#')
                                .count()
                                == hashes
                        {
                            state = State::Code;
                            i += 1 + hashes;
                        } else {
                            i += 1;
                        }
                    }
                    State::Code => {
                        if bytes[i..].starts_with(&['/', '/']) {
                            comment.push_str(&bytes[i..].iter().collect::<String>());
                            break;
                        } else if bytes[i..].starts_with(&['/', '*']) {
                            state = State::Block(1);
                            i += 2;
                        } else if bytes[i] == '"' {
                            state = State::Str;
                            i += 1;
                        } else if bytes[i] == 'r'
                            && bytes[i + 1..]
                                .first()
                                .is_some_and(|c| *c == '"' || *c == '#')
                        {
                            let hashes = bytes[i + 1..].iter().take_while(|c| **c == '#').count();
                            if bytes.get(i + 1 + hashes) == Some(&'"') {
                                state = State::Raw(hashes);
                                i += 2 + hashes;
                            } else {
                                i += 1;
                            }
                        } else if bytes[i] == '\'' {
                            // A char literal, or a lifetime. Only the first can
                            // hold a quote, so only the first is consumed.
                            let closes = match bytes.get(i + 1) {
                                Some('\\') => bytes.get(i + 3) == Some(&'\''),
                                Some(_) => bytes.get(i + 2) == Some(&'\''),
                                None => false,
                            };
                            i += if closes {
                                if bytes.get(i + 1) == Some(&'\\') {
                                    4
                                } else {
                                    3
                                }
                            } else {
                                1
                            };
                        } else {
                            i += 1;
                        }
                    }
                }
            }
            let text = comment.trim_start();
            let text = text
                .strip_prefix("///")
                .or_else(|| text.strip_prefix("//!"))
                .or_else(|| text.strip_prefix("//"))
                .unwrap_or(text);
            out.push_str(text.trim_start());
            out.push('\n');
        }
        out
    }

    /// Every `.rs` file of the engine, `target/` excluded.
    fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .expect("the engine tree is readable")
            .filter_map(Result::ok)
            .map(|e| e.path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n == "target") {
                    continue;
                }
                sources(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    /// The anchor set of a Markdown file on disk.
    ///
    /// The front matter is split off first. A `---` fence read as body would
    /// give the line above it a setext heading and an anchor that no renderer
    /// produces, and an anchor set that is too large is an anchor set that
    /// confirms a broken link.
    fn anchors_of(path: &Path) -> Vec<String> {
        let source = std::fs::read_to_string(path).expect("a document this engine cites");
        match headwater_doc::split::split(&source) {
            Ok(split) => anchors(&headwater_doc::body::scan(
                &source,
                split.body,
                split.body_offset,
            )),
            Err(_) => anchors(&headwater_doc::body::scan(&source, &source, 0)),
        }
    }

    /// Every broken link, as `path:line: target`.
    ///
    /// `walk` is the tree of Rust sources to read and `root` is what a reported
    /// path is relative to. They are two parameters rather than one so that the
    /// whole chain below can be pointed at a tree a test wrote. A function that
    /// walked a hardcoded root could only ever run over material that is clean,
    /// and a filter inverted inside it would read nothing while every test here
    /// stayed green.
    fn broken(walk: &Path, root: &Path) -> Vec<String> {
        let mut files = Vec::new();
        sources(walk, &mut files);
        let mut out = Vec::new();
        for file in files {
            let src = std::fs::read_to_string(&file).expect("a source of this engine");
            let markdown = comment_markdown(&src);
            let body = headwater_doc::body::scan(&markdown, &markdown, 0);
            let dir = file.parent().expect("a source has a directory");
            for link in &body.links {
                // The class this holds is the one a reader follows into
                // this repository, and two classes are skipped for reasons
                // that differ. An absolute URL and an absolute path leave the
                // checkout, so nothing here can resolve them. A relative
                // destination naming no `docs/` component is rustdoc's own
                // output tree, which resolves against `target/doc` and not
                // against the source, so a filesystem test of one would report
                // a defect that is not one. The `docs/` test below is what
                // holds that class out.
                //
                // A dotless relative destination such as
                // `docs/spec/02-taxonomy-model.md#x` is neither. It resolves
                // against the source file's own directory exactly as
                // `./docs/...` does, in a browser and here, so it is checked
                // rather than skipped. Requiring the leading `.` skipped it
                // silently, which is the second half of #206.
                if link.image
                    || link.destination.contains("://")
                    || link.destination.starts_with('/')
                {
                    continue;
                }
                let (target, fragment) = match link.destination.split_once('#') {
                    Some((target, fragment)) => (target, Some(fragment)),
                    None => (link.destination.as_str(), None),
                };
                if !target.contains("docs/") {
                    continue;
                }
                let resolved = normalize(&dir.join(target));
                let where_ = format!(
                    "{}:{}",
                    file.strip_prefix(root).unwrap_or(&file).display(),
                    link.span.start.line
                );
                // The path is resolved before the fragment. They are two
                // defects, and a checker that skipped an unresolved file would
                // see neither the depth error nor the retitled document.
                if !resolved.exists() {
                    out.push(format!("{where_}: no such file: {}", link.destination));
                    continue;
                }
                let Some(fragment) = fragment else { continue };
                if resolved.extension().is_some_and(|e| e == "md")
                    && !anchors_of(&resolved).contains(&fragment.to_lowercase())
                {
                    out.push(format!("{where_}: no such heading: {}", link.destination));
                }
            }
        }
        out
    }

    /// `..` resolved textually, because the path a comment writes may not exist
    /// and `canonicalize` refuses one that does not.
    fn normalize(path: &Path) -> PathBuf {
        let mut out = PathBuf::new();
        for part in path.components() {
            match part {
                std::path::Component::ParentDir => {
                    out.pop();
                }
                std::path::Component::CurDir => {}
                other => out.push(other),
            }
        }
        out
    }

    #[test]
    fn every_comment_link_into_docs_resolves() {
        let root = repository_root();
        let broken = broken(&engine_root(), &root);
        assert!(
            broken.is_empty(),
            "{} comment links into docs/ do not resolve:\n{}",
            broken.len(),
            broken.join("\n")
        );
    }

    /// The instrument fires, and it names the file, the line and the target.
    ///
    /// A checker over material that is clean is indistinguishable from a
    /// checker that reads nothing, so the two defect classes are provoked here
    /// rather than trusted. Both inputs are written in this test, so nothing in
    /// the tree has to be broken to hold it.
    #[test]
    fn a_broken_link_in_a_comment_is_named() {
        let depth = comment_markdown(
            "//! see [spec 2](../../../docs/spec/02-taxonomy-model.md) for the rule\n",
        );
        let scanned = headwater_doc::body::scan(&depth, &depth, 0);
        assert_eq!(scanned.links.len(), 1, "the link is found in the comment");
        assert_eq!(
            scanned.links[0].destination,
            "../../../docs/spec/02-taxonomy-model.md"
        );

        let quoted = comment_markdown("let s = \"[a](../../../docs/nope.md)\"; // and\n");
        let scanned = headwater_doc::body::scan(&quoted, &quoted, 0);
        assert!(
            scanned.links.is_empty(),
            "a docs path inside a string literal is not a link a reader follows"
        );

        // The anchor half, against a heading this repository has. The slugger
        // is the rule above, so this also pins the em-dash reading that made
        // twenty-seven of these look broken to a checker that kept the dash.
        assert_eq!(slug("Q4 — Relation storage"), "q4--relation-storage");
        let real = anchors_of(&repository_root().join("docs/spec/09-decisions.md"));
        assert!(real.contains(&"q4--relation-storage".to_string()));
        assert!(!real.contains(&"the-four-scopes".to_string()));
    }

    /// A directory nothing else in this process writes into.
    ///
    /// The process identifier alone is not a key here, because cargo runs the
    /// cases of one target as threads of one process (#189). The test's own
    /// name and the clock carry the rest.
    fn scratch(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("a clock later than the epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "headwater-fragment-{name}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        dir
    }

    fn write(path: &Path, body: &str) {
        std::fs::create_dir_all(path.parent().expect("a file has a directory"))
            .expect("a fixture directory");
        std::fs::write(path, body).expect("a fixture file");
    }

    /// The whole chain, against a tree written here, one link of each class.
    ///
    /// [`every_comment_link_into_docs_resolves`] runs over material that is
    /// clean, so it returns the same empty list whether the chain reads the
    /// tree or reads nothing at all. Invert a filter inside `broken` and that
    /// test stays green. This one gives the chain five links whose classes are
    /// known and asserts exactly which of them come back, so a filter that
    /// stopped holding its class fails here in one direction or the other.
    #[test]
    fn the_filter_chain_names_what_it_holds_and_passes_what_it_does_not() {
        let dir = scratch("filter-chain");
        write(
            &dir.join("docs/spec/02-taxonomy-model.md"),
            "# A model\n\nThe body.\n\n## The heading that is here\n\nMore body.\n",
        );
        // Four levels below the tree root, as this crate's own sources are.
        write(
            &dir.join("engine/crates/check/src/sample.rs"),
            concat!(
                "//! [here](../../../../docs/spec/02-taxonomy-model.md#the-heading-that-is-here)\n",
                "//! [gone](../../../../docs/spec/99-not-a-document.md)\n",
                "//! [retitled](../../../../docs/spec/02-taxonomy-model.md#the-heading-that-is-not)\n",
                "//! [rustdoc](../../headwater_doc/struct.Body.html)\n",
                "//! [remote](https://example.invalid/docs/spec/02-taxonomy-model.md)\n",
            ),
        );
        // The dotless class, both directions, one level below the tree root.
        write(
            &dir.join("engine/docs/spec/02-taxonomy-model.md"),
            "# A model\n\nThe body.\n\n## The heading that is here\n\nMore body.\n",
        );
        write(
            &dir.join("engine/probe.rs"),
            concat!(
                "//! [here](docs/spec/02-taxonomy-model.md#the-heading-that-is-here)\n",
                "//! [retitled](docs/spec/02-taxonomy-model.md#the-heading-that-is-not)\n",
            ),
        );

        let mut found = broken(&dir, &dir);
        found.sort();
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(
            found,
            [
                "engine/crates/check/src/sample.rs:2: no such file: \
                 ../../../../docs/spec/99-not-a-document.md",
                "engine/crates/check/src/sample.rs:3: no such heading: \
                 ../../../../docs/spec/02-taxonomy-model.md#the-heading-that-is-not",
                "engine/probe.rs:2: no such heading: \
                 docs/spec/02-taxonomy-model.md#the-heading-that-is-not",
            ]
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use headwater_doc::body::scan;

    #[test]
    fn an_em_dash_leaves_the_spaces_around_it() {
        assert_eq!(
            slug("Q5 — Voice checking depth"),
            "q5--voice-checking-depth"
        );
        assert_eq!(slug("`$package.optional`"), "packageoptional");
        assert_eq!(
            slug("Two phases, and why the order matters"),
            "two-phases-and-why-the-order-matters"
        );
    }

    /// The anchors of a document that is these headings and nothing else.
    fn anchors_of(source: &str) -> Vec<String> {
        anchors(&scan(source, source, 0))
    }

    #[test]
    fn a_repeated_heading_takes_a_numeric_suffix() {
        let body = scan("# One\n\n# One\n\n# One\n", "# One\n\n# One\n\n# One\n", 0);
        assert_eq!(anchors(&body), ["one", "one-1", "one-2"]);
    }

    /// A longer heading is not a repeat of the shorter one its slug extends.
    ///
    /// Three identical headings cannot see this rule, because counting an
    /// exact repeat and counting a hyphen-prefixed one give the same three
    /// anchors. The case below is the shape that separates them, and
    /// `docs/evaluations/schema-format-walkthrough.md` carries it: the first
    /// heading is a prefix extension of the two that follow, so a rule that
    /// counted it moved every later suffix up by one.
    ///
    /// GitHub's `/markdown` endpoint returns these three identifiers for this
    /// input.
    #[test]
    fn a_longer_heading_is_not_a_repeat_of_the_one_it_extends() {
        assert_eq!(
            anchors_of("## Edit sites, as spec 2 stands\n\n## Edit sites\n\n## Edit sites\n"),
            ["edit-sites-as-spec-2-stands", "edit-sites", "edit-sites-1"]
        );
    }

    /// A candidate an earlier heading took is stepped past, and the count
    /// resumes from where the search stopped.
    ///
    /// This is the half that counting exact repeats alone still gets wrong.
    /// The third heading wants `one-1`, which the second heading holds, so it
    /// takes `one-1-1`. The fourth heading is the second repeat of `one` and
    /// takes `one-2`, which no rule reading only the anchors already issued
    /// arrives at. GitHub's `/markdown` endpoint returns these four.
    #[test]
    fn a_suffix_an_earlier_heading_took_is_stepped_past() {
        assert_eq!(
            anchors_of("## One\n\n## One\n\n## One-1\n\n## One\n"),
            ["one", "one-1", "one-1-1", "one-2"]
        );
    }

    /// Two headings that differ never receive one anchor.
    ///
    /// The review question this answers is whether two states that must differ
    /// can produce one key. Counting a hyphen-prefixed anchor as a repeat
    /// could: on this input it gave the first two headings `one-1` each, so a
    /// link to `#one-1` named two places and the second was unreachable.
    /// GitHub's `/markdown` endpoint returns `one-1`, `one`, `one-2`.
    #[test]
    fn two_headings_that_differ_never_share_an_anchor() {
        let anchors = anchors_of("## One-1\n\n## One\n\n## One\n");
        assert_eq!(anchors, ["one-1", "one", "one-2"]);
        let issued: std::collections::HashSet<&String> = anchors.iter().collect();
        assert_eq!(issued.len(), anchors.len(), "an anchor is issued once");
    }
}

#[cfg(test)]
mod arms {
    use super::*;
    use crate::scope::CorpusView;
    use headwater_doc::LinkForm;
    use headwater_yaml::{Position, Span};

    const CITER: &str = "docs/spec/01-conceptual-model.md";
    const TARGET: &str = "docs/spec/glossary.md";

    fn span() -> Span {
        Span {
            start: Position {
                line: 30,
                col: 5,
                offset: 0,
            },
            end: Position {
                line: 30,
                col: 6,
                offset: 1,
            },
        }
    }

    fn link(destination: &str, fragment: Option<&str>, binding: Binding) -> Link {
        Link {
            source_path: CITER.to_string(),
            destination: destination.to_string(),
            fragment: fragment.map(str::to_string),
            form: LinkForm::Inline,
            span: span(),
            binding,
        }
    }

    fn corpus(path: &str) -> Binding {
        Binding::Corpus {
            path: path.to_string(),
            class: "typed",
            id: None,
        }
    }

    fn index() -> Anchors {
        Anchors::of_pairs(&[
            (CITER, &["a-heading-of-the-citer"]),
            (TARGET, &["projection"]),
        ])
    }

    /// The defect this rule was widened for. The finding is written at the
    /// citing document and line, and it names the target rather than saying
    /// "this document", which would send an author to the wrong file.
    #[test]
    fn a_fragment_that_names_no_heading_of_the_target_is_an_error_at_the_citing_line() {
        let found = finding(
            &link(
                "glossary.md#projections",
                Some("projections"),
                corpus(TARGET),
            ),
            &index(),
        )
        .expect("a finding");
        assert_eq!(found.rule, self::RULE);
        assert_eq!(found.severity, Severity::Error);
        assert_eq!(found.path, CITER);
        assert_eq!(found.line, 30);
        assert_eq!(found.column, 5);
        assert!(found.message.contains(TARGET), "{found:#?}");
        assert!(!found.message.contains("this document"), "{found:#?}");
        assert!(!found.fixable(), "{found:#?}");
    }

    /// The near arm keeps its own sentence, and it names no path.
    #[test]
    fn the_near_arm_says_this_document_and_the_far_arm_names_the_file() {
        let near = finding(
            &link(
                "#no-such-heading",
                Some("no-such-heading"),
                Binding::SameDocument,
            ),
            &index(),
        )
        .expect("a finding");
        assert!(near.message.contains("this document"), "{near:#?}");
        assert!(!near.message.contains(TARGET), "{near:#?}");
        assert_eq!(near.path, CITER);
    }

    /// A fragment that resolves is nothing to this rule, on both arms.
    #[test]
    fn a_fragment_that_resolves_is_not_a_finding_on_either_arm() {
        assert!(finding(
            &link("glossary.md#projection", Some("projection"), corpus(TARGET)),
            &index()
        )
        .is_none());
        assert!(finding(
            &link(
                "#a-heading-of-the-citer",
                Some("a-heading-of-the-citer"),
                Binding::SameDocument
            ),
            &index()
        )
        .is_none());
    }

    /// The comparison is case folded, which is what a renderer does.
    #[test]
    fn a_fragment_resolves_whatever_case_the_author_wrote_it_in() {
        assert!(finding(
            &link("glossary.md#Projection", Some("Projection"), corpus(TARGET)),
            &index()
        )
        .is_none());
    }

    /// The bindings this rule passes over, enumerated rather than sampled.
    ///
    /// Two belong to [`crate::link_path`], and two name a body this engine
    /// never read. A finding on any of the four would be a second rule
    /// reporting one defect, or a guess about bytes nobody opened.
    #[test]
    fn every_binding_that_is_not_this_rule_produces_nothing() {
        for binding in [
            Binding::Missing {
                path: "docs/spec/gone.md".to_string(),
            },
            Binding::Unnormalizable {
                why: "climbs above the repository root".to_string(),
            },
            Binding::Repository {
                path: "CLAUDE.md".to_string(),
            },
            Binding::External,
        ] {
            assert!(finding(&link("x#y", Some("y"), binding), &index()).is_none());
        }
    }

    /// A link with no fragment, and one with an empty fragment, are both
    /// nothing to this rule. `split_fragment` gives `Some("")` for a
    /// destination that ends in a bare `#`, so the emptiness has to be tested
    /// rather than the option.
    #[test]
    fn a_link_carrying_no_fragment_is_not_this_rules_business() {
        assert!(finding(&link("glossary.md", None, corpus(TARGET)), &index()).is_none());
        assert!(finding(&link("glossary.md#", Some(""), corpus(TARGET)), &index()).is_none());
    }

    /// The case the module comment records as passed over: a corpus path the
    /// census parsed no document out of. There is no heading list, so there is
    /// no verdict, and a finding would be a guess about a file nobody read.
    #[test]
    fn a_corpus_path_with_no_parsed_document_gets_no_verdict_rather_than_a_guess() {
        let untyped = "docs/spec/notes.txt";
        assert_eq!(index().resolves(untyped, "anything"), None);
        assert!(finding(
            &link("notes.txt#anything", Some("anything"), corpus(untyped)),
            &index()
        )
        .is_none());
    }

    /// A run whose scope admitted no links skips with the reason rather than
    /// passing. A pass would report a corpus this rule never read as clean.
    #[test]
    fn a_view_with_no_links_skips_rather_than_passes() {
        let view = CorpusView::only_links_and_anchors(None, None);
        assert!(matches!(Fragments.evaluate(&view), Outcome::Skipped(why) if why == NO_LINKS));
    }

    /// And a run that reached the links but no anchors says the other thing.
    /// Two absences that became one reason would leave an author guessing which
    /// half of the rule was unavailable.
    #[test]
    fn a_view_with_links_and_no_anchors_skips_with_the_other_reason() {
        let links = vec![link(
            "glossary.md#projection",
            Some("projection"),
            corpus(TARGET),
        )];
        let view = CorpusView::only_links_and_anchors(Some(&links), None);
        assert!(matches!(Fragments.evaluate(&view), Outcome::Skipped(why) if why == NO_ANCHORS));
    }

    /// The whole set, and one finding per unresolved member of it. A corpus
    /// whose fragments all resolve passes, which is what lets this rule be an
    /// error at all.
    #[test]
    fn the_unresolved_fragments_of_the_view_become_the_findings() {
        let anchors = index();
        let broken = vec![
            link(
                "glossary.md#projections",
                Some("projections"),
                corpus(TARGET),
            ),
            link("#nope", Some("nope"), Binding::SameDocument),
            link("glossary.md#projection", Some("projection"), corpus(TARGET)),
            link("https://example.com", None, Binding::External),
        ];
        let view = CorpusView::only_links_and_anchors(Some(&broken), Some(&anchors));
        let Outcome::Failed(found) = Fragments.evaluate(&view) else {
            panic!("two unresolved fragments are two findings");
        };
        assert_eq!(found.len(), 2, "{found:#?}");

        let clean = vec![link(
            "glossary.md#projection",
            Some("projection"),
            corpus(TARGET),
        )];
        let view = CorpusView::only_links_and_anchors(Some(&clean), Some(&anchors));
        assert!(matches!(Fragments.evaluate(&view), Outcome::Passed));
    }
}
