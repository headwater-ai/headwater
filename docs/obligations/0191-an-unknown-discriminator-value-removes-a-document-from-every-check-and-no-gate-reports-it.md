---
id: HW-OBL-0191
status: current
status_since: 2026-09-11
summary: "A document whose discriminator value names no kind leaves every check, the census still carries the row, and a strict run exits 0."
last_verified: 2026-09-11
title: "An unknown discriminator value removes a document from every check and no gate reports it"
waiting_on: build
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# An unknown discriminator value removes a document from every check and no gate reports it

## Context

This one was measured rather than reasoned. The [`standards-spec` fixture corpus](../taxonomies/standards-spec/fixtures/README.md) plants `spec_layer: interface_spec` on one document, and no kind of that shelf takes the value. Kind resolution stops. The census carries the row, no rule instantiates over the document, and `headwater check --strict` exits 0 on a run of that document alone.

`engine/crates/check/src/coverage.rs` states the ruling deliberately, so this is not a defect of the engine. It is a property of every heterogeneous shelf. The `design-spec` entry's `spec_series` shelf has carried it since the library opened. The sixth finding at [line 162](https://github.com/headwater-ai/headwater/blob/5fb9518/docs/taxonomies/standards-spec/doctrine.md#L162) of the doctrine holds the run.

## Obligation

A typo in one metadata value is indistinguishable from a clean document at the gate. An adopter meets it directly and silently, on any shelf that resolves a kind from a value.

[#378](https://github.com/headwater-ai/headwater/issues/378) tracks the missing check, so this record cites that issue rather than asks for a second one. What this record adds is a place in the register where a reader who plans work meets the gap. An issue is read by whoever opens it, and this list is read by whoever asks what the corpus owes.

## Discharge

This record discharges when a check reports a document whose discriminator value resolves to no kind. The report has to reach the exit status of a strict run. A ruling that the silence is correct discharges it as well. [Spec 12](../spec/12-check-layer.md) then states the ruling where a reader of the check layer meets it.
