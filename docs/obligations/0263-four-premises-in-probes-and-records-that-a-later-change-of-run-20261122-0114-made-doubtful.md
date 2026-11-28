---
id: HW-OBL-0263
status: current
status_since: 2026-11-24
summary: "Two probe scenarios and two spec pages state premises that later changes made doubtful. HW-DR-0112 says that phase B can discharge four records that stay open."
last_verified: 2026-11-24
title: "Four premises in probes and records that a later change of run 20261122-0114 made doubtful"
waiting_on: ruling
provenance:
  warrant: asserted
relations:
  traces_to:
    - HW-DR-0112
    - HW-OBL-0012
    - HW-OBL-0013
    - HW-OBL-0014
    - HW-OBL-0016
---

# Four premises in probes and records that a later change of run 20261122-0114 made doubtful

## Context

[The register of open obligations](../spec/open-obligations.md) holds a finding that has no reader outside this repository. Run `20261122-0114` wrote intake lines 3, 23, 53 and 154. The product owner ruled each one RECORD into the group of stale text that its end-of-run report calls S2. That group held seventeen lines. A record that bundles gaps is split when it passes five, so this record holds the premises that ask for a ruling before an edit. This record checked each one again on `0cdd96298`.

## Obligation

- `docs/spec/glossary.md` and `docs/spec/design-departures.md` still say "without external evidence". #1974 dropped "external" from stop rule 1 of [AI integration](../spec/ai-integration.md). The adjudication of #1974 ruled both still true, because a stated ruling is outside the record, and the maintainer reads them as stale. Somebody rules which reading holds.
- `docs/probes/a-session-lets-the-required-sections-rule-read-a-numbered-heading.md` says that the code holds `const VERSION: u32 = 1` and does not say when it moves. Since #1976, every `VERSION` constant says when it moves. Somebody rules whether the scenario states a frozen premise on purpose, or whether a rerun now measures a different code base.
- `docs/probes/a-session-that-moves-a-shelf-reaches-the-relocation-guide.md`, which is deprecated, says that no package declares the shelf of verification records. Since #2017, the requirement-verification entry declares it. A build does not rule an edit to the scenario of a probe.
- HW-DR-0112 has a section on the open claims of [the register of open obligations](../spec/open-obligations.md). It says that the evaluation of phase B can discharge HW-OBL-0012, HW-OBL-0013, HW-OBL-0014 and HW-OBL-0016. The evaluation and [the register of open obligations](../spec/open-obligations.md) say that all four stay open.

## Discharge

Each item discharges alone, when a ruling says which reading holds and the document follows it. A rerun of a probe that records the new premise also discharges its item. The record discharges when every item has.
