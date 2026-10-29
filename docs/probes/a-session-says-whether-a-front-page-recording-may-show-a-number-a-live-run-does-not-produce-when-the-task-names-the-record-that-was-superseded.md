---
id: HW-PROBE-a-session-says-whether-a-front-page-recording-may-show-a-number-a-live-run-does-not-produce-when-the-task-names-the-record-that-was-superseded
status: current
status_since: 2026-10-29
summary: "A task that names HW-DR-0061, which confines a recording to output a run holds, is answered right only by a session that follows the supersession to HW-DR-0078 and its dated marker."
last_verified: 2026-10-29
probe_category: sufficiency
expectation: answered
oracle: "none"
title: "A session says whether a front-page recording may show a number a live run does not produce when the task names the record that was superseded"
action: ask_or_locate
moments: [intent]
axes: [names_file, superseded_target]
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-DR-0078
---

# A session says whether a front-page recording may show a number a live run does not produce when the task names the record that was superseded

## Task

### Phrasing 1

Read `docs/decisions/0061-q61-how-a-recorded-terminal-demonstration-is-held-against-a-run.md`. Under the rules of this repository as they stand today, may a terminal recording on the front page show a number that a live run does not produce?

Answer with one word: `no` if it may not, `yes` if it may with no condition, or `marked` if it may only with a marker beside it. Write nothing else.

### Phrasing 2

`docs/decisions/0061-q61-how-a-recorded-terminal-demonstration-is-held-against-a-run.md` rules on recorded terminal demonstrations. As this repository stands now, can a front-page recording show output counts that a fresh run would not reproduce?

Your whole final message is one of three words: `no`, `yes` for yes with no condition, or `marked` for yes only beside a marker.

### Phrasing 3

Start from the decision record in `docs/decisions/` whose file name begins `0061-`. Say whether the ruling in force today lets a front-page recording show a number that differs from a live run.

Reply with `no`, `yes` (with no condition) or `marked` (only with a marker beside it), and no other text.

## Target

[HW-DR-0078](../decisions/0078-a-recorded-terminal-demonstration-may-show-a-frozen-number-behind-a-recorded-on-date-marker.md), "A front-page recording may show a number a live run does not produce, if a visible marker stands beside it", which supersedes the record that the task names.

## Key

The terminal answer is one value from the closed set below, and only the expected value satisfies the probe.

```yaml
answers: [marked, no, yes]
expected: [marked]
```

**The record that the task names gives the old answer.** HW-DR-0061 confines a recording to the tutorial's scratch corpus, where a blocking step holds every printed number against a run, so `no` is the answer of a session that read the named file and stopped. HW-DR-0078 supersedes it and lets a recording show a frozen number beside a `recorded on YYYY-MM-DD` marker, so the answer is `marked`. `yes` is the answer of a session that found the relaxation and dropped its condition.

**Why this record and not the record of the additive campaign.** HW-DR-0111, the superseded design of the additive campaign, names twenty probes by their file names, so the seal removes it from every workspace of the campaign, and a task cannot name it. HW-DR-0061 and HW-DR-0078 name no probe, and both survive the seal.

**The axes.** Each phrasing names the file, so the probe carries `names_file`. The file it names is superseded, so it carries `superseded_target`.

**Leak.** `leaks:` declares HW-DR-0078 and the phrase `recorded on`, which states the condition.
