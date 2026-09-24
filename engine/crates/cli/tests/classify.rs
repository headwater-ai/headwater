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
/// the vendored artifact under `.headwater/packages/`, so this case runs at whatever
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
            &at.join(".headwater/packages/headwater-standard"),
        );
        repoint_bundles(&at.join(".headwater/packages/headwater-standard"));
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

/// [#845](https://github.com/headwater-ai/headwater/issues/845), the decisive
/// fixture: a typo of a real identifier and an invented one, against this
/// repository's own `decision_id` scheme (`{namespace}-DR-{seq:04d}`,
/// namespace `HW`). Before the repair both read the path sentence — "is
/// outside every corpus root this repository declares" — which is true of a
/// path and says nothing true about either target. The fix does not have to
/// tell a typo from an invention (that is `resolve_identifier`'s job, a
/// separate fixture); it only has to stop claiming both are a path.
#[test]
fn an_identifier_shaped_target_refuses_in_the_words_of_an_identifier_and_not_a_path() {
    let root = Root::new("identifier-shaped");
    for target in ["HW-DR-004", "HW-DR-9999"] {
        let explained = root.run(&["explain", target]);
        assert_eq!(
            explained.code,
            Some(1),
            "a target no document carries: {explained:?}"
        );
        assert!(
            explained
                .err
                .contains("is shaped like an identifier of this corpus, and no document declares it"),
            "{target} opens on `HW-DR-`, the fixed prefix `{{namespace}}-DR-{{seq:04d}}` declares: {explained:?}"
        );
        assert!(
            !explained.err.contains("is outside every corpus root"),
            "the path sentence must not print for an identifier-shaped target: {explained:?}"
        );
    }
}

/// The control beside the case above: a path-shaped target with no document
/// still reads the four path states unchanged, because the identifier check
/// is additive and never a rewrite of `Corpus::classify`.
#[test]
fn a_path_shaped_target_with_no_document_still_reads_the_path_states() {
    let root = Root::new("path-shaped-control");
    let explained = root.run(&["explain", "engine/nowhere/at-all.rs"]);
    assert_eq!(
        explained.code,
        Some(1),
        "a target no document carries: {explained:?}"
    );
    assert!(
        explained
            .err
            .contains("is outside every corpus root this repository declares"),
        "a path outside the corpus root still reads the path sentence: {explained:?}"
    );
}

/// Repoint `contents.bundles` in a scratch copy of the authored manifest.
///
/// The scalar is relative to the package directory, and
/// [`package::PACKAGES`] put that directory one level deeper in #792. The
/// prefix is computed from the constant rather than written out, so a root
/// that moves again moves this with it. `taxonomy publish` rewrites this same
/// scalar on every artifact it writes, so a fixture that does it here is not
/// inventing a mechanism.
fn repoint_bundles(package: &std::path::Path) {
    let up = "../".repeat(headwater_resolve::package::PACKAGES.split('/').count() + 1);
    let manifest = package.join(headwater_resolve::package::MANIFEST);
    let text = std::fs::read_to_string(&manifest).expect("the scratch manifest reads");
    let from = "  bundles: ../../docs/taxonomies";
    assert!(text.contains(from), "the authored manifest states `{from}`");
    let to = format!("  bundles: {up}docs/taxonomies");
    std::fs::write(&manifest, text.replace(from, &to)).expect("the scratch manifest writes");
}

/// [#350](https://github.com/headwater-ai/headwater/issues/350), the decisive
/// fixture. The library's doctrine is the prose an adopter reads before
/// choosing an entry, and until this case `docs/taxonomies/**` excluded it
/// whole, so no rule read a word of it. The same defects are planted twice:
/// in `design-spec/doctrine.md`, where every rule of the house regime must
/// now name them, and in a file under `design-spec/fixtures/corpus/`, which
/// is a corpus another root walks and holds defects on purpose, so it must
/// stay excluded and draw nothing. The first half catches the gap; the second
/// catches the over-correction that lets planted fixture defects into this
/// corpus.
const PLANTED: &str = "\nWe will colour this entry in a later release, and it doesn't matter\nhow the next line starts, because this block is wrapped by hand.\n\nThis sentence runs on with many more words than the house profile admits, so that the count of its words goes well past the limit of twenty five words that the regime states. It is a load-bearing claim.\n";

/// Every `(path, rule)` pair a human-readable report names. A finding opens
/// on `  <path>:<line>:<column> <severity>` and the next line opens on the
/// rule identifier.
fn findings(out: &str) -> Vec<(String, String)> {
    let lines: Vec<&str> = out.lines().collect();
    let mut pairs = Vec::new();
    for (at, line) in lines.iter().enumerate() {
        let Some(rest) = line.strip_prefix("  ") else {
            continue;
        };
        if rest.starts_with(' ') || !rest.starts_with("docs/") {
            continue;
        }
        let Some((path, _)) = rest.split_once(':') else {
            continue;
        };
        let Some(next) = lines.get(at + 1) else {
            continue;
        };
        let rule = next.trim_start().split([' ', ':']).next().unwrap_or("");
        pairs.push((path.to_owned(), rule.to_owned()));
    }
    pairs
}

#[test]
fn the_library_doctrine_is_checked_and_a_fixture_corpus_under_it_is_not() {
    let root = Root::new("doctrine-governed");
    let doctrine = "docs/taxonomies/design-spec/doctrine.md";
    let fixture = "docs/taxonomies/design-spec/fixtures/corpus/docs/planted-350.md";
    let path = root.at.join(doctrine);
    let mut text = std::fs::read_to_string(&path).expect("the doctrine reads");
    text.push_str(PLANTED);
    std::fs::write(&path, text).expect("the doctrine writes");
    let planted = root.at.join(fixture);
    std::fs::create_dir_all(planted.parent().expect("it has a parent"))
        .expect("the fixture directory is there");
    std::fs::write(&planted, format!("# A planted fixture page\n{PLANTED}"))
        .expect("the fixture page writes");

    let explained = root.run(&["explain", doctrine]);
    assert_eq!(explained.code, Some(0), "the doctrine is a document: {explained:?}");
    assert!(
        explained.out.contains("library_doctrine"),
        "the doctrine page is typed `library_doctrine`: {explained:?}"
    );
    let excluded = root.run(&["explain", fixture]);
    assert!(
        excluded.err.contains("is excluded by `docs/taxonomies/*/fixtures/**`"),
        "a fixture corpus page is excluded by the fixture rule: {excluded:?}"
    );

    let checked = root.run(&["check", "--strict"]);
    assert_ne!(checked.code, Some(0), "the planted errors fail a strict run: {checked:?}");
    let pairs = findings(&checked.out);
    for rule in [
        "language.source_form.not_met",
        "language.controlled.not_met",
        "language.retired_term.used",
        "voice.forbidden_construction",
    ] {
        assert!(
            pairs.iter().any(|(p, r)| p == doctrine && r == rule),
            "{rule} names {doctrine}: {pairs:?}\n{}",
            checked.out
        );
    }
    assert!(
        !pairs.iter().any(|(p, _)| p.starts_with("docs/taxonomies/design-spec/fixtures/")),
        "no finding names a fixture corpus page: {pairs:?}"
    );
    assert!(
        !checked.out.contains("planted-350.md"),
        "the planted fixture page draws no instance and no census row: {}",
        checked.out
    );
}
