---
id: HW-DR-0061
status: current
status_since: 2026-09-07
summary: "Adjudication runs as its own stage before construction and its prompt licenses refusal, because a run that separated them returned a refusal or a correction from six of eight slots, and that costs one parent turn per issue which is paid."
last_verified: 2026-09-07
title: "Adjudication is a separate stage, and refusal is licensed"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-fable-5-1
  activity: measure+draft
  evidence_basis: evidenced
---

# Adjudication is a separate stage, and refusal is licensed

## Context

A design for the build order fused adjudication into construction as one agent, to retire one parent turn per issue. Under [HW-DR-0062](0062-a-dispatch-pays-when-it-retires-more-parent-turns-than-it-costs.md) a parent turn is the unit of cost, so a stage boundary that can go should go, and the fused shape took three turns per issue against four.

Run 24 measured what the boundary is for. Six of eight slots returned something other than "build as specified". Three rejected part of their own Done-when. One found a pre-existing defect that permanently bricks an output path, reproducible at one run in three with its own change stashed. The owner of that run reads this as a property of the boundary and not of the agents. An agent handed a prescribed remedy implements it and cannot find the error in it. An agent that first settles whether the premise holds can. [The evaluation](../evaluations/the-build-order-as-a-multi-agent-system.md) carries the figures.

The choice is a measured trade. One turn per issue, about 45 per run, against the highest-value output the run produced.

## Decision

Adjudication is a separate stage with its own agent definition, and it runs before construction on every issue. Its prompt licenses refusal in words. The agent may report that the issue's premise is stale, that its Done-when names the wrong thing, or that the change is not worth building. Each of those is a completion rather than a failure. Its report is a fixed block that the parent acts on without reading prose: the verdict, what it builds, the artifact footprint the change regenerates, and the decisive fixture.

The one parent turn this costs over the fused shape is paid. It is not recovered by folding the stages back together on a later run without a measurement that shows the refusals stopped appearing.

## Consequences

The parent takes four turns per issue: adjudication done, construction done, verification done, integration done. The verification-done turn dispatches integration and the next adjudication together, so no turn is spent only on dispatch.

The adjudicator declares the footprint, which is what [HW-DR-0063](0063-coordination-is-a-create-only-claim-and-authority-stays-on-the-tree.md) claims on the parent's next turn. Construction reads the adjudication note from disk and never re-derives it.

A run whose adjudicators return "build as specified" on every slot for a whole run is evidence that the boundary has stopped paying, and that is the condition under which this record is reopened. Until a run measures that, the boundary stands.
