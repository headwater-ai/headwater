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
//! [`write`](fn@write) opens with `create_new`, so the test for an existing claim and the
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
//! for the document it created. The same permission removes a claim whose own
//! bytes did not all land, so this writer never leaves an empty or partial
//! claim (#2041). One refusal keeps the claim: where the run could not remove
//! the document it created, the document is still on the tree, and a claim
//! stands behind its document, so [`settle`] keeps it and says so (#2041).
//! Only a run that dies between the two writes still leaves a claim with no
//! document, and
//! [Spec 3](../../../../docs/spec/authoring-and-lifecycle.md#identifiers)
//! declares that state legitimate: "**Never reused.** … A deleted document
//! does not free its number." The other order can leave a document with no
//! claim, which is exactly the defect `identifier.claim.missing` exists to
//! report.

use crate::{Plan, Refusal};
use headwater_check::claim::contents_for;
use std::fs::File;
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
/// The primitive, held apart from [`write`](fn@write) so that a test reaches the refusal
/// without composing a whole plan, and so that the create-new decision is one
/// function rather than a property of the caller.
pub fn make(root: &Path, claim: &str, claimant: &str) -> Result<(), Refusal> {
    make_with(root, claim, claimant, &mut |file, bytes| {
        std::io::Write::write_all(file, bytes)
    })
}

/// [`make`], with the write of the claim's bytes passed in.
///
/// The seam exists for one test. No unprivileged, thread-safe test can make a
/// `write_all` fail on a file that `create_new` just opened: `/dev/full` and a
/// symlink both fail at the create. It is the reasoning of
/// [`crate::tree::Reserved`]'s own `commit_with`.
fn make_with(
    root: &Path,
    claim: &str,
    claimant: &str,
    write: &mut dyn FnMut(&mut File, &[u8]) -> std::io::Result<()>,
) -> Result<(), Refusal> {
    let at = root.join(claim);
    let refuse = |why: String| Refusal::ClaimUnwritable {
        path: claim.to_string(),
        why,
        removed: None,
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
    // The `Ok` above is the permission to remove this path: `create_new` made
    // it in this run, so it holds no other claimant's record. A claim that did
    // not write whole is empty or partial, and leaving it would spend a number
    // on a file no reader can parse (#2041).
    if let Err(error) = write(&mut file, contents_for(claimant).as_bytes()) {
        drop(file);
        return Err(Refusal::ClaimUnwritable {
            path: claim.to_string(),
            why: error.to_string(),
            removed: Some(withdraw(root, claim)),
        });
    }
    Ok(())
}

/// Write the claim this plan mints, then the composed files, and withdraw the
/// claim when the files cannot be written.
///
/// The one caller of [`write`](fn@write), so the order and its undo live in one place. A
/// claim that cannot be made refuses with no document written. A refusal of
/// [`crate::write::apply`] goes to [`settle`], which removes the claim this
/// call made, with [`withdraw`], unless the created document is still on the
/// tree. The message ends with the sentence [`withdrawn`] or [`kept`] writes,
/// so it names what became of the claim (#1972, #2041).
pub fn land(
    root: &Path,
    plan: &Plan,
    composed: &[crate::write::Composed],
) -> Result<Option<String>, String> {
    let claimed = write(root, plan).map_err(|why| why.to_string())?;
    let Err(why) = crate::write::apply(root, composed) else {
        return Ok(claimed);
    };
    Err(match &claimed {
        None => why.to_string(),
        Some(claim) => settle(root, claim, &why),
    })
}

/// What a run that made `claim` does with it when the files were refused, and
/// the message it then reports.
///
/// The decision of [`land`], held apart so that a test reaches every refusal
/// without a disk that fails on cue.
///
/// A refusal that left the created document on the tree keeps the claim,
/// because a claim stands behind its document and a document with no claim is
/// the defect `identifier.claim.missing` reports (#2041). Every other refusal
/// left no document of this run, so the claim is withdrawn (#1972).
pub fn settle(root: &Path, claim: &str, refusal: &Refusal) -> String {
    let why = refusal.to_string();
    if let Refusal::WriteHalted {
        left: Some(document),
        ..
    } = refusal
    {
        return joined(&why, &kept(claim, document));
    }
    let outcome = withdraw(root, claim);
    joined(&why, &withdrawn(claim, &outcome))
}

/// A refusal and the claim sentence after it, with one period and one space
/// between them whether or not the refusal already ends in a period.
pub fn joined(refusal: &str, sentence: &str) -> String {
    format!("{}. {sentence}", refusal.trim_end_matches('.'))
}

/// Remove one claim file that this run made, because its document was not
/// created.
///
/// Call it only with a path that [`write`](fn@write) or [`make`] returned `Ok` for in
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

/// The sentence a failed run adds to its refusal when the document it created
/// is still on the tree, and the claim it made for that document stays.
pub fn kept(claim: &str, document: &str) -> String {
    format!(
        "The claim {claim} this run wrote was kept, because the document {document} it stands \
         behind is still on the tree."
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    /// A directory that cleans itself up, keyed on the case so that two cases
    /// of this target, which run as threads of one process, never share one.
    struct Dir(PathBuf);

    impl Dir {
        fn new(label: &str) -> Dir {
            let path = std::env::temp_dir()
                .join(format!("headwater-claim-{label}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("the case directory is made");
            Dir(path)
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    const CLAIM: &str = ".headwater/ids/decision_id/HW-DR-0049";

    /// A claim whose bytes did not all land is removed, so this writer never
    /// leaves an empty or partial claim behind, and the refusal says the claim
    /// went (#2041).
    #[test]
    fn a_claim_that_did_not_write_whole_is_removed_and_the_refusal_says_so() {
        let dir = Dir::new("unwritten");
        let refused = make_with(
            &dir.0,
            CLAIM,
            "docs/decisions/0049-x.md",
            &mut |file, bytes| {
                file.write_all(&bytes[..3])?;
                Err(std::io::Error::from_raw_os_error(28))
            },
        )
        .expect_err("a failed write refuses");

        assert!(
            !dir.0.join(CLAIM).exists(),
            "no claim file is left at the path, empty or partial"
        );
        assert!(
            matches!(
                &refused,
                Refusal::ClaimUnwritable {
                    removed: Some(Ok(())),
                    ..
                }
            ),
            "{refused:?}"
        );
        let message = refused.to_string();
        assert!(
            message.contains(&format!("The claim {CLAIM} this run wrote was removed.")),
            "{message}"
        );
        assert!(!message.contains("Nothing else was written"), "{message}");
    }

    /// The twin whose removal fails: the refusal names the claim still there
    /// and asks for it to be deleted by hand, rather than saying nothing was
    /// written.
    #[test]
    fn a_claim_that_could_not_be_removed_is_named_for_deletion_by_hand() {
        let dir = Dir::new("unremoved");
        let at = dir.0.join(CLAIM);
        let refused = make_with(&dir.0, CLAIM, "docs/decisions/0049-x.md", &mut |_, _| {
            // Take the file away first, so the withdraw meets nothing there.
            std::fs::remove_file(&at)?;
            Err(std::io::Error::from_raw_os_error(28))
        })
        .expect_err("a failed write refuses");

        assert!(
            matches!(
                &refused,
                Refusal::ClaimUnwritable {
                    removed: Some(Err(_)),
                    ..
                }
            ),
            "{refused:?}"
        );
        let message = refused.to_string();
        assert!(message.contains("could not be removed"), "{message}");
        assert!(message.contains("Delete it by hand."), "{message}");
        assert!(!message.contains("Nothing else was written"), "{message}");
    }

    /// A create that fails leaves no file of this run, so the refusal names no
    /// removal and still says that nothing else was written.
    #[test]
    fn a_claim_whose_create_failed_names_no_removal() {
        let dir = Dir::new("taken");
        make(&dir.0, CLAIM, "docs/decisions/0049-first.md").expect("the first claim lands");
        let refused =
            make(&dir.0, CLAIM, "docs/decisions/0049-second.md").expect_err("the path is taken");
        assert!(
            matches!(&refused, Refusal::ClaimUnwritable { removed: None, .. }),
            "{refused:?}"
        );
        assert!(
            refused.to_string().contains("Nothing else was written"),
            "{refused}"
        );
    }
}
