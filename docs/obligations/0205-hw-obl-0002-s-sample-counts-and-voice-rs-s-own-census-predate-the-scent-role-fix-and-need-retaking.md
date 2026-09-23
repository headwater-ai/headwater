---
id: HW-OBL-0205
status: draft
status_since: 2026-09-23
summary: "Two sample counts predate the scent-role fix #774 landed: HW-OBL-0002's, and voice.rs's own census. Neither was re-taken, so both figures now understate what the rule covers."
last_verified: 2026-09-23
title: "Two sample counts, HW-OBL-0002's and voice.rs's own census, predate the scent-role fix and need retaking"
waiting_on: build
---

# Two sample counts, HW-OBL-0002's and voice.rs's own census, predate the scent-role fix and need retaking

## Context

[#774](https://github.com/headwater-ai/headwater/issues/774) widened `voice.forbidden_construction` to read the `scent`-role facet (`summary` in this corpus), which the check did not read before. Two records carry counts taken before that change, over the narrower, body-only population. `engine/crates/check/src/voice.rs` already carries its own correction: a comment added by #774 itself, naming the census "edition four" and stating plainly that the two denominators it reports "are now at least as large as stated and the two zero numerators are unconfirmed over the wider set," and that a reader should "re-run the count rather than trust this comment for either number." [HW-OBL-0002](0002-declarative-voice-is-called-detectable-at-useful-precision.md) carries the same shape of debt and none of the same disclaimer. Its reading of 2026-09-11 on commit `58f46a8d` — 36 `change_narration` findings, zero for the other two categories, against denominators of 291 and 293 documents — predates #774 by the same margin and was not touched by it, because #774 was scoped to the engine and not to this record.

## Obligation

HW-OBL-0002's dated reading needs the same disclaimer voice.rs already carries, at minimum, and ideally a fresh count taken over the widened population before anyone treats its sample size or its two zero categories as current. Until a session re-runs the count, the record understates its own denominators and cannot say whether the two zero categories are still saturation-not-a-clean-corpus, or whether the wider population changes either reading.

## Discharge

This discharges when a session re-runs the `future_intent`, `change_narration` and `phased_rollout` counts over the population #774 widened, records a new dated reading in HW-OBL-0002 naming the commit, and either reaches the 50-finding adjudicated sample this record has waited on since 2026-08-11 or restates why it still does not.
