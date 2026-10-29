---
id: HW-PROBE-a-session-records-as-owed-the-corpus-size-axis-that-the-additive-campaign-defers
status: current
status_since: 2026-10-29
summary: "A session writes an obligation record that cites HW-DR-0112, which defers the corpus-size axis and records it nowhere, and not HW-DR-0076, the decision on probe budgets that a search for the campaign also meets."
last_verified: 2026-10-29
probe_category: sufficiency
expectation: patched
oracle: "command:r4-corpus-size"
title: "A session records as owed the corpus-size axis that the additive campaign defers"
action: add_document
moments: [write, review]
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

# A session records as owed the corpus-size axis that the additive campaign defers

## Task

### Phrasing 1

The additive campaign does not measure whether the size of a corpus changes what its layers do for an agent. The decision behind the campaign says this axis is deferred and recorded as debt, but no record of that debt exists. Write it, in the place and the shape this repository uses for what it still owes.

### Phrasing 2

Nothing records that the campaign which measures the layers of this repository leaves out corpus size as a circumstance. Its design calls that a deferred debt. Add the record of that debt where this repository keeps the work it owes.

### Phrasing 3

Record as an open obligation that the additive campaign defers the corpus-size axis. The ruling that defers it says it is recorded as debt, and it is not.

## Target

The session writes an obligation record on the shelf of obligation records that cites [HW-DR-0112](../decisions/0112-the-additive-campaign-measures-eleven-actions-by-five-layers-in-seven-scenarios.md). Its section "The circumstance axes" defers corpus size and says it is recorded as debt.

## Key

`patched` over `command:r4-corpus-size`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/typed-at.sh --check-clean` with the arguments in a held file under `tools/probe/oracle-args/`, which names this probe. The oracle passes when one document directly under `docs/obligations/` has a title that matches `corpus|ax[ie]s|size`, is of the kind `obligation_record`, names `HW-DR-0112` as a whole name, and carries no strict error.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r4-corpus-size.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change writes the record with `headwater new obligation_record` and `--relates traces_to=HW-DR-0112`, and it passes. The wrong change writes the same record with `--relates traces_to=HW-DR-0076`, the decision that prices a probe run, and it fails. The grading tree as it stands fails, because no obligation record cites HW-DR-0112.

**The axes.** No phrasing names a file, so the probe does not carry `names_file`. HW-DR-0111, the superseded design of the same campaign, names probes, so the seal removes it from every workspace and it cannot stand in the way. The probe does not carry `superseded_target`.

**The gap is real on the pin.** On 2026-10-29 no record under `docs/obligations/` or on spec 13 names the corpus-size axis. HW-DR-0112 says the three deferred axes are "recorded as debt", and none of them is.
