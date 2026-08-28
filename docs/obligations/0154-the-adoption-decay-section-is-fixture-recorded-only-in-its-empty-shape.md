---
id: HW-OBL-0154
status: draft
status_since: 2026-08-28
summary: "Every section of the recorded audit report renders populated except the adoption one, whose fixture tree declares no payload."
last_verified: 2026-08-28
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

A second recorded fixture over a tree that declares a payload closes this, and the audit crate already has the two halves it needs. What it does not have is a fixture tree with a lock, because `Series` is a parameter of `take` rather than something the crate reads.

The reader of this record is this repository alone, so it waits.
