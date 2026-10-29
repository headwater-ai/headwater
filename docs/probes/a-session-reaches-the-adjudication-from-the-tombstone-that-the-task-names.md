---
id: HW-PROBE-a-session-reaches-the-adjudication-from-the-tombstone-that-the-task-names
status: current
status_since: 2026-10-29
summary: "A task that names an anchor in the superseded open-questions register is answered only by a session that leaves the tombstone for the decision on probe cost."
last_verified: 2026-10-29
probe_category: navigability
expectation: opened
oracle: "none"
title: "A session reaches the adjudication from the tombstone that the task names"
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
  supersedes:
    - HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it
  examines:
    - HW-REG-decisions
    - HW-DR-0008
  traces_to:
    - HW-OBL-0014
---

# A session reaches the adjudication from the tombstone that the task names

## Task

### Phrasing 1

Read `docs/spec/09-open-questions.md#8--probe-cost-and-cadence` and say what this project decided about probe cost and cadence.

### Phrasing 2

The section `#8--probe-cost-and-cadence` of `docs/spec/09-open-questions.md` is about how much a probe costs and how often it runs. What did this project decide on that question? Start from that section.

### Phrasing 3

Open `docs/spec/09-open-questions.md` at the anchor `#8--probe-cost-and-cadence`. Tell me the decision this repository made about the cost of a probe and how often probes run.

## Target

The session must open [the adjudication for Q8](../decisions/0008-probe-cost-and-cadence.md). The key also accepts the decision register, whose Q8 row links the same record.

## Key

`opened` over `HW-REG-decisions` or over `HW-DR-0008`. Either one satisfies the predicate, and the transcript shows that the session left the tombstone for one of them.

**It supersedes the probe of 2026-09-06 with the same task in three phrasings** ([HW-DR-0112](../decisions/0112-the-additive-campaign-measures-eleven-actions-by-five-layers-in-seven-scenarios.md), "The form of a phrasing"). The superseded probe stays on the shelf as the record that its committed runs cite. The target, the key and the failure under test are the same.

**The axes.** Each phrasing names the file, so the probe carries `names_file`. The file it names is the superseded register, a tombstone that still writes a heading for every decision, so it carries `superseded_target` by construction. The failure this probe is for is a session that reads the heading in the tombstone and stops. Only the read set separates such an answer from an answer taken from the adjudication.

**The task names the anchor because that is the state under test.** Citations across this corpus name anchors in the tombstone file, so a reader who follows one arrives at the losing document. The reading of how many do is `grep -rho "09-open-questions.md#[a-z0-9-]*" docs | wc -l`, and no count is written here, because [HW-DR-0049](../decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md) rules that a fold over this corpus is derived.

[HW-OBL-0014](../obligations/0014-no-probe-tests-whether-an-agent-reaches-the-adjudication-from.md) records that no probe tests this. The pair here is a supersession and not an adjudicated disagreement, so a run of this probe narrows that record and does not close it.
