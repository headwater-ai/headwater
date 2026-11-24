// SPDX-License-Identifier: Apache-2.0
//! What a `relation_view` declaration may state, read over a lock (#2018,
//! HW-DR-0114).
//!
//! Clause 2 of #2018: the engine takes `relations` on a `relation_view` and on
//! no other kind, and it refuses a chain whose steps do not join. Each case
//! below is one refusal, beside the declaration that it would accept, so that a
//! reader that refused everything fails as surely as one that refused nothing.

use headwater_generate::relation_view::{Format, View};
use headwater_generate::{Kind, Projections};

/// A small taxonomy root: the three kinds of the verification chain, an
/// abstract parent, two anchors and the relations between them, as the
/// `requirement-verification` entry declares them.
const TAXONOMY: &str = "\
kinds:
  governed_document: {abstract: true}
  requirement: {is_a: governed_document}
  acceptance_criterion: {is_a: governed_document}
  verification: {is_a: governed_document}
anchors:
  code_path: {resolver: source-tree}
  check_rule: {resolver: check-rule}
  test_site: {resolver: comment-scan}
relations:
  verified_by: {from: [requirement], to: [acceptance_criterion, check_rule]}
  proven_by: {from: [acceptance_criterion], to: [verification]}
  cited_in: {from: [verification], to: [test_site]}
  traces_to: {from: [governed_document], to: [governed_document, code_path]}
  governs: {from: [governed_document], to: [code_path]}
shelves:
  verified_requirements: {path: docs/requirements/**, kind: requirement}
  verifications: {path: docs/verifications/**, kind: verification}
";

fn read(projections: &str) -> Result<Projections, Vec<String>> {
    let source = format!("{TAXONOMY}projections:\n{projections}");
    let root = headwater_yaml::load(&source)
        .expect("it loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();
    Projections::read(&root).map_err(|errors| {
        errors
            .into_iter()
            .map(|error| error.message)
            .collect::<Vec<_>>()
    })
}

fn view(relations: &str, extra: &str) -> String {
    format!(
        "  - kind: relation_view\n    for: [verified_requirements]\n    relations: [{relations}]\n    output: docs/matrix.md\n{extra}"
    )
}

fn refused(projections: &str, says: &str) {
    match read(projections) {
        Ok(_) => panic!("accepted, and it should be refused for `{says}`:\n{projections}"),
        Err(errors) => assert!(
            errors.iter().any(|error| error.contains(says)),
            "refused, and no refusal says `{says}`: {errors:?}"
        ),
    }
}

#[test]
fn the_chain_of_the_ruling_reads_and_states_its_steps_and_its_format() {
    let read = read(&view("verified_by, proven_by, cited_in", "")).expect("it reads");
    let [declaration] = read.declared.as_slice() else {
        panic!("one declaration: {:?}", read.declared);
    };
    assert_eq!(declaration.kind, Kind::RelationView);
    assert_eq!(
        declaration.view,
        Some(View {
            relations: vec![
                "verified_by".to_string(),
                "proven_by".to_string(),
                "cited_in".to_string()
            ],
            format: Format::Markdown,
        })
    );
}

/// `relations` on a `shelf_index` is refused, and the same entry without it
/// reads.
#[test]
fn relations_on_another_kind_is_refused() {
    let without =
        "  - kind: shelf_index\n    for: [verifications]\n    output: \"{shelf}/README.md\"\n";
    assert!(read(without).is_ok(), "{:?}", read(without).err());
    refused(
        &format!("{without}    relations: [proven_by]\n"),
        "only a `relation_view` takes `relations`",
    );
}

#[test]
fn a_view_with_no_relations_or_not_one_shelf_is_refused() {
    refused(
        "  - kind: relation_view\n    for: [verified_requirements]\n    output: docs/matrix.md\n",
        "states no `relations`",
    );
    refused(
        "  - kind: relation_view\n    relations: [verified_by]\n    output: docs/matrix.md\n",
        "names 0 shelves",
    );
    refused(
        "  - kind: relation_view\n    for: [verified_requirements, verifications]\n    relations: [verified_by]\n    output: docs/matrix.md\n",
        "names 2 shelves",
    );
}

#[test]
fn a_step_that_names_no_declared_relation_is_refused() {
    refused(
        &view("verified_by, proven_bye", ""),
        "names `proven_bye`, and no relation of that name is declared",
    );
}

/// `verified_by` reaches a criterion and `cited_in` runs from a verification,
/// so no path can take the second step. `verified_by` then `proven_by` joins.
#[test]
fn two_steps_that_do_not_join_are_refused() {
    assert!(read(&view("verified_by, proven_by", "")).is_ok());
    refused(
        &view("verified_by, cited_in", ""),
        "names `verified_by` and then `cited_in`, and they do not join",
    );
}

/// An abstract endpoint joins every kind below it, in both directions:
/// `traces_to` runs from `governed_document`, and a verification is one, and
/// `traces_to` reaches `governed_document`, which may be the criterion that
/// `proven_by` runs from.
#[test]
fn a_step_joins_through_the_kind_a_target_inherits_from() {
    for chain in [
        "verified_by, proven_by, traces_to",
        "verified_by, traces_to, proven_by",
    ] {
        assert!(
            read(&view(chain, "")).is_ok(),
            "{chain}: {:?}",
            read(&view(chain, "")).err()
        );
    }
}

/// A step that reaches only code ends every path, so it is the last step.
/// `governs` last reads, and `governs` before another step is refused.
#[test]
fn a_code_step_before_the_last_step_is_refused() {
    assert!(read(&view("verified_by, proven_by, governs", "")).is_ok());
    refused(
        &view("verified_by, proven_by, governs, traces_to", ""),
        "names `governs` at step 3, and it reaches only code",
    );
}

/// The first step runs from the kind the start shelf holds.
#[test]
fn a_start_shelf_that_does_not_join_the_first_step_is_refused() {
    refused(
        &view("proven_by", ""),
        "starts from the shelf `verified_requirements`, which holds `requirement`",
    );
}

/// A view reads its own `format`, and the output's extension is the format's.
/// `csv` belongs to a view alone.
#[test]
fn the_format_and_the_extension_of_a_view_agree_and_csv_is_a_view_s_alone() {
    let csv = "  - kind: relation_view\n    for: [verified_requirements]\n    relations: [verified_by]\n    output: docs/matrix.csv\n    format: csv\n";
    let read_csv = read(csv).expect("a CSV view reads");
    assert_eq!(
        read_csv.declared[0].view.as_ref().map(|view| view.format),
        Some(Format::Csv)
    );
    refused(
        &view("verified_by", "    format: csv\n"),
        "writes a file ending in `.csv`",
    );
    refused(
        &view("verified_by", "    format: html\n"),
        "`markdown`, `csv` or `json`",
    );
    refused(
        "  - kind: shelf_index\n    for: [verifications]\n    output: docs/verifications/README.csv\n",
        "only a `relation_view` writes a `.csv` file",
    );
}

/// A view's `format: json` is the view's own JSON, and never the export
/// emitter of the same name, so the entry joins no emitter set.
#[test]
fn a_json_view_is_not_an_export_emitter() {
    let json = "  - kind: relation_view\n    for: [verified_requirements]\n    relations: [verified_by]\n    output: docs/matrix.json\n    format: json\n";
    let read = read(json).expect("a JSON view reads");
    assert_eq!(read.declared[0].membership.emitter, None);
    assert_eq!(
        read.declared[0].view.as_ref().map(|view| view.format),
        Some(Format::Json)
    );
}
