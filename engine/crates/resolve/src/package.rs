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

/// The manifest key that records the references a carried document writes which
/// resolve nowhere inside the artifact.
///
/// It is a mapping from an artifact-relative member to the artifact-relative
/// targets that member names and the artifact does not carry. [`references`]
/// refuses a dangling reference the key does not hold and reports one it does,
/// and [#619](https://github.com/headwater-ai/headwater/issues/619) is where the
/// staging was ruled: `headwater/standard` carries 122 such references over 51
/// distinct pairs, none of them resolvable in any artifact this project has
/// published, so a rule that refused every one of them would refuse the base
/// package on the day it landed.
///
/// **The key is written in artifact coordinates and it ships in the artifact.**
/// Every other path a manifest writes is read against the publisher's tree, and
/// this one is not, because what it describes is the artifact. A consumer who
/// opens a vendored package reads the same key against the tree they received.
pub const RECORDED_REFERENCES: &str = "unresolved_references";

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
pub(crate) fn contents_of(manifest: &Mapping) -> Mapping {
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

    let named = text(declaration, "package");
    let carries = text(root, "taxonomy");
    match (named.as_deref(), carries.as_deref()) {
        (Some(a), Some(b)) if a == b => {}
        (Some(a), Some(b)) => refused.extend(refusal(
            manifest,
            &format!(
                "this declares `package: {a}` and the taxonomy source it names, {}, declares \
                 `taxonomy: {b}`. One package states two names of itself, and each of them \
                 is what some reader downstream takes the package to be",
                source.name
            ),
        )),
        (declaration_value, source_value) => refused.extend(absent(
            manifest,
            &source.name,
            "name",
            "package",
            declaration_value,
            "taxonomy",
            source_value,
        )),
    }

    let declared = text(declaration, "version");
    let carried = text(root, "version");
    match (declared.as_deref(), carried.as_deref()) {
        (Some(a), Some(b)) if a == b => {}
        (Some(a), Some(b)) => refused.extend(refusal(
            manifest,
            &format!(
                "this declares version `{a}` and the taxonomy source it names, {}, \
                 declares `{b}`. One package states two versions of itself, and each of \
                 them is what some reader downstream takes the package to be",
                source.name
            ),
        )),
        (declaration_value, source_value) => refused.extend(absent(
            manifest,
            &source.name,
            "version",
            "version",
            declaration_value,
            "version",
            source_value,
        )),
    }

    if refused.is_empty() {
        return Ok(());
    }
    Err(refused)
}

/// The absent-adjacent half of [`agrees`]'s comparison: one side of the pair
/// is missing its key entirely, or both sides are. [`agrees`] itself peels off
/// the case where both sides are present — equal or different — before falling
/// through to this, so a call here only ever sees at least one `None`.
///
/// A key can be missing from a mapping entirely, or present with an empty or
/// blank value; `text` already collapses "present but blank" (a YAML null,
/// stored as the literal text `~`) and "present, explicit empty string" into
/// ordinary `Some` values, so `None` here means only one thing: the key is
/// absent. That distinction is what separates a message about a missing line
/// from a message about a blank one.
///
/// `subject` names what is being compared ("name" or "version") for the
/// both-absent message. `declaration_key`/`source_key` name the two keys —
/// distinct for the name pair (`package` vs `taxonomy`), identical for the
/// version pair (`version` vs `version`) — so the same wording serves both
/// callers.
fn absent(
    manifest: &str,
    source_name: &str,
    subject: &str,
    declaration_key: &str,
    declaration_value: Option<&str>,
    source_key: &str,
    source_value: Option<&str>,
) -> Vec<ResolveError> {
    match (declaration_value, source_value) {
        (None, None) => refusal(
            manifest,
            &format!(
                "this declares no `{declaration_key}` key, and the taxonomy source it names, \
                 {source_name}, declares no `{source_key}` key either. A package must state its \
                 own {subject} in at least one of the two, and this one states it in neither"
            ),
        ),
        (None, Some(b)) => refusal(
            manifest,
            &format!(
                "this declares no `{declaration_key}` key, and the taxonomy source it names, \
                 {source_name}, declares `{source_key}: {b}`. The {subject} is present in \
                 {source_name} and absent from this manifest — one of the two lines is \
                 missing, not blank"
            ),
        ),
        (Some(a), None) => refusal(
            manifest,
            &format!(
                "this declares `{declaration_key}: {a}` and the taxonomy source it names, \
                 {source_name}, declares no `{source_key}` key. The {subject} is present in \
                 this manifest and absent from {source_name} — one of the two lines is \
                 missing, not blank"
            ),
        ),
        (Some(_), Some(_)) => {
            unreachable!("agrees peels off the both-present case before calling absent")
        }
    }
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

/// The base and every bundle the package ships, in name order: the maximal
/// selection a consumer could make.
///
/// This is not any consumer's taxonomy and it is not meant to be. It is the most
/// the artifact can hold, and it is what a payload target is held against: a
/// value some selection can hold is a value this artifact can hold.
/// [Spec 2](../../../../docs/spec/02-taxonomy-model.md#customization-by-composition)
/// is what makes it a taxonomy at all — a bundle is add-only, so any subset of
/// them commutes and resolves, and the maximal subset is the one that contains
/// every declared closure. That last clause is the reason this is one set rather
/// than a loop over the bundles one at a time: a bundle declares `requires:`, and
/// `decision-record` requires `design-spec`, so base plus that one bundle is not
/// a configuration any consumer can hold.
///
/// The order is the sorted directory name, and it is a formality: the confluence
/// check certifies that every legal order yields one taxonomy, and the `founded`
/// record that does depend on order is discarded by the one caller.
///
/// A directory under the bundle root that holds no `bundle.yml` is skipped
/// rather than refused. It is a bundle no consumer can select, which is a
/// publish-time question this path does not own, and a refusal here would fire
/// only where a migration payload happens to exist. `bundles/README.md`, a
/// regular file this repository's own bundle root carries, is skipped by the
/// same rule.
fn shipped(
    root: &Path,
    directory: &Path,
    contents: &Mapping,
    base: Source,
) -> Result<Vec<Source>, Vec<ResolveError>> {
    let mut out = vec![base];

    let Some((at, names)) = bundle_names(root, directory, contents)? else {
        return Ok(out);
    };
    for name in names {
        let path = at.join(name).join("bundle.yml");
        out.push(Source::read(&path, &display(root, &path), Role::Overlay)?);
    }
    Ok(out)
}

/// The bundle root a `contents` block names, and every bundle under it, in name
/// order.
///
/// `None` where the block names no bundle root, which is a package that ships
/// no bundles and is not an error anywhere.
///
/// [`shipped`] is one caller and [`crate::selection`] is the other, and they
/// share this rather than each reading `contents.bundles` for themselves. Two
/// functions enumerating one declaration differently is
/// [#581](https://github.com/headwater-ai/headwater/issues/581), which is live
/// in this crate as this is written.
pub(crate) fn bundle_names(
    root: &Path,
    directory: &Path,
    contents: &Mapping,
) -> Result<Option<(PathBuf, Vec<std::ffi::OsString>)>, Vec<ResolveError>> {
    let Some(bundles) = text(contents, "bundles") else {
        return Ok(None);
    };
    let at = directory.join(&bundles);
    let mut names: Vec<std::ffi::OsString> = std::fs::read_dir(&at)
        .map_err(|error| {
            refusal(
                &manifest_name(root, directory),
                &format!(
                    "`contents.bundles` names `{bundles}`, and there is no directory to read \
                     there: {error}"
                ),
            )
        })?
        .filter_map(|entry| entry.ok())
        .filter(|entry| at.join(entry.file_name()).join("bundle.yml").is_file())
        .map(|entry| entry.file_name())
        .collect();
    names.sort();
    Ok(Some((at, names)))
}

/// The base with every bundle the package ships, resolved.
///
/// [`shipped`] builds the source list and this resolves it. One resolution per
/// publish, held by [`publish_at`] and read by both [`crate::template::holds`]
/// and [`migrations`]. It is what
/// [#387](https://github.com/headwater-ai/headwater/issues/387) asked for: spec
/// 7 states the confluence guarantee for every release, so a package whose
/// bundles collide is refused whether or not it carries a migration payload.
///
/// The prefix on a refusal says which question reached it, because the only
/// failures a maximal selection can take are `NotConfluent` and `AddCollides`
/// and neither one names the reason it was asked.
fn maximal_from(
    root: &Path,
    directory: &Path,
    name: &str,
    contents: &Mapping,
    base: Source,
) -> Result<crate::Resolution, Vec<ResolveError>> {
    crate::resolve(&shipped(root, directory, contents, base)?).map_err(|errors| {
        let mut out = refusal(
            name,
            "this package is published with every bundle it ships, and that set does not resolve:",
        );
        out.extend(errors);
        out
    })
}

/// Refuse a publish whose taxonomy reads a name nothing in it declares.
///
/// [`crate::rules::referential`] is the rule and this is the sentence a
/// publisher gets in front of it. The findings themselves are the consumer's
/// own, rendered by the consumer's own code, so the two verbs report one defect
/// in one wording — which is what
/// [#582](https://github.com/headwater-ai/headwater/issues/582) means by "the
/// same check". `said` is what the publish was published *as*, because the two
/// paths ship two different selections and the remedy differs: a plain publish
/// ships every bundle, and a recipe ships the list it names.
///
/// The count is stated because the findings are addresses and the names are
/// fewer: seven addresses can read four names, and a publisher who is not told
/// the denominator reads seven problems where there are four.
fn dangles(at: &str, taxonomy: &Mapping, said: &str) -> Result<(), Vec<ResolveError>> {
    let found = crate::rules::referential(taxonomy);
    if found.is_empty() {
        return Ok(());
    }
    let names: std::collections::BTreeSet<String> = crate::rules::dangling(taxonomy)
        .into_iter()
        .map(|one| one.name)
        .collect();
    let counted = if names.len() == 1 {
        "one name".to_string()
    } else {
        format!("{} names", names.len())
    };
    let mut out = refusal(
        at,
        &format!(
            "{said} {counted} nothing declares. A consumer of this artifact is refused at \
             `headwater taxonomy resolve`, so it is refused here:"
        ),
    );
    out.extend(found);
    Err(out)
}

/// The same rule over what a flattening publish is about to write.
///
/// It reads the rendered `taxonomy.yml` back rather than asking the upstream
/// resolution, because the rendered bytes are the artifact that reaches the
/// adopter and a verification of a rendered artifact against its source is a
/// verification of the wrong thing. [`crate::flatten::equivalent`] already reads
/// it back the same way, one step earlier, for the same reason.
///
/// The refusal names the recipe file and the selection in it. A flattened
/// artifact declares no bundles, so its consumer has no selection left to get
/// wrong: this is the whole of what referential integrity can say about that
/// artifact, and the publisher is the only party who can act on it.
fn publish_assembly_referential(
    root: &Path,
    recipe: &crate::assembly::Assembly,
    flattened: &crate::flatten::Flattened,
) -> Result<(), Vec<ResolveError>> {
    let at = display(root, &recipe.at);
    let rendered = Source::from_text(&at, Role::Taxonomy, &flattened.taxonomy)?;
    let Some(taxonomy) = rendered.root.value.as_map() else {
        return Err(refusal(
            &at,
            "the flattened taxonomy this recipe produced is not a mapping",
        ));
    };
    dangles(
        &at,
        taxonomy,
        &format!(
            "this recipe selects [{}], and that selection reads",
            recipe.from.bundles.join(", ")
        ),
    )
}

/// The same resolution, for a caller that holds a package directory and nothing
/// else.
///
/// [`publish_at`] does not use it: that path has already read the taxonomy
/// source to hold the two version declarations to each other, and reading it a
/// second time is the shape of the defect [`agrees`] exists for. So the rule has
/// one implementation, [`maximal_from`], and this wrapper supplies only the
/// reads a publish had already made.
pub fn maximal(root: &Path, directory: &Path) -> Result<crate::Resolution, Vec<ResolveError>> {
    let manifest = manifest_at(directory)?;
    let contents = contents_of(&manifest);
    let source = taxonomy_source(root, directory, &contents)?;
    maximal_from(
        root,
        directory,
        &manifest_name(root, directory),
        &contents,
        source,
    )
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

/// The directory a package sits in, and the manifest it declares.
///
/// # A manifest that is not a mapping is reported, and does not stop the search
///
/// `packages/` holds nothing anybody hand-edits — spec 7: only `taxonomy
/// vendor` and `taxonomy publish --from` write under it — so a manifest that
/// parses into something that is not a mapping is always an anomaly, never a
/// scratch file this walk should quietly step around. But the anomaly belongs
/// to the directory it sits in, not to the name being searched for, and the
/// name asked for may still sit in a different, well-formed directory: a
/// residue `vendor` already tolerates (`packages/<name>~aside`,
/// `packages/~staging`), or simply a different package that happens to be
/// broken while the one being resolved is not. [`sources`], [`publish`],
/// [`find_version`], [`located`] and `vendor`'s own collision guard all want
/// the package that resolves over a report about the first thing that went
/// wrong, when the two are not the same directory. So this walk remembers
/// every non-mapping manifest it meets and keeps looking; a single match found
/// afterward returns `Ok` as though the broken sibling had never been read.
/// A manifest that does not even parse — a genuine YAML syntax error, not
/// merely a value that is not a mapping — joins the same `broken` list rather
/// than aborting the walk with [`crate::source::load`]'s own error: a syntax
/// mistake in one directory is exactly as much somebody else's problem as a
/// non-mapping value is, and the walk has to keep going past it for the same
/// reason. Only where the walk exhausts without exactly one match do the
/// remembered manifests join the refusal, ahead of the summary line they
/// would otherwise be mistaken for.
///
/// # Two directories that declare the same name are both named, not resolved silently
///
/// The walk never stops at the first match. `packages/` holds nothing anybody
/// hand-edits, but a person can still put a second directory there by hand —
/// following `headwater init`'s own suggestion to copy a package directory in,
/// beside one `vendor` already installed — and two directories that each
/// declare the same `package:` name is a real collision, not a shape this
/// engine should pick a winner for by alphabetical accident. So every
/// directory is read to the end, and every manifest whose declared name
/// matches is kept, sorted into two groups by whether it is genuine [`ASIDE`]
/// residue [`vendor`]'s own atomic swap left standing.
///
/// **A directory's name alone does not prove what made it.** The grammar
/// refuses `~` in a *package* name, but nothing refuses it in a *directory*
/// name a person chooses by hand, and copying a package directory into
/// `packages/` under any name at all — including one that happens to end in
/// `~aside` — is exactly the `headwater init`-suggested workflow this
/// function exists to stop from resolving silently. So a directory counts as
/// residue only when its name carries the suffix **and** it carries a release
/// record [`release::at`] can read. The second test is not incidental:
/// [`vendor`] itself, a few dozen lines below, refuses to install over a
/// directory with no release record on the same grounds — "a directory is
/// there and it carries no release record, so it is a package somebody
/// maintains rather than one that was vendored" — and every directory
/// `vendor` ever renames to `<flattened>~aside` is, at the moment of that
/// rename, the package it had just replaced, which [`publish`]'s own artifact
/// always carries the record for. A hand-copied source directory is never a
/// copy of a published artifact — #369's own report calls it the authored
/// source, not a vendored one — so it never carries this record, whatever it
/// is named. A real collision is decided on the ordinary matches alone: one
/// resolves, and two or more are refused by name. Only where no ordinary
/// match exists does a lone, record-carrying residue match answer instead,
/// the same fallback `vendor`'s own collision guard already relies on — read
/// exactly this way, so a kill mid-swap and a genuine duplicate that happens
/// to reuse the reserved suffix are never mistaken for each other.
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

    let mut broken: Vec<ResolveError> = Vec::new();
    let mut matches: Vec<(PathBuf, Mapping)> = Vec::new();
    let mut residues: Vec<(PathBuf, Mapping)> = Vec::new();
    for directory in entries {
        let manifest = directory.join(MANIFEST);
        if !manifest.is_file() {
            continue;
        }
        let loaded = match crate::source::load(&manifest) {
            Ok(loaded) => loaded,
            Err(errors) => {
                // A syntax error here is the same kind of anomaly a
                // non-mapping manifest is, below: it belongs to the directory
                // it sits in, and the walk must keep going so a match sitting
                // in a later directory (unreachable before the early return
                // this function used to take) still resolves.
                broken.extend(errors);
                continue;
            }
        };
        let Some(map) = loaded.value.as_map() else {
            broken.extend(refusal(
                &manifest_name(root, &directory),
                "the manifest is not a mapping",
            ));
            continue;
        };
        if text(map, "package").as_deref() == Some(name) {
            // A directory only counts as vendor-made residue when its name
            // carries the reserved suffix *and* it carries a release record --
            // the same proof `vendor` itself demands, in the opposite
            // direction, before it will treat a directory as one of its own.
            // The suffix alone is not proof of provenance: nothing stops a
            // person from naming a hand-copied directory `<name>~aside`.
            let carries_suffix = directory
                .file_name()
                .and_then(|component| component.to_str())
                .is_some_and(|component| component.ends_with(ASIDE));
            let is_residue = carries_suffix && release::at(&directory).is_ok();
            match is_residue {
                true => residues.push((directory, map.clone())),
                false => matches.push((directory, map.clone())),
            }
        }
    }

    match matches.len() {
        1 => {
            let (directory, map) = matches.into_iter().next().expect("len checked above");
            Ok((directory, map))
        }
        0 => match residues.len() {
            1 => {
                let (directory, map) = residues.into_iter().next().expect("len checked above");
                Ok((directory, map))
            }
            _ => {
                broken.extend(refusal(
                    PACKAGES,
                    &format!("no package under `{PACKAGES}/` declares `{name}`"),
                ));
                Err(broken)
            }
        },
        _ => Err(refusal(
            PACKAGES,
            &format!(
                "more than one package under `{PACKAGES}/` declares `{name}`: {}",
                matches
                    .iter()
                    .map(|(directory, _)| display(root, directory))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        )),
    }
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
/// and [`crate::migration::holds`] checks each one against the taxonomies being
/// published and the version it is published as. Two taxonomies and not one,
/// because a package with bundles ships one for each selection a consumer makes:
/// [`migrations`] says which half of a step reads which end of that set. A
/// payload the publisher cannot ship correctly stops the publish, rather than
/// reaching a digest that makes it permanent.
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
/// **The undo answers a returned error and not a signal.** [`found::Found::unwind`]
/// runs from the `Err` arm of phase 2's own `match`, so a kill or a lost
/// machine never reaches it: `out` is left holding whatever `put` had written
/// at the instant it died, in the common case every artifact file and no
/// [`release::RECORD`], because that record is the last thing this phase
/// writes. That is [`vendor`]'s asymmetry with this verb: `vendor` stages
/// beside its target and renames, so a kill there leaves the target whole or
/// untouched, never partial. Here a kill leaves `out` partial, and the next
/// run's [`found::observe`] is what meets it. It reads [`release::at`] and, if
/// the record is missing or does not parse, says the leftovers are consistent
/// with a killed publish and safe to delete on that understanding — it cannot
/// tell that state apart from a directory a person filled with something else,
/// since neither carries a record, so it reports what it knows rather than
/// guessing which one this is. See [#355].
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
/// [#355]: https://github.com/headwater-ai/headwater/issues/355
pub fn publish(root: &Path, name: &str, out: &Path) -> Result<Release, Vec<ResolveError>> {
    publish_delivered(root, name, out).map(|done| done.release)
}

/// [`publish`], and it hands back how the artifact reached `--out` as well as
/// the record.
///
/// A caller that reports a publish to a person or to a script uses this one.
/// [`publish`] answers the narrower question — what was published — and drops
/// [`Published::delivery`] on the floor, which is safe only because nothing it
/// returns is a claim about atomicity.
pub fn publish_delivered(
    root: &Path,
    name: &str,
    out: &Path,
) -> Result<Published, Vec<ResolveError>> {
    let (directory, manifest) = find(root, name)?;
    publish_at(root, &directory, &manifest, out)
}

/// Write the published artifact of a package whose directory the caller
/// already holds, bypassing [`find`].
///
/// [`find`] locates a package by the name its own manifest declares, under
/// `packages/`, and that lookup is what [`publish`] and `taxonomy resolve`
/// share. A repository that both publishes a package and consumes it cannot
/// use that lookup for both roles over one directory: `taxonomy vendor`
/// refuses to install over a directory that carries no release record, so it
/// refuses the very directory a maintained source sits in, and two
/// directories that both declare the package's name are refused as a
/// collision rather than resolved to whichever sorts first, naming both —
/// [#369](https://github.com/headwater-ai/headwater/issues/369). So a
/// repository in that position keeps its authored source somewhere `find`
/// never walks for consumption, and needs a way to publish it that does not
/// go through `find` either.
///
/// This is that way. It reads the manifest at `directory` directly and runs
/// every step [`publish`] runs beyond the lookup: the same reachability
/// check, the same two-declarations-agree check, the same migration-payload
/// check, and the same stage-then-write-then-unwind sequence. Nothing here is
/// a new publish, which is why it is one line different from [`publish`]
/// rather than a second copy of the function.
///
/// `directory` need not sit under `packages/` at all, and ordinarily does
/// not: it is the caller's own maintained source, wherever that source lives
/// in the repository.
pub fn publish_from(
    root: &Path,
    directory: &Path,
    out: &Path,
) -> Result<Release, Vec<ResolveError>> {
    publish_from_delivered(root, directory, out).map(|done| done.release)
}

/// [`publish_from`], and it hands back how the artifact reached `--out` as well
/// as the record. [`publish_delivered`] carries the argument for the pair.
pub fn publish_from_delivered(
    root: &Path,
    directory: &Path,
    out: &Path,
) -> Result<Published, Vec<ResolveError>> {
    let manifest = manifest_at(directory)?;
    publish_at(root, directory, &manifest, out)
}

/// Write the flattened artifact a named assembly derives from a package found
/// under `packages/`.
pub fn publish_assembly(
    root: &Path,
    name: &str,
    assembly: &str,
    out: &Path,
) -> Result<Published, Vec<ResolveError>> {
    let (directory, manifest) = find(root, name)?;
    publish_assembly_at(root, &directory, &manifest, assembly, out)
}

/// Write the flattened artifact a named assembly derives from a source
/// directory the caller already holds.
pub fn publish_assembly_from(
    root: &Path,
    directory: &Path,
    assembly: &str,
    out: &Path,
) -> Result<Published, Vec<ResolveError>> {
    let manifest = manifest_at(directory)?;
    publish_assembly_at(root, directory, &manifest, assembly, out)
}

/// The publish sequence for an assembly after its source directory and manifest
/// are known.
///
/// This keeps the same read, write, and undo boundary as [`publish_at`]. The
/// generated material replaces the source tree only after every recipe input is
/// in memory, and [`found::Found`] unwinds the same output states when either a
/// file write or release-record write fails.
fn publish_assembly_at(
    root: &Path,
    directory: &Path,
    manifest: &Mapping,
    assembly: &str,
    out: &Path,
) -> Result<Published, Vec<ResolveError>> {
    let declared = manifest_name(root, directory);
    let contents = contents_of(manifest);
    reachable(root, &declared, directory, &contents)?;
    let source = taxonomy_source(root, directory, &contents)?;
    agrees(&declared, manifest, &source)?;

    let recipe = crate::assembly::read(root, directory, manifest, assembly)?;
    let flattened = crate::flatten::materialize(root, directory, manifest, &recipe)?;
    publish_assembly_referential(root, &recipe, &flattened)?;
    let staged = stage_flattened(directory, &declared, &flattened)?;
    let found = found::observe(out).map_err(|why| refusal(&display(root, out), &why))?;

    match deliver(root, out, &staged, &flattened.manifest) {
        Ok((release, delivery)) => Ok(Published {
            release,
            dropped: flattened.dropped,
            delivery,
        }),
        Err(errors) => {
            found.unwind(out);
            Err(errors)
        }
    }
}

/// What a publish produced.
///
/// The release record, how that record reached `--out`, and the `contents` keys the flattened manifest does not
/// declare although the source did. The second half travels out of the run that
/// decided it rather than being re-derived by whoever reports it: two functions
/// answering one question separately is the defect
/// [#581](https://github.com/headwater-ai/headwater/issues/581) exists for, and
/// a reporter that drifted from [`crate::flatten::DROPPED`] would tell a
/// publisher the artifact carries something it does not. `dropped` is empty for
/// every source that declares none of those keys.
pub struct Published {
    pub release: Release,
    pub dropped: Vec<String>,
    /// How the artifact reached `--out`. Empty of judgment: the reporter decides
    /// what to say about it, and [`deliver`] is the only thing that can know it.
    pub delivery: Delivery,
}

/// The publish sequence shared by [`publish`] and [`publish_from`], once each
/// has settled on a directory and the manifest inside it by whichever route
/// it uses.
fn publish_at(
    root: &Path,
    directory: &Path,
    manifest: &Mapping,
    out: &Path,
) -> Result<Published, Vec<ResolveError>> {
    let declared = manifest_name(root, directory);

    // A directory that already carries a release record was written by
    // `vendor`, never by a person, on the same terms `vendor`'s own guard
    // takes in the opposite direction ([`holds_the_same_package`]'s doc
    // comment on the sibling function). Publishing it would republish
    // whatever bytes `vendor` last installed there, which is not necessarily
    // what the maintained source declares now: `find` matches a directory by
    // the name its manifest states, and a vendored copy's manifest still
    // states the same name `vendor` copied it under. So `--package <name>`
    // reaches this directory again once nothing else declares that name,
    // silently republishing a stale vendored copy under a normal-looking
    // `published …` line.
    if release::at(directory).is_ok() {
        return Err(refusal(
            &declared,
            "this directory carries a release record, so `taxonomy vendor` installed it rather \
             than a person maintaining it by hand. Publishing it would republish whatever bytes \
             `vendor` last put there, which may no longer be what the maintained source \
             declares. Name the maintained source directly with `--from <dir>` instead",
        ));
    }

    let contents = contents_of(manifest);

    // Every declared path is held to the tree here, before the first reader of
    // one runs. `taxonomy_source` reads `contents.taxonomy` for the resolver,
    // and while this ran inside `stage` that one key never reached the refusal
    // below: a missing taxonomy source came back as `cannot read
    // …/packages/x/../../elsewhere/taxonomy.yml: No such file or directory`,
    // which names neither the manifest nor the key and carries the `..` that
    // publication exists to remove. One position for one rule, and `stage` no
    // longer holds a second copy of the call.
    reachable(root, &declared, directory, &contents)?;

    let source = taxonomy_source(root, directory, &contents)?;
    agrees(&declared, manifest, &source)?;

    // The base with every bundle the package ships, resolved once for the whole
    // publish. [#387](https://github.com/headwater-ai/headwater/issues/387) is
    // why it is here rather than inside `migrations`: spec 7 states the
    // confluence guarantee for every release, and resolving the shipped set only
    // where a migration payload happens to exist made the proof conditional on
    // something unrelated to it. Two readers take this one resolution — the
    // template reader below, which needs it on every publish, and the payload
    // check, which used to build its own.
    let widest = maximal_from(root, directory, &declared, &contents, source.clone())?;

    // Referential integrity over the widest set, which is the selection a plain
    // publish ships: every bundle the package carries. `maximal_from` above
    // asks whether that set merges and whether it commutes, and until
    // [#582](https://github.com/headwater-ai/headwater/issues/582) nothing
    // asked whether every name it reads is declared. A package that resolved
    // and dangled published with exit 0 and the adopter met the refusal.
    //
    // Here rather than inside `maximal_from`, because that function has a
    // second caller — `maximal`, which `taxonomy diff` reaches — and a
    // comparison of two versions is not a release. The rule belongs to the
    // publish.
    dangles(
        &declared,
        &widest.taxonomy,
        "this package is published with every bundle it ships, and that set reads",
    )?;

    // Before `--out` is observed, so a template refusal fires with no output
    // directory in existence and `found::Found`'s undo is never entangled with
    // it. Spec 7's "Nothing is written until everything is read" holds unchanged.
    crate::template::holds(root, directory, manifest, &widest.taxonomy)?;

    let found = found::observe(out).map_err(|why| refusal(&display(root, out), &why))?;

    migrations(root, directory, manifest, source, &widest)?;

    let staged = stage(root, directory, manifest)?;
    // After `found::observe` and before `deliver`, which is where `migrations`
    // one line above already returns from without unwinding: nothing has been
    // written yet, because [`deliver`] is what reaches the disk. The
    // `!out.exists()` assertion in the CLI's publish target holds that.
    integrity(&staged, &declared)?;

    match deliver(root, out, &staged, manifest) {
        Ok((release, delivery)) => Ok(Published {
            release,
            dropped: Vec::new(),
            delivery,
        }),
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
/// **[`vendor`] does not use this module, and the reason is the two states.**
/// Both of them undo by deleting, which is affordable only because the `--out`
/// precondition refuses a directory that holds anything. A vendor's target is a
/// full package directory, so there is no state here to return it to, and that
/// verb stages beside the target and swaps instead. Its doc comment carries the
/// measurement.
///
/// [#271]: https://github.com/headwater-ai/headwater/issues/271
mod found {
    use super::OUT_STAGING as STAGING;
    use crate::release::{self, ReleaseError};
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
    ///
    /// # What a complete record does and does not tell the next run
    ///
    /// [`release::RECORD`] is the last file [`publish`]'s write phase writes —
    /// its own doc comment states the order — so [`held`] reads it with
    /// [`release::at`] to split one case out of the refusal. It parses, and
    /// `out` already holds a publish this run refuses to repeat, which is the
    /// message this function has always given. It reports [`ReleaseError::Absent`]
    /// or any other error, and `out` holds no complete record — which is what a
    /// kill used to leave and no longer does: [`super::deliver`] assembles the
    /// artifact beside `out` and moves it in one step, so a killed run leaves
    /// `out` as it found it. What remains is a directory a person filled with
    /// something unrelated, which is what the refusal now says, and a mount
    /// point at `out`, which cannot be moved onto and is named there too. It
    /// still only reports. Nothing here deletes anything, and a
    /// later run has to be told to by whoever reads the message. See [#355].
    ///
    /// # One name is not a state and is refused here anyway
    ///
    /// [`super::deliver`] assembles the artifact at `--out` with
    /// [`super::OUT_STAGING`] appended, and clears that path before it writes.
    /// So an `--out` that ends in the suffix is an `--out` a second publish
    /// would sweep, and this is the one function both publish paths reach before
    /// either has written anything. `--out` has no grammar to keep the two
    /// apart, so the reservation is stated rather than derived.
    ///
    /// [#355]: https://github.com/headwater-ai/headwater/issues/355
    pub(super) fn observe(out: &Path) -> Result<Found, String> {
        if out
            .file_name()
            .is_some_and(|name| name.as_encoded_bytes().ends_with(STAGING.as_bytes()))
        {
            return Err(format!(
                "the output path ends in `{STAGING}`, which a publish reserves for the directory \
                 it assembles an artifact in beside `--out` and clears before it writes. Publish \
                 into a path that does not end in it"
            ));
        }
        match std::fs::read_dir(out) {
            Ok(mut entries) => match entries.next() {
                Some(_) => Err(held(out)),
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

    /// The refusal for a non-empty `--out`, naming a complete publish already
    /// there separately from everything else a directory can hold.
    ///
    /// See `observe`'s own doc comment for why [`release::at`] is what draws
    /// that one line, and not a finer one.
    fn held(out: &Path) -> String {
        match release::at(out) {
            Ok(_) => "the output directory holds files already, and a published \
                      artifact is every file under its root. Publish into a directory \
                      that does not exist yet"
                .to_string(),
            Err(ReleaseError::Absent(_)) => format!(
                "the output directory holds files but no {record}. A publish assembles the \
                 artifact beside this path and moves it here in one step, so a killed run leaves \
                 this directory as it found it and what is here is something else. The one \
                 exception is an output path that is a mount point, which cannot be moved onto \
                 and is written into directly. Check what is there before you delete it. Publish \
                 into a directory that does not exist yet",
                record = release::RECORD,
            ),
            Err(error) => format!(
                "the output directory holds files and its {record} does not read back cleanly: \
                 {error}. A publish assembles the artifact beside this path and moves it here in \
                 one step, so a killed run leaves this directory as it found it. Check what is \
                 there before you delete it. Publish into a directory that does not exist yet",
                record = release::RECORD,
                error = error.to_string().trim_end(),
            ),
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

/// The generated flattened package, expressed in the same staged-file shape
/// ordinary publication passes to [`put`].
///
/// `name` is the source manifest a message names, because that is the file a
/// publisher edits: the generated manifest [`carried`] reads is derived from it
/// and exists only in memory.
fn stage_flattened(
    directory: &Path,
    name: &str,
    flattened: &crate::flatten::Flattened,
) -> Result<Vec<Staged>, Vec<ResolveError>> {
    let mode = std::fs::metadata(directory.join(MANIFEST))
        .map_err(|error| {
            refusal(
                &directory.join(MANIFEST).display().to_string(),
                &format!("cannot read it for its file mode: {error}"),
            )
        })?
        .permissions();
    let mut staged = vec![
        Staged {
            path: MANIFEST.to_string(),
            bytes: crate::render::render(&flattened.manifest).into_bytes(),
            mode: mode.clone(),
        },
        Staged {
            path: "taxonomy.yml".to_string(),
            bytes: flattened.taxonomy.as_bytes().to_vec(),
            mode: mode.clone(),
        },
    ];
    staged.extend(flattened.assets.iter().map(|asset| Staged {
        path: asset.path.clone(),
        bytes: asset.bytes.clone(),
        mode: mode.clone(),
    }));
    staged.sort_by(|left, right| left.path.cmp(&right.path));
    integrity(&staged, name)?;
    Ok(staged)
}

/// Referential integrity over the staged artifact: every path the manifest
/// declares and every path a carried document writes, held to the staged set.
///
/// Two readings run here and they answer different questions. [`carried`] reads
/// the `contents` key scalars, which is what a manifest declares.
/// [`references`] reads the bodies of the files the artifact carries, which is
/// what a document points a reader at. The first was
/// [#582](https://github.com/headwater-ai/headwater/issues/582) and the second
/// [#619](https://github.com/headwater-ai/headwater/issues/619), and the gap
/// between them shipped an artifact whose prose named seven files no consumer
/// received. Both collect rather than stopping at the first offender, and both
/// are reported out of one refusal.
///
/// [`reachable`] holds a `contents` path to the publisher's tree, and this
/// holds the same key to the artifact. They are two readings and not one rule
/// twice: `reachable` answers whether a publisher can read what the manifest
/// declares, and a key can pass it and still reach a consumer as a hole. Two
/// mechanisms produce that hole and both were live at
/// [#581](https://github.com/headwater-ai/headwater/issues/581):
///
/// - **The stagers enumerate the manifest differently from each other.**
///   [`crate::flatten::contents`] copies through every source key it does not
///   filter and [`crate::flatten::assets`] writes two of them, so `conformance`
///   was declared by one function and written by neither. That is an engine
///   defect and the instance fix is in `flatten`.
/// - **A declared directory holds no file.** `reachable` admits it — the
///   directory is there and it is a directory — and the walk that carries bytes
///   then carries none, on either publish path. That is a publisher's tree and
///   no fix in this engine removes it, which is why the guard is general and
///   permanent rather than a second per-key check beside [`doctrine_at`].
///
/// **It runs over the staged set in memory, before `put`.** [`doctrine_at`]'s
/// doc comment argues the same placement from `vendor`'s side and the argument
/// here is stronger: nothing has been written when this fires, so `--out` does
/// not exist to be unwound. The alternative — a post-condition over the written
/// artifact — hands a publisher a directory the run has to take back.
///
/// **It reads the manifest out of the staged bytes rather than out of the
/// `Mapping` a caller holds.** [`stage`] rewrites `contents.bundles` into the
/// staged `package.yml` at its end while the in-memory manifest still names the
/// library outside the package, so the staged bytes are the only copy of the
/// manifest the artifact will actually carry. Reading them keeps no second copy
/// of the rewrite rule here.
///
/// **A non-scalar and an empty value are skipped, and that is not a gap.**
/// [`reachable`] refuses both, on both publish paths, above every call of this.
/// Refusing them again would be a second definition of one rule, and the
/// message a publisher reads would depend on which check happened to run first.
///
/// **The declared scalar is resolved into an artifact path before the compare,
/// and comparing the raw string refuses two things a publisher may write.**
/// `headwater/standard` declares `assemblies: assemblies/` and `doctrine:
/// doctrine/`, and a staged path never carries the trailing separator. And
/// [#303](https://github.com/headwater-ai/headwater/issues/303) ruled that
/// `sub/../taxonomy.yml` names `taxonomy.yml` and publishes, because
/// [`reachable`] judges a path by where it resolves rather than by the string a
/// manifest wrote. [`member_path`] is that resolution and the suite pins both.
///
/// **Every bad key is reported, not the first one.** [`reachable`] and
/// [`agrees`] collect for the same stated reason: a second run should not have
/// to discover the second defect.
fn integrity(staged: &[Staged], manifest: &str) -> Result<(), Vec<ResolveError>> {
    let Some(file) = staged.iter().find(|file| file.path == MANIFEST) else {
        return Err(refusal(
            manifest,
            "the staged artifact carries no manifest to read its own members from",
        ));
    };
    let text = String::from_utf8(file.bytes.clone())
        .map_err(|_| refusal(manifest, "the staged manifest is not text"))?;
    let loaded = headwater_yaml::load(&text).map_err(|errors| {
        vec![ResolveError::new(
            ResolveErrorKind::SourceRefused(headwater_yaml::error::render(&errors)),
            manifest,
            "",
            headwater_yaml::Span::default(),
        )]
    })?;
    let Some(map) = loaded.value.as_map() else {
        return Err(refusal(manifest, "the staged manifest is not a mapping"));
    };

    let mut refused = carried(staged, manifest, map).err().unwrap_or_default();
    refused.extend(references(staged, manifest, map).err().unwrap_or_default());
    match refused.is_empty() {
        true => Ok(()),
        false => Err(refused),
    }
}

/// Every path the staged `contents` keys declare, held to the staged set.
///
/// Split out of [`integrity`] so the two readings stay two readings: this one
/// asks what the manifest declares, and [`references`] asks what a carried
/// document writes.
fn carried(staged: &[Staged], manifest: &str, map: &Mapping) -> Result<(), Vec<ResolveError>> {
    let mut refused = Vec::new();
    for entry in &contents_of(map) {
        let key = entry.key.value.as_str();
        let Some(scalar) = entry.value.value.as_scalar() else {
            continue;
        };
        let declared = scalar.text.as_str();
        let Some(member) = member_path(declared) else {
            continue;
        };
        if !held(staged, &member) {
            refused.extend(refusal(
                manifest,
                &format!(
                    "`contents.{key}` names {declared}, and the artifact does not carry it. Every \
                     key a published manifest declares is a path a consumer opens, so publishing \
                     this would ship a manifest naming a member no consumer received. A declared \
                     directory that holds no file is the common way to reach this"
                ),
            ));
        }
    }
    match refused.is_empty() {
        true => Ok(()),
        false => Err(refused),
    }
}

/// The artifact path a declared `contents` scalar names, or `None` where the
/// scalar names nothing inside the artifact.
///
/// A staged path is built out of directory entries, so it carries no `.`
/// segment, no `..` segment and no trailing separator. A declared scalar may
/// carry all three and still be legal, so the compare needs the scalar in the
/// staged form rather than as written.
///
/// `None` covers the two values [`reachable`] has already refused above every
/// call of this — an empty scalar, and a path that climbs above the package —
/// and [`carried`] skips rather than reporting them, so one rule keeps one
/// message.
fn member_path(declared: &str) -> Option<String> {
    lands_inside("", declared)
}

/// Where a path written inside `directory` lands in the artifact, or `None`
/// where it names the artifact root or climbs out of it.
///
/// `directory` is artifact-relative with `/` separators, and empty for a member
/// at the artifact root. The resolution is lexical, because the artifact carries
/// no symlink a publish left undereferenced and the question a reference asks is
/// where the text points.
///
/// **`None` is two answers and the callers want the same thing from both.**
/// [`member_path`] resolves a `contents` scalar against the artifact root, and
/// the two values it can hand back a `None` for — an empty scalar and a path
/// above the package — are the two [`reachable`] has already refused with its
/// own message. [`references`] resolves a link against the directory of the file
/// that wrote it, and a `None` there is a link that deliberately leaves the
/// artifact. Neither is this rule's to report.
fn lands_inside(directory: &str, declared: &str) -> Option<String> {
    let mut parts: Vec<&str> = match directory.is_empty() {
        true => Vec::new(),
        false => directory
            .split('/')
            .filter(|part| !part.is_empty())
            .collect(),
    };
    for part in declared.split('/') {
        match part {
            "" | "." => continue,
            ".." => {
                parts.pop()?;
            }
            other => parts.push(other),
        }
    }
    match parts.is_empty() {
        true => None,
        false => Some(parts.join("/")),
    }
}

/// Whether the staged set holds an artifact path, as a file or as a directory
/// above one.
///
/// A `contents` key and a prose link both name a directory sometimes, and a
/// staged set is a list of files, so a directory is held when a file sits under
/// it. The two readings of [`integrity`] ask this the same way on purpose.
fn held(staged: &[Staged], at: &str) -> bool {
    let prefix = format!("{at}/");
    staged
        .iter()
        .any(|file| file.path == at || file.path.starts_with(&prefix))
}

/// Every reference a carried document writes, held to the staged set.
///
/// [`carried`] reads what the manifest declares and this reads what a document
/// says. They are the same rule over two populations, and until
/// [#619](https://github.com/headwater-ai/headwater/issues/619) only the first
/// ran: a change that dropped members from `headwater/standard` left fifteen
/// links across seven carried files pointing at files no consumer received, and
/// publish reported nothing. The declared half refused a planted `contents`
/// scalar with exit 1 on the same tree, so the gap was measured rather than
/// reasoned.
///
/// # What it reads
///
/// Every staged member whose bytes are text, and every inline Markdown link on a
/// line that no fence and no code span covers. A `.yml` member is read as well
/// as a `.md` one, because a bundle manifest carries prose in its comments and
/// two of them in `headwater/standard` write a link.
///
/// A destination is judged only when it names a path inside the artifact. A
/// scheme, an empty destination and a bare fragment are passed over, and so is a
/// link that climbs out of the artifact: `../../spec/07-…` written for a
/// publisher's own tree is a statement about a directory the artifact never had,
/// and where it climbs clear of the root it is nothing this publish can decide.
///
/// # The recorded population, and why the rule is staged
///
/// The absolute bar — refuse every reference that resolves nowhere — was
/// measured against this repository's own package before it was written, and it
/// refuses `headwater/standard`: 122 references over 51 distinct pairs and 9
/// carried files, naming 23 targets under `spec/`, `evaluations/`,
/// `obligations/` and `decisions/`, **none of which was resolvable in any
/// artifact this project has ever published**. They are links written for the
/// publisher's `docs/` layout, and repairing them needs a ruling on what a
/// doctrine link to a publisher's own specification becomes in a consumer's
/// tree. That ruling is not this rule's.
///
/// So [`RECORDED_REFERENCES`] names the population and this refuses what the
/// population does not hold — the class a publish *creates*, which is the class
/// that shipped the fifteen. **The identity is the pair `(member, resolved
/// target)`**, both artifact-relative, both derived from the source alone, and
/// neither carrying a version or a digest, so a republish of the same source
/// records the same pairs. A pair the record holds and the artifact no longer
/// dangles is not refused either: a repair is not a defect, and the record
/// shrinking is how the population is meant to end.
fn references(staged: &[Staged], manifest: &str, map: &Mapping) -> Result<(), Vec<ResolveError>> {
    let admitted = recorded(map);
    let mut refused = Vec::new();

    // The record is scoped to the artifact it ships in, and a pair naming a
    // member this artifact does not carry is refused rather than ignored. A
    // record that outlives its member is a standing admission: the pair admits
    // nothing today and admits the first document published at that path
    // tomorrow, with nobody having decided that. `publish --assembly` is how a
    // record reaches an artifact it was not written for, because a flattened
    // package takes a new member layout, and `flatten::manifest` drops the key
    // for that reason.
    for (member, target) in &admitted {
        if !held(staged, member) {
            refused.extend(refusal(
                manifest,
                &format!(
                    "`{RECORDED_REFERENCES}` records `{member}` against {target}, and the artifact \
                     does not carry {member}. A record names what one artifact carries, so a pair \
                     whose member never shipped admits nothing today and admits whatever is \
                     published at that path later. Delete the pair, or publish the member it names"
                ),
            ));
        }
    }

    for file in staged {
        let Ok(text) = std::str::from_utf8(&file.bytes) else {
            continue;
        };
        let directory = match file.path.rfind('/') {
            Some(at) => &file.path[..at],
            None => "",
        };
        for written in links(text) {
            let target = destination(&written);
            if target.is_empty() || target.contains("://") || target.starts_with("mailto:") {
                continue;
            }
            let Some(at) = lands_inside(directory, &target) else {
                continue;
            };
            if held(staged, &at) || admitted.contains(&(file.path.clone(), at.clone())) {
                continue;
            }
            let member = &file.path;
            refused.extend(refusal(
                manifest,
                &format!(
                    "`{member}` writes the reference {written}, which names {at}, and the artifact \
                     does not carry it. Every relative reference a published document writes is a \
                     path a consumer follows, so publishing this would ship a document pointing at \
                     a member no consumer received. Repoint the reference, carry the target, or \
                     record the pair under `{RECORDED_REFERENCES}` in the manifest where it \
                     predates this publish"
                ),
            ));
        }
    }
    match refused.is_empty() {
        true => Ok(()),
        false => Err(refused),
    }
}

/// The `(member, target)` pairs a manifest records as resolving nowhere inside
/// the artifact.
///
/// One reading, called by [`references`] on the staged manifest and by
/// [`recorded_references`] on a written artifact, so the gate and the report
/// never disagree about what the key says. A malformed value is skipped rather
/// than refused: the key admits a reference and admitting nothing is the safe
/// direction, so a value nothing can read leaves the reference refused.
fn recorded(map: &Mapping) -> std::collections::BTreeSet<(String, String)> {
    let mut pairs = std::collections::BTreeSet::new();
    let Some(entries) = map
        .get(RECORDED_REFERENCES)
        .and_then(|node| node.value.as_map())
    else {
        return pairs;
    };
    for entry in entries {
        let Some(member) = member_path(entry.key.value.as_str()) else {
            continue;
        };
        let Some(targets) = entry.value.value.as_seq() else {
            continue;
        };
        for target in targets {
            let Some(scalar) = target.value.as_scalar() else {
                continue;
            };
            let Some(at) = member_path(scalar.text.as_str()) else {
                continue;
            };
            pairs.insert((member.clone(), at));
        }
    }
    pairs
}

/// The path a link destination names, with the fragment and the query removed
/// and every percent escape decoded.
///
/// A destination is a URL and a path on disk is not, so `a%20b.md` names
/// `a b.md` and a reader who follows the link opens that file. Reading the
/// escape as written reports a file that is there as a file that is not, which
/// is a refused publish over a correct link. `?` opens a query, which no path
/// carries and no file name here holds.
///
/// A malformed escape stays as written, because a destination this cannot read
/// is one this rule has no business rewriting.
fn destination(written: &str) -> String {
    let path = written
        .split('#')
        .next()
        .unwrap_or_default()
        .split('?')
        .next()
        .unwrap_or_default();
    let bytes = path.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        match bytes[at] {
            b'%' if at + 2 < bytes.len() => {
                let pair = std::str::from_utf8(&bytes[at + 1..at + 3])
                    .ok()
                    .and_then(|text| u8::from_str_radix(text, 16).ok());
                match pair {
                    Some(byte) => {
                        out.push(byte);
                        at += 3;
                    }
                    None => {
                        out.push(bytes[at]);
                        at += 1;
                    }
                }
            }
            byte => {
                out.push(byte);
                at += 1;
            }
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| path.to_string())
}

/// The destination of every inline Markdown link in `text` that no code block
/// and no code span covers.
///
/// A code span, a fenced block and an indented block are read past, because
/// prose about this engine prints a path inside all three, and a rule that
/// refuses a publish over an example is a rule the first publisher it meets
/// turns off. A reference-style link and an HTML anchor are not read, and that
/// is a narrowing this states rather than hides: the population it was measured
/// against writes neither.
///
/// # A fence closes on its own character, and a boolean cannot say that
///
/// The first cut toggled one flag on ` ``` ` or on `~~~`, so a `~~~` line inside
/// a backtick fence closed it, every following line read as prose, and the line
/// that closed the real fence opened a new one. **Everything after an imbalance
/// went unread**, which is a check reporting success while not looking — the
/// failure this whole rule exists to refuse, wearing the other face. A fence
/// therefore remembers the character that opened it and the length of the run,
/// and only a run of the same character at that length or longer closes it. A
/// closing fence carries no information string, so a line with anything else on
/// it stays content.
///
/// **The information string is read at both ends.** CommonMark refuses a
/// backtick run as an opener when the text after it carries a backtick, and the
/// first cut of this tested that at the closing end alone. A line like
/// ` ```a`b ` therefore opened a fence that nothing ever closed, and every line
/// to the end of the file went unread. A tilde run takes any information string,
/// so the rule belongs to the backtick.
///
/// # An indented block is code here, and the boundary is deliberate
///
/// Four spaces after a blank line open an indented code block, and this reads
/// past one for the same reason it reads past a fence: `CLAUDE.md` and this
/// repository's specification both print a command that way. The narrowing is
/// that CommonMark opens no indented block inside a list item, and this does, so
/// a link written in a list continuation indented four spaces is not read. A
/// missed reference is the cost, and a refused publish over a printed example is
/// what it buys.
fn links(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    // The character that opened the fence and the length of its run, where one
    // is open.
    let mut fence: Option<(char, usize)> = None;
    let mut indented = false;
    let mut previous_blank = true;
    for line in text.lines() {
        let opener = line.trim_start();
        if let Some((character, length)) = fence {
            let run = opener.chars().take_while(|held| *held == character).count();
            if run >= length && opener.trim_start_matches(character).trim().is_empty() {
                fence = None;
            }
            continue;
        }
        let blank = opener.is_empty();
        let deep = line.starts_with("    ") || line.starts_with('\t');
        if !blank {
            if deep && (indented || previous_blank) {
                indented = true;
                previous_blank = false;
                continue;
            }
            indented = false;
        }
        previous_blank = blank;
        for character in ['`', '~'] {
            let run = opener.chars().take_while(|held| *held == character).count();
            if run < 3 {
                continue;
            }
            // The same information-string rule the closer already applies, on
            // the other end. CommonMark refuses a backtick run as an opener when
            // the text after it carries a backtick, because that text would be
            // ambiguous with a code span. Reading it as an opener here opens a
            // fence that nothing ever closes, and **every line to the end of the
            // file then goes unread** — the invisibility this rule exists to
            // refuse, reached through the one end that had no test. A tilde run
            // takes any information string, so the rule is the backtick's alone.
            if character == '`' && opener.trim_start_matches('`').contains('`') {
                continue;
            }
            fence = Some((character, run));
            break;
        }
        if fence.is_some() {
            continue;
        }
        let mut outside = String::new();
        let mut spanned = false;
        for character in line.chars() {
            match character {
                '`' => spanned = !spanned,
                _ if !spanned => outside.push(character),
                _ => {}
            }
        }
        let characters: Vec<char> = outside.chars().collect();
        let mut at = 0;
        while at < characters.len() {
            if characters[at] != '[' {
                at += 1;
                continue;
            }
            let Some(close) = (at..characters.len()).find(|index| characters[*index] == ']') else {
                break;
            };
            if characters.get(close + 1) != Some(&'(') {
                at = close + 1;
                continue;
            }
            let Some(end) = (close + 2..characters.len()).find(|index| characters[*index] == ')')
            else {
                break;
            };
            let destination: String = characters[close + 2..end].iter().collect();
            let destination = destination
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .trim_start_matches('<')
                .trim_end_matches('>');
            if !destination.is_empty() {
                found.push(destination.to_string());
            }
            at = end + 1;
        }
    }
    found
}

/// What a published artifact records under [`RECORDED_REFERENCES`], as
/// `<member> -> <target>` lines in a stable order.
///
/// The report half of [`references`]. A publish refuses a dangling reference the
/// record does not hold and says nothing about it; this is how a publisher is
/// told what the record admits, and it reads the manifest the artifact carries
/// rather than a second copy of the key. An artifact whose manifest does not
/// read yields nothing, because the publish that wrote it has already been
/// through [`integrity`] and a refusal is not this function's to raise.
pub fn recorded_references(artifact: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(artifact.join(MANIFEST)) else {
        return Vec::new();
    };
    let Ok(loaded) = headwater_yaml::load(&text) else {
        return Vec::new();
    };
    let Some(map) = loaded.value.as_map() else {
        return Vec::new();
    };
    recorded(map)
        .into_iter()
        .map(|(member, target)| format!("{member} -> {target}"))
        .collect()
}

/// Which kind of thing a `contents` key's reader opens.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    File,
    Directory,
}

impl Kind {
    /// The kind a path on disk is, where it is one of these two.
    pub(crate) fn of(at: &Path) -> Kind {
        match at.is_dir() {
            true => Kind::Directory,
            false => Kind::File,
        }
    }

    pub(crate) fn name(self) -> &'static str {
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
/// - `assemblies` is a directory: [`crate::assembly::read`] opens a named
///   recipe under it.
/// - `migrations` is a directory: [`crate::migration::at`] reads it with
///   `read_dir` and globs `*.yml` out of it.
/// - `doctrine` is a directory: [`doctrine_at`] resolves it against the fetched
///   artifact inside [`vendor`], and the CLI names the installed path. That
///   reader opens a listing and never a file.
/// - `templates` is a directory: [`crate::template::holds`] globs `*.md` out of
///   it, and `crate::flatten::assets` walks the same path for an assembly.
///
/// **A key that is not here keeps the existence check and nothing more.** That
/// is the seam of this table. [`reachable`] still walks the keys the manifest
/// declares rather than this list, so a key nobody reads yet is held to being
/// there, and gains a kind on the day something reads it. Adding a row here is
/// the whole change that takes. `doctrine` is the row that arrived that way, and
/// `templates` is the second, with
/// [#378](https://github.com/headwater-ai/headwater/issues/378) as the reader
/// that took it.
///
/// [`crate::flatten::members`] reads the same table for the same reason, so a
/// flattening publish carries a member as the kind its reader opens rather than
/// as whatever the walk that carries it happened to be written for.
pub(crate) fn required_kind(key: &str) -> Option<Kind> {
    match key {
        "taxonomy" | "conformance" => Some(Kind::File),
        BUNDLES | ASSEMBLIES | DOCTRINE | TEMPLATES | crate::migration::CONTENTS_KEY => {
            Some(Kind::Directory)
        }
        _ => None,
    }
}

/// Every path the manifest's `contents` declares, held to the tree.
///
/// [Spec 7](../../../../docs/spec/07-distribution-and-federation.md#publishing):
/// *"Every `contents` path a publisher writes is read. `taxonomy`, `bundles`,
/// `assemblies`, `conformance` and `migrations` each reach a verb. A key that
/// no verb reads is a claim that a publisher makes and a consumer never sees."* This is that
/// sentence, and it reads the keys the manifest declares rather than a list
/// written here, so a key added to a manifest is covered on the day it arrives.
///
/// **The escape is checked by where a path resolves, not by the string a
/// manifest writes.** [#303] found two mechanisms a lexical check over the
/// declared string could not see: `sub/../taxonomy.yml` never leaves the
/// package and used to be refused for it, and a symlink whose declared name
/// carries no `..` at all can still resolve anywhere `read_tree` will then
/// follow and copy. [`settled`] resolves what is on disk before the compare,
/// so a symlink is judged by its target and a `..` that returns is not an
/// escape. `contents.bundles` may still resolve outside the package, because
/// [`publish`] carries what it points at inside and rewrites the scalar — but
/// the bound is `root`, not the filesystem: a bundle library lives somewhere
/// in this repository or it is refused, and `bundles: /` is one of the cases a
/// fixture pins. No other key has a rewrite, so a path that reaches a consumer
/// in the published manifest at all is the one thing spec 7 says no artifact
/// carries.
///
/// [#303]: https://github.com/headwater-ai/headwater/issues/303
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
    root: &Path,
    manifest: &str,
    directory: &Path,
    contents: &Mapping,
) -> Result<(), Vec<ResolveError>> {
    let mut refused = Vec::new();
    let real_directory = settled(directory);
    let real_root = settled(root);
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
        let at = directory.join(declared);
        let real_at = settled(&at);
        let escapes = !at_or_inside(&real_directory, &real_at);
        if key != BUNDLES {
            if escapes {
                refused.extend(refusal(
                    manifest,
                    &format!(
                        "`contents.{key}` names {declared}, which resolves outside the package. \
                         Only `contents.{BUNDLES}` may name a path outside the package, because \
                         publishing carries what that one points at inside the artifact and \
                         rewrites the scalar. Every other key would reach a consumer with a path \
                         it cannot follow, whether the manifest wrote the `..` itself or a \
                         symlink resolves to one"
                    ),
                ));
                continue;
            }
        } else if !at_or_inside(&real_root, &real_at) {
            refused.extend(refusal(
                manifest,
                &format!(
                    "`contents.{BUNDLES}` names {declared}, which resolves outside this \
                     repository. A bundle library may sit anywhere inside it, because publishing \
                     carries it inside the artifact — it may not sit outside the tree the \
                     publish is reading"
                ),
            ));
            continue;
        }
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
    let package_boundary = settled(directory);
    read_tree(directory, "", &package_boundary, &mut staged)
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
        // A path that stays inside was read with everything else, and it keeps
        // the name the manifest wrote, so that name is where the bundles are.
        staged.retain(|file| !a_bundles_own_corpus(&file.path, &scalar.text));
        return Ok(staged);
    }

    // The bundles walk is held to `root`, not to `directory`: `reachable`
    // already refused a `contents.bundles` that resolves outside the
    // repository, so a symlink this walk meets inside that bound is a path
    // the same rule already let through.
    let repo_boundary = settled(root);
    read_tree(
        &directory.join(&scalar.text),
        BUNDLES,
        &repo_boundary,
        &mut staged,
    )
    .map_err(|why| refusal(&name, &why))?;
    // The bundles walk wrote everything under `BUNDLES`, which is where the
    // rewrite below points the manifest, so that is the prefix the exception is
    // read against.
    staged.retain(|file| !a_bundles_own_corpus(&file.path, BUNDLES));

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

/// Whether a staged path is a bundle's own reference corpus, which is the one
/// exception to publication taking the package directory whole.
///
/// **`at` is where the bundles sit inside the artifact**, and it is not always
/// [`BUNDLES`]: a `contents.bundles` that leaves the package is rewritten to
/// `bundles` by [`stage`], and one that stays inside keeps the name the
/// manifest wrote. A bundle root is one segment under that, so what this
/// answers to is `<at>/<bundle>/fixtures/…` and nothing else. A `fixtures`
/// directory elsewhere in the package, or deeper inside a bundle, is carried
/// like any other file, because spec 7 states the exception over one path
/// rather than over a name. A *file* named `fixtures` at a bundle root is
/// carried too, because the exception is a directory.
///
/// [#518]: https://github.com/headwater-ai/headwater/issues/518
fn a_bundles_own_corpus(path: &str, at: &str) -> bool {
    let at = at.trim_start_matches("./").trim_end_matches('/');
    let inside = match at.is_empty() || at == "." {
        true => path,
        false => match path
            .strip_prefix(at)
            .and_then(|rest| rest.strip_prefix('/'))
        {
            Some(rest) => rest,
            None => return false,
        },
    };
    let mut segments = inside.split('/');
    let _bundle = segments.next();
    matches!(
        (segments.next(), segments.next()),
        (Some(FIXTURES), Some(_))
    )
}

/// Read a directory tree into the staged set, under a prefix inside the
/// artifact.
///
/// **Every entry is judged by where it resolves, not by what its own name
/// says**, the same rule [`reachable`] already holds a declared `contents`
/// path to. `contents` only names what a manifest chose to write down, and
/// this walk reads whatever is actually under the directory whether a key
/// names it or not — [#303] planted a symlink under a package directory that
/// no manifest key declared, and this walk carried it into the artifact as an
/// ordinary file, dereferenced. `boundary` is [`settled`] once by the caller:
/// the package directory for the main walk, `root` for the bundles walk,
/// matching the wider bound [`reachable`] holds `contents.bundles` to.
///
/// [#303]: https://github.com/headwater-ai/headwater/issues/303
fn read_tree(
    from: &Path,
    prefix: &str,
    boundary: &Path,
    into: &mut Vec<Staged>,
) -> Result<(), String> {
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
        if !at_or_inside(boundary, &settled(&entry)) {
            return Err(format!(
                "cannot carry {}: it resolves outside the tree a publish may read from, whether \
                 the entry itself is a symlink or an ancestor of it is. A publish carries only \
                 what a package or its declared bundle library contains, dereferenced or not. \
                 Point it inside that tree, or take it out of the package",
                entry.display()
            ));
        }
        if entry.is_dir() {
            read_tree(&entry, &path, boundary, into)?;
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

/// The suffix a publish appends to `--out` to name the directory it assembles an
/// artifact in before it moves that artifact into place.
///
/// It is the free-path analogue of [`STAGING`], and it needs a guard that
/// [`STAGING`] does not. A `vendor` target is a directory under `packages/` whose
/// name a manifest declares, and `~` is 0x7E, above every byte
/// `names_a_package` admits, so no package can be named into that path. `--out`
/// has no grammar at all, so the reservation has to be stated: [`found::observe`]
/// refuses an `--out` whose final component ends in this suffix, and that
/// refusal is what makes the path below a path this verb owns rather than a path
/// a publisher might have chosen.
pub const OUT_STAGING: &str = "~staging";

/// The directory a publish assembles `--out`'s artifact in, or `None` for a
/// path that has no final component to append to.
///
/// A **sibling** of `--out`, and both halves of that matter. The rename at the
/// end is then a rename inside one directory, so it is on one filesystem by
/// construction and cannot report `EXDEV` for a reason the caller could have
/// avoided. And the parents `create_dir_all` makes to reach the staging
/// directory are exactly the parents `--out` needs, so the swap needs no second
/// directory pass and [`found::Found::unwind`] still finds the chain it observed
/// to be absent.
///
/// `None` is `--out /` and `--out foo/..`: paths with no final component, where
/// appending would produce a child rather than a sibling and the rename would be
/// a directory moved onto its own parent. [`deliver`] writes those the way this
/// verb always wrote every path, because a new refusal for a path that publishes
/// today would be a regression bought with nothing.
fn out_staging(out: &Path) -> Option<PathBuf> {
    let mut name = out.file_name()?.to_os_string();
    name.push(OUT_STAGING);
    Some(out.with_file_name(name))
}

/// Put the artifact at `--out`, having assembled every byte of it somewhere else
/// first.
///
/// # What this is for
///
/// [`put`] writes an artifact one file at a time and [`release::RECORD`] last,
/// so a publish killed inside it used to leave files at `--out` with no record —
/// the state [`found::held`]'s second arm reports, and the state
/// [#485](https://github.com/headwater-ai/headwater/issues/485) asked for a flag
/// to delete. The flag cannot be written: the predicate that would fire it is
/// *files at `--out` and no record*, which is byte-for-byte what a directory
/// holding somebody's unrelated work looks like, so it would be a recursive
/// delete on a directory about which the run has established nothing. The window
/// is closed here instead, and then there is nothing to clear.
///
/// [`vendor`] already argued this and its doc comment named this verb as the
/// weaker case. The two now stage the same way.
///
/// # The three ways a rename can refuse, all three measured
///
/// Onto an **empty** directory it succeeds, and [`found::observe`] has already
/// established that `--out` is empty or absent, so the ordinary path is the one
/// that works. Onto a **non-empty** directory it is `ENOTEMPTY`, which is only
/// reachable when something filled `--out` between the observation and here; the
/// run refuses, having written nothing into `--out`. Onto a **mount point** it is
/// `EBUSY`, because the kernel will not move a directory over a mount, and
/// `EXDEV` is the same shape from the other side. Publishing into a mounted
/// volume works today and is a thing continuous integration does, so those two
/// fall back to writing straight into `--out` rather than refusing. That
/// fallback is the one configuration where a killed publish can still leave
/// files at `--out`, and it is why [`found::held`] names a mount point rather
/// than claiming the guarantee without one.
///
/// # What removes the staging directory
///
/// Every exit path here calls [`clear_staging`], including the successful one:
/// the rename takes `<out>~staging/`[`ASSEMBLY`] and leaves `<out>~staging`
/// itself, holding its marker, to be swept. A run killed anywhere leaves the
/// directory behind, so the next run sweeps it before it writes — a file an
/// earlier run left there would otherwise be carried into an artifact that
/// nothing staged.
///
/// **That sweep refuses rather than guesses**, and [`clear_staging`] carries the
/// argument. A publish removes a directory at that path only when the directory
/// holds the [`MARKER`] a publish writes into it before it writes anything else.
/// Anything else there is somebody's, and the run refuses with a message naming
/// the path.
fn deliver(
    root: &Path,
    out: &Path,
    staged: &[Staged],
    manifest: &Mapping,
) -> Result<(Release, Delivery), Vec<ResolveError>> {
    let named = display(root, out);
    let Some(staging) = out_staging(out) else {
        let record = write_artifact(&named, out, staged, manifest)?;
        return Ok((record, Delivery::Direct(Direct::NoSibling)));
    };
    clear_staging(&staging).map_err(|why| refusal(&display(root, &staging), &why))?;
    let assembled = staging.join(ASSEMBLY);
    if let Err(why) = claim_staging(&staging) {
        return Err(refusal(&display(root, &staging), &why));
    }
    let record = match write_artifact(&named, &assembled, staged, manifest) {
        Ok(record) => record,
        Err(errors) => {
            let _ = clear_staging(&staging);
            return Err(errors);
        }
    };
    match std::fs::rename(&assembled, out) {
        Ok(()) => {
            let _ = clear_staging(&staging);
            Ok((record, Delivery::Renamed))
        }
        Err(error) if carries_a_mount(&error) => {
            let _ = clear_staging(&staging);
            let record = write_artifact(&named, out, staged, manifest)?;
            Ok((
                record,
                Delivery::Direct(Direct::Mount {
                    error: error.to_string(),
                }),
            ))
        }
        Err(error) => {
            let _ = clear_staging(&staging);
            Err(refusal(
                &named,
                &format!(
                    "the artifact was assembled beside it and cannot be moved into place: \
                     {error}. Nothing was written into the output directory. Publish into a \
                     directory that does not exist yet"
                ),
            ))
        }
    }
}

/// How the artifact reached `--out`, decided by [`deliver`] and reported by
/// whoever ran the publish.
///
/// # Why this travels beside the record rather than inside it
///
/// [`release::Release`] is what [`release::render`] writes into `release.yml`
/// and what [`release::at`] reads back, so a field added there would enter the
/// published artifact and move the digest a consumer pins. This is a fact about
/// the *act* of publishing on one machine, not about the artifact, so it leaves
/// [`deliver`] as the second half of a pair and never as a member of the record.
///
/// # Why both values are reported and not only the weaker one
///
/// A signal that appears only when the weaker path is taken cannot be told
/// apart from an engine too old to know the difference, which is the same
/// absence-read-as-satisfaction defect one level up.
/// [`release::document`] therefore writes `delivery` on every publish, with
/// `renamed` for the atomic path and `direct` for both fallbacks.
/// [#664](https://github.com/headwater-ai/headwater/issues/664) is the issue,
/// where a publish into a mount point wrote the whole artifact by the weaker
/// route and said so nowhere on any of its three surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delivery {
    /// The artifact was assembled beside `--out` and moved into place by one
    /// `rename(2)`. A run killed at any moment leaves `--out` either untouched
    /// or complete.
    Renamed,
    /// The artifact was written file by file straight into `--out`. A run killed
    /// during the write leaves files there with no release record, which is the
    /// state [`found::held`] reports to the next run.
    Direct(Direct),
}

/// Why a publish wrote straight into `--out`.
///
/// **There are two of these arms and not one.** The mount point is the one
/// [#664](https://github.com/headwater-ai/headwater/issues/664) names; the
/// missing sibling is the one it does not, and a fix that wired only the first
/// would leave a second silent fallback behind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direct {
    /// `--out` has no final path component, so no `<out>~staging` sibling can be
    /// formed to assemble in. `--out /` and `--out foo/..` are the shapes.
    ///
    /// Presently unreachable through the command line — `found::observe` refuses
    /// `foo/..` because the directory it resolves to is not empty, and `/`
    /// refuses on permissions — which is why it is wired rather than left: it is
    /// unreachable by accident and not by argument.
    NoSibling,
    /// The rename onto `--out` was refused, and the refusal was one
    /// [`carries_a_mount`] classifies: `--out` is a mount point, or it is on
    /// another filesystem. The error is carried so the report can quote the
    /// kernel rather than paraphrase it.
    Mount { error: String },
}

impl Delivery {
    /// The value `delivery` takes in the JSON document.
    ///
    /// Two values and never three: a consumer asks whether the publish it just
    /// ran carried the atomicity guarantee, and both reasons for not carrying it
    /// answer that question the same way. The reason is prose on standard error,
    /// where a person reads it.
    pub fn wire(&self) -> &'static str {
        match self {
            Delivery::Renamed => "renamed",
            Delivery::Direct(_) => "direct",
        }
    }

    /// What a publisher is told, where the artifact did not arrive by a rename.
    ///
    /// `None` for the atomic path, because a line printed by every publish is a
    /// line nobody reads on the publish that loses something.
    pub fn shortfall(&self) -> Option<String> {
        let reason = match self {
            Delivery::Renamed => return None,
            Delivery::Direct(Direct::NoSibling) => "the output path has no final component, so \
                 the artifact could not be assembled at a sibling path beside it"
                .to_string(),
            Delivery::Direct(Direct::Mount { error }) => format!(
                "the output path is a mount point or lies on another filesystem, so the \
                 assembled artifact could not be moved into it: {error}"
            ),
        };
        Some(reason)
    }
}

/// The subdirectory of the staging directory that the artifact is assembled in.
///
/// The marker is what makes the removal decidable, and the marker must not reach
/// the artifact, so the two live at different depths rather than side by side.
/// The rename then moves `<out>~staging/assembly` onto `--out` and leaves the
/// marker behind at `<out>~staging`, which is the path that is then removed —
/// still carrying its marker at the instant it is removed, which is the whole
/// point of writing one. A marker that had to be deleted before the rename would
/// leave a window in which a killed run's own staging directory is
/// indistinguishable from a stranger's, and the window is what this change
/// exists to close.
///
/// `<out>~staging/assembly` is a directory inside a directory beside `--out`, so
/// the rename is still on one filesystem.
const ASSEMBLY: &str = "assembly";

/// The file a publish writes into the staging directory to say the directory is
/// its own.
///
/// It is never empty, for the reason `.headwater/ids` files are never empty:
/// content is what a person reading the path gets, and it costs one string.
const MARKER: &str = ".headwater-publish-staging";

const MARKER_TEXT: &str = "\
This directory is where `headwater taxonomy publish` assembles an artifact before
it moves that artifact onto the output path beside it. A publish creates this
directory, and a publish removes it. A publish that was killed leaves it here,
and the next publish into the same output path removes it and starts again.
Nothing else reads it, and it is safe to delete.
";

/// Make the staging directory and write the marker that says it is this verb's.
///
/// The marker is the **first** thing written, before any artifact byte, so a
/// publish killed at any point after the directory exists leaves a directory
/// that says whose it is. That is what keeps
/// [#485](https://github.com/headwater-ai/headwater/issues/485)'s recovery
/// automatic under a delete that refuses to guess.
fn claim_staging(staging: &Path) -> Result<(), String> {
    std::fs::create_dir_all(staging)
        .map_err(|error| format!("a publish cannot make its staging directory: {error}"))?;
    std::fs::write(staging.join(MARKER), MARKER_TEXT)
        .map_err(|error| format!("a publish cannot claim its staging directory: {error}"))
}

/// Whether a failed rename means the output path is a mount rather than a path
/// this verb may swap.
///
/// `EBUSY` is a rename onto a mount point and `EXDEV` is a rename across a
/// filesystem boundary. Both were measured on a directory renamed onto an empty
/// `tmpfs` mount; neither is a defect of the artifact, and both are answered by
/// writing the artifact where it was asked for.
fn carries_a_mount(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::ResourceBusy | std::io::ErrorKind::CrossesDevices
    )
}

/// Remove a staging directory, whatever state it is in and whether or not it is
/// there.
///
/// # The removal is decidable, and it refuses rather than guesses
///
/// An unconditional `remove_dir_all` here would be the same undecidable delete
/// that [#485](https://github.com/headwater-ai/headwater/issues/485)'s flag was
/// refused for, moved one directory over and with the flag that made it
/// deliberate taken away. `--out` is a path a publisher named, and `<out>~staging`
/// is a path this verb derived from it — but the verb derives it from **every**
/// path anybody ever passes to `--out`, on a machine where it owns none of them.
/// `vendor`'s clear of `packages/~staging` is not the same act: that is one fixed
/// path inside a directory this tool owns.
///
/// So a publish writes [`MARKER`] into the directory before it writes a byte of
/// artifact, and this removes only a directory carrying it. That turns *a path I
/// derived* into *a directory this tool created*, which is the distinction the
/// undecidable delete did not have. A killed run always carries the marker,
/// because the marker is the first thing written, so recovery after a kill is
/// untouched.
///
/// # What each state does
///
/// Nothing there is `Ok`. A directory carrying the marker is removed. A
/// directory without it, and anything at that path that is not a directory, is
/// refused with a message naming the path — which is also what answers a
/// **file** at `<out>~staging`, where a bare `create_dir_all` reported `cannot
/// create it: File exists` against `--out`, a path that exists and is not the
/// one that stopped the run.
///
/// `symlink_metadata` is what asks, so a link at this path is refused rather
/// than followed into somebody's tree.
fn clear_staging(staging: &Path) -> Result<(), String> {
    let held = match std::fs::symlink_metadata(staging) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!(
                "a publish assembles the artifact here before it moves it into place, and this \
                 path cannot be read: {error}. Move it aside, or publish into a different output path"
            ));
        }
        Ok(held) => held,
    };
    if held.is_dir() && staging.join(MARKER).is_file() {
        return std::fs::remove_dir_all(staging)
            .map_err(|error| format!("a publish cannot clear its own staging directory: {error}"));
    }
    Err(format!(
        "a publish assembles the artifact here before it moves it into place, and something is \
         already at this path that no publish wrote. A publish removes only a directory holding \
         its own `{MARKER}` file, so this one stays. Move it aside, or publish into a different \
         output path"
    ))
}

/// Write the staged set at `at`, then the release record that describes it.
///
/// `named` is what a refusal calls the artifact, and it is `--out` whichever
/// path this is writing: a publisher who typed `--out dist` is owed a message
/// about `dist`, not about a staging path they never named. The record is
/// computed at `at` because [`release::compute`] hashes the files it finds
/// there against paths relative to it, and the staging directory holds the same
/// bytes at the same relative paths as the artifact it becomes.
fn write_artifact(
    named: &str,
    at: &Path,
    staged: &[Staged],
    manifest: &Mapping,
) -> Result<Release, Vec<ResolveError>> {
    put(at, staged).map_err(|why| refusal(named, &why))?;
    let record =
        release::compute(at, manifest).map_err(|error| release::as_error(named, &error))?;
    std::fs::write(at.join(release::RECORD), release::render(&record))
        .map_err(|error| refusal(release::RECORD, &format!("cannot write it: {error}")))?;
    Ok(record)
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

/// Every migration payload the manifest declares, read and held to the
/// taxonomies this publish ships.
///
/// It resolves twice, because a package with bundles ships one taxonomy for each
/// selection a consumer can make and the two halves of a step read the two ends
/// of that set. The base alone is the least any consumer resolves, because a
/// consumer may select no bundle. [`shipped`] is the most: the base with every
/// bundle, which
/// [spec 2](../../../../docs/spec/02-taxonomy-model.md#customization-by-composition)
/// makes a taxonomy that resolves because a bundle is add-only and any subset of
/// them commutes. `crate::migration::Scope` carries the two and
/// `crate::migration::holds` argues which half reads which. The adopter overlay
/// is in neither, because it is not the publisher's at all.
///
/// The maximal half is resolved once for the whole publish and handed in, which
/// is [#387](https://github.com/headwater-ai/headwater/issues/387). It used to
/// be built here and only where a payload exists, so a package that had
/// published no major version proved nothing about a guarantee spec 7 states for
/// every release. The base half stays here, because it is a second source list
/// rather than a second reading of the same one, and only a payload asks the
/// question it answers.
///
/// The maximal selection reaches no validate layer, so the only failures it can
/// take are `NotConfluent` and `AddCollides`. Both are defects of the artifact
/// under spec 2, and [`maximal_from`]'s prefix says which question reached them.
///
/// The source is handed in rather than read here. [`publish`] holds the two
/// version declarations to each other and needs the same file to do it, and two
/// reads of one path is the shape of the defect that check exists for.
fn migrations(
    root: &Path,
    directory: &Path,
    manifest: &Mapping,
    source: Source,
    widest: &crate::Resolution,
) -> Result<(), Vec<ResolveError>> {
    let name = manifest_name(root, directory);
    let payloads = crate::migration::at(directory, manifest)
        .map_err(|errors| crate::migration::as_errors(&name, &errors))?;
    if payloads.is_empty() {
        return Ok(());
    }

    let base = crate::resolve(std::slice::from_ref(&source))?;
    let scope = crate::migration::Scope {
        base: &base.taxonomy,
        shipped: &widest.taxonomy,
    };
    let version = text(manifest, "version").unwrap_or_default();

    let refusals: Vec<crate::migration::PayloadError> = payloads
        .iter()
        .flat_map(|payload| crate::migration::holds(payload, &scope, &version))
        .collect();
    match refusals.is_empty() {
        true => Ok(()),
        false => Err(crate::migration::as_errors(&name, &refusals)),
    }
}

/// Where a published package keeps the bundles it ships.
pub const BUNDLES: &str = "bundles";

/// Where a bundle keeps the corpora its publisher measures the bundle against.
///
/// It is the one directory a publish reads and does not carry. The corpora are
/// prose the publisher controls and no consumer verb opens one, so a consumer
/// that vendors this repository's own package took 71 of 102 members and 40 per
/// cent of the bytes in files nothing reads. [`a_bundles_own_corpus`] is the
/// reader, and spec 7's Publishing section is the ruling, stated in the same
/// paragraph that rules publication takes the package directory whole.
///
/// A publisher keeps its corpora where they are. This names what leaves the
/// artifact and never what leaves the disk.
pub const FIXTURES: &str = "fixtures";

/// Where a source package keeps named assembly recipes.
///
/// An assembly is source material for a flattened package, rather than content
/// a consumer resolves at runtime. [`crate::assembly::read`] is its reader.
/// Unlike [`BUNDLES`], this directory is always inside the source package: no
/// publish path rewrites it into the artifact.
pub const ASSEMBLIES: &str = "assemblies";

/// Where a published package keeps the prose that explains its method.
///
/// [`vendor`] resolves it against the artifact it is about to install, and
/// [`doctrine_at`] is that reader. The key names a directory inside the package
/// and never a path that leaves it, so unlike [`BUNDLES`] there is nothing for
/// [`publish`] to carry inside and rewrite.
pub const DOCTRINE: &str = "doctrine";

/// Where a package keeps the document skeletons an adopter copies.
///
/// A package-level `templates/` is the shape spec 7's example manifest block
/// writes. This repository's library keeps one per bundle instead, under
/// `<bundles>/<name>/templates/`, and [`crate::template::holds`] walks both.
pub const TEMPLATES: &str = "templates";

/// The `contents.doctrine` node a manifest declares, and nothing where it
/// declares none.
///
/// One lookup, two readers: [`doctrine_at`] holds the value to an artifact and
/// reports every shape it refuses, and [`doctrine`] answers the shallow
/// question the CLI asks after that verb returned. Where the key sits in a
/// manifest is stated here alone.
fn declared_doctrine(
    manifest: &Mapping,
) -> Option<&headwater_yaml::Spanned<headwater_yaml::Value>> {
    manifest
        .get("contents")
        .and_then(|node| node.value.as_map())
        .and_then(|contents| contents.get(DOCTRINE))
}

/// The directory a manifest declares its prose in, relative to the package.
///
/// Shallow on purpose. It answers what the key says, and [`doctrine_at`] is
/// what holds that value to an artifact. `headwater taxonomy vendor` reads it
/// off the manifest of the package it has just installed, so every shape this
/// answers `None` for — an absent key, a list, an empty value — is a shape
/// [`vendor`] refused before it installed anything.
///
/// This is what keeps [`Release`] a record of bytes. A doctrine path is not one
/// of the record's fields, and widening the record so that one line of a report
/// could be printed would make it one.
pub fn doctrine(manifest: &Mapping) -> Option<PathBuf> {
    let text = declared_doctrine(manifest)?.value.as_scalar()?.text.clone();
    match text.is_empty() {
        true => None,
        false => Some(PathBuf::from(text)),
    }
}

/// The prose a fetched artifact declares, held to that artifact before
/// [`vendor`] installs any of it.
///
/// **It reads, and it writes nothing.** It runs in `vendor`'s reading phase,
/// above the first [`clear`], so an artifact whose manifest names prose it does
/// not carry is refused before `packages/~staging/<flattened>` exists and
/// before the installed package is renamed aside. The all-or-nothing install
/// that [#312](https://github.com/headwater-ai/headwater/issues/312) and
/// [#357](https://github.com/headwater-ai/headwater/issues/357) built is
/// therefore untouched by where this sits rather than by a second undo path.
/// The natural placement — a post-condition over the installed directory, after
/// the swap — is past the point of no return, and it would turn a publisher's
/// manifest mistake into a half-installed adopter tree.
///
/// **The common case is `Ok(None)`, and it costs one lookup.** Every package
/// this repository publishes and every other fixture in the suite declares no
/// `contents.doctrine`, and none of them may start being refused for it.
///
/// **[`reachable`] holds the same path at publish, and that is not this read.**
/// `publish` reads a package directory that a publisher wrote. `vendor` reads
/// an artifact that arrived by a route no crate of this engine can see, and
/// nothing says the two ran on one machine or on one version of this engine.
/// The digest proves the manifest and the prose are the bytes the pin was
/// written for, and it proves nothing about whether any verb ever read either.
/// So the side that is about to name a path to an adopter is the side that
/// opens it.
///
/// The result is relative to the artifact, because what a reader wants is
/// `packages/<flattened>/<doctrine>` and the flattened name belongs to the
/// caller.
fn doctrine_at(
    fetched: &Path,
    manifest: &Mapping,
    name: &str,
) -> Result<Option<PathBuf>, Vec<ResolveError>> {
    let Some(declared) = declared_doctrine(manifest) else {
        return Ok(None);
    };
    let Some(scalar) = declared.value.as_scalar() else {
        return Err(refusal(
            name,
            &format!(
                "`contents.{DOCTRINE}` is not a path. The key names one directory inside the \
                 package, and a consumer is told where that directory lands"
            ),
        ));
    };
    let declared = scalar.text.as_str();
    if declared.is_empty() {
        return Err(refusal(
            name,
            &format!(
                "`contents.{DOCTRINE}` is empty, which names the package directory itself. Write \
                 the directory the prose sits in, or take the key out"
            ),
        ));
    }

    let at = fetched.join(declared);
    if !at_or_inside(&settled(fetched), &settled(&at)) {
        return Err(refusal(
            name,
            &format!(
                "`contents.{DOCTRINE}` names {declared}, which resolves outside this artifact. \
                 Only `contents.{BUNDLES}` may name a path outside the package it is declared in, \
                 because publishing carries what that one points at inside the artifact and \
                 rewrites the scalar. A doctrine path that leaves the artifact names prose no \
                 consumer received"
            ),
        ));
    }
    if !at.is_dir() {
        let found = match at.exists() {
            true => "it is a file",
            false => "it is not there",
        };
        return Err(refusal(
            name,
            &format!(
                "`contents.{DOCTRINE}` names {declared}, and {found}. The key names the directory \
                 of prose that this artifact ships, and vendoring it would tell the adopter to \
                 read a directory that never arrived. Nothing was vendored"
            ),
        ));
    }
    Ok(Some(PathBuf::from(declared)))
}

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
/// name that is not a name, by [`names_a_package`], so the target below is
/// always one segment under `packages/` rather than a path that reaches out of
/// it.
///
/// **The substitution is still not injective, and the segment is the package's
/// own because this verb refuses a directory that a different package holds.**
/// `acme/my-taxonomy` and `acme-my/taxonomy` are two names inside the grammar
/// that flatten to one directory, `packages/acme-my-taxonomy`. Vendoring the
/// second over the first took the replace arm, because the first left a release
/// record there, so it deleted a package the adopter held and exited 0 — two
/// honest publishers and no adversary. Measured, with `find_version` for the
/// first name returning `None` afterwards. That is
/// [#320](https://github.com/headwater-ai/headwater/issues/320), and
/// [`holds_the_same_package`] closes it: the replace arm reads the `package:` of
/// the directory that is there and removes nothing unless it is the name the
/// artifact declares.
///
/// **The cost is that two colliding packages cannot both sit under their derived
/// names, and the refusal names a rename that works once.** [`find`] matches the
/// `package:` of each manifest under `packages/` and never the name of the
/// directory that carries it, so a package moved out of the way keeps resolving
/// from wherever it lands, and every downstream verb reads it there. Measured,
/// through `taxonomy resolve` and `headwater check`. It is the same move the
/// `pin.current` remediation of `packages/headwater-standard/conformance.yml`
/// already asks an adopter to make, and the refusal says it because nothing else
/// the adopter reads does.
///
/// **The first package stays stuck once moved, and the second follow of the
/// remedy is now refused rather than silently duplicated.** The adopter moves
/// the first package aside and vendors the second into the cleared path. From
/// then on the first package cannot be vendored again: every later artifact of
/// it derives the directory the second one now holds, so this verb refuses
/// every upgrade of it, and nothing here lifts that: `vendor` only ever writes
/// to the name-derived directory, never to one an adopter chose by hand. What
/// changed is the second half. Following the message a second time used to exit
/// 0 and leave two directories declaring one name, where [`find`] answered from
/// whichever sorted first while `vendor` reported installing the other. This
/// verb now runs [`find`] for the declared name before it writes anything and
/// refuses that second move, naming the directory the package already resolves
/// from, so the adopter is told at the vendor that would have created the
/// duplicate rather than at their next `resolve`.
/// [#354](https://github.com/headwater-ai/headwater/issues/354) is that
/// hardening.
///
/// **A remapping was the other repair and the grammar closes it, not this
/// verb.** [`names_a_package`] admits any number of segments, so `a/b` and
/// `a/b/c` are both names and a nested `packages/a/b/c` would sit inside
/// `packages/a/b`: upgrading `a/b` removes the installed `a/b/c` with it, which
/// is #320's defect in a new shape. A flat percent-encoded name is injective and
/// unreadable. Both move every directory an adopter already vendored, and this
/// moves none. A later change that bounds a name to two segments would make
/// `packages/<org>/<name>` injective by construction and would reopen the
/// question.
///
/// **The comparison is over two declared names and never over two derived
/// paths**, so a file system that folds case refuses `ACME/Fixture` over an
/// installed `acme/fixture` by the same route rather than colliding with it.
/// That is stated from the mechanism and it is not measured, because this engine
/// is tested on Linux.
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
///
/// **Both of the arms that survived now compose from `under` rather than from
/// [`display`].** `under` is `packages/` and the flattened declared name, and
/// [`identity`] has already held that name to the grammar, so there is no state
/// in which it renders a path the sentence is not about. `display(root, &target)`
/// was safe there too, but safe by an invariant one function away rather than by
/// construction. The two render the same string for every name the grammar
/// admits, so no message moved and no fixture changed.
///
/// # `packages/<name>` is complete or it is absent, and it is never partial
///
/// That holds under a copy that returns an error and under a process killed at
/// any point in the run. The old tree is destroyed only after the new one
/// stands in its place. What a failure or a kill can leave is a tree under a
/// sibling name that no package name can spell, and never a partial tree under
/// the package's own name. This is what
/// [#312](https://github.com/headwater-ai/headwater/issues/312) asked for.
///
/// The sequence is: clear this verb's own scratch, copy the artifact into
/// `packages/~staging/<name>`, rename the installed package to
/// `packages/<name>~aside`, rename the staged tree onto `packages/<name>`, and
/// remove the aside tree last. Every refusal after the reading phase leaves one
/// state, which is why each of them can say what it says: the package that was
/// installed is untouched, or nothing is installed.
///
/// **The one thing it does not buy is a single-instant swap.** `rename(2)`
/// refuses a destination directory that is not empty — `ENOTEMPTY`, measured on
/// this file system — and `std` offers no atomic exchange, so there is a window
/// one `rename(2)` wide in which `packages/<name>` does not exist. A kill inside
/// that window leaves the old tree complete under the aside name, where [`find`]
/// still reaches it.
///
/// **[`publish`] takes the same shape, and [`deliver`] is where.** It used to
/// hold the weaker guarantee: it read everything before it wrote anything and
/// unwound `--out` on a failure, which answers a call that returns an error and
/// answered a kill not at all, so a kill inside [`put`] left a partial artifact
/// at `--out` for the publisher to remove by hand. It now assembles the artifact
/// at `<out>~staging` and renames that onto `--out`, which [`found::observe`]
/// has already established is empty or absent. A publish killed part-way leaves
/// `--out` as it found it, and the retry needs nothing removed.
/// [#485](https://github.com/headwater-ai/headwater/issues/485) is where that
/// was measured, and [`deliver`] carries the one configuration it does not
/// cover.
///
/// **Copying the [`found`] module was the other repair, and it is mechanically
/// unavailable rather than merely weaker.** `found` carries two states, absent
/// and empty, and both of them undo by deleting. It can afford that only
/// because `publish`'s `--out` precondition refuses a directory that holds
/// anything, so the state it returns to is always no bytes at all. This verb's
/// target is, in the one case that matters, a full package directory. Measured:
/// a `remove_dir_all` that failed part-way had already taken `package.yml` and
/// `release.yml` and left 7 of 10 entries, identically across three rounds. At
/// the moment of the failure the bytes an unwind would restore from are gone,
/// including the release record.
///
/// **A copy that fails part-way stopped being a case of its own.** The staging
/// copy never touches `packages/<name>`, so every failure of the copy — at the
/// first file or at the last — leaves the target exactly as it was. A fresh
/// install whose copy fails now leaves nothing under `packages/`, where it used
/// to leave the part it had written.
///
/// **Why the suffix is `~` and not a dot.** [`find`] sorts the entries of
/// `packages/` and returns the first whose manifest declares the name, with no
/// filter on the name of the entry itself, and it is the only listing of
/// `packages/` in this engine that reads what it finds as a package.
/// `markdown_under`, in the CLI, walks every directory under the root while
/// `init` counts markdown, and `packages/` is one of them; it interprets
/// nothing it finds there. A dot-prefixed staging directory
/// therefore sorts *before* the real package and wins the lookup — measured,
/// with `taxonomy resolve` reporting the staged version. `~` is 0x7E, and the
/// highest byte [`names_a_package`] admits is `z` at 0x7A, so no package the
/// grammar accepts can derive `packages/<name>~aside` or the shared
/// `packages/~staging`, and both sort after every flattened name.
/// `<name>~aside` sorts before `~staging` too, because the flattened name's own
/// first byte is always less than `~`, so a run killed in the one-rename window
/// leaves [`find`] returning the old complete tree rather than the new one.
/// **This leaned on that sort order alone**, until
/// [#369](https://github.com/headwater-ai/headwater/issues/369)'s own fix made
/// the choice explicit: `find` no longer decides by where a name happens to
/// sort. It separates a directory's `~aside`/`~staging` residue from an
/// ordinary match by whether the directory carries a release record —
/// [`release::at`] — the same test `vendor` itself already applies, a few
/// lines below, to tell a vendored directory from one a person maintains by
/// hand.
///
/// **The staged tree is not read back and held to the digest before the swap.**
/// [`copy_tree`] returning `Ok` means every file was read and written and no
/// call errored, which is the readable a complete replacement is asked for. A
/// second digest pass was weighed and refused: `std::fs::copy` does not
/// `fsync`, so the re-read comes back out of the page cache, and a file system
/// that lied about the write lies about the read. It would cost a second hash
/// of every artifact and answer a question the copy already answers with an
/// error.
///
/// **An artifact directory that holds the target inside it is refused before a
/// byte is staged.** [`copy_tree`] calls `create_dir_all(to)` before
/// `read_dir(from)`, so a destination inside the source makes it recurse into
/// its own output. `vendor <root>` and `vendor <root>/packages` are the two
/// shapes. Vendoring a package over itself, where the artifact path and the
/// target are one directory, is **not** this case and is not refused: the
/// staged copy is taken while the artifact still stands and the swap puts it
/// back, so it exits 0 with the package installed. Before this it exited 0 with
/// an empty directory.
///
/// **Every wrong version of this sequence compiles.** Swapping the two renames,
/// dropping the clear of one sibling path, and giving the two siblings suffixes
/// the grammar admits were each built and run: the type system refused none of
/// them, so nothing here is unrepresentable and the fixtures are the whole of
/// what holds the order. The clear of the staging path is the one step no other
/// case in the suite pins, and
/// `a_file_an_earlier_run_left_in_the_staging_directory_is_not_installed` is
/// there because 48 of the other 49 pass with it deleted.
///
/// **Two `vendor` runs of one package contend for one staging path.** They
/// contend for the target today, so nothing here is made worse, and there is no
/// lock anywhere in this engine to hang a repair on.
///
/// **A kill inside the one-rename window leaves the package that was installed
/// under `packages/<name>~aside`, and nothing tells the adopter.** The next
/// `vendor` of that package clears it, and [`find`] answers from it in the
/// meantime, so the adopter still resolves.
///
/// **A kill inside the staging copy leaves a partial tree under
/// `packages/~staging/<name>`, and [`find`] never answers from it.** Before
/// [#357](https://github.com/headwater-ai/headwater/issues/357), the staging
/// copy sat flat at `packages/<name>~staged`: where a package was installed,
/// `find` answered from the installed one, which sorted first, and never
/// reached the staging directory, but where none was — a first install —
/// `find` had nothing else to answer from and read the partial tree. Measured,
/// over an artifact of 4003 files laid out so the manifest and the taxonomy
/// source copy before the rest: a run killed at 103 of 4003 files resolved
/// exactly as the complete package does. Staging one level down closes that:
/// `find` reads one level of `packages/` and skips a directory with no
/// manifest beside it, so `packages/~staging` is never a candidate, on a first
/// install or an upgrade. What this does not touch is the state #312 and #356
/// already hold: `find` still reads `packages/<name>` itself whether or not
/// anything else is installed, and this staging path is never that
/// directory.
///
/// **Neither residue is inert to the rest of the engine, and whether it is
/// depends on the adopter's configuration rather than on the residue.**
/// [`headwater_census::walk`] walks the corpus root and every directory under
/// it. This repository declares `corpus.root: docs`, so `packages/` is outside
/// the walk and a residue changes no census number here. An adopter whose root
/// includes `packages/` counts every file of one: measured by an independent
/// verification pass on such an adopter, a `~staged` residue took the census
/// from 55 files and 38 untyped to 107 and 76. Say which of the two
/// configurations a claim about a residue is about.
pub fn vendor(root: &Path, fetched: &Path, pinned: &str) -> Result<Release, Vec<ResolveError>> {
    let name = display(root, fetched);
    let record =
        release::verify(fetched, pinned).map_err(|error| release::as_error(&name, &error))?;

    let declared = identity(root, fetched, &record)?;
    let flattened = declared.replace('/', "-");
    let packages = root.join(PACKAGES);
    let target = packages.join(&flattened);
    let staging_root = packages.join(STAGING);
    let staged = staging_path(&packages, &flattened);
    let aside = packages.join(format!("{flattened}{ASIDE}"));
    let under = format!("{PACKAGES}/{flattened}");
    let staged_under = display(root, &staged);
    let aside_under = format!("{under}{ASIDE}");

    // The reading phase. Nothing below this writes until the staging copy, and
    // the staging copy does not touch `target`.
    let holds = settled(fetched);
    for (path, shown) in [
        (&target, &under),
        (&staged, &staged_under),
        (&aside, &aside_under),
    ] {
        if inside(&holds, &settled(path)) {
            return Err(refusal(
                &name,
                &format!(
                    "this artifact directory holds `{shown}` inside it, so installing it would \
                     copy the artifact into a directory inside itself and never finish. Vendor \
                     from a copy of the artifact that sits outside `{PACKAGES}/`"
                ),
            ));
        }
    }
    for (path, shown) in [(&staged, &staged_under), (&aside, &aside_under)] {
        if at_or_inside(&settled(path), &holds) {
            return Err(refusal(
                &name,
                &format!(
                    "`{shown}` is a directory this verb writes and removes while it installs a \
                     package, so it cannot also be the artifact to install: this run would have \
                     deleted it before reading it. A run that was stopped part-way leaves the \
                     package under that name. Move it outside `{PACKAGES}/` and vendor it from \
                     there"
                ),
            ));
        }
    }

    let installed = target.exists();
    if installed {
        match release::at(&target) {
            // The guard reads and writes nothing, and it sits here rather than
            // in the write phase below. #320 put it in front of a removal that
            // has since moved; the question it answers — is the directory that
            // is there the package this artifact declares — is one for the
            // phase that reads.
            Ok(_) => holds_the_same_package(&target, &under, &declared)?,
            Err(ReleaseError::Absent(_)) => {
                return Err(refusal(
                    &under,
                    "a directory is there and it carries no release record, so it is a package \
                     somebody maintains rather than one that was vendored. Move it before \
                     vendoring over it",
                ))
            }
            Err(error) => return Err(release::as_error(&under, &error)),
        }
    }

    // The package this artifact declares may already resolve from a directory
    // that is neither `target` nor one of this run's own siblings -- the state
    // #320's own remedy produces on the second collision. `target`, `staged`
    // and `aside` are excluded on purpose: a kill mid-swap can leave the
    // installed package's own manifest resolving from `aside` (or, mid-stage,
    // a stale `staged`), and that residue is what the *next* `vendor` of the
    // same package clears, not a directory that belongs to someone else. Only
    // a standing directory outside those three is the adopter's own, moved
    // package, and this is a read, so it runs here rather than in the write
    // phase below.
    if let Ok((found_at, _)) = find(root, &declared) {
        if found_at != target && found_at != staged && found_at != aside {
            let found_under = display(root, &found_at);
            return Err(refusal(
                &under,
                &format!(
                    "the artifact declares `{declared}`, and that name already resolves from \
                     `{found_under}` rather than from `{under}`. Vendoring here would leave two \
                     directories under `{PACKAGES}/` both declaring `{declared}`, and the next \
                     lookup would have to choose between them. This package cannot be vendored \
                     into its derived directory while it resolves from `{found_under}`; move it \
                     back to `{under}` or vendor elsewhere by hand"
                ),
            ));
        }
    }

    // The prose the artifact declares, held to the artifact. [`doctrine_at`]
    // writes nothing and this is the last read, so a manifest that names
    // doctrine the artifact does not carry returns above every write below:
    // nothing is staged, nothing is renamed aside, and an installed package
    // stands as it stood. The value is dropped here because `Release` is a
    // record of bytes and this is not one of them -- the CLI reads the key back
    // off the installed manifest to name the path it printed.
    let _ = doctrine_at(fetched, &manifest_at(fetched)?, &name)?;

    // Every refusal below leaves one state, and this is that state said once.
    let stands = match installed {
        true => format!("`{under}` is still the package that was installed"),
        false => format!("nothing is installed at `{under}`"),
    };

    // The write phase. Whatever stands under the two sibling names goes, and
    // nothing an adopter created can stand there: the grammar refuses `~`, so
    // neither name is one a package can be vendored under.
    for (path, shown) in [(&staged, &staged_under), (&aside, &aside_under)] {
        clear(path).map_err(|error| {
            refusal(
                shown,
                &format!(
                    "this verb stages a package here and cannot clear what an earlier run left: \
                     {error}. Nothing was vendored, and {stands}"
                ),
            )
        })?;
    }

    copy_tree(fetched, &staged).map_err(|why| {
        let _ = std::fs::remove_dir_all(&staged);
        tidy(&staging_root);
        refusal(&name, &format!("{why}. Nothing was vendored, and {stands}"))
    })?;

    if installed {
        std::fs::rename(&target, &aside).map_err(|error| {
            let _ = std::fs::remove_dir_all(&staged);
            tidy(&staging_root);
            refusal(
                &under,
                &format!(
                    "cannot move it aside to put `{name}` in its place: {error}. Nothing was \
                     vendored, and {stands}"
                ),
            )
        })?;
    }

    if let Err(error) = std::fs::rename(&staged, &target) {
        let put_back = !installed || std::fs::rename(&aside, &target).is_ok();
        let _ = std::fs::remove_dir_all(&staged);
        tidy(&staging_root);
        let state = match put_back {
            true => format!("Nothing was vendored, and {stands}"),
            false => format!(
                "Nothing was vendored, and the package that was installed is complete under \
                 `{aside_under}`. Move it back to `{under}`"
            ),
        };
        return Err(refusal(
            &under,
            &format!("cannot put `{name}` in place: {error}. {state}"),
        ));
    }

    // Last, and the error is dropped for the reason `found::unwind` drops its
    // own: this runs on the way out of a success, and a message about scratch
    // would displace the one a reader came for. The shared staging parent goes
    // the same way, whether or not this run installed anything: it is either
    // empty, because this run's own subdirectory was just renamed onto
    // `target`, or a sibling package's `vendor` is staging into it right now,
    // and `tidy` only ever removes an empty directory.
    if installed {
        let _ = std::fs::remove_dir_all(&aside);
    }
    tidy(&staging_root);
    Ok(record)
}

/// The name of the directory [`vendor`] copies a new package into, one level
/// below `packages/`, before the swap onto the package's own name.
///
/// **It is public because it is an assertion rather than a detail.** The byte
/// `~` is one [`names_a_package`] refuses and it sorts after every byte that
/// grammar admits, so no package can derive this name, and [`find`] reads one
/// level of `packages/` and skips a directory with no manifest beside it — so a
/// directory named `STAGING` never answers a lookup in place of a package being
/// staged inside it, on a first install or an upgrade. [`staging_path`] is the
/// one construction that joins this to `packages/` and to a package's own
/// flattened name; [`vendor`] and every test that plants a staging residue by
/// hand share it, rather than hand-joining the same three pieces independently.
pub const STAGING: &str = "~staging";

/// The suffix of the directory [`vendor`] moves the installed package to while
/// the new one takes its place.
///
/// `<flattened>~aside` sorts before the shared [`STAGING`] directory, because
/// the flattened name's own first byte is one the grammar admits and every such
/// byte is less than `~`. So a run killed in the one-rename window leaves
/// [`find`] returning the complete tree that was installed rather than the one
/// being installed.
pub const ASIDE: &str = "~aside";

/// Where [`vendor`] copies a package while it is not yet complete.
///
/// One level below `packages/`, under [`STAGING`], so [`find`] — which reads
/// one level of `packages/` and skips a directory with no manifest beside it —
/// never descends into it. A copy killed mid-stage sits under a name `find`
/// cannot reach, whether or not anything else answers the package's name; that
/// is [#357](https://github.com/headwater-ai/headwater/issues/357).
pub fn staging_path(packages: &Path, flattened: &str) -> PathBuf {
    packages.join(STAGING).join(flattened)
}

/// Remove a directory an earlier run of this verb left, where absent is not a
/// failure.
fn clear(at: &Path) -> std::io::Result<()> {
    match std::fs::remove_dir_all(at) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

/// Remove the shared staging parent [`vendor`] copies into, on every exit path
/// that leaves it standing.
///
/// **Never [`std::fs::remove_dir_all`].** A sibling package's own `vendor` may
/// be staging into its own subdirectory of this same parent at this instant —
/// [`STAGING`]'s doc comment names the concurrency this implies — and the
/// non-recursive [`std::fs::remove_dir`] is what makes that safe: it succeeds
/// only when the directory holds nothing, so it can never take another
/// package's in-flight staging subdirectory with it. Not empty and not there
/// are both ignored on the same terms every other cleanup in this function
/// ignores its own: this runs on an exit path that already has its own outcome
/// to report, and one more scratch directory standing is not it.
fn tidy(staging_root: &Path) {
    let _ = std::fs::remove_dir(staging_root);
}

/// A path with the part of it that is on disk resolved, and the part that is
/// not there yet appended as it was written.
///
/// [`inside`] compares two paths and a lexical comparison answers the wrong
/// question: `packages/` may be reached through a symlink, and the artifact
/// path a caller hands in may be relative where the root is absolute. This
/// climbs to the deepest ancestor that exists, canonicalizes that, and puts the
/// rest back. It is not [`display`], which flattens a `..` for a person to read
/// and never asks the file system anything.
fn settled(path: &Path) -> PathBuf {
    let mut tail: Vec<std::ffi::OsString> = Vec::new();
    let mut at = path.to_path_buf();
    loop {
        if let Ok(real) = at.canonicalize() {
            let mut settled = real;
            for part in tail.iter().rev() {
                settled.push(part);
            }
            return settled;
        }
        let (Some(name), Some(parent)) = (at.file_name(), at.parent()) else {
            return path.to_path_buf();
        };
        if parent.as_os_str().is_empty() {
            return path.to_path_buf();
        }
        tail.push(name.to_owned());
        at = parent.to_path_buf();
    }
}

/// Whether `path` sits strictly under `ancestor`.
///
/// Equality is deliberately not inside. Vendoring a package over itself hands
/// this the same directory twice, and that call installs the package rather
/// than being refused.
fn inside(ancestor: &Path, path: &Path) -> bool {
    path != ancestor && path.starts_with(ancestor)
}

/// Whether `path` is `ancestor` or sits under it.
///
/// **The two sibling names need the equality that [`inside`] excludes, and the
/// exclusion was wrong for them.** `packages/<name>` may be handed to
/// [`vendor`] as the artifact, because a package vendors over itself. The two
/// directories `vendor` stages through may not, because the first thing the
/// write phase does is remove them: handed one of them, the verb deletes the
/// artifact it verified a moment earlier. The staging path is worse than the
/// aside path, because `copy_tree` recreates what it deleted as an empty
/// directory and copies zero files out of it without an error.
fn at_or_inside(ancestor: &Path, path: &Path) -> bool {
    path.starts_with(ancestor)
}

/// Whether the vendored directory standing at the target is the package the
/// artifact declares, read from the `package:` each of them states.
///
/// [`vendor`] derives the target from a name and the derivation is not
/// injective, so a directory that carries a release record is not evidence that
/// the record covers this package. This asks the directory what it is, and it
/// is the whole of what keeps the removal in [`vendor`] from taking a package it
/// is not replacing.
///
/// **It runs before the first write and it writes nothing**, so it is one more
/// read in the phase that already reads. That is where
/// [#312](https://github.com/headwater-ai/headwater/issues/312) needs it: that
/// change restructures the write half of `vendor` so no removal happens before a
/// complete replacement exists, and a guard folded into the removal's error path
/// would have to move with it.
///
/// **Every message here is composed from a declared value**, never from
/// [`display`] of the target. [`identity`] sets the precedent and [`vendor`]'s
/// doc comment records what it cost to learn it.
///
/// **A resident name that is not a name is refused rather than compared.** The
/// manifest may be absent, unreadable, not a mapping, carry no `package:`, or
/// carry a value the grammar refuses, and all five states say the same thing: no
/// name to hold this artifact's against. A comparison would read the first four
/// as "some other package" and say so in a sentence that names nothing, and it
/// would read `package: ""` as a package called nothing at all.
fn holds_the_same_package(
    target: &Path,
    under: &str,
    declared: &str,
) -> Result<(), Vec<ResolveError>> {
    let resident = manifest_at(target)
        .ok()
        .and_then(|manifest| text(&manifest, "package"))
        .filter(|resident| names_a_package(resident));

    match resident {
        Some(resident) if resident == declared => Ok(()),
        Some(resident) => Err(refusal(
            under,
            &format!(
                "this directory holds `{resident}` and the artifact declares `{declared}`. Two \
                 package names reach one directory, because a `/` in a name becomes a `-` in \
                 the directory the name is created under, so vendoring this artifact would \
                 delete a package it is not replacing. Move `{under}` aside and vendor again: \
                 the lookup under `{PACKAGES}/` reads the `package:` of each manifest and never \
                 the name of the directory that carries it, so `{resident}` keeps resolving \
                 from wherever you move it"
            ),
        )),
        None => Err(refusal(
            under,
            &format!(
                "this directory carries a release record, and `{under}/{MANIFEST}` states no \
                 package name that can be read, so there is nothing here to hold the \
                 `{declared}` this artifact declares against. Every directory this verb writes \
                 carries a manifest that names a package, because an artifact without one is \
                 refused before a byte is written, so a directory in this state was not written \
                 by this verb. Move `{under}` aside and vendor again"
            ),
        )),
    }
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
///
/// **`to` must not lie inside `from`.** The `create_dir_all(to)` below runs
/// before the `read_dir(from)`, so a destination under the source is one of the
/// entries the listing returns and the recursion descends into what it is
/// writing. It ends at `ENAMETOOLONG` some hundred levels down, having written
/// a tree nobody asked for. This takes two paths and cannot check that itself
/// without deciding what a caller meant by them; [`vendor`] holds the check,
/// and its doc comment records the shape.
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
pub(crate) fn display(root: &Path, path: &Path) -> String {
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
    vec![refusal_at(source, message)]
}

/// One refusal, for a caller that already holds a list of them.
///
/// [`crate::template`] reports every bad template of a package rather than the
/// first, so it builds its own list and needs one error at a time. Both
/// spellings make the same value.
pub(crate) fn refusal_at(source: &str, message: &str) -> ResolveError {
    ResolveError::new(
        ResolveErrorKind::SourceRefused(message.to_string()),
        source,
        "",
        headwater_yaml::Span::default(),
    )
}

#[cfg(test)]
mod tests {
    use super::names_a_package;
    use super::{carries_a_mount, Delivery, Direct};

    /// Which rename failures say the output path is a mount, and which say the
    /// publish is broken.
    ///
    /// This holds the classifier and **not** the wiring between it and what a
    /// publisher is told. Nothing here would notice
    /// [`super::deliver`] discarding the answer, which is the defect
    /// [#664](https://github.com/headwater-ai/headwater/issues/664) is about, so
    /// this case is not a substitute for the mount-point case in
    /// `tests/killed_publish.rs`. It runs everywhere and that one does not,
    /// which is the whole of what it is for.
    #[test]
    fn two_rename_failures_fall_back_and_every_other_kind_refuses() {
        use std::io::ErrorKind;

        for kind in [ErrorKind::ResourceBusy, ErrorKind::CrossesDevices] {
            assert!(
                carries_a_mount(&std::io::Error::from(kind)),
                "{kind:?} is a mount point or another filesystem, and the artifact is written \
                 where it was asked for"
            );
        }
        for kind in [
            ErrorKind::PermissionDenied,
            ErrorKind::NotFound,
            ErrorKind::AlreadyExists,
            ErrorKind::InvalidInput,
            ErrorKind::Other,
        ] {
            assert!(
                !carries_a_mount(&std::io::Error::from(kind)),
                "{kind:?} is a broken publish and it is refused rather than written around"
            );
        }
    }

    /// Every delivery reports itself, and only the weaker ones explain.
    ///
    /// Two `direct` arms and not one: the mount point is the arm
    /// [#664](https://github.com/headwater-ai/headwater/issues/664) names, and
    /// the missing sibling is the one it does not.
    #[test]
    fn every_delivery_has_a_wire_value_and_only_a_direct_one_explains() {
        assert_eq!(Delivery::Renamed.wire(), "renamed");
        assert_eq!(Delivery::Renamed.shortfall(), None);
        for direct in [
            Direct::NoSibling,
            Direct::Mount {
                error: "Device or resource busy (os error 16)".to_string(),
            },
        ] {
            let delivery = Delivery::Direct(direct.clone());
            assert_eq!(delivery.wire(), "direct", "{direct:?}");
            let shortfall = delivery
                .shortfall()
                .unwrap_or_else(|| panic!("{direct:?} explains nothing to a publisher"));
            assert!(
                !shortfall.is_empty() && shortfall.ends_with(|last: char| last != '.'),
                "the shortfall is a clause the caller finishes, not a sentence: {shortfall}"
            );
        }
    }

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
