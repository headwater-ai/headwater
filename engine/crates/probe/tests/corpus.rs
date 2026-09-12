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
