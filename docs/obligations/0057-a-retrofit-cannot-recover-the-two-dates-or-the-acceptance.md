---
id: HW-OBL-0057
title: "A retrofit cannot recover the two dates or the acceptance"
status: current
status_since: 2026-08-12
last_verified: 2026-08-13
summary: "Three retrofits have now run, and each one recovered a date only where somebody had recorded one."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0012
---

# A retrofit cannot recover the two dates or the acceptance

## Context

The typing derived `status_since` from the first commit on each path and `last_verified` from the last commit that a human merged. Neither is the fact that the facet asks for, and no earlier record of either survives.

It also wrote one `accepted_by` across 36 documents in one change. That is the bulk promotion that [spec 3](../spec/03-authoring-and-lifecycle.md#promotion-is-one-human-one-document-one-diff) says no check can separate from 36 real acceptances. Spec 3 predicts the response, which is a finding about the review rather than about the documents.

## Obligation

[Q12](../spec/09-decisions.md#q12--migration-path-for-an-existing-corpus) covers the findings that an adoption payload holds, and says nothing about these three fields. The corpus owes a statement of what a retrofit may write into a date facet and into `accepted_by`.

## Discharge

**The second retrofit is larger, and it fails in a different place.** [#124](https://github.com/headwater-ai/headwater/issues/124) converted twenty-one decisions, and it derived no date from a commit. It wrote `status_since: 2026-08-11` on all twenty-one. The register recorded all twenty-one as closed on that date, and the history agrees with the register. One commit, `e0d8467`, is the first whose version of the register reports every one of the twenty-one as closed. So the date is a true fact about the register rather than a fact about each decision. Several decisions closed before it, and no record of those earlier dates survives anywhere. `last_verified` carries the same date under the same limit. One `accepted_by` again covers twenty-one documents in one change, which is the same bulk promotion at a smaller multiple. What the second pass adds is that the loss is not an artifact of reading git. The first pass derived the wrong fact from a commit, and this pass found that the right fact was never written down at all.

**The third retrofit is this shelf, and it recovered a per-document date that the first two could not.** [#126](https://github.com/headwater-ai/headwater/issues/126) converted 101 entries of the obligation register into documents. It set no date in bulk. The date on each record is the author date of one commit. It is the earliest commit whose version of the register contains that entry's own opening words. The tombstone that held these entries before the register counts as a version of it. The dates run from 2026-08-10 to 2026-08-13, and no record carries a date that the conversion invented.

That date is the fact the facet asks for. An obligation enters the `current` state on the day somebody writes it down, and a register is the writing down. So the recoverable fact is a property of the artifact rather than of the volume. A retrofit over documents recovers nothing, because a file records when it was created and not when its subject entered a state. A retrofit over the entries of a register recovers a date for every entry, because each entry arrived in a commit of its own.

Two of the three fields stay lost at this volume. `last_verified` reads 2026-08-13 on all 101 records, which is the day one agent read every entry while converting it. That is a true bulk verification and not a per-entry fact. `accepted_by` reads one name across 101 documents in one change, which is the same bulk promotion at five times the multiple of the second pass. The acceptance that exists is the register's, and [spec 3](../spec/03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed) is right that no check separates it from 101 acceptances.
