---
id: HW-OBL-0080
title: "`--changed-only` is the content-addressed cache under another name"
status: current
status_since: 2026-08-13
waiting_on: measurement
last_verified: 2026-09-30
summary: "The warm cache leaves the commit hook past its 200 ms budget, and a change-scoped flag stays refused because it hides a verdict input."
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

[Spec 6](../spec/06-engine-architecture.md#cli) states that there is no `--changed-only`, and this record holds the measurement behind that sentence. The one job such a flag has is to pay for a 200 ms commit hook without moving the verdict. In 2026-08 the cache paid it. At 477 and at 487 typed documents it does not, and the commit hook is past its target.

**The measurement, and the conditions it was taken under.** On 2026-09-30, over this repository at 487 typed documents, a warm `headwater check --strict` cost 243 ms of user and system CPU time. The same run with `--no-cache` cost 623 ms. `sh tools/measure/mcp-check.sh` took both values with the `dev-release` engine on an 18-core host. Each is the median of 5 samples, and each sample is the mean of 3 runs. The load average of the host was near 100 during the run. On 2026-09-29, at 477 documents and a lower load, the values were 212 ms and 530 ms. So a warm cache now saves about three fifths of a cold run, and the rest is more than the hook budget. A timing with no host, no build profile and no corpus size cannot be derived again. So this paragraph states all three.

The first measurement, in 2026-08, was 54 to 61 ms warm and 470 to 486 ms with `--no-cache`. It used seven runs of each, a release build, an eight-core host and 158 classified documents. The warm run grew by about four times while the corpus grew by about three times.

The cache also derives what moved from content hashes rather than from a list that a caller supplies. A flag that took such a list would add an input to the verdict that no reviewer sees. Spec 6 forbids that in the paragraph that now states there is no flag.

**One closed argument loses a premise here.** [Q12](../decisions/0012-migration-path-for-an-existing-corpus.md) refused `--since <ref>` on three grounds, and the third was that the name is taken by `--changed-only`. No binary ever carried that flag, so the name was taken by a line in spec 6 rather than by a mechanism. Q12's ruling stands on its other two grounds. The first is that a flag which decides the findings that count makes two runs over one tree disagree. The second is that such a flag turns an unchecked document into an unreported one. That decision record and the [first-contact evaluation](../evaluations/first-contact.md) keep their text, because each one records what was argued on its date.

## Obligation

What no flag reaches is Phase A. `headwater explain` walks the corpus, parses it, builds the graph and runs no check. On 2026-09-30 it cost 83 ms at 487 typed documents, in the same run as the values above. In 2026-08 it cost 36 to 45 ms at 158 documents. So Phase A is now about a third of the warm run. The warm cache does not serve the other two thirds either. To scope it is to cache the census and the graph, and [Q6](../spec/09-decisions.md#q6--where-the-corpus-graph-lives-at-rest) rules that nothing stores the graph. [HW-OBL-0072](0072-a-cache-of-check-results-does-not-make-a-run-proportional.md) carries what that leaves open in spec 6's own promise.

## Discharge

What reopens this is a parse cache, and no flag at any corpus size. A cache of one parse per file is disposable, derived, and verdict-neutral under the `--no-cache` byte-identity differential that already runs on every gate. Q6 forbids a canonical stored graph, and it does not forbid such a cache.

The trigger to build one is Phase A alone passing the 200 ms hook budget. Phase A costs 83 ms over the 487 documents that this corpus classifies, so the trigger has not fired. A linear reading puts it near 1,200 documents of the length this corpus writes. One corpus supports that reading, so what reopens this record is the measurement rather than the document count. `sh tools/measure/mcp-check.sh` prints the Phase A value on its last line.

The commit hook is past its target already, and a parse cache alone does not bring it back. At 487 documents the warm run spends about 160 ms after Phase A. Spec 6 keeps the 200 ms target and states the measurement beside it. This record stays open, and it waits on the next measurement.
