---
id: HW-OBL-0120
status: current
status_since: 2026-08-14
summary: "A waiver mechanism now ships and reads a conformance rule, so the coverage account of `headwater check` is the one reader of the pair that does not exist."
last_verified: 2026-08-14
title: "No reader takes a waiver against a check finding, so one arm of the escape partition is empty"
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

# No reader takes a waiver against a check finding, so one arm of the escape partition is empty

## Context

[Spec 4](../spec/04-assurance-model.md#suppression) fixes a precedence over three escape mechanisms: waiver, then migration-pending, then suppression. The three inventories partition the escaped findings, so no finding is counted three times. [Spec 7](../spec/07-distribution-and-federation.md#waivers) defines a waiver against a conformance rule. One mechanism carries both populations, because the four required fields and the mandatory expiry are the same in each.

`headwater conformance` shipped the first reader. It reads the `conformance.waivers` block of the consumer declaration. It honors a live waiver over a conformance rule, and reports an expired one rather than honoring it. The second reader is the coverage account of `headwater check`, and it does not exist.

The measurement over this repository, at `--now 2026-08-14`:

| population | waivers declared | reader |
|---|---|---|
| conformance rules | 1, on `pin.current`, live until 2027-02-28 | `headwater conformance` |
| check rules | 0, and no key holds one | none |

The coverage line of a run states the asymmetry rather than a count. It reads "no waiver reaches a check finding, 1 migration-pending, 4 suppressed". It said "no waiver mechanism exists" until this change. That was true and stopped being true, and the SARIF adapter carried the same words.

## Obligation

The corpus owes a reader that takes a waiver against a check rule. It owes a coverage account that counts a finding under it, at the precedence spec 4 fixes.

Three questions come with it and none of them is answered here. Whether a check-rule waiver is scoped to the corpus or to a shelf. How a waived finding is reported, given that a waiver is wider than a suppression and the report has to say which mechanism caught it. And what refuses a waiver against the [withholding rule](../spec/06-engine-architecture.md#an-export-profile-carries-a-filter), which spec 7 puts outside the mechanism and which nothing enforces while the population is empty.

That last one is the reason this record is worth opening rather than leaving to the day somebody needs it. **The exclusion is unenforced and untestable today, because no waiver can reach a check rule at all.** A guard that has never had a population to refuse is a guard nobody has measured.

## Discharge

Three things discharge it. A reader in `headwater-check` that takes the same `Waiver` value `headwater-conformance` already reads. A coverage bucket at the declared precedence. A fixture in which a waiver over the withholding rule is refused by name.

The record closes when a run over a corpus that holds a check-rule waiver reports that finding as waived rather than suppressed. The withholding refusal owes a failing fixture by the same day.
