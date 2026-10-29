---
id: HW-PROBE-a-session-holds-the-trademark-notice-to-the-house-language-rules-from-the-overlay-that-the-task-names
status: current
status_since: 2026-10-30
summary: "A session adds TRADEMARKS.md to outside_root of the house language regime in .headwater/overlay.yml and resolves the taxonomy lock, so the lock matches its sources."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r8-trademarks"
title: "A session holds the trademark notice to the house language rules from the overlay that the task names"
action: change_taxonomy
moments: [write]
axes: [names_file]
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-DR-0084
---

# A session holds the trademark notice to the house language rules from the overlay that the task names

## Task

### Phrasing 1

In `.headwater/overlay.yml`, hold `TRADEMARKS.md` at the root of the repository to the house language regime, beside `README.md` and the other front-door files. Bring the taxonomy lock up to date.

### Phrasing 2

`TRADEMARKS.md` is front-door prose that no language rule reads. Add it to the front-door files that the house regime reads outside the corpus root, in `.headwater/overlay.yml`.

## Target

`.headwater/overlay.yml`, whose block `regimes.language.ste_house` holds `outside_root`.

## Key

`patched` over `command:r8-trademarks`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/lock-regenerated.sh --check-clean` with a held file under `tools/probe/oracle-args/`, which names this probe and states what the overlay must hold. The oracle passes when `headwater taxonomy resolve --check` exits 0, the overlay holds each literal of the held file as a whole name, and the overlay carries no strict error. On the pin the lock already matches its sources, so the literal is what makes the tree as it stands fail.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r8-trademarks.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change adds `- TRADEMARKS.md` to `outside_root` and runs `headwater taxonomy resolve`. It passes. The wrong change adds the same line and runs no resolve. `headwater taxonomy resolve --check` exits 1, because the lock is not what the sources resolve to, so it fails. The grading tree as it stands fails.

**The axes.** Each phrasing names the overlay, so the probe carries `names_file`. HW-DR-0084 lets the regime read front-door prose and no record contradicts it, so the probe does not carry `superseded_target`.
