---
id: HW-OBL-0008
title: "An adoption payload has a first reading and no elapsed time"
status: current
status_since: 2026-08-11
waiting_on: measurement
last_verified: 2026-08-28
summary: "Q12 claims that an adoption payload shrinks, and this repository holds one task that is a day old."
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0012
---

# An adoption payload has a first reading and no elapsed time

## Context

[Q12](../spec/09-decisions.md#q12--migration-path-for-an-existing-corpus) rules that adoption is a migration from no taxonomy and that `headwater infer` computes the payload. An adoption payload should shrink.

## Obligation

The instrument is the remaining pair count over time, against the fraction of payloads that reach zero before the expiry.

## Discharge

**The instrument exists now, and its first reading has a start date and no elapsed time.** `headwater infer` writes a payload, and every task in it carries an owner and an expiry. `headwater check` then reports the open pairs, the closed pairs, and the findings they hold. It reports them on every run, which is what makes a payload that does not move visible before its expiry. What no engine can supply is time.

**The series has a store now, and it holds one reading.** `headwater taxonomy audit --record` appends one line per invocation to `.headwater/adoption.jsonl`, and [spec 7](../spec/07-distribution-and-federation.md#what-the-adoption-store-records-and-what-it-refuses-to) states what a line holds and what it refuses to hold. The report states the payload of the run. It states the fraction of tasks that stood at zero on or before their expiry, and the first and the last date the store holds. Over one reading it states a value and no trend.

This repository declares one task, `AD-1`, which holds one pair and expires on 2027-06-30. That pair raises no finding today, so the first recorded reading states the task at zero pairs open, one closed, and no findings held. A payload standing at zero on the only reading anybody took is a payload with no history. What the series adds is the elapsed time between two readings, and no engine supplies it.

The earlier evidence stays as weak as it was. `.ste-lint-baseline.json` grandfathered 65 violations when the check landed, and it held 2 when the linter retired. Most of that fall came from prose that a later commit rewrote for other reasons, and from a defect in the checker. The last of it came from retiring a rule rather than from working a list. Of the two that remained, one became the pair above. The other stopped being a finding at all. It named a semicolon inside a block quotation, and the engine hands a quoted block to no lexical rule. So the baseline shows that a text-keyed ratchet does not run backwards, and it still shows nothing about whether anyone works a debt list. `AD-1` is the first thing in this repository that can answer that, and it can only answer it later.
