---
id: HW-OBL-0109
title: "The capture-cost denominator is set by declaration and not by work"
status: current
status_since: 2026-08-14
last_verified: 2026-08-14
summary: "Over the eight scaffoldable kinds the verb supplies 48 of 56 units, the one hand entry of each kind is the summary, and the body counts nowhere."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-authoring-and-lifecycle
---

# The capture-cost denominator is set by declaration and not by work

## Context

[Spec 3](../spec/03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric) names four terms of the assisted fraction. They are the required front matter, the required sections, the identifier, and the halves of each proposed edge. A section counts as its heading and never as its prose.

`headwater new` over every concrete kind of this repository is a reading of that denominator. Eight kinds scaffold and `specification` refuses, which [HW-OBL-0107](0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md) already holds. No run named a relation, so no run proposed an edge.

| kind | assisted | required sections |
|---|---|---|
| `decision` | 8 of 9 | 3 |
| `obligation_record` | 8 of 9 | 3 |
| `decision_register` | 6 of 7 | 0 |
| `design_spec` | 6 of 7 | 0 |
| `obligation_register` | 6 of 7 | 0 |
| `review_prompt` | 5 of 6 | 0 |
| `review_record` | 5 of 6 | 0 |
| `evaluation` | 4 of 5 | 0 |

The eight runs together read **48 of 56**. That is front matter 34 of 42, sections 6 of 6, identifiers 8 of 8, and no edge half at all. The store at `.headwater/capture-cost.jsonl` holds the eight readings, and `headwater capture` computes the total.

Three facts follow from the table, and each one is about the denominator rather than about the tooling.

**The one hand entry of every kind is the summary.** In all eight runs the verb reports exactly one field that no declaration determines, and it is `summary` each time. So the unassisted half of this measurement is one sentence.

**Six of the eight kinds require no section at all.** For those six the denominator excludes the body of the document completely. An `evaluation` reads 4 of 5, and a person writes every word under the title.

**A kind that requires more declarations reads higher.** A `decision` reads 8 of 9 and an `evaluation` reads 4 of 5. The difference is what each kind declares rather than what an author typed.

## Obligation

The corpus owes a statement of what the number measures, because two readings of it are available and they disagree.

Under the first reading the assisted fraction is a measure of the scaffolder against the declarations it can fill. That is what the arithmetic does, it is reproducible, and it is a useful design budget for a taxonomy. It is not a measure of author burden.

Under the second reading it is the answer to Grudin, which [spec 10](../spec/10-theoretical-foundations.md#what-the-theory-did-not-settle) names as the strongest claim in the design. Capture cost is what an author pays. Under this reading a metric that never counts prose measures the cheap half of the work and reports a high number for it.

Spec 3 uses the fraction in both ways in one section. It calls a rise in hand entry a sign that the taxonomy demands more than the tooling supports, which is the first reading. It then makes the fraction the test of the agent-authoring claim, which is the second.

A ratio whose numerator and denominator both move with a declaration also has a property that a trend store has to hold. A required facet that the scaffolder can fill raises the number with no change in what a person types. So a reading is comparable only against another reading taken under the same taxonomy. The store records the lock digest of each run for that reason, and `headwater capture` refuses to average readings taken under two of them.

## Discharge

A ruling in the decision register on which of the two readings the metric carries, because the two ask for different instruments.

Under the first reading nothing more is owed. The number is what it is, the store trends it, and the taxonomy is what it reports on.

Under the second reading the metric owes a term for the body. Nothing here proposes one. Spec 12 holds a scaffolder to account for a fabricated fact, and an invented term for author burden would be one. A word count is not author burden either, and a term that measured the wrong thing precisely would be worse than the honest gap.

The measurement above is repeatable. Run `headwater new <kind>` over every concrete kind in a copy of this tree, then `headwater capture` over the copy.
