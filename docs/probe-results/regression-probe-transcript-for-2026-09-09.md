---
"headwater:generated": "probe_result. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file."
id: HW-RESULT-regression-probe-transcript-for-2026-09-09
---

# The result of docs/probe-runs/regression-probe-transcript-for-2026-09-09.md

A probe result is a function of three committed inputs and of nothing else: the transcript at `docs/probe-runs/regression-probe-transcript-for-2026-09-09.md`, the expectations the probes of this corpus declare, and the version of the grader that evaluated them. Fetch the three and this file comes back.

## The run this transcript recorded

A regression run in the present arm, on claude-haiku-4-5 at 2026-09-09.
served version claude-haiku-4-5-20251001, tree sha256:73f19c035efb9259f56750d65d14ca049e3dd31b3749f75be03a4ec359e33aad, selection sha256:2ae2482aea0cf2c00e83fac793d72d09c287514680ef632d535f1607d12a913a, read set sha256:78d36a3aecbfac469d19049a6f1d0286b97bbdf93793b27b216ebf5b95d76e84, seed 0, harness 0.1.1.
realized cost $0.28, which the adaptive layer reads as the cost of its own instrument.

4 events over 4 of the 4 probes this corpus declares, in 4 sessions and 24 tool calls.

The engine confirmed the taxonomy, that every member of the run identity is present, the membership of every probe named, that no key outside the closed set appears, and that a realized cost was recorded. Present is not confirmed: of the six members a plan fixes before a run, the lock is the one compared here, and it refuses the file. `headwater probe stale` compares the read set against the tree in front of it. It graded nothing: a verdict is a function of this transcript, the expectations these probes declare and a grader version, and `headwater probe grade` is the verb that holds all three.

The selection this transcript names is the selection this corpus composes, so the probes graded below are the probes this run was planned over. The tree, the seed and the harness above are provenance: nothing compares them, and the tree in particular is not compared because a result that tracked it would need rewriting after an edit to any document of this corpus.

The `read_set` digest above covers every probe of the selection and every document one of them examines, by path and content. It is recorded here and compared nowhere in this file. A comparison against the tree in front of a reader would move these bytes on every edit to a document the selection points at, and `generate --check` holds this file to its bytes, so the staleness of a measurement would stop a merge. `headwater probe stale` takes the digest and reports which recorded results a change voided.

Graded by grader 0.1.1.
A regression run in the present arm, on claude-haiku-4-5 at served version claude-haiku-4-5-20251001.

## The verdicts

- HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor (discovery, expects opened)
    session regression-20260909-cold-current: not satisfied — 15 recorded calls, and none named any of the 1 document
- HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer (sufficiency, expects answered)
    session regression-20260909-tombstone-current: not satisfied — the session ended with no answer
- HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it (navigability, expects opened)
    session regression-20260909-adjudication-current: not satisfied — 2 recorded calls, and none named any of the 1 document
- HW-PROBE-the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document (discovery, expects opened)
    session regression-20260909-authoring-clean: not satisfied — 3 recorded calls, and none named any of the 1 document

## The rate, and the denominator it is over

0 of 4 graded sessions satisfied their expectation: 0.0%, in a 95% interval of 0.0% to 49.0%.
The denominator is the graded sessions and never the selected probes. 0 sessions reached no verdict, and a session with no verdict is outside both halves of that fraction.

An interval that overlaps the previous run's is variance and one that does not is drift. This is one arm, so it estimates no effect: an efficacy claim is a comparison of two results, and the arm each one recorded is on it.
