---
id: HW-OBL-0130
status: draft
status_since: 2026-08-25
summary: "Q31 rules that this repository becomes public and leaves the date unset. The date belongs to the owner alone, and no measurement, no build and no adopter produces one."
last_verified: 2026-08-25
title: "The publication date of this repository is unset and the owner alone sets it"
waiting_on: ruling
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0031
---

# The publication date of this repository is unset and the owner alone sets it

## Context

[HW-DR-0031](../decisions/0031-q31-whether-this-repository-becomes-public-and-when.md) rules that this repository becomes public. The same record leaves the date unset, and it states who may supply one. "The date belongs to the owner alone. No measurement produces it, no build produces it, and no adopter produces it."

**The posture is settled and the schedule is not.** [HW-DR-0011](../decisions/0011-license-and-distribution-posture.md) ratified Apache-2.0, which settles the terms of redistribution. [HW-DR-0016](../decisions/0016-public-presence.md) settles what a public presence is. Neither record carries a date, and neither record asks for one.

**Three measurements say what a reader outside this repository can reach today.** `gh api repos/headwater-ai/headwater --jq .visibility` reports `private`. `gh api repos/headwater-ai/headwater/releases --jq 'length'` reports `0`. `gh api repos/headwater-ai/headwater/tags --jq 'length'` reports `0`. So there is no clone, no tag and no release artifact that an outside party can obtain.

**The last paragraph of HW-DR-0031 names this record.** "The date is the one thing this record does not carry. An obligation record on the obligations shelf holds it." This is that record. [HW-DR-0030](../decisions/0030-q30-whether-what-an-obligation-waits-on-is-a-state-or-a-property-and-how-many-values-it-takes.md) fixes the value it opens at, and `ruling` is that value.

## Obligation

**The corpus owes the date on which this repository becomes public.** No document of this repository holds one. No verb of this engine derives one. The debt is therefore a single act by a single party, and the record of it is this file.

**The debt is a ruling and not a build.** HW-DR-0030 states what the value means. "`ruling` means that a person has to choose, and that no build and no measurement settles it." HW-DR-0031 states the same thing in its own words, so this record takes that value at birth rather than by a later edit.

**What the absence costs the obligations shelf.** 22 open records on this shelf state `waiting_on: adopter`, counted on this tree. An adopter is an organization that runs this engine over a corpus it keeps, and a reader outside this repository is the precondition of one. A private repository admits no such reader, so those 22 records wait behind an act that no work of this repository performs.

**This record states no date and proposes none.** A date written here would be a schedule that this repository composed for its owner. HW-DR-0031 refuses that, and the refusal is why this record exists rather than a plan.

## Discharge

**What would discharge this.** The owner states the date. The record then leaves `ruling` for the value that names the act after it. HW-DR-0030 calls that the ordinary next edit rather than a new document. The work that the date implies goes on the board under the milestone that carries distribution.

**What would not discharge this.** A date that an agent proposes. A date that a run reads off the board. A date that a milestone or an issue implies by the order of its items. Each of those is the act HW-DR-0031 reserves, performed by something that may not perform it.

**What this record does not carry.** The license, the trademark posture and the commercial tier, which HW-DR-0031 leaves where they stood and which [HW-OBL-0094](0094-where-a-commercial-tier-could-sit-and-how-thin-the-trademark.md) records. The channel by which an adopter obtains a build, which is the machinery that the open issues of the distribution milestone already carry. What a first adopter is asked to do once they hold the engine, which the tutorial holds.
