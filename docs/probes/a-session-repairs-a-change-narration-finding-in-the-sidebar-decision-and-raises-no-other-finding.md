---
id: HW-PROBE-a-session-repairs-a-change-narration-finding-in-the-sidebar-decision-and-raises-no-other-finding
status: current
status_since: 2026-10-30
summary: "A session rewrites the sentence of HW-DR-0108 that ends with the words as before, so the change-narration finding clears, and raises no finding the pin did not carry."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r9-sidebar"
title: "A session repairs a change-narration finding in the sidebar decision and raises no other finding"
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
    - HW-DR-0108
---

# A session repairs a change-narration finding in the sidebar decision and raises no other finding

## Task

### Phrasing 1

`headwater check` reports change narration in the decision on the order of the site sidebar, in its sentence on where a shelf's generated index stands. Repair it, and raise no other finding.

### Phrasing 2

The decision that the site sidebar lists a shelf in path order says the generated index stays first "as before", and the voice rule reports it. State the position that holds instead, with nothing new for the check to report.

## Target

[HW-DR-0108](../decisions/0108-the-site-sidebar-lists-a-shelf-in-path-order-and-the-shelf-index-keeps-the-reading-order.md), whose sentence "The generated index of a shelf stays the first entry of its group, as before." carries the finding.

## Key

`patched` over `command:r9-sidebar`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/nothing-else-broken.sh --check-clean` with a held file under `tools/probe/oracle-args/` that names the baseline. The baseline names this probe, scopes the oracle to the target's file, and holds the findings of that file in the sealed grading tree less the one the task asks to repair. The oracle passes when the file carries no finding, of any severity, that the baseline does not, and no strict error. So the tree as it stands fails on the one finding the baseline lacks.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r9-sidebar.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change writes "The generated index of a shelf is the first entry of its group." It clears the finding, raises none, and passes. The wrong change writes "The generated index of a shelf can't leave the first entry of its group." The contraction raises a finding of the language rule, so it fails. The grading tree as it stands fails.

**The axes.** No phrasing names a file, so the probe does not carry `names_file`. No superseded or contradicted record stands in the way, so the probe does not carry `superseded_target`.

**The baseline moves with its file.** The baseline holds the findings of one file in the sealed grading tree. A change to that file on `main`, or a seal that removes a document it links, moves them, and `tools/probe/probe-oracle-fixtures.sh` then fails this probe's reference change. Re-record it with `sh tools/probe/oracles/nothing-else-broken.sh --record <baseline>` in that tree, and delete the line of the finding the task repairs.
