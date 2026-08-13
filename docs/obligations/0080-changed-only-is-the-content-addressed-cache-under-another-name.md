---
id: OBL-repo-0080
title: "`--changed-only` is the content-addressed cache under another name"
status: current
status_since: 2026-08-13
last_verified: 2026-08-13
summary: "Spec 6 declares a flag whose one job the cache already pays, and the engine ships no such flag."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - DR-repo-0006
---

# `--changed-only` is the content-addressed cache under another name

## Context

[Spec 6](../spec/06-engine-architecture.md#cli) gives the flag one job, which is to pay for a 200 ms commit hook without moving the verdict. The cache pays it already. Over this repository a warm run takes 30 ms and a run with no cache takes 300 ms. So nine tenths of a run is check evaluation, and a warm cache skips all of it.

The cache also derives what moved from content hashes rather than from a list that a caller supplies. A flag that took such a list would add an input to the verdict that no reviewer sees. Spec 6 forbids that in the same paragraph that declares the flag.

## Obligation

What no flag reaches is Phase A. The walk, the parse and the graph build are the whole of the warm 30 ms. To scope them is to cache the census and the graph, and [Q6](../spec/09-decisions.md#q6--where-the-corpus-graph-lives-at-rest) rules that nothing stores the graph. So the engine ships no `--changed-only` and spec 6 still declares one.

## Discharge

What reopens this is the corpus size at which Phase A alone passes 200 ms. This corpus reaches that at about 280 documents of the length it writes. Until then, the remedy is a sentence in spec 6 that withdraws the flag.
