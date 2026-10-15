---
id: HW-OBL-0120
status: current
status_since: 2026-08-14
waiting_on: adopter
summary: "`headwater conformance` reads a waiver against a conformance rule, and nothing reads a waiver against a check rule or refuses one over withholding."
last_verified: 2026-10-15
title: "No reader takes a waiver against a check finding, so one arm of the escape partition is empty"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft+revise
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-distribution-and-federation
---

# No reader takes a waiver against a check finding, so one arm of the escape partition is empty

## Context

[Spec 4](../spec/04-assurance-model.md#suppression) fixes a precedence over three escape mechanisms: waiver, then migration-pending, then suppression. The three inventories partition the escaped findings, so no finding is counted three times. [Spec 7](../spec/07-distribution-and-federation.md#waivers) defines one waiver mechanism with two populations: a waiver may name a conformance rule or a check rule. The four required fields and the mandatory expiry are the same in each.

`headwater conformance` is the reader of the first population. It reads the `conformance.waivers` block of the consumer declaration and honors a live waiver over a conformance rule. It reports an expired waiver and does not honor it. The reader of the second population is the coverage account of `headwater check`, and it does not exist. No module under `engine/crates/check/src` reads a waiver. The coverage line of a run states the absence rather than a count: "no waiver reaches a check finding", then the migration-pending and suppressed counts. The module comments of `engine/crates/check/src/adoption.rs` and `engine/crates/check/src/suppression.rs` say that the waiver arm is absent.

## Obligation

The corpus owes a reader that takes a waiver against a check rule. It also owes a coverage account that counts a waived finding at the precedence that spec 4 fixes.

Three questions come with the reader, and this record does not answer them. Is a check-rule waiver scoped to the corpus or to a shelf? How does a report name a waived finding, given that a waiver is wider than a suppression? What refuses a waiver against a [withholding rule](../spec/07-distribution-and-federation.md#an-export-profile-carries-a-filter)? Spec 7 puts a withholding rule outside the waiver mechanism.

**The rule that the exclusion names now runs, and the exclusion is still not enforced.** `headwater export` writes a profile through its withholding filter, and `engine/crates/cli/tests/export.rs` holds its cases. So a withholding rule is now a real rule of the engine. But no waiver can reach any check rule, so no code refuses a waiver over a withholding rule, and no fixture can show that refusal.

**Nobody needs a check-rule waiver today.** This repository escapes a check finding with a suppression directive or an adoption task. Both are narrower than a waiver, and both have a reader. So the next act is a corpus that asks for a check-rule waiver, which this repository is not. A reader built before that corpus answers the three questions above with no user to ask.

## Discharge

The trigger is an adopter, or an issue, that asks for a waiver against a check rule. When that request arrives, three things discharge this record:

- A reader in `headwater-check` that takes the same `Waiver` value that `headwater-conformance` reads.
- A coverage bucket at the precedence that spec 4 fixes.
- A fixture in which a waiver over a withholding rule is refused by name. That fixture fails before the refusal ships.

The record closes when a run over a corpus that holds a check-rule waiver reports the waived finding as waived, and not as suppressed.
