---
id: HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded
status: current
status_since: 2026-10-10
summary: The one link of a tombstone section reaches a superseded decision, so a session that follows that link and answers from it misses the decision in force, and the read set shows which one it opened.
last_verified: 2026-10-10
probe_category: navigability
expectation: opened
oracle: "none"
title: "An agent reaches the adjudication when the decision a tombstone links is superseded"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  examines:
    - HW-DR-0087
  traces_to:
    - HW-OBL-0014
---

# An agent reaches the adjudication when the decision a tombstone links is superseded

## Task

Read `docs/spec/09-open-questions.md#69--a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words` and say what this project decided about a limit on the length of a paragraph.

## Expectation

`opened` over `HW-DR-0087`. The transcript shows that the session left the tombstone and the decision it links, and opened the decision in force.

**This probe stands beside [the adjudication probe](an-agent-reaches-the-adjudication-from-the-document-that-lost-it.md) and does not replace it (#1718).** The campaign of 2026-10-03 recorded that probe at 30 of 30 in the `present` arm and at 30 of 30 in the `absent` arm. Its task names the Q8 section of the tombstone, and that section has one link, to HW-DR-0008, which is in force. So a session in either arm that followed the one link reached the adjudication, and the route was too short to miss.

**Here the one link reaches a decision that is not in force.** The Q69 section of the tombstone links [HW-DR-0069](../decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md), and HW-DR-0069 is `superseded`. Its `superseded_by` edge names [HW-DR-0087](../decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md), which is `current`. The two records rule opposite things: HW-DR-0069 adds a paragraph limit, and HW-DR-0087 retires it before it lands. So a session that answers from HW-DR-0069 reports a rule that this project does not have.

**The failure this probe is for is a session that stops one record early.** The shallow session reads the tombstone, follows its one link to HW-DR-0069, and answers from that record. It never opens HW-DR-0087, so the grade finds it not satisfied. The sound session reads the same two documents and then opens HW-DR-0087.

**Two cues point to the decision in force, and a session can miss both.** The tombstone section marks the link to HW-DR-0069 as `(superseded)`. The section above it in the same file is the heading of HW-DR-0087. A session that reads the whole file can take the shorter route and open HW-DR-0087 at once, and that satisfies the predicate. The probe grades only whether the session opened the decision in force, and not the route it took there.

**Why it should separate the arms.** In the `present` arm, the orientation skill sends a session to `headwater explain` before it searches `docs/`. `headwater explain HW-DR-0069` prints the `superseded_by` edge to HW-DR-0087 and marks HW-DR-0087 as the document that governs the reading. In the `absent` arm, a session must read the front matter of HW-DR-0069 and act on its state. The original probe needed no such step.

**What this probe expects, stated before any paid run.** In the `present` arm, 24 of 30. In the `absent` arm, 15 of 30. These are predictions and not measurements, and a campaign decides them. If both arms reach 28 of 30 or more, a session follows a `superseded_by` edge as readily as a link, and this probe is at the ceiling too.

**The task section is the prompt, and nothing else of this document reaches the session.** [Spec 15](../spec/15-the-recorder-contract.md#the-prompt-is-the-task-section-and-the-answer-is-the-final-line) fixes that.

[HW-OBL-0014](../obligations/0014-no-probe-tests-whether-an-agent-reaches-the-adjudication-from.md) records that no probe tests this and that the corpus holds no adjudicated pair. The pair here is a supersession rather than an adjudicated disagreement, so a run of this probe narrows that record and does not close it.
