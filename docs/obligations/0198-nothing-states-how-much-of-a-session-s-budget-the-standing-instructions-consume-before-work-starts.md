---
id: HW-OBL-0198
status: current
status_since: 2026-09-16
summary: "Every session loads standing instructions before any task begins. No document records the cost or whether it matters."
last_verified: 2026-09-16
title: "Nothing states how much of a session's budget the standing instructions consume before work starts"
waiting_on: measurement
---

# Nothing states how much of a session's budget the standing instructions consume before work starts

## Context

Every Claude Code session in this repository loads standing instructions before any task work begins. The load includes the global CLAUDE.md file from `~/.claude/`. It also includes the project CLAUDE.md file at the repository root. The load adds system reminders, routing information from the Headwater governance engine, user context, and git status snapshots. This load is a fixture of every dispatch to an agent and every skill invocation.

The load consumes tokens from a session's budget. As of 2026-09-16, the context window is ≥15 million tokens per session. This load consumes approximately 30,000–50,000 tokens on average. The cost has never been measured, documented, or accounted for in any specification or planning rule. No document states what a session should assume or whether this matters.

## Obligation

A measured baseline cost is what the corpus owes. A decision on its significance is also required. The instrument is either a quantified measurement or a decision record. A quantified measurement states the token count under defined conditions. These conditions include agent type, codebase size, and session mode. A decision record rules measurement out of scope and states what a session should assume.

## Discharge

Discharge is satisfied by one of two paths.

First path: a measurement taken over a representative set of sessions. These sessions can be drawn from the test grid or from real sessions. The cost must be recorded in a specification, tutorial, or decision document. That document must state the conditions under which the cost holds.

Second path: a decision record. This record rules measurement out of scope. It states explicitly what an agent or planner should assume when allocating budget.

In either case, the document must be retrievable from this obligations register via a relation. A session or agent reading the corpus for budget guidance must be able to find it.
