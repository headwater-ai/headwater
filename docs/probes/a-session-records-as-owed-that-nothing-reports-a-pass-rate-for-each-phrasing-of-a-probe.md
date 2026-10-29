---
id: HW-PROBE-a-session-records-as-owed-that-nothing-reports-a-pass-rate-for-each-phrasing-of-a-probe
status: current
status_since: 2026-10-29
summary: "A session writes a complete obligation record that cites HW-DR-0112 for the pooled phrasing axis, and the oracle fails a record that is missing a section the kind requires."
last_verified: 2026-10-29
probe_category: sufficiency
expectation: patched
oracle: "command:r4-phrasing-rate"
title: "A session records as owed that nothing reports a pass rate for each phrasing of a probe"
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

# A session records as owed that nothing reports a pass rate for each phrasing of a probe

## Task

### Phrasing 1

Each probe of the additive campaign states its task in two or three wordings, and the campaign pools the results over the wordings. Nothing reports the pass rate of each wording, so a wording that misleads or gives the answer away is hidden in the pool. Record that as an open obligation of this repository, in the shape it uses.

### Phrasing 2

The campaign draws one of several phrasings of a probe's task for each session and reports one pooled rate. Write down, where this repository keeps what it owes, that no rate per phrasing exists.

### Phrasing 3

A probe's phrasings are pooled, and no result splits them. So nobody can see that one phrasing passes far more often than another. Add the record of that gap to this repository's open obligations.

## Target

The session writes an obligation record on the shelf of obligation records that cites [HW-DR-0112](../decisions/0112-the-additive-campaign-measures-eleven-actions-by-five-layers-in-seven-scenarios.md), which states the phrasing axis under "The circumstance axes" and the form of a phrasing.

## Key

`patched` over `command:r4-phrasing-rate`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/typed-at.sh --check-clean` with the arguments in a held file under `tools/probe/oracle-args/`, which names this probe. The oracle passes when one document directly under `docs/obligations/` has a title that matches `phrasing|wording`, is of the kind `obligation_record`, names `HW-DR-0112` as a whole name, and carries no strict error.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r4-phrasing-rate.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change writes the record with `headwater new obligation_record` and `--relates traces_to=HW-DR-0112`, and it passes. The wrong change writes the same record by hand with no `## Discharge` section, and it fails on the check half alone: the title, the kind and the citation are right, and `section.required.missing` is an error on the document. This is the failure that the old row 4 probe graded by a rule finding. The grading tree as it stands fails.

**The axes.** No phrasing names a file, so the probe does not carry `names_file`. No superseded record states the phrasing axis, so the probe does not carry `superseded_target`.
