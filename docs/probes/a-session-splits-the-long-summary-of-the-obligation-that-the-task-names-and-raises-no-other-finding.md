---
id: HW-PROBE-a-session-splits-the-long-summary-of-the-obligation-that-the-task-names-and-raises-no-other-finding
status: current
status_since: 2026-10-30
summary: "A session splits the 30-word summary of HW-OBL-0106 into two sentences of at most 25 words, and raises no finding the pin did not carry."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r9-shelf-layout"
title: "A session splits the long summary of the obligation that the task names and raises no other finding"
action: repair_check
moments: [review]
axes: [names_file]
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-OBL-0106
---

# A session splits the long summary of the obligation that the task names and raises no other finding

## Task

### Phrasing 1

`headwater check` advises that the summary of `docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md` is one sentence of 30 words, past the house limit of 25. Repair it without raising another finding.

### Phrasing 2

Bring the summary of `docs/obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md` within the sentence-length limit of the house language rule. Keep its meaning, and leave nothing new for the check to report.

## Target

[HW-OBL-0106](../obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md), whose `summary` is one sentence of 30 words.

## Key

`patched` over `command:r9-shelf-layout`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/nothing-else-broken.sh --check-clean` with a held file under `tools/probe/oracle-args/` that names the baseline. The baseline names this probe, scopes the oracle to the target's file, and holds the findings of that file in the sealed grading tree less the one the task asks to repair. The oracle passes when the file carries no finding, of any severity, that the baseline does not, and no strict error. So the tree as it stands fails on the one finding the baseline lacks.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r9-shelf-layout.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change splits the summary into two sentences at the comma. It clears the finding, raises none, and passes. The wrong change splits it, and the second sentence writes "no longer". The sentence-length finding clears and a change-narration finding is raised, so it fails. The grading tree as it stands fails.

**The axes.** Each phrasing names the file, so the probe carries `names_file`. No superseded or contradicted record stands in the way, so the probe does not carry `superseded_target`.

**The baseline moves with its file.** The baseline holds the findings of one file in the sealed grading tree. A change to that file on `main`, or a seal that removes a document it links, moves them, and `tools/probe/probe-oracle-fixtures.sh` then fails this probe's reference change. Re-record it with `sh tools/probe/oracles/nothing-else-broken.sh --record <baseline>` in that tree, and delete the line of the finding the task repairs.
