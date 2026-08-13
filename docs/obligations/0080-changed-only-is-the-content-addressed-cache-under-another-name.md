---
id: OBL-repo-0080
title: "`--changed-only` is the content-addressed cache under another name"
status: current
status_since: 2026-08-13
last_verified: 2026-08-14
summary: "The one job a change-scoped flag has is a job the cache already pays, and Phase A is what no flag reaches."
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

[Spec 6](../spec/06-engine-architecture.md#cli) states that there is no `--changed-only`, and this record holds the measurement behind that sentence. The one job such a flag has is to pay for a 200 ms commit hook without moving the verdict. The cache pays it.

**The measurement, and the conditions it was taken under.** Over this repository a warm `headwater check` costs 56 to 61 ms, and the same run with `--no-cache` costs 470 to 486 ms. Seven runs of each, on a release build, on an eight-core host, against 201 files under `docs/` and 158 classified documents and 2,442 check instances. So check evaluation is about nine tenths of a cold run, and a warm cache serves all of it. A timing with no host, no build profile and no corpus size beside it cannot be re-derived, which is why this paragraph carries all three.

The cache also derives what moved from content hashes rather than from a list that a caller supplies. A flag that took such a list would add an input to the verdict that no reviewer sees. Spec 6 forbids that in the paragraph that now states there is no flag.

## Obligation

What no flag reaches is Phase A. `headwater explain` walks the corpus, parses it, builds the graph and runs no check. It costs 38 to 45 ms on the same corpus and the same build. So Phase A is about seven tenths of the warm run and about a twelfth of the cold one. To scope it is to cache the census and the graph, and [Q6](../spec/09-decisions.md#q6--where-the-corpus-graph-lives-at-rest) rules that nothing stores the graph. [OBL-repo-0072](0072-a-cache-of-check-results-does-not-make-a-run-proportional.md) carries what that leaves open in spec 6's own promise.

## Discharge

What reopens this is a parse cache, and no flag at any corpus size. A cache of one parse per file is disposable, derived, and verdict-neutral under the `--no-cache` byte-identity differential that already runs on every gate. Q6 forbids a canonical stored graph, and it does not forbid such a cache.

The trigger to build one is Phase A alone passing the 200 ms hook budget. Phase A costs about 40 ms over the 158 documents that this corpus classifies. A linear reading puts the trigger near 800 documents of the length this corpus writes. One corpus supports that reading, so what reopens this record is the measurement rather than the document count.
