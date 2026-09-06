---
id: HW-OBL-0157
status: current
status_since: 2026-09-06
summary: "The capture report warns that its aggregate crosses two denominators, on a run whose own next line lists seventeen of them."
last_verified: 2026-08-28
title: "The capture report says two denominators over a store that named seventeen taxonomies"
waiting_on: build
---

# The capture report says two denominators over a store that named seventeen taxonomies

## Context

`headwater capture` pools every reading in the store into one assisted fraction. Where more than one taxonomy produced those readings, it warns rather than declining to pool. Over this repository the warning reads:

    17 taxonomies produced these readings, so the aggregate above is across two denominators and is not a trend

Then it lists all 17. The branch is in `engine/crates/cli/src/main.rs`. Its arm binds the count as `many`, and then it writes the word "two" anyway. The one-taxonomy arm beside it interpolates its value correctly.

The warning is right about the thing that matters, which is that the number is not a trend. It is wrong about the arithmetic in the same sentence. A reader who counts the digests below it sees the contradiction at once.

[The help-string audit](../reviews/the-sixty-four-restored-help-strings-checked-against-the-binary.md) found this. That audit corrected the `capture` description, which claimed the verb never pools at all. The description is now true of the behavior. The behavior still describes itself wrong.

## Obligation

The sentence has to state the count it holds. Nothing else in the report is affected, no verdict moves, and no exit status reads it.

The wider question is whether a run report should carry a number a reader can check against the same report's next line. Nothing in this engine holds a report sentence to the values around it. [HW-OBL-0156](0156-a-help-string-and-the-interface-contract-that-restates-it-can-both-be-false-with-the-whole-suite-green.md) records the same shape for the help.

## Discharge

The branch interpolates its own count. A case reads the sentence back, over a store carrying readings under more than two locks.
