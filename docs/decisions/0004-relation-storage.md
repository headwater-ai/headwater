---
id: HW-DR-0004
title: Q4 — Relation storage
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: Front matter is authoritative, and a relation instance is an object rather than a pointer.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-EVAL-relation-storage
---

# Q4 — Relation storage

## Context

Front matter is authoritative, and the [evaluation](../evaluations/relation-storage.md) confirmed the leaning. It did not confirm it in the shape that the open-question entry expected, and three of its findings change the specification.

**The question was not the one that the entry asked.** The entry reads as a choice between three files. [Q18](0018-recording-adjudicated-disagreements.md) wanted an adjudication recorded on the declared edge. [Q20](0020-where-scent-lives.md) wants an optional cue on a relation, and said that it blocked here. Both need an edge that carries data of its own, so the prior question is whether a relation instance is a pointer or an object. It is an object.

**The sidecar loses on ownership rather than on convenience.** DITA relationship tables are this option, shipped for two decades in the same domain. A DITA relationship belongs to the map, so the same topic under a second map has different relationships. A Headwater relation asserts something about two documents, and it holds in every context that contains them. Standoff annotation supplies the second argument. It exists because inline markup cannot express overlapping hierarchies, which is a problem that a relation does not have. Its cost is pointer fragility, which a relation would still pay.

## Decision

A relation is declared under a `relations:` block in front matter and nowhere else. An entry is either a target reference or a mapping with `to:` and instance attributes. The scalar is sugar for the mapping, and both produce the same edge. Targets are identifiers, never paths. An edge is identified by the source identifier, the relation name, and the normalized target. List order therefore carries no meaning, and a repeated triple is an error.

## Consequences

**The annotated prose link is cut, and that is the largest change here.** The old leaning allowed a prose link to carry relation semantics "unless they are annotated". An annotation syntax gives one edge two authoring locations, and three questions then have no good answer. Which location wins when they disagree? Which span does a finding anchor to? What does `--fix` write when it adds a reciprocal? [Spec 1](../spec/01-conceptual-model.md#facet) already refused a second edge syntax once, when it removed reference-valued facets, and the reasoning transfers without change.

**The friction that the old leaning worried about becomes a check.** A prose link that resolves to a corpus document with no declared relation raises an advisory finding. The author writes the link once, and the fix writes the declaration. The fix is mechanical only when exactly one enabled relation type permits the pair of kinds at the two ends. Otherwise the finding lists the candidates and carries no patch. The check has no converse, because a succession edge belongs in no paragraph.

**Instance attributes are governed the way facets are.** A relation type declares which attributes its instances may carry, and an undeclared attribute is a finding. An attribute takes a facet's value space and is never a reference. An edge that must point at a node is a request to make the edge a node. [Q18](0018-recording-adjudicated-disagreements.md) owned that change and declined it, because an adjudication that needs an author, a date and a reason is a document. So [Q20](0020-where-scent-lives.md)'s cue is the one live instance attribute in this specification. Each attribute declares an owning end. A source-owned attribute on a symmetric relation gives one value per direction. An edge-owned attribute with two different values at the two ends is a finding, and no fix resolves it. `created_by` stays on the relation type, because `taxonomy audit` measures the declared intent against a real corpus.

**Two consequences land outside this record.** The internal model is a property graph. So the RDF projection of [Q6](0006-where-the-corpus-graph-lives-at-rest.md) reifies any edge that carries an attribute. Q6 has since closed, and it replaced the round-trip test with a declared loss set and a projection census. The reification is one entry in RDF's loss set. And [Q20](0020-where-scent-lives.md) now has a home for its cue, plus an answer to one of its three questions: the referring end owns it.
