// SPDX-License-Identifier: Apache-2.0
//! A Shape-origin check, generated from the shelf declarations.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#placement-is-primary-metadata-fills-the-gap)
//! states the rule and says it is the one with teeth: "**a homogeneous shelf
//! forbids the discriminator facet.** If the directory already says what a
//! document is, a restatement in front matter creates a second truth that will
//! eventually disagree with the first."
//!
//! Nothing below names a facet or a shelf. The set of discriminator facets is
//! read off the heterogeneous shelves of the resolved taxonomy, and the shelf
//! each document sits on comes from the derivation the census already recorded.
//! A taxonomy that declares one more shelf gets one more instance and this file
//! does not change.
//!
//! # Why the heterogeneous half is a skip and not a pass
//!
//! The rule has two halves, and only one of them is a check. On a heterogeneous
//! shelf the discriminator is what metadata is *for*, and kind resolution
//! already decided it: a document whose discriminator is missing, unreadable or
//! not admitted carries no kind at all, and the census reports it as untyped.
//! So a typed document on a heterogeneous shelf satisfies that half by
//! construction, and a check over it would be a second account of one fact that
//! cannot disagree with the first.
//!
//! An instance that reports a foregone pass is worse than no instance, because
//! coverage then counts a document as checked by a rule that could never have
//! failed. So the instance exists and it is **skipped with a reason**, which is
//! what [spec 4](../../../../docs/spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)
//! asks for: visible, never silent.
//!
//! # Where the finding reports
//!
//! At the restated facet, because that is the line to delete. The fix is
//! mechanical and total: the shelf already carries the kind, so removing the
//! key loses nothing.

use crate::finding::{at, Finding, Severity};
use crate::instance::Instance;
use headwater_census::census::{Census, Outcome};
use headwater_census::resolve::Step;
use headwater_census::shelves::{ShelfBody, Taxonomy};

pub const RULE: &str = "shelf.placement_is_primary";

const HETEROGENEOUS: &str =
    "the shelf is heterogeneous, so the discriminator is the gap metadata fills, \
     and kind resolution already read it";

/// Instantiate the check over a census: one instance per typed document.
pub fn run(census: &Census, taxonomy: &Taxonomy) -> Vec<Instance> {
    // The generation step, in full: every facet that a heterogeneous shelf uses
    // as its discriminator, in declaration order and without repeats.
    let mut discriminators: Vec<&str> = Vec::new();
    for shelf in &taxonomy.shelves {
        let ShelfBody::Heterogeneous { discriminator, .. } = &shelf.body else {
            continue;
        };
        if !discriminators.contains(&discriminator.as_str()) {
            discriminators.push(discriminator);
        }
    }

    let mut instances = Vec::new();
    for row in &census.rows {
        let Outcome::Typed { kind, derivation } = &row.outcome else {
            continue;
        };
        // Which shelf, and which of the two bodies it has, from the derivation
        // the census recorded. Reading it back costs nothing and it cannot
        // disagree with the resolution that produced the kind.
        let Some(shelf) = derivation.steps.iter().find_map(|step| match step {
            Step::PlacementCarriesTheKind { shelf, .. } => Some(shelf),
            _ => None,
        }) else {
            instances.push(Instance::skipped(RULE, row.path.clone(), HETEROGENEOUS));
            continue;
        };

        let Some(document) = &row.document else {
            // A typed row always carries the document it read, so this is
            // unreachable. It is a skip rather than a panic for the reason a
            // check never panics: one bad row must not silence the corpus.
            instances.push(Instance::skipped(
                RULE,
                row.path.clone(),
                "the row carries no document to read",
            ));
            continue;
        };

        let restated = discriminators
            .iter()
            .find_map(|facet| document.facets.entry(facet).map(|entry| (*facet, entry)));

        instances.push(match restated {
            None => Instance::passed(RULE, row.path.clone()),
            Some((facet, entry)) => {
                let (line, column) = at(Some(entry.key.span));
                let value = entry
                    .value
                    .value
                    .as_scalar()
                    .map(|scalar| scalar.text.clone())
                    .unwrap_or_else(|| entry.value.value.kind_name().to_string());
                Instance::failed(
                    RULE,
                    row.path.clone(),
                    Finding {
                        rule: RULE,
                        severity: Severity::Error,
                        obligation: None,
                        path: row.path.clone(),
                        line,
                        column,
                        message: format!(
                            "`{shelf}` is homogeneous and carries the kind `{kind}`, \
                             and this document restates it as `{facet}: {value}`"
                        ),
                        remediation: format!(
                            "remove `{facet}` from the front matter of {}",
                            row.path
                        ),
                        fixable: true,
                    },
                )
            }
        });
    }
    instances
}
