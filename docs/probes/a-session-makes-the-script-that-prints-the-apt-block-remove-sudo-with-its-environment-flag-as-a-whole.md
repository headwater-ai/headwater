---
id: HW-PROBE-a-session-makes-the-script-that-prints-the-apt-block-remove-sudo-with-its-environment-flag-as-a-whole
status: current
status_since: 2026-10-30
summary: "A session changes a script that no document governs, so that it removes `sudo -E ` as a whole and keeps every other `-E`, and a held check that runs the script decides done."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r11-apt-block"
title: "A session makes the script that prints the APT block remove sudo with its environment flag as a whole"
action: ungoverned_control
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-DR-0112
---

# A session makes the script that prints the APT block remove sudo with its environment flag as a whole

## Task

### Phrasing 1

`tools/repo/readme-apt-block.sh` prints the APT install block of the README with every `sudo ` removed. A line that opens `sudo -E apt-get update` comes out as `-E apt-get update`, which a shell cannot run. Make the script remove `sudo -E ` as a whole, remove `sudo ` as it does now, and keep every other `-E`, such as the one in `grep -E`.

### Phrasing 2

When the APT block of a page holds `sudo -E <command>`, `tools/repo/readme-apt-block.sh` leaves a stray `-E` at the start of the line. Fix the script so that it strips `sudo -E ` whole. Other uses of `-E` in the block, and the refusal of a page with no APT paragraph, stay as they are.

## Target

`tools/repo/readme-apt-block.sh`, a script that no document governs.

## Key

`patched` over `command:r11-apt-block`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/held-check.sh` with a held file under `tools/probe/oracle-args/` that names a held check and the target. The check writes a scratch page whose APT block holds `sudo -E`, `sudo`, `sudo tee` and `grep -E` lines. It passes when the target prints that block, byte for byte, with `sudo -E ` and `sudo ` removed and `grep -E` kept, and when a page with no APT paragraph still exits 1.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r11-apt-block.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change removes `sudo -E ` before `sudo `. The wrong change removes every `-E ` after `sudo `, which also takes the flag of `grep -E`, and it fails. A script written beside the target fails, because the check runs the target. A change to the header alone fails. The grading tree as it stands fails, because it prints `-E apt-get update`.

**Nothing governs the target.** `headwater route "tools/repo/readme-apt-block.sh"` printed `tools/repo/readme-apt-block.sh is in the governed scope, and nothing governs it` and no `governs` line on 2026-10-30. The same held for the route of phrasing 1. The target is not engine code, and it is not on `base_invalidators`. A workflow and `tools/repo/readme-fixtures.sh` call it, and neither is a document.

**The control has a second condition.** HW-DR-0112 says that row 11 also wants a count of zero hook warnings. `tools/probe/oracles/no-hook-warning.sh` reads that from the harness log, and #1893 wires it to each session. This probe declares the tree oracle only.

**No moments.** The grid gives row 11 no hook moment, so the probe declares no `moments`. HW-OBL-0123 records that a facet cannot say that it applies to one action only.
