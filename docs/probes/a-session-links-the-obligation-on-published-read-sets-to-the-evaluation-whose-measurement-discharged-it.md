---
id: HW-PROBE-a-session-links-the-obligation-on-published-read-sets-to-the-evaluation-whose-measurement-discharged-it
status: current
status_since: 2026-10-29
summary: "A session finds that the Q21 evaluation measured what HW-OBL-0017 owed and writes the discharges edge at both ends, and not at the measurement-layer evaluation that only reports it."
last_verified: 2026-10-29
probe_category: sufficiency
expectation: patched
oracle: "command:r5-read-set"
title: "A session links the obligation on published read sets to the evaluation whose measurement discharged it"
action: edit_depended_document
moments: [write, review]
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-OBL-0017
---

# A session links the obligation on published read sets to the evaluation whose measurement discharged it

## Task

### Phrasing 1

This repository once owed a measurement of whether publishing a read set lets a gate skip a re-run on most merges. That obligation is now discharged, but its record does not link the evaluation whose measurement discharged it. Find both documents and record the link.

### Phrasing 2

An obligation about read sets and skipped re-runs is marked discharged, and nothing in its front matter says which evaluation discharged it. One evaluation took that measurement. Link the two documents the way this corpus links a discharge.

### Phrasing 3

Find the discharged obligation that claimed a published read set saves re-runs, and the evaluation that measured it and found the claim false. Record in the corpus that the evaluation discharges the obligation.

## Target

[The Q21 evaluation](../evaluations/what-a-check-can-know.md), whose measurement discharged HW-OBL-0017, and whose section on the read set says "The claim that a published read set saves re-runs was measured, and the measurement came out against it."

## Key

`patched` over `command:r5-read-set`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/reciprocal-link.sh --check-clean` with the two documents in a held file under `tools/probe/oracle-args/`, which names this probe. The oracle passes when the `relations:` block of each names the other as a whole name, and neither carries a strict error. The taxonomy declares `discharges` from an evaluation to an obligation record with the inverse `discharged_by`, and it requires both halves.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r5-read-set.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change adds `discharges: [HW-OBL-0017]` to the evaluation and `discharged_by` with the evaluation to the obligation, and it passes. The wrong change links the measurement-layer evaluation instead. Its table row for Q21 says "Ran" and "HW-OBL-0017 records the reading", so a session can take it for the discharging record, and the change fails. The grading tree as it stands fails, because neither document names the other.

**The axes.** No phrasing names a file, so the probe does not carry `names_file`. The measurement-layer evaluation is current and reports the same reading, so it stands in the way, but it is neither superseded nor contradicted, and the probe does not carry `superseded_target`.
