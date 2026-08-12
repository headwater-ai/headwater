// SPDX-License-Identifier: Apache-2.0
//! What an address or a reference can be rejected for.
//!
//! Every rejection carries a byte offset into the text it was parsed from.
//! The caller holds the span of the scalar, and an offset inside that scalar is
//! what lets a finding put the caret under the segment that is wrong rather
//! than under the whole line.

/// A rejection, with the offset in the parsed text where it was found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefError {
    pub kind: RefErrorKind,
    /// Byte offset into the text that was parsed. `0` for a rejection about the
    /// whole text rather than a place inside it.
    pub at: usize,
}

impl RefError {
    pub fn new(kind: RefErrorKind, at: usize) -> Self {
        Self { kind, at }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefErrorKind {
    /// The text is empty.
    Empty,
    /// Two separators in a row, or one at either end.
    EmptySegment,
    /// A character no segment admits.
    BadCharacter(char),
    /// A reference that does not open with the sigil.
    MissingSigil,
    /// A root outside the closed set.
    UnknownRoot(String),
    /// A reference that names a root and nothing inside it.
    RootAlone(String),
}

impl std::fmt::Display for RefError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

/// The message is on the kind rather than on the error, for the reason the
/// loader gives: a caller that carries this rejection inside its own error type
/// reuses this wording instead of writing a second one, and two wordings of one
/// rule drift.
impl std::fmt::Display for RefErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RefErrorKind::Empty => write!(
                f,
                "there is nothing to read; an address names at least one segment, \
                 and a reference names a root and at least one segment in it"
            ),
            RefErrorKind::EmptySegment => write!(
                f,
                "an empty segment; `.` separates two segments and never opens or closes an address"
            ),
            RefErrorKind::BadCharacter(found) => write!(
                f,
                "the character `{found}` is not allowed in a segment; \
                 a segment holds letters, digits, `_` and `-`"
            ),
            RefErrorKind::MissingSigil => write!(
                f,
                "a reference opens with `$`; without it this is an address, \
                 and an address is legal only where nothing else can be written"
            ),
            RefErrorKind::UnknownRoot(found) => write!(
                f,
                "`{found}` is not a reference root; the roots are {}",
                crate::Root::NAMES.join(" and ")
            ),
            RefErrorKind::RootAlone(root) => write!(
                f,
                "`${root}` names a root and nothing in it; a reference names what it reads"
            ),
        }
    }
}

impl std::error::Error for RefError {}
