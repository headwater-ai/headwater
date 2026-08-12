// SPDX-License-Identifier: Apache-2.0
//! The path production, and the address that is a bare one.

use crate::error::{RefError, RefErrorKind};

/// A dotted path into a taxonomy tree.
///
/// An address is what an overlay operation names: the key under `add:` and
/// `override:`, and each entry of the `remove:` list. It is parsed into
/// segments rather than kept as text, because every question the resolver asks
/// of two addresses is a question about their segments. `kinds.playbook` is a
/// prefix of `kinds.playbook_step` as text and not as an address, and a
/// confluence check that compared text would call two disjoint `add`
/// operations a conflict.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address {
    segments: Vec<String>,
}

impl Address {
    /// Parse a bare dotted path. `at` shifts the reported offsets, so a caller
    /// that already consumed a sigil and a root reports into the original text.
    pub(crate) fn parse_at(text: &str, at: usize) -> Result<Self, RefError> {
        if text.is_empty() {
            return Err(RefError::new(RefErrorKind::Empty, at));
        }
        let mut segments = Vec::new();
        let mut start = 0;
        for (offset, found) in text.char_indices() {
            match found {
                '.' => {
                    if offset == start {
                        return Err(RefError::new(RefErrorKind::EmptySegment, at + offset));
                    }
                    segments.push(text[start..offset].to_string());
                    start = offset + 1;
                }
                _ if is_segment_character(found) => {}
                _ => {
                    return Err(RefError::new(
                        RefErrorKind::BadCharacter(found),
                        at + offset,
                    ));
                }
            }
        }
        if start == text.len() {
            return Err(RefError::new(RefErrorKind::EmptySegment, at + text.len()));
        }
        segments.push(text[start..].to_string());
        Ok(Self { segments })
    }

    /// Parse an address.
    pub fn parse(text: &str) -> Result<Self, RefError> {
        Self::parse_at(text, 0)
    }

    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    /// The first segment, which is the declaration the address lands in. An
    /// address always has one, because a zero-segment address does not parse.
    pub fn block(&self) -> &str {
        &self.segments[0]
    }

    /// Whether this address names the same node as `other`, or one above it.
    ///
    /// `kinds.playbook` is a prefix of `kinds.playbook.purpose`. `kinds.play`
    /// is a prefix of neither.
    pub fn is_prefix_of(&self, other: &Address) -> bool {
        self.segments.len() <= other.segments.len()
            && self
                .segments
                .iter()
                .zip(&other.segments)
                .all(|(a, b)| a == b)
    }

    /// Whether two addresses name subtrees that do not overlap.
    ///
    /// This is the predicate the confluence check runs. Two `add` operations at
    /// disjoint addresses commute, so a resolver that can decide this can prove
    /// that every subset of an add-only bundle set resolves
    /// ([spec 7](../../../docs/spec/07-distribution-and-federation.md)).
    pub fn is_disjoint_from(&self, other: &Address) -> bool {
        !self.is_prefix_of(other) && !other.is_prefix_of(self)
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.segments.join("."))
    }
}

/// A segment holds letters, digits and `_`.
///
/// `.` is excluded because it is the separator, and that exclusion is
/// permanent: a key with a dot in it would make an address ambiguous, so the
/// meta-schema refuses to declare one. Every other exclusion is provisional,
/// `-` among them. [Q2](../../../docs/spec/09-decisions.md#q2--schema-format)
/// settles the direction to guess in — a rule can be relaxed later at no cost,
/// and cannot be added later without a finding against every source that
/// already used the form.
fn is_segment_character(found: char) -> bool {
    found.is_ascii_alphanumeric() || found == '_'
}
