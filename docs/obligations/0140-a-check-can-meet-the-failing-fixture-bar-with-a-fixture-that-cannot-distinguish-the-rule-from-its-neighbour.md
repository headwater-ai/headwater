---
id: HW-OBL-0140
status: current
status_since: 2026-09-06
last_verified: 2026-08-26
title: "A check can meet the failing-fixture bar with a fixture that cannot distinguish the rule from its neighbour"
summary: "A fixture set can meet the failing-fixture bar in full and never exercise the choice between a rule and its neighbor."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
waiting_on: adopter
---

# A check can meet the failing-fixture bar with a fixture that cannot distinguish the rule from its neighbour

## Context

[Spec 12](../spec/12-check-layer.md#testing-a-check-without-a-failing-fixture-does-not-ship) sets the floor for a new check: one fixture it fails, and one it passes. `link.fragment.unresolved` met that floor, with passing fixtures, failing fixtures, and a unit test over the function that carries its decision. The defect recorded as #206 shipped inside it anyway, and lived for a whole release, wrong in both directions at once.

The implemented rule counted a prior anchor as a repeat when it began with the slug and a hyphen, rather than counting an exact repeat. The unit test asserted anchors over three identical headings. On three identical headings, the two rules agree, so the test could not distinguish them. The defect needed a fourth heading whose slug extends another one, an input this corpus carries and the fixture did not. Issue #209 later added three fixtures that ask the harder question, and each one fails on `main` before that change.

[HW-OBL-0079](0079-a-correctness-root-that-is-a-type-has-no-failing-fixture.md) records the neighboring failure, where a correctness root that is a type has no failing fixture to write at all. This is the opposite case: a fixture set that meets the written bar and still could not have caught what it shipped with.

## Obligation

The corpus owes a sharper statement of the bar in spec 12. Written as "one fixture it fails and one it passes," the bar answers only whether a check can fire. It says nothing about whether any input exercises the choice between the implemented rule and a plausible neighboring rule. A fixture set can meet the floor in full and still never pose that choice.

Spec 12 owes the discriminating condition, in a form an author can check against fixtures already in hand. The question is whether an input exists where the implemented rule and a plausible neighboring rule disagree. The floor named today remains the floor, and it stops being the entire test.

The statement owes a worked case, so an author can measure a fixture set against a real instance rather than a description. `link.fragment.unresolved` before #209 is that case: the fixture, the two candidate rules, and the input that separates them.

Two skill files repeat the bar for an author outside the spec. `.claude/skills/headwater-taxonomy/SKILL.md` carries it, and so does the line in `.claude/skills/headwater-engine/SKILL.md` that points at it. Both owe the same statement the spec carries, and `sh .claude/skills/fixtures.sh` is what checks that they say it.

Whether every shipped check meets the sharpened bar is a separate survey, and #206 is the only instance measured here. This obligation covers the written bar and the two skill files, and not an audit of every check against it.

## Discharge

This closes when spec 12 states the discriminating condition beside the firing condition, and carries the `link.fragment.unresolved` case as the worked example. It also closes when the two skill files say what the spec says, with `sh .claude/skills/fixtures.sh` passing over both. It waits on the adopter accepting that rewrite of a rule that governs every check yet to ship.
