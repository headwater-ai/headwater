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
    /// `contents.doctrine` names a directory that exists and holds no file.
    ///
    /// This is the state a *correct* stager still produces, on both publish
    /// paths: `reachable` passes it, because the directory is there and it is a
    /// directory, and the walk that carries bytes then carries none. The
    /// manifest reaches a consumer naming a path the artifact does not hold.
    EmptyDoctrine,
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
/// **It declares `contents.conformance`, and that is deliberate.** Until #581
/// no fixture in this file declared a file-valued `contents` key beyond
/// `taxonomy`, so nothing here ever asked whether a flattened artifact carries
/// one. `headwater/standard` declares one and the flattened starter did not
/// carry it.
fn assembly_source(root: &Root, arm: Arm) -> PathBuf {
    let source = root.path().join("source/acme-fixture");
    write(
        &source.join("package.yml"),
        "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n  conformance: conformance.yml\n  bundles: bundles\n  assemblies: assemblies\n  doctrine: doctrine\n  templates: templates\n",
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
        Arm::Sound | Arm::EmptyDoctrine => "[alpha, beta]",
    };
    write(
        &source.join("assemblies/starter/assembly.yml"),
        &format!(
            "assembly: starter\npackage: acme/starter\nversion: 2.0.0\nfrom:\n  package: acme/fixture@1.0.0\n  bundles: {selected}\noverlay: overlay.yml\n"
        ),
    );
    write(
        &source.join("assemblies/starter/overlay.yml"),
        "add:\n  relations.connects:\n    family: derivation\n    from: [alpha]\n    to: [beta]\n    nucleus: from\n    inverse: connected_by\n    reciprocal: required\n    created_by: scaffold\n",
    );
    match arm {
        Arm::EmptyDoctrine => {
            std::fs::create_dir_all(source.join("doctrine"))
                .expect("the empty doctrine directory is made");
        }
        Arm::Sound | Arm::UnselectableBundle => {
            write(&source.join("doctrine/guide.md"), "# Fixture doctrine\n");
        }
    }
    write(&source.join("templates/decision.md"), "# Decision\n");
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
    let record = std::fs::read_to_string(out.join("release.yml")).expect("the release reads");
    assert!(
        record.contains("conformance.yml"),
        "the release record names no conformance rule set: {record}"
    );
}

/// A declared directory with no file in it is the case a correct stager still
/// produces, and it is why the guard is general rather than a second key check.
///
/// `reachable` admits `contents.doctrine` here: the directory is there and it is
/// a directory. The walk that carries bytes then carries none, because a
/// directory with no file in it has nothing to copy. Both these cases were
/// measured at exit 0 before #581, with the manifest naming a path the artifact
/// did not hold, on both publish paths.
#[test]
fn a_flattened_manifest_may_not_name_a_member_the_artifact_does_not_carry() {
    let root = Root::scratch("assembly-empty-doctrine");
    let source = assembly_source(&root, Arm::EmptyDoctrine);
    let out = root.path().join("release");

    let (code, _stdout, stderr) = publish_assembly_from(root.path(), &source, &out);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("`contents.doctrine`"), "{stderr}");
    assert!(stderr.contains("doctrine/starter"), "{stderr}");
    assert!(
        !out.exists(),
        "the refused run left partial output at {out:?}"
    );
}

#[test]
fn a_published_manifest_may_not_name_a_member_the_artifact_does_not_carry() {
    let root = Root::scratch("plain-empty-doctrine");
    let source = assembly_source(&root, Arm::EmptyDoctrine);
    let out = root.path().join("release");

    let (code, _stdout, stderr) = publish_plain_from(root.path(), &source, &out);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("`contents.doctrine`"), "{stderr}");
    assert!(stderr.contains("doctrine"), "{stderr}");
    assert!(
        !out.exists(),
        "the refused run left partial output at {out:?}"
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
    assert!(
        stderr.is_empty(),
        "a JSON run that succeeded accounts for nothing on standard error: {stderr}"
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
