// SPDX-License-Identifier: Apache-2.0
//! `headwater taxonomy publish`, from the state an adopter is actually in.
//!
//! # The defect this target exists for
//!
//! The library guarantee is held one crate down, in
//! `headwater-resolve/tests/publish.rs`: a publish that cannot read what its
//! manifest declares leaves `--out` exactly as it found it. The message a person
//! reads is a different component. `main.rs` prints `headwater: nothing was
//! published` for every error the library returns, with no knowledge of whether
//! a byte was written, so that line was a claim about the disk that no code
//! established. [#271] is the issue where it was false: three files and an empty
//! `bundles/` under a directory the run said it had not written to, and a second
//! run refused by the verb's own precondition catching the first run's
//! leftovers.
//!
//! A correctness root can be completely right and still produce a broken result,
//! and only running the next component reveals it. So this case runs the built
//! binary and reads standard error and the disk, rather than calling the library
//! that the case below it already covers.
//!
//! # The root it runs over
//!
//! This repository's own maintained source of `headwater/standard` —
//! `taxonomy-source/headwater-standard/` since #336 moved it out of
//! `packages/` — copied to where `--package` looks a package up, and nothing
//! else. That is the state `headwater init` sends a newcomer into: its refusal
//! says *"package headwater/standard is not under `packages/`, and nothing here
//! fetches one"*, and the copy that line invites carries a manifest whose
//! `contents.bundles` climbs out with `../..` into a library that the copy left
//! behind. `packages/headwater-standard/` itself is no longer this fixture,
//! because #336 also made it a vendored artifact whose manifest carries the
//! library inside it rather than climbing out to reach one.
//!
//! [#271]: https://github.com/headwater-ai/headwater/issues/271

use headwater_resolve::{assembly, flatten, package};
use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// A scratch tree that removes itself, named for the case that made it.
struct Root(PathBuf);

impl Root {
    /// An empty tree, for a case whose root is this repository itself.
    fn scratch(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-publish-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the scratch tree is made");
        Root(at)
    }

    /// This repository's package and the bundle library it points at, copied,
    /// with one regular file named `bundles` added at the package root.
    ///
    /// This root publishes right up to the write phase and fails inside it: the
    /// staged set holds a file at `bundles` and files under `bundles/`, so `put`
    /// writes part of the artifact and then cannot make a directory where it has
    /// just written a file. It is the only provocation in this file that reaches
    /// the undo, and no mode and no race is in it. See the fixture of the same
    /// shape in `headwater-resolve/tests/publish.rs` for why the suite needed
    /// one.
    fn colliding(label: &str) -> Root {
        let root = Root::scratch(label);
        copy(
            &repository().join("taxonomy-source/headwater-standard"),
            &root.0.join("packages/headwater-standard"),
        );
        copy(
            &repository().join("docs/taxonomies"),
            &root.0.join("docs/taxonomies"),
        );
        std::fs::write(
            root.0.join("packages/headwater-standard/bundles"),
            "a regular file where the artifact needs a directory\n",
        )
        .expect("the colliding file is written");
        root
    }

    /// This repository's maintained source, copied to where `--package` looks
    /// a package up, with no library beside it.
    fn copied(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-publish-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");
        copy(
            &repository().join("taxonomy-source/headwater-standard"),
            &at.join("packages/headwater-standard"),
        );
        assert!(
            !at.join("docs/taxonomies").exists(),
            "the library the manifest points at must not be in the copy"
        );
        Root(at)
    }

    fn publish(&self, out: &Path) -> (Option<i32>, String) {
        publish_from(&self.0, out)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

/// `taxonomy publish` over a root the caller names, as a person types it.
fn publish_from(root: &Path, out: &Path) -> (Option<i32>, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["taxonomy", "publish", "--package", "headwater/standard"])
        .arg("--out")
        .arg(out)
        .arg("--root")
        .arg(root)
        .output()
        .expect("the binary runs");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// `taxonomy publish --from` over this repository's own maintained source.
///
/// `--package headwater/standard` now finds `packages/headwater-standard/`,
/// and #366 made that refuse: the directory carries a release record, so it
/// was vendored rather than maintained by hand. `taxonomy-source/headwater-standard/`
/// is the maintained source, and this is the real, substantial artifact this
/// case needs — not a synthetic one — so it names that directory with `--from`
/// rather than switching to a smaller fixture.
fn publish_real_source_into(out: &Path) -> (Option<i32>, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .arg("taxonomy")
        .arg("publish")
        .arg("--from")
        .arg(repository().join("taxonomy-source/headwater-standard"))
        .arg("--out")
        .arg(out)
        .arg("--root")
        .arg(repository())
        .output()
        .expect("the binary runs");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// `taxonomy publish --from` over the maintained source, with or without
/// `--json`, and both streams kept apart: the document is on one, and an
/// account of a refusal is on the other.
fn publish_real_source(out: &Path, json: bool) -> (Option<i32>, String, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_headwater"));
    command
        .arg("taxonomy")
        .arg("publish")
        .arg("--from")
        .arg(repository().join("taxonomy-source/headwater-standard"))
        .arg("--out")
        .arg(out)
        .arg("--root")
        .arg(repository());
    if json {
        command.arg("--json");
    }
    let output = command.output().expect("the binary runs");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// Which shape of source `assembly_source` writes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Arm {
    /// Every declared path carries what a reader of it opens.
    Sound,
    /// The recipe names a bundle the package does not ship.
    UnselectableBundle,
    /// Three declared directories exist and hold no file: `doctrine`,
    /// `templates`, and `glossary`, which is a key no table in this engine
    /// names.
    ///
    /// This is the state a *correct* stager still produces, on both publish
    /// paths: `reachable` passes it, because each directory is there and is a
    /// directory, and the walk that carries bytes then carries none. The
    /// manifest reaches a consumer naming three paths the artifact does not
    /// hold.
    ///
    /// **Three keys rather than one, deliberately.** Two guarding cases over
    /// `doctrine` alone would pass a `carried` narrowed to the two keys this
    /// issue happened to be about, so nothing would hold the generality the
    /// issue asks for. `templates` is a second key `required_kind` names,
    /// `glossary` is a key it does not, and the pair of them is what makes
    /// "every key the manifest declares" a measurement. It also pins that every
    /// bad key is reported and not the first.
    UnwrittenMembers,
    /// The recipe selects `alpha` alone, so `shelves.betas.kind` reads `beta`
    /// and only the `beta` bundle declares it.
    ///
    /// **This arm declares no overlay, and that is the whole reason it
    /// measures anything.** The overlay every other arm carries is
    /// `relations.connects {from: [alpha], to: [beta]}`, and `validate_glue`
    /// refuses it the moment `beta` is not selected, because glue that is not
    /// cross-bundle is not glue. An arm that narrowed the selection and kept
    /// the overlay would be refused for that reason, go red before this change
    /// as well as after it, and say nothing at all about referential integrity.
    IncompleteSelection,
}

/// A source package with one named assembly, small enough to make the publish
/// boundary visible in isolation from the real one.
///
/// It carries the shapes the shipped starter does not: an assembly overlay, a
/// templates directory, an arm that names a bundle the package does not ship,
/// and an arm whose declared doctrine directory is empty.
/// `the_shipped_starter_recipe_publishes_vendors_and_resolves` is the case that
/// runs the same verb over the real source.
///
/// **It declares `contents.conformance` and `contents.glossary`, and both are
/// deliberate.** Until #581 no fixture in this file declared a file-valued
/// `contents` key beyond `taxonomy`, so nothing here ever asked whether a
/// flattened artifact carries one; `headwater/standard` declares one and the
/// flattened starter did not carry it. `glossary` is a key **no table in this
/// engine names** — `required_kind` returns `None` for it — so it is what holds
/// the claim that a flattened package carries every declared member rather than
/// the members somebody listed in Rust.
fn assembly_source(root: &Root, arm: Arm) -> PathBuf {
    let source = root.path().join("source/acme-fixture");
    write(
        &source.join("package.yml"),
        "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n  conformance: conformance.yml\n  glossary: glossary.yml\n  bundles: bundles\n  assemblies: assemblies\n  doctrine: doctrine\n  templates: templates\n",
    );
    write(
        &source.join("glossary.yml"),
        "glossary:\n  headwater: the system this fixture is a fixture of\n",
    );
    write(
        &source.join("conformance.yml"),
        "conformance:\n  format: 1\n  rules:\n    - name: pin.current\n      title: The pin names the package that is installed\n      decided_by: tree\n      statement: The pin names a version and a digest that the installed artifact carries.\n      remediation: Publish the package, write the printed digest into the pin, and vendor it.\n  levels:\n    - name: L0\n      title: Pointed at\n      rules: [pin.current]\n",
    );
    write(
        &source.join("taxonomy.yml"),
        "taxonomy: acme/fixture\nversion: 1.0.0\npurposes:\n  behavior: {intent: state what the system does}\nkinds:\n  governed_document: {abstract: true}\n  specification: {is_a: governed_document, purpose: behavior}\nshelves:\n  specifications: {path: docs/specifications/**, homogeneous: true, kind: specification}\n  alphas: {path: docs/alphas/**, homogeneous: true, kind: alpha}\n  betas: {path: docs/betas/**, homogeneous: true, kind: beta}\ncore:\n  requires:\n    - purpose: behavior\n",
    );
    write(
        &source.join("bundles/alpha/bundle.yml"),
        "bundle: alpha\nextends: acme/fixture@1.0.0\nrequires: []\nadd:\n  kinds.alpha: {is_a: governed_document, purpose: behavior}\n",
    );
    write(
        &source.join("bundles/beta/bundle.yml"),
        "bundle: beta\nextends: acme/fixture@1.0.0\nrequires: []\nadd:\n  kinds.beta: {is_a: governed_document, purpose: behavior}\n",
    );
    let selected = match arm {
        Arm::UnselectableBundle => "[alpha, missing]",
        Arm::IncompleteSelection => "[alpha]",
        Arm::Sound | Arm::UnwrittenMembers => "[alpha, beta]",
    };
    let glue = match arm {
        Arm::IncompleteSelection => "",
        Arm::Sound | Arm::UnselectableBundle | Arm::UnwrittenMembers => "overlay: overlay.yml\n",
    };
    write(
        &source.join("assemblies/starter/assembly.yml"),
        &format!(
            "assembly: starter\npackage: acme/starter\nversion: 2.0.0\nfrom:\n  package: acme/fixture@1.0.0\n  bundles: {selected}\n{glue}"
        ),
    );
    write(
        &source.join("assemblies/starter/overlay.yml"),
        "add:\n  relations.connects:\n    family: derivation\n    from: [alpha]\n    to: [beta]\n    nucleus: from\n    inverse: connected_by\n    reciprocal: required\n    created_by: scaffold\n",
    );
    match arm {
        Arm::UnwrittenMembers => {
            for key in ["doctrine", "templates", "glossary"] {
                std::fs::create_dir_all(source.join(key))
                    .expect("the empty declared directory is made");
            }
            // The scalar has to name the directory rather than the file the
            // sound arm writes, or `reachable` refuses this arm for the kind
            // rather than `carried` refusing it for the absence, and the case
            // would report on the wrong rule.
            write(
                &source.join("package.yml"),
                "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n  conformance: conformance.yml\n  glossary: glossary\n  bundles: bundles\n  assemblies: assemblies\n  doctrine: doctrine\n  templates: templates\n",
            );
        }
        Arm::Sound | Arm::UnselectableBundle | Arm::IncompleteSelection => {
            write(&source.join("doctrine/guide.md"), "# Fixture doctrine\n");
            write(&source.join("templates/decision.md"), "# Decision\n");
        }
    }
    source
}

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().expect("the file has a parent"))
        .expect("the parent is made");
    std::fs::write(path, text).expect("the fixture file is written");
}

fn publish_assembly_from(root: &Path, source: &Path, out: &Path) -> (Option<i32>, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["taxonomy", "publish", "--assembly", "starter", "--from"])
        .arg(source)
        .arg("--out")
        .arg(out)
        .arg("--root")
        .arg(root)
        .output()
        .expect("the binary runs");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// `taxonomy publish --from` over a source the caller names, with no recipe.
///
/// The plain stager, beside `publish_assembly_from`'s flattening one. The two
/// reach `--out` by different functions and #581 is a defect of both, so a case
/// about what an artifact carries needs one call of each.
fn publish_plain_from(root: &Path, source: &Path, out: &Path) -> (Option<i32>, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["taxonomy", "publish", "--from"])
        .arg(source)
        .arg("--out")
        .arg(out)
        .arg("--root")
        .arg(root)
        .output()
        .expect("the binary runs");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// Run a consumer verb against the root that holds its authored pin.
fn consumer_run(root: &Path, arguments: &[&str]) -> (Option<i32>, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(arguments)
        .arg("--root")
        .arg(root)
        .output()
        .expect("the binary runs");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// A consumer takes the flattened package as a package, rather than repeating
/// the source assembly's bundle selection.
fn flattened_consumer(root: &Root, digest: &str) -> PathBuf {
    let consumer = root.path().join("consumer");
    write(
        &consumer.join(".headwater/taxonomy.yml"),
        &format!(
            "taxonomy:\n  package: acme/starter\n  version: 2.0.0\n  digest: {digest}\ncorpus:\n  root: docs\n"
        ),
    );
    consumer
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the destination is made");
    for entry in std::fs::read_dir(from).expect("the source reads") {
        let entry = entry.expect("the entry reads").path();
        let name = entry.file_name().expect("it has a name");
        match entry.is_dir() {
            true => copy(&entry, &to.join(name)),
            false => {
                std::fs::copy(&entry, to.join(name)).expect("the file copies");
            }
        }
    }
}

#[test]
fn a_named_assembly_publishes_one_flattened_package() {
    let root = Root::scratch("assembly-publishes");
    let source = assembly_source(&root, Arm::Sound);
    let out = root.path().join("release");

    let (code, stdout, stderr) = publish_assembly_from(root.path(), &source, &out);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(stdout.contains("published acme/starter 2.0.0"), "{stdout}");
    assert!(
        out.join("release.yml").is_file(),
        "no release record was written"
    );
    let manifest = std::fs::read_to_string(out.join("package.yml")).expect("the manifest reads");
    assert!(manifest.contains("package: acme/starter"), "{manifest}");
    assert!(manifest.contains("taxonomy: taxonomy.yml"), "{manifest}");
    assert!(manifest.contains("form: flattened"), "{manifest}");
    assert!(
        !out.join("bundles").exists(),
        "a flattened package carries bundles"
    );
    assert!(out.join("doctrine/starter/guide.md").is_file());
    assert!(out.join("templates/starter/decision.md").is_file());
}

/// Every key the source declares reaches the flattened artifact, not five of them.
///
/// `flatten::contents` copies through every `contents` key it does not filter,
/// and `flatten::assets` writes two. `conformance` is on one side of that gap
/// and not the other, so before #581 the flattened manifest named a rule set
/// the artifact did not hold and `headwater conformance` failed on the machine
/// of whoever vendored it.
#[test]
fn a_flattened_package_carries_the_conformance_rule_set_its_source_declared() {
    let root = Root::scratch("assembly-conformance");
    let source = assembly_source(&root, Arm::Sound);
    let out = root.path().join("release");

    let (code, _stdout, stderr) = publish_assembly_from(root.path(), &source, &out);
    assert_eq!(code, Some(0), "{stderr}");

    let manifest = std::fs::read_to_string(out.join("package.yml")).expect("the manifest reads");
    assert!(
        manifest.contains("conformance: conformance.yml"),
        "{manifest}"
    );
    assert!(
        out.join("conformance.yml").is_file(),
        "the manifest declares a conformance rule set the artifact does not carry"
    );
    // The key no table in this engine names. `required_kind` returns `None` for
    // it, so nothing in the fix knows what a `glossary` is, and it travels
    // anyway. That is the difference between "every declared member" and "the
    // members somebody remembered to list".
    assert!(manifest.contains("glossary: glossary.yml"), "{manifest}");
    assert!(
        out.join("glossary.yml").is_file(),
        "a `contents` key no table names did not travel, so the fix is a list rather than a rule"
    );
    let record = std::fs::read_to_string(out.join("release.yml")).expect("the release reads");
    for member in ["conformance.yml", "glossary.yml"] {
        assert!(
            record.contains(member),
            "the release record does not name {member}: {record}"
        );
    }
}

/// A declared directory with no file in it is the case a correct stager still
/// produces, and it is why the guard is general rather than a second key check.
///
/// `reachable` admits each of these: the directory is there and it is a
/// directory. The walk that carries bytes then carries none, because a directory
/// with no file in it has nothing to copy. Both these cases were measured at
/// exit 0 before #581, with the manifest naming a path the artifact did not
/// hold, on both publish paths.
///
/// **Three keys are asserted, and `glossary` is the one that matters most.**
/// `required_kind` names `doctrine` and `templates` and does not name
/// `glossary`, so a guard narrowed to the keys this issue was about would pass a
/// case over `doctrine` alone. Asserting all three at once also pins that every
/// bad key is reported rather than the first.
#[test]
fn a_flattened_manifest_may_not_name_a_member_the_artifact_does_not_carry() {
    let root = Root::scratch("assembly-unwritten");
    let source = assembly_source(&root, Arm::UnwrittenMembers);
    let out = root.path().join("release");

    let (code, _stdout, stderr) = publish_assembly_from(root.path(), &source, &out);
    assert_eq!(code, Some(1), "{stderr}");
    for key in ["doctrine", "templates", "glossary"] {
        assert!(stderr.contains(&format!("`contents.{key}`")), "{stderr}");
    }
    // The namespaced form, which is what the flattened manifest declares and
    // what a consumer would have opened.
    assert!(stderr.contains("doctrine/starter"), "{stderr}");
    assert!(stderr.contains("templates/starter"), "{stderr}");
    assert!(
        !out.exists(),
        "the refused run left partial output at {out:?}"
    );
}

#[test]
fn a_published_manifest_may_not_name_a_member_the_artifact_does_not_carry() {
    let root = Root::scratch("plain-unwritten");
    let source = assembly_source(&root, Arm::UnwrittenMembers);
    let out = root.path().join("release");

    let (code, _stdout, stderr) = publish_plain_from(root.path(), &source, &out);
    assert_eq!(code, Some(1), "{stderr}");
    for key in ["doctrine", "templates", "glossary"] {
        assert!(stderr.contains(&format!("`contents.{key}`")), "{stderr}");
    }
    assert!(
        !out.exists(),
        "the refused run left partial output at {out:?}"
    );
}

/// A key a flattened manifest drops is a key the publisher is told about.
///
/// `flatten::DROPPED` is the ruling — `bundles`, `assemblies` and `migrations`
/// state the source package's composition and version line, and a flattened
/// package inherits neither. The ruling is not in question here. What this pins
/// is that the loss is **reported**, because a publisher who declared
/// `migrations` and receives an artifact without it has lost something they
/// asked for, and #581 is the issue that refuses exactly that in the other
/// direction.
///
/// The source carries a real payload, so the case fails if the drop ever becomes
/// a silent one again: before this, the identical source produced an artifact
/// byte-identical to one declaring no migrations at all.
#[test]
fn a_flattened_publish_names_every_contents_key_it_drops() {
    let root = Root::scratch("assembly-drops");
    let source = assembly_source(&root, Arm::Sound);
    write(
        &source.join("package.yml"),
        "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n  conformance: conformance.yml\n  glossary: glossary.yml\n  bundles: bundles\n  assemblies: assemblies\n  migrations: migrations\n  doctrine: doctrine\n  templates: templates\n",
    );
    write(
        &source.join("migrations/0-to-1.yml"),
        "migration:\n  format: 1\n  from: \">=0 <1\"\n  to: \">=1 <2\"\n\nsteps:\n  - subject: overlay_address\n    from: purposes.gone\n    to: [purposes.behavior]\n    because: the case is about what a flattened manifest drops\n",
    );
    let out = root.path().join("release");

    let (code, _stdout, stderr) = publish_assembly_from(root.path(), &source, &out);
    assert_eq!(code, Some(0), "{stderr}");

    let manifest = std::fs::read_to_string(out.join("package.yml")).expect("the manifest reads");
    // The `contents` block alone. `distribution.derived_from` carries a
    // `bundles:` of its own — the selection this package was flattened from —
    // and that one is provenance rather than a path, so a search over the whole
    // manifest would read it as a declaration and never fail.
    let contents = manifest
        .split_once("contents:")
        .and_then(|(_, rest)| rest.split_once("\ndistribution:"))
        .map(|(block, _)| block.to_string())
        .unwrap_or_else(|| panic!("the flattened manifest has no contents block: {manifest}"));
    for key in ["bundles", "assemblies", "migrations"] {
        assert!(
            !contents.contains(&format!("{key}:")),
            "the flattened `contents` declares `{key}`: {contents}"
        );
        assert!(
            !out.join(key).exists(),
            "the flattened artifact carries `{key}`, which the manifest does not declare"
        );
    }

    // `migrations` is the one the publisher is told about, because it is the one
    // nothing in the artifact represents. `bundles` is resolved into
    // `taxonomy.yml` and `assemblies` is recorded under
    // `distribution.derived_from`, so both are absorbed rather than lost.
    assert!(
        stderr.contains("`contents.migrations`"),
        "the publish dropped a declared payload and said nothing: {stderr}"
    );
    assert!(
        stderr.contains("has no reader here"),
        "the message does not say why the payload was dropped: {stderr}"
    );

    // The absorbed keys stay quiet. A line printed by every flattening publish —
    // and every assembly source declares `bundles`, or there would be no
    // assembly — is a line nobody reads on the publish that loses something.
    for key in ["bundles", "assemblies"] {
        assert!(
            !stderr.contains(&format!("`contents.{key}`")),
            "an absorbed key was reported as a loss: {stderr}"
        );
    }
}

/// The publish that loses nothing says nothing.
///
/// The companion to the case above, and the one that makes its signal worth
/// anything. This repository's own starter recipe is published from a source
/// declaring `bundles` and `assemblies` and no `migrations`, which is the
/// ordinary shape, and its standard error carries no drop notice at all.
#[test]
fn a_flattened_publish_that_loses_nothing_reports_no_drop() {
    let root = Root::scratch("assembly-no-drops");
    let source = assembly_source(&root, Arm::Sound);
    let out = root.path().join("release");

    let (code, _stdout, stderr) = publish_assembly_from(root.path(), &source, &out);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(
        !stderr.contains("nothing in the flattened package carries it"),
        "a publish that loses nothing printed a drop notice: {stderr}"
    );
}

/// The flattened artifact crosses the entire publisher-consumer boundary.
///
/// The source selection has two bundles, but the consumer names no bundles at
/// all. It pins the release digest before `taxonomy vendor` reads the fetched
/// directory, resolves the installed package, and compares the installed
/// taxonomy with the source assembly after identity is removed. This is the
/// batteries-included consumption form: its declarations are the composer's
/// declarations, but bundle choice is no longer part of the consumer's state.
#[test]
fn a_flattened_assembly_is_pinned_vendored_and_resolved_without_bundle_selection() {
    let root = Root::scratch("assembly-consumer");
    let source = assembly_source(&root, Arm::Sound);
    let artifact = root.path().join("release");
    let (code, _stdout, stderr) = publish_assembly_from(root.path(), &source, &artifact);
    assert_eq!(code, Some(0), "{stderr}");

    let release = headwater_resolve::release::read(
        &std::fs::read_to_string(artifact.join("release.yml")).expect("the release reads"),
    )
    .expect("the published record reads");
    let consumer = flattened_consumer(&root, &release.digest);
    let declaration = package::consumer(&consumer).expect("the consumer declaration reads");
    assert!(
        declaration.bundles.is_empty(),
        "a flattened consumer carries no bundle selection: {:?}",
        declaration.bundles
    );

    let (code, _stdout, stderr) = consumer_run(
        &consumer,
        &[
            "taxonomy",
            "vendor",
            artifact.to_str().expect("the artifact is UTF-8"),
        ],
    );
    assert_eq!(code, Some(0), "{stderr}");
    let installed = consumer.join("packages/acme-starter");
    assert!(
        installed.join("release.yml").is_file(),
        "the artifact was not vendored"
    );

    let (code, _stdout, stderr) = consumer_run(&consumer, &["taxonomy", "resolve"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(
        consumer.join(".headwater/taxonomy.lock").is_file(),
        "a resolved consumer has a lock"
    );

    let source_manifest = package::manifest_at(&source).expect("the source manifest reads");
    let recipe = assembly::read(root.path(), &source, &source_manifest, "starter")
        .expect("the source assembly reads");
    let source_resolution = assembly::resolve(root.path(), &source, &source_manifest, &recipe)
        .expect("the source assembly resolves");
    let installed_manifest =
        package::manifest_at(&installed).expect("the installed manifest reads");
    let installed_flattened = flatten::Flattened {
        manifest: installed_manifest,
        taxonomy: std::fs::read_to_string(installed.join("taxonomy.yml"))
            .expect("the installed taxonomy reads"),
        assets: Vec::new(),
        // `equivalent` compares taxonomy declarations and reads neither of
        // these. This value is read back off an installed artifact rather than
        // produced by a publish, so there is no run behind it whose dropped keys
        // this could name.
        dropped: Vec::new(),
    };
    assert!(
        flatten::equivalent(&source_resolution, &installed_flattened, &recipe),
        "the flattened consumer differs from the source assembly after identity is removed"
    );
}

/// The recipe this repository ships, over the whole publisher-consumer boundary.
///
/// The two cases above prove the mechanism over `acme/fixture`, which is a
/// fixture written to make the boundary visible. This is the first case here to
/// run `--assembly` over the real maintained source, and its subject is the
/// artifact spec 0 deliverable 6 promises an adopter:
/// `taxonomy-source/headwater-standard/assemblies/starter/assembly.yml`,
/// published as `headwater/starter`, vendored into a tree that names no bundle,
/// and resolved to a lock.
///
/// It takes the identity and the version out of the recipe rather than writing
/// either one down, so a bump of the starter moves this case with it and never
/// past it.
///
/// **The overlay is the assertion, not scaffolding.** The one answer a
/// batteries-included consumer still owes is the identifier namespace, which no
/// package may hold because a constant a publisher wrote would be minted by
/// every adopter at once. So the consumer below declares two namespace lines and
/// nothing else, and a lock at the end of it is the measurement that the
/// flattened starter needs no other answer.
#[test]
fn the_shipped_starter_recipe_publishes_vendors_and_resolves() {
    let root = Root::scratch("shipped-starter");
    let source = repository().join("taxonomy-source/headwater-standard");
    let artifact = root.path().join("release");

    let source_manifest = package::manifest_at(&source).expect("the source manifest reads");
    let recipe = assembly::read(&repository(), &source, &source_manifest, "starter")
        .expect("the shipped starter recipe reads");
    assert_eq!(recipe.from.bundles.len(), 3, "{:?}", recipe.from.bundles);
    assert!(
        recipe.overlay.is_none(),
        "the shipped recipe declares an overlay, and `validate_glue` admits one only as a \
         relation between two selected bundles"
    );

    let (code, stdout, stderr) = publish_assembly_from(&repository(), &source, &artifact);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(
        stdout.contains(&format!("published {} {}", recipe.package, recipe.version)),
        "{stdout}"
    );
    let manifest =
        std::fs::read_to_string(artifact.join("package.yml")).expect("the manifest reads");
    assert!(
        manifest.contains(&format!("package: {}", recipe.package)),
        "{manifest}"
    );
    assert!(manifest.contains("form: flattened"), "{manifest}");
    assert!(
        !artifact.join("bundles").exists(),
        "a flattened package carries bundles"
    );
    assert!(
        artifact.join("doctrine/starter/starter.md").is_file(),
        "the doctrine this package declares did not reach the flattened artifact"
    );

    let release = headwater_resolve::release::read(
        &std::fs::read_to_string(artifact.join("release.yml")).expect("the release reads"),
    )
    .expect("the published record reads");
    let consumer = root.path().join("consumer");
    write(
        &consumer.join(".headwater/taxonomy.yml"),
        &format!(
            "taxonomy:\n  package: {}\n  version: {}\n  digest: {}\n  overlay: .headwater/overlay.yml\ncorpus:\n  root: docs\n",
            recipe.package, recipe.version, release.digest
        ),
    );
    write(
        &consumer.join(".headwater/overlay.yml"),
        "add:\n  identifier_schemes.decision_id.namespace: ACME\n  identifier_schemes.obligation_record_id.namespace: ACME\n",
    );
    std::fs::create_dir_all(consumer.join("docs")).expect("the corpus root is made");

    let declaration = package::consumer(&consumer).expect("the consumer declaration reads");
    assert!(
        declaration.bundles.is_empty(),
        "a flattened consumer carries no bundle selection: {:?}",
        declaration.bundles
    );

    let (code, _stdout, stderr) = consumer_run(
        &consumer,
        &[
            "taxonomy",
            "vendor",
            artifact.to_str().expect("the artifact is UTF-8"),
        ],
    );
    assert_eq!(code, Some(0), "{stderr}");
    assert!(
        consumer
            .join("packages/headwater-starter/doctrine/starter/starter.md")
            .is_file(),
        "the doctrine did not arrive with the vendored package"
    );

    let (code, _stdout, stderr) = consumer_run(&consumer, &["taxonomy", "resolve"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(
        consumer.join(".headwater/taxonomy.lock").is_file(),
        "a resolved flattened starter has a lock"
    );

    // The verb that reads `contents.conformance`, run against the vendored
    // flattened package rather than against the lock. #581 is the reason it is
    // here: `vendor` and `resolve` both exited 0 over an artifact whose manifest
    // named a rule set it did not carry, and this is the first reader that opens
    // the path. `headwater conformance` also verifies the installed bytes
    // against the release record, so hand-placing the file cannot satisfy it.
    assert!(
        consumer
            .join("packages/headwater-starter/conformance.yml")
            .is_file(),
        "the conformance rule set did not arrive with the vendored package"
    );
    let (code, stdout, stderr) = consumer_run(&consumer, &["conformance"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(stdout.contains("L1"), "{stdout}");
}

/// **Which verb an adopter runs decides whether a hand edit to a vendored
/// package is found.** `headwater conformance --level L0` recomputes the digest
/// of every installed member and refuses; `headwater taxonomy resolve` exits 0
/// over the same tree, because it compares the version the manifest declares
/// against the version the consumer pinned and reads no digest at all
/// (`headwater_resolve::package::sources`).
///
/// This is [#517](https://github.com/headwater-ai/headwater/issues/517)'s only
/// surviving complaint. The recheck it asked for landed in `c09625a` on
/// 2026-08-25, eleven days before the issue was filed, so what remains is the
/// pairing: the recheck exists and the verb an adopter is most likely to put in
/// a build does not perform it.
///
/// The conformance crate holds the same pairing at the library level. This case
/// is the CLI half of it, because an exit status is what a build reads, and a
/// library call that returns `Ok` is not the same claim as a process that
/// returns 0.
///
/// **The order is the assertion.** Publish, then pin, then vendor, then resolve,
/// and the hand edit lands after the publish that computed the pinned digest. A
/// resolve before the publish would move the sources with the record and both
/// sides of the comparison would agree.
///
/// The edited member is a doctrine file, which is a member of the release and
/// names no path in the lock's `sources:` list. That is the same shape as
/// `conformance.yml` in this repository's own package, and it is the shape that
/// no other gate covers.
///
/// **Two arms, because the two numbers fail differently.** A pin that no longer
/// names the installed bytes is a gap in a reading, so a plain run exits 0 with
/// `pin.current` in its report and `--level L0` is what turns that into a
/// non-zero exit. Installed bytes that no longer match their own record refuse
/// ahead of every reading and at every level, because a rule set read out of a
/// diverged package is a rule set the run cannot trust. Only the first arm is
/// the one `--level` decides, and a sentence that credited `--level` with both
/// would be wrong about the second.
#[test]
fn a_pin_that_no_longer_names_the_installed_bytes_is_refused_by_conformance_and_not_by_resolve() {
    let root = Root::scratch("pin-recheck-pairing");
    let source = repository().join("taxonomy-source/headwater-standard");
    let artifact = root.path().join("release");

    let (code, _stdout, stderr) = publish_assembly_from(&repository(), &source, &artifact);
    assert_eq!(code, Some(0), "{stderr}");
    let release = headwater_resolve::release::read(
        &std::fs::read_to_string(artifact.join("release.yml")).expect("the release reads"),
    )
    .expect("the published record reads");

    let source_manifest = package::manifest_at(&source).expect("the source manifest reads");
    let recipe = assembly::read(&repository(), &source, &source_manifest, "starter")
        .expect("the shipped starter recipe reads");
    let consumer = root.path().join("consumer");
    write(
        &consumer.join(".headwater/taxonomy.yml"),
        &format!(
            "taxonomy:\n  package: {}\n  version: {}\n  digest: {}\n  overlay: .headwater/overlay.yml\ncorpus:\n  root: docs\n",
            recipe.package, recipe.version, release.digest
        ),
    );
    write(
        &consumer.join(".headwater/overlay.yml"),
        "add:\n  identifier_schemes.decision_id.namespace: ACME\n  identifier_schemes.obligation_record_id.namespace: ACME\n",
    );
    std::fs::create_dir_all(consumer.join("docs")).expect("the corpus root is made");

    let (code, _stdout, stderr) = consumer_run(
        &consumer,
        &[
            "taxonomy",
            "vendor",
            artifact.to_str().expect("the artifact is UTF-8"),
        ],
    );
    assert_eq!(code, Some(0), "{stderr}");

    // Both verbs agree over the tree `vendor` wrote. Without this, a later
    // non-zero could be anything the fixture got wrong.
    let (code, _stdout, stderr) = consumer_run(&consumer, &["taxonomy", "resolve"]);
    assert_eq!(code, Some(0), "{stderr}");
    let (code, _stdout, stderr) = consumer_run(&consumer, &["conformance", "--level", "L0"]);
    assert_eq!(
        code,
        Some(0),
        "the freshly vendored tree does not reach L0: {stderr}"
    );

    // ---- Arm one: the pin moved and the bytes did not. ------------------
    //
    // `pin.current` reports a gap, so `--level L0` is what turns the report
    // into a non-zero exit and a plain run still exits 0 with the gap in the
    // report. The resolver exits 0 over the same tree because it never reads a
    // digest.
    let stale = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
    let authored = consumer.join(".headwater/taxonomy.yml");
    let pin = std::fs::read_to_string(&authored).expect("the authored pin reads");
    write(&authored, &pin.replace(&release.digest, stale));

    let (code, _stdout, stderr) = consumer_run(&consumer, &["taxonomy", "resolve"]);
    assert_eq!(
        code,
        Some(0),
        "the resolver read the pinned digest, so this pairing no longer holds: {stderr}"
    );
    let (code, stdout, stderr) = consumer_run(&consumer, &["conformance"]);
    assert_eq!(code, Some(0), "a plain run moved its exit status: {stderr}");
    assert!(
        stdout.contains("pin.current"),
        "the report does not name the reading that found the stale pin: {stdout}"
    );
    let (code, stdout, stderr) = consumer_run(&consumer, &["conformance", "--level", "L0"]);
    assert_ne!(code, Some(0), "a stale pin reached L0: {stdout}\n{stderr}");
    assert!(
        format!("{stdout}\n{stderr}").contains("pin.current"),
        "the refusal does not name the reading: {stdout}\n{stderr}"
    );

    // ---- Arm two: the bytes moved and the pin did not. -------------------
    //
    // The hand edit lands after the publish that computed the pinned digest and
    // after the vendor that installed the bytes it names.
    write(&authored, &pin);
    let installed = consumer.join("packages/headwater-starter/doctrine/starter/starter.md");
    let carried = std::fs::read_to_string(&installed).expect("the installed doctrine reads");
    std::fs::write(
        &installed,
        format!("{carried}\n<!-- hand-edited after vendor, never through `vendor` -->\n"),
    )
    .expect("the hand edit writes");

    let (code, _stdout, stderr) = consumer_run(&consumer, &["taxonomy", "resolve"]);
    assert_eq!(
        code,
        Some(0),
        "the resolver refused the edited tree, so this pairing no longer holds: {stderr}"
    );

    // Moved bytes are refused ahead of every reading and at every level: a rule
    // set read out of a package whose bytes no longer match its record is a rule
    // set nothing can trust, so the run ends rather than reports. `--level` is
    // therefore not what makes this one non-zero, which is the difference
    // between this arm and the one above.
    for arguments in [
        vec!["conformance"],
        vec!["conformance", "--level", "L0"],
        vec!["conformance", "--level", "L1"],
    ] {
        let (code, stdout, stderr) = consumer_run(&consumer, &arguments);
        assert_eq!(
            code,
            Some(1),
            "an edited member passed `{arguments:?}`: {stdout}\n{stderr}"
        );
        let said = format!("{stdout}\n{stderr}");
        assert!(
            said.contains("doctrine/starter/starter.md"),
            "the refusal does not name the member that moved: {said}"
        );
        assert!(
            said.contains("no longer match"),
            "the refusal does not say the bytes moved: {said}"
        );
    }
}

#[test]
fn an_invalid_assembly_refuses_before_it_creates_output() {
    let root = Root::scratch("assembly-refuses");
    let source = assembly_source(&root, Arm::UnselectableBundle);
    let out = root.path().join("release");

    let (code, _stdout, stderr) = publish_assembly_from(root.path(), &source, &out);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("`missing`"), "{stderr}");
    assert!(stderr.contains("nothing was published"), "{stderr}");
    assert!(
        !out.exists(),
        "the refused run left partial output at {out:?}"
    );
}

/// A recipe whose selection leaves a name dangling is refused at the publish.
///
/// This is [#582](https://github.com/headwater-ai/headwater/issues/582), and
/// before it the same run exited 0 and printed `published acme/starter 2.0.0`.
/// The artifact it wrote was refused at the consumer's own `taxonomy resolve`,
/// so the publisher was told the handoff was good and the adopter found out it
/// was not.
///
/// **The case lives here rather than in `resolve/tests/publish.rs`, which is
/// where #582's own second clause puts it.** The assembly fixtures are here:
/// `assembly_source` already carries a source package with a recipe and a
/// broken/working boolean, and `an_invalid_assembly_refuses_before_it_creates
/// _output` above already asserts the exit status, the message and the disk over
/// it. A third assembly fixture in the other file would be a second enumeration
/// of one thing. The plain publish path has its own case, over there, beside the
/// widest-set refusal it belongs to.
///
/// Four assertions. The exit status is the one that moved. The recipe path and
/// the selection are what tell a publisher which line to edit. `beta` is the
/// name the omission left dangling, and it is also the bundle the advice names,
/// which is [#579](https://github.com/headwater-ai/headwater/issues/579)'s
/// recipe half arriving here. `!out.exists()` is the same disk claim every
/// refusal on this path makes.
#[test]
fn an_assembly_whose_selection_omits_a_bundle_another_bundle_needs_does_not_publish() {
    let root = Root::scratch("assembly-incomplete");
    let source = assembly_source(&root, Arm::IncompleteSelection);
    let out = root.path().join("release");

    let (code, _stdout, stderr) = publish_assembly_from(root.path(), &source, &out);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("nothing was published"), "{stderr}");
    assert!(
        stderr.contains("assembly.yml"),
        "the refusal names the recipe file to edit: {stderr}"
    );
    assert!(
        stderr.contains("[alpha]"),
        "the refusal names the selection: {stderr}"
    );
    assert!(
        stderr.contains("`beta`"),
        "the refusal names the dangling name: {stderr}"
    );
    assert!(
        stderr.contains("shelves.betas.kind"),
        "the refusal names the address that reads it: {stderr}"
    );
    // #579's recipe half. The consumer path already names the bundle to add;
    // a publisher who narrows a recipe is in the same position and, before
    // this, was told nothing at all.
    assert!(
        stderr.contains("`beta` declares 1 of the 1"),
        "the advice names the bundle to add back: {stderr}"
    );
    assert!(
        stderr.contains("`from.bundles:`"),
        "the advice names the key a publisher edits, not the consumer's: {stderr}"
    );
    assert!(
        !stderr.contains(".headwater/taxonomy.yml"),
        "the advice sends a publisher to the recipe and never to a consumer declaration: {stderr}"
    );
    assert!(
        !out.exists(),
        "the refused run left partial output at {out:?}"
    );
}

/// The same recipe with the selection complete still publishes.
///
/// The negative half, and it matters more than the positive: `publish` is the
/// verb a stranger runs to hand an artifact to somebody else, and a false
/// refusal stops a publisher shipping. `assembly_source`'s sound arm ships a
/// taxonomy that shares no name with `headwater/standard`, so this says the new
/// check admits a library nothing like this repository's.
///
/// `the_shipped_starter_recipe_publishes_vendors_and_resolves` is the other
/// half, over the real maintained source and the recipe an adopter is invited to
/// copy.
#[test]
fn an_assembly_whose_selection_is_complete_still_publishes() {
    let root = Root::scratch("assembly-complete");
    let source = assembly_source(&root, Arm::Sound);
    let out = root.path().join("release");

    let (code, stdout, stderr) = publish_assembly_from(root.path(), &source, &out);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(stdout.contains("published acme/starter 2.0.0"), "{stdout}");
}

/// `nothing was published` and the disk agree, and the second run is the proof.
///
/// Three assertions, and the second is the one no wording change satisfies. The
/// third is what an adopter met: before this, the second run said *the output
/// directory holds files already*, so the person had to delete a directory the
/// tool had told them it never wrote to.
#[test]
fn a_publish_that_says_nothing_was_published_wrote_nothing() {
    let root = Root::copied("says-nothing");
    let out = root.path().join("release");

    let (code, first) = root.publish(&out);
    assert_eq!(code, Some(1), "{first}");
    assert!(
        first.contains("headwater: nothing was published"),
        "{first}"
    );
    assert!(
        first.contains("package.yml"),
        "the refusal does not name the manifest that declares the path: {first}"
    );
    assert!(
        first.contains("../../docs/taxonomies"),
        "the refusal does not name the declared value: {first}"
    );
    assert!(
        !out.exists(),
        "the run that published nothing left {:?}",
        std::fs::read_dir(&out).map(|entries| entries
            .filter_map(Result::ok)
            .map(|entry| entry.file_name())
            .collect::<Vec<_>>())
    );

    let (code, again) = root.publish(&out);
    assert_eq!(code, Some(1), "{again}");
    assert_eq!(
        again, first,
        "the second run met a different refusal, so the first left something behind"
    );
}

/// A mode, set on a path.
#[cfg(unix)]
fn mode(at: &Path, bits: u32) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(at, std::fs::Permissions::from_mode(bits)).expect("the mode is set");
}

/// Whether a mode can stop this process. See the note of the same name in
/// `headwater-resolve/tests/publish.rs`, and `engine/README.md` on why the
/// container half of the toolchain runs with `--user`.
#[cfg(unix)]
fn modes_hold(root: &Root) -> bool {
    let at = root.path().join("mode-probe");
    std::fs::create_dir_all(&at).expect("the probe is made");
    std::fs::write(at.join("held"), "x").expect("the probe holds a file");
    mode(&at, 0o300);
    let held = std::fs::read_dir(&at).is_err();
    mode(&at, 0o700);
    std::fs::remove_dir_all(&at).expect("the probe goes");
    held
}

/// `nothing was published` over a directory that could not be read, with a whole
/// artifact written into it.
///
/// This is the same claim as the case above and the state that reached it is the
/// narrower one. The root here is this repository, so the publish would
/// otherwise succeed and the write phase has every file of a real artifact to
/// put somewhere. `--out` is a directory holding a person's file at mode `0300`:
/// no `r`, so the precondition could not see the file, and `w` and `x`, so every
/// write below it lands. Read as absence, that published the artifact beside
/// their file, failed reading the directory back, printed `nothing was
/// published`, and ran an undo whose one instruction is to remove `--out`.
///
/// A verb that says it wrote nothing and wrote 39 files is the defect the issue
/// is named for. This is that defect through a doorway the `--out` precondition
/// does not cover, so the assertion is the disk rather than the wording.
#[cfg(unix)]
#[test]
fn a_publish_into_a_directory_it_cannot_read_writes_nothing_into_it() {
    let holder = Root::scratch("unreadable-out");
    if !modes_hold(&holder) {
        eprintln!("skipped: this process is root, and root reads through mode 0300");
        return;
    }
    let out = holder.path().join("release");
    std::fs::create_dir_all(&out).expect("the caller's directory is made");
    std::fs::write(out.join("theirs.txt"), "the caller's own file").expect("their file is written");
    mode(&out, 0o300);

    let (code, message) = publish_real_source_into(&out);
    mode(&out, 0o700);

    let left: Vec<String> = std::fs::read_dir(&out)
        .expect("it reads once the mode is back")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        left,
        vec!["theirs.txt".to_string()],
        "the run published into a directory it could not read: {message}"
    );
    assert_eq!(code, Some(1), "{message}");
    assert!(
        message.contains("the output directory cannot be read"),
        "the refusal does not say why the directory was refused: {message}"
    );
}

/// An empty `--out` that the caller made is emptied again, not removed.
///
/// The precondition takes an empty directory as well as an absent one, so the
/// undo has two states to return to and this is the second. A person who ran
/// `mkdir release` first gets their directory back, empty.
///
/// **The root here is the colliding one and it used to be the copied one.** With
/// the copied root this case failed in phase 1, so `--out` was never written to
/// and the assertion was that an untouched empty directory is empty: it held
/// with the undo deleted. The root below reaches the write phase, so what is
/// removed between the failure and these lines is a part of an artifact that was
/// really on disk.
#[test]
fn an_output_directory_the_caller_made_is_left_empty_rather_than_removed() {
    let root = Root::colliding("caller-made");
    let out = root.path().join("release");
    std::fs::create_dir_all(&out).expect("the caller makes it");

    let (code, message) = root.publish(&out);
    assert_eq!(code, Some(1), "{message}");
    assert!(
        message.contains("headwater: nothing was published"),
        "{message}"
    );
    assert!(out.is_dir(), "the caller's own directory was removed");
    assert_eq!(
        std::fs::read_dir(&out)
            .expect("it reads")
            .filter_map(Result::ok)
            .count(),
        0,
        "the directory the caller made is not empty again: {message}"
    );
}

/// A write that fails leaves neither `--out` nor the directories the run made to
/// reach it, and the line a person reads is true of the disk.
///
/// `--out` is three levels below a directory that is not there either. `put`
/// reaches it with `create_dir_all`, so before this the verb printed `nothing
/// was published` and left three directories it had made standing.
#[test]
fn a_failed_write_leaves_neither_the_output_directory_nor_the_path_to_it() {
    let root = Root::colliding("nested-out");
    let nested = root.path().join("nested");
    let out = nested.join("a/b/c");

    let (code, message) = root.publish(&out);
    assert_eq!(code, Some(1), "{message}");
    assert!(
        message.contains("headwater: nothing was published"),
        "{message}"
    );
    assert!(
        !out.exists(),
        "the output directory is still there: {message}"
    );
    assert!(
        !nested.exists(),
        "the run that published nothing left the directories it made: {message}"
    );
}

/// #336's `--from <dir>`, exercised as a person would type it: two flags that
/// name the same thing are refused together, and `--from` alone publishes a
/// directory `--package` would never find under `packages/`.
///
/// The directory this hands `--from` is `taxonomy-source/headwater-standard/`
/// itself, in this repository's own tree rather than a copy of it, precisely
/// because the point of the flag is that it reads a manifest `find` would never
/// walk to (it does not sit under `packages/` at all). A copy would test the
/// read and hide the one thing worth proving: this path bypasses the lookup by
/// name entirely.
#[test]
fn from_and_package_together_are_refused_and_from_alone_publishes_the_relocated_source() {
    let source = repository().join("taxonomy-source/headwater-standard");
    let out_root = Root::scratch("from-flag");
    let out = out_root.path().join("release");

    let both = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["taxonomy", "publish", "--package", "headwater/standard"])
        .arg("--from")
        .arg(&source)
        .arg("--out")
        .arg(&out)
        .arg("--root")
        .arg(repository())
        .output()
        .expect("the binary runs");
    assert_eq!(both.status.code(), Some(1));
    let both_err = String::from_utf8_lossy(&both.stderr);
    assert!(
        both_err.contains("--package") && both_err.contains("--from"),
        "the refusal does not name both flags: {both_err}"
    );
    assert!(!out.exists(), "the refused run wrote into --out anyway");

    let from_alone = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["taxonomy", "publish"])
        .arg("--from")
        .arg(&source)
        .arg("--out")
        .arg(&out)
        .arg("--root")
        .arg(repository())
        .output()
        .expect("the binary runs");
    let stdout = String::from_utf8_lossy(&from_alone.stdout).into_owned();
    assert_eq!(
        from_alone.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&from_alone.stderr)
    );
    assert!(
        stdout.contains("published headwater/standard"),
        "the run did not report a publish: {stdout}"
    );
    assert!(
        out.join("release.yml").is_file(),
        "publishing `--from` a directory outside `packages/` wrote no release record"
    );
}

/// The vendored bundles this repository ships agree with a fresh publish of
/// its own maintained source, file for file and byte for byte.
///
/// #407: `taxonomy-source/headwater-standard/package.yml` declares `bundles:
/// ../../docs/taxonomies`, and `headwater taxonomy publish` copies that
/// directory into `bundles/` inside the artifact `taxonomy vendor` later
/// installs at `packages/headwater-standard/bundles/`. Nothing re-takes that
/// snapshot or compares it against the source it was taken from, so six of
/// seventy-two files drifted before anyone noticed: every rule that reads
/// this repository's corpus excludes `docs/taxonomies/**` (#350), and every
/// rule that reads the lock reads `bundle.yml`, never `doctrine.md` or a
/// fixtures `README.md`.
///
/// This is the comparison the issue's Done-when names, run against the real
/// publisher rather than a description of it: publish this repository's own
/// maintained source with [`publish_real_source_into`], the same call
/// [`from_and_package_together_are_refused_and_from_alone_publishes_the_relocated_source`]
/// makes, and diff the `bundles/` that publish just wrote against
/// `packages/headwater-standard/bundles/`, the copy this repository ships. A
/// path present on one side and not the other, or a path whose bytes differ,
/// is exactly the drift #407 found by grepping for two deleted paths by
/// hand.
#[test]
fn the_vendored_bundles_agree_with_a_fresh_publish_of_the_maintained_source() {
    let root = Root::scratch("vendored-bundles");
    let out = root.path().join("release");

    let (code, message) = publish_real_source_into(&out);
    assert_eq!(
        code,
        Some(0),
        "the publish this comparison depends on failed: {message}"
    );

    let fresh = out.join("bundles");
    let vendored = repository().join("packages/headwater-standard/bundles");

    let mut fresh_paths = relative_files(&fresh);
    let mut vendored_paths = relative_files(&vendored);
    fresh_paths.sort();
    vendored_paths.sort();

    let only_in_fresh: Vec<&String> = fresh_paths
        .iter()
        .filter(|p| !vendored_paths.contains(p))
        .collect();
    let only_in_vendored: Vec<&String> = vendored_paths
        .iter()
        .filter(|p| !fresh_paths.contains(p))
        .collect();
    let differing: Vec<&String> = fresh_paths
        .iter()
        .filter(|p| vendored_paths.contains(p))
        .filter(|p| {
            std::fs::read(fresh.join(p)).expect("the fresh file reads")
                != std::fs::read(vendored.join(p)).expect("the vendored file reads")
        })
        .collect();

    assert!(
        only_in_fresh.is_empty() && only_in_vendored.is_empty() && differing.is_empty(),
        "packages/headwater-standard/bundles/ has drifted from docs/taxonomies/, the source \
         taxonomy-source/headwater-standard/package.yml declares.\n\
         only in a fresh publish, missing from the vendored copy: {only_in_fresh:?}\n\
         only in the vendored copy, missing from a fresh publish: {only_in_vendored:?}\n\
         present on both sides with different bytes: {differing:?}\n\
         Republish and revendor: `headwater taxonomy publish --from \
         taxonomy-source/headwater-standard --out <dir> && headwater taxonomy vendor <dir>`"
    );
}

/// #518: an artifact carries no bundle reference corpus, and the publisher
/// keeps every one of its own.
///
/// Spec 7's Publishing section rules that publication takes the package
/// directory whole, and it names one exception: a `fixtures/` directory at the
/// root of a bundle. Those corpora are how a publisher measures its own
/// taxonomy against prose it controls. No consumer verb opens one, and before
/// this exception they were 71 of the 102 members of this repository's own
/// release record.
///
/// This runs the real publisher over the maintained source rather than a
/// description of it, and it holds the publisher's side as well, because the
/// exception is about what the artifact carries and it must delete nothing.
#[test]
fn a_fresh_publish_carries_no_bundle_fixtures_and_the_publisher_keeps_its_own() {
    let root = Root::scratch("bundle-fixtures");
    let out = root.path().join("release");

    let (code, message) = publish_real_source_into(&out);
    assert_eq!(
        code,
        Some(0),
        "the publish this case depends on failed: {message}"
    );

    let mut carried: Vec<String> = relative_files(&out.join("bundles"))
        .into_iter()
        .filter(|path| path.split('/').nth(1) == Some("fixtures"))
        .collect();
    carried.sort();
    assert!(
        carried.is_empty(),
        "the artifact carries {} bundle fixture files that no consumer verb opens: {carried:#?}",
        carried.len()
    );

    assert!(
        repository()
            .join("docs/taxonomies/standards-spec/fixtures/README.md")
            .is_file(),
        "the exception is about the artifact. The publisher's own reference corpora stay where \
         they are"
    );
}

/// #518: no file the artifact carries links at a path inside the bundle tree
/// that the artifact does not carry.
///
/// A publish decides the shape of `bundles/`, and this case is the reason that
/// decision is not free. Dropping each bundle's own corpus left fifteen links
/// across seven carried files pointing at `fixtures/README.md`,
/// `fixtures/n8n/README.md` and `../<bundle>/fixtures/README.md`. Every one of
/// them resolved in the publisher's tree and in the artifact before, and none
/// resolved in the artifact after. **Nothing else in this repository can see
/// that.** The referential integrity a publish runs reads `contents` key
/// scalars and never a carried file's body, `release::verify` compares the
/// record against the bytes and reads no body either, and `docs/taxonomies/**`
/// is outside the corpus root, so `link.fragment.unresolved` never opens the
/// sources.
///
/// **The population is enumerated and never listed.** Every `.md` file in the
/// artifact, every inline link in it, and every target that lands inside
/// `bundles/` after the `..` segments are resolved. A link that climbs out of
/// the artifact is passed over here, because this case asks only about the
/// bundle tree a publish reshapes. It is no longer passed over anywhere:
/// #633 ruled that a doctrine reference to a document the artifact does not
/// carry is written as an absolute URL, and
/// `every_relative_link_a_carried_file_writes_resolves_inside_the_artifact`
/// below judges both classes — the target inside `bundles/` that is missing,
/// and the target that climbs clear of the artifact root.
///
/// A code span and a fenced block are read past, because this repository
/// prints a link as an example inside both, and a check that reddens on correct
/// Markdown is a check the first person it annoys turns off.
#[test]
fn no_file_the_artifact_carries_links_into_a_bundle_tree_it_does_not_carry() {
    let root = Root::scratch("bundle-links");
    let out = root.path().join("release");

    let (code, message) = publish_real_source_into(&out);
    assert_eq!(
        code,
        Some(0),
        "the publish this case depends on failed: {message}"
    );

    let mut read = 0usize;
    let mut dangling: Vec<String> = Vec::new();
    for member in relative_files(&out) {
        if !member.ends_with(".md") {
            continue;
        }
        let text = std::fs::read_to_string(out.join(&member)).expect("a member reads");
        let directory = Path::new(&member)
            .parent()
            .unwrap_or(Path::new(""))
            .to_owned();
        for target in markdown_links(&text) {
            let body = target.split('#').next().unwrap_or_default();
            if body.is_empty() || body.contains("://") {
                continue;
            }
            let Some(at) = inside_the_artifact(&directory, body) else {
                continue;
            };
            if !at.starts_with("bundles") {
                continue;
            }
            read += 1;
            if !out.join(&at).exists() {
                dangling.push(format!("{member} -> {target}"));
            }
        }
    }

    // A scanner that reads nothing passes everything, and this one reads the
    // real library rather than a fixture, so it says how much it saw.
    assert!(
        read > 0,
        "the scanner found no link into the bundle tree at all, so it held nothing"
    );
    assert!(
        dangling.is_empty(),
        "{} of the {read} links into the bundle tree do not resolve inside the artifact. Every \
         one of them resolves in `docs/taxonomies/`, so the publish carried the prose and left \
         the target behind:\n{dangling:#?}",
        dangling.len()
    );
}

/// #633: every relative link a carried `.md` file writes resolves to a file the
/// artifact carries, so a consumer who vendors this package receives no dead
/// relative link at all.
///
/// This is the bound the ruling on #633 needs, and it is wider than the two
/// cases around it. The ruling says a doctrine reference to a document the
/// artifact does not carry is written as an absolute URL. An absolute URL is
/// read past here, so the only relative link that reaches the assertions is one
/// the artifact is expected to resolve.
///
/// **Two classes, and each returns by a different route.** A target that lands
/// inside `bundles/` and is missing is the class the sibling case above already
/// holds. A target that climbs clear of the artifact root is the class nothing
/// read before this case: `../../spec/07-…` written from
/// `bundles/<name>/doctrine.md` normalizes past the artifact root, so the #619
/// publish rule never sees it (the target is not inside the artifact),
/// `link.fragment.unresolved` never opens the file (`docs/taxonomies/**` is
/// outside the corpus root), and `inside_the_artifact` returns `None` for it,
/// which every other walker in this file reads as "skip". It measures 0 today
/// and it is asserted as a bound rather than recorded as a count, because the
/// way the defect this issue repairs comes back is one doctrine author writing
/// one `../../` link that nothing else in this repository can see.
///
/// A `bundle.yml` is out of scope here on purpose. Three of the fifty-two pairs
/// #633 repaired were written in a YAML comment, and
/// `the_real_package_records_exactly_the_references_it_carries` below walks
/// every member rather than the `.md` ones, so the record holds those.
#[test]
fn every_relative_link_a_carried_file_writes_resolves_inside_the_artifact() {
    let root = Root::scratch("relative-links");
    let out = root.path().join("release");

    let (code, message) = publish_real_source_into(&out);
    assert_eq!(
        code,
        Some(0),
        "the publish this case depends on failed: {message}"
    );

    let mut read = 0usize;
    let mut dangling: Vec<String> = Vec::new();
    let mut escaping: Vec<String> = Vec::new();
    for member in relative_files(&out) {
        if !member.ends_with(".md") {
            continue;
        }
        let text = std::fs::read_to_string(out.join(&member)).expect("a member reads");
        let directory = Path::new(&member)
            .parent()
            .unwrap_or(Path::new(""))
            .to_owned();
        for target in markdown_links(&text) {
            let body = target.split('#').next().unwrap_or_default();
            if body.is_empty() || body.contains("://") || body.starts_with("mailto:") {
                continue;
            }
            read += 1;
            let Some(at) = inside_the_artifact(&directory, body) else {
                escaping.push(format!("{member} -> {target}"));
                continue;
            };
            if !out.join(&at).exists() {
                dangling.push(format!("{member} -> {target}"));
            }
        }
    }

    // A scanner that reads nothing passes everything, and this one reads the
    // real library rather than a fixture, so it says how much it saw.
    assert!(
        read > 0,
        "the scanner found no relative link in any carried file, so it held nothing"
    );
    assert!(
        escaping.is_empty(),
        "{} of the {read} relative links a carried file writes climb clear of the artifact root. \
         #633 ruled that a reference to a document the artifact does not carry is written as an \
         absolute URL, and nothing else in this repository reads these:\n{escaping:#?}",
        escaping.len()
    );
    assert!(
        dangling.is_empty(),
        "{} of the {read} relative links a carried file writes name a path the artifact does not \
         carry:\n{dangling:#?}",
        dangling.len()
    );
}

/// #619: the real package publishes, and the population its manifest records is
/// exactly the population the artifact carries.
///
/// Two clauses of the issue meet in one case. The first is that `headwater
/// taxonomy publish` still exits 0 on this repository's own package, proved on
/// the maintained source rather than on a fixture, because the bar as filed
/// refused it: 122 references over 51 pairs, none of them resolvable in any
/// artifact this project has published. The second is that the record which
/// admits them is held to the artifact in both directions.
///
/// **Both directions, and each one catches a different mistake.** A recorded
/// pair the artifact does not dangle is a line somebody repaired and forgot to
/// delete, and a record that outlives its population is a record that quietly
/// admits a reference nobody looked at. A dangling pair the record does not hold
/// cannot reach here at all — publish refuses it — so that half of the equality
/// is a statement that the engine and this walker agree about what dangles,
/// which is the reason this walker is written here rather than called out of the
/// crate under test.
///
/// The walker reads every member and not only the `.md` ones, because two
/// `bundle.yml` files in this library write a link inside a comment.
#[test]
fn the_real_package_records_exactly_the_references_it_carries() {
    let root = Root::scratch("recorded-references");
    let out = root.path().join("release");

    let (code, message) = publish_real_source_into(&out);
    assert_eq!(
        code,
        Some(0),
        "the maintained source no longer publishes, which is the clause this case holds: {message}"
    );

    let members: Vec<String> = relative_files(&out);
    let mut dangling: Vec<(String, String)> = Vec::new();
    let mut read = 0usize;
    for member in &members {
        let Ok(text) = std::fs::read_to_string(out.join(member)) else {
            continue;
        };
        let directory = Path::new(member)
            .parent()
            .unwrap_or(Path::new(""))
            .to_owned();
        for written in markdown_links(&text) {
            let target = written.split('#').next().unwrap_or_default();
            if target.is_empty() || target.contains("://") || target.starts_with("mailto:") {
                continue;
            }
            let Some(at) = inside_the_artifact(&directory, target) else {
                continue;
            };
            read += 1;
            if !out.join(&at).exists() {
                dangling.push((member.clone(), at.display().to_string()));
            }
        }
    }
    dangling.sort();
    dangling.dedup();

    assert!(
        read > 0,
        "the walker resolved no reference at all, so it held nothing"
    );

    let recorded = recorded_references(&out.join("package.yml"));
    assert_eq!(
        recorded,
        dangling,
        "the manifest records {} pairs and the artifact carries {} of {read} resolved references \
         that resolve nowhere. A pair on the left and not on the right is a repair nobody deleted \
         the record of; a pair on the right and not on the left could not have published at all",
        recorded.len(),
        dangling.len()
    );

    // The publish said so on standard error, and the count it printed is the
    // one this walker derived. `dropped` is the other line that reaches here
    // and it names a `contents` key, so a substring of the count alone would
    // pass on the wrong line.
    //
    // #633 emptied the record, and an empty record prints no line at all rather
    // than a line reporting zero. So the assertion is on the account either way:
    // a population is reported when there is one, and nothing is said when there
    // is none. Reading only the first half here would let the account disappear
    // from a run that still ships a population.
    let account = "references that resolve nowhere inside it";
    if recorded.is_empty() {
        assert!(
            !message.contains(account),
            "the manifest records nothing and the publish still reported a population: {message}"
        );
    } else {
        assert!(
            message.contains(&format!(
                "the artifact records {} {account}",
                recorded.len()
            )),
            "the publish did not report the population it shipped: {message}"
        );
    }
}

/// The `(member, target)` pairs a manifest records under
/// `unresolved_references`, sorted.
///
/// Read by hand rather than through the engine's loader, so what this case
/// compares is the file as written and not the engine's reading of it. The block
/// is a mapping of sequences at a fixed indentation, which is the whole shape
/// the key takes.
fn recorded_references(manifest: &Path) -> Vec<(String, String)> {
    let text = std::fs::read_to_string(manifest).expect("the published manifest reads");
    let mut pairs = Vec::new();
    let mut member: Option<String> = None;
    let mut inside = false;
    for line in text.lines() {
        if line.starts_with("unresolved_references:") {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        match line.strip_prefix("    - ") {
            Some(target) => match &member {
                Some(at) => pairs.push((at.clone(), target.trim().to_string())),
                None => panic!("a recorded target with no member above it: {line}"),
            },
            None => match line.strip_prefix("  ").map(str::trim_end) {
                Some(key) if key.ends_with(':') && !key.starts_with(' ') => {
                    member = Some(key.trim_end_matches(':').to_string());
                }
                // Anything at the left margin ends the block.
                _ => break,
            },
        }
    }
    pairs.sort();
    pairs
}

/// #619: a flattened package inherits no record of another artifact's
/// references.
///
/// `flatten::manifest` copies every source key it does not replace, and
/// `unresolved_references` rode through on the first cut. The starter recipe
/// then shipped four members and a record of 51 pairs naming members like
/// `bundles/README.md` that its artifact does not hold, and the publish printed
/// the count of them. **That is a standing admission**: a recipe inherits
/// permission to dangle a reference it does not contain, and the first
/// assembly that writes a real one at a recorded path is admitted by it.
///
/// A flattened package takes a new member layout — `doctrine` lands at
/// `doctrine/<recipe>/` and the bundle tree is absorbed — so not one recorded
/// member of the source is carried, and the key is dropped rather than
/// filtered. What the recipe's own publish finds is what its own record would
/// hold, and today that is nothing at all.
#[test]
fn the_shipped_starter_recipe_inherits_no_record_of_another_artifacts_references() {
    let root = Root::scratch("starter-record");
    let source = repository().join("taxonomy-source/headwater-standard");
    let artifact = root.path().join("release");

    let (code, _stdout, stderr) = publish_assembly_from(&repository(), &source, &artifact);
    assert_eq!(code, Some(0), "{stderr}");

    let carried = std::fs::read_to_string(artifact.join("package.yml"))
        .expect("the flattened manifest reads");
    assert!(
        !carried.contains(headwater_resolve::package::RECORDED_REFERENCES),
        "the flattened manifest inherited a record written for another artifact:\n{carried}"
    );
    assert!(
        recorded_references(&artifact.join("package.yml")).is_empty(),
        "the flattened artifact records a pair it cannot be about"
    );
    assert!(
        !stderr.contains("resolve nowhere inside it"),
        "the publish reported a population this artifact does not carry: {stderr}"
    );

    // The source it was flattened from does record a population, so this case
    // is about the flattening and not about a record that is empty everywhere.
    let plain = root.path().join("plain");
    let (code, message) = publish_real_source_into(&plain);
    assert_eq!(code, Some(0), "{message}");
    assert!(
        !recorded_references(&plain.join("package.yml")).is_empty(),
        "the source package records nothing, so this case holds nothing"
    );
}

/// The destination of every inline Markdown link on a line that is not inside a
/// fenced block, with a code span removed first and a CommonMark link title
/// dropped.
fn markdown_links(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut fenced = false;
    for line in text.lines() {
        let opener = line.trim_start();
        if opener.starts_with("```") || opener.starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if fenced {
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

/// Where a relative link written in `directory` lands inside the artifact, or
/// `None` where it climbs out of it.
///
/// The resolution is lexical, because the artifact carries no symlink a publish
/// left dereferenced and the question is where the text points.
fn inside_the_artifact(directory: &Path, target: &str) -> Option<PathBuf> {
    let mut at = PathBuf::new();
    for part in directory.join(target).components() {
        match part {
            std::path::Component::ParentDir => {
                if !at.pop() {
                    return None;
                }
            }
            std::path::Component::CurDir => {}
            std::path::Component::Normal(name) => at.push(name),
            _ => return None,
        }
    }
    Some(at)
}

/// Every regular file under `root`, as a path relative to it, in no
/// particular order.
/// #353: the digest a publisher hands on is readable without a text search.
///
/// The prose run prints it inside a paragraph. `--json` writes one document
/// whose `digest` member is the same number, and the number both are held to is
/// the one in the record on disk, so this holds the document to the artifact
/// and not only to the other rendering. The member list is held to the record
/// the same way, and `version` names the document's own shape rather than the
/// engine, which is what `json.rs` asks of every document this binary writes.
#[test]
fn a_json_publish_carries_the_digest_a_consumer_pins() {
    let base = Path::new(env!("CARGO_TARGET_TMPDIR")).join("publish-json");
    let _ = std::fs::remove_dir_all(&base);
    let prose_out = base.join("prose");
    let json_out = base.join("json");

    let (code, prose, stderr) = publish_real_source(&prose_out, false);
    assert_eq!(code, Some(0), "{stderr}");
    let (code, document, stderr) = publish_real_source(&json_out, true);
    assert_eq!(code, Some(0), "{stderr}");
    // #619 put one account on this stream that a successful run makes, and
    // `dropped` above it has had the same shape since #580: both are things the
    // publisher asked for that the artifact represents differently, and both are
    // printed in either output mode so a `--json` caller is not the one reader
    // who never hears them. So the guard subtracts the account it expects rather
    // than being dropped — anything else here on a run that succeeded is still a
    // defect, and standard output is still one document.
    let unaccounted: Vec<&str> = stderr
        .lines()
        .filter(|line| !line.contains("references that resolve nowhere inside it"))
        .filter(|line| !line.contains(headwater_resolve::package::RECORDED_REFERENCES))
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert!(
        unaccounted.is_empty(),
        "a JSON run that succeeded accounts for nothing on standard error beyond the population \
         the artifact records: {unaccounted:#?}"
    );

    let record = std::fs::read_to_string(json_out.join("release.yml")).expect("the record reads");
    let record = headwater_resolve::release::read(&record).expect("the record parses");

    let value = headwater_yaml::load(&document)
        .unwrap_or_else(|errors| panic!("the document does not parse: {errors:?}\n{document}"))
        .value;
    let map = value.as_map().expect("the document is an object");
    let text = |key: &str| -> Option<String> {
        map.get(key)
            .and_then(|spanned| spanned.value.as_scalar())
            .map(headwater_yaml::core_schema::as_str)
            .map(str::to_string)
    };

    assert_eq!(
        text("version").as_deref(),
        Some(headwater_resolve::release::DOCUMENT),
        "the document names its own shape"
    );
    assert_eq!(text("digest").as_deref(), Some(record.digest.as_str()));
    assert_eq!(text("out").as_deref(), json_out.to_str());

    let package = map
        .get("package")
        .and_then(|spanned| spanned.value.as_map())
        .expect("a `package` member");
    let identity = |key: &str| -> Option<String> {
        package
            .get(key)
            .and_then(|spanned| spanned.value.as_scalar())
            .map(headwater_yaml::core_schema::as_str)
            .map(str::to_string)
    };
    assert_eq!(identity("name").as_deref(), Some(record.package.as_str()));
    assert_eq!(
        identity("version").as_deref(),
        Some(record.version.as_str())
    );
    assert_eq!(identity("requires_engine"), record.requires_engine);

    let members = map
        .get("members")
        .and_then(|spanned| spanned.value.as_seq())
        .expect("a `members` member");
    assert_eq!(members.len(), record.members.len());
    for (written, recorded) in members.iter().zip(&record.members) {
        let member = written.value.as_map().expect("a member is an object");
        let field = |key: &str| -> Option<&str> {
            member
                .get(key)
                .and_then(|spanned| spanned.value.as_scalar())
                .map(headwater_yaml::core_schema::as_str)
        };
        assert_eq!(field("path"), Some(recorded.path.as_str()));
        assert_eq!(field("digest"), Some(recorded.digest.as_str()));
    }

    // The prose run and the JSON run published one artifact twice, and the
    // paragraph a person reads carries the number the document carries.
    assert!(prose.contains(&record.digest), "{prose}");
}

fn relative_files(root: &Path) -> Vec<String> {
    fn walk(base: &Path, dir: &Path, into: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).expect("the directory reads") {
            let entry = entry.expect("the entry reads").path();
            if entry.is_dir() {
                walk(base, &entry, into);
            } else {
                into.push(
                    entry
                        .strip_prefix(base)
                        .expect("every entry is under base")
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
    }
    let mut into = Vec::new();
    walk(root, root, &mut into);
    into
}

// ---------------------------------------------------------------------------
// `--clear-killed`. #485.
// ---------------------------------------------------------------------------

/// `taxonomy publish --clear-killed`, as a person types it, over a root the
/// caller names.
fn publish_clearing_killed(root: &Path, out: &Path) -> (Option<i32>, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["taxonomy", "publish", "--package", "headwater/standard"])
        .arg("--clear-killed")
        .arg("--out")
        .arg(out)
        .arg("--root")
        .arg(root)
        .output()
        .expect("the binary runs");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// **The decisive case.** The flag over a non-empty `--out` with no staging
/// directory beside it exits 1 with the refusal a run without the flag gives,
/// and the file that was there is still there byte for byte.
///
/// The exit status is asserted as 1 and not merely as non-zero, because clap
/// exits 2 on an argument it does not know: a case that took non-zero for a
/// refusal would pass against a binary that never grew the flag at all. That is
/// the shape this whole target exists for.
#[test]
fn the_flag_over_an_unrelated_directory_refuses_and_every_file_survives() {
    let root = Root::copied("clear-killed-unrelated");
    let out = root.path().join("artifact");
    std::fs::create_dir_all(&out).expect("the caller's directory is made");
    std::fs::write(out.join("theirs.txt"), "the caller's own file").expect("their file is written");

    let (status, message) = publish_clearing_killed(root.path(), &out);
    assert_eq!(status, Some(1), "the flag did not refuse: {message}");
    assert!(
        message.contains("nothing was published"),
        "the refusal is not the publish's own: {message}"
    );
    assert_eq!(
        std::fs::read_to_string(out.join("theirs.txt")).expect("their file reads"),
        "the caller's own file"
    );
    assert_eq!(
        relative_files(&out),
        vec!["theirs.txt".to_string()],
        "the flag wrote or removed something under the caller's directory"
    );
}

/// The flag over the residue of a killed direct write clears both paths and
/// publishes the real artifact in the same run, exit 0.
///
/// It publishes this repository's own maintained source with `--from`, which is
/// the one invocation in this target that reaches exit 0, so the case measures
/// the whole run rather than a clear followed by a refusal for another reason.
#[test]
fn the_flag_over_a_killed_runs_residue_clears_it_and_publishes() {
    let root = Root::scratch("clear-killed-residue");
    let out = root.path().join("artifact");
    let mut name = out.file_name().expect("it has a name").to_os_string();
    name.push("~staging");
    let staging = out.with_file_name(name);
    std::fs::create_dir_all(&staging).expect("the staging directory is made");
    std::fs::write(
        staging.join(".headwater-publish-staging"),
        "an earlier run claimed this directory\n",
    )
    .expect("the marker is written");
    std::fs::write(
        staging.join(".headwater-publish-direct"),
        format!("an earlier run\nThe output path is: {}\n", out.display()),
    )
    .expect("the note is written");
    std::fs::create_dir_all(&out).expect("the output directory is made");
    std::fs::write(out.join("taxonomy.yml"), "half of an artifact\n")
        .expect("the residue is written");

    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .arg("taxonomy")
        .arg("publish")
        .arg("--clear-killed")
        .arg("--from")
        .arg(repository().join("taxonomy-source/headwater-standard"))
        .arg("--out")
        .arg(&out)
        .arg("--root")
        .arg(repository())
        .output()
        .expect("the binary runs");
    let message = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_eq!(
        output.status.code(),
        Some(0),
        "the clear and the publish did not land in one run: {message}"
    );
    assert!(
        !staging.exists(),
        "the killed run's staging directory outlived the clear: {message}"
    );
    assert!(
        headwater_resolve::release::at(&out).is_ok(),
        "the publish after the clear left no record: {message}"
    );
    assert!(
        message.contains("~staging"),
        "the run does not say what it removed: {message}"
    );
}

/// One step of the maintenance loop the shipped manifest instructs.
#[derive(Debug, PartialEq, Eq)]
enum Step {
    /// A `headwater …` command line, `<scratch-dir>` still unsubstituted.
    Run(Vec<String>),
    /// The hand edit that no verb performs: write the digest `publish` printed
    /// into `.headwater/taxonomy.yml` as `taxonomy.digest`. Spec 7 keeps this
    /// step out of the engine deliberately, so an instruction block that omits
    /// it omits the one step a reader cannot infer from a verb.
    Pin,
}

/// The steps `taxonomy-source/headwater-standard/package.yml` instructs, taken
/// out of its header comment in the order it states them.
///
/// The indented block is the whole grammar: a line under `#` and four spaces is
/// a step, a step that opens `headwater ` is a command, and any other step is
/// the hand-written pin. Nothing else in that file is indented, and a step this
/// function cannot classify fails the case rather than being skipped — a parser
/// that quietly dropped a step would report a loop shorter than the one that
/// ships.
fn maintenance_loop(manifest: &str) -> Vec<Step> {
    manifest
        .lines()
        .filter_map(|line| line.strip_prefix("#    "))
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| match line.strip_prefix("headwater ") {
            Some(rest) => Step::Run(rest.split_whitespace().map(str::to_string).collect()),
            None => {
                assert!(
                    line.contains("taxonomy.digest"),
                    "the instruction block carries a step this case cannot run: {line}"
                );
                Step::Pin
            }
        })
        .collect()
}

/// The binary, run from a directory, with the arguments exactly as a person
/// reading the instruction block would type them.
///
/// No `--root`, because the block states none. The working directory is what
/// makes `--from taxonomy-source/headwater-standard` mean what the block says
/// it means, and a case that passed `--root` would be testing its own
/// substitution rather than the sentence.
fn run_at(cwd: &Path, arguments: &[String]) -> (Option<i32>, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(arguments)
        .current_dir(cwd)
        .output()
        .expect("the binary runs");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// The digest a `publish` run printed, off its own standard output.
fn printed_digest(stdout: &str) -> String {
    stdout
        .lines()
        .find_map(|line| line.trim().strip_prefix("digest "))
        .unwrap_or_else(|| panic!("the publish printed no digest: {stdout}"))
        .trim()
        .to_string()
}

/// Rewrite `taxonomy.digest` in a consumer declaration. This is the hand edit,
/// performed by the case because no verb performs it.
fn pin_digest(consumer: &Path, digest: &str) {
    let at = consumer.join(".headwater/taxonomy.yml");
    let declaration = std::fs::read_to_string(&at).expect("the consumer declaration reads");
    let rewritten = declaration
        .lines()
        .map(|line| match line.trim_start().starts_with("digest:") {
            true => format!("  digest: {digest}"),
            false => line.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&at, format!("{rewritten}\n")).expect("the pin is written");
}

/// **The instruction block that ships inside the published artifact, executed.**
///
/// `the_shipped_starter_recipe_publishes_vendors_and_resolves` and
/// `a_pin_that_no_longer_names_the_installed_bytes_is_refused_by_conformance_and_not_by_resolve`
/// both run the publish/pin/vendor/resolve order, and both hardcode it. So both
/// stay green while the document that instructs a maintainer states a different
/// order. This case takes the order out of the document.
///
/// [#516](https://github.com/headwater-ai/headwater/issues/516) is why it
/// exists. `vendor --expect` verifies a caller-supplied digest and discards it,
/// and the refusal to record it stands: on the order `publish → pin → vendor →
/// resolve` the digest is typed once, and `--expect` is what creates the second
/// typing. That ruling is only true if the shipped instructions state that
/// order, and until this case ran nothing had ever executed them.
///
/// **The provocation is a source that actually changed**, because that is the
/// only state in which the two orders differ. On an unchanged source the
/// artifact carries the digest already pinned, and `vendor` exits 0 wherever
/// the pin step sits. A maintainer opens this block only after editing the
/// source, which is exactly the state that makes the block wrong.
///
/// Green only if every step of the block exits 0, in the block's own order.
/// Against a block that puts the pin after `vendor`, or leaves it to a sentence
/// underneath, the `vendor` step exits 1 against the stale pin.
///
/// `taxonomy-source/headwater-standard/package.yml` ships inside the published
/// artifact, so an adopter of `headwater/standard` reads this same block.
#[test]
fn the_shipped_maintenance_loop_runs_as_written_over_a_source_that_changed() {
    let root = Root::scratch("shipped-maintenance-loop");
    let consumer = root.path().join("consumer");

    // A consumer laid out the way this repository is, because the block names
    // `taxonomy-source/headwater-standard` by that relative path and the
    // manifest reaches its bundle library with `../../docs/taxonomies`.
    copy(
        &repository().join("taxonomy-source/headwater-standard"),
        &consumer.join("taxonomy-source/headwater-standard"),
    );
    copy(
        &repository().join("docs/taxonomies"),
        &consumer.join("docs/taxonomies"),
    );
    std::fs::create_dir_all(consumer.join(".headwater")).expect("the block directory is made");
    std::fs::copy(
        repository().join(".headwater/overlay.yml"),
        consumer.join(".headwater/overlay.yml"),
    )
    .expect("the overlay copies");

    let manifest = std::fs::read_to_string(
        repository().join("taxonomy-source/headwater-standard/package.yml"),
    )
    .expect("the shipped manifest reads");
    let steps = maintenance_loop(&manifest);

    // ---- The state a maintainer is in before they edit anything. ---------
    //
    // Published, pinned, vendored and resolved once by hand, off the source as
    // it stands. Every assertion below is about the second pass, so this pass
    // has to be sound or the case would prove nothing.
    let before = root.path().join("release-before");
    let (code, stdout, stderr) = run_at(
        &consumer,
        &[
            "taxonomy".to_string(),
            "publish".to_string(),
            "--from".to_string(),
            "taxonomy-source/headwater-standard".to_string(),
            "--out".to_string(),
            before.to_str().expect("the path is UTF-8").to_string(),
        ],
    );
    assert_eq!(code, Some(0), "{stderr}");
    write(
        &consumer.join(".headwater/taxonomy.yml"),
        &format!(
            "taxonomy:\n  package: headwater/standard\n  version: 4.2.0\n  digest: {}\n  bundles: [design-spec, evidence-and-obligation, decision-record]\n  overlay: .headwater/overlay.yml\ncorpus:\n  root: docs\n",
            printed_digest(&stdout)
        ),
    );
    for arguments in [
        vec![
            "taxonomy".to_string(),
            "vendor".to_string(),
            before.to_str().expect("the path is UTF-8").to_string(),
        ],
        vec!["taxonomy".to_string(), "resolve".to_string()],
    ] {
        let (code, _stdout, stderr) = run_at(&consumer, &arguments);
        assert_eq!(code, Some(0), "the settled state is not sound: {stderr}");
    }

    // ---- The edit that sends a maintainer to the instruction block. ------
    //
    // A comment appended to the authored taxonomy source. It moves the bytes of
    // a member file and nothing else, so the published digest moves and the
    // package still resolves.
    let source = consumer.join("taxonomy-source/headwater-standard/taxonomy.yml");
    let authored = std::fs::read_to_string(&source).expect("the authored source reads");
    std::fs::write(
        &source,
        format!("{authored}\n# an edit to the authored source, which moves the digest\n"),
    )
    .expect("the edit writes");

    // ---- The block, run as written. --------------------------------------
    let scratch = root.path().join("release-after");
    let mut printed = String::new();
    for step in &steps {
        match step {
            Step::Pin => pin_digest(&consumer, &printed_digest(&printed)),
            Step::Run(arguments) => {
                let arguments: Vec<String> = arguments
                    .iter()
                    .map(|argument| match argument.as_str() {
                        "<scratch-dir>" => scratch.to_str().expect("the path is UTF-8").to_string(),
                        other => other.to_string(),
                    })
                    .collect();
                let (code, stdout, stderr) = run_at(&consumer, &arguments);
                assert_eq!(
                    code,
                    Some(0),
                    "the shipped instruction block does not run as written. `headwater {}` \
                     refused after the source changed:\n{stderr}",
                    arguments.join(" ")
                );
                printed = stdout;
            }
        }
    }

    // Read after the run and not before it, so that a block missing the pin
    // step fails at the step that refuses rather than at a guard: the account a
    // maintainer needs is `vendor`'s own message about the digest it was given.
    assert!(
        steps.contains(&Step::Pin),
        "the instruction block ran green without stating a pin step, so this case no longer \
         holds the order it exists for: {steps:?}"
    );

    // The loop ended somewhere real: the vendored bytes are the ones the second
    // publish wrote, and the pin names them. `pin.current` is the reading that
    // says so, and `--level L0` is what turns a gap in it into an exit status.
    let (code, stdout, stderr) = run_at(
        &consumer,
        &[
            "conformance".to_string(),
            "--level".to_string(),
            "L0".to_string(),
        ],
    );
    assert_eq!(
        code,
        Some(0),
        "the loop ran to the end and left a pin that does not name the installed bytes: \
         {stdout}\n{stderr}"
    );
}
