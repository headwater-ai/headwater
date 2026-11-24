---
id: "{{ACME-AC-0001, minted by `headwater new acceptance_criterion` and never reissued}}"
status: draft
status_since: "{{today}}"
last_verified: "{{today}}"
summary: "{{one sentence a reader scans: the measure, with its number}}"
title: "{{the criterion, stated as the result that passes}}"
verification_method: "{{test | demonstration | inspection | analysis}}"
provenance:
  warrant: asserted
relations:
  verifies:
    - "{{the identifier of the requirement this criterion settles. The requirement names this one back under `verified_by`}}"
  proven_by:
    - "{{the identifier of the verification that applies this criterion, or delete this block. A criterion whose method is `test` and that no verification reaches is reported after 90 days at `current`}}"
---

# {{the criterion, stated as the result that passes}}

## Fit criterion

{{The measure that settles the requirement, with its number and its unit. Write it so that two readers who apply it to one result reach one verdict. "Fast" is not a fit criterion, and "within 60 seconds over 20 runs" is one.}}

## Method

{{How the measure is applied: what runs, what a reviewer reads, or what model the analysis uses. The method agrees with `verification_method`. Name the test file or the procedure, so that a reader can apply it again.}}
