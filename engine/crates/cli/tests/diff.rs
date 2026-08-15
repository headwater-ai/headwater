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
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-diff-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");

        let repository = repository();
        copy(&repository.join("packages"), &at.join("packages"));
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
    fn edit(&self, version: &str, edits: &[(&str, &str)]) {
        let taxonomy = self.at.join("packages/headwater-standard/taxonomy.yml");
        let mut text = std::fs::read_to_string(&taxonomy).expect("the taxonomy reads");
        for (from, to) in edits {
            assert!(text.contains(from), "the fixture still carries `{from}`");
            text = text.replacen(from, to, 1);
        }
        std::fs::write(&taxonomy, text).expect("the taxonomy writes");

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
    assert!(
        ran.out.contains("the base resolved to different text"),
        "the taxonomy did move, and this case is about a diff that stays silent \
         anyway: {ran:?}"
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
    assert!(ran.dimension("instance_validity").starts_with("BROKEN"), "{ran:?}");
    assert!(ran.dimension("consequence").starts_with("BROKEN"), "{ran:?}");
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
    assert_eq!(ran.code, Some(1), "a run that could not measure fails: {ran:?}");
    assert!(ran.dimension("addressability").starts_with("BROKEN"), "{ran:?}");
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
        ran.err.contains("five of the six dimensions were not measured"),
        "{ran:?}"
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
        ran.err.contains("not what its own release record says it is"),
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
