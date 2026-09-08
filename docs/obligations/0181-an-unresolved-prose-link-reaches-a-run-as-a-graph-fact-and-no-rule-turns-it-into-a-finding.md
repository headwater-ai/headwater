---
id: HW-OBL-0181
status: current
status_since: 2026-09-08
summary: A link in prose whose target path is absent is counted in the graph section of a run and reaches no rule, so it never becomes a finding and never fails a strict run.
last_verified: 2026-09-08
title: "An unresolved prose link reaches a run as a graph fact and no rule turns it into a finding"
waiting_on: build
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# An unresolved prose link reaches a run as a graph fact and no rule turns it into a finding

## Context

`headwater check` counts the links it could not resolve and prints the count in the graph section of its report, with the file, the line and the reason. The count is a statistic about the run. No rule reads it, so no finding carries it, `--strict` does not fail on it, and `headwater infer` cannot record it as pending debt.

The near rule is `link.fragment.unresolved`, and it is a different rule. It reads a fragment inside a document, which is the part after the hash. A link whose path does not exist at all is outside what it asks.

[The n8n evaluation](../evaluations/n8n-worked-example.md#the-broken-link-that-no-rule-reports) measured one instance in somebody else's corpus. `packages/@n8n/expression-runtime/ARCHITECTURE.md:426` points one directory too high, the run printed `1 prose links that did not resolve`, and the same run reported 12 findings, none of which was this one. A person reading the graph section found it. Nothing in the report would have raised it.

Three obligations already in the register look like this one and are not. HW-OBL-0116 and HW-OBL-0117 are about anchors in imported documents. HW-OBL-0136 is about rustdoc links inside comments. None of the three asks what happens to a link in prose whose target path is absent.

## Obligation

The corpus owes a decision on whether an unresolved path in a link is a finding, and at what severity. The remedy is mechanical only when the intended target can be derived, and it is a rewrite when it cannot, so spec 12's fixability bar points at advisory rather than error. Until the decision is taken, this corpus and every corpus that adopts this taxonomy carries broken references that a green strict run says nothing about.

## Discharge

A rule that turns an unresolved link target into a finding, declared in the taxonomy and implemented with a fixture that fails before it passes, discharges this. A ruling that the graph-section count is the right place for the fact, written where a reader of the report meets it, discharges it too. Either one closes it. Leaving the count in the graph section with no ruling does not.
