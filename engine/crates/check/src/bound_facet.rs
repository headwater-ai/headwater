// SPDX-License-Identifier: Apache-2.0
//! A Shape-origin check, generated from the shelves that bind a facet.
//!
//! A shelf path can bind a facet to one of its segments:
//! `docs/contexts/{context}/contracts/**` says that the folder under
//! `contexts/` is the value of `context` for every document below it
//! ([`headwater_meta::pattern`] states the form). The owner's ruling on #2101
//! decides what the binding asks of a document: the document states the value
//! in its front matter too, and the engine compares the two. A value that
//! disagrees with the folder is a finding, and so is a document on the shelf
//! with no value at all. The engine does not fill the value in from the folder,
//! because a value that only `check` derived would give `query`, `generate` and
//! the JSON export a different answer to the same question.
//!
//! That is the one stated exception to
//! [spec 2, "Placement is primary"](../../../../docs/spec/taxonomy-model.md#placement-is-primary-metadata-fills-the-gap),
//! which says a document does not restate what its placement states. Spec 2
//! gives the reason and the condition that retires the absent-value half.
//!
//! # What instantiates
//!
//! Only the kinds that a bound shelf can carry, so a taxonomy that binds no
//! facet gets no instance at all, and this repository is one. A document of
//! such a kind on a shelf that binds nothing is skipped with a reason, as
//! [`crate::placement`] skips its heterogeneous half.
//!
//! # Which shelf
//!
//! The shelf comes from [`headwater_census::resolve::shelf_for`] over the path,
//! not from [`DocumentView::placed_on`]. That one answers only for a
//! homogeneous shelf, and a heterogeneous shelf can bind a facet too.
//!
//! # One fact, one finding
//!
//! Where the kind already requires the bound facet, an absent value is
//! `facet.required.missing`, and this rule skips that document with the reason
//! rather than report the same absence twice. A value that is present and
//! disagrees is this rule's in every case.
//!
//! # No patch
//!
//! A disagreement has two remedies, moving the file or editing the value, and
//! which one is right is a judgment. So the finding names both and writes
//! neither.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::scope::{DocumentCheck, DocumentView};
use crate::shape::Shape;
use headwater_census::resolve::{shelf_for, ShelfMatch};
use headwater_census::shelves::{ShelfBody, Taxonomy};
use std::collections::BTreeSet;

pub const RULE: &str = "shelf.bound_facet.disagrees";

const UNBOUND: &str =
    "the document is on a shelf that binds no facet, so its folder states no value";

const OFF_SHELF: &str = "no one shelf claims the document's path, so no folder states a value";

/// The check, generated from the shelf declarations.
pub struct BoundFacet {
    /// Every shelf of the taxonomy, bound or not, because the shelf that wins
    /// a path can be one that binds nothing.
    shelves: Taxonomy,
    /// Every kind a bound shelf can carry. The instances.
    kinds: BTreeSet<String>,
    /// Each of those kinds and the facets it requires, for the skip above.
    required: Vec<(String, Vec<String>)>,
}

impl BoundFacet {
    /// The generation step, in full.
    pub fn over(taxonomy: &Taxonomy, shape: &Shape) -> Self {
        let mut kinds = BTreeSet::new();
        for shelf in &taxonomy.shelves {
            if shelf.pattern.bound().is_empty() {
                continue;
            }
            match &shelf.body {
                ShelfBody::Homogeneous { kind } => {
                    kinds.insert(kind.clone());
                }
                ShelfBody::Heterogeneous { kinds: carried, .. } => {
                    kinds.extend(carried.iter().cloned());
                }
            }
        }
        let required = kinds
            .iter()
            .map(|kind| (kind.clone(), shape.required_facets(kind)))
            .collect();
        BoundFacet {
            shelves: Taxonomy {
                shelves: taxonomy.shelves.clone(),
                kinds: Vec::new(),
            },
            kinds,
            required,
        }
    }

    fn requires(&self, kind: &str, facet: &str) -> bool {
        self.required
            .iter()
            .any(|(known, facets)| known == kind && facets.iter().any(|owed| owed == facet))
    }
}

impl DocumentCheck for BoundFacet {
    const RULE: &'static str = self::RULE;
    /// Which edition of this rule reached a verdict ([`crate::cache`]).
    ///
    /// Raise the rule version when a rule's behavior changes.
    const VERSION: u32 = 1;

    fn instantiates(&self, kind: &str) -> bool {
        self.kinds.contains(kind)
    }

    fn evaluate(&self, view: &DocumentView<'_>) -> Outcome {
        let ShelfMatch::Matched { shelf, .. } = shelf_for(view.path(), &self.shelves) else {
            return Outcome::Skipped(OFF_SHELF.to_string());
        };
        let bindings = shelf.pattern.bindings(view.path()).unwrap_or_default();
        if bindings.is_empty() {
            return Outcome::Skipped(UNBOUND.to_string());
        }
        let name = &shelf.name;
        let kind = view.kind();
        let mut findings = Vec::new();
        let mut owned_elsewhere = Vec::new();
        for (facet, folder) in &bindings {
            match view.facets().entry(facet) {
                Some(entry) => {
                    let value = entry.value.value.as_scalar();
                    if value.map(|scalar| scalar.text.as_str()) == Some(folder.as_str()) {
                        continue;
                    }
                    // A folder name is a scalar. A null, a sequence or a
                    // mapping names no folder to move the document under, so
                    // the one remedy is the value the folder gives.
                    let (stated, elsewhere) = match value {
                        Some(scalar) if headwater_yaml::core_schema::as_null(scalar) => {
                            (format!("a null `{facet}`"), String::new())
                        }
                        Some(scalar) => (
                            format!("`{facet}: {}`", scalar.text),
                            format!(
                                ", or move the document under the folder for `{}`",
                                scalar.text
                            ),
                        ),
                        None => (
                            format!("`{facet}` as {}", entry.value.value.kind_name()),
                            String::new(),
                        ),
                    };
                    let (line, column) = at(Some(entry.key.span));
                    findings.push(Finding {
                        rule: self::RULE,
                        severity: Severity::Error,
                        obligation: None,
                        path: view.path().to_string(),
                        line,
                        column,
                        message: format!(
                            "`{name}` binds `{facet}` to the folder `{folder}`, \
                             and this document declares {stated}"
                        ),
                        remediation: format!(
                            "set `{facet}: {folder}` in the front matter of {}{elsewhere}",
                            view.path()
                        ),
                        patch: None,
                    });
                }
                None if self.requires(kind, facet) => owned_elsewhere.push(facet.as_str()),
                None => findings.push(Finding {
                    rule: self::RULE,
                    severity: Severity::Error,
                    obligation: None,
                    path: view.path().to_string(),
                    // The front matter as a whole: the key is absent, and the
                    // block that should carry it opens on line 1.
                    line: 1,
                    column: 1,
                    message: format!(
                        "`{name}` binds `{facet}` to the folder `{folder}`, \
                         and this document declares no `{facet}`"
                    ),
                    remediation: format!(
                        "add `{facet}: {folder}` to the front matter of {}",
                        view.path()
                    ),
                    patch: None,
                }),
            }
        }
        if findings.is_empty() && !owned_elsewhere.is_empty() {
            return Outcome::Skipped(format!(
                "`{kind}` requires `{}`, so `facet.required.missing` reports its absence",
                owned_elsewhere.join("`, `")
            ));
        }
        Outcome::failed(findings)
    }
}
