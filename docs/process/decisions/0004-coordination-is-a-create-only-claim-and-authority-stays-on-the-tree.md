---
id: HW-PD-0004
status: current
status_since: 2026-09-07
summary: "Workers coordinate through create-only claim files that every worktree can reach and the parent never relays, while the merge veto and a re-verification stay messages from the parent, because a peer's message carries no owner authority."
last_verified: 2026-09-07
title: "Coordination is a create-only claim, and authority stays on the tree"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-fable-5-1
  activity: measure+draft
  evidence_basis: evidenced
---

# Coordination is a create-only claim, and authority stays on the tree

## Context

The orchestrator of the measured run sent 67 messages, and they were of two kinds. Most were authority: a branch sent back with a named defect, or a delta sent for re-verification. A few were coordination: a warning to one worker that another was about to regenerate the same derived artifact. The command already ruled that a worker claims its issue through the board and never through a coordinator, because a coordinator holding N workers' states re-reads its whole context on every message it routes. The collision warnings were that failure in miniature, and the parent was relaying them by hand.

[HW-DR-0054](../../decisions/0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md) already holds a claim store for identifiers, one file per claim, written once, with an `add/add` conflict when two branches claim one value. That shape is right for allocation and wrong for scheduling, because a merge-time conflict is too late to order the merges and the merge driver on `.gitattributes` already refuses a two-sided regenerate. The value of knowing that two branches touch one artifact is at dispatch and at merge ordering.

One session demonstrated the other half of this ruling. It declined to edit `CLAUDE.md` on a peer session's reasoning, because a peer's message carries no owner authority, and the refusal was correct. In a mesh of agents every message has to be adjudicated for provenance. On a tree, authority flows down and provenance is never in question.

## Decision

Coordination moves to a claim file under the run directory. That directory lives under `git rev-parse --git-common-dir`, so every worktree reaches it with one command and no socket. A claim is a file opened create-only, with the `O_EXCL` flag that a `set -C` redirect carries in the shell. It is not made with `mkdir`, although `mkdir` is the textbook primitive. On a host whose coreutils are the uutils rewrite, two racing `mkdir` calls on one path both succeeded in 17 of 20 races. That was measured on 2026-09-07, and a sequential second call was refused. A claim file is never empty, because it holds the issue and the branch that made it. The adjudicator declares the footprint of a change, which is the derived artifacts it will regenerate. The parent claims each one on the turn it reads that report.

A second claimant on an artifact does not fail. It records the claim it waits on, and the integrator merges in footprint order: the widest footprint first, and no branch before every branch it waits on has merged, rebased and regenerated. The integrator never adds rebase work to a branch that is still running, because a finished branch waiting costs nothing and a running branch redoing its derives costs a full pass.

Authority stays on the tree. The veto, and the instruction to re-verify a delta, remain messages from the parent to a named agent. No file replaces them, because what they carry is a decision only the parent may make. The committed identifier claim store is unchanged.

## Consequences

The parent relays nothing. A collision becomes a line in a report the parent already reads, and merge order becomes a rule the integrator applies rather than a judgment the parent makes under pressure. A fixture holds the claim's atomicity: two concurrent claimants over a scratch common directory, one of which owns and one of which records what it waits on, with no empty claim between them.

A worker that dies leaves its claims in place, and the run directory is deleted when the run closes, so a stale claim lives at most one run. A wait a worker starts must not outlive the worker, and every agent definition says so, because the measured run ended with thirty orphaned waits and the oldest polled for eleven hours.
