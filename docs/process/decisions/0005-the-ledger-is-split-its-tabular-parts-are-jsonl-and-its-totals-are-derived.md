---
id: HW-PD-0005
status: current
status_since: 2026-09-07
summary: "The run ledger is one directory of small files rather than one markdown file, its per-iteration log and its findings are JSONL read by the line, every total is derived and never stored, and a database is deferred until a cross-run question asks for one."
last_verified: 2026-09-07
title: "The ledger is split, its tabular parts are JSONL, and its totals are derived"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-fable-5-1
  activity: measure+draft
  evidence_basis: evidenced
---

# The ledger is split, its tabular parts are JSONL, and its totals are derived

## Context

The ledger was one markdown file at `~/.claude/headwater-build-order-ledger.md`, and by 2026-08-24 it was 713 KB across 3,155 lines. Nothing reads it but a model. A design put the parent's doctrine at the head of that file so that it reloads on a turn already paid. That design ran into the file's own shape. A header cannot be read without the body behind it, and pull request #685 measured that a large read is worse than its size, because every later turn re-reads it.

Run 22 measured the other hazard. It spent 31,568 tokens and 2.8 million token-reads writing the ledger through forty `cat >>` heredocs, because a shell argument is context as permanent as any result. Any store whose writes go through a shell command inherits that cost.

[HW-DR-0049](../../decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md) rules that a recorded artifact holds one record per entity and derives every total. The ledger's log carried a net issue delta as a hand-maintained number, which is a stored fold.

## Decision

The ledger is a directory under the run directory, and each part is read by whoever needs it and nobody else.

- `doctrine.md` holds the parent's doctrine block alone, at most ten numbered lines and at most 600 tokens, so it reads without any other part.
- `log.jsonl` holds one object per iteration: the iteration, the issue, the pull request, the merge commit, the verdict, what verification proved, and the counts opened and closed. Any total across iterations is derived by `jq` at the moment it is wanted and is never written.
- `findings.jsonl` holds one object per open finding, so the findings that touch one issue are a query at dispatch.
- `lessons.md` and `decisions.md` stay prose, because the owner reads them.

The integrator writes the log line at the end of each merge, with `Edit` and never through a shell argument. The parent reads `doctrine.md` and the last few lines of `log.jsonl` on the integration-completion turn it already pays, and never the whole log.

A database is deferred and not refused. `jq` is a developer dependency and not one the engine ships, so it is not what refuses it. What refuses it now is that every write would be a shell argument, and that the prose parts are for a human to read. The condition that reopens this record is a cross-run question, such as how median subagent runtime moved across six runs, and JSONL imports cleanly then.

## Consequences

The existing ledger is imported once into the new shape and the archive directory is left as it is. A reader after a compaction gets the doctrine in under 600 tokens rather than in the 713 KB that held it. The net issue delta becomes `jq` over the log, and it cannot go stale because it is not stored.

The run directory lives under `git rev-parse --git-common-dir`, so every worktree reaches it and no path assumes a home directory. It is not committed, and it is removed when the run closes.
