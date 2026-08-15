// SPDX-License-Identifier: Apache-2.0
//! The half of a migration that writes: one front-matter value, replaced in
//! place, in a document somebody authored.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#versioning-by-measured-compatibility)
//! splits a migration payload into "what the engine can apply mechanically
//! (`headwater migrate --apply`) and what needs human or agent judgment". A
//! mechanical step names one new value, and applying it means rewriting one
//! scalar of one front-matter key. This module is that rewrite.
//!
//! # It is held to the bar `check --fix` is held to, and by the same means
//!
//! [`crate::fix`] writes a corrections patch and refuses rather than half-write
//! a file: every patch is applied to an in-memory copy, the result is parsed
//! again, and the parse is compared against the parse it came from. Nothing
//! reaches disk until that comparison passes. The same three moves are here,
//! against a different comparison, because a front-matter write and a body
//! write can each be wrong in ways the other cannot.
//!
//! **A body write cannot reach front matter, and this write is nothing but
//! front matter.** [`crate::fix`] refuses any offset before the end of the
//! front-matter block, which is why this is a second writer rather than a
//! fourth `Patch` variant. The two guard opposite halves of one file.
//!
//! # What the read back compares
//!
//! Two things, and neither is "the file still parses".
//!
//! **The body is byte-identical.** A splice inside the front matter that ran
//! off the end of the block would move the fence, and the body after it would
//! then be front matter or the reverse. So the bytes after the closing fence
//! are compared literally, in the patched text against the source.
//!
//! **Every scalar of the front matter is what it was, except the ones this run
//! meant to move.** The comparison is over a walk of the mapping that carries
//! the key path of each scalar in document order, so a key that vanished, a
//! sequence that became a mapping, or a value that bled into the next line all
//! read as a difference. A run that replaced `draft` with a value holding a
//! colon would produce exactly that, and it is refused with nothing written.
//!
//! # A quoted value keeps its quotes
//!
//! The span of a scalar covers the quotes that wrote it, so the bytes at a
//! site are `draft`, `"draft"` or `'draft'`. This module reads which of the
//! three it found and writes the same shape back. Anything else — a block
//! scalar, an anchor, a value the loader folded — is a site this module refuses
//! rather than guesses at, because a guess here rewrites somebody's corpus.

use crate::tree::Composed;
use headwater_doc::{Document, Mapping, Span, Value};
use std::collections::BTreeMap;
use std::path::Path;

/// One value a migration step moves, in one document.
///
/// It carries no step and no payload. The caller has already decided that this
/// step is mechanical and that this document is one the step names, and a
/// writer that re-decided either would be a second reading of a payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Move {
    /// Relative to the root, written with `/`.
    pub path: String,
    /// The front-matter key that holds the value.
    pub key: String,
    pub from: String,
    pub to: String,
}

/// A document this run will not write, and why.
///
/// Closed, on the terms [`crate::Refusal`] is closed: a test is written against
/// a name, so an invariant in an anonymous arm is invisible to the suite.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refused {
    /// The file did not read off the tree.
    Unreadable { path: String, why: String },
    /// The file did not parse as a document.
    Unparsed { path: String, why: String },
    /// The front matter declares no such key. The census said it did, so the
    /// file moved between the walk and this read.
    Absent { path: String, key: String },
    /// The key is there and no scalar under it holds the old value.
    Moved {
        path: String,
        key: String,
        from: String,
    },
    /// The bytes at the value's own span are not the value, in any of the three
    /// shapes this module writes.
    Unquotable {
        path: String,
        key: String,
        found: String,
    },
    /// The patched text parsed, and it is not the document it was.
    Unrecognizable { path: String, why: String },
}

impl Refused {
    pub fn path(&self) -> &str {
        match self {
            Refused::Unreadable { path, .. }
            | Refused::Unparsed { path, .. }
            | Refused::Absent { path, .. }
            | Refused::Moved { path, .. }
            | Refused::Unquotable { path, .. }
            | Refused::Unrecognizable { path, .. } => path,
        }
    }
}

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refused::Unreadable { path, why } => write!(f, "{path} did not read: {why}"),
            Refused::Unparsed { path, why } => {
                write!(f, "{path} did not parse as a document: {why}")
            }
            Refused::Absent { path, key } => write!(
                f,
                "{path} declares no `{key}`, so the corpus moved under this run"
            ),
            Refused::Moved { path, key, from } => write!(
                f,
                "`{key}` of {path} no longer holds `{from}`, so the corpus moved under this run"
            ),
            Refused::Unquotable { path, key, found } => write!(
                f,
                "`{key}` of {path} is written as `{found}`, which is not a plain or quoted \
                 scalar this engine rewrites"
            ),
            Refused::Unrecognizable { path, why } => write!(
                f,
                "the rewrite of {path} does not read back as the document it patched: {why}"
            ),
        }
    }
}

/// Every file this run would write, and every document it will not.
#[derive(Clone, Debug, Default)]
pub struct Written {
    pub files: Vec<Composed>,
    pub refused: Vec<Refused>,
    /// How many values were replaced, over every file above.
    pub replaced: usize,
}

/// Compose the rewrite of every document, writing nothing.
///
/// Moves are grouped by document, because two steps of one payload can name one
/// document and two writes of one file would each be composed against the
/// bytes the other replaced.
///
/// A refusal over one document does not hold back another. That is the same
/// grain [`crate::fix`] refuses at, and it is not the grain the *write* is
/// atomic at: [`crate::tree::Reserved`] takes the whole set or none of it.
pub fn compose(root: &Path, moves: &[Move]) -> Written {
    let mut by_path: BTreeMap<&str, Vec<&Move>> = BTreeMap::new();
    for moving in moves {
        by_path.entry(&moving.path).or_default().push(moving);
    }

    let mut written = Written::default();
    for (path, moves) in by_path {
        match one_file(root, path, &moves) {
            Ok(None) => {}
            Ok(Some((composed, replaced))) => {
                written.files.push(composed);
                written.replaced += replaced;
            }
            Err(refused) => written.refused.push(refused),
        }
    }
    written
}

/// One document, and every move that names it.
fn one_file(root: &Path, path: &str, moves: &[&Move]) -> Result<Option<(Composed, usize)>, Refused> {
    let source = std::fs::read_to_string(root.join(path)).map_err(|error| Refused::Unreadable {
        path: path.to_string(),
        why: error.to_string(),
    })?;
    let before = headwater_doc::parse(&source).map_err(|errors| Refused::Unparsed {
        path: path.to_string(),
        why: format!("{} parse errors", errors.len()),
    })?;

    let mut edits = Vec::new();
    for moving in moves {
        edits.extend(sites_of(path, &before, moving)?);
    }
    if edits.is_empty() {
        return Ok(None);
    }

    // Back to front, so that an offset ahead of an applied edit is still the
    // offset the parse reported.
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.at.start.offset));
    let mut patched = source.clone();
    for edit in &edits {
        let found = edit
            .at
            .slice(&patched)
            .ok_or_else(|| Refused::Unrecognizable {
                path: path.to_string(),
                why: "a value's span does not lie inside the file".to_string(),
            })?;
        let replacement = shaped(found, &edit.from, &edit.to).ok_or_else(|| Refused::Unquotable {
            path: path.to_string(),
            key: edit.key.clone(),
            found: found.to_string(),
        })?;
        patched.replace_range(edit.at.start.offset..edit.at.end.offset, &replacement);
    }

    recognizable(path, &source, &before, &patched, &edits)?;
    Ok(Some((
        Composed {
            path: path.to_string(),
            text: patched,
        },
        edits.len(),
    )))
}

/// One scalar to replace, at its own span.
#[derive(Clone, Debug)]
struct Edit {
    key: String,
    at: Span,
    from: String,
    to: String,
}

/// Every scalar under one key that holds the old value.
///
/// A facet is written as one scalar or as a sequence holding several, and both
/// spellings are what `headwater_compat::sites` matched on. A mapping is
/// neither, and it reaches [`Refused::Moved`] rather than being skipped.
fn sites_of(path: &str, document: &Document, moving: &Move) -> Result<Vec<Edit>, Refused> {
    let node = document
        .facets
        .get(&moving.key)
        .ok_or_else(|| Refused::Absent {
            path: path.to_string(),
            key: moving.key.clone(),
        })?;
    let edit = |span: Span| Edit {
        key: moving.key.clone(),
        at: span,
        from: moving.from.clone(),
        to: moving.to.clone(),
    };
    let edits: Vec<Edit> = match &node.value {
        Value::Scalar(scalar) if scalar.text == moving.from => vec![edit(node.span)],
        Value::Scalar(_) => Vec::new(),
        Value::Seq(items) => items
            .iter()
            .filter(|item| {
                matches!(&item.value, Value::Scalar(scalar) if scalar.text == moving.from)
            })
            .map(|item| edit(item.span))
            .collect(),
        Value::Map(_) => Vec::new(),
    };
    match edits.is_empty() {
        true => Err(Refused::Moved {
            path: path.to_string(),
            key: moving.key.clone(),
            from: moving.from.clone(),
        }),
        false => Ok(edits),
    }
}

/// The replacement bytes, in the shape the source wrote the old value in.
///
/// `None` where the bytes at the span are none of the three shapes. A block
/// scalar and a folded one both reach it, and so does a plain scalar the loader
/// read through an alias.
fn shaped(found: &str, from: &str, to: &str) -> Option<String> {
    match found {
        _ if found == from => Some(to.to_string()),
        _ if found == format!("\"{from}\"") => Some(format!("\"{to}\"")),
        _ if found == format!("'{from}'") => Some(format!("'{to}'")),
        _ => None,
    }
}

/// The read back: the patched text is the document it patched, moved.
fn recognizable(
    path: &str,
    source: &str,
    before: &Document,
    patched: &str,
    edits: &[Edit],
) -> Result<(), Refused> {
    let after = headwater_doc::parse(patched).map_err(|errors| Refused::Unrecognizable {
        path: path.to_string(),
        why: format!("{} parse errors", errors.len()),
    })?;

    let refuse = |why: String| Refused::Unrecognizable {
        path: path.to_string(),
        why,
    };

    // The body, byte for byte. A splice that ran past the closing fence moves
    // it, and the two halves of the file then swap some of their content.
    let was = source.get(before.block.end.offset..).unwrap_or_default();
    let now = patched.get(after.block.end.offset..).unwrap_or_default();
    if was != now {
        return Err(refuse(
            "the bytes after the front matter are not the bytes they were".to_string(),
        ));
    }

    // Every scalar of the front matter, by key path and in document order.
    let moved: BTreeMap<usize, &Edit> = edits.iter().map(|edit| (edit.at.start.offset, edit)).collect();
    let expected: Vec<(String, String)> = scalars(&before.facets, "")
        .into_iter()
        .map(|(at, text, span)| match moved.get(&span.start.offset) {
            Some(edit) if text == edit.from => (at, edit.to.clone()),
            _ => (at, text),
        })
        .collect();
    let read: Vec<(String, String)> = scalars(&after.facets, "")
        .into_iter()
        .map(|(at, text, _)| (at, text))
        .collect();
    if expected != read {
        let first = expected
            .iter()
            .zip(read.iter())
            .find(|(expected, read)| expected != read)
            .map(|(expected, read)| {
                format!("`{}` reads `{}` and `{}` was expected", read.0, read.1, expected.1)
            })
            .unwrap_or_else(|| {
                format!(
                    "the front matter holds {} scalars and {} were expected",
                    read.len(),
                    expected.len()
                )
            });
        return Err(refuse(first));
    }
    Ok(())
}

/// Every scalar of a mapping: its key path, its text, and its span.
///
/// The path carries a sequence index, so two members of one sequence are two
/// entries rather than one repeated. Order is document order, which
/// [`headwater_yaml::Mapping`] keeps.
fn scalars(mapping: &Mapping, under: &str) -> Vec<(String, String, Span)> {
    let mut out = Vec::new();
    for entry in mapping.iter() {
        let at = match under.is_empty() {
            true => entry.key.value.clone(),
            false => format!("{under}.{}", entry.key.value),
        };
        walk(&entry.value.value, entry.value.span, &at, &mut out);
    }
    out
}

fn walk(value: &Value, span: Span, at: &str, out: &mut Vec<(String, String, Span)>) {
    match value {
        Value::Scalar(scalar) => out.push((at.to_string(), scalar.text.clone(), span)),
        Value::Seq(items) => {
            for (index, item) in items.iter().enumerate() {
                walk(&item.value, item.span, &format!("{at}[{index}]"), out);
            }
        }
        Value::Map(map) => out.extend(scalars(map, at)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct Dir(PathBuf);

    impl Dir {
        fn with(label: &str, files: &[(&str, &str)]) -> Dir {
            let root = std::env::temp_dir()
                .join(format!("headwater-migrate-{}-{label}", std::process::id()));
            let _ = std::fs::remove_dir_all(&root);
            std::fs::create_dir_all(&root).expect("a scratch directory");
            for (path, text) in files {
                std::fs::write(root.join(path), text).expect("a fixture file");
            }
            Dir(root)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    const DOCUMENT: &str = "---\nid: DR-repo-0001\nstatus: draft\ntitle: A document\ntags:\n  - draft\n  - other\n---\n\n# A document\n\nThe word draft appears in the body and nothing here rewrites it.\n";

    fn moving(key: &str, from: &str, to: &str) -> Move {
        Move {
            path: "a.md".to_string(),
            key: key.to_string(),
            from: from.to_string(),
            to: to.to_string(),
        }
    }

    #[test]
    fn one_scalar_moves_and_the_body_is_untouched() {
        let dir = Dir::with("one-scalar", &[("a.md", DOCUMENT)]);
        let written = compose(dir.path(), &[moving("status", "draft", "outline")]);
        assert!(written.refused.is_empty(), "{:?}", written.refused);
        assert_eq!(written.replaced, 1);
        let text = &written.files[0].text;
        assert!(text.contains("status: outline"), "{text}");
        assert!(
            text.contains("The word draft appears in the body"),
            "the body is not a site: {text}"
        );
        assert!(
            text.contains("  - draft\n"),
            "another key that holds the value is not a site: {text}"
        );
    }

    #[test]
    fn a_value_inside_a_sequence_moves_where_it_sits() {
        let dir = Dir::with("in-a-sequence", &[("a.md", DOCUMENT)]);
        let written = compose(dir.path(), &[moving("tags", "draft", "outline")]);
        assert!(written.refused.is_empty(), "{:?}", written.refused);
        assert_eq!(written.replaced, 1);
        let text = &written.files[0].text;
        assert!(text.contains("  - outline\n  - other\n"), "{text}");
        assert!(text.contains("status: draft"), "{text}");
    }

    #[test]
    fn a_key_that_no_longer_holds_the_value_is_refused_with_nothing_composed() {
        let dir = Dir::with("moved", &[("a.md", DOCUMENT)]);
        let written = compose(dir.path(), &[moving("status", "current", "settled")]);
        assert!(written.files.is_empty());
        assert_eq!(
            written.refused,
            vec![Refused::Moved {
                path: "a.md".to_string(),
                key: "status".to_string(),
                from: "current".to_string(),
            }]
        );
    }

    #[test]
    fn a_key_the_document_does_not_declare_is_refused() {
        let dir = Dir::with("absent", &[("a.md", DOCUMENT)]);
        let written = compose(dir.path(), &[moving("lifecycle", "draft", "outline")]);
        assert!(written.files.is_empty());
        assert_eq!(
            written.refused,
            vec![Refused::Absent {
                path: "a.md".to_string(),
                key: "lifecycle".to_string(),
            }]
        );
    }

    /// The read back, exercised by a replacement that YAML reads as two keys.
    ///
    /// The splice succeeds and the parse succeeds. What fails is the comparison
    /// against the document that was patched, which is the guard this module
    /// exists for.
    #[test]
    fn a_replacement_that_reads_as_a_second_key_is_refused() {
        let dir = Dir::with("second-key", &[("a.md", DOCUMENT)]);
        let written = compose(dir.path(), &[moving("status", "draft", "outline\nowner: nobody")]);
        assert!(written.files.is_empty(), "nothing is composed");
        assert!(
            matches!(&written.refused[0], Refused::Unrecognizable { .. }),
            "{:?}",
            written.refused
        );
    }

    #[test]
    fn a_quoted_value_keeps_its_quotes() {
        let dir = Dir::with(
            "quoted",
            &[("a.md", "---\nid: X\nstatus: \"draft\"\n---\n\n# A\n")],
        );
        let written = compose(dir.path(), &[moving("status", "draft", "outline")]);
        assert!(written.refused.is_empty(), "{:?}", written.refused);
        assert!(
            written.files[0].text.contains("status: \"outline\""),
            "{}",
            written.files[0].text
        );
    }

    /// Two steps of one payload naming one document compose against one read.
    #[test]
    fn two_moves_over_one_document_both_land() {
        let dir = Dir::with("two-moves", &[("a.md", DOCUMENT)]);
        let written = compose(
            dir.path(),
            &[
                moving("status", "draft", "outline"),
                moving("tags", "other", "another"),
            ],
        );
        assert!(written.refused.is_empty(), "{:?}", written.refused);
        assert_eq!(written.files.len(), 1, "one file, not two");
        assert_eq!(written.replaced, 2);
        let text = &written.files[0].text;
        assert!(text.contains("status: outline"), "{text}");
        assert!(text.contains("  - another\n"), "{text}");
    }

    #[test]
    fn a_refusal_over_one_document_does_not_hold_back_another() {
        let dir = Dir::with(
            "one-of-two",
            &[("a.md", DOCUMENT), ("b.md", "---\nid: Y\n---\n\n# B\n")],
        );
        let written = compose(
            dir.path(),
            &[
                moving("status", "draft", "outline"),
                Move {
                    path: "b.md".to_string(),
                    key: "status".to_string(),
                    from: "draft".to_string(),
                    to: "outline".to_string(),
                },
            ],
        );
        assert_eq!(written.files.len(), 1);
        assert_eq!(written.files[0].path, "a.md");
        assert_eq!(written.refused.len(), 1);
        assert_eq!(written.refused[0].path(), "b.md");
    }
}
