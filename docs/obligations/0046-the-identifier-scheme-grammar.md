---
id: OBL-repo-0046
title: "The identifier scheme grammar"
status: current
status_since: 2026-08-12
last_verified: 2026-08-13
summary: "A scheme holds the prefix that discriminates a kind as a literal inside a pattern string, so no engine can decide that two schemes mint disjoint identifiers."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - SPEC-HW-taxonomy-model
    - SPEC-HW-authoring-and-lifecycle
---

# The identifier scheme grammar

## Context

The same review closed the namespace gap in place, so `core` now requires one and `taxonomy validate` checks it. What stays open is the shape of a scheme: the prefix that discriminates the kind is a literal inside the `pattern` string.

## Obligation

So the engine cannot read the prefix that it mints, and [identifier integrity](../spec/02-taxonomy-model.md#the-thirteen-declarations) validates the whole pattern rather than its parts. A grammar that declares the prefix, the namespace, and the local part as separate fields makes disjointness decidable by construction.

## Discharge

The generated check `identifier.pattern.not_met` reads the same string and reports around the gap. It parses the pattern into literal runs and placeholders. So it names the segment that stopped a match, and it still cannot decide that two schemes mint disjoint identifiers.

A declared grammar also lets an RDF projection derive a document IRI rather than invent one. The [worked example](../evaluations/owl-skos-worked-example.md#the-method) derives one from a file path today. That is a meta-schema change, so it costs a taxonomy major version once the meta-schema ships, and nothing today. [Spec 3](../spec/03-authoring-and-lifecycle.md#identifiers) is where the grammar belongs.
