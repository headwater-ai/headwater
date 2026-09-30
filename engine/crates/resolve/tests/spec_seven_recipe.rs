// SPDX-License-Identifier: Apache-2.0
//! Spec 7's *An assembly has two consumption forms* section, held against the
//! starter recipe this repository ships.
//!
//! # Why this file exists
//!
//! The section prints the recipe at
//! `taxonomy-source/headwater-standard/assemblies/starter/assembly.yml` twice:
//! once as a fenced YAML block and once as the first line of a text diagram.
//! The sentence above the block says it is "the one this repository ships".
//! Nothing held that sentence, so each bump of `headwater/standard` left the
//! printed pin one version further behind. HW-OBL-0162 records that the pin
//! read `4.0.0` while the package stood at `4.13.0`, and that a rewrite of the
//! passage had fixed it once before and it went stale again.
//!
//! A bump now moves this file's source of truth, `assembly.yml`, and this case
//! fails until spec 7 moves with it. That is the first route HW-OBL-0162
//! names: a check that reads a fenced example against its source.
//!
//! # The model
//!
//! `spec_seven_subjects.rs` binds another passage of spec 7 to the engine, and
//! this file takes its shape. The case compares identifiers and values, never
//! prose, and it reads both sides from their files rather than from a copy
//! written here.
//!
//! # What this does not hold
//!
//! The comments in `assembly.yml` and the paragraphs around the block are
//! prose, and nothing here reads them. The diagram's other lines name the three
//! bundles in a drawing, and only its first line, the pinned package, is read.
//! The block and the recipe are compared on the five values a composer copies:
//! `assembly`, `package`, `version`, `from.package` and `from.bundles`.

use std::path::{Path, PathBuf};

use headwater_yaml::{Mapping, Spanned, Value};

/// The heading the section opens with. The section ends at the next heading.
const SECTION: &str = "### An assembly has two consumption forms";

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn read(relative: &str) -> String {
    let path = repo().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The text of the section, from its heading to the next heading of any level.
fn section() -> String {
    let text = read("docs/spec/07-distribution-and-federation.md");
    let (_, after) = text
        .split_once(SECTION)
        .unwrap_or_else(|| panic!("spec 7: no `{SECTION}` heading"));
    let mut lines = Vec::new();
    let mut in_fence = false;
    for line in after.lines().skip(1) {
        if line.starts_with("```") {
            in_fence = !in_fence;
        }
        if !in_fence && line.starts_with('#') {
            break;
        }
        lines.push(line);
    }
    lines.join("\n")
}

/// The body of the first fence in `section` that opens with ```` ```<info> ````.
fn fence(section: &str, info: &str) -> String {
    let opener = format!("```{info}\n");
    let (_, after) = section
        .split_once(&opener)
        .unwrap_or_else(|| panic!("spec 7 `{SECTION}`: no ```{info} block"));
    let (body, _) = after
        .split_once("\n```")
        .unwrap_or_else(|| panic!("spec 7 `{SECTION}`: the ```{info} block is not closed"));
    body.to_string()
}

fn map<'a>(root: &'a Spanned<Value>, what: &str) -> &'a Mapping {
    root.as_map()
        .unwrap_or_else(|| panic!("{what}: the root is not a mapping"))
}

fn scalar(map: &Mapping, key: &str, what: &str) -> String {
    map.get(key)
        .and_then(|v| v.as_scalar())
        .unwrap_or_else(|| panic!("{what}: `{key}` is not a scalar"))
        .text
        .clone()
}

/// The five values a composer copies, in the order the block writes them.
fn recipe(root: &Spanned<Value>, what: &str) -> Vec<(String, String)> {
    let top = map(root, what);
    let from = top
        .get("from")
        .and_then(|v| v.as_map())
        .unwrap_or_else(|| panic!("{what}: `from` is not a mapping"));
    let bundles: Vec<String> = from
        .get("bundles")
        .and_then(|v| v.as_seq())
        .unwrap_or_else(|| panic!("{what}: `from.bundles` is not a sequence"))
        .iter()
        .map(|b| {
            b.as_scalar()
                .unwrap_or_else(|| panic!("{what}: a bundle is not a scalar"))
                .text
                .clone()
        })
        .collect();
    vec![
        ("assembly".into(), scalar(top, "assembly", what)),
        ("package".into(), scalar(top, "package", what)),
        ("version".into(), scalar(top, "version", what)),
        ("from.package".into(), scalar(from, "package", what)),
        ("from.bundles".into(), bundles.join(", ")),
    ]
}

/// Spec 7 prints the starter recipe that this repository ships, and the text
/// diagram below it starts from the same pinned package.
#[test]
fn the_recipe_spec_seven_prints_is_the_recipe_this_repository_ships() {
    let source_path = "taxonomy-source/headwater-standard/assemblies/starter/assembly.yml";
    let shipped = headwater_yaml::load(&read(source_path))
        .unwrap_or_else(|e| panic!("{source_path}: {e:?}"));
    let shipped = recipe(&shipped, source_path);

    let section = section();
    let block = fence(&section, "yaml");
    let printed = headwater_yaml::load(&block)
        .unwrap_or_else(|e| panic!("spec 7 `{SECTION}` yaml block: {e:?}"));
    let printed = recipe(&printed, "spec 7 yaml block");

    assert_eq!(
        printed, shipped,
        "spec 7 `{SECTION}` prints a recipe that differs from {source_path}. The section says \
         the block is the recipe this repository ships, so move the block to the file"
    );

    let pin = &shipped[3].1;
    let diagram = fence(&section, "text");
    let first = diagram.lines().next().unwrap_or_default().trim();
    assert_eq!(
        first, pin,
        "spec 7 `{SECTION}`: the text diagram starts from `{first}`, but {source_path} pins \
         `from.package: {pin}`"
    );
}
