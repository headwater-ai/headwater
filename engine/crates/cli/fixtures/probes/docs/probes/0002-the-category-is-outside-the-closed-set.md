---
id: HW-PROBE-the-category-is-outside-the-closed-set
status: current
status_since: 2026-08-01
last_verified: 2026-08-01
summary: A probe whose category is outside the closed set, which stops the plan after the probe above it was read.
probe_category: provenance
expectation: opened
oracle: "none"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: a-model
  evidence_basis: unevidenced
relations:
  examines:
    - HW-PROBE-the-session-reads-the-document
---

# The category is outside the closed set

## Task

Do the thing the document below governs.

## Expectation

`opened` over the document this probe names. Nothing here is ever read: `probe_category` holds `provenance`, which the closed set does not carry, so the plan stops on this file and reports the reason.
