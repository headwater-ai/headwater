---
id: SPEC-FIX-suppressed
doc_type: design_spec
status: current
status_since: 2026-02-01
summary: The failing and passing fixture of the suppression inventory, with one directive of each state.
---

# Suppressed findings

Every directive below is deliberate. The four states of the inventory are
applied, expired, unused and refused, and this file carries one of each.

## A file-scoped directive, applied

<!-- headwater allow=voice.forbidden_construction scope=file until=2030-01-01 reason=accepted_deviation note=this fixture states a plan on purpose -->

The loader will be rewritten once the lock format settles.

## A block-scoped directive, applied

This sentence is written to run past the limit that the house profile sets for
descriptive text, and it keeps going for long enough that a reader loses the
thread of the clause. <!-- headwater allow=language.controlled.not_met scope=block until=2030-01-01 reason=false_positive note=the length is the fixture -->

## A directive that lapsed

<!-- headwater allow=language.controlled.not_met scope=block until=2026-01-01 reason=false_positive note=this one is past its expiry -->

This sentence is also written to run past the limit that the house profile sets
for descriptive text, and it goes on far enough to be quite certain of the
count.

## A directive this engine cannot read

<!-- headwater allow=language.controled.not_met scope=file until=2030-01-01 reason=false_positive note=the rule name is mistyped -->

## A directive that matches nothing

<!-- headwater allow=link.fragment.unresolved scope=file until=2030-01-01 reason=accepted_deviation note=this file writes no fragment into itself -->
