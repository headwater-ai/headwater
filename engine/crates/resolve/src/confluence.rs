// SPDX-License-Identifier: Apache-2.0
//! The static check that an overlay set commutes, run before anything merges.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#customization-by-composition):
//! "Overlay application must be **confluent**: the application of a set of
//! overlays in any legal order yields the same resolved taxonomy. The resolver
//! checks this statically, before it applies anything." And: "the resolver
//! rejects any pair that does not commute, and it names both overlays and the
//! contested path."
//!
//! Static is the whole value of it. A resolver that discovered a conflict
//! while merging would report whichever conflict the order it happened to pick
//! reached first, and a publisher could not prove ahead of a release that every
//! subset of its bundles resolves ([spec 7](../../../../docs/spec/07-distribution-and-federation.md#bundles-are-publisher-overlays-in-the-other-direction)).
//! The check here reads the operations and never the tree, so its answer does
//! not depend on the base it is about to be run against.
//!
//! # What commutes
//!
//! Two `add` operations commute when no leaf either one writes is a prefix of a
//! leaf the other writes. [`crate::operation`] states why the predicate runs
//! over the written leaves rather than over the addressed paths.
//!
//! Every other pair owns its whole subtree. An `override` keeps the fields the
//! consumer did not restate and a `remove` takes everything below the key, so
//! two operations that reach one subtree with either of those in the pair leave
//! the result to the order of application. That is the conflict spec 2 names,
//! and it is decided with [`headwater_ref::Address::is_disjoint_from`].
//!
//! # It runs between sources and not inside one
//!
//! Spec 2 asks for a message that "names both overlays". Two operations in one
//! file are ordered by the file, and where they genuinely collide the
//! precondition that each operation asserts about the base reports it during
//! the merge: an `add` under an `add` finds the key already there. So this
//! check runs over pairs from distinct sources, which is the set whose order is
//! not written down anywhere.

use crate::error::{ResolveError, ResolveErrorKind};
use crate::operation::{OpKind, Operation};

/// Every pair that does not commute, each reported against the later operation.
pub fn check(operations: &[Operation], sources: &[String]) -> Vec<ResolveError> {
    let mut out = Vec::new();
    for (index, later) in operations.iter().enumerate() {
        for earlier in &operations[..index] {
            if earlier.source == later.source {
                continue;
            }
            let Some((path, why)) = contested(earlier, later) else {
                continue;
            };
            out.push(ResolveError::new(
                ResolveErrorKind::NotConfluent {
                    other_source: sources[earlier.source].clone(),
                    other_at: earlier.at(),
                    path,
                    why,
                },
                &sources[later.source],
                &later.at(),
                later.span,
            ));
        }
    }
    out
}

/// The contested path and the reason, or `None` when the two commute.
fn contested(a: &Operation, b: &Operation) -> Option<(String, &'static str)> {
    if a.kind == OpKind::Add && b.kind == OpKind::Add {
        for left in a.writes() {
            for right in b.writes() {
                if overlaps(&left, &right) {
                    return Some((
                        deeper(&left, &right),
                        "two `add` operations that write one leaf state two values for it",
                    ));
                }
            }
        }
        return None;
    }

    if a.address.is_disjoint_from(&b.address) {
        return None;
    }
    let path = deeper(a.address.segments(), b.address.segments());
    Some((path, why(a.kind, b.kind)))
}

fn why(a: OpKind, b: OpKind) -> &'static str {
    match (a, b) {
        (OpKind::Remove, _) | (_, OpKind::Remove) => {
            "a `remove` takes everything below its key, so what the other operation writes \
             depends on which ran first"
        }
        (OpKind::Override, OpKind::Override) => {
            "an `override` replaces a value, so two of them over one subtree leave the result \
             to the order of application"
        }
        (OpKind::AddTo, _)
        | (_, OpKind::AddTo)
        | (OpKind::RemoveFrom, _)
        | (_, OpKind::RemoveFrom) => {
            "two list operations over one list leave the order of the result to the order of \
             application"
        }
        _ => {
            "an `override` keeps every field the other source did not restate, so what survives \
             depends on which ran first"
        }
    }
}

fn overlaps(a: &[String], b: &[String]) -> bool {
    a.starts_with(b) || b.starts_with(a)
}

/// The longer of two overlapping paths, which is the node an author looks at.
fn deeper(a: &[String], b: &[String]) -> String {
    if a.len() >= b.len() {
        a.join(".")
    } else {
        b.join(".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation;

    fn operations(sources: &[&str]) -> Vec<Operation> {
        let mut out = Vec::new();
        for (index, source) in sources.iter().enumerate() {
            let root = headwater_yaml::load(source).expect("the fixture loads");
            out.extend(operation::read(index, "test", &root).expect("operations"));
        }
        out
    }

    fn names(count: usize) -> Vec<String> {
        (0..count).map(|index| format!("source-{index}")).collect()
    }

    fn errors(sources: &[&str]) -> Vec<ResolveError> {
        check(&operations(sources), &names(sources.len()))
    }

    #[test]
    fn two_adds_into_one_kind_commute_when_no_leaf_is_shared() {
        assert!(errors(&[
            "add:\n  kinds.design_spec: {is_a: governed_document}\n",
            "add:\n  kinds.design_spec.identifier: {scheme: spec_id}\n",
        ])
        .is_empty());
    }

    #[test]
    fn two_adds_that_write_one_leaf_do_not_commute() {
        let found = errors(&[
            "add:\n  kinds.design_spec: {purpose: behavior}\n",
            "add:\n  kinds.design_spec.purpose: rationale\n",
        ]);
        assert_eq!(found.len(), 1);
        assert!(
            found[0].to_string().contains("kinds.design_spec.purpose"),
            "{}",
            found[0]
        );
    }

    #[test]
    fn a_prefix_that_is_only_textual_is_not_a_conflict() {
        assert!(errors(&[
            "add:\n  kinds.playbook: {purpose: procedure}\n",
            "add:\n  kinds.playbook_step: {purpose: procedure}\n",
        ])
        .is_empty());
    }

    #[test]
    fn an_override_owns_its_subtree_whatever_its_value_holds() {
        let found = errors(&[
            "add:\n  kinds.report.purpose: behavior\n",
            "override:\n  kinds.report: {purpose: rationale}\n",
        ]);
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn a_remove_conflicts_with_anything_under_it() {
        let found = errors(&[
            "add:\n  kinds.report.purpose: behavior\n",
            "remove:\n  - kinds.report\n",
        ]);
        assert_eq!(found.len(), 1);
        assert!(found[0].to_string().contains("remove"), "{}", found[0]);
    }

    /// Two operations in one file are ordered by the file, so the pair check
    /// has nothing to say about them.
    #[test]
    fn the_check_runs_between_sources_and_not_inside_one() {
        assert!(errors(&[
            "add:\n  kinds.report: {purpose: behavior}\n  kinds.report.purpose: rationale\n",
        ])
        .is_empty());
    }
}
