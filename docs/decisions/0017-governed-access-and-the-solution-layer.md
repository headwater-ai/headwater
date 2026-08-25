---
id: HW-DR-0017
title: Q17 — Governed access and the solution layer
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: The serving boundary is the export step of each publishing corpus. A profile filters for an audience, and Headwater has no principals.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-EVAL-the-serving-boundary
---

# Q17 — Governed access and the solution layer

## Context

One [evaluation](../evaluations/the-serving-boundary.md) settles this with [Q14](0014-discovery-surface.md) and [Q7](0007-scope-of-the-mcp-surface.md). The constraint that [Q9](0009-multi-repository-corpora.md) handed this decision changed its answer rather than confirming it.

**The three proposals still separate cleanly, and two of the three verdicts stand.**

| Proposal | Verdict |
|---|---|
| A cross-repository **solution layer** | Yes — [Q9](0009-multi-repository-corpora.md#the-aggregator-authors-its-own-facts)'s aggregator, extended to author its own facts |
| **Access control** over it | Yes, and much smaller than this entry expected |
| **Graph authoritative, Markdown projected** | No — and unnecessary for either of the above |

### Why authority does not move

The case for inversion is that no one can enforce access control on files that someone already cloned. That is true, and it is the right instinct pointed at the wrong layer. Inversion costs four things that the design gets free. The low capture cost of spec 3. Review on a pull request, where documents diff. The detectability of a projector defect against the Markdown. And provenance from git. Against that it buys nothing that the ruling below does not give.

There is a coherent version of the proposal. Name it, so that no one adopts it by accident. An organization for whom a repository clone is *itself* the leak wants documentation that is never committed to the repository at all. That is a real market. It also abandons "documents are files in the repository, next to the code they describe". That is [spec 0](../spec/00-vision-and-scope.md)'s central bet, and the reason that capture is cheap. It is a **pivot, not an extension**.

## Decision

### The boundary is the export step, not the tier

The open-question entry placed enforcement at the federated layer: "the federated graph is a filtered view, and the filtering happens there". That is one tier too far out, on the entry's own argument. A harvesting tier holds pinned, committed exports, so a filter that the tier applies acts on bytes that already crossed the boundary. That is [Serena's failure](../evaluations/adjacent-work.md#l6-a-filter-in-the-tool-layer-is-advisory-and-the-documentation-says-so) at one remove, and the entry diagnosed that failure and then reproduced it.

So the serving boundary is the **export step of each publishing corpus** ([spec 6](../spec/06-engine-architecture.md#an-export-profile-carries-a-filter)). A corpus decides what leaves it, and what reaches a tier is already what that tier may hold.

### The unit is a destination, and there are no principals

A filter that runs at export runs when nobody is reading. There is no request, no session, and no reader to identify. So a corpus filters for an **audience** and never for a person, and Headwater has no principals.

That is the model with an enforcement story rather than a limitation accepted reluctantly. The bytes of a filtered export live in a repository. The platform's permissions on that repository decide who reads them, exactly as they do for the Markdown. Two permission systems become one, which is what the entry asked for and could not reach while it imagined a filter at request time. The identity branch closes with it. Capability systems and centralized authorization services answer whether a principal may act on an object now, and Headwater never asks that ([HW-EVAL-adjacent-work §O](../evaluations/adjacent-work.md#o--the-serving-boundary-descriptors-redaction-and-the-write-path)).

### The mechanism, and what it did not need

An **export profile** names an audience, an emitter target, an output path, a filter over facet values, and a tombstone grain. It is an entry under `projections`, so the declaration count stays at thirteen. Six rules make it honest, and three already hold elsewhere. Carried and withheld partition the corpus and the engine generates both. A withholding is a census reason. A document is withheld whole. The filter is default-deny over classes, so a later schema addition does not widen a profile that nobody re-read. Every projection inside a profile regenerates from the filtered graph. And the declaration travels with the artifact, along with the time that the export ran.

**Two of those six came from the prior art rather than from the argument.** The attenuating-credential literature records that a credential which lists what it forbids widens silently the first time its target grows an operation. And one observed case had a correct text redaction defeated by an alphabetized word index that shipped beside it. A shelf index built at full visibility is that failure, in our own artifact set.

**No facet role was needed.** The entry said that `confidentiality` is a named facet and proposed to promote it to an enforced control. [Spec 1](../spec/01-conceptual-model.md) lists it as an example of what facets express, and no role carries it. A filter that names its own facet and values inside the profile is [principle 1](../spec/00-vision-and-scope.md#design-principles) working, and the closed role registry stays closed. What survives is the entry's best observation, sharpened. A facet that a filter reads is a facet whose every change is a disclosure decision.

## Consequences

### The tension that the entry stated twice and never noticed

The entry requires a view that reports "3 documents withheld" and does not look complete. Four paragraphs later it reports that node counts leak product structure. A count of withheld documents is a node count. The tombstone that the first constraint demands is the leak that the second one reports, and the inference literature says that no design reconciles them.

So the grain is declared per profile. `sealed` gives only the fact of the filter. `counted` puts a placeholder where each withheld node would have sat, carrying the identifier of the rule that withheld it. That shape is not ours. Freedom-of-information law asks for the amount, the position, and the rule, marked at the site of the cut. It omits the marking only where that would harm the interest which the exemption protects. That is the same conditional that the inference literature reached, a century apart. A reason comes from a closed set, because free prose in a tombstone is a second channel.

**What no profile may declare is a view that presents as total.** That invariant holds under both grains because it leaks nothing, and it is what prevents the harm. The harm is an agent that traverses a filtered graph, finds nothing, and reports absence.

### Checks stay privileged and total, and one outcome is new

A check runs inside the publishing repository, over the full graph, on a runner that holds every byte. Filtering is strictly downstream, in a projection, so no configuration exists in which a check sees a partial graph. The entry's worry about "checks that run as the user who made the request" dissolves along with the request.

What is new is a third resolution outcome. An anchor whose target a profile withheld is **withheld**, not unresolved ([spec 2](../spec/02-taxonomy-model.md#behavior-at-the-limits)). Without that distinction, every filtered harvest produces a wall of dangling-anchor findings, and operators learn to ignore the class that also carries real defects.

### Why "visibility before blocking" cannot apply, derived

[Principle 4](../spec/00-vision-and-scope.md#design-principles) is right to except this, and the exception follows from an error asymmetry rather than from the subject matter. A withholding that fires wrongly is visible, cheap and reversible. A withholding that fails to fire is invisible to both promotion instruments, and it is not reversible at all. So the general rule lands in [spec 4](../spec/04-assurance-model.md#where-promotion-does-not-apply). A control walks the promotion path when both error classes are recoverable, and otherwise it ships at its final posture. A withholding rule is the one instance today, it is not suppressible, and it is not waivable.

[Principle 7](../spec/00-vision-and-scope.md#design-principles) resolves the same way. Its positional form is shorthand for a rule about cost, so an exporter that cannot evaluate its filter emits nothing and fails the run.

### Topology, stated as mitigation

Shelf and kind names leak organizational structure. Counts and edge shapes leak product structure. A hidden document with a kept inbound edge leaks its existence. None of this is fixable, and the field that spent decades on it under a larger budget reached the same verdict. `sealed` removes the count, and withholding a document's inbound edges removes the dangling reference. Neither is a guarantee. The honest instruction is the one that the entry implied and did not state. A fact whose *existence* is the secret does not belong in a corpus that is exported at all.

**Revocation is late, and the specification says how late.** A harvesting tier reads a pinned export, so a document withheld today stays in the tier's copy until the next harvest. The authorization systems that solve this in the other direction carry a freshness token on every answer for exactly this reason. A pinned export has none, so the export carries its generation time and the harvest cadence is declared. The lag is then a number rather than a surprise.

### What may be a node

**Declared anchors, and there are now two arguments.** The first is the entry's, and it is about truth. A node that asserts a service's properties takes on an obligation to stay true, and nothing in the design carries it. Its drift then reads as structural rather than editorial ([HW-EVAL-adjacent-work §A.1](../evaluations/adjacent-work.md#a1-the-solution-layer-presses-on-that-boundary)).

The second is about enforcement, and it did not exist before the ruling above. A filter has nothing to attach to on a node that carries properties. A document has facets that a predicate reads, and an anchor is carried whole or withheld whole. Declared anchors are what make a solution-layer export filterable at all ([spec 7](../spec/07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it)). Revisit only against a concrete need that the anchor form cannot meet, argued as the model change that it would be.

### What the project takes on, and what it declines

The entry says that documentation tooling with an access model is security software. Four things follow, by its account: a threat model, an audit obligation, a disclosure process, and a class of bug that nobody can fix forward. Under the destination model the first three shrink and the fourth stays whole.

Headwater authenticates nobody, holds no session, evaluates no policy at request time, issues and revokes no credential, and records no read. Those obligations stay with the platform that already discharges them for the Markdown. [Spec 0](../spec/00-vision-and-scope.md#what-we-do-not-build) now says so in the table of what we do not build. What Headwater does is generate an artifact from a declared rule and account for what it left out. That is a redaction tool, and it keeps the worst property of an authorization system. A leak cannot be fixed forward.

So the project takes on three things, and they arrive with the first filtered profile.

| Obligation | What discharges it |
|---|---|
| The exporter emits exactly the declared set | The projection census, plus a differential fixture set per profile ([spec 12](../spec/12-check-layer.md#the-correctness-roots)) |
| Somebody outside the project can report a defect | A stated coordinated-disclosure process |
| The control never ships in a state where it may be wrong for a while | The [principle 4](../spec/00-vision-and-scope.md#design-principles) exception, recorded in the register |

**The trigger is a published claim, which is more useful than a category.** One vendor's servicing criteria decide whether a report earns a security fix by asking whether it violates a **published** boundary. The same document lists what is deliberately not one. A project does not become security software by writing a filter. It becomes security software by publishing a sentence that says a boundary holds. So [spec 6](../spec/06-engine-architecture.md#what-a-filtered-export-claims-and-what-it-does-not) states one claim and five non-claims. The non-claims are the more useful half, because they turn the tombstone channel, the shape leak and the revocation lag into stated limits.

**One hole under the premise, recorded rather than answered.** Enforcement rests on the platform's repository permissions, and commits in a fork network stay reachable across that network by the platform's own account ([HW-EVAL-adjacent-work §O.12](../evaluations/adjacent-work.md#o12-the-platform-permission-that-this-design-leans-on-has-a-documented-hole)). The premise holds for the current tip of a repository that was never forked and never changed visibility. It is qualified otherwise, and no alternative placement is better.

**Sub-repository filtering is refused, not deferred.** Within one repository a clone is total, so any filter placed there controls one reading path while the bytes stay readable along another. An adopter who needs a contractor to read one shelf and not another puts the other shelf in a second repository and federates it in. The cost is real and stated. The alternative is a control that we would have to call advisory in the one place where advisory is a defect.

The entry's closing sentence survives and belongs in the specification. A filtered view that does not announce its filtering is not a partial implementation of this. It is a defect.
