// SPDX-License-Identifier: Apache-2.0
//! The members of a resolved taxonomy that no typed reader carries.
//!
//! Four of them, and this module is the only reader of any. It is not a second
//! path beside a first one: every member below is declared, validated by
//! `taxonomy validate`, written into the lock, and then read by nothing. A
//! scaffolder is the first caller that needs them, so this is where they arrive
//! rather than a copy of somewhere else.
//!
//! | member | what it decides | why the typed reader drops it |
//! |---|---|---|
//! | `shelves.<name>.layout` | the file name | no check reads a placement it did not already find |
//! | `identifier_schemes.<name>.allocation` | how a number is issued | [`headwater_check::shape::IdentifierScheme`] states it keeps the two members that decide a *shape* |
//! | `regimes.lifecycle.<name>.initial` | the state a new document opens in | no rule reads a transition yet |
//! | `relations.<name>.created_by` | who pays for an edge | [Q4](../../../../docs/decisions/0004-relation-storage.md) puts it on the relation type, and `taxonomy audit` is the reader it names |
//!
//! Each one moves into its own typed reader on the day a check reads it. Until
//! then a second reader would be the drift that
//! [principle 2](../../../../docs/spec/00-vision-and-scope.md#design-principles)
//! rules against.

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

/// `shelves.<shelf>.layout`: the template a file name on this shelf is written
/// from. A shelf that declares none names its files after nothing, and the
/// caller falls back to the slug.
pub fn layout<'a>(root: &'a Mapping, shelf: &str) -> Option<&'a str> {
    member(root, "shelves", shelf, "layout")
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

/// `regimes.lifecycle.<regime>.initial`: the state a document of a kind bound
/// to this regime opens in.
pub fn initial_state<'a>(root: &'a Mapping, regime: &str) -> Option<&'a str> {
    Some(
        root.get("regimes")?
            .value
            .as_map()?
            .get("lifecycle")?
            .value
            .as_map()?
            .get(regime)?
            .value
            .as_map()?
            .get("initial")?
            .value
            .as_scalar()?
            .text
            .as_str(),
    )
}

/// `kinds.<kind>.lifecycle`, walked up `is_a` until a kind names one.
///
/// The walk is bounded by the number of kinds, so a cycle stops rather than
/// hangs. `taxonomy validate` refuses such a taxonomy, and this crate never
/// assumes that it ran.
pub fn lifecycle_of<'a>(root: &'a Mapping, kind: &str) -> Option<&'a str> {
    let kinds = root.get("kinds")?.value.as_map()?;
    let mut name = kind;
    for _ in 0..kinds.len() {
        let declaration = kinds.get(name)?.value.as_map()?;
        if let Some(regime) = declaration
            .get("lifecycle")
            .and_then(|v| v.value.as_scalar())
        {
            return Some(regime.text.as_str());
        }
        name = declaration.get("is_a")?.value.as_scalar()?.text.as_str();
    }
    None
}

/// `facets.<facet>.type`, which decides whether a prompt may stand in a field.
///
/// A free string takes a prompt and stays readable. An integer or a date does
/// not, and [`crate::Refusal::FacetUndeterminable`] is what this reader is for.
pub fn facet_type<'a>(root: &'a Mapping, facet: &str) -> Option<&'a str> {
    member(root, "facets", facet, "type")
}
