---
id: HW-OBL-0036
title: "Identity below the grain of a document"
status: current
status_since: 2026-08-12
waiting_on: build
last_verified: 2026-08-13
summary: "A decision and an obligation are documents now, and a criterion inside a sidecar and a section of a document still reach no identifier index."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-authoring-and-lifecycle
    - HW-SPEC-assurance-model
    - HW-SPEC-glossary
---

# Identity below the grain of a document

## Context

[Spec 3](../spec/03-authoring-and-lifecycle.md#identifiers) reserves an identifier for five artifacts, and a document of this corpus was not one of them. This register cited a decision as Q13, and it cited an obligation by its position in a list. Neither was a document, so neither reached the identifier index and no edge named one.

A declared obligation was out of this first. It carries an identifier, the identifier is its key, and an address reaches it. The entries of this register were not declarations, so they gained nothing from that change.

## Obligation

A taxonomy for this corpus has to say which decision an evaluation closed, and which obligation a piece of work discharges. Each of them has to be a document with an identifier before it can.

**One naming defect belongs with it.** This specification calls two unrelated things an obligation. [Spec 4](../spec/04-assurance-model.md#obligations-are-data) declares `obligations` as a taxonomy block, which holds invariants that a corpus commits to. This register holds work that a corpus owes. The decision-record entry writes `obligation_record` and mints `OBL-` to keep the two apart in one corpus, and the [glossary](../spec/glossary.md) is where the general fix belongs.

## Discharge

**The tradition is authored and the whole conversion has run.** The [decision-record entry](../taxonomies/decision-record/doctrine.md) is the second entry in the library. It reuses the base `decision` kind for a decision. It adds `obligation_record` for an entry of this register, because the two serve different reader intents. Each kind carries an identifier scheme, a shelf, and a section contract. So the schema half closed there.

[#124](https://github.com/headwater-ai/headwater/issues/124) closed the decisions half of the corpus. Twenty-one decision documents sit on `docs/decisions/**`, each with an identifier of the `decision_id` scheme, and 23 `traces_to` edges say which decision an evaluation closed. [#126](https://github.com/headwater-ai/headwater/issues/126) closed the obligations half. Every entry of this register is a document on `docs/obligations/**`. Each one carries an identifier of the `obligation_record_id` scheme, and an edge to the decision or the specification that produced it.

What stays open is the grain below a document. A criterion inside a contract sidecar is not a document, and [the scope of a sub-document identifier](0047-the-scope-of-a-sub-document-identifier.md) holds that half. A section of a document is not one either, so a citation of a heading still names a position rather than a node.

**The register measures what that grain costs.** Twenty-one citations in this corpus name a heading of [13 — Open obligations](../spec/13-open-obligations.md), and nine of them name a heading that no document backs. So no partition of the obligations shelf writes those two sections, and [the register](../spec/13-open-obligations.md#a-human-maintains-this-list-by-hand) stays authored for that reason.
