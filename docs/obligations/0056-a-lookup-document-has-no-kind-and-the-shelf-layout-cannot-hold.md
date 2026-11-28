---
id: HW-OBL-0056
title: "A lookup document has no kind, and the shelf layout cannot hold one"
status: discharged
status_since: 2026-11-28
waiting_on: ruling
last_verified: 2026-11-28
summary: "The glossary sits on the specification shelf with a layout it cannot satisfy, and it carries a sequence that nothing cites."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-glossary
    - HW-DR-0117
---

# A lookup document has no kind, and the shelf layout cannot hold one

## Context

[The glossary](../spec/glossary.md) sits on the specification shelf, whose layout is `{sequence:02d}-{slug}.md`, and its path carries no number. The typing gave it `sequence: 14`, which no reader cites and no document uses.

## Obligation

Three answers are available. The layout is advisory, or the file is renamed, or a lookup surface is a kind that the tradition lacks.

## Discharge

Nothing chooses between the three today. Each one costs something different, and no rule in the engine reports the mismatch.

**This record is discharged.** [HW-DR-0117](../decisions/0117-the-specification-series-names-a-part-by-its-slug-and-carries-no-sequence.md) chose the second answer for every part of the series, not for the glossary alone. headwater/standard 5.0.0 lays out the specification shelf as `{slug}.md`, which `glossary.md` already satisfies, and it deletes the `sequence` facet, so the value that nothing cited has no declaration. `taxonomy audit` counts `glossary.md` as the one name on the shelf that the layout renders. The question of a lookup kind did not need an answer for the layout to hold.
