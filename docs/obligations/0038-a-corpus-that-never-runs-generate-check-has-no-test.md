---
id: HW-OBL-0038
title: "A corpus that never runs `generate --check` has no test of a generated-file marker"
status: current
status_since: 2026-08-13
waiting_on: build
last_verified: 2026-08-13
summary: "The census excuses a marked file from every document check, and only `generate --check` tests that claim."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-engine-architecture
---

# A corpus that never runs `generate --check` has no test of a generated-file marker

## Context

The census excuses a marked file from every document check, and [`generate --check`](../spec/06-engine-architecture.md#projections) tests the claim. The two verbs are separate, so a corpus can run `headwater check` alone and never reach that test.

## Obligation

A person there adds one line to a document, and no run reports the document again. This repository runs both verbs in CI, so the gap belongs to the design rather than to this corpus.

## Discharge

A rule inside `headwater check` would close it, and the generator's dependency on the check layer is what stops one today.
