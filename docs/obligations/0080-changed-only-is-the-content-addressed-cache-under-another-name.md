---
id: HW-OBL-0080
title: "`--changed-only` is the content-addressed cache under another name"
status: current
status_since: 2026-08-13
waiting_on: measurement
last_verified: 2026-10-01
summary: "The warm cache keeps the commit hook inside its 200 ms budget, and a change-scoped flag stays refused because it hides a verdict input."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0006
---

# `--changed-only` is the content-addressed cache under another name

## Context

[Spec 6](../spec/06-engine-architecture.md#cli) states that there is no `--changed-only`, and this record holds the measurement behind that sentence. The one job such a flag has is to pay for a 200 ms commit hook without moving the verdict. In 2026-08 the cache paid it. At 477 and at 489 typed documents it did not, and the commit hook went past its target. At 500 typed documents, after [#1450](https://github.com/headwater-ai/headwater/issues/1450), it pays it again.

**The measurement, and the conditions it was taken under.** On 2026-10-01, over this repository at 500 typed documents, a warm `headwater check --strict` cost 177 ms of user and system CPU time. The same run with `--no-cache` cost 523 ms. `sh tools/measure/mcp-check.sh` took both values with the `dev-release` engine on an 18-core host. Each is the median of 5 samples, and each sample is the mean of 3 runs. The load average of the host was near 11 during the run. In the same session, the engine before #1450 cost 240 ms and 587 ms. On 2026-09-30, at 489 documents, the values were 233 ms and 587 ms. So a warm cache now saves about two thirds of a cold run, and the rest is inside the hook budget.

**Where the time of a warm run goes.** `sh tools/measure/check-phases.sh` prints the CPU time of each stage of one warm run, from an engine built with the `phase-times` feature. On 2026-10-01 the stages summed to 199 ms against 200 ms for the whole run on that build. Phase A was 94 ms. The check of every rule was 89 ms, and the edge rules were 48 ms of that. Everything else was 16 ms. Before #1450 the first edge rule alone cost about 96 ms. It asks for the revision of each `governs` target, and each revision read its own files. So a file that many edges name was read once for each edge. That was 524 reads of 299 files, or 37 MB for 8.5 MB of distinct bytes. Now one resolver reads each file once. A timing with no host, no build profile and no corpus size cannot be derived again. So this paragraph states all three.

The first measurement, in 2026-08, was 54 to 61 ms warm and 470 to 486 ms with `--no-cache`. It used seven runs of each, a release build, an eight-core host and 158 classified documents. The warm run grew by about four times while the corpus grew by about three times.

The cache also derives what moved from content hashes rather than from a list that a caller supplies. A flag that took such a list would add an input to the verdict that no reviewer sees. Spec 6 forbids that in the paragraph that now states there is no flag.

**One closed argument loses a premise here.** [Q12](../decisions/0012-migration-path-for-an-existing-corpus.md) refused `--since <ref>` on three grounds, and the third was that the name is taken by `--changed-only`. No binary ever carried that flag, so the name was taken by a line in spec 6 rather than by a mechanism. Q12's ruling stands on its other two grounds. The first is that a flag which decides the findings that count makes two runs over one tree disagree. The second is that such a flag turns an unchecked document into an unreported one. That decision record and the [first-contact evaluation](../evaluations/first-contact.md) keep their text, because each one records what was argued on its date.

## Obligation

What no flag reaches is Phase A. `headwater explain` walks the corpus, parses it, builds the graph and runs no check. On 2026-10-01 it cost 90 ms at 500 typed documents, in the same run as the values above. In 2026-08 it cost 36 to 45 ms at 158 documents. So Phase A is now about half of the warm run. The warm cache does not serve the other half either, because a served verdict still reads the digest of every file that its key names. To scope it is to cache the census and the graph, and [Q6](../spec/09-decisions.md#q6--where-the-corpus-graph-lives-at-rest) rules that nothing stores the graph. [HW-OBL-0072](0072-a-cache-of-check-results-does-not-make-a-run-proportional.md) carries what that leaves open in spec 6's own promise.

## Discharge

What reopens this is a parse cache, and no flag at any corpus size. A cache of one parse per file is disposable, derived, and verdict-neutral under the `--no-cache` byte-identity differential that already runs on every gate. Q6 forbids a canonical stored graph, and it does not forbid such a cache.

The trigger to build one is Phase A alone passing the 200 ms hook budget. Phase A costs 90 ms over the 500 documents that this corpus classifies, so the trigger has not fired. A linear reading puts it near 1,100 documents of the length this corpus writes. One corpus supports that reading, so what reopens this record is the measurement rather than the document count. `sh tools/measure/mcp-check.sh` prints the Phase A value on its last line.

The commit hook is inside its target by 23 ms, and a larger corpus takes that margin before Phase A reaches the trigger. At 500 documents the warm run spends about 87 ms after Phase A, and `check-phases.sh` names each stage of it. The stage that grows with the governed code is the digest of each governed file. Spec 6 keeps the 200 ms target and states the measurement beside it. This record stays open, and it waits on the next measurement.
