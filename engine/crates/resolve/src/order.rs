// SPDX-License-Identifier: Apache-2.0
//! The order the resolver applies overlays in, and the two refusals that
//! `requires` owes before anything merges.
//!
//! [HW-DR-0095] (Q67) lets one library entry write into the keys of an entry it
//! names in `requires`. That write reaches a node the dependency wrote, so the
//! pair does not commute, and the ruling orders it instead: "The declared
//! dependency orders the pair, and the dependency applies first." This module
//! is where that order is made. It puts every selected bundle after each bundle
//! it names, keeps the consumer's order wherever `requires` says nothing, and
//! leaves every source that was not selected as a bundle where it was. The
//! adopter overlay is last in every selection and stays last.
//!
//! The order is what makes a lock independent of the order a consumer wrote.
//! `bundles: [a, b]` and `bundles: [b, a]` where `b` requires `a` both apply
//! `a` first, so both write one lock.
//!
//! `requires` is read for this and for nothing else. [HW-DR-0040] rules that it
//! never adds a bundle to a selection, and nothing here does. A bundle is known
//! by the name it was selected as ([`Source::selected_as`]), never by its own
//! `bundle:` key, which stays a label.
//!
//! # The two refusals
//!
//! A cycle among the selected bundles has no order that puts each dependency
//! first, so it is refused and every bundle in the cycle is named.
//!
//! A selection that holds a dependent bundle and lacks a bundle it names is
//! refused where the dependent writes into what the missing bundle would have
//! declared. HW-DR-0095 records what the merge does without this refusal: an
//! `add_to` fails as "not a list" with no word of the dependency, and an `add`
//! grafts a mapping that no entry declares and succeeds. The refusal is static.
//! A write is a dependent write when the node it needs is declared by nothing
//! the resolution holds: for `add_to` the list itself, and for `add` the
//! mapping it adds a key under. A bundle with a missing dependency and no such
//! write is left alone, because referential integrity and
//! [`crate::selection`] already report what it reads and cannot find.
//!
//! [HW-DR-0095]: ../../../../docs/decisions/0095-q67-one-library-entry-may-address-the-keys-of-an-entry-it-names-in-requires-and-confluence-holds-over-the-dependency-order.md
//! [HW-DR-0040]: ../../../../docs/decisions/0040-q40-whether-extends-bundle-requires-and-an-overlay-s-taxonomy-key-are-a-mechanism-or-a-label.md

use crate::error::{ResolveError, ResolveErrorKind};
use crate::operation::{OpKind, Operation};
use crate::source::Source;
use headwater_yaml::{Mapping, Span};
use std::collections::BTreeMap;

/// The overlays in application order, and for each one the positions in that
/// order of the bundles it names directly in `requires`.
#[derive(Debug)]
pub struct Ordered {
    /// Indices into the overlay list the caller passed, in application order.
    pub order: Vec<usize>,
    /// `requires[i]` holds the positions, in `order`, of the selected bundles
    /// that the overlay at position `i` names. A name that is not selected is
    /// not here: it is in `missing`.
    pub requires: Vec<Vec<usize>>,
    /// `missing[i]` holds each name the overlay at position `i` requires and
    /// the selection lacks.
    pub missing: Vec<Vec<String>>,
}

/// The names a source requires, when it was selected as a bundle.
fn requires_of(source: &Source) -> Vec<String> {
    if source.selected_as.is_none() {
        return Vec::new();
    }
    let Some(list) = source
        .root
        .value
        .as_map()
        .and_then(|map| map.get("requires"))
        .and_then(|node| node.value.as_seq())
    else {
        return Vec::new();
    };
    list.iter()
        .filter_map(|item| item.value.as_scalar().map(|scalar| scalar.text.clone()))
        .collect()
}

/// The span of a source's `requires` key, for a message that points at it.
fn requires_span(source: &Source) -> Span {
    source
        .root
        .value
        .as_map()
        .and_then(|map| map.get("requires"))
        .map_or(source.root.span, |node| node.span)
}

/// Put the overlays in application order, or refuse a `requires` cycle.
pub fn order(overlays: &[Source]) -> Result<Ordered, Vec<ResolveError>> {
    let mut selected: BTreeMap<&str, usize> = BTreeMap::new();
    for (index, overlay) in overlays.iter().enumerate() {
        if let Some(name) = &overlay.selected_as {
            selected.entry(name.as_str()).or_insert(index);
        }
    }

    // Edges by input index: `depends[i]` are the overlays `i` names.
    let mut depends: Vec<Vec<usize>> = Vec::with_capacity(overlays.len());
    let mut missing_by_input: Vec<Vec<String>> = Vec::with_capacity(overlays.len());
    for overlay in overlays {
        let mut found = Vec::new();
        let mut missing = Vec::new();
        for name in requires_of(overlay) {
            match selected.get(name.as_str()) {
                Some(index) if !found.contains(index) => found.push(*index),
                Some(_) => {}
                None if !missing.contains(&name) => missing.push(name),
                None => {}
            }
        }
        depends.push(found);
        missing_by_input.push(missing);
    }

    // Kahn's algorithm, taking the lowest input index that is ready, so the
    // consumer's order decides every tie.
    let mut placed = vec![false; overlays.len()];
    let mut order = Vec::with_capacity(overlays.len());
    while order.len() < overlays.len() {
        let ready = (0..overlays.len())
            .find(|index| !placed[*index] && depends[*index].iter().all(|dep| placed[*dep]));
        let Some(next) = ready else {
            return Err(vec![cycle(overlays, &depends, &placed)]);
        };
        placed[next] = true;
        order.push(next);
    }

    let mut position = vec![0; overlays.len()];
    for (at, index) in order.iter().enumerate() {
        position[*index] = at;
    }
    let requires = order
        .iter()
        .map(|index| depends[*index].iter().map(|dep| position[*dep]).collect())
        .collect();
    let missing = order
        .iter()
        .map(|index| missing_by_input[*index].clone())
        .collect();
    Ok(Ordered {
        order,
        requires,
        missing,
    })
}

/// One cycle among the overlays not yet placed, named in the order `requires`
/// walks it.
fn cycle(overlays: &[Source], depends: &[Vec<usize>], placed: &[bool]) -> ResolveError {
    // Every unplaced overlay has an unplaced dependency, so a walk that always
    // takes one never stops, and the first index it meets twice is on a cycle.
    let start = (0..overlays.len())
        .find(|index| !placed[*index])
        .expect("a cycle leaves an overlay unplaced");
    let mut walk = vec![start];
    let mut at = start;
    loop {
        at = *depends[at]
            .iter()
            .find(|dep| !placed[**dep])
            .expect("an unplaced overlay waits on an unplaced dependency");
        if let Some(first) = walk.iter().position(|seen| *seen == at) {
            walk.drain(..first);
            break;
        }
        walk.push(at);
    }
    let bundles: Vec<String> = walk
        .iter()
        .map(|index| overlays[*index].selected_as.clone().unwrap_or_default())
        .collect();
    let first = &overlays[walk[0]];
    ResolveError::new(
        ResolveErrorKind::RequiresCycle { bundles },
        &first.name,
        "requires",
        requires_span(first),
    )
}

/// The dependent writes of every bundle whose `requires` the selection does
/// not close, each one refused.
///
/// `sources` and `operations` are in application order, the base at 0.
/// `missing[i]` belongs to the overlay at source index `i + 1`.
pub fn unmet(
    base: &Mapping,
    sources: &[Source],
    operations: &[Operation],
    missing: &[Vec<String>],
) -> Vec<ResolveError> {
    let mut out = Vec::new();
    for (overlay, names) in missing.iter().enumerate() {
        if names.is_empty() {
            continue;
        }
        let source = overlay + 1;
        for operation in operations.iter().filter(|op| op.source == source) {
            let needs: &[String] = match operation.kind {
                OpKind::AddTo => operation.address.segments(),
                OpKind::Add => operation.reaches_into(),
                _ => continue,
            };
            if needs.is_empty() || declared(base, needs) || written(operations, operation, needs) {
                continue;
            }
            out.push(ResolveError::new(
                ResolveErrorKind::MissingDependency {
                    bundle: sources[source].selected_as.clone().unwrap_or_default(),
                    missing: names.clone(),
                    address: needs.join("."),
                },
                &sources[source].name,
                &operation.at(),
                operation.span,
            ));
        }
    }
    out
}

/// Whether the base declares the node at `path`.
fn declared(base: &Mapping, path: &[String]) -> bool {
    let mut at = base;
    for (depth, segment) in path.iter().enumerate() {
        let Some(node) = at.get(segment) else {
            return false;
        };
        if depth + 1 == path.len() {
            return true;
        }
        let Some(map) = node.value.as_map() else {
            return false;
        };
        at = map;
    }
    true
}

/// Whether any `add` other than `this` writes the node at `path` or something
/// below it. `add` is the one operation that declares a node: the others need
/// the node there already.
fn written(operations: &[Operation], this: &Operation, path: &[String]) -> bool {
    operations
        .iter()
        .filter(|other| !std::ptr::eq(*other, this))
        .filter(|other| other.kind == OpKind::Add)
        .any(|other| other.writes().iter().any(|leaf| leaf.starts_with(path)))
}
