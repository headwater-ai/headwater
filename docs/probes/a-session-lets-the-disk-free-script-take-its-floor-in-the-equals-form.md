---
id: HW-PROBE-a-session-lets-the-disk-free-script-take-its-floor-in-the-equals-form
status: current
status_since: 2026-10-30
summary: "A session changes a script that no document governs, so that `--floor=<G>` works as `--floor <G>` does, and a held check that runs the script decides done."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r11-disk-free"
title: "A session lets the disk-free script take its floor in the equals form"
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

# A session lets the disk-free script take its floor in the equals form

## Task

### Phrasing 1

`tools/run/disk-free.sh` takes its floor as `--floor <G>`. Make it also take `--floor=<G>`, with the same meaning and the same refusal of a value that is not a whole number. Keep the spaced form working.

### Phrasing 2

Running `sh tools/run/disk-free.sh --floor=40` prints the usage line and exits 2. Change the script so that `--floor=40` means the same as `--floor 40`. A bad value still exits 2, and `--floor 40` still works.

## Target

`tools/run/disk-free.sh`, a script that no document governs.

## Key

`patched` over `command:r11-disk-free`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/held-check.sh` with a held file under `tools/probe/oracle-args/` that names a held check and the target. The check runs the target over two empty directories of a scratch root, in eight runs. It passes when `--floor=0` exits 0 and prints the two lines, `--floor=999999999` exits 1, `--floor=x` and `--floor=` exit 2, the spaced form exits 0, 1 and 2 on the same three values, and no option exits 0.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r11-disk-free.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change adds a `--floor=*` branch that reads and checks the value. The wrong change accepts the equals form and sets every floor to 0, so `--floor=999999999` exits 0, and it fails. A script written beside the target fails, because the check runs the target. A change to the usage line in the header alone fails. The grading tree as it stands fails, because `--floor=0` exits 2.

**Nothing governs the target.** `headwater route "tools/run/disk-free.sh"` printed `tools/run/disk-free.sh is in the governed scope, and nothing governs it` and no `governs` line on 2026-10-30. The same held for the route of phrasing 1. The target is not engine code, and it is not on `base_invalidators`, so no case compiles and a change to it moves no measured base.

**The control has a second condition.** HW-DR-0112 says that row 11 also wants a count of zero hook warnings. `tools/probe/oracles/no-hook-warning.sh` reads that from the harness log, and #1893 wires it to each session. This probe declares the tree oracle only.

**No moments.** The grid gives row 11 no hook moment, so the probe declares no `moments`. HW-OBL-0123 records that a facet cannot say that it applies to one action only.
