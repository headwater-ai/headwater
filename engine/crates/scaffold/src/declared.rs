// SPDX-License-Identifier: Apache-2.0
//! The members of a resolved taxonomy that no typed reader carries.
//!
//! Three of them, and this module is the only reader of any. It is not a second
//! path beside a first one: every member below is declared, validated by
//! `taxonomy validate`, written into the lock, and then read by nothing. A
//! scaffolder is the first caller that needs them, so this is where they arrive
//! rather than a copy of somewhere else.
//!
//! | member | what it decides | why the typed reader drops it |
//! |---|---|---|
//! | `identifier_schemes.<name>.allocation` | how a number is issued | [`headwater_check::shape::IdentifierScheme`] states it keeps the two members that decide a *shape* |
//! | `relations.<name>.created_by` | who pays for an edge | [Q4](../../../../docs/decisions/0004-relation-storage.md) puts it on the relation type, and `taxonomy audit` is the reader it names |
//!
//! Each one moves into its own typed reader on the day a check reads it. Until
//! then a second reader would be the drift that
//! [principle 2](../../../../docs/spec/00-vision-and-scope.md#design-principles)
//! rules against.
//!
//! `regimes.lifecycle.<name>.initial` was the fourth and it left on that day.
//! [`headwater_check::shape::LifecycleRegime`] carries the whole machine now,
//! because two rules read it, and the scaffolder reads the opening state
//! through that rather than through a walk of its own.
//!
//! `shelves.<name>.layout` was the fifth and it left the same way.
//! `identifier.claim.missing` reads it through
//! [`headwater_check::claim::takes_a_claim`], so
//! [`headwater_census::shelves::Shelf`] carries it and the scaffolder takes the
//! file-name template off the shelf it already holds.

use headwater_yaml::Mapping;

/// One scalar, at `<block>.<entry>.<member>` of a resolved taxonomy.
fn member<'a>(root: &'a Mapping, block: &str, entry: &str, name: &str) -> Option<&'a str> {
    Some(
        root.get(block)?
            .value
            .as_map()?
            .get(entry)?
            .value
            .as_map()?
            .get(name)?
            .value
            .as_scalar()?
            .text
            .as_str(),
    )
}

/// `identifier_schemes.<scheme>.allocation`: `minted-once` or
/// `reconcile-first`. Reported rather than obeyed, and
/// [`crate::Minting`] states the difference this engine can and cannot honor.
pub fn allocation<'a>(root: &'a Mapping, scheme: &str) -> Option<&'a str> {
    member(root, "identifier_schemes", scheme, "allocation")
}

/// `relations.<relation>.created_by`, from the closed set spec 2 declares:
/// `author`, `scaffold`, `generator`, `hook`, `agent`, `import`.
pub fn created_by<'a>(root: &'a Mapping, relation: &str) -> Option<&'a str> {
    member(root, "relations", relation, "created_by")
}

/// `facets.<facet>.type`, which decides whether a prompt may stand in a field.
///
/// A free string takes a prompt and stays readable. An integer or a date does
/// not, and [`crate::Refusal::FacetUndeterminable`] is what this reader is for.
pub fn facet_type<'a>(root: &'a Mapping, facet: &str) -> Option<&'a str> {
    member(root, "facets", facet, "type")
}
