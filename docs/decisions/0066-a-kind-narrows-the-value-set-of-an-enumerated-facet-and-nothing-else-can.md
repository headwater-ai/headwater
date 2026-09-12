---
id: HW-DR-0066
status: current
status_since: 2026-09-12
summary: "A kind states which values of an enumerated facet it means, under `facets.values`. A lifecycle regime, a shelf discriminator and `facets.forbid` each narrow a value set. None of the three reaches a facet with no state machine behind it."
last_verified: 2026-09-12
title: "A kind narrows the value set of an enumerated facet, and nothing else can"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-taxonomy-model
  governs:
    - engine/crates/check/src/facet_value.rs
    - engine/crates/resolve/src/rules.rs
---

# A kind narrows the value set of an enumerated facet, and nothing else can

## Context

A facet declares one value set for the whole taxonomy. `facet.value.not_permitted` reads that set and asks whether a document wrote a member of it. Until this record the only other input was the kind's own `facets.forbid`, so every kind that carried an enumerated facet admitted every value of it. A value that means something for one kind was legal on all of them.

**Three mechanisms narrow a value set today, and each one stops short of the general case.**

A **lifecycle regime** names a subset of the state vocabulary, and a kind binds one under `kinds.<name>.lifecycle`. [#219](https://github.com/headwater-ai/headwater/issues/219) closed the state facet this way and added no member to the meta-schema. The regime needs a machine: an initial value, and edges between values. No other enumerated facet of this taxonomy has one.

A **shelf discriminator** narrows a value set per kind, at census time rather than at check time. `doc_type` is the discriminator of `spec_series` and of `reviews`, and kind resolution refuses a value the shelf does not admit. The discriminator needs a bijection, because the value *is* the kind. It cannot say that one kind takes two of five values.

**`facets.forbid`** removes the whole facet from a kind. [Spec 2](../spec/02-taxonomy-model.md) already states that it takes away and adds nothing. It has no subset form.

So subset narrowing, on a facet with no machine and no shelf that discriminates on it, is expressible by nothing. Eight of the fourteen facets in this repository's resolved taxonomy declare a closed value set. One of the eight is the state facet and one is `doc_type`. Each of the other six is required by exactly one kind, so **a per-kind value set changes no verdict in this corpus**. The reader is an adopter who writes a taxonomy, and [#222](https://github.com/headwater-ai/headwater/issues/222) names that reader.

## Decision

**A kind names the values of an enumerated facet it means, under `kinds.<name>.facets.values.<facet>`, beside `require`, `optional` and `forbid`.** The member is optional, and the meta-schema moves from 0.3.0 to 0.4.0. Every source that validated before validates now, because an absent member is a kind that narrows nothing, which is the reading every kind already had.

**The shape is a member on the kind and not a `regimes.<facet>` family.** A lifecycle regime reaches a kind through a named scalar member, `kinds.<name>.lifecycle`, and `kind.members` is a closed set. A general regime family would therefore need one new member on `kind` for each enumerated facet an adopter declares. That is not a general mechanism at all. Binding the family under `facets:` as a map gives the shape this record takes, with one more indirection and one more declaration to read.

**A narrowing takes values away and it adds none, down the whole chain.** The set a kind admits is the facet's declared list, intersected with the narrowing on the kind and with the narrowing on each kind above it. A child that names a value an ancestor excluded is refused. Spec 2 rules that a child may not un-require what a parent requires, because that voids the parent's contract for a reader who trusts it. A wider value set voids the same contract in the same way.

**An overlay reaches a narrowing at `kinds.<name>.facets.values.<facet>`, under `add`, `set` and `remove`, as it reaches every other address.** Core satisfaction is a property of the resolved result. So an overlay that widens a set past what a kind above admits is refused on the result and not on the operation.

**`taxonomy validate` refuses four shapes**, all of them under the `kind inheritance` rule. A narrowing on a facet that declares no value set. A narrowing to the empty list. A narrowing on a facet the kind or an ancestor forbids. A narrowing that names a value the facet does not declare, or a value that a kind above excluded. The resolver rule set identity moves from 1 to 2, because a taxonomy this engine refused before is one it accepts now.

**This record does not decide what a warrant means.** [HW-OBL-0125](../obligations/0125-nine-documents-state-a-warrant-the-closed-set-does-not-hold-and-no-check-reads-one.md) holds nine documents at `warrant: proposed`, which is a value outside the declared set rather than a declared value on the wrong kind. The defect there is a missing reader, and this member does not supply one. Do not reach for a narrowing to close it.

[HW-DR-0035](0035-q35-whether-one-requirement-kind-holds-an-imported-requirement-and-an-authored-one.md) defers to this question twice, and its "What reopens this" clause states that a per-kind value set does not reopen Q35. That clause was read, and it stands unchanged.

## Consequences

**The check reports the set the kind admits.** The message and the remediation of `facet.value.not_permitted` name the narrowed list. A remediation that quoted the declaration would send an author to a value the kind refuses. That rule's cache version moves to 2, so no warm cache written before this change answers after it.

**A narrowed kind still generates an instance.** `facets.forbid` stops the generation step and a narrowing must not. A kind that narrows nothing is the common case. A rule that instantiated only over a declared narrowing would count no document of most corpora as checked. `crate::coverage`'s OB-COV-2 finding rests on that count.

**The exported JSON Schema narrows with it.** `headwater generate` emits one schema per kind, and the `enum` it writes for an enumerated facet now comes from that kind's admitted set. A schema wider than the check it claims to carry is a wrong answer and not less coverage. A loss set records less coverage only.

**This repository's own taxonomy declares no narrowing, and that is deliberate.** The measurement above shows that one would move no verdict here. Narrowing a value set to demonstrate the member would move the lock and every artifact below it, for no reader. The mechanism ships with a fixture corpus at `engine/crates/check/fixtures/facet-values/`, and the first adopter who needs it is the first reader of it.
