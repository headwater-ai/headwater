---
id: HW-PROBE-a-session-supersedes-the-standing-ruling-on-a-paragraph-limit-when-the-task-describes-the-ruling-that-it-replaced
status: current
status_since: 2026-10-30
summary: "A session records an eight-sentence paragraph limit as the successor of HW-DR-0087, which stands today, and not of HW-DR-0069, the sentence-count ruling the task describes, which HW-DR-0087 superseded."
last_verified: 2026-10-30
probe_category: navigability
expectation: patched
oracle: "command:r6-paragraph-limit"
title: "A session supersedes the standing ruling on a paragraph limit when the task describes the ruling that it replaced"
action: supersede_or_retire
moments: [write, review]
axes: [superseded_target]
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-DR-0087
---

# A session supersedes the standing ruling on a paragraph limit when the task describes the ruling that it replaced

## Task

### Phrasing 1

The owner ruled that the house language rule limits a paragraph to eight sentences, as an advisory defect. Record the ruling so that it replaces the ruling which says a paragraph limit counts sentences and never words.

### Phrasing 2

Write a decision that adds an advisory eight-sentence paragraph limit to the language rule. It replaces the earlier ruling on how a paragraph limit counts, the one that counts sentences rather than words.

## Target

[HW-DR-0087](../decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md), which superseded HW-DR-0069 and is the ruling on a paragraph limit that stands today.

## Key

`patched` over `command:r6-paragraph-limit`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/successor-typed.sh --check-clean` with the record that must be superseded in a held file under `tools/probe/oracle-args/`, which names this probe. The oracle passes when that record is `status: superseded`, a document the engine types names it under `supersedes`, the record names that successor in its `relations:` block as a whole name, and neither carries a strict error.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r6-paragraph-limit.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change writes a decision with `headwater new decision --relates supersedes=HW-DR-0087`, sets it `current`, and runs `headwater check --fix`. It passes. The wrong change writes the same decision with `supersedes=HW-DR-0069`, the ruling the task describes. HW-DR-0087 superseded it, and HW-DR-0087 stays `current`, so it fails. The grading tree as it stands fails.

**The axes.** No phrasing names a file, so the probe does not carry `names_file`. The task describes HW-DR-0069, and HW-DR-0087 superseded it, so a superseded record stands in the way, and the probe carries `superseded_target`.
