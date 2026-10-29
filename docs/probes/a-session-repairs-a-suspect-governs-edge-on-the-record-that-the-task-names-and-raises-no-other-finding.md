---
id: HW-PROBE-a-session-repairs-a-suspect-governs-edge-on-the-record-that-the-task-names-and-raises-no-other-finding
status: current
status_since: 2026-10-30
summary: "A session re-verifies the governs edge of HW-DR-0049 onto .gitattributes, so its verified revision is the file's digest today, and raises no finding the pin did not carry."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r9-gitattributes"
title: "A session repairs a suspect governs edge on the record that the task names and raises no other finding"
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
    - HW-DR-0049
---

# A session repairs a suspect governs edge on the record that the task names and raises no other finding

## Task

### Phrasing 1

`headwater check` reports the `governs` edge of `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md` onto `.gitattributes` as suspect, because the file changed after the edge was verified. Re-read the file against the record and repair the finding without raising another.

### Phrasing 2

Repair the suspect-edge finding on `docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md`. The record still describes `.gitattributes` correctly. Leave the check with nothing new to report.

## Target

[HW-DR-0049](../decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md), whose `governs` entry onto `.gitattributes` names a verified revision that differs from the digest of the file today.

## Key

`patched` over `command:r9-gitattributes`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/nothing-else-broken.sh --check-clean` with a held file under `tools/probe/oracle-args/` that names the baseline. The baseline names this probe, scopes the oracle to the target's file, and holds the findings of that file in the sealed grading tree less the one the task asks to repair. The oracle passes when the file carries no finding, of any severity, that the baseline does not, and no strict error. So the tree as it stands fails on the one finding the baseline lacks.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r9-gitattributes.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change writes the digest that the finding reports into `verified_revision` and sets `last_verified` to the date. It clears the finding, raises none, and passes. The wrong change sets `last_verified` to the date and leaves the digest. The finding stays, so it fails. The grading tree as it stands fails.

**The axes.** Each phrasing names the file, so the probe carries `names_file`. No superseded or contradicted record stands in the way, so the probe does not carry `superseded_target`.

**The baseline moves with its file.** The baseline holds the findings of one file in the sealed grading tree. A change to that file on `main`, or a seal that removes a document it links, moves them, and `tools/probe/probe-oracle-fixtures.sh` then fails this probe's reference change. Re-record it with `sh tools/probe/oracles/nothing-else-broken.sh --record <baseline>` in that tree, and delete the line of the finding the task repairs.

**A deleted edge fails.** Deleting the `governs` entry also clears the finding and raises none. So the baseline holds the line `# edge: <this file> governs .gitattributes`, and the oracle fails a tree whose record does not declare that edge as `headwater explain --json` parses it. A commented edge, the edge under another relation and its text in the body each fail.
