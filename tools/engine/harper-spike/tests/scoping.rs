// SPDX-License-Identifier: Apache-2.0
//! The decisive fixture of #1013, for HW-DR-0041 (Q41) mismatch 3.
//!
//! One file holds the doubled word "the the" six times: in an inline code
//! span, an inline quotation, a block quote, a fenced code block, an HTML block
//! and one sentence of authored prose. Only the last is this author's prose.
//! Fed the author-owned text, Harper must report it once and anchor it at the
//! file bytes of that occurrence. Fed the raw file through its own Markdown
//! parser, it reports more than one.

const FIXTURE: &str = include_str!("fixtures/scoping.md");

#[test]
fn authored_mode_reports_the_authored_occurrence_once_at_its_file_bytes() {
    let mut linter = harper_spike::linter();
    let rows = harper_spike::authored("scoping.md", FIXTURE, &mut linter);
    assert_eq!(rows.len(), 1, "{rows:#?}");

    let needle = "wrote the the fox here";
    let at = FIXTURE.find(needle).expect("the authored sentence") + "wrote ".len();
    let (start, end) = rows[0].range.expect("a file range");
    assert_eq!(&FIXTURE[start..end], rows[0].expect);
    assert!(
        (start..end).contains(&at) || (start..end).contains(&(at + 4)),
        "range {start}..{end} is not the authored occurrence at {at}: {:?}",
        &FIXTURE[start..end]
    );
    assert!(FIXTURE[start..end].contains("the"), "{rows:#?}");
}

#[test]
fn raw_mode_reports_more_than_one() {
    let mut linter = harper_spike::linter();
    let rows = harper_spike::raw("scoping.md", FIXTURE, &mut linter);
    let doubled = rows
        .iter()
        .filter(|row| row.flagged.to_lowercase().contains("the the"))
        .count();
    assert!(doubled > 1, "{rows:#?}");
}
