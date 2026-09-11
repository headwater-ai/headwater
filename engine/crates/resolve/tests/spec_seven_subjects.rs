// SPDX-License-Identifier: Apache-2.0
//! Spec 7's *The migration payload* section, held against
//! [`headwater_resolve::migration::Subject`].
//!
//! # Why this file exists
//!
//! The subject vocabulary is stated twice: once as
//! [`Subject::NAMES`](headwater_resolve::migration::Subject::NAMES), which is
//! what a payload may write and what the refusal message prints, and once as a
//! three-row table in `docs/spec/07-distribution-and-federation.md`, which is
//! what a publisher reads before writing one. Until this file, nothing held the
//! two together in either direction. [#543](https://github.com/headwater-ai/headwater/issues/543)
//! reported the sharper form of the same gap: both lines of the paragraph that
//! states what the vocabulary cannot express were deleted, and the corpus
//! measured byte for byte the same, because no check reads that section at all.
//!
//! Zero migration payloads have ever been written across three majors of
//! `headwater/standard`. That is not a reason to skip this file, it is the
//! reason nothing else would catch a drift here: no run of any verb over this
//! repository's own corpus ever reaches a payload, so a fourth `Subject` arm
//! landing without a table row, or a table row surviving an arm that was
//! removed, would be invisible to every other suite.
//!
//! # The model
//!
//! `engine/crates/generate/tests/spec_six_projections.rs` holds spec 6's
//! projection-kinds block against `Kind::ALL` the same way, and the two design
//! points are taken from it. First, the block carries **engine identifiers**
//! rather than the prose the paragraph uses, so there is no prose-to-identifier
//! mapping to drift. Second, the comparison runs in **both** directions, so
//! neither an emptied table nor an unnamed arm passes.
//!
//! # What this does not hold
//!
//! The second and third columns of the table are prose about what a step moves
//! and which compatibility dimension it remedies. `Subject` carries neither as
//! a string, and giving it one would put a second copy of the sentence in the
//! engine and let a reword pass. Only the first column is bound here.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use headwater_resolve::migration::Subject;

/// The heading the section opens with, and the one the section ends before.
const SECTION: &str = "### The migration payload";

/// The header row of the subject table, which is what the extractor anchors on.
///
/// A header row rather than a sentence, because the section holds a second
/// table-shaped construct in no other place and the header names the column
/// this file reads.
const HEADER: &str = "| subject | what it moves | the dimension it is a remedy for |";

fn spec_seven() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../docs/spec/07-distribution-and-federation.md")
}

/// The text of spec 7's *The migration payload* section, from its heading to
/// the next heading of any level.
fn migration_payload_section() -> (PathBuf, String) {
    let path = spec_seven();
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let (_, after) = text
        .split_once(SECTION)
        .unwrap_or_else(|| panic!("{}: no `{SECTION}` heading", path.display()));
    let section: String = after
        .lines()
        .skip(1)
        .take_while(|line| !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !section.trim().is_empty(),
        "{}: `{SECTION}` is empty, so every case here would read nothing",
        path.display()
    );
    (path, section)
}

/// The backticked names in the first column of the subject table, in the order
/// a reader meets them.
fn subject_rows() -> Vec<String> {
    let (path, section) = migration_payload_section();
    let (_, after_header) = section
        .split_once(HEADER)
        .unwrap_or_else(|| panic!("{}: `{SECTION}` holds no row `{HEADER}`", path.display()));

    let mut rows = Vec::new();
    for line in after_header.lines().skip(1) {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            break;
        }
        let cell = trimmed
            .trim_start_matches('|')
            .split('|')
            .next()
            .unwrap_or_default()
            .trim();
        if cell.chars().all(|c| c == '-' || c == ':') {
            continue;
        }
        let name = cell
            .strip_prefix('`')
            .and_then(|c| c.strip_suffix('`'))
            .unwrap_or_else(|| {
                panic!(
                    "{}: the subject table's first column holds {cell:?}, and a subject name is \
                 written in backticks so that a reader can tell it from prose",
                    path.display()
                )
            });
        rows.push(name.to_string());
    }
    rows
}

/// The subject table of spec 7 names exactly the subjects a payload may write.
///
/// The decisive case. It reddens if a row is deleted, if a row is added, and if
/// a fourth `Subject` arm lands without spec 7 gaining a row — which is the
/// drift #543 exists to prevent, in whichever direction the owner's ruling
/// eventually sends it.
///
/// # Watched failing in three directions
///
/// Adding a `| `facet_required` | … |` row reddens the first assertion, naming
/// `facet_required`. Deleting the `overlay_address` row reddens the second,
/// naming `overlay_address`. Adding a fourth arm to `Subject::NAMES` reddens
/// the second too, which is what proves this reads the engine rather than a
/// list typed here.
#[test]
fn the_subject_table_of_spec_7_names_the_subjects_a_payload_may_write() {
    let rows = subject_rows();
    let named: BTreeSet<&str> = rows.iter().map(String::as_str).collect();
    let known: BTreeSet<&str> = Subject::NAMES.into_iter().collect();

    let unknown: Vec<&&str> = named.difference(&known).collect();
    assert!(
        unknown.is_empty(),
        "docs/spec/07-distribution-and-federation.md's subject table names {unknown:?}, which no \
         `headwater_resolve::migration::Subject` arm carries. A payload naming one is refused, and \
         the refusal prints {:?}",
        Subject::NAMES
    );

    let unstated: Vec<&&str> = known.difference(&named).collect();
    assert!(
        unstated.is_empty(),
        "a payload may write {unstated:?} and docs/spec/07-distribution-and-federation.md's \
         subject table gives them no row. The table names {named:?}"
    );

    assert_eq!(
        rows.len(),
        Subject::NAMES.len(),
        "docs/spec/07-distribution-and-federation.md's subject table holds {} rows and a payload \
         may write {} subjects. The rows are {rows:?}",
        rows.len(),
        Subject::NAMES.len()
    );
}

/// No absence stated in *The migration payload* is left as an open question.
///
/// The section states three absences: `projection` and `identifier` have no
/// subject, and a facet a kind starts to require has none either. Each is a
/// ruling a publisher acts on. `headwater taxonomy diff` already routes a
/// publisher whose whole break is `facet.required.missing` to `headwater infer
/// --owner <name> --write` and `migration-pending`, and it prints that as a
/// settled answer. A section that told the same reader the question was still
/// open would leave them unable to tell whether to wait for a mechanism.
///
/// The assertion is lexical because the property is: a deliberate absence is
/// stated as settled, and a question this project has not answered does not
/// reach a publisher through a specification part. There is no engine value to
/// bind it to, so the alternative is no check at all.
///
/// # Watched failing
///
/// Restoring "Whether the vocabulary should hold a fourth subject for this
/// class is an open question" reddens this, printing the sentence.
#[test]
fn no_absence_in_the_migration_payload_section_is_left_as_an_open_question() {
    let (path, section) = migration_payload_section();
    let deferring: Vec<&str> = section
        .lines()
        .filter(|line| line.contains("an open question"))
        .collect();
    assert!(
        deferring.is_empty(),
        "{}: `{SECTION}` states an absence as an open question, and a publisher reading it cannot \
         tell whether to wait for a mechanism:\n\n{}",
        path.display(),
        deferring.join("\n\n")
    );
}

/// The facet-required absence names the route the engine prints, and the
/// condition that reopens it.
///
/// A product decision merges only if it is reversible and recorded with the
/// condition that reopens it, written where the next reader meets it. For this
/// absence the next reader is a publisher reading spec 7, so the condition
/// lives in that paragraph rather than only in a decision record.
///
/// The two route strings are held against the sentence `headwater taxonomy
/// diff` prints for the same break, so the specification and the verb cannot
/// send one publisher two ways.
///
/// # Watched failing
///
/// Deleting the reopening clause reddens the second assertion. Renaming the
/// verb in one of the two places reddens the first, naming which string the
/// paragraph lost.
#[test]
fn the_facet_required_absence_names_its_route_and_the_condition_that_reopens_it() {
    let (path, section) = migration_payload_section();
    let paragraph = section
        .lines()
        .find(|line| line.contains("A facet that a kind starts to require has no subject"))
        .unwrap_or_else(|| {
            panic!(
                "{}: `{SECTION}` no longer states the facet-required absence",
                path.display()
            )
        });

    for route in [
        "headwater infer --owner <name> --write",
        "migration-pending",
    ] {
        assert!(
            paragraph.contains(route),
            "{}: the facet-required paragraph does not name `{route}`, which is what `headwater \
             taxonomy diff` sends a publisher whose whole break is `facet.required.missing` to. \
             The paragraph is:\n\n{paragraph}",
            path.display()
        );
    }

    assert!(
        paragraph.contains("reopen"),
        "{}: the facet-required paragraph rules the absence deliberate and names no condition \
         that reopens it. A ruling with no reopening condition is one a later reader cannot \
         revisit. The paragraph is:\n\n{paragraph}",
        path.display()
    );
}

/// The sentence the 4.0.0 paragraph of the package's own record opens with.
///
/// An anchor rather than a line number, because the record grows a paragraph at
/// the top on every publish and every number below it moves.
const RECORD_ANCHOR: &str = "The artifact ships no migration payload";

/// The two copies of the package's authored record: the source a publish reads,
/// and the vendored artifact a consumer of this repository resolves against.
const RECORDS: [&str; 2] = [
    "taxonomy-source/headwater-standard/taxonomy.yml",
    "packages/headwater-standard/taxonomy.yml",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// The comment paragraph of `path` that states what 4.0.0 ships for the facet
/// that became required, as one line.
///
/// The record wraps its prose at the width of the file, so the paragraph is
/// joined with single spaces before anything reads it. A case that read the
/// raw lines would redden on a reflow that changed no word.
fn facet_required_paragraph(path: &Path) -> String {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut paragraph: Vec<String> = Vec::new();
    let mut found = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        match trimmed.strip_prefix('#') {
            Some(rest) if !rest.trim().is_empty() => {
                let rest = rest.trim();
                paragraph.push(rest.to_string());
                if rest.contains(RECORD_ANCHOR) {
                    found = true;
                }
            }
            _ => {
                if found {
                    break;
                }
                paragraph.clear();
            }
        }
    }
    assert!(
        found,
        "{}: no comment paragraph names {RECORD_ANCHOR:?}, so the record no longer states what \
         4.0.0 ships for a facet that became required",
        path.display()
    );
    paragraph.join(" ")
}

/// The package's own record of 4.0.0 states the facet-required absence the way
/// spec 7 rules it, and names the route instead of a mechanism to wait for.
///
/// The decisive case for Done-when 4 of
/// [#543](https://github.com/headwater-ai/headwater/issues/543). Spec 7 rules
/// the absence deliberate and sends a publisher whose whole break is
/// `facet.required.missing` to `headwater infer --owner <name> --write`, after
/// which `headwater check` reports each pair as `migration-pending`. Until this
/// case, the record of the release that caused exactly that break closed by
/// calling the absence a finding against spec 7, so a publisher who read the
/// package rather than the specification was told to wait for a mechanism that
/// has been ruled will not come.
///
/// Both copies are read, because a publish carries the source into the vendored
/// artifact and nothing else holds the two together.
///
/// # Watched failing
///
/// Restoring "which is a finding against spec 7 rather than an omission of this
/// release" reddens the first assertion, printing the paragraph. Deleting the
/// `headwater infer` sentence reddens the second, naming the route the record
/// lost.
#[test]
fn the_package_record_of_the_facet_required_absence_follows_the_spec_7_ruling() {
    for relative in RECORDS {
        let path = repo_root().join(relative);
        let paragraph = facet_required_paragraph(&path);
        let lowered = paragraph.to_lowercase();

        for deferring in ["finding against spec 7", "an open question"] {
            assert!(
                !lowered.contains(deferring),
                "{}: the 4.0.0 paragraph calls the facet-required absence {deferring:?}. \
                 docs/spec/07-distribution-and-federation.md rules it \"a ruling rather than an \
                 omission\", so this record tells a publisher to wait for a mechanism that will \
                 not come. The paragraph is:\n\n{paragraph}",
                path.display()
            );
        }

        for route in [
            "headwater infer --owner <name> --write",
            "migration-pending",
        ] {
            assert!(
                paragraph.contains(route),
                "{}: the 4.0.0 paragraph does not name `{route}`, which is what `headwater \
                 taxonomy diff` and spec 7 both send a publisher whose whole break is \
                 `facet.required.missing` to. The paragraph is:\n\n{paragraph}",
                path.display()
            );
        }

        assert!(
            lowered.contains("spec 7"),
            "{}: the 4.0.0 paragraph names no specification part for the absence it states, so a \
             reader cannot reach the ruling it obeys. The paragraph is:\n\n{paragraph}",
            path.display()
        );
    }
}

/// The vendored artifact carries the same record as the source it was published
/// from.
///
/// `headwater taxonomy publish` copies the authored taxonomy into the artifact,
/// and a member digest is over the file bytes, so the two agree after a publish
/// and drift the moment one is edited alone. Nothing else in this repository
/// reads both paths. The case is over this one paragraph rather than the whole
/// file, because the paragraph is what a publisher reads and a future publish
/// step is free to add a member the source does not carry.
///
/// # Watched failing
///
/// Editing `taxonomy-source/headwater-standard/taxonomy.yml` without running
/// `headwater taxonomy publish` and `headwater taxonomy vendor` reddens this,
/// printing both paragraphs.
#[test]
fn the_vendored_record_matches_the_source_it_was_published_from() {
    let source = repo_root().join(RECORDS[0]);
    let vendored = repo_root().join(RECORDS[1]);
    let authored = facet_required_paragraph(&source);
    let shipped = facet_required_paragraph(&vendored);
    assert_eq!(
        authored,
        shipped,
        "{} and {} state 4.0.0 differently, so a consumer reads one record and the publisher \
         edits another. The chain that keeps them together is `headwater taxonomy publish \
         --from taxonomy-source/headwater-standard`, then the pin in `.headwater/taxonomy.yml`, \
         then `headwater taxonomy vendor`, then `headwater taxonomy resolve`.",
        source.display(),
        vendored.display()
    );
}
