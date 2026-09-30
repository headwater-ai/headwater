---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-regression-probe-transcript-for-2026-09-09
title: Probe result for regression-probe-transcript-for-2026-09-09
status: current
status_since: 2026-09-09
summary: "The grade of the transcript `regression-probe-transcript-for-2026-09-09`, taken over the probes this corpus declares and the version of the grader that evaluated them."
last_verified: 2026-09-09
---

# The result of docs/probe-runs/regression-probe-transcript-for-2026-09-09.md

A probe result is a function of three committed inputs and of nothing else: the transcript at `docs/probe-runs/regression-probe-transcript-for-2026-09-09.md`, the expectations the probes of this corpus declare, and the version of the grader that evaluated them. Fetch the three and this file comes back.

## The run this transcript recorded

A regression run in the present arm, on claude-haiku-4-5 at 2026-09-09.
served version claude-haiku-4-5-20251001, tree sha256:73f19c035efb9259f56750d65d14ca049e3dd31b3749f75be03a4ec359e33aad, selection sha256:2ae2482aea0cf2c00e83fac793d72d09c287514680ef632d535f1607d12a913a, read set sha256:78d36a3aecbfac469d19049a6f1d0286b97bbdf93793b27b216ebf5b95d76e84, seed 0, harness 0.1.1.
realized cost $0.28, which the adaptive layer reads as the cost of its own instrument.
It was planned against taxonomy sha256:de4da2c26f7744de974e46575a21d8fcdbeee813ba5dd4f23b71993e99527650, and this tree carries another, and the read set of its probes moved since the recording. Each verdict below is what the session did over the documents it met, graded against the expectations this tree declares now, which can differ from the ones the session ran under. `headwater probe stale` names what moved.

4 events over 4 of the 10 probes this corpus declares, in 4 sessions and 24 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed: of the six members a plan fixes before a run, the lock is the one compared here. A lock that differs refuses the file only where this tree composes no read set over its probes, and a read set that differs marks the verdicts and keeps them. `headwater probe stale` compares the read set against the tree in front of it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

**The selection this transcript names is not the selection this corpus composes.** The transcript names `sha256:2ae2482aea0cf2c00e83fac793d72d09c287514680ef632d535f1607d12a913a` and this corpus composes `sha256:50ed5ce43072d9f674a8dc52e1dcad84de4d649e39a77ce22179836188196b09`. The transcript's digest is the digest of 4 of the 10 probes this corpus composes, so the run was planned over that part of the selection, or the rest were added after it was recorded. Every verdict below is over that part, and a probe outside it is not a session this run owed. The tree, the seed and the harness above are provenance and nothing compares them.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. It is recorded here, and this file names no digest the tree in front of a reader composes. Where that tree composes another one, a single sentence above says that the read set moved, and it moves these bytes once. A digest from that tree would move these bytes on every edit to a document the selection points at, and `generate --check` holds this file to its bytes, so the staleness of a measurement would stop a merge. `headwater probe stale` takes the digest and reports which recorded results a change voided.

Graded by grader 0.5.0.
A regression run in the present arm, on claude-haiku-4-5 at served version claude-haiku-4-5-20251001.

## The verdicts

- HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor (discovery, expects opened)
    session regression-20260909-cold-current: not satisfied — 15 recorded calls, and none named any of the 1 document
- HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer (sufficiency, expects answered)
    session regression-20260909-tombstone-current: not satisfied — the session ended with no answer
- HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it (navigability, expects opened)
    session regression-20260909-adjudication-current: satisfied — event 3, call 2: `Read /tmp/headwater-724-probes.Ufy2y3/adjudication-workspace/docs/decisions/0008-probe-cost-and-cadence.md`
- HW-PROBE-the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document (discovery, expects opened)
    session regression-20260909-authoring-clean: not satisfied — 3 recorded calls, and none named any of the 1 document

## The rate, and the denominator it is over

1 of 4 graded sessions satisfied their expectation: 25.0%, in a 95% interval of 4.6% to 69.9%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.
