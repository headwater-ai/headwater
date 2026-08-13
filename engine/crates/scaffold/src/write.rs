// SPDX-License-Identifier: Apache-2.0
//! Turning a [`Plan`] into bytes, and putting the bytes on a tree.
//!
//! Two documents can move. The new one, which this crate composes whole, and
//! the document at the far end of each edge whose reciprocity is required.
//!
//! # The far end is spliced, and the result is read back before it lands
//!
//! [Q4](../../../../docs/decisions/0004-relation-storage.md) makes front matter
//! the one place an edge is authored, and reciprocity obliges the far document
//! to declare its half. Nobody maintains that half by hand at any scale, which
//! is the finding the decision-record doctrine reports about `adr-tools` and
//! `log4brains`, and it is most of what a scaffolder is for.
//!
//! A splice makes one assumption that no declaration states: that a nested
//! block is indented by two spaces, which is what every document of this corpus
//! writes. The assumption is safe because it is tested rather than trusted.
//! [`splice`] parses its own result and looks for the half it wrote. A file
//! whose shape the splice guessed wrong fails that read, and the run refuses
//! with [`crate::Refusal::ReciprocalUnwritable`] having written nothing.
//!
//! # Nothing is written until everything can be
//!
//! [`apply`] composes every byte of every file first, and writes only when each
//! one is composed. A run that failed halfway would leave an edge with one half
//! on the tree, which is the state the reciprocity rule exists to report and
//! the last state a scaffolder should produce.

use crate::{Half, Plan, Refusal};
use std::path::Path;

/// The new document, whole.
pub fn render(plan: &Plan) -> String {
    let mut out = String::from("---\n");
    if let Some(minting) = &plan.minting {
        out.push_str(&format!("id: {}\n", minting.id));
    }
    for field in &plan.fields {
        match field.quoted {
            true => out.push_str(&format!("{}: {}\n", field.key, quoted(&field.value))),
            false => out.push_str(&format!("{}: {}\n", field.key, field.value)),
        }
    }
    if !plan.edges.is_empty() {
        out.push_str("relations:\n");
        for edge in &plan.edges {
            out.push_str(&format!("  {}:\n    - {}\n", edge.relation, edge.target));
        }
    }
    out.push_str("---\n\n");
    out.push_str(&format!("# {}\n", plan.title));
    for section in &plan.sections {
        out.push_str(&format!(
            "\n## {}\n\nTODO write this section.\n",
            section.heading
        ));
    }
    out
}

/// A scalar as a double-quoted YAML string.
///
/// Everything that is not a date, an integer or a value from a closed set goes
/// through it. A title with a colon in it is the ordinary case, and a payload
/// that does not load is a payload the next command refuses against the wrong
/// file.
fn quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// One file this run will write, composed and not yet on disk.
pub struct Composed {
    pub path: String,
    pub text: String,
    /// Whether the file is new, which is what a report says and what an
    /// overwrite guard reads.
    pub created: bool,
}

/// Compose every file, and write none.
///
/// The caller writes them, so a dry run and a real one differ by one loop
/// rather than by a second composer.
pub fn compose(root: &Path, plan: &Plan) -> Result<Vec<Composed>, Refusal> {
    let mut composed = vec![Composed {
        path: plan.path.clone(),
        text: render(plan),
        created: true,
    }];

    for edge in &plan.edges {
        let Some(half) = &edge.reciprocal else {
            continue;
        };
        let existing = std::fs::read_to_string(root.join(&half.path)).map_err(|error| {
            Refusal::ReciprocalUnwritable {
                path: half.path.clone(),
                why: error.to_string(),
            }
        })?;
        // A second edge into the same document reads the text this run already
        // composed, so two halves into one file both land.
        let source = composed
            .iter()
            .find(|file| file.path == half.path)
            .map(|file| file.text.clone())
            .unwrap_or(existing);
        let spliced = splice(&source, half)?;
        match composed.iter_mut().find(|file| file.path == half.path) {
            Some(file) => file.text = spliced,
            None => composed.push(Composed {
                path: half.path.clone(),
                text: spliced,
                created: false,
            }),
        }
    }
    Ok(composed)
}

/// Write what [`compose`] produced.
pub fn apply(root: &Path, composed: &[Composed]) -> Result<(), Refusal> {
    for file in composed {
        let path = root.join(&file.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| Refusal::ReciprocalUnwritable {
                path: file.path.clone(),
                why: error.to_string(),
            })?;
        }
        std::fs::write(&path, &file.text).map_err(|error| Refusal::ReciprocalUnwritable {
            path: file.path.clone(),
            why: error.to_string(),
        })?;
    }
    Ok(())
}

/// Put one edge half into a document's front matter, and read the result back.
pub fn splice(source: &str, half: &Half) -> Result<String, Refusal> {
    let refuse = |why: &str| Refusal::ReciprocalUnwritable {
        path: half.path.clone(),
        why: why.to_string(),
    };

    let mut lines: Vec<String> = source.lines().map(str::to_string).collect();
    if lines.first().map(String::as_str) != Some("---") {
        return Err(refuse("the file opens with no front-matter block"));
    }
    let close = lines
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, line)| line.as_str() == "---")
        .map(|(position, _)| position)
        .ok_or_else(|| refuse("the front-matter block is never closed"))?;

    let entry = format!("    - {}", half.id);
    match block_of("relations:", &lines, 1, close) {
        None => {
            lines.splice(
                close..close,
                [
                    "relations:".to_string(),
                    format!("  {}:", half.relation),
                    entry,
                ],
            );
        }
        Some((start, end)) => {
            let key = format!("  {}:", half.relation);
            match block_of(&key, &lines, start + 1, end) {
                None => {
                    lines.splice(end..end, [key, entry]);
                }
                Some((_, items_end)) => {
                    lines.splice(items_end..items_end, [entry]);
                }
            }
        }
    }

    let mut spliced = lines.join("\n");
    if source.ends_with('\n') {
        spliced.push('\n');
    }

    // The read back. The splice guessed an indentation, and this is where the
    // guess is tested rather than trusted.
    let parsed = headwater_doc::parse(&spliced)
        .map_err(|errors| refuse(&format!("{} parse errors", errors.len())))?;
    let written = parsed
        .facets
        .get("relations")
        .and_then(|block| block.value.as_map())
        .and_then(|block| block.get(&half.relation))
        .and_then(|targets| targets.value.as_seq())
        .map(|targets| {
            targets
                .iter()
                .filter_map(|target| target.value.as_scalar())
                .any(|target| target.text == half.id)
        })
        .unwrap_or(false);
    match written {
        true => Ok(spliced),
        false => Err(refuse(
            "the spliced block does not read back as the edge half it wrote",
        )),
    }
}

/// The extent of a block that opens with `header`, between two line numbers.
///
/// The block runs from its header to the first later line indented no further
/// than the header is. `(header line, one past the last line of the block)`.
fn block_of(header: &str, lines: &[String], from: usize, to: usize) -> Option<(usize, usize)> {
    let depth = header.len() - header.trim_start().len();
    let start = (from..to).find(|position| lines[*position] == header)?;
    let mut end = start + 1;
    while end < to {
        let line = &lines[end];
        let indent = line.len() - line.trim_start().len();
        if !line.trim().is_empty() && indent <= depth {
            break;
        }
        end += 1;
    }
    Some((start, end))
}
