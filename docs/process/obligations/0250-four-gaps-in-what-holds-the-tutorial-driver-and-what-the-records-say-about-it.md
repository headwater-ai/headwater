---
id: HW-OBL-0250
status: current
status_since: 2026-11-24
summary: "A record states an old claim count for the tutorial driver, and a decision does not say that CI runs it against the release. Two of its modes have no case."
last_verified: 2026-11-24
title: "Four gaps in what holds the tutorial driver and what the records say about it"
waiting_on: build
provenance:
  warrant: asserted
relations:
  traces_to:
    - HW-DR-0061
    - HW-OBL-0145
---

# Four gaps in what holds the tutorial driver and what the records say about it

## Context

`hw-run-policy` sends a finding about the build order, its tooling or the CI of this repository to this shelf. Run `20261122-0114` wrote intake lines 19, 20, 25 and 116. The product owner ruled each one RECORD into this record, which its end-of-run report calls P5. This record checked each item again on `0cdd96298`, except the four failed claims, which are as the builder of #1990 reported them.

## Obligation

- HW-OBL-0145 says that `sh .claude/tutorial/fixtures.sh` covers 66 claims. `origin/main` ran 70 before #1985, and after it the driver runs 72 in release mode and 71 without the switch.
- HW-DR-0061 says that the blocking step holds every printed output against a run, and does not name the engine. Since #1985, CI runs the driver with `HEADWATER_TUTORIAL_RELEASE=1`, against the release that the page installs, and not against the engine of the branch.
- The release mode of `.claude/tutorial/fixtures.sh` checks the downloaded archive with `sha256sum -c`. The verifier of PR #2003 removed that check and every case stayed green, because no case serves a tampered archive.
- Without the release switch, the driver fails 4 of 71 claims. Steps 7 and 9 state 8 and 22 check instances where the run prints 9 and 25, and the `explain` output of step 14 moved. CI runs only the release mode, so this mode drifted with nobody to see it. The page holds to v0.6.1 under the release switch, so no adopter meets the drift today. The counts 9 and 25 come due when v0.7.0 is cut and the page moves to it.

## Discharge

Each item discharges alone, when the document or the suite it names says or holds what the item asks. A change that records why the item stays also discharges it. The record discharges when every item has.
