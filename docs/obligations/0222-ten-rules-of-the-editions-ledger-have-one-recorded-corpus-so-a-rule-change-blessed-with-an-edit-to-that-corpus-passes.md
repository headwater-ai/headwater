---
id: HW-OBL-0222
status: discharged
status_since: 2026-11-20
summary: "A bless of the editions ledger re-recorded any row whose corpus moved, so a rule change blessed with an edit to its corpus passed. A replay of the recorded corpus now refuses it."
last_verified: 2026-11-20
title: "Ten rules of the editions ledger have one recorded corpus, so a rule change blessed with an edit to that corpus passes"
waiting_on: build
---

# Ten rules of the editions ledger have one recorded corpus, so a rule change blessed with an edit to that corpus passes

## Context

[#1105](https://github.com/headwater-ai/headwater/issues/1105) added `engine/crates/check/fixtures/editions.ledger`, which `engine/crates/check/tests/editions.rs` reads. Each row pins one rule over one recorded corpus. The row holds the rule `VERSION`, a fingerprint of the corpus, a digest of every verdict and an instance count. Under `HEADWATER_BLESS=1`, the test re-records a row when its `VERSION` or its corpus moved. It refuses a row when the verdicts moved over the same corpus at the same `VERSION`.

A bless re-records the rows of a raised `VERSION`, and that is correct. For example, `identifier.claim.stale` went from `VERSION` 2 to 3 in `f910f601` ([#1502](https://github.com/headwater-ai/headwater/pull/1502), Refs [#1366](https://github.com/headwater-ai/headwater/issues/1366)), and the bless re-recorded its 7 rows at that time.

Run `20260926-1327` found this gap. The product owner ruled it Record, because the ledger has no reader outside this repository.

## Obligation

On 2026-09-27, the ledger held 31 rules. Ten of them have one row, so one corpus holds each of the ten:

- `check`: `facet.value.blank`, `language.controlled.not_met`, `language.retired_term.used`, `language.source_form.not_met`, `section.required.missing` and `voice.forbidden_construction`.
- `terminal-dependency`: `lifecycle.dependency.on_terminal`.
- `state-set-twice`: `lifecycle.state.set_twice`.
- `editions/governs-suspect`: `relation.target.suspect`.
- `acceptance-criterion-proven`: `relation.target.verification.suspect`.

The function `judge` in `editions.rs` refuses a row only when its verdicts moved and neither its fingerprint nor its `VERSION` moved. When the fingerprint moved, a bless re-records the row, whatever its verdicts did. So a change passes without a `VERSION` raise when each verdict it moves is over a corpus that the same commit also edits. The ledger exists to force that raise.

The gap applies to every rule. A rule with several rows fails only when its verdicts move over a corpus that the change did not edit. On 2026-11-20, the ledger held 39 rules, and 9 of them had one row. The ten rules above are the case that is easiest to meet. Each has one corpus, so an edit of that one corpus is enough to bless any verdict change of the rule. A second recorded corpus for each rule does not close the gap. A change that moves verdicts over one corpus only and edits that corpus still passes.

## Discharge

Discharged on 2026-11-20 by PR_REF, slice C of [#1494](https://github.com/headwater-ai/headwater/issues/1494). A row can move its fingerprint at an unchanged `VERSION`. When it does, `editions.rs` rebuilds that corpus from git, at the commit that last wrote the ledger. It requires the fingerprint of the rebuilt corpus to equal the recorded one, and it runs the current binary over that corpus. When the result is the recorded digest and instance count, the bless re-records the row. When the result is different, the test refuses the row in both modes and asks for a raised `VERSION`. When git cannot rebuild the corpus, the test refuses the row in both modes and says why. The case `a_moved_corpus_whose_verdicts_also_moved_over_the_recorded_corpus_fails_and_bless_keeps_the_row` is the commit that this record describes. The cases `a_corpus_rebuilt_from_head_reaches_the_rows_head_records` and `replays_answers_a_moved_corpus_from_the_corpus_head_records` hold the replay over git.

The text below is the condition as it was recorded. This record discharges when a bless can refuse a row whose fingerprint and verdicts both moved at the same `VERSION`. The bless must separate the two causes, or refuse. One way is to run the changed rule over the corpus as the ledger recorded it. The bless then refuses when those verdicts differ from the recorded digest. A case makes one commit that moves verdicts and edits the corpus of the row, and requires the bless to refuse it.
