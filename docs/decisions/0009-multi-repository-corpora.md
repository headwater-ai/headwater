---
id: HW-DR-0009
title: Q9 — Multi-repository corpora
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: A repository holds one or more corpora, the tier above harvests pinned exports, and no merged graph exists.
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
---

# Q9 — Multi-repository corpora

## Context

One [evaluation](../evaluations/graph-export-and-federation.md) settles this with [Q6](0006-where-the-corpus-graph-lives-at-rest.md) and [Q13](0013-linkml-and-shacl-as-substrate.md). The cross-*taxonomy* half was already answered by declared SKOS mapping relations ([spec 2](../spec/02-taxonomy-model.md#mapping-between-taxonomies)). All three of the remaining points now close, and one of them closes by fixing a sentence.

**The monorepo half was answered all along, in the wrong words.** [Spec 1](../spec/01-conceptual-model.md#the-corpus) said that a repository has exactly one corpus. The binding constraint is not the repository. A corpus is one taxonomy and one root, and a repository holds one or more of them. A monorepo with several independent documentation sets is not one corpus under strain. It is several corpora that share a working tree, each with its own lock. A path resolves to exactly one of them, under the rule that shelf patterns already obey. Namespaced identifiers stop two corpora in one tree from colliding. Nothing in the engine changes, because the sentence named the wrong container and that was the whole defect.

## Decision

The aggregator is in scope, and it is not a component. It is a solution corpus plus one anchor kind ([spec 7](../spec/07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it)). An anchor kind is declared, and exactly one resolver owns it. That resolver reads pinned corpus exports, in the way that `code_path`'s resolver reads a source tree.

There is no merged graph, so the question of where merged graphs live dissolves. Merging *is* anchor resolution, and anchor resolution leaves nothing behind when a run ends. A merged graph is canonical for nothing, and an artifact that is canonical for nothing, that nobody reviews, and that one rebuild reproduces does not need to exist.

Query fan-out does not happen. The tier harvests. Each source corpus carries a pin: an identity, a content hash, and a location. A scheduled job fetches the export out of band and commits it, and the resolver reads the committed copy.

## Consequences

Three arguments agree with the harvest ruling, and the specification already made all three. [Spec 0](../spec/00-vision-and-scope.md#non-negotiables) forbids the network at check time. [Spec 6](../spec/06-engine-architecture.md#performance-targets) budgets 100 ms for a route query, and a fan-out across estates does not fit. And a fan-out that meets an unreachable source either fails whole or returns less with no notice. That second outcome is the silent pass that [spec 4](../spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for) forbids. So a pinned export that the tier cannot read is a finding that names the pin.

**What the prior art settled.** SPARQL's `SILENT` keyword is the documented form of the failure above. The digital-library field ran the fan-out experiment for two decades, and its aggregators harvest through tiers of intermediaries. GraphQL federation is the counter-example that proves the rule, because runtime fan-out works there on three conditions that Headwater cannot meet ([spec 11 §N.7](../spec/11-adjacent-work.md#n7-harvest-beat-fan-out-and-graphql-federation-says-why)).

### The aggregator authors its own facts

The earlier leaning called this "an aggregator that merges exported graphs". That understates it, and the omission matters when anyone tries to build one.

Some facts belong to no repository. Examples: two services share an interface. One system's failure mode is another system's operating assumption. A capability is implemented across four estates and owned by none of them. These are claims about the *space between* repositories. No repository's export can carry them, and an aggregator that only merges can never state one.

So the federation layer is not a merge target at all. It is a **corpus at a higher altitude**: it authors the facts that genuinely live there, and it reads exports for everything else. When that is admitted, the model is unchanged rather than strained. The solution layer is one more corpus that happens to be about other corpora. It has a taxonomy, its documents are Markdown, and its cross-estate edges are declared in front matter like any other.

This also decides where authority sits, in the terms that [principle 2](../spec/00-vision-and-scope.md#design-principles) already sets: one source of truth **per fact**, not per store. A document's content is canonical in its own repository. A cross-estate edge is canonical in the solution corpus that declares it. The merged graph is canonical for nothing, and the ruling above removes it entirely. The question "is the graph or the Markdown authoritative?" has no answer because it is the wrong question — nothing is authoritative *as a store*.

Backstage is this shape in production. Its catalog re-derives entities from descriptors that live beside the code, and it generates the `relations` field rather than accepting one. It also admits entities registered as static configuration ([spec 11 §N.3](../spec/11-adjacent-work.md#n3-backstage--the-catalog-is-a-read-model-that-authors-its-own-entries)).

### What this fixed for Q17

The harvest ruling constrained [Q17](0017-governed-access-and-the-solution-layer.md) in three ways, and Q17 has since closed on them.

The export is the serving artifact, so a filter acts at export and never at graph build. Checks therefore stay privileged and total, which Q17 required and could not point at. The projection census is the mechanism for Q17's tombstone rule. A redaction is a loss with a reason, and the census already reports that shape. And a harvesting tier holds bytes that a publishing corpus gave it. A filter applied when the tier *reads* is a filter applied after the bytes crossed the boundary. Filtering belongs to the publishing corpus's export step. That third constraint changed Q17's answer rather than confirming it, because Q17 had placed the boundary at the tier.
