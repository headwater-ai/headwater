---
id: HW-OBL-0218
status: current
status_since: 2026-09-26
summary: "Spec 13 opens its self-found list with a count of forty-nine items in five ranges, and no check compares that count to the list under it."
last_verified: 2026-09-26
title: "The opening sentence of the self-found list in spec 13 counts forty-nine items in ranges that do not match the list"
waiting_on: build
---

# The opening sentence of the self-found list in spec 13 counts forty-nine items in ranges that do not match the list

## Context

Run `20260926-0718` found this sentence while it read the section. The product owner ruled it Record, because the register has no reader outside this repository. [HW-OBL-0143](0143-a-specification-sentence-closes-a-set-with-a-count-the-source-exceeds-and-115-of-127-candidates-are-unchecked.md) records this class of defect: a sentence that closes a set with a count that its source does not hold. [HW-OBL-0142](0142-the-obligation-register-states-its-own-size-by-hand-and-it-went-stale-three-times-in-two-days.md) recorded an earlier instance on the same page.

## Obligation

The section "What the engine found about itself" in [13 — Open obligations](../spec/13-open-obligations.md) opens with "Forty-nine items". It names five ranges: HW-OBL-0131 through HW-OBL-0159, HW-OBL-0161 through HW-OBL-0168, HW-OBL-0172 through HW-OBL-0181, HW-OBL-0183, and HW-OBL-0196. These ranges hold 49 identifiers.

On 2026-09-26, the list under the sentence held 66 lines. Two differences cause the gap:

- Five identifiers in the ranges have no line in the list, because each one is discharged: HW-OBL-0135, HW-OBL-0142, HW-OBL-0146, HW-OBL-0149 and HW-OBL-0174.
- The list holds 22 lines that no range names: HW-OBL-0170, HW-OBL-0171, HW-OBL-0182, HW-OBL-0185, and HW-OBL-0197 through HW-OBL-0215 without HW-OBL-0199.

The same paragraph also states that three items name a decision and "the other forty-six name none". That count derives from the wrong total. No check compares the sentence to the list. `tools/repo/obligation-register-fixtures.sh` holds the membership of the list and not the prose above it.

## Discharge

This record discharges in one of two ways. The opening sentence states no count and no range that a person keeps aligned with the list by hand. Or a check compares each count and range in the sentence to the list, and the check fails on a difference.
