// SPDX-License-Identifier: Apache-2.0
//! `check --fix` reaches a path a language regime lists outside the corpus root
//! (HW-DR-0084 clause 6).
//!
//! The fixer reads the patch a finding carries, applies it, and parses the
//! result again before it writes anything. A file outside the root usually
//! opens with no front matter, and a read-back that required a block would
//! refuse every patch to one. This test reads the check crate's `outside-root`
//! tree, where `README.md` has one contraction and no block.

use headwater_census::census;
use headwater_census::outside;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::shape::Shape;
use headwater_check::{Cache, Context, Date, Declared, Register};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_scaffold::fix;
use std::path::{Path, PathBuf};

const PINNED: &str = "2026-09-27";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../check/fixtures")
}

#[test]
fn a_contraction_outside_the_root_is_fixed_and_the_read_back_holds() {
    let base = fixtures_dir().join("outside-root");
    let source = std::fs::read_to_string(fixtures_dir().join("outside-root.taxonomy.yml"))
        .expect("the fixture taxonomy");
    let root = headwater_yaml::load(&source)
        .expect("the fixture taxonomy loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();
    let taxonomy = Taxonomy::read(&root).expect("the shelves read");
    let declarations = Declarations::read(&root).expect("the relations read");
    let register = Register::read(&root).expect("the register reads");
    let shape = Shape::read(&root).expect("the shape reads");
    let corpus = Corpus::new(base.clone(), "docs");
    let mut taken = census::take(&corpus, &taxonomy);
    taken.outside = outside::take(&corpus, &shape.outside_root());
    let config = Config::default();
    let graph = Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(&corpus),
        &corpus,
        &config,
    );
    let run = headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: "sha256:outside-root-fixture",
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            observations: &headwater_check::Observations::empty(),
            adoption: None,
            source: "engine/crates/check/fixtures/outside-root.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    );

    let patches: Vec<_> = run
        .findings
        .iter()
        .filter(|finding| finding.path == "README.md")
        .filter_map(|finding| finding.patch.clone())
        .collect();
    assert_eq!(patches.len(), 1, "the contraction carries one patch");

    let composed = fix::compose(&base, &patches);
    assert!(
        composed.refused.is_empty(),
        "the fixer refuses nothing: {:?}",
        composed.refused
    );
    assert_eq!(composed.files.len(), 1);
    assert_eq!(composed.files[0].path, "README.md");
    assert!(
        composed.files[0]
            .text
            .contains("This file does not sit under the corpus root."),
        "the patched text:\n{}",
        composed.files[0].text
    );
}
