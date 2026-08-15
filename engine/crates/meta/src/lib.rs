// SPDX-License-Identifier: Apache-2.0
//! The meta-schema of the taxonomy language, and what it decides over one
//! source.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#the-meta-schema) says that
//! the taxonomy language has a formal schema, published with the engine and
//! versioned with it. [`meta-schema.yml`](../../meta-schema.yml) is that schema
//! and this crate is its reader.
//!
//! # Expressed in its own dialect
//!
//! The meta-schema is a YAML source on the [Q2](../../../../docs/spec/09-decisions.md#q2--schema-format)
//! loader rulings, read by `headwater-yaml` exactly as a taxonomy source is.
//! That is not a stylistic choice. Q2 rules that JSON Schema is an emitted
//! export and never the validator, and the reason is visible in spec 2 itself:
//! the language carries `$`-references, and no JSON Schema keyword resolves
//! one. A second language for the schema would also be a second dialect to
//! keep in step with the first, and the two would drift at the first ruling
//! that only one of them heard about.
//!
//! # What one source can decide, and what it cannot
//!
//! Spec 2 lists what `headwater taxonomy validate` checks. Four of those items
//! are decidable over a single source and the rest read a *resolved* taxonomy:
//!
//! - **structural conformance** — the thirteen declarations, their members, the
//!   type of every scalar, and every closed value set.
//! - **reference well-formedness** — every address and every `$`-reference
//!   parses, a reference stands only where the meta-schema admits one, and a
//!   reference never stands in the position that a reference reads.
//! - **the reserved root** — `package` is a reference root, so no taxonomy may
//!   declare a block with that name.
//! - **an address never reaches into a list** — the grammar cannot refuse
//!   `…transitions.0`, because `0` is a legal key name. The meta-schema can,
//!   because it knows which positions hold lists.
//!
//! [`validate::skipped`] names the rest, each with the reason it needs a
//! resolved tree. A validator that reported a pass over rules it never ran would
//! be the silent pass that [spec 4](../../../../docs/spec/04-assurance-model.md)
//! exists to remove, so the list is data rather than a paragraph in a comment.
//! `headwater_resolve::rules` runs those rules and holds the other half of the
//! same accounting.
//!
//! The resolver is [#50](https://github.com/headwater-ai/headwater/issues/50)
//! and the `taxonomy validate` verb is
//! [#51](https://github.com/headwater-ai/headwater/issues/51). This crate is
//! what both of them call.
//!
//! # The shelf-path language lives here
//!
//! [`pattern`] is the glob language that a shelf path and a corpus exclusion are
//! written in, with the specificity order that decides which of two shelves
//! claims a path. It came from `headwater-census`, which was the first code to
//! read a shelf declaration, and it moved here when `taxonomy validate` needed
//! to ask whether two shelf patterns can collide with no corpus in hand. Spec 13
//! already said that the decision belongs to the meta-schema.
//!
//! # Example
//!
//! ```
//! use headwater_meta::MetaSchema;
//!
//! let schema = MetaSchema::shipped().expect("the shipped meta-schema");
//! let source = "taxonomy: acme\nversion: 1.0.0\nanchors:\n  code_path: {resolver: source-tree}\n";
//! let root = headwater_yaml::load(source).expect("it loads");
//! assert!(headwater_meta::validate::taxonomy(&schema, &root).is_empty());
//! ```

pub mod error;
pub mod pattern;
pub mod schema;
pub mod shape;
pub mod validate;

pub use error::{render, MetaError, MetaErrorKind, SchemaError, SchemaErrorKind};
pub use pattern::Pattern;
pub use schema::{MetaSchema, Position, SOURCE};
pub use shape::{Form, Member, ScalarType, Shape};
