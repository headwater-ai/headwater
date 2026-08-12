// SPDX-License-Identifier: Apache-2.0
//! The `$`-reference sublanguage of the taxonomy language.
//!
//! [Spec 2](../../../docs/spec/02-taxonomy-model.md#the--reference-sublanguage)
//! is the definition, and this crate is the definition made executable. Both
//! close the obligation that
//! [13 — Open obligations](../../../docs/spec/13-open-obligations.md) carried:
//! three uses, written three ways, and no grammar.
//!
//! # The grammar
//!
//! ```abnf
//! address   = segment *( "." segment )
//! reference = "$" root "." address
//! root      = "vocabularies" / "package"
//! segment   = 1*( ALPHA / DIGIT / "_" )
//! ```
//!
//! # Three uses, two productions, one path
//!
//! The three uses that Q2 counted are a vocabulary reference
//! (`$vocabularies.lifecycle_state`), a package reference
//! (`$package.optional.forbids`), and an overlay address
//! (`shelves.decisions.path`). They are not three grammars. They are one path
//! production read from two places, and the difference between the two is
//! where each one is written:
//!
//! - An **address** is written where nothing but an address is legal: the key
//!   under `add:`, `override:` and `add_to:`, and each entry of the `remove:`
//!   list. It wears no sigil, because there is nothing for a sigil to tell it
//!   apart from.
//! - A **reference** is written in value position, where a literal value is
//!   equally legal. `values:` on a facet takes a list or a reference to one.
//!   The sigil is what separates them.
//!
//! So the rule for the sigil is not a convention. A sigil marks a reference in
//! every position where a literal is also admissible, and nowhere else.
//!
//! # What this crate does not do
//!
//! It does not resolve. [`Reference::parse`] says that a text is a well-formed
//! reference, and says nothing about whether the node it names exists. That is
//! the resolver's, and it runs late — after every overlay has been applied,
//! over the merged tree. Spec 2's own worked overlay depends on the order: an
//! overlay that overrides `vocabularies.lifecycle_state` expects the facet that
//! reads `$vocabularies.lifecycle_state` to follow it, and an early resolution
//! would have frozen the base list before the overlay was read.
//!
//! # Example
//!
//! ```
//! use headwater_ref::{classify, Address, Root, Scalar};
//!
//! let Ok(Scalar::Reference(reference)) = classify("$vocabularies.lifecycle_state") else {
//!     panic!("a reference");
//! };
//! assert_eq!(reference.root(), Root::Vocabularies);
//! assert_eq!(reference.address().to_string(), "lifecycle_state");
//!
//! // Two `add` operations reaching into one kind are still disjoint.
//! let identifier = Address::parse("kinds.design_spec.identifier").unwrap();
//! let language = Address::parse("kinds.design_spec.language").unwrap();
//! assert!(identifier.is_disjoint_from(&language));
//! ```

pub mod address;
pub mod error;
pub mod reference;

pub use address::Address;
pub use error::{RefError, RefErrorKind};
pub use reference::{classify, Reference, Root, Scalar};
