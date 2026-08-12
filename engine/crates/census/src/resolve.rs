// SPDX-License-Identifier: Apache-2.0
//! Kind resolution, and the derivation it records.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#kind-resolution) states
//! four steps, and `headwater explain <path>` prints the derivation: "which
//! shelf matched, which rule fired, and the kind's declared purpose.
//! Classification is never a black box, for a human or an agent." So the
//! derivation is not a debug aid that a later release adds. Every resolution
//! returns one, including the ones that fail, because the derivation of a
//! failure is what tells an author which of the four steps to repair.
//!
//! # Three steps run, and the fourth has no declared form
//!
//! Step 4 is "any path-pattern refinement that the shelf declares". No schema in
//! this repository declares one, and nothing in the specification, the base
//! package or the design-spec bundle gives the declaration a syntax. An engine
//! cannot implement a step whose input has no form, and inventing one here would
//! make the engine the authority on a declaration the taxonomy owns. The step is
//! therefore absent rather than approximated, and
//! [13 — Open obligations](../../../../docs/spec/13-open-obligations.md) carries
//! the finding.
//!
//! # Why an unresolved kind is not an error
//!
//! Resolution returns an [`Outcome`], never a `Result`. A document that no shelf
//! claims and a document whose discriminator is missing are both ordinary states
//! of a corpus, and [spec 4](../../../../docs/spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for)
//! requires that they be counted rather than dropped. A `Result` would invite a
//! caller to use `?`, and the file would leave the denominator at that line.

use crate::shelves::{Shelf, ShelfBody, Taxonomy};
use headwater_yaml::{Mapping, Span};

/// What resolution made of one path and its front matter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// A concrete kind.
    Typed(String),
    /// No kind, for a reason that names the step that stopped.
    Untyped(Untyped),
}

/// The closed set of reasons a document on a shelf carries no kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Untyped {
    /// Step 1 found nothing. The path is outside every declared shelf.
    NoShelf,
    /// Step 1 found two shelves of equal specificity. Spec 2 calls this a
    /// schema-validation error rather than a runtime coin-flip, and the
    /// validation is [M2](https://github.com/headwater-ai/headwater/milestone/2).
    /// Until it exists, resolution reports the tie and picks neither.
    AmbiguousShelf { shelves: Vec<String> },
    /// Step 3 read no discriminator on a heterogeneous shelf.
    MissingDiscriminator { facet: String },
    /// Step 3 read a discriminator that the shelf does not admit.
    UnknownDiscriminator {
        facet: String,
        value: String,
        permitted: Vec<String>,
    },
    /// The discriminator is a sequence or a mapping, and a kind name is a word.
    DiscriminatorNotAScalar { facet: String },
    /// Spec 2: no document resolves to an abstract kind. A shelf that declares
    /// one, or a discriminator value that names one, is a taxonomy defect that
    /// shows up here first because this is the layer that has a document.
    AbstractKind { kind: String },
}

impl std::fmt::Display for Untyped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Untyped::NoShelf => write!(f, "no shelf pattern claims this path"),
            Untyped::AmbiguousShelf { shelves } => write!(
                f,
                "{} claim this path with equal specificity, and a tie is not a coin flip",
                shelves.join(" and ")
            ),
            Untyped::MissingDiscriminator { facet } => write!(
                f,
                "the shelf is heterogeneous and the document declares no `{facet}`"
            ),
            Untyped::UnknownDiscriminator {
                facet,
                value,
                permitted,
            } => write!(
                f,
                "`{facet}: {value}` is not a kind this shelf admits ({})",
                permitted.join(", ")
            ),
            Untyped::DiscriminatorNotAScalar { facet } => {
                write!(f, "`{facet}` is not a scalar, and a kind name is a word")
            }
            Untyped::AbstractKind { kind } => {
                write!(
                    f,
                    "`{kind}` is abstract, and no document is an abstract kind"
                )
            }
        }
    }
}

/// One step of the derivation, in the order it ran.
///
/// This is what `headwater explain` prints. It is a value rather than a string
/// so that the MCP surface and the text renderer say the same thing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// Step 1: a shelf pattern matched, and it was the most specific one.
    ShelfMatched {
        shelf: String,
        pattern: String,
        /// The other shelves that also matched and lost, most specific first.
        beaten: Vec<String>,
    },
    /// Step 2: the shelf is homogeneous, so placement carries the kind.
    PlacementCarriesTheKind { shelf: String, kind: String },
    /// Step 3: the shelf is heterogeneous, so a facet carries the kind.
    DiscriminatorRead {
        shelf: String,
        facet: String,
        value: String,
    },
    /// Where the derivation stopped, if it did not reach a kind.
    Stopped(Untyped),
}

impl std::fmt::Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Step::ShelfMatched {
                shelf,
                pattern,
                beaten,
            } => {
                write!(f, "shelf `{shelf}` matched on `{pattern}`")?;
                if !beaten.is_empty() {
                    write!(f, ", over {}", beaten.join(", "))?;
                }
                Ok(())
            }
            Step::PlacementCarriesTheKind { shelf, kind } => write!(
                f,
                "`{shelf}` is homogeneous, so placement carries the kind `{kind}`"
            ),
            Step::DiscriminatorRead {
                shelf,
                facet,
                value,
            } => write!(
                f,
                "`{shelf}` is heterogeneous, so `{facet}: {value}` carries the kind"
            ),
            Step::Stopped(reason) => write!(f, "stopped: {reason}"),
        }
    }
}

/// A resolution and the derivation that produced it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolution {
    pub outcome: Outcome,
    pub steps: Vec<Step>,
    /// Where a finding about this resolution anchors. It is the span of the
    /// discriminator when a discriminator was read, and `None` when the
    /// derivation never reached the front matter — a path is not a span.
    pub span: Option<Span>,
}

impl Resolution {
    pub fn kind(&self) -> Option<&str> {
        match &self.outcome {
            Outcome::Typed(kind) => Some(kind),
            Outcome::Untyped(_) => None,
        }
    }

    /// The derivation, one step per line, in the order the steps ran.
    pub fn explain(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        for step in &self.steps {
            let _ = writeln!(out, "{step}");
        }
        out
    }
}

/// Resolve a kind from a path and the document's front matter.
///
/// `path` is relative to the repository root and written with `/`, because that
/// is the form a shelf pattern is written in and the form a finding reports.
pub fn resolve(path: &str, facets: &Mapping, taxonomy: &Taxonomy) -> Resolution {
    let mut steps = Vec::new();

    // --- step 1: the most specific shelf pattern that matches --------------
    let shelf = match shelf_for(path, taxonomy) {
        ShelfMatch::Matched { shelf, step } => {
            steps.push(step);
            shelf
        }
        ShelfMatch::Stopped(stop) => return stopped(steps, stop.reason, stop.span),
    };

    // --- steps 2 and 3: placement, or the discriminator -------------------
    let (kind, span) = match &shelf.body {
        ShelfBody::Homogeneous { kind } => {
            steps.push(Step::PlacementCarriesTheKind {
                shelf: shelf.name.clone(),
                kind: kind.clone(),
            });
            (kind.clone(), None)
        }
        ShelfBody::Heterogeneous {
            discriminator,
            kinds,
        } => {
            let Some(declared) = facets.get(discriminator) else {
                return stopped(
                    steps,
                    Untyped::MissingDiscriminator {
                        facet: discriminator.clone(),
                    },
                    Some(shelf.span),
                );
            };
            let Some(scalar) = declared.value.as_scalar() else {
                return stopped(
                    steps,
                    Untyped::DiscriminatorNotAScalar {
                        facet: discriminator.clone(),
                    },
                    Some(declared.span),
                );
            };
            if !kinds.contains(&scalar.text) {
                return stopped(
                    steps,
                    Untyped::UnknownDiscriminator {
                        facet: discriminator.clone(),
                        value: scalar.text.clone(),
                        permitted: kinds.clone(),
                    },
                    Some(declared.span),
                );
            }
            steps.push(Step::DiscriminatorRead {
                shelf: shelf.name.clone(),
                facet: discriminator.clone(),
                value: scalar.text.clone(),
            });
            (scalar.text.clone(), Some(declared.span))
        }
    };

    // Spec 2 forbids an abstract kind here whichever step produced it: an
    // abstract kind may never be "a shelf's declared kind, and never a
    // discriminator value".
    if taxonomy.kind(&kind).is_some_and(|k| k.is_abstract) {
        return stopped(steps, Untyped::AbstractKind { kind }, span);
    }

    // Step 4 would run here. See the module comment: it has no declared form.

    Resolution {
        outcome: Outcome::Typed(kind),
        steps,
        span,
    }
}

/// What step 1 made of a path.
///
/// A two-variant enum rather than a `Result`, for the reason in the module
/// comment: no shelf claiming a path is an ordinary state of a corpus, and a
/// `Result` invites a caller to write `?` and drop the file out of the
/// denominator at that line.
#[derive(Clone, Debug)]
pub enum ShelfMatch<'a> {
    Matched { shelf: &'a Shelf, step: Step },
    Stopped(Stop),
}

/// Step 1 on its own: which shelf claims this path.
///
/// It is public because it is the one step that needs a path and nothing else.
/// A file with no front matter has no facets to resolve against, and it still
/// has a place in the tree, so "no shelf claims this path" is a thing the census
/// can say about it and "the discriminator is missing" is not.
pub fn shelf_for<'a>(path: &str, taxonomy: &'a Taxonomy) -> ShelfMatch<'a> {
    let mut matched: Vec<&Shelf> = taxonomy
        .shelves
        .iter()
        .filter(|shelf| shelf.pattern.matches(path))
        .collect();
    matched.sort_by_key(|shelf| std::cmp::Reverse(shelf.pattern.specificity()));

    let Some((winner, rest)) = matched.split_first() else {
        return ShelfMatch::Stopped(Stop {
            reason: Untyped::NoShelf,
            span: None,
        });
    };

    let tied: Vec<String> = rest
        .iter()
        .filter(|other| other.pattern.specificity() == winner.pattern.specificity())
        .map(|other| other.name.clone())
        .collect();
    if !tied.is_empty() {
        let mut shelves = vec![winner.name.clone()];
        shelves.extend(tied);
        shelves.sort();
        return ShelfMatch::Stopped(Stop {
            reason: Untyped::AmbiguousShelf { shelves },
            span: Some(winner.span),
        });
    }

    ShelfMatch::Matched {
        shelf: winner,
        step: Step::ShelfMatched {
            shelf: winner.name.clone(),
            pattern: winner.pattern.source().to_string(),
            beaten: rest.iter().map(|other| other.name.clone()).collect(),
        },
    }
}

/// Where a derivation stopped, and the span a finding about it anchors to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stop {
    pub reason: Untyped,
    pub span: Option<Span>,
}

impl Stop {
    /// The stop as a resolution of its own, for a caller that has no front
    /// matter to carry on with.
    pub fn into_resolution(self) -> Resolution {
        stopped(Vec::new(), self.reason, self.span)
    }
}

fn stopped(mut steps: Vec<Step>, reason: Untyped, span: Option<Span>) -> Resolution {
    steps.push(Step::Stopped(reason.clone()));
    Resolution {
        outcome: Outcome::Untyped(reason),
        steps,
        span,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TAXONOMY: &str = "\
kinds:
  governed_document: {abstract: true}
  design_spec: {is_a: governed_document}
  decision_register: {is_a: governed_document}
  evaluation: {is_a: governed_document}
shelves:
  spec_series:
    path: docs/spec/**
    homogeneous: false
    discriminator: doc_type
    kinds: [design_spec, decision_register]
  evaluations:
    path: docs/evaluations/**
    homogeneous: true
    kind: evaluation
";

    fn taxonomy(source: &str) -> Taxonomy {
        let root = headwater_yaml::load(source).expect("the fixture loads");
        Taxonomy::read(root.value.as_map().expect("a mapping")).expect("the fixture reads")
    }

    fn facets(source: &str) -> Mapping {
        headwater_yaml::load(source)
            .expect("the fixture loads")
            .value
            .as_map()
            .expect("a mapping")
            .clone()
    }

    #[test]
    fn placement_carries_the_kind_on_a_homogeneous_shelf() {
        let resolved = resolve(
            "docs/evaluations/naming-survey.md",
            &facets("id: EVAL-HW-naming\n"),
            &taxonomy(TAXONOMY),
        );
        assert_eq!(resolved.kind(), Some("evaluation"));
        assert_eq!(
            resolved.explain(),
            "shelf `evaluations` matched on `docs/evaluations/**`\n\
             `evaluations` is homogeneous, so placement carries the kind `evaluation`\n"
        );
    }

    #[test]
    fn a_discriminator_carries_the_kind_on_a_heterogeneous_shelf() {
        let resolved = resolve(
            "docs/spec/12-check-layer.md",
            &facets("doc_type: design_spec\n"),
            &taxonomy(TAXONOMY),
        );
        assert_eq!(resolved.kind(), Some("design_spec"));
        assert!(resolved.span.is_some(), "a finding anchors at the facet");
    }

    #[test]
    fn a_missing_discriminator_stops_at_step_three_and_says_so() {
        let resolved = resolve(
            "docs/spec/12-check-layer.md",
            &facets("id: SPEC-HW-check-layer\n"),
            &taxonomy(TAXONOMY),
        );
        assert_eq!(
            resolved.outcome,
            Outcome::Untyped(Untyped::MissingDiscriminator {
                facet: "doc_type".into()
            })
        );
        // The derivation of a failure is what says which step to repair.
        assert!(resolved
            .explain()
            .starts_with("shelf `spec_series` matched"));
    }

    #[test]
    fn a_discriminator_the_shelf_does_not_admit_names_the_ones_it_does() {
        let resolved = resolve(
            "docs/spec/12-check-layer.md",
            &facets("doc_type: reference\n"),
            &taxonomy(TAXONOMY),
        );
        assert!(
            resolved
                .explain()
                .contains("not a kind this shelf admits (design_spec, decision_register)"),
            "{}",
            resolved.explain()
        );
    }

    #[test]
    fn a_path_outside_every_shelf_is_untyped_rather_than_an_error() {
        let resolved = resolve(
            "docs/w3id/README.md",
            &facets("id: x\n"),
            &taxonomy(TAXONOMY),
        );
        assert_eq!(resolved.outcome, Outcome::Untyped(Untyped::NoShelf));
        assert!(resolved.span.is_none(), "a path is not a span");
    }

    #[test]
    fn no_document_resolves_to_an_abstract_kind() {
        let source = TAXONOMY.replace("kind: evaluation", "kind: governed_document");
        let resolved = resolve(
            "docs/evaluations/naming-survey.md",
            &facets("id: x\n"),
            &taxonomy(&source),
        );
        assert_eq!(
            resolved.outcome,
            Outcome::Untyped(Untyped::AbstractKind {
                kind: "governed_document".into()
            })
        );
    }

    #[test]
    fn the_most_specific_shelf_wins_and_the_derivation_names_the_loser() {
        let source = format!("{TAXONOMY}  everything:\n    path: docs/**\n    homogeneous: true\n    kind: evaluation\n");
        let resolved = resolve(
            "docs/spec/12-check-layer.md",
            &facets("doc_type: design_spec\n"),
            &taxonomy(&source),
        );
        assert_eq!(resolved.kind(), Some("design_spec"));
        assert!(
            resolved.explain().contains("over everything"),
            "{}",
            resolved.explain()
        );
    }

    #[test]
    fn two_shelves_of_equal_specificity_are_a_tie_and_never_a_coin_flip() {
        let source = format!(
            "{TAXONOMY}  second_series:\n    path: docs/spec/**\n    homogeneous: true\n    kind: evaluation\n"
        );
        let resolved = resolve(
            "docs/spec/12-check-layer.md",
            &facets("doc_type: design_spec\n"),
            &taxonomy(&source),
        );
        assert_eq!(
            resolved.outcome,
            Outcome::Untyped(Untyped::AmbiguousShelf {
                shelves: vec!["second_series".into(), "spec_series".into()]
            })
        );
    }
}
