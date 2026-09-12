// SPDX-License-Identifier: Apache-2.0
//! Whether a projection `identity` that cannot supply a facet its kind requires
//! is reported by anything.
//!
//! # The defect this file was written against
//!
//! [#780](https://github.com/headwater-ai/headwater/issues/780). The census
//! classifies a marked file `Generated`, and `over_documents` in the check layer
//! skips every census row that is not `Typed`, so `facet.required.missing`
//! creates no instance over a generated document. The census states its own
//! reason for the exemption: `headwater generate --check` holds the file rather
//! than this census. But `generate --check` compares bytes against what the
//! emitter produces now, so it cannot report that the emitter produces bytes the
//! taxonomy refuses. The emitter is the thing being compared against.
//!
//! So the exemption assumes a second reader that does not exist for this class
//! of finding. `decision_register` requires `title`, the tombstone at
//! `docs/spec/09-open-questions.md` is a `decision_register` written by the
//! `shelf_sections` projection, and until [#627](https://github.com/headwater-ai/headwater/issues/627)
//! the `identity` block held no member that could write a `name`-role facet at
//! all. The requirement was unmet for the whole life of the projection and
//! nothing reported it.
//!
//! # What the refusal reads
//!
//! The whole required set of the kind, against everything written into the
//! file: the three members of the `identity` block, and every facet
//! [`headwater_generate`] derives.
//!
//! It used to read three facets only, and the nine it left out were the
//! remainder this file held #780 open for. Over this repository's own lock
//! `governed_document` requires `status`, `status_since`, `last_verified` and
//! `summary`, both generated kinds inherit all four, and `decision_register`
//! adds `doc_type`, `sequence` and `title`. The owner ruled on 2026-09-11 that
//! the engine derives the state, the two dates and the layout facet and that the
//! emitter composes the summary, so all nine are written and the refusal reads
//! the whole set with nothing left over.
//!
//! What it guards now is a taxonomy that requires a facet in no role this
//! engine reads and in no shelf layout. Such a facet is one no author can add,
//! because a generated document's only writer is this engine, and one no check
//! reads, because the census excuses a marked file from every document rule.
//!
//! # The three fixture taxonomies
//!
//! `fixtures/unsuppliable.taxonomy.yml` is `generate.taxonomy.yml` with one
//! difference: the `guide` kind requires `title`, the facet in the `name` role.
//! The `shelf_sections` declaration that writes `generate/archive/RETIRED.md` as
//! a `guide` states no `name`, so the block cannot supply `title`. The repair is
//! a declaration, and the refusal says so.
//!
//! `fixtures/unrolled.taxonomy.yml` is the other half of the guard. It declares
//! a facet with no role at all and requires it of the same kind, so no member of
//! the block writes it and no derivation computes it. There the repair is a
//! change to the taxonomy rather than to the declaration.
//!
//! `fixtures/unheld.taxonomy.yml` is the third refusal, and it is the only one
//! about the body rather than the block. The `guide` kind requires the section
//! `Consequences`, and the `shelf_sections` declaration that writes
//! `generate/archive/RETIRED.md` composes a body out of the shelf label and the
//! name of each document under it. No emitter of this engine reads a section
//! contract, so the repair is the taxonomy: a kind whose population a
//! projection writes states the section contract its emitter can keep.
//!
//! `fixtures/selfread.taxonomy.yml` is about a value rather than a refusal. It
//! is the one shape in this tree where a projection's output sits on the shelf
//! that the projection reads, which makes the output one of its own sources. The
//! committed `selfread/decisions/INDEX.md` carries a `status_since` older than
//! either source document, so a fold that reads it answers a date no source
//! supports. That file carries a `title` for a reason worth knowing: once the
//! output is a document of the shelf, the sections emitter owes it a heading, and
//! without a name the whole declaration is declined before the fold runs. An
//! index that lists itself is a separate question about what an index is for,
//! and nothing in this repository declares one.
//!
//! # Watched failing
//!
//! At `4c298808`, with the fixture taxonomy in the tree and no refusal in
//! `identity::front_matter`, `the_declaration_is_refused_and_names_the_facet`
//! failed with the file planned and `plan.unwritten` empty: the emitter wrote
//! `generate/archive/RETIRED.md` with a front-matter block missing `title` and
//! said nothing.
//!
//! At `9cd6846e`, with the refusal narrowed back to the facet in the `name`
//! role, `a_required_facet_in_no_role_is_refused_because_nothing_can_write_it`
//! failed the same way and on the same path: `generate/archive/RETIRED.md` was
//! written with no value for `tier` and the plan declined nothing. The two
//! cases fail identically and for opposite reasons, which is why both are here.
//!
//! At `484ff613`, with `unheld.taxonomy.yml` in the tree and the refusal reading
//! `facets.require` alone, `a_required_section_no_emitter_writes_is_refused`
//! failed with the file planned and nothing declined: the emitter wrote
//! `generate/archive/RETIRED.md` under the headings `Decisions`, `Rebuild the
//! graph on every run` and `Store the graph on disk`, none of them
//! `Consequences`, and said nothing.
//!
//! At `4e1767d8`, with `selfread.taxonomy.yml` in the tree and the output filter
//! removed from `derived::documents`,
//! `a_generated_document_on_the_shelf_it_reads_does_not_fold_its_own_date`
//! failed on the value rather than on the refusal: the emitter wrote
//! `status_since: 2020-01-01` into `selfread/decisions/INDEX.md`, which is the
//! date the committed file already carried and a date neither source document
//! supports. That failure is the one this file was missing, because the emitter
//! and the gate agree on it. This case was written after a review of
//! [#816](https://github.com/headwater-ai/headwater/pull/816) read the filter in
//! `incoming` and asked why the fold had no equivalent.

use headwater_census::census::{self, Census};
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::Shape;
use headwater_generate::{plan, Identity, Plan, Projections, Runs};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_query::Surface;
use headwater_yaml::Mapping;
use std::path::{Path, PathBuf};

/// The output the declaration under test writes, or declines to write.
const OUTPUT: &str = "generate/archive/RETIRED.md";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("the repository root is three above the crate")
        .to_path_buf()
}

fn load_map(path: &Path) -> Mapping {
    let source =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    headwater_yaml::load(&source)
        .unwrap_or_else(|errors| panic!("{}: {errors:?}", path.display()))
        .value
        .as_map()
        .unwrap_or_else(|| panic!("{} is not a mapping", path.display()))
        .clone()
}

fn identity() -> Identity {
    Identity {
        corpus_root: "generate".to_string(),
        exclusions: Vec::new(),
        package: "headwater/fixture".to_string(),
        version: "1.0.0".to_string(),
        lock: "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string(),
    }
}

struct Built {
    census: Census,
    graph: Graph,
    shape: Shape,
    taxonomy: Taxonomy,
    relations: Declarations,
    config: Config,
}

impl Built {
    fn over(corpus: &Corpus, root: &Mapping) -> Self {
        let taxonomy = Taxonomy::read(root).expect("the taxonomy reads");
        let relations = Declarations::read(root).expect("the declarations read");
        let shape = Shape::read(root).expect("the shape reads");
        let census = census::take(corpus, &taxonomy);
        let graph = Graph::build(
            &census,
            &relations,
            &Resolvers::over(corpus),
            corpus,
            &Config::default(),
        );
        Built {
            census,
            graph,
            shape,
            taxonomy,
            relations,
            config: Config::default(),
        }
    }

    fn surface(&self) -> Surface<'_> {
        Surface::over(
            &self.census,
            &self.graph,
            &self.shape,
            &self.taxonomy,
            &self.relations,
            &self.config,
        )
    }
}

fn plan_over(taxonomy_file: &str) -> Plan {
    let corpus = Corpus::new(fixtures_dir(), "generate");
    let root = load_map(&fixtures_dir().join(taxonomy_file));
    let built = Built::over(&corpus, &root);
    let projections = Projections::read(&root).expect("the projections read");
    plan(
        &built.surface(),
        &built.census,
        &projections,
        &identity(),
        &Runs::default(),
        headwater_verbs::VERBS,
    )
}

/// The decisive case: the block cannot supply `title`, and the verb says so.
///
/// Both halves matter. A declaration that writes a document missing a facet its
/// kind requires must write no document, because a file that is in the corpus is
/// a file the corpus is judged on. And the reason must name the facet, because
/// the person who can act on it is reading a taxonomy source and `title` is the
/// word they will search for.
#[test]
fn the_declaration_is_refused_and_names_the_facet() {
    let plan = plan_over("unsuppliable.taxonomy.yml");

    assert!(
        !plan.outputs.iter().any(|output| output.path == OUTPUT),
        "`{OUTPUT}` was written by a declaration whose `identity` block cannot supply `title`, \
         the facet its kind requires. The outputs were: {:?}",
        plan.outputs
            .iter()
            .map(|output| output.path.as_str())
            .collect::<Vec<_>>()
    );

    let refusal = plan
        .unwritten
        .iter()
        .find(|unwritten| unwritten.at == OUTPUT)
        .unwrap_or_else(|| {
            panic!(
                "nothing reported the declaration that writes `{OUTPUT}`. The plan declined {} \
                 outputs: {:?}",
                plan.unwritten.len(),
                plan.unwritten
                    .iter()
                    .map(|unwritten| unwritten.at.as_str())
                    .collect::<Vec<_>>()
            )
        });
    assert!(
        refusal.reason.contains("title"),
        "the refusal of `{OUTPUT}` does not name the facet the block cannot supply. It reads: {}",
        refusal.reason
    );
    assert!(
        refusal.reason.contains("guide"),
        "the refusal of `{OUTPUT}` does not name the kind that requires the facet. It reads: {}",
        refusal.reason
    );
}

/// The quiet arm, over the same tree with the requirement absent.
///
/// It asserts that the walk reached the declaration and wrote its file, rather
/// than that nothing was reported. A case that asserted only an empty
/// `unwritten` list would pass over a plan that never read the declaration at
/// all, which is the subject rather than the instrument.
#[test]
fn a_kind_that_requires_no_name_facet_is_written_as_before() {
    let plan = plan_over("generate.taxonomy.yml");

    let output = plan
        .outputs
        .iter()
        .find(|output| output.path == OUTPUT)
        .unwrap_or_else(|| {
            panic!(
                "the declaration that writes `{OUTPUT}` produced no output, so this case tests \
                 nothing. The plan wrote: {:?}",
                plan.outputs
                    .iter()
                    .map(|output| output.path.as_str())
                    .collect::<Vec<_>>()
            )
        });
    assert!(
        output.bytes.contains("id: GD-FIX-archive"),
        "the file at `{OUTPUT}` carries no identity block, so the refusal under test was never \
         reached:\n{}",
        output.bytes
    );
    assert!(
        !plan
            .unwritten
            .iter()
            .any(|unwritten| unwritten.at == OUTPUT),
        "`{OUTPUT}` was declined over a taxonomy whose `guide` kind requires no facet this block \
         writes"
    );
}

/// This repository's own declarations, which is what item 2 of the bar measures.
///
/// Two `identity` blocks, and the refusal must be silent over both after this
/// change merges. The numerator and the denominator are both printed, because a
/// property over an empty set reports as a pass and a refactor that stopped
/// reading the declarations would empty this one.
#[test]
fn every_identity_this_repository_declares_supplies_what_this_refusal_reads() {
    let root = repository_root();
    let resolved = headwater_resolve::repository(&root)
        .unwrap_or_else(|errors| panic!("{}", headwater_resolve::render_errors(&errors)));
    let corpus = Corpus::declared(
        &root,
        &resolved.consumer.corpus_root,
        &resolved.consumer.exclusions,
    );
    let built = Built::over(&corpus, &resolved.resolution.taxonomy);
    let projections =
        Projections::read(&resolved.resolution.taxonomy).expect("the projections read");
    let declared = projections
        .declared
        .iter()
        .filter(|entry| entry.identity.is_some())
        .count();
    assert!(
        declared >= 2,
        "this repository declares {declared} `identity` blocks, and it declares at least two: the \
         tombstone at `docs/spec/09-open-questions.md` and the probe result. The reader of the \
         declarations is looking at the wrong field."
    );

    let plan: Plan = plan(
        &built.surface(),
        &built.census,
        &projections,
        &Identity::default(),
        &Runs::default(),
        headwater_verbs::VERBS,
    );
    let refused: Vec<&str> = plan
        .unwritten
        .iter()
        .filter(|unwritten| unwritten.reason.contains("requires the facet"))
        .map(|unwritten| unwritten.at.as_str())
        .collect();
    assert!(
        refused.is_empty(),
        "{} of {declared} identity declarations of this repository cannot supply a facet their \
         kind requires: {refused:?}. This branch must take that count to zero, or it reddens \
         `main` for every branch cut after it.",
        refused.len()
    );
}

/// The other half of the guard: a required facet that carries no role.
///
/// `unsuppliable.taxonomy.yml` covers the facet a declaration could have
/// supplied and did not. This covers the one nothing writing the file can
/// supply at all, which is what the refusal reads for now that the engine
/// derives the state, the two dates and the summary. Without this case the
/// refusal would be exercised only where a repair exists, and a taxonomy that
/// asks for the impossible would be the shape that reaches a corpus unreported.
#[test]
fn a_required_facet_in_no_role_is_refused_because_nothing_can_write_it() {
    let plan = plan_over("unrolled.taxonomy.yml");

    assert!(
        !plan.outputs.iter().any(|output| output.path == OUTPUT),
        "`{OUTPUT}` was written by a declaration whose kind requires `tier`, a facet in no role \
         that no layout names. The outputs were: {:?}",
        plan.outputs
            .iter()
            .map(|output| output.path.as_str())
            .collect::<Vec<_>>()
    );

    let refusal = plan
        .unwritten
        .iter()
        .find(|unwritten| unwritten.at == OUTPUT)
        .unwrap_or_else(|| {
            panic!(
                "nothing reported the declaration that writes `{OUTPUT}`. The plan declined {} \
                 outputs: {:?}",
                plan.unwritten.len(),
                plan.unwritten
                    .iter()
                    .map(|unwritten| unwritten.at.as_str())
                    .collect::<Vec<_>>()
            )
        });
    assert!(
        refusal.reason.contains("tier"),
        "the refusal of `{OUTPUT}` does not name the facet nothing can write. It reads: {}",
        refusal.reason
    );
}

/// The body half: a section the kind requires and no emitter writes.
///
/// The two cases above are about the front-matter block, and a block is a set
/// of facets this engine either writes or does not. A section contract is about
/// the body, and the body of a generated document is composed out of the corpus
/// rather than out of the contract: a `shelf_sections` writes the shelf label
/// and the name of each document on the shelf, and nothing anywhere reads
/// `sections.require` on the way. So a heading that satisfies such a contract is
/// a coincidence, and it stops being true when a document is renamed.
///
/// The silence is the same silence [#780](https://github.com/headwater-ai/headwater/issues/780)
/// reported for a facet, one clause over. `over_documents` creates an instance
/// only for a `Typed` census row, so `section.required.missing` reads no
/// generated file; and `generate --check` compares the body against the emitter
/// that composed it, so it cannot report that the emitter composes a body the
/// taxonomy refuses.
///
/// Ten of the eighteen kinds in this repository's own lock declare
/// `sections.require`, and neither kind this repository generates is one of
/// them. So this fixture is the whole of the evidence, and the case is a guard
/// against a shape no corpus here reaches rather than a repair of one it does.
#[test]
fn a_required_section_no_emitter_writes_is_refused() {
    let plan = plan_over("unheld.taxonomy.yml");

    assert!(
        !plan.outputs.iter().any(|output| output.path == OUTPUT),
        "`{OUTPUT}` was written by a declaration whose kind requires the section \
         `Consequences`, which the body of a generated document never carries. The outputs \
         were: {:?}",
        plan.outputs
            .iter()
            .map(|output| output.path.as_str())
            .collect::<Vec<_>>()
    );

    let refusal = plan
        .unwritten
        .iter()
        .find(|unwritten| unwritten.at == OUTPUT)
        .unwrap_or_else(|| {
            panic!(
                "nothing reported the declaration that writes `{OUTPUT}`. The plan declined {} \
                 outputs: {:?}",
                plan.unwritten.len(),
                plan.unwritten
                    .iter()
                    .map(|unwritten| unwritten.at.as_str())
                    .collect::<Vec<_>>()
            )
        });
    assert!(
        refusal.reason.contains("Consequences"),
        "the refusal of `{OUTPUT}` does not name the section the body does not carry. It reads: \
         {}",
        refusal.reason
    );
    assert!(
        refusal.reason.contains("guide"),
        "the refusal of `{OUTPUT}` does not name the kind that requires the section. It reads: {}",
        refusal.reason
    );
}

/// The derived facets reach the file, and not only the refusal.
///
/// Every other assertion in this file is about a declaration that produces
/// nothing. This one reads the bytes, because a derivation that computed the
/// right values and wrote none of them would pass every case above. The values
/// themselves are the fixture record's business; what this holds is that each
/// facet the kind requires is present exactly once.
#[test]
fn the_derived_facets_are_written_into_the_block() {
    let plan = plan_over("generate.taxonomy.yml");
    let output = plan
        .outputs
        .iter()
        .find(|output| output.path == OUTPUT)
        .unwrap_or_else(|| panic!("`{OUTPUT}` was not written at all"));
    let bytes = &output.bytes;
    let block = bytes
        .split("---")
        .nth(1)
        .unwrap_or_else(|| panic!("`{OUTPUT}` carries no front-matter block. It reads: {bytes}"));

    // `guide` inherits these three from `governed_document` in this fixture, and
    // the `identity` block writes a member for none of them.
    for facet in ["status", "status_since", "summary"] {
        let prefix = format!("{facet}:");
        let written = block
            .lines()
            .filter(|line| line.starts_with(&prefix))
            .count();
        assert_eq!(
            written, 1,
            "`{facet}` is written {written} times in the block of `{OUTPUT}`, and a facet is \
             stated once. The block reads: {block}"
        );
    }
}

/// A generated document on the shelf it reads does not fold its own last value.
///
/// `incoming` already refuses the output path, and the comment there states the
/// reason: an edge an earlier run of this emitter wrote would make the output a
/// function of its own last version. The date fold owes the same refusal and did
/// not make it. `documents` filtered the source set by path alone, and a
/// `shelf_sections` or `shelf_index` whose output sits on the shelf it reads is
/// in that set, because a generated file that declares an identity is a node of
/// the census ([`headwater_census::census::Outcome::node`]).
///
/// The failure is a value that never moves. `stalest_of` takes a minimum, so
/// once the file is committed with a date no source supports, that date is the
/// minimum on every later run and `generate --check` holds it as the right
/// answer. Nothing would report it: the emitter agrees with itself, which is the
/// whole of what the census exemption assumes a second reader for.
///
/// Neither declaration in this repository reaches the shape, so this is a guard
/// rather than a repair. `selfread.taxonomy.yml` is the smallest taxonomy that
/// does reach it, and the committed `INDEX.md` carries 2020-01-01 against
/// sources at 2026-05-01 and 2026-06-01.
#[test]
fn a_generated_document_on_the_shelf_it_reads_does_not_fold_its_own_date() {
    const SELF_READ: &str = "selfread/decisions/INDEX.md";

    let corpus = Corpus::new(fixtures_dir(), "selfread");
    let root = load_map(&fixtures_dir().join("selfread.taxonomy.yml"));
    let built = Built::over(&corpus, &root);
    let projections = Projections::read(&root).expect("the projections read");
    let plan = plan(
        &built.surface(),
        &built.census,
        &projections,
        &identity(),
        &Runs::default(),
        headwater_verbs::VERBS,
    );

    let output = plan
        .outputs
        .iter()
        .find(|output| output.path == SELF_READ)
        .unwrap_or_else(|| {
            panic!(
                "`{SELF_READ}` was not written, so the fold under test never ran. The plan wrote \
                 {:?} and declined {:?}",
                plan.outputs
                    .iter()
                    .map(|output| output.path.as_str())
                    .collect::<Vec<_>>(),
                plan.unwritten
                    .iter()
                    .map(|unwritten| (unwritten.at.as_str(), unwritten.reason.as_str()))
                    .collect::<Vec<_>>()
            )
        });

    // The instrument before the measurement. A fixture whose committed output
    // stopped being a node would make the assertion below pass for the wrong
    // reason, and this is the one shape in the tree that holds it.
    let surface = built.surface();
    let held: Vec<String> = surface
        .documents()
        .into_iter()
        .map(|document| document.path.to_string())
        .collect();
    assert!(
        held.iter().any(|path| path == SELF_READ),
        "`{SELF_READ}` is not a document of this corpus, so it cannot be in its own source set \
         and this case tests nothing. The census holds: {held:?}"
    );

    let written = output
        .bytes
        .lines()
        .find(|line| line.starts_with("status_since:"))
        .unwrap_or_else(|| {
            panic!(
                "`{SELF_READ}` carries no `status_since`, which its kind requires. It reads:\n{}",
                output.bytes
            )
        });
    assert!(
        written.contains("2026-05-01"),
        "`{SELF_READ}` folds its own committed date rather than the stalest of its sources. The \
         sources carry 2026-05-01 and 2026-06-01, the committed file carries 2020-01-01, and the \
         emitter wrote `{written}`. A minimum that includes the output's own last value never \
         moves again."
    );
}
