// SPDX-License-Identifier: Apache-2.0
//! One relation vocabulary, enumerated twice, held against itself.
//!
//! Two functions in two crates answer the same question — *which names may a
//! document write beside its facets, and which of them are relations?*
//!
//! - [`Declarations::named`] is the reader a real document meets. It matches a
//!   relation's own name, else any `inverse` a relation declares, and
//!   `edges.rs` refuses everything else as "neither a declared relation nor a
//!   declared inverse".
//! - `headwater_resolve::template::relations` is the reader a *template* meets.
//!   [PR #630](https://github.com/headwater-ai/headwater/pull/630) added it so
//!   that `taxonomy publish` refuses a template writing a relation name at the
//!   top level of its front matter instead of under a `relations:` block.
//!
//! The second one is a second hand-written enumeration of the first one's
//! population, and until this file nothing compared them
//! ([#631](https://github.com/headwater-ai/headwater/issues/631)).
//!
//! # Why this is an assertion and not one source
//!
//! One source would be better and is not available. `headwater-graph` depends
//! on `headwater-resolve`, so the template reader cannot call `named`: that
//! direction is a dependency inversion. The reverse derivation does not exist
//! either, because `named` returns the `Relation` record it matched and a list
//! of names cannot produce one. Moving `Relation` and `read_relation` into
//! `headwater-resolve` would give one source and is a crate-boundary change
//! that [#631](https://github.com/headwater-ai/headwater/issues/631) rules out
//! of its own scope. So the two stand, and this file is what stops them
//! drifting.
//!
//! # Why the drift would otherwise be silent
//!
//! The template reader is a refusal, so a collector that sees less refuses
//! less: it reports nothing, exits 0, and reads as a working check. Cutting it
//! to the map keys alone reproduces the original defect, because five of this
//! repository's seventeen relation names reach a document only as an `inverse`.
//! `taxonomy publish` ships that refusal, so the narrowing would reach an
//! adopter.

use headwater_graph::declarations::{Declarations, Relation};
use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

/// This repository, resolved: the taxonomy both readers are given.
///
/// It is the resolved taxonomy rather than a fixture because that is the input
/// both functions take, and because it is the taxonomy this repository
/// publishes. A fixture would hold the two readers to a population no adopter
/// meets.
fn resolved_taxonomy() -> headwater_yaml::Mapping {
    let root = repository_root();
    headwater_resolve::repository(&root)
        .unwrap_or_else(|errors| panic!("{}", headwater_resolve::render_errors(&errors)))
        .resolution
        .taxonomy
}

/// Every name [`Declarations`] admits, derived from the records it parsed.
///
/// The destructuring is exhaustive on purpose. A field added to [`Relation`]
/// stops this file compiling, so whoever adds a way for a relation to carry a
/// name has to come here and say whether it is one. That is the half of the
/// drift no set comparison can see, because a name source added to both
/// readers at once is not drift at all.
fn admitted(declarations: &Declarations) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let mut push = |name: &str| {
        if !names.iter().any(|held| held == name) {
            names.push(name.to_string());
        }
    };
    for relation in &declarations.relations {
        let Relation {
            // The two that are names.
            name,
            inverse,
            // The nine that are not. `reciprocal: symmetric` coins no word: a
            // symmetric relation is its own inverse.
            from: _,
            to: _,
            reciprocal: _,
            family: _,
            lifecycle_sensitive: _,
            sets_target_state: _,
            nuclearity: _,
            nucleus: _,
            created_by: _,
            span: _,
        } = relation;
        push(name);
        if let Some(inverse) = inverse {
            push(inverse);
        }
    }
    names
}

fn sorted(names: &[String]) -> Vec<String> {
    let mut out = names.to_vec();
    out.sort();
    out
}

fn missing_from(these: &[String], those: &[String]) -> Vec<String> {
    sorted(these)
        .into_iter()
        .filter(|name| !those.iter().any(|held| held == name))
        .collect()
}

#[test]
fn the_two_relation_vocabularies_are_one_set() {
    let taxonomy = resolved_taxonomy();
    let declarations = Declarations::read(&taxonomy).expect("the resolved declarations read");

    let admits = admitted(&declarations);
    let collects = headwater_resolve::template::relations(&taxonomy);

    let uncollected = missing_from(&admits, &collects);
    let uncollectable = missing_from(&collects, &admits);

    assert!(
        uncollected.is_empty(),
        "`graph::Declarations` admits these relation names and \
         `resolve::template::relations` does not collect them, so a template \
         writing one at the top level would publish unrefused: {uncollected:?}"
    );
    assert!(
        uncollectable.is_empty(),
        "`resolve::template::relations` collects these names and \
         `graph::Declarations` admits none of them, so a template would be \
         refused for a name no document may write: {uncollectable:?}"
    );
}

/// Every name the template reader collects resolves through the reader a
/// document meets, and in the direction the declaration gives it.
///
/// The set comparison above already holds the population. This holds the
/// *oracle*: it asks `named` rather than reading the records `named` reads, so
/// a change to how `named` matches that leaves the records alone is a change
/// this case sees.
#[test]
fn every_collected_name_resolves_through_the_document_reader() {
    let taxonomy = resolved_taxonomy();
    let declarations = Declarations::read(&taxonomy).expect("the resolved declarations read");

    let unresolved: Vec<String> = headwater_resolve::template::relations(&taxonomy)
        .into_iter()
        .filter(|name| declarations.named(name).is_none())
        .collect();

    assert!(
        unresolved.is_empty(),
        "`resolve::template::relations` collects these names and \
         `Declarations::named` resolves none of them: {unresolved:?}"
    );
}

/// Both halves of the population are populated, so neither case above can pass
/// by comparing an empty set to an empty set.
///
/// A relation vocabulary that lost every `inverse` would make the set
/// comparison agree on the keys alone and say nothing about the half that
/// carried the defect. This names the halves it found rather than counting
/// them, so a taxonomy that grows a relation does not have to be edited here.
#[test]
fn neither_half_of_the_vocabulary_is_empty() {
    let taxonomy = resolved_taxonomy();
    let declarations = Declarations::read(&taxonomy).expect("the resolved declarations read");

    let keys: Vec<String> = declarations
        .relations
        .iter()
        .map(|relation| relation.name.clone())
        .collect();
    let inverse_only: Vec<String> = admitted(&declarations)
        .into_iter()
        .filter(|name| !keys.iter().any(|key| key == name))
        .collect();

    assert!(
        !keys.is_empty(),
        "this repository resolves to no relation at all, so the vocabulary \
         comparison holds nothing"
    );
    assert!(
        !inverse_only.is_empty(),
        "this repository resolves to no name reachable only as an `inverse`, \
         so the comparison would pass over a collector that reads the map keys \
         alone. The keys it found: {keys:?}"
    );
}
