// SPDX-License-Identifier: Apache-2.0
//! Spec 6's projection-kinds block, held against [`headwater_generate::unbuilt`].
//!
//! # Why this file exists
//!
//! `docs/spec/06-engine-architecture.md` stated eight projection kinds as
//! implemented, four of which this engine does not emit
//! ([#576](https://github.com/headwater-ai/headwater/issues/576)). The check a
//! reader could run by hand — `headwater generate --check` beside a `grep` of
//! the sentence — could not see three of those four, because this corpus
//! declares no `relation_view`, no `agent_rules` and no `template`, so no run
//! of the verb ever named them. An instrument blind to the defect it exists to
//! catch is worse than none. #596 closed that blind spot in the verb itself,
//! and this file holds the verb to it.
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
//! That is the one reason spec 6 quotes in full, and the other four remain a
//! hand copy in prose. Since
//! [#596](https://github.com/headwater-ai/headwater/issues/596) a run over any
//! corpus prints all five, which
//! [`every_unbuilt_reason_reaches_a_reader_of_a_corpus_that_declares_none`]
//! holds. `Kind::ALL` is a hand-kept list the
//! compiler does not hold complete; the exhaustive `match` in `unbuilt` is
//! what forces a thirteenth variant into that file.

use headwater_check::paint::ColorMode;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use headwater_census::census::{self};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::Shape;
use headwater_generate::{check, plan, unbuilt, Identity, Kind, Projections, Runs};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_query::Surface;

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
/// #576's second clause. Every waiting kind reaches a run over any corpus
/// since #596, and `coverage_report` is the one of them a specification part
/// quotes in full, so it is the one reason that can be held byte-for-byte
/// rather than by hand.
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

/// The report of one run over an empty tree, under the given declarations.
///
/// Everything a [`Surface`] borrows is owned here, so a case states the
/// declarations it cares about and reads the rendered report. The tree is
/// empty and under Cargo's own temporary directory, so every output is missing
/// and no case writes beside its inputs. `name` keeps two cases off one path,
/// because Cargo runs the cases of one target as threads of one process.
fn report_over(projections: &Projections, name: &str) -> String {
    let tree = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&tree);
    std::fs::create_dir_all(&tree).expect("a temporary tree");

    let corpus = Corpus::new(&tree, "corpus");
    let taxonomy = Taxonomy::default();
    let relations = Declarations::default();
    let shape = Shape::default();
    let config = Config::default();
    let census = census::take(&corpus, &taxonomy);
    let graph = Graph::build(
        &census,
        &relations,
        &Resolvers::over(&corpus),
        &corpus,
        &config,
    );
    let surface = Surface::over(&census, &graph, &shape, &taxonomy, &relations, &config);
    let plan = plan(
        &surface,
        &census,
        projections,
        &Identity::default(),
        &Runs::default(),
        headwater_verbs::VERBS,
    );
    check(&tree, &plan).render(ColorMode::Plain)
}

/// Every kind [`unbuilt`] holds a reason for, with the reason it holds.
fn every_unbuilt_kind() -> Vec<(Kind, &'static str)> {
    Kind::ALL
        .into_iter()
        .filter_map(|kind| unbuilt(kind).map(|reason| (kind, reason)))
        .collect()
}

/// The `projections` block of a taxonomy, from its source text.
fn projections_from(source: &str) -> Projections {
    let root = headwater_yaml::load(source)
        .unwrap_or_else(|errors| panic!("the source does not load: {errors:?}"))
        .value
        .as_map()
        .expect("the source is a mapping")
        .clone();
    Projections::read(&root).expect("the projections read")
}

/// Every reason [`unbuilt`] holds is printed by one run over a corpus that
/// declares none of the kinds that hold one.
///
/// #596's clause. `coverage_report` reached a reader because
/// [`headwater_generate::plan`] pushes it with no declaration behind it, and
/// the other four reached a reader only where a taxonomy had already declared
/// the kind. A reader asks "does this emitter exist" before writing the
/// declaration, not after, so the answer cannot be behind the declaration.
///
/// The five come from `Kind::ALL` through `unbuilt`, never from a list typed
/// here, so a sixth kind that acquires a reason joins this case with no edit
/// and a kind that acquires an emitter leaves it.
///
/// # Watched failing
///
/// Before the change this reddened over an empty `Projections`, naming
/// `relation_view`, `agent_rules`, `template` and `transcription`, and not
/// `coverage_report`.
#[test]
fn every_unbuilt_reason_reaches_a_reader_of_a_corpus_that_declares_none() {
    let held = every_unbuilt_kind();
    assert!(
        !held.is_empty(),
        "`headwater_generate::unbuilt` gives no kind a reason, so this case reads nothing"
    );

    let rendered = report_over(&Projections::default(), "every-unbuilt-reason");

    let unreached: Vec<&str> = held
        .iter()
        .filter(|(_, reason)| !rendered.contains(reason))
        .map(|(kind, _)| kind.name())
        .collect();
    assert!(
        unreached.is_empty(),
        "`headwater_generate::unbuilt` gives {unreached:?} a reason that a run over a corpus \
         declaring no projection at all never prints. The report was:\n\n{rendered}"
    );
}

/// A kind a declaration named is reported once, and never twice.
///
/// The guard on the undeclared pass. `plan()` reports a declared kind with no
/// emitter through its own arm, at the declaration's output path, and reports
/// an undeclared one at `no declaration names one`. A kind that took both
/// routes would be printed twice, so a reader would meet one reason under two
/// headings and could not tell which of the two the corpus asked for.
///
/// The case declares two of the four declarable waiting kinds and leaves two
/// undeclared, so both routes run in one report and the count holds over every
/// kind rather than over the declared pair alone.
///
/// # Watched failing
///
/// Deleting the `!declared.contains(kind)` filter from `undeclared` reddens
/// this, naming `relation_view` and the count 2. The case before this one
/// stays green over that regression, because a doubled reason still appears.
#[test]
fn a_kind_a_declaration_named_is_reported_once_and_not_twice() {
    let declared = projections_from(
        "projections:\n  \
         - {kind: relation_view, output: generate/relations.md}\n  \
         - {kind: template, output: generate/templates.md}\n",
    );
    assert_eq!(
        declared.declared.len(),
        2,
        "this case declares two projections, and the block read {:?}",
        declared
            .declared
            .iter()
            .map(|one| one.kind.name())
            .collect::<Vec<_>>()
    );

    let rendered = report_over(&declared, "a-declared-unbuilt-kind");

    let miscounted: Vec<(&str, usize)> = every_unbuilt_kind()
        .into_iter()
        .map(|(kind, reason)| (kind.name(), rendered.matches(reason).count()))
        .filter(|(_, count)| *count != 1)
        .collect();
    assert!(
        miscounted.is_empty(),
        "a run states each unbuilt kind once, and it stated {miscounted:?} some other number of \
         times. The report was:\n\n{rendered}"
    );
}
