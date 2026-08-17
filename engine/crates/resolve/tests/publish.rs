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
    publisher_at(scratch, requires_engine, "1.0.0")
}

/// The same publisher, with the version its taxonomy source declares set by the
/// caller.
///
/// The manifest stays at `1.0.0` whatever this is. A package states its version
/// twice — the manifest key that spec 7 gives it and the taxonomy-source key
/// that the meta-schema requires — and a caller that can move one without the
/// other is what makes the disagreement reachable from a test.
fn publisher_at(scratch: &Scratch, requires_engine: Option<&str>, source: &str) -> PathBuf {
    let mut manifest = String::from("package: acme/fixture\nversion: 1.0.0\n");
    if let Some(range) = requires_engine {
        manifest.push_str(&format!("requires_engine: \"{range}\"\n"));
    }
    manifest.push_str("contents:\n  taxonomy: taxonomy.yml\n  bundles: ../../library\n");
    scratch.write("publisher/packages/acme-fixture/package.yml", &manifest);
    scratch.write(
        "publisher/packages/acme-fixture/taxonomy.yml",
        &TAXONOMY.replace("version: 1.0.0", &format!("version: {source}")),
    );
    scratch.write("publisher/library/extra/bundle.yml", BUNDLE);
    scratch.path().join("publisher")
}

/// A consumer declaration inside the publisher tree, pinning what it takes.
fn takes(scratch: &Scratch, version: &str) {
    scratch.write(
        "publisher/.headwater/taxonomy.yml",
        &format!(
            "\
taxonomy:
  package: acme/fixture
  version: {version}
corpus:
  root: docs
"
        ),
    );
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
    takes(&scratch, "1.0.0");
    let declaration = package::consumer(&root).expect("it reads");
    let refused = package::sources(&root, &declaration).expect_err("the range refuses it");
    let message = headwater_resolve::render_errors(&refused);
    assert!(message.contains(">=9 <10"), "{message}");
    assert!(message.contains(release::ENGINE), "{message}");
}

/// A package that declares one version in its manifest and another in its
/// taxonomy source is refused, whichever of the two the consumer pinned.
///
/// # Why both pins are one case
///
/// The refusal has to fire on the package rather than on the pairing. Before
/// [#212](https://github.com/headwater-ai/headwater/issues/212) the only
/// comparison here was the consumer's pin against the manifest, so a package
/// carrying two versions of itself resolved clean for a consumer who happened
/// to pin the manifest number and was refused for the wrong reason — a message
/// about the pin, naming one file and one number — for a consumer who happened
/// to pin the other. Which of the two a consumer wrote is not a property of the
/// package, so it cannot be what decides whether the package is well formed.
/// Both arms therefore assert the same refusal, and the second arm is the one
/// that fails if the check is put after the pin comparison rather than before
/// it.
#[test]
fn a_package_that_states_two_versions_of_itself_is_refused_on_either_pin() {
    for pinned in ["1.0.0", "2.0.0"] {
        let scratch = Scratch::new(&format!("two-versions-{pinned}"));
        // The manifest stays at 1.0.0 and the taxonomy source goes to 2.0.0.
        let root = publisher_at(&scratch, None, "2.0.0");
        takes(&scratch, pinned);

        let declaration = package::consumer(&root).expect("it reads");
        let refused = package::sources(&root, &declaration)
            .expect_err("a package with two versions of itself does not load");
        let message = headwater_resolve::render_errors(&refused);

        // Both files, so a reader knows where to go.
        assert!(
            message.contains("packages/acme-fixture/package.yml"),
            "the manifest is not named:\n{message}"
        );
        assert!(
            message.contains("packages/acme-fixture/taxonomy.yml"),
            "the taxonomy source is not named:\n{message}"
        );
        // Both numbers, so a reader knows which two disagree.
        assert!(message.contains("1.0.0"), "{message}");
        assert!(message.contains("2.0.0"), "{message}");
        // And not the pin refusal, which is a different question about a
        // different pair of values.
        assert!(
            !message.contains("this takes"),
            "the pin comparison answered first, so the package was never held to itself:\n{message}"
        );
    }
}

/// The same disagreement stops a publish, before a byte reaches an artifact.
///
/// `publish` does not resolve for a consumer, so it does not pass through the
/// comparison above. It copies both files into the artifact and the release
/// digest covers both, so without its own reading of this a publisher seals two
/// numbers under one digest and an adopter receives a package that says two
/// things about what it is.
#[test]
fn a_publish_of_a_package_that_states_two_versions_is_refused() {
    let scratch = Scratch::new("two-versions-publish");
    let root = publisher_at(&scratch, None, "2.0.0");
    let out = scratch.path().join("artifact");

    let refused =
        package::publish(&root, "acme/fixture", &out).expect_err("the publish does not run");
    let message = headwater_resolve::render_errors(&refused);
    assert!(message.contains("package.yml"), "{message}");
    assert!(message.contains("taxonomy.yml"), "{message}");
    assert!(message.contains("1.0.0"), "{message}");
    assert!(message.contains("2.0.0"), "{message}");
    assert!(
        !out.join(package::MANIFEST).exists(),
        "the artifact was written anyway"
    );
}

/// An artifact that already carries two versions is refused on the path
/// `taxonomy diff` and `taxonomy migrate` reach a fetched directory by.
///
/// This engine cannot publish such an artifact any more, so the only publisher
/// that can hand one over is a publisher on some other engine. That is the case
/// that matters: an adopter fetches a directory, the release digest covers both
/// files, and the digest proves the bytes rather than that they agree. So the
/// refusal has to sit on the reading of the fetched directory and not only on
/// the writing of one.
#[test]
fn a_fetched_artifact_that_states_two_versions_is_refused() {
    let scratch = Scratch::new("two-versions-fetched");
    let root = publisher(&scratch, None);
    let out = scratch.path().join("artifact");
    package::publish(&root, "acme/fixture", &out).expect("the honest artifact publishes");

    // What another publisher's engine could have written: the manifest at
    // 1.0.0 and the taxonomy source beside it at 2.0.0.
    let source = out.join("taxonomy.yml");
    let text = std::fs::read_to_string(&source).expect("it is there");
    std::fs::write(&source, text.replace("version: 1.0.0", "version: 2.0.0")).expect("it writes");

    consumer(&scratch, "sha256:0");
    let consumer_root = scratch.path().join("consumer");
    let declaration = package::consumer(&consumer_root).expect("it reads");
    let manifest = package::manifest_at(&out).expect("the manifest reads");
    let refused = package::sources_at(&consumer_root, &out, &manifest, &declaration)
        .expect_err("the fetched artifact does not load");
    let message = headwater_resolve::render_errors(&refused);
    assert!(message.contains("1.0.0"), "{message}");
    assert!(message.contains("2.0.0"), "{message}");
    assert!(message.contains("taxonomy.yml"), "{message}");
}

/// The two declarations of this repository's own package agree.
///
/// The case above proves the refusal fires. This one proves it is not firing on
/// the tree it ships in, and it reads both files rather than asserting a
/// literal, so a version bump that moves one and forgets the other fails here
/// as well as at the gate.
#[test]
fn the_package_in_this_repository_states_one_version_in_both_files() {
    let root = Path::new("../../..");
    let directory = root.join(package::PACKAGES).join("headwater-standard");
    let manifest =
        std::fs::read_to_string(directory.join(package::MANIFEST)).expect("the manifest is there");
    let source =
        std::fs::read_to_string(directory.join("taxonomy.yml")).expect("the source is there");

    let declared = |text: &str| {
        text.lines()
            .find_map(|line| line.strip_prefix("version: "))
            .map(str::to_string)
            .expect("a version is declared")
    };
    assert_eq!(
        declared(&manifest),
        declared(&source),
        "packages/headwater-standard states two versions of itself"
    );
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

/// The same publisher, with the bundle library its manifest names never written.
///
/// This is the shape of the vendored copy that [#271] is about: a manifest whose
/// `..` points at a tree that is not there, because the package directory was
/// copied out of the repository that holds the library beside it.
///
/// **It is the one input that reaches the write phase and comes back out.**
/// Every other refusal `publish` makes — the two version declarations
/// disagreeing, an engine range, a migration payload, a package that is not
/// found — fires before the first byte, so *`--out` untouched* is the ambient
/// outcome for all of them and a case built on one would pass while proving
/// nothing.
///
/// [#271]: https://github.com/headwater-ai/headwater/issues/271
fn publisher_without_its_library(scratch: &Scratch) -> PathBuf {
    let root = publisher(scratch, None);
    std::fs::remove_dir_all(scratch.path().join("publisher/library")).expect("the library goes");
    root
}

/// A publisher whose manifest names content it does not carry, without a `..`.
///
/// `conformance` is declared and absent, and `bundles` is declared, absent, and
/// inside the package. Spec 7 says every `contents` path a publisher writes is
/// read, and before this both of these published an artifact with a hole in it
/// and exited 0.
fn publisher_naming_content_it_does_not_carry(scratch: &Scratch) -> PathBuf {
    scratch.write(
        "hollow/packages/acme-fixture/package.yml",
        "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n  \
         conformance: nosuch-conformance.yml\n  bundles: nosuch-bundles\n",
    );
    scratch.write("hollow/packages/acme-fixture/taxonomy.yml", TAXONOMY);
    scratch.path().join("hollow")
}

/// A publish that cannot read what its manifest declares writes nothing at all.
///
/// The three assertions are the three halves of [#271] that a caller can see.
/// The message names the manifest rather than a directory, so a reader is sent
/// to the declaration rather than to a path with a `..` in it. `--out` does not
/// exist afterwards, which is the one assertion no cosmetic change satisfies.
/// And the second run meets the same refusal as the first, rather than the
/// output-directory precondition catching the leftovers of the first.
///
/// [#271]: https://github.com/headwater-ai/headwater/issues/271
#[test]
fn a_publish_that_cannot_read_its_declared_content_leaves_the_output_directory_as_it_found_it() {
    let scratch = Scratch::new("unreadable-content");
    let root = publisher_without_its_library(&scratch);
    let out = scratch.path().join("artifact");

    let refused = package::publish(&root, "acme/fixture", &out).expect_err("it does not publish");
    let first = headwater_resolve::render_errors(&refused);
    assert!(
        first.contains(package::MANIFEST),
        "the refusal does not name the manifest that declares the path: {first}"
    );
    assert!(
        first.contains("../../library"),
        "the refusal does not name the declared value: {first}"
    );
    assert!(
        !out.exists(),
        "the output directory was created by a publish that says it published nothing: {:?}",
        std::fs::read_dir(&out)
            .map(|entries| entries.filter_map(Result::ok).map(|e| e.path()).collect::<Vec<_>>())
    );

    let again = package::publish(&root, "acme/fixture", &out).expect_err("it does not publish");
    assert_eq!(
        headwater_resolve::render_errors(&again),
        first,
        "the second run met a different refusal, so the first run left something behind"
    );
}

/// A declared content path that stays inside the package is read too.
///
/// The escaping path is the loud half of #271 and this is the quiet one. Spec 7
/// (Publishing): "Every `contents` path a publisher writes is read … A key that
/// no verb reads is a claim that a publisher makes and a consumer never sees."
/// Before this, a package declaring `contents.conformance` and
/// `contents.bundles` with neither on disk published two members and exited 0.
#[test]
fn a_publish_refuses_content_it_does_not_carry_even_where_no_path_escapes() {
    let scratch = Scratch::new("hollow");
    let root = publisher_naming_content_it_does_not_carry(&scratch);
    let out = scratch.path().join("artifact");

    let refused = package::publish(&root, "acme/fixture", &out).expect_err("it does not publish");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains(package::MANIFEST),
        "the refusal does not name the manifest: {message}"
    );
    assert!(
        message.contains("nosuch-conformance.yml") || message.contains("nosuch-bundles"),
        "the refusal names neither declared path: {message}"
    );
    assert!(!out.exists(), "nothing is written for this one either");
}

/// This repository's own package publishes, and the artifact is what the record
/// says it is.
///
/// The precondition above reads every declared `contents` path, and the base
/// package declares one that escapes on purpose. This is the case that would
/// fail if the precondition were written to refuse an escape rather than to
/// read one.
///
/// It also holds the published identity to the files on disk. The digest is
/// taken over the member list, and a staging refactor that dropped one relative
/// path or swept in `release.yml` would move the number every future consumer
/// pins without changing a byte of any file. The literal is never written down
/// here, because it moves with any file under `docs/taxonomies/`.
#[test]
fn the_package_in_this_repository_publishes_and_its_digest_covers_what_is_on_disk() {
    let scratch = Scratch::new("this-repository");
    let root = Path::new("../../..");
    let out = scratch.path().join("artifact");

    let record = package::publish(root, "headwater/standard", &out).expect("it publishes");

    let on_disk = release::members(&out).expect("the artifact reads");
    assert_eq!(
        record.members, on_disk,
        "the record does not name the files that were written"
    );
    assert_eq!(
        record.digest,
        release::digest_of(&on_disk),
        "the digest is not the digest of what is on disk"
    );
    release::verify(&out, &record.digest).expect("the artifact verifies against its own digest");

    let files = walk_files(&out);
    assert_eq!(
        record.members.len() + 1,
        files,
        "every file except {} is a member",
        release::RECORD
    );

    let manifest = std::fs::read_to_string(out.join(package::MANIFEST)).expect("it is there");
    assert!(
        manifest.contains("bundles: bundles"),
        "the escaping path did not become one inside the artifact: {manifest}"
    );
    assert!(
        out.join("bundles").is_dir(),
        "the bundles the manifest now names are not there"
    );
}

/// Every file under a directory, counted.
fn walk_files(at: &Path) -> usize {
    std::fs::read_dir(at)
        .expect("the directory reads")
        .filter_map(Result::ok)
        .map(|entry| match entry.path().is_dir() {
            true => walk_files(&entry.path()),
            false => 1,
        })
        .sum()
}
