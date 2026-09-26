---
id: HW-OBL-0215
status: current
status_since: 2026-09-26
summary: "Under HEADWATER_BLESS=1, one render test reads fixtures/wrapped.report while a sibling test in the same process rewrites it, and the reader can panic."
last_verified: 2026-09-24
title: "A bless run of the conformance render tests races two tests over one fixture file"
waiting_on: build
---

# A bless run of the conformance render tests races two tests over one fixture file

## Context

The integrator of #959 in run `20260924-0411` found this race on main `0b6f8231`. The product owner ruled it Record, because only this repository's test runs meet it.

## Obligation

In `engine/crates/conformance/tests/render.rs`, under `HEADWATER_BLESS=1`, `the_recorded_block_is_the_one_this_width_produces` reads `fixtures/wrapped.report`. A sibling test in the same process rewrites that file. The reader panicked with "the recorded block has lines". The test passes without bless, and it passed under bless at `82fc576a`.

## Discharge

This record discharges when no two tests in one process read and write the same fixture under bless, and a bless run of the suite passes on repeated runs.
