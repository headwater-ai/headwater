---
id: HW-PD-0001
status: current
status_since: 2026-09-07
summary: "Every sentence of orchestration prose has exactly one home, decided by who must obey it and whether it changes per dispatch, so the command holds only what the parent decides and each agent loads only what it must obey."
last_verified: 2026-09-07
title: "Orchestration prose has one owner per sentence"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-fable-5-1
  activity: measure+draft
  evidence_basis: evidenced
---

# Orchestration prose has one owner per sentence

## Context

`.claude/commands/next-run.md` grew to 67 KB, about 16,850 tokens, over eleven runs. The whole of it loaded into the orchestrator's context for the whole of a run. Half of it was text destined for a subagent's prompt: the seven parts of an iteration's prompt, and the adversarial verification checklist. The parent carried that text on every turn and then wrote it out again into each dispatch, so it was paid twice.

Two failures on record follow from that shape. A rule the parent obeyed and did not propagate cost 21% of one run, because the parent was the only holder of the rule and had to remember to paste it. Skill files drifted past rulings unchecked, because the fixtures that held them asserted only engine-verb claims. [The evaluation](../../evaluations/the-build-order-as-a-multi-agent-system.md) records the third failure: the parent compacted five times, and its doctrine was prose of a length a compaction summarizes rather than preserves.

An agent definition is selected by name and loads on every dispatch. A skill is selected by a model reading a description, which this repository treats as a measurement rather than a property. A narrow definition that names a skill to invoke is a reliable draw where a broad session is not.

## Decision

Where a sentence of orchestration prose lives is decided by these tests, applied in order, and the first that matches wins.

1. A check reads it. It lives in the taxonomy or the overlay, and prose cites it in one line.
2. The parent must obey it on every turn. It lives in the doctrine block, which is at most ten numbered lines and at most 600 tokens.
3. One stage must obey it. It lives in that stage's agent definition.
4. Two or more stages must obey it, or the single-iteration command must too. It lives in a skill, and each definition that needs it names the skill in an "invoke before you begin" line.
5. Every agent must obey it. It lives in `CLAUDE.md`, and nothing else goes there.
6. It changes per dispatch. It lives in the dispatch template.
7. Only the parent decides on it. It lives in the command.
8. Nobody must obey it. It lives under `docs/` through `headwater new`, cited once.

One further test applies to every sentence that passes the others. Ask who actually performs the action the sentence constrains. Prose addressed to an agent fails silently when the writer is the harness, and a sentence whose performer is not its reader needs a check rather than a reader.

## Consequences

The command shrinks to the doctrine block, the loop, the veto and the dispatch template. Each stage's static instructions live in its definition and load fresh on every dispatch, so they survive the parent's compaction. The verification bar becomes a skill because the verifier, the integrator and the single-iteration command all read it, and two definitions cannot include one file.

A rule that lives in two places diverges silently. A fixture suite holds the resolvable part of that. Every `subagent_type` names a definition, and every skill an agent names exists. Every decision identifier cited under `.claude/` exists and is not superseded. No heading of the verification bar appears in a second file. No fixture can hold whether an agent obeyed a sentence, and this record does not claim one can.

The measurement and rationale that made up 28% of the command move to the evaluation this record cites. A future reader who wants to know why a rule exists reads the record, and the agent that must obey it reads only the rule.
