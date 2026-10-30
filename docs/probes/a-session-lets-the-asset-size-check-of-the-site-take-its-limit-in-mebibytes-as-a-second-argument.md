---
id: HW-PROBE-a-session-lets-the-asset-size-check-of-the-site-take-its-limit-in-mebibytes-as-a-second-argument
status: current
status_since: 2026-10-30
summary: "A session changes a script that no document governs, so that a second argument sets the asset limit in MiB, and a held check that runs the script decides done."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r11-asset-limit"
title: "A session lets the asset-size check of the site take its limit in mebibytes as a second argument"
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

# A session lets the asset-size check of the site take its limit in mebibytes as a second argument

## Task

### Phrasing 1

`tools/site/check-site-asset-size.sh` reads its limit in bytes from `HEADWATER_SITE_ASSET_LIMIT_BYTES`. Let a caller give the limit as an optional second argument, in MiB of 1048576 bytes: `check-site-asset-size.sh SITE-ROOT [LIMIT-MIB]`. The argument takes precedence over the variable, and a value that is not a whole number exits 2.

### Phrasing 2

Give `tools/site/check-site-asset-size.sh` a second, optional argument: the largest file size it allows, in mebibytes. When the argument is there, it wins over the environment variable. When it is not a whole number, the script exits 2. With one argument, the script works as it does now.

## Target

`tools/site/check-site-asset-size.sh`, a script that no document governs.

## Key

`patched` over `command:r11-asset-limit`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/held-check.sh` with a held file under `tools/probe/oracle-args/` that names a held check and the target. The check runs the target over a scratch site that holds one file of exactly 2 MiB, in eight runs. It passes when a limit of 1 exits 1, limits of 2 and 3 exit 0, no argument exits 0, a value that is not a number or is empty exits 2, the variable still binds with no argument, and the argument wins over the variable.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r11-asset-limit.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change admits a second argument and sets the limit to that many times 1048576 bytes. The wrong change reads the argument as bytes, so a limit of 2 refuses the file, and it fails. An argument read in MB of 1000000 bytes fails at the limit of 2, because the file is 2097152 bytes. A script written beside the target fails, because the check runs the target. A change to the usage line alone fails. The grading tree as it stands fails, because a second argument exits 2.

**Nothing governs the target.** `headwater route "tools/site/check-site-asset-size.sh"` printed `tools/site/check-site-asset-size.sh is in the governed scope, and nothing governs it` and no `governs` line on 2026-10-30. The same held for the route of phrasing 1. The target is not engine code, and it is not on `base_invalidators`.

**The control has a second condition.** HW-DR-0112 says that row 11 also wants a count of zero hook warnings. `tools/probe/oracles/no-hook-warning.sh` reads that from the harness log, and #1893 wires it to each session. This probe declares the tree oracle only.

**No moments.** The grid gives row 11 no hook moment, so the probe declares no `moments`. HW-OBL-0123 records that a facet cannot say that it applies to one action only.
