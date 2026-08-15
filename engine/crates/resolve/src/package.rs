// SPDX-License-Identifier: Apache-2.0
//! Where the sources come from: the consumer declaration, the package manifest,
//! and the bundle selection.
//!
//! [Spec 7](../../../../docs/spec/07-distribution-and-federation.md#consuming)
//! gives the consumer declaration its shape — what this repository takes, which
//! bundles it selects, and where its overlay is — and
//! [Publishing](../../../../docs/spec/07-distribution-and-federation.md#publishing)
//! gives the manifest its shape. This module reads both and hands the resolver
//! an ordered list of sources.
//!
//! # The manifest and the taxonomy source are two files, and this is why
//!
//! Spec 2 makes `package` a reserved reference root that no taxonomy may
//! declare, and spec 7 gives the manifest a `package:` key. The two rules never
//! meet while the two files are two files. The only committed copy of this
//! repository's base package was one file playing both parts, and the
//! meta-schema refused it on the first line
//! ([#49](https://github.com/headwater-ai/headwater/issues/49)). The split is
//! made here: `package.yml` names the package, `taxonomy.yml` is the taxonomy
//! source, and [13 — Open obligations](../../../../docs/spec/13-open-obligations.md)
//! carries the ruling.
//!
//! # Finding a package by name, and where it came from
//!
//! A package is a directory under `packages/` whose manifest declares the name
//! the consumer asked for, and the lookup refuses a version the consumer did not
//! pin. How the directory got there is [`publish`] and [`vendor`]: a publisher
//! writes an artifact with a digest over every file in it, a caller moves that
//! artifact by whatever the organization already uses, and `vendor` checks it
//! against the digest this repository pinned before anything is installed. The
//! engine opens no socket at any point, which is why the verb takes a path
//! rather than a location.
//!
//! [`crate::release`] holds the record and argues what its digest proves.

use crate::error::{ResolveError, ResolveErrorKind};
use crate::release::{self, Release, ReleaseError};
use crate::source::{Role, Source};
use headwater_yaml::Mapping;
use std::path::{Path, PathBuf};

/// The directory a package is looked up in, relative to the repository root.
pub const PACKAGES: &str = "packages";

/// The file that carries a package manifest, inside the package directory.
pub const MANIFEST: &str = "package.yml";

/// The manifest key that states which engines the package is for.
///
/// [Spec 7](../../../../docs/spec/07-distribution-and-federation.md#publishing)
/// puts it in the manifest and until this module nothing read it, so the base
/// package left it out rather than commit a claim no test could fail.
/// [`sources`] reads it now, and a package outside the range is refused before
/// any source is loaded.
pub const REQUIRES_ENGINE: &str = "requires_engine";

/// The consumer declaration: what this repository takes, and what it walks.
#[derive(Clone, Debug)]
pub struct Consumer {
    pub package: String,
    pub version: String,
    pub bundles: Vec<String>,
    /// The digest of the published artifact this repository takes, where it
    /// declares one.
    ///
    /// The pin is authored and it is never written by a verb. A digest that the
    /// engine recorded from whatever it had just fetched would be a pin against
    /// itself, so `vendor` refuses to run without one rather than trusting the
    /// first artifact it meets. See [`crate::release`].
    pub digest: Option<String>,
    pub overlay: Option<String>,
    /// The corpus root, as written, relative to the repository root.
    pub corpus_root: String,
    /// Each declared exclusion, with the reason it states. A reason is not
    /// optional: an exclusion with none is a silent pass with a configuration
    /// file in front of it.
    pub exclusions: Vec<(String, String)>,
}

/// The file that carries it.
pub const CONSUMER: &str = ".headwater/taxonomy.yml";

/// Read the consumer declaration of a repository.
pub fn consumer(root: &Path) -> Result<Consumer, Vec<ResolveError>> {
    let path = root.join(CONSUMER);
    let loaded = crate::source::load(&path)?;
    let name = CONSUMER;
    let map = loaded
        .value
        .as_map()
        .ok_or_else(|| refusal(name, "the consumer declaration is not a mapping"))?;

    let taxonomy = map
        .get("taxonomy")
        .and_then(|node| node.value.as_map())
        .ok_or_else(|| refusal(name, "no `taxonomy` block, so nothing says what this takes"))?;

    let package = text(taxonomy, "package")
        .ok_or_else(|| refusal(name, "`taxonomy.package` names the package this takes"))?;
    let version = text(taxonomy, "version")
        .ok_or_else(|| refusal(name, "`taxonomy.version` pins the version this takes"))?;

    let bundles = taxonomy
        .get("bundles")
        .and_then(|node| node.value.as_seq())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.value.as_scalar())
                .map(|scalar| scalar.text.clone())
                .collect()
        })
        .unwrap_or_default();

    let corpus = map
        .get("corpus")
        .and_then(|node| node.value.as_map())
        .ok_or_else(|| refusal(name, "no `corpus` block, so nothing says what to walk"))?;
    let corpus_root = text(corpus, "root")
        .ok_or_else(|| refusal(name, "`corpus.root` names the directory the census walks"))?;

    let mut exclusions = Vec::new();
    if let Some(items) = corpus.get("exclude").and_then(|node| node.value.as_seq()) {
        for item in items {
            let Some(entry) = item.value.as_map() else {
                return Err(refusal(name, "an exclusion is a mapping"));
            };
            let path =
                text(entry, "path").ok_or_else(|| refusal(name, "an exclusion names a path"))?;
            let reason = text(entry, "reason")
                .ok_or_else(|| refusal(name, "an exclusion states a reason"))?;
            exclusions.push((path, reason));
        }
    }

    Ok(Consumer {
        package,
        version,
        bundles,
        digest: text(taxonomy, "digest"),
        overlay: text(taxonomy, "overlay"),
        corpus_root,
        exclusions,
    })
}

/// The base package and every overlay the consumer selected, in application
/// order: the package, then each selected bundle, then the adopter overlay.
///
/// The order is a formality and the confluence check is what makes it one. It
/// is fixed anyway, so that two runs of one resolution produce one lock.
pub fn sources(root: &Path, consumer: &Consumer) -> Result<Vec<Source>, Vec<ResolveError>> {
    let (directory, manifest) = find(root, &consumer.package)?;

    let declared = text(&manifest, "version").unwrap_or_default();
    if declared != consumer.version {
        return Err(refusal(
            CONSUMER,
            &format!(
                "this takes {} {}, and the package here is {declared}",
                consumer.package, consumer.version
            ),
        ));
    }

    sources_at(root, &directory, &manifest, consumer)
}

/// The same sources, out of a package directory the caller already holds.
///
/// [`sources`] finds the package by the name the consumer pinned and holds it
/// to the version the consumer pinned. A comparison of two versions needs the
/// second half of that dropped and nothing else: the artifact under comparison
/// is by definition not the version this repository takes, and the overlays it
/// is resolved under are this repository's own. So the version check lives in
/// the caller above and every other step is here, in one copy. A second reader
/// of a `contents` block would be a second answer to "what does this package
/// ship", and the two could then disagree about a bundle path.
pub fn sources_at(
    root: &Path,
    directory: &Path,
    manifest: &Mapping,
    consumer: &Consumer,
) -> Result<Vec<Source>, Vec<ResolveError>> {
    // The engine range the package declares, checked before a single source is
    // read. A package that needs a later engine resolves into a taxonomy this
    // engine reads with whatever it does not understand dropped, and that is a
    // lock nobody can reproduce. The refusal names both numbers.
    if let Err(refused) = release::engine_range(
        &consumer.package,
        text(manifest, REQUIRES_ENGINE).as_deref(),
    ) {
        return Err(release::as_error(&manifest_name(root, directory), &refused));
    }

    let contents = manifest
        .get("contents")
        .and_then(|node| node.value.as_map())
        .cloned()
        .unwrap_or_default();

    let taxonomy = directory.join(text(&contents, "taxonomy").unwrap_or_default());
    let mut out = vec![Source::read(
        &taxonomy,
        &display(root, &taxonomy),
        Role::Taxonomy,
    )?];

    if !consumer.bundles.is_empty() {
        let bundles = text(&contents, "bundles").ok_or_else(|| {
            refusal(
                CONSUMER,
                &format!(
                    "this selects {} bundles and {} ships none",
                    consumer.bundles.len(),
                    consumer.package
                ),
            )
        })?;
        for name in &consumer.bundles {
            let path = directory.join(&bundles).join(name).join("bundle.yml");
            out.push(Source::read(&path, &display(root, &path), Role::Overlay)?);
        }
    }

    if let Some(overlay) = &consumer.overlay {
        let path = root.join(overlay);
        out.push(Source::read(&path, overlay, Role::Overlay)?);
    }

    Ok(out)
}

/// The package directory whose manifest declares `name`.
/// The version a package on disk declares, or `None` where no package under
/// `packages/` carries that name.
///
/// `headwater init` writes a consumer declaration that pins a version, and a
/// version it invented would be refused by `sources` two commands later with a
/// message about a mismatch rather than about the missing package. This is the
/// same lookup, asked early, and it answers `None` rather than an error because
/// a repository with no package vendored is the ordinary state of a repository
/// that has not been bound yet.
pub fn find_version(root: &Path, name: &str) -> Option<String> {
    let (_, manifest) = find(root, name).ok()?;
    text(&manifest, "version")
}

/// The directory a package sits in, and the manifest it declares.
///
/// [`sources`] reads the manifest for the taxonomy and the bundles, and it is
/// not the only reader any more. A conformance rule set is package content that
/// no taxonomy source names, so a caller that reads one needs the directory and
/// the `contents` block without resolving anything. It answers `None` on the
/// same terms [`find_version`] does: a repository with no package installed is
/// the ordinary state of a repository that nothing has bound yet.
pub fn located(root: &Path, name: &str) -> Option<(PathBuf, Mapping)> {
    find(root, name).ok()
}

/// The manifest of a package directory the caller already holds.
///
/// [`find`] locates a package by the name it declares, under `packages/`. An
/// artifact somebody fetched is neither: it sits wherever the caller put it and
/// its name is what the comparison is about rather than what finds it. So this
/// is the same read from the other end, and it is the only one this crate
/// exposes that takes a directory.
pub fn manifest_at(directory: &Path) -> Result<Mapping, Vec<ResolveError>> {
    let manifest = directory.join(MANIFEST);
    let loaded = crate::source::load(&manifest)?;
    loaded.value.as_map().cloned().ok_or_else(|| {
        refusal(
            &manifest.display().to_string(),
            "the manifest is not a mapping",
        )
    })
}

fn find(root: &Path, name: &str) -> Result<(PathBuf, Mapping), Vec<ResolveError>> {
    let packages = root.join(PACKAGES);
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&packages)
        .map_err(|error| {
            refusal(
                PACKAGES,
                &format!("cannot read {}: {error}", packages.display()),
            )
        })?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    entries.sort();

    for directory in entries {
        let manifest = directory.join(MANIFEST);
        if !manifest.is_file() {
            continue;
        }
        let loaded = crate::source::load(&manifest)?;
        let Some(map) = loaded.value.as_map() else {
            continue;
        };
        if text(map, "package").as_deref() == Some(name) {
            return Ok((directory, map.clone()));
        }
    }

    Err(refusal(
        PACKAGES,
        &format!("no package under `{PACKAGES}/` declares `{name}`"),
    ))
}

/// Write the published artifact of a package into `out`.
///
/// The artifact is a directory, because this engine carries no archive format
/// and needs none: whatever the organization already uses to move a directory
/// moves this one, and an archive would be a second thing to check the digest of.
///
/// Two things happen that a copy alone would not do.
///
/// **The bundles come inside.** A manifest may point `contents.bundles` outside
/// the package, and this repository's does: the package and the library entry
/// that ships its bundles live in one tree, so the path climbs out with `../..`.
/// That works only while a package is found rather than fetched. Publishing
/// copies the bundles into `bundles/` beside the manifest and rewrites the one
/// scalar, so the published manifest names a path inside the artifact. The
/// escape is therefore a property of the source layout, and no artifact carries
/// it.
///
/// **A record is written.** [`release::compute`] takes the digest of every file
/// in the artifact, and the digest over that list is the number the publisher
/// states in its release notes and the consumer pins.
///
/// **The migration payload is read before a byte is written.** A `contents`
/// key that nothing reads is a claim a publisher makes and a consumer never
/// sees, which is the defect `requires_engine` exists to refuse from the other
/// side. So [`crate::migration::at`] reads every payload the manifest declares
/// and [`crate::migration::holds`] checks each one against the taxonomy being
/// published and the version it is published as. A payload the publisher cannot
/// ship correctly stops the publish, rather than reaching a digest that makes it
/// permanent.
pub fn publish(root: &Path, name: &str, out: &Path) -> Result<Release, Vec<ResolveError>> {
    let (directory, manifest) = find(root, name)?;

    let occupied = std::fs::read_dir(out).map(|mut entries| entries.next().is_some());
    if occupied.unwrap_or(false) {
        return Err(refusal(
            &display(root, out),
            "the output directory holds files already, and a published artifact is every file \
             under its root. Publish into a directory that does not exist yet",
        ));
    }

    migrations(root, &directory, &manifest)?;

    copy_tree(&directory, out).map_err(|why| refusal(&display(root, &directory), &why))?;

    // The bundles path, where the manifest states one that leaves the package.
    // A path that stays inside was copied with everything else.
    if let Some(bundles) = manifest
        .get("contents")
        .and_then(|node| node.value.as_map())
        .and_then(|contents| contents.get("bundles"))
    {
        let Some(scalar) = bundles.value.as_scalar() else {
            return Err(refusal(
                &display(root, &directory.join(MANIFEST)),
                "`contents.bundles` is not a path",
            ));
        };
        if leaves(&scalar.text) {
            copy_tree(&directory.join(&scalar.text), &out.join(BUNDLES))
                .map_err(|why| refusal(&display(root, &directory), &why))?;
            let source = std::fs::read_to_string(directory.join(MANIFEST))
                .map_err(|error| refusal(MANIFEST, &format!("cannot read it: {error}")))?;
            let rewritten = splice(&source, bundles.span, BUNDLES).ok_or_else(|| {
                refusal(
                    &display(root, &directory.join(MANIFEST)),
                    "`contents.bundles` does not lie inside the manifest it was read from",
                )
            })?;
            std::fs::write(out.join(MANIFEST), rewritten)
                .map_err(|error| refusal(MANIFEST, &format!("cannot write it: {error}")))?;
        }
    }

    let record = release::compute(out, &manifest)
        .map_err(|error| release::as_error(&display(root, out), &error))?;
    std::fs::write(out.join(release::RECORD), release::render(&record))
        .map_err(|error| refusal(release::RECORD, &format!("cannot write it: {error}")))?;
    Ok(record)
}

/// Every migration payload the manifest declares, read and held to the taxonomy
/// this publish ships.
///
/// It resolves the package's own taxonomy source and nothing else. The base
/// alone is what the publisher is shipping: a bundle is an overlay a consumer
/// selects and an adopter overlay is not the publisher's at all, so a step
/// checked against either would be checked against a taxonomy that this
/// artifact does not carry.
///
/// The resolution happens only where a payload exists, so a package that has
/// published no major version pays nothing for this.
fn migrations(root: &Path, directory: &Path, manifest: &Mapping) -> Result<(), Vec<ResolveError>> {
    let name = manifest_name(root, directory);
    let payloads =
        crate::migration::at(directory, manifest).map_err(|errors| crate::migration::as_errors(&name, &errors))?;
    if payloads.is_empty() {
        return Ok(());
    }

    let contents = manifest
        .get("contents")
        .and_then(|node| node.value.as_map())
        .cloned()
        .unwrap_or_default();
    let taxonomy = directory.join(text(&contents, "taxonomy").unwrap_or_default());
    let source = Source::read(&taxonomy, &display(root, &taxonomy), Role::Taxonomy)?;
    let resolution = crate::resolve(&[source])?;
    let version = text(manifest, "version").unwrap_or_default();

    let refusals: Vec<crate::migration::PayloadError> = payloads
        .iter()
        .flat_map(|payload| crate::migration::holds(payload, &resolution.taxonomy, &version))
        .collect();
    match refusals.is_empty() {
        true => Ok(()),
        false => Err(crate::migration::as_errors(&name, &refusals)),
    }
}

/// Where a published package keeps the bundles it ships.
pub const BUNDLES: &str = "bundles";

/// Check a fetched artifact against the digest this repository pinned, and
/// install it under `packages/`.
///
/// The caller fetched it. This engine has no idea where from and cannot ask:
/// no crate of it depends on the network, and a verb that took a location
/// rather than a path is the one change that would end that guarantee.
///
/// The pin is the argument of the check and it is never derived from the
/// artifact. A `vendor` with no pin is a fetch nobody checked, so it refuses and
/// names the field to write, rather than recording what it happened to receive.
///
/// An existing directory is replaced only when it is itself a vendored artifact.
/// A package directory that a person maintains is a publisher's source, and
/// overwriting one on a consumer command would delete the thing being published.
pub fn vendor(root: &Path, fetched: &Path, pinned: &str) -> Result<Release, Vec<ResolveError>> {
    let name = display(root, fetched);
    let record =
        release::verify(fetched, pinned).map_err(|error| release::as_error(&name, &error))?;

    let target = root.join(PACKAGES).join(record.package.replace('/', "-"));
    if target.exists() {
        match release::at(&target) {
            Ok(_) => {
                std::fs::remove_dir_all(&target).map_err(|error| {
                    refusal(
                        &display(root, &target),
                        &format!("cannot replace it: {error}"),
                    )
                })?;
            }
            Err(ReleaseError::Absent(_)) => {
                return Err(refusal(
                    &display(root, &target),
                    "a directory is there and it carries no release record, so it is a package \
                     somebody maintains rather than one that was vendored. Move it before \
                     vendoring over it",
                ))
            }
            Err(error) => return Err(release::as_error(&display(root, &target), &error)),
        }
    }
    copy_tree(fetched, &target).map_err(|why| refusal(&name, &why))?;
    Ok(record)
}

/// Whether a path written in a manifest leaves the package that carries it.
fn leaves(path: &str) -> bool {
    Path::new(path)
        .components()
        .any(|part| part == std::path::Component::ParentDir)
        || Path::new(path).is_absolute()
}

/// Replace the text a span covers, where the span lies inside the source.
fn splice(source: &str, span: headwater_yaml::Span, with: &str) -> Option<String> {
    let before = source.get(..span.start.offset)?;
    let after = source.get(span.end.offset..)?;
    Some(format!("{before}{with}{after}"))
}

/// Copy a directory tree, creating what it needs.
fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::create_dir_all(to)
        .map_err(|error| format!("cannot create {}: {error}", to.display()))?;
    let mut entries: Vec<PathBuf> = std::fs::read_dir(from)
        .map_err(|error| format!("cannot read {}: {error}", from.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    entries.sort();
    for entry in entries {
        let Some(name) = entry.file_name() else {
            continue;
        };
        let destination = to.join(name);
        if entry.is_dir() {
            copy_tree(&entry, &destination)?;
        } else {
            std::fs::copy(&entry, &destination)
                .map_err(|error| format!("cannot copy {}: {error}", entry.display()))?;
        }
    }
    Ok(())
}

/// The manifest of a package, as a reader of the repository would name it.
fn manifest_name(root: &Path, directory: &Path) -> String {
    display(root, &directory.join(MANIFEST))
}

/// A path as a reader of the repository would write it.
///
/// The `..` is resolved lexically rather than by the file system, because the
/// name in a message is for a person and `packages/x/../../docs/y` names a file
/// that nobody can find in a tree view.
fn display(root: &Path, path: &Path) -> String {
    let mut parts: Vec<std::ffi::OsString> = Vec::new();
    for part in path.components() {
        match part {
            std::path::Component::ParentDir if !parts.is_empty() => {
                parts.pop();
            }
            std::path::Component::CurDir => {}
            other => parts.push(other.as_os_str().to_owned()),
        }
    }
    let flattened: PathBuf = parts.iter().collect();
    flattened
        .strip_prefix(root)
        .unwrap_or(&flattened)
        .display()
        .to_string()
}

fn text(map: &Mapping, key: &str) -> Option<String> {
    map.get(key)
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.clone())
}

fn refusal(source: &str, message: &str) -> Vec<ResolveError> {
    vec![ResolveError::new(
        ResolveErrorKind::SourceRefused(message.to_string()),
        source,
        "",
        headwater_yaml::Span::default(),
    )]
}
