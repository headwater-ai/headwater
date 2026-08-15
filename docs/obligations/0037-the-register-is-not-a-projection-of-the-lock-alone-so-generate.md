---
id: HW-OBL-0037
title: "The register is not a projection of the lock alone, so `generate --check` cannot hold it"
status: current
status_since: 2026-08-13
last_verified: 2026-08-13
summary: "A rendered register is a function of the corpus, the lock and the clock, and a byte comparison fails on a morning when nobody changed a file."
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
    - HW-SPEC-engine-architecture
---

# The register is not a projection of the lock alone, so `generate --check` cannot hold it

## Context

[Spec 4](../spec/04-assurance-model.md#every-obligation-has-exactly-one-disposition) calls the register a projection like any other. It asks the engine to regenerate and check it. `headwater generate` was built to be that check, and it does not write the register.

A third input is the reason, and spec 4 does not name it. A rendered register states how many findings escaped under each obligation. It also states how many are migration-pending. Both counts move when a migration task lapses or a suppression expires, and each of those turns on a date. So the artifact is a function of the corpus, the lock and the clock.

## Obligation

A committed copy held to a byte comparison fails on a morning when nobody changed a file. A gate that fails for that reason is removed within a week.

## Discharge

Two remedies exist and neither is free. The register can render at a grain that omits the run-dependent lines. That is a second rendering of one artifact, which is the drift spec 4 rules against. Or the gate can take the date as an input, which makes the committed artifact a claim about one day. [Spec 6](../spec/06-engine-architecture.md#projections) has to choose one of them.
