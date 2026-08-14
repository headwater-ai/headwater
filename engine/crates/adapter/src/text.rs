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

use crate::Subject;
use headwater_census::census::{Census, Detail as CensusDetail};
use headwater_check::Run;
use headwater_graph::{Detail as GraphDetail, Graph};

/// One run of the check layer, as a terminal reads it.
pub fn render(run: &Run, census: &Census, graph: &Graph, subject: &Subject<'_>) -> String {
    let mut out = String::new();
    out.push_str("taxonomy\n");
    out.push_str(&format!("  {} {}\n", subject.package, subject.version));
    out.push_str(&format!("  {}\n", subject.lock));
    // The injected clock, reported because it is an input to the verdict. The
    // other three formats state it too, and `Subject` is why all four state the
    // same value.
    out.push_str("\nclock\n");
    out.push_str(&format!("  {}\n", subject.now));
    out.push_str("\ncensus\n");
    out.push_str(&indent(&census.render(CensusDetail::Exceptions)));
    out.push_str("\ngraph\n");
    out.push_str(&indent(&graph.render(GraphDetail::Exceptions)));
    out.push_str("\nchecks\n");
    out.push_str(&indent(&run.render(headwater_check::Detail::Findings)));
    // The same bytes `check --read-set` writes to a file, so a gate reading the
    // file and a reader of this report are looking at one artifact.
    out.push_str("\nread set\n");
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
