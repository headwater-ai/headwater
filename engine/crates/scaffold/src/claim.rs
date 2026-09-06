// SPDX-License-Identifier: Apache-2.0
//! The writer of the identifier claim store.
//!
//! The store itself, its shape and the two rules that hold it are
//! [`headwater_check::claim`]. This module is the one thing that puts a file
//! into it, and it imports the path and the format from there rather than
//! writing either a second time.
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
//! If the document write then fails, the residue is a spent number with no
//! document.
//! [Spec 3](../../../../docs/spec/03-authoring-and-lifecycle.md#identifiers)
//! already declares that state legitimate: "**Never reused.** … A deleted
//! document does not free its number." The other order can leave a document
//! with no claim, which is exactly the defect
//! `identifier.claim.missing` exists to report.

use crate::{Plan, Refusal};
use headwater_check::claim::{contents_for, path_of};
use std::path::Path;

/// Write the claim this plan mints, and nothing for a plan that mints none.
///
/// A run that mints no identifier writes no file, and so does a run whose
/// scheme allocates `minted-once`: a slug collision renames a document and git
/// reports it, so the store has no work to do there.
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

/// The claim path this plan owes, and nothing where it owes none.
pub fn claimed(plan: &Plan) -> Option<String> {
    let minting = plan.minting.as_ref()?;
    match minting.allocation.as_deref() {
        Some(headwater_check::claim::RECONCILE_FIRST) => {
            Some(path_of(&minting.scheme, &minting.id))
        }
        _ => None,
    }
}
