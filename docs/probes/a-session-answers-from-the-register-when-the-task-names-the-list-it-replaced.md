---
id: HW-PROBE-a-session-answers-from-the-register-when-the-task-names-the-list-it-replaced
status: current
status_since: 2026-10-10
summary: The task uses the title of the superseded list of open questions, so a name search opens that list first, and the read set shows whether a session avoided it.
last_verified: 2026-10-10
probe_category: navigability
expectation: not_opened
oracle: "none"
title: "A session answers from the register when the task names the list it replaced"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  examines:
    - HW-REG-open-questions
  traces_to:
    - HW-OBL-0197
---

# A session answers from the register when the task names the list it replaced

## Task

This project keeps its open questions, closed and redirected. Say what it decided on the open question of where scent lives, and name the record that decided it.

## Expectation

`not_opened` over `HW-REG-open-questions`. The session answered and never opened the superseded list of open questions.

**This probe stands beside [the register probe](a-session-answers-from-the-register-without-opening-the-question-it-replaced.md) and does not replace it (#1718).** The campaign of 2026-10-03 recorded that probe at 30 of 30 in the `present` arm and at 30 of 30 in the `absent` arm. Its task asks where scent lives, and the file name of the decision, `docs/decisions/0020-where-scent-lives.md`, holds those words. So a name search in either arm reaches the decision first, and no session had a reason to open the superseded list.

**This task uses the title of the superseded list.** `docs/spec/09-open-questions.md` carries the title `Open questions, closed and redirected`, and its file name holds `open-questions`. Both phrases are in the task. So a name search for the words of the task reaches the superseded list as early as the decision. The question is still Q20, so that the wording of the task is the one thing that differs from the original.

**This probe grades avoidance and never correctness, as the original does.** Both registers carry the Q20 line, so a right answer is evidence of nothing about the route. A session that opens the superseded list, follows its link, and answers correctly is a miss here.

**Why it should separate the arms.** In the `present` arm, a hook runs `headwater route` on the prompt and prints its pointers before the first tool call. Over this task, measured on 2026-10-10 at the commit this probe lands on, those pointers name the decision register `docs/spec/09-decisions.md` and not the superseded list, once the probe shelf is removed. The orientation skill also sends a session to `headwater explain`, which reports `status: superseded` for the list. In the `absent` arm, a session has only a search, and the words of the task name the superseded list.

**What this probe expects, stated before any paid run.** In the `present` arm, 24 of 30. In the `absent` arm, 16 of 30. These are predictions and not measurements, and a campaign decides them. If both arms reach 28 of 30 or more, the title of the list does not draw a session to it, and this probe is at the ceiling too. If the `absent` arm is the higher of the two, the layer sends a session into the list rather than past it, and that is a result too.

**A session that did nothing is not a session that avoided anything.** The grader refuses `not_opened` where the transcript records no tool call at all. So a refusal leaves the denominator and never the numerator.

[HW-OBL-0197](../obligations/0197-a-superseded-document-claims-no-reliance-and-nothing-measures-whether-a-session-honors-that.md) is discharged. This probe adds a harder reading of the claim it recorded, and it changes nothing about that record.
