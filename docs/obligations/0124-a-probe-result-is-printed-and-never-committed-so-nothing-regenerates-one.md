---
id: OBL-repo-0124
status: current
status_since: 2026-08-14
summary: "The grader writes verdicts to standard output, no kind types a result, and the standing test spec 12 names cannot run."
last_verified: 2026-08-14
title: "A probe result is printed and never committed, so nothing regenerates one"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft
  evidence_basis: evidenced
relations:
  traces_to:
    - SPEC-HW-ai-integration
---

# A probe result is printed and never committed, so nothing regenerates one

## Context

[Spec 5](../spec/05-ai-integration.md#a-run-produces-a-snapshot-and-a-document) says what a probe result is: "The **probe result** is a document, generated from the transcript, the expectations and the grader version. It carries the `regenerated` warrant, and `generate --check` proves it." [Spec 12](../spec/12-check-layer.md#the-correctness-roots) reads the same sentence as an instrument, because the grader is a correctness root and a root owes its own test.

`headwater probe grade` now evaluates every expectation form and returns a verdict for each. It writes that report to standard output and it writes no document. Three facts were measured on this tree rather than argued:

- `docs/probe-runs/` is declared by the overlay and holds no file, so no transcript is corpus content.
- No kind of this taxonomy types a probe result, so a result document has nothing to be.
- `headwater generate --check` regenerates three documents, and each is a shelf index or a shelf section.

So the artifact that spec 5 calls a probe result exists as terminal output and as nothing else.

**The second and the third of those three are no longer true, and the first still is.** `probe_result` is a kind of this taxonomy, `docs/probe-results/**` is a shelf that holds it, and `probe_result` is the ninth declarable projection kind. The generator reads every committed transcript, grades it against the selection that `headwater probe plan` composes, and writes one document for each. A recorded fixture tree holds the pair and asserts the three directions of the gate. The committed pair is green. An unrelated edit leaves it green, and an edit to one event of the transcript turns it red.

This corpus writes no result, because the first fact holds. `headwater generate` prints the reason on every run rather than passing the declaration over, so the missing input has a location that a run states.

## Obligation

**The corpus owes a probe result as corpus content, and the debt is one link in a chain.** A result is a function of a transcript, and a recorder that observes a session from outside it writes that transcript. No verb of this engine writes one, and spec 5 refuses a transcript that an agent wrote about its own session. The chain is the recorder, then a committed transcript, then a kind for a result, then a generator over it, and only then the test.

Two consequences follow while the chain is unbuilt, and each is a claim this corpus cannot make yet.

**The standing test of a correctness root is half absent.** A recomputed number is what makes a result citable, and the recomputation is what catches a result that disagrees with its own transcript. The recorded fixture set of `headwater-probe` holds the grader against three transcripts and catches a defect in the engine. It says nothing about a number in a document, because no document holds one.

**A published rate would carry the `asserted` warrant.** [Spec 3](../spec/03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two) refuses an asserted document as evidence for anything. A rate written into this corpus today therefore discharges no evidence obligation. That is the failure the placement table in spec 5 exists to prevent, reached through the layer the table describes.

## Discharge

The issue that lands a recorder and a committed transcript discharges the first link. The issue that declares a `probe_result` kind and a generator over it discharges the rest. It cannot start earlier, because a projection whose source does not exist is a projection of nothing.

**The second of those two issues landed, and it did not have to wait.** A projection whose source does not exist writes no file, and it states the reason for a reader of the run. What the work needed was a source to test against. A fixture tree supplies one that no reader can mistake for evidence about this corpus. So the remaining debt is the first link alone: a recorder, and the transcript it commits.

One thing is settled here and needs no further work. The grader is a pure function of its three inputs, and two runs over one transcript write one set of bytes. That is what makes the projection possible when the source arrives, and it is tested in the crate and again over the verb. What is missing is the source rather than the function.

This record does not ask for a result document written by hand. A hand-typed rate is the `asserted` warrant by another route, and it is the thing the second consequence above names.
