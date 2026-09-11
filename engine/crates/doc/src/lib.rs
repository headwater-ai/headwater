// SPDX-License-Identifier: Apache-2.0
//! Reading a document: spanned front matter, and a CommonMark body scan.
//!
//! A document is a Markdown file that opens with a `---` block of facets.
//! Nothing here decides what a facet *means*. Kind resolution, required facets
//! and the relations that become edges are all the taxonomy's business, and the
//! taxonomy is a later milestone. This crate answers one question: what did the
//! author write, and where.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots) names
//! the parser a correctness root, and the failure it names is a silent pass
//! rather than a crash: a mis-parsed heading lets a section contract hold with
//! no finding anywhere. That is why the body is a real CommonMark parse rather
//! than a scan by regular expression, and why every span is a fixture.
//!
//! # Two sources, one dialect
//!
//! Front matter is YAML, so it loads through [`headwater_yaml`] rather than
//! through a second tree builder. Every Q2 ruling holds here for the reason it
//! was made, with a single exception that the loader takes as a parameter:
//! empty front matter is an untyped document rather than an unreadable file.
//! See [`headwater_yaml::Options`].
//!
//! # Example
//!
//! ```
//! let doc = headwater_doc::parse("---\nkind: decision\n---\n\n# Title\n").unwrap();
//! assert_eq!(doc.facets.get("kind").unwrap().as_scalar().unwrap().text, "decision");
//! assert_eq!(doc.body.headings().next().unwrap().text(), "Title");
//! ```

pub mod body;
pub mod error;
pub mod lines;
pub mod sentences;
pub mod split;

pub use body::{Block, BlockKind, Body, Link, LinkForm, Ownership, Run};
pub use error::{ParseError, Reason};
pub use headwater_yaml::{Mapping, Position, Span, Spanned, Value};
pub use sentences::Sentence;

/// A parsed document.
#[derive(Clone, Debug)]
pub struct Document {
    /// The front matter, as declared. A facet the author did not write is
    /// absent here rather than defaulted, because a default is a taxonomy
    /// decision and this crate has no taxonomy.
    pub facets: Mapping,
    /// The whole front-matter block, both fences included. This is the span a
    /// finding about a *missing* facet points at, since a missing key has no
    /// span of its own.
    pub block: Span,
    pub body: Body,
}

/// The front-matter block that states what stands behind a document.
///
/// [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed):
/// "The provenance block is the exception, and its shape belongs to the
/// engine." So the two names below are this crate's, like the split of front
/// matter from body, and no taxonomy declares either.
pub const PROVENANCE: &str = "provenance";

/// The member of that block that states what stands behind the document.
pub const WARRANT: &str = "warrant";

/// The member of that block that states what kind of evidence the document
/// rests on. Spec 3's three honest states are `evidenced`, `reconstructed` and
/// `unevidenced`, and the block that carries them is the engine's on the same
/// terms as the warrant beside it.
pub const EVIDENCE_BASIS: &str = "evidence_basis";

/// What stands behind a document, from the provenance block of its front
/// matter.
///
/// One reader, because two would be two answers to a question that decides
/// whether a pointer states a warrant out loud and whether a corpus has a
/// population to promote from.
pub fn warrant(facets: &Mapping) -> Option<&str> {
    member(facets, WARRANT)
}

/// What kind of evidence a document claims to rest on, from the same block.
///
/// One reader, for the reason [`warrant`] has one. The value comes back as
/// written, including the unfilled template placeholder that three bundles of
/// the base package ship: a document that still carries
/// `"{{evidenced | reconstructed | unevidenced}}"` has chosen nothing, and a
/// reader that split the string would read a choice into it.
pub fn evidence_basis(facets: &Mapping) -> Option<&str> {
    member(facets, EVIDENCE_BASIS)
}

/// One scalar member of the provenance block, as written.
fn member<'a>(facets: &'a Mapping, name: &str) -> Option<&'a str> {
    facets
        .get(PROVENANCE)
        .and_then(|node| node.value.as_map())
        .and_then(|map| map.get(name))
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.as_str())
}

/// Parse a document.
///
/// The errors come back in source order, and the list is empty only when the
/// result is `Ok`. A document that cannot be split reports one error, because
/// there is nothing after that point to say anything about.
pub fn parse(source: &str) -> Result<Document, Vec<ParseError>> {
    let split = split::split(source).map_err(|error| vec![error])?;

    let loaded = headwater_yaml::load_with(
        split.front_matter,
        headwater_yaml::Options::front_matter(split.front_matter_origin),
    );
    let root = match loaded {
        Ok(root) => root,
        Err(errors) => return Err(errors.into_iter().map(ParseError::from).collect()),
    };

    let facets = match root.value {
        Value::Map(map) => map,
        // The loader deliberately does not require a mapping: it knows the
        // dialect and nothing about the meaning. A facet has a name, so a
        // document does require one, and the requirement lives here.
        other => {
            return Err(vec![ParseError::new(
                Reason::NotAMapping(other.kind_name()),
                root.span,
            )])
        }
    };

    Ok(Document {
        facets,
        block: split.block,
        body: body::scan(source, split.body, split.body_offset),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "\
---
id: HW-SPEC-check-layer
sequence: 12
---

# What a check is

A check reads one [scope](02-taxonomy-model.md#scope) — and no more.

> Boundary objects are both plastic enough to adapt to local needs.

    # not a heading

| Crate | Milestone |
|---|---|
| `headwater-yaml` | M1 |
";

    #[test]
    fn front_matter_spans_are_file_coordinates() {
        let doc = parse(DOC).expect("parses");
        let span = doc.facets.key_span("sequence").expect("the sequence key");
        assert_eq!((span.start.line, span.start.col), (3, 1));
        assert_eq!(&DOC[span.start.offset..span.end.offset], "sequence");
        assert_eq!(doc.block.start.line, 1);
        assert_eq!(doc.block.end.line, 4);
    }

    #[test]
    fn a_heading_inside_an_indented_code_block_is_not_a_heading() {
        // The whole argument for a parse over a regular expression, in one
        // fixture. `# not a heading` is indented four spaces, so it is code.
        let doc = parse(DOC).expect("parses");
        let headings: Vec<String> = doc.body.headings().map(|h| h.text()).collect();
        assert_eq!(headings, vec!["What a check is"]);
        assert_eq!(doc.body.headings().next().unwrap().level(), Some(1));
    }

    #[test]
    fn a_block_quote_is_another_authors_prose() {
        let doc = parse(DOC).expect("parses");
        let quoted: Vec<String> = doc
            .body
            .blocks
            .iter()
            .filter(|b| b.quote_depth > 0)
            .map(|b| b.text())
            .collect();
        assert_eq!(quoted.len(), 1);
        assert!(quoted[0].starts_with("Boundary objects"));
        assert!(doc.body.authored().all(|b| b.quote_depth == 0));
    }

    #[test]
    fn a_link_keeps_its_destination_its_text_and_its_span() {
        let doc = parse(DOC).expect("parses");
        assert_eq!(doc.body.links.len(), 1);
        let link = &doc.body.links[0];
        assert_eq!(link.destination, "02-taxonomy-model.md#scope");
        assert_eq!(link.text, "scope");
        assert_eq!(link.form, LinkForm::Inline);
        assert_eq!(
            &DOC[link.span.start.offset..link.span.end.offset],
            "[scope](02-taxonomy-model.md#scope)"
        );
    }

    #[test]
    fn a_table_is_cells_rather_than_a_paragraph_of_pipes() {
        let doc = parse(DOC).expect("parses");
        let cells: Vec<String> = doc
            .body
            .blocks
            .iter()
            .filter(|b| b.kind == BlockKind::TableCell)
            .map(|b| b.text())
            .collect();
        assert_eq!(cells, vec!["Crate", "Milestone", "headwater-yaml", "M1"]);
    }

    #[test]
    fn an_inline_code_span_is_not_prose() {
        let doc = parse(DOC).expect("parses");
        let code: Vec<&str> = doc
            .body
            .blocks
            .iter()
            .flat_map(|b| &b.runs)
            .filter(|r| r.ownership == Ownership::Code)
            .map(|r| r.text.as_str())
            .collect();
        assert!(code.contains(&"headwater-yaml"), "{code:?}");
        assert!(code.contains(&"# not a heading\n"), "{code:?}");
    }

    #[test]
    fn a_soft_break_reads_as_a_space() {
        let doc = parse("---\nid: x\n---\n\nOne line\nand the next.\n").expect("parses");
        assert_eq!(doc.body.blocks[0].text(), "One line and the next.");
    }

    #[test]
    fn every_span_slices_the_source_it_came_from() {
        // The property that no single fixture proves and every consumer
        // assumes. A byte offset that is really a character index passes every
        // ASCII test and cuts an em dash in half here.
        let doc = parse(DOC).expect("parses");
        for block in &doc.body.blocks {
            assert!(
                block.span.slice(DOC).is_some(),
                "block span does not lie in the source: {:?}",
                block.span
            );
            for run in &block.runs {
                assert!(
                    run.span.slice(DOC).is_some(),
                    "run span does not lie in the source: {:?}",
                    run.span
                );
            }
        }
    }

    #[test]
    fn empty_front_matter_is_a_document_with_no_facets() {
        let doc = parse("---\n---\n\n# Title\n").expect("parses");
        assert!(doc.facets.is_empty());
        assert_eq!(doc.body.headings().count(), 1);
    }

    #[test]
    fn front_matter_that_is_not_a_mapping_is_refused() {
        let errors = parse("---\n- one\n- two\n---\n\nBody.\n").expect_err("refused");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].reason, Reason::NotAMapping("a sequence"));
    }

    #[test]
    fn a_duplicate_facet_is_refused_at_the_line_that_repeats_it() {
        let errors = parse("---\nid: a\nkind: x\nid: b\n---\n\nBody.\n").expect_err("refused");
        assert_eq!(errors[0].span.start.line, 4);
        assert!(
            errors[0].to_string().contains("first declared at 2:1"),
            "{}",
            errors[0]
        );
    }
}
