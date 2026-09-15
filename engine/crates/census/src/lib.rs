// SPDX-License-Identifier: Apache-2.0
//! Kind resolution, and the census that fixes the denominator.
//!
//! Two components, and [spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots)
//! names both a correctness root. They share a crate because they share one
//! failure: each one produces a **systematically green** result when it is
//! wrong. A walk that misses a subtree reports nothing about it. A resolution
//! that assigns the wrong kind runs the wrong checks and passes them. Neither
//! defect raises an error anywhere, so neither is caught by the ordinary
//! mechanism of noticing that something broke.
//!
//! That is why the rules in [`walk`] are written out rather than delegated, and
//! why the fixture tree exists.
//!
//! The glob language that a shelf path and an exclusion are written in used to
//! be here for the same reason, and it is now `headwater_meta::pattern`. It
//! moved because `taxonomy validate` asks whether two shelf patterns can claim
//! one path, and it asks with no corpus in hand. One language and two readers
//! beats two languages that agree until one of them learns a construct.
//!
//! # The order of a run
//!
//! ```no_run
//! use headwater_census::{census, shelves::Taxonomy, walk::{Corpus, Exclusion}};
//!
//! let corpus = Corpus::new(".", "docs").excluding(vec![Exclusion::new(
//!     "docs/taxonomies/**",
//!     "package content",
//! )]);
//! let root = headwater_yaml::load("shelves: {}\n").unwrap();
//! let taxonomy = Taxonomy::read(root.value.as_map().unwrap()).unwrap();
//!
//! let taken = census::take(&corpus, &taxonomy);
//! print!("{}", taken.render(census::Detail::Exceptions));
//! ```
//!
//! # What is deliberately absent
//!
//! **Findings.** The census reports outcomes and never severities. Whether an
//! untyped document blocks a run is a control's business, not a check's
//! ([spec 12](../../../../docs/spec/12-check-layer.md)), and the census is
//! neither. It is the denominator that a later phase computes coverage against.
//!
//! **The overlay resolver.** [`shelves::Taxonomy`] reads an already-resolved
//! taxonomy, and `headwater-resolve` is what resolves one. This crate carried
//! the stand-in for that until
//! [#50](https://github.com/headwater-ai/headwater/issues/50), and the two went
//! away together.
//!
//! **Step 4 of kind resolution.** Path-pattern refinement has no declared form
//! anywhere. See [`resolve`].

pub mod census;
pub mod derived;
pub mod resolve;
pub mod shelves;
pub mod walk;

pub use census::{Census, Detail, Row};
pub use derived::{Member, Population, Producer, Shape, Treatment};
pub use headwater_meta::pattern::{self, Pattern};
pub use resolve::{Resolution, Step};
pub use shelves::{DeclarationError, Shelf, ShelfBody, Taxonomy};
pub use walk::{Corpus, Exclusion};
