---
id: HW-PROBE-a-session-rewrites-the-change-narration-in-the-summary-of-the-obligation-on-eight-stale-comments-and-raises-no-other-finding
status: current
status_since: 2026-10-29
summary: "A session rewrites the change narration in the summary of HW-OBL-0230 so that the voice finding clears, and raises no finding the pin did not carry."
last_verified: 2026-10-29
probe_category: sufficiency
expectation: patched
oracle: "command:r9-stale-comments"
title: "A session rewrites the change narration in the summary of the obligation on eight stale comments and raises no other finding"
action: repair_check
moments: [review]
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-OBL-0230
---

# A session rewrites the change narration in the summary of the obligation on eight stale comments and raises no other finding

## Task

### Phrasing 1

`headwater check` reports change narration in the summary of the obligation record on eight stale comments in the command-surface crates. Repair it, and raise no other finding.

### Phrasing 2

The obligation that lists eight comments in the command-surface crates that state stale facts tells in its summary what the code used to hold, and the voice rule reports it. State what holds instead, with nothing new for the check to report.

## Target

[HW-OBL-0230](../obligations/0230-comments-in-the-command-surface-crates-and-their-subsystem-spec-state-eight-stale-facts.md), whose `summary` writes a change-narration phrase that the `declarative` voice forbids. The same summary is one sentence of 29 words, and that finding stays in the baseline, because the task does not ask for it.

## Key

`patched` over `command:r9-stale-comments`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/nothing-else-broken.sh --check-clean` with a held file under `tools/probe/oracle-args/` that names the baseline. The baseline names this probe, scopes the oracle to the target's file, and holds the findings of that file in the sealed grading tree less the one the task asks to repair. The oracle passes when the file carries no finding, of any severity, that the baseline does not, and no strict error. So the tree as it stands fails on the one finding the baseline lacks.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r9-stale-comments.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change writes "state facts the code does not hold". It clears the finding, raises none, and passes. The wrong change writes "state facts the code doesn't hold". The contraction raises an error of the language rule, so it fails. The grading tree as it stands fails.

**The target is a summary and not a body sentence.** Row 9 grades a repair through findings, and a repair that deletes the narrated clause also clears the finding. So the baseline holds a `# summary:` line with the text `Eight comments in cli, paint, generate, probe, sweep`, and the oracle fails a tree whose summary, as `headwater explain --json` parses it, does not hold that text. No engine reader gives a body sentence, so a probe of this row takes a target whose finding is in a parsed facet (#1874, slice 3b). The text in a comment, in the body or in another key each fail.

**A repair of the body alone fails.** The Context of HW-OBL-0230 narrates in the same words, and that finding is in the baseline. The findings are compared by rule and path, so a session that repairs the body sentence and leaves the summary matches the baseline. So the baseline also holds a `# summary-not:` line with the narrated words, and the oracle fails a tree whose parsed summary still holds them.

**The axes.** No phrasing names a file, so the probe does not carry `names_file`. No superseded or contradicted record stands in the way, so the probe does not carry `superseded_target`.

**The baseline moves with its file.** The baseline holds the findings of one file in the sealed grading tree. A change to that file on `main`, or a seal that removes a document it links, moves them, and `tools/probe/probe-oracle-fixtures.sh` then fails this probe's reference change. A prose cleanup that rewrites the narrated phrase in HW-OBL-0230 removes the target of this probe, and the reference change then fails. Re-record the baseline with `sh tools/probe/oracles/nothing-else-broken.sh --record <baseline>` in that tree, and delete the line of the finding the task repairs.
