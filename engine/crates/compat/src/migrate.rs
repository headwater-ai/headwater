// SPDX-License-Identifier: Apache-2.0
//! Where in a corpus the value a migration step names actually sits.
//!
//! [`crate::subjects`] answers which documents a step names. That answer is
//! enough for a report and it is not enough for a write: a writer needs the
//! front-matter key that holds the old value, and it needs to be told when the
//! document holds no such key at all.
//!
//! # A mechanical step is not the same claim as a writable one
//!
//! [`headwater_resolve::migration::Application::over`] derives the mechanical
//! half from the target list, and that derivation is about the *payload*: one
//! target means nobody is asked anything. Whether a byte of a document can be
//! rewritten is a different question, about the *corpus*, and the census
//! answers it.
//!
//! A kind is the case where the two come apart. A homogeneous shelf carries the
//! kind by placement, so a document typed that way declares the kind nowhere
//! and there is no byte to rewrite. The remedy is to move the file, which is a
//! shelf question rather than a front-matter one. A heterogeneous shelf reads a
//! discriminator facet, and that facet is a front-matter key like any other.
//! [`Site`] is the difference, and it is a value rather than an omission so
//! that a run reports the documents it will not write instead of answering
//! nothing about them.
//!
//! # The empty arm has a location
//!
//! A step that reaches [`Site::Placement`] for every document it names writes
//! nothing, and a caller that filtered the placement arm out would print "0
//! documents written" with no statement of why. So the arm carries the shelf
//! that carries the kind, which is the thing a reader has to act on.

use headwater_census::census::{Census, Outcome as Row};
use headwater_census::resolve::Step as Derivation;
use headwater_resolve::migration::{Step, Subject};

/// Where one document holds the value one step names.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Site {
    /// The front matter of this document declares the value under `key`. A
    /// writer rewrites that key's value and nothing else.
    Front {
        path: String,
        /// The front-matter key: the facet of a `facet_value` step, and the
        /// discriminator of a `kind` step over a heterogeneous shelf.
        key: String,
    },
    /// The shelf carries the kind, so no byte of this document holds it.
    Placement { path: String, shelf: String },
}

impl Site {
    pub fn path(&self) -> &str {
        match self {
            Site::Front { path, .. } | Site::Placement { path, .. } => path,
        }
    }

    /// One line a reader of a report acts on, for a site nothing writes.
    pub fn why(&self) -> Option<String> {
        match self {
            Site::Front { .. } => None,
            Site::Placement { path, shelf } => Some(format!(
                "{path} takes its kind from `{shelf}`, which is homogeneous, so the document \
                 declares the kind nowhere. The remedy is to move the file"
            )),
        }
    }
}

/// Every place in this corpus that one step of a migration payload names.
///
/// **Read off the census of the taxonomy this repository takes**, which is the
/// taxonomy the step's `from` was written against. A reading taken under the
/// candidate would ask a taxonomy that no longer declares the value which
/// documents carry it, and would answer none of them for every step.
///
/// A facet value is matched in either spelling a document may write it: one
/// scalar, or a sequence that holds it among others.
pub fn sites(step: &Step, census: &Census) -> Vec<Site> {
    let mut sites: Vec<Site> = census
        .rows
        .iter()
        .filter_map(|row| match &step.subject {
            Subject::Kind => match &row.outcome {
                Row::Typed { kind, derivation } if *kind == step.from => {
                    Some(kinded(&row.path, derivation))
                }
                _ => None,
            },
            Subject::FacetValue { facet } => row
                .document
                .as_ref()
                .and_then(|document| document.facets.get(facet))
                .filter(|node| carries(&node.value, &step.from))
                .map(|_| Site::Front {
                    path: row.path.clone(),
                    key: facet.clone(),
                }),
        })
        .collect();
    sites.sort();
    sites.dedup();
    sites
}

/// Which of the two shapes a typed row is, from the derivation that typed it.
///
/// The last step of the derivation is the one that produced the kind, and the
/// two arms that produce one are the two arms below. A derivation that reached
/// a kind and ended on neither is a state `headwater_census::resolve` does not
/// produce, and it reads here as a placement site named after no shelf rather
/// than as a document this run would silently rewrite.
fn kinded(path: &str, derivation: &headwater_census::resolve::Resolution) -> Site {
    match derivation.steps.last() {
        Some(Derivation::DiscriminatorRead { facet, .. }) => Site::Front {
            path: path.to_string(),
            key: facet.clone(),
        },
        Some(Derivation::PlacementCarriesTheKind { shelf, .. }) => Site::Placement {
            path: path.to_string(),
            shelf: shelf.clone(),
        },
        Some(Derivation::ShelfMatched { shelf, .. }) => Site::Placement {
            path: path.to_string(),
            shelf: shelf.clone(),
        },
        Some(Derivation::Stopped(_)) | None => Site::Placement {
            path: path.to_string(),
            shelf: "no derivation step named one".to_string(),
        },
    }
}

/// Whether one front-matter value is, or holds, the value a step names.
fn carries(value: &headwater_yaml::Value, wanted: &str) -> bool {
    match value {
        headwater_yaml::Value::Scalar(scalar) => scalar.text == wanted,
        headwater_yaml::Value::Seq(items) => items.iter().any(
            |item| matches!(&item.value, headwater_yaml::Value::Scalar(scalar) if scalar.text == wanted),
        ),
        headwater_yaml::Value::Map(_) => false,
    }
}
