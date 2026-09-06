// SPDX-License-Identifier: Apache-2.0
//! A named assembly rendered as one package with no runtime bundle selection.
//!
//! This module reads an assembly's sources and returns the complete bytes a
//! publisher later writes. It deliberately has no output path: publication owns
//! staging, atomic replacement, and release records. Keeping the derivation
//! here makes the equality a publisher promises testable before any byte is
//! written.

use crate::assembly::{self, Assembly};
use crate::error::{ResolveError, ResolveErrorKind};
use crate::merge;
use crate::package::{self, ASSEMBLIES, BUNDLES};
use crate::source::Source;
use crate::Resolution;
use headwater_hash::digest;
use headwater_yaml::{Entry, Mapping, Scalar, Span, Spanned, Style, Value};
use std::path::{Component, Path, PathBuf};

/// The material a flattened package contains before publication writes it.
#[derive(Clone, Debug)]
pub struct Flattened {
    /// The generated `package.yml` tree.
    pub manifest: Mapping,
    /// The generated `taxonomy.yml` in the resolver's canonical form.
    pub taxonomy: String,
    /// Package-owned prose and templates, under assembly-namespaced paths.
    pub assets: Vec<Asset>,
}

/// One non-taxonomy file a flattened package carries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asset {
    /// The path below the generated package root, with `/` separators.
    pub path: String,
    /// The exact source bytes.
    pub bytes: Vec<u8>,
}

/// Resolve an assembly afresh and derive its flattened package without writing.
///
/// The returned package has its own identity and version, but its taxonomy has
/// the same declarations as the recipe resolution. `distribution.derived_from`
/// records the source taxonomy, each selected bundle, and optional glue by
/// digest. Assets declared as doctrine or templates move below a directory
/// named for the assembly, so two flattened packages can live side by side.
/// Every other declared member travels at the relative path the source names,
/// unchanged. [`members`] is the one enumeration the generated manifest and the
/// carried bytes both come from, and its doc comment carries the arms.
pub fn materialize(
    root: &Path,
    directory: &Path,
    source_manifest: &Mapping,
    recipe: &Assembly,
) -> Result<Flattened, Vec<ResolveError>> {
    let resolution = assembly::resolve(root, directory, source_manifest, recipe)?;
    let sources = assembly_sources(root, directory, source_manifest, recipe)?;
    let members = members(directory, source_manifest, recipe, &recipe.at)?;
    let manifest = manifest(source_manifest, recipe, &sources, &members)?;
    let taxonomy = taxonomy(&resolution, recipe);
    let assets = assets(&members, &recipe.at)?;
    let flattened = Flattened {
        manifest,
        taxonomy,
        assets,
    };
    if equivalent(&resolution, &flattened, recipe) {
        Ok(flattened)
    } else {
        Err(refusal(
            &recipe.at,
            "the generated flattened taxonomy differs from the assembly resolution after identity is removed",
        ))
    }
}

/// Whether a flattened taxonomy has the recipe resolution's declarations.
///
/// Identity is deliberately outside this comparison. A flattened package must
/// call itself `recipe.package@recipe.version`; its source package cannot. The
/// rest of the resolved taxonomy is the claim a batteries-included consumer
/// receives the same declarations as a composer.
pub fn equivalent(resolution: &Resolution, flattened: &Flattened, recipe: &Assembly) -> bool {
    let Ok(loaded) = Source::from_text(
        "flattened taxonomy",
        crate::source::Role::Taxonomy,
        &flattened.taxonomy,
    ) else {
        return false;
    };
    let Some(found) = loaded.root.value.as_map() else {
        return false;
    };
    merge::same(
        &Value::Map(normalize(&resolution.taxonomy, None)),
        &Value::Map(normalize(found, Some(recipe))),
    )
}

fn assembly_sources(
    root: &Path,
    directory: &Path,
    source_manifest: &Mapping,
    recipe: &Assembly,
) -> Result<Vec<Source>, Vec<ResolveError>> {
    let mut sources = assembly::sources(root, directory, source_manifest, recipe)?;
    if let Some(path) = &recipe.overlay {
        let name = path
            .strip_prefix(root)
            .unwrap_or(path)
            .display()
            .to_string();
        sources.push(Source::read(path, &name, crate::source::Role::Overlay)?);
    }
    Ok(sources)
}

fn taxonomy(resolution: &Resolution, recipe: &Assembly) -> String {
    crate::render::render(&rewrite_identity(
        &resolution.taxonomy,
        &recipe.package,
        &recipe.version,
    ))
}

fn manifest(
    source: &Mapping,
    recipe: &Assembly,
    sources: &[Source],
    members: &[Member],
) -> Result<Mapping, Vec<ResolveError>> {
    let mut entries: Vec<Entry> = source
        .entries()
        .iter()
        .filter(|entry| entry.key.value != "package" && entry.key.value != "version")
        .filter(|entry| entry.key.value != "contents" && entry.key.value != "distribution")
        .cloned()
        .collect();
    entries.insert(0, entry("package", scalar(&recipe.package)));
    entries.insert(1, entry("version", scalar(&recipe.version)));
    entries.push(entry("contents", Value::Map(contents(members))));
    entries.push(entry(
        "distribution",
        Value::Map(distribution(recipe, sources)?),
    ));
    Ok(Mapping::new(entries))
}

/// One `contents` key of a flattened package: what the generated manifest
/// declares for it, and where the bytes it names come from.
struct Member {
    /// The key, written the same way in both manifests.
    key: String,
    /// The path the generated manifest declares, relative to the artifact.
    declared: String,
    /// The source path whose bytes travel to `declared`, where any do. The
    /// stager writes `taxonomy.yml` out of the resolution, so that member has
    /// no source file.
    from: Option<PathBuf>,
}

/// The `contents` of a flattened package, enumerated once.
///
/// [`contents`] renders the mapping from this and [`assets`] copies the bytes
/// from it, so the manifest a consumer reads and the files it opens come from
/// one walk of one declaration. That they came from two was
/// [#581](https://github.com/headwater-ai/headwater/issues/581): `contents`
/// passed through every key it did not filter and `assets` wrote two, so a
/// flattened `headwater/standard` declared a conformance rule set it did not
/// carry and `headwater conformance` failed for whoever vendored it.
///
/// Four arms, and each one states what it rests on.
///
/// - **`taxonomy` is declared at the artifact root and has no source file.**
///   [`crate::package::stage_flattened`] writes the rendered resolution there.
/// - **`bundles`, `assemblies` and `migrations` are dropped.** All three are
///   statements about the source package's composition and its version line,
///   and a flattened package inherits neither: it takes a new identity and a
///   new version, and `taxonomy diff` selects a migration payload by the
///   version ranges it declares. A payload written for `headwater/standard`
///   3 to 4 means nothing against a starter version line. `taxonomy`,
///   `conformance`, `doctrine` and `templates` are statements about the
///   taxonomy's content, which a flattened package does inherit.
/// - **`doctrine` and `templates` are namespaced under the recipe name.** The
///   module header states the reason: two flattened packages derived from one
///   source can then hold their prose side by side.
/// - **Every other key travels at the relative path the source declares,
///   unchanged.** `taxonomy` already sits at the artifact root, so a second
///   file beside it is a shape this artifact already has rather than a new one.
///   Namespacing has no purchase on a single file, which has no directory to
///   rename, and `contents.conformance` is resolved against the installed
///   package directory by its one reader. Leaving the scalar alone also makes
///   the manifest that was a lie true with no change to what it declares.
fn members(
    directory: &Path,
    source: &Mapping,
    recipe: &Assembly,
    at: &Path,
) -> Result<Vec<Member>, Vec<ResolveError>> {
    let source_contents = source
        .get("contents")
        .and_then(|node| node.value.as_map())
        .ok_or_else(|| {
            refusal(
                at,
                "the source package manifest has no mapping at `contents`",
            )
        })?;
    let mut out = vec![Member {
        key: "taxonomy".to_string(),
        declared: "taxonomy.yml".to_string(),
        from: None,
    }];
    for found in source_contents {
        let key = found.key.value.as_str();
        if key == "taxonomy"
            || key == BUNDLES
            || key == ASSEMBLIES
            || key == crate::migration::CONTENTS_KEY
        {
            continue;
        }
        // A value that is not a scalar and an empty value are both refused by
        // `crate::package::reachable`, which runs over this same source
        // manifest before `materialize` is called. Refusing them again here
        // would be a second definition of one rule.
        let Some(scalar) = found.value.value.as_scalar() else {
            continue;
        };
        let declared = scalar.text.as_str();
        if declared.is_empty() {
            continue;
        }
        let from = contained(directory, key, declared, at)?;
        let declared = match key {
            package::DOCTRINE | package::TEMPLATES => format!("{key}/{}", recipe.name),
            _ => declared.to_string(),
        };
        out.push(Member {
            key: key.to_string(),
            declared,
            from: Some(from),
        });
    }
    Ok(out)
}

fn contents(members: &[Member]) -> Mapping {
    Mapping::new(
        members
            .iter()
            .map(|member| entry(&member.key, scalar(&member.declared)))
            .collect(),
    )
}

fn distribution(recipe: &Assembly, sources: &[Source]) -> Result<Mapping, Vec<ResolveError>> {
    let base = sources.first().expect("an assembly has a base taxonomy");
    let bundles = sources
        .iter()
        .skip(1)
        .take(recipe.from.bundles.len())
        .zip(&recipe.from.bundles)
        .map(|(source, name)| {
            Spanned::new(
                Value::Map(Mapping::new(vec![
                    entry("name", scalar(name)),
                    entry("digest", scalar(&digest(source.text.as_bytes()))),
                ])),
                Span::default(),
            )
        })
        .collect();
    let mut from = vec![
        entry("package", scalar(&recipe.from.package)),
        entry("version", scalar(&recipe.from.version)),
        entry("taxonomy_digest", scalar(&digest(base.text.as_bytes()))),
        entry(
            "selection_digest",
            scalar(&digest(recipe.from.bundles.join("\n").as_bytes())),
        ),
        entry("bundles", Value::Seq(bundles)),
        entry("recipe_digest", scalar(&digest(&read(&recipe.at)?))),
    ];
    if let Some(overlay) = &recipe.overlay {
        from.push(entry("overlay_digest", scalar(&digest(&read(overlay)?))));
    }
    Ok(Mapping::new(vec![
        entry("form", scalar("flattened")),
        entry("derived_from", Value::Map(Mapping::new(from))),
    ]))
}

/// The bytes each member carries, at the path its `declared` names.
///
/// A directory-valued member is walked and a file-valued one is read. Which
/// one a key is comes from [`crate::package::required_kind`] where that table
/// names it, and from the disk where it does not — [`contained`] settles it
/// once, so this dispatch and the manifest above it cannot disagree.
fn assets(members: &[Member], at: &Path) -> Result<Vec<Asset>, Vec<ResolveError>> {
    let mut out = Vec::new();
    for member in members {
        let Some(from) = &member.from else {
            continue;
        };
        if from.is_dir() {
            collect(from, from, &member.declared, &mut out, at)?;
            continue;
        }
        out.push(Asset {
            path: member.declared.clone(),
            bytes: std::fs::read(from).map_err(|error| {
                refusal(at, &format!("cannot read {}: {error}", from.display()))
            })?,
        });
    }
    Ok(out)
}

/// The source path a `contents` key names, held inside the source package and
/// held to the kind its reader opens.
///
/// The boundary check and the kind check are two questions and this answers
/// both, because a member that leaves the package and a member that is the
/// wrong kind are both paths a consumer cannot use. The kind comes from
/// [`crate::package::required_kind`] rather than from a hard-coded `is_dir`:
/// `contents.conformance` names a file, and requiring a directory here is what
/// kept a file-valued member out of a flattened artifact for as long as this
/// function had only two callers. A key that table does not name keeps the
/// containment check and takes whatever kind is on disk, which is the seam that
/// table's own doc comment states.
fn contained(
    directory: &Path,
    key: &str,
    declared: &str,
    at: &Path,
) -> Result<PathBuf, Vec<ResolveError>> {
    let path = Path::new(declared);
    if path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(refusal(
            at,
            &format!("`contents.{key}` path `{declared}` leaves the source package"),
        ));
    }
    let boundary = directory.canonicalize().map_err(|error| {
        refusal(
            at,
            &format!(
                "cannot read source package {}: {error}",
                directory.display()
            ),
        )
    })?;
    let candidate = directory.join(path).canonicalize().map_err(|error| {
        refusal(
            at,
            &format!("cannot read declared `contents.{key}` path `{declared}`: {error}"),
        )
    })?;
    if !candidate.starts_with(boundary) {
        return Err(refusal(
            at,
            &format!("`contents.{key}` path `{declared}` is not inside the source package"),
        ));
    }
    if let Some(needed) = package::required_kind(key) {
        let found = package::Kind::of(&candidate);
        if found != needed {
            return Err(refusal(
                at,
                &format!(
                    "`contents.{key}` path `{declared}` is a {}, and the verb that reads it reads \
                     a {}",
                    found.name(),
                    needed.name()
                ),
            ));
        }
    }
    Ok(candidate)
}

fn collect(
    root: &Path,
    at: &Path,
    prefix: &str,
    out: &mut Vec<Asset>,
    recipe: &Path,
) -> Result<(), Vec<ResolveError>> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(at)
        .map_err(|error| refusal(recipe, &format!("cannot read {}: {error}", at.display())))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    entries.sort();
    for entry_path in entries {
        let name = entry_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                refusal(
                    recipe,
                    &format!("an asset under {} has a non-UTF-8 name", at.display()),
                )
            })?;
        let settled = entry_path.canonicalize().map_err(|error| {
            refusal(
                recipe,
                &format!("cannot read {}: {error}", entry_path.display()),
            )
        })?;
        if !settled.starts_with(root) {
            return Err(refusal(
                recipe,
                &format!(
                    "an asset under {} leaves its declared directory",
                    at.display()
                ),
            ));
        }
        let path = format!("{prefix}/{name}");
        if settled.is_dir() {
            collect(root, &settled, &path, out, recipe)?;
        } else {
            out.push(Asset {
                path,
                bytes: std::fs::read(&settled).map_err(|error| {
                    refusal(
                        recipe,
                        &format!("cannot read {}: {error}", settled.display()),
                    )
                })?,
            });
        }
    }
    Ok(())
}

fn rewrite_identity(map: &Mapping, package: &str, version: &str) -> Mapping {
    Mapping::new(
        map.entries()
            .iter()
            .map(|found| match found.key.value.as_str() {
                "taxonomy" => entry_at(&found.key.value, scalar(package), found.key.span),
                "version" => entry_at(&found.key.value, scalar(version), found.key.span),
                _ => found.clone(),
            })
            .collect(),
    )
}

fn normalize(map: &Mapping, recipe: Option<&Assembly>) -> Mapping {
    match recipe {
        Some(recipe) => rewrite_identity(map, &recipe.from.package, &recipe.from.version),
        None => map.clone(),
    }
}

fn entry(key: &str, value: Value) -> Entry {
    entry_at(key, value, Span::default())
}

fn entry_at(key: &str, value: Value, span: Span) -> Entry {
    Entry {
        key: Spanned::new(key.to_string(), span),
        value: Spanned::new(value, span),
    }
}

fn scalar(text: &str) -> Value {
    Value::Scalar(Scalar {
        text: text.to_string(),
        style: Style::Plain,
    })
}

fn read(path: &Path) -> Result<Vec<u8>, Vec<ResolveError>> {
    std::fs::read(path).map_err(|error| {
        refusal(
            path,
            &format!("cannot read provenance input {}: {error}", path.display()),
        )
    })
}

fn refusal(path: &Path, message: &str) -> Vec<ResolveError> {
    vec![ResolveError::new(
        ResolveErrorKind::SourceRefused(message.to_string()),
        &path.display().to_string(),
        "",
        Span::default(),
    )]
}
