---
id: HW-PROBE-a-session-retires-a-word-with-its-replacement-so-that-the-check-offers-the-substitution
status: current
status_since: 2026-10-30
summary: "A session adds utilize to the retired terms of the overlay with use as its replacement, which makes it an error with a patch, and resolves the taxonomy lock."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r8-utilize"
title: "A session retires a word with its replacement so that the check offers the substitution"
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

# A session retires a word with its replacement so that the check offers the substitution

## Task

### Phrasing 1

Retire the word "utilize" in this repository's prose, with "use" in its place, so that `headwater check --fix` can write the substitution.

### Phrasing 2

Make "utilize" a retired term whose fix is "use", so that the check reports it as an error and offers the patch.

## Target

`.headwater/overlay.yml`, whose block `regimes.language.ste_house` holds `retired_terms`.

## Key

`patched` over `command:r8-utilize`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/lock-regenerated.sh --check-clean` with a held file under `tools/probe/oracle-args/`, which names this probe and states what the engine must then do. The oracle passes when `headwater taxonomy resolve --check` exits 0, the engine's own retired-term rule does what the held file states in a scratch root, and the overlay carries no strict error. On the pin the lock already matches its sources, so the held file is what makes the tree as it stands fail. No line of the overlay is matched, so text in a comment, in a quoted reason or under another list does nothing and fails.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r8-utilize.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change adds `term: utilize` with a reason and `replacement: use` to `retired_terms` and runs `headwater taxonomy resolve`. It passes. The wrong change adds `term: utilize` with a reason and no replacement, and resolves. The check then reads the term as advisory and offers no patch, and the finding's remediation is not `write use`, so it fails. The grading tree as it stands fails.

**The axes.** No phrasing names a file, so the probe does not carry `names_file`. No superseded or contradicted record stands in the way, so the probe does not carry `superseded_target`.
