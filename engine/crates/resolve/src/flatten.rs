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
pub fn materialize(
    root: &Path,
    directory: &Path,
    source_manifest: &Mapping,
    recipe: &Assembly,
) -> Result<Flattened, Vec<ResolveError>> {
    let resolution = assembly::resolve(root, directory, source_manifest, recipe)?;
    let sources = assembly_sources(root, directory, source_manifest, recipe)?;
    let manifest = manifest(source_manifest, recipe, &sources)?;
    let taxonomy = taxonomy(&resolution, recipe);
    let assets = assets(directory, source_manifest, recipe)?;
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
    entries.push(entry(
        "contents",
        Value::Map(contents(source, recipe, &recipe.at)?),
    ));
    entries.push(entry(
        "distribution",
        Value::Map(distribution(recipe, sources)?),
    ));
    Ok(Mapping::new(entries))
}

fn contents(source: &Mapping, recipe: &Assembly, at: &Path) -> Result<Mapping, Vec<ResolveError>> {
    let source_contents = source
        .get("contents")
        .and_then(|node| node.value.as_map())
        .ok_or_else(|| {
            refusal(
                at,
                "the source package manifest has no mapping at `contents`",
            )
        })?;
    let mut entries = vec![entry("taxonomy", scalar("taxonomy.yml"))];
    entries.extend(
        source_contents
            .entries()
            .iter()
            .filter(|entry| {
                entry.key.value != "taxonomy"
                    && entry.key.value != BUNDLES
                    && entry.key.value != ASSEMBLIES
            })
            .filter(|entry| {
                entry.key.value != package::DOCTRINE && entry.key.value != package::TEMPLATES
            })
            .cloned(),
    );
    for key in [package::DOCTRINE, package::TEMPLATES] {
        if source_contents.get(key).is_some() {
            entries.push(entry(key, scalar(&format!("{key}/{}", recipe.name))));
        }
    }
    Ok(Mapping::new(entries))
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

fn assets(
    directory: &Path,
    manifest: &Mapping,
    recipe: &Assembly,
) -> Result<Vec<Asset>, Vec<ResolveError>> {
    let Some(contents) = manifest
        .get("contents")
        .and_then(|node| node.value.as_map())
    else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for key in [package::DOCTRINE, package::TEMPLATES] {
        let Some(path) = contents
            .get(key)
            .and_then(|node| node.value.as_scalar())
            .map(|scalar| scalar.text.as_str())
        else {
            continue;
        };
        let at = contained(directory, path, &recipe.at)?;
        collect(
            &at,
            &at,
            &format!("{key}/{}", recipe.name),
            &mut out,
            &recipe.at,
        )?;
    }
    Ok(out)
}

fn contained(directory: &Path, declared: &str, at: &Path) -> Result<PathBuf, Vec<ResolveError>> {
    let path = Path::new(declared);
    if path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(refusal(
            at,
            &format!("`contents` path `{declared}` leaves the source package"),
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
            &format!("cannot read declared asset directory `{declared}`: {error}"),
        )
    })?;
    if !candidate.starts_with(boundary) || !candidate.is_dir() {
        return Err(refusal(
            at,
            &format!("`contents` path `{declared}` is not a directory inside the source package"),
        ));
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
