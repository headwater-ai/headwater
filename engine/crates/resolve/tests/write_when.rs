// SPDX-License-Identifier: Apache-2.0
//! What `taxonomy validate` refuses about `kinds.<name>.write_when` (#1580).
//!
//! The member is one sentence that tells a writer when to write a document of
//! the kind. A blank sentence tells a writer nothing, and it is worse than no
//! member: `headwater taxonomy kinds` would print an empty line where it
//! prints the gap sentence for a kind that declares none, and a child that
//! wrote it would hide its ancestor's sentence. So `purpose completeness`
//! refuses it, as `identifier integrity` refuses a namespace written as the
//! empty string.

use headwater_resolve::{resolve, Role, Source};

/// The least a source needs to resolve, with `{kinds}` replaced per case.
const BASE: &str = "\
taxonomy: acme/write-when
version: 1.0.0

facets:
  summary:
    role: scent
    required: true
    volatility: mutable

purposes:
  guide: {intent: walk a reader through one task}

identifier_schemes:
  note_id: {pattern: \"NOTE-{namespace}-{slug}\", namespace: FIX, allocation: minted-once}

kinds:
  page:
    abstract: true
    write_when: a reader needs a page
{kinds}
shelves:
  notes:
    path: notes/**
    homogeneous: true
    kind: note
";

/// The messages a source is refused with.
fn refusals(kinds: &str) -> Vec<String> {
    let text = BASE.replace("{kinds}\n", kinds);
    let source = Source::from_text("write-when.yml", Role::Taxonomy, &text)
        .expect("the source loads and the meta-schema accepts it");
    let resolution = resolve(&[source]).expect("the source resolves");
    resolution
        .validate()
        .iter()
        .map(|error| headwater_resolve::render_errors(std::slice::from_ref(error)))
        .collect()
}

/// A kind that writes a sentence, and a kind that writes none, are accepted.
/// This is the denominator for the refusals below.
#[test]
fn a_sentence_and_no_member_are_both_accepted() {
    let declared = "  note:
    is_a: page
    purpose: guide
    write_when: an operator must repair the widget line
    identifier: {scheme: note_id}
";
    assert!(refusals(declared).is_empty(), "{:#?}", refusals(declared));
    let inherited = "  note:
    is_a: page
    purpose: guide
    identifier: {scheme: note_id}
";
    assert!(refusals(inherited).is_empty(), "{:#?}", refusals(inherited));
}

/// The empty string and a run of spaces are each refused, by name and with
/// the remedy.
#[test]
fn a_blank_sentence_is_refused() {
    for blank in ["\"\"", "\"   \""] {
        let kinds = format!(
            "  note:
    is_a: page
    purpose: guide
    write_when: {blank}
    identifier: {{scheme: note_id}}
"
        );
        let refusals = refusals(&kinds);
        assert_eq!(refusals.len(), 1, "{blank}: {refusals:#?}");
        let message = &refusals[0];
        assert!(message.contains("kinds.note.write_when"), "{message}");
        assert!(message.contains("purpose completeness"), "{message}");
        assert!(message.contains("is blank"), "{message}");
        assert!(message.contains("remove the member"), "{message}");
    }
}

/// An abstract kind is held to the same rule, because its sentence is what
/// each kind under it prints.
#[test]
fn a_blank_sentence_on_an_abstract_kind_is_refused() {
    let text = BASE.replace(
        "    write_when: a reader needs a page\n",
        "    write_when: \"\"\n",
    );
    let source = Source::from_text(
        "write-when.yml",
        Role::Taxonomy,
        &text.replace(
            "{kinds}\n",
            "  note:
    is_a: page
    purpose: guide
    identifier: {scheme: note_id}
",
        ),
    )
    .expect("the source loads");
    let refusals: Vec<String> = resolve(&[source])
        .expect("the source resolves")
        .validate()
        .iter()
        .map(|error| headwater_resolve::render_errors(std::slice::from_ref(error)))
        .collect();
    assert_eq!(refusals.len(), 1, "{refusals:#?}");
    assert!(
        refusals[0].contains("kinds.page.write_when"),
        "{refusals:#?}"
    );
}
