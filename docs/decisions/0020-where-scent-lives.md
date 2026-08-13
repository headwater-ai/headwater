---
id: DR-repo-0020
title: Q20 — Where scent lives
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: An optional source-owned cue sits on a relation instance, graded against the alternatives in view at the point of decision.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - EVAL-HW-the-measurement-layer
---

# Q20 — Where scent lives

## Context

One [evaluation](../evaluations/the-measurement-layer.md) settles this with [Q8](0008-probe-cost-and-cadence.md), because the standing half of the open-question entry asks how anything grades a cue and the grader is a probe.

**The premise of the open half is false, and foraging theory says so.** The entry concluded that spec 5 grades distinctiveness by a comparison of siblings in one place, and that edge cues have no such place. The computational foraging models score a link decision as a utility over the links available at the reader's current position ([spec 11 §R.1](../spec/11-adjacent-work.md#r1-foraging-models-compute-a-utility-over-the-options-in-view)). Scent is never absolute. It is a comparison over the options at the point of decision.

**So one rule grades both placements.** Grade a cue against the alternatives that it competes with at the moment a reader reads it. For a summary that set is the documents a reader is choosing between. The shelf is the static proxy for it, and the pointer list is the real thing. For a cue on an edge it is **the other outbound cues of the document in hand**, which is the choice that a traversal presents. The rule corrects the node side too, because spec 5 never named the set that a summary is compared against.

## Decision

An optional source-owned cue on a relation instance, with the summary still required and absence falling back to the target's summary. [Q4](0004-relation-storage.md) settled where it lives and who writes it, and [spec 2](../spec/02-taxonomy-model.md#instance-attributes-and-which-end-owns-each-one) already declares it on `cites`.

## Consequences

**Three static checks follow, and no new machinery.** Non-restatement is `Edge`-scoped, which [spec 12](../spec/12-check-layer.md#scope--the-declaration-everything-else-rests-on) already defines as one relation instance and both endpoints. Distinctiveness over cues is `Document`-scoped. The length band is trivial. All three stay advisory permanently under the [fixability bar](../spec/12-check-layer.md#fixability), because the remedy for a weak cue is a rewrite. A corpus that declares no cue has no instances and pays nothing.

**Two behavioral measures come from transcripts that already exist.** Traversal precision and traversal abandonment are the edge analogs of routing precision and abandonment. A traversal is visible in a tool-call transcript, as opening one document and then opening a target that it declares an edge to.

**Four serving rules, and three of them follow from existing rulings.** `related` and `explain` serve the cue and routing never does, because a routing result has no referring edge. A cue states the [warrant](../spec/01-conceptual-model.md#warrant) of an `asserted` target, as a pointer does. The confidence gate does not reach a cue, because an author writes it rather than the engine scoring it. Silence is also not available to a reader who can already see the edge. A withheld target takes its cues with it, because the export filter is default-deny over classes.

**The cue's counterfactual is the cheapest in the specification.** Absence falls back to the target's summary, so the ablation is a switch in the serving layer rather than a second corpus. The leaning said to grade the cue only where a probe records a failed traversal. That is close and one step short, because a failure in the present arm alone has no baseline to compare against.

**One consequence arrives from outside, and it is unchanged.** A cue is an instance attribute, so an edge that carries one reifies in any RDF export. It appears in that emitter's declared loss set ([Q6](0006-where-the-corpus-graph-lives-at-rest.md)). The native graph export carries a cue with no loss.
