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
//! # The two fixtures
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
