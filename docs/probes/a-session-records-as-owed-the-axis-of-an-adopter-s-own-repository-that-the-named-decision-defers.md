---
id: HW-PROBE-a-session-records-as-owed-the-axis-of-an-adopter-s-own-repository-that-the-named-decision-defers
status: current
status_since: 2026-10-29
summary: "A task that names HW-DR-0112's file asks for an obligation record of the deferred axis of an adopter's own repository, and the oracle requires that the record cites that decision."
last_verified: 2026-10-29
probe_category: sufficiency
expectation: patched
oracle: "command:r4-adopter-axis"
title: "A session records as owed the axis of an adopter's own repository that the named decision defers"
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

# A session records as owed the axis of an adopter's own repository that the named decision defers

## Task

### Phrasing 1

`docs/decisions/0112-the-additive-campaign-measures-eleven-actions-by-five-layers-in-seven-scenarios.md` defers one circumstance axis, an adopter's own repository, and says it is recorded as debt. It is not recorded anywhere. Write that record, in the place and the shape this repository uses for what it still owes.

### Phrasing 2

Read the section on circumstance axes in `docs/decisions/0112-the-additive-campaign-measures-eleven-actions-by-five-layers-in-seven-scenarios.md`. The campaign measures nothing in a repository other than this one. Add the record of that open debt.

### Phrasing 3

The campaign in `docs/decisions/0112-the-additive-campaign-measures-eleven-actions-by-five-layers-in-seven-scenarios.md` runs only on this repository, and the decision calls testing an adopter's own repository a recorded debt. Record that debt where this repository keeps its open obligations.

## Target

The session writes an obligation record on the shelf of obligation records that cites [HW-DR-0112](../decisions/0112-the-additive-campaign-measures-eleven-actions-by-five-layers-in-seven-scenarios.md), the file that each phrasing names.

## Key

`patched` over `command:r4-adopter-axis`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/typed-at.sh --check-clean` with the arguments in a held file under `tools/probe/oracle-args/`, which names this probe. The oracle passes when one document directly under `docs/obligations/` has a title that matches `adopter|repositor|ax[ie]s`, is of the kind `obligation_record`, names `HW-DR-0112` as a whole name, and carries no strict error.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r4-adopter-axis.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change writes the record with `headwater new obligation_record` and `--relates traces_to=HW-DR-0112`, and it passes. The wrong change writes the record and traces it to spec 13, the register where it is listed, and to nothing that rules the axis, and it fails. The grading tree as it stands fails, because no obligation record cites HW-DR-0112.

**The axes.** Each phrasing names the file, so the probe carries `names_file`. The task names the current record, so no superseded record stands in the way, and the probe does not carry `superseded_target`.
