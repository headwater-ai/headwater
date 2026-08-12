// SPDX-License-Identifier: Apache-2.0
//! The content-hashed lock: one resolution, written down, with a digest over it.
//!
//! [Spec 6](../../../docs/spec/06-engine-architecture.md#pipeline): "Resolve
//! merges the base taxonomy and overlays, validates against the meta-schema, and
//! writes a content-hashed lock. Everything downstream reads the lock, never the
//! sources. Thus a check result depends on a hash that a reviewer can see in a
//! diff." [Spec 7](../../../docs/spec/07-distribution-and-federation.md#consuming)
//! adds the posture: "The lock is committed. Thus the corpus is checked against a
//! resolved, reviewable, reproducible taxonomy, and CI needs no network to check
//! anything."
//!
//! # What makes two resolutions the same result
//!
//! [13 — Open obligations](../../../docs/spec/13-open-obligations.md) filed this
//! question against the lock, and it is the one decision this crate makes rather
//! than implements. Spec 2 requires that any legal order of an overlay set gives
//! "the same resolved taxonomy", and it writes the result to a lock with a
//! content hash over it. A mapping records the order its entries arrived in, so
//! two orders that agree on every declaration can disagree on the text, and a
//! hash over the text would then report a difference that no check can see.
//!
//! **The identity is the canonical text, and the canonical writer is what makes
//! that identity the same one as the tree's.** Three reasons, and the third is
//! the one that decides it.
//!
//! - The lock is read by a person in a diff. An identity that a reader cannot
//!   compute from the artifact in front of them is an identity that fails at the
//!   moment it matters, which is a review.
//! - `headwater_resolve::render` already normalizes both differences that a tree
//!   identity would have to normalize away. Key order is canonical, by the rule
//!   the resolver states: a key the base declares keeps its position, and a key
//!   an overlay contributes follows, sorted. Scalar style is canonical, because
//!   a scalar is written plain where every character allows it and double-quoted
//!   otherwise, whatever style it was authored in.
//! - A tree identity needs a canonical serialization of the tree to hash
//!   anyway. There is no third thing to hash. So a tree identity is this
//!   identity with the artifact hidden from the reader.
//!
//! The claim that makes the two coincide is testable, and `fixtures/` tests it:
//! over every case the resolver holds, every permutation of the overlay set
//! produces one digest. That is spec 2's confluence requirement stated in the
//! coordinate the lock actually uses.
//!
//! # The lock is where "no partial load" is enforced
//!
//! Spec 2 rules that the engine never applies a taxonomy that does not validate.
//! [`write`] runs every rule of `taxonomy validate` and returns the findings
//! rather than a lock when any of them fires. So a lock is by construction a
//! validated taxonomy, and everything downstream reads a lock. The rule is
//! carried by the artifact rather than by a call that a caller may forget.
//!
//! # What this deliberately does not carry
//!
//! Spec 2 says the lock "records the taxonomy version and the measured
//! compatibility result that each corpus was validated against", and
//! [spec 7](../../../docs/spec/07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states)
//! puts the migration state in it: from-version, to-version, an owner, an expiry
//! and the open task list. Neither is here. Compatibility is measured by
//! `taxonomy diff`, and a migration payload is written by `taxonomy migrate`.
//! Both are [#78](https://github.com/headwater-ai/headwater/issues/78), and a
//! field written now would be a claim that no run produces and no test can fail.
//! The lock version below is what lets #78 add them.

pub mod sha256;

use headwater_resolve::{Resolution, ResolveError, ResolveErrorKind, Source};
use headwater_yaml::{Mapping, Span};
use std::path::{Path, PathBuf};

/// Where a repository keeps its lock, beside the consumer declaration that says
/// what to resolve.
pub const LOCK: &str = ".headwater/taxonomy.lock";

/// The format of the lock file. A reader that meets a later one says so rather
/// than guessing, and [#78](https://github.com/headwater-ai/headwater/issues/78)
/// is what raises it next.
pub const FORMAT: u32 = 1;

/// One lock, read or about to be written.
#[derive(Clone, Debug)]
pub struct Lock {
    /// The package the consumer took, and the version it pinned.
    pub package: String,
    pub version: String,
    /// Every source of the resolution, in application order, with the digest of
    /// the bytes each one had.
    pub sources: Vec<SourceDigest>,
    /// The digest of the canonical taxonomy text. This is the number that
    /// [spec 6](../../../docs/spec/06-engine-architecture.md#ci-adapters) means
    /// by "the taxonomy lock hash".
    pub digest: String,
    /// The resolved taxonomy itself.
    pub taxonomy: Mapping,
}

/// One source and the digest of its bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceDigest {
    pub path: String,
    pub digest: String,
}

/// What a lock refuses to be read as.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LockError {
    /// The file is not there. Every downstream verb needs one, and the message
    /// names the verb that writes it.
    Absent(PathBuf),
    Unreadable(String),
    Malformed(String),
    /// A later format than this engine knows.
    Format {
        found: String,
    },
    /// The digest does not match the taxonomy beside it, so somebody edited one
    /// of the two by hand.
    Tampered {
        declared: String,
        actual: String,
    },
}

impl std::fmt::Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockError::Absent(path) => write!(
                f,
                "{} is not there. Everything downstream reads the lock and never the sources, \
                 so run `headwater taxonomy resolve` to write one",
                path.display()
            ),
            LockError::Unreadable(why) => write!(f, "the lock cannot be read: {why}"),
            LockError::Malformed(what) => write!(f, "the lock is malformed: {what}"),
            LockError::Format { found } => write!(
                f,
                "the lock declares format `{found}` and this engine writes {FORMAT}. \
                 A newer engine wrote it"
            ),
            LockError::Tampered { declared, actual } => write!(
                f,
                "the lock declares the digest {declared} and its taxonomy hashes to {actual}. \
                 One of the two was edited by hand. Run `headwater taxonomy resolve`"
            ),
        }
    }
}

/// The digest of a canonical taxonomy text.
///
/// The argument is the text that [`Resolution::render`] produces, and never a
/// lock file, an indented copy, or a source. One function, so that the number in
/// a lock and the number a verifier computes cannot come from two readings.
pub fn digest(canonical: &str) -> String {
    format!("sha256:{}", sha256::hex(canonical.as_bytes()))
}

/// Validate a resolution and render the lock file it produces.
///
/// The findings come back instead of a lock when a rule of `taxonomy validate`
/// fires, because a lock that carried an invalid taxonomy would be a taxonomy
/// that everything downstream applies and nothing validated.
pub fn write(
    package: &str,
    version: &str,
    sources: &[Source],
    resolution: &Resolution,
) -> Result<String, Vec<ResolveError>> {
    let findings = resolution.validate();
    if !findings.is_empty() {
        return Err(findings);
    }
    let canonical = resolution.render();
    let lock = Lock {
        package: package.to_string(),
        version: version.to_string(),
        sources: sources
            .iter()
            .map(|source| SourceDigest {
                path: source.name.clone(),
                digest: digest_of_bytes(source.text.as_bytes()),
            })
            .collect(),
        digest: digest(&canonical),
        taxonomy: resolution.taxonomy.clone(),
    };
    Ok(render(&lock, &canonical))
}

/// Read a lock, and verify its digest against the taxonomy it carries.
pub fn read(text: &str) -> Result<Lock, LockError> {
    let root = headwater_yaml::load(text)
        .map_err(|errors| LockError::Unreadable(headwater_yaml::error::render(&errors)))?;
    let map = root
        .value
        .as_map()
        .ok_or_else(|| LockError::Malformed("the root is not a mapping".to_string()))?;

    let header = map
        .get("lock")
        .and_then(|node| node.value.as_map())
        .ok_or_else(|| LockError::Malformed("no `lock` block".to_string()))?;

    let format = text_of(header, "format").unwrap_or_default();
    if format != FORMAT.to_string() {
        return Err(LockError::Format {
            found: format.to_string(),
        });
    }

    let taxonomy = map
        .get("resolved")
        .and_then(|node| node.value.as_map())
        .ok_or_else(|| LockError::Malformed("no `resolved` block".to_string()))?
        .clone();

    let declared = text_of(header, "digest")
        .ok_or_else(|| LockError::Malformed("the `lock` block declares no digest".to_string()))?;
    let actual = digest(&headwater_resolve::render::render(&taxonomy));
    if declared != actual {
        return Err(LockError::Tampered {
            declared: declared.to_string(),
            actual,
        });
    }

    let sources = header
        .get("sources")
        .and_then(|node| node.value.as_seq())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.value.as_map())
                .filter_map(|entry| {
                    Some(SourceDigest {
                        path: text_of(entry, "path")?.to_string(),
                        digest: text_of(entry, "digest")?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(Lock {
        package: text_of(header, "package").unwrap_or_default().to_string(),
        version: text_of(header, "version").unwrap_or_default().to_string(),
        sources,
        digest: declared.to_string(),
        taxonomy,
    })
}

/// Read the lock of a repository.
pub fn at(root: &Path) -> Result<Lock, LockError> {
    let path = root.join(LOCK);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(LockError::Absent(path))
        }
        Err(error) => {
            return Err(LockError::Unreadable(format!(
                "{}: {error}",
                path.display()
            )))
        }
    };
    read(&text)
}

/// The lock as a file.
///
/// The header is a mapping and the body is the canonical taxonomy indented under
/// one key. Two properties are worth the shape. The file loads on the dialect
/// every other source is on, so nothing needs a second reader. And the taxonomy
/// reads in a diff exactly as its sources do, line for line, which is what
/// "reviewable in a diff" asks for.
fn render(lock: &Lock, canonical: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "\
# This file is generated by `headwater taxonomy resolve`. Do not edit it.
#
# It is the resolved taxonomy of {} {}, with a digest over the canonical text
# below. Everything downstream reads this file and never the sources, so a check
# result depends on a hash that a reviewer sees in this diff.
#
# `headwater taxonomy resolve --check` fails when the sources no longer produce
# this file, which is what makes a stale lock a red build rather than a quiet
# disagreement.

lock:
  format: {FORMAT}
  digest: {}
  package: {}
  version: {}
  sources:
",
        lock.package, lock.version, lock.digest, lock.package, lock.version
    ));
    for source in &lock.sources {
        out.push_str(&format!(
            "    - path: {}\n      digest: {}\n",
            source.path, source.digest
        ));
    }
    out.push_str("\n# The resolved taxonomy. The digest above is over this text with the two\n");
    out.push_str("# leading spaces of each line removed, which is the form the resolver writes\n");
    out.push_str("# and the form a round trip loads back.\n\nresolved:\n");
    for line in canonical.lines() {
        if line.is_empty() {
            out.push('\n');
        } else {
            out.push_str(&format!("  {line}\n"));
        }
    }
    out
}

/// The digest of a source file's bytes, which is a different question from the
/// digest of the canonical taxonomy and is written the same way.
fn digest_of_bytes(bytes: &[u8]) -> String {
    format!("sha256:{}", sha256::hex(bytes))
}

fn text_of<'a>(map: &'a Mapping, key: &str) -> Option<&'a str> {
    map.get(key)
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.as_str())
}

/// A lock failure as a resolution error, so a caller that already renders one
/// list renders one list.
pub fn as_error(error: &LockError) -> ResolveError {
    ResolveError::new(
        ResolveErrorKind::SourceRefused(error.to_string()),
        LOCK,
        "",
        Span::default(),
    )
}

/// Whether a lock still describes what its sources resolve to.
///
/// The comparison is over the digest of the canonical text and never over the
/// file, so a change to the header comment above is not a stale lock and a
/// change to one declaration is.
pub fn matches(lock: &Lock, resolution: &Resolution) -> bool {
    lock.digest == digest(&resolution.render())
}

/// The taxonomy a lock carries, as the value a caller passes downstream.
impl Lock {
    pub fn taxonomy(&self) -> &Mapping {
        &self.taxonomy
    }

    /// The canonical text this lock's digest is over.
    pub fn canonical(&self) -> String {
        headwater_resolve::render::render(&self.taxonomy)
    }

    /// The sources whose bytes have changed since the lock was written.
    ///
    /// This reads the source files and never resolves them, so it is the cheap
    /// half of `--check`: it names which file moved. A run that needs to know
    /// whether the *result* moved resolves and compares digests.
    pub fn moved(&self, root: &Path) -> Vec<String> {
        self.sources
            .iter()
            .filter(|source| match std::fs::read(root.join(&source.path)) {
                Ok(bytes) => digest_of_bytes(&bytes) != source.digest,
                Err(_) => true,
            })
            .map(|source| source.path.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolved(source: &str) -> (Vec<Source>, Resolution) {
        let sources =
            vec![
                Source::from_text("base.yml", headwater_resolve::Role::Taxonomy, source)
                    .expect("the source loads"),
            ];
        let resolution = headwater_resolve::resolve(&sources).expect("it resolves");
        (sources, resolution)
    }

    const VALID: &str = "\
taxonomy: acme/fixture
version: 1.0.0
purposes:
  rationale: {intent: explain why a choice was made and what it forecloses}
facets:
  status:
    role: state
    values: [draft, current]
    required: true
    volatility: mutable
    guidance: {draft: it is being argued over, current: it states what holds now}
kinds:
  decision:
    purpose: rationale
    facets: {require: [status]}
shelves:
  decisions: {path: \"docs/decisions/**\", homogeneous: true, kind: decision}
core:
  requires:
    - facet_role: state
";

    #[test]
    fn a_lock_reads_back_to_the_taxonomy_it_was_written_from() {
        let (sources, resolution) = resolved(VALID);
        let text = write("acme/fixture", "1.0.0", &sources, &resolution).expect("it validates");
        let lock = read(&text).expect("the lock reads");
        assert_eq!(lock.canonical(), resolution.render());
        assert_eq!(lock.digest, digest(&resolution.render()));
        assert!(matches(&lock, &resolution));
    }

    #[test]
    fn an_edit_to_the_taxonomy_inside_the_lock_is_caught() {
        let (sources, resolution) = resolved(VALID);
        let text = write("acme/fixture", "1.0.0", &sources, &resolution).expect("it validates");
        // The edit a reviewer would never see: one word in the body, and the
        // digest left alone.
        let tampered = text.replace("homogeneous: true", "homogeneous: false");
        assert!(tampered != text);
        assert!(matches!(read(&tampered), Err(LockError::Tampered { .. })));
    }

    #[test]
    fn an_edit_to_the_header_comment_is_not_a_stale_lock() {
        let (sources, resolution) = resolved(VALID);
        let text = write("acme/fixture", "1.0.0", &sources, &resolution).expect("it validates");
        let commented = format!("# somebody added a note\n{text}");
        let lock = read(&commented).expect("the lock still reads");
        assert!(matches(&lock, &resolution));
    }

    #[test]
    fn a_taxonomy_that_does_not_validate_produces_findings_and_no_lock() {
        // `audience` is declared and nothing reads it, which is the relevance
        // canon. The taxonomy resolves, so only validation can refuse it.
        let source = VALID.replace(
            "kinds:",
            "  audience: {values: [internal], volatility: stable, guidance: {internal: here}}\nkinds:",
        );
        let (sources, resolution) = resolved(&source);
        let refused = write("acme/fixture", "1.0.0", &sources, &resolution);
        assert!(
            refused.is_err(),
            "a lock was written for an invalid taxonomy"
        );
    }

    #[test]
    fn the_body_of_a_lock_is_the_canonical_text_indented_and_nothing_else() {
        let (sources, resolution) = resolved(VALID);
        let text = write("acme/fixture", "1.0.0", &sources, &resolution).expect("it validates");
        let mut body = String::new();
        for line in text.lines().skip_while(|line| *line != "resolved:").skip(1) {
            body.push_str(line.strip_prefix("  ").unwrap_or(line));
            body.push('\n');
        }
        assert_eq!(body, resolution.render());
    }

    #[test]
    fn a_lock_from_a_later_engine_says_so_rather_than_guessing() {
        let (sources, resolution) = resolved(VALID);
        let text = write("acme/fixture", "1.0.0", &sources, &resolution).expect("it validates");
        let later = text.replace(&format!("format: {FORMAT}"), "format: 9");
        assert!(matches!(read(&later), Err(LockError::Format { .. })));
    }

    #[test]
    fn a_moved_source_is_named_and_an_untouched_one_is_not() {
        let (sources, resolution) = resolved(VALID);
        let text = write("acme/fixture", "1.0.0", &sources, &resolution).expect("it validates");
        let lock = read(&text).expect("the lock reads");
        // The source is `base.yml` relative to a root that holds no such file,
        // so it reads as moved. That is the honest answer for a file that is
        // not there: the lock cannot claim it still agrees.
        assert_eq!(lock.moved(Path::new("/nonexistent")), vec!["base.yml"]);
    }
}
