// SPDX-License-Identifier: Apache-2.0
//! The identifier claim store, and the two rules that hold it to the corpus.
//!
//! [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#identifiers):
//! "Allocation reads a tree, and a tree holds no history… So the highest value
//! that a run finds is a lower bound on every value ever allocated." A tree
//! also holds no concurrency. Every branch cut from one `main` reads the same
//! highest value, so *n* branches minting at once all mint *n+1*, each branch
//! is internally consistent, the file names differ, and git merges both without
//! a word. [`crate::duplicate`] then reports the pair on `main`, after the
//! second merge, when no gate is looking.
//!
//! # What the store is, and what it is not
//!
//! One file per identifier, at `.headwater/ids/<scheme>/<identifier>`, holding
//! the path of the document that minted it. It is written once and never
//! modified.
//!
//! **The store does not detect the collision. It forces the two branches to
//! meet.** Two branches that claim one value add one path with different bytes,
//! which is git's `add/add` conflict and nothing subtler. Resolving it means
//! merging `main` into the branch, and the branch's tree then holds both
//! claimants, so `identifier.claimed_twice` fires there — on the branch, before
//! the merge, where the remedy is a rename rather than a renumber of a
//! published identifier. The file is what makes an existing rule's read set
//! reachable in time, and neither half is sufficient alone.
//!
//! # Why the claimant path inside the file is the mechanism
//!
//! Git compares blobs before it selects a merge strategy, so two identical
//! blobs are one change and no strategy runs. Measured: two branches adding the
//! same claim path with **zero bytes** merge at exit 0 with no conflict. The
//! claimant path is therefore not documentation. It is the only thing that
//! makes the two sides differ, and so the only thing that makes the conflict
//! fire. A zero-byte claim store is the watermark form's silent failure at file
//! granularity. `tools/repo/id-store-fixtures.sh` pins both outcomes, and its
//! header carries the same argument.
//!
//! # Why a rule has to name the store
//!
//! Nothing under `.headwater/` is read by a rule otherwise. The corpus root is
//! `docs`, so a document-scoped instance never reaches a store beside it, and
//! `.headwater/capture-cost.jsonl` is the precedent for what that costs: a
//! renumber left a reading naming a document that no longer exists and
//! `headwater capture` counted the same total before and after, because nothing
//! reads it. So the store is an input of these two instances, with a digest,
//! and [`crate::cache`] keys on it like any other.
//!
//! # The two rules, and the direction that is deliberately not one
//!
//! `identifier.claim.missing` is an error and it carries a patch: an identifier
//! the corpus spends on a shelf the store covers with no file claiming it.
//! [`takes_a_claim`] is the predicate that decides which those are. The
//! correction is one file whose content is the document's own path, with no
//! judgment in it, which is the [fixability](../../../../docs/spec/12-check-layer.md#fixability)
//! bar.
//!
//! `identifier.claim.stale` is advisory: a claim that does not name the
//! document holding its identifier. Choosing whether the store or the document
//! moved is a rewrite, so no patch rides with it.
//!
//! **A claim naming an identifier the corpus no longer holds is correct, and no
//! rule reports it.** Spec 3: "**Never reused.** … A deleted document does not
//! free its number." The store is the first artifact of this repository that
//! can hold that fact at all, and a rule reporting it would report every
//! legitimate deletion as a defect.
//!
//! **A claim naming a path no document stands at, whose identifier one
//! document holds at another path, is stale.** The holder is a typed document
//! whose kind mints under the claim's scheme. An untyped file or a document of
//! another scheme that carries the same string counts as no holder. A rename
//! produces this state, and so does a claim edited by hand, and the rule
//! cannot tell the two apart, so its message names neither as the cause.
//! Never-reuse does not cover either, because neither reuses anything. The
//! finding names the current path. It carries no patch: both writers of the
//! store create and never overwrite, and [`Patch::Create`] refuses an occupied
//! path. Where two documents hold the identifier, `identifier.claimed_twice`
//! reports the pair and this rule reports nothing.

use crate::finding::{at, Finding, Severity};
use crate::instance::Outcome;
use crate::patch::Patch;
use crate::scope::{CorpusCheck, CorpusView};
use crate::shape::{IdentifierScheme, Shape};
use headwater_census::shelves::{Shelf, Taxonomy};
use headwater_graph::index::{Defect, Index};
use std::path::Path;

/// The store, relative to the repository root. Not the corpus root: a corpus
/// root is where documents live, and no claim is a document.
pub const STORE: &str = ".headwater/ids";

/// The allocation a scheme declares that the store covers whatever its shelf
/// says. See [`takes_a_claim`], which is the predicate and the reason.
pub const RECONCILE_FIRST: &str = "reconcile-first";

/// Whether an identifier minted onto this shelf under this scheme takes a claim
/// file.
///
/// **The store has a job wherever the file name does not determine the
/// identifier**, because that is exactly the case where two branches minting
/// one value land at two paths and git merges both without a word. Two
/// declarations put a corpus in that state, and the predicate is their union.
///
/// **A shelf that declares a `layout`.** The name is written from a template,
/// so two documents holding one identifier can differ in a segment the
/// identifier does not carry. `spec_series` of this repository declares
/// `{sequence:02d}-{slug}.md`, and two branches that both mint
/// `HW-SPEC-the-replay-contract` at sequences 17 and 18 write two files, both
/// merge clean, and `identifier.claimed_twice` reports the pair on `main` after
/// the second merge, where no gate is looking.
///
/// **A scheme that allocates `reconcile-first`.** The value is a sequence read
/// off the tree, so every branch cut from one `main` mints the same one, and a
/// pattern that carries no slug puts none of it in the file name. The fixture
/// taxonomy of `headwater_scaffold` declares that shape — `decisions` names its
/// files from the slug alone and `decision_id` patterns `DR-{namespace}-{seq}`
/// — so the layout half alone would take coverage away from a corpus that has
/// it.
///
/// [HW-DR-0054](../../../../docs/decisions/0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md)
/// ruled the second half and declined the first.
/// [HW-DR-0057](../../../../docs/decisions/0057-a-shelf-layout-is-the-second-half-of-what-the-identifier-claim-store-covers.md)
/// adds the first, which is a widening and takes nothing away.
///
/// This is the one reader of that predicate. `headwater_scaffold` asks it once,
/// at plan time, and carries the answer on the plan, so the writer of a claim
/// and the rule that reports a missing one can not disagree about what the
/// store covers.
pub fn takes_a_claim(shelf: &Shelf, scheme: &IdentifierScheme) -> bool {
    shelf.layout.is_some() || scheme.allocation.as_deref() == Some(RECONCILE_FIRST)
}

pub const MISSING: &str = "identifier.claim.missing";
pub const STALE: &str = "identifier.claim.stale";

/// As [`crate::duplicate`]: a run with no phase-A report has nothing to hold
/// the duplicate guard against, and it says so rather than passing.
const NO_REPORT: &str = "the view carries no phase-A report for this corpus";

/// As above, for a run whose scope did not admit the store.
const NO_STORE: &str = "the view carries no claim store for this corpus";

/// One claim: the scheme it was minted under, the identifier, and the document
/// that minted it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Claim {
    pub scheme: String,
    pub id: String,
    /// The claimant, as the file holds it: one path relative to the repository
    /// root. Empty where the file holds no line at all, which is the state
    /// [`STALE`] reports and no writer here can produce.
    pub claimant: String,
    /// The entry is a named pipe, a socket or a device, which the store never
    /// opens, so its claimant is empty. [`STALE`] says what it is rather than
    /// asking the reader to write into it, because a write to a pipe blocks
    /// (#1366).
    pub special: bool,
}

/// Every claim the store holds, in `(scheme, identifier)` order.
///
/// The order is the canonical one, so [`Claims::digest`] is a function of the
/// store's content and never of a directory walk's order.
#[derive(Clone, Debug, Default)]
pub struct Claims {
    entries: Vec<Claim>,
}

impl Claims {
    /// No store. A corpus that has never minted one reads this way, and so does
    /// a run whose caller did not offer one.
    pub fn empty() -> Self {
        Claims::default()
    }

    /// The store of a repository.
    ///
    /// A directory that is absent or unreadable reads as an empty store, on
    /// [`crate::cache::Cache::at`]'s terms: the cost is a run that reports every
    /// identifier unclaimed, and the alternative is a verb that refuses to
    /// check a corpus because of a directory that holds no corpus content.
    pub fn at(root: &Path) -> Self {
        let mut entries = Vec::new();
        let Ok(schemes) = std::fs::read_dir(root.join(STORE)) else {
            return Claims { entries };
        };
        for scheme in schemes.flatten() {
            if !scheme.file_type().is_ok_and(|kind| kind.is_dir()) {
                continue;
            }
            let Some(scheme_name) = scheme.file_name().to_str().map(str::to_string) else {
                continue;
            };
            let Ok(claims) = std::fs::read_dir(scheme.path()) else {
                continue;
            };
            for claim in claims.flatten() {
                // A directory and nothing else is refused. `is_file` would
                // also refuse a symbolic link, and a claim reached through one
                // is a claim, because the reader is `read_to_string`.
                if claim.file_type().is_ok_and(|kind| kind.is_dir()) {
                    continue;
                }
                let Some(id) = claim.file_name().to_str().map(str::to_string) else {
                    continue;
                };
                // Only a regular file is opened. `metadata` follows a link, so
                // a claim reached through one is still read. A named pipe, a
                // socket or a device is never opened, because a pipe with no
                // writer blocks its reader for ever (#1366). Such an entry is
                // recorded as special, and an entry that cannot be read at
                // all reads as a claim that names nobody, so `claim.stale`
                // reports either at its own path.
                let (claimant, special) = match entry_of(&claim.path()) {
                    Entry::Regular => (
                        std::fs::read_to_string(claim.path())
                            .unwrap_or_default()
                            .lines()
                            .next()
                            .unwrap_or_default()
                            .trim()
                            .to_string(),
                        false,
                    ),
                    Entry::Special => (String::new(), true),
                    Entry::Unreadable => (String::new(), false),
                };
                entries.push(Claim {
                    scheme: scheme_name.clone(),
                    id,
                    claimant,
                    special,
                });
            }
        }
        entries.sort_by(|a, b| (&a.scheme, &a.id).cmp(&(&b.scheme, &b.id)));
        Claims { entries }
    }
}

/// What [`Claims::at`] found at one entry of the store.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Entry {
    /// A regular file, reached directly or through a link. The only entry
    /// that is opened.
    Regular,
    /// A named pipe, a socket or a device, reached directly or through a link.
    /// It is never opened, because a pipe with no writer and a device such as
    /// `/dev/zero` never end a read (#1366).
    Special,
    /// Anything else: a dangling link, an entry `metadata` cannot read, or a
    /// link to a directory. It is not opened and names nobody.
    Unreadable,
}

/// One decision per entry of the store, taken on the file type `metadata`
/// reports after it follows a link. What is neither a file nor a directory
/// once the link is followed is a named pipe, a socket or a device.
fn entry_of(path: &Path) -> Entry {
    let Ok(meta) = std::fs::metadata(path) else {
        return Entry::Unreadable;
    };
    match (meta.is_file(), meta.is_dir()) {
        (true, _) => Entry::Regular,
        (false, true) => Entry::Unreadable,
        (false, false) => Entry::Special,
    }
}

impl Claims {
    /// A store built from claims, for a caller that has them already. The
    /// order is imposed here rather than assumed of the caller.
    pub fn of(mut entries: Vec<Claim>) -> Self {
        entries.sort_by(|a, b| (&a.scheme, &a.id).cmp(&(&b.scheme, &b.id)));
        Claims { entries }
    }

    pub fn entries(&self) -> &[Claim] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The document claiming one identifier, and nothing where no file does.
    pub fn claimant(&self, scheme: &str, id: &str) -> Option<&str> {
        self.entries
            .binary_search_by(|claim| (claim.scheme.as_str(), claim.id.as_str()).cmp(&(scheme, id)))
            .ok()
            .map(|index| self.entries[index].claimant.as_str())
    }

    /// Every identifier one scheme has claimed. The allocator reads this and
    /// takes the maximum of it and of the corpus, which is the whole of what
    /// the store is for.
    pub fn ids_of<'a>(&'a self, scheme: &'a str) -> impl Iterator<Item = &'a str> {
        self.entries
            .iter()
            .filter(move |claim| claim.scheme == scheme)
            .map(|claim| claim.id.as_str())
    }

    /// The store as one string, in canonical order, length-prefixed so that no
    /// claimant can write the separator of the next entry. A special entry is
    /// written as `-` in place of the length, which no regular entry writes,
    /// so the digest tells a named pipe from an empty file and a store of
    /// regular files digests as it did before the distinction (#1366).
    pub fn render(&self) -> String {
        let mut out = String::new();
        for claim in &self.entries {
            if claim.special {
                out.push_str(&format!("{}/{} -\n", claim.scheme, claim.id));
                continue;
            }
            out.push_str(&format!(
                "{}/{} {} {}\n",
                claim.scheme,
                claim.id,
                claim.claimant.len(),
                claim.claimant
            ));
        }
        out
    }

    /// The digest that puts the store in a cache key and in a read set.
    ///
    /// An input with no digest un-keys its whole instance
    /// ([`crate::cache::Cache::key`]), so a store named in a read set and not
    /// hashed would leave both rules permanently re-evaluated.
    pub fn digest(&self) -> String {
        headwater_hash::digest(self.render().as_bytes())
    }
}

/// Where one claim lives, relative to the repository root.
pub fn path_of(scheme: &str, id: &str) -> String {
    format!("{STORE}/{scheme}/{id}")
}

/// What one claim file holds. One line, and never zero bytes: see the module
/// comment for why the emptiness is the whole failure mode.
pub fn contents_for(claimant: &str) -> String {
    format!("{claimant}\n")
}

/// Every identifier the phase-A report says two documents claim.
///
/// Both rules skip such an identifier. One path cannot hold two claimants, so
/// a claim written for a contended identifier would silently pick a winner.
fn contended(identity: &[headwater_graph::index::Reported]) -> Vec<&str> {
    let mut out: Vec<&str> = identity
        .iter()
        .filter_map(|reported| match &reported.defect {
            Defect::Duplicate { id, .. } => Some(id.as_str()),
            _ => None,
        })
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// An identifier the corpus spends on a shelf the store covers that no file of
/// the store claims.
pub struct Missing<'a> {
    index: &'a Index,
    /// Each kind whose shelf takes claims, and the name of the scheme it mints
    /// under. Computed once from the taxonomy, as [`crate::identifier`] does.
    schemes: Vec<(String, String)>,
}

impl<'a> Missing<'a> {
    pub fn over(shape: &Shape, taxonomy: &Taxonomy, index: &'a Index) -> Self {
        let schemes = claiming(shape, taxonomy);
        Missing { index, schemes }
    }

    fn scheme_of(&self, kind: &str) -> Option<&str> {
        self.schemes
            .iter()
            .find(|(named, _)| named == kind)
            .map(|(_, scheme)| scheme.as_str())
    }
}

/// Each kind [`takes_a_claim`] admits, with the name of the scheme it mints
/// under.
///
/// A kind on several shelves takes a claim where any one of them admits it: the
/// store covers an identifier the moment one placement of it can be renamed
/// away from a collision. A kind on no shelf at all still reaches the
/// allocation half, because a scheme that reconciles over a tree collides
/// wherever its documents stand.
fn claiming(shape: &Shape, taxonomy: &Taxonomy) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = shape
        .kinds
        .iter()
        .filter_map(|kind| {
            let scheme = shape.identifier_scheme_of(&kind.name)?;
            let shelves: Vec<&Shelf> = taxonomy
                .shelves
                .iter()
                .filter(|shelf| shelf.carries(&kind.name))
                .collect();
            let claims = match shelves.is_empty() {
                true => scheme.allocation.as_deref() == Some(RECONCILE_FIRST),
                false => shelves.iter().any(|shelf| takes_a_claim(shelf, scheme)),
            };
            claims.then(|| (kind.name.clone(), scheme.name.clone()))
        })
        .collect();
    out.sort();
    out
}

impl CorpusCheck for Missing<'_> {
    const RULE: &'static str = self::MISSING;
    /// The second edition. Edition 1 read the `reconcile-first` schemes alone.
    /// Edition 2 reads [`takes_a_claim`], which adds every scheme on a shelf
    /// that declares a `layout`.
    ///
    /// **A widened read set with an unbumped edition is invisible to a warm
    /// cache.** [`crate::scope`] states that this number keys a corpus-scoped
    /// entry, and the key carries no digest of the binary. `.headwater/cache/`
    /// is not committed, so an adopter who upgrades the engine over an
    /// unchanged corpus keeps every entry edition 1 wrote. Measured on a clone
    /// with the two new scheme directories removed: the old engine warms the
    /// cache and reports 0 `identifier.claim.missing`, the new engine reads
    /// that cache and reports 0, and the same new engine with `--no-cache`
    /// reports 16 and fails a strict run. Every test of this workspace ran one
    /// binary, so none of them could report that difference. `tests/editions.rs`
    /// now does, for this rule and every rule it ledgers: a verdict that moves
    /// at an unchanged edition fails there.
    const VERSION: u32 = 2;
    /// The duplicate guard reads the build's own report, on
    /// [`crate::duplicate`]'s terms, rather than comparing identifiers a second
    /// time.
    const NEEDS_PHASE_A: bool = true;
    const NEEDS_CLAIMS: bool = true;

    fn evaluate(&self, view: &CorpusView<'_>) -> Outcome {
        let Some(identity) = view.identity() else {
            return Outcome::Skipped(NO_REPORT.to_string());
        };
        let Some(claims) = view.claims() else {
            return Outcome::Skipped(NO_STORE.to_string());
        };
        let contended = contended(identity);
        let mut findings = Vec::new();
        for node in &self.index.typed {
            let Some(kind) = node.kind.as_deref() else {
                continue;
            };
            let Some(scheme) = self.scheme_of(kind) else {
                continue;
            };
            if contended.binary_search(&node.id.as_str()).is_ok() {
                continue;
            }
            if claims.claimant(scheme, &node.id).is_some() {
                continue;
            }
            let claim = path_of(scheme, &node.id);
            let (line, column) = at(node.id_span);
            findings.push(Finding {
                rule: self::MISSING,
                severity: Severity::Error,
                obligation: None,
                path: node.path.clone(),
                line,
                column,
                message: format!(
                    "`{}` is spent by {} and no file of `{STORE}` claims it, so this identifier \
                     is invisible to the allocator of every other branch and a second document \
                     can be minted onto it",
                    node.id, node.path
                ),
                remediation: format!(
                    "run `headwater check --fix`, which writes `{claim}` holding `{}`",
                    node.path
                ),
                patch: Some(Patch::Create {
                    path: claim,
                    contents: contents_for(&node.path),
                }),
            });
        }
        Outcome::failed(findings)
    }
}

/// A claim that does not name the document holding its identifier.
pub struct Stale<'a> {
    index: &'a Index,
    /// The front-matter key an identifier is read from, as
    /// [`crate::duplicate`] takes it, so that a remedy names the key this
    /// engine actually read.
    facet: String,
    /// Each kind that declares an identifier scheme, and the name of that
    /// scheme. A document holds a claim's identifier only where its kind mints
    /// under the claim's scheme, so an untyped file or a document of another
    /// scheme that carries the same string is no holder.
    schemes: Vec<(String, String)>,
}

impl<'a> Stale<'a> {
    pub fn over(facet: &str, shape: &Shape, index: &'a Index) -> Self {
        let schemes = shape
            .kinds
            .iter()
            .filter_map(|kind| {
                let scheme = shape.identifier_scheme_of(&kind.name)?;
                Some((kind.name.clone(), scheme.name.clone()))
            })
            .collect();
        Stale::with_schemes(facet, schemes, index)
    }

    fn with_schemes(facet: &str, mut schemes: Vec<(String, String)>, index: &'a Index) -> Self {
        schemes.sort();
        Stale {
            index,
            facet: facet.to_string(),
            schemes,
        }
    }

    /// Whether a typed document of `kind` mints under `scheme`.
    fn mints_under(&self, kind: &str, scheme: &str) -> bool {
        self.schemes
            .iter()
            .any(|(named, minted)| named == kind && minted == scheme)
    }
}

impl CorpusCheck for Stale<'_> {
    const RULE: &'static str = self::STALE;
    /// The second edition: a claim naming a path no document stands at, whose
    /// identifier one document holds at another path, is now reported and
    /// names that path. The first reported nothing for it.
    ///
    /// The third edition: a named pipe, a socket or a device in the store is
    /// reported as what it is, and its remediation says to delete it. The
    /// second reported it as a claim that names nobody and told the reader to
    /// write into it, and a write to a named pipe blocks (#1366).
    const VERSION: u32 = 3;
    const NEEDS_CLAIMS: bool = true;

    fn evaluate(&self, view: &CorpusView<'_>) -> Outcome {
        let Some(claims) = view.claims() else {
            return Outcome::Skipped(NO_STORE.to_string());
        };
        let mut findings = Vec::new();
        for claim in claims.entries() {
            let at_path = path_of(&claim.scheme, &claim.id);
            // A named pipe, a socket or a device, which the store never opens.
            // It names no document, and the remediation must not ask for a
            // write into it, because a write to a named pipe blocks (#1366).
            if claim.special {
                findings.push(Finding {
                    rule: self::STALE,
                    severity: Severity::Warn,
                    obligation: None,
                    path: at_path.clone(),
                    line: 0,
                    column: 0,
                    message: format!(
                        "`{at_path}` is a named pipe, a socket or a device, so it names no \
                         document and `{}` is claimed by nobody",
                        claim.id
                    ),
                    remediation: format!(
                        "delete `{at_path}`. If the identifier was spent, put a regular file \
                         there that holds the path of the document whose `{}` is `{}`",
                        self.facet, claim.id
                    ),
                    patch: None,
                });
                continue;
            }
            // A claim naming nothing. No writer of this engine produces one,
            // and a hand edit or a merge that concatenated two claims does. It
            // is reported against the claim itself, because there is no
            // document to report it against.
            if claim.claimant.is_empty() {
                findings.push(Finding {
                    rule: self::STALE,
                    severity: Severity::Warn,
                    obligation: None,
                    path: at_path.clone(),
                    line: 0,
                    column: 0,
                    message: format!(
                        "`{at_path}` names no document, so `{}` is claimed by nobody and two \
                         branches adding this claim would merge without a word",
                        claim.id
                    ),
                    remediation: format!(
                        "write the path of the document whose `{}` is `{}` into `{at_path}`, or \
                         delete the file if the identifier was never spent",
                        self.facet, claim.id
                    ),
                    patch: None,
                });
                continue;
            }
            let Some(entry) = self.index.by_path(&claim.claimant) else {
                // A claimant the corpus does not hold. Where no document holds
                // the identifier either, the document was deleted and the
                // claim is correct: spec 3 never reuses an identifier. Where
                // two or more hold it, `identifier.claimed_twice` reports the
                // pair, and a finding here would pick a winner in silence, as
                // `contended` says. Where exactly one holds it, the claim is
                // stale: the document was renamed, or the claim was edited by
                // hand, and the rule cannot see which. The finding is reported
                // against the current document rather than the claim file, as
                // the mismatch below is, so that SARIF and an `allow` directive
                // land on a Markdown file.
                //
                // A holder is a typed document whose kind mints under the
                // claim's scheme. An untyped file, or a document of another
                // scheme, that carries the same string is no holder: naming it
                // would tell the author to write a path into the store that no
                // rule of the scheme reads.
                let mut holders = self.index.paths.iter().filter(|entry| {
                    entry.id.as_deref() == Some(claim.id.as_str())
                        && entry
                            .kind
                            .as_deref()
                            .is_some_and(|kind| self.mints_under(kind, &claim.scheme))
                });
                let (Some(current), None) = (holders.next(), holders.next()) else {
                    continue;
                };
                findings.push(Finding {
                    rule: self::STALE,
                    severity: Severity::Warn,
                    obligation: None,
                    path: current.path.clone(),
                    line: 0,
                    column: 0,
                    message: format!(
                        "`{at_path}` names {}, and no document stands at that path. The one \
                         document holding `{}` is {}, so the claim names a path that is not its \
                         holder's, which follows a rename or a claim edited by hand",
                        claim.claimant, claim.id, current.path
                    ),
                    remediation: format!(
                        "write {} into `{at_path}` in place of {}",
                        current.path, claim.claimant
                    ),
                    patch: None,
                });
                continue;
            };
            let held = entry.id.as_deref().unwrap_or_default();
            if held == claim.id {
                continue;
            }
            findings.push(Finding {
                rule: self::STALE,
                severity: Severity::Warn,
                obligation: None,
                path: claim.claimant.clone(),
                line: 0,
                column: 0,
                message: match held.is_empty() {
                    true => format!(
                        "`{at_path}` names {} and that document declares no identifier, so the \
                         claim records a mint that no document carries",
                        claim.claimant
                    ),
                    false => format!(
                        "`{at_path}` names {} and that document declares `{held}`, so one of the \
                         two records a mint that did not happen",
                        claim.claimant
                    ),
                },
                remediation: format!(
                    "decide which of the two moved. Repoint `{at_path}` at the document whose \
                     `{}` is `{}`, or correct the document",
                    self.facet, claim.id
                ),
                patch: None,
            });
        }
        Outcome::failed(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use headwater_graph::index::PathEntry;

    fn entry(path: &str, id: &str) -> PathEntry {
        of_kind(path, id, Some("decision"))
    }

    fn of_kind(path: &str, id: &str, kind: Option<&str>) -> PathEntry {
        PathEntry {
            path: path.to_string(),
            class: match kind {
                Some(_) => "typed",
                None => "untyped",
            },
            id: Some(id.to_string()),
            kind: kind.map(str::to_string),
        }
    }

    fn claim(id: &str, claimant: &str) -> Claim {
        Claim {
            scheme: "decision_id".to_string(),
            id: id.to_string(),
            claimant: claimant.to_string(),
            special: false,
        }
    }

    fn findings(index: &Index, claims: &Claims) -> Vec<Finding> {
        let view = CorpusView::only_claims(Some(claims));
        let schemes = vec![
            ("decision".to_string(), "decision_id".to_string()),
            ("requirement".to_string(), "requirement_id".to_string()),
        ];
        match Stale::with_schemes("id", schemes, index).evaluate(&view) {
            Outcome::Passed => Vec::new(),
            Outcome::Failed(found) => found,
            other => panic!("the rule did not run: {other:?}"),
        }
    }

    /// The case table of `identifier.claim.stale` for a claimant the corpus
    /// does not hold. A rename is stale and names the current path. A deletion
    /// is correct, because an identifier is never reused. An identifier two
    /// documents hold is `identifier.claimed_twice`'s, and this rule picks no
    /// winner.
    ///
    /// A holder is a typed document whose kind mints under the claim's
    /// scheme. An untyped file that carries the string (DR-0004) and a
    /// document of another scheme that carries it (DR-0005) are no holder, so
    /// the claim reads as the deleted case. Neither counts toward two holders
    /// either (DR-0006), so a rename beside a stray copy is still a rename.
    #[test]
    fn a_renamed_claimant_is_stale_and_a_deleted_one_is_not() {
        let index = Index {
            paths: vec![
                entry("docs/decisions/0002-new-name.md", "DR-0002"),
                entry("docs/decisions/0003-one.md", "DR-0003"),
                entry("docs/decisions/0003-two.md", "DR-0003"),
                of_kind("docs/w3id/stray.md", "DR-0004", None),
                of_kind(
                    "docs/requirements/0005-other.md",
                    "DR-0005",
                    Some("requirement"),
                ),
                entry("docs/decisions/0006-new-name.md", "DR-0006"),
                of_kind("docs/w3id/stray-0006.md", "DR-0006", None),
            ],
            ..Index::default()
        };
        let claims = Claims::of(vec![
            claim("DR-0001", "docs/decisions/0001-deleted.md"),
            claim("DR-0002", "docs/decisions/0002-old-name.md"),
            claim("DR-0003", "docs/decisions/0003-gone.md"),
            claim("DR-0004", "docs/decisions/0004-gone.md"),
            claim("DR-0005", "docs/decisions/0005-gone.md"),
            claim("DR-0006", "docs/decisions/0006-old-name.md"),
        ]);

        let found = findings(&index, &claims);
        let paths: Vec<&str> = found.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(
            paths,
            vec![
                "docs/decisions/0002-new-name.md",
                "docs/decisions/0006-new-name.md"
            ],
            "{found:#?}"
        );
        let finding = &found[0];
        assert_eq!(finding.rule, STALE);
        assert_eq!(finding.severity, Severity::Warn);
        assert!(finding.patch.is_none(), "{finding:#?}");
        assert_eq!(finding.path, "docs/decisions/0002-new-name.md");
        assert!(
            finding
                .message
                .contains(".headwater/ids/decision_id/DR-0002"),
            "{finding:#?}"
        );
        assert!(
            finding.message.contains("docs/decisions/0002-old-name.md"),
            "{finding:#?}"
        );
        assert!(
            finding.message.contains("docs/decisions/0002-new-name.md"),
            "{finding:#?}"
        );
        assert!(
            finding
                .remediation
                .contains("docs/decisions/0002-new-name.md"),
            "{finding:#?}"
        );
        assert!(!finding.message.contains("DR-0001"), "{finding:#?}");
        // The rule sees a claim whose path is not its holder's path, and not
        // how that came about, so the message claims no rename.
        assert!(
            finding
                .message
                .contains("which follows a rename or a claim edited by hand"),
            "{finding:#?}"
        );
        assert!(!finding.message.contains("renamed"), "{finding:#?}");
    }

    /// A claim edited by hand to a path that no document ever stood at, while
    /// the document stayed put, is the same finding as a rename, and its
    /// message says nothing that is true of a rename alone.
    #[test]
    fn a_claim_edited_by_hand_to_a_wrong_path_is_stale_and_not_called_a_rename() {
        let index = Index {
            paths: vec![entry("docs/decisions/0007-the-title.md", "DR-0007")],
            ..Index::default()
        };
        let claims = Claims::of(vec![claim("DR-0007", "docs/decisions/0007-the-titel.md")]);

        let found = findings(&index, &claims);
        assert_eq!(found.len(), 1, "{found:#?}");
        let finding = &found[0];
        assert_eq!(finding.path, "docs/decisions/0007-the-title.md");
        assert!(
            finding.message.contains("docs/decisions/0007-the-titel.md"),
            "{finding:#?}"
        );
        assert!(!finding.message.contains("renamed"), "{finding:#?}");
        assert!(!finding.message.contains("no longer"), "{finding:#?}");
        assert!(
            finding
                .message
                .contains("which follows a rename or a claim edited by hand"),
            "{finding:#?}"
        );
    }

    /// The store reads a claim reached through a symlink, because the guard
    /// against a named pipe asks what the link names and not what the link
    /// is. A named pipe beside it reads as a claim that names nobody, and the
    /// read ends (#1366, verify round 1).
    #[cfg(unix)]
    #[test]
    fn a_claim_through_a_symlink_counts_and_a_named_pipe_names_nobody() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("a clock later than the epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "headwater-claim-link-{}-{nanos}",
            std::process::id()
        ));
        let scheme = root.join(STORE).join("decision_id");
        std::fs::create_dir_all(&scheme).expect("the scheme directory is made");
        std::fs::write(root.join("held"), contents_for("docs/decisions/0001-a.md"))
            .expect("the claim body writes");
        std::os::unix::fs::symlink(root.join("held"), scheme.join("HW-DR-0001"))
            .expect("the link is made");
        let made = std::process::Command::new("mkfifo")
            .arg(scheme.join("HW-DR-0002"))
            .status()
            .expect("mkfifo runs");
        assert!(made.success(), "the named pipe is made");

        let at = root.clone();
        let (send, receive) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let claims = Claims::at(&at);
            send.send((
                claims
                    .claimant("decision_id", "HW-DR-0001")
                    .map(str::to_string),
                claims
                    .claimant("decision_id", "HW-DR-0002")
                    .map(str::to_string),
            ))
            .expect("the answer is sent");
        });
        let (linked, piped) = receive
            .recv_timeout(std::time::Duration::from_secs(30))
            .expect("Claims::at opened the named pipe, and waited on it");
        std::fs::remove_dir_all(&root).ok();

        assert_eq!(linked.as_deref(), Some("docs/decisions/0001-a.md"));
        assert_eq!(piped.as_deref(), Some(""));
    }

    /// The store opens a regular file, directly or through a link, and no
    /// named pipe, socket or device. A device is asked about and never read:
    /// a read of `/dev/zero` does not end, so a store that read one would not
    /// fail this case, it would never finish it. `/dev/null` stands in, and
    /// this asks the guard itself (#1366, verify round 2).
    #[cfg(unix)]
    #[test]
    fn the_store_opens_a_regular_file_and_no_pipe_socket_or_device() {
        // A socket path must be shorter than 108 bytes, and a runner's
        // temporary directory can be longer than that by itself, so the
        // directory is in `/tmp` whatever `TMPDIR` says. Cargo runs a target's
        // cases as threads of one process, so the pid alone is not a key.
        let thread: String = format!("{:?}", std::thread::current().id())
            .chars()
            .filter(char::is_ascii_digit)
            .collect();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("a clock later than the epoch")
            .subsec_nanos();
        let dir = std::path::PathBuf::from(format!(
            "/tmp/hw-claim-{}-{thread}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("the directory is made");
        std::fs::write(dir.join("file"), "docs/a.md\n").expect("the file writes");
        std::os::unix::fs::symlink(dir.join("file"), dir.join("link")).expect("the link is made");
        std::os::unix::fs::symlink("/dev/null", dir.join("device")).expect("the link is made");
        std::os::unix::fs::symlink("/dev/zero", dir.join("endless")).expect("the link is made");
        let made = std::process::Command::new("mkfifo")
            .arg(dir.join("pipe"))
            .status()
            .expect("mkfifo runs");
        assert!(made.success(), "the named pipe is made");
        let socket = std::os::unix::net::UnixListener::bind(dir.join("socket"))
            .expect("the socket is bound");
        std::os::unix::fs::symlink(dir.join("gone"), dir.join("dangling"))
            .expect("the link is made");
        std::os::unix::fs::symlink(&dir, dir.join("dirlink")).expect("the link is made");

        let answers: Vec<(&str, Entry)> = [
            "file", "link", "device", "endless", "pipe", "socket", "dangling", "dirlink",
        ]
        .into_iter()
        .map(|name| (name, entry_of(&dir.join(name))))
        .collect();
        drop(socket);
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(
            answers,
            vec![
                ("file", Entry::Regular),
                ("link", Entry::Regular),
                ("device", Entry::Special),
                ("endless", Entry::Special),
                ("pipe", Entry::Special),
                ("socket", Entry::Special),
                ("dangling", Entry::Unreadable),
                ("dirlink", Entry::Unreadable),
            ]
        );
    }

    /// A named pipe, a socket or a device in the store names no document, and
    /// `claim.stale` says what it is. A write to a pipe blocks, so the
    /// remediation never tells the reader to write into the entry: it says to
    /// delete it. A regular file that holds no line keeps the empty-claimant
    /// sentence, and the store's digest tells the two apart (#1366).
    #[cfg(unix)]
    #[test]
    fn a_pipe_socket_or_device_in_the_store_is_reported_as_what_it_is() {
        let thread: String = format!("{:?}", std::thread::current().id())
            .chars()
            .filter(char::is_ascii_digit)
            .collect();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("a clock later than the epoch")
            .subsec_nanos();
        // Under `/tmp` for the socket's 108-byte path limit, as above.
        let root = std::path::PathBuf::from(format!(
            "/tmp/hw-stale-{}-{thread}-{nanos}",
            std::process::id()
        ));
        let scheme = root.join(STORE).join("decision_id");
        std::fs::create_dir_all(&scheme).expect("the directory is made");
        std::os::unix::fs::symlink("/dev/null", scheme.join("DR-0001")).expect("the link is made");
        let made = std::process::Command::new("mkfifo")
            .arg(scheme.join("DR-0002"))
            .status()
            .expect("mkfifo runs");
        assert!(made.success(), "the named pipe is made");
        let socket = std::os::unix::net::UnixListener::bind(scheme.join("DR-0003"))
            .expect("the socket is bound");
        std::fs::write(scheme.join("DR-0004"), "").expect("the empty claim writes");
        // A dangling link and a link to a directory are not special: neither
        // is a pipe, a socket or a device, so each keeps the empty-claimant
        // sentence (#1366, verify round 1).
        std::os::unix::fs::symlink(root.join("gone"), scheme.join("DR-0005"))
            .expect("the dangling link is made");
        std::os::unix::fs::symlink(&root, scheme.join("DR-0006"))
            .expect("the directory link is made");

        let claims = Claims::at(&root);
        let empty = Claims::of(
            claims
                .entries()
                .iter()
                .map(|claim| Claim {
                    special: false,
                    ..claim.clone()
                })
                .collect(),
        );
        drop(socket);
        std::fs::remove_dir_all(&root).ok();

        let special: Vec<(&str, bool)> = claims
            .entries()
            .iter()
            .map(|claim| (claim.id.as_str(), claim.special))
            .collect();
        assert_eq!(
            special,
            vec![
                ("DR-0001", true),
                ("DR-0002", true),
                ("DR-0003", true),
                ("DR-0004", false),
                ("DR-0005", false),
                ("DR-0006", false),
            ]
        );
        assert_ne!(
            claims.digest(),
            empty.digest(),
            "the digest tells a special entry from an empty file"
        );

        let found = findings(&Index::default(), &claims);
        assert_eq!(found.len(), 6, "{found:#?}");
        for finding in &found[..3] {
            assert!(
                finding
                    .message
                    .contains("is a named pipe, a socket or a device, so it names no document"),
                "{finding:#?}"
            );
            assert!(
                !finding.remediation.contains("write the path"),
                "{finding:#?}"
            );
            assert!(finding.remediation.starts_with("delete `"), "{finding:#?}");
        }
        for (finding, id) in found[3..].iter().zip(["DR-0004", "DR-0005", "DR-0006"]) {
            assert!(
                finding
                    .message
                    .contains(&format!("names no document, so `{id}`")),
                "{finding:#?}"
            );
        }
    }
}
