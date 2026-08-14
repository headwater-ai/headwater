---
id: DR-SWP-0005
doc_type: decision_record
status: current
status_since: 2026-02-09
summary: A run reports findings and keeps no account of its inputs, so no later tree can be held against it.
---

# A run records nothing about what it read

## Context

A read set is a second artifact to keep in step with the run that produced it.

## Decision

A run reports its findings and records nothing about its inputs. The document that declares the conflict with this one is DR-SWP-0004, and the declaration sits at that end.
