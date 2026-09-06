---
id: HW-OBL-0137
status: current
status_since: 2026-09-06
last_verified: 2026-08-26
title: "A British spelling reaches a serialized grain string, so the rename is a corpus change and a derived-bytes change at once"
summary: "A British spelling in the serialized grain string entangles a prose rename with cached verdicts, a read-set artifact, and recorded fixtures."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
waiting_on: adopter
---

# A British spelling reaches a serialized grain string, so the rename is a corpus change and a derived-bytes change at once

## Context

CLAUDE.md rules American spelling everywhere, and `language.controlled.not_met` enforces it from the `SPELLINGS` table in `engine/crates/check/src/language.rs`. Neither `neighbourhood` nor `centre` is in that table, so no rule reads a `.rs` file and governed prose alone would pass. A count against `main` found 60 occurrences in `engine/**/*.rs` and 11 more under `docs/`, three times the figure Issue #191 reported. Issue #191 measured the gap and split this obligation off rather than fold a fixture re-bless into a change about readability.

## Obligation

The corpus owes one spelling of this concept, not three populations diverging under one name. `engine/crates/check/src/scope.rs` serializes `Grain::Neighbourhood { .. }` to the literal string `neighbourhood`. That string reaches cache keys, the read-set artifact a merge reads, and every recorded fixture that pins the bytes.

A rename of the identifier is a compile-time change, and the compiler finds every site across crates because `neighbourhood_scope` is public. A rename of the serialized string is a corpus change and a derived-bytes change at once. The fixtures need a deliberate re-bless that names the moved bytes, not a silent pass under `HEADWATER_BLESS=1`.

`docs/spec/12-check-layer.md` and `docs/spec/glossary.md` use `Neighbourhood` as the specification's own term, at lines 94, 211, and 358, and at lines 444 and 618. Renaming the code alone leaves the engine and its own specification using two words for one concept.

A fourth question waits on the other three: whether `neighbourhood` and `centre` join the `SPELLINGS` table. Joining it lets `check --fix` rewrite the same specification sentences, arriving at the same corpus change through a different door.

## Discharge

Discharge needs a single rename that moves the identifier, the corpus term, and the serialized string together, not in separate passes. The change description must state the serialized string that moved and name every re-blessed fixture. `HEADWATER_BLESS=1` must not carry that diff silently. A decision must also settle whether `neighbourhood` and `centre` join the `SPELLINGS` table, and which of the corpus and the table moves first. The full suite and all five gates must pass, with the cache identity `hits + misses + unkeyed == instances` reported before and after. This waits on an adopter to choose the rename order and carry the fixtures through it.
