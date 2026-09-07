---
id: HW-DR-0062
status: current
status_since: 2026-09-07
summary: "The unit of orchestration cost is one parent turn at full context, so a dispatch is worth making only when it retires more parent turns than the one it spends, which the integrator passes and a fresh agent per poll fails."
last_verified: 2026-09-07
title: "A dispatch pays when it retires more parent turns than it costs"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-fable-5-1
  activity: measure+draft
  evidence_basis: evidenced
---

# A dispatch pays when it retires more parent turns than it costs

## Context

Pull request #685 measured the orchestrator's polling. Seventy-six `gh pr view` and `gh pr list` calls returned 25 KB in total and occupied 77 turns and 21.3 million cache reads, which is 10.3% of that parent's 207 million for the run. The payload was about 6.3 thousand tokens, so the carrier cost was roughly 3,400 times the payload. The parent re-reads its whole context on every turn, and that re-read is the cost. The output of the call is not.

The obvious remedy fails the same measurement. A fresh agent dispatched to make one poll is a parent turn at the same context, plus a prompt, and it returns a report that enters the context too. Delegating a cheap call does not save the turn that delegates it.

An earlier design argued that decomposing the command into agent definitions reduced parent turns. It does not. It reduces the size of the context each turn re-reads, which is a different quantity. [The evaluation](../evaluations/the-build-order-as-a-multi-agent-system.md) records both.

## Decision

The unit of cost for the orchestrator is one parent turn at the parent's full context. A dispatch is worth making only when it retires more parent turns than the one it spends. A change to the run is stated against three quantities, and it claims only the ones it serves: fewer parent turns, a smaller parent context, and a smaller context per dispatched agent.

Under that rule the integrator is one agent per merge, dispatched on the verification-completion turn the parent already pays. It replaces about six parent turns per merge: the view, the checkout, the build, the regenerate, the merge and the cleanup. It is a depth-one slot, because every merge touches the one main checkout, the one `engine/target` and `origin/main`, and two of them running at once would check out `main` in one directory together. It is a fresh agent each time and never one long-lived agent, because a long-lived integrator accumulates every merge it ran and compacts, which is the parent's own failure one level down.

The parent runs no `gh` and no `cargo build`. Pull request state arrives inside the verifier's report, board state arrives as a queue file, and every build belongs to the integrator or to a worker's own worktree.

## Consequences

A parent that finds itself making a shell call asks first what turn the call is riding. A call on a turn already being paid is free. A call that is its own turn is the most expensive way to get one line.

Repetition counting does not find this class of cost. The 76 calls covered 43 pull requests at about one and a half each. `tools/run-census.sh` groups a session's shell calls by purpose and prints turns and cache reads per group, and that is how the next instance is found.

A stale binary in the shared checkout is a correctness failure and not a throughput one. The integrator builds before it regenerates on every merge, at one profile, so the artifact it writes and the check that reads it come from the same engine.
