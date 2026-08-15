---
id: HW-OBL-0003
title: "The rebuild-and-cache design is measured at spike scale alone"
status: current
status_since: 2026-08-10
last_verified: 2026-08-13
summary: "Q6 rules that the graph never rests, and the only evidence is a spike over generated documents."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0006
---

# The rebuild-and-cache design is measured at spike scale alone

## Context

[Q6](../spec/09-decisions.md#q6--where-the-corpus-graph-lives-at-rest) rules that every run rebuilds the graph and that no derived artifact is canonical. The rebuild-and-cache design is measured at spike scale, on generated documents.

## Obligation

A real corpus and a real harvesting tier are the instruments, and this corpus owes a reading from each.

## Discharge

Neither instrument exists yet. No adopter has brought a corpus of the size the claim is about, and no harvesting tier has been built.
