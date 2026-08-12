// SPDX-License-Identifier: Apache-2.0
//! What the meta-schema refuses, and what refuses the meta-schema.
//!
//! Two error types, because two different people read them. A [`SchemaError`]
//! is a defect in `meta-schema.yml`, which ships with the engine, and only
//! somebody editing this crate ever sees one. A [`MetaError`] is a finding
//! about a taxonomy source that an adopter wrote, and it carries the span that
//! a caret goes under.

use headwater_yaml::Span;

/// A rejection of a taxonomy source, at a place inside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetaError {
    pub kind: MetaErrorKind,
    /// The dotted place in the source, as a reader would name it.
    pub at: String,
    pub span: Span,
}

impl MetaError {
    pub fn new(kind: MetaErrorKind, at: &str, span: Span) -> Self {
        Self {
            kind,
            at: at.to_string(),
            span,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetaErrorKind {
    /// A root key that is not one of the closed root set.
    UnknownDeclaration(String),
    /// `package` at the root of a taxonomy source. It is a reserved reference
    /// root, and spec 7's package manifest is a different file.
    ReservedRoot,
    /// An overlay address whose first segment is `package`. The publisher's
    /// library is reached with a reference and never with an address.
    ReservedRootAddress(String),
    /// An `add_to` or a `remove_from` at a node that is not a list.
    NotAList(String),
    /// A member the meta-schema requires, absent.
    MissingMember(String),
    /// A member the meta-schema does not declare.
    UnknownMember(String),
    /// A mapping where a sequence belongs, or the like.
    WrongForm {
        expected: &'static str,
        found: &'static str,
    },
    /// A scalar whose text is not of the declared type.
    NotOfType {
        declared: &'static str,
        found: String,
    },
    /// A scalar outside a closed value set.
    NotInSet { found: String, allowed: Vec<String> },
    /// A declared key that no address can reach.
    KeyIsNotASegment { key: String, why: String },
    /// A `$`-reference where the meta-schema admits a literal only.
    ReferenceNotAdmitted(String),
    /// A scalar that opens with the sigil and is not a well-formed reference.
    MalformedReference(String),
    /// No alternative of a `one_of` accepted the node.
    NoAlternative,
    /// An overlay operation the resolver does not have.
    UnknownOperation(String),
    /// An overlay address that is not a well-formed address.
    MalformedAddress(String),
    /// An overlay address that names a node the meta-schema does not declare.
    UndeclaredAddress { address: String, stopped_at: String },
    /// An overlay address that reaches a position inside a list. The grammar
    /// cannot catch this, because `0` is a legal key name.
    AddressIntoList { address: String, list: String },
    /// An overlay address that continues below a scalar.
    AddressBelowScalar { address: String, scalar: String },
}

impl std::fmt::Display for MetaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}: {}", self.span.start, self.at, self.kind)
    }
}

impl std::fmt::Display for MetaErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetaErrorKind::UnknownDeclaration(key) => write!(
                f,
                "`{key}` is not a declaration; the root set of a taxonomy source is closed, \
                 and to add a root is a meta-schema change"
            ),
            MetaErrorKind::ReservedRoot => write!(
                f,
                "`package` is a reserved reference root, and no taxonomy may declare a block \
                 with that name; a package manifest is a separate file, and a taxonomy source \
                 names itself with `taxonomy`"
            ),
            MetaErrorKind::ReservedRootAddress(address) => write!(
                f,
                "`{address}` opens on the reserved root `package`; the publisher's library of \
                 defined content is read with a reference and never named by an address"
            ),
            MetaErrorKind::NotAList(address) => write!(
                f,
                "`{address}` does not name a list, and `add_to` and `remove_from` are what a \
                 list takes; every other node takes `add`, `override` or `remove`"
            ),
            MetaErrorKind::MissingMember(name) => {
                write!(f, "`{name}` is required here and it is absent")
            }
            MetaErrorKind::UnknownMember(name) => write!(
                f,
                "`{name}` is not declared here; a declaration takes the members the \
                 meta-schema gives it and no others"
            ),
            MetaErrorKind::WrongForm { expected, found } => {
                write!(f, "{expected} belongs here, and this is {found}")
            }
            MetaErrorKind::NotOfType { declared, found } => write!(
                f,
                "`{found}` is not {declared}; a scalar takes its type from the meta-schema"
            ),
            MetaErrorKind::NotInSet { found, allowed } => write!(
                f,
                "`{found}` is outside the value set; the values are {}",
                quoted(allowed)
            ),
            MetaErrorKind::KeyIsNotASegment { key, why } => write!(
                f,
                "`{key}` cannot be addressed: {why}. Every declared key is a segment of an \
                 address, because an overlay has to be able to name it"
            ),
            MetaErrorKind::ReferenceNotAdmitted(text) => write!(
                f,
                "`{text}` is a reference, and a literal is what belongs here; a reference \
                 stands only where the meta-schema admits one, and never in the position \
                 that a reference reads"
            ),
            MetaErrorKind::MalformedReference(why) => write!(f, "{why}"),
            MetaErrorKind::NoAlternative => write!(
                f,
                "no form the meta-schema admits here accepts this; a core requirement names \
                 exactly one of a facet role, a purpose, a relation family or an \
                 identifier scheme"
            ),
            MetaErrorKind::UnknownOperation(key) => write!(
                f,
                "`{key}` is not an overlay operation; the operations are `add`, `override`, \
                 `remove`, `add_to` and `remove_from`"
            ),
            MetaErrorKind::MalformedAddress(why) => write!(f, "{why}"),
            MetaErrorKind::UndeclaredAddress {
                address,
                stopped_at,
            } => write!(
                f,
                "`{address}` names nothing the meta-schema declares; `{stopped_at}` is the \
                 segment that stops it"
            ),
            MetaErrorKind::AddressIntoList { address, list } => write!(
                f,
                "`{address}` addresses a position inside the list at `{list}`; a list position \
                 does not survive an upstream release, and `add_to` and `remove_from` are what \
                 a list takes instead"
            ),
            MetaErrorKind::AddressBelowScalar { address, scalar } => write!(
                f,
                "`{address}` continues below `{scalar}`, which holds a value rather than a tree"
            ),
        }
    }
}

impl std::error::Error for MetaError {}

/// A defect in the meta-schema that this crate ships.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaError {
    pub kind: SchemaErrorKind,
    pub at: String,
    pub span: Span,
}

impl SchemaError {
    pub fn new(kind: SchemaErrorKind, at: &str, span: Span) -> Self {
        Self {
            kind,
            at: at.to_string(),
            span,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SchemaErrorKind {
    /// The meta-schema source did not load at all.
    Unloadable(String),
    /// A top-level block the meta-schema needs, absent.
    MissingBlock(&'static str),
    ShapeNotAMapping,
    ShapeNotASequence,
    ShapeNotAScalar,
    NoForm,
    TwoForms {
        first: String,
        second: String,
    },
    UnknownForm(String),
    UnknownScalarType(String),
    /// `block:` naming a definition that is not there.
    UndefinedBlock(String),
    /// A `block:` that reaches itself with no node consumed on the way.
    CyclicBlock(String),
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "meta-schema.yml: {}: {}", self.at, self.kind)
    }
}

impl std::fmt::Display for SchemaErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaErrorKind::Unloadable(why) => write!(f, "it does not load: {why}"),
            SchemaErrorKind::MissingBlock(name) => write!(f, "there is no `{name}` block"),
            SchemaErrorKind::ShapeNotAMapping => write!(f, "a shape is a mapping"),
            SchemaErrorKind::ShapeNotASequence => write!(f, "a sequence belongs here"),
            SchemaErrorKind::ShapeNotAScalar => write!(f, "a scalar belongs here"),
            SchemaErrorKind::NoForm => write!(f, "a shape carries one form key, and this has none"),
            SchemaErrorKind::TwoForms { first, second } => write!(
                f,
                "a shape carries exactly one form key, and this carries `{first}` and `{second}`"
            ),
            SchemaErrorKind::UnknownForm(key) => write!(f, "`{key}` is not a form or a modifier"),
            SchemaErrorKind::UnknownScalarType(name) => write!(f, "`{name}` is not a scalar type"),
            SchemaErrorKind::UndefinedBlock(name) => {
                write!(f, "`{name}` is not defined under `definitions`")
            }
            SchemaErrorKind::CyclicBlock(name) => write!(
                f,
                "`{name}` reaches itself through `block:` alone, which never terminates; \
                 a recursive shape goes through a `seq`, a `map` or a `members`"
            ),
        }
    }
}

impl std::error::Error for SchemaError {}

fn quoted(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("`{value}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Every rejection, one per line, in the order they were found.
pub fn render(errors: &[MetaError]) -> String {
    use std::fmt::Write;
    errors.iter().fold(String::new(), |mut out, error| {
        let _ = writeln!(out, "{error}");
        out
    })
}
