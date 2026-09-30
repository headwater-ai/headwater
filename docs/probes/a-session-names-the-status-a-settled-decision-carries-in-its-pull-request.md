---
id: HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request
status: current
status_since: 2026-09-27
summary: A session picks one of two status values for a settled decision in its pull request, and a ruling under docs/decisions decides which.
last_verified: 2026-09-27
probe_category: sufficiency
expectation: answered
oracle: "none"
title: "A session names the status a settled decision carries in its pull request"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-OBL-0223
    - HW-DR-0052
---

# A session names the status a settled decision carries in its pull request

## Task

You add a new decision record to this repository. You consider the decision settled, and you open a pull request for it. A person reads the pull request before it merges.

Which value does the `status` field in the front matter of that document carry in the pull request, `draft` or `current`? Which decision record of this repository rules it? Name that record by the identifier in its `id` field.

Answer on one line with the value, one space, and the identifier, and with nothing else. Your whole final message is that line: no sentence around it, no justification before it, and no Markdown emphasis on it.

## Expectation

The terminal answer is `current HW-DR-0052`, the one expected value of the closed set below.

```yaml
answers: [current HW-DR-0052, draft HW-DR-0052]
expected: [current HW-DR-0052]
```

The closed set holds both values that the task offers, each with the identifier of the ruling. So a recorder records `draft HW-DR-0052` as that line and not as no answer, and the grade finds it wrong. A line that names another identifier, or no identifier, is outside the set. The recorder records it as no answer, and the grade finds it not satisfied. `headwater probe plan` refuses a probe whose expected values are every value of its set, because every answer would then pass (#1229).

**The convention is [HW-DR-0052](../decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md).** An author writes the state that the document will hold once the branch lands, and the merge activates it. `headwater new` writes `draft`, because the lifecycle regime opens at that state.

**Why the task asks for the identifier.** The first task of this probe asked for the value alone, and the model answered it from what it already knew. The campaign of 2026-09-28 recorded 30 sessions in each of three arms. Those transcripts are now `deprecated`, because this change moved the read set of their probes, so the figures below are the reason for the change and not a current measurement. The present arm of the `campaign` tier answered `current` in 21 of 30. The absent arm of that tier answered `current` in 30 of 30, and the absent arm of the `documentation` tier in 28 of 30. None of the 90 sessions made a tool call. So a session that read nothing still gave the expected value, and the probe did not measure the documents. The identifier of the ruling is a fact that only this repository holds. In the `campaign` present arm, a session gets it without a read, because the always-loaded description of the `headwater-authoring` skill names HW-DR-0052 and `status: current`. On 2026-09-30, 30 of 30 present-arm sessions answered with no tool call. The absent arms have no `.claude/`, so a session there gets the identifier only when it reads the ruling or a file that names it.

**Why the probe does not grade `opened` on the ruling.** An `opened` expectation names the ruling through `examines`, and the `documentation` tier removes `docs/`. So `headwater probe plan --tier documentation` refuses a probe whose `examines` target is under an ablation entry. That grade would keep the probe in the `campaign` tier alone and lose the comparison between the two absent arms that [HW-OBL-0223](../obligations/0223-no-run-measures-whether-the-documents-under-docs-change-what-an-agent-builds.md) needs. An identifier in the closed set keeps the probe `answered`, and both tiers admit it.

**This probe is for the `documentation` tier.** Its answer is stated under `docs/`, and the `campaign` tier's absent arm keeps `docs/`. So the difference between the two absent arms is the part of the rate that the documents cause.

**Where else the answer is written, and which arm holds it.** This document states the answer, so it is part of the instrument that `.headwater/probe.yml` names, and every arm of every tier removes it. The name of this document states no answer, because `engine/crates/census/fixtures/corpus.census` lists it and that file sits outside `docs/`. The same census fixture lists the file name of the decision, and that name states the ruling. So the `documentation` ablation removes the census fixture with `docs/`. The hand-built tutorial page `site/tutorial/index.html` states the ruling in one step, and the same ablation removes it. The authoring skill under `.claude/` also states the ruling, and both absent arms remove it.

**A hint stays in both absent arms, so the difference is a lower bound.** A test comment in `engine/crates/scaffold/tests/pipeline.rs` calls a move from `draft` to `current` the author's promotion by hand, as HW-DR-0052 has it. That is not the ruling, but a session that reads it can reach the whole expected line without a document. The identifier also appears in a list of claimed identifiers in `tools/repo/id-store-fixtures.sh`, with nothing that ties it to the status. Both files sit in both absent arms, so they lower the difference between the arms and cannot inflate it.

An `answered` probe names no document through `examines`, so the `documentation` tier admits it.

[HW-OBL-0223](../obligations/0223-no-run-measures-whether-the-documents-under-docs-change-what-an-agent-builds.md) records the unmeasured claim. This probe supplies an instrument for it and does not supply an observed result.
