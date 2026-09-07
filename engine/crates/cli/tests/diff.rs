// SPDX-License-Identifier: Apache-2.0
//! `taxonomy diff`, over two published versions of one package.
//!
//! # The defect this target exists for
//!
//! A diff between two releases of one taxonomy has one failure mode that a
//! green run hides completely: **it fires on every version bump**. Two publishes
//! of an unchanged package differ in a version string, a digest and a file
//! timestamp, and a comparison that reached any of those reports every release
//! as a breaking change. Such a verb passes every test that only ever hands it
//! a real break. So the silent direction is the case this file was written for,
//! and it is the first one below.
//!
//! It found the defect it was written for. The corpus descriptor states the
//! package, the version and the taxonomy digest of the run that wrote it, so
//! the first run of the cosmetic case reported `projection BROKEN` against a
//! change of one comment and one sentence of guidance. The fix is in the caller
//! rather than in a dimension: both plans are built under one identity, because
//! the identity of a run is an injected value and never a consequence of a
//! taxonomy. `a_version_bump_alone_moves_no_dimension` is what holds it.
//!
//! # Why this drives the binary
//!
//! Every dimension is a comparison of two values that four phases produce, and
//! `headwater-compat` takes both already built. A test at that grain proves the
//! comparison and says nothing about whether the caller ran the second phase
//! against the second taxonomy. That is the whole of what can go wrong here, so
//! each case runs the built binary: `taxonomy resolve`, then `taxonomy publish`
//! twice, then `taxonomy diff` over what came out.
//!
//! # The root each case runs over
//!
//! This repository's own package, overlay and consumer declaration, over a
//! corpus of one document. Copied rather than committed a second time, for the
//! reason `wiring.rs` gives: a taxonomy under `fixtures/` is a schema that no
//! gate holds current, and it would go stale in silence. A base package that
//! stops classifying this document fails these cases loudly instead.

use std::path::{Path, PathBuf};
use std::process::Command;

mod common;
use common::pin;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// A repository root that removes itself.
///
/// `label` names the test and not the case. Cargo runs the cases of one target
/// as threads of one process, so a directory keyed on the process identifier
/// alone is a directory one case removes while another is reading it.
struct Root {
    at: PathBuf,
}

impl Root {
    fn new(label: &str) -> Root {
        Root::shaped(label, |_| {})
    }

    /// A root whose bundle order makes one overlay found the kind another one
    /// declares, so the resolution carries a founding and every publish out of
    /// it carries the same founding.
    ///
    /// The two bundles commute, which is the whole point: `zz-a` writes a leaf
    /// under `kinds.zz_thing` and `zz-b` declares that kind, so the declared
    /// order decides which of them creates the key. The consumer is free to
    /// list them either way — `headwater_resolve::package::selected` pushes
    /// bundles in the declared order with no topological pass — so the order
    /// here is a legal one and not a broken root.
    ///
    /// Three things about the pair, each of which cost an earlier attempt.
    /// `taxonomy publish` publishes the package with every bundle it ships, so
    /// a synthetic bundle has to resolve under that maximal set as well as
    /// under this consumer's selection: an address that reads a declaration the
    /// adopter overlay carries resolves for the consumer and fails the publish.
    /// A synthetic concrete kind needs a shelf, or `coverage` refuses the
    /// resolution. And reaching into a key an existing bundle or the adopter
    /// overlay already writes is refused as a collision rather than recorded as
    /// a founding, so the kind name is one nothing else names.
    fn founding(label: &str) -> Root {
        Root::shaped(label, |at| {
            for (name, body) in [
                (
                    "zz-a",
                    "# SPDX-License-Identifier: Apache-2.0\n\nbundle: zz-a\nextends: \
                     headwater/standard@4.1.0\nrequires: []\n\nadd:\n  kinds.zz_thing.voice: \
                     declarative\n",
                ),
                (
                    "zz-b",
                    "# SPDX-License-Identifier: Apache-2.0\n\nbundle: zz-b\nextends: \
                     headwater/standard@4.1.0\nrequires: []\n\nadd:\n  kinds.zz_thing:\n    is_a: \
                     governed_document\n    purpose: behavior\n    lifecycle: standard\n  \
                     shelves.zz_things:\n    title: Zz Things\n    path: docs/zz/**\n    \
                     homogeneous: true\n    kind: zz_thing\n",
                ),
            ] {
                let directory = at.join("docs/taxonomies").join(name);
                std::fs::create_dir_all(&directory).expect("the bundle directory is made");
                std::fs::write(directory.join("bundle.yml"), body).expect("the bundle writes");
            }

            let declaration = at.join(".headwater/taxonomy.yml");
            let text = std::fs::read_to_string(&declaration).expect("the declaration reads");
            let from = "  bundles: [design-spec";
            assert!(text.contains(from), "the declaration lists its bundles");
            std::fs::write(
                &declaration,
                text.replacen(from, "  bundles: [zz-a, zz-b, design-spec", 1),
            )
            .expect("the declaration writes");
        })
    }

    fn shaped(label: &str, prepare: impl FnOnce(&Path)) -> Root {
        let at =
            std::env::temp_dir().join(format!("headwater-cli-diff-{}-{label}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");

        let repository = repository();
        // `packages/headwater-standard/` is a vendored artifact since #366
        // (it carries a `release.yml`, and `taxonomy publish` now refuses to
        // publish a directory in that state — the guard this fixture would
        // otherwise trip, since every case here calls `taxonomy publish` by
        // name with no `--from`). The maintained source is
        // `taxonomy-source/headwater-standard/`, copied here to the path the
        // by-name lookup expects.
        copy(
            &repository.join("taxonomy-source/headwater-standard"),
            &at.join("packages/headwater-standard"),
        );
        copy(
            &repository.join("docs/taxonomies"),
            &at.join("docs/taxonomies"),
        );
        copy(
            &repository.join("engine/crates/cli/fixtures/change/docs"),
            &at.join("docs"),
        );
        for name in ["taxonomy.yml", "overlay.yml"] {
            let to = at.join(".headwater").join(name);
            std::fs::create_dir_all(to.parent().expect("it has a parent"))
                .expect("the declaration directory is there");
            std::fs::copy(repository.join(".headwater").join(name), to)
                .expect("the declaration copies");
        }

        prepare(&at);
        pin(&at, "1.0.0");

        let root = Root { at };
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(resolved.code, Some(0), "the fixture resolves: {resolved:?}");
        root
    }

    fn run(&self, arguments: &[&str]) -> Ran {
        let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(arguments)
            .arg("--root")
            .arg(&self.at)
            .output()
            .expect("the binary runs");
        Ran {
            code: output.status.code(),
            out: String::from_utf8_lossy(&output.stdout).into_owned(),
            err: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    /// Publish the package as it stands, into a directory named for the release.
    fn publish(&self, name: &str) -> PathBuf {
        let out = self.at.join("released").join(name);
        let ran = self.run(&["taxonomy", "publish", "--out", out.to_str().expect("utf-8")]);
        assert_eq!(ran.code, Some(0), "the artifact publishes: {ran:?}");
        out
    }

    /// Rewrite the package source, so that the next publish is a second version.
    ///
    /// The lock is not re-resolved afterwards, and that is the state a real
    /// consumer is in: the lock names the version this repository took, and the
    /// artifact names the version somebody is proposing.
    ///
    /// **Both declarations of the version move.** A package states its version
    /// in the manifest and again at the root of its taxonomy source, and this
    /// moved only the manifest until
    /// [#212](https://github.com/headwater-ai/headwater/issues/212) gave the two
    /// a reader. Every case in this target therefore published a package that
    /// stated two versions of itself, which is the defect the issue is about,
    /// standing inside the harness written to test the verb that ships it. The
    /// sibling helper in `migration.rs` moved both from the day it was written.
    fn edit(&self, version: &str, edits: &[(&str, &str)]) {
        let taxonomy = self.at.join("packages/headwater-standard/taxonomy.yml");
        let mut text = std::fs::read_to_string(&taxonomy).expect("the taxonomy reads");
        for (from, to) in edits {
            assert!(text.contains(from), "the fixture still carries `{from}`");
            text = text.replacen(from, to, 1);
        }
        assert!(text.contains("version: 1.0.0"), "the source is at 1.0.0");
        std::fs::write(&taxonomy, text.replacen("version: 1.0.0", version, 1))
            .expect("the taxonomy writes");

        let manifest = self.at.join("packages/headwater-standard/package.yml");
        let text = std::fs::read_to_string(&manifest).expect("the manifest reads");
        assert!(text.contains("version: 1.0.0"), "the fixture is at 1.0.0");
        std::fs::write(&manifest, text.replacen("version: 1.0.0", version, 1))
            .expect("the manifest writes");
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
}

impl Ran {
    /// The word one dimension reported, out of the summary block.
    fn dimension(&self, name: &str) -> String {
        self.out
            .lines()
            .find_map(|line| line.trim_start().strip_prefix(name))
            .map(|rest| rest.trim().to_string())
            .unwrap_or_else(|| panic!("the report names `{name}`: {self:?}"))
    }

    /// Every report this file produces from an honest lock states this caveat
    /// nowhere.
    ///
    /// `headwater_compat::Caveat::FoundingOutsideTheDigest` fires on a base
    /// that resolved to the same text beside a broken `addressability`, which
    /// says the founding record of the previous side and the taxonomy beside it
    /// came from two resolutions. A guard that nothing ever provokes reports
    /// nothing forever and reads exactly like a working one, so every case here
    /// that receives a report asserts the quiet direction and
    /// `a_lock_whose_founding_record_was_stripped_says_it_cannot_settle_the_answer`
    /// asserts the loud one.
    fn states_no_contradiction(&self) -> &Ran {
        assert!(
            !self.out.contains("this run cannot settle it"),
            "the lock of this case is honest, so nothing here contradicts itself: {self:?}"
        );
        self
    }
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the directory is there");
    for entry in std::fs::read_dir(from).expect("the fixture directory reads") {
        let entry = entry.expect("the entry reads");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("the file type reads").is_dir() {
            true => copy(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("the fixture copies");
            }
        }
    }
}

/// The silent direction, and the case this file exists for.
///
/// A comment, a sentence of guidance and a version number all move, so the base
/// resolves to different text and every published byte differs. Not one of the
/// six dimensions is a question about any of that, and the assertion is that all
/// six say so.
///
/// The `base resolved to different text` assertion is not decoration. Without
/// it this case would pass against a diff of a package that did not change,
/// which is the one comparison that proves nothing.
#[test]
fn a_version_bump_alone_moves_no_dimension() {
    let root = Root::new("version-bump-alone");
    let first = root.publish("1.0.0");
    root.edit(
        "version: 1.1.0",
        &[
            (
                "# The base package of headwater/standard, as a file.",
                "# The base package of headwater/standard, as a file. Reworded.",
            ),
            (
                "intent: explain why a choice was made and what it forecloses",
                "intent: explain the choice that was made and what it rules out",
            ),
        ],
    );
    let second = root.publish("1.1.0");
    assert_ne!(
        std::fs::read_to_string(first.join("taxonomy.yml")).expect("the first reads"),
        std::fs::read_to_string(second.join("taxonomy.yml")).expect("the second reads"),
        "the two artifacts differ, or this case compares one package with itself"
    );

    let ran = root.run(&[
        "taxonomy",
        "diff",
        second.to_str().expect("utf-8"),
        "--to",
        "1.1.0",
        "--now",
        "2026-08-01",
    ]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    ran.states_no_contradiction();
    assert!(
        ran.out.contains("the base resolved to different text"),
        "the taxonomy did move, and this case is about a diff that stays silent \
         anyway: {ran:?}"
    );
    // The provenance of the previous side, on the face of the report. This is
    // the publisher-in-one-tree shape: `Root::edit` rewrote the package source
    // the lock records, so the previous side is the taxonomy the lock carries
    // and not what that file resolves to now. It is stated and never gated on,
    // because gating here would refuse this run, which is a correct one.
    assert!(
        ran.out
            .contains("the previous side was read from a lock whose sources have moved"),
        "{ran:?}"
    );
    assert!(
        ran.out
            .contains("moved  packages/headwater-standard/taxonomy.yml"),
        "the caveat names the file: {ran:?}"
    );
    for dimension in [
        "classification",
        "instance_validity",
        "consequence",
        "projection",
        "identifier",
        "addressability",
    ] {
        assert_eq!(
            ran.dimension(dimension),
            "preserved",
            "`{dimension}` moved on a change no document can see: {ran:?}"
        );
    }
    assert!(
        ran.out.contains("nothing here requires a major version"),
        "{ran:?}"
    );
}

/// The same silent direction, on a root whose bundle order founds a key.
///
/// `addressability` was the one dimension of six that read a single side, so it
/// reported the candidate's whole founding record as a break whether or not the
/// release under test introduced any of it. A consumer whose bundle order
/// happens to found a key was then told that every release of the package
/// requires a major version, on every diff, until they reordered a list they
/// were free to write either way. Only the version string moves here, so the
/// answer is the same one the other five give: preserved.
///
/// The `operations that make what they address: 1` assertion is what keeps this
/// case about something. A root that stopped founding anything would pass this
/// on a dimension that never had an entry to carry.
#[test]
fn a_founding_the_previous_release_already_carried_is_not_a_break() {
    let root = Root::founding("founding-carried");
    let validated = root.run(&["taxonomy", "validate"]);
    assert_eq!(validated.code, Some(0), "{validated:?}");
    assert!(
        validated
            .out
            .contains("operations that make what they address: 1"),
        "the bundle order founds a key, or this case is about nothing: {validated:?}"
    );

    let first = root.publish("1.0.0");
    root.edit("version: 1.1.0", &[]);
    let second = root.publish("1.1.0");
    assert_eq!(
        std::fs::read_to_string(first.join("taxonomy.yml"))
            .expect("the first reads")
            .replacen("version: 1.0.0", "version: 1.1.0", 1),
        std::fs::read_to_string(second.join("taxonomy.yml")).expect("the second reads"),
        "nothing but the version moved between the two artifacts"
    );

    let ran = root.run(&[
        "taxonomy",
        "diff",
        second.to_str().expect("utf-8"),
        "--to",
        "1.1.0",
        "--now",
        "2026-08-01",
    ]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    ran.states_no_contradiction();
    assert_eq!(
        ran.dimension("addressability"),
        "preserved",
        "a founding the release the lock names already carried is not a break this \
         release introduced: {ran:?}"
    );
    assert!(
        ran.out.contains("nothing here requires a major version"),
        "{ran:?}"
    );
}

/// A lock whose founding record no longer describes the taxonomy beside it.
///
/// # The report contradicted itself and nothing noticed
///
/// [#628](https://github.com/headwater-ai/headwater/issues/628). The `founded:`
/// block of a lock sits outside the digest the lock declares, so a lock that
/// lost it still reads, `conformance`'s `lock.current` rule still reports `met`
/// over it, and `taxonomy diff` still runs. What came out said three things at
/// once: the base resolved to the same text byte for byte, `addressability`
/// BROKEN, and this change requires a major version. A publisher who changed
/// nothing was told to bump the major.
///
/// # Why the caveat and not a refusal
///
/// The five other dimensions read the taxonomy body, which the digest covers,
/// so an identical base makes them agree by construction. The founding record
/// is the one reading of the previous side outside that digest, and it is a
/// property of which operation created a key during the merge rather than of
/// the text the merge produced. Two merges can reach identical text by
/// different routes, so a release that moved a founding without moving a
/// declaration reaches this state honestly. Nothing this run holds separates
/// the two, and the report says so rather than picking one.
///
/// The second half of this case is the remedy the report names, run: a
/// `taxonomy resolve` rewrites the block, and the same comparison then reports
/// the release that changed nothing as changing nothing.
#[test]
fn a_lock_whose_founding_record_was_stripped_says_it_cannot_settle_the_answer() {
    let root = Root::founding("founding-stripped");
    let artifact = root.publish("1.0.0");

    let lock = root.at.join(".headwater/taxonomy.lock");
    let text = std::fs::read_to_string(&lock).expect("the lock reads");
    let start = text
        .find("\nfounded:")
        .expect("this root resolves with a founding, so its lock records one")
        + 1;
    // The block runs to the next key at column zero.
    let end = text[start..]
        .match_indices('\n')
        .find(|(offset, _)| {
            text[start + offset + 1..]
                .chars()
                .next()
                .is_some_and(|first| !first.is_whitespace())
        })
        .map(|(offset, _)| start + offset + 1)
        .unwrap_or(text.len());
    let stripped = format!("{}{}", &text[..start], &text[end..]);
    assert!(
        !stripped.contains("\nfounded:"),
        "the whole block goes, and not its first line"
    );
    assert!(
        stripped.len() < text.len(),
        "the strip removed something: {}",
        text.len()
    );
    std::fs::write(&lock, stripped).expect("the lock writes");

    // Nothing between the edit and the report refuses it. The digest the lock
    // declares still matches the taxonomy it carries, which is the one currency
    // question `headwater_lock::read` asks of every reader.
    let ran = root.run(&[
        "taxonomy",
        "diff",
        artifact.to_str().expect("utf-8"),
        "--now",
        "2026-08-01",
    ]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert!(
        ran.out.contains("the base resolved to the same text"),
        "the artifact is the one this lock was written from: {ran:?}"
    );
    assert!(
        ran.dimension("addressability").starts_with("BROKEN"),
        "the stripped block reads back as a founding this release introduced: {ran:?}"
    );
    assert!(
        ran.out.contains("this run cannot settle it"),
        "the report names its own contradiction: {ran:?}"
    );
    assert!(
        ran.out.contains("`headwater taxonomy resolve` rewrites"),
        "and names the one remedy it can name: {ran:?}"
    );
    // Not one source file moved: what moved is the lock itself. So the other
    // caveat is silent here, and the two are independent readings rather than
    // one staleness reading printed twice.
    assert!(
        !ran.out.contains("whose sources have moved"),
        "the sources are untouched: {ran:?}"
    );

    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_eq!(resolved.code, Some(0), "{resolved:?}");
    let ran = root.run(&[
        "taxonomy",
        "diff",
        artifact.to_str().expect("utf-8"),
        "--now",
        "2026-08-01",
    ]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    ran.states_no_contradiction();
    assert_eq!(ran.dimension("addressability"), "preserved", "{ran:?}");
    assert!(
        ran.out.contains("nothing here requires a major version"),
        "the release that changed nothing changed nothing: {ran:?}"
    );
}

/// The loud direction, at the grain a reader acts on.
///
/// A required facet added to the abstract kind that every governed document is
/// a. Every document of the corpus stops validating, and the report names the
/// document rather than the declaration: spec 7 asks the consumer's run to
/// report "which local documents violate the new schema".
#[test]
fn a_declaration_that_breaks_a_document_names_that_document() {
    let root = Root::new("breaks-a-document");
    root.publish("1.0.0");
    root.edit(
        "version: 2.0.0",
        &[
            (
                "facets: {require: [status, status_since, last_verified, summary]}",
                "facets: {require: [status, status_since, last_verified, summary, owner]}",
            ),
            (
                "  summary:\n    role: scent",
                "  owner:\n    type: string\n    required: true\n    volatility: mutable\n  \
                 summary:\n    role: scent",
            ),
        ],
    );
    let candidate = root.publish("2.0.0");

    let ran = root.run(&[
        "taxonomy",
        "diff",
        candidate.to_str().expect("utf-8"),
        "--now",
        "2026-08-01",
    ]);
    assert_eq!(ran.code, Some(0), "a measured break is a report: {ran:?}");
    ran.states_no_contradiction();
    assert!(
        ran.dimension("instance_validity").starts_with("BROKEN"),
        "{ran:?}"
    );
    assert!(
        ran.dimension("consequence").starts_with("BROKEN"),
        "{ran:?}"
    );
    assert!(
        ran.out
            .contains("facet.required.missing at docs/decisions/0001-the-warrant-a-person-set.md"),
        "the report names the document that stopped validating: {ran:?}"
    );
    assert!(
        ran.out.contains("requires the facet `owner`"),
        "and what it now fails: {ran:?}"
    );
    assert!(
        ran.out.contains("this change requires a major version"),
        "spec 2: any dimension broken forces a major: {ran:?}"
    );
    // The dimensions this change is not about stay quiet, which is what makes
    // the two above a measurement rather than an alarm.
    assert_eq!(ran.dimension("classification"), "preserved", "{ran:?}");
    assert_eq!(ran.dimension("identifier"), "preserved", "{ran:?}");
}

/// The one dimension whose subject is the schema, and the five that cannot be
/// measured when it breaks.
///
/// The new base declares an identifier scheme that this repository's overlay
/// adds, so the `add` collides and the candidate does not resolve. No census,
/// no run, no plan and no graph exist under it. Reporting the other five as
/// preserved would be the strongest possible claim made out of a failure, so
/// each one says it did not run, and the verb exits non-zero because it could
/// not measure rather than because it measured something bad.
///
/// Below the table the run prints the collision as a judgment task: both
/// declarations, the file that carries each, and the two operations that
/// settle it. The exit code and the five unmeasured dimensions are asserted in
/// this same test on purpose, so that neither can drift away from it.
#[test]
fn an_overlay_address_the_new_base_takes_is_the_addressability_dimension() {
    let root = Root::new("addressability");
    root.publish("1.0.0");
    root.edit(
        "version: 2.0.0",
        &[(
            "identifier_schemes:\n  decision_id:",
            "identifier_schemes:\n  spec_id: {pattern: \"SPEC-{namespace}-{slug}\", \
             namespace: HW, allocation: reconcile-first}\n  decision_id:",
        )],
    );
    let candidate = root.publish("2.0.0");

    let ran = root.run(&[
        "taxonomy",
        "diff",
        candidate.to_str().expect("utf-8"),
        "--now",
        "2026-08-01",
    ]);
    assert_eq!(
        ran.code,
        Some(1),
        "a run that could not measure fails: {ran:?}"
    );
    ran.states_no_contradiction();
    assert!(
        ran.dimension("addressability").starts_with("BROKEN"),
        "{ran:?}"
    );
    assert!(
        ran.out
            .contains("add.identifier_schemes.spec_id in .headwater/overlay.yml"),
        "the report names the overlay operation and the file that carries it: {ran:?}"
    );
    for dimension in [
        "classification",
        "instance_validity",
        "consequence",
        "projection",
        "identifier",
    ] {
        assert!(
            ran.dimension(dimension).starts_with("not measured"),
            "`{dimension}` claims a reading out of a candidate that did not resolve: {ran:?}"
        );
    }
    assert!(
        ran.err
            .contains("five of the six dimensions were not measured"),
        "{ran:?}"
    );
    // A base that nobody could read is neither the same text nor different
    // text, and the report says the third thing rather than pick one.
    assert!(
        ran.out
            .contains("the base did not resolve, so there is no second text to compare"),
        "{ran:?}"
    );

    // Both declarations, and each one on its own side. Two fields of the
    // scheme differ and the two patterns are the same tokens in two orders, so
    // a report that swapped the sides would still contain every value it
    // should. The split is what makes this a test of which source said what.
    let (base_half, overlay_half) = ran
        .out
        .split_once(".headwater/overlay.yml adds it")
        .unwrap_or_else(|| panic!("the report names the overlay side: {ran:?}"));
    assert!(
        base_half.contains("released/2.0.0"),
        "the report names the base file: {ran:?}"
    );
    assert!(
        base_half.contains(r#"pattern: "SPEC-{namespace}-{slug}""#),
        "the base's pattern is not on the base's side: {ran:?}"
    );
    assert!(base_half.contains("allocation: reconcile-first"), "{ran:?}");
    assert!(
        overlay_half.contains(r#"pattern: "{namespace}-SPEC-{slug}""#),
        "the overlay's pattern is not on the overlay's side: {ran:?}"
    );
    assert!(overlay_half.contains("allocation: minted-once"), "{ran:?}");
    assert!(
        overlay_half.contains("at add.identifier_schemes.spec_id"),
        "{ran:?}"
    );
    // The remedy, and the fact that it is a choice.
    assert!(
        ran.out
            .contains("the two declarations differ, so what this repository inherits is a choice"),
        "{ran:?}"
    );
    assert!(
        ran.out.contains("inherit the base declaration whole"),
        "{ran:?}"
    );
    assert!(ran.out.contains("restate it as an `override`"), "{ran:?}");
    // The section sits below the table rather than inside it. A multi-line
    // value that leaked into a `was`/`now` slot would still satisfy every
    // assertion above.
    assert!(
        ran.out
            .find("1 `add` collision")
            .zip(ran.out.find("addressability     BROKEN"))
            .is_some_and(|(task, table)| task > table),
        "the judgment task is inside the dimension table: {ran:?}"
    );
}

/// `--to` is the assertion and never the address.
///
/// This engine fetches nothing, so the directory decides which artifact is
/// compared. A flag that named a version the directory is not is a wrong
/// directory, and a run that measured it anyway would report a comparison the
/// caller did not ask for.
#[test]
fn the_version_flag_refuses_an_artifact_that_is_not_that_version() {
    let root = Root::new("version-flag");
    let first = root.publish("1.0.0");

    let ran = root.run(&[
        "taxonomy",
        "diff",
        first.to_str().expect("utf-8"),
        "--to",
        "2.0.0",
        "--now",
        "2026-08-01",
    ]);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(ran.err.contains("the artifact declares 1.0.0"), "{ran:?}");
    assert!(
        !ran.out.contains("classification"),
        "nothing is measured after the refusal: {ran:?}"
    );

    // The same artifact, under a range that admits it. `>=1 <2` is read by the
    // one range reader this engine has, which is what reads `requires_engine`.
    let ran = root.run(&[
        "taxonomy",
        "diff",
        first.to_str().expect("utf-8"),
        "--to",
        ">=1 <2",
        "--now",
        "2026-08-01",
    ]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    ran.states_no_contradiction();
    assert_eq!(ran.dimension("classification"), "preserved", "{ran:?}");
}

/// An artifact somebody edited after it was published is not a version.
///
/// Every dimension below would otherwise measure against a taxonomy that no
/// publisher shipped, and the report would name a version number for it.
#[test]
fn an_artifact_that_left_its_own_release_record_is_refused() {
    let root = Root::new("edited-artifact");
    let first = root.publish("1.0.0");
    let taxonomy = first.join("taxonomy.yml");
    let text = std::fs::read_to_string(&taxonomy).expect("the artifact reads");
    std::fs::write(&taxonomy, format!("{text}\n# edited after publication\n"))
        .expect("the artifact writes");

    let ran = root.run(&[
        "taxonomy",
        "diff",
        first.to_str().expect("utf-8"),
        "--now",
        "2026-08-01",
    ]);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(
        ran.err
            .contains("not what its own release record says it is"),
        "{ran:?}"
    );
}

/// `instance_validity` is a partition of `consequence` and not a copy of it.
///
/// The two dimensions are one comparison over two populations, and the
/// partition is [`Grain`], read off the trait a check implements. A wiring that
/// handed both the same population would report the two identically forever,
/// and every case above would still pass: each of them moves a document-grained
/// rule, which is in both populations by construction.
///
/// So this case moves a rule that reads no document at all: a control whose
/// mechanism this engine does not implement is a taxonomy-grained finding.
/// `consequence` reports it and `instance_validity` must not, because no
/// document of this corpus stopped validating.
#[test]
fn a_finding_about_the_taxonomy_alone_is_consequence_and_not_instance_validity() {
    let root = Root::new("taxonomy-grained");
    root.publish("1.0.0");
    root.edit(
        "version: 2.0.0",
        &[(
            "controls:\n\n  CT-COV-1:",
            "controls:\n\n  CT-NEW-1:\n    mechanism: phase:runner.telepathy\n    \
             discharges: [OB-COV-1]\n    trigger: pull_request\n    posture: advisory\n    \
             acts: detective\n\n  CT-COV-1:",
        )],
    );
    let candidate = root.publish("2.0.0");

    let ran = root.run(&[
        "taxonomy",
        "diff",
        candidate.to_str().expect("utf-8"),
        "--now",
        "2026-08-01",
    ]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    ran.states_no_contradiction();
    assert!(
        ran.dimension("consequence").starts_with("BROKEN"),
        "a rule the new base breaks is a consequence: {ran:?}"
    );
    assert_eq!(
        ran.dimension("instance_validity"),
        "preserved",
        "no document stopped validating, so the two populations are not one: {ran:?}"
    );
    assert!(
        ran.out.contains("control.mechanism.unimplemented"),
        "and the report names the rule: {ran:?}"
    );
}

/// `classification`, `identifier` and `projection` go red, and nothing else
/// proves that they can.
///
/// Every case above asserts these three `preserved`, which is a treatment arm
/// that proves the subject and not the instrument: a dimension wired to a
/// comparison that can never differ passes all three of them. So this case
/// moves the shelf that types the one document of the fixture corpus, and each
/// of the three has to say so.
///
/// One edit reaches all three because they are three readings of one event. A
/// document that matches no shelf resolves to no kind, so it leaves the census
/// as untyped, it leaves the identifier index, and it leaves every projection
/// that lists the shelf it was on.
#[test]
fn a_shelf_that_moves_reaches_classification_identifier_and_projection() {
    let root = Root::new("shelf-moves");
    root.publish("1.0.0");
    root.edit(
        "version: 2.0.0",
        &[("path: docs/decisions/**", "path: docs/rulings/**")],
    );
    let candidate = root.publish("2.0.0");

    let ran = root.run(&[
        "taxonomy",
        "diff",
        candidate.to_str().expect("utf-8"),
        "--now",
        "2026-08-01",
    ]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    ran.states_no_contradiction();
    for dimension in ["classification", "identifier", "projection"] {
        assert!(
            ran.dimension(dimension).starts_with("BROKEN"),
            "`{dimension}` cannot go red, so no case above measures it: {ran:?}"
        );
    }
    assert!(
        ran.out
            .contains("docs/decisions/0001-the-warrant-a-person-set.md"),
        "the report names the document that changed kind: {ran:?}"
    );
    assert!(
        ran.out.contains("HW-DR-0001"),
        "and the identifier that stopped resolving: {ran:?}"
    );
    assert!(
        ran.out.contains("this change requires a major version"),
        "{ran:?}"
    );
}
