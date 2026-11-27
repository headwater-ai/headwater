// SPDX-License-Identifier: Apache-2.0

//! Which block a dangling name would be declared in (HW-OBL-0032).
//!
//! `Resolution::validate` blames a `remove` for an orphan only when the
//! `remove` covers the address that would declare the name. That address is
//! read off the sort of thing the position reads, and one wrong row blames a
//! `remove` that did not cause the orphan, or misses one that did. The
//! `validate/` fixtures reach the `kind`, `kind or anchor kind` and `facet`
//! rows through a whole resolution. This table holds every row that
//! `dangling_names` writes.

use headwater_resolve::rules::Dangling;

fn declarations(what: &str) -> Vec<String> {
    Dangling {
        at: "somewhere.reads".to_string(),
        what: what.to_string(),
        name: "x".to_string(),
    }
    .declarations()
    .into_iter()
    .map(|path| path.join("."))
    .collect()
}

#[test]
fn each_sort_of_name_is_declared_in_its_own_block() {
    let table: &[(&str, &[&str])] = &[
        ("kind", &["kinds.x"]),
        ("anchor kind", &["anchors.x"]),
        ("kind or anchor kind", &["kinds.x", "anchors.x"]),
        ("facet", &["facets.x"]),
        ("purpose", &["purposes.x"]),
        ("identifier scheme", &["identifier_schemes.x"]),
        ("shelf", &["shelves.x"]),
        ("relation", &["relations.x"]),
        ("relation or declared inverse", &["relations.x"]),
        ("obligation", &["obligations.x"]),
        ("voice regime", &["regimes.voice.x"]),
        ("lifecycle regime", &["regimes.lifecycle.x"]),
        ("language regime", &["regimes.language.x"]),
    ];
    for (what, expected) in table {
        assert_eq!(
            declarations(what),
            expected.to_vec(),
            "a `{what}` read is declared at {expected:?}"
        );
    }
}

/// A sort this table does not know offers no address, so nothing is blamed
/// and the refusal keeps the line it had.
#[test]
fn an_unknown_sort_offers_no_address() {
    assert!(declarations("thing").is_empty());
}

/// The blamed sentence keeps the reader's address, because the author fixes
/// that line or restates the declaration.
#[test]
fn the_blamed_sentence_names_both_ends() {
    let found = Dangling {
        at: "shelves.decisions.kind".to_string(),
        what: "kind".to_string(),
        name: "decision".to_string(),
    };
    assert_eq!(
        found.blamed("kinds.decision"),
        "removing `kinds.decision` leaves `shelves.decisions.kind` reading `decision`, and no \
         kind of that name is declared"
    );
}
