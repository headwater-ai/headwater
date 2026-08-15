---
id: HW-DR-0003
title: Q3 — How much of the default taxonomy ships in the box
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: The base package is minimal and derived from the core, and optional content ships as add-only bundles.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-EVAL-default-taxonomy-first-run
---

# Q3 — How much of the default taxonomy ships in the box

## Context

The [first-run walkthrough](../evaluations/default-taxonomy-first-run.md) wrote the candidate base package out as real YAML, then ran five adopters through their first day against it.

**The three options never competed.** The open-question entry named a minimal core, a batteries-included default, and an interview as alternatives. All three survive as layers, and each one needs the layer below it. The minimal core is the base. Batteries-included is a selection over it. The interview reaches every other selection. What does not survive is the framing that made them compete.

**Nothing new was needed to package this.** [Spec 7](../spec/07-distribution-and-federation.md#profiles-are-publisher-overlays) already ruled that a profile is a publisher overlay rather than a mechanism. An optional package is the same mechanism pointed the other way. Spec 2 removed the `profiles` declaration for this reason, and a `packages` declaration would repeat the mistake under a new name.

**The reason for a minimal base is the resolver, not adoption feel.** Add-only overlays over disjoint addresses commute, so the existing confluence check proves that every subset of bundles resolves. Only a minimal base lets every bundle stay add-only. A large base forces bundles and profiles to remove, and `remove` carries dependent-key deletion and the most failure modes. The size of the base is thus a property of the resolver.

**The leaning here failed the core.** It named decisions, standards, and guides as the small core. Those serve `rationale`, `constraint`, and `procedure`, and the [immutable core](../spec/02-taxonomy-model.md#the-immutable-core) requires `behavior`. What that list actually describes is the starter kit, which the walkthrough shows is a different artifact.

## Decision

The base package is minimal and derived from the core: two concrete kinds under one abstract kind, four facets, five relations, one anchor, two shelves. Optional content ships as **bundles**, and a bundle is a publisher overlay that only adds. The doctrine starter kit is a named bundle selection over that base, plus the prose that explains the selection. The interview composes a different selection, and it is `headwater infer` with a second evidence source ([spec 7](../spec/07-distribution-and-federation.md#the-interview)).

## Consequences

**What it found on the way.** Six defects, and three of them sit in [spec 2](../spec/02-taxonomy-model.md) with one cause between them. The smallest column of the worked example was drawn as an impression and never derived from the core beside it. So it omits the `behavior` purpose that the core requires. Its four default relations all run between decisions, so no default edge attaches the corpus to code. And two of those four have no mechanical creator, which contradicts what spec 2 claims for its own default set. All six are applied, and the [walkthrough](../evaluations/default-taxonomy-first-run.md#consequences-for-the-specification) records where each one landed.

**The license half went to [Q11](0011-license-and-distribution-posture.md), which found it a constraint rather than a preference.** An overlay resolution contains base content, so the base and the bundles may impose nothing on a derived taxonomy. Apache-2.0 satisfies it, and nothing above depended on which terms did.
