// SPDX-License-Identifier: Apache-2.0
//! The merge itself: five operations over one tree, each asserting what it
//! believes about the tree it is applied to.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#customization-by-composition):
//! "Every operation asserts a precondition about the base, and a failed
//! precondition is always an error. `override` needs the path to exist. `add`
//! needs the key to be absent. `remove` needs the key to be present." The
//! symmetry is what makes an upgrade that falsifies an overlay's belief say so
//! rather than proceed.
//!
//! # An `add` asserts its precondition about the base, and writes leaf by leaf
//!
//! Spec 2 says "a precondition about the **base**", and the word is exact. An
//! `add` at `kinds.design_spec` is refused when the *base package* declares
//! that key, which is the collision an upstream release causes. It is not
//! refused because another overlay of the same resolution already created the
//! mapping on its way to `kinds.design_spec.identifier`. Read the other way,
//! the two `add` operations that this repository has committed would resolve in
//! one order and fail in the other, and the confluence check has just proved
//! that they have no order.
//!
//! So an `add` writes the leaves of its value rather than its whole node, and
//! each leaf asserts its own absence. `add kinds.x: {a: 1}` and
//! `add kinds.x.b: 2` produce one mapping with both members, whichever runs
//! first, and `add kinds.x: {a: 1}` against a base that already declares
//! `kinds.x.a` is still the collision spec 2 refuses.
//!
//! # Nothing here mutates
//!
//! Each operation returns a new tree. A taxonomy is a few hundred nodes and a
//! resolution is a few dozen operations, so the cost is nothing, and what it
//! buys is that a failed operation leaves no half-applied tree behind for the
//! next one to be validated against.

use crate::error::ResolveErrorKind;
use headwater_yaml::{Entry, Mapping, Spanned, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Add,
    Override,
}

/// Write every leaf of `value` under `path`, each one asserting its own
/// absence. This is what an `add` does. See the module comment.
pub fn graft_leaves(
    map: &Mapping,
    path: &[String],
    value: &Spanned<Value>,
    full: &str,
) -> Result<Mapping, ResolveErrorKind> {
    let mut out = map.clone();
    for (leaf, node) in leaf_values(path, value) {
        // The leaf names the collision rather than the operation, because the
        // author has to look at one member and not at a whole declaration.
        let at = if leaf.len() == path.len() {
            full.to_string()
        } else {
            leaf.join(".")
        };
        out = graft(&out, &leaf, &node, Mode::Add, &at)?;
    }
    Ok(out)
}

/// Every leaf of a value, with the path it sits at.
///
/// A mapping with entries is not a leaf. An empty mapping, a sequence and a
/// scalar are. A sequence stops the walk because a list has no addressable
/// interior: spec 2 refuses an address into a list, and `add_to` and
/// `remove_from` are what a list takes instead, so two operations that reach
/// one list reach one node.
pub fn leaf_values(
    prefix: &[String],
    value: &Spanned<Value>,
) -> Vec<(Vec<String>, Spanned<Value>)> {
    match &value.value {
        Value::Map(map) if !map.is_empty() => map
            .iter()
            .flat_map(|entry| {
                let mut deeper = prefix.to_vec();
                deeper.push(entry.key.value.clone());
                leaf_values(&deeper, &entry.value)
            })
            .collect(),
        _ => vec![(prefix.to_vec(), value.clone())],
    }
}

/// The key order that makes one resolved taxonomy one text.
///
/// Two overlays that commute produce the same members and can produce them in
/// two orders, because a mapping records the order its entries arrived in. That
/// difference is invisible to every check and visible in a lock, so it is taken
/// out here rather than left for a reader of a diff to discount.
///
/// The rule: a key the base declares keeps the base's position, and a key the
/// base does not declare follows, sorted. A declaration the base wrote reads in
/// the order its author chose, and anything an overlay contributed reads in an
/// order that no application order can move.
pub fn canonical(base: Option<&Mapping>, merged: &Mapping) -> Mapping {
    let order: Vec<String> = base
        .map(|map| map.iter().map(|entry| entry.key.value.clone()).collect())
        .unwrap_or_default();

    let mut kept: Vec<Entry> = order
        .iter()
        .filter_map(|key| merged.entry(key).cloned())
        .collect();
    let mut added: Vec<Entry> = merged
        .iter()
        .filter(|entry| !order.contains(&entry.key.value))
        .cloned()
        .collect();
    added.sort_by(|one, other| one.key.value.cmp(&other.key.value));
    kept.extend(added);

    for entry in &mut kept {
        if let Value::Map(inner) = &entry.value.value {
            let below = base
                .and_then(|map| map.get(&entry.key.value))
                .and_then(|node| node.value.as_map());
            entry.value.value = Value::Map(canonical(below, inner));
        }
    }
    Mapping::new(kept)
}

/// Write `value` at `path`.
pub fn graft(
    map: &Mapping,
    path: &[String],
    value: &Spanned<Value>,
    mode: Mode,
    full: &str,
) -> Result<Mapping, ResolveErrorKind> {
    graft_at(map, path, value, mode, full, &[])
}

fn graft_at(
    map: &Mapping,
    path: &[String],
    value: &Spanned<Value>,
    mode: Mode,
    full: &str,
    walked: &[String],
) -> Result<Mapping, ResolveErrorKind> {
    let (head, tail) = path.split_first().expect("an address holds a segment");
    let mut entries: Vec<Entry> = map.entries().to_vec();
    let found = entries.iter().position(|entry| &entry.key.value == head);

    if tail.is_empty() {
        match (found, mode) {
            (Some(_), Mode::Add) => return Err(ResolveErrorKind::AddCollides(full.to_string())),
            (None, Mode::Override) => {
                return Err(ResolveErrorKind::OverrideMissing(full.to_string()))
            }
            (Some(index), Mode::Override) => entries[index].value = value.clone(),
            (None, Mode::Add) => entries.push(Entry {
                key: Spanned::new(head.clone(), value.span),
                value: value.clone(),
            }),
        }
        return Ok(Mapping::new(entries));
    }

    let here = joined(walked, head);
    match found {
        Some(index) => {
            let inner = entries[index].value.clone();
            if is_reference(&inner) {
                return Err(ResolveErrorKind::ThroughReference {
                    address: full.to_string(),
                    at: here,
                });
            }
            let Some(child) = inner.value.as_map() else {
                return Err(ResolveErrorKind::ThroughNonMapping {
                    address: full.to_string(),
                    at: here,
                });
            };
            let deeper = graft_at(child, tail, value, mode, full, &segments(walked, head))?;
            entries[index].value = Spanned::new(Value::Map(deeper), inner.span);
        }
        None => {
            if mode == Mode::Override {
                return Err(ResolveErrorKind::OverrideMissing(full.to_string()));
            }
            let deeper = graft_at(
                &Mapping::default(),
                tail,
                value,
                mode,
                full,
                &segments(walked, head),
            )?;
            entries.push(Entry {
                key: Spanned::new(head.clone(), value.span),
                value: Spanned::new(Value::Map(deeper), value.span),
            });
        }
    }
    Ok(Mapping::new(entries))
}

/// Delete the key at `path`, and everything under it.
pub fn prune(map: &Mapping, path: &[String], full: &str) -> Result<Mapping, ResolveErrorKind> {
    prune_at(map, path, full, &[])
}

fn prune_at(
    map: &Mapping,
    path: &[String],
    full: &str,
    walked: &[String],
) -> Result<Mapping, ResolveErrorKind> {
    let (head, tail) = path.split_first().expect("an address holds a segment");
    let mut entries: Vec<Entry> = map.entries().to_vec();
    let Some(index) = entries.iter().position(|entry| &entry.key.value == head) else {
        return Err(ResolveErrorKind::RemoveMissing(full.to_string()));
    };

    if tail.is_empty() {
        entries.remove(index);
        return Ok(Mapping::new(entries));
    }

    let inner = entries[index].value.clone();
    let here = joined(walked, head);
    if is_reference(&inner) {
        return Err(ResolveErrorKind::ThroughReference {
            address: full.to_string(),
            at: here,
        });
    }
    let Some(child) = inner.value.as_map() else {
        return Err(ResolveErrorKind::ThroughNonMapping {
            address: full.to_string(),
            at: here,
        });
    };
    let deeper = prune_at(child, tail, full, &segments(walked, head))?;
    entries[index].value = Spanned::new(Value::Map(deeper), inner.span);
    Ok(Mapping::new(entries))
}

/// Append to the list at `path`. Lists never silently merge, so this is the
/// only way a list grows.
pub fn add_to(
    map: &Mapping,
    path: &[String],
    value: &Spanned<Value>,
    full: &str,
) -> Result<Mapping, ResolveErrorKind> {
    let current = lookup(map, path).ok_or_else(|| ResolveErrorKind::NotAList(full.to_string()))?;
    let Some(items) = current.value.as_seq() else {
        return Err(ResolveErrorKind::NotAList(full.to_string()));
    };
    let mut extended = items.to_vec();
    extended.extend(items_of(value));
    let replaced = Spanned::new(Value::Seq(extended), current.span);
    graft(map, path, &replaced, Mode::Override, full)
}

/// Take items out of the list at `path`. An item the list does not hold is a
/// failed precondition, for the reason `remove` has one.
pub fn remove_from(
    map: &Mapping,
    path: &[String],
    value: &Spanned<Value>,
    full: &str,
) -> Result<Mapping, ResolveErrorKind> {
    let current = lookup(map, path).ok_or_else(|| ResolveErrorKind::NotAList(full.to_string()))?;
    let Some(items) = current.value.as_seq() else {
        return Err(ResolveErrorKind::NotAList(full.to_string()));
    };
    let mut kept = items.to_vec();
    for wanted in items_of(value) {
        let Some(index) = kept
            .iter()
            .position(|item| same(&item.value, &wanted.value))
        else {
            return Err(ResolveErrorKind::ItemNotInList(describe(&wanted.value)));
        };
        kept.remove(index);
    }
    let replaced = Spanned::new(Value::Seq(kept), current.span);
    graft(map, path, &replaced, Mode::Override, full)
}

/// The node at `path`, or `None`.
pub fn lookup<'a>(map: &'a Mapping, path: &[String]) -> Option<&'a Spanned<Value>> {
    let (head, tail) = path.split_first()?;
    let entry = map.get(head)?;
    if tail.is_empty() {
        return Some(entry);
    }
    lookup(entry.value.as_map()?, tail)
}

/// Whether two values are the same, spans aside.
///
/// A span says where a value was written and a merge moves values between
/// sources, so two values that a reader would call identical carry different
/// spans by construction.
pub fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Scalar(left), Value::Scalar(right)) => left.text == right.text,
        (Value::Seq(left), Value::Seq(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(one, other)| same(&one.value, &other.value))
        }
        (Value::Map(left), Value::Map(right)) => {
            left.len() == right.len()
                && left.iter().zip(right).all(|(one, other)| {
                    one.key.value == other.key.value && same(&one.value.value, &other.value.value)
                })
        }
        _ => false,
    }
}

/// A value written where a list of items is expected, read as the items.
///
/// `add_to: {core.requires: [{purpose: procedure}]}` writes a list, and
/// `add_to: {kinds.decision.sections.require: Rationale}` writes one item. The
/// second is the same sugar a relation endpoint already takes.
fn items_of(value: &Spanned<Value>) -> Vec<Spanned<Value>> {
    match &value.value {
        Value::Seq(items) => items.clone(),
        _ => vec![value.clone()],
    }
}

fn describe(value: &Value) -> String {
    match value {
        Value::Scalar(scalar) => scalar.text.clone(),
        other => other.kind_name().to_string(),
    }
}

fn is_reference(value: &Spanned<Value>) -> bool {
    value
        .value
        .as_scalar()
        .is_some_and(|scalar| scalar.text.starts_with('$') && !scalar.text.starts_with("$$"))
}

fn segments(walked: &[String], head: &str) -> Vec<String> {
    let mut out = walked.to_vec();
    out.push(head.to_string());
    out
}

fn joined(walked: &[String], head: &str) -> String {
    segments(walked, head).join(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(source: &str) -> Mapping {
        headwater_yaml::load(source)
            .expect("the fixture loads")
            .value
            .as_map()
            .expect("a mapping")
            .clone()
    }

    fn value(source: &str) -> Spanned<Value> {
        headwater_yaml::load(source).expect("the fixture loads")
    }

    fn at(map: &Mapping, path: &str) -> String {
        let segments: Vec<String> = path.split('.').map(str::to_string).collect();
        describe(&lookup(map, &segments).expect("a node").value)
    }

    #[test]
    fn an_add_creates_the_mappings_on_the_way_down() {
        let base = tree("kinds: {}\n");
        let grafted = graft(
            &base,
            &[
                "kinds".into(),
                "report".into(),
                "identifier".into(),
                "scheme".into(),
            ],
            &value("spec_id"),
            Mode::Add,
            "kinds.report.identifier.scheme",
        )
        .expect("grafts");
        assert_eq!(at(&grafted, "kinds.report.identifier.scheme"), "spec_id");
    }

    #[test]
    fn an_add_over_a_declared_key_is_a_collision_rather_than_a_replacement() {
        let base = tree("kinds:\n  report: {purpose: behavior}\n");
        let error = graft(
            &base,
            &["kinds".into(), "report".into(), "purpose".into()],
            &value("rationale"),
            Mode::Add,
            "kinds.report.purpose",
        )
        .expect_err("a collision");
        assert!(matches!(error, ResolveErrorKind::AddCollides(_)));
    }

    #[test]
    fn an_override_needs_the_path_to_be_there_already() {
        let base = tree("shelves:\n  decisions: {path: docs/decisions/**}\n");
        let error = graft(
            &base,
            &["shelves".into(), "proposals".into(), "path".into()],
            &value("docs/proposals/**"),
            Mode::Override,
            "shelves.proposals.path",
        )
        .expect_err("a missing path");
        assert!(matches!(error, ResolveErrorKind::OverrideMissing(_)));
    }

    #[test]
    fn an_override_replaces_in_place_and_does_not_reorder_the_tree() {
        let base = tree("shelves:\n  a: {path: one}\n  b: {path: two}\n");
        let after = graft(
            &base,
            &["shelves".into(), "a".into(), "path".into()],
            &value("three"),
            Mode::Override,
            "shelves.a.path",
        )
        .expect("overrides");
        let shelves = after.get("shelves").unwrap().value.as_map().unwrap();
        assert_eq!(shelves.entries()[0].key.value, "a");
        assert_eq!(at(&after, "shelves.a.path"), "three");
    }

    #[test]
    fn a_remove_needs_the_key_to_be_there() {
        let base = tree("shelves:\n  decisions: {path: one}\n");
        let error = prune(
            &base,
            &["shelves".into(), "proposals".into()],
            "shelves.proposals",
        )
        .expect_err("a missing key");
        assert!(matches!(error, ResolveErrorKind::RemoveMissing(_)));
    }

    #[test]
    fn an_address_never_travels_through_a_reference() {
        let base = tree("facets:\n  status: {values: $vocabularies.lifecycle_state}\n");
        let error = graft(
            &base,
            &[
                "facets".into(),
                "status".into(),
                "values".into(),
                "0".into(),
            ],
            &value("draft"),
            Mode::Add,
            "facets.status.values.0",
        )
        .expect_err("through a reference");
        assert!(matches!(error, ResolveErrorKind::ThroughReference { .. }));
    }

    #[test]
    fn a_list_grows_only_through_add_to() {
        let base = tree("kinds:\n  decision: {sections: {require: [Context, Decision]}}\n");
        let after = add_to(
            &base,
            &[
                "kinds".into(),
                "decision".into(),
                "sections".into(),
                "require".into(),
            ],
            &value("Consequences"),
            "kinds.decision.sections.require",
        )
        .expect("appends");
        let items = lookup(
            &after,
            &[
                "kinds".into(),
                "decision".into(),
                "sections".into(),
                "require".into(),
            ],
        )
        .unwrap()
        .value
        .as_seq()
        .unwrap()
        .len();
        assert_eq!(items, 3);
    }

    #[test]
    fn remove_from_refuses_an_item_the_list_does_not_hold() {
        let base = tree("kinds:\n  decision: {sections: {require: [Context]}}\n");
        let error = remove_from(
            &base,
            &[
                "kinds".into(),
                "decision".into(),
                "sections".into(),
                "require".into(),
            ],
            &value("Rationale"),
            "kinds.decision.sections.require",
        )
        .expect_err("not in the list");
        assert!(matches!(error, ResolveErrorKind::ItemNotInList(_)));
    }
}
