---
id: OBL-repo-0028
title: "A run cannot report the corpus tree, because nothing computes one"
status: current
status_since: 2026-08-13
last_verified: 2026-08-14
summary: "Spec 6 asks every run for a corpus tree beside its findings, and the engine computes none."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - SPEC-HW-engine-architecture
---

# A run cannot report the corpus tree, because nothing computes one

## Context

[Spec 6](../spec/06-engine-architecture.md#ci-adapters) asks every run to report three things beside its findings: the corpus tree, the taxonomy lock hash, and the read set. The engine reports the last two and computes no tree.

The read set is not one, because it holds what the checks read and a tree holds what the census walked.

## Obligation

A run therefore states nothing about a file that no check opened, which is the exact set a merge can add. The corpus owes a tree that covers every row of the census.

## Discharge

SARIF has the member for a tree, `run.automationDetails.id`, and the adapter leaves it out rather than print the read set's identity there. The cost grows with the corpus. A wider excluded set leaves a gate less of the evaluated state to hold a later tree against.

**The check layer asks for no tree, and this record stands on spec 6 alone.** [Spec 12](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) fixes the gate as a comparison over listed inputs, which needs neither a tree nor a run. So the test that decides whether a verdict survives a merge does not wait on this record. [OBL-repo-0102](0102-two-documents-state-what-voids-a-verdict-and-they-do-not.md) carries that ruling and the construction under it.
