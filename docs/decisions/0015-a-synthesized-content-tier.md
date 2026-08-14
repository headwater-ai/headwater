---
id: DR-repo-0015
title: Q15 — A synthesized content tier
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: Every document carries a warrant from a closed set of four, and `asserted` content is admitted with limits.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - EVAL-HW-warrant-and-adjudication
---

# Q15 — A synthesized content tier

## Context

One [evaluation](../evaluations/warrant-and-adjudication.md) settles this with [Q19](0019-inbound-integration-an-external-system-of-record.md) and [Q18](0018-recording-adjudicated-disagreements.md). All three ask what a provenance record must carry for content that the corpus did not author, and what such content may then do.

**The open-question entry named the wrong field, and its own text says so without following through.** It says that "the boundary that this question draws is a verification method, not an author". It then argues about the author for the rest of its length. Agency was never the boundary. [Spec 3](../spec/03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed) already admits an agent-drafted document in full: `agency: agent`, `drafted_by`, and a human in `accepted_by`. What the entry actually names is content that **nobody accepted**, at a volume where acceptance does not scale.

## Decision

Every document carries a **warrant**: the mechanism by which the corpus can defend that the document is what it claims to be ([spec 1](../spec/01-conceptual-model.md#warrant)). The set is closed and it has four values.

| Warrant | What stands behind the content |
|---|---|
| `accepted` | A named human accepted it |
| `regenerated` | It is a function of inputs inside the repository, and `generate --check` proves it |
| `transcribed` | It is a byte-faithful copy of a pinned external snapshot ([Q19](0019-inbound-integration-an-external-system-of-record.md)) |
| `asserted` | Nothing |

**Yes, Headwater admits `asserted` content.** The mark is positive and never an absent field, which is what SPDX settled when it made `NOASSERTION` a value beside `NONE`. Wikipedia is the observed case at the largest available scale. Its 2025 speedy-deletion criterion for machine output fires on the absence of review, and not on the presence of a model ([spec 11 §P](../spec/11-adjacent-work.md#p--provenance-endorsement-and-the-record-of-a-judgment)).

## Consequences

**What it may not do is derived, and the leaning's list is replaced.** Two rules cover the cases that "never a valid target for `governs` or `verifies`" enumerated, plus every relation that an adopter adds later.

- **No edge may let unwarranted content govern the reading of warranted content.** [Reading precedence](../spec/02-taxonomy-model.md#reading-precedence-is-derived) is already derived from nuclearity, succession and the governance family, so no per-relation list is needed. Anchors carry no reading precedence, so an asserted document may still declare `governs` against a code path. That edge is what gives it write-time impact detection, which is the only thing that will report its decay.
- **An asserted document does not discharge an evidence obligation.** `evidenced` obliges an external, auditable artifact, and an asserted document is neither ([spec 3](../spec/03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two)).

**Staleness does not apply, and there is no date to bump.** `last_verified` records that a human confirmed a document. An asserted document has none and cannot acquire one without becoming `accepted`. Its decay is reported as drift from its production date, and the remedy is to regenerate or to delete.

**Promotion is acceptance, and nothing new is built.** A person reads the document, sets the warrant, and names themselves. Bulk stamping produces identical bytes, so no mechanism detects it. That posture is advisory permanently, for the reason that the shift ratio is.

**The instrument is sited differently from the way this record proposed, and the build is what found that.** This record said that `taxonomy audit` reports promotions per change. That verb reads one working tree, and a promotion is a lifecycle transition whose count needs the prior version of each changed document. [Spec 12](../spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version) makes that input available only in change-scoped evaluation, and it names `needs_prior` as the declaration that asks for it. `warrant.promoted` declares it, and `headwater check --change` is where a caller supplies the change. So the reading belongs to a change-scoped check rather than to this verb, and it is built there. What `taxonomy audit` reports is the standing `asserted` population, which is the denominator and not the rate. [Spec 13](../spec/13-open-obligations.md#what-else-each-decision-left-open) carries what that leaves the decision owing.

**The Q17 constraint changed the answer rather than confirming it.** The open-question entry proposed that an emitter which cannot carry the mark declares that in its loss set. A loss-set entry reaches the consumer who reads it and nobody else. C2PA's own threat model records that ordinary tooling strips a mark that travels beside content. So an emitter that cannot carry the warrant **withholds** the content at the profile's declared tombstone grain ([spec 6](../spec/06-engine-architecture.md#an-export-is-a-projection-and-it-declares-what-it-dropped)). That costs nothing today, because the native export carries the warrant with no loss.
