---
id: DR-SWP-0001
doc_type: decision_record
status: current
status_since: 2026-01-05
summary: An operator may turn the cache off for one run, and the run reports that it did.
---

# An operator may disable the cache

## Context

The cache is keyed on the lock digest and on the content hash of every document a rule read. An operator who suspects the key is wrong needs a way to evaluate every instance again.

## Decision

An operator may disable the cache for one run. The run then evaluates every instance and reports that it read no cache.
