---
id: HW-OBL-0222
status: current
status_since: 2026-09-27
summary: "A bless of the editions ledger re-records any row whose corpus moved. So a rule change needs no VERSION raise when it edits each corpus whose verdicts it moves. Ten of 31 rules have one corpus."
last_verified: 2026-09-27
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

The function `judge` in `editions.rs` refuses a row only when its verdicts moved and neither its fingerprint nor its `VERSION` moved. When the fingerprint moved, a bless re-records the row, whatever its verdicts did. So a change passes without a `VERSION` raise when each verdict it moves is over a corpus that the same commit also edits. The ledger exists to force that raise.

The gap applies to every rule. A rule with several rows fails only when its verdicts move over a corpus that the change did not edit. The ten rules above are the case that is easiest to meet. Each has one corpus, so an edit of that one corpus is enough to bless any verdict change of the rule. A second recorded corpus for each rule does not close the gap. A change that moves verdicts over one corpus only and edits that corpus still passes.

## Discharge

This record discharges when a bless can refuse a row whose fingerprint and verdicts both moved at the same `VERSION`. The bless must separate the two causes, or refuse. One way is to run the changed rule over the corpus as the ledger recorded it. The bless then refuses when those verdicts differ from the recorded digest. A case makes one commit that moves verdicts and edits the corpus of the row, and requires the bless to refuse it.
