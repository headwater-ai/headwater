// SPDX-License-Identifier: Apache-2.0
//! Whether a derived state is one the document's own kind admits.
//!
//! # The gap this closes
//!
//! [`headwater_generate::derived::standing`] computes a state two ways — from
//! an incoming edge's `on_target.set_state`, or by falling back to the facet
//! value whose role is `live` — and called `Shape::lifecycle_of` nowhere.
//! `lifecycle.state.not_admitted`, the check-layer rule that reads a kind's
//! lifecycle regime against the state a document stands in
//! (`headwater_check::lifecycle_state::StateAdmitted`), never sees a generated
//! document either: the census classifies a marked file `Generated`, and
//! `over_documents` creates an instance only for a `Typed` row. So a
//! generator could hand a document a state its own kind's regime does not
//! name, and nothing would notice
//! ([HW-DR-0063](../../../../docs/decisions/0063-every-required-facet-of-a-generated-document-is-derived-and-the-emitter-composes-the-summary.md),
//! [#945](https://github.com/headwater-ai/headwater/issues/945)).
//!
//! A computed state has two possible loci — an incoming edge's
//! `on_target.set_state`, or the fallback — and a refusal has to name whichever
//! one produced the value, because the repair differs: a relation declaration,
//! a taxonomy's state facet, or (for the regime-admission defect) either
//! locus against the kind's own regime. [`headwater_generate::derived::Locus`]
//! is what carries that distinction from `standing` to `admitted`.
//!
//! # The fixture
//!
//! `fixtures/lifecycle-regime.taxonomy.yml` declares two lifecycle regimes over
//! one state vocabulary, the shape
//! `headwater_check::lifecycle_state`'s own `narrow`/`wide` tests already use,
//! renamed here to `wide` and `strict` so that a refusal naming the regime
//! cannot be satisfied by a kind name a test also asserts on.
//!
//! `wide_notice` and `narrow_notice` carry no incoming edge that sets a state,
//! so `derived::standing`'s fallback path is what answers for both: the value
//! whose role is `live`, which this fixture's vocabulary names `current`.
//! `wide_notice` binds a regime that names `current`, so the declaration that
//! writes it is written whole. `narrow_notice` binds `strict`, which never
//! reaches `current`, so the same fallback computes a state that kind's own
//! regime has no place for — the defect class HW-DR-0063 flagged: a state
//! computed correctly against the global vocabulary and wrong against the
//! specific kind's narrower regime.
//!
//! `edge_notice` and `bogus_notice` reach the same two defect classes through
//! the other locus. The first document on the `decisions` shelf declares
//! `flags: [ED-FIX-edge]` and `flags_bogus: [BG-FIX-bogus]`, and the two
//! relations' `on_target.set_state` are what `derived::standing` reads before
//! it ever falls back. `flags` sets `current`, which the vocabulary holds and
//! `edge_notice`'s own regime `strict` does not name — the regime-admission
//! defect again, this time from an edge, and a refusal over it must name the
//! edge and not the fallback the document never reached. `flags_bogus` sets
//! `unheard_of`, which the state facet does not admit at all, whatever regime
//! `bogus_notice` binds — a different defect that a lifecycle regime has no
//! part in, and a refusal over it must not talk about a regime at all.
//!
//! This repository's own lock cannot exercise any of the three failing cases:
//! [HW-OBL-0196](../../../../docs/obligations/0196-a-relation-writes-a-state-onto-a-kind-that-binds-no-lifecycle-regime-and-nothing-reads-that-pair.md)
//! records that all seventeen concrete kinds of `headwater/standard` bind a
//! lifecycle regime, of the six generated documents whose kind binds one every
//! one happens to stand at a state its own regime admits today, and no
//! relation in this repository's lock sets a state its target's regime, or the
//! vocabulary, does not hold. A purpose-built taxonomy is the only way to
//! reach any of the three at all.
//!
//! # Watched failing
//!
//! Before [`headwater_generate::derived::members`] read a kind's lifecycle
//! regime at all, with an earlier form of this fixture in the tree (a
//! `narrow` regime bound by `narrow_notice` alone, no edge-locus cases),
//! `a_state_the_kinds_own_regime_does_not_admit_is_refused` failed with
//! `lifecycle-regime/notices/NARROW.md` planned and `plan.unwritten` empty:
//! the emitter wrote `status: current` into a document of kind
//! `narrow_notice`, whose bound regime never reaches `current`, and said
//! nothing. A second-opinion review of that change found the edge locus
//! untested and a regime-name assertion that a kind name already satisfied
//! trivially; `a_state_a_relation_sets_that_the_kinds_own_regime_does_not_admit_is_refused_and_names_the_edge`
//! and `a_state_a_relation_sets_that_the_vocabulary_does_not_admit_at_all_is_refused_and_names_the_facet_not_a_regime`
//! below, and the `strict` rename, are what closed that.

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

/// The output whose kind binds a regime that admits the fallback state.
const WIDE: &str = "lifecycle-regime/notices/WIDE.md";

/// The output whose kind binds a regime that does not admit the fallback
/// state.
const NARROW: &str = "lifecycle-regime/notices/NARROW.md";

/// The output whose kind binds a regime that does not admit the state an
/// incoming edge sets.
const EDGE: &str = "lifecycle-regime/notices/EDGE.md";

/// The output whose kind's incoming edge sets a state the vocabulary does not
/// admit at all.
const BOGUS: &str = "lifecycle-regime/notices/BOGUS.md";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
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
        corpus_root: "lifecycle-regime".to_string(),
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

fn plan_over_lifecycle_regime() -> Plan {
    let corpus = Corpus::new(fixtures_dir(), "lifecycle-regime");
    let root = load_map(&fixtures_dir().join("lifecycle-regime.taxonomy.yml"));
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

/// The quiet half: a regime that admits the fallback state writes the file.
///
/// Asserted first and separately from the refusal below, so that a fixture
/// wired to always refuse — a taxonomy no document could ever satisfy —
/// cannot pass the decisive case for the wrong reason.
#[test]
fn a_state_the_kinds_own_regime_admits_is_written() {
    let plan = plan_over_lifecycle_regime();

    let output = plan
        .outputs
        .iter()
        .find(|output| output.path == WIDE)
        .unwrap_or_else(|| {
            panic!(
                "`{WIDE}` was not written, even though `wide_notice` binds a regime that admits \
                 `current`, the state the fallback computes. The plan declined {:?}",
                plan.unwritten
                    .iter()
                    .map(|unwritten| (unwritten.at.as_str(), unwritten.reason.as_str()))
                    .collect::<Vec<_>>()
            )
        });
    assert!(
        output.bytes.contains("status: current"),
        "`{WIDE}` does not carry `status: current`, so this case exercises the wrong value. It \
         reads:\n{}",
        output.bytes
    );
    assert!(
        !plan.unwritten.iter().any(|unwritten| unwritten.at == WIDE),
        "`{WIDE}` was both written and declined, which is not a state this plan should reach"
    );
}

/// With no edge setting the state, the date it was entered is the newest
/// `state_entered` over the documents the projection read (#820).
///
/// The owner ruled on 2026-09-25 that a page is no fresher than its newest
/// input, so the fallback fold takes the maximum. `WIDE.md` reads
/// `0001-first-decision.md` (`status_since: 2026-01-01`) and
/// `0003-later-decision.md` (`status_since: 2026-03-01`), and no edge sets its
/// state, so it carries `2026-03-01`. The stalest fold writes `2026-01-01`.
/// `wide_notice` carries no facet in the `freshness` role, so freshness, which
/// stays the stalest, is held by
/// `a_state_a_relation_sets_that_the_kinds_own_regime_admits_is_written_with_the_setters_date`
/// holds over the same two documents.
///
/// Watched failing: at `e63f5383`, with this case and `0003-later-decision.md`
/// in the tree and the fallback fold still taking the minimum, `WIDE.md`
/// carried `status_since: 2026-01-01` and this case failed on that value.
#[test]
fn a_state_no_edge_sets_is_dated_by_the_newest_document_the_projection_read() {
    let plan = plan_over_lifecycle_regime();

    let output = plan
        .outputs
        .iter()
        .find(|output| output.path == WIDE)
        .unwrap_or_else(|| {
            panic!(
                "`{WIDE}` was not written. The plan declined {:?}",
                plan.unwritten
                    .iter()
                    .map(|unwritten| (unwritten.at.as_str(), unwritten.reason.as_str()))
                    .collect::<Vec<_>>()
            )
        });
    let bytes = output.bytes.as_str();
    assert_eq!(
        member(bytes, "status_since"),
        Some("2026-03-01"),
        "`{WIDE}` did not take the date it entered its state as the newest `status_since` over \
         the documents the projection read (2026-03-01). `2026-01-01` is the stalest. It \
         reads:\n{bytes}"
    );
}

/// The decisive case: the fallback computes a state `narrow_notice`'s own
/// regime does not admit, and the declaration is refused rather than written
/// with that state.
///
/// Both halves matter, on the same terms `identity_required_facets.rs` states
/// for the facet-supply refusal: a declaration that would write a document
/// standing at a state its kind's regime does not admit must write no
/// document, and the reason must name the state and the kind, because the
/// person who can act on it is reading a taxonomy source and both words are
/// what they will search for.
#[test]
fn a_state_the_kinds_own_regime_does_not_admit_is_refused() {
    let plan = plan_over_lifecycle_regime();

    assert!(
        !plan.outputs.iter().any(|output| output.path == NARROW),
        "`{NARROW}` was written by a declaration whose kind `narrow_notice` binds a regime that \
         never reaches `current`, the state the fallback computes. The outputs were: {:?}",
        plan.outputs
            .iter()
            .map(|output| output.path.as_str())
            .collect::<Vec<_>>()
    );

    let refusal = plan
        .unwritten
        .iter()
        .find(|unwritten| unwritten.at == NARROW)
        .unwrap_or_else(|| {
            panic!(
                "nothing reported the declaration that writes `{NARROW}`. The plan declined {} \
                 outputs: {:?}",
                plan.unwritten.len(),
                plan.unwritten
                    .iter()
                    .map(|unwritten| unwritten.at.as_str())
                    .collect::<Vec<_>>()
            )
        });
    assert!(
        refusal.reason.contains("current"),
        "the refusal of `{NARROW}` does not name the offending state. It reads: {}",
        refusal.reason
    );
    assert!(
        refusal.reason.contains("narrow_notice"),
        "the refusal of `{NARROW}` does not name the kind that binds the regime. It reads: {}",
        refusal.reason
    );
    assert!(
        refusal.reason.contains("strict"),
        "the refusal of `{NARROW}` does not name the regime that does not admit the state. It \
         reads: {}",
        refusal.reason
    );
}

/// The other locus, over the same defect class: an incoming edge, rather than
/// the fallback, sets a state the kind's own regime does not admit.
///
/// `edge_notice`'s document never reaches `derived::standing`'s fallback at
/// all — the `flags` edge from the one decision answers first — so a refusal
/// that named the `live`-role fallback here would be naming a path this
/// document never took. The reason must name the edge instead, and it must
/// not repeat the fallback's own wording.
#[test]
fn a_state_a_relation_sets_that_the_kinds_own_regime_does_not_admit_is_refused_and_names_the_edge()
{
    let plan = plan_over_lifecycle_regime();

    assert!(
        !plan.outputs.iter().any(|output| output.path == EDGE),
        "`{EDGE}` was written by a declaration whose kind `edge_notice` binds a regime that never \
         reaches `current`, the state the `flags` edge sets. The outputs were: {:?}",
        plan.outputs
            .iter()
            .map(|output| output.path.as_str())
            .collect::<Vec<_>>()
    );

    let refusal = plan
        .unwritten
        .iter()
        .find(|unwritten| unwritten.at == EDGE)
        .unwrap_or_else(|| {
            panic!(
                "nothing reported the declaration that writes `{EDGE}`. The plan declined {} \
                 outputs: {:?}",
                plan.unwritten.len(),
                plan.unwritten
                    .iter()
                    .map(|unwritten| unwritten.at.as_str())
                    .collect::<Vec<_>>()
            )
        });
    assert!(
        refusal.reason.contains("current"),
        "the refusal of `{EDGE}` does not name the offending state. It reads: {}",
        refusal.reason
    );
    assert!(
        refusal.reason.contains("edge_notice"),
        "the refusal of `{EDGE}` does not name the kind that binds the regime. It reads: {}",
        refusal.reason
    );
    assert!(
        refusal.reason.contains("strict"),
        "the refusal of `{EDGE}` does not name the regime that does not admit the state. It \
         reads: {}",
        refusal.reason
    );
    assert!(
        refusal
            .reason
            .contains("lifecycle-regime/decisions/0001-first-decision.md"),
        "the refusal of `{EDGE}` does not name the document whose edge set the state, so a \
         reader cannot find the relation declaration to change. It reads: {}",
        refusal.reason
    );
    assert!(
        !refusal.reason.contains("`live`"),
        "the refusal of `{EDGE}` names the fallback's `live`-role wording, but this document's \
         state came from the `flags` edge and never reached the fallback. It reads: {}",
        refusal.reason
    );
}

/// The vocabulary locus: an incoming edge sets a state the state facet does
/// not admit at all, which is a different defect from a regime that does not
/// name a state the vocabulary holds, and needs a different reason.
///
/// `StateAdmitted::evaluate` skips this class as [`Stood::NotAState`] over a
/// committed document, because `facet.value.not_permitted` owns it; the
/// census exemption means nothing else ever reads it for a generated one, so
/// this refusal is the only place the defect surfaces at all. The reason must
/// name the facet and the value, and it must not talk about a lifecycle
/// regime, because `bogus_notice`'s own regime is not what is wrong here.
#[test]
fn a_state_a_relation_sets_that_the_vocabulary_does_not_admit_at_all_is_refused_and_names_the_facet_not_a_regime(
) {
    let plan = plan_over_lifecycle_regime();

    assert!(
        !plan.outputs.iter().any(|output| output.path == BOGUS),
        "`{BOGUS}` was written by a declaration whose `flags_bogus` edge sets `unheard_of`, a \
         value the state facet does not admit at all. The outputs were: {:?}",
        plan.outputs
            .iter()
            .map(|output| output.path.as_str())
            .collect::<Vec<_>>()
    );

    let refusal = plan
        .unwritten
        .iter()
        .find(|unwritten| unwritten.at == BOGUS)
        .unwrap_or_else(|| {
            panic!(
                "nothing reported the declaration that writes `{BOGUS}`. The plan declined {} \
                 outputs: {:?}",
                plan.unwritten.len(),
                plan.unwritten
                    .iter()
                    .map(|unwritten| unwritten.at.as_str())
                    .collect::<Vec<_>>()
            )
        });
    assert!(
        refusal.reason.contains("unheard_of"),
        "the refusal of `{BOGUS}` does not name the offending value. It reads: {}",
        refusal.reason
    );
    assert!(
        refusal.reason.contains("state facet"),
        "the refusal of `{BOGUS}` does not name the state facet as the thing that does not admit \
         the value. It reads: {}",
        refusal.reason
    );
    assert!(
        !refusal.reason.contains("lifecycle regime"),
        "the refusal of `{BOGUS}` talks about a lifecycle regime, but a value outside the \
         vocabulary is outside every regime over it and the regime is not the defect here. It \
         reads: {}",
        refusal.reason
    );
}

/// The output whose kind's incoming edge sets a state its own regime names, so
/// the file is written.
const WRITTEN: &str = "lifecycle-regime/notices/WRITTEN.md";

/// The one value of `facet` in the front-matter block of `bytes`.
fn member<'a>(bytes: &'a str, facet: &str) -> Option<&'a str> {
    let prefix = format!("{facet}:");
    bytes
        .split("---")
        .nth(1)?
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .map(str::trim)
}

/// The written half of the edge locus: an edge sets a state the target's own
/// regime names, and the file carries it (#820).
///
/// Every other edge case in this file is a refusal, so until this case the
/// three values HW-DR-0063's table derives for such a file were held by the
/// blessed corpus alone. The fixture separates the two sources of each date.
/// The setter, `lifecycle-regime/flaggers/0002-retiring-decision.md`, sits on
/// a shelf the projection does not read and carries `status_since: 2026-04-01`
/// and `last_verified: 2026-05-01`. The two documents the projection reads
/// carry `2026-01-01` and `2026-02-01` (`0001-first-decision.md`) and
/// `2026-03-01` and `2026-03-15` (`0003-later-decision.md`). So a fold that
/// took `state_entered` from the read set writes `2026-01-01` or `2026-03-01`,
/// a fold that took freshness from the setter writes `2026-05-01`, and a fold
/// that took freshness as the newest over the read set writes `2026-03-15`.
/// This case refuses all three: the setter's date wins for the state, and the
/// stalest read-set date wins for freshness.
#[test]
fn a_state_a_relation_sets_that_the_kinds_own_regime_admits_is_written_with_the_setters_date() {
    let plan = plan_over_lifecycle_regime();

    let output = plan
        .outputs
        .iter()
        .find(|output| output.path == WRITTEN)
        .unwrap_or_else(|| {
            panic!(
                "`{WRITTEN}` was not written, even though the `flags_written` edge sets `retired` \
                 and `written_notice` binds `wide`, which names it. The plan declined {:?}",
                plan.unwritten
                    .iter()
                    .map(|unwritten| (unwritten.at.as_str(), unwritten.reason.as_str()))
                    .collect::<Vec<_>>()
            )
        });
    let bytes = output.bytes.as_str();
    assert_eq!(
        member(bytes, "status"),
        Some("retired"),
        "`{WRITTEN}` does not stand at the state its incoming edge sets. It reads:\n{bytes}"
    );
    assert_eq!(
        member(bytes, "status_since"),
        Some("2026-04-01"),
        "`{WRITTEN}` did not take the date it entered its state from the document whose edge \
         set it (2026-04-01). `2026-01-01` is the read set's date. It reads:\n{bytes}"
    );
    assert_eq!(
        member(bytes, "last_verified"),
        Some("2026-02-01"),
        "`{WRITTEN}` did not take its freshness as the stalest date over the documents the \
         projection read (2026-02-01). `2026-03-15` is the newest of them, and `2026-05-01` is \
         the setter's date, and the setter was not read. It reads:\n{bytes}"
    );
}
