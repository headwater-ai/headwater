---
id: HW-PROBE-a-session-answers-from-the-register-without-opening-the-question-it-replaced
status: current
status_since: 2026-09-11
summary: Both registers carry the same answer, so nothing in what a session says separates the route it took, and the read set is the only instrument left.
last_verified: 2026-09-11
probe_category: navigability
expectation: not_opened
oracle: "none"
title: "A session answers from the register without opening the question it replaced"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft
  evidence_basis: unevidenced
relations:
  examines:
    - HW-REG-open-questions
  traces_to:
    - HW-OBL-0186
---

# A session answers from the register without opening the question it replaced

## Task

Say what this project decided about where scent lives, and name the record that decided it.

The task names no verb and no file. It asks for one settled answer, and this corpus states that answer in two places.

## Expectation

`not_opened` over `HW-REG-open-questions`. The session answered and never opened the document that the decision register superseded.

**This probe grades avoidance and never correctness.** Both registers carry the Q20 line today, word for word, so a right answer is evidence of nothing about the route. A session that opens the superseded document, follows `superseded_by`, and answers correctly is a miss here. That is the intent: it is a session the corpus sent to a document it says nobody may rely on, and the answer hides it. [HW-OBL-0014](../obligations/0014-no-probe-tests-whether-an-agent-reaches-the-adjudication-from.md) owns recovery from that document, and the probe that narrows it grades `opened` over the register. The pair separates a session that never went there from a session that went and recovered. One reading alone separates neither.

**The task names a question that both registers carry, and that is a measured choice rather than a convenience.** On the commit that lands this probe, the superseded document holds 62 headings and the register that replaced it holds 36. So the superseded document is the wider index, and a question the register does not carry would make this predicate unreachable rather than hard. Q20 is carried by both.

**A session that did nothing is not a session that avoided anything.** The grader refuses `not_opened` where the transcript records no tool call at all, because such a session would satisfy the predicate without ever being at risk of failing it. So a refusal leaves the denominator and never the numerator, and the count of refusals is printed beside the rate.

[HW-OBL-0186](../obligations/0186-a-superseded-document-claims-no-reliance-and-nothing-measures-whether-a-session-honors-that.md) records the claim this narrows: reliance is a property a state declares, every mechanism here acts on a relation, and no mechanism acts on a reading.
