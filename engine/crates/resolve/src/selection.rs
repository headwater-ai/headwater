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
//! 1. **The label is optional, and most publishers leave it empty.** Five of the
//!    seven bundles this repository's own package ships declare `requires: []`:
//!    `brd-prd`, `diataxis`, `diataxis-site`, `evidence-and-obligation` and
//!    `standards-spec`. Only `design-spec` and `decision-record` name anything. A derivation works
//!    on a third-party package whose author never wrote the key.
//! 2. **The label is unread, so no rule reports one that is wrong.** The two
//!    values here are correct because a person keeps them so, with the reason
//!    written at the point of each change: HW-DR-0044 moved `decision-record`
//!    from `design-spec` to `evidence-and-obligation`, and the comment above the
//!    key names the two addresses that moved. That is care rather than a
//!    mechanism, and care does not reach a package this engine did not write.
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
use crate::source::Source;
use headwater_yaml::Mapping;
use std::collections::BTreeSet;
use std::path::Path;

/// The declaration a bundle selection is written in.
///
/// One incomplete selection, two readers. A corpus writes its selection in
/// `.headwater/taxonomy.yml` and meets the refusal at `taxonomy resolve`; a
/// publisher writes one in an assembly recipe and meets it at `taxonomy
/// publish`. The names to add are the same names and the sentence that names
/// them is the same sentence. What differs is the file the reader opens and the
/// key inside it, and telling a publisher to edit a consumer declaration they do
/// not have is advice nobody can take.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selection {
    /// A corpus's own `bundles:`, read by `taxonomy resolve` and `taxonomy
    /// validate`.
    Corpus,
    /// An assembly recipe's `from.bundles:`, read by `taxonomy publish
    /// --assembly`, at the path the publisher edits.
    Recipe(String),
}

impl Selection {
    /// Who made the selection, as the message's subject.
    fn selector(&self) -> &'static str {
        match self {
            Selection::Corpus => "this repository",
            Selection::Recipe(_) => "this recipe",
        }
    }

    /// The key and the file a reader adds a bundle to.
    fn add_to(&self) -> String {
        match self {
            Selection::Corpus => format!("`bundles:` in {}", package::CONSUMER),
            Selection::Recipe(at) => format!("`from.bundles:` in {at}"),
        }
    }
}

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
    /// Where the selection is written, which decides what the last line tells
    /// the reader to edit.
    pub selection: Selection,
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
             declares. {subject} `{}` ships that {} did not select {stated} {supplied} of \
             them:\n",
            self.package,
            self.selection.selector()
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
            "Add what you need to {}\n",
            self.selection.add_to()
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
///
/// # What this does not measure, and what a reader meets because of it
///
/// A candidate is judged on the names it **supplies** out of the current
/// dangling set, and never on the names it **introduces**. So a bundle that
/// declares the one name the reader is missing, and whose own declarations read
/// a second name nothing declares, is reported as supplying 1 of the 1.
///
/// The reader adds it, runs again, and takes a fresh refusal on the new name
/// with no advice under it, because nothing in the library supplies that one.
/// **A reader is sent round twice and the second round is silent.** The counts
/// are true at each step and the sequence is still worse than one refusal.
///
/// Nothing in `headwater/standard` reaches this: `evidence-and-obligation`
/// supplies all four of the names the incomplete triple leaves dangling and
/// introduces none. A third-party library can. Reporting what a candidate
/// introduces means a second traversal per candidate and a message that carries
/// two lists, and it is not built here.
pub fn advice(root: &Path, consumer: &Consumer) -> Option<Advice> {
    let (directory, manifest) = package::located(root, &consumer.package)?;
    derive(
        root,
        &directory,
        &manifest,
        consumer,
        Selection::Corpus,
        &|trial| dangling(root, trial),
    )
}

/// The same advice for a publisher whose selection is an assembly recipe.
///
/// [#582](https://github.com/headwater-ai/headwater/issues/582) put a
/// referential-integrity refusal on the publish path, and this is what a
/// publisher who meets it reads under it. It is the recipe half of
/// [#579](https://github.com/headwater-ai/headwater/issues/579): the refusal
/// names the addresses that read a missing name, and this names the bundle that
/// declares them and the key to add it to.
///
/// It is one entry point rather than two, because the recipe is where every
/// input comes from: `assembly::read` holds the recipe to the package that
/// carries it, and `recipe.from` is already the shape of a [`Consumer`]. A
/// caller that had to build that pairing itself would be a second answer to
/// "what did this recipe select".
///
/// `None` for every reason [`advice`] returns `None`, and additionally where the
/// recipe does not read at all. A caller reaching this is already reporting a
/// refusal, so a second refusal here is noise: it prints nothing rather than
/// saying that the thing the reader is already being refused for could not be
/// read a second time.
pub fn for_recipe(root: &Path, directory: &Path, assembly: &str) -> Option<Advice> {
    let manifest = package::manifest_at(directory).ok()?;
    let recipe = crate::assembly::read(root, directory, &manifest, assembly).ok()?;
    let consumer = Consumer {
        package: recipe.from.package.clone(),
        version: recipe.from.version.clone(),
        bundles: recipe.from.bundles.clone(),
        digest: None,
        overlay: None,
        corpus_root: String::new(),
        exclusions: Vec::new(),
    };
    let at = recipe
        .at
        .strip_prefix(root)
        .unwrap_or(&recipe.at)
        .display()
        .to_string();
    derive(
        root,
        directory,
        &manifest,
        &consumer,
        Selection::Recipe(at),
        &|trial| dangling_at(root, directory, &manifest, trial),
    )
}

/// The reading both entry points share.
///
/// `dangling` is passed rather than chosen here because the two callers reach a
/// package two ways. A corpus names its package and is held to the version it
/// pinned; a recipe holds a directory it was read out of and the pin was already
/// checked when the recipe was read. One traversal, two ways in.
fn derive(
    root: &Path,
    directory: &Path,
    manifest: &Mapping,
    consumer: &Consumer,
    selection: Selection,
    dangling: &dyn Fn(&Consumer) -> Option<Vec<String>>,
) -> Option<Advice> {
    let wanted = dangling(consumer)?;
    if wanted.is_empty() {
        return None;
    }

    let contents = package::contents_of(manifest);
    let (_, shipped) = package::bundle_names(root, directory, &contents).ok()??;

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
        let Some(remaining) = dangling(&trial) else {
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
        selection,
    })
}

/// The names one selection leaves dangling, in the order the refusal reports
/// them, each one once.
///
/// `None` where the selection does not resolve at all. That is a different
/// refusal with a different remedy, and this module is about the one refusal it
/// can explain.
fn dangling(root: &Path, consumer: &Consumer) -> Option<Vec<String>> {
    names(package::sources(root, consumer).ok()?)
}

/// The same names out of a package directory the caller already holds.
///
/// [`package::sources_at`] is [`package::sources`] with the version-pin
/// comparison dropped, and dropping it is right here: a recipe's `from.package`
/// was held to the manifest it sits beside when the recipe was read, so a second
/// comparison would answer a question already answered.
fn dangling_at(
    root: &Path,
    directory: &Path,
    manifest: &Mapping,
    consumer: &Consumer,
) -> Option<Vec<String>> {
    names(package::sources_at(root, directory, manifest, consumer).ok()?)
}

fn names(sources: Vec<Source>) -> Option<Vec<String>> {
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
