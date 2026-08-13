---
id: OBL-repo-0103
title: "The front-matter half of a patch has no writer"
status: current
status_since: 2026-08-14
last_verified: 2026-08-14
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
    - SPEC-HW-check-layer
    - SPEC-HW-authoring-and-lifecycle
    - SPEC-HW-taxonomy-model
---

# The front-matter half of a patch has no writer

## Context

[Spec 12](../spec/12-check-layer.md#fixability) lists four mechanical corrections. Two of them are edits inside front matter: "to normalize front-matter key order, to correct the format of an identifier". [Spec 3](../spec/03-authoring-and-lifecycle.md#authoring-surfaces) names a third in its table of authoring surfaces, where `headwater check --fix` is to "format front matter". [Spec 2](../spec/02-taxonomy-model.md#participation-expectations) names a fourth: "`check --fix` stamps the date only when the transition sits in the same diff".

`headwater check --fix` carries two patch shapes and neither one reaches a facet. One replaces a byte range of the body, under a read-back that compares the parse of the result against the parse of the source. The other declares one edge half, and `headwater_scaffold::write::splice` writes it under a read-back of its own.

`shelf.placement_is_primary` is the rule that measures the gap. Its remedy is to delete one key of one mapping, which meets the bar in the sentence above. The rule declared `fixable: true` with no patch behind it until `--fix` landed, and it now reports the same error with remediation prose.

## Obligation

A patch that edits front matter needs a read-back over a mapping, and the corpus owes the statement of what that read-back compares.

The prose guard compares two parses of a body. It holds four properties: the blocks, their owners, the links, and the text of every run. A mapping has no such shape to compare. A deletion moves the span of every key below it, and a normalization of key order moves all of them. A stamped date adds a key that the source never held. So a front-matter patch cannot state the property a reader states for prose, which is that everything stands except here.

Two questions follow, and no document answers either.

The first is what a front-matter read-back compares. A comparison of the loaded mapping against an expectation is one answer, and it needs the expectation to be a value rather than a diff. A comparison of the spans of the keys that stayed is another, and it holds a weaker property.

The second is the grain. [OBL-repo-0088](0088-correcting-an-identifier-is-mechanical-and-it-is-not-local.md) records that correcting an identifier is mechanical and reaches every document that cites it. A patch shape that names one file cannot express that repair, and this record is where the shape would gain the reach.

## Discharge

A patch shape that writes a facet, under a read-back that a reader can state in one sentence. Each way the read-back refuses owes a failing fixture. `shelf.placement_is_primary` is the first customer and its patch is a deletion, and the fixture corpus already holds the document that fails the rule.

Until then every rule whose remedy is a facet reports remediation prose, and `fixable` in a report stays the narrow reading that [OBL-repo-0087](0087-fixable-has-two-readings-inside-one-engine.md) settled.
