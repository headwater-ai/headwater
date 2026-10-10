---
id: HW-PROBE-a-session-says-which-date-a-corrected-status-carries
status: current
status_since: 2026-10-10
summary: A session corrects the status of a settled decision that is already on main and names the date its status_since then carries, which only the body of a ruling under docs/decisions states.
last_verified: 2026-10-10
probe_category: sufficiency
expectation: answered
oracle: "none"
title: "A session says which date a corrected status carries"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-OBL-0223
    - HW-DR-0052
---

# A session says which date a corrected status carries

## Task

A decision record of this repository was settled when it merged to `main`, but it still carries `status: draft`. You correct its `status` to `current` in a pull request.

After your correction, which date does its `status_since` field carry: the date of the merge that first added it, or the date of your correction? Answer `merge` or `correction`.

When this repository made the ruling that decides this, the change that carried the ruling corrected every document on `main` that then stood at `draft`. How many documents was that?

Answer on one line with the word, one space, and the number, and with nothing else. Your whole final message is that line: no sentence around it, no justification before it, and no Markdown emphasis on it.

## Expectation

The terminal answer is `correction 66`, the one expected value of the closed set below.

```yaml
answers: [correction 66, merge 66, correction 277, merge 277]
expected: [correction 66]
```

**This probe stands beside [the status probe](a-session-names-the-status-a-settled-decision-carries-in-its-pull-request.md) and does not replace it (#1718).** The campaign of 2026-10-03 recorded that probe at 30 of 30 in the `present` arm and at 30 of 30 in the `absent` arm. In the `present` arm, the description of the internal `hw-corpus` skill states its whole answer, so `.headwater/probe.yml` keeps that leak on purpose. The summary of the ruling states the value that task asks for. So a session in either arm that found the ruling needed nothing past its summary.

**The answer is in the body of the ruling, and in no summary and no description.** [HW-DR-0052](../decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md) states, under `## Decision`, that a correction stamps `status_since` with the date of the correction. It states the count of 66 documents at `draft`, of 277 typed documents, under `## Context` and again under `## Consequences`. Its summary states neither. The description of the `hw-corpus` skill names HW-DR-0052 and states neither. So this probe keeps no leak, and `.headwater/probe.yml` declares its leak strings under `leaks:` and not under `leaks_kept:`.

**The corpus is full of examples that point to the wrong word.** `headwater new` stamps `status_since` with the day it runs, so a document proposed at the state it holds carries a date near the day it merged. A session that infers the rule from those examples answers `merge`. Many documents also share the words `status`, `draft` and `pull request` with the task. The number makes the answer one that a session cannot give from what it already knows. The value alone could be a guess, which is what the first task of the original probe recorded.

**The wrong values are the two misreadings of one paragraph.** `merge` is the answer from the examples. `277` is the other number in the sentence that states `66`. A line outside the set is recorded as no answer, and the grade finds it not satisfied.

**Why it should separate the arms.** In the `present` arm, the description of the `hw-corpus` skill names HW-DR-0052, so a session can open the ruling first and read its body. In the `absent` arm, a session must find the ruling by a search, among documents that share every word of the task, and it must then read past the summary.

**What this probe expects, stated before any paid run.** In the `present` arm, 25 of 30. In the `absent` arm, 15 of 30. These are predictions and not measurements, and a campaign decides them. If both arms reach 28 of 30 or more, the body of the ruling is as easy to reach as its summary, and this probe is at the ceiling too.

**This probe is for the `documentation` tier too.** The ruling is under `docs/`, so the `documentation` tier's `absent` arm removes it. An `answered` probe names no document through `examines`, so that tier admits it.

[HW-OBL-0223](../obligations/0223-no-run-measures-whether-the-documents-under-docs-change-what-an-agent-builds.md) records the unmeasured claim. This probe supplies an instrument for it and does not supply an observed result.
