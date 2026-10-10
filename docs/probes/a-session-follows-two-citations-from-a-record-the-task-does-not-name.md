---
id: HW-PROBE-a-session-follows-two-citations-from-a-record-the-task-does-not-name
status: current
status_since: 2026-10-10
summary: The answer is three records away from a record the task describes and does not name, so a session that follows one citation and answers from the record it reaches is wrong.
last_verified: 2026-10-10
probe_category: sufficiency
expectation: answered
oracle: "none"
title: "A session follows two citations from a record the task does not name"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-OBL-0014
---

# A session follows two citations from a record the task does not name

## Task

A decision record of this repository that is in force today lets the recorded terminal session on the project's front page show a number that no live run produces, if a date marker stands beside it. That record replaced an earlier decision record. The earlier record rests on a third decision record, which it names in its summary. The third record is not in force today. Which decision record is in force in its place?

Answer on one line with the identifier of that record, in the form `HW-DR-NNNN`, and with nothing else. Your whole final message is that identifier.

## Expectation

The terminal answer is `HW-DR-0097`, the one expected value of the closed set below.

```yaml
answers: [HW-DR-0039, HW-DR-0061, HW-DR-0078, HW-DR-0097]
expected: [HW-DR-0097]
```

**This probe stands beside [the citation probe](a-session-follows-a-citation-from-one-record-to-the-next.md) and does not replace it (#1718).** The campaign of 2026-10-03 recorded that probe at 30 of 30 in the `present` arm and at 30 of 30 in the `absent` arm. So it measured no difference that the layer makes. Its task names the first record by identifier, and the answer is one citation and one `superseded_by` edge away from it. A session in either arm opens the named record, follows its one citation, and reads the state of the record it reaches. That route is short enough that no arm misses it.

**The answer is four documents away, and the task names none of them.** The chain is:

1. [HW-DR-0078](../decisions/0078-a-recorded-terminal-demonstration-may-show-a-frozen-number-behind-a-recorded-on-date-marker.md) is the record the task describes. It is `current`, and its `supersedes` edge names HW-DR-0061. It names no other decision record.
2. [HW-DR-0061](../decisions/0061-q61-how-a-recorded-terminal-demonstration-is-held-against-a-run.md) is `superseded`, and its `superseded_by` edge names HW-DR-0078. Its summary names HW-DR-0039.
3. [HW-DR-0039](../decisions/0039-q39-how-a-figure-reaches-a-hand-built-page.md) is `superseded`, and its `superseded_by` edge names HW-DR-0097.
4. [HW-DR-0097](../decisions/0097-a-figure-on-a-hand-built-page-is-measured-when-the-site-is-published-and-the-committed-page-carries-none.md) is `current`.

**Each wrong value is where a shorter route stops.** A session that follows one citation from HW-DR-0078 reaches HW-DR-0061 and finds it not in force. If it answers with the record in force in its place, which is the route that passed the original probe, it answers `HW-DR-0078`. A session that answers with the record HW-DR-0061 names, and does not read its state, answers `HW-DR-0039`. A session that answers with the record HW-DR-0078 replaced answers `HW-DR-0061`. All three are values of the closed set, so the grade finds each one wrong and not absent.

**Why it should separate the arms.** In the `present` arm, a hook runs `headwater route` on the prompt, and the orientation skill sends a session to `headwater explain` before a search of `docs/`. `headwater explain` prints every edge in and out of a record, with the summary of each target, and it marks the successor of a superseded record as the one that governs the reading. So in that arm each step of the chain is one call. In the `absent` arm, a session must read the front matter of each record and join the edges itself, and it must do that three times. Each step is a place to stop early.

**What this probe expects, stated before any paid run.** In the `present` arm, 24 of 30. In the `absent` arm, 18 of 30. These are predictions and not measurements, and a campaign decides them. If both arms reach 28 of 30 or more, this probe is at the ceiling too, and a longer chain is not the separator. If the `absent` arm is the higher of the two, the layer costs a session on this task, and that is a result too.

**An index holds the pieces and does not join them.** The generated index `docs/decisions/README.md` lists each of the four records with its state. A session that reads the index alone must still join three edges from the summaries, and no summary there names HW-DR-0097 beside HW-DR-0061.

**The original probe and this one share HW-DR-0097 as their answer.** A campaign runs each session alone, and every arm removes both probe documents as part of the instrument. So one probe does not give the other away.

[HW-OBL-0014](../obligations/0014-no-probe-tests-whether-an-agent-reaches-the-adjudication-from.md) records that no probe tests whether an agent reaches a ruling from a document that does not hold it. This probe narrows that record and does not close it.
