// SPDX-License-Identifier: Apache-2.0
//! The writer of the identifier claim store.
//!
//! The store itself, its shape and the two rules that hold it are
//! [`headwater_check::claim`]. This module writes the claim of a mint. The
//! store has a second writer, [`crate::fix::make`], which creates the claim
//! file that an `identifier.claim.missing` finding names. Both create with
//! `create_new` and never overwrite. This module imports the format,
//! [`contents_for`], from the store's module rather than writing it a second
//! time. The path reaches it on [`crate::Minting::claim`], which
//! [`crate::propose`] took from [`headwater_check::claim::path_of`].
//!
//! # Create-new, and why it is the strictest line in this crate
//!
//! [`write`] opens with `create_new`, so the test for an existing claim and the
//! creation of one are a single syscall and there is no window between them.
//! `AlreadyExists` is a refusal.
//!
//! A claim file is the only record of which document minted an identifier. The
//! tree does not hold it — that a tree cannot hold it is the whole reason the
//! store exists — and nothing in this engine ever modifies a claim. So a writer
//! that truncated one would delete a fact that no later run can reconstruct,
//! which is a worse failure than any refusal this crate can report.
//!
//! # The order: the claim first, the document second
//!
//! [`crate::propose`] refuses with [`crate::Refusal::IdentifierTaken`] where the
//! store already holds the value, so in ordinary operation this create never
//! meets an occupied path. Where it does — two processes minting at once — the
//! create fails, nothing is written at all, and the refusal is atomic.
//!
//! If the document write then fails, the run withdraws the claim it wrote with
//! [`withdraw`], so a failed run spends no number (#1972). The permission to
//! remove that file is the `Ok` that `create_new` returned for it in this run:
//! no other claimant's record can be at that path, so the removal is not an
//! overwrite of anybody's fact. It is the rule [`crate::write::apply`] follows
//! for the document it created. Only a run that dies between the two writes
//! still leaves a claim with no document, and
//! [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#identifiers)
//! declares that state legitimate: "**Never reused.** … A deleted document
//! does not free its number." The other order can leave a document with no
//! claim, which is exactly the defect `identifier.claim.missing` exists to
//! report.

use crate::{Plan, Refusal};
use headwater_check::claim::contents_for;
use std::path::Path;

/// Write the claim this plan mints, and nothing for a plan that mints none.
///
/// A run that mints no identifier writes no file, and neither does a run whose
/// shelf declares no `layout` under a scheme that does not reconcile: there the
/// file name is the slug the identifier already patterns, so a collision
/// renames a document and git reports it.
/// [`headwater_check::claim::takes_a_claim`] is the predicate,
/// [`crate::propose`] is the one caller of it, and the answer reaches here on
/// the plan.
pub fn write(root: &Path, plan: &Plan) -> Result<Option<String>, Refusal> {
    let Some(claim) = claimed(plan) else {
        return Ok(None);
    };
    make(root, &claim, &plan.path)?;
    Ok(Some(claim))
}

/// Make one claim file, or refuse.
///
/// The primitive, held apart from [`write`] so that a test reaches the refusal
/// without composing a whole plan, and so that the create-new decision is one
/// function rather than a property of the caller.
pub fn make(root: &Path, claim: &str, claimant: &str) -> Result<(), Refusal> {
    let at = root.join(claim);
    let refuse = |why: String| Refusal::ClaimUnwritable {
        path: claim.to_string(),
        why,
    };
    if let Some(parent) = at.parent() {
        std::fs::create_dir_all(parent).map_err(|error| refuse(error.to_string()))?;
    }
    // `create_new`, so the test for an existing claim and the create are one
    // syscall and `AlreadyExists` is a refusal. See the module comment: an
    // overwrite here destroys the only record of the other claimant.
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&at)
        .map_err(|error| refuse(error.to_string()))?;
    std::io::Write::write_all(&mut file, contents_for(claimant).as_bytes())
        .map_err(|error| refuse(error.to_string()))?;
    Ok(())
}

/// Write the claim this plan mints, then the composed files, and withdraw the
/// claim when the files cannot be written.
///
/// The one caller of [`write`], so the order and its undo live in one place. A
/// claim that cannot be made refuses with nothing written. A refusal of
/// [`crate::write::apply`] removes the claim this call made, with [`withdraw`],
/// and the message ends with the sentence [`withdrawn`] writes, so it names
/// the claim it removed or the claim it could not remove (#1972).
pub fn land(
    root: &Path,
    plan: &Plan,
    composed: &[crate::write::Composed],
) -> Result<Option<String>, String> {
    let claimed = write(root, plan).map_err(|why| why.to_string())?;
    let Err(why) = crate::write::apply(root, composed) else {
        return Ok(claimed);
    };
    let why = why.to_string();
    Err(match &claimed {
        None => why,
        Some(claim) => {
            let outcome = withdraw(root, claim);
            format!(
                "{}. {}",
                why.trim_end_matches('.'),
                withdrawn(claim, &outcome)
            )
        }
    })
}

/// Remove one claim file that this run made, because its document was not
/// created.
///
/// Call it only with a path that [`write`] or [`make`] returned `Ok` for in
/// this run. That `Ok` is the permission: `create_new` made the file, so the
/// path held no other claimant's record, and removing it destroys no fact. A
/// claim this run did not make is never passed here, and this function touches
/// no other path, the shelf directory of the scheme included.
pub fn withdraw(root: &Path, claim: &str) -> Result<(), String> {
    std::fs::remove_file(root.join(claim)).map_err(|error| error.to_string())
}

/// The sentence a failed run adds to its refusal, after it has tried to
/// withdraw the claim it made.
///
/// The refusals of [`crate::write::apply`] speak of documents only, so a run
/// that made a claim says what became of it, and the whole message never says
/// that nothing was written while a claim is on disk.
pub fn withdrawn(claim: &str, outcome: &Result<(), String>) -> String {
    match outcome {
        Ok(()) => format!("The claim {claim} this run wrote was removed."),
        Err(why) => format!(
            "The claim {claim} this run wrote could not be removed: {why}. Delete it by hand."
        ),
    }
}

/// The claim path this plan owes, and nothing where it owes none.
///
/// It reads the plan and decides nothing. The decision was made once, against
/// the shelf and the scheme, by the predicate
/// [`headwater_check::claim::takes_a_claim`] that `identifier.claim.missing`
/// reads too.
pub fn claimed(plan: &Plan) -> Option<String> {
    plan.minting.as_ref()?.claim.clone()
}
