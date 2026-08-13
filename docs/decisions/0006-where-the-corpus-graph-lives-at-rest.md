---
id: DR-repo-0006
title: Q6 — Where the corpus graph lives at rest
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: The graph never rests. Every run rebuilds it, and no derived artifact is canonical for anything.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - EVAL-HW-graph-export-and-federation
---

# Q6 — Where the corpus graph lives at rest

## Context

One [evaluation](../evaluations/graph-export-and-federation.md) settles this with [Q13](0013-linkml-and-shacl-as-substrate.md) and [Q9](0009-multi-repository-corpora.md), because the three were one question asked at three radii.

**The question was not the one that the open-question entry asked.** The entry offered four storage options and asked which one wins. But nothing in the design permits a stored graph to be authoritative for anything. [Spec 0](../spec/00-vision-and-scope.md#non-negotiables) makes the Markdown the corpus, [principle 2](../spec/00-vision-and-scope.md#design-principles) puts truth per fact rather than per store, and [Q17](0017-governed-access-and-the-solution-layer.md) refused the inversion on four grounds. So the storage question was answered before the entry was written. What stayed open was a different one. Which artifacts derive from the graph, how long does each one live, and what may each one claim?

## Decision

Every run rebuilds the graph. Three artifacts derive from it, and none of them is canonical for anything ([spec 6](../spec/06-engine-architecture.md#nothing-stores-the-graph)). The in-memory graph lives for one run. The cache lives until its inputs change, and version control ignores it. An export lives until a run regenerates it, and the taxonomy decides whether it is committed by declaring an output path for it.

## Consequences

**The cache is disposable by test.** `headwater check --no-cache` produces output byte-identical to `headwater check`. A cache that can change a verdict is a store under another name, and only the test keeps that distinction true under maintenance.

**No embedded database, and the trigger is named.** The [Q1 spike](../evaluations/language-spike-results.md) ran a warm change-scoped pass over 1,000 documents in about 2 ms, against a 200 ms budget. A named query workload that misses one of spec 6's targets reopens this, and nothing else does. The industry does not agree, and spec 6 now records the disagreement rather than omitting it.

**The largest correction: a round trip is the wrong instrument.** The leaning promised round-trip fidelity tests for an RDF view, and the same paragraph called RDF the lossy direction. Both cannot hold. A projection that is lossy by design cannot round-trip.

What replaces it is the coverage doctrine of [spec 4](../spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for), one layer out. Each emitter declares a **loss set** — the node classes, edge classes, and attributes that its target cannot carry, each with a reason. Each export run emits a **projection census**. Every node and every edge is either present in the output, or accounted for by a declared reason. An uncovered omission fails the run. The native graph export keeps its round trip, because an empty loss set is what a round trip proves.

That answers the trust problem that the [SHACL evaluation](../evaluations/shacl-worked-example.md#problem-one-everything-downstream-trusts-the-projection-and-shacl-does-not-check-it) raised. The projector was the component that everything downstream trusted and nothing could check. Now the engine checks it against the graph, with no help from any consumer.

**Two classes of export, and only one preserves fidelity.** The native export carries the property graph whole, including the edge attributes that [Q4](0004-relation-storage.md) introduced. Every interoperability export is lossy by construction. So the Q4 consequence recorded here still holds — an RDF projection reifies an attributed edge, as Wikidata reifies a statement before it attaches a qualifier. It is now one entry in RDF's declared loss set rather than a special case.

**What it found on the way.** Three findings, and two of them are about other documents. [Spec 1](../spec/01-conceptual-model.md#the-corpus) said that a repository has exactly one corpus, which named the wrong container and quietly blocked the monorepo half of Q9. The command list in [spec 6](../spec/06-engine-architecture.md#cli) never carried `headwater export`, although the pipeline diagram and [spec 2](../spec/02-taxonomy-model.md#mapping-between-taxonomies) both used it. And the projector had no check of its own, only a warning that it needed one.
