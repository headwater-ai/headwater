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
    /// The closed set and the expected values under `## Key`, sorted.
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

/// The three headings of a probe, in order (HW-DR-0111, "One probe shape").
const SHAPE: [&str; 3] = ["Task", "Target", "Key"];

/// The level-two headings of a document, in order.
fn headings_of(document: &headwater_doc::Document) -> Vec<String> {
    use headwater_doc::body::BlockKind;
    document
        .body
        .blocks
        .iter()
        .filter(|block| matches!(block.kind, BlockKind::Heading(2)))
        .map(|block| block.text().trim().to_string())
        .collect()
}

/// The text under one level-two heading, up to the next one.
fn section_of<'a>(source: &'a str, heading: &str) -> &'a str {
    let open = format!("\n## {heading}\n");
    let Some(start) = source.find(&open) else {
        return "";
    };
    let rest = &source[start + open.len()..];
    let end = rest.find("\n## ").unwrap_or(rest.len());
    &rest[..end]
}

/// The destination of every Markdown link in a text, as written: each inline
/// link and each reference definition. A reference definition is a link the
/// site build follows as surely as an inline one, so a check that reads only
/// `](` lets a link out of `docs/` through.
fn links_in(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find("](") {
        let after = &rest[at + 2..];
        let end = after.find(')').unwrap_or(after.len());
        out.push(after[..end].to_string());
        rest = &after[end..];
    }
    for line in text.lines() {
        let line = line.trim_start();
        if let Some(label_end) = line.strip_prefix('[').and_then(|it| it.find("]:")) {
            let destination = line[label_end + 3..].split_whitespace().next();
            if let Some(destination) = destination {
                out.push(destination.trim_matches(['<', '>']).to_string());
            }
        }
    }
    out
}

/// The text of every inline code span in a text, as written.
fn code_spans_in(text: &str) -> Vec<String> {
    text.split('`')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

/// Whether a code span is written as a path: it holds a `/`, or it ends in a
/// file extension. `superseded_by` is not a path, and
/// `.claude/skills/headwater-authoring/SKILL.md` is one. A span written as a
/// path names a file under `Target` whether that file exists or not, so it is
/// counted before it is resolved.
fn is_path(span: &str) -> bool {
    span.contains('/')
        || span.rsplit_once('.').is_some_and(|(stem, extension)| {
            !stem.is_empty()
                && !extension.is_empty()
                && extension.chars().all(|c| c.is_ascii_alphanumeric())
        })
}

/// Every probe on the shelf states one task, one target and one key (#1774,
/// clause 3).
///
/// The headings are exactly `Task`, `Target` and `Key`, in that order, so a
/// probe left on the old `Expectation` heading fails here as well as in the
/// grader. The `Target` section names exactly one file of this repository.
/// A target under `docs/` is one link, and the link resolves to a file there.
/// A target outside `docs/` is a code-path anchor that the site does not
/// serve, and a link to it fails the strict site build, so it is named by
/// exactly one code span holding its path from the repository root, and by no
/// link. Every link counts, inline or by reference definition, and so does
/// every code span written as a path, whether its file exists or not. A path
/// is resolved before it is judged, so it must stay inside the repository,
/// and a detour into `docs/` is a target under `docs/`. For the forms that
/// are a predicate over what a session opened or cited, the target is one of the probe's `examines` entries, so the target
/// and the key read the same document.
///
/// `not_opened` is left out of that last clause on purpose: its target is the
/// document a session answers from, and its `examines` entry is the document
/// it must not open. `answered` and `patched` name no `examines` entry.
#[test]
fn every_probe_on_the_shelf_states_one_task_one_target_and_one_key() {
    let root = repository_root();
    let mut bad = Vec::new();
    let mut seen = 0;
    for probe in probes() {
        seen += 1;
        let (source, _) = front_matter(&root.join(&probe.path));
        if let Some(finding) = shape_finding(&probe.path, &source) {
            bad.push(finding);
        }
    }
    assert!(seen > 0, "the shelf held no probe");
    assert!(
        bad.is_empty(),
        "{} of {seen} probes do not state one task, one target and one key:\n{}",
        bad.len(),
        bad.join("\n")
    );
}

/// What is wrong with the shape of one probe document, or `None`. `path` is
/// the document's path from the repository root, on the probe shelf, because
/// a link under `Target` resolves against that directory.
fn shape_finding(path: &str, source: &str) -> Option<String> {
    let root = repository_root();
    let Ok(document) = headwater_doc::parse(source) else {
        return Some(format!("{path}: the front matter does not parse"));
    };
    let headings = headings_of(&document);
    if headings != SHAPE {
        return Some(format!(
            "{path} has the headings {headings:?}, not {SHAPE:?}"
        ));
    }
    let section = section_of(source, "Target");
    let links = links_in(section);
    let spans: Vec<String> = code_spans_in(section)
        .into_iter()
        .filter(|span| is_path(span))
        .collect();
    if links.len() + spans.len() != 1 {
        if spans.is_empty() && !links.is_empty() {
            return Some(format!(
                "{path} holds {} links under `## Target`, not one: {links:?}",
                links.len()
            ));
        }
        return Some(format!(
            "{path} names {} files under `## Target`, not one: links {links:?}, paths {spans:?}",
            links.len() + spans.len()
        ));
    }
    let Ok(home) = root.canonicalize() else {
        return Some(format!("{path}: the repository root does not resolve"));
    };
    let served = home.join("docs");
    let (named, target) = if let [link] = links.as_slice() {
        let directory = root.join(path);
        let directory = directory.parent().unwrap_or(&root);
        let file = link.split('#').next().unwrap_or_default();
        let Ok(target) = directory.join(file).canonicalize() else {
            return Some(format!("{path} targets `{link}`, which is no file"));
        };
        if !target.starts_with(&served) {
            return Some(format!(
                "{path} links `{link}`, which is outside `docs/`, so the site cannot serve \
                 it. Name a target outside `docs/` by its path in one code span"
            ));
        }
        if !target.is_file() {
            return Some(format!("{path} targets `{link}`, which is not a file"));
        }
        (link.clone(), target)
    } else {
        let span = &spans[0];
        let Ok(target) = root.join(span).canonicalize() else {
            return Some(format!("{path} names `{span}`, which is no file"));
        };
        if !target.starts_with(&home) {
            return Some(format!(
                "{path} names `{span}`, which is outside the repository"
            ));
        }
        if !target.is_file() {
            return Some(format!("{path} names `{span}`, which is not a file"));
        }
        if target.starts_with(&served) {
            return Some(format!(
                "{path} names `{span}` in a code span, and a target under `docs/` is a link"
            ));
        }
        (span.clone(), target)
    };
    let expectation = scalar(&document, "expectation");
    let names_documents = expectation.as_deref() == Some(Expectation::Opened.name())
        || expectation.as_deref() == Some(Expectation::Cited.name());
    if !names_documents {
        return None;
    }
    let examines = examines_of(&document);
    let relative = target
        .strip_prefix(&home)
        .map(|it| it.to_string_lossy().into_owned())
        .unwrap_or_default();
    let id = std::fs::read_to_string(&target)
        .ok()
        .and_then(|text| headwater_doc::parse(&text).ok())
        .and_then(|it| scalar(&it, "id"));
    let matched = examines
        .iter()
        .any(|entry| *entry == relative || Some(entry) == id.as_ref());
    if matched {
        None
    } else {
        Some(format!(
            "{path} targets `{named}` (id {id:?}), which is none of its `examines` entries \
             {examines:?}"
        ))
    }
}

/// Each clause of the shape refuses the probe that misses it, and a probe
/// that meets every clause passes. The shelf case above cannot show this,
/// because a conforming shelf passes it whether or not a clause is checked.
#[test]
fn the_shape_refuses_each_way_a_probe_can_miss_it() {
    const RULING: &str = "../decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md";
    const OTHER: &str = "../decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md";
    const SKILL: &str = ".claude/skills/headwater-authoring/SKILL.md";
    let probe_examining = |examined: &str, expectation: &str, headings: &[(&str, &str)]| {
        let mut source = format!(
            "---\nid: HW-PROBE-synthetic\nexpectation: {expectation}\nrelations:\n  \
             examines:\n    - {examined}\n---\n\n# A synthetic probe\n"
        );
        for (heading, body) in headings {
            source.push_str(&format!("\n## {heading}\n\n{body}\n"));
        }
        source
    };
    let probe = |expectation: &str, headings: &[(&str, &str)]| {
        probe_examining("HW-DR-0049", expectation, headings)
    };
    let path = format!("{SHELF}/synthetic.md");
    let one =
        format!("The ruling is [the ruling]({RULING}), at the end of one `superseded_by` edge.");
    let good = |expectation: &str, target: &str| {
        probe(
            expectation,
            &[
                ("Task", "Say it."),
                ("Target", target),
                ("Key", "`opened`."),
            ],
        )
    };
    let anchored = |expectation: &str, target: &str| {
        probe_examining(
            SKILL,
            expectation,
            &[
                ("Task", "Say it."),
                ("Target", target),
                ("Key", "`opened`."),
            ],
        )
    };
    let skill = format!("The session must open the skill at `{SKILL}`.");

    // A file that exists and is outside the repository, so that the check
    // that the target is under `docs/` is the one that refuses it. One case
    // of this process writes it, so the pid alone keeps it apart.
    let outside =
        std::env::temp_dir().join(format!("headwater-probe-shape-{}.md", std::process::id()));
    std::fs::write(&outside, "---\nid: HW-DR-0049\n---\n").expect("a temp file");
    let outside_link = format!("The [ruling]({}).", outside.display());

    for expectation in ["opened", "cited", "answered", "patched", "not_opened"] {
        assert_eq!(
            shape_finding(&path, &good(expectation, &one)),
            None,
            "a sound `{expectation}` probe was refused"
        );
    }
    assert_eq!(
        shape_finding(&path, &anchored("opened", &skill)),
        None,
        "a sound probe that names an anchor outside `docs/` was refused"
    );

    let refused = [
        (
            "the old heading",
            probe(
                "opened",
                &[("Task", "Say it."), ("Target", &one), ("Expectation", "x")],
            ),
            "has the headings",
        ),
        (
            "no target",
            probe("opened", &[("Task", "Say it."), ("Key", "x")]),
            "has the headings",
        ),
        (
            "the headings out of order",
            probe(
                "opened",
                &[("Task", "Say it."), ("Key", "x"), ("Target", &one)],
            ),
            "has the headings",
        ),
        (
            "two links",
            good("answered", &format!("{one} And [another]({OTHER}).")),
            "holds 2 links",
        ),
        (
            "no link and no file named",
            good("answered", "The ruling, named in words and `code`."),
            "names 0 files",
        ),
        (
            "no link and two files named",
            good("answered", &format!("`{SKILL}` or `CLAUDE.md`.")),
            "names 2 files",
        ),
        (
            "a link to no file",
            good("answered", "The [ruling](../decisions/no-such-record.md)."),
            "which is no file",
        ),
        (
            "a link to a directory under docs/",
            good("answered", "The [shelf](../decisions/)."),
            "which is not a file",
        ),
        (
            "a link to a file of this repository outside docs/",
            good("answered", "The [instructions](../../CLAUDE.md)."),
            "outside `docs/`",
        ),
        (
            "a link to a file outside the repository",
            good("answered", &outside_link),
            "outside `docs/`",
        ),
        (
            "a document under docs/ named in a code span",
            good(
                "answered",
                "The ruling at `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`.",
            ),
            "a target under `docs/` is a link",
        ),
        (
            "a link and a path in a code span",
            good("answered", &format!("{one} The skill is at `{SKILL}`.")),
            "names 2 files",
        ),
        (
            "two paths in code spans, one of them no file",
            good("answered", &format!("`{SKILL}` or `no/such/file.md`.")),
            "names 2 files",
        ),
        (
            "a path in a code span that names no file",
            good("answered", "The session opens `no/such/file.md`."),
            "which is no file",
        ),
        (
            "an absolute path in a code span outside the repository",
            good("answered", &format!("The file `{}`.", outside.display())),
            "outside the repository",
        ),
        (
            "a relative path in a code span that leaves the repository",
            good(
                "answered",
                "The file `../../../../../../../../../../../../etc/hosts`.",
            ),
            "outside the repository",
        ),
        (
            "a directory in a code span",
            good("answered", "The shelf at `.claude/skills/`."),
            "which is not a file",
        ),
        (
            "a document under docs/ reached through a detour in a code span",
            good(
                "answered",
                "The ruling at `.claude/../docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`.",
            ),
            "a target under `docs/` is a link",
        ),
        (
            "a reference-style link out of docs/",
            good(
                "answered",
                &format!("The [skill][skill].\n\n[skill]: ../../{SKILL}"),
            ),
            "outside `docs/`",
        ),
        (
            "a reference-style link beside a path in a code span",
            good(
                "answered",
                &format!("The [skill][skill] at `{SKILL}`.\n\n[skill]: ../../{SKILL}"),
            ),
            "names 2 files",
        ),
        (
            "an opened target the key does not read",
            good("opened", &format!("The [record]({OTHER}).")),
            "none of its `examines` entries",
        ),
        (
            "a cited target the key does not read",
            good("cited", &format!("The [record]({OTHER}).")),
            "none of its `examines` entries",
        ),
        (
            "an opened anchor the key does not read",
            anchored("opened", "The session must open `CLAUDE.md`."),
            "none of its `examines` entries",
        ),
    ];
    let findings: Vec<_> = refused
        .iter()
        .map(|(case, source, wanted)| (case, wanted, shape_finding(&path, source)))
        .collect();
    let _ = std::fs::remove_file(&outside);
    for (case, wanted, finding) in findings {
        assert!(
            finding.as_deref().is_some_and(|it| it.contains(*wanted)),
            "{case}: wanted a finding naming `{wanted}`, got {finding:?}"
        );
    }
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
    /// Words the task must hold, because the row's separation rests on them.
    /// For the vocabulary shape these are the content words of the task, and
    /// each is also in no identifier, file name or title of an examined
    /// document.
    words: &'static [&'static str],
    /// Strings, in lower case, that the task must not hold. A row whose
    /// separation rests on what the task does not name states it here.
    absent: &'static [&'static str],
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
        absent: &[],
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
        absent: &[],
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
        absent: &[],
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
        absent: &[],
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
        absent: &[],
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
    // The harder variants of #1718. Each stands beside a probe that sat at 30
    // of 30 in the present and the absent arm of the campaign of 2026-10-03,
    // and each shallow session is the strategy that reached that ceiling.
    Harder {
        id: "HW-PROBE-a-session-follows-two-citations-from-a-record-the-task-does-not-name",
        shape: Shape::MultiDocument,
        examines: &[],
        words: &[],
        absent: &["hw-dr-0", "hw-dr-1"],
        statuses: &[
            (
                "docs/decisions/0078-a-recorded-terminal-demonstration-may-show-a-frozen-number-behind-a-recorded-on-date-marker.md",
                "current",
            ),
            ("docs/decisions/0061-q61-how-a-recorded-terminal-demonstration-is-held-against-a-run.md", "superseded"),
            ("docs/decisions/0039-q39-how-a-figure-reaches-a-hand-built-page.md", "superseded"),
            (
                "docs/decisions/0097-a-figure-on-a-hand-built-page-is-measured-when-the-site-is-published-and-the-committed-page-carries-none.md",
                "current",
            ),
        ],
        // The strategy of the original row's absent arm: follow the one
        // citation of the record reached first, and answer with the record
        // in force in place of the one it reached. From HW-DR-0078 that
        // citation is HW-DR-0061, whose successor is HW-DR-0078 itself.
        shallow: Session::Answered("HW-DR-0078"),
        sound: Session::Answered("HW-DR-0097"),
    },
    Harder {
        id: "HW-PROBE-a-session-says-which-date-a-corrected-status-carries",
        shape: Shape::Distractors,
        examines: &[],
        words: &[],
        absent: &["hw-dr-0", "hw-dr-1"],
        statuses: &[(
            "docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md",
            "current",
        )],
        // Nearly every document of the shelf carries a `status_since` equal to
        // the day it merged, so a session that reads the examples and not the
        // ruling answers `merge`.
        shallow: Session::Answered("merge 66"),
        sound: Session::Answered("correction 66"),
    },
    Harder {
        id: "HW-PROBE-a-session-answers-from-the-register-when-the-task-names-the-list-it-replaced",
        shape: Shape::SupersessionTrap,
        examines: &[("HW-REG-open-questions", "docs/spec/09-open-questions.md")],
        words: &["open questions, closed and redirected"],
        absent: &[],
        statuses: &[("docs/spec/09-open-questions.md", "superseded")],
        // The task uses the title of the superseded list, so a name search
        // opens that list first.
        shallow: Session::Read(&[
            "docs/spec/09-open-questions.md",
            "docs/decisions/0020-where-scent-lives.md",
        ]),
        sound: Session::Read(&["docs/decisions/0020-where-scent-lives.md"]),
    },
    Harder {
        id: "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded",
        shape: Shape::SupersessionTrap,
        examines: &[(
            "HW-DR-0087",
            "docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md",
        )],
        words: &["09-open-questions.md#69--a-paragraph-limit"],
        absent: &[],
        statuses: &[
            ("docs/spec/09-open-questions.md", "superseded"),
            (
                "docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md",
                "superseded",
            ),
            (
                "docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md",
                "current",
            ),
        ],
        // The strategy of the original's absent arm: follow the one link of
        // the tombstone's section and answer from the record it reaches.
        shallow: Session::Read(&[
            "docs/spec/09-open-questions.md",
            "docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md",
        ]),
        sound: Session::Read(&[
            "docs/spec/09-open-questions.md",
            "docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md",
            "docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md",
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
        action: scalar(&document, "action"),
        moments: Vec::new(),
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
/// holds at least one probe of each of the five shapes. The harder variants of
/// #1718 add more than one to some shapes, because each one stands beside a
/// probe of the campaign of 2026-10-03 that sat at 30 of 30 in both arms. Every
/// probe is a document of the shelf rather than a fixture of this crate.
#[test]
fn every_harder_probe_fails_the_shallow_session_and_passes_the_sound_one() {
    let root = repository_root();
    for shape in Shape::ALL {
        let count = HARDER.iter().filter(|row| row.shape == shape).count();
        assert!(
            count >= 1,
            "{shape:?} has no probe in the table, and it needs at least one"
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

        // A harder probe goes into its pull request settled (HW-DR-0052), and
        // a draft one is a probe nobody has agreed to run (#1709).
        let (_, own) = front_matter(&root.join(&selected.path));
        assert_eq!(
            scalar(&own, "status").as_deref(),
            Some("current"),
            "{}: a harder probe holds `status: current`",
            row.id
        );

        // The leak check prints a probe with no `leaks:` entry as
        // `undeclared` and passes it, so a harder probe whose answer key a
        // session could read in the present arm would go unseen (#1709).
        let declaration = std::fs::read_to_string(root.join(".headwater/probe.yml"))
            .expect(".headwater/probe.yml reads");
        assert!(
            declaration.contains(&format!("\n  {}: [", row.id)),
            "{}: `.headwater/probe.yml` declares no `leaks:` entry for it",
            row.id
        );

        for (path, status) in row.statuses {
            let (_, document) = front_matter(&root.join(path));
            assert_eq!(
                scalar(&document, "status").as_deref(),
                Some(*status),
                "{} rests on {path} holding `status: {status}`",
                row.id
            );
        }

        // A harder task differs from its original in what it names, so the
        // words it must hold and the strings it must not hold are part of the
        // row (#1718).
        let (source, _) = front_matter(&root.join(&selected.path));
        let task = task_of(&source).to_lowercase();
        for word in row.words {
            assert!(
                task.contains(word),
                "{}: `{word}` is not a word of the task",
                row.id
            );
        }
        for string in row.absent {
            assert!(
                !task.contains(string),
                "{}: the task holds `{string}`, and the row rests on its absence",
                row.id
            );
        }

        if row.shape == Shape::VocabularyMismatch {
            assert!(!row.words.is_empty(), "{}: no words to check", row.id);
            for word in row.words {
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

/// The campaign spec plans no superseded probe.
///
/// `headwater probe plan` selects every probe of a category, whatever its
/// status, so a line of `tools/probe/layer-campaign.spec` that names a
/// category plans a superseded probe unless it leaves it out by identifier.
/// A superseded probe stays on the shelf as the record of the campaign that
/// ran it (#1709), so the spec, and not the shelf, must keep it out. The dry
/// run counts the probes of a line and names none, so it cannot tell the
/// superseded probe from its successor.
#[test]
fn the_campaign_spec_plans_no_superseded_probe() {
    let root = repository_root();
    let spec_path = root.join("tools/probe/layer-campaign.spec");
    let spec = std::fs::read_to_string(&spec_path)
        .unwrap_or_else(|e| panic!("{}: {e}", spec_path.display()));

    // Each superseded probe on the shelf, with its category.
    let mut superseded = Vec::new();
    for probe in probes() {
        let (_, document) = front_matter(&root.join(&probe.path));
        if scalar(&document, "status").as_deref() == Some("superseded") {
            let id = probe.id.clone().expect("a probe declares an identifier");
            let category = scalar(&document, "probe_category")
                .unwrap_or_else(|| panic!("{id}: no probe_category"));
            superseded.push((id, category));
        }
    }
    assert!(
        !superseded.is_empty(),
        "the shelf holds no superseded probe, so this case reads nothing; the governs \
         probe of 2026-10-03 is one (#1709)"
    );

    let mut lines = 0;
    for (number, line) in spec.lines().enumerate() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.is_empty() || fields[0].starts_with('#') {
            continue;
        }
        assert!(
            fields.len() >= 3,
            "line {}: a spec line names a tier, an arm and a category",
            number + 1
        );
        lines += 1;
        let (category, left_out) = (fields[2], &fields[3..]);
        for (id, of) in &superseded {
            assert!(
                of != category || left_out.contains(&id.as_str()),
                "line {} of tools/probe/layer-campaign.spec plans the superseded probe {id}: \
                 it names the category `{category}` and does not leave the probe out",
                number + 1
            );
        }
    }
    assert!(lines > 0, "the campaign spec holds no line");
}

/// The probes on the shelf that no campaign may run until the owner approves
/// the spend (#1718). A probe leaves this list only by that approval, in the
/// same change that lets the spec plan it.
const AWAITING_SPEND: &[&str] = &[
    "HW-PROBE-a-session-follows-two-citations-from-a-record-the-task-does-not-name",
    "HW-PROBE-a-session-says-which-date-a-corrected-status-carries",
    "HW-PROBE-a-session-answers-from-the-register-when-the-task-names-the-list-it-replaced",
    "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded",
    // The three probes of the additive tuning set (#1833). The owner
    // approved their spend for that campaign and not for this one.
    "HW-PROBE-a-session-that-changes-the-recorder-reaches-the-recorder-contract",
    "HW-PROBE-a-session-that-moves-a-shelf-reaches-the-relocation-guide",
    "HW-PROBE-a-session-that-changes-a-verb-s-default-reaches-the-verb-s-interface-contract",
    // The five probes of the code set (#1835), which run in the additive
    // campaign's code round and in no round of this campaign.
    "HW-PROBE-a-rule-that-reads-the-date-takes-it-from-the-run-and-ships-with-the-fixture-that-fails-it",
    "HW-PROBE-a-figure-a-session-adds-to-a-hand-built-page-is-measured-and-never-typed",
    "HW-PROBE-a-session-says-where-a-check-gets-the-version-that-stood-before-a-change",
    "HW-PROBE-a-session-says-what-a-refusal-writes-on-standard-output-under-json",
    "HW-PROBE-a-session-says-what-a-verb-that-mints-an-identifier-writes-beside-the-document",
];

/// The campaign spec plans no probe that awaits the owner's approval of spend.
///
/// `headwater probe plan` selects every probe of a category, so a probe added
/// to the shelf is planned by every line of its category that does not leave
/// it out by identifier. The dry run would then price a run nobody approved,
/// and a campaign would spend on it (#1718).
#[test]
fn the_campaign_spec_plans_no_probe_that_awaits_spend() {
    let root = repository_root();
    let spec_path = root.join("tools/probe/layer-campaign.spec");
    let spec = std::fs::read_to_string(&spec_path)
        .unwrap_or_else(|e| panic!("{}: {e}", spec_path.display()));

    let shelf = probes();
    let mut awaiting = Vec::new();
    for id in AWAITING_SPEND {
        let probe = shelf
            .iter()
            .find(|probe| probe.id.as_deref() == Some(*id))
            .unwrap_or_else(|| panic!("{id} is not on {SHELF}, so this list names nothing"));
        let (_, document) = front_matter(&root.join(&probe.path));
        let category = scalar(&document, "probe_category")
            .unwrap_or_else(|| panic!("{id}: no probe_category"));
        awaiting.push((*id, category));
    }

    let mut lines = 0;
    for (number, line) in spec.lines().enumerate() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.is_empty() || fields[0].starts_with('#') {
            continue;
        }
        lines += 1;
        let (category, left_out) = (fields[2], &fields[3..]);
        for (id, of) in &awaiting {
            assert!(
                of != category || left_out.contains(id),
                "line {} of tools/probe/layer-campaign.spec plans {id}, which awaits the \
                 owner's approval of spend: it names the category `{category}` and does not \
                 leave the probe out",
                number + 1
            );
        }
    }
    assert!(lines > 0, "the campaign spec holds no line");
}

/// The committed `.headwater/probe.yml` declares the seven scenarios of
/// HW-DR-0112 (#1872), and no scenario holds this repository's own tooling.
///
/// The engine already refuses a scenario that differs from its base by more
/// than one layer outside the declared exception. This case holds the
/// declaration to the decision: a re-cut can read as well-formed while
/// scenario 0 and scenario 6 still carry this repository's own instructions,
/// skills and hooks, because a path in no layer is plain code and every
/// scenario keeps it. So it reads the delta of each scenario, not only the
/// layers it names.
#[test]
fn the_committed_scenarios_are_the_seven_of_hw_dr_0112_and_none_holds_the_tooling() {
    let root = repository_root();
    let path = root.join(headwater_probe::budget::PATH);
    let source =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let budgets = headwater_probe::Budgets::read(&source).unwrap_or_else(|e| panic!("{e}"));

    assert_eq!(
        budgets
            .layers
            .iter()
            .map(|layer| layer.name.as_str())
            .collect::<Vec<_>>(),
        ["docs", "engine", "skills", "hooks", "mcp"],
        "the five layers of HW-DR-0112"
    );
    let full = ["docs", "engine", "skills", "hooks", "mcp"];
    let table: [(u32, &[&str], Option<u32>, bool); 7] = [
        (0, &[], None, false),
        (1, &["docs"], Some(0), false),
        (2, &["docs", "engine"], Some(1), false),
        (3, &["docs", "engine", "skills"], Some(2), false),
        (4, &["docs", "engine", "hooks"], Some(2), false),
        (5, &["docs", "engine", "mcp"], Some(2), false),
        (6, &full, Some(2), true),
    ];
    assert_eq!(
        budgets.scenarios.len(),
        table.len(),
        "{:?}",
        budgets.scenarios
    );
    for (number, layers, base, exception) in table {
        let scenario = budgets
            .scenario(number)
            .unwrap_or_else(|| panic!("no scenario {number}"));
        let mut held = scenario.layers.clone();
        held.sort();
        let mut expected: Vec<String> = layers.iter().map(|l| l.to_string()).collect();
        expected.sort();
        assert_eq!(held, expected, "the layers of scenario {number}");
        assert_eq!(scenario.base, base, "the base of scenario {number}");
        assert_eq!(
            scenario.exception, exception,
            "the exception of scenario {number}"
        );
    }

    use headwater_probe::budget::Hold;
    for number in 0..=6 {
        let delta = budgets
            .scenario_delta(number)
            .unwrap_or_else(|| panic!("no delta for scenario {number}"));
        for tooling in [
            "CLAUDE.md",
            "AGENTS.md",
            ".claude/skills/hw-run-policy",
            ".claude/skills/headwater-engine",
            ".claude/commands",
            ".githooks",
        ] {
            let entries: Vec<_> = delta.iter().filter(|(_, _, p)| *p == tooling).collect();
            assert_eq!(
                entries,
                [&(Hold::Removes, "tooling", tooling)],
                "scenario {number} must remove {tooling}"
            );
        }
        let settings: Vec<_> = delta
            .iter()
            .filter(|(_, _, p)| *p == ".claude/settings.json")
            .collect();
        let hold = if [4, 6].contains(&number) {
            Hold::Writes
        } else {
            Hold::Removes
        };
        assert_eq!(
            settings,
            [&(hold, "hooks", ".claude/settings.json")],
            "the settings file of scenario {number}"
        );
    }
}

/// The settings file the hooks layer writes is the one `headwater init
/// --harness` writes into a fresh repository (#1872): six entries over four
/// events, each running a script the hooks layer holds.
///
/// The probe crate does not link the CLI, so this reads the arms file and the
/// engine's source of the entries, `engine/crates/cli/src/harness.rs`, and
/// checks that each script and event the source names is wired in the file.
/// `engine/crates/cli/tests/init_hooks.rs` holds the byte equality.
#[test]
fn the_hooks_arms_settings_wire_every_script_the_hooks_layer_holds() {
    let root = repository_root();
    let path = root.join("tools/probe/arms/hooks/.claude/settings.json");
    let settings =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    for event in ["UserPromptSubmit", "PreToolUse", "PostToolUse", "Stop"] {
        assert!(
            settings.contains(&format!("\"{event}\"")),
            "the arms settings file wires no {event} hook"
        );
    }
    let commands = settings.matches("\"type\": \"command\"").count();
    assert_eq!(commands, 6, "six hook entries over four events");
    let source =
        std::fs::read_to_string(root.join(headwater_probe::budget::PATH)).expect("the declaration");
    let budgets = headwater_probe::Budgets::read(&source).unwrap_or_else(|e| panic!("{e}"));
    let hooks = budgets
        .layers
        .iter()
        .find(|layer| layer.name == "hooks")
        .expect("the hooks layer");
    assert_eq!(hooks.arms, [".claude/settings.json"]);
    for script in ["intent", "read", "write", "review", "touch"] {
        let wired = format!(".claude/hooks/{script}.sh");
        assert!(
            settings.contains(&wired),
            "the arms settings file does not run {wired}"
        );
        assert!(
            hooks.paths.contains(&wired),
            "the hooks layer does not hold {wired}"
        );
    }
}

/// The committed additive spec plans each declared scenario in each of the
/// three rounds, and runs no held-out probe before the final round (#1774,
/// clause 4).
///
/// A line is `additive scenario-<n> <round>` and the probes of the round's set
/// it leaves out. The screening round and each tuning try run the `tuning`
/// set, and the final round runs `held-out`, as HW-DR-0111 fixes. So the lines
/// of one scenario and one round must run each probe of that set exactly once:
/// a split line that dropped a probe, or ran one twice, reads as a whole spec.
/// The engine refuses a probe in both sets, so no held-out probe can be a
/// member of the set a screening or tuning line runs.
#[test]
fn the_additive_spec_plans_each_scenario_in_each_round() {
    let root = repository_root();
    let spec_path = root.join("tools/probe/additive-campaign.spec");
    let spec = std::fs::read_to_string(&spec_path)
        .unwrap_or_else(|e| panic!("{}: {e}", spec_path.display()));
    let path = root.join(headwater_probe::budget::PATH);
    let source =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let budgets = headwater_probe::Budgets::read(&source).unwrap_or_else(|e| panic!("{e}"));
    let set_of = |round: &str| match round {
        "screening" | "tuning" => "tuning",
        "final" => "held-out",
        other => panic!("tools/probe/additive-campaign.spec names the round `{other}`"),
    };
    let held_out = budgets
        .set("held-out")
        .expect("the held-out set is declared");

    // (scenario, round) -> each probe run, with the number of lines that run it.
    let mut runs: std::collections::BTreeMap<(u32, String), Vec<String>> =
        std::collections::BTreeMap::new();
    for (number, line) in spec.lines().enumerate() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.is_empty() || fields[0].starts_with('#') {
            continue;
        }
        let at = number + 1;
        assert!(
            fields.len() >= 3,
            "line {at}: an additive line names a tier, a scenario arm and a round"
        );
        assert_eq!(fields[0], "additive", "line {at}: the tier");
        let arm = headwater_probe::Arm::read(fields[1])
            .unwrap_or_else(|| panic!("line {at}: `{}` is no arm", fields[1]));
        let headwater_probe::Arm::Scenario(scenario) = arm else {
            panic!("line {at}: `{}` is not a scenario arm", fields[1]);
        };
        let round = fields[2];
        let set = budgets
            .set(set_of(round))
            .unwrap_or_else(|| panic!("line {at}: no set `{}`", set_of(round)));
        let left_out = &fields[3..];
        for id in left_out {
            assert!(
                set.iter().any(|member| member == id),
                "line {at} leaves out {id}, which the set `{}` does not hold",
                set_of(round)
            );
        }
        let entry = runs.entry((scenario, round.to_string())).or_default();
        for member in set
            .iter()
            .filter(|member| !left_out.contains(&member.as_str()))
        {
            if round != "final" {
                assert!(
                    !held_out.contains(member),
                    "line {at} runs the held-out probe {member} in the {round} round"
                );
            }
            entry.push(member.clone());
        }
    }

    let declared: Vec<u32> = budgets.scenarios.iter().map(|s| s.number).collect();
    for &scenario in &declared {
        for round in ["screening", "tuning", "final"] {
            let mut ran = runs
                .remove(&(scenario, round.to_string()))
                .unwrap_or_else(|| {
                    panic!("no line plans scenario-{scenario} in the {round} round")
                });
            ran.sort();
            let mut expected = budgets.set(set_of(round)).expect("declared").to_vec();
            expected.sort();
            assert_eq!(
                ran, expected,
                "the lines of scenario-{scenario} in the {round} round must run each probe of the set `{}` once",
                set_of(round)
            );
        }
    }
    assert!(
        runs.is_empty(),
        "lines outside the {} declared scenarios and 3 rounds: {runs:?}",
        declared.len()
    );
}

/// The probes the owner ruled into the tuning set on 2026-10-22 (#1833): the
/// two kept probes, and the three of proposal 1 that replace the two dropped.
const TUNING_SET: &[&str] = &[
    "HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged",
    "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded",
    "HW-PROBE-a-session-that-changes-the-recorder-reaches-the-recorder-contract",
    "HW-PROBE-a-session-that-moves-a-shelf-reaches-the-relocation-guide",
    "HW-PROBE-a-session-that-changes-a-verb-s-default-reaches-the-verb-s-interface-contract",
];

/// The tuning set is the five probes the owner ruled on 2026-10-22, and the
/// two it dropped are in no set (#1833, clauses 1 and 2).
///
/// The two dropped probes stay on the shelf as records: the two-citations
/// probe sat at the ceiling in every scenario that holds `docs/`, and the
/// cold-agent probe asked for a rule that already exists. Each new probe keys
/// on `opened`, so it declares no oracle.
#[test]
fn the_tuning_set_is_the_five_probes_the_owner_ruled_on_2026_10_22() {
    let root = repository_root();
    let path = root.join(headwater_probe::budget::PATH);
    let source =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let budgets = headwater_probe::Budgets::read(&source).unwrap_or_else(|e| panic!("{e}"));

    let mut tuning = budgets
        .set("tuning")
        .expect("the tuning set is declared")
        .to_vec();
    tuning.sort();
    let mut expected: Vec<String> = TUNING_SET.iter().map(|id| id.to_string()).collect();
    expected.sort();
    assert_eq!(tuning, expected, "the tuning set of .headwater/probe.yml");

    let dropped = [
        "HW-PROBE-a-session-follows-two-citations-from-a-record-the-task-does-not-name",
        "HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor",
    ];
    let shelf = probes();
    for id in dropped {
        assert!(
            shelf.iter().any(|probe| probe.id.as_deref() == Some(id)),
            "{id} left the shelf; it stays as a record"
        );
        // `code` is the set #1835 adds, and a set the file does not declare
        // holds nothing.
        for set in ["tuning", "held-out", "code"] {
            let members = budgets.set(set).unwrap_or_default();
            assert!(
                !members.iter().any(|member| member == id),
                "{id} is still a member of the set `{set}`"
            );
        }
    }

    // Each new probe examines its target, and the target stands at `current`:
    // the relocation guide was `draft` with an accepted warrant, and the
    // owner ruled that it is promoted before any base is measured (clause 4).
    let targets = [
        (
            "HW-SPEC-the-recorder-contract",
            "docs/spec/15-the-recorder-contract.md",
        ),
        ("HW-HOW-relocate-a-shelf", "docs/how-to/relocate-a-shelf.md"),
        (
            "HW-IFACE-headwater-route",
            "docs/interfaces/headwater-route.md",
        ),
    ];
    for (id, (target, target_path)) in TUNING_SET[2..].iter().zip(targets) {
        let probe = shelf
            .iter()
            .find(|probe| probe.id.as_deref() == Some(*id))
            .unwrap_or_else(|| panic!("{id} is not on {SHELF}"));
        let (_, document) = front_matter(&root.join(&probe.path));
        assert_eq!(
            examines_of(&document),
            vec![target.to_string()],
            "{id}: examines"
        );
        let (_, examined) = front_matter(&root.join(target_path));
        assert_eq!(
            scalar(&examined, "id").as_deref(),
            Some(target),
            "{target_path}: id"
        );
        assert_eq!(
            scalar(&examined, "status").as_deref(),
            Some("current"),
            "{target}, which {id} examines, stands at `current`"
        );
        assert_eq!(
            scalar(&document, "expectation").as_deref(),
            Some("opened"),
            "{id}: expectation"
        );
        assert_eq!(
            scalar(&document, "oracle").as_deref(),
            Some("none"),
            "{id}: oracle"
        );
    }
}

/// The lead-in of the paragraph under `## Key` that names where plain code
/// carries what a code probe needs (#1835).
const PLAIN_CODE: &str = "**Where plain code carries it.**";

/// The path a code span names, less a `:<lines>` suffix.
fn path_of(span: &str) -> &str {
    span.split_once(':').map_or(span, |(path, _)| path)
}

/// Whether a path of the repository is in plain code: outside `docs/` and
/// outside every path of every layer under `layers:`, so scenario 0 holds it.
fn in_plain_code(budgets: &headwater_probe::Budgets, path: &str) -> bool {
    let under = |prefix: &str| path == prefix || path.starts_with(&format!("{prefix}/"));
    !under("docs")
        && !budgets
            .layers
            .iter()
            .flat_map(|layer| layer.paths.iter().chain(layer.arms.iter()))
            .chain(budgets.tooling.iter())
            .any(|held| under(held.trim_end_matches('/')))
}

/// The code set is five probes whose answer plain code carries (#1835).
///
/// A code probe compares scenario 1 with scenario 0, so a probe that cannot
/// pass in scenario 0 by construction has its gain built in, and a `patched`
/// probe that any write satisfies measures nothing. So, for each member of
/// `sets.code`:
///
/// - it is in neither fixed set, which HW-DR-0111 keeps as #1833 left them;
/// - a `patched` member grades against an oracle `.headwater/probe.yml`
///   declares under `oracles:`, and each file that oracle `holds` exists and
///   names the probe in its first lines, so the seal removes it from every
///   workspace and no session reads the oracle;
/// - its `## Key` has a paragraph that opens with the lead-in above and names
///   at least one existing file outside `docs/` and outside every layer, so
///   scenario 0 holds what the probe needs;
/// - an `answered` member expects a proper part of its closed set.
///
/// No expected answer is written here, because the seal removes from a
/// probe's arm tree every file that names it, and this file names none of
/// the code probes.
#[test]
fn the_code_set_is_five_probes_whose_answer_plain_code_carries() {
    let root = repository_root();
    let path = root.join(headwater_probe::budget::PATH);
    let source =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let budgets = headwater_probe::Budgets::read(&source).unwrap_or_else(|e| panic!("{e}"));

    let code = budgets
        .set("code")
        .expect("the code set is declared under `sets:` (#1835)")
        .to_vec();
    assert_eq!(code.len(), 5, "the code set holds five probes: {code:?}");

    let shelf = probes();
    let mut findings = Vec::new();
    for id in &code {
        for fixed in ["tuning", "held-out"] {
            if budgets.set(fixed).unwrap_or_default().contains(id) {
                findings.push(format!("{id} is also in the fixed set `{fixed}`"));
            }
        }
        let Some(probe) = shelf.iter().find(|probe| probe.id.as_deref() == Some(id)) else {
            findings.push(format!("{id} is not on {SHELF}"));
            continue;
        };
        let (text, document) = front_matter(&root.join(&probe.path));
        let slug = id.strip_prefix("HW-PROBE-").unwrap_or(id);

        match probe.expectation.as_deref() {
            Some("patched") => {
                let oracle = scalar(&document, "oracle").unwrap_or_default();
                match budgets.oracles.iter().find(|it| it.finding() == oracle) {
                    None => findings.push(format!(
                        "{id} grades against `{oracle}`, which no entry under `oracles:` is"
                    )),
                    Some(entry) => {
                        if entry.holds.is_empty() && entry.parser.prefix() == "test" {
                            findings.push(format!(
                                "{id}: the oracle `{oracle}` runs a test and holds no file, so \
                                 the session's tree holds the test it is graded by"
                            ));
                        }
                        for held in &entry.holds {
                            let head: String = std::fs::read_to_string(root.join(held))
                                .unwrap_or_default()
                                .lines()
                                .take(10)
                                .collect::<Vec<_>>()
                                .join("\n");
                            if !head.contains(slug) {
                                findings.push(format!(
                                    "{id}: {held}, which the oracle `{oracle}` holds, does not \
                                     exist or does not name the probe in its first ten lines, \
                                     so the seal leaves it in the session's tree"
                                ));
                            }
                        }
                    }
                }
            }
            Some("answered") => {
                if probe.expected.is_empty()
                    || probe.expected.len() >= probe.answers.len()
                    || probe.expected.iter().any(|it| !probe.answers.contains(it))
                {
                    findings.push(format!("{id} expects no proper part of its closed set"));
                }
            }
            other => findings.push(format!(
                "{id} declares the expectation {other:?}, and a code probe is `patched` or \
                 `answered`"
            )),
        }

        let paragraph = section_of(&text, "Key")
            .lines()
            .find(|line| line.trim_start().starts_with(PLAIN_CODE))
            .unwrap_or_default();
        let carried = code_spans_in(paragraph)
            .iter()
            .map(|span| path_of(span))
            .filter(|path| is_path(path))
            .filter(|path| in_plain_code(&budgets, path))
            .any(|path| root.join(path).is_file());
        if !carried {
            findings.push(format!(
                "{id}: `## Key` has no paragraph opening `{PLAIN_CODE}` that names an existing \
                 path outside `docs/` and outside every layer, so nothing shows scenario 0 holds \
                 what the probe needs"
            ));
        }
    }
    assert!(findings.is_empty(), "{}", findings.join("\n"));
}

/// The actions of rows 2 to 9 and 11 of the grid in HW-DR-0112, which pass or
/// fail by a mechanical oracle. Rows 1 and 10, `ask_or_locate` and
/// `review_change`, are graded from the transcript and keep `oracle: none`
/// (HW-OBL-0123 records why the facet cannot say so per action).
const MECHANICAL_ACTIONS: [&str; 9] = [
    "change_governed_code",
    "refactor_across_files",
    "add_document",
    "edit_depended_document",
    "supersede_or_retire",
    "move_document",
    "change_taxonomy",
    "repair_check",
    "ungoverned_control",
];

/// The probes of a mechanical row on the shelf that grade against no
/// declared oracle yet. #1874.2 and #1874.3 rewrite each into the 44-probe
/// set or retire it with a reason, and delete its line here. The case below
/// fails on a line whose probe now passes, so the list can only shrink.
const AWAITING_ORACLE: &[&str] = &[
    "HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks",
    "HW-PROBE-a-session-records-which-obligation-an-evaluation-discharged",
    "HW-PROBE-a-session-that-changes-a-verb-s-default-reaches-the-verb-s-interface-contract",
    "HW-PROBE-a-session-that-changes-the-recorder-reaches-the-recorder-contract",
    "HW-PROBE-a-session-that-moves-a-shelf-reaches-the-relocation-guide",
    "HW-PROBE-the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document",
    "HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on",
];

/// The directory of the generic oracle scripts (#1874.1). A script there
/// names no probe, because many probes call it with their own arguments.
const GENERIC_ORACLES: &str = "tools/probe/oracles/";

/// What the decisive case reads of one probe.
struct Graded<'a> {
    id: &'a str,
    action: &'a str,
    expectation: &'a str,
    oracle: &'a str,
}

/// Why a probe of a mechanical row cannot grade against its oracle, one line
/// for each reason. Empty for a probe of rows 1 and 10, and for a sound one.
///
/// A sound probe of rows 2 to 9 and 11 is `patched` and names a `command:`
/// or `test:` oracle declared under `oracles:`. Each file that oracle holds
/// exists. A held file outside the generic scripts names the probe in its
/// first ten lines, so the seal removes it from every workspace. Each generic
/// script the command runs is held, and so is `lib.sh` that it sources,
/// because the recorder restores a held file over the session's copy and a
/// session can otherwise rewrite the script that grades it.
fn mechanical_findings(
    probe: &Graded,
    oracles: &[headwater_probe::Oracle],
    root: &Path,
) -> Vec<String> {
    let mut findings = Vec::new();
    if !MECHANICAL_ACTIONS.contains(&probe.action) {
        return findings;
    }
    let id = probe.id;
    if probe.expectation != "patched" {
        findings.push(format!(
            "{id} is on the mechanical row `{}` and declares the expectation `{}`, not `patched`",
            probe.action, probe.expectation
        ));
    }
    if probe.oracle == "none" {
        findings.push(format!(
            "{id} is on the mechanical row `{}` and declares `oracle: none`, so HW-DR-0112 refuses it from the set",
            probe.action
        ));
        return findings;
    }
    let Some(entry) = oracles.iter().find(|it| it.finding() == probe.oracle) else {
        findings.push(format!(
            "{id} grades against `{}`, which is no `command:` or `test:` oracle declared under `oracles:`",
            probe.oracle
        ));
        return findings;
    };
    if entry.holds.is_empty() && entry.parser.prefix() == "test" {
        findings.push(format!(
            "{id}: the oracle `{}` runs a test and holds no file, so the session's tree holds the test it is graded by",
            probe.oracle
        ));
    }
    let slug = id.strip_prefix("HW-PROBE-").unwrap_or(id);
    for held in &entry.holds {
        let path = root.join(held);
        if !path.is_file() {
            findings.push(format!(
                "{id}: {held}, which the oracle `{}` holds, does not exist",
                probe.oracle
            ));
            continue;
        }
        if held.starts_with(GENERIC_ORACLES) {
            continue;
        }
        let head: String = std::fs::read_to_string(&path)
            .unwrap_or_default()
            .lines()
            .take(10)
            .collect::<Vec<_>>()
            .join("\n");
        if !head.contains(slug) {
            findings.push(format!(
                "{id}: {held}, which the oracle `{}` holds, does not name the probe in its first ten lines, so the seal leaves it in the session's tree",
                probe.oracle
            ));
        }
    }
    let scripts: Vec<&str> = entry
        .command
        .split_whitespace()
        .filter(|word| word.starts_with(GENERIC_ORACLES))
        .collect();
    for script in &scripts {
        if !entry.holds.iter().any(|held| held == script) {
            findings.push(format!(
                "{id}: the oracle `{}` runs {script} and does not hold it, so a session can rewrite the script that grades it",
                probe.oracle
            ));
        }
    }
    let lib = format!("{GENERIC_ORACLES}lib.sh");
    if !scripts.is_empty() && !entry.holds.contains(&lib) {
        findings.push(format!(
            "{id}: the oracle `{}` runs a generic script and does not hold {lib}, which every one of them sources",
            probe.oracle
        ));
    }
    findings
}

/// Every probe of a mechanical row grades against a declared oracle (#1874,
/// HW-DR-0112 "The probe set").
///
/// The seeded probes come first, and each one is refused or accepted for the
/// reason its label states. Then the shelf: each probe of rows 2 to 9 and 11
/// has no finding unless `AWAITING_ORACLE` names it, and each probe that list
/// names is on the shelf and still has one. #1874.3 adds the count of 4 for
/// each action, 2 in `tuning` and 2 in `held-out`, to this case.
#[test]
fn every_mechanical_row_probe_grades_against_a_declared_oracle() {
    let root = repository_root();
    let oracle = |name: &str,
                  command: &str,
                  parser: headwater_probe::OracleParser,
                  holds: &[&str]| headwater_probe::Oracle {
        name: name.to_owned(),
        paths: vec!["docs/*".to_owned()],
        command: command.to_owned(),
        parser,
        holds: holds.iter().map(|it| (*it).to_owned()).collect(),
    };
    let exit = headwater_probe::OracleParser::Exit;
    let seeded_oracles = vec![
        oracle(
            "sound",
            "sh tools/probe/oracles/check-clean.sh",
            exit,
            &[
                "tools/probe/oracles/check-clean.sh",
                "tools/probe/oracles/lib.sh",
            ],
        ),
        oracle(
            "no-lib",
            "sh tools/probe/oracles/check-clean.sh",
            exit,
            &["tools/probe/oracles/check-clean.sh"],
        ),
        oracle(
            "unheld",
            "sh tools/probe/oracles/check-clean.sh",
            exit,
            &["tools/probe/oracles/lib.sh"],
        ),
        oracle(
            "absent",
            "sh tools/probe/oracles/no-such.sh",
            exit,
            &[
                "tools/probe/oracles/no-such.sh",
                "tools/probe/oracles/lib.sh",
            ],
        ),
        oracle(
            "hidden",
            "sh tools/hw-cargo test -p headwater-check --test probe_oracle_future_date",
            headwater_probe::OracleParser::CargoTest,
            &["engine/crates/check/tests/probe_oracle_future_date.rs"],
        ),
        oracle(
            "bare-test",
            "sh tools/hw-cargo test",
            headwater_probe::OracleParser::CargoTest,
            &[],
        ),
    ];
    let date = "HW-PROBE-a-rule-that-reads-the-date-takes-it-from-the-run-and-ships-with-the-fixture-that-fails-it";
    let seeded: [(&str, Graded, bool); 12] = [
        (
            "a mechanical probe with `oracle: none`",
            Graded {
                id: "HW-PROBE-seeded",
                action: "add_document",
                expectation: "patched",
                oracle: "none",
            },
            false,
        ),
        (
            "a sound command oracle",
            Graded {
                id: "HW-PROBE-seeded",
                action: "add_document",
                expectation: "patched",
                oracle: "command:sound",
            },
            true,
        ),
        (
            "row 11 with `oracle: none`",
            Graded {
                id: "HW-PROBE-seeded",
                action: "ungoverned_control",
                expectation: "patched",
                oracle: "none",
            },
            false,
        ),
        (
            "row 1 keeps `oracle: none`",
            Graded {
                id: "HW-PROBE-seeded",
                action: "ask_or_locate",
                expectation: "answered",
                oracle: "none",
            },
            true,
        ),
        (
            "row 10 keeps `oracle: none`",
            Graded {
                id: "HW-PROBE-seeded",
                action: "review_change",
                expectation: "cited",
                oracle: "none",
            },
            true,
        ),
        (
            "an expectation other than `patched`",
            Graded {
                id: "HW-PROBE-seeded",
                action: "move_document",
                expectation: "opened",
                oracle: "command:sound",
            },
            false,
        ),
        (
            "a rule finding in place of an oracle",
            Graded {
                id: "HW-PROBE-seeded",
                action: "edit_depended_document",
                expectation: "patched",
                oracle: "relation.reciprocity.missing",
            },
            false,
        ),
        (
            "a generic script whose lib.sh is not held",
            Graded {
                id: "HW-PROBE-seeded",
                action: "add_document",
                expectation: "patched",
                oracle: "command:no-lib",
            },
            false,
        ),
        (
            "a generic script that is not held",
            Graded {
                id: "HW-PROBE-seeded",
                action: "add_document",
                expectation: "patched",
                oracle: "command:unheld",
            },
            false,
        ),
        (
            "a held file that does not exist",
            Graded {
                id: "HW-PROBE-seeded",
                action: "add_document",
                expectation: "patched",
                oracle: "command:absent",
            },
            false,
        ),
        (
            "a hidden test that names another probe",
            Graded {
                id: "HW-PROBE-seeded",
                action: "change_governed_code",
                expectation: "patched",
                oracle: "test:hidden",
            },
            false,
        ),
        (
            "a test oracle that holds no file",
            Graded {
                id: date,
                action: "change_governed_code",
                expectation: "patched",
                oracle: "test:bare-test",
            },
            false,
        ),
    ];
    let mut wrong = Vec::new();
    for (label, probe, sound) in &seeded {
        let found = mechanical_findings(probe, &seeded_oracles, &root);
        if found.is_empty() != *sound {
            wrong.push(format!(
                "seeded, {label}: wanted {}, got {found:?}",
                if *sound { "no finding" } else { "a finding" }
            ));
        }
    }
    let hidden = Graded {
        id: date,
        action: "change_governed_code",
        expectation: "patched",
        oracle: "test:hidden",
    };
    let found = mechanical_findings(&hidden, &seeded_oracles, &root);
    if !found.is_empty() {
        wrong.push(format!(
            "seeded, a hidden test that names its probe: wanted no finding, got {found:?}"
        ));
    }
    let none = Graded {
        id: "HW-PROBE-seeded",
        action: "add_document",
        expectation: "patched",
        oracle: "none",
    };
    let found = mechanical_findings(&none, &seeded_oracles, &root);
    if !found.iter().any(|it| it.contains("`oracle: none`")) {
        wrong.push(format!(
            "seeded, `oracle: none` on a mechanical row is refused as itself, not as an undeclared oracle: got {found:?}"
        ));
    }

    // The awaiting list, seeded: a listed probe that now passes, a listed
    // probe gone from the shelf, and an unlisted probe that fails are each
    // reported, and a listed probe that still fails is not.
    let seeded_shelf = vec![
        ("HW-PROBE-fixed".to_owned(), Vec::new()),
        ("HW-PROBE-waiting".to_owned(), vec!["a finding".to_owned()]),
        (
            "HW-PROBE-unlisted".to_owned(),
            vec!["an unlisted finding".to_owned()],
        ),
    ];
    let reported = awaiting_findings(
        &seeded_shelf,
        &["HW-PROBE-fixed", "HW-PROBE-waiting", "HW-PROBE-gone"],
    );
    for want in ["HW-PROBE-fixed", "HW-PROBE-gone", "an unlisted finding"] {
        if !reported.iter().any(|it| it.contains(want)) {
            wrong.push(format!(
                "seeded awaiting list: nothing reports {want}: {reported:?}"
            ));
        }
    }
    if reported.len() != 3 {
        wrong.push(format!(
            "seeded awaiting list: wanted 3 reports, got {reported:?}"
        ));
    }

    let path = root.join(headwater_probe::budget::PATH);
    let source =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let budgets = headwater_probe::Budgets::read(&source).unwrap_or_else(|e| panic!("{e}"));
    let mut shelf = Vec::new();
    for probe in probes() {
        let (_, document) = front_matter(&root.join(&probe.path));
        let id = probe.id.clone().unwrap_or_else(|| probe.path.clone());
        let action = scalar(&document, "action").unwrap_or_default();
        let oracle = scalar(&document, "oracle").unwrap_or_default();
        let expectation = probe.expectation.clone().unwrap_or_default();
        let graded = Graded {
            id: &id,
            action: &action,
            expectation: &expectation,
            oracle: &oracle,
        };
        let found = mechanical_findings(&graded, &budgets.oracles, &root);
        shelf.push((id, found));
    }
    wrong.extend(awaiting_findings(&shelf, AWAITING_ORACLE));
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// What the shelf owes, given each probe's findings and the probes still
/// awaiting an oracle: every finding of a probe the list does not name, each
/// listed probe that has none now, and each listed probe not on the shelf.
fn awaiting_findings(shelf: &[(String, Vec<String>)], awaiting: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for (id, found) in shelf {
        if awaiting.contains(&id.as_str()) {
            if found.is_empty() {
                out.push(format!(
                    "{id} grades against a declared oracle now, so delete its line from AWAITING_ORACLE"
                ));
            }
        } else {
            out.extend(found.iter().cloned());
        }
    }
    for id in awaiting {
        if !shelf.iter().any(|(seen, _)| seen == id) {
            out.push(format!(
                "{id} is in AWAITING_ORACLE and not on {SHELF}, so delete its line"
            ));
        }
    }
    out
}

/// Every file of this repository's harness belongs to a layer of the
/// additive campaign or to its `tooling`, so plain code holds none of it
/// (#1774, #1872).
///
/// A layer names its paths one by one under `.claude/`, because the skills,
/// the hooks and this repository's own tooling share that directory. So a
/// skill, an agent or a hook added later is in no layer and no tooling path
/// until somebody declares it, and every scenario, plain code included,
/// would hold it. That is the hidden difference of #1712. This case walks
/// the directories that deliver a harness and fails on the first file that
/// neither a layer path nor a tooling path holds.
#[test]
fn every_harness_file_belongs_to_a_layer() {
    let root = repository_root();
    let path = root.join(headwater_probe::budget::PATH);
    let source =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let budgets = headwater_probe::Budgets::read(&source).unwrap_or_else(|e| panic!("{e}"));
    let held: Vec<&str> = budgets
        .layers
        .iter()
        .flat_map(|layer| layer.paths.iter().chain(layer.arms.iter()))
        .chain(budgets.tooling.iter())
        .map(String::as_str)
        .collect();
    let in_a_layer = |file: &str| {
        held.iter().any(|path| {
            file == *path
                || file
                    .strip_prefix(path)
                    .is_some_and(|rest| rest.starts_with('/'))
        })
    };

    // What git ignores under `.claude/`: the local state, the agents' trees,
    // and the local settings file.
    // Under `.github/`, what serves GitHub and not a session: the workflows,
    // the images, the issue template and the two community files. The
    // Copilot copies of the harness sit beside them and are walked (#1774).
    let ignored = [
        ".claude/state",
        ".claude/worktrees",
        ".claude/settings.local.json",
        ".github/workflows",
        ".github/assets",
        ".github/ISSUE_TEMPLATE",
        ".github/CONTRIBUTING.md",
        ".github/SECURITY.md",
    ];
    let mut files = Vec::new();
    let mut pending: Vec<String> = [".claude", ".agents", ".codex", ".githooks", ".github"]
        .iter()
        .map(|d| d.to_string())
        .collect();
    while let Some(relative) = pending.pop() {
        if ignored.contains(&relative.as_str()) {
            continue;
        }
        let full = root.join(&relative);
        if full.is_dir() {
            for entry in std::fs::read_dir(&full).unwrap_or_else(|e| panic!("{relative}: {e}")) {
                let name = entry.expect("an entry").file_name();
                pending.push(format!("{relative}/{}", name.to_string_lossy()));
            }
        } else if full.exists() {
            files.push(relative);
        }
    }
    files.extend(["CLAUDE.md".to_string(), "AGENTS.md".to_string()]);
    assert!(
        files.iter().any(|f| f.starts_with(".claude/skills/")),
        "the walk found no skill, so it read nothing"
    );
    let orphans: Vec<&String> = files.iter().filter(|file| !in_a_layer(file)).collect();
    assert!(
        orphans.is_empty(),
        "these files deliver a harness and neither a layer nor the tooling of {} holds \
         them, so plain code holds them: {orphans:?}",
        headwater_probe::budget::PATH
    );
}

// ------------------------------------------------- the spend of each round
//
// `.headwater/probe.yml` declares three rounds of the additive campaign, each
// with the cents the owner approved for it, and every transcript states the
// round that spent for it in its `round` facet (#1822). So the committed
// spend of a round is the sum of `cost_cents` over its transcripts, and these
// two tests hold that sum at or below the approval, and the facet to the
// tier. A halted attempt that left no transcript is in no sum, so the
// committed sum is a floor of what a round spent, which is enough for a test
// that it stays at or below its approval.

/// The shelf `kinds.probe_transcript` is placed on.
const TRANSCRIPTS: &str = "docs/probe-runs";

/// One transcript, as these two tests read it.
struct Spent {
    path: String,
    tier: Option<String>,
    round: Option<String>,
    /// `cost_cents` of the YAML block under `## Run identity`.
    cost: Option<u64>,
}

/// `cost_cents` of the first YAML block under `## Run identity`.
fn identity_cost(source: &str) -> Option<u64> {
    let start = source.find("\n## Run identity\n")?;
    let rest = &source[start..];
    let open = rest.find("\n```yaml\n")? + "\n```yaml\n".len();
    let close = rest[open..].find("\n```")?;
    rest[open..open + close]
        .lines()
        .find_map(|line| line.strip_prefix("cost_cents: "))
        .and_then(|value| value.trim().parse().ok())
}

/// Every transcript of the shelf, in path order. The index `README.md` opens
/// with no front matter fence, which is how it is told from a transcript.
fn transcripts() -> Vec<Spent> {
    let shelf = repository_root().join(TRANSCRIPTS);
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&shelf)
        .unwrap_or_else(|e| panic!("{}: {e}", shelf.display()))
        .map(|entry| entry.expect("a shelf entry").path())
        .filter(|path| path.extension().is_some_and(|it| it == "md"))
        .collect();
    paths.sort();
    paths
        .iter()
        .filter(|path| {
            std::fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
                .starts_with("---\n")
        })
        .map(|path| {
            let (source, document) = front_matter(path);
            Spent {
                path: path.display().to_string(),
                tier: scalar(&document, "tier"),
                round: scalar(&document, "round"),
                cost: identity_cost(&source),
            }
        })
        .collect()
}

/// `approved_cents` of each round under `rounds:` in `.headwater/probe.yml`.
fn approvals() -> Vec<(String, u64)> {
    let path = repository_root().join(".headwater/probe.yml");
    let source =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let loaded = headwater_yaml::load(&source).expect("the probe declaration parses");
    let rounds = loaded
        .value
        .as_map()
        .and_then(|root| root.get("rounds"))
        .and_then(|rounds| rounds.value.as_map())
        .expect("`rounds:` is a mapping");
    rounds
        .iter()
        .map(|entry| {
            let cents = entry
                .value
                .value
                .as_map()
                .and_then(|round| round.get("approved_cents"))
                .and_then(|cents| cents.value.as_scalar())
                .and_then(|cents| cents.text.parse().ok())
                .unwrap_or_else(|| panic!("round {} has no whole approved_cents", entry.key.value));
            (entry.key.value.clone(), cents)
        })
        .collect()
}

/// Each way the transcripts break a round's approval: a round above its
/// approval, a round no declaration names, a transcript with no round, and a
/// transcript of a round whose cost does not read.
fn over_approval(approved: &[(String, u64)], spent: &[Spent]) -> Vec<String> {
    let mut findings = Vec::new();
    let mut sums: Vec<(String, u64, usize)> = approved
        .iter()
        .map(|(round, _)| (round.clone(), 0, 0))
        .collect();
    for transcript in spent {
        match transcript.round.as_deref() {
            None => findings.push(format!("{} declares no round", transcript.path)),
            Some("none") => {}
            Some(round) => match sums.iter_mut().find(|(name, _, _)| name == round) {
                None => findings.push(format!(
                    "{} names the round {round}, which `rounds:` does not declare",
                    transcript.path
                )),
                Some(sum) => match transcript.cost {
                    None => findings.push(format!(
                        "{} states no cost_cents under Run identity",
                        transcript.path
                    )),
                    Some(cost) => {
                        sum.1 += cost;
                        sum.2 += 1;
                    }
                },
            },
        }
    }
    for ((round, sum, count), (_, cents)) in sums.iter().zip(approved) {
        if sum > cents {
            findings.push(format!(
                "round {round}: {sum} cents over {count} transcripts, above the {cents} approved"
            ));
        }
    }
    findings
}

/// Each transcript whose round does not pair with its tier: an additive
/// transcript names a round, and every other one writes `none`.
fn unpaired(spent: &[Spent]) -> Vec<String> {
    spent
        .iter()
        .filter(|transcript| {
            let additive = transcript.tier.as_deref() == Some("additive");
            let none = transcript
                .round
                .as_deref()
                .is_none_or(|round| round == "none");
            additive == none
        })
        .map(|transcript| {
            format!(
                "{}: tier {:?} with round {:?}",
                transcript.path, transcript.tier, transcript.round
            )
        })
        .collect()
}

#[test]
fn each_round_of_the_additive_campaign_spent_at_most_its_approval() {
    let spent = transcripts();
    let rounds = approvals();
    assert!(
        spent
            .iter()
            .any(|it| it.round.as_deref() == Some("screening")),
        "no transcript of the screening round was read, so the sum is over nothing"
    );
    assert!(
        rounds.iter().any(|(round, _)| round == "tuning"),
        "`rounds:` declares no tuning round"
    );
    let findings = over_approval(&rounds, &spent);
    assert!(findings.is_empty(), "{}", findings.join("\n"));
}

#[test]
fn a_transcript_names_a_round_exactly_when_its_tier_is_additive() {
    let findings = unpaired(&transcripts());
    assert!(findings.is_empty(), "{}", findings.join("\n"));
}

fn spent(path: &str, tier: &str, round: Option<&str>, cost: Option<u64>) -> Spent {
    Spent {
        path: path.to_string(),
        tier: Some(tier.to_string()),
        round: round.map(str::to_string),
        cost,
    }
}

/// The twin of the spend test, over a set it writes: one cent above the
/// approval is a finding, the approval exactly is not, and each way a
/// transcript escapes the sum is a finding of its own.
#[test]
fn the_spend_test_finds_a_round_one_cent_above_its_approval() {
    let approved = vec![("screening".to_string(), 100), ("tuning".to_string(), 50)];
    let at = vec![
        spent("a", "additive", Some("screening"), Some(60)),
        spent("b", "additive", Some("screening"), Some(40)),
        spent("c", "additive", Some("tuning"), Some(50)),
        spent("d", "campaign", Some("none"), Some(999_999)),
    ];
    assert_eq!(over_approval(&approved, &at), Vec::<String>::new());

    let mut above = at;
    above[1].cost = Some(41);
    assert_eq!(
        over_approval(&approved, &above),
        vec!["round screening: 101 cents over 2 transcripts, above the 100 approved".to_string()]
    );

    let escapes = vec![
        spent("e", "additive", Some("pilot"), Some(1)),
        spent("f", "additive", None, Some(1)),
        spent("g", "additive", Some("tuning"), None),
    ];
    assert_eq!(
        over_approval(&approved, &escapes),
        vec![
            "e names the round pilot, which `rounds:` does not declare".to_string(),
            "f declares no round".to_string(),
            "g states no cost_cents under Run identity".to_string(),
        ]
    );
}

#[test]
fn the_pairing_test_finds_each_direction() {
    let set = vec![
        spent("kept", "additive", Some("tuning"), None),
        spent("kept-too", "campaign", Some("none"), None),
        spent("additive-none", "additive", Some("none"), None),
        spent("additive-absent", "additive", None, None),
        spent("campaign-round", "campaign", Some("screening"), None),
    ];
    let found: Vec<String> = unpaired(&set)
        .into_iter()
        .map(|it| it.split(':').next().unwrap_or_default().to_string())
        .collect();
    assert_eq!(
        found,
        vec!["additive-none", "additive-absent", "campaign-round"]
    );
}

#[test]
fn identity_cost_reads_the_run_identity_block_and_no_other() {
    let source = "---\nid: x\n---\n\n## Events\n\n```yaml\ncost_cents: 1\n```\n\n## Run identity\n\n```yaml\nmodel: m\ncost_cents: 2452\n```\n";
    assert_eq!(identity_cost(source), Some(2452));
    assert_eq!(identity_cost("---\n---\n\n## Run identity\n\nTODO\n"), None);
}

/// No probe on the shelf quotes the text a figure marker opens with.
///
/// The site renders every probe as a page, and `tools/site/refresh-figures.sh`
/// counts each `data-figure=` on an assembled page as a marker. A quotation in
/// a code span renders escaped, so the pattern does not read it as an element,
/// the count disagrees, and the CI step that fills the figures exits 1
/// (#1835, verify round 1: the C2 probe quoted the marker twice). Only
/// `docs/probe-runs/` is excused, because the filler skips it. A probe that
/// must name the attribute names it without the `=`.
#[test]
fn no_probe_on_the_shelf_quotes_a_figure_marker() {
    const MARKER: &str = "data-figure=";
    let shelf = repository_root().join(SHELF);
    let mut quoting: Vec<String> = std::fs::read_dir(&shelf)
        .unwrap_or_else(|e| panic!("{}: {e}", shelf.display()))
        .map(|entry| entry.expect("a shelf entry").path())
        .filter(|path| path.extension().is_some_and(|it| it == "md"))
        .filter(|path| {
            std::fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
                .contains(MARKER)
        })
        .map(|path| path.display().to_string())
        .collect();
    quoting.sort();
    assert!(
        quoting.is_empty(),
        "these probes carry the text `{MARKER}`, which the published page carries too and \
         `tools/site/refresh-figures.sh --into` reads as a marker it cannot fill: {quoting:?}"
    );
}
