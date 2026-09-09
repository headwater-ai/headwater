---
id: HW-OBL-0182
status: current
status_since: 2026-09-09
summary: "publish_with_retry retries a 429 forever, so GitHub Actions' unstated default timeout is its only ceiling, and nobody has measured it."
last_verified: 2026-09-09
title: "The publish-crates retry loop has no retry ceiling"
waiting_on: measurement
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-sonnet-5
  activity: measure+draft
  evidence_basis: evidenced
---

# The publish-crates retry loop has no retry ceiling

## Context

`publish_with_retry` in `.github/workflows/publish-crates.yml` wraps `cargo publish` in a `while true` loop. On a 429 from crates.io's `PublishNew` limiter, it reads the "try again after" timestamp out of `cargo publish`'s own error. It sleeps to that moment plus a five-second margin, and calls `cargo publish` again. Any failure that is not a 429 exits the loop with a non-zero status. A 429 never does. Twenty-three crates share one workspace, and most of them are new names. The comment above the function calls this expected, and not a failure.

The loop carries no counter and no accumulated-wait check. Nothing in the file bounds how many times it may retry one crate, or how long the job may run in total. The job's `timeout-minutes` is the only outer bound, and this workflow does not set it. GitHub Actions then applies its own default of 360 minutes. That default belongs to the runner, and not to a decision this workflow states. Nothing here says whether 23 crates, behind a limiter that reopens every ten minutes, can reach it.

## Obligation

The corpus owes a measurement of this loop's real retry count and elapsed time. It also owes a decision on the ceiling. The workflow may keep the unstated 360-minute default, or state its own `timeout-minutes` and retry limit instead. Until that measurement exists, a dispatch can run for six hours against the rate limiter. Nothing tells a reader of the run whether that is expected, or whether the workflow has stalled.

## Discharge

A measurement of this loop's retry count and elapsed time over a real dispatch discharges this, together with a ruling on the ceiling. Record the measurement where a future reader of this workflow will meet it. Adding an explicit `timeout-minutes` or a maximum attempt count to the loop, with the reasoning for the chosen value, discharges it on its own. Leaving the loop as it stands, with no measurement and no stated ceiling, does not.
