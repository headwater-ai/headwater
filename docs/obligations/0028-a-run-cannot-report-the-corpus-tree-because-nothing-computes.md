---
id: HW-OBL-0028
title: "A run cannot report the corpus tree, because nothing computes one"
status: current
status_since: 2026-08-13
waiting_on: build
last_verified: 2026-08-14
summary: "Spec 6 asks every run for a corpus tree beside its findings, and the engine computes none."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-engine-architecture
---

# A run cannot report the corpus tree, because nothing computes one

## Context

[Spec 6](../spec/06-engine-architecture.md#ci-adapters) asks every run to report three things beside its findings: the corpus tree, the taxonomy lock hash, and the read set. The engine reports the last two and computes no tree.

The read set is not one, because it holds what the checks read and a tree holds what the census walked.

## Obligation

A run therefore states nothing about a file that no check opened, which is the exact set a merge can add. The corpus owes a tree that covers every row of the census.

## Discharge

SARIF has the member for a tree, `run.automationDetails.id`, and the adapter leaves it out rather than print the read set's identity there. The cost grows with the corpus. A wider excluded set leaves a gate less of the evaluated state to hold a later tree against.

**The check layer asks for no tree, and this record stands on spec 6 alone.** [Spec 12](../spec/12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict) fixes the gate as a comparison over listed inputs, which needs neither a tree nor a run. So the test that decides whether a verdict survives a merge does not wait on this record. [HW-OBL-0102](0102-two-documents-state-what-voids-a-verdict-and-they-do-not.md) carries that ruling and the construction under it.

**The MCP server is the first consumer that can answer about a tree that is gone.** It walks the corpus once, before it accepts a message, so a session outlives the walk behind it. A `check` call over a server started before an edit was measured at 28 findings where a fresh run reported 30. The two answers of that server were identical across the edit. Every other caller re-walks on every invocation, so no other caller can be wrong this way. The `text` format leaves a reader the read set, which is a hash per listed input and not a tree. The other three formats leave a reader nothing at all. Until a run reports a tree, the remedy is the [operational one](../spec/05-ai-integration.md#what-a-check-tool-decides-and-where-each-decision-is-taken). A server holds one tree for its life, and a caller that wants another starts another server.

**A server that writes cannot make that mistake, because it ends instead.** The working-tree write class arrived on 2026-08-14, and a call that moves a byte of the checkout spends the server ([spec 5](../spec/05-ai-integration.md#what-the-working-tree-write-class-registers-and-what-a-session-looks-like-after-a-write)). Every later call is refused rather than answered from the walk the write made stale. One British spelling was planted in a copy of this tree, which then carried 28 findings. One `fix` call over the protocol wrote the patch and answered 27. The next call was refused. Under the reading above the second answer would have been 28. The edit an unwitting reader makes beside a running server is still the case this record is open for.
