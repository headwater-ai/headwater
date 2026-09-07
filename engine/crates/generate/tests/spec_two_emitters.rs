// SPDX-License-Identifier: Apache-2.0
//! Spec 2's staged-emitter block, held against [`headwater_generate::export`].
//!
//! # Why this file exists
//!
//! [R1](https://github.com/headwater-ai/headwater/issues/421)'s bar is that no
//! specification part states as built an `export` target or a projection kind
//! that this engine reports as unbuilt. The bar has two halves. The
//! projection-kind half is held by `spec_six_projections.rs`. The export-target
//! half was corrected by hand in `docs/spec/02-taxonomy-model.md` and nothing
//! read it afterwards, so a reword of the refusal, an emptied fenced block, or
//! the day someone builds `skos` all leave the paragraph false with the suite
//! green. This file is the missing half.
//!
//! # What the existing case table does not hold
//!
//! `fixtures.rs`, in `an_unbuilt_emitter_refuses_and_says_what_it_waits_on`,
//! runs every `Emitter::ALL` through `export::emit` and asserts of an unbuilt
//! one only that its reason contains the emitter name and the word `consumer`.
//! That is a substring assertion over the exact message spec 2 quotes, so every
//! reword of the message passes it. This file compares the block byte for byte
//! instead, which is why the two are not the same test.
//!
//! # The three directions
//!
//! First, the emitter the block names must be **unbuilt**. That is the
//! direction that fires on the day `skos` is built: the paragraph then says
//! "staged and unbuilt" about a target that ships, and this test forces the
//! prose to move with the code.
//!
//! Second, the block's output lines must be **exactly** what the refusal
//! carries. The reason is read out of the engine rather than copied here, so
//! there is no third copy of the sentence to drift.
//!
//! Third, in reverse, every `--format <name>` occurrence anywhere in spec 2
//! must name a built emitter or lie inside this one block. Without it an
//! emptied block passes, and a second `--format shacl` example added elsewhere
//! later passes too.
//!
//! # What this does not hold
//!
//! That `export::emit` refuses through `Refusal::NotBuilt` at all: that is
//! `fixtures.rs`'s case table, which runs the real function over a real
//! surface. This file holds the wording that table lets through, and needs no
//! corpus to do it.
//!
//! The extractor below is deliberately a copy of the one in
//! `spec_six_projections.rs` rather than a shared helper. Two branches were in
//! flight over these files when it was written, and twenty duplicated lines
//! cost less than the merge they would otherwise collide on.

use std::path::{Path, PathBuf};

use headwater_generate::export::Refusal;
use headwater_generate::Emitter;

/// The heading-free anchor the extractor splits on.
///
/// A sentence rather than a heading, because the section that carries it opens
/// with other fenced blocks (a mapping declaration among them) and anchoring on
/// the heading would read one of those.
const ANCHOR: &str = "A SKOS projection of the resolved taxonomy is staged and unbuilt";

/// The prompt that opens the block's one command line.
const PROMPT: &str = "$ ";

/// The line `headwater export` prints before the reason.
///
/// A hand copy of `engine/crates/cli/src/main.rs`'s refusal arm, which a test
/// of this crate cannot reach: `headwater-generate` does not depend on
/// `headwater-cli`, and the dependency runs the other way. The reason on the
/// line under it is the part that carries the wording, and that part is read
/// out of the engine below rather than copied.
const REFUSAL_HEADER: &str = "headwater: nothing was exported";

/// The indent the CLI puts in front of the reason.
const REASON_INDENT: &str = "  ";

fn spec_two() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../docs/spec/02-taxonomy-model.md")
}

fn spec_two_text() -> String {
    let path = spec_two();
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The staged-emitter block of spec 2, and where it sits in the file.
///
/// Returns the block's lines with the blank lines the fence contributes
/// removed, and the byte range the block occupies, which the reverse direction
/// needs to tell an occurrence inside the block from one outside it.
fn staged_emitter_block(text: &str) -> (Vec<String>, std::ops::Range<usize>) {
    let path = spec_two();
    let anchor_at = text
        .find(ANCHOR)
        .unwrap_or_else(|| panic!("{}: no '{ANCHOR}' sentence", path.display()));
    let open_at = text[anchor_at..]
        .find("```")
        .map(|at| anchor_at + at)
        .unwrap_or_else(|| panic!("{}: no fenced block after '{ANCHOR}'", path.display()));
    let body_at = open_at + "```".len();
    let close_at = text[body_at..]
        .find("```")
        .map(|at| body_at + at)
        .unwrap_or_else(|| {
            panic!(
                "{}: unterminated fenced block after '{ANCHOR}'",
                path.display()
            )
        });
    let lines: Vec<String> = text[body_at..close_at]
        .lines()
        .skip_while(|line| line.trim().is_empty())
        .map(str::to_string)
        .collect();
    let lines = match lines.iter().rposition(|line| !line.trim().is_empty()) {
        Some(last) => lines[..=last].to_vec(),
        None => panic!(
            "{}: the fenced block after '{ANCHOR}' is empty",
            path.display()
        ),
    };
    (lines, open_at..close_at)
}

/// The emitter named by the `--format <name>` of the block's command line.
fn quoted_emitter(command: &str) -> Emitter {
    let path = spec_two();
    assert!(
        command.starts_with(PROMPT),
        "{}: the staged-emitter block opens with `{command}`, and a quoted command opens with \
         `{PROMPT}`",
        path.display()
    );
    let name = format_argument(command).unwrap_or_else(|| {
        panic!(
            "{}: the staged-emitter block's command `{command}` names no `--format`",
            path.display()
        )
    });
    Emitter::parse(name).unwrap_or_else(|| {
        panic!(
            "{}: the staged-emitter block asks for `--format {name}`, which is no emitter this \
             engine carries. The emitters are {:?}",
            path.display(),
            Emitter::ALL.map(Emitter::name)
        )
    })
}

/// The token after the first `--format` of one line, if the line has one.
fn format_argument(line: &str) -> Option<&str> {
    let (_, after) = line.split_once("--format")?;
    after.split_whitespace().next()
}

/// The emitter spec 2 quotes is one this engine does not build.
///
/// The direction that fires the day `skos` is built. Spec 2 calls the target
/// "staged and unbuilt" and quotes a refusal underneath the sentence, so an
/// emitter that ships makes both false at once, and neither the compiler nor
/// any other test in this suite reads the paragraph.
///
/// # Watched failing
///
/// Adding `Emitter::Skos` to the `true` arm of `Emitter::is_built` reddens
/// this, naming `skos`.
#[test]
fn the_emitter_spec_2_calls_staged_is_one_this_engine_does_not_build() {
    let text = spec_two_text();
    let (lines, _) = staged_emitter_block(&text);
    let emitter = quoted_emitter(&lines[0]);
    assert!(
        !emitter.is_built(),
        "docs/spec/02-taxonomy-model.md calls the `{}` projection staged and unbuilt and quotes a \
         refusal for it, and this engine builds that emitter. The paragraph at '{ANCHOR}' has to \
         move with the code",
        emitter.name()
    );
}

/// The output spec 2 quotes is the refusal this engine returns, byte for byte.
///
/// The whole reason this file is not the case table in `fixtures.rs`, which
/// asserts two substrings of this same message and so passes under any reword
/// of it. The expected text is built out of `Refusal::reason`, so the engine
/// holds the wording and the document holds a copy a reader reviews.
///
/// # Watched failing
///
/// Changing one word of either copy reddens this, printing both.
#[test]
fn the_output_spec_2_quotes_is_the_refusal_this_engine_returns() {
    let text = spec_two_text();
    let (lines, _) = staged_emitter_block(&text);
    let emitter = quoted_emitter(&lines[0]);

    let expected = vec![
        REFUSAL_HEADER.to_string(),
        format!("{REASON_INDENT}{}", Refusal::NotBuilt(emitter).reason()),
    ];
    let quoted: Vec<String> = lines[1..].to_vec();
    assert_eq!(
        quoted,
        expected,
        "docs/spec/02-taxonomy-model.md's staged-emitter block does not quote what `headwater \
         export --format {}` prints on stderr.\n\nthe document:\n{}\n\nthe engine:\n{}",
        emitter.name(),
        quoted.join("\n"),
        expected.join("\n")
    );
}

/// Every `--format` spec 2 writes names a built emitter, or sits in that block.
///
/// The reverse direction. Without it an emptied fenced block leaves both cases
/// above passing over nothing, and a second unbuilt-emitter example added to
/// this part later goes unread.
///
/// # Watched failing
///
/// Deleting the block's command line reddens the extractor rather than this
/// case, naming the file. Adding `--format shacl` to any other line of spec 2
/// reddens this, naming `shacl` and the line it sits on.
#[test]
fn no_other_format_in_spec_2_names_an_emitter_this_engine_does_not_build() {
    let text = spec_two_text();
    let (_, block) = staged_emitter_block(&text);

    let stray: Vec<(usize, &str)> = text
        .match_indices("--format")
        .filter(|(at, _)| !block.contains(at))
        .filter_map(|(at, _)| {
            let line_start = text[..at].rfind('\n').map(|nl| nl + 1).unwrap_or(0);
            let line_end = text[at..]
                .find('\n')
                .map(|nl| at + nl)
                .unwrap_or(text.len());
            let line = &text[line_start..line_end];
            let unbuilt = format_argument(&text[at..line_end])
                .and_then(Emitter::parse)
                .is_some_and(|emitter| !emitter.is_built());
            unbuilt.then(|| (text[..at].matches('\n').count() + 1, line))
        })
        .collect();

    assert!(
        stray.is_empty(),
        "docs/spec/02-taxonomy-model.md names an emitter this engine does not build outside the \
         staged block at '{ANCHOR}', where the refusal is quoted beside it: {stray:#?}"
    );
}
