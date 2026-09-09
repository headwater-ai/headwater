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
//!
//! # The scope, and what it costs this check
//!
//! [`DocumentCheck`] is the whole declaration: one document, and the front
//! matter without the body. The rule needs no more, and the type is now what
//! says so ([spec 12](../../../../docs/spec/12-check-layer.md#the-declaration-is-a-type-not-a-returned-value)).
//!
//! The taxonomy is not in that view, so the discriminator set is read once
//! when the check is built rather than per document. That is the right split
//! and not a workaround: the generation step reads the taxonomy, and the
//! evaluation step reads one document.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView};
use headwater_census::shelves::{ShelfBody, Taxonomy};

pub const RULE: &str = "shelf.placement_is_primary";

const HETEROGENEOUS: &str =
    "the shelf is heterogeneous, so the discriminator is the gap metadata fills, \
     and kind resolution already read it";

/// The check, generated from the shelf declarations.
pub struct Placement {
    /// Every facet that a heterogeneous shelf uses as its discriminator, in
    /// declaration order and without repeats.
    discriminators: Vec<String>,
}

impl Placement {
    /// The generation step, in full.
    pub fn over(taxonomy: &Taxonomy) -> Self {
        let mut discriminators: Vec<String> = Vec::new();
        for shelf in &taxonomy.shelves {
            let ShelfBody::Heterogeneous { discriminator, .. } = &shelf.body else {
                continue;
            };
            if !discriminators.contains(discriminator) {
                discriminators.push(discriminator.clone());
            }
        }
        Placement { discriminators }
    }
}

impl DocumentCheck for Placement {
    const RULE: &'static str = self::RULE;
    /// The first edition of this rule. Raise it when what the rule decides
    /// changes, because that is what invalidates the cached verdicts of the
    /// edition before it ([`crate::cache`]).
    const VERSION: u32 = 2;

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        // Which of the two bodies the shelf has, from the derivation the
        // census recorded. Reading it back costs nothing and it cannot
        // disagree with the resolution that produced the kind.
        let Some(shelf) = view.placed_on() else {
            return Outcome::Skipped(HETEROGENEOUS.to_string());
        };

        let restated = std::iter::once("kind")
            .chain(self.discriminators.iter().map(String::as_str))
            .find_map(|facet| view.facets().entry(facet).map(|entry| (facet, entry)));

        let Some((facet, entry)) = restated else {
            return Outcome::Passed;
        };

        let (line, column) = at(Some(entry.key.span));
        let value = entry
            .value
            .value
            .as_scalar()
            .map(|scalar| scalar.text.clone())
            .unwrap_or_else(|| entry.value.value.kind_name().to_string());
        let kind = view.kind();
        Outcome::failed_with(Finding {
            rule: self::RULE,
            severity: Severity::Error,
            obligation: None,
            path: view.path().to_string(),
            line,
            column,
            message: format!(
                "`{shelf}` is homogeneous and carries the kind `{kind}`, \
                 and this document restates it as `{facet}: {value}`"
            ),
            remediation: format!("remove `{facet}` from the front matter of {}", view.path()),
            // No patch, and this rule used to claim one. The remedy is
            // mechanical — spec 12 puts "normalize front-matter key order"
            // among its own examples — and the engine does not write it. What
            // [`crate::Patch`] carries is what `check --fix` will do, so the
            // claim goes with the capability. A deletion inside front matter
            // needs a read-back guard over a mapping, and the two shapes that
            // ship guard a run of prose and a `relations:` block.
            // [HW-OBL-0103](../../../../docs/obligations/0103-the-front-matter-half-of-a-patch-has-no-writer.md)
            // holds the remainder.
            patch: None,
        })
    }
}
