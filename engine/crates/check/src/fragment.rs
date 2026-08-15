// SPDX-License-Identifier: Apache-2.0
//! A Document-origin check: a fragment this document writes into itself.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-five-origins-of-a-check)
//! lists prose-link resolution among the Document-origin examples, and
//! [`headwater_graph::links`] left this half deliberately undone: "whether
//! `#q4--relation-storage` names a heading of the target is a Document check
//! ... a graph build that started checking headings would be the check layer
//! with no scope declaration."
//!
//! # One half of that sentence is reachable at this grain, and one is not
//!
//! A fragment with no path names a heading of **this** document, and this
//! document is what a `Document` scope carries. So that half is here, whole.
//!
//! A fragment on a path names a heading of **another** document, and no scope
//! this engine has carries one. `Document` carries one document. `Edge` carries
//! a relation instance, and a prose link is not a relation
//! ([Q4](../../../../docs/spec/09-decisions.md#q4--relation-storage)).
//! `Neighbourhood` carries the documents one *relation* away, and it carries
//! their identity rather than their body. The grain that would reach it is one
//! document and the documents its prose links reach, and inventing a fifth
//! grain is not this issue's work.
//!
//! [13 — Open obligations](../../../../docs/spec/13-open-obligations.md) carries
//! the rest, with the number: 1519 links of this corpus carry a fragment on a
//! path, against 277 that carry one alone.
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
//! # Every instance says what it read
//!
//! A document that writes no fragment into itself **skips with that reason**
//! rather than passing. A pass would count the document as checked by a rule
//! that had nothing to check, and `coverage.document_unchecked` would then be
//! unable to fire anywhere: this rule generates over every kind, so a vacuous
//! pass here would route every classified document to a check before anything
//! was read.

use crate::finding::{Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView};

pub const RULE: &str = "link.fragment.unresolved";

/// The check. It reads no declaration, and the module comment says why: the
/// taxonomy language has no member that turns prose-link resolution on or off,
/// because [spec 1](../../../../docs/spec/01-conceptual-model.md#prose-links-are-not-relations)
/// makes it a property of a corpus rather than of a kind.
pub struct Fragments;

impl DocumentCheck for Fragments {
    const RULE: &'static str = self::RULE;
    /// The first edition of this rule.
    const VERSION: u32 = 1;
    const NEEDS_BODY: bool = true;

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let Some(body) = view.body() else {
            return Outcome::Passed;
        };
        // A quoted link belongs to another author, and an image is a reference
        // to an asset. Both are the graph build's rules, held to here so that
        // one corpus has one answer about what a prose link is.
        let fragments: Vec<&headwater_doc::Link> = body
            .links
            .iter()
            .filter(|link| !link.quoted && !link.image)
            .filter(|link| link.destination.starts_with('#') && link.destination.len() > 1)
            .collect();
        if fragments.is_empty() {
            return Outcome::Skipped("this document writes no fragment into itself".to_string());
        }

        let anchors = anchors(body);
        let findings = fragments
            .iter()
            .filter(|link| !anchors.contains(&link.destination[1..].to_lowercase()))
            .map(|link| Finding {
                rule: self::RULE,
                severity: Severity::Error,
                obligation: None,
                path: view.path().to_string(),
                line: link.span.start.line,
                column: link.span.start.col,
                message: format!("`{}` names no heading of this document", link.destination),
                remediation: format!(
                    "point it at a heading of {}, or write the heading it names",
                    view.path()
                ),
                // The heading an author meant is a guess among the headings
                // this document has, and spec 12 admits a fix only where one
                // outcome is derivable without judgment.
                patch: None,
            })
            .collect();
        Outcome::failed(findings)
    }
}

/// Every anchor a renderer gives this document, in heading order.
///
/// A repeated heading takes a numeric suffix, first occurrence bare. Two
/// `## Consequences` headings therefore give `consequences` and
/// `consequences-1`, and a link to the second resolves.
fn anchors(body: &headwater_doc::Body) -> Vec<String> {
    let mut anchors: Vec<String> = Vec::new();
    for heading in body.headings() {
        let slug = slug(&heading.text());
        let seen = anchors
            .iter()
            .filter(|known| **known == slug || known.starts_with(&format!("{slug}-")))
            .count();
        anchors.push(match seen {
            0 => slug,
            n => format!("{slug}-{n}"),
        });
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
    fn broken(root: &Path) -> Vec<String> {
        let mut files = Vec::new();
        sources(&engine_root(), &mut files);
        let mut out = Vec::new();
        for file in files {
            let src = std::fs::read_to_string(&file).expect("a source of this engine");
            let markdown = comment_markdown(&src);
            let body = headwater_doc::body::scan(&markdown, &markdown, 0);
            let dir = file.parent().expect("a source has a directory");
            for link in &body.links {
                // The class this holds is the one a reader follows into this
                // repository. A link into rustdoc's own output tree resolves
                // against `target/doc` and not against the source, so a
                // filesystem test of one would report a defect that is not one.
                if link.image || !link.destination.starts_with('.') {
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
        let broken = broken(&root);
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

    #[test]
    fn a_repeated_heading_takes_a_numeric_suffix() {
        let body = scan("# One\n\n# One\n\n# One\n", "# One\n\n# One\n\n# One\n", 0);
        assert_eq!(anchors(&body), ["one", "one-1", "one-2"]);
    }
}
