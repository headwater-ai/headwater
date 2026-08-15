---
id: HW-OBL-0089
title: "A profile names its rules and never states them"
status: current
status_since: 2026-08-13
last_verified: 2026-08-15
summary: "`ste_house` names a controlled language and a profile, and four of its rules are constants in a Rust file."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-taxonomy-model
---

# A profile names its rules and never states them

## Context

`regimes.language.ste_house` writes `controlled: ASD-STE100` and `profile: house`, and the engine matches that pair against a closed table and then applies four rules it holds in code. The 25-word limit is a constant in a Rust file.

## Obligation

So an adopter who wants 20 words has no declaration to write. One who wants the semicolon rule without the spelling rule waits for a build.

**The spelling rule is also smaller than the sentence it prints.** The finding says that the regime declares the tag `en-US`. The rule is a table of 24 words, built from the seven stems that `CLAUDE.md` names. So `colour`, `centre`, `analyse`, `recognise`, `defence` and `whilst` all pass a governed document. This corpus writes none of them outside a fenced code block, measured over the three governed shelves. The gap costs this repository nothing today, and it costs an adopter the difference between a tag and a list.

## Discharge

`source_form` and `retired_terms` went the other way in this milestone, and both are data. What is open is whether the remaining four rules of the profile should follow them. The cost is a value set per rule of a controlled language, and no specification states one. [Spec 2](../spec/02-taxonomy-model.md#language-is-declared-not-assumed) is where the statement belongs.
