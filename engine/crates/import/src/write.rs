// SPDX-License-Identifier: Apache-2.0
//! Putting an imported edge into the document that will declare it.
//!
//! The splice is [`headwater_scaffold::write::splice`] and not a second one.
//! That function guesses an indentation, writes, parses its own result, and
//! refuses rather than half-writes a file. A second splice here would be a
//! second guess with no read-back behind it, and the documents an importer
//! writes into are documents somebody else wrote.
//!
//! What this module owns is the pair of decisions the splice does not make:
//! whether a document already declares an edge at the revision the snapshot
//! pinned, and the name of the attribute an imported edge carries.
//!
//! # All of a run or none of it, through the writer that already ships
//!
//! One `headwater import --write` puts an edge half into every document the
//! snapshot names, and [`apply`] was a loop of `std::fs::write` that stopped on
//! its first error with every document before it written. What that left was a
//! corpus half-way through one import: some documents carrying edges at the
//! pinned revision and some not, with nothing on the tree to say which run
//! stopped or where.
//!
//! [`apply`] now goes through [`headwater_scaffold::tree::Reserved`], which
//! opens every document of the run before it writes any of them and puts back
//! what it wrote if a write stops. **This verb creates nothing**, so it needs
//! [`headwater_scaffold::tree::Reserved::over`] and
//! [`headwater_scaffold::tree::Reserved::commit`] and no create step at all:
//! [`compose`] read every one of these paths off the tree, and a path it could
//! not read is its own refusal, so every target of the write already exists —
//! which is exactly what `over` requires. `headwater new` composes one document
//! that is not there yet and needs the create step beside these two. That is the
//! difference between the two copies of this loop, and it is why the fix for
//! them is one mechanism and two callers rather than two mechanisms.

use headwater_scaffold::Half;
use std::path::Path;

/// The instance attribute an imported edge carries.
///
/// [Spec 2](../../../../docs/spec/02-taxonomy-model.md#instance-attributes-and-which-end-owns-each-one)
/// declares it by this name, owned by the edge: "the pinned revision this edge
/// was checked against". It is what makes Q19's drift report per edge rather
/// than per snapshot. When a later fetch advances the pin, the edges whose
/// recorded revision no longer matches the snapshot's are the suspect ones, and
/// the finding names the document whose author can act.
///
/// The name moved to `headwater_graph::edges` when `relation.target.suspect`
/// arrived, because the rule that reads it is in a crate this one depends on.
/// It is re-exported here so that a reader of the writer still meets it beside
/// the splice that writes it.
pub use headwater_graph::edges::VERIFIED_REVISION;

/// One document, patched and not yet on disk.
#[derive(Clone, Debug)]
pub struct Composed {
    pub path: String,
    pub text: String,
}

/// Whether a document already declares this edge at this revision.
///
/// Three answers collapse to two on purpose. A document with no such edge and a
/// document with the edge at a different revision both read as absent, because
/// an import that advanced a revision has something to write in each case. What
/// this stops is the second run over one unchanged snapshot writing a second
/// copy of every line the first one wrote.
pub fn declares(root: &Path, path: &str, relation: &str, to: &str, revision: &str) -> bool {
    let Ok(text) = std::fs::read_to_string(root.join(path)) else {
        return false;
    };
    let Ok(parsed) = headwater_doc::parse(&text) else {
        return false;
    };
    parsed
        .facets
        .get("relations")
        .and_then(|block| block.value.as_map())
        .and_then(|block| block.get(relation))
        .and_then(|targets| targets.value.as_seq())
        .map(|targets| {
            targets.iter().any(|target| {
                let Some(map) = target.value.as_map() else {
                    return false;
                };
                let text = |key: &str| {
                    map.get(key)
                        .and_then(|node| node.value.as_scalar())
                        .map(|scalar| scalar.text.as_str())
                };
                text("to") == Some(to) && text(VERIFIED_REVISION) == Some(revision)
            })
        })
        .unwrap_or(false)
}

/// Compose every document this import touches, and write none.
///
/// One document can take several edges, and each one goes through the splice in
/// turn over the text the last one produced. So the read-back that the splice
/// runs is a read-back of the whole file after every edge, rather than of the
/// first edge alone.
pub fn compose(root: &Path, edges: &[&crate::Proposed]) -> Result<Vec<Composed>, String> {
    let mut composed: Vec<Composed> = Vec::new();
    for edge in edges {
        let position = composed.iter().position(|file| file.path == edge.path);
        let source = match position {
            Some(position) => composed[position].text.clone(),
            None => std::fs::read_to_string(root.join(&edge.path))
                .map_err(|error| format!("{}: {error}", edge.path))?,
        };
        let half = Half {
            relation: edge.relation.clone(),
            path: edge.path.clone(),
            id: edge.to.clone(),
            attributes: vec![(
                VERIFIED_REVISION.to_string(),
                edge.verified_revision.clone(),
            )],
        };
        let text = headwater_scaffold::write::splice(&source, &half)
            .map_err(|refusal| refusal.to_string())?;
        match position {
            Some(position) => composed[position].text = text,
            None => composed.push(Composed {
                path: edge.path.clone(),
                text,
            }),
        }
    }
    Ok(composed)
}

/// Why a write did not go, and which phase is saying so.
///
/// Two arms because there are two phases and they leave different trees. One
/// value over both of them cannot print a true sentence about either, which is
/// the defect `headwater_scaffold::Refusal::ReciprocalUnwritable` carried until
/// it was split: it was raised before a byte moved and again after one, and its
/// message ended *Nothing was written*. The same fault was here, one layer up —
/// the caller printed *the write stopped part way* over every failure of
/// [`apply`], including the first-document failure that stopped before it
/// started. [`Unwritten::headline`] is that sentence, and it now comes off the
/// value rather than off the call site.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unwritten {
    /// A document this run would have written could not be opened for writing.
    ///
    /// The reservation opens every document of the run before it writes any of
    /// them, so the tree here is the tree the run started with.
    Unopened { path: String, why: String },
    /// The write started and stopped, and the rollback put back what it could.
    ///
    /// `report` is [`headwater_scaffold::tree::Halted`]'s own account, which
    /// names the document that stopped the run, what it holds now, and which of
    /// the others went back. Nothing here restates it.
    ///
    /// **No case in this crate reaches this arm, and that is stated rather
    /// than hidden.** It is raised by a `write_all` that fails part way through
    /// a handle `Reserved::over` already opened, which
    /// `headwater_scaffold::tree::Reserved::commit_with`'s own comment argues
    /// is not provocable from an unprivileged, deterministic, thread-safe test.
    /// The rollback behind it is held by that module's in-module cases, through
    /// the seam it owns. What this crate holds is that the arm exists, that it
    /// carries the report rather than a sentence of its own, and that the other
    /// arm is not printed in its place.
    Halted { path: String, report: String },
}

impl Unwritten {
    /// The document the run stopped on.
    pub fn path(&self) -> &str {
        match self {
            Unwritten::Unopened { path, .. } | Unwritten::Halted { path, .. } => path,
        }
    }

    /// What the run did to the tree, in the one line a caller prints first.
    ///
    /// It is on the type rather than at the call site because a caller that
    /// chooses this sentence itself is a caller that can go on printing it
    /// after a new arm makes it false.
    pub fn headline(&self) -> &'static str {
        match self {
            Unwritten::Unopened { .. } => "nothing was written",
            Unwritten::Halted { .. } => "the write stopped part way",
        }
    }
}

impl std::fmt::Display for Unwritten {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unwritten::Unopened { path, why } => write!(
                f,
                "{path} takes an edge half this run would write, and it could not be opened for \
                 writing: {why}"
            ),
            Unwritten::Halted { report, .. } => write!(f, "{report}"),
        }
    }
}

/// Write what [`compose`] produced: all of it, or none of it.
///
/// Every path here was read off the tree by [`compose`], so every one of them
/// already exists and [`headwater_scaffold::tree::Reserved::over`] is the whole
/// of what this run needs. There is no create on this path and therefore no
/// question about undoing one — the asymmetry that made `headwater new`'s copy
/// of this loop the harder half is absent here.
pub fn apply(root: &Path, files: &[Composed]) -> Result<(), Unwritten> {
    let writing = files
        .iter()
        .map(|file| headwater_scaffold::tree::Composed {
            path: file.path.clone(),
            text: file.text.clone(),
        })
        .collect();
    headwater_scaffold::tree::Reserved::over(root, writing)
        .map_err(|unopened| Unwritten::Unopened {
            path: unopened.path,
            why: unopened.why,
        })?
        .commit()
        .map_err(|halted| Unwritten::Halted {
            path: halted.path().to_string(),
            report: halted.to_string(),
        })?;
    Ok(())
}
