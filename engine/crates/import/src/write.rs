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
pub const VERIFIED_REVISION: &str = "verified_revision";

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

/// Write what [`compose`] produced.
pub fn apply(root: &Path, files: &[Composed]) -> Result<(), String> {
    for file in files {
        std::fs::write(root.join(&file.path), &file.text)
            .map_err(|error| format!("{}: {error}", file.path))?;
    }
    Ok(())
}
