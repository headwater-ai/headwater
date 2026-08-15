---
id: HW-OBL-0046
title: "The identifier scheme grammar"
status: current
status_since: 2026-08-12
last_verified: 2026-08-15
summary: "A scheme holds the prefix that discriminates a kind as a literal inside a pattern string, so no declaration names the prefix that a document IRI would carry."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-taxonomy-model
    - HW-SPEC-authoring-and-lifecycle
---

# The identifier scheme grammar

## Context

The same review closed the namespace gap in place, so `taxonomy validate` refuses a resolved scheme that carries no namespace. `core` requires nothing of the kind and can require nothing of the kind. The identifier form of a core requirement names a scheme rather than a property of one. What stays open is the shape of a scheme: the prefix that discriminates the kind is a literal inside the `pattern` string.

## Obligation

So no member of a scheme names the prefix. `pattern` holds it as a run of literal characters between the namespace and the local part.

Disjointness is not what this costs. [Identifier integrity](../spec/02-taxonomy-model.md#the-thirteen-declarations) reads each pattern into its segments and compares the two sets of strings, which is exact. The reader is `headwater_meta::identifier`, and both `taxonomy validate` and the generated check take a pattern through it.

What the missing member costs is a name. An RDF or SKOS projection has no declared part to build a document IRI out of. The [worked example](../evaluations/owl-skos-worked-example.md#the-method) derives an IRI from a repository path instead, and a rename of the file breaks every reference to it. A grammar that declares the prefix, the namespace and the local part as separate members gives that projection the parts it needs.

## Discharge

A meta-schema that declares `prefix`, `namespace` and the local-part form as three members, and nine schemes that write them, discharge this. The rendered form of every identifier stays as it is, which `engine/crates/check/fixtures/identifiers.mint` records scheme by scheme.

This record stays at `current` and it does not reach `discharged`. Half of what it asks for is met. `taxonomy validate` decides disjointness out of the parsed segments, so `identifier integrity` is decided in whole. The half that stays open is the name, and [#217](https://github.com/headwater-ai/headwater/issues/217) tracks it.

The meta-schema has shipped at `engine/crates/meta/meta-schema.yml`. So the change costs a major version of the meta-schema and of the package that reads it. The cost buys a projection that nobody has asked for yet. [Q13](../spec/09-decisions.md#q13--linkml-and-shacl-as-substrate) puts each unbuilt format behind a named external consumer, and none exists. [Spec 3](../spec/03-authoring-and-lifecycle.md#identifiers) is where the grammar belongs.
