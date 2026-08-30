---
id: HW-DR-0044
status: current
status_since: 2026-08-31
summary: "Design-spec's evaluation kind and its two purposes move to a new evidence-and-obligation bundle, so decision-record stops requiring the whole specification tradition."
last_verified: 2026-08-31
title: "Q44 — Whether bundles decompose into capabilities and assemblies compose practices"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: [codex, claude-sonnet-5]
  accepted_by: j.baxter
  activity: inspect+draft+revise
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0003
    - HW-DR-0040
    - HW-SPEC-distribution-and-federation
    - HW-OBL-0040
    - HW-EVAL-capability-bundle-address-inventory
---

# Q44 — Whether bundles decompose into capabilities and assemblies compose practices

## Context

**The library still packages whole traditions as bundles.** `design-spec` carries specifications, registers, evaluations, and reviews. `decision-record` changes the base decision kind and adds obligation records. `standards-spec` and `brd-prd` each also carry several kinds, shelves, relations, and identifier schemes.

**The `decision-record` dependency exposes a poor boundary.** It requires `design-spec` only because that bundle declares the `obligation` purpose. A team that uses decision records therefore selects a numbered-specification tradition that it may not use. The library doctrine records this cost and the `requires` label does not expand the consumer selection.

**Assemblies now separate a named practice from its sources.** The assembly implementation resolves one base package, an explicit bundle selection, and optional cross-bundle glue. It emits a flattened package for a batteries-included consumer. It also preserves the composer path for a consumer that selects capabilities directly.

**The implementation proves the mechanism and does not establish a new module map.** Commits `eb74b21` through `c0e0263` test the recipe reader, resolver, flattening, publication, and consumer path. They use temporary fixture assemblies. No adopter evidence yet identifies the capability boundaries that a published library should carry.

## Decision

**The library separates capability bundles from practice assemblies, as a direction.** The `design-spec`/`decision-record` boundary is the first split under that direction. The [address inventory](../evaluations/capability-bundle-address-inventory.md) this decision required names the real dependency. `decision-record` reaches exactly two addresses inside `design-spec`: the `obligation` purpose and the `evaluation` kind. It never reaches the specification series, the decision register, the review pair, or the three shelves beside them. This repository's own corpus is the adopter evidence. No outside adopter exists yet, and the inventory records that gap rather than waiting on one.

**A new `evidence-and-obligation` bundle carries the two addresses that can move.** It declares `kinds.evaluation`, `purposes.evidence`, `purposes.obligation`, and the `narrative` voice regime that `evaluation` binds. `design-spec` keeps everything else. It now declares `requires: [evidence-and-obligation]` for the references that left it. `decision-record` repoints its own `requires`, from `design-spec` to `evidence-and-obligation`.

**The investigation tested the four-way split that its own Context section named, and found it does not compose.** `obligation_register`, `review_prompt`, and `review_record` each need `doc_type` or `sequence`. Both facets discriminate the two shelves that stay with `design_spec` and `decision_register`. Moving any of the three kinds would need `evidence-and-obligation` to require `design-spec` back. So would keeping `cites_evidence`'s full three-kind endpoint list, or the `evidence-cited` participation expectation, in the new bundle. Each of those needs a `requires` edge back. That edge cycles against the one `design-spec` already carries the other way, and the resolver has no reading for a cycle. This decision makes the widest cut that does not cycle under today's add-only composition rule. It is not the four-way split the investigation set out to test.

**No assembly is introduced for `design-spec`.** The one relation that would have needed one, `cites_evidence`, stays declared in `design-spec`, beside two of its three endpoint kinds. No bundle in this split needs a flattening layer to stay whole. `HW-DR-0040`'s finding holds unchanged: `requires` is a label rather than an enforced mechanism. Each of the two `requires` edges this decision adds is exactly the kind of reference that finding already describes.

**An assembly, when one is needed, never becomes a second authored taxonomy.** A flattened package stays derived from the capability selection. Publication compares its declarations against a fresh assembly resolution.

## Consequences

**`packages/headwater-standard` moves to 3.5.0.** The move adds nothing for a consumer that already selects both bundles together. `design-spec` and `decision-record` resolve to the declarations the draft of this decision found. They reach `evaluation`, `purposes.evidence`, `purposes.obligation`, and the narrative regime through one hop of `requires`, rather than by co-location. A consumer of `decision-record` alone stops resolving `design_spec`, `decision_register`, `obligation_register`, `review_prompt`, `review_record`, and the three shelves it never used.

**One participation expectation does not survive the split, and the loss is documented rather than silent.** `evaluation`'s `evidence-cited` expectation named `decision_register` as its target kind, and `decision_register` stays in `design-spec`. Declaring the expectation in the new bundle would need the cycle the Decision section states. `design-spec`'s own doctrine already called the expectation too narrow. `decision-record`'s fixture already measured it as unfixable for a corpus that keeps no register. This decision removes a declaration that finding 4 had already found wanting.

**This repository's own consumption changes with the package.** `.headwater/overlay.yml`'s `bundles:` selection now names three entries: `design-spec`, `evidence-and-obligation`, `decision-record`. `headwater check --strict` over this corpus reports the same findings it reported before the split, including the `cites_evidence` relations that `docs/spec/09-decisions.md` and `docs/spec/13-open-obligations.md` carry.

**HW-OBL-0040 stays open.** This decision resolves one named combination, by moving the two addresses that can move without a cycle. It states nothing about a future pair of bundles that need to share an address neither side can relocate. The general remedy stays unbuilt: an add-only form for cross-bundle composition, or an abstract kind wide enough to absorb an expectation like `evidence-cited`.

**The remaining work no longer fits the three bounded stages the draft named, because the first stage is done.** One question stays open from the draft. An adopter this repository does not own may select one of these bundles some day. Whether a wider or a narrower cut earns its place then is still unknown. A second question stays open beside it: whether `HW-OBL-0040`'s general form is worth building before a second capability split needs it.
