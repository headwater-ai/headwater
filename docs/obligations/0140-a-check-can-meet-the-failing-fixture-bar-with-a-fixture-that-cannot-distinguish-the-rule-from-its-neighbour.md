---
id: HW-OBL-0140
status: current
status_since: 2026-09-06
last_verified: 2026-10-01
title: "A check can meet the failing-fixture bar with a fixture that cannot distinguish the rule from its neighbour"
summary: "A fixture set can meet the failing-fixture bar in full and never exercise the choice between a rule and its neighbor."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
waiting_on: ruling
---

# A check can meet the failing-fixture bar with a fixture that cannot distinguish the rule from its neighbour

## Context

[Spec 12](../spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship) sets the floor for a new check: one fixture it fails, and one it passes. `link.fragment.unresolved` met that floor, with passing fixtures, failing fixtures, and a unit test over the function that carries its decision. The defect recorded as #206 shipped inside it anyway, and lived for a whole release, wrong in both directions at once.

The implemented rule counted a prior anchor as a repeat when it began with the slug and a hyphen, rather than counting an exact repeat. The unit test asserted anchors over three identical headings. On three identical headings, the two rules agree, so the test could not distinguish them. The defect needed a fourth heading whose slug extends another one, an input this corpus carries and the fixture did not. Issue #209 later added three fixtures that ask the harder question, and each one fails on `main` before that change.

[HW-OBL-0079](0079-a-correctness-root-that-is-a-type-has-no-failing-fixture.md) records the neighboring failure, where a correctness root that is a type has no failing fixture to write at all. This is the opposite case: a fixture set that meets the written bar and still could not have caught what it shipped with.

## Obligation

[#1487](https://github.com/headwater-ai/headwater/issues/1487) wrote the sharper bar. [Spec 12](../spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship) now states the discriminating condition beside the firing condition: an input must exist on which the implemented rule and a plausible neighboring rule disagree. It carries `link.fragment.unresolved` before #209 as the worked case, with the two candidate rules and the input that separates them. `.claude/skills/headwater-taxonomy/SKILL.md` states the same condition, and the pointer in `.claude/skills/headwater-engine/SKILL.md` names both conditions. `sh .claude/skills/fixtures.sh` holds the sentence in all three files.

One thing is still owed. The owner accepts that wording as the bar that governs every check yet to ship, or rewrites it. The rewrite changes a rule for every future author, so an agent does not accept it.

Whether every shipped check meets the sharpened bar is a separate survey, and #206 is the only instance measured here. This record does not cover that audit.

## Discharge

This closes when the owner accepts the wording of the discriminating condition in spec 12, or replaces it with a wording that `sh .claude/skills/fixtures.sh` then holds in the spec and in the two skill files.
