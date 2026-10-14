---
id: HW-OBL-0137
status: current
status_since: 2026-09-06
last_verified: 2026-10-15
title: "A British spelling reaches a serialized grain string, so the rename is a corpus change and a derived-bytes change at once"
summary: "The grain string `neighbourhood` is a value of the versioned JSON report, so a rename is the first shape change with no new member."
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft+revise
  evidence_basis: evidenced
waiting_on: ruling
---

# A British spelling reaches a serialized grain string, so the rename is a corpus change and a derived-bytes change at once

## Context

CLAUDE.md rules American spelling everywhere, and `language.controlled.not_met` enforces it from the `SPELLINGS` table in `engine/crates/check/src/language.rs`. Neither `neighbourhood` nor `centre` is in that table. No rule reads a `.rs` file, so the engine source passes. Governed prose passes too, because the table does not hold the word. Issue #191 measured the gap and split this obligation off, rather than put a fixture re-bless into a change about readability.

## Obligation

The corpus owes one spelling of this concept.

**The string is in a published, versioned format.** `Grain::name` in `engine/crates/check/src/scope.rs` writes `Grain::Neighbourhood { .. }` as the literal string `neighbourhood`. That string reaches cache keys and the read-set artifact. It also reaches `rules[].scope` of the JSON report, which [the `headwater check` contract](../interfaces/headwater-check.md#raising-the-version) defines, and the same member of the SARIF report. `engine/crates/adapter/tests/fixture.json` and `engine/crates/adapter/tests/fixture.sarif` pin it. The releases from v0.2.1 to v0.5.0 shipped this format.

**A rename is the first change to that shape that adds no member.** The contract says that each version of the JSON shape added a member, and that each addition moved the minor. A renamed value removes a value that a reader may branch on. So a rename needs a decision on how the shape version moves: a minor raise, or a major raise to `2.0`.

**Waiting makes the rename cost more.** Before an adopter reads the report, a rename costs a re-bless of the fixtures that pin the bytes, and a version raise. After an adopter reads it, a rename also costs that adopter a migration. So the next act is a ruling, and an adopter is not the trigger.

The rest of the rename is mechanical. A rename of the Rust identifier is a compile-time change, and the compiler finds every site. The specification uses `neighbourhood` as its own term, in `docs/spec/12-check-layer.md` and in `docs/spec/glossary.md`. A rename of the code alone leaves the engine and its own specification with two words for one concept. `centre` occurs in engine source too, and it reaches no serialized value.

## Discharge

The questions for a person: **is the serialized grain string renamed to `neighborhood` before a first adopter?** **Does the JSON shape version then move to the next minor or to `2.0`?** And **do `neighbourhood` and `centre` join the `SPELLINGS` table?** If they join it, `headwater check --fix` rewrites the specification sentences, and that is the same corpus change through a different path.

When the ruling is to rename, one change moves the identifier, the term in the specification, and the serialized string together. The change description states the serialized string that moved and names every re-blessed fixture, so `HEADWATER_BLESS=1` does not carry the diff silently. The contract records the version raise under its own heading. The full suite and every gate pass, and the change reports the cache identity `hits + misses + unkeyed == instances` before and after.

When the ruling is to keep the string, the contract records that `neighbourhood` is a fixed value of the shape. That statement discharges the record.
