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
//! # Nothing is written until everything can be, and this is where that became
//! true of the writing rather than of the composing
//!
//! [`compose`] composes every byte of every file before any of it reaches disk.
//! That was the whole of the claim for a long time, and it was a claim about
//! the phase that cannot fail: underneath it, [`apply`] was a loop of
//! `std::fs::write` that stopped on the first error with every file before it
//! written. The run it left behind was the exact state the reciprocity rule
//! exists to report — a new document declaring one half of an edge, with the
//! other half nowhere — and the corpus then failed its own commit gate over a
//! document the author never chose to keep.
//!
//! [`apply`] now goes through [`crate::tree::Reserved`], in the order that
//! decides everything:
//!
//! 1. [`crate::tree::Reserved::over`] opens every document this run will
//!    **edit**. No byte moves, so the failure that actually happens — a
//!    reciprocal end that is read-only — refuses here with the tree exactly as
//!    it was and nothing to undo.
//! 2. [`crate::tree::Reserved::making`] creates the one document this run
//!    **makes**, through `create_new`, so *this run made it* is an answer the
//!    kernel returned.
//! 3. [`crate::tree::Reserved::commit`] writes, reads every path back, and on a
//!    failure puts every edited file back and unlinks the created one.
//!
//! **A crash between two writes is still uncovered**, as
//! [`crate::tree`]'s own comment says and as no user-space scheme covers
//! without a journal. What is covered is the failure a run is still there to
//! undo.

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
///
/// # Exactly one entry is new, and it is the first
///
/// The vector opens with the document this run makes, and every entry appended
/// after it is a reciprocal end this function `read_to_string`'d off the tree.
/// A reciprocal can never be the new document either: [`crate::propose`] refuses
/// with [`Refusal::TargetUnresolved`] unless the far end already carries the
/// identifier the edge names, and with [`Refusal::PathTaken`] unless the new
/// path is free. So the invariant [`apply`] reads is established here, and
/// [`apply`] refuses rather than picking when it does not hold.
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

/// Write what [`compose`] produced: all of it, or none of it.
///
/// The documents this run edits are reserved first and the document it makes is
/// created after them, which is the ordering [`crate::tree`] argues for at
/// length. A failure at the reservation leaves the tree untouched, and a failure
/// after it puts every edited file back and unlinks the created one.
///
/// # `created` routes, and never licenses
///
/// The flag says which entry [`compose`] planned to make, and routing on it is
/// safe because a wrong answer is caught rather than acted on: an entry marked
/// new whose path is taken fails `create_new` with `AlreadyExists` and refuses,
/// and an entry marked old that is not there fails the reservation's open. What
/// the flag never does is license the unlink. The undo removes the path
/// `create_new` returned `Ok` for, so the licence is a syscall's answer and not
/// a bool three functions upstream.
pub fn apply(root: &Path, composed: &[Composed]) -> Result<(), Refusal> {
    let new: Vec<&Composed> = composed.iter().filter(|file| file.created).collect();
    let [new] = new[..] else {
        return Err(Refusal::NotOneDocument { created: new.len() });
    };
    let editing = composed
        .iter()
        .filter(|file| !file.created)
        .map(|file| crate::tree::Composed {
            path: file.path.clone(),
            text: file.text.clone(),
        })
        .collect();

    crate::tree::Reserved::over(root, editing)
        .map_err(|unopened| Refusal::TargetUnopened {
            path: unopened.path,
            why: unopened.why,
        })?
        .making(root, &new.path, new.text.clone())
        .map_err(|unopened| Refusal::DocumentUncreated {
            path: unopened.path,
            why: unopened.why,
        })?
        .commit()
        .map_err(|halted| Refusal::WriteHalted {
            path: halted.path().to_string(),
            report: halted.to_string(),
        })?;
    Ok(())
}

/// Put one edge half into a document's front matter, and read the result back.
///
/// # Replace when the target is already there, append otherwise
///
/// A relation's item list holds at most one entry per target — spec 2 states
/// that no attribute name collides with `to`, and the importer's own
/// `RepeatedLink` refusal holds it at plan time, so two entries for one
/// target is not a shape this system produces on purpose. When `half.relation`
/// already carries an entry whose identity (the mapping form's `to`, or the
/// bare form's own scalar) matches `half.id`, this function replaces that
/// entry's lines in place rather than appending a second one beside it. Every
/// attribute value, `half`'s own included, is ignored for the match — that is
/// exactly the field an update is for, most visibly `verified_revision` when
/// a re-import runs after the snapshot's revision moved. Only when no entry
/// matches does the function fall back to the append it always did.
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

    // An entry with no attribute is a bare target reference, and one with
    // attributes is the mapping form that spec 2 declares: `to`, and the
    // declared attributes beside it. The two forms are one list, so a relation
    // that already holds one takes the other.
    let entry: Vec<String> = match half.attributes.is_empty() {
        true => vec![format!("    - {}", half.id)],
        false => {
            let mut lines = vec![format!("    - to: {}", half.id)];
            for (name, value) in &half.attributes {
                lines.push(format!("      {name}: {value}"));
            }
            lines
        }
    };
    match block_of("relations:", &lines, 1, close) {
        None => {
            let mut block = vec!["relations:".to_string(), format!("  {}:", half.relation)];
            block.extend(entry);
            lines.splice(close..close, block);
        }
        Some((start, end)) => {
            let key = format!("  {}:", half.relation);
            match block_of(&key, &lines, start + 1, end) {
                None => {
                    let mut block = vec![key];
                    block.extend(entry);
                    lines.splice(end..end, block);
                }
                Some((_, items_end)) => match existing_entry(source, half) {
                    Some((start, end)) => {
                        lines.splice(start..end, entry);
                    }
                    None => {
                        lines.splice(items_end..items_end, entry);
                    }
                },
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
        .map(|targets| targets.iter().any(|target| holds(&target.value, half)))
        .unwrap_or(false);
    match written {
        true => Ok(spliced),
        false => Err(refuse(
            "the spliced block does not read back as the edge half it wrote",
        )),
    }
}

/// Whether one entry of a relation's target list is the half that was written.
///
/// Both forms, and every attribute. An entry that reads back with the right
/// target and a lost attribute is a different edge from the one composed, and
/// the caller of the splice is the one that recorded what the attribute is for.
fn holds(value: &headwater_yaml::Value, half: &Half) -> bool {
    if half.attributes.is_empty() {
        return value
            .as_scalar()
            .map(|scalar| scalar.text == half.id)
            .unwrap_or(false);
    }
    let Some(map) = value.as_map() else {
        return false;
    };
    let text = |key: &str| {
        map.get(key)
            .and_then(|node| node.value.as_scalar())
            .map(|scalar| scalar.text.as_str())
    };
    text("to") == Some(half.id.as_str())
        && half
            .attributes
            .iter()
            .all(|(name, expected)| text(name) == Some(expected.as_str()))
}

/// The 0-indexed, half-open line range of the entry already in `half.relation`'s
/// list that targets `half.id`, read from `source` as it stood before this
/// splice's own surgery.
///
/// `None` covers three cases the caller treats alike: the relation holds no
/// such entry, the document declares no such relation at all, and `source`
/// fails to reparse — which should not happen, since it is always
/// previously-valid text, but a caller that meets it anyway falls through to
/// the append this function performed unconditionally before this match
/// existed.
///
/// # The span arithmetic, and why it differs by form
///
/// A bare-form entry's span is inclusive at the line level: `start.line ==
/// end.line` for a one-line item. A mapping-form entry's span is exclusive
/// past the end: `end.line` is the file line *after* the item's last content
/// line, which is what lets a multi-line item's range compose with the next
/// one without overlap. Both were measured against the real parser rather
/// than assumed. Converting 1-indexed span lines to the 0-indexed half-open
/// range [`Vec::splice`] wants, and unifying the two forms, takes one line of
/// arithmetic: the end is `end.line - 1`, floored at one past the start so a
/// bare entry's inclusive single line still yields a range that covers it.
fn existing_entry(source: &str, half: &Half) -> Option<(usize, usize)> {
    let parsed = headwater_doc::parse(source).ok()?;
    let items = parsed
        .facets
        .get("relations")?
        .value
        .as_map()?
        .get(&half.relation)?
        .value
        .as_seq()?;
    let item = items
        .iter()
        .find(|item| identity(&item.value) == Some(half.id.as_str()))?;
    let start = item.span.start.line - 1;
    let end = (item.span.end.line - 1).max(start + 1);
    Some((start, end))
}

/// The target identity an entry names: the mapping form's `to`, or the bare
/// form's own scalar. Every attribute value is ignored on purpose — spec 2's
/// "no attribute name collides with `to`" makes this injective over what
/// legitimately varies between two proposals for the same target, and
/// `verified_revision` is exactly the attribute a re-import at a moved
/// revision means to overwrite rather than match on.
fn identity(value: &headwater_yaml::Value) -> Option<&str> {
    match value.as_map() {
        Some(map) => map
            .get("to")
            .and_then(|node| node.value.as_scalar())
            .map(|scalar| scalar.text.as_str()),
        None => value.as_scalar().map(|scalar| scalar.text.as_str()),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn half(id: &str, attributes: &[(&str, &str)]) -> Half {
        Half {
            relation: "audited_by".to_string(),
            path: "corpus/spec/02-the-second-part.md".to_string(),
            id: id.to_string(),
            attributes: attributes
                .iter()
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect(),
        }
    }

    /// A target already present is replaced, not duplicated — growing from
    /// the bare form to the mapping form because the new half carries an
    /// attribute the old entry did not.
    #[test]
    fn a_present_target_is_replaced_not_duplicated_when_it_grows() {
        let source = "\
---
id: DR-FIX-0009
relations:
  audited_by:
    - 12345
---

# A document
";
        let spliced = splice(source, &half("12345", &[("verified_revision", "8")]))
            .expect("a present bare target replaces");
        assert_eq!(
            spliced.matches("12345").count(),
            1,
            "one entry, not two:\n{spliced}"
        );
        assert!(spliced.contains("    - to: 12345\n      verified_revision: 8"));
        assert!(!spliced.contains("    - 12345\n"));
    }

    /// The same direction, shrinking: a mapping-form entry with two
    /// attributes is replaced by one with a single attribute, and the line
    /// the old entry no longer needs does not survive the splice.
    #[test]
    fn a_present_target_is_replaced_not_duplicated_when_it_shrinks() {
        let source = "\
---
id: DR-FIX-0010
relations:
  audited_by:
    - to: 12345
      verified_revision: 7
      note: something
---

# A document
";
        let spliced = splice(source, &half("12345", &[("verified_revision", "8")]))
            .expect("a present mapping target replaces");
        assert_eq!(
            spliced.matches("12345").count(),
            1,
            "one entry, not two:\n{spliced}"
        );
        assert!(spliced.contains("    - to: 12345\n      verified_revision: 8"));
        assert!(!spliced.contains("verified_revision: 7"));
        assert!(!spliced.contains("note: something"));
    }

    /// The decisive shape for #384: same attribute name, a revision that
    /// moved. The line count does not change, and the value still updates in
    /// place rather than appending a second entry.
    #[test]
    fn a_present_target_is_replaced_when_only_an_attribute_value_moves() {
        let source = "\
---
id: DR-FIX-0011
relations:
  audited_by:
    - to: 12345
      verified_revision: 7
---

# A document
";
        let spliced = splice(source, &half("12345", &[("verified_revision", "8")]))
            .expect("a present mapping target replaces");
        assert_eq!(
            spliced.matches("12345").count(),
            1,
            "one entry, not two:\n{spliced}"
        );
        assert!(spliced.contains("    - to: 12345\n      verified_revision: 8"));
        assert!(!spliced.contains("verified_revision: 7"));
    }

    /// A target not already present is still appended, on a relation the
    /// document declares with no items under it yet.
    #[test]
    fn an_absent_target_is_appended_on_an_empty_relation() {
        let source = "\
---
id: DR-FIX-0012
relations:
  audited_by:
---

# A document
";
        let spliced =
            splice(source, &half("12345", &[])).expect("an empty relation still takes an entry");
        assert!(spliced.contains("  audited_by:\n    - 12345"));
    }

    /// A target not already present is still appended when the document has
    /// no `audited_by` relation at all, only other relations populated.
    #[test]
    fn an_absent_target_is_appended_on_a_document_with_other_relations() {
        let source = "\
---
id: DR-FIX-0013
relations:
  superseded_by:
    - DR-FIX-0001
---

# A document
";
        let spliced = splice(source, &half("12345", &[]))
            .expect("a document with other relations still takes a new one");
        assert!(spliced.contains("  audited_by:\n    - 12345"));
        assert!(spliced.contains("  superseded_by:\n    - DR-FIX-0001"));
    }

    /// A target not already present is still appended alongside other
    /// targets already declared under the same relation.
    #[test]
    fn an_absent_target_is_appended_beside_other_targets_of_the_same_relation() {
        let source = "\
---
id: DR-FIX-0014
relations:
  audited_by:
    - 99999
---

# A document
";
        let spliced = splice(source, &half("12345", &[("verified_revision", "8")]))
            .expect("a new target appends beside an existing one");
        assert!(spliced.contains("    - 99999"));
        assert!(spliced.contains("    - to: 12345\n      verified_revision: 8"));
    }
}
