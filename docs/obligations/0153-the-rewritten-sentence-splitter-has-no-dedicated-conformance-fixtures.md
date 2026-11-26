---
id: HW-OBL-0153
status: discharged
status_since: 2026-11-27
summary: "A data-driven set of 15 recorded cases now holds the sentence splitter, rebuilt from its rules and failure classes."
provenance:
  warrant: asserted
  agency: agent
  drafted_by: codex
  activity: draft
  evidence_basis: evidenced
last_verified: 2026-11-27
title: "The rewritten sentence splitter has no dedicated conformance fixtures"
waiting_on: build
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

Discharged on 2026-11-27 by [#2132](https://github.com/headwater-ai/headwater/pull/2132), slice D of [#1494](https://github.com/headwater-ai/headwater/issues/1494). `engine/crates/doc/fixtures/sentences/` holds 15 cases. Each case is a Markdown body and a recorded `.sentences` file. The recorded file has one line for each sentence: the start position, the word count and the text. The runner `segmented_documents_produce_the_recorded_sentences` in `engine/crates/doc/tests/fixtures.rs` compares them and re-records them under `HEADWATER_BLESS=1`.

Q5 recorded the count of 32 and not the 32 cases, so the cases that caused the rewrite could not be recovered. The set covers the three documented rules and each failure class that the 27 inline tests name. It also holds one excerpt of real corpus prose. The module comment of `sentences.rs` now names the fixture location and states this limit.
