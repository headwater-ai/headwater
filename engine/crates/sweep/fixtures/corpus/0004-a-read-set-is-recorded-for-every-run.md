---
id: DR-SWP-0004
doc_type: decision_record
status: current
status_since: 2026-02-02
summary: Every run records what it read, so a later tree can be held against the verdicts of an earlier one.
relations:
  conflicts_with:
    - DR-SWP-0005
---

# A read set is recorded for every run

## Context

A verdict is about a tree. A tree that moved under the verdict makes the verdict a claim about bytes nobody holds.

## Decision

Every run records the content hash of every document and every edge it read. The declared conflict with DR-SWP-0005 is in the front matter above, and it is what makes a sweep finding about this pair a restatement.
