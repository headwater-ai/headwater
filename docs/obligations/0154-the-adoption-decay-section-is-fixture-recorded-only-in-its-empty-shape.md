---
id: HW-OBL-0154
status: discharged
status_since: 2026-11-27
summary: "A second recorded audit report holds the adoption section populated, over a series the case injects, so no lock was needed."
last_verified: 2026-11-27
title: "The adoption decay section is fixture-recorded only in its empty shape"
waiting_on: build
---

# The adoption decay section is fixture-recorded only in its empty shape

## Context

`engine/crates/audit/fixtures/audit.report` records the whole report of one run over the tree under `engine/crates/audit/fixtures/`. That file is the strongest instrument the audit crate has. Nothing outside the crate can move those bytes, and the clock is injected. Every other section of it renders over a populated corpus. The creator rows carry relations, the warrant rows carry documents, and the layout rows carry three separable answers.

The adoption section does not. The fixture tree has no lock and it declares no payload. So the recorded report holds the section in one shape: no reading, no task, and no denominator for the fraction.

## Obligation

Every other shape of that section is asserted by substring rather than held byte for byte. `engine/crates/audit/tests/readings.rs` names the words of an open payload, a discharged one, two digests and an unreadable line. `engine/crates/cli/tests/adoption.rs` names a few of the same words end to end. A substring case passes over a report whose surrounding line moved. It says nothing about the order of the lines, or about the shape of the whole.

The identifier of this record is `HW-OBL-0154` rather than the number the scaffolder minted. Two obligation drafts were open elsewhere at the numbers below it when this landed.

## Discharge

Discharged on 2026-11-27 by [#2132](https://github.com/headwater-ai/headwater/pull/2132), slice D of [#1494](https://github.com/headwater-ai/headwater/issues/1494). The case `the_fixture_tree_with_an_adoption_series_produces_the_recorded_audit` in `engine/crates/audit/tests/readings.rs` renders the same fixture tree with a populated `Series`, and records the whole report in `engine/crates/audit/fixtures/audit-adoption.report`. The series holds one open task and one discharged task, two recorded readings under two digests, and one line the store cannot read. So the recorded file holds the pairs line, both digests, the per-task trend and the unreadable line, in order.

No lock and no second tree were needed, because `Built::audit_over` already takes the `Series` as a closure. The text below is the condition as it was recorded, and its last paragraph was stale on that point.

A second recorded fixture over a tree that declares a payload closes this, and the audit crate already has the two halves it needs. What it does not have is a fixture tree with a lock, because `Series` is a parameter of `take` rather than something the crate reads.

The reader of this record is this repository alone, so it waits.
