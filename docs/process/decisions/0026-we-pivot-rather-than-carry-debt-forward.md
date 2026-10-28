---
id: HW-PD-0026
status: current
status_since: 2026-10-28
summary: "When a better structure costs a comparison with earlier runs, a baseline or a probe set, we take the loss and undo the old work. A measured result stays as a dated record. The 0.07 re-cut is the first application."
last_verified: 2026-10-28
title: "We pivot rather than carry debt forward"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: draft
relations:
  traces_to:
    - HW-DR-0111
---

# We pivot rather than carry debt forward

## Context

The additive campaign of [HW-DR-0111](../../decisions/0111-the-additive-campaign-adds-six-layers-to-plain-code-in-fourteen-scenarios-and-registers-one-comparison-each.md) measured its layers in fourteen scenarios. Its tuning round closed under [#1777](https://github.com/headwater-ai/headwater/issues/1777). The re-cut of milestone 0.07, [#1868](https://github.com/headwater-ai/headwater/issues/1868), replaces that design with eleven actions, five layers and seven scenarios. A result from the new design does not compare with a baseline from the old one. So the re-cut had to choose: keep the old design for the comparison, or take the loss.

The owner stated the principle below on 2026-10-28, during the re-cut, and approved this wording on 2026-10-29 ([#1869](https://github.com/headwater-ai/headwater/issues/1869)). Before this record, no document in `DEVELOPING.md` or under `docs/process/` stated a rule about a structure that a better one replaces.

## Decision

The owner's words, 2026-10-28, approved 2026-10-29:

> We pivot rather than carry debt forward. When a better structure costs us something we already have, such as the ability to compare with earlier runs, a baseline or a probe set, we take that loss and undo the work. What was measured stays as a dated record; what was built is kept only while it is the best structure we know.

## Consequences

The first application is the 0.07 re-cut, [#1868](https://github.com/headwater-ai/headwater/issues/1868). It retires the fourteen-scenario baselines of HW-DR-0111 and starts new ones. The results of the fourteen-scenario campaign stay on the corpus as dated records. A separate decision of #1868 supersedes the scenario design of HW-DR-0111, and this record does not.

So the loss of a comparison, a baseline or a probe set is not, by itself, a reason to keep an old structure. A result measured before a pivot is a dated record, and it is not a baseline for a result measured after the pivot.
