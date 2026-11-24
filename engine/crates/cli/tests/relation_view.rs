// SPDX-License-Identifier: Apache-2.0
//! The `relation_view` projection over the worked corpus of the
//! `requirement-verification` library entry (#2018, HW-DR-0114).
//!
//! # The defect this target exists for
//!
//! A traceability matrix that lists only the requirements something verifies
//! reads as complete and is not. The worked corpus holds `SCR-REQ-0002`, which
//! nothing verifies, and two criteria that no verification proves. The first
//! case asserts that each of them is a row with empty cells, so a matrix that
//! dropped one of them fails here.
//!
//! The second half of the first case is the reason the view is held by
//! `generate --check`: an observation recorded against a criterion, and then a
//! one-byte edit of that criterion, moves the cell from `observed` to
//! `suspect`, and the committed copy goes stale.
//!
//! Every case runs the built binary over the real maintained source, published
//! the way `taxonomy publish --from` publishes it, and over the worked corpus
//! assembled the way the entry's `fixtures/README.md` says.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

fn entry() -> PathBuf {
    repository().join("docs/taxonomies/requirement-verification/fixtures")
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
            "headwater-cli-relation-view-{}-{label}",
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

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn copy_tree(from: &Path, to: &Path) {
    for entry in std::fs::read_dir(from).expect("the directory reads") {
        let entry = entry.expect("the entry reads");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("the type reads").is_dir() {
            true => copy_tree(&entry.path(), &target),
            false => {
                std::fs::create_dir_all(to).expect("the directory is made");
                std::fs::copy(entry.path(), &target).expect("the file is copied");
            }
        }
    }
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

/// The worked corpus, assembled as the entry's `fixtures/README.md` says, with
/// `projections` appended to the entry's own overlay. Resolved, and not yet
/// generated.
fn worked(root: &Root, projections: &str) -> PathBuf {
    let (artifact, version, digest) = publish(root);
    let consumer = root.path().join("consumer");
    copy_tree(&entry().join("corpus/docs"), &consumer.join("docs"));
    copy_tree(&entry().join("corpus/src"), &consumer.join("src"));
    copy_tree(
        &entry().join("corpus/.headwater/ids"),
        &consumer.join(".headwater/ids"),
    );
    write(
        &consumer.join(".headwater/overlay.yml"),
        &format!(
            "{}\nadd_to:\n  projections:\n{projections}",
            read(&entry().join("overlay.yml"))
        ),
    );
    write(
        &consumer.join(".headwater/taxonomy.yml"),
        &format!(
            "taxonomy:\n  package: headwater/standard\n  version: {version}\n  digest: {digest}\n  bundles: [evidence-and-obligation, decision-record, requirement-verification]\n  overlay: .headwater/overlay.yml\ncorpus:\n  root: docs\n"
        ),
    );
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

/// The chain the ruling names: a requirement, the criteria that verify it, the
/// verifications that prove each criterion, and the test file each
/// verification cites.
fn chain(output: &str, format: Option<&str>) -> String {
    let format = format
        .map(|format| format!("      format: {format}\n"))
        .unwrap_or_default();
    format!(
        "    - kind: relation_view\n      for: [verified_requirements]\n      relations: [verified_by, proven_by, cited_in]\n      output: {output}\n{format}"
    )
}

/// The table rows of a Markdown view, without the header and the rule.
fn rows(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| line.starts_with("| "))
        .skip(1)
        .map(str::to_string)
        .collect()
}

/// The cells of one Markdown row, trimmed.
fn cells(row: &str) -> Vec<String> {
    row.trim()
        .trim_start_matches('|')
        .trim_end_matches('|')
        .split(" | ")
        .map(|cell| cell.trim().to_string())
        .collect()
}

#[test]
fn every_requirement_is_a_row_and_an_observed_verification_turns_suspect_when_its_criterion_moves()
{
    let root = Root::scratch("markdown");
    let consumer = worked(&root, &chain("docs/matrix.md", None));

    let generated = run(&consumer, &["generate"]);
    assert_eq!(generated.code, Some(0), "{generated:?}");
    let matrix = consumer.join("docs/matrix.md");
    let text = read(&matrix);
    assert!(
        text.starts_with("<!-- headwater:generated relation_view."),
        "the view carries the generated-file marker on its first line\n{text}"
    );

    let rows = rows(&text);
    assert_eq!(
        rows.len(),
        4,
        "three criteria verify SCR-REQ-0001 and nothing verifies SCR-REQ-0002, so the view holds \
         four paths\n{text}"
    );
    let table: Vec<Vec<String>> = rows.iter().map(|row| cells(row)).collect();
    for row in &table {
        assert_eq!(row.len(), 4, "a start column and one per step: {row:?}");
    }

    // The requirement nothing verifies is a row, and every later cell is empty.
    let unverified = table
        .iter()
        .find(|row| row[0].contains("SCR-REQ-0002"))
        .unwrap_or_else(|| panic!("SCR-REQ-0002 is dropped from the view\n{text}"));
    assert!(
        unverified[1..].iter().all(String::is_empty),
        "SCR-REQ-0002 reaches nothing, so its later cells are empty: {unverified:?}"
    );

    // The two criteria no verification proves end in empty cells.
    for criterion in ["SCR-AC-0002", "SCR-AC-0003"] {
        let row = table
            .iter()
            .find(|row| row[1].contains(criterion))
            .unwrap_or_else(|| panic!("{criterion} is dropped from the view\n{text}"));
        assert!(row[0].contains("SCR-REQ-0001"), "{row:?}");
        assert!(
            row[2].is_empty() && row[3].is_empty(),
            "nothing proves {criterion}, so its verification and code cells are empty: {row:?}"
        );
    }

    // The clean path runs to the test file the verification cites.
    let clean = table
        .iter()
        .find(|row| row[1].contains("SCR-AC-0001"))
        .unwrap_or_else(|| panic!("SCR-AC-0001 is dropped from the view\n{text}"));
    assert!(clean[0].contains("SCR-REQ-0001"), "{clean:?}");
    assert!(clean[1].contains("current"), "the status: {clean:?}");
    assert!(clean[2].contains("SCR-VER-0001"), "{clean:?}");
    assert!(
        clean[2].contains("declared"),
        "no snapshot names SCR-VER-0001 yet: {clean:?}"
    );
    assert!(
        clean[3].contains("src/replay_latency_test.rs"),
        "the last column names the code path the chain reaches: {clean:?}"
    );

    // The rows are in order of the start path, then of each later path.
    let first: Vec<&str> = table.iter().map(|row| row[0].as_str()).collect();
    assert!(first[0..3].iter().all(|cell| cell.contains("SCR-REQ-0001")));
    assert!(first[3].contains("SCR-REQ-0002"));
    assert!(table[0][1].contains("SCR-AC-0001"));
    assert!(table[1][1].contains("SCR-AC-0002"));
    assert!(table[2][1].contains("SCR-AC-0003"));

    let held = run(&consumer, &["generate", "--check"]);
    assert_eq!(held.code, Some(0), "a fresh view is current: {held:?}");

    // An observation of the verification moves the cell, and the committed copy
    // is stale until it is written again.
    let observed = run(
        &consumer,
        &["observe", "SCR-VER-0001", "--commit", "abc1234"],
    );
    assert_eq!(observed.code, Some(0), "{observed:?}");
    let stale = run(&consumer, &["generate", "--check"]);
    assert_ne!(
        stale.code,
        Some(0),
        "an observation moves the view, so the committed copy is stale: {stale:?}"
    );
    let generated = run(&consumer, &["generate"]);
    assert_eq!(generated.code, Some(0), "{generated:?}");
    let text = read(&matrix);
    let clean = table_row(&text, "SCR-AC-0001");
    assert!(
        clean[2].contains(" · observed · ") && clean[2].contains("observed at `abc1234`"),
        "the snapshot names SCR-VER-0001 at abc1234 and the criterion has not moved, so the \
         state is `observed` and the observation is the commit: {clean:?}"
    );
    assert!(!clean[2].contains("suspect"), "{clean:?}");

    // One byte of the criterion moves, and the verification is suspect.
    let criterion = consumer
        .join("docs/acceptance-criteria/0001-a-replay-reaches-the-endpoint-within-one-minute.md");
    let edited = read(&criterion).replacen("Over 20 replays", "Over 21 replays", 1);
    std::fs::write(&criterion, edited).expect("the criterion is edited");
    let stale = run(&consumer, &["generate", "--check"]);
    assert_ne!(
        stale.code,
        Some(0),
        "the criterion moved after the snapshot, so the committed view is stale: {stale:?}"
    );
    assert!(
        stale.out.contains("docs/matrix.md") || stale.err.contains("docs/matrix.md"),
        "the stale report names the view: {stale:?}"
    );
    let generated = run(&consumer, &["generate"]);
    assert_eq!(generated.code, Some(0), "{generated:?}");
    let clean = table_row(&read(&matrix), "SCR-AC-0001");
    assert!(
        clean[2].contains("suspect"),
        "SCR-AC-0001 changed after the snapshot, so SCR-VER-0001 is suspect: {clean:?}"
    );
}

/// A link that only the far end writes is still a link, and the rows sort by
/// path whatever order the front matter writes. The requirement drops
/// `SCR-AC-0002` from its own `verified_by` and lists the other two in reverse,
/// while `SCR-AC-0002` still `verifies` it. It also names a check rule first,
/// which the walk reaches before any document, and whose cell sorts after the
/// criteria because its key is the rule name and not a `docs/` path.
#[test]
fn a_link_written_only_at_the_far_end_is_a_row_and_rows_sort_by_path() {
    let root = Root::scratch("far-end");
    let consumer = worked(&root, &chain("docs/matrix.md", None));
    let requirement = consumer
        .join("docs/verified-requirements/0001-an-operator-can-replay-a-failed-delivery.md");
    let text = read(&requirement);
    let edited = text.replacen(
        "    - SCR-AC-0001\n    - SCR-AC-0002\n    - SCR-AC-0003\n",
        "    - section.required.missing\n    - SCR-AC-0003\n    - SCR-AC-0001\n",
        1,
    );
    assert_ne!(
        edited, text,
        "the fixture requirement lists its three criteria"
    );
    write(&requirement, &edited);

    let generated = run(&consumer, &["generate"]);
    assert_eq!(generated.code, Some(0), "{generated:?}");
    let text = read(&consumer.join("docs/matrix.md"));
    let criteria: Vec<String> = rows(&text)
        .iter()
        .map(|row| named(&cells(row)))
        .filter(|row| row[0] == "SCR-REQ-0001")
        .map(|row| row[1].clone())
        .collect();
    assert_eq!(
        criteria,
        [
            "SCR-AC-0001",
            "SCR-AC-0002",
            "SCR-AC-0003",
            "section.required.missing"
        ],
        "SCR-AC-0002 verifies SCR-REQ-0001 from its own end, and the rows are in path order\n{text}"
    );
}

fn table_row(text: &str, criterion: &str) -> Vec<String> {
    rows(text)
        .iter()
        .map(|row| cells(row))
        .find(|row| row[1].contains(criterion))
        .unwrap_or_else(|| panic!("{criterion} is dropped from the view\n{text}"))
}

/// The identifiers a Markdown row names, one per cell, empty where the cell is.
fn named(cells: &[String]) -> Vec<String> {
    cells
        .iter()
        .map(|cell| {
            cell.split(" · ")
                .next()
                .unwrap_or_default()
                .trim_start_matches('[')
                .split(']')
                .next()
                .unwrap_or_default()
                .trim_matches('`')
                .to_string()
        })
        .collect()
}

/// The paths of the Markdown view, as the identifier in each cell, so that a
/// CSV or JSON view of the same chain can be compared with it row for row.
fn markdown_paths(label: &str) -> Vec<Vec<String>> {
    let root = Root::scratch(label);
    let consumer = worked(&root, &chain("docs/matrix.md", None));
    let generated = run(&consumer, &["generate"]);
    assert_eq!(generated.code, Some(0), "{generated:?}");
    rows(&read(&consumer.join("docs/matrix.md")))
        .iter()
        .map(|row| named(&cells(row)))
        .collect()
}

/// `format: csv` writes the rows of the Markdown view, four fields to a cell,
/// under a `#` marker line, and `generate --check` holds it.
#[test]
fn a_csv_view_writes_the_rows_of_the_markdown_view() {
    let expected = markdown_paths("csv-markdown");
    let root = Root::scratch("csv");
    let consumer = worked(&root, &chain("docs/matrix.csv", Some("csv")));
    let generated = run(&consumer, &["generate"]);
    assert_eq!(generated.code, Some(0), "{generated:?}");
    let text = read(&consumer.join("docs/matrix.csv"));
    let mut lines = text.lines();
    assert!(
        lines
            .next()
            .is_some_and(|line| line.starts_with("# headwater:generated relation_view.")),
        "the first line is the marker\n{text}"
    );
    let header = lines.next().expect("a header record");
    assert!(
        header.starts_with("verified_requirements,verified_requirements status,"),
        "{header}"
    );
    assert_eq!(
        header.split(',').count(),
        16,
        "four fields for each of four columns"
    );
    let records: Vec<Vec<String>> = lines
        .map(|line| line.split(',').map(str::to_string).collect::<Vec<String>>())
        .collect();
    let paths: Vec<Vec<String>> = records
        .iter()
        .map(|record| record.iter().step_by(4).cloned().collect())
        .collect();
    assert_eq!(
        paths, expected,
        "the CSV view writes the Markdown view's rows\n{text}"
    );
    let unverified = records
        .iter()
        .find(|record| record[0] == "SCR-REQ-0002")
        .expect("SCR-REQ-0002 is a record");
    assert!(
        unverified[4..].iter().all(String::is_empty),
        "{unverified:?}"
    );
    let clean = records
        .iter()
        .find(|record| record[4] == "SCR-AC-0001")
        .expect("SCR-AC-0001 is a record");
    assert_eq!(
        &clean[8..12],
        &["SCR-VER-0001", "current", "declared", ""],
        "{clean:?}"
    );
    let held = run(&consumer, &["generate", "--check"]);
    assert_eq!(held.code, Some(0), "{held:?}");
}

/// `format: json` writes the rows of the Markdown view as arrays of cells, a
/// missing link as `null`, with the marker as a member.
#[test]
fn a_json_view_writes_the_rows_of_the_markdown_view() {
    let expected = markdown_paths("json-markdown");
    let root = Root::scratch("json");
    let consumer = worked(&root, &chain("docs/matrix.json", Some("json")));
    let generated = run(&consumer, &["generate"]);
    assert_eq!(generated.code, Some(0), "{generated:?}");
    let text = read(&consumer.join("docs/matrix.json"));
    let value = headwater_yaml::load(&text).expect("the view is JSON").value;
    let object = value.as_map().expect("an object");
    assert!(
        object
            .get("headwater:generated")
            .and_then(|node| node.value.as_scalar())
            .is_some_and(|scalar| scalar.text.starts_with("relation_view.")),
        "the marker is a member\n{text}"
    );
    let rows = object
        .get("rows")
        .and_then(|node| node.value.as_seq())
        .expect("rows");
    let paths: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            row.value
                .as_seq()
                .expect("a row is an array")
                .iter()
                .map(|cell| match cell.value.as_map() {
                    None => String::new(),
                    Some(cell) => cell
                        .get("document")
                        .and_then(|node| node.value.as_scalar())
                        .map(|scalar| scalar.text.clone())
                        .or_else(|| {
                            cell.get("code")
                                .and_then(|node| node.value.as_seq())
                                .and_then(|paths| paths.first())
                                .and_then(|path| path.value.as_scalar())
                                .map(|scalar| scalar.text.clone())
                        })
                        .unwrap_or_default(),
                })
                .collect()
        })
        .collect();
    assert_eq!(
        paths, expected,
        "the JSON view writes the Markdown view's rows\n{text}"
    );
    let held = run(&consumer, &["generate", "--check"]);
    assert_eq!(held.code, Some(0), "{held:?}");
}

/// Point 4: a chain may end at code. `governs` reaches a `source-tree`
/// anchor, which states a digest, so the last column names the path and the
/// stamp of the edge: unstamped, then a baseline, then suspect once the file
/// moves.
#[test]
fn a_chain_that_ends_with_governs_writes_the_code_path_and_its_stamp_in_the_last_column() {
    let root = Root::scratch("governs");
    let consumer = worked(
        &root,
        "    - kind: relation_view\n      for: [verified_requirements]\n      relations: [verified_by, proven_by, governs]\n      output: docs/matrix.md\n",
    );
    let verification =
        consumer.join("docs/verifications/0001-the-replay-latency-test-measures-twenty-replays.md");
    let text = read(&verification).replacen(
        "  cited_in:\n",
        "  governs:\n    - src/replay_latency_test.rs\n  cited_in:\n",
        1,
    );
    std::fs::write(&verification, text).expect("the verification governs the test");

    let generated = run(&consumer, &["generate"]);
    assert_eq!(generated.code, Some(0), "{generated:?}");
    let matrix = consumer.join("docs/matrix.md");
    let clean = table_row(&read(&matrix), "SCR-AC-0001");
    assert!(
        clean[3].contains("`src/replay_latency_test.rs`") && clean[3].contains("unstamped"),
        "the last column names the code path, and no digest is recorded on the edge: {clean:?}"
    );
    let empty = table_row(&read(&matrix), "SCR-AC-0002");
    assert!(empty[3].is_empty(), "{empty:?}");

    let stamped = run(&consumer, &["check", "--fix", "--baseline"]);
    assert_eq!(stamped.code, Some(0), "{stamped:?}");
    let stale = run(&consumer, &["generate", "--check"]);
    assert_ne!(stale.code, Some(0), "a stamp moves the view: {stale:?}");
    let generated = run(&consumer, &["generate"]);
    assert_eq!(generated.code, Some(0), "{generated:?}");
    let clean = table_row(&read(&matrix), "SCR-AC-0001");
    assert!(clean[3].contains("baseline"), "{clean:?}");

    let code = consumer.join("src/replay_latency_test.rs");
    let moved = format!("{}\n// one more line\n", read(&code));
    std::fs::write(&code, moved).expect("the test file moves");
    let generated = run(&consumer, &["generate"]);
    assert_eq!(generated.code, Some(0), "{generated:?}");
    let clean = table_row(&read(&matrix), "SCR-AC-0001");
    assert!(
        clean[3].contains("suspect"),
        "the code moved after the stamp: {clean:?}"
    );
}
