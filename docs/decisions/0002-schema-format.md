---
id: DR-repo-0002
title: Q2 — Schema format
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: YAML 1.2 is the concrete syntax, and Headwater owns the schema language, the reference sublanguage, the overlay language and the meta-schema.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - EVAL-HW-schema-format-walkthrough
---

# Q2 — Schema format

## Context

The [cognitive-dimensions walkthrough](../evaluations/schema-format-walkthrough.md) that the open-question entry prescribed has run, over the five authoring scenarios and the seven dimensions that it named.

The question decomposed into three, and only two of them were open. Who owns the schema language? [Q13](0013-linkml-and-shacl-as-substrate.md) answered that already: Headwater owns it, and standard formats come out of it. What syntax do authors type, and what checks partial work? Those two are what the walkthrough settled.

The earlier leaning read "YAML plus JSON Schema for the authored surface". That phrasing implies an answer to the first question, and the implied answer is wrong. [Spec 2](../spec/02-taxonomy-model.md) already carries references that no JSON Schema keyword resolves: `$vocabularies.lifecycle_state`, `$package.optional.forbids`, and every dotted overlay address. JSON Schema checks that a reference has the shape of one. Only the engine checks that it points at something. The authored surface was thus always YAML plus a Headwater resolver.

This is the same finding that Q13 reached from the other end. Q13 found that LinkML and SHACL validate one instance against a shape. Q2 finds that JSON Schema validates one document against a shape. Two questions, opposite directions, one architecture.

**Why the typed configuration languages lost.** CUE unifies, and unification only narrows. It expresses `add` well, and it expresses neither `override` nor `remove`. The workaround is a default inside a disjunction, which makes the publisher pre-enumerate every value that an adopter might choose. That is premature commitment, placed on the party least able to carry it. Dhall does express override, and an overlay there becomes a function. But spec 2 requires a static confluence check over the paths that each overlay addresses. No reader can take that address set off a function that branches on its input.

**What the walkthrough derived rather than assumed.** A statically checkable confluence property needs the address set of an overlay to be readable off its syntax. That leaves a first-order path-addressed patch language as the only available shape, which is the shape that spec 2 already sketched. The overlay design is now derived, not preferred.

## Decision

YAML 1.2 is the concrete syntax. The schema language, the reference sublanguage, the overlay language and the meta-schema are Headwater's. JSON Schema is an emitted export and never the validator.

## Consequences

**Loader rulings.** YAML 1.2 core schema, so `no` stays the string `no`. Duplicate keys are an error. Anchors, aliases and merge keys are forbidden in taxonomy sources. The `$`-reference is the sanctioned reuse mechanism, and an alias is a second one that no overlay can address. Scalar types come from the meta-schema and never from the YAML resolver.

**Explicit tags, ruled after the loader met the case.** An explicit tag is forbidden, on the argument that the alias ruling already makes. A tag declares a type, and the meta-schema is the authority on type. The engine derived the rule and this record accepts it. A loader can relax a rule later at no cost. It cannot add one later without a finding against every source that already used the form.

**The implementation cost is known, and the spike retired it.** [Q1](0001-implementation-language.md) closed on Rust, where the YAML crate ecosystem is in poor repair. That matters less than it looks, because spec 12 requires source spans that no convenient deserializer supplies. The engine writes its own parse either way. The spike built one for front matter, and a taxonomy file is the same problem ([results](../evaluations/language-spike-results.md)).

**The lock.** The lock is generated, so authorability is not one of its criteria. "A stricter representation" thus means a canonical serialization rather than a typed language. The lock is JSON with sorted keys, one spelling per value, every reference resolved, and a hash over the bytes.

**CUE as an optional front-end.** Yes, outside the engine and in one direction. An organization writes CUE, runs its own build, and commits generated Headwater YAML. The engine never reads CUE. This costs nothing, because it follows from the format being ordinary YAML.

The trade that the open-question entry named still holds, and the decision does not soften it. **Overlays deliberately trade viscosity for hidden dependencies.** Customization by overlay makes change cheap, at the cost of a resolved result that nobody authored directly. Thus `explain`, `resolve`, and a readable lock file are not conveniences here. They are the mitigation.

**What it found on the way.** The walkthrough found five defects in [spec 2](../spec/02-taxonomy-model.md) that no notation fixes, and all five are now applied. The kind-to-relation permission was declared twice, with nothing to make the two directions agree. `abstract` appeared in the meta-schema with no semantics. Reading precedence had no clause for the governance family. Compatibility had no dimension for the overlay address surface. And a major version migrated documents but never overlays. The [walkthrough](../evaluations/schema-format-walkthrough.md#consequences-for-the-specification) records where each one landed.

**A note on the method.** The open-question entry predicted that viscosity and hidden dependencies would decide the question. They did not. All three candidates scored near-equal on both, because most of the viscosity lives in the model rather than in the notation. Premature commitment, abstraction gradient and a new dimension for machine authors decided it instead. The framework earned its place by contradiction of the prediction that chose it.
