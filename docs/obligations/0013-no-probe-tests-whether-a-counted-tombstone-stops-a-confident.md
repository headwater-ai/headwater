---
id: HW-OBL-0013
title: "No probe tests whether a counted tombstone stops a confident report of absence"
status: current
status_since: 2026-08-10
waiting_on: build
last_verified: 2026-09-30
summary: "Q17 claims a counted tombstone stops a confident report of absence, and the recorder discarded the answer of both runs that reached one."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0017
    - HW-RUN-campaign-of-2026-09-30-campaign-tier-present-arm-sufficiency
    - HW-RUN-campaign-of-2026-09-30-campaign-tier-absent-arm-sufficiency
    - HW-RUN-campaign-of-2026-09-30-documentation-tier-absent-arm-sufficiency
---

# No probe tests whether a counted tombstone stops a confident report of absence

## Context

[Q17](../spec/09-decisions.md#q17--governed-access-and-the-solution-layer) rules how governed access and the solution layer work. A `counted` tombstone should stop an agent reporting absence with confidence.

## Obligation

A probe over a withheld answer is the instrument.

## Discharge

**The instrument now exists, and this corpus still withholds nothing.** [HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer](../probes/a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer.md) states the task and the closed answer set, and `headwater probe plan` selects it. A fixture at `engine/crates/cli/fixtures/answered-export/` proves the mechanism. A `filtered` profile withholds one document behind a counted tombstone, a `control` profile carries it, and only the control artifact states the recovery word.

The recording this record waited for landed and then went stale, twice. [The transcript of 2026-09-11](../probe-runs/regression-probe-transcript-for-2026-09-11.md) was graded for one day, and then a later change moved the lock under it. Since #1338 (PR #1448), a moved lock marks the verdicts of a result and keeps them. So [that result](../probe-results/regression-probe-transcript-for-2026-09-11.md) carries its verdict again, graded over a moved read set: `not satisfied`, because the probe now expects `absent`. That session searched the tree for the word, found it in a fixture under `engine/`, and answered `present`, reading the search and not a tombstone, because this corpus still declares no export profile.

**Two graded sessions converged on the answer, and the recorder discarded both.** [The transcript of 2026-09-16](../probe-runs/regression-probe-transcript-for-2026-09-16.md) and [the transcript of 2026-09-17](../probe-runs/regression-probe-transcript-for-2026-09-17.md) each carry a verdict of `not satisfied` here. Each result reads "the session ended with no answer". Neither session was undecided. Both searched `.headwater/overlay.yml` for an export filter, both found the `site` profile with none, and both closed on `absent`. The 2026-09-16 session set the word in bold Markdown after three paragraphs of justification. The 2026-09-17 session wrote the bare word after two sentences of justification. [Spec 15](../spec/15-the-recorder-contract.md#the-prompt-is-the-task-section-and-the-answer-is-the-final-line) rules that the derivation compares the whole final message and never a substring of one, so neither answer survived the transform. The probe now states the output form in its own task text, which is the remedy. The recordings of 2026-09-29 and 2026-09-30 below read it with that text.

The earlier account here read those two results as an undecided session. It was wrong in the direction that matters. Both sessions reached the closed set and the instrument lost the value. So neither run says whether a tombstone would have moved the answer.

**This run corrects the premise this record stated.** `.headwater/overlay.yml` declares one export profile, `site`, and this record's earlier readings said the corpus declares none. The accurate gap is narrower: the one profile this corpus serves withholds nothing, so no session that reaches it meets a counted tombstone. The wait moves from "an export profile that serves one" to a filter configuration on the profile that already exists.

**The probe now has one right answer, and it is the control reading (#1229).** Its closed set named all three words that its task offers, so the grader passed every in-domain answer. The probe now names the top-level `docs/` directory that `headwater export` serves, and expects `absent`, because that corpus serves no counted tombstone. Since #1293, the task also tells the session not to read a lower-level `docs/`. In the three sufficiency transcripts of the #980 batch, 10 of the 90 sessions of this probe answered `present`, and each of the 10 opened a fixture under `engine/` that states the recovery word. Every recording above also ran in a workspace that kept the probe shelves. The 2026-09-17 session after the corrections read the file of this probe before it answered. So each reading above predates the sealed workspace and the expected value, and none of them counts as evidence about a tombstone. This record stays open until a filter on the `site` profile serves a tombstone that the probe can meet.

**Two sealed recordings now read the probe, and both give the control reading.** [The tombstone pilot of 2026-09-29](../probe-runs/tombstone-probe-pilot-of-2026-09-29-campaign-tier-present-arm-sufficiency.md) answered `absent` in 10 of 10 sessions in each arm. The batch of 2026-09-30 answered `absent` in 29 of 30 `campaign` present-arm sessions, and `withheld` once. The `campaign` absent arm answered `absent` in 30 of 30, and the `documentation` absent arm in 30 of 30. So 0 of 60 absent-arm sessions answered `withheld` or `present`. [The evaluation of that batch](../evaluations/what-the-counterfactual-campaign-of-2026-09-30-measured-by-component.md) states the limits and the cost.

**The sealed present arm cannot reach this record.** On 2026-09-28, 23 of 30 present-arm sessions of this probe met this record through its index line, its register line or its claim file. The seal now deletes this record and every line that names it, because it names the tombstone probe. On 2026-09-30, 0 of 30 present-arm sessions made a call or got a result that names HW-OBL-0013.

**The engine can now bind a withheld identifier, and this probe still meets no tombstone.** #1309 (PR #1428) gave the resolver its first `withheld` binding, from the digests that a `counted` tombstone lists. This corpus declares no export profile that filters a document, so it serves no tombstone for a session to meet. The probe therefore stays the control reading. Its rate says that sessions do not invent a withheld answer. It says nothing about whether a tombstone stops a confident report of absence.

The corpus was not frozen between the two arms' sessions, because this repository merged changes while the batch ran. Each session read one archive of one pin, so both arms read the same bytes. This record stays open by the owner's ruling of 2026-09-30, pending [#1472](https://github.com/headwater-ai/headwater/issues/1472). The probe meets a tombstone only when an export profile filters a document.

**The design of #1472 does not discharge this record.** That issue declares the component arms of the campaign tier, a power calculation and a dry run that prices the plan. It runs no session, and the owner ruled that this record stays open until the campaign runs.
