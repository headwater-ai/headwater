---
id: HW-DR-0019
title: "Q19 — Inbound integration: an external system of record"
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: Imported content is `transcribed` against a committed pin, and an imported edge carries full weight from the first release.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-EVAL-warrant-and-adjudication
---

# Q19 — Inbound integration: an external system of record

## Context

One [evaluation](../evaluations/warrant-and-adjudication.md) settles this with [Q15](0015-a-synthesized-content-tier.md) and [Q18](0018-recording-adjudicated-disagreements.md). Four of the five open points close, and the first one dissolves.

**The tier question was Q15's question, and it has one answer.** Imported requirement text is not a fourth tier and not a qualifier on `generated`. It is `transcribed`, one of the four [warrant](../spec/01-conceptual-model.md#warrant) values, and W3C PROV already names it `prov:Quotation`. Q15 reached for the boundary and did not follow through. It is this: **is the content a function of a pinned input that the repository holds?** If it is, `generate --check` proves the equality on every run. If somebody edits, summarizes or merges it with local prose, no mechanism proves anything and the warrant is `asserted`. Regenerability was never a disqualifier from a tier. It is the tier axis.

## Decision

Imported content is `transcribed` against a committed pin, and an imported edge carries full weight from the first release.

**Snapshot format dissolves, and home closes.** [Spec 2](../spec/02-taxonomy-model.md#behavior-at-the-limits) already rules that exactly one resolver owns each anchor kind, and that a resolver reads repository content or a committed snapshot. So the shape of a snapshot is a property of the resolver, and the specification privileges neither ReqIF nor a vendor's API export. ReqIF exists so that two requirements tools can exchange a set without either owning the format, which is the adopter's asset rather than the engine's. What the specification does owe is three properties. The snapshot is committed inside the governed repository, because check time is a function of repository content and because a reviewer reads the diff. It carries its fetch time and the upstream identity and revision of every requirement. And it is a pin, so [spec 7](../spec/07-distribution-and-federation.md#upstream-awareness)'s one pattern with three instances covers it unchanged.

## Consequences

**Imported prose may enter, as a projection, and reference-first survives with a better reason.** A transcribed document is generated, held to regeneration against the pin, and marked as generated. A hand edit to it is the finding that a hand edit to a shelf index is. Truth stays upstream, which satisfies [principle 2](../spec/00-vision-and-scope.md#design-principles) rather than straining it. The leaning said that text "imports a maintenance obligation". The sharper reason is cost. Anchors and edges need only the anchor machinery, which exists. A transcription needs a resolver that reads text, a projection kind that writes it, and a comparison over the result.

**An imported edge is worth what any generated edge is worth, and the leaning was wrong.** It proposed that imported edges "start advisory and walk the same evidence-driven promotion path as every other control". [Principle 4](../spec/00-vision-and-scope.md#design-principles) promotes a **rule** against evidence about that rule's false-positive rate. An importer is not a rule. It is a producer of graph facts, and a wrong imported edge produces a correct check result over a wrong graph. No advisory posture ever finds that. The instrument is a fixture set over the importer, which is what [spec 12](../spec/12-check-layer.md#the-correctness-roots) already demands of the scaffolder for the identical reason. So an importer joins the correctness roots, an imported edge satisfies a participation expectation, and an imported pointer supports `evidenced`. A work item in a requirements tool is the clearest external auditable artifact that this specification has. `evidenced` never claimed that Headwater governs the artifact.

**Imported text does not leave, and the mechanism landed one group earlier.** [Q17](0017-governed-access-and-the-solution-layer.md) made the export filter default-deny over classes, and a transcribed document is a class. So an adopter carries imported prose to an audience only by naming it in a profile, and that is a line that a reviewer reads. What this ruling adds is a statement rather than a mechanism, and it sits with the other non-claims. Headwater checks nothing about a license. A profile that carries transcribed content is the adopter's redistribution decision ([spec 6](../spec/06-engine-architecture.md#what-a-filtered-export-claims-and-what-it-does-not)). Debian segregates by archive area for the same reason, and carries the rest by an explicit act.

**Drift is reported on the edge, not on the snapshot.** When a scheduled comparison advances the pin, the requirements whose revision changed are known. Every `traces_to` edge into one of them is a finding until a person re-verifies it. That is the suspect-link mechanism of requirements practice, and it is [spec 4](../spec/04-assurance-model.md#absence-is-a-finding-class-of-its-own)'s report-at-the-origin rule. A proposal against the whole snapshot names a file. A finding on an edge names the document whose author can act.
