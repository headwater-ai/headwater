// SPDX-License-Identifier: Apache-2.0
//! The sigil, the closed root set, and what a scalar in a reference-admitting
//! position is.

use crate::address::Address;
use crate::error::{RefError, RefErrorKind};

/// The set of things a reference may read from, and it is closed.
///
/// A root is not a declaration name. `vocabularies` happens to be one, and
/// `package` is a reserved word that names content the resolved taxonomy does
/// not contain. To add a root is a meta-schema change, exactly as adding a
/// facet role is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Root {
    /// The `vocabularies` declaration of the taxonomy under resolution.
    Vocabularies,
    /// The publisher package's library of defined but unenabled declarations.
    Package,
}

impl Root {
    pub const NAMES: [&'static str; 2] = ["`vocabularies`", "`package`"];

    fn parse(text: &str) -> Option<Self> {
        match text {
            "vocabularies" => Some(Root::Vocabularies),
            "package" => Some(Root::Package),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Root::Vocabularies => "vocabularies",
            Root::Package => "package",
        }
    }
}

/// A `$`-reference: a root, and an address inside it.
///
/// The sigil is what tells a reference from a literal, and it is needed in
/// exactly the positions where both are legal. `values:` on a facet takes a
/// list of values or a reference to one, so the two have to be distinguishable.
/// The key under `add:` takes an address and nothing else, so an address wears
/// no sigil.
///
/// Quoting is not an escape. [Q2](../../../docs/spec/09-decisions.md#q2--schema-format)
/// rules that a scalar takes its type from the meta-schema and never from the
/// YAML resolver, so `"$vocabularies.audience"` and `$vocabularies.audience`
/// are one value. [`classify`] carries the escape that quoting cannot.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Reference {
    root: Root,
    address: Address,
}

impl Reference {
    pub fn parse(text: &str) -> Result<Self, RefError> {
        if text.is_empty() {
            return Err(RefError::new(RefErrorKind::Empty, 0));
        }
        let Some(rest) = text.strip_prefix('$') else {
            return Err(RefError::new(RefErrorKind::MissingSigil, 0));
        };
        if rest.is_empty() {
            return Err(RefError::new(RefErrorKind::Empty, 1));
        }
        let (root_text, after) = match rest.find('.') {
            Some(dot) => (&rest[..dot], Some(&rest[dot + 1..])),
            None => (rest, None),
        };
        if root_text.is_empty() {
            return Err(RefError::new(RefErrorKind::EmptySegment, 1));
        }
        let Some(root) = Root::parse(root_text) else {
            return Err(RefError::new(
                RefErrorKind::UnknownRoot(root_text.to_string()),
                1,
            ));
        };
        let Some(after) = after else {
            return Err(RefError::new(
                RefErrorKind::RootAlone(root_text.to_string()),
                1,
            ));
        };
        let at = 1 + root_text.len() + 1;
        // A trailing separator is an empty segment and not an empty address.
        // The two read the same to a parser and not to an author, and the one
        // that names the character they typed is the one that helps.
        if after.is_empty() {
            return Err(RefError::new(RefErrorKind::EmptySegment, at));
        }
        let address = Address::parse_at(after, at)?;
        Ok(Self { root, address })
    }

    pub fn root(&self) -> Root {
        self.root
    }

    /// The path under the root. It is an [`Address`] because it is the same
    /// production: one path grammar, read from two places.
    pub fn address(&self) -> &Address {
        &self.address
    }
}

impl std::fmt::Display for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}.{}", self.root.name(), self.address)
    }
}

/// What a scalar is, in a position the meta-schema says admits a reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Scalar<'a> {
    Reference(Reference),
    Literal(&'a str),
}

/// Read a scalar in a reference-admitting position.
///
/// A leading `$` opens a reference. A leading `$$` is the escape, and it stands
/// for one literal `$`. The escape exists because quoting cannot serve as one:
/// a quoted scalar and a plain one are the same value under Q2, so a corpus
/// whose value genuinely starts with a dollar sign would otherwise have no way
/// to say so.
///
/// In every other position `$` is an ordinary character, and the meta-schema is
/// what says which positions those are. An identifier scheme `pattern` is one:
/// it is typed as a string, `"^DR-[A-Z]{2,6}-[0-9]{4}$"` is a pattern rather
/// than a reference, and nothing calls this function on it.
pub fn classify(text: &str) -> Result<Scalar<'_>, RefError> {
    if text.starts_with("$$") {
        // One `$` is removed, and the rest is literal. `$$$x` is `$$x`.
        return Ok(Scalar::Literal(&text[1..]));
    }
    if text.starts_with('$') {
        return Ok(Scalar::Reference(Reference::parse(text)?));
    }
    Ok(Scalar::Literal(text))
}
