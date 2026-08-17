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
//! # The one file a run may create, and why its undo is an unlink
//!
//! [`Reserved::over`] cannot open a file that is not there, so a writer that
//! makes a new document needs a second step: [`Reserved::making`]. The order is
//! the whole of its safety. `over` runs first and changes no byte, so the state
//! it refuses in — a target that cannot be opened for writing, which is the
//! failure that actually happens — has nothing to undo. `making` runs only
//! after every one of those obstacles is past, and the file it makes is the
//! only thing a rollback here can have to remove.
//!
//! Undoing a create is a delete, and a delete of the wrong file is worse than
//! the mess it cleans up. Two rules keep it honest. The create is
//! `create_new`, so `Ok` is the kernel saying *this call made this file*: the
//! test and the create are one syscall, there is no window between them, and no
//! error is ever read as absence. That is the lesson of the refused `publish`
//! change, whose `Err(_) => Found::Absent` put `remove_dir_all` behind every
//! failure it could not read. And the removal is `remove_file` followed by a
//! `symlink_metadata` of that path, so [`Halted`] says the document went only
//! when a reader of the path would find nothing. `remove_dir_all` appears
//! nowhere on this path; the directories the create made go with `remove_dir`,
//! which refuses a directory somebody else filled, and that refusal is the
//! answer a race is owed.
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
    /// The one file this run made, when [`Reserved::making`] made one.
    made: Option<Made>,
}

/// The file this run created, and the directories it created above it.
///
/// The type and its fields are private and [`Reserved::making`] is its only
/// constructor, so nothing outside this module can hand a rollback a claim that
/// a file was created. The claim licenses an unlink, and the only thing allowed
/// to make it is the `create_new` that returned `Ok`.
struct Made {
    /// Where in `held` the created file sits. `making` pushes it last and
    /// nothing removes an entry, so it stays the entry it named.
    index: usize,
    path: String,
    at: PathBuf,
    /// The directories that were not on the tree before the create, deepest
    /// first. Observed before `create_dir_all` ran, because afterwards there is
    /// nothing left to see.
    dirs: Vec<PathBuf>,
}

/// What became of the file the run created.
///
/// Both arms are observations off the tree. [`Created::Removed`] is written
/// only when a `symlink_metadata` of that path reported nothing there, for the
/// same reason a restore is read back: an unlink that returns `Ok` is a return
/// value, and what a caller needs to know is what the next reader of the path
/// will get.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Created {
    /// Unlinked, and the path holds nothing.
    Removed { path: String },
    /// Still there. `why` is the unlink's own error, or the read back that
    /// found something at the path after it.
    NotRemoved { path: String, why: String },
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
    /// The file the run failed on is the file this run created, so it has no
    /// bytes to be put back to.
    ///
    /// Its fate is an unlink and it is reported in one place, the created
    /// clause of [`Halted`], rather than told twice in two idioms. This arm
    /// exists so that clause is the only thing that speaks for it: *it was put
    /// back* over a file that never existed would be the same class of untrue
    /// sentence this module was built to stop printing.
    Unmade,
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
    /// What became of the file this run created, when it created one.
    ///
    /// `None` is a run that created nothing, which is every `--fix` run and
    /// every import, and it prints no clause at all. That is why every message
    /// this type wrote before the create existed is the message it writes now.
    ///
    /// Behind a `Box` so that the ordinary case — a run that finished — carries
    /// eight bytes of this and not fifty-six. `commit` returns this type in its
    /// `Err`, and `clippy::result_large_err` reads that size on every call.
    created: Option<Box<Created>>,
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

    /// The clause about the document this run created, and nothing when it
    /// created none.
    ///
    /// A run that created nothing writes no clause here at all, which is why
    /// every message this type printed before the create exists is the message
    /// it prints now. The path is named once: when the run failed on the very
    /// document it created, the clause before this one already named it, and
    /// naming it twice would read as two files.
    fn made(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Some(created) = self.created.as_deref() else {
            return Ok(());
        };
        let named = |path: &String| match path == &self.path {
            true => "it".to_string(),
            false => format!("the document this run created at {path}"),
        };
        match created {
            Created::Removed { path } => write!(f, ", and {} was removed", named(path)),
            Created::NotRemoved { path, why } => {
                write!(f, ", and {} could not be removed: {why}", named(path))
            }
        }
    }

    /// How many files of this run hold neither what they held nor what was
    /// asked for.
    ///
    /// The failing file is one when it could not be put back, and the created
    /// document is one when it would not go. Both are observations off the
    /// tree, made in [`Reserved::undo`], and this is the only place that counts
    /// them — so the sentence *the tree is as it was* is keyed on the same
    /// facts the clauses above it printed.
    fn damaged(&self) -> usize {
        usize::from(matches!(self.failed, Failed::Damaged { .. }))
            + usize::from(matches!(
                self.created.as_deref(),
                Some(Created::NotRemoved { .. })
            ))
    }
}

/// The report, and the one sentence it may not print falsely.
///
/// **The words *the tree is as it was* appear only when the file the run failed
/// on was put back, `lost` is empty, and the document this run created is off
/// the tree.** Every arm below is keyed on `(failed, restored, lost, created)`
/// and there is no arm that omits the fate of the
/// failing file, which is what the old two-key match did: it read
/// `(restored.len(), lost.is_empty())` alone, so a run that emptied one
/// document and stopped printed *the tree is as it was* over it — on a
/// single-file run, the ordinary shape of `--fix` over one document, through
/// the arm that also said *no other file had been written*.
///
/// All three of those keys are observations off the tree rather than return
/// values. `failed` is [`Failed::PutBack`] only when a read of that path
/// returned the bytes the file held, `lost` carries every other file whose
/// restore did not reach the path it names — see [`reads_back`] — and `created`
/// is [`Created::Removed`] only when a `symlink_metadata` of that path found
/// nothing. The sentence therefore stopped being printable over a file that
/// vanished under its own handle without any arm of this match moving, and it
/// is not printable over a document this run made and could not unlink.
impl std::fmt::Display for Halted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} did not write: {}", self.path, self.why)?;
        match &self.failed {
            Failed::PutBack => write!(f, ". It was put back")?,
            Failed::Damaged { why } => write!(
                f,
                ". It could not be put back and now holds neither what it held nor what this run \
                 asked for ({why})"
            )?,
            Failed::Unmade => write!(f, ". It is the document this run created")?,
        }
        self.made(f)?;
        // *too* joins the other files to a failing file that went back, and is
        // false of a file that did not go back and of one that never existed.
        self.others(
            f,
            match &self.failed {
                Failed::PutBack => " too",
                Failed::Damaged { .. } | Failed::Unmade => "",
            },
        )?;
        if self.lost.is_empty() {
            return match self.damaged() {
                0 => write!(f, ", so the tree is as it was"),
                1 => write!(f, ", so this run's damage is that one file"),
                _ => write!(f, ", so this run's damage is those two files"),
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
    /// fails here exactly as a read-only one does. A writer that also makes a
    /// new file calls [`Reserved::making`] after this, and never instead of it:
    /// this call changes no byte, so putting it first is what leaves the
    /// ordinary failure with nothing to undo.
    /// `headwater_resolve::package::publish` copied this module's shape rather
    /// than calling it, and its own comment carries the argument.
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
        Ok(Reserved { held, made: None })
    }

    /// Create the one file this run makes, after every file it will overwrite
    /// is already held open.
    ///
    /// **The order is the guarantee.** [`Reserved::over`] changes no byte, so
    /// its refusal needs no undo, and the failure that actually happens — a
    /// reciprocal end nobody can write — is refused there with the tree exactly
    /// as it was. By the time this runs, every foreseeable obstacle is past and
    /// the only thing a rollback can have to remove is the file below.
    ///
    /// **`create_new` is what licenses the removal.** `Ok` means the kernel
    /// made this file for this call: the test for an existing file and the
    /// create are one syscall, so there is no window between them and no error
    /// is read as absence. `AlreadyExists` is a refusal like every other error,
    /// which is the shape a `publish` change was refused for missing when it
    /// read every `Err` as *nothing was there* and unlinked on the strength of
    /// it.
    ///
    /// A caller may not pass its own belief that a file is new. The path is an
    /// argument, the create is the answer, and a `Composed::created` set three
    /// functions upstream never reaches this decision.
    pub fn making(mut self, root: &Path, path: &str, text: String) -> Result<Reserved, Unopened> {
        let at = root.join(path);
        let refuse = |why: String| Unopened {
            path: path.to_string(),
            why,
        };
        // Observed before `create_dir_all` runs, because afterwards there is
        // nothing left to see.
        let dirs = match at.parent() {
            Some(parent) => {
                let dirs = absent_above(parent);
                if let Err(error) = std::fs::create_dir_all(parent) {
                    unmake_dirs(&dirs);
                    return Err(refuse(error.to_string()));
                }
                dirs
            }
            None => Vec::new(),
        };
        let file = match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&at)
        {
            Ok(file) => file,
            Err(error) => {
                unmake_dirs(&dirs);
                return Err(refuse(error.to_string()));
            }
        };
        let index = self.held.len();
        self.held.push(Held {
            path: path.to_string(),
            at: at.clone(),
            file,
            // A file that did not exist has no bytes to be put back to, which is
            // why the undo below is an unlink and never a write of this field.
            was: String::new(),
            now: text,
        });
        self.made = Some(Made {
            index,
            path: path.to_string(),
            at,
            dirs,
        });
        Ok(self)
    }

    /// What this run will write, for a report that runs without `--apply`.
    pub fn paths(&self) -> Vec<&str> {
        self.held.iter().map(|held| held.path.as_str()).collect()
    }

    /// Write every file, read every file back, and undo the lot on a failure.
    ///
    /// The read back is off the tree rather than out of the handle: what a
    /// caller needs to know is what the next reader of that path will get.
    /// The report is boxed because it is a failure path and a wide type: it
    /// carries four observations off the tree, and `clippy::result_large_err`
    /// reads that width on every call of this function rather than on the runs
    /// that stop.
    pub fn commit(self) -> Result<Vec<String>, Box<Halted>> {
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
    ) -> Result<Vec<String>, Box<Halted>> {
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
    ///
    /// **The file this run created is never written back.** It has no bytes to
    /// be put back to, so it is skipped by the loop and by the restore after it,
    /// and it leaves through [`Made::unmake`] instead. It is unlinked whatever
    /// `upto` says: a run that failed before reaching it created it all the
    /// same, and an empty document declaring one half of an edge is the state
    /// this whole change exists to stop leaving behind.
    fn undo(
        mut self,
        upto: usize,
        failed: usize,
        path: String,
        why: String,
        write: &mut dyn FnMut(&mut Held, &str) -> Result<(), String>,
    ) -> Box<Halted> {
        let made = self.made.take();
        let created_index = made.as_ref().map(|made| made.index);

        let mut done: Vec<(usize, Result<(), String>)> = Vec::new();
        for index in 0..upto.min(self.held.len()) {
            if index == failed || Some(index) == created_index {
                continue;
            }
            let was = self.held[index].was.clone();
            let outcome = write(&mut self.held[index], &was);
            done.push((index, outcome));
        }
        let mut fate_of_failed = match Some(failed) == created_index {
            true => None,
            false => {
                let was = self.held[failed].was.clone();
                Some(write(&mut self.held[failed], &was))
            }
        };

        for (index, outcome) in &mut done {
            if outcome.is_ok() && !reads_back(&self.held[*index]) {
                *outcome = Err(NOT_READ_BACK.to_string());
            }
        }
        if let Some(fate) = &mut fate_of_failed {
            if fate.is_ok() && !reads_back(&self.held[failed]) {
                *fate = Err(NOT_READ_BACK.to_string());
            }
        }

        // The handle goes before the unlink, so that nothing of this run is
        // still holding the file open when the path is read back. `making`
        // pushes last and nothing removes an entry, so this truncation drops
        // that one entry and leaves every index `done` names where it was.
        let created = match made {
            None => None,
            Some(made) => {
                self.held.truncate(made.index);
                Some(Box::new(made.unmake()))
            }
        };

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
            None => Failed::Unmade,
            Some(Ok(())) => Failed::PutBack,
            Some(Err(why)) => Failed::Damaged { why },
        };
        Box::new(Halted {
            path,
            why,
            failed: fate,
            restored,
            lost,
            created,
        })
    }
}

impl Made {
    /// Unlink the file, and read the path back before saying it went.
    ///
    /// `remove_file` returning `Ok` is a return value, exactly as a restore's
    /// `Ok` was, and the question is what the next reader of the path gets. So
    /// the answer comes from `symlink_metadata`, which does not follow a link
    /// and does not swallow an error the way `Path::exists` does: only
    /// [`std::io::ErrorKind::NotFound`] licenses [`Created::Removed`].
    ///
    /// A `NotFound` out of the unlink itself is not a failure. Somebody else
    /// removed the path, which is the state this call wanted, and the read back
    /// below is what decides either way.
    ///
    /// The directories go only after the file is observed gone, with
    /// `remove_dir`, which takes an empty directory and nothing else. A
    /// directory somebody filled while this run was writing therefore stops the
    /// climb by refusing to go. **`remove_dir_all` appears nowhere here**, for
    /// the reason `publish`'s own unwind gives: it is the call that turns a
    /// misread state into somebody else's lost work.
    fn unmake(self) -> Created {
        if let Err(error) = std::fs::remove_file(&self.at) {
            if error.kind() != std::io::ErrorKind::NotFound {
                return Created::NotRemoved {
                    path: self.path,
                    why: error.to_string(),
                };
            }
        }
        match std::fs::symlink_metadata(&self.at) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                unmake_dirs(&self.dirs);
                Created::Removed { path: self.path }
            }
            Ok(_) => Created::NotRemoved {
                path: self.path,
                why: "the unlink returned no error and the path still holds a file".to_string(),
            },
            Err(error) => Created::NotRemoved {
                path: self.path,
                why: error.to_string(),
            },
        }
    }
}

/// `dir` and every directory above it that is not on the tree either, deepest
/// first.
///
/// The walk stops at the first path that exists, and `symlink_metadata` is what
/// asks, so a link above `dir` stops it rather than being read through. This is
/// the shape `headwater_resolve::package`'s own climb reached, and the argument
/// there is the argument here.
fn absent_above(dir: &Path) -> Vec<PathBuf> {
    let mut chain = Vec::new();
    let mut at = Some(dir);
    while let Some(path) = at {
        if path.as_os_str().is_empty() {
            break;
        }
        match std::fs::symlink_metadata(path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                chain.push(path.to_path_buf());
                at = path.parent();
            }
            _ => break,
        }
    }
    chain
}

/// Take back the directories a create made, deepest first, and stop at the
/// first one that will not go.
///
/// Every error is dropped. This runs on the way out of a failure that is
/// already being reported, and the fact a reader needs — whether the *document*
/// is gone — is the one [`Made::unmake`] observed and [`Halted`] prints. An
/// empty directory is no document, no rule reads one, and nothing about a
/// corpus's checks turns on it.
fn unmake_dirs(dirs: &[PathBuf]) {
    for dir in dirs {
        if std::fs::remove_dir(dir).is_err() {
            return;
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
            created: None,
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
            let intact = halted.damaged() == 0 && halted.lost.is_empty();
            assert_eq!(
                halted.to_string().contains("the tree is as it was"),
                intact,
                "the sentence is printed exactly when the tree is as it was: {halted}"
            );
        }
    }

    /// Every state the created document adds, and the sentence none of them may
    /// fake.
    ///
    /// A run that created nothing carries `created: None` and prints no clause
    /// at all, which is why every string the table above records is the string
    /// this type still writes. The states below are the ones a create adds: the
    /// document went or it did not, and the run failed on it or on something
    /// else. The invariant is the same one, read through [`Halted::damaged`],
    /// and it is checked against all of them rather than against the two a
    /// fixture happens to reach.
    #[test]
    fn a_created_document_that_would_not_go_is_never_called_an_intact_tree() {
        let removed = || {
            Some(Box::new(Created::Removed {
                path: "new.md".to_string(),
            }))
        };
        let stuck = |path: &str| {
            Some(Box::new(Created::NotRemoved {
                path: path.to_string(),
                why: "permission denied".to_string(),
            }))
        };
        let halted = |failed: Failed, path: &str, created: Option<Box<Created>>| Halted {
            path: path.to_string(),
            why: "no space left on device".to_string(),
            failed,
            restored: vec!["a.md".to_string()],
            lost: Vec::new(),
            created,
        };

        let table = [
            (halted(Failed::PutBack, "b.md", removed()),
             "b.md did not write: no space left on device. It was put back, and the document this run created at new.md was removed, and the 1 file already written was put back too, so the tree is as it was"),
            (halted(Failed::PutBack, "b.md", stuck("new.md")),
             "b.md did not write: no space left on device. It was put back, and the document this run created at new.md could not be removed: permission denied, and the 1 file already written was put back too, so this run's damage is that one file"),
            (halted(Failed::Damaged { why: "no space left on device".to_string() }, "b.md", stuck("new.md")),
             "b.md did not write: no space left on device. It could not be put back and now holds neither what it held nor what this run asked for (no space left on device), and the document this run created at new.md could not be removed: permission denied, and the 1 file already written was put back, so this run's damage is those two files"),
            // The run failed on the document it created. The clause about the
            // failing file already named the path, so the created clause says
            // *it* rather than naming a second file that is the same file.
            (halted(Failed::Unmade, "new.md", removed()),
             "new.md did not write: no space left on device. It is the document this run created, and it was removed, and the 1 file already written was put back, so the tree is as it was"),
            (halted(Failed::Unmade, "new.md", stuck("new.md")),
             "new.md did not write: no space left on device. It is the document this run created, and it could not be removed: permission denied, and the 1 file already written was put back, so this run's damage is that one file"),
        ];

        for (halted, expected) in &table {
            assert_eq!(&halted.to_string(), expected);
            let intact = halted.damaged() == 0 && halted.lost.is_empty();
            assert_eq!(
                halted.to_string().contains("the tree is as it was"),
                intact,
                "the sentence is printed exactly when the tree is as it was: {halted}"
            );
        }
    }

    /// The create lands, and so does the directory it needed.
    ///
    /// `making` is the only way a shelf's first document reaches the tree, so
    /// the absent parent is the ordinary case rather than an edge of it.
    #[test]
    fn a_created_document_lands_under_a_directory_that_was_not_there() {
        let dir = Dir::with("making", &[("a.md", "one")]);
        let reserved = Reserved::over(dir.path(), composed(&[("a.md", "ONE")]))
            .expect("the one target opens")
            .making(dir.path(), "shelf/new.md", "NEW".to_string())
            .expect("the new document is created");

        assert_eq!(reserved.paths(), vec!["a.md", "shelf/new.md"]);
        assert_eq!(reserved.commit().expect("both write").len(), 2);
        assert_eq!(dir.read("a.md"), "ONE");
        assert_eq!(dir.read("shelf/new.md"), "NEW");
    }

    /// A path that is already taken refuses, and takes its directory back.
    ///
    /// `create_new` is what answers, and its `AlreadyExists` is a refusal like
    /// every other error rather than a state to reason about. The second
    /// assertion is the one about the undo of the refusal itself: the directory
    /// this call made on the way to a create that did not happen does not stay.
    #[test]
    fn a_created_document_whose_path_is_taken_refuses_and_leaves_no_directory() {
        let dir = Dir::with("already-there", &[("a.md", "one")]);
        std::fs::create_dir_all(dir.path().join("shelf")).expect("a shelf");
        std::fs::write(dir.path().join("shelf/new.md"), "SOMEBODY ELSE").expect("a file there");

        let refused = Reserved::over(dir.path(), composed(&[("a.md", "ONE")]))
            .expect("the one target opens")
            .making(dir.path(), "shelf/new.md", "NEW".to_string())
            .expect_err("the path is taken");
        assert_eq!(refused.path, "shelf/new.md");
        assert_eq!(
            dir.read("shelf/new.md"),
            "SOMEBODY ELSE",
            "the file at the path is somebody else's and this call never opened it for writing"
        );
        assert_eq!(dir.read("a.md"), "one", "and nothing else moved");
        assert!(
            dir.path().join("shelf").exists(),
            "a directory this call did not make is not one it may remove"
        );
    }

    /// The test the ruling turns on: a run that stops leaves no new document.
    ///
    /// The reciprocal's write fails part way, which is one write before the
    /// created document's own. So at the moment of the failure `shelf/new.md`
    /// is on disk and empty — a document declaring one half of an edge, which
    /// is the exact state `relation.reciprocity.missing` reports over the
    /// author's whole corpus. A loop of `std::fs::write` leaves it there, and
    /// leaves the shelf directory it made for it. This asserts both are gone.
    #[test]
    fn a_commit_that_fails_removes_the_document_this_run_created() {
        let dir = Dir::with("commit-fails", &[("a.md", "one")]);
        let reserved = Reserved::over(dir.path(), composed(&[("a.md", "ONE")]))
            .expect("the one target opens")
            .making(dir.path(), "shelf/new.md", "NEW".to_string())
            .expect("the new document is created");
        assert!(
            dir.path().join("shelf/new.md").exists(),
            "the create really did make a file, which is what the rollback below has to remove"
        );

        let halted = reserved
            .commit_with(&mut |held: &mut Held, text: &str| -> Result<(), String> {
                match (held.path.as_str(), text) {
                    ("a.md", "ONE") => fails_part_way(&mut held.file, "O"),
                    _ => put_text(&mut held.file, text),
                }
            })
            .expect_err("the first write fails part way");

        assert!(
            !dir.path().join("shelf/new.md").exists(),
            "the document this run created is off the tree: {halted}"
        );
        assert!(
            !dir.path().join("shelf").exists(),
            "and so is the directory the create made for it: {halted}"
        );
        assert_eq!(dir.read("a.md"), "one", "and the file it edited is back");
        assert!(
            halted
                .to_string()
                .contains("the document this run created at shelf/new.md was removed"),
            "and the report says so rather than leaving the author to look: {halted}"
        );
        assert!(
            halted.to_string().contains("the tree is as it was"),
            "everything went back and the new document went, so this run may say so: {halted}"
        );
    }

    /// The same property one phase later, and with no seam at all.
    ///
    /// `a.md` is unlinked under its handle after the reservation, so every write
    /// of the run returns `Ok` and the read back off the tree is what fails.
    /// By then `new.md` holds this run's bytes rather than nothing, so *absent*
    /// is the opposite of what a writer without a rollback leaves in both of
    /// this file's create cases and not only in the empty one.
    #[test]
    fn a_read_back_that_fails_removes_the_document_this_run_created() {
        let dir = Dir::with("read-back-create", &[("a.md", "one")]);
        let reserved = Reserved::over(dir.path(), composed(&[("a.md", "ONE")]))
            .expect("the one target opens")
            .making(dir.path(), "new.md", "NEW".to_string())
            .expect("the new document is created");
        std::fs::remove_file(dir.path().join("a.md")).expect("the edited target goes away");

        let halted = reserved.commit().expect_err("the read back fails");

        assert_eq!(halted.path, "a.md");
        assert!(
            !dir.path().join("new.md").exists(),
            "the document this run created is off the tree: {halted}"
        );
    }

    /// A mode, set on a path.
    #[cfg(unix)]
    fn mode(at: &Path, bits: u32) {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(at, std::fs::Permissions::from_mode(bits))
            .expect("the mode is set");
    }

    /// Whether a mode can stop this process.
    ///
    /// Root unlinks through every bit, so under root the case below is
    /// unreachable rather than failing, and it says so and stops. The answer is
    /// a probe rather than a user id, because the question is what the file
    /// system does to this process.
    #[cfg(unix)]
    fn modes_hold(dir: &Dir) -> bool {
        let at = dir.path().join("mode-probe");
        std::fs::create_dir_all(&at).expect("the probe is made");
        std::fs::write(at.join("held"), "x").expect("the probe holds a file");
        mode(&at, 0o555);
        let held = std::fs::remove_file(at.join("held")).is_err();
        mode(&at, 0o755);
        std::fs::remove_dir_all(&at).expect("the probe goes");
        held
    }

    /// An unlink that will not go is named, and the report may not say the tree
    /// is as it was.
    ///
    /// This is the terminal arm of the ruling. The rollback did everything it
    /// could and one file of this run is still there, so the sentence that says
    /// otherwise is the one thing the report is not allowed to print.
    ///
    /// Mode `0555` is what makes both halves reachable at once: no `w`, so no
    /// name can leave the directory, and `r` and `x`, so every read the
    /// rollback makes still answers and the other file goes back as it would
    /// have.
    #[cfg(unix)]
    #[test]
    fn a_created_file_that_will_not_go_is_named_rather_than_claimed_removed() {
        let dir = Dir::with("will-not-go", &[("a.md", "one")]);
        if !modes_hold(&dir) {
            eprintln!("skipped: this process is root, and root unlinks through mode 0555");
            return;
        }
        let reserved = Reserved::over(dir.path(), composed(&[("a.md", "ONE")]))
            .expect("the one target opens")
            .making(dir.path(), "new.md", "NEW".to_string())
            .expect("the new document is created");

        let root = dir.path().to_path_buf();
        let halted = reserved
            .commit_with(&mut |held: &mut Held, text: &str| -> Result<(), String> {
                match (held.path.as_str(), text) {
                    ("a.md", "ONE") => {
                        mode(&root, 0o555);
                        fails_part_way(&mut held.file, "O")
                    }
                    _ => put_text(&mut held.file, text),
                }
            })
            .expect_err("the first write fails part way");

        mode(&root, 0o755);

        assert!(
            dir.path().join("new.md").exists(),
            "the unlink could not go, which is the state this case is about"
        );
        assert!(
            halted
                .to_string()
                .contains("the document this run created at new.md could not be removed"),
            "the file that is still there is named: {halted}"
        );
        assert!(
            !halted.to_string().contains("the tree is as it was"),
            "one file of this run is still on the tree: {halted}"
        );
    }

    #[test]
    fn an_empty_run_commits_nothing_and_says_so() {
        let dir = Dir::with("empty", &[]);
        let reserved = Reserved::over(dir.path(), Vec::new()).expect("nothing to open");
        assert!(reserved.paths().is_empty());
        assert!(reserved.commit().expect("nothing to write").is_empty());
    }
}
