// SPDX-License-Identifier: Apache-2.0
//! Every declared expectation form has a probe of the real corpus written for
//! it.
//!
//! # Why this reads the repository rather than a fixture
//!
//! [`Expectation`] is a closed set of five, and `fixtures/` holds a probe for
//! every one of them, so the grader is covered whatever this corpus writes. The
//! gap that fixture coverage cannot see is a different one: a form the engine
//! grades and **this corpus has never written a task for**. A campaign priced
//! over such a form is a campaign over probes that do not exist, which is what
//! [#88](https://github.com/headwater-ai/headwater/issues/88) recorded and what
//! [#531](https://github.com/headwater-ai/headwater/issues/531) answered for
//! `not_opened`, `cited` and `patched`.
//!
//! A fixture cannot hold that, because a fixture is a probe this crate wrote
//! for itself. Only the real shelf answers it.
//!
//! The precedent is `contract.rs` beside this file, which holds a document of
//! the real corpus to the engine from a crate test and states there why the
//! reader of that document is somewhere else.
//!
//! # The grain
//!
//! The enumeration is [`Expectation::ALL`] rather than a hand-written list of
//! five, so a sixth form added to the enum fails here on the day it is added
//! and not on the day somebody remembers this file. The corpus side is every
//! document on the probe shelf, and a document there that declares no
//! `expectation` is a failure rather than a skip: a filter that quietly drops
//! what it cannot read is a test that passes by finding nothing.
//!
//! The same rule applies to the identifier, and it did not at first. The walk
//! used to drop a document that declared no `id`, which is how the shelf index
//! was separated from the probes, and a probe whose `id` line was deleted then
//! left the denominator in silence. `identifier.unusable` catches that under
//! `headwater check --strict`, so it was never a live gap, and a test whose own
//! denominator depends on a second gate to be right is a test that reports a
//! number it did not establish. The index is separated by its missing front
//! matter fence instead, and a fenced document with no identifier fails here.

use headwater_probe::plan::{declared_answers, ANSWERS, EXPECTED};
use headwater_probe::Expectation;
use std::path::{Path, PathBuf};

/// The shelf the `probe` kind is placed on, which `.headwater/overlay.yml`
/// declares and `headwater new probe` writes into. Named here rather than read
/// from the overlay because a move of the shelf should fail this test loudly
/// and be answered by an edit here, the same way `contract.rs` names its
/// document.
const SHELF: &str = "docs/probes";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

/// One probe document of the corpus: the file it is, and what it declares.
///
/// Both declarations are optional here, because the question this file asks is
/// what the shelf declares rather than whether the declaration is admissible.
/// An absent one is a finding of a case below and never a reason to drop the
/// document from the set.
struct Probe {
    path: String,
    id: Option<String>,
    expectation: Option<String>,
    /// The closed set and the expected values under `## Expectation`, sorted.
    answers: Vec<String>,
    expected: Vec<String>,
}

/// One scalar of the front matter, read without a taxonomy, because this test
/// asks what the shelf declares and not whether the declaration is admissible.
/// `headwater check` and `headwater probe plan` ask the second question.
fn scalar(document: &headwater_doc::Document, key: &str) -> Option<String> {
    document
        .facets
        .get(key)?
        .value
        .as_scalar()
        .map(|it| it.text.clone())
}

/// Every document on the probe shelf, in path order.
///
/// The shelf index `README.md` opens with no front matter fence, which is how
/// it is separated from the probes on the shelf without naming the index by
/// file name and without reading a facet that a probe may also be missing.
fn probes() -> Vec<Probe> {
    let shelf = repository_root().join(SHELF);
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&shelf)
        .unwrap_or_else(|e| panic!("{}: {e}", shelf.display()))
        .map(|entry| entry.expect("a shelf entry").path())
        .filter(|path| path.extension().is_some_and(|it| it == "md"))
        .collect();
    entries.sort();

    let mut out = Vec::new();
    for path in entries {
        let source =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        // A file with no front matter block is not a governed document at all.
        // The shelf index `README.md` is the one on this shelf, and it opens
        // with a generated-artifact marker rather than a `---` fence.
        if !source.starts_with("---\n") {
            continue;
        }
        let Ok(document) = headwater_doc::parse(&source) else {
            // A document that opens a front matter block and does not parse is
            // the check layer's finding and not this test's, but it must not
            // vanish from the denominator by being read as an index.
            panic!("{}: the front matter does not parse", path.display());
        };
        let name = path
            .file_name()
            .expect("a file name")
            .to_string_lossy()
            .into_owned();
        out.push(Probe {
            path: format!("{SHELF}/{name}"),
            expectation: scalar(&document, "expectation"),
            id: scalar(&document, "id"),
            answers: sorted(declared_answers(&document, ANSWERS)),
            expected: sorted(declared_answers(&document, EXPECTED)),
        });
    }
    assert!(
        !out.is_empty(),
        "{SHELF} holds no document with front matter, so this test would pass over an empty \
         shelf. Either the shelf moved or the walk is wrong."
    );
    out
}

/// Every form the grader evaluates has a task written for it in this corpus.
///
/// This is the one assertion that stops the corpus sliding back to the state
/// #88 hit, where a powered campaign was proposed over forms nobody had written
/// a probe for.
#[test]
fn every_expectation_form_has_a_probe_of_this_corpus() {
    let written = probes();

    let unwritten: Vec<&str> = Expectation::ALL
        .into_iter()
        .filter(|form| {
            !written
                .iter()
                .any(|probe| probe.expectation.as_deref() == Some(form.name()))
        })
        .map(Expectation::name)
        .collect();

    assert!(
        unwritten.is_empty(),
        "{} of the {} declared expectation forms have no probe in {SHELF}: {}.\nThe grader \
         evaluates every form, so a form with no task is a form a campaign would price and could \
         never run. Write one through `headwater new probe`.\nWritten today: {}",
        unwritten.len(),
        Expectation::ALL.len(),
        unwritten.join(", "),
        written
            .iter()
            .map(|probe| format!(
                "{} ({})",
                probe.id.as_deref().unwrap_or("no identifier"),
                probe.expectation.as_deref().unwrap_or("no expectation")
            ))
            .collect::<Vec<_>>()
            .join(", "),
    );
}

/// A probe on the shelf declares an identifier.
///
/// This is the guard on the walk above rather than a rule of its own. Without
/// it the walk would have to drop a document with no `id` to keep the shelf
/// index out, and a probe whose `id` line was deleted would leave the
/// denominator of every case here without saying so.
#[test]
fn every_probe_on_the_shelf_declares_an_identifier() {
    let unnamed: Vec<String> = probes()
        .into_iter()
        .filter(|probe| probe.id.is_none())
        .map(|probe| probe.path)
        .collect();

    assert!(
        unnamed.is_empty(),
        "{} on {SHELF} opens a front matter block and declares no `id`: {}.\nA document with no \
         identifier is the end of no edge, and the cases in this file would count it under a form \
         nothing can name.",
        unnamed.len(),
        unnamed.join(", "),
    );
}

/// A probe on the shelf declares a form, and the form is one the engine knows.
///
/// Without this, the test above passes for a corpus whose probes all declare
/// `expectaton: opened`, because a misspelled key reads as one more unwritten
/// form only when some other probe covers the form it meant.
#[test]
fn every_probe_on_the_shelf_declares_a_form_the_engine_grades() {
    let bad: Vec<String> = probes()
        .into_iter()
        .filter(|probe| {
            probe
                .expectation
                .as_deref()
                .and_then(Expectation::read)
                .is_none()
        })
        .map(|probe| {
            format!(
                "{} declares `expectation: {}`",
                probe.path,
                probe.expectation.as_deref().unwrap_or("<absent>")
            )
        })
        .collect();

    assert!(
        bad.is_empty(),
        "{}.\nThe closed set is: {}",
        bad.join("; "),
        Expectation::ALL
            .into_iter()
            .map(Expectation::name)
            .collect::<Vec<_>>()
            .join(", "),
    );
}

fn sorted(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

/// Every `answered` probe on the shelf expects a proper part of its closed set.
///
/// `headwater probe plan` refuses the run otherwise, but only when somebody
/// plans one, and nobody plans a paid run to find out. #1229 found the one
/// `answered` probe on this shelf declaring every word its task offered as its
/// set, and the grader then passed every in-domain answer. This case holds the
/// shelf to the plan's rule on every change.
#[test]
fn every_answered_probe_on_the_shelf_expects_a_proper_part_of_its_set() {
    let bad: Vec<String> = probes()
        .into_iter()
        .filter(|probe| probe.expectation.as_deref() == Some(Expectation::Answered.name()))
        .filter(|probe| {
            probe.expected.is_empty()
                || probe.expected.len() >= probe.answers.len()
                || probe
                    .expected
                    .iter()
                    .any(|value| !probe.answers.contains(value))
        })
        .map(|probe| {
            format!(
                "{} declares `{ANSWERS}: [{}]` and `{EXPECTED}: [{}]`",
                probe.path,
                probe.answers.join(", "),
                probe.expected.join(", ")
            )
        })
        .collect();

    assert!(
        bad.is_empty(),
        "{}.\nThe expected values are the answer key, and a key that holds every value of the set \
         passes every answer a recorder can extract.",
        bad.join("; ")
    );
}

/// The five shapes of a probe that a shallow strategy fails (#1472, clause 2).
///
/// The #1384 re-run put the absent arm at the ceiling, because a search by
/// name answered every probe. Each shape below is one way a name search or a
/// one-document read reaches a wrong answer, and the case after this enum
/// holds one probe of the shelf to each.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    /// The words of the task occur in no identifier, file name or title of
    /// the governing document.
    VocabularyMismatch,
    /// A name search reaches a superseded document first, and its value is a
    /// value of the closed set.
    SupersessionTrap,
    /// The expected value comes from two or more documents and from no one of
    /// them alone.
    MultiDocument,
    /// Many documents share the terms of the task, and one or more of them
    /// state another value of the closed set.
    Distractors,
    /// The oracle reports a finding over a patch that touches the asked file
    /// alone.
    ChangeTask,
}

impl Shape {
    const ALL: [Shape; 5] = [
        Shape::VocabularyMismatch,
        Shape::SupersessionTrap,
        Shape::MultiDocument,
        Shape::Distractors,
        Shape::ChangeTask,
    ];
}

/// What one synthetic session of a harder probe did, in the terms a recorder
/// writes.
enum Session {
    /// The session read these paths and gave no answer.
    Read(&'static [&'static str]),
    /// The session gave this final answer.
    Answered(&'static str),
    /// The session produced these artifacts, each with the rules that
    /// reported over it.
    Produced(&'static [(&'static str, &'static [&'static str])]),
}

/// One harder probe of the shelf, with the session a shallow strategy
/// produces and the session a sound one produces.
struct Harder {
    id: &'static str,
    shape: Shape,
    /// The documents an `opened` probe examines, as identifier and path. The
    /// case holds each path to its identifier and the probe to its `examines`
    /// list, so the path here cannot drift from the shelf.
    examines: &'static [(&'static str, &'static str)],
    /// For the vocabulary shape: the content words of the task. Each is in
    /// the task and in no identifier, file name or title of an examined
    /// document.
    words: &'static [&'static str],
    /// Documents whose `status` the shape depends on, as path and status.
    statuses: &'static [(&'static str, &'static str)],
    shallow: Session,
    sound: Session,
}

const ASKED_EVALUATION: &str = "docs/evaluations/governs-edges-what-an-anchor-reaches-what-covers-it-when-it-ages-and-where-a-session-meets-it.md";
const GOVERNING_OBLIGATION: &str =
    "docs/obligations/0104-a-governs-edge-reaches-the-path-it-names-and-nothing.md";

const HARDER: &[Harder] = &[
    Harder {
        id: "HW-PROBE-a-session-finds-a-ruling-from-a-task-in-plain-words",
        shape: Shape::VocabularyMismatch,
        examines: &[
            (
                "HW-DR-0049",
                "docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md",
            ),
            (
                "HW-EVAL-why-corpus-counts-are-derived",
                "docs/evaluations/why-corpus-counts-are-derived-not-stored.md",
            ),
        ],
        words: &["branches", "page", "total", "shared", "merges", "conflict", "wrong"],
        statuses: &[],
        // A name search for the task's words reaches files whose names hold
        // `merge` or `page`, and none of them is the ruling.
        shallow: Session::Read(&[
            ".githooks/merge-regenerate",
            "docs/process/decisions/0020-merges-go-through-the-github-merge-queue-one-squash-commit-per-pull-request.md",
            "docs/decisions/0050-q50-where-the-visual-register-of-the-hand-built-pages-lives.md",
        ]),
        sound: Session::Read(&[
            ".githooks/merge-regenerate",
            "docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md",
        ]),
    },
    Harder {
        id: "HW-PROBE-a-session-says-where-a-figure-on-the-site-is-written",
        shape: Shape::SupersessionTrap,
        examines: &[],
        words: &[],
        statuses: &[
            ("docs/decisions/0039-q39-how-a-figure-reaches-a-hand-built-page.md", "superseded"),
            (
                "docs/decisions/0097-a-figure-on-a-hand-built-page-is-measured-when-the-site-is-published-and-the-committed-page-carries-none.md",
                "current",
            ),
        ],
        shallow: Session::Answered("committed"),
        sound: Session::Answered("published"),
    },
    Harder {
        id: "HW-PROBE-a-session-follows-a-citation-from-one-record-to-the-next",
        shape: Shape::MultiDocument,
        examines: &[],
        words: &[],
        statuses: &[
            ("docs/decisions/0061-q61-how-a-recorded-terminal-demonstration-is-held-against-a-run.md", "superseded"),
            ("docs/decisions/0039-q39-how-a-figure-reaches-a-hand-built-page.md", "superseded"),
            (
                "docs/decisions/0097-a-figure-on-a-hand-built-page-is-measured-when-the-site-is-published-and-the-committed-page-carries-none.md",
                "current",
            ),
        ],
        // The one record the task names cites HW-DR-0039 and stops there.
        shallow: Session::Answered("HW-DR-0039"),
        sound: Session::Answered("HW-DR-0097"),
    },
    Harder {
        id: "HW-PROBE-a-session-says-whether-a-command-or-a-hand-writes-a-governs-line",
        shape: Shape::Distractors,
        examines: &[],
        words: &[],
        statuses: &[
            // The probe the campaign of 2026-10-03 ran stays on the shelf
            // as its record, superseded by this one (#1709).
            (
                "docs/probes/a-session-says-how-a-governs-line-reaches-a-document.md",
                "superseded",
            ),
            (
                "docs/decisions/0083-governs-and-traces-to-are-created-by-an-agent-because-a-session-proposes-the-line-and-a-person-types-it.md",
                "superseded",
            ),
            (
                "docs/decisions/0104-an-agent-writes-governs-traces-to-and-cited-in-through-the-verb-and-the-review-of-its-pull-request-is-the-acceptance.md",
                "current",
            ),
        ],
        // The value the superseded ruling and its near copies state.
        shallow: Session::Answered("hand"),
        sound: Session::Answered("command"),
    },
    Harder {
        id: "HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged",
        shape: Shape::ChangeTask,
        examines: &[],
        words: &[],
        statuses: &[],
        // What `probe-transform.sh --oracle-tree` derives over each patch, the
        // derivation a campaign runs. The asked file alone carries the
        // oracle's finding, and the two halves together carry none. The
        // `--oracle-tree` case of `tools/probe/probe-record-fixtures.sh` runs
        // both patches through that script and holds the oracle's place in
        // these lists to the engine, with the oracle read from the probe.
        shallow: Session::Produced(&[(
            ASKED_EVALUATION,
            &["relation.reciprocity.missing", "warrant.evidence.unsupported"],
        )]),
        sound: Session::Produced(&[
            (ASKED_EVALUATION, &["warrant.evidence.unsupported"]),
            (GOVERNING_OBLIGATION, &[]),
        ]),
    },
];

/// The `examines` list of a probe document, as written.
fn examines_of(document: &headwater_doc::Document) -> Vec<String> {
    let Some(relations) = document.facets.get("relations") else {
        return Vec::new();
    };
    let Some(examines) = relations
        .value
        .as_map()
        .and_then(|it| it.get("examines"))
        .and_then(|it| it.value.as_seq())
    else {
        return Vec::new();
    };
    examines
        .iter()
        .filter_map(|it| it.value.as_scalar().map(|s| s.text.clone()))
        .collect()
}

/// The body under `## Task`, which is the whole of what a session is given.
fn task_of(source: &str) -> String {
    let Some(start) = source.find("\n## Task\n") else {
        return String::new();
    };
    let rest = &source[start + "\n## Task\n".len()..];
    let end = rest.find("\n## ").unwrap_or(rest.len());
    rest[..end].to_string()
}

fn front_matter(path: &Path) -> (String, headwater_doc::Document) {
    let source =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let document = headwater_doc::parse(&source)
        .unwrap_or_else(|_| panic!("{}: the front matter does not parse", path.display()));
    (source, document)
}

/// The probe of the shelf this row names, as the grader meets it.
fn selected(row: &Harder) -> headwater_probe::plan::Selected {
    let root = repository_root();
    let probe = probes()
        .into_iter()
        .find(|probe| probe.id.as_deref() == Some(row.id))
        .unwrap_or_else(|| {
            panic!(
                "{} is not on {SHELF}. Each harder shape is held to a probe of the shelf, and a \
                 row with no document grades nothing.",
                row.id
            )
        });
    let (_, document) = front_matter(&root.join(&probe.path));
    // The grader reads the document's list, so the table holds that list and no
    // other. A target the document adds and the table omits is one the shallow
    // session may read, and a subset check would let it pass unseen.
    let declared = examines_of(&document);
    let mut tabled: Vec<&str> = row.examines.iter().map(|(id, _)| *id).collect();
    let mut written: Vec<&str> = declared.iter().map(String::as_str).collect();
    tabled.sort_unstable();
    written.sort_unstable();
    assert_eq!(
        tabled, written,
        "{}: this table and the document's `examines` list name different targets",
        row.id
    );
    for (id, path) in row.examines {
        let (_, target) = front_matter(&root.join(path));
        assert_eq!(
            scalar(&target, "id").as_deref(),
            Some(*id),
            "{path} is not the document {id}"
        );
    }
    let expectation = probe
        .expectation
        .as_deref()
        .and_then(Expectation::read)
        .unwrap_or_else(|| panic!("{} declares no form the grader knows", row.id));
    headwater_probe::plan::Selected {
        path: probe.path.clone(),
        id: row.id.to_string(),
        category: scalar(&document, "probe_category")
            .as_deref()
            .and_then(headwater_probe::Category::read)
            .unwrap_or_else(|| panic!("{} declares no category", row.id)),
        expectation,
        examines: row
            .examines
            .iter()
            .map(|(id, path)| headwater_probe::plan::Examined {
                id: Some(id.to_string()),
                path: path.to_string(),
            })
            .collect(),
        oracle: match expectation {
            Expectation::Patched => scalar(&document, "oracle"),
            _ => None,
        },
        answers: probe.answers.clone(),
        expected: probe.expected.clone(),
    }
}

/// One recorded session of one probe, as the intake would hand it over.
fn record(probe: &str, session: &Session) -> headwater_probe::Record {
    use headwater_probe::intake::{Answer, Call, Event, Produced};
    let (calls, produced, answer) = match session {
        Session::Read(paths) => (
            paths
                .iter()
                .map(|path| Call {
                    tool: "Read".into(),
                    argument: format!("/workspace/{path}"),
                    result: "sha256:0".into(),
                })
                .collect(),
            Vec::new(),
            Answer::Absent,
        ),
        Session::Answered(value) => (Vec::new(), Vec::new(), Answer::Value(value.to_string())),
        Session::Produced(artifacts) => (
            Vec::new(),
            artifacts
                .iter()
                .map(|(path, findings)| Produced {
                    path: path.to_string(),
                    result: "sha256:0".into(),
                    cites: Vec::new(),
                    findings: Some(findings.iter().map(|it| it.to_string()).collect()),
                })
                .collect(),
            Answer::Absent,
        ),
    };
    let calls_made = calls.len();
    headwater_probe::Record {
        identity: None,
        read: 1,
        probes: vec![probe.to_string()],
        sessions: 1,
        calls: calls_made,
        events: vec![Event {
            at: 1,
            probe: probe.to_string(),
            session: "1".into(),
            calls: Some(calls),
            produced: Some(produced),
            answer,
        }],
        rejected: Vec::new(),
        refusal: None,
        declared: 1,
        lock_moved: None,
        read_set_moved: None,
    }
}

fn grade(selected: &headwater_probe::plan::Selected, session: &Session) -> Verdict {
    let results = headwater_probe::Results::over(
        &record(&selected.id, session),
        std::slice::from_ref(selected),
    );
    assert_eq!(results.rows.len(), 1, "one probe, one row");
    let row = &results.rows[0];
    assert_eq!(row.sessions.len(), 1, "one session, one verdict");
    row.sessions[0].verdict.clone()
}

use headwater_probe::grade::Verdict;

/// Each harder probe of the shelf fails the shallow session and passes the
/// sound one, through the grader a campaign uses (#1472, clause 2). The
/// findings of the change task are the ones a campaign's `--oracle-tree`
/// derivation writes, and the case of that flag in
/// `tools/probe/probe-record-fixtures.sh` holds them to the engine.
///
/// A probe that a name search or a one-document read passes measures nothing
/// about the documents, which is how the absent arm of the #1384 re-run sat at
/// the ceiling. So each row grades the session that strategy produces and
/// asserts a miss, and grades the sound session and asserts a pass. The table
/// holds one probe of each of the five shapes, and every probe is a document of
/// the shelf rather than a fixture of this crate.
#[test]
fn every_harder_probe_fails_the_shallow_session_and_passes_the_sound_one() {
    let root = repository_root();
    for shape in Shape::ALL {
        let count = HARDER.iter().filter(|row| row.shape == shape).count();
        assert_eq!(
            count, 1,
            "{shape:?} has {count} probes in the table, and it needs one"
        );
    }

    for row in HARDER {
        let selected = selected(row);

        let shallow = grade(&selected, &row.shallow);
        assert!(
            matches!(shallow, Verdict::NotSatisfied(_)),
            "{} ({:?}): the shallow session is {shallow:?}, and the shape exists so that it misses",
            row.id,
            row.shape
        );
        let sound = grade(&selected, &row.sound);
        assert!(
            matches!(sound, Verdict::Satisfied(_)),
            "{} ({:?}): the sound session is {sound:?}",
            row.id,
            row.shape
        );

        // A trap value outside the closed set grades as no answer, and a
        // recorder may drop it. Inside the set it grades as wrong.
        if let Session::Answered(value) = row.shallow {
            assert!(
                selected.answers.iter().any(|it| it == value),
                "{}: the shallow value `{value}` is not in the closed set {:?}",
                row.id,
                selected.answers
            );
        }

        for (path, status) in row.statuses {
            let (_, document) = front_matter(&root.join(path));
            assert_eq!(
                scalar(&document, "status").as_deref(),
                Some(*status),
                "{} rests on {path} holding `status: {status}`",
                row.id
            );
        }

        if row.shape == Shape::VocabularyMismatch {
            let (source, _) = front_matter(&root.join(&selected.path));
            let task = task_of(&source).to_lowercase();
            assert!(!row.words.is_empty(), "{}: no words to check", row.id);
            for word in row.words {
                assert!(
                    task.contains(word),
                    "{}: `{word}` is not a word of the task",
                    row.id
                );
                for (id, path) in row.examines {
                    let (_, target) = front_matter(&root.join(path));
                    let title = scalar(&target, "title").unwrap_or_default().to_lowercase();
                    let named = [id.to_lowercase(), path.to_lowercase(), title];
                    assert!(
                        !named.iter().any(|it| it.contains(word)),
                        "{}: `{word}` names {id} by its identifier, path or title, so a name \
                         search reaches it",
                        row.id
                    );
                }
            }
        }
    }
}
