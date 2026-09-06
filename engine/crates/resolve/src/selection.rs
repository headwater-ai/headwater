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

use crate::package::{self, Consumer};
use std::collections::BTreeSet;
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
    /// How many of them any of these bundles declares, which is never more than
    /// [`Advice::dangling`] and is often less. The message says both numbers,
    /// because "add this" over a bundle that supplies four of seven is the
    /// wrong-bundle failure in slow motion.
    pub supplied: usize,
    /// Each unselected bundle that declares at least one of them, in the order
    /// the package ships them.
    pub bundles: Vec<Supplies>,
}

impl Advice {
    /// The lines a verb prints under the refusal.
    ///
    /// Every bundle is named with what it supplies, and no bundle is presented
    /// as sufficient unless its own count says so. A reader who cannot see
    /// which names are still unaccounted for after adding one bundle is a
    /// reader who runs the same refusal twice.
    pub fn render(&self) -> String {
        let total = self.dangling;
        let supplied = self.supplied;
        // The findings and the names are two counts over two populations: seven
        // addresses can read four names. A message that says "4 of the 4 names
        // above" over seven printed lines asks the reader to reconcile them, so
        // this states the second population before it counts anything.
        let named = if total == 1 {
            "one name".to_string()
        } else {
            format!("{total} names")
        };
        let (subject, stated) = if self.bundles.len() == 1 {
            ("A bundle", "declares")
        } else {
            ("Bundles", "declare")
        };
        let mut out = format!(
            "this bundle selection is incomplete. The findings above read {named} that nothing \
             declares. {subject} `{}` ships that this repository did not select {stated} \
             {supplied} of them:\n",
            self.package
        );
        for supplies in &self.bundles {
            let names: Vec<String> = supplies
                .names
                .iter()
                .map(|name| format!("`{name}`"))
                .collect();
            out.push_str(&format!(
                "  `{}` declares {} of the {total}: {}\n",
                supplies.bundle,
                supplies.names.len(),
                names.join(", ")
            ));
        }
        out.push_str(&format!(
            "Add what you need to `bundles:` in {}\n",
            package::CONSUMER
        ));
        out
    }
}

/// Which bundle the consumer left out, or nothing.
///
/// `None` where the selection resolves, where it fails for a reason that is not
/// a dangling name, where the package ships no unselected bundle, and where no
/// unselected bundle declares any of the dangling names. The last of those is
/// the case the message exists to stay quiet about.
pub fn advice(root: &Path, consumer: &Consumer) -> Option<Advice> {
    let wanted = dangling(root, consumer)?;
    if wanted.is_empty() {
        return None;
    }

    let (directory, manifest) = package::located(root, &consumer.package)?;
    let contents = package::contents_of(&manifest);
    let (_, shipped) = package::bundle_names(root, &directory, &contents).ok()??;

    let mut bundles: Vec<Supplies> = Vec::new();
    for candidate in shipped {
        // A directory name that is not text is a bundle no consumer can write
        // into `bundles:`, so naming it would be advice nobody can take.
        let Some(candidate) = candidate.to_str() else {
            continue;
        };
        if consumer.bundles.iter().any(|held| held == candidate) {
            continue;
        }

        // The source list the consumer would have had: the package, the bundles
        // it already selected, this candidate, then its overlay. The candidate
        // sits where `package::selected` would have put it, because a candidate
        // resolved after the overlay answers a question no consumer can ask.
        let mut trial = consumer.clone();
        trial.bundles.push(candidate.to_string());

        // A candidate that collides with the base, or with a bundle already
        // selected, is not advice. Its refusal is about the candidate and the
        // reader asked about their own selection, so it is dropped here rather
        // than carried up in place of the refusal they are reading.
        let Some(remaining) = dangling(root, &trial) else {
            continue;
        };
        let remaining: BTreeSet<String> = remaining.into_iter().collect();
        let supplies: Vec<String> = wanted
            .iter()
            .filter(|name| !remaining.contains(name.as_str()))
            .cloned()
            .collect();
        if !supplies.is_empty() {
            bundles.push(Supplies {
                bundle: candidate.to_string(),
                names: supplies,
            });
        }
    }

    if bundles.is_empty() {
        return None;
    }
    let supplied: BTreeSet<&str> = bundles
        .iter()
        .flat_map(|one| one.names.iter().map(String::as_str))
        .collect();
    Some(Advice {
        package: consumer.package.clone(),
        dangling: wanted.len(),
        supplied: supplied.len(),
        bundles,
    })
}

/// The names one selection leaves dangling, in the order the refusal reports
/// them, each one once.
///
/// `None` where the selection does not resolve at all. That is a different
/// refusal with a different remedy, and this module is about the one refusal it
/// can explain.
fn dangling(root: &Path, consumer: &Consumer) -> Option<Vec<String>> {
    let sources = package::sources(root, consumer).ok()?;
    let resolution = crate::resolve(&sources).ok()?;
    let mut seen: BTreeSet<String> = BTreeSet::new();
    Some(
        crate::rules::dangling(&resolution.taxonomy)
            .into_iter()
            .map(|found| found.name)
            .filter(|name| seen.insert(name.clone()))
            .collect(),
    )
}
