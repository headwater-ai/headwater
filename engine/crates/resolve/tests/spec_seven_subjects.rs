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
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
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

/// The clause of spec 7 that the ruling sentence opens with, which is what the
/// extractor anchors on.
const RULING_OPENS: &str = "The vocabulary holds no fourth subject";

/// Spec 7's own ruling sentence, read out of the specification at run time.
///
/// Typing the sentence here would be a second copy of it, and a reword of
/// spec 7 would leave the copy standing. Reading it means the package record is
/// held against whatever spec 7 says today: a reword of the ruling reddens the
/// record until the record follows.
fn spec_seven_ruling() -> String {
    let (path, section) = migration_payload_section();
    let start = section.find(RULING_OPENS).unwrap_or_else(|| {
        panic!(
            "{}: `{SECTION}` no longer holds a sentence opening {RULING_OPENS:?}, so nothing \
             states the ruling the package record has to carry",
            path.display()
        )
    });
    let rest = &section[start..];
    let end = rest.find(". ").unwrap_or_else(|| {
        panic!(
            "{}: the ruling sentence of `{SECTION}` does not end, so the extractor cannot bound \
             it. It reads:\n\n{rest}",
            path.display()
        )
    });
    rest[..=end].to_string()
}

/// Wordings that put the answer on the specification rather than take it.
///
/// A curated family and not a complete one. Each member says the same thing:
/// that this project owes a mechanism it has not built. `facet.required.missing`
/// is the break every one of them was written about, and spec 7 has ruled it.
/// The list is the second line of this case rather than the first, because a
/// blocklist over prose saturates: a phrasing nobody listed passes it. What
/// holds the record is the positive assertion below, which reads spec 7's own
/// sentence and requires the record to carry it.
///
/// `pending a decision` rather than `pending`, because the paragraph has to name
/// `migration-pending` and a bare stem would refuse the route it is asking for.
const DEFERRALS: [&str; 8] = [
    "finding against spec 7",
    "an open question",
    "yet to answer",
    "yet to be answered",
    "unanswered",
    "not yet decided",
    "pending a decision",
    "a gap in spec 7",
];

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
/// # What this holds, and what it does not
///
/// The first assertion is the one that holds the ruling. It reads spec 7's own
/// ruling sentence out of the specification and requires the record to carry it
/// word for word, so the record cannot state the absence in its own terms at
/// all, and a reword of spec 7 reddens the record rather than passing it.
///
/// The second is a curated family of deferring phrasings. It is a blocklist and
/// it saturates: a paraphrase nobody listed passes it. The residual hole is a
/// paragraph that carries the ruling and contradicts it in a neighboring
/// sentence, which reads as agreement to every lexical case and as nonsense to a
/// person. Nothing here closes that, and no engine value exists to bind it to.
///
/// # Watched failing
///
/// Restoring "which is a finding against spec 7 rather than an omission of this
/// release" reddens the deferral assertion, printing the paragraph. Deleting the
/// quoted ruling reddens the first, printing the sentence spec 7 holds and the
/// record does not. Rewording that sentence in spec 7 reddens the first too,
/// which is what proves it reads the specification rather than a list typed
/// here. Deleting the `headwater infer` sentence reddens the route assertion.
#[test]
fn the_package_record_of_the_facet_required_absence_follows_the_spec_7_ruling() {
    let ruling = spec_seven_ruling();

    for relative in RECORDS {
        let path = repo_root().join(relative);
        let paragraph = facet_required_paragraph(&path);
        let lowered = paragraph.to_lowercase();

        assert!(
            paragraph.contains(&ruling),
            "{}: the 4.0.0 paragraph does not carry the ruling that \
             docs/spec/07-distribution-and-federation.md states, so the record says what the \
             absence means in its own words rather than in the words of the part that ruled it. \
             Spec 7 says:\n\n{ruling}\n\nThe paragraph says:\n\n{paragraph}",
            path.display()
        );

        for deferring in DEFERRALS {
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
/// `headwater taxonomy publish` stages every member by reading the file whole,
/// and it transforms exactly one of them, the package manifest, through a single
/// splice. The taxonomy is carried byte for byte, so the two paths agree after a
/// publish and drift the moment one is edited alone. A member digest is over
/// those bytes, so a drift here is also a pin that no longer names what the
/// publisher edits. Nothing else in this repository reads both paths.
///
/// The assertion is over the whole file rather than over one paragraph, because
/// byte equality is what publish actually gives for this member. A comparison of
/// a fresh publish into a scratch directory against `packages/headwater-standard`
/// returns no differences across all 37 members, which is the measurement behind
/// this sentence.
///
/// # Watched failing
///
/// Editing `taxonomy-source/headwater-standard/taxonomy.yml` without running
/// `headwater taxonomy publish` and `headwater taxonomy vendor` reddens this,
/// naming the first line that differs.
#[test]
fn the_vendored_record_matches_the_source_it_was_published_from() {
    let source = repo_root().join(RECORDS[0]);
    let vendored = repo_root().join(RECORDS[1]);
    let authored =
        std::fs::read_to_string(&source).unwrap_or_else(|e| panic!("{}: {e}", source.display()));
    let shipped = std::fs::read_to_string(&vendored)
        .unwrap_or_else(|e| panic!("{}: {e}", vendored.display()));

    let divergence = authored
        .lines()
        .zip(shipped.lines())
        .position(|(a, b)| a != b)
        .map(|index| {
            let (a, b) = (
                authored.lines().nth(index).unwrap_or_default(),
                shipped.lines().nth(index).unwrap_or_default(),
            );
            format!(
                "first at line {}:\n  source   {a}\n  vendored {b}",
                index + 1
            )
        })
        .unwrap_or_else(|| {
            format!(
                "same on every shared line, and the lengths differ: {} against {}",
                authored.lines().count(),
                shipped.lines().count()
            )
        });

    assert_eq!(
        authored,
        shipped,
        "{} and {} are not the same bytes, so a consumer reads one record and the publisher edits \
         another, and the pinned digest names neither one of them alone. {divergence}\n\nThe chain \
         that keeps them together is `headwater taxonomy publish --from \
         taxonomy-source/headwater-standard`, then the pin in `.headwater/taxonomy.yml`, then \
         `headwater taxonomy vendor`, then `headwater taxonomy resolve`.",
        source.display(),
        vendored.display()
    );
}
