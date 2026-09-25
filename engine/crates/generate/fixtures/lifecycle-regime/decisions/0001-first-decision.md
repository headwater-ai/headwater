---
id: DR-FIX-0001
title: First decision
status: current
status_since: 2026-01-01
last_verified: 2026-02-01
summary: the first document on the `decisions` shelf, and the only one that declares an edge, read as a source of the four identity-bearing declarations that this fixture is written to exercise.
relations:
  flags:
    - ED-FIX-edge
  flags_bogus:
    - BG-FIX-bogus
---

# First decision

The first document on the `decisions` shelf of this fixture tree, and the only one that declares an edge. `flags` and `flags_bogus` are the incoming edges the two edge-locus cases read: neither target exists as a file yet, and the raw target is what `derived.rs` reads for exactly that reason.
