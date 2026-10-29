---
id: HW-PROBE-a-session-retires-a-stock-phrase-in-the-list-that-the-check-reads-and-resolves-the-lock
status: current
status_since: 2026-10-30
summary: "A session adds game-changer to the retired terms of the house language regime in the overlay and resolves the taxonomy lock, and does not only extend the prose list in CLAUDE.md."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r8-game-changer"
title: "A session retires a stock phrase in the list that the check reads and resolves the lock"
action: change_taxonomy
moments: [write]
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

# A session retires a stock phrase in the list that the check reads and resolves the lock

## Task

### Phrasing 1

Retire the phrase "game-changer" from the prose of this repository, so that the check reports it, with the reason it is retired.

### Phrasing 2

"game-changer" is stock phrasing that this repository does not want. Add it to the retired terms that `headwater check` reads, and give the reason.

## Target

[The overlay](../../.headwater/overlay.yml), whose block `regimes.language.ste_house` holds `retired_terms`.

## Key

`patched` over `command:r8-game-changer`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/lock-regenerated.sh --check-clean` with a held file under `tools/probe/oracle-args/`, which names this probe and states what the overlay must hold. The oracle passes when `headwater taxonomy resolve --check` exits 0, the overlay holds each literal of the held file as a whole name, and the overlay carries no strict error. On the pin the lock already matches its sources, so the literal is what makes the tree as it stands fail.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r8-game-changer.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change adds `term: game-changer` with a reason to `retired_terms` in `.headwater/overlay.yml` and runs `headwater taxonomy resolve`. It passes. The wrong change adds the phrase to the prose list in `CLAUDE.md` and leaves the overlay as it was. No check reads that list, the overlay does not hold the term, and it fails. The grading tree as it stands fails.

**The axes.** No phrasing names a file, so the probe does not carry `names_file`. No superseded or contradicted record stands in the way, so the probe does not carry `superseded_target`.
