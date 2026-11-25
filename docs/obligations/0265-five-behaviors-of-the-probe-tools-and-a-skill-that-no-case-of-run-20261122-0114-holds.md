---
id: HW-OBL-0265
status: current
status_since: 2026-11-24
summary: "Mutants of two oracle tools, the orient skill gate and the grade of an escaping path survive. No case holds the analysis verification method."
last_verified: 2026-11-24
title: "Five behaviors of the probe tools and a skill that no case of run 20261122-0114 holds"
waiting_on: build
provenance:
  warrant: asserted
---

# Five behaviors of the probe tools and a skill that no case of run 20261122-0114 holds

## Context

Spec 13 holds a finding that has no reader outside this repository. Run `20261122-0114` wrote intake lines 10, 11, 22, 24 and 156. The product owner ruled each one RECORD into the group that its end-of-run report calls S3, behavior that no case holds. That group held eight lines. A record that bundles gaps is split when it passes five, so this record holds the probe tools and the skill. The mutants are as the verifiers reported them, and this record did not run them again.

## Obligation

- In `tools/probe/oracles/nothing-else-broken.sh`, the verifier of PR #1994 deleted a loop that it called `more_sound`, and the suite stayed green. Nothing tests the oracle runner itself.
- In `tools/probe/oracles/`, a summary that negates its claim and keeps every term passes the concept lines of the `repair_check` oracles (PR #1994).
- No test reads the meaning of the direct-instruction gate of the `headwater-orient` skill. Mutants M4 and M5 of PR #2000 survived.
- In `engine/crates/probe/src/grade.rs`, a produced path such as `../x` or `~/` is relative to the workspace and escapes it. The grade does not treat it as outside, and no fixture covers it (PR #2002).
- `facets.verification_method` in `docs/taxonomies/requirement-verification/bundle.yml` lists `analysis`, and no fixture or test names that value. An adopter-test criterion with `verification_method=analysis` would hold it.

## Discharge

Each item discharges alone, when a case fails on the mutant it names, or when a change records why the behavior needs no case. The record discharges when every item has.
