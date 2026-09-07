---
id: HW-PD-0006
status: current
status_since: 2026-09-07
summary: "The build order keeps the name next-run and gains a run directory it writes as it goes, so that a compaction, a crash or a second invocation resumes from a doctrine block of ten lines and a log read by the line, with each stage's model declared in its own frontmatter."
last_verified: 2026-09-07
title: "The entrypoint keeps its name and becomes a resumable run"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-fable-5-1
  activity: measure+draft
  evidence_basis: evidenced
---

# The entrypoint keeps its name and becomes a resumable run

## Context

A rename of `/next-run` was proposed, on the ground that the name says when and how much rather than what. The name was not what was wrong. The command was the orchestrator's whole brain and its only memory, and both of those degrade over a run. The measured orchestrator compacted five times in twenty hours, and its longest gap, 82.8 minutes, ended at a compaction. The orchestrator is the one agent that cannot be reloaded: a subagent gets its definition fresh on every dispatch, and the orchestrator runs in the human's own session with no reload path.

Re-reading the doctrine at the top of each iteration was proposed and rejected, because a re-read is itself a parent turn and it grows the context every later turn pays for. Under [HW-PD-0003](0003-a-dispatch-pays-when-it-retires-more-parent-turns-than-it-costs.md) that is the expensive direction.

An agent definition carries `model` in its frontmatter. The maintainer already runs on one model and the product owner on another, so per-stage model assignment is declarative today and needs no prose to carry it.

## Decision

The command keeps its name. It owns three things and delegates the rest: the doctrine block, the loop, and the veto. Selection is delegated to a queue agent that writes an ordered file of issue and footprint. Every stage is delegated to a definition.

A run writes a directory under `git rev-parse --git-common-dir` from its first iteration, and the ledger of [HW-PD-0005](0005-the-ledger-is-split-its-tabular-parts-are-jsonl-and-its-totals-are-derived.md) lives there. A compaction, a crash or a second invocation reads that directory and continues.

The doctrine block is at most ten numbered lines and at most 600 tokens, and it is the first file of the run directory. The parent reads it on the integration-completion turn it already pays, so it re-enters context without a re-read turn. The parent's decisions are keyed to fixed report blocks such as `VERDICT: PASS` and `WAITS-ON:`, so a degraded parent acts correctly by matching a block rather than by recalling a rule. The canonical copy of the doctrine lives in the repository, and a fixture holds the copy in the command byte-identical to it.

Each stage's model is its frontmatter. Adjudication, construction and verification run on Opus. Integration and the queue run on Sonnet. Verification stays on Opus on the argument that a cheaper verifier of a stronger builder is credulous where it should be diligent, and the frontmatter is the one-line change by which a later run measures the alternative.

`/next` stays as the single-iteration form and becomes a stub over the same definitions and skills.

## Consequences

A run survives what ends an orchestrator today. The doctrine that survives is small because it must be: a compaction summary preserves a short numbered list where it does not preserve a narrative, and that claim is the one untested assumption this design rests on. The next measurement takes it.

The name stays, so nothing that cites `/next-run` moves. The file behind it is under 8 KB rather than 72 KB, and everything it carried has an owner under [HW-PD-0001](0001-orchestration-prose-has-one-owner-per-sentence.md).
