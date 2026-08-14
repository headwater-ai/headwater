---
id: OBL-repo-0004
title: "Working-tree write tools have no measured effect on the assisted fraction"
status: current
status_since: 2026-08-10
last_verified: 2026-08-14
summary: "Q7 claims that write tools raise the assisted fraction, and the treatment has never been applied."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - DR-repo-0007
---

# Working-tree write tools have no measured effect on the assisted fraction

## Context

[Q7](../spec/09-decisions.md#q7--scope-of-the-mcp-surface) rules three classes of tool and refuses a landed write. Working-tree write tools should raise the assisted fraction, which [spec 3](../spec/03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric) declares as a tracked metric.

## Obligation

That metric is the instrument, and a comparison needs a baseline and a treatment.

## Discharge

The unassisted baseline that such a comparison needs is the typing pass that [Q4's entry](0001-the-promotion-fix-has-no-reading-of-the-assisted-fraction.md) records.

**The treatment arm exists and it holds one reading.** `headwater mcp --write` registers the `new` tool, and the run that wrote [OBL-repo-0111](0111-the-capture-cost-surface-names-an-entry-point-and-never-the-caller.md) on 2026-08-14 is the first authoring this repository has done over a protocol. The store names the arm of every reading it takes: `.headwater/capture-cost.jsonl` carries a `surface` term, and `headwater capture` groups by it. A line the store took before that day names no arm, and the report counts such a line under neither.

**The instrument is not the one this record assumed.** The four counts of a reading are derived from the plan, and a plan is the same plan whatever asked for it. So the per-run fraction cannot separate the arms at all. [OBL-repo-0112](0112-a-surface-cannot-move-the-assisted-fraction-of-a-run.md) measures that: two readings of one kind, one from each arm, carry the same four pairs. What a surface can move is the count of runs and the mix of kinds those runs chose. This corpus holds five readings, three of which name an arm, so the comparison is still unmade.

**And a reading of the arms says less than the two names suggest.** The term records an entry point rather than a caller, and one agent produced every reading in both arms on the day the term arrived. OBL-repo-0111 holds that.
