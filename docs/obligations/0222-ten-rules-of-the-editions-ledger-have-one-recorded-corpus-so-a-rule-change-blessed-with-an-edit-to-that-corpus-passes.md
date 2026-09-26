---
id: HW-OBL-0222
status: current
status_since: 2026-09-26
summary: "The editions ledger pins 10 of its 31 rules over one recorded corpus only. For each of them, a bless re-records a moved verdict when the same change adds a byte to that corpus."
last_verified: 2026-09-26
title: "Ten rules of the editions ledger have one recorded corpus, so a rule change blessed with an edit to that corpus passes"
waiting_on: build
---

# Ten rules of the editions ledger have one recorded corpus, so a rule change blessed with an edit to that corpus passes

## Context

[#1105](https://github.com/headwater-ai/headwater/issues/1105) added `engine/crates/check/fixtures/editions.ledger`, which `engine/crates/check/tests/editions.rs` reads. Each row pins one rule over one recorded corpus. The row holds the rule `VERSION`, a fingerprint of the corpus, a digest of every verdict and an instance count. Under `HEADWATER_BLESS=1`, the test re-records a row when its `VERSION` or its corpus moved. It refuses a row when the verdicts moved over the same corpus at the same `VERSION`.

Run `20260926-1327` found this gap. The product owner ruled it Record, because the ledger has no reader outside this repository.

## Obligation

On 2026-09-27, the ledger held 31 rules. Ten of them have one row, so one corpus holds each of the ten:

- `check`: `facet.value.blank`, `language.controlled.not_met`, `language.retired_term.used`, `language.source_form.not_met`, `section.required.missing` and `voice.forbidden_construction`.
- `terminal-dependency`: `lifecycle.dependency.on_terminal`.
- `state-set-twice`: `lifecycle.state.set_twice`.
- `editions/governs-suspect`: `relation.target.suspect`.
- `acceptance-criterion-proven`: `relation.target.verification.suspect`.

A change can move the verdicts of one of these rules and add one byte to its corpus in the same commit. The fingerprint then moves, so a bless re-records the row. No second row holds the old verdicts over a corpus that did not move. So the change passes without a `VERSION` raise, which is the thing the ledger exists to force. A rule with two or more rows fails in that case, unless the same change also moves each of its other corpora.

## Discharge

This record discharges in one of two ways. Each of the ten rules gains a second recorded corpus in the ledger, and a case holds that no rule has fewer than two. Or a bless refuses to re-record the row of a single-corpus rule when that row's verdicts moved, and a case shows the refusal.
