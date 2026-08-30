// SPDX-License-Identifier: Apache-2.0
//! Assembly recipes are source declarations, not generated artifacts.

use headwater_resolve::{assembly, package};
use std::path::PathBuf;

struct Scratch(PathBuf);

impl Scratch {
    fn new(case: &str) -> Self {
        let at =
            std::env::temp_dir().join(format!("headwater-assembly-{}-{case}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the scratch tree is made");
        Self(at)
    }

    fn write(&self, relative: &str, text: &str) {
        let at = self.0.join(relative);
        std::fs::create_dir_all(at.parent().expect("the file has a parent"))
            .expect("the parent is made");
        std::fs::write(at, text).expect("the file is written");
    }

    fn package(&self) -> PathBuf {
        self.0.join("packages/acme-fixture")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const PACKAGE: &str = "\
package: acme/fixture
version: 1.0.0
contents:
  taxonomy: taxonomy.yml
  bundles: bundles
  assemblies: assemblies
";

const TAXONOMY: &str = "\
taxonomy: acme/fixture
version: 1.0.0
purposes:
  behavior: {intent: state what the system does}
kinds:
  governed_document: {abstract: true}
  specification: {is_a: governed_document, purpose: behavior}
core:
  requires:
    - purpose: behavior
";

const ALPHA: &str = "\
bundle: alpha
extends: acme/fixture@1.0.0
requires: []
add:
  kinds.alpha: {is_a: governed_document, purpose: behavior}
";

const BETA: &str = "\
bundle: beta
extends: acme/fixture@1.0.0
requires: []
add:
  kinds.beta: {is_a: governed_document, purpose: behavior}
";

const GLUE: &str = "\
add:
  relations.connects:
    family: derivation
    from: [alpha]
    to: [beta]
    inverse: connected_by
    reciprocal: required
    created_by: scaffold
";

const ASSEMBLY: &str = "\
assembly: starter
package: acme/starter
version: 2.0.0
from:
  package: acme/fixture@1.0.0
  bundles: [alpha, beta]
overlay: overlay.yml
";

fn publisher(scratch: &Scratch) {
    scratch.write("packages/acme-fixture/package.yml", PACKAGE);
    scratch.write("packages/acme-fixture/taxonomy.yml", TAXONOMY);
    scratch.write("packages/acme-fixture/bundles/alpha/bundle.yml", ALPHA);
    scratch.write("packages/acme-fixture/bundles/beta/bundle.yml", BETA);
    scratch.write(
        "packages/acme-fixture/assemblies/starter/assembly.yml",
        ASSEMBLY,
    );
    scratch.write("packages/acme-fixture/assemblies/starter/overlay.yml", GLUE);
}

#[test]
fn an_assembly_resolves_the_base_bundles_then_its_glue_relation() {
    let scratch = Scratch::new("resolve");
    publisher(&scratch);
    let directory = scratch.package();
    let manifest = package::manifest_at(&directory).expect("the manifest reads");
    let recipe =
        assembly::read(&scratch.0, &directory, &manifest, "starter").expect("the recipe reads");

    let sources = assembly::sources(&scratch.0, &directory, &manifest, &recipe)
        .expect("the selected sources read");
    assert_eq!(sources.len(), 3);
    assert!(sources[0].name.ends_with("taxonomy.yml"));
    assert!(sources[1].name.ends_with("bundles/alpha/bundle.yml"));
    assert!(sources[2].name.ends_with("bundles/beta/bundle.yml"));

    let resolved = assembly::resolve(&scratch.0, &directory, &manifest, &recipe)
        .expect("the assembly resolves");
    assert_eq!(resolved.sources.len(), 4);
    assert!(resolved.sources[3].ends_with("assemblies/starter/overlay.yml"));
    assert!(resolved.taxonomy.get("relations").is_some());
}

#[test]
fn an_assembly_overlay_cannot_restate_a_source_declaration() {
    let scratch = Scratch::new("restate");
    publisher(&scratch);
    scratch.write(
        "packages/acme-fixture/assemblies/starter/overlay.yml",
        "add:\n  purposes.behavior: {intent: a second statement}\n",
    );
    let directory = scratch.package();
    let manifest = package::manifest_at(&directory).expect("the manifest reads");
    let recipe =
        assembly::read(&scratch.0, &directory, &manifest, "starter").expect("the recipe reads");

    let refused = assembly::resolve(&scratch.0, &directory, &manifest, &recipe)
        .expect_err("the restatement is refused");
    assert!(
        refused[0].to_string().contains("cannot restate"),
        "the refusal does not name the assembly boundary: {}",
        refused[0]
    );
}

#[test]
fn an_assembly_relation_must_connect_two_selected_bundle_owners() {
    let scratch = Scratch::new("one-owner");
    publisher(&scratch);
    scratch.write(
        "packages/acme-fixture/assemblies/starter/overlay.yml",
        GLUE.replace("to: [beta]", "to: [alpha]").as_str(),
    );
    let directory = scratch.package();
    let manifest = package::manifest_at(&directory).expect("the manifest reads");
    let recipe =
        assembly::read(&scratch.0, &directory, &manifest, "starter").expect("the recipe reads");

    let refused = assembly::resolve(&scratch.0, &directory, &manifest, &recipe)
        .expect_err("the one-bundle relation is refused");
    assert!(
        refused[0]
            .to_string()
            .contains("at least two selected bundles"),
        "the refusal does not name the required connection: {}",
        refused[0]
    );
}

fn read(scratch: &Scratch) -> Result<assembly::Assembly, Vec<headwater_resolve::ResolveError>> {
    let directory = scratch.package();
    let manifest = package::manifest_at(&directory).expect("the manifest reads");
    assembly::read(&scratch.0, &directory, &manifest, "starter")
}

#[test]
fn a_recipe_reads_to_its_typed_source_package_and_overlay() {
    let scratch = Scratch::new("valid");
    publisher(&scratch);

    let found = read(&scratch).expect("the recipe reads");
    assert_eq!(found.name, "starter");
    assert_eq!(found.package, "acme/starter");
    assert_eq!(found.version, "2.0.0");
    assert_eq!(found.from.package, "acme/fixture");
    assert_eq!(found.from.version, "1.0.0");
    assert_eq!(found.from.bundles, ["alpha", "beta"]);
    assert_eq!(
        found.overlay,
        Some(scratch.package().join("assemblies/starter/overlay.yml"))
    );
}

#[test]
fn a_recipe_refuses_an_unknown_or_duplicate_bundle_before_resolution() {
    let scratch = Scratch::new("bundle");
    publisher(&scratch);
    let recipe = scratch.package().join("assemblies/starter/assembly.yml");
    std::fs::write(
        &recipe,
        ASSEMBLY.replace("[alpha, beta]", "[alpha, alpha, missing]"),
    )
    .expect("the recipe changes");

    let refused = read(&scratch).expect_err("the duplicate is refused");
    assert!(
        refused[0].to_string().contains("alpha")
            && refused[0].to_string().contains("more than once"),
        "the refusal does not name the duplicate: {}",
        refused[0]
    );

    std::fs::write(&recipe, ASSEMBLY.replace("[alpha, beta]", "[missing]"))
        .expect("the recipe changes again");
    let refused = read(&scratch).expect_err("the unknown bundle is refused");
    assert!(
        refused[0].to_string().contains("missing") && refused[0].to_string().contains("bundle.yml"),
        "the refusal does not name the unknown bundle: {}",
        refused[0]
    );
}

#[test]
fn a_recipe_refuses_malformed_fields_and_an_overlay_that_leaves_its_directory() {
    let scratch = Scratch::new("malformed");
    publisher(&scratch);
    let recipe = scratch.package().join("assemblies/starter/assembly.yml");
    std::fs::write(
        &recipe,
        ASSEMBLY.replace("acme/fixture@1.0.0", "acme/fixture"),
    )
    .expect("the recipe changes");
    let refused = read(&scratch).expect_err("the unpinned source is refused");
    assert!(
        refused[0].to_string().contains("owner/name@version"),
        "the refusal does not state the source-package form: {}",
        refused[0]
    );

    std::fs::write(&recipe, ASSEMBLY.replace("overlay.yml", "../overlay.yml"))
        .expect("the recipe changes again");
    let refused = read(&scratch).expect_err("the escaping overlay is refused");
    assert!(
        refused[0].to_string().contains("leaves this assembly"),
        "the refusal does not name the boundary: {}",
        refused[0]
    );
}

#[test]
fn an_assemblies_directory_must_stay_inside_its_source_package() {
    let scratch = Scratch::new("directory");
    publisher(&scratch);
    scratch.write("outside/starter/assembly.yml", ASSEMBLY);
    let package = scratch.package().join("package.yml");
    std::fs::write(
        &package,
        PACKAGE.replace("assemblies: assemblies", "assemblies: ../../outside"),
    )
    .expect("the manifest changes");

    let refused = read(&scratch).expect_err("the escaping directory is refused");
    assert!(
        refused[0].to_string().contains("contents.assemblies")
            && refused[0].to_string().contains("outside the package"),
        "the refusal does not state the package boundary: {}",
        refused[0]
    );
}
