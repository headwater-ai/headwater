---
id: HW-OBL-0239
status: current
status_since: 2026-10-09
summary: "A CI record states a step count that the job has passed, and the relevance grade's fixtures and record are ungoverned with a surviving mutant. None has a reader outside this repository."
last_verified: 2026-10-09
title: "Three run-tooling gaps from run 20261003-1026, filed together"
waiting_on: build
provenance:
  warrant: asserted
---

# Three run-tooling gaps from run 20261003-1026, filed together

## Context

`hw-run-policy` sends a finding about the build order, its tooling or the CI of this repository to this shelf. Run `20261003-1026` wrote these as intake lines 3, 4 and 27, from the work on #1670. The product owner ruled each one RECORD at the end-of-run pass on 2026-10-09. Each item was checked on `66dec0f9`. The product owner read the steps of the `headwater` job in `ci.yml`, the `traces_to` list of the blind-grade evaluation, and `cmd_grade` in `tools/run/relevance-grade.sh`.

## Obligation

- [HW-OBL-0145](0145-the-ci-job-named-advisory-carries-every-blocking-shell-suite-in-the-repository.md) says "The job carries 55 steps". The `headwater` job now carries 89 entries in its `steps` list, so the count and the split that follows it are stale.
- The blind-grade evaluation, `docs/evaluations/what-a-blind-grade-of-the-two-routing-paths-offers-showed.md`, now lists `tools/run/relevance-grade.sh` under `traces_to`. No document names `tools/run/relevance-grade-fixtures.sh` or the record under `tools/run/relevance-grade/`, so both stay in the gap that [HW-OBL-0233](0233-thirteen-run-tooling-gaps-from-run-20261002-1233-filed-together.md) names.
- Mutant N5 of `cmd_grade` survives: removing the test that refuses a sample row graded twice leaves the suite green. A case that appends one sample row twice and expects exit 4 with no `grade.txt` would hold it. #1689 changes the same suite, so it is the natural change to carry this case.

## Discharge

Each item discharges alone. The first does when HW-OBL-0145 states the count the job holds or drops it. The second does when a document names the fixtures and the record, and the third when a case kills N5. The record discharges when every item has.
