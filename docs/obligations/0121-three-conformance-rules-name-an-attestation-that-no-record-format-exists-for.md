---
id: HW-OBL-0121
status: current
status_since: 2026-08-14
summary: "Three of the seven conformance rules can never be met, because the attestation each one names has no record format, and four rungs of the ladder wait behind them."
last_verified: 2026-08-14
title: "Three conformance rules name an attestation that no record format exists for"
provenance:
  warrant: proposed
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-distribution-and-federation
---

# Three conformance rules name an attestation that no record format exists for

## Context

[Spec 7](../spec/07-distribution-and-federation.md#conformance) says that a rule no tree decides degrades to a recorded attestation with an owner and a date. It adds that such rules are not silently dropped. The verb keeps the second half of that promise and cannot keep the first.

The measurement over this repository, at `--now 2026-08-14`:

| rule | why no tree decides it | verdict today |
|---|---|---|
| `checks.wired` | which file runs the checks on every change is a property of the forge | not decided |
| `gates.required` | branch protection lives in the administration surface of a forge | not decided |
| `hooks.installed` | `core.hooksPath` is per-clone configuration, so the tree that carries the hooks says nothing about who runs them | not decided |

Four of the seven rules the package declares carry a reading and three carry none. A rule with none is reported, states what would decide it, and counts as neither met nor missing. It is therefore never met, so **a rung that named one would be a rung nobody reaches.** No rung names one, and the three sit outside the ladder.

That is what holds the ladder at three rungs. [The maturity model](../doctrine/maturity-model.md) describes seven. L4 Gated is `gates.required` alone, and L3 Linked needs `hooks.installed` beside a participation reading that also does not exist.

## Obligation

The corpus owes a record format for an attestation: what it asserts, who owns it, the date it was taken, and what makes it stale. Nothing today states any of the four.

Two constraints are already fixed and neither is negotiable. An attestation is a claim a person makes about a thing this engine cannot read. It is evidence rather than a measurement, so a rung that rests on one rests on a person's word. And [spec 4](../spec/04-assurance-model.md#accuracy-audit) rules that an audit output which is not addressable is theater. The same argument holds one level down: an attestation that is not a typed, tracked document is a checkbox.

The open question is the expiry. A waiver and a suppression each carry a mandatory one. An attestation states that something was true on a date, and says nothing about the day after. So it may need a staleness window rather than an expiry, and no document states which.

## Discharge

Three things discharge it. A kind for an attestation record, with a section contract and an identifier scheme. A reading in `headwater-conformance` that takes an attestation over the rule it names. A rung in the package that names one of the three rules above.

The record closes when `headwater conformance` reports one of those three rules as met, on the strength of a document in this corpus. A stale attestation owes a report of its own by the same day, rather than a silent pass.
