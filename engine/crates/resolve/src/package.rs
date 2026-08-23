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
use crate::operation::{self, Operation};
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

    // The package is held to itself before it is held to the pin, and the order
    // is the whole point. A package that states two versions of itself is
    // malformed whichever number a consumer wrote down, so a comparison against
    // the consumer's pin cannot be what decides it: pinning the manifest number
    // hid the disagreement completely and pinning the other one reported it as
    // a wrong pin. See [`agrees`].
    let (base, contents) = base(root, &directory, &manifest, &consumer.package)?;

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

    selected(root, &directory, &contents, consumer, base)
}

/// The same sources, out of a package directory the caller already holds.
///
/// [`sources`] finds the package by the name the consumer pinned and holds it
/// to the version the consumer pinned. A comparison of two versions needs the
/// second half of that dropped and nothing else: the artifact under comparison
/// is by definition not the version this repository takes, and the overlays it
/// is resolved under are this repository's own. So the pin comparison lives in
/// the caller above and every other step is in [`base`] and [`selected`], in one
/// copy. A second reader of a `contents` block would be a second answer to
/// "what does this package ship", and the two could then disagree about a
/// bundle path.
///
/// The version a package declares of *itself*, twice, is a different question,
/// and it is not dropped here. An artifact whose two files disagree is refused
/// on this path as well, which is what puts `taxonomy diff` and
/// `taxonomy migrate` behind the same refusal as a resolve. See [`agrees`].
pub fn sources_at(
    root: &Path,
    directory: &Path,
    manifest: &Mapping,
    consumer: &Consumer,
) -> Result<Vec<Source>, Vec<ResolveError>> {
    let (base, contents) = base(root, directory, manifest, &consumer.package)?;
    selected(root, directory, &contents, consumer, base)
}

/// The package's own taxonomy source, and the `contents` block it was found
/// through.
///
/// Everything here is a question about the package alone, so nothing in it
/// reads the consumer beyond the name a message calls the package by. The
/// caller then asks the questions about the pairing: which version this
/// repository pinned, and which bundles it selected.
fn base(
    root: &Path,
    directory: &Path,
    manifest: &Mapping,
    package: &str,
) -> Result<(Source, Mapping), Vec<ResolveError>> {
    // The engine range the package declares, checked before a single source is
    // read. A package that needs a later engine resolves into a taxonomy this
    // engine reads with whatever it does not understand dropped, and that is a
    // lock nobody can reproduce. The refusal names both numbers.
    if let Err(refused) = release::engine_range(package, text(manifest, REQUIRES_ENGINE).as_deref())
    {
        return Err(release::as_error(&manifest_name(root, directory), &refused));
    }

    let contents = contents_of(manifest);
    let source = taxonomy_source(root, directory, &contents)?;
    agrees(&manifest_name(root, directory), manifest, &source)?;
    Ok((source, contents))
}

/// The taxonomy source a manifest's `contents.taxonomy` points at.
fn taxonomy_source(
    root: &Path,
    directory: &Path,
    contents: &Mapping,
) -> Result<Source, Vec<ResolveError>> {
    let taxonomy = directory.join(text(contents, "taxonomy").unwrap_or_default());
    Source::read(&taxonomy, &display(root, &taxonomy), Role::Taxonomy)
}

/// The `contents` block of a manifest, empty where it declares none.
fn contents_of(manifest: &Mapping) -> Mapping {
    manifest
        .get("contents")
        .and_then(|node| node.value.as_map())
        .cloned()
        .unwrap_or_default()
}

/// A package declares what it is twice — its name and its version — and this is
/// the only thing that holds either pair together.
///
/// [Spec 7](../../../../docs/spec/07-distribution-and-federation.md#publishing)
/// gives the manifest a `package` key and a `version` key, and the meta-schema
/// requires a `taxonomy` and a `version` at the root of a taxonomy source. The
/// split above says why the two files are two files, and it left both of those
/// values declared in both of them. Nothing compared the versions until
/// [#212](https://github.com/headwater-ai/headwater/issues/212) and nothing
/// compared the names until
/// [#267](https://github.com/headwater-ai/headwater/issues/267), so a package
/// could ship with either pair apart and pass every gate this repository runs.
///
/// **The manifest decides, and this is what makes that statement true.** Every
/// consumer-facing reader takes the manifest value. For the version that is the
/// pin comparison in [`sources`], [`find_version`], and the release record that
/// `taxonomy diff` reads a version out of. For the name it is [`find`], the
/// release record's `package` field, the vendor target directory derived from
/// it, and the corpus descriptor a served host publishes. The meta-schema had
/// already ruled this half in a comment: `package` is a reserved reference root,
/// no taxonomy may declare a block with that name, and it is the manifest that
/// names the package. The taxonomy source's copies reach the lock, where the
/// `taxonomy` block sits beside the `package` block — so before this the
/// committed lock could state two names of one taxonomy, or two versions of it,
/// in two of its own blocks. Rather than pick a winner and leave the loser
/// writable, each pair is required to agree.
///
/// **The refusals cannot be one refusal.** Both version keys are called
/// `version`, so that message says the word once. The name keys are two
/// different keys, `package:` in the manifest and `taxonomy:` in the source, so
/// its message names each key beside its value. A message that named the value
/// alone would send a reader looking for a `package:` key in the taxonomy source
/// that the meta-schema forbids.
///
/// **The name is compared before the version**, because a package that is not
/// the package you asked for makes the question of its version moot. Where both
/// disagree both are reported, so a second run does not have to discover the
/// second one.
///
/// Whether a taxonomy source should carry a version at all is a meta-schema
/// question, and #212 does not answer it. This holds the declarations that
/// exist to each other and rules on neither.
fn agrees(manifest: &str, declaration: &Mapping, source: &Source) -> Result<(), Vec<ResolveError>> {
    let Some(root) = source.root.value.as_map() else {
        // Not a mapping, so it declares nothing at all. `validate` refuses it
        // against the meta-schema with a message about its shape, which is the
        // finding a reader needs rather than one about a missing key.
        return Ok(());
    };
    let mut refused = Vec::new();

    let named = text(declaration, "package").unwrap_or_default();
    let carries = text(root, "taxonomy").unwrap_or_default();
    if named != carries {
        refused.extend(refusal(
            manifest,
            &format!(
                "this declares `package: {named}` and the taxonomy source it names, {}, declares \
                 `taxonomy: {carries}`. One package states two names of itself, and each of them \
                 is what some reader downstream takes the package to be",
                source.name
            ),
        ));
    }

    let declared = text(declaration, "version").unwrap_or_default();
    let carried = text(root, "version").unwrap_or_default();
    if declared != carried {
        refused.extend(refusal(
            manifest,
            &format!(
                "this declares version `{declared}` and the taxonomy source it names, {}, \
                 declares `{carried}`. One package states two versions of itself, and each of \
                 them is what some reader downstream takes the package to be",
                source.name
            ),
        ));
    }

    if refused.is_empty() {
        return Ok(());
    }
    Err(refused)
}

/// The bundles the consumer selected and the overlay it declares, on top of the
/// package's own source.
fn selected(
    root: &Path,
    directory: &Path,
    contents: &Mapping,
    consumer: &Consumer,
    base: Source,
) -> Result<Vec<Source>, Vec<ResolveError>> {
    let mut out = vec![base];

    if !consumer.bundles.is_empty() {
        let bundles = text(contents, "bundles").ok_or_else(|| {
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

/// The adopter's own overlay, read on its own and merged into nothing.
///
/// # Why the bundles are not here
///
/// A migration rewrites the addresses of the file this repository owns. A
/// bundle is package content, and
/// [spec 7](../../../../docs/spec/07-distribution-and-federation.md#waivers)
/// gives the reason a local edit to package content does not survive:
/// `headwater taxonomy vendor` replaces a vendored package directory whole. An
/// address rewritten inside a bundle is lost at the next upgrade, and the
/// publisher of the bundle is who moves it. So the writable set is this one
/// file, and a bundle whose address no longer reaches a declaration is reported
/// by the `addressability` dimension against the publisher instead.
///
/// The source index of every operation is 0, because there is one source.
pub fn adopted(root: &Path, consumer: &Consumer) -> Result<Adopted, Vec<ResolveError>> {
    let Some(declared) = &consumer.overlay else {
        return Ok(Adopted::None);
    };
    let source = Source::read(&root.join(declared), declared, Role::Overlay)?;
    let operations = operation::read(0, declared, &source.root)?;
    Ok(Adopted::Declared {
        at: declared.clone(),
        operations,
    })
}

/// What a repository declares as its own overlay.
///
/// Two arms rather than an [`Option`], because "this repository declares no
/// overlay" and "the overlay it declares holds no operation" are two states a
/// report says different things about, and one of them is a file somebody has
/// to open.
#[derive(Clone, Debug)]
pub enum Adopted {
    Declared {
        /// The path, as the consumer declaration writes it.
        at: String,
        operations: Vec<Operation>,
    },
    /// The consumer declaration names no overlay, so no address of this
    /// repository is a migration subject.
    None,
}

impl Adopted {
    pub fn operations(&self) -> &[Operation] {
        match self {
            Adopted::Declared { operations, .. } => operations,
            Adopted::None => &[],
        }
    }

    /// The file a rewrite would write, where there is one.
    pub fn at(&self) -> Option<&str> {
        match self {
            Adopted::Declared { at, .. } => Some(at),
            Adopted::None => None,
        }
    }
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
///
/// **The two version declarations are held to each other before a byte is
/// copied.** A publish does not resolve for a consumer, so it does not pass
/// through the comparison [`sources`] makes. It copies both files into the
/// artifact and [`release::compute`] takes a digest over both, so a package that
/// states two versions of itself would reach an adopter with the disagreement
/// sealed under one number. See [`agrees`].
///
/// # Everything is read before anything is written
///
/// The three paragraphs above each say *before a byte is written* about one
/// input, and until [#271] the whole of the rest of the artifact was read by the
/// write loop itself. A manifest whose `contents.bundles` climbed out of a
/// package directory that somebody had copied — which is what `headwater init`
/// invites and what a reader of the tutorial does — reached
/// [`copy_tree`]'s `ENOENT` with `package.yml`, `taxonomy.yml`,
/// `conformance.yml` and an empty `bundles/` already under `out`. The verb then
/// printed `nothing was published`, and the next run was refused by the
/// `--out` precondition catching the leftovers of the first.
///
/// So this is now three phases and the order is the guarantee.
///
/// 1. Read. [`reachable`] holds every path the manifest's `contents` declares
///    to the tree, and [`stage`] reads every byte of the artifact into memory.
///    Nothing under `out` exists yet, and a refusal here names the manifest, the
///    key and the declared value rather than a file system error carrying a
///    `..`.
/// 2. Write. [`put`] creates `out` and writes the staged set.
/// 3. Undo. A failure in phase 2 — a full disk, a permission, a race — returns
///    `out` to the state phase 1 found it in. [`found::observe`] is what reads
///    that state and the only thing that can make one, and the `--out`
///    precondition is what makes the undo total. A directory that holds
///    anything, a directory this process cannot read, and a symlink that leads
///    nowhere are all refused before a byte is staged, so every state the undo
///    can return to is one the run observed and nothing it removes belonged to
///    anybody else. The undo takes `out` and every directory above it that this
///    run made on the way to it, and it stops climbing at the first directory
///    that is not empty.
///
/// **This is [`headwater_scaffold::tree::Reserved`]'s shape and not its code.**
/// `headwater-scaffold` depends on `headwater-check`, which depends on this
/// crate, so a dependency the other way is a cycle the compiler refuses. The
/// technique does not transfer either: `Reserved` opens every target for writing
/// and holds the handle, which is a fact rather than a probe, and it can do that
/// only because its targets already exist. Every target of a publish is a file
/// that must not exist yet. Reading every *input* fully into memory is the
/// nearest thing to holding a handle that a writer of new files has.
///
/// [#271]: https://github.com/headwater-ai/headwater/issues/271
pub fn publish(root: &Path, name: &str, out: &Path) -> Result<Release, Vec<ResolveError>> {
    let (directory, manifest) = find(root, name)?;
    let contents = contents_of(&manifest);
    let declared = manifest_name(root, &directory);

    // Every declared path is held to the tree here, before the first reader of
    // one runs. `taxonomy_source` reads `contents.taxonomy` for the resolver,
    // and while this ran inside `stage` that one key never reached the refusal
    // below: a missing taxonomy source came back as `cannot read
    // …/packages/x/../../elsewhere/taxonomy.yml: No such file or directory`,
    // which names neither the manifest nor the key and carries the `..` that
    // publication exists to remove. One position for one rule, and `stage` no
    // longer holds a second copy of the call.
    reachable(&declared, &directory, &contents)?;

    let source = taxonomy_source(root, &directory, &contents)?;
    agrees(&declared, &manifest, &source)?;

    let found = found::observe(out).map_err(|why| refusal(&display(root, out), &why))?;

    migrations(root, &directory, &manifest, source)?;

    let staged = stage(root, &directory, &manifest)?;

    let written = put(out, &staged)
        .map_err(|why| refusal(&display(root, out), &why))
        .and_then(|()| {
            let record = release::compute(out, &manifest)
                .map_err(|error| release::as_error(&display(root, out), &error))?;
            std::fs::write(out.join(release::RECORD), release::render(&record))
                .map_err(|error| refusal(release::RECORD, &format!("cannot write it: {error}")))?;
            Ok(record)
        });

    match written {
        Ok(record) => Ok(record),
        Err(errors) => {
            found.unwind(out);
            Err(errors)
        }
    }
}

/// What a publish found at `--out`, and the only thing that can find it out.
///
/// The module is the whole point of the module. [`Found`] carries a state that
/// decides whether an undo removes `--out` itself, so a caller that could name
/// that state could assert absence rather than observe it. Nothing outside these
/// lines can make a `Found`: the variants are private to the module and
/// [`observe`] is the one function that returns one. That is the shape [#271]
/// went looking for, because the first reading of this took *did not observe*
/// for *observed nothing* and the undo then removed a path the run had never
/// seen.
///
/// [#271]: https://github.com/headwater-ai/headwater/issues/271
mod found {
    use std::io::ErrorKind;
    use std::path::{Path, PathBuf};

    /// The state `--out` was in before anything was written.
    pub(super) struct Found(State);

    enum State {
        /// Nothing was there, so the undo removes the directory itself, and
        /// every directory above it that was not there either. [`put`] reaches
        /// `--out` with `create_dir_all`, so `--out nested/a/b/c` under an
        /// absent `nested` is four directories that the run made and that a
        /// failed run has no reason to leave standing.
        ///
        /// The list is deepest first, and it is what this run *saw* to be
        /// absent rather than what it later created. Everything above the first
        /// directory that already existed is somebody else's and is not in it.
        ///
        /// [`put`]: super::put
        Absent(Vec<PathBuf>),
        /// A directory was there and it was empty, so the undo empties it again.
        Empty,
    }

    /// `out` and every directory above it that is not there either, deepest
    /// first.
    ///
    /// The walk stops at the first path that exists, and `symlink_metadata` is
    /// what asks, so a link above `out` stops it rather than being read through.
    fn absent_above(out: &Path) -> Vec<PathBuf> {
        let mut chain = vec![out.to_path_buf()];
        let mut at = out.parent();
        while let Some(parent) = at {
            if parent.as_os_str().is_empty() {
                break;
            }
            match std::fs::symlink_metadata(parent) {
                Err(error) if error.kind() == ErrorKind::NotFound => {
                    chain.push(parent.to_path_buf());
                    at = parent.parent();
                }
                _ => break,
            }
        }
        chain
    }

    /// Read `--out`, or say why a publish cannot start.
    ///
    /// # Only `NotFound` can mean absent, and on its own it does not mean it
    ///
    /// `read_dir` reports *the path is not there* and *this process cannot read
    /// what is there* through one `Err`, and [`State::Absent`] is the state
    /// whose undo removes `--out`. So every error that is not
    /// [`ErrorKind::NotFound`] is refused here, before a byte is staged. A
    /// directory this run could not inspect is one whose contents a publisher
    /// never agreed to interleave an artifact with, and a state the run did not
    /// observe is one the undo has no way back to. Letting it reach the write
    /// is what put a whole artifact into a `0300` directory beside somebody's
    /// file, under a verb that then printed `nothing was published`.
    ///
    /// # A dangling symlink is a path, and `NotFound` does not see it
    ///
    /// `read_dir` follows a symlink, so a link whose target does not exist
    /// reports `NotFound` while a path very much exists under that name.
    /// `symlink_metadata` is the call that does not follow, and it is what
    /// separates the two. The link is then refused rather than published into:
    /// a publisher who asked for a link's target got a name whose target is not
    /// there, which is a mistake worth reading about, and the alternative is a
    /// failed publish whose undo removes a link that the run did not make and
    /// that nothing in the artifact records.
    pub(super) fn observe(out: &Path) -> Result<Found, String> {
        match std::fs::read_dir(out) {
            Ok(mut entries) => match entries.next() {
                Some(_) => Err("the output directory holds files already, and a published \
                                artifact is every file under its root. Publish into a directory \
                                that does not exist yet"
                    .to_string()),
                None => Ok(Found(State::Empty)),
            },
            Err(error) if error.kind() == ErrorKind::NotFound => {
                match std::fs::symlink_metadata(out) {
                    Err(error) if error.kind() == ErrorKind::NotFound => {
                        Ok(Found(State::Absent(absent_above(out))))
                    }
                    Ok(_) => Err(
                        "there is a symlink at the output path and it leads nowhere. \
                                  Publish into a path that does not exist yet"
                            .to_string(),
                    ),
                    Err(error) => Err(format!("the output path cannot be read: {error}")),
                }
            }
            Err(error) => Err(format!("the output directory cannot be read: {error}")),
        }
    }

    impl Found {
        /// Put `out` back the way the run found it.
        ///
        /// Every error is dropped. This runs on the way out of a failure that is
        /// already being reported, and a second message about the cleanup would
        /// displace the one a reader needs. What a caller is owed is the
        /// property, and the property is what the fixture asserts.
        pub(super) fn unwind(self, out: &Path) {
            match self.0 {
                State::Absent(chain) => {
                    let _ = std::fs::remove_dir_all(out);
                    // Everything above `out` goes with `remove_dir`, which
                    // takes an empty directory and nothing else. A directory
                    // that somebody filled while this run was writing therefore
                    // stops the climb by refusing to go, which is the answer a
                    // race is owed.
                    for above in chain.into_iter().skip(1) {
                        if std::fs::remove_dir(&above).is_err() {
                            break;
                        }
                    }
                }
                State::Empty => {
                    let Ok(entries) = std::fs::read_dir(out) else {
                        return;
                    };
                    for entry in entries.filter_map(Result::ok) {
                        let at = entry.path();
                        let _ = match at.is_dir() {
                            true => std::fs::remove_dir_all(&at),
                            false => std::fs::remove_file(&at),
                        };
                    }
                }
            }
        }
    }
}

/// One file of an artifact, read off the publisher's tree.
struct Staged {
    /// Relative to the artifact root, with `/` separators.
    path: String,
    bytes: Vec<u8>,
    /// What the source file carried. `std::fs::copy` took these across before
    /// staging did, and a publish is not the place to start normalizing them.
    mode: std::fs::Permissions,
}

/// Which kind of thing a `contents` key's reader opens.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    File,
    Directory,
}

impl Kind {
    /// The kind a path on disk is, where it is one of these two.
    fn of(at: &Path) -> Kind {
        match at.is_dir() {
            true => Kind::Directory,
            false => Kind::File,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Kind::File => "file",
            Kind::Directory => "directory",
        }
    }
}

/// What the reader of a `contents` key opens at the path that key names.
///
/// Each row was read off the verb that reads the key, and not off the manifest
/// or off spec 7:
///
/// - `taxonomy` is a file: [`taxonomy_source`] hands it to [`crate::source::load`],
///   which is `read_to_string`.
/// - `conformance` is a file: `headwater_conformance::set` reads it with
///   `read_to_string`. The key is a literal here rather than that crate's
///   constant, because `headwater-conformance` depends on this crate and the
///   dependency cannot run the other way.
/// - `bundles` is a directory: [`stage`] reads it with [`read_tree`], which is
///   `read_dir`, and the resolver then reads `<bundles>/<name>/bundle.yml`.
/// - `migrations` is a directory: [`crate::migration::at`] reads it with
///   `read_dir` and globs `*.yml` out of it.
///
/// **A key that is not here keeps the existence check and nothing more.** That
/// is the seam of this table. [`reachable`] still walks the keys the manifest
/// declares rather than this list, so a key nobody reads yet — spec 7's example
/// block declares `doctrine`, `templates` and `plugins` — is held to being
/// there, and gains a kind on the day something reads it. Adding a row here is
/// the whole change that takes.
fn required_kind(key: &str) -> Option<Kind> {
    match key {
        "taxonomy" | "conformance" => Some(Kind::File),
        BUNDLES | crate::migration::CONTENTS_KEY => Some(Kind::Directory),
        _ => None,
    }
}

/// Every path the manifest's `contents` declares, held to the tree.
///
/// [Spec 7](../../../../docs/spec/07-distribution-and-federation.md#publishing):
/// *"Every `contents` path a publisher writes is read. `taxonomy`, `bundles`,
/// `conformance` and `migrations` each reach a verb. A key that no verb reads is
/// a claim that a publisher makes and a consumer never sees."* This is that
/// sentence, and it reads the keys the manifest declares rather than a list
/// written here, so a key added to a manifest is covered on the day it arrives.
///
/// **The escape is checked as well as the existence.** `contents.bundles` may
/// name a path outside the package, because [`publish`] carries what it points
/// at inside and rewrites the scalar. No other key has a rewrite, so a `..`
/// under any other key would reach a consumer in the published manifest, which
/// is the one thing spec 7 says no artifact carries.
///
/// **The kind is checked as well as the existence**, against
/// [`required_kind`]. `exists` answers presence and not kind, so a `bundles`
/// naming a file passed it, [`stage`] then took the arm that is right for a
/// bundles directory inside the package, and the publish exited 0 with an
/// artifact that carried no bundle at all — 39 members down to 3 on this
/// repository's own package, with the published manifest still naming the file.
/// A `conformance` naming a directory published the same way and failed on the
/// machine of whoever installed it.
///
/// **The empty value is refused first, and the kind check cannot stand in for
/// it.** `join("")` is the package directory, which exists and which *is* a
/// directory, so `bundles: ""` and `migrations: ""` satisfy a kind check that
/// requires one. `migrations: ""` then reaches [`crate::migration::at`], which
/// reads the package directory and globs the manifest itself as a payload. Only
/// an arm of its own catches an empty value, and it runs before the escape, the
/// existence and the kind.
///
/// `contents.migrations` is read twice over, here for existence and kind and by
/// [`crate::migration::at`] for its payloads. This one runs first, so its
/// refusal is the one a publisher sees and that one is a second reading of a
/// path rather than a second definition of a rule.
///
/// **Every bad key is reported, not the first one.** [`agrees`] collects for the
/// same reason: a second run should not have to discover the second defect.
fn reachable(
    manifest: &str,
    directory: &Path,
    contents: &Mapping,
) -> Result<(), Vec<ResolveError>> {
    let mut refused = Vec::new();
    for entry in contents {
        let key = entry.key.value.as_str();
        // A value that is not a scalar used to be skipped here, and skipping it
        // meant neither rule below ran over it. A sequence holding `../../x`
        // reached a published manifest with the `..` in it, and one naming a
        // file that is not there published an artifact with a hole where the key
        // points, both at exit 0. Nothing downstream reads it either: `stage`
        // rewrites `bundles` alone, so there is no reader for whom a list of
        // paths under any key would mean anything.
        let Some(scalar) = entry.value.value.as_scalar() else {
            refused.extend(refusal(
                manifest,
                &format!(
                    "`contents.{key}` is not a path. Every value under `contents` names one file \
                     or one directory inside the package, and a publish reads each one before it \
                     writes anything"
                ),
            ));
            continue;
        };
        let declared = scalar.text.as_str();
        if declared.is_empty() {
            refused.extend(refusal(
                manifest,
                &format!(
                    "`contents.{key}` is empty. Every value under `contents` names one file or \
                     one directory inside the package, and an empty value names the package \
                     directory itself, which is neither. Write the path the key points at, or \
                     take the key out"
                ),
            ));
            continue;
        }
        let escapes = leaves(declared);
        if escapes && key != BUNDLES {
            refused.extend(refusal(
                manifest,
                &format!(
                    "`contents.{key}` names {declared}, which is outside the package. Only \
                     `contents.{BUNDLES}` may name a path outside the package, because publishing \
                     carries what that one points at inside the artifact and rewrites the scalar. \
                     Every other key would reach a consumer with the `..` in it"
                ),
            ));
            continue;
        }
        let at = directory.join(declared);
        if !at.exists() {
            let outside = match escapes {
                true => {
                    " A package directory copied out of the tree that holds its bundle library is \
                     not that tree. Publish from the tree the manifest was written for, or take a \
                     published artifact with `headwater taxonomy vendor`."
                }
                false => "",
            };
            refused.extend(refusal(
                manifest,
                &format!(
                    "`contents.{key}` names {declared}, and it is not there. A publish reads \
                     every path a manifest declares before it writes anything, so a key that \
                     names nothing on disk stops it.{outside}"
                ),
            ));
            continue;
        }
        let Some(needed) = required_kind(key) else {
            continue;
        };
        let found = Kind::of(&at);
        if found != needed {
            let (found, needed) = (found.name(), needed.name());
            refused.extend(refusal(
                manifest,
                &format!(
                    "`contents.{key}` names {declared}, which is a {found}. The verb that reads \
                     `contents.{key}` reads a {needed}, so publishing this would ship a key that \
                     is there and cannot be read"
                ),
            ));
        }
    }
    match refused.is_empty() {
        true => Ok(()),
        false => Err(refused),
    }
}

/// The whole artifact, in memory, before `out` exists.
///
/// The package directory, then the bundles where the manifest points them
/// outside it, then the manifest with that one scalar rewritten. The rewrite
/// replaces the staged bytes rather than writing over a file that was just
/// copied, so the manifest reaches the tree once.
///
/// [`reachable`] ran here and now runs in [`publish`], above the first reader of
/// a declared path. Nothing calls this without that call before it.
fn stage(
    root: &Path,
    directory: &Path,
    manifest: &Mapping,
) -> Result<Vec<Staged>, Vec<ResolveError>> {
    let name = manifest_name(root, directory);

    let mut staged = Vec::new();
    read_tree(directory, "", &mut staged)
        .map_err(|why| refusal(&display(root, directory), &why))?;

    let Some(bundles) = manifest
        .get("contents")
        .and_then(|node| node.value.as_map())
        .and_then(|contents| contents.get(BUNDLES))
    else {
        return Ok(staged);
    };
    let Some(scalar) = bundles.value.as_scalar() else {
        return Err(refusal(&name, "`contents.bundles` is not a path"));
    };
    if !leaves(&scalar.text) {
        // A path that stays inside was read with everything else.
        return Ok(staged);
    }

    read_tree(&directory.join(&scalar.text), BUNDLES, &mut staged)
        .map_err(|why| refusal(&name, &why))?;

    let Some(at) = staged.iter().position(|file| file.path == MANIFEST) else {
        return Err(refusal(&name, "the package carries no manifest to rewrite"));
    };
    let source = String::from_utf8(staged[at].bytes.clone())
        .map_err(|_| refusal(&name, "the manifest is not text"))?;
    let rewritten = splice(&source, bundles.span, BUNDLES).ok_or_else(|| {
        refusal(
            &name,
            "`contents.bundles` does not lie inside the manifest it was read from",
        )
    })?;
    staged[at].bytes = rewritten.into_bytes();
    Ok(staged)
}

/// Read a directory tree into the staged set, under a prefix inside the artifact.
fn read_tree(from: &Path, prefix: &str, into: &mut Vec<Staged>) -> Result<(), String> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(from)
        .map_err(|error| format!("cannot read {}: {error}", from.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    entries.sort();
    for entry in entries {
        // A name that is not UTF-8 used to be skipped, and the file then left no
        // trace anywhere: not in the artifact, not in the record, not in the
        // count the verb prints, and not in the exit code. A publisher whose
        // claim is that everything is read before anything is written cannot
        // drop a file it did not read, and the release record has nowhere to put
        // a path that is not text. `to_string_lossy` is good enough for a
        // message and is not good enough for a member path, which is the whole
        // asymmetry.
        let Some(name) = entry.file_name().and_then(|name| name.to_str()) else {
            return Err(format!(
                "cannot carry {}: the name is not UTF-8, and every path in a release record is \
                 text. Rename it, or take it out of the package",
                entry.display()
            ));
        };
        let path = match prefix.is_empty() {
            true => name.to_string(),
            false => format!("{prefix}/{name}"),
        };
        if entry.is_dir() {
            read_tree(&entry, &path, into)?;
            continue;
        }
        let bytes = std::fs::read(&entry)
            .map_err(|error| format!("cannot read {}: {error}", entry.display()))?;
        let mode = std::fs::metadata(&entry)
            .map_err(|error| format!("cannot read {}: {error}", entry.display()))?
            .permissions();
        into.push(Staged { path, bytes, mode });
    }
    Ok(())
}

/// Write the staged set into `out`, creating what it needs.
fn put(out: &Path, staged: &[Staged]) -> Result<(), String> {
    std::fs::create_dir_all(out).map_err(|error| format!("cannot create it: {error}"))?;
    for file in staged {
        let at = out.join(&file.path);
        if let Some(parent) = at.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
        }
        std::fs::write(&at, &file.bytes)
            .map_err(|error| format!("cannot write {}: {error}", file.path))?;
        std::fs::set_permissions(&at, file.mode.clone())
            .map_err(|error| format!("cannot write {}: {error}", file.path))?;
    }
    Ok(())
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
///
/// The source is handed in rather than read here. [`publish`] holds the two
/// version declarations to each other and needs the same file to do it, and two
/// reads of one path is the shape of the defect that check exists for.
fn migrations(
    root: &Path,
    directory: &Path,
    manifest: &Mapping,
    source: Source,
) -> Result<(), Vec<ResolveError>> {
    let name = manifest_name(root, directory);
    let payloads = crate::migration::at(directory, manifest)
        .map_err(|errors| crate::migration::as_errors(&name, &errors))?;
    if payloads.is_empty() {
        return Ok(());
    }

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
///
/// **The identity comes from the manifest and never from the record's header.**
/// [`identity`] states why, and it runs before the target directory is named,
/// because that name is what steers the removal below. It now also refuses a
/// name that is not a name, by [`names_a_package`], which is what makes the
/// target below always a directory of the package's own.
///
/// **One refusal below became unreachable, and no issue is filed for it.** The
/// maintained-package arm composes its message from `display(root, &target)`,
/// and [`display`] flattens a `..` lexically, so on a package named `..` the
/// target `<root>/packages/..` rendered as the empty string: an adopter was
/// told that "a directory is there and it carries no release record, so it is a
/// package somebody maintains" about their whole repository, with nothing
/// named. The predicate was right at its own level and the sentence was false
/// one level of scope out. `..` is the only name whose `display` flattens to
/// nothing, and [`identity`] refuses it before this arm can run, so the route
/// is closed rather than the message repaired. Every message [`identity`] adds
/// is composed from `beside` and a literal key, and never from a derived path,
/// which is why none of them can go false the same way.
pub fn vendor(root: &Path, fetched: &Path, pinned: &str) -> Result<Release, Vec<ResolveError>> {
    let name = display(root, fetched);
    let record =
        release::verify(fetched, pinned).map_err(|error| release::as_error(&name, &error))?;

    let declared = identity(root, fetched, &record)?;
    let target = root.join(PACKAGES).join(declared.replace('/', "-"));
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

/// The package a fetched artifact is, taken from the file the digest covers,
/// with the record's header held against it.
///
/// **The record's header is outside the digest and the manifest is inside it.**
/// [`release::members`] walks every file in the artifact except the record
/// itself, so `package.yml` — that it is there, and what it says — is covered by
/// the digest the consumer pinned, while the header lines of `release.yml` above
/// that digest are covered by nothing. An adversary who rewrites the header
/// moves no digest, and the honest pin still verifies. Until
/// [#297](https://github.com/headwater-ai/headwater/issues/297) that header
/// named the target directory of the vendor, so a record edited to name the
/// adopter's own package sent `remove_dir_all` at it and put the attacker's
/// bytes under its name, with the verb exiting 0.
///
/// **This is not a new rule, and no digest moves.**
/// [Spec 7](../../../../docs/spec/07-distribution-and-federation.md#publishing)
/// already says "every consumer-facing reader takes the name from the manifest:
/// the lookup under `packages/`, the release record, the vendor target directory
/// and the corpus descriptor", and the vendor target was the one reader in that
/// list that did not. Covering the header with the digest would be the other
/// repair and it is a different change: the digest field lives inside the file
/// it would hash, so it needs a canonical elided form and it moves every digest
/// anybody has published. Nothing here needs that, because the authenticated
/// declaration was already in the artifact — this reads it instead of the
/// unauthenticated one.
///
/// **What the digest buys the name, and what [`names_a_package`] buys it.** The
/// digest says a publisher wrote this name and that nobody has changed it since
/// the pin. It says nothing at all about the name being usable as a directory
/// name, and for a while nothing here checked that: `/` was turned into `-`, so
/// a name with a slash could not leave `packages/`, and `..` has no slash and
/// survived, which made the target of a package named `..` the adopter's own
/// root. [#314](https://github.com/headwater-ai/headwater/issues/314) records
/// what that cost — an artifact scattered over an adopter's repository on the
/// first vendor and the repository deleted on the second — and the grammar
/// below closes it, along with `.`, a YAML null, an absolute path and every
/// other spelling of a value that is not a name.
///
/// **The two guarantees are separate and both are needed.** The digest says the
/// name is one a publisher wrote; the grammar says the name is one that can be
/// joined onto a path. Neither implies the other, and a forged header now fails
/// the first while a publisher's own honest `package: ..` fails the second.
///
/// An artifact that carries no manifest reaches [`manifest_at`] and is refused
/// by the error of the read rather than by anything written here. **The narrow
/// claim is the true one**: removing `package.yml` from a *published* artifact
/// moves the digest and [`release::verify`] refuses it by name. An artifact that
/// never carried one is a different case, it verifies cleanly against a record
/// resealed over what it does hold, and the refusal an adopter then reads is a
/// raw `No such file or directory` where every other refusal on this path is a
/// composed sentence.
///
/// **What the comparison covers, and why each field is in or out.**
///
/// The name is compared and it is what the target is derived from. **The
/// derivation is the fix and the comparison is the report**: with the comparison
/// removed the adopter's package still survives, because the target no longer
/// comes from the header either way. The comparison is what says so out loud
/// instead of silently vendoring an artifact under a name its own record
/// disputes. Between them they cover the only header field that steers a write.
///
/// The version is compared. It steers nothing here — resolution takes the
/// version from the manifest, and the record's copy is read only by the `--to`
/// guard of `taxonomy diff` and `taxonomy migrate`, which takes no pin at all —
/// so this adds no protection that path did not have. It is compared because
/// [`release::compute`] derives both from the manifest, so no publisher on any
/// engine that writes a record this way can produce a disagreeing pair, and
/// because [`agrees`] holds the name and the version of a package together one
/// level down. Reporting half of a forged header and passing the other half
/// would be the odd thing to do.
///
/// The engine range is compared, and this one is a refusal moved rather than a
/// refusal added. The manifest's copy is enforced at resolve by [`base`], so an
/// adversary who strips `requires_engine` from the record delays the refusal to
/// the adopter's next resolve rather than escaping it. A delayed refusal is one
/// the adopter meets after the bytes are on disk, which is exactly what the
/// range in the record exists to prevent.
///
/// **Absent is not a value that can agree with another absent, for the name,
/// and neither is any other value that is not a name.** A manifest whose
/// `package:` is absent, one whose value is `""`, and one whose value is a
/// YAML null are all refused outright rather than compared. Two blanks compared
/// equal is [#298](https://github.com/headwater-ai/headwater/issues/298)'s
/// defect in [`agrees`], and here it would name the target directory
/// `packages/`. The guard is [`names_a_package`] and it reaches all three
/// spellings, because it asks what a name is rather than listing what a name is
/// not.
///
/// **The null case is a symptom of #298 closed here, and #298 itself stays
/// open.** This engine hands back a scalar's source text, and the source text
/// of a null is the literal `~`. So `is_empty()` is false, both sides compare
/// equal at `~`, and the artifact used to vendor into `packages/~`. The grammar
/// refuses `~` as a name, so no adopter gets that directory — and it teaches
/// [`agrees`] nothing about absent versus null, which is where the ambiguity is
/// born and what #298 holds. Read this as *the route into an adopter's
/// `packages/` is closed*, never as *the parse was fixed*.
///
/// The engine range is the opposite case and is compared as an [`Option`]: absent on
/// both sides is a publisher that states no floor, which spec 7 gives a meaning
/// to. The version sits between them — absent on both sides is a versionless
/// package, which the version pin refuses at resolve on its own terms, and a
/// second copy of that rule here would be a copy in a worse place.
///
/// **The name is compared before the version**, which is [`agrees`]'s ordering
/// and its reason: a package that is not the package you asked for makes the
/// question of its version moot. Every disagreement is reported, so a second run
/// does not have to discover the second one.
fn identity(root: &Path, fetched: &Path, record: &Release) -> Result<String, Vec<ResolveError>> {
    let manifest = manifest_at(fetched)?;
    let at = display(root, &fetched.join(release::RECORD));
    let beside = display(root, &fetched.join(MANIFEST));

    let declared = text(&manifest, "package").unwrap_or_default();
    if !names_a_package(&declared) {
        let states = match declared.is_empty() {
            true => "this states no `package:`".to_string(),
            false => format!("this states `package: {declared}`"),
        };
        return Err(refusal(
            &beside,
            &format!(
                "{states}, which is not a package name. A package name is one or more segments \
                 separated by `/`, and every segment opens and closes with a letter or a digit \
                 and otherwise holds letters, digits, `.`, `-` and `_`. It is the name a \
                 directory under `packages/` is created under and replaced under, so a value \
                 outside that grammar names a directory that belongs to somebody else: `..` \
                 names the adopter's own root, `.` names `packages/` itself, and an absent \
                 value would name `packages/`. The name in the record is not a substitute, \
                 because the record is the one file the release digest does not cover"
            ),
        ));
    }

    let mut refused = Vec::new();
    if record.package != declared {
        refused.extend(refusal(
            &at,
            &format!(
                "this declares `package: {}` and {beside}, which the release digest covers, \
                 declares `package: {declared}`. One artifact states two names of itself, and \
                 the record's is the one a consumer reads without opening the artifact. The \
                 manifest decides, so `{}` is the directory under `packages/` this would have \
                 replaced",
                record.package,
                declared.replace('/', "-"),
            ),
        ));
    }

    let carried = text(&manifest, "version").unwrap_or_default();
    if record.version != carried {
        refused.extend(refusal(
            &at,
            &format!(
                "this declares version `{}` and {beside}, which the release digest covers, \
                 declares `{carried}`. One artifact states two versions of itself, and each of \
                 them is what some reader downstream takes the release to be",
                record.version,
            ),
        ));
    }

    let range = text(&manifest, REQUIRES_ENGINE);
    if record.requires_engine != range {
        refused.extend(refusal(
            &at,
            &format!(
                "this declares {} and {beside}, which the release digest covers, declares {}. \
                 The record's copy is the one a consumer checks before the bytes land, so a \
                 record that states a different floor than the package does moves that refusal \
                 to the adopter's next resolve",
                stated(record.requires_engine.as_deref()),
                stated(range.as_deref()),
            ),
        ));
    }

    match refused.is_empty() {
        true => Ok(declared),
        false => Err(refused),
    }
}

/// An engine range as a message should say it, where absent is a state and not
/// an empty string.
fn stated(range: Option<&str>) -> String {
    match range {
        Some(range) => format!("`{REQUIRES_ENGINE}: {range}`"),
        None => format!("no `{REQUIRES_ENGINE}`"),
    }
}

/// Whether a string is a package name.
///
/// One or more segments separated by `/`, each matching
/// `[A-Za-z0-9]([A-Za-z0-9._-]*[A-Za-z0-9])?`. Spec 7's *Publishing* section
/// states the same grammar in prose, and this is the only reader of it.
///
/// **It is stated positively, and that is the whole of why it is complete.** A
/// blocklist would have to name `..`, then `.`, then `~`, then the empty
/// segment, then an absolute path, then a control character, and would still be
/// open to the next spelling of "not a name" that somebody writes. Everything
/// on that list is outside this grammar without being named by it.
///
/// **This is deliberately not [`leaves`], and the difference is the subject.**
/// `leaves` answers whether a *path* a publisher declared points out of the
/// package, and it is sound at its own level and false one level out:
/// [#303](https://github.com/headwater-ai/headwater/issues/303) records a
/// symlink walking past it, because a lexical reading of a declared path cannot
/// see the file system. A package name is a single scalar with no such
/// indirection in the question — measured on this path,
/// `std::fs::remove_dir_all` removes a symlink standing at `packages/<name>`
/// rather than following it, so a link there escapes nothing — which is what
/// lets a lexical rule be the whole guard here. Reusing `leaves`'s
/// [`std::path::Component`] scan would inherit an argument that has already
/// failed once, in this file, on a neighbouring key.
///
/// **What it costs, said out loud rather than discovered later.** It refuses a
/// name holding a character outside ASCII, and it refuses `+`. There is no
/// registry and so no migration path, so a publisher already using such a name
/// is refused outright: every name declared anywhere in this tree passes, and
/// no fixture moved. It does **not** refuse the names Windows reserves —
/// `con`, `nul`, `aux` — because refusing `con` costs a legitimate name for a
/// package about containers and buys nothing on the platform this engine is
/// tested on.
fn names_a_package(declared: &str) -> bool {
    !declared.is_empty()
        && declared.split('/').all(|segment| {
            let bytes = segment.as_bytes();
            let alphanumeric = |byte: &u8| byte.is_ascii_alphanumeric();
            matches!(bytes.first(), Some(byte) if alphanumeric(byte))
                && matches!(bytes.last(), Some(byte) if alphanumeric(byte))
                && bytes
                    .iter()
                    .all(|byte| alphanumeric(byte) || matches!(byte, b'.' | b'-' | b'_'))
        })
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

#[cfg(test)]
mod tests {
    use super::names_a_package;

    /// Every package name declared anywhere in this tree.
    ///
    /// The grammar has no registry behind it and so no migration path, which
    /// makes "nothing in this tree moves" a claim worth holding rather than
    /// asserting. `packages/headwater-standard/package.yml` declares the first
    /// one and the fixtures of `tests/publish.rs` and the conformance suite
    /// declare the rest.
    #[test]
    fn every_name_this_tree_declares_is_a_package_name() {
        for name in [
            "headwater/standard",
            "acme/headwater-taxonomy",
            "acme/fixture",
            "acme/taxonomy",
            "acme/work-items",
            "acme/victim",
            "acme/attacker",
            "audit/fixture",
            "headwater/fixture",
        ] {
            assert!(names_a_package(name), "{name} is declared in this tree");
        }
    }

    /// The values that reached a directory under an adopter's `packages/`, and
    /// the one that was already closed.
    ///
    /// Each of these was measured landing somewhere before
    /// [#314](https://github.com/headwater-ai/headwater/issues/314). `..` and
    /// `.` are the two that reached outside the directory the name is for, and
    /// the rest are litter. None of them is named by the grammar, which is the
    /// point of stating it positively.
    #[test]
    fn no_value_that_is_not_a_name_is_one() {
        for name in [
            "", "..", ".", "...", "~", "-x", ".hidden", "..-x", "/etc/hw", "a/../..", "a/", "/a",
            "a//b", "a b", "a\0b",
        ] {
            assert!(!names_a_package(name), "{name:?} is not a package name");
        }
    }

    /// The costs the doc comment states, held rather than left in prose.
    ///
    /// A name outside ASCII is refused and so is `+`. The names Windows
    /// reserves are not refused, because refusing `con` costs a legitimate name
    /// and buys nothing on the platform this engine is tested on. A `.`, a `-`
    /// and a `_` inside a segment are all fine, and the same characters at
    /// either end of one are not.
    #[test]
    fn the_boundary_of_the_grammar_is_where_the_doc_comment_says() {
        assert!(!names_a_package("acmé/fixture"));
        assert!(!names_a_package("acme/c++"));
        assert!(names_a_package("con"));
        assert!(names_a_package("nul"));
        assert!(names_a_package("acme/head.water_taxonomy-2"));
        assert!(!names_a_package("acme/.fixture"));
        assert!(!names_a_package("acme/fixture-"));
        assert!(names_a_package("a"));
        assert!(names_a_package("a/b/c"));
    }
}
