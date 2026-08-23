---
id: HW-OBL-0119
status: current
status_since: 2026-08-14
waiting_on: adopter
summary: "Of the six readings `taxonomy audit` takes, one has a number a taxonomy states and five have none, so five report a population and no verdict."
last_verified: 2026-08-14
title: "An audit reading carries no declared bar, so a distribution cannot become a finding"
provenance:
  warrant: proposed
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-engine-architecture
---

# An audit reading carries no declared bar, so a distribution cannot become a finding

## Context

[Spec 6](../spec/06-engine-architecture.md#taxonomy-validate-versus-taxonomy-audit) says the findings of `taxonomy audit` are "advisory by construction, because a young or small corpus fails differentiation for reasons that are not defects". That sentence reads as a statement about severity. The build of the verb found that it is a statement about something earlier. For five of the six readings there is nothing to be advisory *about*, because no declaration says what the reading would have to cross.

The one exception is `stale_after_days`, which the freshness facet declares. `taxonomy audit` is its first reader in this engine, and the taxonomy states 180 days. So "this relation has a half on a document past the window" is a verdict out of the corpus rather than out of the code. Every other reading arrives with a population and a number, and with nothing to compare the number against.

The measurement over this repository, at `--now 2026-08-14`, is what makes the gap concrete rather than theoretical.

| reading | what it reported here | the bar it would need |
|---|---|---|
| staleness by creator | 0 halves past 180 days, of 397 | declared |
| capture by relation | `governs` at 0.6%, `traces_to` at 80.7% | none |
| facet differentiation | `status` at 100% one value, over 4 declared | none |
| facet orthogonality | one pair where a value fixes another | none |
| relation drift by family | 0 relations contradict a family default | none |
| discriminator distribution | `spec_series` at 13 of 16 one kind | none |
| state dwell | every document in one state, 0 to 13 days | none |

The `status` row is the sharpest. [Spec 2](../spec/02-taxonomy-model.md#facet-acceptance-tests) puts the acceptance test on a facet as "does it separate documents that a reader treats differently". The state facet holds four declared values, every document of this corpus carries one of them, and it separates nothing at all today. That is a fact about a young corpus rather than a defect in the taxonomy, which is why the engine cannot decide it.

`governs` at 0.6% of eligible documents is the case [spec 2](../spec/02-taxonomy-model.md#who-creates-each-edge) writes out in words. It reads: "a relation declared `created_by: author` but present on 4% of eligible documents visibly receives no maintenance". Spec 2 names 4% and names no rule, so 0.6% is a number a person reads.

## Obligation

The corpus owes a statement of which audit readings carry a bar and where each bar is written. Three answers are open and the record takes none of them.

**A bar could live in the taxonomy, beside the reading it governs.** A facet would declare the concentration it accepts, and a relation would declare the capture it expects. That puts the judgment where the rest of the schema is, and it costs every adopter a decision on first contact.

**A bar could live in the corpus, as an obligation with a control.** [Spec 4](../spec/04-assurance-model.md#obligations-are-data) already carries that shape. An audit reading is then evidence for a claim rather than a rule of its own.

**A bar could stay absent, and the readings stay readings.** This is what ships, and it is defensible. [Spec 6](../spec/06-engine-architecture.md#taxonomy-validate-versus-taxonomy-audit) calls the verb "a periodic design review with a tool attached", and a design review reads numbers. What it costs is a corpus with no trend anybody is answerable for. A facet that stops separating documents then does so with nothing to report it.

The engine may not settle this on its own. A threshold that this engine invented would be a verdict derived from nothing a corpus declared. That is the failure mode [spec 12](../spec/12-check-layer.md) refuses for a check, and an advisory posture does not excuse it.

## Discharge

A first adopter reads one audit and says which of its readings they wanted a verdict from. That is the evidence rather than a further argument here. The readings that need a bar over 176 documents are not the readings that need one over ten thousand.

Until then the report states what it does today. The declared window is the one bar, and every other section prints its population and no verdict. `headwater taxonomy audit` is the instrument, and the row of the table above is the reading.
