---
id: HW-OBL-0016
title: "A cue has no measured effect on traversal precision"
status: current
status_since: 2026-08-11
waiting_on: measurement
last_verified: 2026-09-30
summary: "Q20 claims that a cue raises traversal precision over the summary fallback, and no campaign has graded one."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - to: HW-DR-0020
      cue: "The ruling that lets a hand-written cue exist at all, and the reason none has been authored to grade yet."
    - HW-RUN-campaign-of-2026-09-30-campaign-tier-present-arm-navigability
    - HW-RUN-campaign-of-2026-09-30-campaign-tier-absent-arm-navigability
---

# A cue has no measured effect on traversal precision

## Context

[Q20](../spec/09-decisions.md#q20--where-scent-lives) rules where scent lives and gives the referring end the cue. A cue should raise traversal precision over the summary fallback.

## Obligation

A paired campaign over one corpus is the instrument.

## Discharge

When this record was written, no campaign had run, and this corpus authored no cue. If a cue does not raise precision, it is authoring cost with no scent gain. The honest response is then to remove it rather than to make it longer.

**A campaign has run, and it did not grade a cue.** The batch of 2026-09-30 graded the navigability selection in both arms of the `campaign` tier ([#1384](https://github.com/headwater-ai/headwater/issues/1384)). The present arm satisfied 69 of 90 (76.7%, 66.9% to 84.2%). The absent arm satisfied 68 of 90 (75.6%, 65.8% to 83.3%). The difference is +1.1 points, in a 95% Newcombe interval of -11.3 to +13.5 points ([the result](../probe-results/campaign-of-2026-09-30-campaign-tier-present-arm-navigability.md)). So the governance layer did not raise traversal precision by more than about 13 points.

**No cue is removed, because this corpus authors none for a session to follow.** The one cue in the corpus is on the edge of this record to HW-DR-0020, and no navigability probe traverses that edge. The other `cue:` lines are examples in spec 1 and spec 2 and a declaration in the base package. So the batch measured the summary fallback in both arms and not a cue. The removal that the paragraph above names has nothing to act on. For the same reason, no `headwater capture` trend is cited: no hand-written cue was removed to cost or to save.

**The reading has limits.** 8 of the 90 present-arm sessions and 0 of the 90 absent-arm sessions reached this repository from outside their workspace. That count uses the rule in [the Limits section of the evaluation](../evaluations/what-the-counterfactual-campaign-of-2026-09-30-measured-by-component.md#limits): a call names a copy of this repository, a `headwater` binary or GitHub. It does not count every path outside the workspace. The corpus was not frozen between the two arms' sessions. `docs/spec/09-open-questions.md` changed after the recording, so each verdict is graded over a moved read set. [The evaluation of that batch](../evaluations/what-the-counterfactual-campaign-of-2026-09-30-measured-by-component.md) states every limit and the cost. This record stays open by the owner's ruling of 2026-09-30, pending [#1472](https://github.com/headwater-ai/headwater/issues/1472).
