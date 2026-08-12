// SPDX-License-Identifier: Apache-2.0
//! Cutting a document into its front-matter block and its body.
//!
//! This is the only part of the parser that reads the source as bytes rather
//! than through a grammar, and it is short on purpose. It finds two fences and
//! reports where everything after them sits in the file. The YAML inside goes to
//! the loader, and the Markdown after it goes to the CommonMark scan.

use crate::error::{ParseError, Reason};
use headwater_yaml::{Origin, Span};

/// A byte-order mark. An editor on Windows writes one, and a split that did not
/// know about it would find no opening fence and report every file in such a
/// corpus as untyped.
const BOM: &str = "\u{feff}";

#[derive(Debug)]
pub struct Split<'a> {
    /// The YAML between the fences, without either of them.
    pub front_matter: &'a str,
    /// Where that YAML sits in the file.
    pub front_matter_origin: Origin,
    /// The whole block, from the first `-` of the opening fence to the end of
    /// the closing one. A finding about a facet that is *missing* has no span of
    /// its own, and this is the span it points at.
    pub block: Span,
    /// Everything after the closing fence line.
    pub body: &'a str,
    /// The file byte offset of the body's first byte.
    pub body_offset: usize,
}

/// Split a document.
///
/// The opening fence is the first line of the file, which is what makes a
/// leading `---` unambiguous. A thematic break later in the body is therefore
/// never mistaken for front matter, and a document that opens with one is a
/// document with no front matter.
pub fn split(source: &str) -> Result<Split<'_>, ParseError> {
    let bom = if source.starts_with(BOM) {
        BOM.len()
    } else {
        0
    };
    let after_bom = &source[bom..];

    let rest = after_bom
        .strip_prefix("---\r\n")
        .or_else(|| after_bom.strip_prefix("---\n"))
        .ok_or_else(|| ParseError::at_start(Reason::NoFrontMatter))?;
    let open_len = source.len() - rest.len();

    // A closing fence is a `---` that starts its own line and has nothing after
    // it. Checking what follows is what keeps a `---` inside the YAML — a
    // nested document marker, or a value that begins with three dashes — from
    // ending the block early.
    let closes_here = |at: usize| {
        rest[at..].starts_with("---")
            && matches!(
                rest[at + 3..].chars().next(),
                None | Some('\n') | Some('\r')
            )
    };
    // The empty block, `---` directly under `---`, has its closing fence at
    // offset zero, where there is no preceding newline to search for.
    let mut cursor = 0usize;
    let close_at = if closes_here(0) {
        0
    } else {
        loop {
            let Some(found) = rest[cursor..].find("\n---") else {
                return Err(ParseError::at_start(Reason::UnterminatedFrontMatter));
            };
            let at = cursor + found + 1;
            if closes_here(at) {
                break at;
            }
            cursor = at + 3;
        }
    };

    let front_matter = &rest[..close_at];
    let after_fence = &rest[close_at + 3..];
    let body = after_fence
        .strip_prefix("\r\n")
        .or_else(|| after_fence.strip_prefix('\n'))
        .unwrap_or(after_fence);

    // `front_matter` ends with the newline that precedes the closing fence, so
    // its newline count *is* its line count. Adding one to that count would
    // shift every body position by a line, which shows up as an editor landing
    // one line short of the finding and looking almost right.
    let lines = crate::lines::Lines::new(source);
    let block = Span::new(
        lines.position(source, bom),
        lines.position(source, open_len + close_at + 3),
    );
    debug_assert_eq!(block.end.line, 1 + front_matter.matches('\n').count() + 1);

    Ok(Split {
        front_matter,
        // Line 1 is the opening fence, so the YAML's line 1 is file line 2.
        front_matter_origin: Origin::new(2, open_len),
        block,
        body,
        body_offset: source.len() - body.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_document_splits() {
        let source = "---\nid: DR-0001\n---\n\n# Title\n";
        let split = split(source).expect("splits");
        assert_eq!(split.front_matter, "id: DR-0001\n");
        assert_eq!(split.body, "\n# Title\n");
        assert_eq!(split.front_matter_origin, Origin::new(2, 4));
        assert_eq!(split.body_offset, 20);
    }

    #[test]
    fn crlf_splits_the_same_way() {
        let source = "---\r\nid: DR-0001\r\n---\r\n# Title\r\n";
        let split = split(source).expect("splits");
        assert_eq!(split.front_matter, "id: DR-0001\r\n");
        assert_eq!(split.body, "# Title\r\n");
    }

    #[test]
    fn a_thematic_break_in_the_body_does_not_reopen_the_block() {
        let source = "---\nid: DR-0001\n---\n\nBefore.\n\n---\n\nAfter.\n";
        let split = split(source).expect("splits");
        assert_eq!(split.front_matter, "id: DR-0001\n");
        assert!(split.body.contains("After."));
    }

    #[test]
    fn a_dash_run_inside_the_yaml_does_not_close_the_block() {
        // A value that begins with three dashes sits at a line start once the
        // key is on its own line, and a naive search for `\n---` ends the block
        // in the middle of the facets.
        let source = "---\nsummary: >\n  ----------\nid: DR-0001\n---\n\nBody.\n";
        let split = split(source).expect("splits");
        assert!(split.front_matter.contains("id: DR-0001"));
    }

    #[test]
    fn a_byte_order_mark_does_not_hide_the_fence() {
        let source = "\u{feff}---\nid: DR-0001\n---\n\nBody.\n";
        let split = split(source).expect("splits");
        assert_eq!(split.front_matter, "id: DR-0001\n");
        assert_eq!(split.front_matter_origin.byte, 7);
        assert_eq!(split.block.start.line, 1);
    }

    #[test]
    fn a_document_with_no_front_matter_is_reported_as_such() {
        let error = split("# Title\n").expect_err("no front matter");
        assert_eq!(error.reason, Reason::NoFrontMatter);
    }

    #[test]
    fn an_unterminated_block_is_not_the_same_as_a_missing_one() {
        let error = split("---\nid: DR-0001\n").expect_err("unterminated");
        assert_eq!(error.reason, Reason::UnterminatedFrontMatter);
    }

    #[test]
    fn an_empty_block_splits_to_an_empty_source() {
        let split = split("---\n---\n\nBody.\n").expect("splits");
        assert_eq!(split.front_matter, "");
        assert_eq!(split.body, "\nBody.\n");
    }
}
