---
id: SPEC-FIX-the-second-part
status: current
status_since: 2026-08-01
summary: The second part, which already declares a relations block that names no inverse of a scaffolded edge.
doc_type: design_spec
sequence: 2
relations:
  traces_to:
    - SPEC-FIX-the-first-part
  supersedes:
    - SPEC-FIX-the-third-part
---

# 2 — The second part

## Scope

A splice into this document adds a key to a block that is already there.

## Behavior

It declares one edge under a relation that is not the one a successor writes.
