---
id: HW-OBL-0053
title: "The front-matter key that carries an identifier is undeclared"
status: discharged
status_since: 2026-09-16
waiting_on: build
last_verified: 2026-09-16
summary: "A kind declares an identifier scheme, and nothing states which key in front matter holds the minted value."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0002
---

# The front-matter key that carries an identifier is undeclared

## Context

A kind declares `identifier: {scheme: …}`, and nothing states which key in front matter holds the minted value. The typing pass wrote `id`.

This is the fourth surface that has a required declaration and no stated form. [Q2](../spec/09-decisions.md#q2--schema-format) owns the schema language that would state it.

## Obligation

The language owes a declaration of the key, so that an adopter reads it rather than infers it.

## Discharge

The guess is contained rather than closed. `headwater_graph::Config` holds the key as a parameter, and the generated identifier check takes the same parameter. So the index and the rule read one key and cannot disagree. Where that key finds nothing, the check skips and the skip quotes the key. A finding there would report the engine's assumption as the author's defect. The containment keeps the guess out of every report about a document.

## Discharge, 2026-09-16

**The declaration is stated, and this record closes.** [HW-DR-0068](../decisions/0068-the-front-matter-key-that-carries-a-minted-identifier-is-id.md) rules that the front-matter key `id` carries the value a kind's `identifier: {scheme: …}` facet mints, as a global fixed rule with no per-kind override. It follows the precedent [Q4](../decisions/0004-relation-storage.md) set for `relations:`. That precedent is a fixed literal key, stated once for every kind, and never a parameter left to a declaration nobody wrote.

`headwater_graph::Config`'s `identifier_facet` doc comment, and the matching comment in `index.rs`, now cite HW-DR-0068 instead of naming the key an unsettled guess. `.headwater/README.md`'s guess table lost the row for this gap. The containment this record described stays exactly as built. The field is still a parameter rather than a literal. What changed is that the value it defaults to is now a ruling, and not a guess.
