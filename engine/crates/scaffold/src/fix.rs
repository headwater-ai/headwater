// SPDX-License-Identifier: Apache-2.0
//! Applying the patches a run produced: `headwater check --fix`.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#fixability) fixes what may
//! be offered: "A fix is offered only when it is **mechanical and total** — one
//! correct outcome, derivable without judgment." [`headwater_check::Patch`] is
//! that offer, and this module is the only thing that acts on one.
//!
//! # Why it lives beside the scaffolder
//!
//! Because [`crate::write::splice`] is here. The named example of a mechanical
//! fix is "add a missing reciprocal link", and the splice already writes
//! exactly that: one edge half into a target document's front matter, across
//! the three shapes a target can be in. A second writer into one `relations:`
//! block would disagree with the first the day a document nested differently.
//! So [`Patch::Half`] reaches the splice rather than a copy of it, and this
//! module holds the second shape only.
//!
//! # The guarantee, and it is the splice's guarantee applied to prose
//!
//! The splice assumes an indentation that no declaration states, and it is safe
//! because it parses its own result and looks for the half it wrote. A patch
//! over prose needs the same shape of promise against
//! [`headwater_doc::sentences`], which
//! [spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots) holds
//! as a correctness root: a rewrite at a wrong span **corrupts a document**
//! rather than reporting a wrong line.
//!
//! So a text patch passes four guards, and every failure is a refusal with
//! nothing written for that file.
//!
//! 1. **The patch is in the body.** Its range opens after the front-matter
//!    block. A prose rule reads prose, and a facet is a different writer with a
//!    different read-back.
//! 2. **The bytes are what the check read.** `expect` states what the check
//!    believed lay at the offset, and this is where the belief meets the file.
//!    A code span, a link and a block quote each reach a rule with their markup
//!    removed, so an offset taken from the text a reader sees can land early;
//!    this guard is what catches that, whatever the cause.
//! 3. **The patch lies inside one authored run that reads every byte it
//!    spans.** A patch that straddled two runs would be a patch over markup,
//!    and a run whose source spends more bytes than its text reads has no
//!    offset arithmetic that holds.
//! 4. **The result re-reads as the same document with the substitution in it.**
//!    The patched text is parsed again. Every block keeps its kind, its quote
//!    depth, its run count and its soft breaks; every run keeps its ownership;
//!    every link keeps its destination and its form; and every run's text is the
//!    text it had, with the substitutions this run made and no others.
//!
//! Guard 4 is the one that makes the other three insurance rather than the
//! promise. A patch that landed inside a link's destination, or that turned a
//! word into markup, changes the parse — and the parse is what is compared.
//!
//! # One file at a time, and all of a file or none of it
//!
//! [`crate::write::apply`] puts every file of a scaffold on the tree or none of
//! them, because a half-written scaffold leaves an edge with one end. A fix is
//! not that shape. Each file's patches are independent of every other file's,
//! and a refusal over one document is no evidence about another. So the unit is
//! the file: all of its patches land or none of them do, and a refusal is
//! reported beside the files that did land.
//!
//! That sentence used to say *composes every file before it writes any*, which
//! was a claim about the phase that cannot fail standing in for the phase that
//! can: composition finished, and the write beneath it was a loop that stopped
//! on its first error. Both writers now go through [`crate::tree::Reserved`],
//! and [`apply`] here calls [`crate::tree::Reserved::over`] alone because a fix
//! creates nothing.

use crate::write::splice;
use crate::{Half, Refusal};
use headwater_check::Patch;
use headwater_doc::body::{Block, Body, Ownership, Run};
use std::collections::BTreeMap;
use std::path::Path;

/// One file, patched and not yet on disk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fixed {
    pub path: String,
    pub text: String,
    /// How many patches this file took.
    pub applied: usize,
}

/// Why a file was not fixed. A closed set, for the reason [`Refusal`] is one: a
/// test is written against a name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refused {
    /// The file a patch names could not be read, or does not parse as it
    /// stands.
    Unreadable { path: String, why: String },
    /// The patch opens inside the front-matter block. See guard 1.
    NotInTheBody { path: String, at: usize },
    /// The bytes at the offset are not the bytes the check read. See guard 2.
    Moved {
        path: String,
        at: usize,
        expected: String,
        found: String,
    },
    /// No single authored run holds the range, or the run that does spends
    /// more bytes than its text reads. See guard 3.
    Unanchored { path: String, at: usize },
    /// Two patches want the same bytes.
    Overlapping { path: String, at: usize },
    /// The result no longer parses. See guard 4.
    Unparseable { path: String, why: String },
    /// The result parses and reads as a different document. See guard 4.
    Unrecognizable { path: String, why: String },
    /// The reciprocal half did not splice. The reason is the scaffolder's own.
    HalfUnwritable { path: String, why: String },
}

impl Refused {
    pub fn path(&self) -> &str {
        match self {
            Refused::Unreadable { path, .. }
            | Refused::NotInTheBody { path, .. }
            | Refused::Moved { path, .. }
            | Refused::Unanchored { path, .. }
            | Refused::Overlapping { path, .. }
            | Refused::Unparseable { path, .. }
            | Refused::Unrecognizable { path, .. }
            | Refused::HalfUnwritable { path, .. } => path,
        }
    }
}

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refused::Unreadable { path, why } => write!(f, "{path} cannot be read: {why}"),
            Refused::NotInTheBody { path, at } => write!(
                f,
                "the patch at byte {at} of {path} opens inside the front-matter block, and this \
                 writer only rewrites prose"
            ),
            Refused::Moved {
                path,
                at,
                expected,
                found,
            } => write!(
                f,
                "{path} holds `{found}` at byte {at} where the check read `{expected}`, so the \
                 file moved under the run and nothing is written"
            ),
            Refused::Unanchored { path, at } => write!(
                f,
                "no single run of authored prose in {path} holds byte {at} and reads every byte \
                 it spans, so no offset here is safe to rewrite"
            ),
            Refused::Overlapping { path, at } => write!(
                f,
                "two patches over {path} both want byte {at}, and this writer applies neither"
            ),
            Refused::Unparseable { path, why } => write!(
                f,
                "the patched {path} does not parse: {why}. Nothing is written"
            ),
            Refused::Unrecognizable { path, why } => write!(
                f,
                "the patched {path} parses and reads as a different document: {why}. Nothing is \
                 written"
            ),
            Refused::HalfUnwritable { path, why } => write!(f, "{path}: {why}"),
        }
    }
}

/// What one `--fix` run would do to a tree, composed and not yet written.
#[derive(Clone, Debug, Default)]
pub struct Composed {
    /// In path order, so two runs over one tree report the same thing.
    pub files: Vec<Fixed>,
    pub refused: Vec<Refused>,
}

impl Composed {
    pub fn is_empty(&self) -> bool {
        self.files.is_empty() && self.refused.is_empty()
    }
}

/// Compose every file the patches touch, and write none.
///
/// The caller writes them, so a dry run and a real one differ by one loop
/// rather than by a second composer. That is [`crate::write::compose`]'s rule,
/// and it holds here for the same reason.
pub fn compose(root: &Path, patches: &[Patch]) -> Composed {
    let mut grouped: BTreeMap<&str, Vec<&Patch>> = BTreeMap::new();
    for patch in patches {
        grouped.entry(patch.path()).or_default().push(patch);
    }
    let mut out = Composed::default();
    for (path, patches) in grouped {
        match one_file(root, path, &patches) {
            Ok(Some(fixed)) => out.files.push(fixed),
            Ok(None) => {}
            Err(refused) => out.refused.push(refused),
        }
    }
    out
}

/// Write what [`compose`] produced, all of it or none of it.
///
/// The batch goes through [`crate::tree::Reserved`], which is the one writer in
/// this crate that puts a set of files on a tree. A loop of `std::fs::write`
/// here would stop on the first failure with every file before it written, and
/// `--fix` would then have applied part of a corrections patch with nothing
/// saying which part.
pub fn apply(root: &Path, files: &[Fixed]) -> Result<(), Refused> {
    let composed = files
        .iter()
        .map(|file| crate::tree::Composed {
            path: file.path.clone(),
            text: file.text.clone(),
        })
        .collect();
    let unreadable = |path: String, why: String| Refused::Unreadable { path, why };
    crate::tree::Reserved::over(root, composed)
        .map_err(|unopened| unreadable(unopened.path, unopened.why))?
        .commit()
        .map_err(|halted| unreadable(halted.path().to_string(), halted.to_string()))?;
    Ok(())
}

/// Every patch over one file, or a refusal and nothing written.
///
/// The text patches go first and the halves after them. A splice puts lines
/// into the front matter, which moves every byte of the body along, so an
/// offset the check computed is an offset into the file before the splice.
fn one_file(root: &Path, path: &str, patches: &[&Patch]) -> Result<Option<Fixed>, Refused> {
    let source = std::fs::read_to_string(root.join(path)).map_err(|error| Refused::Unreadable {
        path: path.to_string(),
        why: error.to_string(),
    })?;

    let text: Vec<&Patch> = patches
        .iter()
        .copied()
        .filter(|patch| matches!(patch, Patch::Text { .. }))
        .collect();
    let halves = halves_of(path, patches);

    let mut patched = match text.is_empty() {
        true => source.clone(),
        false => substitute(path, &source, &text)?,
    };
    for half in &halves {
        patched = splice(&patched, half).map_err(|refusal| match refusal {
            Refusal::ReciprocalUnwritable { path, why } => Refused::HalfUnwritable { path, why },
            other => Refused::HalfUnwritable {
                path: path.to_string(),
                why: other.to_string(),
            },
        })?;
    }
    if patched == source {
        return Ok(None);
    }
    Ok(Some(Fixed {
        path: path.to_string(),
        text: patched,
        applied: text.len() + halves.len(),
    }))
}

/// The halves this file owes, each one once.
///
/// Two documents can each declare the same missing half of one pair, and the
/// splice would then write it twice.
fn halves_of(path: &str, patches: &[&Patch]) -> Vec<Half> {
    let mut out: Vec<Half> = Vec::new();
    for patch in patches {
        let Patch::Half { relation, id, .. } = patch else {
            continue;
        };
        if out
            .iter()
            .any(|half| &half.relation == relation && &half.id == id)
        {
            continue;
        }
        out.push(Half {
            path: path.to_string(),
            relation: relation.clone(),
            id: id.clone(),
            attributes: Vec::new(),
        });
    }
    out
}

/// Every text patch over one file, under the four guards.
fn substitute(path: &str, source: &str, patches: &[&Patch]) -> Result<String, Refused> {
    let before = headwater_doc::parse(source).map_err(|errors| Refused::Unreadable {
        path: path.to_string(),
        why: format!("{} parse errors", errors.len()),
    })?;

    // Sorted by offset, so the overlap test is a comparison with the last one
    // taken and the application below runs backwards over a stable order.
    let mut ordered: Vec<(usize, usize, &str, &str)> = Vec::new();
    for patch in patches {
        let Patch::Text {
            start,
            end,
            expect,
            replacement,
            ..
        } = patch
        else {
            continue;
        };
        // Guard 1.
        if *start < before.block.end.offset {
            return Err(Refused::NotInTheBody {
                path: path.to_string(),
                at: *start,
            });
        }
        // Guard 2.
        let found = source.get(*start..*end).unwrap_or_default();
        if found != expect.as_str() {
            return Err(Refused::Moved {
                path: path.to_string(),
                at: *start,
                expected: expect.clone(),
                found: found.to_string(),
            });
        }
        // Guard 3.
        if anchoring(&before.body, *start, *end).is_none() {
            return Err(Refused::Unanchored {
                path: path.to_string(),
                at: *start,
            });
        }
        ordered.push((*start, *end, expect, replacement));
    }
    ordered.sort_by_key(|(start, end, _, _)| (*start, *end));
    for pair in ordered.windows(2) {
        if pair[0].1 > pair[1].0 {
            return Err(Refused::Overlapping {
                path: path.to_string(),
                at: pair[1].0,
            });
        }
    }

    let mut patched = source.to_string();
    for (start, end, _, replacement) in ordered.iter().rev() {
        patched.replace_range(*start..*end, replacement);
    }

    // Guard 4. The read back.
    let after = headwater_doc::parse(&patched).map_err(|errors| Refused::Unparseable {
        path: path.to_string(),
        why: format!("{} parse errors", errors.len()),
    })?;
    match recognizable(&before.body, &after.body, &ordered) {
        Ok(()) => Ok(patched),
        Err(why) => Err(Refused::Unrecognizable {
            path: path.to_string(),
            why,
        }),
    }
}

/// The run that holds a range, and nothing where no single run does.
///
/// Authored prose, and a run that reads every byte it spans. A range that
/// straddles two runs crosses markup the parser removed, and a run whose source
/// is wider than its text has no offset arithmetic that holds.
fn anchoring(body: &Body, start: usize, end: usize) -> Option<(&Block, &Run)> {
    body.blocks
        .iter()
        .filter(|block| block.quote_depth == 0)
        .flat_map(|block| block.runs.iter().map(move |run| (block, run)))
        .find(|(_, run)| {
            run.ownership == Ownership::Authored
                && run.span.start.offset <= start
                && end <= run.span.end.offset
                && run.span.end.offset - run.span.start.offset == run.text.len()
        })
}

/// Whether the patched body is the original body with these substitutions in
/// it, and nothing else.
fn recognizable(
    before: &Body,
    after: &Body,
    patches: &[(usize, usize, &str, &str)],
) -> Result<(), String> {
    if before.blocks.len() != after.blocks.len() {
        return Err(format!(
            "it holds {} blocks where the original held {}",
            after.blocks.len(),
            before.blocks.len()
        ));
    }
    if before.links.len() != after.links.len() {
        return Err(format!(
            "it holds {} links where the original held {}",
            after.links.len(),
            before.links.len()
        ));
    }
    for (was, is) in before.links.iter().zip(&after.links) {
        if was.destination != is.destination || was.form != is.form || was.image != is.image {
            return Err(format!(
                "the link to `{}` is not the one it was",
                was.destination
            ));
        }
    }
    for (index, (was, is)) in before.blocks.iter().zip(&after.blocks).enumerate() {
        if was.kind != is.kind || was.quote_depth != is.quote_depth {
            return Err(format!("block {index} is not the kind of block it was"));
        }
        if was.runs.len() != is.runs.len() || was.soft_breaks.len() != is.soft_breaks.len() {
            return Err(format!("block {index} is not built of the runs it was"));
        }
        for (position, (was, is)) in was.runs.iter().zip(&is.runs).enumerate() {
            if was.ownership != is.ownership {
                return Err(format!(
                    "run {position} of block {index} is now {} text",
                    is.ownership.name()
                ));
            }
            let expected = expected_text(was, patches);
            if expected != is.text {
                return Err(format!(
                    "run {position} of block {index} reads `{}` where this fix expected `{expected}`",
                    is.text
                ));
            }
        }
    }
    Ok(())
}

/// One run's text, with the substitutions that fall inside it applied.
fn expected_text(run: &Run, patches: &[(usize, usize, &str, &str)]) -> String {
    let (from, to) = (run.span.start.offset, run.span.end.offset);
    let mut text = run.text.clone();
    if to - from != run.text.len() {
        return text;
    }
    for (start, end, _, replacement) in patches.iter().rev() {
        if *start >= from && *end <= to {
            text.replace_range((start - from)..(end - from), replacement);
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_patch(path: &str, source: &str, word: &str, replacement: &str) -> Patch {
        let at = source.find(word).expect("the word is in the source");
        Patch::Text {
            path: path.to_string(),
            start: at,
            end: at + word.len(),
            expect: word.to_string(),
            replacement: replacement.to_string(),
        }
    }

    fn tree(files: &[(&str, &str)]) -> tempdir::Dir {
        tempdir::Dir::with(files)
    }

    const DOC: &str = "---\nid: D-1\n---\n\n# A title\n\nThe behaviour of a check.\n";

    #[test]
    fn a_substitution_lands_and_the_rest_of_the_file_is_untouched() {
        let dir = tree(&[("a.md", DOC)]);
        let composed = compose(
            dir.path(),
            &[text_patch("a.md", DOC, "behaviour", "behavior")],
        );
        assert!(composed.refused.is_empty(), "{:?}", composed.refused);
        assert_eq!(composed.files.len(), 1);
        assert_eq!(
            composed.files[0].text,
            "---\nid: D-1\n---\n\n# A title\n\nThe behavior of a check.\n"
        );
    }

    /// Guard 2, and the case the verifier of this change reaches for first. A
    /// patch computed over one version of a file and applied to another names
    /// bytes that hold something else.
    #[test]
    fn a_file_that_moved_under_the_patch_is_refused_with_nothing_written() {
        let dir = tree(&[(
            "a.md",
            "---\nid: D-1\n---\n\n# A title\n\nQuite other words here.\n",
        )]);
        let composed = compose(
            dir.path(),
            &[text_patch("a.md", DOC, "behaviour", "behavior")],
        );
        assert!(composed.files.is_empty());
        assert!(
            matches!(composed.refused[0], Refused::Moved { .. }),
            "{:?}",
            composed.refused
        );
    }

    /// Guard 3. A code span reaches a rule with its backticks gone, so an
    /// offset from the text a reader sees lands one byte early. The bytes at
    /// that offset are the word, so guard 2 passes and this one refuses.
    #[test]
    fn an_offset_inside_a_code_span_is_refused() {
        let source = "---\nid: D-1\n---\n\n# A title\n\nThe `behaviour` flag.\n";
        let dir = tree(&[("a.md", source)]);
        let composed = compose(
            dir.path(),
            &[text_patch("a.md", source, "behaviour", "behavior")],
        );
        assert!(composed.files.is_empty());
        assert!(
            matches!(composed.refused[0], Refused::Unanchored { .. }),
            "{:?}",
            composed.refused
        );
    }

    /// The same for a quotation. Spec 3 puts another author's words outside
    /// every lexical rule, and this is that rule at the write end.
    #[test]
    fn an_offset_inside_a_block_quote_is_refused() {
        let source = "---\nid: D-1\n---\n\n# A title\n\n> The behaviour they describe.\n";
        let dir = tree(&[("a.md", source)]);
        let composed = compose(
            dir.path(),
            &[text_patch("a.md", source, "behaviour", "behavior")],
        );
        assert!(composed.files.is_empty());
        assert!(
            matches!(composed.refused[0], Refused::Unanchored { .. }),
            "{:?}",
            composed.refused
        );
    }

    /// A link's text is authored prose, so this one lands — and the link's
    /// destination and form survive it, which is what guard 4 compares.
    #[test]
    fn a_substitution_inside_a_links_text_lands_and_keeps_the_link() {
        let source = "---\nid: D-1\n---\n\n# A title\n\nRead [the behaviour note](x.md) first.\n";
        let dir = tree(&[("a.md", source)]);
        let composed = compose(
            dir.path(),
            &[text_patch("a.md", source, "behaviour", "behavior")],
        );
        assert!(composed.refused.is_empty(), "{:?}", composed.refused);
        assert!(composed.files[0].text.contains("[the behavior note](x.md)"));
    }

    /// Guard 1. Front matter has its own writer and its own read-back.
    #[test]
    fn an_offset_inside_the_front_matter_is_refused() {
        let source = "---\nid: D-1\nname: behaviour\n---\n\n# A title\n\nWords.\n";
        let dir = tree(&[("a.md", source)]);
        let composed = compose(
            dir.path(),
            &[text_patch("a.md", source, "behaviour", "behavior")],
        );
        assert!(composed.files.is_empty());
        assert!(
            matches!(composed.refused[0], Refused::NotInTheBody { .. }),
            "{:?}",
            composed.refused
        );
    }

    /// Guard 4, and the reason it is the promise rather than insurance. A
    /// replacement that reads as markup changes the parse, and the parse is
    /// what is compared.
    #[test]
    fn a_replacement_that_reads_as_markup_is_refused() {
        let dir = tree(&[("a.md", DOC)]);
        let composed = compose(
            dir.path(),
            &[text_patch("a.md", DOC, "behaviour", "beha`viou`r")],
        );
        assert!(composed.files.is_empty());
        assert!(
            matches!(composed.refused[0], Refused::Unrecognizable { .. }),
            "{:?}",
            composed.refused
        );
    }

    #[test]
    fn two_patches_over_one_range_are_refused_together() {
        let dir = tree(&[("a.md", DOC)]);
        let composed = compose(
            dir.path(),
            &[
                text_patch("a.md", DOC, "behaviour", "behavior"),
                text_patch("a.md", DOC, "behaviou", "behavio"),
            ],
        );
        assert!(composed.files.is_empty());
        assert!(
            matches!(composed.refused[0], Refused::Overlapping { .. }),
            "{:?}",
            composed.refused
        );
    }

    /// Several patches over one file all land, and the later offsets are not
    /// moved by the earlier substitutions.
    #[test]
    fn two_substitutions_in_one_file_both_land_at_the_bytes_they_named() {
        let source =
            "---\nid: D-1\n---\n\n# A title\n\nThe behaviour of it.\n\nA catalogue of them.\n";
        let dir = tree(&[("a.md", source)]);
        let composed = compose(
            dir.path(),
            &[
                text_patch("a.md", source, "behaviour", "behavior"),
                text_patch("a.md", source, "catalogue", "catalog"),
            ],
        );
        assert!(composed.refused.is_empty(), "{:?}", composed.refused);
        assert_eq!(composed.files[0].applied, 2);
        assert!(composed.files[0].text.contains("The behavior of it."));
        assert!(composed.files[0].text.contains("A catalog of them."));
    }

    /// A reciprocal half goes through the splice, and the splice is the one
    /// writer into a `relations:` block.
    #[test]
    fn a_half_is_written_by_the_splice() {
        let source = "---\nid: D-2\nrelations:\n  cites:\n    - D-9\n---\n\n# Two\n\nWords.\n";
        let dir = tree(&[("b.md", source)]);
        let composed = compose(
            dir.path(),
            &[Patch::Half {
                path: "b.md".to_string(),
                relation: "cited_by".to_string(),
                id: "D-1".to_string(),
            }],
        );
        assert!(composed.refused.is_empty(), "{:?}", composed.refused);
        assert!(composed.files[0].text.contains("  cited_by:\n    - D-1\n"));
    }

    /// The same half asked for twice writes it once. Two documents can each
    /// report the pair.
    #[test]
    fn one_half_asked_for_twice_is_written_once() {
        let source = "---\nid: D-2\n---\n\n# Two\n\nWords.\n";
        let dir = tree(&[("b.md", source)]);
        let half = Patch::Half {
            path: "b.md".to_string(),
            relation: "cited_by".to_string(),
            id: "D-1".to_string(),
        };
        let composed = compose(dir.path(), &[half.clone(), half]);
        assert!(composed.refused.is_empty(), "{:?}", composed.refused);
        assert_eq!(composed.files[0].text.matches("- D-1").count(), 1);
    }

    #[test]
    fn a_patch_that_changes_nothing_writes_no_file() {
        let dir = tree(&[("a.md", DOC)]);
        let composed = compose(
            dir.path(),
            &[text_patch("a.md", DOC, "behaviour", "behaviour")],
        );
        assert!(composed.is_empty(), "{composed:?}");
    }

    /// A refusal over one document is no evidence about another, so a correct
    /// fix elsewhere still lands.
    #[test]
    fn a_refusal_over_one_file_does_not_hold_back_another() {
        let bad = "---\nid: D-1\n---\n\n# A title\n\nThe `behaviour` flag.\n";
        let dir = tree(&[("a.md", DOC), ("b.md", bad)]);
        let composed = compose(
            dir.path(),
            &[
                text_patch("a.md", DOC, "behaviour", "behavior"),
                text_patch("b.md", bad, "behaviour", "behavior"),
            ],
        );
        assert_eq!(composed.files.len(), 1);
        assert_eq!(composed.files[0].path, "a.md");
        assert_eq!(composed.refused.len(), 1);
        assert_eq!(composed.refused[0].path(), "b.md");
    }

    /// A directory that cleans itself up, so no fixture leaves a tree behind.
    mod tempdir {
        use std::path::{Path, PathBuf};
        use std::sync::atomic::{AtomicUsize, Ordering};

        /// One name per call, because these tests run in parallel and a name
        /// built from a thread identity is reused when a thread ends.
        static NEXT: AtomicUsize = AtomicUsize::new(0);

        pub(super) struct Dir(PathBuf);

        impl Dir {
            pub(super) fn with(files: &[(&str, &str)]) -> Dir {
                let root = std::env::temp_dir().join(format!(
                    "headwater-fix-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
                let _ = std::fs::remove_dir_all(&root);
                std::fs::create_dir_all(&root).expect("a scratch directory");
                for (path, text) in files {
                    std::fs::write(root.join(path), text).expect("a fixture file");
                }
                Dir(root)
            }

            pub(super) fn path(&self) -> &Path {
                &self.0
            }
        }

        impl Drop for Dir {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
    }
}
