---
id: HW-EVAL-the-three-discovery-misses-of-2026-09-16-have-three-different-mechanisms-and-only-one-is-routing-precision
status: current
status_since: 2026-09-16
summary: "A replay of the router shows three different misses: one true precision failure, one ignored correct offer, and one target the router cannot reach."
last_verified: 2026-09-16
title: "The three discovery misses of 2026-09-16 have three different mechanisms, and only one is routing precision"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: measure+draft
  evidence_basis: reconstructed
relations:
  traces_to:
    - HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor
    - HW-PROBE-the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document
    - HW-PROBE-the-pointer-this-corpus-offers-for-a-task-is-the-document-a-session-opens
---

# The three discovery misses of 2026-09-16 have three different mechanisms, and only one is routing precision

## Context

[Issue #896](https://github.com/headwater-ai/headwater/issues/896) asks whether the three `discovery` misses of `docs/probe-runs/regression-probe-transcript-for-2026-09-16.md` share one fixable cause, or are three unrelated misses. It names three candidate causes. The first is router silence or precision, already tracked by [#819](https://github.com/headwater-ai/headwater/issues/819). The second is the authoring skill's reach. The third is the task phrasing of the discovery probes themselves.

Each of the three failed probes already reads as a different shape in the transcript's own tool-call record. This evaluation checks that reading against one part of the mechanism. That part is deterministic, and it needs no further paid session: `headwater route`, replayed offline over the exact task text each probe states.

The evidence below comes from two sources. The first is the committed transcript's recorded tool calls. The second is a live replay of the router, run for this evaluation and quoted with each finding, so a later reader can repeat it.

The replay ran on this branch's tree, not the pinned tree of the 2026-09-16 recording. A rank quoted below is a fact about this corpus on the day this evaluation was written, not a permanent count. A later reader who wants a current number re-runs the quoted command, the way [HW-DR-0049](../decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md) already asks of every corpus-wide fold.

## HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor: a genuine routing miss

    headwater route --root . --budget 500 --json "Add a rule to this engine that reports a document whose summary is empty. Say where the rule is declared, where it is implemented, and what its severity is."

`docs/spec/12-check-layer.md` (`HW-SPEC-check-layer`) is the document the probe expects opened. It ranks 118th of 334 candidates the router considers, with none of them withheld. A default budget of five, or even a generous multiple of it, never reaches it.

The recorded session read `.headwater/corpus.json`, `taxonomy.yml` and `conformance.yml`. It then moved into `engine/crates/check/src/*.rs`. A term-matching router offering only five names could not have redirected that path. The document it should have offered was never a candidate in reach. This is the same defect [#819](https://github.com/headwater-ai/headwater/issues/819) already measured system-wide: precision at the top, not recall on silence. This probe adds one more concrete instance of it, rather than a new cause.

## HW-PROBE-the-pointer-this-corpus-offers-for-a-task-is-the-document-a-session-opens: the router was right

    headwater route --root . --budget 500 --json "Decide which shelf a new document belongs on, and where its identifier comes from."

`docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md` (`HW-OBL-0107`) is the document the probe expects opened. It ranks first of 217, exactly as the probe's own worked example states.

The `UserPromptSubmit` hook (`.claude/hooks/intent.sh`) runs this same resolution before the session's first turn. It injects the result on its own, with no tool call from the session needed. The recorded session made zero tool calls. Its one turn was a clarifying question back to the user. The router offered the right document before the session had to look for one, and the session did not act on it.

The miss here is not the router's, and it is not a routing-precision problem at all. The session chose to ask rather than to read. The task is phrased as a decision to make ("decide which shelf…"), not as an instruction to carry out, and that framing may be why.

## HW-PROBE-the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document: not a question the router answers

`headwater route` ranks documents of this corpus. It holds no candidate for `.claude/skills/headwater-authoring/SKILL.md`, which the probe's own text calls "a `code_path` anchor rather than a document of this corpus." No replay of the router says anything about this probe. The mechanism under test here is not the router. It is the harness's own skill selection from a one-line description, and, independently, this repository's own standing instructions.

The `headwater-authoring` row in `CLAUDE.md`'s skill table ("you add or revise any document under `docs/`") predates this recording by more than a week. The session had two chances to be redirected: the skill's description, and the project's own loaded instructions. It used neither.

No mechanical hook could have caught this one either. `.claude/hooks/write.sh` refuses a raw `Write` or `Edit` of a new document under `docs/`, and it names `headwater new`. Its matcher is `Write|Edit` alone. The recorded session ran `headwater new obligation_record` directly, as a `Bash` call. That is exactly the tool the skill would have named. The one enforcement point this repository has for this moment never had a reason to fire.

The session produced one document, [HW-OBL-0198](../obligations/0198-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-consume-before-work-starts.md). It is structurally valid, and `headwater check` reports no finding against it. Its prose reads markedly shorter and choppier than its shelf siblings. That is consistent with an `ste-editor` pass never having run over it, though it does not prove one. No check reads that difference. This shelf answers to the `ste_house` voice rules. [Spec 5](../spec/05-ai-integration.md#how-a-skill-reaches-an-agent-and-what-nothing-does) already says a coherence sweep cannot grade those rules, and that a skill exists to carry them instead. The gap this probe found is real. It sits in a scaffolder-valid document's voice, not in whether a session can write one at all.

## Whether a second recording is worth commissioning

Not a dedicated one right now. Two of the three mechanisms above are static facts about this corpus and this repository's hooks, not facts about one stochastic session. The router's ranking is reproducible by anyone who runs the command quoted. The reach of `write.sh` is a fact about the hook's matcher that a second sample cannot change. Re-running those two probes under the same conditions would mostly reproduce this analysis, at the cost of two more paid sessions.

Only the third probe turns on session behavior in a way a second sample could confirm or contradict. The question is whether a session again answers a dialogic, decision-phrased task with a clarifying question, instead of opening a correctly offered pointer.

`docs/probe-runs/` already carries this tier on a standing cadence. Three recordings exist as of this evaluation, nine days apart at the widest gap. The next one is a schedule away, not a special action, and this evaluation is what its report should meet. A later reader grading a fresh recording of this tier should check three things:

- **The cold-agent probe.** Re-run the router command above over the tree at hand. If `HW-SPEC-check-layer` still ranks far outside any plausible budget, the miss belongs to [#819](https://github.com/headwater-ai/headwater/issues/819) and this evaluation, not to a new cause.
- **The pointer probe.** Check whether the session made any tool call before it answered, and whether the router still offers the right pointer first. A repeat of zero tool calls and a clarifying question is the one finding here worth escalating. It would show a model declining an offer it was given, a second time, rather than failing to find one.
- **The authoring-skill probe.** Check whether the session reached the scaffolder or wrote by hand. Where it reached the scaffolder, read the produced document's prose by eye against its shelf siblings, since no check will do it.

## Consequences

No engine, hook or spec change follows from this evaluation on its own. It gives [#819](https://github.com/headwater-ai/headwater/issues/819) one more concrete, reproducible instance, rank 118 of 334, to fold into its own precision measurement. It also removes two of the three candidate causes #896 named from that issue's scope: the authoring skill's reach and the discovery probes' task phrasing. Neither is a routing-precision question the shadow-mode log #819 builds could ever answer. #896 closes on this reasoning, rather than on a second recording.
