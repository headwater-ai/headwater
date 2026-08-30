// SPDX-License-Identifier: Apache-2.0
//! The report a person reads in a terminal, and the one format that is not a
//! translation.
//!
//! The other three write a run into a vocabulary some platform already speaks.
//! This one writes it in the engine's own words, so it is the only format that
//! carries the two phases before the checks: the census is the denominator
//! every coverage number is against, and the graph is what an edge-scoped rule
//! read. A reader who sees a finding count and neither of those cannot tell a
//! clean corpus from an unclassified one.
//!
//! # One composition, and every caller renders through it
//!
//! `headwater check` prints this, and the MCP `check` tool returns it. Two
//! compositions of one report is the drift
//! [spec 6](../../../../docs/spec/06-engine-architecture.md#library) rules
//! against by making the CLI "a thin shell over a library API": a terminal and
//! an agent that read different text are reading about different corpora as
//! far as either can tell.
//!
//! # The order of the blocks is the order of the pipeline
//!
//! Taxonomy, clock, census, graph, checks, read set. The two injected values
//! come first because a verdict is a function of them, and a report that
//! omitted them could not be reproduced from itself. The read set comes last
//! because it is the artifact a later `headwater gate` reads, and it is the
//! longest block by a wide margin.
//!
//! # The layout, and the one block that is exempt from it
//!
//! Three of the four blocks below arrive as the finished text of a renderer in
//! another crate, and two of those crates sit under `headwater-check` and cannot
//! reach the fill at all. So the layout is a pass over the composed text here,
//! by [`headwater_check::fill::filled`], rather than a width threaded through
//! `Census::render`, `Graph::render` and `Run::render`. That is also what keeps
//! `crates/census/fixtures/corpus.census`, `crates/graph/fixtures/corpus.graph`
//! and `crates/check/fixtures/corpus.checks` where they are: the composers still
//! write what they wrote, and this function lays it out.
//!
//! **The read set is not laid out, and that is the sharpest edge in this file.**
//! It is the same bytes `check --read-set` writes to a file, and
//! `headwater_check::Recorded::parse` reads that file back in `headwater gate`.
//! It is a grammar rather than prose — `lock`, `clock`, `barrier`, `windowed`,
//! `version` and `input <path> <sha256>` — so a fold inside it would break the
//! gate outright, and a reader comparing the block to the file would find two
//! artifacts where the sentence above promises one. It is also where 127 of the
//! report's unbreakable lines are: a document path is one word, and no
//! space-respecting fill narrows it.
//!
//! Nothing else is exempt. A word longer than the room it lands in is written
//! past the width, whole, so a path, a rule name and a digest arrive intact
//! wherever they are — which is what `crate::census` needs, because it audits
//! every format by `artifact.contains(path)`.

use crate::Subject;
use headwater_census::census::{Census, Detail as CensusDetail};
use headwater_check::paint::{paint, ColorMode, Role};
use headwater_check::{fill, Run};
use headwater_graph::{Detail as GraphDetail, Graph};

/// One run of the check layer, as a terminal reads it, at the standard width,
/// with no color.
///
/// [`render_at`] is the one caller that colors anything: every other caller
/// of this shorthand is a machine format's cousin or a fixture, neither of
/// which a color decision was ever asked of.
pub fn render(run: &Run, census: &Census, graph: &Graph, subject: &Subject<'_>) -> String {
    render_at(run, census, graph, subject, fill::WIDTH, ColorMode::Plain)
}

/// The same report, laid out at a width the caller states and colored under
/// the mode the caller decided.
///
/// `headwater check --wide` is the one caller that states a width, out of
/// `headwater_cli::paint::width`, which holds a `COLUMNS` reading to
/// `[80, 120]`. Every other caller goes through [`render`] and gets 80, so a run
/// piped into a file and a run under a terminal write the same bytes unless
/// somebody asked for something else on the command line.
///
/// `mode` colors the six block headings below and cascades into `run.render`,
/// which colors a finding's severity, path, obligation and `fix` label. The
/// census and the graph blocks stay uncolored: neither carries a severity, a
/// path role or a heading role of its own today, and inventing one for a
/// palette item this report does not otherwise use would be decoration
/// without a reader. Every color word is a single word with no space in it,
/// so [`fill::filled`]'s word-boundary fold never splits one — see
/// `headwater_check::paint`'s module comment for why counting an escape
/// sequence as columns costs at most an early wrap, never a broken one.
pub fn render_at(
    run: &Run,
    census: &Census,
    graph: &Graph,
    subject: &Subject<'_>,
    width: usize,
    mode: ColorMode,
) -> String {
    let mut out = String::new();
    out.push_str(&paint(Role::Heading, "taxonomy", mode));
    out.push('\n');
    out.push_str(&format!("  {} {}\n", subject.package, subject.version));
    out.push_str(&format!("  {}\n", subject.lock));
    // The injected clock, reported because it is an input to the verdict. The
    // other three formats state it too, and `Subject` is why all four state the
    // same value.
    out.push_str(&format!("\n{}\n", paint(Role::Heading, "clock", mode)));
    out.push_str(&format!("  {}\n", subject.now));
    out.push_str(&format!("\n{}\n", paint(Role::Heading, "census", mode)));
    out.push_str(&fill::filled(
        &indent(&census.render(CensusDetail::Exceptions)),
        width,
    ));
    out.push_str(&format!("\n{}\n", paint(Role::Heading, "graph", mode)));
    out.push_str(&fill::filled(
        &indent(&graph.render(GraphDetail::Exceptions)),
        width,
    ));
    out.push_str(&format!("\n{}\n", paint(Role::Heading, "checks", mode)));
    out.push_str(&fill::filled(
        &indent(&run.render(headwater_check::Detail::Findings, mode)),
        width,
    ));
    // The same bytes `check --read-set` writes to a file, so a gate reading the
    // file and a reader of this report are looking at one artifact. Not laid
    // out, for that reason — see the module comment.
    out.push_str(&format!("\n{}\n", paint(Role::Heading, "read set", mode)));
    out.push_str(&indent(&run.read_set.render()));
    out
}

/// Every line moved two spaces right, and a blank line left blank.
///
/// A blank line that carried the indent would put trailing whitespace in an
/// artifact that a test compares byte for byte.
fn indent(text: &str) -> String {
    text.lines()
        .map(|line| match line.is_empty() {
            true => String::from("\n"),
            false => format!("  {line}\n"),
        })
        .collect()
}
