---
id: OBL-repo-0116
status: current
status_since: 2026-08-14
summary: "An import writes an edge onto an anchor kind, one resolver ships, and it reads the source tree, so the check layer reports every imported edge as unresolved."
last_verified: 2026-08-14
title: "No anchor resolver reads a committed snapshot, so every imported edge lands unresolved"
provenance:
  warrant: proposed
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - DR-repo-0019
---

# No anchor resolver reads a committed snapshot, so every imported edge lands unresolved

## Context

[Q19](../spec/09-decisions.md#q19--inbound-integration-an-external-system-of-record) rules that an imported edge carries full weight, and `headwater import` writes one. The far end of such an edge is an external item rather than a document, so it is an anchor. [Spec 2](../spec/02-taxonomy-model.md#behavior-at-the-limits) rules that exactly one resolver owns each anchor kind. It also rules that a resolver reads repository content or a committed snapshot.

**This engine has one resolver and it reads the source tree.** `headwater_graph::anchors::SourceTree` is it, and `Resolvers::over` builds the whole set from a corpus. Nothing reads a committed snapshot, and nothing carries the snapshot to the graph build at all.

## Obligation

An imported edge is worth what a reader of the graph can do with it, and today a reader is told it resolves to nothing.

The measurement is a hand run of the verb over a repository built for it on 2026-08-14. One document, one snapshot of one item, one relation declared `created_by: import` with an `ado_work_item` anchor at its far end. `headwater import ado --write` wrote the edge, `headwater explain` read it back, and `headwater check` then reported one instance of `relation.target.unresolved` against the document the import had just written:

    docs/spec/00-first.md:16:11
      edge: `audited_by` -> 12345 `ado_work_item` names the resolver `ado-snapshot`, which this run does not have

So the corpus owes a resolver that binds an item of a committed snapshot, and one ruling beside it. `Resolvers::over` takes a corpus and a snapshot resolver needs the snapshot. Either the resolver set is built from something wider than a corpus, or the caller adds to it. That choice reaches every verb that builds a graph.

## Discharge

Nothing discharges this yet. The record states what was true on 2026-08-14, so that no later reader mistakes an importer that writes for an import that resolves.

**What runs.** `headwater import` refuses every wrong link and writes the right ones, and `engine/crates/import/tests/fixtures.rs` provokes each refusal. The edge it writes carries the upstream revision of the item it was checked against.

**What no test here shows.** No fixture of the importer can fail on this, because the importer is right. The edge it wrote names the anchor kind the relation declares, and the taxonomy declares the resolver. The gap is one component further on, and it appears only when a check runs over the result. That is why the measurement above is a hand run of four verbs rather than a case in a suite.

**Who this reaches.** No adopter has imported anything. Until one does, an edge that reports as unresolved costs a finding rather than a wrong graph. That is the cheaper of the two failures, and it is why this is a record rather than a hold on the importer.
