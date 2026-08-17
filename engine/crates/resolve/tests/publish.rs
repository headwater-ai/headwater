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
        std::fs::read_dir(&out).map(|entries| entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .collect::<Vec<_>>())
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

/// A mode, set on a path.
#[cfg(unix)]
fn mode(at: &Path, bits: u32) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(at, std::fs::Permissions::from_mode(bits)).expect("the mode is set");
}

/// Whether a mode can stop this process.
///
/// Root reads and writes through every bit, so under root the four cases that
/// set one are unreachable rather than failing, and they say so and stop. The
/// answer is a probe rather than a user id, because the question is what the
/// file system does to this process and no dependency of this crate can ask for
/// a uid. `engine/README.md` says why the container half of the toolchain runs
/// with `--user` for the same reason.
#[cfg(unix)]
fn modes_hold(scratch: &Scratch) -> bool {
    let at = scratch.path().join("mode-probe");
    std::fs::create_dir_all(&at).expect("the probe is made");
    std::fs::write(at.join("held"), "x").expect("the probe holds a file");
    mode(&at, 0o300);
    let held = std::fs::read_dir(&at).is_err();
    mode(&at, 0o700);
    std::fs::remove_dir_all(&at).expect("the probe goes");
    held
}

/// An `--out` that this run cannot read is refused, and the file in it is still
/// there.
///
/// # Why the destructive reading was the one that was easy to write
///
/// `read_dir` reports *did not observe* and *observed nothing* through one
/// `Err`, and the first reading of that error read every failure as absence.
/// Absence is the state whose undo removes `--out` itself. So a directory
/// holding a person's file, unreadable and writable, was recorded as absent: the
/// precondition never saw the file, the write phase put a whole artifact in
/// beside it, `release::compute` then could not read the directory back, and the
/// verb printed `nothing was published` over the files it had just written. The
/// undo it ran on the way out was `remove_dir_all` against the caller's own
/// directory, which fails on this mode and would have taken the file with it on
/// any other.
///
/// Mode `0300` is what makes both halves reachable at once: no `r`, so the
/// precondition is blind, and `w` and `x`, so every write below it succeeds.
#[cfg(unix)]
#[test]
fn an_output_directory_that_cannot_be_read_is_refused_and_nothing_is_written() {
    let scratch = Scratch::new("unreadable-out");
    if !modes_hold(&scratch) {
        eprintln!("skipped: this process is root, and root reads through mode 0300");
        return;
    }
    let root = publisher(&scratch, None);
    let out = scratch.path().join("artifact");
    std::fs::create_dir_all(&out).expect("the caller's directory is made");
    std::fs::write(out.join("theirs.txt"), "the caller's own file").expect("their file is written");
    mode(&out, 0o300);

    let refused = package::publish(&root, "acme/fixture", &out).expect_err("it does not publish");
    let message = headwater_resolve::render_errors(&refused);
    mode(&out, 0o700);

    let left: Vec<String> = std::fs::read_dir(&out)
        .expect("it reads once the mode is back")
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        left,
        vec!["theirs.txt".to_string()],
        "the publish wrote into a directory it could not read, or removed what was in it"
    );
    assert_eq!(
        std::fs::read_to_string(out.join("theirs.txt")).expect("their file reads"),
        "the caller's own file"
    );
    assert!(
        message.contains("artifact"),
        "the refusal does not name the output path: {message}"
    );
    assert!(
        message.contains("the output directory cannot be read"),
        "the refusal is not the one the precondition owes a reader: {message}"
    );
}

/// A dangling symlink at `--out` is refused, and the link is still there.
///
/// `read_dir` follows a symlink, so a link with nothing at the other end reports
/// `NotFound` while a path very much exists under that name. Read as absence,
/// the undo of a failed write then removes a link that the run did not make and
/// nothing in the corpus records. The publisher of this fixture is one that
/// publishes, so the run reaches the write phase and comes back through the
/// undo, which is where the link used to go.
#[cfg(unix)]
#[test]
fn a_dangling_symlink_at_the_output_path_is_refused_and_is_still_there() {
    let scratch = Scratch::new("dangling-out");
    let root = publisher(&scratch, None);
    let out = scratch.path().join("artifact");
    std::os::unix::fs::symlink(scratch.path().join("nowhere"), &out).expect("the link is made");

    let refused = package::publish(&root, "acme/fixture", &out).expect_err("it does not publish");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains("artifact"),
        "the refusal does not name the output path: {message}"
    );
    assert!(
        std::fs::symlink_metadata(&out).is_ok(),
        "the publish removed a symlink that was there before it started: {message}"
    );
    assert!(
        !out.exists(),
        "the refusal followed the link and made something at the other end"
    );
}

/// A publisher that reaches the write phase and fails inside it, with files
/// already on disk.
///
/// # Why this shape, and why the suite needed one
///
/// Every other refusal `publish` makes fires in phase 1, so before this the undo
/// was unreachable from any test in the workspace: an inspection that replaced
/// `unwind` with an early return left 851 tests passing. *`--out` untouched* is
/// the ambient outcome of a phase-1 refusal, and a case built on one asserts
/// that an untouched directory is untouched.
///
/// The provocation is a regular file named `bundles` at the package root, beside
/// a `contents.bundles` that points outside the package. Both are legal to read:
/// the staged set holds a file at `bundles` and a file under `bundles/extra/`,
/// in that order. `put` then writes three files and cannot make a directory
/// where it has just written a file. Three files on disk, a failure, and an undo
/// with something to do — and no mode and no race anywhere in it.
fn publisher_that_fails_inside_the_write(scratch: &Scratch) -> PathBuf {
    let root = publisher(scratch, None);
    scratch.write(
        "publisher/packages/acme-fixture/bundles",
        "a regular file where the artifact needs a directory\n",
    );
    root
}

/// A write that fails takes `--out` with it, and every directory it made to
/// reach it.
///
/// This is the undo's first state. `--out` is three levels below a directory
/// that is not there either, which `put` reaches with `create_dir_all`, so a run
/// that says it published nothing would otherwise leave three directories it
/// made standing.
#[test]
fn a_write_that_fails_takes_the_output_directory_and_what_it_made_to_reach_it() {
    let scratch = Scratch::new("failed-write-absent");
    let root = publisher_that_fails_inside_the_write(&scratch);
    let nested = scratch.path().join("nested");
    let out = nested.join("a/b/c");

    let refused = package::publish(&root, "acme/fixture", &out).expect_err("it does not publish");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains("bundles"),
        "this case is not failing inside the write phase any more: {message}"
    );
    assert!(
        !out.exists(),
        "the output directory is still there: {message}"
    );
    assert!(
        !nested.exists(),
        "the undo left the directories the run made to reach --out"
    );
    assert!(
        scratch.path().is_dir(),
        "the undo climbed past what the run created"
    );
}

/// The same failure, into an empty `--out` that the caller made: the directory
/// stays and the files the write phase put in it go.
///
/// This is the undo's second state and the one the arm above must not be taken
/// for. A person who ran `mkdir release` first gets their directory back, empty.
#[test]
fn a_write_that_fails_empties_the_output_directory_the_caller_made() {
    let scratch = Scratch::new("failed-write-empty");
    let root = publisher_that_fails_inside_the_write(&scratch);
    let out = scratch.path().join("artifact");
    std::fs::create_dir_all(&out).expect("the caller makes it");

    let refused = package::publish(&root, "acme/fixture", &out).expect_err("it does not publish");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        out.is_dir(),
        "the caller's own directory was removed: {message}"
    );
    assert_eq!(
        walk_files(&out),
        0,
        "the files the write phase put in the caller's directory are still there"
    );
}

/// A `contents` value that is not a path is refused, and nothing is published.
///
/// A sequence under `contents` used to be skipped by the reachability check with
/// no reading at all: not for existence and not for escape. So a manifest could
/// carry `../../secrets.yml` under any key and publish at exit 0, with the `..`
/// verbatim in the manifest an adopter reads. Spec 7 (Publishing) says `bundles`
/// is the only key whose path is rewritten and that no published artifact
/// carries a path that leaves the package.
#[test]
fn a_contents_value_that_is_not_a_path_is_refused() {
    let scratch = Scratch::new("contents-sequence");
    let root = publisher(&scratch, None);
    scratch.write(
        "publisher/packages/acme-fixture/package.yml",
        "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n  bundles: \
         ../../library\n  conformance: [\"../../secrets.yml\", \"nosuch.yml\"]\n",
    );

    let refused = package::publish(&root, "acme/fixture", &out_of(&scratch))
        .expect_err("it does not publish");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains("`contents.conformance`"),
        "the refusal does not name the key: {message}"
    );
    assert!(!out_of(&scratch).exists(), "an artifact was written anyway");
}

/// A `contents` path that leaves the package under any key but `bundles` is
/// refused, and the file it names is there.
///
/// The refusal existed before this and nothing in the workspace failed when it
/// was deleted, which is the same gap the case above was found through. The file
/// exists on disk, so the existence check cannot be what refuses it and the
/// escape rule is the only thing under test. `bundles` is the one key publishing
/// rewrites, and every other key would reach a consumer with the `..` in it.
#[test]
fn a_contents_path_that_leaves_the_package_is_refused_under_every_key_but_bundles() {
    let scratch = Scratch::new("contents-escape");
    let root = publisher(&scratch, None);
    scratch.write(
        "publisher/packages/acme-fixture/package.yml",
        "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n  bundles: \
         ../../library\n  conformance: ../../library/extra/bundle.yml\n",
    );
    assert!(
        root.join("packages/acme-fixture/../../library/extra/bundle.yml")
            .exists(),
        "the case is testing the existence check rather than the escape rule"
    );

    let refused = package::publish(&root, "acme/fixture", &out_of(&scratch))
        .expect_err("it does not publish");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains("`contents.conformance`") && message.contains("outside the package"),
        "the refusal is not the escape rule: {message}"
    );
    assert!(!out_of(&scratch).exists(), "an artifact was written anyway");
}

/// A file whose name is not UTF-8 stops the publish, rather than falling out of
/// the artifact without a word.
///
/// It used to be skipped by the read of the package tree, so the file was in the
/// package, absent from the artifact, absent from the release record, absent
/// from the count the verb prints, and absent from the exit code. A publisher
/// whose claim is that everything is read before anything is written cannot drop
/// a file it never read. The positive half of the case is the same publisher
/// with the file removed, which publishes.
#[cfg(unix)]
#[test]
fn a_file_whose_name_is_not_utf8_is_refused_rather_than_dropped() {
    use std::os::unix::ffi::OsStrExt;

    let scratch = Scratch::new("not-utf8");
    let root = publisher(&scratch, None);
    let name = std::ffi::OsStr::from_bytes(b"bad\xffname.yml");
    let at = root.join("packages/acme-fixture").join(name);
    std::fs::write(&at, "x").expect("the file is written");

    let refused = package::publish(&root, "acme/fixture", &out_of(&scratch))
        .expect_err("it does not publish");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains("not UTF-8"),
        "the refusal does not say what is wrong with the name: {message}"
    );
    assert!(!out_of(&scratch).exists(), "an artifact was written anyway");

    std::fs::remove_file(&at).expect("the file goes");
    package::publish(&root, "acme/fixture", &out_of(&scratch))
        .expect("the same package without it publishes");
}

/// The output path each case above hands the verb.
fn out_of(scratch: &Scratch) -> PathBuf {
    scratch.path().join("artifact")
}

/// An `--out` that holds a file is refused, with the message it has always had,
/// and the file is untouched.
///
/// The precondition is what makes the undo total, so it is the one thing here
/// that a fix to the undo must not weaken. A publish into a directory somebody
/// else is using is refused before anything is read.
#[test]
fn an_output_directory_that_holds_a_file_is_refused_and_the_file_survives() {
    let scratch = Scratch::new("occupied-out");
    let root = publisher(&scratch, None);
    let out = scratch.path().join("artifact");
    std::fs::create_dir_all(&out).expect("the caller's directory is made");
    std::fs::write(out.join("theirs.txt"), "the caller's own file").expect("their file is written");

    let refused = package::publish(&root, "acme/fixture", &out).expect_err("it does not publish");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains("the output directory holds files already"),
        "the precondition's message moved: {message}"
    );
    assert_eq!(
        std::fs::read_to_string(out.join("theirs.txt")).expect("their file reads"),
        "the caller's own file"
    );
    assert_eq!(walk_files(&out), 1, "the publish wrote beside the file");
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
