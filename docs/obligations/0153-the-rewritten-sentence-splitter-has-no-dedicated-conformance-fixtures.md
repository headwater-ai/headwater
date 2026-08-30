---
id: HW-OBL-0153
status: draft
status_since: 2026-08-27
summary: "The sentence splitter has inline tests, but it has no data-driven corpus that preserves the failures behind Q5's rewrite."
provenance:
  warrant: asserted
  agency: agent
  drafted_by: codex
  activity: draft
  evidence_basis: evidenced
last_verified: 2026-08-27
title: "The rewritten sentence splitter has no dedicated conformance fixtures"
waiting_on: adopter
---

# The rewritten sentence splitter has no dedicated conformance fixtures

## Context

[Q5](../decisions/0005-voice-checking-depth.md) found that sentence segmentation caused 32 of 58 sentence-length findings in one specification run. Commit `25354e6` rewrote the splitter to use Headwater's CommonMark parse instead of raw-line scanning.

The module now has 15 inline tests for its three documented rules. Other correctness roots in `engine/crates/doc` also have data-driven fixture corpora. [Issue #439](https://github.com/headwater-ai/headwater/issues/439) found no equivalent corpus for sentence segmentation.

## Obligation

No dedicated fixture set preserves the real segmentation failures that caused the rewrite. A future splitter change can restore those failures without testing against the recorded cases.

This debt serves Headwater's own test surface today. It therefore belongs in this register under the value rule, recorded rather than planned.

## Discharge

Add a data-driven sentence fixture set to `engine/crates/doc`. Cover the three documented rules and practical regression cases from Q5. Run the set in the crate's normal tests.

Update the module comment to name the fixture location. The update discharges the comment's statement that the module still owes conformance fixtures.
