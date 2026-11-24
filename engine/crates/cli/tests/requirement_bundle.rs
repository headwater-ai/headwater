// SPDX-License-Identifier: Apache-2.0
//! The `requirement-verification` library entry, from the state an adopter is
//! in: a published `headwater/standard`, a consumer that selects the entry, and
//! an overlay that carries nothing a bundle may not (#2017).
//!
//! # The defect this target exists for
//!
//! Until #2017 the requirement, the acceptance criterion and the verification
//! were declared only in this repository's own overlay, about 200 lines of
//! declarations under almost 300 lines of reasoning. An adopter who wanted the
//! three kinds had to copy that block, and a copy is a fork nobody upgrades. The
//! first case below is what the entry promises instead: a consumer overlay of
//! namespace leaves and one anchor pattern, and the three documents linked by
//! `headwater new` alone.
//!
//! The second case is criterion 6 of the library index. The `brd-prd` entry
//! already holds `purposes.requirement`, `regimes.voice.prospective` and
//! `shelves.requirements` at `docs/requirements/**`, so this entry renames all
//! three. An adopter who selects both entries is the reader that rename serves,
//! and a resolve that refuses that selection is the failure it prevents.
//!
//! Both cases run the built binary over the real maintained source, published
//! the way `taxonomy publish --from` publishes it, because a correct bundle that
//! the publish leaves out is still a missing entry.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
}

/// A scratch tree that removes itself. The label keeps two cases apart, because
/// cargo runs them as threads of one process and the pid alone names one
/// directory for both.
struct Root(PathBuf);

impl Root {
    fn scratch(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-requirement-bundle-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the scratch tree is made");
        Root(at)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run(root: &Path, arguments: &[&str]) -> Ran {
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(arguments)
        .arg("--root")
        .arg(root)
        .output()
        .expect("the binary runs");
    Ran {
        code: output.status.code(),
        out: String::from_utf8_lossy(&output.stdout).into_owned(),
        err: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().expect("the path has a parent"))
        .expect("the parent is made");
    std::fs::write(path, text).expect("the file is written");
}

/// The real maintained source, published into `<root>/release`, and the
/// version and digest a consumer pins it by.
fn publish(root: &Root) -> (PathBuf, String, String) {
    let artifact = root.path().join("release");
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .arg("taxonomy")
        .arg("publish")
        .arg("--from")
        .arg(repository().join("taxonomy-source/headwater-standard"))
        .arg("--out")
        .arg(&artifact)
        .arg("--root")
        .arg(repository())
        .output()
        .expect("the binary runs");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        artifact
            .join("bundles/requirement-verification/bundle.yml")
            .is_file(),
        "the publish did not carry the requirement-verification entry"
    );
    let release = headwater_resolve::release::read(
        &std::fs::read_to_string(artifact.join("release.yml")).expect("the release reads"),
    )
    .expect("the published record reads");
    let manifest =
        std::fs::read_to_string(artifact.join("package.yml")).expect("the manifest reads");
    let version = manifest
        .lines()
        .find_map(|line| line.strip_prefix("version: "))
        .expect("the manifest declares a version")
        .trim()
        .to_string();
    (artifact, version, release.digest)
}

/// A consumer of the published package that selects `bundles`, with `overlay`
/// as its whole overlay, vendored and resolved.
fn consumer(root: &Root, bundles: &str, overlay: &str) -> PathBuf {
    let (artifact, version, digest) = publish(root);
    let consumer = root.path().join("consumer");
    write(
        &consumer.join(".headwater/taxonomy.yml"),
        &format!(
            "taxonomy:\n  package: headwater/standard\n  version: {version}\n  digest: {digest}\n  bundles: [{bundles}]\n  overlay: .headwater/overlay.yml\ncorpus:\n  root: docs\n"
        ),
    );
    write(&consumer.join(".headwater/overlay.yml"), overlay);
    std::fs::create_dir_all(consumer.join("docs")).expect("the corpus root is made");
    let vendored = run(
        &consumer,
        &[
            "taxonomy",
            "vendor",
            artifact.to_str().expect("the artifact is UTF-8"),
        ],
    );
    assert_eq!(vendored.code, Some(0), "{vendored:?}");
    let resolved = run(&consumer, &["taxonomy", "resolve"]);
    assert_eq!(resolved.code, Some(0), "{resolved:?}");
    consumer
}

/// Everything an adopter writes to take the three kinds: a namespace for each
/// scheme the selected entries leave without one, and the prefix a test cites.
/// No kind, relation, shelf or purpose is declared here.
///
/// The selection names `decision-record` as well. The entry requires
/// `evidence-and-obligation` for `purposes.evidence`, and that bundle alone
/// declares `purposes.obligation` with no kind to serve it, so purpose
/// completeness refuses it until `decision-record` or `design-spec` supplies
/// the kind. The doctrine records that as a finding.
const OVERLAY: &str = "add:\n  identifier_schemes.decision_id.namespace: ACME\n  identifier_schemes.obligation_record_id.namespace: ACME\n  identifier_schemes.requirement_id.namespace: ACME\n  identifier_schemes.acceptance_criterion_id.namespace: ACME\n  identifier_schemes.verification_id.namespace: ACME\n  anchors.test_site.pattern: \"ACME-VER-\"\n";

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

#[test]
fn an_adopter_selects_the_entry_and_links_a_requirement_a_criterion_and_a_verification() {
    let root = Root::scratch("adopter");
    assert!(
        OVERLAY.lines().count() < 10,
        "the adopter overlay is meant to hold leaves and nothing else"
    );
    let consumer = consumer(
        &root,
        "evidence-and-obligation, decision-record, requirement-verification",
        OVERLAY,
    );

    let made = run(
        &consumer,
        &["new", "requirement", "--title", "Replay a failed webhook"],
    );
    assert_eq!(made.code, Some(0), "{made:?}");
    let made = run(
        &consumer,
        &[
            "new",
            "acceptance_criterion",
            "--title",
            "A replay delivers within one minute",
            "--facet",
            "verification_method=test",
            "--relates",
            "verifies=ACME-REQ-0001",
        ],
    );
    assert_eq!(made.code, Some(0), "{made:?}");
    let made = run(
        &consumer,
        &[
            "new",
            "verification",
            "--title",
            "The replay latency test",
            "--relates",
            "proves=ACME-AC-0001",
        ],
    );
    assert_eq!(made.code, Some(0), "{made:?}");

    let requirement_path =
        consumer.join("docs/verified-requirements/0001-replay-a-failed-webhook.md");
    let criterion_path =
        consumer.join("docs/acceptance-criteria/0001-a-replay-delivers-within-one-minute.md");
    let verification_path = consumer.join("docs/verifications/0001-the-replay-latency-test.md");

    let criterion = read(&criterion_path);
    assert!(criterion.contains("id: ACME-AC-0001"), "{criterion}");
    assert!(
        criterion.contains("verification_method: test"),
        "{criterion}"
    );
    assert!(criterion.contains("verifies:"), "{criterion}");
    let verification = read(&verification_path);
    assert!(verification.contains("id: ACME-VER-0001"), "{verification}");
    assert!(verification.contains("proves:"), "{verification}");

    // A draft owes its far half only once it leaves `draft`, so a fresh trio
    // passes a strict run with the near halves alone.
    let checked = run(&consumer, &["check", "--strict"]);
    assert_eq!(checked.code, Some(0), "{checked:?}");

    // All three are settled, and `check --fix` writes the two far halves the
    // scaffolder named. The requirement then reaches
    // the criterion over `verified_by` and the criterion reaches the
    // verification over `proven_by`, with no hand edit to either relation.
    for path in [&requirement_path, &criterion_path, &verification_path] {
        let text = read(path).replacen("status: draft\n", "status: current\n", 1);
        std::fs::write(path, text).expect("the state is set");
    }
    let fixed = run(&consumer, &["check", "--fix"]);
    assert_eq!(fixed.code, Some(0), "{fixed:?}");

    let requirement = read(&requirement_path);
    assert!(requirement.contains("id: ACME-REQ-0001"), "{requirement}");
    assert!(
        requirement.contains("verified_by:\n    - ACME-AC-0001"),
        "{requirement}"
    );
    let criterion = read(&criterion_path);
    assert!(
        criterion.contains("proven_by:\n    - ACME-VER-0001"),
        "{criterion}"
    );

    let checked = run(&consumer, &["check", "--strict"]);
    assert_eq!(checked.code, Some(0), "{checked:?}");
    assert!(
        checked.out.contains("\n  0 findings\n"),
        "a settled trio reports nothing, a missing half or an unverified requirement included\n{}\n{}",
        checked.out,
        checked.err
    );
}

#[test]
fn the_entry_resolves_beside_brd_prd_because_it_renames_the_three_addresses_brd_prd_holds() {
    let root = Root::scratch("beside-brd-prd");
    let overlay = format!(
        "{OVERLAY}  identifier_schemes.brd_id.namespace: ACME\n  identifier_schemes.prd_id.namespace: ACME\n"
    );
    let consumer = consumer(
        &root,
        "brd-prd, evidence-and-obligation, decision-record, requirement-verification",
        &overlay,
    );
    let validated = run(&consumer, &["taxonomy", "validate"]);
    assert_eq!(validated.code, Some(0), "{validated:?}");
    let checked = run(&consumer, &["check", "--strict"]);
    assert_eq!(checked.code, Some(0), "{checked:?}");
}

/// The two participation expectations the entry ships, read past their
/// 90-day windows. A requirement that a criterion verifies is settled, and one
/// that nothing verifies is reported. A criterion with the method `test` that
/// no verification proves is reported, and one with the method `inspection` is
/// not read at all, because HW-DR-0073 leaves the cadence of an inspection
/// open.
#[test]
fn past_the_window_the_entry_reports_an_unverified_requirement_and_an_unproved_test_criterion_and_nothing_else(
) {
    let root = Root::scratch("expectations");
    let consumer = consumer(
        &root,
        "evidence-and-obligation, decision-record, requirement-verification",
        OVERLAY,
    );
    for arguments in [
        vec!["new", "requirement", "--title", "Verified"],
        vec!["new", "requirement", "--title", "Unverified"],
        vec![
            "new",
            "acceptance_criterion",
            "--title",
            "Tested",
            "--facet",
            "verification_method=test",
            "--relates",
            "verifies=ACME-REQ-0001",
        ],
        vec![
            "new",
            "acceptance_criterion",
            "--title",
            "Inspected",
            "--facet",
            "verification_method=inspection",
            "--relates",
            "verifies=ACME-REQ-0001",
        ],
    ] {
        let made = run(&consumer, &arguments);
        assert_eq!(made.code, Some(0), "{made:?}");
    }
    for path in [
        "docs/verified-requirements/0001-verified.md",
        "docs/verified-requirements/0002-unverified.md",
        "docs/acceptance-criteria/0001-tested.md",
        "docs/acceptance-criteria/0002-inspected.md",
    ] {
        let path = consumer.join(path);
        let text = read(&path).replacen("status: draft\n", "status: current\n", 1);
        std::fs::write(&path, text).expect("the state is set");
    }
    let fixed = run(&consumer, &["check", "--fix"]);
    assert_eq!(fixed.code, Some(0), "{fixed:?}");

    let checked = run(&consumer, &["check", "--now", "2099-01-01"]);
    assert_eq!(checked.code, Some(0), "{checked:?}");
    let overdue: Vec<&str> = checked
        .out
        .lines()
        .filter(|line| line.contains("relation.participation.overdue (OB-REL-3)"))
        .collect();
    assert_eq!(overdue.len(), 2, "{}", checked.out);
    assert!(
        checked
            .out
            .contains("docs/verified-requirements/0002-unverified.md:"),
        "the requirement nothing verifies is not reported\n{}",
        checked.out
    );
    assert!(
        checked
            .out
            .contains("docs/acceptance-criteria/0001-tested.md:"),
        "the test criterion no verification proves is not reported\n{}",
        checked.out
    );
    assert!(
        !checked
            .out
            .contains("docs/acceptance-criteria/0002-inspected.md:"),
        "an inspection criterion is reported, and HW-DR-0073 gives it no remedy\n{}",
        checked.out
    );
    assert!(
        !checked
            .out
            .contains("docs/verified-requirements/0001-verified.md:"),
        "a requirement that a criterion verifies is reported\n{}",
        checked.out
    );
}
