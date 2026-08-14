---
id: DR-SWP-0002
doc_type: decision_record
status: current
status_since: 2026-03-11
summary: The cache is an implementation detail, so no operator setting reaches it and no run may skip it.
---

# The cache is not an operator setting

## Context

Two runs over one tree have to write the same bytes. A setting that changes what a run evaluates is a setting that can make two runs disagree.

## Decision

No operator setting reaches the cache. Every run reads it and every run writes it, and there is no flag that turns it off.
