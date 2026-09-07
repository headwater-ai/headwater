---
id: HW-OBL-0175
status: current
status_since: 2026-09-07
summary: "Two design specs declaring sequence 16, whose file names both open with 16-, give check --strict exit 0 and the same finding count as the tree without them."
last_verified: 2026-09-07
title: "Two documents take one sequence on a sequence-declaring shelf and no rule reads it"
waiting_on: adopter
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-OBL-0106
    - HW-SPEC-taxonomy-model
---

# Two documents take one sequence on a sequence-declaring shelf and no rule reads it

## Context

The `spec_series` shelf declares `layout: "{sequence:02d}-{slug}.md"` in `.headwater/taxonomy.lock`, and `{sequence}` reads the `sequence` facet of the document. Four other shelves declare `{seq:04d}-{slug}.md`, where the number comes from the identifier the run mints. Both forms put a number in a file name and expect it to separate one document from its siblings.

The measurement was taken on `c574ddc`. A copy of `docs/spec/16-harness-support.md` was written to `docs/spec/16-a-second-part-at-sixteen.md` under a fresh identifier, keeping `sequence: 16`. `headwater check --strict` reported one new finding, `identifier.claim.missing`, because the claim store held no file for the new identifier. Writing that claim file took the run to exit 0, at 408 seen, 296 classified, 296 checked, 5703 check instances and 54 findings. That is the same finding count as the tree without the second document. Not one rule named the shared sequence, and not one named the two file names that both open with `16-`.

Nothing about the rule set makes that a surprise. The 30 rules this corpus instantiates hold identifiers, facet values, relations, lifecycle states, sections, links and prose. `identifier.claimed_twice` holds an identifier against the claim store. No rule of any of them reads a shelf layout, and no rule reads one facet value against the same facet on a sibling.

`docs/spec/glossary.md` states the same gap from the other side. It declares `sequence: 14` and its file name carries no number at all. The shelf that declares the layout holds a document the layout does not describe, and the run reports nothing.

Two branches cut from one `main` reach the collision without either one being wrong. Each mints its own identifier, and each reads the shelf and picks the next free sequence. The merge produces two documents at one value with no conflict. That is the shape [HW-DR-0054](../decisions/0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md) settled for identifiers, and the sequence facet has no equivalent.

## Obligation

[HW-OBL-0106](0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md) records that a shelf layout is read by the scaffolder and by nothing else. It asks a rule that re-renders each file name through the function `headwater new` writes one with. That rule reaches the `glossary.md` half of this and it does not reach the other half. Two documents at one sequence render two different file names, both of which match the layout. So this is a second obligation on the same declaration rather than an instance of that one.

The corpus owes a reading of the sequence facet across a shelf. It also owes the ruling on whether a shelf that numbers its documents requires the numbers to be distinct.

## Discharge

A rule that groups the documents of a shelf by the facet its layout reads discharges the measurement half. It reports a value that two documents hold. It needs no new declaration where the layout already names the facet, and `.headwater/taxonomy.lock` names it on all five shelves.

The ruling half asks whether a shared sequence is a defect at all. A shelf may number for sort order alone, where two documents at one value sort adjacently and nothing else follows. Where the number is an address that a citation carries, a duplicate is a broken address. The two readings need different remedies, and neither one is chosen here.
