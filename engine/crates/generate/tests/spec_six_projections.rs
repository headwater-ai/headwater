// SPDX-License-Identifier: Apache-2.0
//! Spec 6's projection-kinds block, held against [`headwater_generate::unbuilt`].
//!
//! # Why this file exists
//!
//! `docs/spec/06-engine-architecture.md` stated eight projection kinds as
//! implemented, four of which this engine does not emit
//! ([#576](https://github.com/headwater-ai/headwater/issues/576)). The check a
//! reader could run by hand — `headwater generate --check` beside a `grep` of
//! the sentence — cannot see three of those four, because this corpus declares
//! no `relation_view`, no `agent_rules` and no `template`, so no run of the
//! verb ever names them. An instrument blind to the defect it exists to catch
//! is worse than none.
//!
//! # The model, and where this is stronger than it
//!
//! `engine/crates/cli/tests/verbs.rs` holds spec 6's CLI grammar block against
//! `headwater_verbs::VERBS` the same way, and the two design points are taken
//! from it. First, the block carries **engine identifiers** rather than the
//! prose the paragraph uses, so there is no prose-to-identifier mapping to
//! drift: a `Kind::prose_name()` returning "agent rule files" would put a
//! second copy of the sentence's words in the engine, and a reword of the
//! sentence would then pass. Second, the comparison runs in **both**
//! directions, so neither an emptied block nor an unnamed emitter passes.
//!
//! `verbs.rs` needs a hardcoded `GRAMMAR_MAY_WAIT` list because the only place
//! a waiting verb is declared is that paragraph's own prose. This file needs no
//! such list: `unbuilt` **is** the declared-wait structure, over all twelve
//! kinds, so both directions read the engine.
//!
//! # What this does not hold
//!
//! The *wording* of a reason is held for `coverage_report` alone, in
//! [`the_coverage_report_reason_of_spec_6_is_the_reason_the_verb_prints`].
//! That is the one reason a run over any corpus prints, and the other four
//! reasons remain a hand copy in prose. `Kind::ALL` is a hand-kept list the
//! compiler does not hold complete; the exhaustive `match` in `unbuilt` is
//! what forces a thirteenth variant into that file.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use headwater_generate::{unbuilt, Kind};

/// The heading-free anchor the extractor splits on.
///
/// A sentence rather than a heading, because the `## Projections` section
/// opens with a different fenced block (the `headwater generate` contract) and
/// anchoring on the heading would read that one.
const ANCHOR: &str = "The block below names all twelve";

/// The two group labels the block uses, left-aligned inside the fence.
const RUNS: &str = "runs";
const WAITS: &str = "waits";

fn spec_six() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../docs/spec/06-engine-architecture.md")
}

/// The `runs` and `waits` groups of spec 6's projection-kinds block.
///
/// The block is a fenced code block. A left-aligned line names a group and an
/// indented line names a kind, so the shape a reader sees is the shape this
/// reads.
fn spec_six_projection_groups() -> (BTreeSet<String>, BTreeSet<String>) {
    let path = spec_six();
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let (_, after_anchor) = text
        .split_once(ANCHOR)
        .unwrap_or_else(|| panic!("{}: no '{ANCHOR}' sentence", path.display()));
    let (_, after_open) = after_anchor
        .split_once("```")
        .unwrap_or_else(|| panic!("{}: no fenced block after '{ANCHOR}'", path.display()));
    let (block, _) = after_open.split_once("```").unwrap_or_else(|| {
        panic!(
            "{}: unterminated fenced block after '{ANCHOR}'",
            path.display()
        )
    });

    let mut runs = BTreeSet::new();
    let mut waits = BTreeSet::new();
    let mut group: Option<&str> = None;
    for line in block.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !line.starts_with(char::is_whitespace) {
            group = match trimmed {
                RUNS => Some(RUNS),
                WAITS => Some(WAITS),
                other => panic!(
                    "{}: the projection-kinds block names a group `{other}`, and this test knows \
                     `{RUNS}` and `{WAITS}`",
                    path.display()
                ),
            };
            continue;
        }
        match group {
            Some(RUNS) => {
                runs.insert(trimmed.to_string());
            }
            Some(WAITS) => {
                waits.insert(trimmed.to_string());
            }
            _ => panic!(
                "{}: the projection-kinds block names `{trimmed}` under no group",
                path.display()
            ),
        }
    }
    (runs, waits)
}

/// Every name in the block is a `Kind::name()`, and the block names all twelve.
///
/// D3 and D4 of the four assertions. Without the second, a renamed sentence or
/// a moved fence makes the extractor read a smaller set and every other case
/// here passes over it.
///
/// # Watched failing
///
/// Renaming the anchor sentence panics in the extractor, naming the file.
/// Deleting one line from either group fails the count, naming the count.
#[test]
fn the_projection_kinds_block_of_spec_6_names_the_twelve_kinds_this_engine_declares() {
    let (runs, waits) = spec_six_projection_groups();
    let named: BTreeSet<&String> = runs.union(&waits).collect();

    let known: BTreeSet<&str> = Kind::ALL.into_iter().map(Kind::name).collect();
    let unknown: Vec<&&String> = named
        .iter()
        .filter(|name| !known.contains(name.as_str()))
        .collect();
    assert!(
        unknown.is_empty(),
        "docs/spec/06-engine-architecture.md's projection-kinds block names {unknown:?}, which is \
         no projection kind this engine carries"
    );

    assert_eq!(
        named.len(),
        Kind::ALL.len(),
        "docs/spec/06-engine-architecture.md's projection-kinds block names {} kinds and this \
         engine carries {}. The block is {:?} under `{RUNS}` and {:?} under `{WAITS}`",
        named.len(),
        Kind::ALL.len(),
        runs,
        waits
    );

    let overlap: Vec<&String> = runs.intersection(&waits).collect();
    assert!(
        overlap.is_empty(),
        "docs/spec/06-engine-architecture.md's projection-kinds block names {overlap:?} under both \
         `{RUNS}` and `{WAITS}`"
    );
}

/// A kind under `runs` has no `unbuilt` reason, and a kind with none is under
/// `runs`.
///
/// D1 and D2. D1 is the defect #576 reports: `relation_view`, `agent_rules`,
/// `template` and `coverage_report` were all stated as implemented, and
/// `headwater generate --check` over this corpus names only the last of the
/// four, because this corpus declares no instance of the other three. D2 is
/// the positive control, without which deleting the `runs` group leaves D1
/// passing over nothing.
///
/// The relation is equality rather than containment, which is where this
/// differs from the CLI grammar case in `engine/crates/cli/tests/verbs.rs`.
/// That block may be ahead of the engine because a waiting verb is declared
/// only in its prose. Here the wait has a home in the engine, so a kind that
/// waits belongs under `{WAITS}` and is still named.
///
/// # Watched failing in three directions
///
/// Moving `relation_view` into the `runs` group reddens the first assertion,
/// naming `relation_view` — the exact defect #576 reports. Deleting
/// `shelf_index` from the `runs` group reddens the second, naming
/// `shelf_index`. Moving `Kind::CoverageReport` back into `unbuilt`'s `None`
/// arm reddens the second too, naming `coverage_report` — which is what proves
/// this case reads the function rather than a list baked in here.
#[test]
fn the_runs_group_of_spec_6_is_exactly_the_projection_kinds_this_engine_emits() {
    let (runs, waits) = spec_six_projection_groups();

    let stated_built_but_waiting: Vec<(&String, &str)> = runs
        .iter()
        .filter_map(|name| {
            Kind::ALL
                .into_iter()
                .find(|kind| kind.name() == name.as_str())
                .and_then(|kind| unbuilt(kind).map(|reason| (name, reason)))
        })
        .collect();
    assert!(
        stated_built_but_waiting.is_empty(),
        "docs/spec/06-engine-architecture.md's projection-kinds block puts {:?} under `{RUNS}`, \
         and `headwater_generate::unbuilt` gives each one a reason it writes nothing: {:#?}",
        stated_built_but_waiting
            .iter()
            .map(|(name, _)| name)
            .collect::<Vec<_>>(),
        stated_built_but_waiting
    );

    let emitted_but_not_stated: Vec<&str> = Kind::ALL
        .into_iter()
        .filter(|kind| unbuilt(*kind).is_none())
        .map(Kind::name)
        .filter(|name| !runs.contains(*name))
        .collect();
    assert!(
        emitted_but_not_stated.is_empty(),
        "this engine emits {emitted_but_not_stated:?} and \
         docs/spec/06-engine-architecture.md's projection-kinds block does not name them under \
         `{RUNS}`. The block is {runs:?} under `{RUNS}` and {waits:?} under `{WAITS}`"
    );
}

/// The reason spec 6 quotes for `coverage_report` is the reason the verb prints.
///
/// #576's second clause. `coverage_report` is the one waiting kind that a run
/// over any corpus names, because it is engine-defined and needs no
/// declaration to reach the report, so it is the one reason a specification
/// part can be held to byte-for-byte rather than by hand.
///
/// # Watched failing
///
/// Changing one word of either copy reddens this, printing both.
#[test]
fn the_coverage_report_reason_of_spec_6_is_the_reason_the_verb_prints() {
    let path = spec_six();
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let reason = unbuilt(Kind::CoverageReport).expect(
        "`unbuilt` gives the register a reason: it is the entry `headwater generate --check` \
         prints under `what this verb does not write, and why`",
    );
    assert!(
        text.contains(reason),
        "docs/spec/06-engine-architecture.md does not quote the reason \
         `headwater generate --check` prints for `coverage_report`:\n\n{reason}"
    );
}
