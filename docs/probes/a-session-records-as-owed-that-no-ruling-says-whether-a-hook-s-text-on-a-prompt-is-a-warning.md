---
id: HW-PROBE-a-session-records-as-owed-that-no-ruling-says-whether-a-hook-s-text-on-a-prompt-is-a-warning
status: current
status_since: 2026-10-29
summary: "A session writes an obligation record under docs/obligations that cites HW-DR-0112 for the unruled question of whether the row 11 oracle may count a hook's prompt text as a warning."
last_verified: 2026-10-29
probe_category: sufficiency
expectation: patched
oracle: "command:r4-hook-warning"
title: "A session records as owed that no ruling says whether a hook's text on a prompt is a warning"
action: add_document
moments: [write, review]
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

# A session records as owed that no ruling says whether a hook's text on a prompt is a warning

## Task

### Phrasing 1

`tools/probe/oracles/no-hook-warning.sh` counts any text that a hook prints on success as a warning, and that includes the route that the intent hook adds to a prompt. Nobody has ruled whether that is right. Record that open question in this repository, in the place and the shape it uses for what it still owes.

### Phrasing 2

The script `tools/probe/oracles/no-hook-warning.sh` fails a session when a hook printed any text, even the pointer that the hook on the user's prompt adds. Whether that text should count as a warning was never decided. Write that gap down where this repository keeps the work it owes.

### Phrasing 3

Read `tools/probe/oracles/no-hook-warning.sh`. It treats the text the prompt hook injects as a hook warning, and no ruling says it should. Add the record this repository keeps for an open obligation, so that a later reader finds the question.

## Target

The session writes an obligation record on the shelf of obligation records that cites [HW-DR-0112](../decisions/0112-the-additive-campaign-measures-eleven-actions-by-five-layers-in-seven-scenarios.md), the record that governs the generic oracle scripts and states the row 11 oracle in "The form of an oracle".

## Key

`patched` over `command:r4-hook-warning`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/typed-at.sh --check-clean` with the arguments in a held file under `tools/probe/oracle-args/`, which names this probe, so the seal removes it from every workspace and the grading copy restores it. The oracle passes when one document directly under `docs/obligations/` has a title that matches `hook`, is of the kind `obligation_record`, names `HW-DR-0112` as a whole name, and carries no error that a strict check fails on.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r4-hook-warning.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change runs `headwater new obligation_record` with a title about the hook text, the facet `waiting_on`, and `--relates traces_to=HW-DR-0112`, and it passes. The wrong change files the same record as a process obligation under `docs/process/obligations/`, because the question is about tooling, and it fails: a probe of the campaign is a measurement of the product, and its debt goes to spec 13. The grading tree as it stands fails, because no obligation under that shelf with `hook` in its title cites HW-DR-0112.

**The axes.** Each phrasing names the file, so the probe carries `names_file`. No superseded record in a sealed workspace states the hook text or rules on it, and the probe does not carry `superseded_target`.

**Why the check half reads the new document alone.** The sealed tree carries errors that the seal made in documents it kept, so a strict check of the whole tree fails every session. `--check-clean` on a script that names its document reads the strict errors of that document ([HW-DR-0112](../decisions/0112-the-additive-campaign-measures-eleven-actions-by-five-layers-in-seven-scenarios.md), "The form of an oracle").
