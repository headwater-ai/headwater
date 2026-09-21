---
id: HW-PD-0007
status: current
status_since: 2026-09-20
summary: "A wait that might outlive the five-minute prompt-cache lifetime is wrapped in a timeout under it and re-issued on return, so a background wait still ends the turn without paying to rewrite a context the cache would otherwise have kept warm."
last_verified: 2026-09-21
title: "A background wait caps below the cache lifetime and re-issues itself"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: measure+draft
  evidence_basis: evidenced
---

# A background wait caps below the cache lifetime and re-issues itself

## Context

Session `9ab3be93`, a `/next-run 6 --parallel 2` run measured on 2026-09-20, spent $14.85 of a $130.73 total, 11.4%, on thirteen turns. Each turn read a background-task notification after its own wait had run longer than the prompt cache holds a copy of a subagent's context, about five minutes. A cache write costs 1.25 times the input rate, and a cache read costs 0.1 times it. Each of the thirteen turns therefore paid roughly twelve and a half times what a warm read would have cost, to carry the same context back in. The three affected builders rewrote 3.72M, 1.48M and 0.74M tokens across six, five and two such gaps each.

The parent that dispatched them paid none of this cost, because its cache holds a copy for one hour, not five minutes. A count on 2026-09-21 read the `cache_creation` field of every request in 17 parent transcripts and 94 subagent transcripts of this repository. Every parent write went to `ephemeral_1h_input_tokens`, and every subagent write went to `ephemeral_5m_input_tokens`. Of 87 parent turns that woke after a gap of five to sixty minutes, none paid a rewrite. After the same gap, 33 of 35 subagent turns paid one. So this decision binds a subagent, and the parent waits by ending its turn.

`hw-run-policy` already rules that a wait blocks rather than polls. It also rules that a wait long enough to reach the harness's ten-minute foreground cap starts in the background instead, runs to completion however long that takes, and is never re-issued once started. Both rules answer a measured cost of their own: 85 status-file checks in five minutes burned 29% of one run, and 47 foreground waits that reached the cap burned 7.8 hours of another. Read together, though, the two rules tell an agent that a wait of any length ends its turn once and stays silent until one notification. That is exactly the shape that pays the cache-write price above. Neither rule names a bound on how long that silence may run.

## Decision

A wait that might outlive the five-minute cache lifetime is wrapped in a timeout under it, `timeout 240 sh -c 'until ...; do sleep 30; done'`, so the wrapped loop always exits inside four minutes, on its own condition or on the timeout. On the timeout, the agent re-issues the same bounded call and ends its turn again. On the condition, the wait is over. Blocking and backgrounding stay as they are: each bounded call is still one blocking loop, started with `run_in_background: true`, and ended before the agent that started it exits.

Re-issuing this kind of call is not the case `hw-run-policy` already forbids. That case is a loop the harness itself moved to the background at the ten-minute cap, and that loop is still running. A bounded call that hit its own four-minute timeout has genuinely stopped, so a fresh one is a new wait rather than a duplicate of the old one. `.claude/hooks/wait.sh` states the five-minute bound and this remedy inside the text of its ten-minute-cap refusal, so an agent that meets that refusal for the first time learns both costs in one reading.

## Consequences

`hw-run-policy` carries one rule for both ends of the cost, in place of the two that covered one apiece. `tools/run/run-census.sh` reports the expiry class directly: the turns whose cache write outweighed their cache read after a gap past the lifetime, the tokens they rewrote, and what that cost at Sonnet 5's cache-write rate. A fixture pins that reading against a recorded transcript, so the next run answers to a number rather than to this record. [The evaluation](../../evaluations/the-build-order-as-a-multi-agent-system.md) carries the $14.85 figure beside the polling finding it already holds, because both spend the same unit of cost, a turn at full context, on carrying something the agent already had.

This decision does not move the wait off the agent that holds the large context, and it does not ask for a longer cache lifetime. Both stay open. The measured saving from bounding and re-entering, about $3.84 against the $14.85 paid, does not depend on either change. A future measurement that finds the chained re-entry itself costing more than the four-minute bound saves would reopen this decision rather than confirm it.
