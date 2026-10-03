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
//! the grader's own version, `headwater_probe::grade::VERSION`, and a change to
//! grading moves it. That change makes every committed result stale until it is
//! regenerated. A release that bumps the workspace version does not move it, so
//! a release leaves every committed result as it was (#1317). That is the
//! behavior spec 5 asks for: a result is a function of the grader version, and
//! a series that averaged over two of them would report a change in the
//! instrument as a change in the corpus.

use crate::{
    AmbiguousArms, Declaration, DeclaredIdentity, DefectiveArms, Identity, Kind, Output, Plan,
    Runs, Unwritten,
};
use headwater_census::census::{Census, Outcome};
use headwater_check::lifecycle_state::{Standing, StateFacet, Stood};
use headwater_graph::links::Binding;
use headwater_probe::grade::{Interval, Results};
use headwater_probe::intake::{Record, Tree};
use headwater_probe::plan::Selected;
use headwater_probe::Arm;
use headwater_probe::Tier;
use headwater_query::Surface;

use crate::{MovedSinceRecording, RefusedTranscript};
use headwater_probe::grade::Planned;

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
    // Every committed transcript, in census order, which is path order, with
    // what the state it declares claims about it. The claim decides whether a
    // refusal fails the run: see `RefusedTranscript::held`.
    let state = StateFacet::of(surface.shape());
    // The same reading also decides what the result says first and which
    // state it takes: see `Recorded`.
    let committed: Vec<(&str, bool, Recorded)> = census
        .rows
        .iter()
        .filter(|row| matches!(&row.outcome, Outcome::Typed { kind, .. } if kind == TRANSCRIPT))
        .map(|row| {
            (
                row.path.as_str(),
                holds_a_refusal(&state, row),
                Recorded::of(&state, row),
            )
        })
        .collect();

    // Every file this engine writes, which is the population `readers` takes
    // out of the link set. A generated index links every document of its shelf
    // by construction, so it claims nothing and naming it would put a shelf
    // index in the list under every refusal.
    let generated: Vec<&str> = census
        .rows
        .iter()
        .filter(|row| matches!(&row.outcome, Outcome::Generated { .. }))
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
        for (path, _, _) in &committed {
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
        selected: Some(&runs.selected),
    };
    // Every transcript is graded before any body is written, because a body
    // of a paired run carries the comparisons its transcript takes part in,
    // and a comparison reads the other arm's grade. A transcript the intake
    // refused whole takes part in no comparison: `RefusedTranscript` already
    // fails the run over it.
    let mut graded: Vec<Graded> = Vec::new();
    for (path, promoted, recorded) in committed {
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
                    // `{run}` reaches the name for the same reason it reaches
                    // the identifier: one declaration writes one file per
                    // transcript, so a name that did not carry the stem would
                    // label every result of the shelf the same.
                    name: declared.name.as_ref().map(|name| name.replace(RUN, &stem)),
                };
                // The result names what it is rather than what it found. A
                // verdict is the body's business, and a refused run would
                // otherwise put a rate in the cue a reader scans.
                let composed = crate::derived::Composed {
                    summary: format!(
                        "The grade of the transcript `{stem}`, taken over the probes this corpus \
                         declares and the version of the grader that evaluated them."
                    ),
                    sources: vec![path],
                    // A result over a withdrawn transcript is withdrawn with
                    // it (HW-DR-0063, amended 2026-10-01). A draft transcript
                    // leaves the result at the `live` value, and the first
                    // paragraph of the body says why no figure is current.
                    state: recorded.terminal().map(str::to_string),
                };
                match crate::identity::front_matter(
                    surface,
                    &minted,
                    &output,
                    Kind::ProbeResult,
                    &composed,
                ) {
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
        // A run planned by category, or with a probe named out of it, is
        // graded against the part of the selection it was planned over (#980).
        // A selection this tree cannot recover from the probes the transcript
        // names is graded over those probes alone, because a probe added to
        // this corpus after the recording is not one the run read (#1481).
        let planned = record.identity.as_ref().map(|identity| {
            headwater_probe::grade::planned_over(
                &runs.selected,
                &identity.selection,
                &record.probes,
            )
        });
        let (selected, recovered): (Vec<Selected>, bool) = match planned {
            Some(Some(Planned::Part(part))) => (part, true),
            Some(None) => (
                runs.selected
                    .iter()
                    .filter(|selected| record.probes.contains(&selected.id))
                    .cloned()
                    .collect(),
                false,
            ),
            Some(Some(Planned::Whole)) | None => (runs.selected.clone(), true),
        };
        let mut results = Results::over(&record, &selected);
        // The page names the lock the transcript recorded and not the tree's,
        // so a later lock move leaves a refused result alone (#1481).
        if let Some(refusal) = &record.refusal {
            results.unusable = Some(refusal.recorded());
        }
        // Who reads a refusal, which the state of the recording does not
        // answer. Taken for a refused transcript alone: a result that carries
        // verdicts costs a reader nothing, and a list of readers on every
        // result would move the bytes of a healthy one whenever a document
        // cited it.
        let read_by = match record.refusal {
            Some(_) => readers(surface, &generated, path, &output),
            None => Vec::new(),
        };
        // The file below still says this, and saying it there is not enough:
        // the refusal text is the derived output, so `generate --check`
        // regenerates it faithfully and every gate this repository has stays
        // green over a result that carries no verdict. The run says it too.
        if let Some(refusal) = &record.refusal {
            plan.refused.push(RefusedTranscript {
                transcript: path.to_string(),
                output: output.clone(),
                confirmation: refusal.confirmation(),
                why: refusal.to_string(),
                held: promoted,
                readers: read_by.clone(),
            });
        }
        // A read set, a lock or a selection that moved is graded, and the run
        // names it so that the author of the move meets it (#1338). The page
        // compares none of the three with the tree (#1481). It fails nothing.
        let moved = MovedSinceRecording {
            transcript: path.to_string(),
            output: output.clone(),
            read_set: record.read_set_moved.clone(),
            lock: record.lock_moved.clone(),
            selection: match recovered {
                true => None,
                false => record
                    .identity
                    .as_ref()
                    .map(|identity| identity.selection.clone()),
            },
        };
        if moved.moved() {
            plan.moved_since_recording.push(moved);
        }
        graded.push(Graded {
            path: path.to_string(),
            output,
            front,
            record,
            results,
            recovered,
            read_by,
            recorded,
        });
    }

    let comparisons = pair_arms(&graded, plan);
    for one in &graded {
        let mine: Vec<&Comparison> = comparisons
            .iter()
            .filter(|comparison| comparison.treated == one.path || comparison.control == one.path)
            .collect();
        let bytes = body(one, &mine);
        if let Some(declared) = &declaration.identity {
            if let Err(reason) = crate::identity::unheld(surface, &declared.kind, &bytes) {
                plan.unwritten.push(Unwritten {
                    at: one.output.clone(),
                    kind: Kind::ProbeResult,
                    reason,
                });
                continue;
            }
        }
        plan.outputs.push(Output {
            path: one.output.clone(),
            kind: Kind::ProbeResult,
            committed: true,
            bytes,
        });
    }
}

/// One transcript, graded, before its body is written.
struct Graded {
    path: String,
    output: String,
    front: String,
    record: Record,
    results: Results,
    /// Whether this tree recovered the selection the transcript recorded, as
    /// the whole selection or as the part its events name. Where it did not,
    /// the transcript is graded over the probes it names alone.
    recovered: bool,
    read_by: Vec<String>,
    /// The state the transcript stands at, which decides whether any figure
    /// of this result is a current finding.
    recorded: Recorded,
}

/// Which claim a comparison measures, named by what the two arms differ by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Claim {
    /// `campaign` present against `campaign` absent.
    Governance,
    /// `campaign` absent against `documentation` absent. Both removed the
    /// governance, and only the second removed `docs/`.
    Documents,
    /// A present arm against `documentation` absent.
    Both,
    /// One component arm (#1472) against the `campaign` present arm. A
    /// component that removes a part is the control, and `mcp`, which adds
    /// one, is the treated arm.
    Component(Arm),
}

impl Claim {
    fn sentence(self) -> &'static str {
        match self {
            Claim::Governance => {
                "What the governance changes. The treated arm is the `campaign` present arm and \
                 the control is the `campaign` absent arm, which removed the paths the \
                 `campaign` tier's `ablation` names and kept `docs/`."
            }
            Claim::Documents => {
                "What the documents change. The treated arm is the `campaign` absent arm and the \
                 control is the `documentation` absent arm. Both removed the governance, and \
                 only the control removed `docs/`, so this is the effect of the documents alone \
                 (spec 5)."
            }
            Claim::Both => {
                "What the documents and the governance change together. The treated arm is a \
                 present arm and the control is the `documentation` absent arm."
            }
            Claim::Component(Arm::NoHook) => {
                "What the intent hook changes. The treated arm is the `campaign` present arm and \
                 the control is the `campaign` `no-hook` arm, which removed the paths the \
                 `no-hook` component's delta names. The documents are in both arms (spec 5)."
            }
            Claim::Component(Arm::NoSkills) => {
                "What the skills change. The treated arm is the `campaign` present arm and the \
                 control is the `campaign` `no-skills` arm, which removed the paths the \
                 `no-skills` component's delta names. The documents are in both arms (spec 5)."
            }
            Claim::Component(Arm::NoClaudeMd) => {
                "What `CLAUDE.md` changes. The treated arm is the `campaign` present arm and the \
                 control is the `campaign` `no-claude-md` arm, which removed the paths the \
                 `no-claude-md` component's delta names. The documents are in both arms \
                 (spec 5)."
            }
            Claim::Component(Arm::Mcp) => {
                "What the MCP server adds. The treated arm is the `campaign` `mcp` arm, which \
                 added the paths the `mcp` component's delta names, and the control is the \
                 `campaign` present arm. The documents are in both arms (spec 5)."
            }
            Claim::Component(Arm::Present | Arm::Absent) => {
                unreachable!("the present and absent arms are not components")
            }
        }
    }
}

/// Two graded arms of one run, compared.
#[derive(Clone, Debug)]
struct Comparison {
    claim: Claim,
    treated: String,
    control: String,
    treated_rate: Option<Interval>,
    control_rate: Option<Interval>,
    treated_graded: usize,
    control_graded: usize,
    treated_session_refused: usize,
    control_session_refused: usize,
    /// Whether both arms' transcripts stand at a `live` state. A difference
    /// between two runs that are not both current is a figure, and a
    /// direction read off it is a claim nobody stands behind (#1509).
    current: bool,
}

impl Comparison {
    fn of(claim: Claim, treated: &Graded, control: &Graded) -> Comparison {
        Comparison {
            claim,
            treated: treated.path.clone(),
            control: control.path.clone(),
            treated_rate: treated.results.rate(),
            control_rate: control.results.rate(),
            treated_graded: treated.results.graded(),
            control_graded: control.results.graded(),
            treated_session_refused: treated.results.session_refusals(),
            control_session_refused: control.results.session_refusals(),
            current: treated.recorded == Recorded::Live && control.recorded == Recorded::Live,
        }
    }

    fn render(&self) -> String {
        use headwater_probe::grade::{percent, points};
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(out, "{}", self.claim.sentence());
        let _ = writeln!(out);
        let arm = |path: &str, rate: &Option<Interval>, graded: usize, refused: usize| {
            let rate = match rate {
                Some(rate) => format!(
                    "{} of {graded} graded sessions satisfied their expectation, {}, in a 95% \
                     interval of {} to {}",
                    (rate.point * graded as f64).round() as usize,
                    percent(rate.point),
                    percent(rate.low),
                    percent(rate.high)
                ),
                None => "no graded session".to_string(),
            };
            format!(
                "`{path}`: {rate}. {} refused by the session itself.",
                count(refused, "session")
            )
        };
        let _ = writeln!(
            out,
            "- treated, {}",
            arm(
                &self.treated,
                &self.treated_rate,
                self.treated_graded,
                self.treated_session_refused
            )
        );
        let _ = writeln!(
            out,
            "- control, {}",
            arm(
                &self.control,
                &self.control_rate,
                self.control_graded,
                self.control_session_refused
            )
        );
        let _ = writeln!(out);
        match (&self.treated_rate, &self.control_rate) {
            (Some(treated), Some(control)) => {
                let difference = treated.minus(control);
                let reading = match (difference.low > 0.0, difference.high < 0.0) {
                    _ if !self.current => {
                        "The transcript of at least one arm is not current, so this page states \
                         no direction at the 5% level."
                    }
                    (true, _) => {
                        "The interval is above zero, so the treated arm satisfied more often at \
                         the 5% level."
                    }
                    (_, true) => {
                        "The interval is below zero, so the treated arm satisfied less often at \
                         the 5% level."
                    }
                    _ => {
                        "The interval contains zero, so this run does not separate the two arms \
                         at the 5% level."
                    }
                };
                let _ = writeln!(
                    out,
                    "The difference is {}, in a 95% Newcombe interval of {} to {}. {reading}",
                    points(difference.point),
                    points(difference.low),
                    points(difference.high)
                );
            }
            _ => {
                let _ = writeln!(
                    out,
                    "One arm has no graded session, so this run states no difference."
                );
            }
        }
        out
    }
}

/// Every paired-tier transcript this run graded, grouped into runs and
/// compared, and reported where a compared arm carries a defect or where no
/// arm could be chosen at all.
///
/// # The key is the run, and the tier is a role inside it
///
/// Two transcripts belong to one run when they share the selection, the model,
/// the served version and the tree. The first three are the members spec 5 pins
/// before a run starts. The tree joined them for #980: a campaign is recorded
/// from workspaces built from one pinned commit, so a pair recorded over two
/// commits compares two corpora, and the key must say so rather than pair them.
///
/// Inside a run each transcript has a role, its tier and its arm, and a role
/// holds at most one transcript. Three comparisons read the roles:
///
/// - `campaign` present against `campaign` absent, the governance.
/// - `campaign` absent against `documentation` absent, the documents alone,
///   which spec 5 names as the only reading of that claim.
/// - a present arm against `documentation` absent, both together.
///
/// Each component arm (#1472) is one more role of the `campaign` tier, and it
/// is compared with the `campaign` present arm alone: the present arm is
/// treated against a component that removes a part, and `mcp`, which adds
/// one, is treated against the present arm (spec 5).
///
/// # One present arm serves both tiers
///
/// A present arm removes the instrument and the seal and nothing else, at every
/// tier, so the `campaign` present tree and the `documentation` present tree
/// are one tree. The owner ruled on #980 (2026-09-28) that a run records it
/// once. The third comparison reads the `documentation` present arm where one
/// was recorded and the `campaign` present arm otherwise. The comparisons share
/// that arm, so their estimates are correlated. Each one is still unbiased,
/// which is the condition the ruling set.
///
/// # A compared arm with a defect fails the run
///
/// A refusal the recorder or the probe declaration caused is a defect, and its
/// remedy is to record the session again. A refusal the session caused is data
/// and is printed per arm. See [`headwater_probe::grade::Refusal::is_session`].
///
/// # A role is chosen only where it is unambiguous
///
/// A corpus may hold a stale transcript beside its replacement under one key.
/// Nothing in the run identity says which one a reader means, so a key where
/// any role holds more than one transcript is reported as [`AmbiguousArms`]
/// and compares nothing.
fn pair_arms(graded: &[Graded], plan: &mut Plan) -> Vec<Comparison> {
    type Key<'a> = (&'a str, &'a str, &'a str, &'a str);
    let paired: Vec<(&Graded, &headwater_probe::intake::Identity)> = graded
        .iter()
        .filter(|one| one.record.refusal.is_none())
        .filter_map(|one| Some((one, one.record.identity.as_ref()?)))
        .filter(|(_, identity)| identity.tier.pairs_arms())
        .collect();
    let mut keys: Vec<Key> = paired
        .iter()
        .map(|(_, identity)| {
            (
                identity.selection.as_str(),
                identity.model.as_str(),
                identity.served_version.as_str(),
                identity.tree.as_str(),
            )
        })
        .collect();
    keys.sort_unstable();
    keys.dedup();

    let mut comparisons = Vec::new();
    for (selection, model, served_version, tree) in keys {
        let under: Vec<&(&Graded, &headwater_probe::intake::Identity)> = paired
            .iter()
            .filter(|(_, identity)| {
                identity.selection == selection
                    && identity.model == model
                    && identity.served_version == served_version
                    && identity.tree == tree
            })
            .collect();
        let role = |tier: Tier, arm: Arm| -> Vec<&Graded> {
            let mut found: Vec<&Graded> = under
                .iter()
                .filter(|(_, identity)| identity.tier == tier && identity.arm == arm)
                .map(|(one, _)| *one)
                .collect();
            found.sort_by(|a, b| a.path.cmp(&b.path));
            found
        };
        let roles = [
            role(Tier::Campaign, Arm::Present),
            role(Tier::Campaign, Arm::Absent),
            role(Tier::Documentation, Arm::Present),
            role(Tier::Documentation, Arm::Absent),
        ];
        // Each component arm of #1472 is one more role, compared with the
        // `campaign` present arm alone (spec 5).
        let components: Vec<(Arm, Vec<&Graded>)> = Arm::ALL
            .into_iter()
            .filter(|arm| arm.is_component())
            .map(|arm| (arm, role(Tier::Campaign, arm)))
            .collect();
        if roles
            .iter()
            .chain(components.iter().map(|(_, holders)| holders))
            .any(|holders| holders.len() > 1)
        {
            let of_arm = |arm: Arm| {
                let mut paths: Vec<String> = under
                    .iter()
                    .filter(|(_, identity)| identity.arm == arm)
                    .map(|(one, _)| one.path.clone())
                    .collect();
                paths.sort();
                paths
            };
            plan.ambiguous_arms.push(AmbiguousArms {
                selection: selection.to_string(),
                model: model.to_string(),
                served_version: served_version.to_string(),
                present: of_arm(Arm::Present),
                absent: of_arm(Arm::Absent),
                components: components
                    .iter()
                    .map(|(arm, _)| (arm.name().to_string(), of_arm(*arm)))
                    .filter(|(_, paths)| !paths.is_empty())
                    .collect(),
            });
            continue;
        }
        let one = |index: usize| roles[index].first().copied();
        let (campaign_present, campaign_absent, documentation_present, documentation_absent) =
            (one(0), one(1), one(2), one(3));

        let mut chosen: Vec<(Claim, &Graded, &Graded)> = Vec::new();
        if let (Some(treated), Some(control)) = (campaign_present, campaign_absent) {
            chosen.push((Claim::Governance, treated, control));
        }
        if let (Some(treated), Some(control)) = (campaign_absent, documentation_absent) {
            chosen.push((Claim::Documents, treated, control));
        }
        if let (Some(treated), Some(control)) = (
            documentation_present.or(campaign_present),
            documentation_absent,
        ) {
            chosen.push((Claim::Both, treated, control));
        }
        for (arm, holders) in &components {
            if let (Some(present), Some(component)) = (campaign_present, holders.first().copied()) {
                if arm.adds() {
                    chosen.push((Claim::Component(*arm), component, present));
                } else {
                    chosen.push((Claim::Component(*arm), present, component));
                }
            }
        }
        for (claim, treated, control) in chosen {
            let (treated_defects, control_defects) =
                (treated.results.defects(), control.results.defects());
            if treated_defects > 0 || control_defects > 0 {
                plan.defective_arms.push(DefectiveArms {
                    selection: selection.to_string(),
                    model: model.to_string(),
                    served_version: served_version.to_string(),
                    present: treated.path.clone(),
                    absent: control.path.clone(),
                    present_defects: treated_defects,
                    absent_defects: control_defects,
                });
            }
            comparisons.push(Comparison::of(claim, treated, control));
        }
    }
    comparisons
}

/// Every document of this corpus that links a refused recording or the result
/// derived from it, in path order, once each.
///
/// # The state of a recording says nothing about who reads it
///
/// [HW-DR-0062](../../../../docs/decisions/0062-a-refused-recording-is-held-by-the-reliance-its-state-claims-and-not-by-promotion.md)
/// releases the gate where the state a recording stands in says that no reader
/// relies on it, and that reading is about one document: the recording. The
/// documents that cite it are other documents, and retiring the recording
/// leaves every sentence in them where it was. `0149a92c` is the measurement:
/// it moved a refused transcript to `deprecated`, the run went green, and
/// [HW-OBL-0010](../../../../docs/obligations/0010-the-corpus-descriptor-exists-and-no-probe-has-run-against.md)
/// and
/// [HW-OBL-0124](../../../../docs/obligations/0124-a-probe-result-is-printed-and-never-committed-so-nothing-regenerates-one.md)
/// went on standing at `discharged` over a rate the result does not carry.
///
/// So this is reported and it fails nothing, which is that ruling's posture one
/// hop out. The remedy is a rewrite of a sentence a person has to read, and a
/// gate over it would be a gate on prose that the run cannot repair.
///
/// # It reads the links the graph build already bound
///
/// [`headwater_graph::links`] resolves every prose link of every document the
/// census read, against the directory of the document that wrote it. A second
/// reading here would be a second definition of what a prose link is, which is
/// the argument `headwater_check::link_path` already makes about the same set.
/// A link into a result that no run has written yet binds as `Missing` and a
/// link into one that exists binds as `Corpus`, so both bindings are read: a
/// reader of a result is a reader whether or not the file is on disk.
///
/// Two paths are dropped from the population. A generated file links what its
/// declaration tells it to, so a shelf index would appear under every refusal
/// and mean nothing. The transcript and the result themselves are not their own
/// readers.
fn readers(
    surface: &Surface<'_>,
    generated: &[&str],
    transcript: &str,
    output: &str,
) -> Vec<String> {
    let mut found: Vec<String> = surface
        .graph()
        .links
        .iter()
        .filter(|link| points_at(&link.binding, transcript) || points_at(&link.binding, output))
        .map(|link| link.source_path.clone())
        .filter(|from| from != transcript && from != output && !generated.contains(&from.as_str()))
        .collect();
    found.sort();
    found.dedup();
    found
}

/// Whether a bound link resolved to this path.
///
/// Three of the six bindings carry a path and each of the three is a reader.
/// `Repository` is a file outside the corpus root, which no result or
/// transcript is, and it is matched rather than dropped because a binding that
/// names the path is the same claim wherever the census drew its boundary.
fn points_at(binding: &Binding, path: &str) -> bool {
    match binding {
        Binding::Corpus {
            path: destination, ..
        }
        | Binding::Repository { path: destination }
        | Binding::Missing { path: destination } => destination == path,
        Binding::Unnormalizable { .. } | Binding::SameDocument | Binding::External => false,
    }
}

/// Whether a refusal over this transcript fails the run.
///
/// The question is whether the corpus claims a reader may rely on the
/// recording, and the taxonomy answers it rather than this crate. Every value
/// of a state vocabulary carries a role, and
/// `.headwater/packages/headwater-standard/taxonomy.yml` gives the roles in its own words:
/// `current` is "the document states what holds now, and a reader may rely on
/// it", and `deprecated` is "the document is no longer to be relied on". So the
/// reading is [`headwater_check::lifecycle_state::StateFacet::standing`], which
/// is the fold spec 3's dependency rule already takes, and a sixth state added
/// to the vocabulary is classified here by the role it declares and by no edit
/// to this crate.
///
/// **A run fails unless a role this engine can read says nothing relies on the
/// document.** Two roles say it, and each releases the refusal for its own
/// reason:
///
/// - `initial` is a recording somebody is still working on. The remedy for a
///   refusal is a fresh recording rather than an edit anybody can make, and a
///   gate a contributor cannot clear is a gate that gets removed.
/// - a `terminal-` role is a recording the corpus has retired. It is kept as a
///   record and nothing new may rest on it, so a refusal over it contradicts
///   nothing the corpus asserts. This arm is
///   [#814](https://github.com/headwater-ai/headwater/issues/814). The standard
///   lifecycle has no transition back to the initial state, so until it landed
///   there was no legal state in which a recording could go stale, and every
///   change that moved the lock was tolled four fresh sessions.
///
/// Everything else fails, and that is the silent pass this module exists to
/// close. A row that parsed no document, a document that declares no state, a
/// value the vocabulary does not admit and a value whose role this engine
/// cannot fold are four ways of saying that nothing has stated where this
/// recording stands. A transcript that escaped the gate by declaring nothing is
/// the shape `f615fb86` landed, and the reading that deciding nothing about a
/// state means deciding nothing about the run would reopen it. The rules that
/// own the middle two are `facet.required.missing` and
/// `facet.value.not_permitted`, and this holds the refusal until one of them is
/// answered.
///
/// It reads the row the census already parsed rather than opening the file
/// again, for the reason [`headwater_census::census::Row::document`] states: a
/// second read is a second account of one file, and the two can differ.
fn holds_a_refusal(state: &StateFacet, row: &headwater_census::census::Row) -> bool {
    let Some(document) = row.document.as_ref() else {
        return true;
    };
    match state.stood(&document.facets) {
        Stood::At(value) => !matches!(
            state.standing(value),
            Standing::Initial | Standing::Terminal
        ),
        Stood::Undeclared | Stood::NotAState(_) => true,
    }
}

/// The state a transcript stands at, in the terms the role beside it gives.
///
/// [`holds_a_refusal`] asks one question of the same reading, whether a refusal
/// fails the run. This asks what a reader of the result is owed before any
/// figure: a result over a transcript that is not at a `live` state reports a
/// run nobody stands behind, and until #1509 it said so nowhere. It names no
/// state value. The values are the taxonomy's, and the answer is the role.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Recorded {
    /// A state whose role is `live`. The result is a current finding.
    Live,
    /// A state the vocabulary holds with another answer: terminal, initial,
    /// or none of the three.
    At(String, Standing),
    /// The transcript declares no state, or a value the state facet does not
    /// hold, so nothing this taxonomy reads says what the run stands at.
    Unread(Option<String>),
}

impl Recorded {
    fn of(state: &StateFacet, row: &headwater_census::census::Row) -> Recorded {
        let Some(document) = row.document.as_ref() else {
            return Recorded::Unread(None);
        };
        match state.stood(&document.facets) {
            Stood::At(value) => match state.standing(value) {
                Standing::Live => Recorded::Live,
                standing => Recorded::At(value.to_string(), standing),
            },
            Stood::Undeclared => Recorded::Unread(None),
            Stood::NotAState(value) => Recorded::Unread(Some(value.to_string())),
        }
    }

    /// The state value, where its role is terminal. This is the one answer a
    /// result takes as its own state.
    fn terminal(&self) -> Option<&str> {
        match self {
            Recorded::At(value, Standing::Terminal) => Some(value),
            _ => None,
        }
    }

    /// The paragraph a result opens with when its transcript is not current,
    /// and `None` when it is.
    fn opening(&self) -> Option<String> {
        const OWED: &str = "No figure below is a current finding, and this result states no \
                            direction at the 5% level.";
        let said = match self {
            Recorded::Live => return None,
            Recorded::At(value, Standing::Terminal) => format!(
                "The transcript this result grades stands at `{value}`, a state that ends its \
                 lifecycle, so the run it recorded is withdrawn."
            ),
            Recorded::At(value, Standing::Initial) => format!(
                "The transcript this result grades stands at `{value}`, the state a document \
                 holds before anything promotes it, so nothing relies on the run it recorded yet."
            ),
            Recorded::At(value, _) => format!(
                "The transcript this result grades stands at `{value}`, a state whose role this \
                 taxonomy does not read as live, terminal or initial."
            ),
            Recorded::Unread(Some(value)) => format!(
                "The transcript this result grades declares the state `{value}`, and the state \
                 facet of this taxonomy holds no such value, so it declares no state this \
                 taxonomy reads."
            ),
            Recorded::Unread(None) => "The transcript this result grades declares no state this \
                                       taxonomy reads."
                .to_string(),
        };
        Some(format!("{said} {OWED}"))
    }
}

/// The whole file: the front matter, the marker where there is no front matter,
/// and the report.
fn body(graded: &Graded, comparisons: &[&Comparison]) -> String {
    use std::fmt::Write;
    let (front, transcript, record, results, read_by) = (
        graded.front.as_str(),
        graded.path.as_str(),
        &graded.record,
        &graded.results,
        graded.read_by.as_slice(),
    );
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
    // Before any figure, because a reader who stops at the first paragraph
    // must not leave with a withdrawn run's numbers as a finding (#1509).
    if let Some(opening) = graded.recorded.opening() {
        let _ = writeln!(out, "{opening}");
        let _ = writeln!(out);
    }
    let _ = writeln!(
        out,
        "A probe result is a function of committed inputs and of nothing else: the transcript \
         at `{transcript}` and the state it stands at, the expectations the probes it names \
         declare, the version of the grader that evaluated them, and for a compared arm the \
         result of the other arm. Fetch those and this file comes back."
    );
    let _ = writeln!(out);

    let _ = writeln!(out, "## The run this transcript recorded");
    let _ = writeln!(out);
    // The page variant: plain, because this is a generated document's own
    // bytes and never a terminal report, and with no comparison against the
    // tree, because a committed result pins only what it read (#1481).
    out.push_str(&record.render_page());
    if let Some(identity) = &record.identity {
        let _ = writeln!(out);
        out.push_str(&provenance(&identity.selection, graded.recovered));
        let _ = writeln!(out);
        out.push_str(READ_SET);
    }
    let _ = writeln!(out);

    out.push_str(&results.render(headwater_check::paint::ColorMode::Plain));
    if !comparisons.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(out, "## The comparisons this arm takes part in");
        let _ = writeln!(out);
        for (index, comparison) in comparisons.iter().enumerate() {
            if index > 0 {
                let _ = writeln!(out);
            }
            out.push_str(&comparison.render());
        }
    }
    if record.refusal.is_some() {
        let _ = writeln!(out);
        out.push_str(&read_set_of_this_result(read_by));
    }
    out
}

/// The section a refused result carries: where a reader of this file would meet
/// a measurement that does not exist.
///
/// # This is the one comparison against the tree that a result keeps
///
/// [`READ_SET`] rules that a committed result pins only what it read (the
/// owner's ruling on #1481, 2026-10-03), so it compares no recorded digest
/// with the tree in front of a reader. This list is not something the
/// transcript read either: it names the documents that link the result. It
/// stays for a reason an outside reader holds, and the ruling on #1481 has not
/// been put to it. It moves when a document
/// starts or stops linking this result or the transcript it graded, which is an
/// act somebody takes on purpose. It does not move when the prose of a citing
/// document is edited. And the merge it stops is the merge that should stop: a
/// document that starts citing a result with no verdict in it changes these
/// bytes, `generate --check` asks for a regeneration, and the list a reader
/// meets is the list this corpus holds.
///
/// A result that carries verdicts has no such section, so nothing here moves
/// the bytes of a healthy result.
fn read_set_of_this_result(read_by: &[String]) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "## What reads this result");
    let _ = writeln!(out);
    match read_by {
        [] => {
            let _ = writeln!(
                out,
                "This result carries no verdict, so a rate taken off it is a rate over \
                 none. No document of this corpus links this result or the transcript it \
                 graded, so no sentence of this corpus rests on it."
            );
        }
        readers => {
            let _ = writeln!(
                out,
                "This result carries no verdict, so a rate taken off it is a rate over \
                 none. The documents of this corpus below link this result or the transcript \
                 it graded, and each one is where a reader meets a measurement this corpus \
                 does not hold:"
            );
            // The list stays and its count goes: a count of the readers is a
            // fold that a text merge of two branches writes wrong (#1058).
            let _ = writeln!(out);
            for reader in readers {
                let _ = writeln!(out, "- `{reader}`");
            }
            let _ = writeln!(out);
            let _ = writeln!(
                out,
                "The state this recording stands in decides whether the refusal fails a \
                 run, and it says nothing about the list above: those are other documents, \
                 and retiring the recording leaves every sentence in them where it was."
            );
        }
    }
    out
}

/// The read set, named here and compared somewhere else.
///
/// # A committed result pins only what it read
///
/// The owner ruled on #1481 (2026-10-03) that a committed result pins only
/// what it read, so that an unrelated merge cannot eject a queued pull
/// request. A result is a function of the transcript's bytes, the state it
/// stands at, the declarations of the probes it names, the grader version and,
/// for a compared arm, the other arm's result. It names the `lock`, the
/// `selection` and the `read_set` the transcript recorded as provenance, and it
/// compares none of them with the tree in front of a reader.
///
/// `generate --check` holds every committed projection to its own bytes. So a
/// sentence in this file that compares a recorded value against the tree in
/// front of the reader changes these bytes whenever that tree moves, and the
/// gate then asks for a regeneration. The read-set digest moves on any edit to
/// any probe of the selection or to any document one of them examines, the
/// lock moves on every package publish, and the composed selection moves on
/// every new probe. Each comparison moved every result it touched, and each
/// moved result was a conflict for every other branch that regenerated the
/// shelf. [#171](https://github.com/headwater-ai/headwater/issues/171) rules
/// that the staleness of a measurement does not stop a build.
///
/// Before #1481 two comparisons stood as exceptions: a mark that the read set
/// or the lock moved, which moved the bytes once (#1338), and the composed
/// selection with the count of the probes this corpus declares. The ruling
/// removed both. The run report of `headwater generate` names each result
/// whose recorded lock, read set or selection moved, and `headwater probe
/// stale` names which recorded results a change voided. That output is
/// printed, never committed, and no exit status carries it.
///
/// A regeneration that refreshed a digest here would also claim the run was
/// taken over a state it was never taken over, so the honest value is the
/// recorded one and nothing else.
const READ_SET: &str = "The `read_set` digest above covers every probe of the selection and every \
     document one of them examines, by path and content. This file names it as provenance and \
     compares it with nothing, and the same holds for the lock and the selection: an edit to a \
     document, a new probe or a moved lock leaves these bytes alone. `headwater generate` names \
     each result whose recorded lock, read set or selection moved, and `headwater probe stale` \
     takes the digest and reports which recorded results a change voided. Neither one fails for \
     it.\n";

/// What the four unconfirmed members of the run identity are worth.
///
/// # None of them is compared on the page
///
/// A transcript names six members that a plan fixed before the run. The intake
/// compares the lock to decide whether to read the file at all, and the run
/// report and `headwater probe stale` compare the lock, the read set and the
/// selection, for the reason [`READ_SET`] gives. The page compares none of
/// them.
///
/// The selection decides which probes are graded and nothing else. Where this
/// tree recovers the recorded digest, as the whole selection or as the part of
/// it the transcript's events name, the probes graded are the probes the run
/// was planned over. A probe added later is not in that part, so it is not a
/// session this run owed, and the page does not change for it. Where this tree
/// cannot recover the digest, the transcript is graded over the probes its
/// events name alone, and a probe the run was planned over and never ran is
/// not reported. That is the cost of not comparing, and the run report names
/// such a result. Spec 15 records it.
///
/// The `tree`, the `seed` and the `harness` are provenance and stay provenance.
/// The tree digest covers every classified document, so a result that tracked
/// it would move on any edit to the corpus. A seed is a number the caller
/// stated and this corpus holds nothing to compare it against. A harness
/// version is the version of the engine that planned the run, and holding a
/// recorded run to the version reading it would refuse every transcript on the
/// first release.
fn provenance(recorded: &str, recovered: bool) -> String {
    match recovered {
        true => format!(
            "The selection this transcript names is `{recorded}`, and the probes graded below are \
             the probes it was planned over, as this corpus declares them now. A probe added to \
             this corpus after the recording is not a session this run owed. The tree, the seed \
             and the harness above are provenance and nothing compares them.\n"
        ),
        false => format!(
            "**This corpus cannot recover the selection this transcript names** from the probes \
             its events name. The transcript names `{recorded}`, and every verdict below is over \
             the probes its events name alone, so a probe the run was planned over and never ran \
             is not graded here. The tree, the seed and the harness above are provenance and \
             nothing compares them.\n"
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
