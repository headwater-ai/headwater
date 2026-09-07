---
id: HW-OBL-0177
status: current
status_since: 2026-09-07
summary: "The value rule prescribes waiting_on=adopter on every self-audit record, HW-DR-0030 defines that value as waiting on an outside corpus, and almost none of those records does."
last_verified: 2026-09-07
title: "The value rule writes adopter on every self-audit record and most of them wait on a build"
waiting_on: adopter
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0030
    - .claude/commands/next-run.md
---

# The value rule writes adopter on every self-audit record and most of them wait on a build

## Context

[HW-DR-0030](../decisions/0030-q30-whether-what-an-obligation-waits-on-is-a-state-or-a-property-and-how-many-values-it-takes.md) rules that `waiting_on` is a state and that the closed set holds four values. It defines each one. `ruling` means a person has to choose. `build` means the answer is known and somebody has to write the code or the prose. `measurement` means an instrument exists and no run of it has been made. `adopter` means an outside corpus has to exist before anything can move.

*The value rule* in `.claude/commands/next-run.md` routes a self-audit finding to this register, and it states the command literally: `headwater new obligation_record --title "<the finding, stated as a debt>" --facet waiting_on=adopter`. The flag is part of the prescription and no sentence beside it asks the author to read the four meanings first.

Measured on 2026-09-07, 43 records are listed under [what the engine found about itself](../spec/13-open-obligations.md#what-the-engine-found-about-itself). 31 of them declare `adopter`, 9 declare `build` and 3 declare `ruling`. Across the whole obligations shelf 53 records declare `adopter`. The 22 that sit outside that heading are the population HW-DR-0030 measured on 2026-08-30. Its reading over 129 records was `build` 59, `ruling` 40, `adopter` 22 and `measurement` 8.

So every `adopter` value added since that reading comes from the prescription rather than from the definition. Read against HW-DR-0030, almost every one of the 31 is a `build`: a rule to write, an assertion to add, a fixture to record. No outside corpus stands between any of them and the work.

The cost is the query the facet exists to answer. Spec 13 already states three adopter counts on one tree, and the largest of them is the one this facet derives. A reader asking what waits on a first adopter is handed a population that is mostly work this repository can do alone.

## Obligation

A register entry is a claim, and this is a claim written by a command rather than by a reading. The facet was ruled a state, and a state that a scaffolding instruction fixes at filing is the property HW-DR-0030 refused. The corpus owes a correction of the 31 values and a repair of the instruction that produced them.

## Discharge

Each of the 31 records is read for the act that comes next. The value is set to the one of the four that names it. That reading is the same one HW-DR-0030 made over 129 records, and its Discharge section describes the method.

*The value rule* is repaired when the command it prints stops naming a value. `headwater new obligation_record` refuses a run that states none. The refusal is the point, because a record that does not say what it waits on is not written. The instruction can name the flag, name the four meanings, and leave the choice to the author.
