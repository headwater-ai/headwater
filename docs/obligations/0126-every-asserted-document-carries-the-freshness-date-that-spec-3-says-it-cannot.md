---
id: HW-OBL-0126
status: current
status_since: 2026-09-06
waiting_on: ruling
summary: "Spec 3 rules that an `asserted` document cannot carry `last_verified`. The facet contract requires it on every kind that has one, and nine of nine asserted documents carry it."
last_verified: 2026-08-15
title: "Every asserted document carries the freshness date that spec 3 says it cannot"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-authoring-and-lifecycle
---

# Every asserted document carries the freshness date that spec 3 says it cannot

## Context

[Spec 3](../spec/03-authoring-and-lifecycle.md#the-warrant-and-what-each-value-requires) states the rule in one sentence: "Content with the `asserted` warrant carries no freshness value, and the engine never reports it as stale. `last_verified` records that a human confirmed a document, and nobody confirmed this one. It cannot acquire the date without becoming `accepted`, which is what promotion is."

The pairing table beside it names what each warrant requires and what it forbids. `asserted` requires `drafted_by`, `activity` and the sources that produced the content, and it forbids `accepted_by`. The table says nothing about `last_verified`, so the ruling stands in the paragraph alone.

The facet contract answers the other way. `last_verified` is in the `freshness` role, every kind that carries the facet requires it, and `facet.required.missing` reports its absence as an error. A document with the `asserted` warrant and no `last_verified` therefore fails the commit gate. This record was drafted after a run of `headwater check` reported exactly that against a new decision.

## Obligation

The corpus stands on the facet contract's side, in every case. `headwater taxonomy audit` reports nine documents under `warrant: asserted` on 2026-08-15, and nine of the nine carry `last_verified`. No document exercises the reading that spec 3 states.

Nothing reports the split. `facet.required.missing` reads the kind and never the warrant, and no rule pairs the two. So the sentence in spec 3 is a rule that no check enforces and that no document obeys. A reader who follows it writes a document the gate refuses.

Three answers are open and each says something different about what a warrant means. Spec 3 is wrong. A freshness date on unaccepted content records when the content was produced, rather than when a human confirmed it. Or the facet contract is wrong, and `last_verified` belongs to the kinds whose documents can be accepted rather than to every kind. Or the nine documents are wrong, and `asserted` is a warrant that this corpus has been writing for the wrong reason.

## Discharge

A ruling that names which of the three holds, recorded where the next author meets it. If spec 3 stands, the facet contract needs a declaration that makes `last_verified` conditional on the warrant. [HW-OBL-0123](0123-a-facet-that-applies-to-one-value-of-another-facet-has-nowhere-to-say-so.md) already reports that such a declaration has nowhere to be stated. If the facet contract stands, the sentence in spec 3 goes and the pairing table gains a row.

This is not settled by an agent. The question is what an acceptance stamp and a freshness date each mean, which is the same question [HW-OBL-0108](0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md) and [HW-OBL-0125](0125-nine-documents-state-a-warrant-the-closed-set-does-not-hold-and-no-check-reads-one.md) put to a human. An agent that answered it would be deciding what stands behind its own drafts.
