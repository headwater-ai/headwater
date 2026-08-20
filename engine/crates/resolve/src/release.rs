// SPDX-License-Identifier: Apache-2.0
//! A published package, and the check a consumer runs over one that arrived
//! from somewhere else.
//!
//! [Spec 7](../../../../docs/spec/07-distribution-and-federation.md#publishing):
//! "Publishing is a release: a semantic version, a changelog, an integrity
//! digest, and a migration payload for any major bump. Distribution is over the
//! registry or repository that the organization already uses. The engine
//! requires only that it can fetch a version and check its digest." This module
//! is the digest half of that sentence, and the paragraphs below say what the
//! digest proves and what it does not.
//!
//! # The engine never fetches, and that is a structural guarantee
//!
//! [Spec 0](../../../../docs/spec/00-vision-and-scope.md#non-negotiables)
//! forbids a network dependency at check time, and no crate of this engine
//! depends on the network. `headwater taxonomy vendor` takes the path of a
//! directory that the caller already fetched, by whatever moves a directory in
//! the organization it runs in. So the guarantee is a property of the verb
//! rather than a rule anybody polices: a verb that takes a path opens no socket,
//! and there is no code path here that could.
//!
//! # What the digest proves, and what it does not
//!
//! **It proves that the artifact in front of the engine is the artifact the
//! consumer pinned.** The pin is a digest written into
//! `.headwater/taxonomy.yml`, which is authored, committed and read in a diff.
//! `vendor` recomputes the digest from the bytes on disk and refuses when the
//! two differ, naming each file that moved. An artifact that a proxy, a mirror
//! or a rebuilt release changed between the pin and the use is refused.
//!
//! **It does not prove that the record's own header is the publisher's.** The
//! digest is over the member list and [`members`] skips the record itself, so
//! the header lines above the digest — the format, the package, the version and
//! the engine range — are covered by nothing. Spec 7 states that non-coverage as
//! the design and it cannot be otherwise while the digest field sits inside the
//! file that would be hashed. What makes it harmless is that `package.yml` is
//! inside the walk, so the artifact carries an authenticated declaration of what
//! it is: [`crate::package::vendor`] reads the name and the version out of that
//! and refuses a header disagreeing with it, rather than believing the header.
//! Until [#297](https://github.com/headwater-ai/headwater/issues/297) it
//! believed the header, and the header named the directory it deleted.
//!
//! **It does not prove who published the artifact.** A digest with no signature
//! over it authenticates the pin and never the publisher, so the first fetch —
//! the one that produced the pin — rests on whatever channel carried the digest.
//! That channel is the publisher's release notes or its registry, and it is
//! outside this engine. A consumer who copies the digest out of the artifact
//! they just downloaded has pinned nothing, which is why `vendor` refuses to
//! run with no pin instead of recording what it found.
//!
//! Nothing here is a signature, and closing that gap needs a key, a channel that
//! distributes the key, and a revocation story. None of the three is decided,
//! and the obligation record says so rather than a field named `signature` that
//! holds a digest.
//!
//! # Why the in-house SHA-256 and not a vetted crate
//!
//! `headwater_hash` states the limit of its own argument: every other digest
//! this engine writes is an integrity mark inside a repository that already
//! holds the bytes, and this one is a check on an artifact from elsewhere. The
//! decision to reuse it anyway is [Q22](../../../../docs/spec/09-decisions.md#q22--the-integrity-posture-of-a-published-package),
//! and it rests on what the two options actually buy.
//!
//! A vetted implementation buys a correct SHA-256. It does not buy
//! authentication, because the design above gets its strength from where the
//! expected digest comes from rather than from who wrote the compression
//! function. What the adversary has to produce is a second preimage of SHA-256,
//! and the defense against that is the algorithm.
//!
//! So the risk that a dependency would answer is an implementation defect, and
//! that risk is answerable by evidence. `headwater_hash` carries the published
//! FIPS 180-4 vectors and every length around a block boundary, and
//! `headwater_hash`'s `tests/oracle.rs` holds sixteen further subjects against
//! `sha256sum`, which is an implementation nobody here wrote. A cross-check
//! against a second implementation measures the property a vetted crate would
//! only assert.

use crate::error::{ResolveError, ResolveErrorKind};
use headwater_yaml::Mapping;
use std::path::{Path, PathBuf};

/// The file a published package carries at its root.
pub const RECORD: &str = "release.yml";

/// The format of that file. A reader that meets a later one says so rather than
/// guessing, on the same terms the lock takes.
pub const FORMAT: u32 = 1;

/// The version of this engine, which is what a `requires_engine` range is read
/// against.
///
/// A range is unreadable without a version to compare it to, and until this
/// module there was none: the workspace carried the placeholder `0.0.0` and no
/// verb printed it. The number below is the tag this repository published.
pub const ENGINE: &str = env!("CARGO_PKG_VERSION");

/// One published package: what it is, and the digest of every file in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    pub package: String,
    pub version: String,
    /// The engine range the manifest declares, where it declares one.
    pub requires_engine: Option<String>,
    /// The digest over the member list, which is the number a consumer pins.
    pub digest: String,
    /// Every file in the artifact except the record itself, sorted by path.
    pub members: Vec<Member>,
}

/// One file of a published package, and the digest of its bytes.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Member {
    /// The path inside the artifact, with `/` separators on every platform.
    pub path: String,
    pub digest: String,
}

/// What a release record refuses to be read as, or an artifact refuses to be.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReleaseError {
    /// The directory carries no record, so it is not a published package.
    Absent(PathBuf),
    Unreadable(String),
    Malformed(String),
    /// A later format than this engine knows.
    Format {
        found: String,
    },
    /// The record's own digest does not cover the member list beside it, so the
    /// record was edited.
    RecordMoved {
        declared: String,
        actual: String,
    },
    /// The artifact is not the one the consumer pinned.
    NotPinned {
        pinned: String,
        actual: String,
    },
    /// The artifact does not match its own record. Each entry names one file.
    Diverged(Vec<Divergence>),
    /// The package declares an engine range this engine is outside of.
    Engine {
        package: String,
        range: String,
        engine: String,
    },
    /// A range that this engine cannot read at all.
    Range {
        range: String,
        why: String,
    },
}

/// One way an artifact fails to be what its record says it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Divergence {
    /// The file is there and its bytes are not the published ones.
    Changed {
        path: String,
        published: String,
        actual: String,
    },
    /// The record names the file and the artifact does not carry it.
    Missing(String),
    /// The artifact carries the file and the record names no such member.
    Unnamed(String),
}

impl std::fmt::Display for Divergence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Divergence::Changed {
                path,
                published,
                actual,
            } => write!(
                f,
                "{path} was published as {published} and these bytes are {actual}"
            ),
            Divergence::Missing(path) => {
                write!(
                    f,
                    "{path} is named by the record and is not in the artifact"
                )
            }
            Divergence::Unnamed(path) => {
                write!(
                    f,
                    "{path} is in the artifact and the record names no such member"
                )
            }
        }
    }
}

impl std::fmt::Display for ReleaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReleaseError::Absent(path) => write!(
                f,
                "{} carries no {RECORD}, so it is not a published package. \
                 `headwater taxonomy publish` writes one",
                path.display()
            ),
            ReleaseError::Unreadable(why) => write!(f, "the release record cannot be read: {why}"),
            ReleaseError::Malformed(what) => write!(f, "the release record is malformed: {what}"),
            ReleaseError::Format { found } => write!(
                f,
                "the release record declares format `{found}` and this engine writes {FORMAT}. \
                 A newer engine published it"
            ),
            ReleaseError::RecordMoved { declared, actual } => write!(
                f,
                "the release record declares the digest {declared} and its own member list \
                 hashes to {actual}. The record was edited after it was written"
            ),
            ReleaseError::NotPinned { pinned, actual } => write!(
                f,
                "this repository pins {pinned} and the artifact is {actual}. \
                 The pin is what a fetch is checked against, so the artifact is refused"
            ),
            ReleaseError::Diverged(entries) => {
                write!(
                    f,
                    "the artifact is not what its own record says it is, in {} place{}:",
                    entries.len(),
                    if entries.len() == 1 { "" } else { "s" }
                )?;
                for entry in entries {
                    write!(f, "\n  {entry}")?;
                }
                Ok(())
            }
            ReleaseError::Engine {
                package,
                range,
                engine,
            } => write!(
                f,
                "{package} requires an engine in `{range}` and this engine is {engine}"
            ),
            ReleaseError::Range { range, why } => {
                write!(f, "`requires_engine: \"{range}\"` cannot be read: {why}")
            }
        }
    }
}

/// A release failure as a resolution error, so a caller that renders one list
/// renders one list.
pub fn as_error(source: &str, error: &ReleaseError) -> Vec<ResolveError> {
    vec![ResolveError::new(
        ResolveErrorKind::SourceRefused(error.to_string()),
        source,
        "",
        headwater_yaml::Span::default(),
    )]
}

/// The canonical text the release digest is over.
///
/// One line per member, path and digest separated by a tab, sorted by path, and
/// the record's own bytes are not in it. The digest is therefore over what the
/// artifact holds rather than over a file that carries the digest, which is the
/// self-reference a digest inside its own artifact would otherwise have.
pub fn canonical(members: &[Member]) -> String {
    let mut sorted = members.to_vec();
    sorted.sort();
    let mut out = String::new();
    for member in &sorted {
        out.push_str(&member.path);
        out.push('\t');
        out.push_str(&member.digest);
        out.push('\n');
    }
    out
}

/// The digest of a member list.
pub fn digest_of(members: &[Member]) -> String {
    headwater_hash::digest(canonical(members).as_bytes())
}

/// Every file under a directory, as members, sorted by path.
///
/// Nothing is skipped except the record itself. A directory that arrived with
/// the machinery of its transport attached — a checkout's `.git`, an editor's
/// backup — is not the directory that was published, and an artifact check that
/// looked away from part of the tree would be an artifact check with a hole in
/// it that the publisher never agreed to.
///
/// **The record is that hole, and this says what holds it.** The one skip here
/// is forced: the digest field is inside the file that would be hashed, so
/// covering the record needs a canonical form of the record with its own digest
/// elided, and adopting one moves every digest anybody has published. What the
/// skip does not reach is identity, because `package.yml` is inside this walk —
/// its presence and its bytes are both pinned. So the artifact declares what it
/// is in a file the digest covers, and
/// [`crate::package::vendor`] takes the name and the version from there and
/// refuses a header that disagrees
/// ([#297](https://github.com/headwater-ai/headwater/issues/297)).
pub fn members(dir: &Path) -> Result<Vec<Member>, ReleaseError> {
    let mut out = Vec::new();
    walk(dir, dir, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<Member>) -> Result<(), ReleaseError> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|error| ReleaseError::Unreadable(format!("{}: {error}", dir.display())))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    entries.sort();
    for entry in entries {
        if entry.is_dir() {
            walk(root, &entry, out)?;
            continue;
        }
        let relative = relative(root, &entry);
        if relative == RECORD {
            continue;
        }
        let bytes = std::fs::read(&entry)
            .map_err(|error| ReleaseError::Unreadable(format!("{}: {error}", entry.display())))?;
        out.push(Member {
            path: relative,
            digest: headwater_hash::digest(&bytes),
        });
    }
    Ok(())
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// The record a published package carries, rendered.
pub fn render(release: &Release) -> String {
    let mut out = String::new();
    out.push_str(
        "\
# This file is generated by `headwater taxonomy publish`. Do not edit it.
#
# It is the identity of one release: every file in this artifact, with the
# digest of its bytes, and one digest over that list. A consumer pins the digest
# below in `.headwater/taxonomy.yml` and `headwater taxonomy vendor` refuses an
# artifact that does not match it, naming each file that moved.
#
# The digest covers the files listed below it and not the lines above it. It
# cannot cover them, because it is one of them. So nothing reads this package's
# identity out of this header: `headwater taxonomy vendor` takes the name and
# the version from `package.yml`, which the digest does cover, and refuses an
# artifact whose header disagrees with it.
#
# The digest is an integrity check against the pin and it is not a signature. It
# says that these bytes are the bytes the pin was written for. It says nothing
# about who published them, so the channel that carried the digest to the
# consumer is what the first fetch rests on.

release:
",
    );
    out.push_str(&format!("  format: {FORMAT}\n"));
    out.push_str(&format!("  package: {}\n", release.package));
    out.push_str(&format!("  version: {}\n", release.version));
    if let Some(range) = &release.requires_engine {
        out.push_str(&format!("  requires_engine: \"{range}\"\n"));
    }
    out.push_str(&format!("  digest: {}\n", release.digest));
    out.push_str("  members:\n");
    for member in &release.members {
        out.push_str(&format!(
            "    - path: {}\n      digest: {}\n",
            member.path, member.digest
        ));
    }
    out
}

/// Read a release record.
pub fn read(text: &str) -> Result<Release, ReleaseError> {
    let root = headwater_yaml::load(text)
        .map_err(|errors| ReleaseError::Unreadable(headwater_yaml::error::render(&errors)))?;
    let map = root
        .value
        .as_map()
        .ok_or_else(|| ReleaseError::Malformed("the root is not a mapping".to_string()))?;
    let header = map
        .get("release")
        .and_then(|node| node.value.as_map())
        .ok_or_else(|| ReleaseError::Malformed("no `release` block".to_string()))?;

    let format = text_of(header, "format").unwrap_or_default();
    if format != FORMAT.to_string() {
        return Err(ReleaseError::Format {
            found: format.to_string(),
        });
    }

    let declared = text_of(header, "digest")
        .ok_or_else(|| ReleaseError::Malformed("the `release` block declares no digest".into()))?
        .to_string();

    let members: Vec<Member> = header
        .get("members")
        .and_then(|node| node.value.as_seq())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.value.as_map())
                .filter_map(|entry| {
                    Some(Member {
                        path: text_of(entry, "path")?.to_string(),
                        digest: text_of(entry, "digest")?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // The record's own digest is checked here rather than by a caller, because a
    // record that does not cover its own member list is a record that a caller
    // would then compare against an artifact and find agreeing.
    let actual = digest_of(&members);
    if declared != actual {
        return Err(ReleaseError::RecordMoved { declared, actual });
    }

    Ok(Release {
        package: text_of(header, "package").unwrap_or_default().to_string(),
        version: text_of(header, "version").unwrap_or_default().to_string(),
        requires_engine: text_of(header, "requires_engine").map(str::to_string),
        digest: declared,
        members,
    })
}

/// Read the record of a published package on disk.
pub fn at(dir: &Path) -> Result<Release, ReleaseError> {
    let path = dir.join(RECORD);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(ReleaseError::Absent(dir.to_path_buf()))
        }
        Err(error) => {
            return Err(ReleaseError::Unreadable(format!(
                "{}: {error}",
                path.display()
            )))
        }
    };
    read(&text)
}

/// Every way the artifact on disk differs from what its record says it is.
///
/// Three kinds, and the third is the one a per-file comparison alone would miss.
/// A changed file, a file the record names that is not there, and a file that is
/// there which the record names no member for. Without the third, an artifact
/// with a bundle added to it would pass every comparison the record can drive.
pub fn diverged(dir: &Path, release: &Release) -> Result<Vec<Divergence>, ReleaseError> {
    let actual = members(dir)?;
    let mut out = Vec::new();
    for published in &release.members {
        match actual.iter().find(|member| member.path == published.path) {
            None => out.push(Divergence::Missing(published.path.clone())),
            Some(found) if found.digest != published.digest => out.push(Divergence::Changed {
                path: published.path.clone(),
                published: published.digest.clone(),
                actual: found.digest.clone(),
            }),
            Some(_) => {}
        }
    }
    for found in &actual {
        if !release
            .members
            .iter()
            .any(|member| member.path == found.path)
        {
            out.push(Divergence::Unnamed(found.path.clone()));
        }
    }
    Ok(out)
}

/// Check a fetched artifact against the digest a consumer pinned.
///
/// The order is what makes each refusal say the right thing. The record is read
/// first, and reading it verifies that the record covers its own member list.
/// The pin is compared next, so an artifact that is the wrong release is refused
/// before anybody hears about a file. Then the bytes on disk are compared with
/// the record, which is the message that names what moved.
pub fn verify(dir: &Path, pinned: &str) -> Result<Release, ReleaseError> {
    let release = at(dir)?;
    if release.digest != pinned {
        return Err(ReleaseError::NotPinned {
            pinned: pinned.to_string(),
            actual: release.digest.clone(),
        });
    }
    let divergences = diverged(dir, &release)?;
    if !divergences.is_empty() {
        return Err(ReleaseError::Diverged(divergences));
    }
    engine_range(&release.package, release.requires_engine.as_deref())?;
    Ok(release)
}

/// Whether this engine is inside the range a package declares.
///
/// A package that declares no range is taken, and the absence is what
/// [spec 7](../../../../docs/spec/07-distribution-and-federation.md#publishing)
/// means by a publisher that states no floor. A range that this engine cannot
/// read is refused rather than ignored: an unreadable range read as no range is
/// a claim the publisher made and the consumer dropped.
pub fn engine_range(package: &str, range: Option<&str>) -> Result<(), ReleaseError> {
    let Some(range) = range else { return Ok(()) };
    match satisfies(range, ENGINE) {
        Err(why) => Err(ReleaseError::Range {
            range: range.to_string(),
            why,
        }),
        Ok(true) => Ok(()),
        Ok(false) => Err(ReleaseError::Engine {
            package: package.to_string(),
            range: range.to_string(),
            engine: ENGINE.to_string(),
        }),
    }
}

/// Whether a version satisfies a range.
///
/// A range is a space-separated list of comparators, in the form spec 7 writes:
/// `">=1.4 <2"`. Every comparator must hold. A version is one, two or three
/// numeric parts, and an absent part is zero, so `1.4` in a comparator means
/// `1.4.0` and `<2` excludes every `2.x`.
pub fn satisfies(range: &str, version: &str) -> Result<bool, String> {
    let target = parse(version)?;
    let mut comparators = 0;
    for term in range.split_whitespace() {
        comparators += 1;
        let (operator, rest) = split_operator(term);
        let bound = parse(rest)?;
        let held = match operator {
            ">=" => target >= bound,
            ">" => target > bound,
            "<=" => target <= bound,
            "<" => target < bound,
            "=" => target == bound,
            _ => return Err(format!("`{term}` states no comparison this engine reads")),
        };
        if !held {
            return Ok(false);
        }
    }
    if comparators == 0 {
        return Err("it states no comparison at all".to_string());
    }
    Ok(true)
}

fn split_operator(term: &str) -> (&str, &str) {
    for operator in [">=", "<=", ">", "<", "="] {
        if let Some(rest) = term.strip_prefix(operator) {
            return (operator, rest);
        }
    }
    ("=", term)
}

/// A version as three numbers.
///
/// Public because a caller that needs the major alone would otherwise write a
/// second reading of a version string, and this crate exists partly so that the
/// engine has one. [`satisfies`] is built on it, so a version this answers for
/// is a version a range can be asked about, and a version it refuses is refused
/// everywhere.
pub fn parts(version: &str) -> Result<(u64, u64, u64), String> {
    parse(version)
}

fn parse(version: &str) -> Result<(u64, u64, u64), String> {
    let mut parts = [0u64; 3];
    let trimmed = version.trim();
    if trimmed.is_empty() {
        return Err("a version with no digits in it".to_string());
    }
    let split: Vec<&str> = trimmed.split('.').collect();
    if split.len() > 3 {
        return Err(format!("`{trimmed}` has more than three parts"));
    }
    for (index, part) in split.iter().enumerate() {
        parts[index] = part.parse().map_err(|_| {
            format!("`{trimmed}` is not three numbers, and this engine reads no other form")
        })?;
    }
    Ok((parts[0], parts[1], parts[2]))
}

/// The release record a directory of package content produces.
///
/// The manifest supplies the identity, so a record this writes cannot disagree
/// with the package it describes about a name or a version.
///
/// **That is a property of the write and never of a record on disk.** The header
/// it produces sits outside the digest below it ([`members`]), so a record can
/// be edited afterwards to say anything at all and still verify against the
/// honest pin. Every reader of a record therefore has to hold the header to the
/// manifest itself rather than inherit the agreement from here, which is what
/// [`crate::package::vendor`] does
/// ([#297](https://github.com/headwater-ai/headwater/issues/297)).
pub fn compute(dir: &Path, manifest: &Mapping) -> Result<Release, ReleaseError> {
    let members = members(dir)?;
    Ok(Release {
        package: text_of(manifest, "package").unwrap_or_default().to_string(),
        version: text_of(manifest, "version").unwrap_or_default().to_string(),
        requires_engine: text_of(manifest, "requires_engine").map(str::to_string),
        digest: digest_of(&members),
        members,
    })
}

fn text_of<'a>(map: &'a Mapping, key: &str) -> Option<&'a str> {
    map.get(key)
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(path: &str, bytes: &[u8]) -> Member {
        Member {
            path: path.to_string(),
            digest: headwater_hash::digest(bytes),
        }
    }

    /// The digest is over the member list and never over the order a walk
    /// happened to return.
    #[test]
    fn the_digest_does_not_depend_on_the_order_the_members_arrived_in() {
        let a = member("a.yml", b"one");
        let b = member("b/c.yml", b"two");
        assert_eq!(
            digest_of(&[a.clone(), b.clone()]),
            digest_of(&[b.clone(), a.clone()])
        );
    }

    /// A path and a digest are separated, so no pair of members can be shuffled
    /// between the two columns and hash the same.
    #[test]
    fn a_member_that_moves_moves_the_digest() {
        let one = vec![member("a.yml", b"one")];
        let two = vec![member("a.yml", b"two")];
        assert_ne!(digest_of(&one), digest_of(&two));
        let renamed = vec![member("b.yml", b"one")];
        assert_ne!(digest_of(&one), digest_of(&renamed));
    }

    #[test]
    fn a_record_round_trips() {
        let release = Release {
            package: "acme/taxonomy".to_string(),
            version: "3.2.0".to_string(),
            requires_engine: Some(">=0.1 <2".to_string()),
            digest: digest_of(&[member("taxonomy.yml", b"one")]),
            members: vec![member("taxonomy.yml", b"one")],
        };
        assert_eq!(read(&render(&release)).expect("it reads"), release);
    }

    /// The record carries a digest over its own member list, so an edit to a
    /// member digest is caught before any artifact is compared with it.
    #[test]
    fn an_edited_record_is_refused_before_the_artifact_is_read() {
        let release = Release {
            package: "acme/taxonomy".to_string(),
            version: "3.2.0".to_string(),
            requires_engine: None,
            digest: digest_of(&[member("taxonomy.yml", b"one")]),
            members: vec![member("taxonomy.yml", b"one")],
        };
        let text = render(&release);
        let forged = text.replace(
            &headwater_hash::digest(b"one"),
            &headwater_hash::digest(b"two"),
        );
        assert_ne!(forged, text);
        assert!(matches!(
            read(&forged),
            Err(ReleaseError::RecordMoved { .. })
        ));
    }

    #[test]
    fn a_record_from_a_later_engine_says_so_rather_than_guessing() {
        let release = Release {
            package: "acme/taxonomy".to_string(),
            version: "3.2.0".to_string(),
            requires_engine: None,
            digest: digest_of(&[]),
            members: vec![],
        };
        let later = render(&release).replace(&format!("format: {FORMAT}"), "format: 9");
        assert!(matches!(read(&later), Err(ReleaseError::Format { .. })));
    }

    #[test]
    fn the_ranges_spec_7_writes() {
        assert_eq!(satisfies(">=1.4 <2", "1.4.0"), Ok(true));
        assert_eq!(satisfies(">=1.4 <2", "1.9.9"), Ok(true));
        assert_eq!(satisfies(">=1.4 <2", "1.3.9"), Ok(false));
        assert_eq!(satisfies(">=1.4 <2", "2.0.0"), Ok(false));
        assert_eq!(satisfies(">=0.1 <2", "0.1.0"), Ok(true));
        assert_eq!(satisfies("=1.0.0", "1.0.0"), Ok(true));
        assert_eq!(satisfies("1.0.0", "1.0.1"), Ok(false));
    }

    /// A range this engine cannot read is a refusal and never a pass. A range
    /// read as no range is a claim the publisher made and the consumer dropped.
    #[test]
    fn a_range_that_cannot_be_read_is_refused() {
        assert!(satisfies(">=1.4.0-beta", "1.4.0").is_err());
        assert!(satisfies("~1.4", "1.4.0").is_err());
        assert!(satisfies("", "1.4.0").is_err());
        assert!(matches!(
            engine_range("acme/taxonomy", Some("^1")),
            Err(ReleaseError::Range { .. })
        ));
    }

    /// The engine range refuses this engine when the package is outside it, and
    /// the message names both numbers.
    #[test]
    fn a_package_that_needs_a_later_engine_is_refused() {
        let refused = engine_range("acme/taxonomy", Some(">=9 <10"))
            .expect_err("this engine is not version nine");
        assert!(matches!(refused, ReleaseError::Engine { .. }));
        assert!(refused.to_string().contains(ENGINE));
        assert!(refused.to_string().contains(">=9 <10"));
        assert_eq!(engine_range("acme/taxonomy", None), Ok(()));
    }

    /// The range in this repository's own manifest holds for this engine.
    /// A floor that the engine shipping it fails would be a package nobody can
    /// resolve, and the number is in two files.
    #[test]
    fn this_engine_satisfies_the_range_the_base_package_declares() {
        assert_eq!(satisfies(">=0.1 <2", ENGINE), Ok(true));
    }
}
