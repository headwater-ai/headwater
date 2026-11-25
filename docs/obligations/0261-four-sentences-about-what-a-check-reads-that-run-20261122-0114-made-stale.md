---
id: HW-OBL-0261
status: current
status_since: 2026-11-24
summary: "Spec 12 names seven barriers where ten print, and the check contract names only transition rules. CLAUDE.md and the oracle library miss the blocking posture."
last_verified: 2026-11-24
title: "Four sentences about what a check reads that run 20261122-0114 made stale"
waiting_on: build
provenance:
  warrant: asserted
---

# Four sentences about what a check reads that run 20261122-0114 made stale

## Context

Spec 13 holds a finding that has no reader outside this repository. Run `20261122-0114` wrote intake lines 68, 85, 87, 132 and 133. The product owner ruled each one RECORD into the group of stale text that its end-of-run report calls S2. That group held seventeen lines. A record that bundles gaps is split when it passes five, so this record holds the lines about what a check reads. This record checked each sentence again on `0cdd96298`.

## Obligation

- `docs/spec/12-check-layer.md`, in the section on corpus-scoped barriers, says "This paragraph names the seven that headwater check --read-set printed on 2026-10-01". It calls them the full list. The read set now prints ten. `shelf.discriminator.unresolved`, `relation.mention.undeclared` and `relation.scope.ungoverned` are not named. Lines 68 and 132 are this one gap.
- The `--change` row of `docs/interfaces/headwater-check.md` speaks of "the rules that read a transition". Since #1573, `relation.scope.ungoverned` reads additions, and not transitions, so one clause is owed.
- The section of `CLAUDE.md` on what a check does says that `headwater check --strict` "fails on an error and reports everything else". Since #2015 (PR #2046), `Run::fails_strict` also fails on a live finding of any severity that a `posture: blocking` control discharges.
- `hw_oracle_claims` in `tools/probe/oracles/lib.sh` selects `severity error` with `escape none` as the set that a strict run fails on, and reads no `blocked_by`. No blocking control of the standard package fires below error on the probe trees today, so no grade moves yet.

## Discharge

Each item discharges alone, when the sentence or the condition it names says what the engine does, or when a change records why it stays. The record discharges when every item has.
