---
id: HW-DR-0007
title: Q7 — Scope of the MCP surface
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: Three classes of tool, and a landed write never ships.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-EVAL-the-serving-boundary
---

# Q7 — Scope of the MCP surface

## Context

One [evaluation](../evaluations/the-serving-boundary.md) settles this with [Q14](0014-discovery-surface.md) and [Q17](0017-governed-access-and-the-solution-layer.md). All three are about one boundary, where a corpus meets a reader that it does not control.

**The open-question entry's axis is wrong, and that is why the leaning came out half right.** It asks whether an agent may write, and calls that "a question of trust and workflow". `headwater check --fix` writes files today and nobody calls it a write surface, because the result lands in a diff that a human commits. The axis is **whose review the result passes through**, not whether bytes move.

**The specification had already answered the commit half, and the entry did not cite it.** [Spec 3](../spec/03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed) requires `accepted_by` and states that acceptance is a human act. [Spec 5](../spec/05-ai-integration.md#what-we-do-not-do) forbids an agent-authored document merged without review. A server-side commit produces a document with no `accepted_by`, or an invented one. No judgment about trust is needed to reach that.

## Decision

Three classes of tool, and the boundary between the second and the third is where the whole question lived ([spec 5](../spec/05-ai-integration.md#what-the-server-may-do-and-the-axis-that-decides-it)).

| Class | Ships | Why |
|---|---|---|
| Query | first release | It changes nothing |
| Working-tree write (`new`, `fix`) | first release, off by default per server | The human reviews at commit, and the [fixability bar](../spec/12-check-layer.md#fixability) forbids a judgment-bearing patch |
| Landed write | never | Acceptance is a human act, and no forge is privileged in the core |

## Consequences

**So the third row is a refusal and not a deferral.** "Writes arrive later" describes a thing that never arrives. Headwater emits what a change proposal needs, and an adapter opens the proposal ([spec 7](../spec/07-distribution-and-federation.md#upstream-awareness)). That is the boundary that keeps the engine out of the merge-queue business ([Q21](0021-terminological-succession-and-validity-under-merge.md)).

**And the second row corrects the leaning the other way.** A server that only reads would ship the agent surface without the authoring half. [Spec 0](../spec/00-vision-and-scope.md#what-we-build) puts that half in the same release, for a stated reason. The two working-tree tools are the mechanical, total operations that the fixability bar already admits.

**Three findings from the prior art change how this is stated** ([spec 11 §O](../spec/11-adjacent-work.md#o--the-serving-boundary-descriptors-redaction-and-the-write-path)). The protocol's own annotations are hints, and a client must not trust one from an untrusted server. So the enforcement is an unregistered tool and never a flag. The published attacks arrive at discovery time, before any call, so a confirmation prompt at each call is not a safety argument. And an observed exploit used a write tool as an exfiltration channel, which makes this record an access-control ruling as well as a workflow one.
