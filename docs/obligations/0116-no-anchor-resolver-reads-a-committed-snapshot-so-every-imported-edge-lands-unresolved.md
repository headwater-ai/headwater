---
id: HW-OBL-0116
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
    - HW-DR-0019
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

**This record is discharged, and a resolver reads the committed snapshot.** `headwater_import::anchors` is the second resolver [spec 2](../spec/02-taxonomy-model.md#behavior-at-the-limits) names. It binds an item identity that the pin holds, and it refuses one the pin does not hold.

**The resolver set is built from a corpus and from what a caller adds to it.** `Resolvers::over` still takes a corpus, and `Resolvers::with` takes one resolver that the graph crate cannot build. The `load` function of the command line is the one place that call sits, so every verb that builds a graph has the resolver. `headwater-import` depends on `headwater-graph`, so the graph crate cannot name a snapshot resolver at all. The dependency order decided the shape, rather than a preference between two designs.

**It looks an identity up and it never normalizes one.** The whole of the normalization is a trim, and everything after it is a lookup in the pinned item list. An upstream identity has no equality rules that this repository can know. A resolver that guessed would bind a typo to a real item, which is a correct check result over a wrong graph.

**The measurement is the hand run above, repeated on the branch on 2026-08-14.** The same four verbs over the same shape of repository now report the edge as bound:

    docs/specifications/00-first.md
      to 12345 audited_by
    1 ado_work_item `12345` via ado-snapshot

The check layer reports no instance of `relation.target.unresolved` against the document the import wrote. A second document that names `12346` by hand, which the snapshot does not hold, is reported rather than bound.

**A declaration that supplies no resolver is refused.** `imports.<name>.resolver` names the resolver the snapshot serves, and `headwater import` refuses a declaration that leaves it out. An import with no resolver writes edges that nothing can bind, which is the state this record measured. The refusal is what stops that state from being reachable by omission.

**What the fixture set gained.** `engine/crates/import/tests/resolution.rs` runs the whole chain in a suite. It plans the import, writes it, walks the result, and holds both the graph and the check layer to the outcome. The case that carries the weight is an identity the snapshot does not hold, reported rather than bound. Each case was proved able to fail by regressing the code it holds.
