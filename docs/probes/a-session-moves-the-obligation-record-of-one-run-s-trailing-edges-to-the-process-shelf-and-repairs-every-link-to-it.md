---
id: HW-PROBE-a-session-moves-the-obligation-record-of-one-run-s-trailing-edges-to-the-process-shelf-and-repairs-every-link-to-it
status: current
status_since: 2026-10-30
summary: "A session moves HW-OBL-0242 to docs/process/obligations/ under the same file name and identifier, and no link in the tree resolves to the old path, the shelf index included."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r7-run-findings"
title: "A session moves the obligation record of one run's trailing edges to the process shelf and repairs every link to it"
action: move_document
axes: [names_file]
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-PD-0024
---

# A session moves the obligation record of one run's trailing edges to the process shelf and repairs every link to it

## Task

### Phrasing 1

Move `docs/obligations/0242-three-edges-and-a-comment-that-trail-a-change-of-run-20261012-2342.md` to the process shelf, `docs/process/obligations/`, with the same file name and identifier. Repair every link to it.

### Phrasing 2

The record `docs/obligations/0242-three-edges-and-a-comment-that-trail-a-change-of-run-20261012-2342.md` is a run's finding and belongs on `docs/process/obligations/`. Move it there, keep its name and identifier, and leave no link pointing at the old path.

## Target

[HW-OBL-0242](../obligations/0242-three-edges-and-a-comment-that-trail-a-change-of-run-20261012-2342.md), which moves to `docs/process/obligations/`.

## Key

`patched` over `command:r7-run-findings`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/moved.sh --check-clean` with the old and the new path in a held file under `tools/probe/oracle-args/`, which names this probe. The oracle passes when nothing is at the old path, a file is at the new one, no Markdown link resolves to the old path, and neither the moved file nor any file that links it carries a strict error.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r7-run-findings.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change moves the file, rewrites each link that resolved to the old path and each relative link inside the file, and runs `headwater generate`. It passes. The wrong change moves the file and repairs every link except the one in the shelf index, `docs/obligations/README.md`, and runs no `headwater generate`. That link still resolves to the old path, so it fails. The grading tree as it stands fails.

**The axes.** Each phrasing names the file, so the probe carries `names_file`. No superseded or contradicted record stands in the way, so the probe does not carry `superseded_target`.

**The moments.** Row 7 declares no `moments`. The hooks cell of row 7 in the grid of HW-DR-0112 reads "often bypassed via the shell" and names no moment, so a hook variant that selects probes by moment does not run this probe. HW-OBL-0123 records why the taxonomy cannot say that a facet applies to some rows only.
