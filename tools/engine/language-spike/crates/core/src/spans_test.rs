// SPDX-License-Identifier: Apache-2.0
//! Item 1: does a source position survive from the YAML parser to a rendered
//! finding, in *file* coordinates?
//!
//! These are unit tests rather than integration tests on purpose. Reaching
//! `DocumentView::new` from `tests/` would need a public constructor or a
//! `testing` feature, and either one widens exactly the surface that item 2
//! claims is closed. The compile-fail tests in `tests/ui/` are the external
//! consumer, and they run against the unmodified public API.
//!
//! Every expected line number below was read off the fixture by hand.

use crate::check::DocumentCheck;
use crate::checks::{FacetRequired, HeadingRequired, StatusEnum};
use crate::corpus::parse_document;
use crate::frontmatter::Value;
use crate::model::Document;
use crate::view::DocumentView;

const DOC: &str = "\
---
id: DR-0001
kind: decision
status: currrent
owner: platform
tags:
  - governance
  - lifecycle
nested:
  inner: value
empty_facet:
---

# Decision DR-0001

## Context

Some prose here.

## References

See [the standard](std-0001.md).
";
// 1  ---
// 2  id: DR-0001
// 3  kind: decision
// 4  status: currrent
// 5  owner: platform
// 6  tags:
// 7    - governance
// 8    - lifecycle
// 9  nested:
// 10   inner: value
// 11 empty_facet:
// 12 ---
// 13 (blank)
// 14 # Decision DR-0001
// 15 (blank)
// 16 ## Context
// 17 (blank)
// 18 Some prose here.
// 19 (blank)
// 20 ## References
// 21 (blank)
// 22 See [the standard](std-0001.md).

fn doc() -> Document {
    parse_document("docs/decisions/dr-0001.md", DOC).expect("fixture parses")
}

#[test]
fn every_top_level_key_reports_its_own_line() {
    let d = doc();
    let expected = [
        ("id", 2usize),
        ("kind", 3),
        ("status", 4),
        ("owner", 5),
        ("tags", 6),
        ("nested", 9),
        ("empty_facet", 11),
    ];
    for (key, line) in expected {
        let span = d
            .front
            .key_span(key)
            .unwrap_or_else(|| panic!("no span for `{key}`"));
        assert_eq!(span.start.line, line, "line for key `{key}`");
        assert_eq!(span.start.col, 1, "top-level keys start at column 1");
    }
}

#[test]
fn sequence_elements_keep_their_own_lines() {
    let d = doc();
    let tags = d.front.scalar_seq("tags");
    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0].value, "governance");
    assert_eq!(tags[0].span.start.line, 7);
    assert_eq!(tags[1].value, "lifecycle");
    assert_eq!(tags[1].span.start.line, 8);
}

#[test]
fn nested_keys_keep_their_own_lines() {
    let d = doc();
    let Value::Map(entries) = &d.front.get("nested").unwrap().value else {
        panic!("expected a mapping");
    };
    assert_eq!(entries[0].key.value, "inner");
    assert_eq!(entries[0].key.span.start.line, 10);
    assert_eq!(entries[0].key.span.start.col, 3);
}

#[test]
fn body_headings_are_in_file_coordinates() {
    let d = doc();
    let context = d
        .body
        .headings
        .iter()
        .find(|h| h.value == "Context")
        .expect("Context heading");
    assert_eq!(context.span.start.line, 16);
}

#[test]
fn body_links_are_in_file_coordinates() {
    let d = doc();
    assert_eq!(d.body.links.len(), 1);
    assert_eq!(d.body.links[0].value, "std-0001.md");
    assert_eq!(d.body.links[0].span.start.line, 22);
}

// ---- the half that matters: positions reaching rendered output ----

#[test]
fn a_finding_about_a_present_key_renders_that_keys_line() {
    let d = doc();
    let view = DocumentView::new(&d);
    let found = StatusEnum {
        allowed: &["draft", "current", "superseded", "retired"],
    }
    .evaluate(&view);
    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].render(),
        "docs/decisions/dr-0001.md:4:1: [error] status_enum: `currrent` is not a declared status"
    );
    assert_eq!(found[0].fix.as_ref().unwrap().replacement, "current");
}

#[test]
fn a_finding_about_an_empty_key_renders_that_keys_line() {
    let d = doc();
    let view = DocumentView::new(&d);
    let found = FacetRequired {
        facet: "empty_facet",
    }
    .evaluate(&view);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].span.start.line, 11);
    assert!(found[0].message.contains("is empty"));
}

#[test]
fn a_finding_about_an_absent_key_anchors_to_the_block_not_line_zero() {
    let d = doc();
    let view = DocumentView::new(&d);
    let found = FacetRequired {
        facet: "last_verified",
    }
    .evaluate(&view);
    assert_eq!(found.len(), 1);
    // The failure this guards: a missing key has no span of its own, and a
    // naive implementation reports line 0, which no editor can jump to.
    assert_eq!(found[0].span.start.line, 1);
    assert!(found[0].span.end.line >= 12);
    assert!(found[0]
        .render()
        .starts_with("docs/decisions/dr-0001.md:1:1:"));
}

#[test]
fn a_body_finding_renders_a_body_line() {
    let d = doc();
    let view = DocumentView::new(&d);
    let found = HeadingRequired {
        heading: "Consequences",
    }
    .evaluate(&view);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].span.start.line, 14);
}

#[test]
fn crlf_input_does_not_shift_line_numbers() {
    let crlf = DOC.replace('\n', "\r\n");
    let d = parse_document("docs/decisions/dr-0001.md", &crlf).expect("crlf fixture parses");
    assert_eq!(d.front.key_span("status").unwrap().start.line, 4);
}
