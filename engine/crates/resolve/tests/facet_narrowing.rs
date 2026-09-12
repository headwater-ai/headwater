// SPDX-License-Identifier: Apache-2.0
//! What `taxonomy validate` refuses about `kinds.<name>.facets.values`.
//!
//! [HW-DR-0066](../../../../docs/decisions/0066-a-kind-narrows-the-value-set-of-an-enumerated-facet-and-nothing-else-can.md)
//! rules that a kind names the values of an enumerated facet it means, and that
//! a narrowing takes values away and adds none. Four declarations say something
//! the facet's own declaration does not hold, and `kind inheritance` refuses
//! each one.
//!
//! This file exists because the member shipped once without it. The check layer
//! carried the whole of the new behavior and the only `facets.values` in the
//! workspace was a check fixture, so nothing ran the resolver over one. The
//! fixture that resulted named a value its parent excluded: the check layer
//! read it as an intersection and reported an answer, and the resolver would
//! have refused the taxonomy that produced it. Two enforcement paths that
//! disagree about one file is the defect a rule with no fixture always has, and
//! [spec 12](../../../../docs/spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship)
//! names it for the check layer.
//!
//! Every case below asserts the refusal's **text** and not only its count. A
//! message is what an adopter reads to learn what to write instead, nothing
//! else in `engine/` asserts one of these five, and the first version of all
//! five carried a run of twenty-six spaces from a wrap that lost its backslash
//! — invisible to `cargo fmt --check`, which does not format a string literal.

use headwater_resolve::{resolve, Role, Source};

/// The taxonomy every case narrows, with `{kinds}` replaced per case.
///
/// Everything outside the kinds is the least a source needs to resolve: one
/// enumerated facet, one facet that enumerates nothing, one purpose, one
/// identifier scheme, and one shelf per concrete kind. A case that added a
/// declaration of its own would be a case about two things.
const BASE: &str = "\
taxonomy: acme/facet-narrowing
version: 1.0.0

facets:
  evidence_basis:
    required: true
    values: [measured, cited, asserted, observed]
    volatility: stable
    guidance:
      measured: a number this corpus produced, with the command that produced it
      cited: a claim attributed to a source outside this corpus
      asserted: a claim the author stands behind and did not measure
      observed: something seen once and not repeated
  summary:
    role: scent
    required: true
    volatility: mutable

purposes:
  hold_evidence: {intent: state what stands behind a claim}

identifier_schemes:
  note_id: {pattern: \"NOTE-{namespace}-{slug}\", namespace: FIX, allocation: minted-once}

kinds:
{kinds}
shelves:
  notes:
    path: notes/**
    homogeneous: true
    kind: note
";

/// The messages `kind inheritance` refuses a source with, in the order the
/// rules ran.
///
/// A source that does not load, or that the meta-schema refuses, fails here
/// rather than returning an empty list, because an empty list is what a passing
/// case looks like and the two must not be one answer.
fn refusals(kinds: &str) -> Vec<String> {
    let text = BASE.replace("{kinds}\n", kinds);
    let source = Source::from_text("facet-narrowing.yml", Role::Taxonomy, &text)
        .expect("the source loads and the meta-schema accepts it");
    let resolution = resolve(&[source]).expect("the source resolves");
    resolution
        .validate()
        .iter()
        .map(|error| headwater_resolve::render_errors(std::slice::from_ref(error)))
        .collect()
}

/// The one message of a case that refuses exactly once.
fn only(kinds: &str) -> String {
    let refusals = refusals(kinds);
    assert_eq!(refusals.len(), 1, "{refusals:#?}");
    refusals.into_iter().next().expect("the one refusal")
}

const BASELINE: &str = "  note:
    purpose: hold_evidence
    identifier: {scheme: note_id}
    facets:
      require: [evidence_basis, summary]
      values:
        evidence_basis: [measured, cited]
";

/// A narrowing that says nothing the facet does not is accepted, and the file
/// that says so is the denominator for every refusal below.
///
/// Without it, a rule that refused every `facets.values` whatever it held would
/// pass all four refusal cases.
#[test]
fn a_narrowing_inside_the_declared_set_is_accepted() {
    assert!(refusals(BASELINE).is_empty(), "{:#?}", refusals(BASELINE));
}

/// A narrowing on a facet that declares no value set is refused.
///
/// The remedy is not to add a value. There is nothing to narrow, so the
/// declaration is a statement about a set that does not exist, and a rule that
/// treated it as a no-op would accept a source whose author believed it did
/// something.
#[test]
fn a_narrowing_on_a_facet_that_enumerates_nothing_is_refused() {
    let message = only(
        "  note:
    purpose: hold_evidence
    identifier: {scheme: note_id}
    facets:
      require: [evidence_basis, summary]
      values:
        summary: [anything]
",
    );
    assert!(message.contains("kinds.note.facets.values"), "{message}");
    assert!(
        message.contains(
            "narrows `summary`, which declares no value set. A narrowing names values of a \
             closed set, and a facet that enumerates nothing has none to name"
        ),
        "{message}"
    );
}

/// A narrowing to the empty list is refused, and the refusal names the member
/// that does mean "not on this kind".
///
/// An empty list would admit no value, so no document of the kind could be
/// valid, and the check layer would report every one of them against a
/// remediation that named nothing.
#[test]
fn a_narrowing_to_no_value_at_all_is_refused_and_names_forbid() {
    let message = only(
        "  note:
    purpose: hold_evidence
    identifier: {scheme: note_id}
    facets:
      require: [evidence_basis, summary]
      values:
        evidence_basis: []
",
    );
    assert!(
        message.contains(
            "narrows `evidence_basis` to no value at all, so no document of this kind could \
             ever be valid. `facets.forbid` is how a kind takes a facet away"
        ),
        "{message}"
    );
}

/// A narrowing on a facet the chain forbids is refused.
///
/// `facets.forbid` removes the facet, so there are no values of it on this kind
/// to name. The prohibition is inherited, which is why the case puts the
/// `forbid` on the parent and the narrowing on the child.
#[test]
fn a_narrowing_on_a_facet_an_ancestor_forbids_is_refused() {
    let refusals = refusals(
        "  holder:
    abstract: true
    purpose: hold_evidence
    identifier: {scheme: note_id}
    facets:
      forbid: [evidence_basis]
  note:
    is_a: holder
    identifier: {scheme: note_id}
    facets:
      values:
        evidence_basis: [measured]
",
    );
    let named: Vec<&String> = refusals
        .iter()
        .filter(|message| message.contains("facets.values"))
        .collect();
    assert_eq!(named.len(), 1, "{refusals:#?}");
    assert!(
        named[0].contains(
            "narrows `evidence_basis`, and this kind or one above it forbids the same facet. \
             A forbidden facet has no values on this kind to narrow"
        ),
        "{}",
        named[0]
    );
}

/// A narrowing that names a value the facet does not declare is refused.
///
/// The direction that matters: a narrowing takes values away and it adds none,
/// so a name outside the declared set is a widening written as a narrowing.
#[test]
fn a_narrowing_that_names_a_value_the_facet_does_not_declare_is_refused() {
    let message = only(
        "  note:
    purpose: hold_evidence
    identifier: {scheme: note_id}
    facets:
      require: [evidence_basis, summary]
      values:
        evidence_basis: [measured, inferred]
",
    );
    assert!(
        message.contains(
            "narrows `evidence_basis` to `inferred`, which `facets.evidence_basis.values` does \
             not hold. A narrowing takes values away and it adds none"
        ),
        "{message}"
    );
    assert!(
        !message.contains("`measured`"),
        "the refusal names a value that is in the set: {message}"
    );
}

/// A child that names a value an ancestor's narrowing excluded is refused, and
/// the refusal names the ancestor.
///
/// The case the record turns on. Spec 2 rules that a child may not un-require
/// what a parent requires, because that voids the parent's contract for a
/// reader who trusts it, and a wider value set voids the same contract the same
/// way. `asserted` is a value of the facet, so the previous rule passes it and
/// only this one catches it.
#[test]
fn a_child_that_widens_what_an_ancestor_narrowed_is_refused() {
    let message = only(
        "  holder:
    abstract: true
    purpose: hold_evidence
    identifier: {scheme: note_id}
    facets:
      require: [evidence_basis, summary]
      values:
        evidence_basis: [measured, cited]
  note:
    is_a: holder
    identifier: {scheme: note_id}
    facets:
      values:
        evidence_basis: [cited, asserted]
",
    );
    assert!(
        message.contains(
            "narrows `evidence_basis` to `asserted`, and `holder` above it does not admit that \
             value. A child may not void a contract that a reader of the parent trusts"
        ),
        "{message}"
    );
    assert!(
        !message.contains("`cited`"),
        "the refusal names the value both kinds admit: {message}"
    );
}

/// A child that narrows further inside what its ancestor admits is accepted.
///
/// The converse of the case above, and the one that says the rule refuses a
/// widening rather than every child narrowing. It is also the shape
/// `engine/crates/check/fixtures/facet-values.taxonomy.yml` uses, so this is
/// where that fixture is held to resolving.
#[test]
fn a_child_that_narrows_further_inside_its_ancestors_set_is_accepted() {
    let refusals = refusals(
        "  holder:
    abstract: true
    purpose: hold_evidence
    identifier: {scheme: note_id}
    facets:
      require: [evidence_basis, summary]
      values:
        evidence_basis: [measured, cited]
  note:
    is_a: holder
    identifier: {scheme: note_id}
    facets:
      values:
        evidence_basis: [cited]
",
    );
    assert!(refusals.is_empty(), "{refusals:#?}");
}

/// The fixture the check layer runs against resolves and validates.
///
/// The one assertion that makes the two enforcement paths one. Without it, the
/// check crate can assert a verdict over a taxonomy `taxonomy validate` would
/// refuse, which is what it did before this file existed.
#[test]
fn the_check_layers_own_facet_values_fixture_validates() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../check/fixtures/facet-values.taxonomy.yml");
    let text = std::fs::read_to_string(&path).expect("the check crate's fixture taxonomy");

    // The fixture is a taxonomy body with no package header, because the check
    // crate reads it with `Shape::read` and never through a source. Two lines
    // make it one a resolver will take, and nothing else about it moves.
    let source = Source::from_text(
        "facet-values.taxonomy.yml",
        Role::Taxonomy,
        &format!("taxonomy: acme/facet-values\nversion: 1.0.0\n{text}"),
    )
    .expect("the fixture loads and the meta-schema accepts it");
    let resolution = resolve(&[source]).expect("the fixture resolves");
    let refusals = resolution.validate();
    assert!(
        refusals.is_empty(),
        "the corpus the check crate asserts verdicts over does not validate:\n{}",
        headwater_resolve::render_errors(&refusals)
    );
}
