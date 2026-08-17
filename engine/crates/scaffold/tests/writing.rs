// SPDX-License-Identifier: Apache-2.0
//! What `headwater new` leaves on a tree when it cannot finish.
//!
//! The claim [#187](https://github.com/headwater-ai/headwater/issues/187) is
//! about. [`write::compose`] produces one new document and the reciprocal end
//! of every edge it declares, and [`write::apply`] used to put them on the tree
//! with a loop of `std::fs::write` that stopped on its first error. The new
//! document is composed first, so a reciprocal end that could not be written
//! left it on disk declaring one half of an edge — the exact state
//! `relation.reciprocity.missing` reports, over a whole corpus, about a
//! document the author never chose to keep.
//!
//! Every case here is written against `apply` rather than against the writer
//! underneath it, because the writer's own guarantees are held in
//! `headwater_scaffold::tree` and what this file has to say is that the verb
//! reaches them. The first case is the issue's measurement in miniature and the
//! only one that fires in the phase the measurement fires in.
//!
//! Each case also reaches one branch of the write-phase refusals, which
//! `fixtures.rs` names and requires a case for.

use headwater_scaffold::write::{self, Composed};
use headwater_scaffold::Refusal;
use std::path::{Path, PathBuf};

/// A directory this test owns, named for the case rather than the process.
///
/// Cargo runs the cases of one target as threads of one process, so a
/// directory keyed on the process identifier alone is one that a second case
/// removes while the first is reading it.
struct Dir(PathBuf);

impl Dir {
    fn with(label: &str, files: &[(&str, &str)]) -> Dir {
        let root =
            std::env::temp_dir().join(format!("headwater-writing-{}-{label}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch directory");
        for (path, text) in files {
            std::fs::write(root.join(path), text).expect("a fixture file");
        }
        Dir(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn read(&self, name: &str) -> String {
        std::fs::read_to_string(self.0.join(name)).expect("the file reads")
    }

    fn holds(&self, name: &str) -> bool {
        std::fs::symlink_metadata(self.0.join(name)).is_ok()
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn new(path: &str, text: &str) -> Composed {
    Composed {
        path: path.to_string(),
        text: text.to_string(),
        created: true,
    }
}

fn edited(path: &str, text: &str) -> Composed {
    Composed {
        path: path.to_string(),
        text: text.to_string(),
        created: false,
    }
}

fn lock(at: &Path) {
    let mut permissions = std::fs::metadata(at)
        .expect("the file is there")
        .permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(at, permissions).expect("the file locks");
}

/// The measurement, and the assertion the whole change is for.
///
/// The reciprocal end is read-only. `compose` never sees it, because a
/// `read_to_string` of a `0444` file succeeds and that is the only read it
/// does — which is why the failure used to land three steps later, after the
/// new document was already on disk.
///
/// **The *before* assertion is deliberately loose.** That `apply` returns an
/// error is true of the old writer too, so if that assertion were the tight one
/// it would be the failure that fires and the case would look like it held
/// something it does not. The decisive assertion is the absence below it: the
/// ambient outcome of this exact scenario, measured against the writer this
/// change replaces, is `new.md` **present**.
#[test]
fn a_reciprocal_that_cannot_be_opened_leaves_the_new_document_absent() {
    let dir = Dir::with("read-only-reciprocal", &[("recip.md", "---\nid: A\n---\n")]);
    lock(&dir.path().join("recip.md"));

    let refused = write::apply(
        dir.path(),
        &[
            new("new.md", "---\nid: B\n---\n"),
            edited("recip.md", "---\nid: A\nrelations:\n---\n"),
        ],
    )
    .expect_err("the reciprocal end cannot be opened for writing");

    assert!(
        !dir.holds("new.md"),
        "the new document is not on the tree, and the writer this replaces put it there first"
    );
    assert_eq!(
        dir.read("recip.md"),
        "---\nid: A\n---\n",
        "and the document it could not write is as it was"
    );
    assert!(
        matches!(refused, Refusal::TargetUnopened { .. }),
        "{refused}"
    );
    assert!(
        refused
            .to_string()
            .contains("Nothing was written and no document was created"),
        "the sentence the author reads is true of this state: {refused}"
    );
}

/// A path that is already taken refuses, and the reciprocal end does not move.
///
/// `propose` refuses this with `PathTaken` before a writer ever runs, off an
/// index of the census. This is the same question asked of the kernel instead,
/// one syscall wide, so a file that reached the path after the index was built
/// is a refusal rather than an overwrite.
#[test]
fn a_document_already_at_the_new_path_is_refused_and_no_edit_lands() {
    let dir = Dir::with(
        "path-taken",
        &[
            ("recip.md", "---\nid: A\n---\n"),
            ("new.md", "SOMEBODY ELSE"),
        ],
    );

    let refused = write::apply(
        dir.path(),
        &[
            new("new.md", "---\nid: B\n---\n"),
            edited("recip.md", "EDITED"),
        ],
    )
    .expect_err("the new document's path is taken");

    assert_eq!(
        dir.read("new.md"),
        "SOMEBODY ELSE",
        "the file at the path is somebody else's and this run never opened it for writing"
    );
    assert_eq!(
        dir.read("recip.md"),
        "---\nid: A\n---\n",
        "and the edit did not land either"
    );
    assert!(
        matches!(refused, Refusal::DocumentUncreated { .. }),
        "{refused}"
    );
}

/// A write that does not read back off the tree halts, and the new document
/// goes with it.
///
/// Two entries name one path, so the second write lands over the first and the
/// read back of the first finds bytes that are not the ones it wrote. No
/// composition produces that — [`write::compose`] merges two halves into one
/// entry — and it is used here because it is the one way to reach the halt
/// through the public verb without a seam. What the case is about is the
/// rollback: by the time it runs, the new document is on disk holding this
/// run's own bytes, and it still has to go.
#[test]
fn a_write_that_does_not_read_back_halts_and_the_new_document_goes_with_it() {
    let dir = Dir::with("no-read-back", &[("recip.md", "was")]);

    let refused = write::apply(
        dir.path(),
        &[
            new("new.md", "NEW"),
            edited("recip.md", "FIRST"),
            edited("recip.md", "SECOND"),
        ],
    )
    .expect_err("the first write does not read back");

    assert!(
        !dir.holds("new.md"),
        "the document this run created is off the tree"
    );
    assert_eq!(dir.read("recip.md"), "was", "and the edited file is back");
    match &refused {
        Refusal::WriteHalted { path, report } => {
            assert_eq!(path, "recip.md");
            assert!(
                report.contains("the document this run created at new.md was removed"),
                "the report is the rollback's own account: {report}"
            );
            assert!(
                report.contains("the tree is as it was"),
                "everything went back and the new document went: {report}"
            );
        }
        other => panic!("{other}"),
    }
}

/// A composition naming two new documents is refused rather than chosen
/// between.
///
/// The undo of a create is an unlink, and a writer that met two candidates
/// would have to decide which path it may remove. [`write::compose`] pushes
/// exactly one, so this is a defect in the scaffolder rather than a state a
/// user can reach — and the point of the refusal is that it stays that way.
#[test]
fn a_composition_with_two_new_documents_is_refused_rather_than_chosen_between() {
    let dir = Dir::with("two-new", &[]);

    let refused = write::apply(dir.path(), &[new("one.md", "ONE"), new("two.md", "TWO")])
        .expect_err("two new documents is not a scaffold");
    assert!(!dir.holds("one.md"), "and neither of them was written");
    assert!(!dir.holds("two.md"));
    assert!(
        matches!(refused, Refusal::NotOneDocument { created: 2 }),
        "{refused}"
    );

    let refused = write::apply(dir.path(), &[edited("recip.md", "EDITED")])
        .expect_err("a scaffold that creates nothing is not one either");
    assert!(
        matches!(refused, Refusal::NotOneDocument { created: 0 }),
        "{refused}"
    );
}

/// The ordinary run: the new document lands, the reciprocal end is edited, and
/// the shelf directory the document needed is made on the way.
///
/// An adopter's first document on a shelf is the case where the directory is
/// not there yet, and it is the case the old writer's `create_dir_all` served.
#[test]
fn a_run_that_can_finish_writes_the_new_document_and_the_edited_one() {
    let dir = Dir::with("lands", &[("recip.md", "was")]);

    write::apply(
        dir.path(),
        &[new("shelf/new.md", "NEW"), edited("recip.md", "EDITED")],
    )
    .expect("both files write");

    assert_eq!(dir.read("shelf/new.md"), "NEW");
    assert_eq!(dir.read("recip.md"), "EDITED");
}
