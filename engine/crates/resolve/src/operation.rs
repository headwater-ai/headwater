// SPDX-License-Identifier: Apache-2.0
//! An overlay read as operations, and the set of leaves each one writes.
//!
//! # The write set is not the address, and this repository is why
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#customization-by-composition)
//! states the confluence rule as "two `add`s at disjoint paths commute", and
//! [#48](https://github.com/headwater-ai/headwater/issues/48) handed over
//! [`headwater_ref::Address::is_disjoint_from`] as the predicate that decides
//! disjointness. Read literally over the *addressed* paths, that rule refuses a
//! pair this repository has had committed since the first typing pass. The
//! design-spec bundle writes `add: {kinds.design_spec: {...}}` and the adopter
//! overlay writes `add: {kinds.design_spec.identifier: {...}}`. One address is
//! a prefix of the other, so they are not disjoint, and the resolver would name
//! two files that an author put together on purpose.
//!
//! They do commute, and the reason says what the predicate has to run over. An
//! `add` writes a set of *leaves*: the address, extended by the path to each
//! leaf of the value it carries. The bundle writes `kinds.design_spec.is_a`,
//! `kinds.design_spec.purpose` and five more. The overlay writes
//! `kinds.design_spec.identifier.scheme`. No leaf of one is a prefix of a leaf
//! of the other, the two operations graft into disjoint parts of one mapping,
//! and either order produces the same tree. So the rule is disjointness of the
//! written leaves, and the addressed path is the special case of it where the
//! value is a leaf already.
//!
//! The direction is the one that matters. Leaf disjointness is *stricter* where
//! it differs on values: `add kinds.x: {a: 1}` and `add kinds.x.a: 2` write one
//! leaf twice and are refused, where an address-level check that stopped at
//! `kinds.x` against `kinds.x.a` would refuse them too, and both are right. It
//! is *weaker* only where two operations reach into one declaration without
//! meeting, which is the case above.
//!
//! Only `add` gets this treatment. `override` replaces a value and keeps
//! whatever the consumer did not restate, and `remove` deletes a key and
//! everything under it, so each one owns the whole subtree at its address
//! whatever its value looks like.

use crate::error::{ResolveError, ResolveErrorKind};
use headwater_ref::Address;
use headwater_yaml::{Mapping, Span, Spanned, Value};

/// The five merge operations of [spec 2](../../../../docs/spec/02-taxonomy-model.md#customization-by-composition).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpKind {
    Add,
    Override,
    AddTo,
    RemoveFrom,
    Remove,
}

impl OpKind {
    pub fn word(self) -> &'static str {
        match self {
            OpKind::Add => "add",
            OpKind::Override => "override",
            OpKind::AddTo => "add_to",
            OpKind::RemoveFrom => "remove_from",
            OpKind::Remove => "remove",
        }
    }

    /// The block a source writes this operation under, in the order the
    /// resolver applies them. Additions before replacements before deletions,
    /// so that one source can add a declaration and remove another without the
    /// order inside the file deciding the result.
    pub const APPLICATION_ORDER: [OpKind; 5] = [
        OpKind::Add,
        OpKind::Override,
        OpKind::AddTo,
        OpKind::RemoveFrom,
        OpKind::Remove,
    ];
}

/// One operation, with the source that wrote it.
#[derive(Clone, Debug)]
pub struct Operation {
    /// Index into the resolver's source list.
    pub source: usize,
    pub kind: OpKind,
    pub address: Address,
    /// Absent for `remove`, which names an address and carries no value.
    pub value: Option<Spanned<Value>>,
    pub span: Span,
}

impl Operation {
    /// The operation as an overlay writes it, which is what a message names.
    pub fn at(&self) -> String {
        format!("{}.{}", self.kind.word(), self.address)
    }

    /// Every leaf this operation writes, as a path from the root.
    ///
    /// See the module comment. An `add` is the union of its value's leaves
    /// under its address. Everything else owns its whole subtree, which is the
    /// address alone.
    pub fn writes(&self) -> Vec<Vec<String>> {
        let base = self.address.segments().to_vec();
        match (self.kind, &self.value) {
            (OpKind::Add, Some(value)) => crate::merge::leaf_values(&base, value)
                .into_iter()
                .map(|(path, _)| path)
                .collect(),
            _ => vec![base],
        }
    }
}

/// Read every operation out of one overlay source.
///
/// The meta-schema has already validated the source, so an address here parses
/// and lands on a declared position. A parse that fails anyway is reported
/// rather than skipped, because a resolver that silently dropped an operation
/// would produce a taxonomy that no source asked for.
pub fn read(
    source: usize,
    name: &str,
    root: &Spanned<Value>,
) -> Result<Vec<Operation>, Vec<ResolveError>> {
    let mut out = Vec::new();
    let mut errors = Vec::new();
    let Some(map) = root.value.as_map() else {
        return Err(vec![ResolveError::new(
            ResolveErrorKind::WrongRole {
                expected: "a mapping of operations",
            },
            name,
            "",
            root.span,
        )]);
    };

    for kind in OpKind::APPLICATION_ORDER {
        match kind {
            OpKind::Remove => read_remove(source, name, map, &mut out, &mut errors),
            _ => read_addressed(source, name, map, kind, &mut out, &mut errors),
        }
    }

    if errors.is_empty() {
        Ok(out)
    } else {
        Err(errors)
    }
}

fn read_addressed(
    source: usize,
    name: &str,
    map: &Mapping,
    kind: OpKind,
    out: &mut Vec<Operation>,
    errors: &mut Vec<ResolveError>,
) {
    let Some(block) = map.get(kind.word()) else {
        return;
    };
    let Some(entries) = block.value.as_map() else {
        return; // the meta-schema reported the shape
    };
    for entry in entries {
        match Address::parse(&entry.key.value) {
            Ok(address) => out.push(Operation {
                source,
                kind,
                address,
                value: Some(entry.value.clone()),
                span: entry.key.span,
            }),
            Err(error) => errors.push(ResolveError::new(
                ResolveErrorKind::SourceRefused(format!(
                    "`{}` is not an address: {error}",
                    entry.key.value
                )),
                name,
                kind.word(),
                entry.key.span,
            )),
        }
    }
}

fn read_remove(
    source: usize,
    name: &str,
    map: &Mapping,
    out: &mut Vec<Operation>,
    errors: &mut Vec<ResolveError>,
) {
    let Some(block) = map.get("remove") else {
        return;
    };
    let Some(items) = block.value.as_seq() else {
        return; // the meta-schema reported the shape
    };
    for item in items {
        let Some(scalar) = item.value.as_scalar() else {
            continue;
        };
        match Address::parse(&scalar.text) {
            Ok(address) => out.push(Operation {
                source,
                kind: OpKind::Remove,
                address,
                value: None,
                span: item.span,
            }),
            Err(error) => errors.push(ResolveError::new(
                ResolveErrorKind::SourceRefused(format!(
                    "`{}` is not an address: {error}",
                    scalar.text
                )),
                name,
                "remove",
                item.span,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ops(source: &str) -> Vec<Operation> {
        let root = headwater_yaml::load(source).expect("the fixture loads");
        read(0, "test", &root).expect("operations")
    }

    #[test]
    fn an_add_writes_the_leaves_of_its_value_and_not_its_address() {
        let operations =
            ops("add:\n  kinds.design_spec: {is_a: governed_document, purpose: behavior}\n");
        assert_eq!(
            operations[0].writes(),
            vec![
                vec!["kinds".to_string(), "design_spec".into(), "is_a".into()],
                vec!["kinds".to_string(), "design_spec".into(), "purpose".into()],
            ]
        );
    }

    /// The pair this repository committed. Neither address is disjoint from the
    /// other, and no leaf of one is a prefix of a leaf of the other.
    #[test]
    fn a_bundle_and_an_overlay_may_reach_into_one_kind_without_meeting() {
        let bundle = ops("add:\n  kinds.design_spec: {is_a: governed_document}\n");
        let overlay = ops("add:\n  kinds.design_spec.identifier: {scheme: spec_id}\n");
        assert!(!bundle[0].address.is_disjoint_from(&overlay[0].address));
        for a in bundle[0].writes() {
            for b in overlay[0].writes() {
                assert_ne!(a, b);
                assert!(!a.starts_with(&b) && !b.starts_with(&a));
            }
        }
    }

    /// A list is a leaf. Two operations that reach one list reach one node.
    #[test]
    fn a_sequence_stops_the_walk_because_a_list_has_no_addressable_interior() {
        let operations = ops("add:\n  kinds.x: {sections: {require: [Scope, Behavior]}}\n");
        assert_eq!(
            operations[0].writes(),
            vec![vec![
                "kinds".to_string(),
                "x".into(),
                "sections".into(),
                "require".into()
            ]]
        );
    }

    #[test]
    fn an_empty_mapping_is_a_leaf_because_it_is_a_value_somebody_wrote() {
        let operations = ops("add:\n  regimes.voice.narrative: {}\n");
        assert_eq!(
            operations[0].writes(),
            vec![vec![
                "regimes".to_string(),
                "voice".into(),
                "narrative".into()
            ]]
        );
    }

    #[test]
    fn a_remove_owns_its_whole_subtree_whatever_is_under_it() {
        let operations = ops("remove:\n  - shelves.proposals\n");
        assert_eq!(operations[0].kind, OpKind::Remove);
        assert_eq!(
            operations[0].writes(),
            vec![vec!["shelves".to_string(), "proposals".into()]]
        );
    }
}
