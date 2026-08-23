---
id: HW-OBL-0066
title: "A finding names one obligation, and a control discharges a list"
status: current
status_since: 2026-08-12
waiting_on: build
last_verified: 2026-08-13
summary: "Several controls may name one rule, and nothing says which obligation the finding of such a rule names."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-assurance-model
    - HW-SPEC-check-layer
---

# A finding names one obligation, and a control discharges a list

## Context

The [finding](../spec/04-assurance-model.md#findings) shape carries one identifier, and `discharges` is a sequence. Several controls may also name one rule, because a rule can run at more than one trigger. Nothing says which obligation the finding of such a rule names.

## Obligation

[Spec 4](../spec/04-assurance-model.md#findings) and [spec 12](../spec/12-check-layer.md#the-plugin-interface) owe a rule for the many-to-one case.

## Discharge

The engine binds one where the controls that name a rule reach exactly one obligation. Where they reach more, it names them all, binds none, and reports that in the run. That is the reading it takes for two anchor kinds that claim one string.

The register is where the asymmetry is visible from both ends. It lists the rule beside every obligation that the rule reaches, and `crates/check/fixtures/check.report` records one such rule.
