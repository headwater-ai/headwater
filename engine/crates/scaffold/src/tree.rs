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
//! the tree, and compares. A failure at either point restores every file it
//! already wrote from the bytes it read before writing, and says so.
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

/// A run that stopped part way, and what it did about it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Halted {
    /// The file the run failed on.
    pub path: String,
    pub why: String,
    /// The files the rollback put back as they were, in the order written.
    pub restored: Vec<String>,
    /// The files the rollback could not put back. Empty is the ordinary case
    /// and a reader has to be told when it is not, because this is the one
    /// state in which the tree is neither what it was nor what was asked for.
    pub lost: Vec<Unopened>,
}

impl std::fmt::Display for Halted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} did not write: {}", self.path, self.why)?;
        match (self.restored.len(), self.lost.is_empty()) {
            (0, true) => write!(
                f,
                ". No other file had been written, so the tree is as it was"
            ),
            (count, true) => write!(
                f,
                ". The {count} file{} already written {} put back, so the tree is as it was",
                match count {
                    1 => "",
                    _ => "s",
                },
                match count {
                    1 => "was",
                    _ => "were",
                }
            ),
            (_, false) => {
                write!(
                    f,
                    ". The tree is now neither what it was nor what was asked for, and these \
                     files hold this run's write with no other file beside them:"
                )?;
                for lost in &self.lost {
                    write!(f, "\n  {lost}")?;
                }
                Ok(())
            }
        }
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
        self.commit_with(
            &mut |held: &mut Held, text: &str| -> Result<(), String> {
                put_text(&mut held.file, text)
            },
        )
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
    /// build, `commit_with` is private, and [`Reserved::undo`] takes the same
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
                return Err(self.undo(index, path, why, write));
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
                    path,
                    "the bytes read back off the tree are not the bytes this run wrote".to_string(),
                    write,
                ));
            }
        }
        Ok(self.held.iter().map(|held| held.path.clone()).collect())
    }

    /// Put back every file up to `upto`, and say which ones would not go back.
    fn undo(
        mut self,
        upto: usize,
        path: String,
        why: String,
        write: &mut dyn FnMut(&mut Held, &str) -> Result<(), String>,
    ) -> Halted {
        let mut restored = Vec::new();
        let mut lost = Vec::new();
        for index in 0..upto.min(self.held.len()) {
            let was = self.held[index].was.clone();
            let held = &mut self.held[index];
            match write(held, &was) {
                Ok(()) => restored.push(held.path.clone()),
                Err(error) => lost.push(Unopened {
                    path: held.path.clone(),
                    why: error,
                }),
            }
        }
        Halted {
            path,
            why,
            restored,
            lost,
        }
    }
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
            .commit_with(
                &mut |held: &mut Held, text: &str| -> Result<(), String> {
                    match (held.path.as_str(), text) {
                        ("b.md", "TWO") => fails_part_way(&mut held.file, "TW"),
                        _ => put_text(&mut held.file, text),
                    }
                },
            )
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
            .commit_with(
                &mut |held: &mut Held, text: &str| -> Result<(), String> {
                    match text {
                        "ONE" => fails_part_way(&mut held.file, "O"),
                        _ => put_text(&mut held.file, text),
                    }
                },
            )
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
            .commit_with(
                &mut |held: &mut Held, text: &str| -> Result<(), String> {
                    match (held.path.as_str(), text) {
                        ("b.md", "TWO") => fails_part_way(&mut held.file, "TW"),
                        ("b.md", "two") => fails_part_way(&mut held.file, "t"),
                        _ => put_text(&mut held.file, text),
                    }
                },
            )
            .expect_err("the second write fails part way");

        assert_eq!(halted.path, "b.md");
        assert!(
            !halted.to_string().contains("the tree is as it was"),
            "one file holds neither what it held nor what this run asked for: {halted}"
        );
        assert_eq!(
            dir.read("b.md"),
            "t",
            "the rollback of the failing file failed in its turn, so the damage is real"
        );
        assert_eq!(
            dir.read("a.md"),
            "one",
            "the file that did write is back"
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
