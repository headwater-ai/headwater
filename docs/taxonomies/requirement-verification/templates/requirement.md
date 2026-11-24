---
id: "{{ACME-REQ-0001, minted by `headwater new requirement` and never reissued}}"
status: draft
status_since: "{{today}}"
last_verified: "{{today}}"
summary: "{{one sentence a reader scans: what must hold, and for whom}}"
title: "{{the requirement, stated as what must hold}}"
provenance:
  warrant: asserted
relations:
  verified_by:
    - "{{the identifier of an acceptance criterion that settles this requirement, or a check rule identifier, or delete this block. `headwater new acceptance_criterion --relates verifies=<this id>` writes the criterion, and `headwater check --fix` writes this half once the criterion leaves `draft`}}"
---

# {{the requirement, stated as what must hold}}

## Context

{{Why this requirement exists: the issue, the decision or the person that asked for it, and what goes wrong without it. Name the source so that a reader can ask it.}}

## Requirement

{{What must hold, in one or a few sentences that a test or a reviewer can fail. Write one requirement per document, because the identity of a requirement is the identity of its document. A second requirement under this heading has no identifier of its own.}}

## Verification

{{How a reader will know that this holds: name each acceptance criterion by its identifier and say what it measures. A requirement that nothing verifies is reported after 90 days at `current`.}}
