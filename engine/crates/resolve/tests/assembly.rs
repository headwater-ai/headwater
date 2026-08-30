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
    scratch.write(
        "packages/acme-fixture/bundles/alpha/bundle.yml",
        "overlay: acme/fixture\n",
    );
    scratch.write(
        "packages/acme-fixture/bundles/beta/bundle.yml",
        "overlay: acme/fixture\n",
    );
    scratch.write(
        "packages/acme-fixture/assemblies/starter/assembly.yml",
        ASSEMBLY,
    );
    scratch.write(
        "packages/acme-fixture/assemblies/starter/overlay.yml",
        "overlay: acme/fixture\n",
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
