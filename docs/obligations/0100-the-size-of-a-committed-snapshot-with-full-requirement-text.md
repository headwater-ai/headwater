---
id: OBL-repo-0100
title: "The size of a committed snapshot with full requirement text"
status: current
status_since: 2026-08-11
last_verified: 2026-08-13
summary: "Q19 pins an external system of record, and nobody has measured what a snapshot with full requirement text costs."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - DR-repo-0019
    - DR-repo-0009
---

# The size of a committed snapshot with full requirement text

## Context

[Q19](../spec/09-decisions.md#q19--inbound-integration-an-external-system-of-record) rules that a pointer resolves offline against a committed snapshot.

## Obligation

The corpus owes the size of a committed snapshot that carries full requirement text.

## Discharge

That is the same question that [Q9](../spec/09-decisions.md#q9--multi-repository-corpora) holds about a vendored source export, and no importer has run to answer either.
