---
id: HW-DR-0013
title: Q13 — LinkML and SHACL as substrate
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: Headwater owns the language, emitters never chain, and LinkML is the last of six siblings.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-EVAL-graph-export-and-federation
    - HW-EVAL-linkml-worked-example
    - HW-EVAL-shacl-worked-example
---

# Q13 — LinkML and SHACL as substrate

## Context

One [evaluation](../evaluations/graph-export-and-federation.md) settles this with [Q6](0006-where-the-corpus-graph-lives-at-rest.md) and [Q9](0009-multi-repository-corpora.md). Two worked examples supply the substance underneath, on [LinkML](../evaluations/linkml-worked-example.md) and on [SHACL](../evaluations/shacl-worked-example.md).

**What the worked examples established, and what does not change.** LinkML already ships three of the twenty research-derived changes: SKOS mapping slots, PROV `slot_uri` alignment, and `recommended` as advisory severity. Its `designates_type` is our heterogeneous-shelf discriminator. The boundary is not structural against governance. LinkML, SHACL and JSON Schema all validate one instance against a shape. Everything else that Headwater does is a property of the whole graph, or of the corpus over time. SHACL reaches the graph layer that LinkML cannot, and every interesting constraint there is embedded SPARQL that nobody reads because a generator wrote it.

Two objections recorded here were withdrawn, and three survive. The error-message objection fell to `sh:message` interpolation, and the SPARQL-engine objection fell to embeddable Rust engines. Line numbers, remediation and fixability still do not survive the RDF round trip, and [spec 4](../spec/04-assurance-model.md#findings) needs all three.

## Decision

The ownership half of the leaning survives untouched, and the architecture around it does not. Headwater owns the language, and standard formats come out of it. But LinkML is not the substrate, and it is not the compiler either. It is the last of six sibling emitters, and the one with the weakest case.

**Emitters never chain, and that is now measured rather than feared.** The open-question entry already named the chaining trap, and framed it as a loss of the graph layer. The stronger fact sits inside LinkML's own shape layer. Its SHACL generator does not translate `any_of` or `equals_string_in`, which LinkML itself expresses ([spec 11 §N.5](../spec/11-adjacent-work.md#n5-spdx-30-and-what-linkmls-own-generator-drops)). A chained pipeline inherits every loss of every hop and declares none of them. So every emitter reads the resolved lock and the graph directly.

**The staging order, and why LinkML is last.**

| Order | Emitter | Ships when | Consumer |
|---|---|---|---|
| 1 | JSON Schema for front matter | first release | the adopter's own editor, through a language server |
| 2 | Native graph JSON | first release | the solution corpus of [Q9](0009-multi-repository-corpora.md), and any local tool |
| 3 | SHACL | a named external consumer asks | a validation stack with no Headwater installation |
| 4 | RDF and SKOS | the same trigger | knowledge-organization tooling |
| 5 | OKF | the same trigger | LeanCTX, and whatever reads its bundles |
| 6 | LinkML | the same trigger | LinkML's own fan-out to OWL, Pydantic and SQL |

## Consequences

Only the first two pay off with no external adopter, and that is the whole reason for their position. The title of the open-question entry is what hid the answer. It named LinkML and SHACL together and put LinkML first, which made a pipeline look natural. Once emitters may not chain, LinkML stops being the route to anything else. It becomes a sibling whose distinctive value is a fan-out that nobody has asked for.

SPDX 3.0 is the observed application of the model-first half, at standards scale. One model yields an OWL ontology with SHACL restrictions, a JSON-LD context, and a JSON Schema, all derived rather than authored in parallel.

**`exportable_as` is a set, with a partition rule and an equivalence bar.** [Spec 12](../spec/12-check-layer.md#exportable_as-is-a-set-with-a-partition-rule) owns the rules. A check declares a set of emitter targets, and `none` is the common value. The exported and unexported sets partition the check registry, and the engine generates both. A target may appear only when the emitted constraint catches exactly what the native check catches, which a differential test establishes. The declaration travels with the artifact, because a consumer who copies an export copies its limits too. OGC API and STAC ship that convention already ([spec 11 §N.6](../spec/11-adjacent-work.md#n6-ogc-api-and-stac--the-subset-declaration-shipped)).

**The export is for three things, and the list is unchanged.** External validation to a declared depth. LinkML's generator fan-out, of which JSON Schema is the immediately useful part and now arrives without LinkML. And a differential-testing oracle, which is no longer an extra. It is the admission test for any claim of coverage.

**The engine still never runs on SHACL's validation machinery**, and the disqualifications hold for any authoring surface. SHACL defines conformance as "no validation results" and has no notion of completeness. Nothing in SHACL checks that a projection represents the corpus. Source positions and fixability do not survive the round trip. Document-body, corpus-scope and temporal checks never reach the graph at all.

**The OKF half is not a separate question, and the open-question entry misfiled it.** Q13 separated OKF from the substrate question correctly. One is about the TBox and the other about the ABox, and to conflate them imports weight that the smaller decision does not carry. What it failed to notice is that the native graph export is an ABox emitter too, and [Q6](0006-where-the-corpus-graph-lives-at-rest.md) owns it. So OKF is a second ABox emitter beside the native one, on identical terms. It reads the graph, declares a loss set, and emits a census.

That placement makes the standing worry concrete rather than hypothetical. OKF's own conformance check is four advisory warnings, so a consumer that validates a bundle verifies almost nothing about it. The census is the answer, because it is the emitter's own account of what it dropped, checked where the emitter runs. OKF carries unrecognized front-matter keys through a parse-emit cycle untouched, so Headwater facets ride along under a `headwater_*` prefix and the loss set stays small.

**Counter-evidence, still standing.** [OpenGEO](../spec/11-adjacent-work.md#e-opengeo--same-substrate-opposite-direction) declines RDF, OWL and SHACL for a neighboring problem. [TrustGraph](../spec/11-adjacent-work.md#j-trustgraph--the-same-pitch-the-opposite-mechanism) chose the whole standards stack and ships it. The staging order is what respects both. Nothing standards-based is refused, and nothing is built before a consumer exists.
