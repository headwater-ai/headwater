// SPDX-License-Identifier: Apache-2.0
//! Publishing a package and consuming one, over a tree on disk.
//!
//! The unit tests in `release.rs` hold the record and the range. These hold the
//! two verbs against a publisher and a consumer that are separate directories,
//! because every property worth having here is about what crosses between them:
//! a `..` that must not survive publication, a byte that must not survive a pin,
//! and a package directory that a consumer command must not overwrite.
//!
//! Each case provokes the refusal it is about. A vendor that has refused nothing
//! is a vendor nobody has seen work.

use headwater_resolve::package;
use headwater_resolve::release::{self, ReleaseError};
use std::path::{Path, PathBuf};

/// A tree that removes itself, named for the case that made it.
struct Scratch(PathBuf);

impl Scratch {
    fn new(case: &str) -> Self {
        let at =
            std::env::temp_dir().join(format!("headwater-publish-{}-{case}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the scratch tree is made");
        Scratch(at)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.0.join(relative);
        std::fs::create_dir_all(path.parent().expect("it has a parent"))
            .expect("the parent is made");
        std::fs::write(path, text).expect("the file is written");
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const TAXONOMY: &str = "\
taxonomy: acme/fixture
version: 1.0.0
purposes:
  rationale: {intent: explain why a choice was made and what it forecloses}
";

const BUNDLE: &str = "\
overlay: acme/fixture
add:
  purposes:
    procedure: {intent: state how a task is carried out}
";

/// A publisher whose manifest points its bundles out of the package, the way
/// this repository's own does while the package and the library share a tree.
fn publisher(scratch: &Scratch, requires_engine: Option<&str>) -> PathBuf {
    let mut manifest = String::from("package: acme/fixture\nversion: 1.0.0\n");
    if let Some(range) = requires_engine {
        manifest.push_str(&format!("requires_engine: \"{range}\"\n"));
    }
    manifest.push_str("contents:\n  taxonomy: taxonomy.yml\n  bundles: ../../library\n");
    scratch.write("publisher/packages/acme-fixture/package.yml", &manifest);
    scratch.write("publisher/packages/acme-fixture/taxonomy.yml", TAXONOMY);
    scratch.write("publisher/library/extra/bundle.yml", BUNDLE);
    scratch.path().join("publisher")
}

/// A consumer that pins one digest and takes the bundle the package ships.
fn consumer(scratch: &Scratch, digest: &str) {
    scratch.write(
        "consumer/.headwater/taxonomy.yml",
        &format!(
            "\
taxonomy:
  package: acme/fixture
  version: 1.0.0
  bundles: [extra]
  digest: {digest}
corpus:
  root: docs
"
        ),
    );
}

/// The `..` is a property of the source layout, and nothing that leaves the
/// publisher carries it.
///
/// This is the finding the issue filed against `contents.bundles`. The published
/// manifest names a path inside the artifact, the bundle is there, and the
/// consumer resolves the same taxonomy the publisher would.
#[test]
fn a_published_package_carries_its_bundles_and_no_path_that_leaves_it() {
    let scratch = Scratch::new("bundles");
    let root = publisher(&scratch, None);
    let out = scratch.path().join("artifact");

    let record = package::publish(&root, "acme/fixture", &out).expect("it publishes");
    assert_eq!(record.package, "acme/fixture");

    let manifest = std::fs::read_to_string(out.join(package::MANIFEST)).expect("it is there");
    assert!(
        !manifest.contains(".."),
        "the published manifest still climbs out of the package:\n{manifest}"
    );
    assert!(manifest.contains("bundles: bundles"));
    assert!(out.join("bundles/extra/bundle.yml").is_file());

    // The record names every file in the artifact and never itself.
    let paths: Vec<&str> = record.members.iter().map(|m| m.path.as_str()).collect();
    assert!(paths.contains(&"package.yml"));
    assert!(paths.contains(&"taxonomy.yml"));
    assert!(paths.contains(&"bundles/extra/bundle.yml"));
    assert!(!paths.contains(&release::RECORD));

    // And the artifact resolves for a consumer that vendors it.
    consumer(&scratch, &record.digest);
    let consumer_root = scratch.path().join("consumer");
    package::vendor(&consumer_root, &out, &record.digest).expect("it vendors");
    let declaration = package::consumer(&consumer_root).expect("it reads");
    let sources = package::sources(&consumer_root, &declaration).expect("the sources are found");
    assert_eq!(sources.len(), 2, "the taxonomy and the bundle it selects");
}

/// One byte changed in one file is refused, and the message names the file.
#[test]
fn a_changed_byte_is_refused_and_the_file_is_named() {
    let scratch = Scratch::new("changed");
    let root = publisher(&scratch, None);
    let out = scratch.path().join("artifact");
    let record = package::publish(&root, "acme/fixture", &out).expect("it publishes");

    let path = out.join("bundles/extra/bundle.yml");
    let text = std::fs::read_to_string(&path).expect("it is there");
    std::fs::write(&path, format!("{text}# one comment\n")).expect("it is written");

    let refused = release::verify(&out, &record.digest).expect_err("a changed artifact is refused");
    let message = refused.to_string();
    assert!(matches!(refused, ReleaseError::Diverged(_)), "{message}");
    assert!(
        message.contains("bundles/extra/bundle.yml"),
        "the message does not name what moved: {message}"
    );
}

/// A file added to the artifact is refused, which a per-file comparison over the
/// record alone would miss.
#[test]
fn a_file_the_record_does_not_name_is_refused() {
    let scratch = Scratch::new("added");
    let root = publisher(&scratch, None);
    let out = scratch.path().join("artifact");
    let record = package::publish(&root, "acme/fixture", &out).expect("it publishes");

    std::fs::create_dir_all(out.join("bundles/smuggled")).expect("the directory is made");
    std::fs::write(out.join("bundles/smuggled/bundle.yml"), BUNDLE).expect("it is written");

    let refused = release::verify(&out, &record.digest).expect_err("an added file is refused");
    assert!(refused.to_string().contains("bundles/smuggled/bundle.yml"));
}

/// A file removed from the artifact is refused.
#[test]
fn a_file_the_artifact_lost_is_refused() {
    let scratch = Scratch::new("missing");
    let root = publisher(&scratch, None);
    let out = scratch.path().join("artifact");
    let record = package::publish(&root, "acme/fixture", &out).expect("it publishes");

    std::fs::remove_file(out.join("taxonomy.yml")).expect("it is removed");

    let refused = release::verify(&out, &record.digest).expect_err("a missing file is refused");
    assert!(refused.to_string().contains("taxonomy.yml"));
}

/// An adversary who republishes the whole artifact, record and all, is refused
/// by the pin.
///
/// This is the case the record cannot catch on its own: every file agrees with
/// the record, because the record was written for these files. What refuses it
/// is the digest the consumer wrote down before the artifact arrived.
#[test]
fn a_consistent_forgery_is_refused_by_the_pin() {
    let scratch = Scratch::new("forged");
    let root = publisher(&scratch, None);
    let honest = scratch.path().join("honest");
    let published = package::publish(&root, "acme/fixture", &honest).expect("it publishes");

    // The publisher's tree, edited, and published again. Every internal check
    // passes over the result.
    let taxonomy = root.join("packages/acme-fixture/taxonomy.yml");
    let text = std::fs::read_to_string(&taxonomy).expect("it is there");
    std::fs::write(&taxonomy, format!("{text}# what the adversary added\n")).expect("written");
    let forged = scratch.path().join("forged");
    let second = package::publish(&root, "acme/fixture", &forged).expect("it publishes");
    assert!(release::verify(&forged, &second.digest).is_ok());

    let refused =
        release::verify(&forged, &published.digest).expect_err("the pin refuses the forgery");
    assert!(matches!(refused, ReleaseError::NotPinned { .. }));
    let message = refused.to_string();
    assert!(message.contains(&published.digest));
    assert!(message.contains(&second.digest));
}

/// A package directory that a person maintains is not overwritten by a consumer
/// command.
///
/// The publisher's own tree is the case: `packages/acme-fixture/` there is a
/// source, not a vendored artifact, and a vendor that replaced it would delete
/// the thing being published.
#[test]
fn vendoring_over_a_maintained_package_is_refused() {
    let scratch = Scratch::new("maintained");
    let root = publisher(&scratch, None);
    let out = scratch.path().join("artifact");
    let record = package::publish(&root, "acme/fixture", &out).expect("it publishes");

    // Vendor into the publisher's own root, where `packages/acme-fixture/`
    // carries no release record.
    let refused = package::vendor(&root, &out, &record.digest)
        .expect_err("a maintained package is not overwritten");
    assert!(headwater_resolve::render_errors(&refused).contains("somebody maintains"));
    assert!(root.join("packages/acme-fixture/taxonomy.yml").is_file());

    // A directory that was vendored carries one, and is replaced.
    let consumer_root = scratch.path().join("consumer");
    consumer(&scratch, &record.digest);
    package::vendor(&consumer_root, &out, &record.digest).expect("the first vendor lands");
    package::vendor(&consumer_root, &out, &record.digest).expect("the second replaces it");
}

/// A package that declares an engine range this engine is outside of is refused
/// before a source is read.
#[test]
fn a_package_that_needs_a_later_engine_does_not_resolve() {
    let scratch = Scratch::new("engine");
    let root = publisher(&scratch, Some(">=9 <10"));
    scratch.write(
        "publisher/.headwater/taxonomy.yml",
        "\
taxonomy:
  package: acme/fixture
  version: 1.0.0
corpus:
  root: docs
",
    );
    let declaration = package::consumer(&root).expect("it reads");
    let refused = package::sources(&root, &declaration).expect_err("the range refuses it");
    let message = headwater_resolve::render_errors(&refused);
    assert!(message.contains(">=9 <10"), "{message}");
    assert!(message.contains(release::ENGINE), "{message}");
}

/// The range the base package of this repository declares is one this engine
/// satisfies, so the sources of this repository still load.
#[test]
fn the_package_in_this_repository_declares_a_range_this_engine_is_inside() {
    let root = Path::new("../../..");
    let declaration = package::consumer(root).expect("this repository declares one");
    let manifest = std::fs::read_to_string(
        root.join(package::PACKAGES)
            .join("headwater-standard")
            .join(package::MANIFEST),
    )
    .expect("the manifest is there");
    assert!(
        manifest.contains(package::REQUIRES_ENGINE),
        "the base package declares no engine range, and this issue is where it earned one"
    );
    package::sources(root, &declaration).expect("the sources of this repository load");
}
