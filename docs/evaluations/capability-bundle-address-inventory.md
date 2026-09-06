---
id: HW-EVAL-capability-bundle-address-inventory
status: current
status_since: 2026-09-06
last_verified: 2026-08-31
summary: The cross-bundle address inventory HW-DR-0044 asks for, and what it shows decision-record actually reaches inside design-spec.
title: "The capability-bundle address inventory"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-sonnet-5
  activity: inspect+draft
  evidence_basis: evidenced
---

# The capability-bundle address inventory

## The question

[HW-DR-0044](../decisions/0044-q44-whether-bundles-decompose-into-capabilities-and-assemblies-compose-practices.md) names its own Context "a poor boundary." `decision-record` declares `requires: [design-spec]` for one purpose. It inherits six kinds and three shelves along with that one purpose. Its stage-1 instruction asks for two things before the package changes. The first is an address inventory that names every declaration owner and every direct dependency. The second is adopter evidence for which capabilities are actually selected together. This document closes the first question and states what this repository's own corpus supplies for the second.

## The instrument

Read `packages/headwater-standard/bundles/*/bundle.yml` for all five bundles: `design-spec`, `decision-record`, `standards-spec`, `brd-prd`, and `diataxis`. For `decision-record`, traced every relation and kind reference back to the address it needs from `design-spec`. The `requires: [design-spec]` label does not describe that footprint on its own. Cross-checked the finding against this repository's own resolved corpus. `.headwater/overlay.yml`'s `bundles:` selection is one part of that check. Whether `docs/spec/09-decisions.md` and `docs/spec/13-open-obligations.md` exercise the relations a split would have to preserve is the other part. Both are this repository's own `decision_register` and `obligation_register` documents.

## What it found

**The address inventory.** One real dependency edge exists among the five bundles today: `decision-record → design-spec`. `standards-spec`, `brd-prd`, and `diataxis` each declare `requires: []`.

| Bundle | `requires` | Purposes declared | Concrete kinds | Shelves |
|---|---|---|---|---|
| `design-spec` | `[]` | `evidence`, `obligation` | `design_spec`, `decision_register`, `obligation_register`, `evaluation`, `review_prompt`, `review_record` | `spec_series`, `evaluations`, `reviews` |
| `decision-record` | `[design-spec]` | none (reuses `obligation`) | `obligation_record` (plus `title` added to the base `decision`) | `obligations` |
| `standards-spec` | `[]` | `constraint` | `standard`, `functional_spec`, `technical_spec` | `standards`, `component_specs` |
| `brd-prd` | `[]` | `requirement` | `brd`, `prd` | `requirements` |
| `diataxis` | `[]` | none | none (facet-only, on the base's abstract kind) | none |

**`decision-record` reaches exactly two addresses inside `design-spec`.** It never reaches the other four fifths of it. `kinds.obligation_record` declares `purpose: obligation`, reusing the address `design-spec` declares first. `relations.discharges` names `evaluation` as its `from` kind. No other declaration of `decision-record`'s own bundle, doctrine, or fixture corpus names `kinds.decision_register`, `kinds.design_spec`, `kinds.review_prompt`, `kinds.review_record`, or any of the three shelves. The fixture corpus confirms this from the outside. [Its own README](../taxonomies/decision-record/fixtures/README.md#what-a-run-reports) records that its corpus "holds no `decision_register` at all." That is why one of its three measured findings is unfixable rather than merely unplanted.

**`standards-spec` and `brd-prd` avoid the same cost on purpose, by claiming an unclaimed purpose rather than reusing one.** `standards-spec`'s doctrine states this directly. It declares `purposes.constraint`, "the second entry with an empty closure," because it "meets no address that design-spec, decision-record or diataxis writes." `brd-prd`'s doctrine records that it considered, and refused, a `requires: [standards-spec]` dependency to reach a `prd → functional_spec` edge. It cites the same [HW-OBL-0040](../obligations/0040-composition-between-two-library-entries-has-no-add-only-form.md) cost `decision-record` already paid once. Two of four possible dependents have already chosen, on their own, not to pay this cost. `decision-record` is the one bundle that had already paid it, for want of anywhere narrower to reach.

**A finer cut than the one this address inventory supports would cycle.** `design-spec`'s `cites_evidence` relation runs `from: [design_spec, decision_register, obligation_register]` to `[evaluation]`. It spans both halves of any split along the "specification series vs. evidence and obligation" line. `evaluation`'s own `evidence-cited` participation expectation names `decision_register` as its target kind. That is the same entanglement in a second declaration. `obligation_register`, `review_prompt`, and `review_record` each require `doc_type` or `sequence`. Both facets discriminate the two shelves that stay with `design_spec` and `decision_register`. Moving any of those three kinds would need whichever bundle ends up smaller to `require` the other one back. So would keeping `cites_evidence`'s full endpoint list, or the `evidence-cited` expectation, intact in the new bundle. `design-spec` already requires the new bundle for the one kind that moves cleanly. A dependency running the other way, at the same time, is not a stricter split. It is a cycle the resolver has no reading for. The only cut this repository's own corpus and fixtures support today is `evaluation`, its two purposes, and the `narrative` voice regime it needs. Neither `obligation_register` on its own, nor a four-way split into specification-series, decision-refinement, evidence-and-review, and obligation-tracking, has that support yet.

**This repository is the adopter evidence.** `.headwater/overlay.yml` selects `bundles: [design-spec, decision-record]` today. Both `docs/spec/09-decisions.md` (`decision_register`) and `docs/spec/13-open-obligations.md` (`obligation_register`) declare `cites_evidence` against a real `evaluation` document, `HW-EVAL-default-taxonomy-first-run`. A split that broke either relation would fail this repository's own strict check on the next run. It would fail before it reached any adopter this repository does not own. That is the bar the split was measured against, and not a fixture invented for the purpose.

## The decision it supports

[HW-DR-0044](../decisions/0044-q44-whether-bundles-decompose-into-capabilities-and-assemblies-compose-practices.md) rules on exactly the cut this inventory supports. A new `evidence-and-obligation` bundle carries `kinds.evaluation`, `purposes.evidence`, `purposes.obligation`, and `regimes.voice.narrative`. `design-spec` keeps everything else and adds `requires: [evidence-and-obligation]`. `decision-record` repoints its own `requires`, from `design-spec` to `evidence-and-obligation`. No assembly is introduced for `design-spec`. The one relation that would have needed one, `cites_evidence`, stays with the two kinds that still name it directly.

## What it did not settle

**Whether a narrower or wider cut earns its place once a real outside adopter exists.** This inventory reads this repository's own corpus, which selects two of five bundles. HW-DR-0044's Context names a team that keeps ADRs and never writes a specification series. No corpus like that exists yet to measure against.

**[HW-OBL-0040](../obligations/0040-composition-between-two-library-entries-has-no-add-only-form.md) stays open.** This split resolved one named combination. It moved the one address that could move without a cycle. It states nothing about a future pair of entries that need to share an address neither one can relocate.

**The `evidence-cited` participation expectation did not move with `evaluation`, and nothing replaced it.** [Design-spec's doctrine](../taxonomies/design-spec/doctrine.md#what-hw-dr-0044-moved-out-and-why-the-rest-could-not-follow) records why. The general remedy is still open: an abstract kind over the two register kinds. Design-spec's own finding 4 named that remedy before this split existed.
