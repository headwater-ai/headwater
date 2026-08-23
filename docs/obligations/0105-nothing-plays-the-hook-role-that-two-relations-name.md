---
id: HW-OBL-0105
title: "Nothing plays the hook role that two relations name as their author"
status: current
status_since: 2026-08-13
waiting_on: ruling
last_verified: 2026-08-13
summary: "`governs` and `traces_to` declare `created_by: hook`, no verb of this engine writes either one, and every such edge in this corpus is hand entry."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-taxonomy-model
    - HW-SPEC-ai-integration
    - HW-EVAL-default-taxonomy-first-run
---

# Nothing plays the hook role that two relations name as their author

## Context

`created_by` is required on every relation, and its value names the actor that a taxonomy expects to pay for the edge. The base package gives two of its five relations the value `hook`. They are `governs`, which ends on a `code_path` anchor, and `traces_to`, which ends on a document or on an anchor.

[Spec 2](../spec/02-taxonomy-model.md#the-decision-relation-vocabulary) states the expectation as a fact about the engine. Its sentence is that a hook proposes `traces_to` and `governs` from the change. The [first-run walkthrough](../evaluations/default-taxonomy-first-run.md) counts how many default relations lack a mechanical creator, reaches two, and names `conflicts_with` and `constrains`. It counts `governs` and `traces_to` on the other side of that line.

No verb of this engine writes either edge. `headwater new --relates` refuses a relation whose `created_by` is not `scaffold`. It then prints both relations in the block that names what a document may also declare and nobody did. `headwater check --fix` writes a reciprocal half of an edge that a document already declares, and it proposes none. The three harness hooks that [#72](https://github.com/headwater-ai/headwater/issues/72) shipped read the graph and write no front matter at all.

The corpus reports the result. It holds seven edges onto a `code_path` anchor, and a person typed every one of them into front matter by hand. The assisted fraction over the two relations is zero, and no run can raise it.

## Obligation

The corpus owes either the actor or the correction.

The claim that these two relations have a mechanical creator carries weight that no reader can check. The walkthrough uses it to argue that the base taxonomy is a design rather than a bet on author diligence. [Spec 10](../spec/10-theoretical-foundations.md#b5-traceability-information-models--our-idea-has-a-name-and-a-literature) says that author-maintained links decay because the payer is not the beneficiary. Under today's engine both relations sit on the decaying side, and one document says otherwise.

Which document governs which code is a judgment. The [stop rules](../spec/05-ai-integration.md#the-stop-rules) forbid an agent that invents structure. So a deterministic hook cannot read a diff and decide that a specification governs a file. That is what makes this a ruling rather than a task, and the honest actor may be `agent` rather than `hook`. The closed set holds both.

`traces_to` is the easier half and it is not free either. A commit that cites an identifier supplies a candidate edge, and nothing reads a commit message today.

## Discharge

A verb that proposes these edges from a change, with the fixtures that hold what it refuses to propose. The proposal reaches a human, because [spec 3](../spec/03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed) makes acceptance a human act.

Where no such verb is written, the base package owes a `created_by` value that is true, and the walkthrough owes the corrected count. `author` states the cost plainly, and `agent` states that the coherence sweep owns the proposal.

[HW-OBL-0104](0104-a-governs-edge-reaches-the-path-it-names-and-nothing.md) raises what either answer costs. A `governs` edge reaches one path, so an actor that proposes them proposes one for every governed file.
