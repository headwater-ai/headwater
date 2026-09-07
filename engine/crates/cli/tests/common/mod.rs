// SPDX-License-Identifier: Apache-2.0
//! The scratch repository that several targets run over, and the version pin
//! they all take.
//!
//! `diff.rs` and `migration.rs` both build a scratch repository out of this
//! one's `packages/`, `docs/taxonomies/` and `.headwater/`, and both then read
//! a step from one package version to the next.
//!
//! [`Root`] moved here from `diff.rs` when `conformance_lock.rs` needed the
//! same root. A second copy would be a second fixture, and two targets that
//! disagreed about what the root declares would each be measuring their own.
//!
//! A target that includes this module is not obliged to use all of it —
//! `migration.rs` takes [`pin`] and builds its own root — so the module allows
//! what it does not use rather than each target carrying a list of exemptions.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// This repository, which every scratch root is copied out of.
pub(crate) fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// A repository root that removes itself.
///
/// `label` names the test and not the case. Cargo runs the cases of one target
/// as threads of one process, so a directory keyed on the process identifier
/// alone is a directory one case removes while another is reading it.
pub(crate) struct Root {
    pub(crate) at: PathBuf,
}

impl Root {
    pub(crate) fn new(label: &str) -> Root {
        Root::shaped(label, |_| {})
    }

    /// A root whose bundle order makes one overlay found the kind another one
    /// declares, so the resolution carries a founding and every publish out of
    /// it carries the same founding.
    ///
    /// The two bundles commute, which is the whole point: `zz-a` writes a leaf
    /// under `kinds.zz_thing` and `zz-b` declares that kind, so the declared
    /// order decides which of them creates the key. The consumer is free to
    /// list them either way — `headwater_resolve::package::selected` pushes
    /// bundles in the declared order with no topological pass — so the order
    /// here is a legal one and not a broken root.
    ///
    /// Three things about the pair, each of which cost an earlier attempt.
    /// `taxonomy publish` publishes the package with every bundle it ships, so
    /// a synthetic bundle has to resolve under that maximal set as well as
    /// under this consumer's selection: an address that reads a declaration the
    /// adopter overlay carries resolves for the consumer and fails the publish.
    /// A synthetic concrete kind needs a shelf, or `coverage` refuses the
    /// resolution. And reaching into a key an existing bundle or the adopter
    /// overlay already writes is refused as a collision rather than recorded as
    /// a founding, so the kind name is one nothing else names.
    pub(crate) fn founding(label: &str) -> Root {
        Root::shaped(label, |at| {
            write_bundle(at, "zz-a", FOUNDS);
            write_bundle(at, "zz-b", DECLARES);

            let declaration = at.join(".headwater/taxonomy.yml");
            let text = std::fs::read_to_string(&declaration).expect("the declaration reads");
            let from = "  bundles: [design-spec";
            assert!(text.contains(from), "the declaration lists its bundles");
            std::fs::write(
                &declaration,
                text.replacen(from, "  bundles: [zz-a, zz-b, design-spec", 1),
            )
            .expect("the declaration writes");
        })
    }

    pub(crate) fn shaped(label: &str, prepare: impl FnOnce(&Path)) -> Root {
        let at =
            std::env::temp_dir().join(format!("headwater-cli-root-{}-{label}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");

        let repository = repository();
        // `packages/headwater-standard/` is a vendored artifact since #366
        // (it carries a `release.yml`, and `taxonomy publish` now refuses to
        // publish a directory in that state — the guard this fixture would
        // otherwise trip, since every case here calls `taxonomy publish` by
        // name with no `--from`). The maintained source is
        // `taxonomy-source/headwater-standard/`, copied here to the path the
        // by-name lookup expects.
        copy(
            &repository.join("taxonomy-source/headwater-standard"),
            &at.join("packages/headwater-standard"),
        );
        copy(
            &repository.join("docs/taxonomies"),
            &at.join("docs/taxonomies"),
        );
        copy(
            &repository.join("engine/crates/cli/fixtures/change/docs"),
            &at.join("docs"),
        );
        for name in ["taxonomy.yml", "overlay.yml"] {
            let to = at.join(".headwater").join(name);
            std::fs::create_dir_all(to.parent().expect("it has a parent"))
                .expect("the declaration directory is there");
            std::fs::copy(repository.join(".headwater").join(name), to)
                .expect("the declaration copies");
        }

        prepare(&at);
        pin(&at, "1.0.0");

        let root = Root { at };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(resolved.code, Some(0), "the fixture resolves: {resolved:?}");
        root
    }

    pub(crate) fn run(&self, arguments: &[&str]) -> Ran {
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

/// The `add:` block that writes a leaf under a kind nothing has declared yet,
/// so the resolver creates the parent on the way down and records a founding.
pub(crate) const FOUNDS: &str = "add:\n  kinds.zz_thing.voice: declarative\n";

/// The `add:` block that declares the kind, and the shelf `coverage` requires
/// of a concrete one.
pub(crate) const DECLARES: &str = "add:\n  kinds.zz_thing:\n    is_a: governed_document\n    \
                                   purpose: behavior\n    lifecycle: standard\n  \
                                   shelves.zz_things:\n    title: Zz Things\n    path: \
                                   docs/zz/**\n    homogeneous: true\n    kind: zz_thing\n";

/// Write one bundle of the founding pair.
///
/// The two blocks are held apart from the two names because the pair commutes:
/// a case that swaps which bundle carries which block writes a root that founds
/// the same kind out of the other file, and it does that with these two
/// constants rather than with a second copy of both bundles.
pub(crate) fn write_bundle(at: &Path, name: &str, add: &str) {
    let directory = at.join("docs/taxonomies").join(name);
    std::fs::create_dir_all(&directory).expect("the bundle directory is made");
    std::fs::write(
        directory.join("bundle.yml"),
        format!(
            "# SPDX-License-Identifier: Apache-2.0\n\nbundle: {name}\nextends: \
             headwater/standard@4.1.0\nrequires: []\n\n{add}"
        ),
    )
    .expect("the bundle writes");
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

#[derive(Debug)]
pub(crate) struct Ran {
    pub(crate) code: Option<i32>,
    pub(crate) out: String,
    pub(crate) err: String,
}

pub(crate) fn copy(from: &Path, to: &Path) {
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

/// Fix the copied package at one version, whatever this repository's own
/// package is at.
///
/// Every case in both targets reads a step from 1.0.0 to 2.0.0. Without this
/// the base version is inherited from `packages/`, so a major of the real
/// package rewrites a literal in every case at once. The version under test is
/// a property of the case and not of the repository the fixture is copied from.
///
/// Three files carry the number and all three move together: the manifest that
/// `resolve::package::sources` holds a consumer to, the taxonomy source beside
/// it, and the consumer declaration that pins what this root takes. The
/// manifest is read first, because it is the one the resolver compares.
pub(crate) fn pin(at: &Path, version: &str) {
    let manifest = at.join("packages/headwater-standard/package.yml");
    let text = std::fs::read_to_string(&manifest).expect("the manifest reads");
    let found = text
        .lines()
        .find_map(|line| line.strip_prefix("version: "))
        .expect("the manifest declares a version")
        .to_string();

    for (relative, indent) in [
        ("packages/headwater-standard/package.yml", ""),
        ("packages/headwater-standard/taxonomy.yml", ""),
        (".headwater/taxonomy.yml", "  "),
    ] {
        let path = at.join(relative);
        let text = std::fs::read_to_string(&path).expect("the source reads");
        let from = format!("\n{indent}version: {found}\n");
        assert!(
            text.contains(&from),
            "{relative} does not declare version {found}"
        );
        let to = format!("\n{indent}version: {version}\n");
        std::fs::write(&path, text.replacen(&from, &to, 1)).expect("the source writes");
    }
}
