// SPDX-License-Identifier: Apache-2.0
//! Putting a set of already-composed files onto a tree: all of them, or none.
//!
//! Every writer in this engine composes every byte of every file before a byte
//! reaches disk. That discipline is what [`crate::fix`] and [`crate::write`]
//! both call "nothing is written until everything can be", and until this
//! module it was only half true. Composition cannot fail after it has finished,
//! but a *write* can, and a loop of `std::fs::write` that stops on the first
//! error leaves every file before it written and every file after it not.
//!
//! That state is the one a caller can least act on. The tree then satisfies
//! neither the schema it came from nor the one it was moving to, and nothing
//! records which half it is in.
//!
//! # The guarantee, and the one thing it does not cover
//!
//! [`Reserved::over`] opens every target for writing before any byte is
//! written. A path that is read-only, that is a directory, or that is missing
//! fails there, and a [`Reserved`] cannot be built — so the run refuses with the
//! tree exactly as it was. The permission case is the one that actually
//! happens, and holding the handle rather than probing it is what makes the
//! answer a fact rather than a guess a later `open` could contradict.
//!
//! [`Reserved::commit`] writes through those handles, reads every file back off
//! the tree, and compares. A failure at either point restores every file the
//! run touched from the bytes it read before writing, **including the one it
//! failed on** — a write that stops part way has already emptied its target and
//! rewritten a prefix of it, so that file is the likeliest damage of the whole
//! run and the last one to leave outside the rollback. A restore that fails in
//! its turn is named, and [`Halted`] says the tree is as it was only when it is.
//!
//! **A crash of the process between two writes is not covered**, and no
//! user-space scheme covers it without a journal. What this module removes is
//! the case a run can see and report: a write that fails while the run is still
//! there to undo it.
//!
//! # Why the handles are held rather than the paths re-opened
//!
//! A probe that opened each file and closed it would answer about a moment that
//! has passed by the time the write runs. Holding the handle means the state
//! [`Reserved`] asserts is the state the write uses. The cost is one file
//! descriptor per document of one run, which is the size of the set a migration
//! step names rather than the size of the corpus.

use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};

/// One composed file, waiting for a tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Composed {
    /// Relative to the root, written with `/`.
    pub path: String,
    pub text: String,
}

/// Every file of a run, opened for writing, before a byte is written.
///
/// The only constructor is [`Reserved::over`], so a value of this type is a
/// statement that every target could be opened. There is no arm in which a
/// caller holds one of these and a target is unwritable.
pub struct Reserved {
    held: Vec<Held>,
}

struct Held {
    path: String,
    at: PathBuf,
    file: std::fs::File,
    /// What the file held before the run. This is what a rollback writes back,
    /// and it is read through the same handle so that it is the content the
    /// write is about to replace.
    was: String,
    now: String,
}

/// A target this run will not write, with the tree untouched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unopened {
    pub path: String,
    pub why: String,
}

impl std::fmt::Display for Unopened {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} could not be opened for writing: {}",
            self.path, self.why
        )
    }
}

/// What the file the run failed on holds now.
///
/// A write that fails part way has already emptied its target and written a
/// prefix of the new text over it, so this is the file most likely to be
/// damaged of the whole run. It is never one of `restored`, because it was
/// never written in full.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failed {
    /// Put back, from the bytes read before the run, through the handle the run
    /// still held.
    PutBack,
    /// Not put back. This file holds neither what it held nor what the run
    /// asked for.
    ///
    /// Not back for either of two reasons: the restore could not be written, or
    /// it was written, returned `Ok`, and the path does not hold it — the state
    /// a handle whose path was unlinked or replaced under it produces. `why`
    /// says which.
    Damaged { why: String },
}

/// A run that stopped part way, and what it did about it.
///
/// The fields are private and `undo` is the only constructor, so a value of
/// this type is a report of a rollback that ran and not an assertion about a
/// tree nothing observed. That is worth the two accessors because this type's
/// [`std::fmt::Display`] prints *the tree is as it was*, and nothing outside
/// this module is in a position to say it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Halted {
    /// The file the run failed on.
    path: String,
    why: String,
    /// What that one file holds now.
    failed: Failed,
    /// The *other* files the rollback put back as they were, in the order
    /// written. The file the run failed on is never here — it is `failed` —
    /// which keeps this list meaning the files this run wrote in full.
    ///
    /// A path is here only when a read *of that path*, off the tree and after
    /// every restore of the run had run, returned the bytes the file held
    /// before the run. It is a list of files somebody looked at, and not a list
    /// of writes that returned `Ok`.
    restored: Vec<String>,
    /// The other files the rollback could not put back. Empty is the ordinary
    /// case and a reader has to be told when it is not, because this and
    /// `Failed::Damaged` are the states in which the tree is neither what it
    /// was nor what was asked for.
    ///
    /// A path is here when the restore could not be written, **or** when it was
    /// written and the path does not hold it. `why` tells the two apart.
    lost: Vec<Unopened>,
}

impl Halted {
    /// The file the run failed on.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// What that file holds now, for a caller that acts on damage rather than
    /// printing it. Every other fact of this report reaches a reader through
    /// [`std::fmt::Display`].
    pub fn failed(&self) -> &Failed {
        &self.failed
    }

    /// The clause about every file other than the one the run failed on.
    ///
    /// `also` is the word joining them to a failing file that went back, and is
    /// empty when it did not, because *too* would then be false. Nothing
    /// restored and something lost writes no clause at all: the list below
    /// speaks for those files, and *no other file had been written* would be
    /// false of them.
    fn others(&self, f: &mut std::fmt::Formatter<'_>, also: &str) -> std::fmt::Result {
        if self.restored.is_empty() {
            if self.lost.is_empty() {
                return write!(f, ", and no other file had been written");
            }
            return Ok(());
        }
        let count = self.restored.len();
        write!(
            f,
            ", and the {count} file{} already written {} put back{also}",
            match count {
                1 => "",
                _ => "s",
            },
            match count {
                1 => "was",
                _ => "were",
            }
        )
    }
}

/// The report, and the one sentence it may not print falsely.
///
/// **The words *the tree is as it was* appear only when the file the run failed
/// on was put back and `lost` is empty.** Every arm below is keyed on
/// `(failed, restored, lost)` and there is no arm that omits the fate of the
/// failing file, which is what the old two-key match did: it read
/// `(restored.len(), lost.is_empty())` alone, so a run that emptied one
/// document and stopped printed *the tree is as it was* over it — on a
/// single-file run, the ordinary shape of `--fix` over one document, through
/// the arm that also said *no other file had been written*.
///
/// Both of those keys are observations off the tree rather than return values.
/// `failed` is [`Failed::PutBack`] only when a read of that path returned the
/// bytes the file held, and `lost` carries every other file whose restore did
/// not reach the path it names — see [`reads_back`]. The sentence therefore
/// stopped being printable over a file that vanished under its own handle
/// without any arm of this match moving.
impl std::fmt::Display for Halted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} did not write: {}", self.path, self.why)?;
        match &self.failed {
            Failed::PutBack => {
                write!(f, ". It was put back")?;
                self.others(f, " too")?;
            }
            Failed::Damaged { why } => {
                write!(
                    f,
                    ". It could not be put back and now holds neither what it held nor what this \
                     run asked for ({why})"
                )?;
                self.others(f, "")?;
            }
        }
        if self.lost.is_empty() {
            return match &self.failed {
                Failed::PutBack => write!(f, ", so the tree is as it was"),
                Failed::Damaged { .. } => write!(f, ", so this run's damage is that one file"),
            };
        }
        // `lost` is printed as its path and its reason rather than through
        // `Unopened`'s own `Display`, which says *could not be opened for
        // writing* over a file that opened perfectly well: every member of
        // `lost` came out of `Reserved::over` successfully and failed later.
        // The lead-in says the files did not go back rather than what they hold
        // now, because the three ways in are a restore that failed part way, a
        // path that was unlinked, and a path someone else replaced, and only
        // the first of them holds any of this run's bytes.
        write!(
            f,
            ". The tree is now neither what it was nor what was asked for, and these files did \
             not go back:"
        )?;
        for lost in &self.lost {
            write!(f, "\n  {}: {}", lost.path, lost.why)?;
        }
        Ok(())
    }
}

/// The paths, and never the handles or the bytes.
///
/// A reservation holds what every file held before the run, and a derived
/// `Debug` would put a whole corpus into a panic message.
impl std::fmt::Debug for Reserved {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Reserved{:?}", self.paths())
    }
}

impl Reserved {
    /// Open every target, or refuse having opened nothing that matters.
    ///
    /// **Every target must already exist.** The open below carries `.read(true)`
    /// and `.write(true)` and no `.create(true)`, so a path that is missing
    /// fails here exactly as a read-only one does, and a writer whose job is to
    /// create the files it writes cannot use this one. That is the reason
    /// `headwater_resolve::package::publish` copied this module's shape
    /// rather than calling it, and its own comment carries the argument.
    ///
    /// The handles opened before a failing one are dropped on the way out. A
    /// handle that was only opened has changed no byte, so the tree after a
    /// refusal is the tree before the call.
    pub fn over(root: &Path, files: Vec<Composed>) -> Result<Reserved, Unopened> {
        let mut held = Vec::with_capacity(files.len());
        for file in files {
            let at = root.join(&file.path);
            let mut open = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&at)
                .map_err(|error| Unopened {
                    path: file.path.clone(),
                    why: error.to_string(),
                })?;
            let mut was = String::new();
            open.read_to_string(&mut was).map_err(|error| Unopened {
                path: file.path.clone(),
                why: error.to_string(),
            })?;
            held.push(Held {
                path: file.path,
                at,
                file: open,
                was,
                now: file.text,
            });
        }
        Ok(Reserved { held })
    }

    /// What this run will write, for a report that runs without `--apply`.
    pub fn paths(&self) -> Vec<&str> {
        self.held.iter().map(|held| held.path.as_str()).collect()
    }

    /// Write every file, read every file back, and undo the lot on a failure.
    ///
    /// The read back is off the tree rather than out of the handle: what a
    /// caller needs to know is what the next reader of that path will get.
    pub fn commit(self) -> Result<Vec<String>, Halted> {
        self.commit_with(&mut |held: &mut Held, text: &str| -> Result<(), String> {
            put_text(&mut held.file, text)
        })
    }

    /// The write, as a parameter, for the one failure a test cannot cause.
    ///
    /// A `write_all` that fails part way through a file is the failure this
    /// loop's rollback exists for, and it is not provocable from an
    /// unprivileged, deterministic, thread-safe test against a handle
    /// [`Reserved::over`] opened. `over` demands a path that opens `O_RDWR`
    /// *and* that `read_to_string` runs to EOF on, and that pair excludes every
    /// candidate: `/dev/full` gives `ENOSPC` on write but reads endless zeros,
    /// so `over` never returns; a FIFO opened read-write blocks in
    /// `read_to_string` forever; a read-only file, a directory and a missing
    /// path all fail at `over`'s own open, one phase earlier than this loop;
    /// and `RLIMIT_FSIZE` is process-wide while cargo runs a target's cases as
    /// threads of one process, so it would break unrelated cases beside this
    /// one. A fixture built on any of those tests an earlier phase than the
    /// mechanism and reads as if it tested this one.
    ///
    /// So the seam is not a shortcut here, it is the only route. It is placed
    /// on the per-file write and nowhere else: [`Reserved::commit`] is the only
    /// production caller and it passes the real write unconditionally in every
    /// build, `commit_with` is private, and `undo` takes the same
    /// closure so a rollback goes through one write path too. A test closure
    /// therefore inflicts the damage on disk itself rather than reporting that
    /// it did, which is what lets a case observe the state of the file the run
    /// failed on. Do not delete this parameter as test-only scaffolding.
    fn commit_with(
        mut self,
        write: &mut dyn FnMut(&mut Held, &str) -> Result<(), String>,
    ) -> Result<Vec<String>, Halted> {
        for index in 0..self.held.len() {
            let now = self.held[index].now.clone();
            if let Err(why) = write(&mut self.held[index], &now) {
                let path = self.held[index].path.clone();
                return Err(self.undo(index, index, path, why, write));
            }
        }
        for index in 0..self.held.len() {
            let read = std::fs::read_to_string(&self.held[index].at);
            let landed = match read {
                Ok(text) => text == self.held[index].now,
                Err(_) => false,
            };
            if !landed {
                let path = self.held[index].path.clone();
                let every = self.held.len();
                return Err(self.undo(
                    every,
                    index,
                    path,
                    "the bytes read back off the tree are not the bytes this run wrote".to_string(),
                    write,
                ));
            }
        }
        Ok(self.held.iter().map(|held| held.path.clone()).collect())
    }

    /// Put back every file up to `upto`, and say which ones would not go back.
    ///
    /// `failed` is the index the run stopped at, at whichever of the two
    /// failure points it stopped. The loop skips it and the restore after the
    /// loop takes it, so it goes back exactly once and its result lands in
    /// [`Failed`] rather than in `restored`. That keeps `restored` meaning
    /// *every file other than the one the run failed on*, which is the reading
    /// both older cases were written against, and it keeps the one file a
    /// reader most needs to hear about out of a count and in a sentence.
    ///
    /// Restoring it needs nothing that is not already in hand: `Held::was` is
    /// the file's bytes, read in [`Reserved::over`] through the same handle the
    /// write used, and that handle is still open. Nothing is re-read and
    /// nothing can have gone stale. The restore can still fail for the reason
    /// the write did — an `ENOSPC` that stopped `TWO` stops `two` — and that is
    /// the state [`Failed::Damaged`] carries out to the report.
    ///
    /// **A restore that returns `Ok` is not a file that went back.** The write
    /// goes through the handle [`Reserved::over`] opened, and a handle keeps
    /// working after its path is unlinked or replaced, so the bytes can land
    /// where no reader of that path will find them. Every restore is therefore
    /// read back off the tree by [`reads_back`] and downgraded to an error when
    /// the path does not hold them. That is the same mechanism, and the same
    /// argument, as [`Reserved::commit_with`]'s read back of the run's own
    /// writes, and it runs in the same order: restore everything, then read
    /// everything, so a path two files of one run both name is seen in its
    /// final state.
    ///
    /// The cost is one read per file of the run, on a failure path only.
    ///
    /// `failed` indexes `held` at both call sites, which are the only two.
    fn undo(
        mut self,
        upto: usize,
        failed: usize,
        path: String,
        why: String,
        write: &mut dyn FnMut(&mut Held, &str) -> Result<(), String>,
    ) -> Halted {
        let mut done: Vec<(usize, Result<(), String>)> = Vec::new();
        for index in 0..upto.min(self.held.len()) {
            if index == failed {
                continue;
            }
            let was = self.held[index].was.clone();
            let outcome = write(&mut self.held[index], &was);
            done.push((index, outcome));
        }
        let was = self.held[failed].was.clone();
        let mut fate_of_failed = write(&mut self.held[failed], &was);

        for (index, outcome) in &mut done {
            if outcome.is_ok() && !reads_back(&self.held[*index]) {
                *outcome = Err(NOT_READ_BACK.to_string());
            }
        }
        if fate_of_failed.is_ok() && !reads_back(&self.held[failed]) {
            fate_of_failed = Err(NOT_READ_BACK.to_string());
        }

        let mut restored = Vec::new();
        let mut lost = Vec::new();
        for (index, outcome) in done {
            let path = self.held[index].path.clone();
            match outcome {
                Ok(()) => restored.push(path),
                Err(why) => lost.push(Unopened { path, why }),
            }
        }
        let fate = match fate_of_failed {
            Ok(()) => Failed::PutBack,
            Err(why) => Failed::Damaged { why },
        };
        Halted {
            path,
            why,
            failed: fate,
            restored,
            lost,
        }
    }
}

/// Why a restore that returned `Ok` is nevertheless not back.
const NOT_READ_BACK: &str =
    "the bytes read back off the tree are not the bytes this rollback wrote";

/// Does this path hold what the file held before the run?
///
/// The read is off the tree and never through [`Held::file`], which is the only
/// reason a path replaced under the open handle is detectable at all: the write
/// through such a handle returns `Ok` and reaches an inode no reader of that
/// path will ever open. A comparison rather than a test for existence, for the
/// same reason — an unlinked path stops existing, and a replaced one exists and
/// reads. A read error counts as not back, exactly as it does in
/// [`Reserved::commit_with`]'s own read back.
///
/// This is the rollback's only comparison. The loop and the file the run failed
/// on both come through here, so there is no second copy to drift and one
/// change to it moves every case.
///
/// A path replaced by a file that happens to hold exactly [`Held::was`] reads
/// back and is counted as restored. That is this module's own principle rather
/// than a gap in it: what a caller needs to know is what the next reader of
/// that path will get, and the next reader gets the bytes the file held.
fn reads_back(held: &Held) -> bool {
    matches!(std::fs::read_to_string(&held.at), Ok(text) if text == held.was)
}

fn put_text(file: &mut std::fs::File, text: &str) -> Result<(), String> {
    file.rewind().map_err(|error| error.to_string())?;
    file.set_len(0).map_err(|error| error.to_string())?;
    file.write_all(text.as_bytes())
        .map_err(|error| error.to_string())?;
    file.flush().map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A directory that cleans itself up.
    ///
    /// `label` names the case and not the target. Cargo runs the cases of one
    /// target as threads of one process, so a directory keyed on the process
    /// identifier alone is one that a second case removes while the first is
    /// reading it.
    struct Dir(PathBuf);

    impl Dir {
        fn with(label: &str, files: &[(&str, &str)]) -> Dir {
            let root =
                std::env::temp_dir().join(format!("headwater-tree-{}-{label}", std::process::id()));
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
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn composed(files: &[(&str, &str)]) -> Vec<Composed> {
        files
            .iter()
            .map(|(path, text)| Composed {
                path: path.to_string(),
                text: text.to_string(),
            })
            .collect()
    }

    #[test]
    fn every_file_lands_and_reads_back() {
        let dir = Dir::with("lands", &[("a.md", "one"), ("b.md", "two")]);
        let reserved = Reserved::over(dir.path(), composed(&[("a.md", "ONE"), ("b.md", "TWO")]))
            .expect("both open");
        assert_eq!(reserved.commit().expect("both write").len(), 2);
        assert_eq!(dir.read("a.md"), "ONE");
        assert_eq!(dir.read("b.md"), "TWO");
    }

    /// The case this module exists for.
    ///
    /// One target of a two-file run cannot be opened. Nothing is written, and
    /// the file that *could* have been written is the assertion: a loop that
    /// wrote as it went would have written it before it reached the second.
    #[test]
    fn a_target_that_will_not_open_stops_the_run_before_any_file_moves() {
        let dir = Dir::with("will-not-open", &[("a.md", "one")]);
        let refused = Reserved::over(
            dir.path(),
            composed(&[("a.md", "ONE"), ("missing.md", "TWO")]),
        )
        .expect_err("the second target does not open");
        assert_eq!(refused.path, "missing.md");
        assert_eq!(
            dir.read("a.md"),
            "one",
            "the first file was not written, and a loop that wrote as it went would have"
        );
    }

    #[test]
    fn a_read_only_target_is_named_and_nothing_is_written() {
        let dir = Dir::with("read-only", &[("a.md", "one"), ("b.md", "two")]);
        let locked = dir.path().join("b.md");
        let mut permissions = std::fs::metadata(&locked)
            .expect("the file is there")
            .permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(&locked, permissions).expect("the file locks");

        let refused = Reserved::over(dir.path(), composed(&[("a.md", "ONE"), ("b.md", "TWO")]))
            .expect_err("the read-only target does not open");
        assert_eq!(refused.path, "b.md");
        assert_eq!(dir.read("a.md"), "one");
        assert_eq!(dir.read("b.md"), "two");
    }

    /// A commit whose write does not read back off the tree undoes every file.
    ///
    /// The failure is manufactured by removing the *file* under an open handle
    /// after the reservation: the handle still writes, and the read back at the
    /// path finds nothing, which is the second of the two failure points. `a.md`
    /// is the assertion, because it wrote before the run reached the read back.
    ///
    /// `b.md` is the second assertion, and it is the one this case was blind to
    /// for as long as the rollback trusted its own `Ok`. That path is gone, so
    /// the restore through the stale handle lands on an inode with no links and
    /// returns `Ok`; the fate is [`Failed::Damaged`] only because the rollback
    /// reads the path back. The disk truth beside it is context and passes
    /// either way, so it is placed after the fate to keep the failure that
    /// fires attributable to the fate.
    #[test]
    fn a_write_that_does_not_read_back_off_the_tree_undoes_the_run() {
        let dir = Dir::with("read-back", &[("a.md", "one"), ("b.md", "two")]);
        let reserved = Reserved::over(dir.path(), composed(&[("a.md", "ONE"), ("b.md", "TWO")]))
            .expect("both open");
        std::fs::remove_file(dir.path().join("b.md")).expect("the second target goes away");

        let halted = reserved.commit().expect_err("the read back fails");
        assert_eq!(halted.path, "b.md");
        assert!(
            halted.restored.contains(&"a.md".to_string()),
            "the file that wrote is named as put back: {halted}"
        );
        assert!(halted.lost.is_empty(), "{halted}");
        assert_eq!(
            dir.read("a.md"),
            "one",
            "the file that did write was put back"
        );
        assert!(
            matches!(halted.failed(), Failed::Damaged { .. }),
            "the restore of the failing file returned Ok into a path that is gone, so it is \
             damaged rather than put back: {halted}"
        );
        assert!(
            !dir.path().join("b.md").exists(),
            "and the path really is gone, which is the fact the fate above reports"
        );
    }

    /// What a `write_all` that fails part way leaves behind, done for real.
    ///
    /// [`put_text`] rewinds, truncates and then writes, so a failure inside
    /// `write_all` lands on a file that has already been emptied and partly
    /// rewritten. This does the same thing to the same handle and then reports
    /// the failure, so a case built on it observes the true state of the disk
    /// rather than a report that the state exists. A closure that only returned
    /// an error would test the message and not the mechanism.
    ///
    /// The prefix is deliberately non-empty. An assertion that only told "put
    /// back" from "empty" would also pass against a file no run had opened.
    fn fails_part_way(file: &mut std::fs::File, prefix: &str) -> Result<(), String> {
        file.rewind().expect("the handle rewinds");
        file.set_len(0).expect("the handle truncates");
        file.write_all(prefix.as_bytes()).expect("the prefix lands");
        Err("no space left on device".to_string())
    }

    /// The case this issue is about: the file the run failed on comes back.
    ///
    /// `b.md` is emptied and rewritten as far as `TW`, and then the write
    /// fails. Every byte needed to repair it is already in hand, so the run
    /// puts it back through the handle it still holds.
    #[test]
    fn a_write_that_fails_part_way_puts_the_file_it_failed_on_back() {
        let dir = Dir::with("fails-part-way", &[("a.md", "one"), ("b.md", "two")]);
        let reserved = Reserved::over(dir.path(), composed(&[("a.md", "ONE"), ("b.md", "TWO")]))
            .expect("both open");

        let halted = reserved
            .commit_with(&mut |held: &mut Held, text: &str| -> Result<(), String> {
                match (held.path.as_str(), text) {
                    ("b.md", "TWO") => fails_part_way(&mut held.file, "TW"),
                    _ => put_text(&mut held.file, text),
                }
            })
            .expect_err("the second write fails part way");

        assert_eq!(halted.path, "b.md");
        assert_eq!(
            dir.read("b.md"),
            "two",
            "the run emptied this file and rewrote part of it, so a rollback that skips it \
             leaves a prefix of the new text where a document was: {halted}"
        );
        assert_eq!(
            dir.read("a.md"),
            "one",
            "the file that did write is back too"
        );
        assert!(
            halted.to_string().contains("the tree is as it was"),
            "every file is back, so this run is one that may say so: {halted}"
        );
    }

    /// The arm a reader fixing one document actually meets.
    ///
    /// The run fails on its first and only file, so nothing else was written
    /// and `restored` is empty. That is the ordinary shape of `--fix` over one
    /// document, not an edge of it.
    #[test]
    fn a_single_file_run_that_fails_part_way_puts_that_file_back() {
        let dir = Dir::with("single-file", &[("a.md", "one")]);
        let reserved =
            Reserved::over(dir.path(), composed(&[("a.md", "ONE")])).expect("the one target opens");

        let halted = reserved
            .commit_with(&mut |held: &mut Held, text: &str| -> Result<(), String> {
                match text {
                    "ONE" => fails_part_way(&mut held.file, "O"),
                    _ => put_text(&mut held.file, text),
                }
            })
            .expect_err("the only write fails part way");

        assert_eq!(halted.path, "a.md");
        assert_eq!(
            dir.read("a.md"),
            "one",
            "no other file had been written, and this one had: {halted}"
        );
        assert!(
            halted.to_string().contains("the tree is as it was"),
            "the one file is back, so this run is one that may say so: {halted}"
        );
    }

    /// The other half of the ruling: a restore that fails is not an intact tree.
    ///
    /// The closure fails `b.md` twice, and it tells the two apart by the text it
    /// is handed: once writing `TWO`, and again writing `two` back. The second
    /// failure is the state `Failed::Damaged` exists for, and the message must
    /// not claim an intact tree over it.
    #[test]
    fn a_file_that_could_not_be_put_back_is_named_rather_than_called_intact() {
        let dir = Dir::with("not-put-back", &[("a.md", "one"), ("b.md", "two")]);
        let reserved = Reserved::over(dir.path(), composed(&[("a.md", "ONE"), ("b.md", "TWO")]))
            .expect("both open");

        let halted = reserved
            .commit_with(&mut |held: &mut Held, text: &str| -> Result<(), String> {
                match (held.path.as_str(), text) {
                    ("b.md", "TWO") => fails_part_way(&mut held.file, "TW"),
                    ("b.md", "two") => fails_part_way(&mut held.file, "t"),
                    _ => put_text(&mut held.file, text),
                }
            })
            .expect_err("the second write fails part way");

        assert_eq!(halted.path, "b.md");
        assert!(
            !halted.to_string().contains("the tree is as it was"),
            "one file holds neither what it held nor what this run asked for: {halted}"
        );
        assert_eq!(
            halted.failed(),
            &Failed::Damaged {
                why: "no space left on device".to_string()
            },
            "the report carries the reason and not only the fact: {halted}"
        );
        assert_eq!(
            dir.read("b.md"),
            "t",
            "the rollback of the failing file failed in its turn, so the damage is real"
        );
        assert_eq!(dir.read("a.md"), "one", "the file that did write is back");
    }

    /// A file whose path was unlinked under the handle is lost, not restored.
    ///
    /// `a.md` goes away after the reservation, and the run then fails on `b.md`,
    /// so `a.md` is an ordinary member of the rollback loop rather than the file
    /// the run failed on. The restore writes through the handle and returns
    /// `Ok`, and the bytes reach an inode with no links. Nothing in the return
    /// value says so, and a read of the path does.
    ///
    /// The two assertions are one property. Whether the report may still say
    /// the tree is as it was is a different property, held over all eight of its
    /// states by `the_tree_is_as_it_was_is_printed_only_when_it_is`.
    #[test]
    fn a_file_unlinked_under_its_handle_is_lost_rather_than_restored() {
        let dir = Dir::with("unlinked", &[("a.md", "one"), ("b.md", "two")]);
        let reserved = Reserved::over(dir.path(), composed(&[("a.md", "ONE"), ("b.md", "TWO")]))
            .expect("both open");
        std::fs::remove_file(dir.path().join("a.md")).expect("the first target goes away");

        let halted = reserved
            .commit_with(&mut |held: &mut Held, text: &str| -> Result<(), String> {
                match (held.path.as_str(), text) {
                    ("b.md", "TWO") => fails_part_way(&mut held.file, "TW"),
                    _ => put_text(&mut held.file, text),
                }
            })
            .expect_err("the second write fails part way");

        assert!(
            halted.lost.iter().any(|lost| lost.path == "a.md"),
            "the rollback wrote this file back through a handle whose path is gone: {halted}"
        );
        assert!(
            halted.restored.is_empty(),
            "nothing of this run was observed back on the tree: {halted}"
        );
    }

    /// A file whose path was replaced under the handle is lost, not restored.
    ///
    /// The same shape, and the arm that proves the instrument. An unlinked path
    /// stops existing, so a read back written as a test for existence would
    /// catch it. A replaced path exists and reads, holding a third party's
    /// bytes, and only a comparison against what the file held before the run
    /// tells the difference.
    #[test]
    fn a_file_replaced_under_its_handle_is_lost_rather_than_restored() {
        let dir = Dir::with("replaced", &[("a.md", "one"), ("b.md", "two")]);
        let reserved = Reserved::over(dir.path(), composed(&[("a.md", "ONE"), ("b.md", "TWO")]))
            .expect("both open");
        let other = dir.path().join("other");
        std::fs::write(&other, "SOMEONE ELSE").expect("a third party writes its own file");
        std::fs::rename(&other, dir.path().join("a.md")).expect("and moves it over the target");

        let halted = reserved
            .commit_with(&mut |held: &mut Held, text: &str| -> Result<(), String> {
                match (held.path.as_str(), text) {
                    ("b.md", "TWO") => fails_part_way(&mut held.file, "TW"),
                    _ => put_text(&mut held.file, text),
                }
            })
            .expect_err("the second write fails part way");

        assert!(
            halted.lost.iter().any(|lost| lost.path == "a.md"),
            "the rollback wrote this file back through a handle the path no longer names: {halted}"
        );
        assert!(
            halted.restored.is_empty(),
            "nothing of this run was observed back on the tree: {halted}"
        );
        assert_eq!(
            dir.read("a.md"),
            "SOMEONE ELSE",
            "the path exists and reads, so only a comparison of the bytes can tell it apart \
             from a file that went back"
        );
    }

    /// Every arm of the redrawn message, and the sentence none of them may fake.
    ///
    /// The old match was keyed on `(restored.len(), lost.is_empty())` and had
    /// three arms. Restoring the failing file adds a third key, and the arm a
    /// reader met most often — one file, nothing else written — is the one that
    /// changed meaning rather than the one that went away. So the table is
    /// written out here rather than argued about: every state is named, every
    /// state has a location, and the invariant is checked against all of them
    /// rather than against the two a fixture happens to reach.
    ///
    /// This is the one case that builds a `Halted` by hand. It can, because it
    /// is inside the module that owns the fields, and it is the reason nothing
    /// outside can.
    #[test]
    fn the_tree_is_as_it_was_is_printed_only_when_it_is() {
        let damaged = || Failed::Damaged {
            why: "no space left on device".to_string(),
        };
        let unopened = |path: &str| Unopened {
            path: path.to_string(),
            why: "no space left on device".to_string(),
        };
        let halted = |failed: Failed, restored: &[&str], lost: Vec<Unopened>| Halted {
            path: "b.md".to_string(),
            why: "no space left on device".to_string(),
            failed,
            restored: restored.iter().map(|path| path.to_string()).collect(),
            lost,
        };

        let table = [
            (halted(Failed::PutBack, &[], Vec::new()),
             "b.md did not write: no space left on device. It was put back, and no other file had been written, so the tree is as it was"),
            (halted(Failed::PutBack, &["a.md"], Vec::new()),
             "b.md did not write: no space left on device. It was put back, and the 1 file already written was put back too, so the tree is as it was"),
            (halted(Failed::PutBack, &["a.md", "c.md"], Vec::new()),
             "b.md did not write: no space left on device. It was put back, and the 2 files already written were put back too, so the tree is as it was"),
            (halted(damaged(), &[], Vec::new()),
             "b.md did not write: no space left on device. It could not be put back and now holds neither what it held nor what this run asked for (no space left on device), and no other file had been written, so this run's damage is that one file"),
            (halted(damaged(), &["a.md"], Vec::new()),
             "b.md did not write: no space left on device. It could not be put back and now holds neither what it held nor what this run asked for (no space left on device), and the 1 file already written was put back, so this run's damage is that one file"),
            (halted(Failed::PutBack, &["a.md"], vec![unopened("c.md")]),
             "b.md did not write: no space left on device. It was put back, and the 1 file already written was put back too. The tree is now neither what it was nor what was asked for, and these files did not go back:\n  c.md: no space left on device"),
            (halted(Failed::PutBack, &[], vec![unopened("c.md")]),
             "b.md did not write: no space left on device. It was put back. The tree is now neither what it was nor what was asked for, and these files did not go back:\n  c.md: no space left on device"),
            (halted(damaged(), &[], vec![unopened("c.md")]),
             "b.md did not write: no space left on device. It could not be put back and now holds neither what it held nor what this run asked for (no space left on device). The tree is now neither what it was nor what was asked for, and these files did not go back:\n  c.md: no space left on device"),
        ];

        for (halted, expected) in &table {
            assert_eq!(&halted.to_string(), expected);
            let intact = halted.failed() == &Failed::PutBack && halted.lost.is_empty();
            assert_eq!(
                halted.to_string().contains("the tree is as it was"),
                intact,
                "the sentence is printed exactly when the tree is as it was: {halted}"
            );
        }
    }

    #[test]
    fn an_empty_run_commits_nothing_and_says_so() {
        let dir = Dir::with("empty", &[]);
        let reserved = Reserved::over(dir.path(), Vec::new()).expect("nothing to open");
        assert!(reserved.paths().is_empty());
        assert!(reserved.commit().expect("nothing to write").is_empty());
    }
}
