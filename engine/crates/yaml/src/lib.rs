// SPDX-License-Identifier: Apache-2.0
//! Reading taxonomy sources, on the [Q2](../../../../docs/spec/09-decisions.md#q2--schema-format)
//! dialect rules.
//!
//! YAML 1.2 is the concrete syntax, and this crate is the only place that knows
//! it. Four rulings decide what the loader accepts:
//!
//! - **YAML 1.2 core schema, so `no` stays the string `no`.** YAML 1.1 reads
//!   `no`, `yes`, `on` and `off` as booleans, which rewrites a facet value that
//!   an author wrote as a word.
//! - **Duplicate keys are an error.** A resolver that reads the last one and a
//!   reviewer who reads the first one disagree, and neither is wrong.
//! - **Anchors, aliases and merge keys are forbidden.** The `$`-reference is
//!   the sanctioned reuse mechanism, and an alias is a second one that no
//!   overlay can address.
//! - **Scalar types come from the meta-schema and never from the YAML
//!   resolver.** So [`value::Value`] has no `Null` variant and no `Bool`. A
//!   scalar keeps its text and its style, and [`core_schema`] turns that pair
//!   into a type only where a declared type asks for one.
//!
//! A fifth ruling reached Q2 from this direction rather than the other. The
//! engine derived it from the alias argument, and the decision then accepted
//! it: an explicit tag is refused, because a tag declares a type and the
//! meta-schema is the authority on type. Q2 records why it settled the rule
//! instead of deferring it — a loader can relax a rule later at no cost, and
//! cannot add one later without a finding against every source that already
//! used the form.
//!
//! # Example
//!
//! ```
//! let root = headwater_yaml::load("kind: decision\nstable: no\n").unwrap();
//! let map = root.as_map().unwrap();
//! let stable = map.get("stable").unwrap().as_scalar().unwrap();
//! assert_eq!(headwater_yaml::core_schema::as_bool(stable), None);
//! assert_eq!(stable.text, "no");
//! ```

pub mod core_schema;
pub mod error;
pub mod json;
pub mod loader;
pub mod span;
pub mod value;

pub use error::{ErrorKind, LoadError};
pub use loader::{load, load_with, Options};
pub use span::{Origin, Position, Span, Spanned};
pub use value::{Entry, Mapping, Scalar, Style, Value};
