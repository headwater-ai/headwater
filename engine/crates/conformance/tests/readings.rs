// SPDX-License-Identifier: Apache-2.0
//! The four readings this engine holds, each driven against a tree written for
//! it.
//!
//! # Why `pin.current` gets six arms and the others get two
//!
//! `pin.current` is the reading with branches. The pin is **two numbers** — a
//! version and a digest — and a rule that read the version alone would pass a
//! repository whose pinned digest names an artifact nobody publishes any more.
//! Each of the six states of those two numbers is asserted here, because the one
//! this repository happens to be in exercises exactly one of them.
//!
//! The other three are a comparison each, and the arm that matters for each is
//! the failing one. A reading tested only where it says "met" is a reading whose
//! green answer is the only one anybody measured.

use headwater_census::census::{Census, Outcome, Row, Unreadable, Untyped};
use headwater_conformance::{
    corpus_classified, lock_current, pin_current, projections_current, Verdict,
};
use headwater_resolve::package::Consumer;
use std::path::{Path, PathBuf};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "headwater-conformance-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    dir
}

fn consumer(version: &str, digest: Option<&str>) -> Consumer {
    Consumer {
        package: "acme/taxonomy".to_string(),
        version: version.to_string(),
        bundles: Vec::new(),
        digest: digest.map(str::to_string),
        overlay: None,
        corpus_root: "docs".to_string(),
        exclusions: Vec::new(),
    }
}

/// A package directory under `packages/`, with a manifest and one content file.
fn package(root: &Path, version: &str) -> PathBuf {
    package_at(root, "acme", version)
}

/// The same package, under a directory the caller names.
///
/// The directory name matters to exactly one caller and to nothing else here.
/// `pin_current` finds a package by reading every manifest under `packages/`, so
/// any name serves it. `headwater taxonomy vendor` installs at
/// `packages/<package name with the slash replaced>`, so a test about the route
/// has to put the source on that one path or the collision the route turns on
/// never happens.
fn package_at(root: &Path, directory: &str, version: &str) -> PathBuf {
    let dir = root.join("packages").join(directory);
    std::fs::create_dir_all(&dir).expect("the package directory");
    std::fs::write(
        dir.join("package.yml"),
        format!(
            "package: acme/taxonomy\nversion: {version}\ncontents:\n  taxonomy: taxonomy.yml\n"
        ),
    )
    .expect("the manifest");
    std::fs::write(
        dir.join("taxonomy.yml"),
        format!("taxonomy: acme/taxonomy\nversion: {version}\n"),
    )
    .expect("a source");
    dir
}

/// Write the release record `taxonomy publish` would write, and return its
/// digest. The record is computed rather than typed, so the test cannot pin a
/// number the reader would reject for a reason other than the one under test.
fn publish(dir: &Path) -> String {
    let members = headwater_resolve::release::members(dir).expect("the members");
    let release = headwater_resolve::release::Release {
        package: "acme/taxonomy".to_string(),
        version: "1.0.0".to_string(),
        requires_engine: None,
        digest: headwater_resolve::release::digest_of(&members),
        members,
    };
    std::fs::write(
        dir.join(headwater_resolve::release::RECORD),
        headwater_resolve::release::render(&release),
    )
    .expect("the record");
    release.digest
}

fn gap(verdict: &Verdict) -> String {
    match verdict {
        Verdict::Gap(detail) => detail.clone(),
        other => panic!("expected a gap and got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// pin.current — the two numbers
// ---------------------------------------------------------------------------

/// **Met needs both numbers.** This is the only arm in which the rule passes,
/// and it needs a published artifact and a pin that names it.
#[test]
fn the_pin_is_current_when_the_version_and_the_digest_both_name_what_is_installed() {
    let root = scratch("pin-met");
    let dir = package(&root, "1.0.0");
    let digest = publish(&dir);
    assert_eq!(
        pin_current(&root, &consumer("1.0.0", Some(&digest))),
        Verdict::Met
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// **The arm a version-only rule would get wrong.** The version agrees and the
/// digest does not, so a reading of the version alone would call this met. That
/// repository takes an artifact that nobody publishes any more.
#[test]
fn a_matching_version_with_a_stale_digest_is_a_gap() {
    let root = scratch("pin-stale-digest");
    let dir = package(&root, "1.0.0");
    let published = publish(&dir);
    let stale = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
    assert_ne!(published, stale);
    let detail = gap(&pin_current(&root, &consumer("1.0.0", Some(stale))));
    assert!(detail.contains(stale), "the message names the pin");
    assert!(detail.contains(&published), "and the artifact on disk");
    let _ = std::fs::remove_dir_all(&root);
}

/// A published artifact with nothing pinning it. `vendor` refuses to run with no
/// pin rather than recording what it found, and this is the same fact reported
/// as a gap.
#[test]
fn a_published_artifact_that_nothing_pins_is_a_gap() {
    let root = scratch("pin-unpinned");
    let dir = package(&root, "1.0.0");
    let digest = publish(&dir);
    let detail = gap(&pin_current(&root, &consumer("1.0.0", None)));
    assert!(detail.contains("nothing pins a digest"));
    assert!(detail.contains(&digest));
    let _ = std::fs::remove_dir_all(&root);
}

/// No release record and no pin. **This is the arm this repository is in**: it
/// consumes the package it publishes, from source, so no published artifact
/// stands behind the directory.
#[test]
fn a_package_with_no_release_record_and_no_pin_is_a_gap() {
    let root = scratch("pin-unpublished");
    package(&root, "1.0.0");
    let detail = gap(&pin_current(&root, &consumer("1.0.0", None)));
    assert!(detail.contains("carries no release record"));
    let _ = std::fs::remove_dir_all(&root);
}

/// A pin with nothing to check it against. A consumer who copied a digest out of
/// a directory that carries no record has pinned nothing.
#[test]
fn a_pin_with_no_release_record_to_check_it_against_is_a_gap() {
    let root = scratch("pin-no-record");
    package(&root, "1.0.0");
    let detail = gap(&pin_current(
        &root,
        &consumer("1.0.0", Some("sha256:whatever")),
    ));
    assert!(detail.contains("no release record to check it against"));
    let _ = std::fs::remove_dir_all(&root);
}

/// The version half on its own. A pin that names a version the installed package
/// does not declare is refused before any digest is read, because the message
/// about a version is the one an author acts on.
#[test]
fn a_version_the_installed_package_does_not_declare_is_a_gap() {
    let root = scratch("pin-version");
    let dir = package(&root, "2.0.0");
    let digest = publish(&dir);
    let detail = gap(&pin_current(&root, &consumer("1.0.0", Some(&digest))));
    assert!(detail.contains("1.0.0"));
    assert!(detail.contains("2.0.0"));
    let _ = std::fs::remove_dir_all(&root);
}

/// No package at all. A repository that pins a package nothing installed is not
/// pin-current, and the message says which name it looked for.
#[test]
fn a_package_that_is_not_installed_at_all_is_a_gap() {
    let root = scratch("pin-absent");
    std::fs::create_dir_all(root.join("packages")).expect("an empty packages directory");
    let detail = gap(&pin_current(&root, &consumer("1.0.0", None)));
    assert!(detail.contains("acme/taxonomy"));
    let _ = std::fs::remove_dir_all(&root);
}

// ---------------------------------------------------------------------------
// pin.current — the route its remediation names
// ---------------------------------------------------------------------------
//
// The block above holds the six states of the two numbers. These two hold
// something else: the sequence of actions that carries a repository from the
// gap to the met arm, and whether the string the package ships names it. A
// remediation naming two of five steps sends a reader to a dead end, and a
// green reading of the arm says nothing about that.

/// Each row is one step of the route, and the token is what the remediation must
/// carry for that step to be named at all.
///
/// The destination of step 4 is its own row. A string that names the verb but
/// not the path the verb writes to is the exact defect
/// [#276](https://github.com/headwater-ai/headwater/issues/276) was filed on, so
/// a failure has to be able to say which of the two is absent.
const ROUTE: [(&str, &str); 7] = [
    (
        "step 1 — publish the artifact",
        "headwater taxonomy publish",
    ),
    ("step 1 — no verb of this engine fetches one", "HW-OBL-0085"),
    ("step 2 — write the digest into the pin", "taxonomy.digest"),
    (
        "step 3 — move the existing package directory aside",
        "directory aside",
    ),
    ("step 4 — vendor the artifact", "headwater taxonomy vendor"),
    (
        "step 4 — it installs under `packages/`",
        "under `packages/`",
    ),
    (
        "step 5 — a vendored source moves the lock, so resolve follows",
        "headwater taxonomy resolve",
    ),
];

/// What a remediation string fails to say about [`ROUTE`]: a step it does not
/// name at all, and a step it names out of turn.
///
/// **Order is part of what the string asserts, so a reading of the tokens alone
/// is not enough.** The string numbers its own steps, and the numbers are not
/// decoration: `vendor` refuses over a directory that carries no release record,
/// so a reader who takes step 4 before step 3 meets the very refusal the string
/// exists to route them around. A `contains` over the whole string cannot see
/// that, and swapping two sentences would leave it green. So the position of each
/// token's first occurrence is read, and each named step is held to the step
/// named before it.
fn unsaid(fix: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut previous: Option<(usize, &str)> = None;
    for (label, token) in ROUTE {
        let Some(at) = fix.find(token) else {
            out.push(format!(
                "the remediation of pin.current does not name {label}"
            ));
            continue;
        };
        if let Some((was, earlier)) = previous {
            if at < was {
                out.push(format!(
                    "the remediation of pin.current puts {label} before {earlier}, and the route \
                     runs the other way"
                ));
            }
        }
        previous = Some((at, label));
    }
    out
}

/// **The route, executed, with every step bracketed by the reading that proves
/// it necessary.**
///
/// This is what stops the test below being a snapshot of a string somebody
/// typed. The route is run in process — `headwater-resolve` is already a
/// dependency of this crate — so if the engine starts requiring a step the
/// string does not name, or stops requiring one it does, an assertion here moves.
///
/// Every assertion before the last one is deliberately loose (`assert_ne!`
/// against `Met` rather than a match on the gap text), because the ambient state
/// is a gap and a tight assertion before the change would take the attribution
/// away from the `Met` at the end.
#[test]
fn the_route_the_remediation_names_is_the_route_that_reaches_the_met_arm() {
    let root = scratch("pin-route");

    // The source sits on the path `vendor` installs to. That is the state of a
    // publisher who consumes the package it maintains, and it is the state the
    // whole route turns on: without the collision, step 3 has no reason.
    let installed = package_at(&root, "acme-taxonomy", "1.0.0");
    assert_eq!(
        installed,
        root.join("packages").join("acme-taxonomy"),
        "the source and the destination of a vendor are one path"
    );
    assert_ne!(pin_current(&root, &consumer("1.0.0", None)), Verdict::Met);

    // Step 1 — publish. The digest is what the verb wrote, never a number typed
    // into the test.
    let artifact = root.join("artifact");
    let digest = headwater_resolve::package::publish(&root, "acme/taxonomy", &artifact)
        .expect("step 1 publishes")
        .digest;

    // Step 2 — the pin, and steps 1 and 2 together do not reach the rule. This
    // is the dead end the issue was filed from: a digest written beside a
    // maintained source is a number nothing accepts.
    assert_ne!(
        pin_current(&root, &consumer("1.0.0", Some(&digest))),
        Verdict::Met,
        "step 4 is necessary, so the two steps the old string named do not reach the rule"
    );

    // Step 3, omitted. The refusal is why the move is a step of the route and
    // not an afterthought, and it is the step the string never named.
    let refused = headwater_resolve::package::vendor(&root, &artifact, &digest)
        .expect_err("a vendor over a maintained source is refused");
    let message = headwater_resolve::render_errors(&refused);
    assert!(message.contains("somebody maintains"), "{message}");
    assert!(
        message.contains("Move it before vendoring over it"),
        "{message}"
    );
    assert!(message.contains("packages/acme-taxonomy"), "{message}");

    // Step 3 — the move.
    std::fs::rename(&installed, root.join("aside")).expect("the source moves aside");
    assert_ne!(
        pin_current(&root, &consumer("1.0.0", Some(&digest))),
        Verdict::Met
    );

    // Step 4 — vendor, and the artifact lands under `packages/` at the package's
    // own directory, carrying the release record the source never had.
    headwater_resolve::package::vendor(&root, &artifact, &digest).expect("step 4 installs it");
    assert!(
        installed.join(headwater_resolve::release::RECORD).is_file(),
        "the installed directory carries the release record that the source did not"
    );

    // Step 5 is `taxonomy resolve`, and `lock.current` owns it. `pin_current`
    // reads the release record and the pin, and never the lock, so the rule is
    // reached here and the lock is the next rule's business.
    assert_eq!(
        pin_current(&root, &consumer("1.0.0", Some(&digest))),
        Verdict::Met
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// **The string this package ships names every step of that route, in the order
/// the route runs.**
///
/// The remediation is the only copy of those instructions, and nothing but this
/// reads it. A reading that compared the string to itself would pass on any
/// string, so the table is the reading and the string is only the subject.
///
/// **The last two thirds calibrate the instrument rather than the subject**, and
/// each arm of [`unsaid`] gets its own calibration, because an arm asserted only
/// where it passes is an arm whose green answer is the only one anybody measured.
/// `PRIOR` is the remediation this one replaced: it names steps 1 and 2 and the
/// verb of step 4, so a table that reports it as complete is a table that reports
/// anything as complete. `OUT_OF_TURN` names every step and puts two of them the
/// wrong way round, which no reading of the tokens alone can see. Both belong here
/// rather than in a pull request body, because a demonstration that lives in prose
/// is a demonstration nobody runs again.
#[test]
fn the_remediation_of_pin_current_names_every_step_of_the_route_in_order() {
    let text = std::fs::read_to_string(
        repository_root().join("packages/headwater-standard/conformance.yml"),
    )
    .expect("this repository's conformance rule set");
    let set = headwater_conformance::read(&text, "headwater/standard").expect("the rule set reads");
    // The parsed value, never the file bytes. A token that straddles a YAML fold
    // is one space in the parsed string and a newline in the file, so a read of
    // the bytes would fail for a reason that is not this rule.
    let fix = &set
        .rule("pin.current")
        .expect("the package declares pin.current")
        .remediation;

    let faults = unsaid(fix);
    assert!(
        faults.is_empty(),
        "{}\n\nthe whole string:\n{fix}",
        faults.join("\n")
    );

    // The remediation before #276, which named step 1, step 2, and the verb of
    // step 4 with neither its destination nor the move it needs.
    const PRIOR: &str = "Publish the package with `headwater taxonomy publish --out <dir>`, and \
                         write the digest it prints into `taxonomy.digest`. Where the package \
                         arrived from elsewhere, run `headwater taxonomy vendor <dir>`, which \
                         refuses an artifact that is not the pinned one.";
    assert_eq!(
        unsaid(PRIOR),
        [
            "the remediation of pin.current does not name step 1 — no verb of this engine fetches \
             one",
            "the remediation of pin.current does not name step 3 — move the existing package \
             directory aside",
            "the remediation of pin.current does not name step 4 — it installs under `packages/`",
            "the remediation of pin.current does not name step 5 — a vendored source moves the \
             lock, so resolve follows",
        ],
        "the table has to report the string it replaced as incomplete, and to report \
         steps 1, 2 and the vendor verb as named, or it discriminates nothing. It also \
         reports no step out of turn, because the three that string does name are in \
         order, so this holds the order arm against a false positive as well"
    );

    // Every step named and two of them the wrong way round. `vendor` refuses over
    // a directory that carries no release record, so a reader who took step 4
    // before step 3 would meet that refusal — the string is wrong in practice
    // rather than merely untidy, and a reading of the tokens alone calls it
    // complete.
    const OUT_OF_TURN: &str = "headwater taxonomy publish writes one, HW-OBL-0085 records the \
                               fetch, taxonomy.digest takes the number, then run `headwater \
                               taxonomy vendor` and it installs under `packages/`, then move \
                               the existing directory aside, then headwater taxonomy resolve.";
    assert_eq!(
        unsaid(OUT_OF_TURN),
        ["the remediation of pin.current puts step 4 — vendor the artifact before step 3 — move \
          the existing package directory aside, and the route runs the other way"],
        "a string that names every step in the wrong order has to be reported, or the \
         word `order` in this test's name is held by nothing"
    );
}

// ---------------------------------------------------------------------------
// lock.current
// ---------------------------------------------------------------------------

/// The failing arm, and it is the one that matters. The lock records the digest
/// of every source it was written from, so a source whose bytes moved is a lock
/// that no run can reproduce.
#[test]
fn a_source_that_moved_since_the_lock_was_written_is_a_gap() {
    let root = scratch("lock-moved");
    let lock = headwater_lock::at(&repository_root()).expect("this repository's lock reads");
    // The lock names sources relative to a root, and this scratch root holds
    // none of them, so every source reads as moved.
    let detail = gap(&lock_current(&root, &lock));
    assert!(detail.contains("moved since the lock was written"));
    assert!(detail.contains(".headwater/overlay.yml"));
    let _ = std::fs::remove_dir_all(&root);

    // The met arm, over the tree the lock was actually written from.
    assert_eq!(
        lock_current(&repository_root(), &lock),
        Verdict::Met,
        "this repository's committed lock is current, and `taxonomy resolve --check` agrees"
    );
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

// ---------------------------------------------------------------------------
// corpus.classified
// ---------------------------------------------------------------------------

fn row(path: &str, outcome: Outcome) -> Row {
    Row {
        path: path.to_string(),
        outcome,
        document: None,
        digest: None,
    }
}

/// **The population is the two outcomes that leave a file with no kind and no
/// stated reason.** The four that state one are asserted in the same test,
/// because a reading that counted an excluded file would report a gap on every
/// repository that declares an exclusion.
#[test]
fn the_reading_counts_a_file_with_no_kind_and_no_stated_reason_and_no_other() {
    let accounted = Census {
        rows: vec![
            row(
                "docs/a.md",
                Outcome::Typed {
                    kind: "decision".to_string(),
                    derivation: Box::new(headwater_census::Resolution {
                        outcome: headwater_census::resolve::Outcome::Typed("decision".to_string()),
                        steps: Vec::new(),
                        span: None,
                    }),
                },
            ),
            row(
                "docs/b.md",
                Outcome::Generated {
                    projection: Some("shelf_index".to_string()),
                    kind: None,
                    derivation: None,
                },
            ),
            row(
                "docs/c.md",
                Outcome::Excluded {
                    pattern: "docs/c.md".to_string(),
                    reason: "it states one".to_string(),
                },
            ),
            row("docs/LICENSE", Outcome::NotADocument),
        ],
    };
    assert_eq!(corpus_classified(&accounted), Verdict::Met);

    let untyped = Census {
        rows: vec![row(
            "docs/loose.md",
            Outcome::Untyped(Untyped::NoFrontMatter),
        )],
    };
    let detail = gap(&corpus_classified(&untyped));
    assert!(detail.contains("docs/loose.md"));
    assert!(detail.contains("1 file under the corpus root carries"));

    let unreadable = Census {
        rows: vec![row(
            "docs/bytes.md",
            Outcome::Unreadable(Unreadable::NotText),
        )],
    };
    assert!(gap(&corpus_classified(&unreadable)).contains("docs/bytes.md"));
}

// ---------------------------------------------------------------------------
// projections.current
// ---------------------------------------------------------------------------

/// The failing arm. A declared projection whose committed file is not what the
/// plan produces is a derived file that somebody edited unseen, and a reader
/// then trusts a file that is wrong.
#[test]
fn a_projection_that_is_not_what_the_plan_produces_is_a_gap() {
    let root = scratch("projection-drift");
    std::fs::write(root.join("index.md"), "what somebody typed\n").expect("the committed file");
    let plan = headwater_generate::Plan {
        outputs: vec![headwater_generate::Output {
            path: "index.md".to_string(),
            kind: headwater_generate::Kind::ShelfIndex,
            bytes: "what the plan produces\n".to_string(),
        }],
        ..Default::default()
    };
    let detail = gap(&projections_current(&root, &plan));
    assert!(detail.contains("index.md"));

    // The met arm, over the same plan and the bytes it produces.
    std::fs::write(root.join("index.md"), "what the plan produces\n")
        .expect("the regenerated file");
    assert_eq!(projections_current(&root, &plan), Verdict::Met);
    let _ = std::fs::remove_dir_all(&root);
}
