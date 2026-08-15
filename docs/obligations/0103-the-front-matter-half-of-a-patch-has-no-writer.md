---
id: HW-OBL-0103
title: "The front-matter half of a patch has no writer"
status: current
status_since: 2026-08-14
last_verified: 2026-08-15
summary: "Three documents name a mechanical correction inside front matter, and the two patch shapes that ship reach a run of prose and a relations block."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-check-layer
    - HW-SPEC-authoring-and-lifecycle
    - HW-SPEC-taxonomy-model
---

# The front-matter half of a patch has no writer

## Context

[Spec 12](../spec/12-check-layer.md#fixability) lists four mechanical corrections. Two of them are edits inside front matter: "to normalize front-matter key order, to correct the format of an identifier". [Spec 3](../spec/03-authoring-and-lifecycle.md#authoring-surfaces) names a third in its table of authoring surfaces, where `headwater check --fix` is to "format front matter". [Spec 2](../spec/02-taxonomy-model.md#participation-expectations) names a fourth: "`check --fix` stamps the date only when the transition sits in the same diff".

`headwater check --fix` carries two patch shapes and neither one reaches a facet. One replaces a byte range of the body, under a read-back that compares the parse of the result against the parse of the source. The other declares one edge half, and `headwater_scaffold::write::splice` writes it under a read-back of its own.

`shelf.placement_is_primary` is the rule that measures the gap. Its remedy is to delete one key of one mapping, which meets the bar in the sentence above. The rule declared `fixable: true` with no patch behind it until `--fix` landed, and it now reports the same error with remediation prose.

## Obligation

A patch that edits front matter needs a read-back over a mapping, and the corpus owes the statement of what that read-back compares.

The prose guard compares two parses of a body. It holds four properties: the blocks, their owners, the links, and the text of every run. A mapping has a shape to compare, and `headwater_scaffold::migrate` now states it. The comparison covers every scalar of the front matter, as a key path and a text in document order. The bytes of the body stand beside it.

That comparison holds a replacement of one value, which is the edit a migration step makes. It does not hold the three corrections this record opened with. A deletion moves the span of every key below it, and a normalization of key order moves all of them. A stamped date adds a key that the source never held. Each of the three changes the key-path list itself, so the expectation for that edit is not the list the source read.

Two questions follow, and the first now has an answer for one shape of edit.

The first is what a front-matter read-back compares. `taxonomy migrate --apply` answers it for a replacement, and the expectation is a value rather than a diff. The value is the scalar list of the source with the moved entries substituted. The other answer, a comparison of the spans of the keys that stayed, holds a weaker property and nothing takes it. A deletion, a re-order and an addition each need an expectation of their own, and no document states one.

The second is the grain. [HW-OBL-0088](0088-correcting-an-identifier-is-mechanical-and-it-is-not-local.md) records that correcting an identifier is mechanical and reaches every document that cites it. A patch shape that names one file cannot express that repair, and this record is where the shape would gain the reach.

## Discharge

A patch shape that writes a facet, under a read-back that a reader can state in one sentence. `headwater taxonomy migrate --apply` supplies the read-back and not the patch shape. It writes a facet, it is no `Patch`, and no rule reaches it. What stays open is the variant `check --fix` carries, an expectation for an edit that moves a key, and the reach that [HW-OBL-0088](0088-correcting-an-identifier-is-mechanical-and-it-is-not-local.md) prices. Each way the read-back refuses owes a failing fixture. `shelf.placement_is_primary` is the first customer and its patch is a deletion, and the fixture corpus already holds the document that fails the rule.

Until then every rule whose remedy is a facet reports remediation prose, and `fixable` in a report stays the narrow reading that [HW-OBL-0087](0087-fixable-has-two-readings-inside-one-engine.md) settled.
