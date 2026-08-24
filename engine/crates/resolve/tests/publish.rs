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

/// The same publisher, with the name its taxonomy source declares set by the
/// caller.
///
/// The manifest stays at `acme/fixture` whatever this is. A package states its
/// name twice as well as its version, and the two names sit under two different
/// keys: `package:` in the manifest that spec 7 gives it, and `taxonomy:` at the
/// root of the source, where the meta-schema requires one and forbids a
/// `package:` beside it. This is the knob that moves one without the other.
///
/// It edits the file [`publisher_at`] wrote rather than taking a fourth
/// positional parameter, so the fifteen call sites of that helper are untouched.
fn publisher_named(scratch: &Scratch, source: &str) -> PathBuf {
    let root = publisher_at(scratch, None, "1.0.0");
    let path = root.join("packages/acme-fixture/taxonomy.yml");
    let text = std::fs::read_to_string(&path).expect("the source was just written");
    let moved = text.replace("taxonomy: acme/fixture", &format!("taxonomy: {source}"));
    assert_ne!(text, moved, "the source did not declare the name it was to");
    std::fs::write(&path, moved).expect("the source writes");
    root
}

/// A publisher of one named package, in a tree of its own.
///
/// [`publisher_at`] has fifteen call sites and every one of them publishes
/// `acme/fixture` out of `publisher/`. The identity case needs two packages
/// under two names in one scratch tree — one the adopter already holds and one
/// an adversary hands them — so this takes the directory and the name. It ships
/// no bundles, because nothing here resolves the result.
fn publisher_of(scratch: &Scratch, at: &str, package: &str) -> PathBuf {
    let directory = package.replace('/', "-");
    scratch.write(
        &format!("{at}/packages/{directory}/package.yml"),
        &format!("package: {package}\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n"),
    );
    scratch.write(
        &format!("{at}/packages/{directory}/taxonomy.yml"),
        &TAXONOMY.replace("taxonomy: acme/fixture", &format!("taxonomy: {package}")),
    );
    scratch.path().join(at)
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

/// A record whose header renames the artifact does not steer the vendor target,
/// and the adopter's package of that name survives.
///
/// This is the case the pin cannot catch, and it is the mirror of the one above.
/// There the whole artifact was republished and the pin refused it. Here the
/// artifact is honest, the pin is the honest one, and the four header lines of
/// `release.yml` above the digest are the part no digest covers — so an
/// adversary rewrites the name in them, `release::verify` passes, and the target
/// directory the vendor deletes and rewrites was named by the edit.
///
/// **The victim is vendored rather than written.** A `packages/acme-victim/`
/// made by hand carries no release record, so `vendor` refuses it on the
/// maintained-package guard — `is_err` with no fix in the tree at all, which is
/// a green test measuring nothing. That is why the assertions below are on the
/// message and not on the shape of the result.
///
/// **The digest is asserted unchanged before the refusal is.** Without that
/// line, a later change that put the header inside the digest would keep this
/// test green while it measured a pin mismatch — the test would outlive the
/// thing it was written for and say nothing about it.
#[test]
fn a_record_that_renames_the_artifact_does_not_steer_the_vendor_target() {
    let scratch = Scratch::new("record-renames");
    let adopter = scratch.path().join("adopter");

    // The adopter holds `acme/victim`, vendored honestly.
    let victim_root = publisher_of(&scratch, "victim", "acme/victim");
    let victim_out = scratch.path().join("victim-artifact");
    let victim =
        package::publish(&victim_root, "acme/victim", &victim_out).expect("the victim publishes");
    package::vendor(&adopter, &victim_out, &victim.digest).expect("the victim vendors");
    let landed = adopter.join("packages/acme-victim/taxonomy.yml");
    let held = std::fs::read_to_string(&landed).expect("the victim's source landed");
    assert!(held.contains("taxonomy: acme/victim"), "{held}");

    // The adversary publishes their own package honestly, then edits the one
    // line of the record that no digest covers.
    let attacker_root = publisher_of(&scratch, "attacker", "acme/attacker");
    let attacker_out = scratch.path().join("attacker-artifact");
    let attacker = package::publish(&attacker_root, "acme/attacker", &attacker_out)
        .expect("the attacker publishes");
    let path = attacker_out.join(release::RECORD);
    let text = std::fs::read_to_string(&path).expect("the record is there");
    let forged = text.replace("package: acme/attacker", "package: acme/victim");
    assert_ne!(text, forged, "the record did not name what it was to");
    std::fs::write(&path, forged).expect("the record writes");

    // The edit moved no digest, so the attacker's own honest pin still verifies.
    let reread = release::at(&attacker_out).expect("the edited record reads");
    assert_eq!(reread.digest, attacker.digest, "the edit moved the digest");
    assert_eq!(reread.package, "acme/victim", "the edit did not take");
    assert!(release::verify(&attacker_out, &attacker.digest).is_ok());

    let refused = package::vendor(&adopter, &attacker_out, &attacker.digest)
        .expect_err("a record that renames its own artifact is refused");
    let message = headwater_resolve::render_errors(&refused);
    assert!(message.contains("package: acme/victim"), "{message}");
    assert!(message.contains("package: acme/attacker"), "{message}");
    assert!(message.contains(release::RECORD), "{message}");
    assert!(message.contains(package::MANIFEST), "{message}");

    // The adopter's real package is where it was, with the bytes it had.
    assert_eq!(
        std::fs::read_to_string(&landed).expect("the victim's source is still there"),
        held
    );
    assert!(!adopter.join("packages/acme-attacker").exists());
}

/// A record whose header states a version or an engine range the artifact's own
/// manifest does not is refused on the same read.
///
/// Neither field steers a write, so neither carries the harm the name does. They
/// are held because `release::compute` derives all three from the manifest, so
/// no publisher writing a record that way can produce a disagreeing pair — and
/// because a stripped `requires_engine` moves the range refusal from the vendor,
/// which runs before the bytes land, to the adopter's next resolve.
#[test]
fn a_record_that_restates_the_version_or_the_engine_range_is_refused() {
    let scratch = Scratch::new("record-restates");

    let root = publisher(&scratch, Some(">=0 <9"));
    let out = scratch.path().join("artifact");
    let record = package::publish(&root, "acme/fixture", &out).expect("it publishes");
    let path = out.join(release::RECORD);
    let text = std::fs::read_to_string(&path).expect("the record is there");

    let bumped = text.replace("version: 1.0.0", "version: 9.9.9");
    assert_ne!(
        text, bumped,
        "the record did not carry the version it was to"
    );
    std::fs::write(&path, bumped).expect("the record writes");
    let adopter = scratch.path().join("adopter-version");
    let refused =
        package::vendor(&adopter, &out, &record.digest).expect_err("a restated version is refused");
    let message = headwater_resolve::render_errors(&refused);
    assert!(message.contains("version `9.9.9`"), "{message}");
    assert!(message.contains("`1.0.0`"), "{message}");

    let stripped: String = text
        .lines()
        .filter(|line| !line.contains("requires_engine"))
        .map(|line| format!("{line}\n"))
        .collect();
    assert_ne!(text, stripped, "the record declared no range to strip");
    std::fs::write(&path, stripped).expect("the record writes");
    let adopter = scratch.path().join("adopter-range");
    let refused =
        package::vendor(&adopter, &out, &record.digest).expect_err("a dropped range is refused");
    let message = headwater_resolve::render_errors(&refused);
    assert!(message.contains("no `requires_engine`"), "{message}");
    assert!(message.contains(">=0 <9"), "{message}");
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

/// A publisher whose package directory is named `source` and whose manifest
/// declares whatever name the caller hands it.
///
/// The directory is deliberately not named after the package. The four cases
/// below publish names that cannot be directory names at all — `..`, `.`, a
/// YAML null — so a helper that derived the directory from the name, the way
/// [`publisher_of`] does, could not lay the publisher's own tree out. `find`
/// matches the string in the manifest and never the directory that carries it,
/// which is what makes one fixed directory enough.
fn publisher_declaring(scratch: &Scratch, at: &str, name: &str) -> PathBuf {
    publisher_declaring_at(scratch, at, name, "1.0.0")
}

/// The same publisher, at the version the caller hands it.
///
/// **Both version keys move together**, because a package that states two
/// versions of itself is refused at publish and the upgrade case needs a second
/// artifact that publishes. [`publisher_at`] is the helper that moves one
/// without the other, and it exists to reach that refusal rather than to pass
/// it.
fn publisher_declaring_at(scratch: &Scratch, at: &str, name: &str, version: &str) -> PathBuf {
    scratch.write(
        &format!("{at}/packages/source/package.yml"),
        &format!("package: {name}\nversion: {version}\ncontents:\n  taxonomy: taxonomy.yml\n"),
    );
    scratch.write(
        &format!("{at}/packages/source/taxonomy.yml"),
        &TAXONOMY
            .replace("taxonomy: acme/fixture", &format!("taxonomy: {name}"))
            .replace("version: 1.0.0", &format!("version: {version}")),
    );
    scratch.path().join(at)
}

/// A package named `..` does not reach the adopter's own root.
///
/// **The adopter must not already hold a `packages/` directory, and the case
/// asserts it rather than assuming it.** `Path::exists()` asks the operating
/// system, which resolves a `..` only under a directory that is on disk, so
/// `<root>/packages/..` does not exist on an adopter that has never vendored
/// anything. That single fact decides which arm of `vendor` runs: with no
/// `packages/` the target looks absent and the artifact scatters over the
/// adopter's root, and with a `packages/` the maintained-package guard fires
/// first and refuses for a reason that has nothing to do with the name. A
/// setup that hands the adopter a `packages/` — the natural thing to write,
/// and what every other case in this file does — passes with no fix in the
/// tree at all.
///
/// **The digest is asserted unchanged before the refusal is**, for the reason
/// [`a_record_that_renames_the_artifact_does_not_steer_the_vendor_target`]
/// states: an artifact that no longer verifies would refuse for the pin and
/// the case would stop measuring the name.
///
/// The assertion on `release.yml` is what breaks the chain. Left in place, that
/// record is one of the two things a second vendor needs to take the deleting
/// arm over the whole root.
#[test]
fn a_package_named_dot_dot_does_not_reach_the_adopters_own_root() {
    let scratch = Scratch::new("name-parent");
    let root = publisher_declaring(&scratch, "publisher", "..");
    let out = scratch.path().join("artifact");
    let record = package::publish(&root, "..", &out).expect("it publishes");
    assert!(
        release::verify(&out, &record.digest).is_ok(),
        "the artifact does not verify against its own digest, so what follows measures the pin"
    );

    let adopter = scratch.path().join("adopter");
    std::fs::create_dir_all(&adopter).expect("the adopter root is made");
    std::fs::write(adopter.join("keepme.txt"), "the adopter's own file\n").expect("it writes");
    assert!(
        !adopter.join(package::PACKAGES).exists(),
        "the adopter already holds `packages/`, so this measures the maintained-package guard"
    );

    let refused = package::vendor(&adopter, &out, &record.digest)
        .expect_err("a package name that is not a name is refused");
    let message = headwater_resolve::render_errors(&refused);
    assert!(message.contains(package::MANIFEST), "{message}");
    assert!(message.contains("package: .."), "{message}");

    assert!(
        adopter.join("keepme.txt").is_file(),
        "the adopter lost a file"
    );
    assert!(
        !adopter.join(release::RECORD).exists(),
        "a release record at the adopter's root is half of what a second vendor needs to \
         delete the whole of it"
    );
    assert!(
        !adopter.join(package::PACKAGES).exists(),
        "nothing was created, because the refusal runs before the target is named"
    );
}

/// The same name, over an adopter that already holds both of the things a
/// deleting vendor needs.
///
/// **The precondition is built by hand and never by running the case above.**
/// A first vendor of this artifact used to create `packages/` and leave a
/// `release.yml` at the adopter's root, which is exactly the pair that sends
/// `remove_dir_all` at the root on the second run. The moment the guard lands
/// that route closes, so an arm that seeded itself by vendoring once would go
/// vacuous and green while measuring nothing at all.
///
/// The message assertion is what keeps it honest. `is_err()` alone passes here
/// on the maintained-package guard, which refuses for a reason of its own and
/// leaves the harm untouched on every other shape.
#[test]
fn a_package_named_dot_dot_does_not_reach_a_root_that_already_holds_packages() {
    let scratch = Scratch::new("name-parent-seeded");
    let root = publisher_declaring(&scratch, "publisher", "..");
    let out = scratch.path().join("artifact");
    let record = package::publish(&root, "..", &out).expect("it publishes");
    assert!(
        release::verify(&out, &record.digest).is_ok(),
        "the artifact does not verify against its own digest, so what follows measures the pin"
    );

    let adopter = scratch.path().join("adopter");
    std::fs::create_dir_all(adopter.join(package::PACKAGES)).expect("the directory is made");
    std::fs::create_dir_all(adopter.join("docs")).expect("the directory is made");
    std::fs::write(adopter.join("keepme.txt"), "the adopter's own file\n").expect("it writes");
    std::fs::write(adopter.join("docs/spec.md"), "# the adopter's corpus\n").expect("it writes");
    std::fs::copy(out.join(release::RECORD), adopter.join(release::RECORD))
        .expect("the record copies");
    assert!(
        adopter.join(package::PACKAGES).is_dir(),
        "without `packages/` on disk the operating system cannot resolve the `..` under it"
    );
    assert!(
        release::at(&adopter).is_ok(),
        "without a record at the root the maintained-package guard answers instead"
    );

    let refused = package::vendor(&adopter, &out, &record.digest)
        .expect_err("a package name that is not a name is refused");
    let message = headwater_resolve::render_errors(&refused);
    assert!(message.contains("package: .."), "{message}");
    assert!(
        !message.contains("somebody maintains"),
        "the maintained-package guard answered, so the precondition is wrong: {message}"
    );

    assert!(
        adopter.join("keepme.txt").is_file(),
        "the adopter lost a file"
    );
    assert!(
        adopter.join("docs/spec.md").is_file(),
        "the adopter lost its corpus"
    );
}

/// A package named `.` does not empty the adopter's `packages/` directory.
///
/// **The result type says nothing about this shape and the surviving bytes say
/// everything.** `packages/.` is `packages/`, and `remove_dir_all` on a path
/// ending in `.` returns `EINVAL` *after* it has emptied the directory. So
/// before the guard this call returned `Err` and destroyed the adopter's whole
/// `packages/` tree in the same breath, and a case that asserted only "it
/// refused" scored that as a pass.
///
/// It is also the shape a path-containment guard cannot see: `packages/.` is
/// strictly under `packages/` by any reading of the paths, so only a rule about
/// the name refuses it.
#[test]
fn a_package_named_dot_does_not_empty_the_adopters_packages_directory() {
    let scratch = Scratch::new("name-dot");
    let root = publisher_declaring(&scratch, "publisher", ".");
    let out = scratch.path().join("artifact");
    let record = package::publish(&root, ".", &out).expect("it publishes");
    assert!(
        release::verify(&out, &record.digest).is_ok(),
        "the artifact does not verify against its own digest, so what follows measures the pin"
    );

    let adopter = scratch.path().join("adopter");
    let packages = adopter.join(package::PACKAGES);
    std::fs::create_dir_all(&packages).expect("the directory is made");
    std::fs::write(packages.join("keepme.txt"), "the adopter's own file\n").expect("it writes");
    std::fs::copy(out.join(release::RECORD), packages.join(release::RECORD))
        .expect("the record copies");
    assert!(
        release::at(&packages).is_ok(),
        "without a record in `packages/` the maintained-package guard answers instead, and \
         nothing reaches the removal this case is about"
    );

    let refused = package::vendor(&adopter, &out, &record.digest)
        .expect_err("a package name that is not a name is refused");

    assert!(
        packages.join("keepme.txt").is_file(),
        "`packages/` was emptied behind the refusal, which is what the result type hides"
    );
    let message = headwater_resolve::render_errors(&refused);
    assert!(message.contains(package::MANIFEST), "{message}");
    assert!(message.contains("package: ."), "{message}");
}

/// A manifest whose `package:` is a YAML null does not vendor into
/// `packages/~`.
///
/// This engine hands back a scalar's source text and the source text of a null
/// is the literal `~`, so `is_empty()` is false and the value travels as a
/// one-character name. The grammar refuses the result. It teaches the reader
/// nothing about absent versus null, which is
/// [#298](https://github.com/headwater-ai/headwater/issues/298) and stays open.
#[test]
fn a_manifest_whose_package_is_a_yaml_null_does_not_vendor_into_a_directory() {
    let scratch = Scratch::new("name-null");
    scratch.write(
        "publisher/packages/source/package.yml",
        "package:\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n",
    );
    scratch.write(
        "publisher/packages/source/taxonomy.yml",
        &TAXONOMY.replace("taxonomy: acme/fixture", "taxonomy:"),
    );
    let root = scratch.path().join("publisher");
    let out = scratch.path().join("artifact");
    let record = package::publish(&root, "~", &out).expect("it publishes");
    assert!(
        release::verify(&out, &record.digest).is_ok(),
        "the artifact does not verify against its own digest, so what follows measures the pin"
    );

    let adopter = scratch.path().join("adopter");
    let refused = package::vendor(&adopter, &out, &record.digest)
        .expect_err("a package name that is not a name is refused");
    let message = headwater_resolve::render_errors(&refused);
    assert!(message.contains("package: ~"), "{message}");
    assert!(
        !adopter.join(package::PACKAGES).join("~").exists(),
        "the artifact landed in a directory called `~`"
    );
}

/// The directories under an adopter's `packages/`, sorted.
///
/// The three cases below assert what `packages/` holds as well as what `find`
/// answers, because a refusal that left a second tree beside the first would
/// answer every lookup correctly and still have written where it must not.
fn packages_under(root: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(root.join(package::PACKAGES))
        .expect("the adopter holds `packages/`")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// Two package names inside the grammar flatten to one directory, and the
/// second does not delete the first.
///
/// `acme/my-taxonomy` and `acme-my/taxonomy` are both names the grammar
/// accepts, and `declared.replace('/', "-")` sends both of them to
/// `packages/acme-my-taxonomy`. The first vendor leaves a release record there,
/// so `release::at` succeeds, the maintained-package guard does not fire, and
/// the replace arm removed the first publisher's package before writing the
/// second — two honest publishers, no adversary, and both runs exiting 0. That
/// is [#320](https://github.com/headwater-ai/headwater/issues/320).
///
/// **[`publisher_declaring`] is the helper this needs and [`publisher_of`] is
/// not.** The latter derives the publisher's own directory from the name, so
/// two colliding names lay out over one path in one scratch tree and the second
/// publisher overwrites the first before either is published. The collision
/// under test would then be a collision in the fixture's own tree.
///
/// **The digest of each artifact is asserted before the refusal is**, for the
/// reason [`a_package_named_dot_dot_does_not_reach_the_adopters_own_root`]
/// states: an artifact that stopped verifying would refuse for the pin, and the
/// case would quietly stop measuring the name.
///
/// **The surviving package is read and not only found.** `find_version` reads
/// the manifest alone, so a removal that ran and then wrote a partial tree
/// answers it correctly. The taxonomy source beside the manifest is what says
/// the whole directory is still there.
#[test]
fn two_names_that_flatten_to_one_directory_do_not_delete_each_other() {
    let scratch = Scratch::new("collide");

    let first_root = publisher_declaring(&scratch, "first", "acme/my-taxonomy");
    let first_out = scratch.path().join("artifact-first");
    let first =
        package::publish(&first_root, "acme/my-taxonomy", &first_out).expect("the first publishes");
    assert!(
        release::verify(&first_out, &first.digest).is_ok(),
        "the artifact does not verify against its own digest, so what follows measures the pin"
    );

    let second_root = publisher_declaring(&scratch, "second", "acme-my/taxonomy");
    let second_out = scratch.path().join("artifact-second");
    let second = package::publish(&second_root, "acme-my/taxonomy", &second_out)
        .expect("the second publishes");
    assert!(
        release::verify(&second_out, &second.digest).is_ok(),
        "the artifact does not verify against its own digest, so what follows measures the pin"
    );

    let adopter = scratch.path().join("adopter");
    std::fs::create_dir_all(&adopter).expect("the adopter root is made");
    package::vendor(&adopter, &first_out, &first.digest).expect("the first vendor lands");
    assert_eq!(
        package::find_version(&adopter, "acme/my-taxonomy"),
        Some("1.0.0".to_string()),
        "the first package is not installed, so what follows measures nothing"
    );

    let refused = package::vendor(&adopter, &second_out, &second.digest)
        .expect_err("a second name that flattens onto an installed package is refused");
    let message = headwater_resolve::render_errors(&refused);

    // The refusal names both names and the directory they contend for, and
    // none of the three is a substring of either of the others.
    assert!(
        message.contains("acme/my-taxonomy"),
        "the package that is installed is not named:\n{message}"
    );
    assert!(
        message.contains("acme-my/taxonomy"),
        "the package the artifact declares is not named:\n{message}"
    );
    assert!(
        message.contains("packages/acme-my-taxonomy"),
        "the directory the two names contend for is not named:\n{message}"
    );

    // The first publisher's package is installed, and whole.
    assert_eq!(
        package::find_version(&adopter, "acme/my-taxonomy"),
        Some("1.0.0".to_string()),
        "the adopter's package is gone, which is the defect"
    );
    assert_eq!(
        package::find_version(&adopter, "acme-my/taxonomy"),
        None,
        "the refused artifact is installed"
    );
    let installed = adopter.join(package::PACKAGES).join("acme-my-taxonomy");
    let manifest =
        std::fs::read_to_string(installed.join(package::MANIFEST)).expect("the manifest is there");
    assert!(manifest.contains("package: acme/my-taxonomy"), "{manifest}");
    let source =
        std::fs::read_to_string(installed.join("taxonomy.yml")).expect("the source is there");
    assert!(source.contains("taxonomy: acme/my-taxonomy"), "{source}");

    // And the refusal wrote nothing anywhere else under `packages/`.
    assert_eq!(
        packages_under(&adopter),
        vec!["acme-my-taxonomy".to_string()],
        "the refusal left a second tree behind"
    );
}

/// After #320's rename remedy is followed a second time, the second collision
/// is refused rather than silently duplicated.
///
/// This extends [`two_names_that_flatten_to_one_directory_do_not_delete_each_other`]
/// past the point that case stops at. `widgets/core-schema` and
/// `widgets-core/schema` both flatten to `packages/widgets-core-schema`, so the
/// second name collides with the first exactly as `acme/my-taxonomy` and
/// `acme-my/taxonomy` do above. Renaming the target once, per the refusal's own
/// remedy, lets the second package land — that still has to work, or the
/// remedy is broken. Upgrading the *first* package then collides again,
/// because it still derives the directory the second package now holds.
/// Renaming the target aside a second time used to let that upgrade land too,
/// leaving two directories that both declare `widgets/core-schema` — one of
/// them, `packages/a-moved-aside`, holding the package the adopter renamed
/// away in the first place. That is [#354](https://github.com/headwater-ai/headwater/issues/354):
/// `find` would then answer from whichever of the two sorts first while
/// `vendor` reports installing the other. This case asserts that the second
/// rename is refused instead, naming the directory the package already
/// resolves from, and that `packages/` never ends with two directories
/// declaring one name.
///
/// **The permanent limitation stays, and is asserted rather than hidden.**
/// Nothing here lets `widgets/core-schema` upgrade past `1.0.0`: it derives a
/// directory `widgets-core/schema` now holds, and `vendor` only ever writes to
/// the name-derived directory. The adopter is told this at the vendor that
/// would have created the duplicate, which is the whole of what this issue
/// asks for.
#[test]
fn a_second_rename_around_a_collision_is_refused_rather_than_duplicated() {
    let scratch = Scratch::new("collide-twice");

    // Step 1: publish and vendor `widgets/core-schema` 1.0.0.
    let a1_root = publisher_declaring_at(&scratch, "a1", "widgets/core-schema", "1.0.0");
    let a1_out = scratch.path().join("artifact-a1");
    let a1 = package::publish(&a1_root, "widgets/core-schema", &a1_out).expect("1.0.0 a publishes");
    assert!(
        release::verify(&a1_out, &a1.digest).is_ok(),
        "the artifact does not verify against its own digest, so what follows measures the pin"
    );

    let adopter = scratch.path().join("adopter");
    std::fs::create_dir_all(&adopter).expect("the adopter root is made");
    package::vendor(&adopter, &a1_out, &a1.digest).expect("widgets/core-schema 1.0.0 vendors");
    assert_eq!(
        packages_under(&adopter),
        vec!["widgets-core-schema".to_string()],
        "the first vendor did not land where expected"
    );

    // Step 2: publish `widgets-core/schema` 1.0.0 and vendor it — refused by
    // #320's existing guard, because the target holds a different package.
    let b1_root = publisher_declaring_at(&scratch, "b1", "widgets-core/schema", "1.0.0");
    let b1_out = scratch.path().join("artifact-b1");
    let b1 = package::publish(&b1_root, "widgets-core/schema", &b1_out).expect("1.0.0 b publishes");
    assert!(
        release::verify(&b1_out, &b1.digest).is_ok(),
        "the artifact does not verify against its own digest, so what follows measures the pin"
    );
    package::vendor(&adopter, &b1_out, &b1.digest)
        .expect_err("widgets-core/schema collides with the installed widgets/core-schema");

    // The adopter follows the remedy: rename the target aside.
    let packages = adopter.join(package::PACKAGES);
    std::fs::rename(
        packages.join("widgets-core-schema"),
        packages.join("a-moved-aside"),
    )
    .expect("the first rename lands");

    // Vendoring `widgets-core/schema` again still has to land — this is
    // done-when #2, the one-time remedy still works.
    package::vendor(&adopter, &b1_out, &b1.digest).expect("the remedy still works, followed once");
    assert_eq!(
        package::find_version(&adopter, "widgets/core-schema"),
        Some("1.0.0".to_string()),
        "the moved package no longer resolves after the remedy"
    );
    assert_eq!(
        package::find_version(&adopter, "widgets-core/schema"),
        Some("1.0.0".to_string()),
        "the remedied vendor did not land"
    );

    // Step 3: publish `widgets/core-schema` 2.0.0 and vendor it — refused,
    // because the target now holds `widgets-core/schema`.
    let a2_root = publisher_declaring_at(&scratch, "a2", "widgets/core-schema", "2.0.0");
    let a2_out = scratch.path().join("artifact-a2");
    let a2 = package::publish(&a2_root, "widgets/core-schema", &a2_out).expect("2.0.0 publishes");
    assert!(
        release::verify(&a2_out, &a2.digest).is_ok(),
        "the artifact does not verify against its own digest, so what follows measures the pin"
    );
    let refused = package::vendor(&adopter, &a2_out, &a2.digest)
        .expect_err("the target holds widgets-core/schema, not widgets/core-schema");
    let message = headwater_resolve::render_errors(&refused);
    assert!(message.contains("widgets-core/schema"), "{message}");
    assert!(message.contains("widgets/core-schema"), "{message}");

    // Step 4: the adopter follows the same remedy a second time.
    std::fs::rename(
        packages.join("widgets-core-schema"),
        packages.join("b-moved-aside"),
    )
    .expect("the second rename lands");

    // Vendoring `widgets/core-schema` 2.0.0 again must now be refused — today,
    // pre-fix, this exits `Ok` and is the regression this issue closes.
    let refused = package::vendor(&adopter, &a2_out, &a2.digest).expect_err(
        "widgets/core-schema already resolves from a-moved-aside, and vendoring here would          leave two directories declaring it",
    );
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains("a-moved-aside"),
        "the refusal does not name where the package already resolves from:\n{message}"
    );
    assert!(message.contains("widgets/core-schema"), "{message}");

    // Step 5: exactly two directories stand, and neither declares the other's
    // name — packages/ never ends with two directories declaring one name.
    assert_eq!(
        packages_under(&adopter),
        vec!["a-moved-aside".to_string(), "b-moved-aside".to_string()],
        "the refused vendor left a third directory, or removed one of the first two"
    );
    let a_moved = std::fs::read_to_string(packages.join("a-moved-aside").join(package::MANIFEST))
        .expect("a-moved-aside carries a manifest");
    assert!(
        a_moved.contains("package: widgets/core-schema"),
        "{a_moved}"
    );
    let b_moved = std::fs::read_to_string(packages.join("b-moved-aside").join(package::MANIFEST))
        .expect("b-moved-aside carries a manifest");
    assert!(
        b_moved.contains("package: widgets-core/schema"),
        "{b_moved}"
    );

    // Step 6: the moved package is unaffected, and permanently stuck at
    // 1.0.0 — the accepted, now clearly reported limitation.
    assert_eq!(
        package::find_version(&adopter, "widgets/core-schema"),
        Some("1.0.0".to_string()),
        "the refused vendor changed what the moved package resolves to"
    );
}

/// A later version of the package that is installed still replaces it.
///
/// **This case separates an executed replace from a silent no-op, and that is
/// what it adds.** [`vendoring_over_a_maintained_package_is_refused`] vendors
/// one artifact twice, so the bytes it expects are the bytes that are already
/// there and a guard that quietly did nothing would satisfy it. Moving the
/// version is what makes the replace arm say whether it ran.
///
/// **A blanket refusal is not what this alone catches, and the measurement says
/// so.** Patch the equal arm of `holds_the_same_package` to refuse whatever it
/// finds and the target reports `40 passed; 2 failed`: this case, and
/// [`vendoring_over_a_maintained_package_is_refused`], which ends in
/// `.expect("the second replaces it")` and so observes an `Err` whatever the
/// bytes are. A refusal is visible to that case. A guard that returns `Ok` and
/// lets nothing happen is visible only to this one, which is the reason to keep
/// it rather than the reason it was written.
///
/// Both version keys move together, because a package that states two versions
/// of itself is refused at publish. [`publisher_declaring_at`] is the knob.
#[test]
fn a_later_version_of_the_installed_package_still_replaces_it() {
    let scratch = Scratch::new("upgrade");

    let root = publisher_declaring_at(&scratch, "publisher", "acme/fixture", "1.0.0");
    let out = scratch.path().join("artifact-1");
    let first = package::publish(&root, "acme/fixture", &out).expect("1.0.0 publishes");

    let adopter = scratch.path().join("adopter");
    std::fs::create_dir_all(&adopter).expect("the adopter root is made");
    package::vendor(&adopter, &out, &first.digest).expect("the first vendor lands");
    assert_eq!(
        package::find_version(&adopter, "acme/fixture"),
        Some("1.0.0".to_string())
    );

    let later_root = publisher_declaring_at(&scratch, "later", "acme/fixture", "2.0.0");
    let later_out = scratch.path().join("artifact-2");
    let later = package::publish(&later_root, "acme/fixture", &later_out).expect("2.0.0 publishes");
    assert_ne!(
        first.digest, later.digest,
        "the two artifacts are the same bytes, so a vendor that did nothing would pass"
    );

    package::vendor(&adopter, &later_out, &later.digest)
        .expect("a later version of the same package replaces the one that is installed");
    assert_eq!(
        package::find_version(&adopter, "acme/fixture"),
        Some("2.0.0".to_string()),
        "the upgrade did not land, so the guard refuses a package its own name"
    );
    assert_eq!(
        packages_under(&adopter),
        vec!["acme-fixture".to_string()],
        "the upgrade wrote a second tree instead of replacing the first"
    );
}

/// The resident's manifest decides, and its release record does not.
///
/// A vendored directory states its name twice: `package.yml`, which the release
/// digest covers, and the header of `release.yml`, which the digest cannot cover
/// because it is one of the lines the digest is written over.
/// [`a_record_that_renames_the_artifact_does_not_steer_the_vendor_target`] holds
/// that distinction for the *artifact* being vendored. This holds it for the
/// directory already installed, which is the other end of the same comparison
/// and reaches it through [`holds_the_same_package`] rather than through
/// [`identity`].
///
/// **Without this the choice is pinned only by absence.** Every other case here
/// reaches the guard over a resident whose two names agree, or over one with no
/// readable manifest at all, so a guard that read the record's header instead
/// would pass all of them. Here the two names disagree, and only one answer
/// leaves the upgrade running.
///
/// The version moves, so what is asserted is that the replace ran rather than
/// that nothing was refused.
#[test]
fn the_resident_manifest_decides_and_its_record_header_does_not() {
    let scratch = Scratch::new("resident-header");

    let root = publisher_declaring_at(&scratch, "publisher", "acme/fixture", "1.0.0");
    let out = scratch.path().join("artifact-1");
    let first = package::publish(&root, "acme/fixture", &out).expect("1.0.0 publishes");

    let adopter = scratch.path().join("adopter");
    std::fs::create_dir_all(&adopter).expect("the adopter root is made");
    package::vendor(&adopter, &out, &first.digest).expect("the first vendor lands");

    // Rewrite the installed record's header to name a different package, and
    // leave the manifest beside it alone. The digest covers the manifest and
    // not this line, which is why the two can disagree at all.
    let installed = adopter.join(package::PACKAGES).join("acme-fixture");
    let record = installed.join(release::RECORD);
    let text = std::fs::read_to_string(&record).expect("the record is there");
    let forged = text.replace("package: acme/fixture", "package: acme/somebody-else");
    assert_ne!(text, forged, "the record header did not name the package");
    std::fs::write(&record, forged).expect("the record writes");
    assert!(
        release::at(&installed).is_ok(),
        "the edited record no longer reads, so the guard is never reached"
    );
    assert!(
        std::fs::read_to_string(installed.join(package::MANIFEST))
            .expect("the manifest is there")
            .contains("package: acme/fixture"),
        "the manifest moved with the record, so the two no longer disagree"
    );

    let later_root = publisher_declaring_at(&scratch, "later", "acme/fixture", "2.0.0");
    let later_out = scratch.path().join("artifact-2");
    let later = package::publish(&later_root, "acme/fixture", &later_out).expect("2.0.0 publishes");

    package::vendor(&adopter, &later_out, &later.digest)
        .expect("the manifest names this package, so the upgrade lands");
    assert_eq!(
        package::find_version(&adopter, "acme/fixture"),
        Some("2.0.0".to_string()),
        "the guard read the record's header, which the release digest does not cover"
    );
    assert_eq!(
        packages_under(&adopter),
        vec!["acme-fixture".to_string()],
        "the upgrade wrote a second tree instead of replacing the first"
    );
}

/// A vendored directory whose manifest cannot be read is not removed either.
///
/// The guard reads the `package:` of the directory that is there. A directory
/// that carries a release record and no readable manifest states no name to
/// compare, so it is refused rather than assumed to be the package the artifact
/// declares. Nothing this verb writes reaches that state: `identity` reads the
/// artifact's own manifest and refuses an artifact without one before a byte is
/// written, so a directory in this state was not written here.
///
/// The release record is asserted present first. Without one the
/// maintained-package guard answers instead, for a reason of its own, and the
/// case would measure that refusal rather than this one.
#[test]
fn a_vendored_directory_that_declares_no_readable_name_is_not_replaced() {
    let scratch = Scratch::new("no-manifest");

    let root = publisher_declaring(&scratch, "publisher", "acme/fixture");
    let out = scratch.path().join("artifact");
    let record = package::publish(&root, "acme/fixture", &out).expect("it publishes");

    let adopter = scratch.path().join("adopter");
    std::fs::create_dir_all(&adopter).expect("the adopter root is made");
    package::vendor(&adopter, &out, &record.digest).expect("the first vendor lands");

    let installed = adopter.join(package::PACKAGES).join("acme-fixture");
    std::fs::remove_file(installed.join(package::MANIFEST)).expect("the manifest is removed");
    assert!(
        release::at(&installed).is_ok(),
        "without a release record the maintained-package guard answers instead, and nothing \
         reaches the comparison this case is about"
    );

    let refused = package::vendor(&adopter, &out, &record.digest)
        .expect_err("a directory that declares no readable name is not removed");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains("packages/acme-fixture/package.yml"),
        "the file that cannot be read is not named:\n{message}"
    );
    assert!(
        installed.join("taxonomy.yml").is_file(),
        "the rest of the directory was removed behind the refusal"
    );
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

/// A package states its name twice, and the two names must be one name.
///
/// This is the same shape as the version case above, one key up, and one thing
/// about it cannot be copied from there. Both version keys are called `version`,
/// so that message says "version" once and a reader knows where to look. The
/// name keys are different keys: `package:` in the manifest and `taxonomy:` at
/// the root of the source, where the meta-schema forbids a `package:`. A message
/// that named the value and not the key would send a reader hunting for a
/// `package:` key in `taxonomy.yml` that no taxonomy source may declare, so the
/// refusal names both keys as well as both files and both names.
///
/// The version case has two arms and this has one, for a reason that is a
/// property of `find` rather than an omission. A consumer pins a version, so
/// which of the two numbers it wrote decided which refusal it got, and the
/// second arm is what holds the comparison in front of the pin. A consumer
/// cannot pin either name: `find` searches `packages/` by the manifest key
/// alone, so a consumer naming the source's name never reaches this comparison
/// and is told "no package under `packages/` declares" instead. There is no
/// `find`-side arm to write, and asserting that this refusal is not that one is
/// what stands in place of it.
#[test]
fn a_package_that_states_two_names_of_itself_is_refused() {
    let scratch = Scratch::new("two-names");
    // The manifest stays at acme/fixture and the taxonomy source goes to
    // acme/other.
    let root = publisher_named(&scratch, "acme/other");
    takes(&scratch, "1.0.0");

    let declaration = package::consumer(&root).expect("it reads");
    let refused = package::sources(&root, &declaration)
        .expect_err("a package with two names of itself does not load");
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
    // Both keys, because they are two different keys.
    assert!(
        message.contains("package: acme/fixture"),
        "the manifest's key is not named, so a reader cannot find the value:\n{message}"
    );
    assert!(
        message.contains("taxonomy: acme/other"),
        "the source's key is not named, so a reader looks for a `package:` key the \
         meta-schema forbids:\n{message}"
    );
    // And neither of the two refusals it could be mistaken for. `find` answers
    // about a name nothing declares, and the pin comparison about a consumer's
    // number; this is about the package being two things at once.
    assert!(
        !message.contains("no package under"),
        "the lookup answered first, so the package was never held to itself:\n{message}"
    );
    assert!(
        !message.contains("this takes"),
        "the pin comparison answered first, so the package was never held to itself:\n{message}"
    );
}

/// The same disagreement stops a publish, before a byte reaches an artifact.
///
/// `publish` does not resolve for a consumer, so it does not pass through the
/// comparison above. It copies both files into the artifact and the release
/// digest covers both, so without its own reading of this a publisher seals two
/// names under one digest and every adopter of that artifact receives a package
/// that says two things about what it is.
#[test]
fn a_publish_of_a_package_that_states_two_names_is_refused() {
    let scratch = Scratch::new("two-names-publish");
    let root = publisher_named(&scratch, "acme/other");
    let out = scratch.path().join("artifact");

    let refused =
        package::publish(&root, "acme/fixture", &out).expect_err("the publish does not run");
    let message = headwater_resolve::render_errors(&refused);
    assert!(message.contains("package.yml"), "{message}");
    assert!(message.contains("taxonomy.yml"), "{message}");
    assert!(message.contains("package: acme/fixture"), "{message}");
    assert!(message.contains("taxonomy: acme/other"), "{message}");
    assert!(
        !out.join(package::MANIFEST).exists(),
        "the artifact was written anyway"
    );
}

/// An artifact that already carries two names is refused on the path a fetched
/// directory is read by.
///
/// This engine cannot publish such an artifact any more, so the only publisher
/// that can hand one over is a publisher on some other engine. That is the case
/// that matters for a fetched package: the release digest covers both files and
/// proves the bytes, never that the bytes agree. It matters more here than one
/// key down, because the vendor target directory is derived from the manifest
/// name — so without this an adopter lands a source naming one thing in a
/// directory named after another, and their own resolve passes.
#[test]
fn a_fetched_artifact_that_states_two_names_is_refused() {
    let scratch = Scratch::new("two-names-fetched");
    let root = publisher(&scratch, None);
    let out = scratch.path().join("artifact");
    package::publish(&root, "acme/fixture", &out).expect("the honest artifact publishes");

    // What another publisher's engine could have written: the manifest at
    // acme/fixture and the taxonomy source beside it at acme/other.
    let source = out.join("taxonomy.yml");
    let text = std::fs::read_to_string(&source).expect("it is there");
    std::fs::write(
        &source,
        text.replace("taxonomy: acme/fixture", "taxonomy: acme/other"),
    )
    .expect("it writes");

    consumer(&scratch, "sha256:0");
    let consumer_root = scratch.path().join("consumer");
    let declaration = package::consumer(&consumer_root).expect("it reads");
    let manifest = package::manifest_at(&out).expect("the manifest reads");
    let refused = package::sources_at(&consumer_root, &out, &manifest, &declaration)
        .expect_err("the fetched artifact does not load");
    let message = headwater_resolve::render_errors(&refused);
    assert!(message.contains("package: acme/fixture"), "{message}");
    assert!(message.contains("taxonomy: acme/other"), "{message}");
    assert!(message.contains("taxonomy.yml"), "{message}");
}

/// The two declarations of this repository's own package name one thing.
///
/// The cases above prove the refusal fires. This one proves it is not firing on
/// the tree it ships in, and it reads both files under their own keys rather
/// than asserting the literal `headwater/standard`, so a rename that moves one
/// and forgets the other fails here as well as at the gate.
#[test]
fn the_package_in_this_repository_states_one_name_in_both_files() {
    let root = Path::new("../../..");
    let directory = root.join(package::PACKAGES).join("headwater-standard");
    let manifest =
        std::fs::read_to_string(directory.join(package::MANIFEST)).expect("the manifest is there");
    let source =
        std::fs::read_to_string(directory.join("taxonomy.yml")).expect("the source is there");

    let named = |text: &str, key: &str| {
        text.lines()
            .find_map(|line| line.strip_prefix(key))
            .map(str::to_string)
            .unwrap_or_else(|| panic!("no root `{key}` is declared"))
    };
    assert_eq!(
        named(&manifest, "package: "),
        named(&source, "taxonomy: "),
        "packages/headwater-standard states two names of itself"
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
///
/// It names *both* paths. The assertion took either one while the check
/// returned on the first bad key, and a publisher then had to run twice to see
/// two defects. `reachable` collects now, which is what makes the pair
/// assertable.
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
        message.contains("nosuch-conformance.yml") && message.contains("nosuch-bundles"),
        "the refusal does not name both declared paths: {message}"
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

    // `package::publish` by name finds `packages/headwater-standard/` first,
    // and #366 made that refuse: the directory carries a release record, so
    // it is a vendored copy and not the maintained source. The maintained
    // source is `taxonomy-source/headwater-standard/`, and `--from` is how a
    // real publish of this repository's own package reaches it.
    let record =
        package::publish_from(root, &root.join("taxonomy-source/headwater-standard"), &out)
            .expect("it publishes");

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

/// A `contents` path that leaves the package and returns to a location inside
/// it publishes.
///
/// The escape rule above used to be lexical: any `..` in the declared string
/// refused, whether or not the path it named ever left the package. [`settled`]
/// judges where the path resolves rather than the string a manifest wrote, and
/// this pins the direction that rule must not over-refuse in. [#303].
///
/// [#303]: https://github.com/headwater-ai/headwater/issues/303
#[test]
fn a_contents_path_that_leaves_and_returns_inside_the_package_publishes() {
    let scratch = Scratch::new("leaves-and-returns");
    let root = publisher(&scratch, None);
    scratch.write("publisher/packages/acme-fixture/sub/inside.yml", "x: 1\n");
    scratch.write(
        "publisher/packages/acme-fixture/package.yml",
        "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: sub/../taxonomy.yml\n  \
         bundles: ../../library\n",
    );

    let record = package::publish(&root, "acme/fixture", &out_of(&scratch)).expect("it publishes");
    assert_eq!(record.package, "acme/fixture");
}

/// A `contents` value that names no `..` at all is still refused when it is a
/// symlink resolving outside the package.
///
/// [#303]'s first mechanism: the escape check read the declared string, and a
/// symlink's declared name never carries the escape its target does.
/// `taxonomy.yml` is replaced with a link to a file elsewhere in the scratch
/// tree, so the string a manifest author wrote and the bytes a publish would
/// carry disagree about where the file is.
///
/// [#303]: https://github.com/headwater-ai/headwater/issues/303
#[cfg(unix)]
#[test]
fn a_contents_path_that_is_a_symlink_resolving_outside_the_package_is_refused() {
    let scratch = Scratch::new("contents-symlink-escape");
    let root = publisher(&scratch, None);
    scratch.write(
        "secret/stolen.yml",
        "taxonomy: acme/fixture\nversion: 1.0.0\n",
    );
    let linked = root.join("packages/acme-fixture/taxonomy.yml");
    std::fs::remove_file(&linked).expect("the real file makes way for the link");
    std::os::unix::fs::symlink(scratch.path().join("secret/stolen.yml"), &linked)
        .expect("the link is made");

    let refused = package::publish(&root, "acme/fixture", &out_of(&scratch))
        .expect_err("it does not publish");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains("`contents.taxonomy`") && message.contains("outside the package"),
        "the refusal is not the escape rule: {message}"
    );
    assert!(!out_of(&scratch).exists(), "an artifact was written anyway");
}

/// A symlink under the package directory that no `contents` key names still
/// reaches the artifact if nothing refuses it, because `stage` walks the whole
/// directory and `contents` only says what a manifest chose to write down.
///
/// [#303]'s sharper finding: the escape rule on `contents` cannot reach this
/// case at all, because no key names `undeclared.yml` for it to hold to
/// anything.
///
/// [#303]: https://github.com/headwater-ai/headwater/issues/303
#[cfg(unix)]
#[test]
fn an_undeclared_symlink_under_the_package_directory_is_refused() {
    let scratch = Scratch::new("undeclared-symlink");
    let root = publisher(&scratch, None);
    scratch.write("secret/stolen.yml", "STOLEN\n");
    std::os::unix::fs::symlink(
        scratch.path().join("secret/stolen.yml"),
        root.join("packages/acme-fixture/undeclared.yml"),
    )
    .expect("the link is made");

    let refused = package::publish(&root, "acme/fixture", &out_of(&scratch))
        .expect_err("it does not publish");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains("undeclared.yml") && message.contains("outside"),
        "the refusal does not name the offending path: {message}"
    );
    assert!(!out_of(&scratch).exists(), "an artifact was written anyway");
}

/// A symlink inside the directory `contents.bundles` points at is held to the
/// same bound `bundles` itself is, and not to the wider filesystem.
///
/// The bundles walk reads from wherever the manifest points it, outside the
/// package by design, so its boundary is `root` rather than the package
/// directory — and a symlink inside that tree pointing further out is still
/// refused by it. [#303].
///
/// [#303]: https://github.com/headwater-ai/headwater/issues/303
#[cfg(unix)]
#[test]
fn a_symlink_inside_the_bundles_tree_that_resolves_outside_the_repository_is_refused() {
    let scratch = Scratch::new("bundles-symlink-escape");
    let root = publisher(&scratch, None);
    scratch.write("secret/stolen.yml", "STOLEN\n");
    std::os::unix::fs::symlink(
        scratch.path().join("secret/stolen.yml"),
        root.join("library/extra/linked.yml"),
    )
    .expect("the link is made");

    let refused = package::publish(&root, "acme/fixture", &out_of(&scratch))
        .expect_err("it does not publish");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains("linked.yml") && message.contains("outside"),
        "the refusal does not name the offending path: {message}"
    );
    assert!(!out_of(&scratch).exists(), "an artifact was written anyway");
}

/// `contents.bundles` may resolve outside the package, and not outside the
/// repository the publish is reading from.
///
/// [#303]'s first mechanism: the `bundles` exemption from the escape check had
/// no bound at all, so a manifest could point it at any directory the
/// publishing process could read, including the filesystem root. This pins
/// the bound at `root` and includes `bundles: /` among the refused cases, so
/// the unbounded form this issue found is not merely narrowed but closed.
///
/// [#303]: https://github.com/headwater-ai/headwater/issues/303
#[test]
fn a_contents_bundles_that_resolves_outside_the_repository_is_refused() {
    let scratch = Scratch::new("bundles-outside-repository");
    let root = publisher(&scratch, None);
    scratch.write("outside-lib/secret.yml", "x: 1\n");

    for declared in [
        scratch.path().join("outside-lib").display().to_string(),
        "/".to_string(),
    ] {
        scratch.write(
            "publisher/packages/acme-fixture/package.yml",
            &format!(
                "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n  \
                 bundles: {declared}\n"
            ),
        );

        let refused = package::publish(&root, "acme/fixture", &out_of(&scratch))
            .expect_err("it does not publish");
        let message = headwater_resolve::render_errors(&refused);
        assert!(
            message.contains("`contents.bundles`") && message.contains("outside"),
            "the refusal is not the repository bound, for `bundles: {declared}`: {message}"
        );
        assert!(
            !out_of(&scratch).exists(),
            "an artifact was written anyway, for `bundles: {declared}`"
        );
    }
}

/// A `contents.taxonomy` that is not there meets the refusal every other key
/// meets, in both of its forms.
///
/// `taxonomy_source` reads that one key for the resolver, and while the
/// reachability check ran inside `stage` the resolver answered first. The
/// publisher got `cannot read …/packages/acme-fixture/../../elsewhere/taxonomy.yml:
/// No such file or directory`: a file system error that names neither the
/// manifest nor the key, and that carries the `..` publication exists to remove.
/// Spec 7 says the refusal names the manifest, the key and the declared value,
/// and this was the one key for which that sentence was false.
///
/// Both arms of the check are here because a declared path can be missing in two
/// ways, and only the second one existed as a case: a path that leaves the
/// package is refused for leaving it, and a path that stays inside is refused
/// for not being there.
#[test]
fn a_missing_taxonomy_source_is_refused_by_the_manifest_rather_than_by_the_resolver() {
    for declared in ["../../elsewhere/taxonomy.yml", "nosuch.yml"] {
        let scratch = Scratch::new("missing-taxonomy");
        let root = publisher(&scratch, None);
        scratch.write(
            "publisher/packages/acme-fixture/package.yml",
            &format!(
                "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: {declared}\n  \
                 bundles: ../../library\n"
            ),
        );

        let refused = package::publish(&root, "acme/fixture", &out_of(&scratch))
            .expect_err("it does not publish");
        let message = headwater_resolve::render_errors(&refused);
        assert!(
            message.contains(package::MANIFEST),
            "the refusal does not name the manifest: {message}"
        );
        assert!(
            message.contains("`contents.taxonomy`"),
            "the refusal does not name the key: {message}"
        );
        assert!(
            message.contains(declared),
            "the refusal does not name the declared value: {message}"
        );
        assert!(
            !message.contains("No such file or directory"),
            "a file system error reached the publisher instead: {message}"
        );
        assert!(!out_of(&scratch).exists(), "an artifact was written anyway");
    }
}

/// `contents.bundles` naming a file that is there is refused, and nothing is
/// published.
///
/// This is the case that loses content rather than the case that fails to read.
/// The existence check answers presence and not kind, so a `bundles` naming a
/// file passed it, `stage` then took its `if !leaves(…) { return Ok(staged) }`
/// arm — correct for a bundles directory *inside* the package — and the publish
/// exited 0 with an artifact that carried no `bundles/` at all. Measured on this
/// repository's own package it went from 39 members to 3, with the published
/// manifest still pointing at the file and a digest of its own over the three. Spec 7 says every `contents` path is
/// read, and a key whose reader is `read_dir` is not read by naming a file.
#[test]
fn a_contents_bundles_naming_a_file_is_refused() {
    let scratch = Scratch::new("bundles-file");
    let root = publisher(&scratch, None);
    scratch.write(
        "publisher/packages/acme-fixture/package.yml",
        "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n  bundles: \
         taxonomy.yml\n",
    );
    assert!(
        root.join("packages/acme-fixture/taxonomy.yml").is_file(),
        "the case is testing the existence check rather than the kind rule"
    );

    let refused = package::publish(&root, "acme/fixture", &out_of(&scratch))
        .expect_err("it does not publish");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains("`contents.bundles`"),
        "the refusal does not name the key: {message}"
    );
    assert!(
        message.contains("reads a directory"),
        "the refusal does not say which kind the key's reader needs: {message}"
    );
    assert!(!out_of(&scratch).exists(), "an artifact was written anyway");
}

/// `contents.conformance` naming a directory that is there is refused.
///
/// The body of #279's first case. `headwater conformance` is the only reader of
/// the key and it reads the path with `read_to_string`, so an adopter who
/// installed the artifact got `Is a directory (os error 21)` on their own
/// machine while the publish that shipped it exited 0. The directory exists on
/// disk, so the existence check cannot be what refuses it and the kind rule is
/// the only thing under test.
#[test]
fn a_contents_conformance_naming_a_directory_is_refused() {
    let scratch = Scratch::new("conformance-directory");
    let root = publisher(&scratch, None);
    scratch.write(
        "publisher/packages/acme-fixture/package.yml",
        "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n  bundles: \
         ../../library\n  conformance: confdir\n",
    );
    scratch.write(
        "publisher/packages/acme-fixture/confdir/inside.yml",
        "x: 1\n",
    );
    assert!(
        root.join("packages/acme-fixture/confdir").is_dir(),
        "the case is testing the existence check rather than the kind rule"
    );

    let refused = package::publish(&root, "acme/fixture", &out_of(&scratch))
        .expect_err("it does not publish");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains("`contents.conformance`"),
        "the refusal does not name the key: {message}"
    );
    assert!(
        message.contains("reads a file"),
        "the refusal does not say which kind the key's reader needs: {message}"
    );
    assert!(!out_of(&scratch).exists(), "an artifact was written anyway");
}

/// `contents.taxonomy` naming a directory is refused by the manifest, rather
/// than by the resolver reading it.
///
/// The sibling of the missing-source case above, and it was false in the same
/// way. `taxonomy_source` runs after the reachability check, so before a kind
/// comparison existed the publisher got `packages/acme-fixture/confdir: cannot
/// read …: Is a directory (os error 21)` — a file system error that names
/// neither the manifest nor the key.
#[test]
fn a_contents_taxonomy_naming_a_directory_is_refused_by_the_manifest() {
    let scratch = Scratch::new("taxonomy-directory");
    let root = publisher(&scratch, None);
    scratch.write(
        "publisher/packages/acme-fixture/package.yml",
        "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: confdir\n  bundles: \
         ../../library\n",
    );
    scratch.write(
        "publisher/packages/acme-fixture/confdir/inside.yml",
        "x: 1\n",
    );

    let refused = package::publish(&root, "acme/fixture", &out_of(&scratch))
        .expect_err("it does not publish");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains(package::MANIFEST),
        "the refusal does not name the manifest: {message}"
    );
    assert!(
        message.contains("`contents.taxonomy`") && message.contains("reads a file"),
        "the refusal is not the kind rule: {message}"
    );
    assert!(
        !message.contains("Is a directory"),
        "a file system error reached the publisher instead: {message}"
    );
    assert!(!out_of(&scratch).exists(), "an artifact was written anyway");
}

/// `contents.migrations` naming a file is refused.
///
/// This case is hand-built and it is the only one here that is. This
/// repository's own package declares no `contents.migrations`, so no probe over
/// it reaches this arm and a fixture that looked symmetric with the three above
/// would be asserting a guard against a state nothing produces. `migration::at`
/// reads the declared path with `read_dir` and globs `*.yml` out of it, which is
/// where the required kind was read off.
#[test]
fn a_contents_migrations_naming_a_file_is_refused() {
    let scratch = Scratch::new("migrations-file");
    let root = publisher(&scratch, None);
    scratch.write(
        "publisher/packages/acme-fixture/package.yml",
        "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n  bundles: \
         ../../library\n  migrations: taxonomy.yml\n",
    );

    let refused = package::publish(&root, "acme/fixture", &out_of(&scratch))
        .expect_err("it does not publish");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains("`contents.migrations`") && message.contains("reads a directory"),
        "the refusal is not the kind rule: {message}"
    );
    assert!(!out_of(&scratch).exists(), "an artifact was written anyway");
}

/// An empty value under any `contents` key is refused, and the refusal says the
/// value is empty.
///
/// The kind comparison does not catch this and cannot. An empty value joins to
/// the package directory itself, which exists and which *is* a directory, so
/// `bundles: ""` and `migrations: ""` pass a kind check that requires one —
/// `bundles: ""` published the same 4-member artifact as the case above, with
/// every bundle silently gone. The empty arm runs before the escape check, the
/// existence check and the kind check, so it is the arm every key meets first.
#[test]
fn an_empty_contents_value_is_refused_under_every_key() {
    for key in ["taxonomy", "conformance", "bundles", "migrations"] {
        let scratch = Scratch::new(&format!("empty-{key}"));
        let root = publisher(&scratch, None);
        // The emptied key replaces the value the working manifest declares for
        // it, rather than being appended beside it. A manifest that declared
        // `taxonomy` twice would be refused for the duplicate and never reach
        // the rule under test.
        let mut declared = vec![("taxonomy", "taxonomy.yml"), ("bundles", "../../library")];
        match declared.iter_mut().find(|(name, _)| *name == key) {
            Some(pair) => pair.1 = "\"\"",
            None => declared.push((key, "\"\"")),
        }
        let block: String = declared
            .iter()
            .map(|(name, value)| format!("  {name}: {value}\n"))
            .collect();
        scratch.write(
            "publisher/packages/acme-fixture/package.yml",
            &format!("package: acme/fixture\nversion: 1.0.0\ncontents:\n{block}"),
        );

        let refused = package::publish(&root, "acme/fixture", &out_of(&scratch))
            .expect_err("it does not publish");
        let message = headwater_resolve::render_errors(&refused);
        assert!(
            message.contains(&format!("`contents.{key}`")) && message.contains("is empty"),
            "the refusal is not the empty-value rule: {message}"
        );
        assert!(
            !message.contains("No such file or directory") && !message.contains("Is a directory"),
            "a path error about the package directory reached the publisher: {message}"
        );
        assert!(!out_of(&scratch).exists(), "an artifact was written anyway");
    }
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
/// The three files a vendored `acme/fixture` leaves under `packages/`, sorted.
///
/// The cases below read the whole directory rather than asking `find_version`,
/// because a partial tree answers `find_version` correctly as long as the
/// manifest is one of the files that survived. That is exactly the state
/// [#312](https://github.com/headwater-ai/headwater/issues/312) is about.
fn installed_files(adopter: &Path) -> Vec<String> {
    let at = adopter.join(package::PACKAGES).join("acme-fixture");
    let Ok(entries) = std::fs::read_dir(&at) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// An adopter holding `acme/fixture` 1.0.0, and the artifact it came from.
fn adopter_holding(scratch: &Scratch) -> (PathBuf, PathBuf, String) {
    let root = publisher_declaring_at(scratch, "publisher", "acme/fixture", "1.0.0");
    let out = scratch.path().join("artifact-1");
    let first = package::publish(&root, "acme/fixture", &out).expect("1.0.0 publishes");

    let adopter = scratch.path().join("adopter");
    std::fs::create_dir_all(&adopter).expect("the adopter root is made");
    package::vendor(&adopter, &out, &first.digest).expect("the first vendor lands");
    assert_eq!(
        installed_files(&adopter),
        vec![
            "package.yml".to_string(),
            "release.yml".to_string(),
            "taxonomy.yml".to_string()
        ],
        "the first vendor did not install the whole package"
    );
    (adopter, out, first.digest)
}

/// A `packages/` this process cannot write takes nothing from the package that
/// is installed, and the verb is not locked out of its own repair.
///
/// # The state this pins is worse than a partial tree
///
/// `remove_dir_all(&target)` needed write on the target to empty it and write
/// on `packages/` to unlink the target itself. Mode `0500` on `packages/` grants
/// the first and refuses the second, so the removal emptied the adopter's
/// installed package and then failed. The verb printed `nothing was vendored`
/// over a directory it had just emptied, `find_version` answered `None`, and —
/// the part no message said — the empty directory carries no release record, so
/// **every later run took the maintained-package arm and refused to touch it**.
/// The adopter's recovery was `rm -rf` by hand.
///
/// So the third vendor here is the sharpest assertion in the case. It is the
/// one that says the verb can still repair the state a failure left.
///
/// The mode is the injection because no source-side failure can reach the copy:
/// `release::verify` walks the whole artifact tree, so every byte the copy reads
/// has already been read and any unreadable file is refused before this.
#[cfg(unix)]
#[test]
fn a_packages_directory_that_cannot_be_written_takes_nothing_from_the_installed_package() {
    let scratch = Scratch::new("locked-packages");
    if !modes_hold(&scratch) {
        eprintln!("skipped: this process is root, and root writes through mode 0500");
        return;
    }
    let (adopter, out, digest) = adopter_holding(&scratch);

    let packages = adopter.join(package::PACKAGES);
    mode(&packages, 0o500);
    let refused = package::vendor(&adopter, &out, &digest);
    mode(&packages, 0o700);

    let errors = refused.expect_err("a `packages/` that cannot be written refuses the vendor");

    // The state comes first, because the state is the defect and the sentence
    // about it is the clause underneath.
    assert_eq!(
        installed_files(&adopter),
        vec![
            "package.yml".to_string(),
            "release.yml".to_string(),
            "taxonomy.yml".to_string()
        ],
        "the failed vendor took files out of the package the adopter had installed"
    );
    assert_eq!(
        package::find_version(&adopter, "acme/fixture"),
        Some("1.0.0".to_string()),
        "the installed package no longer resolves after a vendor that refused"
    );

    let said = errors
        .iter()
        .map(|error| format!("{error}"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        said.contains("~staging") && said.contains("acme-fixture"),
        "the refusal does not name the staging path it could not make: {said}"
    );
    assert!(
        said.contains("still the package that was installed"),
        "the refusal does not say the installed package is untouched: {said}"
    );

    package::vendor(&adopter, &out, &digest)
        .expect("the verb can still vendor over the package once `packages/` is writable again");
    assert_eq!(
        packages_under(&adopter),
        vec!["acme-fixture".to_string()],
        "a scratch directory survived a vendor that succeeded"
    );
}

/// Vendoring a package over itself installs it, and does not empty it.
///
/// # This case is what decides the shape of the repair
///
/// The artifact path and the target are one directory. Under the old write half
/// `remove_dir_all(&target)` removed the artifact, `copy_tree` then created the
/// directory again and copied nothing into it, and the verb exited **0** with a
/// success message naming the release over an empty directory.
///
/// **The assertion that does the work is `expect`, not survival.** Two plausible
/// repairs both leave the old tree standing and both are wrong here: an unwind
/// that copies the target aside, removes it, and copies the artifact in reaches
/// `copy_tree` with an empty source and exits 0 over an empty directory; an
/// unwind that renames the target aside first no longer finds the artifact and
/// refuses with `ENOENT`. Only staging the copy while the artifact still stands,
/// then swapping, exits 0 with the package installed.
#[test]
fn vendoring_a_package_over_itself_installs_it() {
    let scratch = Scratch::new("over-itself");
    let (adopter, _, digest) = adopter_holding(&scratch);

    let installed = adopter.join(package::PACKAGES).join("acme-fixture");
    package::vendor(&adopter, &installed, &digest)
        .expect("an artifact that is the installed package vendors over itself");

    assert_eq!(
        installed_files(&adopter),
        vec![
            "package.yml".to_string(),
            "release.yml".to_string(),
            "taxonomy.yml".to_string()
        ],
        "vendoring the package over itself did not leave the whole package"
    );
    assert_eq!(
        package::find_version(&adopter, "acme/fixture"),
        Some("1.0.0".to_string()),
        "the package no longer resolves after vendoring it over itself"
    );
    assert_eq!(
        packages_under(&adopter),
        vec!["acme-fixture".to_string()],
        "vendoring the package over itself left a second directory behind"
    );
}

/// A first vendor whose copy fails leaves nothing under `packages/`, rather than
/// the part of the tree it had written.
///
/// The other half of the clause the case above holds: where nothing is
/// installed, *the tree that was there before* is nothing, and that is the state
/// the adopter must be left in. `packages/` is made by hand at mode `0500` so
/// the copy is refused rather than the directory listing.
#[cfg(unix)]
#[test]
fn a_first_vendor_whose_copy_fails_installs_no_part_of_the_package() {
    let scratch = Scratch::new("fresh-fails");
    if !modes_hold(&scratch) {
        eprintln!("skipped: this process is root, and root writes through mode 0500");
        return;
    }
    let root = publisher_declaring_at(&scratch, "publisher", "acme/fixture", "1.0.0");
    let out = scratch.path().join("artifact-1");
    let record = package::publish(&root, "acme/fixture", &out).expect("1.0.0 publishes");

    let adopter = scratch.path().join("adopter");
    let packages = adopter.join(package::PACKAGES);
    std::fs::create_dir_all(&packages).expect("the adopter holds an empty `packages/`");

    mode(&packages, 0o500);
    let refused = package::vendor(&adopter, &out, &record.digest);
    mode(&packages, 0o700);

    let errors = refused.expect_err("a `packages/` that cannot be written refuses the vendor");

    assert!(
        !packages.join("acme-fixture").exists(),
        "a vendor that refused left a directory at `packages/acme-fixture`"
    );
    assert_eq!(
        packages_under(&adopter),
        Vec::<String>::new(),
        "a vendor that refused left something under `packages/`"
    );

    let said = errors
        .iter()
        .map(|error| format!("{error}"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        said.contains("nothing is installed at `packages/acme-fixture`"),
        "the refusal does not say that nothing was installed: {said}"
    );
}

/// An artifact directory that holds `packages/` inside it is refused before a
/// byte is staged.
///
/// `copy_tree` calls `create_dir_all(to)` before `read_dir(from)`, so a
/// destination inside the source is listed by its own copy and the recursion
/// descends into what it is writing. Staging does not fix that and would make it
/// new: today's destination is inside the source in this shape too.
///
/// **The adopter root here is itself the artifact**, which is the cheapest tree
/// that `release::verify` accepts at a path above `packages/`. That is the
/// `headwater taxonomy vendor .` shape.
#[test]
fn an_artifact_that_holds_the_target_inside_it_is_refused() {
    let scratch = Scratch::new("artifact-above");
    let root = publisher_declaring_at(&scratch, "publisher", "acme/fixture", "1.0.0");
    let out = scratch.path().join("artifact-1");
    let record = package::publish(&root, "acme/fixture", &out).expect("1.0.0 publishes");

    // The adopter root is the artifact, so `<root>/packages/acme-fixture` is a
    // path inside the directory being vendored.
    let adopter = scratch.path().join("adopter");
    std::fs::create_dir_all(&adopter).expect("the adopter root is made");
    for name in ["package.yml", "taxonomy.yml", "release.yml"] {
        std::fs::copy(out.join(name), adopter.join(name)).expect("the artifact is copied over");
    }

    let errors = package::vendor(&adopter, &adopter, &record.digest)
        .expect_err("an artifact that holds the target inside it is refused");
    let said = errors
        .iter()
        .map(|error| format!("{error}"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        said.contains("inside itself"),
        "the refusal does not say the copy would descend into its own output: {said}"
    );
    assert!(
        !adopter.join(package::PACKAGES).exists(),
        "the refusal came after something had already been staged under `packages/`"
    );
}

/// A directory this verb stages into never wins the lookup, and the next run
/// clears the one an earlier run left.
///
/// # Two properties of one name, and both of them are load-bearing for `find`
///
/// [`package::find_version`] goes through the one listing of `packages/` this
/// engine has. It sorts the entries and returns the first whose manifest
/// declares the name, with no filter on the name of the entry itself, so a
/// staging tree — which carries a manifest like any other — answers the lookup
/// if it sorts first. A dot-prefixed name does sort first, measured. `~` is
/// 0x7E, and the highest byte the package-name grammar admits is `z` at 0x7A, so
/// the two names this verb writes sort after every package directory and no
/// package can be created under one of them.
///
/// **The vendor at the end is what holds the second property.** The residues
/// here are planted by hand, because a vendor that succeeded leaves none, and a
/// verb whose staging names did not match these would walk past them and leave
/// them standing. So the last assertion says the names in this file are the
/// names the verb writes, which is the half a planted residue cannot say by
/// itself.
#[test]
fn a_directory_this_verb_stages_into_never_wins_the_lookup() {
    let scratch = Scratch::new("staging-shadow");
    let (adopter, _, _) = adopter_holding(&scratch);
    let packages = adopter.join(package::PACKAGES);

    let residues = [
        package::staging_path(&packages, "acme-fixture"),
        packages.join(format!("acme-fixture{}", package::ASIDE)),
    ];
    for residue in residues {
        std::fs::create_dir_all(&residue).expect("the residue is made");
        std::fs::write(
            residue.join(package::MANIFEST),
            "package: acme/fixture\nversion: 9.9.9\ncontents:\n  taxonomy: taxonomy.yml\n",
        )
        .expect("the residue carries a manifest");
    }

    assert_eq!(
        package::find_version(&adopter, "acme/fixture"),
        Some("1.0.0".to_string()),
        "a directory this verb stages into answered the lookup instead of the installed package"
    );

    let later_root = publisher_declaring_at(&scratch, "later", "acme/fixture", "2.0.0");
    let later_out = scratch.path().join("artifact-2");
    let later = package::publish(&later_root, "acme/fixture", &later_out).expect("2.0.0 publishes");
    package::vendor(&adopter, &later_out, &later.digest).expect("2.0.0 vendors over 1.0.0");

    assert_eq!(
        package::find_version(&adopter, "acme/fixture"),
        Some("2.0.0".to_string()),
        "the upgrade did not land over the residues"
    );
    assert_eq!(
        packages_under(&adopter),
        vec!["acme-fixture".to_string()],
        "the vendor left the residues of an earlier run standing, so the names this file plants \
         are not the names the verb writes"
    );
}

/// A plain retry vendor of a package killed mid-swap still lands, because the
/// collision check excludes this run's own staging and aside paths.
///
/// #354's own hardening must not regress #312's self-heal. [`vendor`]'s doc
/// comment already states the reachable state: a kill inside the one-rename
/// window leaves the installed package complete under `<name>~aside`, and
/// [`package::find`] answers from it in the meantime. The staging residue
/// under `packages/~staging/<name>` never answers a lookup at all, complete
/// or not, which is [#357](https://github.com/headwater-ai/headwater/issues/357).
/// The *next* `vendor` of that same package is what clears both residues,
/// which is the self-heal [#312](https://github.com/headwater-ai/headwater/issues/312)
/// asked for. A version of #354's new check that compared only
/// `found_at != target` — without excluding `staged` and `aside` — would run
/// before that clearing, find the same package resolving from `<name>~aside`,
/// and refuse the legitimate retry. This proves the exclusion holds: the
/// residues are planted by hand exactly as
/// [`a_directory_this_verb_stages_into_never_wins_the_lookup`] plants them,
/// with the target itself removed first to stand in for the kill, and the
/// plain retry below must still succeed.
#[test]
fn a_plain_retry_after_a_kill_mid_swap_still_lands() {
    let scratch = Scratch::new("kill-mid-swap-retry");
    let (adopter, _, _) = adopter_holding(&scratch);

    // Simulate a kill mid-swap: the target is gone, and both siblings stand,
    // each declaring the package that was installed.
    let packages = adopter.join(package::PACKAGES);
    std::fs::remove_dir_all(packages.join("acme-fixture"))
        .expect("the target is cleared to simulate the kill");
    let residues = [
        package::staging_path(&packages, "acme-fixture"),
        packages.join(format!("acme-fixture{}", package::ASIDE)),
    ];
    for residue in residues {
        std::fs::create_dir_all(&residue).expect("the residue is made");
        std::fs::write(
            residue.join(package::MANIFEST),
            "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n",
        )
        .expect("the residue carries a manifest");
    }
    assert_eq!(
        package::find_version(&adopter, "acme/fixture"),
        Some("1.0.0".to_string()),
        "the aside residue should still answer the lookup, per vendor's own doc comment"
    );

    // A plain retry of the same package must self-heal rather than be refused
    // as a collision with itself.
    let retry_root = publisher_declaring_at(&scratch, "retry", "acme/fixture", "1.0.0");
    let retry_out = scratch.path().join("artifact-retry");
    let retry =
        package::publish(&retry_root, "acme/fixture", &retry_out).expect("the retry publishes");
    package::vendor(&adopter, &retry_out, &retry.digest).expect(
        "a retry of the same package must land, not be refused as though it collided with \
         itself",
    );

    assert_eq!(
        packages_under(&adopter),
        vec!["acme-fixture".to_string()],
        "the retry did not clear the residues an earlier, killed run left"
    );
}

/// An artifact that is one of the directories this verb stages through is
/// refused, and the artifact is still there.
///
/// # The reachable state, and why the natural gesture is the destructive one
///
/// A run killed after the staging copy and before the first rename leaves
/// exactly `packages/~staging/<name>`, complete, with nothing at the target.
/// [`find`] never reads it there, which is what
/// [#357](https://github.com/headwater-ai/headwater/issues/357) closes: this
/// state does not resolve at all, complete or not, until a further `vendor`
/// clears it. A run killed between the two renames leaves
/// `packages/<name>~aside`, complete, with nothing at the target, and `find`
/// does read that one — it is a flat sibling of the target, one level down
/// like any package — which is the state
/// [#312](https://github.com/headwater-ai/headwater/issues/312) asked to keep
/// resolving. The aside case is where the adopter holds one copy of the
/// package under a name they did not choose, and pointing this verb at it is
/// what finishing the install looks like from outside; the refusal there says
/// the package "is complete under `packages/<name>~aside`", so the message
/// names the directory the gesture would use.
///
/// The first step of the write phase clears both of those paths. Handed one of
/// them as the artifact, the verb deletes the artifact it verified moments
/// earlier. `copy_tree` then calls `create_dir_all(to)` before `read_dir(from)`,
/// and for the staging path `to` and `from` are one directory, so the delete is
/// undone as an empty directory, the listing succeeds over it, nothing copies,
/// the swap installs it and the verb **exits 0 over an empty package** with a
/// success line naming the release. The aside path fails `read_dir` instead and
/// refuses, having eaten the adopter's only complete copy.
///
/// So this is [#312](https://github.com/headwater-ai/headwater/issues/312)'s own
/// defect, reachable only through a state the fix for it creates.
#[test]
fn an_artifact_that_is_a_directory_this_verb_stages_through_is_refused() {
    for kind in ["staging", "aside"] {
        let case = format!("staged-as-artifact-{kind}");
        let scratch = Scratch::new(&case);
        let (adopter, _, digest) = adopter_holding(&scratch);
        let packages = adopter.join(package::PACKAGES);

        // The state a kill leaves: one complete copy, under the sibling name,
        // and nothing at the target.
        let residue = match kind {
            "staging" => package::staging_path(&packages, "acme-fixture"),
            _ => packages.join(format!("acme-fixture{}", package::ASIDE)),
        };
        if let Some(parent) = residue.parent() {
            std::fs::create_dir_all(parent).expect("the residue's parent exists");
        }
        std::fs::rename(packages.join("acme-fixture"), &residue)
            .expect("the package moves to the name a kill would leave it under");

        let errors = package::vendor(&adopter, &residue, &digest)
            .expect_err("an artifact that is a directory this verb stages through is refused");

        let mut left: Vec<String> = std::fs::read_dir(&residue)
            .expect("the artifact this verb refused is still a directory")
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        left.sort();
        assert_eq!(
            left,
            vec![
                "package.yml".to_string(),
                "release.yml".to_string(),
                "taxonomy.yml".to_string()
            ],
            "the verb took files out of the artifact it was handed"
        );
        match kind {
            "aside" => assert_eq!(
                package::find_version(&adopter, "acme/fixture"),
                Some("1.0.0".to_string()),
                "the package the adopter still held stopped resolving"
            ),
            _ => assert_eq!(
                package::find_version(&adopter, "acme/fixture"),
                None,
                "a residue under `~staging/` resolved, which #357 says it must not, complete or not"
            ),
        }

        let said = errors
            .iter()
            .map(|error| format!("{error}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            said.contains("cannot also be the artifact to install"),
            "the refusal does not say why this directory cannot be the artifact: {said}"
        );

        // And the gesture that does work is the one the refusal names.
        let moved = scratch.path().join(format!("moved-{kind}"));
        std::fs::rename(&residue, &moved).expect("the artifact moves out of `packages/`");
        package::vendor(&adopter, &moved, &digest)
            .expect("the artifact vendors once it sits outside `packages/`");
        assert_eq!(
            packages_under(&adopter),
            vec!["acme-fixture".to_string()],
            "the install from outside `packages/` did not land cleanly"
        );
    }
}

/// A file left in the staging directory by an earlier run does not reach the
/// package this run installs.
///
/// # The regression this exists for passes every other case in this file
///
/// The write phase clears both sibling paths before it stages. Clearing only
/// the aside path, and leaving the staging path as it is, passes 48 of the 49
/// cases here: `copy_tree` calls `create_dir_all` on a directory that is
/// already there, writes the artifact over the top of whatever it holds, and
/// the swap then installs the union of the two. Every assertion about the
/// *artifact's* files still holds. What does not hold is that the installed
/// package is only the artifact.
///
/// So the assertion is over a file that no artifact carries. A stale staging
/// directory is what a run killed inside the staging copy leaves, which is a
/// state this change creates and the doc comment on `vendor` records, so this
/// is not a hypothetical input.
#[test]
fn a_file_an_earlier_run_left_in_the_staging_directory_is_not_installed() {
    let scratch = Scratch::new("stale-staging");
    let (adopter, _, _) = adopter_holding(&scratch);

    let stale = package::staging_path(&adopter.join(package::PACKAGES), "acme-fixture");
    std::fs::create_dir_all(&stale).expect("the stale staging directory is made");
    std::fs::write(stale.join("poison.txt"), "not from any artifact")
        .expect("the stale directory holds a file no artifact carries");

    let later_root = publisher_declaring_at(&scratch, "later", "acme/fixture", "2.0.0");
    let later_out = scratch.path().join("artifact-2");
    let later = package::publish(&later_root, "acme/fixture", &later_out).expect("2.0.0 publishes");
    package::vendor(&adopter, &later_out, &later.digest).expect("2.0.0 vendors over 1.0.0");

    assert_eq!(
        installed_files(&adopter),
        vec![
            "package.yml".to_string(),
            "release.yml".to_string(),
            "taxonomy.yml".to_string()
        ],
        "a file an earlier run left in the staging directory reached the installed package"
    );
    assert_eq!(
        package::find_version(&adopter, "acme/fixture"),
        Some("2.0.0".to_string()),
        "the upgrade did not land over the stale staging directory"
    );
    assert_eq!(
        packages_under(&adopter),
        vec!["acme-fixture".to_string()],
        "the stale staging directory is still there after a vendor that succeeded"
    );
}

/// A partial staging tree with a readable manifest and nothing installed does
/// not resolve, on a first install.
///
/// This is [#357](https://github.com/headwater-ai/headwater/issues/357)'s own
/// Done-when: a residue under `packages/~staging/<name>` carries a manifest
/// like any other package directory, but [`find`] reads only one level of
/// `packages/`, and `packages/~staging` itself carries no manifest beside it.
/// So the residue never reaches the comparison that would answer this lookup,
/// whether it is complete or, as here, missing everything but its manifest.
#[test]
fn a_partial_staging_tree_with_no_package_installed_does_not_resolve() {
    let scratch = Scratch::new("partial-staging-first-install");
    let adopter = scratch.path().join("adopter");
    std::fs::create_dir_all(adopter.join(package::PACKAGES)).expect("adopter root");

    let staging = package::staging_path(&adopter.join(package::PACKAGES), "acme-fixture");
    std::fs::create_dir_all(&staging).expect("the partial staging tree is made");
    std::fs::write(
        staging.join(package::MANIFEST),
        "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n",
    )
    .expect("the residue carries a readable manifest, and nothing else");

    assert_eq!(
        package::find_version(&adopter, "acme/fixture"),
        None,
        "a partial staging tree resolved as though it were the installed package"
    );
    assert_eq!(
        packages_under(&adopter),
        vec!["~staging".to_string()],
        "packages/ should hold only the shared staging parent, not a package name"
    );
}

/// A successful vendor of a first install leaves no shared staging parent
/// behind.
///
/// The other cases in this file assert `packages_under` after an upgrade,
/// which would already fail if `~staging` survived — this one pins the
/// fresh-install case on its own and names the path directly, since a shared
/// parent standing after a clean run is exactly what
/// [#357](https://github.com/headwater-ai/headwater/issues/357)'s cost —
/// removing it on every exit path — is about.
#[test]
fn a_successful_first_vendor_removes_the_shared_staging_parent() {
    let scratch = Scratch::new("staging-parent-tidied");
    let root = publisher_declaring_at(&scratch, "publisher", "acme/fixture", "1.0.0");
    let out = scratch.path().join("artifact-1");
    let record = package::publish(&root, "acme/fixture", &out).expect("1.0.0 publishes");

    let adopter = scratch.path().join("adopter");
    std::fs::create_dir_all(&adopter).expect("the adopter root is made");
    package::vendor(&adopter, &out, &record.digest).expect("the vendor lands");

    let staging_root = adopter.join(package::PACKAGES).join(package::STAGING);
    assert!(
        !staging_root.exists(),
        "a successful vendor left the shared staging parent behind"
    );
}

/// A copy failure still removes the shared staging parent this run created.
///
/// The parent is made by `create_dir_all` inside `copy_tree`'s first call, and
/// mode `0500` set here on the parent itself — rather than on `packages/` as
/// the other permission-injected cases in this file do — blocks only the leaf
/// underneath: the one write the copy needs and the one this run made. That
/// isolates the copy-phase cleanup site from the earlier `clear` calls and
/// from the read phase above them, both of which still succeed. `remove_dir`
/// on the now-empty parent needs write only on `packages/`, which this mode
/// leaves alone, so the assertion below needs no mode restored first.
#[cfg(unix)]
#[test]
fn a_copy_failure_still_removes_the_shared_staging_parent_it_created() {
    let scratch = Scratch::new("copy-fails-tidies-parent");
    if !modes_hold(&scratch) {
        eprintln!("skipped: this process is root, and root writes through mode 0500");
        return;
    }
    let root = publisher_declaring_at(&scratch, "publisher", "acme/fixture", "1.0.0");
    let out = scratch.path().join("artifact-1");
    let record = package::publish(&root, "acme/fixture", &out).expect("1.0.0 publishes");

    let adopter = scratch.path().join("adopter");
    let packages = adopter.join(package::PACKAGES);
    std::fs::create_dir_all(&packages).expect("the adopter holds an empty `packages/`");
    let staging_root = packages.join(package::STAGING);
    std::fs::create_dir_all(&staging_root).expect(
        "the shared staging parent exists already, as a concurrent sibling's vendor might leave it",
    );
    mode(&staging_root, 0o500);

    let refused = package::vendor(&adopter, &out, &record.digest);
    refused.expect_err("a staging parent this run cannot write into refuses the vendor");

    assert!(
        !staging_root.exists(),
        "a copy failure left the shared staging parent behind"
    );
}

/// A vendor's cleanup never removes another package's own staging
/// subdirectory, or the shared parent while that subdirectory still stands.
///
/// Two concurrent vendors of different packages now share the literal
/// `packages/~staging` parent as a mkdir/rmdir target, which they did not
/// before this repair — each used to write its own flat `<name>~staged`
/// sibling and never touched the other's path at all. [`tidy`]'s
/// non-recursive `remove_dir`, rather than `remove_dir_all`, is the safety
/// property, and this plants a second package's own subdirectory under the
/// shared parent before vendoring the first, so the parent is never empty at
/// the moment this run's own cleanup runs.
#[test]
fn a_concurrent_siblings_staging_subdirectory_is_never_touched() {
    let scratch = Scratch::new("concurrent-sibling-staging");
    let root = publisher_declaring_at(&scratch, "publisher", "acme/fixture", "1.0.0");
    let out = scratch.path().join("artifact-1");
    let record = package::publish(&root, "acme/fixture", &out).expect("1.0.0 publishes");

    let adopter = scratch.path().join("adopter");
    let packages = adopter.join(package::PACKAGES);
    std::fs::create_dir_all(&packages).expect("the adopter holds an empty `packages/`");

    // A sibling package's own vendor is staging into the shared parent right
    // now, and this run must neither remove it nor be blocked by it.
    let sibling = package::staging_path(&packages, "widgets-core-schema");
    std::fs::create_dir_all(&sibling).expect("the sibling's own staging subdirectory exists");
    std::fs::write(
        sibling.join("marker.txt"),
        "the sibling's own scratch, mid-copy",
    )
    .expect("the sibling's own file is there");

    package::vendor(&adopter, &out, &record.digest).expect("the vendor of the other package lands");

    assert_eq!(
        package::find_version(&adopter, "acme/fixture"),
        Some("1.0.0".to_string()),
        "the vendor that ran alongside a sibling's staging subdirectory did not land"
    );
    let staging_root = packages.join(package::STAGING);
    assert!(
        staging_root.exists(),
        "the shared staging parent was removed while a sibling's own subdirectory still stood in it"
    );
    assert!(
        sibling.join("marker.txt").exists(),
        "a concurrent sibling's own staging subdirectory was touched by this run's cleanup"
    );
    assert_eq!(
        packages_under(&adopter),
        vec!["acme-fixture".to_string(), package::STAGING.to_string()],
        "the successful vendor left something under `packages/` besides the package and the \
         still-occupied shared parent"
    );
}

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

/// #336's first fixture: hand-editing the vendored taxonomy source, bypassing
/// `vendor` entirely, is what `taxonomy resolve --check` exists to catch.
///
/// The committed lock records a digest of each source's raw text
/// (`headwater_lock::write` hashes `source.text`, and [`Source`]'s own doc
/// comment states why: "the lock records a digest of it, so that a stale lock
/// can name the file that moved without resolving anything"). This proves the
/// half of that claim [`package::sources`] is responsible for: a hand edit to
/// the vendored file, made without going through `vendor`, changes the text
/// that a second call to [`package::sources`] reads back, which is what makes
/// a previously-recorded digest of it stale.
///
/// The corruption is a single appended comment line in the vendored
/// `taxonomy.yml`, written directly to the file `vendor` installed rather than
/// through any verb of this engine — the shape #336's adjudication asked for,
/// "bypassing `vendor` itself".
#[test]
fn hand_editing_the_vendored_taxonomy_source_moves_the_digest_a_lock_would_hold() {
    let scratch = Scratch::new("hand-edited-vendored-source");
    let (adopter, _, digest) = adopter_holding(&scratch);

    let consumer = package::Consumer {
        package: "acme/fixture".to_string(),
        version: "1.0.0".to_string(),
        bundles: Vec::new(),
        digest: Some(digest.clone()),
        overlay: None,
        corpus_root: "docs".to_string(),
        exclusions: Vec::new(),
    };

    // The digest a `taxonomy resolve` run would have recorded in the lock for
    // the base source, over the package exactly as `vendor` installed it.
    let sources = package::sources(&adopter, &consumer).expect("the vendored package resolves");
    let base = sources
        .iter()
        .find(|source| source.role == headwater_resolve::Role::Taxonomy)
        .expect("the base source is one of them");
    let digest_before = headwater_hash::digest(base.text.as_bytes());

    // Nothing here goes through `vendor`. This is a person, or a script, that
    // reached into `packages/` directly.
    let taxonomy = adopter
        .join(package::PACKAGES)
        .join("acme-fixture")
        .join("taxonomy.yml");
    let before_edit = std::fs::read_to_string(&taxonomy).expect("the vendored source reads");
    let mut edited = before_edit.clone();
    edited.push_str("\n# a hand edit, never written by `vendor`\n");
    std::fs::write(&taxonomy, &edited).expect("the hand edit writes");
    assert_ne!(
        before_edit, edited,
        "the corruption did not change the file it was meant to change"
    );

    // The same read, over the tree as it stands now.
    let sources_after =
        package::sources(&adopter, &consumer).expect("the edited package still resolves");
    let base_after = sources_after
        .iter()
        .find(|source| source.role == headwater_resolve::Role::Taxonomy)
        .expect("the base source is still one of them");
    let digest_after = headwater_hash::digest(base_after.text.as_bytes());

    assert_ne!(
        digest_before, digest_after,
        "hand-editing the vendored taxonomy source did not move the digest a committed lock \
         would hold for it, so `taxonomy resolve --check` would have read the edited bytes as \
         unchanged"
    );
}

/// #336's second fixture: corrupting the vendored release record's declared
/// digest, after a real vendor, reopens `pin.current` even though the lock
/// above is untouched.
///
/// `headwater_conformance::pin_current` (`crates/conformance/src/lib.rs`)
/// reads `release::at(directory)` and compares `record.digest` against the
/// consumer's pin. This exercises `release::at` directly rather than pulling
/// in `headwater-conformance` as a second dev-dependency: the comparison
/// `pin_current` runs is exactly the one [`release::read`] makes internally,
/// between the header's declared digest and the digest recomputed over the
/// member list beside it, and that comparison lives in this crate.
#[test]
fn corrupting_the_vendored_release_records_digest_reopens_pin_current() {
    let scratch = Scratch::new("corrupted-release-record");
    let (adopter, _, digest) = adopter_holding(&scratch);
    let installed = adopter.join(package::PACKAGES).join("acme-fixture");

    // Sanity: before the corruption, the pin this fixture will read is met,
    // the same way `pin_current` reports `Verdict::Met` when the two digests
    // agree.
    let before = release::at(&installed).expect("the freshly vendored record reads");
    assert_eq!(
        before.digest, digest,
        "the freshly vendored artifact does not carry the pinned digest"
    );

    // Corrupt the declared digest in the header, leaving the member list under
    // it untouched. This is exactly a withdrawal: the record no longer states
    // an accurate account of the bytes it lists.
    let record = installed.join(release::RECORD);
    let text = std::fs::read_to_string(&record).expect("the release record reads");
    let corrupted = text.replacen(
        &digest,
        "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        1,
    );
    assert_ne!(
        text, corrupted,
        "the digest to corrupt was not found in the record"
    );
    std::fs::write(&record, corrupted).expect("the corrupted record writes");

    let after = release::at(&installed);
    let error = after.expect_err(
        "a release record whose declared digest disagrees with its own member list must not read \
         as a valid release",
    );
    assert!(
        matches!(error, ReleaseError::RecordMoved { .. }),
        "the corruption was not reported as the record's own digest disagreeing with its member \
         list: {error:?}"
    );

    // `pin_current` reports exactly this shape of error as a gap, on the same
    // terms as every other unreadable record — `Verdict::Gap(other.to_string())`
    // in `crates/conformance/src/lib.rs`. The message names both digests, which
    // is what a reader of the reopened gap needs.
    let message = error.to_string();
    assert!(
        message.contains("The record was edited after it was written"),
        "the refusal does not say the record was edited after it was written: {message}"
    );
}

/// Every file the corrected manifest block of [spec 7's example](../../../../docs/spec/07-distribution-and-federation.md#publishing)
/// names, so `publish` sees exactly what a publisher who copied that block
/// would hand it.
///
/// `broken` places `bundles` and `migrations` at the manifest's top level,
/// which is where the example read before this fix moved them under
/// `contents`. The directories on disk are identical either way — only the
/// manifest's own placement of the two keys differs — because the defect this
/// fixture exists for is exactly that placement, and not a missing file.
fn spec_seven_example(scratch: &Scratch, at: &str, broken: bool) -> PathBuf {
    let under_contents = if broken {
        String::new()
    } else {
        String::from("  bundles: bundles/\n  migrations: migrations/\n")
    };
    let top_level = if broken {
        String::from("bundles: bundles/\nmigrations: migrations/\n")
    } else {
        String::new()
    };

    let manifest = format!(
        "package: acme/headwater-taxonomy\n\
         version: 3.2.0\n\
         contents:\n\
         \u{20}\u{20}taxonomy: taxonomy.yml\n\
         \u{20}\u{20}conformance: conformance.yml\n\
         {under_contents}\
         \u{20}\u{20}doctrine: doctrine/\n\
         \u{20}\u{20}templates: templates/\n\
         profiles: [service-repo, docs-only, platform]\n\
         {top_level}\
         interview: interview.yml\n"
    );
    let package_dir = format!("{at}/packages/acme-headwater-taxonomy");
    scratch.write(&format!("{package_dir}/package.yml"), &manifest);
    scratch.write(
        &format!("{package_dir}/taxonomy.yml"),
        "\
taxonomy: acme/headwater-taxonomy
version: 3.2.0
purposes:
  rationale: {intent: explain why a choice was made and what it forecloses}
",
    );
    scratch.write(
        &format!("{package_dir}/conformance.yml"),
        "conformance: {}\n",
    );
    scratch.write(
        &format!("{package_dir}/bundles/extra/bundle.yml"),
        "\
overlay: acme/headwater-taxonomy
add:
  purposes:
    procedure: {intent: state how a task is carried out}
",
    );
    scratch.write(
        &format!("{package_dir}/doctrine/doctrine.md"),
        "# Doctrine\n\nProse that explains the method, vendored to consumers.\n",
    );
    scratch.write(
        &format!("{package_dir}/templates/note.md"),
        "# Templates\n\nProse for a publisher's own authors, not a scaffolder source.\n",
    );
    scratch.write(&format!("{package_dir}/interview.yml"), "questions: []\n");
    // `migrations/` is a real, existing directory — empty is the ordinary state
    // of a package that has published no major version yet.
    std::fs::create_dir_all(scratch.path().join(format!("{package_dir}/migrations")))
        .expect("the migrations directory is made");

    scratch.path().join(at)
}

/// A consumer who pins this package and selects its one bundle.
fn takes_the_example(scratch: &Scratch, at: &str, digest: &str) {
    scratch.write(
        &format!("{at}/.headwater/taxonomy.yml"),
        &format!(
            "\
taxonomy:
  package: acme/headwater-taxonomy
  version: 3.2.0
  bundles: [extra]
  digest: {digest}
corpus:
  root: docs
"
        ),
    );
}

/// **The old placement publishes, and a consumer who selects the bundle it
/// ships is refused anyway.**
///
/// This is the defect the issue filed against spec 7's example: `bundles` sat
/// at the manifest's top level, where `reachable` and `stage` never look, so
/// nothing about `publish` itself catches the misplacement — the bundle
/// directory is copied into the artifact regardless, because the whole-tree
/// walk that `stage` runs carries every file the package directory holds,
/// declared or not. The claim only breaks where the manifest is read back:
/// `package::sources` looks up `contents.bundles` to find a bundle a consumer
/// names, and a `bundles` key sitting outside `contents` is invisible to that
/// lookup. So the artifact publishes clean, and the one adopter who asked for
/// the bundle it carries is turned away — the exact "claim a publisher makes
/// and a consumer never sees" that spec 7 names.
#[test]
fn the_old_top_level_placement_of_bundles_publishes_but_refuses_the_consumer_who_selects_one() {
    let scratch = Scratch::new("spec7-broken");
    let root = spec_seven_example(&scratch, "publisher", true);
    let out = scratch.path().join("artifact");

    let record = package::publish(&root, "acme/headwater-taxonomy", &out).expect(
        "the misplaced manifest still publishes — nothing at publish time reads a \
                 top-level `bundles` any differently from an unused key",
    );
    assert!(
        out.join("bundles/extra/bundle.yml").is_file(),
        "the bundle directory is copied whether or not `contents` names it, since `stage` \
         carries the whole package directory regardless of what any key declares"
    );

    takes_the_example(&scratch, "consumer", &record.digest);
    let consumer_root = scratch.path().join("consumer");
    package::vendor(&consumer_root, &out, &record.digest).expect("it vendors");
    let declaration = package::consumer(&consumer_root).expect("it reads");

    let refused = package::sources(&consumer_root, &declaration)
        .expect_err("a consumer who selects a bundle that `contents` never named is refused");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains("this selects 1 bundles and acme/headwater-taxonomy ships none"),
        "the refusal does not name the gap between what the consumer asked for and what \
         `contents` declares: {message}"
    );
}

/// **The corrected placement publishes, and the same consumer's selection of
/// the same bundle succeeds.**
///
/// Same package name, same bundle directory, same consumer declaration as the
/// case above — the only difference is where `bundles` and `migrations` sit in
/// the manifest. Moving them under `contents` is the whole of the fix, and
/// this is the proof that moving them is what closes the gap: nothing else
/// about the fixture changed.
#[test]
fn the_corrected_spec_seven_example_publishes_and_its_bundle_reaches_a_consumer() {
    let scratch = Scratch::new("spec7-corrected");
    let root = spec_seven_example(&scratch, "publisher", false);
    let out = scratch.path().join("artifact");

    let record = package::publish(&root, "acme/headwater-taxonomy", &out)
        .expect("the corrected example publishes");
    assert!(out.join("bundles/extra/bundle.yml").is_file());
    assert!(out.join("doctrine/doctrine.md").is_file());
    assert!(out.join("templates/note.md").is_file());
    assert!(out.join("interview.yml").is_file());

    let manifest = std::fs::read_to_string(out.join(package::MANIFEST)).expect("it is there");
    assert!(manifest.contains("bundles: bundles"), "{manifest}");
    assert!(manifest.contains("migrations: migrations"), "{manifest}");

    takes_the_example(&scratch, "consumer", &record.digest);
    let consumer_root = scratch.path().join("consumer");
    package::vendor(&consumer_root, &out, &record.digest).expect("it vendors");
    let declaration = package::consumer(&consumer_root).expect("it reads");

    let sources = package::sources(&consumer_root, &declaration)
        .expect("the corrected placement resolves for the consumer who selects the bundle");
    assert_eq!(
        sources.len(),
        2,
        "the taxonomy and the one bundle it selects"
    );
}

/// A manifest under `packages/` that parses but is not a mapping is named in
/// the refusal, distinguishably from a plain "nothing declares that name".
///
/// Before this, `find` skipped a manifest of this shape in silence and the
/// walk fell through to the same "no package under `packages/` declares"
/// message a typo in the requested name produces. A reader chasing a typo
/// and a reader chasing a corrupt, unrelated manifest saw the identical
/// sentence.
#[test]
fn a_broken_manifest_under_packages_is_named_in_the_refusal() {
    let scratch = Scratch::new("broken-manifest");
    scratch.write("root/packages/broken/package.yml", "- one\n- two\n");
    let root = scratch.path().join("root");
    let out = scratch.path().join("artifact");

    let refused = package::publish(&root, "acme/fixture", &out)
        .expect_err("nothing here declares acme/fixture, and the broken sibling cannot either");
    let message = headwater_resolve::render_errors(&refused);

    assert!(
        message.contains("packages/broken/package.yml"),
        "the broken manifest is not named:\n{message}"
    );
    assert!(
        message.contains("not a mapping"),
        "the broken manifest's shape is not stated, so it reads like the plain \
         not-found case:\n{message}"
    );
    assert!(
        message.contains("no package under"),
        "the summary line is dropped rather than joined beside the broken-file line:\n{message}"
    );
}

/// A broken, unrelated manifest elsewhere under `packages/` does not stop a
/// package that resolves fine from resolving.
///
/// This is the case that separates two designs `find` could have taken: stop
/// the walk the moment any manifest fails to parse into a mapping, or keep
/// looking and report the broken sibling only when the search comes up empty.
/// The broken directory (`aaa-broken`) sorts before the real one
/// (`acme-fixture`), so a hard-stop implementation would abort here before
/// ever reaching the package this call actually asks for; the ruled design
/// finds it anyway.
#[test]
fn a_broken_sibling_manifest_does_not_stop_a_package_that_resolves_fine() {
    let scratch = Scratch::new("broken-sibling");
    scratch.write("root/packages/aaa-broken/package.yml", "- one\n- two\n");
    let root = publisher_of(&scratch, "root", "acme/fixture");
    let out = scratch.path().join("artifact");

    let record = package::publish(&root, "acme/fixture", &out)
        .expect("a broken, unrelated sibling does not stop a package that resolves fine");

    assert_eq!(record.package, "acme/fixture");
}

/// Two directories that each carry a well-formed manifest declaring the same
/// `package:` name are detected and reported, naming both, instead of
/// resolving silently to whichever sorts first.
///
/// `aaa-vendored` sorts before `zzz-copied-source`, so the pre-fix walk
/// returns the first one the instant it matches and never reads the second at
/// all — the exact silent-duplicate defect #369 files. The second directory
/// is written by a plain `Scratch::write`, the shape a person gets by copying
/// a package directory in beside one `vendor` already installed, following
/// `headwater init`'s own suggested workflow, and not through `package::vendor`
/// itself.
#[test]
fn two_directories_declaring_one_name_are_both_named_rather_than_resolved_silently() {
    let scratch = Scratch::new("duplicate-declared-name");
    scratch.write(
        "root/packages/aaa-vendored/package.yml",
        "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n",
    );
    scratch.write("root/packages/aaa-vendored/taxonomy.yml", TAXONOMY);
    scratch.write(
        "root/packages/zzz-copied-source/package.yml",
        "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n",
    );
    scratch.write("root/packages/zzz-copied-source/taxonomy.yml", TAXONOMY);
    let root = scratch.path().join("root");
    let out = scratch.path().join("artifact");

    let refused = package::publish(&root, "acme/fixture", &out)
        .expect_err("two directories declaring one name must not resolve silently");
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains("packages/aaa-vendored"),
        "the first colliding directory is not named:\n{message}"
    );
    assert!(
        message.contains("packages/zzz-copied-source"),
        "the second colliding directory is not named:\n{message}"
    );
}

/// The same collision, caught through [`package::sources`] rather than
/// [`package::publish`] — the literal path `taxonomy resolve` runs, and the
/// one a reader who has gone no further than "copy a package directory in,
/// then resolve" actually exercises.
///
/// The consumer declaration is built directly rather than read from a written
/// `.headwater/taxonomy.yml`, because the collision this proves lives entirely
/// under `packages/` and a hand-built [`package::Consumer`] is the smaller
/// fixture for it.
#[test]
fn two_directories_declaring_one_name_are_caught_by_sources_too() {
    let scratch = Scratch::new("duplicate-declared-name-sources");
    scratch.write(
        "root/packages/aaa-vendored/package.yml",
        "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n",
    );
    scratch.write("root/packages/aaa-vendored/taxonomy.yml", TAXONOMY);
    scratch.write(
        "root/packages/zzz-copied-source/package.yml",
        "package: acme/fixture\nversion: 1.0.0\ncontents:\n  taxonomy: taxonomy.yml\n",
    );
    scratch.write("root/packages/zzz-copied-source/taxonomy.yml", TAXONOMY);
    let root = scratch.path().join("root");

    let consumer = package::Consumer {
        package: "acme/fixture".to_string(),
        version: "1.0.0".to_string(),
        bundles: Vec::new(),
        digest: None,
        overlay: None,
        corpus_root: "docs".to_string(),
        exclusions: Vec::new(),
    };

    let refused = package::sources(&root, &consumer).expect_err(
        "`sources`, the path `taxonomy resolve` runs, must not resolve a duplicate \
                     name silently either",
    );
    let message = headwater_resolve::render_errors(&refused);
    assert!(
        message.contains("packages/aaa-vendored"),
        "the first colliding directory is not named:\n{message}"
    );
    assert!(
        message.contains("packages/zzz-copied-source"),
        "the second colliding directory is not named:\n{message}"
    );
}

/// A manifest that fails to parse at all — a genuine YAML syntax error, not
/// merely a value that parses but is not a mapping — sitting in a directory
/// that sorts after a package that resolves fine, does not stop that package
/// from resolving.
///
/// Before this fix, `find` returned the instant it matched `acme-fixture` and
/// never read `zzz-broken-syntax` at all, so a syntax error there was
/// invisible by construction. Removing the early return to detect duplicate
/// names means the walk now reaches every directory regardless of where the
/// match sits, including ones after it — and `crate::source::load`'s own
/// `?` would have turned this manifest's parse failure into a hard error for
/// the whole call, regressing the "a broken sibling does not stop a package
/// that resolves fine" guarantee #296 already shipped. Folding a load failure
/// into `broken` the same way a non-mapping manifest already is folds this
/// case in too. `package: [oops` is unclosed flow-sequence syntax that
/// `headwater_yaml` refuses outright, confirmed directly against
/// [`headwater_resolve::source::load`] before this fixture was written.
#[test]
fn a_yaml_syntax_error_after_a_match_does_not_stop_the_match_from_resolving() {
    let scratch = Scratch::new("syntax-error-after-match");
    let root = publisher_of(&scratch, "root", "acme/fixture");
    scratch.write(
        "root/packages/zzz-broken-syntax/package.yml",
        "package: [oops\nversion: 1.0.0\n",
    );
    let out = scratch.path().join("artifact");

    let record = package::publish(&root, "acme/fixture", &out).expect(
        "a manifest that fails to parse, sitting after a match the walk now has to keep reading \
         past, must not stop the match from resolving",
    );

    assert_eq!(record.package, "acme/fixture");
}
