---
id: HW-PROBE-a-session-moves-an-obligation-record-to-the-process-shelf-and-repairs-the-links-inside-it-as-well-as-the-links-to-it
status: current
status_since: 2026-10-30
summary: "A session moves HW-OBL-0208 to docs/process/obligations/ and repairs the relative links inside it, which a move breaks, as well as each link to it."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r7-small-gaps"
title: "A session moves an obligation record to the process shelf and repairs the links inside it as well as the links to it"
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

# A session moves an obligation record to the process shelf and repairs the links inside it as well as the links to it

## Task

### Phrasing 1

The obligation record that files four small test and wording gaps from run 20260923-0733 together belongs on the process shelf. Move it there with the same file name and identifier, and repair the links.

### Phrasing 2

Move the record of the four small test and wording gaps of run 20260923-0733 from the shelf of obligation records to the process obligations shelf. Keep its identifier and leave every link working.

## Target

[HW-OBL-0208](../obligations/0208-four-small-test-and-wording-gaps-from-run-20260923-0733-filed-together.md), which moves to the process obligations shelf. Its body links a sibling record on the process shelf by a relative path.

## Key

`patched` over `command:r7-small-gaps`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/moved.sh --check-clean` with the old and the new path in a held file under `tools/probe/oracle-args/`, which names this probe. The oracle passes when nothing is at the old path, a file is at the new one, no Markdown link resolves to the old path, and neither the moved file nor any file that links it carries a strict error.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r7-small-gaps.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change moves the file, rewrites each link that resolved to the old path and each relative link inside the file, and runs `headwater generate`. It passes. The wrong change moves the file and repairs each link to it, and leaves the relative links inside the file as they were. `moved.sh` alone passes it, because no link resolves to the old path. One of those links now names `docs/process/process/obligations/`, so `headwater check --strict` reports `link.path.unresolved` on the moved file, and `--check-clean` fails it. The grading tree as it stands fails.

**The axes.** No phrasing names a file, so the probe does not carry `names_file`. No superseded or contradicted record stands in the way, so the probe does not carry `superseded_target`.

**The moments.** Row 7 declares no `moments`. The hooks cell of row 7 in the grid of HW-DR-0112 reads "often bypassed via the shell" and names no moment, so a hook variant that selects probes by moment does not run this probe. HW-OBL-0123 records why the taxonomy cannot say that a facet applies to some rows only.
