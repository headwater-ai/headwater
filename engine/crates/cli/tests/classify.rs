// SPDX-License-Identifier: Apache-2.0
//! [#319](https://github.com/headwater-ai/headwater/issues/319): one matcher
//! decides an exclusion, for a path that exists and for a path that does not.
//!
//! # The defect this target is written against
//!
//! `.claude/hooks/write.sh` used to answer "does the corpus claim this path"
//! with a matcher of its own: it read `.headwater/corpus.json` and tested each
//! exclusion with Python's `fnmatch`. `headwater_meta::pattern::Pattern` is
//! what the census walk matches an existing file against, and nothing compared
//! the two. They agree on a plain `prefix/**` exclusion, which is the only
//! shape this repository's own corpus declares, so the drift never showed up
//! here. It shows up on a mid-pattern `*`: `fnmatch` treats `*` as "any run of
//! characters, including `/`", so `fnmatch.fnmatch("docs/excluded/sub/dir.md",
//! "docs/excluded/*.md")` is true. `Pattern` keeps `*` inside one path segment
//! ([`headwater_meta::pattern`]'s module comment), so the same test is false —
//! the path is four segments against a three-segment pattern with no `**`, and
//! `Corpus::classify` reports it as corpus content rather than excluded.
//!
//! This target builds a corpus that declares exactly that pattern and asks
//! `headwater explain` about a path with no file behind it, on both sides of
//! the divergence. A second matcher, reintroduced anywhere on the path from
//! `.claude/hooks/write.sh` to the walk, answers the first case wrong.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the directory is there");
    for entry in std::fs::read_dir(from).expect("the fixture directory reads") {
        let entry = entry.expect("the entry reads");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("the file type reads").is_dir() {
            true => copy(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("the fixture copies");
            }
        }
    }
}

/// A scratch corpus over this repository's own taxonomy, with one exclusion
/// this case adds: `docs/excluded/*.md`, a pattern no exclusion this
/// repository declares for real ever needs, because it is the shape that
/// tells the two matchers apart.
///
/// The maintained source (`taxonomy-source/headwater-standard`) rather than
/// the vendored artifact under `packages/`, so this case runs at whatever
/// version this repository is at rather than pinning one and going stale
/// under it, the same choice `diff.rs` and `migration.rs` make for the same
/// reason.
struct Root {
    at: PathBuf,
}

impl Root {
    /// `label` names the case rather than the target, because cargo runs the
    /// cases of one target as threads of one process and a directory keyed on
    /// the process identifier alone is one that a second case removes while
    /// the first reads it.
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-classify-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");

        let repository = repository();
        copy(
            &repository.join("taxonomy-source/headwater-standard"),
            &at.join("packages/headwater-standard"),
        );
        copy(
            &repository.join("docs/taxonomies"),
            &at.join("docs/taxonomies"),
        );
        for name in ["taxonomy.yml", "overlay.yml"] {
            let to = at.join(".headwater").join(name);
            std::fs::create_dir_all(to.parent().expect("it has a parent"))
                .expect("the declaration directory is there");
            std::fs::copy(repository.join(".headwater").join(name), to)
                .expect("the declaration copies");
        }

        // The one line this case adds to the consumer declaration: a second
        // exclusion, in the one shape that a whole-string `fnmatch` and a
        // segment-aware `Pattern` read differently.
        let declaration = at.join(".headwater/taxonomy.yml");
        let text = std::fs::read_to_string(&declaration).expect("the declaration reads");
        let from = "  exclude:\n";
        assert!(
            text.contains(from),
            "the copied declaration still declares `exclude:` at two spaces"
        );
        let to = "  exclude:\n    - path: docs/excluded/*.md\n      reason: >-\n        a fixture for #319: `*` inside one segment for `Pattern`, any run of\n        characters for `fnmatch`, so the two disagree about a path four\n        segments deep against this three-segment pattern.\n";
        let patched = text.replacen(from, to, 1);
        std::fs::write(&declaration, patched).expect("the declaration writes");

        let root = Root { at };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolved.code,
            Some(0),
            "the fixture resolves\n{}{}",
            resolved.out,
            resolved.err
        );
        root
    }

    fn run(&self, arguments: &[&str]) -> Ran {
        let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(arguments)
            .arg("--root")
            .arg(&self.at)
            .output()
            .expect("the binary runs");
        Ran {
            code: output.status.code(),
            out: String::from_utf8_lossy(&output.stdout).into_owned(),
            err: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
}

/// The case the two matchers disagree on. Four segments, no file behind it,
/// against a three-segment pattern with no `**`: `Pattern` does not match, so
/// this is corpus content and the hook that reads this exact sentence refuses
/// a raw write there. A reintroduced `fnmatch` would call it excluded instead,
/// and the write would silently pass.
#[test]
fn a_path_four_segments_deep_is_corpus_content_under_the_segment_aware_matcher() {
    let root = Root::new("four-segments");
    let explained = root.run(&["explain", "docs/excluded/sub/dir.md"]);
    assert_eq!(
        explained.code,
        Some(1),
        "a target with no document: {explained:?}"
    );
    assert!(
        explained.out.is_empty(),
        "no document to print: {explained:?}"
    );
    assert!(
        explained
            .err
            .contains("is a path of this corpus, with no document written there yet"),
        "`fnmatch` would have called this excluded; `Pattern` does not, \
         because `*` does not cross the `/` before `sub`: {explained:?}"
    );
}

/// The control: one segment past `docs/excluded/`, which both matchers agree
/// is excluded. Without this, the case above could pass because the pattern
/// never excludes anything at all.
#[test]
fn a_path_one_segment_deep_is_excluded_under_either_matcher() {
    let root = Root::new("one-segment");
    let explained = root.run(&["explain", "docs/excluded/dir.md"]);
    assert_eq!(
        explained.code,
        Some(1),
        "a target with no document: {explained:?}"
    );
    assert!(
        explained
            .err
            .contains("is excluded by `docs/excluded/*.md`"),
        "both matchers exclude a path this shallow: {explained:?}"
    );
}
