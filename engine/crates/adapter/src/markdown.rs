// SPDX-License-Identifier: Apache-2.0
//! The run as a job summary or a review comment.
//!
//! The second vocabulary [spec 6](../../../../docs/spec/06-engine-architecture.md#ci-adapters)
//! names — "annotations, check runs, job summaries, review comments" — and the
//! second adapter that [spec 8](../../../../docs/spec/08-design-departures.md)
//! requires from the start. It exists beside SARIF rather than after it,
//! because one adapter cannot show where the adapter boundary is.
//!
//! # It reads as a report and not as a dump
//!
//! Before this module, this repository's own workflow put the whole text report
//! inside a fenced block in the job summary. That is a stand-in for an adapter:
//! it privileges nothing, and it also translates nothing. A reader of a
//! proposal wants the three counts, the escaped findings marked as escaped, and
//! a table with a path they can click. This writes that.
//!
//! # What Markdown cannot carry
//!
//! [`LOSS`], and it is a different set from SARIF's, which is the whole reason
//! to hold two. Prose for a person drops what a machine reads: the per-input
//! digests of the read set, the check editions, and the loss set itself.
//! Nothing here is machine-readable, so nothing here declares its loss inside
//! the artifact the way [`crate::sarif`] does. The declaration is in this
//! source, and `headwater check --format json` is where the dropped values are.

use crate::{reported, Escape, Loss, Reported, Subject};
use headwater_check::{Run, Scoped, Severity};

/// What a job summary cannot carry, and where each value went instead.
pub const LOSS: &[Loss] = &[
    Loss {
        field: "the read set",
        reason: "a digest for each of the inputs is the artifact a gate reads, and a summary a \
                 person reads cannot hold one line per document",
        carried_in: "the count alone, and `--read-set` or `--format json` for the rest",
    },
    Loss {
        field: "the check editions and scopes",
        reason: "a rule's edition is a component of a cache key rather than something a reviewer \
                 acts on",
        carried_in: "",
    },
    Loss {
        field: "the loss set",
        reason: "no member of this format is machine-readable, so an artifact that declared its \
                 own loss would be declaring it to a person who cannot act on it",
        carried_in: "",
    },
    Loss {
        field: "the obligation's severity",
        reason: "the same drop SARIF makes, and for the same reason: one column per finding, and \
                 the check's severity is the one a reviewer acts on",
        carried_in: "",
    },
];

/// One run as Markdown.
pub fn render(run: &Run, subject: &Subject<'_>) -> String {
    use std::fmt::Write;
    let all = reported(run);
    let live: Vec<&Reported<'_>> = all.iter().filter(|entry| entry.is_live()).collect();
    let escaped: Vec<&Reported<'_>> = all.iter().filter(|entry| !entry.is_live()).collect();
    let mut out = String::new();

    let _ = writeln!(out, "## `headwater check`\n");
    let _ = writeln!(
        out,
        "{} across {} of {} documents, against `{}` {} at `{}`, evaluated at {}.\n",
        headline(&live),
        run.coverage.checked(),
        run.coverage.seen(),
        subject.package,
        subject.version,
        subject.lock,
        subject.now
    );

    // What the run was told about, above the findings. A reviewer of a proposal
    // needs it, because the promotion count and every transition check are
    // about that change and about nothing else. A full-corpus run writes
    // nothing here, so the block stands exactly when there is a change to name.
    if let Some(scoped) = &run.change {
        out.push_str(&scoped_to(scoped));
    }

    if !live.is_empty() {
        let _ = writeln!(out, "| Severity | Where | Rule | Finding |");
        let _ = writeln!(out, "|---|---|---|---|");
        for entry in &live {
            let _ = writeln!(
                out,
                "| {} | `{}` | `{}` | {} |",
                word(entry.finding.severity),
                at(entry),
                entry.finding.rule,
                cell(&entry.finding.message)
            );
        }
        out.push('\n');
    }

    // The escaped findings, marked as escaped. A surface that showed them
    // beside the live ones would report debt as a regression, and one that
    // dropped them would report a suppression nobody can count, which spec 12
    // says is indistinguishable from a rule that never fires.
    if !escaped.is_empty() {
        let _ = writeln!(
            out,
            "<details><summary>{} not reported: {}</summary>\n",
            escaped.len(),
            partition(&escaped)
        );
        let _ = writeln!(out, "| Held by | Where | Rule | Finding |");
        let _ = writeln!(out, "|---|---|---|---|");
        for entry in &escaped {
            let _ = writeln!(
                out,
                "| {} | `{}` | `{}` | {} |",
                entry.escape.map(Escape::name).unwrap_or("none"),
                at(entry),
                entry.finding.rule,
                cell(&entry.finding.message)
            );
        }
        let _ = writeln!(out, "\n</details>\n");
    }

    // Spec 6: a run reports what it evaluated. The read set is a count here and
    // an artifact elsewhere, which is the first entry of the loss set above.
    let _ = writeln!(
        out,
        "{}. {} obligations, {} verified.",
        run.read_set.summary(),
        run.register.obligations.len(),
        run.register
            .at(headwater_check::register::Disposition::Verified)
            .count()
    );
    out
}

/// The change this run was scoped to, as a reviewer of a proposal reads it.
///
/// The same six values `--format json` writes as data, in the one form this
/// format has for anything: a sentence and a list. The unmatched paths are
/// named rather than counted, because a path that reached no row of the corpus
/// is a document nothing was checked over, and the reviewer is the person who
/// can see that it should have been.
fn scoped_to(scoped: &Scoped) -> String {
    use std::fmt::Write;
    let named = &scoped.named;
    let mut out = String::new();
    let unreadable = match named.unreadable {
        0 => String::new(),
        one => format!(
            ", and {one} whose prior version did not read, so every check that needed one was \
             skipped over it"
        ),
    };
    let _ = writeln!(
        out,
        "**Scoped to a change.** This run was told what one change carries, so the checks that \
         read the version a document stood at before it could run at all. {} documents were \
         named: {} that the change adds, {} with a prior version this run read{}. The findings \
         above are over the whole corpus, as they are in a run that names no change.\n",
        named.documents, named.added, named.carried, unreadable
    );
    let _ = writeln!(
        out,
        "{} promoted from `{}` to `{}` in this change. Nothing declares how many promotions in \
         one change is too many.\n",
        scoped.promotions,
        headwater_check::promotion::FROM,
        headwater_check::promotion::TO
    );
    if !scoped.unmatched.is_empty() {
        let _ = writeln!(
            out,
            "{} of those paths named no row of this corpus, so nothing was checked over them:\n",
            scoped.unmatched.len()
        );
        for path in &scoped.unmatched {
            let _ = writeln!(out, "- `{path}`");
        }
        out.push('\n');
    }
    out
}

/// The first sentence, which is the only line a reader is guaranteed to read.
///
/// It states the count and never a verdict. Whether these findings stop a
/// landing is the control's business and the forge's, and a summary that said
/// "failed" would be the engine ordering what lands.
fn headline(live: &[&Reported<'_>]) -> String {
    let errors = live
        .iter()
        .filter(|entry| entry.finding.severity == Severity::Error)
        .count();
    match (live.len(), errors) {
        (0, _) => "No findings".to_string(),
        (1, 0) => "1 finding".to_string(),
        (many, 0) => format!("{many} findings"),
        (many, errors) => format!("{many} findings, {errors} of them errors"),
    }
}

/// How the escaped set divides, in the precedence spec 4 fixes.
fn partition(escaped: &[&Reported<'_>]) -> String {
    let count = |escape: Escape| {
        escaped
            .iter()
            .filter(|entry| entry.escape == Some(escape))
            .count()
    };
    let (pending, suppressed) = (count(Escape::MigrationPending), count(Escape::Suppression));
    match (pending, suppressed) {
        (0, one) => format!("{one} suppressed"),
        (one, 0) => format!("{one} migration-pending"),
        (one, two) => format!("{one} migration-pending, {two} suppressed"),
    }
}

fn word(severity: Severity) -> &'static str {
    crate::severity(severity)
}

/// The location, in the form the text report writes: a finding with no line is
/// about the document rather than about a place in it.
fn at(entry: &Reported<'_>) -> String {
    match entry.finding.line {
        0 => entry.finding.path.clone(),
        line => format!("{}:{line}", entry.finding.path),
    }
}

/// One message inside a table cell.
///
/// A pipe would end the cell and a newline would end the row, so both are
/// escaped rather than dropped: a message this engine writes is the sentence a
/// reader acts on, and a truncated one sends them to the source to find out
/// what it said.
fn cell(message: &str) -> String {
    message.replace('|', "\\|").replace('\n', " ")
}
