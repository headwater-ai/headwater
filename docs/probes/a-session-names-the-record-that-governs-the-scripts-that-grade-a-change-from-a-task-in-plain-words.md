---
id: HW-PROBE-a-session-names-the-record-that-governs-the-scripts-that-grade-a-change-from-a-task-in-plain-words
status: current
status_since: 2026-10-29
summary: "A task that describes the generic oracle scripts in plain words is answered right only by a session that finds them and reads the record that governs them, HW-DR-0112."
last_verified: 2026-10-29
probe_category: navigability
expectation: answered
oracle: "none"
title: "A session names the record that governs the scripts that grade a change from a task in plain words"
action: ask_or_locate
moments: [intent]
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-DR-0112
---

# A session names the record that governs the scripts that grade a change from a task in plain words

## Task

### Phrasing 1

This repository has a set of shared shell scripts that decide whether a change an agent made to the repository passes its task, such as a script that checks that a new document was written at the right place. Which decision record governs those scripts?

Answer with the identifier of that decision record, in the form `HW-DR-0000`, or with `none` if no record governs them. Your whole final message is that one value.

### Phrasing 2

Some scripts in this repository grade the result of an agent's change: one of them passes when two documents link each other, another when a document moved and every link to it was repaired. Name the decision record that rules those scripts.

Reply with only its identifier, such as `HW-DR-0000`, or with only `none` when nothing governs them.

### Phrasing 3

Find the generic grading scripts that this repository runs over an agent's finished change to say pass or fail. What is the identifier of the decision that governs them?

Give the identifier alone as your final message, in the shape `HW-DR-0000`, or the word `none`.

## Target

[HW-DR-0112](../decisions/0112-the-additive-campaign-measures-eleven-actions-by-five-layers-in-seven-scenarios.md), "The form of an oracle", and its `governs` edge on the directory of the generic oracle scripts. A route over any one of the scripts names it.

## Key

The terminal answer is one value from the closed set below, and only the expected value satisfies the probe.

```yaml
answers: [HW-DR-0112, HW-DR-0076, HW-DR-0008, none]
expected: [HW-DR-0112]
```

**Each wrong value is one a session meets.** HW-DR-0076 governs `.headwater/probe.yml`, the file that declares which script each probe runs. HW-DR-0008 is the decision on probe cost and cadence, which a search for a decision about probes finds first by its title. `none` is the answer of a session that found the scripts and did not ask what governs them. A generic script names no probe, so the seal keeps the scripts in every workspace, and HW-DR-0112 names no probe, so the seal keeps it too. HW-DR-0111, the superseded design of the campaign, governed scripts under `tools/probe/` too, but it names probes and the seal removes it, so it is not a value a session can meet.

**The axes.** No phrasing names a file or a directory, so the probe does not carry `names_file`. No superseded document in a sealed workspace governs the scripts, so the probe does not carry `superseded_target`.
