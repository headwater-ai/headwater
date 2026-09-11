---
id: HW-OBL-0124
status: discharged
status_since: 2026-09-11
waiting_on: build
summary: "A committed transcript generates a result, regeneration holds the pair, and the result this corpus holds carries a verdict for every probe it graded."
last_verified: 2026-09-11
title: "A probe result is printed and never committed, so nothing regenerates one"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-ai-integration
---

<!-- headwater allow=lifecycle.transition.not_permitted scope=file until=2027-12-31 reason=accepted_deviation note=a discharge taken on a measurement that does not exist has to be retractable, and `discharged` is terminal in the `obligation` regime -->

# A probe result is printed and never committed, so nothing regenerates one

## Context

[Spec 5](../spec/05-ai-integration.md#a-run-produces-a-snapshot-and-a-document) says what a probe result is: "The **probe result** is a document, generated from the transcript, the expectations and the grader version. It carries the `regenerated` warrant, and `generate --check` proves it." [Spec 12](../spec/12-check-layer.md#the-correctness-roots) reads the same sentence as an instrument, because the grader is a correctness root and a root owes its own test.

`headwater probe grade` now evaluates every expectation form and returns a verdict for each. It writes that report to standard output and it writes no document. Three facts were measured on this tree rather than argued:

- `docs/probe-runs/` is declared by the overlay and held no file, so no transcript was corpus content. The section at the end of this record is what moved that fact and what it did not move.
- No kind of this taxonomy types a probe result, so a result document has nothing to be.
- `headwater generate --check` regenerates three documents, and each is a shelf index or a shelf section.

So the artifact that spec 5 calls a probe result exists as terminal output and as nothing else.

**The second and the third of those three are false, and the first is true.** `probe_result` is a kind of this taxonomy, `docs/probe-results/**` is a shelf that holds it, and `probe_result` is the ninth declarable projection kind. The generator reads every committed transcript, grades it against the selection that `headwater probe plan` composes, and writes one document for each. It writes nothing at all when the plan refuses. A plan that gives up part way through holds the probes it read and none of the rest. A result over that part would report a rate over a denominator no document declares. The run names the probe that stopped the plan in the place a result would be. A recorded fixture tree holds the pair and asserts the three directions of the gate. The committed pair is green. An unrelated edit leaves it green, and an edit to one event of the transcript turns it red.

This corpus wrote no result while the first fact held, and `headwater generate` printed the reason on every run rather than passing the declaration over. The first fact stopped holding on 2026-09-09. The run now writes a result and reports that a confirmation refused the transcript it wrote it from, which is the same posture applied to the failure that replaced the empty one.

## Obligation

**The corpus owes a probe result as corpus content, and the debt is one link in a chain.** A result is a function of a transcript, and a recorder that observes a session from outside it writes that transcript. No verb of this engine writes one, and spec 5 refuses a transcript that an agent wrote about its own session. The chain is the recorder, then a committed transcript, then a kind for a result, then a generator over it, and only then the test.

Two consequences follow while the chain is unbuilt, and each is a claim this corpus cannot make yet.

**The standing test of a correctness root is half absent.** A recomputed number is what makes a result citable, and the recomputation is what catches a result that disagrees with its own transcript. The recorded fixture set of `headwater-probe` holds the grader against three transcripts and catches a defect in the engine. It says nothing about a number in a document, because no document holds one.

**A published rate would carry the `asserted` warrant.** [Spec 3](../spec/03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two) refuses an asserted document as evidence for anything. A rate written into this corpus today therefore discharges no evidence obligation. That is the failure the placement table in spec 5 exists to prevent, reached through the layer the table describes.

## Discharge

The issue that lands a recorder and a committed transcript discharges the first link. The issue that declares a `probe_result` kind and a generator over it discharges the rest. It cannot start earlier, because a projection whose source does not exist is a projection of nothing.

**The second of those two issues landed, and it did not have to wait.** A projection whose source does not exist writes no file, and it states the reason for a reader of the run. What the work needed was a source to test against. A fixture tree supplies one that no reader can mistake for evidence about this corpus. The first link then landed too: the recorder is `tools/probe/probe-record.sh` and the transcript it committed is on the shelf. The remaining debt is neither link. It is a recording that a confirmation does not refuse.

**The contract that link satisfies is a document of this corpus.** [Spec 15](../spec/15-the-recorder-contract.md) states the twelve identity keys, the five event keys, the three tool-call keys, the four produced-artifact keys and the five confirmations. A test derives its four tables from the arrays the intake enforces. So a key this engine starts to refuse fails a test, rather than reaching a recorder as a refusal nobody wrote down. The process exists and it ran once. What the run met is the first confirmation of that contract.

One thing is settled here and needs no further work. The grader is a pure function of its three inputs, and two runs over one transcript write one set of bytes. That is what made the projection possible when the source arrived, and it is tested in the crate and again over the verb. The source arrived on 2026-09-09 and the function graded it to nothing, because a transcript the first confirmation refuses reaches no grader. What is missing is a usable source rather than the function.

This record does not ask for a result document written by hand. A hand-typed rate is the `asserted` warrant by another route, and it is the thing the second consequence above names.

## The first link landed and the debt did not close

`docs/probe-runs/regression-probe-transcript-for-2026-09-09.md` is a recorded transcript, `docs/probe-results/regression-probe-transcript-for-2026-09-09.md` is generated from it, and `generate --check` holds the pair. So the pair this record asks for exists, and the record stood at `discharged` on that reading from 2026-08-14 to 2026-09-11.

**The pair is not the measurement.** The commit that landed the transcript also moved `.headwater/taxonomy.lock`, so the `lock` the transcript pins stopped being the lock of this tree at the moment of the merge, the first of the five confirmations refused the recording whole, and the generated result carries zero verdicts of four. The two consequences above therefore both still stand: no document of this corpus holds a recomputed rate, and every efficacy rate here is `asserted`. That is why the status returns to `current`.

What closes this record is a transcript that a confirmation does not refuse, and [the transcript of 2026-09-11](../probe-runs/regression-probe-transcript-for-2026-09-11.md) is one. It stands at `current` rather than at `draft`, so a lock that moves under it fails the run instead of waiting for a reader. [The result](../probe-results/regression-probe-transcript-for-2026-09-11.md) is generated from it, `generate --check` holds the pair, and it states a rate of 1 of 4 over a denominator it names. So a document of this corpus now holds a recomputed rate, and this record discharges.

**The rate it holds is 1 of 4, and the first transform of those four logs made it 0 of 4.** `tools/probe/probe-record.sh` called the transform with no `--answer`, so the one probe that declares a closed answer set recorded `answer: null` whatever the session said, and a quarter of this corpus's only efficacy figure graded the recorder. The driver now derives the answer from the same harness line it already reads the cost from, and the result above is the same four logs read again. [#803](https://github.com/headwater-ai/headwater/issues/803) carries what is left of that defect.
