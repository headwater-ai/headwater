---
id: HW-OBL-0205
status: discharged
status_since: 2026-09-27
summary: "HW-OBL-0002 and the voice.rs census now carry the reading of 2026-09-27, taken over the body and the summary facet that the scent-role fix #774 added."
last_verified: 2026-09-27
title: "Two sample counts, HW-OBL-0002's and voice.rs's own census, predate the scent-role fix and need retaking"
waiting_on: build
---

# Two sample counts, HW-OBL-0002's and voice.rs's own census, predate the scent-role fix and need retaking

## Context

[#774](https://github.com/headwater-ai/headwater/issues/774) widened `voice.forbidden_construction` to read the `scent`-role facet (`summary` in this corpus), which the check did not read before. Two records carry counts taken before that change, over the narrower, body-only population. `engine/crates/check/src/voice.rs` already carries its own correction, in a comment that #774 itself added. The comment names the census "edition four". It states plainly that the two denominators it reports "are now at least as large as stated". It adds that "the two zero numerators are unconfirmed over the wider set". It also states that a reader should "re-run the count rather than trust this comment for either number." [HW-OBL-0002](0002-declarative-voice-is-called-detectable-at-useful-precision.md) carries the same shape of debt and none of the same disclaimer. Its reading of 2026-09-11 on commit `58f46a8d` found 36 `change_narration` findings and zero for the other two categories. The denominators were 291 and 293 documents. That reading predates #774 by the same margin. #774 did not touch it, because #774 was scoped to the engine and not to this record.

## Obligation

HW-OBL-0002's dated reading needs the same disclaimer voice.rs already carries, at minimum. Ideally, it also needs a fresh count taken over the widened population before anyone treats its sample size or its two zero categories as current. Until a session re-runs the count, the record understates its own denominators. Until then, it also cannot say whether the two zero categories are still saturation-not-a-clean-corpus, or whether the wider population changes either reading.

## Discharge

This discharges when a session does three things. It re-runs the `future_intent`, `change_narration` and `phased_rollout` counts over the population #774 widened. It records a new dated reading in HW-OBL-0002 that names the commit. It either reaches the 50-finding adjudicated sample or restates why it still does not. HW-OBL-0002 has waited on that sample since 2026-08-11.

**The reading of 2026-09-27, on commit `0ee88a2d`, discharges this record.** It counts the body and the `summary` facet together. [HW-OBL-0002](0002-declarative-voice-is-called-detectable-at-useful-precision.md) carries it as a dated reading that names the commit, and the census in `engine/crates/check/src/voice.rs` carries the same numbers. The wider population did not change either reading of saturation. `phased_rollout` has 0 of its 13 patterns in the 382 documents it governs. `future_intent` reports one finding, and it is false. No category reaches the sample of 50, and HW-OBL-0002 states why.
