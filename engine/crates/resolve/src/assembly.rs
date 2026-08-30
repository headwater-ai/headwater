// SPDX-License-Identifier: Apache-2.0
//! Named selections of one package's bundles.
//!
//! An assembly is authored beside the package that owns its base taxonomy and
//! bundles. This module reads its recipe only. It does not resolve the selected
//! sources or write a flattened package; those operations take this typed value
//! as their input.

use crate::error::{ResolveError, ResolveErrorKind};
use crate::package::{self, ASSEMBLIES, BUNDLES};
use headwater_yaml::{Mapping, Span};
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

/// The name of the recipe file in one assembly directory.
pub const MANIFEST: &str = "assembly.yml";

/// One named recipe for a generated flattened package.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assembly {
    /// The directory name and `assembly:` value that identify this recipe.
    pub name: String,
    /// The generated package's identity.
    pub package: String,
    /// The generated package's version.
    pub version: String,
    /// The source package and complete bundle selection.
    pub from: From,
    /// The optional overlay, relative to this assembly directory.
    pub overlay: Option<PathBuf>,
    /// The recipe file this value was read from.
    pub at: PathBuf,
}

/// The package version and bundles an assembly combines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct From {
    /// The source package identity.
    pub package: String,
    /// The source package version.
    pub version: String,
    /// The complete explicit selection, in recipe order.
    pub bundles: Vec<String>,
}

/// Read a named assembly under a source package directory.
///
/// `manifest` is the source package's already-loaded `package.yml`. The reader
/// holds the recipe to that package's declared `contents.assemblies` directory,
/// checks that `from.package` pins that same package version, and verifies that
/// every listed bundle is present in its declared bundle directory. It makes no
/// claim about the overlay's operations: resolution owns that later judgment.
pub fn read(
    root: &Path,
    directory: &Path,
    manifest: &Mapping,
    name: &str,
) -> Result<Assembly, Vec<ResolveError>> {
    let manifest_name = directory.join(package::MANIFEST).display().to_string();
    let assemblies = declared_directory(root, directory, manifest, ASSEMBLIES, &manifest_name)?;
    let assembly_directory = child(&assemblies, name).ok_or_else(|| {
        refusal(
            &manifest_name,
            &format!(
                "assembly name `{name}` is not one directory name under `contents.{ASSEMBLIES}`"
            ),
        )
    })?;
    if !assembly_directory.is_dir()
        || !at_or_inside(&settled(&assemblies), &settled(&assembly_directory))
    {
        return Err(refusal(
            &manifest_name,
            &format!("assembly `{name}` is not a directory inside `contents.{ASSEMBLIES}`"),
        ));
    }
    let at = assembly_directory.join(MANIFEST);
    let source_name = at.display().to_string();
    let loaded = crate::source::load(&at)?;
    let map = loaded.value.as_map().ok_or_else(|| {
        refusal(
            &source_name,
            "an assembly recipe is a mapping with `assembly`, `package`, `version`, and `from`",
        )
    })?;

    unknown_keys(
        map,
        &source_name,
        &["assembly", "package", "version", "from", "overlay"],
    )?;
    let declared_name = required_text(map, "assembly", &source_name)?;
    if declared_name != name {
        return Err(refusal(
            &source_name,
            &format!(
                "this declares `assembly: {declared_name}`, but its directory and requested recipe are `{name}`. An assembly has one name"
            ),
        ));
    }
    let package = required_text(map, "package", &source_name)?;
    let version = required_text(map, "version", &source_name)?;
    let from = required_from(map, &source_name)?;

    let source_package = required_text(manifest, "package", &manifest_name)?;
    let source_version = required_text(manifest, "version", &manifest_name)?;
    if from.package != source_package || from.version != source_version {
        return Err(refusal(
            &source_name,
            &format!(
                "`from.package` pins {}@{}, but this recipe is under `contents.{}`, whose manifest declares {}@{}. An assembly reads the package that carries it",
                from.package, from.version, ASSEMBLIES, source_package, source_version
            ),
        ));
    }
    verify_bundles(root, directory, manifest, &from, &source_name)?;
    let overlay = optional_overlay(&assembly_directory, map, &source_name)?;

    Ok(Assembly {
        name: declared_name,
        package,
        version,
        from,
        overlay,
        at,
    })
}

fn declared_directory(
    root: &Path,
    directory: &Path,
    manifest: &Mapping,
    key: &str,
    manifest_name: &str,
) -> Result<PathBuf, Vec<ResolveError>> {
    let contents = manifest
        .get("contents")
        .and_then(|node| node.value.as_map())
        .ok_or_else(|| refusal(manifest_name, "this package declares no `contents` mapping"))?;
    let Some(node) = contents.get(key) else {
        return Err(refusal(
            manifest_name,
            &format!("this package declares no `contents.{key}` directory"),
        ));
    };
    let Some(scalar) = node.value.as_scalar() else {
        return Err(refusal(
            manifest_name,
            &format!("`contents.{key}` is one directory path"),
        ));
    };
    if scalar.text.is_empty() {
        return Err(refusal(
            manifest_name,
            &format!("`contents.{key}` is not empty"),
        ));
    }
    let declared = scalar.text.clone();
    let at = directory.join(&declared);
    if !at.is_dir() {
        return Err(refusal(
            manifest_name,
            &format!("`contents.{key}` names `{declared}`, which is not a directory"),
        ));
    }
    let settled_directory = settled(directory);
    let settled_root = settled(root);
    let settled_at = settled(&at);
    let allowed = if key == BUNDLES {
        at_or_inside(&settled_root, &settled_at)
    } else {
        at_or_inside(&settled_directory, &settled_at)
    };
    if !allowed {
        return Err(refusal(
            manifest_name,
            &format!(
                "`contents.{key}` names `{declared}`, which resolves outside the package{}",
                if key == BUNDLES { " repository" } else { "" }
            ),
        ));
    }
    Ok(at)
}

fn required_from(map: &Mapping, source: &str) -> Result<From, Vec<ResolveError>> {
    let from = map
        .get("from")
        .and_then(|node| node.value.as_map())
        .ok_or_else(|| refusal(source, "`from` is a mapping with `package` and `bundles`"))?;
    unknown_keys(from, source, &["package", "bundles"])?;
    let pinned = required_text(from, "package", source)?;
    let (package, version) = pinned.rsplit_once('@').ok_or_else(|| {
        refusal(
            source,
            "`from.package` is a package identity and version written as `owner/name@version`",
        )
    })?;
    if package.is_empty() || version.is_empty() || pinned.matches('@').count() != 1 {
        return Err(refusal(
            source,
            "`from.package` is a package identity and version written as `owner/name@version`",
        ));
    }
    let bundles = from
        .get("bundles")
        .and_then(|node| node.value.as_seq())
        .ok_or_else(|| refusal(source, "`from.bundles` is a sequence of bundle names"))?;
    let mut names = Vec::with_capacity(bundles.len());
    let mut seen = BTreeSet::new();
    for item in bundles {
        let Some(scalar) = item.value.as_scalar() else {
            return Err(refusal(
                source,
                "each `from.bundles` item is one bundle name",
            ));
        };
        if scalar.text.is_empty() || !seen.insert(scalar.text.clone()) {
            return Err(refusal(
                source,
                &format!(
                    "`from.bundles` names `{}` more than once or not at all",
                    scalar.text
                ),
            ));
        }
        names.push(scalar.text.clone());
    }
    Ok(From {
        package: package.to_string(),
        version: version.to_string(),
        bundles: names,
    })
}

fn verify_bundles(
    root: &Path,
    directory: &Path,
    manifest: &Mapping,
    from: &From,
    source: &str,
) -> Result<(), Vec<ResolveError>> {
    if from.bundles.is_empty() {
        return Ok(());
    }
    let manifest_name = directory.join(package::MANIFEST).display().to_string();
    let bundles = declared_directory(root, directory, manifest, BUNDLES, &manifest_name)?;
    for name in &from.bundles {
        let Some(bundle) = child(&bundles, name) else {
            return Err(refusal(
                source,
                &format!("`from.bundles` names `{name}`, which is not one bundle directory"),
            ));
        };
        let recipe = bundle.join("bundle.yml");
        if !recipe.is_file() || !at_or_inside(&settled(&bundles), &settled(&recipe)) {
            return Err(refusal(
                source,
                &format!(
                    "`from.bundles` names `{name}`, but {} has no `bundle.yml`",
                    bundle.display()
                ),
            ));
        }
    }
    Ok(())
}

fn optional_overlay(
    directory: &Path,
    map: &Mapping,
    source: &str,
) -> Result<Option<PathBuf>, Vec<ResolveError>> {
    let Some(node) = map.get("overlay") else {
        return Ok(None);
    };
    let Some(scalar) = node.value.as_scalar() else {
        return Err(refusal(
            source,
            "`overlay` is a file path relative to this assembly",
        ));
    };
    if scalar.text.is_empty() {
        return Err(refusal(
            source,
            "`overlay` is not empty; take the key out when there is none",
        ));
    }
    let Some(at) = relative(directory, &scalar.text) else {
        return Err(refusal(
            source,
            &format!(
                "`overlay` names `{}`, which leaves this assembly",
                scalar.text
            ),
        ));
    };
    if !at.is_file() || !at_or_inside(&settled(directory), &settled(&at)) {
        return Err(refusal(
            source,
            &format!(
                "`overlay` names `{}`, which is not a file in this assembly",
                scalar.text
            ),
        ));
    }
    Ok(Some(at))
}

fn child(directory: &Path, name: &str) -> Option<PathBuf> {
    let path = Path::new(name);
    match path.components().next() {
        Some(Component::Normal(_)) if path.components().count() == 1 => Some(directory.join(path)),
        _ => None,
    }
}

fn relative(directory: &Path, name: &str) -> Option<PathBuf> {
    let path = Path::new(name);
    if path
        .components()
        .all(|part| matches!(part, Component::Normal(_) | Component::CurDir))
        && path
            .components()
            .any(|part| matches!(part, Component::Normal(_)))
    {
        Some(directory.join(path))
    } else {
        None
    }
}

fn required_text(map: &Mapping, key: &str, source: &str) -> Result<String, Vec<ResolveError>> {
    let Some(node) = map.get(key) else {
        return Err(refusal(source, &format!("this assembly has no `{key}`")));
    };
    let Some(scalar) = node.value.as_scalar() else {
        return Err(refusal(source, &format!("`{key}` is a non-empty scalar")));
    };
    if scalar.text.is_empty() {
        return Err(refusal(source, &format!("`{key}` is not empty")));
    }
    Ok(scalar.text.clone())
}

fn unknown_keys(map: &Mapping, source: &str, allowed: &[&str]) -> Result<(), Vec<ResolveError>> {
    let unknown: Vec<&str> = map
        .iter()
        .map(|entry| entry.key.value.as_str())
        .filter(|key| !allowed.contains(key))
        .collect();
    match unknown.is_empty() {
        true => Ok(()),
        false => Err(refusal(
            source,
            &format!(
                "this assembly has an unknown key `{}`",
                unknown.join("`, `")
            ),
        )),
    }
}

fn settled(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn at_or_inside(directory: &Path, path: &Path) -> bool {
    path.starts_with(directory)
}

fn refusal(source: &str, message: &str) -> Vec<ResolveError> {
    vec![ResolveError::new(
        ResolveErrorKind::SourceRefused(message.to_string()),
        source,
        "",
        Span::default(),
    )]
}
