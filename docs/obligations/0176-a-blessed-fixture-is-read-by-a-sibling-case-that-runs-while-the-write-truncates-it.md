---
id: HW-OBL-0176
status: current
status_since: 2026-09-07
summary: "Under HEADWATER_BLESS one case truncates crates/conformance/fixtures/wrapped.report while a sibling thread of the same binary reads it, which can panic the run and cannot corrupt the artifact."
last_verified: 2026-09-07
title: "A blessed fixture is read by a sibling case that runs while the write truncates it"
waiting_on: adopter
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - engine/crates/conformance/tests/render.rs
    - HW-OBL-0133
---

# A blessed fixture is read by a sibling case that runs while the write truncates it

## Context

Twenty-three test files of this workspace read `HEADWATER_BLESS`, and each one carries a `compare` helper of the same shape. Under the variable the helper calls `std::fs::write(recorded, actual)` and returns. `std::fs::write` truncates the file and then writes the new bytes, so the recorded path holds zero bytes for the length of the call.

`cargo` runs the cases of one test target as threads of one process. In `engine/crates/conformance/tests/render.rs` two cases name one path. `the_report_wraps_every_line_to_the_width` passes `fixtures_dir().join("wrapped.report")` to `compare`, which is the writer. `the_recorded_block_is_the_one_this_width_produces` calls `std::fs::read_to_string` on the same path, and it does that whether or not the variable is set. So a blessed run has a reader and a writer of one file in two threads with nothing between them.

The reader panics on what it can see. An empty read reaches `.expect("the recorded block has lines")` through a `max()` over no lines. A partial read reaches an assertion that the longest line is over `WIDTH - 12`, which a truncated block fails.

A scan for the pair found one instance. The shape is a file that reads `HEADWATER_BLESS`, where one fixture name appears both inside a `compare` call and inside a `read_to_string` call. Twenty-three files carry the variable and one carries the pair.

**The artifact is not at risk, and this is the part to state precisely.** The bytes `compare` writes are `actual`, which is the run's own in-memory render, computed before the call and independent of what any other thread does. Eighteen blessed runs across two independent checkouts fired the race zero times, and every artifact they produced was byte-identical. So the failure mode is a flaky panic in a reader, and blessing is not non-deterministic. The condition predates any one branch.

## Obligation

A suite that re-records its own expectations has one shared mutable file and two threads that reach it. Nothing in the helper, the test file or the runner orders them. The cost of the race is a red run that a rerun clears. That is the class of failure that teaches a reader to rerun rather than to read.

## Discharge

The reader stops being a reader. `the_recorded_block_is_the_one_this_width_produces` asks whether the recorded block was recorded at `WIDTH`, and the rendered string that the writer holds answers the same question. Folding the width assertion into the case that already renders removes the file from the second thread.

Where two cases must keep separate names, the pair is discharged by reading the file once at the top of the target. Any order the runner honors does the same. `--test-threads=1` under the variable is a third route and the weakest, because it holds the property by a flag a caller can drop.

The wider question is whether any other blessed fixture is read outside its `compare`. The scan above answers it for the one shape it matched. A scan over a path built by a helper other than `fixtures_dir` reaches further.
