---
id: HW-PROBE-a-session-supersedes-the-record-that-keeps-a-grammar-checker-out-of-the-engine-and-not-the-older-record-that-it-upholds
status: current
status_since: 2026-10-30
summary: "A session records the owner's ruling to link harper-core as the successor of HW-DR-0096, and not of HW-DR-0041, the Vale record whose question HW-DR-0096 says still stands."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r6-harper"
title: "A session supersedes the record that keeps a grammar checker out of the engine and not the older record that it upholds"
action: supersede_or_retire
moments: [write, review]
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-DR-0096
---

# A session supersedes the record that keeps a grammar checker out of the engine and not the older record that it upholds

## Task

### Phrasing 1

An adopter asked for grammar findings from harper-core inside `headwater check`, and the owner ruled that the engine now links harper-core as an advisory backend of the language regime. Record that ruling in place of the one that kept harper-core out of the engine.

### Phrasing 2

The owner reversed the decision that the engine does not link the harper grammar checker. Harper-core now joins the engine as an advisory backend, because an adopter asked for it. Write the new decision so that it replaces the old one.

### Phrasing 3

Record that harper-core becomes part of the engine, as an advisory backend, on an adopter's request. This corpus already holds the ruling that it does not, and the new ruling supersedes that one.

## Target

[HW-DR-0096](../decisions/0096-harper-does-not-become-part-of-the-engine-now-and-q41-stands.md), which keeps harper-core out of the engine and names an adopter request as what reopens it.

## Key

`patched` over `command:r6-harper`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/successor-typed.sh --check-clean` with the record that must be superseded in a held file under `tools/probe/oracle-args/`, which names this probe. The oracle passes when that record is `status: superseded`, a document the engine types names it under `supersedes`, the record names that successor in its `relations:` block as a whole name, and neither carries a strict error.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r6-harper.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change writes a decision with `headwater new decision --relates supersedes=HW-DR-0096`, sets it `current`, and runs `headwater check --fix`. It passes. The wrong change writes the same decision with `supersedes=HW-DR-0041`, the Vale record that the title of HW-DR-0096 says still stands. HW-DR-0096 stays `current`, so it fails. The grading tree as it stands fails.

**The axes.** No phrasing names a file, so the probe does not carry `names_file`. HW-DR-0041 stands beside the target and is `current`, and it is not superseded or contradicted, so the probe does not carry `superseded_target`.
