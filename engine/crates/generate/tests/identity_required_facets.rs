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
//! # What the refusal reads, and what it deliberately does not
//!
//! It reads the facets the `identity` block is the writer of: the identifier
//! facet, the discriminator of a heterogeneous shelf, and the facet in the
//! `name` role. It does not read every facet the kind requires.
//!
//! That line is measured rather than chosen for convenience. Over this
//! repository's own lock, `governed_document` requires `status`,
//! `status_since`, `last_verified` and `summary`, and all four are inherited by
//! both kinds this corpus generates. A refusal over the whole required set
//! would name 5 facets on `docs/spec/09-open-questions.md` and 4 on
//! `docs/probe-results/regression-probe-transcript-for-2026-09-09.md`, and it
//! would refuse both of the two identity declarations this repository makes,
//! with no member of the block able to answer any of the nine. Whether a
//! generated document should be excused from `status` and `summary` is a
//! question about the census exemption and about spec 6's stated position, not
//! about this block, and #780 stays open holding it.
//!
//! # The fixture
//!
//! `fixtures/unsuppliable.taxonomy.yml` is `generate.taxonomy.yml` with one
//! difference: the `guide` kind requires `title`, the facet in the `name` role.
//! The `shelf_sections` declaration that writes `generate/archive/RETIRED.md` as
//! a `guide` states no `name`, so the block cannot supply `title`.
//!
//! # Watched failing
//!
//! At `4c298808`, with the fixture taxonomy in the tree and no refusal in
//! `identity::front_matter`, `the_declaration_is_refused_and_names_the_facet`
//! failed with the file planned and `plan.unwritten` empty: the emitter wrote
//! `generate/archive/RETIRED.md` with a front-matter block missing `title` and
//! said nothing.

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
