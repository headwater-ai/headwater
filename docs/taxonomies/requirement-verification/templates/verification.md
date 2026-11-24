---
id: "{{ACME-VER-0001, minted by `headwater new verification` and never reissued}}"
status: draft
status_since: "{{today}}"
last_verified: "{{today}}"
summary: "{{one sentence a reader scans: what this checks, and what makes it fail}}"
title: "{{the verification, named for what it checks}}"
provenance:
  warrant: asserted
relations:
  proves:
    - "{{the identifier of the acceptance criterion this verification applies. The criterion names this one back under `proven_by`}}"
  cited_in:
    - "{{the path of the test file that cites this identifier in a comment, or delete this block. The edge binds once a comment in that file opens with the prefix the consumer declares as `anchors.test_site.pattern`}}"
---

# {{the verification, named for what it checks}}

## Approach

{{What the verification does and how: what it sets up, what it runs, and what it compares against the fit criterion. Name the file. A reader who opens the file finds this identifier in a comment there.}}

## What this does not cover

{{The edge of the claim: the conditions this verification does not set up, and the failures it would not see. A verification that states no limit claims more than it checks.}}
