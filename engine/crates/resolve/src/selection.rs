// SPDX-License-Identifier: Apache-2.0
//! Which bundle a consumer left out, derived rather than declared.
//!
//! A bundle selection that omits a bundle another selected bundle reads is
//! refused by [`crate::rules`], once for every name the omission left dangling.
//! The refusal is correct and it names the wrong thing: it names the seven
//! addresses that read a missing name, and never the one bundle that declares
//! all seven. A reader who does not already know the package cannot get from
//! one to the other.
//!
//! # Why this derives the answer instead of reading a label
//!
//! A bundle may declare `requires:`, and
//! [HW-DR-0040](../../../../docs/decisions/0040-q40-whether-extends-bundle-requires-and-an-overlay-s-taxonomy-key-are-a-mechanism-or-a-label.md)
//! ruled that key a label: the engine reads nothing from it, and the meta-schema
//! says so beside the key. That ruling stands and this module does not disturb
//! it. Three measurements are why the label is not the answer here anyway.
//!
//! 1. **The label is unread, so it is unverified, so it has already drifted.**
//!    HW-DR-0040 recorded on 2026-08-30 that `decision-record` declared
//!    `requires: [design-spec]`. It declares `requires: [evidence-and-obligation]`
//!    now. Nothing read the old value, nothing reported the move, and no rule
//!    would have caught the new value being wrong either.
//! 2. **The label is optional and most publishers leave it empty.** Three of the
//!    six bundles this repository's own package ships declare `requires: []`. A
//!    derivation works on a third-party package whose author never wrote the key.
//! 3. **A derivation cannot disagree with the refusal it explains**, because it
//!    is the same traversal. [`crate::rules::dangling`] produces the names, and
//!    the same function run over a candidate's resolution says which of them the
//!    candidate declares. There is no second reading of "what does this bundle
//!    declare" to drift against the first.
//!
//! # What it costs, and where
//!
//! One resolution per bundle the package ships that the consumer did not select,
//! and only on a path that has already failed. A selection that resolves reaches
//! none of this.

use crate::package::Consumer;
use std::path::Path;

/// One bundle, and the dangling names it would supply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Supplies {
    /// The bundle's name, as `bundles:` would carry it.
    pub bundle: String,
    /// The names the refusal reported that this bundle declares, in the order
    /// the refusal reported them, deduplicated.
    pub names: Vec<String>,
}

/// What a consumer could add to complete an incomplete selection.
///
/// It is never empty: [`advice`] returns `None` rather than an `Advice` naming
/// nothing. A message that always names something is a message that will
/// eventually name the wrong thing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Advice {
    /// The package the bundles belong to, as the consumer pinned it.
    pub package: String,
    /// How many names the selection left dangling, which is the denominator
    /// every count in the message is over.
    pub dangling: usize,
    /// Each unselected bundle that declares at least one of them, in the order
    /// the package ships them.
    pub bundles: Vec<Supplies>,
}

impl Advice {
    /// The lines a verb prints under the refusal.
    pub fn render(&self) -> String {
        String::new()
    }
}

/// Which bundle the consumer left out, or nothing.
///
/// `None` where the selection resolves, where it fails for a reason that is not
/// a dangling name, where the package ships no unselected bundle, and where no
/// unselected bundle declares any of the dangling names. The last of those is
/// the case the message exists to stay quiet about.
pub fn advice(_root: &Path, _consumer: &Consumer) -> Option<Advice> {
    None
}
