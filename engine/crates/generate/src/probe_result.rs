// SPDX-License-Identifier: Apache-2.0
//! The probe result: the verdicts a grader returns over one committed
//! transcript, written as a document of this corpus.
//!
//! # Why the result is a projection and not something a verb prints
//!
//! [Spec 5](../../../../docs/spec/05-ai-integration.md#a-run-produces-a-snapshot-and-a-document)
//! settles it in one sentence: "The **probe result** is a document, generated
//! from the transcript, the expectations and the grader version. It carries the
//! `regenerated` warrant, and `generate --check` proves it." A rate that only a
//! verb prints is a number a reader cannot fetch, and a rate somebody pasted
//! into a document is a number nothing re-derives. This emitter is the third
//! option, and it is the one that makes the standing test possible: edit the
//! transcript and the committed result no longer matches what this corpus
//! produces.
//!
//! # The grader runs inside a gate, and that is not the thing that may not gate
//!
//! [`headwater_probe`](../../../probe/index.html) states four things that keep a
//! probe out of every gate, and the third of them used to read "no caller in any
//! gate". This emitter is now such a caller: `generate --check` runs in
//! continuous integration and it reaches the grader through this module.
//!
//! The property that matters is unchanged, and it is worth naming exactly. What
//! may never gate is **a model's behavior**. A run that answered every question
//! wrongly produces a transcript whose result reports a rate near zero, and the
//! gate exits 0 over it, because the gate compares the committed result against
//! the result this corpus derives. What fails the gate is a *derivation* that no
//! longer agrees with its source, which spec 5 names as "a defect in the grader,
//! the parser, or the committed inputs". No socket is opened here, no crate
//! below opens one, and the component that reaches the network is the recorder,
//! which is not a crate of this workspace.
//!
//! # `{run}` is the transcript's file stem
//!
//! One transcript is one run and one result, so one declaration writes as many
//! files as the corpus holds transcripts. `{shelf}` already sets the precedent
//! for a placeholder in an output path, and this is the second one. It is the
//! transcript's file stem rather than its identifier, because the same
//! substitution has to serve the output path and the declared identifier, and a
//! file called `HW-RUN-first.md` reads as a shout where `first-regression.md`
//! reads as a name. So a transcript at `docs/probe-runs/first-regression.md`
//! under `output: docs/probe-results/{run}.md` writes
//! `docs/probe-results/first-regression.md`, and an `identity` of
//! `HW-RESULT-{run}` mints `HW-RESULT-first-regression`.
//!
//! A rename of the transcript therefore renames the result and moves its
//! identifier. That is the cost of deriving a name from a path, and it is the
//! cost `{shelf}` already pays.
//!
//! # Nothing here reads a clock
//!
//! `--check` compares bytes, so an artifact whose content turns on the time of
//! day fails the gate on a morning nobody touched a file. Every value this
//! module writes comes from the transcript, from the probes, or from the grader
//! version. The transcript carries the wall-clock time of the run it recorded,
//! and that value is a committed input rather than a reading taken here.
//!
//! The grader version is the one input that moves without a corpus edit. It is
//! the version of `headwater-probe`, so a release that bumps the workspace
//! version makes every committed result stale until it is regenerated. That is
//! the behavior spec 5 asks for: a result is a function of the grader version,
//! and a series that averaged over two of them would report a change in the
//! instrument as a change in the corpus.

use crate::{Declaration, DeclaredIdentity, Identity, Kind, Output, Plan, Runs, Unwritten};
use headwater_census::census::{Census, Outcome};
use headwater_probe::grade::Results;
use headwater_probe::intake::{Record, Tree};
use headwater_query::Surface;

/// The placeholder an output path and a declared identity may carry.
const RUN: &str = "{run}";

/// The kind of document a transcript is, which is the input this emitter reads.
const TRANSCRIPT: &str = headwater_probe::intake::KIND;

pub(crate) fn emit(
    surface: &Surface<'_>,
    census: &Census,
    declaration: &Declaration,
    runs: &Runs,
    identity: &Identity,
    plan: &mut Plan,
) {
    // Every committed transcript, in census order, which is path order.
    let committed: Vec<&str> = census
        .rows
        .iter()
        .filter(|row| matches!(&row.outcome, Outcome::Typed { kind, .. } if kind == TRANSCRIPT))
        .map(|row| row.path.as_str())
        .collect();

    // The first of the three inputs a result is a function of. This is the arm
    // that is empty in this repository, and reporting it here gives the
    // emptiness a location that a run prints, rather than a register entry that
    // a reader has to go and look up.
    if committed.is_empty() {
        plan.unwritten.push(Unwritten {
            at: declaration.output.clone(),
            kind: Kind::ProbeResult,
            reason: format!(
                "this corpus holds no `{TRANSCRIPT}` document. A result is a function of a \
                 transcript, the expectations and the grader version, and a transcript is written \
                 by a recorder that observes a session from outside it. No verb of this engine \
                 writes one"
            ),
        });
        return;
    }

    // A selection is the second input, and it comes from the caller for the
    // reason the identity does. A plan that refused to compose one says why,
    // and the reason is reported against each file the declaration would have
    // written rather than against the pattern, so that the run names the file
    // whose bytes are now what an earlier corpus derived.
    if let Some(refusal) = &runs.refusal {
        for path in &committed {
            plan.unwritten.push(Unwritten {
                at: declaration.output.replace(RUN, &stem(path)),
                kind: Kind::ProbeResult,
                reason: format!(
                    "`headwater probe plan` composes the probes a transcript is graded against, \
                     and it does not compose them over this corpus: {refusal}. A plan that stops \
                     partway has read some of the probes of this corpus and none of the rest, so \
                     grading `{path}` against what it managed would report a rate over a \
                     denominator no document declares"
                ),
            });
        }
        return;
    }

    // An empty one grades nothing, and a result over no probe would report a
    // rate whose denominator nobody declared. No plan reached this, or the
    // refusal above would name it: this is the caller that composed nothing at
    // all.
    if runs.selected.is_empty() {
        plan.unwritten.push(Unwritten {
            at: declaration.output.clone(),
            kind: Kind::ProbeResult,
            reason: format!(
                "nothing composed a probe selection and nothing said why, so there is nothing to \
                 grade {} against. `headwater probe plan` composes one over the probes this \
                 corpus classifies, and it reads `{}` to do it. A grade of a run that already \
                 happened spends nothing, so no ceiling in that file is enforced here",
                count(committed.len(), "transcript"),
                headwater_probe::budget::PATH
            ),
        });
        return;
    }

    // One output path for several transcripts writes each run over the last.
    // The same refusal a shelf index makes, for the same reason.
    if committed.len() > 1 && !declaration.output.contains(RUN) {
        plan.unwritten.push(Unwritten {
            at: declaration.output.clone(),
            kind: Kind::ProbeResult,
            reason: format!(
                "covers {} and its output holds no `{RUN}`, so every run would be written over \
                 the last one",
                count(committed.len(), "transcript")
            ),
        });
        return;
    }

    let tree = Tree {
        census,
        config: surface.config(),
        lock: &identity.lock,
    };
    for path in committed {
        let stem = stem(path);
        let output = declaration.output.replace(RUN, &stem);
        let Some(transcript) = runs
            .transcripts
            .iter()
            .find(|candidate| candidate.path == path)
        else {
            plan.unwritten.push(Unwritten {
                at: output,
                kind: Kind::ProbeResult,
                reason: format!(
                    "the caller supplied no source for `{path}`, and a grade is taken over the \
                     bytes of a transcript rather than over the row that classified it"
                ),
            });
            continue;
        };

        let front = match &declaration.identity {
            None => String::new(),
            Some(declared) => {
                let minted = DeclaredIdentity {
                    id: declared.id.replace(RUN, &stem),
                    kind: declared.kind.clone(),
                };
                match crate::identity::front_matter(surface, &minted, &output, Kind::ProbeResult) {
                    Ok(block) => block,
                    Err(why) => {
                        plan.unwritten.push(Unwritten {
                            at: output,
                            kind: Kind::ProbeResult,
                            reason: why,
                        });
                        continue;
                    }
                }
            }
        };

        let record = Record::read(&transcript.source, &tree);
        let results = Results::over(&record, &runs.selected);
        plan.outputs.push(Output {
            path: output,
            kind: Kind::ProbeResult,
            bytes: body(&front, path, &record, &results, &runs.selection),
        });
    }
}

/// The whole file: the front matter, the marker where there is no front matter,
/// and the report.
fn body(
    front: &str,
    transcript: &str,
    record: &Record,
    results: &Results,
    selection: &str,
) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    match front.is_empty() {
        // A declaration that states no identity writes a file that is no node.
        // The marker still has to be there, or the next census reports the file
        // as an untyped document of the shelf it landed on.
        true => {
            out.push_str(&headwater_mark::marker_text(Kind::ProbeResult.name()));
            out.push_str("\n\n");
        }
        false => out.push_str(front),
    }

    let _ = writeln!(out, "# The result of {transcript}");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "A probe result is a function of three committed inputs and of nothing else: the \
         transcript at `{transcript}`, the expectations the probes of this corpus declare, and \
         the version of the grader that evaluated them. Fetch the three and this file comes back."
    );
    let _ = writeln!(out);

    let _ = writeln!(out, "## The run this transcript recorded");
    let _ = writeln!(out);
    out.push_str(&record.render());
    if let Some(identity) = &record.identity {
        let _ = writeln!(out);
        out.push_str(&provenance(&identity.selection, selection));
        let _ = writeln!(out);
        out.push_str(READ_SET);
    }
    let _ = writeln!(out);

    out.push_str(&results.render());
    out
}

/// The read set, named here and compared somewhere else.
///
/// # A comparison against the tree in front of a reader cannot live in a
/// derived document
///
/// This is the rule the `selection` comparison below looks like an exception
/// to, and it is worth stating in the file that carries both.
///
/// `generate --check` holds every committed projection to its own bytes. So a
/// sentence in this file that compares a recorded value against the tree in
/// front of the reader changes these bytes whenever that tree moves, and the
/// gate then asks for a regeneration. The read-set digest moves on any edit to
/// any probe of the selection or to any document one of them examines, which is
/// a prose edit somebody makes most weeks. Writing that comparison here would
/// put the staleness of a measurement on a build, through the bytes of a
/// derived document rather than through a rule, and
/// [#171](https://github.com/headwater-ai/headwater/issues/171) rules that it
/// may not.
///
/// Worse than the gate is what a regeneration would write. A result is a
/// statement about the corpus a session met. Refreshing a digest in it would
/// claim the run was taken over a state it was never taken over, so the honest
/// value here is the recorded one and nothing else.
///
/// The `selection` comparison stays because the value it compares against moves
/// only when somebody adds, removes or renames a probe. That is a deliberate
/// act, it is rare, and a regeneration after it states something true.
///
/// `headwater probe stale` is where the read set meets the tree, and no exit
/// status of that verb carries the answer.
const READ_SET: &str = "The `read_set` digest above covers every probe of the selection and every \
     document one of them examines, by path and content. It is recorded here and compared nowhere \
     in this file. A comparison against the tree in front of a reader would move these bytes on \
     every edit to a document the selection points at, and `generate --check` holds this file to \
     its bytes, so the staleness of a measurement would stop a merge. `headwater probe stale` \
     takes the digest and reports which recorded results a change voided.\n";

/// What the four unconfirmed members of the run identity are worth, and the
/// one of them this corpus can answer.
///
/// # The selection is compared and the tree is not, and the difference is what
/// a gate would do about it
///
/// A transcript names six members that a plan fixed before the run. The intake
/// compares the lock and refuses the file when it moved. `headwater probe
/// stale` compares the read set, outside this file and for the reason
/// [`READ_SET`] gives. That leaves three, and the reason none of the three is
/// compared is that the obvious comparison is worse than the gap. The `tree` digest covers every classified document, so a result
/// that reported whether it still agreed would change its own bytes on the first
/// edit to any document of the corpus, and `generate --check` would ask for a
/// regeneration of every committed result on every pull request. A statement
/// nobody can leave standing is not a statement.
///
/// The `selection` digest is different, and it is different in the way that
/// matters: it is taken over the identifiers of the probes selected and over
/// nothing else. It does not move when a probe's prose is edited, and it does
/// move when a probe is added, removed or renamed — which is the one change that
/// makes a recorded run cover a population this corpus no longer declares. So it
/// is compared, and the comparison is reported here rather than refused, because
/// a refusal would replace a graded rate with a notice at the moment somebody
/// added a probe.
///
/// The `seed` and the `harness` are provenance and stay provenance. A seed is a
/// number the caller stated and this corpus holds nothing to compare it against.
/// A harness version is the version of the engine that planned the run, and
/// holding a recorded run to the version reading it would refuse every
/// transcript on the first release.
fn provenance(recorded: &str, composed: &str) -> String {
    match recorded == composed {
        true => "The selection this transcript names is the selection this corpus composes, so \
                 the probes graded below are the probes this run was planned over. The tree, the \
                 seed and the harness above are provenance: nothing compares them, and the tree \
                 in particular is not compared because a result that tracked it would need \
                 rewriting after an edit to any document of this corpus.\n"
            .to_string(),
        false => format!(
            "**The selection this transcript names is not the selection this corpus composes.** \
             The transcript names `{recorded}` and this corpus composes `{composed}`, so a probe \
             was added, removed or renamed after this run was recorded. Every verdict below is \
             over the probes as they stand now, and the rate is over a population this session \
             did not meet. The tree, the seed and the harness above are provenance and nothing \
             compares them.\n"
        ),
    }
}

/// The file stem of a path, which is what `{run}` stands for.
fn stem(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    name.strip_suffix(".md").unwrap_or(name).to_string()
}

/// A count and its noun, in the words the probe crate's reports already use.
fn count(how_many: usize, noun: &str) -> String {
    match how_many {
        1 => format!("1 {noun}"),
        other => format!("{other} {noun}s"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_placeholder_stands_for_the_file_stem_and_never_for_the_directory() {
        assert_eq!(
            stem("docs/probe-runs/first-regression.md"),
            "first-regression"
        );
        assert_eq!(stem("first-regression.md"), "first-regression");
        assert_eq!(stem("docs/probe-runs/no-extension"), "no-extension");
    }
}
