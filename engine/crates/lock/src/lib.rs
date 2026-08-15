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
//! # The adoption payload rides here, and it is the one authored part
//!
//! [Spec 7](../../../docs/spec/07-distribution-and-federation.md#between-majors-the-corpus-is-legitimately-between-valid-states)
//! puts the migration state in the lock: from-version, to-version, an owner, an
//! expiry and the open task list. [Q12](../../../docs/spec/09-decisions.md#q12--migration-path-for-an-existing-corpus)
//! makes first contact the same state with the from-version absent, and it says
//! where the payload lives in as many words: "the payload is large on a large
//! corpus, and it sits in the lock, which is committed and reviewed".
//!
//! So [`Lock::adoption`] carries it and [`FORMAT`] is 2. The bump is not
//! decoration. A payload moves a finding out of the report, so an engine that
//! reads the block and an engine that skips it disagree about one corpus. An
//! engine that cannot honor a payload has to refuse the lock rather than report
//! a louder verdict than the adopter agreed to.
//!
//! **The block is authored and every other line of this file is generated.**
//! The rest of the lock is a function of the sources. An owner is not, an expiry
//! is not, and a task that closes is a person deleting lines. `taxonomy resolve`
//! therefore reads the committed lock and carries the block through unchanged.
//! It is written in the same canonical form as the taxonomy body, so a hand edit
//! in another style normalizes on the next resolve exactly as a hand edit to the
//! body does. The digest does not cover it, because the digest is the identity
//! of a resolution and a payload is not part of one.
//!
//! # What this still does not carry
//!
//! Spec 2 says the lock "records the taxonomy version and the measured
//! compatibility result that each corpus was validated against". That is not
//! here. Compatibility is measured by `taxonomy diff`, which is
//! [#78](https://github.com/headwater-ai/headwater/issues/78), and a field
//! written now would be a claim that no run produces and no test can fail.
//! `adoption.from` is absent for the same reason. `headwater infer` writes a
//! payload against no prior version, and `taxonomy migrate` is the verb that
//! would fill that field in.

use headwater_resolve::{Resolution, ResolveError, ResolveErrorKind, Source};
use headwater_yaml::{Mapping, Span};
use std::path::{Path, PathBuf};

/// Where a repository keeps its lock, beside the consumer declaration that says
/// what to resolve.
pub const LOCK: &str = ".headwater/taxonomy.lock";

/// The format of the lock file. A reader that meets a later one says so rather
/// than guessing.
///
/// 1 carried a resolution alone. 2 admits [`Lock::adoption`], and the bump is
/// the refusal an older engine owes an adopter: a payload it cannot read is a
/// set of findings it would report that the adopter already accounted for.
pub const FORMAT: u32 = 2;

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
    /// The adoption payload, as it was written, or `None` where the file
    /// declares no `adoption` block.
    ///
    /// This crate reads the block as a mapping and validates nothing in it.
    /// A pair names a rule, and the set of rules is the check layer's to state,
    /// so `headwater_check::adoption` is where a payload is read into tasks and
    /// where a refusal is reported. A second opinion about a rule name here
    /// would be a second place to change when a rule is added.
    pub adoption: Option<Mapping>,
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
    headwater_hash::digest(canonical.as_bytes())
}

/// Validate a resolution and render the lock file it produces.
///
/// The findings come back instead of a lock when a rule of `taxonomy validate`
/// fires, because a lock that carried an invalid taxonomy would be a taxonomy
/// that everything downstream applies and nothing validated.
///
/// `adoption` is the payload to carry through. A caller that is rewriting a
/// lock passes the block the committed one held, because a resolve that dropped
/// it would delete an adopter's accounting as a side effect of a taxonomy edit.
pub fn write(
    package: &str,
    version: &str,
    sources: &[Source],
    resolution: &Resolution,
    adoption: Option<&Mapping>,
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
                digest: headwater_hash::digest(source.text.as_bytes()),
            })
            .collect(),
        digest: digest(&canonical),
        taxonomy: resolution.taxonomy.clone(),
        adoption: adoption.cloned(),
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

    // A block that is present and is not a mapping is malformed rather than
    // absent. The two read the same downstream and mean opposite things: one
    // adopter declared no debt, the other wrote a payload this cannot see.
    let adoption = match map.get("adoption") {
        None => None,
        Some(node) => match node.value.as_map() {
            Some(inner) => Some(inner.clone()),
            None => {
                return Err(LockError::Malformed(format!(
                    "the `adoption` block is {} rather than a mapping",
                    node.value.kind_name()
                )))
            }
        },
    };

    Ok(Lock {
        package: text_of(header, "package").unwrap_or_default().to_string(),
        version: text_of(header, "version").unwrap_or_default().to_string(),
        sources,
        digest: declared.to_string(),
        taxonomy,
        adoption,
    })
}

/// The payload the committed lock carries, for a rewrite to carry through.
///
/// A rewrite of the lock is a rewrite of a resolution, and the payload is not
/// part of one. Every caller that writes a lock over an existing one passes
/// this, so the rule lives in one place instead of in each caller.
///
/// A lock that will not read carries nothing. The three ways that happens are a
/// first resolve with no lock at all, a lock this engine is too old to read, and
/// a lock somebody broke. In all three the caller is about to write a correct
/// one, and refusing here would only move the report to the wrong verb.
pub fn adoption_at(root: &Path) -> Option<Mapping> {
    at(root).ok().and_then(|lock| lock.adoption)
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
# This file is generated by `headwater taxonomy resolve`. Do not edit it, with
# one exception named below.
#
# It is the resolved taxonomy of {} {}, with a digest over the canonical text
# below. Everything downstream reads this file and never the sources, so a check
# result depends on a hash that a reviewer sees in this diff.
#
# The exception is the `adoption` block, where one appears. That block is
# authored, and `headwater taxonomy resolve` carries it through rather than
# producing it. It is the only part of this file that is not a function of the
# sources listed under `sources` below.
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
    if let Some(adoption) = &lock.adoption {
        out.push_str(
            "
# The adoption payload: the debt this corpus declared when the taxonomy first
# reached it. Every other line of this file is generated and this block is not.
# `headwater infer` writes it, a person edits it, and `headwater taxonomy
# resolve` carries it through untouched. Each task names an owner and an expiry,
# and holds the `(document, rule)` pairs it accounts for. A pair that stops
# failing is a task shrinking, and `headwater check` reports the count that
# remains on every run.

adoption:
",
        );
        for line in headwater_resolve::render::render(adoption).lines() {
            if line.is_empty() {
                out.push('\n');
            } else {
                out.push_str(&format!("  {line}\n"));
            }
        }
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
                Ok(bytes) => headwater_hash::digest(&bytes) != source.digest,
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
        let text =
            write("acme/fixture", "1.0.0", &sources, &resolution, None).expect("it validates");
        let lock = read(&text).expect("the lock reads");
        assert_eq!(lock.canonical(), resolution.render());
        assert_eq!(lock.digest, digest(&resolution.render()));
        assert!(matches(&lock, &resolution));
    }

    #[test]
    fn an_edit_to_the_taxonomy_inside_the_lock_is_caught() {
        let (sources, resolution) = resolved(VALID);
        let text =
            write("acme/fixture", "1.0.0", &sources, &resolution, None).expect("it validates");
        // The edit a reviewer would never see: one word in the body, and the
        // digest left alone.
        let tampered = text.replace("homogeneous: true", "homogeneous: false");
        assert_ne!(tampered, text);
        assert!(matches!(read(&tampered), Err(LockError::Tampered { .. })));
    }

    #[test]
    fn an_edit_to_the_header_comment_is_not_a_stale_lock() {
        let (sources, resolution) = resolved(VALID);
        let text =
            write("acme/fixture", "1.0.0", &sources, &resolution, None).expect("it validates");
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
        let refused = write("acme/fixture", "1.0.0", &sources, &resolution, None);
        assert!(
            refused.is_err(),
            "a lock was written for an invalid taxonomy"
        );
    }

    #[test]
    fn the_body_of_a_lock_is_the_canonical_text_indented_and_nothing_else() {
        let (sources, resolution) = resolved(VALID);
        let text =
            write("acme/fixture", "1.0.0", &sources, &resolution, None).expect("it validates");
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
        let text =
            write("acme/fixture", "1.0.0", &sources, &resolution, None).expect("it validates");
        let later = text.replace(&format!("format: {FORMAT}"), "format: 9");
        assert!(matches!(read(&later), Err(LockError::Format { .. })));
    }

    #[test]
    fn a_moved_source_is_named_and_an_untouched_one_is_not() {
        let (sources, resolution) = resolved(VALID);
        let text =
            write("acme/fixture", "1.0.0", &sources, &resolution, None).expect("it validates");
        let lock = read(&text).expect("the lock reads");
        // The source is `base.yml` relative to a root that holds no such file,
        // so it reads as moved. That is the honest answer for a file that is
        // not there: the lock cannot claim it still agrees.
        assert_eq!(lock.moved(Path::new("/nonexistent")), vec!["base.yml"]);
    }

    /// A payload written into a lock reads back as the mapping it was.
    ///
    /// The property that matters downstream is not that the text survives but
    /// that the tree does. `headwater_check::adoption` reads the mapping, so a
    /// render that lost a task or reordered a pair would move a verdict.
    #[test]
    fn an_adoption_payload_round_trips_through_a_lock() {
        let (sources, resolution) = resolved(VALID);
        let payload = headwater_yaml::load(
            "\
to: acme/fixture 1.0.0
tasks:
  - id: AD-1
    statement: every document states no summary
    owner: the docs guild
    until: 2027-01-01
    pairs:
      - {path: docs/a.md, rule: facet.required.missing}
      - {path: docs/b.md, rule: facet.required.missing}
",
        )
        .expect("the payload loads");
        let payload = payload.value.as_map().expect("it is a mapping").clone();

        let text = write(
            "acme/fixture",
            "1.0.0",
            &sources,
            &resolution,
            Some(&payload),
        )
        .expect("it validates");
        let lock = read(&text).expect("the lock reads");
        let back = lock.adoption.expect("the payload survived");
        assert_eq!(
            headwater_resolve::render::render(&back),
            headwater_resolve::render::render(&payload)
        );
    }

    /// The digest is the identity of a resolution, and a payload is not part of
    /// one.
    ///
    /// Two locks over the same sources, one carrying debt and one not, describe
    /// the same taxonomy. A digest that moved would make `resolve --check` red
    /// for a reviewer who changed no source, and would make the number in the
    /// lock mean two things.
    #[test]
    fn a_payload_does_not_move_the_digest() {
        let (sources, resolution) = resolved(VALID);
        let payload = headwater_yaml::load("tasks: []\n").expect("it loads");
        let payload = payload.value.as_map().expect("a mapping").clone();

        let bare = read(&write("acme/fixture", "1.0.0", &sources, &resolution, None).expect("ok"))
            .expect("reads");
        let laden = read(
            &write(
                "acme/fixture",
                "1.0.0",
                &sources,
                &resolution,
                Some(&payload),
            )
            .expect("ok"),
        )
        .expect("reads");
        assert_eq!(bare.digest, laden.digest);
        assert!(bare.adoption.is_none());
        assert!(laden.adoption.is_some());
    }

    /// An `adoption` key that is not a mapping is refused rather than skipped.
    ///
    /// Absent and unreadable mean opposite things. One adopter declared no
    /// debt, the other wrote a payload this engine cannot see, and reading the
    /// second as the first reports findings they already accounted for.
    #[test]
    fn an_adoption_block_that_is_not_a_mapping_is_refused() {
        let (sources, resolution) = resolved(VALID);
        let text = write("acme/fixture", "1.0.0", &sources, &resolution, None).expect("ok");
        let broken = text.replace("\nresolved:\n", "\nadoption: []\n\nresolved:\n");
        assert!(matches!(read(&broken), Err(LockError::Malformed(_))));
    }
}
