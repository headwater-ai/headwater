---
id: HW-PROBE-a-session-adds-a-retired-term-to-the-overlay-that-the-task-names-and-leaves-the-lock-matching-it
status: current
status_since: 2026-10-30
summary: "A session adds cutting-edge to the retired terms in .headwater/overlay.yml and resolves, so headwater taxonomy resolve --check exits 0."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r8-cutting-edge"
title: "A session adds a retired term to the overlay that the task names and leaves the lock matching it"
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
    - HW-DR-0112
---

# A session adds a retired term to the overlay that the task names and leaves the lock matching it

## Task

### Phrasing 1

Add "cutting-edge" to the retired terms in `.headwater/overlay.yml`, with the reason it is retired, and leave the taxonomy lock matching the overlay.

### Phrasing 2

In `.headwater/overlay.yml`, retire the phrase "cutting-edge" with a reason. The repository must stay consistent after the change.

## Target

`.headwater/overlay.yml`, whose block `regimes.language.ste_house` holds `retired_terms`.

## Key

`patched` over `command:r8-cutting-edge`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/lock-regenerated.sh --check-clean` with a held file under `tools/probe/oracle-args/`, which names this probe and states what the engine must then do. The oracle passes when `headwater taxonomy resolve --check` exits 0, the engine's own retired-term rule does what the held file states in a scratch root, and the overlay carries no strict error. On the pin the lock already matches its sources, so the held file is what makes the tree as it stands fail. No line of the overlay is matched, so text in a comment, in a quoted reason or under another list does nothing and fails.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r8-cutting-edge.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change adds `term: cutting-edge` with a reason to `retired_terms` and runs `headwater taxonomy resolve`. It passes. The wrong change adds the same entry and runs no resolve, so `headwater taxonomy resolve --check` exits 1, and it fails. The grading tree as it stands fails.

**The axes.** Each phrasing names the overlay, so the probe carries `names_file`. No superseded or contradicted record stands in the way, so the probe does not carry `superseded_target`.
