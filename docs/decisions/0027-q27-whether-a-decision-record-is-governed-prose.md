---
id: HW-DR-0027
status: draft
status_since: 2026-08-15
summary: A decision record is governed prose, and the overlay binds the house language regime to it. The shelf carries a prose problem rather than a quotation problem, and collapsing all 37 inline quotations moves 64 findings to 63.
last_verified: 2026-08-15
title: "Q27 — Whether a decision record is governed prose"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  governs:
    - .headwater/overlay.yml
---

# Q27 — Whether a decision record is governed prose

## Context

The house language regime of this repository is declared in `.headwater/overlay.yml` and bound to a kind by name. It bound four kinds. `design_spec`, `decision_register`, `obligation_register` and `obligation_record` answered to it. `decision` did not, and no document stated why.

[#205](https://github.com/headwater-ai/headwater/issues/205) measured the gap rather than recalling it. One line was appended to an obligation record and then to a decision record. That line carries a British spelling, a retired term, a hard wrap, a contraction and a semicolon. The obligation record moved the corpus from 27 findings to 31. The decision record moved it from 27 to 27. The same test on `main`, before the first change of this run, gave the same answer, so the gap predates the run.

**The asymmetry is what makes the gap read as drift.** `obligation_register` and `obligation_record` are both bound. `decision_register` is bound and `decision` is not, and the two pairs stand in the same relation. `docs/spec/09-decisions.md` indexes 26 documents, answers to every lexical rule, and none of the 26 answered to one.

**The issue states a case against binding, and it is a real one.** A decision record cites prior art more heavily than any other kind here. [HW-OBL-0086](../obligations/0086-an-inline-quotation-reaches-every-lexical-rule-as-this-authors.md) records that an inline quotation reaches every lexical rule as the author's prose, and that a block quotation reaches none of them. So a count of findings on this shelf is not a count of defects. The issue asks for the findings to be characterized before the ruling is taken, and that is the first section below.

**The population moved while the question stood open.** The shelf held 24 records when #205 was filed and it holds 26 now. The three most recent were drafted by an agent of one run. Each of those carries `warrant: asserted` and no acceptance, so no human has read it, and no lexical rule read it either.

## Decision

**A decision record is governed prose.** `.headwater/overlay.yml` binds `kinds.decision.language` to `ste_house` and the lock records it. Four reasons follow. The first two are measurements rather than arguments.

**First, the shelf carries a prose problem and not a quotation problem.** The binding reported 64 findings across 26 records. Every one of them is `language.controlled.not_met`. `language.retired_term.used` and `language.source_form.not_met` reported nothing at all. Of the 64, one names a semicolon and 63 name a sentence past the word limit.

The quotation reading was then measured directly rather than reasoned about. Every inline quotation of five characters or more on the shelf was replaced by one word, which is 37 quotations across the 26 files. The finding count moved from 64 to 63. Five of the 64 findings fall on a sentence that carries a quotation mark of any kind. So the reading HW-OBL-0086 holds accounts for one finding in 64 here. The case against binding rested on a share of this shelf that the shelf does not have.

**Second, the binding costs no gate work.** All 64 findings are advisory and none is an error, so `headwater check --strict` exits 0 over the whole tree with the binding in place. No directive was written and no adoption pair was added. Four classes of finding are errors, and they are a contraction, a British spelling, a hard-wrapped block, and a retired term that names a replacement. Across 26 records this shelf produces none of the four.

The instrument was proved on the treatment arm rather than assumed. The five-defect line of #205 was appended again to `docs/decisions/0001-implementation-language.md`, with the binding in place. It reported four findings. The British spelling and the hard wrap are errors, the retired term and the semicolon are warnings, and the strict run exits 1. Removing the line returns the tree to exit 0.

**Third, the findings land where no reader has been.** 60 of the 64 fall on the five most recent records, which are `0022` through `0026`. The 21 records before them carry 4 findings between them. Three of the five were drafted by an agent, in the run that asked this question. The unchecked prose of this corpus and its least reviewed prose are the same documents, and the binding closes one of the two halves.

**Fourth, a record of a register is prose of this repository in the way its register is.** [The decision-record tradition](../taxonomies/decision-record/doctrine.md) states the shape of the repair. The bundle declares a voice regime for `decision` and leaves a controlled profile to the adopter that wants one. This repository wants one, and the overlay is where an adopter says so.

## Consequences

**What a writer meets.** A decision record now answers to the six rules that read prose. A hard wrap or a British spelling in one stops a commit. `headwater check --fix` writes the spelling and leaves the wrap, because the source-form rule carries no patch. The 64 findings above are advisory and nothing works them down on a clock.

**Why no adoption pair was written.** The `adoption` block of the lock holds a `(document, rule)` pair so that a pending finding does not fail a strict run. None of these 64 fails a strict run. So a pair would suppress nothing, and it would put an expiry on work that no gate waits for. What this leaves is 64 advisory lines that a reader of `headwater check` meets on every run.

**Which kinds stay unbound, and what each one would cost.** Each was bound alone against the same tree, resolved, and measured.

| kind | documents | findings added | of which errors |
|---|---|---|---|
| `evaluation` | 15 | 432 | 0 |
| `review_record` | 5 | 202 | 21 |
| `probe` | 3 | 8 | 0 |
| `review_prompt` | 1 | 8 | 3 |
| `probe_result` | 0 | 0 | 0 |
| `probe_transcript` | 0 | 0 | 0 |
| `specification` | 0 | 0 | 0 |

The line this ruling draws is the one the taxonomy already carries. Every kind the language regime now binds also binds the `declarative` voice regime, and the language set is the declarative set less four. `docs/reviews/` is exempt by design, which `CLAUDE.md` states in three places, and it holds `review_prompt` and `review_record`. The words of a probe are the instrument of a measurement, so a rewrite to a word limit changes what the probe measures. Three of the seven rows hold no document in this corpus, so binding one would be a declaration with no subject.

**`evaluation` is the row this reason does not settle, and it is filed rather than decided.** An evaluation binds the `narrative` voice regime and every bound kind binds `declarative`, which is a real line and is the one adopted here. It is a line about voice rather than about lexical rules, and nothing states that a narrative document is free of a word limit. Its 432 findings have the same shape as this shelf, with 422 over the word limit and 10 on a semicolon. Nobody has read them. [#247](https://github.com/headwater-ai/headwater/issues/247) carries the characterization that this ruling required of itself and did not perform there.

**What reopens this.** Two conditions, and each one is a thing that can happen.

The first is a record that has to quote at length. The measurement above covers one shelf at one time, and a record that quotes a standard for a paragraph would move it. Two remedies come before a rewrite. A block quotation reaches no lexical rule at all, and a directive naming `reason=false_positive` states the exception where a reader meets it. A record that can use neither is the case that reopens this, and HW-OBL-0086 is where the reading itself is held.

The second is an advisory count that nobody works down. 64 advisory findings are a report rather than a rule. A later measurement that finds the count higher, with no finding on this shelf repaired, is evidence that the binding reports without governing. The question then is whether a word limit belongs on this shelf at all.

**What this record does not settle.** Whether an agent may write the acceptance stamp of a document it drafted is [HW-OBL-0108](../obligations/0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md). Whether `warrant: proposed` is a value of that facet at all is [HW-OBL-0125](../obligations/0125-nine-documents-state-a-warrant-the-closed-set-does-not-hold-and-no-check-reads-one.md). This ruling closes the half of the problem a machine can close.

**This record is checked prose and unaccepted prose.** Every lexical rule of this repository read it, and no human has.
