---
id: HW-DR-0093
status: current
status_since: 2026-09-27
summary: "explanation and process_explanation state what holds now across one domain, and draws_on reports an account whose source decision ends."
last_verified: 2026-09-27
title: "The live account of a domain is its own kind, split on the product and process line, and it draws on the records it cites"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: reconstructed
---

# The live account of a domain is its own kind, split on the product and process line, and it draws on the records it cites

## Context

A decision record answers one question once. It changes only when a successor supersedes it. No kind of this corpus stated what holds now across a whole domain. Two examples are where a CI job runs and what a release tag does. Those facts lived in comments of the workflow files and in `DEVELOPING.md`, on no shelf, and they cited no record. Issue #1005 asked for a kind that states the live account of one domain and cites the decisions that produced it.

A design session drafted the design and it is not available to this record. The Done-when of #1005 states the kinds, the purpose, the sections, the split, the relation and the shelves. This record takes each of them from that list. The list leaves two choices open, the identifier scheme and the file layout. This record gives the reason for each, so that the owner can correct them at review.

The records that make an account possible exist now. The seven CI records are HW-PD-0013 to HW-PD-0019. The release records are HW-PD-0009 to HW-PD-0012 and HW-DR-0088 to HW-DR-0091.

## Decision

**Two kinds.** `.headwater/overlay.yml` declares `explanation` and `process_explanation`. The split is the line that `decision` and `process_decision` already draw. An account of how this repository is built is true of no adopter, so it goes on a process shelf under its own kind.

**The purpose is `behavior`.** The base package declares it as "state what the system does, as it is now". Both kinds reuse it. `rationale` stays with the decision records, and an account links to them in place of a second copy of their reasons.

**Three sections.** Each kind requires `Scope`, `How it works` and `Why it is this way`, in that order.

**One relation, `draws_on`.** It goes from either kind to any `governed_document`. Its family is `evidence`, its cardinality is `many`, and the author writes each edge, the same shape as `relations.examines`. It declares `lifecycle_sensitive: true` for itself, which [HW-DR-0065](0065-a-relation-declares-lifecycle-sensitive-for-itself-and-a-core-requirement-demands-it-of-a-family.md) permits. So `lifecycle.dependency.on_terminal` reports an account on the day a record that it draws on is superseded or withdrawn. It names `author` and not `hook`, because [HW-OBL-0105](../obligations/0105-nothing-plays-the-hook-role-that-two-relations-name.md) is discharged and no relation names `hook` after headwater-standard 4.8.0.

**Two shelves.** `docs/explanations/**` holds `explanation`, and `docs/process/explanations/**` holds `process_explanation`. Each is homogeneous. `headwater generate` writes the index of each shelf, as it does for `docs/decisions/` and `docs/process/decisions/`. That generated index is the complete list of a shelf, which is the reading [HW-DR-0081](0081-the-hand-authored-decision-register-indexes-a-subset-of-the-shelf-and-stops-asserting-it-supersedes-the-complete-one.md) gives to the generated register. An account is not an index, and it does not replace one.

**A slug identifier, minted once, and no layout.** The identifier of an `explanation` is `HW-EXP-<slug>`, and of a `process_explanation` is `HW-PEXP-<slug>`. The allocation is `minted-once`, the same as `how_to_id`. Each document is the one account of its domain and not a member of a numbered sequence. The file takes its name from the title. A second account of one domain is a defect, and the collision of two slugs reports it. This choice is the one that the Done-when did not state. A `seq` scheme with a layout is the alternative, and the claim store would then cover it ([HW-DR-0057](0057-a-shelf-layout-is-the-second-half-of-what-the-identifier-claim-store-covers.md)).

**Not the documentation-site kind.** The `diataxis-site` bundle also declares an `explanation` kind on `docs/explanations/**`. This repository cannot select that bundle, for the reason the `how_to` block of the overlay gives. The bundle's kind is one of four modes of a documentation site. This kind is the live account of one domain, and it cites the records under it. This record does not settle [HW-OBL-0097](../obligations/0097-whether-the-four-modes-of-di-taxis-are-the-kind-set.md), which waits on an adopter.

## Consequences

The first document of the process shelf is [Where a CI job runs](../process/explanations/where-a-ci-job-runs.md). It draws on HW-PD-0013 to HW-PD-0019, so the count of `lifecycle.dependency.on_terminal` instances over this corpus rises by seven.

When one of those records is superseded, `headwater check` reports the account, and its author must change the account in the same change or after it. Before this record, a superseded CI record left the workflow comments and `DEVELOPING.md` stale and nothing reported them.

The product shelf, `docs/explanations/`, has no document yet. A kind on an empty shelf is declared and not proven, and the process shelf is the proof.

An author who writes an account must not state a new rule in it. A rule goes into a decision record, and the account then draws on that record. No check reads this, so the section contract and this paragraph are the whole mechanism.
