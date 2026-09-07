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
//! granularity. `tools/id-store-fixtures.sh` pins both outcomes, and its
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
//! **A claim naming a document the corpus no longer holds is correct, and no
//! rule reports it.** Spec 3: "**Never reused.** … A deleted document does not
//! free its number." The store is the first artifact of this repository that
//! can hold that fact at all, and a rule reporting it would report every
//! legitimate deletion as a defect.

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
                let claimant = std::fs::read_to_string(claim.path())
                    .unwrap_or_default()
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                entries.push(Claim {
                    scheme: scheme_name.clone(),
                    id,
                    claimant,
                });
            }
        }
        entries.sort_by(|a, b| (&a.scheme, &a.id).cmp(&(&b.scheme, &b.id)));
        Claims { entries }
    }

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
    /// claimant can write the separator of the next entry.
    pub fn render(&self) -> String {
        let mut out = String::new();
        for claim in &self.entries {
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
    /// reports 16 and fails a strict run. Every test of this workspace starts
    /// cold, so none of them can report that difference.
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
}

impl<'a> Stale<'a> {
    pub fn over(facet: &str, index: &'a Index) -> Self {
        Stale {
            index,
            facet: facet.to_string(),
        }
    }
}

impl CorpusCheck for Stale<'_> {
    const RULE: &'static str = self::STALE;
    /// The first edition of this rule.
    const VERSION: u32 = 1;
    const NEEDS_CLAIMS: bool = true;

    fn evaluate(&self, view: &CorpusView<'_>) -> Outcome {
        let Some(claims) = view.claims() else {
            return Outcome::Skipped(NO_STORE.to_string());
        };
        let mut findings = Vec::new();
        for claim in claims.entries() {
            let at_path = path_of(&claim.scheme, &claim.id);
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
            // A claimant the corpus does not hold is correct: spec 3 never
            // reuses an identifier, so a deleted document leaves its claim
            // standing. This is the direction the rule deliberately does not
            // report.
            let Some(entry) = self.index.by_path(&claim.claimant) else {
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
