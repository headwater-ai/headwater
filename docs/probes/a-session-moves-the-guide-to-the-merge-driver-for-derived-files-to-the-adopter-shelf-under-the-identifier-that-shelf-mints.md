---
id: HW-PROBE-a-session-moves-the-guide-to-the-merge-driver-for-derived-files-to-the-adopter-shelf-under-the-identifier-that-shelf-mints
status: current
status_since: 2026-10-30
summary: "A session moves the process how-to on derived files that conflict to docs/how-to/, gives it the HW-HOW identifier that shelf's scheme requires, and repairs every link to it."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r7-merge-driver"
title: "A session moves the guide to the merge driver for derived files to the adopter shelf under the identifier that shelf mints"
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

# A session moves the guide to the merge driver for derived files to the adopter shelf under the identifier that shelf mints

## Task

### Phrasing 1

An adopter runs the same merge driver for derived files that this repository runs, so the guide `docs/process/how-to/keep-derived-files-from-conflicting-in-parallel-pull-requests.md` belongs with the adopter how-to guides. Move it to `docs/how-to/` with the same file name, and leave the corpus clean.

### Phrasing 2

Move `docs/process/how-to/keep-derived-files-from-conflicting-in-parallel-pull-requests.md` to `docs/how-to/`, keeping its file name. Repair every link to it, and give it whatever the new shelf requires.

## Target

[The how-to on derived files that conflict](../process/how-to/keep-derived-files-from-conflicting-in-parallel-pull-requests.md), which moves to the shelf of adopter how-to guides.

## Key

`patched` over `command:r7-merge-driver`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/moved.sh --check-clean` with the old and the new path in a held file under `tools/probe/oracle-args/`, which names this probe. The oracle passes when nothing is at the old path, a file is at the new one, no Markdown link resolves to the old path, and neither the moved file nor any file that links it carries a strict error.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r7-merge-driver.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change moves the file, repairs the links, rewrites its identifier from `HW-PHOW-` to `HW-HOW-`, the scheme of the adopter shelf, and runs `headwater generate`. It passes. The wrong change moves the file and repairs the links, and keeps the identifier `HW-PHOW-`. `moved.sh` alone passes it, and `headwater check --strict` reports `identifier.pattern.not_met` on the moved file, so `--check-clean` fails it. The grading tree as it stands fails.

**The axes.** Each phrasing names the file, so the probe carries `names_file`. No superseded or contradicted record stands in the way, so the probe does not carry `superseded_target`.

**The moments.** Row 7 declares no `moments`. The hooks cell of row 7 in the grid of HW-DR-0112 reads "often bypassed via the shell" and names no moment, so a hook variant that selects probes by moment does not run this probe. HW-OBL-0123 records why the taxonomy cannot say that a facet applies to some rows only.
