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
//! One pair is ordered rather than tested. [HW-DR-0095] (Q67) lets a library
//! entry write into an entry it names directly in `requires`, with `add` or
//! `add_to`. The declared dependency applies first, so the pair has one order
//! and nothing is left for the order of application to decide. The resolver
//! puts the dependency first before it merges ([`crate::order`]). Every other
//! pair, a transitive dependency included, keeps the predicate above.
//!
//! [HW-DR-0095]: ../../../../docs/decisions/0095-q67-one-library-entry-may-address-the-keys-of-an-entry-it-names-in-requires-and-confluence-holds-over-the-dependency-order.md
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
pub fn check(
    operations: &[Operation],
    sources: &[String],
    requires: &[Vec<usize>],
) -> Vec<ResolveError> {
    let mut out = Vec::new();
    for (index, later) in operations.iter().enumerate() {
        for earlier in &operations[..index] {
            if earlier.source == later.source {
                continue;
            }
            if ordered_by_requires(earlier, later, requires)
                || ordered_by_requires(later, earlier, requires)
            {
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

/// Whether `dependent` writes into `dependency` under the one permission
/// [HW-DR-0095] grants: its source names the other's source directly in
/// `requires`, and it adds rather than replaces or deletes.
///
/// A transitive dependency grants nothing. The ruling says "an entry it
/// names", and an entry that reaches another only through a third has not
/// named it.
///
/// [HW-DR-0095]: ../../../../docs/decisions/0095-q67-one-library-entry-may-address-the-keys-of-an-entry-it-names-in-requires-and-confluence-holds-over-the-dependency-order.md
fn ordered_by_requires(
    dependent: &Operation,
    dependency: &Operation,
    requires: &[Vec<usize>],
) -> bool {
    matches!(dependent.kind, OpKind::Add | OpKind::AddTo)
        && requires
            .get(dependent.source)
            .is_some_and(|named| named.contains(&dependency.source))
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
        errors_requiring(sources, &[])
    }

    /// The same, where each `(dependent, dependency)` pair is a source that
    /// names another in `requires`.
    fn errors_requiring(sources: &[&str], pairs: &[(usize, usize)]) -> Vec<ResolveError> {
        let mut requires = vec![Vec::new(); sources.len()];
        for (dependent, dependency) in pairs {
            requires[*dependent].push(*dependency);
        }
        check(&operations(sources), &names(sources.len()), &requires)
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

    const DEPENDENCY: &str = "add:\n  kinds.design_spec: {facets: {require: [status]}}\n";
    const DEPENDENT: &str = "add_to:\n  kinds.design_spec.facets.require: [mode]\n";

    /// HW-DR-0095: an entry may write into an entry it names in `requires`,
    /// and the declared dependency orders the pair.
    #[test]
    fn a_dependent_add_to_into_its_dependency_commutes_by_order() {
        let found = errors_requiring(&[DEPENDENCY, DEPENDENT], &[(1, 0)]);
        assert!(found.is_empty(), "{found:?}");
    }

    /// The check reads pairs and not an order, so the permission holds when
    /// the dependent is the earlier of the two sources as well.
    #[test]
    fn the_permission_holds_whichever_of_the_two_sources_is_first() {
        let found = errors_requiring(&[DEPENDENT, DEPENDENCY], &[(0, 1)]);
        assert!(found.is_empty(), "{found:?}");
    }

    /// Two `add` operations that write one leaf state two values for it, and
    /// no order makes that a write into the dependency. The pair keeps the
    /// leaf predicate, so the refusal names both bundles rather than leaving
    /// the merge to report a collision that names one.
    #[test]
    fn a_dependent_add_over_a_leaf_its_dependency_sets_is_refused() {
        let found = errors_requiring(
            &[
                "add:\n  kinds.design_spec: {purpose: behavior}\n",
                "add:\n  kinds.design_spec.purpose: rationale\n",
            ],
            &[(1, 0)],
        );
        assert_eq!(found.len(), 1, "{found:?}");
        let text = found[0].to_string();
        assert!(text.contains("kinds.design_spec.purpose"), "{text}");
        assert!(text.contains("source-0"), "{text}");
    }

    #[test]
    fn the_same_add_to_without_requires_is_refused() {
        let found = errors(&[DEPENDENCY, DEPENDENT]);
        assert_eq!(found.len(), 1, "{found:?}");
        let text = found[0].to_string();
        assert!(text.contains("kinds.design_spec.facets.require"), "{text}");
        assert!(found[0].source == "source-1", "{:?}", found[0]);
        assert!(text.contains("source-0"), "{text}");
    }

    /// "An entry it names": C requires B and B requires A, so C has not named A.
    #[test]
    fn a_transitive_dependency_grants_no_write() {
        let found = errors_requiring(
            &[
                DEPENDENCY,
                "add:\n  kinds.other: {purpose: behavior}\n",
                DEPENDENT,
            ],
            &[(1, 0), (2, 1)],
        );
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].source, "source-2");
    }

    /// The permission is for `add` and `add_to`. An `override` from the
    /// dependent still owns its subtree and still meets the dependency.
    #[test]
    fn a_dependent_override_into_its_dependency_is_refused() {
        let found = errors_requiring(
            &[
                DEPENDENCY,
                "override:\n  kinds.design_spec.facets: {require: [mode]}\n",
            ],
            &[(1, 0)],
        );
        assert_eq!(found.len(), 1, "{found:?}");
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
