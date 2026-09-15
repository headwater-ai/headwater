---
id: HW-PROBE-the-pointer-this-corpus-offers-for-a-task-is-the-document-a-session-opens
status: current
status_since: 2026-09-11
summary: Precision at intent time has no reading at all, and this takes the half of it that a transcript of tool calls can carry.
last_verified: 2026-09-11
probe_category: discovery
expectation: opened
oracle: "none"
title: "The pointer this corpus offers for a task is the document a session opens"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  examines:
    - HW-OBL-0107
  traces_to:
    - HW-SPEC-ai-integration
---

# The pointer this corpus offers for a task is the document a session opens

## Task

Decide which shelf a new document belongs on, and where its identifier comes from.

The task names no verb and no file. It is the text the router below was given, word for word, because the predicate is over what that text resolves to.

## Expectation

`opened` over `HW-OBL-0107`. The session read the document that `headwater route` ranks first for the task text above.

**The pointer, on the commit that lands this probe.** `headwater route --root . "decide which shelf a new document belongs on, and where its identifier comes from"` ranks `docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md` first, and the default budget of five withholds the rest. The withheld count is not quoted here, because it moves on every commit that adds a document and a quoted one would be stale on the merge that lands this. A later commit may also rank a different document first, and the predicate then names the wrong one. Re-run the router and move the `examines` edge when this corpus moves.

**This is the routing-precision instrument that [#404](https://github.com/headwater-ai/headwater/issues/404) waits on.** [Spec 5](../spec/05-ai-integration.md#intent-time-routing) names precision and abandonment as the two measures that decide whether the router needs an embedding fallback, and #404 records that no probe measures either. The rate this probe returns is the first reading of the first measure.

**Abandonment stays unmeasured and this probe does not reach it.** [Spec 15](../spec/15-the-recorder-contract.md) fixes what a transcript carries, and a transcript records the tool calls a session made. It records no list of the pointers a session was offered. So "of the pointers offered, how many did the session ignore" cannot be computed from any recording this contract admits, and no probe can grade it until the contract or the harness carries the offered list.

**A `present` arm that ranks the wrong document is the finding rather than a fault of the run.** The router reads the corpus and the text and nothing else, so its ranking is computable offline and reproducible. A low rate here says that the summary the corpus wrote for that document did not separate it from its shelf siblings, which is the failure spec 5 puts on the summary rather than on the session.
