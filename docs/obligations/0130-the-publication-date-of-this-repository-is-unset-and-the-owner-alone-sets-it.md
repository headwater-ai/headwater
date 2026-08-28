---
id: HW-OBL-0130
status: current
status_since: 2026-08-27
summary: "Q31 originally left the publication date unset. The owner set 2026-09-08, and this record remains current until the repository is public."
last_verified: 2026-08-27
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

[HW-DR-0031](../decisions/0031-q31-whether-this-repository-becomes-public-and-when.md) rules that this repository becomes public on 2026-09-08. The owner set the date that the record once left unset.

**The posture and schedule are settled.** [HW-DR-0011](../decisions/0011-license-and-distribution-posture.md) ratified Apache-2.0, which settles the terms of redistribution. [HW-DR-0016](../decisions/0016-public-presence.md) settles what a public presence is.

**Three measurements say what a reader outside this repository can reach today.** `gh api repos/headwater-ai/headwater --jq .visibility` reports `private`. `gh api repos/headwater-ai/headwater/releases --jq 'length'` reports `0`. `gh api repos/headwater-ai/headwater/tags --jq 'length'` reports `0`. So there is no clone, no tag and no release artifact that an outside party can obtain.

**The last paragraph of HW-DR-0031 named this record.** This record held the date under the `ruling` value [HW-DR-0030](../decisions/0030-q30-whether-what-an-obligation-waits-on-is-a-state-or-a-property-and-how-many-values-it-takes.md) defines.

## Obligation

**The corpus owes the public release on 2026-09-08.** No document, engine verb, measurement or adopter could derive the date. The owner set it.

**The debt was a ruling and not a build.** HW-DR-0030 states what the value means. The owner performed that act on 2026-08-27.

**What the absence costs the obligations shelf.** 22 open records on this shelf state `waiting_on: adopter`, counted on this tree. An adopter is an organization that runs this engine over a corpus it keeps, and a reader outside this repository is the precondition of one. A private repository admits no such reader, so those 22 records wait behind an act that no work of this repository performs.

**This record did not state or propose a date.** The owner supplied 2026-09-08. The repository remains private until the scheduled release.

## Discharge

**This record discharges when the repository is public on 2026-09-08.** Publication work remains under the distribution milestone.

**What did not discharge this.** The owner's date ruling did not make the repository public. An agent, a run, a milestone or an issue could not set the date.

**What this record does not carry.** The license, the trademark posture and the commercial tier, which HW-DR-0031 leaves where they stood and which [HW-OBL-0094](0094-where-a-commercial-tier-could-sit-and-how-thin-the-trademark.md) records. The channel by which an adopter obtains a build, which is the machinery that the open issues of the distribution milestone already carry. What a first adopter is asked to do once they hold the engine, which the tutorial holds.
