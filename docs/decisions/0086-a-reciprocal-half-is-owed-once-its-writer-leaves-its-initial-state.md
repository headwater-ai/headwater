---
id: HW-DR-0086
status: current
status_since: 2026-09-26
summary: "While a draft holds the only half of a required pair, nothing is owed. The far document owes its half once the draft is promoted."
last_verified: 2026-09-26
title: "A reciprocal half is owed once its writer leaves its initial state"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: evidenced
relations:
  constrains:
    - HW-DR-0004
    - HW-DR-0085
  traces_to:
    - HW-DR-0052
    - HW-SPEC-authoring-and-lifecycle
---

# A reciprocal half is owed once its writer leaves its initial state

## Context

A relation that declares `reciprocal: required` asks both documents of a pair to declare it. [HW-DR-0004](0004-relation-storage.md) puts every relation in front matter, and `relation.reciprocity.missing` reports a pair that has one half. Before this record, the rule read no state, so the far half was owed at all times.

`headwater new` opens a document at the initial state of its regime. For a required relation, it also wrote the far half into the target document. The target is usually live. [HW-DR-0085](0085-a-live-document-that-rests-on-a-draft-one-is-reported-over-every-relation-because-a-draft-leaves-no-record-to-cite.md) reads each half from the document that wrote it. So the live target then wrote a line to a draft, and `lifecycle.dependency.on_initial` warned on it. HW-DR-0085 accepted that warning as a cost, and its repair was to promote the new document before the author proposes it.

The warning reaches an adopter. The base `supersedes` is exempt, because it writes a state onto its target. Three relations of the published bundles are not exempt: `applied_in`, `realizes` and `elaborates`. Each one requires both ends, and `headwater new` creates it. So `headwater new prd --relates elaborates=<BRD>` writes `elaborated_by` into a live BRD, and that BRD warns.

[#1168](https://github.com/headwater-ai/headwater/issues/1168) reported the defect. The owner ruled on it on 2026-09-26, in [a comment on the issue](https://github.com/headwater-ai/headwater/issues/1168#issuecomment-5846695748): the reciprocal half is deferred until the draft is promoted.

## Decision

A far half is owed only when the document that wrote the one half that exists stands at a state whose role is not `initial`. The document that wrote that half is the *writer*.

- When the writer stands at a state with the role `initial`, nothing is owed yet. `relation.reciprocity.missing` passes the pair, and `headwater new` writes no far half.
- When the writer stands at any other state, the half is owed now. The rule reports it on the writer, and `headwater check --fix` writes it into the far document.
- When the engine can read no state for the writer, the half is owed. This covers a taxonomy with no state facet, a document that writes no value, and a value that is not a state. An absence defers nothing.
- The deferral follows the writer and never the target. A live document that writes its half onto a draft is reported, and the draft owes the half now.

The engine reads "initial" through one predicate, `StateFacet::standing` in `headwater-check`. The rule and `headwater new` both call it, so `new` defers exactly where the rule passes. No state name is written in the engine, so a taxonomy that renames every state is read the same way.

This record changes what HW-DR-0004 and spec 3 mean when they say that both documents declare a pair. The pair is complete when both declare it, and the far document owes its half only from the writer's promotion. This record also withdraws the cost that HW-DR-0085 accepts in its Consequences. A scaffold does not write a line from a live document to a draft.

## Consequences

**`headwater new` writes one document for a required relation.** It records the far half as owed. Its report names the relation, the document that owes it, and the state the new document must leave. After the author promotes the new document, `headwater check` reports the missing half on it. `headwater check --fix` then writes the half through the same splice that `new` used before.

**The assisted fraction counts the halves of one run.** An owed half is not a half of the run, so it is in neither term. Each edge counts its near half, and a far half counts only when the run writes it. [HW-OBL-0001](../obligations/0001-the-promotion-fix-has-no-reading-of-the-assisted-fraction.md) records that no run has measured the fraction yet, and this record does not discharge it.

**A promotion now has a second step.** The author moves the state and then runs `headwater check --fix`, or writes the far half by hand. Until then, the rule reports an error on the promoted document. That error is what makes the far half visible, and the remedy is mechanical.

**A symmetric relation is unchanged.** `headwater new` still writes the far half of a `reciprocal: symmetric` relation, because no check reads a symmetric pair. The base declares one symmetric relation, `conflicts_with`, and `new` refuses it because an agent creates it.

**What reopens this.** Two cases are a reason to revisit the rule. The first is a regime whose initial state other documents must rely on. The second is a relation whose far half must exist before promotion.
