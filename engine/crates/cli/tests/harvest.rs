// SPDX-License-Identifier: Apache-2.0
//! A solution corpus resolves an anchor against the export it pinned, and
//! against no other one.
//!
//! [Spec 7](../../../../docs/spec/07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it)
//! says a tier above a corpus reaches it through one anchor kind, that exactly
//! one resolver owns that kind, and that the resolver reads a pinned corpus
//! export. It also says what happens when the export cannot be read: "a
//! pinned export that the tier cannot read is a **finding that names the
//! pin**. It is never a narrower answer, delivered quietly."
//!
//! [#1233](https://github.com/headwater-ai/headwater/issues/1233) is that
//! sentence as a test.
//!
//! # Why both exports hold `SVC-1`
//!
//! Two source repositories share the kind `service` and define it
//! differently, so each one's export holds a document `SVC-1` with its own
//! facets. That is the trap. If a failed lookup in B's export ever fell back to
//! A's, the edge into B would bind to A's `SVC-1` and nothing would report it.
//! So the case that matters deletes B's export and asserts that the edge into
//! B is reported and names B's pin, while A still holds an `SVC-1`.
//!
//! # Why this drives the binary
//!
//! The defect is that no resolver reaches an export, and a resolver set that a
//! test assembles by hand measures the test. So each case runs the built
//! binary, which builds its resolver set in the one place every verb does.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

/// The declarations the scratch overlay adds: a kind on one shelf, one anchor
/// kind per source repository, each naming its own resolver, and one relation
/// into each anchor kind.
const DECLARED: &str = "
  identifier_schemes.solution_note_id:
    pattern: \"{namespace}-SOL-{slug}\"
    namespace: HW
    allocation: minted-once

  kinds.solution_note:
    is_a: governed_document
    purpose: rationale
    voice: declarative
    lifecycle: standard
    language: ste_house
    identifier: {scheme: solution_note_id}

  shelves.solution_notes:
    title: The solution notes
    path: docs/solution/*.md
    homogeneous: true
    kind: solution_note

  anchors.service_in_a: {resolver: export-repo-a}
  anchors.service_in_b: {resolver: export-repo-b}

  relations.uses_service_in_a:
    family: association
    from: [solution_note]
    to:   [service_in_a]
    cardinality: many
    created_by: author

  relations.uses_service_in_b:
    family: association
    from: [solution_note]
    to:   [service_in_b]
    cardinality: many
    created_by: author
";

/// The one document of the solution corpus. It names `SVC-1` in each source
/// repository.
const NOTE: &str = "\
---
id: HW-SOL-checkout
title: The checkout path
summary: The checkout path calls one service in each source repository.
status: current
status_since: 2026-09-01
last_verified: 2026-09-01
relations:
  uses_service_in_a:
    - SVC-1
  uses_service_in_b:
    - SVC-1
---

# The checkout path

The checkout path calls one service in each repository.
";

/// A native export, in the shape `headwater export` writes, that holds one
/// document `SVC-1` of kind `service` with the facet value given.
fn export(tier: &str) -> String {
    format!(
        "{{\"version\":\"0.4.0\",\"export_version\":\"0.4.0\",\
         \"profile\":{{\"name\":\"full\",\"target\":\"native\",\"filtered\":false}},\
         \"graph\":{{\"documents\":[{{\"path\":\"docs/services/svc-1.md\",\"kind\":\"service\",\
         \"id\":\"SVC-1\",\"facets\":{{\"tier\":\"{tier}\"}}}}],\"anchors\":[],\"edges\":[]}}}}\n"
    )
}

const EXPORT_A: &str = "harvest/repo-a.json";
const EXPORT_B: &str = "harvest/repo-b.json";

/// A scratch solution corpus with this repository's taxonomy, the scratch
/// declarations, two committed exports and a pin for each.
struct Root {
    at: PathBuf,
}

impl Root {
    fn new(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-harvest-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");

        let repository = repository();
        copy(
            &repository.join(headwater_resolve::package::PACKAGES),
            &at.join(headwater_resolve::package::PACKAGES),
        );
        copy(
            &repository.join("docs/taxonomies"),
            &at.join("docs/taxonomies"),
        );
        let overlay = std::fs::read_to_string(repository.join(".headwater/overlay.yml"))
            .expect("the overlay reads");
        assert!(
            overlay.contains("\nadd:\n"),
            "the overlay opens its additions with `add:`, where the scratch kinds go"
        );
        std::fs::write(
            at.join(".headwater/overlay.yml"),
            overlay.replacen("\nadd:\n", &format!("\nadd:\n{DECLARED}\n"), 1),
        )
        .expect("the overlay writes");

        let root = Root { at };
        root.write("docs/solution/checkout.md", NOTE);
        root.write(EXPORT_A, &export("gold"));
        root.write(EXPORT_B, &export("bronze"));
        // The files this repository's language regime names outside the corpus
        // root. A scratch corpus that lacks them is refused for that, which is
        // a finding about the scratch and not about the pins.
        for stub in [
            "README.md",
            ".github/CONTRIBUTING.md",
            ".github/ISSUE_TEMPLATE/issue.md",
            ".github/SECURITY.md",
        ] {
            root.write(stub, "# Scratch\n\nThis file is a stub.\n");
        }

        let consumer = std::fs::read_to_string(repository.join(".headwater/taxonomy.yml"))
            .expect("the declaration reads");
        let pins = format!(
            "\nharvests:\n  repo-a:\n    at: {EXPORT_A}\n    digest: {}\n    channel: the tier's nightly harvest job\n    resolver: export-repo-a\n  repo-b:\n    at: {EXPORT_B}\n    digest: {}\n    channel: the tier's nightly harvest job\n    resolver: export-repo-b\n",
            root.digest(EXPORT_A),
            root.digest(EXPORT_B),
        );
        root.write(".headwater/taxonomy.yml", &format!("{consumer}{pins}"));

        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolved.code,
            Some(0),
            "the fixture resolves\n{}{}",
            resolved.out,
            resolved.err
        );
        root
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.at.join(relative);
        std::fs::create_dir_all(path.parent().expect("it has a parent"))
            .expect("the directory is made");
        std::fs::write(path, text).expect("the file writes");
    }

    fn digest(&self, relative: &str) -> String {
        headwater_hash::digest(&std::fs::read(self.at.join(relative)).expect("the export reads"))
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

    /// Every error a check reports on the solution note, one block each: the
    /// header line that names the file and the lines indented under it.
    fn errors(ran: &Ran) -> Vec<String> {
        Root::errors_on(ran, "docs/solution/checkout.md")
    }

    /// Every error a check reports on one file, one block each.
    fn errors_on(ran: &Ran, path: &str) -> Vec<String> {
        let header = format!("  {path}");
        let mut out: Vec<String> = Vec::new();
        let mut open = false;
        for line in ran.out.lines() {
            if line.starts_with("  ") && !line.starts_with("   ") {
                open = line.starts_with(&header) && line.contains("error");
                if open {
                    out.push(line.to_string());
                }
            } else if open && line.starts_with("    ") {
                let last = out.last_mut().expect("a block is open");
                last.push(' ');
                last.push_str(line.trim());
            } else {
                open = false;
            }
        }
        out
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

/// The two streams held apart.
#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
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

/// The note's errors that name an unresolved relation target.
fn unresolved(ran: &Ran) -> Vec<String> {
    Root::errors(ran)
        .into_iter()
        .filter(|block| block.contains("relation.target.unresolved"))
        .collect()
}

/// Both pins read, so both edges bind, each into its own repository.
#[test]
fn an_anchor_into_each_pinned_export_binds_there() {
    let root = Root::new("both");
    let ran = root.run(&["check", "--strict"]);
    assert_eq!(Root::errors(&ran), Vec::<String>::new(), "{ran:?}");
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert!(!ran.out.contains("unbound"), "both edges bind: {ran:?}");
}

/// The case the issue exists for. B's export is gone, with a warm cache. The
/// edge into B is reported and the finding names B's pin, the edge into A still
/// binds, and nothing binds B's edge to the `SVC-1` that A's export holds.
#[test]
fn a_missing_export_is_a_finding_that_names_its_pin_and_never_a_lookup_elsewhere() {
    let root = Root::new("missing");
    let warm = root.run(&["check", "--strict"]);
    assert_eq!(warm.code, Some(0), "the cache is populated: {warm:?}");

    std::fs::remove_file(root.at.join(EXPORT_B)).expect("B's export is removed");
    let ran = root.run(&["check", "--strict"]);
    assert_ne!(ran.code, Some(0), "the strict gate fails: {ran:?}");
    assert!(
        !ran.err.starts_with("0 served from cache"),
        "the second run reads a warm cache, so the verdict that moved is one a key moved: {}",
        ran.err
    );
    let unresolved = unresolved(&ran);
    assert_eq!(unresolved.len(), 1, "only the edge into B: {unresolved:?}");
    let finding = &unresolved[0];
    assert!(finding.contains("uses_service_in_b"), "{finding}");
    assert!(
        finding.contains("`repo-b`") && finding.contains(EXPORT_B),
        "the finding names B's pin and where it is: {finding}"
    );
    assert!(
        !finding.contains("repo-a") && !finding.contains("which this run does not have"),
        "the finding names B's pin, not A's and not a missing resolver: {finding}"
    );
    assert_eq!(
        Root::errors(&ran).len(),
        1,
        "nothing else is wrong: {ran:?}"
    );
}

/// B's export is back with one byte changed, so it is not the pinned artifact.
/// The result is the result of a missing file, and the message says why.
#[test]
fn an_export_that_is_not_the_pinned_artifact_binds_nothing_and_names_the_pin() {
    let root = Root::new("moved");
    let warm = root.run(&["check", "--strict"]);
    assert_eq!(warm.code, Some(0), "the cache is populated: {warm:?}");

    root.write(EXPORT_B, &export("silver"));
    let ran = root.run(&["check", "--strict"]);
    assert_ne!(ran.code, Some(0), "the strict gate fails: {ran:?}");
    let unresolved = unresolved(&ran);
    assert_eq!(unresolved.len(), 1, "only the edge into B: {unresolved:?}");
    let finding = &unresolved[0];
    assert!(finding.contains("uses_service_in_b"), "{finding}");
    assert!(
        finding.contains("`repo-b`") && finding.contains("not the pinned artifact"),
        "the finding names the pin and says the bytes moved: {finding}"
    );
    assert!(!finding.contains("repo-a"), "{finding}");
}

/// An identifier that B's export does not hold is reported against B, even
/// though A's export holds it. The far end owns identity, so nothing looks for
/// it anywhere else.
#[test]
fn an_identifier_only_the_other_export_holds_is_unresolved() {
    let root = Root::new("elsewhere");
    let pinned = std::fs::read_to_string(root.at.join(".headwater/taxonomy.yml"))
        .expect("the declaration reads");
    let before = root.digest(EXPORT_A);
    root.write(EXPORT_A, &export("gold").replace("\"SVC-1\"", "\"SVC-2\""));
    root.write(
        ".headwater/taxonomy.yml",
        &pinned.replace(&before, &root.digest(EXPORT_A)),
    );
    root.write(
        "docs/solution/checkout.md",
        &NOTE.replace(
            "  uses_service_in_b:\n    - SVC-1",
            "  uses_service_in_b:\n    - SVC-2",
        ),
    );
    let ran = root.run(&["check", "--strict"]);
    let unresolved = unresolved(&ran);
    assert_eq!(unresolved.len(), 2, "{unresolved:?}");
    assert!(
        unresolved
            .iter()
            .any(|finding| finding.contains("uses_service_in_b")
                && finding.contains("`repo-b`")
                && finding.contains("SVC-2")),
        "{unresolved:?}"
    );
}

/// The hand-written exports above copy the shape `headwater export` writes.
/// This case holds the copy to the original: it pins what the binary itself
/// exports, and an anchor into a document of that export binds.
#[test]
fn an_export_this_binary_wrote_is_one_the_resolver_reads() {
    let root = Root::new("real");
    let exported = root.run(&["export", "--format", "json"]);
    assert_eq!(exported.code, Some(0), "{exported:?}");
    assert!(exported.out.contains("\"HW-SOL-checkout\""), "{exported:?}");
    let pinned = std::fs::read_to_string(root.at.join(".headwater/taxonomy.yml"))
        .expect("the declaration reads");
    let before = root.digest(EXPORT_A);
    root.write(EXPORT_A, &exported.out);
    root.write(
        ".headwater/taxonomy.yml",
        &pinned.replace(&before, &root.digest(EXPORT_A)),
    );
    root.write(
        "docs/solution/checkout.md",
        &NOTE.replace(
            "  uses_service_in_a:\n    - SVC-1",
            "  uses_service_in_a:\n    - HW-SOL-checkout",
        ),
    );
    let ran = root.run(&["check", "--strict"]);
    assert_eq!(unresolved(&ran), Vec::<String>::new(), "{ran:?}");
    assert_eq!(ran.code, Some(0), "{ran:?}");
}

/// The identifier of the document a filtered profile withholds.
const WITHHELD: &str = "HW-SOL-refund";

/// An identifier that no document of either corpus ever held.
const TYPO: &str = "HW-SOL-chekout";

/// A scratch root that pins, as repository A, a filtered export that this
/// binary wrote of the root itself, under the grain named. The filter
/// withholds the one `draft` note, [`WITHHELD`]. The checkout note then names
/// two documents in A: the withheld one and [`TYPO`].
///
/// The export is one the binary wrote and not a hand-written copy, so the case
/// holds the shape the writer emits against the shape the resolver reads.
fn filtered(label: &str, grain: &str) -> Root {
    filtered_with(
        label,
        grain,
        "{exclude: {status: [draft]}}",
        &[(WITHHELD, "draft")],
        &[WITHHELD, TYPO],
    )
}

/// [`filtered`], with the filter, the notes the scratch root adds as
/// `(identifier, status)`, and the anchors the checkout note writes into A,
/// each as it appears in YAML.
fn filtered_with(
    label: &str,
    grain: &str,
    filter: &str,
    notes: &[(&str, &str)],
    anchors: &[&str],
) -> Root {
    let root = Root::new(label);
    let overlay =
        std::fs::read_to_string(root.at.join(".headwater/overlay.yml")).expect("the overlay reads");
    let anchor = "\nadd_to:\n\n  projections:\n";
    assert_eq!(
        overlay.matches(anchor).count(),
        1,
        "the overlay appends to its projections"
    );
    root.write(
        ".headwater/overlay.yml",
        &overlay.replacen(
            anchor,
            &format!(
                "{anchor}    - kind: graph_export\n      profile: partner\n      format: json\n      output: exports/partner.json\n      filter: {filter}\n      tombstone: {grain}\n\n"
            ),
            1,
        ),
    );
    let resolved = root.run(&["taxonomy", "resolve"]);
    assert_eq!(resolved.code, Some(0), "{resolved:?}");
    for (id, status) in notes {
        root.write(
            &format!("docs/solution/{}.md", id.to_lowercase()),
            &format!(
                "---\nid: {id}\ntitle: The note {id}\nsummary: The note {id} is not settled yet.\nstatus: {status}\nstatus_since: 2026-09-01\nlast_verified: 2026-09-01\n---\n\n# The note {id}\n\nThis note is not settled yet.\n"
            ),
        );
    }

    let exported = root.run(&["export", "--profile", "partner", "--format", "json"]);
    assert_eq!(exported.code, Some(0), "{exported:?}");
    assert!(exported.out.contains("\"filtered\": true"), "{exported:?}");
    for (id, _) in notes {
        assert!(
            !exported.out.contains(id),
            "the withheld identifier {id} reached the export: {exported:?}"
        );
    }
    let pinned = std::fs::read_to_string(root.at.join(".headwater/taxonomy.yml"))
        .expect("the declaration reads");
    let before = root.digest(EXPORT_A);
    root.write(EXPORT_A, &exported.out);
    root.write(
        ".headwater/taxonomy.yml",
        &pinned.replace(&before, &root.digest(EXPORT_A)),
    );
    root.write(
        "docs/solution/checkout.md",
        &NOTE.replace(
            "  uses_service_in_a:\n    - SVC-1",
            &format!(
                "  uses_service_in_a:\n{}",
                anchors
                    .iter()
                    .map(|anchor| format!("    - {anchor}\n"))
                    .collect::<String>()
                    .trim_end()
            ),
        ),
    );
    root
}

/// The decisive case of #1309. Under `counted`, the export lists a digest of
/// each withheld identifier, so an anchor to the withheld document binds as
/// withheld and a typo beside it stays unresolved. Before #1309 both were
/// unresolved, and nothing could tell the two apart.
#[test]
fn a_withheld_identifier_binds_withheld_and_a_typo_stays_unresolved() {
    let root = filtered("counted", "counted");
    let ran = root.run(&["check", "--strict"]);
    let unresolved = unresolved(&ran);
    assert_eq!(unresolved.len(), 1, "{unresolved:?}\n{ran:?}");
    assert!(unresolved[0].contains(TYPO), "{unresolved:?}");
    assert!(
        unresolved[0].contains("not one of the identifiers it withheld"),
        "the reason says the typo is not withheld: {unresolved:?}"
    );
    assert!(
        !ran.out.contains(WITHHELD),
        "a finding named the withheld identifier: {ran:?}"
    );

    // The graph summary of the run counts the edge in the withheld class.
    assert!(
        ran.out
            .lines()
            .chain(ran.err.lines())
            .any(|line| line.trim() == "1 withheld"),
        "the run counts one withheld edge: {ran:?}"
    );

    // The tier's own export carries the withheld edge with the profile that
    // withheld its target, which is the name a reader asks for access by.
    let tier = root.run(&["export", "--profile", "default", "--format", "json"]);
    assert_eq!(tier.code, Some(0), "{tier:?}");
    let loaded = headwater_yaml::load(&tier.out).expect("the tier export loads");
    let withheld: Vec<(String, String)> = loaded
        .value
        .as_map()
        .and_then(|map| map.get("graph"))
        .and_then(|graph| graph.value.as_map())
        .and_then(|graph| graph.get("edges"))
        .and_then(|edges| edges.value.as_seq())
        .expect("the tier export holds `graph.edges`")
        .iter()
        .filter_map(|edge| edge.value.as_map())
        .filter_map(|edge| edge.get("target"))
        .filter_map(|target| target.value.as_map())
        .filter(|target| {
            target
                .get("bound")
                .and_then(|bound| bound.value.as_scalar())
                .is_some_and(|bound| bound.text == "withheld")
        })
        .map(|target| {
            let text = |key: &str| {
                target
                    .get(key)
                    .and_then(|node| node.value.as_scalar())
                    .map(|node| node.text.clone())
                    .unwrap_or_default()
            };
            (text("anchor_kind"), text("rule"))
        })
        .collect();
    assert_eq!(
        withheld,
        vec![("service_in_a".to_string(), "partner".to_string())],
        "{}",
        tier.out
    );
}

/// Under `sealed` the publisher chose that existence is the secret. The export
/// lists nothing, so this tier cannot tell a withheld document from a typo, and
/// both anchors stay unresolved with a reason that names the grain.
#[test]
fn under_a_sealed_grain_a_withheld_identifier_and_a_typo_both_stay_unresolved() {
    let root = filtered("sealed", "sealed");
    let ran = root.run(&["check", "--strict"]);
    let unresolved = unresolved(&ran);
    assert_eq!(unresolved.len(), 2, "{unresolved:?}\n{ran:?}");
    for id in [WITHHELD, TYPO] {
        assert!(
            unresolved
                .iter()
                .any(|finding| finding.contains(id) && finding.contains("`sealed`")),
            "{id} is unresolved and the reason names the sealed grain: {unresolved:?}"
        );
    }
}

/// A `counted` export older than version 1.3 carries a tombstone with no
/// `identifiers` list. The resolver cannot tell a withheld document from a
/// typo there, so a miss stays unresolved, and the reason says that the export
/// predates the list rather than that the string is not withheld.
#[test]
fn a_counted_export_older_than_the_identifier_list_leaves_a_miss_unresolved_and_says_why() {
    let root = Root::new("older");
    let pinned = std::fs::read_to_string(root.at.join(".headwater/taxonomy.yml"))
        .expect("the declaration reads");
    let before = root.digest(EXPORT_A);
    // Version 1.2 is the last one before the list, so it is the boundary a
    // version test has to hold.
    let older = export("gold")
        .replace(
            "\"filtered\":false}",
            "\"filtered\":true,\"tombstone\":\"counted\"},\
             \"tombstones\":[{\"rule\":\"full.exclude.status\",\"documents\":1}]",
        )
        .replace("\"export_version\":\"0.4.0\"", "\"export_version\":\"1.2\"");
    assert!(older.contains("\"tombstones\""), "{older}");
    assert!(older.contains("\"export_version\":\"1.2\""), "{older}");
    root.write(EXPORT_A, &older);
    root.write(
        ".headwater/taxonomy.yml",
        &pinned.replace(&before, &root.digest(EXPORT_A)),
    );
    root.write(
        "docs/solution/checkout.md",
        &NOTE.replace(
            "  uses_service_in_a:\n    - SVC-1",
            "  uses_service_in_a:\n    - SVC-2",
        ),
    );
    let ran = root.run(&["check", "--strict"]);
    let unresolved = unresolved(&ran);
    assert_eq!(unresolved.len(), 1, "{unresolved:?}\n{ran:?}");
    assert!(
        unresolved[0].contains("SVC-2") && unresolved[0].contains("predates"),
        "the reason says the export predates the identifier list: {unresolved:?}"
    );
}

/// An anchor is trimmed before it is looked up, and the digest is over the
/// trimmed string. A quoted anchor with spaces around the withheld identifier
/// binds as withheld, as the plain identifier does.
#[test]
fn a_padded_anchor_to_a_withheld_identifier_binds_withheld() {
    let padded = format!("\"  {WITHHELD}  \"");
    let root = filtered_with(
        "padded",
        "counted",
        "{exclude: {status: [draft]}}",
        &[(WITHHELD, "draft")],
        &[&padded],
    );
    let ran = root.run(&["check", "--strict"]);
    assert_eq!(unresolved(&ran), Vec::<String>::new(), "{ran:?}");
    assert!(
        ran.out
            .lines()
            .chain(ran.err.lines())
            .any(|line| line.trim() == "1 withheld"),
        "the padded anchor binds as withheld: {ran:?}"
    );
}

/// A filter with two rules writes two tombstones, each with its own list. An
/// identifier withheld under either rule binds as withheld, so the resolver
/// reads every list and not only one.
#[test]
fn an_identifier_withheld_under_either_of_two_rules_binds_withheld() {
    let other = "HW-SOL-legacy";
    let root = filtered_with(
        "two-rules",
        "counted",
        "{include: {status: [current, draft]}, exclude: {status: [draft]}}",
        &[(WITHHELD, "draft"), (other, "deprecated")],
        &[WITHHELD, other, TYPO],
    );
    let exported = std::fs::read_to_string(root.at.join(EXPORT_A)).expect("the export reads");
    assert_eq!(
        exported.matches("\"rule\"").count(),
        2,
        "the filter withheld one document under each rule: {exported}"
    );
    let loaded = headwater_yaml::load(&exported).expect("the export loads");
    let mut stones: Vec<(String, Vec<String>)> = loaded
        .value
        .as_map()
        .and_then(|map| map.get("tombstones"))
        .and_then(|stones| stones.value.as_seq())
        .expect("the export holds `tombstones`")
        .iter()
        .filter_map(|stone| stone.value.as_map())
        .map(|stone| {
            let rule = stone
                .get("rule")
                .and_then(|rule| rule.value.as_scalar())
                .map(|rule| rule.text.clone())
                .unwrap_or_default();
            let listed = stone
                .get("identifiers")
                .and_then(|listed| listed.value.as_seq())
                .map(|listed| {
                    listed
                        .iter()
                        .filter_map(|digest| digest.value.as_scalar())
                        .map(|digest| digest.text.clone())
                        .collect()
                })
                .unwrap_or_default();
            (rule, listed)
        })
        .collect();
    stones.sort();
    assert_eq!(
        stones,
        vec![
            (
                "partner.exclude.status".to_string(),
                vec![headwater_hash::digest(WITHHELD.as_bytes())]
            ),
            (
                "partner.include.status".to_string(),
                vec![headwater_hash::digest(other.as_bytes())]
            ),
        ],
        "each tombstone lists the digest of the identifier its own rule withheld, \
         and no other: {exported}"
    );
    let ran = root.run(&["check", "--strict"]);
    let unresolved = unresolved(&ran);
    assert_eq!(unresolved.len(), 1, "{unresolved:?}\n{ran:?}");
    assert!(unresolved[0].contains(TYPO), "{unresolved:?}");
    assert!(
        ran.out
            .lines()
            .chain(ran.err.lines())
            .any(|line| line.trim() == "2 withheld"),
        "both withheld anchors bind as withheld: {ran:?}"
    );
}

/// A 1.3 `counted` export whose filter withheld nothing writes no tombstone,
/// so no list is there to read. The miss can only be a typo, and the reason
/// must not say that the export predates the list.
#[test]
fn a_counted_export_that_withheld_nothing_reports_a_miss_as_not_withheld() {
    let root = filtered_with(
        "none-withheld",
        "counted",
        "{exclude: {status: [draft]}}",
        &[],
        &[TYPO],
    );
    let ran = root.run(&["check", "--strict"]);
    let unresolved = unresolved(&ran);
    assert_eq!(unresolved.len(), 1, "{unresolved:?}\n{ran:?}");
    assert!(
        unresolved[0].contains(TYPO)
            && unresolved[0].contains("not one of the identifiers it withheld")
            && !unresolved[0].contains("predates"),
        "the reason says the typo is not withheld: {unresolved:?}"
    );
}

/// Rewrite the consumer declaration of a scratch root, and assert the one
/// substitution landed.
fn declare(root: &Root, from: &str, to: &str) {
    let declared = std::fs::read_to_string(root.at.join(".headwater/taxonomy.yml"))
        .expect("the declaration reads");
    assert_eq!(declared.matches(from).count(), 1, "{declared}");
    root.write(".headwater/taxonomy.yml", &declared.replacen(from, to, 1));
}

/// Two pins that name one resolver are refused before any graph is built. A
/// run that kept the first and skipped the second would bind an anchor in
/// whichever export registered first, which is the silent answer spec 7
/// forbids. `docs/interfaces/headwater-check.md` promises exit 1 for it.
#[test]
fn two_pins_that_name_one_resolver_refuse_the_run() {
    let root = Root::new("one-name");
    declare(
        &root,
        "    resolver: export-repo-b\n",
        "    resolver: export-repo-a\n",
    );
    let ran = root.run(&["check", "--strict"]);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(
        ran.err.contains("the resolver set is ambiguous") && ran.err.contains("export-repo-a"),
        "the refusal names the resolver two pins share: {ran:?}"
    );
    assert!(
        !ran.out.contains("docs/solution/checkout.md"),
        "no report was written over a graph built from half the pins: {ran:?}"
    );
}

/// A `harvests` entry that does not read refuses the run and names the entry.
/// Read as "no pins", every anchor into it would report a missing resolver
/// rather than the declaration a person has to repair.
#[test]
fn a_harvests_entry_that_does_not_read_refuses_the_run() {
    let root = Root::new("unreadable");
    declare(&root, "    resolver: export-repo-b\n", "");
    let ran = root.run(&["check", "--strict"]);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(
        ran.err
            .contains("the pinned export declarations did not read")
            && ran.err.contains("harvests.repo-b"),
        "the refusal names the entry: {ran:?}"
    );
    assert!(
        !ran.out.contains("docs/solution/checkout.md"),
        "no report was written: {ran:?}"
    );
}

const PIN_RULE: &str = "harvest.pin.unread";
const EXPORT_C: &str = "harvest/repo-c.json";

/// The decisive case of #1311. A third pin names an export that is not there,
/// and no anchor kind names its resolver, so no edge reaches it and the
/// per-anchor finding never fires. Spec 7 says a pinned export the tier cannot
/// read is a finding that names the pin, so the pin itself is reported, once,
/// on the declaration, and the edges into A and B still bind.
#[test]
fn an_unread_pin_that_no_anchor_names_is_a_finding_that_names_the_pin() {
    let root = Root::new("unread");
    declare(
        &root,
        "    resolver: export-repo-b\n",
        &format!(
            "    resolver: export-repo-b\n  repo-c:\n    at: {EXPORT_C}\n    digest: sha256:0000\n    \
             resolver: export-repo-c\n"
        ),
    );
    let ran = root.run(&["check", "--strict"]);
    assert_ne!(ran.code, Some(0), "the strict gate fails: {ran:?}");
    let pins: Vec<String> = Root::errors_on(&ran, ".headwater/taxonomy.yml")
        .into_iter()
        .filter(|block| block.contains(PIN_RULE))
        .collect();
    assert_eq!(pins.len(), 1, "one finding, for C alone: {ran:?}");
    // The head of the message is the rule's own naming of the pin. The
    // resolver's reason also names `repo-c`, so a bare `contains("repo-c")`
    // passes a finding headed by another pin's name.
    assert!(
        pins[0].contains(&format!(
            "`harvests.repo-c` pins an export at `{EXPORT_C}` that binds nothing"
        )),
        "the finding names the pin and where it is: {}",
        pins[0]
    );
    assert!(
        !pins[0].contains("`harvests.repo-a`") && !pins[0].contains("`harvests.repo-b`"),
        "{}",
        pins[0]
    );
    assert!(pins[0].contains("did not read"), "{}", pins[0]);
    assert_eq!(
        unresolved(&ran),
        Vec::<String>::new(),
        "A and B bind: {ran:?}"
    );

    // Both pins that read are silent, and so is a run where C is repaired.
    root.write(EXPORT_C, &export("tin"));
    declare(&root, "sha256:0000", &root.digest(EXPORT_C));
    let ran = root.run(&["check", "--strict"]);
    assert_eq!(
        Root::errors_on(&ran, ".headwater/taxonomy.yml"),
        Vec::<String>::new(),
        "{ran:?}"
    );
    assert_eq!(ran.code, Some(0), "{ran:?}");
}

/// Each pinned export joins the read set with the digest of its bytes, so a
/// gate over a later tree names an export that moved, and names none over the
/// tree the read set was taken from.
#[test]
fn a_pinned_export_joins_the_read_set_and_a_gate_sees_it_move() {
    let root = Root::new("read-set");
    let read_set = root.at.join("clean.readset");
    let read_set = read_set.to_str().expect("the read set path is UTF-8");
    let ran = root.run(&["check", "--read-set", read_set]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    let recorded = std::fs::read_to_string(read_set).expect("the read set reads");
    for (export, digest) in [
        (EXPORT_A, root.digest(EXPORT_A)),
        (EXPORT_B, root.digest(EXPORT_B)),
    ] {
        assert!(
            recorded
                .lines()
                .any(|line| line.contains(export) && line.contains(&digest)),
            "the read set lists `{export}` at {digest}: {recorded}"
        );
    }
    let still = root.run(&["gate", "--read-set", read_set]);
    assert!(!still.out.contains(EXPORT_B), "{still:?}");

    root.write(EXPORT_B, &export("silver"));
    let moved = root.run(&["gate", "--read-set", read_set]);
    assert!(
        moved.out.contains(EXPORT_B),
        "the gate names the export that moved: {moved:?}"
    );
}

/// A pinned export that is absent joins the read set too, with no digest. The
/// gate then carries no verdict across it, over the same tree or once the file
/// appears, because nothing compares. An absent export is the case the rule
/// reports, so it is the case a read set must not drop: dropped, the gate would
/// say nothing about the export at all.
#[test]
fn an_absent_pinned_export_joins_the_read_set_and_a_gate_never_carries_across_it() {
    let root = Root::new("read-set-absent");
    let pinned = export("tin");
    declare(
        &root,
        "    resolver: export-repo-b\n",
        &format!(
            "    resolver: export-repo-b\n  repo-c:\n    at: {EXPORT_C}\n    digest: {}\n    \
             resolver: export-repo-c\n",
            headwater_hash::digest(pinned.as_bytes())
        ),
    );
    let read_set = root.at.join("absent.readset");
    let read_set = read_set.to_str().expect("the read set path is UTF-8");
    let ran = root.run(&["check", "--read-set", read_set]);
    let recorded = std::fs::read_to_string(read_set)
        .unwrap_or_else(|error| panic!("the read set is written: {error}: {ran:?}"));
    assert!(
        recorded.lines().any(|line| line.contains(EXPORT_C)),
        "the read set lists the absent export: {recorded}"
    );
    let unhashed = format!("{EXPORT_C} carried no hash when it was read");
    let still = root.run(&["gate", "--read-set", read_set]);
    assert!(still.out.contains(&unhashed), "{still:?}");

    root.write(EXPORT_C, &pinned);
    let appeared = root.run(&["gate", "--read-set", read_set]);
    assert!(
        appeared.out.contains(&unhashed),
        "the gate names the export that appeared: {appeared:?}"
    );
}

/// `--root .` from inside the repository is the invocation a hook and a
/// person type. The containment test compares resolved paths, so a relative
/// root must hold both pins as under it, and both edges must bind.
#[test]
fn a_relative_root_holds_every_pin_inside_it() {
    let root = Root::new("relative");
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["check", "--strict", "--root", "."])
        .current_dir(&root.at)
        .output()
        .expect("the binary runs");
    let ran = Ran {
        code: output.status.code(),
        out: String::from_utf8_lossy(&output.stdout).into_owned(),
        err: String::from_utf8_lossy(&output.stderr).into_owned(),
    };
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert_eq!(Root::errors(&ran), Vec::<String>::new(), "{ran:?}");
}

/// Two pins that do not read are two findings, each naming its own pin.
#[test]
fn every_unread_pin_is_its_own_finding() {
    let root = Root::new("two-unread");
    std::fs::remove_file(root.at.join(EXPORT_A)).expect("A's export is removed");
    std::fs::remove_file(root.at.join(EXPORT_B)).expect("B's export is removed");
    let ran = root.run(&["check", "--strict"]);
    let pins: Vec<String> = Root::errors_on(&ran, ".headwater/taxonomy.yml")
        .into_iter()
        .filter(|block| block.contains(PIN_RULE))
        .collect();
    assert_eq!(pins.len(), 2, "{ran:?}");
    for (pin, at) in [("repo-a", EXPORT_A), ("repo-b", EXPORT_B)] {
        let head = format!("`harvests.{pin}` pins an export at `{at}` that binds nothing");
        assert!(
            pins.iter().any(|block| block.contains(&head)),
            "{head}: {pins:?}"
        );
    }
}

/// `infer` runs the checks over the same readings `check` does, so an unread
/// pin is a finding it can record as debt. Handed no readings, it would never
/// see the rule.
#[test]
fn infer_sees_an_unread_pin() {
    let root = Root::new("infer");
    std::fs::remove_file(root.at.join(EXPORT_B)).expect("B's export is removed");
    let ran = root.run(&["infer", "--owner", "tier-team"]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    assert!(ran.out.contains(PIN_RULE), "{ran:?}");
}

/// A pin with no digest binds nothing either, and the finding says what to
/// write, so it is reported on the same terms as a missing file.
#[test]
fn a_pin_with_no_digest_is_a_finding_that_says_what_to_write() {
    let root = Root::new("undigested");
    let digest = root.digest(EXPORT_B);
    declare(&root, &format!("    digest: {digest}\n"), "");
    let ran = root.run(&["check", "--strict"]);
    assert_ne!(ran.code, Some(0), "{ran:?}");
    let pins: Vec<String> = Root::errors_on(&ran, ".headwater/taxonomy.yml")
        .into_iter()
        .filter(|block| block.contains(PIN_RULE))
        .collect();
    assert_eq!(pins.len(), 1, "{ran:?}");
    assert!(
        pins[0].contains("repo-b") && pins[0].contains("harvests.repo-b.digest"),
        "{}",
        pins[0]
    );
}

/// `harvests` written as a list reads as no pins on a lenient reader, and every
/// anchor into it would then report a missing resolver. The run refuses and
/// names the block instead.
#[test]
fn a_harvests_block_that_is_not_a_mapping_refuses_the_run() {
    let root = Root::new("list");
    let declared = std::fs::read_to_string(root.at.join(".headwater/taxonomy.yml"))
        .expect("the declaration reads");
    let (head, _) = declared
        .split_once("\nharvests:\n")
        .expect("the fixture declares harvests");
    root.write(
        ".headwater/taxonomy.yml",
        &format!("{head}\nharvests:\n  - repo-a\n  - repo-b\n"),
    );
    let ran = root.run(&["check", "--strict"]);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(
        ran.err
            .contains("the pinned export declarations did not read")
            && ran.err.contains("`harvests`")
            && ran.err.contains("a sequence"),
        "{ran:?}"
    );
}

/// `harvest/repo-b.json` is a clean relative path, and a committed symlink at
/// `harvest` takes the read out of the repository. The run refuses and names
/// the entry rather than binding an anchor to a file outside the root.
#[cfg(unix)]
#[test]
fn a_pin_that_a_symlink_takes_out_of_the_root_refuses_the_run() {
    let root = Root::new("symlink");
    let outside = std::env::temp_dir().join(format!(
        "headwater-cli-harvest-{}-symlink-outside",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&outside);
    std::fs::create_dir_all(&outside).expect("the outside directory is made");
    for export in [EXPORT_A, EXPORT_B] {
        let name = Path::new(export).file_name().expect("it has a name");
        std::fs::copy(root.at.join(export), outside.join(name)).expect("the export copies");
    }
    std::fs::remove_dir_all(root.at.join("harvest")).expect("the directory is removed");
    std::os::unix::fs::symlink(&outside, root.at.join("harvest")).expect("the symlink is made");
    let ran = root.run(&["check", "--strict"]);
    let _ = std::fs::remove_dir_all(&outside);
    assert_eq!(ran.code, Some(1), "{ran:?}");
    assert!(
        ran.err.contains("harvests.repo-a.at") && ran.err.contains(EXPORT_A),
        "{ran:?}"
    );
}

/// The gate's twin of the case above (#1345). A read set is taken over a clean
/// tree, and `harvest` is then replaced by a symlink to an identical copy
/// outside the root. `check` refuses that tree, so a gate over it must not
/// carry the verdict by hashing bytes the tree does not hold: it names each
/// listed export as resolving outside the repository root.
#[cfg(unix)]
#[test]
fn a_gate_does_not_read_a_listed_path_that_a_symlink_takes_out_of_the_root() {
    let root = Root::new("gate-symlink");
    let read_set = root.at.join("clean.readset");
    let read_set = read_set.to_str().expect("the read set path is UTF-8");
    let ran = root.run(&["check", "--read-set", read_set]);
    assert_eq!(ran.code, Some(0), "{ran:?}");
    let escaped = format!("{EXPORT_A} resolves outside the repository root");
    let still = root.run(&["gate", "--read-set", read_set]);
    assert!(!still.out.contains(&escaped), "{still:?}");

    let outside = std::env::temp_dir().join(format!(
        "headwater-cli-harvest-{}-gate-symlink-outside",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&outside);
    std::fs::create_dir_all(&outside).expect("the outside directory is made");
    for export in [EXPORT_A, EXPORT_B] {
        let name = Path::new(export).file_name().expect("it has a name");
        std::fs::copy(root.at.join(export), outside.join(name)).expect("the export copies");
    }
    std::fs::remove_dir_all(root.at.join("harvest")).expect("the directory is removed");
    std::os::unix::fs::symlink(&outside, root.at.join("harvest")).expect("the symlink is made");
    let gated = root.run(&["gate", "--read-set", read_set]);
    let _ = std::fs::remove_dir_all(&outside);
    assert!(
        gated.out.contains(&escaped),
        "the gate names the export a symlink took out of the root: {gated:?}"
    );
    assert!(
        gated
            .out
            .contains(&format!("{EXPORT_B} resolves outside the repository root")),
        "{gated:?}"
    );
}
