---
id: HW-OBL-0191
status: discharged
status_since: 2026-10-15
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

This one was measured rather than reasoned. The [`standards-spec` fixture corpus](https://github.com/headwater-ai/headwater/blob/main/docs/taxonomies/standards-spec/fixtures/README.md) plants `spec_layer: interface_spec` on one document, and no kind of that shelf takes the value. Kind resolution stops. The census carries the row, no rule instantiates over the document, and `headwater check --strict` exits 0 on a run of that document alone.

`engine/crates/check/src/coverage.rs` states the ruling deliberately, so this is not a defect of the engine. It is a property of every heterogeneous shelf. The `design-spec` entry's `spec_series` shelf has carried it since the library opened. The sixth finding at [line 162](https://github.com/headwater-ai/headwater/blob/5fb9518/docs/taxonomies/standards-spec/doctrine.md#L162) of the doctrine holds the run.

## Obligation

A typo in one metadata value is indistinguishable from a clean document at the gate. An adopter meets it directly and silently, on any shelf that resolves a kind from a value.

**Nothing tracks the missing check.** [#378](https://github.com/headwater-ai/headwater/issues/378) is the nearest issue and it closed as completed on 2026-09-01. It asked why nothing reads a template's front matter, so that a package could ship an example that resolves to no kind. Its remedy reads a template at publish, in `engine/crates/resolve/src/template.rs`. A template is not a corpus document, and the case this record states is still unread by any rule. So this record is the only place the gap is written down, and whoever takes the work opens the issue for it.

## Discharge

This record discharges when a check reports a document whose discriminator value resolves to no kind. The report has to reach the exit status of a strict run. A ruling that the silence is correct discharges it as well. [Spec 12](../spec/12-check-layer.md) then states the ruling where a reader of the check layer meets it.

**`shelf.discriminator.unresolved` discharges this record ([#1754](https://github.com/headwater-ai/headwater/pull/1754), for [#1491](https://github.com/headwater-ai/headwater/issues/1491)).** The rule is corpus-scoped, and `headwater_check::discriminator` implements it. It reads the census rows where kind resolution stopped at step 3 of [spec 2](../spec/02-taxonomy-model.md#kind-resolution). That step stops for three causes: no discriminator, a value that the shelf does not admit, or a value that is not a scalar. Each such document gets one error, so a strict run fails. The finding anchors at the line of the value, or at line 1 when the value is missing. Its message is the census reason, and its remedy names the kinds that the shelf admits. A path that no shelf claims, a file with no front matter, a shelf tie and an abstract kind get no finding from this rule. `engine/crates/cli/tests/discriminator_unresolved.rs` holds each case. The rule adds one instance to a run, and it moves no coverage count, because a corpus-scoped instance routes to no document.

Spec 2 says that the shelf declares the severity. No shelf schema has a field for it, so the severity is fixed at error. The new rule reaches no obligation yet. A control that binds it to OB-COV-1 belongs to the next release of `headwater/standard`.
