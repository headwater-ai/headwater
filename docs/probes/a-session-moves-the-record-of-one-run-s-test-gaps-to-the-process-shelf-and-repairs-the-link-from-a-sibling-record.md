---
id: HW-PROBE-a-session-moves-the-record-of-one-run-s-test-gaps-to-the-process-shelf-and-repairs-the-link-from-a-sibling-record
status: current
status_since: 2026-10-30
summary: "A session moves HW-OBL-0234 to docs/process/obligations/ and repairs every link to it, the bare relative link in the sibling record HW-OBL-0237 included."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r7-test-gaps"
title: "A session moves the record of one run's test gaps to the process shelf and repairs the link from a sibling record"
action: move_document
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

# A session moves the record of one run's test gaps to the process shelf and repairs the link from a sibling record

## Task

### Phrasing 1

Move the obligation record of the thirteen test gaps that the verifiers of run 20261002-1233 left to the process shelf. Keep its file name and identifier, and repair every link to it.

### Phrasing 2

The record that lists the test gaps the verifiers of run 20261002-1233 left in the recorder, the probe tools and the spec guards goes on the process obligations shelf. Move it, and leave no link to the old place.

## Target

[HW-OBL-0234](../obligations/0234-thirteen-test-gaps-that-the-verifiers-of-run-20261002-1233-left-in-the-recorder-the-probe-tools-and-the-spec-guards.md), which moves to the process obligations shelf. The sibling record HW-OBL-0237 links it by its bare file name.

## Key

`patched` over `command:r7-test-gaps`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/moved.sh --check-clean` with the old and the new path in a held file under `tools/probe/oracle-args/`, which names this probe. The oracle passes when nothing is at the old path, a file is at the new one, no Markdown link resolves to the old path, and neither the moved file nor any file that links it carries a strict error.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r7-test-gaps.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change moves the file, rewrites each link that resolved to the old path and each relative link inside the file, and runs `headwater generate`. It passes. The wrong change moves the file and repairs each link except the one in HW-OBL-0237, which names the file with no directory, so a search for `obligations/0234` misses it. That link still resolves to the old path, so it fails. The grading tree as it stands fails.

**The axes.** No phrasing names a file, so the probe does not carry `names_file`. No superseded or contradicted record stands in the way, so the probe does not carry `superseded_target`.

**The moments.** Row 7 declares no `moments`. The hooks cell of row 7 in the grid of HW-DR-0112 reads "often bypassed via the shell" and names no moment, so a hook variant that selects probes by moment does not run this probe. HW-OBL-0123 records why the taxonomy cannot say that a facet applies to some rows only.
