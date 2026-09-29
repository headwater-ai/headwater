// SPDX-License-Identifier: Apache-2.0
//! A relation entry that names the document that declares it, and the ways a
//! rule about it is easy to get wrong.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship)
//! sets the floor: "every check ships with at least one fixture that it fails
//! and one that it passes." The tree under `fixtures/self-target/` is
//! [#1232](https://github.com/headwater-ai/headwater/issues/1232).
//!
//! **A rule that trusted the endpoint check** reports nothing, because the
//! fixture relation goes from `explanation` to `explanation` and both ends of a
//! self-edge are admitted.
//!
//! **A rule that compared the target with a different document** reports
//! `notes/b.md`, which draws on `notes/a.md`.
//!
//! **A rule that read only the declared half** misses `notes/d.md`, which
//! writes the inverse name onto itself.
//!
//! **A rule that reported an unbound target** reports `notes/c.md` a second
//! time. It names itself by path, and a target is an identifier and never a
//! path, so the entry binds to nothing and `relation.target.unresolved` owns it.
//!
//! **A rule that read "no relation" where spec 2 reads "no family except
//! association"** reports `notes/e.md`, which relates to itself under an
//! association relation.
//!
//! **A rule that exempted every relation outside a named family** misses
//! `notes/f.md`, whose relation declares no family. Only a declared
//! `association` is exempt.
//!
//! **A rule that compared only documents** misses `notes/g.md`, which governs
//! its own file through a `code_path` anchor whose one literal pattern is that
//! file. The owner ruled on #1350 that this exact case is a self-reference.
//!
//! **A rule that compared the path as the author spelled it** misses
//! `notes/k.md`, which writes its own path with `./` and a `..` segment. The
//! resolver normalizes the pattern, and the rule compares the normalized one.
//!
//! **A rule that reported any anchor** reports `notes/h.md`, whose one literal
//! pattern names another file.
//!
//! **A rule that compared the matched set with the own file** reports
//! `notes/i.md`, whose wildcard matches its own file among others, and
//! `notes/j.md`, whose list holds its own file and another. The ruling reaches
//! the exact own file and nothing wider.
//!
//! **A rule whose instances were its findings** reports a denominator of five.
//! Every entry outside the association family is an instance, so the count is
//! ten.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Register, Run, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

/// As every other recorded run: a verdict is a function of the injected clock.
const PINNED: &str = "2026-08-12";

const RULE: &str = headwater_check::self_target::RULE;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn run() -> Run {
    let corpus = Corpus::new(fixtures_dir(), "self-target");
    let source = std::fs::read_to_string(fixtures_dir().join("self-target.taxonomy.yml"))
        .expect("the fixture taxonomy");
    let root = headwater_yaml::load(&source)
        .expect("the fixture taxonomy loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();

    let taxonomy = Taxonomy::read(&root).expect("the taxonomy reads");
    let declarations = Declarations::read(&root).expect("the declarations read");
    let register = Register::read(&root).expect("the register reads");
    let shape = Shape::read(&root).expect("the shape reads");
    let taken = census::take(&corpus, &taxonomy);
    let config = Config::default();
    let graph = Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(&corpus),
        &corpus,
        &config,
    );
    headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: "sha256:self-target-fixture",
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            observations: &headwater_check::Observations::empty(),
            pin: None,
            adoption: None,
            source: "engine/crates/check/fixtures/self-target.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(PINNED).expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

/// The messages of one rule at one file of the tree.
fn at<'a>(run: &'a Run, rule: &str, file: &str) -> Vec<&'a str> {
    let path = format!("self-target/{file}");
    run.findings
        .iter()
        .filter(|finding| finding.rule == rule && finding.path == path)
        .map(|finding| finding.message.as_str())
        .collect()
}

/// The decisive case: the entry that names its own document is reported, once,
/// with the relation and the document named, and the entry that names another
/// document is not.
#[test]
fn a_document_that_draws_on_itself_is_reported_and_one_that_draws_on_another_is_not() {
    let run = run();

    let found = at(&run, RULE, "notes/a.md");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("draws_on"), "{}", found[0]);
    assert!(found[0].contains("NOTE-FIX-a"), "{}", found[0]);

    assert!(at(&run, RULE, "notes/b.md").is_empty());

    let rule = run
        .findings
        .iter()
        .find(|finding| finding.rule == RULE)
        .expect("a finding");
    assert_eq!(rule.severity, headwater_check::Severity::Error);
    assert!(
        rule.patch.is_none(),
        "only the author knows which target was meant"
    );
}

/// The inverse half is an entry an author wrote, and it names its own document
/// just as the declared half does.
#[test]
fn the_inverse_half_written_onto_its_own_document_is_reported() {
    let run = run();
    let found = at(&run, RULE, "notes/d.md");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("drawn_on_by"), "{}", found[0]);
    assert!(found[0].contains("NOTE-FIX-d"), "{}", found[0]);
}

/// A path that spells the document's own file binds to nothing, and the rule
/// that owns an unbound target reports it. This rule does not report it again.
#[test]
fn a_self_path_that_binds_to_nothing_is_the_unresolved_rules_finding_alone() {
    let run = run();
    assert!(at(&run, RULE, "notes/c.md").is_empty());
    assert_eq!(
        at(&run, headwater_check::target::RULE, "notes/c.md").len(),
        1
    );
}

/// A relation that declares no family is not in the association family, so a
/// self-edge on it is reported like any other.
#[test]
fn a_relation_with_no_declared_family_that_points_at_its_own_document_is_reported() {
    let run = run();
    let found = at(&run, RULE, "notes/f.md");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("leans_on"), "{}", found[0]);
    assert!(found[0].contains("NOTE-FIX-f"), "{}", found[0]);
}

/// Spec 2 lets an association relation point at its own document, so that
/// entry is no instance and no finding.
#[test]
fn an_association_relation_that_points_at_its_own_document_is_not_reported() {
    let run = run();
    assert!(at(&run, RULE, "notes/e.md").is_empty());
    assert!(!run.instances.iter().any(|instance| instance.rule == RULE
        && instance
            .reads
            .iter()
            .any(|input| input.path == "self-target/notes/e.md")));
}

/// Every entry is an instance, whatever its verdict, so the rule's count is its
/// denominator and not its findings.
#[test]
fn every_entry_is_an_instance() {
    let run = run();
    let instances = run
        .instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .count();
    assert_eq!(instances, 10);
    let findings = run
        .findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .count();
    assert_eq!(findings, 5);
}

/// The decisive case of #1350: a `code_path` anchor whose one literal pattern
/// is the declaring document's own file is reported, once, with the relation
/// and the file named. A wildcard that matches the same file is not.
#[test]
fn an_anchor_onto_the_declaring_file_is_reported_and_a_wildcard_over_it_is_not() {
    let run = run();

    let found = at(&run, RULE, "notes/g.md");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("governs"), "{}", found[0]);
    assert!(found[0].contains("self-target/notes/g.md"), "{}", found[0]);
    assert!(found[0].contains("its own file"), "{}", found[0]);

    let finding = run
        .findings
        .iter()
        .find(|finding| finding.rule == RULE && finding.path == "self-target/notes/g.md")
        .expect("the finding");
    assert!(
        finding.patch.is_none(),
        "only the author knows the file meant"
    );
    assert!(
        finding.remediation.contains("file"),
        "{}",
        finding.remediation
    );

    assert!(at(&run, RULE, "notes/i.md").is_empty());
}

/// The rule compares the pattern the resolver normalized, not the spelling the
/// author wrote: an own path written with `./` and a `..` segment is the same
/// file, and it is reported.
#[test]
fn an_own_file_anchor_spelled_in_a_non_canonical_form_is_reported() {
    let run = run();
    let found = at(&run, RULE, "notes/k.md");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("self-target/notes/k.md"), "{}", found[0]);
    assert!(at(&run, headwater_check::target::RULE, "notes/k.md").is_empty());
}

/// A literal anchor onto another file, and a list that holds the own file among
/// other paths, pass: the ruling reaches the exact own file and nothing wider.
#[test]
fn an_anchor_onto_another_file_or_a_list_that_holds_the_own_file_is_not_reported() {
    let run = run();
    assert!(at(&run, RULE, "notes/h.md").is_empty());
    assert!(at(&run, RULE, "notes/j.md").is_empty());
    // Each of them still binds, so the pass is this rule's and not a target
    // that never resolved.
    for file in ["notes/g.md", "notes/h.md", "notes/i.md", "notes/j.md"] {
        assert!(
            at(&run, headwater_check::target::RULE, file).is_empty(),
            "{file}: {:?}",
            at(&run, headwater_check::target::RULE, file)
        );
    }
}
