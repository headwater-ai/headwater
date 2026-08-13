---
id: OBL-repo-0004
title: "Working-tree write tools have no measured effect on the assisted fraction"
status: current
status_since: 2026-08-10
last_verified: 2026-08-13
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

The unassisted baseline that such a comparison needs is the typing pass that [Q4's entry](0001-the-promotion-fix-has-no-reading-of-the-assisted-fraction.md) records. The treatment is absent, because no write tool has authored anything in this repository.
