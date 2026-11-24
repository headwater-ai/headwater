// SPDX-License-Identifier: Apache-2.0
//! `taxonomy.lock.stale`: a lock that its sources no longer resolve to, as an
//! error.
//!
//! `headwater check` takes its taxonomy from `.headwater/taxonomy.lock` alone,
//! so an edit to `.headwater/overlay.yml`, to a vendored package file, or to
//! the selection in `.headwater/taxonomy.yml` reached no check until somebody
//! ran `headwater taxonomy resolve`. Until then `check --strict` passed at exit
//! 0 over a taxonomy the author had already changed
//! ([#1975](https://github.com/headwater-ai/headwater/issues/1975)). In the
//! campaign's `change_taxonomy` scenario, 9 of 9 failures edited the overlay
//! and never resolved.
//!
//! This rule makes the cheap half of `taxonomy resolve --check` on every run,
//! and takes no declaration from the sources. It hashes each source the lock
//! records against the digest recorded for it, and it compares the package,
//! the version and the source list that `.headwater/taxonomy.yml` selects now,
//! in the order a resolve applies it (`headwater_resolve::order`), with the
//! ones the lock records. The resolved text is a function
//! of the sources and of the resolver rules, and `lock.rules` already refuses a
//! lock written under other rules, so these comparisons reach the verdict that
//! `--check` reaches on the generated half without resolving anything. An edit
//! to `.headwater/taxonomy.yml` that changes no resolution, such as the corpus
//! root, an exclusion or the digest pin, gives no finding, and that is correct.
//!
//! **Why not the full resolve.** `headwater_conformance::lock_current`
//! re-resolves and compares the result with the committed file, which is the
//! `lock.current` reading of `headwater conformance`. This crate cannot call
//! it, because `headwater-conformance` depends on this one, and a full resolve
//! would run on every warm, cached `check`. Measured on 2026-11-24 over this
//! repository, dev-release build, 3 runs each: `taxonomy resolve --check` took
//! 0.10 s wall and `headwater check --root .` took 0.61 to 1.21 s. The
//! comparison here reads the consumer declaration and the handful of source
//! files the lock names. Reopen this choice when a lock divergence is found
//! that this comparison misses and `taxonomy resolve --check` reports.
//!
//! **Where it is silent.** Where no lock was read, which is the candidate
//! taxonomy of `taxonomy diff` and `gate`, the caller takes no reading. Where
//! the package the consumer names is not on disk under `.headwater/packages/`,
//! the root carries a lock and no sources, and this rule has nothing to hold
//! the lock to, so it is silent, as [`crate::pin`] is silent where nothing is
//! pinned. That limit means that deleting the whole package directory is not
//! reported here; `taxonomy resolve --check` reports it.
//!
//! **Beside the pin.** Where a digest pin is declared and a vendored file is
//! edited, [`crate::pin`] reports the same edit against the pin and this rule
//! reports it against the lock. The remedies differ: the pin's is to re-vendor
//! and this one's is to resolve. Both findings stand, and the message here
//! names the other one.
//!
//! The rule is about the declaration and not about the prose of a file, so it
//! creates no instance, and it enters the findings list for the reason
//! [`crate::outside_root`] does. It is never cached: the reading is taken on
//! every run by the caller, and every file it hashed joins the read set, so
//! `headwater gate` over a later tree sees an edit to any of them.

use crate::finding::{Finding, Severity};
use crate::instance::Input;
use crate::scope::Scope;
use headwater_resolve::package::{Consumer, CONSUMER, PACKAGES};
use std::path::Path;

pub const RULE: &str = "taxonomy.lock.stale";

/// The grain of [`RULE`]: a fact about the lock read against the tree, so it
/// creates no instance and accounts nothing against the census.
pub const SCOPE: Scope = Scope::taxonomy();

/// Which edition of [`findings`] reached a verdict.
///
/// Raise the rule version when a rule's behavior changes.
pub const VERSION: u32 = 1;

/// Empty: a front-matter schema has no instance to hold this against.
pub const EXPORTABLE_AS: crate::scope::ExportTargets = &[];

/// The remedy every finding of this rule names.
const REMEDY: &str = "run `headwater taxonomy resolve` to write the lock from the sources as \
                      they are now, then run `headwater check` again";

/// One source the lock records, and what its bytes hash to now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    /// The path relative to the root, as the lock records it.
    pub path: String,
    /// The digest the lock records for it.
    pub recorded: String,
    /// The digest of its bytes now, and `None` where it does not read.
    pub computed: Option<String>,
}

/// The lock held against the sources on disk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stale {
    /// The digest of `.headwater/taxonomy.yml`'s own bytes, where it reads,
    /// because the selection is written there and a gate has to see it move.
    pub declaration: Option<String>,
    /// Every source the lock records, in the lock's order.
    pub sources: Vec<Source>,
    /// Why the selection in `.headwater/taxonomy.yml` is not the one the lock
    /// was resolved from, and `None` where it is.
    pub selection: Option<String>,
    /// Whether `.headwater/taxonomy.yml` declares a digest pin, so that a
    /// moved vendored file names the pin finding beside this one.
    pub pinned: bool,
}

impl Stale {
    /// Take the reading for one tree against the lock's own record: the
    /// package, the version and each source with its digest. It hashes every
    /// source, so a caller takes it once per run and never caches it.
    ///
    /// `None` where the package is not on disk, for the reason the module doc
    /// gives.
    pub fn read(
        root: &Path,
        consumer: &Consumer,
        package: &str,
        version: &str,
        recorded: &[(String, String)],
    ) -> Option<Stale> {
        headwater_resolve::package::located(root, &consumer.package)?;
        let declaration = hashed(&root.join(CONSUMER));
        let sources: Vec<Source> = recorded
            .iter()
            .map(|(path, digest)| Source {
                path: path.clone(),
                recorded: digest.clone(),
                computed: hashed(&root.join(path)),
            })
            .collect();
        // The engine range is held by its own report, and a range this engine
        // is outside is no fact about the lock, so the selection is read
        // without it.
        let consumer = &Consumer {
            engine: None,
            ..consumer.clone()
        };
        let selection = if sources.iter().any(|source| source.computed.is_none()) {
            // A source that does not read is already one finding, and the
            // resolver would read the same file again, so the selection waits
            // for the next run.
            None
        } else if consumer.package != package || consumer.version != version {
            Some(format!(
                "it takes {} {}, and the lock was resolved from {package} {version}",
                consumer.package, consumer.version
            ))
        } else {
            match headwater_resolve::package::sources(root, consumer).and_then(applied) {
                Err(errors) => Some(format!(
                    "the sources it selects do not resolve: {}",
                    errors
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("; ")
                )),
                Ok(now) => {
                    let now: Vec<&str> = now.iter().map(|source| source.name.as_str()).collect();
                    let then: Vec<&str> = recorded.iter().map(|(path, _)| path.as_str()).collect();
                    (now != then).then(|| {
                        format!(
                            "it selects {} source(s) now, in this order: {}; and the lock was \
                             resolved from {}: {}",
                            now.len(),
                            listed(&now),
                            then.len(),
                            listed(&then)
                        )
                    })
                }
            }
        };
        Some(Stale {
            declaration,
            sources,
            selection,
            pinned: consumer.digest.is_some(),
        })
    }

    /// What this reading read, as read-set inputs: the declaration, and each
    /// source under its path from the root. Sorted by path.
    pub fn inputs(&self) -> Vec<Input> {
        let mut out = Vec::new();
        if let Some(digest) = &self.declaration {
            out.push(Input::new(CONSUMER, Some(digest)));
        }
        for source in &self.sources {
            out.push(Input::new(source.path.clone(), source.computed.as_deref()));
        }
        out.sort_by(|a, b| a.path.cmp(&b.path));
        out.dedup_by(|a, b| a.path == b.path);
        out
    }
}

/// The digest of one file, where it is a regular file that reads. A named
/// pipe or a device is refused before it is opened, as the lock reader
/// refuses one, so a run never waits on it (#1366).
fn hashed(path: &Path) -> Option<String> {
    if !std::fs::metadata(path).ok()?.is_file() {
        return None;
    }
    std::fs::read(path)
        .ok()
        .map(|bytes| headwater_hash::digest(&bytes))
}

/// The selected sources in the order a resolve applies them, which is the
/// order the lock records: the base, then each overlay once every bundle it
/// requires has applied, with the adopter overlay last. This reads each
/// overlay's `requires` and applies no operation.
fn applied(
    selected: Vec<headwater_resolve::Source>,
) -> Result<Vec<headwater_resolve::Source>, Vec<headwater_resolve::ResolveError>> {
    let Some((base, overlays)) = selected.split_first() else {
        return Ok(selected);
    };
    let ordered = headwater_resolve::order::order(overlays)?;
    Ok(std::iter::once(base.clone())
        .chain(ordered.order.iter().map(|index| overlays[*index].clone()))
        .collect())
}

fn listed(paths: &[&str]) -> String {
    match paths.is_empty() {
        true => "none".to_string(),
        false => paths
            .iter()
            .map(|path| format!("`{path}`"))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

/// One error on each source whose bytes moved or that is gone, and one on the
/// consumer declaration where the selection moved.
pub fn findings(stale: Option<&Stale>) -> Vec<Finding> {
    let Some(stale) = stale else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for source in &stale.sources {
        if source.computed.as_deref() == Some(source.recorded.as_str()) {
            continue;
        }
        let what = match &source.computed {
            Some(computed) => format!(
                "`{}` hashes to {computed}, and `.headwater/taxonomy.lock` was resolved from \
                 bytes that hash to {}",
                source.path, source.recorded
            ),
            None => format!(
                "`{}` is a source `.headwater/taxonomy.lock` was resolved from, at {}, and it \
                 does not read now (no digest)",
                source.path, source.recorded
            ),
        };
        let pin = if stale.pinned && source.path.starts_with(&format!("{PACKAGES}/")) {
            "; `taxonomy.pin.diverged` reports the same edit against the digest pin, and its \
             remedy is to re-vendor"
        } else {
            ""
        };
        out.push(finding(
            source.path.clone(),
            format!("{what}, so no check reads this edit yet{pin}"),
        ));
    }
    if let Some(why) = &stale.selection {
        out.push(finding(
            CONSUMER.to_string(),
            format!(
                "`{CONSUMER}` no longer selects what `.headwater/taxonomy.lock` was resolved \
                 from: {why}"
            ),
        ));
    }
    out
}

fn finding(path: String, message: String) -> Finding {
    Finding {
        rule: RULE,
        severity: Severity::Error,
        obligation: None,
        path,
        line: 0,
        column: 0,
        message,
        remediation: REMEDY.to_string(),
        patch: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(path: &str, recorded: &str, computed: Option<&str>) -> Source {
        Source {
            path: path.to_string(),
            recorded: recorded.to_string(),
            computed: computed.map(str::to_string),
        }
    }

    fn stale(sources: Vec<Source>, selection: Option<&str>, pinned: bool) -> Stale {
        Stale {
            declaration: Some("sha256:decl".to_string()),
            sources,
            selection: selection.map(str::to_string),
            pinned,
        }
    }

    #[test]
    fn a_lock_its_sources_hash_to_is_silent() {
        let reading = stale(
            vec![source(
                ".headwater/overlay.yml",
                "sha256:a",
                Some("sha256:a"),
            )],
            None,
            false,
        );
        assert!(findings(Some(&reading)).is_empty());
        assert!(findings(None).is_empty());
    }

    #[test]
    fn a_moved_source_is_an_error_on_it_that_names_both_digests_and_the_verb() {
        let reading = stale(
            vec![
                source(
                    ".headwater/packages/p/taxonomy.yml",
                    "sha256:p",
                    Some("sha256:p"),
                ),
                source(".headwater/overlay.yml", "sha256:then", Some("sha256:now")),
            ],
            None,
            false,
        );
        let found = findings(Some(&reading));
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].rule, RULE);
        assert_eq!(found[0].severity, Severity::Error);
        assert_eq!(found[0].path, ".headwater/overlay.yml");
        for needle in ["sha256:then", "sha256:now"] {
            assert!(found[0].message.contains(needle), "{}", found[0].message);
        }
        assert!(found[0].remediation.contains("headwater taxonomy resolve"));
        assert!(!found[0].message.contains("pin"), "{}", found[0].message);
    }

    #[test]
    fn a_missing_source_is_an_error_with_no_computed_digest() {
        let reading = stale(
            vec![source(".headwater/overlay.yml", "sha256:then", None)],
            None,
            false,
        );
        let found = findings(Some(&reading));
        assert_eq!(found.len(), 1);
        assert!(
            found[0].message.contains("no digest"),
            "{}",
            found[0].message
        );
    }

    #[test]
    fn a_moved_vendored_file_under_a_pin_names_the_pin_finding() {
        let vendored = ".headwater/packages/p/taxonomy.yml";
        let pinned = stale(
            vec![source(vendored, "sha256:then", Some("sha256:now"))],
            None,
            true,
        );
        let found = findings(Some(&pinned));
        assert!(
            found[0].message.contains("taxonomy.pin.diverged"),
            "{}",
            found[0].message
        );
        let overlay = stale(
            vec![source(
                ".headwater/overlay.yml",
                "sha256:then",
                Some("sha256:now"),
            )],
            None,
            true,
        );
        assert!(!findings(Some(&overlay))[0].message.contains("pin.diverged"));
    }

    #[test]
    fn a_moved_selection_is_an_error_on_the_consumer_declaration() {
        let reading = stale(Vec::new(), Some("it selects one source now"), false);
        let found = findings(Some(&reading));
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].path, CONSUMER);
        assert!(found[0].message.contains("it selects one source now"));
    }

    #[test]
    fn the_inputs_are_the_declaration_and_every_source() {
        let reading = stale(
            vec![
                source(".headwater/overlay.yml", "sha256:a", Some("sha256:b")),
                source(".headwater/gone.yml", "sha256:c", None),
            ],
            None,
            false,
        );
        let inputs = reading.inputs();
        let paths: Vec<&str> = inputs.iter().map(|input| input.path.as_str()).collect();
        assert_eq!(
            paths,
            [".headwater/gone.yml", ".headwater/overlay.yml", CONSUMER]
        );
        assert_eq!(inputs[1].digest.as_deref(), Some("sha256:b"));
        assert_eq!(inputs[0].digest, None);
    }
}
