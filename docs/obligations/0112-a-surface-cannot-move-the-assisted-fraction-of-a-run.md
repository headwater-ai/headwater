---
id: HW-OBL-0112
title: "A surface cannot move the assisted fraction of a run"
status: current
status_since: 2026-08-14
waiting_on: measurement
last_verified: 2026-08-14
summary: "The four counts of a reading are derived from the plan, and a plan is the same plan whatever asked for it, so Q7's comparison has to read reach rather than the fraction."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0007
    - engine/crates/scaffold/src/reading.rs
---

# A surface cannot move the assisted fraction of a run

## Context

[Q7](../spec/09-decisions.md#q7--scope-of-the-mcp-surface) claims that a working-tree write tool raises the assisted fraction. [HW-OBL-0004](0004-working-tree-write-tools-have-no-measured-effect.md) makes that claim a comparison of a baseline arm and a treatment arm. The capture-cost store now names the arm of every reading, and `headwater capture` groups by it.

## Obligation

The four counts of a reading are derived from the plan. The scaffolder proposes a plan from a kind, a title, a date and the relations a caller named. The surface is not one of those inputs, and no field of a plan carries one.

So two runs of one kind, with the same relations, write the same four counts whatever asked for them. A comparison of the per-run fraction across two arms therefore measures nothing about either arm. What a surface can move is the count of runs, which is the reach figure, and the mix of kinds that those runs chose. It cannot move the number inside one run.

## Discharge

Measured on 2026-08-14 over this repository. Two readings of one kind, one from each arm, carry the same four pairs. Front matter is 4 of 5, sections 3 of 3, identifier 1 of 1, and edge halves 0 of 0. Each run is 8 of 9. The two lines of the store differ in `surface`, in `document` and in `id`, and in nothing else.

The comparison that HW-OBL-0004 asks for therefore reads one of two other things. The first is reach, which is a count over runs that happened. The report already states it. The second is the aggregate fraction over a mix of kinds that the two surfaces reached for differently. That second reading is a claim about what an author picks. It needs many runs and a stated population. This corpus holds five readings, three of which name an arm, so neither is available.
