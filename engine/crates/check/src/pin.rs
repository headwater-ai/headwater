// SPDX-License-Identifier: Apache-2.0
//! `taxonomy.pin.diverged`: a digest pin that the vendored package no longer
//! hashes to, as an error.
//!
//! `headwater taxonomy vendor` holds a fetched artifact to the pin once, when
//! it installs it, and nothing held the installed bytes to it afterwards
//! except `headwater conformance`, which a commit hook and CI do not run
//! ([#1186](https://github.com/headwater-ai/headwater/issues/1186)). So a pin
//! edited by hand, and a vendored file edited by hand, both passed
//! `check --strict`. This rule makes the same comparison `vendor` makes, on
//! every run, through [`headwater_resolve::release::pinned`], which is the one
//! copy of it.
//!
//! It takes no declaration from the vendored bytes. The taxonomy still comes
//! from the lock alone, and the bytes are only hashed. It is silent where no
//! pin is declared, because that gap is `pin.current`'s to report and a
//! repository that takes its package from source must not go red.
//!
//! The rule is about the declaration and not about the prose of a file, so it
//! creates no instance, and it enters the findings list for the reason
//! [`crate::outside_root`] does. It is never cached: the reading is taken on
//! every run by the caller, and every member it hashed joins the read set, so
//! `headwater gate` over a later tree sees a vendored edit.

use crate::finding::{Finding, Severity};
use crate::instance::Input;
use crate::scope::Scope;
use headwater_resolve::package::{Consumer, CONSUMER};
use headwater_resolve::release;
use std::path::Path;

pub const RULE: &str = "taxonomy.pin.diverged";

/// The grain of [`RULE`]: a fact about the taxonomy's pin read against the
/// tree, so it creates no instance and accounts nothing against the census.
pub const SCOPE: Scope = Scope::taxonomy();

/// Which edition of [`findings`] reached a verdict.
pub const VERSION: u32 = 1;

/// Empty: a front-matter schema has no instance to hold this against.
pub const EXPORTABLE_AS: crate::scope::ExportTargets = &[];

/// The pin a repository declares, and what its vendored bytes hash to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pin {
    /// The package the consumer takes, as `.headwater/taxonomy.yml` names it.
    pub package: String,
    /// The version the consumer takes.
    pub version: String,
    /// The digest of `.headwater/taxonomy.yml`'s own bytes, where it reads,
    /// because the pin is written there and a gate has to see it move.
    pub declaration: Option<String>,
    /// The package directory relative to the root, with `/` separators, where
    /// one was found.
    pub directory: Option<String>,
    /// What the comparison found, and `None` where nothing is pinned.
    pub reading: Option<release::Pinned>,
}

impl Pin {
    /// Take the reading for one tree. It hashes every vendored member, so a
    /// caller takes it once per run and never caches it.
    pub fn read(root: &Path, consumer: &Consumer) -> Pin {
        let declaration = std::fs::read(root.join(CONSUMER))
            .ok()
            .map(|bytes| headwater_hash::digest(&bytes));
        let located = headwater_resolve::package::located(root, &consumer.package)
            .map(|(directory, _)| directory);
        let directory = located.as_ref().map(|directory| {
            directory
                .strip_prefix(root)
                .unwrap_or(directory)
                .components()
                .map(|part| part.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/")
        });
        let reading = consumer
            .digest
            .as_deref()
            .map(|pinned| release::pinned(located.as_deref(), pinned));
        Pin {
            package: consumer.package.clone(),
            version: consumer.version.clone(),
            declaration,
            directory,
            reading,
        }
    }

    /// What this reading read, as read-set inputs: the declaration, and each
    /// vendored member under its path from the root. Sorted by path.
    pub fn inputs(&self) -> Vec<Input> {
        let mut out = Vec::new();
        if let Some(digest) = &self.declaration {
            out.push(Input::new(CONSUMER, Some(digest)));
        }
        if let (Some(directory), Some(reading)) = (&self.directory, &self.reading) {
            for member in &reading.members {
                out.push(Input::new(
                    format!("{directory}/{}", member.path),
                    Some(&member.digest),
                ));
            }
        }
        out.sort_by(|a, b| a.path.cmp(&b.path));
        out
    }
}

/// One error where a pin is declared and the vendored bytes do not hash to it.
pub fn findings(pin: Option<&Pin>) -> Vec<Finding> {
    let Some(pin) = pin else { return Vec::new() };
    let Some(drift) = pin
        .reading
        .as_ref()
        .and_then(|reading| reading.drift.as_ref())
    else {
        return Vec::new();
    };
    let place = match &pin.directory {
        Some(directory) => format!("the package vendored under `{directory}/`"),
        None => format!(
            "no package under `{}/`",
            headwater_resolve::package::PACKAGES
        ),
    };
    vec![Finding {
        rule: RULE,
        severity: Severity::Error,
        obligation: None,
        path: CONSUMER.to_string(),
        line: 0,
        column: 0,
        message: format!(
            "`{CONSUMER}` pins a digest of {} {}, and {place} does not hash to it: {drift}",
            pin.package, pin.version
        ),
        remediation: "re-vendor the artifact the pin names with `headwater taxonomy vendor`, \
                      or pin the digest `headwater taxonomy publish` printed for these bytes"
            .to_string(),
        patch: None,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A scratch directory that removes itself when the guard is dropped, so
    /// a case that fails an assertion leaves nothing behind (#1158).
    struct Scratch(PathBuf);

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    impl std::ops::Deref for Scratch {
        type Target = Path;
        fn deref(&self) -> &Path {
            &self.0
        }
    }

    /// A scratch package directory, keyed on the test name as well as the
    /// pid: cargo runs the cases of one target as threads of one process.
    fn package(name: &str) -> Scratch {
        let dir = std::env::temp_dir().join(format!("hw-pin-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let dir = Scratch(dir);
        std::fs::create_dir_all(dir.join("doctrine")).expect("the scratch package is made");
        std::fs::write(dir.join("package.yml"), "name: acme/pkg\n").expect("manifest");
        std::fs::write(dir.join("doctrine/a.md"), "one\n").expect("member");
        dir
    }

    fn digest(dir: &Path) -> String {
        release::digest_of(&release::members(dir).expect("the members read"))
    }

    fn pin(reading: Option<release::Pinned>, directory: Option<&str>) -> Pin {
        Pin {
            package: "acme/pkg".to_string(),
            version: "1.0.0".to_string(),
            declaration: Some("sha256:decl".to_string()),
            directory: directory.map(str::to_string),
            reading,
        }
    }

    #[test]
    fn a_pin_the_bytes_hash_to_is_silent() {
        let dir = package("agrees");
        let reading = release::pinned(Some(&*dir), &digest(&dir));
        assert_eq!(reading.drift, None);
        assert_eq!(reading.members.len(), 2);
        assert!(findings(Some(&pin(Some(reading), Some("pkgs/p")))).is_empty());
    }

    #[test]
    fn nothing_pinned_is_silent_even_with_no_package() {
        assert!(findings(Some(&pin(None, None))).is_empty());
        assert!(findings(None).is_empty());
    }

    #[test]
    fn a_pin_with_no_package_directory_is_an_error_with_no_computed_digest() {
        let reading = release::pinned(None, "sha256:pinned");
        let drift = reading.drift.clone().expect("a pin with no bytes is drift");
        assert_eq!(drift.computed, None);
        let found = findings(Some(&pin(Some(reading), None)));
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].rule, RULE);
        assert_eq!(found[0].severity, Severity::Error);
        assert_eq!(found[0].path, CONSUMER);
        assert!(
            found[0].message.contains("sha256:pinned"),
            "{}",
            found[0].message
        );
        assert!(
            found[0].message.contains("no digest"),
            "{}",
            found[0].message
        );
    }

    #[test]
    fn an_edited_member_is_an_error_that_names_both_digests() {
        let dir = package("edited");
        let pinned = digest(&dir);
        std::fs::write(dir.join("doctrine/a.md"), "two\n").expect("the hand edit");
        let computed = digest(&dir);
        let reading = release::pinned(Some(&*dir), &pinned);
        let found = findings(Some(&pin(Some(reading), Some("pkgs/p"))));
        assert_eq!(found.len(), 1);
        for needle in [pinned.as_str(), computed.as_str(), "pkgs/p/"] {
            assert!(found[0].message.contains(needle), "{}", found[0].message);
        }
        assert!(
            !found[0].remediation.contains("publisher"),
            "HW-OBL-0115: a digest never names who wrote the bytes"
        );
    }

    #[test]
    fn the_inputs_are_the_declaration_and_every_member_under_the_directory() {
        let dir = package("inputs");
        let reading = release::pinned(Some(&*dir), "sha256:other");
        let inputs = pin(Some(reading), Some("pkgs/p")).inputs();
        let paths: Vec<&str> = inputs.iter().map(|input| input.path.as_str()).collect();
        assert_eq!(
            paths,
            [CONSUMER, "pkgs/p/doctrine/a.md", "pkgs/p/package.yml"]
        );
        assert!(inputs.iter().all(|input| input.digest.is_some()));
    }
}
