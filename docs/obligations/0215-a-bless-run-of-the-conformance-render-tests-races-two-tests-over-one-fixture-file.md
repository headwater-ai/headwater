---
id: HW-OBL-0215
status: discharged
status_since: 2026-09-28
summary: "Under HEADWATER_BLESS=1, one render test reads fixtures/wrapped.report while a sibling test in the same process rewrites it, and the reader can panic."
last_verified: 2026-09-28
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

[#1162](https://github.com/headwater-ai/headwater/issues/1162) discharged this record on 2026-09-28. It removed `the_recorded_block_is_the_one_this_width_produces`, which was the reader. The writer, `the_report_wraps_every_line_to_the_width`, made the same two width assertions against the rendered string, with one difference. The reader did not count a line of one word toward the lower bound, and the writer did. A maximum over all lines is never smaller than a maximum over some of them, so the writer's bound was the weaker one. One word longer than the column could hide a fill that stops short on every other line. The same change added that filter to the writer. A scratch mutation shows the gap. It stopped the fill 20 columns short and put a 69-character URL into a remediation. The old bound passed that render, and the new bound failed it. Without bless, `compare` fails when `WIDTH` moves and the file does not. So `wrapped.report` now has one case that names it, and the file did not change. A local test put a 200 ms empty file into the blessed write. With that window, 9 of 20 blessed runs failed before the change. Each failure was the panic that this record names. After the change, 0 of 20 failed. Without the window, 0 of 50 blessed runs of the render target failed, and 0 of 20 blessed runs of the whole crate failed. This record repeats the `render.rs` half of HW-OBL-0176, and the same change discharges both.
