---
id: OBL-repo-0053
title: "The front-matter key that carries an identifier is undeclared"
status: current
status_since: 2026-08-12
last_verified: 2026-08-13
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
    - DR-repo-0002
---

# The front-matter key that carries an identifier is undeclared

## Context

A kind declares `identifier: {scheme: …}`, and nothing states which key in front matter holds the minted value. The typing pass wrote `id`.

This is the fourth surface that has a required declaration and no stated form. [Q2](../spec/09-decisions.md#q2--schema-format) owns the schema language that would state it.

## Obligation

The language owes a declaration of the key, so that an adopter reads it rather than infers it.

## Discharge

The guess is contained rather than closed. `headwater_graph::Config` holds the key as a parameter, and the generated identifier check takes the same parameter. So the index and the rule read one key and cannot disagree. Where that key finds nothing, the check skips and the skip quotes the key. A finding there would report the engine's assumption as the author's defect. The containment keeps the guess out of every report about a document.
