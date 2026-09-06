// SPDX-License-Identifier: Apache-2.0
//! What `headwater init` writes, byte for byte.
//!
//! # The defect this target exists for
//!
//! `headwater init` writes two files, and until this target no test of this
//! workspace read one byte of either. One `match` arm of the verb emits two
//! messages under one condition: the comment it writes above `version: 0.0.0`,
//! and the line it prints in its report. #276 found those two drifted apart —
//! the printed one naming two routes and the written one naming a route an
//! adopter cannot take — because the printed one was held and the written one
//! was held by nothing. That arm is still one arm.
//!
//! `.claude/tutorial/drive.py` does hold part of it: three substrings of the
//! declaration, the literal `  version: 0.0.0` line, and the fact that the
//! overlay ends `add: {}`. That guard is real and it sits in the wrong place
//! twice over. It is six claims rather than the file, and its failure message
//! is about a sixteen-step tutorial rather than about `init`. Its own comment
//! named #174 as the missing test's home, and #174 is closed.
//!
//! This target is a characterization of the verb rather than a statement of
//! what the verb should write. It passed on the day it was written, against an
//! unmodified `init`. Its value is that the bytes now have a reader.
//!
//! # Why the two arms are asserted differently
//!
//! The blank-tree arm is asserted as bytes, because every byte of it is a
//! literal in the verb.
//!
//! The package-present arm embeds the version the package declares, so this
//! target reads that version out of the manifest it copied rather than writing
//! it here. A literal would be the defect #427 already caused once: a bump to
//! 4.0.0 moved six mentions in the tutorial, left one behind, and failed every
//! step from 5 to 16 of a tutorial that was right.
//!
//! # What the last line is for
//!
//! `add: {}` is the last line of the overlay, and `.claude/tutorial/drive.py`
//! splices the reader's edit in at exactly that offset. Holding it here makes
//! a break point at `init`, which is where it belongs.
//!
//! # One oddity this target pins rather than repairs
//!
//! The declaration carries one comment at column zero — `# A bundle is an
//! optional part of the package` — where every comment around it carries two
//! spaces. A `\` line continuation in the verb's format string eats the
//! leading whitespace of the line that follows it. YAML reads a comment
//! wherever it sits, so nothing downstream is wrong, and the adopter commits
//! the file all the same. It is pinned here rather than repaired, because this
//! target exists to record what the verb writes today.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository this test tree sits in.
fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// Everything the declaration carries before the version block.
const DECLARATION_HEAD: &str = r"# The consumer declaration, written by `headwater init`. It says two things,
# and they are different questions: what schema this repository takes, and what
# tree it walks.
#
# `headwater taxonomy resolve` reads this and writes .headwater/taxonomy.lock. Everything after that
# reads the lock and never these sources.

taxonomy:
  package: headwater/standard
";

/// The version block of the arm that found no package, which is an interview
/// stub and four lines of comment.
const NO_PACKAGE_VERSION: &str = r"  # INTERVIEW: no package of this name is under `packages/`, and nothing in this
  # engine fetches one. Copy a package directory into `packages/`, or run
  # `headwater taxonomy vendor <dir>` on a published artifact. Either way, pin
  # the version that the package itself declares.
  version: 0.0.0
";

/// Everything the declaration carries after the version block.
///
/// Its first line is the comment at column zero that the module header names.
const DECLARATION_TAIL: &str = r"# A bundle is an optional part of the package, and a selection is add-only.
  # INTERVIEW: which traditions does this corpus already follow?
  bundles: []
  overlay: .headwater/overlay.yml

corpus:
  # Proposed from this tree: the directory holding the most Markdown.
  root: docs
  # An exclusion states a reason. A pattern with none is a silent pass with a
  # configuration file in front of it, so the reason is not optional.
  # exclude:
  #   - path: docs/vendor/**
  #     reason: vendored copies of documents another team owns
";

/// The whole overlay. It is one arm, because `init` writes the same overlay
/// whether a package is on the tree or not.
const OVERLAY: &str = r#"# The adopter overlay, written by `headwater init`. It is an overlay and never a
# resolved taxonomy, so nothing here can weaken the package it sits on: a
# bundle selection is add-only, and an add-only overlay carries no operation
# that removes a base rule.
#
# Every block below is a question this engine cannot answer from a tree. It is
# prose about what this corpus is for, and a corpus does not state it.
#
# INTERVIEW 1 --- what does each purpose answer?
#
# A task is matched against declared purposes before it is matched against any
# text, and it is matched on the `answers` phrases first. Two purposes whose
# phrases share every term separate nothing, and every task then matches both
# equally. Read `headwater/standard`'s purposes, and add the phrases a person here would
# actually type.
#
#   add:
#     purposes.rationale.answers: ["why is it this way", "what was rejected"]
#
# INTERVIEW 2 --- what identifies a document, and what does the prefix mean?
#
# A relation names its target by identifier. A corpus whose documents carry none
# has no edges, and no check about an edge can say anything about it.
#
#   add:
#     identifier_schemes.doc_id: {pattern: "{namespace}-DOC-{slug}", namespace: ACME, allocation: minted-once}
#     kinds.<kind>.identifier: {scheme: doc_id}
#
# INTERVIEW 3 --- what does this corpus already write?
#
# Run `headwater infer` once this file resolves. It reports the files that
# classify as nothing, which is the half a payload cannot carry, and the
# documents that state no summary, which nothing will route to.

add: {}
"#;

/// A scratch repository, and what one run of `init` leaves in it.
struct Root {
    at: PathBuf,
}

impl Root {
    /// A tree holding one Markdown file under `docs/`, which is the least that
    /// reaches the write path: `busiest_directory` proposes a corpus root from
    /// the directory holding the most Markdown, and a tree with none is
    /// refused.
    ///
    /// `label` names the case rather than the process, for the reason
    /// `wiring.rs` records beside its own helper. Cargo runs the cases of one
    /// target as threads of one process, so a directory keyed on the process
    /// alone is a directory one case removes while another is reading it.
    fn over(label: &str) -> Root {
        let at =
            std::env::temp_dir().join(format!("headwater-cli-init-{}-{label}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(at.join("docs")).expect("the corpus directory is there");
        std::fs::write(at.join("docs/one.md"), "# a document\n").expect("the document writes");
        Root { at }
    }

    /// The same tree, plus the manifest of the package this repository
    /// maintains.
    ///
    /// The manifest alone is the whole of the package-present arm, because
    /// `find_version` reads `package.yml` and opens nothing else. Copying the
    /// real one rather than writing a stand-in is what couples this target to
    /// the version the repository actually ships.
    fn with_package(label: &str) -> Root {
        let root = Root::over(label);
        let directory = root.at.join("packages/headwater-standard");
        std::fs::create_dir_all(&directory).expect("the package directory is there");
        std::fs::copy(
            repository().join("packages/headwater-standard/package.yml"),
            directory.join("package.yml"),
        )
        .expect("the manifest copies");
        root
    }

    /// One run of the verb over this root.
    ///
    /// The exit status is asserted here rather than returned, because every
    /// case below reads a file the verb writes and a refusal writes neither.
    fn init(&self) {
        let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(["init", "--root"])
            .arg(&self.at)
            .output()
            .expect("the binary runs");
        assert_eq!(
            output.status.code(),
            Some(0),
            "`headwater init` writes both files:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn read(&self, relative: &str) -> String {
        std::fs::read_to_string(self.at.join(relative)).expect("the written file reads")
    }

    /// The version the copied manifest declares, read rather than written.
    ///
    /// The assertion on the count is the point of the helper. A manifest that
    /// stops declaring a version at the top level, or that declares two, fails
    /// here and names what it found, rather than silently answering with the
    /// wrong line.
    fn declared_version(&self) -> String {
        let manifest = self.read("packages/headwater-standard/package.yml");
        let lines: Vec<&str> = manifest
            .lines()
            .filter(|line| line.starts_with("version:"))
            .collect();
        assert_eq!(
            lines.len(),
            1,
            "the manifest declares one version at the top level, and it declares {}",
            lines.len()
        );
        lines[0].trim_start_matches("version:").trim().to_string()
    }
}

/// The declaration of a tree with no package under `packages/`.
///
/// This is the first-run state that spec 7 calls the surface `init` is for,
/// and it is the state the tutorial runs its step 2 in. Every byte of it is a
/// literal in the verb, so every byte of it is asserted.
#[test]
fn the_declaration_of_a_tree_with_no_package_is_written_byte_for_byte() {
    let root = Root::over("declaration-no-package");
    root.init();
    let expected = format!("{DECLARATION_HEAD}{NO_PACKAGE_VERSION}{DECLARATION_TAIL}");
    assert_eq!(
        root.read(".headwater/taxonomy.yml"),
        expected,
        "the declaration `headwater init` writes on a blank tree"
    );
}

/// The overlay, which is one arm and carries no package-dependent byte.
#[test]
fn the_overlay_is_written_byte_for_byte() {
    let root = Root::over("overlay");
    root.init();
    assert_eq!(
        root.read(".headwater/overlay.yml"),
        OVERLAY,
        "the overlay `headwater init` writes"
    );
}

/// The declaration of a tree that carries the package, which pins the version
/// the package declares.
///
/// The version is read out of the manifest this case copied. A literal here
/// would go stale on the next bump of the package, which is what #427 did to
/// the tutorial: six mentions moved, one stayed, and every step from 5 to 16
/// failed on a document that was right.
#[test]
fn the_declaration_of_a_tree_with_a_package_pins_the_version_that_package_declares() {
    let root = Root::with_package("declaration-with-package");
    let version = root.declared_version();
    root.init();
    let expected = format!("{DECLARATION_HEAD}  version: {version}\n{DECLARATION_TAIL}");
    assert_eq!(
        root.read(".headwater/taxonomy.yml"),
        expected,
        "the declaration `headwater init` writes over a package it found"
    );
    assert!(
        !expected.contains("0.0.0"),
        "a package on the tree replaces the interview stub rather than joining it"
    );
}

/// `add: {}` is the last line of the overlay.
///
/// `.claude/tutorial/drive.py` asserts the same thing and then splices the
/// reader's edit in at that offset, so anything appended after it breaks a
/// driver assertion rather than an output block, and re-recording the block
/// does not repair it. Held here as well, so that a break points at the verb.
#[test]
fn the_last_line_of_the_overlay_is_the_line_the_tutorial_replaces() {
    let root = Root::over("overlay-last-line");
    root.init();
    let overlay = root.read(".headwater/overlay.yml");
    assert!(
        overlay.trim_end_matches('\n').ends_with("add: {}"),
        "the last line of the overlay is `add: {{}}`, and the tutorial replaces it in place:\n{}",
        overlay
            .lines()
            .rev()
            .take(3)
            .collect::<Vec<&str>>()
            .join("\n")
    );
}
