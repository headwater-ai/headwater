// SPDX-License-Identifier: Apache-2.0
//! The static check that an overlay set commutes, run before anything merges.
//!
//! [The taxonomy model](../../../../docs/spec/taxonomy-model.md#customization-by-composition):
//! "Overlay application must be **confluent**: the application of a set of
//! overlays in any legal order yields the same resolved taxonomy. The resolver
//! checks this statically, before it applies anything." And: "the resolver
//! rejects any pair that does not commute, and it names both overlays and the
//! contested path."
//!
//! Static is the whole value of it. A resolver that discovered a conflict
//! while merging would report whichever conflict the order it happened to pick
//! reached first, and a publisher could not prove ahead of a release that every
//! subset of its bundles resolves ([distribution and federation](../../../../docs/spec/distribution-and-federation.md#bundles-are-publisher-overlays-in-the-other-direction)).
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
//! Two pairs are ordered rather than tested. [HW-DR-0095] (Q67) lets a library
//! entry write into an entry it names directly in `requires`, with `add` or
//! `add_to`. The declared dependency applies first, so the pair has one order
//! and nothing is left for the order of application to decide. The resolver
//! puts the dependency first before it merges ([`crate::order`]).
//!
//! The second pair is the adopter's overlay and a selected bundle. The owner
//! ruled on 2026-11-22 (#2022) that an adopter's overlay is ordered after every
//! bundle it selects, so it may `override` and `remove` what those bundles add.
//! The bundle applies first, so every operation of the adopter's overlay has
//! one order against it, except an `add` against an `add`: two values for one
//! leaf have no answer in any order.
//!
//! Every other pair keeps the predicate above: a transitive dependency, two
//! bundles beside an adopter's overlay, and an assembly's glue overlay, which
//! is not an adopter's overlay.
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
///
/// `requires[s]` holds the sources that source `s` names in `requires`, and
/// `after[s]` holds the selected bundles that source `s` applies after because
/// it is the adopter's overlay. Both are indexed by source.
pub fn check(
    operations: &[Operation],
    sources: &[String],
    requires: &[Vec<usize>],
    after: &[Vec<usize>],
) -> Vec<ResolveError> {
    let mut out = Vec::new();
    for (index, later) in operations.iter().enumerate() {
        for earlier in &operations[..index] {
            if earlier.source == later.source {
                continue;
            }
            if ordered_by_requires(earlier, later, requires)
                || ordered_by_requires(later, earlier, requires)
                || ordered_as_adopter(earlier, later, after)
                || ordered_as_adopter(later, earlier, after)
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
    // Two `add` operations keep the leaf predicate. A dependent `add` of a new
    // key already commutes with its dependency's `add` on disjoint leaves, and
    // one that sets a leaf the dependency sets is two values for one leaf,
    // which no order resolves.
    let both_add = dependent.kind == OpKind::Add && dependency.kind == OpKind::Add;
    matches!(dependent.kind, OpKind::Add | OpKind::AddTo)
        && !both_add
        && requires
            .get(dependent.source)
            .is_some_and(|named| named.contains(&dependency.source))
}

/// Whether `adopter` comes from the adopter's overlay and `bundle` from a
/// selected bundle that it applies after (#2022). Every operation is ordered
/// that way except an `add` against an `add`, which keeps the leaf predicate
/// for the reason [`ordered_by_requires`] gives.
fn ordered_as_adopter(bundle: &Operation, adopter: &Operation, after: &[Vec<usize>]) -> bool {
    let both_add = adopter.kind == OpKind::Add && bundle.kind == OpKind::Add;
    !both_add
        && after
            .get(adopter.source)
            .is_some_and(|bundles| bundles.contains(&bundle.source))
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
        let after = vec![Vec::new(); sources.len()];
        check(
            &operations(sources),
            &names(sources.len()),
            &requires,
            &after,
        )
    }

    /// The same, where each `(adopter, bundle)` pair is the adopter's overlay
    /// and a selected bundle it applies after (#2022).
    fn errors_after(sources: &[&str], pairs: &[(usize, usize)]) -> Vec<ResolveError> {
        let requires = vec![Vec::new(); sources.len()];
        let mut after = vec![Vec::new(); sources.len()];
        for (adopter, bundle) in pairs {
            after[*adopter].push(*bundle);
        }
        check(
            &operations(sources),
            &names(sources.len()),
            &requires,
            &after,
        )
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

    /// The two-`add` rule reads both kinds. A dependent `add` under a key its
    /// dependency replaces with `override` is a write into the dependency, and
    /// the declared order admits it. Without `requires` the same pair is
    /// refused, because the `override` owns its subtree.
    #[test]
    fn a_dependent_add_under_a_key_its_dependency_overrides_is_ordered() {
        let sources = [
            "override:\n  kinds.specification: {purpose: behavior}\n",
            "add:\n  kinds.specification.voice: declarative\n",
        ];
        let found = errors_requiring(&sources, &[(1, 0)]);
        assert!(found.is_empty(), "{found:?}");
        assert_eq!(errors(&sources).len(), 1);
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

    const BUNDLE: &str = "add:\n  kinds.report: {purpose: behavior, facets: {require: [status]}}\n";

    /// #2022: the adopter's overlay applies after every selected bundle, so its
    /// `override` of what a bundle adds has one order.
    #[test]
    fn an_adopter_override_into_a_bundle_is_ordered() {
        let sources = [BUNDLE, "override:\n  kinds.report.purpose: rationale\n"];
        assert!(errors_after(&sources, &[(1, 0)]).is_empty());
        assert_eq!(
            errors(&sources).len(),
            1,
            "without the order it is a conflict"
        );
    }

    #[test]
    fn an_adopter_remove_of_what_a_bundle_adds_is_ordered() {
        let sources = [BUNDLE, "remove:\n  - kinds.report\n"];
        assert!(errors_after(&sources, &[(1, 0)]).is_empty());
        assert_eq!(
            errors(&sources).len(),
            1,
            "without the order it is a conflict"
        );
    }

    #[test]
    fn an_adopter_list_operation_on_a_bundle_list_is_ordered() {
        let sources = [
            BUNDLE,
            "add_to:\n  kinds.report.facets.require: [mode]\nremove_from:\n  kinds.report.facets.require: [status]\n",
        ];
        assert!(errors_after(&sources, &[(1, 0)]).is_empty());
    }

    /// The permission is read from `after`, so it holds whichever of the two
    /// operations the check meets first.
    #[test]
    fn the_adopter_order_holds_whichever_source_is_first() {
        let sources = ["override:\n  kinds.report.purpose: rationale\n", BUNDLE];
        assert!(errors_after(&sources, &[(0, 1)]).is_empty());
    }

    /// Two values for one leaf have no answer in any order, so an adopter `add`
    /// over a leaf a bundle adds keeps the leaf predicate.
    #[test]
    fn an_adopter_add_over_a_bundle_leaf_is_refused() {
        let found = errors_after(
            &[BUNDLE, "add:\n  kinds.report.purpose: rationale\n"],
            &[(1, 0)],
        );
        assert_eq!(found.len(), 1, "{found:?}");
    }

    /// Only `add` against `add` keeps the predicate. An adopter `add` against a
    /// bundle `override`, `remove` or `add_to` that reaches the same subtree is
    /// ordered like the adopter's other operations.
    #[test]
    fn an_adopter_add_against_a_bundle_operation_that_is_not_an_add_is_ordered() {
        let adopter = "add:\n  kinds.report.identifier: {scheme: report_id}\n";
        for bundle in [
            "override:\n  kinds.report: {purpose: rationale}\n",
            "remove:\n  - kinds.report\n",
            "add_to:\n  kinds.report.identifier.aliases: [rpt]\n",
        ] {
            let sources = [bundle, adopter];
            assert!(
                errors_after(&sources, &[(1, 0)]).is_empty(),
                "{bundle} then the adopter's add is ordered"
            );
            assert_eq!(
                errors(&sources).len(),
                1,
                "without the order, {bundle} and the add are a conflict"
            );
        }
    }

    /// The order runs one way: a bundle that the adopter applies after gets no
    /// permission over the adopter's own writes, and two bundles beside it keep
    /// the predicate.
    #[test]
    fn the_adopter_order_grants_nothing_to_a_bundle() {
        let found = errors_after(
            &[
                "override:\n  kinds.report.purpose: rationale\n",
                "add:\n  kinds.report.purpose: behavior\n",
                BUNDLE,
            ],
            &[(0, 2)],
        );
        // Source 0 against source 1 is not ordered; source 1 against source 2
        // is two bundles writing one leaf.
        assert_eq!(found.len(), 2, "{found:?}");
    }

    /// A source with no `after`, such as an assembly's glue overlay, gets no
    /// permission over a bundle.
    #[test]
    fn a_source_that_is_not_the_adopter_overlay_gets_no_order() {
        let found = errors_after(
            &[BUNDLE, "override:\n  kinds.report.purpose: rationale\n"],
            &[],
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
