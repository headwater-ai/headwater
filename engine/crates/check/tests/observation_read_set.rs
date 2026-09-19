// SPDX-License-Identifier: Apache-2.0
//! The decisive fixture behind veto 1, item 1: `.headwater/observations.yml`
//! belonged to no instance, so it was in no read set, so `headwater gate`
//! carried a verdict across an edit to it that flipped the obligation it
//! feeds.
//!
//! Repro, run once as a probe before this fixture existed: `check
//! --read-set r` on a tree with an entry for `CT-FIX-16`, delete the file,
//! `gate --read-set r` on the same tree afterward. The verdict carried while
//! `OB-FIX-19` had moved from `verified` to unobserved. This file is that
//! probe, kept.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Observations, Register, Run, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the copy directory");
    for entry in std::fs::read_dir(from).expect("the fixture tree reads") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("a file type").is_dir() {
            true => copy(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), target).expect("a fixture copies");
            }
        }
    }
}

/// A private copy of the fixture tree, so this file's writes to
/// `.headwater/observations.yml` never touch the tree other tests share.
fn corpus_for(case: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(case);
    match std::fs::remove_dir_all(&root) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("{} will not clear: {error}", root.display()),
    }
    copy(&fixtures_dir().join("check"), &root.join("check"));
    root
}

fn run_over(root: &Path) -> Run {
    let source = std::fs::read_to_string(fixtures_dir().join("check.taxonomy.yml"))
        .expect("the fixture taxonomy");
    let loaded = headwater_yaml::load(&source).expect("it loads");
    let declared = loaded.value.as_map().expect("a mapping");
    let lock = headwater_hash::hex(source.as_bytes());
    let corpus = Corpus::new(root, "check");
    let taxonomy = Taxonomy::read(declared).expect("the taxonomy reads");
    let declarations = Declarations::read(declared).expect("the declarations read");
    let register = Register::read(declared).expect("the register reads");
    let shape = Shape::read(declared).expect("the shape reads");
    let taken = census::take(&corpus, &taxonomy);
    let config = Config::default();
    let graph = Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(&corpus),
        &corpus,
        &config,
    );
    let observations = Observations::at(root);
    headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: &lock,
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            observations: &observations,
            adoption: None,
            source: "engine/crates/check/tests/observation_read_set.rs",
        },
        &headwater_check::claim::Claims::at(root),
        &Context::at(Date::parse("2026-08-12").expect("a date")),
        &mut Cache::disabled(),
    )
}

fn digest_of(root: &Path, listed: &str) -> Option<String> {
    if listed == headwater_check::claim::STORE {
        return Some(headwater_check::claim::Claims::at(root).digest());
    }
    std::fs::read(root.join(listed))
        .ok()
        .map(|bytes| headwater_hash::digest(&bytes))
}

/// A present, parsable snapshot joins the read set with a digest a gate can
/// use, the same way the claim store does.
#[test]
fn a_present_snapshot_joins_the_read_set() {
    let root = corpus_for("read-set-present");
    std::fs::create_dir_all(root.join(".headwater")).expect("the dir");
    std::fs::write(
        root.join(".headwater/observations.yml"),
        "CT-FIX-16:\n  commit: 788885a9\n",
    )
    .expect("the fixture writes");
    let run = run_over(&root);
    let input = run
        .read_set
        .inputs
        .iter()
        .find(|input| input.path == headwater_check::observation::PATH)
        .expect("the snapshot is in the read set when it is present");
    assert!(
        input.digest.is_some(),
        "a parsable snapshot carries a digest a gate can compare"
    );
}

/// An absent snapshot is not an input this run read, so it is not in the read
/// set at all: see `observation::Observations::read_set_digest`'s own
/// argument for why "absent" and "unhashed" are not the same report.
#[test]
fn an_absent_snapshot_is_not_in_the_read_set() {
    let root = corpus_for("read-set-absent");
    let run = run_over(&root);
    assert!(
        !run.read_set
            .inputs
            .iter()
            .any(|input| input.path == headwater_check::observation::PATH),
        "an absent file is not an input this run read"
    );
}

/// The decisive fixture: a control's snapshot entry deleted between the run
/// that published a read set and the tree a gate reads it against flips
/// `OB-FIX-19` from `verified` to unobserved, and the gate must say the
/// verdict does not carry — before the read-set fix, it said the verdict
/// carried, because `.headwater/observations.yml` was not a component of the
/// set at all.
#[test]
fn deleting_the_snapshot_entry_between_check_and_gate_does_not_carry() {
    let root = corpus_for("read-set-deleted");
    std::fs::create_dir_all(root.join(".headwater")).expect("the dir");
    std::fs::write(
        root.join(".headwater/observations.yml"),
        "CT-FIX-16:\n  commit: 788885a9\n",
    )
    .expect("the fixture writes");

    let run = run_over(&root);
    let disposed = run
        .register
        .obligations
        .iter()
        .find(|o| o.id == "OB-FIX-19")
        .expect("the fixture obligation");
    assert_eq!(
        disposed.disposition(),
        headwater_check::register::Disposition::Verified,
        "the snapshot names CT-FIX-16, so this run reads OB-FIX-19 as verified"
    );

    let recorded = headwater_check::gate::Recorded::parse(&run.read_set.render())
        .expect("the rendered read set parses back");

    // Tree B: the entry is gone.
    std::fs::remove_file(root.join(".headwater/observations.yml")).expect("the file removes");

    let verdict = headwater_check::gate::decide(
        &recorded,
        &headwater_hash::hex(
            std::fs::read_to_string(fixtures_dir().join("check.taxonomy.yml"))
                .expect("the fixture taxonomy")
                .as_bytes(),
        ),
        Date::parse("2026-08-12").expect("a date"),
        |listed| digest_of(&root, listed),
    );
    assert!(
        !verdict.carries(),
        "a snapshot deleted after the run that read it must not carry: {}",
        verdict.render()
    );
}
