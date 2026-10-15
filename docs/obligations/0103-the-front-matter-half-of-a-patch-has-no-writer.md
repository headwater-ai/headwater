---
id: HW-OBL-0103
title: "The front-matter half of a patch writes a value and no key"
status: current
status_since: 2026-08-14
waiting_on: build
last_verified: 2026-10-15
summary: "`Patch::Facets` replaces the value of a declared facet, and a fix that deletes, re-orders or adds a front-matter key still has no writer."
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-check-layer
    - HW-SPEC-authoring-and-lifecycle
    - HW-SPEC-taxonomy-model
---

# The front-matter half of a patch writes a value and no key

## Context

[Spec 12](../spec/12-check-layer.md#fixability) lists four mechanical corrections. Two of them are edits inside front matter: "to normalize front-matter key order, to correct the format of an identifier". [Spec 3](../spec/03-authoring-and-lifecycle.md#authoring-surfaces) names a third in its table of authoring surfaces, where `headwater check --fix` is to "format front matter". [Spec 2](../spec/02-taxonomy-model.md#participation-expectations) names a fourth: "`check --fix` stamps the date only when the transition sits in the same diff".

`headwater check --fix` now carries four patch shapes, and one of them reaches a facet. `Patch::Text` replaces a byte range of the body, under a read-back that compares the parse of the result against the parse of the source. `Patch::Half` declares one edge half, and `headwater_scaffold::write::splice` writes it under a read-back of its own. `Patch::Create` makes a file where none stands.

`Patch::Facets` arrived with [#1207](https://github.com/headwater-ai/headwater/issues/1207). It rewrites the value of a top-level scalar facet that the front matter already declares, and it adds no key. Its first customer is `lifecycle.state.not_set_by_edge`, whose fix writes a state and its state-entry date together. Its read-back is the one that `headwater_scaffold::migrate` states for a replaced value: the result reads back with the written values moved and nothing else.

## Obligation

The corpus owes a writer for the three front-matter edits that change the list of keys, and a stated read-back for each one.

The read-back that ships compares every scalar of the front matter, as a key path and a text in document order. It also compares the bytes of the body. For a replacement, the expectation is the scalar list of the source with the written values substituted. That expectation does not hold the three corrections that this record opens with. Each one changes the key-path list itself:

- **A deletion.** `shelf.placement_is_primary` is the first customer. It reports an error whose remedy is to remove one key of one mapping, and it reports that remedy as remediation prose with no patch.
- **A normalization of key order.** It moves every key, so no entry stays at its source position.
- **An added key.** Spec 2 stamps a date where the source holds no date key.

One sentence states the expectation for a deletion: the scalar list of the source, minus each entry under the removed key path, in source order. So the next act for a deletion is a build and not a choice. A re-order and an addition each still need an expectation that somebody states.

Two more gaps stay inside this record.

- **The grain.** [HW-OBL-0088](0088-correcting-an-identifier-is-mechanical-and-it-is-not-local.md) records that correcting an identifier is mechanical and reaches every document that cites it. A patch shape that names one file cannot express that repair.
- **The `scent` facet.** [Spec 12](../spec/12-check-layer.md#the-five-origins-of-a-check) widens `language.controlled.not_met` and `language.retired_term.used` to the facet in the `scent` role. A contraction or a British spelling in a `summary` is a replacement of one value, which is the edit `Patch::Facets` already holds. The two rules still report it with remediation prose, and the test `a_lexical_rule_reads_the_facet_in_the_scent_role_and_no_other` over `engine/crates/check/fixtures/check/spec/21-summary-facet-prose.md` holds that absence.

Until each edit has a writer, a rule whose remedy is that edit reports remediation prose. `fixable` in a report stays the narrow reading that [HW-OBL-0087](0087-fixable-has-two-readings-inside-one-engine.md) settled.

## Discharge

This record is discharged when `headwater check --fix` writes each of the three key-list edits under a read-back that a reader can state in one sentence. Each way that read-back refuses has a failing fixture.

The first step is the deletion, with the expectation stated above, and `shelf.placement_is_primary` offers a patch for it. The fixture corpus already holds a document that fails that rule. The re-order and the added date each close when their expectation is written into the module comment of `headwater_scaffold::fix` and a fixture holds it.

The `scent` facet closes when the two lexical rules offer a `Patch::Facets` over a `summary`. The test above then asserts that patch, and not its absence. The reach across documents stays with [HW-OBL-0088](0088-correcting-an-identifier-is-mechanical-and-it-is-not-local.md), and this record does not wait on it.
